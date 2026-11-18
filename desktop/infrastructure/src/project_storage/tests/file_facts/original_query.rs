use super::*;

fn query(file: &InspectedFileFacts, id: u128) -> FileHistoricalOriginalsRequest {
    FileHistoricalOriginalsRequest {
        version: FILE_HISTORICAL_ORIGINALS_VERSION,
        observation: file.summary().observation,
        file: file.summary().request,
        commit: CommitId::new(id).unwrap(),
    }
}
fn context(
    file: &InspectedFileFacts,
    q: &FileSaveCandidate,
    id: u128,
) -> FileHistoricalOriginalsCompareContext {
    FileHistoricalOriginalsCompareContext {
        request: query(file, id),
        ticket: q.summary().ticket,
    }
}
fn original_vectors() -> Value {
    serde_json::from_str(
        &std::fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/P3_2B1_2E_VECTORS.json"),
        )
        .unwrap(),
    )
    .unwrap()
}
pub(super) fn future_before_old(path: &Path, store: &BlobStore, previous: SaveTicket) -> BlobRef {
    let mut writer = save::load_driver(path, store, 91);
    writer.ids = Ids(4000);
    writer.nav(HistoryAction::Checkout {
        node: NodeId::new(3).unwrap(),
    });
    let raw = store.stage_bytes(b"opaque\0ORIGINAL\r\n".to_vec()).unwrap();
    writer.command(vec![Command::RegisterAsset {
        asset: Asset::new(
            AssetId::new(2).unwrap(),
            "future-before-old".into(),
            raw.reference(),
            AssetKind::Opaque,
        )
        .unwrap(),
    }]);
    let (mut archive, _) = open(path, store.clone(), RestoreSelection::Manual);
    archive
        .write_capture(
            &mut writer.capture(),
            request(3001, Some(previous.commit.get()), SaveOrigin::Exit),
        )
        .unwrap();
    raw.reference()
}
#[test]
fn historical_original_queries_keep_dense_entries_provenance_and_the_old_full_golden() {
    let (dir, req) = copy_golden_file();
    let before = std::fs::read(dir.file()).unwrap();
    let scratch = Directory::new();
    let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    let file = reader
        .inspect(&dir.file(), req, CaptureCancellation::default())
        .unwrap();
    let vectors = original_vectors();
    for id in [400, 401] {
        let request = query(&file, id);
        let result = file.historical_originals(request).unwrap();
        assert_eq!(result.version, FILE_HISTORICAL_ORIGINALS_VERSION);
        assert_eq!(result.request, request);
        assert_eq!(result.assurance, FileFactsAssurance::ExperimentalValidation);
        let facts = result.facts.unwrap();
        assert_eq!(facts.commit.fact.commit, request.commit);
        assert_eq!(facts.proof_version, FILE_COORD_PROOF_VERSION);
        assert_eq!((facts.len, facts.bytes), (1, 17));
        assert_eq!(hex(&facts.digest), vectors["cases"][2]["sha256"]);
        assert_eq!(
            facts.entries[0].unwrap().sha256(),
            sha(b"opaque\0original\r\n")
        );
        assert!(facts.entries[1..].iter().all(Option::is_none));
        assert_eq!(facts.digest, file.summary().proof.originals);
    }
    let request = query(&file, 999);
    let missing = file.historical_originals(request).unwrap();
    assert_eq!(missing.request, request);
    assert_eq!(missing.facts, None);
    drop(file);
    assert_eq!(std::fs::read(dir.file()).unwrap(), before);
}
#[test]
fn real_q_keeps_its_old_raw_set_when_a_future_sha_moves_before_it() {
    let (dir, store, mut d, _port, work, mut req) = save::fixture_on(true);
    let ticket = work.ticket();
    let before = d.session.snapshot();
    let scratch = Directory::new();
    let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    let q = reader
        .prepare_save(work, req.selection, &store, CaptureCancellation::default())
        .unwrap();
    save::committed(&dir.file(), &store, ticket);
    let future = future_before_old(&dir.file(), &store, ticket);
    let old = BlobRef::new(sha(b"opaque\0original\r\n"), 17);
    assert!(future.sha256() < old.sha256());
    req.binding.expected = ExpectedFileCommit {
        commit: Some(CommitId::new(3001).unwrap()),
        generation: Some(4),
    };
    let file = reader
        .inspect(&dir.file(), req, CaptureCancellation::default())
        .unwrap();
    let result = file
        .compare_historical_originals(&q, context(&file, &q, ticket.commit.get()))
        .unwrap();
    assert_eq!(result.originals_equal, Some(true));
    assert_eq!(result.intent.ticket, ticket);
    assert_eq!(result.intent.ticket.target.expected, ticket.target.expected);
    assert_eq!(
        result.historical.request.file.binding.expected,
        req.binding.expected
    );
    assert_ne!(ticket.target.expected, req.binding.expected);
    let facts = result.historical.facts.unwrap();
    assert_eq!(facts.entries[0], Some(old));
    assert_eq!(facts.len, 1);
    assert!(facts.entries[1..].iter().all(Option::is_none));
    let latest = file
        .compare_historical_originals(&q, context(&file, &q, 3001))
        .unwrap();
    assert_eq!(latest.originals_equal, Some(false));
    let facts = latest.historical.facts.unwrap();
    assert_eq!((facts.len, facts.bytes), (2, 34));
    assert_eq!(facts.entries[..2], [Some(future), Some(old)]);
    assert_eq!(facts.digest, file.summary().proof.originals);
    let other = file
        .compare_historical_originals(&q, context(&file, &q, 400))
        .unwrap();
    assert_eq!(other.originals_equal, Some(true));
    assert_ne!(other.historical.request.commit, ticket.commit);
    assert_eq!(
        file.compare_historical_content(
            &q,
            FileHistoricalContentCompareContext {
                request: FileHistoricalContentRequest {
                    version: FILE_HISTORICAL_CONTENT_VERSION,
                    observation: file.summary().observation,
                    file: req,
                    commit: CommitId::new(400).unwrap()
                },
                ticket
            }
        )
        .unwrap()
        .content_equal,
        Some(false)
    );
    for value in [result, latest, other] {
        assert_eq!(
            value.conclusion,
            FileCaptureConclusion::InsufficientForSaveReconciliation
        );
    }
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
    drop(file);
    drop(q);
    assert_eq!(
        d.session.acknowledge_save(ticket),
        Ok(SaveAcknowledgement::Released)
    );
}
#[test]
fn real_work_and_valid_file_with_same_counts_and_lengths_but_different_raw_do_not_match() {
    let (dir, _old_req) = copy_golden_file();
    let (other, _e, req) = golden_bytes("Unknown:雪!\r\n", b"opaque\0ORIGINAL\r\n");
    let store = BlobStore::default();
    let loaded = save::load_driver(&other.file(), &store, 88);
    let (mut d, _port, work, req) = save::attach_work(loaded, req);
    let ticket = work.ticket();
    let scratch = Directory::new();
    let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    let q = reader
        .prepare_save(work, req.selection, &store, CaptureCancellation::default())
        .unwrap();
    let file = reader
        .inspect(&dir.file(), req, CaptureCancellation::default())
        .unwrap();
    let value = file
        .compare_historical_originals(&q, context(&file, &q, 401))
        .unwrap();
    let facts = value.historical.facts.unwrap();
    assert_eq!(
        (facts.len, facts.bytes),
        (
            q.summary().capture.originals,
            q.summary().capture.original_bytes
        )
    );
    assert_eq!((facts.len, facts.bytes), (1, 17));
    let vectors = original_vectors();
    assert_eq!(hex(&facts.digest), vectors["cases"][2]["sha256"]);
    assert_eq!(
        hex(&q.summary().capture.projection.originals),
        vectors["cases"][3]["sha256"]
    );
    assert_eq!(value.originals_equal, Some(false));
    assert_eq!(value.intent.ticket, ticket);
    assert_eq!(
        value.conclusion,
        FileCaptureConclusion::InsufficientForSaveReconciliation
    );
    drop(file);
    drop(q);
    assert_eq!(
        d.session.acknowledge_save(ticket),
        Ok(SaveAcknowledgement::Released)
    );
    assert!(dir.file().exists());
}
#[test]
fn complete_historical_original_context_and_real_ticket_are_required_even_for_missing_commits() {
    let (dir, store, _d, _port, work, req) = save::fixture_on(true);
    let scratch = Directory::new();
    let a = Directory::new();
    let b = Directory::new();
    std::fs::copy(dir.file(), a.file()).unwrap();
    std::fs::copy(dir.file(), b.file()).unwrap();
    let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    let q = reader
        .prepare_save(work, req.selection, &store, CaptureCancellation::default())
        .unwrap();
    let file = reader
        .inspect(&a.file(), req, CaptureCancellation::default())
        .unwrap();
    let request = query(&file, 401);
    let mut variants = Vec::new();
    let mut bad = request;
    bad.version += 1;
    assert_eq!(
        file.historical_originals(bad),
        Err(FileFactsError::UnsupportedVersion)
    );
    bad = request;
    bad.file.version += 1;
    assert_eq!(
        file.historical_originals(bad),
        Err(FileFactsError::UnsupportedVersion)
    );
    bad = request;
    bad.observation.sequence += 1;
    variants.push(bad);
    bad = request;
    bad.observation.instance += 1;
    variants.push(bad);
    bad = request;
    bad.file.key.sequence += 1;
    variants.push(bad);
    bad = request;
    bad.file.key.session = HistorySessionId::new(999).unwrap();
    variants.push(bad);
    bad = request;
    bad.file.binding.version += 1;
    variants.push(bad);
    bad = request;
    bad.file.binding.key.generation += 1;
    variants.push(bad);
    bad = request;
    bad.file.binding.target = SaveTargetId::new(999).unwrap();
    variants.push(bad);
    bad = request;
    bad.file.binding.project = ProjectId::new(999).unwrap();
    variants.push(bad);
    bad = request;
    bad.file.binding.history = None;
    variants.push(bad);
    bad = request;
    bad.file.binding.expected.generation = Some(1);
    variants.push(bad);
    bad = request;
    bad.file.binding.expected.commit = Some(CommitId::new(400).unwrap());
    variants.push(bad);
    bad = request;
    bad.file.selection = RestoreSelection::Latest;
    variants.push(bad);
    for bad in variants {
        assert_eq!(
            file.historical_originals(bad),
            Err(FileFactsError::StalePage)
        );
    }
    let mut missing = context(&file, &q, 999);
    let none = file.compare_historical_originals(&q, missing).unwrap();
    assert_eq!(none.historical.facts, None);
    assert_eq!(none.originals_equal, None);
    missing.ticket.commit = CommitId::new(999).unwrap();
    assert_eq!(
        file.compare_historical_originals(&q, missing),
        Err(FileFactsError::StalePage)
    );
    let newer = reader
        .inspect(&b.file(), req, CaptureCancellation::default())
        .unwrap();
    assert_eq!(
        newer.historical_originals(request),
        Err(FileFactsError::StalePage)
    );
    assert!(file.historical_originals(request).unwrap().facts.is_some());
    let other_root = Directory::new();
    let mut other = FileFactsReader::open(&other_root.path, FileFactsLimits::default()).unwrap();
    let unrelated = other
        .inspect(&dir.file(), req, CaptureCancellation::default())
        .unwrap();
    assert_eq!(
        unrelated.compare_historical_originals(&q, context(&unrelated, &q, 999)),
        Err(FileFactsError::StalePage)
    );
}
#[test]
fn cancellation_after_an_actual_original_summary_keeps_old_results_pages_and_real_owners() {
    let (dir, store, mut d, _port, work, req) = save::fixture_on(true);
    let copy = Directory::new();
    std::fs::copy(dir.file(), copy.file()).unwrap();
    let scratch = Directory::new();
    let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    let q = reader
        .prepare_save(work, req.selection, &store, CaptureCancellation::default())
        .unwrap();
    let ticket = q.summary().ticket;
    let file = reader
        .inspect(&dir.file(), req, CaptureCancellation::default())
        .unwrap();
    let original = file.historical_originals(query(&file, 400)).unwrap();
    let old_page = file.commits(page(file.summary(), 0, 1)).unwrap();
    let bytes = std::fs::read(copy.file()).unwrap();
    let before = reader.usage().managed_bytes;
    support::probe(
        &mut reader,
        Probe {
            cancel_at: Some(Stage::ReadOriginalSummary),
            ..Probe::default()
        },
    );
    let cancel = CaptureCancellation::default();
    assert_eq!(
        reader.inspect(&copy.file(), req, cancel.clone()).err(),
        Some(FileFactsError::Cancelled)
    );
    assert!(cancel.is_cancelled());
    assert_eq!(reader.usage().managed_bytes, before);
    assert_eq!(reader.usage().scratch_reserved_bytes, 0);
    assert_eq!(
        file.historical_originals(original.request).unwrap(),
        original
    );
    assert_eq!(file.commits(page(file.summary(), 0, 1)).unwrap(), old_page);
    assert_eq!(
        file.compare_historical_originals(&q, context(&file, &q, 401))
            .unwrap()
            .originals_equal,
        Some(true)
    );
    assert_eq!(std::fs::read(copy.file()).unwrap(), bytes);
    assert_eq!(q.summary().ticket, ticket);
    let counter = support::control_observation(&reader);
    drop(reader);
    assert!(counter.usage().bytes > 0);
    drop(file);
    assert!(counter.usage().bytes > 0);
    assert_eq!(
        d.session.acknowledge_save(ticket),
        Ok(SaveAcknowledgement::WaitingResources)
    );
    drop(q);
    assert_eq!(counter.usage().bytes, 0);
    assert_eq!(
        d.session.acknowledge_save(ticket),
        Ok(SaveAcknowledgement::Released)
    );
}
#[test]
fn new_original_layout_has_exact_inspect_peak_and_queries_keep_retained_usage() {
    let (dir, req) = copy_golden_file();
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
            assert_eq!(
                result.err(),
                Some(FileFactsError::Limit(FileFactsLimit::Control))
            );
        } else {
            let file = result.unwrap();
            assert_eq!(reader.usage().managed_peak_bytes, required);
            assert_eq!(reader.usage().managed_bytes, support::controls());
            let held = reader.usage().managed_bytes;
            for id in [400, 401, 999] {
                let result = file.historical_originals(query(&file, id)).unwrap();
                assert_eq!(result.facts.is_some(), id != 999);
            }
            assert_eq!(reader.usage().managed_bytes, held);
            drop(file);
        }
        assert_eq!(reader.usage().managed_bytes, before);
        assert_eq!(reader.usage().scratch_reserved_bytes, 0);
    }
}
