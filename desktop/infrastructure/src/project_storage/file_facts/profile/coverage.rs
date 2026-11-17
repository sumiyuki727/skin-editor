use super::*;

const WORK_BYTES: usize = intent_context::WORK_BYTES
    + 2 * (size_of::<FileNoChangeCoverage>()
        + size_of::<FileSaveComparison>()
        + size_of::<FileCapturedMetadataComparison>()
        + size_of::<FileHistoricalMetadataResult>()
        + size_of::<FileSaveIntentSummary>()
        + size_of::<FileFactsSummary>());

impl InspectedFileFacts {
    fn checkpoint_navigation_matches(
        &self,
        fact: SaveCommitFact,
        manifest: &CaptureManifest,
        scratch: &mut intent_context::Workspace,
    ) -> Result<bool, FileFactsError> {
        let (i, checkpoint) = self.data.commits[..self.data.summary.commits]
            .iter()
            .enumerate()
            .find_map(|(i, c)| c.filter(|c| c.fact == fact).map(|c| (i, c)))
            .ok_or(FileFactsError::ValidationFailed)?;
        if fact.head != manifest.head {
            return Ok(false);
        }
        // The request's live allocation counters belong to the latest tree.
        // Compare head and Redo in the historical checkpoint's own encoding.
        let digest = intent_context::navigation(
            manifest.head,
            checkpoint.next_alias,
            checkpoint.next_branch,
            &manifest.redo_preferences,
            scratch,
        )
        .map_err(storage)?;
        Ok(digest == self.data.commit_navigation[i])
    }
}

impl FileFactsReader {
    #[cfg(all(test, windows))]
    pub(crate) fn hold_coverage_workspace_margin(&self, short_by: usize) -> ByteLease {
        let remaining = self.limits.control_bytes - self.control.used();
        self.control
            .reserve(remaining - WORK_BYTES + short_by)
            .unwrap()
    }

    pub fn compare_no_change_coverage(
        &self,
        baseline: &InspectedFileFacts,
        q: &FileSaveCandidate,
        context: FileCapturedMetadataContext,
        cancel: CaptureCancellation,
    ) -> Result<Option<FileNoChangeCoverage>, FileFactsError> {
        if context.version != FILE_NO_CHANGE_COVERAGE_VERSION {
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
        let Some((captured, parent)) = baseline.original_metadata_baseline(q, context)? else {
            return Ok(None);
        };
        let s = baseline.summary();
        let intent = q.summary();
        let capture = intent.capture;
        let protected = s.files.protected.ok_or(FileFactsError::ValidationFailed)?;
        let ticket = context.ticket;
        let origin = captured.origin;
        let (manual_navigation_equal, protection_navigation_equal) = {
            let mut scratch = [0; 4096];
            let manifest = q.snapshot().manifest();
            let manual = s
                .files
                .manual
                .map(|f| baseline.checkpoint_navigation_matches(f, manifest, &mut scratch))
                .transpose()?;
            let protection =
                baseline.checkpoint_navigation_matches(protected, manifest, &mut scratch)?;
            (manual, protection)
        };
        let unchanged_opening = origin.file.selection == context.file.selection
            && origin.opened_head == ticket.head
            && s.selected_head == ticket.head
            && ticket.source.state.revision == 0
            && ticket.source.navigation_revision == 0
            && intent.navigation == baseline.data.intent.opening_navigation;
        #[cfg(test)]
        self.probe
            .step(Stage::ReadNoChangeCoverage, &cancel)
            .map_err(storage)?;
        check(&cancel).map_err(storage)?;
        Ok(Some(FileNoChangeCoverage {
            context,
            origin,
            files: s.files,
            history_equal: capture.projection.content == s.proof.content
                && capture.nodes == s.nodes
                && capture.admissions == s.admissions,
            originals_equal: capture.projection.originals == s.proof.originals
                && capture.originals == s.originals
                && capture.original_bytes == s.original_bytes,
            metadata_equal: captured.metadata == parent.metadata,
            counters_equal: intent.next_alias == parent.commit.next_alias
                && intent.next_branch == parent.commit.next_branch,
            manual_navigation_equal,
            protection_navigation_equal,
            unchanged_opening,
            assurance: s.assurance,
            conclusion: FileCaptureConclusion::InsufficientForSaveReconciliation,
        }))
    }
}
