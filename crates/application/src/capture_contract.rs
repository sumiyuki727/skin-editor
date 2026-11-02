use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::{
    BranchCoordinate, DraftId, ExpectedState, HistoryQueryError, HistorySessionId,
    PersistentHistoryIdentity, PersistentNodeId,
};
use skin_editor_domain::{BlobRef, ObjectId, ProjectId, SkinProject};

pub const HISTORY_CAPTURE_VERSION: u16 = 2;

#[derive(Clone, Default)]
pub struct CaptureCancellation(Arc<AtomicBool>);
impl CaptureCancellation {
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}

/// Logical admission costs, not allocator measurements or stable format rules.
#[derive(Clone, Copy, Debug)]
pub struct CaptureLimits {
    pub nodes: usize,
    pub object_ids: usize,
    pub originals: usize,
    pub original_bytes: u64,
    pub retained_metadata_bytes: usize,
    pub state_bytes: usize,
    pub total_state_bytes: usize,
}
impl Default for CaptureLimits {
    fn default() -> Self {
        Self {
            nodes: 65536,
            object_ids: 262144,
            originals: 4096,
            original_bytes: 128 * 1024 * 1024,
            retained_metadata_bytes: 32 * 1024 * 1024,
            state_bytes: 8 * 1024 * 1024,
            total_state_bytes: 512 * 1024 * 1024,
        }
    }
}
impl CaptureLimits {
    pub(crate) fn valid(self) -> bool {
        self.nodes > 0
            && self.object_ids > 0
            && self.originals > 0
            && self.original_bytes > 0
            && self.retained_metadata_bytes > 0
            && self.state_bytes > 0
            && self.total_state_bytes > 0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CaptureSource {
    pub session_id: HistorySessionId,
    pub state: ExpectedState,
    pub navigation_revision: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OperationOrigin {
    Root,
    Command,
    Draft,
}
/// Versioned, non-executable operation summary; outcomes are in the node state.
/// These stable names are explicitly mapped, never derived with Debug/serde.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapturedOperation {
    pub version: u16,
    pub origin: OperationOrigin,
    pub steps: Vec<String>,
}
impl CapturedOperation {
    pub(crate) fn root() -> Self {
        Self {
            version: 1,
            origin: OperationOrigin::Root,
            steps: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapturedNodeInfo {
    pub id: PersistentNodeId,
    pub parent: Option<PersistentNodeId>,
    pub alias: u64,
    pub coordinate: BranchCoordinate,
    pub depth: u64,
    pub operation: Arc<CapturedOperation>,
    pub changed_objects: Vec<ObjectId>,
    pub origin: Option<Arc<crate::OriginalNodeRecord>>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapturedAdmission {
    pub object: ObjectId,
    pub first_node: PersistentNodeId,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CaptureManifest {
    pub version: u16,
    pub project_id: ProjectId,
    pub identity: PersistentHistoryIdentity,
    /// Runtime correlation only; not a persistent file epoch or request receipt.
    pub source: CaptureSource,
    pub root: PersistentNodeId,
    pub head: PersistentNodeId,
    /// Parent before child, stable append order; no ancestor-path copies.
    pub nodes: Vec<CapturedNodeInfo>,
    pub admissions: Vec<CapturedAdmission>,
    pub redo_preferences: Vec<(PersistentNodeId, PersistentNodeId)>,
    pub originals: Vec<BlobRef>,
    /// Zero means the existing local counter is exhausted, not a reusable ID.
    pub next_alias: u64,
    pub next_branch: u64,
    pub archive_metadata: Arc<crate::ArchiveMetadata>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CaptureError {
    UnsupportedVersion,
    WrongSession,
    Conflict {
        expected: ExpectedState,
        actual: ExpectedState,
    },
    Busy(DraftId),
    InFlight,
    InvalidLimits,
    InvalidWorkLimit,
    Cancelled,
    Terminated,
    IdentityUnavailable,
    InvalidIdentity,
    IdentityCollision,
    ResourceLimit(&'static str),
    MissingOriginal(BlobRef),
    InvalidOriginal(BlobRef),
    History(HistoryQueryError),
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CaptureStatistics {
    pub nodes_emitted: usize,
    pub forward_edges: usize,
    pub reverse_edges: usize,
    pub state_bytes: usize,
    pub retained_metadata_bytes: usize,
    pub original_bytes: u64,
}

pub struct CaptureRequest {
    pub version: u16,
    pub session_id: HistorySessionId,
    pub expected: ExpectedState,
    pub limits: CaptureLimits,
    pub cancellation: CaptureCancellation,
}

/// Every emitted state owns a snapshot lease, independently of the worker.
#[derive(Debug)]
pub struct CapturedNode {
    pub(crate) project: SkinProject,
    pub(crate) snapshot: crate::CaptureSnapshot,
    pub(crate) index: usize,
}
impl CapturedNode {
    pub fn project(&self) -> &SkinProject {
        &self.project
    }
    pub fn info(&self) -> &CapturedNodeInfo {
        &self.snapshot.manifest().nodes[self.index]
    }
    pub fn snapshot(&self) -> &crate::CaptureSnapshot {
        &self.snapshot
    }
}
#[derive(Debug)]
pub enum CapturePoll {
    Pending,
    Node(Box<CapturedNode>),
    Complete,
}
