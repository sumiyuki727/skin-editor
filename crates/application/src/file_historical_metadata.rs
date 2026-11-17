//! File-side post-commit metadata only; actual q still has no features source.
use crate::*;
pub const FILE_HISTORICAL_METADATA_VERSION: u16 = 1;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileHistoricalMetadataRequest {
    pub version: u16,
    pub observation: FileObservationId,
    pub file: FileFactsRequest,
    pub commit: CommitId,
}
/// A file metadata digest cannot establish actual q metadata or save authority.
/// ```compile_fail
/// use skin_editor_application::{FileHistoricalMetadataFacts, PreparedWitness};
/// fn elevate(value: FileHistoricalMetadataFacts) -> PreparedWitness { value.into() }
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileHistoricalMetadataFacts {
    pub proof_version: u16,
    pub commit: FileCommitFacts,
    pub metadata: [u8; 32],
    pub features_root: [u8; 32],
    pub tags: usize,
    pub extensions: usize,
    pub features: usize,
    pub required_features: usize,
    pub manual_marks: usize,
    pub extension_bytes: u64,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileHistoricalMetadataResult {
    pub version: u16,
    pub request: FileHistoricalMetadataRequest,
    pub assurance: FileFactsAssurance,
    pub facts: Option<FileHistoricalMetadataFacts>,
}
pub trait FileHistoricalMetadataPort {
    fn historical_metadata(
        &self,
        request: FileHistoricalMetadataRequest,
    ) -> Result<FileHistoricalMetadataResult, FileFactsError>;
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn historical_metadata_values_remain_fixed_without_q_or_save_capabilities() {
        assert!(std::mem::size_of::<FileHistoricalMetadataRequest>() <= 4096);
        assert!(std::mem::size_of::<FileHistoricalMetadataResult>() <= 4096);
    }
}
