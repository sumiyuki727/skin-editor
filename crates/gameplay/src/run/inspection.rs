use super::*;

/// Only an inspection operation can produce this key. It binds the complete
/// base source, target and a request counter independent of advance requests.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InspectionKey {
    base: RunSource,
    target: MapTime,
    id: u64,
}
impl InspectionKey {
    pub const fn base(self) -> RunSource {
        self.base
    }
    pub const fn target(self) -> MapTime {
        self.target
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InspectionCursor {
    key: InspectionKey,
    checked: usize,
}
impl InspectionCursor {
    pub const fn key(self) -> InspectionKey {
        self.key
    }
}

/// The current frame remains the old frozen frame until explicit activation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InspectionInfo {
    pub key: InspectionKey,
    pub checked: usize,
    pub total: usize,
    pub ready: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InspectionReport {
    Pending {
        cursor: InspectionCursor,
        checked: usize,
        total: usize,
        steps: u16,
    },
    Ready {
        key: InspectionKey,
        at: MapTime,
        steps: u16,
    },
}

/// One inline candidate shares the existing immutable map. There are no
/// detached results, heap allocations or deferred releases in this slice.
pub(super) struct InspectionCandidate {
    key: InspectionKey,
    checked: usize,
    ready: bool,
    frame: ReadyFrame,
    skipped: [bool; MAX_CIRCLES],
    appeared: usize,
    expired: usize,
    retired: usize,
}
impl InspectionCandidate {
    pub(super) fn info(&self) -> InspectionInfo {
        InspectionInfo {
            key: self.key,
            checked: self.checked,
            total: self.frame.len,
            ready: self.ready,
        }
    }
}

impl SyntheticRun {
    /// Start a same-map inspection from a completed frozen run. The current
    /// run, ledger and event queue remain intact until successful activation.
    pub fn request_inspection(
        &mut self,
        source: RunSource,
        target: MapTime,
    ) -> Result<InspectionCursor, RunError> {
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
        let transport = source
            .transport
            .checked_next()
            .ok_or(RunError::CounterExhausted)?;
        let input = source
            .input
            .checked_next()
            .ok_or(RunError::CounterExhausted)?;
        let id = self.next_inspection.ok_or(RunError::CounterExhausted)?;
        let key = InspectionKey {
            base: source,
            target,
            id,
        };
        self.inspection = Some(InspectionCandidate {
            key,
            checked: 0,
            ready: false,
            frame: ReadyFrame {
                source: RunSource {
                    transport,
                    input,
                    ..source
                },
                at: target,
                evidence: Evidence::SyntheticTest,
                statistics_origin: StatisticsOrigin::Inspection { at: target },
                objects: [None; MAX_CIRCLES],
                len: self.map.len,
            },
            skipped: [false; MAX_CIRCLES],
            appeared: 0,
            expired: 0,
            retired: 0,
        });
        self.next_inspection = id.checked_add(1);
        Ok(InspectionCursor { key, checked: 0 })
    }

    /// Each unit inspects at most one circle and builds its projection. Empty
    /// maps still require one unit to complete the closed target section.
    pub fn step_inspection(
        &mut self,
        cursor: InspectionCursor,
        budget: WorkBudget,
    ) -> Result<InspectionReport, RunError> {
        self.check_inspection(cursor.key)?;
        let candidate = self.inspection.as_mut().expect("checked candidate");
        if candidate.ready || candidate.checked != cursor.checked {
            return Err(RunError::StaleInspection);
        }
        if budget.get() > self.limits.max_work {
            return Err(RunError::BudgetLimit);
        }
        let target = candidate.key.target;
        let mut steps = 0;
        while steps < budget.get() && !candidate.ready {
            if let Some(circle) = self.map.circle(candidate.checked) {
                let index = candidate.checked;
                let skipped = target > circle.close;
                candidate.skipped[index] = skipped;
                candidate.appeared += usize::from(target >= circle.visible);
                candidate.expired += usize::from(skipped);
                candidate.retired += usize::from(target >= circle.retire);
                // This slice omits inspection visuals for expired objects.
                // None is accompanied by an explicit unavailable reason.
                candidate.frame.objects[index] = Some(CircleFrame {
                    object: circle.id,
                    phase: if skipped {
                        Phase::Retired
                    } else if target < circle.visible {
                        Phase::Future
                    } else {
                        Phase::VisibleUnjudged
                    },
                    judgement: None,
                    unavailable: skipped.then_some(UnavailableReason::InspectionSkipped),
                });
                candidate.checked += 1;
            }
            steps += 1;
            candidate.ready = candidate.checked == self.map.len;
        }
        Ok(if candidate.ready {
            InspectionReport::Ready {
                key: candidate.key,
                at: target,
                steps,
            }
        } else {
            InspectionReport::Pending {
                cursor: InspectionCursor {
                    key: candidate.key,
                    checked: candidate.checked,
                },
                checked: candidate.checked,
                total: self.map.len,
                steps,
            }
        })
    }

    /// Publish a fully rebuilt frozen section, never an intermediate frame.
    /// Fixed queue bookkeeping is reset; no map scan/copy occurs here.
    pub fn activate_inspection(&mut self, key: InspectionKey) -> Result<RunSource, RunError> {
        self.check_inspection(key)?;
        if !self.inspection.as_ref().expect("checked candidate").ready {
            return Err(RunError::InspectionIncomplete);
        }
        let candidate = self.inspection.take().expect("ready candidate");
        self.source = candidate.frame.source;
        self.start = key.target;
        self.statistics_origin = candidate.frame.statistics_origin;
        self.inspection_skipped = candidate.skipped;
        self.appeared = candidate.appeared;
        self.expired = candidate.expired;
        self.retired = candidate.retired;
        self.judgements.fill(None);
        self.sealed = Some(key.target);
        self.highest_input = 0;
        self.inputs.fill(None);
        self.input_len = 0;
        self.replay.fill(None);
        self.replay_next = 0;
        self.held.fill(false);
        self.armed.fill(false);
        self.armed_from.fill(None);
        self.freeze_info = Some(FreezeInfo {
            source: self.source,
            boundary: key.target,
            cancelled_inputs: 0,
        });
        self.cancelled_sound_through = 0;
        self.events.fill(None);
        self.acknowledged.fill(false);
        self.event_head = 0;
        self.event_len = 0;
        self.next_event = Some(1);
        self.issued_through = 0;
        self.retired_through = 0;
        self.frames[0] = candidate.frame;
        self.ready = Some(0);
        self.dirty = None;
        self.last_committed = Some(key.target);
        self.active = None;
        self.next_request = Some(1);
        self.step = 0;
        self.work = Work::Select;
        // The base was frozen, and stays frozen until explicit resume.
        Ok(self.source)
    }

    /// Cancellation needs no new request, event credit or generation. Inline
    /// storage remains reserved as part of this run, with no late owner.
    pub fn cancel_inspection(&mut self, key: InspectionKey) -> Result<(), RunError> {
        self.check_inspection(key)?;
        self.inspection = None;
        Ok(())
    }

    fn check_inspection(&self, key: InspectionKey) -> Result<(), RunError> {
        self.check_source(key.base)?;
        if self
            .inspection
            .as_ref()
            .is_none_or(|value| value.key != key)
        {
            return Err(RunError::StaleInspection);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{InputEpoch, MapRevision};

    fn frozen(transport: u64, input: u64) -> SyntheticRun {
        let source = RunSource {
            run: RunId::new(60_001).unwrap(),
            transport: TransportEpoch::new(transport).unwrap(),
            input: InputEpoch::new(input).unwrap(),
            revision: MapRevision::new(7),
        };
        let map = SyntheticMap::try_new(source.revision, &[]).unwrap();
        let at = MapTime::from_micros(0);
        let mut run = SyntheticRun::new(source, map, at, Limits::tiny()).unwrap();
        run.request_freeze(source, at).unwrap();
        run.seal_through(source, at).unwrap();
        run.settle_freeze(source, WorkBudget::one()).unwrap();
        run
    }

    #[test]
    fn exhausted_epochs_reject_candidates_without_changing_the_frozen_base() {
        for (transport, input) in [(u64::MAX, 1), (1, u64::MAX)] {
            let mut run = frozen(transport, input);
            let before = run.observation();
            assert_eq!(
                run.request_inspection(run.source(), MapTime::from_micros(-1)),
                Err(RunError::CounterExhausted)
            );
            assert_eq!(run.observation(), before);
        }
    }

    #[test]
    fn last_candidate_key_can_be_cancelled_without_any_other_capacity() {
        let mut run = frozen(1, 1);
        let before = run.observation();
        run.next_inspection = Some(u64::MAX);
        run.next_request = None;
        run.next_event = None;
        let cursor = run
            .request_inspection(run.source(), MapTime::from_micros(-1))
            .unwrap();
        run.cancel_inspection(cursor.key()).unwrap();
        assert_eq!(run.observation(), before);
        assert_eq!(
            run.request_inspection(run.source(), MapTime::from_micros(1)),
            Err(RunError::CounterExhausted)
        );
        assert_eq!(run.observation(), before);
        assert_eq!(
            run.step_inspection(cursor, WorkBudget::one()),
            Err(RunError::StaleInspection)
        );
    }

    #[test]
    fn full_base_source_and_operation_target_are_checked_before_any_work() {
        let mut run = frozen(1, 1);
        let cursor = run
            .request_inspection(run.source(), MapTime::from_micros(-1))
            .unwrap();
        let source = run.source();
        let before = run.observation();
        for base in [
            RunSource {
                run: RunId::new(60_002).unwrap(),
                ..source
            },
            RunSource {
                transport: TransportEpoch::new(2).unwrap(),
                ..source
            },
            RunSource {
                input: InputEpoch::new(2).unwrap(),
                ..source
            },
            RunSource {
                revision: MapRevision::new(8),
                ..source
            },
        ] {
            let old = InspectionCursor {
                key: InspectionKey { base, ..cursor.key },
                ..cursor
            };
            assert_eq!(
                run.step_inspection(old, WorkBudget::one()),
                Err(RunError::StaleSource)
            );
            assert_eq!(run.cancel_inspection(old.key()), Err(RunError::StaleSource));
            assert_eq!(
                run.activate_inspection(old.key()),
                Err(RunError::StaleSource)
            );
            assert_eq!(run.observation(), before);
        }
        let wrong_target = InspectionKey {
            target: MapTime::from_micros(0),
            ..cursor.key
        };
        assert_eq!(
            run.activate_inspection(wrong_target),
            Err(RunError::StaleInspection)
        );
        assert_eq!(run.observation(), before);
    }
}
