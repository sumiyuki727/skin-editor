#![allow(dead_code)]
use skin_editor_gameplay::*;

pub const N: usize = 10_000;
pub const START: i64 = -200_001;
pub const END: i64 = 9_999_300_000;
pub const BOUND: usize = N * 64 + 256;
pub fn t(at: i64) -> MapTime {
    MapTime::from_micros(at)
}
pub fn target(i: usize) -> i64 {
    i64::try_from(i).unwrap().checked_mul(1_000_000).unwrap()
}
pub fn source(id: u128) -> RunSource {
    RunSource {
        run: RunId::new(id).unwrap(),
        transport: TransportEpoch::new(1).unwrap(),
        input: InputEpoch::new(1).unwrap(),
        revision: MapRevision::new(720),
    }
}
pub fn circles() -> Vec<Circle> {
    (0..N)
        .map(|i| {
            Circle::new(
                ObjectId::new(u32::try_from(i).unwrap()),
                t(target(i)),
                Point::new(0, 0).unwrap(),
            )
            .unwrap()
        })
        .collect()
}
pub fn edge(sequence: u64, at: i64, slot: PhysicalSlot, edge: InputEdge) -> InputEvent {
    InputEvent {
        sequence: InputSequence::new(sequence).unwrap(),
        at: t(at),
        slot,
        edge,
    }
}
pub fn press(sequence: u64, at: i64) -> InputEvent {
    edge(
        sequence,
        at,
        PhysicalSlot::PrimaryOne,
        InputEdge::Press(Point::new(0, 0).unwrap()),
    )
}
pub fn fact(sequence: u64, slot: PhysicalSlot) -> ReleaseObservation {
    ReleaseObservation {
        sequence: InputSequence::new(sequence).unwrap(),
        slot,
    }
}
pub fn finish(task: &mut PreparationTask<'_>, budget: WorkBudget) {
    for _ in 0..BOUND {
        match task.step(task.cursor(), budget).unwrap() {
            PreparationReport::Ready { steps, .. } => {
                assert!(steps <= budget.get());
                return;
            }
            PreparationReport::Pending { steps, .. } => assert!(steps > 0 && steps <= budget.get()),
        }
    }
    panic!("preparation bound: {:?} {:?}", task.stage(), task.cursor());
}

/// An independent formula oracle. Retains one event cursor, never a song log.
pub struct Oracle {
    pub source: RunSource,
    pub first: usize,
    pub next: usize,
    pub sequence: u64,
    pub misses: [Option<usize>; 4],
    pub miss_range: core::ops::Range<usize>,
    pub first_hit_offset: i64,
    pub hit_override: Option<(usize, i64)>,
    sound: bool,
    pub hits: usize,
    pub miss_count: usize,
    pub sounds: usize,
}
impl Oracle {
    pub fn new(source: RunSource, first: usize) -> Self {
        Self {
            source,
            first,
            next: first,
            sequence: 1,
            misses: [None; 4],
            miss_range: 0..0,
            first_hit_offset: 0,
            hit_override: None,
            sound: false,
            hits: 0,
            miss_count: 0,
            sounds: 0,
        }
    }
    pub fn result(&self, i: usize) -> Judgement {
        if self.misses.contains(&Some(i)) || self.miss_range.contains(&i) {
            Judgement::Miss
        } else {
            Judgement::Hit
        }
    }
    pub fn hit_at(&self, i: usize) -> i64 {
        if let Some((index, at)) = self.hit_override
            && index == i
        {
            return at;
        }
        target(i)
            .checked_add(if i == self.first {
                self.first_hit_offset
            } else {
                0
            })
            .unwrap()
    }
    pub fn record(&self, i: usize) -> JudgementRecord {
        let result = self.result(i);
        JudgementRecord {
            object: ObjectId::new(i as u32),
            at: t(if result == Judgement::Miss {
                target(i) + 100_001
            } else {
                self.hit_at(i)
            }),
            result,
        }
    }
    pub fn accept(&mut self, event: GameEvent) {
        assert!(self.next < N, "unexpected extra event {event:?}");
        let record = self.record(self.next);
        let kind = if self.sound {
            EventKind::SoundIntent
        } else {
            EventKind::Judgement(record.result)
        };
        assert_eq!(
            event,
            GameEvent {
                key: EventKey::new(
                    self.source.run,
                    self.source.transport,
                    EventSequence::new(self.sequence).unwrap()
                ),
                object: record.object,
                at: record.at,
                kind,
                evidence: Evidence::SyntheticTest,
                profile_version: 1
            }
        );
        self.sequence += 1;
        if self.sound {
            self.sounds += 1;
            self.sound = false;
            self.next += 1;
        } else if record.result == Judgement::Hit {
            self.hits += 1;
            self.sound = true;
        } else {
            self.miss_count += 1;
            self.next += 1;
        }
    }
}
#[derive(Default, Clone, Copy, Debug)]
pub struct Peaks {
    pub inputs: usize,
    pub events: usize,
    pub reserved_source: usize,
    pub reserved_runtime: usize,
    pub payload_source: usize,
    pub payload_runtime: usize,
}
pub struct Playback {
    pub owner: SyntheticCoordinator,
    pub budget: WorkBudget,
    pub oracle: Oracle,
    pub peaks: Peaks,
    pub input_sequence_offset: u64,
}
impl Playback {
    pub fn new(data: &[Circle], id: u128, budget: WorkBudget) -> Self {
        let s = source(id);
        let mut owner = SyntheticCoordinator::new(
            CoordinatorId::new(id).unwrap(),
            Limits::reference(),
            PreparationLimits::reference(),
        )
        .unwrap();
        let mut task = owner.request_create(s, data, t(START)).unwrap();
        finish(&mut task, budget);
        owner.activate(task.key(), &mut task).unwrap();
        drop(task);
        let mut value = Self {
            owner,
            budget,
            oracle: Oracle::new(s, 0),
            peaks: Peaks::default(),
            input_sequence_offset: 0,
        };
        value.sample();
        value
    }
    pub fn source(&self) -> RunSource {
        self.owner.current().unwrap().source()
    }
    pub fn sample(&mut self) {
        let o = self.owner.current().unwrap().observation();
        let r = self.owner.resources();
        let a = self.owner.allocations();
        assert!(o.pending_inputs <= 64 && o.pending_events <= 64);
        assert!(r.precompiled_bytes <= 8 * 1024 * 1024 && r.runtime_bytes <= 8 * 1024 * 1024);
        assert!(a.precompiled_bytes <= r.precompiled_bytes && a.runtime_bytes <= r.runtime_bytes);
        self.peaks.inputs = self.peaks.inputs.max(o.pending_inputs);
        self.peaks.events = self.peaks.events.max(o.pending_events);
        self.peaks.reserved_source = self.peaks.reserved_source.max(r.precompiled_bytes);
        self.peaks.reserved_runtime = self.peaks.reserved_runtime.max(r.runtime_bytes);
        self.peaks.payload_source = self.peaks.payload_source.max(a.precompiled_bytes);
        self.peaks.payload_runtime = self.peaks.payload_runtime.max(a.runtime_bytes);
    }
    pub fn drain(&mut self) {
        self.sample();
        for _ in 0..64 {
            let Some(event) = self.owner.current().unwrap().event_at(0) else {
                return;
            };
            self.oracle.accept(event);
            assert!(matches!(
                self.owner.acknowledge(event.key).unwrap(),
                Acknowledgement::Acknowledged { removed: 1, .. }
            ));
        }
        assert!(self.owner.current().unwrap().event_at(0).is_none());
    }
    pub fn seal(&mut self, at: i64) {
        if self
            .owner
            .current()
            .unwrap()
            .observation()
            .sealed_through
            .is_none_or(|old| old < t(at))
        {
            self.owner.seal_through(self.source(), t(at)).unwrap();
        }
    }
    pub fn advance(&mut self, at: i64) {
        self.seal(at);
        let mut report = self
            .owner
            .advance(self.source(), t(at), self.budget)
            .unwrap();
        for _ in 0..BOUND {
            self.sample();
            match report {
                AdvanceReport::Complete { through, steps } => {
                    assert_eq!(through, t(at));
                    assert!(steps <= self.budget.get());
                    self.drain();
                    return;
                }
                AdvanceReport::Pending {
                    cursor,
                    reason,
                    steps,
                    ..
                } => {
                    assert!(steps <= self.budget.get());
                    if reason == PendingReason::EventBackpressure {
                        assert!(self.owner.current().unwrap().event_at(0).is_some());
                        self.drain();
                    } else {
                        assert!(steps > 0);
                    }
                    report = self.owner.continue_advance(cursor, self.budget).unwrap();
                }
            }
        }
        panic!("advance bound at {at}: {report:?}");
    }
    pub fn freeze(&mut self, at: i64) {
        self.owner.request_freeze(self.source(), t(at)).unwrap();
        self.seal(at);
        for _ in 0..BOUND {
            let report = self
                .owner
                .settle_freeze(self.source(), self.budget)
                .unwrap();
            self.sample();
            match report {
                FreezeReport::Frozen {
                    at: done, steps, ..
                } => {
                    assert_eq!(done, t(at));
                    assert!(steps <= self.budget.get());
                    return;
                }
                FreezeReport::Pending { reason, steps, .. } => {
                    assert!(steps <= self.budget.get());
                    if reason == PendingReason::EventBackpressure {
                        self.drain();
                    } else {
                        assert!(steps > 0);
                    }
                }
            }
        }
        panic!("freeze bound at {at}");
    }
    pub fn automatic(&mut self) {
        self.freeze(START);
        let changed = self
            .owner
            .select_driver(self.source(), InputDriver::Automatic)
            .unwrap();
        self.owner.resume(changed).unwrap();
        assert_eq!(self.source().input.get(), 3);
    }
    pub fn prepare(&mut self, at: i64) {
        for _ in 0..BOUND {
            let report = self
                .owner
                .prepare_automatic_inputs(self.source(), t(at), self.budget)
                .unwrap();
            self.sample();
            match report {
                AutomaticReport::Complete {
                    source,
                    through,
                    steps,
                    ..
                } => {
                    assert_eq!(source, self.source());
                    assert_eq!(through, t(at));
                    assert!(steps <= self.budget.get());
                    return;
                }
                AutomaticReport::Pending {
                    source,
                    target,
                    reason,
                    ready_through,
                    steps,
                    ..
                } => {
                    assert_eq!(source, self.source());
                    assert_eq!(target, t(at));
                    assert!(steps > 0 && steps <= self.budget.get());
                    if reason == AutomaticPendingReason::InputBackpressure {
                        let prefix = ready_through.expect("safe prefix");
                        assert!(prefix < t(at));
                        self.advance(prefix.micros());
                    }
                }
            }
        }
        panic!("automatic preparation bound at {at}");
    }
    pub fn auto_range(&mut self, begin: usize, end: usize) {
        for first in (begin..end).step_by(32) {
            let last = (first + 32).min(end) - 1;
            let at = target(last) + 1;
            self.prepare(at);
            self.advance(at);
        }
    }
    pub fn external_range(&mut self, begin: usize, end: usize) {
        let driver = self.owner.current().unwrap().driver();
        let empty = edge(1, START, PhysicalSlot::PrimaryOne, InputEdge::Release);
        for first in (begin..end).step_by(32) {
            let count = (end - first).min(32);
            let mut batch = [empty; 64];
            for offset in 0..count {
                let i = first + offset;
                let at = self.oracle.hit_at(i);
                let seq = u64::try_from(i)
                    .unwrap()
                    .checked_mul(2)
                    .unwrap()
                    .checked_add(1)
                    .unwrap()
                    .checked_add(self.input_sequence_offset)
                    .unwrap();
                batch[2 * offset] = press(seq, at);
                batch[2 * offset + 1] = edge(
                    seq + 1,
                    at + 1,
                    PhysicalSlot::PrimaryOne,
                    InputEdge::Release,
                );
            }
            assert_eq!(
                self.owner
                    .submit_driver_inputs(self.source(), driver, &batch[..2 * count])
                    .unwrap(),
                Admission {
                    added: 2 * count,
                    repeated: 0
                }
            );
            self.sample();
            self.advance(batch[2 * count - 1].at.micros());
        }
    }
    pub fn tail(&mut self, first: usize) {
        if self.owner.current().unwrap().driver() == InputDriver::Automatic {
            self.auto_range(first, N);
            self.prepare(END);
        } else {
            self.external_range(first, N);
        }
        self.advance(END);
    }
    pub fn verify_end(&mut self, origin: StatisticsOrigin) -> (FrameCounts, u64, usize) {
        self.drain();
        assert_eq!(self.oracle.next, N);
        assert!(!self.oracle.sound);
        let run = self.owner.current().unwrap();
        let s = run.source();
        let o = run.observation();
        assert_eq!(o.status, RunStatus::Ready);
        assert_eq!(o.completed_through, Some(t(END)));
        assert_eq!((o.pending_inputs, o.pending_events), (0, 0));
        let frame = run.ready_frame(s).unwrap().unwrap();
        let h = frame.header();
        let counts = FrameCounts {
            hits: self.oracle.hits,
            misses: self.oracle.miss_count,
            inspection_skipped: self.oracle.first,
        };
        assert_eq!(h.counts, counts);
        assert_eq!(h.statistics_origin, origin);
        assert_eq!(
            (h.at, h.len(), h.evidence, h.profile_version),
            (t(END), N, Evidence::SyntheticTest, 1)
        );
        assert_eq!(o.committed_counts, counts);
        assert_eq!(self.oracle.sounds, self.oracle.hits);
        let mut output = [None; 64];
        let mut checked = 0;
        let mut cursor = h.key().cursor(0);
        for _ in 0..N.div_ceil(64) {
            let page = frame.read_page(cursor, 64, &mut output).unwrap();
            for value in output.iter().take(page.written) {
                let skipped = checked < self.oracle.first;
                let record = (!skipped).then(|| self.oracle.record(checked));
                assert_eq!(
                    *value,
                    Some(CircleFrame {
                        object: ObjectId::new(checked as u32),
                        phase: Phase::Retired,
                        judgement: record,
                        unavailable: skipped.then_some(UnavailableReason::InspectionSkipped)
                    })
                );
                assert_eq!(run.committed_judgement(s, checked), Ok(record));
                checked += 1;
            }
            if let Some(next) = page.next {
                cursor = next;
            } else {
                assert!(page.end);
                break;
            }
        }
        assert_eq!(checked, N);
        assert!(run.work_counts().valid);
        (counts, self.oracle.sequence - 1, checked)
    }
}
