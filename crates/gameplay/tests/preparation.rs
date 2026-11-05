#[allow(dead_code)]
mod common;
use common::SnapshotExt;
use skin_editor_gameplay::*;

fn time(value: i64) -> MapTime {
    MapTime::from_micros(value)
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
        .map(|(i, t)| {
            Circle::new(
                ObjectId::new(u32::try_from(i).unwrap()),
                time(*t),
                Point::new(0, 0).unwrap(),
            )
            .unwrap()
        })
        .collect()
}
fn owner(id: u128) -> SyntheticCoordinator {
    SyntheticCoordinator::new(
        CoordinatorId::new(id).unwrap(),
        Limits::tiny(),
        PreparationLimits::tiny(),
    )
    .unwrap()
}
fn current(owner: &SyntheticCoordinator) -> RunSource {
    owner.current().unwrap().source()
}
fn complete(task: &mut PreparationTask<'_>, budget: WorkBudget) -> PreparationKey {
    let mut cursor = task.cursor();
    for _ in 0..256 {
        match task.step(cursor, budget).unwrap() {
            PreparationReport::Pending {
                cursor: next,
                steps,
                ..
            } => {
                assert!(steps > 0 && steps <= budget.get());
                cursor = next;
            }
            PreparationReport::Ready { key, source, steps } => {
                assert!(steps > 0 && steps <= budget.get());
                assert_eq!(key.target(), source);
                return key;
            }
        }
    }
    panic!("preparation exceeded fixture bound");
}
fn create(
    owner: &mut SyntheticCoordinator,
    id: u128,
    input: &[Circle],
    start: i64,
    budget: WorkBudget,
) {
    let mut task = owner
        .request_create(source(id), input, time(start))
        .unwrap();
    let key = complete(&mut task, budget);
    owner.activate(key, &mut task).unwrap();
}
fn freeze(owner: &mut SyntheticCoordinator, at: i64, budget: WorkBudget) {
    let source = current(owner);
    owner.request_freeze(source, time(at)).unwrap();
    owner.seal_through(source, time(at)).unwrap();
    for _ in 0..256 {
        match owner.settle_freeze(source, budget).unwrap() {
            FreezeReport::Frozen { .. } => return,
            FreezeReport::Pending { reason, .. } => assert_eq!(reason, PendingReason::WorkBudget),
        }
    }
    panic!("freeze exceeded fixture bound");
}
fn advance(owner: &mut SyntheticCoordinator, at: i64, budget: WorkBudget) {
    let source = current(owner);
    owner.seal_through(source, time(at)).unwrap();
    let mut report = owner.advance(source, time(at), budget).unwrap();
    for _ in 0..256 {
        match report {
            AdvanceReport::Complete { .. } => return,
            AdvanceReport::Pending { cursor, reason, .. } => {
                assert_eq!(reason, PendingReason::WorkBudget);
                report = owner.continue_advance(cursor, budget).unwrap();
            }
        }
    }
    panic!("advance exceeded fixture bound");
}
fn press(sequence: u64, at: i64) -> InputEvent {
    InputEvent {
        sequence: InputSequence::new(sequence).unwrap(),
        at: time(at),
        slot: PhysicalSlot::PrimaryOne,
        edge: InputEdge::Press(Point::new(0, 0).unwrap()),
    }
}
fn inspect(owner: &mut SyntheticCoordinator, at: i64, budget: WorkBudget) {
    let mut task = owner.request_inspection(current(owner), time(at)).unwrap();
    let key = complete(&mut task, budget);
    owner.activate(key, &mut task).unwrap();
}
fn limited(id: u128, sources: usize, runs: usize, tasks: usize) -> SyntheticCoordinator {
    let c = SyntheticCoordinator::costs();
    let limits = PreparationLimits::try_new(
        sources * c.source_bytes,
        c.coordinator_bytes + c.pool_bytes + runs * c.run_bytes + tasks * c.task_bytes,
        1,
    )
    .unwrap();
    SyntheticCoordinator::new(CoordinatorId::new(id).unwrap(), Limits::tiny(), limits).unwrap()
}

#[test]
fn create_only_activates_complete_validated_runtime_and_initial_section_stays_uncomputed() {
    let mut owner = owner(70_001);
    let input = circles(&[0, 1_000_000]);
    let before = owner.resources();
    let mut task = owner
        .request_create(source(1), &input, time(-200_001))
        .unwrap();
    let key = task.key();
    assert_eq!(key.base(), None);
    assert_eq!(
        key.operation(),
        PreparationOperation::Create {
            start: time(-200_001)
        }
    );
    assert_eq!(
        owner.activate(key, &mut task),
        Err(PreparationError::NotReady)
    );
    assert!(owner.current().is_none());
    complete(&mut task, WorkBudget::one());
    assert!(owner.current().is_none());
    assert_eq!(owner.activate(key, &mut task), Ok(source(1)));
    let state = owner.current().unwrap().snapshot();
    assert_eq!(state.status, RunStatus::Prepared);
    assert_eq!(state.ready_frame, None);
    assert_eq!(state.completed_through, None);
    assert_eq!(state.sealed_through, None);
    assert_eq!(owner.candidate(), None);
    let cost = SyntheticCoordinator::costs();
    assert_eq!(
        owner.resources().runtime_bytes,
        before.runtime_bytes + cost.task_bytes + cost.run_bytes
    );
    drop(task);
    assert_eq!(
        owner.resources().runtime_bytes,
        before.runtime_bytes + cost.run_bytes
    );
    assert_eq!(owner.resources().precompiled_bytes, cost.source_bytes);
    owner
        .submit_inputs(source(1), &[press(1, -200_001)])
        .unwrap();
    advance(&mut owner, -200_001, WorkBudget::one());
    assert_eq!(
        owner.current().unwrap().snapshot().completed_through,
        Some(time(-200_001))
    );
}

#[test]
fn one_eligible_slot_covers_load_and_inspection_without_publishing_early() {
    let mut owner = limited(70_002, 2, 3, 3);
    let input = circles(&[0, 1_000_000]);
    let replacement = circles(&[2_000_000]);
    create(&mut owner, 2, &input, -200_001, WorkBudget::one());
    freeze(&mut owner, -200_001, WorkBudget::one());
    let before = owner.current().unwrap().snapshot();
    let mut load = owner
        .request_load(current(&owner), source(3), &replacement, time(1_799_999))
        .unwrap();
    assert!(matches!(
        owner.request_inspection(current(&owner), time(0)),
        Err(PreparationError::Busy)
    ));
    complete(&mut load, WorkBudget::one());
    assert_eq!(owner.current().unwrap().snapshot(), before);
    owner.cancel(load.key()).unwrap();
    let mut inspection = owner.request_inspection(current(&owner), time(0)).unwrap();
    assert!(matches!(
        owner.request_load(current(&owner), source(3), &replacement, time(1_799_999)),
        Err(PreparationError::Busy)
    ));
    assert!(matches!(
        owner.request_create(source(3), &replacement, time(1_799_999)),
        Err(PreparationError::InvalidOperation)
    ));
    complete(&mut inspection, WorkBudget::one());
    assert_eq!(
        owner.activate(load.key(), &mut load),
        Err(PreparationError::StaleRequest)
    );
    assert_eq!(owner.current().unwrap().snapshot(), before);
    let source = owner.activate(inspection.key(), &mut inspection).unwrap();
    assert_eq!(source.transport.get(), 2);
    assert_eq!(source.input.get(), 2);
    assert_eq!(
        owner.current().unwrap().snapshot().completed_through,
        Some(time(0))
    );
    assert_eq!(
        owner.resources().precompiled_bytes,
        2 * SyntheticCoordinator::costs().source_bytes
    );
    drop(load);
    assert_eq!(
        owner.resources().precompiled_bytes,
        SyntheticCoordinator::costs().source_bytes
    );
}

#[test]
fn cancelled_create_and_its_drop_cannot_displace_the_replacement() {
    let mut owner = limited(70_003, 2, 2, 2);
    let input = circles(&[0]);
    let mut a = owner
        .request_create(source(4), &input, time(-200_001))
        .unwrap();
    let a_cursor = a.cursor();
    owner.cancel(a.key()).unwrap();
    let mut b = owner
        .request_create(source(4), &input, time(-200_001))
        .unwrap();
    assert_ne!(a.key(), b.key());
    assert_eq!(
        a.step(a_cursor, WorkBudget::one()),
        Err(PreparationError::StaleRequest)
    );
    let b_key = b.key();
    assert_eq!(
        owner.activate(b_key, &mut a),
        Err(PreparationError::StaleRequest)
    );
    assert_eq!(owner.cancel(a.key()), Err(PreparationError::StaleRequest));
    drop(a);
    assert_eq!(owner.candidate(), Some(b_key));
    complete(&mut b, WorkBudget::one());
    owner.activate(b_key, &mut b).unwrap();
    assert_eq!(current(&owner), source(4));
}

#[test]
fn dropping_eligible_work_releases_it_but_stale_drop_does_not_cancel_new_work() {
    let mut owner = limited(70_004, 2, 2, 2);
    let input = circles(&[0]);
    let before = owner.resources();
    let a = owner
        .request_create(source(5), &input, time(-200_001))
        .unwrap();
    drop(a);
    assert_eq!(owner.candidate(), None);
    assert_eq!(owner.resources(), before);
    let b = owner
        .request_create(source(5), &input, time(-200_001))
        .unwrap();
    owner.cancel(b.key()).unwrap();
    let c = owner
        .request_create(source(5), &input, time(-200_001))
        .unwrap();
    drop(b);
    assert_eq!(owner.candidate(), Some(c.key()));
    drop(c);
    assert_eq!(owner.resources(), before);
}

#[test]
fn invalid_cursor_and_budget_preserve_the_valid_request() {
    let mut owner = owner(70_005);
    let input = circles(&[0, 1_000_000, 2_000_000, 3_000_000]);
    let mut task = owner
        .request_create(source(6), &input, time(-200_001))
        .unwrap();
    let first = task.cursor();
    assert_eq!(
        task.step(first, WorkBudget::full()),
        Err(PreparationError::BudgetLimit)
    );
    assert_eq!(task.cursor(), first);
    task.step(first, WorkBudget::one()).unwrap();
    let latest = task.cursor();
    let usage = owner.resources();
    assert_eq!(
        task.step(first, WorkBudget::one()),
        Err(PreparationError::StaleCursor)
    );
    assert_eq!(task.cursor(), latest);
    assert_eq!(
        owner.activate(task.key(), &mut task),
        Err(PreparationError::NotReady)
    );
    assert_eq!(owner.resources(), usage);
    complete(&mut task, WorkBudget::one());
    owner.activate(task.key(), &mut task).unwrap();
}

#[test]
fn foreign_result_is_rejected_even_when_external_owner_ids_and_keys_match() {
    // Deliberately violate the caller ID uniqueness contract to ensure a result
    // from a different actual pool cannot steal the matching active request.
    let mut a = owner(70_006);
    let mut b = owner(70_006);
    let input = circles(&[0]);
    let mut task_a = a.request_create(source(7), &input, time(-200_001)).unwrap();
    let mut task_b = b.request_create(source(7), &input, time(-200_001)).unwrap();
    complete(&mut task_a, WorkBudget::one());
    complete(&mut task_b, WorkBudget::one());
    assert_eq!(task_a.key(), task_b.key());
    let before = b.resources();
    assert_eq!(
        b.activate(task_b.key(), &mut task_a),
        Err(PreparationError::StaleRequest)
    );
    assert_eq!(b.resources(), before);
    assert_eq!(b.candidate(), Some(task_b.key()));
    b.activate(task_b.key(), &mut task_b).unwrap();
    assert!(a.current().is_none());
}

#[test]
fn invalid_map_or_start_cannot_replace_current_run_and_retained_failure_stays_charged() {
    let input = circles(&[0]);
    let good = circles(&[1_000_000]);
    let duplicate = [input[0], input[0]];
    let unsorted = [
        Circle::new(
            ObjectId::new(99),
            time(1_000_000),
            Point::new(0, 0).unwrap(),
        )
        .unwrap(),
        input[0],
    ];
    let overlap = circles(&[0, 1]);
    for (index, candidate, start, expected) in [
        (&duplicate[..], -200_001, ValueError::DuplicateObject),
        (&unsorted[..], -200_001, ValueError::UnsortedMap),
        (&overlap[..], -200_001, ValueError::UnsupportedOverlap),
        (&good[..], 800_001, ValueError::InvalidStart),
    ]
    .into_iter()
    .enumerate()
    .map(|(i, (a, b, c))| (i, a, b, c))
    {
        let mut owner = owner(70_010 + index as u128);
        create(&mut owner, 10, &input, -200_001, WorkBudget::one());
        freeze(&mut owner, -200_001, WorkBudget::one());
        let before = owner.current().unwrap().snapshot();
        let usage = owner.resources();
        let mut task = owner
            .request_load(current(&owner), source(11), candidate, time(start))
            .unwrap();
        let charged = owner.resources();
        let mut failure = None;
        for _ in 0..64 {
            match task.step(task.cursor(), WorkBudget::one()) {
                Err(error) => {
                    failure = Some(error);
                    break;
                }
                Ok(PreparationReport::Pending { .. }) => {}
                Ok(PreparationReport::Ready { .. }) => panic!("invalid candidate ready"),
            }
        }
        assert_eq!(failure, Some(PreparationError::Invalid(expected)));
        assert_eq!(task.stage(), PreparationStage::Failed);
        assert_eq!(owner.candidate(), None);
        assert_eq!(owner.resources(), charged);
        assert_eq!(
            owner.activate(task.key(), &mut task),
            Err(PreparationError::StaleRequest)
        );
        assert_eq!(owner.current().unwrap().snapshot(), before);
        drop(task);
        assert_eq!(owner.resources(), usage);
    }
}

#[test]
fn oversized_input_and_wrong_base_are_rejected_before_reserving_or_cancelling() {
    let mut owner = owner(70_020);
    let input = circles(&[0]);
    let too_many = circles(&[0, 1_000_000, 2_000_000, 3_000_000, 4_000_000]);
    let usage = owner.resources();
    assert!(matches!(
        owner.request_create(source(12), &too_many, time(-200_001)),
        Err(PreparationError::Invalid(ValueError::ObjectLimit))
    ));
    assert_eq!(owner.resources(), usage);
    create(&mut owner, 12, &input, -200_001, WorkBudget::one());
    freeze(&mut owner, -200_001, WorkBudget::one());
    let task = owner.request_inspection(current(&owner), time(0)).unwrap();
    let before = owner.resources();
    let wrong = RunSource {
        revision: MapRevision::new(99),
        ..current(&owner)
    };
    assert!(matches!(
        owner.request_load(wrong, source(13), &input, time(-200_001)),
        Err(PreparationError::Runtime(RunError::StaleSource))
    ));
    assert!(matches!(
        owner.request_inspection(wrong, time(0)),
        Err(PreparationError::Runtime(RunError::StaleSource))
    ));
    assert_eq!(owner.resources(), before);
    assert_eq!(owner.candidate(), Some(task.key()));
}

#[test]
fn cancellation_keeps_credit_until_the_held_task_is_dropped() {
    let mut owner = limited(70_021, 2, 2, 2);
    let input = circles(&[0]);
    create(&mut owner, 14, &input, -200_001, WorkBudget::one());
    freeze(&mut owner, -200_001, WorkBudget::one());
    let before = owner.resources();
    let old = owner.request_inspection(current(&owner), time(0)).unwrap();
    let occupied = owner.resources();
    owner.cancel(old.key()).unwrap();
    assert_eq!(owner.resources(), occupied);
    assert!(matches!(
        owner.request_inspection(current(&owner), time(1)),
        Err(PreparationError::Budget {
            pool: ResourcePool::Runtime,
            ..
        })
    ));
    assert_eq!(owner.resources(), occupied);
    assert_eq!(owner.candidate(), None);
    drop(old);
    assert_eq!(owner.resources(), before);
    let _next = owner.request_inspection(current(&owner), time(1)).unwrap();
}

#[test]
fn source_budget_refuses_load_atomically_but_allows_shared_inspection() {
    let mut owner = limited(70_022, 1, 2, 2);
    let input = circles(&[0]);
    create(&mut owner, 15, &input, -200_001, WorkBudget::one());
    freeze(&mut owner, -200_001, WorkBudget::one());
    let before = owner.resources();
    assert!(matches!(
        owner.request_load(current(&owner), source(16), &input, time(-200_001)),
        Err(PreparationError::Budget {
            pool: ResourcePool::Precompiled,
            ..
        })
    ));
    assert_eq!(owner.resources(), before);
    let mut task = owner.request_inspection(current(&owner), time(0)).unwrap();
    complete(&mut task, WorkBudget::one());
    assert_eq!(
        owner.resources().precompiled_bytes,
        before.precompiled_bytes
    );
    owner.activate(task.key(), &mut task).unwrap();
    drop(task);
    assert_eq!(owner.resources(), before);
}

#[test]
fn shared_old_source_survives_replacement_until_the_last_stale_task_drops() {
    let mut owner = limited(70_023, 2, 3, 3);
    let input = circles(&[0]);
    let replacement = circles(&[1_000_000]);
    create(&mut owner, 17, &input, -200_001, WorkBudget::one());
    freeze(&mut owner, -200_001, WorkBudget::one());
    let mut old = owner.request_inspection(current(&owner), time(0)).unwrap();
    let cursor = old.cursor();
    owner.cancel(old.key()).unwrap();
    let mut new = owner
        .request_load(current(&owner), source(18), &replacement, time(799_999))
        .unwrap();
    complete(&mut new, WorkBudget::one());
    let cost = SyntheticCoordinator::costs();
    assert_eq!(owner.resources().precompiled_bytes, 2 * cost.source_bytes);
    owner.activate(new.key(), &mut new).unwrap();
    drop(new);
    assert_eq!(owner.resources().precompiled_bytes, 2 * cost.source_bytes);
    assert_eq!(
        old.step(cursor, WorkBudget::one()),
        Err(PreparationError::StaleRequest)
    );
    drop(old);
    assert_eq!(owner.resources().precompiled_bytes, cost.source_bytes);
}

#[test]
fn owner_drop_invalidates_work_but_keeps_its_actual_reservations() {
    let mut owner = owner(70_024);
    let input = circles(&[0]);
    create(&mut owner, 19, &input, -200_001, WorkBudget::one());
    freeze(&mut owner, -200_001, WorkBudget::one());
    let mut task = owner.request_inspection(current(&owner), time(0)).unwrap();
    let cursor = task.cursor();
    let cost = SyntheticCoordinator::costs();
    drop(owner);
    assert_eq!(
        task.step(cursor, WorkBudget::one()),
        Err(PreparationError::StaleRequest)
    );
    assert_eq!(
        task.resources(),
        ResourceUsage {
            precompiled_bytes: cost.source_bytes,
            runtime_bytes: cost.pool_bytes + cost.task_bytes + cost.run_bytes
        }
    );
}

#[test]
fn inspection_boundaries_and_resume_keep_the_declared_semantics() {
    for (index, at) in [i64::MIN, -1, 1_000_000, 1_100_000, 1_100_001, i64::MAX]
        .into_iter()
        .enumerate()
    {
        let mut owner = owner(70_030 + index as u128);
        let input = circles(&[1_000_000]);
        create(&mut owner, 20, &input, 799_999, WorkBudget::one());
        freeze(&mut owner, 799_999, WorkBudget::one());
        let old = current(&owner);
        inspect(&mut owner, at, WorkBudget::one());
        let state = owner.current().unwrap().snapshot();
        assert_eq!(state.status, RunStatus::Frozen);
        assert_eq!(state.completed_through, Some(time(at)));
        assert_eq!(state.sealed_through, Some(time(at)));
        assert_eq!(
            state.statistics_origin,
            StatisticsOrigin::Inspection { at: time(at) }
        );
        assert_eq!(state.committed_judgements, [None; MAX_CIRCLES]);
        assert_eq!(state.pending_events, 0);
        assert_eq!(
            state.ready_frame.unwrap().object(0).unwrap().unavailable,
            (at > 1_100_000).then_some(UnavailableReason::InspectionSkipped)
        );
        assert_eq!(current(&owner).transport.get(), old.transport.get() + 1);
        if at == 1_000_000 {
            let source = owner.resume(current(&owner)).unwrap();
            assert_eq!(
                owner.submit_inputs(old, &[press(2, 1_000_001)]),
                Err(PreparationError::Runtime(RunError::StaleSource))
            );
            assert_eq!(
                owner.submit_inputs(source, &[press(2, 1_000_001)]),
                Err(PreparationError::Runtime(RunError::RearmRequired))
            );
            owner
                .observe_release(
                    source,
                    ReleaseObservation {
                        sequence: InputSequence::new(1).unwrap(),
                        slot: PhysicalSlot::PrimaryOne,
                    },
                )
                .unwrap();
            owner.submit_inputs(source, &[press(2, 1_000_001)]).unwrap();
            advance(&mut owner, 1_000_001, WorkBudget::one());
            assert_eq!(
                owner.current().unwrap().snapshot().committed_judgements[0]
                    .unwrap()
                    .result,
                Judgement::Hit
            );
            assert_eq!(
                owner
                    .current()
                    .unwrap()
                    .event_at(0)
                    .unwrap()
                    .key
                    .sequence()
                    .get(),
                1
            );
        }
    }
}

#[test]
fn resume_and_input_epoch_changes_do_not_revive_a_cancelled_inspection() {
    let mut owner = limited(70_040, 2, 3, 3);
    let input = circles(&[0]);
    create(&mut owner, 21, &input, -200_001, WorkBudget::one());
    freeze(&mut owner, -200_001, WorkBudget::one());
    let mut a = owner.request_inspection(current(&owner), time(0)).unwrap();
    complete(&mut a, WorkBudget::one());
    let before = owner.current().unwrap().snapshot();
    assert_eq!(owner.resume(current(&owner)), Err(PreparationError::Busy));
    assert_eq!(owner.current().unwrap().snapshot(), before);
    owner.cancel(a.key()).unwrap();
    owner.resume(current(&owner)).unwrap();
    freeze(&mut owner, -200_001, WorkBudget::one());
    let mut b = owner.request_inspection(current(&owner), time(0)).unwrap();
    assert_eq!(a.key().target().transport, b.key().target().transport);
    assert_ne!(a.key().target().input, b.key().target().input);
    assert_eq!(
        owner.activate(a.key(), &mut a),
        Err(PreparationError::StaleRequest)
    );
    complete(&mut b, WorkBudget::one());
    owner.activate(b.key(), &mut b).unwrap();
}

#[test]
fn base_fault_invalidates_inspection_then_explicit_load_can_recover() {
    let mut owner = owner(70_041);
    let input = circles(&[0]);
    create(&mut owner, 22, &input, -200_001, WorkBudget::one());
    let base = current(&owner);
    owner.submit_inputs(base, &[press(1, 0)]).unwrap();
    freeze(&mut owner, 0, WorkBudget::one());
    let mut task = owner.request_inspection(base, time(0)).unwrap();
    complete(&mut task, WorkBudget::one());
    let held = owner.resources();
    let conflict = InputEvent {
        edge: InputEdge::Release,
        ..press(1, 0)
    };
    assert_eq!(
        owner.submit_inputs(base, &[conflict]),
        Err(PreparationError::Runtime(RunError::InputConflict))
    );
    assert_eq!(owner.candidate(), None);
    assert_eq!(owner.resources(), held);
    assert_eq!(
        owner.activate(task.key(), &mut task),
        Err(PreparationError::StaleRequest)
    );
    drop(task);
    let mut load = owner
        .request_load(base, source(23), &input, time(-200_001))
        .unwrap();
    complete(&mut load, WorkBudget::one());
    owner.activate(load.key(), &mut load).unwrap();
    assert_eq!(
        owner.current().unwrap().snapshot().status,
        RunStatus::Prepared
    );
    assert_eq!(current(&owner), source(23));
}

#[test]
fn runtime_and_source_limits_reject_even_a_one_byte_short_initial_reservation() {
    let cost = SyntheticCoordinator::costs();
    let runtime = cost.pool_bytes + cost.coordinator_bytes + cost.task_bytes + cost.run_bytes;
    assert_eq!(
        PreparationLimits::try_new(cost.source_bytes, runtime, 0),
        Err(ValueError::InvalidLimits)
    );
    assert_eq!(
        PreparationLimits::try_new(cost.source_bytes, runtime, 257),
        Err(ValueError::InvalidLimits)
    );
    assert_eq!(
        PreparationLimits::try_new(cost.source_bytes, runtime - 1, 1),
        Err(ValueError::StorageLimit)
    );
    assert_eq!(
        PreparationLimits::try_new(cost.source_bytes - 1, runtime, 1),
        Err(ValueError::StorageLimit)
    );
    let limits = PreparationLimits::try_new(cost.source_bytes, runtime, 1).unwrap();
    let mut owner =
        SyntheticCoordinator::new(CoordinatorId::new(70_042).unwrap(), Limits::tiny(), limits)
            .unwrap();
    let input = circles(&[0]);
    create(&mut owner, 24, &input, -200_001, WorkBudget::one());
    assert!(owner.resources().runtime_bytes <= runtime);
    assert_eq!(owner.resources().precompiled_bytes, cost.source_bytes);
}

#[test]
fn preparation_work_one_and_full_produce_the_same_run_and_events() {
    let input = circles(&[0, 1_000_000, 2_000_000, 3_000_000]);
    let mut a = owner(70_043);
    let mut b = SyntheticCoordinator::new(
        CoordinatorId::new(70_044).unwrap(),
        Limits::reference(),
        PreparationLimits::reference(),
    )
    .unwrap();
    for (owner, budget) in [(&mut a, WorkBudget::one()), (&mut b, WorkBudget::full())] {
        create(owner, 25, &input, -200_001, budget);
        freeze(owner, -200_001, budget);
        inspect(owner, 1_000_000, budget);
        let source = owner.resume(current(owner)).unwrap();
        owner
            .observe_release(
                source,
                ReleaseObservation {
                    sequence: InputSequence::new(1).unwrap(),
                    slot: PhysicalSlot::PrimaryOne,
                },
            )
            .unwrap();
        owner.submit_inputs(source, &[press(2, 1_000_001)]).unwrap();
        advance(owner, 1_000_001, budget);
    }
    assert_eq!(
        a.current().unwrap().snapshot(),
        b.current().unwrap().snapshot()
    );
    assert_eq!(
        a.current().unwrap().event_at(0),
        b.current().unwrap().event_at(0)
    );
    assert_eq!(
        a.current().unwrap().event_at(1),
        b.current().unwrap().event_at(1)
    );
}

#[test]
fn load_publishes_a_fresh_prepared_run_and_rejects_old_inputs_events_and_cursors() {
    let mut owner = owner(70_050);
    let input = circles(&[0]);
    let replacement = circles(&[1_000_000]);
    create(&mut owner, 30, &input, -200_001, WorkBudget::one());
    let old = current(&owner);
    owner.submit_inputs(old, &[press(1, 0)]).unwrap();
    owner.seal_through(old, time(0)).unwrap();
    let AdvanceReport::Pending { cursor, .. } =
        owner.advance(old, time(0), WorkBudget::one()).unwrap()
    else {
        panic!("bounded advance");
    };
    freeze(&mut owner, 0, WorkBudget::one());
    let event = owner.current().unwrap().event_at(0).unwrap();
    let before = owner.current().unwrap().snapshot();
    let usage = owner.resources();
    assert!(matches!(
        owner.request_load(old, old, &replacement, time(799_999)),
        Err(PreparationError::RunIdReuse)
    ));
    assert_eq!(owner.resources(), usage);
    let mut task = owner
        .request_load(old, source(31), &replacement, time(799_999))
        .unwrap();
    complete(&mut task, WorkBudget::one());
    assert_eq!(owner.current().unwrap().snapshot(), before);
    owner.activate(task.key(), &mut task).unwrap();
    let fresh = owner.current().unwrap().snapshot();
    assert_eq!(fresh.status, RunStatus::Prepared);
    assert_eq!(fresh.ready_frame, None);
    assert_eq!(fresh.committed_judgements, [None; MAX_CIRCLES]);
    assert_eq!(fresh.pending_events, 0);
    assert_eq!(
        owner.acknowledge(event.key),
        Err(PreparationError::Runtime(RunError::StaleSource))
    );
    assert_eq!(
        owner.continue_advance(cursor, WorkBudget::one()),
        Err(PreparationError::Runtime(RunError::StaleSource))
    );
    assert_eq!(
        owner.cancel_advance(cursor.request()),
        Err(PreparationError::Runtime(RunError::StaleSource))
    );
    assert_eq!(
        owner.submit_inputs(old, &[press(1, 0)]),
        Err(PreparationError::Runtime(RunError::StaleSource))
    );
    assert_eq!(owner.current().unwrap().snapshot(), fresh);
    owner
        .submit_inputs(source(31), &[press(1, 1_000_000)])
        .unwrap();
    advance(&mut owner, 1_000_000, WorkBudget::one());
    let new = owner.current().unwrap().event_at(0).unwrap();
    assert_eq!(new.key.sequence(), event.key.sequence());
    assert_ne!(new.key.run(), event.key.run());
    assert_eq!(new.kind, EventKind::Judgement(Judgement::Hit));
}

#[test]
fn event_acknowledgements_survive_preparation_cancellation_without_replaying_sound() {
    let mut owner = owner(70_051);
    let input = circles(&[0, 1_000_000]);
    create(&mut owner, 32, &input, -200_001, WorkBudget::one());
    let source = current(&owner);
    owner.submit_inputs(source, &[press(1, 0)]).unwrap();
    freeze(&mut owner, 0, WorkBudget::one());
    let judgement = owner.current().unwrap().event_at(0).unwrap();
    let sound = owner.current().unwrap().event_at(1).unwrap();
    let mut task = owner.request_inspection(source, time(1_000_000)).unwrap();
    task.step(task.cursor(), WorkBudget::one()).unwrap();
    owner.acknowledge(judgement.key).unwrap();
    assert_eq!(owner.candidate(), Some(task.key()));
    let before = owner.current().unwrap().snapshot();
    owner.cancel(task.key()).unwrap();
    assert_eq!(owner.current().unwrap().snapshot(), before);
    assert_eq!(
        owner.acknowledge(judgement.key),
        Ok(Acknowledgement::AlreadyRetired)
    );
    owner.resume(source).unwrap();
    assert!(
        !owner
            .current()
            .unwrap()
            .sound_intent_allowed(sound.key)
            .unwrap()
    );
    owner.acknowledge(sound.key).unwrap();
    assert_eq!(owner.current().unwrap().event_at(0), None);
}

#[test]
fn empty_maps_support_all_three_operations_without_sentinel_time_arithmetic() {
    let mut owner = owner(70_052);
    create(&mut owner, 33, &[], i64::MIN, WorkBudget::one());
    assert_eq!(owner.current().unwrap().snapshot().ready_frame, None);
    freeze(&mut owner, i64::MIN, WorkBudget::one());
    inspect(&mut owner, i64::MAX, WorkBudget::one());
    let state = owner.current().unwrap().snapshot();
    assert_eq!(state.completed_through, Some(time(i64::MAX)));
    assert!(state.ready_frame.unwrap().is_empty());
    assert_eq!(state.pending_events, 0);
    let mut load = owner
        .request_load(current(&owner), source(34), &[], time(0))
        .unwrap();
    complete(&mut load, WorkBudget::one());
    owner.activate(load.key(), &mut load).unwrap();
    let state = owner.current().unwrap().snapshot();
    assert_eq!(state.status, RunStatus::Prepared);
    assert_eq!(state.completed_through, None);
}

fn operation_owner(
    id: u128,
    operation: PreparationOperation,
    input: &[Circle],
) -> SyntheticCoordinator {
    let mut owner = limited(id, 3, 3, 2);
    if !matches!(operation, PreparationOperation::Create { .. }) {
        create(&mut owner, 40, input, -200_001, WorkBudget::one());
        owner
            .submit_inputs(current(&owner), &[press(1, 0)])
            .unwrap();
        freeze(&mut owner, 0, WorkBudget::one());
        assert_eq!(owner.current().unwrap().snapshot().pending_events, 2);
    }
    owner
}

fn request_operation<'a>(
    owner: &mut SyntheticCoordinator,
    operation: PreparationOperation,
    input: &'a [Circle],
) -> PreparationTask<'a> {
    match operation {
        PreparationOperation::Create { start } => owner.request_create(source(41), input, start),
        PreparationOperation::Load { start } => {
            owner.request_load(current(owner), source(41), input, start)
        }
        PreparationOperation::Inspect { at } => owner.request_inspection(current(owner), at),
    }
    .unwrap()
}

fn operations() -> [PreparationOperation; 3] {
    [
        PreparationOperation::Create {
            start: time(-200_001),
        },
        PreparationOperation::Load {
            start: time(-200_001),
        },
        PreparationOperation::Inspect {
            at: time(1_000_000),
        },
    ]
}

#[test]
fn cancellation_at_every_preparation_return_point_isolates_late_work_after_replacement() {
    let input = circles(&[0, 1_000_000, 2_000_000, 3_000_000]);
    let cost = SyntheticCoordinator::costs();
    for operation in operations() {
        let mut reached_ready = false;
        for cut in 0..128 {
            let mut owner = operation_owner(71_001, operation, &input);
            let before = owner.current().map(SnapshotExt::snapshot);
            let events = owner
                .current()
                .map(|run| [run.event_at(0), run.event_at(1)]);
            let mut old = request_operation(&mut owner, operation, &input);
            for _ in 0..cut {
                if old.stage() == PreparationStage::Ready {
                    break;
                }
                old.step(old.cursor(), WorkBudget::one()).unwrap();
            }
            reached_ready = old.stage() == PreparationStage::Ready;
            let cursor = old.cursor();
            let held = owner.resources();
            owner.cancel(old.key()).unwrap();
            assert_eq!(owner.resources(), held);
            assert_eq!(owner.current().map(SnapshotExt::snapshot), before);
            assert_eq!(
                owner
                    .current()
                    .map(|run| [run.event_at(0), run.event_at(1)]),
                events
            );
            let mut new = request_operation(&mut owner, operation, &input);
            assert_ne!(old.key(), new.key());
            assert_eq!(old.key().target(), new.key().target());
            complete(&mut new, WorkBudget::one());
            assert_eq!(
                old.step(cursor, WorkBudget::one()),
                Err(PreparationError::StaleRequest)
            );
            assert_eq!(
                owner.activate(old.key(), &mut old),
                Err(PreparationError::StaleRequest)
            );
            assert_eq!(
                owner.activate(new.key(), &mut old),
                Err(PreparationError::StaleRequest)
            );
            assert_eq!(owner.candidate(), Some(new.key()));
            assert_eq!(owner.current().map(SnapshotExt::snapshot), before);
            owner.activate(new.key(), &mut new).unwrap();
            let published = owner.current().unwrap().snapshot();
            let occupied = owner.resources();
            assert_eq!(
                old.step(cursor, WorkBudget::one()),
                Err(PreparationError::StaleRequest)
            );
            assert_eq!(
                owner.activate(old.key(), &mut old),
                Err(PreparationError::StaleRequest)
            );
            assert_eq!(
                owner.activate(new.key(), &mut new),
                Err(PreparationError::StaleRequest)
            );
            assert_eq!(owner.cancel(old.key()), Err(PreparationError::StaleRequest));
            assert_eq!(owner.current().unwrap().snapshot(), published);
            assert_eq!(owner.resources(), occupied);
            drop(old);
            assert_eq!(owner.candidate(), None);
            assert_eq!(owner.current().unwrap().snapshot(), published);
            assert_eq!(
                owner.resources(),
                ResourceUsage {
                    precompiled_bytes: cost.source_bytes,
                    runtime_bytes: cost.pool_bytes
                        + cost.coordinator_bytes
                        + cost.task_bytes
                        + cost.run_bytes,
                }
            );
            drop(new);
            assert_eq!(
                owner.resources().runtime_bytes,
                cost.pool_bytes + cost.coordinator_bytes + cost.run_bytes
            );
            if reached_ready {
                break;
            }
        }
        assert!(
            reached_ready,
            "all return points through Ready must be covered"
        );
    }
}

#[test]
fn owner_drop_at_every_preparation_return_point_keeps_only_live_task_reservations() {
    let input = circles(&[0, 1_000_000, 2_000_000, 3_000_000]);
    let cost = SyntheticCoordinator::costs();
    for operation in operations() {
        let mut reached_ready = false;
        for cut in 0..128 {
            let mut owner = operation_owner(71_002, operation, &input);
            let mut task = request_operation(&mut owner, operation, &input);
            for _ in 0..cut {
                if task.stage() == PreparationStage::Ready {
                    break;
                }
                task.step(task.cursor(), WorkBudget::one()).unwrap();
            }
            reached_ready = task.stage() == PreparationStage::Ready;
            let cursor = task.cursor();
            drop(owner);
            assert_eq!(
                task.step(cursor, WorkBudget::one()),
                Err(PreparationError::StaleRequest)
            );
            assert_eq!(
                task.resources(),
                ResourceUsage {
                    precompiled_bytes: cost.source_bytes,
                    runtime_bytes: cost.pool_bytes + cost.task_bytes + cost.run_bytes,
                }
            );
            if reached_ready {
                break;
            }
        }
        assert!(reached_ready);
    }
}

#[test]
fn failed_work_and_its_drop_cannot_cancel_a_new_ready_replacement() {
    let input = circles(&[0, 1_000_000, 2_000_000, 3_000_000]);
    let invalid = [input[0], input[1], input[2], input[0]];
    let operation = PreparationOperation::Load {
        start: time(-200_001),
    };
    let mut owner = operation_owner(71_003, operation, &input);
    let before = owner.current().unwrap().snapshot();
    let usage = owner.resources();
    let mut failed = request_operation(&mut owner, operation, &invalid);
    let held = owner.resources();
    let mut error = None;
    for _ in 0..128 {
        match failed.step(failed.cursor(), WorkBudget::one()) {
            Ok(PreparationReport::Pending { .. }) => {}
            Ok(PreparationReport::Ready { .. }) => panic!("invalid last record accepted"),
            Err(result) => {
                error = Some(result);
                break;
            }
        }
    }
    assert_eq!(
        error,
        Some(PreparationError::Invalid(ValueError::DuplicateObject))
    );
    assert_eq!(failed.stage(), PreparationStage::Failed);
    assert_eq!(owner.candidate(), None);
    assert_eq!(owner.resources(), held);
    let mut new = request_operation(&mut owner, operation, &input);
    complete(&mut new, WorkBudget::one());
    assert_eq!(
        failed.step(failed.cursor(), WorkBudget::one()),
        Err(PreparationError::StaleRequest)
    );
    assert_eq!(
        owner.cancel(failed.key()),
        Err(PreparationError::StaleRequest)
    );
    drop(failed);
    assert_eq!(owner.candidate(), Some(new.key()));
    assert_eq!(owner.current().unwrap().snapshot(), before);
    let cost = SyntheticCoordinator::costs();
    assert_eq!(
        owner.resources(),
        ResourceUsage {
            precompiled_bytes: usage.precompiled_bytes + cost.source_bytes,
            runtime_bytes: usage.runtime_bytes + cost.task_bytes + cost.run_bytes,
        }
    );
    owner.activate(new.key(), &mut new).unwrap();
    drop(new);
    assert_eq!(owner.resources(), usage);
}

#[test]
fn cancellation_with_both_pools_and_old_event_queue_full_needs_no_spare_credit() {
    let mut owner = limited(71_004, 2, 2, 1);
    let input = circles(&[0]);
    create(&mut owner, 40, &input, -200_001, WorkBudget::one());
    owner
        .submit_inputs(current(&owner), &[press(1, 0)])
        .unwrap();
    freeze(&mut owner, 0, WorkBudget::one());
    let before = owner.current().unwrap().snapshot();
    let events = [
        owner.current().unwrap().event_at(0),
        owner.current().unwrap().event_at(1),
    ];
    assert_eq!(before.pending_events, 2);
    let usage = owner.resources();
    let operation = PreparationOperation::Load {
        start: time(-200_001),
    };
    let mut task = request_operation(&mut owner, operation, &input);
    complete(&mut task, WorkBudget::one());
    let full = owner.resources();
    let c = SyntheticCoordinator::costs();
    assert_eq!(
        full,
        ResourceUsage {
            precompiled_bytes: 2 * c.source_bytes,
            runtime_bytes: c.pool_bytes + c.coordinator_bytes + 2 * c.run_bytes + c.task_bytes,
        }
    );
    owner.cancel(task.key()).unwrap();
    assert_eq!(owner.resources(), full);
    assert!(matches!(
        owner.request_load(current(&owner), source(42), &input, time(-200_001)),
        Err(PreparationError::Budget {
            pool: ResourcePool::Precompiled,
            ..
        })
    ));
    assert_eq!(owner.candidate(), None);
    assert_eq!(owner.resources(), full);
    assert_eq!(owner.current().unwrap().snapshot(), before);
    assert_eq!(
        [
            owner.current().unwrap().event_at(0),
            owner.current().unwrap().event_at(1)
        ],
        events
    );
    drop(task);
    assert_eq!(owner.resources(), usage);
    let next = request_operation(&mut owner, operation, &input);
    drop(next);
    assert_eq!(owner.resources(), usage);
    assert_eq!(owner.candidate(), None);
}

#[test]
fn each_wrong_base_component_is_rejected_before_control_or_candidate_mutation() {
    let input = circles(&[0]);
    let operation = PreparationOperation::Inspect { at: time(0) };
    let mut owner = operation_owner(71_005, operation, &input);
    let base = current(&owner);
    let mut task = request_operation(&mut owner, operation, &input);
    complete(&mut task, WorkBudget::one());
    let before = owner.current().unwrap().snapshot();
    let usage = owner.resources();
    let stale = Err(PreparationError::Runtime(RunError::StaleSource));
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
            input: InputEpoch::new(2).unwrap(),
            ..base
        },
        RunSource {
            revision: MapRevision::new(99),
            ..base
        },
    ] {
        assert_eq!(owner.submit_inputs(wrong, &[press(1, -1)]), stale);
        assert_eq!(
            owner.observe_release(
                wrong,
                ReleaseObservation {
                    sequence: InputSequence::new(1).unwrap(),
                    slot: PhysicalSlot::PrimaryOne
                }
            ),
            Err(PreparationError::Runtime(RunError::StaleSource))
        );
        assert_eq!(
            owner.seal_through(wrong, time(0)),
            Err(PreparationError::Runtime(RunError::StaleSource))
        );
        assert_eq!(
            owner.advance(wrong, time(0), WorkBudget::one()),
            Err(PreparationError::Runtime(RunError::StaleSource))
        );
        assert_eq!(
            owner.request_freeze(wrong, time(0)),
            Err(PreparationError::Runtime(RunError::StaleSource))
        );
        assert_eq!(
            owner.settle_freeze(wrong, WorkBudget::one()),
            Err(PreparationError::Runtime(RunError::StaleSource))
        );
        assert_eq!(
            owner.resume(wrong),
            Err(PreparationError::Runtime(RunError::StaleSource))
        );
        assert!(matches!(
            owner.request_inspection(wrong, time(0)),
            Err(PreparationError::Runtime(RunError::StaleSource))
        ));
        assert!(matches!(
            owner.request_load(wrong, source(42), &input, time(-200_001)),
            Err(PreparationError::Runtime(RunError::StaleSource))
        ));
        assert_eq!(owner.candidate(), Some(task.key()));
        assert_eq!(owner.current().unwrap().snapshot(), before);
        assert_eq!(owner.resources(), usage);
    }
    owner.activate(task.key(), &mut task).unwrap();
}

#[test]
fn dropping_an_activated_task_releases_only_its_workspace_and_preserves_new_work() {
    let input = circles(&[0]);
    let mut owner = limited(71_006, 1, 2, 2);
    let mut first = owner
        .request_create(source(40), &input, time(-200_001))
        .unwrap();
    complete(&mut first, WorkBudget::one());
    owner.activate(first.key(), &mut first).unwrap();
    assert_eq!(first.stage(), PreparationStage::Activated);
    freeze(&mut owner, -200_001, WorkBudget::one());
    let mut next = owner.request_inspection(current(&owner), time(0)).unwrap();
    complete(&mut next, WorkBudget::one());
    let held = owner.resources();
    assert_eq!(
        first.step(first.cursor(), WorkBudget::one()),
        Err(PreparationError::StaleRequest)
    );
    drop(first);
    assert_eq!(owner.candidate(), Some(next.key()));
    assert_eq!(
        owner.resources(),
        ResourceUsage {
            runtime_bytes: held.runtime_bytes - SyntheticCoordinator::costs().task_bytes,
            ..held
        }
    );
    owner.activate(next.key(), &mut next).unwrap();
    assert_eq!(
        owner.current().unwrap().snapshot().completed_through,
        Some(time(0))
    );
}
