use super::*;
mod append;
mod close;
mod coverage;
mod cursor;

fn models() -> Value {
    serde_json::from_str(
        &std::fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/P3_2B1_2H_VECTORS.json"),
        )
        .unwrap(),
    )
    .unwrap()
}
fn features(case: &str) -> Vec<Value> {
    models()["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["name"] == case)
        .unwrap()["model"]["features"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| json!({"name":f["name"],"version":f["version"],"required":f["required"]==1}))
        .collect()
}
fn expected(case: &str) -> String {
    models()["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["name"] == case)
        .unwrap()["sha256"]
        .as_str()
        .unwrap()
        .to_owned()
}
fn restore(
    reader: &mut FileFactsReader,
    dir: &Directory,
    mut req: FileFactsRequest,
    store: &BlobStore,
) -> (Driver, FileFactsRequest) {
    req.key.session = HistorySessionId::new(88).unwrap();
    req.key.sequence = 9; // Original read sequence is not the later save sequence.
    let candidate = reader
        .restore_profile(
            &dir.file(),
            FileProfileRestoreRequest {
                version: 1,
                file: req,
            },
            store,
            CaptureCancellation::default(),
        )
        .unwrap();
    assert_eq!(
        candidate.file_profile(2).err(),
        Some(FileFactsError::UnsupportedVersion)
    );
    let origin = candidate.file_profile(1).unwrap().unwrap().origin();
    assert_eq!(origin.file, req);
    assert_eq!(
        origin.source_sha256,
        sha(&std::fs::read(dir.file()).unwrap())
    );
    assert_eq!(origin.opened_head, candidate.header().head);
    let session = candidate
        .into_session(
            req.key.session,
            Box::new(store.clone()),
            SessionLimits::default(),
            CaptureCancellation::default(),
        )
        .unwrap();
    (
        Driver {
            session,
            request: 0,
            ids: Ids(2000),
        },
        req,
    )
}
fn context(
    view: &InspectedFileFacts,
    q: &FileSaveCandidate,
    commit: u128,
) -> FileCapturedMetadataContext {
    FileCapturedMetadataContext {
        version: 1,
        observation: view.summary().observation,
        file: view.summary().request,
        commit: CommitId::new(commit).unwrap(),
        ticket: q.summary().ticket,
    }
}

#[test]
fn actual_profile_tracks_latest_exit_not_manual_opening_and_preserves_full_before() {
    let (dir, req, roots) = historical_metadata::profile_fixture(None, None, true);
    let raw = std::fs::read(dir.file()).unwrap();
    let scratch = Directory::new();
    let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    let store = BlobStore::default();
    let (d, req) = restore(&mut reader, &dir, req, &store);
    let source_bytes = reader.usage().managed_bytes - 16384;
    assert_eq!(source_bytes, support::profile_controls(73));
    let (mut d, _port, work, req) = save::attach_work(d, req);
    let ticket = work.ticket();
    let snapshot = work.snapshot().clone();
    // Pure scope probes on a real authenticated source. A's separate core
    // regression confirms that actual confirmation advances expected but keeps origin.
    for mode in 0..4 {
        let mut changed = ticket;
        match mode {
            0 => changed.target.expected.commit = Some(CommitId::new(2001).unwrap()),
            1 => changed.target.expected.generation = Some(3),
            2 => changed.target.key.generation += 1,
            _ => changed.source.session_id = HistorySessionId::new(89).unwrap(),
        }
        assert!(support::profile_work_scope(&reader, &work, changed).is_none());
    }
    let source = snapshot.file_profile(1).unwrap().unwrap().origin();
    assert_eq!(source.features_root, roots[1]);
    assert_ne!(source.features_root, roots[0]);
    assert_eq!(source.opened_head.get(), 1005);
    assert_eq!(source.file.binding.expected.commit.unwrap().get(), 2);
    assert_ne!(source.file.key.sequence, req.key.sequence);
    let q = reader
        .prepare_save(work, req.selection, &store, CaptureCancellation::default())
        .unwrap();
    let before = q.captured_metadata().unwrap();
    assert_eq!(
        hex(&before.metadata),
        expected("latest_exit_opening_manual")
    );
    assert_eq!((before.features, before.required_features), (5, 4));
    let view = reader
        .inspect(&dir.file(), req, CaptureCancellation::default())
        .unwrap();
    let result = view
        .compare_captured_metadata(&q, context(&view, &q, 2))
        .unwrap();
    assert_eq!(result.metadata_equal, Some(true));
    assert_eq!(
        result.conclusion,
        FileCaptureConclusion::InsufficientForSaveReconciliation
    );
    assert_eq!(
        view.compare_captured_metadata(&q, context(&view, &q, 900))
            .unwrap()
            .metadata_equal,
        Some(false)
    );
    let missing = view
        .compare_captured_metadata(&q, context(&view, &q, 999))
        .unwrap();
    assert!(
        missing.selected.is_none()
            && missing.captured.is_some()
            && missing.metadata_equal.is_none()
    );
    for n in 0..10 {
        let mut bad = context(&view, &q, 999);
        match n {
            0 => bad.version += 1,
            1 => bad.observation.sequence += 1,
            2 => bad.ticket.source.navigation_revision += 1,
            3 => bad.ticket.capture.0 += 1,
            4 => bad.ticket.coverage.0 += 1,
            5 => bad.file.key.sequence += 1,
            6 => bad.file.binding.target = SaveTargetId::new(99).unwrap(),
            7 => bad.ticket.target.expected.generation = Some(1),
            8 => bad.file.selection = RestoreSelection::Latest,
            _ => bad.file.version += 1,
        }
        assert!(view.compare_captured_metadata(&q, bad).is_err());
    }
    assert_eq!(
        view.compare_historical_save(
            &q,
            FileHistoricalSaveContext {
                version: 1,
                observation: view.summary().observation,
                file: req,
                commit: CommitId::new(2).unwrap(),
                ticket: q.summary().ticket
            }
        )
        .unwrap()
        .metadata,
        FileHistoricalSaveMetadata::NotCompared
    );
    // Typical source + two retained observations + real q, with Session alive.
    let copy = Directory::new();
    std::fs::write(copy.file(), &raw).unwrap();
    let second = reader
        .inspect(&copy.file(), req, CaptureCancellation::default())
        .unwrap();
    assert!(reader.usage().managed_peak_bytes <= 512 * 1024);
    drop(second);
    drop(view);
    drop(q);
    // Use the actual ticket from the active slot; profile ownership is independent.
    assert_eq!(
        d.session.acknowledge_save(ticket),
        Ok(SaveAcknowledgement::WaitingResources)
    );
    drop(snapshot);
    assert_eq!(
        d.session.acknowledge_save(ticket),
        Ok(SaveAcknowledgement::Released)
    );
    assert_eq!(reader.usage().managed_bytes, 16384 + source_bytes);
    drop(d);
    assert_eq!(reader.usage().managed_bytes, 16384);
    assert_eq!(std::fs::read(dir.file()).unwrap(), raw);
}

#[test]
fn same_count_real_optional_changes_never_replace_a_captured_profile() {
    let (base, req, _) = historical_metadata::profile_fixture(None, None, false);
    let scratch = Directory::new();
    let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    let store = BlobStore::default();
    let (d, req) = restore(&mut reader, &base, req, &store);
    let (d, _port, work, req) = save::attach_work(d, req);
    let q = reader
        .prepare_save(work, req.selection, &store, CaptureCancellation::default())
        .unwrap();
    assert_eq!(
        hex(&q.captured_metadata().unwrap().metadata),
        expected("captured_latest")
    );
    for case in [
        "same_count_name",
        "same_count_version",
        "nul_and_empty_version",
    ] {
        let (changed, _, _) =
            historical_metadata::profile_fixture(None, Some(features(case)), false);
        let view = reader
            .inspect(&changed.file(), req, CaptureCancellation::default())
            .unwrap();
        let result = view
            .compare_captured_metadata(&q, context(&view, &q, 2))
            .unwrap();
        assert_eq!(result.metadata_equal, Some(false));
        assert_eq!(hex(&result.selected.unwrap().metadata), expected(case));
        assert_eq!(result.captured, q.captured_metadata());
        let old = view
            .compare_save(
                &q,
                FileSaveCompareContext {
                    version: 1,
                    observation: view.summary().observation,
                    ticket: q.summary().ticket,
                },
            )
            .unwrap();
        assert!(old.content_equal && old.originals_equal && old.opening_navigation_equal);
        drop(view);
        // The same modified file can produce its own genuine immutable source.
        let (other, other_req) = restore(&mut reader, &changed, req, &store);
        let (other, _other_port, work, other_req) = save::attach_work(other, other_req);
        let other_q = reader
            .prepare_save(
                work,
                other_req.selection,
                &store,
                CaptureCancellation::default(),
            )
            .unwrap();
        assert_eq!(other_q.summary(), q.summary());
        assert_eq!(
            hex(&other_q.captured_metadata().unwrap().metadata),
            expected(case)
        );
        drop(other_q);
        drop(other);
    }
    drop(q);
    drop(d);
    assert_eq!(reader.usage().managed_bytes, 16384);
}

#[test]
fn source_and_metadata_cancellation_cleanup_and_failed_activation_publish_nothing() {
    let (dir, mut req) = copy_golden_file();
    req.key.session = HistorySessionId::new(88).unwrap();
    let original = std::fs::read(dir.file()).unwrap();
    let scratch = Directory::new();
    let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    let store = BlobStore::default();
    for stage in [
        Stage::ReadMetadataSummary,
        Stage::ReadOriginalSummary,
        Stage::ReadProfileSource,
    ] {
        support::probe(
            &mut reader,
            Probe {
                cancel_at: Some(stage),
                ..Probe::default()
            },
        );
        assert!(matches!(
            reader.restore_profile(
                &dir.file(),
                FileProfileRestoreRequest {
                    version: 1,
                    file: req
                },
                &store,
                CaptureCancellation::default()
            ),
            Err(FileFactsError::Cancelled)
        ));
        assert_eq!(reader.usage().managed_bytes, 16384);
        assert_eq!(reader.usage().scratch_reserved_bytes, 0);
        assert_eq!(store.usage().unwrap().bytes, 0);
    }
    support::probe(&mut reader, Probe::default());
    support::cleanup(&mut reader, true);
    assert!(matches!(
        reader.restore_profile(
            &dir.file(),
            FileProfileRestoreRequest {
                version: 1,
                file: req
            },
            &store,
            CaptureCancellation::default()
        ),
        Err(FileFactsError::Cleanup { .. })
    ));
    assert_eq!(store.usage().unwrap().bytes, 0);
    support::cleanup(&mut reader, false);
    reader.retry_cleanup().unwrap();
    assert_eq!(reader.usage().managed_bytes, 16384);
    let candidate = reader
        .restore_profile(
            &dir.file(),
            FileProfileRestoreRequest {
                version: 1,
                file: req,
            },
            &store,
            CaptureCancellation::default(),
        )
        .unwrap();
    assert!(matches!(
        candidate.into_session(
            HistorySessionId::new(89).unwrap(),
            Box::new(store.clone()),
            SessionLimits::default(),
            CaptureCancellation::default()
        ),
        Err(RestoreError::Invalid("profile session"))
    ));
    assert_eq!(store.usage().unwrap().bytes, 0);
    assert_eq!(reader.usage().managed_bytes, 16384);
    let (d, req) = restore(&mut reader, &dir, req, &store);
    let (mut d, _port, work, req) = save::attach_work(d, req);
    let ticket = work.ticket();
    let retained = reader.usage().managed_bytes;
    support::probe(
        &mut reader,
        Probe {
            cancel_at: Some(Stage::ReadCapturedMetadata),
            ..Probe::default()
        },
    );
    assert!(matches!(
        reader.prepare_save(work, req.selection, &store, CaptureCancellation::default()),
        Err(FileFactsError::Cancelled)
    ));
    assert_eq!(reader.usage().managed_bytes, retained);
    assert_eq!(
        d.session.acknowledge_save(ticket),
        Ok(SaveAcknowledgement::Released)
    );
    drop(d);
    assert_eq!(reader.usage().managed_bytes, 16384);
    assert_eq!(store.usage().unwrap().bytes, 0);
    assert_eq!(std::fs::read(dir.file()).unwrap(), original);
}

#[test]
fn legal_maximum_profile_has_exact_text_charge_and_retains_q_when_next_view_is_limited() {
    let mut list = features("wrong_writer_defaults");
    for i in 0..252 {
        let prefix = format!("optional.{i:03}.");
        list.push(json!({"name":format!("{}{}",prefix,"x".repeat(256-prefix.len())),"version":"v".repeat(64),"required":false}));
    }
    assert_eq!(
        list.iter()
            .map(|f| f["name"].as_str().unwrap().len() + f["version"].as_str().unwrap().len())
            .sum::<usize>(),
        80702
    );
    let (dir, req, _) = historical_metadata::profile_fixture(None, Some(list), false);
    let copy = Directory::new();
    std::fs::copy(dir.file(), copy.file()).unwrap();
    let scratch = Directory::new();
    let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    let store = BlobStore::default();
    let (d, req) = restore(&mut reader, &dir, req, &store);
    let profile_bytes = support::profile_controls(80702);
    assert_eq!(reader.usage().managed_bytes, 16384 + profile_bytes);
    let (mut d, _port, work, req) = save::attach_work(d, req);
    let ticket = work.ticket();
    let q = reader
        .prepare_save(work, req.selection, &store, CaptureCancellation::default())
        .unwrap();
    assert_eq!(
        (
            q.captured_metadata().unwrap().features,
            q.captured_metadata().unwrap().required_features
        ),
        (256, 4)
    );
    let view = reader
        .inspect(&dir.file(), req, CaptureCancellation::default())
        .unwrap();
    let kept = reader.usage().managed_bytes;
    let before = q.captured_metadata();
    assert!(matches!(
        reader.inspect(&copy.file(), req, CaptureCancellation::default()),
        Err(FileFactsError::Limit(FileFactsLimit::Control))
    ));
    assert_eq!(reader.usage().managed_bytes, kept);
    assert_eq!(q.captured_metadata(), before);
    let projection = reader
        .project_fresh_append_metadata(
            &view,
            &q,
            context(&view, &q, 2),
            CaptureCancellation::default(),
        )
        .unwrap()
        .unwrap();
    assert_eq!(projection.origin, before.unwrap().origin);
    assert!(
        reader
            .compare_no_change_coverage(
                &view,
                &q,
                context(&view, &q, 2),
                CaptureCancellation::default()
            )
            .unwrap()
            .unwrap()
            .unchanged_opening
    );
    assert_eq!(reader.usage().managed_bytes, kept);
    assert_eq!(
        view.compare_captured_metadata(&q, context(&view, &q, 2))
            .unwrap()
            .metadata_equal,
        Some(true)
    );
    assert!(reader.usage().managed_peak_bytes <= 512 * 1024);
    drop(view);
    drop(q);
    assert_eq!(
        d.session.acknowledge_save(ticket),
        Ok(SaveAcknowledgement::Released)
    );
    assert_eq!(reader.usage().managed_bytes, 16384 + profile_bytes);
    drop(d);
    assert_eq!(reader.usage().managed_bytes, 16384);
}

struct Fake {
    origin: FileProfileOrigin,
    bytes: usize,
}
impl FileProfileSource for Fake {
    fn origin(&self) -> FileProfileOrigin {
        self.origin
    }
    fn logical_bytes(&self) -> usize {
        self.bytes
    }
}

#[test]
fn absent_forged_and_foreign_profiles_cannot_qualify_or_bypass_missing_guards() {
    let (dir, mut req) = copy_golden_file();
    let scratch = Directory::new();
    let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    let store = BlobStore::default();
    req.key.session = HistorySessionId::new(88).unwrap();
    let candidate = reader
        .restore_profile(
            &dir.file(),
            FileProfileRestoreRequest {
                version: 1,
                file: req,
            },
            &store,
            CaptureCancellation::default(),
        )
        .unwrap();
    let real_origin = candidate.file_profile(1).unwrap().unwrap().origin();
    drop(candidate);
    assert_eq!(reader.usage().managed_bytes, 16384);
    assert_eq!(store.usage().unwrap().bytes, 0);
    for mode in 0..2 {
        let (file, mut candidate) = open(&dir.file(), store.clone(), req.selection);
        drop(file);
        assert!(candidate.file_profile(1).unwrap().is_none());
        if mode == 1 {
            for bad in 0..7 {
                let mut origin = real_origin;
                let mut bytes = 1;
                match bad {
                    0 => origin.version += 1,
                    1 => {
                        origin.file.binding.project =
                            skin_editor_application::values::ProjectId::new(99).unwrap()
                    }
                    2 => origin.root = PersistentNodeId::new(99).unwrap(),
                    3 => origin.opened_head = PersistentNodeId::new(99).unwrap(),
                    4 => origin.file.binding.expected.generation = Some(1),
                    5 => bytes = MAX_FILE_PROFILE_BYTES + 1,
                    _ => bytes = 16 * 1024 * 1024,
                }
                assert!(
                    candidate
                        .attach_file_profile(1, Arc::new(Fake { origin, bytes }))
                        .is_err()
                );
                assert!(candidate.file_profile(1).unwrap().is_none());
            }
            candidate
                .attach_file_profile(
                    1,
                    Arc::new(Fake {
                        origin: real_origin,
                        bytes: 1,
                    }),
                )
                .unwrap();
            assert!(
                candidate
                    .attach_file_profile(
                        1,
                        Arc::new(Fake {
                            origin: real_origin,
                            bytes: 1
                        })
                    )
                    .is_err()
            );
        }
        let session = candidate
            .into_session(
                req.key.session,
                Box::new(store.clone()),
                SessionLimits::default(),
                CaptureCancellation::default(),
            )
            .unwrap();
        let d = Driver {
            session,
            request: 0,
            ids: Ids(2000),
        };
        let (mut d, _port, work, qreq) = save::attach_work(d, req);
        let ticket = work.ticket();
        let q = reader
            .prepare_save(work, qreq.selection, &store, CaptureCancellation::default())
            .unwrap();
        assert!(q.captured_metadata().is_none());
        let view = reader
            .inspect(&dir.file(), qreq, CaptureCancellation::default())
            .unwrap();
        let c = context(&view, &q, 999);
        let result = view.compare_captured_metadata(&q, c).unwrap();
        assert!(
            result.captured.is_none()
                && result.selected.is_none()
                && result.metadata_equal.is_none()
        );
        let parent = context(&view, &q, req.binding.expected.commit.unwrap().get());
        assert!(
            reader
                .compare_no_change_coverage(&view, &q, parent, CaptureCancellation::default())
                .unwrap()
                .is_none()
        );
        assert!(
            reader
                .project_fresh_append_metadata(&view, &q, parent, CaptureCancellation::default())
                .unwrap()
                .is_none()
        );
        let mut bad = c;
        bad.ticket.capture.0 += 1;
        assert_eq!(
            view.compare_captured_metadata(&q, bad),
            Err(FileFactsError::StalePage)
        );
        drop(view);
        drop(q);
        assert_eq!(
            d.session.acknowledge_save(ticket),
            Ok(SaveAcknowledgement::Released)
        );
        drop(d);
    }
    let (d, req) = restore(&mut reader, &dir, req, &store);
    let (mut d, _port, work, req) = save::attach_work(d, req);
    let ticket = work.ticket();
    let other_scratch = Directory::new();
    let other = FileFactsReader::open(&other_scratch.path, FileFactsLimits::default()).unwrap();
    let q = other
        .prepare_save(work, req.selection, &store, CaptureCancellation::default())
        .unwrap();
    assert!(q.captured_metadata().is_none());
    let view = reader
        .inspect(&dir.file(), req, CaptureCancellation::default())
        .unwrap();
    assert_eq!(
        view.compare_captured_metadata(&q, context(&view, &q, 999)),
        Err(FileFactsError::StalePage)
    );
    drop(view);
    drop(q);
    assert_eq!(
        d.session.acknowledge_save(ticket),
        Ok(SaveAcknowledgement::Released)
    );
}

#[test]
fn real_wrong_target_work_is_unavailable_and_late_file_cannot_rewrite_old_q() {
    let (dir, req) = copy_golden_file();
    let scratch = Directory::new();
    let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    let store = BlobStore::default();
    for mode in 0..2 {
        let (d, mut read_req) = restore(&mut reader, &dir, req, &store);
        if mode == 0 {
            read_req.binding.target = SaveTargetId::new(99).unwrap();
        } else {
            read_req.binding.key.generation += 1;
        }
        let (mut d, _port, work, qreq) = save::attach_work(d, read_req);
        let ticket = work.ticket();
        let q = reader
            .prepare_save(work, qreq.selection, &store, CaptureCancellation::default())
            .unwrap();
        assert!(q.captured_metadata().is_none());
        drop(q);
        assert_eq!(
            d.session.acknowledge_save(ticket),
            Ok(SaveAcknowledgement::Released)
        );
        drop(d);
    }
    let (d, req) = restore(&mut reader, &dir, req, &store);
    let (d, _port, work, mut req) = save::attach_work(d, req);
    let q = reader
        .prepare_save(work, req.selection, &store, CaptureCancellation::default())
        .unwrap();
    let before = q.captured_metadata().unwrap();
    // Independently use the old standalone writer solely on this authored file.
    save::committed(&dir.file(), &store, q.summary().ticket);
    req.binding.expected = ExpectedFileCommit {
        commit: Some(q.summary().ticket.commit),
        generation: Some(3),
    };
    let view = reader
        .inspect(&dir.file(), req, CaptureCancellation::default())
        .unwrap();
    let at_old = view
        .compare_captured_metadata(&q, context(&view, &q, 401))
        .unwrap();
    assert_eq!(at_old.metadata_equal, Some(true));
    assert_eq!(at_old.captured, Some(before));
    assert_eq!(
        view.compare_captured_metadata(&q, context(&view, &q, q.summary().ticket.commit.get()))
            .unwrap()
            .metadata_equal,
        Some(false)
    );
    drop(view);
    drop(q);
    drop(d);
    assert_eq!(reader.usage().managed_bytes, 16384);
}

#[test]
fn reader_and_session_drop_keep_profile_and_originals_until_the_last_emitted_node() {
    let (dir, req) = copy_golden_file();
    let scratch = Directory::new();
    let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    let counter = support::control_observation(&reader);
    let store = BlobStore::default();
    let (mut d, _) = restore(&mut reader, &dir, req, &store);
    let profile_bytes = reader.usage().managed_bytes - 16384;
    let mut capture = d.capture();
    let snapshot = capture.snapshot().unwrap();
    let node = loop {
        if let CapturePoll::Node(n) = capture.poll(1).unwrap() {
            break n;
        }
    };
    capture.cancel();
    drop(capture);
    drop(d);
    drop(reader);
    assert_eq!(counter.usage().bytes, profile_bytes);
    assert!(store.usage().unwrap().bytes > 0);
    assert!(snapshot.file_profile(1).unwrap().is_some());
    drop(snapshot);
    assert_eq!(counter.usage().bytes, profile_bytes);
    assert!(store.usage().unwrap().bytes > 0);
    assert_eq!(node.project().id().get(), 7);
    drop(node);
    assert_eq!(counter.usage().bytes, 0);
    assert_eq!(store.usage().unwrap().bytes, 0);
}

#[test]
fn unsupported_required_feature_never_publishes_a_profile_even_with_a_valid_domain_vector() {
    let (dir, req, _) =
        historical_metadata::profile_fixture(None, Some(features("same_count_required")), false);
    let before = std::fs::read(dir.file()).unwrap();
    let scratch = Directory::new();
    let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    let store = BlobStore::default();
    assert!(matches!(
        reader.restore_profile(
            &dir.file(),
            FileProfileRestoreRequest {
                version: 1,
                file: req
            },
            &store,
            CaptureCancellation::default()
        ),
        Err(FileFactsError::Unsupported)
    ));
    assert_eq!(reader.usage().managed_bytes, 16384);
    assert_eq!(reader.usage().scratch_reserved_bytes, 0);
    assert_eq!(store.usage().unwrap().bytes, 0);
    assert_eq!(std::fs::read(dir.file()).unwrap(), before);
}
