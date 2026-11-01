//! Read-only sound selection. Playback controls never enter project history.
use crate::{DraftId, DraftState, ExpectedState, HistorySessionId, ProjectSession};
use skin_editor_domain::SkinProject;

pub const SKIN_AUDIO_API_VERSION: u16 = 1;
pub const MAX_AUDIO_DRAFT_REFERENCES: usize = 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SkinAudioSource {
    pub session: HistorySessionId,
    pub state: ExpectedState,
    pub navigation_revision: u64,
    pub draft: Option<DraftState>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SkinAudioError {
    ResourceLimit,
    Backend(String),
}

pub trait SkinAudioPort {
    /// Only retained candidates/bindings/voices, never every historical draft.
    fn retained_drafts(&self) -> Vec<DraftId> {
        Vec::new()
    }
    /// Synchronous control-owner boundary only; never call from a device callback.
    fn synchronize(
        &mut self,
        project: &SkinProject,
        source: SkinAudioSource,
        cancelled_drafts: &[DraftId],
    ) -> Result<(), SkinAudioError>;
}

impl ProjectSession {
    /// Always selects the actual active draft, otherwise the committed project.
    /// Synchronous adapters call before accepting results/events or mixing.
    /// Detached adapters must synchronize every project control boundary and send
    /// a bounded immutable projection; they never move this session to a worker.
    pub fn synchronize_audio(&self, port: &mut dyn SkinAudioPort) -> Result<(), SkinAudioError> {
        let retained = port.retained_drafts();
        if retained.len() > MAX_AUDIO_DRAFT_REFERENCES {
            return Err(SkinAudioError::ResourceLimit);
        }
        let cancelled: Vec<_> = retained
            .into_iter()
            .filter(|id| self.cancelled_drafts.contains(id))
            .collect();
        let (project, draft) = match &self.draft {
            Some(draft) => (&draft.project, Some(draft.state)),
            None => (&self.project, None),
        };
        port.synchronize(
            project,
            SkinAudioSource {
                session: self.session_id,
                state: self.state,
                navigation_revision: self.navigation_revision,
                draft,
            },
            &cancelled,
        )
    }
}
