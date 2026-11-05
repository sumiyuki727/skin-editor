#[allow(dead_code)]
mod common;
use common::SnapshotExt;
use skin_editor_gameplay::*;

fn time(value: i64) -> MapTime {
    MapTime::from_micros(value)
}
fn run(id: u128, targets: &[i64], limits: Limits) -> SyntheticRun {
    let source = RunSource {
        run: RunId::new(id).unwrap(),
        transport: TransportEpoch::new(1).unwrap(),
        input: InputEpoch::new(1).unwrap(),
        revision: MapRevision::new(7),
    };
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
    let map = SyntheticMap::try_new(source.revision, &circles).unwrap();
    let start = targets.first().map_or(0, |at| at - 200_001);
    SyntheticRun::new(source, map, time(start), limits).unwrap()
}
fn press(sequence: u64, at: i64) -> InputEvent {
    InputEvent {
        sequence: InputSequence::new(sequence).unwrap(),
        at: time(at),
        slot: PhysicalSlot::PrimaryOne,
        edge: InputEdge::Press(Point::new(0, 0).unwrap()),
    }
}
fn freeze(run: &mut SyntheticRun, at: i64, budget: WorkBudget) {
    let source = run.source();
    run.request_freeze(source, time(at)).unwrap();
    run.seal_through(source, time(at)).unwrap();
    for _ in 0..256 {
        match run.settle_freeze(source, budget).unwrap() {
            FreezeReport::Frozen { steps, .. } => {
                assert!(steps <= budget.get());
                return;
            }
            FreezeReport::Pending { reason, steps, .. } => {
                assert_eq!(reason, PendingReason::WorkBudget);
                assert!(steps <= budget.get());
            }
        }
    }
    panic!("freeze exceeded fixture bound");
}
fn prepare(run: &mut SyntheticRun, target: i64, budget: WorkBudget) -> InspectionKey {
    let mut cursor = run.request_inspection(run.source(), time(target)).unwrap();
    for _ in 0..16 {
        match run.step_inspection(cursor, budget).unwrap() {
            InspectionReport::Ready { key, at, steps } => {
                assert_eq!(at, time(target));
                assert!(steps > 0 && steps <= budget.get());
                return key;
            }
            InspectionReport::Pending {
                cursor: next,
                steps,
                ..
            } => {
                assert!(steps > 0 && steps <= budget.get());
                cursor = next;
            }
        }
    }
    panic!("inspection exceeded fixture bound");
}
fn inspect(run: &mut SyntheticRun, target: i64, budget: WorkBudget) -> RunSource {
    let key = prepare(run, target, budget);
    run.activate_inspection(key).unwrap()
}
fn advance(run: &mut SyntheticRun, target: i64, budget: WorkBudget) {
    run.seal_through(run.source(), time(target)).unwrap();
    let mut result = run.advance(run.source(), time(target), budget).unwrap();
    for _ in 0..256 {
        match result {
            AdvanceReport::Complete { through, .. } => {
                assert_eq!(through, time(target));
                return;
            }
            AdvanceReport::Pending { cursor, reason, .. } => {
                assert_eq!(reason, PendingReason::WorkBudget);
                result = run.continue_advance(cursor, budget).unwrap();
            }
        }
    }
    panic!("advance exceeded fixture bound");
}
fn rearm(run: &mut SyntheticRun) {
    run.observe_release(
        run.source(),
        ReleaseObservation {
            sequence: InputSequence::new(1).unwrap(),
            slot: PhysicalSlot::PrimaryOne,
        },
    )
    .unwrap();
}

#[test]
fn inspection_boundaries_are_closed_without_fake_judgements_or_sounds() {
    for (index, target) in [
        i64::MIN,
        -1,
        799_999,
        800_000,
        900_000,
        1_000_000,
        1_100_000,
        1_100_001,
        1_299_999,
        1_300_000,
        i64::MAX,
    ]
    .into_iter()
    .enumerate()
    {
        let mut run = run(50_000 + index as u128, &[1_000_000], Limits::tiny());
        freeze(&mut run, 799_999, WorkBudget::one());
        let old = run.source();
        let source = inspect(&mut run, target, WorkBudget::one());
        assert_eq!(source.transport.get(), old.transport.get() + 1);
        assert_eq!(source.input.get(), old.input.get() + 1);
        assert_eq!(source.run, old.run);
        assert_eq!(source.revision, old.revision);
        let state = run.snapshot();
        assert_eq!(state.status, RunStatus::Frozen);
        assert_eq!(state.completed_through, Some(time(target)));
        assert_eq!(state.last_committed_time, Some(time(target)));
        assert_eq!(state.sealed_through, Some(time(target)));
        assert_eq!(
            state.statistics_origin,
            StatisticsOrigin::Inspection { at: time(target) }
        );
        assert_eq!(state.committed_judgements, [None; MAX_CIRCLES]);
        assert_eq!(state.pending_inputs, 0);
        assert_eq!(state.pending_events, 0);
        assert_eq!(state.armed_slots, [false; 4]);
        assert_eq!(run.event_at(0), None);
        let frame = state.ready_frame.unwrap();
        assert_eq!(frame.source, source);
        assert_eq!(frame.at, time(target));
        assert_eq!(frame.statistics_origin, state.statistics_origin);
        let circle = frame.object(0).unwrap();
        assert_eq!(circle.judgement, None);
        assert_eq!(
            circle.unavailable,
            (target > 1_100_000).then_some(UnavailableReason::InspectionSkipped)
        );
        assert_eq!(
            circle.phase,
            if target > 1_100_000 {
                Phase::Retired
            } else if target < 800_000 {
                Phase::Future
            } else {
                Phase::VisibleUnjudged
            }
        );
    }
}

#[test]
fn pending_and_ready_candidate_never_publish_a_partial_target() {
    let mut run = run(
        50_100,
        &[0, 1_000_000, 2_000_000, 3_000_000],
        Limits::tiny(),
    );
    freeze(&mut run, -200_001, WorkBudget::one());
    let before = run.snapshot();
    let mut cursor = run
        .request_inspection(run.source(), time(1_000_000))
        .unwrap();
    let key = cursor.key();
    assert_eq!(key.base(), before.source);
    assert_eq!(key.target(), time(1_000_000));
    assert_eq!(
        run.activate_inspection(key),
        Err(RunError::InspectionIncomplete)
    );
    // Allocation is a separate management step, with its own cursor progress.
    for checked in 0..=4 {
        let previous = cursor;
        let report = run.step_inspection(cursor, WorkBudget::one()).unwrap();
        let state = run.snapshot();
        assert_eq!(state.ready_frame, before.ready_frame);
        assert_eq!(state.completed_through, before.completed_through);
        assert_eq!(state.source, before.source);
        assert_eq!(state.statistics_origin, before.statistics_origin);
        let info = state.inspection.unwrap();
        assert_eq!(
            (info.checked, info.total, info.ready),
            (checked, 4, checked == 4)
        );
        assert_eq!(
            run.step_inspection(previous, WorkBudget::one()),
            Err(RunError::StaleInspection)
        );
        match report {
            InspectionReport::Pending {
                cursor: next,
                steps,
                ..
            } => {
                assert_eq!(steps, 1);
                assert!(checked < 4);
                cursor = next;
            }
            InspectionReport::Ready { steps, .. } => {
                assert_eq!(steps, 1);
                assert_eq!(checked, 4);
            }
        }
    }
    run.activate_inspection(key).unwrap();
    let state = run.snapshot();
    assert_eq!(state.inspection, None);
    assert_eq!(state.completed_through, Some(time(1_000_000)));
    let frame = state.ready_frame.unwrap();
    assert_eq!(
        frame.object(0).unwrap().unavailable,
        Some(UnavailableReason::InspectionSkipped)
    );
    assert_eq!(frame.object(1).unwrap().phase, Phase::VisibleUnjudged);
    assert_eq!(frame.object(2).unwrap().phase, Phase::Future);
    assert_eq!(frame.object(3).unwrap().phase, Phase::Future);
}

#[test]
fn cancelled_candidate_cannot_reappear_before_or_after_a_replacement() {
    let mut run = run(50_101, &[0, 1_000_000], Limits::tiny());
    run.submit_inputs(run.source(), &[press(1, 0)]).unwrap();
    freeze(&mut run, 0, WorkBudget::one());
    let before = run.snapshot();
    let event = run.event_at(0).unwrap();
    let a = run.request_inspection(run.source(), time(900_000)).unwrap();
    assert!(matches!(
        run.step_inspection(a, WorkBudget::one()).unwrap(),
        InspectionReport::Pending { .. }
    ));
    run.cancel_inspection(a.key()).unwrap();
    assert_eq!(run.snapshot(), before);
    assert_eq!(run.event_at(0), Some(event));
    let b = run
        .request_inspection(run.source(), time(1_000_000))
        .unwrap();
    assert_ne!(a.key(), b.key());
    assert_eq!(
        run.step_inspection(a, WorkBudget::one()),
        Err(RunError::StaleInspection)
    );
    assert_eq!(
        run.activate_inspection(a.key()),
        Err(RunError::StaleInspection)
    );
    assert_eq!(
        run.cancel_inspection(a.key()),
        Err(RunError::StaleInspection)
    );
    let InspectionReport::Pending { cursor, .. } =
        run.step_inspection(b, WorkBudget::one()).unwrap()
    else {
        panic!("allocation only");
    };
    let InspectionReport::Pending { cursor, .. } =
        run.step_inspection(cursor, WorkBudget::one()).unwrap()
    else {
        panic!("first circle only");
    };
    assert!(matches!(
        run.step_inspection(cursor, WorkBudget::one()).unwrap(),
        InspectionReport::Ready { .. }
    ));
    assert_eq!(
        run.step_inspection(a, WorkBudget::one()),
        Err(RunError::StaleInspection)
    );
    run.activate_inspection(b.key()).unwrap();
    let after = run.snapshot();
    assert_eq!(after.committed_judgements, [None; MAX_CIRCLES]);
    assert_eq!(after.pending_events, 0);
    assert_eq!(run.activate_inspection(a.key()), Err(RunError::StaleSource));
    assert_eq!(run.cancel_inspection(b.key()), Err(RunError::StaleSource));
    assert_eq!(run.snapshot(), after);
}

#[test]
fn ready_candidate_can_be_cancelled_and_repeated_target_has_a_new_key() {
    let mut run = run(50_102, &[1_000_000], Limits::tiny());
    freeze(&mut run, 799_999, WorkBudget::one());
    let before = run.snapshot();
    let a = prepare(&mut run, 1_000_000, WorkBudget::one());
    run.cancel_inspection(a).unwrap();
    assert_eq!(run.snapshot(), before);
    let b = prepare(&mut run, 1_000_000, WorkBudget::one());
    assert_ne!(a, b);
    assert_eq!(run.activate_inspection(a), Err(RunError::StaleInspection));
    run.activate_inspection(b).unwrap();
}

#[test]
fn invalid_calls_do_not_advance_or_replace_the_candidate() {
    let mut run = run(50_103, &[0, 1_000_000], Limits::tiny());
    let source = run.source();
    assert_eq!(
        run.request_inspection(source, time(0)),
        Err(RunError::NotFrozen)
    );
    freeze(&mut run, -200_001, WorkBudget::one());
    let cursor = run.request_inspection(source, time(0)).unwrap();
    let before = run.snapshot();
    assert_eq!(run.request_inspection(source, time(1)), Err(RunError::Busy));
    assert_eq!(run.resume(source), Err(RunError::Busy));
    assert_eq!(
        run.step_inspection(cursor, WorkBudget::full()),
        Err(RunError::BudgetLimit)
    );
    assert_eq!(
        run.activate_inspection(cursor.key()),
        Err(RunError::InspectionIncomplete)
    );
    let other = RunSource {
        revision: MapRevision::new(8),
        ..source
    };
    assert_eq!(
        run.request_inspection(other, time(0)),
        Err(RunError::StaleSource)
    );
    assert_eq!(run.snapshot(), before);
    assert!(matches!(
        run.step_inspection(cursor, WorkBudget::one()).unwrap(),
        InspectionReport::Pending { .. }
    ));
    run.cancel_inspection(cursor.key()).unwrap();
    run.resume(source).unwrap();
}

#[test]
fn inspection_resume_rearms_and_only_hits_after_the_closed_target() {
    let mut run = run(50_104, &[1_000_000], Limits::tiny());
    freeze(&mut run, 799_999, WorkBudget::one());
    let old_source = run.source();
    inspect(&mut run, 1_000_000, WorkBudget::one());
    let source = run.resume(run.source()).unwrap();
    let before = run.snapshot();
    assert_eq!(
        run.submit_inputs(old_source, &[press(2, 1_000_001)]),
        Err(RunError::StaleSource)
    );
    assert_eq!(
        run.submit_inputs(source, &[press(2, 1_000_001)]),
        Err(RunError::RearmRequired)
    );
    assert_eq!(run.snapshot(), before);
    rearm(&mut run);
    run.submit_inputs(source, &[press(2, 1_000_001)]).unwrap();
    advance(&mut run, 1_000_001, WorkBudget::one());
    let state = run.snapshot();
    assert_eq!(
        state.committed_judgements[0],
        Some(JudgementRecord {
            object: ObjectId::new(0),
            at: time(1_000_001),
            result: Judgement::Hit
        })
    );
    let frame = state.ready_frame.unwrap();
    assert_eq!(
        frame.statistics_origin,
        StatisticsOrigin::Inspection {
            at: time(1_000_000)
        }
    );
    assert_eq!(frame.object(0).unwrap().unavailable, None);
    let first = run.event_at(0).unwrap();
    let sound = run.event_at(1).unwrap();
    assert_eq!(first.key.sequence().get(), 1);
    assert_eq!(first.key.transport(), source.transport);
    assert_eq!(first.at, time(1_000_001));
    assert_eq!(first.kind, EventKind::Judgement(Judgement::Hit));
    assert_eq!(sound.key.sequence().get(), 2);
    assert_eq!(sound.kind, EventKind::SoundIntent);
    assert!(run.sound_intent_allowed(sound.key).unwrap());
}

#[test]
fn input_at_or_before_inspection_target_does_not_reopen_the_seal() {
    for (index, target) in [999_999, 1_000_000].into_iter().enumerate() {
        let mut run = run(50_110 + index as u128, &[1_000_000], Limits::tiny());
        freeze(&mut run, 799_999, WorkBudget::one());
        inspect(&mut run, 1_000_000, WorkBudget::one());
        run.resume(run.source()).unwrap();
        rearm(&mut run);
        assert_eq!(
            run.submit_inputs(run.source(), &[press(2, target)]),
            Err(RunError::LateInput)
        );
        assert!(matches!(run.snapshot().status, RunStatus::Failed(_)));
        assert_eq!(run.snapshot().committed_judgements, [None; MAX_CIRCLES]);
        assert_eq!(run.event_at(0), None);
    }
}

#[test]
fn skipped_objects_remain_unavailable_while_only_future_objects_can_miss() {
    let mut run = run(50_120, &[0, 1_000_000], Limits::reference());
    freeze(&mut run, -200_001, WorkBudget::full());
    inspect(&mut run, 100_001, WorkBudget::full());
    run.resume(run.source()).unwrap();
    advance(&mut run, 1_300_000, WorkBudget::full());
    let state = run.snapshot();
    assert_eq!(state.committed_judgements[0], None);
    assert_eq!(
        state.committed_judgements[1].unwrap().result,
        Judgement::Miss
    );
    let frame = state.ready_frame.unwrap();
    assert_eq!(
        frame.object(0).unwrap().unavailable,
        Some(UnavailableReason::InspectionSkipped)
    );
    assert_eq!(frame.object(1).unwrap().unavailable, None);
    assert_eq!(frame.statistics_origin, state.statistics_origin);
    assert_eq!(run.event_at(0).unwrap().object, ObjectId::new(1));
    assert_eq!(run.event_at(1), None);
}

#[test]
fn old_input_events_acknowledgements_and_cursors_cannot_touch_new_transport() {
    let mut run = run(50_121, &[1_000_000], Limits::tiny());
    let old_source = run.source();
    run.submit_inputs(old_source, &[press(1, 1_000_000)])
        .unwrap();
    run.seal_through(old_source, time(1_000_000)).unwrap();
    let AdvanceReport::Pending { cursor, .. } = run
        .advance(old_source, time(1_000_000), WorkBudget::one())
        .unwrap()
    else {
        panic!("bounded advance");
    };
    freeze(&mut run, 1_000_000, WorkBudget::one());
    let old_event = run.event_at(0).unwrap();
    let old_sound = run.event_at(1).unwrap();
    inspect(&mut run, 1_000_000, WorkBudget::one());
    run.resume(run.source()).unwrap();
    rearm(&mut run);
    run.submit_inputs(run.source(), &[press(2, 1_000_001)])
        .unwrap();
    advance(&mut run, 1_000_001, WorkBudget::one());
    let before = run.snapshot();
    let new_event = run.event_at(0).unwrap();
    assert_eq!(old_event.key.sequence(), new_event.key.sequence());
    assert_ne!(old_event.key, new_event.key);
    assert_eq!(run.acknowledge(old_event.key), Err(RunError::StaleSource));
    assert_eq!(
        run.sound_intent_allowed(old_sound.key),
        Err(RunError::StaleSource)
    );
    assert_eq!(
        run.continue_advance(cursor, WorkBudget::one()),
        Err(RunError::StaleSource)
    );
    assert_eq!(
        run.cancel_request(cursor.request()),
        Err(RunError::StaleSource)
    );
    assert_eq!(
        run.submit_inputs(old_source, &[press(1, 1_000_000)]),
        Err(RunError::StaleSource)
    );
    assert_eq!(run.snapshot(), before);
    assert_eq!(run.event_at(0), Some(new_event));
}

#[test]
fn forward_then_backward_inspection_rebuilds_instead_of_copying_old_outcomes() {
    let mut run = run(50_122, &[1_000_000], Limits::reference());
    freeze(&mut run, 799_999, WorkBudget::full());
    let first = inspect(&mut run, 1_300_000, WorkBudget::full());
    assert_eq!(
        run.snapshot()
            .ready_frame
            .unwrap()
            .object(0)
            .unwrap()
            .unavailable,
        Some(UnavailableReason::InspectionSkipped)
    );
    let second = inspect(&mut run, -1, WorkBudget::full());
    assert_eq!(second.transport.get(), first.transport.get() + 1);
    let frame = run.snapshot().ready_frame.unwrap();
    assert_eq!(frame.object(0).unwrap().phase, Phase::Future);
    assert_eq!(frame.object(0).unwrap().unavailable, None);
    run.resume(second).unwrap();
    rearm(&mut run);
    run.submit_inputs(run.source(), &[press(2, 1_000_000)])
        .unwrap();
    advance(&mut run, 1_000_000, WorkBudget::full());
    assert_eq!(
        run.snapshot().committed_judgements[0].unwrap().result,
        Judgement::Hit
    );
}

#[test]
fn empty_map_still_requires_evaluation_before_activation() {
    let mut run = run(50_123, &[], Limits::tiny());
    freeze(&mut run, 0, WorkBudget::one());
    let cursor = run
        .request_inspection(run.source(), time(i64::MIN))
        .unwrap();
    assert!(!run.snapshot().inspection.unwrap().ready);
    assert_eq!(
        run.activate_inspection(cursor.key()),
        Err(RunError::InspectionIncomplete)
    );
    assert_eq!(
        run.step_inspection(cursor, WorkBudget::one()),
        Ok(InspectionReport::Ready {
            key: cursor.key(),
            at: time(i64::MIN),
            steps: 1
        })
    );
    run.activate_inspection(cursor.key()).unwrap();
    let frame = run.snapshot().ready_frame.unwrap();
    assert!(frame.is_empty());
    assert_eq!(frame.at, time(i64::MIN));
    run.resume(run.source()).unwrap();
    advance(&mut run, i64::MIN, WorkBudget::one());
    assert_eq!(run.event_at(0), None);
}

#[test]
fn a_base_fault_invalidates_pending_inspection_without_publishing_it() {
    let mut run = run(50_124, &[1_000_000], Limits::tiny());
    run.submit_inputs(run.source(), &[press(1, 1_000_000)])
        .unwrap();
    freeze(&mut run, 1_000_000, WorkBudget::one());
    let key = prepare(&mut run, -1, WorkBudget::one());
    let old_frame = run.snapshot().ready_frame;
    let conflict = InputEvent {
        edge: InputEdge::Release,
        ..press(1, 1_000_000)
    };
    assert_eq!(
        run.submit_inputs(run.source(), &[conflict]),
        Err(RunError::InputConflict)
    );
    assert_eq!(run.snapshot().inspection, None);
    assert_eq!(run.activate_inspection(key), Err(RunError::StaleInspection));
    assert_eq!(run.snapshot().ready_frame, old_frame);
    assert!(matches!(run.snapshot().status, RunStatus::Failed(_)));
}

#[test]
fn single_step_and_full_budget_produce_identical_inspection_and_later_events() {
    let mut tiny = run(
        50_125,
        &[0, 1_000_000, 2_000_000, 3_000_000],
        Limits::tiny(),
    );
    let mut reference = run(
        50_125,
        &[0, 1_000_000, 2_000_000, 3_000_000],
        Limits::reference(),
    );
    for (run, budget) in [
        (&mut tiny, WorkBudget::one()),
        (&mut reference, WorkBudget::full()),
    ] {
        freeze(run, -200_001, budget);
        inspect(run, 1_000_000, budget);
        run.resume(run.source()).unwrap();
        rearm(run);
        run.submit_inputs(run.source(), &[press(2, 1_000_001)])
            .unwrap();
        advance(run, 1_000_001, budget);
    }
    assert_eq!(tiny.snapshot(), reference.snapshot());
    for index in 0..=2 {
        assert_eq!(tiny.event_at(index), reference.event_at(index));
    }
}

#[test]
fn inspection_requires_a_completed_nonfaulted_freeze() {
    let mut run = run(50_130, &[1_000_000], Limits::tiny());
    let source = run.source();
    let before = run.snapshot();
    assert_eq!(
        run.request_inspection(source, time(1_000_000)),
        Err(RunError::NotFrozen)
    );
    assert_eq!(run.snapshot(), before);
    run.submit_inputs(source, &[press(1, 1_000_000)]).unwrap();
    run.request_freeze(source, time(1_000_000)).unwrap();
    run.seal_through(source, time(1_000_000)).unwrap();
    assert!(matches!(
        run.settle_freeze(source, WorkBudget::one()).unwrap(),
        FreezeReport::Pending { .. }
    ));
    let before = run.snapshot();
    assert_eq!(before.status, RunStatus::Freezing);
    assert_eq!(before.ready_frame, None);
    assert_eq!(
        run.request_inspection(source, time(1_000_000)),
        Err(RunError::NotFrozen)
    );
    assert_eq!(run.snapshot(), before);
    freeze(&mut run, 1_000_000, WorkBudget::one());
    let conflict = InputEvent {
        edge: InputEdge::Release,
        ..press(1, 1_000_000)
    };
    assert_eq!(
        run.submit_inputs(source, &[conflict]),
        Err(RunError::InputConflict)
    );
    let before = run.snapshot();
    assert!(matches!(before.status, RunStatus::Failed(_)));
    assert_eq!(
        run.request_inspection(source, time(1_000_001)),
        Err(RunError::Faulted)
    );
    assert_eq!(run.snapshot(), before);
}

#[test]
fn inspecting_at_window_close_misses_only_after_resuming_past_it() {
    let mut run = run(50_131, &[1_000_000], Limits::tiny());
    freeze(&mut run, 799_999, WorkBudget::one());
    inspect(&mut run, 1_100_000, WorkBudget::one());
    let circle = run.snapshot().ready_frame.unwrap().object(0).unwrap();
    assert_eq!(circle.phase, Phase::VisibleUnjudged);
    assert_eq!(circle.judgement, None);
    assert_eq!(circle.unavailable, None);
    assert_eq!(run.event_at(0), None);
    run.resume(run.source()).unwrap();
    advance(&mut run, 1_100_000, WorkBudget::one());
    assert_eq!(run.snapshot().committed_judgements[0], None);
    assert_eq!(run.event_at(0), None);
    advance(&mut run, 1_100_001, WorkBudget::one());
    let record = JudgementRecord {
        object: ObjectId::new(0),
        at: time(1_100_001),
        result: Judgement::Miss,
    };
    assert_eq!(run.snapshot().committed_judgements[0], Some(record));
    let event = run.event_at(0).unwrap();
    assert_eq!(event.kind, EventKind::Judgement(Judgement::Miss));
    assert_eq!(event.at, record.at);
    assert_eq!(event.key.sequence().get(), 1);
    assert_eq!(run.event_at(1), None);
    advance(&mut run, 1_300_000, WorkBudget::one());
    assert_eq!(run.event_at(0), Some(event));
    assert_eq!(run.event_at(1), None);
    let frame = run.snapshot().ready_frame.unwrap();
    assert_eq!(frame.object(0).unwrap().judgement, Some(record));
    assert_eq!(frame.object(0).unwrap().unavailable, None);
    assert_eq!(
        frame.statistics_origin,
        StatisticsOrigin::Inspection {
            at: time(1_100_000)
        }
    );
}

#[test]
fn cancelling_a_candidate_keeps_acknowledgements_and_does_not_rearm_old_sound() {
    let mut run = run(50_132, &[0, 1_000_000], Limits::tiny());
    run.submit_inputs(run.source(), &[press(1, 0)]).unwrap();
    freeze(&mut run, 0, WorkBudget::one());
    let judgement = run.event_at(0).unwrap();
    let sound = run.event_at(1).unwrap();
    let cursor = run.request_inspection(run.source(), time(900_000)).unwrap();
    assert!(matches!(
        run.step_inspection(cursor, WorkBudget::one()).unwrap(),
        InspectionReport::Pending { .. }
    ));
    assert!(matches!(
        run.acknowledge(judgement.key).unwrap(),
        Acknowledgement::Acknowledged { removed: 1, .. }
    ));
    let mut expected = run.snapshot();
    expected.inspection = None;
    run.cancel_inspection(cursor.key()).unwrap();
    assert_eq!(run.snapshot(), expected);
    assert_eq!(run.event_at(0), Some(sound));
    assert_eq!(run.event_at(1), None);
    assert_eq!(
        run.acknowledge(judgement.key),
        Ok(Acknowledgement::AlreadyRetired)
    );
    run.resume(run.source()).unwrap();
    assert!(!run.sound_intent_allowed(sound.key).unwrap());
    run.acknowledge(sound.key).unwrap();
    assert_eq!(run.event_at(0), None);
}

#[test]
fn overlapping_visibility_does_not_replay_skipped_deadlines() {
    // The acceptance intervals touch at consecutive integer times while the
    // visible/fade intervals overlap. Inspection must rebuild all four heads.
    let mut run = run(50_133, &[0, 200_001], Limits::tiny());
    freeze(&mut run, -200_001, WorkBudget::one());
    inspect(&mut run, 100_001, WorkBudget::one());
    run.resume(run.source()).unwrap();
    rearm(&mut run);
    run.submit_inputs(run.source(), &[press(2, 100_002)])
        .unwrap();
    advance(&mut run, 100_002, WorkBudget::one());
    let state = run.snapshot();
    assert_eq!(state.committed_judgements[0], None);
    assert_eq!(
        state.committed_judgements[1],
        Some(JudgementRecord {
            object: ObjectId::new(1),
            at: time(100_002),
            result: Judgement::Hit,
        })
    );
    let frame = state.ready_frame.unwrap();
    assert_eq!(
        frame.object(0).unwrap().unavailable,
        Some(UnavailableReason::InspectionSkipped)
    );
    assert_eq!(frame.object(1).unwrap().phase, Phase::ResolvedVisual);
    let first = run.event_at(0).unwrap();
    let second = run.event_at(1).unwrap();
    assert_eq!(first.object, ObjectId::new(1));
    assert_eq!(first.kind, EventKind::Judgement(Judgement::Hit));
    assert_eq!(second.object, ObjectId::new(1));
    assert_eq!(second.kind, EventKind::SoundIntent);
    run.acknowledge(first.key).unwrap();
    run.acknowledge(second.key).unwrap();
    advance(&mut run, 500_001, WorkBudget::one());
    assert_eq!(run.event_at(0), None);
    assert_eq!(
        run.snapshot().committed_judgements,
        state.committed_judgements
    );
    let frame = run.snapshot().ready_frame.unwrap();
    assert_eq!(frame.object(0).unwrap().phase, Phase::Retired);
    assert_eq!(frame.object(1).unwrap().phase, Phase::Retired);
}

#[test]
fn repeated_inspection_after_a_real_hit_rebuilds_ledger_and_freeze_state() {
    let mut run = run(50_134, &[1_000_000], Limits::tiny());
    freeze(&mut run, 799_999, WorkBudget::one());
    inspect(&mut run, 1_000_000, WorkBudget::one());
    run.resume(run.source()).unwrap();
    rearm(&mut run);
    run.submit_inputs(run.source(), &[press(2, 1_000_001)])
        .unwrap();
    advance(&mut run, 1_000_001, WorkBudget::one());
    let old_event = run.event_at(0).unwrap();
    freeze(&mut run, 1_000_001, WorkBudget::one());
    let before = run.snapshot();
    let cancelled = prepare(&mut run, 1_000_002, WorkBudget::one());
    run.cancel_inspection(cancelled).unwrap();
    assert_eq!(run.snapshot(), before);
    let source = inspect(&mut run, 1_000_002, WorkBudget::one());
    assert_eq!(source.transport.get(), old_event.key.transport().get() + 1);
    assert_eq!(run.snapshot().committed_judgements, [None; MAX_CIRCLES]);
    assert_eq!(run.event_at(0), None);
    let before = run.snapshot();
    run.request_freeze(source, time(1_000_002)).unwrap();
    assert_eq!(
        run.settle_freeze(source, WorkBudget::one()),
        Ok(FreezeReport::Frozen {
            at: time(1_000_002),
            steps: 0,
            cancelled_inputs: 0,
        })
    );
    assert_eq!(run.snapshot(), before);
    run.resume(source).unwrap();
    rearm(&mut run);
    run.submit_inputs(run.source(), &[press(2, 1_000_003)])
        .unwrap();
    advance(&mut run, 1_000_003, WorkBudget::one());
    let current = run.snapshot();
    assert_eq!(
        current.statistics_origin,
        StatisticsOrigin::Inspection {
            at: time(1_000_002)
        }
    );
    assert_eq!(current.committed_judgements[0].unwrap().at, time(1_000_003));
    assert_eq!(run.event_at(0).unwrap().key.sequence().get(), 1);
    assert_eq!(run.acknowledge(old_event.key), Err(RunError::StaleSource));
    assert_eq!(run.snapshot(), current);
    assert!(
        run.sound_intent_allowed(run.event_at(1).unwrap().key)
            .unwrap()
    );
}
