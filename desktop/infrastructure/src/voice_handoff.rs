//! Candidate SPSC ready-voice boundary, not a CPAL stream or project interface.
//! Background reserve and authoritative event scheduling stay outside this type.
//! Endpoint destruction/`stop`/`finish` must run outside the native callback.
use rtrb::{Consumer, Producer, PushError, RingBuffer};
use skin_editor_audio::{
    Channel, MAX_VOICES, MonitorFormat, Origin, PcmBudget, PcmBuffer, PreparedSound, Runtime,
    VoiceHandoff, VoiceRuntimeKey, VoiceValidity,
};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Weak};

pub const MAX_HANDOFF_FRAMES: usize = 2048;
const EPOCH_BITS: u32 = 30;
const EPOCH_MAX: u64 = (1 << EPOCH_BITS) - 1;
const RUNNING: u64 = 1 << 60;
const CLOSED: u64 = 1 << 61;
fn index(channel: Channel) -> usize {
    match channel {
        Channel::Scene => 0,
        Channel::Audition => 1,
    }
}
fn epoch(word: u64, channel: Channel) -> u64 {
    (word >> (index(channel) as u32 * EPOCH_BITS)) & EPOCH_MAX
}
fn permits(word: u64, token: &VoiceToken) -> bool {
    word & CLOSED == 0
        && epoch(word, token.channel) == token.epoch
        && (token.channel == Channel::Audition || word & RUNNING != 0)
}
fn valid_origin(origin: Origin) -> bool {
    origin.session != 0 && origin.node != 0
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HandoffError {
    InvalidOrigin,
    InvalidFormat,
    WrongPool,
    Stale,
    Frozen,
    Closed,
    Busy,
    Exhausted,
    InvalidBlock,
    RuntimeMode,
    ForeignRuntime,
}
#[derive(Clone)]
pub struct VoiceToken {
    boundary: Weak<Shared>,
    epoch: u64,
    channel: Channel,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VoiceKey {
    slot: usize,
    generation: u64,
}

struct Shared {
    gate: AtomicU64,
}
struct Packet {
    key: VoiceKey,
    token: VoiceToken,
    sound: PreparedSound,
    cursor: usize,
    started: bool,
    validity: Option<VoiceValidity>,
    runtime_voice: Option<u64>,
}
impl Drop for Packet {
    fn drop(&mut self) {
        crate::callback_audit::released();
    }
}
struct Pin {
    key: VoiceKey,
    pcm: PcmBuffer,
    runtime: Option<VoiceHandoff>,
}

pub struct VoiceOwner {
    shared: Arc<Shared>,
    pending: Option<Producer<Packet>>,
    returned: Consumer<Packet>,
    pins: [Option<Pin>; MAX_VOICES],
    budget: PcmBudget,
    format: (u32, u16),
    source: Origin,
    serial: u64,
    runtime: Option<VoiceRuntimeKey>,
}
pub struct VoiceConsumer {
    shared: Arc<Shared>,
    pending: Consumer<Packet>,
    returned: Producer<Packet>,
    active: [Option<Packet>; MAX_VOICES],
    deferred: [Option<Packet>; MAX_VOICES],
    channels: usize,
    format: MonitorFormat,
}
/// Only construct after the callback executor has relinquished its endpoint.
/// Does not itself certify that an OS/backend stream has stopped.
pub struct StoppedVoiceConsumer(Box<VoiceConsumer>);
impl std::fmt::Debug for StoppedVoiceConsumer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StoppedVoiceConsumer")
            .finish_non_exhaustive()
    }
}
#[derive(Clone, Copy, Debug, Default)]
pub struct HandoffReport {
    pub first: [Option<(VoiceKey, Channel)>; MAX_VOICES],
    /// First descriptor copied, scoped to the bound Runtime. Silence inside a
    /// sound is not skipped; only nonzero diagnostic fixtures measure latency.
    pub runtime_first: [Option<(u64, Channel)>; MAX_VOICES],
    /// Scoped to this boundary. A future device adapter must retain the stream
    /// identity and filter invalidated batches again at its own copy boundary.
    pub lane_epochs: [u64; 2],
    pub invalidated_lanes: [bool; 2],
    pub deferred_returns: usize,
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RuntimePumpReport {
    pub admitted: usize,
    pub retired: usize,
    pub waiting: usize,
}

/// Allocates two fixed32-entry queues and boxed consumer state before rendering.
/// PCM is shared, not copied or decoded here.
/// Same-format monitoring only. Conversion remains in the existing audio path.
pub fn boundary(
    budget: PcmBudget,
    rate: u32,
    channels: u16,
    source: Origin,
) -> Result<(VoiceOwner, Box<VoiceConsumer>), HandoffError> {
    if !valid_origin(source) {
        return Err(HandoffError::InvalidOrigin);
    }
    if !matches!(rate, 44100 | 48000) || !matches!(channels, 1 | 2) {
        return Err(HandoffError::InvalidFormat);
    }
    let (publish, pending) = RingBuffer::new(MAX_VOICES);
    let (returned, collect) = RingBuffer::new(MAX_VOICES);
    let shared = Arc::new(Shared {
        gate: AtomicU64::new(1 | (1 << EPOCH_BITS)),
    });
    Ok((
        VoiceOwner {
            shared: shared.clone(),
            pending: Some(publish),
            returned: collect,
            pins: std::array::from_fn(|_| None),
            budget,
            format: (rate, channels),
            source,
            serial: 0,
            runtime: None,
        },
        Box::new(VoiceConsumer {
            shared,
            pending,
            returned,
            active: std::array::from_fn(|_| None),
            deferred: std::array::from_fn(|_| None),
            channels: usize::from(channels),
            format: MonitorFormat::new(rate, channels).expect("validated format"),
        }),
    ))
}
impl VoiceOwner {
    pub(crate) fn retained_drafts(&self) -> impl Iterator<Item = u64> + '_ {
        self.pins
            .iter()
            .flatten()
            .filter_map(|p| p.runtime.as_ref()?.info().origin.draft.map(|(id, _)| id))
    }
    pub fn token(&self, channel: Channel) -> VoiceToken {
        VoiceToken {
            boundary: Arc::downgrade(&self.shared),
            channel,
            epoch: epoch(self.shared.gate.load(Ordering::Acquire), channel),
        }
    }
    pub fn outstanding(&self) -> usize {
        self.pins.iter().filter(|p| p.is_some()).count()
    }
    pub fn admit(
        &mut self,
        source: Origin,
        token: VoiceToken,
        sound: &PreparedSound,
    ) -> Result<VoiceKey, HandoffError> {
        if self.runtime.is_some() {
            return Err(HandoffError::RuntimeMode);
        }
        self.publish(source, token, sound, None)
    }
    fn publish(
        &mut self,
        source: Origin,
        token: VoiceToken,
        sound: &PreparedSound,
        runtime: Option<&VoiceHandoff>,
    ) -> Result<VoiceKey, HandoffError> {
        let word = self.shared.gate.load(Ordering::Acquire);
        if word & CLOSED != 0 {
            return Err(HandoffError::Closed);
        }
        if (runtime.is_none() && source != self.source)
            || runtime
                .is_some_and(|v| !v.validity().is_valid() || v.info().channel != token.channel)
            || !token.boundary.ptr_eq(&Arc::downgrade(&self.shared))
            || epoch(word, token.channel) != token.epoch
        {
            return Err(HandoffError::Stale);
        }
        if !permits(word, &token) {
            return Err(HandoffError::Frozen);
        }
        if runtime.is_none() && sound.format() != self.format {
            return Err(HandoffError::InvalidFormat);
        }
        if !self.budget.same_pool(sound.pcm().budget()) {
            return Err(HandoffError::WrongPool);
        }
        let slot = self
            .pins
            .iter()
            .position(Option::is_none)
            .ok_or(HandoffError::Busy)?;
        let generation = self.serial.checked_add(1).ok_or(HandoffError::Exhausted)?;
        let key = VoiceKey { slot, generation };
        self.pins[slot] = Some(Pin {
            key,
            pcm: sound.pcm().clone(),
            runtime: runtime.cloned(),
        });
        let packet = Packet {
            key,
            token,
            sound: sound.clone(),
            cursor: 0,
            started: false,
            validity: runtime.map(VoiceHandoff::validity),
            runtime_voice: runtime.map(|v| v.info().id),
        };
        match self.pending.as_mut().expect("open producer").push(packet) {
            Ok(()) => {
                self.serial = generation;
                Ok(key)
            }
            Err(PushError::Full(packet)) => {
                // All rollback/drop work is on the owner, never the callback.
                drop(packet);
                self.pins[slot] = None;
                Err(HandoffError::Busy)
            }
        }
    }
    /// Owner-side bridge only. Runtime remains the binding/candidate/cancellation
    /// authority; old effective origins are legal and are never rewritten here.
    /// Immediate voices only: Runtime explicitly rejects future scheduling in
    /// this mode. The default mixer and desktop host do not select this path.
    pub fn pump_runtime(
        &mut self,
        runtime: &mut Runtime,
    ) -> Result<RuntimePumpReport, HandoffError> {
        if !runtime.voice_handoff_enabled() {
            return Err(HandoffError::RuntimeMode);
        }
        if runtime.output_format() != self.format {
            return Err(HandoffError::InvalidFormat);
        }
        if !self.budget.same_pool(&runtime.pcm_budget()) {
            return Err(HandoffError::WrongPool);
        }
        let key = runtime.voice_runtime_key().expect("checked mode");
        if self
            .runtime
            .as_ref()
            .is_some_and(|bound| !bound.same_runtime(&key))
        {
            return Err(HandoffError::ForeignRuntime);
        }
        if self.runtime.is_none() && self.outstanding() != 0 {
            return Err(HandoffError::Busy);
        }
        self.runtime = Some(key);
        let retired = self.collect_with(|lease| {
            runtime.finish_voice_handoff(&lease);
        });
        self.set_scene_running(runtime.scene_running())?;
        let mut admitted = 0;
        while self.outstanding() < MAX_VOICES {
            if self.serial == u64::MAX {
                return Err(HandoffError::Exhausted);
            }
            let Some(lease) = runtime
                .take_voice_handoff()
                .map_err(|_| HandoffError::RuntimeMode)?
            else {
                break;
            };
            let result = self.publish(
                lease.info().origin,
                self.token(lease.info().channel),
                lease.sound(),
                Some(&lease),
            );
            if let Err(error) = result {
                runtime.retry_voice_handoff(&lease);
                return Err(error);
            }
            admitted += 1;
        }
        Ok(RuntimePumpReport {
            admitted,
            retired,
            waiting: runtime.handoff_waiting(),
        })
    }
    fn update(&mut self, lanes: [bool; 2], running: Option<bool>) -> Result<(), HandoffError> {
        let result = self
            .shared
            .gate
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |word| {
                if word & CLOSED != 0 {
                    return None;
                }
                let mut next = word;
                for (i, change) in lanes.into_iter().enumerate() {
                    let channel = if i == 0 {
                        Channel::Scene
                    } else {
                        Channel::Audition
                    };
                    if change {
                        if epoch(word, channel) == EPOCH_MAX {
                            return None;
                        }
                        next += 1 << (i as u32 * EPOCH_BITS);
                    }
                }
                if let Some(running) = running {
                    next = if running {
                        next | RUNNING
                    } else {
                        next & !RUNNING
                    };
                }
                Some(next)
            });
        match result {
            Ok(_) => Ok(()),
            Err(word) if word & CLOSED != 0 => Err(HandoffError::Closed),
            Err(_) => {
                self.close();
                Err(HandoffError::Exhausted)
            }
        }
    }
    /// Voice lane gate only: a transport coordinator must prime background and
    /// reconcile its pause anchor before requesting a real scene resume.
    pub fn set_scene_running(&mut self, running: bool) -> Result<(), HandoffError> {
        let word = self.shared.gate.load(Ordering::Acquire);
        if word & CLOSED != 0 {
            return Err(HandoffError::Closed);
        }
        if (word & RUNNING != 0) == running {
            return Ok(());
        }
        self.update([true, false], Some(running))
    }
    pub fn stop_audition(&mut self) -> Result<(), HandoffError> {
        self.update([false, true], None)
    }
    /// Explicit cancellation/history invalidation, not ordinary candidate ready.
    pub fn invalidate_source(&mut self, next: Origin) -> Result<(), HandoffError> {
        if !valid_origin(next) {
            return Err(HandoffError::InvalidOrigin);
        }
        if self.shared.gate.load(Ordering::Acquire) & CLOSED != 0 {
            return Err(HandoffError::Closed);
        }
        if self.source != next {
            self.update([true; 2], None)?;
            self.source = next;
        }
        Ok(())
    }
    pub fn close(&mut self) {
        self.shared.gate.fetch_or(CLOSED, Ordering::Release);
    }
    pub fn collect(&mut self) -> usize {
        // Runtime credits are acknowledged only by pump_runtime. A caller of
        // the raw candidate API must not consume and lose those return leases.
        if self.pins.iter().flatten().any(|p| p.runtime.is_some()) {
            return 0;
        }
        self.collect_with(|_| {})
    }
    fn collect_with(&mut self, mut finished: impl FnMut(VoiceHandoff)) -> usize {
        let mut count = 0;
        for _ in 0..MAX_VOICES {
            let Ok(packet) = self.returned.pop() else {
                break;
            };
            let mut pin = self.pins[packet.key.slot]
                .take()
                .expect("pinned slot until return");
            assert_eq!(pin.key, packet.key);
            assert!(pin.pcm.same_allocation(packet.sound.pcm()));
            drop(packet);
            if let Some(lease) = pin.runtime.take() {
                finished(lease);
            }
            drop(pin);
            count += 1;
        }
        if self.pending.as_ref().is_some_and(Producer::is_abandoned) {
            self.close();
            // The unique callback endpoint has been destroyed. Drop the orphan
            // pending ring and remaining pins on the owner, not in rendering.
            self.pending = None;
            for pin in &mut self.pins {
                if let Some(mut pin) = pin.take() {
                    if let Some(lease) = pin.runtime.take() {
                        finished(lease);
                    }
                    count += 1;
                }
            }
        }
        count
    }
    /// Call on the owner after joining/reclaiming the callback endpoint. A
    /// foreign endpoint is returned intact; it cannot release this owner's pins.
    pub fn finish(
        &mut self,
        mut stopped: StoppedVoiceConsumer,
    ) -> Result<(), StoppedVoiceConsumer> {
        if !Arc::ptr_eq(&self.shared, &stopped.0.shared) {
            return Err(stopped);
        }
        self.close();
        stopped.0.drain_pending();
        stopped.0.retire_all();
        self.collect_with(|_| {});
        // Defensive full-return fallback is bounded, never waits for a reader.
        stopped.0.flush_deferred();
        self.collect_with(|_| {});
        assert_eq!(self.outstanding(), 0);
        Ok(())
    }
    /// Runtime bridge shutdown on the owner, after the executor relinquished
    /// the endpoint. Reject foreign identities before closing either party.
    pub fn finish_runtime(
        &mut self,
        runtime: &mut Runtime,
        stopped: StoppedVoiceConsumer,
    ) -> Result<(), StoppedVoiceConsumer> {
        if !Arc::ptr_eq(&self.shared, &stopped.0.shared)
            || self
                .runtime
                .as_ref()
                .zip(runtime.voice_runtime_key().as_ref())
                .is_none_or(|(a, b)| !a.same_runtime(b))
        {
            return Err(stopped);
        }
        runtime.shutdown();
        self.finish(stopped)
    }
}
impl Drop for VoiceOwner {
    fn drop(&mut self) {
        crate::callback_audit::released();
        self.close();
    }
}
impl Drop for VoiceConsumer {
    fn drop(&mut self) {
        crate::callback_audit::released();
        // Endpoint destruction is outside rendering. Pin/queue ownership is
        // retained until owner/endpoint teardown, including abnormal drop order.
        self.shared.gate.fetch_or(CLOSED, Ordering::Release);
    }
}
#[derive(Clone, Copy)]
enum Checkpoint {
    AfterDrain,
    BeforeFence,
    AfterRetire,
}
impl VoiceConsumer {
    pub(crate) fn format(&self) -> (u32, u16) {
        (self.format.rate(), self.channels as u16)
    }
    pub fn stop(self: Box<Self>) -> StoppedVoiceConsumer {
        self.shared.gate.fetch_or(CLOSED, Ordering::Release);
        StoppedVoiceConsumer(self)
    }
    fn drain_pending(&mut self) {
        for _ in 0..MAX_VOICES {
            let Ok(packet) = self.pending.pop() else {
                break;
            };
            let slot = packet.key.slot;
            assert!(self.active[slot].is_none() && self.deferred[slot].is_none());
            self.active[slot] = Some(packet);
        }
    }
    fn return_packet(&mut self, packet: Packet) {
        if let Err(PushError::Full(packet)) = self.returned.push(packet) {
            let slot = packet.key.slot;
            assert!(self.deferred[slot].is_none());
            self.deferred[slot] = Some(packet); // No last-PCM drop on backpressure.
        }
    }
    fn retire_slot(&mut self, slot: usize) {
        if let Some(packet) = self.active[slot].take() {
            self.return_packet(packet);
        }
    }
    fn retire_all(&mut self) {
        for slot in 0..MAX_VOICES {
            self.retire_slot(slot);
        }
    }
    fn flush_deferred(&mut self) {
        for slot in 0..MAX_VOICES {
            if let Some(packet) = self.deferred[slot].take() {
                self.return_packet(packet);
            }
        }
    }
    /// Unsaturated split lanes; final background addition, volume and clipping
    /// belong to the output adapter. Writes valid slices completely; invalid
    /// blocks are rejected before clearing, dequeueing or advancing voices.
    pub fn render_split(
        &mut self,
        scene: &mut [f32],
        audition: &mut [f32],
    ) -> Result<HandoffReport, HandoffError> {
        self.render_inner(scene, audition, |_| {})
    }
    fn render_inner(
        &mut self,
        scene: &mut [f32],
        audition: &mut [f32],
        mut checkpoint: impl FnMut(Checkpoint),
    ) -> Result<HandoffReport, HandoffError> {
        self.render_committed(scene, audition, &mut checkpoint, |_, _, _| {})
    }
    /// Internal output composition seam: descriptor return follows the caller's
    /// bounded buffer copy. No Runtime or project is passed to the consumer.
    pub(crate) fn render_to(
        &mut self,
        scene: &mut [f32],
        audition: &mut [f32],
        mut before_fence: impl FnMut(),
        copy: impl FnOnce(&[f32], &[f32], &mut HandoffReport),
    ) -> Result<HandoffReport, HandoffError> {
        self.render_committed(
            scene,
            audition,
            |stage| {
                if matches!(stage, Checkpoint::BeforeFence) {
                    before_fence();
                }
            },
            copy,
        )
    }
    fn render_committed(
        &mut self,
        scene: &mut [f32],
        audition: &mut [f32],
        mut checkpoint: impl FnMut(Checkpoint),
        copy: impl FnOnce(&[f32], &[f32], &mut HandoffReport),
    ) -> Result<HandoffReport, HandoffError> {
        if scene.len() != audition.len()
            || !scene.len().is_multiple_of(self.channels)
            || scene.len() / self.channels > MAX_HANDOFF_FRAMES
        {
            return Err(HandoffError::InvalidBlock);
        }
        scene.fill(0.0);
        audition.fill(0.0);
        self.flush_deferred();
        self.drain_pending();
        checkpoint(Checkpoint::AfterDrain); // Private hook; production instantiation is no-op.
        let before = self.shared.gate.load(Ordering::Acquire);
        let mut report = HandoffReport::default();
        let starts = std::array::from_fn::<_, MAX_VOICES, _>(|slot| {
            self.active[slot].as_ref().map_or(0, |p| p.cursor)
        });
        let mut ended = [false; MAX_VOICES];
        for slot in 0..MAX_VOICES {
            if self.active[slot].as_ref().is_some_and(|p| {
                !permits(before, &p.token) || p.validity.as_ref().is_some_and(|v| !v.is_valid())
            }) {
                self.retire_slot(slot);
            }
        }
        // Each output sample still receives Voice additions in slot 0..31 order.
        // The audio layer hoists immutable format/layout work once per block.
        let gains = std::array::from_fn::<_, MAX_VOICES, _>(|slot| {
            self.active[slot].as_ref().map_or(0.0, |p| {
                p.validity
                    .as_ref()
                    .map_or_else(|| p.sound.monitor_gain(), |v| v.gain())
            })
        });
        for (slot, done) in ended.iter_mut().enumerate() {
            let Some(packet) = self.active[slot].as_mut() else {
                continue;
            };
            let target = if packet.token.channel == Channel::Scene {
                &mut *scene
            } else {
                &mut *audition
            };
            let added = packet
                .sound
                .add_monitor_frames_with_gain(
                    (packet.cursor / self.channels) as u64,
                    self.format,
                    target,
                    gains[slot],
                )
                .unwrap_or(0);
            if added > 0 && !packet.started {
                report.first[slot] = Some((packet.key, packet.token.channel));
                packet.started = true;
            }
            packet.cursor += added * self.channels;
            *done = added < target.len() / self.channels;
        }
        // Sample exhaustion inside a block is already reported by monitor_frame.
        // Check exact block-end completion once per remaining Voice, rather than
        // repeating the rate conversion after every sample. Empty callbacks keep
        // their previous no-progress semantics.
        if !scene.is_empty() {
            for (slot, done) in ended.iter_mut().enumerate() {
                if let Some(packet) = self.active[slot].as_ref().filter(|_| !*done) {
                    *done = packet
                        .sound
                        .monitor_finished((packet.cursor / self.channels) as u64, self.format);
                }
            }
        }
        checkpoint(Checkpoint::BeforeFence);
        // The raw boundary still allows concurrent retirement/reuse. Runtime
        // descriptors must survive the final revocation fence, even if their
        // samples already ended. Raw and Runtime admissions cannot be mixed.
        for (slot, done) in ended.iter().copied().enumerate() {
            if done
                && self.active[slot]
                    .as_ref()
                    .is_some_and(|p| p.validity.is_none())
            {
                self.retire_slot(slot);
            }
        }
        checkpoint(Checkpoint::AfterRetire);
        // Retain descriptors until the per-voice revocation fence, including
        // voices that ended in this block. If a selective cancel raced mixing,
        // replay once from saved cursors with the same sampler/summation order.
        // This keeps unrelated voices exact without per-voice PCM scratch or
        // subtracting rounded contributions. No retry loop or callback lease drop.
        let cancelled = std::array::from_fn::<_, MAX_VOICES, _>(|slot| {
            self.active[slot]
                .as_ref()
                .is_some_and(|p| p.validity.as_ref().is_some_and(|v| !v.is_valid()))
        });
        if cancelled.iter().any(|c| *c) {
            scene.fill(0.0);
            audition.fill(0.0);
            for start in (0..scene.len()).step_by(self.channels) {
                for slot in 0..MAX_VOICES {
                    let Some(packet) = self.active[slot].as_ref().filter(|_| !cancelled[slot])
                    else {
                        continue;
                    };
                    if let Some(samples) = packet.sound.monitor_frame_with_gain(
                        ((starts[slot] + start) / self.channels) as u64,
                        self.format,
                        gains[slot],
                    ) {
                        let target = if packet.token.channel == Channel::Scene {
                            &mut *scene
                        } else {
                            &mut *audition
                        };
                        for channel in 0..self.channels {
                            target[start + channel] += samples[channel];
                        }
                    }
                }
            }
        }
        for (slot, cancelled) in cancelled.iter().copied().enumerate() {
            if cancelled {
                report.first[slot] = None;
            }
        }
        let after = self.shared.gate.load(Ordering::Acquire);
        report.lane_epochs = [
            epoch(after, Channel::Scene),
            epoch(after, Channel::Audition),
        ];
        for (i, target) in [&mut *scene, &mut *audition].into_iter().enumerate() {
            let channel = if i == 0 {
                Channel::Scene
            } else {
                Channel::Audition
            };
            if epoch(before, channel) != epoch(after, channel)
                || after & CLOSED != 0
                || (i == 0 && after & RUNNING == 0)
            {
                target.fill(0.0);
                report.invalidated_lanes[i] = true;
                for first in &mut report.first {
                    if first.is_some_and(|(_, lane)| lane == channel) {
                        *first = None;
                    }
                }
            }
        }
        copy(scene, audition, &mut report);
        for (slot, first) in report.first.iter().enumerate() {
            report.runtime_first[slot] = first.and_then(|(_, lane)| {
                self.active[slot]
                    .as_ref()
                    .and_then(|p| p.runtime_voice.map(|id| (id, lane)))
            });
        }
        for slot in 0..MAX_VOICES {
            if self.active[slot]
                .as_ref()
                .is_some_and(|p| !permits(after, &p.token))
                || cancelled[slot]
                || ended[slot]
            {
                self.retire_slot(slot);
            }
        }
        report.deferred_returns = self.deferred.iter().filter(|p| p.is_some()).count();
        Ok(report)
    }
}

#[cfg(test)]
mod tests;
