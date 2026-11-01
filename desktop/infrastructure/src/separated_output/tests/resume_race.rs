use super::*;

#[test]
fn resume_then_pause_before_first_running_copy_keeps_the_explicit_seek_anchor() {
    for in_flight in [false, true] {
        let (pool, mut r, mut owner, mut cb) = setup(8192);
        start(&mut owner, &mut r);
        cb.render(&mut [0.0; 960]).unwrap();
        owner.pump(&mut r).unwrap();
        owner.seek(&mut r, 100).unwrap();
        cb.render(&mut [0.0; 16]).unwrap();
        owner.pump(&mut r).unwrap();
        owner.refill().unwrap();
        if in_flight {
            cb = std::thread::scope(|scope| {
                let (tx, rx) = sync_channel(0);
                let (go, wait) = sync_channel(0);
                let worker = scope.spawn(move || {
                    cb.render_inner(&mut [0.0; 16], |stage| {
                        if matches!(stage, Checkpoint::BeforeCopy) {
                            tx.send(()).unwrap();
                            wait.recv_timeout(GUARD).unwrap();
                        }
                    })
                    .unwrap();
                    cb
                });
                rx.recv_timeout(GUARD).unwrap();
                owner.resume(&mut r).unwrap();
                owner.pause(&mut r).unwrap();
                go.send(()).unwrap();
                worker.join().unwrap()
            });
        } else {
            owner.resume(&mut r).unwrap();
            owner.pause(&mut r).unwrap();
            cb.render(&mut [0.0; 16]).unwrap();
        }
        assert!(owner.pump(&mut r).unwrap().freeze_completed);
        assert_eq!(owner.position().copy_clock.next_background, 100);
        assert_eq!(r.scene_frame(), 100);
        owner.refill().unwrap();
        owner.resume(&mut r).unwrap();
        let mut out = [0.0; 16];
        cb.render(&mut out).unwrap();
        assert_background(&out, 100);
        owner.finish(&mut r, cb.stop()).unwrap();
        assert_eq!(pool.usage().bytes, 0);
    }
}

#[test]
fn frozen_callback_finishing_after_resume_cannot_restore_pre_seek_scene_position() {
    let (pool, mut r, mut owner, mut cb) = setup(8192);
    start(&mut owner, &mut r);
    cb.render(&mut [0.0; 960]).unwrap();
    owner.pump(&mut r).unwrap();
    owner.seek(&mut r, 100).unwrap();
    cb.render(&mut [0.0; 16]).unwrap();
    owner.pump(&mut r).unwrap();
    owner.refill().unwrap();
    cb = std::thread::scope(|scope| {
        let (tx, rx) = sync_channel(0);
        let (go, wait) = sync_channel(0);
        let worker = scope.spawn(move || {
            let mut out = [0.0; 16];
            cb.render_inner(&mut out, |stage| {
                if matches!(stage, Checkpoint::BeforeCopy) {
                    tx.send(()).unwrap();
                    wait.recv_timeout(GUARD).unwrap();
                }
            })
            .unwrap();
            assert_eq!(out, [0.0; 16]);
            cb
        });
        rx.recv_timeout(GUARD).unwrap();
        owner.resume(&mut r).unwrap();
        go.send(()).unwrap();
        worker.join().unwrap()
    });
    owner.pump(&mut r).unwrap();
    assert_eq!(r.scene_frame(), 100);
    let mut out = [0.0; 16];
    cb.render(&mut out).unwrap();
    assert_background(&out, 100);
    owner.pump(&mut r).unwrap();
    assert_eq!(r.scene_frame(), 108);
    owner.finish(&mut r, cb.stop()).unwrap();
    assert_eq!(pool.usage().bytes, 0);
}

#[test]
fn abandoned_consumer_reclaims_source_ring_scratch_and_runtime_on_owner() {
    let (pool, mut r, mut owner, cb) = setup(8192);
    add_sound(&mut r, &pool, origin(), 0.25);
    start(&mut owner, &mut r);
    trigger(&mut r, Channel::Scene);
    owner.pump(&mut r).unwrap();
    assert_eq!(owner.reclaim_abandoned(&mut r), Err(OutputError::Busy));
    drop(cb);
    assert!(matches!(owner.pump(&mut r), Err(OutputError::Closed)));
    owner.reclaim_abandoned(&mut r).unwrap();
    assert_eq!(r.voices().count(), 0);
    assert_eq!(pool.usage().bytes, 0);
}

#[test]
fn normal_voice_return_is_not_visible_before_composed_buffer_copy() {
    let (pool, mut r, mut owner, mut cb) = setup(8192);
    add_sound(&mut r, &pool, origin(), 0.25);
    start(&mut owner, &mut r);
    trigger(&mut r, Channel::Audition);
    owner.pump(&mut r).unwrap();
    cb = std::thread::scope(|scope| {
        let (tx, rx) = sync_channel(0);
        let (go, wait) = sync_channel(0);
        let worker = scope.spawn(move || {
            let mut out = [0.0; 16];
            cb.render_inner(&mut out, |stage| {
                if matches!(stage, Checkpoint::BeforeCopy) {
                    tx.send(()).unwrap();
                    wait.recv_timeout(GUARD).unwrap();
                }
            })
            .unwrap();
            assert_eq!(out[0], 0.25);
            cb
        });
        rx.recv_timeout(GUARD).unwrap();
        assert_eq!(owner.pump(&mut r).unwrap().voices.retired, 0);
        assert_eq!(r.voices().count(), 1);
        go.send(()).unwrap();
        worker.join().unwrap()
    });
    assert_eq!(owner.pump(&mut r).unwrap().voices.retired, 1);
    assert_eq!(r.voices().count(), 0);
    owner.finish(&mut r, cb.stop()).unwrap();
    assert_eq!(pool.usage().bytes, 0);
}

#[test]
fn three_hundred_pause_seek_resume_cycles_keep_pcm_budget_and_old_frame_cleanup_stable() {
    let (pool, mut r, mut owner, mut cb) = setup(8192);
    start(&mut owner, &mut r);
    let cost = pool.usage().bytes;
    for cycle in 0..300 {
        let target = (cycle * 137) % 6000;
        owner.pause(&mut r).unwrap();
        cb.render(&mut [0.0; 16]).unwrap();
        owner.pump(&mut r).unwrap();
        owner.seek(&mut r, target).unwrap();
        cb.render(&mut [0.0; 16]).unwrap();
        owner.pump(&mut r).unwrap();
        owner.refill().unwrap();
        owner.resume(&mut r).unwrap();
        let mut out = [0.0; 64];
        let report = cb.render(&mut out).unwrap();
        assert_background(&out, target);
        assert_eq!(report.missing_frames, 0);
        owner.pump(&mut r).unwrap();
        assert_eq!(r.scene_frame(), target + 32);
        assert_eq!(pool.usage().bytes, cost);
    }
    assert_eq!(pool.usage().peak_bytes, cost);
    owner.finish(&mut r, cb.stop()).unwrap();
    assert_eq!(pool.usage().bytes, 0);
    println!(
        "BACKGROUND_SEEK_PROOF cycles=300 missing_frames=0 pcm_peak_bytes={cost} pcm_final_bytes=0"
    );
}

#[test]
fn runtime_shutdown_before_copy_suppresses_both_lanes_background_and_first_receipts() {
    let (pool, mut r, mut owner, mut cb) = setup(8192);
    add_sound(&mut r, &pool, origin(), 0.25);
    start(&mut owner, &mut r);
    trigger(&mut r, Channel::Audition);
    trigger(&mut r, Channel::Scene);
    owner.pump(&mut r).unwrap();
    cb = std::thread::scope(|scope| {
        let (tx, rx) = sync_channel(0);
        let (go, wait) = sync_channel(0);
        let worker = scope.spawn(move || {
            let mut out = [1.0; 16];
            let report = cb
                .render_inner(&mut out, |stage| {
                    if matches!(stage, Checkpoint::BeforeCopy) {
                        tx.send(()).unwrap();
                        wait.recv_timeout(GUARD).unwrap();
                    }
                })
                .unwrap();
            assert_eq!(out, [0.0; 16]);
            assert!(report.voices.first.iter().all(Option::is_none));
            assert_eq!(report.voices.invalidated_lanes, [true; 2]);
            assert_eq!(report.background_frames, 0);
            assert_eq!(report.missing_frames, 0);
            cb
        });
        rx.recv_timeout(GUARD).unwrap();
        r.shutdown();
        assert!(matches!(owner.pump(&mut r), Err(OutputError::Closed)));
        go.send(()).unwrap();
        worker.join().unwrap()
    });
    owner.finish(&mut r, cb.stop()).unwrap();
    assert_eq!(pool.usage().bytes, 0);
}
