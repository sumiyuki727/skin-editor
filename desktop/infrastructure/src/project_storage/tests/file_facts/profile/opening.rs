use super::*;

fn context(view: &InspectedFileFacts, ticket: SaveTicket) -> FileOpeningTargetContext {
    let observed = view.summary();
    FileOpeningTargetContext {
        version: FILE_OPENING_TARGET_VERSION,
        ticket,
        observation: observed.observation,
        observed_request: observed.request,
    }
}

pub(super) fn compare(
    reader: &FileFactsReader,
    view: &InspectedFileFacts,
    work: &mut SavePreparationWork,
) -> FileOpeningTargetComparison {
    let c = context(view, work.ticket());
    let cursor = work.capture_cursor(SAVE_CAPTURE_CURSOR_VERSION).unwrap();
    reader
        .compare_opening_target(&cursor, view, c, &CaptureCancellation::default())
        .unwrap()
}

#[test]
fn opening_separates_spelling_object_fields_and_content_without_polling() {
    let (dir, req) = copy_golden_file();
    let source = dir.file();
    let original = dir.closed_main_bytes();
    let scratch = Directory::new();
    let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    let store = BlobStore::default();
    let (d, opening_request) = restore(&mut reader, &dir, req, &store);
    let (mut d, _port, mut work, mut req) = save::attach_work(d, opening_request);
    let ticket = work.ticket();
    let initial = work.statistics();
    let link = dir.path.join("opening-link.db");
    let copy = dir.path.join("opening-copy.db");
    std::fs::hard_link(&source, &link).unwrap();
    std::fs::write(&copy, &original).unwrap();
    for (path, name, id) in [
        (&source, true, true),
        (&link, false, true),
        (&copy, false, false),
    ] {
        let view = reader
            .inspect(path, req, CaptureCancellation::default())
            .unwrap();
        let result = compare(&reader, &view, &mut work);
        let facts = result.facts.unwrap();
        assert_eq!(facts.origin.file, opening_request);
        assert_eq!(facts.name_digest_equal, name);
        assert_eq!(facts.file_id_fields_equal, Some(id));
        assert!(facts.source_digest_equal && facts.expected_equal && facts.selected_head_equal);
        assert_eq!(
            result.conclusion,
            FileCaptureConclusion::InsufficientForSaveReconciliation
        );
    }
    // The hard link keeps the original object alive, ruling out index reuse here.
    std::fs::rename(&source, dir.path.join("previous.db")).unwrap();
    std::fs::rename(&copy, &source).unwrap();
    let view = reader
        .inspect(&source, req, CaptureCancellation::default())
        .unwrap();
    let replacement = compare(&reader, &view, &mut work).facts.unwrap();
    assert!(
        replacement.name_digest_equal
            && replacement.source_digest_equal
            && replacement.expected_equal
    );
    assert_eq!(replacement.file_id_fields_equal, Some(false));
    drop(view);
    save::committed(&source, &store, ticket);
    req.binding.expected = ExpectedFileCommit {
        commit: Some(ticket.commit),
        generation: Some(3),
    };
    let advanced = reader
        .inspect(&source, req, CaptureCancellation::default())
        .unwrap();
    let facts = compare(&reader, &advanced, &mut work).facts.unwrap();
    assert_eq!(facts.origin.file.binding.expected, ticket.target.expected);
    assert!(facts.name_digest_equal && !facts.source_digest_equal && !facts.expected_equal);
    drop(advanced);
    // Restore the original object to the original name after an intervening replacement.
    std::fs::rename(&source, &copy).unwrap();
    std::fs::rename(dir.path.join("previous.db"), &source).unwrap();
    req.binding.expected = ticket.target.expected;
    let returned = reader
        .inspect(&source, req, CaptureCancellation::default())
        .unwrap();
    let result = compare(&reader, &returned, &mut work);
    let facts = result.facts.unwrap();
    assert!(facts.name_digest_equal && facts.source_digest_equal && facts.expected_equal);
    assert_eq!(facts.file_id_fields_equal, Some(true));
    assert_eq!(
        result.conclusion,
        FileCaptureConclusion::InsufficientForSaveReconciliation
    );
    drop(returned);
    assert_eq!(work.statistics(), initial);
    assert!(
        d.session
            .save_protocol_status(1)
            .unwrap()
            .facts
            .installed_metadata
            .is_none()
    );
    drop(work);
    assert_eq!(
        d.session.acknowledge_save(ticket),
        Ok(SaveAcknowledgement::Released)
    );
    assert_eq!(std::fs::read(&link).unwrap(), original);
}

#[test]
fn only_a_later_scoped_observation_can_compare_and_budget_releases_on_return() {
    let (dir, mut req) = copy_golden_file();
    let copy = Directory::from_main_bytes(&dir.closed_main_bytes());
    let scratch = Directory::new();
    let other_root = Directory::new();
    let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    let other = FileFactsReader::open(&other_root.path, FileFactsLimits::default()).unwrap();
    let store = BlobStore::default();
    req.key = FileFactsKey {
        session: HistorySessionId::new(88).unwrap(),
        sequence: 1,
    };
    let older = reader
        .inspect(&copy.file(), req, CaptureCancellation::default())
        .unwrap();
    let (d, req) = restore(&mut reader, &dir, req, &store);
    let (d, _port, mut work, req) = save::attach_work(d, req);
    assert!(compare(&reader, &older, &mut work).facts.is_none());
    drop(older);
    let other_selection = FileFactsRequest {
        selection: RestoreSelection::Latest,
        ..req
    };
    let latest = reader
        .inspect(&dir.file(), other_selection, CaptureCancellation::default())
        .unwrap();
    assert!(compare(&reader, &latest, &mut work).facts.is_none());
    drop(latest);
    let before = work.statistics();
    let idle_with_session = reader.usage().managed_bytes;
    let view = reader
        .inspect(&dir.file(), req, CaptureCancellation::default())
        .unwrap();
    let held = reader.usage().managed_bytes;
    let good = context(&view, work.ticket());
    {
        let cursor = work.capture_cursor(SAVE_CAPTURE_CURSOR_VERSION).unwrap();
        let run = |r: &FileFactsReader, c, token: &CaptureCancellation| {
            r.compare_opening_target(&cursor, &view, c, token)
        };
        let token = CaptureCancellation::default();
        for n in 0..5 {
            let mut bad = good;
            match n {
                0 => bad.version += 1,
                1 => bad.observation.sequence += 1,
                2 => bad.ticket.target.key.generation += 1,
                3 => bad.observed_request.binding.expected.generation = Some(99),
                _ => bad.ticket.capture.0 += 1,
            }
            assert!(run(&reader, bad, &token).is_err());
        }
        assert_eq!(run(&other, good, &token), Err(FileFactsError::StalePage));
        let cancelled = CaptureCancellation::default();
        cancelled.cancel();
        assert_eq!(
            run(&reader, good, &cancelled),
            Err(FileFactsError::Cancelled)
        );
        let short = support::hold_opening_margin(&reader, 1);
        assert_eq!(
            run(&reader, good, &token),
            Err(FileFactsError::Limit(FileFactsLimit::Control))
        );
        drop(short);
        let exact = support::hold_opening_margin(&reader, 0);
        assert!(run(&reader, good, &token).unwrap().facts.is_some());
        drop(exact);
    }
    assert_eq!(work.statistics(), before);
    assert_eq!(reader.usage().managed_bytes, held);
    drop(view);
    assert_eq!(reader.usage().managed_bytes, idle_with_session);
    drop(work);
    drop(d);
    assert_eq!(reader.usage().managed_bytes, other.usage().managed_bytes);
}
