//! A3 uses the same real capture and independently checked synthetic file
//! fixtures as A2. No filesystem, host shutdown, clock, or native device.
use super::*;

fn close_request(d: &Driver) -> CloseRequest {
    CloseRequest {
        api_version: API_VERSION,
        key: d.session.close_status(1).unwrap().next_key.unwrap(),
        source: d.session.save_source(),
        purpose: ClosePurpose::Exit,
    }
}

fn active_close(d: &Driver) -> CloseActiveStatus {
    d.session.close_status(1).unwrap().active.unwrap()
}

fn start_close(d: &mut Driver, choice: CloseChoice) -> CloseTicket {
    let ticket = d.session.request_close(close_request(d)).unwrap().ticket();
    d.session.choose_close(ticket, choice).unwrap();
    ticket
}

fn reach(d: &mut Driver, ticket: CloseTicket, phase: ClosePhase) {
    for _ in 0..8 {
        let active = active_close(d);
        assert_eq!(active.ticket, ticket);
        if active.phase == phase {
            return;
        }
        d.session.advance_close(ticket, &mut d.ids).unwrap();
    }
    assert_eq!(active_close(d).phase, phase);
}

fn command_request(d: &mut Driver, text: &str) -> CommandRequest {
    let info = d.session.project_info();
    let command = if d
        .session
        .snapshot()
        .project
        .configuration()
        .records()
        .is_empty()
    {
        Command::InsertConfigRecord {
            index: 0,
            record: record(text),
        }
    } else {
        Command::SetConfigRecord {
            index: 0,
            record: record(text),
        }
    };
    let request = CommandRequest {
        api_version: API_VERSION,
        session_id: info.session_id,
        request_id: RequestId::new(d.next).unwrap(),
        expected: info.state,
        commands: vec![command],
    };
    d.next += 1;
    request
}

fn draft_request(d: &mut Driver, action: DraftAction) -> DraftRequest {
    let request = DraftRequest {
        api_version: API_VERSION,
        session_id: d.session.project_info().session_id,
        request_id: RequestId::new(d.next).unwrap(),
        action,
    };
    d.next += 1;
    request
}

fn draft_state(d: &Driver) -> DraftState {
    d.session
        .draft_info(d.session.project_info().session_id)
        .unwrap()
        .unwrap()
        .state
}

fn begin(d: &mut Driver) -> DraftState {
    let expected = d.session.project_info().state;
    let request = draft_request(d, DraftAction::Begin { expected });
    d.session.edit_draft(request).unwrap();
    draft_state(d)
}

fn replace(d: &mut Driver, text: &str) -> DraftState {
    let expected = draft_state(d);
    let commands = command_request(d, text).commands;
    let request = draft_request(d, DraftAction::Replace { expected, commands });
    d.session.edit_draft(request).unwrap();
    draft_state(d)
}

fn remember(d: &mut Driver) {
    let snapshot = d.session.snapshot();
    d.expected
        .insert(snapshot.state.head.get(), snapshot.project);
}

fn confirm_child(d: &mut Driver, port: &SaveProtocolPort, ticket: SaveTicket) -> SaveCommitFact {
    let golden = d.golden();
    let mut work = d.session.take_save_execution(port, ticket).unwrap();
    let witness = prepare(&mut work, &golden, &d.pool).unwrap();
    let receipt = fact(&work);
    let sender = work.take_completion().unwrap();
    work.begin_write().unwrap();
    sender.publish(committed(witness, receipt)).unwrap();
    drop(work);
    drop(sender);
    receipt
}

fn external_capture(d: &mut Driver) -> Result<HistoryCapture, CaptureError> {
    let info = d.session.project_info();
    d.session.capture_history(
        CaptureRequest {
            version: HISTORY_CAPTURE_VERSION,
            session_id: info.session_id,
            expected: info.state,
            limits: save_fixture_capture_limits(),
            cancellation: CaptureCancellation::default(),
        },
        &mut d.ids,
    )
}

#[test]
fn close_gate_precedes_cached_command_navigation_and_draft_success() {
    let mut d = Driver::new();
    let _port = d.install(SaveCandidateLimits::default(), false);
    let command = command_request(&mut d, "committed");
    d.session.submit(command.clone()).unwrap();
    remember(&mut d);
    let info = d.session.project_info();
    let navigation = HistoryRequest {
        api_version: API_VERSION,
        session_id: info.session_id,
        request_id: RequestId::new(d.next).unwrap(),
        expected: info.state,
        action: HistoryAction::Undo,
    };
    d.next += 1;
    d.session.navigate(navigation.clone()).unwrap();
    begin(&mut d);
    let expected = replace(&mut d, "draft");
    let commit = draft_request(&mut d, DraftAction::Commit { expected });
    d.session.edit_draft(commit.clone()).unwrap();
    remember(&mut d);
    let validated = command_request(&mut d, "validated-before-close");
    assert!(d.session.validate(&validated).is_ok());
    let close = d.session.request_close(close_request(&d)).unwrap().ticket();
    let before = d.session.snapshot();
    let receipts = d.session.receipts.len();
    let ids = d.ids.0;
    assert_eq!(d.session.submit(command), Err(CommandError::BlockedByClose));
    assert_eq!(
        d.session.submit(validated),
        Err(CommandError::BlockedByClose)
    );
    assert_eq!(
        d.session.navigate(navigation),
        Err(HistoryError::BlockedByClose)
    );
    assert_eq!(
        d.session.edit_draft(commit),
        Err(DraftError::Request(CommandError::BlockedByClose))
    );
    let expected = before.state;
    let begin_request = draft_request(&mut d, DraftAction::Begin { expected });
    assert_eq!(
        d.session.edit_draft(begin_request),
        Err(DraftError::Request(CommandError::BlockedByClose))
    );
    assert!(matches!(
        external_capture(&mut d),
        Err(CaptureError::Closing)
    ));
    let save = request(&d.session);
    assert_eq!(
        d.session.request_save(save, &mut d.ids),
        Err(SaveCoordinationError::BlockedByClose)
    );
    for _ in 0..32 {
        assert_eq!(active_close(&d).phase, ClosePhase::AwaitChoice);
        assert_eq!(d.session.snapshot(), before);
    }
    assert_eq!(d.session.receipts.len(), receipts);
    assert_eq!(d.ids.0, ids);
    assert_eq!(
        d.session.acknowledge_close(close),
        Err(SaveCoordinationError::NotTerminal)
    );
    d.session.cancel_close(close).unwrap();
    d.session.acknowledge_close(close).unwrap();
    d.set("editing-resumed");
}

#[test]
fn exact_close_and_save_repeats_do_not_recapture_or_change_choice() {
    let mut d = Driver::new();
    d.set("q");
    let _port = d.install(SaveCandidateLimits::default(), false);
    let save_request = request(&d.session);
    let save = d
        .session
        .request_save(save_request, &mut d.ids)
        .unwrap()
        .ticket();
    let close_request = close_request(&d);
    let close = d.session.request_close(close_request).unwrap().ticket();
    let ids = d.ids.0;
    assert_eq!(
        d.session.request_close(close_request),
        Ok(CloseAdmission::Existing(close))
    );
    let changed = CloseRequest {
        purpose: ClosePurpose::Open,
        ..close_request
    };
    assert_eq!(
        d.session.request_close(changed),
        Err(SaveCoordinationError::Conflict)
    );
    assert_eq!(
        d.session.request_save(save_request, &mut d.ids),
        Ok(SaveAdmission::Existing(save))
    );
    assert!(matches!(
        external_capture(&mut d),
        Err(CaptureError::Closing)
    ));
    d.session.choose_close(close, CloseChoice::Exit).unwrap();
    d.session.choose_close(close, CloseChoice::Exit).unwrap();
    assert_eq!(
        d.session.choose_close(close, CloseChoice::Manual),
        Err(SaveCoordinationError::Conflict)
    );
    assert_eq!(d.ids.0, ids);
    d.session.cancel_close(close).unwrap();
    d.session.acknowledge_close(close).unwrap();
    assert_eq!(
        d.session.request_close(close_request),
        Err(SaveCoordinationError::Stale)
    );
    assert_eq!(
        d.session.cancel_close(close),
        Err(SaveCoordinationError::Stale)
    );
    assert_eq!(d.active().ticket, save);
    assert!(!d.active().acknowledged);
}

#[test]
fn old_draft_commit_and_cancel_receipts_cannot_consume_a_new_close_permit() {
    let mut d = Driver::new();
    let _port = d.install(SaveCandidateLimits::default(), false);
    let expected = begin(&mut d);
    let old_commit = draft_request(&mut d, DraftAction::Commit { expected });
    d.session.edit_draft(old_commit.clone()).unwrap();
    let expected = begin(&mut d);
    let old_cancel = draft_request(&mut d, DraftAction::Cancel { expected });
    d.session.edit_draft(old_cancel.clone()).unwrap();
    begin(&mut d);
    let current = replace(&mut d, "pending");
    let close_request = close_request(&d);
    let close = d.session.request_close(close_request).unwrap().ticket();
    assert_eq!(active_close(&d).phase, ClosePhase::AwaitChoice);
    let permit = active_close(&d).finish.unwrap();
    let before = d.session.snapshot();
    let receipts = d.session.receipts.len();
    assert!(d.session.finish_close_draft(permit, old_commit).is_err());
    assert!(
        d.session
            .finish_close_draft(permit, old_cancel.clone())
            .is_err()
    );
    let _ = d.session.edit_draft(old_cancel);
    assert_eq!(draft_state(&d), current);
    assert_eq!(active_close(&d).finish, Some(permit));
    assert_eq!(d.session.snapshot(), before);
    assert_eq!(d.session.receipts.len(), receipts);
    let ordinary = draft_request(&mut d, DraftAction::Commit { expected: current });
    assert_eq!(
        d.session.edit_draft(ordinary),
        Err(DraftError::Request(CommandError::BlockedByClose))
    );
    let ordinary = draft_request(
        &mut d,
        DraftAction::Replace {
            expected: current,
            commands: Vec::new(),
        },
    );
    assert_eq!(
        d.session.edit_draft(ordinary),
        Err(DraftError::Request(CommandError::BlockedByClose))
    );
    assert!(matches!(
        external_capture(&mut d),
        Err(CaptureError::Closing)
    ));
    let cancel = draft_request(&mut d, DraftAction::Cancel { expected: current });
    assert_eq!(
        d.session.edit_draft(cancel).unwrap().outcome,
        DraftOutcome::Cancelled(current)
    );
    assert!(active_close(&d).finish.is_none());
    assert!(
        d.session
            .draft_info(d.session.project_info().session_id)
            .unwrap()
            .is_none()
    );
    assert_eq!(d.session.snapshot(), before);
    assert_eq!(active_close(&d).phase, ClosePhase::AwaitChoice);
    assert!(active_close(&d).choice.is_none());
    assert_eq!(
        d.session.request_close(close_request),
        Ok(CloseAdmission::Existing(close))
    );
}

#[test]
fn failed_close_commit_keeps_permit_and_the_last_correct_cancel_receipt() {
    let mut d = Driver::new();
    d.set("completed-before-draft");
    let _port = d.install(SaveCandidateLimits::default(), false);
    begin(&mut d);
    let expected = replace(&mut d, "unpublished-draft");
    let close = start_close(&mut d, CloseChoice::Exit);
    let permit = active_close(&d).finish.unwrap();
    let before = d.session.snapshot();
    let nodes = d.session.history_len();
    d.session.limits.history_nodes = nodes;
    d.session.limits.retained_requests = d.session.receipts.len() + 2;
    let commit = draft_request(&mut d, DraftAction::Commit { expected });
    assert_eq!(
        d.session.finish_close_draft(permit, commit.clone()),
        Err(DraftError::Request(CommandError::ResourceLimit(
            "history_nodes"
        )))
    );
    let receipts = d.session.receipts.len();
    assert_eq!(d.session.snapshot(), before);
    assert_eq!(draft_state(&d), expected);
    assert_eq!(active_close(&d).finish, Some(permit));
    assert!(d.session.finish_close_draft(permit, commit).is_err());
    assert_eq!(d.session.receipts.len(), receipts);
    let cancel = draft_request(&mut d, DraftAction::Cancel { expected });
    assert_eq!(
        d.session.edit_draft(cancel).unwrap().outcome,
        DraftOutcome::Cancelled(expected)
    );
    assert_eq!(d.session.receipts.len(), d.session.limits.retained_requests);
    assert_eq!(d.session.history_len(), nodes);
    assert!(active_close(&d).finish.is_none());
    reach(&mut d, close, ClosePhase::Protecting);
    assert_eq!(
        active_close(&d).child.unwrap().source,
        d.session.save_source()
    );
    d.session.cancel_close(close).unwrap();
    d.session.acknowledge_close(close).unwrap();
    assert_eq!(d.session.snapshot(), before);
}

#[test]
fn no_change_close_commit_ends_only_the_existing_draft_without_a_node() {
    let mut d = Driver::new();
    let _port = d.install(SaveCandidateLimits::default(), false);
    let expected = begin(&mut d);
    let request = close_request(&d);
    let close = d.session.request_close(request).unwrap().ticket();
    d.session.choose_close(close, CloseChoice::Exit).unwrap();
    let permit = active_close(&d).finish.unwrap();
    let before = d.session.snapshot();
    let commit = draft_request(&mut d, DraftAction::Commit { expected });
    let outcome = d
        .session
        .finish_close_draft(permit, commit.clone())
        .unwrap()
        .outcome;
    assert!(
        matches!(outcome, DraftOutcome::Committed(result) if result.status == CommandStatus::NoChange)
    );
    assert_eq!(d.session.snapshot(), before);
    assert_eq!(d.session.history_len(), 1);
    assert!(active_close(&d).finish.is_none());
    assert!(d.session.finish_close_draft(permit, commit).is_err());
    assert_eq!(
        d.session.request_close(request),
        Ok(CloseAdmission::Existing(close))
    );
    reach(&mut d, close, ClosePhase::EmptyCreationBaselineSatisfied);
    assert_eq!(
        d.session.acknowledge_close(close),
        Err(SaveCoordinationError::NotTerminal)
    );
}

#[test]
fn existing_save_is_settled_and_released_before_the_final_draft_capture() {
    let mut d = Driver::new();
    d.set("old-q");
    let port = d.install(SaveCandidateLimits::default(), false);
    let old = d.admit();
    let old_golden = d.golden();
    let mut work = d.session.take_save_execution(&port, old).unwrap();
    let snapshot = work.snapshot().clone();
    let witness = prepare(&mut work, &old_golden, &d.pool).unwrap();
    let old_fact = fact(&work);
    let sender = work.take_completion().unwrap();
    d.set("q-plus-one");
    begin(&mut d);
    let expected = replace(&mut d, "final-draft");
    let original_close = close_request(&d);
    let close = d.session.request_close(original_close).unwrap().ticket();
    d.session.choose_close(close, CloseChoice::Exit).unwrap();
    let permit = active_close(&d).finish.unwrap();
    let commit = draft_request(&mut d, DraftAction::Commit { expected });
    d.session.finish_close_draft(permit, commit).unwrap();
    remember(&mut d);
    let final_source = d.session.save_source();
    assert_ne!(final_source, original_close.source);
    let identities = d.ids.0;
    assert_eq!(
        d.session.request_close(original_close),
        Ok(CloseAdmission::Existing(close))
    );
    assert_eq!(d.ids.0, identities);
    assert_eq!(d.session.save_source(), final_source);
    work.begin_write().unwrap();
    sender.publish(committed(witness, old_fact)).unwrap();
    drop(work);
    drop(sender);
    reach(&mut d, close, ClosePhase::SettleExisting);
    // No separate advance_save_coordination or UI refresh is needed.
    d.session.advance_close(close, &mut d.ids).unwrap();
    assert_eq!(d.status().facts.files.manual, Some(old_fact));
    assert_eq!(d.active().ticket, old);
    assert!(d.active().acknowledged);
    assert!(!d.active().capture_consumers_released);
    assert!(active_close(&d).child.is_none());
    drop(snapshot);
    reach(&mut d, close, ClosePhase::Protecting);
    let child = active_close(&d).child.unwrap();
    assert_eq!(child.source, final_source);
    assert_ne!(child.source, old.source);
    assert_eq!(child.parent_close, Some(close.key));
    assert_eq!(child.origin, SaveOrigin::Exit);
    let final_fact = confirm_child(&mut d, &port, child);
    reach(&mut d, close, ClosePhase::ProtectionSatisfied);
    assert_eq!(d.status().facts.files.manual, Some(old_fact));
    assert_eq!(d.status().facts.files.protected, Some(final_fact));
    assert_eq!(
        d.session.snapshot().project.configuration().to_bytes(),
        b"final-draft"
    );
    assert_eq!(
        d.session.acknowledge_close(close),
        Err(SaveCoordinationError::NotTerminal)
    );
    let blocked = command_request(&mut d, "blocked-after-protection");
    assert_eq!(d.session.submit(blocked), Err(CommandError::BlockedByClose));
    d.session.cancel_close(close).unwrap();
    d.session.acknowledge_close(close).unwrap();
    d.set("editing-after-explicit-cancel");
}

#[test]
fn cancelling_close_does_not_cancel_an_independent_manual_save() {
    let mut d = Driver::new();
    d.set("old");
    let port = d.install(SaveCandidateLimits::default(), false);
    let old = d.admit();
    let golden = d.golden();
    let mut work = d.session.take_save_execution(&port, old).unwrap();
    let witness = prepare(&mut work, &golden, &d.pool).unwrap();
    let old_fact = fact(&work);
    let sender = work.take_completion().unwrap();
    let first = start_close(&mut d, CloseChoice::Exit);
    d.session.cancel_close(first).unwrap();
    d.session.acknowledge_close(first).unwrap();
    assert!(!work.is_cancelled());
    d.set("new");
    let second = start_close(&mut d, CloseChoice::Exit);
    work.begin_write().unwrap();
    sender.publish(committed(witness, old_fact)).unwrap();
    drop(work);
    drop(sender);
    reach(&mut d, second, ClosePhase::Protecting);
    let child = active_close(&d).child.unwrap();
    assert_ne!(child, old);
    assert_eq!(child.parent_close, Some(second.key));
    assert_eq!(d.status().facts.files.manual, Some(old_fact));
    assert_eq!(
        d.session.acknowledge_close(first),
        Err(SaveCoordinationError::Stale)
    );
    assert_eq!(active_close(&d).ticket, second);
}

#[test]
fn cancelled_close_keeps_unknown_child_and_late_fact_separate_from_new_close() {
    let mut d = Driver::new();
    d.set("first");
    let port = d.install(SaveCandidateLimits::default(), false);
    let first = start_close(&mut d, CloseChoice::Exit);
    reach(&mut d, first, ClosePhase::Protecting);
    let old = active_close(&d).child.unwrap();
    let golden = d.golden();
    let mut work = d.session.take_save_execution(&port, old).unwrap();
    let witness = prepare(&mut work, &golden, &d.pool).unwrap();
    let old_fact = fact(&work);
    let sender = work.take_completion().unwrap();
    work.begin_write().unwrap();
    drop(work);
    reach(&mut d, first, ClosePhase::Reconciling);
    assert!(d.session.retry_close(first, old, &mut d.ids).is_err());
    assert_eq!(
        d.session.acknowledge_save(old),
        Err(SaveCoordinationError::NotTerminal)
    );
    d.session.cancel_close(first).unwrap();
    d.session.acknowledge_close(first).unwrap();
    assert_eq!(d.active().ticket, old);
    assert_eq!(d.active().phase, SaveProtocolPhase::Reconciling);
    d.set("second");
    let next = close_request(&d);
    assert_eq!(
        d.session.request_close(next),
        Err(SaveCoordinationError::Busy(SaveBusy::SaveOutstanding))
    );
    assert_eq!(d.session.close_status(1).unwrap().next_key, Some(next.key));
    assert!(d.session.close_status(1).unwrap().active.is_none());
    sender.publish(committed(witness, old_fact)).unwrap();
    drop(sender);
    d.advance();
    assert_eq!(
        d.session.acknowledge_save(old),
        Ok(SaveAcknowledgement::Released)
    );
    let second = d.session.request_close(next).unwrap().ticket();
    d.session.choose_close(second, CloseChoice::Exit).unwrap();
    reach(&mut d, second, ClosePhase::Protecting);
    assert_eq!(d.status().facts.files.protected, Some(old_fact));
    let child = active_close(&d).child.unwrap();
    assert_eq!(old.parent_close, Some(first.key));
    assert_eq!(child.parent_close, Some(second.key));
    assert_ne!(child.source, old.source);
    assert_eq!(
        d.session.acknowledge_save(old),
        Err(SaveCoordinationError::Stale)
    );
    assert_eq!(
        d.session.acknowledge_close(first),
        Err(SaveCoordinationError::Stale)
    );
    assert_eq!(active_close(&d).ticket, second);
    assert_eq!(active_close(&d).child, Some(child));
}

#[test]
fn exit_capture_cursor_reads_the_frozen_tree_without_confirming_close() {
    let mut d = restored(true, RestoreSelection::Manual, 2);
    let port = d.install(SaveCandidateLimits::default(), true);
    let facts = d.status().facts;
    let close = start_close(&mut d, CloseChoice::Exit);
    reach(&mut d, close, ClosePhase::Protecting);
    let ticket = active_close(&d).child.unwrap();
    assert_eq!(ticket.parent_close, Some(close.key));
    assert_eq!(ticket.origin, SaveOrigin::Exit);

    let mut work = d.session.take_save_execution(&port, ticket).unwrap();
    let initial = work.statistics();
    assert_eq!(
        work.capture_cursor(0).err(),
        Some(SaveCoordinationError::UnsupportedVersion)
    );
    assert_eq!(work.statistics(), initial);
    let mut stale = close.key;
    stale.sequence += 1;
    assert_eq!(
        work.exit_capture_cursor(SAVE_CAPTURE_CURSOR_VERSION, stale)
            .err(),
        Some(SaveCoordinationError::Conflict)
    );
    assert_eq!(work.statistics(), initial);
    let cursor = work
        .exit_capture_cursor(SAVE_CAPTURE_CURSOR_VERSION, close.key)
        .unwrap();
    assert_eq!(cursor.ticket(), ticket);
    let snapshot = cursor.snapshot().clone();
    let checked = check_capture(cursor, &d.golden(), &d.pool).unwrap();
    assert_eq!(checked.ticket, ticket);
    assert!(snapshot.is_complete());
    assert_eq!(
        work.statistics().nodes_emitted,
        snapshot.manifest().nodes.len()
    );

    d.advance();
    assert!(!d.active().witness_retained);
    assert!(!d.active().writer_released);
    assert_eq!(d.active().outcome, None);
    assert_eq!(
        work.begin_write().err(),
        Some(SaveCoordinationError::NotPrepared)
    );
    assert_eq!(
        d.session.acknowledge_save(ticket),
        Err(SaveCoordinationError::NotTerminal)
    );
    assert_eq!(active_close(&d).phase, ClosePhase::Protecting);
    assert_eq!(d.status().facts, facts);

    d.session.cancel_close(close).unwrap();
    {
        let cursor = work
            .exit_capture_cursor(SAVE_CAPTURE_CURSOR_VERSION, close.key)
            .unwrap();
        assert!(cursor.snapshot().is_complete() && cursor.is_cancelled());
    }
    drop(work);
    d.advance();
    assert!(d.active().writer_released);
    assert!(!d.active().capture_consumers_released);
    assert_eq!(
        d.session.acknowledge_save(ticket),
        Ok(SaveAcknowledgement::WaitingResources)
    );
    drop(snapshot);
    assert_eq!(
        d.session.acknowledge_save(ticket),
        Ok(SaveAcknowledgement::Released)
    );
    d.session.acknowledge_close(close).unwrap();
}

#[test]
fn cancelling_exit_during_a_cursor_borrow_preserves_the_owner_and_node() {
    let mut d = Driver::new();
    d.set("content");
    let port = d.install(SaveCandidateLimits::default(), false);
    let close = start_close(&mut d, CloseChoice::Exit);
    reach(&mut d, close, ClosePhase::Protecting);
    let child = active_close(&d).child.unwrap();
    let mut work = d.session.take_save_execution(&port, child).unwrap();
    let mut cursor = work.capture_cursor(SAVE_CAPTURE_CURSOR_VERSION).unwrap();
    assert!(!cursor.is_cancelled());
    let node = loop {
        match cursor.poll(1).unwrap() {
            CapturePoll::Node(node) => break node,
            CapturePoll::Pending => (),
            CapturePoll::Complete => panic!("capture must include the root"),
        }
    };
    d.session.cancel_close(close).unwrap();
    let before = cursor.statistics();
    assert!(cursor.is_cancelled());
    assert_eq!(cursor.statistics(), before);
    assert!(matches!(cursor.poll(1), Err(CaptureError::Cancelled)));
    assert!(work.is_cancelled());
    assert!(!d.active().writer_released);
    drop(work);
    d.advance();
    assert_eq!(
        d.active().outcome,
        Some(SaveProtocolOutcome::NotCommitted(diagnostic(
            SaveDiagnosticCode::Cancelled
        )))
    );
    assert_eq!(
        d.session.acknowledge_save(child),
        Ok(SaveAcknowledgement::WaitingResources)
    );
    drop(node);
    assert_eq!(
        d.session.acknowledge_save(child),
        Ok(SaveAcknowledgement::Released)
    );
    d.session.acknowledge_close(close).unwrap();
}

#[test]
fn cancelled_child_keeps_save_slot_until_the_last_captured_node_is_dropped() {
    let mut d = Driver::new();
    d.set("content");
    let port = d.install(SaveCandidateLimits::default(), false);
    let close = start_close(&mut d, CloseChoice::Exit);
    reach(&mut d, close, ClosePhase::Protecting);
    let child = active_close(&d).child.unwrap();
    let mut work = d.session.take_save_execution(&port, child).unwrap();
    let node = loop {
        match work.poll(1).unwrap() {
            CapturePoll::Node(node) => break node,
            CapturePoll::Pending => (),
            CapturePoll::Complete => panic!("capture must include the root"),
        }
    };
    d.session.cancel_close(close).unwrap();
    assert!(work.is_cancelled());
    d.session.acknowledge_close(close).unwrap();
    assert!(d.session.close_status(1).unwrap().active.is_none());
    drop(work);
    d.advance();
    assert_eq!(
        d.session.acknowledge_save(child),
        Ok(SaveAcknowledgement::WaitingResources)
    );
    let next_save = request(&d.session);
    assert_eq!(
        d.session.request_save(next_save, &mut d.ids),
        Err(SaveCoordinationError::Busy(SaveBusy::SaveOutstanding))
    );
    assert!(!d.active().capture_consumers_released);
    drop(node);
    d.advance();
    assert!(d.status().active.is_none());
    assert!(d.session.request_save(next_save, &mut d.ids).is_ok());
}

#[test]
fn failed_child_has_no_automatic_retry_and_explicit_retry_waits_for_actual_release() {
    let mut d = Driver::new();
    d.set("content");
    let port = d.install(SaveCandidateLimits::default(), false);
    let close = start_close(&mut d, CloseChoice::Manual);
    reach(&mut d, close, ClosePhase::Protecting);
    let failed = active_close(&d).child.unwrap();
    let golden = d.golden();
    let mut work = d.session.take_save_execution(&port, failed).unwrap();
    assert_eq!(
        work.exit_capture_cursor(SAVE_CAPTURE_CURSOR_VERSION, close.key)
            .err(),
        Some(SaveCoordinationError::Conflict)
    );
    let snapshot = work.snapshot().clone();
    let witness = prepare(&mut work, &golden, &d.pool).unwrap();
    let sender = work.take_completion().unwrap();
    sender
        .publish(SaveStorageObservation::VerifiedNotCommitted {
            witness,
            diagnostic: diagnostic(SaveDiagnosticCode::ReadFailed),
        })
        .unwrap();
    drop(work);
    drop(sender);
    reach(&mut d, close, ClosePhase::Failed);
    let ids = d.ids.0;
    for _ in 0..8 {
        d.session.advance_close(close, &mut d.ids).unwrap();
        assert_eq!(active_close(&d).child, Some(failed));
        assert_eq!(active_close(&d).phase, ClosePhase::Failed);
    }
    assert_eq!(d.ids.0, ids);
    let mut wrong = failed;
    wrong.key.sequence += 1;
    assert!(d.session.retry_close(close, wrong, &mut d.ids).is_err());
    assert!(d.session.retry_close(close, failed, &mut d.ids).is_err());
    assert_eq!(d.ids.0, ids);
    drop(snapshot);
    assert_eq!(
        d.session.acknowledge_save(failed),
        Ok(SaveAcknowledgement::Released)
    );
    let save_key = d.session.save_coordination_status(1).unwrap().next_key;
    let close_key = d.session.close_status(1).unwrap().next_key;
    struct Unavailable(usize);
    impl HistoryIdentitySource for Unavailable {
        fn next_identity(&mut self) -> Result<u128, IdentityError> {
            self.0 += 1;
            Err(IdentityError::Unavailable)
        }
    }
    let mut unavailable = Unavailable(0);
    assert!(
        matches!(d.session.retry_close(close, failed, &mut unavailable),
        Err(SaveCoordinationError::Capture(error))
        if error.code == SaveDiagnosticCode::IdentityUnavailable)
    );
    assert_eq!(unavailable.0, 1);
    assert_eq!(active_close(&d).child, Some(failed));
    assert_eq!(active_close(&d).phase, ClosePhase::Failed);
    assert_eq!(
        d.session.save_coordination_status(1).unwrap().next_key,
        save_key
    );
    assert_eq!(d.session.close_status(1).unwrap().next_key, close_key);
    assert!(d.status().active.is_none());
    d.session.retry_close(close, failed, &mut d.ids).unwrap();
    reach(&mut d, close, ClosePhase::Protecting);
    let retry = active_close(&d).child.unwrap();
    assert_ne!(retry.key, failed.key);
    assert_ne!(retry.commit, failed.commit);
    assert_ne!(retry.capture, failed.capture);
    assert_eq!(retry.parent_close, Some(close.key));
    assert_eq!(retry.origin, SaveOrigin::Manual);
    assert_eq!(retry.source, d.session.save_source());
    let receipt = confirm_child(&mut d, &port, retry);
    reach(&mut d, close, ClosePhase::ProtectionSatisfied);
    assert_eq!(d.status().facts.files.manual, Some(receipt));
}

#[test]
fn closing_query_survives_close_cancel_and_late_main_fact_waits_for_query_drop() {
    let mut d = Driver::new();
    d.set("content");
    let port = d.install(SaveCandidateLimits::default(), false);
    let before = d.session.snapshot();
    let nodes = d.session.history_len();
    for _ in d.session.receipts.len()..d.session.limits.retained_requests {
        d.edit(Vec::new());
    }
    assert_eq!(d.session.snapshot(), before);
    assert_eq!(d.session.history_len(), nodes);
    assert_eq!(d.session.receipts.len(), d.session.limits.retained_requests);
    let next_ordinary_request = d.next;
    let close = start_close(&mut d, CloseChoice::Exit);
    reach(&mut d, close, ClosePhase::Protecting);
    let child = active_close(&d).child.unwrap();
    let golden = d.golden();
    let mut work = d.session.take_save_execution(&port, child).unwrap();
    let witness = prepare(&mut work, &golden, &d.pool).unwrap();
    let receipt = fact(&work);
    let sender = work.take_completion().unwrap();
    let reserved = test_query_allocation(&d.session).unwrap();
    work.begin_write().unwrap();
    drop(work);
    reach(&mut d, close, ClosePhase::Reconciling);
    let ticket = d.session.begin_save_reconcile(child).unwrap().ticket();
    assert_eq!(ticket.attempt, 1);
    let query = d.session.take_save_reconcile(&port, ticket).unwrap();
    assert_eq!(test_query_allocation(&d.session), Some(reserved));
    d.session.cancel_close(close).unwrap();
    d.session.acknowledge_close(close).unwrap();
    assert!(!query.is_cancelled());
    sender.publish(committed(witness, receipt)).unwrap();
    drop(sender);
    d.advance();
    assert_eq!(d.status().facts.files.protected, Some(receipt));
    assert!(query.is_cancelled());
    assert_eq!(
        d.session.acknowledge_save(child),
        Ok(SaveAcknowledgement::WaitingResources)
    );
    assert_eq!(d.active().ticket, child);
    drop(query);
    d.advance();
    assert!(d.status().active.is_none());
    assert_eq!(d.status().facts.files.protected, Some(receipt));
    assert_eq!(d.session.receipts.len(), d.session.limits.retained_requests);
    assert_eq!(d.next, next_ordinary_request);
    assert_eq!(d.session.history_len(), nodes);
}

#[test]
fn unchanged_opening_exit_confirms_final_q_without_replacing_manual_a_or_protected_c() {
    let mut d = restored(true, RestoreSelection::Manual, 2);
    let port = d.install(SaveCandidateLimits::default(), true);
    let metadata = d.session.archive_metadata.clone();
    let facts = d.status().facts.files;
    let before = d.session.snapshot();
    let close = start_close(&mut d, CloseChoice::Exit);
    reach(&mut d, close, ClosePhase::Protecting);
    let child = active_close(&d).child.unwrap();
    assert_eq!(child.origin, SaveOrigin::Exit);
    assert_eq!(child.source, d.session.save_source());
    publish_no_change(&mut d, &port, child, SaveNoChangeBasis::VerifiedOpening);
    reach(&mut d, close, ClosePhase::ProtectionSatisfied);
    assert_eq!(d.session.snapshot(), before);
    assert!(Arc::ptr_eq(&metadata, &d.session.archive_metadata));
    assert_eq!(d.status().facts.files, facts);
    assert_ne!(facts.manual.unwrap().head, facts.protected.unwrap().head);
    assert_eq!(
        d.session.acknowledge_close(close),
        Err(SaveCoordinationError::NotTerminal)
    );
    assert_eq!(
        d.session.acknowledge_save(child),
        Ok(SaveAcknowledgement::Released)
    );
    d.session.cancel_close(close).unwrap();
    d.session.acknowledge_close(close).unwrap();
    for new_branch in [false, true] {
        let mut d = restored(true, RestoreSelection::Manual, 2);
        let port = d.install(SaveCandidateLimits::default(), true);
        let facts = d.status().facts.files;
        if new_branch {
            d.set("new branch after opening A");
        } else {
            d.nav(HistoryAction::Checkout {
                node: NodeId::new(4).unwrap(),
            });
        }
        d.nav(HistoryAction::Checkout {
            node: NodeId::new(2).unwrap(),
        });
        let changed = start_close(&mut d, CloseChoice::Exit);
        reach(&mut d, changed, ClosePhase::Protecting);
        let child = active_close(&d).child.unwrap();
        let golden = d.golden();
        assert!(
            verify_no_change_file(
                &golden,
                d.file.as_ref().unwrap(),
                SaveNoChangeBasis::VerifiedOpening,
            )
            .is_err()
        );
        let mut work = d.session.take_save_execution(&port, child).unwrap();
        let witness = prepare(&mut work, &golden, &d.pool).unwrap();
        let sender = work.take_completion().unwrap();
        // Deliberately invalid adapter claim: a real new final-q witness is
        // paired with the opening basis invalidated by navigation/new history.
        sender
            .publish(SaveStorageObservation::NoChange {
                witness,
                files: facts,
                basis: SaveNoChangeBasis::VerifiedOpening,
            })
            .unwrap();
        drop(work);
        drop(sender);
        d.advance();
        assert_eq!(d.active().phase, SaveProtocolPhase::Reconciling);
        assert!(d.active().outcome.is_none());
        assert_eq!(
            d.active().diagnostic.unwrap().code,
            SaveDiagnosticCode::AdapterFault
        );
        assert_eq!(active_close(&d).phase, ClosePhase::Reconciling);
        assert_eq!(
            d.status().facts.protection,
            SaveProtectionStatus::NeedsProtection
        );
        assert_eq!(d.status().facts.files, facts);
        assert_eq!(
            d.session.acknowledge_close(changed),
            Err(SaveCoordinationError::NotTerminal)
        );
    }
}

#[test]
fn untouched_empty_exit_needs_no_fake_fact_but_first_manual_really_saves() {
    let mut d = Driver::new();
    let port = d.install(SaveCandidateLimits::default(), false);
    let ids = d.ids.0;
    let close = start_close(&mut d, CloseChoice::Exit);
    reach(&mut d, close, ClosePhase::EmptyCreationBaselineSatisfied);
    assert!(active_close(&d).child.is_none());
    assert_eq!(d.status().facts.files, SaveFileFacts::default());
    assert_eq!(d.ids.0, ids);
    assert_eq!(
        d.session.acknowledge_close(close),
        Err(SaveCoordinationError::NotTerminal)
    );
    let blocked = command_request(&mut d, "not-unlocked-by-protection");
    assert_eq!(d.session.submit(blocked), Err(CommandError::BlockedByClose));
    d.session.cancel_close(close).unwrap();
    d.session.acknowledge_close(close).unwrap();
    let manual = start_close(&mut d, CloseChoice::Manual);
    reach(&mut d, manual, ClosePhase::Protecting);
    let child = active_close(&d).child.unwrap();
    assert_eq!(child.origin, SaveOrigin::Manual);
    let fact = confirm_child(&mut d, &port, child);
    reach(&mut d, manual, ClosePhase::ProtectionSatisfied);
    assert_eq!(d.status().facts.files.manual, Some(fact));
    assert_eq!(d.session.history_len(), 1);
}

#[test]
fn editing_then_undo_to_empty_still_protects_the_complete_tree() {
    let mut d = Driver::new();
    d.set("retained-descendant");
    d.nav(HistoryAction::Undo);
    assert!(
        d.session
            .snapshot()
            .project
            .configuration()
            .records()
            .is_empty()
    );
    let port = d.install(SaveCandidateLimits::default(), false);
    let close = start_close(&mut d, CloseChoice::Exit);
    reach(&mut d, close, ClosePhase::Protecting);
    let child = active_close(&d).child.unwrap();
    let fact = confirm_child(&mut d, &port, child);
    reach(&mut d, close, ClosePhase::ProtectionSatisfied);
    assert_eq!(d.session.history_len(), 2);
    assert_eq!(d.status().facts.files.protected, Some(fact));
    assert!(d.status().facts.files.manual.is_none());
}

#[test]
fn missing_target_keeps_completed_history_open_without_a_recovery_path() {
    for choice in [CloseChoice::Manual, CloseChoice::Exit] {
        let mut d = Driver::new();
        d.set("needs-protection");
        d.session
            .install_close_control(SaveControlProfile::default())
            .unwrap();
        let close = start_close(&mut d, choice);
        let before = d.session.snapshot();
        let outcome = d.session.advance_close(close, &mut d.ids);
        let expected = if choice == CloseChoice::Manual {
            SaveCoordinationError::NeedsTarget
        } else {
            SaveCoordinationError::RecoveryUnavailable
        };
        assert!(outcome.is_ok() || outcome == Err(expected));
        assert_eq!(active_close(&d).error, Some(expected));
        assert_eq!(d.session.snapshot(), before);
        assert!(active_close(&d).child.is_none());
        assert_eq!(
            d.session.acknowledge_close(close),
            Err(SaveCoordinationError::NotTerminal)
        );
        d.session.cancel_close(close).unwrap();
        d.session.acknowledge_close(close).unwrap();
        d.set("still-editable");
    }
}

#[test]
fn close_tickets_and_permits_never_cross_runtime_sessions() {
    let mut d = Driver::new();
    let _port = d.install(SaveCandidateLimits::default(), false);
    let expected = begin(&mut d);
    let close = start_close(&mut d, CloseChoice::Exit);
    let permit = active_close(&d).finish.unwrap();
    let mut other = ProjectSession::new(7, 71).unwrap();
    other
        .install_close_control(SaveControlProfile::default())
        .unwrap();
    assert_eq!(
        other.cancel_close(close),
        Err(SaveCoordinationError::WrongSession)
    );
    assert_eq!(
        other.acknowledge_close(close),
        Err(SaveCoordinationError::WrongSession)
    );
    assert_eq!(
        other.choose_close(close, CloseChoice::Exit),
        Err(SaveCoordinationError::WrongSession)
    );
    let request = DraftRequest {
        api_version: API_VERSION,
        session_id: other.project_info().session_id,
        request_id: RequestId::new(1).unwrap(),
        action: DraftAction::Cancel { expected },
    };
    assert!(other.finish_close_draft(permit, request).is_err());
    assert!(other.close_status(1).unwrap().active.is_none());
    assert_eq!(other.history_len(), 1);
    assert_eq!(draft_state(&d), expected);
    assert_eq!(active_close(&d).finish, Some(permit));
}

#[test]
fn an_external_capture_last_owner_holds_final_capture_without_bypassing_closing() {
    let mut d = Driver::new();
    d.set("retained-tree");
    let _port = d.install(SaveCandidateLimits::default(), false);
    let mut capture = external_capture(&mut d).unwrap();
    let node = loop {
        match capture.poll(1).unwrap() {
            CapturePoll::Node(node) => break node,
            CapturePoll::Pending => (),
            CapturePoll::Complete => panic!("capture must include the root"),
        }
    };
    drop(capture);
    let close = start_close(&mut d, CloseChoice::Exit);
    let ids = d.ids.0;
    for _ in 0..4 {
        d.session.advance_close(close, &mut d.ids).unwrap();
        assert_eq!(active_close(&d).phase, ClosePhase::SettleExisting);
        assert!(active_close(&d).child.is_none());
        assert!(matches!(
            external_capture(&mut d),
            Err(CaptureError::Closing)
        ));
    }
    assert_eq!(d.ids.0, ids);
    drop(node);
    reach(&mut d, close, ClosePhase::Protecting);
    let child = active_close(&d).child.unwrap();
    assert_eq!(child.source, d.session.save_source());
    assert_eq!(child.parent_close, Some(close.key));
}

#[test]
fn exact_current_exit_protection_no_change_preserves_the_older_manual_point() {
    let mut d = Driver::new();
    d.set("manual-a");
    let port = d.install(SaveCandidateLimits::default(), false);
    let manual_ticket = d.admit();
    let manual = confirm_child(&mut d, &port, manual_ticket);
    d.advance();
    assert_eq!(
        d.session.acknowledge_save(manual_ticket),
        Ok(SaveAcknowledgement::Released)
    );
    d.set("protected-b");
    let first = start_close(&mut d, CloseChoice::Exit);
    reach(&mut d, first, ClosePhase::Protecting);
    let first_child = active_close(&d).child.unwrap();
    let protection = confirm_child(&mut d, &port, first_child);
    reach(&mut d, first, ClosePhase::ProtectionSatisfied);
    let file = d.golden();
    let metadata = d.session.archive_metadata.clone();
    assert_eq!(
        d.session.acknowledge_save(first_child),
        Ok(SaveAcknowledgement::Released)
    );
    d.session.cancel_close(first).unwrap();
    d.session.acknowledge_close(first).unwrap();

    let second = start_close(&mut d, CloseChoice::Exit);
    reach(&mut d, second, ClosePhase::Protecting);
    let child = active_close(&d).child.unwrap();
    let golden = d.golden();
    // Independent whole-file checks for CurrentProtection. The A2 helper is
    // intentionally Manual-specific, so this basis verifies its own intent.
    assert_eq!(golden.states, file.states);
    assert_eq!(golden.parents, file.parents);
    assert_eq!(golden.originals, file.originals);
    assert_eq!(golden.admissions, file.admissions);
    assert_eq!(golden.redo, file.redo);
    assert_eq!(golden.metadata, file.metadata);
    assert_eq!(golden.source, file.source);
    assert_eq!(file.metadata.protected.as_ref().unwrap().head, child.head);
    assert_ne!(manual.head, child.head);
    let mut work = d.session.take_save_execution(&port, child).unwrap();
    let witness = prepare(&mut work, &golden, &d.pool).unwrap();
    let sender = work.take_completion().unwrap();
    sender
        .publish(SaveStorageObservation::NoChange {
            witness,
            files: SaveFileFacts::from(file.metadata.as_ref()),
            basis: SaveNoChangeBasis::CurrentProtection,
        })
        .unwrap();
    drop(work);
    drop(sender);
    reach(&mut d, second, ClosePhase::ProtectionSatisfied);
    assert_eq!(d.status().facts.files.manual, Some(manual));
    assert_eq!(d.status().facts.files.protected, Some(protection));
    assert_eq!(d.session.archive_metadata.manual_marks.len(), 1);
    assert!(Arc::ptr_eq(&metadata, &d.session.archive_metadata));
    assert!(
        matches!(d.active().outcome, Some(SaveProtocolOutcome::Confirmed(c))
        if c.kind == SaveConfirmationKind::NoChange && c.confirmed_commit.is_none())
    );
}

#[test]
fn another_protected_head_cannot_claim_current_exit_coverage_for_a_new_head() {
    let mut d = Driver::new();
    d.set("manual-and-protected-a");
    let port = d.install(SaveCandidateLimits::default(), false);
    let manual_ticket = d.admit();
    let manual = confirm_child(&mut d, &port, manual_ticket);
    d.advance();
    assert_eq!(
        d.session.acknowledge_save(manual_ticket),
        Ok(SaveAcknowledgement::Released)
    );
    let files = d.status().facts.files;
    d.set("unprotected-b");
    let close = start_close(&mut d, CloseChoice::Exit);
    reach(&mut d, close, ClosePhase::Protecting);
    let child = active_close(&d).child.unwrap();
    assert_ne!(files.protected.unwrap().head, child.head);
    let golden = d.golden();
    let mut work = d.session.take_save_execution(&port, child).unwrap();
    let witness = prepare(&mut work, &golden, &d.pool).unwrap();
    let sender = work.take_completion().unwrap();
    // Fault injection: the old file does not contain the newly captured B.
    sender
        .publish(SaveStorageObservation::NoChange {
            witness,
            files,
            basis: SaveNoChangeBasis::CurrentProtection,
        })
        .unwrap();
    drop(work);
    drop(sender);
    d.advance();
    assert_eq!(d.active().phase, SaveProtocolPhase::Reconciling);
    assert!(d.active().outcome.is_none());
    assert_eq!(
        d.active().diagnostic.unwrap().code,
        SaveDiagnosticCode::AdapterFault
    );
    assert_eq!(active_close(&d).phase, ClosePhase::Reconciling);
    assert_eq!(
        d.status().facts.protection,
        SaveProtectionStatus::NeedsProtection
    );
    assert!(
        d.status()
            .facts
            .covered
            .is_none_or(|coverage| coverage.source != child.source)
    );
    assert_eq!(d.status().facts.files, files);
    assert_eq!(d.status().facts.files.manual, Some(manual));
    assert_eq!(
        d.session.acknowledge_close(close),
        Err(SaveCoordinationError::NotTerminal)
    );
    assert!(d.session.retry_close(close, child, &mut d.ids).is_err());
}

#[test]
fn a_late_contradictory_query_retains_real_fact_but_revokes_close_satisfaction() {
    let mut d = Driver::new();
    d.set("content");
    let port = d.install(SaveCandidateLimits::default(), false);
    let close = start_close(&mut d, CloseChoice::Exit);
    reach(&mut d, close, ClosePhase::Protecting);
    let child = active_close(&d).child.unwrap();
    let golden = d.golden();
    let mut work = d.session.take_save_execution(&port, child).unwrap();
    let witness = prepare(&mut work, &golden, &d.pool).unwrap();
    let receipt = fact(&work);
    let sender = work.take_completion().unwrap();
    work.begin_write().unwrap();
    drop(work);
    reach(&mut d, close, ClosePhase::Reconciling);
    let query_ticket = d.session.begin_save_reconcile(child).unwrap().ticket();
    let query = d.session.take_save_reconcile(&port, query_ticket).unwrap();
    sender.publish(committed(witness, receipt)).unwrap();
    drop(sender);
    reach(&mut d, close, ClosePhase::ProtectionSatisfied);
    assert_eq!(d.status().facts.protection, SaveProtectionStatus::Covered);
    assert_eq!(d.status().facts.files.protected, Some(receipt));
    query
        .publish(SaveStorageObservation::VerifiedNotCommitted {
            witness,
            diagnostic: diagnostic(SaveDiagnosticCode::TargetConflict),
        })
        .unwrap();
    d.advance();
    assert!(d.status().facts.binding_conflict);
    assert_eq!(d.status().facts.files.protected, Some(receipt));
    assert_eq!(active_close(&d).phase, ClosePhase::Failed);
    assert_eq!(
        active_close(&d).error,
        Some(SaveCoordinationError::InvalidBinding)
    );
    assert_eq!(
        d.session.acknowledge_close(close),
        Err(SaveCoordinationError::NotTerminal)
    );
    let blocked = command_request(&mut d, "still-gated");
    assert_eq!(d.session.submit(blocked), Err(CommandError::BlockedByClose));
    drop(query);
    d.session.cancel_close(close).unwrap();
    d.session.acknowledge_close(close).unwrap();
    assert_eq!(d.status().facts.files.protected, Some(receipt));
}

#[test]
fn the_same_draft_needs_a_new_permit_after_cancelled_close_is_replaced() {
    let mut d = Driver::new();
    let _port = d.install(SaveCandidateLimits::default(), false);
    begin(&mut d);
    let expected = replace(&mut d, "still-pending");
    let first = start_close(&mut d, CloseChoice::Exit);
    let old_permit = active_close(&d).finish.unwrap();
    let before = d.session.snapshot();
    d.session.cancel_close(first).unwrap();
    d.session.acknowledge_close(first).unwrap();
    assert_eq!(draft_state(&d), expected);
    assert_eq!(d.session.snapshot(), before);

    let second = start_close(&mut d, CloseChoice::Exit);
    let current = active_close(&d).finish.unwrap();
    assert_ne!(current, old_permit);
    assert_eq!(current.context().draft, old_permit.context().draft);
    assert_eq!(current.context().close, second);
    let cancel = draft_request(&mut d, DraftAction::Cancel { expected });
    let receipts = d.session.receipts.len();
    assert_eq!(
        d.session.finish_close_draft(old_permit, cancel.clone()),
        Err(DraftError::Request(CommandError::BlockedByClose))
    );
    assert_eq!(d.session.receipts.len(), receipts);
    assert_eq!(draft_state(&d), expected);
    assert_eq!(active_close(&d).finish, Some(current));
    assert_eq!(
        d.session
            .finish_close_draft(current, cancel)
            .unwrap()
            .outcome,
        DraftOutcome::Cancelled(expected)
    );
    assert!(active_close(&d).finish.is_none());
    assert_eq!(d.session.snapshot(), before);
    assert_eq!(d.session.history_len(), 1);
}

#[test]
fn an_unsaved_empty_capture_reserves_identity_without_creating_work_to_protect() {
    let mut d = Driver::new();
    let _port = d.install(SaveCandidateLimits::default(), false);
    let capture = external_capture(&mut d).unwrap();
    let snapshot = capture.snapshot().unwrap();
    assert_eq!(snapshot.manifest().nodes.len(), 1);
    assert!(d.session.persistence.is_some());
    let before = d.session.snapshot();
    drop(snapshot);
    drop(capture);
    let ids = d.ids.0;
    let close = start_close(&mut d, CloseChoice::Exit);
    reach(&mut d, close, ClosePhase::EmptyCreationBaselineSatisfied);
    assert!(active_close(&d).child.is_none());
    assert_eq!(d.ids.0, ids);
    assert_eq!(d.status().facts.files, SaveFileFacts::default());
    assert_eq!(
        d.session.archive_metadata.as_ref(),
        &ArchiveMetadata::default()
    );
    assert_eq!(d.session.snapshot(), before);
    assert_eq!(
        d.session.acknowledge_close(close),
        Err(SaveCoordinationError::NotTerminal)
    );
}

#[test]
fn a_really_restored_empty_root_uses_final_no_change_instead_of_creation_baseline() {
    let mut d = Driver::new();
    let mut capture = external_capture(&mut d).unwrap();
    let snapshot = capture.snapshot().unwrap();
    let manifest = snapshot.manifest();
    let receipt = ArchiveReceipt {
        commit: CommitId::new(41).unwrap(),
        generation: 1,
        head: manifest.root,
        origin: SaveOrigin::Manual,
    };
    let metadata = Arc::new(ArchiveMetadata {
        last_commit: Some(receipt.clone()),
        manual: Some(receipt.clone()),
        protected: Some(receipt.clone()),
        manual_marks: vec![(receipt.commit, receipt.head)],
        tags: Vec::new(),
        extensions: Vec::new(),
    });
    let mut builder = RestoreBuilder::new(
        RestoreHeader {
            node_count: 1,
            project: manifest.project_id,
            identity: manifest.identity.clone(),
            root: manifest.root,
            head: manifest.head,
            next_alias: manifest.next_alias,
            next_branch: manifest.next_branch,
            admissions: manifest.admissions.clone(),
            redo_preferences: manifest.redo_preferences.clone(),
            originals: manifest.originals.clone(),
            metadata,
        },
        RestoreLimits::default(),
        CaptureCancellation::default(),
    )
    .unwrap();
    loop {
        match capture.poll(1).unwrap() {
            CapturePoll::Pending => (),
            CapturePoll::Complete => break,
            CapturePoll::Node(node) => {
                let info = node.info();
                builder
                    .push(RestoreNode {
                        id: info.id,
                        parent: info.parent,
                        alias: info.alias,
                        coordinate: info.coordinate,
                        depth: info.depth,
                        operation: info.operation.clone(),
                        origin: Arc::new(OriginalNodeRecord {
                            record_version: 2,
                            state: [1; 32],
                            action: b"synthetic empty root record".to_vec(),
                            checksum: [2; 32],
                        }),
                        project: node.project().clone(),
                    })
                    .unwrap();
            }
        }
    }
    drop(snapshot);
    drop(capture);
    d.session = builder
        .finish(&d.pool)
        .unwrap()
        .into_session(
            HistorySessionId::new(71).unwrap(),
            Box::new(d.pool.clone()),
            limits(),
            CaptureCancellation::default(),
        )
        .unwrap();
    d.ids = Ids(5000);
    d.file = Some(d.golden());
    let port = d.install(SaveCandidateLimits::default(), true);
    let before = d.session.snapshot();
    let close = start_close(&mut d, CloseChoice::Exit);
    reach(&mut d, close, ClosePhase::Protecting);
    let child = active_close(&d).child.unwrap();
    assert_eq!(child.source, d.session.save_source());
    assert_eq!(child.origin, SaveOrigin::Exit);
    publish_no_change(&mut d, &port, child, SaveNoChangeBasis::VerifiedOpening);
    reach(&mut d, close, ClosePhase::ProtectionSatisfied);
    assert_eq!(d.session.snapshot(), before);
    assert_eq!(d.session.history_len(), 1);
    assert_eq!(
        d.status().facts.files.manual,
        Some(SaveCommitFact::from(&receipt))
    );
    assert_eq!(
        d.status().facts.files.protected,
        Some(SaveCommitFact::from(&receipt))
    );
}
