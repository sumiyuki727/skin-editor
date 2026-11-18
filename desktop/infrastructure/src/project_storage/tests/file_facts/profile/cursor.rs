use super::*;

#[test]
fn later_observation_cannot_replace_the_borrowed_tickets_original_expected() {
    let (dir, store, mut d, _port, mut work, mut req) = save::fixture_on(true);
    let ticket = work.ticket();
    save::committed(&dir.file(), &store, ticket);
    let bytes = std::fs::read(dir.file()).unwrap();
    req.binding.expected = ExpectedFileCommit {
        commit: Some(ticket.commit),
        generation: Some(3),
    };
    let scratch = Directory::new();
    let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    let view = reader
        .inspect(&dir.file(), req, CaptureCancellation::default())
        .unwrap();
    let context = FileSaveCompareContext {
        version: FILE_SAVE_INTENT_VERSION,
        observation: view.summary().observation,
        ticket,
    };
    let mut changed = context;
    changed.ticket.target.expected = req.binding.expected;
    let initial = work.statistics();
    let compare = |w: &mut SavePreparationWork, c| {
        reader.compare_save_cursor(
            w.capture_cursor(SAVE_CAPTURE_CURSOR_VERSION).unwrap(),
            req.selection,
            &store,
            &view,
            c,
            CaptureCancellation::default(),
        )
    };
    assert_eq!(compare(&mut work, changed), Err(FileFactsError::StalePage));
    assert_eq!(work.statistics(), initial);
    let result = compare(&mut work, context).unwrap();
    assert_eq!(result.intent.ticket, ticket);
    assert_eq!(result.observed_expected, req.binding.expected);
    assert!(result.content_equal && result.originals_equal && !result.baseline_expected_equal);
    assert_eq!(result.before_archive_equal, None);
    let commit = result.commit.unwrap();
    assert_eq!(commit.fact.commit, ticket.commit);
    assert!(
        commit.is_latest
            && commit.generation_equal
            && commit.navigation_equal
            && commit.legacy_fields_equal
    );
    assert_eq!(commit.would_commit_archive_equal, Some(true));
    assert_eq!(
        result.conclusion,
        FileCaptureConclusion::InsufficientForSaveReconciliation
    );
    drop(work);
    assert_eq!(
        d.session.acknowledge_save(ticket),
        Ok(SaveAcknowledgement::Released)
    );
    assert!(
        d.session
            .save_protocol_status(1)
            .unwrap()
            .facts
            .installed_metadata
            .is_none()
    );
    drop(view);
    assert_eq!(std::fs::read(dir.file()).unwrap(), bytes);
}

#[test]
fn borrowed_manual_capture_checks_scope_and_budget_before_consuming_the_stream() {
    let (dir, req) = copy_golden_file();
    let bytes = std::fs::read(dir.file()).unwrap();
    let scratch = Directory::new();
    let other_root = Directory::new();
    let limits = FileFactsLimits::default();
    let mut reader = FileFactsReader::open(&scratch.path, limits).unwrap();
    let other = FileFactsReader::open(&other_root.path, limits).unwrap();
    let store = BlobStore::default();
    let idle = reader.usage().managed_bytes;
    let (d, req) = restore(&mut reader, &dir, req, &store);
    let (mut d, _port, mut work, req) = save::attach_work(d, req);
    let ticket = work.ticket();
    let snapshot = work.snapshot().clone();
    let view = reader
        .inspect(&dir.file(), req, CaptureCancellation::default())
        .unwrap();
    let held = reader.usage().managed_bytes;
    let initial = work.statistics();
    let context = FileSaveCompareContext {
        version: FILE_SAVE_INTENT_VERSION,
        ticket,
        observation: view.summary().observation,
    };
    let compare =
        |reader: &FileFactsReader, work: &mut SavePreparationWork, selection, context, cancel| {
            reader.compare_save_cursor(
                work.capture_cursor(SAVE_CAPTURE_CURSOR_VERSION).unwrap(),
                selection,
                &store,
                &view,
                context,
                cancel,
            )
        };
    let mut swapped = context;
    swapped.ticket.origin = SaveOrigin::Exit;
    let mut stale = context;
    stale.observation.sequence += 1;
    for (source, selection, query) in [
        (&reader, req.selection, swapped),
        (&reader, req.selection, stale),
        (&other, req.selection, context),
        (&reader, RestoreSelection::Latest, context),
    ] {
        assert_eq!(
            compare(
                source,
                &mut work,
                selection,
                query,
                CaptureCancellation::default()
            ),
            Err(FileFactsError::StalePage)
        );
    }
    let cancelled = CaptureCancellation::default();
    cancelled.cancel();
    assert_eq!(
        compare(&reader, &mut work, req.selection, context, cancelled),
        Err(FileFactsError::Cancelled)
    );
    let short = support::hold_cursor_margin(&reader, 1);
    assert_eq!(
        compare(
            &reader,
            &mut work,
            req.selection,
            context,
            CaptureCancellation::default()
        ),
        Err(FileFactsError::Limit(FileFactsLimit::Control))
    );
    drop(short);
    assert_eq!(work.statistics(), initial);
    assert_eq!(reader.usage().managed_bytes, held);
    assert_eq!(
        d.session.acknowledge_save(ticket),
        Err(SaveCoordinationError::NotTerminal)
    );

    let exact = support::hold_cursor_margin(&reader, 0);
    let evidence = compare(
        &reader,
        &mut work,
        req.selection,
        context,
        CaptureCancellation::default(),
    )
    .unwrap();
    assert_eq!(reader.usage().managed_peak_bytes, limits.control_bytes);
    drop(exact);
    assert_eq!(reader.usage().managed_bytes, held);
    assert_eq!(evidence.intent.ticket, ticket);
    assert_eq!(evidence.observation, context.observation);
    assert!(evidence.content_equal && evidence.originals_equal && evidence.baseline_expected_equal);
    assert!(evidence.opening_navigation_equal && !evidence.latest_navigation_equal);
    assert_eq!(evidence.before_archive_equal, Some(true));
    assert_eq!(evidence.commit, None);
    assert_eq!(
        evidence.conclusion,
        FileCaptureConclusion::InsufficientForSaveReconciliation
    );
    assert!(snapshot.is_complete());
    assert_eq!(
        compare(
            &reader,
            &mut work,
            req.selection,
            context,
            CaptureCancellation::default()
        ),
        Err(FileFactsError::InvalidRequest)
    );
    assert_eq!(
        d.session.acknowledge_save(ticket),
        Ok(SaveAcknowledgement::WaitingResources)
    );
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
    assert!(!active.work_released);
    drop(work);
    assert_eq!(
        d.session.acknowledge_save(ticket),
        Ok(SaveAcknowledgement::WaitingResources)
    );
    assert!(
        d.session
            .save_coordination_status(1)
            .unwrap()
            .active
            .unwrap()
            .work_released
    );
    drop(snapshot);
    assert_eq!(
        d.session.acknowledge_save(ticket),
        Ok(SaveAcknowledgement::Released)
    );
    drop(view);
    drop(d);
    assert_eq!(reader.usage().managed_bytes, idle);
    assert_eq!(std::fs::read(dir.file()).unwrap(), bytes);
}
