use super::*;

fn query(file: &InspectedFileFacts, id: u128) -> FileHistoricalContentRequest {
    FileHistoricalContentRequest {
        version: FILE_HISTORICAL_CONTENT_VERSION,
        observation: file.summary().observation,
        file: file.summary().request,
        commit: CommitId::new(id).unwrap(),
    }
}
fn context(
    file: &InspectedFileFacts,
    q: &FileSaveCandidate,
    id: u128,
) -> FileHistoricalContentCompareContext {
    FileHistoricalContentCompareContext {
        request: query(file, id),
        ticket: q.summary().ticket,
    }
}
fn vectors() -> Value {
    serde_json::from_str(
        &std::fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/P3_2B1_2C_VECTORS.json"),
        )
        .unwrap(),
    )
    .unwrap()
}
fn append_future(path: &Path, store: &BlobStore, previous: SaveTicket) -> BlobRef {
    let mut writer = save::load_driver(path, store, 91);
    writer.ids = Ids(4000);
    writer.nav(HistoryAction::Checkout {
        node: NodeId::new(3).unwrap(),
    });
    let raw = store
        .stage_bytes(b"future-unused-raw-branch".to_vec())
        .unwrap();
    writer.command(vec![Command::RegisterAsset {
        asset: Asset::new(
            AssetId::new(2).unwrap(),
            "future-unused".into(),
            raw.reference(),
            AssetKind::Opaque,
        )
        .unwrap(),
    }]);
    let (mut archive, _) = open(path, store.clone(), RestoreSelection::Manual);
    assert!(matches!(
        archive
            .write_capture(
                &mut writer.capture(),
                request(3001, Some(previous.commit.get()), SaveOrigin::Exit)
            )
            .unwrap(),
        ArchiveWriteOutcome::Committed(_)
    ));
    raw.reference()
}
#[test]
fn historical_prefixes_match_independent_vectors_and_keep_query_provenance() {
    let (dir, _d, req) = golden();
    let before = std::fs::read(dir.file()).unwrap();
    let scratch = Directory::new();
    let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    let file = reader
        .inspect(&dir.file(), req, CaptureCancellation::default())
        .unwrap();
    let v = vectors();
    let usage = reader.usage().managed_bytes;
    for (i, commit) in [400, 401].into_iter().enumerate() {
        let request = query(&file, commit);
        let result = file.historical_content(request).unwrap();
        assert_eq!(result.version, FILE_HISTORICAL_CONTENT_VERSION);
        assert_eq!(result.request, request);
        let facts = result.facts.unwrap();
        assert_eq!(facts.proof_version, FILE_COORD_PROOF_VERSION);
        assert_eq!(facts.commit.fact.commit, request.commit);
        assert_eq!(facts.commit.nodes, i + 3);
        assert_eq!(facts.admissions, 1);
        assert_eq!(
            hex(&facts.content),
            v["cases"][i]["sha256"].as_str().unwrap()
        );
    }
    assert_eq!(
        file.historical_content(query(&file, 401))
            .unwrap()
            .facts
            .unwrap()
            .content,
        file.summary().proof.content
    );
    let absent = query(&file, 999);
    let result = file.historical_content(absent).unwrap();
    assert_eq!(result.request, absent);
    assert_eq!(result.facts, None);
    assert_eq!(reader.usage().managed_bytes, usage);
    drop(file);
    assert_eq!(std::fs::read(dir.file()).unwrap(), before);
}
#[test]
fn shared_node_encoder_keeps_original_state_distinct_from_canonical() {
    let v = vectors();
    let c = &v["cases"][2];
    let n = &c["model"]["node"];
    let store = BlobStore::default();
    let mut d = Driver::new(store);
    let capture = d.capture();
    let snapshot = capture.snapshot().unwrap();
    let mut node = snapshot.manifest().nodes[0].clone();
    node.id = PersistentNodeId::new(n["id"].as_u64().unwrap() as u128).unwrap();
    node.alias = n["alias"].as_u64().unwrap();
    node.coordinate = BranchCoordinate::new(
        n["branch"].as_u64().unwrap(),
        n["ordinal"].as_u64().unwrap(),
    )
    .unwrap();
    node.depth = n["depth"].as_u64().unwrap();
    node.origin = Some(Arc::new(OriginalNodeRecord {
        record_version: n["record_version"].as_u64().unwrap() as u16,
        state: bytes(n["state"].as_str().unwrap()).try_into().unwrap(),
        action: bytes(n["action"].as_str().unwrap()),
        checksum: bytes(n["checksum"].as_str().unwrap()).try_into().unwrap(),
    }));
    let canonical =
        StateHash::from_bytes(bytes(n["canonical"].as_str().unwrap()).try_into().unwrap());
    assert_ne!(node.origin.as_ref().unwrap().state, canonical.bytes());
    let h = &c["model"]["header"];
    let header =
        ["project", "history", "lineage", "epoch", "root"].map(|k| h[k].as_u64().unwrap() as u128);
    assert_eq!(
        hex(&support::single_node_content(header, &node, canonical)),
        c["sha256"].as_str().unwrap()
    );
}
#[test]
fn later_branch_and_unused_asset_do_not_replace_the_real_q_historical_content() {
    let (dir, store, d, _port, work, mut req) = save::fixture_on(true);
    let ticket = work.ticket();
    let before = d.session.snapshot();
    let scratch = Directory::new();
    let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    let q = reader
        .prepare_save(work, req.selection, &store, CaptureCancellation::default())
        .unwrap();
    save::committed(&dir.file(), &store, ticket);
    append_future(&dir.file(), &store, ticket);
    req.binding.expected = ExpectedFileCommit {
        commit: Some(CommitId::new(3001).unwrap()),
        generation: Some(4),
    };
    let file = reader
        .inspect(&dir.file(), req, CaptureCancellation::default())
        .unwrap();
    let current = file
        .compare_save(
            &q,
            FileSaveCompareContext {
                version: FILE_SAVE_INTENT_VERSION,
                observation: file.summary().observation,
                ticket,
            },
        )
        .unwrap();
    assert!(!current.content_equal && !current.originals_equal);
    let old = file
        .compare_historical_content(&q, context(&file, &q, ticket.commit.get()))
        .unwrap();
    assert_eq!(
        (old.content_equal, old.head_equal, old.navigation_equal),
        (Some(true), Some(true), Some(true))
    );
    let f = old.historical.facts.unwrap();
    assert_eq!((f.commit.nodes, f.admissions), (4, 1));
    assert_eq!(old.intent.ticket, ticket);
    assert_eq!(old.intent.ticket.target.expected, ticket.target.expected);
    assert_eq!(
        old.historical.request.file.binding.expected,
        req.binding.expected
    );
    assert_eq!(
        old.conclusion,
        FileCaptureConclusion::InsufficientForSaveReconciliation
    );
    let latest = file
        .historical_content(query(&file, 3001))
        .unwrap()
        .facts
        .unwrap();
    assert_eq!((latest.commit.nodes, latest.admissions), (5, 2));
    assert_ne!(latest.content, f.content);
    // Protection and q have equal node content/head/navigation, but remain
    // distinct commits and save origins. No coverage is installed.
    let other = file
        .compare_historical_content(&q, context(&file, &q, 401))
        .unwrap();
    assert_eq!(
        (
            other.content_equal,
            other.head_equal,
            other.navigation_equal
        ),
        (Some(true), Some(true), Some(true))
    );
    assert_ne!(other.historical.request.commit, other.intent.ticket.commit);
    assert_eq!(
        other.historical.facts.unwrap().commit.fact.origin,
        SaveOrigin::Exit
    );
    assert_eq!(other.intent.ticket.origin, SaveOrigin::Manual);
    assert_eq!(
        other.conclusion,
        FileCaptureConclusion::InsufficientForSaveReconciliation
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
fn content_head_navigation_and_absent_commit_are_separate_without_nochange_qualification() {
    let (dir, store, d, _port, work, req) = save::fixture_on(false);
    let ticket = work.ticket();
    let scratch = Directory::new();
    let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    let q = reader
        .prepare_save(work, req.selection, &store, CaptureCancellation::default())
        .unwrap();
    let file = reader
        .inspect(&dir.file(), req, CaptureCancellation::default())
        .unwrap();
    let protected = file
        .compare_historical_content(&q, context(&file, &q, 401))
        .unwrap();
    assert_eq!(
        (
            protected.content_equal,
            protected.head_equal,
            protected.navigation_equal
        ),
        (Some(true), Some(false), Some(false))
    );
    let manual = file
        .compare_historical_content(&q, context(&file, &q, 400))
        .unwrap();
    assert_eq!(manual.content_equal, Some(false));
    assert_eq!(manual.head_equal, Some(true));
    let absent = file
        .compare_historical_content(&q, context(&file, &q, ticket.commit.get()))
        .unwrap();
    assert_eq!(absent.historical.facts, None);
    assert_eq!(
        (
            absent.content_equal,
            absent.head_equal,
            absent.navigation_equal
        ),
        (None, None, None)
    );
    assert_eq!(absent.intent.ticket, ticket);
    assert_eq!(absent.historical.request.commit, ticket.commit);
    assert_eq!(
        absent.conclusion,
        FileCaptureConclusion::InsufficientForSaveReconciliation
    );
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
fn historical_query_and_real_ticket_context_reject_stale_observations_and_readers() {
    let (dir, store, _d, _port, work, req) = save::fixture_on(true);
    let copies = [Directory::new(), Directory::new()];
    for copy in &copies {
        std::fs::copy(dir.file(), copy.file()).unwrap();
    }
    let scratch = Directory::new();
    let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    let q = reader
        .prepare_save(work, req.selection, &store, CaptureCancellation::default())
        .unwrap();
    let old = reader
        .inspect(&dir.file(), req, CaptureCancellation::default())
        .unwrap();
    let request = query(&old, 401);
    for mode in 0..9 {
        let mut bad = request;
        match mode {
            0 => bad.version += 1,
            1 => bad.file.version += 1,
            2 => bad.observation.sequence += 1,
            3 => bad.file.key.sequence += 1,
            4 => bad.file.selection = RestoreSelection::Latest,
            5 => bad.file.binding.key.generation += 1,
            6 => bad.file.binding.target = SaveTargetId::new(99).unwrap(),
            7 => bad.file.binding.expected.generation = Some(1),
            _ => bad.file.key.session = HistorySessionId::new(99).unwrap(),
        }
        assert!(old.historical_content(bad).is_err());
    }
    let mut c = context(&old, &q, 401);
    c.ticket.commit = c.request.commit;
    assert!(old.compare_historical_content(&q, c).is_err());
    c = context(&old, &q, 401);
    c.ticket.coverage.0 += 1;
    assert!(old.compare_historical_content(&q, c).is_err());
    // The existing live source guard deliberately refuses reopening its file.
    assert!(matches!(
        reader.inspect(&dir.file(), req, CaptureCancellation::default()),
        Err(FileFactsError::Busy)
    ));
    let new = reader
        .inspect(&copies[0].file(), req, CaptureCancellation::default())
        .unwrap();
    assert_eq!(
        new.historical_content(request),
        Err(FileFactsError::StalePage)
    );
    assert_eq!(old.historical_content(request).unwrap().request, request);
    let other_root = Directory::new();
    let mut other_reader =
        FileFactsReader::open(&other_root.path, FileFactsLimits::default()).unwrap();
    let other = other_reader
        .inspect(&copies[1].file(), req, CaptureCancellation::default())
        .unwrap();
    assert_eq!(
        other.compare_historical_content(&q, context(&other, &q, 401)),
        Err(FileFactsError::StalePage)
    );
}
#[test]
fn cancellation_after_historical_hash_keeps_old_views_pages_and_actual_owners() {
    let (dir, store, _d, _port, work, req) = save::fixture_on(false);
    let copy = Directory::new();
    std::fs::copy(dir.file(), copy.file()).unwrap();
    let source = std::fs::read(dir.file()).unwrap();
    let scratch = Directory::new();
    let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    let counter = support::control_observation(&reader);
    let q = reader
        .prepare_save(work, req.selection, &store, CaptureCancellation::default())
        .unwrap();
    let file = reader
        .inspect(&dir.file(), req, CaptureCancellation::default())
        .unwrap();
    let old_query = query(&file, 400);
    let old = file.historical_content(old_query).unwrap();
    let before = reader.usage().managed_bytes;
    support::probe(
        &mut reader,
        Probe {
            cancel_at: Some(Stage::ReadHistoricalContent),
            ..Probe::default()
        },
    );
    let cancel = CaptureCancellation::default();
    assert!(matches!(
        reader.inspect(&copy.file(), req, cancel.clone()),
        Err(FileFactsError::Cancelled)
    ));
    assert!(cancel.is_cancelled());
    assert_eq!(reader.usage().managed_bytes, before);
    assert_eq!(reader.usage().scratch_reserved_bytes, 0);
    assert_eq!(file.historical_content(old_query).unwrap(), old);
    assert_eq!(file.commits(page(file.summary(), 0, 16)).unwrap().len, 2);
    assert_eq!(std::fs::read(copy.file()).unwrap(), source);
    drop(reader);
    assert!(counter.usage().bytes > 0);
    assert_eq!(file.historical_content(old_query).unwrap(), old);
    drop(file);
    assert_eq!(std::fs::read(dir.file()).unwrap(), source);
    assert!(counter.usage().bytes > 0);
    drop(q);
    assert_eq!(counter.usage().bytes, 0);
}
#[test]
fn historical_payload_has_exact_shared_control_admission_and_queries_keep_managed_usage() {
    let (dir, _d, req) = golden();
    for less in [false, true] {
        let scratch = Directory::new();
        let required = support::inspect_controls(1);
        let mut reader = FileFactsReader::open(
            &scratch.path,
            FileFactsLimits {
                control_bytes: required - usize::from(less),
                ..FileFactsLimits::default()
            },
        )
        .unwrap();
        let before = reader.usage().managed_bytes;
        let result = reader.inspect(&dir.file(), req, CaptureCancellation::default());
        if less {
            assert!(matches!(
                result,
                Err(FileFactsError::Limit(FileFactsLimit::Control))
            ));
        } else {
            let file = result.unwrap();
            assert_eq!(reader.usage().managed_peak_bytes, required);
            let current = reader.usage().managed_bytes;
            assert_eq!(current, support::controls());
            let facts = file.historical_content(query(&file, 400)).unwrap();
            assert!(facts.facts.is_some());
            assert_eq!(reader.usage().managed_bytes, current);
            drop(file);
        }
        assert_eq!(reader.usage().managed_bytes, before);
    }
}
#[test]
fn later_branch_and_future_original_corruption_reject_the_entire_historical_observation() {
    for table in ["nodes", "blob_chunks"] {
        let (dir, store, _d, _port, work, mut req) = save::fixture_on(true);
        let ticket = work.ticket();
        drop(work);
        save::committed(&dir.file(), &store, ticket);
        let future = append_future(&dir.file(), &store, ticket);
        req.binding.expected = ExpectedFileCommit {
            commit: Some(CommitId::new(3001).unwrap()),
            generation: Some(4),
        };
        let sql = if table == "nodes" {
            "UPDATE nodes SET checksum=zeroblob(32) WHERE id=(SELECT node_id FROM addresses ORDER BY alias DESC LIMIT 1)".to_owned()
        } else {
            format!(
                "UPDATE blob_chunks SET bytes=zeroblob(length(bytes)) WHERE sha=x'{}'",
                hex(&future.sha256())
            )
        };
        corrupt(&dir.file(), table, &sql);
        let before = std::fs::read(dir.file()).unwrap();
        let scratch = Directory::new();
        let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
        let usage = reader.usage().managed_bytes;
        assert!(matches!(
            reader.inspect(&dir.file(), req, CaptureCancellation::default()),
            Err(FileFactsError::ValidationFailed)
        ));
        assert_eq!(reader.usage().managed_bytes, usage);
        assert_eq!(reader.usage().scratch_reserved_bytes, 0);
        assert_eq!(std::fs::read(dir.file()).unwrap(), before);
    }
}
