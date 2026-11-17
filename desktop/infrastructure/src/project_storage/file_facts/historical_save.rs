//! One trusted association of existing domains from one inspected observation.
use super::*;

// Named fixed value copies, not an ABI stack-peak or host aggregation proof.
const _: () = assert!(
    std::mem::size_of::<FileHistoricalSaveContext>()
        + std::mem::size_of::<FileSaveComparison>()
        + std::mem::size_of::<FileSaveIntentSummary>()
        + std::mem::size_of::<FileCommitFacts>()
        + std::mem::size_of::<FileSaveCompareContext>()
        + std::mem::size_of::<FileHistoricalContentRequest>()
        + std::mem::size_of::<FileHistoricalOriginalsRequest>()
        + std::mem::size_of::<FileFactsSummary>()
        + std::mem::size_of::<(Option<u64>, Option<bool>)>()
        + std::mem::size_of::<FileHistoricalContentResult>()
        + std::mem::size_of::<FileHistoricalOriginalsResult>()
        + 2 * std::mem::size_of::<FileHistoricalSaveFacts>()
        + std::mem::size_of::<FileHistoricalSaveComparison>()
        <= 16 * 1024
);

fn generation(before: Option<u64>, actual: u64) -> (Option<u64>, Option<bool>) {
    let expected = before.and_then(|n| n.checked_add(1));
    (expected, expected.map(|n| n == actual))
}

impl InspectedFileFacts {
    pub fn compare_historical_save(
        &self,
        candidate: &FileSaveCandidate,
        context: FileHistoricalSaveContext,
    ) -> Result<FileHistoricalSaveComparison, FileFactsError> {
        if context.version != FILE_HISTORICAL_SAVE_VERSION {
            return Err(FileFactsError::UnsupportedVersion);
        }
        // Even Missing must pass the actual opaque candidate's full guard.
        // current.commit addresses the Ticket, never the selected commit.
        let current = self.compare_save(
            candidate,
            FileSaveCompareContext {
                version: FILE_SAVE_INTENT_VERSION,
                observation: context.observation,
                ticket: context.ticket,
            },
        )?;
        let content = self.historical_content(FileHistoricalContentRequest {
            version: FILE_HISTORICAL_CONTENT_VERSION,
            observation: context.observation,
            file: context.file,
            commit: context.commit,
        })?;
        let originals = self.historical_originals(FileHistoricalOriginalsRequest {
            version: FILE_HISTORICAL_ORIGINALS_VERSION,
            observation: context.observation,
            file: context.file,
            commit: context.commit,
        })?;
        let intent = current.intent;
        let selected = match (content.facts, originals.facts) {
            (None, None) => None,
            (Some(content), Some(originals))
                if content.commit == originals.commit
                    && content.commit.fact.commit == context.commit
                    && content.proof_version == FILE_COORD_PROOF_VERSION
                    && originals.proof_version == content.proof_version =>
            {
                let commit = content.commit;
                let (expected_generation, generation_equal) = generation(
                    intent.ticket.target.expected.generation,
                    commit.fact.generation,
                );
                Some(FileHistoricalSaveFacts {
                    relation: if commit.fact.commit == intent.ticket.commit {
                        FileHistoricalCommitRelation::TicketCommit
                    } else {
                        FileHistoricalCommitRelation::OtherCommit
                    },
                    content,
                    originals,
                    content_equal: content.content == intent.capture.projection.content,
                    originals_equal: originals.digest == intent.capture.projection.originals
                        && originals.len == intent.capture.originals
                        && originals.bytes == intent.capture.original_bytes,
                    head_equal: commit.fact.head == intent.ticket.head,
                    navigation_equal: content.intent_navigation == intent.navigation,
                    previous_equal: commit.previous == intent.ticket.target.expected.commit,
                    origin_equal: commit.fact.origin == intent.ticket.origin,
                    expected_generation,
                    generation_equal,
                    legacy_fields_equal: save::legacy_fields_equal(intent, commit),
                    is_latest: self
                        .summary()
                        .files
                        .latest
                        .is_some_and(|f| f.commit == context.commit),
                })
            }
            _ => return Err(FileFactsError::ValidationFailed),
        };
        Ok(FileHistoricalSaveComparison {
            version: FILE_HISTORICAL_SAVE_VERSION,
            context,
            intent,
            assurance: FileFactsAssurance::ExperimentalValidation,
            observed_expected: current.observed_expected,
            baseline_expected_equal: current.baseline_expected_equal,
            selected,
            metadata: FileHistoricalSaveMetadata::NotCompared,
            conclusion: FileCaptureConclusion::InsufficientForSaveReconciliation,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn generation_unavailable_is_distinct_from_a_computed_mismatch() {
        // Pure arithmetic boundary, not real q/file admission evidence.
        let matrix: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(
                Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/P3_2B1_2F_MATRIX.json"),
            )
            .unwrap(),
        )
        .unwrap();
        for case in matrix["generation_boundaries"].as_array().unwrap() {
            assert_eq!(
                generation(case["before"].as_u64(), case["actual"].as_u64().unwrap()),
                (case["expected"].as_u64(), case["equal"].as_bool())
            );
        }
    }
}
