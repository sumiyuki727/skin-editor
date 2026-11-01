use super::*;
use skin_editor_application::values::{AudioComponentId, AudioMetadata, BlobRef, Gain};
use skin_editor_audio::{Origin, PcmBudget, PreparedSound, SoundPlan};
use std::sync::mpsc::sync_channel;
use std::time::Duration;
const GUARD: Duration = Duration::from_secs(10);
mod resume_race;
mod worker_fence;
#[test]
fn selective_cancel_at_the_output_copy_boundary_preserves_only_other_contributions() {
    let (pool, mut r, mut owner, mut cb) = setup(8192);
    add_sound(&mut r, &pool, origin(), 0.01);
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
            assert!(report.voices.first.iter().all(Option::is_none));
            assert!(report.voices.runtime_first.iter().all(Option::is_none));
            cb
        });
        rx.recv_timeout(GUARD).unwrap();
        r.stop_audition(r.token(Channel::Audition)).unwrap();
        go.send(()).unwrap();
        worker.join().unwrap()
    });
    owner.finish(&mut r, cb.stop()).unwrap();
    assert_eq!(pool.usage().bytes, 0);
}
fn origin() -> Origin {
    Origin {
        session: 1,
        node: 1,
        revision: 1,
        navigation_revision: 1,
        draft: None,
    }
}
fn sample(frame: u64) -> f32 {
    (frame % 1024) as f32 / 8192.0
}
fn reference(pool: &PcmBudget, frames: usize, rate: u32, channels: u16) -> BackgroundTrack {
    let mut pcm = pool
        .reserve(frames * usize::from(channels))
        .unwrap()
        .allocate();
    for (frame, values) in pcm
        .samples_mut()
        .unwrap()
        .chunks_exact_mut(usize::from(channels))
        .enumerate()
    {
        values.fill(sample(frame as u64));
    }
    BackgroundTrack::new(pcm, rate, channels).unwrap()
}
fn runtime(pool: &PcmBudget) -> Runtime {
    let mut r = Runtime::with_budget(48000, 2, pool.clone()).unwrap();
    r.enable_voice_handoff().unwrap();
    r
}
fn setup(frames: usize) -> (PcmBudget, Runtime, BackgroundOwner, Box<OutputConsumer>) {
    let pool = PcmBudget::default();
    let mut r = runtime(&pool);
    let track = reference(&pool, frames, 48000, 2);
    let (owner, consumer) = boundary(&mut r, track, origin()).unwrap();
    (pool, r, owner, consumer)
}
fn add_sound(r: &mut Runtime, pool: &PcmBudget, source: Origin, value: f32) -> SoundPlan {
    let p = SoundPlan {
        target: AudioComponentId::new(1).unwrap().into(),
        source: BlobRef::new([source.revision as u8; 32], 16),
        metadata: Some(AudioMetadata::new(48000, 2, 2).unwrap()),
        gain: Gain::UNITY,
        visible: true,
    };
    let job = r.synchronize(source, &[p]).unwrap()[0];
    r.complete(
        job,
        Ok(PreparedSound::from_source(p, &[value; 4], pool).unwrap()),
    )
    .unwrap();
    p
}
fn trigger(r: &mut Runtime, channel: Channel) {
    r.trigger(
        r.token(channel),
        AudioComponentId::new(1).unwrap(),
        r.mix_frame(),
    )
    .unwrap();
}
fn assert_background(values: &[f32], start: u64) {
    for (i, frame) in values.chunks_exact(2).enumerate() {
        assert_eq!(frame, [sample(start + i as u64); 2]);
    }
}
fn start(owner: &mut BackgroundOwner, r: &mut Runtime) {
    owner.refill().unwrap();
    owner.resume(r).unwrap();
}

#[test]
fn reserve_frontier_and_copy_clock_are_separate_and_do_not_need_gui_ticks() {
    let (pool, mut r, mut owner, mut cb) = setup(10000);
    assert_eq!(owner.resume(&mut r), Err(OutputError::NotPrimed));
    start(&mut owner, &mut r);
    assert_eq!(owner.position().generated_frame, 2048);
    assert_eq!(r.scene_frame(), 0);
    for index in 0..3 {
        let mut out = [0.0; 960];
        let report = cb.render(&mut out).unwrap();
        assert_background(&out, index * 480);
        assert_eq!(report.missing_frames, 0);
    }
    owner.pump(&mut r).unwrap();
    assert_eq!(r.scene_frame(), 1440);
    assert_eq!(r.mix_frame(), 1440);
    assert_eq!(owner.position().generated_frame, 2048);
    assert_eq!(owner.refill(), Ok(1440));
    assert_eq!(owner.position().generated_frame, 3488);
    owner.finish(&mut r, cb.stop()).unwrap();
    assert_eq!(pool.usage().bytes, 0);
}

#[test]
fn one_refill_turn_is_bounded_even_if_consumer_frees_every_published_slot() {
    let (pool, mut r, mut owner, mut cb) = setup(10000);
    start(&mut owner, &mut r);
    cb.render(&mut [0.0; 4096]).unwrap();
    owner.pump(&mut r).unwrap();
    let mut consumed = 0;
    let count = owner
        .refill_inner(|| {
            let mut out = [0.0; 2];
            let report = cb.render(&mut out).unwrap();
            assert_eq!(report.missing_frames, 0);
            assert_background(&out, 2048 + consumed);
            consumed += 1;
        })
        .unwrap();
    assert_eq!(count, BACKGROUND_RESERVE);
    assert_eq!(consumed, 2048);
    assert_eq!(owner.position().generated_frame, 4096);
    owner.finish(&mut r, cb.stop()).unwrap();
    assert_eq!(pool.usage().bytes, 0);
}

#[test]
fn matching_reference_formats_keep_frame_units_and_exact_managed_storage_cost() {
    for rate in [44100, 48000] {
        for channels in [1, 2] {
            let pool = PcmBudget::default();
            let mut r = Runtime::with_budget(rate, channels, pool.clone()).unwrap();
            r.enable_voice_handoff().unwrap();
            let track = reference(&pool, 4096, rate, channels);
            let (mut owner, mut cb) = boundary(&mut r, track, origin()).unwrap();
            let samples = usize::from(channels);
            assert_eq!(
                pool.usage().bytes,
                4096 * samples * 4 + 2048 * 2 * 4 + 3 * 2048 * samples * 4
            );
            start(&mut owner, &mut r);
            let mut out = vec![0.0; 16 * samples];
            let report = cb.render(&mut out).unwrap();
            for (i, frame) in out.chunks_exact(samples).enumerate() {
                assert!(frame.iter().all(|v| *v == sample(i as u64)));
            }
            assert_eq!(report.missing_frames, 0);
            owner.pump(&mut r).unwrap();
            assert_eq!((r.mix_frame(), r.scene_frame()), (16, 16));
            owner.finish(&mut r, cb.stop()).unwrap();
            assert_eq!(pool.usage().bytes, 0);
        }
    }
}

#[test]
fn ready_voice_bypasses_a_full_background_reserve_before_and_after_refill() {
    for trigger_first in [false, true] {
        let (pool, mut r, mut owner, mut cb) = setup(10000);
        add_sound(&mut r, &pool, origin(), 0.25);
        start(&mut owner, &mut r);
        cb.render(&mut [0.0; 960]).unwrap();
        owner.pump(&mut r).unwrap();
        if trigger_first {
            trigger(&mut r, Channel::Scene);
            owner.pump(&mut r).unwrap();
        }
        owner.refill().unwrap();
        if !trigger_first {
            trigger(&mut r, Channel::Scene);
            owner.pump(&mut r).unwrap();
        }
        let mut out = [0.0; 960];
        let report = cb.render(&mut out).unwrap();
        assert_eq!(report.missing_frames, 0);
        assert_eq!(out[0], sample(480) + 0.25);
        assert_eq!(out[4], sample(482));
        assert_eq!(report.voices.first.iter().flatten().count(), 1);
        owner.finish(&mut r, cb.stop()).unwrap();
        assert_eq!(pool.usage().bytes, 0);
    }
}

#[test]
fn full_32_voice_load_shares_pcm_and_mixes_with_background_without_queue_delay() {
    let (pool, mut r, mut owner, mut cb) = setup(8192);
    add_sound(&mut r, &pool, origin(), 0.01);
    start(&mut owner, &mut r);
    let before = pool.usage().bytes;
    for _ in 0..16 {
        trigger(&mut r, Channel::Scene);
        trigger(&mut r, Channel::Audition);
    }
    assert_eq!(
        r.trigger(
            r.token(Channel::Audition),
            AudioComponentId::new(1).unwrap(),
            r.mix_frame()
        ),
        Err(skin_editor_audio::RuntimeError::ResourceLimit)
    );
    assert_eq!(owner.pump(&mut r).unwrap().voices.admitted, 32);
    assert_eq!(pool.usage().bytes, before);
    let mut out = [0.0; 960];
    let report = cb.render(&mut out).unwrap();
    assert_eq!(report.missing_frames, 0);
    assert_eq!(report.voices.first.iter().flatten().count(), 32);
    assert!((out[0] - 0.32).abs() < 0.0000001);
    assert_eq!(out[4], sample(2));
    assert_eq!(owner.pump(&mut r).unwrap().voices.retired, 32);
    assert_eq!(r.voices().count(), 0);
    assert_eq!(pool.usage().bytes, before);
    owner.finish(&mut r, cb.stop()).unwrap();
    assert_eq!(pool.usage().bytes, 0);
}

#[test]
fn pause_copy_interleavings_restore_only_unsent_background_and_keep_audition() {
    for phase in 0..3 {
        let (pool, mut r, mut owner, mut cb) = setup(8192);
        add_sound(&mut r, &pool, origin(), 0.125);
        start(&mut owner, &mut r);
        trigger(&mut r, Channel::Scene);
        trigger(&mut r, Channel::Audition);
        owner.pump(&mut r).unwrap();
        let (cb_back, report, out) = std::thread::scope(|scope| {
            let (tx, rx) = sync_channel(0);
            let (go, wait) = sync_channel(0);
            let worker = scope.spawn(move || {
                let mut out = [0.0; 256];
                let report = cb
                    .render_inner(&mut out, |stage| {
                        if matches!(
                            (stage, phase),
                            (Checkpoint::AfterBackground, 0)
                                | (Checkpoint::BeforeCopy, 1)
                                | (Checkpoint::AfterCopy, 2)
                        ) {
                            tx.send(()).unwrap();
                            wait.recv_timeout(GUARD).unwrap();
                        }
                    })
                    .unwrap();
                (cb, report, out)
            });
            rx.recv_timeout(GUARD).unwrap();
            owner.pause(&mut r).unwrap();
            assert_eq!(owner.resume(&mut r), Err(OutputError::AwaitingAck));
            assert_eq!(owner.refill(), Err(OutputError::AwaitingAck));
            go.send(()).unwrap();
            worker.join().unwrap()
        });
        cb = cb_back;
        let submitted = if phase == 2 { 128 } else { 0 };
        assert_eq!(report.clock.next_background, submitted);
        assert_eq!(report.missing_frames, 0);
        assert_eq!(report.background_frames, submitted as usize);
        if phase == 2 {
            assert_eq!(out[0], 0.25);
        } else {
            assert_eq!(out[0], 0.125);
            assert!(out[4..].iter().all(|v| *v == 0.0));
        }
        assert!(owner.pump(&mut r).unwrap().freeze_completed);
        assert_eq!(owner.position().generated_frame, submitted);
        assert_eq!(r.scene_frame(), submitted);
        assert_eq!(owner.resume(&mut r), Err(OutputError::NotPrimed));
        owner.refill().unwrap();
        let generated = owner.position().generated_frame;
        assert!(!owner.pump(&mut r).unwrap().freeze_completed);
        assert_eq!(owner.position().generated_frame, generated);
        owner.resume(&mut r).unwrap();
        let mut out = [0.0; 64];
        cb.render(&mut out).unwrap();
        assert_background(&out, submitted);
        owner.pump(&mut r).unwrap();
        assert_eq!(r.scene_frame(), submitted + 32);
        owner.finish(&mut r, cb.stop()).unwrap();
        assert_eq!(pool.usage().bytes, 0);
    }
}

#[test]
fn seeks_forward_backward_and_latest_request_clear_old_frames_and_stay_frozen() {
    let (pool, mut r, mut owner, mut cb) = setup(10000);
    start(&mut owner, &mut r);
    cb.render(&mut [0.0; 960]).unwrap();
    owner.pump(&mut r).unwrap();
    for target in [6000, 100, 8000, 0] {
        owner.seek(&mut r, target).unwrap();
        assert_eq!(owner.resume(&mut r), Err(OutputError::AwaitingAck));
        let mut silent = [1.0; 64];
        cb.render(&mut silent).unwrap();
        assert_eq!(silent, [0.0; 64]);
        assert!(owner.pump(&mut r).unwrap().freeze_completed);
        assert_eq!(owner.position().phase, Phase::Frozen);
        assert_eq!(r.scene_frame(), target);
        owner.refill().unwrap();
        cb.render(&mut [0.0; 64]).unwrap();
        owner.pump(&mut r).unwrap();
        assert_eq!(r.scene_frame(), target); // A frozen old copy anchor cannot undo seek.
        owner.resume(&mut r).unwrap();
        let mut out = [0.0; 64];
        cb.render(&mut out).unwrap();
        assert_background(&out, target);
        owner.pump(&mut r).unwrap();
        assert_eq!(r.scene_frame(), target + 32);
    }
    owner.seek(&mut r, 500).unwrap();
    cb.render(&mut [0.0; 2]).unwrap(); // Old ack.
    owner.seek(&mut r, 700).unwrap();
    assert!(!owner.pump(&mut r).unwrap().freeze_completed);
    assert_eq!(owner.resume(&mut r), Err(OutputError::AwaitingAck));
    cb.render(&mut [0.0; 2]).unwrap();
    assert!(owner.pump(&mut r).unwrap().freeze_completed);
    assert_eq!(r.scene_frame(), 700);
    owner.finish(&mut r, cb.stop()).unwrap();
    assert_eq!(pool.usage().bytes, 0);
}

#[test]
fn hot_replacement_draft_cancel_and_history_never_reset_background_generation() {
    let (pool, mut r, mut owner, mut cb) = setup(10000);
    add_sound(&mut r, &pool, origin(), 0.125);
    start(&mut owner, &mut r);
    cb.render(&mut [0.0; 960]).unwrap();
    owner.pump(&mut r).unwrap();
    let epoch = owner.position().epoch;
    let generated = owner.position().generated_frame;
    for (i, source) in [
        Origin {
            revision: 2,
            node: 2,
            ..origin()
        },
        Origin {
            revision: 3,
            draft: Some((7, 1)),
            ..origin()
        },
        Origin {
            revision: 4,
            navigation_revision: 2,
            ..origin()
        },
    ]
    .into_iter()
    .enumerate()
    {
        add_sound(&mut r, &pool, source, 0.25);
        trigger(&mut r, Channel::Scene);
        owner.pump(&mut r).unwrap();
        if i == 1 {
            r.invalidate_drafts(&[7]);
        }
        let mut out = [0.0; 64];
        let report = cb.render(&mut out).unwrap();
        let first = 480 + i as u64 * 32;
        assert_eq!(out[0], sample(first) + if i == 1 { 0.0 } else { 0.25 });
        assert_eq!(report.missing_frames, 0);
        owner.pump(&mut r).unwrap();
        assert_eq!(owner.position().epoch, epoch);
        assert_eq!(owner.position().generated_frame, generated);
        assert_eq!(r.scene_frame(), first + 32);
    }
    owner.finish(&mut r, cb.stop()).unwrap();
    assert_eq!(pool.usage().bytes, 0);
}

#[test]
fn frozen_seek_does_not_stop_explicit_audition_or_admit_scene_input() {
    let (pool, mut r, mut owner, mut cb) = setup(8192);
    add_sound(&mut r, &pool, origin(), 0.25);
    start(&mut owner, &mut r);
    owner.seek(&mut r, 1234).unwrap();
    trigger(&mut r, Channel::Audition);
    owner.pump(&mut r).unwrap();
    assert_eq!(
        r.trigger(
            r.token(Channel::Scene),
            AudioComponentId::new(1).unwrap(),
            r.mix_frame()
        ),
        Err(skin_editor_audio::RuntimeError::Frozen)
    );
    let mut out = [0.0; 16];
    let report = cb.render(&mut out).unwrap();
    assert_eq!(out[..4], [0.25; 4]);
    assert_eq!(report.missing_frames, 0);
    owner.pump(&mut r).unwrap();
    assert_eq!(r.scene_frame(), 1234);
    assert_eq!(r.mix_frame(), 8);
    owner.finish(&mut r, cb.stop()).unwrap();
    assert_eq!(pool.usage().bytes, 0);
}

#[test]
fn invalid_input_is_rejected_before_mutating_output_queue_or_transport() {
    let (pool, mut r, mut owner, mut cb) = setup(8192);
    start(&mut owner, &mut r);
    let before = owner.position();
    let mut out = [7.0; 3];
    assert!(matches!(
        cb.render(&mut out),
        Err(OutputError::InvalidBlock)
    ));
    assert_eq!(out, [7.0; 3]);
    assert!(matches!(
        cb.render(&mut vec![7.0; 4098]),
        Err(OutputError::InvalidBlock)
    ));
    assert_eq!(owner.seek(&mut r, 8193), Err(OutputError::InvalidPosition));
    assert_eq!(owner.position().epoch, before.epoch);
    assert_eq!(owner.position().queued, before.queued);
    assert_eq!(r.scene_frame(), 0);
    let mut other = runtime(&pool);
    assert!(matches!(
        owner.pump(&mut other),
        Err(OutputError::WrongRuntime)
    ));
    let stopped = owner.finish(&mut other, cb.stop()).unwrap_err();
    assert_eq!(owner.position().phase, Phase::Running);
    owner.finish(&mut r, stopped).unwrap();
    assert_eq!(pool.usage().bytes, 0);
}

#[test]
fn end_of_reference_is_intentional_silence_but_exhausted_reserve_is_counted() {
    for total in [3, 8192] {
        let (pool, mut r, mut owner, mut cb) = setup(total);
        start(&mut owner, &mut r);
        let report = cb.render(&mut [0.0; 4096]).unwrap();
        assert_eq!(report.missing_frames, 0);
        let report = cb.render(&mut [0.0; 2]).unwrap();
        assert_eq!(report.missing_frames, usize::from(total != 3));
        assert_eq!(
            report.clock.next_background,
            if total == 3 { 3 } else { 2048 }
        );
        owner.finish(&mut r, cb.stop()).unwrap();
        assert_eq!(pool.usage().bytes, 0);
    }
}

#[test]
fn construction_failure_releases_all_reserved_storage_and_refuses_wrong_source() {
    for limit in [64, 20000, 40000, 60000] {
        let pool = PcmBudget::new(limit, limit).unwrap();
        let mut r = runtime(&pool);
        assert!(matches!(
            boundary(&mut r, reference(&pool, 1, 48000, 2), origin()),
            Err(OutputError::ResourceLimit)
        ));
        assert_eq!(pool.usage().bytes, 0);
        assert!(!r.scene_running());
    }
    let pool = PcmBudget::default();
    let mut r = runtime(&pool);
    assert!(matches!(
        boundary(&mut r, reference(&pool, 1, 44100, 2), origin()),
        Err(OutputError::WrongFormat)
    ));
    let other = PcmBudget::default();
    assert!(matches!(
        boundary(&mut r, reference(&other, 1, 48000, 2), origin()),
        Err(OutputError::WrongPool)
    ));
    assert_eq!(pool.usage().bytes, 0);
    assert_eq!(other.usage().bytes, 0);
}

#[test]
fn shutdown_without_a_later_render_releases_pending_active_and_returned_resources() {
    for stage in 0..3 {
        let (pool, mut r, mut owner, mut cb) = setup(8192);
        add_sound(&mut r, &pool, origin(), 0.25);
        start(&mut owner, &mut r);
        trigger(&mut r, Channel::Scene);
        owner.pump(&mut r).unwrap();
        if stage > 0 {
            cb.render(&mut vec![0.0; if stage == 1 { 2 } else { 4 }])
                .unwrap();
        }
        owner.pause(&mut r).unwrap(); // No pause callback follows.
        owner.finish(&mut r, cb.stop()).unwrap();
        assert_eq!(pool.usage().bytes, 0);
    }
}

#[test]
fn owner_drop_does_not_refund_still_live_ring_and_consumer_scratch() {
    let (pool, mut r, mut owner, mut cb) = setup(8192);
    start(&mut owner, &mut r);
    drop(owner);
    assert_eq!(pool.usage().bytes, 65536);
    let mut out = [1.0; 16];
    cb.render(&mut out).unwrap();
    assert_eq!(out, [0.0; 16]);
    assert_eq!(pool.usage().bytes, 65536);
    drop(cb);
    r.shutdown();
    assert_eq!(pool.usage().bytes, 0);
}

#[test]
fn three_hundred_threaded_blocks_keep_clock_and_shared_resources_bounded() {
    let (pool, mut r, mut owner, cb) = setup(20000);
    start(&mut owner, &mut r);
    let cb = std::thread::scope(|scope| {
        let (step, steps) = sync_channel(0);
        let (reply, results) = sync_channel(0);
        let worker = scope.spawn(move || {
            let mut cb = cb;
            for _ in 0..300 {
                steps.recv_timeout(GUARD).unwrap();
                let mut out = [0.0; 64];
                let report = cb.render(&mut out).unwrap();
                reply.send((report, out)).unwrap();
            }
            cb
        });
        for round in 0..300 {
            owner.refill().unwrap();
            step.send(()).unwrap();
            let (report, out) = results.recv_timeout(GUARD).unwrap();
            assert_eq!(report.missing_frames, 0);
            assert_background(&out, round * 32);
            owner.pump(&mut r).unwrap();
            assert_eq!(r.scene_frame(), (round + 1) * 32);
        }
        worker.join().unwrap()
    });
    let peak = pool.usage().peak_bytes;
    owner.finish(&mut r, cb.stop()).unwrap();
    assert_eq!(pool.usage().bytes, 0);
    println!(
        "BACKGROUND_COORDINATION_PROOF threaded_blocks=300 submitted_frames=9600 missing_frames=0 pcm_peak_bytes={peak} pcm_final_bytes=0 reserve_frames=2048"
    );
}

#[test]
fn counter_exhaustion_closes_without_wrapping_and_partial_clock_rows_are_rejected() {
    let (pool, mut r, mut owner, mut cb) = setup(8192);
    owner.shared.clock.stamp.store(1, SeqCst);
    assert_eq!(owner.shared.clock.read(), None);
    owner.shared.clock.stamp.store(0, SeqCst);
    cb.serial = u64::MAX - 1;
    let mut out = [7.0; 2];
    assert!(matches!(
        cb.render(&mut out),
        Err(OutputError::ResourceLimit)
    ));
    assert_eq!(out, [7.0; 2]);
    assert_eq!(owner.refill(), Err(OutputError::Closed));
    owner.finish(&mut r, cb.stop()).unwrap();
    assert_eq!(pool.usage().bytes, 0);
    let (pool, mut r, mut owner, cb) = setup(8192);
    owner.generation = EPOCH_MAX;
    assert_eq!(owner.seek(&mut r, 1), Err(OutputError::ResourceLimit));
    assert_eq!(owner.position().phase, Phase::Closed);
    owner.finish(&mut r, cb.stop()).unwrap();
    assert_eq!(pool.usage().bytes, 0);
}
