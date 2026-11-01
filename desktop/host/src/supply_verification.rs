//! Explicit native reference: control/GUI sleeps while the supply owner continues.
use crate::audio_verification::{component, submit, tone};
use skin_editor_application::{values::*, *};
use skin_editor_audio::{BackgroundTrack, Channel, PcmBudget};
use skin_editor_infrastructure::{
    AudioSupplyClient, BlobStore, NativeOutput, SupplyOutcome, WavDecoder,
};
use std::time::{Duration, Instant};
fn outcome(client: &mut AudioSupplyClient, id: u64) -> Result<SupplyOutcome, String> {
    let until = Instant::now() + Duration::from_secs(3);
    while Instant::now() < until {
        let receipt = client
            .wait_receipt(Duration::from_millis(100))
            .map_err(|e| format!("{e:?}"))?;
        if let Some(receipt) = receipt {
            let value = receipt.result.as_ref().map_err(|e| format!("{e:?}"))?;
            if receipt.id == id {
                // Preserve the private bounded credit until this reply has been inspected.
                let _ = value;
                return receipt.result.map_err(|e| format!("{e:?}"));
            }
        }
    }
    Err("supply receipt timeout".into())
}
fn ready(client: &mut AudioSupplyClient, session: &ProjectSession) -> Result<(), String> {
    client.synchronize(session).map_err(|e| format!("{e:?}"))?;
    let info = session.project_info();
    let wanted_draft = session
        .draft_info(info.session_id)
        .map_err(|e| format!("{e:?}"))?
        .map(|d| (d.state.id.get(), d.state.revision));
    let until = Instant::now() + Duration::from_secs(3);
    loop {
        while let Some(receipt) = client.poll() {
            receipt.result.map_err(|e| format!("{e:?}"))?;
        }
        let state = client.status().map_err(|e| format!("{e:?}"))?;
        if let Some(error) = state.fault {
            return Err(error);
        }
        if state.ready_components == 1
            && state.origin.is_some_and(|o| {
                o.session == info.session_id.get()
                    && o.node == info.state.head.get()
                    && o.revision == info.state.revision
                    && o.draft == wanted_draft
            })
        {
            return Ok(());
        }
        if Instant::now() >= until {
            return Err("WAV preparation timeout".into());
        }
        std::thread::sleep(Duration::from_millis(2));
    }
}
fn trigger(
    client: &mut AudioSupplyClient,
    session: &ProjectSession,
    lane: Channel,
) -> Result<(), String> {
    let id = client
        .trigger(session, lane, component())
        .map_err(|e| format!("{e:?}"))?;
    match outcome(client, id)? {
        SupplyOutcome::Triggered(receipt) if receipt.voice.is_some() && !receipt.stale => Ok(()),
        other => Err(format!("unexpected trigger: {other:?}")),
    }
}
pub(super) fn draft(
    session: &mut ProjectSession,
    next: &mut u128,
    action: DraftAction,
) -> Result<(), String> {
    *next += 1;
    session
        .edit_draft(DraftRequest {
            api_version: API_VERSION,
            session_id: session.project_info().session_id,
            request_id: RequestId::new(*next).unwrap(),
            action,
        })
        .map_err(|e| format!("{e:?}"))?;
    Ok(())
}
pub fn run(destination: &str) -> Result<(), String> {
    let path = std::path::Path::new(destination);
    if path.exists() {
        return Err("supply diagnostic output must be new".into());
    }
    std::fs::create_dir_all(path).map_err(|e| e.to_string())?;
    let budget = PcmBudget::default();
    let store = BlobStore::default();
    let mut session =
        ProjectSession::with_resources(1, 1, Box::new(store.clone()), SessionLimits::default())
            .map_err(|e| format!("{e:?}"))?;
    let mut next = 0;
    let mut originals = Vec::new();
    let mut commands = Vec::new();
    for (id, rate, channels, hz) in [(1, 44100, 1, 330.0), (2, 48000, 2, 550.0)] {
        let raw = store
            .stage_audio(tone(rate, channels, hz), &WavDecoder)
            .map_err(|e| format!("{e:?}"))?;
        commands.push(Command::RegisterAsset {
            asset: Asset::new(
                AssetId::new(id).unwrap(),
                format!("self {id}.wav"),
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
            SoundSlotKey::new("self.supply".into()).unwrap(),
            AssetId::new(1).unwrap(),
        ),
    });
    submit(&mut session, &mut next, commands)?;
    let native = NativeOutput::open(&budget)?;
    let (rate, channels, format) = (
        native.rate,
        native.channels,
        format!("{:?}", native.sample_format),
    );
    let (stream, producer) = native.into_parts();
    let mut client = AudioSupplyClient::spawn(producer, rate, channels, budget.clone(), store)
        .map_err(|e| format!("{e:?}"))?;
    let observer = client.output().map_err(|e| format!("{e:?}"))?;
    // Finite same-format external reference, not a skin slot or a long-song streamer.
    let frames = rate as usize * 8;
    let mut pcm = budget
        .reserve(frames * usize::from(channels))
        .map_err(|e| format!("{e:?}"))?
        .allocate();
    for (frame, samples) in pcm
        .samples_mut()
        .map_err(|e| format!("{e:?}"))?
        .chunks_exact_mut(usize::from(channels))
        .enumerate()
    {
        let value = (frame as f32 * 220.0 * std::f32::consts::TAU / rate as f32).sin() * 0.012;
        samples.fill(value);
    }
    let reference = BackgroundTrack::new(pcm, rate, channels).map_err(|e| format!("{e:?}"))?;
    let id = client
        .set_background(reference)
        .map_err(|e| format!("{e:?}"))?;
    outcome(&mut client, id)?;
    ready(&mut client, &session)?;
    stream.play()?;
    std::thread::sleep(Duration::from_millis(60));
    let startup_empty = observer.stats().empty_frames;
    let id = client.scene_running(true).map_err(|e| format!("{e:?}"))?;
    outcome(&mut client, id)?;
    trigger(&mut client, &session, Channel::Scene)?;
    let before_stall = observer.stats().background_next;
    // No controls, project reads, GUI ticks or output pumping for these intervals.
    std::thread::sleep(Duration::from_millis(500));
    let after_stall = observer.stats().background_next;
    if after_stall <= before_stall {
        return Err("background stopped with the control caller".into());
    }
    submit(
        &mut session,
        &mut next,
        vec![Command::SetAudioAsset {
            component: component(),
            asset: AssetId::new(2).unwrap(),
        }],
    )?;
    ready(&mut client, &session)?;
    trigger(&mut client, &session, Channel::Scene)?;
    std::thread::sleep(Duration::from_millis(500));
    let after_hot = observer.stats().background_next;
    if after_hot <= after_stall || observer.stats().background_gaps != 0 {
        return Err("skin replacement reset/skipped background".into());
    }
    let id = client.scene_running(false).map_err(|e| format!("{e:?}"))?;
    let anchor = match outcome(&mut client, id)? {
        SupplyOutcome::Frozen {
            next_background_frame,
        } => next_background_frame,
        other => return Err(format!("missing confirmed freeze: {other:?}")),
    };
    trigger(&mut client, &session, Channel::Audition)?;
    let audible = observer.stats().nonzero_samples;
    std::thread::sleep(Duration::from_millis(120));
    if observer.stats().background_next != anchor || observer.stats().nonzero_samples <= audible {
        return Err("freeze/audition isolation failed".into());
    }
    let id = client.stop_audition().map_err(|e| format!("{e:?}"))?;
    outcome(&mut client, id)?;
    // Ring may still hold stale copies; the consumer filters them independently.
    std::thread::sleep(Duration::from_millis(40));
    let quiet = observer.stats().nonzero_samples;
    std::thread::sleep(Duration::from_millis(40));
    if observer.stats().nonzero_samples != quiet {
        return Err("audition copies survived stop".into());
    }
    let id = client.scene_running(true).map_err(|e| format!("{e:?}"))?;
    outcome(&mut client, id)?;
    let snapshot = session.snapshot();
    let nodes = session.history_len();
    let expected = session.project_info().state;
    draft(&mut session, &mut next, DraftAction::Begin { expected })?;
    let state = session
        .draft_info(session.project_info().session_id)
        .unwrap()
        .unwrap()
        .state;
    draft(
        &mut session,
        &mut next,
        DraftAction::Replace {
            expected: state,
            commands: vec![Command::SetAudioProperties {
                component: component(),
                gain: Gain::new(0.5).unwrap(),
                visible: true,
            }],
        },
    )?;
    ready(&mut client, &session)?;
    trigger(&mut client, &session, Channel::Audition)?;
    let state = session
        .draft_info(session.project_info().session_id)
        .unwrap()
        .unwrap()
        .state;
    draft(
        &mut session,
        &mut next,
        DraftAction::Cancel { expected: state },
    )?;
    ready(&mut client, &session)?;
    if session.snapshot() != snapshot || session.history_len() != nodes {
        return Err("cancel changed project/history".into());
    }
    let after_cancel = observer.stats().background_next;
    trigger(&mut client, &session, Channel::Audition)?;
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
    ready(&mut client, &session)?;
    std::thread::sleep(Duration::from_millis(500));
    let stats = observer.stats();
    if stats.background_next <= after_cancel || stats.background_gaps != 0 || stats.failed {
        return Err("history/resume background continuity failed".into());
    }
    client.close();
    drop(client);
    drop(stream);
    drop(observer);
    if budget.usage().bytes != 0 {
        return Err("supply close retained PCM".into());
    }
    let report = format!(
        "{{\n  \"schema\": 1,\n  \"rate\": {rate},\n  \"channels\": {channels},\n  \"format\": \"{format}\",\n  \"supply_threads\": 1,\n  \"decoder_threads\": 1,\n  \"control_stall_ms\": 500,\n  \"control_stall_rounds\": 3,\n  \"before_stall_frame\": {before_stall},\n  \"after_stall_frame\": {after_stall},\n  \"after_hot_replace_frame\": {after_hot},\n  \"freeze_anchor_frame\": {anchor},\n  \"after_cancel_frame\": {after_cancel},\n  \"background_next_frame\": {},\n  \"background_submitted_frames\": {},\n  \"background_position_gaps\": {},\n  \"callbacks\": {},\n  \"max_callback_frames\": {},\n  \"startup_empty_frames\": {startup_empty},\n  \"steady_empty_frames\": {},\n  \"stale_skin_lane_frames\": {},\n  \"nonzero_samples\": {},\n  \"pcm_peak_bytes\": {},\n  \"pcm_final_bytes\": 0,\n  \"freeze_audition_isolation\": true,\n  \"cancel_project_history_unchanged\": true,\n  \"skin_replace_preserves_background\": true,\n  \"acoustic_or_device_consumption_verified\": false\n}}\n",
        stats.background_next,
        stats.background_frames,
        stats.background_gaps,
        stats.callbacks,
        stats.max_callback_frames,
        stats.empty_frames - startup_empty,
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
