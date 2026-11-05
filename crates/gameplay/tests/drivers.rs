use skin_editor_gameplay::*;

fn time(at: i64) -> MapTime {
    MapTime::from_micros(at)
}
fn source(id: u128) -> RunSource {
    RunSource {
        run: RunId::new(id).unwrap(),
        transport: TransportEpoch::new(1).unwrap(),
        input: InputEpoch::new(1).unwrap(),
        revision: MapRevision::new(id),
    }
}
fn circles(targets: &[i64]) -> Vec<Circle> {
    targets
        .iter()
        .enumerate()
        .map(|(i, at)| {
            Circle::new(
                ObjectId::new(i as u32),
                time(*at),
                Point::new(i as i32 * 30, 0).unwrap(),
            )
            .unwrap()
        })
        .collect()
}
fn fixture(targets: &[i64], start: i64, limits: Limits) -> SyntheticRun {
    let source = source(90_001);
    SyntheticRun::new(
        source,
        SyntheticMap::try_new(source.revision, &circles(targets)).unwrap(),
        time(start),
        limits,
    )
    .unwrap()
}
fn freeze(run: &mut SyntheticRun, at: i64, budget: WorkBudget) {
    let source = run.source();
    run.request_freeze(source, time(at)).unwrap();
    run.seal_through(source, time(at)).unwrap();
    for _ in 0..256 {
        match run.settle_freeze(source, budget).unwrap() {
            FreezeReport::Frozen { .. } => return,
            FreezeReport::Pending { reason, .. } => assert_eq!(reason, PendingReason::WorkBudget),
        }
    }
    panic!("freeze fixture bound");
}
fn selected(targets: &[i64], limits: Limits, driver: InputDriver) -> SyntheticRun {
    let mut run = fixture(targets, -200_001, limits);
    freeze(&mut run, -200_001, limits.budget());
    let source = run.select_driver(run.source(), driver).unwrap();
    run.resume(source).unwrap();
    run
}
fn input(sequence: u64, at: i64, edge: InputEdge) -> InputEvent {
    InputEvent {
        sequence: InputSequence::new(sequence).unwrap(),
        at: time(at),
        slot: PhysicalSlot::PrimaryOne,
        edge,
    }
}
fn fact(sequence: u64) -> ReleaseObservation {
    ReleaseObservation {
        sequence: InputSequence::new(sequence).unwrap(),
        slot: PhysicalSlot::PrimaryOne,
    }
}
fn collect(run: &mut SyntheticRun, events: &mut Vec<GameEvent>) {
    while let Some(event) = run.event_at(0) {
        events.push(event);
        run.acknowledge(event.key).unwrap();
    }
}
fn advance(run: &mut SyntheticRun, at: i64, budget: WorkBudget, events: &mut Vec<GameEvent>) {
    let source = run.source();
    run.seal_through(source, time(at)).unwrap();
    let mut report = run.advance(source, time(at), budget).unwrap();
    for _ in 0..256 {
        collect(run, events);
        match report {
            AdvanceReport::Complete { .. } => return,
            AdvanceReport::Pending { cursor, steps, .. } => {
                assert!(steps <= budget.get());
                report = run.continue_advance(cursor, budget).unwrap();
            }
        }
    }
    panic!("advance fixture bound");
}
fn prepare(run: &mut SyntheticRun, at: i64, budget: WorkBudget) {
    for _ in 0..128 {
        match run
            .prepare_automatic_inputs(run.source(), time(at), budget)
            .unwrap()
        {
            AutomaticReport::Complete { steps, .. } => {
                assert!(steps <= budget.get());
                return;
            }
            AutomaticReport::Pending { reason, steps, .. } => {
                assert_eq!(reason, AutomaticPendingReason::WorkBudget);
                assert!(steps > 0 && steps <= budget.get());
            }
        }
    }
    panic!("preparation fixture bound");
}
fn drive(run: &mut SyntheticRun, at: i64, budget: WorkBudget, events: &mut Vec<GameEvent>) {
    for _ in 0..256 {
        match run
            .prepare_automatic_inputs(run.source(), time(at), budget)
            .unwrap()
        {
            AutomaticReport::Complete { steps, .. } => {
                assert!(steps <= budget.get());
                advance(run, at, budget, events);
                return;
            }
            AutomaticReport::Pending {
                reason,
                ready_through,
                steps,
                ..
            } => {
                assert!(steps > 0 && steps <= budget.get());
                if reason == AutomaticPendingReason::InputBackpressure {
                    let prefix = ready_through.expect("known admitted prefix");
                    assert!(prefix < time(at));
                    advance(run, prefix.micros(), budget, events);
                }
            }
        }
    }
    panic!("automatic fixture bound");
}

#[test]
fn three_drivers_are_exclusive_and_all_valid_hits_use_the_same_judgement_path() {
    for driver in [
        InputDriver::Manual,
        InputDriver::Automatic,
        InputDriver::Regression,
    ] {
        let mut run = selected(&[0], Limits::tiny(), driver);
        let source = run.source();
        let before = run.observation();
        let oversized = vec![input(1, -1, InputEdge::Release); 65];
        for other in [
            InputDriver::Manual,
            InputDriver::Automatic,
            InputDriver::Regression,
        ] {
            let expected = if other == driver && other == InputDriver::Automatic {
                RunError::AutomaticInputReserved
            } else if other == driver {
                continue;
            } else {
                RunError::DriverMismatch
            };
            assert_eq!(
                run.submit_driver_inputs(source, other, &oversized),
                Err(expected)
            );
            assert_eq!(
                run.observe_driver_release(source, other, fact(1)),
                Err(expected)
            );
            assert_eq!(run.observation(), before);
        }
        let mut events = Vec::new();
        if driver == InputDriver::Automatic {
            drive(&mut run, 1, WorkBudget::one(), &mut events);
        } else {
            assert_eq!(
                run.prepare_automatic_inputs(source, time(0), WorkBudget::one()),
                Err(RunError::DriverMismatch)
            );
            run.observe_driver_release(source, driver, fact(1)).unwrap();
            run.submit_driver_inputs(
                source,
                driver,
                &[input(2, 0, InputEdge::Press(Point::new(0, 0).unwrap()))],
            )
            .unwrap();
            advance(&mut run, 1, WorkBudget::one(), &mut events);
        }
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].kind, EventKind::Judgement(Judgement::Hit));
        assert_eq!(events[1].kind, EventKind::SoundIntent);
        assert_eq!(events[0].at, time(0));
        assert_eq!(events[0].key.sequence().get(), 1);
        assert_eq!(events[1].key.sequence().get(), 2);
        assert_eq!(events[0].evidence, Evidence::SyntheticTest);
        assert_eq!(events[0].profile_version, 1);
    }
}

#[test]
fn automatic_preparation_cannot_seal_early_or_publish_a_gameplay_frame() {
    let mut run = selected(&[0], Limits::tiny(), InputDriver::Automatic);
    let source = run.source();
    let before = run.observation();
    assert_eq!(
        run.seal_through(source, time(0)),
        Err(RunError::AutomaticInputsPending)
    );
    assert_eq!(
        run.prepare_automatic_inputs(source, time(0), WorkBudget::full()),
        Err(RunError::BudgetLimit)
    );
    assert_eq!(run.observation(), before);
    for _ in 0..2 {
        assert!(matches!(
            run.prepare_automatic_inputs(source, time(0), WorkBudget::one()),
            Ok(AutomaticReport::Pending {
                reason: AutomaticPendingReason::WorkBudget,
                admitted: 0,
                steps: 1,
                ..
            })
        ));
        assert_eq!(
            run.seal_through(source, time(0)),
            Err(RunError::AutomaticInputsPending)
        );
    }
    assert!(matches!(
        run.prepare_automatic_inputs(source, time(0), WorkBudget::one()),
        Ok(AutomaticReport::Complete {
            admitted: 2,
            steps: 1,
            ..
        })
    ));
    let prepared = run.observation();
    assert_eq!(prepared.ready_frame, before.ready_frame);
    assert_eq!(prepared.committed_judgements, before.committed_judgements);
    assert_eq!(prepared.pending_inputs, 2);
    assert_eq!(prepared.pending_events, 0);
    assert_eq!(prepared.armed_slots, [true, false, false, false]);
    assert!(matches!(
        run.prepare_automatic_inputs(source, time(0), WorkBudget::one()),
        Ok(AutomaticReport::Complete {
            admitted: 0,
            steps: 0,
            ..
        })
    ));
    for _ in 0..10 {
        assert_eq!(run.observation(), prepared);
    }
    let mut events = Vec::new();
    advance(&mut run, 0, WorkBudget::one(), &mut events);
    assert_eq!(events.len(), 2);
}

#[test]
fn four_object_automatic_results_match_across_tiny_backpressure_and_full_budget() {
    let mut a = selected(
        &[0, 1_000_000, 2_000_000, 3_000_000],
        Limits::tiny(),
        InputDriver::Automatic,
    );
    let mut b = selected(
        &[0, 1_000_000, 2_000_000, 3_000_000],
        Limits::reference(),
        InputDriver::Automatic,
    );
    let mut ea = Vec::new();
    let mut eb = Vec::new();
    drive(&mut a, 3_300_000, WorkBudget::one(), &mut ea);
    drive(&mut b, 3_300_000, WorkBudget::full(), &mut eb);
    assert_eq!(a.observation(), b.observation());
    assert_eq!(ea, eb);
    assert_eq!(ea.len(), 8);
    for (index, event) in ea.iter().enumerate() {
        assert_eq!(event.key.sequence().get(), index as u64 + 1);
        assert_eq!(event.object.get(), (index / 2) as u32);
        assert_eq!(
            event.kind,
            if index % 2 == 0 {
                EventKind::Judgement(Judgement::Hit)
            } else {
                EventKind::SoundIntent
            }
        );
    }
}

#[test]
fn a_smaller_preparation_horizon_keeps_the_uncommitted_future_pulse() {
    let mut run = selected(&[0], Limits::tiny(), InputDriver::Automatic);
    for _ in 0..2 {
        run.prepare_automatic_inputs(run.source(), time(0), WorkBudget::one())
            .unwrap();
    }
    assert!(matches!(
        run.prepare_automatic_inputs(run.source(), time(-1), WorkBudget::one()),
        Ok(AutomaticReport::Complete {
            admitted: 0,
            steps: 1,
            ..
        })
    ));
    assert_eq!(run.observation().pending_inputs, 0);
    let mut events = Vec::new();
    advance(&mut run, -1, WorkBudget::one(), &mut events);
    assert!(events.is_empty());
    drive(&mut run, 0, WorkBudget::one(), &mut events);
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].kind, EventKind::Judgement(Judgement::Hit));
}

#[test]
fn an_unadmitted_plan_can_be_prepared_afresh_only_if_its_target_remains_future() {
    let mut run = selected(&[0], Limits::tiny(), InputDriver::Automatic);
    let old = run.source();
    for _ in 0..2 {
        run.prepare_automatic_inputs(old, time(0), WorkBudget::one())
            .unwrap();
    }
    freeze(&mut run, -1, WorkBudget::one());
    assert_eq!(run.observation().freeze.unwrap().cancelled_inputs, 0);
    run.resume(old).unwrap();
    assert_eq!(
        run.prepare_automatic_inputs(old, time(0), WorkBudget::one()),
        Err(RunError::StaleSource)
    );
    let mut events = Vec::new();
    drive(&mut run, 0, WorkBudget::one(), &mut events);
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].at, time(0));
}

#[test]
fn input_backpressure_keeps_the_whole_pulse_pending_without_faulting_or_replaying() {
    let mut run = selected(
        &[0, 1_000_000, 2_000_000],
        Limits::tiny(),
        InputDriver::Automatic,
    );
    let mut blocked = None;
    for _ in 0..128 {
        let report = run
            .prepare_automatic_inputs(run.source(), time(2_000_001), WorkBudget::one())
            .unwrap();
        if matches!(
            report,
            AutomaticReport::Pending {
                reason: AutomaticPendingReason::InputBackpressure,
                ..
            }
        ) {
            blocked = Some(report);
            break;
        }
    }
    let report = blocked.expect("fixture must fill its input queue");
    assert!(
        matches!(report, AutomaticReport::Pending { ready_through: Some(at), admitted: 0, .. } if at == time(1_000_001))
    );
    let before = run.observation();
    assert_eq!(before.pending_inputs, 4);
    for _ in 0..3 {
        assert!(matches!(
            run.prepare_automatic_inputs(run.source(), time(2_000_001), WorkBudget::one()),
            Ok(AutomaticReport::Pending {
                reason: AutomaticPendingReason::InputBackpressure,
                admitted: 0,
                steps: 1,
                ..
            })
        ));
        assert_eq!(run.observation(), before);
    }
    let mut events = Vec::new();
    advance(&mut run, 1_000_001, WorkBudget::one(), &mut events);
    drive(&mut run, 2_000_001, WorkBudget::one(), &mut events);
    assert_eq!(events.len(), 6);
}

#[test]
fn freezing_at_each_automatic_step_preserves_admitted_input_but_cancels_uncommitted_plans() {
    for cut in 0..=3 {
        let mut run = selected(&[0], Limits::tiny(), InputDriver::Automatic);
        let old = run.source();
        for _ in 0..cut {
            run.prepare_automatic_inputs(old, time(0), WorkBudget::one())
                .unwrap();
        }
        run.request_freeze(old, time(0)).unwrap();
        let closed = run.observation();
        assert_eq!(
            run.prepare_automatic_inputs(old, time(i64::MAX), WorkBudget::one()),
            Err(RunError::GateClosed)
        );
        assert_eq!(run.observation(), closed);
        freeze(&mut run, 0, WorkBudget::one());
        assert_eq!(
            run.observation().committed_judgements[0].is_some(),
            cut == 3
        );
        let new = run.resume(old).unwrap();
        let before = run.observation();
        assert_eq!(
            run.prepare_automatic_inputs(old, time(1), WorkBudget::one()),
            Err(RunError::StaleSource)
        );
        assert_eq!(
            run.observe_driver_release(old, InputDriver::Automatic, fact(1)),
            Err(RunError::StaleSource)
        );
        assert_eq!(run.observation(), before);
        assert_ne!(new.input, old.input);
        prepare(&mut run, 1, WorkBudget::one());
        assert_eq!(run.observation().pending_inputs, 0);
        let mut events = Vec::new();
        drive(&mut run, 100_001, WorkBudget::one(), &mut events);
        assert_eq!(
            run.observation().committed_judgements[0].unwrap().result,
            if cut == 3 {
                Judgement::Hit
            } else {
                Judgement::Miss
            }
        );
        assert_eq!(
            events
                .iter()
                .filter(|event| event.kind == EventKind::SoundIntent)
                .count(),
            usize::from(cut == 3)
        );
    }
}

#[test]
fn future_automatic_inputs_cancelled_by_freeze_are_never_backfilled_after_resume() {
    let mut run = selected(
        &[0, 1_000_000, 2_000_000],
        Limits::tiny(),
        InputDriver::Automatic,
    );
    prepare(&mut run, 1_000_001, WorkBudget::one());
    freeze(&mut run, 0, WorkBudget::one());
    assert_eq!(run.observation().freeze.unwrap().cancelled_inputs, 3);
    let sound = run.event_at(1).unwrap();
    assert!(!run.sound_intent_allowed(sound.key).unwrap());
    run.resume(run.source()).unwrap();
    assert!(!run.sound_intent_allowed(sound.key).unwrap());
    let mut events = Vec::new();
    drive(&mut run, 2_300_000, WorkBudget::one(), &mut events);
    let state = run.observation();
    assert_eq!(
        state.committed_judgements[0].unwrap().result,
        Judgement::Hit
    );
    assert_eq!(
        state.committed_judgements[1].unwrap().result,
        Judgement::Miss
    );
    assert_eq!(
        state.committed_judgements[2].unwrap().result,
        Judgement::Hit
    );
    assert_eq!(
        events
            .iter()
            .filter(|event| event.kind == EventKind::SoundIntent)
            .count(),
        2
    );
}

#[test]
fn switching_driver_requires_frozen_boundary_and_current_source_release_to_rearm() {
    let mut run = fixture(&[0, 1_000_000], -200_001, Limits::tiny());
    let initial = run.source();
    assert_eq!(run.driver(), InputDriver::Manual);
    assert_eq!(
        run.select_driver(initial, InputDriver::Regression),
        Err(RunError::NotFrozen)
    );
    run.submit_inputs(
        initial,
        &[input(1, 0, InputEdge::Press(Point::new(0, 0).unwrap()))],
    )
    .unwrap();
    run.request_freeze(initial, time(0)).unwrap();
    assert_eq!(
        run.select_driver(initial, InputDriver::Regression),
        Err(RunError::NotFrozen)
    );
    freeze(&mut run, 0, WorkBudget::one());
    let before = run.observation();
    let changed = run.select_driver(initial, InputDriver::Regression).unwrap();
    assert_eq!(changed.input.get(), initial.input.get() + 1);
    assert_eq!(changed.transport, initial.transport);
    assert_eq!(run.observation().status, RunStatus::Frozen);
    assert_eq!(
        run.observation().committed_judgements,
        before.committed_judgements
    );
    assert_eq!(
        run.select_driver(changed, InputDriver::Regression),
        Ok(changed)
    );
    let active = run.resume(changed).unwrap();
    let press = input(1, 1_000_000, InputEdge::Press(Point::new(30, 0).unwrap()));
    assert_eq!(
        run.submit_driver_inputs(active, InputDriver::Regression, &[press]),
        Err(RunError::RearmRequired)
    );
    assert_eq!(
        run.observe_driver_release(initial, InputDriver::Manual, fact(1)),
        Err(RunError::StaleSource)
    );
    assert_eq!(
        run.observe_release(active, fact(1)),
        Err(RunError::DriverMismatch)
    );
    assert_eq!(run.observation().armed_slots, [false; 4]);
    run.observe_driver_release(active, InputDriver::Regression, fact(1))
        .unwrap();
    run.submit_driver_inputs(
        active,
        InputDriver::Regression,
        &[InputEvent {
            sequence: InputSequence::new(2).unwrap(),
            ..press
        }],
    )
    .unwrap();
    let mut events = Vec::new();
    advance(&mut run, 1_000_000, WorkBudget::one(), &mut events);
    assert_eq!(
        run.observation().committed_judgements[1].unwrap().result,
        Judgement::Hit
    );
}

#[test]
fn switching_away_and_back_cannot_retag_cancelled_future_auto_input_as_new() {
    let mut run = selected(
        &[0, 1_000_000, 2_000_000],
        Limits::tiny(),
        InputDriver::Automatic,
    );
    prepare(&mut run, 1_000_001, WorkBudget::one());
    freeze(&mut run, 0, WorkBudget::one());
    let mut events = Vec::new();
    collect(&mut run, &mut events);
    let source = run
        .select_driver(run.source(), InputDriver::Regression)
        .unwrap();
    run.resume(source).unwrap();
    freeze(&mut run, 0, WorkBudget::one());
    let source = run
        .select_driver(run.source(), InputDriver::Automatic)
        .unwrap();
    run.resume(source).unwrap();
    drive(&mut run, 2_300_000, WorkBudget::one(), &mut events);
    assert_eq!(
        run.observation().committed_judgements[1].unwrap().result,
        Judgement::Miss
    );
    assert_eq!(
        run.observation().committed_judgements[2].unwrap().result,
        Judgement::Hit
    );
}

#[test]
fn automatic_inspection_keeps_driver_but_only_generates_strictly_after_closed_target() {
    for at in [999_999, 1_000_000, 1_000_001] {
        let mut run = selected(&[0, 1_000_000], Limits::tiny(), InputDriver::Automatic);
        freeze(&mut run, -200_001, WorkBudget::one());
        let old = run.source();
        let mut cursor = run.request_inspection(old, time(at)).unwrap();
        assert_eq!(
            run.select_driver(old, InputDriver::Regression),
            Err(RunError::Busy)
        );
        for _ in 0..8 {
            match run.step_inspection(cursor, WorkBudget::one()).unwrap() {
                InspectionReport::Pending { cursor: next, .. } => cursor = next,
                InspectionReport::Ready { key, .. } => {
                    run.activate_inspection(key).unwrap();
                    break;
                }
            }
        }
        assert_eq!(run.driver(), InputDriver::Automatic);
        assert_eq!(run.observation().status, RunStatus::Frozen);
        run.resume(run.source()).unwrap();
        assert_eq!(
            run.prepare_automatic_inputs(old, time(1_000_002), WorkBudget::one()),
            Err(RunError::StaleSource)
        );
        let mut events = Vec::new();
        drive(&mut run, 1_000_002, WorkBudget::one(), &mut events);
        assert_eq!(
            run.observation().committed_judgements[1].is_some(),
            at < 1_000_000
        );
        assert_eq!(events.len(), if at < 1_000_000 { 2 } else { 0 });
        assert_eq!(
            run.observation().statistics_origin,
            StatisticsOrigin::Inspection { at: time(at) }
        );
    }
}

#[test]
fn wrong_source_components_are_checked_before_driver_time_capacity_and_rearming() {
    let mut run = selected(&[0], Limits::tiny(), InputDriver::Automatic);
    let base = run.source();
    let before = run.observation();
    for wrong in [
        RunSource {
            run: RunId::new(99).unwrap(),
            ..base
        },
        RunSource {
            transport: TransportEpoch::new(2).unwrap(),
            ..base
        },
        RunSource {
            input: InputEpoch::new(99).unwrap(),
            ..base
        },
        RunSource {
            revision: MapRevision::new(99),
            ..base
        },
    ] {
        assert_eq!(
            run.prepare_automatic_inputs(wrong, time(i64::MIN), WorkBudget::full()),
            Err(RunError::StaleSource)
        );
        assert_eq!(
            run.submit_driver_inputs(
                wrong,
                InputDriver::Regression,
                &[input(1, i64::MIN, InputEdge::Release); 65]
            ),
            Err(RunError::StaleSource)
        );
        assert_eq!(
            run.observe_driver_release(wrong, InputDriver::Automatic, fact(1)),
            Err(RunError::StaleSource)
        );
        assert_eq!(
            run.select_driver(wrong, InputDriver::Regression),
            Err(RunError::StaleSource)
        );
        assert_eq!(run.observation(), before);
    }
}

#[test]
fn unsupported_pulse_capacity_and_epoch_exhaustion_reject_switch_atomically() {
    let mut run = fixture(
        &[0],
        -200_001,
        Limits::try_new(1, 2, 1, 1, 64 * 1024).unwrap(),
    );
    freeze(&mut run, -200_001, WorkBudget::one());
    let before = run.observation();
    assert_eq!(
        run.select_driver(run.source(), InputDriver::Automatic),
        Err(RunError::AutomaticCapacity)
    );
    assert_eq!(run.observation(), before);
    let source = RunSource {
        input: InputEpoch::new(u64::MAX).unwrap(),
        ..source(90_002)
    };
    let map = SyntheticMap::try_new(source.revision, &[]).unwrap();
    let mut run = SyntheticRun::new(source, map, time(0), Limits::tiny()).unwrap();
    freeze(&mut run, 0, WorkBudget::one());
    let before = run.observation();
    assert_eq!(
        run.select_driver(source, InputDriver::Automatic),
        Err(RunError::CounterExhausted)
    );
    assert_eq!(run.observation(), before);
    assert_eq!(run.select_driver(source, InputDriver::Manual), Ok(source));
}

#[test]
fn empty_maps_and_signed_time_endpoints_do_not_need_sentinels_or_wrapping() {
    for (targets, start, through) in [
        (vec![], i64::MIN, i64::MAX),
        (vec![i64::MIN + 200_000], i64::MIN, i64::MIN + 200_001),
        (vec![i64::MAX - 300_000], i64::MAX - 500_001, i64::MAX),
    ] {
        let mut run = fixture(&targets, start, Limits::tiny());
        freeze(&mut run, start, WorkBudget::one());
        let source = run
            .select_driver(run.source(), InputDriver::Automatic)
            .unwrap();
        run.resume(source).unwrap();
        let mut events = Vec::new();
        drive(&mut run, through, WorkBudget::one(), &mut events);
        assert_eq!(events.len(), targets.len() * 2);
        assert_eq!(run.observation().completed_through, Some(time(through)));
    }
}

fn complete(task: &mut PreparationTask<'_>) {
    for _ in 0..128 {
        if matches!(
            task.step(task.cursor(), WorkBudget::one()).unwrap(),
            PreparationReport::Ready { .. }
        ) {
            return;
        }
    }
    panic!("coordinator fixture bound");
}

#[test]
fn coordinated_inspection_inherits_driver_load_defaults_manual_and_both_reject_old_auto_sources() {
    let mut owner = SyntheticCoordinator::new(
        CoordinatorId::new(90_100).unwrap(),
        Limits::tiny(),
        PreparationLimits::tiny(),
    )
    .unwrap();
    let input = circles(&[0, 1_000_000]);
    let mut task = owner
        .request_create(source(90_101), &input, time(-200_001))
        .unwrap();
    complete(&mut task);
    owner.activate(task.key(), &mut task).unwrap();
    drop(task);
    let source = owner.current().unwrap().source();
    owner.request_freeze(source, time(-200_001)).unwrap();
    owner.seal_through(source, time(-200_001)).unwrap();
    for _ in 0..128 {
        if matches!(
            owner.settle_freeze(source, WorkBudget::one()).unwrap(),
            FreezeReport::Frozen { .. }
        ) {
            break;
        }
    }
    let source = owner.select_driver(source, InputDriver::Automatic).unwrap();
    let source = owner.resume(source).unwrap();
    let usage = owner.resources();
    for _ in 0..128 {
        if matches!(
            owner
                .prepare_automatic_inputs(source, time(0), WorkBudget::one())
                .unwrap(),
            AutomaticReport::Complete { .. }
        ) {
            break;
        }
    }
    assert_eq!(owner.resources(), usage);
    assert_eq!(owner.current().unwrap().observation().pending_inputs, 2);
    owner.request_freeze(source, time(0)).unwrap();
    owner.seal_through(source, time(0)).unwrap();
    for _ in 0..128 {
        if matches!(
            owner.settle_freeze(source, WorkBudget::one()).unwrap(),
            FreezeReport::Frozen { .. }
        ) {
            break;
        }
    }
    assert_eq!(
        owner.current().unwrap().observation().committed_judgements[0]
            .unwrap()
            .result,
        Judgement::Hit
    );
    let mut task = owner.request_inspection(source, time(1_000_000)).unwrap();
    assert_eq!(task.key().driver(), InputDriver::Automatic);
    assert_eq!(
        owner.select_driver(source, InputDriver::Regression),
        Err(PreparationError::Busy)
    );
    complete(&mut task);
    let inspected = owner.activate(task.key(), &mut task).unwrap();
    drop(task);
    assert_eq!(owner.current().unwrap().driver(), InputDriver::Automatic);
    assert_eq!(
        owner.current().unwrap().observation().status,
        RunStatus::Frozen
    );
    assert_eq!(
        owner.prepare_automatic_inputs(source, time(1_000_001), WorkBudget::one()),
        Err(PreparationError::Runtime(RunError::StaleSource))
    );
    let replacement = circles(&[2_000_000]);
    let mut task = owner
        .request_load(
            inspected,
            crate::source(90_102),
            &replacement,
            time(1_799_999),
        )
        .unwrap();
    assert_eq!(task.key().driver(), InputDriver::Manual);
    complete(&mut task);
    let loaded = owner.activate(task.key(), &mut task).unwrap();
    drop(task);
    assert_eq!(owner.current().unwrap().driver(), InputDriver::Manual);
    assert_eq!(
        owner.current().unwrap().observation().status,
        RunStatus::Prepared
    );
    assert_eq!(
        owner.prepare_automatic_inputs(inspected, time(2_000_000), WorkBudget::one()),
        Err(PreparationError::Runtime(RunError::StaleSource))
    );
    owner
        .submit_inputs(loaded, &[input_event_for_loaded()])
        .unwrap();
}

fn input_event_for_loaded() -> InputEvent {
    input(1, 2_000_000, InputEdge::Press(Point::new(0, 0).unwrap()))
}
