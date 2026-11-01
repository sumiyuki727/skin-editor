use super::*;
use crate::separated_output::{self, BackgroundOwner};
use skin_editor_application::values::{AudioComponentId, AudioMetadata, BlobRef, Gain};
use skin_editor_audio::{BackgroundTrack, Origin, PcmBudget, PreparedSound, Runtime, SoundPlan};
#[test]
fn missing_return_keeps_receiver_and_endpoint_until_the_controlled_owner_exits() {
    let (pool, mut r, mut owner, mut cb, mut native) = setup();
    trigger(&mut r, Channel::Audition);
    owner.pump(&mut r).unwrap();
    cb.render(&mut [0.0; 2], 100);
    let used = pool.usage().bytes;
    let anchor = cb.endpoint.as_ref().unwrap().copy_clock();
    for _ in 0..3 {
        assert!(matches!(
            native.try_stop(),
            Err(EndpointReturnError::Pending)
        ));
        cb.render(&mut [1.0; 2], 200);
        assert_eq!(cb.endpoint.as_ref().unwrap().copy_clock(), anchor);
        assert_eq!(pool.usage().bytes, used);
        assert_eq!(native.stats().endpoint_returns, 0);
    }
    drop(cb);
    assert_eq!(pool.usage().bytes, used);
    let (stopped, stats) = native.try_stop().unwrap();
    assert_eq!(stats.endpoint_returns, 1);
    assert_eq!(stats.endpoint_returns_outside_render, 1);
    assert_eq!(stats.callback_resource_releases, 0);
    assert!(matches!(
        native.try_stop(),
        Err(EndpointReturnError::AlreadyReceived)
    ));
    owner.finish(&mut r, stopped).unwrap();
    assert_eq!(pool.usage().bytes, 0);
}
#[test]
fn disconnected_return_does_not_release_the_live_callback_or_a_failed_delivery() {
    let (pool, mut r, mut owner, mut cb, native) = setup();
    let endpoint = cb.endpoint.take().unwrap();
    drop(cb);
    drop(native);
    let (mut cb, mut native, gate) = executor_gated(endpoint, true);
    let mut gate = gate.unwrap();
    let used = pool.usage().bytes;
    gate.disconnect();
    for _ in 0..3 {
        assert!(matches!(
            native.try_stop(),
            Err(EndpointReturnError::Disconnected)
        ));
        let mut samples = [1.0; 32];
        cb.render(&mut samples, 100);
        assert_eq!(samples, [0.0; 32]);
        assert_eq!(pool.usage().bytes, used);
    }
    drop(cb);
    assert!(gate.deliver().is_err());
    assert!(gate.deliver().is_err());
    assert_eq!(pool.usage().bytes, used);
    assert!(owner.reclaim_abandoned(&mut r).is_err());
    assert_eq!(native.stats().endpoint_returns, 0);
    assert_eq!(native.stats().callback_resource_releases, 0);
    drop(gate);
    owner.reclaim_abandoned(&mut r).unwrap();
    assert_eq!(pool.usage().bytes, 0);
}
#[test]
fn device_error_silences_the_next_copy_without_advancing_or_freeing_the_endpoint() {
    let (pool, mut r, mut owner, mut cb, native) = setup();
    trigger(&mut r, Channel::Scene);
    owner.pump(&mut r).unwrap();
    cb.render(&mut [0.0; 2], 100);
    let anchor = cb.endpoint.as_ref().unwrap().copy_clock();
    let used = pool.usage().bytes;
    native.fault_signal().report();
    let mut out = [1.0; 32];
    cb.render(&mut out, 200);
    assert_eq!(out, [0.0; 32]);
    assert_eq!(cb.endpoint.as_ref().unwrap().copy_clock(), anchor);
    assert_eq!(pool.usage().bytes, used);
    assert_eq!(native.stats().callback_resource_releases, 0);
    finish(pool, r, owner, cb, native);
}
#[test]
fn retired_backend_signal_cannot_fault_the_rebuilt_endpoint() {
    let (pool, r, owner, cb, native) = setup();
    let old = native.fault_signal();
    finish(pool, r, owner, cb, native);
    let (pool, mut r, mut owner, mut cb, native) = setup();
    old.report();
    assert!(!native.stats().device_fault);
    trigger(&mut r, Channel::Audition);
    owner.pump(&mut r).unwrap();
    let mut out = [0.0; 2];
    cb.render(&mut out, 100);
    assert!(out.iter().any(|v| *v != 0.0));
    finish(pool, r, owner, cb, native);
}
fn setup() -> (
    PcmBudget,
    Runtime,
    BackgroundOwner,
    Callback,
    NativeExecutor,
) {
    let pool = PcmBudget::default();
    let mut r = Runtime::with_budget(48000, 2, pool.clone()).unwrap();
    r.enable_voice_handoff().unwrap();
    let origin = Origin {
        session: 1,
        node: 1,
        revision: 1,
        navigation_revision: 1,
        draft: None,
    };
    let plan = SoundPlan {
        target: AudioComponentId::new(1).unwrap().into(),
        source: BlobRef::new([1; 32], 16),
        metadata: Some(AudioMetadata::new(48000, 2, 2).unwrap()),
        gain: Gain::UNITY,
        visible: true,
    };
    let job = r.synchronize(origin, &[plan]).unwrap()[0];
    r.complete(
        job,
        Ok(PreparedSound::from_source(plan, &[0.01; 4], &pool).unwrap()),
    )
    .unwrap();
    let pcm = pool.reserve(10000 * 2).unwrap().allocate();
    let track = BackgroundTrack::new(pcm, 48000, 2).unwrap();
    let (mut owner, endpoint) = separated_output::boundary(&mut r, track, origin).unwrap();
    owner.refill().unwrap();
    owner.resume(&mut r).unwrap();
    let (callback, native) = executor(endpoint);
    (pool, r, owner, callback, native)
}
fn trigger(r: &mut Runtime, lane: Channel) -> u64 {
    r.trigger(
        r.token(lane),
        AudioComponentId::new(1).unwrap(),
        r.mix_frame(),
    )
    .unwrap()
    .voice
    .unwrap()
}
fn finish(
    pool: PcmBudget,
    mut r: Runtime,
    mut owner: BackgroundOwner,
    cb: Callback,
    native: NativeExecutor,
) {
    drop(cb);
    let (stopped, stats) = native.stop().unwrap();
    assert_eq!(stats.endpoint_returns, 1);
    assert_eq!(stats.callback_resource_releases, 0);
    assert_eq!(stats.endpoint_returns_in_render, 0);
    assert_eq!(stats.endpoint_returns_outside_render, 1);
    owner.finish(&mut r, stopped).unwrap();
    assert_eq!(pool.usage().bytes, 0);
}
#[test]
fn unstarted_and_active_closures_return_the_endpoint_before_control_releases_pcm() {
    for active in [false, true] {
        let (pool, mut r, mut owner, mut cb, native) = setup();
        trigger(&mut r, Channel::Audition);
        owner.pump(&mut r).unwrap();
        if active {
            cb.render(&mut [0.0; 2], 100);
        }
        let before = pool.usage().bytes;
        drop(cb);
        assert_eq!(pool.usage().bytes, before); // Return carries scratch + Voice owners.
        let (stopped, stats) = native.stop().unwrap();
        assert_eq!(stats.endpoint_returns, 1);
        owner.finish(&mut r, stopped).unwrap();
        assert_eq!(pool.usage().bytes, 0);
    }
}
#[test]
fn abandoned_return_receiver_does_not_refund_live_endpoint_storage() {
    let (pool, mut r, mut owner, cb, native) = setup();
    assert!(native.play().is_err()); // Device-free executor has no OS stream.
    let before = pool.usage().bytes;
    drop(native);
    assert_eq!(pool.usage().bytes, before);
    drop(cb); // Closure destruction, never rendering.
    owner.reclaim_abandoned(&mut r).unwrap();
    assert_eq!(pool.usage().bytes, 0);
}
#[test]
fn thirty_two_exact_voice_observations_follow_buffer_copy_and_retire_on_owner() {
    let (pool, mut r, mut owner, mut cb, mut native) = setup();
    let ids: Vec<_> = (0..32)
        .map(|i| {
            trigger(
                &mut r,
                if i % 2 == 0 {
                    Channel::Scene
                } else {
                    Channel::Audition
                },
            )
        })
        .collect();
    assert!(
        r.trigger(
            r.token(Channel::Audition),
            AudioComponentId::new(1).unwrap(),
            r.mix_frame()
        )
        .is_err()
    );
    owner.pump(&mut r).unwrap();
    let mut out = [0.0; 4];
    cb.render(&mut out, 123);
    assert!(out.iter().all(|v| (*v - 0.32).abs() < 0.000001));
    let mut observed = Vec::new();
    while let Some(row) = native.next_first() {
        assert_eq!(
            (
                row.callback_nanos,
                row.callback_ordinal,
                row.offset_frames,
                row.callback_frames
            ),
            (123, 1, 0, 2)
        );
        observed.push(row.voice);
    }
    assert_eq!(observed, ids);
    assert_eq!(r.voices().count(), 32);
    owner.pump(&mut r).unwrap();
    assert_eq!(r.voices().count(), 0);
    assert_eq!(native.stats().observation_overflows, 0);
    finish(pool, r, owner, cb, native);
}
#[test]
fn cancel_after_handoff_never_reports_or_copies_the_cancelled_voice() {
    let (pool, mut r, mut owner, mut cb, mut native) = setup();
    trigger(&mut r, Channel::Audition);
    owner.pump(&mut r).unwrap();
    r.stop_audition(r.token(Channel::Audition)).unwrap();
    let mut out = [1.0; 4];
    cb.render(&mut out, 200);
    assert_eq!(out, [0.0; 4]);
    assert!(native.next_first().is_none());
    finish(pool, r, owner, cb, native);
}
#[test]
fn callback_shape_rejection_does_not_consume_the_background_or_voice() {
    let (pool, mut r, mut owner, mut cb, mut native) = setup();
    let id = trigger(&mut r, Channel::Audition);
    owner.pump(&mut r).unwrap();
    cb.render(&mut [1.0; 3], 100);
    assert_eq!(native.stats().render_errors, 1);
    assert!(native.next_first().is_none());
    cb.render(&mut [0.0; 4], 200);
    assert_eq!(native.next_first().unwrap().voice, id);
    owner.pump(&mut r).unwrap();
    assert_eq!(owner.position().timeline_frame, 2);
    finish(pool, r, owner, cb, native);
}
#[test]
fn oversized_callbacks_use_bounded_chunks_and_report_the_real_missing_reserve() {
    let (pool, r, owner, mut cb, native) = setup();
    let mut out = vec![1.0; 4096 * 2];
    cb.render(&mut out, 100);
    let stats = native.stats();
    assert_eq!(
        (
            stats.callbacks,
            stats.frames,
            stats.max_callback_frames,
            stats.missing_frames
        ),
        (1, 4096, 4096, 2048)
    );
    assert_eq!(stats.render_errors, 0);
    finish(pool, r, owner, cb, native);
}
#[test]
fn observation_overflow_is_counted_and_never_reinterpreted_as_no_voice() {
    let (pool, mut r, mut owner, mut cb, mut native) = setup();
    for _ in 0..129 {
        trigger(&mut r, Channel::Audition);
        owner.pump(&mut r).unwrap();
        cb.render(&mut [0.0; 4], 100);
        owner.pump(&mut r).unwrap();
        owner.refill().unwrap();
    }
    assert_eq!(native.stats().observation_overflows, 1);
    assert_eq!(std::iter::from_fn(|| native.next_first()).count(), 128);
    finish(pool, r, owner, cb, native);
}
#[test]
fn endpoint_return_from_another_executor_does_not_close_either_owner() {
    let (pool, mut r, mut owner, cb, native) = setup();
    let (pool2, r2, owner2, cb2, native2) = setup();
    drop(cb2);
    let (foreign, _) = native2.stop().unwrap();
    let foreign = owner.finish(&mut r, foreign).unwrap_err();
    assert!(!r.is_closed());
    let mut r2 = r2;
    let mut owner2 = owner2;
    owner2.finish(&mut r2, foreign).unwrap();
    assert_eq!(pool2.usage().bytes, 0);
    finish(pool, r, owner, cb, native);
}
#[test]
fn worker_destruction_returns_without_an_extra_render_or_sleep() {
    let (pool, r, owner, mut cb, native) = setup();
    let (ready, rx) = sync_channel(0);
    let (go, wait) = sync_channel(0);
    let thread = std::thread::spawn(move || {
        cb.render(&mut [0.0; 32], 100);
        ready.send(()).unwrap();
        wait.recv().unwrap();
        drop(cb);
    });
    rx.recv_timeout(std::time::Duration::from_secs(10)).unwrap();
    assert_eq!(native.stats().endpoint_returns, 0);
    go.send(()).unwrap();
    thread.join().unwrap();
    let mut r = r;
    let mut owner = owner;
    let (stopped, stats) = native.stop().unwrap();
    assert_eq!((stats.callbacks, stats.endpoint_returns), (1, 1));
    owner.finish(&mut r, stopped).unwrap();
    assert_eq!(pool.usage().bytes, 0);
}
#[test]
fn monitor_volume_is_bounded_applied_after_mix_and_not_a_project_edit() {
    let (pool, mut r, mut owner, mut cb, native) = setup();
    native.set_monitor_volume(500).unwrap();
    assert!(native.set_monitor_volume(1001).is_err());
    trigger(&mut r, Channel::Audition);
    owner.pump(&mut r).unwrap();
    let mut samples = [0.0; 4];
    cb.render(&mut samples, 1);
    assert_eq!(samples, [0.005; 4]);
    owner.pump(&mut r).unwrap();
    native.set_monitor_volume(0).unwrap();
    trigger(&mut r, Channel::Audition);
    owner.pump(&mut r).unwrap();
    cb.render(&mut samples, 2);
    assert_eq!(samples, [0.0; 4]);
    assert_eq!(native.stats().nonzero_samples, 4);
    finish(pool, r, owner, cb, native);
}
#[test]
fn detached_observer_is_unique_and_cannot_pin_endpoint_pcm() {
    let (pool, mut r, mut owner, mut cb, mut native) = setup();
    let mut observer = native.take_first_observer().unwrap();
    assert!(native.take_first_observer().is_none());
    let voice = trigger(&mut r, Channel::Audition);
    owner.pump(&mut r).unwrap();
    cb.render(&mut [0.0; 4], 3);
    assert!(native.next_first().is_none());
    assert_eq!(observer.next_first().unwrap().voice, voice);
    assert!(observer.next_first().is_none());
    finish(pool.clone(), r, owner, cb, native);
    assert_eq!(pool.usage().bytes, 0);
    assert!(observer.next_first().is_none());
}
#[test]
fn thirty_two_voice_retirements_and_cancel_do_not_destroy_managed_resources_in_render() {
    let (pool, mut r, mut owner, mut cb, mut native) = setup();
    for round in 0..10 {
        for i in 0..32 {
            trigger(
                &mut r,
                if i % 2 == 0 {
                    Channel::Scene
                } else {
                    Channel::Audition
                },
            );
        }
        owner.pump(&mut r).unwrap();
        if round % 2 == 0 {
            r.stop_audition(r.token(Channel::Audition)).unwrap();
        }
        cb.render(&mut [0.0; 4], round + 1);
        assert_eq!(native.stats().callback_resource_releases, 0);
        while native.next_first().is_some() {}
        owner.pump(&mut r).unwrap();
        owner.refill().unwrap();
    }
    finish(pool, r, owner, cb, native);
}
#[test]
fn audit_detects_accidental_endpoint_destruction_inside_a_data_callback_scope() {
    let (pool, mut r, mut owner, cb, native) = setup();
    let scope = crate::callback_audit::Scope::enter();
    drop(cb);
    assert_eq!(scope.finish(), 0); // Ownership transferred, not destroyed.
    let (stopped, stats) = native.stop().unwrap();
    assert_eq!(stats.endpoint_returns_in_render, 1);
    assert_eq!(stats.endpoint_returns_outside_render, 0);
    owner.finish(&mut r, stopped).unwrap();
    assert_eq!(pool.usage().bytes, 0);
}
#[test]
fn managed_drop_detector_is_live_for_a_returned_endpoint_dropped_in_the_wrong_scope() {
    let (pool, r, owner, cb, native) = setup();
    drop(cb);
    let (stopped, _) = native.stop().unwrap();
    let scope = crate::callback_audit::Scope::enter();
    drop(stopped);
    assert!(scope.finish() >= 2);
    drop((r, owner));
    assert_eq!(pool.usage().bytes, 0);
}
#[test]
fn optional_work_observation_does_not_change_samples_or_hold_resources() {
    let (pool, mut r, mut owner, mut cb, native) = setup();
    trigger(&mut r, Channel::Audition);
    owner.pump(&mut r).unwrap();
    let mut samples = [0.0; 4];
    cb.render(&mut samples, 1);
    assert_eq!(samples, [0.01; 4]);
    assert_eq!(native.work_cost(), Some([0; 6]));
    owner.pump(&mut r).unwrap();
    trigger(&mut r, Channel::Audition);
    owner.pump(&mut r).unwrap();
    native.capture_work_cost();
    cb.render(&mut samples, 2);
    assert_eq!(samples, [0.01; 4]);
    let cost = native.work_cost().unwrap();
    assert_eq!(cost[0], 1);
    assert_eq!(cost[5], 2);
    assert!(cost[1] >= cost[2..5].iter().sum());
    assert_eq!(native.stats().callback_resource_releases, 0);
    finish(pool, r, owner, cb, native);
}
#[test]
fn partial_callback_work_snapshot_is_rejected_without_spinning() {
    let v = WorkCounters::default();
    v.publish([7, 8, 1, 2, 3, 9]);
    assert_eq!(v.read(), Some([7, 8, 1, 2, 3, 9]));
    v.stamp.fetch_add(1, Ordering::SeqCst);
    assert!(v.read().is_none());
}
