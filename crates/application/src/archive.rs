//! Storage ports and detached restore values; no paths, SQLite or file handles.
use crate::values::{BlobRef, ModelError, ProjectId, SkinProject};
use crate::{
    BranchCoordinate, CaptureCancellation, CaptureError, CapturedAdmission, CapturedOperation,
    CommitId, HistoryCapture, PersistentHistoryIdentity, PersistentNodeId, ResourceError,
    SessionLimits,
};
use std::sync::Arc;

pub const ARCHIVE_PORT_VERSION: u16 = 1;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SaveOrigin {
    Manual,
    Automatic,
    Exit,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RestoreSelection {
    Manual,
    Latest,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArchiveReceipt {
    pub commit: CommitId,
    pub generation: u64,
    pub head: PersistentNodeId,
    pub origin: SaveOrigin,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ArchiveWriteOutcome {
    Committed(ArchiveReceipt),
    Replayed(ArchiveReceipt),
    NoChange(ArchiveReceipt),
}
pub struct ArchiveWriteRequest {
    pub version: u16,
    pub commit: CommitId,
    pub expected: Option<CommitId>,
    pub origin: SaveOrigin,
    pub cancellation: CaptureCancellation,
}
pub trait ProjectArchive {
    type Error;
    fn write_capture(
        &mut self,
        capture: &mut HistoryCapture,
        request: ArchiveWriteRequest,
    ) -> Result<ArchiveWriteOutcome, Self::Error>;
    fn restore_candidate(
        &mut self,
        selection: RestoreSelection,
        cancel: CaptureCancellation,
    ) -> Result<crate::RestoreCandidate, Self::Error>;
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OriginalNodeRecord {
    pub record_version: u16,
    pub state: [u8; 32],
    pub action: Vec<u8>,
    pub checksum: [u8; 32],
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArchiveTag {
    pub node: PersistentNodeId,
    pub name: String,
    pub value: String,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArchiveExtension {
    pub name: String,
    pub bytes: Vec<u8>,
}
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ArchiveMetadata {
    pub last_commit: Option<ArchiveReceipt>,
    pub manual: Option<ArchiveReceipt>,
    pub protected: Option<ArchiveReceipt>,
    pub manual_marks: Vec<(CommitId, PersistentNodeId)>,
    pub tags: Vec<ArchiveTag>,
    pub extensions: Vec<ArchiveExtension>,
}
impl ArchiveMetadata {
    pub(crate) fn logical_bytes(&self) -> Result<usize, CaptureError> {
        let mut c = crate::capture_cost::Cost::default();
        c.count(self.manual_marks.len(), 64)?;
        for t in &self.tags {
            c.add(64)?;
            c.add(t.name.len())?;
            c.add(t.value.len())?;
        }
        for e in &self.extensions {
            c.add(64)?;
            c.add(e.name.len())?;
            c.add(e.bytes.len())?;
        }
        Ok(c.bytes())
    }
}
#[derive(Clone, Debug)]
pub struct RestoreHeader {
    pub node_count: usize,
    pub project: ProjectId,
    pub identity: PersistentHistoryIdentity,
    pub root: PersistentNodeId,
    pub head: PersistentNodeId,
    pub next_alias: u64,
    pub next_branch: u64,
    pub admissions: Vec<CapturedAdmission>,
    pub redo_preferences: Vec<(PersistentNodeId, PersistentNodeId)>,
    pub originals: Vec<BlobRef>,
    pub metadata: Arc<ArchiveMetadata>,
}
#[derive(Clone, Debug)]
pub struct RestoreNode {
    pub id: PersistentNodeId,
    pub parent: Option<PersistentNodeId>,
    pub alias: u64,
    pub coordinate: BranchCoordinate,
    pub depth: u64,
    pub operation: Arc<CapturedOperation>,
    pub origin: Arc<OriginalNodeRecord>,
    pub project: SkinProject,
}
#[derive(Clone, Copy, Debug)]
pub struct RestoreLimits {
    pub nodes: usize,
    pub object_ids: usize,
    pub metadata_bytes: usize,
    pub original_bytes: u64,
}
impl Default for RestoreLimits {
    fn default() -> Self {
        Self {
            nodes: 65536,
            object_ids: 262144,
            metadata_bytes: 32 * 1024 * 1024,
            original_bytes: 128 * 1024 * 1024,
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum RestoreError {
    Invalid(&'static str),
    Limit(&'static str),
    Cancelled,
    Resource(BlobRef, ResourceError),
    Model(ModelError),
}
pub(crate) fn restore_cancel(c: &CaptureCancellation) -> Result<(), RestoreError> {
    if c.is_cancelled() {
        Err(RestoreError::Cancelled)
    } else {
        Ok(())
    }
}
pub(crate) fn session_limits(l: SessionLimits) -> bool {
    l.commands_per_request > 0 && l.retained_requests > 0 && l.history_nodes > 0
}
