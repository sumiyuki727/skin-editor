use super::*;
use skin_editor_audio::{Completion, PrepareFailure, RuntimeError};

fn selected(
    pool: &PcmBudget,
    p: SoundPlan,
    sound: PreparedSound,
    rate: u32,
    channels: u16,
) -> Runtime {
    let mut runtime = Runtime::with_budget(rate, channels, pool.clone()).unwrap();
    runtime.enable_minimum_conversion();
    runtime.enable_voice_handoff().unwrap();
    let jobs = runtime.synchronize(origin(), &[p]).unwrap();
    runtime.complete(jobs[0], Ok(sound)).unwrap();
    runtime
        .set_scene_running(runtime.token(Channel::Scene), true)
        .unwrap();
    runtime
}
fn trigger(runtime: &mut Runtime, lane: Channel) -> skin_editor_audio::TriggerReceipt {
    runtime
        .trigger(
            runtime.token(lane),
            AudioComponentId::new(1).unwrap(),
            runtime.mix_frame(),
        )
        .unwrap()
}
fn replacement(p: SoundPlan) -> SoundPlan {
    SoundPlan {
        source: BlobRef::new([2; 32], p.source.byte_len()),
        gain: Gain::new(0.5).unwrap(),
        ..p
    }
}

#[test]
fn live_audition_gain_including_zero_and_cancel_preserves_native_cursor_and_pcm_pin() {
    let pool = PcmBudget::default();
    let mut p = plan(48000, 2, 16);
    p.gain = Gain::new(0.25).unwrap();
    let samples: Vec<f32> = (1..=16).flat_map(|i| [i as f32 / 32.0; 2]).collect();
    let sound = PreparedSound::from_source(p, &samples, &pool).unwrap();
    let weak = sound.pcm().downgrade();
    let mut runtime = selected(&pool, p, sound, 48000, 2);
    let (mut owner, mut callback) = pair(&pool);
    let voice = trigger(&mut runtime, Channel::Audition).voice;
    owner.pump_runtime(&mut runtime).unwrap();
    let mut out = [0.; 4];
    callback.render_split(&mut [0.; 4], &mut out).unwrap();
    assert_eq!(out, [1. / 128., 1. / 128., 2. / 128., 2. / 128.]);
    for (revision, gain, expected) in [
        (1, 0., [0.; 4]),
        (2, 1., [5. / 32., 5. / 32., 6. / 32., 6. / 32.]),
    ] {
        let next = Origin {
            draft: Some((8, revision)),
            ..origin()
        };
        runtime
            .synchronize(
                next,
                &[SoundPlan {
                    gain: Gain::new(gain).unwrap(),
                    ..p
                }],
            )
            .unwrap();
        callback.render_split(&mut [0.; 4], &mut out).unwrap();
        assert_eq!(out, expected);
    }
    runtime.synchronize_cancelled(origin(), &[p], &[8]).unwrap();
    callback.render_split(&mut [0.; 4], &mut out).unwrap();
    assert_eq!(out, [7. / 128., 7. / 128., 8. / 128., 8. / 128.]);
    assert_eq!(runtime.voices().next().unwrap().id, voice.unwrap());
    assert_eq!(runtime.voices().next().unwrap().origin, origin());
    assert_eq!(pool.usage().bytes, 128);
    runtime.shutdown();
    assert!(weak.is_alive()); // Transport still owns the raw PCM until endpoint return.
    owner.finish_runtime(&mut runtime, callback.stop()).unwrap();
    assert!(!weak.is_alive());
    assert_eq!(pool.usage().bytes, 0);
}

#[test]
fn scene_activation_uses_runtime_authority_and_preserves_started_old_versions() {
    let pool = PcmBudget::default();
    let p = plan(48000, 2, 16);
    let sound = ready(&pool, 16);
    let mut runtime = selected(&pool, p, sound, 48000, 2);
    let (mut owner, mut callback) = pair(&pool);
    let a = trigger(&mut runtime, Channel::Scene);
    assert_eq!(owner.pump_runtime(&mut runtime).unwrap().admitted, 1);
    let next = Origin {
        revision: 2,
        node: 2,
        ..origin()
    };
    let p2 = replacement(p);
    let job = runtime.synchronize(next, &[p2]).unwrap()[0];
    let fallback = trigger(&mut runtime, Channel::Scene);
    assert!(fallback.stale);
    assert_eq!(fallback.origin, a.origin);
    assert_eq!(
        runtime.complete(job, Err(PrepareFailure::InvalidPcm)),
        Ok(Completion::Failed(PrepareFailure::InvalidPcm))
    );
    let retry = runtime.retry(AudioComponentId::new(1).unwrap()).unwrap();
    let late = PreparedSound::from_source(p2, &[0.5; 32], &pool).unwrap();
    assert_eq!(
        runtime.complete(job, Ok(late.clone())),
        Ok(Completion::Stale)
    );
    assert!(matches!(
        runtime.complete(retry, Ok(late)),
        Ok(Completion::Activated { .. })
    ));
    let fresh = trigger(&mut runtime, Channel::Scene);
    assert!(!fresh.stale);
    assert_eq!(fresh.origin, next);
    assert_eq!(owner.pump_runtime(&mut runtime).unwrap().admitted, 2);
    assert_eq!(runtime.voices().filter(|v| v.origin == origin()).count(), 2);
    let mut audition = [0.0; 40];
    callback
        .render_split(&mut audition, &mut [0.0; 40])
        .unwrap();
    assert_eq!(&audition[..32], &[0.75; 32]);
    assert_eq!(&audition[32..], &[0.0; 8]);
    assert_eq!(owner.collect(), 0); // Raw API cannot lose Runtime acknowledgements.
    assert_eq!(owner.pump_runtime(&mut runtime).unwrap().retired, 3);
    assert_eq!(runtime.voices().count(), 0);
    runtime.shutdown();
    owner.finish(callback.stop()).unwrap();
    assert_eq!(pool.usage().bytes, 0);
}

#[test]
fn latched_draft_cancel_removes_only_its_contribution_even_after_short_voice_ended() {
    for frames in [1, 16] {
        for phase in 0..3 {
            let pool = PcmBudget::default();
            let p = plan(48000, 2, frames);
            let sound = ready(&pool, frames);
            let mut runtime = selected(&pool, p, sound, 48000, 2);
            let (mut owner, mut callback) = pair(&pool);
            trigger(&mut runtime, Channel::Scene);
            trigger(&mut runtime, Channel::Audition);
            owner.pump_runtime(&mut runtime).unwrap();
            let draft = Origin {
                draft: Some((7, 1)),
                ..origin()
            };
            let p2 = replacement(p);
            let job = runtime.synchronize(draft, &[p2]).unwrap()[0];
            let sound2 = PreparedSound::from_source(p2, &vec![0.5; frames * 2], &pool).unwrap();
            let weak = sound2.pcm().downgrade();
            runtime.complete(job, Ok(sound2)).unwrap();
            trigger(&mut runtime, Channel::Scene);
            trigger(&mut runtime, Channel::Audition);
            owner.pump_runtime(&mut runtime).unwrap();
            callback = std::thread::scope(|scope| {
                let (ready_tx, ready_rx) = sync_channel(0);
                let (go_tx, go_rx) = sync_channel(0);
                let worker = scope.spawn(move || {
                    let mut scene = [0.0; 2];
                    let mut audition = [0.0; 2];
                    let report = callback
                        .render_inner(&mut scene, &mut audition, |stage| {
                            if matches!(
                                (stage, phase),
                                (Checkpoint::BeforeFence, 1)
                                    | (Checkpoint::AfterDrain, 0)
                                    | (Checkpoint::AfterRetire, 2)
                            ) {
                                ready_tx.send(()).unwrap();
                                go_rx.recv_timeout(GUARD).unwrap();
                            }
                        })
                        .unwrap();
                    (callback, report, scene, audition)
                });
                ready_rx.recv_timeout(GUARD).unwrap();
                runtime.invalidate_drafts(&[7]); // No pump/UI refresh needed to revoke packets.
                assert!(weak.is_alive());
                assert_eq!(runtime.voices().count(), 1);
                go_tx.send(()).unwrap();
                let (returned, report, scene, audition) = worker.join().unwrap();
                assert_eq!(scene, [0.25; 2]);
                assert_eq!(audition, [0.0; 2]);
                assert_eq!(report.first.iter().flatten().count(), 1);
                returned
            });
            owner.pump_runtime(&mut runtime).unwrap();
            assert!(!weak.is_alive());
            runtime.shutdown();
            owner.finish(callback.stop()).unwrap();
            assert_eq!(pool.usage().bytes, 0);
        }
    }
}

#[test]
fn same_node_history_navigation_revokes_both_lanes_without_another_owner_pump() {
    for phase in 0..3 {
        let pool = PcmBudget::default();
        let p = plan(48000, 2, 16);
        let mut runtime = selected(&pool, p, ready(&pool, 16), 48000, 2);
        let (mut owner, mut callback) = pair(&pool);
        trigger(&mut runtime, Channel::Scene);
        trigger(&mut runtime, Channel::Audition);
        owner.pump_runtime(&mut runtime).unwrap();
        std::thread::scope(|scope| {
            let (ready_tx, ready_rx) = sync_channel(0);
            let (go_tx, go_rx) = sync_channel(0);
            let worker = scope.spawn(move || {
                let mut scene = [0.0; 2];
                let mut audition = [0.0; 2];
                let report = callback
                    .render_inner(&mut scene, &mut audition, |stage| {
                        if matches!(
                            (stage, phase),
                            (Checkpoint::BeforeFence, 1)
                                | (Checkpoint::AfterDrain, 0)
                                | (Checkpoint::AfterRetire, 2)
                        ) {
                            ready_tx.send(()).unwrap();
                            go_rx.recv_timeout(GUARD).unwrap();
                        }
                    })
                    .unwrap();
                (callback.stop(), report, scene, audition)
            });
            ready_rx.recv_timeout(GUARD).unwrap();
            let next = Origin {
                navigation_revision: 2,
                ..origin()
            };
            runtime.synchronize(next, &[p]).unwrap();
            assert_eq!(runtime.voices().count(), 0);
            go_tx.send(()).unwrap();
            let (stopped, report, scene, audition) = worker.join().unwrap();
            assert_eq!(scene, [0.0; 2]);
            assert_eq!(audition, [0.0; 2]);
            assert!(report.first.iter().all(Option::is_none));
            runtime.shutdown();
            owner.finish(stopped).unwrap();
        });
        assert_eq!(pool.usage().bytes, 0);
    }
}

#[test]
fn runtime_pause_resume_keeps_audition_cursor_and_rejects_frozen_scene_input() {
    let pool = PcmBudget::default();
    let p = plan(48000, 2, 4);
    let samples = [0.125, 0.125, 0.25, 0.25, 0.375, 0.375, 0.5, 0.5];
    let sound = PreparedSound::from_source(p, &samples, &pool).unwrap();
    let mut runtime = selected(&pool, p, sound, 48000, 2);
    let (mut owner, mut callback) = pair(&pool);
    trigger(&mut runtime, Channel::Scene);
    trigger(&mut runtime, Channel::Audition);
    owner.pump_runtime(&mut runtime).unwrap();
    callback.render_split(&mut [0.0; 2], &mut [0.0; 2]).unwrap();
    let old = runtime.token(Channel::Scene);
    runtime.set_scene_running(old, false).unwrap();
    assert_eq!(
        runtime.trigger(runtime.token(Channel::Scene), p.target, 0),
        Err(RuntimeError::Frozen)
    );
    owner.pump_runtime(&mut runtime).unwrap();
    let mut scene = [1.0; 2];
    let mut audition = [0.0; 2];
    callback.render_split(&mut scene, &mut audition).unwrap();
    assert_eq!(scene, [0.0; 2]);
    assert_eq!(audition, [0.25; 2]);
    runtime
        .set_scene_running(runtime.token(Channel::Scene), true)
        .unwrap();
    assert_eq!(runtime.trigger(old, p.target, 0), Err(RuntimeError::Stale));
    trigger(&mut runtime, Channel::Scene);
    owner.pump_runtime(&mut runtime).unwrap();
    callback.render_split(&mut scene, &mut audition).unwrap();
    assert_eq!(scene, [0.125; 2]);
    assert_eq!(audition, [0.375; 2]);
    runtime.shutdown();
    owner.finish(callback.stop()).unwrap();
    assert_eq!(pool.usage().bytes, 0);
}

#[test]
fn retired_transport_credits_bound_new_runtime_voices_and_never_lose_busy_requests() {
    let pool = PcmBudget::default();
    let p = plan(48000, 2, 1);
    let sound = ready(&pool, 1);
    let mut runtime = selected(&pool, p, sound.clone(), 48000, 2);
    let (mut owner, mut callback) = pair(&pool);
    for _ in 0..MAX_VOICES {
        trigger(&mut runtime, Channel::Audition);
    }
    assert_eq!(
        owner.pump_runtime(&mut runtime).unwrap().admitted,
        MAX_VOICES
    );
    let next = Origin {
        navigation_revision: 2,
        ..origin()
    };
    let job = runtime.synchronize(next, &[p]).unwrap()[0];
    runtime.complete(job, Ok(sound)).unwrap();
    for _ in 0..MAX_VOICES {
        trigger(&mut runtime, Channel::Audition);
    }
    assert_eq!(
        owner.pump_runtime(&mut runtime),
        Ok(RuntimePumpReport {
            admitted: 0,
            retired: 0,
            waiting: MAX_VOICES
        })
    );
    let mut audition = [1.0; 2];
    callback.render_split(&mut [0.0; 2], &mut audition).unwrap();
    assert_eq!(audition, [0.0; 2]);
    assert_eq!(owner.collect(), 0);
    assert_eq!(
        owner.pump_runtime(&mut runtime),
        Ok(RuntimePumpReport {
            admitted: MAX_VOICES,
            retired: MAX_VOICES,
            waiting: 0
        })
    );
    assert_eq!(runtime.voices().count(), MAX_VOICES);
    callback.render_split(&mut [0.0; 2], &mut audition).unwrap();
    assert_eq!(audition, [8.0; 2]);
    assert_eq!(
        owner.pump_runtime(&mut runtime).unwrap().retired,
        MAX_VOICES
    );
    assert_eq!(runtime.voices().count(), 0);
    runtime.shutdown();
    owner.finish(callback.stop()).unwrap();
    assert_eq!(pool.usage().bytes, 0);
}

#[test]
fn bridge_rejects_mode_pool_format_and_exhaustion_before_exporting_any_voice() {
    let pool = PcmBudget::default();
    let (mut owner, callback) = pair(&pool);
    let mut ordinary = Runtime::with_budget(48000, 2, pool.clone()).unwrap();
    assert_eq!(
        owner.pump_runtime(&mut ordinary),
        Err(HandoffError::RuntimeMode)
    );
    let mut wrong_format = Runtime::with_budget(44100, 2, pool.clone()).unwrap();
    wrong_format.enable_voice_handoff().unwrap();
    assert_eq!(
        owner.pump_runtime(&mut wrong_format),
        Err(HandoffError::InvalidFormat)
    );
    let mut foreign = Runtime::new(48000, 2).unwrap();
    foreign.enable_voice_handoff().unwrap();
    assert_eq!(
        owner.pump_runtime(&mut foreign),
        Err(HandoffError::WrongPool)
    );
    let mut runtime = selected(&pool, plan(48000, 2, 1), ready(&pool, 1), 48000, 2);
    trigger(&mut runtime, Channel::Audition);
    owner.serial = u64::MAX;
    assert_eq!(
        owner.pump_runtime(&mut runtime),
        Err(HandoffError::Exhausted)
    );
    assert_eq!(runtime.handoff_waiting(), 1);
    assert_eq!(owner.outstanding(), 0);
    runtime.shutdown();
    owner.finish(callback.stop()).unwrap();
    assert_eq!(pool.usage().bytes, 0);
}

#[test]
fn foreign_runtime_cannot_drain_returns_or_close_the_bound_runtime() {
    let pool = PcmBudget::default();
    let p = plan(48000, 2, 1);
    let mut a = selected(&pool, p, ready(&pool, 1), 48000, 2);
    let mut b = selected(&pool, p, ready(&pool, 1), 48000, 2);
    let (mut owner, mut callback) = pair(&pool);
    trigger(&mut a, Channel::Audition);
    trigger(&mut b, Channel::Audition);
    owner.pump_runtime(&mut a).unwrap();
    callback.render_split(&mut [0.0; 2], &mut [0.0; 2]).unwrap();
    assert_eq!(
        owner.pump_runtime(&mut b),
        Err(HandoffError::ForeignRuntime)
    );
    assert_eq!(owner.outstanding(), 1);
    assert_eq!(a.voices().count(), 1);
    assert_eq!(b.voices().count(), 1);
    let stopped = owner.finish_runtime(&mut b, callback.stop()).unwrap_err();
    assert_eq!(b.voices().count(), 1);
    assert_eq!(owner.outstanding(), 1);
    owner.finish_runtime(&mut a, stopped).unwrap();
    assert_eq!(a.voices().count(), 0);
    b.shutdown();
    assert_eq!(pool.usage().bytes, 0);
}

#[test]
fn failed_publish_can_retry_and_abandoned_consumer_does_not_leak_runtime_credits() {
    let pool = PcmBudget::default();
    let p = plan(48000, 2, 16);
    let mut runtime = selected(&pool, p, ready(&pool, 16), 48000, 2);
    let (mut owner, callback) = pair(&pool);
    trigger(&mut runtime, Channel::Audition);
    let lease = runtime.take_voice_handoff().unwrap().unwrap();
    // A stopped lane between export/admission must not leave an exported ghost.
    let old = owner.token(Channel::Audition);
    owner.stop_audition().unwrap();
    assert_eq!(
        owner.publish(lease.info().origin, old, lease.sound(), Some(&lease)),
        Err(HandoffError::Stale)
    );
    assert!(runtime.retry_voice_handoff(&lease));
    assert!(!lease.validity().is_valid());
    drop(lease);
    assert_eq!(runtime.handoff_waiting(), 1);
    assert_eq!(owner.pump_runtime(&mut runtime).unwrap().admitted, 1);
    drop(callback); // Outside render; no future render call.
    assert_eq!(owner.pump_runtime(&mut runtime), Err(HandoffError::Closed));
    assert_eq!(runtime.voices().count(), 0);
    assert_eq!(owner.outstanding(), 0);
    runtime.shutdown();
    assert_eq!(pool.usage().bytes, 0);
}

#[test]
fn binding_runtime_rejects_raw_voices_and_does_not_adopt_an_existing_raw_queue() {
    let pool = PcmBudget::default();
    let sound = ready(&pool, 1);
    let mut runtime = selected(&pool, plan(48000, 2, 1), sound.clone(), 48000, 2);
    let (mut owner, mut callback) = pair(&pool);
    admit(&mut owner, Channel::Audition, &sound);
    assert_eq!(owner.pump_runtime(&mut runtime), Err(HandoffError::Busy));
    callback.render_split(&mut [0.0; 2], &mut [0.0; 2]).unwrap();
    owner.collect();
    owner.pump_runtime(&mut runtime).unwrap();
    assert_eq!(
        owner.admit(origin(), owner.token(Channel::Audition), &sound),
        Err(HandoffError::RuntimeMode)
    );
    drop(sound);
    owner.finish_runtime(&mut runtime, callback.stop()).unwrap();
    assert_eq!(pool.usage().bytes, 0);
}

#[test]
fn runtime_lane_cancellation_at_each_fence_keeps_the_other_lane_exact() {
    for cancel_scene in [false, true] {
        for phase in 0..3 {
            let pool = PcmBudget::default();
            let p = plan(48000, 2, 1);
            let mut runtime = selected(&pool, p, ready(&pool, 1), 48000, 2);
            let (mut owner, mut callback) = pair(&pool);
            trigger(&mut runtime, Channel::Scene);
            trigger(&mut runtime, Channel::Audition);
            owner.pump_runtime(&mut runtime).unwrap();
            std::thread::scope(|scope| {
                let (ready_tx, ready_rx) = sync_channel(0);
                let (go_tx, go_rx) = sync_channel(0);
                let worker = scope.spawn(move || {
                    let mut scene = [0.0; 2];
                    let mut audition = [0.0; 2];
                    let report = callback
                        .render_inner(&mut scene, &mut audition, |stage| {
                            if matches!(
                                (stage, phase),
                                (Checkpoint::AfterDrain, 0)
                                    | (Checkpoint::BeforeFence, 1)
                                    | (Checkpoint::AfterRetire, 2)
                            ) {
                                ready_tx.send(()).unwrap();
                                go_rx.recv_timeout(GUARD).unwrap();
                            }
                        })
                        .unwrap();
                    (callback.stop(), report, scene, audition)
                });
                ready_rx.recv_timeout(GUARD).unwrap();
                if cancel_scene {
                    runtime
                        .set_scene_running(runtime.token(Channel::Scene), false)
                        .unwrap();
                } else {
                    runtime
                        .stop_audition(runtime.token(Channel::Audition))
                        .unwrap();
                }
                go_tx.send(()).unwrap();
                let (stopped, report, scene, audition) = worker.join().unwrap();
                assert_eq!(scene, [if cancel_scene { 0.0 } else { 0.25 }; 2]);
                assert_eq!(audition, [if cancel_scene { 0.25 } else { 0.0 }; 2]);
                assert_eq!(report.first.iter().flatten().count(), 1);
                owner.finish_runtime(&mut runtime, stopped).unwrap();
            });
            assert_eq!(pool.usage().bytes, 0);
        }
    }
}

#[test]
fn selective_replay_preserves_float_summation_instead_of_subtracting_old_contribution() {
    let pool = PcmBudget::default();
    let mut plans = [plan(48000, 2, 1); 3];
    let values = [0.1, 0.3, 0.2]; // Third contribution belongs to the cancelled draft.
    for (i, p) in plans.iter_mut().enumerate() {
        p.target = AudioComponentId::new(i as u128 + 1).unwrap().into();
        p.source = BlobRef::new([i as u8 + 1; 32], 8);
    }
    let mut runtime = Runtime::with_budget(48000, 2, pool.clone()).unwrap();
    runtime.enable_voice_handoff().unwrap();
    for job in runtime.synchronize(origin(), &plans[..2]).unwrap() {
        let index = if job.plan == plans[0] { 0 } else { 1 };
        runtime
            .complete(
                job,
                Ok(PreparedSound::from_source(job.plan, &[values[index]; 2], &pool).unwrap()),
            )
            .unwrap();
    }
    for p in &plans[..2] {
        runtime
            .trigger(runtime.token(Channel::Audition), p.target, 0)
            .unwrap();
    }
    let (mut owner, mut callback) = pair(&pool);
    owner.pump_runtime(&mut runtime).unwrap();
    let draft = Origin {
        draft: Some((9, 1)),
        ..origin()
    };
    let job = runtime.synchronize(draft, &plans).unwrap()[0];
    runtime
        .complete(
            job,
            Ok(PreparedSound::from_source(job.plan, &[values[2]; 2], &pool).unwrap()),
        )
        .unwrap();
    runtime
        .trigger(runtime.token(Channel::Audition), plans[2].target, 0)
        .unwrap();
    owner.pump_runtime(&mut runtime).unwrap();
    std::thread::scope(|scope| {
        let (ready_tx, ready_rx) = sync_channel(0);
        let (go_tx, go_rx) = sync_channel(0);
        let worker = scope.spawn(move || {
            let mut audition = [0.0; 2];
            callback
                .render_inner(&mut [0.0; 2], &mut audition, |stage| {
                    if matches!(stage, Checkpoint::AfterRetire) {
                        ready_tx.send(()).unwrap();
                        go_rx.recv_timeout(GUARD).unwrap();
                    }
                })
                .unwrap();
            (callback.stop(), audition)
        });
        ready_rx.recv_timeout(GUARD).unwrap();
        runtime.invalidate_drafts(&[9]);
        go_tx.send(()).unwrap();
        let (stopped, actual) = worker.join().unwrap();
        let expected = values[0] + values[1];
        assert_ne!((expected + values[2]) - values[2], expected);
        assert_eq!(actual, [expected; 2]);
        owner.finish_runtime(&mut runtime, stopped).unwrap();
    });
    assert_eq!(pool.usage().bytes, 0);
}

#[test]
fn stop_without_a_further_callback_releases_runtime_pending_active_and_returned_pcm() {
    for phase in 0..3 {
        let pool = PcmBudget::default();
        let p = plan(48000, 2, 2);
        let mut runtime = selected(&pool, p, ready(&pool, 2), 48000, 2);
        let (mut owner, mut callback) = pair(&pool);
        trigger(&mut runtime, Channel::Audition);
        owner.pump_runtime(&mut runtime).unwrap();
        if phase > 0 {
            let count = if phase == 1 { 2 } else { 4 };
            callback
                .render_split(&mut vec![0.0; count], &mut vec![0.0; count])
                .unwrap();
        }
        runtime.shutdown();
        assert_eq!(pool.usage().bytes, 16); // Not refunded until transport releases.
        owner.finish(callback.stop()).unwrap();
        assert_eq!(owner.outstanding(), 0);
        assert_eq!(pool.usage().bytes, 0);
    }
}

#[test]
fn shared_sampler_matches_runtime_across_all_minimum_conversion_pairs() {
    for source_rate in [44100, 48000] {
        for source_channels in [1, 2] {
            for rate in [44100, 48000] {
                for channels in [1, 2] {
                    let pool = PcmBudget::default();
                    let p = plan(source_rate, source_channels, 7);
                    let raw: Vec<_> = (0..7 * source_channels)
                        .map(|i| (i as f32 - 7.0) / 16.0)
                        .collect();
                    let sound = PreparedSound::from_source(p, &raw, &pool).unwrap();
                    let mut runtime = selected(&pool, p, sound.clone(), rate, channels);
                    let mut reference = Runtime::with_budget(rate, channels, pool.clone()).unwrap();
                    reference.enable_minimum_conversion();
                    let job = reference.synchronize(origin(), &[p]).unwrap()[0];
                    reference.complete(job, Ok(sound)).unwrap();
                    let (mut owner, mut callback) =
                        boundary(pool.clone(), rate, channels, origin()).unwrap();
                    trigger(&mut runtime, Channel::Audition);
                    trigger(&mut reference, Channel::Audition);
                    owner.pump_runtime(&mut runtime).unwrap();
                    for frames in [1, 2, 9] {
                        let mut expected = vec![0.0; frames * usize::from(channels)];
                        let mut actual = expected.clone();
                        let mut scene = expected.clone();
                        reference.mix(&mut expected).unwrap();
                        callback.render_split(&mut scene, &mut actual).unwrap();
                        assert_eq!(
                            actual, expected,
                            "{source_rate}/{source_channels}->{rate}/{channels}"
                        );
                        assert!(scene.iter().all(|s| *s == 0.0));
                        owner.pump_runtime(&mut runtime).unwrap();
                    }
                    assert_eq!(runtime.voices().count(), 0);
                    runtime.shutdown();
                    reference.shutdown();
                    owner.finish(callback.stop()).unwrap();
                    assert_eq!(pool.usage().bytes, 0);
                }
            }
        }
    }
}

#[test]
fn three_hundred_runtime_rounds_share_pcm_and_release_authoritative_voice_credits() {
    let pool = PcmBudget::new(128, 128).unwrap();
    let mut runtime = Runtime::with_budget(48000, 2, pool.clone()).unwrap();
    runtime.enable_voice_handoff().unwrap();
    let (mut owner, mut callback) = pair(&pool);
    for round in 0..300 {
        let p = plan(48000, 2, 1);
        let next = Origin {
            revision: round * 2 + 2,
            ..origin()
        };
        let job = runtime.synchronize(next, &[p]).unwrap()[0];
        let sound = ready(&pool, 1);
        let weak = sound.pcm().downgrade();
        runtime.complete(job, Ok(sound)).unwrap();
        for _ in 0..MAX_VOICES {
            trigger(&mut runtime, Channel::Audition);
        }
        owner.pump_runtime(&mut runtime).unwrap();
        assert_eq!(pool.usage().bytes, 8);
        let mut audition = [0.0; 2];
        callback.render_split(&mut [0.0; 2], &mut audition).unwrap();
        assert_eq!(audition, [8.0; 2]);
        let report = owner.pump_runtime(&mut runtime).unwrap();
        assert_eq!(report.retired, MAX_VOICES);
        assert_eq!(report.waiting, 0);
        assert_eq!(runtime.voices().count(), 0);
        runtime
            .synchronize(
                Origin {
                    revision: next.revision + 1,
                    ..next
                },
                &[],
            )
            .unwrap();
        assert!(!weak.is_alive());
        assert_eq!(pool.usage().bytes, 0);
    }
    runtime.shutdown();
    owner.finish(callback.stop()).unwrap();
    println!(
        "RUNTIME_BRIDGE_PROOF rounds=300 voices=9600 conversion_pairs=16 selective_cancel_latches=6 history_latches=3 pcm_peak_bytes={} pcm_final_bytes={}",
        pool.usage().peak_bytes,
        pool.usage().bytes
    );
}
