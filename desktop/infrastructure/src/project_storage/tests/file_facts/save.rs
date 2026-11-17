use super::*;

pub(super) fn load_driver(path: &Path, store: &BlobStore, epoch: u128) -> Driver {
    let (file, candidate) = open(path, store.clone(), RestoreSelection::Manual);
    drop(file);
    let session = candidate
        .into_session(
            HistorySessionId::new(epoch).unwrap(),
            Box::new(store.clone()),
            SessionLimits::default(),
            CaptureCancellation::default(),
        )
        .unwrap();
    Driver {
        session,
        request: 0,
        ids: Ids(2000),
    }
}
fn fixture() -> (
    Directory,
    BlobStore,
    Driver,
    SavePreparationPort,
    SavePreparationWork,
    FileFactsRequest,
) {
    fixture_on(false)
}
pub(super) fn fixture_on(
    latest: bool,
) -> (
    Directory,
    BlobStore,
    Driver,
    SavePreparationPort,
    SavePreparationWork,
    FileFactsRequest,
) {
    let (dir, _d, req) = golden();
    let store = BlobStore::default();
    let mut d = load_driver(&dir.file(), &store, 88);
    if latest {
        d.nav(HistoryAction::Checkout {
            node: NodeId::new(4).unwrap(),
        });
    }
    let (d, port, work, req) = attach_work(d, req);
    (dir, store, d, port, work, req)
}
fn attach_work(
    mut d: Driver,
    mut req: FileFactsRequest,
) -> (
    Driver,
    SavePreparationPort,
    SavePreparationWork,
    FileFactsRequest,
) {
    let port = d
        .session
        .install_save_preparation(SavePreparationConfig {
            version: 1,
            binding: req.binding,
            control: SaveControlProfile::default(),
            capture: save_fixture_capture_limits(),
        })
        .unwrap();
    let status = d.session.save_coordination_status(1).unwrap();
    let ticket = d
        .session
        .request_save(
            SaveRequest {
                api_version: API_VERSION,
                capture_version: HISTORY_CAPTURE_VERSION,
                key: status.next_key.unwrap(),
                source: d.session.save_source(),
                binding: req.binding.key,
                origin: SaveOrigin::Manual,
            },
            &mut d.ids,
        )
        .unwrap()
        .ticket();
    let work = d.session.take_save_preparation(&port, ticket).unwrap();
    req.key = FileFactsKey {
        session: ticket.key.session,
        sequence: ticket.key.sequence,
    };
    (d, port, work, req)
}
fn context(file: &InspectedFileFacts, q: &FileSaveCandidate) -> FileSaveCompareContext {
    FileSaveCompareContext {
        version: FILE_SAVE_INTENT_VERSION,
        observation: file.summary().observation,
        ticket: q.summary().ticket,
    }
}
fn vectors() -> Value {
    serde_json::from_str(
        &std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../docs/P3_2B1_2B_VECTORS.json"),
        )
        .unwrap(),
    )
    .unwrap()
}
pub(super) fn committed(path: &Path, store: &BlobStore, ticket: SaveTicket) {
    let mut writer = load_driver(path, store, 90);
    writer.nav(HistoryAction::Checkout {
        node: NodeId::new(4).unwrap(),
    });
    let (mut file, _) = open(path, store.clone(), RestoreSelection::Manual);
    assert!(matches!(
        file.write_capture(
            &mut writer.capture(),
            request(
                ticket.commit.get(),
                ticket.target.expected.commit.map(CommitId::get),
                ticket.origin
            )
        )
        .unwrap(),
        ArchiveWriteOutcome::Committed(_)
    ));
}
#[test]
fn actual_work_ticket_and_opening_navigation_match_independent_vectors_without_saving() {
    let (dir, store, d, _port, work, req) = fixture();
    let before = d.session.snapshot();
    let ticket = work.ticket();
    let scratch = Directory::new();
    let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    let q = reader
        .prepare_save(work, req.selection, &store, CaptureCancellation::default())
        .unwrap();
    assert_eq!(q.summary().ticket, ticket);
    let v = vectors();
    let s = q.summary();
    assert_eq!(hex(&s.prefix), v["prefix"].as_str().unwrap());
    assert_eq!(
        hex(&s.legacy_request),
        v["legacy_request"].as_str().unwrap()
    );
    assert_eq!(
        hex(&s.navigation),
        v["cases"][0]["sha256"].as_str().unwrap()
    );
    assert_eq!(
        hex(&s.archive_before),
        v["cases"][2]["sha256"].as_str().unwrap()
    );
    assert_eq!(
        hex(&s.would_commit.unwrap().archive),
        v["cases"][3]["sha256"].as_str().unwrap()
    );
    // This hypothetical projection must not turn an actual NoChange into a
    // new commit. The independent legacy writer preserves the existing file.
    let bytes = std::fs::read(dir.file()).unwrap();
    let mut writer = load_driver(&dir.file(), &store, 90);
    let (mut archive, _) = open(&dir.file(), store.clone(), RestoreSelection::Manual);
    assert!(matches!(
        archive
            .write_capture(
                &mut writer.capture(),
                request(ticket.commit.get(), Some(401), SaveOrigin::Manual)
            )
            .unwrap(),
        ArchiveWriteOutcome::NoChange(_)
    ));
    drop(archive);
    assert_eq!(std::fs::read(dir.file()).unwrap(), bytes);
    let file = reader
        .inspect(&dir.file(), req, CaptureCancellation::default())
        .unwrap();
    let result = file.compare_save(&q, context(&file, &q)).unwrap();
    assert!(result.baseline_expected_equal && result.content_equal && result.originals_equal);
    assert!(result.opening_navigation_equal);
    assert!(!result.latest_navigation_equal);
    assert_eq!(result.before_archive_equal, Some(true));
    assert_eq!(result.commit, None);
    assert_eq!(
        result.conclusion,
        FileCaptureConclusion::InsufficientForSaveReconciliation
    );
    assert_eq!(
        hex(&file.intent_facts().latest_navigation),
        v["cases"][1]["sha256"].as_str().unwrap()
    );
    assert_eq!(d.session.snapshot(), before);
    assert_eq!(
        d.session
            .save_coordination_status(1)
            .unwrap()
            .active
            .unwrap()
            .outcome,
        Some(SavePreparationOutcome::CompleteNotSaved)
    );
}
#[test]
fn actual_work_preserves_branch_derived_redo_without_confusing_latest_checkpoint() {
    let (dir, _d, req) = branched_golden();
    let store = BlobStore::default();
    let d = load_driver(&dir.file(), &store, 88);
    let (_d, _port, work, req) = attach_work(d, req);
    assert_eq!(
        work.snapshot().manifest().redo_preferences,
        vec![(
            PersistentNodeId::new(1006).unwrap(),
            PersistentNodeId::new(1007).unwrap()
        )]
    );
    let scratch = Directory::new();
    let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    let q = reader
        .prepare_save(work, req.selection, &store, CaptureCancellation::default())
        .unwrap();
    assert_eq!(
        hex(&q.summary().navigation),
        vectors()["cases"][5]["sha256"].as_str().unwrap()
    );
    let file = reader
        .inspect(&dir.file(), req, CaptureCancellation::default())
        .unwrap();
    let c = file.compare_save(&q, context(&file, &q)).unwrap();
    assert!(c.opening_navigation_equal);
    assert!(!c.latest_navigation_equal);
}
#[test]
fn actual_append_and_later_file_preserve_original_expected_and_per_commit_evidence() {
    let (dir, store, d, _port, work, mut req) = fixture_on(true);
    let original = work.ticket();
    let scratch = Directory::new();
    let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    let q = reader
        .prepare_save(work, req.selection, &store, CaptureCancellation::default())
        .unwrap();
    let v = vectors();
    assert_eq!(
        hex(&q.summary().legacy_request),
        v["latest_intent"]["legacy_request"].as_str().unwrap()
    );
    assert_eq!(
        hex(&q.summary().would_commit.unwrap().archive),
        v["latest_intent"]["archive"]["sha256"].as_str().unwrap()
    );
    committed(&dir.file(), &store, original);
    req.binding.expected = ExpectedFileCommit {
        commit: Some(original.commit),
        generation: Some(3),
    };
    {
        let file = reader
            .inspect(&dir.file(), req, CaptureCancellation::default())
            .unwrap();
        let c = file.compare_save(&q, context(&file, &q)).unwrap();
        let e = c.commit.unwrap();
        assert!(!c.baseline_expected_equal);
        assert!(c.content_equal && c.originals_equal);
        assert_eq!(c.before_archive_equal, None);
        assert!(e.legacy_fields_equal && e.generation_equal && e.navigation_equal && e.is_latest);
        assert_eq!(e.would_commit_archive_equal, Some(true));
        assert_eq!(c.intent.ticket.target.expected, original.target.expected);
        assert_eq!(
            c.conclusion,
            FileCaptureConclusion::InsufficientForSaveReconciliation
        );
        let mut changed = q.summary();
        changed.ticket.target.expected.generation = Some(99);
        let page = file.commits(super::page(file.summary(), 0, 16)).unwrap();
        let fact = page.entries[..page.len]
            .iter()
            .flatten()
            .find(|p| p.fact.commit == original.commit)
            .copied()
            .unwrap();
        let changed = support::commit_match(
            changed,
            fact,
            q.summary().navigation,
            file.intent_facts().archive,
            file.summary().files.latest,
        );
        assert!(changed.legacy_fields_equal);
        assert!(!changed.generation_equal);
    }
    let mut writer = load_driver(&dir.file(), &store, 91);
    writer.set("later independent branch");
    let (mut file, _) = open(&dir.file(), store.clone(), RestoreSelection::Latest);
    file.write_capture(
        &mut writer.capture(),
        request(3001, Some(original.commit.get()), SaveOrigin::Exit),
    )
    .unwrap();
    drop(file);
    req.binding.expected = ExpectedFileCommit {
        commit: Some(CommitId::new(3001).unwrap()),
        generation: Some(4),
    };
    let file = reader
        .inspect(&dir.file(), req, CaptureCancellation::default())
        .unwrap();
    let c = file.compare_save(&q, context(&file, &q)).unwrap();
    let e = c.commit.unwrap();
    assert!(e.legacy_fields_equal && e.generation_equal && e.navigation_equal);
    assert!(!e.is_latest);
    assert_eq!(e.would_commit_archive_equal, None);
    assert!(!c.content_equal);
    assert_eq!(
        d.session
            .save_coordination_status(1)
            .unwrap()
            .active
            .unwrap()
            .outcome,
        Some(SavePreparationOutcome::CompleteNotSaved)
    );
}
#[test]
fn actual_work_with_a_new_edit_matches_independently_written_mixed_record_history() {
    let (dir, _d, req) = golden();
    let store = BlobStore::default();
    let mut d = load_driver(&dir.file(), &store, 88);
    d.set("fresh work branch 雪\r\n");
    let (_d, _port, work, mut req) = attach_work(d, req);
    let ticket = work.ticket();
    let manifest = work.snapshot().manifest();
    assert_eq!(
        manifest.nodes.iter().filter(|n| n.origin.is_some()).count(),
        4
    );
    assert_eq!(
        manifest.nodes.iter().filter(|n| n.origin.is_none()).count(),
        1
    );
    let scratch = Directory::new();
    let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    let q = reader
        .prepare_save(work, req.selection, &store, CaptureCancellation::default())
        .unwrap();
    let mut writer = load_driver(&dir.file(), &store, 90);
    writer.set("fresh work branch 雪\r\n");
    let (mut archive, _) = open(&dir.file(), store.clone(), RestoreSelection::Manual);
    assert!(matches!(
        archive
            .write_capture(
                &mut writer.capture(),
                request(ticket.commit.get(), Some(401), SaveOrigin::Manual)
            )
            .unwrap(),
        ArchiveWriteOutcome::Committed(_)
    ));
    drop(archive);
    req.binding.expected = ExpectedFileCommit {
        commit: Some(ticket.commit),
        generation: Some(3),
    };
    let file = reader
        .inspect(&dir.file(), req, CaptureCancellation::default())
        .unwrap();
    let c = file.compare_save(&q, context(&file, &q)).unwrap();
    let e = c.commit.unwrap();
    assert!(
        c.content_equal
            && c.originals_equal
            && e.legacy_fields_equal
            && e.generation_equal
            && e.navigation_equal
    );
    assert_eq!(e.would_commit_archive_equal, Some(true));
    assert_eq!(
        c.conclusion,
        FileCaptureConclusion::InsufficientForSaveReconciliation
    );
}
#[test]
fn outer_cancellation_after_complete_keeps_core_complete_not_saved() {
    for stage in [Stage::States, Stage::BeforePublish] {
        let (_dir, store, mut d, _port, work, req) = fixture();
        let ticket = work.ticket();
        let scratch = Directory::new();
        let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
        support::probe(
            &mut reader,
            Probe {
                cancel_at: Some(stage),
                ..Probe::default()
            },
        );
        let before = reader.usage().managed_bytes;
        let cancel = CaptureCancellation::default();
        assert!(matches!(
            reader.prepare_save(work, req.selection, &store, cancel.clone()),
            Err(FileFactsError::Cancelled)
        ));
        assert!(cancel.is_cancelled());
        let active = d
            .session
            .save_coordination_status(1)
            .unwrap()
            .active
            .unwrap();
        assert_eq!(
            active.outcome,
            Some(SavePreparationOutcome::CompleteNotSaved)
        );
        assert!(active.work_released);
        assert_eq!(reader.usage().managed_bytes, before);
        assert_eq!(
            d.session.acknowledge_save(ticket),
            Ok(SaveAcknowledgement::Released)
        );
    }
}
#[test]
fn full_ticket_and_observation_context_mismatches_are_not_hidden_by_equal_hashes() {
    let (dir, store, _d, _port, work, req) = fixture();
    let scratch = Directory::new();
    let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    let q = reader
        .prepare_save(work, req.selection, &store, CaptureCancellation::default())
        .unwrap();
    let file = reader
        .inspect(&dir.file(), req, CaptureCancellation::default())
        .unwrap();
    let valid = context(&file, &q);
    for n in 0..8 {
        let mut c = valid;
        match n {
            0 => c.observation.sequence += 1,
            1 => c.ticket.coverage.0 += 1,
            2 => c.ticket.capture.0 += 1,
            3 => c.ticket.source.navigation_revision += 1,
            4 => c.ticket.target.expected.generation = Some(1),
            5 => c.ticket.origin = SaveOrigin::Exit,
            6 => c.ticket.target.key.generation += 1,
            _ => c.version += 1,
        }
        assert!(file.compare_save(&q, c).is_err());
    }
    drop(file);
    let mut changed = req;
    changed.binding.target = SaveTargetId::new(99).unwrap();
    let file = reader
        .inspect(&dir.file(), changed, CaptureCancellation::default())
        .unwrap();
    assert_eq!(
        file.compare_save(&q, context(&file, &q)),
        Err(FileFactsError::StalePage)
    );
}
#[test]
fn ack_and_reader_drop_wait_for_real_work_and_external_snapshot_owners() {
    let (_dir, store, mut d, _port, work, req) = fixture();
    let snapshot = work.snapshot().clone();
    let ticket = work.ticket();
    let scratch = Directory::new();
    let reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    let counter = support::control_observation(&reader);
    let q = reader
        .prepare_save(work, req.selection, &store, CaptureCancellation::default())
        .unwrap();
    assert_eq!(
        d.session.acknowledge_save(ticket),
        Ok(SaveAcknowledgement::WaitingResources)
    );
    assert!(
        !d.session
            .save_coordination_status(1)
            .unwrap()
            .active
            .unwrap()
            .work_released
    );
    drop(reader);
    assert!(counter.usage().bytes > 0);
    drop(q);
    assert_eq!(counter.usage().bytes, 0);
    assert!(
        d.session
            .save_coordination_status(1)
            .unwrap()
            .active
            .unwrap()
            .work_released
    );
    assert_eq!(
        d.session.acknowledge_save(ticket),
        Ok(SaveAcknowledgement::WaitingResources)
    );
    drop(snapshot);
    assert_eq!(
        d.session.acknowledge_save(ticket),
        Ok(SaveAcknowledgement::Released)
    );
}
#[test]
fn work_cancel_partial_entry_and_adapter_cancel_never_publish_saved_results() {
    for mode in 0..3 {
        let (_dir, store, mut d, _port, mut work, req) = fixture();
        let ticket = work.ticket();
        let scratch = Directory::new();
        let reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
        let cancel = CaptureCancellation::default();
        let mut node = None;
        match mode {
            0 => d.session.cancel_save(ticket).unwrap(),
            1 => {
                let CapturePoll::Node(n) = work.poll(64).unwrap() else {
                    panic!("node")
                };
                node = Some(n);
            }
            _ => cancel.cancel(),
        }
        let baseline = reader.usage().managed_bytes;
        assert!(
            reader
                .prepare_save(work, req.selection, &store, cancel)
                .is_err()
        );
        assert_eq!(reader.usage().managed_bytes, baseline);
        assert!(matches!(
            d.session
                .save_coordination_status(1)
                .unwrap()
                .active
                .unwrap()
                .outcome,
            Some(SavePreparationOutcome::StoppedNotSaved(_))
        ));
        assert!(
            d.session
                .save_coordination_status(1)
                .unwrap()
                .active
                .unwrap()
                .work_released
        );
        if node.is_some() {
            assert_eq!(
                d.session.acknowledge_save(ticket),
                Ok(SaveAcknowledgement::WaitingResources)
            );
        }
        drop(node);
        assert_eq!(
            d.session.acknowledge_save(ticket),
            Ok(SaveAcknowledgement::Released)
        );
    }
}
#[test]
fn work_release_precedes_its_wrapper_charge_and_temporary_admission_failure_releases_both() {
    let (_dir, _store, d, _port, work, _req) = fixture();
    let scratch = Directory::new();
    let reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    let before = reader.usage().managed_bytes;
    support::release_work_observed(&reader, work, || {
        assert!(
            d.session
                .save_coordination_status(1)
                .unwrap()
                .active
                .unwrap()
                .work_released
        );
        assert_eq!(
            reader.usage().managed_bytes,
            before + support::save_controls().0
        );
    });
    assert_eq!(reader.usage().managed_bytes, before);
    drop(reader);
    let (_dir, store, d, _port, work, req) = fixture();
    let ticket = work.ticket();
    let (extra, temporary) = support::save_controls();
    let reader = FileFactsReader::open(
        &scratch.path,
        FileFactsLimits {
            control_bytes: before + extra + temporary - 1,
            ..FileFactsLimits::default()
        },
    )
    .unwrap();
    assert!(matches!(
        reader.prepare_save(work, req.selection, &store, CaptureCancellation::default()),
        Err(FileFactsError::Limit(FileFactsLimit::Control))
    ));
    assert_eq!(reader.usage().managed_bytes, before);
    assert!(
        d.session
            .save_coordination_status(1)
            .unwrap()
            .active
            .unwrap()
            .work_released
    );
    assert_ne!(ticket.commit.get(), 0);
}
#[test]
fn save_work_and_live_file_share_exact_control_admission_without_reclaiming_owners() {
    for exact in [true, false] {
        let (dir, store, mut d, _port, work, req) = fixture();
        let ticket = work.ticket();
        let (retained, work_bytes) = support::capture_controls();
        let (extra, context_work) = support::save_controls();
        let required = support::controls() + retained + work_bytes + extra + context_work;
        let scratch = Directory::new();
        let mut reader = FileFactsReader::open(
            &scratch.path,
            FileFactsLimits {
                control_bytes: required - usize::from(!exact),
                ..FileFactsLimits::default()
            },
        )
        .unwrap();
        let file = reader
            .inspect(&dir.file(), req, CaptureCancellation::default())
            .unwrap();
        let before = reader.usage().managed_bytes;
        let result =
            reader.prepare_save(work, req.selection, &store, CaptureCancellation::default());
        if exact {
            let q = result.unwrap();
            assert_eq!(reader.usage().managed_peak_bytes, required);
            assert_eq!(reader.usage().managed_bytes, before + retained + extra);
            drop(q);
        } else {
            assert_eq!(
                result
                    .err()
                    .map(|e| matches!(e, FileFactsError::Limit(FileFactsLimit::Control))),
                Some(true)
            );
        }
        assert_eq!(reader.usage().managed_bytes, before);
        drop(file);
        assert_eq!(
            d.session.acknowledge_save(ticket),
            Ok(SaveAcknowledgement::Released)
        );
    }
}
#[test]
fn metadata_projection_preserves_bytes_order_and_only_predicts_new_commits() {
    let (_dir, _store, _d, _port, work, _req) = fixture();
    let ticket = work.ticket();
    let before = work.snapshot().manifest().archive_metadata.clone();
    let mut metadata = before.as_ref().clone();
    let v = vectors();
    for t in v["extra_model"]["tags"].as_array().unwrap() {
        metadata.tags.push(ArchiveTag {
            node: PersistentNodeId::new(t["node"].as_u64().unwrap() as u128).unwrap(),
            name: t["name"].as_str().unwrap().into(),
            value: t["value"].as_str().unwrap().into(),
        });
    }
    for e in v["extra_model"]["extensions"].as_array().unwrap() {
        metadata.extensions.push(ArchiveExtension {
            name: e["name"].as_str().unwrap().into(),
            bytes: bytes(e["bytes"].as_str().unwrap()),
        });
    }
    let raw = metadata.clone();
    assert_eq!(
        hex(&support::intent_archive(&metadata, None).unwrap()),
        v["cases"][4]["sha256"].as_str().unwrap()
    );
    assert_eq!(metadata, raw);
    for origin in [SaveOrigin::Exit, SaveOrigin::Automatic] {
        let mut t = ticket;
        t.origin = origin;
        let delta = support::intent_prediction(&metadata, t).unwrap();
        let hash = support::intent_archive(&metadata, Some(&delta)).unwrap();
        let mut expected = metadata.clone();
        expected.last_commit = Some(delta.clone());
        expected.protected = Some(delta);
        assert_eq!(hash, support::intent_archive(&expected, None).unwrap());
        assert_eq!(metadata.manual, before.manual);
    }
    let mut exhausted = ticket;
    exhausted.target.expected.generation = Some(u64::MAX);
    assert_eq!(support::intent_prediction(&metadata, exhausted), None);
    assert_eq!(metadata, raw);
}
#[test]
fn context_length_gates_and_duplicate_redo_reject_before_fixed_indexing() {
    let m = ArchiveMetadata {
        tags: vec![
            ArchiveTag {
                node: PersistentNodeId::new(1).unwrap(),
                name: "user:x".into(),
                value: String::new()
            };
            4097
        ],
        ..ArchiveMetadata::default()
    };
    assert!(support::intent_archive(&m, None).is_err());
    let id = PersistentNodeId::new(1).unwrap();
    let redo = vec![(id, id); 65];
    assert!(support::intent_navigation(id, 0, 0, &redo).is_err());
    assert!(support::intent_navigation(id, 0, 0, &redo[..2]).is_err());
}
