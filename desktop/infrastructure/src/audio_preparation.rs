//! Admission/control owner. A2 can execute the single leased job on a worker.
use crate::{BlobError, BlobStore, RawBlob};
use skin_editor_application::values::AudioMetadata;
use skin_editor_audio::{
    BuildRequest, Origin, PcmBudget, PcmReservation, PcmUsage, PrepareFailure, PreparedKey,
    PreparedSound, PreparedWeak,
};
use std::collections::{BTreeMap, VecDeque};
use std::sync::{
    Arc, Weak,
    atomic::{AtomicBool, Ordering},
};

use std::time::{Duration, Instant};

pub const MAX_PENDING_PREPARATIONS: usize = 4;
const MAX_PCM_CACHE_KEYS: usize = 256;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PreparationError {
    Unsupported,
    Busy,
    Closed,
    ResourceLimit,
    MetadataMismatch,
    Cancelled,
    Stale,
    MissingResource,
}
pub trait AudioDecoder {
    /// Trusted codec adapter probes actual immutable bytes, not UI declarations.
    fn probe(&self, bytes: &[u8]) -> Result<AudioMetadata, PrepareFailure>;
    /// Fill the preallocated source-format output and return interleaved samples written.
    fn decode_into(
        &mut self,
        bytes: &[u8],
        metadata: AudioMetadata,
        output: &mut [f32],
        cancel: &Cancellation,
    ) -> Result<usize, PrepareFailure>;
}
impl BlobStore {
    /// Explicit supported-audio import. Unsupported data can separately be
    /// staged as Opaque without being advertised as playable Audio.
    pub fn stage_audio(
        &self,
        bytes: Vec<u8>,
        decoder: &dyn AudioDecoder,
    ) -> Result<RawBlob, BlobError> {
        let raw = self.stage_bytes(bytes)?;
        let metadata = decoder
            .probe(raw.bytes())
            .map_err(|_| BlobError::Unsupported)?;
        raw.attest_audio(metadata)?;
        Ok(raw)
    }
}
struct JobLife {
    cancelled: AtomicBool,
}
pub struct Cancellation(Arc<JobLife>);
impl Cancellation {
    pub fn is_cancelled(&self) -> bool {
        self.0.cancelled.load(Ordering::Acquire)
    }
}
struct Pending {
    request: BuildRequest,
    raw: RawBlob,
    reservation: PcmReservation,
    admitted: Instant,
}
struct Active {
    id: u64,
    request: BuildRequest,
    life: Weak<JobLife>,
}
pub struct RunningPreparation {
    id: u64,
    job: Pending,
    life: Arc<JobLife>,
}
pub struct FinishedPreparation {
    id: u64,
    request: BuildRequest,
    result: Result<PreparedSound, PrepareFailure>,
    life: Arc<JobLife>,
    observation: PreparationObservation,
    finished: Instant,
}
/// One bounded observation, never collected or queried by the native callback.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PreparationObservation {
    pub request: BuildRequest,
    pub queue_wait: Duration,
    pub decode_and_allocate: Duration,
    pub process: Duration,
    pub result_handoff: Duration,
}
impl FinishedPreparation {
    pub fn observation(&self) -> PreparationObservation {
        PreparationObservation {
            result_handoff: self.finished.elapsed(),
            ..self.observation
        }
    }
}
impl RunningPreparation {
    pub fn request(&self) -> BuildRequest {
        self.job.request
    }
    pub fn execute(self, decoder: &mut dyn AudioDecoder) -> FinishedPreparation {
        let Self { id, job, life } = self;
        let request = job.request;
        let started = Instant::now();
        let mut observation = PreparationObservation {
            request,
            queue_wait: started.duration_since(job.admitted),
            decode_and_allocate: Duration::ZERO,
            process: Duration::ZERO,
            result_handoff: Duration::ZERO,
        };
        let cancel = Cancellation(life.clone());
        let result = (|| {
            if cancel.is_cancelled() {
                return Err(PrepareFailure::Cancelled);
            }
            let metadata = decoder.probe(job.raw.bytes())?;
            if Some(metadata) != request.plan.metadata {
                return Err(PrepareFailure::MetadataMismatch);
            }
            let mut pcm = job.reservation.allocate();
            let written = decoder.decode_into(
                job.raw.bytes(),
                metadata,
                pcm.samples_mut().map_err(|_| PrepareFailure::InvalidPcm)?,
                &cancel,
            )?;
            if cancel.is_cancelled() {
                return Err(PrepareFailure::Cancelled);
            }
            if written != pcm.samples().len() {
                return Err(PrepareFailure::InvalidPcm);
            }
            observation.decode_and_allocate = started.elapsed();
            let processing = Instant::now();
            let sound = PreparedSound::from_decoded(request.plan, pcm);
            observation.process = processing.elapsed();
            sound
        })();
        FinishedPreparation {
            id,
            request,
            result,
            life,
            observation,
            finished: Instant::now(),
        }
    }
}
pub enum Admission {
    Queued,
    Cached(PreparedSound),
    AlreadyQueued,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PreparationUsage {
    pub active: usize,
    pub pending: usize,
    pub cached: usize,
    pub pcm: PcmUsage,
}
pub type PreparationResult = (BuildRequest, Result<PreparedSound, PrepareFailure>);
pub struct AudioPreparationQueue {
    budget: PcmBudget,
    pending: VecDeque<Pending>,
    active: Option<Active>,
    cache: BTreeMap<PreparedKey, PreparedWeak>,
    next_id: u64,
    closed: bool,
}
impl AudioPreparationQueue {
    pub fn new(budget: PcmBudget) -> Self {
        Self {
            budget,
            pending: VecDeque::with_capacity(MAX_PENDING_PREPARATIONS),
            active: None,
            cache: BTreeMap::new(),
            next_id: 1,
            closed: false,
        }
    }
    fn prune(&mut self) {
        self.cache.retain(|_, v| v.is_alive());
        if self
            .active
            .as_ref()
            .is_some_and(|a| a.life.strong_count() == 0)
        {
            self.active = None;
        }
    }
    pub fn usage(&mut self) -> PreparationUsage {
        self.prune();
        PreparationUsage {
            active: usize::from(self.active.is_some()),
            pending: self.pending.len(),
            cached: self.cache.len(),
            pcm: self.budget.usage(),
        }
    }
    pub fn submit(
        &mut self,
        request: BuildRequest,
        raw: RawBlob,
    ) -> Result<Admission, PreparationError> {
        if self.closed {
            return Err(PreparationError::Closed);
        }
        if request.plan.metadata.is_none() {
            return Err(PreparationError::Unsupported);
        }
        if raw.reference() != request.plan.source || raw.audio_metadata() != request.plan.metadata {
            return Err(PreparationError::MetadataMismatch);
        }
        self.prune();
        if self.pending.iter().any(|p| p.request == request)
            || self.active.as_ref().is_some_and(|a| {
                a.request == request
                    && a.life
                        .upgrade()
                        .is_some_and(|l| !l.cancelled.load(Ordering::Acquire))
            })
        {
            return Ok(Admission::AlreadyQueued);
        }
        let key = PreparedKey::new(request.plan).map_err(|_| PreparationError::MetadataMismatch)?;
        if let Some(sound) = self
            .cache
            .get(&key)
            .and_then(|c| c.upgrade_for(request.plan, &self.budget))
        {
            return Ok(Admission::Cached(sound));
        }
        // Preview candidates coalesce per component; every new caller still
        // receives Queued/Cached/error. Explicit file imports use another queue.
        self.cancel_target(request.plan.target);
        if self.pending.len() == MAX_PENDING_PREPARATIONS {
            return Err(PreparationError::Busy);
        }
        let metadata = request
            .plan
            .metadata
            .ok_or(PreparationError::MetadataMismatch)?;
        let samples = metadata
            .frames()
            .checked_mul(u64::from(metadata.channels()))
            .and_then(|n| usize::try_from(n).ok())
            .filter(|n| *n <= skin_editor_audio::MAX_SOUND_SAMPLES)
            .ok_or(PreparationError::ResourceLimit)?;
        let reservation = self
            .budget
            .reserve(samples)
            .map_err(|_| PreparationError::ResourceLimit)?;
        self.pending.push_back(Pending {
            request,
            raw,
            reservation,
            admitted: Instant::now(),
        });
        Ok(Admission::Queued)
    }
    pub fn start_next(&mut self) -> Option<RunningPreparation> {
        self.prune();
        if self.closed || self.active.is_some() {
            return None;
        }
        let id = self.next_id;
        let next = id.checked_add(1)?;
        let job = self.pending.pop_front()?;
        let life = Arc::new(JobLife {
            cancelled: AtomicBool::new(false),
        });
        self.active = Some(Active {
            id,
            request: job.request,
            life: Arc::downgrade(&life),
        });
        self.next_id = next;
        Some(RunningPreparation { id, job, life })
    }
    pub fn finish(
        &mut self,
        finished: FinishedPreparation,
    ) -> Result<PreparationResult, PreparationError> {
        if self.active.as_ref().is_none_or(|a| {
            a.id != finished.id
                || a.request != finished.request
                || !Weak::ptr_eq(&a.life, &Arc::downgrade(&finished.life))
        }) {
            return Err(PreparationError::Stale);
        }
        self.active = None;
        let result = if self.closed || finished.life.cancelled.load(Ordering::Acquire) {
            Err(PrepareFailure::Cancelled)
        } else {
            finished.result
        };
        if let Ok(sound) = &result {
            self.prune();
            if self.cache.len() < MAX_PCM_CACHE_KEYS {
                self.cache.insert(sound.cache_key(), sound.downgrade());
            }
        }
        Ok((finished.request, result))
    }
    pub fn cancel_component(&mut self, component: impl Into<skin_editor_audio::SoundTarget>) {
        self.cancel_target(component.into());
    }
    pub fn cancel_target(&mut self, target: skin_editor_audio::SoundTarget) {
        self.pending.retain(|p| p.request.plan.target != target);
        if let Some(active) = self
            .active
            .as_ref()
            .filter(|a| a.request.plan.target == target)
            .and_then(|a| a.life.upgrade())
        {
            active.cancelled.store(true, Ordering::Release);
        }
    }
    pub fn cancel_library(&mut self) {
        self.pending.retain(|p| {
            !matches!(
                p.request.plan.target,
                skin_editor_audio::SoundTarget::Library(_)
            )
        });
        if let Some(life) = self
            .active
            .as_ref()
            .filter(|a| {
                matches!(
                    a.request.plan.target,
                    skin_editor_audio::SoundTarget::Library(_)
                )
            })
            .and_then(|a| a.life.upgrade())
        {
            life.cancelled.store(true, Ordering::Release);
        }
    }
    pub fn cancel_outdated(&mut self, origin: Origin) {
        self.pending.retain(|p| p.request.origin == origin);
        if let Some(active) = self
            .active
            .as_ref()
            .filter(|a| a.request.origin != origin)
            .and_then(|a| a.life.upgrade())
        {
            active.cancelled.store(true, Ordering::Release);
        }
    }
    pub fn close(&mut self) {
        self.closed = true;
        self.pending.clear();
        self.cache.clear();
        if let Some(life) = self.active.as_ref().and_then(|a| a.life.upgrade()) {
            life.cancelled.store(true, Ordering::Release);
        }
    }
}
impl Drop for AudioPreparationQueue {
    fn drop(&mut self) {
        self.close();
    }
}
