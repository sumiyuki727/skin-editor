use super::*;
use registered_coverage::registered;

#[test]
fn real_initial_session_matches_its_held_opening_without_consuming_a_ticket() {
    for selection in [RestoreSelection::Manual, RestoreSelection::Latest] {
        let (dir, mut req) = copy_golden_file();
        let bytes = dir.closed_main_bytes();
        req.selection = selection;
        let scratch = Directory::new();
        let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
        let store = BlobStore::default();
        let idle = reader.usage().managed_bytes;
        let (mut registry, d) = registered(&mut reader, &dir, req, &store);
        let stamp = registry.stamp();
        let held = reader.usage().managed_bytes;
        let cancel = CaptureCancellation::default();
        let opening = registration::opening(req).file;
        let before = d.session.save_coordination_status(1).unwrap();
        assert!(before.next_key.is_none());
        assert_eq!(before.reserved_control_bytes, 0);
        assert!(!d.session.save_protocol_status(1).unwrap().available);
        let pair = registry
            .preflight_session(
                &reader,
                FILE_SESSION_OPENING_VERSION,
                stamp,
                &d.session,
                &cancel,
            )
            .unwrap();
        assert_eq!(pair.file(), opening);
        assert_eq!(pair.stamp(), stamp);
        assert_eq!(pair.coverage().source, d.session.save_source());
        assert_eq!(pair.coverage().nodes, 4);
        assert_ne!(pair.observation().sequence, 0);
        assert!(reader.usage().managed_bytes > held);
        drop(pair);
        assert_eq!(reader.usage().managed_bytes, held);
        assert_eq!(d.session.save_coordination_status(1).unwrap(), before);
        assert!(!d.session.save_protocol_status(1).unwrap().available);
        let (d, _port, work, _) = save::attach_work(d, req);
        assert_eq!(work.ticket().key.sequence, 1);
        assert_eq!(
            registry
                .preflight_session(&reader, 1, stamp, &d.session, &cancel)
                .err(),
            Some(FileFactsError::Busy)
        );
        drop(work);
        drop(d);
        registry.release(stamp).unwrap();
        assert!(reader.usage().managed_bytes > idle);
        drop(registry);
        assert_eq!(reader.usage().managed_bytes, idle);
        assert_eq!(dir.closed_main_bytes(), bytes);
    }
}

#[test]
fn preflight_capacity_cancellation_and_rebinding_preserve_the_current_slot() {
    let (dir, req) = copy_golden_file();
    let independent = Directory::from_main_bytes(&dir.closed_main_bytes());
    let scratch = Directory::new();
    let foreign_root = Directory::new();
    let foreign = FileFactsReader::open(&foreign_root.path, FileFactsLimits::default()).unwrap();
    let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    let store = BlobStore::default();
    let (mut r, old) = registered(&mut reader, &dir, req, &store);
    let stamp = r.stamp();
    let held = reader.usage().managed_bytes;
    let cancel = CaptureCancellation::default();
    assert_eq!(
        r.preflight_session(&reader, 2, stamp, &old.session, &cancel)
            .err(),
        Some(FileFactsError::UnsupportedVersion)
    );
    assert_eq!(
        r.preflight_session(&foreign, 1, stamp, &old.session, &cancel)
            .err(),
        Some(FileFactsError::StalePage)
    );
    for short in [0, 1] {
        let margin = reader.hold_session_preflight_margin(short);
        let admission = r.preflight_session(&reader, 1, stamp, &old.session, &cancel);
        if short == 0 {
            assert!(admission.is_ok());
        } else {
            assert_eq!(
                admission.as_ref().err(),
                Some(&FileFactsError::Limit(FileFactsLimit::Control))
            );
        }
        drop(admission);
        drop(margin);
        assert_eq!(reader.usage().managed_bytes, held);
    }
    let early = CaptureCancellation::default();
    early.cancel();
    assert_eq!(
        r.preflight_session(&reader, 1, stamp, &old.session, &early)
            .err(),
        Some(FileFactsError::Cancelled)
    );
    support::probe(
        &mut reader,
        Probe {
            cancel_at: Some(Stage::ReadCapturedMetadata),
            ..Probe::default()
        },
    );
    assert_eq!(
        r.preflight_session(&reader, 1, stamp, &old.session, &cancel)
            .err(),
        Some(FileFactsError::Cancelled)
    );
    support::probe(&mut reader, Probe::default());
    assert_eq!(reader.usage().managed_bytes, held);
    support::cleanup(&mut reader, true);
    let cleanup = reader
        .inspect(&independent.file(), req, CaptureCancellation::default())
        .err();
    assert!(matches!(cleanup, Some(FileFactsError::Cleanup { .. })));
    assert_eq!(
        r.preflight_session(
            &reader,
            1,
            stamp,
            &old.session,
            &CaptureCancellation::default()
        )
        .err(),
        cleanup
    );
    support::cleanup(&mut reader, false);
    reader.retry_cleanup().unwrap();
    assert_eq!(reader.usage().managed_bytes, held);
    r.release(stamp).unwrap();
    let opening = registration::opening(req);
    let candidate = r
        .bind(
            &mut reader,
            r.stamp(),
            &dir.file(),
            opening,
            &store,
            CaptureCancellation::default(),
        )
        .unwrap();
    let mut new = activate(candidate, opening.file, &store);
    let latest = r.stamp();
    let cancel = CaptureCancellation::default();
    assert_eq!(
        r.preflight_session(&reader, 1, stamp, &old.session, &cancel)
            .err(),
        Some(FileFactsError::StalePage)
    );
    assert_eq!(
        r.preflight_session(&reader, 1, latest, &old.session, &cancel)
            .err(),
        Some(FileFactsError::TargetConflict)
    );
    assert!(
        r.preflight_session(&reader, 1, latest, &new.session, &cancel)
            .is_ok()
    );
    new.set("committed after opening");
    assert_eq!(
        r.preflight_session(&reader, 1, latest, &new.session, &cancel)
            .err(),
        Some(FileFactsError::TargetConflict)
    );
}

#[test]
fn draft_capture_leases_and_navigation_gate_initial_session_evidence() {
    let (dir, req) = copy_golden_file();
    let scratch = Directory::new();
    let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    let store = BlobStore::default();
    let (mut r, mut d) = registered(&mut reader, &dir, req, &store);
    let stamp = r.stamp();
    let cancel = CaptureCancellation::default();
    let capture = d.capture();
    let snapshot = capture.snapshot().unwrap();
    drop(capture);
    assert_eq!(
        r.preflight_session(&reader, 1, stamp, &d.session, &cancel)
            .err(),
        Some(FileFactsError::Busy)
    );
    drop(snapshot);
    let info = d.session.project_info();
    let before = d.session.snapshot();
    let draft = |id, action| DraftRequest {
        api_version: API_VERSION,
        session_id: info.session_id,
        request_id: RequestId::new(id).unwrap(),
        action,
    };
    d.session
        .edit_draft(draft(
            310,
            DraftAction::Begin {
                expected: info.state,
            },
        ))
        .unwrap();
    assert_eq!(
        r.preflight_session(&reader, 1, stamp, &d.session, &cancel)
            .err(),
        Some(FileFactsError::Busy)
    );
    let state = d
        .session
        .draft_info(info.session_id)
        .unwrap()
        .unwrap()
        .state;
    let record =
        ConfigRecord::new(b"draft replacement".to_vec(), ConfigInterpretation::Opaque).unwrap();
    d.session
        .edit_draft(draft(
            311,
            DraftAction::Replace {
                expected: state,
                commands: vec![Command::SetConfigRecord { index: 0, record }],
            },
        ))
        .unwrap();
    let changed = d.session.draft_info(info.session_id).unwrap().unwrap();
    assert!(changed.has_changes);
    assert_eq!(
        r.preflight_session(&reader, 1, stamp, &d.session, &cancel)
            .err(),
        Some(FileFactsError::Busy)
    );
    d.session
        .edit_draft(draft(
            312,
            DraftAction::Cancel {
                expected: changed.state,
            },
        ))
        .unwrap();
    assert_eq!(d.session.snapshot(), before);
    assert!(
        r.preflight_session(&reader, 1, stamp, &d.session, &cancel)
            .is_ok()
    );
    d.nav(HistoryAction::Checkout {
        node: NodeId::new(4).unwrap(),
    });
    d.nav(HistoryAction::Checkout {
        node: info.state.head,
    });
    assert_eq!(
        r.preflight_session(&reader, 1, stamp, &d.session, &cancel)
            .err(),
        Some(FileFactsError::TargetConflict)
    );
}
