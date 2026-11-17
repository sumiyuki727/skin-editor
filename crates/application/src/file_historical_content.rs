//! Historical node/admission content only; no historical-original or save proof.
use crate::*;

pub const FILE_HISTORICAL_CONTENT_VERSION: u16 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileHistoricalContentRequest {
    pub version: u16,
    pub observation: FileObservationId,
    pub file: FileFactsRequest,
    pub commit: CommitId,
}
/// A content digest has no authority and does not include historical originals.
/// ```compile_fail
/// use skin_editor_application::{FileHistoricalContentFacts, PreparedWitness};
/// fn elevate(value: FileHistoricalContentFacts) -> PreparedWitness { value.into() }
/// ```
/// ```compile_fail
/// use skin_editor_application::FileHistoricalContentFacts;
/// fn originals(value: FileHistoricalContentFacts) -> [u8; 32] { value.originals }
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileHistoricalContentFacts {
    pub proof_version: u16,
    pub commit: FileCommitFacts,
    pub admissions: usize,
    pub content: [u8; 32],
    /// SaveIntent navigation subset, distinct from commit.navigation's block hash.
    pub intent_navigation: [u8; 32],
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileHistoricalContentResult {
    pub version: u16,
    pub request: FileHistoricalContentRequest,
    pub facts: Option<FileHistoricalContentFacts>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileHistoricalContentCompareContext {
    pub request: FileHistoricalContentRequest,
    pub ticket: SaveTicket,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileHistoricalContentComparison {
    pub version: u16,
    pub intent: FileSaveIntentSummary,
    pub historical: FileHistoricalContentResult,
    pub content_equal: Option<bool>,
    pub head_equal: Option<bool>,
    pub navigation_equal: Option<bool>,
    pub conclusion: FileCaptureConclusion,
}

pub trait FileHistoricalContentPort {
    fn historical_content(
        &self,
        request: FileHistoricalContentRequest,
    ) -> Result<FileHistoricalContentResult, FileFactsError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn historical_content_results_remain_fixed_and_provenance_scoped() {
        assert!(std::mem::size_of::<FileHistoricalContentRequest>() <= 4096);
        assert!(std::mem::size_of::<FileHistoricalContentResult>() <= 4096);
        assert!(std::mem::size_of::<FileHistoricalContentComparison>() <= 4096);
    }
}
