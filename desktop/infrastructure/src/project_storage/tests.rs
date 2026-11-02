use super::*;
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

use rusqlite::OptionalExtension;

fn fixture() -> (Directory, Driver, SqliteProjectFile) {
    let dir = Directory::new();
    let store = BlobStore::default();
    let mut d = Driver::new(store.clone());
    let raw = store
        .stage_bytes(b"opaque-original-keep-even-on-old-branch".to_vec())
        .unwrap();
    d.command(vec![Command::RegisterAsset {
        asset: Asset::new(
            AssetId::new(1).unwrap(),
            "opaque".into(),
            raw.reference(),
            AssetKind::Opaque,
        )
        .unwrap(),
    }]);
    d.set("saved");
    d.nav(HistoryAction::Undo);
    d.nav(HistoryAction::Redo { child: None });
    let (file, _) = SqliteProjectFile::publish_new(
        dir.file(),
        store,
        FileLimits::default(),
        &mut d.capture(),
        request(400, None, SaveOrigin::Manual),
    )
    .unwrap();
    (dir, d, file)
}
fn digest(p: &Path) -> [u8; 32] {
    sha(&std::fs::read(p).unwrap())
}
#[test]
fn all_precommit_faults_and_cancellation_leave_old_file_exactly_readable() {
    for stage in [
        Stage::Originals,
        Stage::States,
        Stage::Nodes,
        Stage::BeforeCommit,
    ] {
        for cancel in [false, true] {
            let (dir, mut d, mut f) = fixture();
            let before = digest(&dir.file());
            d.set("new");
            let probe = if cancel {
                Probe {
                    cancel_at: Some(stage),
                    fault: None,
                }
            } else {
                Probe {
                    fault: Some(stage),
                    cancel_at: None,
                }
            };
            let error = write::existing(
                &mut f,
                &mut d.capture(),
                request(401, Some(400), SaveOrigin::Automatic),
                probe,
            )
            .unwrap_err();
            assert_eq!(
                error,
                if cancel {
                    StorageError::Cancelled
                } else {
                    StorageError::Io
                },
                "{stage:?}"
            );
            drop(f);
            assert_eq!(before, digest(&dir.file()));
            let fresh = BlobStore::default();
            let (f, c) = open(&dir.file(), fresh.clone(), RestoreSelection::Latest);
            assert_eq!(f.receipt().unwrap().commit, CommitId::new(400).unwrap());
            drop(c);
            drop(f);
            assert_eq!(fresh.usage().unwrap().bytes, 0);
        }
    }
}
#[test]
fn committed_but_unacknowledged_write_is_resolved_by_id_after_reopen() {
    let (dir, mut d, mut f) = fixture();
    d.set("new");
    let error = write::existing(
        &mut f,
        &mut d.capture(),
        request(401, Some(400), SaveOrigin::Exit),
        Probe {
            fault: Some(Stage::AfterCommit),
            cancel_at: None,
        },
    )
    .unwrap_err();
    assert_eq!(
        error,
        StorageError::OutcomeUnknown(CommitId::new(401).unwrap())
    );
    drop(f);
    let (f, c) = open(&dir.file(), BlobStore::default(), RestoreSelection::Latest);
    assert_eq!(
        f.committed(CommitId::new(401).unwrap())
            .unwrap()
            .unwrap()
            .generation,
        2
    );
    assert!(f.committed(CommitId::new(999).unwrap()).unwrap().is_none());
    assert_eq!(
        c.nodes().last().unwrap().project.configuration().to_bytes(),
        b"new"
    );
}
#[test]
fn first_publication_is_non_overwriting_and_temp_failure_has_no_half_file() {
    for stage in [
        Stage::BeforeCommit,
        Stage::BeforePublish,
        Stage::AfterPublish,
    ] {
        let dir = Directory::new();
        let store = BlobStore::default();
        let mut d = Driver::new(store.clone());
        d.set("A");
        let error = write::publish(
            &dir.file(),
            store.clone(),
            FileLimits::default(),
            &mut d.capture(),
            &request(400, None, SaveOrigin::Manual),
            Probe {
                fault: Some(stage),
                cancel_at: None,
            },
        )
        .unwrap_err();
        if stage == Stage::AfterPublish {
            assert_eq!(
                error,
                StorageError::OutcomeUnknown(CommitId::new(400).unwrap())
            );
            let (f, c) = open(&dir.file(), store.clone(), RestoreSelection::Manual);
            drop(c);
            drop(f);
        } else {
            assert!(!dir.file().exists());
        }
        assert!(
            !std::fs::read_dir(&dir.path).unwrap().any(|e| e
                .unwrap()
                .file_name()
                .to_string_lossy()
                .ends_with(".tmp"))
        );
    }
    let (dir, mut d, f) = fixture();
    let before = digest(&dir.file());
    drop(f);
    let error = SqliteProjectFile::publish_new(
        dir.file(),
        BlobStore::default(),
        FileLimits::default(),
        &mut d.capture(),
        request(900, None, SaveOrigin::Manual),
    )
    .unwrap_err();
    assert_eq!(error, StorageError::TargetExists);
    assert_eq!(before, digest(&dir.file()));
}
#[test]
fn cancel_during_original_and_state_validation_releases_leases_without_touching_file() {
    let (dir, _d, f) = fixture();
    drop(f);
    let before = digest(&dir.file());
    for stage in [Stage::ReadOriginal, Stage::ReadNodes] {
        let store = BlobStore::default();
        let mut conn =
            Connection::open_with_flags(dir.file(), rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
                .unwrap();
        let error = read::full_probe(
            &mut conn,
            &store,
            FileLimits::default(),
            RestoreSelection::Manual,
            &CaptureCancellation::default(),
            Probe {
                cancel_at: Some(stage),
                fault: None,
            },
        )
        .err()
        .unwrap();
        assert_eq!(error, StorageError::Cancelled);
        assert!(conn.is_autocommit());
        drop(conn);
        assert_eq!(store.usage().unwrap().bytes, 0);
        assert_eq!(before, digest(&dir.file()));
    }
}
fn corrupt(path: &Path, table: &str, update: &str) {
    let conn = Connection::open(path).unwrap();
    let trigger = format!("guard_{table}_UPDATE");
    let sql: Option<String> = conn
        .query_row(
            "SELECT sql FROM sqlite_schema WHERE type='trigger' AND name=?1",
            [&trigger],
            |r| r.get(0),
        )
        .optional()
        .unwrap();
    if sql.is_some() {
        conn.execute_batch(&format!("DROP TRIGGER {trigger}"))
            .unwrap();
    }
    conn.execute_batch(update).unwrap();
    if let Some(sql) = sql {
        conn.execute_batch(&sql).unwrap();
    }
}
#[test]
fn full_file_rejects_tampered_records_indexes_originals_and_unknown_profiles() {
    let (dir, _d, f) = fixture();
    drop(f);
    for (table, sql) in [
        (
            "nodes",
            "UPDATE nodes SET action=x'7b7d' WHERE parent IS NOT NULL",
        ),
        (
            "blob_chunks",
            "UPDATE blob_chunks SET bytes=zeroblob(length(bytes))",
        ),
        (
            "blocks",
            "UPDATE blocks SET bytes=x'7b7d' WHERE sequence=(SELECT max(sequence) FROM blocks)",
        ),
        (
            "addresses",
            "UPDATE addresses SET alias=x'0000000000000063' WHERE alias=x'0000000000000001'",
        ),
        (
            "object_ids",
            "UPDATE object_ids SET first_node=(SELECT id FROM nodes WHERE depth=2)",
        ),
        ("control", "UPDATE control SET node_count=node_count+1"),
        ("header", "UPDATE header SET profile='future-profile'"),
        (
            "features",
            "INSERT INTO features VALUES('future.required','1',1)",
        ),
    ] {
        let target = dir.path.join("bad.oskproj");
        std::fs::copy(dir.file(), &target).unwrap();
        corrupt(&target, table, sql);
        let before = digest(&target);
        let store = BlobStore::default();
        let result = SqliteProjectFile::load(
            &target,
            store.clone(),
            FileLimits::default(),
            RestoreSelection::Manual,
            CaptureCancellation::default(),
        );
        assert!(result.is_err(), "{table}");
        drop(result);
        assert_eq!(store.usage().unwrap().bytes, 0);
        assert_eq!(before, digest(&target));
        std::fs::remove_file(target).unwrap();
    }
}
#[test]
fn sqlite_guards_deny_update_delete_and_replace_of_history() {
    let (_dir, _d, f) = fixture();
    for sql in [
        "UPDATE nodes SET depth=depth",
        "DELETE FROM nodes",
        "INSERT OR REPLACE INTO nodes SELECT * FROM nodes LIMIT 1",
        "DELETE FROM blobs",
        "UPDATE commits SET origin=origin",
    ] {
        assert!(f.conn.execute_batch(sql).is_err(), "{sql}");
    }
    assert!(schema::verify(&f.conn).is_ok());
}
#[test]
fn node_and_address_hashes_match_independent_python_golden() {
    let id = PersistentNodeId::new(u128::from_be_bytes([3; 16])).unwrap();
    let info = CapturedNodeInfo {
        id,
        parent: None,
        alias: 1,
        coordinate: BranchCoordinate::new(1, 0).unwrap(),
        depth: 0,
        operation: Arc::new(CapturedOperation {
            version: 1,
            origin: OperationOrigin::Root,
            steps: vec![],
        }),
        changed_objects: vec![],
        origin: None,
    };
    assert_eq!(
        hex(&node_checksum(
            id,
            None,
            StateHash::from_bytes([4; 32]),
            b"{ \"kind\": \"root\", \"x\": 1e-07 }"
        )),
        "43b740347b492d11c5c2f10bbe3533c35050496e223d612a72a7f9ec931adfb4"
    );
    assert_eq!(
        hex(&address_checksum(&info)),
        "3bb830e2bd53ef20560c007cf49e0793c89ac7ef402479d5b39a8ee6d69e2a7a"
    );
}

#[test]
fn png_wav_hidden_nested_layers_and_independent_sound_survive_file_roundtrip() {
    use image::ImageEncoder;
    let (dir, mut d, mut f) = fixture();
    let store = f.originals.clone();
    let mut png = Vec::new();
    image::codecs::png::PngEncoder::new(&mut png)
        .write_image(
            &[255, 0, 0, 255, 0, 0, 0, 0],
            2,
            1,
            image::ExtendedColorType::Rgba8,
        )
        .unwrap();
    let image = store.stage_bytes(png.clone()).unwrap();
    let mut wav = b"RIFF".to_vec();
    wav.extend(40u32.to_le_bytes());
    wav.extend(b"WAVEfmt ");
    wav.extend(16u32.to_le_bytes());
    wav.extend(1u16.to_le_bytes());
    wav.extend(1u16.to_le_bytes());
    wav.extend(48000u32.to_le_bytes());
    wav.extend(96000u32.to_le_bytes());
    wav.extend(2u16.to_le_bytes());
    wav.extend(16u16.to_le_bytes());
    wav.extend(b"data");
    wav.extend(4u32.to_le_bytes());
    wav.extend([0, 0, 255, 127]);
    let audio = store.stage_bytes(wav.clone()).unwrap();
    audio
        .attest_audio(AudioMetadata::new(48000, 1, 2).unwrap())
        .unwrap();
    let a = AssetId::new(2).unwrap();
    let b = AssetId::new(3).unwrap();
    let doc = DocumentId::new(8).unwrap();
    let l = LayerId::new(8).unwrap();
    let child = LayerId::new(9).unwrap();
    d.command(vec![
        Command::RegisterAsset {
            asset: Asset::new(
                a,
                "image.png".into(),
                image.reference(),
                AssetKind::RasterImage(PixelSize::new(2, 1).unwrap()),
            )
            .unwrap(),
        },
        Command::RegisterAsset {
            asset: Asset::new(
                b,
                "sound.wav".into(),
                audio.reference(),
                AssetKind::Audio(AudioMetadata::new(48000, 1, 2).unwrap()),
            )
            .unwrap(),
        },
        Command::CreateDocument {
            document: ElementDocument::new(
                doc,
                PixelSize::new(2, 1).unwrap(),
                LayerTree::new(
                    vec![l],
                    vec![
                        LayerNode {
                            id: l,
                            local: Affine2::IDENTITY,
                            opacity: Opacity::OPAQUE,
                            visible: false,
                            kind: LayerKind::Group {
                                children: vec![child],
                            },
                        },
                        LayerNode {
                            id: child,
                            local: Affine2::IDENTITY,
                            opacity: Opacity::OPAQUE,
                            visible: true,
                            kind: LayerKind::Image { asset: a },
                        },
                    ],
                )
                .unwrap(),
            ),
        },
        Command::CreateAudioComponent {
            component: AudioComponent::new(
                AudioComponentId::new(5).unwrap(),
                SoundSlotKey::new("fixture.sound".into()).unwrap(),
                b,
            ),
        },
    ]);
    let expected = d.session.snapshot().project;
    f.write_capture(
        &mut d.capture(),
        request(401, Some(400), SaveOrigin::Manual),
    )
    .unwrap();
    drop(f);
    drop(d);
    drop(image);
    drop(audio);
    assert_eq!(store.usage().unwrap().bytes, 0);
    let (f, c) = open(&dir.file(), store.clone(), RestoreSelection::Manual);
    assert_eq!(c.nodes().last().unwrap().project, expected);
    for a in expected.assets().iter() {
        let raw = store.get(a.blob()).unwrap();
        match a.kind() {
            AssetKind::RasterImage(_) => assert_eq!(raw.bytes(), png),
            AssetKind::Audio(_) => assert_eq!(raw.bytes(), wav),
            AssetKind::Opaque => (),
        }
    }
    drop(c);
    drop(f);
    assert_eq!(store.usage().unwrap().bytes, 0);
}
#[test]
fn busy_external_modification_and_raw_pool_limit_fail_closed() {
    let (dir, mut d, mut f) = fixture();
    let other = Connection::open(dir.file()).unwrap();
    other.execute_batch("BEGIN IMMEDIATE").unwrap();
    d.set("new");
    assert_eq!(
        f.write_capture(
            &mut d.capture(),
            request(401, Some(400), SaveOrigin::Automatic)
        )
        .unwrap_err(),
        StorageError::Busy
    );
    other.execute_batch("ROLLBACK").unwrap();
    drop(f);
    let (f, c) = open(&dir.file(), BlobStore::default(), RestoreSelection::Manual);
    drop(c);
    other
        .execute("UPDATE control SET node_count=node_count+1", [])
        .unwrap();
    assert_eq!(f.receipt().unwrap_err(), StorageError::ExternalChange);
    drop(f);
    // Limits are checked before publishing any candidate and return all raw leases.
    let (dir, _d, f) = fixture();
    drop(f);
    let limited = BlobStore::with_limits(1, 1).unwrap();
    assert!(
        SqliteProjectFile::load(
            dir.file(),
            limited.clone(),
            FileLimits::default(),
            RestoreSelection::Manual,
            CaptureCancellation::default()
        )
        .is_err()
    );
    assert_eq!(limited.usage().unwrap().bytes, 0);
}

#[test]
fn incremental_commit_reuses_old_records_and_only_indexes_new_nodes() {
    let (_dir, mut d, mut f) = fixture();
    let rows = |c: &Connection| {
        c.prepare("SELECT hex(id)||':'||hex(state)||':'||hex(action)||':'||hex(checksum) FROM nodes ORDER BY depth,id").unwrap().query_map([],|r|r.get::<_,String>(0)).unwrap().collect::<Result<BTreeSet<_>,_>>().unwrap()
    };
    let before = rows(&f.conn);
    let original_count = f.cache.originals.len();
    d.set("later");
    f.write_capture(
        &mut d.capture(),
        request(401, Some(400), SaveOrigin::Automatic),
    )
    .unwrap();
    let after = rows(&f.conn);
    assert!(before.is_subset(&after));
    assert_eq!(after.len(), before.len() + 1);
    assert_eq!(f.cache.originals.len(), original_count);
    let checkpoint = f.cache.checkpoints.last().unwrap();
    let row = &f.cache.blocks[&checkpoint.navigation];
    let mut v = blocks::value(row, f.limits.codec).unwrap();
    let root = v.hash("nodes").unwrap();
    let delta = blocks::read_paged(&f.cache, root, "nodes", f.limits).unwrap();
    assert_eq!(delta.len(), 1);
    eprintln!("archive SQLite {}", SqliteProjectFile::sqlite_version());
}

#[test]
fn non_selected_branch_is_fully_verified_and_restore_drop_cycles_release_originals() {
    let (dir, mut d, mut f) = fixture();
    let b = d.set("other branch");
    d.nav(HistoryAction::Undo);
    d.set("selected branch");
    f.write_capture(
        &mut d.capture(),
        request(401, Some(400), SaveOrigin::Automatic),
    )
    .unwrap();
    let other = f
        .cache
        .nodes
        .values()
        .find(|n| n.info.alias == b.get())
        .unwrap()
        .info
        .id;
    drop(f);
    drop(d);
    let store = BlobStore::default();
    for _ in 0..20 {
        let (f, c) = open(&dir.file(), store.clone(), RestoreSelection::Manual);
        assert!(store.usage().unwrap().bytes > 0);
        drop(c);
        drop(f);
        assert_eq!(store.usage().unwrap().bytes, 0);
    }
    corrupt(
        &dir.file(),
        "nodes",
        &format!(
            "UPDATE nodes SET action=x'7b7d' WHERE id=x'{}'",
            ptext(other)
        ),
    );
    let before = digest(&dir.file());
    assert!(
        SqliteProjectFile::load(
            dir.file(),
            store.clone(),
            FileLimits::default(),
            RestoreSelection::Latest,
            CaptureCancellation::default()
        )
        .is_err()
    );
    assert_eq!(store.usage().unwrap().bytes, 0);
    assert_eq!(before, digest(&dir.file()));
}

#[test]
fn unsupported_metadata_edits_are_rejected_instead_of_silently_discarded() {
    for edit_extension in [false, true] {
        let (dir, _source, mut file) = fixture();
        let before = digest(&dir.file());
        let candidate = file
            .restore_candidate(RestoreSelection::Manual, CaptureCancellation::default())
            .unwrap();
        let mut header = candidate.header().clone();
        let mut meta = (*header.metadata).clone();
        if edit_extension {
            meta.extensions.push(ArchiveExtension {
                name: "future-extra".into(),
                bytes: b"must not disappear".to_vec(),
            });
        } else {
            meta.tags.push(ArchiveTag {
                node: header.root,
                name: "user:note".into(),
                value: "must not disappear".into(),
            });
        }
        header.metadata = Arc::new(meta);
        let mut b = RestoreBuilder::new(
            header,
            RestoreLimits::default(),
            CaptureCancellation::default(),
        )
        .unwrap();
        for n in candidate.nodes() {
            b.push(n.clone()).unwrap();
        }
        let c = b.finish(&file.originals).unwrap();
        drop(candidate);
        let session = c
            .into_session(
                HistorySessionId::new(333).unwrap(),
                Box::new(file.originals.clone()),
                SessionLimits::default(),
                CaptureCancellation::default(),
            )
            .unwrap();
        let mut modified = Driver {
            session,
            request: 0,
            ids: Ids(5000),
        };
        modified.set("unsaved edit");
        assert_eq!(
            file.write_capture(
                &mut modified.capture(),
                request(401, Some(400), SaveOrigin::Manual)
            )
            .unwrap_err(),
            StorageError::Unsupported("metadata editing")
        );
        assert_eq!(
            modified
                .session
                .snapshot()
                .project
                .configuration()
                .to_bytes(),
            b"unsaved edit"
        );
        drop(file);
        assert_eq!(before, digest(&dir.file()));
    }
}
