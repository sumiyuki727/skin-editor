use skin_editor_gameplay::*;

fn time(at: i64) -> MapTime {
    MapTime::from_micros(at)
}
fn source(id: u128) -> RunSource {
    RunSource {
        run: RunId::new(id).unwrap(),
        transport: TransportEpoch::new(1).unwrap(),
        input: InputEpoch::new(1).unwrap(),
        revision: MapRevision::new(7),
    }
}
fn run(id: u128, targets: &[i64], limits: Limits) -> SyntheticRun {
    let circles: Vec<_> = targets
        .iter()
        .enumerate()
        .map(|(index, at)| {
            Circle::new(
                ObjectId::new(u32::try_from(index).unwrap()),
                time(*at),
                Point::new(0, 0).unwrap(),
            )
            .unwrap()
        })
        .collect();
    let map = SyntheticMap::try_new(MapRevision::new(7), &circles).unwrap();
    let start = targets.first().map_or(0, |at| at - 200_001);
    SyntheticRun::new(source(id), map, time(start), limits).unwrap()
}
fn press(sequence: u64, at: i64, slot: PhysicalSlot, x: i32) -> InputEvent {
    InputEvent {
        sequence: InputSequence::new(sequence).unwrap(),
        at: time(at),
        slot,
        edge: InputEdge::Press(Point::new(x, 0).unwrap()),
    }
}
fn release(sequence: u64, at: i64, slot: PhysicalSlot) -> InputEvent {
    InputEvent {
        sequence: InputSequence::new(sequence).unwrap(),
        at: time(at),
        slot,
        edge: InputEdge::Release,
    }
}
fn fact(sequence: u64, slot: PhysicalSlot) -> ReleaseObservation {
    ReleaseObservation {
        sequence: InputSequence::new(sequence).unwrap(),
        slot,
    }
}
fn advance(run: &mut SyntheticRun, at: i64, budget: WorkBudget) {
    if run
        .observation()
        .sealed_through
        .is_none_or(|sealed| sealed < time(at))
    {
        run.seal_through(run.source(), time(at)).unwrap();
    }
    let mut report = run.advance(run.source(), time(at), budget).unwrap();
    for _ in 0..1024 {
        match report {
            AdvanceReport::Complete { through, steps } => {
                assert_eq!(through, time(at));
                assert!(steps <= budget.get());
                return;
            }
            AdvanceReport::Pending {
                cursor,
                reason,
                steps,
                ..
            } => {
                assert!(steps <= budget.get());
                assert_eq!(reason, PendingReason::WorkBudget);
                report = run.continue_advance(cursor, budget).unwrap();
            }
        }
    }
    panic!("advance exceeded fixture bound");
}
fn settle(run: &mut SyntheticRun, at: i64, budget: WorkBudget) -> usize {
    if run
        .observation()
        .sealed_through
        .is_none_or(|sealed| sealed < time(at))
    {
        run.seal_through(run.source(), time(at)).unwrap();
    }
    for _ in 0..1024 {
        match run.settle_freeze(run.source(), budget).unwrap() {
            FreezeReport::Frozen {
                at: through,
                steps,
                cancelled_inputs,
            } => {
                assert_eq!(through, time(at));
                assert!(steps <= budget.get());
                assert_eq!(run.observation().status, RunStatus::Frozen);
                assert_eq!(run.observation().completed_through, Some(time(at)));
                assert_eq!(run.observation().held_slots, [false; 4]);
                assert_eq!(run.observation().pending_inputs, 0);
                return cancelled_inputs;
            }
            FreezeReport::Pending {
                reason,
                steps,
                boundary,
                ..
            } => {
                assert!(steps <= budget.get());
                assert_eq!(boundary, time(at));
                assert_eq!(reason, PendingReason::WorkBudget);
                assert_eq!(run.observation().status, RunStatus::Freezing);
            }
        }
    }
    panic!("freeze exceeded fixture bound");
}
fn stop(run: &mut SyntheticRun, at: i64, budget: WorkBudget) -> usize {
    run.request_freeze(run.source(), time(at)).unwrap();
    settle(run, at, budget)
}
fn drain(run: &mut SyntheticRun) -> Vec<GameEvent> {
    let mut events = Vec::new();
    while let Some(event) = run.event_at(0) {
        events.push(event);
        run.acknowledge(event.key).unwrap();
    }
    events
}

#[test]
fn accepted_input_settles_but_freeze_first_rejects_new_same_time_input() {
    let mut first = run(20_001, &[1_000_000], Limits::tiny());
    first
        .submit_inputs(
            first.source(),
            &[press(1, 1_000_000, PhysicalSlot::PrimaryOne, 0)],
        )
        .unwrap();
    let old_source = first.source();
    stop(&mut first, 1_000_000, WorkBudget::one());
    assert_eq!(
        first.observation().committed_judgements[0].unwrap().result,
        Judgement::Hit
    );
    assert_eq!(first.source(), old_source); // Closing does not allocate an epoch.
    assert_eq!(
        first.event_at(0).unwrap().kind,
        EventKind::Judgement(Judgement::Hit)
    );
    let sound = first.event_at(1).unwrap();
    assert_eq!(sound.kind, EventKind::SoundIntent);
    assert!(!first.sound_intent_allowed(sound.key).unwrap());

    let mut second = run(20_002, &[1_000_000], Limits::tiny());
    second
        .request_freeze(second.source(), time(1_000_000))
        .unwrap();
    let before = second.observation();
    assert_eq!(
        second.submit_inputs(
            second.source(),
            &[press(1, 1_000_000, PhysicalSlot::PrimaryOne, 0)]
        ),
        Err(RunError::GateClosed)
    );
    assert_eq!(
        second.settle_freeze(second.source(), WorkBudget::one()),
        Err(RunError::BeyondSeal)
    );
    assert_eq!(second.observation(), before);
    settle(&mut second, 1_000_000, WorkBudget::one());
    assert_eq!(
        second.observation().committed_judgements,
        [None; MAX_CIRCLES]
    );
    assert_eq!(second.event_at(0), None);
    let frozen = second.observation();
    for _ in 0..20 {
        second
            .request_freeze(second.source(), time(1_000_000))
            .unwrap();
        assert_eq!(
            second
                .settle_freeze(second.source(), WorkBudget::one())
                .unwrap(),
            FreezeReport::Frozen {
                at: time(1_000_000),
                steps: 0,
                cancelled_inputs: 0
            }
        );
        assert_eq!(
            second.advance(second.source(), time(1_300_000), WorkBudget::one()),
            Err(RunError::GateClosed)
        );
        assert_eq!(second.observation(), frozen);
    }
}

#[test]
fn partial_timestamp_and_full_queues_stop_without_losing_prefix_or_replaying_future_input() {
    let mut run = run(20_003, &[1_000_000], Limits::tiny());
    advance(&mut run, 900_000, WorkBudget::one());
    run.submit_inputs(
        run.source(),
        &[
            press(1, 1_000_000, PhysicalSlot::PrimaryOne, 0),
            release(2, 1_000_000, PhysicalSlot::PrimaryOne),
            press(3, 1_000_000, PhysicalSlot::SecondaryOne, 0),
            release(4, 1_000_000, PhysicalSlot::SecondaryOne),
        ],
    )
    .unwrap();
    run.seal_through(run.source(), time(1_000_000)).unwrap();
    let mut report = run
        .advance(run.source(), time(1_000_000), WorkBudget::one())
        .unwrap();
    let mut old_cursor = None;
    for _ in 0..100 {
        let AdvanceReport::Pending { cursor, .. } = report else {
            panic!("need partial timestamp")
        };
        if run.observation().committed_judgements[0].is_some() {
            old_cursor = Some(cursor);
            break;
        }
        report = run.continue_advance(cursor, WorkBudget::one()).unwrap();
    }
    let old_cursor = old_cursor.expect("hit must commit within fixture bound");
    run.submit_inputs(
        run.source(),
        &[press(5, 1_000_001, PhysicalSlot::PrimaryOne, 0)],
    )
    .unwrap();
    assert_eq!(
        (
            run.observation().pending_inputs,
            run.observation().pending_events
        ),
        (4, 2)
    );
    let old_source = run.source();
    let old_frame = run.observation().ready_frame;
    run.request_freeze(old_source, time(1_000_000)).unwrap();
    assert_eq!(run.observation().status, RunStatus::Freezing);
    assert_eq!(run.observation().ready_frame, old_frame);
    assert_eq!(
        run.continue_advance(old_cursor, WorkBudget::one()),
        Err(RunError::StaleCursor)
    );
    assert_eq!(
        run.submit_inputs(
            old_source,
            &[press(6, 1_000_000, PhysicalSlot::PrimaryOne, 0); 5]
        ),
        Err(RunError::GateClosed)
    );
    assert_eq!(settle(&mut run, 1_000_000, WorkBudget::one()), 1);
    assert_eq!(run.observation().pending_events, 2);
    let sound = run.event_at(1).unwrap();
    let frozen_frame = run.observation().ready_frame.unwrap();
    let resumed = run.resume(old_source).unwrap();
    assert_eq!(resumed.input.get(), 2);
    assert_eq!(resumed.transport, old_source.transport);
    assert_eq!(run.observation().completed_through, Some(time(1_000_000)));
    assert_eq!(run.observation().freeze.unwrap().cancelled_inputs, 1);
    assert_eq!(run.observation().ready_frame.unwrap().source, resumed);
    assert_eq!(frozen_frame.source, old_source);
    assert!(!run.sound_intent_allowed(sound.key).unwrap());
    let before = run.observation();
    assert_eq!(
        run.submit_inputs(
            old_source,
            &[press(5, 1_000_001, PhysicalSlot::PrimaryOne, 0)]
        ),
        Err(RunError::StaleSource)
    );
    assert_eq!(
        run.continue_advance(old_cursor, WorkBudget::one()),
        Err(RunError::StaleSource)
    );
    assert_eq!(
        run.cancel_request(old_cursor.request()),
        Err(RunError::StaleSource)
    );
    assert_eq!(run.observation(), before);
    let old_events = drain(&mut run);
    assert_eq!(old_events.len(), 2);
    assert_eq!(
        run.acknowledge(sound.key),
        Ok(Acknowledgement::AlreadyRetired)
    );
    advance(&mut run, 1_000_001, WorkBudget::one());
    assert_eq!(run.event_at(0), None);
    assert_eq!(
        run.observation().committed_judgements[0].unwrap().at,
        time(1_000_000)
    );
}

#[test]
fn freezing_event_backpressure_preserves_atomic_hit_until_credit_returns() {
    let mut run = run(20_004, &[0, 1_000_000], Limits::tiny());
    advance(&mut run, 100_001, WorkBudget::one());
    let miss = run.event_at(0).unwrap();
    run.submit_inputs(
        run.source(),
        &[press(1, 1_000_000, PhysicalSlot::PrimaryOne, 0)],
    )
    .unwrap();
    run.seal_through(run.source(), time(1_000_000)).unwrap();
    run.request_freeze(run.source(), time(1_000_000)).unwrap();
    let mut blocked = false;
    for _ in 0..100 {
        if let FreezeReport::Pending {
            reason: PendingReason::EventBackpressure,
            ..
        } = run.settle_freeze(run.source(), WorkBudget::one()).unwrap()
        {
            blocked = true;
            break;
        }
    }
    assert!(blocked);
    let before = run.observation();
    assert_eq!(before.status, RunStatus::Freezing);
    assert_eq!(before.committed_judgements[1], None);
    assert_eq!(before.held_slots, [false; 4]);
    assert_eq!((before.pending_inputs, before.pending_events), (1, 1));
    assert!(before.completed_through.unwrap() < time(1_000_000));
    assert!(matches!(
        run.settle_freeze(run.source(), WorkBudget::one()).unwrap(),
        FreezeReport::Pending {
            reason: PendingReason::EventBackpressure,
            steps: 0,
            ..
        }
    ));
    assert_eq!(run.observation(), before);
    run.acknowledge(miss.key).unwrap();
    settle(&mut run, 1_000_000, WorkBudget::one());
    assert_eq!(
        run.observation().committed_judgements[1].unwrap().result,
        Judgement::Hit
    );
    assert_eq!(run.event_at(0).unwrap().key.sequence().get(), 2);
    assert_eq!(run.event_at(1).unwrap().key.sequence().get(), 3);
    assert_eq!(run.event_at(1).unwrap().kind, EventKind::SoundIntent);
}

#[test]
fn new_epoch_release_fences_old_press_and_does_not_release_consumed_hold() {
    let mut run = run(20_005, &[1_000_000], Limits::reference());
    run.submit_inputs(
        run.source(),
        &[
            press(1, 900_000, PhysicalSlot::PrimaryOne, 11),
            release(2, 1_050_000, PhysicalSlot::PrimaryOne),
        ],
    )
    .unwrap();
    advance(&mut run, 900_000, WorkBudget::full());
    let old_source = run.source();
    assert_eq!(stop(&mut run, 900_000, WorkBudget::full()), 1);
    let current = run.resume(old_source).unwrap();
    let before = run.observation();
    assert_eq!(
        run.submit_inputs(current, &[press(1, 900_001, PhysicalSlot::PrimaryOne, 0)]),
        Err(RunError::RearmRequired)
    );
    assert_eq!(
        run.observe_release(old_source, fact(2, PhysicalSlot::PrimaryOne)),
        Err(RunError::StaleSource)
    );
    assert_eq!(run.observation(), before);
    assert_eq!(
        run.observe_release(current, fact(2, PhysicalSlot::PrimaryOne)),
        Ok(RearmReport::Observed { newly_armed: true })
    );
    let before = run.observation();
    assert_eq!(
        run.observe_release(current, fact(2, PhysicalSlot::PrimaryOne)),
        Ok(RearmReport::Repeated)
    );
    assert_eq!(run.observation(), before);
    assert_eq!(
        run.submit_inputs(current, &[press(1, 900_001, PhysicalSlot::PrimaryOne, 0)]),
        Err(RunError::OldInputSequence)
    );
    assert_eq!(
        run.submit_inputs(current, &[press(3, 900_002, PhysicalSlot::PrimaryTwo, 0)]),
        Err(RunError::RearmRequired)
    );
    run.submit_inputs(current, &[press(3, 900_002, PhysicalSlot::PrimaryOne, 0)])
        .unwrap();
    advance(&mut run, 900_002, WorkBudget::full());
    let sound = run.event_at(1).unwrap();
    assert!(run.sound_intent_allowed(sound.key).unwrap());
    assert_eq!(run.observation().held_slots, [true, false, false, false]);
    assert_eq!(
        run.observe_release(current, fact(4, PhysicalSlot::PrimaryOne)),
        Ok(RearmReport::Observed { newly_armed: false })
    );
    assert_eq!(run.observation().held_slots, [true, false, false, false]);
    run.submit_inputs(current, &[release(5, 1_050_000, PhysicalSlot::PrimaryOne)])
        .unwrap();
    assert_eq!(run.observation().held_slots, [true, false, false, false]);
    advance(&mut run, 1_049_999, WorkBudget::full());
    assert_eq!(run.observation().held_slots, [true, false, false, false]);
    advance(&mut run, 1_050_000, WorkBudget::full());
    assert_eq!(run.observation().held_slots, [false; 4]);
}

#[test]
fn release_batches_arm_atomically_and_capture_facts_need_no_queue_credit() {
    let mut run = run(20_006, &[1_000_000], Limits::tiny());
    stop(&mut run, 900_000, WorkBudget::one());
    let current = run.resume(run.source()).unwrap();
    // The captured release is earlier by sequence, despite the batch order.
    run.submit_inputs(
        current,
        &[
            press(2, 1_000_000, PhysicalSlot::PrimaryOne, 0),
            release(1, 999_999, PhysicalSlot::PrimaryOne),
        ],
    )
    .unwrap();
    let before = run.observation();
    assert_eq!(
        run.submit_inputs(
            current,
            &[
                release(3, 1_000_000, PhysicalSlot::SecondaryOne),
                press(4, 1_000_000, PhysicalSlot::PrimaryTwo, 0)
            ]
        ),
        Err(RunError::RearmRequired)
    );
    assert_eq!(run.observation(), before); // No partial arming of SecondaryOne.
    run.submit_inputs(
        current,
        &[
            release(3, 1_000_000, PhysicalSlot::SecondaryOne),
            press(4, 1_000_000, PhysicalSlot::SecondaryOne, 0),
        ],
    )
    .unwrap();
    assert_eq!(run.observation().pending_inputs, 4);
    assert_eq!(
        run.observe_release(current, fact(5, PhysicalSlot::PrimaryTwo)),
        Ok(RearmReport::Observed { newly_armed: true })
    );
    assert_eq!(run.observation().pending_inputs, 4);
    run.request_freeze(current, time(900_000)).unwrap();
    assert!(matches!(
        run.settle_freeze(current, WorkBudget::one()).unwrap(),
        FreezeReport::Pending {
            cancelled_inputs: 1,
            steps: 1,
            ..
        }
    ));
    assert_eq!(run.observation().pending_inputs, 3);
    assert_eq!(settle(&mut run, 900_000, WorkBudget::one()), 4);
    assert_eq!(run.event_at(0), None);
}

#[test]
fn release_receipts_compare_full_payload_and_record_kind() {
    let mut first = run(20_007, &[1_000_000], Limits::tiny());
    stop(&mut first, 900_000, WorkBudget::one());
    let current = first.resume(first.source()).unwrap();
    first
        .observe_release(current, fact(1, PhysicalSlot::PrimaryOne))
        .unwrap();
    assert_eq!(
        first.observe_release(current, fact(1, PhysicalSlot::PrimaryTwo)),
        Err(RunError::InputConflict)
    );
    assert_eq!(
        first.observation().status,
        RunStatus::Failed(Failure {
            cause: RunError::InputConflict,
            cancelled_inputs: 0
        })
    );
    assert_eq!(first.resume(current), Err(RunError::Faulted));
    assert_eq!(first.observation().armed_slots, [false; 4]);
    let mut second = run(20_008, &[1_000_000], Limits::tiny());
    stop(&mut second, 900_000, WorkBudget::one());
    let current = second.resume(second.source()).unwrap();
    second
        .submit_inputs(current, &[release(1, 900_001, PhysicalSlot::PrimaryOne)])
        .unwrap();
    assert_eq!(
        second.observe_release(current, fact(1, PhysicalSlot::PrimaryOne)),
        Err(RunError::InputConflict)
    );
    assert_eq!(
        second.observation().status,
        RunStatus::Failed(Failure {
            cause: RunError::InputConflict,
            cancelled_inputs: 1
        })
    );
    assert_eq!(second.observation().pending_inputs, 0);
    let mut third = run(20_009, &[1_000_000], Limits::tiny());
    stop(&mut third, 900_000, WorkBudget::one());
    let current = third.resume(third.source()).unwrap();
    third
        .observe_release(current, fact(1, PhysicalSlot::PrimaryOne))
        .unwrap();
    assert_eq!(
        third.submit_inputs(current, &[release(1, 900_001, PhysicalSlot::PrimaryOne)]),
        Err(RunError::InputConflict)
    );
    assert_eq!(third.observation().pending_inputs, 0);
}

#[test]
fn freeze_discards_only_uncommitted_future_projection_and_keeps_the_sealed_horizon() {
    let mut run = run(20_010, &[1_000_000, 2_000_000], Limits::tiny());
    advance(&mut run, 900_000, WorkBudget::one());
    run.seal_through(run.source(), time(1_000_000)).unwrap();
    let AdvanceReport::Pending { cursor, .. } = run
        .advance(run.source(), time(950_000), WorkBudget::one())
        .unwrap()
    else {
        panic!("two-object draft must yield")
    };
    let old_frame = run.observation().ready_frame;
    run.request_freeze(run.source(), time(925_000)).unwrap();
    assert_eq!(run.observation().ready_frame, old_frame);
    assert_eq!(
        run.continue_advance(cursor, WorkBudget::one()),
        Err(RunError::StaleCursor)
    );
    settle(&mut run, 925_000, WorkBudget::one());
    assert_eq!(run.observation().sealed_through, Some(time(1_000_000)));
    let current = run.resume(run.source()).unwrap();
    advance(&mut run, 950_000, WorkBudget::one());
    assert_eq!(run.observation().ready_frame.unwrap().source, current);
    run.observe_release(current, fact(1, PhysicalSlot::PrimaryOne))
        .unwrap();
    let before = run.observation();
    assert_eq!(
        run.submit_inputs(current, &[press(2, 950_001, PhysicalSlot::PrimaryOne, 0)]),
        Err(RunError::LateInput)
    );
    assert_eq!(run.observation().ready_frame, before.ready_frame);
    assert_eq!(run.observation().committed_judgements, [None; MAX_CIRCLES]);
}

#[test]
fn invalid_controls_and_stale_sources_do_not_change_the_freeze() {
    let mut run = run(20_011, &[1_000_000], Limits::tiny());
    let before = run.observation();
    assert_eq!(
        run.sound_intent_allowed(EventKey::new(
            RunId::new(999_999).unwrap(),
            run.source().transport,
            EventSequence::new(1).unwrap()
        )),
        Err(RunError::StaleSource)
    );
    assert_eq!(
        run.sound_intent_allowed(EventKey::new(
            run.source().run,
            run.source().transport,
            EventSequence::new(1).unwrap()
        )),
        Err(RunError::UnknownEvent)
    );
    assert_eq!(run.resume(run.source()), Err(RunError::NotFrozen));
    assert_eq!(
        run.settle_freeze(run.source(), WorkBudget::one()),
        Err(RunError::NotFreezing)
    );
    assert_eq!(
        run.request_freeze(run.source(), time(799_998)),
        Err(RunError::InvalidTarget)
    );
    assert_eq!(run.observation(), before);
    advance(&mut run, 900_000, WorkBudget::one());
    assert_eq!(
        run.request_freeze(run.source(), time(899_999)),
        Err(RunError::InvalidTarget)
    );
    run.request_freeze(run.source(), time(900_000)).unwrap();
    let before = run.observation();
    let stale = RunSource {
        input: InputEpoch::new(2).unwrap(),
        ..run.source()
    };
    assert_eq!(
        run.request_freeze(stale, time(900_000)),
        Err(RunError::StaleSource)
    );
    assert_eq!(
        run.settle_freeze(stale, WorkBudget::full()),
        Err(RunError::StaleSource)
    );
    assert_eq!(
        run.observe_release(stale, fact(1, PhysicalSlot::PrimaryOne)),
        Err(RunError::StaleSource)
    );
    assert_eq!(
        run.request_freeze(run.source(), time(900_001)),
        Err(RunError::Busy)
    );
    assert_eq!(
        run.seal_through(run.source(), time(900_001)),
        Err(RunError::GateClosed)
    );
    assert_eq!(run.resume(run.source()), Err(RunError::NotFrozen));
    assert_eq!(
        run.settle_freeze(run.source(), WorkBudget::full()),
        Err(RunError::BudgetLimit)
    );
    assert_eq!(run.observation(), before);
    settle(&mut run, 900_000, WorkBudget::one());
    assert_eq!(
        run.observe_release(run.source(), fact(1, PhysicalSlot::PrimaryOne)),
        Err(RunError::GateClosed)
    );
}

#[test]
fn empty_signed_start_freezes_without_sentinels_or_mutating_retained_snapshots() {
    let map = SyntheticMap::try_new(MapRevision::new(7), &[]).unwrap();
    let mut run = SyntheticRun::new(source(20_012), map, time(i64::MIN), Limits::tiny()).unwrap();
    assert_eq!(run.observation().ready_frame, None);
    stop(&mut run, i64::MIN, WorkBudget::one());
    let snapshot = run.observation().ready_frame.unwrap();
    assert!(snapshot.is_empty());
    let old = run.source();
    let current = run.resume(old).unwrap();
    assert_eq!(snapshot.source, old);
    assert_eq!(snapshot.at, time(i64::MIN));
    assert_eq!(run.observation().ready_frame.unwrap().source, current);
    advance(&mut run, i64::MIN + 1, WorkBudget::one());
    stop(&mut run, i64::MIN + 1, WorkBudget::one());
    assert_eq!(run.event_at(0), None);
    assert_eq!(
        run.request_freeze(old, time(i64::MIN)),
        Err(RunError::StaleSource)
    );
}

fn resumed_trace(
    id: u128,
    limits: Limits,
    budget: WorkBudget,
    segmented: bool,
) -> Vec<(u64, ObjectId, MapTime, EventKind)> {
    let mut run = run(id, &[1_000_000, 2_000_000], limits);
    if segmented {
        advance(&mut run, 800_000, budget);
        advance(&mut run, 899_999, budget);
    }
    stop(&mut run, 900_000, budget);
    let mut events = Vec::new();
    for target in [1_000_000, 2_000_000] {
        let old = run.source();
        let current = run.resume(old).unwrap();
        assert_eq!(current.input.get(), old.input.get() + 1);
        assert_eq!(current.transport, old.transport);
        run.observe_release(current, fact(1, PhysicalSlot::PrimaryOne))
            .unwrap();
        run.submit_inputs(
            current,
            &[
                press(2, target, PhysicalSlot::PrimaryOne, 0),
                release(3, target + 1, PhysicalSlot::PrimaryOne),
            ],
        )
        .unwrap();
        if segmented {
            advance(&mut run, target, budget);
        }
        advance(&mut run, target + 1, budget);
        let sound = run.event_at(1).unwrap();
        assert!(run.sound_intent_allowed(sound.key).unwrap());
        stop(&mut run, target + 1, budget);
        assert!(!run.sound_intent_allowed(sound.key).unwrap());
        for event in drain(&mut run) {
            events.push((
                event.key.sequence().get(),
                event.object,
                event.at,
                event.kind,
            ));
        }
        assert!(!run.sound_intent_allowed(sound.key).unwrap());
    }
    assert_eq!(run.observation().status, RunStatus::Frozen);
    assert_eq!(run.observation().freeze.unwrap().cancelled_inputs, 0);
    assert_eq!(
        run.observation().committed_judgements[0].unwrap().at,
        time(1_000_000)
    );
    assert_eq!(
        run.observation().committed_judgements[1].unwrap().at,
        time(2_000_000)
    );
    events
}

#[test]
fn budgets_and_segmentation_preserve_resumed_event_identity() {
    let expected = vec![
        (
            1,
            ObjectId::new(0),
            time(1_000_000),
            EventKind::Judgement(Judgement::Hit),
        ),
        (2, ObjectId::new(0), time(1_000_000), EventKind::SoundIntent),
        (
            3,
            ObjectId::new(1),
            time(2_000_000),
            EventKind::Judgement(Judgement::Hit),
        ),
        (4, ObjectId::new(1), time(2_000_000), EventKind::SoundIntent),
    ];
    let mut id = 30_000;
    for (limits, budget) in [
        (Limits::tiny(), WorkBudget::one()),
        (Limits::reference(), WorkBudget::one()),
        (Limits::reference(), WorkBudget::full()),
    ] {
        for segmented in [false, true] {
            id += 1;
            assert_eq!(resumed_trace(id, limits, budget, segmented), expected);
        }
    }
}

#[test]
fn freezing_at_each_advance_yield_preserves_only_the_authenticated_prefix() {
    for cut in 0..24 {
        let mut run = run(50_000 + cut, &[1_000_000], Limits::tiny());
        run.submit_inputs(
            run.source(),
            &[
                press(1, 1_000_000, PhysicalSlot::PrimaryOne, 0),
                release(2, 1_000_000, PhysicalSlot::PrimaryOne),
                press(3, 1_000_000, PhysicalSlot::SecondaryOne, 0),
                release(4, 1_000_000, PhysicalSlot::SecondaryOne),
            ],
        )
        .unwrap();
        run.seal_through(run.source(), time(1_000_000)).unwrap();
        let mut report = run
            .advance(run.source(), time(1_000_000), WorkBudget::one())
            .unwrap();
        for _ in 0..cut {
            if let AdvanceReport::Pending { cursor, .. } = report {
                report = run.continue_advance(cursor, WorkBudget::one()).unwrap();
            } else {
                break;
            }
        }
        run.request_freeze(run.source(), time(1_000_000)).unwrap();
        if let AdvanceReport::Pending { cursor, .. } = report {
            assert_eq!(
                run.continue_advance(cursor, WorkBudget::one()),
                Err(RunError::StaleCursor)
            );
        }
        assert_eq!(settle(&mut run, 1_000_000, WorkBudget::one()), 0);
        assert_eq!(
            run.observation().committed_judgements[0],
            Some(JudgementRecord {
                object: ObjectId::new(0),
                at: time(1_000_000),
                result: Judgement::Hit
            })
        );
        assert_eq!(run.observation().pending_events, 2);
    }
    let mut future = run(50_024, &[1_000_000], Limits::tiny());
    future
        .submit_inputs(
            future.source(),
            &[press(1, 1_000_000, PhysicalSlot::PrimaryOne, 0)],
        )
        .unwrap();
    advance(&mut future, 900_000, WorkBudget::one());
    future
        .seal_through(future.source(), time(1_000_000))
        .unwrap();
    // One unit selects the input; its candidate search/commit is still private.
    assert!(matches!(
        future
            .advance(future.source(), time(1_000_000), WorkBudget::one())
            .unwrap(),
        AdvanceReport::Pending { .. }
    ));
    assert_eq!(stop(&mut future, 999_999, WorkBudget::one()), 1);
    assert_eq!(
        future.observation().committed_judgements,
        [None; MAX_CIRCLES]
    );
    assert_eq!(future.event_at(0), None);
}

#[test]
fn quarantined_press_precedes_time_errors_without_reopening_the_sealed_prefix() {
    let mut run = run(60_000, &[1_000_000], Limits::tiny());
    stop(&mut run, 900_000, WorkBudget::one());
    let current = run.resume(run.source()).unwrap();
    let before = run.observation();
    for at in [900_000, 799_998] {
        assert_eq!(
            run.submit_inputs(current, &[press(1, at, PhysicalSlot::PrimaryOne, 0)]),
            Err(RunError::RearmRequired)
        );
        assert_eq!(run.observation(), before);
    }
    run.observe_release(current, fact(2, PhysicalSlot::PrimaryOne))
        .unwrap();
    assert_eq!(
        run.submit_inputs(current, &[press(1, 900_001, PhysicalSlot::PrimaryOne, 0)]),
        Err(RunError::OldInputSequence)
    );
    // Once the slot is armed, ordinary C1 remains strict, including F itself.
    assert_eq!(
        run.submit_inputs(current, &[press(3, 900_000, PhysicalSlot::PrimaryOne, 0)]),
        Err(RunError::LateInput)
    );
    assert_eq!(
        run.observation().status,
        RunStatus::Failed(Failure {
            cause: RunError::LateInput,
            cancelled_inputs: 0
        })
    );
}

#[test]
fn quarantined_press_cannot_cancel_a_full_valid_input_prefix() {
    let mut run = run(60_001, &[1_000_000], Limits::tiny());
    stop(&mut run, 900_000, WorkBudget::one());
    let current = run.resume(run.source()).unwrap();
    run.submit_inputs(
        current,
        &[
            release(1, 1_000_000, PhysicalSlot::PrimaryOne),
            press(2, 1_000_000, PhysicalSlot::PrimaryOne, 0),
            release(3, 1_000_000, PhysicalSlot::SecondaryOne),
            press(4, 1_000_000, PhysicalSlot::SecondaryOne, 0),
        ],
    )
    .unwrap();
    let before = run.observation();
    assert_eq!(before.pending_inputs, 4);
    assert_eq!(
        run.submit_inputs(current, &[press(5, 1_000_000, PhysicalSlot::PrimaryTwo, 0)]),
        Err(RunError::RearmRequired)
    );
    assert_eq!(run.observation(), before);
    advance(&mut run, 1_000_000, WorkBudget::one());
    assert_eq!(run.observation().pending_inputs, 0);
    assert_eq!(
        run.observation().committed_judgements[0].unwrap().result,
        Judgement::Hit
    );
    assert_eq!(run.observation().pending_events, 2);
}

#[test]
fn timed_rearm_cannot_qualify_an_earlier_press_in_the_same_or_later_batch() {
    for (case, same_batch) in [true, false].into_iter().enumerate() {
        let mut run = run(
            60_010 + u128::try_from(case).unwrap(),
            &[1_000_000],
            Limits::tiny(),
        );
        stop(&mut run, 900_000, WorkBudget::one());
        let current = run.resume(run.source()).unwrap();
        let up = release(1, 1_000_010, PhysicalSlot::PrimaryOne);
        let early = press(2, 1_000_000, PhysicalSlot::PrimaryOne, 0);
        if same_batch {
            let before = run.observation();
            assert_eq!(
                run.submit_inputs(current, &[up, early]),
                Err(RunError::RearmRequired)
            );
            assert_eq!(run.observation(), before);
        }
        run.submit_inputs(current, &[up]).unwrap();
        let before = run.observation();
        assert_eq!(
            run.submit_inputs(current, &[early]),
            Err(RunError::RearmRequired)
        );
        assert_eq!(run.observation(), before);
        // Equal map time is valid: the release's lower sequence is consumed first.
        run.submit_inputs(current, &[press(2, 1_000_010, PhysicalSlot::PrimaryOne, 0)])
            .unwrap();
        advance(&mut run, 1_000_010, WorkBudget::one());
        assert_eq!(
            run.observation().committed_judgements[0].unwrap().at,
            time(1_000_010)
        );
        assert_eq!(run.observation().pending_events, 2);
    }
}

#[test]
fn timed_arming_evidence_survives_receipt_eviction_and_repeated_untimed_facts() {
    let mut run = run(60_012, &[1_000_000], Limits::tiny());
    stop(&mut run, 900_000, WorkBudget::one());
    let current = run.resume(run.source()).unwrap();
    let up = release(1, 1_000_010, PhysicalSlot::PrimaryOne);
    run.submit_inputs(current, &[up]).unwrap();
    assert_eq!(
        run.observe_release(current, fact(2, PhysicalSlot::PrimaryOne)),
        Ok(RearmReport::Observed { newly_armed: false })
    );
    for (seq, slot) in [
        (3, PhysicalSlot::SecondaryOne),
        (4, PhysicalSlot::SecondaryTwo),
        (5, PhysicalSlot::PrimaryTwo),
    ] {
        run.observe_release(current, fact(seq, slot)).unwrap();
    }
    assert_eq!(
        run.submit_inputs(current, &[up]),
        Err(RunError::OldInputSequence)
    );
    let before = run.observation();
    assert_eq!(
        run.submit_inputs(current, &[press(6, 1_000_000, PhysicalSlot::PrimaryOne, 0)]),
        Err(RunError::RearmRequired)
    );
    assert_eq!(
        run.observe_release(current, fact(2, PhysicalSlot::PrimaryOne)),
        Ok(RearmReport::Repeated)
    );
    assert_eq!(run.observation(), before);
    run.submit_inputs(current, &[press(6, 1_000_010, PhysicalSlot::PrimaryOne, 0)])
        .unwrap();
    advance(&mut run, 1_000_010, WorkBudget::one());
    assert_eq!(
        run.observation().committed_judgements[0].unwrap().at,
        time(1_000_010)
    );
}

#[test]
fn next_freeze_cycle_does_not_inherit_cancelled_timed_arming_evidence() {
    let mut run = run(60_013, &[1_000_000], Limits::tiny());
    stop(&mut run, 900_000, WorkBudget::one());
    let previous = run.resume(run.source()).unwrap();
    run.submit_inputs(previous, &[release(1, 1_000_010, PhysicalSlot::PrimaryOne)])
        .unwrap();
    assert_eq!(stop(&mut run, 925_000, WorkBudget::one()), 1);
    let current = run.resume(previous).unwrap();
    assert_eq!(
        run.observe_release(previous, fact(2, PhysicalSlot::PrimaryOne)),
        Err(RunError::StaleSource)
    );
    run.observe_release(current, fact(1, PhysicalSlot::PrimaryOne))
        .unwrap();
    run.submit_inputs(current, &[press(2, 925_001, PhysicalSlot::PrimaryOne, 0)])
        .unwrap();
    advance(&mut run, 925_001, WorkBudget::one());
    assert_eq!(
        run.observation().committed_judgements[0].unwrap().at,
        time(925_001)
    );
}
