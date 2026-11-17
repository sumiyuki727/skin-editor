//! Experimental, independently versioned file observations. No save authority.
use crate::{
    CommitId, HistorySessionId, PersistentNodeId, RestoreSelection, SaveCommitFact, SaveFileFacts,
    SaveTargetBinding,
};

pub const FILE_FACTS_PORT_VERSION: u16 = 1;
pub const FILE_COORD_PROOF_VERSION: u16 = 1;
pub const MAX_FILE_FACTS_PAGE: usize = 16;
pub const MAX_FILE_FACTS_COMMITS: usize = 128;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileFactsKey {
    pub session: HistorySessionId,
    pub sequence: u64,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileObservationId {
    pub instance: u128,
    pub sequence: u64,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileFactsRequest {
    pub version: u16,
    pub key: FileFactsKey,
    pub binding: SaveTargetBinding,
    pub selection: RestoreSelection,
}

/// A digest value is not a verification capability or a write witness.
///
/// ```compile_fail
/// use skin_editor_application::{FileFactsProof, PreparedWitness};
/// fn write(proof: FileFactsProof) -> PreparedWitness { proof.into() }
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileFactsProof {
    pub version: u16,
    pub content: [u8; 32],
    pub originals: [u8; 32],
    pub metadata: [u8; 32],
    pub request: [u8; 32],
    pub baseline: [u8; 32],
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FileFactsAssurance {
    /// The old full verifier's aggregate managed allocation is not yet proven.
    ExperimentalValidation,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileFactsSummary {
    pub request: FileFactsRequest,
    pub observation: FileObservationId,
    pub assurance: FileFactsAssurance,
    pub files: SaveFileFacts,
    pub selected_head: PersistentNodeId,
    pub nodes: usize,
    pub admissions: usize,
    pub originals: usize,
    pub original_bytes: u64,
    pub commits: usize,
    pub proof: FileFactsProof,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileCommitFacts {
    pub fact: SaveCommitFact,
    pub previous: Option<CommitId>,
    pub legacy_request: [u8; 32],
    pub navigation: [u8; 32],
    pub prefix: [u8; 32],
    pub nodes: usize,
    pub next_alias: u64,
    pub next_branch: u64,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileFactsPageRequest {
    pub version: u16,
    pub observation: FileObservationId,
    pub key: FileFactsKey,
    pub binding: SaveTargetBinding,
    pub offset: usize,
    pub limit: usize,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileFactsPage {
    pub observation: FileObservationId,
    pub offset: usize,
    pub entries: [Option<FileCommitFacts>; MAX_FILE_FACTS_PAGE],
    pub len: usize,
    pub next: Option<usize>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FileFactsLimit {
    File,
    Scratch,
    Control,
    Commits,
    Validation,
    Sequence,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FileFactsError {
    UnsupportedVersion,
    InvalidRequest,
    InvalidLimits,
    Unsupported,
    Busy,
    Cancelled,
    IdentityUnavailable,
    TargetConflict,
    NeedsExplicitRecovery,
    ValidationFailed,
    Io,
    Limit(FileFactsLimit),
    InvalidPage,
    StalePage,
    Cleanup { token: u128 },
}

/// The adapter is trusted; returned values alone cannot install save facts.
pub trait FileFactsPort {
    fn summary(&self) -> FileFactsSummary;
    fn commits(&self, request: FileFactsPageRequest) -> Result<FileFactsPage, FileFactsError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn file_fact_carriers_remain_fixed_and_separate_from_write_witnesses() {
        assert!(std::mem::size_of::<FileFactsSummary>() <= 4096);
        assert!(std::mem::size_of::<FileFactsPage>() <= 16 * 1024);
        assert!(std::mem::size_of::<FileFactsProof>() <= 4096);
        assert!(std::mem::size_of::<FileFactsError>() <= 128);
        assert_eq!(MAX_FILE_FACTS_PAGE, 16);
        assert_eq!(MAX_FILE_FACTS_COMMITS, 128);
    }
}
