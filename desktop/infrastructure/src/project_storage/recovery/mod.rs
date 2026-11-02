//! Explicit source-preserving recovery into a new file, never an in-place open.
use super::*;
mod journal;
mod source;

pub const RECOVERY_VERSION: u16 = 1;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveryDigest {
    pub byte_len: u64,
    pub sha256: [u8; 32],
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecoveryMode {
    VerifiedCopy,
    JournalRollback,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveryReport {
    pub version: u16,
    pub mode: RecoveryMode,
    pub source_main: RecoveryDigest,
    pub source_journal: Option<RecoveryDigest>,
    pub journal_records: u64,
    pub restored: ArchiveReceipt,
}
#[derive(Debug)]
pub struct RecoveredArchive {
    pub archive: SqliteProjectFile,
    pub candidate: RestoreCandidate,
    pub report: RecoveryReport,
}
impl SqliteProjectFile {
    /// The source and its sidecars remain byte-for-byte intact. Windows uses
    /// read-only exclusive sharing leases; other platforms reject this use case.
    pub fn recover_copy(
        source: impl AsRef<Path>,
        target: impl AsRef<Path>,
        originals: BlobStore,
        limits: FileLimits,
        selection: RestoreSelection,
        cancel: CaptureCancellation,
    ) -> Result<RecoveredArchive, StorageError> {
        recover(
            source.as_ref(),
            target.as_ref(),
            originals,
            limits,
            selection,
            &cancel,
            Probe::default(),
        )
    }
}
// Private call bundle keeps the fault harness out of the public port.
pub(super) fn recover(
    source_path: &Path,
    target: &Path,
    originals: BlobStore,
    limits: FileLimits,
    selection: RestoreSelection,
    cancel: &CaptureCancellation,
    probe: Probe,
) -> Result<RecoveredArchive, StorageError> {
    check(cancel)?;
    source::new_target(target)?;
    source::separate_target(source_path, target)?;
    let mut frozen = source::Frozen::open(source_path, limits, cancel)?;
    let records = journal::validate(&mut frozen, limits, cancel)?;
    let mut scratch = source::Scratch::create(target)?;
    let result = Work {
        originals,
        limits,
        selection,
        cancel,
        probe,
        records,
    }
    .attempt(&mut frozen, &mut scratch, target);
    let result = match result {
        Err(e) if cancel.is_cancelled() && !matches!(e, StorageError::OutcomeUnknown(_)) => {
            Err(StorageError::Cancelled)
        }
        other => other,
    };
    if scratch.cleanup(probe).is_err() {
        let cause = match result {
            Ok(r) => StorageError::OutcomeUnknown(r.report.restored.commit),
            Err(e) => e,
        };
        return Err(StorageError::RecoveryCleanup {
            token: scratch.token,
            cause: Box::new(cause),
        });
    }
    result
}
struct Work<'a> {
    originals: BlobStore,
    limits: FileLimits,
    selection: RestoreSelection,
    cancel: &'a CaptureCancellation,
    probe: Probe,
    records: u64,
}
impl Work<'_> {
    fn attempt(
        self,
        frozen: &mut source::Frozen,
        scratch: &mut source::Scratch,
        target: &Path,
    ) -> Result<RecoveredArchive, StorageError> {
        let Self {
            originals,
            limits,
            selection,
            cancel,
            probe,
            records,
        } = self;
        use std::fs::OpenOptions;
        scratch.copy(frozen, cancel, probe)?;
        probe.step(Stage::BeforeRecovery, cancel)?;
        let mut conn = Connection::open_with_flags(
            &scratch.path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE,
        )?;
        configure(&mut conn, limits, cancel)?;
        // SQLite may roll back a hot journal even for a SELECT. Only this owned
        // byte copy is passed to SQLite; the source has no SQLite connection here.
        conn.query_row("PRAGMA schema_version", [], |r| r.get::<_, i64>(0))?;
        probe.step(Stage::AfterRecovery, cancel)?;
        let (cache, candidate) =
            read::full_probe(&mut conn, &originals, limits, selection, cancel, probe)?;
        probe.step(Stage::RecoveryValidated, cancel)?;
        sidecars(&scratch.path)?;
        let receipt = cache.checkpoints.last().expect("validated").receipt.clone();
        conn.close().map_err(|_| StorageError::Io)?;
        OpenOptions::new()
            .read(true)
            .write(true)
            .open(&scratch.path)
            .map_err(io)?
            .sync_all()
            .map_err(io)?;
        frozen.verify(cancel)?;
        let stage_stamp = FileStamp::read(&scratch.path)?;
        probe.step(Stage::BeforePublish, cancel)?;
        source::new_target(target)?;
        std::fs::hard_link(&scratch.path, target).map_err(|e| {
            if e.kind() == std::io::ErrorKind::AlreadyExists {
                StorageError::TargetExists
            } else {
                io(e)
            }
        })?;
        if probe.step(Stage::AfterPublish, cancel).is_err() {
            return Err(StorageError::OutcomeUnknown(receipt.commit));
        }
        let stamp =
            FileStamp::read(target).map_err(|_| StorageError::OutcomeUnknown(receipt.commit))?;
        if stamp.handle != stage_stamp.handle {
            return Err(StorageError::OutcomeUnknown(receipt.commit));
        }
        let mut conn =
            Connection::open_with_flags(target, rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE)
                .map_err(|_| StorageError::OutcomeUnknown(receipt.commit))?;
        configure(&mut conn, limits, &CaptureCancellation::default())
            .map_err(|_| StorageError::OutcomeUnknown(receipt.commit))?;
        conn.progress_handler(0, None::<fn() -> bool>)
            .map_err(|_| StorageError::OutcomeUnknown(receipt.commit))?;
        let data_version = conn
            .query_row("PRAGMA data_version", [], |r| r.get(0))
            .map_err(|_| StorageError::OutcomeUnknown(receipt.commit))?;
        let report = RecoveryReport {
            version: RECOVERY_VERSION,
            mode: if records > 0 {
                RecoveryMode::JournalRollback
            } else {
                RecoveryMode::VerifiedCopy
            },
            source_main: frozen.main.digest.clone(),
            source_journal: frozen.journal.as_ref().map(|j| j.digest.clone()),
            journal_records: records,
            restored: receipt,
        };
        Ok(RecoveredArchive {
            archive: SqliteProjectFile {
                conn,
                path: target.to_path_buf(),
                stamp,
                data_version,
                cache,
                originals,
                limits,
                invalid: false,
            },
            candidate,
            report,
        })
    }
}
