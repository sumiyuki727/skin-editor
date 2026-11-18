//! Observed container provenance, independent from project edits and save authority.
use crate::*;
use std::any::Any;
use std::sync::Arc;

pub const FILE_PROFILE_VERSION: u16 = 1;
pub const FILE_OPENING_TARGET_VERSION: u16 = 1;
pub const MAX_FILE_PROFILE_BYTES: usize = 128 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileOpeningTargetContext {
    pub version: u16,
    pub ticket: SaveTicket,
    pub observation: FileObservationId,
    pub observed_request: FileFactsRequest,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileOpeningTargetFacts {
    pub origin: FileProfileOrigin,
    pub name_digest_equal: bool,
    /// Equality of the reported volume serial and 64-bit file index at two instants.
    /// None means at least one observation lacked usable fields.
    pub file_id_fields_equal: Option<bool>,
    pub source_digest_equal: bool,
    pub expected_equal: bool,
    pub selected_head_equal: bool,
}

/// Read-only comparison. Matching fields cannot prove continuity across a closed handle.
/// ```compile_fail
/// use skin_editor_application::{FileOpeningTargetComparison, PreparedWitness};
/// fn elevate(v: FileOpeningTargetComparison) -> PreparedWitness { v.into() }
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileOpeningTargetComparison {
    pub context: FileOpeningTargetContext,
    /// Absent if the capture has no authenticated, in-scope original opening.
    pub facts: Option<FileOpeningTargetFacts>,
    pub assurance: FileFactsAssurance,
    pub conclusion: FileCaptureConclusion,
}
const _: () = assert!(std::mem::size_of::<FileOpeningTargetComparison>() <= 4096);

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

pub const FILE_FRESH_APPEND_VERSION: u16 = 1;

/// Conditional domain-3 bytes for an ordinary append to the observed baseline.
/// The legacy writer can still take its NoChange branch.
/// ```compile_fail
/// use skin_editor_application::{FileFreshAppendMetadata, PreparedWitness};
/// fn elevate(value: FileFreshAppendMetadata) -> PreparedWitness { value.into() }
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileFreshAppendMetadata {
    pub context: FileCapturedMetadataContext,
    pub origin: FileProfileOrigin,
    pub proof_version: u16,
    pub before: [u8; 32],
    pub receipt: SaveCommitFact,
    pub metadata: [u8; 32],
    pub assurance: FileFactsAssurance,
    pub conclusion: FileCaptureConclusion,
}
const _: () = assert!(std::mem::size_of::<FileFreshAppendMetadata>() <= 4096);

pub const FILE_NO_CHANGE_COVERAGE_VERSION: u16 = 1;

/// Frozen request compared to one complete, existing file baseline.
/// ```compile_fail
/// use skin_editor_application::{FileNoChangeCoverage, SaveStorageObservation};
/// fn install(v: FileNoChangeCoverage) -> SaveStorageObservation { v.into() }
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileNoChangeCoverage {
    pub context: FileCapturedMetadataContext,
    pub origin: FileProfileOrigin,
    pub files: SaveFileFacts,
    pub history_equal: bool,
    pub originals_equal: bool,
    pub metadata_equal: bool,
    pub counters_equal: bool,
    /// Head and raw Redo of each checkpoint, using that checkpoint's counters.
    pub manual_navigation_equal: Option<bool>,
    pub protection_navigation_equal: bool,
    /// Includes the derived opening Redo and unchanged session revisions.
    pub unchanged_opening: bool,
    pub assurance: FileFactsAssurance,
    pub conclusion: FileCaptureConclusion,
}
impl FileNoChangeCoverage {
    /// None means this basis does not apply to the ticket's intent.
    /// A match describes logical coverage of q only.
    pub fn matches_basis(&self, basis: SaveNoChangeBasis) -> Option<bool> {
        let ticket = self.context.ticket;
        let covered = self.history_equal
            && self.originals_equal
            && self.metadata_equal
            && self.counters_equal;
        let manual_head = self
            .files
            .manual
            .is_some_and(|f| f.origin == SaveOrigin::Manual && f.head == ticket.head);
        let matched = match (ticket.origin, basis) {
            (SaveOrigin::Manual, SaveNoChangeBasis::CurrentManual) => {
                manual_head && self.manual_navigation_equal == Some(true)
            }
            (SaveOrigin::Exit, SaveNoChangeBasis::CurrentProtection) => {
                self.files.protected.is_some_and(|f| f.head == ticket.head)
                    && self.protection_navigation_equal
            }
            (SaveOrigin::Manual, SaveNoChangeBasis::VerifiedOpening) => {
                manual_head && self.unchanged_opening
            }
            (SaveOrigin::Exit, SaveNoChangeBasis::VerifiedOpening) => {
                self.files.protected.is_some() && self.unchanged_opening
            }
            _ => return None,
        };
        Some(covered && matched)
    }
}
const _: () = assert!(std::mem::size_of::<FileNoChangeCoverage>() <= 4096);
