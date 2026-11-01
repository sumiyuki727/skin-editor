//! Bounded opt-in causal observations. The callback only copies supplied stamps.
use std::sync::{
    OnceLock,
    atomic::{AtomicU64, Ordering},
};
use std::time::Instant;

struct Row<const N: usize> {
    sequence: AtomicU64,
    values: [AtomicU64; N],
}
impl<const N: usize> Default for Row<N> {
    fn default() -> Self {
        Self {
            sequence: AtomicU64::new(0),
            values: std::array::from_fn(|_| AtomicU64::new(0)),
        }
    }
}
impl<const N: usize> Row<N> {
    // One writer per row. SeqCst keeps the bounded reader from accepting torn data.
    fn write(&self, values: [u64; N]) {
        self.sequence.fetch_add(1, Ordering::SeqCst);
        for (slot, value) in self.values.iter().zip(values) {
            slot.store(value, Ordering::SeqCst);
        }
        self.sequence.fetch_add(1, Ordering::SeqCst);
    }
    fn read(&self) -> Option<[u64; N]> {
        for _ in 0..3 {
            let before = self.sequence.load(Ordering::SeqCst);
            if before == 0 || !before.is_multiple_of(2) {
                continue;
            }
            let row = std::array::from_fn(|i| self.values[i].load(Ordering::SeqCst));
            if self.sequence.load(Ordering::SeqCst) == before {
                return Some(row);
            }
        }
        None
    }
}
#[derive(Default)]
pub(crate) struct SubmissionCapture {
    origin: OnceLock<Instant>,
    producer: [Row<9>; 2],
    callback: [Row<7>; 2],
    progress: Row<4>,
    stage: AtomicU64,
    stages: [[AtomicU64; 5]; 8],
    underruns: [Row<44>; 32],
    supply: [Row<10>; 4],
    supply_head: AtomicU64,
    slow_supply: [Row<10>; 32],
    slow_supply_count: AtomicU64,
    schedule_events: [Row<86>; 32],
    schedule_event_count: AtomicU64,
}
/// Opt-in render-completion observation. Unknown must not advance a negative
/// bound: a concurrent row writer may have replaced an already completed Voice.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VoiceSubmission {
    Unknown,
    Pending,
    Complete([u64; 7]),
    Superseded,
    Invalidated,
}
/// Latest exact Voice per lane; overwrite is explicit, never attributed by >=.
#[derive(Clone, Copy, Debug)]
pub struct SubmissionTrace {
    pub origin: Instant,
    /// Voice, epoch, first audible serial, pump start / before / after commit ns,
    /// queued-before, batch frames, first audible offset in batch.
    pub producer: [u64; 9],
    /// Voice, epoch, first audible serial, supplied stream callback ns,
    /// first audible offset in callback, callback frames, callback ordinal.
    pub callback: [u64; 7],
}
impl SubmissionCapture {
    pub fn begin(&self) -> Result<Instant, String> {
        let now = Instant::now();
        self.origin
            .set(now)
            .map_err(|_| "submission capture already armed")?;
        Ok(now)
    }
    pub fn origin(&self) -> Option<Instant> {
        self.origin.get().copied()
    }
    pub fn produced(&self, lane: usize, row: [u64; 9]) {
        self.producer[lane].write(row);
    }
    pub fn submitted(&self, lane: usize, row: [u64; 7]) {
        self.callback[lane].write(row);
    }
    pub fn completed_callback(&self, row: [u64; 4]) {
        self.progress.write(row);
    }
    pub fn callback_progress(&self) -> Option<[u64; 4]> {
        self.progress.read()
    }
    pub fn set_stage(&self, stage: u64) -> Result<(), String> {
        if self.origin().is_none() || !(1..8).contains(&stage) {
            return Err("arm capture and use diagnostic stage 1..7".into());
        }
        self.stage.store(stage, Ordering::Release);
        Ok(())
    }
    pub fn stage(&self) -> u64 {
        self.stage.load(Ordering::Acquire)
    }
    pub fn completed_context(&self, history: [[u64; 11]; 4], empty_index: usize) {
        let current = history[3];
        for (slot, value) in self.stages[current[9] as usize].iter().zip([
            1,
            current[2],
            u64::from(current[5] > 0),
            current[5],
            current[8],
        ]) {
            slot.fetch_add(value, Ordering::Relaxed);
        }
        if current[5] > 0
            && let Some(row) = self.underruns.get(empty_index)
        {
            row.write(std::array::from_fn(|i| history[i / 11][i % 11]));
        }
        let previous = history[2];
        let gap = previous[0] != 0 && current[1].saturating_sub(previous[1]) > 20_000_000;
        if gap || current[5] > 0 {
            let index = self.schedule_event_count.fetch_add(1, Ordering::Relaxed) as usize;
            if let Some(slot) = self.schedule_events.get(index) {
                let head = self.supply_head.load(Ordering::Acquire);
                let mut event = [0; 86];
                event[0] = head;
                event[1] = u64::from(gap) | (u64::from(current[5] > 0) << 1);
                for (slot, value) in event[2..46].iter_mut().zip(history.into_iter().flatten()) {
                    *slot = value;
                }
                for (i, row) in self.recent_supply(head).into_iter().enumerate() {
                    if let Some(row) = row {
                        event[46 + i * 10..56 + i * 10].copy_from_slice(&row);
                    }
                }
                slot.write(event);
            }
        }
    }
    pub fn completed_supply(&self, row: [u64; 10]) {
        self.supply[row[0] as usize % 4].write(row);
        self.supply_head.store(row[0], Ordering::Release);
        let slow = row[5] > 0 && row[4].saturating_sub(row[5]) > 20_000_000;
        if slow {
            let index = self.slow_supply_count.fetch_add(1, Ordering::Relaxed) as usize;
            if let Some(slot) = self.slow_supply.get(index) {
                slot.write(row);
            }
        }
    }
    fn recent_supply(&self, head: u64) -> [Option<[u64; 10]>; 4] {
        std::array::from_fn(|i| {
            let pass = head.checked_sub(3 - i as u64).filter(|p| *p > 0)?;
            self.supply[pass as usize % 4]
                .read()
                .filter(|r| r[0] == pass)
        })
    }
    pub fn scheduling(&self) -> crate::SupplyTimingSnapshot {
        let head = self.supply_head.load(Ordering::Acquire);
        let slow_count = self.slow_supply_count.load(Ordering::Acquire);
        let event_count = self.schedule_event_count.load(Ordering::Acquire);
        crate::SupplyTimingSnapshot {
            completed_pass: head,
            recent: self.recent_supply(head),
            slow_count,
            slow: self.slow_supply[..slow_count.min(32) as usize]
                .iter()
                .map(Row::read)
                .collect(),
            event_count,
            events: self.schedule_events[..event_count.min(32) as usize]
                .iter()
                .map(Row::read)
                .collect(),
        }
    }
    pub fn stage_counts(&self) -> [[u64; 5]; 8] {
        std::array::from_fn(|s| std::array::from_fn(|i| self.stages[s][i].load(Ordering::Acquire)))
    }
    pub fn underrun_contexts(&self, count: usize) -> Vec<Option<[[u64; 11]; 4]>> {
        self.underruns[..count.min(32)]
            .iter()
            .map(|r| {
                r.read()
                    .map(|v| std::array::from_fn(|i| std::array::from_fn(|j| v[i * 11 + j])))
            })
            .collect()
    }
    pub fn observe(&self, lane: usize, voice: u64, epoch: u64) -> VoiceSubmission {
        let Some(row) = self.callback[lane].read() else {
            return VoiceSubmission::Unknown;
        };
        if row[0] < voice {
            VoiceSubmission::Pending
        } else if row[1] != epoch {
            VoiceSubmission::Invalidated
        } else if row[0] == voice {
            VoiceSubmission::Complete(row)
        } else {
            VoiceSubmission::Superseded
        }
    }
    pub fn read(&self, lane: usize, voice: u64) -> Option<SubmissionTrace> {
        let producer = self.producer.get(lane)?.read()?;
        let callback = self.callback.get(lane)?.read()?;
        (producer[0] == voice && callback[..3] == producer[..3]).then_some(SubmissionTrace {
            origin: self.origin()?,
            producer,
            callback,
        })
    }
}
/// Control-thread sample of CPAL's stream clock, bracketed by host Instants.
/// No assumption that different streams or the two clock origins agree.
#[derive(Clone, Copy, Debug)]
pub struct StreamClockSample {
    pub before: Instant,
    pub after: Instant,
    pub stream_nanos: u64,
}
impl StreamClockSample {
    pub fn callback_bounds_ms(self, nanos: u64, request: Instant) -> [f64; 2] {
        let delta = (i128::from(nanos) - i128::from(self.stream_nanos)) as f64 / 1_000_000.;
        let relative = |at: Instant| {
            if at >= request {
                at.duration_since(request).as_secs_f64() * 1000.
            } else {
                -request.duration_since(at).as_secs_f64() * 1000.
            }
        };
        // Current Windows WASAPI stamps and now() use a 100 ns grid. Keep
        // rounding uncertainty as well as the measured control-call bracket.
        let rounding_ms = if cfg!(windows) { 0.0001 } else { 0. };
        [
            relative(self.before) + delta - rounding_ms,
            relative(self.after) + delta + rounding_ms,
        ]
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    #[test]
    fn scheduling_joins_exact_completed_pass_and_rejects_torn_or_overwritten_rows() {
        let capture = SubmissionCapture::default();
        capture.begin().unwrap();
        let mut supply = [1, 0, 0, 1, 2, 0, 1024, 1024, 0, 0];
        capture.completed_supply(supply);
        supply = [2, 3, 50_000_000, 50_010_000, 50_020_000, 2, 832, 832, 0, 2];
        capture.completed_supply(supply);
        let mut history = [[0; 11]; 4];
        history[2] = [2, 10_000_000, 480, 800, 320, 0, 1, 1, 0, 4, 4];
        history[3] = [3, 112_900_000, 1056, 832, 0, 224, 2, 2, 0, 4, 4];
        capture.completed_context(history, 0);
        let snapshot = capture.scheduling();
        assert_eq!(snapshot.slow_count, 1);
        assert_eq!(snapshot.slow[0], Some(supply));
        let event = snapshot.events[0].unwrap();
        assert_eq!(&event[..2], &[2, 3]);
        assert_eq!(&event[76..86], &supply);
        capture.supply[2].sequence.fetch_add(1, Ordering::SeqCst);
        capture.completed_context(history, 1);
        assert_eq!(capture.scheduling().events[1].unwrap()[76..86], [0; 10]);
        assert!(capture.scheduling().recent[3].is_none());
        capture.supply[2].sequence.fetch_add(1, Ordering::SeqCst); // release the controlled writer
        for i in 3..8 {
            capture.completed_supply([i, 0, 0, i, i, 0, 0, 0, 0, 0]);
        }
        assert_eq!(capture.scheduling().recent[0].unwrap()[0], 4);
    }
    #[test]
    fn scheduling_overflow_keeps_counts_and_first_event_windows() {
        let capture = SubmissionCapture::default();
        capture.begin().unwrap();
        for i in 1..41 {
            capture.completed_supply([i, 0, 0, i * 30_000_000, i * 30_000_000 + 1, 1, 0, 0, 0, 0]);
            let mut history = [[0; 11]; 4];
            history[3] = [i, 0, 480, 0, 0, 480, i, i, 0, 4, 4];
            capture.completed_context(history, (i - 1) as usize);
        }
        let s = capture.scheduling();
        assert_eq!((s.slow_count, s.event_count), (40, 40));
        assert_eq!((s.slow.len(), s.events.len()), (32, 32));
        assert_eq!(s.slow[0].unwrap()[0], 1);
        assert_eq!(s.events[31].unwrap()[0], 32);
    }
    #[test]
    fn exact_completion_does_not_accept_newer_or_invalidated_voice_or_torn_row() {
        let capture = SubmissionCapture::default();
        assert_eq!(capture.observe(0, 4, 2), VoiceSubmission::Unknown);
        capture.submitted(0, [3, 2, 90, 100, 17, 480, 3]);
        assert_eq!(capture.observe(0, 4, 2), VoiceSubmission::Pending);
        capture.submitted(0, [4, 2, 91, 110, 18, 480, 4]);
        assert!(matches!(
            capture.observe(0, 4, 2),
            VoiceSubmission::Complete(_)
        ));
        assert_eq!(capture.observe(0, 3, 2), VoiceSubmission::Superseded);
        assert_eq!(capture.observe(0, 4, 3), VoiceSubmission::Invalidated);
        assert_eq!(capture.observe(0, 5, 3), VoiceSubmission::Pending);
        capture.callback[0].sequence.fetch_add(1, Ordering::SeqCst);
        assert_eq!(capture.observe(0, 4, 2), VoiceSubmission::Unknown);
        assert!(capture.callback_progress().is_none());
        capture.completed_callback([4, 110, 480, 768]);
        assert_eq!(capture.callback_progress(), Some([4, 110, 480, 768]));
    }
    #[test]
    fn partial_row_is_rejected_without_waiting_for_writer() {
        let row = Row::<3>::default();
        assert!(row.read().is_none());
        row.sequence.store(1, Ordering::SeqCst); // deterministic latch inside write
        row.values[0].store(42, Ordering::SeqCst);
        assert!(row.read().is_none());
        row.values[1].store(7, Ordering::SeqCst);
        row.values[2].store(9, Ordering::SeqCst);
        row.sequence.store(2, Ordering::SeqCst);
        assert_eq!(row.read(), Some([42, 7, 9]));
    }
    #[test]
    fn exact_voice_epoch_serial_and_bounded_capture_are_required() {
        let trace = SubmissionCapture::default();
        trace.begin().unwrap();
        assert!(trace.begin().is_err());
        trace.produced(1, [4, 2, 90, 1, 2, 3, 768, 128, 1]);
        trace.submitted(1, [4, 2, 90, 100, 17, 480, 3]);
        assert!(trace.read(1, 4).is_some());
        assert!(trace.read(0, 4).is_none());
        assert!(trace.read(1, 3).is_none());
        trace.submitted(1, [4, 3, 90, 100, 17, 480, 3]);
        assert!(trace.read(1, 4).is_none());
        trace.submitted(1, [4, 2, 91, 100, 17, 480, 3]);
        assert!(trace.read(1, 4).is_none());
    }
    #[test]
    fn calibration_keeps_clock_uncertainty_and_overlapping_callback() {
        let request = Instant::now();
        let sample = StreamClockSample {
            before: request,
            after: request + Duration::from_micros(50),
            stream_nanos: 10_000_000,
        };
        let rounding = if cfg!(windows) { 0.0001 } else { 0. };
        assert_eq!(
            sample.callback_bounds_ms(28_000_000, request),
            [18. - rounding, 18.05 + rounding]
        );
        assert_eq!(
            sample.callback_bounds_ms(9_000_000, request),
            [-1. - rounding, -0.95 + rounding]
        );
    }
}
