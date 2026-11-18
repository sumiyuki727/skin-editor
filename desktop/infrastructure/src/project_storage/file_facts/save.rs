//! Real preparation-only Work association, never storage completion.
use super::*;
use intent_context as ctx;
// Field drop order keeps the wrapper reservation through WorkRelease, including
// early returns while preparing the candidate.
struct WorkOwner {
    work: SavePreparationWork,
    _lease: ByteLease,
}
#[cfg(all(test, windows))]
pub(super) fn release_work_observed(
    reader: &FileFactsReader,
    work: SavePreparationWork,
    observe: impl FnOnce(),
) {
    let owner = WorkOwner {
        work,
        _lease: reader.control.reserve(extra_bytes()).unwrap(),
    };
    let WorkOwner {
        work,
        _lease: lease,
    } = owner;
    drop(work);
    observe();
    drop(lease);
}
pub struct FileSaveCandidate {
    capture: FileCaptureCandidate,
    summary: FileSaveIntentSummary,
    metadata: Option<FileCapturedMetadataFacts>,
    _owner: WorkOwner,
}
impl std::fmt::Debug for FileSaveCandidate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FileSaveCandidate")
            .field("summary", &self.summary)
            .finish_non_exhaustive()
    }
}
impl FileSaveCandidate {
    pub(super) fn snapshot(&self) -> &CaptureSnapshot {
        &self.capture._snapshot
    }
    pub fn captured_metadata(&self) -> Option<FileCapturedMetadataFacts> {
        self.metadata
    }
    pub fn summary(&self) -> FileSaveIntentSummary {
        self.summary
    }
}
pub(super) fn extra_bytes() -> usize {
    2 * (std::mem::size_of::<FileSaveIntentSummary>()
        + std::mem::size_of::<Option<FileCapturedMetadataFacts>>()
        + std::mem::size_of::<SavePreparationWork>()
        + std::mem::size_of::<ByteLease>())
}
fn cancelled(c: &CaptureCancellation) -> Result<(), FileFactsError> {
    if c.is_cancelled() {
        Err(FileFactsError::Cancelled)
    } else {
        Ok(())
    }
}
fn build_intent(
    capture: &FileCaptureCandidate,
    ticket: SaveTicket,
    scratch: &mut ctx::Workspace,
) -> Result<FileSaveIntentSummary, FileFactsError> {
    let m = capture._snapshot.manifest();
    let navigation = ctx::navigation(
        m.head,
        m.next_alias,
        m.next_branch,
        &m.redo_preferences,
        scratch,
    )
    .map_err(storage)?;
    let archive_before = ctx::archive(&m.archive_metadata, None, scratch).map_err(storage)?;
    let legacy_request = ctx::legacy(ticket, m, capture.prefix, scratch).map_err(storage)?;
    let would_commit = if let Some(receipt) = predict(&m.archive_metadata, ticket) {
        Some(FileArchivePrediction {
            fact: (&receipt).into(),
            archive: ctx::archive(&m.archive_metadata, Some(&receipt), scratch).map_err(storage)?,
        })
    } else {
        None
    };
    Ok(FileSaveIntentSummary {
        version: FILE_SAVE_INTENT_VERSION,
        ticket,
        capture: capture.summary(),
        prefix: capture.prefix,
        next_alias: m.next_alias,
        next_branch: m.next_branch,
        legacy_request,
        navigation,
        archive_before,
        would_commit,
    })
}
pub(super) fn predict(metadata: &ArchiveMetadata, ticket: SaveTicket) -> Option<ArchiveReceipt> {
    let generation = ticket.target.expected.generation?.checked_add(1)?;
    if metadata
        .manual_marks
        .iter()
        .any(|(id, _)| *id == ticket.commit)
    {
        return None;
    }
    Some(ArchiveReceipt {
        commit: ticket.commit,
        generation,
        head: ticket.head,
        origin: ticket.origin,
    })
}
pub(super) fn legacy_fields_equal(q: FileSaveIntentSummary, c: FileCommitFacts) -> bool {
    c.previous == q.ticket.target.expected.commit
        && c.fact.origin == q.ticket.origin
        && c.fact.head == q.ticket.head
        && c.legacy_request == q.legacy_request
        && c.prefix == q.prefix
        && c.nodes == q.capture.nodes
        && c.next_alias == q.next_alias
        && c.next_branch == q.next_branch
}
pub(super) fn match_commit(
    q: FileSaveIntentSummary,
    c: FileCommitFacts,
    navigation: [u8; 32],
    archive: [u8; 32],
    latest: Option<SaveCommitFact>,
) -> FileSaveCommitEvidence {
    let is_latest = latest.is_some_and(|r| r.commit == c.fact.commit);
    FileSaveCommitEvidence {
        fact: c.fact,
        previous: c.previous,
        legacy_fields_equal: legacy_fields_equal(q, c),
        generation_equal: q
            .ticket
            .target
            .expected
            .generation
            .and_then(|n| n.checked_add(1))
            == Some(c.fact.generation),
        navigation_equal: navigation == q.navigation,
        is_latest,
        would_commit_archive_equal: if is_latest {
            q.would_commit.map(|p| p.archive == archive)
        } else {
            None
        },
    }
}
pub(super) fn capture_request(
    ticket: SaveTicket,
    snapshot: &CaptureSnapshot,
    selection: RestoreSelection,
) -> Result<FileFactsRequest, FileFactsError> {
    if ticket.key.version != SAVE_COORDINATION_VERSION
        || ticket.capture_version != HISTORY_CAPTURE_VERSION
    {
        return Err(FileFactsError::UnsupportedVersion);
    }
    let request = FileFactsRequest {
        version: FILE_FACTS_PORT_VERSION,
        key: FileFactsKey {
            session: ticket.key.session,
            sequence: ticket.key.sequence,
        },
        binding: ticket.target,
        selection,
    };
    validate_request(request)?;
    let m = snapshot.manifest();
    if ticket.source != m.source
        || ticket.head != m.head
        || ticket.history != SaveHistoryScope::from(&m.identity)
        || ticket.capture.0 == 0
        || ticket.coverage.0 == 0
        || m.archive_metadata.last_commit.as_ref().map(|r| r.commit)
            != ticket.target.expected.commit
        || m.archive_metadata
            .last_commit
            .as_ref()
            .map(|r| r.generation)
            != ticket.target.expected.generation
    {
        return Err(FileFactsError::TargetConflict);
    }
    Ok(request)
}

// Fixed comparison values can overlap the encoder and the charged capture.
// Only the caller's detached result survives the query.
pub(super) const CURSOR_WORK_BYTES: usize = ctx::WORK_BYTES
    + 2 * (size_of::<FileSaveComparison>()
        + size_of::<FileSaveIntentSummary>()
        + size_of::<FileFactsSummary>()
        + size_of::<FileFactsRequest>()
        + size_of::<FileSaveCompareContext>()
        + size_of::<SaveCaptureCursor<'static>>());

impl FileFactsReader {
    /// Traverse an unconsumed save capture and compare it against one held observation.
    /// The owner retains the execution and any protocol capabilities. The returned
    /// values describe this observation only. The observation's expected commit may
    /// have advanced. Readers can retry a preflight rejection on the same owner;
    /// after polling starts the owner's traversal position persists on every exit.
    pub fn compare_save_cursor(
        &self,
        mut cursor: SaveCaptureCursor<'_>,
        selection: RestoreSelection,
        originals: &BlobStore,
        view: &InspectedFileFacts,
        context: FileSaveCompareContext,
        cancel: CaptureCancellation,
    ) -> Result<FileSaveComparison, FileFactsError> {
        cancelled(&cancel)?;
        if context.version != FILE_SAVE_INTENT_VERSION {
            return Err(FileFactsError::UnsupportedVersion);
        }
        let _workspace = self
            .control
            .reserve(CURSOR_WORK_BYTES)
            .ok_or(FileFactsError::Limit(FileFactsLimit::Control))?;
        let ticket = cursor.ticket();
        let request = capture_request(ticket, cursor.snapshot(), selection)?;
        view.check_save_context(
            self.instance,
            request,
            ticket,
            cursor.snapshot().manifest().root,
            context,
        )?;
        let capture = self.prepare_stream(request, &mut cursor, originals, cancel.clone())?;
        let intent = build_intent(&capture, ticket, &mut [0; 4096])?;
        let result = view.compare_intent(intent);
        cancelled(&cancel)?;
        Ok(result)
    }

    pub fn prepare_save(
        &self,
        work: SavePreparationWork,
        selection: RestoreSelection,
        originals: &BlobStore,
        cancel: CaptureCancellation,
    ) -> Result<FileSaveCandidate, FileFactsError> {
        cancelled(&cancel)?;
        let ticket = work.ticket();
        let request = capture_request(ticket, work.snapshot(), selection)?;
        let lease = self
            .control
            .reserve(extra_bytes())
            .ok_or(FileFactsError::Limit(FileFactsLimit::Control))?;
        let mut owner = WorkOwner {
            work,
            _lease: lease,
        };
        let temporary = self
            .control
            .reserve(ctx::WORK_BYTES)
            .ok_or(FileFactsError::Limit(FileFactsLimit::Control))?;
        let mut cursor = owner
            .work
            .capture_cursor(SAVE_CAPTURE_CURSOR_VERSION)
            .map_err(|_| FileFactsError::UnsupportedVersion)?;
        let capture = self.prepare_stream(request, &mut cursor, originals, cancel.clone())?;
        let (summary, metadata) = {
            let mut scratch = [0; 4096];
            let summary = build_intent(&capture, ticket, &mut scratch)?;
            #[cfg(test)]
            let probe = self.probe;
            #[cfg(not(test))]
            let probe = Probe::default();
            let metadata = profile::captured(
                &capture._snapshot,
                ticket,
                self.instance,
                &mut scratch,
                &cancel,
                probe,
            )?;
            (summary, metadata)
        }; // Both projections and their shared workspace have left scope.
        cancelled(&cancel)?;
        drop(temporary);
        Ok(FileSaveCandidate {
            capture,
            summary,
            metadata,
            _owner: owner,
        })
    }
}
impl InspectedFileFacts {
    pub fn compare_save(
        &self,
        candidate: &FileSaveCandidate,
        context: FileSaveCompareContext,
    ) -> Result<FileSaveComparison, FileFactsError> {
        if context.version != FILE_SAVE_INTENT_VERSION {
            return Err(FileFactsError::UnsupportedVersion);
        }
        let q = candidate.summary;
        self.check_save_context(
            candidate.capture.reader_instance,
            q.capture.request,
            q.ticket,
            q.capture.root,
            context,
        )?;
        Ok(self.compare_intent(q))
    }

    pub(super) fn check_save_context(
        &self,
        reader: u128,
        original: FileFactsRequest,
        ticket: SaveTicket,
        root: PersistentNodeId,
        context: FileSaveCompareContext,
    ) -> Result<(), FileFactsError> {
        let f = &self.data.summary;
        let a = original.binding;
        let b = f.request.binding;
        let scope_equal = original.version == f.request.version
            && original.key == f.request.key
            && original.selection == f.request.selection
            && a.version == b.version
            && a.key == b.key
            && a.target == b.target
            && a.project == b.project
            && a.phase == b.phase
            && a.history == b.history;
        if context.ticket != ticket
            || context.observation != f.observation
            || !scope_equal
            || reader != f.observation.instance
        {
            return Err(FileFactsError::StalePage);
        }
        if root != self.data.root {
            return Err(FileFactsError::TargetConflict);
        }
        Ok(())
    }

    fn compare_intent(&self, q: FileSaveIntentSummary) -> FileSaveComparison {
        let f = self.summary();
        let baseline_expected_equal = q.ticket.target.expected == f.request.binding.expected;
        let commit = self.data.commits[..f.commits]
            .iter()
            .position(|c| c.is_some_and(|c| c.fact.commit == q.ticket.commit))
            .map(|i| {
                let c = self.data.commits[i].expect("fixed fact");
                match_commit(
                    q,
                    c,
                    self.data.commit_navigation[i],
                    self.data.intent.archive,
                    f.files.latest,
                )
            });
        FileSaveComparison {
            version: FILE_SAVE_INTENT_VERSION,
            intent: q,
            observation: f.observation,
            observed_expected: f.request.binding.expected,
            baseline_expected_equal,
            content_equal: q.capture.projection.content == f.proof.content,
            originals_equal: q.capture.projection.originals == f.proof.originals,
            opening_navigation_equal: q.navigation == self.data.intent.opening_navigation,
            latest_navigation_equal: q.navigation == self.data.intent.latest_navigation,
            before_archive_equal: baseline_expected_equal
                .then_some(q.archive_before == self.data.intent.archive),
            commit,
            conclusion: FileCaptureConclusion::InsufficientForSaveReconciliation,
        }
    }
}
