//! Application's read-only source -> device-free sound runtime adapter.
use crate::{
    Admission, AudioPreparationQueue, BlobStore, FinishedPreparation, PreparationError,
    PreparationUsage, RunningPreparation,
};
use skin_editor_application::{
    DraftId, ProjectSession, SkinAudioError, SkinAudioPort, SkinAudioSource, values::SkinProject,
};
use skin_editor_audio::{
    BuildRequest, Completion, MAX_BINDINGS, Origin, PcmBudget, PrepareFailure, PreparedSound,
    Runtime, RuntimeError, SoundPlan,
};

pub struct SkinAudioController {
    runtime: Runtime,
    preparations: AudioPreparationQueue,
    builds: Vec<BuildRequest>,
    last_origin: Option<Origin>,
    output_invalidation: u64,
}
impl SkinAudioController {
    pub fn new(rate: u32, channels: u16) -> Result<Self, RuntimeError> {
        Self::with_budget(rate, channels, PcmBudget::default())
    }
    pub fn with_budget(rate: u32, channels: u16, budget: PcmBudget) -> Result<Self, RuntimeError> {
        let runtime = Runtime::with_budget(rate, channels, budget.clone())?;
        Ok(Self {
            runtime,
            preparations: AudioPreparationQueue::new(budget),
            builds: Vec::new(),
            last_origin: None,
            output_invalidation: 0,
        })
    }
    /// Control owner only. Audio callbacks must not borrow a ProjectSession.
    /// A2 must transfer prepared control values into its callback boundary.
    pub fn runtime(&mut self) -> &mut Runtime {
        &mut self.runtime
    }
    pub fn output_invalidation(&self) -> u64 {
        self.output_invalidation
    }
    /// Replaced on a newer selection, never an unbounded job backlog.
    pub fn take_builds(&mut self) -> Vec<BuildRequest> {
        std::mem::take(&mut self.builds)
    }

    pub fn admit_builds(
        &mut self,
        session: &ProjectSession,
        store: &BlobStore,
    ) -> Result<Vec<BuildAdmission>, SkinAudioError> {
        session.synchronize_audio(self)?;
        self.admit_selected(store)
    }
    pub(crate) fn admit_selected(
        &mut self,
        store: &BlobStore,
    ) -> Result<Vec<BuildAdmission>, SkinAudioError> {
        let mut results = Vec::new();
        for request in self.take_builds() {
            let admission = store
                .get(request.plan.source)
                .ok_or(PreparationError::MissingResource)
                .and_then(|raw| self.preparations.submit(request, raw));
            let status = match admission {
                Ok(Admission::Queued) => Ok(AdmissionState::Queued),
                Ok(Admission::AlreadyQueued) => Ok(AdmissionState::AlreadyQueued),
                Ok(Admission::Cached(sound)) => self
                    .runtime
                    .complete(request, Ok(sound))
                    .map(AdmissionState::Cached)
                    .map_err(|_| PreparationError::Stale),
                Err(error) => {
                    let failure = match error {
                        PreparationError::Unsupported => PrepareFailure::Unsupported,
                        PreparationError::Busy => PrepareFailure::Busy,
                        PreparationError::ResourceLimit => PrepareFailure::ResourceLimit,
                        PreparationError::Cancelled => PrepareFailure::Cancelled,
                        _ => PrepareFailure::MetadataMismatch,
                    };
                    self.runtime
                        .complete(request, Err(failure))
                        .map_err(|e| SkinAudioError::Backend(format!("{e:?}")))?;
                    Err(error)
                }
            };
            results.push(BuildAdmission { request, status });
        }
        Ok(results)
    }
    pub(crate) fn request_library(
        &mut self,
        plan: SoundPlan,
        store: &BlobStore,
    ) -> Result<(), SkinAudioError> {
        let retained = self.runtime.library_sound();
        self.preparations.cancel_library();
        let request = self
            .runtime
            .request_library(plan)
            .map_err(|e| SkinAudioError::Backend(format!("{e:?}")))?;
        self.builds.push(request);
        self.admit_selected(store)?;
        drop(retained);
        Ok(())
    }
    pub(crate) fn cancel_library(&mut self) {
        self.preparations.cancel_library();
    }
    pub fn start_preparation(&mut self) -> Option<RunningPreparation> {
        self.preparations.start_next()
    }
    pub fn retry_preparation(
        &mut self,
        session: &ProjectSession,
        component: skin_editor_application::values::AudioComponentId,
    ) -> Result<(), SkinAudioError> {
        session.synchronize_audio(self)?;
        self.retry_selected(component)
    }
    pub(crate) fn retry_selected(
        &mut self,
        component: skin_editor_application::values::AudioComponentId,
    ) -> Result<(), SkinAudioError> {
        let request = self
            .runtime
            .retry(component)
            .map_err(|e| SkinAudioError::Backend(format!("{e:?}")))?;
        self.builds.push(request);
        Ok(())
    }
    pub fn preparation_usage(&mut self) -> PreparationUsage {
        self.preparations.usage()
    }
    pub fn finish_preparation(
        &mut self,
        session: &ProjectSession,
        finished: FinishedPreparation,
    ) -> Result<Completion, SkinAudioError> {
        session.synchronize_audio(self)?;
        self.finish_selected(finished)
    }
    pub(crate) fn finish_selected(
        &mut self,
        finished: FinishedPreparation,
    ) -> Result<Completion, SkinAudioError> {
        let (request, result) = self
            .preparations
            .finish(finished)
            .map_err(|e| SkinAudioError::Backend(format!("{e:?}")))?;
        self.runtime
            .complete(request, result)
            .map_err(|e| SkinAudioError::Backend(format!("{e:?}")))
    }
    pub fn close(&mut self) {
        self.builds.clear();
        self.preparations.close();
        self.runtime.shutdown();
    }

    /// Refresh selection before accepting a worker result, including when no
    /// UI refresh occurred between cancellation/navigation and completion.
    pub fn accept_prepared(
        &mut self,
        session: &ProjectSession,
        request: BuildRequest,
        result: Result<PreparedSound, PrepareFailure>,
    ) -> Result<Completion, SkinAudioError> {
        session.synchronize_audio(self)?;
        self.runtime
            .complete(request, result)
            .map_err(|e| SkinAudioError::Backend(format!("{e:?}")))
    }
}
impl SkinAudioController {
    pub(crate) fn select(
        &mut self,
        origin: Origin,
        plans: &[SoundPlan],
        cancelled_drafts: &[DraftId],
    ) -> Result<(), SkinAudioError> {
        if !cancelled_drafts.is_empty()
            || self.last_origin.is_some_and(|old| {
                old.session != origin.session
                    || old.navigation_revision != origin.navigation_revision
            })
        {
            self.output_invalidation = self
                .output_invalidation
                .checked_add(1)
                .ok_or(SkinAudioError::ResourceLimit)?;
        }
        self.last_origin = Some(origin);
        self.preparations.cancel_outdated(origin);
        let changed = self
            .runtime
            .synchronize_cancelled(
                origin,
                plans,
                &cancelled_drafts.iter().map(|d| d.get()).collect::<Vec<_>>(),
            )
            .map_err(|e| match e {
                RuntimeError::ResourceLimit => SkinAudioError::ResourceLimit,
                _ => SkinAudioError::Backend(format!("{e:?}")),
            })?;
        // An unchanged sync has no new jobs and must not discard untaken jobs.
        // Their complete origin/serial is still validated when they arrive.
        if !changed.is_empty() || self.builds.first().is_some_and(|b| b.origin != origin) {
            self.builds = changed;
        }
        Ok(())
    }
}

impl SkinAudioPort for SkinAudioController {
    fn retained_drafts(&self) -> Vec<DraftId> {
        self.runtime
            .retained_drafts()
            .into_iter()
            .filter_map(DraftId::new)
            .collect()
    }
    fn synchronize(
        &mut self,
        project: &SkinProject,
        source: SkinAudioSource,
        cancelled_drafts: &[DraftId],
    ) -> Result<(), SkinAudioError> {
        let origin = Origin {
            session: source.session.get(),
            node: source.state.head.get(),
            revision: source.state.revision,
            navigation_revision: source.navigation_revision,
            draft: source.draft.map(|d| (d.id.get(), d.revision)),
        };
        self.preparations.cancel_outdated(origin);
        if project.audio_components().len() > MAX_BINDINGS {
            self.runtime
                .invalidate_drafts(&cancelled_drafts.iter().map(|d| d.get()).collect::<Vec<_>>());
            return Err(SkinAudioError::ResourceLimit);
        }
        let plans = project
            .audio_components()
            .map(|c| SoundPlan::build(project, c.id()))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| SkinAudioError::Backend(format!("{e:?}")))?;
        self.select(origin, &plans, cancelled_drafts)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdmissionState {
    Queued,
    AlreadyQueued,
    Cached(Completion),
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BuildAdmission {
    pub request: BuildRequest,
    pub status: Result<AdmissionState, PreparationError>,
}
