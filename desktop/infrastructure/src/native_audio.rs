//! Callback boundary contains Copy frames, atomics and an SPSC consumer only.
use crate::submission_trace::SubmissionCapture;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rtrb::{Consumer, Producer, RingBuffer};
use skin_editor_audio::{BackgroundTrack, PcmBudget, PcmBuffer, PcmReservation, Runtime};
use std::sync::{
    Arc, Weak,
    atomic::{AtomicBool, AtomicU64, Ordering},
};

pub const OUTPUT_QUEUE_FRAMES: usize = 2048;
pub const OUTPUT_AHEAD_FRAMES: usize = 768;
pub const OUTPUT_BLOCK_FRAMES: usize = 128;
const STARTUP_OBSERVATIONS: usize = 32;
/// Recovery is opt-in until its native latency and workload gates are accepted.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SupplyPolicy {
    #[default]
    FixedAhead,
    RecoverLateWake,
}
struct CallbackObservation([AtomicU64; 4]);
struct UnderrunObservation([AtomicU64; 6]);
#[derive(Clone, Copy, Default)]
struct Frame {
    scene: [f32; 2],
    audition: [f32; 2],
    epochs: [u64; 2],
    background: [f32; 2],
    background_epoch: u64,
    background_position: u64,
    serial: u64,
    activity: u8,
    voices: [u64; 2],
}
struct Shared {
    capture: SubmissionCapture,
    monitor_volume: AtomicU64,
    background_epoch: AtomicU64,
    background_ack: AtomicU64,
    background_next: AtomicU64,
    background_gaps: AtomicU64,
    background_frames: AtomicU64,
    epochs: [AtomicU64; 2],
    acknowledged: [AtomicU64; 2],
    callbacks: AtomicU64,
    max_callback_frames: AtomicU64,
    frames: AtomicU64,
    empty: AtomicU64,
    empty_callbacks: AtomicU64,
    max_empty_callback_frames: AtomicU64,
    stale: AtomicU64,
    nonzero: AtomicU64,
    lane_nonzero: [AtomicU64; 2],
    lane_voices: [AtomicU64; 2],
    capture_startup: AtomicBool,
    startup: [CallbackObservation; STARTUP_OBSERVATIONS],
    underruns: [UnderrunObservation; STARTUP_OBSERVATIONS],
    supply_pass: AtomicU64,
    recovery_passes: AtomicU64,
    diagnostic_recovery_passes: AtomicU64,
    diagnostic_refill_passes: AtomicU64,
    diagnostic_refill_frames: AtomicU64,
    max_recovery_target: AtomicU64,
    idle_cutover: AtomicU64,
    idle_guard: AtomicU64,
    idle_sequence: AtomicU64,
    retimed_idle_frames: AtomicU64,
    failed: AtomicBool,
    closed: AtomicBool,
    // Accounts the six float payloads per frame in the ring, not tag metadata.
    _ring_payload: PcmReservation,
}
#[derive(Clone, Copy, Debug, Default)]
pub struct OutputStats {
    pub recovery_passes: u64,
    pub diagnostic_recovery_passes: u64,
    pub diagnostic_refill_passes: u64,
    pub diagnostic_refill_frames: u64,
    pub max_recovery_target: u64,
    pub background_next: u64,
    pub background_gaps: u64,
    pub background_frames: u64,
    pub callbacks: u64,
    pub max_callback_frames: u64,
    pub frames: u64,
    pub empty_frames: u64,
    pub empty_callbacks: u64,
    pub retimed_idle_frames: u64,
    pub max_empty_callback_frames: u64,
    pub stale_lane_frames: u64,
    pub nonzero_samples: u64,
    /// Valid pre-monitor lane frames submitted by the callback, not acoustic output.
    pub lane_nonzero_frames: [u64; 2],
    /// Monotonic Voice observation scoped to this output/runtime, never identity.
    pub submitted_voice_ids: [u64; 2],
    pub lane_epochs: [u64; 2],
    pub lane_acknowledged: [u64; 2],
    pub failed: bool,
}
pub use crate::output_fence::OutputFence;
pub(crate) use crate::output_fence::OutputMark;
#[derive(Clone)]
pub struct OutputControl(Arc<Shared>);
/// Opt-in scheduling metadata, never PCM or project ownership. Supply rows:
/// pass, previous sleep, owner wake, pump start/finish, previous pump finish,
/// queue before/after, frames produced, completed callback count.
#[derive(Debug)]
pub struct SupplyTimingSnapshot {
    pub completed_pass: u64,
    pub recent: [Option<[u64; 10]>; 4],
    pub slow_count: u64,
    pub slow: Vec<Option<[u64; 10]>>,
    /// Event: published supply head, reason bits (gap=1, empty=2), four
    /// callback contexts (44 values), four matching supply rows (40 values).
    pub event_count: u64,
    pub events: Vec<Option<[u64; 86]>>,
}
impl OutputControl {
    pub fn capture_origin(&self) -> Option<std::time::Instant> {
        self.0.capture.origin()
    }
    pub fn supply_timing(&self) -> SupplyTimingSnapshot {
        self.0.capture.scheduling()
    }
    /// Diagnostic only, armed before playback. No callback clocks or waits.
    pub fn capture_submissions(&self) -> Result<std::time::Instant, String> {
        if self.0.callbacks.load(Ordering::Acquire) != 0 {
            return Err("arm capture before playback".into());
        }
        self.0.capture.begin()
    }
    pub fn submission_trace(&self, lane: usize, voice: u64) -> Option<crate::SubmissionTrace> {
        self.0.capture.read(lane, voice)
    }
    /// Exact Voice after all output writes and callback statistics. This is a
    /// render-completion marker, not backend return, device consumption or sound.
    pub fn voice_submission(
        &self,
        lane: usize,
        voice: u64,
    ) -> Result<crate::VoiceSubmission, String> {
        if lane >= 2 || voice == 0 {
            return Err("invalid submission identity".into());
        }
        let epoch = self.0.epochs[lane].load(Ordering::Acquire);
        let value = self.0.capture.observe(lane, voice, epoch);
        if epoch != self.0.epochs[lane].load(Ordering::Acquire) {
            return Ok(crate::VoiceSubmission::Unknown);
        }
        Ok(value)
    }
    /// Ordinal, backend-supplied entry stamp (ns), frames, queued-before.
    /// Published at completion; entry time alone is never a completion upper bound.
    pub fn callback_progress(&self) -> Option<[u64; 4]> {
        self.0.capture.callback_progress()
    }
    /// Metadata only. Stage 1..7 is assigned by the diagnostic host, never a
    /// playback gate. Stage 0 is unclassified; entry-stage owns each callback.
    pub fn set_diagnostic_stage(&self, stage: u64) -> Result<(), String> {
        self.0.capture.set_stage(stage)
    }
    /// Per entry-stage: callbacks, requested frames, empty callbacks, empty
    /// frames, retimed idle frames. Read after stream drop for a coherent total.
    pub fn diagnostic_stage_counts(&self) -> [[u64; 5]; 8] {
        self.0.capture.stage_counts()
    }
    /// First 32 underruns, each with three prior callbacks and the empty one.
    /// Row: ordinal+1 (0=absent), supplied stamp, frames, queue before/after,
    /// empty, supply pass before/after, retimed, entry/exit stage. Null is an
    /// incomplete bounded read, never a valid empty row. No new callback clock.
    pub fn underrun_contexts(&self) -> Vec<Option<[[u64; 11]; 4]>> {
        self.0
            .capture
            .underrun_contexts(self.0.empty_callbacks.load(Ordering::Acquire).min(32) as usize)
    }

    /// First 32 empty callbacks: callback index, request, queued-before, empty,
    /// supply pass before/after. No callback clock, allocation or PCM owner.
    pub fn underrun_observations(&self) -> Vec<[u64; 6]> {
        let count = self
            .0
            .empty_callbacks
            .load(Ordering::Acquire)
            .min(STARTUP_OBSERVATIONS as u64) as usize;
        self.0.underruns[..count]
            .iter()
            .map(|row| std::array::from_fn(|i| row.0[i].load(Ordering::Relaxed)))
            .collect()
    }
    /// One bounded, opt-in startup window. No callback clocks or allocation.
    pub fn capture_startup(&self) -> Result<(), String> {
        if self.0.callbacks.load(Ordering::Acquire) != 0 {
            return Err("capture must be armed before play".into());
        }
        self.0.capture_startup.store(true, Ordering::Release);
        Ok(())
    }
    /// Rows: requested frames, queued frames at callback entry, empty frames, supply pass.
    pub fn startup_observations(&self) -> Vec<[u64; 4]> {
        let count = (self.0.callbacks.load(Ordering::Acquire) as usize).min(STARTUP_OBSERVATIONS);
        if !self.0.capture_startup.load(Ordering::Acquire) {
            return Vec::new();
        }
        self.0.startup[..count]
            .iter()
            .map(|r| std::array::from_fn(|i| r.0[i].load(Ordering::Relaxed)))
            .collect()
    }
    /// Last-stage monitoring attenuation, even for already queued Copy frames.
    pub fn set_monitor_volume(&self, permille: u16) -> Result<(), String> {
        if permille > 1000 {
            return Err("invalid monitor volume".into());
        }
        self.0
            .monitor_volume
            .store(u64::from(permille), Ordering::Release);
        Ok(())
    }

    pub fn is_closed(&self) -> bool {
        self.0.closed.load(Ordering::Acquire) || self.0.failed.load(Ordering::Acquire)
    }
    pub(crate) fn mark(&self) -> OutputMark {
        OutputMark {
            skin: OutputFence(std::array::from_fn(|i| {
                self.0.epochs[i].load(Ordering::Acquire)
            })),
            background: self.0.background_epoch.load(Ordering::Acquire),
        }
    }
    pub(crate) fn invalidate_skin(
        &self,
        scene: bool,
        audition: bool,
    ) -> Result<OutputMark, String> {
        for (i, change) in [scene, audition].into_iter().enumerate() {
            if change {
                self.0.epochs[i]
                    .fetch_update(Ordering::AcqRel, Ordering::Acquire, |n| n.checked_add(1))
                    .map_err(|_| "output epoch exhausted")?;
            }
        }
        Ok(self.mark())
    }
    pub(crate) fn invalidate_scene(&self) -> Result<OutputMark, String> {
        self.0
            .background_epoch
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |n| n.checked_add(1))
            .map_err(|_| "background epoch exhausted")?;
        self.invalidate_skin(true, false)
    }
    pub(crate) fn background_acknowledged(&self, mark: OutputMark) -> bool {
        self.0.background_ack.load(Ordering::Acquire) >= mark.background
            && self.0.acknowledged[0].load(Ordering::Acquire) >= mark.skin.0[0]
    }
    pub fn stats(&self) -> OutputStats {
        shared_stats(&self.0)
    }
    pub fn close(&self) {
        self.0.closed.store(true, Ordering::Release);
    }
}
pub struct OutputProducer {
    producer: Producer<Frame>,
    shared: Arc<Shared>,
    scene: PcmBuffer,
    audition: PcmBuffer,
    voice_marks: [[u64; 2]; OUTPUT_BLOCK_FRAMES],
    rate: u32,
    channels: u16,
    idle_reserve: usize,
    was_idle: bool,
    next_serial: u64,
    traced_voices: [u64; 2],
    owner_wake_ns: u64,
    previous_sleep_ns: u64,
    previous_pump_finish_ns: u64,
    policy: SupplyPolicy,
    period_frames: usize,
    previous_wake: Option<std::time::Instant>,
    late_wake: bool,
    diagnostic_recovery: bool,
}
pub struct OutputConsumer {
    consumer: Consumer<Frame>,
    shared: Arc<Shared>,
    channels: u16,
    idle_cutover: u64,
    idle_guard: u64,
    traced_voices: [u64; 2],
    context: [[u64; 11]; 4],
}
pub(crate) struct OutputSafety(Weak<Shared>);
impl OutputSafety {
    pub fn is_for(&self, output: &OutputProducer) -> bool {
        self.0.ptr_eq(&Arc::downgrade(&output.shared))
    }
    pub fn close(&self) {
        if let Some(shared) = self.0.upgrade() {
            shared.closed.store(true, Ordering::Release);
        }
    }
}
pub fn output_boundary(
    budget: &PcmBudget,
    rate: u32,
    channels: u16,
) -> Result<(OutputProducer, OutputConsumer), String> {
    if !matches!(rate, 44100 | 48000) || !matches!(channels, 1 | 2) {
        return Err("unsupported native rate/channels".into());
    }
    let payload = budget
        .reserve(OUTPUT_QUEUE_FRAMES * 6)
        .map_err(|e| format!("{e:?}"))?;
    let scene = budget
        .reserve(OUTPUT_BLOCK_FRAMES * usize::from(channels))
        .map_err(|e| format!("{e:?}"))?
        .allocate();
    let audition = budget
        .reserve(OUTPUT_BLOCK_FRAMES * usize::from(channels))
        .map_err(|e| format!("{e:?}"))?
        .allocate();
    let shared = Arc::new(Shared {
        capture: SubmissionCapture::default(),
        monitor_volume: AtomicU64::new(1000),
        background_epoch: AtomicU64::new(1),
        background_ack: AtomicU64::new(0),
        background_next: AtomicU64::new(0),
        background_gaps: AtomicU64::new(0),
        background_frames: AtomicU64::new(0),
        epochs: [AtomicU64::new(1), AtomicU64::new(1)],
        acknowledged: [AtomicU64::new(0), AtomicU64::new(0)],
        callbacks: AtomicU64::new(0),
        max_callback_frames: AtomicU64::new(0),
        frames: AtomicU64::new(0),
        empty: AtomicU64::new(0),
        empty_callbacks: AtomicU64::new(0),
        max_empty_callback_frames: AtomicU64::new(0),
        stale: AtomicU64::new(0),
        nonzero: AtomicU64::new(0),
        lane_nonzero: [AtomicU64::new(0), AtomicU64::new(0)],
        lane_voices: [AtomicU64::new(0), AtomicU64::new(0)],
        capture_startup: AtomicBool::new(false),
        startup: std::array::from_fn(|_| {
            CallbackObservation(std::array::from_fn(|_| AtomicU64::new(0)))
        }),
        underruns: std::array::from_fn(|_| {
            UnderrunObservation(std::array::from_fn(|_| AtomicU64::new(0)))
        }),
        supply_pass: AtomicU64::new(0),
        recovery_passes: AtomicU64::new(0),
        diagnostic_recovery_passes: AtomicU64::new(0),
        diagnostic_refill_passes: AtomicU64::new(0),
        diagnostic_refill_frames: AtomicU64::new(0),
        max_recovery_target: AtomicU64::new(0),
        idle_cutover: AtomicU64::new(0),
        idle_guard: AtomicU64::new(0),
        idle_sequence: AtomicU64::new(0),
        retimed_idle_frames: AtomicU64::new(0),
        failed: AtomicBool::new(false),
        closed: AtomicBool::new(false),
        _ring_payload: payload,
    });
    let (producer, consumer) = RingBuffer::new(OUTPUT_QUEUE_FRAMES);
    Ok((
        OutputProducer {
            producer,
            shared: shared.clone(),
            scene,
            audition,
            voice_marks: [[0; 2]; OUTPUT_BLOCK_FRAMES],
            rate,
            channels,
            idle_reserve: OUTPUT_AHEAD_FRAMES,
            was_idle: false,
            next_serial: 1,
            traced_voices: [0; 2],
            owner_wake_ns: 0,
            previous_sleep_ns: 0,
            previous_pump_finish_ns: 0,
            policy: SupplyPolicy::FixedAhead,
            period_frames: 0,
            previous_wake: None,
            late_wake: false,
            diagnostic_recovery: false,
        },
        OutputConsumer {
            consumer,
            shared,
            channels,
            idle_cutover: 0,
            idle_guard: 0,
            traced_voices: [0; 2],
            context: [[0; 11]; 4],
        },
    ))
}
impl OutputProducer {
    pub(crate) fn arm_diagnostic_recovery(&mut self) -> Result<(), String> {
        if self.policy != SupplyPolicy::RecoverLateWake
            || self.shared.capture.origin().is_none()
            || self.shared.closed.load(Ordering::Acquire)
        {
            return Err(
                "recovery probe requires capture and recovery policy on a live output".into(),
            );
        }
        if self.diagnostic_recovery {
            return Err("recovery probe already armed".into());
        }
        self.diagnostic_recovery = true;
        Ok(())
    }
    pub(crate) fn cancel_diagnostic_recovery(&mut self) {
        self.diagnostic_recovery = false;
    }
    pub(crate) fn owner_woke(&mut self, now: std::time::Instant) {
        self.late_wake = self.period_frames != 0
            && self.previous_wake.is_some_and(|previous| {
                now.saturating_duration_since(previous).as_nanos()
                    > self.period_frames as u128 * 1_000_000_000 / u128::from(self.rate)
            });
        self.previous_wake = Some(now);
        if let Some(origin) = self.shared.capture.origin() {
            self.owner_wake_ns = now.saturating_duration_since(origin).as_nanos() as u64;
        }
    }
    pub(crate) fn owner_sleeping(&mut self) {
        if let Some(origin) = self.shared.capture.origin() {
            self.previous_sleep_ns = origin.elapsed().as_nanos() as u64;
        }
    }
    fn complete_pump(
        &mut self,
        origin: Option<std::time::Instant>,
        started: Option<u64>,
        before: usize,
        produced: usize,
    ) {
        let pass = self.shared.supply_pass.fetch_add(1, Ordering::Release) + 1;
        if let Some(origin) = origin {
            let finished = origin.elapsed().as_nanos() as u64;
            self.shared.capture.completed_supply([
                pass,
                self.previous_sleep_ns,
                self.owner_wake_ns,
                started.unwrap(),
                finished,
                self.previous_pump_finish_ns,
                before as u64,
                self.queued_frames() as u64,
                produced as u64,
                self.shared.callbacks.load(Ordering::Acquire),
            ]);
            self.previous_pump_finish_ns = finished;
        }
    }
    pub fn control(&self) -> OutputControl {
        OutputControl(self.shared.clone())
    }
    pub(crate) fn safety(&self) -> OutputSafety {
        OutputSafety(Arc::downgrade(&self.shared))
    }
    /// Terminal fault for this stream; rebuilding requires a fresh explicit output instance.
    pub fn fail_device(&self) {
        self.shared.failed.store(true, Ordering::Release);
    }
    pub fn stats(&self) -> OutputStats {
        shared_stats(&self.shared)
    }
}
fn shared_stats(s: &Shared) -> OutputStats {
    // Read acknowledgements first: observing an epoch must include its lane counters.
    let acknowledged = std::array::from_fn(|i| s.acknowledged[i].load(Ordering::Acquire));
    OutputStats {
        recovery_passes: s.recovery_passes.load(Ordering::Acquire),
        diagnostic_recovery_passes: s.diagnostic_recovery_passes.load(Ordering::Acquire),
        diagnostic_refill_passes: s.diagnostic_refill_passes.load(Ordering::Acquire),
        diagnostic_refill_frames: s.diagnostic_refill_frames.load(Ordering::Acquire),
        max_recovery_target: s.max_recovery_target.load(Ordering::Acquire),
        background_next: s.background_next.load(Ordering::Acquire),
        background_gaps: s.background_gaps.load(Ordering::Acquire),
        background_frames: s.background_frames.load(Ordering::Acquire),
        callbacks: s.callbacks.load(Ordering::Acquire),
        max_callback_frames: s.max_callback_frames.load(Ordering::Acquire),
        frames: s.frames.load(Ordering::Acquire),
        empty_frames: s.empty.load(Ordering::Acquire),
        empty_callbacks: s.empty_callbacks.load(Ordering::Acquire),
        retimed_idle_frames: s.retimed_idle_frames.load(Ordering::Acquire),
        max_empty_callback_frames: s.max_empty_callback_frames.load(Ordering::Acquire),
        stale_lane_frames: s.stale.load(Ordering::Acquire),
        nonzero_samples: s.nonzero.load(Ordering::Acquire),
        lane_nonzero_frames: std::array::from_fn(|i| s.lane_nonzero[i].load(Ordering::Acquire)),
        submitted_voice_ids: std::array::from_fn(|i| s.lane_voices[i].load(Ordering::Acquire)),
        lane_epochs: std::array::from_fn(|i| s.epochs[i].load(Ordering::Acquire)),
        lane_acknowledged: acknowledged,
        failed: s.failed.load(Ordering::Acquire),
    }
}
impl OutputProducer {
    /// Configure before prefill/play. The backend period is a planning hint, not
    /// a guarantee on callback size. Preserve room for one active refill batch.
    pub fn configure_period(&mut self, period_frames: usize) -> Result<(), String> {
        let reserve = period_frames
            .checked_mul(2)
            .ok_or("period overflow")?
            .max(OUTPUT_AHEAD_FRAMES);
        if period_frames == 0
            || reserve > OUTPUT_QUEUE_FRAMES - OUTPUT_AHEAD_FRAMES - (OUTPUT_BLOCK_FRAMES - 1)
            || self.next_serial != 1
            || self.queued_frames() != 0
            || self.stats().callbacks != 0
        {
            return Err("unsupported period or output already started".into());
        }
        self.idle_reserve = reserve;
        self.period_frames = period_frames;
        self.shared
            .idle_guard
            .store((reserve - OUTPUT_AHEAD_FRAMES) as u64, Ordering::Release);
        Ok(())
    }
    /// Select only before prefill/play. No environment or project state is read.
    pub fn configure_supply_policy(&mut self, policy: SupplyPolicy) -> Result<(), String> {
        if self.period_frames == 0 || self.next_serial != 1 || self.stats().callbacks != 0 {
            return Err("configure supply policy after period and before output starts".into());
        }
        self.policy = policy;
        Ok(())
    }
    pub fn idle_reserve_frames(&self) -> usize {
        self.idle_reserve
    }
    pub fn queued_frames(&self) -> usize {
        OUTPUT_QUEUE_FRAMES - self.producer.slots()
    }
    pub fn invalidate(&self, scene: bool, audition: bool) -> Result<OutputFence, String> {
        for (i, change) in [scene, audition].into_iter().enumerate() {
            if change {
                self.shared.epochs[i]
                    .fetch_update(Ordering::AcqRel, Ordering::Acquire, |n| n.checked_add(1))
                    .map_err(|_| "output epoch exhausted")?;
            }
        }
        Ok(OutputFence(std::array::from_fn(|i| {
            self.shared.epochs[i].load(Ordering::Acquire)
        })))
    }
    pub fn acknowledged(&self, fence: OutputFence) -> bool {
        (0..2).all(|i| self.shared.acknowledged[i].load(Ordering::Acquire) >= fence.0[i])
    }
    /// Called by the control owner, never the native callback. Does not block.
    pub fn pump(&mut self, runtime: &mut Runtime) -> Result<usize, String> {
        self.pump_to(runtime, OUTPUT_AHEAD_FRAMES)
    }
    /// Startup silence can fill the larger ring before opening the stream gate.
    pub fn prefill(&mut self, runtime: &mut Runtime) -> Result<usize, String> {
        self.pump_to(runtime, OUTPUT_QUEUE_FRAMES)
    }
    fn pump_to(&mut self, runtime: &mut Runtime, target: usize) -> Result<usize, String> {
        self.pump_timeline(runtime, None, self.control().mark(), target)
    }
    pub(crate) fn pump_timeline(
        &mut self,
        runtime: &mut Runtime,
        mut background: Option<&mut BackgroundTrack>,
        mark: OutputMark,
        target: usize,
    ) -> Result<usize, String> {
        if runtime.output_format() != (self.rate, self.channels)
            || !runtime.pcm_budget().same_pool(self.scene.budget())
            || self.stats().failed
            || self.shared.closed.load(Ordering::Acquire)
        {
            return Err("output unavailable or mismatched runtime".into());
        }
        if background.as_ref().is_some_and(|b| {
            b.format() != (self.rate, self.channels)
                || !b.pcm().budget().same_pool(&runtime.pcm_budget())
        }) {
            return Err("background format or budget mismatch".into());
        }
        let capture = self.shared.capture.origin();
        let pump_ns = capture.map(|start| start.elapsed().as_nanos() as u64);
        let mut traced = [None; 2];
        let activity = runtime.output_activity();
        let idle =
            activity == [false, false] && background.as_ref().is_none_or(|b| !b.is_running());
        let leaving_idle = self.was_idle && !idle;
        let slots = self.producer.slots();
        // A delayed owner may resume just before a larger catch-up callback.
        // Refill despite an already satisfied normal target, once per wake.
        // This is bounded protection, not a guarantee against arbitrary stalls.
        let recovering = target == OUTPUT_AHEAD_FRAMES
            && !idle
            && self.policy == SupplyPolicy::RecoverLateWake
            && (self.late_wake || self.diagnostic_recovery);
        self.late_wake = false;
        let diagnostic_recovery = std::mem::take(&mut self.diagnostic_recovery);
        let target = if recovering {
            let observed = usize::try_from(self.shared.max_callback_frames.load(Ordering::Acquire))
                .unwrap_or(usize::MAX);
            let recovery = observed
                .saturating_add(OUTPUT_BLOCK_FRAMES)
                .max(self.period_frames * 2)
                .clamp(OUTPUT_AHEAD_FRAMES, OUTPUT_QUEUE_FRAMES)
                .div_ceil(OUTPUT_BLOCK_FRAMES)
                * OUTPUT_BLOCK_FRAMES;
            self.shared.recovery_passes.fetch_add(1, Ordering::Release);
            if diagnostic_recovery {
                self.shared
                    .diagnostic_recovery_passes
                    .fetch_add(1, Ordering::Release);
            }
            self.shared
                .max_recovery_target
                .fetch_max(recovery as u64, Ordering::Release);
            recovery
        } else if idle && target == OUTPUT_AHEAD_FRAMES {
            self.idle_reserve
        } else {
            target
        };
        let guard = if recovering {
            target.saturating_sub(OUTPUT_AHEAD_FRAMES)
        } else {
            self.idle_reserve - OUTPUT_AHEAD_FRAMES
        };
        let needed = if leaving_idle {
            // Append before publishing cutover. The consumer must never see
            // an incomplete replacement batch when it trims obsolete blank time.
            if recovering {
                target
                    .saturating_sub(guard.min(OUTPUT_QUEUE_FRAMES - slots))
                    .div_ceil(OUTPUT_BLOCK_FRAMES)
                    * OUTPUT_BLOCK_FRAMES
            } else {
                OUTPUT_AHEAD_FRAMES
            }
        } else {
            target
                .saturating_sub(OUTPUT_QUEUE_FRAMES - slots)
                .div_ceil(OUTPUT_BLOCK_FRAMES)
                * OUTPUT_BLOCK_FRAMES
        };
        if leaving_idle && slots < needed {
            self.complete_pump(capture, pump_ns, OUTPUT_QUEUE_FRAMES - slots, 0);
            return Ok(0);
        }
        let count = needed.min(slots / OUTPUT_BLOCK_FRAMES * OUTPUT_BLOCK_FRAMES);
        if count == 0 {
            self.was_idle = idle;
            self.complete_pump(capture, pump_ns, OUTPUT_QUEUE_FRAMES - slots, 0);
            return Ok(0);
        }
        let next = self
            .next_serial
            .checked_add(count as u64)
            .ok_or("output serial exhausted")?;
        let cutover = self.next_serial;
        let mut chunk = self
            .producer
            .write_chunk(count)
            .map_err(|_| "output backpressure")?;
        let (one, two) = chunk.as_mut_slices();
        let mut mask = 0;
        for (i, destination) in one.iter_mut().chain(two).enumerate() {
            if i.is_multiple_of(OUTPUT_BLOCK_FRAMES) {
                let active = runtime.output_activity();
                mask = u8::from(active[0]) | (u8::from(active[1]) << 1);
                runtime
                    .mix_split_buffers_marked(
                        &mut self.scene,
                        &mut self.audition,
                        &mut self.voice_marks,
                    )
                    .map_err(|e| format!("{e:?}"))?;
            }
            let offset = i % OUTPUT_BLOCK_FRAMES;
            let mut frame = Frame {
                epochs: mark.skin.0,
                background_epoch: mark.background,
                background_position: u64::MAX,
                serial: self.next_serial + i as u64,
                activity: mask,
                voices: self.voice_marks[offset],
                ..Frame::default()
            };
            if let Some((position, samples)) = background.as_mut().and_then(|b| b.next_frame()) {
                frame.background = samples;
                frame.background_position = position;
            }
            for c in 0..usize::from(self.channels) {
                frame.scene[c] = self.scene.samples()[offset * usize::from(self.channels) + c];
                frame.audition[c] =
                    self.audition.samples()[offset * usize::from(self.channels) + c];
            }
            if capture.is_some() {
                for (lane, data) in [frame.scene, frame.audition].iter().enumerate() {
                    if frame.voices[lane] > self.traced_voices[lane]
                        && data
                            .iter()
                            .take(usize::from(self.channels))
                            .any(|v| *v != 0.)
                    {
                        self.traced_voices[lane] = frame.voices[lane];
                        traced[lane] = Some([
                            frame.voices[lane],
                            frame.epochs[lane],
                            frame.serial,
                            i as u64,
                        ]);
                    }
                }
            }
            *destination = frame;
        }
        let before_ns = capture.map(|start| start.elapsed().as_nanos() as u64);
        chunk.commit_all();
        if let Some(origin) = capture {
            let after_ns = origin.elapsed().as_nanos() as u64;
            for (lane, row) in traced.into_iter().enumerate() {
                if let Some([voice, epoch, serial, offset]) = row {
                    self.shared.capture.produced(
                        lane,
                        [
                            voice,
                            epoch,
                            serial,
                            pump_ns.unwrap(),
                            before_ns.unwrap(),
                            after_ns,
                            (OUTPUT_QUEUE_FRAMES - slots) as u64,
                            count as u64,
                            offset,
                        ],
                    );
                }
            }
        }
        self.next_serial = next;
        if recovering && diagnostic_recovery {
            self.shared
                .diagnostic_refill_passes
                .fetch_add(1, Ordering::Release);
            self.shared
                .diagnostic_refill_frames
                .fetch_add(count as u64, Ordering::Release);
        }
        self.was_idle = idle;
        if leaving_idle {
            // One writer, bounded reader: never pair an old cutover with a
            // later recovery guard. A callback can conservatively defer trim.
            self.shared.idle_sequence.fetch_add(1, Ordering::SeqCst);
            self.shared.idle_guard.store(guard as u64, Ordering::SeqCst);
            self.shared.idle_cutover.store(cutover, Ordering::SeqCst);
            self.shared.idle_sequence.fetch_add(1, Ordering::SeqCst);
        }
        self.complete_pump(capture, pump_ns, OUTPUT_QUEUE_FRAMES - slots, count);
        Ok(count)
    }
}
impl OutputConsumer {
    fn render(&mut self, samples: usize, stamp: Option<u64>, mut write: impl FnMut(usize, f32)) {
        let diagnostic = self.shared.capture.origin().is_some();
        let capture = diagnostic && stamp.is_some();
        let stage = if diagnostic {
            self.shared.capture.stage()
        } else {
            0
        };
        let mut traced = [None; 2];
        let channels = usize::from(self.channels);
        let queued_before = self.consumer.slots();
        let pass = self.shared.supply_pass.load(Ordering::Acquire);
        let epochs =
            std::array::from_fn::<_, 2, _>(|i| self.shared.epochs[i].load(Ordering::Acquire));
        let failed = self.shared.failed.load(Ordering::Acquire)
            || self.shared.closed.load(Ordering::Acquire);
        let background_epoch = self.shared.background_epoch.load(Ordering::Acquire);
        let volume = self.shared.monitor_volume.load(Ordering::Acquire) as f32 / 1000.0;
        for _ in 0..3 {
            let sequence = self.shared.idle_sequence.load(Ordering::SeqCst);
            if !sequence.is_multiple_of(2) {
                continue;
            }
            let cutover = self.shared.idle_cutover.load(Ordering::SeqCst);
            let guard = self.shared.idle_guard.load(Ordering::SeqCst);
            if self.shared.idle_sequence.load(Ordering::SeqCst) == sequence {
                if cutover != self.idle_cutover {
                    self.idle_cutover = cutover;
                    self.idle_guard = guard;
                }
                break;
            }
        }
        let cutover = self.idle_cutover;
        let mut retimed = 0;
        let mut empty = 0;
        let mut stale = 0;
        let mut nonzero = 0;
        let mut lane_nonzero = [0u64; 2];
        let mut lane_voices = [0u64; 2];
        for frame_index in 0..samples.div_ceil(channels) {
            let mut mixed = [0.0f32; 2];
            let mut selected = None;
            // Total discarded prefix is bounded by the fixed ring, even if
            // another producer batch is published while this callback runs.
            while let Ok(frame) = self.consumer.pop() {
                let obsolete_idle = frame.serial < cutover
                    && (frame.background_position == u64::MAX
                        || frame.background_epoch != background_epoch)
                    && (0..2).all(|lane| {
                        frame.activity & (1 << lane) == 0 || frame.epochs[lane] != epochs[lane]
                    });
                if obsolete_idle && self.idle_guard == 0 && retimed < OUTPUT_QUEUE_FRAMES as u64 {
                    retimed += 1;
                    continue;
                }
                if frame.serial < cutover {
                    self.idle_guard = self.idle_guard.saturating_sub(1);
                } else {
                    self.idle_guard = 0;
                }
                selected = Some(frame);
                break;
            }
            if let Some(frame) = selected {
                if !failed
                    && frame.background_epoch == background_epoch
                    && frame.background_position != u64::MAX
                {
                    if frame.background_position
                        != self.shared.background_next.load(Ordering::Relaxed)
                    {
                        self.shared.background_gaps.fetch_add(1, Ordering::Relaxed);
                    }
                    self.shared
                        .background_next
                        .store(frame.background_position + 1, Ordering::Release);
                    self.shared
                        .background_frames
                        .fetch_add(1, Ordering::Relaxed);
                    for (value, bg) in mixed.iter_mut().zip(frame.background).take(channels) {
                        *value += bg;
                    }
                }
                for (lane, data) in [frame.scene, frame.audition].into_iter().enumerate() {
                    if !failed && frame.epochs[lane] == epochs[lane] {
                        let audible = data.iter().take(channels).any(|v| *v != 0.0);
                        lane_nonzero[lane] += u64::from(audible);
                        if audible {
                            lane_voices[lane] = lane_voices[lane].max(frame.voices[lane]);
                            if capture && frame.voices[lane] > self.traced_voices[lane] {
                                self.traced_voices[lane] = frame.voices[lane];
                                traced[lane] = Some([
                                    frame.voices[lane],
                                    epochs[lane],
                                    frame.serial,
                                    stamp.unwrap(),
                                    frame_index as u64,
                                    (samples / channels) as u64,
                                    self.shared.callbacks.load(Ordering::Relaxed),
                                ]);
                            }
                        }
                        for c in 0..channels {
                            mixed[c] += data[c];
                        }
                    } else {
                        stale += 1;
                    }
                }
            } else {
                empty += 1;
            }
            for (c, value) in mixed.iter().take(channels).enumerate() {
                let index = frame_index * channels + c;
                if index < samples {
                    let value = (value * volume).clamp(-1.0, 1.0);
                    nonzero += u64::from(value != 0.0);
                    write(index, value);
                }
            }
        }
        for (i, epoch) in epochs.into_iter().enumerate() {
            self.shared.lane_nonzero[i].fetch_add(lane_nonzero[i], Ordering::Release);
            self.shared.lane_voices[i].fetch_max(lane_voices[i], Ordering::Release);
            self.shared.acknowledged[i].store(epoch, Ordering::Release);
        }
        let index = self.shared.callbacks.load(Ordering::Relaxed) as usize;
        if self.shared.capture_startup.load(Ordering::Acquire) && index < STARTUP_OBSERVATIONS {
            for (slot, value) in self.shared.startup[index].0.iter().zip([
                (samples / channels) as u64,
                queued_before as u64,
                empty,
                pass,
            ]) {
                slot.store(value, Ordering::Relaxed);
            }
        }
        self.shared.callbacks.fetch_add(1, Ordering::Release);
        self.shared
            .background_ack
            .store(background_epoch, Ordering::Release);
        self.shared
            .max_callback_frames
            .fetch_max((samples / channels) as u64, Ordering::Relaxed);
        self.shared
            .frames
            .fetch_add((samples / channels) as u64, Ordering::Relaxed);
        let empty_index = self.shared.empty_callbacks.load(Ordering::Relaxed) as usize;
        let pass_after = if diagnostic || empty > 0 {
            self.shared.supply_pass.load(Ordering::Acquire)
        } else {
            0
        };
        if empty > 0 {
            let row = empty_index;
            if row < STARTUP_OBSERVATIONS {
                for (slot, value) in self.shared.underruns[row].0.iter().zip([
                    index as u64,
                    (samples / channels) as u64,
                    queued_before as u64,
                    empty,
                    pass,
                    pass_after,
                ]) {
                    slot.store(value, Ordering::Relaxed);
                }
            }
            self.shared.empty_callbacks.fetch_add(1, Ordering::Release);
            self.shared
                .max_empty_callback_frames
                .fetch_max((samples / channels) as u64, Ordering::Relaxed);
        }
        self.shared
            .retimed_idle_frames
            .fetch_add(retimed, Ordering::Relaxed);
        self.shared.empty.fetch_add(empty, Ordering::Relaxed);
        self.shared.stale.fetch_add(stale, Ordering::Relaxed);
        self.shared.nonzero.fetch_add(nonzero, Ordering::Relaxed);
        if diagnostic {
            self.context.rotate_left(1);
            self.context[3] = [
                index as u64 + 1,
                stamp.unwrap_or(0),
                (samples / channels) as u64,
                queued_before as u64,
                self.consumer.slots() as u64,
                empty,
                pass,
                pass_after,
                retimed,
                stage,
                self.shared.capture.stage(),
            ];
            self.shared
                .capture
                .completed_context(self.context, empty_index);
        }
        // Diagnostic publication only. No clock, allocation, wait or PCM owner;
        // producer pacing, ring policy, output samples and invalidation stay intact.
        for (lane, row) in traced.into_iter().enumerate() {
            if let Some(row) = row {
                self.shared.capture.submitted(lane, row);
            }
        }
        if capture {
            self.shared.capture.completed_callback([
                index as u64,
                stamp.unwrap(),
                (samples / channels) as u64,
                queued_before as u64,
            ]);
        }
    }
    /// Device-free tests invoke exactly the callback consumer used by CPAL.
    pub fn render_f32(&mut self, output: &mut [f32]) {
        self.render(output.len(), None, |i, value| output[i] = value);
    }
    #[cfg(test)]
    fn render_i16(&mut self, output: &mut [i16]) {
        self.render(output.len(), None, |i, value| {
            output[i] = (value * 32768.).round().clamp(-32768., 32767.) as i16
        });
    }
}
pub struct NativeOutput {
    // Drop stream first; callback stops before the control-side ring/resources.
    stream: cpal::Stream,
    pub producer: OutputProducer,
    pub rate: u32,
    pub channels: u16,
    pub sample_format: cpal::SampleFormat,
    pub period_frames: u32,
}
pub struct NativeStream(cpal::Stream);
impl NativeStream {
    /// Samples the stream clock on the control thread; never called in a callback.
    pub fn clock_sample(&self) -> Result<crate::StreamClockSample, String> {
        let before = std::time::Instant::now();
        let stamp = self.0.now();
        let after = std::time::Instant::now();
        Ok(crate::StreamClockSample {
            before,
            after,
            stream_nanos: stamp
                .as_nanos()
                .try_into()
                .map_err(|_| "stream timestamp overflow")?,
        })
    }
    pub fn play(&self) -> Result<(), String> {
        self.0.play().map_err(|e| e.to_string())
    }
}
impl NativeOutput {
    pub fn into_parts(self) -> (NativeStream, OutputProducer) {
        (NativeStream(self.stream), self.producer)
    }
    pub fn open(budget: &PcmBudget) -> Result<Self, String> {
        let device = cpal::default_host()
            .default_output_device()
            .ok_or("no default output device")?;
        let supported = device.default_output_config().map_err(|e| e.to_string())?;
        let rate = supported.sample_rate();
        let channels = supported.channels();
        let sample_format = supported.sample_format();
        let (mut producer, mut consumer) = output_boundary(budget, rate, channels)?;
        let fault = producer.shared.clone();
        let error = move |_| {
            fault.failed.store(true, Ordering::Release);
        };
        let mut config = supported.config();
        if let cpal::SupportedBufferSize::Range { min, max } = *supported.buffer_size() {
            config.buffer_size =
                cpal::BufferSize::Fixed(128u32.clamp(min.max(1), max.max(min).max(1)));
        }
        let stream = match sample_format {
            cpal::SampleFormat::F32 => device.build_output_stream(
                config,
                move |out: &mut [f32], info| {
                    let stamp = u64::try_from(info.timestamp().callback.as_nanos()).ok();
                    consumer.render(out.len(), stamp, |i, value| out[i] = value);
                },
                error,
                Some(std::time::Duration::from_secs(3)),
            ),
            cpal::SampleFormat::I16 => device.build_output_stream(
                config,
                move |out: &mut [i16], info| {
                    let stamp = u64::try_from(info.timestamp().callback.as_nanos()).ok();
                    consumer.render(out.len(), stamp, |i, value| {
                        out[i] = (value * 32768.).round().clamp(-32768., 32767.) as i16
                    });
                },
                error,
                Some(std::time::Duration::from_secs(3)),
            ),
            _ => return Err("native sample format not supported by this increment".into()),
        }
        .map_err(|e| e.to_string())?;
        let period_frames = stream.buffer_size().map_err(|e| e.to_string())?;
        producer.configure_period(period_frames as usize)?;
        Ok(Self {
            stream,
            producer,
            rate,
            channels,
            sample_format,
            period_frames,
        })
    }
    /// Explicitly start only after prefill; opening/preparing never emits sound.
    pub fn play(&self) -> Result<(), String> {
        self.stream.play().map_err(|e| e.to_string())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn explicit_recovery_requires_capture_policy_and_a_single_pending_probe() {
        let budget = PcmBudget::default();
        let (mut producer, _) = output_boundary(&budget, 48000, 1).unwrap();
        producer.configure_period(480).unwrap();
        assert!(producer.arm_diagnostic_recovery().is_err());
        producer
            .configure_supply_policy(SupplyPolicy::RecoverLateWake)
            .unwrap();
        assert!(producer.arm_diagnostic_recovery().is_err());
        producer.control().capture_submissions().unwrap();
        producer.arm_diagnostic_recovery().unwrap();
        assert!(producer.arm_diagnostic_recovery().is_err());
        producer.cancel_diagnostic_recovery();
        producer.arm_diagnostic_recovery().unwrap();
        producer.control().close();
        assert!(producer.arm_diagnostic_recovery().is_err());
    }
    #[test]
    fn explicit_probe_counts_real_commits_separately_from_recovery_attempts() {
        use skin_editor_audio::Channel;
        let budget = PcmBudget::default();
        let (mut producer, mut consumer) = output_boundary(&budget, 48000, 1).unwrap();
        producer.configure_period(480).unwrap();
        producer
            .configure_supply_policy(SupplyPolicy::RecoverLateWake)
            .unwrap();
        producer.control().capture_submissions().unwrap();
        let mut runtime = Runtime::with_budget(48000, 1, budget.clone()).unwrap();
        let id = ready_tone(&mut runtime, 96000);
        runtime
            .trigger(runtime.token(Channel::Audition), id, 0)
            .unwrap();
        let before = budget.usage().bytes;
        producer.arm_diagnostic_recovery().unwrap();
        assert_eq!(producer.pump(&mut runtime).unwrap(), 1024);
        let stats = producer.stats();
        assert_eq!(
            (
                stats.recovery_passes,
                stats.diagnostic_recovery_passes,
                stats.diagnostic_refill_passes,
                stats.diagnostic_refill_frames
            ),
            (1, 1, 1, 1024)
        );
        assert_eq!(budget.usage().bytes, before);
        consumer.render(512, None, |_, value| assert_eq!(value, 0.25));
        producer.pump(&mut runtime).unwrap();
        assert_eq!(producer.queued_frames(), 768);
        assert_eq!(producer.stats().diagnostic_refill_frames, 1024);
        drop((producer, consumer, runtime));
        assert_eq!(budget.usage().bytes, 0);
    }
    #[test]
    fn full_ring_diagnostic_attempt_is_not_misreported_as_a_completed_refill() {
        use skin_editor_audio::Channel;
        let budget = PcmBudget::default();
        let (mut producer, _) = output_boundary(&budget, 48000, 1).unwrap();
        producer.configure_period(480).unwrap();
        producer
            .configure_supply_policy(SupplyPolicy::RecoverLateWake)
            .unwrap();
        producer.control().capture_submissions().unwrap();
        let mut runtime = Runtime::with_budget(48000, 1, budget).unwrap();
        let id = ready_tone(&mut runtime, 4096);
        producer.prefill(&mut runtime).unwrap();
        runtime
            .trigger(runtime.token(Channel::Audition), id, runtime.mix_frame())
            .unwrap();
        let before = runtime.mix_frame();
        producer.arm_diagnostic_recovery().unwrap();
        assert_eq!(producer.pump(&mut runtime).unwrap(), 0);
        assert_eq!(runtime.mix_frame(), before);
        let stats = producer.stats();
        assert_eq!(
            (
                stats.diagnostic_recovery_passes,
                stats.diagnostic_refill_passes,
                stats.diagnostic_refill_frames
            ),
            (1, 0, 0)
        );
    }
    #[test]
    fn late_wake_replenishes_before_a_larger_callback_even_above_normal_target() {
        use skin_editor_audio::Channel;
        for (tail, delay_ms) in [(128, 54), (224, 34)] {
            let budget = PcmBudget::default();
            let (mut producer, mut callback) = output_boundary(&budget, 48000, 1).unwrap();
            producer.configure_period(480).unwrap();
            producer
                .configure_supply_policy(SupplyPolicy::RecoverLateWake)
                .unwrap();
            let mut runtime = Runtime::with_budget(48000, 1, budget.clone()).unwrap();
            let id = ready_tone(&mut runtime, 96000);
            runtime
                .set_scene_running(runtime.token(Channel::Scene), true)
                .unwrap();
            runtime
                .trigger(runtime.token(Channel::Scene), id, 0)
                .unwrap();
            producer.prefill(&mut runtime).unwrap();
            let wake = std::time::Instant::now();
            producer.owner_woke(wake);
            callback.render(1056, None, |_, _| {});
            callback.render(tail, None, |_, _| {});
            assert!(producer.queued_frames() >= OUTPUT_AHEAD_FRAMES);
            producer.owner_woke(wake + std::time::Duration::from_millis(delay_ms));
            producer.pump(&mut runtime).unwrap();
            assert_eq!(producer.stats().recovery_passes, 1);
            assert_eq!(producer.stats().max_recovery_target, 1280);
            callback.render(1056, None, |_, value| assert_eq!(value, 0.25));
            assert_eq!(producer.stats().empty_frames, 0);
            drop((producer, callback, runtime));
            assert_eq!(budget.usage().bytes, 0);
        }
    }
    #[test]
    fn fixed_policy_preserves_the_original_larger_callback_failure() {
        use skin_editor_audio::Channel;
        for (tail, missing) in [(128, 192), (224, 288)] {
            let budget = PcmBudget::default();
            let (mut producer, mut callback) = output_boundary(&budget, 48000, 1).unwrap();
            producer.configure_period(480).unwrap();
            let mut runtime = Runtime::with_budget(48000, 1, budget).unwrap();
            let id = ready_tone(&mut runtime, 96000);
            runtime
                .set_scene_running(runtime.token(Channel::Scene), true)
                .unwrap();
            runtime
                .trigger(runtime.token(Channel::Scene), id, 0)
                .unwrap();
            producer.prefill(&mut runtime).unwrap();
            let now = std::time::Instant::now();
            producer.owner_woke(now);
            callback.render(1056, None, |_, _| {});
            callback.render(tail, None, |_, _| {});
            producer.owner_woke(now + std::time::Duration::from_millis(54));
            assert_eq!(producer.pump(&mut runtime).unwrap(), 0);
            callback.render(1056, None, |_, _| {});
            assert_eq!(producer.stats().empty_frames, missing);
            assert_eq!(producer.stats().recovery_passes, 0);
        }
    }
    #[test]
    fn recovery_is_opt_in_once_per_late_wake_and_not_a_permanent_larger_target() {
        use skin_editor_audio::Channel;
        let budget = PcmBudget::default();
        let (mut producer, mut callback) = output_boundary(&budget, 48000, 1).unwrap();
        assert!(
            producer
                .configure_supply_policy(SupplyPolicy::RecoverLateWake)
                .is_err()
        );
        producer.configure_period(480).unwrap();
        producer
            .configure_supply_policy(SupplyPolicy::RecoverLateWake)
            .unwrap();
        let mut runtime = Runtime::with_budget(48000, 1, budget).unwrap();
        let id = ready_tone(&mut runtime, 96000);
        runtime
            .set_scene_running(runtime.token(Channel::Scene), true)
            .unwrap();
        runtime
            .trigger(runtime.token(Channel::Scene), id, 0)
            .unwrap();
        let now = std::time::Instant::now();
        producer.owner_woke(now);
        producer.pump(&mut runtime).unwrap();
        assert_eq!(producer.queued_frames(), 768);
        producer.owner_woke(now + std::time::Duration::from_millis(10));
        assert_eq!(producer.pump(&mut runtime).unwrap(), 0); // exactly one period
        producer.owner_woke(now + std::time::Duration::from_millis(21));
        producer.pump(&mut runtime).unwrap();
        assert_eq!(producer.queued_frames(), 1024);
        assert_eq!(producer.stats().recovery_passes, 1);
        callback.render(512, None, |_, _| {});
        producer.pump(&mut runtime).unwrap();
        assert_eq!(producer.queued_frames(), 768);
        assert_eq!(producer.stats().recovery_passes, 1);
        assert!(
            producer
                .configure_supply_policy(SupplyPolicy::FixedAhead)
                .is_err()
        );
    }
    #[test]
    fn late_idle_exit_keeps_a_bounded_blank_guard_without_consuming_voice_on_backpressure() {
        use skin_editor_audio::Channel;
        let budget = PcmBudget::default();
        let (mut producer, mut callback) = output_boundary(&budget, 48000, 1).unwrap();
        producer.configure_period(480).unwrap();
        producer
            .configure_supply_policy(SupplyPolicy::RecoverLateWake)
            .unwrap();
        let mut runtime = Runtime::with_budget(48000, 1, budget).unwrap();
        let id = ready_tone(&mut runtime, 96000);
        let now = std::time::Instant::now();
        producer.owner_woke(now);
        producer.prefill(&mut runtime).unwrap();
        callback.render(1056, None, |_, _| {});
        producer.pump(&mut runtime).unwrap(); // idle, 992 frames left
        runtime
            .trigger(runtime.token(Channel::Audition), id, runtime.mix_frame())
            .unwrap();
        producer.owner_woke(now + std::time::Duration::from_millis(34));
        producer.pump(&mut runtime).unwrap();
        let mut out = [0.0; 1056];
        callback.render_f32(&mut out);
        assert_eq!(out[..512], [0.0; 512]);
        assert_eq!(out[512..], [0.25; 544]);
        assert_eq!(producer.stats().empty_frames, 0);
        // Fill idle queue in a fresh boundary: no replacement fits yet.
        let budget = PcmBudget::default();
        let (mut producer, _) = output_boundary(&budget, 48000, 1).unwrap();
        producer.configure_period(480).unwrap();
        producer
            .configure_supply_policy(SupplyPolicy::RecoverLateWake)
            .unwrap();
        let mut runtime = Runtime::with_budget(48000, 1, budget).unwrap();
        let id = ready_tone(&mut runtime, 96000);
        producer.owner_woke(now);
        producer.prefill(&mut runtime).unwrap();
        runtime
            .trigger(runtime.token(Channel::Audition), id, runtime.mix_frame())
            .unwrap();
        let before = runtime.mix_frame();
        producer.owner_woke(now + std::time::Duration::from_millis(34));
        assert_eq!(producer.pump(&mut runtime).unwrap(), 0);
        assert_eq!(runtime.mix_frame(), before);
        assert_eq!(producer.queued_frames(), OUTPUT_QUEUE_FRAMES);
    }
    #[test]
    fn recovery_keeps_background_continuous_and_queued_lane_cancellation_independent() {
        use skin_editor_audio::Channel;
        let budget = PcmBudget::default();
        let (mut producer, mut callback) = output_boundary(&budget, 48000, 1).unwrap();
        producer.configure_period(480).unwrap();
        producer
            .configure_supply_policy(SupplyPolicy::RecoverLateWake)
            .unwrap();
        let mut runtime = Runtime::with_budget(48000, 1, budget.clone()).unwrap();
        let id = ready_tone(&mut runtime, 96000);
        runtime
            .set_scene_running(runtime.token(Channel::Scene), true)
            .unwrap();
        for lane in [Channel::Scene, Channel::Audition] {
            runtime.trigger(runtime.token(lane), id, 0).unwrap();
        }
        let mut pcm = budget.reserve(8192).unwrap().allocate();
        pcm.samples_mut().unwrap().fill(0.125);
        let mut background = BackgroundTrack::new(pcm, 48000, 1).unwrap();
        background.set_running(true);
        let output = producer.control();
        let mark = output.mark();
        let before = budget.usage().bytes;
        let now = std::time::Instant::now();
        producer.owner_woke(now);
        producer
            .pump_timeline(
                &mut runtime,
                Some(&mut background),
                mark,
                OUTPUT_QUEUE_FRAMES,
            )
            .unwrap();
        callback.render(1056, None, |_, value| assert_eq!(value, 0.625));
        callback.render(224, None, |_, value| assert_eq!(value, 0.625));
        producer.owner_woke(now + std::time::Duration::from_millis(34));
        producer
            .pump_timeline(
                &mut runtime,
                Some(&mut background),
                mark,
                OUTPUT_AHEAD_FRAMES,
            )
            .unwrap();
        assert_eq!(budget.usage().bytes, before);
        output.invalidate_skin(true, false).unwrap();
        callback.render(1056, None, |_, value| assert_eq!(value, 0.375));
        output.invalidate_skin(false, true).unwrap();
        callback.render(224, None, |_, value| assert_eq!(value, 0.125));
        assert_eq!(output.stats().background_next, 2560);
        assert_eq!(output.stats().background_gaps, 0);
        assert_eq!(output.stats().empty_frames, 0);
        drop((producer, callback, runtime, background, output));
        assert_eq!(budget.usage().bytes, 0);
    }
    #[test]
    fn recovery_cannot_erase_a_callback_that_drained_before_the_owner_woke() {
        use skin_editor_audio::Channel;
        let budget = PcmBudget::default();
        let (mut producer, mut callback) = output_boundary(&budget, 48000, 1).unwrap();
        producer.configure_period(480).unwrap();
        producer
            .configure_supply_policy(SupplyPolicy::RecoverLateWake)
            .unwrap();
        let mut runtime = Runtime::with_budget(48000, 1, budget).unwrap();
        let id = ready_tone(&mut runtime, 96000);
        runtime
            .trigger(runtime.token(Channel::Audition), id, 0)
            .unwrap();
        let now = std::time::Instant::now();
        producer.owner_woke(now);
        producer.pump(&mut runtime).unwrap();
        callback.render(1056, None, |_, _| {});
        assert_eq!(producer.stats().empty_frames, 288);
        producer.owner_woke(now + std::time::Duration::from_millis(34));
        producer.pump(&mut runtime).unwrap();
        assert_eq!(producer.stats().empty_frames, 288);
        callback.render(1056, None, |_, value| assert_eq!(value, 0.25));
        assert_eq!(producer.stats().empty_frames, 288);
    }
    #[test]
    fn recovery_caps_unknown_larger_demand_at_existing_ring_capacity() {
        use skin_editor_audio::Channel;
        let budget = PcmBudget::default();
        let (mut producer, _) = output_boundary(&budget, 48000, 1).unwrap();
        producer.configure_period(480).unwrap();
        producer
            .configure_supply_policy(SupplyPolicy::RecoverLateWake)
            .unwrap();
        producer
            .shared
            .max_callback_frames
            .store(u64::MAX, Ordering::Release);
        let mut runtime = Runtime::with_budget(48000, 1, budget).unwrap();
        let id = ready_tone(&mut runtime, 96000);
        runtime
            .trigger(runtime.token(Channel::Audition), id, 0)
            .unwrap();
        let now = std::time::Instant::now();
        producer.owner_woke(now);
        producer.owner_woke(now + std::time::Duration::from_millis(34));
        producer.pump(&mut runtime).unwrap();
        assert_eq!(producer.queued_frames(), OUTPUT_QUEUE_FRAMES);
        assert_eq!(
            producer.stats().max_recovery_target,
            OUTPUT_QUEUE_FRAMES as u64
        );
    }
    #[test]
    fn callback_defers_an_incomplete_cutover_guard_pair_without_waiting() {
        use skin_editor_audio::Channel;
        let budget = PcmBudget::default();
        let (mut producer, mut callback) = output_boundary(&budget, 48000, 1).unwrap();
        producer.configure_period(480).unwrap();
        producer
            .configure_supply_policy(SupplyPolicy::RecoverLateWake)
            .unwrap();
        let mut runtime = Runtime::with_budget(48000, 1, budget).unwrap();
        let id = ready_tone(&mut runtime, 4096);
        producer.pump(&mut runtime).unwrap();
        runtime
            .trigger(runtime.token(Channel::Audition), id, runtime.mix_frame())
            .unwrap();
        producer.pump(&mut runtime).unwrap();
        let cutover = producer.shared.idle_cutover.load(Ordering::SeqCst);
        producer.shared.idle_sequence.store(3, Ordering::SeqCst);
        producer.shared.idle_guard.store(512, Ordering::SeqCst);
        callback.render(480, None, |_, value| assert_eq!(value, 0.0));
        assert_eq!(callback.idle_cutover, 0);
        assert_eq!(producer.stats().retimed_idle_frames, 0);
        producer.shared.idle_guard.store(192, Ordering::SeqCst);
        producer.shared.idle_sequence.store(4, Ordering::SeqCst);
        let mut out = [0.0; 480];
        callback.render_f32(&mut out);
        assert_eq!(callback.idle_cutover, cutover);
        assert_eq!(out[..192], [0.0; 192]);
        assert_eq!(out[192..], [0.25; 288]);
        assert_eq!(producer.stats().empty_frames, 0);
    }
    #[test]
    fn diagnostic_context_identifies_three_callback_burst_and_stage_crossing() {
        let budget = PcmBudget::default();
        let (mut producer, mut callback) = output_boundary(&budget, 48000, 1).unwrap();
        let output = producer.control();
        assert!(output.set_diagnostic_stage(4).is_err());
        producer.configure_period(480).unwrap();
        let mut runtime = Runtime::with_budget(48000, 1, budget.clone()).unwrap();
        assert_eq!(producer.pump(&mut runtime).unwrap(), 1024);
        output.capture_submissions().unwrap();
        assert!(output.set_diagnostic_stage(0).is_err());
        assert!(output.set_diagnostic_stage(8).is_err());
        output.set_diagnostic_stage(4).unwrap();
        for index in 0..3 {
            callback.render(480, Some(100 + index), |_, _| {
                if index == 2 {
                    output.set_diagnostic_stage(6).unwrap();
                }
            });
        }
        let rows = output.underrun_contexts();
        let context = rows[0].unwrap();
        assert_eq!(context[0], [0; 11]);
        assert_eq!(context[1], [1, 100, 480, 1024, 544, 0, 1, 1, 0, 4, 4]);
        assert_eq!(context[2], [2, 101, 480, 544, 64, 0, 1, 1, 0, 4, 4]);
        assert_eq!(context[3], [3, 102, 480, 64, 0, 416, 1, 1, 0, 4, 6]);
        assert_eq!(output.diagnostic_stage_counts()[4], [3, 1440, 1, 416, 0]);
        assert_eq!(output.diagnostic_stage_counts()[6], [0; 5]);
        drop((callback, producer, output, runtime));
        assert_eq!(budget.usage().bytes, 0);
    }
    #[test]
    fn diagnostic_context_is_bounded_keeps_first_events_and_total_counts() {
        let budget = PcmBudget::default();
        let (producer, mut callback) = output_boundary(&budget, 48000, 1).unwrap();
        let output = producer.control();
        output.capture_submissions().unwrap();
        output.set_diagnostic_stage(7).unwrap();
        for _ in 0..40 {
            callback.render_f32(&mut [0.; 480]);
        }
        let rows = output.underrun_contexts();
        assert_eq!(rows.len(), 32);
        assert_eq!(rows[0].unwrap()[3][..6], [1, 0, 480, 0, 0, 480]);
        assert_eq!(rows[31].unwrap()[3][0], 32);
        assert_eq!(
            output.diagnostic_stage_counts()[7],
            [40, 19200, 40, 19200, 0]
        );
        assert_eq!(output.stats().empty_callbacks, 40);
        drop((callback, producer, output));
        assert_eq!(budget.usage().bytes, 0);
    }
    #[test]
    fn completion_marker_waits_for_entire_buffer_and_preserves_epoch_identity() {
        use crate::VoiceSubmission;
        use skin_editor_audio::Channel;
        let budget = PcmBudget::default();
        let (mut producer, mut callback) = output_boundary(&budget, 48000, 1).unwrap();
        let output = producer.control();
        output.capture_submissions().unwrap();
        let mut runtime = Runtime::with_budget(48000, 1, budget.clone()).unwrap();
        let id = ready_tone(&mut runtime, 4096);
        let voice = runtime
            .trigger(runtime.token(Channel::Audition), id, 0)
            .unwrap()
            .voice
            .unwrap();
        producer.pump(&mut runtime).unwrap();
        let mut writes = 0;
        callback.render(480, Some(123), |_, _| {
            writes += 1;
            assert!(!matches!(
                output.voice_submission(1, voice).unwrap(),
                VoiceSubmission::Complete(_)
            ));
        });
        assert_eq!(writes, 480);
        assert_eq!(output.stats().frames, 480);
        let VoiceSubmission::Complete(row) = output.voice_submission(1, voice).unwrap() else {
            panic!("no exact completion");
        };
        assert_eq!(row, output.submission_trace(1, voice).unwrap().callback);
        assert_eq!(output.callback_progress(), Some([0, 123, 480, 768]));
        output.invalidate_skin(false, true).unwrap();
        assert_eq!(
            output.voice_submission(1, voice).unwrap(),
            VoiceSubmission::Invalidated
        );
        assert!(output.voice_submission(2, voice).is_err());
        assert!(output.voice_submission(0, 0).is_err());
        drop((producer, callback, output, runtime));
        assert_eq!(budget.usage().bytes, 0);
    }
    #[test]
    fn controlled_callback_owner_phases_change_latency_without_changing_supply_policy() {
        use crate::VoiceSubmission;
        use skin_editor_audio::Channel;
        let mut latencies = std::collections::BTreeSet::new();
        // Virtual quarter-ms ticks, 480 frames/10 ms callbacks, unchanged
        // 2 ms owner and 768-frame active target. No device, sleeps or race guess.
        for owner_phase in [0, 4] {
            for request_phase in 0..40 {
                let budget = PcmBudget::default();
                let (mut producer, mut callback) = output_boundary(&budget, 48000, 1).unwrap();
                producer.configure_period(480).unwrap();
                let output = producer.control();
                output.capture_submissions().unwrap();
                let mut runtime = Runtime::with_budget(48000, 1, budget.clone()).unwrap();
                let id = ready_tone(&mut runtime, 96000);
                runtime
                    .set_scene_running(runtime.token(Channel::Scene), true)
                    .unwrap();
                runtime
                    .trigger(runtime.token(Channel::Scene), id, 0)
                    .unwrap();
                producer.prefill(&mut runtime).unwrap();
                let arrival = 400 + request_phase;
                let mut voice = None;
                let mut completed = None;
                for tick in 0..600 {
                    if tick % 40 == 0 {
                        callback.render(480, Some(tick * 250_000), |_, _| {});
                        if voice.is_some_and(|v| {
                            matches!(
                                output.voice_submission(0, v).unwrap(),
                                VoiceSubmission::Complete(_)
                            )
                        }) {
                            completed = Some(tick);
                            break;
                        }
                    }
                    if tick % 8 == owner_phase {
                        if tick >= arrival && voice.is_none() {
                            voice = runtime
                                .trigger(runtime.token(Channel::Scene), id, runtime.mix_frame())
                                .unwrap()
                                .voice;
                        }
                        producer.pump(&mut runtime).unwrap();
                    }
                }
                latencies.insert(completed.expect("virtual Voice never submitted") - arrival);
                assert_eq!(output.stats().empty_frames, 0);
                drop((producer, callback, output, runtime));
                assert_eq!(budget.usage().bytes, 0);
            }
        }
        assert!(
            latencies.len() > 20,
            "request phase must not collapse to one latency"
        );
        assert!(latencies.first().unwrap() < latencies.last().unwrap());
    }
    #[test]
    fn causal_capture_distinguishes_waiting_for_generation_and_native_submission() {
        use skin_editor_audio::Channel;
        let budget = PcmBudget::default();
        let (mut producer, mut callback) = output_boundary(&budget, 48000, 1).unwrap();
        let output = producer.control();
        output.capture_submissions().unwrap();
        let mut runtime = Runtime::with_budget(48000, 1, budget.clone()).unwrap();
        let id = ready_tone(&mut runtime, 4096);
        producer.prefill(&mut runtime).unwrap();
        let voice = runtime
            .trigger(runtime.token(Channel::Audition), id, runtime.mix_frame())
            .unwrap()
            .voice
            .unwrap();
        assert_eq!(producer.pump(&mut runtime).unwrap(), 0); // no room yet
        assert!(output.submission_trace(1, voice).is_none());
        callback.render(1024, Some(1_000_000), |_, _| {});
        producer.pump(&mut runtime).unwrap();
        // Callback is the same consumer; supplied fake stamp, no timing sleeps.
        callback.render(2048, Some(20_000_000), |_, _| {});
        let trace = output.submission_trace(1, voice).unwrap();
        assert_eq!(trace.producer[0], voice);
        assert!(trace.producer[3] <= trace.producer[4] && trace.producer[4] <= trace.producer[5]);
        assert_eq!(trace.callback[3], 20_000_000);
        assert_eq!(trace.callback[4], trace.producer[8]); // idle trimming is visible, not guessed as latency
        assert_eq!(trace.callback[5], 2048);
        assert!(output.submission_trace(0, voice).is_none());
        assert!(output.capture_submissions().is_err());
        drop((trace, output, callback, producer, runtime));
        assert_eq!(budget.usage().bytes, 0);
    }
    #[test]
    fn invalidation_cannot_attribute_old_copy_frames_to_captured_submission() {
        use skin_editor_audio::Channel;
        let budget = PcmBudget::default();
        let (mut producer, mut callback) = output_boundary(&budget, 48000, 1).unwrap();
        let output = producer.control();
        output.capture_submissions().unwrap();
        let mut runtime = Runtime::with_budget(48000, 1, budget).unwrap();
        let id = ready_tone(&mut runtime, 4096);
        let voice = runtime
            .trigger(runtime.token(Channel::Audition), id, 0)
            .unwrap()
            .voice
            .unwrap();
        producer.pump(&mut runtime).unwrap();
        output.invalidate_skin(false, true).unwrap();
        callback.render(768, Some(7), |_, value| assert_eq!(value, 0.));
        assert!(output.submission_trace(1, voice).is_none());
        assert_eq!(output.stats().submitted_voice_ids, [0, 0]);
    }
    #[test]
    fn empty_callback_trace_is_bounded_and_does_not_require_startup_capture() {
        let budget = PcmBudget::default();
        let (producer, mut callback) = output_boundary(&budget, 48000, 1).unwrap();
        let output = producer.control();
        for _ in 0..40 {
            callback.render_f32(&mut [0.; 480]);
        }
        let rows = output.underrun_observations();
        assert_eq!(rows.len(), 32);
        assert_eq!(rows[0], [0, 480, 0, 480, 0, 0]);
        assert_eq!(rows[31], [31, 480, 0, 480, 0, 0]);
        assert_eq!(output.stats().empty_callbacks, 40); // overflow remains visible
        assert_eq!(output.stats().empty_frames, 40 * 480);
        drop((callback, producer, output));
        assert_eq!(budget.usage().bytes, 0);
    }
    #[test]
    fn native_voice_observation_identifies_a_new_contribution_among_old_voices() {
        use skin_editor_audio::Channel;
        let budget = PcmBudget::default();
        let (mut producer, mut callback) = output_boundary(&budget, 48000, 1).unwrap();
        let mut runtime = Runtime::with_budget(48000, 1, budget).unwrap();
        let id = ready_tone(&mut runtime, 4096);
        let one = runtime
            .trigger(runtime.token(Channel::Audition), id, runtime.mix_frame())
            .unwrap()
            .voice
            .unwrap();
        producer.pump(&mut runtime).unwrap();
        callback.render_f32(&mut [0.; 128]);
        assert_eq!(producer.stats().submitted_voice_ids, [0, one]);
        let two = runtime
            .trigger(runtime.token(Channel::Audition), id, runtime.mix_frame())
            .unwrap()
            .voice
            .unwrap();
        assert!(two > one);
        producer.pump(&mut runtime).unwrap();
        callback.render_f32(&mut [0.; OUTPUT_AHEAD_FRAMES - 128]);
        assert_eq!(producer.stats().submitted_voice_ids, [0, one]);
        callback.render_f32(&mut [0.; 128]);
        assert_eq!(producer.stats().submitted_voice_ids, [0, two]);
    }
    #[test]
    fn silent_and_invalidated_frames_cannot_publish_voice_observations() {
        let budget = PcmBudget::default();
        let (mut producer, mut callback) = output_boundary(&budget, 48000, 1).unwrap();
        let frame = Frame {
            scene: [0.25, 0.],
            epochs: [1, 1],
            voices: [3, 0],
            background_position: u64::MAX,
            ..Frame::default()
        };
        producer.producer.push(frame).unwrap();
        callback.render_f32(&mut [0.; 1]);
        assert_eq!(producer.stats().submitted_voice_ids, [3, 0]);
        producer.control().invalidate_skin(true, false).unwrap();
        producer
            .producer
            .push(Frame {
                voices: [8, 0],
                ..frame
            })
            .unwrap();
        producer
            .producer
            .push(Frame {
                scene: [0., 0.],
                epochs: [2, 1],
                voices: [9, 0],
                ..frame
            })
            .unwrap();
        callback.render_f32(&mut [0.; 2]);
        assert_eq!(producer.stats().submitted_voice_ids, [3, 0]);
    }
    #[test]
    fn malformed_marker_blocks_do_not_advance_the_mixer() {
        let budget = PcmBudget::default();
        let (mut producer, _) = output_boundary(&budget, 48000, 1).unwrap();
        let mut runtime = Runtime::with_budget(48000, 1, budget).unwrap();
        assert_eq!(
            runtime.mix_split_buffers_marked(
                &mut producer.scene,
                &mut producer.audition,
                &mut [[0; 2]; 127]
            ),
            Err(skin_editor_audio::RuntimeError::InvalidBlock)
        );
        assert_eq!(runtime.mix_frame(), 0);
    }
    #[test]
    fn replay_of_observed_initial_callbacks_changes_192_empty_frames_to_zero() {
        for configured in [false, true] {
            let budget = PcmBudget::default();
            let (mut producer, mut callback) = output_boundary(&budget, 48000, 1).unwrap();
            if configured {
                producer.configure_period(480).unwrap();
            }
            let mut runtime = Runtime::with_budget(48000, 1, budget).unwrap();
            producer.prefill(&mut runtime).unwrap();
            callback.render_f32(&mut [0.0; 1056]);
            producer
                .pump_to(
                    &mut runtime,
                    if configured { OUTPUT_AHEAD_FRAMES } else { 768 },
                )
                .unwrap();
            callback.render_f32(&mut [0.0; 480]);
            producer
                .pump_to(
                    &mut runtime,
                    if configured { OUTPUT_AHEAD_FRAMES } else { 768 },
                )
                .unwrap();
            callback.render_f32(&mut [0.0; 480]);
            callback.render_f32(&mut [0.0; 480]);
            assert_eq!(
                producer.stats().empty_frames,
                if configured { 0 } else { 192 }
            );
        }
    }
    #[test]
    fn first_trigger_waits_for_space_without_consuming_voice_or_discarding_startup_prefill() {
        use skin_editor_audio::Channel;
        let budget = PcmBudget::default();
        let (mut producer, mut callback) = output_boundary(&budget, 48000, 1).unwrap();
        producer.configure_period(480).unwrap();
        let mut runtime = Runtime::with_budget(48000, 1, budget).unwrap();
        let id = ready_tone(&mut runtime, 4096);
        producer.prefill(&mut runtime).unwrap();
        runtime
            .trigger(runtime.token(Channel::Audition), id, runtime.mix_frame())
            .unwrap();
        assert_eq!(producer.pump(&mut runtime).unwrap(), 0);
        assert_eq!(runtime.voices().next().unwrap().cursor, 0);
        assert_eq!(producer.queued_frames(), 2048);
        callback.render_f32(&mut [0.0; 1056]);
        producer.pump(&mut runtime).unwrap();
        callback.render_f32(&mut [0.0; 480]);
        callback.render_f32(&mut [0.0; 480]);
        assert_eq!(producer.stats().empty_frames, 0);
        assert!(producer.stats().lane_nonzero_frames[1] > 0);
    }
    #[test]
    fn cancellation_during_idle_cutover_cannot_publish_the_old_voice() {
        use skin_editor_audio::Channel;
        let budget = PcmBudget::default();
        let (mut producer, mut callback) = output_boundary(&budget, 48000, 1).unwrap();
        producer.configure_period(480).unwrap();
        let mut runtime = Runtime::with_budget(48000, 1, budget).unwrap();
        let id = ready_tone(&mut runtime, 4096);
        producer.pump(&mut runtime).unwrap();
        runtime
            .trigger(runtime.token(Channel::Audition), id, runtime.mix_frame())
            .unwrap();
        producer.pump(&mut runtime).unwrap();
        runtime
            .stop_audition(runtime.token(Channel::Audition))
            .unwrap();
        let fence = producer.invalidate(false, true).unwrap();
        let mut out = [9.0; 480];
        callback.render_f32(&mut out);
        assert_eq!(out, [0.0; 480]);
        assert!(producer.acknowledged(fence));
        assert_eq!(producer.stats().lane_nonzero_frames, [0, 0]);
        assert_eq!(runtime.voices().count(), 0);
    }
    #[test]
    fn startup_capture_is_one_fixed_window_and_cannot_be_armed_late() {
        let budget = PcmBudget::default();
        let (mut producer, mut callback) = output_boundary(&budget, 48000, 1).unwrap();
        producer.control().capture_startup().unwrap();
        let mut runtime = Runtime::with_budget(48000, 1, budget).unwrap();
        producer.prefill(&mut runtime).unwrap();
        for _ in 0..40 {
            callback.render_f32(&mut [0.0; 1]);
        }
        let rows = producer.control().startup_observations();
        assert_eq!(rows.len(), STARTUP_OBSERVATIONS);
        assert_eq!(rows[31], [1, 2017, 0, 1]);
        assert!(producer.control().capture_startup().is_err());
    }
    fn ready_tone(
        runtime: &mut Runtime,
        frames: u64,
    ) -> skin_editor_application::values::AudioComponentId {
        use skin_editor_application::values::*;
        use skin_editor_audio::{Origin, PreparedSound, SoundPlan};
        let component = AudioComponentId::new(1).unwrap();
        let plan = SoundPlan {
            target: component.into(),
            source: BlobRef::new([1; 32], frames * 2),
            metadata: Some(AudioMetadata::new(48000, 1, frames).unwrap()),
            gain: Gain::UNITY,
            visible: true,
        };
        let job = runtime
            .synchronize(
                Origin {
                    session: 1,
                    node: 1,
                    revision: 0,
                    navigation_revision: 0,
                    draft: None,
                },
                &[plan],
            )
            .unwrap()[0];
        let mut pcm = runtime
            .pcm_budget()
            .reserve(frames as usize)
            .unwrap()
            .allocate();
        pcm.samples_mut().unwrap().fill(0.25);
        runtime
            .complete(job, Ok(PreparedSound::from_decoded(plan, pcm).unwrap()))
            .unwrap();
        component
    }
    #[test]
    fn configured_idle_reserve_survives_the_same_two_callback_burst() {
        let budget = PcmBudget::default();
        let (mut producer, mut callback) = output_boundary(&budget, 48000, 1).unwrap();
        producer.configure_period(480).unwrap();
        let mut runtime = Runtime::with_budget(48000, 1, budget).unwrap();
        producer.pump(&mut runtime).unwrap();
        assert_eq!(producer.queued_frames(), 1024);
        callback.render_f32(&mut [0.0; 480]);
        callback.render_f32(&mut [0.0; 480]);
        assert_eq!(producer.stats().empty_frames, 0);
    }
    #[test]
    fn idle_exit_keeps_burst_guard_but_retimes_only_obsolete_blank_frames() {
        use skin_editor_audio::Channel;
        let budget = PcmBudget::default();
        let (mut producer, mut callback) = output_boundary(&budget, 48000, 1).unwrap();
        producer.configure_period(480).unwrap();
        let mut runtime = Runtime::with_budget(48000, 1, budget).unwrap();
        let id = ready_tone(&mut runtime, 4096);
        producer.pump(&mut runtime).unwrap();
        runtime
            .trigger(runtime.token(Channel::Audition), id, runtime.mix_frame())
            .unwrap();
        producer.pump(&mut runtime).unwrap();
        let mut out = [0.0; 480];
        callback.render_f32(&mut out);
        assert_eq!(out[..192], [0.0; 192]);
        assert_eq!(out[192..], [0.25; 288]);
        callback.render_f32(&mut out);
        assert_eq!(out, [0.25; 480]);
        assert_eq!(producer.stats().empty_frames, 0);
        assert_eq!(producer.stats().retimed_idle_frames, 832);
    }
    #[test]
    fn zero_valued_valid_background_time_is_never_retimed() {
        use skin_editor_audio::Channel;
        let budget = PcmBudget::default();
        let (mut producer, mut callback) = output_boundary(&budget, 48000, 1).unwrap();
        producer.configure_period(480).unwrap();
        let mut runtime = Runtime::with_budget(48000, 1, budget.clone()).unwrap();
        let id = ready_tone(&mut runtime, 4096);
        let pcm = budget.reserve(4096).unwrap().allocate();
        let mut background = BackgroundTrack::new(pcm, 48000, 1).unwrap();
        background.set_running(true);
        let mark = producer.control().mark();
        producer
            .pump_timeline(
                &mut runtime,
                Some(&mut background),
                mark,
                OUTPUT_AHEAD_FRAMES,
            )
            .unwrap();
        background.set_running(false); // Deliberately keep queued background's generation valid.
        producer
            .pump_timeline(
                &mut runtime,
                Some(&mut background),
                mark,
                OUTPUT_AHEAD_FRAMES,
            )
            .unwrap();
        runtime
            .trigger(runtime.token(Channel::Audition), id, runtime.mix_frame())
            .unwrap();
        producer
            .pump_timeline(
                &mut runtime,
                Some(&mut background),
                mark,
                OUTPUT_AHEAD_FRAMES,
            )
            .unwrap();
        let mut out = [0.0; 480];
        callback.render_f32(&mut out);
        assert_eq!(out, [0.0; 480]);
        callback.render_f32(&mut out);
        assert_eq!(out[..288], [0.0; 288]);
        assert_eq!(out[288..], [0.25; 192]);
        assert_eq!(producer.stats().background_next, 768);
        assert_eq!(producer.stats().background_gaps, 0);
        assert_eq!(producer.stats().retimed_idle_frames, 256);
    }
    #[test]
    fn pending_event_keeps_its_scheduled_time_and_prevents_idle_reserve() {
        use skin_editor_audio::Channel;
        let budget = PcmBudget::default();
        let (mut producer, _) = output_boundary(&budget, 48000, 1).unwrap();
        producer.configure_period(480).unwrap();
        let mut runtime = Runtime::with_budget(48000, 1, budget).unwrap();
        let id = ready_tone(&mut runtime, 4096);
        runtime
            .trigger(runtime.token(Channel::Audition), id, 2048)
            .unwrap();
        assert_eq!(runtime.output_activity(), [false, true]);
        producer.pump(&mut runtime).unwrap();
        assert_eq!(producer.queued_frames(), OUTPUT_AHEAD_FRAMES);
        assert_eq!(producer.stats().retimed_idle_frames, 0);
    }
    #[test]
    fn replacement_batch_is_invisible_until_committed_and_cutover_published() {
        let budget = PcmBudget::default();
        let (mut producer, mut callback) = output_boundary(&budget, 48000, 1).unwrap();
        producer.configure_period(480).unwrap();
        let mut runtime = Runtime::with_budget(48000, 1, budget).unwrap();
        producer.pump(&mut runtime).unwrap();
        let mut chunk = producer.producer.write_chunk(OUTPUT_AHEAD_FRAMES).unwrap();
        let (one, two) = chunk.as_mut_slices();
        for (i, slot) in one.iter_mut().chain(two).enumerate() {
            *slot = Frame {
                audition: [0.25, 0.0],
                epochs: [1; 2],
                serial: 3000 + i as u64,
                activity: 2,
                background_position: u64::MAX,
                ..Frame::default()
            };
        }
        let mut out = [0.0; 480];
        callback.render_f32(&mut out); // Controlled boundary, before publication.
        assert_eq!(out, [0.0; 480]);
        chunk.commit_all();
        producer.shared.idle_cutover.store(3000, Ordering::Release);
        callback.render_f32(&mut out);
        assert_eq!(out[..192], [0.0; 192]);
        assert_eq!(out[192..], [0.25; 288]);
        assert_eq!(producer.stats().empty_frames, 0);
    }
    #[test]
    fn unsupported_or_late_period_and_exhausted_serial_do_not_advance_mix() {
        let budget = PcmBudget::default();
        let (mut producer, _) = output_boundary(&budget, 48000, 1).unwrap();
        assert!(producer.configure_period(0).is_err());
        assert!(producer.configure_period(1024).is_err());
        assert!(producer.configure_period(640).is_err());
        assert!(producer.configure_period(usize::MAX / 2).is_err());
        producer.configure_period(480).unwrap();
        let mut runtime = Runtime::with_budget(48000, 1, budget).unwrap();
        producer.next_serial = u64::MAX - 64;
        assert!(producer.pump(&mut runtime).is_err());
        assert_eq!(runtime.mix_frame(), 0);
        assert_eq!(producer.queued_frames(), 0);
        assert!(producer.configure_period(480).is_err());
    }
    #[test]
    fn two_adjacent_480_frame_callbacks_reproduce_192_without_supply_between_them() {
        let budget = PcmBudget::default();
        let (mut producer, mut callback) = output_boundary(&budget, 48000, 1).unwrap();
        let mut runtime = Runtime::with_budget(48000, 1, budget).unwrap();
        producer.control().capture_startup().unwrap();
        producer.pump_to(&mut runtime, 768).unwrap();
        callback.render_f32(&mut [0.0; 480]);
        callback.render_f32(&mut [0.0; 480]);
        assert_eq!(
            producer.control().startup_observations(),
            vec![[480, 768, 0, 1], [480, 288, 192, 1]]
        );
        assert_eq!(producer.stats().empty_frames, 192);
    }
    #[test]
    fn same_callbacks_with_supply_interleaved_have_no_empty_frames() {
        let budget = PcmBudget::default();
        let (mut producer, mut callback) = output_boundary(&budget, 48000, 1).unwrap();
        let mut runtime = Runtime::with_budget(48000, 1, budget).unwrap();
        producer.pump(&mut runtime).unwrap();
        callback.render_f32(&mut [0.0; 480]);
        producer.pump(&mut runtime).unwrap();
        callback.render_f32(&mut [0.0; 480]);
        assert_eq!(producer.stats().empty_frames, 0);
    }
    #[test]
    fn empty_callback_size_is_separate_from_frame_count_and_larger_valid_callback() {
        let budget = PcmBudget::default();
        let (mut producer, mut consumer) = output_boundary(&budget, 48000, 1).unwrap();
        let mut runtime = Runtime::with_budget(48000, 1, budget).unwrap();
        consumer.render_f32(&mut [0.0; 10]);
        producer.prefill(&mut runtime).unwrap();
        consumer.render_f32(&mut [0.0; 128]);
        let stats = producer.stats();
        assert_eq!(stats.empty_frames, 10);
        assert_eq!(stats.empty_callbacks, 1);
        assert_eq!(stats.max_empty_callback_frames, 10);
        assert_eq!(stats.max_callback_frames, 128);
    }
    #[test]
    fn integer_callback_quantizes_and_saturates_at_the_output_boundary() {
        let budget = PcmBudget::default();
        let (mut producer, mut consumer) = output_boundary(&budget, 48000, 1).unwrap();
        for value in [-1.5, -1.0, -0.5, 0.0, 0.5, 1.0, 1.5] {
            assert!(
                producer
                    .producer
                    .push(Frame {
                        scene: [value, 0.0],
                        audition: [0.0; 2],
                        epochs: [1; 2],
                        background: [0.0; 2],
                        background_epoch: 1,
                        background_position: u64::MAX,
                        ..Frame::default()
                    })
                    .is_ok()
            );
        }
        let mut output = [0i16; 7];
        consumer.render_i16(&mut output);
        assert_eq!(output, [-32768, -32768, -16384, 0, 16384, 32767, 32767]);
        drop(producer);
        drop(consumer);
        assert_eq!(budget.usage().bytes, 0);
    }
    #[test]
    fn monitor_volume_applies_to_queued_sum_before_clipping_without_resetting_background() {
        let budget = PcmBudget::default();
        let (mut producer, mut consumer) = output_boundary(&budget, 48000, 1).unwrap();
        for position in 0..4 {
            producer
                .producer
                .push(Frame {
                    scene: [0.75, 0.0],
                    audition: [0.75, 0.0],
                    epochs: [1; 2],
                    background: [0.5, 0.0],
                    background_epoch: 1,
                    background_position: position,
                    ..Frame::default()
                })
                .ok()
                .unwrap();
        }
        let control = producer.control();
        control.set_monitor_volume(250).unwrap();
        let mut output = [9.0; 1];
        consumer.render_f32(&mut output);
        assert_eq!(output, [0.5]);
        assert!(control.set_monitor_volume(1001).is_err());
        consumer.render_f32(&mut output);
        assert_eq!(output, [0.5]);
        control.set_monitor_volume(0).unwrap();
        consumer.render_f32(&mut output);
        assert_eq!(output, [0.0]);
        assert_eq!(control.stats().background_next, 3);
        control.set_monitor_volume(1000).unwrap();
        consumer.render_f32(&mut output);
        assert_eq!(output, [1.0]);
        drop(control);
        drop(producer);
        drop(consumer);
        assert_eq!(budget.usage().bytes, 0);
    }
    #[test]
    fn native_lane_observations_reject_stale_frames_and_confirm_counters_before_epoch_ack() {
        let budget = PcmBudget::default();
        let (mut producer, mut callback) = output_boundary(&budget, 48000, 1).unwrap();
        let control = producer.control();
        control.set_monitor_volume(0).unwrap();
        let frame = Frame {
            scene: [0.25, 0.0],
            audition: [0.5, 0.0],
            epochs: [1; 2],
            background: [0.0; 2],
            background_epoch: 1,
            background_position: u64::MAX,
            ..Frame::default()
        };
        producer.producer.push(frame).ok().unwrap();
        callback.render_f32(&mut [1.0; 1]);
        assert_eq!(control.stats().lane_nonzero_frames, [1, 1]);
        assert_eq!(control.stats().nonzero_samples, 0);
        control.invalidate_skin(true, false).unwrap();
        producer.producer.push(frame).ok().unwrap();
        callback.render_f32(&mut [1.0; 1]);
        let stats = control.stats();
        assert_eq!(stats.lane_nonzero_frames, [1, 2]);
        assert_eq!(stats.lane_acknowledged, stats.lane_epochs);
        control.close();
        producer.producer.push(frame).ok().unwrap();
        callback.render_f32(&mut [1.0; 1]);
        assert_eq!(control.stats().lane_nonzero_frames, [1, 2]);
        drop(control);
        drop(producer);
        drop(callback);
        assert_eq!(budget.usage().bytes, 0);
    }
}
