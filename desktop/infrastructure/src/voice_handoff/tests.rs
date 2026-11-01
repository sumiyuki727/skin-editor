use super::*;
use skin_editor_application::values::{AudioComponentId, AudioMetadata, BlobRef, Gain};
use skin_editor_audio::{Runtime, SoundPlan};
use std::sync::mpsc::sync_channel;
use std::time::Duration;

const GUARD: Duration = Duration::from_secs(10); // Deadlock guard, not a timing assertion.
mod runtime_bridge;
fn origin() -> Origin {
    Origin {
        session: 1,
        node: 1,
        revision: 1,
        navigation_revision: 1,
        draft: None,
    }
}
fn plan(rate: u32, channels: u16, frames: usize) -> SoundPlan {
    SoundPlan {
        target: AudioComponentId::new(1).unwrap().into(),
        source: BlobRef::new([1; 32], (frames * usize::from(channels) * 2) as u64),
        metadata: Some(AudioMetadata::new(rate, channels, frames as u64).unwrap()),
        gain: Gain::UNITY,
        visible: true,
    }
}
fn ready(pool: &PcmBudget, frames: usize) -> PreparedSound {
    PreparedSound::from_source(plan(48000, 2, frames), &vec![0.25; frames * 2], pool).unwrap()
}
fn pair(pool: &PcmBudget) -> (VoiceOwner, Box<VoiceConsumer>) {
    boundary(pool.clone(), 48000, 2, origin()).unwrap()
}
fn admit(owner: &mut VoiceOwner, channel: Channel, sound: &PreparedSound) -> VoiceKey {
    owner
        .admit(owner.source, owner.token(channel), sound)
        .unwrap()
}

#[test]
fn ready_handoff_and_shared_gain_match_existing_runtime_for_same_format() {
    for rate in [44100, 48000] {
        for channels in [1, 2] {
            for (gain, visible) in [(1.0, true), (0.5, true), (0.0, true), (1.0, false)] {
                let pool = PcmBudget::default();
                let mut p = plan(rate, channels, 3);
                p.gain = Gain::new(gain).unwrap();
                p.visible = visible;
                let source: Vec<_> = (0..3 * channels).map(|i| (i as f32 - 2.0) / 8.0).collect();
                let sound = PreparedSound::from_source(p, &source, &pool).unwrap();
                assert_eq!(sound.format(), (rate, channels));
                let (mut owner, mut callback) =
                    boundary(pool.clone(), rate, channels, origin()).unwrap();
                admit(&mut owner, Channel::Audition, &sound);
                let mut scene = vec![f32::NAN; 10 * usize::from(channels)];
                let mut audition = scene.clone();
                let before = pool.usage();
                callback.render_split(&mut scene, &mut audition).unwrap();
                assert_eq!(pool.usage(), before);
                let mut reference = Runtime::with_budget(rate, channels, pool.clone()).unwrap();
                let jobs = reference.synchronize(origin(), &[p]).unwrap();
                if let Some(&job) = jobs.first() {
                    reference.complete(job, Ok(sound)).unwrap();
                } else {
                    drop(sound); // Hidden/known-silent plans need no prepared Voice.
                }
                reference
                    .trigger(reference.token(Channel::Audition), p.target, 0)
                    .unwrap();
                let mut expected = vec![0.0; audition.len()];
                reference.mix(&mut expected).unwrap();
                assert_eq!(audition, expected);
                assert!(scene.iter().all(|s| *s == 0.0));
                owner.finish(callback.stop()).unwrap();
                reference.shutdown();
                assert_eq!(pool.usage().bytes, 0);
            }
        }
    }
}
#[test]
fn voice_completion_is_exact_at_block_end_partial_tail_and_zero_length_callbacks() {
    for frames in [1, 2, 3, 7] {
        for block in [1usize, 2, 3, 8] {
            let pool = PcmBudget::default();
            let sound = ready(&pool, frames);
            let (mut owner, mut cb) = pair(&pool);
            admit(&mut owner, Channel::Audition, &sound);
            cb.render_split(&mut [], &mut []).unwrap();
            assert_eq!(owner.collect(), 0);
            let mut position = 0;
            while position < frames {
                let mut scene = vec![9.0; block * 2];
                let mut audition = scene.clone();
                cb.render_split(&mut scene, &mut audition).unwrap();
                for (i, values) in audition.chunks_exact(2).enumerate() {
                    assert_eq!(
                        values,
                        if position + i < frames {
                            &[0.25; 2]
                        } else {
                            &[0.0; 2]
                        }
                    );
                }
                assert_eq!(scene, vec![0.0; block * 2]);
                position += block;
                assert_eq!(owner.collect(), usize::from(position >= frames));
            }
            let report = cb.render_split(&mut [0.0; 2], &mut [0.0; 2]).unwrap();
            assert!(report.first.iter().all(Option::is_none));
            owner.finish(cb.stop()).unwrap();
            drop(sound);
            assert_eq!(pool.usage().bytes, 0);
        }
    }
}
#[test]
fn thirty_two_voice_block_visits_preserve_each_sample_sum_bits_and_mixed_ends() {
    for block in [1usize, 2, 7, 64] {
        let pool = PcmBudget::default();
        let (mut owner, mut cb) = pair(&pool);
        owner.set_scene_running(true).unwrap();
        let mut sounds = Vec::new();
        let mut expected = [vec![0.0f32; 128], vec![0.0f32; 128]];
        for i in 0..32 {
            let frames = 1 + i % 13;
            let value = [1.0f32, 1e-8, -1.0, -1e-8, 0.1][i % 5];
            let source: Vec<_> = (0..frames).flat_map(|_| [value, -value * 0.25]).collect();
            let sound = PreparedSound::from_source(plan(48000, 2, frames), &source, &pool).unwrap();
            let lane = if i % 2 == 0 {
                Channel::Scene
            } else {
                Channel::Audition
            };
            admit(&mut owner, lane, &sound);
            for frame in 0..frames {
                for c in 0..2 {
                    expected[i % 2][frame * 2 + c] += source[frame * 2 + c];
                }
            }
            sounds.push(sound);
        }
        let mut actual = [Vec::new(), Vec::new()];
        let mut position = 0;
        while position < 64 {
            let count = block.min(64 - position);
            let mut scene = vec![0.0; count * 2];
            let mut audition = scene.clone();
            let scope = crate::callback_audit::Scope::enter();
            cb.render_split(&mut scene, &mut audition).unwrap();
            assert_eq!(scope.finish(), 0);
            actual[0].extend(scene);
            actual[1].extend(audition);
            owner.collect();
            position += count;
        }
        for (actual, expected) in actual.iter().zip(&expected) {
            assert_eq!(actual.len(), expected.len());
            for (a, b) in actual.iter().zip(expected) {
                assert_eq!(a.to_bits(), b.to_bits());
            }
        }
        owner.finish(cb.stop()).unwrap();
        drop(sounds);
        assert_eq!(pool.usage().bytes, 0);
    }
}

#[test]
fn saturated_pending_active_and_retired_credits_never_block_stop_or_reuse_early() {
    for phase in 0..3 {
        let pool = PcmBudget::default();
        let (mut owner, mut callback) = pair(&pool);
        let sound = ready(&pool, if phase == 2 { 1 } else { 1000 });
        let mut first = None;
        for _ in 0..MAX_VOICES {
            first.get_or_insert(admit(&mut owner, Channel::Audition, &sound));
        }
        assert_eq!(
            owner.admit(origin(), owner.token(Channel::Audition), &sound),
            Err(HandoffError::Busy)
        );
        if phase > 0 {
            let mut scene = [0.0; 960];
            let mut audition = [0.0; 960];
            let before = pool.usage();
            let report = callback.render_split(&mut scene, &mut audition).unwrap();
            assert_eq!(pool.usage(), before);
            assert_eq!(report.deferred_returns, 0);
            assert_eq!(report.first.iter().flatten().count(), MAX_VOICES);
            assert_eq!(audition[0], 8.0); // Preserve unsaturated lanes for final adapter mix.
            assert_eq!(owner.outstanding(), MAX_VOICES);
            assert_eq!(
                owner.admit(origin(), owner.token(Channel::Audition), &sound),
                Err(HandoffError::Busy)
            );
        }
        owner.stop_audition().unwrap();
        let mut scene = [1.0; 2];
        let mut audition = [1.0; 2];
        callback.render_split(&mut scene, &mut audition).unwrap();
        assert_eq!(audition, [0.0; 2]);
        assert_eq!(owner.collect(), MAX_VOICES);
        let new = admit(&mut owner, Channel::Audition, &sound);
        assert_eq!(new.slot, first.unwrap().slot);
        assert_ne!(Some(new), first);
        owner.finish(callback.stop()).unwrap();
        drop(sound);
        assert_eq!(pool.usage().bytes, 0);
    }
}

#[test]
fn full_source_and_boundary_identity_reject_stale_or_foreign_requests() {
    let pool = PcmBudget::default();
    let (mut owner, callback) = pair(&pool);
    let (other, other_callback) = pair(&pool);
    let sound = ready(&pool, 1);
    assert_eq!(
        owner.admit(origin(), other.token(Channel::Audition), &sound),
        Err(HandoffError::Stale)
    );
    let mut variants = [origin(); 6];
    variants[0].session += 1;
    variants[1].node += 1;
    variants[2].revision += 1;
    variants[3].navigation_revision += 1;
    variants[4].draft = Some((1, 1));
    variants[5].draft = Some((1, 2));
    for old in variants {
        assert_eq!(
            owner.admit(old, owner.token(Channel::Audition), &sound),
            Err(HandoffError::Stale)
        );
    }
    let old = owner.token(Channel::Audition);
    owner.stop_audition().unwrap();
    assert_eq!(owner.admit(origin(), old, &sound), Err(HandoffError::Stale));
    assert_eq!(owner.outstanding(), 0);
    owner.finish(callback.stop()).unwrap();
    drop((other, other_callback, sound));
    assert_eq!(pool.usage().bytes, 0);
}

#[test]
fn same_pool_format_and_origin_failures_leave_no_published_voice_or_pin() {
    let pool = PcmBudget::default();
    assert!(matches!(
        boundary(pool.clone(), 48000, 3, origin()),
        Err(HandoffError::InvalidFormat)
    ));
    assert!(matches!(
        boundary(pool.clone(), 32000, 2, origin()),
        Err(HandoffError::InvalidFormat)
    ));
    let mut bad = origin();
    bad.session = 0;
    assert!(matches!(
        boundary(pool.clone(), 48000, 2, bad),
        Err(HandoffError::InvalidOrigin)
    ));
    let (mut owner, mut callback) = pair(&pool);
    let wrong_pool = ready(&PcmBudget::default(), 1);
    assert_eq!(
        owner.admit(origin(), owner.token(Channel::Audition), &wrong_pool),
        Err(HandoffError::WrongPool)
    );
    let wrong_format = PreparedSound::from_source(plan(44100, 1, 1), &[0.5], &pool).unwrap();
    assert_eq!(
        owner.admit(origin(), owner.token(Channel::Audition), &wrong_format),
        Err(HandoffError::InvalidFormat)
    );
    assert_eq!(
        owner.invalidate_source(bad),
        Err(HandoffError::InvalidOrigin)
    );
    assert_eq!(owner.outstanding(), 0);
    let mut a = [0.0; 2];
    let mut b = [0.0; 2];
    assert!(
        callback
            .render_split(&mut a, &mut b)
            .unwrap()
            .first
            .iter()
            .all(Option::is_none)
    );
    owner.finish(callback.stop()).unwrap();
    drop(wrong_format);
    assert_eq!(pool.usage().bytes, 0);
}

#[test]
fn pause_resume_reject_frozen_inputs_and_leave_audition_valid() {
    let pool = PcmBudget::default();
    let (mut owner, mut callback) = pair(&pool);
    let sound = ready(&pool, 100);
    let frozen = owner.token(Channel::Scene);
    assert_eq!(
        owner.admit(origin(), frozen.clone(), &sound),
        Err(HandoffError::Frozen)
    );
    let audition = owner.token(Channel::Audition);
    owner.set_scene_running(true).unwrap();
    assert_eq!(
        owner.admit(origin(), frozen, &sound),
        Err(HandoffError::Stale)
    );
    admit(&mut owner, Channel::Scene, &sound);
    owner.admit(origin(), audition, &sound).unwrap();
    owner.set_scene_running(false).unwrap();
    let mut a = [1.0; 2];
    let mut b = [1.0; 2];
    callback.render_split(&mut a, &mut b).unwrap();
    assert_eq!(a, [0.0; 2]);
    assert_eq!(b, [0.25; 2]);
    let held = owner.token(Channel::Scene);
    owner.set_scene_running(true).unwrap();
    assert_eq!(
        owner.admit(origin(), held, &sound),
        Err(HandoffError::Stale)
    );
    owner.finish(callback.stop()).unwrap();
}

#[test]
fn latched_control_changes_reject_drained_and_already_generated_old_output() {
    // Cross-thread ordering is enforced by rendezvous channels, never sleep.
    for short in [false, true] {
        for at_fence in [false, true] {
            for action in 0..5 {
                let pool = PcmBudget::default();
                let (mut owner, mut callback) = pair(&pool);
                owner.set_scene_running(true).unwrap();
                if action == 3 {
                    let mut draft = origin();
                    draft.draft = Some((3, 7));
                    owner.invalidate_source(draft).unwrap();
                }
                let sound = ready(&pool, if short { 1 } else { 16 });
                admit(&mut owner, Channel::Scene, &sound);
                admit(&mut owner, Channel::Audition, &sound);
                let weak = sound.pcm().downgrade();
                drop(sound);
                let mut owner = Some(owner);
                std::thread::scope(|scope| {
                    let (reached_tx, reached_rx) = sync_channel(0);
                    let (continue_tx, continue_rx) = sync_channel(0);
                    let worker = scope.spawn(move || {
                        let mut a = [0.0; 8];
                        let mut b = [0.0; 8];
                        let report = callback
                            .render_inner(&mut a, &mut b, |stage| {
                                if matches!(
                                    (stage, at_fence),
                                    (Checkpoint::AfterDrain, false)
                                        | (Checkpoint::BeforeFence, true)
                                ) {
                                    reached_tx.send(()).unwrap();
                                    continue_rx.recv_timeout(GUARD).unwrap();
                                }
                            })
                            .unwrap();
                        (callback.stop(), a, b, report)
                    });
                    reached_rx.recv_timeout(GUARD).unwrap();
                    match action {
                        0 => owner.as_mut().unwrap().set_scene_running(false).unwrap(),
                        1 => owner.as_mut().unwrap().stop_audition().unwrap(),
                        2 => {
                            let mut next = origin();
                            next.navigation_revision += 1;
                            owner.as_mut().unwrap().invalidate_source(next).unwrap();
                        }
                        3 => owner.as_mut().unwrap().invalidate_source(origin()).unwrap(),
                        _ => drop(owner.take()),
                    }
                    assert!(weak.is_alive());
                    continue_tx.send(()).unwrap();
                    let (stopped, a, b, report) = worker.join().unwrap();
                    assert!(a.iter().enumerate().all(|(i, v)| *v
                        == if action == 1 && (!short || i < 2) {
                            0.25
                        } else {
                            0.0
                        }));
                    assert!(b.iter().enumerate().all(|(i, v)| *v
                        == if action == 0 && (!short || i < 2) {
                            0.25
                        } else {
                            0.0
                        }));
                    assert_eq!(
                        report.first.iter().flatten().count(),
                        usize::from(action < 2)
                    );
                    assert_eq!(report.deferred_returns, 0);
                    assert!(
                        weak.is_alive(),
                        "callback never releases last PCM even after owner drop"
                    );
                    if let Some(owner) = owner.as_mut() {
                        owner.finish(stopped).unwrap();
                    } else {
                        drop(stopped);
                    }
                });
                assert!(!weak.is_alive());
                assert_eq!(pool.usage().bytes, 0);
            }
        }
    }
}

#[test]
fn stopping_without_any_further_callback_reclaims_pending_active_and_returned_packets() {
    for phase in 0..3 {
        let pool = PcmBudget::default();
        let (mut owner, mut callback) = pair(&pool);
        let sound = ready(&pool, if phase == 2 { 1 } else { 100 });
        for _ in 0..MAX_VOICES {
            admit(&mut owner, Channel::Audition, &sound);
        }
        let weak = sound.pcm().downgrade();
        drop(sound);
        let stopped = std::thread::spawn(move || {
            if phase > 0 {
                callback.render_split(&mut [0.0; 2], &mut [0.0; 2]).unwrap();
            }
            callback.stop() // Executor relinquishes endpoint; no synthetic extra render.
        })
        .join()
        .unwrap();
        assert!(weak.is_alive());
        owner.finish(stopped).unwrap();
        assert_eq!(owner.outstanding(), 0);
        assert!(!weak.is_alive());
        assert_eq!(pool.usage().bytes, 0);
    }
}

#[test]
fn unexpected_consumer_destruction_is_detected_and_orphans_are_released_by_owner() {
    for phase in 0..3 {
        let pool = PcmBudget::default();
        let (mut owner, mut callback) = pair(&pool);
        let sound = ready(&pool, if phase == 2 { 1 } else { 100 });
        for _ in 0..MAX_VOICES {
            admit(&mut owner, Channel::Audition, &sound);
        }
        let weak = sound.pcm().downgrade();
        drop(sound);
        std::thread::spawn(move || {
            if phase > 0 {
                callback.render_split(&mut [0.0; 2], &mut [0.0; 2]).unwrap();
            }
            drop(callback); // After rendering, not inside it.
        })
        .join()
        .unwrap();
        assert!(weak.is_alive());
        assert_eq!(owner.collect(), MAX_VOICES);
        assert_eq!(owner.outstanding(), 0);
        assert!(!weak.is_alive());
        assert_eq!(pool.usage().bytes, 0);
        assert_eq!(owner.set_scene_running(true), Err(HandoffError::Closed));
    }
}

#[test]
fn foreign_stop_receipt_is_returned_intact_and_does_not_release_another_owners_credit() {
    let pool = PcmBudget::default();
    let (mut a, ca) = pair(&pool);
    let (mut b, cb) = pair(&pool);
    let sound = ready(&pool, 1);
    admit(&mut a, Channel::Audition, &sound);
    admit(&mut b, Channel::Audition, &sound);
    let foreign = a.finish(cb.stop()).unwrap_err();
    assert_eq!(a.outstanding(), 1);
    assert_eq!(b.outstanding(), 1);
    b.finish(foreign).unwrap();
    a.finish(ca.stop()).unwrap();
    drop(sound);
    assert_eq!(pool.usage().bytes, 0);
}

#[test]
fn invalid_output_blocks_never_consume_packets_or_advance_voice_position() {
    let pool = PcmBudget::default();
    let (mut owner, mut callback) = pair(&pool);
    let sound = ready(&pool, 2);
    let key = admit(&mut owner, Channel::Audition, &sound);
    let mut unchanged_a = [1.0; 2];
    let mut unchanged_b = [1.0; 4];
    assert!(matches!(
        callback.render_split(&mut unchanged_a, &mut unchanged_b),
        Err(HandoffError::InvalidBlock)
    ));
    assert_eq!(unchanged_a, [1.0; 2]);
    assert_eq!(unchanged_b, [1.0; 4]);
    assert!(matches!(
        callback.render_split(&mut [1.0], &mut [1.0]),
        Err(HandoffError::InvalidBlock)
    ));
    assert!(matches!(
        callback.render_split(&mut [1.0; 2], &mut [1.0; 4]),
        Err(HandoffError::InvalidBlock)
    ));
    assert!(matches!(
        callback.render_split(&mut [1.0; 4098], &mut [1.0; 4098]),
        Err(HandoffError::InvalidBlock)
    ));
    let report = callback.render_split(&mut [0.0; 2], &mut [0.0; 2]).unwrap();
    assert_eq!(report.first[key.slot], Some((key, Channel::Audition)));
    owner.finish(callback.stop()).unwrap();
}

#[test]
fn generation_exhaustion_fails_closed_or_rejects_without_aliasing_or_allocating() {
    let pool = PcmBudget::default();
    let (mut owner, callback) = pair(&pool);
    let sound = ready(&pool, 1);
    owner.serial = u64::MAX;
    assert_eq!(
        owner.admit(origin(), owner.token(Channel::Audition), &sound),
        Err(HandoffError::Exhausted)
    );
    assert_eq!(owner.outstanding(), 0);
    owner
        .shared
        .gate
        .store(1 | (EPOCH_MAX << EPOCH_BITS), Ordering::Release);
    assert_eq!(owner.stop_audition(), Err(HandoffError::Exhausted));
    assert_eq!(owner.set_scene_running(true), Err(HandoffError::Closed));
    owner.finish(callback.stop()).unwrap();
    let (mut owner, callback) = pair(&pool);
    let full = EPOCH_MAX | (1 << EPOCH_BITS);
    owner.shared.gate.store(full, Ordering::Release);
    let mut next = origin();
    next.navigation_revision += 1;
    assert_eq!(owner.invalidate_source(next), Err(HandoffError::Exhausted));
    assert_eq!(owner.shared.gate.load(Ordering::Acquire), full | CLOSED);
    assert_eq!(owner.source, origin()); // Neither lane nor source partially advances.
    owner.finish(callback.stop()).unwrap();
}

#[test]
fn concurrent_retirement_and_slot_reuse_do_not_alias_inflight_reports_or_new_pcm() {
    for invalidate in [false, true] {
        let pool = PcmBudget::default();
        let (mut owner, mut callback) = pair(&pool);
        let old = ready(&pool, 1);
        let weak = old.pcm().downgrade();
        let old_key = admit(&mut owner, Channel::Audition, &old);
        drop(old);
        std::thread::scope(|scope| {
            let (ready_tx, ready_rx) = sync_channel(0);
            let (go_tx, go_rx) = sync_channel(0);
            let worker = scope.spawn(move || {
                let first = callback
                    .render_inner(&mut [0.0; 2], &mut [0.0; 2], |stage| {
                        if matches!(stage, Checkpoint::AfterRetire) {
                            ready_tx.send(()).unwrap();
                            go_rx.recv_timeout(GUARD).unwrap();
                        }
                    })
                    .unwrap();
                let second = callback.render_split(&mut [0.0; 2], &mut [0.0; 2]).unwrap();
                (callback.stop(), first, second)
            });
            ready_rx.recv_timeout(GUARD).unwrap();
            assert_eq!(owner.collect(), 1);
            assert!(!weak.is_alive());
            if invalidate {
                let mut next = origin();
                next.navigation_revision += 1;
                owner.invalidate_source(next).unwrap();
            }
            let new = ready(&pool, 1);
            let new_key = admit(&mut owner, Channel::Audition, &new);
            drop(new);
            assert_eq!(old_key.slot, new_key.slot);
            assert_ne!(old_key, new_key);
            go_tx.send(()).unwrap();
            let (stopped, a, b) = worker.join().unwrap();
            assert_eq!(
                a.first[old_key.slot],
                if invalidate {
                    None
                } else {
                    Some((old_key, Channel::Audition))
                }
            );
            assert_eq!(b.first[new_key.slot], Some((new_key, Channel::Audition)));
            owner.finish(stopped).unwrap();
        });
        assert_eq!(pool.usage().bytes, 0);
    }
}

#[test]
fn three_hundred_threaded_rounds_keep_32_credits_and_release_each_shared_allocation() {
    let pool = PcmBudget::new(128, 128).unwrap();
    let (mut owner, mut callback) = pair(&pool);
    let (go_tx, go_rx) = sync_channel(0);
    let (done_tx, done_rx) = sync_channel(0);
    let worker = std::thread::spawn(move || {
        for _ in 0..300 {
            go_rx.recv_timeout(GUARD).unwrap();
            let mut a = [0.0; 960];
            let mut b = [0.0; 960];
            let report = callback.render_split(&mut a, &mut b).unwrap();
            assert!(a.iter().all(|v| *v == 0.0));
            assert!(b[..32].iter().all(|v| *v == 8.0));
            assert!(b[32..].iter().all(|v| *v == 0.0));
            assert_eq!(report.deferred_returns, 0);
            done_tx.send(report).unwrap();
        }
        callback.stop()
    });
    let mut generation = 0;
    for _ in 0..300 {
        let sound = ready(&pool, 16);
        assert!(pool.reserve(1).is_err()); // Ready handles must work at a completely full pool.
        let weak = sound.pcm().downgrade();
        for _ in 0..MAX_VOICES {
            let key = admit(&mut owner, Channel::Audition, &sound);
            assert!(key.generation > generation);
            generation = key.generation;
        }
        assert_eq!(
            owner.admit(origin(), owner.token(Channel::Audition), &sound),
            Err(HandoffError::Busy)
        );
        drop(sound);
        go_tx.send(()).unwrap();
        let report = done_rx.recv_timeout(GUARD).unwrap();
        assert_eq!(report.first.iter().flatten().count(), MAX_VOICES);
        assert_eq!(pool.usage().bytes, 128);
        assert!(weak.is_alive());
        assert_eq!(owner.collect(), MAX_VOICES);
        assert!(!weak.is_alive());
        assert_eq!(pool.usage().bytes, 0);
    }
    owner.finish(worker.join().unwrap()).unwrap();
    assert_eq!(pool.usage().peak_bytes, 128);
    println!(
        "HANDOFF_PROOF {{\"rounds\":300,\"voices\":9600,\"pcm_peak_bytes\":128,\"pcm_final_bytes\":0,\"deferred_returns\":0,\"capacity\":32,\"owner_inline_bytes\":{},\"consumer_state_bytes\":{},\"packet_bytes\":{}}}",
        std::mem::size_of::<VoiceOwner>(),
        std::mem::size_of::<VoiceConsumer>(),
        std::mem::size_of::<Packet>()
    );
}
