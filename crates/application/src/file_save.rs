//! Read-only save intent evidence; no completion or writer capability.
use crate::*;
pub const FILE_SAVE_INTENT_VERSION: u16 = 1;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileArchivePrediction {
    pub fact: SaveCommitFact,
    pub archive: [u8; 32],
}
/// ArchiveMetadata-only and navigation subsets, never FileCoord's full metadata.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileIntentFacts {
    pub version: u16,
    pub observation: FileObservationId,
    pub archive: [u8; 32],
    pub opening_navigation: [u8; 32],
    pub latest_navigation: [u8; 32],
}
/// ```compile_fail
/// use skin_editor_application::{FileSaveIntentSummary, PreparedWitness};
/// fn elevate(value: FileSaveIntentSummary) -> PreparedWitness { value.into() }
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileSaveIntentSummary {
    pub version: u16,
    pub ticket: SaveTicket,
    pub capture: FileCaptureSummary,
    pub prefix: [u8; 32],
    pub next_alias: u64,
    pub next_branch: u64,
    pub legacy_request: [u8; 32],
    pub navigation: [u8; 32],
    pub archive_before: [u8; 32],
    /// Hypothetical new commit only. None when a new generation is unavailable.
    pub would_commit: Option<FileArchivePrediction>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileSaveCompareContext {
    pub version: u16,
    pub observation: FileObservationId,
    pub ticket: SaveTicket,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileSaveCommitEvidence {
    pub fact: SaveCommitFact,
    pub previous: Option<CommitId>,
    pub legacy_fields_equal: bool,
    pub generation_equal: bool,
    pub navigation_equal: bool,
    pub is_latest: bool,
    pub would_commit_archive_equal: Option<bool>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileSaveComparison {
    pub version: u16,
    pub intent: FileSaveIntentSummary,
    pub observation: FileObservationId,
    pub observed_expected: ExpectedFileCommit,
    pub baseline_expected_equal: bool,
    pub content_equal: bool,
    pub originals_equal: bool,
    pub opening_navigation_equal: bool,
    pub latest_navigation_equal: bool,
    pub before_archive_equal: Option<bool>,
    pub commit: Option<FileSaveCommitEvidence>,
    pub conclusion: FileCaptureConclusion,
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn save_intent_evidence_is_fixed_and_cannot_carry_write_capabilities() {
        assert!(std::mem::size_of::<FileSaveIntentSummary>() <= 4096);
        assert!(std::mem::size_of::<FileSaveComparison>() <= 4096);
        assert!(std::mem::size_of::<FileIntentFacts>() <= 4096);
    }
}
