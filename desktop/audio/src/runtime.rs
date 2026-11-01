//! Bounded, deterministic software control/mixer. No clock, worker or device.
use crate::{SoundPlan, SoundTarget};
use skin_editor_domain::AudioComponentId;
use std::collections::{BTreeMap, VecDeque};

pub const MAX_VOICES: usize = 32;
pub const MAX_EVENTS: usize = 64;
pub const MAX_BINDINGS: usize = 256;
pub const MAX_SOUND_SAMPLES: usize = 1024 * 1024; // 4 MiB float32, not process budget.
/// Linear interpolation and mono duplicate / stereo mean, monitoring only.
pub const MIX_CONVERSION_VERSION: u16 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Origin {
    pub session: u128,
    pub node: u64,
    pub revision: u64,
    pub navigation_revision: u64,
    pub draft: Option<(u64, u64)>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Channel {
    Scene,
    Audition,
}
impl Channel {
    fn index(self) -> usize {
        match self {
            Self::Scene => 0,
            Self::Audition => 1,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GateToken {
    device: u64,
    channel: u64,
    selection: u64,
    lane: Channel,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BuildRequest {
    pub origin: Origin,
    pub plan: SoundPlan,
    serial: u64,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrepareFailure {
    Busy,
    Cancelled,
    MetadataMismatch,
    Unsupported,
    InvalidPcm,
    FormatMismatch,
    ResourceLimit,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeError {
    Busy,
    Stale,
    NotReady,
    Frozen,
    DeviceUnavailable,
    Closed,
    MissingComponent,
    ResourceLimit,
    InvalidBlock,
    InvalidOrigin,
    InvalidPrepared,
    ExecutionMode,
    UnsupportedScheduling,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Completion {
    Activated { effective_from_mix_frame: u64 },
    Failed(PrepareFailure),
    Stale,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Preparation {
    Pending,
    Ready,
    Failed(PrepareFailure),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BindingStatus {
    pub desired: Origin,
    pub effective: Option<Origin>,
    pub preparation: Preparation,
}

#[derive(Clone, Debug)]
pub struct PreparedSound {
    plan: SoundPlan,
    rate: u32,
    channels: u16,
    pcm: crate::PcmBuffer,
}
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PreparedKey {
    blob: skin_editor_domain::BlobRef,
    rate: u32,
    channels: u16,
    frames: u64,
    gain: u64,
    visible: bool,
    version: u16,
}
impl PreparedKey {
    pub fn new(plan: SoundPlan) -> Result<Self, PrepareFailure> {
        let m = plan.metadata.ok_or(PrepareFailure::Unsupported)?;
        Ok(Self {
            blob: plan.source,
            rate: m.sample_rate(),
            channels: m.channels(),
            frames: m.frames(),
            gain: plan.gain.get().to_bits(),
            visible: plan.visible,
            version: crate::SOUND_PLAN_VERSION,
        })
    }
}
#[derive(Clone)]
pub struct PreparedWeak {
    key: PreparedKey,
    pcm: crate::PcmWeak,
}
impl PreparedWeak {
    pub fn is_alive(&self) -> bool {
        self.pcm.is_alive()
    }
    pub fn upgrade_for(&self, plan: SoundPlan, budget: &crate::PcmBudget) -> Option<PreparedSound> {
        if PreparedKey::new(plan).ok()? != self.key {
            return None;
        }
        let pcm = self.pcm.upgrade()?;
        if !pcm.budget().same_pool(budget) {
            return None;
        }
        Some(PreparedSound {
            plan,
            rate: self.key.rate,
            channels: self.key.channels,
            pcm,
        })
    }
}
impl PreparedSound {
    /// Caller-owned decoded input is copied once into the shared budget.
    /// Decoder workers instead fill an already reserved buffer in place.
    pub fn from_source(
        plan: SoundPlan,
        samples: &[f32],
        budget: &crate::PcmBudget,
    ) -> Result<Self, PrepareFailure> {
        if samples.len() > MAX_SOUND_SAMPLES {
            return Err(PrepareFailure::ResourceLimit);
        }
        let mut pcm = budget
            .reserve(samples.len())
            .map_err(|_| PrepareFailure::ResourceLimit)?
            .allocate();
        pcm.samples_mut()
            .map_err(|_| PrepareFailure::InvalidPcm)?
            .copy_from_slice(samples);
        Self::from_decoded(plan, pcm)
    }
    pub fn from_decoded(plan: SoundPlan, pcm: crate::PcmBuffer) -> Result<Self, PrepareFailure> {
        let m = plan.metadata.ok_or(PrepareFailure::Unsupported)?;
        let count = m
            .frames()
            .checked_mul(u64::from(m.channels()))
            .ok_or(PrepareFailure::ResourceLimit)?;
        if count > MAX_SOUND_SAMPLES as u64 {
            return Err(PrepareFailure::ResourceLimit);
        }
        if count != pcm.samples().len() as u64
            || pcm
                .samples()
                .iter()
                .any(|s| !s.is_finite() || s.abs() > 1.0)
        {
            return Err(PrepareFailure::InvalidPcm);
        }
        Ok(Self {
            plan,
            rate: m.sample_rate(),
            channels: m.channels(),
            pcm,
        })
    }
    /// Neutral decoded PCM; monitoring applies the recipe or live Voice gain.
    /// Export processing remains SoundPlan::process and must not copy this raw slice as edited audio.
    pub fn samples(&self) -> &[f32] {
        self.pcm.samples()
    }
    pub fn pcm(&self) -> &crate::PcmBuffer {
        &self.pcm
    }
    pub fn monitor_gain(&self) -> f32 {
        if self.plan.visible {
            self.plan.gain.get() as f32
        } else {
            0.0
        }
    }
    /// Source PCM format, independent of any device or monitoring conversion.
    pub fn format(&self) -> (u32, u16) {
        (self.rate, self.channels)
    }
    pub fn cache_key(&self) -> PreparedKey {
        PreparedKey::new(self.plan).expect("validated metadata")
    }
    pub fn downgrade(&self) -> PreparedWeak {
        PreparedWeak {
            key: self.cache_key(),
            pcm: self.pcm.downgrade(),
        }
    }
}
#[derive(Clone)]
struct Effective {
    origin: Origin,
    sound: Option<PreparedSound>,
}
struct Binding {
    plan: SoundPlan,
    pending: Option<BuildRequest>,
    effective: Option<Effective>,
    preparation: Preparation,
}
struct Event {
    target: SoundTarget,
    gate: GateToken,
    at: u64,
    effective: Effective,
    stale: bool,
}
struct Voice {
    id: u64,
    target: SoundTarget,
    lane: Channel,
    effective: Effective,
    cursor: usize,
    gain: f32,
    handoff: Option<crate::VoiceValidity>,
}
impl Drop for Voice {
    fn drop(&mut self) {
        if let Some(lease) = &self.handoff {
            lease.revoke();
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TriggerReceipt {
    pub origin: Origin,
    pub stale: bool,
    pub voice: Option<u64>,
    pub queued: bool,
    pub silent: bool,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VoiceInfo {
    pub id: u64,
    pub target: SoundTarget,
    pub channel: Channel,
    pub origin: Origin,
    pub cursor: usize,
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MixReport {
    pub started: u32,
    pub dropped: u32,
    pub clipped_samples: u32,
}

pub struct Runtime {
    budget: crate::PcmBudget,
    rate: u32,
    channels: u16,
    origin: Option<Origin>,
    minimum_conversion: bool,
    external_voices: Option<std::sync::Arc<()>>,
    bindings: BTreeMap<SoundTarget, Binding>,
    voices: [Option<Voice>; MAX_VOICES],
    events: VecDeque<Event>,
    selection: u64,
    device: u64,
    gates: [u64; 2],
    device_ready: bool,
    scene_running: bool,
    closed: bool,
    next_build: u64,
    next_voice: u64,
    mix_frame: u64,
    scene_frame: u64,
    audition_edits: crate::AuditionEdits,
}
fn next(value: u64) -> Result<u64, RuntimeError> {
    value.checked_add(1).ok_or(RuntimeError::ResourceLimit)
}
impl Runtime {
    pub fn new(rate: u32, channels: u16) -> Result<Self, RuntimeError> {
        Self::with_budget(rate, channels, crate::PcmBudget::default())
    }
    pub fn pcm_budget(&self) -> crate::PcmBudget {
        self.budget.clone()
    }
    pub fn with_budget(
        rate: u32,
        channels: u16,
        budget: crate::PcmBudget,
    ) -> Result<Self, RuntimeError> {
        if rate == 0 || !(1..=2).contains(&channels) {
            return Err(RuntimeError::InvalidBlock);
        }
        Ok(Self {
            budget,
            rate,
            channels,
            origin: None,
            minimum_conversion: false,
            external_voices: None,
            bindings: BTreeMap::new(),
            voices: std::array::from_fn(|_| None),
            events: VecDeque::with_capacity(MAX_EVENTS),
            selection: 0,
            device: 1,
            gates: [1, 1],
            device_ready: true,
            scene_running: false,
            closed: false,
            next_build: 1,
            next_voice: 1,
            mix_frame: 0,
            scene_frame: 0,
            audition_edits: crate::AuditionEdits::default(),
        })
    }

    /// Explicit monitoring policy; source PCM and export recipes remain unchanged.
    pub fn enable_minimum_conversion(&mut self) {
        self.minimum_conversion = true;
    }

    /// Conservative audio activity, including scheduled events. The independent
    /// scene clock alone does not turn blank PCM into an audio time reference.
    /// This is control-side metadata, never inferred from zero-valued PCM.
    pub fn output_activity(&self) -> [bool; 2] {
        let mut active = [false, false];
        for voice in self.voices.iter().flatten() {
            active[voice.lane.index()] = true;
        }
        for event in &self.events {
            active[event.gate.lane.index()] = true;
        }
        active
    }

    pub fn output_format(&self) -> (u32, u16) {
        (self.rate, self.channels)
    }

    /// Atomic control snapshot. Every completion must still match the newest
    /// origin AND build serial. Changing selection invalidates queued events.
    pub fn synchronize(
        &mut self,
        origin: Origin,
        plans: &[SoundPlan],
    ) -> Result<Vec<BuildRequest>, RuntimeError> {
        self.synchronize_cancelled(origin, plans, &[])
    }
    /// Explicit cancellation IDs still revoke all non-reconciled provenance.
    pub fn synchronize_cancelled(
        &mut self,
        origin: Origin,
        plans: &[SoundPlan],
        cancelled_drafts: &[u64],
    ) -> Result<Vec<BuildRequest>, RuntimeError> {
        if self.closed {
            return Err(RuntimeError::Closed);
        }
        if origin.session == 0 || origin.node == 0 {
            return Err(RuntimeError::InvalidOrigin);
        }
        if plans.len() > MAX_BINDINGS {
            return Err(RuntimeError::ResourceLimit);
        }
        let mut unique = BTreeMap::new();
        for plan in plans {
            if !matches!(plan.target, SoundTarget::Component(_))
                || unique.insert(plan.target, *plan).is_some()
            {
                return Err(RuntimeError::InvalidPrepared);
            }
        }
        if self.origin == Some(origin) {
            if self
                .bindings
                .keys()
                .filter(|t| matches!(t, SoundTarget::Component(_)))
                .count()
                != plans.len()
                || plans
                    .iter()
                    .any(|p| self.bindings.get(&p.target).is_none_or(|b| b.plan != *p))
            {
                return Err(RuntimeError::InvalidOrigin);
            }
            self.invalidate_drafts(cancelled_drafts);
            return Ok(Vec::new());
        }
        let selection = next(self.selection)?;
        self.next_build
            .checked_add(plans.len() as u64)
            .ok_or(RuntimeError::ResourceLimit)?;
        let reset = self.origin.is_some_and(|old| {
            old.session != origin.session || old.navigation_revision != origin.navigation_revision
        });
        let cancelled = self.origin.and_then(|old| {
            let id = old.draft?.0;
            (old.node == origin.node
                && old.revision == origin.revision
                && origin.draft.map(|d| d.0) != Some(id))
            .then_some(id)
        });
        let change = self.audition_edits.select(origin, plans, cancelled_drafts);
        for voice in &mut self.voices {
            let Some(v) = voice.as_mut().filter(|v| v.lane == Channel::Audition) else {
                continue;
            };
            let sound = v.effective.sound.as_ref().expect("audible voice");
            let plan = unique.get(&v.target);
            if reset
                || plan.is_none_or(|p| {
                    !p.visible || !crate::audition_edit::same_content(sound.plan, *p)
                })
            {
                *voice = None;
            } else if change.gain_edit {
                v.gain = plan.expect("compatible target").gain.get() as f32;
                if let Some(lease) = &v.handoff {
                    lease.set_gain(v.gain);
                }
            }
        }
        self.invalidate_drafts(cancelled_drafts);
        self.clear_library();
        let mut old = std::mem::take(&mut self.bindings);
        if reset {
            self.voices.fill_with(|| None);
        }
        if let Some(id) = cancelled {
            for voice in &mut self.voices {
                if voice
                    .as_ref()
                    .is_some_and(|v| v.effective.origin.draft.is_some_and(|d| d.0 == id))
                {
                    *voice = None;
                }
            }
        }
        self.events.clear();
        self.selection = selection;
        self.origin = Some(origin);
        let mut builds = Vec::new();
        for plan in unique.into_values() {
            let prior = old.remove(&plan.target);
            let effective = prior.and_then(|b| b.effective).filter(|e| {
                !reset && cancelled.is_none_or(|id| e.origin.draft.is_none_or(|d| d.0 != id))
            });
            let reusable = effective.as_ref().is_some_and(|e| match &e.sound {
                Some(sound) => sound.plan == plan,
                None => !plan.visible,
            });
            let (pending, effective, preparation) = if !plan.visible {
                (
                    None,
                    Some(Effective {
                        origin,
                        sound: None,
                    }),
                    Preparation::Ready,
                )
            } else if reusable {
                let mut ready = effective.expect("checked ready");
                ready.origin = origin;
                (None, Some(ready), Preparation::Ready)
            } else {
                let request = BuildRequest {
                    origin,
                    plan,
                    serial: self.next_build,
                };
                self.next_build += 1;
                builds.push(request);
                (Some(request), effective, Preparation::Pending)
            };
            self.bindings.insert(
                plan.target,
                Binding {
                    plan,
                    pending,
                    effective,
                    preparation,
                },
            );
        }
        Ok(builds)
    }

    /// One transient library candidate; it never belongs to the project plan.
    pub fn request_library(&mut self, plan: SoundPlan) -> Result<BuildRequest, RuntimeError> {
        let SoundTarget::Library(_) = plan.target else {
            return Err(RuntimeError::InvalidPrepared);
        };
        if plan.gain != skin_editor_domain::Gain::UNITY || !plan.visible {
            return Err(RuntimeError::InvalidPrepared);
        }
        let origin = self.origin.ok_or(RuntimeError::InvalidOrigin)?;
        self.valid(self.token(Channel::Audition))?;
        let serial = self.next_build;
        let following = next(serial)?;
        self.stop_audition(self.token(Channel::Audition))?;
        self.next_build = following;
        let request = BuildRequest {
            origin,
            plan,
            serial,
        };
        self.bindings.insert(
            plan.target,
            Binding {
                plan,
                pending: Some(request),
                effective: None,
                preparation: Preparation::Pending,
            },
        );
        Ok(request)
    }
    /// Short read-only lease for cache lookup while replacing the transient slot.
    pub fn library_sound(&self) -> Option<PreparedSound> {
        self.bindings.iter().find_map(|(target, b)| {
            if matches!(target, SoundTarget::Library(_)) {
                b.effective.as_ref()?.sound.clone()
            } else {
                None
            }
        })
    }
    pub fn library_status(&self) -> Option<(skin_editor_domain::AssetId, BindingStatus)> {
        self.bindings.iter().find_map(|(target, b)| {
            let SoundTarget::Library(id) = target else {
                return None;
            };
            Some((
                *id,
                BindingStatus {
                    desired: self.origin?,
                    effective: b.effective.as_ref().map(|e| e.origin),
                    preparation: b.preparation,
                },
            ))
        })
    }
    pub fn fail_library(&mut self, error: PrepareFailure) {
        for (target, binding) in &mut self.bindings {
            if matches!(target, SoundTarget::Library(_)) {
                binding.preparation = Preparation::Failed(error);
                binding.pending = None;
                binding.effective = None;
            }
        }
    }
    fn clear_library(&mut self) {
        self.bindings
            .retain(|target, _| matches!(target, SoundTarget::Component(_)));
        self.events
            .retain(|e| matches!(e.target, SoundTarget::Component(_)));
        for voice in &mut self.voices {
            if voice
                .as_ref()
                .is_some_and(|v| matches!(v.target, SoundTarget::Library(_)))
            {
                *voice = None;
            }
        }
    }

    pub fn complete(
        &mut self,
        request: BuildRequest,
        result: Result<PreparedSound, PrepareFailure>,
    ) -> Result<Completion, RuntimeError> {
        if self.closed || self.origin != Some(request.origin) {
            return Ok(Completion::Stale);
        }
        let Some(binding) = self.bindings.get_mut(&request.plan.target) else {
            return Ok(Completion::Stale);
        };
        if binding.pending != Some(request) {
            return Ok(Completion::Stale);
        }
        let result = match result {
            Ok(sound)
                if sound.plan != request.plan || !sound.pcm.budget().same_pool(&self.budget) =>
            {
                return Err(RuntimeError::InvalidPrepared);
            }
            Ok(sound)
                if (sound.rate != self.rate || sound.channels != self.channels)
                    && !(self.minimum_conversion
                        && matches!(sound.channels, 1 | 2)
                        && matches!(sound.rate, 44100 | 48000)
                        && matches!(self.rate, 44100 | 48000)) =>
            {
                Err(PrepareFailure::FormatMismatch)
            }
            value => value,
        };
        binding.pending = None;
        match result {
            Ok(sound) => {
                binding.effective = Some(Effective {
                    origin: request.origin,
                    sound: Some(sound),
                });
                binding.preparation = Preparation::Ready;
                self.events.retain(|e| e.target != request.plan.target);
                Ok(Completion::Activated {
                    effective_from_mix_frame: self.mix_frame,
                })
            }
            Err(error) => {
                binding.preparation = Preparation::Failed(error);
                Ok(Completion::Failed(error))
            }
        }
    }
    /// Explicit retry only; a failure never spins an automatic retry loop.
    pub fn retry(&mut self, component: AudioComponentId) -> Result<BuildRequest, RuntimeError> {
        if self.closed {
            return Err(RuntimeError::Closed);
        }
        let binding = self
            .bindings
            .get_mut(&component.into())
            .ok_or(RuntimeError::MissingComponent)?;
        if binding.pending.is_some() {
            return Err(RuntimeError::Busy);
        }
        if !binding.plan.visible {
            return Err(RuntimeError::InvalidPrepared);
        }
        let serial = self.next_build;
        self.next_build = next(serial)?;
        let request = BuildRequest {
            origin: self.origin.ok_or(RuntimeError::InvalidOrigin)?,
            plan: binding.plan,
            serial,
        };
        binding.pending = Some(request);
        binding.preparation = Preparation::Pending;
        Ok(request)
    }

    pub fn status(&self, component: AudioComponentId) -> Option<BindingStatus> {
        let binding = self.bindings.get(&component.into())?;
        Some(BindingStatus {
            desired: self.origin?,
            effective: binding.effective.as_ref().map(|e| e.origin),
            preparation: binding.preparation,
        })
    }

    /// Bounded retained provenance query; no history-sized cancellation payload.
    pub fn retained_drafts(&self) -> Vec<u64> {
        let mut ids = Vec::new();
        let mut add = |origin: Origin| {
            if let Some((id, _)) = origin.draft {
                ids.push(id);
            }
        };
        if let Some(origin) = self.origin {
            add(origin);
        }
        for binding in self.bindings.values() {
            if let Some(effective) = &binding.effective {
                add(effective.origin);
            }
        }
        for voice in self.voices.iter().flatten() {
            add(voice.effective.origin);
        }
        for event in &self.events {
            add(event.effective.origin);
        }
        ids.sort_unstable();
        ids.dedup();
        ids
    }

    /// Called with application-verified cancellation facts. These remain valid
    /// even when cancel and later commits happened between control snapshots.
    pub fn invalidate_drafts(&mut self, ids: &[u64]) {
        let cancelled = |origin: Origin| origin.draft.is_some_and(|d| ids.contains(&d.0));
        self.events.retain(|e| !cancelled(e.effective.origin));
        for voice in &mut self.voices {
            if voice
                .as_ref()
                .is_some_and(|v| cancelled(v.effective.origin))
            {
                *voice = None;
            }
        }
        for binding in self.bindings.values_mut() {
            if binding
                .effective
                .as_ref()
                .is_some_and(|e| cancelled(e.origin))
            {
                binding.effective = None;
            }
            if binding.pending.is_some_and(|b| cancelled(b.origin)) {
                binding.pending = None;
            }
        }
    }
    pub fn token(&self, lane: Channel) -> GateToken {
        GateToken {
            device: self.device,
            channel: self.gates[lane.index()],
            selection: self.selection,
            lane,
        }
    }
    fn valid(&self, token: GateToken) -> Result<(), RuntimeError> {
        if self.closed {
            return Err(RuntimeError::Closed);
        }
        if token != self.token(token.lane) {
            return Err(RuntimeError::Stale);
        }
        if !self.device_ready {
            return Err(RuntimeError::DeviceUnavailable);
        }
        Ok(())
    }
    fn clear_lane(&mut self, lane: Channel) {
        self.events.retain(|e| e.gate.lane != lane);
        for voice in &mut self.voices {
            if voice.as_ref().is_some_and(|v| v.lane == lane) {
                *voice = None;
            }
        }
    }
    pub fn set_scene_running(
        &mut self,
        token: GateToken,
        running: bool,
    ) -> Result<GateToken, RuntimeError> {
        self.valid(token)?;
        if token.lane != Channel::Scene {
            return Err(RuntimeError::Stale);
        }
        let epoch = next(self.gates[0])?;
        self.clear_lane(Channel::Scene);
        self.scene_running = running;
        self.gates[0] = epoch;
        Ok(self.token(Channel::Scene))
    }
    pub fn stop_audition(&mut self, token: GateToken) -> Result<GateToken, RuntimeError> {
        self.valid(token)?;
        if token.lane != Channel::Audition {
            return Err(RuntimeError::Stale);
        }
        let epoch = next(self.gates[1])?;
        self.clear_lane(Channel::Audition);
        self.clear_library();
        self.gates[1] = epoch;
        Ok(self.token(Channel::Audition))
    }
    pub fn seek_scene(&mut self, token: GateToken, frame: u64) -> Result<GateToken, RuntimeError> {
        let token = self.set_scene_running(token, false)?;
        self.scene_frame = frame;
        Ok(token)
    }
    pub fn invalidate_device(&mut self, expected: u64) -> Result<u64, RuntimeError> {
        if self.closed {
            return Err(RuntimeError::Closed);
        }
        if expected != self.device {
            return Err(RuntimeError::Stale);
        }
        let device = next(self.device)?;
        self.device = device;
        self.device_ready = false;
        self.scene_running = false;
        self.voices.fill_with(|| None);
        self.events.clear();
        Ok(device)
    }
    pub fn rebuild_device(&mut self, expected: u64) -> Result<(), RuntimeError> {
        if self.closed {
            return Err(RuntimeError::Closed);
        }
        if expected != self.device {
            return Err(RuntimeError::Stale);
        }
        self.device_ready = true;
        Ok(())
    }
    pub fn device_epoch(&self) -> u64 {
        self.device
    }

    fn start(&mut self, event: Event) -> Result<TriggerReceipt, RuntimeError> {
        let slot = self
            .voices
            .iter()
            .position(Option::is_none)
            .ok_or(RuntimeError::ResourceLimit)?;
        let id = self.next_voice;
        self.next_voice = next(id)?;
        let receipt = TriggerReceipt {
            origin: event.effective.origin,
            stale: event.stale,
            voice: Some(id),
            queued: false,
            silent: false,
        };
        self.voices[slot] = Some(Voice {
            id,
            target: event.target,
            lane: event.gate.lane,
            gain: event
                .effective
                .sound
                .as_ref()
                .expect("audible voice")
                .monitor_gain(),
            effective: event.effective,
            cursor: 0,
            handoff: None,
        });
        Ok(receipt)
    }
    pub fn trigger(
        &mut self,
        token: GateToken,
        component: impl Into<SoundTarget>,
        at_mix_frame: u64,
    ) -> Result<TriggerReceipt, RuntimeError> {
        let target = component.into();
        if matches!(target, SoundTarget::Library(_)) && token.lane != Channel::Audition {
            return Err(RuntimeError::Stale);
        }
        self.valid(token)?;
        if token.lane == Channel::Scene && !self.scene_running {
            return Err(RuntimeError::Frozen);
        }
        if at_mix_frame < self.mix_frame {
            return Err(RuntimeError::Stale);
        }
        if self.external_voices.is_some() && at_mix_frame != self.mix_frame {
            return Err(RuntimeError::UnsupportedScheduling);
        }
        let binding = self
            .bindings
            .get(&target)
            .ok_or(RuntimeError::MissingComponent)?;
        let effective = binding.effective.clone().ok_or(RuntimeError::NotReady)?;
        let stale = Some(effective.origin) != self.origin;
        if effective.sound.is_none() {
            return Ok(TriggerReceipt {
                origin: effective.origin,
                stale,
                voice: None,
                queued: false,
                silent: true,
            });
        }
        let event = Event {
            target,
            gate: token,
            at: at_mix_frame,
            effective,
            stale,
        };
        if at_mix_frame == self.mix_frame {
            return self.start(event);
        }
        if self.events.len() == MAX_EVENTS {
            return Err(RuntimeError::ResourceLimit);
        }
        let receipt = TriggerReceipt {
            origin: event.effective.origin,
            stale,
            voice: None,
            queued: true,
            silent: false,
        };
        let index = self
            .events
            .iter()
            .position(|e| e.at > at_mix_frame)
            .unwrap_or(self.events.len());
        self.events.insert(index, event);
        Ok(receipt)
    }

    /// A deterministic software block, not evidence of native consumption.
    /// Shared mix time advances while the scene is frozen, for explicit audition.
    /// Voice slot order defines summation; no automatic gain normalisation.
    pub fn mix(&mut self, output: &mut [f32]) -> Result<MixReport, RuntimeError> {
        let channels = usize::from(self.channels);
        if !output.len().is_multiple_of(channels) {
            return Err(RuntimeError::InvalidBlock);
        }
        self.mix_frames(output.len() / channels, |i, scene, audition, _| {
            for c in 0..channels {
                output[i * channels + c] = (scene[c] + audition[c]).clamp(-1.0, 1.0);
            }
        })
    }
    /// Unclipped lanes allow native scene fences without truncating audition.
    pub fn mix_split_buffers(
        &mut self,
        scene: &mut crate::PcmBuffer,
        audition: &mut crate::PcmBuffer,
    ) -> Result<MixReport, RuntimeError> {
        self.split_buffers(scene, audition, None)
    }
    /// Per-frame highest Voice serial with a nonzero lane contribution. These
    /// bounded diagnostic values do not replace origin/generation validation.
    pub fn mix_split_buffers_marked(
        &mut self,
        scene: &mut crate::PcmBuffer,
        audition: &mut crate::PcmBuffer,
        marks: &mut [[u64; 2]],
    ) -> Result<MixReport, RuntimeError> {
        self.split_buffers(scene, audition, Some(marks))
    }
    fn split_buffers(
        &mut self,
        scene: &mut crate::PcmBuffer,
        audition: &mut crate::PcmBuffer,
        mut marks: Option<&mut [[u64; 2]]>,
    ) -> Result<MixReport, RuntimeError> {
        if !scene.budget().same_pool(&self.budget) || !audition.budget().same_pool(&self.budget) {
            return Err(RuntimeError::InvalidPrepared);
        }
        let channels = usize::from(self.channels);
        if scene.samples().len() != audition.samples().len()
            || !scene.samples().len().is_multiple_of(channels)
        {
            return Err(RuntimeError::InvalidBlock);
        }
        if marks
            .as_ref()
            .is_some_and(|m| m.len() != scene.samples().len() / channels)
        {
            return Err(RuntimeError::InvalidBlock);
        }
        let a = scene
            .samples_mut()
            .map_err(|_| RuntimeError::InvalidBlock)?;
        let b = audition
            .samples_mut()
            .map_err(|_| RuntimeError::InvalidBlock)?;
        self.mix_frames(a.len() / channels, |i, scene, audition, ids| {
            if let Some(m) = marks.as_deref_mut() {
                m[i] = ids;
            }
            a[i * channels..(i + 1) * channels].copy_from_slice(&scene[..channels]);
            b[i * channels..(i + 1) * channels].copy_from_slice(&audition[..channels]);
        })
    }
    fn mix_frames(
        &mut self,
        frames: usize,
        mut write: impl FnMut(usize, [f32; 2], [f32; 2], [u64; 2]),
    ) -> Result<MixReport, RuntimeError> {
        if self.closed {
            return Err(RuntimeError::Closed);
        }
        if self.external_voices.is_some() {
            return Err(RuntimeError::ExecutionMode);
        }
        let channels = usize::from(self.channels);
        self.mix_frame
            .checked_add(frames as u64)
            .ok_or(RuntimeError::ResourceLimit)?;
        if self.scene_running {
            self.scene_frame
                .checked_add(frames as u64)
                .ok_or(RuntimeError::ResourceLimit)?;
        }
        if !self.device_ready {
            return Err(RuntimeError::DeviceUnavailable);
        }
        let mut report = MixReport::default();
        for frame in 0..frames {
            let mut lanes = [[0.0f32; 2]; 2];
            let mut voice_ids = [0u64; 2];
            while self.events.front().is_some_and(|e| e.at <= self.mix_frame) {
                let event = self.events.pop_front().expect("checked event");
                if self.valid(event.gate).is_err() || self.start(event).is_err() {
                    report.dropped += 1;
                } else {
                    report.started += 1;
                }
            }
            for voice in self.voices.iter_mut().flatten() {
                let sound = voice.effective.sound.as_ref().expect("audible voice");
                let samples = sound
                    .monitor_frame_with_gain(
                        (voice.cursor / channels) as u64,
                        crate::MonitorFormat::new(self.rate, self.channels)
                            .expect("runtime format"),
                        voice.gain,
                    )
                    .expect("active voice position");
                for (c, value) in lanes[voice.lane.index()]
                    .iter_mut()
                    .take(channels)
                    .enumerate()
                {
                    let contribution = samples[c];
                    *value += contribution;
                    if contribution != 0. {
                        voice_ids[voice.lane.index()] = voice_ids[voice.lane.index()].max(voice.id);
                    }
                }
                voice.cursor += channels;
            }
            for (scene, audition) in lanes[0].iter().zip(&lanes[1]).take(channels) {
                if (scene + audition).abs() > 1.0 {
                    report.clipped_samples += 1;
                }
            }
            write(frame, lanes[0], lanes[1], voice_ids);
            for voice in &mut self.voices {
                if voice.as_ref().is_some_and(|v| {
                    let sound = v.effective.sound.as_ref().expect("audible voice");
                    (v.cursor / channels) as u64 * u64::from(sound.rate)
                        >= (sound.samples().len() / usize::from(sound.channels)) as u64
                            * u64::from(self.rate)
                }) {
                    *voice = None;
                }
            }
            self.mix_frame += 1;
            if self.scene_running {
                self.scene_frame += 1;
            }
        }
        Ok(report)
    }
    /// Product output-block pools use the same budget as prepared voices.
    /// mix(&mut [f32]) remains the allocation-free borrowed-slice DSP entry.
    pub fn mix_buffer(&mut self, output: &mut crate::PcmBuffer) -> Result<MixReport, RuntimeError> {
        if !output.budget().same_pool(&self.budget) {
            return Err(RuntimeError::InvalidPrepared);
        }
        self.mix(
            output
                .samples_mut()
                .map_err(|_| RuntimeError::InvalidBlock)?,
        )
    }

    pub fn voices(&self) -> impl Iterator<Item = VoiceInfo> + '_ {
        self.voices.iter().flatten().map(|v| VoiceInfo {
            id: v.id,
            target: v.target,
            channel: v.lane,
            origin: v.effective.origin,
            cursor: v.cursor,
        })
    }
    pub fn queued(&self) -> usize {
        self.events.len()
    }
    pub fn mix_frame(&self) -> u64 {
        self.mix_frame
    }
    pub fn scene_frame(&self) -> u64 {
        self.scene_frame
    }
    /// Experimental one-way execution choice. Existing bindings/preparation,
    /// trigger validation and cancellation remain authoritative here. Switching
    /// an active owner mixer would duplicate or rewind playback, so is rejected.
    pub fn enable_voice_handoff(&mut self) -> Result<(), RuntimeError> {
        if self.closed {
            return Err(RuntimeError::Closed);
        }
        if self.external_voices.is_some()
            || self.voices.iter().any(Option::is_some)
            || !self.events.is_empty()
        {
            return Err(RuntimeError::Busy);
        }
        self.external_voices = Some(std::sync::Arc::new(()));
        Ok(())
    }
    pub fn voice_handoff_enabled(&self) -> bool {
        self.external_voices.is_some()
    }
    pub fn is_closed(&self) -> bool {
        self.closed
    }
    /// Owner-side copy-boundary clock, not a device-consumption estimate.
    /// Backward scene movement requires the already frozen seek path.
    pub fn synchronize_handoff_clock(
        &mut self,
        output: u64,
        scene: u64,
    ) -> Result<(), RuntimeError> {
        if self.closed {
            return Err(RuntimeError::Closed);
        }
        if self.external_voices.is_none() {
            return Err(RuntimeError::ExecutionMode);
        }
        if output < self.mix_frame || (self.scene_running && scene < self.scene_frame) {
            return Err(RuntimeError::Stale);
        }
        self.mix_frame = output;
        self.scene_frame = scene;
        Ok(())
    }
    pub fn voice_runtime_key(&self) -> Option<crate::VoiceRuntimeKey> {
        self.external_voices
            .as_ref()
            .map(|key| crate::VoiceRuntimeKey(std::sync::Arc::downgrade(key)))
    }
    pub fn scene_running(&self) -> bool {
        self.scene_running
    }
    pub fn handoff_waiting(&self) -> usize {
        self.voices
            .iter()
            .flatten()
            .filter(|v| v.handoff.is_none())
            .count()
    }
    /// Export once, only after downstream admission has a credit. No future
    /// event clock is implied: this candidate explicitly rejects scheduled
    /// triggers until the independent background coordinator is connected.
    pub fn take_voice_handoff(&mut self) -> Result<Option<crate::VoiceHandoff>, RuntimeError> {
        if self.external_voices.is_none() {
            return Err(RuntimeError::ExecutionMode);
        }
        let Some(v) = self
            .voices
            .iter_mut()
            .flatten()
            .find(|v| v.handoff.is_none())
        else {
            return Ok(None);
        };
        let validity = crate::VoiceValidity::new(v.gain);
        v.handoff = Some(validity.clone());
        Ok(Some(crate::VoiceHandoff {
            info: VoiceInfo {
                id: v.id,
                target: v.target,
                channel: v.lane,
                origin: v.effective.origin,
                cursor: v.cursor,
            },
            sound: v.effective.sound.as_ref().expect("audible voice").clone(),
            validity,
        }))
    }
    /// A late/foreign return cannot retire a new Voice with the same numeric ID.
    pub fn finish_voice_handoff(&mut self, returned: &crate::VoiceHandoff) -> bool {
        for slot in &mut self.voices {
            if slot.as_ref().is_some_and(|v| {
                v.id == returned.info.id
                    && v.handoff
                        .as_ref()
                        .is_some_and(|lease| std::sync::Arc::ptr_eq(&lease.0, &returned.validity.0))
            }) {
                // Normal completion is not cancellation of the buffer just
                // generated. Future native integration must ack at its actual
                // submission boundary and keep separate copy/stream fences.
                slot.as_mut().expect("matched voice").handoff = None;
                *slot = None;
                return true;
            }
        }
        false
    }
    /// Failed transport admission restores only this still-current export.
    /// Copies of the rejected lease are revoked before it becomes retryable.
    pub fn retry_voice_handoff(&mut self, returned: &crate::VoiceHandoff) -> bool {
        for v in self.voices.iter_mut().flatten() {
            if v.id == returned.info.id
                && v.handoff
                    .as_ref()
                    .is_some_and(|lease| std::sync::Arc::ptr_eq(&lease.0, &returned.validity.0))
            {
                returned.validity.revoke();
                v.handoff = None;
                return true;
            }
        }
        false
    }
    pub fn shutdown(&mut self) {
        self.closed = true;
        self.device_ready = false;
        self.scene_running = false;
        self.events.clear();
        self.voices.fill_with(|| None);
        self.bindings.clear();
    }
}
