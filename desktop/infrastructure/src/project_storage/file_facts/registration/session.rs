use super::*;

/// Association at initial open, held only while the session and registered
/// opening remain borrowed. This is not an installation token.
/// ```compile_fail,E0499
/// use skin_editor_application::{CaptureCancellation, FILE_SESSION_OPENING_VERSION, ProjectSession};
/// use skin_editor_infrastructure::project_storage::{FileFactsReader, FileTargetRegistry, FileTargetStamp};
/// fn rebind(r: &mut FileTargetRegistry, reader: &FileFactsReader, s: &ProjectSession, stamp: FileTargetStamp) {
///     let pair = r.preflight_session(reader, FILE_SESSION_OPENING_VERSION, stamp, s, &CaptureCancellation::default()).unwrap();
///     r.release(stamp).unwrap();
///     let _ = pair.coverage();
/// }
/// ```
/// ```compile_fail,E0502
/// use skin_editor_application::{CaptureCancellation, FILE_SESSION_OPENING_VERSION, ProjectSession, SavePreparationConfig};
/// use skin_editor_infrastructure::project_storage::{FileFactsReader, FileTargetRegistry, FileTargetStamp};
/// fn install(r: &mut FileTargetRegistry, reader: &FileFactsReader, s: &mut ProjectSession, stamp: FileTargetStamp, cfg: SavePreparationConfig) {
///     let pair = r.preflight_session(reader, FILE_SESSION_OPENING_VERSION, stamp, s, &CaptureCancellation::default()).unwrap();
///     let _ = s.install_save_preparation(cfg);
///     let _ = pair.coverage();
/// }
/// ```
pub struct RegisteredSessionOpening<'a> {
    session: RestoredSessionOpening<'a>,
    view: &'a InspectedFileFacts,
    stamp: FileTargetStamp,
    _lease: ByteLease,
}
impl RegisteredSessionOpening<'_> {
    pub fn coverage(&self) -> SaveCoverageStamp {
        self.session.coverage()
    }
    pub fn observation(&self) -> FileObservationId {
        self.view.data.summary.observation
    }
    pub fn file(&self) -> FileFactsRequest {
        self.view.data.summary.request
    }
    pub fn stamp(&self) -> FileTargetStamp {
        self.stamp
    }
}

const RETAINED_BYTES: usize = 2 * size_of::<RegisteredSessionOpening<'static>>();
const WORK_BYTES: usize = intent_context::WORK_BYTES
    + 2 * (size_of::<FileCapturedMetadataFacts>()
        + size_of::<FileFactsRequest>()
        + size_of::<FileProfileOrigin>()
        + size_of::<SaveFileFacts>()
        + size_of::<SaveCoverageStamp>());

impl FileTargetRegistry {
    /// Authenticate an activated file source against the currently held opening
    /// before any coordinator or capture exists. No file is reopened here.
    pub fn preflight_session<'a>(
        &'a mut self,
        reader: &FileFactsReader,
        version: u16,
        expected: FileTargetStamp,
        session: &'a ProjectSession,
        cancel: &CaptureCancellation,
    ) -> Result<RegisteredSessionOpening<'a>, FileFactsError> {
        if version != FILE_SESSION_OPENING_VERSION {
            return Err(FileFactsError::UnsupportedVersion);
        }
        self.check_stamp(expected)?;
        if self.reader != reader.instance {
            return Err(FileFactsError::StalePage);
        }
        check(cancel).map_err(storage)?;
        if let Some(token) = reader.pending_cleanup() {
            return Err(FileFactsError::Cleanup { token });
        }
        let view = self.active.as_ref().ok_or(FileFactsError::InvalidRequest)?;
        let summary = &view.data.summary;
        let lease = reader
            .control
            .reserve(RETAINED_BYTES)
            .ok_or(FileFactsError::Limit(FileFactsLimit::Control))?;
        let _work = reader
            .control
            .reserve(WORK_BYTES)
            .ok_or(FileFactsError::Limit(FileFactsLimit::Control))?;
        let session = session.restored_opening(version, summary.request)?;
        #[cfg(test)]
        let probe = reader.probe;
        #[cfg(not(test))]
        let probe = Probe::default();
        let metadata =
            profile::session_opening_metadata(&session, view, &mut [0; 4096], cancel, probe)?;
        if metadata.metadata != summary.proof.metadata
            || SaveFileFacts::from(session.metadata()) != summary.files
            || session.coverage().nodes != summary.nodes
            || session.coverage().admissions != summary.admissions
            || session.originals() != summary.originals
        {
            return Err(FileFactsError::TargetConflict);
        }
        check(cancel).map_err(storage)?;
        Ok(RegisteredSessionOpening {
            session,
            view,
            stamp: expected,
            _lease: lease,
        })
    }
}

#[cfg(all(test, windows))]
impl FileFactsReader {
    pub(crate) fn hold_session_preflight_margin(&self, short_by: usize) -> ByteLease {
        self.control
            .reserve(
                self.limits.control_bytes - self.control.used() - RETAINED_BYTES - WORK_BYTES
                    + short_by,
            )
            .unwrap()
    }
}
