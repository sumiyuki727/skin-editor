use super::*;

/// Latest freeze cycle, retained as diagnostics after resuming. Counts only
/// previously admitted future inputs actually cancelled by this cycle.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FreezeInfo {
    pub source: RunSource,
    pub boundary: MapTime,
    pub cancelled_inputs: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FreezeReport {
    Pending {
        boundary: MapTime,
        completed_through: Option<MapTime>,
        reason: PendingReason,
        steps: u16,
        cancelled_inputs: usize,
    },
    Frozen {
        at: MapTime,
        steps: u16,
        cancelled_inputs: usize,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RearmReport {
    Observed { newly_armed: bool },
    Repeated,
}

impl SyntheticRun {
    /// Close admission immediately without allocating a request ID or epoch.
    /// The boundary may still need sealing before `settle_freeze` can work.
    /// Authenticated queued work through the boundary keeps its eligibility.
    pub fn request_freeze(&mut self, source: RunSource, boundary: MapTime) -> Result<(), RunError> {
        self.check_source(source)?;
        if self.failure.is_some() {
            return Err(RunError::Faulted);
        }
        if self.mode != ControlMode::Running {
            return if self.freeze_info.expect("closed boundary").boundary == boundary {
                Ok(())
            } else {
                Err(RunError::Busy)
            };
        }
        if boundary < self.start
            || self.last_committed.is_some_and(|old| boundary < old)
            || self.completed().is_some_and(|old| boundary < old)
        {
            return Err(RunError::InvalidTarget);
        }
        self.mode = ControlMode::Freezing;
        self.freeze_info = Some(FreezeInfo {
            source,
            boundary,
            cancelled_inputs: 0,
        });
        self.armed.fill(false);
        self.armed_from.fill(None);
        self.active = None;
        self.work = match self.work {
            Work::Search(draft) | Work::Commit(draft) if draft.input.at > boundary => Work::Select,
            Work::Publish { at, .. } if at > boundary => Work::Select,
            work => work,
        };
        Ok(())
    }

    /// Settle an already closed prefix under the ordinary work/event limits.
    /// No ordinary request token or queue credit is needed to keep stopping.
    pub fn settle_freeze(
        &mut self,
        source: RunSource,
        budget: WorkBudget,
    ) -> Result<FreezeReport, RunError> {
        self.check_source(source)?;
        if self.failure.is_some() {
            return Err(RunError::Faulted);
        }
        if budget.get() > self.limits.max_work {
            return Err(RunError::BudgetLimit);
        }
        if self.mode == ControlMode::Running {
            return Err(RunError::NotFreezing);
        }
        let info = self.freeze_info.expect("closed boundary");
        if self.mode == ControlMode::Frozen {
            return Ok(FreezeReport::Frozen {
                at: info.boundary,
                steps: 0,
                cancelled_inputs: info.cancelled_inputs,
            });
        }
        if self.sealed.is_none_or(|sealed| sealed < info.boundary) {
            return Err(RunError::BeyondSeal);
        }
        let report = self.pump(info.boundary, budget)?;
        let cancelled_inputs = self.freeze_info.expect("closed boundary").cancelled_inputs;
        match report {
            PumpReport::Complete { steps } => {
                self.mode = ControlMode::Frozen;
                self.held.fill(false);
                Ok(FreezeReport::Frozen {
                    at: info.boundary,
                    steps,
                    cancelled_inputs,
                })
            }
            PumpReport::Pending { reason, steps } => Ok(FreezeReport::Pending {
                boundary: info.boundary,
                completed_through: self.completed(),
                reason,
                steps,
                cancelled_inputs,
            }),
        }
    }

    /// Resume at the same completed map time. All physical slots are unknown
    /// until a fresh release fact/edge in the new input epoch arms that slot.
    pub fn resume(&mut self, source: RunSource) -> Result<RunSource, RunError> {
        self.check_source(source)?;
        if self.failure.is_some() {
            return Err(RunError::Faulted);
        }
        if self.mode != ControlMode::Frozen {
            return Err(RunError::NotFrozen);
        }
        if self.inspection.is_some() {
            return Err(RunError::Busy);
        }
        let input = self
            .source
            .input
            .checked_next()
            .ok_or(RunError::CounterExhausted)?;
        if self.next_request.is_none() || self.step == u64::MAX {
            return Err(RunError::CounterExhausted);
        }
        debug_assert_eq!(self.input_len, 0);
        self.source.input = input;
        self.highest_input = 0;
        self.replay.fill(None);
        self.replay_next = 0;
        self.held.fill(false);
        self.armed.fill(false);
        self.armed_from.fill(None);
        self.cancelled_sound_through = self.issued_through;
        self.mode = ControlMode::Running;
        if let Some(index) = self.ready {
            self.frames[index].source = self.source;
        }
        Ok(self.source)
    }

    /// An untimed, current-source capture fact. It cannot clear consumed held
    /// state or grant settlement eligibility to externally supplied old inputs.
    pub fn observe_release(
        &mut self,
        source: RunSource,
        fact: ReleaseObservation,
    ) -> Result<RearmReport, RunError> {
        self.check_source(source)?;
        let record = InputRecord::Release(fact);
        if let Some(old) = self.replay[..self.limits.replay]
            .iter()
            .flatten()
            .find(|old| old.sequence() == fact.sequence)
        {
            if *old != record {
                return Err(self.fail(RunError::InputConflict));
            }
            return Ok(RearmReport::Repeated);
        }
        if fact.sequence.get() <= self.highest_input {
            return Err(RunError::OldInputSequence);
        }
        if self.failure.is_some() || self.mode != ControlMode::Running {
            return Err(RunError::GateClosed);
        }
        let slot = fact.slot.index();
        let newly_armed = !self.armed[slot];
        if newly_armed {
            self.armed_from[slot] = None;
        }
        self.armed[slot] = true;
        self.replay[self.replay_next] = Some(record);
        self.replay_next = (self.replay_next + 1) % self.limits.replay;
        self.highest_input = fact.sequence.get();
        Ok(RearmReport::Observed { newly_armed })
    }

    /// Pure eligibility for a retained synthetic SoundIntent, never a device
    /// submission or playback receipt. Semantic records stay acknowledgeable.
    pub fn sound_intent_allowed(&self, key: EventKey) -> Result<bool, RunError> {
        if key.run != self.source.run || key.transport != self.source.transport {
            return Err(RunError::StaleSource);
        }
        if key.sequence.get() <= self.retired_through {
            return Ok(false);
        }
        let event = (0..self.event_len)
            .filter_map(|index| self.event_at(index))
            .find(|event| event.key == key)
            .ok_or(RunError::UnknownEvent)?;
        Ok(self.failure.is_none()
            && self.mode == ControlMode::Running
            && key.sequence.get() > self.cancelled_sound_through
            && event.kind == EventKind::SoundIntent)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{InputEpoch, MapRevision};

    fn empty(id: u128, input: u64) -> SyntheticRun {
        let source = RunSource {
            run: RunId::new(id).unwrap(),
            transport: TransportEpoch::new(1).unwrap(),
            input: InputEpoch::new(input).unwrap(),
            revision: MapRevision::new(1),
        };
        SyntheticRun::new(
            source,
            SyntheticMap::try_new(source.revision, &[]).unwrap(),
            MapTime::from_micros(0),
            Limits::tiny(),
        )
        .unwrap()
    }

    #[test]
    fn exhausted_identifiers_never_block_closing_or_reopen_a_frozen_run() {
        for case in 0..3 {
            let mut run = empty(40_000 + case, if case == 0 { u64::MAX } else { 1 });
            let source = run.source();
            let at = MapTime::from_micros(0);
            run.seal_through(source, at).unwrap();
            assert!(matches!(
                run.advance(source, at, WorkBudget::one()).unwrap(),
                AdvanceReport::Complete { .. }
            ));
            if case == 1 {
                run.next_request = None;
            }
            if case == 2 {
                run.step = u64::MAX;
            }
            run.request_freeze(source, at).unwrap();
            assert_eq!(
                run.settle_freeze(source, WorkBudget::one()).unwrap(),
                FreezeReport::Frozen {
                    at,
                    steps: 0,
                    cancelled_inputs: 0
                }
            );
            let before = run.observation();
            assert_eq!(run.resume(source), Err(RunError::CounterExhausted));
            assert_eq!(run.observation(), before);
            assert_eq!(run.observation().status, RunStatus::Frozen);
        }
    }

    #[test]
    fn exhausted_work_counter_cannot_claim_an_unfinished_frozen_frame() {
        let mut run = empty(40_003, 1);
        let source = run.source();
        let at = MapTime::from_micros(0);
        run.seal_through(source, at).unwrap();
        run.step = u64::MAX;
        run.request_freeze(source, at).unwrap();
        assert_eq!(
            run.settle_freeze(source, WorkBudget::one()),
            Err(RunError::CounterExhausted)
        );
        assert_eq!(
            run.observation().status,
            RunStatus::Failed(Failure {
                cause: RunError::CounterExhausted,
                cancelled_inputs: 0
            })
        );
        assert_eq!(run.observation().ready_frame, None);
        assert_eq!(run.observation().completed_through, None);
        assert_eq!(run.resume(source), Err(RunError::Faulted));
    }
}
