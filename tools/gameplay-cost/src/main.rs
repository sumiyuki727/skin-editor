//! Independent, versioned, headless SyntheticTest cost probe.
#![forbid(unsafe_code)]
mod driver;
mod meter;
mod pc;
mod query;
use driver::{Calibration, Driver, Peaks, Trace, bound, calibrate, fixture, t};
use meter::{Meter, Op};
use skin_editor_gameplay::*;
use std::io::{self, BufRead, Read, Write};
use std::time::Instant;

const SCHEMA: u32 = 3;
type Receipts<'a> = (
    Option<&'a Calibration>,
    Option<(&'a query::Storage, query::Receipt<'a>)>,
    Option<(&'a pc::Storage, pc::Receipt)>,
);
struct Context {
    plan: &'static str,
    case_ordinal: usize,
    trace: Trace,
    budget: WorkBudget,
    n: usize,
    previous_emit: u64,
    previous_gate: u64,
    fixture_bytes: usize,
    peaks: Peaks,
    candidate: Option<WorkCounts>,
    events_checked: usize,
}
fn write_work(out: &mut impl Write, w: WorkCounts) -> io::Result<()> {
    write!(out, "{{\"valid\":{}", w.valid)?;
    macro_rules! fields {($($field:ident),*)=>{$(write!(out,",\"{}\":{}",stringify!($field),w.$field)?;)*};}
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
    write!(out, "}}")
}
impl Context {
    fn boundary(
        &mut self,
        phase: &str,
        elapsed: u64,
        m: &Meter,
        state: Option<&Driver>,
        calibration: Option<&Calibration>,
    ) -> io::Result<()> {
        self.emit(phase, elapsed, m, state, (calibration, None, None))?;
        self.acknowledge()
    }
    fn query_boundary(
        &mut self,
        phase: &str,
        elapsed: u64,
        m: &Meter,
        state: Option<&Driver>,
        storage: &query::Storage,
        receipt: query::Receipt<'_>,
    ) -> io::Result<()> {
        self.emit(
            phase,
            elapsed,
            m,
            state,
            (None, Some((storage, receipt)), None),
        )?;
        self.acknowledge()
    }
    fn pc_boundary(
        &mut self,
        phase: &str,
        elapsed: u64,
        m: &Meter,
        storage: &pc::Storage,
        receipt: pc::Receipt,
    ) -> io::Result<()> {
        self.emit(
            phase,
            elapsed,
            m,
            None,
            (None, None, Some((storage, receipt))),
        )?;
        self.acknowledge()
    }
    fn acknowledge(&mut self) -> io::Result<()> {
        let begin = Instant::now();
        let input = io::stdin();
        let mut reader = input.lock().take(32);
        let mut line = String::new();
        reader.read_line(&mut line)?;
        if line != "continue\n" {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "bounded phase acknowledgement required",
            ));
        }
        self.previous_gate = begin.elapsed().as_nanos() as u64;
        Ok(())
    }
    fn emit(
        &mut self,
        phase: &str,
        elapsed: u64,
        m: &Meter,
        state: Option<&Driver>,
        receipts: Receipts<'_>,
    ) -> io::Result<()> {
        let begin = Instant::now();
        let output = io::stdout();
        let mut out = io::BufWriter::new(output.lock());
        self.write_row(&mut out, (phase, elapsed), m, state, receipts)?;
        out.flush()?;
        self.previous_emit = begin.elapsed().as_nanos() as u64;
        Ok(())
    }
    fn write_row(
        &mut self,
        mut out: impl Write,
        (phase, elapsed): (&str, u64),
        m: &Meter,
        state: Option<&Driver>,
        (calibration, query, pc): Receipts<'_>,
    ) -> io::Result<()> {
        if let Some(s) = state {
            self.events_checked = s.events;
        }
        write!(
            out,
            "{{\"schema_version\":{SCHEMA},\"phase\":\"{phase}\",\"trace\":\"{}\",\"budget\":{},\"objects\":{},\"api_version\":{},\"profile_version\":{},\"status\":\"{}\",\"elapsed_ns\":{elapsed},\"previous_emit_ns\":{},\"previous_gate_ns\":{},\"sample_storage_bytes\":{},\"fixture_bytes\":{},\"owner_alive\":{},\"events_checked\":{},\"managed\":",
            self.trace.name(),
            self.budget.get(),
            self.n,
            GAMEPLAY_API_VERSION,
            SYNTHETIC_PROFILE_VERSION,
            if m.invalid.is_some() {
                "Invalid"
            } else {
                "Valid"
            },
            self.previous_emit,
            self.previous_gate,
            m.bytes,
            self.fixture_bytes,
            state.is_some() || pc.is_some_and(|(s, _)| s.owners_alive()),
            self.events_checked
        )?;
        if let Some(s) = state {
            let r = s.owner.resources();
            let a = s.owner.allocations();
            let source = s.source();
            let o = s.owner.current().unwrap().observation();
            write!(
                out,
                "{{\"source_reserved\":{},\"runtime_reserved\":{},\"source_payload\":{},\"runtime_payload\":{},\"source_allocations\":{},\"runtime_allocations\":{},\"hits\":{},\"misses\":{},\"skipped\":{},\"pending_inputs\":{},\"pending_events\":{},\"run\":{},\"transport\":{},\"input\":{},\"revision\":{},\"source_capacity\":{}}}",
                r.precompiled_bytes,
                r.runtime_bytes,
                a.precompiled_bytes,
                a.runtime_bytes,
                a.precompiled_allocations,
                a.runtime_allocations,
                o.committed_counts.hits,
                o.committed_counts.misses,
                o.committed_counts.inspection_skipped,
                o.pending_inputs,
                o.pending_events,
                source.run.get(),
                source.transport.get(),
                source.input.get(),
                source.revision.get(),
                s.owner.current().unwrap().source_capacity()
            )?;
        } else {
            write!(out, "null")?;
        }
        write!(
            out,
            ",\"peaks\":{{\"inputs\":{},\"events\":{},\"source_reserved\":{},\"runtime_reserved\":{},\"source_payload\":{},\"runtime_payload\":{}}},\"work_counts\":",
            self.peaks.inputs,
            self.peaks.events,
            self.peaks.source,
            self.peaks.runtime,
            self.peaks.payload_source,
            self.peaks.payload_runtime
        )?;
        if let Some(s) = state {
            write_work(&mut out, s.owner.current().unwrap().work_counts())?;
        } else {
            write!(out, "null")?;
        }
        write!(out, ",\"candidate_work\":")?;
        if let Some(w) = self.candidate {
            write_work(&mut out, w)?;
        } else {
            write!(out, "null")?;
        }
        write!(out, ",\"calibration\":")?;
        if let Some(c) = calibration {
            write!(
                out,
                "{{\"valid\":{},\"observed_resolution_ns\":{},\"empty_span_ns\":{},\"small_reference_ns\":{},\"large_reference_ns\":{},\"small_work\":{},\"large_work\":{},\"cpu_reference_ns\":{},\"cpu_reference_cycles\":{},\"cpu_reference_objects\":256}}",
                c.valid,
                c.resolution,
                c.empty_span,
                c.small_ns,
                c.large_ns,
                c.small_work,
                c.large_work,
                c.cpu_ns,
                c.cpu_cycles
            )?;
        } else {
            write!(out, "null")?;
        }
        write!(
            out,
            ",\"target_arch\":\"{}\",\"target_os\":\"{}\",\"pointer_bits\":{}",
            std::env::consts::ARCH,
            std::env::consts::OS,
            std::mem::size_of::<usize>() * 8
        )?;
        write!(
            out,
            ",\"plan_id\":\"{}\",\"case_ordinal\":{},\"query_storage_bytes\":{},\"query\":",
            self.plan,
            self.case_ordinal,
            query::storage_bytes()
        )?;
        if let Some((storage, receipt)) = query {
            storage.write(&mut out, receipt, state)?;
        } else {
            write!(out, "null")?;
        }
        assert!(matches!(self.plan, pc::PLAN | query::PLAN));
        if self.plan == pc::PLAN {
            write!(
                out,
                ",\"closure_storage_bytes\":{},\"pc\":",
                pc::storage_bytes()
            )?;
            if let Some((storage, receipt)) = pc {
                storage.write(&mut out, receipt)?;
            } else {
                write!(out, "null")?;
            }
        }
        write!(out, ",\"metrics\":")?;
        m.json(&mut out)?;
        writeln!(out, "}}")
    }
}
fn run(context: &mut Context, m: &mut Meter, storage: &mut query::Storage, pc: &mut pc::Storage) {
    pc::admit(context.n, false, query::STORAGE_LIMIT, pc::AGGREGATE_LIMIT).unwrap();
    query::admit(context.n, false, query::STORAGE_LIMIT).unwrap();
    context.boundary("ready", 0, m, None, None).unwrap();
    m.reset();
    let begin = Instant::now();
    let calibration = calibrate(m);
    let elapsed = begin.elapsed().as_nanos() as u64;
    if !calibration.valid {
        m.invalid = Some("clock-calibration");
    }
    context
        .boundary("calibrated", elapsed, m, None, Some(&calibration))
        .unwrap();
    assert!(calibration.valid, "invalid clock calibration");
    m.reset();
    let begin = Instant::now();
    let data = m.timed(Op::Fixture, || fixture(context.n));
    context.fixture_bytes = data.capacity() * std::mem::size_of::<Circle>();
    let mut d = Driver::create_with(
        context.n,
        context.trace,
        context.budget,
        &data,
        m,
        driver::Setup {
            id: query::base_id(context.case_ordinal) + 1,
            tiny: false,
        },
    );
    context.peaks = d.peaks;
    context
        .boundary(
            "prepared",
            begin.elapsed().as_nanos() as u64,
            m,
            Some(&d),
            None,
        )
        .unwrap();
    m.reset();
    let begin = Instant::now();
    d.play(m);
    context.peaks = d.peaks;
    context
        .boundary(
            "played",
            begin.elapsed().as_nanos() as u64,
            m,
            Some(&d),
            None,
        )
        .unwrap();
    m.reset();
    let begin = Instant::now();
    d.query(m, storage.page_buffer());
    context
        .boundary(
            "queried",
            begin.elapsed().as_nanos() as u64,
            m,
            Some(&d),
            None,
        )
        .unwrap();
    m.reset();
    let begin = Instant::now();
    d.inspect(m);
    context.peaks = d.peaks;
    context
        .boundary(
            "inspected",
            begin.elapsed().as_nanos() as u64,
            m,
            Some(&d),
            None,
        )
        .unwrap();
    m.reset();
    let begin = Instant::now();
    let mut task = m
        .timed(Op::InspectRequest, || {
            d.owner.request_inspection(d.source(), t(0))
        })
        .unwrap();
    let mut reached = false;
    for _ in 0..bound(d.n) {
        m.timed(Op::CancelPrepare, || {
            task.step(task.cursor(), WorkBudget::one())
        })
        .unwrap();
        d.peaks.observe(&d.owner);
        let c = task.work_counts();
        if c.inspection_objects >= d.n as u64 / 2 && c.allocation_attempts >= 3 {
            reached = true;
            break;
        }
    }
    assert!(reached, "cancel checkpoint bound");
    context.candidate = Some(task.work_counts());
    let held = d.owner.resources();
    let allocated = d.owner.allocations();
    m.timed(Op::Cancel, || d.owner.cancel(task.key())).unwrap();
    assert_eq!(d.owner.resources(), held);
    assert_eq!(d.owner.allocations(), allocated);
    assert!(d.owner.candidate().is_none());
    context.peaks = d.peaks;
    context
        .boundary(
            "cancelled",
            begin.elapsed().as_nanos() as u64,
            m,
            Some(&d),
            None,
        )
        .unwrap();
    m.reset();
    let begin = Instant::now();
    m.timed(Op::TaskDrop, || drop(task));
    context.candidate = None;
    let baseline = d.owner.resources();
    m.timed(Op::ControlBatch, || {
        for _ in 0..256 {
            let job = d.owner.request_inspection(d.source(), t(0)).unwrap();
            d.owner.cancel(job.key()).unwrap();
            drop(job);
        }
    });
    assert_eq!(d.owner.resources(), baseline);
    let cost = SyntheticCoordinator::costs_for_objects(d.n).unwrap();
    assert_eq!(
        baseline.runtime_bytes,
        cost.pool_bytes + cost.coordinator_bytes + cost.run_bytes
    );
    context
        .boundary(
            "task_dropped",
            begin.elapsed().as_nanos() as u64,
            m,
            Some(&d),
            None,
        )
        .unwrap();
    m.reset();
    let begin = Instant::now();
    m.timed(Op::OwnerDrop, || drop(d));
    m.timed(Op::FixtureDrop, || drop(data));
    context.fixture_bytes = 0;
    context
        .boundary("released", begin.elapsed().as_nanos() as u64, m, None, None)
        .unwrap();
    m.reset();
    let begin = Instant::now();
    let data = fixture(context.n);
    context.fixture_bytes = data.capacity() * std::mem::size_of::<Circle>();
    let mut setup = Meter::unmeasured();
    let d = query::establish(
        context.n,
        context.trace,
        context.budget,
        &data,
        driver::Setup {
            id: query::base_id(context.case_ordinal) + 2,
            tiny: false,
        },
        &mut setup,
    );
    storage.bind(&d);
    context
        .query_boundary(
            "query_group_ready",
            begin.elapsed().as_nanos() as u64,
            m,
            Some(&d),
            storage,
            query::Receipt::Setup(&setup),
        )
        .unwrap();
    for (kind, phase) in ["query_summary", "query_ledger", "query_page"]
        .iter()
        .enumerate()
    {
        let begin = Instant::now();
        storage.measure(kind, &d);
        context
            .query_boundary(
                phase,
                begin.elapsed().as_nanos() as u64,
                m,
                Some(&d),
                storage,
                query::Receipt::Batch(kind),
            )
            .unwrap();
    }
    let begin = Instant::now();
    drop(d);
    drop(data);
    context.fixture_bytes = 0;
    context
        .query_boundary(
            "query_group_released",
            begin.elapsed().as_nanos() as u64,
            m,
            None,
            storage,
            query::Receipt::Released,
        )
        .unwrap();
    let begin = Instant::now();
    let data = fixture(context.n);
    context.fixture_bytes = data.capacity() * std::mem::size_of::<Circle>();
    pc.establish(
        context.n,
        context.trace,
        context.budget,
        &data,
        context.case_ordinal,
        false,
    );
    context
        .pc_boundary(
            "pc_group_ready",
            begin.elapsed().as_nanos() as u64,
            m,
            pc,
            pc::Receipt::Ready,
        )
        .unwrap();
    for r in 0..pc::REPEATS {
        let begin = Instant::now();
        pc.prepare(r, true);
        context
            .pc_boundary(
                "pc_prepared",
                begin.elapsed().as_nanos() as u64,
                m,
                pc,
                pc::Receipt::Prepared(r),
            )
            .unwrap();
        let begin = Instant::now();
        pc.cancel(r, true);
        context
            .pc_boundary(
                "pc_cancelled_held",
                begin.elapsed().as_nanos() as u64,
                m,
                pc,
                pc::Receipt::Held(r),
            )
            .unwrap();
        let begin = Instant::now();
        pc.drop_tasks(r, true);
        context
            .pc_boundary(
                "pc_dropped",
                begin.elapsed().as_nanos() as u64,
                m,
                pc,
                pc::Receipt::Dropped(r),
            )
            .unwrap();
    }
    let begin = Instant::now();
    pc.references(true);
    pc.release();
    drop(data);
    context.fixture_bytes = 0;
    context
        .pc_boundary(
            "pc_group_released",
            begin.elapsed().as_nanos() as u64,
            m,
            pc,
            pc::Receipt::Released,
        )
        .unwrap();
    context
        .emit(
            "batch_summary",
            0,
            m,
            None,
            (
                None,
                Some((storage, query::Receipt::Summary)),
                Some((pc, pc::Receipt::Summary)),
            ),
        )
        .unwrap();
    context.acknowledge().unwrap();
    let output = io::stdout();
    let mut out = output.lock();
    writeln!(out,"{{\"schema_version\":{SCHEMA},\"plan_id\":\"{}\",\"case_ordinal\":{},\"trace\":\"{}\",\"budget\":{},\"objects\":{},\"phase\":\"end\",\"status\":\"Valid\",\"previous_emit_ns\":{},\"previous_gate_ns\":{}}}",context.plan,context.case_ordinal,context.trace.name(),context.budget.get(),context.n,context.previous_emit,context.previous_gate).unwrap();
    out.flush().unwrap();
}
fn parse(args: &[String]) -> Result<(Trace, WorkBudget, usize), &'static str> {
    if args.len() != 3 {
        return Err("usage: probe g0|g1|a1 1|256 case-ordinal(0..11)");
    }
    let trace = Trace::parse(&args[0]).ok_or("invalid trace")?;
    let budget = match args[1].as_str() {
        "1" => WorkBudget::one(),
        "256" => WorkBudget::full(),
        _ => return Err("invalid budget"),
    };
    let ordinal = args[2].parse::<usize>().map_err(|_| "invalid ordinal")?;
    let trace_index = match trace {
        Trace::G0 => 0,
        Trace::G1 => 1,
        Trace::A1 => 2,
    };
    if ordinal >= 12
        || ordinal / 4 != trace_index
        || (ordinal / 2) % 2 != usize::from(budget == WorkBudget::full())
    {
        return Err("case identity mismatch");
    }
    Ok((trace, budget, ordinal))
}
fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let (trace, budget, case_ordinal) = parse(&args).unwrap_or_else(|error| {
        eprintln!("{error}");
        std::process::exit(2)
    });
    let mut context = Context {
        plan: pc::PLAN,
        case_ordinal,
        trace,
        budget,
        n: 10000,
        previous_emit: 0,
        previous_gate: 0,
        fixture_bytes: 0,
        peaks: Peaks::default(),
        candidate: None,
        events_checked: 0,
    };
    let mut meter = Meter::new(context.n);
    assert!(meter.failure_capacity_bytes() + 256 * 1024 < 32 * 1024 * 1024);
    let mut storage = query::Storage::new();
    let mut pc = pc::Storage::new();
    if std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        run(&mut context, &mut meter, &mut storage, &mut pc)
    }))
    .is_err()
    {
        pc.cleanup();
        meter.invalid = Some(meter.invalid.unwrap_or("workload-or-protocol"));
        let output = io::stdout();
        let mut out = output.lock();
        // Bounded failure channel; no normal sample slot, observer or ACK is needed.
        let _ = write!(
            out,
            "{{\"schema_version\":{SCHEMA},\"plan_id\":\"{}\",\"case_ordinal\":{},\"phase\":\"failed\",\"status\":\"Invalid\",\"reason\":\"{}\",\"retained_query_samples\":",
            context.plan,
            context.case_ordinal,
            meter.invalid.unwrap()
        );
        let _ = storage.write_failure(&mut out);
        let _ = write!(out, ",\"retained_pc\":");
        let _ = pc.write_failure(&mut out);
        let _ = write!(out, ",\"retained_baseline_metrics\":");
        let _ = meter.write_failure(&mut out);
        let _ = writeln!(out, "}}");
        let _ = out.flush();
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Full Reference matrix through the real writer and real tasks, clocks disabled.
    #[test]
    fn closure_wire_canary_without_clocks() {
        for (index, trace) in [Trace::G0, Trace::G1, Trace::A1].into_iter().enumerate() {
            for (b, budget) in [WorkBudget::one(), WorkBudget::full()]
                .into_iter()
                .enumerate()
            {
                let ordinal = index * 4 + b * 2;
                let data = fixture(10000);
                let mut setup = Meter::unmeasured();
                let d = query::establish(
                    10000,
                    trace,
                    budget,
                    &data,
                    driver::Setup {
                        id: query::base_id(ordinal) + 2,
                        tiny: false,
                    },
                    &mut setup,
                );
                let mut q = query::Storage::new();
                q.bind(&d);
                let m = Meter::new(10000);
                let mut context = Context {
                    plan: pc::PLAN,
                    case_ordinal: ordinal,
                    trace,
                    budget,
                    n: 10000,
                    previous_emit: 0,
                    previous_gate: 0,
                    fixture_bytes: data.capacity() * std::mem::size_of::<Circle>(),
                    peaks: d.peaks,
                    candidate: None,
                    events_checked: 0,
                };
                let mut row = Vec::new();
                fn emit(row: &mut Vec<u8>) {
                    assert!(row.len() < 1024 * 1024);
                    println!(
                        "\nPC_CANARY\t{}",
                        std::str::from_utf8(row).unwrap().trim_end()
                    );
                    row.clear();
                }
                context
                    .write_row(
                        &mut row,
                        ("query_group_ready", 0),
                        &m,
                        Some(&d),
                        (None, Some((&q, query::Receipt::Setup(&setup))), None),
                    )
                    .unwrap();
                emit(&mut row);
                for (kind, phase) in ["query_summary", "query_ledger", "query_page"]
                    .into_iter()
                    .enumerate()
                {
                    q.collect(kind, &d, false);
                    context
                        .write_row(
                            &mut row,
                            (phase, 0),
                            &m,
                            Some(&d),
                            (None, Some((&q, query::Receipt::Batch(kind))), None),
                        )
                        .unwrap();
                    emit(&mut row);
                }
                drop(d);
                drop(data);
                context.fixture_bytes = 0;
                context
                    .write_row(
                        &mut row,
                        ("query_group_released", 0),
                        &m,
                        None,
                        (None, Some((&q, query::Receipt::Released)), None),
                    )
                    .unwrap();
                emit(&mut row);
                let data = fixture(10000);
                context.fixture_bytes = data.capacity() * std::mem::size_of::<Circle>();
                let mut pc = pc::Storage::new();
                pc.establish(10000, trace, budget, &data, ordinal, false);
                context
                    .write_row(
                        &mut row,
                        ("pc_group_ready", 0),
                        &m,
                        None,
                        (None, None, Some((&pc, pc::Receipt::Ready))),
                    )
                    .unwrap();
                emit(&mut row);
                for r in 0..pc::REPEATS {
                    pc.prepare(r, false);
                    context
                        .write_row(
                            &mut row,
                            ("pc_prepared", 0),
                            &m,
                            None,
                            (None, None, Some((&pc, pc::Receipt::Prepared(r)))),
                        )
                        .unwrap();
                    emit(&mut row);
                    pc.cancel(r, false);
                    context
                        .write_row(
                            &mut row,
                            ("pc_cancelled_held", 0),
                            &m,
                            None,
                            (None, None, Some((&pc, pc::Receipt::Held(r)))),
                        )
                        .unwrap();
                    emit(&mut row);
                    pc.drop_tasks(r, false);
                    context
                        .write_row(
                            &mut row,
                            ("pc_dropped", 0),
                            &m,
                            None,
                            (None, None, Some((&pc, pc::Receipt::Dropped(r)))),
                        )
                        .unwrap();
                    emit(&mut row);
                }
                pc.references(false);
                pc.release();
                drop(data);
                context.fixture_bytes = 0;
                context
                    .write_row(
                        &mut row,
                        ("pc_group_released", 0),
                        &m,
                        None,
                        (None, None, Some((&pc, pc::Receipt::Released))),
                    )
                    .unwrap();
                emit(&mut row);
                context
                    .write_row(
                        &mut row,
                        ("batch_summary", 0),
                        &m,
                        None,
                        (
                            None,
                            Some((&q, query::Receipt::Summary)),
                            Some((&pc, pc::Receipt::Summary)),
                        ),
                    )
                    .unwrap();
                emit(&mut row);
            }
        }
    }

    #[test]
    fn fixed_track_uses_independent_microsecond_anchors() {
        let data = fixture(10_000);
        assert_eq!(driver::START, -200_001);
        for (i, circle) in data.iter().enumerate() {
            assert_eq!(circle.target(), MapTime::from_micros(i as i64 * 1_000_000));
            assert_eq!(circle.id(), ObjectId::new(i as u32));
            assert_eq!(circle.center(), Point::new(0, 0).unwrap());
        }
        assert_eq!(data[9999].retires_at(), MapTime::from_micros(9_999_300_000));
        assert_eq!(driver::end(10_000), 9_999_300_000);
    }
    #[test]
    fn trace_budget_and_count_are_explicit_not_silently_clamped() {
        for args in [vec!["g0", "1", "0"], vec!["a1", "256", "11"]] {
            let args: Vec<_> = args.into_iter().map(str::to_owned).collect();
            assert!(parse(&args).is_ok());
        }
        for args in [
            vec![],
            vec!["g2", "1", "4"],
            vec!["g1", "2", "4"],
            vec!["g0", "256", "0"],
            vec!["g0", "1", "10001"],
        ] {
            let args: Vec<_> = args.into_iter().map(str::to_owned).collect();
            assert!(parse(&args).is_err());
        }
    }
    #[test]
    fn independent_driver_exercises_all_three_traces_and_rebuild_without_devices() {
        for trace in [Trace::G0, Trace::G1, Trace::A1] {
            for budget in [WorkBudget::one(), WorkBudget::full()] {
                let data = fixture(4);
                let mut m = Meter::new(4);
                let mut d = Driver::create(4, trace, budget, &data, &mut m);
                assert_eq!(d.source().transport.get(), 1);
                assert_eq!(d.source().input.get(), 1);
                assert_eq!(d.source().revision.get(), 730);
                m.reset();
                d.play(&mut m);
                assert_eq!(
                    d.source().input.get(),
                    if trace == Trace::A1 { 3 } else { 1 }
                );
                m.reset();
                d.query(&mut m, &mut [None; 64]);
                m.reset();
                d.inspect(&mut m);
                assert_eq!(d.source().transport.get(), 2);
                assert_eq!(
                    d.source().input.get(),
                    if trace == Trace::A1 { 4 } else { 2 }
                );
                assert_eq!(
                    d.owner
                        .current()
                        .unwrap()
                        .observation()
                        .committed_counts
                        .inspection_skipped,
                    2
                );
            }
        }
    }

    /// Actual query rows through the production writer, with no clock, ACK,
    /// calibration or native observer. Python --check consumes these six cases.
    #[test]
    fn query_wire_canary_without_clocks() {
        for (index, trace) in [Trace::G0, Trace::G1, Trace::A1].into_iter().enumerate() {
            for (b, budget) in [WorkBudget::one(), WorkBudget::full()]
                .into_iter()
                .enumerate()
            {
                let ordinal = index * 4 + b * 2;
                let data = fixture(10000);
                let mut setup = Meter::unmeasured();
                let d = query::establish(
                    10000,
                    trace,
                    budget,
                    &data,
                    driver::Setup {
                        id: query::base_id(ordinal) + 2,
                        tiny: false,
                    },
                    &mut setup,
                );
                let mut storage = query::Storage::new();
                storage.bind(&d);
                let m = Meter::new(10000);
                let mut context = Context {
                    plan: query::PLAN,
                    case_ordinal: ordinal,
                    trace,
                    budget,
                    n: 10000,
                    previous_emit: 0,
                    previous_gate: 0,
                    fixture_bytes: data.capacity() * std::mem::size_of::<Circle>(),
                    peaks: d.peaks,
                    candidate: None,
                    events_checked: 0,
                };
                let mut rows = Vec::new();
                context
                    .write_row(
                        &mut rows,
                        ("query_group_ready", 0),
                        &m,
                        Some(&d),
                        (None, Some((&storage, query::Receipt::Setup(&setup))), None),
                    )
                    .unwrap();
                for (kind, phase) in ["query_summary", "query_ledger", "query_page"]
                    .into_iter()
                    .enumerate()
                {
                    storage.collect(kind, &d, false);
                    context
                        .write_row(
                            &mut rows,
                            (phase, 0),
                            &m,
                            Some(&d),
                            (None, Some((&storage, query::Receipt::Batch(kind))), None),
                        )
                        .unwrap();
                }
                drop(d);
                drop(data);
                context.fixture_bytes = 0;
                for (phase, receipt) in [
                    ("query_group_released", query::Receipt::Released),
                    ("batch_summary", query::Receipt::Summary),
                ] {
                    context
                        .write_row(
                            &mut rows,
                            (phase, 0),
                            &m,
                            None,
                            (None, Some((&storage, receipt)), None),
                        )
                        .unwrap();
                }
                let text = String::from_utf8(rows).unwrap();
                assert_eq!(text.lines().count(), 6);
                assert!(text.len() < 1024 * 1024);
                for row in text.lines() {
                    println!("\nQUERY_CANARY\t{row}");
                }
            }
        }
    }
}
