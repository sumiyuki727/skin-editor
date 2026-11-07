//! Independent, versioned, headless SyntheticTest cost probe.
#![forbid(unsafe_code)]
mod driver;
mod meter;
use driver::{Calibration, Driver, Peaks, Trace, bound, calibrate, fixture, t};
use meter::{Meter, Op};
use skin_editor_gameplay::*;
use std::io::{self, BufRead, Read, Write};
use std::time::Instant;

const SCHEMA: u32 = 2;
struct Context {
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
        self.emit(phase, elapsed, m, state, calibration)?;
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
        calibration: Option<&Calibration>,
    ) -> io::Result<()> {
        let begin = Instant::now();
        let output = io::stdout();
        if let Some(s) = state {
            self.events_checked = s.events;
        }
        let mut out = io::BufWriter::new(output.lock());
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
            state.is_some(),
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
        write!(out, ",\"metrics\":")?;
        m.json(&mut out)?;
        writeln!(out, "}}")?;
        out.flush()?;
        self.previous_emit = begin.elapsed().as_nanos() as u64;
        Ok(())
    }
}
fn run(context: &mut Context, m: &mut Meter) {
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
    let mut d = Driver::create(context.n, context.trace, context.budget, &data, m);
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
    d.query(m);
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
    let output = io::stdout();
    let mut out = output.lock();
    writeln!(out,"{{\"schema_version\":{SCHEMA},\"phase\":\"end\",\"status\":\"Valid\",\"previous_emit_ns\":{},\"previous_gate_ns\":{}}}",context.previous_emit,context.previous_gate).unwrap();
    out.flush().unwrap();
}
fn parse(args: &[String]) -> Result<(Trace, WorkBudget, usize), &'static str> {
    if args.len() != 3 {
        return Err("usage: probe g0|g1|a1 1|256 objects(1..10000)");
    }
    let trace = Trace::parse(&args[0]).ok_or("invalid trace")?;
    let budget = match args[1].as_str() {
        "1" => WorkBudget::one(),
        "256" => WorkBudget::full(),
        _ => return Err("invalid budget"),
    };
    let n = args[2].parse::<usize>().map_err(|_| "invalid count")?;
    if n == 0 || n > 10_000 {
        return Err("invalid count");
    }
    Ok((trace, budget, n))
}
fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let (trace, budget, n) = parse(&args).unwrap_or_else(|error| {
        eprintln!("{error}");
        std::process::exit(2)
    });
    let mut context = Context {
        trace,
        budget,
        n,
        previous_emit: 0,
        previous_gate: 0,
        fixture_bytes: 0,
        peaks: Peaks::default(),
        candidate: None,
        events_checked: 0,
    };
    let mut meter = Meter::new(n);
    if std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        run(&mut context, &mut meter)
    }))
    .is_err()
    {
        meter.invalid = Some(meter.invalid.unwrap_or("workload-or-protocol"));
        let _ = context.emit("failed", 0, &meter, None, None);
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
        for args in [vec!["g0", "1", "10000"], vec!["a1", "256", "4"]] {
            let args: Vec<_> = args.into_iter().map(str::to_owned).collect();
            assert!(parse(&args).is_ok());
        }
        for args in [
            vec![],
            vec!["g2", "1", "4"],
            vec!["g1", "2", "4"],
            vec!["g0", "1", "0"],
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
                m.reset();
                d.play(&mut m);
                m.reset();
                d.query(&mut m);
                m.reset();
                d.inspect(&mut m);
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
}
