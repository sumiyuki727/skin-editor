use super::*;

pub const FILE_REGISTERED_COVERAGE_VERSION: u16 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileRegisteredCoverageRequest {
    pub version: u16,
    pub stamp: FileTargetStamp,
    pub ticket: SaveTicket,
}

/// Fixed evidence about the original registered opening at query time.
/// A detached observation neither retains the registration nor acknowledges a save.
/// ```compile_fail
/// use skin_editor_application::PreparedWitness;
/// use skin_editor_infrastructure::project_storage::FileRegisteredCoverage;
/// fn install(v: FileRegisteredCoverage) -> PreparedWitness { v.into() }
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileRegisteredCoverage {
    pub request: FileRegisteredCoverageRequest,
    pub coverage: Option<FileNoChangeCoverage>,
}
const _: () = assert!(size_of::<FileRegisteredCoverage>() <= 8192);

// The caller owns the detached result. The stream charges its retained snapshot,
// root table and encoder separately, before its first poll.
const WORK_BYTES: usize = intent_context::WORK_BYTES
    + 2 * (size_of::<FileRegisteredCoverage>()
        + size_of::<SaveCaptureCursor<'static>>()
        + size_of::<FileRegisteredCoverageRequest>()
        + size_of::<FileCapturedMetadataContext>()
        + size_of::<FileCaptureSummary>()
        + size_of::<FileCapturedMetadataFacts>()
        + size_of::<FileHistoricalMetadataResult>()
        + size_of::<FileFactsRequest>()
        + size_of::<FileFactsSummary>());

fn check_cancel(
    cursor: &SaveCaptureCursor<'_>,
    cancel: &CaptureCancellation,
) -> Result<(), FileFactsError> {
    if cursor.is_cancelled() || cancel.is_cancelled() {
        Err(FileFactsError::Cancelled)
    } else {
        Ok(())
    }
}

impl FileFactsReader {
    #[cfg(all(test, windows))]
    pub(crate) fn hold_registered_coverage_margin(&self, short_by: usize) -> ByteLease {
        let peak = WORK_BYTES + capture::retained_bytes() + capture::work_bytes();
        self.control
            .reserve(self.limits.control_bytes - self.control.used() - peak + short_by)
            .unwrap()
    }

    /// Compare the complete frozen capture to its continuously held opening.
    /// A missing eligible baseline returns no coverage. A used stream cannot
    /// be replayed. Only the owner can publish protocol outcomes.
    ///
    /// An already admitted close execution can lend the child verified against
    /// its parent's key. The example does not issue the execution or any authority.
    /// ```no_run
    /// use skin_editor_application::{CaptureCancellation, CloseAdmissionKey, FileFactsError,
    ///     SAVE_CAPTURE_CURSOR_VERSION, SaveExecution};
    /// use skin_editor_infrastructure::{BlobStore, project_storage::*};
    /// fn inspect_child(reader: &FileFactsReader, registry: &mut FileTargetRegistry,
    ///     request: FileRegisteredCoverageRequest, close: CloseAdmissionKey, work: &mut SaveExecution,
    ///     originals: &BlobStore, cancel: CaptureCancellation) -> Result<FileRegisteredCoverage, FileFactsError> {
    ///     let cursor = work.exit_capture_cursor(SAVE_CAPTURE_CURSOR_VERSION, close)
    ///         .map_err(|_| FileFactsError::StalePage)?;
    ///     reader.compare_registered_coverage(registry, request, cursor, originals, cancel)
    /// }
    /// ```
    pub fn compare_registered_coverage(
        &self,
        registry: &mut FileTargetRegistry,
        request: FileRegisteredCoverageRequest,
        mut cursor: SaveCaptureCursor<'_>,
        originals: &BlobStore,
        cancel: CaptureCancellation,
    ) -> Result<FileRegisteredCoverage, FileFactsError> {
        if request.version != FILE_REGISTERED_COVERAGE_VERSION {
            return Err(FileFactsError::UnsupportedVersion);
        }
        let ticket = cursor.ticket();
        let view = registry.opening_for_capture(request.stamp, &cursor, request.ticket, &cancel)?;
        let summary = &view.data.summary;
        if summary.observation.instance != self.instance {
            return Err(FileFactsError::StalePage);
        }
        if let Some(token) = self.pending_cleanup() {
            return Err(FileFactsError::Cleanup { token });
        }
        if cursor.statistics().nodes_emitted != 0 || cursor.snapshot().is_complete() {
            return Err(FileFactsError::InvalidRequest);
        }
        let _workspace = self
            .control
            .reserve(WORK_BYTES)
            .ok_or(FileFactsError::Limit(FileFactsLimit::Control))?;
        let capture_request =
            save::capture_request(ticket, cursor.snapshot(), summary.request.selection)?;
        let context = FileCapturedMetadataContext {
            version: FILE_NO_CHANGE_COVERAGE_VERSION,
            observation: summary.observation,
            file: summary.request,
            ticket,
            commit: ticket
                .target
                .expected
                .commit
                .ok_or(FileFactsError::InvalidRequest)?,
        };
        let selected = view
            .historical_metadata(FileHistoricalMetadataRequest {
                version: FILE_HISTORICAL_METADATA_VERSION,
                observation: context.observation,
                file: context.file,
                commit: context.commit,
            })?
            .facts;
        let mut scratch = [0; 4096];
        #[cfg(test)]
        let probe = self.probe;
        #[cfg(not(test))]
        let probe = Probe::default();
        let captured = captured(
            cursor.snapshot(),
            ticket,
            self.instance,
            &mut scratch,
            &cancel,
            probe,
        )?;
        let metadata = view.check_original_metadata(context, captured, selected)?;
        check_cancel(&cursor, &cancel)?;
        let coverage = if let Some(metadata) = metadata {
            let capture =
                self.prepare_stream(capture_request, &mut cursor, originals, cancel.clone())?;
            check_cancel(&cursor, &cancel)?;
            let manifest = capture._snapshot.manifest();
            let navigation = intent_context::navigation(
                manifest.head,
                manifest.next_alias,
                manifest.next_branch,
                &manifest.redo_preferences,
                &mut scratch,
            )
            .map_err(storage)?;
            Some(view.coverage_facts(
                context,
                capture.summary(),
                manifest,
                navigation,
                metadata,
                &mut scratch,
            )?)
        } else {
            None
        };
        self.finish_coverage(&cancel)?;
        // Complete is sticky even if the original work was subsequently cancelled.
        check_cancel(&cursor, &cancel)?;
        Ok(FileRegisteredCoverage { request, coverage })
    }
}
