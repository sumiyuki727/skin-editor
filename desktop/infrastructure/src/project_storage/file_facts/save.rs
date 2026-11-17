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
    pub fn summary(&self) -> FileSaveIntentSummary {
        self.summary
    }
}
pub(super) fn extra_bytes() -> usize {
    2 * (std::mem::size_of::<FileSaveIntentSummary>()
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
) -> Result<FileSaveIntentSummary, FileFactsError> {
    let m = capture._snapshot.manifest();
    let mut scratch = [0; 4096];
    let navigation = ctx::navigation(
        m.head,
        m.next_alias,
        m.next_branch,
        &m.redo_preferences,
        &mut scratch,
    )
    .map_err(storage)?;
    let archive_before = ctx::archive(&m.archive_metadata, None, &mut scratch).map_err(storage)?;
    let legacy_request = ctx::legacy(ticket, m, capture.prefix, &mut scratch).map_err(storage)?;
    let would_commit = if let Some(receipt) = predict(&m.archive_metadata, ticket) {
        Some(FileArchivePrediction {
            fact: (&receipt).into(),
            archive: ctx::archive(&m.archive_metadata, Some(&receipt), &mut scratch)
                .map_err(storage)?,
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
impl FileFactsReader {
    pub fn prepare_save(
        &self,
        work: SavePreparationWork,
        selection: RestoreSelection,
        originals: &BlobStore,
        cancel: CaptureCancellation,
    ) -> Result<FileSaveCandidate, FileFactsError> {
        cancelled(&cancel)?;
        let ticket = work.ticket();
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
        let m = work.snapshot().manifest();
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
        let capture = self.prepare_stream(request, &mut owner.work, originals, cancel.clone())?;
        let summary = build_intent(&capture, ticket)?;
        cancelled(&cancel)?;
        drop(temporary);
        Ok(FileSaveCandidate {
            capture,
            summary,
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
        let f = self.summary();
        let original = q.capture.request;
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
        if context.ticket != q.ticket
            || context.observation != f.observation
            || !scope_equal
            || candidate.capture.reader_instance != f.observation.instance
        {
            return Err(FileFactsError::StalePage);
        }
        if q.capture.root != self.data.root {
            return Err(FileFactsError::TargetConflict);
        }
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
        Ok(FileSaveComparison {
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
        })
    }
}
