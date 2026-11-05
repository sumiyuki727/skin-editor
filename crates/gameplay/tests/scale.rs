use skin_editor_gameplay::*;

fn t(value: i64) -> MapTime {
    MapTime::from_micros(value)
}
fn source(id: u128) -> RunSource {
    RunSource {
        run: RunId::new(id).unwrap(),
        transport: TransportEpoch::new(1).unwrap(),
        input: InputEpoch::new(1).unwrap(),
        revision: MapRevision::new(700),
    }
}
fn input(n: usize) -> Vec<Circle> {
    (0..n)
        .map(|i| {
            Circle::new(
                ObjectId::new(i as u32),
                t(i as i64 * 1_000_000),
                Point::new(0, 0).unwrap(),
            )
            .unwrap()
        })
        .collect()
}
fn large_limits() -> Limits {
    Limits::reference()
        .with_object_capacity(MAX_REFERENCE_CIRCLES)
        .unwrap()
}
fn levels(n: usize) -> u64 {
    if n <= 1 {
        0
    } else {
        usize::BITS as u64 - (n - 1).leading_zeros() as u64
    }
}
fn bound(n: usize) -> usize {
    n * (2 * levels(n) as usize + 16) + 64
}
fn finish(task: &mut PreparationTask<'_>, n: usize, budget: WorkBudget) {
    for _ in 0..bound(n) {
        match task.step(task.cursor(), budget).unwrap() {
            PreparationReport::Ready { .. } => return,
            PreparationReport::Pending { steps, .. } => assert!(steps > 0 && steps <= budget.get()),
        }
    }
    panic!("finite preparation bound");
}

#[test]
fn capacity_proof_and_each_initialization_unit_cover_empty_through_ten_thousand() {
    for n in [0, 1, 4, 5, 31, 32, 9999, 10000] {
        let mut data = input(n);
        for (i, circle) in data.iter_mut().enumerate() {
            *circle = Circle::new(
                ObjectId::new((n - 1 - i) as u32 * 17),
                circle.target(),
                circle.center(),
            )
            .unwrap();
        }
        let mut preparation =
            MapPreparation::with_limits(source(1).revision, &data, large_limits()).unwrap();
        assert_eq!(preparation.capacities(), [0; 3]);
        let mut done = false;
        for _ in 0..bound(n) {
            let before = preparation.work_counts();
            let report = preparation.step(WorkBudget::one()).unwrap();
            let after = preparation.work_counts();
            assert!(after.valid);
            assert!(after.record_reads - before.record_reads <= 1);
            assert!(after.record_copies - before.record_copies <= 1);
            assert!(after.index_initializations - before.index_initializations <= 1);
            assert!(after.id_comparisons - before.id_comparisons <= 1);
            assert!(after.index_moves - before.index_moves <= 1);
            assert!(after.allocation_attempts - before.allocation_attempts <= 1);
            assert_ne!(after, before, "no zero-work Pending phase");
            for capacity in preparation.capacities() {
                assert!(capacity == 0 || capacity == n.max(4));
            }
            if matches!(report, PreparationProgress::Complete { .. }) {
                done = true;
                break;
            }
        }
        assert!(done);
        let counts = preparation.work_counts();
        assert_eq!(
            (
                counts.record_reads,
                counts.record_copies,
                counts.index_initializations
            ),
            (n as u64, n as u64, 2 * n as u64)
        );
        assert!(counts.id_comparisons <= n as u64 * (levels(n) + 1));
        assert!(counts.index_moves <= 2 * n as u64 * (levels(n) + 1));
        assert_eq!(counts.allocation_attempts, 3);
        let cost = SyntheticCoordinator::costs_for_objects(n).unwrap();
        assert_eq!(
            counts.allocation_bytes as usize,
            cost.source_map_bytes - core::mem::size_of::<SyntheticMap>() + cost.index_bytes
        );
        assert_eq!(preparation.capacities(), [n.max(4); 3]);
        let map = preparation.finish().unwrap();
        assert_eq!((map.len(), map.capacity()), (n, n.max(4)));
        for (i, circle) in data.iter().enumerate() {
            assert_eq!(map.circle(i), Some(*circle));
        }
        let mut initialization =
            RunPreparation::new(source(1), map, t(-200_001), large_limits()).unwrap();
        let mut ready = false;
        for _ in 0..n + 2 {
            let before = initialization.work_counts();
            let result = initialization.step(WorkBudget::one()).unwrap();
            let after = initialization.work_counts();
            assert!(after.valid);
            assert!(after.state_initializations - before.state_initializations <= 1);
            assert_eq!(after.record_reads, 0);
            if result == RunPreparationProgress::Complete {
                ready = true;
                break;
            }
        }
        assert!(ready);
        assert_eq!(initialization.work_counts().state_initializations, n as u64);
        let run = initialization.finish().unwrap();
        assert_eq!(run.state_capacities(), [n.max(4), 0]);
        assert_eq!(run.source_capacity(), n.max(4));
        assert!(run.ready_frame(run.source()).unwrap().is_none());
        assert_eq!(run.observation().object_count, n);
        assert_eq!(run.work_counts().frame_objects, 0);
    }
}

fn oracle(data: &[Circle]) -> Option<(usize, ValueError)> {
    for i in 0..data.len() {
        if data[..i].iter().any(|old| old.id() == data[i].id()) {
            return Some((i, ValueError::DuplicateObject));
        }
        if i > 0 && data[i].target() < data[i - 1].target() {
            return Some((i, ValueError::UnsortedMap));
        }
        if i > 0 && data[i].accepts_from() <= data[i - 1].accepts_through() {
            return Some((i, ValueError::UnsupportedOverlap));
        }
    }
    None
}

#[test]
fn merge_validation_preserves_original_record_error_order_and_arbitrary_ids() {
    for ids in [
        [0, 1, 0],
        [0, 1, 1],
        [2, 0, 2],
        [u32::MAX, 0, 1],
        [1, 1, 1],
        [0, 1, 2],
    ] {
        for targets in [
            [0, 1_000_000, 500_000],
            [1_000_000, 0, 2_000_000],
            [0, 0, 1_000_000],
            [0, 1_000_000, 2_000_000],
        ] {
            let data: Vec<_> = (0..3)
                .map(|i| {
                    Circle::new(
                        ObjectId::new(ids[i]),
                        t(targets[i]),
                        Point::new(0, 0).unwrap(),
                    )
                    .unwrap()
                })
                .collect();
            for budget in [WorkBudget::one(), WorkBudget::full()] {
                let expected = oracle(&data);
                let mut preparation =
                    MapPreparation::with_limits(source(2).revision, &data, large_limits()).unwrap();
                let mut finished = false;
                for _ in 0..bound(3) {
                    match preparation.step(budget) {
                        Err(error) => {
                            assert_eq!(
                                Some((preparation.failed_record_index().unwrap(), error)),
                                expected
                            );
                            finished = true;
                            break;
                        }
                        Ok(PreparationProgress::Complete { .. }) => {
                            assert_eq!(expected, None);
                            finished = true;
                            break;
                        }
                        Ok(PreparationProgress::Pending { .. }) => {}
                    }
                }
                assert!(finished);
                match expected {
                    Some((_, error)) => assert_eq!(preparation.finish(), Err(error)),
                    None => {
                        let map = preparation.finish().unwrap();
                        for (i, circle) in data.iter().enumerate() {
                            assert_eq!(map.circle(i), Some(*circle));
                        }
                    }
                }
            }
        }
    }
    let data: Vec<_> = [9, 0, 9, 0]
        .into_iter()
        .enumerate()
        .map(|(i, id)| {
            Circle::new(
                ObjectId::new(id),
                t(i as i64 * 1_000_000),
                Point::new(0, 0).unwrap(),
            )
            .unwrap()
        })
        .collect();
    let mut prep = MapPreparation::with_limits(source(2).revision, &data, large_limits()).unwrap();
    assert_eq!(
        prep.step(WorkBudget::full()),
        Err(ValueError::DuplicateObject)
    );
    assert_eq!(prep.failed_record_index(), Some(2));
}

#[test]
fn large_requests_reserve_both_pools_and_single_run_limits_before_allocation() {
    let data = input(10000);
    let c = SyntheticCoordinator::costs_for_objects(data.len()).unwrap();
    let required = c.pool_bytes + c.coordinator_bytes + c.task_bytes + c.run_bytes;
    for (source_limit, runtime_limit, pool) in [
        (c.source_bytes - 1, required, ResourcePool::Precompiled),
        (c.source_bytes, required - 1, ResourcePool::Runtime),
    ] {
        let mut owner = SyntheticCoordinator::new(
            CoordinatorId::new(3).unwrap(),
            large_limits(),
            PreparationLimits::try_new(source_limit, runtime_limit, 256).unwrap(),
        )
        .unwrap();
        let reserved = owner.resources();
        let allocated = owner.allocations();
        assert!(
            matches!(owner.request_create(source(3), &data, t(-200_001)), Err(PreparationError::Budget { pool: p, .. }) if p == pool)
        );
        assert_eq!(owner.resources(), reserved);
        assert_eq!(owner.allocations(), allocated);
        assert_eq!(owner.candidate(), None);
    }
    let limited = Limits::try_new(64, 64, 64, 256, c.run_bytes - 1)
        .unwrap()
        .with_object_capacity(10000)
        .unwrap();
    let mut owner = SyntheticCoordinator::new(
        CoordinatorId::new(4).unwrap(),
        limited,
        PreparationLimits::reference(),
    )
    .unwrap();
    let baseline = owner.resources();
    let actual = owner.allocations();
    assert!(matches!(
        owner.request_create(source(4), &data, t(-200_001)),
        Err(PreparationError::Invalid(ValueError::StorageLimit))
    ));
    assert_eq!(owner.resources(), baseline);
    assert_eq!(owner.allocations(), actual);
    let mut owner = SyntheticCoordinator::new(
        CoordinatorId::new(5).unwrap(),
        large_limits(),
        PreparationLimits::try_new(c.source_bytes, required, 256).unwrap(),
    )
    .unwrap();
    let mut task = owner.request_create(source(5), &data, t(-200_001)).unwrap();
    assert_eq!(owner.allocations().precompiled_bytes, 0);
    finish(&mut task, 10000, WorkBudget::full());
    assert!(owner.current().is_none());
    assert_eq!(task.allocations().precompiled_bytes, c.source_bytes);
    assert_eq!(task.allocations().precompiled_allocations, 3);
    assert_eq!(
        task.allocations().runtime_bytes,
        c.pool_bytes + core::mem::size_of::<SyntheticRun>() + c.state_bytes
    );
    owner.activate(task.key(), &mut task).unwrap();
    drop(task);
    assert_eq!(
        owner.resources().runtime_bytes,
        c.pool_bytes + c.coordinator_bytes + c.run_bytes
    );
    assert_eq!(owner.allocations().precompiled_bytes, c.source_bytes);
    assert_eq!(owner.allocations().runtime_allocations, 3);
    assert!(
        owner
            .current()
            .unwrap()
            .ready_frame(source(5))
            .unwrap()
            .is_none()
    );
}

#[test]
fn raw_overlimits_and_tiny_sources_cannot_enter_the_large_path() {
    assert_eq!(Limits::tiny().object_capacity(), 4);
    assert_eq!(
        Limits::tiny().with_object_capacity(5),
        Err(ValueError::InvalidLimits)
    );
    let data = input(10001);
    assert!(matches!(
        MapPreparation::new(source(6).revision, &data[..5]),
        Err(ValueError::ObjectLimit)
    ));
    assert!(matches!(
        MapPreparation::with_limits(source(6).revision, &data[..5], Limits::tiny()),
        Err(ValueError::ObjectLimit)
    ));
    assert!(matches!(
        MapPreparation::with_limits(source(6).revision, &data, large_limits()),
        Err(ValueError::ObjectLimit)
    ));
    for limits in [Limits::tiny(), large_limits()] {
        let mut owner = SyntheticCoordinator::new(
            CoordinatorId::new(6).unwrap(),
            limits,
            PreparationLimits::reference(),
        )
        .unwrap();
        let before = owner.resources();
        let allocated = owner.allocations();
        assert!(matches!(
            owner.request_create(source(6), &data, t(-200_001)),
            Err(PreparationError::Invalid(ValueError::ObjectLimit))
        ));
        assert_eq!(owner.resources(), before);
        assert_eq!(owner.allocations(), allocated);
        assert_eq!(owner.candidate(), None);
    }
    assert_eq!(
        SyntheticCoordinator::costs_for_objects(usize::MAX),
        Err(ValueError::ObjectLimit)
    );
    assert_eq!(
        SyntheticRun::storage_bytes_for_objects(usize::MAX),
        Err(ValueError::ObjectLimit)
    );
    let mut prep =
        MapPreparation::with_limits(source(6).revision, &data[..5], large_limits()).unwrap();
    assert!(matches!(
        prep.step(WorkBudget::full()),
        Ok(PreparationProgress::Complete { .. })
    ));
    assert!(matches!(
        RunPreparation::new(
            source(6),
            prep.finish().unwrap(),
            t(-200_001),
            Limits::tiny()
        ),
        Err(ValueError::ObjectLimit)
    ));
}

#[test]
fn convenience_constructor_rejects_large_sources_before_record_start_validation() {
    let data = input(5);
    let mut prep =
        MapPreparation::with_limits(source(40).revision, &data, Limits::reference()).unwrap();
    assert!(matches!(
        prep.step(WorkBudget::full()),
        Ok(PreparationProgress::Complete { .. })
    ));
    assert!(matches!(
        SyntheticRun::new(
            source(40),
            prep.finish().unwrap(),
            t(0),
            Limits::reference()
        ),
        Err(ValueError::ObjectLimit)
    ));
}

#[test]
fn cancelled_large_sort_keeps_actual_buffers_until_drop_and_cannot_touch_replacement() {
    let data = input(10000);
    let c = SyntheticCoordinator::costs_for_objects(data.len()).unwrap();
    let mut owner = SyntheticCoordinator::new(
        CoordinatorId::new(7).unwrap(),
        large_limits(),
        PreparationLimits::reference(),
    )
    .unwrap();
    let mut a = owner.request_create(source(7), &data, t(-200_001)).unwrap();
    let mut sorting = false;
    for _ in 0..bound(10000) {
        a.step(a.cursor(), WorkBudget::one()).unwrap();
        if a.preparation_phase() == Some(PreparationPhase::Sorting) {
            sorting = true;
            break;
        }
    }
    assert!(sorting);
    let actual = owner.allocations();
    let held = owner.resources();
    assert_eq!(
        actual.precompiled_bytes,
        c.source_map_bytes - core::mem::size_of::<SyntheticMap>()
    );
    assert_eq!(actual.runtime_bytes, c.pool_bytes + c.index_bytes);
    owner.cancel(a.key()).unwrap();
    assert_eq!(owner.resources(), held);
    assert_eq!(owner.allocations(), actual);
    let mut b = owner.request_create(source(8), &data, t(-200_001)).unwrap();
    let reserved = owner.resources();
    let actual = owner.allocations();
    assert_eq!(
        a.step(a.cursor(), WorkBudget::one()),
        Err(PreparationError::StaleRequest)
    );
    assert_eq!(owner.allocations(), actual);
    assert_eq!(owner.resources(), reserved);
    finish(&mut b, 10000, WorkBudget::full());
    owner.activate(b.key(), &mut b).unwrap();
    drop(a);
    assert_eq!(owner.allocations().precompiled_bytes, c.source_bytes);
    assert_eq!(
        owner.allocations().runtime_bytes,
        c.pool_bytes + core::mem::size_of::<SyntheticRun>() + c.state_bytes
    );
    assert_eq!(owner.current().unwrap().source(), source(8));
    drop(b);
    assert_eq!(owner.resources().precompiled_bytes, c.source_bytes);
    assert_eq!(
        owner.resources().runtime_bytes,
        c.pool_bytes + c.coordinator_bytes + c.run_bytes
    );
}

fn drain_misses(owner: &mut SyntheticCoordinator, expected: &mut usize) {
    for _ in 0..64 {
        let Some(event) = owner.current().unwrap().event_at(0) else {
            return;
        };
        let run = owner.current().unwrap();
        assert_eq!(event.key.run(), run.source().run);
        assert_eq!(event.key.transport(), run.source().transport);
        assert_eq!(event.key.sequence().get(), *expected as u64 + 1);
        assert_eq!(event.object, ObjectId::new(*expected as u32));
        assert_eq!(event.at, t(*expected as i64 * 1_000_000 + 100_001));
        assert_eq!(event.kind, EventKind::Judgement(Judgement::Miss));
        assert_eq!(event.evidence, Evidence::SyntheticTest);
        owner.acknowledge(event.key).unwrap();
        *expected += 1;
    }
    assert!(owner.current().unwrap().event_at(0).is_none());
}

#[test]
fn reference_g0_ten_thousand_misses_is_identical_at_work_budgets_one_and_256() {
    const N: usize = 10000;
    let data = input(N);
    for (case, budget) in [WorkBudget::one(), WorkBudget::full()]
        .into_iter()
        .enumerate()
    {
        let s = source(20 + case as u128);
        assert_eq!(Limits::reference().object_capacity(), N);
        let mut owner = SyntheticCoordinator::new(
            CoordinatorId::new(20 + case as u128).unwrap(),
            Limits::reference(),
            PreparationLimits::reference(),
        )
        .unwrap();
        let mut task = owner.request_create(s, &data, t(-200_001)).unwrap();
        finish(&mut task, N, budget);
        let prepared = task.work_counts();
        assert!(prepared.valid);
        assert_eq!(
            (
                prepared.record_reads,
                prepared.record_copies,
                prepared.state_initializations
            ),
            (N as u64, N as u64, N as u64)
        );
        assert!(prepared.id_comparisons <= N as u64 * (levels(N) + 1));
        assert!(prepared.index_moves <= 2 * N as u64 * (levels(N) + 1));
        assert_eq!(prepared.allocation_attempts, 7);
        owner.activate(task.key(), &mut task).unwrap();
        drop(task);
        assert_eq!(owner.current().unwrap().driver(), InputDriver::Manual);
        let end = t(9_999_300_000);
        owner.seal_through(s, end).unwrap();
        let mut report = owner.advance(s, end, budget).unwrap();
        let mut events = 0;
        let mut complete = false;
        for _ in 0..20 * N + 64 {
            assert!(owner.current().unwrap().observation().pending_events <= 64);
            match report {
                AdvanceReport::Complete { through, .. } => {
                    assert_eq!(through, end);
                    complete = true;
                    break;
                }
                AdvanceReport::Pending { cursor, reason, .. } => {
                    if reason == PendingReason::EventBackpressure {
                        drain_misses(&mut owner, &mut events);
                    }
                    report = owner.continue_advance(cursor, budget).unwrap();
                }
            }
        }
        assert!(complete);
        drain_misses(&mut owner, &mut events);
        assert_eq!(events, N);
        let run = owner.current().unwrap();
        let frame = run.ready_frame(s).unwrap().unwrap();
        let h = frame.header();
        assert_eq!(
            (h.at, h.len(), h.profile_version, h.evidence),
            (end, N, 1, Evidence::SyntheticTest)
        );
        assert_eq!(
            h.counts,
            FrameCounts {
                hits: 0,
                misses: N,
                inspection_skipped: 0
            }
        );
        assert_eq!(
            h.statistics_origin,
            StatisticsOrigin::FromStart { start: t(-200_001) }
        );
        let mut output = [None; 64];
        let mut cursor = h.key().cursor(0);
        let mut checked = 0;
        for _ in 0..N.div_ceil(64) {
            let page = frame.read_page(cursor, 64, &mut output).unwrap();
            for value in output.iter().take(page.written) {
                let expected = JudgementRecord {
                    object: ObjectId::new(checked as u32),
                    at: t(checked as i64 * 1_000_000 + 100_001),
                    result: Judgement::Miss,
                };
                assert_eq!(
                    *value,
                    Some(CircleFrame {
                        object: expected.object,
                        phase: Phase::Retired,
                        judgement: Some(expected),
                        unavailable: None
                    })
                );
                assert_eq!(run.committed_judgement(s, checked), Ok(Some(expected)));
                checked += 1;
            }
            if let Some(next) = page.next {
                cursor = next;
            } else {
                assert!(page.end);
                break;
            }
        }
        assert_eq!(checked, N);
        let reserved = owner.resources();
        let allocated = owner.allocations();
        let before = run.work_counts();
        for _ in 0..10 {
            assert_eq!(run.observation().ready_frame, Some(h));
            assert!(frame.object(4999).is_some());
            frame
                .read_page(h.key().cursor(31), 64, &mut output)
                .unwrap();
        }
        let after = run.work_counts();
        assert!(after.valid);
        assert_eq!(after.frame_objects - before.frame_objects, 650);
        assert_eq!(after.summaries - before.summaries, 10);
        assert_eq!(after.allocation_attempts, before.allocation_attempts);
        assert_eq!(after.publications, before.publications);
        assert!(after.publications <= 3 * N as u64 + 1);
        assert_eq!((after.search_objects, after.automatic_objects), (0, 0));
        assert_eq!(owner.resources(), reserved);
        assert_eq!(owner.allocations(), allocated);
    }
}

#[test]
fn large_inspection_rebuild_is_stepped_and_shares_only_the_immutable_source() {
    let data = input(10000);
    let c = SyntheticCoordinator::costs_for_objects(data.len()).unwrap();
    for budget in [WorkBudget::one(), WorkBudget::full()] {
        let s = source(30);
        let mut owner = SyntheticCoordinator::new(
            CoordinatorId::new(30).unwrap(),
            Limits::reference(),
            PreparationLimits::reference(),
        )
        .unwrap();
        let mut create = owner.request_create(s, &data, t(-200_001)).unwrap();
        finish(&mut create, 10000, budget);
        owner.activate(create.key(), &mut create).unwrap();
        drop(create);
        owner.request_freeze(s, t(-200_001)).unwrap();
        owner.seal_through(s, t(-200_001)).unwrap();
        let mut frozen = false;
        for _ in 0..4 {
            if matches!(
                owner.settle_freeze(s, budget).unwrap(),
                FreezeReport::Frozen { .. }
            ) {
                frozen = true;
                break;
            }
        }
        assert!(frozen);
        let old = owner.current().unwrap().observation().ready_frame.unwrap();
        let mut inspection = owner.request_inspection(s, t(5_000_000_000)).unwrap();
        let mut ready = false;
        for _ in 0..bound(10000) {
            let before = inspection.work_counts();
            let report = inspection.step(inspection.cursor(), budget).unwrap();
            let after = inspection.work_counts();
            assert!(after.valid);
            assert!(
                after.state_initializations - before.state_initializations
                    <= u64::from(budget.get())
            );
            assert!(
                after.inspection_objects - before.inspection_objects <= u64::from(budget.get())
            );
            let active = owner.current().unwrap();
            assert_eq!(active.observation().ready_frame, Some(old));
            let mut page = [None; 1];
            active
                .read_frame_page(old.key().cursor(4999), 1, &mut page)
                .unwrap();
            assert_eq!(page[0].unwrap().phase, Phase::Future);
            assert_eq!(page[0].unwrap().judgement, None);
            assert_eq!(owner.resources().precompiled_bytes, c.source_bytes);
            assert_eq!(owner.allocations().precompiled_bytes, c.source_bytes);
            if matches!(report, PreparationReport::Ready { .. }) {
                ready = true;
                break;
            }
        }
        assert!(ready);
        let stats = inspection.work_counts();
        assert_eq!(
            (stats.record_reads, stats.id_comparisons, stats.index_moves),
            (0, 0, 0)
        );
        assert_eq!(
            (stats.state_initializations, stats.inspection_objects),
            (20000, 10000)
        );
        assert_eq!(stats.allocation_attempts, 3);
        let source = owner.activate(inspection.key(), &mut inspection).unwrap();
        assert_eq!(source.transport, s.transport.checked_next().unwrap());
        assert_eq!(source.input, s.input.checked_next().unwrap());
        let run = owner.current().unwrap();
        let frame = run.ready_frame(source).unwrap().unwrap();
        assert_eq!(
            frame.header().counts,
            FrameCounts {
                inspection_skipped: 5000,
                ..FrameCounts::default()
            }
        );
        assert_eq!(
            frame.header().statistics_origin,
            StatisticsOrigin::Inspection {
                at: t(5_000_000_000)
            }
        );
        assert_eq!(
            frame.object(4999).unwrap().unavailable,
            Some(UnavailableReason::InspectionSkipped)
        );
        assert_eq!(frame.object(5000).unwrap().phase, Phase::VisibleUnjudged);
        assert_eq!(frame.object(5001).unwrap().phase, Phase::Future);
        assert_eq!(run.committed_judgement(source, 4999), Ok(None));
        assert_eq!(run.event_at(0), None);
        let sentinel = [frame.object(0)];
        let mut out = sentinel;
        assert_eq!(
            run.read_frame_page(old.key().cursor(0), 0, &mut out),
            Err(FrameReadError::StaleSource)
        );
        assert_eq!(out, sentinel);
        drop(inspection);
        assert_eq!(owner.allocations().precompiled_bytes, c.source_bytes);
        assert_eq!(
            owner.allocations().runtime_bytes,
            c.pool_bytes + core::mem::size_of::<SyntheticRun>() + c.state_bytes
        );
    }
}
