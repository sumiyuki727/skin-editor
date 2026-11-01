use super::*;
#[test]
fn initial_copy_anchor_is_visible_before_any_callback() {
    let pool = PcmBudget::default();
    let mut r = runtime(&pool);
    let mut track = reference(&pool, 8192, 48000, 2);
    track.restore_position(100).unwrap();
    let (mut owner, cb) = boundary(&mut r, track, origin()).unwrap();
    assert_eq!(owner.control().background_next(), 100);
    owner.pump(&mut r).unwrap();
    assert_eq!(r.scene_frame(), 100);
    owner.finish(&mut r, cb.stop()).unwrap();
    assert_eq!(pool.usage().bytes, 0);
}
#[test]
fn pending_pause_skips_dequeue_until_owner_freezes_and_acknowledges_it() {
    let (pool, mut r, mut owner, mut cb) = setup(8192);
    start(&mut owner, &mut r);
    cb.render(&mut [0.0; 64]).unwrap();
    owner.pump(&mut r).unwrap();
    let control = owner.control();
    let mark = control.invalidate(true, false, true).unwrap();
    for _ in 0..3 {
        let mut out = [1.0; 64];
        let report = cb.render(&mut out).unwrap();
        assert_eq!(out, [0.0; 64]);
        assert_eq!(report.clock.next_background, 32);
        assert_eq!(report.missing_frames, 0);
        owner.refill().unwrap();
        assert!(!control.background_acknowledged(mark));
    }
    owner.pause(&mut r).unwrap();
    owner.apply_mark(mark);
    cb.render(&mut [0.0; 64]).unwrap();
    owner.pump(&mut r).unwrap();
    assert!(control.background_acknowledged(mark));
    assert_eq!(owner.position().timeline_frame, 32);
    owner.finish(&mut r, cb.stop()).unwrap();
    assert_eq!(pool.usage().bytes, 0);
}
#[test]
fn applying_new_fence_during_copy_cannot_relabel_old_mixed_samples() {
    let (pool, mut r, mut owner, mut cb) = setup(8192);
    add_sound(&mut r, &pool, origin(), 0.125);
    start(&mut owner, &mut r);
    trigger(&mut r, Channel::Audition);
    owner.pump(&mut r).unwrap();
    cb = std::thread::scope(|scope| {
        let (tx, rx) = sync_channel(0);
        let (go, wait) = sync_channel(0);
        let worker = scope.spawn(move || {
            let mut out = [0.0; 4];
            let report = cb
                .render_inner(&mut out, |stage| {
                    if matches!(stage, Checkpoint::BeforeCopy) {
                        tx.send(()).unwrap();
                        wait.recv().unwrap();
                    }
                })
                .unwrap();
            assert_background(&out, 0);
            assert!(report.voices.runtime_first.iter().all(Option::is_none));
            cb
        });
        rx.recv_timeout(GUARD).unwrap();
        let mark = owner.control().invalidate(false, true, false).unwrap();
        owner.apply_mark(mark);
        go.send(()).unwrap();
        worker.join().unwrap()
    });
    owner.finish(&mut r, cb.stop()).unwrap();
    assert_eq!(pool.usage().bytes, 0);
}
#[test]
fn out_of_band_close_suppresses_pending_copy_and_control_handle_keeps_no_pcm() {
    let (pool, mut r, mut owner, mut cb) = setup(8192);
    start(&mut owner, &mut r);
    let control = owner.control();
    let mut out = [1.0; 4];
    let report = cb
        .render_inner(&mut out, |stage| {
            if matches!(stage, Checkpoint::BeforeCopy) {
                control.close();
            }
        })
        .unwrap();
    assert_eq!(out, [0.0; 4]);
    assert_eq!(report.background_frames, 0);
    assert!(!report.clock.running);
    // The owner may be descheduled after an out-of-band close. Repeated
    // callbacks must stay silent, without draining and re-reading old positions.
    cb.render(&mut out).unwrap();
    assert_eq!(out, [0.0; 4]);
    owner.finish(&mut r, cb.stop()).unwrap();
    assert!(control.is_closed());
    assert_eq!(pool.usage().bytes, 0);
}
