//! Historical raw sets stay scoped to one fully inspected observation.
use super::*;

impl FileHistoricalOriginalsPort for InspectedFileFacts {
    fn historical_originals(
        &self,
        request: FileHistoricalOriginalsRequest,
    ) -> Result<FileHistoricalOriginalsResult, FileFactsError> {
        if request.version != FILE_HISTORICAL_ORIGINALS_VERSION
            || request.file.version != FILE_FACTS_PORT_VERSION
        {
            return Err(FileFactsError::UnsupportedVersion);
        }
        let summary = self.summary();
        if request.observation != summary.observation || request.file != summary.request {
            return Err(FileFactsError::StalePage);
        }
        let index = self.data.commits[..summary.commits]
            .iter()
            .position(|c| c.is_some_and(|c| c.fact.commit == request.commit));
        let facts = if let Some(i) = index {
            let point = self.data.historical_originals.points[i];
            let mut entries = [None; MAX_FILE_HISTORICAL_ORIGINALS];
            let mut len = 0;
            for (bit, blob) in self.data.historical_originals.dictionary.iter().enumerate() {
                if point.mask & (1u8 << bit) != 0 {
                    entries[len] = Some(blob.ok_or(FileFactsError::ValidationFailed)?);
                    len += 1;
                }
            }
            Some(FileHistoricalOriginalsFacts {
                proof_version: FILE_COORD_PROOF_VERSION,
                commit: self.data.commits[i].expect("validated fixed commit"),
                digest: point.digest,
                entries,
                len,
                bytes: point.bytes,
            })
        } else {
            None
        };
        Ok(FileHistoricalOriginalsResult {
            version: FILE_HISTORICAL_ORIGINALS_VERSION,
            request,
            assurance: FileFactsAssurance::ExperimentalValidation,
            facts,
        })
    }
}
impl InspectedFileFacts {
    pub fn compare_historical_originals(
        &self,
        candidate: &FileSaveCandidate,
        context: FileHistoricalOriginalsCompareContext,
    ) -> Result<FileHistoricalOriginalsComparison, FileFactsError> {
        let historical = self.historical_originals(context.request)?;
        let current = self.compare_save(
            candidate,
            FileSaveCompareContext {
                version: FILE_SAVE_INTENT_VERSION,
                observation: context.request.observation,
                ticket: context.ticket,
            },
        )?;
        let intent = current.intent;
        Ok(FileHistoricalOriginalsComparison {
            version: FILE_HISTORICAL_ORIGINALS_VERSION,
            intent,
            historical,
            originals_equal: historical.facts.map(|f| {
                f.digest == intent.capture.projection.originals
                    && f.len == intent.capture.originals
                    && f.bytes == intent.capture.original_bytes
            }),
            conclusion: FileCaptureConclusion::InsufficientForSaveReconciliation,
        })
    }
}
