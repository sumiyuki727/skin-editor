use crate::meter::{Meter, Op};
use skin_editor_gameplay::*;
use std::hint::black_box;
use std::time::{Duration, Instant};

pub const START: i64 = -200_001;
pub const QUERY_BATCH: usize = 4096;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Trace {
    G0,
    G1,
    A1,
}
impl Trace {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "g0" => Some(Self::G0),
            "g1" => Some(Self::G1),
            "a1" => Some(Self::A1),
            _ => None,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::G0 => "g0",
            Self::G1 => "g1",
            Self::A1 => "a1",
        }
    }
}
pub fn t(value: i64) -> MapTime {
    MapTime::from_micros(value)
}
pub fn target(index: usize) -> i64 {
    i64::try_from(index)
        .unwrap()
        .checked_mul(1_000_000)
        .unwrap()
}
pub fn end(n: usize) -> i64 {
    target(n - 1) + 300_000
}
pub fn bound(n: usize) -> usize {
    n * 64 + 256
}
pub fn source() -> RunSource {
    RunSource {
        run: RunId::new(800_001).unwrap(),
        transport: TransportEpoch::new(1).unwrap(),
        input: InputEpoch::new(1).unwrap(),
        revision: MapRevision::new(730),
    }
}
pub fn fixture(n: usize) -> Vec<Circle> {
    (0..n)
        .map(|i| {
            Circle::new(
                ObjectId::new(i as u32),
                t(target(i)),
                Point::new(0, 0).unwrap(),
            )
            .unwrap()
        })
        .collect()
}
#[derive(Default, Clone, Copy)]
pub struct Peaks {
    pub inputs: usize,
    pub events: usize,
    pub source: usize,
    pub runtime: usize,
    pub payload_source: usize,
    pub payload_runtime: usize,
}
impl Peaks {
    pub fn observe(&mut self, owner: &SyntheticCoordinator) {
        let r = owner.resources();
        let a = owner.allocations();
        assert!(a.precompiled_bytes <= r.precompiled_bytes && a.runtime_bytes <= r.runtime_bytes);
        assert!(r.precompiled_bytes <= 8 * 1024 * 1024 && r.runtime_bytes <= 8 * 1024 * 1024);
        self.source = self.source.max(r.precompiled_bytes);
        self.runtime = self.runtime.max(r.runtime_bytes);
        self.payload_source = self.payload_source.max(a.precompiled_bytes);
        self.payload_runtime = self.payload_runtime.max(a.runtime_bytes);
        if let Some(run) = owner.current() {
            let o = run.observation();
            assert!(o.pending_inputs <= 64 && o.pending_events <= 64);
            self.inputs = self.inputs.max(o.pending_inputs);
            self.events = self.events.max(o.pending_events);
        }
    }
}
pub struct Driver {
    pub owner: SyntheticCoordinator,
    pub n: usize,
    pub budget: WorkBudget,
    pub trace: Trace,
    pub events: usize,
    pub peaks: Peaks,
}
impl Driver {
    pub fn create(
        n: usize,
        trace: Trace,
        budget: WorkBudget,
        data: &[Circle],
        meter: &mut Meter,
    ) -> Self {
        let mut owner = SyntheticCoordinator::new(
            CoordinatorId::new(800_001).unwrap(),
            Limits::reference(),
            PreparationLimits::reference(),
        )
        .unwrap();
        let mut peaks = Peaks::default();
        let mut job = meter
            .timed(Op::Request, || {
                owner.request_create(source(), data, t(START))
            })
            .unwrap();
        peaks.observe(&owner);
        let mut ready = false;
        for _ in 0..bound(n) {
            let report = meter
                .timed(Op::Prepare, || job.step(job.cursor(), budget))
                .unwrap();
            peaks.observe(&owner);
            match report {
                PreparationReport::Ready { steps, .. } => {
                    assert!(steps <= budget.get());
                    ready = true;
                    break;
                }
                PreparationReport::Pending { steps, .. } => {
                    assert!(steps > 0 && steps <= budget.get())
                }
            }
        }
        assert!(ready, "create progress bound");
        assert!(job.work_counts().valid);
        assert_eq!(job.work_counts().record_reads, n as u64);
        assert_eq!(job.work_counts().state_initializations, n as u64);
        meter
            .timed(Op::Activate, || owner.activate(job.key(), &mut job))
            .unwrap();
        peaks.observe(&owner);
        meter.timed(Op::TaskDrop, || drop(job));
        Self {
            owner,
            n,
            budget,
            trace,
            events: 0,
            peaks,
        }
    }
    pub fn source(&self) -> RunSource {
        self.owner.current().unwrap().source()
    }
    pub fn drain(&mut self, m: &mut Meter) {
        self.peaks.observe(&self.owner);
        for _ in 0..64 {
            let Some(event) = self.owner.current().unwrap().event_at(0) else {
                return;
            };
            let i = if self.trace == Trace::G0 {
                self.events
            } else {
                self.events / 2
            };
            assert!(i < self.n);
            let kind = if self.trace == Trace::G0 {
                EventKind::Judgement(Judgement::Miss)
            } else if self.events.is_multiple_of(2) {
                EventKind::Judgement(Judgement::Hit)
            } else {
                EventKind::SoundIntent
            };
            assert_eq!(
                event,
                GameEvent {
                    key: EventKey::new(
                        self.source().run,
                        self.source().transport,
                        EventSequence::new(self.events as u64 + 1).unwrap()
                    ),
                    object: ObjectId::new(i as u32),
                    at: t(target(i) + if self.trace == Trace::G0 { 100_001 } else { 0 }),
                    kind,
                    evidence: Evidence::SyntheticTest,
                    profile_version: 1
                }
            );
            m.timed(Op::Ack, || self.owner.acknowledge(event.key))
                .unwrap();
            self.events += 1;
        }
        assert!(self.owner.current().unwrap().event_at(0).is_none());
    }
    fn seal(&mut self, m: &mut Meter, at: i64) {
        m.timed(Op::Seal, || self.owner.seal_through(self.source(), t(at)))
            .unwrap();
    }
    pub fn advance(&mut self, m: &mut Meter, at: i64) {
        self.seal(m, at);
        let mut report = m
            .timed(Op::Advance, || {
                self.owner.advance(self.source(), t(at), self.budget)
            })
            .unwrap();
        for _ in 0..bound(self.n) {
            self.peaks.observe(&self.owner);
            match report {
                AdvanceReport::Complete { through, steps } => {
                    assert_eq!(through, t(at));
                    assert!(steps <= self.budget.get());
                    self.drain(m);
                    return;
                }
                AdvanceReport::Pending {
                    cursor,
                    steps,
                    reason,
                    ..
                } => {
                    assert!(steps <= self.budget.get());
                    if reason == PendingReason::EventBackpressure {
                        assert!(self.owner.current().unwrap().event_at(0).is_some());
                        self.drain(m);
                    } else {
                        assert!(steps > 0);
                    }
                    report = m
                        .timed(Op::Advance, || {
                            self.owner.continue_advance(cursor, self.budget)
                        })
                        .unwrap();
                }
            }
        }
        panic!("advance progress bound");
    }
    fn automatic(&mut self, m: &mut Meter, at: i64) {
        for _ in 0..bound(self.n) {
            let report = m
                .timed(Op::Auto, || {
                    self.owner
                        .prepare_automatic_inputs(self.source(), t(at), self.budget)
                })
                .unwrap();
            self.peaks.observe(&self.owner);
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
                    ready_through,
                    reason,
                    steps,
                    ..
                } => {
                    assert!(steps > 0 && steps <= self.budget.get());
                    if reason == AutomaticPendingReason::InputBackpressure {
                        let prefix = ready_through.expect("safe prefix");
                        assert!(prefix < t(at));
                        self.advance(m, prefix.micros());
                    }
                }
            }
        }
        panic!("automatic progress bound");
    }
    pub fn freeze(&mut self, m: &mut Meter, at: i64) {
        m.timed(Op::Freeze, || {
            self.owner.request_freeze(self.source(), t(at))
        })
        .unwrap();
        let sealed = self.owner.current().unwrap().observation().sealed_through;
        if sealed.is_none_or(|v| v < t(at)) {
            self.seal(m, at);
        }
        for _ in 0..bound(self.n) {
            let report = m
                .timed(Op::Freeze, || {
                    self.owner.settle_freeze(self.source(), self.budget)
                })
                .unwrap();
            self.peaks.observe(&self.owner);
            match report {
                FreezeReport::Frozen { at: done, .. } => {
                    assert_eq!(done, t(at));
                    return;
                }
                FreezeReport::Pending { reason, steps, .. } => {
                    if reason == PendingReason::EventBackpressure {
                        self.drain(m);
                    } else {
                        assert!(steps > 0 && steps <= self.budget.get());
                    }
                }
            }
        }
        panic!("freeze progress bound");
    }
    pub fn play(&mut self, m: &mut Meter) {
        if self.trace == Trace::A1 {
            self.freeze(m, START);
            let selected = m
                .timed(Op::Freeze, || {
                    self.owner
                        .select_driver(self.source(), InputDriver::Automatic)
                })
                .unwrap();
            m.timed(Op::Freeze, || self.owner.resume(selected)).unwrap();
            assert_eq!(self.source().input.get(), 3);
        }
        if self.trace != Trace::G0 {
            for first in (0..self.n).step_by(32) {
                let count = (self.n - first).min(32);
                let at = target(first + count - 1) + 1;
                if self.trace == Trace::A1 {
                    self.automatic(m, at);
                } else {
                    let empty = InputEvent {
                        sequence: InputSequence::new(1).unwrap(),
                        at: t(START),
                        slot: PhysicalSlot::PrimaryOne,
                        edge: InputEdge::Release,
                    };
                    let mut batch = [empty; 64];
                    for offset in 0..count {
                        let i = first + offset;
                        let seq = 2 * i as u64 + 1;
                        batch[2 * offset] = InputEvent {
                            sequence: InputSequence::new(seq).unwrap(),
                            at: t(target(i)),
                            slot: PhysicalSlot::PrimaryOne,
                            edge: InputEdge::Press(Point::new(0, 0).unwrap()),
                        };
                        batch[2 * offset + 1] = InputEvent {
                            sequence: InputSequence::new(seq + 1).unwrap(),
                            at: t(target(i) + 1),
                            slot: PhysicalSlot::PrimaryOne,
                            edge: InputEdge::Release,
                        };
                    }
                    assert_eq!(
                        m.timed(Op::Submit, || self
                            .owner
                            .submit_inputs(self.source(), &batch[..2 * count]))
                            .unwrap(),
                        Admission {
                            added: 2 * count,
                            repeated: 0
                        }
                    );
                    self.peaks.observe(&self.owner);
                }
                self.advance(m, at);
            }
        }
        if self.trace == Trace::A1 {
            self.automatic(m, end(self.n));
        }
        self.advance(m, end(self.n));
        assert_eq!(
            self.events,
            self.n * if self.trace == Trace::G0 { 1 } else { 2 }
        );
        let state = self.owner.current().unwrap().observation();
        assert_eq!(state.status, RunStatus::Ready);
        assert_eq!(
            state.committed_counts,
            FrameCounts {
                hits: if self.trace == Trace::G0 { 0 } else { self.n },
                misses: if self.trace == Trace::G0 { self.n } else { 0 },
                inspection_skipped: 0
            }
        );
    }
    pub fn query(&mut self, m: &mut Meter) {
        let run = self.owner.current().unwrap();
        let s = run.source();
        let header = run.observation().ready_frame.unwrap();
        assert_eq!(header.source, s);
        assert_eq!(header.at, t(end(self.n)));
        for _ in 0..256 {
            assert_eq!(
                m.timed(Op::Summary, || black_box(run).observation())
                    .ready_frame,
                Some(header)
            );
        }
        let mut out = [None; 64];
        let mut cursor = header.key().cursor(0);
        let mut checked = 0;
        for _ in 0..self.n.div_ceil(64) {
            let page = m
                .timed(Op::Page, || {
                    black_box(run).read_frame_page(black_box(cursor), 64, &mut out)
                })
                .unwrap()
                .unwrap();
            for value in out.iter().take(page.written) {
                let record = JudgementRecord {
                    object: ObjectId::new(checked as u32),
                    at: t(target(checked) + if self.trace == Trace::G0 { 100_001 } else { 0 }),
                    result: if self.trace == Trace::G0 {
                        Judgement::Miss
                    } else {
                        Judgement::Hit
                    },
                };
                assert_eq!(
                    *value,
                    Some(CircleFrame {
                        object: record.object,
                        phase: Phase::Retired,
                        judgement: Some(record),
                        unavailable: None
                    })
                );
                assert_eq!(
                    m.timed(Op::Ledger, || black_box(run)
                        .committed_judgement(black_box(s), black_box(checked)))
                        .unwrap(),
                    Some(record)
                );
                checked += 1;
            }
            if let Some(next) = page.next {
                cursor = next;
            } else {
                assert!(page.end);
                break;
            }
        }
        assert_eq!(checked, self.n);
        m.timed(Op::QueryBatch, || {
            for _ in 0..QUERY_BATCH {
                let page = black_box(run)
                    .read_frame_page(black_box(header.key().cursor(0)), 64, &mut out)
                    .unwrap()
                    .unwrap();
                assert_eq!(page.written, self.n.min(64));
                black_box(&out);
            }
        });
        assert!(run.work_counts().valid);
    }
    pub fn inspect(&mut self, m: &mut Meter) {
        self.freeze(m, end(self.n));
        let at = target(self.n / 2);
        let mut job = m
            .timed(Op::InspectRequest, || {
                self.owner.request_inspection(self.source(), t(at))
            })
            .unwrap();
        let mut ready = false;
        for _ in 0..2 * self.n + 128 {
            let report = m
                .timed(Op::InspectStep, || job.step(job.cursor(), self.budget))
                .unwrap();
            self.peaks.observe(&self.owner);
            if matches!(report, PreparationReport::Ready { .. }) {
                ready = true;
                break;
            }
        }
        assert!(ready, "inspection progress bound");
        assert_eq!(job.work_counts().inspection_objects, self.n as u64);
        m.timed(Op::InspectActivate, || {
            self.owner.activate(job.key(), &mut job)
        })
        .unwrap();
        self.peaks.observe(&self.owner);
        m.timed(Op::TaskDrop, || drop(job));
        let state = self.owner.current().unwrap().observation();
        assert_eq!(state.status, RunStatus::Frozen);
        assert_eq!(state.pending_events, 0);
        assert_eq!(
            state.committed_counts,
            FrameCounts {
                inspection_skipped: self.n / 2,
                ..FrameCounts::default()
            }
        );
        assert_eq!(
            state.statistics_origin,
            StatisticsOrigin::Inspection { at: t(at) }
        );
    }
}

/// Bounded representative prepare + advance, no per-call instrumentation or I/O.
fn reference(n: usize) -> u64 {
    let data = fixture(n);
    let mut owner = SyntheticCoordinator::new(
        CoordinatorId::new(900_001).unwrap(),
        Limits::reference(),
        PreparationLimits::reference(),
    )
    .unwrap();
    let mut task = owner.request_create(source(), &data, t(START)).unwrap();
    let mut steps = 0;
    let mut ready = false;
    for _ in 0..bound(n) {
        match task.step(task.cursor(), WorkBudget::full()).unwrap() {
            PreparationReport::Ready { steps: s, .. } => {
                steps += s as u64;
                ready = true;
                break;
            }
            PreparationReport::Pending { steps: s, .. } => steps += s as u64,
        }
    }
    assert!(ready);
    assert_eq!(task.work_counts().record_reads, n as u64);
    owner.activate(task.key(), &mut task).unwrap();
    drop(task);
    owner.seal_through(source(), t(end(n))).unwrap();
    let mut report = owner
        .advance(source(), t(end(n)), WorkBudget::full())
        .unwrap();
    let mut events = 0;
    let mut complete = false;
    for _ in 0..bound(n) {
        for _ in 0..64 {
            let Some(event) = owner.current().unwrap().event_at(0) else {
                break;
            };
            assert_eq!(
                (event.object, event.kind),
                (ObjectId::new(events), EventKind::Judgement(Judgement::Miss))
            );
            owner.acknowledge(event.key).unwrap();
            events += 1;
        }
        match report {
            AdvanceReport::Complete { steps: s, .. } => {
                steps += s as u64;
                complete = true;
                break;
            }
            AdvanceReport::Pending {
                steps: s, cursor, ..
            } => {
                steps += s as u64;
                report = owner.continue_advance(cursor, WorkBudget::full()).unwrap();
            }
        }
    }
    assert!(complete);
    assert_eq!(events, n as u32);
    steps
}
pub struct Calibration {
    pub resolution: u64,
    pub empty_span: u64,
    pub small_ns: u64,
    pub large_ns: u64,
    pub small_work: u64,
    pub large_work: u64,
    pub cpu_ns: u64,
    pub cpu_cycles: usize,
    pub valid: bool,
}
pub fn calibrate(m: &mut Meter) -> Calibration {
    let mut resolution = u64::MAX;
    let mut previous = Instant::now();
    for _ in 0..20_000 {
        let now = Instant::now();
        let delta = now.duration_since(previous).as_nanos() as u64;
        if delta > 0 {
            resolution = resolution.min(delta);
        }
        previous = now;
    }
    let begin = Instant::now();
    for _ in 0..20_000 {
        m.timed(Op::Empty, || black_box(()));
    }
    let empty_span = begin.elapsed().as_nanos() as u64;
    let begin = Instant::now();
    let mut small_work = 0;
    for _ in 0..4 {
        small_work += reference(32);
    }
    let small_ns = begin.elapsed().as_nanos() as u64;
    let begin = Instant::now();
    let mut large_work = 0;
    for _ in 0..4 {
        large_work += reference(2048);
    }
    let large_ns = begin.elapsed().as_nanos() as u64;
    let begin = Instant::now();
    let mut cpu_cycles = 0;
    for _ in 0..100_000 {
        black_box(reference(256));
        cpu_cycles += 1;
        if begin.elapsed() >= Duration::from_secs(2) {
            break;
        }
    }
    let cpu_ns = begin.elapsed().as_nanos() as u64;
    let valid = resolution != u64::MAX
        && small_ns > resolution.saturating_mul(100)
        && large_ns > resolution.saturating_mul(100)
        && large_work > small_work
        && cpu_ns >= 2_000_000_000;
    Calibration {
        resolution,
        empty_span,
        small_ns,
        large_ns,
        small_work,
        large_work,
        cpu_ns,
        cpu_cycles,
        valid,
    }
}
