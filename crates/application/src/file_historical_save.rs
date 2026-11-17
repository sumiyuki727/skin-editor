//! Correlated existing historical domains, never complete save qualification.
use crate::*;

pub const FILE_HISTORICAL_SAVE_VERSION: u16 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileHistoricalSaveContext {
    pub version: u16,
    pub observation: FileObservationId,
    pub file: FileFactsRequest,
    pub commit: CommitId,
    pub ticket: SaveTicket,
}
/// Only an ID relation; neither variant establishes a successful save.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FileHistoricalCommitRelation {
    TicketCommit,
    OtherCommit,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FileHistoricalSaveMetadata {
    NotCompared,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileHistoricalSaveFacts {
    pub relation: FileHistoricalCommitRelation,
    pub content: FileHistoricalContentFacts,
    pub originals: FileHistoricalOriginalsFacts,
    pub content_equal: bool,
    pub originals_equal: bool,
    pub head_equal: bool,
    pub navigation_equal: bool,
    pub previous_equal: bool,
    pub origin_equal: bool,
    pub expected_generation: Option<u64>,
    pub generation_equal: Option<bool>,
    /// Existing legacy request fields; not complete binding or domain equality.
    pub legacy_fields_equal: bool,
    pub is_latest: bool,
}
/// Fixed evidence cannot install facts or become a writer witness.
/// ```compile_fail
/// use skin_editor_application::{FileHistoricalSaveComparison, PreparedWitness};
/// fn elevate(value: FileHistoricalSaveComparison) -> PreparedWitness { value.into() }
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileHistoricalSaveComparison {
    pub version: u16,
    pub context: FileHistoricalSaveContext,
    pub intent: FileSaveIntentSummary,
    pub assurance: FileFactsAssurance,
    pub observed_expected: ExpectedFileCommit,
    pub baseline_expected_equal: bool,
    pub selected: Option<FileHistoricalSaveFacts>,
    pub metadata: FileHistoricalSaveMetadata,
    pub conclusion: FileCaptureConclusion,
}
const _: () = {
    assert!(std::mem::size_of::<FileHistoricalSaveContext>() <= 4096);
    assert!(std::mem::size_of::<FileHistoricalSaveComparison>() <= 4096);
};

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn correlated_historical_save_values_remain_fixed_and_without_authority() {
        assert!(std::mem::size_of::<FileHistoricalSaveFacts>() <= 2048);
        assert_eq!(FILE_HISTORICAL_SAVE_VERSION, 1);
    }
}
