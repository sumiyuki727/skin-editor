#[allow(dead_code)]
mod common;
use common::SnapshotExt;
use skin_editor_gameplay::*;

fn time(value: i64) -> MapTime {
    MapTime::from_micros(value)
}
fn point(x: i32, y: i32) -> Point {
    Point::new(x, y).unwrap()
}
fn source(id: u128) -> RunSource {
    RunSource {
        run: RunId::new(id).unwrap(),
        transport: TransportEpoch::new(1).unwrap(),
        input: InputEpoch::new(1).unwrap(),
        revision: MapRevision::new(7),
    }
}
fn circle(id: u32, at: i64) -> Circle {
    Circle::new(ObjectId::new(id), time(at), point(0, 0)).unwrap()
}
fn make_run(id: u128, circles: &[Circle], start: i64, limits: Limits) -> SyntheticRun {
    let map = SyntheticMap::try_new(source(id).revision, circles).unwrap();
    SyntheticRun::new(source(id), map, time(start), limits).unwrap()
}
fn single(id: u128, limits: Limits) -> SyntheticRun {
    make_run(id, &[circle(0, 1_000_000)], 799_999, limits)
}
fn press(sequence: u64, at: i64, slot: PhysicalSlot, x: i32) -> InputEvent {
    InputEvent {
        sequence: InputSequence::new(sequence).unwrap(),
        at: time(at),
        slot,
        edge: InputEdge::Press(point(x, 0)),
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
fn complete(run: &mut SyntheticRun, at: i64, budget: WorkBudget) {
    run.seal_through(run.source(), time(at)).unwrap();
    complete_sealed(run, at, budget);
}
fn complete_sealed(run: &mut SyntheticRun, at: i64, budget: WorkBudget) {
    let mut report = run.advance(run.source(), time(at), budget).unwrap();
    for _ in 0..10_000 {
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
                assert_eq!(
                    reason,
                    PendingReason::WorkBudget,
                    "unexpected event backpressure"
                );
                report = run.continue_advance(cursor, budget).unwrap();
            }
        }
    }
    panic!("bounded fixture did not complete");
}
fn frame(run: &SyntheticRun, index: usize) -> CircleFrame {
    run.snapshot().ready_frame.unwrap().object(index).unwrap()
}
fn collect_events(run: &mut SyntheticRun, output: &mut Vec<GameEvent>) {
    while let Some(event) = run.event_at(0) {
        output.push(event);
        run.acknowledge(event.key).unwrap();
    }
}

#[test]
fn lifecycle_and_hit_have_exact_source_time_and_no_read_side_effects() {
    let mut run = single(1, Limits::tiny());
    let initial = run.snapshot();
    assert_eq!(initial.status, RunStatus::Prepared);
    assert_eq!(initial.completed_through, None);
    assert_eq!(initial.ready_frame, None);
    complete(&mut run, 799_999, WorkBudget::one());
    assert_eq!(frame(&run, 0).phase, Phase::Future);
    complete(&mut run, 800_000, WorkBudget::one());
    assert_eq!(frame(&run, 0).phase, Phase::VisibleUnjudged);
    let input = press(1, 1_000_000, PhysicalSlot::PrimaryOne, 0);
    assert_eq!(
        run.submit_inputs(run.source(), &[input]).unwrap(),
        Admission {
            added: 1,
            repeated: 0
        }
    );
    complete(&mut run, 1_000_000, WorkBudget::one());
    assert_eq!(frame(&run, 0).phase, Phase::ResolvedVisual);
    let record = run.snapshot().committed_judgements[0].unwrap();
    assert_eq!(
        record,
        JudgementRecord {
            object: ObjectId::new(0),
            at: time(1_000_000),
            result: Judgement::Hit
        }
    );
    let first = run.event_at(0).unwrap();
    let second = run.event_at(1).unwrap();
    assert_eq!(first.kind, EventKind::Judgement(Judgement::Hit));
    assert_eq!(second.kind, EventKind::SoundIntent);
    assert_eq!(
        (first.key.sequence().get(), second.key.sequence().get()),
        (1, 2)
    );
    assert_eq!(first.key.run(), run.source().run);
    assert_eq!(first.at, time(1_000_000));
    assert_eq!(first.evidence, Evidence::SyntheticTest);
    assert_eq!(first.profile_version, SYNTHETIC_PROFILE_VERSION);
    let before = run.snapshot();
    for _ in 0..10 {
        assert_eq!(run.snapshot(), before);
        assert_eq!(run.event_at(0), Some(first));
    }
    complete(&mut run, 1_000_000, WorkBudget::one());
    complete(&mut run, 1_299_999, WorkBudget::one());
    assert_eq!(frame(&run, 0).phase, Phase::ResolvedVisual);
    complete(&mut run, 1_300_000, WorkBudget::one());
    assert_eq!(frame(&run, 0).phase, Phase::Retired);
    assert_eq!(run.event_at(0), Some(first));
    assert_eq!(run.event_at(1), Some(second));
    assert_eq!(run.snapshot().committed_judgements[0], Some(record));
}

#[test]
fn closed_acceptance_window_expires_one_microsecond_later_without_sound() {
    let mut run = single(2, Limits::reference());
    complete(&mut run, 1_100_000, WorkBudget::full());
    assert_eq!(run.snapshot().committed_judgements[0], None);
    assert_eq!(run.event_at(0), None);
    complete(&mut run, 1_100_001, WorkBudget::full());
    assert_eq!(
        run.snapshot().committed_judgements[0].unwrap().result,
        Judgement::Miss
    );
    assert_eq!(run.event_at(0).unwrap().at, time(1_100_001));
    assert_eq!(
        run.event_at(0).unwrap().kind,
        EventKind::Judgement(Judgement::Miss)
    );
    assert_eq!(run.event_at(1), None);
    complete(&mut run, 2_000_000, WorkBudget::full());
    assert_eq!(frame(&run, 0).phase, Phase::Retired);
    assert_eq!(run.snapshot().pending_events, 1);
}

#[test]
fn time_and_spatial_edges_use_the_explicit_synthetic_rule() {
    for (case, (at, x, expected)) in [
        (899_999, 0, Judgement::Miss),
        (900_000, 0, Judgement::Hit),
        (1_100_000, 0, Judgement::Hit),
        (1_100_001, 0, Judgement::Miss),
        (1_000_000, 9, Judgement::Hit),
        (1_000_000, 10, Judgement::Hit),
        (1_000_000, 11, Judgement::Miss),
        (1_000_000, -10, Judgement::Hit),
        (1_000_000, -11, Judgement::Miss),
    ]
    .into_iter()
    .enumerate()
    {
        let mut run = single(100 + u128::try_from(case).unwrap(), Limits::reference());
        run.submit_inputs(run.source(), &[press(1, at, PhysicalSlot::PrimaryOne, x)])
            .unwrap();
        complete(&mut run, 1_300_000, WorkBudget::full());
        assert_eq!(
            run.snapshot().committed_judgements[0].unwrap().result,
            expected,
            "{at}/{x}"
        );
        assert_eq!(
            run.snapshot().pending_events,
            if expected == Judgement::Hit { 2 } else { 1 }
        );
    }
}

#[test]
fn preparation_is_complete_or_cancelled_and_preserves_original_identities() {
    let data = [circle(99, 0), circle(7, 1_000_000)];
    let mut preparing = MapPreparation::new(MapRevision::new(7), &data).unwrap();
    assert_eq!(
        preparing.step(WorkBudget::one()).unwrap(),
        PreparationProgress::Pending {
            checked: 1,
            total: 2
        }
    );
    preparing.cancel();
    preparing.cancel();
    assert_eq!(
        preparing.step(WorkBudget::one()),
        Err(ValueError::PreparationCancelled)
    );
    assert_eq!(preparing.finish(), Err(ValueError::PreparationCancelled));
    let mut preparing = MapPreparation::new(MapRevision::new(7), &data).unwrap();
    preparing.step(WorkBudget::one()).unwrap();
    assert_eq!(
        preparing.step(WorkBudget::one()).unwrap(),
        PreparationProgress::Pending {
            checked: 1,
            total: 2
        }
    );
    assert_eq!(
        preparing.step(WorkBudget::one()).unwrap(),
        PreparationProgress::Complete { checked: 2 }
    );
    let map = preparing.finish().unwrap();
    assert_eq!(map.circle(0).unwrap().id(), ObjectId::new(99));
    assert_eq!(map.circle(1).unwrap().id(), ObjectId::new(7));
    assert_eq!(map.circle(99), None);
    let mut run = SyntheticRun::new(source(3), map, time(-200_001), Limits::reference()).unwrap();
    complete(&mut run, 1_300_000, WorkBudget::full());
    assert_eq!(run.event_at(0).unwrap().object, ObjectId::new(99));
    assert_eq!(run.event_at(1).unwrap().object, ObjectId::new(7));
}

#[test]
fn invalid_candidates_never_yield_partial_maps_or_change_an_existing_run() {
    let existing = single(4, Limits::tiny());
    let before = existing.snapshot();
    let data = [circle(1, 0), circle(2, 1_000_000), circle(1, 2_000_000)];
    let mut preparing = MapPreparation::new(MapRevision::new(8), &data).unwrap();
    preparing.step(WorkBudget::new(3).unwrap()).unwrap();
    assert_eq!(
        preparing.step(WorkBudget::one()).unwrap(),
        PreparationProgress::Pending {
            checked: 2,
            total: 3
        }
    );
    assert_eq!(
        preparing.step(WorkBudget::one()),
        Err(ValueError::DuplicateObject)
    );
    assert_eq!(preparing.finish(), Err(ValueError::DuplicateObject));
    assert_eq!(existing.snapshot(), before);
    assert_eq!(
        SyntheticMap::try_new(MapRevision::new(7), &[circle(0, 0), circle(1, 0)]),
        Err(ValueError::UnsupportedOverlap)
    );
    assert_eq!(
        SyntheticMap::try_new(MapRevision::new(7), &[circle(0, 1_000_000), circle(1, 0)]),
        Err(ValueError::UnsortedMap)
    );
    assert_eq!(
        SyntheticMap::try_new(MapRevision::new(7), &[circle(0, 0); 5]),
        Err(ValueError::ObjectLimit)
    );
    let pending = MapPreparation::new(MapRevision::new(1), &data).unwrap();
    assert_eq!(pending.finish(), Err(ValueError::PreparationIncomplete));
}

#[test]
fn signed_time_extremes_empty_maps_and_storage_limits_are_explicit() {
    assert_eq!(RunId::new(0), Err(ValueError::ZeroIdentity));
    assert_eq!(InputSequence::new(0), Err(ValueError::ZeroIdentity));
    assert_eq!(EventSequence::new(0), Err(ValueError::ZeroIdentity));
    assert_eq!(TransportEpoch::new(u64::MAX).unwrap().checked_next(), None);
    assert_eq!(InputEpoch::new(u64::MAX).unwrap().checked_next(), None);
    assert_eq!(Point::new(1025, 0), Err(ValueError::CoordinateRange));
    assert_eq!(
        Circle::new(ObjectId::new(0), time(i64::MIN), point(0, 0)),
        Err(ValueError::TimeOverflow)
    );
    assert_eq!(
        Circle::new(ObjectId::new(0), time(i64::MAX), point(0, 0)),
        Err(ValueError::TimeOverflow)
    );
    assert_eq!(WorkBudget::new(0), Err(ValueError::InvalidBudget));
    assert_eq!(
        Limits::try_new(4, 1, 4, 1, 65536),
        Err(ValueError::InvalidLimits)
    );
    assert_eq!(
        Limits::try_new(4, 2, 4, 1, 0),
        Err(ValueError::StorageLimit)
    );
    assert!(SyntheticRun::static_storage_bytes() <= 64 * 1024);
    let mut empty = make_run(5, &[], i64::MIN, Limits::tiny());
    complete(&mut empty, i64::MIN, WorkBudget::one());
    assert!(empty.snapshot().ready_frame.unwrap().is_empty());
    complete(&mut empty, i64::MAX, WorkBudget::one());
    assert_eq!(empty.snapshot().pending_events, 0);
    let mut negative = make_run(6, &[circle(0, 0)], i64::MIN, Limits::reference());
    complete(&mut negative, i64::MIN, WorkBudget::full());
    assert_eq!(frame(&negative, 0).phase, Phase::Future);
}

#[test]
fn stale_sources_are_rejected_before_capacity_and_do_not_disturb_continuation() {
    let mut run = single(10, Limits::tiny());
    run.seal_through(run.source(), time(1_000_000)).unwrap();
    let AdvanceReport::Pending { cursor, .. } = run
        .advance(run.source(), time(1_000_000), WorkBudget::one())
        .unwrap()
    else {
        panic!("expected continuation")
    };
    let valid = run.source();
    let wrong = [
        RunSource {
            run: RunId::new(999).unwrap(),
            ..valid
        },
        RunSource {
            transport: TransportEpoch::new(2).unwrap(),
            ..valid
        },
        RunSource {
            input: InputEpoch::new(2).unwrap(),
            ..valid
        },
        RunSource {
            revision: MapRevision::new(8),
            ..valid
        },
    ];
    let before = run.snapshot();
    for stale in wrong {
        assert_eq!(
            run.submit_inputs(stale, &[press(1, 0, PhysicalSlot::PrimaryOne, 0); 5]),
            Err(RunError::StaleSource)
        );
        assert_eq!(run.seal_through(stale, time(0)), Err(RunError::StaleSource));
        assert_eq!(
            run.advance(stale, time(0), WorkBudget::full()),
            Err(RunError::StaleSource)
        );
        assert_eq!(run.snapshot(), before);
    }
    assert!(run.continue_advance(cursor, WorkBudget::one()).is_ok());
}

#[test]
fn seal_is_inclusive_but_identical_replay_still_succeeds() {
    let mut run = single(11, Limits::tiny());
    let input = press(1, 1_000_000, PhysicalSlot::PrimaryOne, 0);
    run.submit_inputs(run.source(), &[input]).unwrap();
    complete(&mut run, 1_000_000, WorkBudget::one());
    let before = run.snapshot();
    assert_eq!(
        run.submit_inputs(run.source(), &[input]).unwrap(),
        Admission {
            added: 0,
            repeated: 1
        }
    );
    assert_eq!(run.snapshot(), before);
    let event = run.event_at(0);
    assert_eq!(
        run.submit_inputs(
            run.source(),
            &[press(2, 1_000_000, PhysicalSlot::SecondaryOne, 0)]
        ),
        Err(RunError::LateInput)
    );
    let after = run.snapshot();
    assert_eq!(
        after.status,
        RunStatus::Failed(Failure {
            cause: RunError::LateInput,
            cancelled_inputs: 0
        })
    );
    assert_eq!(after.committed_judgements, before.committed_judgements);
    assert_eq!(after.ready_frame, before.ready_frame);
    assert_eq!(run.event_at(0), event);
    assert_eq!(
        run.advance(run.source(), time(1_000_001), WorkBudget::one()),
        Err(RunError::Faulted)
    );
    assert!(run.acknowledge(event.unwrap().key).is_ok());
    let mut early = single(12, Limits::tiny());
    assert_eq!(
        early.submit_inputs(
            early.source(),
            &[press(1, 799_998, PhysicalSlot::PrimaryOne, 0)]
        ),
        Err(RunError::InputBeforeStart)
    );
    let mut unsealed = single(13, Limits::tiny());
    let before = unsealed.snapshot();
    assert_eq!(
        unsealed.advance(unsealed.source(), time(900_000), WorkBudget::one()),
        Err(RunError::BeyondSeal)
    );
    assert_eq!(unsealed.snapshot(), before);
    complete(&mut unsealed, 900_000, WorkBudget::one());
}

#[test]
fn admission_is_atomic_across_duplicates_conflicts_and_capacity_failure() {
    let mut run = single(14, Limits::tiny());
    let original = press(1, 900_000, PhysicalSlot::PrimaryOne, 99);
    run.submit_inputs(run.source(), &[original]).unwrap();
    let changed = press(1, 900_000, PhysicalSlot::PrimaryOne, 0);
    let new = release(2, 900_001, PhysicalSlot::PrimaryOne);
    assert_eq!(
        run.submit_inputs(run.source(), &[original, new, changed]),
        Err(RunError::InputConflict)
    );
    let failed = run.snapshot();
    assert_eq!(
        failed.status,
        RunStatus::Failed(Failure {
            cause: RunError::InputConflict,
            cancelled_inputs: 1
        })
    );
    assert_eq!(failed.pending_inputs, 0);
    assert_eq!(failed.pending_events, 0);
    assert_eq!(failed.held_slots, [false; 4]);
    assert_eq!(
        run.submit_inputs(run.source(), &[original])
            .unwrap()
            .repeated,
        1
    );
    assert_eq!(
        run.submit_inputs(run.source(), &[new]),
        Err(RunError::GateClosed)
    );
    let mut run = single(15, Limits::tiny());
    assert_eq!(
        run.submit_inputs(run.source(), &[original, original])
            .unwrap(),
        Admission {
            added: 1,
            repeated: 1
        }
    );
    assert_eq!(run.snapshot().pending_inputs, 1);
    let batch = [
        release(2, 900_001, PhysicalSlot::PrimaryOne),
        press(3, 900_002, PhysicalSlot::PrimaryOne, 99),
        release(4, 900_003, PhysicalSlot::PrimaryOne),
        press(5, 900_004, PhysicalSlot::PrimaryOne, 99),
    ];
    assert_eq!(
        run.submit_inputs(run.source(), &batch),
        Err(RunError::InputCapacity)
    );
    assert_eq!(
        run.snapshot().status,
        RunStatus::Failed(Failure {
            cause: RunError::InputCapacity,
            cancelled_inputs: 1
        })
    );
    let mut duplicate = single(16, Limits::tiny());
    assert_eq!(
        duplicate.submit_inputs(duplicate.source(), &[original, changed]),
        Err(RunError::InputConflict)
    );
    assert_eq!(duplicate.snapshot().pending_inputs, 0);
}

#[test]
fn replay_window_expires_without_refresh_and_sequence_holes_are_not_new() {
    let mut run = single(17, Limits::tiny());
    let first = press(1, 900_001, PhysicalSlot::PrimaryOne, 99);
    run.submit_inputs(run.source(), &[first]).unwrap();
    complete(&mut run, 900_001, WorkBudget::one());
    for seq in 2..=5 {
        assert_eq!(
            run.submit_inputs(run.source(), &[first]).unwrap().repeated,
            1
        );
        let at = 900_000 + i64::try_from(seq).unwrap();
        let input = if seq % 2 == 0 {
            release(seq, at, PhysicalSlot::PrimaryOne)
        } else {
            press(seq, at, PhysicalSlot::PrimaryOne, 99)
        };
        run.submit_inputs(run.source(), &[input]).unwrap();
        complete(&mut run, at, WorkBudget::one());
    }
    let before = run.snapshot();
    assert_eq!(
        run.submit_inputs(run.source(), &[first]),
        Err(RunError::OldInputSequence)
    );
    assert_eq!(run.snapshot(), before);
    let mut holes = single(18, Limits::tiny());
    holes
        .submit_inputs(
            holes.source(),
            &[press(3, 900_000, PhysicalSlot::PrimaryOne, 99)],
        )
        .unwrap();
    let before = holes.snapshot();
    assert_eq!(
        holes.submit_inputs(
            holes.source(),
            &[release(2, 900_001, PhysicalSlot::PrimaryOne)]
        ),
        Err(RunError::OldInputSequence)
    );
    assert_eq!(holes.snapshot(), before);
}

#[test]
fn two_physical_slots_share_one_logical_press_and_release_independently() {
    let mut run = single(19, Limits::reference());
    run.submit_inputs(
        run.source(),
        &[
            press(1, 900_000, PhysicalSlot::PrimaryOne, 99),
            press(2, 1_000_000, PhysicalSlot::PrimaryTwo, 0),
            release(3, 1_000_001, PhysicalSlot::PrimaryOne),
            release(4, 1_000_002, PhysicalSlot::PrimaryTwo),
            press(5, 1_000_003, PhysicalSlot::PrimaryTwo, 0),
            press(6, 1_000_003, PhysicalSlot::SecondaryOne, 0),
        ],
    )
    .unwrap();
    complete(&mut run, 1_000_001, WorkBudget::full());
    assert_eq!(run.snapshot().held_slots, [false, true, false, false]);
    assert_eq!(run.snapshot().committed_judgements[0], None);
    complete(&mut run, 1_000_002, WorkBudget::full());
    assert_eq!(run.snapshot().held_slots, [false; 4]);
    complete(&mut run, 1_000_003, WorkBudget::full());
    assert_eq!(
        run.snapshot().committed_judgements[0].unwrap().at,
        time(1_000_003)
    );
    assert_eq!(run.snapshot().pending_events, 2);
}

#[test]
fn event_backpressure_preserves_whole_hit_and_out_of_order_ack_returns_credit_once() {
    let mut run = make_run(
        20,
        &[circle(0, 0), circle(1, 1_000_000)],
        -200_001,
        Limits::tiny(),
    );
    complete(&mut run, 100_001, WorkBudget::one());
    let miss = run.event_at(0).unwrap();
    run.submit_inputs(
        run.source(),
        &[press(1, 1_000_000, PhysicalSlot::PrimaryOne, 0)],
    )
    .unwrap();
    run.seal_through(run.source(), time(1_000_000)).unwrap();
    let mut report = run
        .advance(run.source(), time(1_000_000), WorkBudget::one())
        .unwrap();
    let blocked = loop {
        match report {
            AdvanceReport::Pending {
                cursor,
                reason: PendingReason::EventBackpressure,
                ..
            } => break cursor,
            AdvanceReport::Pending { cursor, .. } => {
                report = run.continue_advance(cursor, WorkBudget::one()).unwrap()
            }
            _ => panic!("hit requires two event slots"),
        }
    };
    let before = run.snapshot();
    assert_eq!(before.committed_judgements[1], None);
    assert_eq!(before.pending_inputs, 1);
    assert_eq!(before.pending_events, 1);
    assert_eq!(before.held_slots, [false; 4]);
    assert!(before.completed_through.unwrap() < time(1_000_000));
    assert!(matches!(
        run.continue_advance(blocked, WorkBudget::one()).unwrap(),
        AdvanceReport::Pending {
            reason: PendingReason::EventBackpressure,
            steps: 0,
            ..
        }
    ));
    assert_eq!(run.snapshot(), before);
    run.acknowledge(miss.key).unwrap();
    let mut report = run.continue_advance(blocked, WorkBudget::one()).unwrap();
    loop {
        match report {
            AdvanceReport::Complete { .. } => break,
            AdvanceReport::Pending { cursor, .. } => {
                report = run.continue_advance(cursor, WorkBudget::one()).unwrap()
            }
        }
    }
    assert_eq!(
        run.snapshot().committed_judgements[1].unwrap().result,
        Judgement::Hit
    );
    let hit = run.event_at(0).unwrap();
    let sound = run.event_at(1).unwrap();
    assert_eq!(
        (hit.key.sequence().get(), sound.key.sequence().get()),
        (2, 3)
    );
    assert_eq!(
        run.acknowledge(sound.key).unwrap(),
        Acknowledgement::Acknowledged {
            retired_through: Some(EventSequence::new(1).unwrap()),
            removed: 0
        }
    );
    assert_eq!(run.snapshot().pending_events, 2);
    assert_eq!(
        run.acknowledge(hit.key).unwrap(),
        Acknowledgement::Acknowledged {
            retired_through: Some(EventSequence::new(3).unwrap()),
            removed: 2
        }
    );
    assert_eq!(
        run.acknowledge(sound.key).unwrap(),
        Acknowledgement::AlreadyRetired
    );
    let before = run.snapshot();
    assert_eq!(
        run.acknowledge(EventKey::new(
            run.source().run,
            run.source().transport,
            EventSequence::new(4).unwrap()
        )),
        Err(RunError::UnknownEvent)
    );
    assert_eq!(
        run.acknowledge(EventKey::new(
            RunId::new(999).unwrap(),
            run.source().transport,
            EventSequence::new(3).unwrap()
        )),
        Err(RunError::StaleSource)
    );
    assert_eq!(run.snapshot(), before);
}

#[test]
fn cancelling_a_partially_committed_timestamp_keeps_prefix_and_old_ready_frame() {
    let mut run = single(21, Limits::tiny());
    complete(&mut run, 900_000, WorkBudget::one());
    let old = run.snapshot().ready_frame;
    run.submit_inputs(
        run.source(),
        &[
            press(1, 1_000_000, PhysicalSlot::PrimaryOne, 0),
            press(2, 1_000_000, PhysicalSlot::SecondaryOne, 0),
        ],
    )
    .unwrap();
    run.seal_through(run.source(), time(1_000_000)).unwrap();
    let AdvanceReport::Pending {
        cursor: original, ..
    } = run
        .advance(run.source(), time(1_000_000), WorkBudget::one())
        .unwrap()
    else {
        panic!()
    };
    let mut cursor = original;
    while run.snapshot().committed_judgements[0].is_none() {
        let AdvanceReport::Pending { cursor: next, .. } =
            run.continue_advance(cursor, WorkBudget::one()).unwrap()
        else {
            panic!()
        };
        cursor = next;
    }
    let partial = run.snapshot();
    assert_eq!(partial.status, RunStatus::Pending);
    assert_eq!(partial.pending_inputs, 1);
    assert_eq!(partial.completed_through, Some(time(900_000)));
    assert_eq!(partial.ready_frame, old);
    assert_eq!(
        run.continue_advance(original, WorkBudget::one()),
        Err(RunError::StaleCursor)
    );
    assert_eq!(run.snapshot(), partial);
    run.cancel_request(original.request()).unwrap(); // Operation identity survives step advancement.
    assert_eq!(
        run.continue_advance(cursor, WorkBudget::one()),
        Err(RunError::StaleCursor)
    );
    assert_eq!(
        run.advance(run.source(), time(999_999), WorkBudget::one()),
        Err(RunError::InvalidTarget)
    );
    complete(&mut run, 1_000_000, WorkBudget::one());
    assert_eq!(run.snapshot().pending_inputs, 0);
    assert_eq!(run.snapshot().pending_events, 2);
    assert_eq!(frame(&run, 0).judgement, partial.committed_judgements[0]);
    assert_eq!(
        run.cancel_request(original.request()),
        Err(RunError::StaleCursor)
    );
}

#[test]
fn adjacent_acceptance_windows_process_input_before_same_time_miss() {
    let mut run = make_run(
        22,
        &[circle(0, 0), circle(1, 200_001)],
        -200_001,
        Limits::reference(),
    );
    run.submit_inputs(
        run.source(),
        &[press(1, 100_001, PhysicalSlot::PrimaryOne, 0)],
    )
    .unwrap();
    complete(&mut run, 100_001, WorkBudget::full());
    assert_eq!(run.event_at(0).unwrap().object, ObjectId::new(1));
    assert_eq!(
        run.event_at(0).unwrap().kind,
        EventKind::Judgement(Judgement::Hit)
    );
    assert_eq!(run.event_at(2).unwrap().object, ObjectId::new(0));
    assert_eq!(
        run.event_at(2).unwrap().kind,
        EventKind::Judgement(Judgement::Miss)
    );
}

fn cadence(hz: i64, budget: WorkBudget, irregular: bool) -> (common::Snapshot, Vec<GameEvent>) {
    let mut run = make_run(
        30,
        &[circle(0, 1_000_000), circle(1, 2_000_000)],
        799_999,
        Limits::reference(),
    );
    run.submit_inputs(
        run.source(),
        &[
            press(1, 1_000_000, PhysicalSlot::PrimaryOne, 0),
            release(2, 1_000_001, PhysicalSlot::PrimaryOne),
            press(3, 2_000_000, PhysicalSlot::SecondaryOne, 0),
            release(4, 2_000_001, PhysicalSlot::SecondaryOne),
        ],
    )
    .unwrap();
    let mut events = Vec::new();
    let mut at = 799_999;
    let mut tick = 0;
    while at < 2_300_000 {
        tick += 1;
        at = (799_999 + tick * 1_000_000 / hz + if irregular && tick % 3 == 0 { 719 } else { 0 })
            .min(2_300_000);
        run.seal_through(run.source(), time(at)).unwrap();
        let mut report = run.advance(run.source(), time(at), budget).unwrap();
        loop {
            // Querying and consuming at different host cadences cannot affect gameplay.
            let before = run.snapshot();
            assert_eq!(run.snapshot(), before);
            collect_events(&mut run, &mut events);
            match report {
                AdvanceReport::Complete { steps, .. } => {
                    assert!(steps <= budget.get());
                    break;
                }
                AdvanceReport::Pending { cursor, steps, .. } => {
                    assert!(steps <= budget.get());
                    report = run.continue_advance(cursor, budget).unwrap();
                }
            }
        }
    }
    (run.snapshot(), events)
}

#[test]
fn absolute_time_segmentation_budget_and_gui_sampling_do_not_change_results() {
    let expected = cadence(30, WorkBudget::full(), false);
    assert_eq!(expected.1.len(), 4);
    for hz in [30, 60, 144, 240] {
        for budget in [WorkBudget::one(), WorkBudget::full()] {
            for irregular in [false, true] {
                assert_eq!(
                    cadence(hz, budget, irregular),
                    expected,
                    "{hz}/{budget:?}/{irregular}"
                );
            }
        }
    }
}

#[test]
fn cancelling_search_and_uncommitted_publication_allows_a_smaller_target() {
    let mut run = make_run(
        40,
        &[circle(0, 1_000_000), circle(1, 2_000_000)],
        799_999,
        Limits::tiny(),
    );
    complete(&mut run, 900_000, WorkBudget::one());
    let old = run.snapshot().ready_frame;
    run.seal_through(run.source(), time(950_000)).unwrap();
    let AdvanceReport::Pending { cursor, .. } = run
        .advance(run.source(), time(950_000), WorkBudget::one())
        .unwrap()
    else {
        panic!("two-object publication must yield")
    };
    assert_eq!(run.snapshot().ready_frame, old);
    run.cancel_request(cursor.request()).unwrap();
    complete_sealed(&mut run, 925_000, WorkBudget::one());
    assert_eq!(run.snapshot().completed_through, Some(time(925_000)));
    run.submit_inputs(
        run.source(),
        &[press(1, 1_000_000, PhysicalSlot::PrimaryOne, 0)],
    )
    .unwrap();
    run.seal_through(run.source(), time(1_000_000)).unwrap();
    let AdvanceReport::Pending { cursor, .. } = run
        .advance(run.source(), time(1_000_000), WorkBudget::one())
        .unwrap()
    else {
        panic!()
    };
    assert_eq!(run.snapshot().held_slots, [false; 4]);
    run.cancel_request(cursor.request()).unwrap();
    complete_sealed(&mut run, 975_000, WorkBudget::one());
    assert_eq!(run.snapshot().pending_inputs, 1);
    complete(&mut run, 1_000_000, WorkBudget::one());
    assert_eq!(run.snapshot().pending_events, 2);
}

#[test]
fn full_event_queue_cannot_prevent_cancellation() {
    let mut run = make_run(
        41,
        &[circle(0, 0), circle(1, 1_000_000)],
        -200_001,
        Limits::tiny(),
    );
    run.submit_inputs(
        run.source(),
        &[
            press(1, 0, PhysicalSlot::PrimaryOne, 0),
            release(2, 1, PhysicalSlot::PrimaryOne),
        ],
    )
    .unwrap();
    complete(&mut run, 1, WorkBudget::one());
    run.seal_through(run.source(), time(1_100_001)).unwrap();
    let mut report = run
        .advance(run.source(), time(1_100_001), WorkBudget::one())
        .unwrap();
    loop {
        match report {
            AdvanceReport::Pending {
                cursor,
                reason: PendingReason::EventBackpressure,
                ..
            } => {
                run.cancel_request(cursor.request()).unwrap();
                assert_eq!(
                    run.continue_advance(cursor, WorkBudget::one()),
                    Err(RunError::StaleCursor)
                );
                assert_eq!(run.snapshot().pending_events, 2);
                assert_eq!(run.snapshot().committed_judgements[1], None);
                break;
            }
            AdvanceReport::Pending { cursor, .. } => {
                report = run.continue_advance(cursor, WorkBudget::one()).unwrap()
            }
            _ => panic!("miss must wait for event credit"),
        }
    }
}

#[test]
fn preparation_budget_counts_each_record_and_previous_identity_check() {
    let data = [circle(0, 0), circle(1, 1_000_000), circle(1, 2_000_000)];
    let mut preparing = MapPreparation::new(MapRevision::new(7), &data).unwrap();
    // Load/commit the first record, load the second, then check its predecessor.
    assert_eq!(
        preparing.step(WorkBudget::new(3).unwrap()).unwrap(),
        PreparationProgress::Pending {
            checked: 2,
            total: 3
        }
    );
    // Loading the third and comparing ID 0 must not hide the later ID 1 check.
    for _ in 0..2 {
        assert_eq!(
            preparing.step(WorkBudget::one()).unwrap(),
            PreparationProgress::Pending {
                checked: 2,
                total: 3
            }
        );
    }
    assert_eq!(
        preparing.step(WorkBudget::one()),
        Err(ValueError::DuplicateObject)
    );
    assert_eq!(preparing.finish(), Err(ValueError::DuplicateObject));
    let mut cancelled = MapPreparation::new(MapRevision::new(7), &data).unwrap();
    cancelled.step(WorkBudget::new(5).unwrap()).unwrap();
    cancelled.cancel();
    assert_eq!(cancelled.finish(), Err(ValueError::PreparationCancelled));
}

#[test]
fn mixed_retry_is_checked_against_the_pre_batch_window_before_eviction() {
    let limits = Limits::try_new(8, 2, 4, 1, 64 * 1024).unwrap();
    let mut run = single(50, limits);
    let first = press(1, 900_000, PhysicalSlot::PrimaryOne, 11);
    run.submit_inputs(
        run.source(),
        &[
            first,
            release(2, 900_001, PhysicalSlot::PrimaryOne),
            press(3, 900_002, PhysicalSlot::PrimaryOne, 11),
            release(4, 900_003, PhysicalSlot::PrimaryOne),
        ],
    )
    .unwrap();
    complete(&mut run, 900_003, WorkBudget::one());
    let next = [
        press(5, 900_004, PhysicalSlot::PrimaryOne, 11),
        release(6, 900_005, PhysicalSlot::PrimaryOne),
        press(7, 900_006, PhysicalSlot::PrimaryOne, 11),
        release(8, 900_007, PhysicalSlot::PrimaryOne),
    ];
    let mut batch = next.to_vec();
    batch.push(first);
    assert_eq!(
        run.submit_inputs(run.source(), &batch).unwrap(),
        Admission {
            added: 4,
            repeated: 1
        }
    );
    assert_eq!(
        run.submit_inputs(run.source(), &[first]),
        Err(RunError::OldInputSequence)
    );
    complete(&mut run, 900_007, WorkBudget::one());
    assert_eq!(run.snapshot().committed_judgements, [None; MAX_CIRCLES]);
    assert_eq!(run.snapshot().held_slots, [false; 4]);
    assert_eq!(run.snapshot().pending_events, 0);
}

#[test]
fn same_time_edges_follow_sequence_not_batch_position() {
    let batch = [
        release(4, 1_000_000, PhysicalSlot::PrimaryOne),
        press(3, 1_000_000, PhysicalSlot::PrimaryOne, 0),
        release(2, 1_000_000, PhysicalSlot::PrimaryOne),
        press(1, 1_000_000, PhysicalSlot::PrimaryOne, 11),
    ];
    let mut run = single(51, Limits::tiny());
    run.submit_inputs(run.source(), &batch).unwrap();
    complete(&mut run, 1_000_000, WorkBudget::one());
    assert_eq!(
        run.snapshot().committed_judgements[0].unwrap().result,
        Judgement::Hit
    );
    assert_eq!(run.snapshot().held_slots, [false; 4]);
    assert_eq!(run.snapshot().pending_events, 2);
}

type CanonicalEvent = (u64, ObjectId, MapTime, EventKind);
fn four_circle_trace(
    id: u128,
    limits: Limits,
    budget: WorkBudget,
    cancel_at: Option<usize>,
) -> (Vec<CanonicalEvent>, usize) {
    let circles = [
        circle(99, 0),
        circle(8, 200_001),
        circle(4, 400_002),
        circle(77, 1_000_000),
    ];
    let mut run = make_run(id, &circles, -200_001, limits);
    run.submit_inputs(
        run.source(),
        &[
            press(1, 100_001, PhysicalSlot::PrimaryOne, 0),
            release(2, 100_002, PhysicalSlot::PrimaryOne),
            press(3, 400_002, PhysicalSlot::SecondaryTwo, 0),
            release(4, 400_003, PhysicalSlot::SecondaryTwo),
        ],
    )
    .unwrap();
    run.seal_through(run.source(), time(1_300_000)).unwrap();
    let mut report = run.advance(run.source(), time(1_300_000), budget).unwrap();
    let mut events = Vec::new();
    for call in 0..256 {
        let (steps, cursor, backpressure) = match report {
            AdvanceReport::Complete { steps, .. } => (steps, None, false),
            AdvanceReport::Pending {
                cursor,
                reason,
                steps,
                ..
            } => (
                steps,
                Some(cursor),
                reason == PendingReason::EventBackpressure,
            ),
        };
        assert!(steps <= budget.get());
        if backpressure || cursor.is_none() {
            let mut read = Vec::new();
            collect_events(&mut run, &mut read);
            for event in read {
                assert_eq!(event.key.run(), run.source().run);
                assert_eq!(event.key.transport(), run.source().transport);
                assert_eq!(event.evidence, Evidence::SyntheticTest);
                events.push((
                    event.key.sequence().get(),
                    event.object,
                    event.at,
                    event.kind,
                ));
            }
        }
        let Some(cursor) = cursor else {
            let observation = run.snapshot();
            assert_eq!(observation.status, RunStatus::Ready);
            assert_eq!(observation.completed_through, Some(time(1_300_000)));
            assert_eq!(observation.pending_inputs, 0);
            assert_eq!(observation.pending_events, 0);
            assert_eq!(observation.held_slots, [false; 4]);
            for (index, (result, at)) in [
                (Judgement::Miss, 100_001),
                (Judgement::Hit, 100_001),
                (Judgement::Hit, 400_002),
                (Judgement::Miss, 1_100_001),
            ]
            .into_iter()
            .enumerate()
            {
                assert_eq!(
                    observation.committed_judgements[index],
                    Some(JudgementRecord {
                        object: circles[index].id(),
                        at: time(at),
                        result
                    })
                );
                assert_eq!(frame(&run, index).phase, Phase::Retired);
                assert_eq!(
                    frame(&run, index).judgement,
                    observation.committed_judgements[index]
                );
            }
            return (events, call);
        };
        if cancel_at == Some(call) {
            run.cancel_request(cursor.request()).unwrap();
            let before = run.snapshot();
            assert_eq!(
                run.continue_advance(cursor, budget),
                Err(RunError::StaleCursor)
            );
            assert_eq!(run.snapshot(), before);
            report = run.advance(run.source(), time(1_300_000), budget).unwrap();
        } else {
            report = run.continue_advance(cursor, budget).unwrap();
        }
    }
    panic!("four-circle trace exceeded its finite call bound");
}

#[test]
fn tiny_and_reference_agree_after_cancellation_at_every_yield() {
    let expected = vec![
        (
            1,
            ObjectId::new(8),
            time(100_001),
            EventKind::Judgement(Judgement::Hit),
        ),
        (2, ObjectId::new(8), time(100_001), EventKind::SoundIntent),
        (
            3,
            ObjectId::new(99),
            time(100_001),
            EventKind::Judgement(Judgement::Miss),
        ),
        (
            4,
            ObjectId::new(4),
            time(400_002),
            EventKind::Judgement(Judgement::Hit),
        ),
        (5, ObjectId::new(4), time(400_002), EventKind::SoundIntent),
        (
            6,
            ObjectId::new(77),
            time(1_100_001),
            EventKind::Judgement(Judgement::Miss),
        ),
    ];
    let configurations = [
        (Limits::tiny(), WorkBudget::one()),
        (Limits::reference(), WorkBudget::one()),
        (Limits::reference(), WorkBudget::full()),
    ];
    let mut id = 1_000;
    for (limits, budget) in configurations {
        id += 1;
        let (events, yields) = four_circle_trace(id, limits, budget, None);
        assert_eq!(events, expected);
        for cut in 0..yields {
            id += 1;
            assert_eq!(
                four_circle_trace(id, limits, budget, Some(cut)).0,
                expected,
                "{budget:?}/{cut}"
            );
        }
    }
}

#[test]
fn radius_uses_both_axes_relative_to_center_and_valid_time_endpoints_survive() {
    for (case, (dx, dy, expected)) in [
        (6, 8, Judgement::Hit),
        (-6, -8, Judgement::Hit),
        (6, 9, Judgement::Miss),
        (0, 11, Judgement::Miss),
    ]
    .into_iter()
    .enumerate()
    {
        let center = point(100, -100);
        let object = Circle::new(ObjectId::new(0), time(1_000_000), center).unwrap();
        let mut run = make_run(
            60 + u128::try_from(case).unwrap(),
            &[object],
            799_999,
            Limits::reference(),
        );
        let mut input = press(1, 1_000_000, PhysicalSlot::PrimaryOne, 0);
        input.edge = InputEdge::Press(point(100 + dx, -100 + dy));
        run.submit_inputs(run.source(), &[input]).unwrap();
        complete(&mut run, 1_300_000, WorkBudget::full());
        assert_eq!(
            run.snapshot().committed_judgements[0].unwrap().result,
            expected
        );
    }
    for (id, target) in [(70, i64::MIN + 200_000), (71, i64::MAX - 300_000)] {
        let object = circle(0, target);
        let start = target - 200_000;
        let mut run = make_run(id, &[object], start, Limits::reference());
        run.submit_inputs(
            run.source(),
            &[press(1, target, PhysicalSlot::PrimaryOne, 0)],
        )
        .unwrap();
        complete(&mut run, target + 300_000, WorkBudget::full());
        assert_eq!(
            run.snapshot().committed_judgements[0].unwrap().result,
            Judgement::Hit
        );
        assert_eq!(frame(&run, 0).phase, Phase::Retired);
    }
    let map = SyntheticMap::try_new(MapRevision::new(8), &[circle(0, 1_000_000)]).unwrap();
    assert!(matches!(
        SyntheticRun::new(source(72), map, time(799_999), Limits::tiny()),
        Err(ValueError::RevisionMismatch)
    ));
    let map = SyntheticMap::try_new(MapRevision::new(7), &[circle(0, 1_000_000)]).unwrap();
    assert!(matches!(
        SyntheticRun::new(source(73), map, time(800_001), Limits::tiny()),
        Err(ValueError::InvalidStart)
    ));
}

#[test]
fn call_errors_and_foreign_tokens_leave_the_active_continuation_usable() {
    let mut first = single(80, Limits::tiny());
    let before = first.snapshot();
    assert_eq!(
        first.advance(first.source(), time(799_998), WorkBudget::one()),
        Err(RunError::InvalidTarget)
    );
    assert_eq!(first.snapshot(), before);
    first.seal_through(first.source(), time(1_000_000)).unwrap();
    let AdvanceReport::Pending {
        cursor: foreign, ..
    } = first
        .advance(first.source(), time(1_000_000), WorkBudget::one())
        .unwrap()
    else {
        panic!()
    };
    let mut second = single(81, Limits::tiny());
    second
        .seal_through(second.source(), time(1_000_000))
        .unwrap();
    let AdvanceReport::Pending { cursor, .. } = second
        .advance(second.source(), time(1_000_000), WorkBudget::one())
        .unwrap()
    else {
        panic!()
    };
    let before = second.snapshot();
    assert_eq!(
        second.continue_advance(foreign, WorkBudget::full()),
        Err(RunError::StaleSource)
    );
    assert_eq!(
        second.cancel_request(foreign.request()),
        Err(RunError::StaleSource)
    );
    assert_eq!(
        second.continue_advance(cursor, WorkBudget::full()),
        Err(RunError::BudgetLimit)
    );
    assert_eq!(
        second.advance(second.source(), time(1_000_000), WorkBudget::one()),
        Err(RunError::Busy)
    );
    assert_eq!(
        second.seal_through(second.source(), time(999_999)),
        Err(RunError::InvalidTarget)
    );
    assert_eq!(second.snapshot(), before);
    assert!(second.continue_advance(cursor, WorkBudget::one()).is_ok());
    second.cancel_request(cursor.request()).unwrap();
    first.cancel_request(foreign.request()).unwrap();
}
