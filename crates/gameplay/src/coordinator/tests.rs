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
