use skin_editor_application::{values::*, *};
use skin_editor_audio::*;
use skin_editor_infrastructure::*;
use std::sync::mpsc;

fn wave(rate: u32, channels: u16, samples: &[i16]) -> Vec<u8> {
    let size = (samples.len() * 2) as u32;
    let mut bytes = Vec::new();
    bytes.extend(b"RIFF");
    bytes.extend((36 + size).to_le_bytes());
    bytes.extend(b"WAVEfmt ");
    bytes.extend(16u32.to_le_bytes());
    bytes.extend(1u16.to_le_bytes());
    bytes.extend(channels.to_le_bytes());
    bytes.extend(rate.to_le_bytes());
    bytes.extend((rate * u32::from(channels) * 2).to_le_bytes());
    bytes.extend((channels * 2).to_le_bytes());
    bytes.extend(16u16.to_le_bytes());
    bytes.extend(b"data");
    bytes.extend(size.to_le_bytes());
    for sample in samples {
        bytes.extend(sample.to_le_bytes());
    }
    bytes
}
fn origin(revision: u64) -> Origin {
    Origin {
        session: 1,
        node: revision + 1,
        revision,
        navigation_revision: 0,
        draft: None,
    }
}
fn id() -> AudioComponentId {
    AudioComponentId::new(1).unwrap()
}
fn plan(raw: &RawBlob) -> SoundPlan {
    SoundPlan {
        target: (id()).into(),
        source: raw.reference(),
        metadata: raw.audio_metadata(),
        gain: Gain::UNITY,
        visible: true,
    }
}
fn prepared(runtime: &Runtime, raw: &RawBlob, job: BuildRequest) -> PreparedSound {
    let mut queue = AudioPreparationQueue::new(runtime.pcm_budget());
    queue.submit(job, raw.clone()).unwrap();
    let done = queue.start_next().unwrap().execute(&mut WavDecoder);
    queue.finish(done).unwrap().1.unwrap()
}
#[test]
fn real_wav_minimum_matrix_decodes_signed_samples_and_preserves_original_bytes() {
    for rate in [44100, 48000] {
        for channels in [1, 2] {
            let original = wave(rate, channels, &[-32768, -16384, 0, 16384, 32767, 0]);
            let store = BlobStore::default();
            let raw = store.stage_audio(original.clone(), &WavDecoder).unwrap();
            assert_eq!(
                raw.audio_metadata().unwrap(),
                AudioMetadata::new(rate, channels, 6 / u64::from(channels)).unwrap()
            );
            let mut runtime = Runtime::new(rate, channels).unwrap();
            let job = runtime.synchronize(origin(0), &[plan(&raw)]).unwrap()[0];
            let sound = prepared(&runtime, &raw, job);
            assert_eq!(
                sound.samples(),
                [-1.0, -0.5, 0.0, 0.5, 32767.0 / 32768.0, 0.0]
            );
            assert_eq!(raw.bytes(), original);
        }
    }
}
#[test]
fn corrupt_and_unsupported_wav_never_publish_partial_resources() {
    let good = wave(48000, 1, &[1, 2, 3, 4]);
    let store = BlobStore::default();
    let mut cases = vec![vec![], b"not a wave".to_vec()];
    for index in [4, 16, 20, 22, 24, 28, 32, 34, 40] {
        let mut bad = good.clone();
        bad[index] ^= 7;
        cases.push(bad);
    }
    let mut trailing = good.clone();
    trailing.push(0);
    cases.push(trailing);
    let mut truncated = good.clone();
    truncated.pop();
    cases.push(truncated);
    for bytes in cases {
        assert!(store.stage_audio(bytes, &WavDecoder).is_err());
        assert_eq!(store.usage().unwrap().bytes, 0);
    }
}
#[test]
fn riff_unknown_padded_chunks_are_skipped_but_duplicate_data_is_rejected() {
    let mut bytes = wave(48000, 1, &[1, 2]);
    bytes.splice(12..12, *b"JUNK\x01\0\0\0a\0");
    let size = (bytes.len() - 8) as u32;
    bytes[4..8].copy_from_slice(&size.to_le_bytes());
    assert_eq!(WavDecoder.probe(&bytes).unwrap().frames(), 2);
    bytes.extend(b"data\0\0\0\0");
    let size = (bytes.len() - 8) as u32;
    bytes[4..8].copy_from_slice(&size.to_le_bytes());
    assert!(WavDecoder.probe(&bytes).is_err());
}
#[test]
fn minimum_monitor_conversion_is_explicit_and_keeps_source_pcm_in_its_budget() {
    let store = BlobStore::default();
    for rate in [44100, 48000] {
        for channels in [1, 2] {
            for output_rate in [44100, 48000] {
                for output_channels in [1, 2] {
                    let input = vec![8192i16; 441 * usize::from(channels)];
                    let raw = store
                        .stage_audio(wave(rate, channels, &input), &WavDecoder)
                        .unwrap();
                    let mut runtime = Runtime::new(output_rate, output_channels).unwrap();
                    runtime.enable_minimum_conversion();
                    let job = runtime.synchronize(origin(0), &[plan(&raw)]).unwrap()[0];
                    let sound = prepared(&runtime, &raw, job);
                    assert_eq!(sound.samples().len(), input.len());
                    runtime.complete(job, Ok(sound)).unwrap();
                    runtime
                        .trigger(runtime.token(Channel::Audition), id(), 0)
                        .unwrap();
                    let frames =
                        (441u64 * u64::from(output_rate)).div_ceil(u64::from(rate)) as usize;
                    let mut output = vec![0.0; (frames + 1) * usize::from(output_channels)];
                    runtime.mix(&mut output).unwrap();
                    assert!(
                        output[..frames * usize::from(output_channels)]
                            .iter()
                            .all(|n| *n == 0.25)
                    );
                    assert!(
                        output[frames * usize::from(output_channels)..]
                            .iter()
                            .all(|n| *n == 0.0)
                    );
                    assert_eq!(runtime.voices().count(), 0);
                }
            }
        }
    }
}
#[test]
fn native_consumer_fences_scene_without_cutting_audition_and_releases_ring_budget() {
    let budget = PcmBudget::default();
    let store = BlobStore::default();
    let raw = store
        .stage_audio(wave(48000, 1, &vec![4096; 2048]), &WavDecoder)
        .unwrap();
    let mut runtime = Runtime::with_budget(48000, 1, budget.clone()).unwrap();
    let job = runtime.synchronize(origin(0), &[plan(&raw)]).unwrap()[0];
    let sound = prepared(&runtime, &raw, job);
    runtime.complete(job, Ok(sound)).unwrap();
    let (mut output, mut callback) = output_boundary(&budget, 48000, 1).unwrap();
    runtime
        .set_scene_running(runtime.token(Channel::Scene), true)
        .unwrap();
    runtime
        .trigger(runtime.token(Channel::Scene), id(), 0)
        .unwrap();
    runtime
        .trigger(runtime.token(Channel::Audition), id(), 0)
        .unwrap();
    output.pump(&mut runtime).unwrap();
    let before = runtime.scene_frame();
    runtime
        .set_scene_running(runtime.token(Channel::Scene), false)
        .unwrap();
    let fence = output.invalidate(true, false).unwrap();
    assert!(!output.acknowledged(fence));
    let mut samples = [0.0; 128];
    callback.render_f32(&mut samples);
    assert_eq!(samples, [0.125; 128]);
    assert!(output.acknowledged(fence));
    output.pump(&mut runtime).unwrap();
    assert_eq!(runtime.scene_frame(), before);
    runtime
        .stop_audition(runtime.token(Channel::Audition))
        .unwrap();
    output.invalidate(false, true).unwrap();
    callback.render_f32(&mut samples);
    assert_eq!(samples, [0.0; 128]);
    runtime
        .set_scene_running(runtime.token(Channel::Scene), true)
        .unwrap();
    output.invalidate(true, false).unwrap();
    callback.render_f32(&mut [0.0; OUTPUT_QUEUE_FRAMES]);
    output.pump(&mut runtime).unwrap();
    callback.render_f32(&mut samples);
    assert_eq!(samples, [0.0; 128]);
    runtime.shutdown();
    drop(runtime);
    drop(output);
    assert!(budget.usage().bytes > 0); // consumer still owns the ring payload lease
    drop(callback);
    assert_eq!(budget.usage().bytes, 0);
}
#[test]
fn output_backpressure_does_not_advance_mix_and_underrun_outputs_explicit_silence() {
    let budget = PcmBudget::default();
    let mut runtime = Runtime::with_budget(48000, 2, budget.clone()).unwrap();
    let (mut output, mut callback) = output_boundary(&budget, 48000, 2).unwrap();
    assert_eq!(output.pump(&mut runtime).unwrap(), OUTPUT_AHEAD_FRAMES);
    let frame = runtime.mix_frame();
    assert_eq!(output.pump(&mut runtime).unwrap(), 0);
    assert_eq!(runtime.mix_frame(), frame);
    let mut samples = [1.0; 2048];
    callback.render_f32(&mut samples);
    assert_eq!(samples, [0.0; 2048]);
    assert_eq!(
        output.stats().empty_frames,
        1024 - OUTPUT_AHEAD_FRAMES as u64
    );
}
#[test]
fn output_device_fault_mutes_prequeued_samples_and_requires_explicit_rebuild() {
    let (session, store) = project();
    let budget = PcmBudget::default();
    let (mut output, mut callback) = output_boundary(&budget, 48000, 1).unwrap();
    let mut audio = AudioPlayback::new(48000, 1, budget.clone()).unwrap();
    audio.step(&session, &store, &mut output).unwrap();
    assert!(
        audio
            .wait_preparation(&session, &output, std::time::Duration::from_secs(5))
            .unwrap()
    );
    callback.render_f32(&mut [0.0; 512]);
    audio
        .trigger(&session, &output, Channel::Audition, id())
        .unwrap();
    audio.step(&session, &store, &mut output).unwrap();
    output.fail_device();
    let mut samples = [1.0; 128];
    callback.render_f32(&mut samples);
    assert_eq!(samples, [0.0; 128]);
    assert!(audio.step(&session, &store, &mut output).is_err());
    drop(audio);
    drop(output);
    drop(callback);
    assert_eq!(budget.usage().bytes, 0);
}
#[test]
fn dropping_playback_mutes_copies_even_if_native_output_outlives_the_control_owner() {
    let (session, store) = project();
    let budget = PcmBudget::default();
    let (mut output, mut callback) = output_boundary(&budget, 48000, 1).unwrap();
    let mut audio = AudioPlayback::new(48000, 1, budget.clone()).unwrap();
    audio.step(&session, &store, &mut output).unwrap();
    assert!(
        audio
            .wait_preparation(&session, &output, std::time::Duration::from_secs(5))
            .unwrap()
    );
    callback.render_f32(&mut [0.0; OUTPUT_QUEUE_FRAMES]);
    audio
        .trigger(&session, &output, Channel::Audition, id())
        .unwrap();
    audio.step(&session, &store, &mut output).unwrap();
    let mut samples = [0.0; 128];
    callback.render_f32(&mut samples);
    assert_eq!(samples, [0.25; 128]);
    drop(audio);
    callback.render_f32(&mut samples);
    assert_eq!(samples, [0.0; 128]);
    drop(output);
    drop(callback);
    assert_eq!(budget.usage().bytes, 0);
}
#[test]
fn native_boundary_combines_unclipped_lanes_before_saturation() {
    let budget = PcmBudget::default();
    let store = BlobStore::default();
    let positive = store
        .stage_audio(wave(48000, 1, &vec![24576; 2048]), &WavDecoder)
        .unwrap();
    let negative = store
        .stage_audio(wave(48000, 1, &vec![-20480; 2048]), &WavDecoder)
        .unwrap();
    let a = plan(&positive);
    let mut b = plan(&negative);
    b.target = AudioComponentId::new(2).unwrap().into();
    let mut runtime = Runtime::with_budget(48000, 1, budget.clone()).unwrap();
    let jobs = runtime.synchronize(origin(0), &[a, b]).unwrap();
    let mut queue = AudioPreparationQueue::new(budget.clone());
    for (job, raw) in jobs.into_iter().zip([positive, negative]) {
        queue.submit(job, raw).unwrap();
        let result = queue.start_next().unwrap().execute(&mut WavDecoder);
        runtime
            .complete(job, queue.finish(result).unwrap().1)
            .unwrap();
    }
    runtime
        .set_scene_running(runtime.token(Channel::Scene), true)
        .unwrap();
    for _ in 0..2 {
        runtime
            .trigger(runtime.token(Channel::Scene), a.target, 0)
            .unwrap();
        runtime
            .trigger(runtime.token(Channel::Audition), b.target, 0)
            .unwrap();
    }
    let (mut output, mut callback) = output_boundary(&budget, 48000, 1).unwrap();
    output.pump(&mut runtime).unwrap();
    let mut samples = [0.0; 128];
    callback.render_f32(&mut samples);
    assert_eq!(samples, [0.25; 128]);
    output.invalidate(true, false).unwrap();
    callback.render_f32(&mut samples);
    assert_eq!(samples, [-1.0; 128]);
}
struct Latched {
    entered: mpsc::SyncSender<()>,
    release: mpsc::Receiver<()>,
    first: bool,
}
impl AudioDecoder for Latched {
    fn probe(&self, bytes: &[u8]) -> Result<AudioMetadata, PrepareFailure> {
        WavDecoder.probe(bytes)
    }
    fn decode_into(
        &mut self,
        bytes: &[u8],
        metadata: AudioMetadata,
        output: &mut [f32],
        cancel: &Cancellation,
    ) -> Result<usize, PrepareFailure> {
        if self.first {
            self.first = false;
            self.entered.send(()).unwrap();
            self.release
                .recv_timeout(std::time::Duration::from_secs(5))
                .unwrap();
        }
        WavDecoder.decode_into(bytes, metadata, output, cancel)
    }
}
#[test]
fn background_worker_cancel_at_latch_rejects_real_wav_and_returns_all_leases() {
    let budget = PcmBudget::default();
    let store = BlobStore::default();
    let raw = store
        .stage_audio(wave(48000, 1, &[1; 32]), &WavDecoder)
        .unwrap();
    let mut runtime = Runtime::with_budget(48000, 1, budget.clone()).unwrap();
    let job = runtime.synchronize(origin(0), &[plan(&raw)]).unwrap()[0];
    let mut queue = AudioPreparationQueue::new(budget.clone());
    queue.submit(job, raw).unwrap();
    let (entered, wait) = mpsc::sync_channel(1);
    let (release, gate) = mpsc::sync_channel(1);
    let mut worker = PreparationWorker::new(Latched {
        entered,
        release: gate,
        first: true,
    })
    .unwrap();
    assert!(worker.try_submit(queue.start_next().unwrap()).is_ok());
    wait.recv_timeout(std::time::Duration::from_secs(5))
        .unwrap();
    queue.close();
    assert_eq!(budget.usage().bytes, 128);
    release.send(()).unwrap();
    // Receiving completion, not timing guesses, establishes decoder exit.
    let done = worker
        .wait_result(std::time::Duration::from_secs(5))
        .unwrap();
    assert!(matches!(
        queue.finish(done).unwrap().1,
        Err(PrepareFailure::Cancelled)
    ));
    worker.close();
    assert!(worker.closed());
    assert_eq!(budget.usage().bytes, 0);
    assert_eq!(store.usage().unwrap().bytes, 0);
}
struct PanickingDecoder(mpsc::SyncSender<()>);
impl AudioDecoder for PanickingDecoder {
    fn probe(&self, bytes: &[u8]) -> Result<AudioMetadata, PrepareFailure> {
        WavDecoder.probe(bytes)
    }
    fn decode_into(
        &mut self,
        _: &[u8],
        _: AudioMetadata,
        _: &mut [f32],
        _: &Cancellation,
    ) -> Result<usize, PrepareFailure> {
        self.0.send(()).unwrap();
        panic!("controlled decoder failure");
    }
}
#[test]
fn decoder_panic_releases_running_allocations_but_is_never_a_successful_join() {
    let store = BlobStore::default();
    let budget = PcmBudget::default();
    let raw = store
        .stage_audio(wave(48000, 1, &[1; 32]), &WavDecoder)
        .unwrap();
    let mut runtime = Runtime::with_budget(48000, 1, budget.clone()).unwrap();
    let job = runtime.synchronize(origin(0), &[plan(&raw)]).unwrap()[0];
    let mut queue = AudioPreparationQueue::new(budget.clone());
    queue.submit(job, raw.clone()).unwrap();
    let (entered, receive) = mpsc::sync_channel(1);
    let mut worker = PreparationWorker::new(PanickingDecoder(entered)).unwrap();
    assert!(worker.try_submit(queue.start_next().unwrap()).is_ok());
    receive
        .recv_timeout(std::time::Duration::from_secs(5))
        .unwrap();
    worker.close();
    assert!(!worker.closed());
    worker.close();
    assert!(!worker.closed());
    queue.close();
    drop(queue);
    drop(runtime);
    drop(raw);
    assert_eq!(budget.usage().bytes, 0);
    assert_eq!(store.usage().unwrap().bytes, 0);
}
#[test]
fn a_late_real_worker_result_cannot_override_a_newer_project_recipe() {
    let (mut session, store) = project();
    let budget = PcmBudget::default();
    let mut controller = SkinAudioController::with_budget(48000, 1, budget.clone()).unwrap();
    controller.admit_builds(&session, &store).unwrap();
    let (entered, wait) = mpsc::sync_channel(1);
    let (release, gate) = mpsc::sync_channel(1);
    let mut worker = PreparationWorker::new(Latched {
        entered,
        release: gate,
        first: true,
    })
    .unwrap();
    assert!(
        worker
            .try_submit(controller.start_preparation().unwrap())
            .is_ok()
    );
    wait.recv_timeout(std::time::Duration::from_secs(5))
        .unwrap();
    session
        .submit(CommandRequest {
            api_version: API_VERSION,
            session_id: session.project_info().session_id,
            request_id: RequestId::new(2).unwrap(),
            expected: session.project_info().state,
            commands: vec![Command::SetAudioProperties {
                component: id(),
                gain: Gain::new(0.5).unwrap(),
                visible: true,
            }],
        })
        .unwrap();
    controller.admit_builds(&session, &store).unwrap();
    release.send(()).unwrap();
    let old = worker
        .wait_result(std::time::Duration::from_secs(5))
        .unwrap();
    assert_eq!(
        controller.finish_preparation(&session, old).unwrap(),
        Completion::Stale
    );
    assert!(
        worker
            .try_submit(controller.start_preparation().unwrap())
            .is_ok()
    );
    let new = worker
        .wait_result(std::time::Duration::from_secs(5))
        .unwrap();
    assert!(matches!(
        controller.finish_preparation(&session, new).unwrap(),
        Completion::Activated { .. }
    ));
    let runtime = controller.runtime();
    runtime
        .trigger(runtime.token(Channel::Audition), id(), 0)
        .unwrap();
    let mut samples = [0.0; 32];
    runtime.mix(&mut samples).unwrap();
    assert_eq!(samples, [0.125; 32]);
    controller.close();
    worker.close();
    drop(controller);
    assert_eq!(budget.usage().bytes, 0);
}
#[test]
fn real_stereo_ramp_downmix_and_resampling_match_independent_reference() {
    let bytes = wave(44100, 2, &[0, 16384, 16384, 32767, -16384, 0, 8192, 8192]);
    let store = BlobStore::default();
    let raw = store.stage_audio(bytes, &WavDecoder).unwrap();
    let mut runtime = Runtime::new(48000, 1).unwrap();
    runtime.enable_minimum_conversion();
    let job = runtime.synchronize(origin(0), &[plan(&raw)]).unwrap()[0];
    let sound = prepared(&runtime, &raw, job);
    runtime.complete(job, Ok(sound)).unwrap();
    runtime
        .trigger(runtime.token(Channel::Audition), id(), 0)
        .unwrap();
    let mut samples = [0.0; 6];
    runtime.mix(&mut samples).unwrap();
    let reference = [0.25f64, (0.5 + 32767.0 / 32768.0) / 2.0, -0.25, 0.25];
    for (i, sample) in samples[..5].iter().enumerate() {
        let x = i as f64 * 44100.0 / 48000.0;
        let a = x.floor() as usize;
        let b = (a + 1).min(3);
        let fraction = x - x.floor();
        let expected = reference[a] * (1.0 - fraction) + reference[b] * fraction;
        assert!((f64::from(*sample) - expected).abs() < 1e-7);
    }
    assert_eq!(samples[5], 0.0);
}

fn project() -> (ProjectSession, BlobStore) {
    let store = BlobStore::default();
    let raw = store
        .stage_audio(wave(48000, 1, &vec![8192; 4096]), &WavDecoder)
        .unwrap();
    let mut session =
        ProjectSession::with_resources(1, 1, Box::new(store.clone()), SessionLimits::default())
            .unwrap();
    session
        .submit(CommandRequest {
            api_version: API_VERSION,
            session_id: session.project_info().session_id,
            request_id: RequestId::new(1).unwrap(),
            expected: session.project_info().state,
            commands: vec![
                Command::RegisterAsset {
                    asset: Asset::new(
                        AssetId::new(1).unwrap(),
                        "self.wav".into(),
                        raw.reference(),
                        AssetKind::Audio(raw.audio_metadata().unwrap()),
                    )
                    .unwrap(),
                },
                Command::CreateAudioComponent {
                    component: AudioComponent::new(
                        id(),
                        SoundSlotKey::new("self".into()).unwrap(),
                        AssetId::new(1).unwrap(),
                    ),
                },
            ],
        })
        .unwrap();
    (session, store)
}
fn draft(session: &mut ProjectSession, request: u128, action: DraftAction) {
    session
        .edit_draft(DraftRequest {
            api_version: API_VERSION,
            session_id: session.project_info().session_id,
            request_id: RequestId::new(request).unwrap(),
            action,
        })
        .unwrap();
}
#[test]
fn three_hundred_real_wav_draft_versions_activate_and_retire_old_events_voices_and_pcm() {
    let (mut session, store) = project();
    let budget = PcmBudget::default();
    let mut controller = SkinAudioController::with_budget(48000, 1, budget.clone()).unwrap();
    controller.admit_builds(&session, &store).unwrap();
    let done = controller
        .start_preparation()
        .unwrap()
        .execute(&mut WavDecoder);
    controller.finish_preparation(&session, done).unwrap();
    let base = session.snapshot();
    let nodes = session.history_len();
    let expected = session.project_info().state;
    draft(&mut session, 2, DraftAction::Begin { expected });
    for i in 0..300 {
        let runtime = controller.runtime();
        let token = runtime.token(Channel::Audition);
        let now = runtime.mix_frame();
        runtime.trigger(token, id(), now).unwrap();
        // This future event must not fire after selecting a different draft revision.
        runtime.trigger(token, id(), now + 2).unwrap();
        let gain = 0.2 + (i + 1) as f64 / 500.0;
        let expected = session
            .draft_info(session.project_info().session_id)
            .unwrap()
            .unwrap()
            .state;
        draft(
            &mut session,
            3 + i,
            DraftAction::Replace {
                expected,
                commands: vec![Command::SetAudioProperties {
                    component: id(),
                    gain: Gain::new(gain).unwrap(),
                    visible: true,
                }],
            },
        );
        let admission = controller.admit_builds(&session, &store).unwrap();
        assert_eq!(admission.len(), 1);
        assert_eq!(admission[0].status, Ok(AdmissionState::Queued));
        let job = admission[0].request;
        assert_eq!(job.origin.draft.unwrap().1, i as u64 + 1);
        assert_eq!(controller.runtime().voices().count(), 1); // Same audition cursor, live gain.
        assert_ne!(
            controller.runtime().status(id()).unwrap().effective,
            Some(job.origin)
        );
        let done = controller
            .start_preparation()
            .unwrap()
            .execute(&mut WavDecoder);
        assert_eq!(done.observation().request, job);
        assert!(matches!(
            controller.finish_preparation(&session, done).unwrap(),
            Completion::Activated { .. }
        ));
        let runtime = controller.runtime();
        assert_eq!(runtime.status(id()).unwrap().effective, Some(job.origin));
        let r = runtime
            .trigger(runtime.token(Channel::Audition), id(), runtime.mix_frame())
            .unwrap();
        assert_eq!(r.origin, job.origin);
        assert!(!r.stale);
        let mut output = [0.0; 4];
        runtime.mix(&mut output).unwrap();
        let reference = 0.5 * gain as f32; // Both audition voices follow the component gain.
        assert!(output.iter().all(|s| (*s - reference).abs() < 1e-7));
        assert_eq!(runtime.voices().count(), 2); // No stale third event at offset 2.
        runtime
            .stop_audition(runtime.token(Channel::Audition))
            .unwrap();
        assert_eq!(runtime.voices().count(), 0);
        let usage = controller.preparation_usage();
        assert_eq!((usage.active, usage.pending, usage.cached), (0, 0, 1));
        assert_eq!(usage.pcm.bytes, 4096 * 4); // Only current effective PCM, no old lease.
        assert_eq!(session.history_len(), nodes);
    }
    let expected = session
        .draft_info(session.project_info().session_id)
        .unwrap()
        .unwrap()
        .state;
    draft(&mut session, 303, DraftAction::Cancel { expected });
    controller.admit_builds(&session, &store).unwrap();
    assert_eq!(session.snapshot(), base);
    assert_eq!(session.history_len(), nodes);
    controller.close();
    drop(controller);
    assert_eq!(budget.usage().bytes, 0);
}
#[test]
fn actual_draft_cancel_and_history_switch_fence_native_copies_without_ui_refresh() {
    let (mut session, store) = project();
    let budget = PcmBudget::default();
    let (mut output, mut callback) = output_boundary(&budget, 48000, 1).unwrap();
    let mut audio = AudioPlayback::new(48000, 1, budget.clone()).unwrap();
    audio.step(&session, &store, &mut output).unwrap();
    assert!(
        audio
            .wait_preparation(&session, &output, std::time::Duration::from_secs(5))
            .unwrap()
    );
    callback.render_f32(&mut [0.0; OUTPUT_QUEUE_FRAMES]);
    let base = session.snapshot();
    let nodes = session.history_len();
    let expected = session.project_info().state;
    draft(&mut session, 2, DraftAction::Begin { expected });
    let state = session
        .draft_info(session.project_info().session_id)
        .unwrap()
        .unwrap()
        .state;
    draft(
        &mut session,
        3,
        DraftAction::Replace {
            expected: state,
            commands: vec![Command::SetAudioProperties {
                component: id(),
                gain: Gain::new(0.5).unwrap(),
                visible: true,
            }],
        },
    );
    audio.step(&session, &store, &mut output).unwrap();
    assert!(
        audio
            .wait_preparation(&session, &output, std::time::Duration::from_secs(5))
            .unwrap()
    );
    callback.render_f32(&mut [0.0; OUTPUT_QUEUE_FRAMES]);
    audio
        .trigger(&session, &output, Channel::Audition, id())
        .unwrap();
    audio.step(&session, &store, &mut output).unwrap();
    let mut samples = [0.0; 128];
    callback.render_f32(&mut samples);
    assert_eq!(samples, [0.125; 128]);
    let state = session
        .draft_info(session.project_info().session_id)
        .unwrap()
        .unwrap()
        .state;
    draft(&mut session, 4, DraftAction::Cancel { expected: state });
    let _ = audio.trigger(&session, &output, Channel::Scene, id());
    callback.render_f32(&mut samples);
    assert_eq!(samples, [0.0; 128]);
    assert_eq!(session.snapshot(), base);
    assert_eq!(session.history_len(), nodes);
    audio.step(&session, &store, &mut output).unwrap();
    assert!(
        audio
            .wait_preparation(&session, &output, std::time::Duration::from_secs(5))
            .unwrap()
    );
    callback.render_f32(&mut [0.0; OUTPUT_QUEUE_FRAMES]);
    audio
        .trigger(&session, &output, Channel::Audition, id())
        .unwrap();
    audio.step(&session, &store, &mut output).unwrap();
    callback.render_f32(&mut samples);
    assert_eq!(samples, [0.25; 128]);
    session
        .navigate(HistoryRequest {
            api_version: API_VERSION,
            session_id: session.project_info().session_id,
            request_id: RequestId::new(5).unwrap(),
            expected: session.project_info().state,
            action: HistoryAction::Undo,
        })
        .unwrap();
    assert!(
        audio
            .trigger(&session, &output, Channel::Audition, id())
            .is_err()
    );
    callback.render_f32(&mut samples);
    assert_eq!(samples, [0.0; 128]);
    audio.close(&output).unwrap();
    drop(audio);
    drop(output);
    drop(callback);
    assert_eq!(budget.usage().bytes, 0);
}

#[test]
fn library_uses_shared_cache_admission_and_holds_cancelled_active_credit_until_release() {
    let store = BlobStore::default();
    let raw = store
        .stage_audio(wave(48000, 1, &[8192; 4]), &WavDecoder)
        .unwrap();
    let budget = PcmBudget::new(80, 16).unwrap();
    let mut runtime = Runtime::with_budget(48000, 1, budget.clone()).unwrap();
    let component = runtime.synchronize(origin(0), &[plan(&raw)]).unwrap()[0];
    let mut queue = AudioPreparationQueue::new(budget.clone());
    queue.submit(component, raw.clone()).unwrap();
    let done = queue.start_next().unwrap().execute(&mut WavDecoder);
    let (_, sound) = queue.finish(done).unwrap();
    runtime.complete(component, sound).unwrap();
    let mut library = plan(&raw);
    library.target = SoundTarget::Library(AssetId::new(1).unwrap());
    let request = runtime.request_library(library).unwrap();
    let Admission::Cached(sound) = queue.submit(request, raw.clone()).unwrap() else {
        panic!("shared PCM was not reused")
    };
    assert_eq!(budget.usage().bytes, 16);
    runtime.complete(request, Ok(sound)).unwrap();
    runtime
        .stop_audition(runtime.token(Channel::Audition))
        .unwrap();
    runtime.shutdown();
    assert_eq!(budget.usage().bytes, 0);

    let mut runtime = Runtime::with_budget(48000, 1, budget.clone()).unwrap();
    let requests = runtime
        .synchronize(
            origin(0),
            &(1..=4)
                .map(|n| {
                    let mut p = plan(&raw);
                    p.target = AudioComponentId::new(n).unwrap().into();
                    p
                })
                .collect::<Vec<_>>(),
        )
        .unwrap();
    for request in &requests {
        queue.submit(*request, raw.clone()).unwrap();
    }
    let running = queue.start_next().unwrap();
    let request = runtime.request_library(library).unwrap();
    queue.submit(request, raw.clone()).unwrap();
    assert_eq!(queue.usage().pending, 4);
    let mut extra = library;
    extra.target = SoundTarget::Library(AssetId::new(2).unwrap());
    let refused = runtime.request_library(extra).unwrap();
    assert!(matches!(
        queue.submit(refused, raw.clone()),
        Err(PreparationError::Busy)
    ));
    queue.cancel_library();
    queue.cancel_outdated(origin(1));
    assert_eq!(queue.usage().pending, 0);
    assert_eq!(budget.usage().bytes, 16);
    assert!(queue.start_next().is_none());
    let finished = running.execute(&mut WavDecoder);
    assert_eq!(budget.usage().bytes, 0);
    assert!(matches!(
        queue.finish(finished).unwrap().1,
        Err(PrepareFailure::Cancelled)
    ));
    queue.close();
    runtime.shutdown();
    drop(raw);
    assert_eq!(store.usage().unwrap().bytes, 0);
}
