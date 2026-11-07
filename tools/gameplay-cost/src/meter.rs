use std::io::{self, Write};
use std::time::Instant;

#[derive(Clone, Copy)]
#[repr(usize)]
pub enum Op {
    Empty,
    Fixture,
    Request,
    Prepare,
    Activate,
    Auto,
    Submit,
    Seal,
    Advance,
    Ack,
    Summary,
    Page,
    Ledger,
    QueryBatch,
    Freeze,
    InspectRequest,
    InspectStep,
    InspectActivate,
    CancelPrepare,
    Cancel,
    TaskDrop,
    OwnerDrop,
    FixtureDrop,
    ControlBatch,
}
const NAMES: [&str; 24] = [
    "empty",
    "fixture",
    "create_request",
    "prepare_step",
    "activate",
    "automatic_prepare",
    "submit",
    "seal",
    "advance",
    "acknowledge",
    "summary",
    "page",
    "ledger",
    "query_batch",
    "freeze",
    "inspection_request",
    "inspection_step",
    "inspection_activate",
    "cancel_candidate_step",
    "cancel",
    "task_drop",
    "owner_drop",
    "fixture_drop",
    "request_cancel_drop_batch",
];
struct Bucket {
    raw: Vec<u64>,
    limit: usize,
    total: u64,
}
pub struct Meter {
    buckets: Vec<Bucket>,
    pub invalid: Option<&'static str>,
    pub bytes: usize,
}
impl Meter {
    pub fn new(n: usize) -> Self {
        let levels = usize::BITS as usize - (n.max(2) - 1).leading_zeros() as usize;
        let linear = n * 32 + 256;
        let limits = [
            20_000,
            4,
            4,
            n * (2 * levels + 16) + 128,
            4,
            n * 4 + 1024,
            n.div_ceil(32) + 8,
            n.div_ceil(32) + 32,
            linear,
            2 * n + 16,
            1024,
            2 * n.div_ceil(64) + 16,
            2 * n + 16,
            4,
            linear,
            4,
            2 * n + 128,
            4,
            2 * n + 128,
            4,
            8,
            4,
            4,
            4,
        ];
        Self::with_limits(&limits)
    }
    fn with_limits(limits: &[usize; 24]) -> Self {
        let mut bytes = 0;
        let buckets = limits
            .iter()
            .map(|&limit| {
                // Touch the diagnostic sample pages before the ready baseline.
                let mut raw = vec![0; limit];
                raw.fill(1);
                std::hint::black_box(&raw);
                bytes += raw.capacity() * 8;
                raw.clear();
                Bucket {
                    raw,
                    limit,
                    total: 0,
                }
            })
            .collect();
        Self {
            buckets,
            invalid: None,
            bytes,
        }
    }
    pub fn reset(&mut self) {
        if self.invalid.is_some() {
            return;
        }
        for b in &mut self.buckets {
            b.raw.clear();
            b.total = 0;
        }
    }
    fn room(&mut self, op: Op) -> Result<(), &'static str> {
        if self.invalid.is_some() {
            return Err("invalid-meter");
        }
        if self.buckets[op as usize].raw.len() == self.buckets[op as usize].limit {
            self.invalid = Some("sample-capacity");
            return Err("sample-capacity");
        }
        Ok(())
    }
    fn record(&mut self, op: Op, ns: u64) -> Result<(), &'static str> {
        self.room(op)?;
        let bucket = &mut self.buckets[op as usize];
        let Some(total) = bucket.total.checked_add(ns) else {
            self.invalid = Some("sample-total-overflow");
            return Err("sample-total-overflow");
        };
        bucket.total = total;
        bucket.raw.push(ns);
        Ok(())
    }
    pub fn timed<R>(&mut self, op: Op, call: impl FnOnce() -> R) -> R {
        self.room(op)
            .expect("sample capacity exhausted before invoking operation");
        let begin = Instant::now();
        // The barrier is part of diagnostic cost, never subtracted from results.
        let result = std::hint::black_box(call());
        let elapsed = u64::try_from(
            Instant::now()
                .checked_duration_since(begin)
                .expect("monotonic call clock")
                .as_nanos(),
        )
        .expect("finite call time");
        self.record(op, elapsed).expect("invalid timing receipt");
        result
    }
    pub fn json(&self, out: &mut impl Write) -> io::Result<()> {
        write!(out, "[")?;
        for (index, b) in self.buckets.iter().enumerate() {
            if index > 0 {
                write!(out, ",")?;
            }
            let mut sorted = b.raw.clone();
            sorted.sort_unstable();
            let value = |p| {
                if self.invalid.is_some() {
                    "null".to_owned()
                } else {
                    percentile(&sorted, p).map_or("null".to_owned(), |v| v.to_string())
                }
            };
            write!(
                out,
                "{{\"name\":\"{}\",\"capacity\":{},\"calls\":{},\"sum_ns\":{},\"p50_ns\":{},\"p95_ns\":{},\"p99_ns\":{},\"max_ns\":{},\"raw_ns\":[",
                NAMES[index],
                b.limit,
                b.raw.len(),
                b.total,
                value(50),
                value(95),
                value(99),
                value(100)
            )?;
            for (i, ns) in b.raw.iter().enumerate() {
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
pub fn percentile(sorted: &[u64], percent: usize) -> Option<u64> {
    if sorted.is_empty() {
        return None;
    }
    Some(sorted[(sorted.len() * percent).div_ceil(100) - 1])
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nearest_rank_has_no_zero_sample_statistics() {
        assert_eq!(percentile(&[], 95), None);
        for (p, v) in [(50, 2), (95, 4), (99, 4), (100, 4)] {
            assert_eq!(percentile(&[1, 2, 3, 4], p), Some(v));
        }
    }
    #[test]
    fn full_samples_stop_before_an_unrecorded_operation_and_keep_the_receipt() {
        let mut m = Meter::with_limits(&[1; 24]);
        m.record(Op::Advance, 7).unwrap();
        let mut called = false;
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(
                || m.timed(Op::Advance, || called = true)
            ))
            .is_err()
        );
        assert!(!called);
        assert_eq!(m.invalid, Some("sample-capacity"));
        assert_eq!(m.buckets[Op::Advance as usize].raw, [7]);
        m.reset();
        assert!(m.record(Op::Advance, 1).is_err());
        assert_eq!(m.buckets[Op::Advance as usize].raw, [7]);
    }
    #[test]
    fn total_overflow_invalidates_without_wrapping_or_replacing_old_samples() {
        let mut m = Meter::with_limits(&[2; 24]);
        m.record(Op::Page, u64::MAX).unwrap();
        assert!(m.record(Op::Page, 1).is_err());
        assert_eq!(m.buckets[Op::Page as usize].raw, [u64::MAX]);
        let mut out = Vec::new();
        m.json(&mut out).unwrap();
        assert!(String::from_utf8(out).unwrap().contains("\"p95_ns\":null"));
    }
}
