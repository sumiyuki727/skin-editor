//! Observed container provenance, independent from project edits and save authority.
use crate::*;
use std::any::Any;
use std::sync::Arc;

pub const FILE_PROFILE_VERSION: u16 = 1;
pub const MAX_FILE_PROFILE_BYTES: usize = 128 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileProfileOrigin {
    pub version: u16,
    pub file: FileFactsRequest,
    pub observation: FileObservationId,
    pub root: PersistentNodeId,
    pub opened_head: PersistentNodeId,
    pub features_root: [u8; 32],
    pub source_sha256: [u8; 32],
}
/// Implementations must be immutable. Public values alone are not authenticated
/// provenance: the consuming adapter must recognize its own actual provider.
pub trait FileProfileSource: Any + Send + Sync {
    fn origin(&self) -> FileProfileOrigin;
    fn logical_bytes(&self) -> usize;
}
#[derive(Clone)]
pub(crate) struct ProfileAttachment {
    source: Arc<dyn FileProfileSource>,
    pub origin: FileProfileOrigin,
    pub bytes: usize,
}
impl ProfileAttachment {
    pub fn new(source: Arc<dyn FileProfileSource>) -> Result<Self, RestoreError> {
        let origin = source.origin();
        let bytes = source.logical_bytes();
        if origin.version != FILE_PROFILE_VERSION
            || origin.file.version != FILE_FACTS_PORT_VERSION
            || origin.file.binding.version != SAVE_COORDINATION_VERSION
        {
            return Err(RestoreError::Invalid("profile version"));
        }
        if origin.file.binding.phase != SaveTargetPhase::ExistingVerified
            || origin.file.key.sequence == 0
            || origin.observation.instance == 0
            || origin.observation.sequence == 0
            || bytes == 0
            || bytes > MAX_FILE_PROFILE_BYTES
        {
            return Err(RestoreError::Invalid("profile origin/capacity"));
        }
        Ok(Self {
            source,
            origin,
            bytes,
        })
    }
    pub fn source(&self) -> &dyn FileProfileSource {
        self.source.as_ref()
    }
    pub fn logical_cost(&self) -> Option<usize> {
        self.bytes.checked_add(std::mem::size_of::<Self>())
    }
}
pub(crate) fn profile_ref(
    profile: Option<&ProfileAttachment>,
    version: u16,
) -> Result<Option<&dyn FileProfileSource>, FileFactsError> {
    if version != FILE_PROFILE_VERSION {
        return Err(FileFactsError::UnsupportedVersion);
    }
    Ok(profile.map(ProfileAttachment::source))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileProfileRestoreRequest {
    pub version: u16,
    pub file: FileFactsRequest,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileCapturedMetadataFacts {
    pub origin: FileProfileOrigin,
    pub proof_version: u16,
    pub metadata: [u8; 32],
    pub features: usize,
    pub required_features: usize,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileCapturedMetadataContext {
    pub version: u16,
    pub observation: FileObservationId,
    pub file: FileFactsRequest,
    pub commit: CommitId,
    pub ticket: SaveTicket,
}
/// Equality of captured *existing* metadata is not hypothetical append equality.
/// ```compile_fail
/// use skin_editor_application::{FileCapturedMetadataComparison, PreparedWitness};
/// fn elevate(v: FileCapturedMetadataComparison) -> PreparedWitness { v.into() }
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileCapturedMetadataComparison {
    pub version: u16,
    pub context: FileCapturedMetadataContext,
    /// None means unavailable, including old restores and advanced expected.
    pub captured: Option<FileCapturedMetadataFacts>,
    /// None means the selected commit is absent in this validated observation.
    pub selected: Option<FileHistoricalMetadataFacts>,
    pub metadata_equal: Option<bool>,
    pub assurance: FileFactsAssurance,
    pub conclusion: FileCaptureConclusion,
}
const _: () = assert!(std::mem::size_of::<FileCapturedMetadataComparison>() <= 4096);
