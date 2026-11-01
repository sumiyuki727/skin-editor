use super::*;
use crate::{Channel, Origin, PcmBudget, Runtime, RuntimeError, SoundPlan};
use skin_editor_domain::{AudioComponentId, AudioMetadata, BlobRef, Gain};

fn source() -> Origin {
    Origin {
        session: 1,
        node: 1,
        revision: 1,
        navigation_revision: 1,
        draft: None,
    }
}
fn prepared(pool: &PcmBudget, rate: u32, channels: u16, samples: &[f32]) -> PreparedSound {
    let plan = SoundPlan {
        target: AudioComponentId::new(1).unwrap().into(),
        source: BlobRef::new([1; 32], samples.len() as u64 * 4),
        metadata: Some(
            AudioMetadata::new(rate, channels, samples.len() as u64 / u64::from(channels)).unwrap(),
        ),
        gain: Gain::UNITY,
        visible: true,
    };
    PreparedSound::from_source(plan, samples, pool).unwrap()
}
fn runtime(pool: &PcmBudget) -> Runtime {
    let mut runtime = Runtime::with_budget(48000, 1, pool.clone()).unwrap();
    let sound = prepared(pool, 48000, 1, &[0.25; 4]);
    let plan = SoundPlan {
        target: AudioComponentId::new(1).unwrap().into(),
        source: BlobRef::new([1; 32], 16),
        metadata: Some(AudioMetadata::new(48000, 1, 4).unwrap()),
        gain: Gain::UNITY,
        visible: true,
    };
    let jobs = runtime.synchronize(source(), &[plan]).unwrap();
    runtime.complete(jobs[0], Ok(sound)).unwrap();
    runtime
}
fn trigger(runtime: &mut Runtime) {
    runtime
        .trigger(
            runtime.token(Channel::Audition),
            AudioComponentId::new(1).unwrap(),
            runtime.mix_frame(),
        )
        .unwrap();
}

#[test]
fn monitoring_sampler_has_independent_interpolation_channel_and_end_goldens() {
    let pool = PcmBudget::default();
    let mono = prepared(&pool, 44100, 1, &[0.0, 1.0, 0.0]);
    let output = MonitorFormat::new(48000, 2).unwrap();
    for (frame, expected) in [0.0, 0.91875, 0.1625, 0.0].into_iter().enumerate() {
        let values = mono.monitor_frame(frame as u64, output).unwrap();
        assert_eq!(values[0], values[1]);
        assert!((values[0] - expected).abs() < 0.0000001);
    }
    assert_eq!(mono.monitor_frame(4, output), None);
    assert_eq!(mono.monitor_frame(u64::MAX, output), None);
    let stereo = prepared(&pool, 48000, 2, &[0.75, -0.25, 0.5, -1.0]);
    let mono_output = MonitorFormat::new(48000, 1).unwrap();
    assert_eq!(stereo.monitor_frame(0, mono_output), Some([0.25, 0.0]));
    assert_eq!(stereo.monitor_frame(1, mono_output), Some([-0.25, 0.0]));
    assert_eq!(stereo.monitor_frame(2, mono_output), None);
    assert_eq!(MonitorFormat::new(0, 1), None);
    assert_eq!(MonitorFormat::new(48000, 3), None);
    let unknown_layout = prepared(&pool, 48000, 3, &[0.1, 0.2, 0.3]);
    assert_eq!(unknown_layout.monitor_frame(0, mono_output), None);
    assert!(unknown_layout.monitor_finished(0, mono_output));
}
#[test]
fn integer_rate_positions_keep_interpolation_bits_zero_sign_channel_mapping_and_end() {
    let pool = PcmBudget::default();
    let source = [
        -0.0f32,
        0.0,
        -1.0,
        1.0,
        0.25,
        -0.5,
        -f32::MIN_POSITIVE,
        f32::MIN_POSITIVE,
    ];
    for rate in [44100, 48000] {
        for channels in [1, 2] {
            for output_channels in [1, 2] {
                let sound = prepared(&pool, rate, channels, &source);
                let frames = source.len() / usize::from(channels);
                let format = MonitorFormat::new(rate, output_channels).unwrap();
                for frame in 0..frames {
                    // Integer-position golden retains the original interpolation arithmetic,
                    // including the sign of zero. This golden uses unity gain.
                    let sample = |channel: usize| {
                        let c = channel.min(usize::from(channels) - 1);
                        let a = sound.samples()[frame * usize::from(channels) + c];
                        let b = sound.samples()
                            [(frame + 1).min(frames - 1) * usize::from(channels) + c];
                        a + (b - a) * 0.0
                    };
                    let mut expected = [0.0f32; 2];
                    for (c, value) in expected
                        .iter_mut()
                        .enumerate()
                        .take(usize::from(output_channels))
                    {
                        *value = if output_channels == 1 && channels == 2 {
                            (sample(0) + sample(1)) * 0.5
                        } else {
                            sample(c)
                        };
                    }
                    assert_eq!(
                        sound
                            .monitor_frame(frame as u64, format)
                            .unwrap()
                            .map(f32::to_bits),
                        expected.map(f32::to_bits)
                    );
                    assert!(!sound.monitor_finished(frame as u64, format));
                }
                for frame in [frames as u64, frames as u64 + 1, u64::MAX] {
                    assert!(sound.monitor_finished(frame, format));
                    assert_eq!(sound.monitor_frame(frame, format), None);
                }
            }
        }
    }
}
#[test]
fn block_addition_matches_scalar_bits_across_formats_offsets_layouts_and_tail() {
    let pool = PcmBudget::default();
    for source_rate in [44100, 48000] {
        for target_rate in [44100, 48000] {
            for source_channels in [1, 2] {
                for target_channels in [1, 2] {
                    let samples = [
                        -0.0f32,
                        0.0,
                        -1.0,
                        1.0,
                        0.25,
                        -0.5,
                        -f32::MIN_POSITIVE,
                        f32::MIN_POSITIVE,
                    ];
                    let sound = prepared(&pool, source_rate, source_channels, &samples);
                    let format = MonitorFormat::new(target_rate, target_channels).unwrap();
                    for start in [0u64, 1, 3, 7, 8, u64::MAX] {
                        for frames in [0usize, 1, 3, 17] {
                            let mut output = vec![-0.0f32; frames * usize::from(target_channels)];
                            let mut expected = output.clone();
                            let mut count = 0;
                            for target in expected.chunks_exact_mut(usize::from(target_channels)) {
                                let Some(values) = start
                                    .checked_add(count as u64)
                                    .and_then(|p| sound.monitor_frame(p, format))
                                else {
                                    break;
                                };
                                for (c, value) in target.iter_mut().enumerate() {
                                    *value += values[c];
                                }
                                count += 1;
                            }
                            // A huge source cursor reports no added data, as scalar EOF does.
                            assert_eq!(
                                sound
                                    .add_monitor_frames(start, format, &mut output)
                                    .unwrap_or(0),
                                count
                            );
                            assert_eq!(
                                output.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
                                expected.iter().map(|v| v.to_bits()).collect::<Vec<_>>()
                            );
                        }
                    }
                }
            }
        }
    }
}
#[test]
fn invalid_block_layout_does_not_modify_output_or_allocate_pcm() {
    let pool = PcmBudget::default();
    let sound = prepared(&pool, 48000, 2, &[0.25; 4]);
    let before = pool.usage();
    let mut output = [0.7; 3];
    assert_eq!(
        sound.add_monitor_frames(0, MonitorFormat::new(48000, 2).unwrap(), &mut output),
        None
    );
    assert_eq!(output, [0.7; 3]);
    assert_eq!(pool.usage(), before);
}

#[test]
fn external_execution_is_explicit_and_never_silently_mixes_or_queues_future_events() {
    let pool = PcmBudget::default();
    let mut runtime = runtime(&pool);
    assert!(matches!(
        runtime.take_voice_handoff(),
        Err(RuntimeError::ExecutionMode)
    ));
    runtime.enable_voice_handoff().unwrap();
    assert_eq!(runtime.enable_voice_handoff(), Err(RuntimeError::Busy));
    assert_eq!(
        runtime.trigger(
            runtime.token(Channel::Audition),
            AudioComponentId::new(1).unwrap(),
            1
        ),
        Err(RuntimeError::UnsupportedScheduling)
    );
    assert_eq!(runtime.queued(), 0);
    let mut output = [7.0; 4];
    assert_eq!(runtime.mix(&mut output), Err(RuntimeError::ExecutionMode));
    assert_eq!(output, [7.0; 4]);
    assert_eq!(runtime.mix_frame(), 0);
    trigger(&mut runtime);
    let lease = runtime.take_voice_handoff().unwrap().unwrap();
    assert!(runtime.take_voice_handoff().unwrap().is_none());
    assert!(lease.validity().is_valid());
    assert_eq!(runtime.voices().count(), 1); // Credit stays authoritative until ack.
    assert!(runtime.finish_voice_handoff(&lease));
    assert!(lease.validity().is_valid()); // Completion is distinct from cancellation.
    assert!(!runtime.finish_voice_handoff(&lease));
}

#[test]
fn switching_an_active_owner_mixer_or_scheduled_queue_is_rejected_without_rewind() {
    for queued in [false, true] {
        let pool = PcmBudget::default();
        let mut runtime = runtime(&pool);
        runtime
            .trigger(
                runtime.token(Channel::Audition),
                AudioComponentId::new(1).unwrap(),
                u64::from(queued),
            )
            .unwrap();
        assert_eq!(runtime.enable_voice_handoff(), Err(RuntimeError::Busy));
        let mut output = [0.0; 2];
        runtime.mix(&mut output).unwrap();
        assert_eq!(output, if queued { [0.0, 0.25] } else { [0.25; 2] });
        assert_eq!(runtime.mix_frame(), 2);
        assert!(!runtime.voice_handoff_enabled());
    }
}

#[test]
fn foreign_and_retried_completion_cannot_retire_reused_numeric_voice_identity() {
    let pool = PcmBudget::default();
    let mut a = runtime(&pool);
    let mut b = runtime(&pool);
    a.enable_voice_handoff().unwrap();
    b.enable_voice_handoff().unwrap();
    trigger(&mut a);
    trigger(&mut b);
    let old = a.take_voice_handoff().unwrap().unwrap();
    let other = b.take_voice_handoff().unwrap().unwrap();
    assert_eq!(old.info().id, other.info().id);
    assert!(!b.finish_voice_handoff(&old));
    assert!(!b.retry_voice_handoff(&old));
    assert!(a.retry_voice_handoff(&old));
    assert!(!old.validity().is_valid());
    let fresh = a.take_voice_handoff().unwrap().unwrap();
    assert_eq!(old.info(), fresh.info());
    assert!(!a.finish_voice_handoff(&old));
    assert!(!a.retry_voice_handoff(&old));
    assert!(fresh.validity().is_valid());
    assert!(a.finish_voice_handoff(&fresh));
    assert!(other.validity().is_valid());
}

#[test]
fn runtime_destruction_revokes_export_but_actual_pcm_lease_outlives_it() {
    let pool = PcmBudget::default();
    let mut runtime = runtime(&pool);
    runtime.enable_voice_handoff().unwrap();
    trigger(&mut runtime);
    let lease = runtime.take_voice_handoff().unwrap().unwrap();
    let weak = lease.sound().pcm().downgrade();
    drop(runtime);
    assert!(!lease.validity().is_valid());
    assert!(weak.is_alive());
    assert_eq!(pool.usage().bytes, 16);
    drop(lease);
    assert!(!weak.is_alive());
    assert_eq!(pool.usage().bytes, 0);
}

#[test]
fn handoff_clock_rejects_stale_updates_and_allows_backward_seek_only_when_frozen() {
    let pool = PcmBudget::default();
    let mut runtime = runtime(&pool);
    assert_eq!(
        runtime.synchronize_handoff_clock(10, 5),
        Err(RuntimeError::ExecutionMode)
    );
    runtime.enable_voice_handoff().unwrap();
    runtime.synchronize_handoff_clock(80, 40).unwrap();
    runtime
        .set_scene_running(runtime.token(Channel::Scene), true)
        .unwrap();
    for (output, scene) in [(79, 50), (81, 39)] {
        assert_eq!(
            runtime.synchronize_handoff_clock(output, scene),
            Err(RuntimeError::Stale)
        );
        assert_eq!((runtime.mix_frame(), runtime.scene_frame()), (80, 40));
    }
    runtime
        .set_scene_running(runtime.token(Channel::Scene), false)
        .unwrap();
    runtime.synchronize_handoff_clock(90, 12).unwrap();
    assert_eq!((runtime.mix_frame(), runtime.scene_frame()), (90, 12));
    runtime.shutdown();
    assert_eq!(
        runtime.synchronize_handoff_clock(91, 13),
        Err(RuntimeError::Closed)
    );
}
