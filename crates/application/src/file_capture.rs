//! Experimental partial capture evidence; no save acknowledgement capability.
use crate::{CaptureSource, FileFactsRequest, FileObservationId, PersistentNodeId};

pub const FILE_CAPTURE_PROJECTION_VERSION: u16 = 1;
pub const FILE_READ_PREFLIGHT_VERSION: u16 = 1;
pub const FILE_READ_PREFLIGHT_TABLES: usize = 16;
pub const MAX_FILE_CAPTURE_NODES: usize = 64;

/// Only the first two FileCoord-Proof-v1 domains, never a complete file proof.
///
/// ```compile_fail
/// use skin_editor_application::{FileCaptureProjection, PreparedWitness};
/// fn elevate(value: FileCaptureProjection) -> PreparedWitness { value.into() }
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileCaptureProjection {
    pub version: u16,
    pub content: [u8; 32],
    pub originals: [u8; 32],
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileCaptureSummary {
    pub version: u16,
    pub request: FileFactsRequest,
    pub source: CaptureSource,
    pub root: PersistentNodeId,
    pub head: PersistentNodeId,
    pub nodes: usize,
    pub admissions: usize,
    pub originals: usize,
    pub original_bytes: u64,
    /// Logical encoded sizes, not allocations, peak memory or timing.
    pub encoded_block_bytes: usize,
    pub projection: FileCaptureProjection,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileCaptureCompareContext {
    pub version: u16,
    pub observation: FileObservationId,
    pub source: CaptureSource,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FileCaptureConclusion {
    InsufficientForSaveReconciliation,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileCaptureComparison {
    pub version: u16,
    pub candidate: FileCaptureSummary,
    pub observation: FileObservationId,
    pub content_equal: bool,
    pub originals_equal: bool,
    pub selected_head_equal: bool,
    pub conclusion: FileCaptureConclusion,
}

/// Per-table raw sizes (declared original sizes for `blobs`), not heap usage.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FileReadAggregate {
    pub rows: u64,
    pub bytes: u64,
    pub largest: u64,
}
/// Fixed order: header, features, blobs, blob_chunks, blocks, block_refs,
/// blob_refs, nodes, addresses, state_views, commits, manual_marks, object_ids,
/// tags, control, extensions. A successful preflight is not full validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileReadPreflight {
    pub version: u16,
    pub observation: FileObservationId,
    pub catalog: FileReadAggregate,
    pub tables: [FileReadAggregate; FILE_READ_PREFLIGHT_TABLES],
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn partial_file_evidence_and_preflight_remain_fixed_values() {
        assert!(std::mem::size_of::<FileCaptureSummary>() <= 4096);
        assert!(std::mem::size_of::<FileCaptureComparison>() <= 4096);
        assert!(std::mem::size_of::<FileReadPreflight>() <= 4096);
        assert_eq!(MAX_FILE_CAPTURE_NODES, 64);
        assert_eq!(FILE_READ_PREFLIGHT_TABLES, 16);
    }
}
