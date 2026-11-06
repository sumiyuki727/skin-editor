mod scale_support;
use scale_support::*;
use skin_editor_gameplay::*;

#[test]
fn cancelled_large_advances_resume_the_exact_committed_prefix_with_no_duplicate_events() {
    let data = circles();
    for budget in [WorkBudget::one(), WorkBudget::full()] {
        for index in [0, 1, 31, 32, 4999, 9999] {
            let mut play = Playback::new(&data, 300_200 + index as u128, budget);
            play.automatic();
            play.auto_range(0, index);
            let last = (index + 32).min(N) - 1;
            let through = target(last) + 1;
            play.prepare(through);
            play.seal(through);
            let s = play.source();
            let mut report = play
                .owner
                .advance(s, t(through), WorkBudget::one())
                .unwrap();
            let mut committed = None;
            for _ in 0..64 {
                let AdvanceReport::Pending { cursor, .. } = report else {
                    panic!("one-unit prefix checkpoint");
                };
                if play
                    .owner
                    .current()
                    .unwrap()
                    .committed_judgement(s, index)
                    .unwrap()
                    .is_some()
                {
                    committed = Some(cursor);
                    break;
                }
                report = play
                    .owner
                    .continue_advance(cursor, WorkBudget::one())
                    .unwrap();
            }
            let cursor = committed.unwrap();
            let before = play.owner.current().unwrap().observation();
            let r = play.owner.resources();
            let a = play.owner.allocations();
            play.owner.cancel_advance(cursor.request()).unwrap();
            let counts = play.owner.current().unwrap().work_counts();
            assert_eq!(
                play.owner.continue_advance(cursor, budget),
                Err(PreparationError::Runtime(RunError::StaleCursor))
            );
            assert_eq!(play.owner.current().unwrap().work_counts(), counts);
            let after = play.owner.current().unwrap().observation();
            assert_eq!(after.ready_frame, before.ready_frame);
            assert_eq!(after.committed_counts, before.committed_counts);
            assert_eq!(play.owner.resources(), r);
            assert_eq!(play.owner.allocations(), a);
            play.advance(through);
            play.tail(last + 1);
            assert_eq!(
                play.verify_end(StatisticsOrigin::FromStart { start: t(START) })
                    .1,
                20_000
            );
        }
    }
}

#[test]
fn automatic_input_backpressure_exposes_only_the_safe_prefix_and_freeze_cancels_future_edges() {
    let data = circles();
    let mut receipts = [None; 2];
    for (case, budget) in [WorkBudget::one(), WorkBudget::full()]
        .into_iter()
        .enumerate()
    {
        let mut play = Playback::new(&data, 300_100 + case as u128, budget);
        play.automatic();
        play.oracle.miss_range = 1..32;
        let old = play.source();
        let mut blocked = false;
        for _ in 0..128 {
            let report = play
                .owner
                .prepare_automatic_inputs(old, t(END), budget)
                .unwrap();
            play.sample();
            if let AutomaticReport::Pending {
                reason: AutomaticPendingReason::InputBackpressure,
                ready_through,
                ..
            } = report
            {
                assert_eq!(ready_through, Some(t(target(31) + 1)));
                blocked = true;
                break;
            }
        }
        assert!(blocked);
        let before = play.owner.current().unwrap().observation();
        assert_eq!((before.pending_inputs, before.pending_events), (64, 0));
        assert_eq!(
            play.owner.seal_through(old, t(END)),
            Err(PreparationError::Runtime(RunError::AutomaticInputsPending))
        );
        for _ in 0..3 {
            assert!(matches!(
                play.owner.prepare_automatic_inputs(old, t(END), budget),
                Ok(AutomaticReport::Pending {
                    reason: AutomaticPendingReason::InputBackpressure,
                    admitted: 0,
                    ..
                })
            ));
            assert_eq!(play.owner.current().unwrap().observation(), before);
        }
        play.freeze(0);
        assert_eq!(
            play.owner
                .current()
                .unwrap()
                .observation()
                .freeze
                .unwrap()
                .cancelled_inputs,
            63
        );
        play.owner.resume(old).unwrap();
        play.drain();
        play.prepare(END);
        play.advance(END);
        receipts[case] = Some(play.verify_end(StatisticsOrigin::FromStart { start: t(START) }));
        assert_eq!(play.oracle.miss_count, 31);
        assert_eq!(play.oracle.hits, N - 31);
    }
    assert_eq!(receipts[0], receipts[1]);
}

#[test]
fn automatic_freeze_at_planning_admission_and_committed_prefixes_never_replays_old_work() {
    let data = circles();
    for budget in [WorkBudget::one(), WorkBudget::full()] {
        for index in [0, 1, 31, 32, 4999, 9999] {
            for cut in 0..4 {
                let mut play = Playback::new(&data, 301_000 + index as u128 * 4 + cut, budget);
                play.automatic();
                if cut < 2 {
                    play.oracle.misses[0] = Some(index);
                }
                play.auto_range(0, index);
                let old = play.source();
                let at = target(index);
                if cut > 0 {
                    let wanted = if cut == 1 { 0 } else { 2 };
                    let mut reached = false;
                    for _ in 0..8 {
                        play.owner
                            .prepare_automatic_inputs(old, t(at), WorkBudget::one())
                            .unwrap();
                        let run = play.owner.current().unwrap();
                        if run.work_counts().automatic_objects > index as u64
                            && run.observation().pending_inputs == wanted
                        {
                            reached = true;
                            break;
                        }
                    }
                    assert!(reached, "planning/admission checkpoint {index}/{cut}");
                }
                let mut old_cursor = None;
                if cut == 3 {
                    play.seal(at);
                    let mut report = play.owner.advance(old, t(at), WorkBudget::one()).unwrap();
                    let mut committed = false;
                    for _ in 0..64 {
                        let AdvanceReport::Pending { cursor, .. } = report else {
                            panic!("one-unit committed checkpoint");
                        };
                        old_cursor = Some(cursor);
                        if play
                            .owner
                            .current()
                            .unwrap()
                            .committed_judgement(old, index)
                            .unwrap()
                            .is_some()
                        {
                            committed = true;
                            break;
                        }
                        report = play
                            .owner
                            .continue_advance(cursor, WorkBudget::one())
                            .unwrap();
                    }
                    assert!(committed);
                }
                play.freeze(at);
                let frozen = play.owner.current().unwrap().observation();
                assert_eq!(
                    frozen.freeze.unwrap().cancelled_inputs,
                    usize::from(cut >= 2)
                );
                let header = frozen.ready_frame.unwrap();
                let before = play.owner.current().unwrap().work_counts();
                assert_eq!(
                    play.owner.prepare_automatic_inputs(old, t(END), budget),
                    Err(PreparationError::Runtime(RunError::GateClosed))
                );
                assert_eq!(play.owner.current().unwrap().work_counts(), before);
                if let Some(cursor) = old_cursor {
                    assert_eq!(
                        play.owner.continue_advance(cursor, budget),
                        Err(PreparationError::Runtime(RunError::StaleCursor))
                    );
                }
                let resumed = play.owner.resume(old).unwrap();
                assert_eq!(resumed.input, old.input.checked_next().unwrap());
                assert_eq!(
                    play.owner
                        .current()
                        .unwrap()
                        .observation()
                        .completed_through,
                    Some(t(at))
                );
                let before = play.owner.current().unwrap().observation();
                assert_eq!(
                    play.owner.prepare_automatic_inputs(old, t(END), budget),
                    Err(PreparationError::Runtime(RunError::StaleSource))
                );
                assert_eq!(
                    play.owner.observe_driver_release(
                        old,
                        InputDriver::Automatic,
                        fact(1, PhysicalSlot::PrimaryOne)
                    ),
                    Err(PreparationError::Runtime(RunError::StaleSource))
                );
                assert_eq!(play.owner.current().unwrap().observation(), before);
                let mut page = [None; 1];
                assert_eq!(
                    play.owner.current().unwrap().read_frame_page(
                        header.key().cursor(index),
                        1,
                        &mut page
                    ),
                    Err(FrameReadError::StaleSource)
                );
                play.drain();
                play.tail(index);
                let receipt = play.verify_end(StatisticsOrigin::FromStart { start: t(START) });
                assert_eq!(receipt.0.misses, usize::from(cut < 2));
                assert_eq!(receipt.0.hits, N - usize::from(cut < 2));
            }
        }
    }
}

#[test]
fn full_event_queue_and_one_free_slot_cannot_half_commit_a_hit_or_block_freezing() {
    let data = circles();
    for budget in [WorkBudget::one(), WorkBudget::full()] {
        let mut play = Playback::new(&data, 302_000, budget);
        play.automatic();
        play.prepare(target(31) + 1);
        play.seal(target(31) + 1);
        let old = play.source();
        let mut report = play.owner.advance(old, t(target(31) + 1), budget).unwrap();
        let mut complete = false;
        for _ in 0..BOUND {
            play.sample();
            match report {
                AdvanceReport::Complete { .. } => {
                    complete = true;
                    break;
                }
                AdvanceReport::Pending {
                    cursor,
                    reason,
                    steps,
                    ..
                } => {
                    assert_eq!(reason, PendingReason::WorkBudget);
                    assert!(steps > 0);
                    report = play.owner.continue_advance(cursor, budget).unwrap();
                }
            }
        }
        assert!(complete);
        assert_eq!(
            play.owner.current().unwrap().observation().pending_events,
            64
        );
        play.prepare(target(32) + 1);
        play.seal(target(32) + 1);
        let mut report = play.owner.advance(old, t(target(32) + 1), budget).unwrap();
        let mut blocked = None;
        for _ in 0..128 {
            let AdvanceReport::Pending { cursor, reason, .. } = report else {
                panic!("full event queue must block");
            };
            if reason == PendingReason::EventBackpressure {
                blocked = Some(cursor);
                break;
            }
            report = play.owner.continue_advance(cursor, budget).unwrap();
        }
        let blocked = blocked.unwrap();
        let run = play.owner.current().unwrap();
        assert_eq!(run.committed_judgement(old, 32), Ok(None));
        assert_eq!(run.observation().committed_counts.hits, 32);
        let old_sound = run.event_at(63).unwrap().key;
        assert!(run.sound_intent_allowed(old_sound).unwrap());
        play.owner.request_freeze(old, t(target(32))).unwrap();
        assert_eq!(
            play.owner.continue_advance(blocked, budget),
            Err(PreparationError::Runtime(RunError::StaleCursor))
        );
        for free in 0..2 {
            let mut blocked = false;
            for _ in 0..128 {
                match play.owner.settle_freeze(old, budget).unwrap() {
                    FreezeReport::Pending {
                        reason: PendingReason::EventBackpressure,
                        ..
                    } => {
                        blocked = true;
                        break;
                    }
                    FreezeReport::Pending {
                        reason: PendingReason::WorkBudget,
                        steps,
                        ..
                    } => assert!(steps > 0 && steps <= budget.get()),
                    result => panic!("must remain blocked with {free} free slots: {result:?}"),
                }
            }
            assert!(blocked);
            assert_eq!(
                play.owner.current().unwrap().committed_judgement(old, 32),
                Ok(None)
            );
            assert_eq!(
                play.owner.current().unwrap().observation().pending_events,
                64 - free
            );
            let event = play.owner.current().unwrap().event_at(0).unwrap();
            play.oracle.accept(event);
            play.owner.acknowledge(event.key).unwrap();
        }
        play.freeze(target(32));
        assert_eq!(
            play.owner
                .current()
                .unwrap()
                .observation()
                .committed_counts
                .hits,
            33
        );
        assert_eq!(
            play.owner
                .current()
                .unwrap()
                .observation()
                .freeze
                .unwrap()
                .cancelled_inputs,
            1
        );
        assert!(
            !play
                .owner
                .current()
                .unwrap()
                .sound_intent_allowed(old_sound)
                .unwrap()
        );
        play.owner.resume(old).unwrap();
        assert!(
            (0..64)
                .filter_map(|i| play.owner.current().unwrap().event_at(i))
                .any(|event| event.key == old_sound),
            "eligibility must be tested on a retained, unacknowledged sound"
        );
        assert!(
            !play
                .owner
                .current()
                .unwrap()
                .sound_intent_allowed(old_sound)
                .unwrap()
        );
        play.drain();
        play.tail(33);
        assert_eq!(
            play.verify_end(StatisticsOrigin::FromStart { start: t(START) })
                .0
                .hits,
            N
        );
    }
}

#[test]
fn all_four_slots_require_current_source_and_driver_release_after_large_prefix_resume() {
    let data = circles();
    for budget in [WorkBudget::one(), WorkBudget::full()] {
        let mut play = Playback::new(&data, 303_000, budget);
        play.external_range(0, 5000);
        let old = play.source();
        play.freeze(target(5000));
        let old_page = play
            .owner
            .current()
            .unwrap()
            .observation()
            .ready_frame
            .unwrap();
        let selected = play
            .owner
            .select_driver(old, InputDriver::Regression)
            .unwrap();
        let active = play.owner.resume(selected).unwrap();
        assert_eq!(active.transport, old.transport);
        assert_eq!(active.input.get(), old.input.get() + 2);
        for (i, slot) in [
            PhysicalSlot::PrimaryOne,
            PhysicalSlot::PrimaryTwo,
            PhysicalSlot::SecondaryOne,
            PhysicalSlot::SecondaryTwo,
        ]
        .into_iter()
        .enumerate()
        {
            let before = play.owner.current().unwrap().observation();
            let event = edge(
                99,
                target(5000) + 1,
                slot,
                InputEdge::Press(Point::new(0, 0).unwrap()),
            );
            assert_eq!(
                play.owner
                    .submit_driver_inputs(active, InputDriver::Regression, &[event]),
                Err(PreparationError::Runtime(RunError::RearmRequired))
            );
            assert_eq!(
                play.owner.observe_driver_release(
                    old,
                    InputDriver::Manual,
                    fact(i as u64 + 1, slot)
                ),
                Err(PreparationError::Runtime(RunError::StaleSource))
            );
            assert_eq!(
                play.owner.observe_release(active, fact(i as u64 + 1, slot)),
                Err(PreparationError::Runtime(RunError::DriverMismatch))
            );
            assert_eq!(play.owner.current().unwrap().observation(), before);
            play.owner
                .observe_driver_release(active, InputDriver::Regression, fact(i as u64 + 1, slot))
                .unwrap();
            assert!(play.owner.current().unwrap().observation().armed_slots[i]);
        }
        assert_eq!(
            play.owner.current().unwrap().observation().armed_slots,
            [true; 4]
        );
        let mut batch = [press(5, target(5000) + 1); 8];
        for (i, slot) in [
            PhysicalSlot::PrimaryOne,
            PhysicalSlot::PrimaryTwo,
            PhysicalSlot::SecondaryOne,
            PhysicalSlot::SecondaryTwo,
        ]
        .into_iter()
        .enumerate()
        {
            let at = target(5000 + i) + i64::from(i == 0);
            batch[2 * i] = edge(
                5 + 2 * i as u64,
                at,
                slot,
                InputEdge::Press(Point::new(0, 0).unwrap()),
            );
            batch[2 * i + 1] = edge(6 + 2 * i as u64, at + 1, slot, InputEdge::Release);
        }
        play.oracle.hit_override = Some((5000, target(5000) + 1));
        play.owner
            .submit_driver_inputs(active, InputDriver::Regression, &batch)
            .unwrap();
        play.advance(target(5003) + 1);
        assert_eq!(
            play.owner
                .current()
                .unwrap()
                .committed_judgement(active, 5000)
                .unwrap()
                .unwrap()
                .at,
            t(target(5000) + 1)
        );
        let mut page = [None; 1];
        assert_eq!(
            play.owner
                .current()
                .unwrap()
                .read_frame_page(old_page.key().cursor(0), 1, &mut page),
            Err(FrameReadError::StaleSource)
        );
        play.tail(5004);
        assert_eq!(
            play.verify_end(StatisticsOrigin::FromStart { start: t(START) })
                .0
                .hits,
            N
        );
    }
}

#[test]
fn inspection_targets_keep_exact_skip_provenance_and_resume_only_new_input_at_scale() {
    let data = circles();
    for budget in [WorkBudget::one(), WorkBudget::full()] {
        for driver in [InputDriver::Manual, InputDriver::Automatic] {
            for at in [-1, 0, 5_000_000_000, 9_999_100_001, i64::MIN] {
                let mut play = Playback::new(&data, 304_000, budget);
                if driver == InputDriver::Automatic {
                    play.automatic();
                }
                play.freeze(START);
                let old = play.source();
                let old_frame = play
                    .owner
                    .current()
                    .unwrap()
                    .observation()
                    .ready_frame
                    .unwrap();
                let mut task = play.owner.request_inspection(old, t(at)).unwrap();
                finish(&mut task, budget);
                assert_eq!(
                    play.owner.current().unwrap().observation().ready_frame,
                    Some(old_frame)
                );
                let next = play.owner.activate(task.key(), &mut task).unwrap();
                drop(task);
                play.sample();
                let skipped = (0..N).take_while(|i| target(*i) + 100_000 < at).count();
                let run = play.owner.current().unwrap();
                assert_eq!(run.driver(), driver);
                let h = run.ready_frame(next).unwrap().unwrap().header();
                assert_eq!(h.at, t(at));
                assert_eq!(
                    h.counts,
                    FrameCounts {
                        inspection_skipped: skipped,
                        ..FrameCounts::default()
                    }
                );
                assert_eq!(
                    h.statistics_origin,
                    StatisticsOrigin::Inspection { at: t(at) }
                );
                assert_eq!(
                    (
                        run.observation().pending_events,
                        run.observation().pending_inputs
                    ),
                    (0, 0)
                );
                assert_eq!(next.transport, old.transport.checked_next().unwrap());
                assert_eq!(next.input, old.input.checked_next().unwrap());
                let mut output = [None; 64];
                for first in (0..N).step_by(64) {
                    let page = run
                        .read_frame_page(h.key().cursor(first), 64, &mut output)
                        .unwrap()
                        .unwrap();
                    for (offset, value) in output.iter().take(page.written).enumerate() {
                        let i = first + offset;
                        let skipped = i < skipped;
                        let phase = if at < target(i) - 200_000 {
                            Phase::Future
                        } else if at >= target(i) + 300_000 || skipped {
                            Phase::Retired
                        } else {
                            Phase::VisibleUnjudged
                        };
                        assert_eq!(
                            *value,
                            Some(CircleFrame {
                                object: ObjectId::new(i as u32),
                                phase,
                                judgement: None,
                                unavailable: skipped
                                    .then_some(UnavailableReason::InspectionSkipped)
                            })
                        );
                        assert_eq!(run.committed_judgement(next, i), Ok(None));
                    }
                }
                let resumed = play.owner.resume(next).unwrap();
                play.oracle = Oracle::new(resumed, skipped);
                assert_eq!(
                    play.owner.submit_driver_inputs(old, driver, &[press(1, 0)]),
                    Err(PreparationError::Runtime(RunError::StaleSource))
                );
                assert_eq!(
                    play.owner.current().unwrap().read_frame_page(
                        h.key().cursor(0),
                        1,
                        &mut output
                    ),
                    Err(FrameReadError::StaleSource)
                );
                if driver == InputDriver::Manual {
                    if skipped < N {
                        let when = target(skipped).max(at.checked_add(1).unwrap());
                        let event = press(2, when);
                        assert_eq!(
                            play.owner.submit_inputs(resumed, &[event]),
                            Err(PreparationError::Runtime(RunError::RearmRequired))
                        );
                        play.owner
                            .observe_release(resumed, fact(1, PhysicalSlot::PrimaryOne))
                            .unwrap();
                        if skipped == 0 {
                            play.input_sequence_offset = 1;
                        }
                        play.oracle.first_hit_offset = when - target(skipped);
                    }
                } else if skipped < N && target(skipped) <= at {
                    play.oracle.misses[0] = Some(skipped);
                }
                play.tail(skipped);
                let result = play.verify_end(StatisticsOrigin::Inspection { at: t(at) });
                assert_eq!(result.0.inspection_skipped, skipped);
                assert_eq!(
                    result.0.misses,
                    usize::from(
                        driver == InputDriver::Automatic && skipped < N && target(skipped) <= at
                    )
                );
                if at == 5_000_000_000 && driver == InputDriver::Automatic {
                    assert_eq!(result.1, 9999);
                }
            }
        }
    }
}

#[test]
fn trusted_current_release_cannot_reopen_the_inspection_target_or_leak_queued_input_on_failure() {
    let data = circles();
    for budget in [WorkBudget::one(), WorkBudget::full()] {
        for at in [-1, 0, 5_000_000_000, 9_999_100_001, i64::MIN] {
            let mut play = Playback::new(&data, 308_001, budget);
            play.freeze(START);
            let mut task = play.owner.request_inspection(play.source(), t(at)).unwrap();
            finish(&mut task, budget);
            let prepared = play.owner.activate(task.key(), &mut task).unwrap();
            drop(task);
            let current = play.owner.resume(prepared).unwrap();
            let header = play
                .owner
                .current()
                .unwrap()
                .observation()
                .ready_frame
                .unwrap();
            play.owner
                .observe_release(current, fact(1, PhysicalSlot::PrimaryOne))
                .unwrap();
            let after = at.checked_add(1).unwrap();
            play.owner
                .submit_inputs(
                    current,
                    &[
                        press(2, after),
                        edge(
                            3,
                            after.checked_add(1).unwrap(),
                            PhysicalSlot::PrimaryOne,
                            InputEdge::Release,
                        ),
                    ],
                )
                .unwrap();
            assert_eq!(
                play.owner.current().unwrap().observation().pending_inputs,
                2
            );
            let r = play.owner.resources();
            let a = play.owner.allocations();
            assert_eq!(
                play.owner.submit_inputs(current, &[press(4, at)]),
                Err(PreparationError::Runtime(RunError::LateInput))
            );
            let failed = play.owner.current().unwrap().observation();
            assert_eq!(
                failed.status,
                RunStatus::Failed(Failure {
                    cause: RunError::LateInput,
                    cancelled_inputs: 2
                })
            );
            assert_eq!((failed.pending_inputs, failed.pending_events), (0, 0));
            assert_eq!(failed.armed_slots, [false; 4]);
            assert_eq!(failed.ready_frame, Some(header));
            assert_eq!(failed.committed_counts, header.counts);
            assert_eq!(failed.sealed_through, Some(t(at)));
            assert_eq!(play.owner.resources(), r);
            assert_eq!(play.owner.allocations(), a);
            let counts = play.owner.current().unwrap().work_counts();
            assert_eq!(
                play.owner.advance(current, t(after), budget),
                Err(PreparationError::Runtime(RunError::Faulted))
            );
            assert_eq!(
                play.owner.resume(current),
                Err(PreparationError::Runtime(RunError::Faulted))
            );
            assert_eq!(
                play.owner.select_driver(current, InputDriver::Automatic),
                Err(PreparationError::Runtime(RunError::Faulted))
            );
            assert_eq!(play.owner.current().unwrap().work_counts(), counts);
            let run = play.owner.current().unwrap();
            let mut page = [None; 1];
            run.read_frame_page(header.key().cursor(0), 1, &mut page)
                .unwrap()
                .unwrap();
            assert_eq!(page[0].unwrap().judgement, None);
            assert_eq!(run.committed_judgement(current, 0), Ok(None));
            assert_eq!(run.observation(), failed);
        }
    }
}
