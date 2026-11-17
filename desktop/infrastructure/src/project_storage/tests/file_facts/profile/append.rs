use super::*;

fn golden_digest(case: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/P3_2B1_2I_VECTORS.json");
    let vectors: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    vectors["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["name"] == case)
        .unwrap()["sha256"]
        .as_str()
        .unwrap()
        .to_owned()
}

#[test]
fn conditional_manual_digest_matches_independent_append_and_later_historical_observation() {
    let (dir, req, _) = historical_metadata::profile_fixture(None, None, true);
    let scratch = Directory::new();
    let store = BlobStore::default();
    let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    let (mut d, req) = restore(&mut reader, &dir, req, &store);
    let latest = NodeId::new(3).unwrap();
    d.nav(HistoryAction::Checkout { node: latest });
    let (_d, _port, work, mut req) = save::attach_work(d, req);
    let ticket = work.ticket();
    let q = reader
        .prepare_save(work, req.selection, &store, CaptureCancellation::default())
        .unwrap();
    let view = reader
        .inspect(&dir.file(), req, CaptureCancellation::default())
        .unwrap();
    let baseline = context(&view, &q, 2);
    let prediction = reader
        .project_fresh_append_metadata(&view, &q, baseline, CaptureCancellation::default())
        .unwrap()
        .unwrap();
    assert_eq!(prediction.context, baseline);
    assert_eq!(prediction.origin, q.captured_metadata().unwrap().origin);
    assert_eq!(hex(&prediction.metadata), golden_digest("manual_latest"));
    assert_ne!(prediction.before, prediction.metadata);
    assert_eq!(
        prediction.conclusion,
        FileCaptureConclusion::InsufficientForSaveReconciliation
    );
    drop(view);

    let mut writer = save::load_driver(&dir.file(), &store, 90);
    writer.nav(HistoryAction::Checkout { node: latest });
    let (mut file, _) = open(&dir.file(), store.clone(), req.selection);
    let write = || request(ticket.commit.get(), Some(2), ticket.origin);
    let ArchiveWriteOutcome::Committed(actual) =
        file.write_capture(&mut writer.capture(), write()).unwrap()
    else {
        panic!("expected independent fresh append")
    };
    assert_eq!(SaveCommitFact::from(&actual), prediction.receipt);
    assert_eq!(
        file.write_capture(&mut writer.capture(), write()).unwrap(),
        ArchiveWriteOutcome::Replayed(actual)
    );
    writer.set("later independent edit");
    assert!(matches!(
        file.write_capture(
            &mut writer.capture(),
            request(3001, Some(ticket.commit.get()), SaveOrigin::Exit)
        )
        .unwrap(),
        ArchiveWriteOutcome::Committed(_)
    ));
    drop(file);
    req.binding.expected = ExpectedFileCommit {
        commit: CommitId::new(3001),
        generation: Some(4),
    };
    let later = reader
        .inspect(&dir.file(), req, CaptureCancellation::default())
        .unwrap();
    let evidence = later
        .compare_save(
            &q,
            FileSaveCompareContext {
                version: FILE_SAVE_INTENT_VERSION,
                observation: later.summary().observation,
                ticket,
            },
        )
        .unwrap()
        .commit
        .unwrap();
    assert!(
        evidence.legacy_fields_equal
            && evidence.generation_equal
            && evidence.navigation_equal
            && !evidence.is_latest
    );
    for (commit, matches_prediction) in [(ticket.commit.get(), true), (3001, false)] {
        let selected = later
            .historical_metadata(FileHistoricalMetadataRequest {
                version: FILE_HISTORICAL_METADATA_VERSION,
                observation: later.summary().observation,
                file: req,
                commit: CommitId::new(commit).unwrap(),
            })
            .unwrap()
            .facts
            .unwrap();
        assert_eq!(selected.metadata == prediction.metadata, matches_prediction);
    }
    assert!(
        reader
            .project_fresh_append_metadata(
                &later,
                &q,
                context(&later, &q, 2),
                CaptureCancellation::default()
            )
            .unwrap()
            .is_none()
    );
}

#[test]
fn nochange_keeps_file_bytes_despite_a_conditional_digest_and_cancel_releases_workspace() {
    let (dir, req, _) = historical_metadata::profile_fixture(None, None, true);
    let original = std::fs::read(dir.file()).unwrap();
    let store = BlobStore::default();
    let scratch = Directory::new();
    let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    let (d, req) = restore(&mut reader, &dir, req, &store);
    let (_d, _port, work, req) = save::attach_work(d, req);
    let ticket = work.ticket();
    let q = reader
        .prepare_save(work, req.selection, &store, CaptureCancellation::default())
        .unwrap();
    let view = reader
        .inspect(&dir.file(), req, CaptureCancellation::default())
        .unwrap();
    let c = context(&view, &q, 2);
    let kept = reader.usage().managed_bytes;
    support::probe(
        &mut reader,
        Probe {
            cancel_at: Some(Stage::ReadFreshAppendMetadata),
            ..Default::default()
        },
    );
    assert_eq!(
        reader.project_fresh_append_metadata(&view, &q, c, CaptureCancellation::default()),
        Err(FileFactsError::Cancelled)
    );
    assert_eq!(reader.usage().managed_bytes, kept);
    support::probe(&mut reader, Probe::default());
    let prediction = reader
        .project_fresh_append_metadata(&view, &q, c, CaptureCancellation::default())
        .unwrap()
        .unwrap();
    assert_eq!(hex(&prediction.metadata), golden_digest("manual_opening"));
    assert_eq!(reader.usage().managed_bytes, kept);
    assert_eq!(
        reader.project_fresh_append_metadata(
            &view,
            &q,
            context(&view, &q, 900),
            CaptureCancellation::default()
        ),
        Err(FileFactsError::InvalidRequest)
    );
    drop(view);
    let mut writer = save::load_driver(&dir.file(), &store, 90);
    let (mut file, _) = open(&dir.file(), store, req.selection);
    assert!(matches!(
        file.write_capture(
            &mut writer.capture(),
            request(ticket.commit.get(), Some(2), ticket.origin)
        )
        .unwrap(),
        ArchiveWriteOutcome::NoChange(_)
    ));
    drop(file);
    assert_eq!(std::fs::read(dir.file()).unwrap(), original);
}

#[test]
fn older_exit_id_is_occupied_even_after_manual_and_protected_pointers_advance() {
    let (dir, mut author, mut req) = golden();
    let store = BlobStore::default();
    let (mut file, _) = open(&dir.file(), store.clone(), req.selection);
    author.set("advance beyond the Exit receipt");
    file.write_capture(
        &mut author.capture(),
        request(600, Some(401), SaveOrigin::Manual),
    )
    .unwrap();
    drop(file);
    req.binding.expected = ExpectedFileCommit {
        commit: CommitId::new(600),
        generation: Some(3),
    };
    let scratch = Directory::new();
    let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    let (mut d, req) = restore(&mut reader, &dir, req, &store);
    d.ids = Ids(400);
    let (_d, _port, work, req) = save::attach_work(d, req);
    assert_eq!(work.ticket().commit.get(), 401);
    let q = reader
        .prepare_save(work, req.selection, &store, CaptureCancellation::default())
        .unwrap();
    assert!(q.summary().would_commit.is_some());
    let view = reader
        .inspect(&dir.file(), req, CaptureCancellation::default())
        .unwrap();
    assert!(
        reader
            .project_fresh_append_metadata(
                &view,
                &q,
                context(&view, &q, 600),
                CaptureCancellation::default()
            )
            .unwrap()
            .is_none()
    );
    drop(view);
    let (mut file, _) = open(&dir.file(), store, req.selection);
    assert!(matches!(
        file.write_capture(
            &mut author.capture(),
            request(401, Some(600), SaveOrigin::Manual)
        ),
        Err(StorageError::Invalid("commit id reused"))
    ));
}

#[test]
fn reader_limits_and_same_logical_metadata_with_different_source_bytes() {
    let (dir, req, _) = historical_metadata::profile_fixture(None, None, true);
    let changed = Directory::new();
    std::fs::copy(dir.file(), changed.file()).unwrap();
    Connection::open(changed.file())
        .unwrap()
        .execute_batch("VACUUM")
        .unwrap();
    assert_ne!(
        sha(&std::fs::read(dir.file()).unwrap()),
        sha(&std::fs::read(changed.file()).unwrap())
    );
    for commits in [2, 3] {
        let store = BlobStore::default();
        let scratch = Directory::new();
        let mut reader = FileFactsReader::open(
            &scratch.path,
            FileFactsLimits {
                commits,
                ..Default::default()
            },
        )
        .unwrap();
        let (d, req) = restore(&mut reader, &dir, req, &store);
        let (_d, _port, work, req) = save::attach_work(d, req);
        let q = reader
            .prepare_save(work, req.selection, &store, CaptureCancellation::default())
            .unwrap();
        let view = reader
            .inspect(&dir.file(), req, CaptureCancellation::default())
            .unwrap();
        let baseline = context(&view, &q, 2);
        let kept = reader.usage().managed_bytes;
        let mut wrong_ticket = baseline;
        wrong_ticket.ticket.coverage.0 += 1;
        assert_eq!(
            reader.project_fresh_append_metadata(
                &view,
                &q,
                wrong_ticket,
                CaptureCancellation::default()
            ),
            Err(FileFactsError::StalePage)
        );
        let other_root = Directory::new();
        let other = FileFactsReader::open(&other_root.path, FileFactsLimits::default()).unwrap();
        assert_eq!(
            other.project_fresh_append_metadata(
                &view,
                &q,
                baseline,
                CaptureCancellation::default()
            ),
            Err(FileFactsError::StalePage)
        );
        for short_by in [1, 0] {
            let hold = reader.hold_append_workspace_margin(short_by);
            let reserved = reader.usage().managed_bytes;
            let result = reader.project_fresh_append_metadata(
                &view,
                &q,
                baseline,
                CaptureCancellation::default(),
            );
            if short_by == 1 {
                assert_eq!(result, Err(FileFactsError::Limit(FileFactsLimit::Control)));
            } else if commits == 2 {
                assert_eq!(result, Err(FileFactsError::Limit(FileFactsLimit::Commits)));
            } else {
                assert_eq!(
                    hex(&result.unwrap().unwrap().metadata),
                    golden_digest("manual_opening")
                );
            }
            assert_eq!(reader.usage().managed_bytes, reserved);
            drop(hold);
            assert_eq!(reader.usage().managed_bytes, kept);
        }
        for bad in [
            FileCapturedMetadataContext {
                version: 2,
                ..baseline
            },
            FileCapturedMetadataContext {
                observation: FileObservationId {
                    sequence: 99,
                    ..baseline.observation
                },
                ..baseline
            },
            FileCapturedMetadataContext {
                file: FileFactsRequest {
                    selection: RestoreSelection::Latest,
                    ..req
                },
                ..baseline
            },
        ] {
            assert!(
                reader
                    .project_fresh_append_metadata(&view, &q, bad, CaptureCancellation::default())
                    .is_err()
            );
        }
        drop(view);
        let view = reader
            .inspect(&changed.file(), req, CaptureCancellation::default())
            .unwrap();
        let c = context(&view, &q, 2);
        assert_eq!(
            view.compare_captured_metadata(&q, c)
                .unwrap()
                .metadata_equal,
            Some(true)
        );
        let retained = reader.usage().managed_bytes;
        assert!(
            reader
                .project_fresh_append_metadata(&view, &q, c, CaptureCancellation::default())
                .unwrap()
                .is_none()
        );
        assert_eq!(reader.usage().managed_bytes, retained);
    }
}

#[test]
fn legacy_nonmanual_appends_preserve_manual_marks_and_match_the_conditional_vectors() {
    // No application Work is issued for either origin here. Exercise the old
    // writer's receipt rules against the same independent models as the encoder.
    for (origin, case) in [
        (SaveOrigin::Exit, "exit"),
        (SaveOrigin::Automatic, "automatic"),
    ] {
        let (dir, mut req, _) = historical_metadata::profile_fixture(None, None, true);
        let store = BlobStore::default();
        let mut writer = save::load_driver(&dir.file(), &store, 90);
        writer.ids = Ids(7000);
        writer.set("preserve this new branch when returning to C");
        writer.nav(HistoryAction::Checkout {
            node: NodeId::new(3).unwrap(),
        });
        let (mut file, _) = open(&dir.file(), store, req.selection);
        let ArchiveWriteOutcome::Committed(receipt) = file
            .write_capture(&mut writer.capture(), request(2001, Some(2), origin))
            .unwrap()
        else {
            panic!("expected a fresh checkpoint")
        };
        assert_eq!(receipt.head.get(), 1006);
        drop(file);
        req.binding.expected = ExpectedFileCommit {
            commit: Some(receipt.commit),
            generation: Some(3),
        };
        let scratch = Directory::new();
        let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
        let view = reader
            .inspect(&dir.file(), req, CaptureCancellation::default())
            .unwrap();
        let selected = view
            .historical_metadata(FileHistoricalMetadataRequest {
                version: FILE_HISTORICAL_METADATA_VERSION,
                observation: view.summary().observation,
                file: req,
                commit: receipt.commit,
            })
            .unwrap()
            .facts
            .unwrap();
        assert_eq!(hex(&selected.metadata), golden_digest(case));
        assert_eq!(selected.manual_marks, 1);
        assert_eq!(view.summary().files.manual.unwrap().commit.get(), 900);
    }
}
