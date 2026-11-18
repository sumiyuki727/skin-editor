use super::*;

fn registered(
    reader: &mut FileFactsReader,
    dir: &Directory,
    req: FileFactsRequest,
    store: &BlobStore,
) -> (FileTargetRegistry, Driver) {
    let mut registry = FileTargetRegistry::new(reader, FILE_TARGET_REGISTRY_VERSION).unwrap();
    let opening = registration::opening(req);
    let candidate = registry
        .bind(
            reader,
            registry.stamp(),
            &dir.file(),
            opening,
            store,
            CaptureCancellation::default(),
        )
        .unwrap();
    (registry, activate(candidate, opening.file, store))
}

fn request_for(
    registry: &FileTargetRegistry,
    work: &SavePreparationWork,
) -> FileRegisteredCoverageRequest {
    FileRegisteredCoverageRequest {
        version: FILE_REGISTERED_COVERAGE_VERSION,
        stamp: registry.stamp(),
        ticket: work.ticket(),
    }
}

fn compare(
    reader: &FileFactsReader,
    registry: &mut FileTargetRegistry,
    request: FileRegisteredCoverageRequest,
    work: &mut SavePreparationWork,
    store: &BlobStore,
) -> Result<FileRegisteredCoverage, FileFactsError> {
    reader.compare_registered_coverage(
        registry,
        request,
        work.capture_cursor(SAVE_CAPTURE_CURSOR_VERSION).unwrap(),
        store,
        CaptureCancellation::default(),
    )
}

#[test]
fn one_walk_preserves_all_checkpoint_and_history_distinctions_and_releases_actual_owners() {
    for case in [
        "linear_manual_A_exit_C",
        "branch_derived_redo",
        "linear_checkout_C_then_A",
        "new_branch_then_A",
    ] {
        let (dir, req) = if case == "branch_derived_redo" {
            let (dir, _, req) = branched_golden();
            (dir, req)
        } else {
            copy_golden_file()
        };
        let bytes = dir.closed_main_bytes();
        let scratch = Directory::new();
        let store = BlobStore::default();
        let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
        let idle = reader.usage().managed_bytes;
        let (mut registry, mut d) = registered(&mut reader, &dir, req, &store);
        match case {
            "linear_checkout_C_then_A" => {
                d.nav(HistoryAction::Checkout {
                    node: NodeId::new(4).unwrap(),
                });
                d.nav(HistoryAction::Checkout {
                    node: NodeId::new(3).unwrap(),
                });
            }
            "new_branch_then_A" => {
                let raw = store
                    .stage_bytes(b"new registered original".to_vec())
                    .unwrap();
                d.command(vec![Command::RegisterAsset {
                    asset: Asset::new(
                        AssetId::new(999).unwrap(),
                        "unused".into(),
                        raw.reference(),
                        AssetKind::Opaque,
                    )
                    .unwrap(),
                }]);
                d.nav(HistoryAction::Checkout {
                    node: NodeId::new(3).unwrap(),
                });
            }
            _ => (),
        }
        let (mut d, _port, mut work, _) = save::attach_work(d, req);
        let query = request_for(&registry, &work);
        let snapshot = work.snapshot().clone();
        let held = reader.usage().managed_bytes;
        d.set("edit made after freezing the registered capture");
        let project = d.session.snapshot();
        let saved = d.session.save_protocol_status(1).unwrap().facts;
        let result = compare(&reader, &mut registry, query, &mut work, &store).unwrap();
        let evidence = result.coverage.unwrap();
        assert_eq!(result.request, query);
        assert_eq!(evidence.context.file, evidence.origin.file);
        assert_eq!(evidence.context.observation, evidence.origin.observation);
        assert_eq!(evidence.context.file.key.sequence, 9);
        assert_eq!(evidence.context.ticket, query.ticket);
        assert_eq!(query.ticket.key.sequence, 1);
        coverage::check_case(case, evidence);
        assert_eq!(
            work.statistics().nodes_emitted,
            snapshot.manifest().nodes.len()
        );
        assert!(snapshot.is_complete());
        assert_eq!(
            compare(&reader, &mut registry, query, &mut work, &store),
            Err(FileFactsError::InvalidRequest)
        );
        let complete = work.statistics();
        d.session.cancel_save(query.ticket).unwrap();
        assert_eq!(
            compare(&reader, &mut registry, query, &mut work, &store),
            Err(FileFactsError::Cancelled)
        );
        assert_eq!(work.statistics(), complete);
        assert_eq!(reader.usage().managed_bytes, held);
        assert_eq!(d.session.save_protocol_status(1).unwrap().facts, saved);
        assert_eq!(d.session.snapshot(), project);
        assert_eq!(
            d.session.acknowledge_save(query.ticket),
            Ok(SaveAcknowledgement::WaitingResources)
        );
        drop(work);
        assert_eq!(
            d.session.acknowledge_save(query.ticket),
            Ok(SaveAcknowledgement::WaitingResources)
        );
        drop(snapshot);
        assert_eq!(
            d.session.acknowledge_save(query.ticket),
            Ok(SaveAcknowledgement::Released)
        );
        drop(d);
        assert!(std::fs::File::open(dir.file()).is_err());
        drop(registry);
        assert_eq!(dir.closed_main_bytes(), bytes);
        assert_eq!(reader.usage().managed_bytes, idle);
        assert_eq!(
            evidence.assurance,
            FileFactsAssurance::ExperimentalValidation
        );
    }
}

#[test]
fn wrong_owner_stamp_parent_or_budget_cannot_consume_the_frozen_capture() {
    let (dir, req) = copy_golden_file();
    let scratch = Directory::new();
    let other_root = Directory::new();
    let store = BlobStore::default();
    let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    let other = FileFactsReader::open(&other_root.path, FileFactsLimits::default()).unwrap();
    let (mut r, d) = registered(&mut reader, &dir, req, &store);
    let (_d, _port, mut work, _) = save::attach_work(d, req);
    let query = request_for(&r, &work);
    let foreign = FileTargetRegistry::new(&reader, 1).unwrap();
    let initial = work.statistics();
    for n in 0..4 {
        let mut bad = query;
        match n {
            0 => bad.version += 1,
            1 => bad.stamp = foreign.stamp(),
            2 => {
                bad.ticket.parent_close = Some(CloseAdmissionKey {
                    version: SAVE_COORDINATION_VERSION,
                    session: query.ticket.key.session,
                    sequence: 1,
                })
            }
            _ => bad.ticket.coverage.0 += 1,
        }
        assert!(compare(&reader, &mut r, bad, &mut work, &store).is_err());
    }
    assert_eq!(
        compare(&other, &mut r, query, &mut work, &store),
        Err(FileFactsError::StalePage)
    );
    let held = reader.usage().managed_bytes;
    let short = reader.hold_registered_coverage_margin(1);
    assert_eq!(
        compare(&reader, &mut r, query, &mut work, &store),
        Err(FileFactsError::Limit(FileFactsLimit::Control))
    );
    drop(short);
    assert_eq!(work.statistics(), initial);
    assert_eq!(reader.usage().managed_bytes, held);
    let exact = reader.hold_registered_coverage_margin(0);
    let facts = compare(&reader, &mut r, query, &mut work, &store)
        .unwrap()
        .coverage
        .unwrap();
    assert_eq!(
        facts.matches_basis(SaveNoChangeBasis::CurrentManual),
        Some(true)
    );
    drop(exact);
    assert_eq!(reader.usage().managed_bytes, held);
}

#[test]
fn rebind_partial_traversal_and_late_cancellation_preserve_their_actual_progress() {
    for case in [
        "rebind",
        "partial",
        "work cancelled",
        "late query cancellation",
    ] {
        let (dir, req) = copy_golden_file();
        let scratch = Directory::new();
        let store = BlobStore::default();
        let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
        let (mut r, d) = registered(&mut reader, &dir, req, &store);
        let (mut d, _port, mut work, _) = save::attach_work(d, req);
        let query = request_for(&r, &work);
        let held = reader.usage().managed_bytes;
        match case {
            "rebind" => {
                let (copy, _) = copy_golden_file();
                let new = r
                    .bind(
                        &mut reader,
                        query.stamp,
                        &copy.file(),
                        registration::opening(req),
                        &store,
                        CaptureCancellation::default(),
                    )
                    .unwrap();
                assert_eq!(
                    compare(&reader, &mut r, query, &mut work, &store),
                    Err(FileFactsError::StalePage)
                );
                let new_stamp = FileRegisteredCoverageRequest {
                    stamp: r.stamp(),
                    ..query
                };
                assert_eq!(
                    compare(&reader, &mut r, new_stamp, &mut work, &store),
                    Err(FileFactsError::TargetConflict)
                );
                assert_eq!(work.statistics().nodes_emitted, 0);
                drop(new);
                assert_eq!(reader.usage().managed_bytes, held);
                drop(r);
                continue;
            }
            "partial" => {
                assert!(matches!(work.poll(1).unwrap(), CapturePoll::Node(_)));
                let before = work.statistics();
                assert_eq!(
                    compare(&reader, &mut r, query, &mut work, &store),
                    Err(FileFactsError::InvalidRequest)
                );
                assert_eq!(work.statistics(), before);
            }
            "work cancelled" => {
                d.session.cancel_save(query.ticket).unwrap();
                assert_eq!(
                    compare(&reader, &mut r, query, &mut work, &store),
                    Err(FileFactsError::Cancelled)
                );
                assert_eq!(work.statistics().nodes_emitted, 0);
            }
            _ => {
                support::probe(
                    &mut reader,
                    Probe {
                        cancel_at: Some(Stage::ReadNoChangeCoverage),
                        ..Probe::default()
                    },
                );
                assert_eq!(
                    compare(&reader, &mut r, query, &mut work, &store),
                    Err(FileFactsError::Cancelled)
                );
                assert!(work.snapshot().is_complete());
                assert_eq!(
                    work.statistics().nodes_emitted,
                    work.snapshot().manifest().nodes.len()
                );
            }
        }
        assert_eq!(reader.usage().managed_bytes, held);
        assert!(
            d.session
                .save_protocol_status(1)
                .unwrap()
                .facts
                .installed_metadata
                .is_none()
        );
    }
}

pub(super) fn check_historical_collision(
    reader: &mut FileFactsReader,
    dir: &Directory,
    req: FileFactsRequest,
    store: &BlobStore,
) {
    let (mut r, mut d) = registered(reader, dir, req, store);
    d.ids = Ids(400);
    let (_d, _port, mut work, _) = save::attach_work(d, req);
    let query = request_for(&r, &work);
    assert_eq!(query.ticket.commit.get(), 401);
    let before = work.statistics();
    assert!(
        compare(reader, &mut r, query, &mut work, store)
            .unwrap()
            .coverage
            .is_none()
    );
    assert_eq!(work.statistics(), before);
}

pub(super) fn check_full_table(
    reader: &mut FileFactsReader,
    dir: &Directory,
    req: FileFactsRequest,
    store: &BlobStore,
) {
    let (mut r, d) = registered(reader, dir, req, store);
    let (_d, _port, mut work, _) = save::attach_work(d, req);
    let query = request_for(&r, &work);
    let evidence = compare(reader, &mut r, query, &mut work, store)
        .unwrap()
        .coverage
        .unwrap();
    assert_eq!(evidence.files.latest.unwrap().generation, 128);
    assert_eq!(
        evidence.matches_basis(SaveNoChangeBasis::CurrentManual),
        Some(true)
    );
}
