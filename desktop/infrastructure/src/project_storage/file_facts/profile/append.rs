use super::*;

// One sort workspace and fixed intermediate/return values. No new retained
// state on q, its profile, or the observation.
const WORK_BYTES: usize = intent_context::WORK_BYTES
    + 2 * (size_of::<FileSaveComparison>()
        + size_of::<FileCapturedMetadataComparison>()
        + size_of::<FileFreshAppendMetadata>()
        + size_of::<FileHistoricalMetadataResult>()
        + size_of::<ArchiveReceipt>());

impl FileFactsReader {
    #[cfg(all(test, windows))]
    pub(crate) fn hold_append_workspace_margin(&self, short_by: usize) -> ByteLease {
        let remaining = self.limits.control_bytes - self.control.used();
        self.control
            .reserve(remaining - WORK_BYTES + short_by)
            .unwrap()
    }

    /// Project the ordinary append branch on the exact original baseline.
    /// None means there is no qualified projection for this q and observation.
    pub fn project_fresh_append_metadata(
        &self,
        baseline: &InspectedFileFacts,
        q: &FileSaveCandidate,
        context: FileCapturedMetadataContext,
        cancel: CaptureCancellation,
    ) -> Result<Option<FileFreshAppendMetadata>, FileFactsError> {
        if context.version != FILE_FRESH_APPEND_VERSION {
            return Err(FileFactsError::UnsupportedVersion);
        }
        if context.observation.instance != self.instance {
            return Err(FileFactsError::StalePage);
        }
        check(&cancel).map_err(storage)?;
        let _lease = self
            .control
            .reserve(WORK_BYTES)
            .ok_or(FileFactsError::Limit(FileFactsLimit::Control))?;
        let before = baseline.compare_captured_metadata(
            q,
            FileCapturedMetadataContext {
                version: FILE_PROFILE_VERSION,
                ..context
            },
        )?;
        let ticket = context.ticket;
        if Some(context.commit) != ticket.target.expected.commit {
            return Err(FileFactsError::InvalidRequest);
        }
        let (Some(captured), Some(parent)) = (before.captured, before.selected) else {
            return Ok(None);
        };
        let summary = baseline.summary();
        if before.metadata_equal != Some(true)
            || context.file.binding != ticket.target
            || summary.files.latest != Some(parent.commit.fact)
            || Some(parent.commit.fact.generation) != ticket.target.expected.generation
            || captured.origin.features_root != parent.features_root
            || captured.origin.source_sha256 != baseline._source.digest
            || baseline.data.commits[..summary.commits]
                .iter()
                .flatten()
                .any(|c| c.fact.commit == ticket.commit)
        {
            return Ok(None);
        }
        if summary.commits >= self.limits.commits {
            return Err(FileFactsError::Limit(FileFactsLimit::Commits));
        }
        let snapshot = q.snapshot();
        let metadata = &snapshot.manifest().archive_metadata;
        let Some(receipt) = save::predict(metadata, ticket) else {
            return Ok(None);
        };
        let provider = provider(snapshot).ok_or(FileFactsError::ValidationFailed)?;
        let (digest, _) = encode_metadata(
            metadata,
            &provider.payload,
            &mut [0; 4096],
            &cancel,
            Some(&receipt),
        )?;
        #[cfg(test)]
        self.probe
            .step(Stage::ReadFreshAppendMetadata, &cancel)
            .map_err(storage)?;
        check(&cancel).map_err(storage)?;
        Ok(Some(FileFreshAppendMetadata {
            context,
            origin: captured.origin,
            proof_version: FILE_COORD_PROOF_VERSION,
            before: captured.metadata,
            receipt: (&receipt).into(),
            metadata: digest,
            assurance: before.assurance,
            conclusion: before.conclusion,
        }))
    }
}
