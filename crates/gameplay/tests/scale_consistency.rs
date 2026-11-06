use skin_editor_gameplay::*;

fn t(value: i64) -> MapTime {
    MapTime::from_micros(value)
}
fn source(id: u128) -> RunSource {
    RunSource {
        run: RunId::new(id).unwrap(),
        transport: TransportEpoch::new(1).unwrap(),
        input: InputEpoch::new(1).unwrap(),
        revision: MapRevision::new(710),
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
fn call_bound(n: usize) -> usize {
    64 + n * 48
}
fn owner(id: u128, limits: Limits) -> SyntheticCoordinator {
    SyntheticCoordinator::new(
        CoordinatorId::new(id).unwrap(),
        limits,
        PreparationLimits::reference(),
    )
    .unwrap()
}
fn finish(task: &mut PreparationTask<'_>, n: usize, budget: WorkBudget) {
    for _ in 0..call_bound(n) {
        match task.step(task.cursor(), budget).unwrap() {
            PreparationReport::Ready { .. } => return,
            PreparationReport::Pending { steps, .. } => assert!(steps > 0 && steps <= budget.get()),
        }
    }
    panic!("finite preparation bound");
}
fn freeze(owner: &mut SyntheticCoordinator, at: i64) {
    let source = owner.current().unwrap().source();
    owner.request_freeze(source, t(at)).unwrap();
    if owner
        .current()
        .unwrap()
        .observation()
        .sealed_through
        .is_none_or(|old| old < t(at))
    {
        owner.seal_through(source, t(at)).unwrap();
    }
    for _ in 0..32 {
        if matches!(
            owner.settle_freeze(source, WorkBudget::one()).unwrap(),
            FreezeReport::Frozen { .. }
        ) {
            return;
        }
    }
    panic!("initial freeze bound");
}

#[test]
fn activated_tasks_keep_frozen_work_receipts_for_create_load_and_inspection() {
    for (n, limits, budget) in [
        (0, Limits::tiny(), WorkBudget::one()),
        (4, Limits::tiny(), WorkBudget::one()),
        (10000, Limits::reference(), WorkBudget::full()),
    ] {
        let data = input(n);
        let mut owner = owner(200_001, limits);
        let mut create = owner
            .request_create(source(200_001), &data, t(-200_001))
            .unwrap();
        finish(&mut create, n, budget);
        let create_counts = create.work_counts();
        assert_eq!(create_counts.state_initializations, n as u64);
        owner.activate(create.key(), &mut create).unwrap();
        assert_eq!(
            create.work_counts(),
            create_counts,
            "activation must retain the transferred runtime work"
        );
        freeze(&mut owner, -200_001);
        let mut load = owner
            .request_load(source(200_001), source(200_002), &data, t(-200_001))
            .unwrap();
        finish(&mut load, n, budget);
        let load_counts = load.work_counts();
        owner.activate(load.key(), &mut load).unwrap();
        assert_eq!(load.work_counts(), load_counts);
        freeze(&mut owner, -200_001);
        let mut inspection = owner
            .request_inspection(source(200_002), t(1_000_000))
            .unwrap();
        finish(&mut inspection, n, budget);
        let inspection_counts = inspection.work_counts();
        assert_eq!(inspection_counts.state_initializations, 2 * n as u64);
        assert_eq!(inspection_counts.inspection_objects, n as u64);
        owner.activate(inspection.key(), &mut inspection).unwrap();
        assert_eq!(inspection.work_counts(), inspection_counts);
        let _ = owner.current().unwrap().observation();
        let frame = owner
            .current()
            .unwrap()
            .ready_frame(owner.current().unwrap().source())
            .unwrap()
            .unwrap();
        let _ = frame.object(0);
        assert_eq!(create.work_counts(), create_counts);
        assert_eq!(load.work_counts(), load_counts);
        assert_eq!(
            inspection.work_counts(),
            inspection_counts,
            "later active reads must not grow a transferred task receipt"
        );
    }
}

#[test]
fn invalid_start_does_not_count_a_runtime_allocation_that_was_never_attempted() {
    for n in [1, 10000] {
        let data = input(n);
        let mut owner = owner(200_003, Limits::reference());
        let baseline = owner.resources();
        let allocated = owner.allocations();
        let mut task = owner.request_create(source(200_003), &data, t(0)).unwrap();
        let mut failed = false;
        for _ in 0..call_bound(n) {
            match task.step(task.cursor(), WorkBudget::full()) {
                Err(error) => {
                    assert_eq!(error, PreparationError::Invalid(ValueError::InvalidStart));
                    failed = true;
                    break;
                }
                Ok(PreparationReport::Pending { .. }) => {}
                result => panic!("invalid start must not publish: {result:?}"),
            }
        }
        assert!(failed);
        let stats = task.work_counts();
        let cost = SyntheticCoordinator::costs_for_objects(n).unwrap();
        assert_eq!(
            stats.allocation_attempts, 5,
            "three buffers and two source wrappers; no runtime boundary reached"
        );
        assert_eq!(
            stats.allocation_bytes as usize,
            cost.source_bytes + cost.index_bytes
        );
        assert_eq!(stats.state_initializations, 0);
        assert_eq!(owner.allocations().runtime_bytes, allocated.runtime_bytes);
        assert_eq!(owner.allocations().precompiled_bytes, cost.source_bytes);
        assert_eq!(owner.candidate(), None);
        drop(task);
        assert_eq!(owner.resources(), baseline);
        assert_eq!(owner.allocations(), allocated);
    }
}

#[test]
fn stale_or_invalid_controls_leave_current_work_counts_unchanged() {
    let data = input(4);
    let s = source(200_004);
    let mut owner = owner(200_004, Limits::tiny());
    let mut create = owner.request_create(s, &data, t(-200_001)).unwrap();
    finish(&mut create, 4, WorkBudget::one());
    owner.activate(create.key(), &mut create).unwrap();
    drop(create);
    owner.seal_through(s, t(-200_001)).unwrap();
    let AdvanceReport::Pending { cursor, .. } =
        owner.advance(s, t(-200_001), WorkBudget::one()).unwrap()
    else {
        panic!("header preparation");
    };
    assert!(matches!(
        owner.continue_advance(cursor, WorkBudget::one()).unwrap(),
        AdvanceReport::Complete { .. }
    ));
    let wrong = RunSource {
        input: InputEpoch::new(2).unwrap(),
        ..s
    };
    for case in 0..9 {
        let before = owner.current().unwrap().work_counts();
        let reserved = owner.resources();
        let allocated = owner.allocations();
        let result = match case {
            0 => owner
                .continue_advance(cursor, WorkBudget::one())
                .map(|_| ()),
            1 => owner.advance(wrong, t(0), WorkBudget::one()).map(|_| ()),
            2 => owner.seal_through(wrong, t(0)),
            3 => owner.submit_inputs(wrong, &[]).map(|_| ()),
            4 => owner
                .observe_release(
                    wrong,
                    ReleaseObservation {
                        sequence: InputSequence::new(1).unwrap(),
                        slot: PhysicalSlot::PrimaryOne,
                    },
                )
                .map(|_| ()),
            5 => owner.request_freeze(wrong, t(0)),
            6 => owner.settle_freeze(wrong, WorkBudget::one()).map(|_| ()),
            7 => owner.advance(s, t(-200_002), WorkBudget::one()).map(|_| ()),
            _ => owner
                .acknowledge(EventKey::new(
                    s.run,
                    s.transport,
                    EventSequence::new(1).unwrap(),
                ))
                .map(|_| ()),
        };
        let expected = match case {
            0 => RunError::StaleCursor,
            7 => RunError::InvalidTarget,
            8 => RunError::UnknownEvent,
            _ => RunError::StaleSource,
        };
        assert_eq!(result, Err(PreparationError::Runtime(expected)));
        assert_eq!(
            owner.current().unwrap().work_counts(),
            before,
            "rejected control case {case}"
        );
        assert_eq!(owner.resources(), reserved);
        assert_eq!(owner.allocations(), allocated);
    }
}

#[test]
fn cancelled_large_inspections_keep_payloads_and_old_pages_until_their_last_owner_drops() {
    const N: usize = 10000;
    let data = input(N);
    let cost = SyntheticCoordinator::costs_for_objects(N).unwrap();
    for checkpoint in 0..4 {
        let s = source(200_010 + checkpoint);
        let mut owner = owner(200_010 + checkpoint, Limits::reference());
        let mut create = owner.request_create(s, &data, t(-200_001)).unwrap();
        finish(&mut create, N, WorkBudget::full());
        owner.activate(create.key(), &mut create).unwrap();
        drop(create);
        freeze(&mut owner, 0);
        let old = owner.current().unwrap().observation().ready_frame.unwrap();
        let old_first = owner
            .current()
            .unwrap()
            .ready_frame(s)
            .unwrap()
            .unwrap()
            .object(0);
        let mut task = owner.request_inspection(s, t(5_000_000_000)).unwrap();
        let mut reached = checkpoint == 0;
        for _ in 0..2 * N + 32 {
            if reached {
                break;
            }
            let report = task.step(task.cursor(), WorkBudget::one()).unwrap();
            let counts = task.work_counts();
            reached = match checkpoint {
                1 => {
                    task.stage() == PreparationStage::Initializing
                        && counts.state_initializations >= 4999
                }
                2 => counts.inspection_objects >= 4999,
                _ => matches!(report, PreparationReport::Ready { .. }),
            };
        }
        assert!(reached, "checkpoint {checkpoint}");
        let reserved = owner.resources();
        let allocated = owner.allocations();
        let counts = task.work_counts();
        owner.cancel(task.key()).unwrap();
        assert_eq!(owner.resources(), reserved);
        assert_eq!(owner.allocations(), allocated);
        assert_eq!(
            task.step(task.cursor(), WorkBudget::one()),
            Err(PreparationError::StaleRequest)
        );
        assert_eq!(task.work_counts(), counts);
        let mut out = [old_first; 2];
        let page = owner
            .current()
            .unwrap()
            .read_frame_page(old.key().cursor(0), 1, &mut out)
            .unwrap()
            .unwrap();
        assert_eq!((page.written, page.end), (1, false));
        assert_eq!(out, [old_first; 2]);
        let resumed = owner.resume(s).unwrap();
        let after_resume = owner.current().unwrap().observation().ready_frame.unwrap();
        assert_eq!(after_resume.at, old.at);
        assert_eq!(
            owner
                .current()
                .unwrap()
                .read_frame_page(old.key().cursor(0), 0, &mut out),
            Err(FrameReadError::StaleSource)
        );
        assert_eq!(out, [old_first; 2]);
        freeze(&mut owner, 0);
        let next = source(200_020 + checkpoint);
        let mut load = owner
            .request_load(resumed, next, &data, t(-200_001))
            .unwrap();
        finish(&mut load, N, WorkBudget::full());
        owner.activate(load.key(), &mut load).unwrap();
        drop(load);
        let active = owner.current().unwrap();
        assert_eq!(active.source(), next);
        assert!(active.ready_frame(next).unwrap().is_none());
        assert_eq!(
            active.read_frame_page(old.key().cursor(0), 0, &mut out),
            Err(FrameReadError::StaleSource)
        );
        assert_eq!(
            active.read_frame_page(after_resume.key().cursor(0), 1, &mut out),
            Err(FrameReadError::StaleSource)
        );
        assert_eq!(out, [old_first; 2]);
        assert_eq!(owner.resources().precompiled_bytes, 2 * cost.source_bytes);
        assert_eq!(owner.allocations().precompiled_bytes, 2 * cost.source_bytes);
        assert_eq!(task.work_counts(), counts);
        drop(task);
        assert_eq!(owner.resources().precompiled_bytes, cost.source_bytes);
        assert_eq!(
            owner.resources().runtime_bytes,
            cost.pool_bytes + cost.coordinator_bytes + cost.run_bytes
        );
        assert_eq!(owner.allocations().precompiled_bytes, cost.source_bytes);
        assert_eq!(owner.allocations().precompiled_allocations, 3);
        assert_eq!(
            owner.allocations().runtime_bytes,
            cost.pool_bytes + core::mem::size_of::<SyntheticRun>() + cost.state_bytes
        );
        assert_eq!(owner.allocations().runtime_allocations, 3);
    }
}

fn advance(owner: &mut SyntheticCoordinator, at: i64) {
    let s = owner.current().unwrap().source();
    owner.seal_through(s, t(at)).unwrap();
    let mut report = owner.advance(s, t(at), WorkBudget::one()).unwrap();
    for _ in 0..32 {
        match report {
            AdvanceReport::Complete { .. } => return,
            AdvanceReport::Pending { cursor, reason, .. } => {
                assert_eq!(reason, PendingReason::WorkBudget);
                report = owner.continue_advance(cursor, WorkBudget::one()).unwrap();
            }
        }
    }
    panic!("early complete frame bound");
}

#[test]
fn large_pending_and_failed_prefixes_preserve_the_old_complete_section_and_page_identity() {
    const N: usize = 10000;
    let data = input(N);
    let s = source(200_030);
    let mut owner = owner(200_030, Limits::reference());
    let mut create = owner.request_create(s, &data, t(-200_001)).unwrap();
    finish(&mut create, N, WorkBudget::full());
    owner.activate(create.key(), &mut create).unwrap();
    drop(create);
    advance(&mut owner, -1);
    let stale = owner.current().unwrap().observation().ready_frame.unwrap();
    advance(&mut owner, 100_000);
    let old = owner.current().unwrap().observation().ready_frame.unwrap();
    let first = owner
        .current()
        .unwrap()
        .ready_frame(s)
        .unwrap()
        .unwrap()
        .object(0);
    owner.seal_through(s, t(100_001)).unwrap();
    let AdvanceReport::Pending { cursor, .. } =
        owner.advance(s, t(100_001), WorkBudget::one()).unwrap()
    else {
        panic!("committed prefix only");
    };
    let partial = owner.current().unwrap().observation();
    assert_eq!(partial.status, RunStatus::Pending);
    assert_eq!(partial.committed_counts.misses, 1);
    assert_eq!(partial.ready_frame, Some(old));
    assert_eq!(
        owner
            .current()
            .unwrap()
            .committed_judgement(s, 0)
            .unwrap()
            .unwrap()
            .at,
        t(100_001)
    );
    owner.cancel_advance(cursor.request()).unwrap();
    let before = owner.current().unwrap().work_counts();
    assert_eq!(
        owner.continue_advance(cursor, WorkBudget::one()),
        Err(PreparationError::Runtime(RunError::StaleCursor))
    );
    assert_eq!(owner.current().unwrap().work_counts(), before);
    let allocated = owner.allocations();
    for failed in [false, true] {
        if failed {
            assert_eq!(
                owner.submit_inputs(
                    s,
                    &[InputEvent {
                        sequence: InputSequence::new(1).unwrap(),
                        at: t(100_001),
                        slot: PhysicalSlot::PrimaryOne,
                        edge: InputEdge::Press(Point::new(0, 0).unwrap()),
                    }]
                ),
                Err(PreparationError::Runtime(RunError::LateInput))
            );
        }
        let run = owner.current().unwrap();
        let state = run.observation();
        if failed {
            assert!(matches!(state.status, RunStatus::Failed(_)));
        }
        assert_eq!(state.ready_frame, Some(old));
        assert_eq!(state.committed_counts.misses, 1);
        assert_eq!(old.counts.misses, 0);
        assert_eq!(old.at, t(100_000));
        let frame = run.ready_frame(s).unwrap().unwrap();
        assert_eq!(frame.object(0), first);
        let before = run.work_counts();
        let mut out = [first; 64];
        let page = frame.read_page(old.key().cursor(0), 64, &mut out).unwrap();
        assert_eq!(page.written, 64);
        assert_eq!(out[0], first);
        assert_eq!(run.work_counts().frame_objects - before.frame_objects, 64);
        assert_eq!(out[0].unwrap().judgement, None);
        assert_eq!(out[0].unwrap().phase, Phase::VisibleUnjudged);
        assert!(
            out[1..]
                .iter()
                .all(|v| v.is_some_and(|v| v.phase == Phase::Future && v.judgement.is_none()))
        );
        let intact = out;
        let before = run.work_counts();
        assert_eq!(
            run.read_frame_page(stale.key().cursor(0), 0, &mut out),
            Err(FrameReadError::StaleFrame)
        );
        assert_eq!(
            run.read_frame_page(old.key().cursor(N + 1), 1, &mut out),
            Err(FrameReadError::InvalidPosition)
        );
        assert_eq!(
            run.read_frame_page(old.key().cursor(0), 65, &mut out),
            Err(FrameReadError::InvalidPageSize)
        );
        assert_eq!(
            run.read_frame_page(old.key().cursor(0), 64, &mut out[..63]),
            Err(FrameReadError::OutputCapacity {
                required: 64,
                available: 63
            })
        );
        assert_eq!(run.work_counts(), before);
        assert_eq!(out, intact);
        let end = run
            .read_frame_page(old.key().cursor(N), 64, &mut [])
            .unwrap()
            .unwrap();
        assert_eq!((end.written, end.end, end.next), (0, true, None));
        assert_eq!(owner.allocations(), allocated);
    }
    assert_eq!(
        owner.current().unwrap().event_at(0).unwrap().kind,
        EventKind::Judgement(Judgement::Miss)
    );
    assert_eq!(owner.current().unwrap().observation().pending_events, 1);
}

#[test]
fn uneven_merge_rounds_preserve_the_earliest_input_error_in_large_maps() {
    for n in [5, 31, 32, 33, 65, 10000] {
        for earlier_order_error in [false, true] {
            let mut data = input(n);
            for (i, circle) in data.iter_mut().enumerate() {
                *circle = Circle::new(
                    ObjectId::new((i as u32).wrapping_mul(2_654_435_761).rotate_left(13)),
                    circle.target(),
                    circle.center(),
                )
                .unwrap();
            }
            data[3] = Circle::new(data[1].id(), data[3].target(), data[3].center()).unwrap();
            data[n - 1] =
                Circle::new(data[0].id(), data[n - 1].target(), data[n - 1].center()).unwrap();
            if earlier_order_error {
                data[2] = Circle::new(data[2].id(), t(-1_000_000), data[2].center()).unwrap();
            }
            let expected = if earlier_order_error {
                (2, ValueError::UnsortedMap)
            } else {
                (3, ValueError::DuplicateObject)
            };
            for budget in [WorkBudget::one(), WorkBudget::full()] {
                let mut prep = MapPreparation::with_limits(
                    source(200_040).revision,
                    &data,
                    Limits::reference(),
                )
                .unwrap();
                let mut rejected = false;
                for _ in 0..call_bound(n) {
                    match prep.step(budget) {
                        Err(error) => {
                            assert_eq!((prep.failed_record_index().unwrap(), error), expected);
                            rejected = true;
                            break;
                        }
                        Ok(PreparationProgress::Pending { .. }) => {}
                        result => panic!("invalid source published: {result:?}"),
                    }
                }
                assert!(rejected);
                assert_eq!(prep.finish(), Err(expected.1));
            }
        }
    }
}

#[test]
fn standalone_capacity_checks_and_partial_initialization_keep_their_own_scope() {
    const N: usize = 10000;
    let data = input(N);
    let s = source(200_050);
    let map_bytes = MapPreparation::storage_bytes(N).unwrap();
    let limits = |bytes| {
        Limits::try_new(64, 64, 64, 1, bytes)
            .unwrap()
            .with_object_capacity(N)
            .unwrap()
    };
    assert!(matches!(
        MapPreparation::with_limits(s.revision, &data, limits(map_bytes - 1)),
        Err(ValueError::StorageLimit)
    ));
    let run_bytes = SyntheticRun::standalone_bytes_for_objects(N).unwrap();
    for exact in [false, true] {
        let mut prep = MapPreparation::with_limits(s.revision, &data, limits(map_bytes)).unwrap();
        let counts = prep.work_counts();
        assert_eq!(
            prep.step(WorkBudget::full()),
            Err(ValueError::InvalidBudget)
        );
        assert_eq!(prep.work_counts(), counts);
        assert_eq!(prep.capacities(), [0; 3]);
        let mut complete = false;
        for _ in 0..call_bound(N) {
            if matches!(
                prep.step(WorkBudget::one()).unwrap(),
                PreparationProgress::Complete { .. }
            ) {
                complete = true;
                break;
            }
        }
        assert!(complete);
        let map = prep.finish().unwrap();
        if !exact {
            assert!(matches!(
                RunPreparation::new(s, map, t(-200_001), limits(run_bytes - 1)),
                Err(ValueError::StorageLimit)
            ));
            continue;
        }
        let mut initialization =
            RunPreparation::new(s, map, t(-200_001), limits(run_bytes)).unwrap();
        let before = initialization.work_counts();
        assert_eq!(
            initialization.step(WorkBudget::full()),
            Err(ValueError::InvalidBudget)
        );
        assert_eq!(initialization.work_counts(), before);
        for initialized in 0..=N {
            let result = initialization.step(WorkBudget::one()).unwrap();
            assert_eq!(
                initialization.work_counts().state_initializations,
                initialized as u64
            );
            if initialized < N {
                assert_eq!(
                    result,
                    RunPreparationProgress::Pending {
                        initialized,
                        total: N
                    }
                );
            } else {
                assert_eq!(result, RunPreparationProgress::Complete);
            }
        }
        let before = initialization.work_counts();
        let run = initialization.finish().unwrap();
        assert_eq!(run.work_counts(), before);
        assert!(run.ready_frame(s).unwrap().is_none());
        assert_eq!(run.state_capacities(), [N, 0]);
    }
}
