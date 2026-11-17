//! Fixed historical content subsets, never complete historical save evidence.
use super::*;

pub(super) struct HistoricalContentData {
    pub content: [[u8; 32]; MAX_FILE_FACTS_COMMITS],
    pub admissions: [usize; MAX_FILE_FACTS_COMMITS],
}
pub(super) fn build(
    cache: &Cache,
    cancel: &CaptureCancellation,
    probe: Probe,
) -> Result<HistoricalContentData, StorageError> {
    if cache.checkpoints.len() > MAX_FILE_FACTS_COMMITS
        || cache.order.len() > MAX_FILE_CAPTURE_NODES
        || cache.admissions.len() > FileFactsLimits::default().admissions
    {
        return Err(StorageError::Limit("historical content"));
    }
    let mut data = HistoricalContentData {
        content: [[0; 32]; MAX_FILE_FACTS_COMMITS],
        admissions: [0; MAX_FILE_FACTS_COMMITS],
    };
    for (i, point) in cache.checkpoints.iter().enumerate() {
        check(cancel)?;
        let (digest, admissions) = proof::content(cache, point.node_count, cancel)?;
        // A batch cancellation after real work must not publish partial facts.
        probe.step(Stage::ReadHistoricalContent, cancel)?;
        data.content[i] = digest;
        data.admissions[i] = admissions;
    }
    Ok(data)
}
impl FileHistoricalContentPort for InspectedFileFacts {
    fn historical_content(
        &self,
        request: FileHistoricalContentRequest,
    ) -> Result<FileHistoricalContentResult, FileFactsError> {
        if request.version != FILE_HISTORICAL_CONTENT_VERSION
            || request.file.version != FILE_FACTS_PORT_VERSION
        {
            return Err(FileFactsError::UnsupportedVersion);
        }
        let f = self.summary();
        if request.observation != f.observation || request.file != f.request {
            return Err(FileFactsError::StalePage);
        }
        let facts = self.data.commits[..f.commits]
            .iter()
            .position(|c| c.is_some_and(|c| c.fact.commit == request.commit))
            .map(|i| FileHistoricalContentFacts {
                proof_version: FILE_COORD_PROOF_VERSION,
                commit: self.data.commits[i].expect("validated fixed commit"),
                admissions: self.data.historical.admissions[i],
                content: self.data.historical.content[i],
                intent_navigation: self.data.commit_navigation[i],
            });
        Ok(FileHistoricalContentResult {
            version: FILE_HISTORICAL_CONTENT_VERSION,
            request,
            facts,
        })
    }
}
impl InspectedFileFacts {
    pub fn compare_historical_content(
        &self,
        candidate: &FileSaveCandidate,
        context: FileHistoricalContentCompareContext,
    ) -> Result<FileHistoricalContentComparison, FileFactsError> {
        let historical = self.historical_content(context.request)?;
        // Reuse the complete original-ticket/reader/root/binding guard. Only
        // original expected versus observed latest may differ, as in B1-2B.
        let current = self.compare_save(
            candidate,
            FileSaveCompareContext {
                version: FILE_SAVE_INTENT_VERSION,
                observation: context.request.observation,
                ticket: context.ticket,
            },
        )?;
        let intent = current.intent;
        Ok(FileHistoricalContentComparison {
            version: FILE_HISTORICAL_CONTENT_VERSION,
            intent,
            historical,
            content_equal: historical
                .facts
                .map(|f| f.content == intent.capture.projection.content),
            head_equal: historical
                .facts
                .map(|f| f.commit.fact.head == intent.ticket.head),
            navigation_equal: historical
                .facts
                .map(|f| f.intent_navigation == intent.navigation),
            conclusion: FileCaptureConclusion::InsufficientForSaveReconciliation,
        })
    }
}
