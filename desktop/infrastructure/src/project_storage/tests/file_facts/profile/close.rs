use super::*;

#[test]
fn restored_read_only_coverage_cannot_authorize_an_exit_child() {
    let version = SAVE_COORDINATION_VERSION;
    let (dir, req) = copy_golden_file();
    let bytes = std::fs::read(dir.file()).unwrap();
    let scratch = Directory::new();
    let store = BlobStore::default();
    let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    let idle_bytes = reader.usage().managed_bytes;
    let (d, req) = restore(&mut reader, &dir, req, &store);
    let restored_bytes = reader.usage().managed_bytes;
    assert!(restored_bytes > idle_bytes);
    let (mut d, port, work, req) = save::attach_work(d, req);
    let manual = work.ticket();
    let snapshot = work.snapshot().clone();
    let metadata = snapshot.manifest().archive_metadata.clone();
    let project = d.session.snapshot();
    let protocol = d.session.save_protocol_status(version).unwrap();
    assert!(!protocol.available);
    let next_key = d
        .session
        .save_coordination_status(version)
        .unwrap()
        .next_key
        .unwrap();
    let identity = d.ids.0;

    let q = reader
        .prepare_save(work, req.selection, &store, CaptureCancellation::default())
        .unwrap();
    let view = reader
        .inspect(&dir.file(), req, CaptureCancellation::default())
        .unwrap();
    let original = context(&view, &q, manual.target.expected.commit.unwrap().get());
    let evidence = reader
        .compare_no_change_coverage(&view, &q, original, CaptureCancellation::default())
        .unwrap()
        .unwrap();
    assert_eq!(
        evidence.matches_basis(SaveNoChangeBasis::CurrentManual),
        Some(true)
    );
    assert_eq!(
        evidence.matches_basis(SaveNoChangeBasis::VerifiedOpening),
        Some(true)
    );
    assert_eq!(
        evidence.matches_basis(SaveNoChangeBasis::CurrentProtection),
        None
    );
    assert_ne!(
        evidence.files.manual.unwrap().head,
        evidence.files.protected.unwrap().head
    );
    let mut wrong_origin = original;
    wrong_origin.ticket.origin = SaveOrigin::Exit;
    assert_eq!(
        reader.compare_no_change_coverage(&view, &q, wrong_origin, CaptureCancellation::default()),
        Err(FileFactsError::StalePage)
    );
    assert_eq!(
        reader.project_fresh_append_metadata(
            &view,
            &q,
            wrong_origin,
            CaptureCancellation::default()
        ),
        Err(FileFactsError::StalePage)
    );
    assert_eq!(
        d.session.begin_save_reconcile(manual),
        Err(SaveCoordinationError::Unavailable)
    );
    drop(view);

    let close = d
        .session
        .request_close(CloseRequest {
            api_version: API_VERSION,
            key: d.session.close_status(version).unwrap().next_key.unwrap(),
            source: d.session.save_source(),
            purpose: ClosePurpose::Exit,
        })
        .unwrap()
        .ticket();
    d.session.choose_close(close, CloseChoice::Exit).unwrap();
    d.session.advance_close(close, &mut d.ids).unwrap();
    let waiting = d.session.close_status(version).unwrap().active.unwrap();
    assert_eq!(waiting.phase, ClosePhase::SettleExisting);
    assert_eq!(waiting.child, None);
    let active = d
        .session
        .save_coordination_status(version)
        .unwrap()
        .active
        .unwrap();
    assert_eq!(active.ticket, manual);
    assert_eq!(
        active.outcome,
        Some(SavePreparationOutcome::CompleteNotSaved)
    );
    assert!(active.acknowledged && !active.work_released && !active.capture_consumers_released);

    drop(q);
    d.session.advance_close(close, &mut d.ids).unwrap();
    assert_eq!(
        d.session.close_status(version).unwrap().active,
        Some(waiting)
    );
    let active = d
        .session
        .save_coordination_status(version)
        .unwrap()
        .active
        .unwrap();
    assert!(active.work_released && !active.capture_consumers_released);
    drop(snapshot);
    assert_eq!(
        d.session.advance_close(close, &mut d.ids),
        Err(SaveCoordinationError::Unavailable)
    );
    let failed = d.session.close_status(version).unwrap().active.unwrap();
    assert_eq!(failed.phase, ClosePhase::Failed);
    assert_eq!(failed.error, Some(SaveCoordinationError::Unavailable));
    assert_eq!(failed.child, None);
    d.session.advance_close(close, &mut d.ids).unwrap();
    assert_eq!(
        d.session.close_status(version).unwrap().active,
        Some(failed)
    );
    assert_eq!(
        d.session.acknowledge_close(close),
        Err(SaveCoordinationError::NotTerminal)
    );
    assert_eq!(
        d.session.acknowledge_save(manual),
        Err(SaveCoordinationError::Stale)
    );

    let edit = CommandRequest {
        api_version: API_VERSION,
        session_id: manual.key.session,
        request_id: RequestId::new(1).unwrap(),
        expected: manual.source.state,
        commands: vec![Command::SetConfigRecord {
            index: 0,
            record: ConfigRecord::new(b"after cancel".to_vec(), ConfigInterpretation::Opaque)
                .unwrap(),
        }],
    };
    assert_eq!(
        d.session.submit(edit.clone()),
        Err(CommandError::BlockedByClose)
    );
    assert_eq!(d.session.snapshot(), project);
    assert_eq!(d.session.save_protocol_status(version).unwrap(), protocol);
    let status = d.session.save_coordination_status(version).unwrap();
    assert_eq!(status.availability, SaveAvailability::PreparationOnly);
    assert_eq!(status.active, None);
    assert_eq!(status.next_key, Some(next_key));
    assert_eq!(reader.usage().managed_bytes, restored_bytes);

    d.session.cancel_close(close).unwrap();
    d.session.acknowledge_close(close).unwrap();
    let request = SaveRequest {
        api_version: API_VERSION,
        capture_version: HISTORY_CAPTURE_VERSION,
        key: next_key,
        source: manual.source,
        binding: req.binding.key,
        origin: SaveOrigin::Manual,
    };
    for (origin, error) in [
        (SaveOrigin::Exit, SaveCoordinationError::NeedsClose),
        (
            SaveOrigin::Automatic,
            SaveCoordinationError::UnsupportedOrigin,
        ),
    ] {
        assert_eq!(
            d.session
                .request_save(SaveRequest { origin, ..request }, &mut d.ids),
            Err(error)
        );
    }
    assert_eq!(d.ids.0, identity);
    let next = d
        .session
        .request_save(request, &mut d.ids)
        .unwrap()
        .ticket();
    assert_eq!(next.key, next_key);
    assert_eq!(next.parent_close, None);
    let next_work = d.session.take_save_preparation(&port, next).unwrap();
    assert_eq!(next_work.snapshot().manifest().archive_metadata, metadata);
    drop(next_work);
    assert_eq!(
        d.session.acknowledge_save(next),
        Ok(SaveAcknowledgement::Released)
    );
    assert_ne!(d.session.submit(edit).unwrap().state, manual.source.state);
    drop(d);
    assert_eq!(reader.usage().managed_bytes, idle_bytes);
    assert_eq!(std::fs::read(dir.file()).unwrap(), bytes);
}
