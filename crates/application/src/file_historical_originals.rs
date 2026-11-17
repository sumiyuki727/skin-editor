//! Historical raw-reference sets only, never complete save evidence.
use crate::*;

pub const FILE_HISTORICAL_ORIGINALS_VERSION: u16 = 1;
pub const MAX_FILE_HISTORICAL_ORIGINALS: usize = 8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileHistoricalOriginalsRequest {
    pub version: u16,
    pub observation: FileObservationId,
    pub file: FileFactsRequest,
    pub commit: CommitId,
}
/// A raw set does not establish content/metadata equality or save authority.
/// ```compile_fail
/// use skin_editor_application::{FileHistoricalOriginalsFacts, PreparedWitness};
/// fn elevate(value: FileHistoricalOriginalsFacts) -> PreparedWitness { value.into() }
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileHistoricalOriginalsFacts {
    pub proof_version: u16,
    pub commit: FileCommitFacts,
    pub digest: [u8; 32],
    /// SHA-sorted dense Some prefix, then None; empty bytes remain a member.
    pub entries: [Option<values::BlobRef>; MAX_FILE_HISTORICAL_ORIGINALS],
    pub len: usize,
    pub bytes: u64,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileHistoricalOriginalsResult {
    pub version: u16,
    pub request: FileHistoricalOriginalsRequest,
    pub assurance: FileFactsAssurance,
    pub facts: Option<FileHistoricalOriginalsFacts>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileHistoricalOriginalsCompareContext {
    pub request: FileHistoricalOriginalsRequest,
    pub ticket: SaveTicket,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileHistoricalOriginalsComparison {
    pub version: u16,
    pub intent: FileSaveIntentSummary,
    pub historical: FileHistoricalOriginalsResult,
    pub originals_equal: Option<bool>,
    pub conclusion: FileCaptureConclusion,
}
pub trait FileHistoricalOriginalsPort {
    fn historical_originals(
        &self,
        request: FileHistoricalOriginalsRequest,
    ) -> Result<FileHistoricalOriginalsResult, FileFactsError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn historical_originals_results_remain_fixed_and_separate_from_authority() {
        assert_eq!(MAX_FILE_HISTORICAL_ORIGINALS, 8);
        assert!(std::mem::size_of::<FileHistoricalOriginalsResult>() <= 4096);
        assert!(std::mem::size_of::<FileHistoricalOriginalsComparison>() <= 4096);
    }
}
