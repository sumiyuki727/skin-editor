//! Headless project owner and owned queries. Commands arrive in P1-2.
#![forbid(unsafe_code)]

use skin_editor_domain::{ProjectId, SkinProject};

/// Current version of the owned application query values.
pub const API_VERSION: u16 = 1;

/// A structured lifecycle validation error, usable without UI types.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SessionError {
    /// Zero is reserved and cannot identify a project.
    InvalidProjectId,
}

/// An owned query value, independent of mutable session storage.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectInfo {
    /// Version of this query contract, not the future project file format.
    pub api_version: u16,
    /// Stable resolved project identity.
    pub project_id: u128,
}

/// An owned, validated snapshot; this is not a project-replacement request.
#[derive(Clone, Debug, PartialEq)]
pub struct ProjectSnapshot {
    pub api_version: u16,
    pub project: SkinProject,
}

/// Owns the active validated project. No whole-state replacement is exposed.
///
/// Adapters cannot obtain or replace its internal storage:
/// ```compile_fail
/// use skin_editor_application::ProjectSession;
/// let mut session = ProjectSession::new(7).unwrap();
/// session.project = session.project;
/// ```
pub struct ProjectSession {
    project: SkinProject,
}

impl ProjectSession {
    /// Creates an empty project root; this does not load external content.
    pub fn new(resolved_project_id: u128) -> Result<Self, SessionError> {
        let identity = ProjectId::new(resolved_project_id).ok_or(SessionError::InvalidProjectId)?;
        Ok(Self {
            project: SkinProject::empty(identity),
        })
    }

    /// Returns a detached value; editing the returned value cannot change state.
    pub fn project_info(&self) -> ProjectInfo {
        ProjectInfo {
            api_version: API_VERSION,
            project_id: self.project.id().get(),
        }
    }

    pub fn snapshot(&self) -> ProjectSnapshot {
        ProjectSnapshot {
            api_version: API_VERSION,
            project: self.project.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{API_VERSION, ProjectSession, SessionError};

    #[test]
    fn rejects_an_invalid_session_identity() {
        assert!(matches!(
            ProjectSession::new(0),
            Err(SessionError::InvalidProjectId)
        ));
    }

    #[test]
    fn queries_keep_the_version_and_identity() {
        let first = ProjectSession::new(7).unwrap();
        let other = ProjectSession::new(8).unwrap();
        assert_eq!(first.project_info().api_version, API_VERSION);
        assert_eq!(first.project_info().project_id, 7);
        assert_ne!(first.project_info(), other.project_info());
        assert_eq!(first.project_info(), first.project_info());
    }

    #[test]
    fn modifying_a_query_value_cannot_modify_the_active_session() {
        let session = ProjectSession::new(7).unwrap();
        let mut view = session.project_info();
        view.project_id = 100;
        view.api_version = 999;
        assert_ne!(view, session.project_info());
        assert_eq!(session.project_info().project_id, 7);
        assert_eq!(session.project_info().api_version, API_VERSION);
    }

    #[test]
    fn new_session_owns_a_valid_empty_project() {
        let session = ProjectSession::new(7).unwrap();
        let snapshot = session.snapshot();
        assert_eq!(snapshot.api_version, API_VERSION);
        assert_eq!(snapshot.project.id().get(), 7);
        assert_eq!(snapshot.project.documents().len(), 0);
        assert_eq!(snapshot.project.assets().iter().len(), 0);
    }

    #[test]
    fn snapshot_values_are_detached_even_for_populated_project_content() {
        use skin_editor_domain::{
            ConfigInterpretation, ConfigRecord, DocumentId, ElementDocument, LayerTree, PixelSize,
            ProjectDefinition, ProjectId, SkinConfiguration, SkinProject,
        };
        let mut definition = ProjectDefinition::empty(ProjectId::new(7).unwrap());
        definition.documents.push(ElementDocument::new(
            DocumentId::new(10).unwrap(),
            PixelSize::new(8, 8).unwrap(),
            LayerTree::default(),
        ));
        definition.configuration = SkinConfiguration::new(vec![
            ConfigRecord::new(
                b"unknown: preserve\r\n".to_vec(),
                ConfigInterpretation::Opaque,
            )
            .unwrap(),
        ]);
        // A unit fixture inside the owner, not a public restoration/write API.
        let session = ProjectSession {
            project: SkinProject::from_definition(definition).unwrap(),
        };
        let mut snapshot = session.snapshot();
        assert_eq!(snapshot.project.documents().len(), 1);
        snapshot.api_version = 999;
        snapshot.project = SkinProject::empty(ProjectId::new(99).unwrap());
        let actual = session.snapshot();
        assert_eq!(actual.api_version, API_VERSION);
        assert_eq!(actual.project.id().get(), 7);
        assert_eq!(actual.project.documents().len(), 1);
        assert_eq!(
            actual.project.configuration().to_bytes(),
            b"unknown: preserve\r\n"
        );
    }
}
