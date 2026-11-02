//! Rust archive profile. No UI or active project write access.
mod blocks;
mod read;
mod schema;
mod values;
mod write;
use crate::project_format::{
    CodecError, FormatLimits, StateBlock, StateBundle, StateHash, decode_project, encode_project,
};
use crate::{BlobStore, RawBlob};
use rusqlite::{Connection, params};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use skin_editor_application::{values::*, *};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use values::*;

pub const FILE_PROFILE: &str = "skin-editor-rust-v1";
pub const CONTAINER_VERSION: u32 = 2;
const APP_ID: u32 = 0x534b4550;
#[derive(Clone, Copy, Debug)]
pub struct FileLimits {
    pub file_bytes: u64,
    pub metadata_bytes: usize,
    pub blocks: usize,
    pub originals: usize,
    pub original_bytes: u64,
    pub decoded_bytes: usize,
    pub restore: RestoreLimits,
    pub codec: FormatLimits,
}
impl Default for FileLimits {
    fn default() -> Self {
        Self {
            file_bytes: 1024 * 1024 * 1024,
            metadata_bytes: 32 * 1024 * 1024,
            blocks: 131072,
            originals: 4096,
            original_bytes: 128 * 1024 * 1024,
            decoded_bytes: 64 * 1024 * 1024,
            restore: RestoreLimits::default(),
            codec: FormatLimits::default(),
        }
    }
}
#[derive(Debug, PartialEq)]
pub enum StorageError {
    Io,
    Sql {
        code: Option<i32>,
        class: &'static str,
    },
    Busy,
    Cancelled,
    TargetExists,
    ExternalChange,
    Invalidated,
    JournalPresent,
    Invalid(&'static str),
    Unsupported(&'static str),
    Limit(&'static str),
    Conflict,
    Capture(CaptureError),
    Restore(RestoreError),
    Codec(CodecError),
    MissingOriginal(BlobRef),
    OutcomeUnknown(CommitId),
}
impl From<rusqlite::Error> for StorageError {
    fn from(e: rusqlite::Error) -> Self {
        match e.sqlite_error_code() {
            Some(rusqlite::ErrorCode::DatabaseBusy | rusqlite::ErrorCode::DatabaseLocked) => {
                Self::Busy
            }
            _ => Self::Sql {
                code: e.sqlite_error().map(|s| s.extended_code),
                class: match e {
                    rusqlite::Error::InvalidColumnType(..) => "column type",
                    rusqlite::Error::QueryReturnedNoRows => "missing row",
                    rusqlite::Error::FromSqlConversionFailure(..) => "conversion",
                    _ => "sqlite",
                },
            },
        }
    }
}
impl From<CaptureError> for StorageError {
    fn from(e: CaptureError) -> Self {
        Self::Capture(e)
    }
}
impl From<RestoreError> for StorageError {
    fn from(e: RestoreError) -> Self {
        Self::Restore(e)
    }
}
impl From<CodecError> for StorageError {
    fn from(e: CodecError) -> Self {
        Self::Codec(e)
    }
}
fn io(_: std::io::Error) -> StorageError {
    StorageError::Io
}
fn check(c: &CaptureCancellation) -> Result<(), StorageError> {
    if c.is_cancelled() {
        Err(StorageError::Cancelled)
    } else {
        Ok(())
    }
}

#[derive(Clone)]
struct Header {
    project: ProjectId,
    identity: PersistentHistoryIdentity,
    root: PersistentNodeId,
}
#[derive(Clone)]
struct CachedNode {
    info: CapturedNodeInfo,
    canonical: StateHash,
}
#[derive(Clone)]
struct Checkpoint {
    receipt: ArchiveReceipt,
    previous: Option<CommitId>,
    navigation: StateHash,
    prefix: [u8; 32],
    node_count: usize,
    next_alias: u64,
    next_branch: u64,
    redo: Vec<(PersistentNodeId, PersistentNodeId)>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Stage {
    Originals,
    States,
    Nodes,
    BeforeCommit,
    AfterCommit,
    BeforePublish,
    AfterPublish,
    ReadOriginal,
    ReadNodes,
}
#[derive(Clone, Copy, Default)]
struct Probe {
    fault: Option<Stage>,
    cancel_at: Option<Stage>,
}
impl Probe {
    fn step(self, stage: Stage, cancel: &CaptureCancellation) -> Result<(), StorageError> {
        if self.cancel_at == Some(stage) {
            cancel.cancel();
        }
        check(cancel)?;
        if self.fault == Some(stage) {
            Err(StorageError::Io)
        } else {
            Ok(())
        }
    }
}
struct Cache {
    header: Header,
    blocks: BTreeMap<StateHash, blocks::Row>,
    nodes: BTreeMap<PersistentNodeId, CachedNode>,
    order: Vec<PersistentNodeId>,
    admissions: BTreeMap<ObjectId, PersistentNodeId>,
    originals: BTreeSet<BlobRef>,
    checkpoints: Vec<Checkpoint>,
    metadata: Arc<ArchiveMetadata>,
    features: Vec<Value>,
    metadata_bytes: usize,
    prefix: [u8; 32],
}
struct FileStamp {
    handle: same_file::Handle,
    len: u64,
    modified: std::time::SystemTime,
    header: [u8; 100],
}
impl FileStamp {
    fn read(path: &Path) -> Result<Self, StorageError> {
        if std::fs::symlink_metadata(path)
            .map_err(io)?
            .file_type()
            .is_symlink()
        {
            return Err(StorageError::Unsupported("symlink target"));
        }
        use std::io::Read;
        let mut file = std::fs::File::open(path).map_err(io)?;
        let m = file.metadata().map_err(io)?;
        if !m.is_file() {
            return Err(StorageError::Invalid("file"));
        }
        let mut header = [0; 100];
        file.read_exact(&mut header).map_err(io)?;
        Ok(Self {
            handle: same_file::Handle::from_file(file).map_err(io)?,
            len: m.len(),
            modified: m.modified().map_err(io)?,
            header,
        })
    }
    fn matches(&self, path: &Path) -> Result<bool, StorageError> {
        let now = Self::read(path)?;
        Ok(self.handle == now.handle
            && self.len == now.len
            && self.modified == now.modified
            && self.header == now.header)
    }
}
pub struct SqliteProjectFile {
    conn: Connection,
    path: PathBuf,
    stamp: FileStamp,
    data_version: i64,
    cache: Cache,
    originals: BlobStore,
    limits: FileLimits,
    invalid: bool,
}
impl std::fmt::Debug for SqliteProjectFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SqliteProjectFile")
            .field("profile", &FILE_PROFILE)
            .finish_non_exhaustive()
    }
}
fn sidecars(path: &Path) -> Result<(), StorageError> {
    for suffix in ["-journal", "-wal", "-shm"] {
        let mut name = path.as_os_str().to_os_string();
        name.push(suffix);
        match std::fs::metadata(PathBuf::from(name)) {
            Ok(m) if m.len() > 0 => return Err(StorageError::JournalPresent),
            Ok(_) => (),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
            Err(e) => return Err(io(e)),
        }
    }
    Ok(())
}
fn configure(
    c: &mut Connection,
    limits: FileLimits,
    cancel: &CaptureCancellation,
) -> Result<(), StorageError> {
    c.busy_timeout(std::time::Duration::ZERO)?;
    c.set_db_config(rusqlite::config::DbConfig::SQLITE_DBCONFIG_DEFENSIVE, true)?;
    c.execute_batch("PRAGMA foreign_keys=ON; PRAGMA trusted_schema=OFF; PRAGMA synchronous=EXTRA; PRAGMA cache_size=-2048; PRAGMA mmap_size=0; PRAGMA temp_store=FILE;")?;
    c.set_limit(
        rusqlite::limits::Limit::SQLITE_LIMIT_LENGTH,
        2 * 1024 * 1024,
    )?;
    c.set_limit(rusqlite::limits::Limit::SQLITE_LIMIT_SQL_LENGTH, 65536)?;
    if limits.file_bytes == 0
        || limits.metadata_bytes == 0
        || limits.blocks == 0
        || limits.originals == 0
        || limits.decoded_bytes == 0
    {
        return Err(StorageError::Invalid("limits"));
    }
    let flag = cancel.clone();
    c.progress_handler(1000, Some(move || flag.is_cancelled()))?;
    Ok(())
}
struct Transaction<'a> {
    conn: &'a mut Connection,
    active: bool,
}
impl<'a> Transaction<'a> {
    fn begin(conn: &'a mut Connection, write: bool) -> Result<Self, StorageError> {
        conn.execute_batch(if write { "BEGIN IMMEDIATE" } else { "BEGIN" })?;
        Ok(Self { conn, active: true })
    }
    fn commit(&mut self) -> Result<(), StorageError> {
        self.conn.progress_handler(0, None::<fn() -> bool>)?;
        self.conn.execute_batch("COMMIT")?;
        self.active = false;
        Ok(())
    }
}
impl Drop for Transaction<'_> {
    fn drop(&mut self) {
        let _ = self.conn.progress_handler(0, None::<fn() -> bool>);
        if self.active {
            let _ = self.conn.execute_batch("ROLLBACK");
        }
    }
}

impl SqliteProjectFile {
    pub fn load(
        path: impl AsRef<Path>,
        originals: BlobStore,
        limits: FileLimits,
        selection: RestoreSelection,
        cancel: CaptureCancellation,
    ) -> Result<(Self, RestoreCandidate), StorageError> {
        let path = path.as_ref().to_path_buf();
        sidecars(&path)?;
        check(&cancel)?;
        let stamp = FileStamp::read(&path)?;
        if stamp.len > limits.file_bytes {
            return Err(StorageError::Limit("file bytes"));
        }
        let mut reader =
            Connection::open_with_flags(&path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        configure(&mut reader, limits, &cancel)?;
        let (cache, candidate) = read::full(&mut reader, &originals, limits, selection, &cancel)?;
        if !stamp.matches(&path)? {
            return Err(StorageError::ExternalChange);
        }
        drop(reader);
        check(&cancel)?;
        let mut conn =
            Connection::open_with_flags(&path, rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE)?;
        configure(&mut conn, limits, &cancel)?;
        conn.progress_handler(0, None::<fn() -> bool>)?;
        if !stamp.matches(&path)? {
            return Err(StorageError::ExternalChange);
        }
        let data_version = conn.query_row("PRAGMA data_version", [], |r| r.get(0))?;
        Ok((
            Self {
                conn,
                path,
                stamp,
                data_version,
                cache,
                originals,
                limits,
                invalid: false,
            },
            candidate,
        ))
    }
    fn guard(&self) -> Result<(), StorageError> {
        if self.invalid {
            return Err(StorageError::Invalidated);
        }
        if !self.stamp.matches(&self.path)?
            || self
                .conn
                .query_row("PRAGMA data_version", [], |r| r.get::<_, i64>(0))?
                != self.data_version
        {
            return Err(StorageError::ExternalChange);
        }
        Ok(())
    }
    pub fn receipt(&self) -> Result<&ArchiveReceipt, StorageError> {
        self.guard()?;
        Ok(&self
            .cache
            .checkpoints
            .last()
            .expect("validated commit")
            .receipt)
    }
    /// Resolve a possibly committed request after reopening an uncertain outcome.
    pub fn committed(&self, id: CommitId) -> Result<Option<&ArchiveReceipt>, StorageError> {
        self.guard()?;
        Ok(self
            .cache
            .checkpoints
            .iter()
            .find(|p| p.receipt.commit == id)
            .map(|p| &p.receipt))
    }
    pub fn sqlite_version() -> String {
        rusqlite::version().to_owned()
    }
}
impl ProjectArchive for SqliteProjectFile {
    type Error = StorageError;
    fn write_capture(
        &mut self,
        capture: &mut HistoryCapture,
        request: ArchiveWriteRequest,
    ) -> Result<ArchiveWriteOutcome, StorageError> {
        if let Err(e) = self.guard() {
            capture.cancel();
            return Err(e);
        }
        let result = write::existing(self, capture, request, Probe::default());
        if result.is_err() {
            capture.cancel();
            self.invalid = true;
        }
        result
    }
    fn restore_candidate(
        &mut self,
        selection: RestoreSelection,
        cancel: CaptureCancellation,
    ) -> Result<RestoreCandidate, StorageError> {
        self.guard()?;
        sidecars(&self.path)?;
        let (_, candidate) = read::full(
            &mut self.conn,
            &self.originals,
            self.limits,
            selection,
            &cancel,
        )?;
        Ok(candidate)
    }
}

#[cfg(test)]
mod tests;
