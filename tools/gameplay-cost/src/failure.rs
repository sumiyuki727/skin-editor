//! Fixed bounded failure channel. Cleanup never depends on stdout or ACK.
use crate::{Context, meter::Meter, pc, query};
use std::io::{self, Write};

pub const LINE_LIMIT: usize = 32 * 1024 * 1024;

struct Limited<W> {
    inner: W,
    bytes: usize,
    limit: usize,
}
impl<W: Write> Limited<W> {
    fn new(inner: W, limit: usize) -> Self {
        Self {
            inner,
            bytes: 0,
            limit,
        }
    }
}
impl<W: Write> Write for Limited<W> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.len() > self.limit.saturating_sub(self.bytes) {
            return Err(io::ErrorKind::Other.into());
        }
        let n = self.inner.write(bytes)?;
        self.bytes = self.bytes.checked_add(n).ok_or(io::ErrorKind::Other)?;
        Ok(n)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

pub fn report(
    out: &mut impl Write,
    context: &Context,
    meter: Option<&Meter>,
    query: Option<&query::Storage>,
    pc: &mut Option<pc::Storage>,
) -> io::Result<()> {
    if let Some(storage) = pc.as_mut() {
        storage.cleanup();
    }
    write(out, context, meter, query, pc.as_ref())
}

fn write(
    out: &mut impl Write,
    context: &Context,
    meter: Option<&Meter>,
    query: Option<&query::Storage>,
    pc: Option<&pc::Storage>,
) -> io::Result<()> {
    let mut whole = Limited::new(out, LINE_LIMIT);
    // Two bytes are reserved for the closing brace/newline after baseline data.
    {
        let mut supplement = Limited::new(&mut whole, pc::FAILURE_LIMIT - 2);
        let attempt = pc.map_or(pc::Attempt::setup(), |p| p.attempt);
        write!(
            supplement,
            "{{\"schema_version\":3,\"plan_id\":\"{}\",\"case_ordinal\":{},\"phase\":\"failed\",\"status\":\"Invalid\",\"reason\":\"{}\",\"attempt_r\":",
            context.plan,
            context.case_ordinal,
            meter
                .and_then(|m| m.invalid)
                .unwrap_or("workload-or-protocol")
        )?;
        match attempt.round {
            Some(r) => write!(supplement, "{r}")?,
            None => write!(supplement, "null")?,
        }
        write!(
            supplement,
            ",\"attempt_phase\":\"{}\",\"retained_query_samples\":",
            attempt.phase
        )?;
        match query {
            Some(q) => q.write_failure(&mut supplement)?,
            None => write!(supplement, "null")?,
        }
        write!(supplement, ",\"retained_pc\":")?;
        match pc {
            Some(p) => p.write_failure(&mut supplement)?,
            None => write!(supplement, "null")?,
        }
        write!(supplement, ",\"retained_baseline_metrics\":")?;
    }
    match meter {
        Some(m) => m.write_failure(&mut whole)?,
        None => write!(whole, "null")?,
    }
    writeln!(whole, "}}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::driver::{Trace, fixture};
    use skin_editor_gameplay::WorkBudget;

    fn context() -> Context {
        Context {
            plan: pc::PLAN,
            case_ordinal: 11,
            trace: Trace::A1,
            budget: WorkBudget::one(),
            n: 10000,
            previous_emit: 0,
            previous_gate: 0,
            fixture_bytes: 0,
            peaks: Default::default(),
            candidate: None,
            events_checked: 0,
        }
    }

    #[test]
    fn failure_before_container_construction_has_null_observers_and_no_ack() {
        let mut out = Vec::new();
        report(&mut out, &context(), None, None, &mut None).unwrap();
        let text = String::from_utf8(out).unwrap();
        for field in [
            "retained_query_samples",
            "retained_pc",
            "retained_baseline_metrics",
            "attempt_r",
        ] {
            assert!(text.contains(&format!("\"{field}\":null")));
        }
        assert!(text.ends_with("}\n"));
        assert!(text.contains("\"status\":\"Invalid\""));
    }

    #[test]
    fn broken_output_does_not_keep_any_candidate_or_owner_alive() {
        struct Broken;
        impl Write for Broken {
            fn write(&mut self, _: &[u8]) -> io::Result<usize> {
                Err(io::ErrorKind::BrokenPipe.into())
            }
            fn flush(&mut self) -> io::Result<()> {
                Err(io::ErrorKind::BrokenPipe.into())
            }
        }
        let mut pc = Some(pc::Storage::new());
        pc.as_mut()
            .unwrap()
            .establish(4, Trace::G0, WorkBudget::one(), &fixture(4), 0, true);
        pc.as_mut().unwrap().prepare(0, false);
        assert!(report(&mut Broken, &context(), None, None, &mut pc).is_err());
        assert!(!pc.as_ref().unwrap().owners_alive());
        let mut out = Vec::new();
        report(&mut out, &context(), None, None, &mut pc).unwrap();
        let text = String::from_utf8(out).unwrap();
        assert!(text.contains("\"attempt_r\":0,\"attempt_phase\":\"prepare\""));
        assert!(text.contains("\"owners_dropped\":128"));
    }

    #[test]
    fn production_failure_writer_fits_complete_maximum_width_values() {
        let mut q = query::Storage::new();
        q.maximum_failure_fixture();
        let mut p = pc::Storage::new();
        p.maximum_failure_fixture();
        p.attempt = pc::Attempt {
            round: Some(15),
            phase: "reference",
        };
        let mut count = Limited::new(io::sink(), pc::FAILURE_LIMIT);
        write(&mut count, &context(), None, Some(&q), Some(&p)).unwrap();
        assert!(count.bytes > 256 * 1024);
        assert!(count.bytes < pc::FAILURE_LIMIT);
        let null_baseline = count.bytes;
        let mut m = Meter::new(10000);
        m.maximum_failure_fixture();
        assert!(m.failure_capacity_bytes() + pc::FAILURE_LIMIT < LINE_LIMIT);
        let mut total = Limited::new(io::sink(), LINE_LIMIT);
        write(&mut total, &context(), Some(&m), Some(&q), Some(&p)).unwrap();
        assert!(total.bytes > null_baseline);
        assert!(total.bytes < LINE_LIMIT);
        let mut baseline = Limited::new(io::sink(), LINE_LIMIT);
        m.write_failure(&mut baseline).unwrap();
        let supplemental = total.bytes.checked_sub(baseline.bytes).unwrap();
        assert!(supplemental > 256 * 1024 && supplemental < pc::FAILURE_LIMIT);
        println!(
            "PC128 failure encoding: supplemental={supplemental}, complete={}",
            total.bytes
        );
    }

    #[test]
    fn bounded_sink_rejects_before_forwarding_excess_bytes() {
        let mut out = Vec::new();
        let mut writer = Limited::new(&mut out, 3);
        writer.write_all(b"abc").unwrap();
        assert!(writer.write_all(b"d").is_err());
        assert_eq!(writer.bytes, 3);
        assert_eq!(out, b"abc");
    }
}
