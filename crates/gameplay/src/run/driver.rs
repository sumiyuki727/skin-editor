use super::*;
use crate::PhysicalSlot;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AutomaticPendingReason {
    WorkBudget,
    InputBackpressure,
}

/// Preparation only: these reports never mean gameplay has reached `through`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AutomaticReport {
    Complete {
        source: RunSource,
        through: MapTime,
        admitted: usize,
        steps: u16,
    },
    Pending {
        source: RunSource,
        target: MapTime,
        ready_through: Option<MapTime>,
        reason: AutomaticPendingReason,
        admitted: usize,
        steps: u16,
    },
}

pub(super) struct AutomaticState {
    next: usize,
    pub(super) pending: Option<[InputEvent; 2]>,
    pub(super) ready: Option<MapTime>,
    pub(super) rearm: bool,
}
impl AutomaticState {
    pub(super) const fn new() -> Self {
        Self {
            next: 0,
            pending: None,
            ready: None,
            rearm: true,
        }
    }
    pub(super) fn after_boundary(at: MapTime) -> Self {
        Self {
            ready: Some(at),
            ..Self::new()
        }
    }
    pub(super) fn cover(&mut self, at: MapTime) {
        self.ready = Some(self.ready.map_or(at, |old| old.max(at)));
    }
}

impl SyntheticRun {
    pub const fn driver(&self) -> InputDriver {
        self.driver
    }
    /// A switch uses an already settled frozen boundary; no pending gameplay
    /// prefix is discarded to make the switch succeed. Same-driver is a no-op.
    pub fn select_driver(
        &mut self,
        source: RunSource,
        driver: InputDriver,
    ) -> Result<RunSource, RunError> {
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
        if driver == self.driver {
            return Ok(source);
        }
        if driver == InputDriver::Automatic && self.limits.inputs < 2 {
            return Err(RunError::AutomaticCapacity);
        }
        let source = self.renew_input_epoch()?;
        self.driver = driver;
        Ok(source)
    }
    pub(super) fn check_external_driver(
        &self,
        source: RunSource,
        driver: InputDriver,
    ) -> Result<(), RunError> {
        self.check_source(source)?;
        if driver != self.driver {
            return Err(RunError::DriverMismatch);
        }
        if driver == InputDriver::Automatic {
            return Err(RunError::AutomaticInputReserved);
        }
        Ok(())
    }
    /// Each unit observes one object, rearms the owned virtual slot, or commits
    /// one previously planned two-edge pulse through ordinary atomic admission.
    /// A pulse includes target+1 release even when `through` is its press time.
    /// A backpressured report exposes the safe prefix to seal/consume first.
    pub fn prepare_automatic_inputs(
        &mut self,
        source: RunSource,
        through: MapTime,
        budget: WorkBudget,
    ) -> Result<AutomaticReport, RunError> {
        self.check_source(source)?;
        if self.driver != InputDriver::Automatic {
            return Err(RunError::DriverMismatch);
        }
        if self.failure.is_some() {
            return Err(RunError::Faulted);
        }
        if self.mode != ControlMode::Running {
            return Err(RunError::GateClosed);
        }
        if budget.get() > self.limits.max_work {
            return Err(RunError::BudgetLimit);
        }
        if through < self.start || self.sealed.is_some_and(|old| through < old) {
            return Err(RunError::InvalidTarget);
        }
        let mut steps = 0;
        let mut admitted = 0;
        loop {
            if self.automatic.ready.is_some_and(|at| at >= through) {
                return Ok(AutomaticReport::Complete {
                    source,
                    through,
                    admitted,
                    steps,
                });
            }
            if steps == budget.get() {
                return Ok(self.automatic_pending(
                    through,
                    AutomaticPendingReason::WorkBudget,
                    admitted,
                    steps,
                ));
            }
            steps += 1;
            self.count(WorkCounts {
                management_steps: 1,
                ..WorkCounts::default()
            });
            if self.automatic.rearm {
                let sequence = self
                    .highest_input
                    .checked_add(1)
                    .ok_or(RunError::CounterExhausted)?;
                self.admit_release(
                    source,
                    ReleaseObservation {
                        sequence: InputSequence::new(sequence).expect("nonzero checked increment"),
                        slot: PhysicalSlot::PrimaryOne,
                    },
                )?;
                self.automatic.rearm = false;
            } else if let Some(pair) = self.automatic.pending {
                if pair[0].at > through {
                    self.automatic.cover(through);
                    continue;
                }
                if self.limits.inputs - self.input_len < 2 {
                    return Ok(self.automatic_pending(
                        through,
                        AutomaticPendingReason::InputBackpressure,
                        admitted,
                        steps,
                    ));
                }
                self.admit_inputs(source, &pair)?;
                self.automatic.pending = None;
                self.automatic.next += 1;
                self.automatic.cover(pair[1].at);
                admitted += 2;
            } else {
                let Some(circle) = self.map.circle(self.automatic.next) else {
                    self.automatic.cover(through);
                    continue;
                };
                self.count(WorkCounts {
                    automatic_objects: 1,
                    ..WorkCounts::default()
                });
                if self.sealed.is_some_and(|sealed| circle.target <= sealed)
                    || self.objects[self.automatic.next].judgement.is_some()
                    || self.objects[self.automatic.next].inspection_skipped
                {
                    self.automatic.next += 1;
                    continue;
                }
                if circle.target > through {
                    self.automatic.cover(through);
                    continue;
                }
                let first = self
                    .highest_input
                    .checked_add(1)
                    .ok_or(RunError::CounterExhausted)?;
                let last = first.checked_add(1).ok_or(RunError::CounterExhausted)?;
                // Circle construction checked a later retirement endpoint.
                let release = circle
                    .target
                    .checked_offset(1)
                    .expect("validated circle endpoint");
                self.automatic.pending = Some([
                    InputEvent {
                        sequence: InputSequence::new(first).expect("checked sequence"),
                        at: circle.target,
                        slot: PhysicalSlot::PrimaryOne,
                        edge: InputEdge::Press(circle.center),
                    },
                    InputEvent {
                        sequence: InputSequence::new(last).expect("checked sequence"),
                        at: release,
                        slot: PhysicalSlot::PrimaryOne,
                        edge: InputEdge::Release,
                    },
                ]);
            }
        }
    }
    fn automatic_pending(
        &self,
        target: MapTime,
        reason: AutomaticPendingReason,
        admitted: usize,
        steps: u16,
    ) -> AutomaticReport {
        AutomaticReport::Pending {
            source: self.source,
            target,
            ready_through: self.automatic.ready,
            reason,
            admitted,
            steps,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Circle, InputEpoch, MapRevision, Point};

    #[test]
    fn ten_thousand_generated_pulses_keep_exact_source_local_input_sequences() {
        const N: usize = 10_000;
        let at = |index: usize| MapTime::from_micros(index as i64 * 1_000_000);
        for budget in [WorkBudget::one(), WorkBudget::full()] {
            let source = RunSource {
                run: RunId::new(309_001).unwrap(),
                transport: TransportEpoch::new(1).unwrap(),
                input: InputEpoch::new(1).unwrap(),
                revision: MapRevision::new(720),
            };
            // Known ordered, unique test records. Public stepped compilation is
            // separately exercised by every scale integration fixture.
            let mut circles = Buffer::new(N);
            circles
                .allocate(
                    &mut WorkCounts::default(),
                    None,
                    crate::AllocationKind::Circles,
                    crate::ResourcePool::Precompiled,
                )
                .unwrap();
            for i in 0..N {
                circles.push(
                    Circle::new(ObjectId::new(i as u32), at(i), Point::new(0, 0).unwrap()).unwrap(),
                );
            }
            let map = SyntheticMap {
                revision: source.revision,
                circles,
                len: N,
                payload: None,
            };
            let mut preparation = crate::RunPreparation::new_driven(
                source,
                map,
                MapTime::from_micros(-200_001),
                Limits::reference(),
                InputDriver::Automatic,
            )
            .unwrap();
            let mut ready = false;
            for _ in 0..N + 2 {
                if preparation.step(budget).unwrap() == crate::RunPreparationProgress::Complete {
                    ready = true;
                    break;
                }
            }
            assert!(ready);
            let mut run = preparation.finish().unwrap();
            let start = MapTime::from_micros(-200_001);
            run.request_freeze(source, start).unwrap();
            run.seal_through(source, start).unwrap();
            let mut frozen = false;
            for _ in 0..16 {
                if matches!(
                    run.settle_freeze(source, budget).unwrap(),
                    FreezeReport::Frozen { .. }
                ) {
                    frozen = true;
                    break;
                }
            }
            assert!(frozen);
            let source = run.resume(source).unwrap();
            assert!(matches!(
                run.prepare_automatic_inputs(source, at(0), WorkBudget::one())
                    .unwrap(),
                AutomaticReport::Pending {
                    steps: 1,
                    admitted: 0,
                    ..
                }
            ));
            assert_eq!(run.highest_input, 1);
            assert_eq!(run.input_len, 0);
            assert!(run.replay.iter().flatten().any(|record|matches!(record,InputRecord::Release(fact) if fact.sequence.get()==1 && fact.slot==PhysicalSlot::PrimaryOne)));
            for first in (0..N).step_by(32) {
                let count = (N - first).min(32);
                let through = at(first + count - 1).checked_offset(1).unwrap();
                let mut complete = false;
                for _ in 0..128 {
                    if matches!(
                        run.prepare_automatic_inputs(source, through, budget)
                            .unwrap(),
                        AutomaticReport::Complete { .. }
                    ) {
                        complete = true;
                        break;
                    }
                }
                assert!(complete);
                assert_eq!(run.highest_input, 2 * (first + count) as u64 + 1);
                assert_eq!(run.input_len, 2 * count);
                for offset in 0..count {
                    let i = first + offset;
                    let sequence = 2 * i as u64 + 2;
                    assert_eq!(
                        run.inputs[2 * offset],
                        Some(InputEvent {
                            sequence: InputSequence::new(sequence).unwrap(),
                            at: at(i),
                            slot: PhysicalSlot::PrimaryOne,
                            edge: InputEdge::Press(Point::new(0, 0).unwrap())
                        })
                    );
                    assert_eq!(
                        run.inputs[2 * offset + 1],
                        Some(InputEvent {
                            sequence: InputSequence::new(sequence + 1).unwrap(),
                            at: at(i).checked_offset(1).unwrap(),
                            slot: PhysicalSlot::PrimaryOne,
                            edge: InputEdge::Release
                        })
                    );
                }
                run.seal_through(source, through).unwrap();
                let mut report = run.advance(source, through, budget).unwrap();
                let mut complete = false;
                for _ in 0..4096 {
                    for _ in 0..64 {
                        let Some(event) = run.event_at(0) else {
                            break;
                        };
                        run.acknowledge(event.key).unwrap();
                    }
                    match report {
                        AdvanceReport::Complete { .. } => {
                            complete = true;
                            break;
                        }
                        AdvanceReport::Pending { cursor, .. } => {
                            report = run.continue_advance(cursor, budget).unwrap()
                        }
                    }
                }
                assert!(complete);
            }
            assert_eq!(run.highest_input, 20_001);
            assert_eq!(run.observation().committed_counts.hits, N);
        }
    }

    #[test]
    fn exhausted_pulse_sequences_never_half_admit_or_prevent_freezing() {
        let source = RunSource {
            run: RunId::new(91_001).unwrap(),
            transport: TransportEpoch::new(1).unwrap(),
            input: InputEpoch::new(1).unwrap(),
            revision: MapRevision::new(1),
        };
        let circle = Circle::new(
            ObjectId::new(0),
            MapTime::from_micros(0),
            Point::new(0, 0).unwrap(),
        )
        .unwrap();
        for highest in [u64::MAX - 1, u64::MAX] {
            let map = SyntheticMap::try_new(source.revision, &[circle]).unwrap();
            let mut run = SyntheticRun::new_driven(
                source,
                map,
                MapTime::from_micros(-200_001),
                Limits::tiny(),
                InputDriver::Automatic,
            )
            .unwrap();
            run.highest_input = highest;
            run.automatic.rearm = false;
            let before = run.observation();
            assert_eq!(
                run.prepare_automatic_inputs(source, MapTime::from_micros(0), WorkBudget::one()),
                Err(RunError::CounterExhausted)
            );
            assert_eq!(run.observation(), before);
            assert!(run.automatic.pending.is_none());
            assert_eq!(run.highest_input, highest);
            run.next_request = None;
            run.next_event = None;
            run.request_freeze(source, MapTime::from_micros(-200_001))
                .unwrap();
            run.seal_through(source, MapTime::from_micros(-200_001))
                .unwrap();
            for _ in 0..32 {
                if matches!(
                    run.settle_freeze(source, WorkBudget::one()).unwrap(),
                    FreezeReport::Frozen { .. }
                ) {
                    break;
                }
            }
            assert_eq!(run.observation().status, RunStatus::Frozen);
        }
    }

    #[test]
    fn a_runtime_fault_keeps_retained_auto_work_inert_and_cannot_rearm_inputs() {
        let source = RunSource {
            run: RunId::new(91_002).unwrap(),
            transport: TransportEpoch::new(1).unwrap(),
            input: InputEpoch::new(1).unwrap(),
            revision: MapRevision::new(2),
        };
        let circles = core::array::from_fn::<_, 3, _>(|i| {
            let at = [0, 1_000_000, 2_000_000][i];
            Circle::new(
                ObjectId::new(i as u32),
                MapTime::from_micros(at),
                Point::new(0, 0).unwrap(),
            )
            .unwrap()
        });
        let map = SyntheticMap::try_new(source.revision, &circles).unwrap();
        let mut run = SyntheticRun::new_driven(
            source,
            map,
            MapTime::from_micros(-200_001),
            Limits::tiny(),
            InputDriver::Automatic,
        )
        .unwrap();
        for _ in 0..128 {
            if matches!(
                run.prepare_automatic_inputs(
                    source,
                    MapTime::from_micros(2_000_001),
                    WorkBudget::one()
                )
                .unwrap(),
                AutomaticReport::Pending {
                    reason: AutomaticPendingReason::InputBackpressure,
                    ..
                }
            ) {
                break;
            }
        }
        assert_eq!(run.observation().pending_inputs, 4);
        assert!(run.automatic.pending.is_some());
        run.next_event = None;
        run.seal_through(source, MapTime::from_micros(1_000_001))
            .unwrap();
        let mut report = run.advance(source, MapTime::from_micros(0), WorkBudget::one());
        for _ in 0..128 {
            match report {
                Ok(AdvanceReport::Pending { cursor, .. }) => {
                    report = run.continue_advance(cursor, WorkBudget::one())
                }
                _ => break,
            }
        }
        assert_eq!(report, Err(RunError::CounterExhausted));
        let before = run.observation();
        assert!(matches!(before.status, RunStatus::Failed(_)));
        assert_eq!(before.armed_slots, [false; 4]);
        let retained = (
            run.automatic.next,
            run.automatic.pending,
            run.automatic.ready,
            run.automatic.rearm,
            run.highest_input,
        );
        assert_eq!(
            run.prepare_automatic_inputs(
                source,
                MapTime::from_micros(i64::MAX),
                WorkBudget::full()
            ),
            Err(RunError::Faulted)
        );
        assert_eq!(
            run.select_driver(source, InputDriver::Regression),
            Err(RunError::Faulted)
        );
        assert_eq!(run.resume(source), Err(RunError::Faulted));
        assert_eq!(
            run.observe_driver_release(
                source,
                InputDriver::Automatic,
                ReleaseObservation {
                    sequence: InputSequence::new(99).unwrap(),
                    slot: PhysicalSlot::PrimaryOne
                }
            ),
            Err(RunError::AutomaticInputReserved)
        );
        assert_eq!(run.observation(), before);
        assert_eq!(
            (
                run.automatic.next,
                run.automatic.pending,
                run.automatic.ready,
                run.automatic.rearm,
                run.highest_input
            ),
            retained
        );
    }
}
