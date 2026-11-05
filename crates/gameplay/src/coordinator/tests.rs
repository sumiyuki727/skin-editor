use super::*;
use crate::{InputEpoch, MapRevision, RunId, TransportEpoch, WorkBudget};

fn source(transport: u64, input: u64) -> RunSource {
    RunSource {
        run: RunId::new(80_001).unwrap(),
        transport: TransportEpoch::new(transport).unwrap(),
        input: InputEpoch::new(input).unwrap(),
        revision: MapRevision::new(1),
    }
}
fn empty() -> SyntheticCoordinator {
    SyntheticCoordinator::new(
        CoordinatorId::new(80_001).unwrap(),
        Limits::tiny(),
        PreparationLimits::tiny(),
    )
    .unwrap()
}

fn finish(job: &mut PreparationTask<'_>) {
    for _ in 0..128 {
        if matches!(
            job.step(job.cursor(), WorkBudget::one()).unwrap(),
            PreparationReport::Ready { .. }
        ) {
            return;
        }
    }
    panic!("four-slot task bound");
}

#[test]
fn managed_source_payload_is_shared_and_released_only_after_the_last_old_task() {
    let mut owner = empty();
    let base = source(1, 1);
    freeze_empty(&mut owner, base);
    let old_map = Rc::downgrade(&owner.active.as_ref().unwrap().content.map);
    let old_source = Rc::downgrade(&owner.active.as_ref().unwrap().content);
    assert!(
        owner
            .current()
            .unwrap()
            .shares_map(&owner.active.as_ref().unwrap().content.map)
    );
    let mut inspection = owner
        .request_inspection(base, MapTime::from_micros(-1))
        .unwrap();
    finish(&mut inspection);
    let candidate = inspection.run.as_ref().unwrap();
    assert!(Rc::ptr_eq(
        &candidate.content.map,
        &owner.active.as_ref().unwrap().content.map
    ));
    assert!(candidate.run.shares_map(&candidate.content.map));
    owner.cancel(inspection.key()).unwrap();
    let next = RunSource {
        run: RunId::new(80_002).unwrap(),
        ..base
    };
    let mut load = owner
        .request_load(base, next, &[], MapTime::from_micros(0))
        .unwrap();
    finish(&mut load);
    assert!(!Rc::ptr_eq(
        &load.run.as_ref().unwrap().content.map,
        &owner.active.as_ref().unwrap().content.map
    ));
    let costs = SyntheticCoordinator::costs();
    assert_eq!(owner.resources().precompiled_bytes, 2 * costs.source_bytes);
    owner.activate(load.key(), &mut load).unwrap();
    drop(load);
    assert!(old_map.upgrade().is_some());
    assert!(old_source.upgrade().is_some());
    drop(inspection);
    assert!(old_map.upgrade().is_none());
    assert!(old_source.upgrade().is_none());
    assert_eq!(owner.resources().precompiled_bytes, costs.source_bytes);
    let current_map = Rc::downgrade(&owner.active.as_ref().unwrap().content.map);
    drop(owner);
    assert!(current_map.upgrade().is_none());
}

#[test]
fn controlled_allocation_rejections_keep_the_frozen_base_and_reservations_atomic() {
    for kind in [
        AllocationKind::SourceMap,
        AllocationKind::SourceOwner,
        AllocationKind::Runtime,
    ] {
        let mut owner = empty();
        let base = source(1, 1);
        freeze_empty(&mut owner, base);
        let before = owner.current().unwrap().observation();
        let baseline = owner.resources();
        owner.pool.allocation_failure.set(Some(kind));
        let next = RunSource {
            run: RunId::new(80_003).unwrap(),
            ..base
        };
        let mut job = owner
            .request_load(base, next, &[], MapTime::from_micros(0))
            .unwrap();
        let reserved = owner.resources();
        let mut failed = false;
        for _ in 0..16 {
            match job.step(job.cursor(), WorkBudget::one()) {
                Err(PreparationError::AllocationUnavailable(point)) => {
                    assert_eq!(point, kind);
                    failed = true;
                    break;
                }
                Ok(PreparationReport::Pending { .. }) => {}
                result => panic!("unexpected allocation result: {result:?}"),
            }
        }
        assert!(failed);
        assert_eq!(job.stage(), PreparationStage::Failed);
        assert_eq!(owner.candidate(), None);
        assert_eq!(owner.resources(), reserved);
        assert_eq!(owner.current().unwrap().observation(), before);
        assert_eq!(
            owner.activate(job.key(), &mut job),
            Err(PreparationError::StaleRequest)
        );
        let payload = job
            .content
            .as_ref()
            .map(|content| Rc::downgrade(&content.map));
        drop(job);
        if let Some(payload) = payload {
            assert!(payload.upgrade().is_none());
        }
        assert_eq!(owner.resources(), baseline);
        let mut retry = owner
            .request_load(base, next, &[], MapTime::from_micros(0))
            .unwrap();
        finish(&mut retry);
        owner.activate(retry.key(), &mut retry).unwrap();
        assert_eq!(owner.current().unwrap().source(), next);
    }
}

fn four_frozen() -> SyntheticCoordinator {
    use crate::{InputEdge, InputEvent, InputSequence, ObjectId, PhysicalSlot, Point};
    let mut owner = empty();
    let base = source(1, 1);
    let input = core::array::from_fn::<_, 4, _>(|i| {
        Circle::new(
            ObjectId::new(i as u32),
            MapTime::from_micros(i as i64 * 1_000_000),
            Point::new(0, 0).unwrap(),
        )
        .unwrap()
    });
    let mut create = owner
        .request_create(base, &input, MapTime::from_micros(-200_001))
        .unwrap();
    finish(&mut create);
    owner.activate(create.key(), &mut create).unwrap();
    drop(create);
    owner
        .submit_inputs(
            base,
            &[InputEvent {
                sequence: InputSequence::new(1).unwrap(),
                at: MapTime::from_micros(0),
                slot: PhysicalSlot::PrimaryOne,
                edge: InputEdge::Press(Point::new(0, 0).unwrap()),
            }],
        )
        .unwrap();
    owner.request_freeze(base, MapTime::from_micros(0)).unwrap();
    owner.seal_through(base, MapTime::from_micros(0)).unwrap();
    for _ in 0..64 {
        if matches!(
            owner.settle_freeze(base, WorkBudget::one()).unwrap(),
            crate::FreezeReport::Frozen { .. }
        ) {
            assert_eq!(
                owner.current().unwrap().observation().committed_counts.hits,
                1
            );
            return owner;
        }
    }
    panic!("four-slot frozen fixture bound");
}

#[test]
fn actual_shared_payload_outlives_the_owner_at_every_inspection_return_point() {
    let mut reached_ready = false;
    for stop in 0..32 {
        let mut owner = four_frozen();
        let active = owner.active.as_ref().unwrap();
        let map = Rc::downgrade(&active.content.map);
        let content = Rc::downgrade(&active.content);
        let pool = Rc::downgrade(&owner.pool);
        let mut task = owner
            .request_inspection(source(1, 1), MapTime::from_micros(1_000_000))
            .unwrap();
        for _ in 0..stop {
            if matches!(
                task.step(task.cursor(), WorkBudget::one()).unwrap(),
                PreparationReport::Ready { .. }
            ) {
                reached_ready = true;
                break;
            }
        }
        if let Some(candidate) = &task.run {
            assert!(
                candidate
                    .run
                    .shares_map(&owner.active.as_ref().unwrap().content.map)
            );
            assert!(Rc::ptr_eq(
                &candidate.content,
                &owner.active.as_ref().unwrap().content
            ));
            if task.stage() == PreparationStage::Ready {
                // The source is shared, while the old real Hit and the new
                // inspection's skipped ledger belong to independent runs.
                assert_eq!(candidate.run.observation().committed_counts.hits, 0);
                assert_eq!(
                    candidate
                        .run
                        .observation()
                        .committed_counts
                        .inspection_skipped,
                    1
                );
                assert_eq!(
                    candidate.run.committed_judgement(candidate.run.source(), 0),
                    Ok(None)
                );
                assert_eq!(
                    owner.current().unwrap().observation().committed_counts.hits,
                    1
                );
            }
        }
        let costs = SyntheticCoordinator::costs();
        assert_eq!(owner.resources().precompiled_bytes, costs.source_bytes);
        assert_eq!(
            owner.resources().runtime_bytes,
            costs.pool_bytes + costs.coordinator_bytes + costs.task_bytes + 2 * costs.run_bytes
        );
        let cursor = task.cursor();
        drop(owner);
        assert!(map.upgrade().is_some());
        assert!(content.upgrade().is_some());
        let held = task.resources();
        assert_eq!(held.precompiled_bytes, costs.source_bytes);
        assert_eq!(
            held.runtime_bytes,
            costs.pool_bytes + costs.task_bytes + costs.run_bytes
        );
        assert_eq!(
            task.step(cursor, WorkBudget::one()),
            Err(PreparationError::StaleRequest)
        );
        assert_eq!(task.resources(), held);
        drop(task);
        assert!(map.upgrade().is_none());
        assert!(content.upgrade().is_none());
        assert!(pool.upgrade().is_none());
        if reached_ready {
            return;
        }
    }
    panic!("all four-object inspection return points must reach Ready");
}

#[test]
fn shared_inspection_allocation_failure_preserves_old_pages_and_real_hit_ledger() {
    let mut owner = four_frozen();
    let base = source(1, 1);
    let before = owner.current().unwrap().observation();
    let header = before.ready_frame.unwrap();
    let frame = owner.current().unwrap().ready_frame(base).unwrap().unwrap();
    let objects = core::array::from_fn::<_, 4, _>(|i| frame.object(i));
    let baseline = owner.resources();
    owner
        .pool
        .allocation_failure
        .set(Some(AllocationKind::Runtime));
    let mut task = owner
        .request_inspection(base, MapTime::from_micros(1_000_000))
        .unwrap();
    let held = owner.resources();
    assert_eq!(
        task.step(task.cursor(), WorkBudget::one()),
        Err(PreparationError::AllocationUnavailable(
            AllocationKind::Runtime
        ))
    );
    assert_eq!(task.stage(), PreparationStage::Failed);
    assert_eq!(owner.candidate(), None);
    assert_eq!(owner.resources(), held);
    assert_eq!(owner.current().unwrap().observation(), before);
    let mut output = [None; 4];
    let page = owner
        .current()
        .unwrap()
        .read_frame_page(header.key().cursor(0), 64, &mut output)
        .unwrap()
        .unwrap();
    assert_eq!((page.written, page.end), (4, true));
    assert_eq!(output, objects);
    assert!(Rc::ptr_eq(
        task.content.as_ref().unwrap(),
        &owner.active.as_ref().unwrap().content
    ));
    assert_eq!(held.precompiled_bytes, baseline.precompiled_bytes);
    let map = Rc::downgrade(&task.content.as_ref().unwrap().map);
    drop(task);
    assert!(map.upgrade().is_some());
    assert_eq!(owner.resources(), baseline);
    let mut retry = owner
        .request_inspection(base, MapTime::from_micros(1_000_000))
        .unwrap();
    finish(&mut retry);
    let new = owner.activate(retry.key(), &mut retry).unwrap();
    assert!(map.upgrade().is_some());
    assert_eq!(owner.current().unwrap().source(), new);
    assert_eq!(
        owner
            .current()
            .unwrap()
            .read_frame_page(header.key().cursor(0), 64, &mut output),
        Err(crate::FrameReadError::StaleSource)
    );
    assert_eq!(output, objects);
    drop(retry);
    drop(owner);
    assert!(map.upgrade().is_none());
}

fn freeze_empty(owner: &mut SyntheticCoordinator, source: RunSource) {
    let mut job = owner
        .request_create(source, &[], MapTime::from_micros(0))
        .unwrap();
    job.step(job.cursor(), WorkBudget::one()).unwrap();
    job.step(job.cursor(), WorkBudget::one()).unwrap();
    owner.activate(job.key(), &mut job).unwrap();
    owner
        .request_freeze(source, MapTime::from_micros(0))
        .unwrap();
    owner.seal_through(source, MapTime::from_micros(0)).unwrap();
    owner.settle_freeze(source, WorkBudget::one()).unwrap();
}

#[test]
fn last_request_id_and_exhausted_progress_never_lock_out_cancellation() {
    let mut owner = empty();
    owner.next_request = Some(u64::MAX);
    let job = owner
        .request_create(source(1, 1), &[], MapTime::from_micros(0))
        .unwrap();
    let before = owner.resources();
    owner.cancel(job.key()).unwrap();
    assert_eq!(owner.resources(), before);
    assert!(matches!(
        owner.request_create(source(1, 1), &[], MapTime::from_micros(0)),
        Err(PreparationError::CounterExhausted)
    ));
    drop(job);
    assert_eq!(owner.resources().precompiled_bytes, 0);
    let mut owner = empty();
    let mut job = owner
        .request_create(source(1, 1), &[], MapTime::from_micros(0))
        .unwrap();
    job.set_exhausted_progress();
    let held = owner.resources();
    assert_eq!(
        job.step(job.cursor(), WorkBudget::one()),
        Err(PreparationError::CounterExhausted)
    );
    assert_eq!(owner.candidate(), None);
    assert_eq!(owner.resources(), held);
    assert!(owner.current().is_none());
}

#[test]
fn epoch_exhaustion_and_wrong_operation_or_target_preserve_the_current_candidate() {
    for (transport, input) in [(u64::MAX, 1), (1, u64::MAX)] {
        let mut owner = empty();
        let source = source(transport, input);
        freeze_empty(&mut owner, source);
        let before = owner.resources();
        let state = owner.current().unwrap().observation();
        assert!(matches!(
            owner.request_inspection(source, MapTime::from_micros(1)),
            Err(PreparationError::CounterExhausted)
        ));
        assert_eq!(owner.resources(), before);
        assert_eq!(owner.current().unwrap().observation(), state);
    }
    let mut owner = empty();
    let source = source(1, 1);
    freeze_empty(&mut owner, source);
    let mut job = owner
        .request_inspection(source, MapTime::from_micros(1))
        .unwrap();
    let original = job.key();
    for key in [
        PreparationKey {
            operation: PreparationOperation::Load {
                start: MapTime::from_micros(1),
            },
            ..original
        },
        PreparationKey {
            operation: PreparationOperation::Inspect {
                at: MapTime::from_micros(2),
            },
            ..original
        },
        PreparationKey {
            target: RunSource {
                revision: MapRevision::new(2),
                ..original.target
            },
            ..original
        },
        PreparationKey {
            driver: InputDriver::Regression,
            ..original
        },
    ] {
        assert_eq!(
            owner.activate(key, &mut job),
            Err(PreparationError::StaleRequest)
        );
        assert_eq!(owner.cancel(key), Err(PreparationError::StaleRequest));
        assert_eq!(owner.candidate(), Some(original));
    }
}

#[test]
fn budget_overflow_does_not_wrap_or_partially_reserve_the_other_pool() {
    for kind in [ResourcePool::Precompiled, ResourcePool::Runtime] {
        let mut owner = empty();
        owner.next_request = Some(u64::MAX);
        let initial = owner.resources();
        let mut injected = initial;
        match kind {
            ResourcePool::Precompiled => injected.precompiled_bytes = usize::MAX,
            ResourcePool::Runtime => injected.runtime_bytes = usize::MAX,
        }
        owner.pool.usage.set(injected);
        assert!(matches!(
            owner.request_create(source(1, 1), &[], MapTime::from_micros(0)),
            Err(PreparationError::Budget { pool, .. }) if pool == kind
        ));
        assert_eq!(owner.resources(), injected);
        assert_eq!(owner.candidate(), None);
        // Restore injected accounting before any real lease is dropped.
        owner.pool.usage.set(initial);
        let job = owner
            .request_create(source(1, 1), &[], MapTime::from_micros(0))
            .unwrap();
        assert_eq!(job.key().generation, u64::MAX);
        owner.cancel(job.key()).unwrap();
        drop(job);
        assert_eq!(owner.resources(), initial);
    }
}
