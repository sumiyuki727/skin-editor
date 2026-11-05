use skin_editor_gameplay::*;

fn t(at: i64) -> MapTime {
    MapTime::from_micros(at)
}
fn source(id: u128) -> RunSource {
    RunSource {
        run: RunId::new(id).unwrap(),
        transport: TransportEpoch::new(1).unwrap(),
        input: InputEpoch::new(1).unwrap(),
        revision: MapRevision::new(6),
    }
}
fn circles(n: usize) -> Vec<Circle> {
    (0..n)
        .map(|i| {
            Circle::new(
                ObjectId::new([400, 7, 99, 42, 8][i]),
                t(i as i64 * 1_000_000),
                Point::new(0, 0).unwrap(),
            )
            .unwrap()
        })
        .collect()
}
fn run(id: u128, n: usize) -> SyntheticRun {
    SyntheticRun::new(
        source(id),
        SyntheticMap::try_new(source(id).revision, &circles(n)).unwrap(),
        t(-200_001),
        Limits::tiny(),
    )
    .unwrap()
}
fn advance(run: &mut SyntheticRun, at: i64) {
    run.seal_through(run.source(), t(at)).unwrap();
    let mut report = run.advance(run.source(), t(at), WorkBudget::one()).unwrap();
    for _ in 0..256 {
        match report {
            AdvanceReport::Complete { .. } => return,
            AdvanceReport::Pending { cursor, reason, .. } => {
                assert_eq!(reason, PendingReason::WorkBudget);
                report = run.continue_advance(cursor, WorkBudget::one()).unwrap();
            }
        }
    }
    panic!("four-slot advance bound");
}
fn freeze(run: &mut SyntheticRun, at: i64) {
    run.request_freeze(run.source(), t(at)).unwrap();
    if run
        .observation()
        .sealed_through
        .is_none_or(|sealed| sealed < t(at))
    {
        run.seal_through(run.source(), t(at)).unwrap();
    }
    for _ in 0..256 {
        if matches!(
            run.settle_freeze(run.source(), WorkBudget::one()).unwrap(),
            FreezeReport::Frozen { .. }
        ) {
            return;
        }
    }
    panic!("freeze bound");
}
fn press() -> InputEvent {
    InputEvent {
        sequence: InputSequence::new(1).unwrap(),
        at: t(0),
        slot: PhysicalSlot::PrimaryOne,
        edge: InputEdge::Press(Point::new(0, 0).unwrap()),
    }
}
fn pending_hit(run: &mut SyntheticRun) -> AdvanceCursor {
    advance(run, -1);
    run.submit_inputs(run.source(), &[press()]).unwrap();
    run.seal_through(run.source(), t(0)).unwrap();
    let mut report = run.advance(run.source(), t(0), WorkBudget::one()).unwrap();
    for _ in 0..32 {
        let AdvanceReport::Pending { cursor, .. } = report else {
            panic!("must expose committed prefix");
        };
        if run.committed_judgement(run.source(), 0).unwrap().is_some() {
            return cursor;
        }
        report = run.continue_advance(cursor, WorkBudget::one()).unwrap();
    }
    panic!("hit bound");
}

#[test]
fn pages_follow_record_order_and_validate_output_atomically() {
    let mut run = run(100_001, 4);
    assert!(run.ready_frame(run.source()).unwrap().is_none());
    assert_eq!(run.committed_judgement(run.source(), usize::MAX), Ok(None));
    advance(&mut run, 0);
    let before = run.observation();
    let view = run.ready_frame(run.source()).unwrap().unwrap();
    let header = view.header();
    assert_eq!(header.len(), 4);
    assert_eq!(header.profile_version, 1);
    assert_eq!(header.counts, FrameCounts::default());
    assert_eq!(view.object(0).unwrap().object, ObjectId::new(400));
    assert_eq!(view.object(1).unwrap().object, ObjectId::new(7));
    assert_eq!(view.object(usize::MAX), None);
    let sentinel = Some(view.object(3).unwrap());
    let mut out = [sentinel; 6];
    let first = view.read_page(header.key().cursor(0), 1, &mut out).unwrap();
    assert_eq!((first.written, first.end), (1, false));
    assert_eq!(out[0], view.object(0));
    assert!(out[1..].iter().all(|item| *item == sentinel));
    let repeated = view.read_page(header.key().cursor(0), 1, &mut out).unwrap();
    assert_eq!(repeated, first);
    let last = view
        .read_page(first.next.unwrap(), 64, &mut out[..3])
        .unwrap();
    assert_eq!((last.written, last.end, last.next), (3, true, None));
    assert_eq!(out[0].unwrap().object, ObjectId::new(7));
    assert_eq!(out[2].unwrap().object, ObjectId::new(42));
    let intact = out;
    assert_eq!(
        view.read_page(header.key().cursor(0), 0, &mut out),
        Err(FrameReadError::InvalidPageSize)
    );
    assert_eq!(
        view.read_page(header.key().cursor(0), 65, &mut out),
        Err(FrameReadError::InvalidPageSize)
    );
    assert_eq!(
        view.read_page(header.key().cursor(usize::MAX), 1, &mut out),
        Err(FrameReadError::InvalidPosition)
    );
    assert_eq!(
        view.read_page(header.key().cursor(5), 1, &mut out),
        Err(FrameReadError::InvalidPosition)
    );
    assert_eq!(
        view.read_page(header.key().cursor(0), 4, &mut out[..3]),
        Err(FrameReadError::OutputCapacity {
            required: 4,
            available: 3
        })
    );
    assert_eq!(out, intact);
    let end = view.read_page(header.key().cursor(4), 64, &mut []).unwrap();
    assert_eq!((end.written, end.end, end.next), (0, true, None));
    assert_eq!(run.observation(), before);
}

#[test]
fn empty_complete_sections_are_distinct_from_not_yet_evaluated() {
    let mut run = run(100_002, 0);
    assert_eq!(run.observation().ready_frame, None);
    assert!(run.ready_frame(run.source()).unwrap().is_none());
    advance(&mut run, 0);
    let view = run.ready_frame(run.source()).unwrap().unwrap();
    assert!(view.is_empty());
    assert_eq!(view.header().key().publication(), 1);
    assert_eq!(view.object(0), None);
    assert_eq!(
        view.read_page(view.header().key().cursor(0), 1, &mut [])
            .unwrap()
            .written,
        0
    );
    assert_eq!(
        view.read_page(view.header().key().cursor(0), 0, &mut []),
        Err(FrameReadError::InvalidPageSize)
    );
}

#[test]
fn pending_ledger_and_old_complete_frame_remain_distinct_through_cancel_and_publish() {
    let mut run = run(100_003, 2);
    let cursor = pending_hit(&mut run);
    let partial = run.observation();
    let old = partial.ready_frame.unwrap();
    assert_eq!(partial.status, RunStatus::Pending);
    assert_eq!(partial.committed_counts.hits, 1);
    assert_eq!(old.counts.hits, 0);
    assert_eq!(old.at, t(-1));
    assert_eq!(
        run.committed_judgement(run.source(), 0)
            .unwrap()
            .unwrap()
            .result,
        Judgement::Hit
    );
    let view = run.ready_frame(run.source()).unwrap().unwrap();
    assert_eq!(view.object(0).unwrap().judgement, None);
    assert_eq!(view.object(0).unwrap().phase, Phase::VisibleUnjudged);
    let mut out = [None; 2];
    run.read_frame_page(old.key().cursor(0), 2, &mut out)
        .unwrap();
    assert_eq!(out[0].unwrap().judgement, None);
    run.acknowledge(run.event_at(0).unwrap().key).unwrap();
    assert_eq!(run.observation().ready_frame, Some(old));
    run.cancel_request(cursor.request()).unwrap();
    assert_eq!(run.observation().ready_frame, Some(old));
    advance(&mut run, 0);
    let new = run.observation().ready_frame.unwrap();
    assert_eq!(new.counts.hits, 1);
    assert!(new.key().publication() > old.key().publication());
    let intact = out;
    assert_eq!(
        run.read_frame_page(old.key().cursor(0), 0, &mut out),
        Err(FrameReadError::StaleFrame)
    );
    assert_eq!(out, intact);
    assert_eq!(
        run.ready_frame(run.source())
            .unwrap()
            .unwrap()
            .object(0)
            .unwrap()
            .judgement,
        run.committed_judgement(run.source(), 0).unwrap()
    );
}

#[test]
fn failed_prefix_still_exposes_only_the_old_complete_frame() {
    let mut run = run(100_004, 1);
    pending_hit(&mut run);
    let old = run.observation().ready_frame.unwrap();
    let late = InputEvent {
        sequence: InputSequence::new(2).unwrap(),
        ..press()
    };
    assert_eq!(
        run.submit_inputs(run.source(), &[late]),
        Err(RunError::LateInput)
    );
    let before = run.observation();
    assert!(matches!(before.status, RunStatus::Failed(_)));
    assert_eq!(before.committed_counts.hits, 1);
    assert_eq!(before.ready_frame, Some(old));
    assert_eq!(
        run.ready_frame(run.source())
            .unwrap()
            .unwrap()
            .object(0)
            .unwrap()
            .judgement,
        None
    );
    let mut out = [None; 1];
    run.read_frame_page(old.key().cursor(0), 1, &mut out)
        .unwrap();
    assert_eq!(out[0].unwrap().judgement, None);
    assert_eq!(run.observation(), before);
}

#[test]
fn no_op_controls_keep_keys_but_source_changes_invalidate_old_pages_at_the_same_time() {
    let mut run = run(100_005, 1);
    run.submit_inputs(run.source(), &[press()]).unwrap();
    advance(&mut run, 0);
    let old = run.observation().ready_frame.unwrap();
    run.acknowledge(run.event_at(0).unwrap().key).unwrap();
    advance(&mut run, 0);
    assert_eq!(run.observation().ready_frame, Some(old));
    run.seal_through(run.source(), t(1)).unwrap();
    let AdvanceReport::Pending { cursor, .. } =
        run.advance(run.source(), t(1), WorkBudget::one()).unwrap()
    else {
        panic!("fixed header preparation yields");
    };
    run.cancel_request(cursor.request()).unwrap();
    freeze(&mut run, 0);
    assert_eq!(run.observation().ready_frame, Some(old));
    let switched = run
        .select_driver(run.source(), InputDriver::Regression)
        .unwrap();
    let at_switch = run.observation().ready_frame.unwrap();
    assert_eq!(
        (at_switch.at, at_switch.key().publication()),
        (old.at, old.key().publication())
    );
    let mut out = [None; 1];
    assert_eq!(
        run.read_frame_page(old.key().cursor(0), 0, &mut out),
        Err(FrameReadError::StaleSource)
    );
    let resumed = run.resume(switched).unwrap();
    assert_eq!(
        run.read_frame_page(at_switch.key().cursor(0), 1, &mut out),
        Err(FrameReadError::StaleSource)
    );
    assert_eq!(
        run.ready_frame(resumed)
            .unwrap()
            .unwrap()
            .header()
            .counts
            .hits,
        1
    );
}

#[test]
fn every_source_component_is_checked_before_page_size_or_position() {
    let mut run = run(100_006, 1);
    advance(&mut run, 0);
    let header = run.observation().ready_frame.unwrap();
    for changed in [
        RunSource {
            run: RunId::new(100_007).unwrap(),
            ..run.source()
        },
        RunSource {
            transport: TransportEpoch::new(2).unwrap(),
            ..run.source()
        },
        RunSource {
            input: InputEpoch::new(2).unwrap(),
            ..run.source()
        },
        RunSource {
            revision: MapRevision::new(7),
            ..run.source()
        },
    ] {
        let mut wrong = header;
        wrong.source = changed;
        let before = run.observation();
        assert_eq!(
            run.read_frame_page(wrong.key().cursor(usize::MAX), 0, &mut []),
            Err(FrameReadError::StaleSource)
        );
        assert_eq!(
            run.ready_frame(changed).unwrap_err(),
            FrameReadError::StaleSource
        );
        assert_eq!(
            run.committed_judgement(changed, usize::MAX),
            Err(FrameReadError::StaleSource)
        );
        assert_eq!(run.observation(), before);
    }
}

fn finish(task: &mut PreparationTask<'_>) {
    for _ in 0..128 {
        if matches!(
            task.step(task.cursor(), WorkBudget::one()).unwrap(),
            PreparationReport::Ready { .. }
        ) {
            return;
        }
    }
    panic!("preparation bound");
}
fn freeze_owner(owner: &mut SyntheticCoordinator, at: i64) {
    let s = owner.current().unwrap().source();
    owner.request_freeze(s, t(at)).unwrap();
    owner.seal_through(s, t(at)).unwrap();
    for _ in 0..256 {
        if matches!(
            owner.settle_freeze(s, WorkBudget::one()).unwrap(),
            FreezeReport::Frozen { .. }
        ) {
            return;
        }
    }
    panic!("owner freeze bound");
}

#[test]
fn inspection_cancellation_keeps_pages_and_activation_closes_the_old_source() {
    let mut owner = SyntheticCoordinator::new(
        CoordinatorId::new(100_008).unwrap(),
        Limits::tiny(),
        PreparationLimits::tiny(),
    )
    .unwrap();
    let input = circles(2);
    let mut create = owner
        .request_create(source(100_008), &input, t(-200_001))
        .unwrap();
    finish(&mut create);
    owner.activate(create.key(), &mut create).unwrap();
    drop(create);
    freeze_owner(&mut owner, 0);
    let base = owner.current().unwrap().source();
    let old = owner.current().unwrap().observation().ready_frame.unwrap();
    let mut a = owner.request_inspection(base, t(1_000_000)).unwrap();
    finish(&mut a);
    owner.cancel(a.key()).unwrap();
    let before_read = owner.resources();
    let mut out = [None; 2];
    owner
        .current()
        .unwrap()
        .read_frame_page(old.key().cursor(0), 2, &mut out)
        .unwrap();
    assert_eq!(owner.resources(), before_read);
    drop(a);
    let mut b = owner.request_inspection(base, t(1_000_000)).unwrap();
    finish(&mut b);
    let active = owner.activate(b.key(), &mut b).unwrap();
    let after = owner.current().unwrap();
    assert_eq!(
        after.read_frame_page(old.key().cursor(0), 2, &mut out),
        Err(FrameReadError::StaleSource)
    );
    let view = after.ready_frame(active).unwrap().unwrap();
    assert_eq!(
        view.header().counts,
        FrameCounts {
            inspection_skipped: 1,
            ..FrameCounts::default()
        }
    );
    assert_eq!(
        view.object(0).unwrap().unavailable,
        Some(UnavailableReason::InspectionSkipped)
    );
    assert_eq!(view.object(1).unwrap().judgement, None);
    assert_eq!(after.event_at(0), None);
}

#[test]
fn capacities_apply_to_standalone_and_coordinated_entry_points_without_extra_pools() {
    assert_eq!(MAX_CIRCLES, 4);
    assert_eq!(Limits::reference().object_capacity(), 4);
    for bad in [0, 5, usize::MAX] {
        assert_eq!(
            Limits::tiny().with_object_capacity(bad),
            Err(ValueError::InvalidLimits)
        );
    }
    let s = source(100_009);
    assert!(matches!(
        MapPreparation::new(s.revision, &circles(5)),
        Err(ValueError::ObjectLimit)
    ));
    assert_eq!(
        SyntheticMap::try_new(s.revision, &circles(5)),
        Err(ValueError::ObjectLimit)
    );
    let one = Limits::tiny().with_object_capacity(1).unwrap();
    let map = SyntheticMap::try_new(s.revision, &circles(2)).unwrap();
    assert!(matches!(
        SyntheticRun::new(s, map, t(-200_001), one),
        Err(ValueError::ObjectLimit)
    ));
    let mut owner = SyntheticCoordinator::new(
        CoordinatorId::new(100_009).unwrap(),
        one,
        PreparationLimits::reference(),
    )
    .unwrap();
    let before = owner.resources();
    assert!(matches!(
        owner.request_create(s, &circles(2), t(-200_001)),
        Err(PreparationError::Invalid(ValueError::ObjectLimit))
    ));
    assert_eq!(owner.resources(), before);
    assert_eq!(owner.candidate(), None);
    let costs = SyntheticCoordinator::costs();
    assert_eq!(
        costs.source_bytes,
        costs.source_map_bytes + costs.source_owner_bytes
    );
    assert_eq!(costs.source_map_bytes, core::mem::size_of::<SyntheticMap>());
    let exact = Limits::try_new(4, 2, 4, 1, SyntheticRun::static_storage_bytes()).unwrap();
    assert!(matches!(
        SyntheticRun::new(s, map, t(-200_001), exact.with_object_capacity(1).unwrap()),
        Err(ValueError::ObjectLimit)
    ));
    let empty = SyntheticMap::try_new(s.revision, &[]).unwrap();
    assert!(matches!(
        SyntheticRun::new(s, empty, t(0), exact),
        Err(ValueError::StorageLimit)
    ));
    let standalone = Limits::try_new(4, 2, 4, 1, SyntheticRun::standalone_storage_bytes()).unwrap();
    assert!(SyntheticRun::new(s, empty, t(0), standalone).is_ok());
    let pool = PreparationLimits::try_new(
        costs.source_bytes,
        costs.pool_bytes + costs.coordinator_bytes + costs.task_bytes + costs.run_bytes,
        1,
    )
    .unwrap();
    let mut owner =
        SyntheticCoordinator::new(CoordinatorId::new(100_010).unwrap(), exact, pool).unwrap();
    let mut task = owner.request_create(s, &[], t(0)).unwrap();
    finish(&mut task);
    owner.activate(task.key(), &mut task).unwrap();
    assert_eq!(
        owner.resources().runtime_bytes,
        costs.pool_bytes + costs.coordinator_bytes + costs.task_bytes + costs.run_bytes
    );
    drop(task);
    assert_eq!(
        owner.resources().runtime_bytes,
        costs.pool_bytes + costs.coordinator_bytes + costs.run_bytes
    );
}

#[test]
fn complete_miss_counts_and_retirement_keep_their_own_time_and_statistics_origin() {
    let mut run = run(100_014, 1);
    advance(&mut run, 0);
    let before = run.observation().ready_frame.unwrap();
    assert_eq!(before.counts.misses, 0);
    advance(&mut run, 100_001);
    let frame = run.ready_frame(run.source()).unwrap().unwrap();
    assert_eq!(
        frame.header().counts,
        FrameCounts {
            misses: 1,
            ..FrameCounts::default()
        }
    );
    assert_eq!(
        frame.header().statistics_origin,
        StatisticsOrigin::FromStart { start: t(-200_001) }
    );
    assert_eq!(frame.object(0).unwrap().phase, Phase::ResolvedVisual);
    assert_eq!(frame.object(0).unwrap().judgement.unwrap().at, t(100_001));
    assert_eq!(
        run.read_frame_page(before.key().cursor(0), 1, &mut [None]),
        Err(FrameReadError::StaleFrame)
    );
    advance(&mut run, 300_000);
    let frame = run.ready_frame(run.source()).unwrap().unwrap();
    assert_eq!(frame.object(0).unwrap().phase, Phase::Retired);
    assert_eq!(frame.header().counts.misses, 1);
    assert_eq!(run.observation().pending_events, 1);
    assert_eq!(
        run.event_at(0).unwrap().kind,
        EventKind::Judgement(Judgement::Miss)
    );
}

#[test]
fn replacement_pages_and_lower_object_limits_do_not_reuse_the_old_run() {
    let mut owner = SyntheticCoordinator::new(
        CoordinatorId::new(100_011).unwrap(),
        Limits::tiny().with_object_capacity(1).unwrap(),
        PreparationLimits::tiny(),
    )
    .unwrap();
    let input = circles(1);
    let mut create = owner
        .request_create(source(100_011), &input, t(-200_001))
        .unwrap();
    finish(&mut create);
    owner.activate(create.key(), &mut create).unwrap();
    drop(create);
    freeze_owner(&mut owner, 0);
    let old = owner.current().unwrap().observation().ready_frame.unwrap();
    let usage = owner.resources();
    assert!(matches!(
        owner.request_load(old.source, source(100_012), &circles(2), t(-200_001)),
        Err(PreparationError::Invalid(ValueError::ObjectLimit))
    ));
    assert_eq!(owner.resources(), usage);
    assert_eq!(
        owner.current().unwrap().observation().ready_frame,
        Some(old)
    );
    let mut held = owner.request_inspection(old.source, t(0)).unwrap();
    finish(&mut held);
    owner.cancel(held.key()).unwrap();
    let mut load = owner
        .request_load(old.source, source(100_013), &input, t(-200_001))
        .unwrap();
    finish(&mut load);
    owner.activate(load.key(), &mut load).unwrap();
    let current = owner.current().unwrap();
    assert_eq!(current.source().transport, old.source.transport);
    assert_eq!(current.source().input, old.source.input);
    assert_eq!(current.source().revision, old.source.revision);
    assert_eq!(current.observation().ready_frame, None);
    assert_eq!(
        current.read_frame_page(old.key().cursor(0), 0, &mut []),
        Err(FrameReadError::StaleSource)
    );
    assert_eq!(
        owner.resources().precompiled_bytes,
        2 * SyntheticCoordinator::costs().source_bytes
    );
    drop(held);
    assert_eq!(
        owner.resources().precompiled_bytes,
        SyntheticCoordinator::costs().source_bytes
    );
}

fn assert_old_pair(run: &SyntheticRun, old: FrameHeader, objects: [Option<CircleFrame>; 2]) {
    let before = run.observation();
    let view = run.ready_frame(run.source()).unwrap().unwrap();
    assert_eq!(view.header(), old);
    assert_eq!([view.object(0), view.object(1)], objects);
    let mut output = [objects[0]; 3];
    let page = view
        .read_page(old.key().cursor(0), 64, &mut output)
        .unwrap();
    assert_eq!((page.written, page.end, page.next), (2, true, None));
    assert_eq!(&output[..2], &objects);
    assert_eq!(output[2], objects[0]);
    assert_eq!(run.observation(), before);
}

#[test]
fn same_time_hit_and_miss_never_leak_into_old_pages_even_when_the_prefix_fails() {
    for fault in [false, true] {
        let s = source(if fault { 100_016 } else { 100_015 });
        let input = [
            Circle::new(ObjectId::new(90), t(0), Point::new(0, 0).unwrap()).unwrap(),
            Circle::new(ObjectId::new(3), t(200_001), Point::new(0, 0).unwrap()).unwrap(),
        ];
        let mut run = SyntheticRun::new(
            s,
            SyntheticMap::try_new(s.revision, &input).unwrap(),
            t(-200_001),
            Limits::tiny(),
        )
        .unwrap();
        advance(&mut run, 100_000);
        let old = run.observation().ready_frame.unwrap();
        let view = run.ready_frame(s).unwrap().unwrap();
        let objects = [view.object(0), view.object(1)];
        assert!(objects.iter().all(|object| {
            object.is_some_and(|object| {
                object.phase == Phase::VisibleUnjudged && object.judgement.is_none()
            })
        }));
        run.submit_inputs(
            s,
            &[InputEvent {
                at: t(100_001),
                ..press()
            }],
        )
        .unwrap();
        run.seal_through(s, t(100_001)).unwrap();
        let mut report = run.advance(s, t(100_001), WorkBudget::one()).unwrap();
        let mut blocked = None;
        for _ in 0..32 {
            assert_old_pair(&run, old, objects);
            let AdvanceReport::Pending { cursor, reason, .. } = report else {
                panic!("same-time miss needs a third event slot");
            };
            if reason == PendingReason::EventBackpressure {
                blocked = Some(cursor);
                break;
            }
            report = run.continue_advance(cursor, WorkBudget::one()).unwrap();
        }
        let blocked = blocked.expect("four-slot same-time backpressure bound");
        assert_eq!(run.observation().committed_counts.hits, 1);
        assert_eq!(run.observation().committed_counts.misses, 0);
        assert_eq!(run.committed_judgement(s, 0), Ok(None));
        assert_eq!(
            run.committed_judgement(s, 1).unwrap().unwrap().result,
            Judgement::Hit
        );
        assert_eq!(run.observation().pending_events, 2);
        run.acknowledge(run.event_at(0).unwrap().key).unwrap();
        assert_old_pair(&run, old, objects);
        report = run.continue_advance(blocked, WorkBudget::one()).unwrap();
        assert!(matches!(report, AdvanceReport::Pending { .. }));
        assert_eq!(
            run.observation().committed_counts,
            FrameCounts {
                hits: 1,
                misses: 1,
                inspection_skipped: 0,
            }
        );
        assert_old_pair(&run, old, objects);
        assert_eq!(
            run.committed_judgement(s, 0).unwrap().unwrap().at,
            t(100_001)
        );
        if fault {
            assert_eq!(
                run.submit_inputs(
                    s,
                    &[InputEvent {
                        sequence: InputSequence::new(2).unwrap(),
                        at: t(100_001),
                        slot: PhysicalSlot::SecondaryOne,
                        ..press()
                    }]
                ),
                Err(RunError::LateInput)
            );
            assert!(matches!(run.observation().status, RunStatus::Failed(_)));
            assert_old_pair(&run, old, objects);
            assert_eq!(run.observation().pending_events, 2);
        } else {
            for _ in 0..16 {
                match report {
                    AdvanceReport::Complete { .. } => break,
                    AdvanceReport::Pending { cursor, .. } => {
                        assert_old_pair(&run, old, objects);
                        report = run.continue_advance(cursor, WorkBudget::one()).unwrap();
                    }
                }
            }
            assert!(matches!(report, AdvanceReport::Complete { .. }));
            let new = run.ready_frame(s).unwrap().unwrap();
            assert_eq!(new.header().counts, run.observation().committed_counts);
            for index in 0..2 {
                assert_eq!(
                    new.object(index).unwrap().judgement,
                    run.committed_judgement(s, index).unwrap()
                );
            }
            let mut output = objects;
            assert_eq!(
                run.read_frame_page(old.key().cursor(0), 2, &mut output),
                Err(FrameReadError::StaleFrame)
            );
            assert_eq!(output, objects);
        }
        assert_eq!(run.event_at(0).unwrap().kind, EventKind::SoundIntent);
        assert_eq!(
            run.event_at(1).unwrap().kind,
            EventKind::Judgement(Judgement::Miss)
        );
    }
}

#[test]
fn every_tiny_return_point_reads_lifecycle_and_miss_facts_at_the_complete_time() {
    let mut run = run(100_017, 4);
    advance(&mut run, -1);
    run.seal_through(run.source(), t(3_300_000)).unwrap();
    let mut report = run
        .advance(run.source(), t(3_300_000), WorkBudget::one())
        .unwrap();
    let mut previous: Option<FrameKey> = None;
    for _ in 0..256 {
        let before = run.observation();
        let frame = run.ready_frame(run.source()).unwrap().unwrap();
        let h = frame.header();
        let mut expected = [None; 4];
        let mut misses = 0;
        for (i, slot) in expected.iter_mut().enumerate() {
            let target = i as i64 * 1_000_000;
            let missed = h.at >= t(target + 100_001);
            misses += usize::from(missed);
            *slot = Some(CircleFrame {
                object: ObjectId::new([400, 7, 99, 42][i]),
                phase: if h.at < t(target - 200_000) {
                    Phase::Future
                } else if h.at >= t(target + 300_000) {
                    Phase::Retired
                } else if missed {
                    Phase::ResolvedVisual
                } else {
                    Phase::VisibleUnjudged
                },
                judgement: missed.then_some(JudgementRecord {
                    object: ObjectId::new([400, 7, 99, 42][i]),
                    at: t(target + 100_001),
                    result: Judgement::Miss,
                }),
                unavailable: None,
            });
            assert_eq!(frame.object(i), *slot);
        }
        assert_eq!(
            h.counts,
            FrameCounts {
                misses,
                ..FrameCounts::default()
            }
        );
        let mut out = [None; 5];
        let first = frame.read_page(h.key().cursor(0), 2, &mut out).unwrap();
        let last = frame
            .read_page(first.next.unwrap(), 64, &mut out[2..])
            .unwrap();
        assert!(last.end);
        assert_eq!(&out[..4], &expected);
        assert_eq!(out[4], None);
        if let Some(old) = previous.filter(|old| *old != h.key()) {
            assert_eq!(
                run.read_frame_page(old.cursor(0), 4, &mut out),
                Err(FrameReadError::StaleFrame)
            );
            assert_eq!(&out[..4], &expected);
        }
        assert_eq!(run.observation(), before);
        previous = Some(h.key());
        match report {
            AdvanceReport::Complete { .. } => {
                assert_eq!(h.at, t(3_300_000));
                assert_eq!(h.counts.misses, 4);
                return;
            }
            AdvanceReport::Pending { cursor, reason, .. } => {
                if reason == PendingReason::EventBackpressure {
                    run.acknowledge(run.event_at(0).unwrap().key).unwrap();
                    assert_eq!(run.observation().ready_frame, Some(h));
                }
                report = run.continue_advance(cursor, WorkBudget::one()).unwrap();
            }
        }
    }
    panic!("four-object complete-frame return-point bound");
}
