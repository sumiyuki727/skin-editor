use super::{EditorUi, ProjectSession, SkinProject};

/// Optional synchronous diagnostic boundaries. Values contain no project,
/// input, GPU handles or clock. The desktop observer owns any bounded storage.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiPhase {
    Prelude,
    SceneControls,
    LayersAssets,
    PropertiesHistory,
    Canvas,
    AudioPanel,
    SettingsDiagnostics,
    ProjectSnapshot,
    SceneWidgets,
    SceneReadout,
    AudioDevice,
    AudioStatus,
    AudioLibrary,
    AudioComponents,
    AudioProperties,
    AudioReadiness,
    AudioActions,
}

pub(crate) fn phase(observer: &mut Option<&mut dyn FnMut(UiPhase)>, value: UiPhase) {
    if let Some(observer) = observer {
        observer(value);
    }
}

impl EditorUi {
    pub(crate) fn observed_project(
        session: &ProjectSession,
        observer: &mut Option<&mut dyn FnMut(UiPhase)>,
        resume: UiPhase,
    ) -> SkinProject {
        phase(observer, UiPhase::ProjectSnapshot);
        let project = Self::project(session);
        phase(observer, resume);
        project
    }
}
