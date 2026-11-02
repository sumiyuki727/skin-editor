use super::*;
use std::io::{BufRead, Write};
use std::process::{Child, Command as ProcessCommand, Stdio};
use std::sync::mpsc;
use std::time::Duration;
const ENTRY: &str = "project_storage::tests::recovery_process::child_entry";
const MAGIC: [u8; 8] = [0xd9, 0xd5, 0x05, 0xf9, 0x20, 0xa1, 0x63, 0xd7];

struct GatedChild {
    child: Child,
    drain: Option<std::thread::JoinHandle<()>>,
}
impl GatedChild {
    fn start(path: &Path, mode: &str) -> Self {
        use std::os::windows::process::CommandExt;
        let mut nonce = [0; 16];
        getrandom::fill(&mut nonce).unwrap();
        let token = hex(&nonce);
        let mut child = ProcessCommand::new(std::env::current_exe().unwrap())
            .args(["--exact", ENTRY, "--nocapture"])
            .env("SKIN_EDITOR_STORAGE_CHILD_MODE", mode)
            .env("SKIN_EDITOR_STORAGE_CHILD_PATH", path)
            .env("SKIN_EDITOR_STORAGE_CHILD_NONCE", &token)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .creation_flags(0x08000000)
            .spawn()
            .unwrap();
        let stdout = child.stdout.take().unwrap();
        let (sender, receiver) = mpsc::channel();
        let expected = format!("READY:{token}");
        let drain = std::thread::spawn(move || {
            let mut found = false;
            for line in std::io::BufReader::new(stdout).lines() {
                let Ok(line) = line else { break };
                if line == expected && !found {
                    found = true;
                    let _ = sender.send(true);
                }
            }
            if !found {
                let _ = sender.send(false);
            }
        });
        let mut owned = Self {
            child,
            drain: Some(drain),
        };
        if receiver.recv_timeout(Duration::from_secs(30)) != Ok(true) {
            owned.stop();
            panic!("owned storage child did not reach the explicit gate: {mode}");
        }
        owned
    }
    fn stop(&mut self) {
        if self.child.try_wait().unwrap().is_none() {
            self.child.kill().unwrap();
        }
        let _ = self.child.wait();
        if let Some(d) = self.drain.take() {
            d.join().unwrap();
        }
    }
    fn resume(&mut self) {
        self.child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(b"go\n")
            .unwrap();
        self.child.stdin.as_mut().unwrap().flush().unwrap();
        assert!(self.child.wait().unwrap().success());
        if let Some(d) = self.drain.take() {
            d.join().unwrap();
        }
    }
}
impl Drop for GatedChild {
    fn drop(&mut self) {
        self.stop();
    }
}

#[test]
fn child_entry() {
    let Ok(mode) = std::env::var("SKIN_EDITOR_STORAGE_CHILD_MODE") else {
        return;
    };
    let path = PathBuf::from(std::env::var_os("SKIN_EDITOR_STORAGE_CHILD_PATH").unwrap());
    let parent = path.parent().unwrap().canonicalize().unwrap();
    assert_eq!(
        parent.parent(),
        Some(std::env::temp_dir().canonicalize().unwrap().as_path())
    );
    assert!(
        parent
            .file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("skin-editor-archive-")
    );
    assert_eq!(path.file_name().unwrap(), "test.oskproj");
    let stage = match mode.as_str() {
        "before" => Stage::BeforeCommit,
        "after" => Stage::AfterCommit,
        "first-before" => Stage::BeforePublish,
        "first-after" => Stage::AfterPublish,
        "recover-copied" => Stage::CopyJournal,
        "recover-validated" => Stage::RecoveryValidated,
        "recover-published" => Stage::AfterPublish,
        _ => panic!("unknown owned child mode"),
    };
    let probe = Probe {
        pause_at: Some(stage),
        ..Probe::default()
    };
    let store = BlobStore::default();
    if mode.starts_with("recover-") {
        recovery::recover(
            &path,
            &parent.join("interrupted-copy.oskproj"),
            store,
            FileLimits::default(),
            RestoreSelection::Latest,
            &CaptureCancellation::default(),
            probe,
        )
        .unwrap();
    } else if mode.starts_with("first-") {
        let mut d = Driver::new(store.clone());
        d.set("first A");
        d.set("first B");
        d.nav(HistoryAction::Undo);
        d.set("first C");
        d.nav(HistoryAction::Undo);
        write::publish(
            &path,
            store,
            FileLimits::default(),
            &mut d.capture(),
            &request(400, None, SaveOrigin::Manual),
            probe,
        )
        .unwrap();
    } else {
        let (mut file, c) = open(&path, store.clone(), RestoreSelection::Latest);
        let session = c
            .into_session(
                HistorySessionId::new(900).unwrap(),
                Box::new(store.clone()),
                SessionLimits::default(),
                CaptureCancellation::default(),
            )
            .unwrap();
        let mut d = Driver {
            session,
            request: 0,
            ids: Ids(10000),
        };
        let raw = store.stage_bytes(pending_bytes()).unwrap();
        d.command(vec![
            Command::RegisterAsset {
                asset: Asset::new(
                    AssetId::new(3).unwrap(),
                    "pending opaque".into(),
                    raw.reference(),
                    AssetKind::Opaque,
                )
                .unwrap(),
            },
            Command::SetConfigRecord {
                index: 0,
                record: ConfigRecord::new(
                    b"after interruption".to_vec(),
                    ConfigInterpretation::Opaque,
                )
                .unwrap(),
            },
        ]);
        write::existing(
            &mut file,
            &mut d.capture(),
            request(402, Some(401), SaveOrigin::Automatic),
            probe,
        )
        .unwrap();
    }
}
fn pending_bytes() -> Vec<u8> {
    (0..131089).map(|n| (n % 251) as u8).collect()
}
type BranchBaseline = (
    Directory,
    Vec<u8>,
    PersistentHistoryIdentity,
    BTreeMap<PersistentNodeId, [u8; 32]>,
);
fn branched() -> BranchBaseline {
    let (dir, mut d, mut f) = fixture();
    let branch = f
        .originals
        .stage_bytes(b"branch-only original".to_vec())
        .unwrap();
    d.command(vec![
        Command::RegisterAsset {
            asset: Asset::new(
                AssetId::new(2).unwrap(),
                "branch-only".into(),
                branch.reference(),
                AssetKind::Opaque,
            )
            .unwrap(),
        },
        Command::SetConfigRecord {
            index: 0,
            record: ConfigRecord::new(b"branch B".to_vec(), ConfigInterpretation::Opaque).unwrap(),
        },
    ]);
    d.nav(HistoryAction::Undo);
    d.set("branch C");
    d.nav(HistoryAction::Undo);
    f.write_capture(
        &mut d.capture(),
        request(401, Some(400), SaveOrigin::Manual),
    )
    .unwrap();
    let identity = f.cache.header.identity.clone();
    let nodes = f
        .cache
        .nodes
        .iter()
        .map(|(id, n)| (*id, n.info.origin.as_ref().unwrap().checksum))
        .collect();
    drop(f);
    drop(d);
    let bytes = std::fs::read(dir.file()).unwrap();
    (dir, bytes, identity, nodes)
}
fn sidecar(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.as_os_str().to_os_string();
    name.push(suffix);
    PathBuf::from(name)
}
fn bundle(path: &Path) -> (Vec<u8>, Option<Vec<u8>>) {
    (
        std::fs::read(path).unwrap(),
        std::fs::read(sidecar(path, "-journal")).ok(),
    )
}
fn scratch_count(dir: &Directory) -> usize {
    std::fs::read_dir(&dir.path)
        .unwrap()
        .filter(|e| {
            e.as_ref()
                .unwrap()
                .file_name()
                .to_string_lossy()
                .contains(".recover-")
        })
        .count()
}
fn recover(
    source: &Path,
    target: &Path,
    store: BlobStore,
    probe: Probe,
) -> Result<RecoveredArchive, StorageError> {
    recovery::recover(
        source,
        target,
        store,
        FileLimits::default(),
        RestoreSelection::Latest,
        &CaptureCancellation::default(),
        probe,
    )
}

#[test]
fn actual_kill_before_commit_rolls_back_only_the_copy_and_keeps_all_old_branches() {
    let (dir, baseline, identity, nodes) = branched();
    let mut child = GatedChild::start(&dir.file(), "before");
    child.stop();
    let original = bundle(&dir.file());
    assert!(original.1.as_ref().unwrap().len() > 512);
    assert_ne!(original.0, baseline);
    assert_eq!(
        SqliteProjectFile::load(
            dir.file(),
            BlobStore::default(),
            FileLimits::default(),
            RestoreSelection::Latest,
            CaptureCancellation::default()
        )
        .unwrap_err(),
        StorageError::JournalPresent
    );
    assert_eq!(bundle(&dir.file()), original);
    let target = dir.path.join("recovered.oskproj");
    let store = BlobStore::default();
    let recovered = recover(&dir.file(), &target, store.clone(), Probe::default()).unwrap();
    assert_eq!(recovered.report.mode, RecoveryMode::JournalRollback);
    assert!(recovered.report.journal_records > 0);
    assert_eq!(
        recovered.report.restored.commit,
        CommitId::new(401).unwrap()
    );
    assert_eq!(recovered.candidate.header().identity, identity);
    assert_eq!(recovered.candidate.header().node_count, 5);
    let branch_blob = recovered
        .candidate
        .nodes()
        .find(|n| n.alias == 4)
        .unwrap()
        .project
        .assets()
        .get(AssetId::new(2).unwrap())
        .unwrap()
        .blob();
    assert_eq!(
        store.get(branch_blob).unwrap().bytes(),
        b"branch-only original"
    );
    for (id, checksum) in &nodes {
        assert_eq!(
            recovered.archive.cache.nodes[id]
                .info
                .origin
                .as_ref()
                .unwrap()
                .checksum,
            *checksum
        );
    }
    assert!(!same_file::is_same_file(dir.file(), &target).unwrap());
    assert_eq!(std::fs::read(&target).unwrap(), baseline);
    assert_eq!(bundle(&dir.file()), original);
    assert_eq!(scratch_count(&dir), 0);
    let session = recovered
        .candidate
        .into_session(
            HistorySessionId::new(901).unwrap(),
            Box::new(store.clone()),
            SessionLimits::default(),
            CaptureCancellation::default(),
        )
        .unwrap();
    let mut d = Driver {
        session,
        request: 0,
        ids: Ids(20000),
    };
    d.nav(HistoryAction::Redo { child: None });
    assert_eq!(
        d.session.snapshot().project.configuration().to_bytes(),
        b"branch C"
    );
    d.nav(HistoryAction::Checkout {
        node: NodeId::new(4).unwrap(),
    });
    assert_eq!(
        d.session.snapshot().project.configuration().to_bytes(),
        b"branch B"
    );
    let mut archive = recovered.archive;
    d.set("new recovered edit");
    archive
        .write_capture(
            &mut d.capture(),
            request(403, Some(401), SaveOrigin::Manual),
        )
        .unwrap();
    assert_eq!(bundle(&dir.file()), original);
    drop(d);
    drop(archive);
    assert_eq!(store.usage().unwrap().bytes, 0);
}
#[test]
fn actual_kill_after_commit_keeps_new_nodes_and_resolves_lost_acknowledgement() {
    let (dir, _baseline, identity, nodes) = branched();
    let mut child = GatedChild::start(&dir.file(), "after");
    child.stop();
    let original = bundle(&dir.file());
    assert!(original.1.is_none());
    let store = BlobStore::default();
    let target = dir.path.join("committed-copy.oskproj");
    let r = recover(&dir.file(), &target, store.clone(), Probe::default()).unwrap();
    assert_eq!(r.report.mode, RecoveryMode::VerifiedCopy);
    assert_eq!(
        r.archive
            .committed(CommitId::new(402).unwrap())
            .unwrap()
            .unwrap()
            .generation,
        3
    );
    assert_eq!(r.candidate.header().identity, identity);
    assert_eq!(r.candidate.header().node_count, 6);
    for (id, sum) in nodes {
        assert_eq!(
            r.archive.cache.nodes[&id]
                .info
                .origin
                .as_ref()
                .unwrap()
                .checksum,
            sum
        );
    }
    let pending = r
        .candidate
        .nodes()
        .last()
        .unwrap()
        .project
        .assets()
        .iter()
        .find(|a| a.id() == AssetId::new(3).unwrap())
        .unwrap()
        .blob();
    assert_eq!(store.get(pending).unwrap().bytes(), pending_bytes());
    let manual = SqliteProjectFile::recover_copy(
        dir.file(),
        dir.path.join("manual-copy.oskproj"),
        store.clone(),
        FileLimits::default(),
        RestoreSelection::Manual,
        CaptureCancellation::default(),
    )
    .unwrap();
    assert_eq!(manual.candidate.header().node_count, 6);
    assert_eq!(
        manual
            .candidate
            .nodes()
            .find(|n| n.id == manual.candidate.header().head)
            .unwrap()
            .alias,
        3
    );
    assert_ne!(manual.candidate.header().head, r.candidate.header().head);
    assert_eq!(manual.report.restored.commit, CommitId::new(402).unwrap());
    drop(manual);
    assert_eq!(bundle(&dir.file()), original);
    drop(r);
    assert_eq!(store.usage().unwrap().bytes, 0);
}
#[test]
fn live_writer_is_busy_and_an_existing_adapter_never_inspects_a_known_hot_journal() {
    let (dir, _baseline, _identity, _nodes) = branched();
    let (parent, c) = open(&dir.file(), BlobStore::default(), RestoreSelection::Latest);
    drop(c);
    let mut child = GatedChild::start(&dir.file(), "before");
    let before = bundle(&dir.file());
    let target = dir.path.join("busy.oskproj");
    assert_eq!(
        recover(&dir.file(), &target, BlobStore::default(), Probe::default()).unwrap_err(),
        StorageError::Busy
    );
    assert!(!target.exists());
    assert_eq!(parent.receipt().unwrap_err(), StorageError::JournalPresent);
    assert_eq!(bundle(&dir.file()), before);
    child.stop();
    assert_eq!(parent.receipt().unwrap_err(), StorageError::JournalPresent);
    drop(parent);
    assert!(recover(&dir.file(), &target, BlobStore::default(), Probe::default()).is_ok());
    assert_eq!(bundle(&dir.file()), before);
}
#[test]
fn a_live_writer_can_resume_after_recovery_refusal_without_any_implicit_rollback() {
    let (dir, _b, _i, _n) = branched();
    let mut child = GatedChild::start(&dir.file(), "before");
    let before = bundle(&dir.file());
    assert_eq!(
        recover(
            &dir.file(),
            &dir.path.join("busy.oskproj"),
            BlobStore::default(),
            Probe::default()
        )
        .unwrap_err(),
        StorageError::Busy
    );
    assert_eq!(bundle(&dir.file()), before);
    child.resume();
    let (f, c) = open(&dir.file(), BlobStore::default(), RestoreSelection::Latest);
    assert_eq!(f.receipt().unwrap().commit, CommitId::new(402).unwrap());
    assert_eq!(c.header().node_count, 6);
}
#[test]
fn actual_first_publish_interruptions_preserve_a_complete_recoverable_source() {
    for mode in ["first-before", "first-after"] {
        let dir = Directory::new();
        let mut child = GatedChild::start(&dir.file(), mode);
        child.stop();
        let source = if mode == "first-after" {
            assert!(dir.file().exists());
            dir.file()
        } else {
            assert!(!dir.file().exists());
            let files: Vec<_> = std::fs::read_dir(&dir.path)
                .unwrap()
                .map(|e| e.unwrap().path())
                .filter(|p| p.extension().is_some_and(|s| s == "tmp"))
                .collect();
            assert_eq!(files.len(), 1);
            files[0].clone()
        };
        let original = bundle(&source);
        let target = dir.path.join("first-recovered.oskproj");
        let r = recover(&source, &target, BlobStore::default(), Probe::default()).unwrap();
        assert_eq!(r.candidate.header().node_count, 4);
        assert_eq!(r.report.restored.commit, CommitId::new(400).unwrap());
        assert_eq!(bundle(&source), original);
        assert_eq!(scratch_count(&dir), 0);
    }
}

#[test]
fn killing_recovery_itself_preserves_the_source_and_does_not_promote_or_delete_orphans() {
    for mode in ["recover-copied", "recover-validated", "recover-published"] {
        let (dir, baseline, identity, _) = branched();
        let mut writer = GatedChild::start(&dir.file(), "before");
        writer.stop();
        let source = bundle(&dir.file());
        let mut worker = GatedChild::start(&dir.file(), mode);
        worker.stop();
        assert_eq!(bundle(&dir.file()), source);
        let target = dir.path.join("interrupted-copy.oskproj");
        assert_eq!(target.exists(), mode == "recover-published");
        if target.exists() {
            assert_eq!(std::fs::read(&target).unwrap(), baseline);
            assert!(!same_file::is_same_file(dir.file(), &target).unwrap());
        }
        let orphans: BTreeMap<_, _> = std::fs::read_dir(&dir.path)
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| {
                p.file_name()
                    .unwrap()
                    .to_string_lossy()
                    .contains(".recover-")
            })
            .map(|p| {
                let bytes = std::fs::read(&p).unwrap();
                (p, bytes)
            })
            .collect();
        assert_eq!(orphans.len(), if mode == "recover-copied" { 2 } else { 1 });
        let store = BlobStore::default();
        let fresh = dir.path.join("retry-copy.oskproj");
        let r = recover(&dir.file(), &fresh, store.clone(), Probe::default()).unwrap();
        assert_eq!(r.candidate.header().identity, identity);
        assert_eq!(r.candidate.header().node_count, 5);
        assert_eq!(std::fs::read(&fresh).unwrap(), baseline);
        drop(r);
        assert_eq!(store.usage().unwrap().bytes, 0);
        assert_eq!(bundle(&dir.file()), source);
        assert_eq!(scratch_count(&dir), orphans.len());
        for (path, bytes) in orphans {
            assert_eq!(std::fs::read(path).unwrap(), bytes);
        }
    }
}

#[test]
fn repeated_recovery_shares_original_allocations_and_releases_only_the_final_lease() {
    let (dir, baseline, _, _) = branched();
    let store = BlobStore::default();
    let anchor = recover(
        &dir.file(),
        &dir.path.join("anchor.oskproj"),
        store.clone(),
        Probe::default(),
    )
    .unwrap();
    let pinned = store.usage().unwrap().bytes;
    assert!(pinned > 0);
    for cycle in 0..20 {
        let target = dir.path.join(format!("cycle-{cycle}.oskproj"));
        let r = recover(&dir.file(), &target, store.clone(), Probe::default()).unwrap();
        assert_eq!(r.candidate.header().node_count, 5);
        assert_eq!(store.usage().unwrap().bytes, pinned);
        drop(r);
        assert_eq!(store.usage().unwrap().bytes, pinned);
        assert_eq!(scratch_count(&dir), 0);
        assert_eq!(std::fs::read(dir.file()).unwrap(), baseline);
    }
    drop(anchor);
    assert_eq!(store.usage().unwrap().bytes, 0);
}

#[test]
fn recovery_rejects_corrupt_unselected_branch_without_rewriting_or_pruning_it() {
    let (dir, _, _, _) = branched();
    let (f, c) = open(&dir.file(), BlobStore::default(), RestoreSelection::Manual);
    let branch = c.nodes().find(|n| n.alias == 4).unwrap().id;
    drop(c);
    drop(f);
    corrupt(
        &dir.file(),
        "nodes",
        &format!(
            "UPDATE nodes SET action=x'7b7d' WHERE id=x'{}'",
            ptext(branch)
        ),
    );
    let before = bundle(&dir.file());
    let store = BlobStore::default();
    let target = dir.path.join("bad-branch.oskproj");
    assert!(recover(&dir.file(), &target, store.clone(), Probe::default()).is_err());
    assert_eq!(bundle(&dir.file()), before);
    assert!(!target.exists());
    assert_eq!(scratch_count(&dir), 0);
    assert_eq!(store.usage().unwrap().bytes, 0);
}

#[test]
fn cancellation_and_cleanup_failure_after_publication_leave_a_valid_target_and_explicit_outcome() {
    let (dir, baseline) = closed_fixture();
    let store = BlobStore::default();
    for (name, probe, cleanup) in [
        (
            "cancel-published.oskproj",
            Probe {
                cancel_at: Some(Stage::AfterPublish),
                ..Probe::default()
            },
            false,
        ),
        (
            "cleanup-published.oskproj",
            Probe {
                fail_cleanup: true,
                ..Probe::default()
            },
            true,
        ),
    ] {
        let target = dir.path.join(name);
        let error = recover(&dir.file(), &target, store.clone(), probe).unwrap_err();
        if cleanup {
            assert!(matches!(error, StorageError::RecoveryCleanup { cause, .. }
                if matches!(*cause, StorageError::OutcomeUnknown(_))));
        } else {
            assert!(matches!(error, StorageError::OutcomeUnknown(_)));
        }
        assert_eq!(store.usage().unwrap().bytes, 0);
        assert_eq!(std::fs::read(&target).unwrap(), baseline);
        let (file, candidate) = open(&target, store.clone(), RestoreSelection::Manual);
        drop(candidate);
        drop(file);
        assert_eq!(store.usage().unwrap().bytes, 0);
        assert_eq!(std::fs::read(dir.file()).unwrap(), baseline);
    }
    assert_eq!(scratch_count(&dir), 1);
}

fn synthetic(main: &[u8]) -> Vec<u8> {
    let mut j = vec![0; 512];
    j[..8].copy_from_slice(&MAGIC);
    j[8..12].copy_from_slice(&1u32.to_be_bytes());
    j[12..16].copy_from_slice(&17u32.to_be_bytes());
    j[16..20].copy_from_slice(&((main.len() / 4096) as u32).to_be_bytes());
    j[20..24].copy_from_slice(&512u32.to_be_bytes());
    j[24..28].copy_from_slice(&4096u32.to_be_bytes());
    j.extend(1u32.to_be_bytes());
    j.extend(&main[..4096]);
    let mut sum = 17u32;
    for i in (0..4096).rev().filter(|i| (4096 - i) % 200 == 0) {
        sum = sum.wrapping_add(u32::from(main[i]));
    }
    j.extend(sum.to_be_bytes());
    j
}
fn closed_fixture() -> (Directory, Vec<u8>) {
    let (dir, d, f) = fixture();
    drop(f);
    drop(d);
    let data = std::fs::read(dir.file()).unwrap();
    (dir, data)
}
#[test]
fn malformed_journals_and_super_journals_are_preserved_and_never_salvaged_by_pruning() {
    for case in 0..8 {
        let (dir, main) = closed_fixture();
        let mut j = synthetic(&main);
        match case {
            0 => j.truncate(1000),
            1 => {
                let end = j.len() - 1;
                j[end] ^= 1;
            }
            2 => j[512..516].fill(0),
            3 => j[..8].fill(0),
            4 => {
                j[8..12].copy_from_slice(&2u32.to_be_bytes());
                let duplicate = j[512..].to_vec();
                j.extend(duplicate);
            }
            5 => {
                j[24..28].copy_from_slice(&8192u32.to_be_bytes());
            }
            6 => {
                let name = b"external-mj0000009a1";
                j.extend(name);
                j.extend((name.len() as u32).to_be_bytes());
                j.extend(1u32.to_be_bytes());
                j.extend(MAGIC);
            }
            7 => {
                j[8..12].copy_from_slice(&u32::MAX.to_be_bytes());
                j.push(1);
            }
            _ => unreachable!(),
        }
        std::fs::write(sidecar(&dir.file(), "-journal"), j).unwrap();
        let before = bundle(&dir.file());
        let store = BlobStore::default();
        let target = dir.path.join("refused.oskproj");
        assert!(
            recover(&dir.file(), &target, store.clone(), Probe::default()).is_err(),
            "case {case}"
        );
        assert_eq!(bundle(&dir.file()), before);
        assert!(!target.exists());
        assert_eq!(scratch_count(&dir), 0);
        assert_eq!(store.usage().unwrap().bytes, 0);
    }
}
#[test]
fn copy_engine_validation_faults_and_cancellation_release_all_resources() {
    for stage in [
        Stage::CopyMain,
        Stage::CopyJournal,
        Stage::BeforeRecovery,
        Stage::AfterRecovery,
        Stage::ReadOriginal,
        Stage::ReadNodes,
        Stage::RecoveryValidated,
        Stage::BeforePublish,
    ] {
        for cancel in [false, true] {
            let (dir, main) = closed_fixture();
            std::fs::write(sidecar(&dir.file(), "-journal"), synthetic(&main)).unwrap();
            let before = bundle(&dir.file());
            let store = BlobStore::default();
            let target = dir.path.join("failed.oskproj");
            let probe = if cancel {
                Probe {
                    cancel_at: Some(stage),
                    ..Probe::default()
                }
            } else {
                Probe {
                    fault: Some(stage),
                    ..Probe::default()
                }
            };
            assert_eq!(
                recover(&dir.file(), &target, store.clone(), probe).unwrap_err(),
                if cancel {
                    StorageError::Cancelled
                } else {
                    StorageError::Io
                },
                "{stage:?}"
            );
            assert_eq!(bundle(&dir.file()), before);
            assert!(!target.exists());
            assert_eq!(scratch_count(&dir), 0);
            assert_eq!(store.usage().unwrap().bytes, 0);
            // All source handles, not just resource counters, must be returned.
            use std::os::windows::fs::OpenOptionsExt;
            assert!(
                std::fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .share_mode(0)
                    .open(dir.file())
                    .is_ok()
            );
        }
    }
}
#[test]
fn publication_ambiguity_cleanup_failure_and_existing_targets_never_overwrite_the_source() {
    let (dir, _main) = closed_fixture();
    let before = bundle(&dir.file());
    let target = dir.path.join("published.oskproj");
    let store = BlobStore::default();
    assert!(matches!(
        recover(
            &dir.file(),
            &target,
            store.clone(),
            Probe {
                fault: Some(Stage::AfterPublish),
                ..Probe::default()
            }
        ),
        Err(StorageError::OutcomeUnknown(_))
    ));
    let (f, c) = open(&target, store.clone(), RestoreSelection::Latest);
    drop(c);
    drop(f);
    assert_eq!(store.usage().unwrap().bytes, 0);
    let bytes = std::fs::read(&target).unwrap();
    assert_eq!(
        recover(&dir.file(), &target, store.clone(), Probe::default()).unwrap_err(),
        StorageError::TargetExists
    );
    assert_eq!(std::fs::read(&target).unwrap(), bytes);
    let target = dir.path.join("cleanup-failed.oskproj");
    assert!(
        matches!(recover(&dir.file(),&target,store.clone(),Probe{fault:Some(Stage::RecoveryValidated),fail_cleanup:true,..Probe::default()}),Err(StorageError::RecoveryCleanup{cause,..}) if *cause==StorageError::Io)
    );
    assert!(!target.exists());
    assert!(scratch_count(&dir) > 0);
    assert_eq!(store.usage().unwrap().bytes, 0);
    assert_eq!(bundle(&dir.file()), before);
}
#[test]
fn empty_sidecars_wal_and_budget_cases_are_explicit_and_source_preserving() {
    let (dir, _) = closed_fixture();
    std::fs::write(sidecar(&dir.file(), "-journal"), []).unwrap();
    let before = bundle(&dir.file());
    let store = BlobStore::default();
    let target = dir.path.join("empty-journal.oskproj");
    let r = recover(&dir.file(), &target, store.clone(), Probe::default()).unwrap();
    assert_eq!(r.report.mode, RecoveryMode::VerifiedCopy);
    assert_eq!(r.report.source_journal.as_ref().unwrap().byte_len, 0);
    drop(r);
    assert_eq!(bundle(&dir.file()), before);
    std::fs::write(sidecar(&dir.file(), "-wal"), b"retain unsupported WAL").unwrap();
    assert_eq!(
        recover(
            &dir.file(),
            &dir.path.join("wal-refused.oskproj"),
            store.clone(),
            Probe::default()
        )
        .unwrap_err(),
        StorageError::Unsupported("WAL recovery")
    );
    assert_eq!(
        std::fs::read(sidecar(&dir.file(), "-wal")).unwrap(),
        b"retain unsupported WAL"
    );
    assert_eq!(bundle(&dir.file()), before);
    assert_eq!(store.usage().unwrap().bytes, 0);
}

#[test]
fn source_sidecar_names_cannot_be_used_as_recovery_destinations() {
    for suffix in ["-journal", "-wal", "-shm", "-JOURNAL."] {
        let (dir, _) = closed_fixture();
        let before = bundle(&dir.file());
        let target = sidecar(&dir.file(), suffix);
        assert!(
            recover(&dir.file(), &target, BlobStore::default(), Probe::default()).is_err(),
            "{suffix}"
        );
        assert_eq!(bundle(&dir.file()), before);
        assert!(!target.exists());
        assert_eq!(scratch_count(&dir), 0);
    }
}

#[test]
fn admission_limits_and_pre_cancel_leave_source_and_destination_untouched() {
    let (dir, _) = closed_fixture();
    let before = bundle(&dir.file());
    let target = dir.path.join("limit.oskproj");
    let store = BlobStore::with_limits(1, 1).unwrap();
    assert_eq!(
        recover(&dir.file(), &target, store.clone(), Probe::default()).unwrap_err(),
        StorageError::Limit("original admission")
    );
    assert_eq!(store.usage().unwrap().bytes, 0);
    assert!(!target.exists());
    assert_eq!(scratch_count(&dir), 0);
    let c = CaptureCancellation::default();
    c.cancel();
    assert_eq!(
        SqliteProjectFile::recover_copy(
            dir.file(),
            &target,
            BlobStore::default(),
            FileLimits::default(),
            RestoreSelection::Manual,
            c
        )
        .unwrap_err(),
        StorageError::Cancelled
    );
    assert_eq!(
        SqliteProjectFile::recover_copy(
            dir.file(),
            &target,
            BlobStore::default(),
            FileLimits {
                file_bytes: 1,
                ..FileLimits::default()
            },
            RestoreSelection::Manual,
            CaptureCancellation::default()
        )
        .unwrap_err(),
        StorageError::Limit("recovery source")
    );
    assert_eq!(bundle(&dir.file()), before);
    assert!(!target.exists());
    assert_eq!(scratch_count(&dir), 0);
}
