//! Explicit audible native diagnostic; no UI, personal media or client access.
use skin_editor_application::{values::*, *};
use skin_editor_audio::{Channel, PcmBudget};
use skin_editor_infrastructure::{AudioPlayback, BlobStore, NativeOutput, WavDecoder};
use std::time::{Duration, Instant};

pub(super) fn tone(rate: u32, channels: u16, hz: f32) -> Vec<u8> {
    let frames = rate / 2;
    let size = frames * u32::from(channels) * 2;
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
    for frame in 0..frames {
        let sample =
            ((frame as f32 * hz * std::f32::consts::TAU / rate as f32).sin() * 600.0) as i16;
        for _ in 0..channels {
            bytes.extend(sample.to_le_bytes());
        }
    }
    bytes
}
pub(super) fn component() -> AudioComponentId {
    AudioComponentId::new(1).unwrap()
}
pub(super) fn submit(
    session: &mut ProjectSession,
    next: &mut u128,
    commands: Vec<Command>,
) -> Result<(), String> {
    *next += 1;
    session
        .submit(CommandRequest {
            api_version: API_VERSION,
            session_id: session.project_info().session_id,
            request_id: RequestId::new(*next).unwrap(),
            expected: session.project_info().state,
            commands,
        })
        .map_err(|e| format!("{e:?}"))?;
    Ok(())
}
fn phase(
    audio: &mut AudioPlayback,
    session: &ProjectSession,
    store: &BlobStore,
    native: &mut NativeOutput,
    millis: u64,
) -> Result<(), String> {
    let until = Instant::now() + Duration::from_millis(millis);
    while Instant::now() < until {
        audio.step(session, store, &mut native.producer)?;
        // Scheduling interval for this explicit hardware probe, never a race-test oracle.
        std::thread::sleep(Duration::from_millis(2));
    }
    Ok(())
}
fn settle(
    audio: &mut AudioPlayback,
    session: &ProjectSession,
    store: &BlobStore,
    native: &mut NativeOutput,
) -> Result<(), String> {
    audio.step(session, store, &mut native.producer)?;
    if !audio.wait_preparation(session, &native.producer, Duration::from_secs(3))? {
        return Err("decoder completion timed out".into());
    }
    Ok(())
}
fn quiet(
    audio: &mut AudioPlayback,
    session: &ProjectSession,
    store: &BlobStore,
    native: &mut NativeOutput,
) -> Result<(), String> {
    let fence = native.producer.invalidate(false, false)?;
    let deadline = Instant::now() + Duration::from_secs(2);
    while !native.producer.acknowledged(fence) {
        if Instant::now() >= deadline {
            return Err("native fence was not acknowledged".into());
        }
        phase(audio, session, store, native, 2)?;
    }
    let start = native.producer.stats();
    phase(audio, session, store, native, 80)?;
    let end = native.producer.stats();
    if start.nonzero_samples != end.nonzero_samples || start.callbacks == end.callbacks {
        return Err("obsolete samples survived the acknowledged callback fence".into());
    }
    Ok(())
}
pub fn run(destination: &str) -> Result<(), String> {
    let path = std::path::Path::new(destination);
    if path.exists() {
        return Err("audio diagnostic output must be new".into());
    }
    std::fs::create_dir_all(path).map_err(|e| e.to_string())?;
    let budget = PcmBudget::default();
    let store = BlobStore::default();
    let mut session =
        ProjectSession::with_resources(1, 1, Box::new(store.clone()), SessionLimits::default())
            .map_err(|e| format!("{e:?}"))?;
    let mut next = 0;
    let mut commands = Vec::new();
    let mut originals = Vec::new();
    for (id, rate, channels, hz) in [(1, 44100, 1, 330.0), (2, 48000, 2, 550.0)] {
        let raw = store
            .stage_audio(tone(rate, channels, hz), &WavDecoder)
            .map_err(|e| format!("{e:?}"))?;
        commands.push(Command::RegisterAsset {
            asset: Asset::new(
                AssetId::new(id).unwrap(),
                format!("self authored {id}.wav"),
                raw.reference(),
                AssetKind::Audio(raw.audio_metadata().unwrap()),
            )
            .unwrap(),
        });
        originals.push(raw);
    }
    commands.push(Command::CreateAudioComponent {
        component: AudioComponent::new(
            component(),
            SoundSlotKey::new("self.native".into()).unwrap(),
            AssetId::new(1).unwrap(),
        ),
    });
    submit(&mut session, &mut next, commands)?;
    let mut native = NativeOutput::open(&budget)?;
    let mut audio = AudioPlayback::new(native.rate, native.channels, budget.clone())?;
    settle(&mut audio, &session, &store, &mut native)?;
    audio.prefill(&mut native.producer)?;
    native.play()?;
    phase(&mut audio, &session, &store, &mut native, 30)?;
    let startup_empty = native.producer.stats().empty_frames;
    audio.scene_running(&native.producer, true)?;
    audio.trigger(&session, &native.producer, Channel::Scene, component())?;
    let initial = native.producer.stats().nonzero_samples;
    phase(&mut audio, &session, &store, &mut native, 100)?;
    if native.producer.stats().nonzero_samples == initial {
        return Err("no scene samples reached callback".into());
    }
    audio.scene_running(&native.producer, false)?;
    let anchor = audio.scene_frame();
    quiet(&mut audio, &session, &store, &mut native)?;
    audio.trigger(&session, &native.producer, Channel::Audition, component())?;
    let initial = native.producer.stats().nonzero_samples;
    phase(&mut audio, &session, &store, &mut native, 100)?;
    if audio.scene_frame() != anchor || native.producer.stats().nonzero_samples == initial {
        return Err("audition/frozen scene isolation failed".into());
    }
    audio.stop_audition(&native.producer)?;
    quiet(&mut audio, &session, &store, &mut native)?;
    audio.scene_running(&native.producer, true)?;
    quiet(&mut audio, &session, &store, &mut native)?; // no old events on resume
    submit(
        &mut session,
        &mut next,
        vec![Command::SetAudioAsset {
            component: component(),
            asset: AssetId::new(2).unwrap(),
        }],
    )?;
    settle(&mut audio, &session, &store, &mut native)?;
    audio.trigger(&session, &native.producer, Channel::Scene, component())?;
    phase(&mut audio, &session, &store, &mut native, 100)?;
    audio.scene_running(&native.producer, false)?;
    quiet(&mut audio, &session, &store, &mut native)?;
    let snapshot = session.snapshot();
    let nodes = session.history_len();
    let action = DraftAction::Begin {
        expected: session.project_info().state,
    };
    next += 1;
    session
        .edit_draft(DraftRequest {
            api_version: API_VERSION,
            session_id: session.project_info().session_id,
            request_id: RequestId::new(next).unwrap(),
            action,
        })
        .map_err(|e| format!("{e:?}"))?;
    let state = session
        .draft_info(session.project_info().session_id)
        .unwrap()
        .unwrap()
        .state;
    next += 1;
    session
        .edit_draft(DraftRequest {
            api_version: API_VERSION,
            session_id: session.project_info().session_id,
            request_id: RequestId::new(next).unwrap(),
            action: DraftAction::Replace {
                expected: state,
                commands: vec![Command::SetAudioProperties {
                    component: component(),
                    gain: Gain::new(0.5).unwrap(),
                    visible: true,
                }],
            },
        })
        .map_err(|e| format!("{e:?}"))?;
    settle(&mut audio, &session, &store, &mut native)?;
    audio.trigger(&session, &native.producer, Channel::Audition, component())?;
    phase(&mut audio, &session, &store, &mut native, 60)?;
    let state = session
        .draft_info(session.project_info().session_id)
        .unwrap()
        .unwrap()
        .state;
    next += 1;
    session
        .edit_draft(DraftRequest {
            api_version: API_VERSION,
            session_id: session.project_info().session_id,
            request_id: RequestId::new(next).unwrap(),
            action: DraftAction::Cancel { expected: state },
        })
        .map_err(|e| format!("{e:?}"))?;
    audio.step(&session, &store, &mut native.producer)?;
    quiet(&mut audio, &session, &store, &mut native)?;
    if session.snapshot() != snapshot || session.history_len() != nodes {
        return Err("cancel changed project/history".into());
    }
    audio.trigger(&session, &native.producer, Channel::Audition, component())?;
    phase(&mut audio, &session, &store, &mut native, 60)?;
    next += 1;
    session
        .navigate(HistoryRequest {
            api_version: API_VERSION,
            session_id: session.project_info().session_id,
            request_id: RequestId::new(next).unwrap(),
            expected: session.project_info().state,
            action: HistoryAction::Undo,
        })
        .map_err(|e| format!("{e:?}"))?;
    audio.step(&session, &store, &mut native.producer)?;
    quiet(&mut audio, &session, &store, &mut native)?;
    audio.close(&native.producer)?;
    let stats = native.producer.stats();
    let rate = native.rate;
    let channels = native.channels;
    let format = format!("{:?}", native.sample_format);
    let max_callback_frames = stats.max_callback_frames;
    let steady_empty = stats.empty_frames - startup_empty;
    drop(audio);
    drop(native);
    if budget.usage().bytes != 0 || stats.failed {
        return Err("native shutdown failed or retained PCM".into());
    }
    let report = format!(
        "{{\n  \"schema\": 1,\n  \"rate\": {rate},\n  \"channels\": {channels},\n  \"format\": \"{format}\",\n  \"callbacks\": {},\n  \"callback_frames\": {},\n  \"max_callback_frames\": {max_callback_frames},\n  \"startup_empty_frames\": {startup_empty},\n  \"steady_empty_frames\": {steady_empty},\n  \"empty_frames\": {},\n  \"stale_lane_frames\": {},\n  \"nonzero_samples\": {},\n  \"pcm_peak_bytes\": {},\n  \"pcm_final_bytes\": 0,\n  \"scene_pause_audition_isolation\": true,\n  \"resume_no_old_events\": true,\n  \"draft_cancel_native_fence\": true,\n  \"history_native_fence\": true,\n  \"acoustic_or_consumption_clock_verified\": false\n}}\n",
        stats.callbacks,
        stats.frames,
        stats.empty_frames,
        stats.stale_lane_frames,
        stats.nonzero_samples,
        budget.usage().peak_bytes
    );
    use std::io::Write;
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path.join("report.json"))
        .map_err(|e| e.to_string())?
        .write_all(report.as_bytes())
        .map_err(|e| e.to_string())?;
    print!("{report}");
    Ok(())
}
