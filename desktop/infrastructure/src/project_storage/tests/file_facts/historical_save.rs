use super::*;

fn context(
    file: &InspectedFileFacts,
    q: &FileSaveCandidate,
    id: u128,
) -> FileHistoricalSaveContext {
    FileHistoricalSaveContext {
        version: FILE_HISTORICAL_SAVE_VERSION,
        observation: file.summary().observation,
        file: file.summary().request,
        commit: CommitId::new(id).unwrap(),
        ticket: q.summary().ticket,
    }
}
fn check_case(value: FileHistoricalSaveComparison, name: &str) {
    let matrix: Value = serde_json::from_str(
        &std::fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/P3_2B1_2F_MATRIX.json"),
        )
        .unwrap(),
    )
    .unwrap();
    let case = matrix["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == name)
        .unwrap();
    assert_eq!(value.version, FILE_HISTORICAL_SAVE_VERSION);
    assert_eq!(value.context.ticket, value.intent.ticket);
    assert_eq!(value.assurance, FileFactsAssurance::ExperimentalValidation);
    assert_eq!(value.metadata, FileHistoricalSaveMetadata::NotCompared);
    assert_eq!(
        value.conclusion,
        FileCaptureConclusion::InsufficientForSaveReconciliation
    );
    assert_eq!(value.observed_expected, value.context.file.binding.expected);
    assert_eq!(
        value.baseline_expected_equal,
        value.intent.ticket.target.expected == value.observed_expected
    );
    if case["relation"].is_null() {
        assert_eq!(value.selected, None);
        return;
    }
    let f = value.selected.unwrap();
    assert_eq!(f.content.commit, f.originals.commit);
    assert_eq!(f.content.commit.fact.commit, value.context.commit);
    assert_eq!(f.content.proof_version, FILE_COORD_PROOF_VERSION);
    assert_eq!(f.originals.proof_version, FILE_COORD_PROOF_VERSION);
    assert_eq!(
        format!("{:?}", f.relation),
        case["relation"].as_str().unwrap()
    );
    let expected = case["equal"].as_array().unwrap();
    for (i, actual) in [
        f.content_equal,
        f.originals_equal,
        f.head_equal,
        f.navigation_equal,
        f.legacy_fields_equal,
        f.generation_equal.unwrap(),
        f.is_latest,
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(actual, expected[i].as_bool().unwrap(), "{name} column {i}");
    }
    assert_eq!(f.previous_equal, case["previous_equal"].as_bool().unwrap());
    assert_eq!(f.origin_equal, case["origin_equal"].as_bool().unwrap());
    assert_eq!(f.expected_generation, case["expected_generation"].as_u64());
}
fn complete_not_saved(d: &Driver) {
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
fn one_observation_keeps_equal_domains_separate_from_other_commit_and_missing() {
    for latest in [true, false] {
        let (dir, store, d, _port, work, req) = save::fixture_on(latest);
        let source = std::fs::read(dir.file()).unwrap();
        let before = d.session.snapshot();
        let scratch = Directory::new();
        let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
        let q = reader
            .prepare_save(work, req.selection, &store, CaptureCancellation::default())
            .unwrap();
        let file = reader
            .inspect(&dir.file(), req, CaptureCancellation::default())
            .unwrap();
        let held = reader.usage();
        let c = context(&file, &q, 401);
        let value = file.compare_historical_save(&q, c).unwrap();
        assert_eq!(value.context, c);
        assert_eq!(value.intent, q.summary());
        check_case(
            value,
            if latest {
                "latest_q_before_write_401"
            } else {
                "manual_q_before_write_401"
            },
        );
        if latest {
            check_case(
                file.compare_historical_save(&q, context(&file, &q, 400))
                    .unwrap(),
                "latest_q_before_write_400",
            );
            check_case(
                file.compare_historical_save(
                    &q,
                    context(&file, &q, q.summary().ticket.commit.get()),
                )
                .unwrap(),
                "latest_q_before_write_missing",
            );
        }
        assert_eq!(reader.usage().managed_bytes, held.managed_bytes);
        assert_eq!(reader.usage().managed_peak_bytes, held.managed_peak_bytes);
        assert_eq!(d.session.snapshot(), before);
        complete_not_saved(&d);
        drop(file);
        assert_eq!(std::fs::read(dir.file()).unwrap(), source);
    }
}

#[test]
fn actual_q_historical_correlation_survives_future_branch_raw_and_latest_change() {
    let (dir, store, mut d, _port, work, mut req) = save::fixture_on(true);
    let ticket = work.ticket();
    let before = d.session.snapshot();
    let scratch = Directory::new();
    let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    let q = reader
        .prepare_save(work, req.selection, &store, CaptureCancellation::default())
        .unwrap();
    save::committed(&dir.file(), &store, ticket);
    req.binding.expected = ExpectedFileCommit {
        commit: Some(ticket.commit),
        generation: Some(3),
    };
    let written = reader
        .inspect(&dir.file(), req, CaptureCancellation::default())
        .unwrap();
    check_case(
        written
            .compare_historical_save(&q, context(&written, &q, ticket.commit.get()))
            .unwrap(),
        "latest_q_written",
    );
    drop(written);
    let future = original_query::future_before_old(&dir.file(), &store, ticket);
    assert!(future.sha256() < sha(b"opaque\0original\r\n"));
    let source = std::fs::read(dir.file()).unwrap();
    req.binding.expected = ExpectedFileCommit {
        commit: Some(CommitId::new(3001).unwrap()),
        generation: Some(4),
    };
    let file = reader
        .inspect(&dir.file(), req, CaptureCancellation::default())
        .unwrap();
    let held = reader.usage();
    for (id, name) in [
        (ticket.commit.get(), "latest_q_after_future"),
        (3001, "latest_q_select_future"),
        (401, "latest_q_after_future_select_401"),
    ] {
        let value = file
            .compare_historical_save(&q, context(&file, &q, id))
            .unwrap();
        assert_eq!(value.intent.ticket, ticket);
        assert_eq!(value.context.file, req);
        assert_ne!(value.intent.ticket.target.expected, value.observed_expected);
        check_case(value, name);
    }
    assert_eq!(reader.usage().managed_bytes, held.managed_bytes);
    assert_eq!(reader.usage().managed_peak_bytes, held.managed_peak_bytes);
    assert_eq!(d.session.snapshot(), before);
    complete_not_saved(&d);
    drop(file);
    assert_eq!(std::fs::read(dir.file()).unwrap(), source);
    drop(q);
    assert_eq!(
        d.session.acknowledge_save(ticket),
        Ok(SaveAcknowledgement::Released)
    );
}

#[test]
fn same_ticket_commit_id_and_equal_raw_content_do_not_hide_wrong_head_and_request() {
    let (dir, store, d, _port, work, mut req) = save::fixture_on(false);
    let ticket = work.ticket();
    let scratch = Directory::new();
    let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    let q = reader
        .prepare_save(work, req.selection, &store, CaptureCancellation::default())
        .unwrap();
    save::committed(&dir.file(), &store, ticket);
    req.binding.expected = ExpectedFileCommit {
        commit: Some(ticket.commit),
        generation: Some(3),
    };
    let file = reader
        .inspect(&dir.file(), req, CaptureCancellation::default())
        .unwrap();
    let value = file
        .compare_historical_save(&q, context(&file, &q, ticket.commit.get()))
        .unwrap();
    check_case(value, "manual_q_same_id_wrong_head");
    assert_ne!(
        value.selected.unwrap().content.commit.fact.head,
        ticket.head
    );
    complete_not_saved(&d);
}

#[test]
fn same_raw_counts_and_lengths_from_real_work_do_not_make_historical_domains_equal() {
    let (old, _req) = copy_golden_file();
    let (upper, _d, req) = golden_bytes("Unknown:雪!\r\n", b"opaque\0ORIGINAL\r\n");
    let store = BlobStore::default();
    let loaded = save::load_driver(&upper.file(), &store, 88);
    let (d, _port, work, req) = save::attach_work(loaded, req);
    let scratch = Directory::new();
    let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    let q = reader
        .prepare_save(work, req.selection, &store, CaptureCancellation::default())
        .unwrap();
    let file = reader
        .inspect(&old.file(), req, CaptureCancellation::default())
        .unwrap();
    let value = file
        .compare_historical_save(&q, context(&file, &q, 401))
        .unwrap();
    check_case(value, "same_counts_different_raw");
    let f = value.selected.unwrap();
    assert_eq!((f.originals.len, f.originals.bytes), (1, 17));
    assert_eq!(
        (
            q.summary().capture.originals,
            q.summary().capture.original_bytes
        ),
        (1, 17)
    );
    complete_not_saved(&d);
}

#[test]
fn complete_context_and_original_ticket_are_guarded_even_when_selected_id_is_missing() {
    let (dir, store, _d, _port, work, req) = save::fixture_on(true);
    let a = Directory::new();
    let b = Directory::new();
    std::fs::copy(dir.file(), a.file()).unwrap();
    std::fs::copy(dir.file(), b.file()).unwrap();
    let scratch = Directory::new();
    let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    let q = reader
        .prepare_save(work, req.selection, &store, CaptureCancellation::default())
        .unwrap();
    let file = reader
        .inspect(&a.file(), req, CaptureCancellation::default())
        .unwrap();
    for id in [401, 999] {
        let c = context(&file, &q, id);
        let mut bad = c;
        bad.version += 1;
        assert_eq!(
            file.compare_historical_save(&q, bad),
            Err(FileFactsError::UnsupportedVersion)
        );
        bad = c;
        bad.file.version += 1;
        assert_eq!(
            file.compare_historical_save(&q, bad),
            Err(FileFactsError::UnsupportedVersion)
        );
        let mut cases = Vec::new();
        bad = c;
        bad.observation.sequence += 1;
        cases.push(bad);
        bad = c;
        bad.observation.instance += 1;
        cases.push(bad);
        bad = c;
        bad.file.key.sequence += 1;
        cases.push(bad);
        bad = c;
        bad.file.key.session = HistorySessionId::new(999).unwrap();
        cases.push(bad);
        bad = c;
        bad.file.binding.version += 1;
        cases.push(bad);
        bad = c;
        bad.file.binding.key.id = SaveBindingId::new(999).unwrap();
        cases.push(bad);
        bad = c;
        bad.file.binding.key.generation += 1;
        cases.push(bad);
        bad = c;
        bad.file.binding.target = SaveTargetId::new(999).unwrap();
        cases.push(bad);
        bad = c;
        bad.file.binding.project = ProjectId::new(999).unwrap();
        cases.push(bad);
        bad = c;
        bad.file.binding.phase = SaveTargetPhase::NewUnpublished;
        cases.push(bad);
        bad = c;
        bad.file.binding.history = None;
        cases.push(bad);
        bad = c;
        bad.file.binding.expected.commit = Some(CommitId::new(400).unwrap());
        cases.push(bad);
        bad = c;
        bad.file.binding.expected.generation = Some(1);
        cases.push(bad);
        bad = c;
        bad.file.selection = RestoreSelection::Latest;
        cases.push(bad);
        bad = c;
        bad.ticket.key.sequence += 1;
        cases.push(bad);
        bad = c;
        bad.ticket.capture_version += 1;
        cases.push(bad);
        bad = c;
        bad.ticket.target.expected = ExpectedFileCommit::default();
        cases.push(bad);
        bad = c;
        bad.ticket.history.address_epoch = AddressEpoch::new(999).unwrap();
        cases.push(bad);
        bad = c;
        bad.ticket.capture.0 += 1;
        cases.push(bad);
        bad = c;
        bad.ticket.coverage.0 += 1;
        cases.push(bad);
        bad = c;
        bad.ticket.head = PersistentNodeId::new(999).unwrap();
        cases.push(bad);
        bad = c;
        bad.ticket.commit = CommitId::new(999).unwrap();
        cases.push(bad);
        bad = c;
        bad.ticket.origin = SaveOrigin::Exit;
        cases.push(bad);
        for bad in cases {
            assert_eq!(
                file.compare_historical_save(&q, bad),
                Err(FileFactsError::StalePage)
            );
        }
    }
    let c = context(&file, &q, 401);
    let old = file.compare_historical_save(&q, c).unwrap();
    let newer = reader
        .inspect(&b.file(), req, CaptureCancellation::default())
        .unwrap();
    assert_eq!(
        newer.compare_historical_save(&q, c),
        Err(FileFactsError::StalePage)
    );
    assert_eq!(file.compare_historical_save(&q, c).unwrap(), old);
    let scratch2 = Directory::new();
    let mut reader2 = FileFactsReader::open(&scratch2.path, FileFactsLimits::default()).unwrap();
    let unrelated = reader2
        .inspect(&dir.file(), req, CaptureCancellation::default())
        .unwrap();
    assert_eq!(
        unrelated.compare_historical_save(&q, context(&unrelated, &q, 999)),
        Err(FileFactsError::StalePage)
    );
}

#[test]
fn canceled_or_corrupt_future_observation_preserves_old_correlation_pages_and_real_owners() {
    let (dir, store, mut d, _port, work, req) = save::fixture_on(true);
    let copy = Directory::new();
    let damaged = Directory::new();
    std::fs::copy(dir.file(), copy.file()).unwrap();
    std::fs::copy(dir.file(), damaged.file()).unwrap();
    let ticket = work.ticket();
    save::committed(&damaged.file(), &store, ticket);
    let future = original_query::future_before_old(&damaged.file(), &store, ticket);
    corrupt(
        &damaged.file(),
        "blob_chunks",
        &format!(
            "UPDATE blob_chunks SET bytes=zeroblob(length(bytes)) WHERE sha=x'{}'",
            hex(&future.sha256())
        ),
    );
    let damaged_bytes = std::fs::read(damaged.file()).unwrap();
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
    let c = context(&file, &q, 401);
    let old = file.compare_historical_save(&q, c).unwrap();
    let old_page = file.commits(page(file.summary(), 0, 1)).unwrap();
    let held = reader.usage().managed_bytes;
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
    support::probe(&mut reader, Probe::default());
    let mut latest = req;
    latest.binding.expected = ExpectedFileCommit {
        commit: Some(CommitId::new(3001).unwrap()),
        generation: Some(4),
    };
    assert_eq!(
        reader
            .inspect(&damaged.file(), latest, CaptureCancellation::default())
            .err(),
        Some(FileFactsError::ValidationFailed)
    );
    assert_eq!(reader.usage().managed_bytes, held);
    assert_eq!(reader.usage().scratch_reserved_bytes, 0);
    assert_eq!(file.compare_historical_save(&q, c).unwrap(), old);
    assert_eq!(file.commits(page(file.summary(), 0, 1)).unwrap(), old_page);
    assert_eq!(std::fs::read(copy.file()).unwrap(), source);
    assert_eq!(std::fs::read(damaged.file()).unwrap(), damaged_bytes);
    complete_not_saved(&d);
    drop(reader);
    assert_eq!(file.compare_historical_save(&q, c).unwrap(), old);
    assert!(counter.usage().bytes > 0);
    drop(file);
    assert_eq!(std::fs::read(dir.file()).unwrap(), source);
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
fn historical_correlation_keeps_the_existing_exact_aggregate_peak_without_query_growth() {
    let (retained, capture_work) = support::capture_controls();
    let (extra, context_work) = support::save_controls();
    let prepare_peak = 16 * 1024 + extra + context_work + retained + capture_work;
    let formula = prepare_peak.max(support::inspect_controls(1) + extra + retained);
    // Measure the existing managed counter first, then reopen at P / P-1.
    // This is admission bookkeeping, not an allocator/product measurement.
    let required = {
        let (dir, store, _d, _port, work, req) = save::fixture_on(true);
        let scratch = Directory::new();
        let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
        let q = reader
            .prepare_save(work, req.selection, &store, CaptureCancellation::default())
            .unwrap();
        let file = reader
            .inspect(&dir.file(), req, CaptureCancellation::default())
            .unwrap();
        let peak = reader.usage().managed_peak_bytes;
        assert_eq!(peak, formula);
        drop(file);
        drop(q);
        peak
    };
    for exact in [true, false] {
        let (dir, store, mut d, _port, work, req) = save::fixture_on(true);
        let ticket = work.ticket();
        let scratch = Directory::new();
        let mut reader = FileFactsReader::open(
            &scratch.path,
            FileFactsLimits {
                control_bytes: required - usize::from(!exact),
                ..FileFactsLimits::default()
            },
        )
        .unwrap();
        match reader.prepare_save(work, req.selection, &store, CaptureCancellation::default()) {
            Err(e) => {
                assert!(!exact);
                assert_eq!(e, FileFactsError::Limit(FileFactsLimit::Control));
            }
            Ok(q) => {
                let result = reader.inspect(&dir.file(), req, CaptureCancellation::default());
                if exact {
                    let file = result.unwrap();
                    assert_eq!(reader.usage().managed_peak_bytes, required);
                    let held = reader.usage().managed_bytes;
                    for id in [400, 401, 999] {
                        let value = file
                            .compare_historical_save(&q, context(&file, &q, id))
                            .unwrap();
                        assert_eq!(value.selected.is_some(), id != 999);
                    }
                    assert_eq!(reader.usage().managed_peak_bytes, required);
                    assert_eq!(reader.usage().managed_bytes, held);
                    drop(file);
                } else {
                    assert_eq!(
                        result.err(),
                        Some(FileFactsError::Limit(FileFactsLimit::Control))
                    );
                }
                drop(q);
            }
        }
        assert_eq!(reader.usage().managed_bytes, 16 * 1024);
        assert_eq!(reader.usage().scratch_reserved_bytes, 0);
        assert_eq!(
            d.session.acknowledge_save(ticket),
            Ok(SaveAcknowledgement::Released)
        );
    }
}
