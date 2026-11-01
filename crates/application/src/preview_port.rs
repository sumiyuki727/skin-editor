//! Value-only application ports. No runtime, GPU handles or project writer.
use crate::{BlobLease, DraftState, ExpectedState, HistorySessionId, ProjectSession};
use skin_editor_domain::{BlobRef, DocumentId, PixelSize, SkinProject};
use std::sync::Arc;

pub const PREVIEW_API_VERSION: u16 = 2;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PreviewTarget {
    Document(DocumentId),
    Components,
    /// Read-only editing workspace, including documents outside ComponentTree.
    Workspace,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PreviewContext {
    pub target: PreviewTarget,
    pub draft: Option<DraftState>,
    pub size: [u32; 2],
    pub viewport_generation: u64,
    pub scene: u8,
    pub simulation_micros: u64,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FrameSource {
    pub session: HistorySessionId,
    pub state: ExpectedState,
    pub context: PreviewContext,
}
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct FrameKey(pub u64);
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FrameStatus {
    Pending,
    Ready,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FrameDescriptor {
    pub api_version: u16,
    pub key: FrameKey,
    pub source: FrameSource,
    pub status: FrameStatus,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PreviewError {
    InvalidViewport,
    InvalidDraft,
    MissingDocument,
    ResourceLimit,
    Backend(String),
}
pub trait PreviewPort {
    fn request(
        &mut self,
        project: &SkinProject,
        source: FrameSource,
    ) -> Result<FrameDescriptor, PreviewError>;
}
impl ProjectSession {
    pub fn request_preview(
        &self,
        port: &mut dyn PreviewPort,
        context: PreviewContext,
    ) -> Result<FrameDescriptor, PreviewError> {
        if context.size.contains(&0) {
            return Err(PreviewError::InvalidViewport);
        }
        let project = if let Some(expected) = context.draft {
            &self
                .expected_draft(expected)
                .map_err(|_| PreviewError::InvalidDraft)?
                .project
        } else {
            &self.project
        };
        if let PreviewTarget::Document(id) = context.target
            && project.document(id).is_none()
        {
            return Err(PreviewError::MissingDocument);
        }
        port.request(
            project,
            FrameSource {
                session: self.session_id,
                state: self.state,
                context,
            },
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ImportError {
    Unsupported,
    ResourceLimit,
    InvalidImage,
    Io(String),
}
pub struct StagedRaster {
    pub blob: BlobRef,
    pub size: PixelSize,
    pub label: String,
    pub lease: Arc<dyn BlobLease>,
}
pub trait RasterImportPort {
    /// Explicit local source only; the path never becomes a project value.
    fn stage_png(&mut self, source: &str) -> Result<StagedRaster, ImportError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Port(bool);
    #[test]
    fn workspace_projection_is_read_only_and_uses_version_two() {
        let session = ProjectSession::new(1, 2).unwrap();
        let before = session.snapshot();
        let mut port = Port(false);
        let frame = session
            .request_preview(
                &mut port,
                PreviewContext {
                    target: PreviewTarget::Workspace,
                    draft: None,
                    size: [64, 64],
                    viewport_generation: 1,
                    scene: 0,
                    simulation_micros: 0,
                },
            )
            .unwrap();
        assert_eq!(frame.api_version, 2);
        assert!(port.0);
        assert_eq!(session.snapshot(), before);
        assert_eq!(session.history_len(), 1);
    }
    impl PreviewPort for Port {
        fn request(
            &mut self,
            _: &SkinProject,
            source: FrameSource,
        ) -> Result<FrameDescriptor, PreviewError> {
            self.0 = true;
            Ok(FrameDescriptor {
                api_version: PREVIEW_API_VERSION,
                key: FrameKey(1),
                source,
                status: FrameStatus::Ready,
            })
        }
    }
    #[test]
    fn preview_is_scoped_read_only_and_rejects_invalid_requests_before_adapter() {
        let session = ProjectSession::new(1, 2).unwrap();
        let before = session.snapshot();
        let mut port = Port(false);
        let mut context = PreviewContext {
            target: PreviewTarget::Components,
            draft: None,
            size: [64, 64],
            viewport_generation: 1,
            scene: 0,
            simulation_micros: 42,
        };
        let frame = session.request_preview(&mut port, context).unwrap();
        assert_eq!(frame.source.state, before.state);
        assert_eq!(frame.source.session, before.session_id);
        assert_eq!(frame.source.context.simulation_micros, 42);
        assert_eq!(session.snapshot(), before);
        port.0 = false;
        context.size[0] = 0;
        assert_eq!(
            session.request_preview(&mut port, context),
            Err(PreviewError::InvalidViewport)
        );
        assert!(!port.0);
    }
    #[test]
    fn cancelled_draft_token_cannot_select_a_project_for_preview() {
        use crate::{API_VERSION, DraftAction, DraftRequest, RequestId};
        let mut session = ProjectSession::new(1, 2).unwrap();
        let scope = session.project_info().session_id;
        session
            .edit_draft(DraftRequest {
                api_version: API_VERSION,
                session_id: scope,
                request_id: RequestId::new(1).unwrap(),
                action: DraftAction::Begin {
                    expected: session.project_info().state,
                },
            })
            .unwrap();
        let token = session.draft_info(scope).unwrap().unwrap().state;
        let context = PreviewContext {
            target: PreviewTarget::Components,
            draft: Some(token),
            size: [32, 32],
            viewport_generation: 1,
            scene: 0,
            simulation_micros: 0,
        };
        let mut port = Port(false);
        assert_eq!(
            session
                .request_preview(&mut port, context)
                .unwrap()
                .source
                .context
                .draft,
            Some(token)
        );
        session
            .edit_draft(DraftRequest {
                api_version: API_VERSION,
                session_id: scope,
                request_id: RequestId::new(2).unwrap(),
                action: DraftAction::Cancel { expected: token },
            })
            .unwrap();
        port.0 = false;
        assert_eq!(
            session.request_preview(&mut port, context),
            Err(PreviewError::InvalidDraft)
        );
        assert!(!port.0);
    }
}
