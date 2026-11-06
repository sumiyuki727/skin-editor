mod scale_support;
use scale_support::*;
use skin_editor_gameplay::*;

#[derive(Clone, Copy, Debug)]
enum Cut {
    Phase(PreparationPhase),
    Record(u64),
    State(u64),
    RebuildStart,
    Inspect(u64),
    Ready,
}
fn at_cut(task: &PreparationTask<'_>, cut: Cut) -> bool {
    match cut {
        Cut::Phase(phase) => task.preparation_phase() == Some(phase),
        Cut::Record(n) => {
            task.preparation_phase() == Some(PreparationPhase::Records)
                && task.work_counts().record_reads >= n
        }
        Cut::State(n) => {
            task.stage() == PreparationStage::Initializing
                && task.work_counts().state_initializations >= n
        }
        Cut::RebuildStart => task.stage() == PreparationStage::Rebuilding,
        Cut::Inspect(n) => {
            task.stage() == PreparationStage::Rebuilding
                && task.work_counts().allocation_attempts >= 3
                && task.work_counts().inspection_objects >= n
        }
        Cut::Ready => task.stage() == PreparationStage::Ready,
    }
}
fn request<'a>(
    owner: &mut SyntheticCoordinator,
    data: &'a [Circle],
    operation: u8,
    id: u128,
) -> PreparationTask<'a> {
    match operation {
        0 => owner.request_create(source(id), data, t(START)).unwrap(),
        1 => owner
            .request_load(
                owner.current().unwrap().source(),
                source(id),
                data,
                t(START),
            )
            .unwrap(),
        _ => owner
            .request_inspection(owner.current().unwrap().source(), t(5_000_000_000))
            .unwrap(),
    }
}
fn base(data: &[Circle], operation: u8, budget: WorkBudget) -> SyntheticCoordinator {
    if operation == 0 {
        return SyntheticCoordinator::new(
            CoordinatorId::new(305_000).unwrap(),
            Limits::reference(),
            PreparationLimits::reference(),
        )
        .unwrap();
    }
    let mut play = Playback::new(data, 305_000, budget);
    play.automatic();
    play.freeze(START);
    play.owner
}

#[test]
fn cancelled_scale_candidates_keep_real_owners_but_cannot_affect_the_next_request() {
    let data = circles();
    let cost = SyntheticCoordinator::costs_for_objects(N).unwrap();
    for budget in [WorkBudget::one(), WorkBudget::full()] {
        for operation in 0..3 {
            let cuts: &[Cut] = match operation {
                0 => &[
                    Cut::Phase(PreparationPhase::Allocating),
                    Cut::Phase(PreparationPhase::Records),
                    Cut::Phase(PreparationPhase::IndexInitialization),
                    Cut::Phase(PreparationPhase::Sorting),
                    Cut::Phase(PreparationPhase::DuplicateCheck),
                    Cut::State(0),
                    Cut::State(32),
                    Cut::Ready,
                ],
                1 => &[
                    Cut::Record(0),
                    Cut::Record(1),
                    Cut::Record(31),
                    Cut::Record(32),
                    Cut::Record(4999),
                    Cut::Record(9999),
                    Cut::State(4999),
                    Cut::Ready,
                ],
                _ => &[
                    Cut::State(0),
                    Cut::State(1),
                    Cut::State(31),
                    Cut::State(32),
                    Cut::State(4999),
                    Cut::State(9999),
                    Cut::RebuildStart,
                    Cut::Inspect(0),
                    Cut::Inspect(1),
                    Cut::Inspect(31),
                    Cut::Inspect(32),
                    Cut::Inspect(4999),
                    Cut::Inspect(9999),
                    Cut::Ready,
                ],
            };
            for cut in cuts.iter().copied() {
                let mut owner = base(&data, operation, budget);
                let before = owner.current().map(SyntheticRun::observation);
                let mut old = request(&mut owner, &data, operation, 305_001);
                let mut reached = false;
                // One-unit checkpoint discovery is identical in both schedules.
                for _ in 0..BOUND {
                    if at_cut(&old, cut) {
                        reached = true;
                        break;
                    }
                    old.step(old.cursor(), WorkBudget::one()).unwrap();
                }
                assert!(reached, "{operation}/{cut:?}");
                if matches!(cut, Cut::Inspect(0)) {
                    assert_eq!(
                        old.work_counts().allocation_attempts,
                        3,
                        "inspection zero checkpoint must own its private buffer"
                    );
                    assert_eq!(old.work_counts().state_initializations, N as u64);
                    assert_eq!(
                        owner.allocations().runtime_bytes,
                        cost.pool_bytes
                            + 2 * core::mem::size_of::<SyntheticRun>()
                            + 3 * cost.state_bytes
                    );
                }
                assert_eq!(owner.current().map(SyntheticRun::observation), before);
                if old.stage() != PreparationStage::Ready {
                    assert_eq!(
                        owner.activate(old.key(), &mut old),
                        Err(PreparationError::NotReady)
                    );
                }
                let held = owner.resources();
                let payload = owner.allocations();
                let counts = old.work_counts();
                let cursor = old.cursor();
                let key = old.key();
                owner.cancel(key).unwrap();
                assert_eq!(owner.resources(), held);
                assert_eq!(owner.allocations(), payload);
                let mut next = request(&mut owner, &data, operation, 305_002);
                let next_key = next.key();
                let r = owner.resources();
                let a = owner.allocations();
                assert_eq!(
                    old.step(cursor, budget),
                    Err(PreparationError::StaleRequest)
                );
                assert_eq!(
                    owner.activate(key, &mut old),
                    Err(PreparationError::StaleRequest)
                );
                assert_eq!(
                    owner.activate(next_key, &mut old),
                    Err(PreparationError::StaleRequest)
                );
                assert_eq!(old.work_counts(), counts);
                assert_eq!(owner.resources(), r);
                assert_eq!(owner.allocations(), a);
                assert_eq!(owner.candidate(), Some(next_key));
                drop(old);
                assert_eq!(owner.candidate(), Some(next_key));
                assert_eq!(
                    owner.resources().runtime_bytes,
                    r.runtime_bytes - cost.task_bytes - cost.run_bytes
                );
                assert_eq!(
                    owner.resources().precompiled_bytes,
                    r.precompiled_bytes - if operation == 2 { 0 } else { cost.source_bytes }
                );
                finish(&mut next, budget);
                assert_eq!(owner.current().map(SyntheticRun::observation), before);
                let receipt = next.work_counts();
                let current = owner.activate(next_key, &mut next).unwrap();
                assert_eq!(next.work_counts(), receipt);
                assert_eq!(
                    owner.current().unwrap().driver(),
                    if operation == 2 {
                        InputDriver::Automatic
                    } else {
                        InputDriver::Manual
                    }
                );
                assert_eq!(owner.cancel(key), Err(PreparationError::StaleRequest));
                assert_eq!(
                    owner.activate(next_key, &mut next),
                    Err(PreparationError::StaleRequest)
                );
                assert_eq!(owner.current().unwrap().source(), current);
                let kept = owner.resources();
                assert_eq!(kept.precompiled_bytes, cost.source_bytes);
                assert_eq!(
                    kept.runtime_bytes,
                    cost.pool_bytes + cost.coordinator_bytes + cost.run_bytes + cost.task_bytes
                );
                drop(next);
                assert_eq!(
                    owner.resources(),
                    ResourceUsage {
                        precompiled_bytes: cost.source_bytes,
                        runtime_bytes: cost.pool_bytes + cost.coordinator_bytes + cost.run_bytes
                    }
                );
                assert_eq!(
                    owner.allocations(),
                    AllocationUsage {
                        precompiled_bytes: cost.source_bytes,
                        runtime_bytes: cost.pool_bytes
                            + core::mem::size_of::<SyntheticRun>()
                            + cost.state_bytes,
                        precompiled_allocations: 3,
                        runtime_allocations: 3
                    }
                );
                if let Some(old_frame) = before.and_then(|v| v.ready_frame) {
                    let mut page = [None; 1];
                    assert_eq!(
                        owner.current().unwrap().read_frame_page(
                            old_frame.key().cursor(0),
                            1,
                            &mut page
                        ),
                        Err(FrameReadError::StaleSource)
                    );
                    assert_eq!(page, [None; 1]);
                }
            }
        }
    }
}

#[test]
fn failed_large_create_and_load_keep_the_old_run_and_payload_until_real_cleanup() {
    let data = circles();
    let mut invalid = data.clone();
    invalid[N - 1] = Circle::new(data[0].id(), data[N - 1].target(), data[N - 1].center()).unwrap();
    for budget in [WorkBudget::one(), WorkBudget::full()] {
        for operation in 0..2 {
            let mut owner = base(&data, operation, budget);
            let before = owner.current().map(SyntheticRun::observation);
            let baseline = owner.resources();
            let actual = owner.allocations();
            let mut failed = request(&mut owner, &invalid, operation, 306_001);
            let mut rejected = false;
            for _ in 0..BOUND {
                match failed.step(failed.cursor(), budget) {
                    Err(error) => {
                        assert_eq!(
                            error,
                            PreparationError::Invalid(ValueError::DuplicateObject)
                        );
                        rejected = true;
                        break;
                    }
                    Ok(PreparationReport::Pending { steps, .. }) => {
                        assert!(steps > 0 && steps <= budget.get())
                    }
                    result => panic!("invalid large source published {result:?}"),
                }
            }
            assert!(rejected);
            assert_eq!(failed.stage(), PreparationStage::Failed);
            assert_eq!(failed.failed_record_index(), Some(N - 1));
            assert_eq!(owner.candidate(), None);
            assert_eq!(owner.current().map(SyntheticRun::observation), before);
            assert!(owner.resources().precompiled_bytes > baseline.precompiled_bytes);
            assert!(owner.allocations().precompiled_bytes > actual.precompiled_bytes);
            let mut next = request(&mut owner, &data, operation, 306_002);
            let key = next.key();
            assert_eq!(
                owner.activate(key, &mut failed),
                Err(PreparationError::StaleRequest)
            );
            drop(failed);
            assert_eq!(owner.candidate(), Some(key));
            finish(&mut next, budget);
            owner.activate(key, &mut next).unwrap();
            drop(next);
            assert_eq!(owner.current().unwrap().source(), source(306_002));
            assert_eq!(
                owner.resources().precompiled_bytes,
                SyntheticCoordinator::costs_for_objects(N)
                    .unwrap()
                    .source_bytes
            );
        }
    }
}

#[test]
fn cancelled_large_inspection_survives_coordinator_drop_with_only_its_real_owners_charged() {
    let data = circles();
    let cost = SyntheticCoordinator::costs_for_objects(N).unwrap();
    for budget in [WorkBudget::one(), WorkBudget::full()] {
        for cut in [
            Cut::State(0),
            Cut::State(4999),
            Cut::Inspect(0),
            Cut::Inspect(4999),
            Cut::Ready,
        ] {
            let mut owner = base(&data, 2, budget);
            let mut task = request(&mut owner, &data, 2, 307_001);
            let mut reached = false;
            for _ in 0..BOUND {
                if at_cut(&task, cut) {
                    reached = true;
                    break;
                }
                task.step(task.cursor(), WorkBudget::one()).unwrap();
            }
            assert!(reached, "close checkpoint {cut:?}");
            let key = task.key();
            let cursor = task.cursor();
            let counts = task.work_counts();
            let resources = owner.resources();
            let payload = owner.allocations();
            owner.cancel(key).unwrap();
            assert_eq!(owner.resources(), resources);
            assert_eq!(owner.allocations(), payload);
            drop(owner);
            assert_eq!(
                task.resources(),
                ResourceUsage {
                    precompiled_bytes: cost.source_bytes,
                    runtime_bytes: cost.pool_bytes + cost.task_bytes + cost.run_bytes
                }
            );
            let retained = task.allocations();
            assert_eq!(retained.precompiled_bytes, cost.source_bytes);
            assert_eq!(retained.precompiled_allocations, 3);
            assert_eq!(
                retained.runtime_bytes,
                payload.runtime_bytes - core::mem::size_of::<SyntheticRun>() - cost.state_bytes
            );
            assert_eq!(
                retained.runtime_allocations,
                payload.runtime_allocations - 2
            );
            assert_eq!(
                task.step(cursor, budget),
                Err(PreparationError::StaleRequest)
            );
            assert_eq!(task.work_counts(), counts);
            assert_eq!(task.allocations(), retained);
            assert_eq!(
                task.resources().runtime_bytes,
                resources.runtime_bytes - cost.coordinator_bytes - cost.run_bytes
            );
            // No observer holds a source lease. The existing Rc::Weak unit
            // regression observes actual last-owner destruction after this Drop.
            drop(task);
        }
    }
}
