//! Headless application entry foundation; project editing is not implemented yet.
#![forbid(unsafe_code)]

use skin_editor_domain::ProjectId;

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

/// Owns the active project identity. P1-1 will add validated project content.
///
/// Adapters cannot obtain or replace its internal storage:
/// ```compile_fail
/// use skin_editor_application::ProjectSession;
/// let mut session = ProjectSession::new(7).unwrap();
/// session.identity = session.identity;
/// ```
pub struct ProjectSession {
    identity: ProjectId,
}

impl ProjectSession {
    /// Starts a new identity-only session; this does not load or edit a project.
    pub fn new(resolved_project_id: u128) -> Result<Self, SessionError> {
        let identity = ProjectId::new(resolved_project_id).ok_or(SessionError::InvalidProjectId)?;
        Ok(Self { identity })
    }

    /// Returns a detached value; editing the returned value cannot change state.
    pub fn project_info(&self) -> ProjectInfo {
        ProjectInfo {
            api_version: API_VERSION,
            project_id: self.identity.get(),
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
}
