use super::*;

#[derive(Clone, Copy)]
enum Action {
    Input,
    Deadline(usize),
    Appear(usize),
    Retire(usize),
}
#[derive(Clone, Copy)]
struct Scheduled {
    at: MapTime,
    rank: u8,
    action: Action,
}
enum StepResult {
    Worked,
    Blocked,
}

impl SyntheticRun {
    pub fn advance(
        &mut self,
        source: RunSource,
        target: MapTime,
        budget: WorkBudget,
    ) -> Result<AdvanceReport, RunError> {
        self.check_source(source)?;
        if self.failure.is_some() {
            return Err(RunError::Faulted);
        }
        if self.mode != ControlMode::Running {
            return Err(RunError::GateClosed);
        }
        if budget.get() > self.limits.max_work {
            return Err(RunError::BudgetLimit);
        }
        if self.active.is_some() {
            return Err(RunError::Busy);
        }
        if target < self.start
            || self.last_committed.is_some_and(|old| target < old)
            || self.completed().is_some_and(|old| target < old)
        {
            return Err(RunError::InvalidTarget);
        }
        if self.sealed.is_none_or(|sealed| target > sealed) {
            return Err(RunError::BeyondSeal);
        }
        let Some(id) = self.next_request else {
            return Err(self.fail(RunError::CounterExhausted));
        };
        self.next_request = id.checked_add(1);
        self.active = Some(Active {
            key: RequestKey { source, id },
            target,
        });
        self.drive(budget)
    }
    pub fn continue_advance(
        &mut self,
        cursor: AdvanceCursor,
        budget: WorkBudget,
    ) -> Result<AdvanceReport, RunError> {
        self.check_source(cursor.request.source)?;
        if self
            .active
            .is_none_or(|active| active.key != cursor.request)
            || cursor.step != self.step
        {
            return Err(RunError::StaleCursor);
        }
        if budget.get() > self.limits.max_work {
            return Err(RunError::BudgetLimit);
        }
        self.drive(budget)
    }
    fn drive(&mut self, budget: WorkBudget) -> Result<AdvanceReport, RunError> {
        let active = self.active.expect("active advance");
        match self.pump(active.target, budget)? {
            PumpReport::Complete { steps } => {
                self.active = None;
                Ok(AdvanceReport::Complete {
                    through: active.target,
                    steps,
                })
            }
            PumpReport::Pending { reason, steps } => Ok(self.pending(active, reason, steps)),
        }
    }
    pub(super) fn pump(
        &mut self,
        target: MapTime,
        budget: WorkBudget,
    ) -> Result<PumpReport, RunError> {
        let mut steps = 0;
        loop {
            if matches!(self.work, Work::Select)
                && self.dirty.is_none()
                && !self.has_freeze_cleanup()
                && self.completed() == Some(target)
                && self.next_action().is_none_or(|next| next.at > target)
            {
                return Ok(PumpReport::Complete { steps });
            }
            if steps == budget.get() {
                return Ok(PumpReport::Pending {
                    reason: PendingReason::WorkBudget,
                    steps,
                });
            }
            let Some(next_step) = self.step.checked_add(1) else {
                return Err(self.fail(RunError::CounterExhausted));
            };
            match self.perform_step(target) {
                Ok(StepResult::Blocked) => {
                    return Ok(PumpReport::Pending {
                        reason: PendingReason::EventBackpressure,
                        steps,
                    });
                }
                Ok(StepResult::Worked) => {
                    self.step = next_step;
                    steps += 1;
                }
                Err(error) => return Err(self.fail(error)),
            }
        }
    }
    fn pending(&self, active: Active, reason: PendingReason, steps: u16) -> AdvanceReport {
        AdvanceReport::Pending {
            cursor: AdvanceCursor {
                request: active.key,
                step: self.step,
            },
            reason,
            completed_through: self.completed(),
            steps,
        }
    }
    // Four chronological heads, independent of the number of accepted inputs.
    // Candidate lookup is resumable; complete publication commits one fixed header.
    fn next_action(&self) -> Option<Scheduled> {
        let candidates = [
            self.inputs[0].map(|input| Scheduled {
                at: input.at,
                rank: 0,
                action: Action::Input,
            }),
            self.map.circle(self.expired).map(|circle| Scheduled {
                at: circle.miss,
                rank: 1,
                action: Action::Deadline(self.expired),
            }),
            self.map.circle(self.appeared).map(|circle| Scheduled {
                at: circle.visible,
                rank: 2,
                action: Action::Appear(self.appeared),
            }),
            self.map.circle(self.retired).map(|circle| Scheduled {
                at: circle.retire,
                rank: 3,
                action: Action::Retire(self.retired),
            }),
        ];
        candidates
            .into_iter()
            .flatten()
            .min_by_key(|value| (value.at, value.rank))
    }
    fn perform_step(&mut self, target: MapTime) -> Result<StepResult, RunError> {
        if self.has_freeze_cleanup() {
            self.input_len -= 1;
            self.inputs[self.input_len] = None;
            // Closed admission and a bounded queue limit this count to 64.
            self.freeze_info
                .as_mut()
                .expect("freezing boundary")
                .cancelled_inputs += 1;
            return Ok(StepResult::Worked);
        }
        self.count(WorkCounts {
            management_steps: 1,
            ..WorkCounts::default()
        });
        match self.work {
            Work::Select => {
                let next = self.next_action();
                if let Some(at) = self.dirty
                    && next.is_none_or(|next| next.at > at)
                {
                    return self.begin_publication(at);
                }
                let Some(next) = next.filter(|next| next.at <= target) else {
                    return self.begin_publication(target);
                };
                match next.action {
                    Action::Input => {
                        let input = self.inputs[0].expect("input head");
                        let mut held = self.held;
                        let slot = input.slot.index();
                        let logical = slot / 2 * 2;
                        let was_down = held[logical] || held[logical + 1];
                        let rising = match input.edge {
                            InputEdge::Press(_) => {
                                let rising = !was_down && !held[slot];
                                held[slot] = true;
                                rising
                            }
                            InputEdge::Release => {
                                held[slot] = false;
                                false
                            }
                        };
                        let draft = InputDraft {
                            input,
                            held,
                            next: self.expired,
                            candidate: None,
                        };
                        self.work = if rising {
                            Work::Search(draft)
                        } else {
                            Work::Commit(draft)
                        };
                        Ok(StepResult::Worked)
                    }
                    Action::Deadline(index) => {
                        if self.objects[index].judgement.is_none() {
                            let Some(reservation) = self.reserve_events(1)? else {
                                return Ok(StepResult::Blocked);
                            };
                            let record = JudgementRecord {
                                object: self.map.circle(index).expect("deadline circle").id,
                                at: next.at,
                                result: Judgement::Miss,
                            };
                            self.objects[index].judgement = Some(record);
                            self.emit(reservation, record);
                        }
                        self.expired += 1;
                        self.commit_time(next.at);
                        Ok(StepResult::Worked)
                    }
                    Action::Appear(index) => {
                        debug_assert_eq!(index, self.appeared);
                        self.appeared += 1;
                        self.commit_time(next.at);
                        Ok(StepResult::Worked)
                    }
                    Action::Retire(index) => {
                        debug_assert_eq!(index, self.retired);
                        self.retired += 1;
                        self.commit_time(next.at);
                        Ok(StepResult::Worked)
                    }
                }
            }
            Work::Search(mut draft) => {
                self.count(WorkCounts {
                    search_objects: u64::from(draft.next < self.map.len),
                    ..WorkCounts::default()
                });
                let Some(circle) = self.map.circle(draft.next) else {
                    self.work = Work::Commit(draft);
                    return Ok(StepResult::Worked);
                };
                let InputEdge::Press(point) = draft.input.edge else {
                    self.work = Work::Commit(draft);
                    return Ok(StepResult::Worked);
                };
                if draft.input.at < circle.open {
                    self.work = Work::Commit(draft);
                } else if draft.input.at <= circle.close
                    && self.objects[draft.next].judgement.is_none()
                    && point.within_synthetic_radius(circle.center)
                {
                    draft.candidate = Some(draft.next);
                    self.work = Work::Commit(draft);
                } else {
                    draft.next += 1;
                    self.work = Work::Search(draft);
                }
                Ok(StepResult::Worked)
            }
            Work::Commit(draft) => {
                let reservation = if draft.candidate.is_some() {
                    let Some(reservation) = self.reserve_events(2)? else {
                        return Ok(StepResult::Blocked);
                    };
                    Some(reservation)
                } else {
                    None
                };
                // No input edge or ledger mutation until all event slots and
                // identifiers for the complete transition have been reserved.
                if let Some(index) = draft.candidate {
                    let record = JudgementRecord {
                        object: self.map.circle(index).expect("hit circle").id,
                        at: draft.input.at,
                        result: Judgement::Hit,
                    };
                    self.objects[index].judgement = Some(record);
                    self.emit(reservation.expect("hit reservation"), record);
                }
                self.held = draft.held;
                self.inputs.copy_within(1..self.input_len, 0);
                self.input_len -= 1;
                self.inputs[self.input_len] = None;
                self.work = Work::Select;
                self.commit_time(draft.input.at);
                Ok(StepResult::Worked)
            }
            Work::Publish { at } => self.publish_section(at),
        }
    }
    fn commit_time(&mut self, at: MapTime) {
        debug_assert!(self.last_committed.is_none_or(|old| old <= at));
        self.last_committed = Some(at);
        self.dirty = Some(at);
    }
    fn begin_publication(&mut self, at: MapTime) -> Result<StepResult, RunError> {
        if self.map.is_empty() {
            return self.publish_section(at);
        }
        // Separately resumable preparation of the fixed header. No object scan.
        self.work = Work::Publish { at };
        Ok(StepResult::Worked)
    }
    fn publish_section(&mut self, at: MapTime) -> Result<StepResult, RunError> {
        if self.ready.is_none_or(|old| old.at != at) {
            let publication = self.next_publication.ok_or(RunError::CounterExhausted)?;
            self.count(WorkCounts {
                publications: 1,
                ..WorkCounts::default()
            });
            self.ready = Some(FrameHeader {
                source: self.source,
                at,
                evidence: Evidence::SyntheticTest,
                profile_version: SYNTHETIC_PROFILE_VERSION,
                statistics_origin: self.statistics_origin,
                counts: self.committed_counts,
                publication,
                len: self.map.len,
            });
            self.next_publication = publication.checked_add(1);
        }
        self.dirty = None;
        self.last_committed = Some(at);
        self.work = Work::Select;
        Ok(StepResult::Worked)
    }
}
