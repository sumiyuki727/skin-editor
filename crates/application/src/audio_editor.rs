//! Device-free editor values. Adapters own preparation and native lifetimes.
use crate::{ProjectSession, SkinAudioError, SkinAudioPort, SkinAudioSource};
use skin_editor_domain::{AssetId, AssetKind, AudioComponentId, AudioMetadata, BlobRef};
pub const AUDIO_EDITOR_API_VERSION: u16 = 3;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AudioEditorAction {
    Enable,
    Disable,
    SceneRunning(bool),
    Audition(AudioComponentId),
    AuditionAsset(AssetId),
    MonitorVolume(u16),
    StopAudition,
    Retry(AudioComponentId),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AudioEditorError {
    Unavailable,
    Busy,
    MissingComponent,
    MissingAsset,
    ImageAsset,
    InvalidVolume,
    Synchronization(SkinAudioError),
    Backend(String),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AudioPreparationState {
    Pending,
    Ready,
    Failed(String),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AudioComponentState {
    pub component: AudioComponentId,
    pub desired: SkinAudioSource,
    pub effective: Option<SkinAudioSource>,
    pub preparation: AudioPreparationState,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AssetAudition {
    pub source: SkinAudioSource,
    pub asset: AssetId,
    pub blob: BlobRef,
    pub metadata: Option<AudioMetadata>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssetAuditionState {
    pub asset: AssetId,
    pub source: SkinAudioSource,
    pub preparation: AudioPreparationState,
}
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AudioOutputObservation {
    pub candidate: bool,
    pub rate: u32,
    pub channels: u16,
    pub callbacks: u64,
    pub missing_frames: u64,
    pub generated_background: u64,
    pub copied_background: u64,
    pub background_frames: u64,
    pub pcm_bytes: usize,
}
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AudioEditorState {
    /// Callback submissions only, not device consumption or acoustic evidence.
    pub submitted_nonzero_samples: u64,
    pub output: Option<AudioOutputObservation>,
    pub enabled: bool,
    /// Monitoring only, 0..=1000; independent of project gain/export.
    pub monitor_volume: u16,
    pub asset_audition: Option<AssetAuditionState>,
    pub source: Option<SkinAudioSource>,
    pub components: Vec<AudioComponentState>,
    pub frozen: bool,
    pub freezing: bool,
    pub pending_controls: bool,
    pub voices: [usize; 2],
    pub last_audition: Option<(SkinAudioSource, bool)>,
    pub error: Option<String>,
}
pub trait AudioEditorPort: SkinAudioPort {
    fn audition_asset(&mut self, _: AssetAudition) -> Result<(), AudioEditorError> {
        Err(AudioEditorError::Unavailable)
    }
    fn request(&mut self, action: AudioEditorAction) -> Result<(), AudioEditorError>;
    fn poll(&mut self) -> AudioEditorState;
}
#[derive(Default)]
pub struct DisabledAudio;
impl SkinAudioPort for DisabledAudio {
    fn synchronize(
        &mut self,
        _: &skin_editor_domain::SkinProject,
        _: SkinAudioSource,
        _: &[crate::DraftId],
    ) -> Result<(), SkinAudioError> {
        Ok(())
    }
}
impl AudioEditorPort for DisabledAudio {
    fn request(&mut self, _: AudioEditorAction) -> Result<(), AudioEditorError> {
        Err(AudioEditorError::Unavailable)
    }
    fn poll(&mut self) -> AudioEditorState {
        AudioEditorState::default()
    }
}
impl ProjectSession {
    /// A read-only source identity; no project pointers or native handles.
    pub fn skin_audio_source(&self) -> SkinAudioSource {
        SkinAudioSource {
            session: self.session_id,
            state: self.state,
            navigation_revision: self.navigation_revision,
            draft: self.draft.as_ref().map(|d| d.state),
        }
    }
    pub fn control_audio(
        &self,
        port: &mut dyn AudioEditorPort,
        action: AudioEditorAction,
    ) -> Result<(), AudioEditorError> {
        if matches!(action, AudioEditorAction::MonitorVolume(v) if v > 1000) {
            return Err(AudioEditorError::InvalidVolume);
        }
        if matches!(
            action,
            AudioEditorAction::Disable
                | AudioEditorAction::StopAudition
                | AudioEditorAction::MonitorVolume(_)
        ) {
            return port.request(action);
        }
        if action == AudioEditorAction::Enable {
            port.request(action)?;
        }
        self.synchronize_audio(port)
            .map_err(AudioEditorError::Synchronization)?;
        if let AudioEditorAction::Audition(id) | AudioEditorAction::Retry(id) = action {
            let project = self.draft.as_ref().map_or(&self.project, |d| &d.project);
            if project.audio_component(id).is_none() {
                return Err(AudioEditorError::MissingComponent);
            }
        }
        if let AudioEditorAction::AuditionAsset(id) = action {
            let project = self.draft.as_ref().map_or(&self.project, |d| &d.project);
            let asset = project
                .assets()
                .get(id)
                .ok_or(AudioEditorError::MissingAsset)?;
            let metadata = match asset.kind() {
                AssetKind::Audio(m) => Some(*m),
                AssetKind::Opaque => None,
                AssetKind::RasterImage(_) => return Err(AudioEditorError::ImageAsset),
            };
            return port.audition_asset(AssetAudition {
                source: self.skin_audio_source(),
                asset: id,
                blob: asset.blob(),
                metadata,
            });
        }
        if action == AudioEditorAction::Enable {
            Ok(())
        } else {
            port.request(action)
        }
    }
}
