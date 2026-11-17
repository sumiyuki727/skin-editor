use super::*;

fn source() -> (BlobStore, RawBlob) {
    let store = BlobStore::default();
    let raw = store.stage_bytes(b"opaque\0original\r\n".to_vec()).unwrap();
    (store, raw)
}
fn context(
    file: &InspectedFileFacts,
    candidate: &FileCaptureCandidate,
) -> FileCaptureCompareContext {
    FileCaptureCompareContext {
        version: FILE_CAPTURE_PROJECTION_VERSION,
        observation: file.summary().observation,
        source: candidate.summary().source,
    }
}
#[test]
fn real_none_capture_matches_independent_full_file_domains_without_saving_or_resolving() {
    let (dir, mut d, req) = golden();
    let before = std::fs::read(dir.file()).unwrap();
    let active = d.session.snapshot();
    let scratch = Directory::new();
    let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    let (store, _raw) = source();
    for _ in 0..2 {
        let capture = d.capture();
        assert!(
            capture
                .snapshot()
                .unwrap()
                .manifest()
                .nodes
                .iter()
                .all(|n| n.origin.is_none())
        );
        let q = reader
            .prepare_capture(req, capture, &store, CaptureCancellation::default())
            .unwrap();
        let file = reader
            .inspect(&dir.file(), req, CaptureCancellation::default())
            .unwrap();
        let comparison = file.compare_capture(&q, context(&file, &q)).unwrap();
        let vectors = vectors();
        assert_eq!(
            hex(&q.summary().projection.content),
            vectors["file_case"]["domains"][0]["sha256"]
                .as_str()
                .unwrap()
        );
        assert_eq!(
            hex(&q.summary().projection.originals),
            vectors["file_case"]["domains"][1]["sha256"]
                .as_str()
                .unwrap()
        );
        assert!(comparison.content_equal && comparison.originals_equal);
        assert!(!comparison.selected_head_equal);
        assert_eq!(
            comparison.conclusion,
            FileCaptureConclusion::InsufficientForSaveReconciliation
        );
        assert_eq!(file.preflight().observation, file.summary().observation);
        assert_eq!(file.preflight().tables[7].rows, 4);
        assert_eq!(file.preflight().tables[2].bytes, 17);
        assert_eq!(d.session.snapshot(), active);
    }
    assert_eq!(std::fs::read(dir.file()).unwrap(), before);
}
#[test]
fn branched_dfs_emission_is_reordered_by_persistent_manifest_identity() {
    let (dir, mut d, mut req) = branched_golden();
    d.nav(HistoryAction::Checkout {
        node: NodeId::new(4).unwrap(),
    });
    d.set("new child of old B after D");
    let (mut file, _) = open(&dir.file(), BlobStore::default(), RestoreSelection::Latest);
    file.write_capture(&mut d.capture(), request(405, Some(404), SaveOrigin::Exit))
        .unwrap();
    drop(file);
    req.binding.expected = ExpectedFileCommit {
        commit: Some(CommitId::new(405).unwrap()),
        generation: Some(6),
    };
    let mut capture = d.capture();
    let snapshot = capture.snapshot().unwrap();
    let mut emitted = Vec::new();
    loop {
        match capture.poll(64).unwrap() {
            CapturePoll::Node(n) => emitted.push(n.info().id),
            CapturePoll::Complete => break,
            CapturePoll::Pending => (),
        }
    }
    assert_ne!(
        emitted,
        snapshot
            .manifest()
            .nodes
            .iter()
            .map(|n| n.id)
            .collect::<Vec<_>>()
    );
    drop(capture);
    drop(snapshot);
    let scratch = Directory::new();
    let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    let (store, _raw) = source();
    let q = reader
        .prepare_capture(req, d.capture(), &store, CaptureCancellation::default())
        .unwrap();
    let file = reader
        .inspect(&dir.file(), req, CaptureCancellation::default())
        .unwrap();
    assert!(
        file.compare_capture(&q, context(&file, &q))
            .unwrap()
            .content_equal
    );
}
#[test]
fn frozen_q_survives_later_commands_and_does_not_claim_current_session_state() {
    let (dir, mut d, req) = golden();
    let capture = d.capture();
    let frozen = capture.snapshot().unwrap().manifest().source;
    d.set("q+1 distinct edit");
    let scratch = Directory::new();
    let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    let (store, _raw) = source();
    let q = reader
        .prepare_capture(req, capture, &store, CaptureCancellation::default())
        .unwrap();
    assert_eq!(q.summary().source, frozen);
    assert_ne!(d.session.project_info().state, frozen.state);
    let file = reader
        .inspect(&dir.file(), req, CaptureCancellation::default())
        .unwrap();
    assert!(
        file.compare_capture(&q, context(&file, &q))
            .unwrap()
            .content_equal
    );
}
#[test]
fn full_source_request_observation_and_reader_namespaces_are_all_required() {
    let (dir, mut d, req) = golden();
    let scratch = Directory::new();
    let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    let (store, _raw) = source();
    let q = reader
        .prepare_capture(req, d.capture(), &store, CaptureCancellation::default())
        .unwrap();
    let file = reader
        .inspect(&dir.file(), req, CaptureCancellation::default())
        .unwrap();
    let correct = context(&file, &q);
    let mut variants = vec![correct; 5];
    variants[0].version += 1;
    variants[1].observation.sequence += 1;
    variants[2].source.navigation_revision += 1;
    variants[3].source.state.revision += 1;
    variants[4].source.session_id = HistorySessionId::new(99).unwrap();
    for v in variants {
        assert!(file.compare_capture(&q, v).is_err());
    }
    drop(file);
    for mut other in [req; 4].into_iter().enumerate() {
        match other.0 {
            0 => other.1.key.sequence += 1,
            1 => other.1.binding.key.generation += 1,
            2 => other.1.binding.target = SaveTargetId::new(99).unwrap(),
            _ => other.1.selection = RestoreSelection::Latest,
        }
        let file = reader
            .inspect(&dir.file(), other.1, CaptureCancellation::default())
            .unwrap();
        assert_eq!(
            file.compare_capture(&q, context(&file, &q)),
            Err(FileFactsError::StalePage)
        );
    }
    let other_root = Directory::new();
    let mut other_reader =
        FileFactsReader::open(&other_root.path, FileFactsLimits::default()).unwrap();
    let file = other_reader
        .inspect(&dir.file(), req, CaptureCancellation::default())
        .unwrap();
    assert_eq!(
        file.compare_capture(&q, context(&file, &q)),
        Err(FileFactsError::StalePage)
    );
}
#[test]
fn partial_producers_and_missing_actual_originals_never_publish_a_candidate() {
    let (_dir, mut d, req) = golden();
    let scratch = Directory::new();
    let reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    for complete in [false, true] {
        let mut capture = d.capture();
        if complete {
            while !matches!(capture.poll(64).unwrap(), CapturePoll::Complete) {}
        } else {
            assert!(matches!(capture.poll(64).unwrap(), CapturePoll::Node(_)));
        }
        assert_eq!(
            reader
                .prepare_capture(
                    req,
                    capture,
                    &BlobStore::default(),
                    CaptureCancellation::default()
                )
                .err(),
            Some(FileFactsError::InvalidRequest)
        );
    }
    assert_eq!(
        reader
            .prepare_capture(
                req,
                d.capture(),
                &BlobStore::default(),
                CaptureCancellation::default()
            )
            .err(),
        Some(FileFactsError::ValidationFailed)
    );
}
#[test]
fn candidate_and_external_snapshot_keep_the_real_capture_slot_until_last_drop() {
    let (_dir, mut d, req) = golden();
    let scratch = Directory::new();
    let reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    let observer = support::control_observation(&reader);
    let (store, _raw) = source();
    let capture = d.capture();
    let snapshot = capture.snapshot().unwrap();
    let q = reader
        .prepare_capture(req, capture, &store, CaptureCancellation::default())
        .unwrap();
    drop(reader);
    assert_eq!(observer.usage().bytes, support::capture_controls().0);
    drop(q);
    assert_eq!(observer.usage().bytes, 0);
    let i = d.session.project_info();
    assert_eq!(
        d.session
            .capture_history(
                CaptureRequest {
                    version: HISTORY_CAPTURE_VERSION,
                    session_id: i.session_id,
                    expected: i.state,
                    limits: CaptureLimits::default(),
                    cancellation: CaptureCancellation::default()
                },
                &mut d.ids
            )
            .err(),
        Some(CaptureError::InFlight)
    );
    drop(snapshot);
    drop(d.capture());
}

fn root_request(d: &mut Driver) -> FileFactsRequest {
    let capture = d.capture();
    let s = capture.snapshot().unwrap();
    let m = s.manifest();
    FileFactsRequest {
        version: FILE_FACTS_PORT_VERSION,
        key: FileFactsKey {
            session: m.source.session_id,
            sequence: 1,
        },
        binding: SaveTargetBinding {
            version: SAVE_COORDINATION_VERSION,
            key: SaveBindingKey {
                id: SaveBindingId::new(11).unwrap(),
                generation: 1,
            },
            target: SaveTargetId::new(12).unwrap(),
            project: m.project_id,
            phase: SaveTargetPhase::ExistingVerified,
            history: Some((&m.identity).into()),
            expected: ExpectedFileCommit {
                commit: Some(CommitId::new(401).unwrap()),
                generation: Some(2),
            },
        },
        selection: RestoreSelection::Manual,
    }
}
fn projection_vectors() -> Value {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/P3_2B1_2A_CAPTURE_VECTORS.json");
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}
fn restored_root(mode: usize) -> (Driver, BlobStore) {
    let data = projection_vectors();
    let original = &data["cases"][1]["node"];
    let store = BlobStore::default();
    let mut d = Driver::new(store.clone());
    let mut capture = d.capture();
    let snapshot = capture.snapshot().unwrap();
    let m = snapshot.manifest();
    let node = match capture.poll(64).unwrap() {
        CapturePoll::Node(n) => n,
        _ => panic!("root"),
    };
    let mut operation = node.info().operation.as_ref().clone();
    if mode == 1 {
        operation.steps.push("inconsistent summary".into());
    }
    let mut checksum: [u8; 32] = bytes(original["checksum"].as_str().unwrap())
        .try_into()
        .unwrap();
    if mode == 2 {
        checksum[0] ^= 1;
    }
    let mut builder = RestoreBuilder::new(
        RestoreHeader {
            node_count: 1,
            project: m.project_id,
            identity: m.identity.clone(),
            root: m.root,
            head: m.head,
            next_alias: m.next_alias,
            next_branch: m.next_branch,
            admissions: vec![],
            redo_preferences: vec![],
            originals: vec![],
            metadata: m.archive_metadata.clone(),
        },
        RestoreLimits::default(),
        CaptureCancellation::default(),
    )
    .unwrap();
    builder
        .push(RestoreNode {
            id: node.info().id,
            parent: None,
            alias: 1,
            coordinate: node.info().coordinate,
            depth: 0,
            operation: Arc::new(operation),
            origin: Arc::new(OriginalNodeRecord {
                record_version: 2,
                state: bytes(original["state"].as_str().unwrap())
                    .try_into()
                    .unwrap(),
                action: bytes(original["action"].as_str().unwrap()),
                checksum,
            }),
            project: node.project().clone(),
        })
        .unwrap();
    let session = builder
        .finish(&store)
        .unwrap()
        .into_session(
            HistorySessionId::new(88).unwrap(),
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
        store,
    )
}
#[test]
fn independent_none_and_some_vectors_preserve_original_action_and_original_state() {
    for some in [false, true] {
        let (mut d, store) = if some {
            restored_root(0)
        } else {
            let store = BlobStore::default();
            (Driver::new(store.clone()), store)
        };
        let req = root_request(&mut d);
        let scratch = Directory::new();
        let reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
        let capture = d.capture();
        let snapshot = capture.snapshot().unwrap();
        let original = snapshot.manifest().nodes[0].origin.clone();
        let q = reader
            .prepare_capture(req, capture, &store, CaptureCancellation::default())
            .unwrap();
        let data = projection_vectors();
        let vector = &data["cases"][usize::from(some)];
        assert_eq!(
            hex(&q.summary().projection.content),
            vector["content_sha256"].as_str().unwrap()
        );
        if some {
            let record = original.as_ref().unwrap();
            assert_eq!(
                record.action,
                bytes(vector["node"]["action"].as_str().unwrap())
            );
            assert_ne!(
                record.state,
                bytes(vector["node"]["canonical"].as_str().unwrap()).as_slice()
            );
        } else {
            assert!(original.is_none());
        }
    }
}
#[test]
fn some_record_semantic_or_checksum_conflict_rejects_the_whole_projection() {
    for mode in [1, 2] {
        let (mut d, store) = restored_root(mode);
        let req = root_request(&mut d);
        let scratch = Directory::new();
        let reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
        let before = reader.usage().managed_bytes;
        assert_eq!(
            reader
                .prepare_capture(req, d.capture(), &store, CaptureCancellation::default())
                .err(),
            Some(FileFactsError::ValidationFailed)
        );
        assert_eq!(reader.usage().managed_bytes, before);
        drop(d.capture());
    }
}
#[test]
fn same_counts_different_states_and_later_file_history_only_yield_partial_differences() {
    let (dir, _d, req) = golden();
    let (_other_dir, mut other, _) = golden_variant("different captured contents 雪");
    let scratch = Directory::new();
    let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    let (store, _raw) = source();
    let q = reader
        .prepare_capture(req, other.capture(), &store, CaptureCancellation::default())
        .unwrap();
    let file = reader
        .inspect(&dir.file(), req, CaptureCancellation::default())
        .unwrap();
    assert_eq!(q.summary().nodes, file.summary().nodes);
    let value = file.compare_capture(&q, context(&file, &q)).unwrap();
    assert!(!value.content_equal && value.originals_equal);
    assert_eq!(
        value.conclusion,
        FileCaptureConclusion::InsufficientForSaveReconciliation
    );
    drop(file);
    drop(q);
    let (later, _d, later_req) = branched_golden();
    let q = reader
        .prepare_capture(
            later_req,
            other.capture(),
            &store,
            CaptureCancellation::default(),
        )
        .unwrap();
    let file = reader
        .inspect(&later.file(), later_req, CaptureCancellation::default())
        .unwrap();
    assert!(file.summary().nodes > q.summary().nodes);
    let value = file.compare_capture(&q, context(&file, &q)).unwrap();
    assert!(!value.content_equal);
    assert_eq!(
        value.conclusion,
        FileCaptureConclusion::InsufficientForSaveReconciliation
    );
}
#[test]
fn wrong_expected_is_preserved_and_cannot_be_rewritten_to_match_the_observation() {
    let (dir, mut d, req) = golden();
    let scratch = Directory::new();
    let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    let (store, _raw) = source();
    let mut wrong = req;
    wrong.binding.expected = ExpectedFileCommit {
        commit: Some(CommitId::new(400).unwrap()),
        generation: Some(1),
    };
    let q = reader
        .prepare_capture(wrong, d.capture(), &store, CaptureCancellation::default())
        .unwrap();
    let file = reader
        .inspect(&dir.file(), req, CaptureCancellation::default())
        .unwrap();
    assert_eq!(q.summary().request.binding.expected, wrong.binding.expected);
    assert_eq!(
        file.compare_capture(&q, context(&file, &q)),
        Err(FileFactsError::StalePage)
    );
}
#[test]
fn live_observation_and_candidate_use_one_exact_control_pool() {
    for exact in [true, false] {
        let (dir, mut d, req) = golden();
        let (retained, work) = support::capture_controls();
        let required = support::controls() + retained + work;
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
        let (store, _raw) = source();
        let result =
            reader.prepare_capture(req, d.capture(), &store, CaptureCancellation::default());
        if exact {
            let q = result.unwrap();
            assert_eq!(reader.usage().managed_peak_bytes, required);
            assert_eq!(reader.usage().managed_bytes, before + retained);
            drop(q);
        } else {
            assert_eq!(
                result.err(),
                Some(FileFactsError::Limit(FileFactsLimit::Control))
            );
        }
        assert_eq!(reader.usage().managed_bytes, before);
        drop(file);
        drop(d.capture());
    }
}
#[test]
fn both_cancellation_tokens_fail_without_partial_candidates_or_premature_release() {
    let (_dir, mut d, req) = golden();
    let (store, _raw) = source();
    let scratch = Directory::new();
    let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    let original_cancel = CaptureCancellation::default();
    let i = d.session.project_info();
    let capture = d
        .session
        .capture_history(
            CaptureRequest {
                version: HISTORY_CAPTURE_VERSION,
                session_id: i.session_id,
                expected: i.state,
                limits: CaptureLimits::default(),
                cancellation: original_cancel.clone(),
            },
            &mut d.ids,
        )
        .unwrap();
    original_cancel.cancel();
    assert_eq!(
        reader
            .prepare_capture(req, capture, &store, CaptureCancellation::default())
            .err(),
        Some(FileFactsError::Cancelled)
    );
    for stage in [Stage::States, Stage::Originals, Stage::BeforePublish] {
        support::probe(
            &mut reader,
            Probe {
                cancel_at: Some(stage),
                ..Probe::default()
            },
        );
        let cancel = CaptureCancellation::default();
        let before = reader.usage().managed_bytes;
        assert_eq!(
            reader
                .prepare_capture(req, d.capture(), &store, cancel.clone())
                .err(),
            Some(FileFactsError::Cancelled)
        );
        assert!(cancel.is_cancelled());
        assert_eq!(reader.usage().managed_bytes, before);
        drop(d.capture());
    }
}
#[test]
fn retained_node_consumer_keeps_capture_ownership_after_partial_entry_rejection() {
    let (_dir, mut d, req) = golden();
    let scratch = Directory::new();
    let reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    let mut capture = d.capture();
    let node = match capture.poll(64).unwrap() {
        CapturePoll::Node(n) => n,
        _ => panic!("root"),
    };
    assert_eq!(
        reader
            .prepare_capture(
                req,
                capture,
                &BlobStore::default(),
                CaptureCancellation::default()
            )
            .err(),
        Some(FileFactsError::InvalidRequest)
    );
    let i = d.session.project_info();
    assert_eq!(
        d.session
            .capture_history(
                CaptureRequest {
                    version: HISTORY_CAPTURE_VERSION,
                    session_id: i.session_id,
                    expected: i.state,
                    limits: CaptureLimits::default(),
                    cancellation: CaptureCancellation::default()
                },
                &mut d.ids
            )
            .err(),
        Some(CaptureError::InFlight)
    );
    drop(node);
    drop(d.capture());
}

#[test]
fn equal_counts_with_different_actual_originals_do_not_compare_equal() {
    let (dir, _d, req) = golden();
    let store = BlobStore::default();
    let raw = store.stage_bytes(b"opaque\0ORIGINAL\r\n".to_vec()).unwrap();
    let mut d = Driver::new(store.clone());
    d.command(vec![Command::RegisterAsset {
        asset: Asset::new(
            AssetId::new(1).unwrap(),
            "opaque".into(),
            raw.reference(),
            AssetKind::Opaque,
        )
        .unwrap(),
    }]);
    d.set("Unknown:雪\r\n");
    d.set("Unknown:雪!\r\n");
    let scratch = Directory::new();
    let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    let q = reader
        .prepare_capture(req, d.capture(), &store, CaptureCancellation::default())
        .unwrap();
    let file = reader
        .inspect(&dir.file(), req, CaptureCancellation::default())
        .unwrap();
    assert_eq!(q.summary().nodes, file.summary().nodes);
    assert_eq!(q.summary().originals, file.summary().originals);
    assert_eq!(q.summary().original_bytes, file.summary().original_bytes);
    let value = file.compare_capture(&q, context(&file, &q)).unwrap();
    assert!(!value.content_equal && !value.originals_equal);
    assert_eq!(
        value.conclusion,
        FileCaptureConclusion::InsufficientForSaveReconciliation
    );
}
#[test]
fn a_second_candidate_is_aggregated_without_reclaiming_the_first_owner() {
    for exact in [true, false] {
        let (dir, mut d, req) = golden();
        let (mut second, second_store) = restored_root(0);
        let second_req = root_request(&mut second);
        let (retained, work) = support::capture_controls();
        let required = support::controls() + 2 * retained + work;
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
        let (store, _raw) = source();
        let first = reader
            .prepare_capture(req, d.capture(), &store, CaptureCancellation::default())
            .unwrap();
        let before = reader.usage().managed_bytes;
        let result = reader.prepare_capture(
            second_req,
            second.capture(),
            &second_store,
            CaptureCancellation::default(),
        );
        if exact {
            let second = result.unwrap();
            assert_eq!(reader.usage().managed_bytes, before + retained);
            drop(second);
        } else {
            assert_eq!(
                result.err(),
                Some(FileFactsError::Limit(FileFactsLimit::Control))
            );
        }
        assert_eq!(reader.usage().managed_bytes, before);
        assert!(
            file.compare_capture(&first, context(&file, &first))
                .unwrap()
                .content_equal
        );
    }
}
