//! Independent Copy-only background reserve plus the existing Voice boundary.
//! No GUI clock, project/session, decoder, OS stream or stable rules here.
use crate::background_input::BackgroundInput;
use crate::voice_handoff::{
    self, HandoffError, HandoffReport, RuntimePumpReport, VoiceConsumer, VoiceOwner,
};
use rtrb::{Consumer, Producer, RingBuffer};
use skin_editor_audio::{
    BackgroundTrack, Channel, PcmBuffer, PcmReservation, Runtime, VoiceRuntimeKey,
};
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering::SeqCst},
};
mod control;
pub(crate) use control::Control;

pub const BACKGROUND_RESERVE: usize = 2048;
const RUNNING: u64 = 1 << 62;
const CLOSED: u64 = 1 << 63;
const EPOCH_MAX: u64 = RUNNING - 1;
fn epoch(word: u64) -> u64 {
    word & EPOCH_MAX
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutputError {
    Voice(HandoffError),
    Runtime,
    WrongRuntime,
    WrongFormat,
    WrongPool,
    ResourceLimit,
    Closed,
    Busy,
    AwaitingAck,
    NotPrimed,
    InvalidPosition,
    InvalidBlock,
    Protocol,
    BackgroundFault,
}
impl From<HandoffError> for OutputError {
    fn from(e: HandoffError) -> Self {
        Self::Voice(e)
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Phase {
    Frozen,
    Freezing,
    Running,
    Closed,
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CopyClock {
    pub serial: u64,
    pub epoch: u64,
    pub running: bool,
    pub next_background: u64,
    pub next_output: u64,
    pub missing_total: u64,
}
struct Clock {
    stamp: AtomicU64,
    epoch: AtomicU64,
    running: AtomicU64,
    background: AtomicU64,
    output: AtomicU64,
    missing: AtomicU64,
}
impl Clock {
    fn new(position: u64) -> Self {
        Self {
            stamp: AtomicU64::new(0),
            epoch: AtomicU64::new(1),
            running: AtomicU64::new(0),
            background: AtomicU64::new(position),
            output: AtomicU64::new(0),
            missing: AtomicU64::new(0),
        }
    }
    fn read(&self) -> Option<CopyClock> {
        let a = self.stamp.load(SeqCst);
        if a & 1 != 0 {
            return None;
        }
        let snapshot = CopyClock {
            serial: a,
            epoch: self.epoch.load(SeqCst),
            running: self.running.load(SeqCst) != 0,
            next_background: self.background.load(SeqCst),
            next_output: self.output.load(SeqCst),
            missing_total: self.missing.load(SeqCst),
        };
        (a == self.stamp.load(SeqCst)).then_some(snapshot)
    }
    // Single consumer writer. All fields are atomic; bounded reader rejects an
    // in-progress/torn row without waiting or a seqlock retry loop.
    fn publish(&self, value: CopyClock) {
        self.stamp.store(value.serial - 1, SeqCst);
        self.epoch.store(value.epoch, SeqCst);
        self.running.store(u64::from(value.running), SeqCst);
        self.background.store(value.next_background, SeqCst);
        self.output.store(value.next_output, SeqCst);
        self.missing.store(value.missing_total, SeqCst);
        self.stamp.store(value.serial, SeqCst);
    }
}
struct Shared {
    control: Arc<control::State>,
    gate: AtomicU64,
    resume_at: AtomicU64,
    resume_epoch: AtomicU64,
    clock: Clock,
    // Counts Copy ring sample payload until the queue is really destroyed.
    payload: Option<PcmReservation>,
}
#[derive(Clone, Copy)]
struct Frame {
    epoch: u64,
    position: u64,
    samples: [f32; 2],
}
pub struct BackgroundOwner {
    voices: VoiceOwner,
    runtime: VoiceRuntimeKey,
    shared: Arc<Shared>,
    tx: Option<Producer<Frame>>,
    track: Option<BackgroundInput>,
    phase: Phase,
    generation: u64,
    anchor: u64,
    target: Option<u64>,
    last_clock: CopyClock,
}
pub struct OutputConsumer {
    voices: Option<Box<VoiceConsumer>>,
    shared: Arc<Shared>,
    rx: Consumer<Frame>,
    scene: PcmBuffer,
    audition: PcmBuffer,
    background: PcmBuffer,
    channels: usize,
    source_frames: u64,
    generation: u64,
    running: bool,
    started_epoch: u64,
    next_background: u64,
    next_output: u64,
    serial: u64,
    missing: u64,
}
pub struct StoppedOutput(Box<OutputConsumer>);
impl StoppedOutput {
    /// Final callback copy anchor, after the backend returned this endpoint.
    /// This is not a device-consumption or acoustic position.
    pub fn copy_clock(&self) -> Option<CopyClock> {
        self.0.copy_clock()
    }
}
impl std::fmt::Debug for StoppedOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StoppedOutput").finish_non_exhaustive()
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Position {
    pub phase: Phase,
    pub epoch: u64,
    pub timeline_frame: u64,
    pub generated_frame: u64,
    pub copy_clock: CopyClock,
    pub queued: usize,
}
#[derive(Clone, Copy, Debug, Default)]
pub struct PumpReport {
    pub voices: RuntimePumpReport,
    pub freeze_completed: bool,
}
#[derive(Clone, Copy, Debug, Default)]
pub struct OutputReport {
    pub voices: HandoffReport,
    pub background_frames: usize,
    pub missing_frames: usize,
    pub clock: CopyClock,
}

/// Build a frozen, unprimed candidate. Reserve all managed sample storage before
/// allocation; the reference PCM and ring/scratch share Runtime's PCM budget.
pub fn boundary(
    runtime: &mut Runtime,
    track: BackgroundTrack,
    source: skin_editor_audio::Origin,
) -> Result<(BackgroundOwner, Box<OutputConsumer>), OutputError> {
    boundary_input(runtime, track.into(), source)
}
pub(crate) fn boundary_input(
    runtime: &mut Runtime,
    track: BackgroundInput,
    source: skin_editor_audio::Origin,
) -> Result<(BackgroundOwner, Box<OutputConsumer>), OutputError> {
    let key = runtime.voice_runtime_key().ok_or(OutputError::Runtime)?;
    if runtime.is_closed() {
        return Err(OutputError::Closed);
    }
    if runtime.scene_running() || runtime.voices().next().is_some() {
        return Err(OutputError::Busy);
    }
    let (rate, channels) = runtime.output_format();
    if track.format() != (rate, channels) {
        return Err(OutputError::WrongFormat);
    }
    let budget = runtime.pcm_budget();
    if !budget.same_pool(track.budget()) {
        return Err(OutputError::WrongPool);
    }
    if runtime.mix_frame() != 0 {
        return Err(OutputError::Runtime);
    }
    let payload = budget
        .reserve(BACKGROUND_RESERVE * 2)
        .map_err(|_| OutputError::ResourceLimit)?;
    let samples = BACKGROUND_RESERVE * usize::from(channels);
    let a = budget
        .reserve(samples)
        .map_err(|_| OutputError::ResourceLimit)?;
    let b = budget
        .reserve(samples)
        .map_err(|_| OutputError::ResourceLimit)?;
    let c = budget
        .reserve(samples)
        .map_err(|_| OutputError::ResourceLimit)?;
    let (mut voices, consumer) = voice_handoff::boundary(budget, rate, channels, source)?;
    voices.pump_runtime(runtime)?;
    let (tx, rx) = RingBuffer::new(BACKGROUND_RESERVE);
    let anchor = track.position();
    let frames = track.frames();
    let shared = Arc::new(Shared {
        control: control::State::new(anchor),
        gate: AtomicU64::new(1),
        resume_at: AtomicU64::new(anchor),
        resume_epoch: AtomicU64::new(0),
        clock: Clock::new(anchor),
        payload: Some(payload),
    });
    let last_clock = shared.clock.read().expect("initial clock");
    Ok((
        BackgroundOwner {
            voices,
            runtime: key,
            shared: shared.clone(),
            tx: Some(tx),
            track: Some(track),
            phase: Phase::Frozen,
            generation: 1,
            anchor,
            target: None,
            last_clock,
        },
        Box::new(OutputConsumer {
            voices: Some(consumer),
            shared,
            rx,
            scene: a.allocate(),
            audition: b.allocate(),
            background: c.allocate(),
            channels: usize::from(channels),
            source_frames: frames,
            generation: 1,
            running: false,
            started_epoch: 0,
            next_background: anchor,
            next_output: 0,
            serial: 0,
            missing: 0,
        }),
    ))
}
impl BackgroundOwner {
    pub(crate) fn retained_drafts(&self) -> impl Iterator<Item = u64> + '_ {
        self.voices.retained_drafts()
    }
    pub(crate) fn control(&self) -> Control {
        Control(self.shared.control.clone())
    }
    pub(crate) fn apply_mark(&self, mark: crate::output_fence::OutputMark) {
        self.shared.control.apply(mark);
    }
    fn validate(&self, runtime: &Runtime) -> Result<(), OutputError> {
        if runtime
            .voice_runtime_key()
            .as_ref()
            .is_none_or(|key| !self.runtime.same_runtime(key))
        {
            return Err(OutputError::WrongRuntime);
        }
        if runtime.is_closed() {
            self.shared.gate.fetch_or(CLOSED, SeqCst);
            return Err(OutputError::Closed);
        }
        if self.shared.gate.load(SeqCst) & CLOSED != 0 || self.shared.control.closed.load(SeqCst) {
            return Err(OutputError::Closed);
        }
        Ok(())
    }
    pub fn position(&self) -> Position {
        Position {
            phase: self.phase,
            epoch: self.generation,
            timeline_frame: if self.phase == Phase::Running && self.last_clock.running {
                self.last_clock.next_background
            } else {
                self.anchor
            },
            generated_frame: self
                .track
                .as_ref()
                .map_or(self.anchor, BackgroundInput::position),
            copy_clock: self.last_clock,
            queued: self
                .tx
                .as_ref()
                .map_or(0, |tx| BACKGROUND_RESERVE - tx.slots()),
        }
    }
    pub fn pump(&mut self, runtime: &mut Runtime) -> Result<PumpReport, OutputError> {
        self.validate(runtime)?;
        let mut freeze_completed = false;
        if let Some(clock) = self
            .shared
            .clock
            .read()
            .filter(|c| c.epoch == self.generation)
        {
            self.last_clock = clock;
            if self.phase == Phase::Freezing && !clock.running {
                self.anchor = self.target.take().unwrap_or(clock.next_background);
                self.track
                    .as_mut()
                    .expect("open track")
                    .restore_position(self.anchor)?;
                runtime
                    .seek_scene(runtime.token(Channel::Scene), self.anchor)
                    .map_err(|_| OutputError::Runtime)?;
                self.phase = Phase::Frozen;
                freeze_completed = true;
            }
            let scene = if self.phase == Phase::Running && clock.running {
                clock.next_background
            } else {
                self.anchor
            };
            runtime
                .synchronize_handoff_clock(clock.next_output, scene)
                .map_err(|_| OutputError::Runtime)?;
        }
        if runtime.scene_running() != (self.phase == Phase::Running) {
            return Err(OutputError::Protocol);
        }
        Ok(PumpReport {
            voices: self.voices.pump_runtime(runtime)?,
            freeze_completed,
        })
    }
    /// Can run before or after new Voice admission; only background enters this
    /// queue, so a full reserve cannot put a new short sound behind 2048 frames.
    pub fn refill(&mut self) -> Result<usize, OutputError> {
        self.refill_inner(|| {})
    }
    fn refill_inner(&mut self, mut published: impl FnMut()) -> Result<usize, OutputError> {
        if self.shared.gate.load(SeqCst) & CLOSED != 0 {
            return Err(OutputError::Closed);
        }
        if self.phase == Phase::Freezing {
            return Err(OutputError::AwaitingAck);
        }
        let track = self.track.as_mut().expect("open track");
        let tx = self.tx.as_mut().expect("open queue");
        if tx.is_abandoned() {
            self.close();
            return Err(OutputError::Closed);
        }
        track.set_running(true);
        let mut count = 0;
        while count < BACKGROUND_RESERVE && tx.slots() > 0 {
            let Some((position, samples)) = track.next_frame()? else {
                break;
            };
            assert!(
                tx.push(Frame {
                    epoch: self.generation,
                    position,
                    samples
                })
                .is_ok()
            );
            count += 1;
            published();
        }
        track.set_running(false);
        Ok(count)
    }
    fn freeze(&mut self, runtime: &mut Runtime, target: Option<u64>) -> Result<(), OutputError> {
        self.validate(runtime)?;
        if target.is_none() && self.phase != Phase::Running {
            return Ok(());
        }
        if target.is_some_and(|t| t > self.track.as_ref().expect("open track").frames()) {
            return Err(OutputError::InvalidPosition);
        }
        let Some(generation) = self.generation.checked_add(1).filter(|e| *e <= EPOCH_MAX) else {
            self.close();
            return Err(OutputError::ResourceLimit);
        };
        runtime
            .set_scene_running(runtime.token(Channel::Scene), false)
            .map_err(|_| OutputError::Runtime)?;
        if let Err(e) = self.voices.set_scene_running(false) {
            self.close();
            return Err(e.into());
        }
        self.generation = generation;
        self.target = target;
        self.phase = Phase::Freezing;
        self.shared.gate.store(generation, SeqCst);
        Ok(())
    }
    pub fn pause(&mut self, runtime: &mut Runtime) -> Result<(), OutputError> {
        self.freeze(runtime, None)
    }
    /// Latest seek supersedes an unacknowledged one. Frozen by default; no old
    /// scene voices or elapsed wall-clock events are replayed on resume.
    pub fn seek(&mut self, runtime: &mut Runtime, frame: u64) -> Result<(), OutputError> {
        self.freeze(runtime, Some(frame))
    }
    pub fn resume(&mut self, runtime: &mut Runtime) -> Result<(), OutputError> {
        self.validate(runtime)?;
        self.pump(runtime)?;
        if self.phase == Phase::Running {
            return Ok(());
        }
        if self.phase == Phase::Freezing {
            return Err(OutputError::AwaitingAck);
        }
        let track = self.track.as_ref().expect("open track");
        let required = (track.frames() - self.anchor).min(BACKGROUND_RESERVE as u64);
        if track.position() - self.anchor < required {
            return Err(OutputError::NotPrimed);
        }
        runtime
            .set_scene_running(runtime.token(Channel::Scene), true)
            .map_err(|_| OutputError::Runtime)?;
        if let Err(e) = self.voices.set_scene_running(true) {
            self.close();
            return Err(e.into());
        }
        self.shared.resume_at.store(self.anchor, SeqCst);
        self.shared.resume_epoch.store(self.generation, SeqCst);
        self.phase = Phase::Running;
        self.shared.gate.store(self.generation | RUNNING, SeqCst);
        Ok(())
    }
    pub(crate) fn is_primed(&self) -> bool {
        self.track.as_ref().is_some_and(|t| {
            t.position().saturating_sub(self.anchor)
                >= (t.frames() - self.anchor).min(BACKGROUND_RESERVE as u64)
        })
    }
    pub fn close(&mut self) {
        self.shared.control.closed.store(true, SeqCst);
        self.shared.gate.fetch_or(CLOSED, SeqCst);
        self.voices.close();
        self.phase = Phase::Closed;
    }
    /// A destroyed consumer endpoint is an executor fact, not an OS stop claim.
    /// Reclaim its orphaned queue/leases on the owner without another callback.
    pub fn reclaim_abandoned(&mut self, runtime: &mut Runtime) -> Result<(), OutputError> {
        if runtime
            .voice_runtime_key()
            .as_ref()
            .is_none_or(|key| !self.runtime.same_runtime(key))
        {
            return Err(OutputError::WrongRuntime);
        }
        if self.tx.as_ref().is_none_or(|tx| !tx.is_abandoned()) {
            return Err(OutputError::Busy);
        }
        self.close();
        match self.voices.pump_runtime(runtime) {
            Err(HandoffError::Closed) | Ok(_) => {}
            Err(e) => return Err(e.into()),
        }
        runtime.shutdown();
        self.track = None;
        self.tx = None;
        Arc::get_mut(&mut self.shared)
            .expect("consumer endpoint destroyed")
            .payload = None;
        Ok(())
    }
    pub fn finish(
        &mut self,
        runtime: &mut Runtime,
        mut stopped: StoppedOutput,
    ) -> Result<(), StoppedOutput> {
        if !Arc::ptr_eq(&self.shared, &stopped.0.shared)
            || runtime
                .voice_runtime_key()
                .as_ref()
                .is_none_or(|k| !self.runtime.same_runtime(k))
        {
            return Err(stopped);
        }
        self.close();
        self.voices
            .finish_runtime(
                runtime,
                stopped.0.voices.take().expect("unique endpoint").stop(),
            )
            .expect("matched runtime and endpoint");
        self.track = None;
        self.tx = None;
        drop(stopped);
        Arc::get_mut(&mut self.shared)
            .expect("only endpoints own Shared")
            .payload = None;
        Ok(())
    }
}
impl Drop for BackgroundOwner {
    fn drop(&mut self) {
        crate::callback_audit::released();
        self.close();
    }
}
impl Drop for OutputConsumer {
    fn drop(&mut self) {
        crate::callback_audit::released();
        self.shared.gate.fetch_or(CLOSED, SeqCst);
    }
}
#[derive(Clone, Copy)]
enum Checkpoint {
    AfterBackground,
    BeforeCopy,
    AfterCopy,
}
impl OutputConsumer {
    pub(crate) fn control(&self) -> Control {
        Control(self.shared.control.clone())
    }
    pub(crate) fn copy_clock(&self) -> Option<CopyClock> {
        self.shared.clock.read()
    }
    pub fn format(&self) -> (u32, u16) {
        self.voices.as_ref().expect("live endpoint").format()
    }
    pub fn stop(self: Box<Self>) -> StoppedOutput {
        self.shared.gate.fetch_or(CLOSED, SeqCst);
        StoppedOutput(self)
    }
    fn observe_freeze(&mut self, word: u64) {
        // Resume may have been published and paused again before a consumer
        // ever saw RUNNING. Preserve that explicit seek/resume anchor, rather
        // than acknowledging the source position from before the seek.
        let resumed = self.shared.resume_epoch.load(SeqCst);
        if !self.running && resumed == self.generation && resumed > self.started_epoch {
            self.next_background = self.shared.resume_at.load(SeqCst);
            self.started_epoch = resumed;
        }
        if epoch(word) != self.generation {
            for _ in 0..BACKGROUND_RESERVE {
                if self.rx.pop().is_err() {
                    break;
                }
            }
            self.generation = epoch(word);
        }
        self.running = false;
    }
    /// Software output-buffer copy boundary, not device consumption/acoustics.
    /// Invalid blocks are rejected before dequeue/cursor/output modification.
    pub fn render(&mut self, output: &mut [f32]) -> Result<OutputReport, OutputError> {
        self.render_inner(output, |_| {})
    }
    /// Explicit diagnostic only: wall work within our callback, not thread CPU
    /// or device consumption. Default render never reads a clock.
    pub(crate) fn render_profiled(
        &mut self,
        output: &mut [f32],
    ) -> Result<(OutputReport, [u64; 3]), OutputError> {
        let mut previous = std::time::Instant::now();
        let mut nanos = [0u64; 3];
        let report = self.render_inner(output, |stage| {
            let now = std::time::Instant::now();
            let index = match stage {
                Checkpoint::AfterBackground => 0,
                Checkpoint::BeforeCopy => 1,
                Checkpoint::AfterCopy => 2,
            };
            nanos[index] = now
                .duration_since(previous)
                .as_nanos()
                .min(u128::from(u64::MAX)) as u64;
            previous = now;
        })?;
        Ok((report, nanos))
    }
    fn render_inner(
        &mut self,
        output: &mut [f32],
        mut checkpoint: impl FnMut(Checkpoint),
    ) -> Result<OutputReport, OutputError> {
        if !output.len().is_multiple_of(self.channels)
            || output.len() / self.channels > BACKGROUND_RESERVE
        {
            return Err(OutputError::InvalidBlock);
        }
        let frames = output.len() / self.channels;
        let Some(next_output) = self.next_output.checked_add(frames as u64) else {
            self.shared.gate.fetch_or(CLOSED, SeqCst);
            return Err(OutputError::ResourceLimit);
        };
        let Some(serial) = self
            .serial
            .checked_add(2)
            .filter(|_| self.missing.checked_add(frames as u64).is_some())
        else {
            self.shared.gate.fetch_or(CLOSED, SeqCst);
            return Err(OutputError::ResourceLimit);
        };
        let word = self.shared.gate.load(SeqCst);
        let applied = self.shared.control.applied();
        let initial_desired = self.shared.control.desired();
        if word & RUNNING == 0 {
            self.observe_freeze(word);
        } else if epoch(word) != self.generation {
            self.shared.gate.fetch_or(CLOSED, SeqCst);
            return Err(OutputError::Protocol);
        } else if !self.running {
            self.next_background = self.shared.resume_at.load(SeqCst);
            self.started_epoch = self.generation;
            self.running = true;
        }
        let start = self.next_background;
        let background =
            &mut self.background.samples_mut().expect("unique scratch")[..output.len()];
        background.fill(0.0);
        let mut count = 0;
        let mut missing = 0;
        if word & (RUNNING | CLOSED) == RUNNING
            && applied.background == initial_desired.background
            && !self.shared.control.closed.load(SeqCst)
        {
            for target in background.chunks_exact_mut(self.channels) {
                if self.next_background >= self.source_frames {
                    break;
                }
                match self.rx.pop() {
                    Ok(frame)
                        if frame.epoch == self.generation
                            && frame.position == self.next_background =>
                    {
                        target.copy_from_slice(&frame.samples[..self.channels]);
                        self.next_background += 1;
                        count += 1;
                    }
                    Ok(_) => {
                        self.shared.gate.fetch_or(CLOSED, SeqCst);
                        return Err(OutputError::Protocol);
                    }
                    Err(_) => {
                        missing += 1;
                    }
                }
            }
        }
        checkpoint(Checkpoint::AfterBackground);
        let scene = &mut self.scene.samples_mut().expect("unique scratch")[..output.len()];
        let audition = &mut self.audition.samples_mut().expect("unique scratch")[..output.len()];
        let shared = &self.shared;
        let mut copied = false;
        let mut observed = initial_desired;
        let voices = self.voices.as_mut().expect("live endpoint").render_to(
            scene,
            audition,
            || checkpoint(Checkpoint::BeforeCopy),
            |scene, audition, report| {
                let final_word = shared.gate.load(SeqCst);
                observed = shared.control.desired();
                copied = word & (RUNNING | CLOSED) == RUNNING
                    && word == final_word
                    && applied.background == observed.background;
                let open = final_word & CLOSED == 0 && !shared.control.closed.load(SeqCst);
                copied &= open;
                let scene_valid = applied.skin.0[0] == observed.skin.0[0];
                let audition_valid = applied.skin.0[1] == observed.skin.0[1];
                if !open {
                    report.first.fill(None);
                    report.invalidated_lanes = [true; 2];
                }
                if !copied {
                    if word & RUNNING != 0 {
                        report.invalidated_lanes[0] = true;
                    }
                    for first in &mut report.first {
                        if first.is_some_and(|(_, lane)| lane == Channel::Scene) {
                            *first = None;
                        }
                    }
                }
                for (lane, valid) in [
                    (Channel::Scene, scene_valid),
                    (Channel::Audition, audition_valid),
                ] {
                    if !valid {
                        report.invalidated_lanes[usize::from(lane == Channel::Audition)] = true;
                        for first in &mut report.first {
                            if first.is_some_and(|(_, c)| c == lane) {
                                *first = None;
                            }
                        }
                    }
                }
                for i in 0..output.len() {
                    output[i] = if open {
                        ((if copied {
                            background[i] + if scene_valid { scene[i] } else { 0.0 }
                        } else {
                            0.0
                        }) + if audition_valid { audition[i] } else { 0.0 })
                        .clamp(-1.0, 1.0)
                    } else {
                        0.0
                    };
                }
            },
        )?;
        if !copied {
            self.next_background = start;
            count = 0;
            missing = 0;
        }
        self.next_output = next_output;
        checkpoint(Checkpoint::AfterCopy);
        let after = self.shared.gate.load(SeqCst);
        if after & RUNNING == 0 {
            self.observe_freeze(after);
        }
        self.serial = serial;
        self.missing = self
            .missing
            .checked_add(missing as u64)
            .ok_or(OutputError::ResourceLimit)?;
        let clock = CopyClock {
            serial,
            epoch: self.generation,
            running: self.running
                && after & CLOSED == 0
                && !self.shared.control.closed.load(SeqCst),
            next_background: self.next_background,
            next_output,
            missing_total: self.missing,
        };
        self.shared.clock.publish(clock);
        self.shared
            .control
            .acknowledge(observed, applied, self.next_background, clock.running);
        Ok(OutputReport {
            voices,
            background_frames: count,
            missing_frames: missing,
            clock,
        })
    }
}
#[cfg(test)]
mod tests;
