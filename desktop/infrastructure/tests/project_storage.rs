use skin_editor_application::{values::*, *};
use skin_editor_infrastructure::{
    BlobStore,
    project_storage::{FileLimits, SqliteProjectFile, StorageError},
};
use std::path::{Path, PathBuf};

struct Directory {
    path: PathBuf,
    parent: PathBuf,
}
impl Directory {
    fn new() -> Self {
        let parent = std::env::temp_dir().canonicalize().unwrap();
        let mut nonce = [0; 16];
        getrandom::fill(&mut nonce).unwrap();
        let name = nonce.iter().map(|n| format!("{n:02x}")).collect::<String>();
        let path = parent.join(format!("skin-editor-archive-{name}"));
        std::fs::create_dir(&path).unwrap();
        Self { path, parent }
    }
    fn file(&self) -> PathBuf {
        self.path.join("test.oskproj")
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        if self.path.canonicalize().ok().is_some_and(|p| {
            p.parent() == Some(self.parent.as_path())
                && p.file_name()
                    .is_some_and(|n| n.to_string_lossy().starts_with("skin-editor-archive-"))
        }) {
            if let Ok(entries) = std::fs::read_dir(&self.path) {
                for e in entries.flatten() {
                    let _ = std::fs::remove_file(e.path());
                }
            }
            let _ = std::fs::remove_dir(&self.path);
        }
    }
}
struct Ids(u128);
impl HistoryIdentitySource for Ids {
    fn next_identity(&mut self) -> Result<u128, IdentityError> {
        self.0 += 1;
        Ok(self.0)
    }
}
struct Driver {
    session: ProjectSession,
    request: u128,
    ids: Ids,
}
impl Driver {
    fn new(store: BlobStore) -> Self {
        Self {
            session: ProjectSession::with_resources(
                7,
                70,
                Box::new(store),
                SessionLimits::default(),
            )
            .unwrap(),
            request: 0,
            ids: Ids(1000),
        }
    }
    fn command(&mut self, commands: Vec<Command>) -> CommandResult {
        self.request += 1;
        let i = self.session.project_info();
        self.session
            .submit(CommandRequest {
                api_version: API_VERSION,
                session_id: i.session_id,
                request_id: RequestId::new(self.request).unwrap(),
                expected: i.state,
                commands,
            })
            .unwrap()
    }
    fn set(&mut self, value: &str) -> NodeId {
        let record =
            ConfigRecord::new(value.as_bytes().to_vec(), ConfigInterpretation::Opaque).unwrap();
        let command = if self
            .session
            .snapshot()
            .project
            .configuration()
            .records()
            .is_empty()
        {
            Command::InsertConfigRecord { index: 0, record }
        } else {
            Command::SetConfigRecord { index: 0, record }
        };
        self.command(vec![command]).state.head
    }
    fn nav(&mut self, action: HistoryAction) {
        self.request += 1;
        let i = self.session.project_info();
        self.session
            .navigate(HistoryRequest {
                api_version: API_VERSION,
                session_id: i.session_id,
                request_id: RequestId::new(self.request).unwrap(),
                expected: i.state,
                action,
            })
            .unwrap();
    }
    fn capture(&mut self) -> HistoryCapture {
        let i = self.session.project_info();
        self.session
            .capture_history(
                CaptureRequest {
                    version: HISTORY_CAPTURE_VERSION,
                    session_id: i.session_id,
                    expected: i.state,
                    limits: CaptureLimits::default(),
                    cancellation: CaptureCancellation::default(),
                },
                &mut self.ids,
            )
            .unwrap()
    }
}
fn request(id: u128, previous: Option<u128>, origin: SaveOrigin) -> ArchiveWriteRequest {
    ArchiveWriteRequest {
        version: ARCHIVE_PORT_VERSION,
        commit: CommitId::new(id).unwrap(),
        expected: previous.map(|n| CommitId::new(n).unwrap()),
        origin,
        cancellation: CaptureCancellation::default(),
    }
}
fn open(
    path: &Path,
    store: BlobStore,
    mode: RestoreSelection,
) -> (SqliteProjectFile, RestoreCandidate) {
    SqliteProjectFile::load(
        path,
        store,
        FileLimits::default(),
        mode,
        CaptureCancellation::default(),
    )
    .unwrap()
}

#[test]
fn real_sqlite_full_history_roundtrip_and_raw_originals_survive_source_drop() {
    let dir = Directory::new();
    let source = BlobStore::with_limits(1024 * 1024, 1024 * 1024).unwrap();
    let bytes = b"unknown-audio-original\0\xffretain exactly".to_vec();
    let raw = source.stage_bytes(bytes.clone()).unwrap();
    let reference = raw.reference();
    let mut d = Driver::new(source.clone());
    d.command(vec![Command::RegisterAsset {
        asset: Asset::new(
            AssetId::new(1).unwrap(),
            "unknown audio".into(),
            reference,
            AssetKind::Opaque,
        )
        .unwrap(),
    }]);
    let a = d.set("A");
    let b = d.set("B");
    d.nav(HistoryAction::Checkout { node: a });
    d.set("C");
    d.nav(HistoryAction::Undo);
    let expected = d.session.snapshot();
    let mut capture = d.capture();
    let original = capture.snapshot().unwrap().manifest().identity.clone();
    let (file, result) = SqliteProjectFile::publish_new(
        dir.file(),
        source.clone(),
        FileLimits::default(),
        &mut capture,
        request(400, None, SaveOrigin::Manual),
    )
    .unwrap();
    assert!(matches!(result, ArchiveWriteOutcome::Committed(_)));
    assert_eq!(file.receipt().unwrap().generation, 1);
    drop(file);
    drop(raw);
    drop(d);
    assert_eq!(source.usage().unwrap().bytes, 0);
    let loaded = BlobStore::with_limits(1024 * 1024, 1024 * 1024).unwrap();
    let (file, candidate) = open(&dir.file(), loaded.clone(), RestoreSelection::Manual);
    assert_eq!(candidate.header().identity, original);
    assert_eq!(candidate.header().node_count, 5);
    let mut restored = candidate
        .into_session(
            HistorySessionId::new(71).unwrap(),
            Box::new(loaded.clone()),
            SessionLimits::default(),
            CaptureCancellation::default(),
        )
        .unwrap();
    assert_eq!(restored.snapshot().project, expected.project);
    assert_eq!(restored.history_len(), 5);
    assert_eq!(loaded.get(reference).unwrap().bytes(), bytes);
    let i = restored.project_info();
    restored
        .navigate(HistoryRequest {
            api_version: API_VERSION,
            session_id: i.session_id,
            request_id: RequestId::new(1).unwrap(),
            expected: i.state,
            action: HistoryAction::Redo { child: None },
        })
        .unwrap();
    assert_eq!(restored.snapshot().project.configuration().to_bytes(), b"C");
    let i = restored.project_info();
    restored
        .navigate(HistoryRequest {
            api_version: API_VERSION,
            session_id: i.session_id,
            request_id: RequestId::new(2).unwrap(),
            expected: i.state,
            action: HistoryAction::Checkout { node: b },
        })
        .unwrap();
    assert_eq!(restored.snapshot().project.configuration().to_bytes(), b"B");
    drop(restored);
    drop(file);
    assert_eq!(loaded.usage().unwrap().bytes, 0);
}

#[test]
fn append_is_incremental_preserves_old_nodes_and_default_open_keeps_newer_recovery() {
    let dir = Directory::new();
    let store = BlobStore::default();
    let mut d = Driver::new(store.clone());
    let a = d.set("A");
    let b = d.set("B");
    d.nav(HistoryAction::Undo);
    let mut capture = d.capture();
    let (mut file, _) = SqliteProjectFile::publish_new(
        dir.file(),
        store.clone(),
        FileLimits::default(),
        &mut capture,
        request(400, None, SaveOrigin::Manual),
    )
    .unwrap();
    let c = d.set("C");
    d.nav(HistoryAction::Undo);
    let mut capture = d.capture();
    assert!(matches!(
        file.write_capture(&mut capture, request(401, Some(400), SaveOrigin::Automatic))
            .unwrap(),
        ArchiveWriteOutcome::Committed(_)
    ));
    drop(file);
    let (mut file, manual) = open(&dir.file(), store.clone(), RestoreSelection::Manual);
    assert_eq!(manual.nodes().len(), 4);
    let manual_identity = manual.header().identity.clone();
    let session = manual
        .into_session(
            HistorySessionId::new(71).unwrap(),
            Box::new(store.clone()),
            SessionLimits::default(),
            CaptureCancellation::default(),
        )
        .unwrap();
    let mut loaded = Driver {
        session,
        request: 0,
        ids: Ids(9000),
    };
    assert_eq!(loaded.session.project_info().state.head, a);
    let mut capture = loaded.capture();
    assert!(matches!(
        file.write_capture(&mut capture, request(402, Some(401), SaveOrigin::Exit))
            .unwrap(),
        ArchiveWriteOutcome::NoChange(_)
    ));
    assert_eq!(file.receipt().unwrap().commit, CommitId::new(401).unwrap());
    loaded.nav(HistoryAction::Redo { child: None });
    assert_eq!(loaded.session.project_info().state.head, b);
    let latest = file
        .restore_candidate(RestoreSelection::Latest, CaptureCancellation::default())
        .unwrap();
    assert_eq!(latest.header().identity, manual_identity);
    let session = latest
        .into_session(
            HistorySessionId::new(72).unwrap(),
            Box::new(store.clone()),
            SessionLimits::default(),
            CaptureCancellation::default(),
        )
        .unwrap();
    let mut loaded = Driver {
        session,
        request: 0,
        ids: Ids(10000),
    };
    loaded.nav(HistoryAction::Redo { child: None });
    assert_eq!(loaded.session.project_info().state.head, c);
    let mut capture = loaded.capture();
    assert!(matches!(
        file.write_capture(&mut capture, request(403, Some(401), SaveOrigin::Manual))
            .unwrap(),
        ArchiveWriteOutcome::Committed(_)
    ));
    assert_eq!(file.receipt().unwrap().generation, 3);
}

#[test]
fn existing_file_conflict_retry_and_resource_reuse_are_explicit() {
    let dir = Directory::new();
    let store = BlobStore::default();
    let mut d = Driver::new(store.clone());
    d.set("A");
    let (mut f, _) = SqliteProjectFile::publish_new(
        dir.file(),
        store.clone(),
        FileLimits::default(),
        &mut d.capture(),
        request(400, None, SaveOrigin::Manual),
    )
    .unwrap();
    let replay = f
        .write_capture(&mut d.capture(), request(400, None, SaveOrigin::Manual))
        .unwrap();
    assert!(matches!(replay, ArchiveWriteOutcome::Replayed(_)));
    d.set("B");
    let error = f
        .write_capture(
            &mut d.capture(),
            request(401, Some(999), SaveOrigin::Automatic),
        )
        .unwrap_err();
    assert_eq!(error, StorageError::Conflict);
    assert_eq!(f.receipt().unwrap_err(), StorageError::Invalidated);
    drop(f);
    let (mut f, c) = open(&dir.file(), store.clone(), RestoreSelection::Manual);
    drop(c);
    f.write_capture(
        &mut d.capture(),
        request(401, Some(400), SaveOrigin::Automatic),
    )
    .unwrap();
    drop(f);
    for _ in 0..20 {
        let (f, c) = open(&dir.file(), store.clone(), RestoreSelection::Manual);
        drop(c);
        drop(f);
        assert_eq!(store.usage().unwrap().bytes, 0);
    }
}
#[test]
fn limits_and_pending_journal_do_not_replace_active_project_or_mutate_input() {
    let dir = Directory::new();
    let store = BlobStore::default();
    let mut d = Driver::new(store.clone());
    d.set("A");
    let (f, _) = SqliteProjectFile::publish_new(
        dir.file(),
        store,
        FileLimits::default(),
        &mut d.capture(),
        request(400, None, SaveOrigin::Manual),
    )
    .unwrap();
    drop(f);
    let before = std::fs::read(dir.file()).unwrap();
    let active = d.session.snapshot();
    let limits = FileLimits {
        file_bytes: 1,
        ..FileLimits::default()
    };
    let error = SqliteProjectFile::load(
        dir.file(),
        BlobStore::default(),
        limits,
        RestoreSelection::Manual,
        CaptureCancellation::default(),
    )
    .unwrap_err();
    assert_eq!(error, StorageError::Limit("file bytes"));
    let journal = dir.path.join("test.oskproj-journal");
    std::fs::write(&journal, b"unresolved journal evidence").unwrap();
    assert_eq!(
        SqliteProjectFile::load(
            dir.file(),
            BlobStore::default(),
            FileLimits::default(),
            RestoreSelection::Manual,
            CaptureCancellation::default()
        )
        .unwrap_err(),
        StorageError::JournalPresent
    );
    assert_eq!(
        std::fs::read(&journal).unwrap(),
        b"unresolved journal evidence"
    );
    assert_eq!(std::fs::read(dir.file()).unwrap(), before);
    assert_eq!(d.session.snapshot().project, active.project);
}
#[test]
fn independent_restore_activation_rejects_old_runtime_requests() {
    let dir = Directory::new();
    let store = BlobStore::default();
    let mut d = Driver::new(store.clone());
    d.set("A");
    let old = d.session.project_info();
    let (f, _) = SqliteProjectFile::publish_new(
        dir.file(),
        store.clone(),
        FileLimits::default(),
        &mut d.capture(),
        request(400, None, SaveOrigin::Manual),
    )
    .unwrap();
    drop(f);
    let (f, c) = open(&dir.file(), store.clone(), RestoreSelection::Manual);
    let mut restored = c
        .into_session(
            HistorySessionId::new(999).unwrap(),
            Box::new(store),
            SessionLimits::default(),
            CaptureCancellation::default(),
        )
        .unwrap();
    assert!(
        restored
            .navigate(HistoryRequest {
                api_version: API_VERSION,
                session_id: old.session_id,
                request_id: RequestId::new(1).unwrap(),
                expected: old.state,
                action: HistoryAction::Undo
            })
            .is_err()
    );
    assert_eq!(restored.snapshot().project.configuration().to_bytes(), b"A");
    assert_eq!(f.receipt().unwrap().head, capture_head(&mut d));
}
fn capture_head(d: &mut Driver) -> PersistentNodeId {
    d.capture().snapshot().unwrap().manifest().head
}
#[test]
fn ten_thousand_deep_history_file_retains_identity_and_iterative_navigation() {
    let dir = Directory::new();
    let store = BlobStore::default();
    let mut d = Driver::new(store.clone());
    for n in 0..10000 {
        d.set(&n.to_string());
    }
    let mut capture = d.capture();
    let identity = capture.snapshot().unwrap().manifest().identity.clone();
    let start = std::time::Instant::now();
    let (f, _) = SqliteProjectFile::publish_new(
        dir.file(),
        store.clone(),
        FileLimits::default(),
        &mut capture,
        request(400, None, SaveOrigin::Manual),
    )
    .unwrap();
    let written = start.elapsed();
    drop(f);
    drop(d);
    let start = std::time::Instant::now();
    let (f, c) = open(&dir.file(), store.clone(), RestoreSelection::Manual);
    let read = start.elapsed();
    assert_eq!(c.header().node_count, 10001);
    assert_eq!(c.header().identity, identity);
    let mut restored = c
        .into_session(
            HistorySessionId::new(999).unwrap(),
            Box::new(store),
            SessionLimits::default(),
            CaptureCancellation::default(),
        )
        .unwrap();
    assert_eq!(restored.history_len(), 10001);
    let i = restored.project_info();
    restored
        .navigate(HistoryRequest {
            api_version: API_VERSION,
            session_id: i.session_id,
            request_id: RequestId::new(1).unwrap(),
            expected: i.state,
            action: HistoryAction::Checkout {
                node: NodeId::new(1).unwrap(),
            },
        })
        .unwrap();
    assert!(
        restored
            .snapshot()
            .project
            .configuration()
            .records()
            .is_empty()
    );
    eprintln!(
        "storage-depth10000 write_and_verify_ms={} full_read_ms={} file_bytes={}",
        written.as_millis(),
        read.as_millis(),
        std::fs::metadata(dir.file()).unwrap().len()
    );
    drop(f);
}
