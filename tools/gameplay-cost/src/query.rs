//! Fixed query batches. No PC batches, devices, I/O sources, or product verdicts.
use crate::driver::{Driver, Setup, Trace, end, target};
use crate::meter::{Meter, NAMES, percentile};
use crate::write_work;
use skin_editor_gameplay::*;
use std::hint::black_box;
use std::io::{self, Write};
use std::time::Instant;

pub const PLAN: &str = "CostClosure-Query-v1";
pub const REPEATS: usize = 16;
pub const IDS: [&str; 3] = ["Q-SUMMARY-4096", "Q-LEDGER-N", "Q-PAGE-4096"];
pub const STORAGE_LIMIT: usize = 1024 * 1024;

#[derive(Clone, Copy, Default)]
struct Sample {
    ns: u64,
    before: WorkCounts,
    after: WorkCounts,
    delta: WorkCounts,
}
#[derive(Clone, Copy, Default)]
struct Series {
    samples: [Sample; REPEATS],
    references: [u64; REPEATS],
    len: usize,
    reference_len: usize,
}
pub struct Storage {
    series: [Series; 3],
    output: [Option<CircleFrame>; 64],
    frame: Option<FrameHeader>,
}
pub enum Receipt<'a> {
    Setup(&'a Meter),
    Batch(usize),
    Released,
    Summary,
}

pub fn base_id(ordinal: usize) -> u64 {
    assert!(ordinal < 12);
    10_000_000_u64
        .checked_add(1000_u64.checked_mul(ordinal as u64).unwrap())
        .unwrap()
}
pub fn storage_bytes() -> usize {
    // Caller descriptors/temporary input and counters are included, not just time slots.
    std::mem::size_of::<Storage>()
        .checked_add(std::mem::size_of::<Driver>())
        .unwrap()
        .checked_add(std::mem::size_of::<Meter>())
        .unwrap()
        .checked_add(64 * std::mem::size_of::<InputEvent>())
        .unwrap()
}
pub fn admit(n: usize, tiny: bool, available: usize) -> Result<(), &'static str> {
    if storage_bytes() > available
        || available > STORAGE_LIMIT
        || (tiny && n != 4)
        || (!tiny && n != 10_000)
    {
        return Err("query capacity/configuration");
    }
    let c = SyntheticCoordinator::costs_for_objects(n).map_err(|_| "query costs")?;
    let runtime = c
        .pool_bytes
        .checked_add(c.coordinator_bytes)
        .and_then(|v| v.checked_add(c.task_bytes))
        .and_then(|v| v.checked_add(c.run_bytes))
        .ok_or("query cost overflow")?;
    let limit = if tiny { 64 * 1024 } else { 8 * 1024 * 1024 };
    if c.source_bytes > limit
        || runtime > limit
        || c.run_bytes > limit
        || c.source_bytes
            .checked_add(runtime)
            .ok_or("aggregate overflow")?
            > 64 * 1024 * 1024
    {
        return Err("query pool admission");
    }
    Ok(())
}
pub fn establish(
    n: usize,
    trace: Trace,
    budget: WorkBudget,
    data: &[Circle],
    setup: Setup,
    m: &mut Meter,
) -> Driver {
    admit(n, setup.tiny, STORAGE_LIMIT).unwrap();
    let mut d = Driver::create_with(n, trace, budget, data, m, setup);
    d.play(m);
    assert!(d.owner.candidate().is_none());
    d
}
fn difference(before: WorkCounts, after: WorkCounts) -> WorkCounts {
    assert!(before.valid && after.valid);
    let mut delta = WorkCounts::default();
    macro_rules! fields {($($f:ident),*) => {$(delta.$f = after.$f.checked_sub(before.$f).expect("monotonic query work");)*};}
    fields!(
        record_reads,
        record_copies,
        index_initializations,
        id_comparisons,
        index_moves,
        state_initializations,
        search_objects,
        inspection_objects,
        automatic_objects,
        publications,
        summaries,
        frame_objects,
        ledger_objects,
        management_steps,
        allocation_attempts,
        allocation_bytes
    );
    delta
}
fn expected_delta(kind: usize, n: usize) -> WorkCounts {
    let mut delta = WorkCounts::default();
    match kind {
        0 => delta.summaries = 4096,
        1 => delta.ledger_objects = n as u64,
        2 => delta.frame_objects = (4096 * n.min(64)) as u64,
        _ => panic!("query kind"),
    }
    delta
}
fn record(trace: Trace, i: usize) -> JudgementRecord {
    JudgementRecord {
        object: ObjectId::new(i as u32),
        at: MapTime::from_micros(target(i) + if trace == Trace::G0 { 100_001 } else { 0 }),
        result: if trace == Trace::G0 {
            Judgement::Miss
        } else {
            Judgement::Hit
        },
    }
}
fn page(header: FrameHeader) -> FramePage {
    FramePage {
        frame: header.key(),
        written: header.len().min(64),
        next: (header.len() > 64).then(|| header.key().cursor(64)),
        end: header.len() <= 64,
    }
}
fn body(kind: usize, d: &Driver, expected: Observation, out: &mut [Option<CircleFrame>; 64]) {
    let run = d.owner.current().unwrap();
    let header = expected.ready_frame.unwrap();
    match kind {
        0 => {
            for _ in 0..4096 {
                assert_eq!(black_box(black_box(run).observation()), expected);
            }
        }
        1 => {
            for i in 0..d.n {
                assert_eq!(
                    black_box(
                        black_box(run)
                            .committed_judgement(black_box(expected.source), black_box(i))
                    ),
                    Ok(Some(record(d.trace, i)))
                );
            }
        }
        2 => {
            for _ in 0..4096 {
                let got = black_box(black_box(run).read_frame_page(
                    black_box(header.key().cursor(0)),
                    black_box(64),
                    out,
                ));
                assert_eq!(got, Ok(Some(page(header))));
                for (i, value) in out.iter().take(d.n.min(64)).enumerate() {
                    assert_eq!(
                        *value,
                        Some(CircleFrame {
                            object: ObjectId::new(i as u32),
                            phase: Phase::Retired,
                            judgement: Some(record(d.trace, i)),
                            unavailable: None
                        })
                    );
                }
                black_box(&*out);
            }
        }
        _ => panic!("query kind"),
    }
}
fn empty_body(kind: usize, d: &Driver, expected: Observation, out: &mut [Option<CircleFrame>; 64]) {
    // Outer loop/result/barriers only. No API calls, comparisons or subtraction.
    match kind {
        0 => {
            for _ in 0..4096 {
                black_box(black_box(expected));
            }
        }
        1 => {
            for i in 0..d.n {
                black_box(expected.source);
                black_box(i);
                let _ = black_box(black_box(Ok::<_, FrameReadError>(Some(record(d.trace, i)))));
            }
        }
        2 => {
            for _ in 0..4096 {
                black_box(expected.ready_frame.unwrap().key().cursor(0));
                black_box(64);
                let _ = black_box(black_box(Ok::<_, FrameReadError>(Some(page(
                    expected.ready_frame.unwrap(),
                )))));
                black_box(&*out);
            }
        }
        _ => panic!("query kind"),
    }
}
impl Storage {
    pub fn page_buffer(&mut self) -> &mut [Option<CircleFrame>; 64] {
        &mut self.output
    }
    pub fn new() -> Self {
        assert!(storage_bytes() <= STORAGE_LIMIT);
        let mut value = Self {
            series: [Series::default(); 3],
            output: [None; 64],
            frame: None,
        };
        for series in &mut value.series {
            for sample in &mut series.samples {
                sample.ns = 1;
            }
            series.references.fill(1);
        }
        black_box(&value);
        for series in &mut value.series {
            for sample in &mut series.samples {
                sample.ns = 0;
            }
            series.references.fill(0);
        }
        value
    }
    pub fn bind(&mut self, d: &Driver) {
        let o = d.owner.current().unwrap().observation();
        assert_eq!(o.status, RunStatus::Ready);
        assert_eq!(o.pending_inputs, 0);
        assert_eq!(o.pending_events, 0);
        let h = o.ready_frame.unwrap();
        assert_eq!(h.at, MapTime::from_micros(end(d.n)));
        assert_eq!(h.len(), d.n);
        assert_eq!(h.source, d.source());
        self.frame = Some(h);
    }
    fn sample(&mut self, kind: usize, d: &Driver, timed: bool) {
        let index = self.series[kind].len;
        assert!(index < REPEATS, "query sample capacity before work");
        let run = d.owner.current().unwrap();
        let expected = run.observation();
        assert_eq!(expected.ready_frame, self.frame);
        let resources = d.owner.resources();
        let allocations = d.owner.allocations();
        let before = run.work_counts();
        // Functional canaries take neither branch of Instant measurement.
        let begin = timed.then(Instant::now);
        body(kind, d, expected, &mut self.output);
        let ns = begin.map_or(0, |b| u64::try_from(b.elapsed().as_nanos()).unwrap());
        let after = run.work_counts();
        let delta = difference(before, after);
        assert_eq!(delta, expected_delta(kind, d.n));
        assert_eq!(run.observation(), expected);
        assert_eq!(d.owner.resources(), resources);
        assert_eq!(d.owner.allocations(), allocations);
        assert!(d.owner.candidate().is_none());
        self.series[kind].samples[index] = Sample {
            ns,
            before,
            after,
            delta,
        };
        self.series[kind].len += 1;
    }
    pub fn measure(&mut self, kind: usize, d: &Driver) {
        for _ in 0..REPEATS {
            self.sample(kind, d, true);
        }
        let expected = d.owner.current().unwrap().observation();
        let before = d.owner.current().unwrap().work_counts();
        let series = &mut self.series[kind];
        for ns in &mut series.references {
            let begin = Instant::now();
            empty_body(kind, d, expected, &mut self.output);
            *ns = u64::try_from(begin.elapsed().as_nanos()).unwrap();
            series.reference_len += 1;
        }
        assert_eq!(before, d.owner.current().unwrap().work_counts());
    }
    pub fn write(
        &self,
        out: &mut impl Write,
        receipt: Receipt<'_>,
        d: Option<&Driver>,
    ) -> io::Result<()> {
        match receipt {
            Receipt::Setup(m) => {
                write!(out, "{{\"kind\":\"setup\",\"frame\":")?;
                write_frame(out, self.frame.unwrap())?;
                write!(out, ",\"prepared_work\":")?;
                write_work(out, d.unwrap().prepared_work)?;
                write!(out, ",\"calls\":{{")?;
                for (i, name) in NAMES.iter().enumerate() {
                    if i > 0 {
                        write!(out, ",")?;
                    }
                    write!(out, "\"{name}\":{}", m.calls()[i])?;
                }
                write!(out, "}}}}")
            }
            Receipt::Batch(kind) => {
                let s = &self.series[kind];
                assert_eq!(s.len, REPEATS);
                assert_eq!(s.reference_len, REPEATS);
                write!(
                    out,
                    "{{\"kind\":\"batch\",\"id\":\"{}\",\"frame\":",
                    IDS[kind]
                )?;
                write_frame(out, self.frame.unwrap())?;
                write!(
                    out,
                    ",\"state_unchanged\":true,\"resource_unchanged\":true,\"reference_scope\":\"outer_loop_only\",\"guard_summaries\":34,\"samples\":["
                )?;
                for (i, sample) in s.samples.iter().enumerate() {
                    if i > 0 {
                        write!(out, ",")?;
                    }
                    write!(
                        out,
                        "{{\"ordinal\":{i},\"ns\":{},\"checked\":true,\"before\":",
                        sample.ns
                    )?;
                    write_work(out, sample.before)?;
                    write!(out, ",\"after\":")?;
                    write_work(out, sample.after)?;
                    write!(out, ",\"delta\":")?;
                    write_work(out, sample.delta)?;
                    write!(out, "}}")?;
                }
                write!(out, "],\"timing\":")?;
                write_times(out, &s.samples.map(|x| x.ns))?;
                write!(out, ",\"reference\":")?;
                write_times(out, &s.references)?;
                write!(out, "}}")
            }
            Receipt::Released => write!(out, "{{\"kind\":\"released\",\"owners_dropped\":true}}"),
            Receipt::Summary => {
                write!(
                    out,
                    "{{\"kind\":\"summary\",\"scope\":\"QueryOnly\",\"coverage\":\"NeedsEvidence\",\"product_verdict\":\"NotEvaluated\",\"batches\":["
                )?;
                for (i, s) in self.series.iter().enumerate() {
                    assert_eq!(s.len, REPEATS);
                    if i > 0 {
                        write!(out, ",")?;
                    }
                    write!(out, "{{\"id\":\"{}\",\"timing\":", IDS[i])?;
                    write_times(out, &s.samples.map(|x| x.ns))?;
                    write!(out, ",\"reference\":")?;
                    write_times(out, &s.references)?;
                    write!(out, "}}")?;
                }
                write!(out, "]}}")
            }
        }
    }
    /// Failure output reuses retained bounded slots, without normal ACK/sample admission.
    pub fn write_failure(&self, out: &mut impl Write) -> io::Result<()> {
        write!(out, "[")?;
        for (kind, series) in self.series.iter().enumerate() {
            if kind > 0 {
                write!(out, ",")?;
            }
            write!(out, "{{\"id\":\"{}\",\"samples\":[", IDS[kind])?;
            for (i, sample) in series.samples.iter().take(series.len).enumerate() {
                if i > 0 {
                    write!(out, ",")?;
                }
                write!(out, "{{\"ordinal\":{i},\"ns\":{},\"before\":", sample.ns)?;
                write_work(out, sample.before)?;
                write!(out, ",\"after\":")?;
                write_work(out, sample.after)?;
                write!(out, ",\"delta\":")?;
                write_work(out, sample.delta)?;
                write!(out, "}}")?;
            }
            write!(out, "],\"reference_ns\":[")?;
            for (i, ns) in series
                .references
                .iter()
                .take(series.reference_len)
                .enumerate()
            {
                if i > 0 {
                    write!(out, ",")?;
                }
                write!(out, "{ns}")?;
            }
            write!(out, "]}}")?;
        }
        write!(out, "]")
    }
}
fn write_times(out: &mut impl Write, raw: &[u64; REPEATS]) -> io::Result<()> {
    let mut sorted = *raw;
    sorted.sort_unstable();
    let total = raw
        .iter()
        .try_fold(0_u64, |s, v| s.checked_add(*v))
        .expect("bounded timing sum");
    write!(
        out,
        "{{\"n\":16,\"sum_ns\":{total},\"p50_ns\":{},\"p95_ns\":{},\"p99_ns\":{},\"max_ns\":{},\"raw_ns\":[",
        percentile(&sorted, 50).unwrap(),
        percentile(&sorted, 95).unwrap(),
        percentile(&sorted, 99).unwrap(),
        sorted[15]
    )?;
    for (i, value) in raw.iter().enumerate() {
        if i > 0 {
            write!(out, ",")?;
        }
        write!(out, "{value}")?;
    }
    write!(out, "]}}")
}
fn write_frame(out: &mut impl Write, h: FrameHeader) -> io::Result<()> {
    let StatisticsOrigin::FromStart { start } = h.statistics_origin else {
        panic!("query from-start origin");
    };
    write!(
        out,
        "{{\"run\":{},\"transport\":{},\"input\":{},\"revision\":{},\"at\":{},\"publication\":{},\"objects\":{},\"hits\":{},\"misses\":{},\"skipped\":{},\"start\":{},\"evidence\":\"SyntheticTest\",\"profile_version\":1}}",
        h.source.run.get(),
        h.source.transport.get(),
        h.source.input.get(),
        h.source.revision.get(),
        h.at.micros(),
        h.key().publication(),
        h.len(),
        h.counts.hits,
        h.counts.misses,
        h.counts.inspection_skipped,
        start.micros()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::driver::fixture;
    #[test]
    fn capacity_and_identity_are_checked_before_group_work() {
        assert!(admit(10000, false, storage_bytes()).is_ok());
        assert!(admit(10000, false, storage_bytes() - 1).is_err());
        assert!(admit(4, true, STORAGE_LIMIT).is_ok());
        for (n, tiny) in [(5, true), (10001, false), (4, false)] {
            assert!(admit(n, tiny, STORAGE_LIMIT).is_err());
        }
        assert_eq!(base_id(11) + 2, 10_011_002);
        assert!(std::panic::catch_unwind(|| base_id(12)).is_err());
    }
    #[test]
    fn real_tiny_all_traces_keep_four_two_four_limits_without_clocks() {
        for trace in [Trace::G0, Trace::G1, Trace::A1] {
            exercise(4, trace, WorkBudget::one(), true);
        }
    }
    #[test]
    fn reference_all_traces_and_budgets_preserve_every_query_sample() {
        for trace in [Trace::G0, Trace::G1, Trace::A1] {
            for budget in [WorkBudget::one(), WorkBudget::full()] {
                exercise(10000, trace, budget, false);
            }
        }
    }
    fn exercise(n: usize, trace: Trace, budget: WorkBudget, tiny: bool) {
        let data = fixture(n);
        let mut m = Meter::unmeasured();
        let d = establish(
            n,
            trace,
            budget,
            &data,
            Setup {
                id: base_id(0) + 2,
                tiny,
            },
            &mut m,
        );
        let mut storage = Storage::new();
        storage.bind(&d);
        let calls = *m.calls();
        let before = d.owner.resources();
        for kind in 0..3 {
            for _ in 0..REPEATS {
                storage.sample(kind, &d, false);
            }
            let work = d.owner.current().unwrap().work_counts();
            assert!(
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(
                    || storage.sample(kind, &d, false)
                ))
                .is_err()
            );
            assert_eq!(d.owner.current().unwrap().work_counts(), work);
            assert_eq!(storage.series[kind].len, 16);
            assert!(
                storage.series[kind]
                    .samples
                    .iter()
                    .all(|s| s.ns == 0 && s.delta == expected_delta(kind, n))
            );
        }
        assert_eq!(d.owner.resources(), before);
        assert_eq!(*m.calls(), calls);
        assert_eq!(m.bytes, 0);
        assert!(d.owner.candidate().is_none());
        let mut retained = Vec::new();
        storage.write_failure(&mut retained).unwrap();
        assert!(retained.len() < 256 * 1024);
        let text = String::from_utf8(retained).unwrap();
        assert_eq!(text.matches("\"ordinal\":15").count(), 3);
        assert_eq!(text.matches("\"reference_ns\":[]").count(), 3);
    }
    #[test]
    fn wrong_golden_and_changed_nonquery_work_cannot_be_accepted() {
        let data = fixture(4);
        let mut m = Meter::unmeasured();
        let mut d = establish(
            4,
            Trace::G1,
            WorkBudget::one(),
            &data,
            Setup { id: 99, tiny: true },
            &mut m,
        );
        let mut s = Storage::new();
        s.bind(&d);
        d.trace = Trace::G0;
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| s.sample(1, &d, false)))
                .is_err()
        );
        assert_eq!(s.series[1].len, 0);
        let before = WorkCounts::default();
        let mut after = before;
        after.summaries = 4096;
        after.publications = 1;
        assert_ne!(difference(before, after), expected_delta(0, 4));
    }
}
