//! Direct A exit combinations omitted by the component-level matrices.
//! Uses only real application/capture paths and the existing synthetic adapter.
use super::*;

fn submit(d: &mut Driver, commands: Vec<Command>) -> Result<CommandResult, CommandError> {
    let info = d.session.project_info();
    let request = CommandRequest {
        api_version: API_VERSION,
        session_id: info.session_id,
        request_id: RequestId::new(d.next).unwrap(),
        expected: info.state,
        commands,
    };
    d.next += 1;
    d.session.submit(request)
}

fn confirm_manual(d: &mut Driver, port: &SaveProtocolPort) {
    let ticket = d.admit();
    let golden = d.golden();
    let mut work = d.session.take_save_execution(port, ticket).unwrap();
    let witness = prepare(&mut work, &golden, &d.pool).unwrap();
    let receipt = fact(&work);
    let sender = work.take_completion().unwrap();
    work.begin_write().unwrap();
    sender.publish(committed(witness, receipt)).unwrap();
    drop(work);
    drop(sender);
    d.advance();
    assert_eq!(d.status().facts.protection, SaveProtectionStatus::Covered);
    assert_eq!(
        d.session.acknowledge_save(ticket),
        Ok(SaveAcknowledgement::Released)
    );
    assert!(d.status().active.is_none());
}

#[test]
fn confirmed_coverage_survives_atomic_failure_and_empty_or_net_zero_commands() {
    let mut d = Driver::new();
    d.set("saved A\r\n");
    let port = d.install(SaveCandidateLimits::default(), false);
    confirm_manual(&mut d, &port);
    let before = d.session.snapshot();
    let facts = d.status().facts;
    let source = d.session.save_source();
    let metadata = d.session.archive_metadata.clone();
    let nodes = d.session.history_len();
    let receipts = d.session.receipts.len();
    let changes = |text| Command::SetConfigRecord {
        index: 0,
        record: record(text),
    };
    assert!(
        submit(
            &mut d,
            vec![
                changes("temporary successful first step"),
                Command::SetConfigRecord {
                    index: 99,
                    record: record("invalid later step")
                },
            ]
        )
        .is_err()
    );
    assert_eq!(d.session.snapshot(), before);
    assert_eq!(d.status().facts, facts);
    assert_eq!(d.session.save_source(), source);
    assert_eq!(d.session.history_len(), nodes);

    for commands in [
        Vec::new(),
        vec![changes("temporary"), changes("saved A\r\n")],
    ] {
        assert_eq!(
            submit(&mut d, commands).unwrap().status,
            CommandStatus::NoChange
        );
        assert_eq!(d.session.snapshot(), before);
        assert_eq!(d.status().facts, facts);
        assert_eq!(d.session.save_source(), source);
        assert_eq!(d.session.history_len(), nodes);
        assert!(Arc::ptr_eq(&metadata, &d.session.archive_metadata));
        assert!(d.status().active.is_none());
    }
    // Ordinary failed/NoChange receipts are retained, but are not new content
    // or new persistent work which could invalidate the confirmed coverage.
    assert_eq!(d.session.receipts.len(), receipts + 3);
    assert_eq!(facts.protection, SaveProtectionStatus::Covered);
    assert_eq!(facts.covered, Some(facts.current));
}

#[test]
fn lower_nonzero_generation_or_old_fact_cannot_confirm_a_new_prepared_request() {
    for old_fact in [false, true] {
        let mut d = restored(true, RestoreSelection::Latest, 2);
        let port = d.install(SaveCandidateLimits::default(), true);
        let before = d.session.snapshot();
        let files = d.status().facts.files;
        let metadata = d.session.archive_metadata.clone();
        let ticket = d.admit();
        let golden = d.golden();
        let mut work = d.session.take_save_execution(&port, ticket).unwrap();
        let witness = prepare(&mut work, &golden, &d.pool).unwrap();
        let expected = fact(&work);
        assert_eq!(expected.generation, 3);
        let bad = if old_fact {
            files.latest.unwrap()
        } else {
            SaveCommitFact {
                generation: 2,
                ..expected
            }
        };
        assert_eq!(bad.generation, 2);
        let sender = work.take_completion().unwrap();
        work.begin_write().unwrap();
        sender.publish(committed(witness, bad)).unwrap();
        drop(work);
        drop(sender);
        d.advance();
        assert_eq!(d.active().phase, SaveProtocolPhase::Reconciling);
        assert!(d.active().outcome.is_none());
        assert_eq!(
            d.active().diagnostic.unwrap().code,
            SaveDiagnosticCode::AdapterFault
        );
        assert_eq!(d.status().facts.files, files);
        assert!(Arc::ptr_eq(&metadata, &d.session.archive_metadata));
        assert_eq!(
            d.session.archive_metadata.manual_marks,
            metadata.manual_marks
        );
        assert_eq!(d.session.snapshot(), before);
        assert_eq!(
            d.session.acknowledge_save(ticket),
            Err(SaveCoordinationError::NotTerminal)
        );
        assert_eq!(
            d.session.request_save(request(&d.session), &mut d.ids),
            Err(SaveCoordinationError::Busy(SaveBusy::SaveOutstanding))
        );
    }
}

#[test]
fn exact_original_byte_limit_and_one_less_have_atomic_admission_and_real_release() {
    for one_less in [false, true] {
        let mut d = resource_golden();
        let golden = d.golden();
        let bytes: u64 = golden.originals.values().map(|b| b.len() as u64).sum();
        assert!(bytes > 0 && bytes < 256 * 1024);
        let weak: Vec<_> = d.pool.0.values().map(Arc::downgrade).collect();
        let mut cfg = config(&d.session);
        cfg.capture.original_bytes = bytes - u64::from(one_less);
        let authority = synthetic_authority(&d.session, false);
        let port = d
            .session
            .install_save_protocol(cfg, SaveCandidateLimits::default(), authority)
            .unwrap();
        let request = request(&d.session);
        let before = d.session.snapshot();
        let stats = d.session.history_statistics(before.session_id).unwrap();
        let identities = d.ids.0;
        let admitted = d.session.request_save(request, &mut d.ids);
        if one_less {
            assert!(
                matches!(admitted, Err(SaveCoordinationError::Capture(error))
                if error.code == SaveDiagnosticCode::CaptureLimit)
            );
            assert_eq!(d.ids.0, identities);
            assert_eq!(
                d.session.save_coordination_status(1).unwrap().next_key,
                Some(request.key)
            );
            assert!(d.status().active.is_none());
            let mut limits = save_fixture_capture_limits();
            limits.original_bytes = bytes;
            let capture = d
                .session
                .capture_history(
                    CaptureRequest {
                        version: HISTORY_CAPTURE_VERSION,
                        session_id: before.session_id,
                        expected: before.state,
                        limits,
                        cancellation: CaptureCancellation::default(),
                    },
                    &mut d.ids,
                )
                .unwrap();
            assert_eq!(
                capture.snapshot().unwrap().manifest().originals,
                golden.originals.keys().copied().collect::<Vec<_>>()
            );
            drop(capture);
            assert_eq!(
                d.session.save_coordination_status(1).unwrap().next_key,
                Some(request.key)
            );
        } else {
            let ticket = admitted.unwrap().ticket();
            let candidate = test_candidate_weak(&d.session).unwrap();
            let mut work = d.session.take_save_execution(&port, ticket).unwrap();
            let snapshot = work.snapshot().clone();
            prepare(&mut work, &golden, &d.pool).unwrap();
            assert_eq!(
                snapshot
                    .manifest()
                    .originals
                    .iter()
                    .map(|b| b.byte_len())
                    .sum::<u64>(),
                bytes
            );
            d.session.cancel_save(ticket).unwrap();
            drop(work);
            d.advance();
            assert_eq!(
                d.session.acknowledge_save(ticket),
                Ok(SaveAcknowledgement::WaitingResources)
            );
            assert!(candidate.upgrade().is_some());
            drop(snapshot);
            d.advance();
            assert!(d.status().active.is_none());
            assert!(candidate.upgrade().is_none());
        }
        assert_eq!(d.session.snapshot(), before);
        assert_eq!(
            d.session.history_statistics(before.session_id).unwrap(),
            stats
        );
        drop(d.pool);
        assert!(weak.iter().all(|w| w.upgrade().is_some()));
        drop(d.session);
        assert!(weak.iter().all(|w| w.upgrade().is_none()));
    }
}

#[test]
fn equal_project_and_head_unknown_sessions_isolate_reconcile_controls_and_results() {
    let mut first = Driver::new();
    first.set("first A");
    let mut second = restored(true, RestoreSelection::Manual, 2);
    let p1 = first.install(SaveCandidateLimits::default(), false);
    let p2 = second.install(SaveCandidateLimits::default(), true);
    let t1 = first.admit();
    let t2 = second.admit();
    assert_eq!(t1.target.project, t2.target.project);
    assert_eq!(t1.source.state.head, t2.source.state.head);
    assert_ne!(t1.key.session, t2.key.session);
    let g1 = first.golden();
    let g2 = second.golden();
    let mut w1 = first.session.take_save_execution(&p1, t1).unwrap();
    let mut w2 = second.session.take_save_execution(&p2, t2).unwrap();
    let proof1 = prepare(&mut w1, &g1, &first.pool).unwrap();
    let proof2 = prepare(&mut w2, &g2, &second.pool).unwrap();
    let fact1 = fact(&w1);
    let fact2 = fact(&w2);
    w1.begin_write().unwrap();
    w2.begin_write().unwrap();
    drop(w1);
    drop(w2);
    first.advance();
    second.advance();
    let q1 = first.session.begin_save_reconcile(t1).unwrap().ticket();
    let q2 = second.session.begin_save_reconcile(t2).unwrap().ticket();
    let s1 = first.status();
    let s2 = second.status();
    assert_eq!(
        first.session.begin_save_reconcile(t2),
        Err(SaveCoordinationError::WrongSession)
    );
    assert_eq!(
        second.session.begin_save_reconcile(t1),
        Err(SaveCoordinationError::WrongSession)
    );
    assert_eq!(
        first.session.cancel_save_reconcile(q2),
        Err(SaveCoordinationError::WrongSession)
    );
    assert_eq!(
        second.session.cancel_save_reconcile(q1),
        Err(SaveCoordinationError::WrongSession)
    );
    assert!(matches!(
        first.session.take_save_reconcile(&p1, q2),
        Err(SaveCoordinationError::WrongSession)
    ));
    assert!(matches!(
        second.session.take_save_reconcile(&p2, q1),
        Err(SaveCoordinationError::WrongSession)
    ));
    assert!(matches!(
        first.session.take_save_reconcile(&p2, q1),
        Err(SaveCoordinationError::WrongPort)
    ));
    assert!(matches!(
        second.session.take_save_reconcile(&p1, q2),
        Err(SaveCoordinationError::WrongPort)
    ));
    assert_eq!(first.status(), s1);
    assert_eq!(second.status(), s2);
    let query1 = first.session.take_save_reconcile(&p1, q1).unwrap();
    let query2 = second.session.take_save_reconcile(&p2, q2).unwrap();
    query1.publish(committed(proof2, fact2)).unwrap();
    query2.publish(committed(proof1, fact1)).unwrap();
    first.advance();
    second.advance();
    for (d, before) in [(&first, s1), (&second, s2)] {
        assert_eq!(d.active().phase, SaveProtocolPhase::Reconciling);
        assert!(d.active().outcome.is_none());
        assert_eq!(d.status().facts.files, before.facts.files);
        assert_eq!(d.session.save_source(), before.facts.current.source);
    }
    drop(query1);
    drop(query2);
    first.advance();
    second.advance();
    let next1 = first.session.begin_save_reconcile(t1).unwrap().ticket();
    let next2 = second.session.begin_save_reconcile(t2).unwrap().ticket();
    assert_eq!(next1.attempt, 2);
    assert_eq!(next2.attempt, 2);
    let query1 = first.session.take_save_reconcile(&p1, next1).unwrap();
    let query2 = second.session.take_save_reconcile(&p2, next2).unwrap();
    query1
        .publish(SaveStorageObservation::Replayed {
            witness: proof1,
            fact: fact1,
            binding_conflict: false,
        })
        .unwrap();
    query2
        .publish(SaveStorageObservation::Replayed {
            witness: proof2,
            fact: fact2,
            binding_conflict: false,
        })
        .unwrap();
    first.advance();
    second.advance();
    assert_eq!(first.status().facts.files.manual, Some(fact1));
    assert_eq!(second.status().facts.files.manual, Some(fact2));
    assert_eq!(
        first.session.acknowledge_save(t1),
        Ok(SaveAcknowledgement::WaitingResources)
    );
    assert_eq!(
        second.session.acknowledge_save(t2),
        Ok(SaveAcknowledgement::WaitingResources)
    );
    drop(query1);
    drop(query2);
    first.advance();
    second.advance();
    assert!(first.status().active.is_none());
    assert!(second.status().active.is_none());
}
