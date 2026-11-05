#[allow(dead_code)]
mod common;
use common::SnapshotExt;
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
        let before = run.snapshot();
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
            assert_eq!(run.snapshot(), before);
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
    let before = run.snapshot();
    assert_eq!(
        run.seal_through(source, time(0)),
        Err(RunError::AutomaticInputsPending)
    );
    assert_eq!(
        run.prepare_automatic_inputs(source, time(0), WorkBudget::full()),
        Err(RunError::BudgetLimit)
    );
    assert_eq!(run.snapshot(), before);
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
    let prepared = run.snapshot();
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
        assert_eq!(run.snapshot(), prepared);
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
    assert_eq!(a.snapshot(), b.snapshot());
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
    assert_eq!(run.snapshot().pending_inputs, 0);
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
    assert_eq!(run.snapshot().freeze.unwrap().cancelled_inputs, 0);
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
    let before = run.snapshot();
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
        assert_eq!(run.snapshot(), before);
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
        let closed = run.snapshot();
        assert_eq!(
            run.prepare_automatic_inputs(old, time(i64::MAX), WorkBudget::one()),
            Err(RunError::GateClosed)
        );
        assert_eq!(run.snapshot(), closed);
        freeze(&mut run, 0, WorkBudget::one());
        assert_eq!(run.snapshot().committed_judgements[0].is_some(), cut == 3);
        let new = run.resume(old).unwrap();
        let before = run.snapshot();
        assert_eq!(
            run.prepare_automatic_inputs(old, time(1), WorkBudget::one()),
            Err(RunError::StaleSource)
        );
        assert_eq!(
            run.observe_driver_release(old, InputDriver::Automatic, fact(1)),
            Err(RunError::StaleSource)
        );
        assert_eq!(run.snapshot(), before);
        assert_ne!(new.input, old.input);
        prepare(&mut run, 1, WorkBudget::one());
        assert_eq!(run.snapshot().pending_inputs, 0);
        let mut events = Vec::new();
        drive(&mut run, 100_001, WorkBudget::one(), &mut events);
        assert_eq!(
            run.snapshot().committed_judgements[0].unwrap().result,
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
    assert_eq!(run.snapshot().freeze.unwrap().cancelled_inputs, 3);
    let sound = run.event_at(1).unwrap();
    assert!(!run.sound_intent_allowed(sound.key).unwrap());
    run.resume(run.source()).unwrap();
    assert!(!run.sound_intent_allowed(sound.key).unwrap());
    let mut events = Vec::new();
    drive(&mut run, 2_300_000, WorkBudget::one(), &mut events);
    let state = run.snapshot();
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
    let before = run.snapshot();
    let changed = run.select_driver(initial, InputDriver::Regression).unwrap();
    assert_eq!(changed.input.get(), initial.input.get() + 1);
    assert_eq!(changed.transport, initial.transport);
    assert_eq!(run.snapshot().status, RunStatus::Frozen);
    assert_eq!(
        run.snapshot().committed_judgements,
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
    assert_eq!(run.snapshot().armed_slots, [false; 4]);
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
        run.snapshot().committed_judgements[1].unwrap().result,
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
        run.snapshot().committed_judgements[1].unwrap().result,
        Judgement::Miss
    );
    assert_eq!(
        run.snapshot().committed_judgements[2].unwrap().result,
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
        assert_eq!(run.snapshot().status, RunStatus::Frozen);
        run.resume(run.source()).unwrap();
        assert_eq!(
            run.prepare_automatic_inputs(old, time(1_000_002), WorkBudget::one()),
            Err(RunError::StaleSource)
        );
        let mut events = Vec::new();
        drive(&mut run, 1_000_002, WorkBudget::one(), &mut events);
        assert_eq!(
            run.snapshot().committed_judgements[1].is_some(),
            at < 1_000_000
        );
        assert_eq!(events.len(), if at < 1_000_000 { 2 } else { 0 });
        assert_eq!(
            run.snapshot().statistics_origin,
            StatisticsOrigin::Inspection { at: time(at) }
        );
    }
}

#[test]
fn wrong_source_components_are_checked_before_driver_time_capacity_and_rearming() {
    let mut run = selected(&[0], Limits::tiny(), InputDriver::Automatic);
    let base = run.source();
    let before = run.snapshot();
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
        assert_eq!(run.snapshot(), before);
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
    let before = run.snapshot();
    assert_eq!(
        run.select_driver(run.source(), InputDriver::Automatic),
        Err(RunError::AutomaticCapacity)
    );
    assert_eq!(run.snapshot(), before);
    let source = RunSource {
        input: InputEpoch::new(u64::MAX).unwrap(),
        ..source(90_002)
    };
    let map = SyntheticMap::try_new(source.revision, &[]).unwrap();
    let mut run = SyntheticRun::new(source, map, time(0), Limits::tiny()).unwrap();
    freeze(&mut run, 0, WorkBudget::one());
    let before = run.snapshot();
    assert_eq!(
        run.select_driver(source, InputDriver::Automatic),
        Err(RunError::CounterExhausted)
    );
    assert_eq!(run.snapshot(), before);
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
        assert_eq!(run.snapshot().completed_through, Some(time(through)));
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
    assert_eq!(owner.current().unwrap().snapshot().pending_inputs, 2);
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
        owner.current().unwrap().snapshot().committed_judgements[0]
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
        owner.current().unwrap().snapshot().status,
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
        owner.current().unwrap().snapshot().status,
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

fn freeze_with_retained_seal(run: &mut SyntheticRun, at: i64, budget: WorkBudget) {
    let source = run.source();
    run.request_freeze(source, time(at)).unwrap();
    if run
        .snapshot()
        .sealed_through
        .is_none_or(|old| old < time(at))
    {
        run.seal_through(source, time(at)).unwrap();
    }
    for _ in 0..256 {
        match run.settle_freeze(source, budget).unwrap() {
            FreezeReport::Frozen { .. } => return,
            FreezeReport::Pending { reason, steps, .. } => {
                assert_eq!(reason, PendingReason::WorkBudget);
                assert!(steps <= budget.get());
            }
        }
    }
    panic!("retained seal freeze bound");
}

fn fill_with_pending_plan(run: &mut SyntheticRun) {
    for _ in 0..128 {
        if let AutomaticReport::Pending {
            reason: AutomaticPendingReason::InputBackpressure,
            ready_through,
            ..
        } = run
            .prepare_automatic_inputs(run.source(), time(3_000_001), WorkBudget::one())
            .unwrap()
        {
            assert_eq!(ready_through, Some(time(1_000_001)));
            assert_eq!(run.snapshot().pending_inputs, 4);
            return;
        }
    }
    panic!("automatic backpressure bound");
}

#[test]
fn freezing_at_each_advance_yield_keeps_the_auto_prefix_and_invalidates_old_controls() {
    let mut passed_boundary = false;
    let mut checked = 0;
    for cut in 0..128 {
        let mut run = selected(
            &[0, 1_000_000, 2_000_000, 3_000_000],
            Limits::tiny(),
            InputDriver::Automatic,
        );
        fill_with_pending_plan(&mut run);
        let old = run.source();
        run.seal_through(old, time(1_000_001)).unwrap();
        let mut cursor = None;
        for step in 0..cut {
            let report = if step == 0 {
                run.advance(old, time(1_000_001), WorkBudget::one())
                    .unwrap()
            } else {
                run.continue_advance(cursor.unwrap(), WorkBudget::one())
                    .unwrap()
            };
            let AdvanceReport::Pending { cursor: next, .. } = report else {
                panic!("full event queue must retain bounded progress");
            };
            cursor = Some(next);
        }
        if run
            .snapshot()
            .last_committed_time
            .is_some_and(|at| at > time(0))
        {
            passed_boundary = true;
            break;
        }
        checked += 1;
        run.request_freeze(old, time(0)).unwrap();
        let closed = run.snapshot();
        assert_eq!(
            run.prepare_automatic_inputs(old, time(3_000_001), WorkBudget::one()),
            Err(RunError::GateClosed)
        );
        if let Some(cursor) = cursor {
            assert_eq!(
                run.continue_advance(cursor, WorkBudget::one()),
                Err(RunError::StaleCursor)
            );
            assert_eq!(
                run.cancel_request(cursor.request()),
                Err(RunError::StaleCursor)
            );
        }
        assert_eq!(run.snapshot(), closed);
        freeze_with_retained_seal(&mut run, 0, WorkBudget::one());
        let frozen = run.snapshot();
        assert_eq!(frozen.completed_through, Some(time(0)));
        assert_eq!(frozen.sealed_through, Some(time(1_000_001)));
        assert_eq!(frozen.freeze.unwrap().cancelled_inputs, 3);
        assert_eq!(
            frozen.committed_judgements[0].unwrap().result,
            Judgement::Hit
        );
        run.resume(old).unwrap();
        let resumed = run.snapshot();
        assert_eq!(
            run.prepare_automatic_inputs(old, time(3_300_000), WorkBudget::one()),
            Err(RunError::StaleSource)
        );
        assert_eq!(
            run.observe_release(old, fact(1)),
            Err(RunError::StaleSource)
        );
        if let Some(cursor) = cursor {
            assert_eq!(
                run.continue_advance(cursor, WorkBudget::one()),
                Err(RunError::StaleSource)
            );
        }
        assert_eq!(run.snapshot(), resumed);
        let mut events = Vec::new();
        drive(&mut run, 3_300_000, WorkBudget::one(), &mut events);
        let results: Vec<_> = run
            .snapshot()
            .committed_judgements
            .iter()
            .flatten()
            .map(|record| record.result)
            .collect();
        assert_eq!(
            results,
            [
                Judgement::Hit,
                Judgement::Miss,
                Judgement::Hit,
                Judgement::Hit
            ]
        );
        assert_eq!(events.len(), 7);
        for (index, event) in events.iter().enumerate() {
            assert_eq!(event.key.sequence().get(), index as u64 + 1);
        }
    }
    assert!(
        passed_boundary && checked >= 4,
        "all yields before and through the first hit must be checked"
    );
}

#[test]
fn event_backpressure_cannot_turn_an_automatic_freeze_into_half_a_hit_or_replay() {
    let mut run = selected(
        &[0, 1_000_000, 2_000_000, 3_000_000],
        Limits::tiny(),
        InputDriver::Automatic,
    );
    fill_with_pending_plan(&mut run);
    let old = run.source();
    run.seal_through(old, time(1_000_001)).unwrap();
    let mut report = run
        .advance(old, time(1_000_001), WorkBudget::one())
        .unwrap();
    let mut blocked = None;
    for _ in 0..256 {
        let AdvanceReport::Pending { cursor, reason, .. } = report else {
            panic!("events must backpressure");
        };
        if reason == PendingReason::EventBackpressure {
            blocked = Some(cursor);
            break;
        }
        report = run.continue_advance(cursor, WorkBudget::one()).unwrap();
    }
    let blocked = blocked.expect("second automatic hit must wait for two event slots");
    let judgement = run.event_at(0).unwrap();
    let sound = run.event_at(1).unwrap();
    run.request_freeze(old, time(1_000_000)).unwrap();
    let mut pending = false;
    for _ in 0..16 {
        if matches!(
            run.settle_freeze(old, WorkBudget::one()).unwrap(),
            FreezeReport::Pending {
                reason: PendingReason::EventBackpressure,
                ..
            }
        ) {
            pending = true;
            break;
        }
    }
    assert!(pending);
    assert_eq!(run.snapshot().status, RunStatus::Freezing);
    assert_eq!(run.snapshot().committed_judgements[1], None);
    assert!(!run.sound_intent_allowed(sound.key).unwrap());
    run.acknowledge(judgement.key).unwrap();
    assert!(matches!(
        run.settle_freeze(old, WorkBudget::one()).unwrap(),
        FreezeReport::Pending {
            reason: PendingReason::EventBackpressure,
            ..
        }
    ));
    assert_eq!(run.snapshot().committed_judgements[1], None);
    run.acknowledge(sound.key).unwrap();
    freeze_with_retained_seal(&mut run, 1_000_000, WorkBudget::one());
    assert_eq!(run.snapshot().freeze.unwrap().cancelled_inputs, 1);
    assert_eq!(
        run.snapshot().committed_judgements[1].unwrap().result,
        Judgement::Hit
    );
    run.resume(old).unwrap();
    assert_eq!(
        run.continue_advance(blocked, WorkBudget::one()),
        Err(RunError::StaleSource)
    );
    let mut events = vec![judgement, sound];
    drive(&mut run, 3_300_000, WorkBudget::one(), &mut events);
    assert_eq!(events.len(), 8);
    assert!(
        run.snapshot()
            .committed_judgements
            .iter()
            .flatten()
            .all(|record| record.result == Judgement::Hit)
    );
    for (index, event) in events.iter().enumerate() {
        assert_eq!(event.key.sequence().get(), index as u64 + 1);
    }
}

#[test]
fn every_external_slot_rearms_only_from_current_driver_release_and_preserves_time_floor() {
    for driver in [InputDriver::Manual, InputDriver::Regression] {
        for (index, slot) in [
            PhysicalSlot::PrimaryOne,
            PhysicalSlot::PrimaryTwo,
            PhysicalSlot::SecondaryOne,
            PhysicalSlot::SecondaryTwo,
        ]
        .into_iter()
        .enumerate()
        {
            let mut run = selected(&[1_000_000], Limits::tiny(), driver);
            let old = run.source();
            run.observe_driver_release(old, driver, ReleaseObservation { slot, ..fact(1) })
                .unwrap();
            run.submit_driver_inputs(
                old,
                driver,
                &[InputEvent {
                    slot,
                    ..input(2, 0, InputEdge::Press(Point::new(0, 0).unwrap()))
                }],
            )
            .unwrap();
            freeze(&mut run, 0, WorkBudget::one());
            let other = if driver == InputDriver::Manual {
                InputDriver::Regression
            } else {
                InputDriver::Manual
            };
            let intermediate = run.select_driver(old, other).unwrap();
            let switched = run.select_driver(intermediate, driver).unwrap();
            let active = run.resume(switched).unwrap();
            let before = run.snapshot();
            let press = InputEvent {
                slot,
                ..input(2, 1_000_000, InputEdge::Press(Point::new(0, 0).unwrap()))
            };
            assert_eq!(
                run.submit_driver_inputs(active, driver, &[press]),
                Err(RunError::RearmRequired)
            );
            for stale in [old, intermediate, switched] {
                assert_eq!(
                    run.observe_driver_release(
                        stale,
                        driver,
                        ReleaseObservation { slot, ..fact(1) }
                    ),
                    Err(RunError::StaleSource)
                );
                assert_eq!(
                    run.submit_driver_inputs(stale, driver, &[press]),
                    Err(RunError::StaleSource)
                );
            }
            assert_eq!(
                run.observe_driver_release(active, other, ReleaseObservation { slot, ..fact(1) }),
                Err(RunError::DriverMismatch)
            );
            let future_release = InputEvent {
                slot,
                ..input(1, 1_000_001, InputEdge::Release)
            };
            assert_eq!(
                run.submit_driver_inputs(active, driver, &[future_release, press]),
                Err(RunError::RearmRequired)
            );
            assert_eq!(run.snapshot(), before);
            run.submit_driver_inputs(
                active,
                driver,
                &[
                    InputEvent {
                        at: time(1_000_000),
                        ..future_release
                    },
                    press,
                ],
            )
            .unwrap();
            let mut armed = [false; 4];
            armed[index] = true;
            assert_eq!(run.snapshot().armed_slots, armed);
            let mut events = Vec::new();
            advance(&mut run, 1_000_000, WorkBudget::one(), &mut events);
            assert_eq!(events.len(), 2);
            assert_eq!(events[0].kind, EventKind::Judgement(Judgement::Hit));
        }
    }
}

#[test]
fn identical_admitted_auto_sets_keep_the_same_frozen_and_resumed_results_across_budgets() {
    let mut runs = Vec::new();
    for budget in [WorkBudget::one(), WorkBudget::full()] {
        let mut run = selected(
            &[0, 1_000_000, 2_000_000, 3_000_000],
            Limits::reference(),
            InputDriver::Automatic,
        );
        prepare(&mut run, 3_000_001, budget);
        assert_eq!(run.snapshot().pending_inputs, 8);
        freeze_with_retained_seal(&mut run, 1_000_000, budget);
        assert_eq!(run.snapshot().freeze.unwrap().cancelled_inputs, 5);
        let frozen = run.snapshot();
        run.resume(run.source()).unwrap();
        let mut events = Vec::new();
        drive(&mut run, 3_300_000, budget, &mut events);
        assert_eq!(events.len(), 6);
        let results: Vec<_> = run
            .snapshot()
            .committed_judgements
            .iter()
            .flatten()
            .map(|record| record.result)
            .collect();
        assert_eq!(
            results,
            [
                Judgement::Hit,
                Judgement::Hit,
                Judgement::Miss,
                Judgement::Miss
            ]
        );
        runs.push((frozen, run.snapshot(), events));
    }
    assert_eq!(runs[0], runs[1]);
}

fn freeze_owner(owner: &mut SyntheticCoordinator, at: i64) {
    let source = owner.current().unwrap().source();
    owner.request_freeze(source, time(at)).unwrap();
    if owner
        .current()
        .unwrap()
        .snapshot()
        .sealed_through
        .is_none_or(|old| old < time(at))
    {
        owner.seal_through(source, time(at)).unwrap();
    }
    for _ in 0..256 {
        if matches!(
            owner.settle_freeze(source, WorkBudget::one()).unwrap(),
            FreezeReport::Frozen { .. }
        ) {
            return;
        }
    }
    panic!("owner freeze bound");
}

#[test]
fn cancelled_automatic_candidates_cannot_cross_a_driver_switch_even_with_same_next_transport() {
    let costs = SyntheticCoordinator::costs();
    let limits = PreparationLimits::try_new(
        costs.source_bytes,
        costs.pool_bytes + costs.coordinator_bytes + 3 * costs.run_bytes + 2 * costs.task_bytes,
        1,
    )
    .unwrap();
    let input = circles(&[0, 1_000_000]);
    for ready in [false, true] {
        let mut owner =
            SyntheticCoordinator::new(CoordinatorId::new(92_100).unwrap(), Limits::tiny(), limits)
                .unwrap();
        let mut create = owner
            .request_create(source(92_101), &input, time(-200_001))
            .unwrap();
        complete(&mut create);
        owner.activate(create.key(), &mut create).unwrap();
        drop(create);
        freeze_owner(&mut owner, -200_001);
        let base = owner.current().unwrap().source();
        let base = owner.select_driver(base, InputDriver::Automatic).unwrap();
        owner.resume(base).unwrap();
        freeze_owner(&mut owner, -200_001);
        let old = owner.current().unwrap().source();
        let mut a = owner.request_inspection(old, time(1_000_000)).unwrap();
        if ready {
            complete(&mut a);
        } else {
            a.step(a.cursor(), WorkBudget::one()).unwrap();
        }
        let cursor = a.cursor();
        let held = owner.resources();
        owner.cancel(a.key()).unwrap();
        assert_eq!(owner.resources(), held);
        let base = owner.select_driver(old, InputDriver::Regression).unwrap();
        let mut b = owner.request_inspection(base, time(1_000_000)).unwrap();
        assert_eq!(a.key().target().transport, b.key().target().transport);
        assert_ne!(a.key().target().input, b.key().target().input);
        assert_eq!(a.key().driver(), InputDriver::Automatic);
        assert_eq!(b.key().driver(), InputDriver::Regression);
        complete(&mut b);
        let before = owner.current().unwrap().snapshot();
        assert_eq!(
            a.step(cursor, WorkBudget::one()),
            Err(PreparationError::StaleRequest)
        );
        assert_eq!(
            owner.activate(a.key(), &mut a),
            Err(PreparationError::StaleRequest)
        );
        assert_eq!(
            owner.activate(b.key(), &mut a),
            Err(PreparationError::StaleRequest)
        );
        assert_eq!(owner.current().unwrap().snapshot(), before);
        let held = owner.resources();
        drop(a);
        assert_eq!(owner.candidate(), Some(b.key()));
        assert_eq!(
            owner.resources().runtime_bytes,
            held.runtime_bytes - costs.task_bytes - costs.run_bytes
        );
        assert_eq!(owner.resources().precompiled_bytes, costs.source_bytes);
        let activated = owner.activate(b.key(), &mut b).unwrap();
        drop(b);
        assert_eq!(owner.current().unwrap().driver(), InputDriver::Regression);
        let active = owner.resume(activated).unwrap();
        assert_eq!(
            owner.prepare_automatic_inputs(old, time(1_000_001), WorkBudget::one()),
            Err(PreparationError::Runtime(RunError::StaleSource))
        );
        assert_eq!(
            owner.observe_driver_release(old, InputDriver::Automatic, fact(1)),
            Err(PreparationError::Runtime(RunError::StaleSource))
        );
        assert_eq!(owner.current().unwrap().snapshot().armed_slots, [false; 4]);
        owner
            .observe_driver_release(active, InputDriver::Regression, fact(1))
            .unwrap();
        owner
            .submit_driver_inputs(
                active,
                InputDriver::Regression,
                &[crate::input(
                    2,
                    1_000_001,
                    InputEdge::Press(Point::new(30, 0).unwrap()),
                )],
            )
            .unwrap();
        owner.seal_through(active, time(1_000_001)).unwrap();
        let mut report = owner
            .advance(active, time(1_000_001), WorkBudget::one())
            .unwrap();
        for _ in 0..256 {
            match report {
                AdvanceReport::Complete { .. } => break,
                AdvanceReport::Pending { cursor, reason, .. } => {
                    assert_eq!(reason, PendingReason::WorkBudget);
                    report = owner.continue_advance(cursor, WorkBudget::one()).unwrap();
                }
            }
        }
        assert_eq!(
            owner.current().unwrap().snapshot().completed_through,
            Some(time(1_000_001))
        );
        assert_eq!(
            owner.current().unwrap().snapshot().committed_judgements[1]
                .unwrap()
                .result,
            Judgement::Hit
        );
    }
}
