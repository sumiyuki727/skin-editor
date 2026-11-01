//! Opt-in native callback observation. No GUI, microphone, personal media or device clock.
use crate::audio_verification::submit;
use skin_editor_application::{values::*, *};
use skin_editor_audio::{Channel, PcmBudget};
use skin_editor_infrastructure::{
    AudioSupplyClient, BlobStore, NativeOutput, OutputControl, SupplyOutcome, WavDecoder,
};
use std::io::Write;
use std::time::{Duration, Instant};

pub(super) fn phase(name: &str) {
    println!("{{\"phase\":\"{name}\"}}");
    std::io::stdout().flush().expect("diagnostic stdout");
}
pub(super) fn wait(
    output: &OutputControl,
    mut condition: impl FnMut() -> bool,
) -> Result<(), String> {
    let until = Instant::now() + Duration::from_secs(5);
    while !condition() {
        if output.is_closed() || Instant::now() >= until {
            return Err("native observation timeout/fault".into());
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    Ok(())
}
pub(super) fn outcome(client: &mut AudioSupplyClient, id: u64) -> Result<SupplyOutcome, String> {
    loop {
        let receipt = client
            .wait_receipt(Duration::from_secs(3))
            .map_err(|e| format!("{e:?}"))?
            .ok_or("control timeout")?;
        if receipt.id == id {
            return receipt.result.map_err(|e| format!("{e:?}"));
        }
        receipt.result.map_err(|e| format!("{e:?}"))?;
    }
}
fn control(client: &mut AudioSupplyClient, running: bool) -> Result<(), String> {
    let id = client
        .scene_running(running)
        .map_err(|e| format!("{e:?}"))?;
    outcome(client, id)?;
    Ok(())
}

/// Repeated empty-output startup only; not the full editor lifecycle acceptance.
pub fn startup(destination: &str) -> Result<(), String> {
    let path = std::path::Path::new(destination);
    if path.exists() {
        return Err("startup diagnostic output must be new".into());
    }
    std::fs::create_dir_all(path).map_err(|e| e.to_string())?;
    let mut rounds = Vec::new();
    let mut traces = Vec::new();
    phase("idle");
    for _ in 0..20 {
        let budget = PcmBudget::default();
        let native = NativeOutput::open(&budget)?;
        let rate = native.rate;
        let channels = native.channels;
        let period = native.period_frames;
        let reserve = native.producer.idle_reserve_frames();
        let (stream, producer) = native.into_parts();
        let mut client = AudioSupplyClient::spawn(
            producer,
            rate,
            channels,
            budget.clone(),
            BlobStore::default(),
        )
        .map_err(|e| format!("{e:?}"))?;
        let output = client.output().map_err(|e| format!("{e:?}"))?;
        output.capture_startup()?;
        stream.play()?;
        consume(&output, rate, 600)?;
        let stats = output.stats();
        traces.push(output.startup_observations());
        client.close();
        drop(client);
        drop(stream);
        drop(output);
        rounds.push([
            stats.callbacks,
            stats.frames,
            stats.empty_frames,
            stats.max_callback_frames,
            stats.retimed_idle_frames,
            budget.usage().peak_bytes as u64,
            u64::from(period),
            reserve as u64,
            budget.usage().bytes as u64,
        ]);
        if stats.failed {
            return Err("startup device failed".into());
        }
    }
    phase("closed");
    std::thread::sleep(Duration::from_secs(2));
    let empty: u64 = rounds.iter().map(|r| r[2]).sum();
    let report = format!(
        "{{\"schema\":1,\"rounds\":{rounds:?},\"startup_observations\":{traces:?},\"empty_frames\":{empty}}}\n"
    );
    std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path.join("report.json"))
        .map_err(|e| e.to_string())?
        .write_all(report.as_bytes())
        .map_err(|e| e.to_string())?;
    phase("done");
    Ok(())
}
pub(super) fn stop(client: &mut AudioSupplyClient, output: &OutputControl) -> Result<(), String> {
    control(client, false)?;
    let id = client.stop_audition().map_err(|e| format!("{e:?}"))?;
    outcome(client, id)?;
    wait(output, || {
        let s = output.stats();
        s.lane_acknowledged
            .iter()
            .zip(s.lane_epochs)
            .all(|(a, e)| *a >= e)
    })
}
pub(super) fn consume(output: &OutputControl, rate: u32, millis: u64) -> Result<(), String> {
    let start = output.stats().frames;
    let needed = u64::from(rate) * millis / 1000;
    // Long load intervals leave the control caller asleep; callback frames finish the observation.
    if millis >= 500 {
        std::thread::sleep(Duration::from_millis(millis - 50));
    }
    wait(output, || output.stats().frames >= start + needed)
}
pub(super) fn wav(index: u32) -> Vec<u8> {
    let frames: u32 = [6000, 24000, 48000, 96000][index as usize / 4];
    let size = frames * 4;
    let mut b = Vec::with_capacity(size as usize + 44);
    b.extend(b"RIFF");
    b.extend((36 + size).to_le_bytes());
    b.extend(b"WAVEfmt ");
    b.extend(16u32.to_le_bytes());
    b.extend(1u16.to_le_bytes());
    b.extend(2u16.to_le_bytes());
    b.extend(48000u32.to_le_bytes());
    b.extend(192000u32.to_le_bytes());
    b.extend(4u16.to_le_bytes());
    b.extend(16u16.to_le_bytes());
    b.extend(b"data");
    b.extend(size.to_le_bytes());
    for frame in 0..frames {
        let sample = ((frame as f32 * (220 + index * 17) as f32 * std::f32::consts::TAU / 48000.0)
            .sin()
            * 16.0) as i16;
        for _ in 0..2 {
            b.extend(sample.to_le_bytes());
        }
    }
    b
}
pub fn run(destination: &str) -> Result<(), String> {
    let path = std::path::Path::new(destination);
    if path.exists() {
        return Err("load diagnostic output must be new".into());
    }
    std::fs::create_dir_all(path).map_err(|e| e.to_string())?;
    phase("setup");
    let budget = PcmBudget::default();
    let store = BlobStore::default();
    let mut session =
        ProjectSession::with_resources(1, 1, Box::new(store.clone()), SessionLimits::default())
            .map_err(|e| format!("{e:?}"))?;
    let mut next = 0;
    let mut commands = Vec::new();
    let mut originals = Vec::new();
    for n in 1..=16 {
        let raw = store
            .stage_audio(wav(n - 1), &WavDecoder)
            .map_err(|e| format!("{e:?}"))?;
        commands.push(Command::RegisterAsset {
            asset: Asset::new(
                AssetId::new(n.into()).unwrap(),
                format!("self {n}.wav"),
                raw.reference(),
                AssetKind::Audio(raw.audio_metadata().unwrap()),
            )
            .unwrap(),
        });
        commands.push(Command::CreateAudioComponent {
            component: AudioComponent::new(
                AudioComponentId::new(n.into()).unwrap(),
                SoundSlotKey::new(format!("self.load{n}")).unwrap(),
                AssetId::new(n.into()).unwrap(),
            ),
        });
        originals.push(raw);
    }
    submit(&mut session, &mut next, commands)?;
    let native = NativeOutput::open(&budget)?;
    let (rate, channels, format) = (
        native.rate,
        native.channels,
        format!("{:?}", native.sample_format),
    );
    let period_frames = native.period_frames;
    let idle_reserve_frames = native.producer.idle_reserve_frames();
    let (stream, producer) = native.into_parts();
    let mut client =
        AudioSupplyClient::spawn(producer, rate, channels, budget.clone(), store.clone())
            .map_err(|e| format!("{e:?}"))?;
    let output = client.output().map_err(|e| format!("{e:?}"))?;
    output.capture_startup()?;
    stream.play()?;
    // Explicitly retry only capacity-refused project candidates until all 16 are ready.
    client.synchronize(&session).map_err(|e| format!("{e:?}"))?;
    let until = Instant::now() + Duration::from_secs(5);
    loop {
        while let Some(r) = client.poll() {
            r.result.map_err(|e| format!("{e:?}"))?;
        }
        let s = client.status().map_err(|e| format!("{e:?}"))?;
        if s.ready_components == 16 {
            break;
        }
        if Instant::now() >= until {
            return Err(format!("fixture preparation failed: {s:?}"));
        }
        for (id, b) in s.bindings {
            if b.preparation
                == skin_editor_audio::Preparation::Failed(skin_editor_audio::PrepareFailure::Busy)
            {
                let request = client.retry_selected(id).map_err(|e| format!("{e:?}"))?;
                outcome(&mut client, request)?;
            }
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    stop(&mut client, &output)?;
    let snapshot = session.snapshot();
    let history = session.history_len();
    let steady_start = output.stats().empty_frames;
    phase("idle");
    consume(&output, rate, 3000)?;
    phase("latency");
    let mut latency = [Vec::new(), Vec::new()];
    let mut receipt_ms = [Vec::new(), Vec::new()];
    let mut max_poll_gap_ms = 0.0f64;
    for (lane_index, lane) in [Channel::Scene, Channel::Audition].into_iter().enumerate() {
        for round in 0..64 {
            stop(&mut client, &output)?;
            if lane == Channel::Scene {
                control(&mut client, true)?;
            }
            // Deliberate request-phase sweep, not a race-test synchronization oracle.
            std::thread::sleep(Duration::from_millis(round % 16));
            let baseline = output.stats().lane_nonzero_frames[lane_index];
            let start = Instant::now();
            let id = client
                .trigger_selected(lane, AudioComponentId::new(1).unwrap())
                .map_err(|e| format!("{e:?}"))?;
            match outcome(&mut client, id)? {
                SupplyOutcome::Triggered(r) if r.voice.is_some() && !r.stale => {}
                other => return Err(format!("trigger refused: {other:?}")),
            }
            receipt_ms[lane_index].push(start.elapsed().as_secs_f64() * 1000.0);
            let mut last = Instant::now();
            wait(&output, || {
                let now = Instant::now();
                max_poll_gap_ms = max_poll_gap_ms.max((now - last).as_secs_f64() * 1000.0);
                last = now;
                output.stats().lane_nonzero_frames[lane_index] > baseline
            })?;
            latency[lane_index].push(start.elapsed().as_secs_f64() * 1000.0);
        }
    }
    stop(&mut client, &output)?;
    let mut rejected = 0;
    let mut peak_voices = 0;
    for count in [1, 32] {
        phase(if count == 1 { "voices_1" } else { "voices_32" });
        for _ in 0..4 {
            stop(&mut client, &output)?;
            control(&mut client, true)?;
            let entries: Vec<_> = (0..count)
                .map(|i| {
                    (
                        if i % 2 == 0 {
                            Channel::Scene
                        } else {
                            Channel::Audition
                        },
                        AudioComponentId::new(13 + (i % 4) as u128).unwrap(),
                    )
                })
                .collect();
            let id = client
                .trigger_batch_selected(&entries)
                .map_err(|e| format!("{e:?}"))?;
            match outcome(&mut client, id)? {
                SupplyOutcome::BatchTriggered { results, .. }
                    if results.len() == count
                        && results
                            .iter()
                            .all(|r| r.as_ref().is_ok_and(|r| r.voice.is_some() && !r.stale)) => {}
                other => return Err(format!("batch refused: {other:?}")),
            }
            let status = client.status().map_err(|e| format!("{e:?}"))?;
            let voices = status.voices.iter().sum::<usize>();
            peak_voices = peak_voices.max(voices);
            if voices != count {
                return Err(format!("expected {count} voices, got {voices}"));
            }
            if count == 32 {
                let id = client
                    .trigger_batch_selected(&[(
                        Channel::Audition,
                        AudioComponentId::new(13).unwrap(),
                    )])
                    .map_err(|e| format!("{e:?}"))?;
                match outcome(&mut client, id)? {
                    SupplyOutcome::BatchTriggered { results, .. }
                        if matches!(
                            results.as_slice(),
                            [Err(skin_editor_audio::RuntimeError::ResourceLimit)]
                        ) =>
                    {
                        rejected += 1
                    }
                    other => return Err(format!("33rd voice was not refused: {other:?}")),
                }
            }
            consume(&output, rate, 1500)?;
        }
    }
    phase("isolation");
    // 16+16 remain alive from the final 32-voice burst.
    control(&mut client, false)?;
    let frozen = output.stats();
    consume(&output, rate, 100)?;
    let after = output.stats();
    if after.lane_nonzero_frames[0] != frozen.lane_nonzero_frames[0]
        || after.lane_nonzero_frames[1] <= frozen.lane_nonzero_frames[1]
    {
        return Err("scene freeze leaked or stopped audition".into());
    }
    control(&mut client, true)?;
    let id = client
        .trigger_selected(Channel::Scene, AudioComponentId::new(13).unwrap())
        .map_err(|e| format!("{e:?}"))?;
    outcome(&mut client, id)?;
    let id = client.stop_audition().map_err(|e| format!("{e:?}"))?;
    outcome(&mut client, id)?;
    wait(&output, || {
        let s = output.stats();
        s.lane_acknowledged[1] >= s.lane_epochs[1]
    })?;
    let stopped = output.stats();
    consume(&output, rate, 100)?;
    let after = output.stats();
    if after.lane_nonzero_frames[1] != stopped.lane_nonzero_frames[1]
        || after.lane_nonzero_frames[0] <= stopped.lane_nonzero_frames[0]
    {
        return Err("audition stop leaked or stopped scene".into());
    }
    stop(&mut client, &output)?;
    let retimed_idle_frames = output.stats().retimed_idle_frames;
    let startup_observations = output.startup_observations();
    let stats = output.stats();
    if stats.failed
        || stats.empty_frames != steady_start
        || peak_voices != 32
        || rejected != 4
        || session.snapshot() != snapshot
        || session.history_len() != history
    {
        return Err(format!(
            "load/underflow/history validation failed: stats={stats:?}, steady_start={steady_start}, peak={peak_voices}, refusals={rejected}"
        ));
    }
    client.close();
    drop(client);
    drop(stream);
    drop(output);
    if budget.usage().bytes != 0 {
        return Err("PCM retained after close".into());
    }
    phase("closed");
    std::thread::sleep(Duration::from_secs(2));
    let report = format!(
        "{{\"schema\":1,\"period_frames\":{period_frames},\"idle_reserve_frames\":{idle_reserve_frames},\"retimed_idle_frames\":{retimed_idle_frames},\"startup_observations\":{startup_observations:?},\"rate\":{rate},\"channels\":{channels},\"format\":\"{format}\",\"latency_upper_bound_ms\":{:?},\"receipt_ms\":{:?},\"max_post_receipt_poll_gap_ms\":{max_poll_gap_ms},\"peak_voices\":{peak_voices},\"capacity_refusals\":{rejected},\"steady_empty_frames\":{},\"startup_empty_frames\":{steady_start},\"callbacks\":{},\"lane_nonzero_frames\":{:?},\"pcm_peak_bytes\":{},\"pcm_final_bytes\":0,\"raw_bytes\":{},\"dual_channel_isolation\":true,\"project_history_unchanged\":true,\"device_consumption_or_acoustic_verified\":false}}\n",
        latency,
        receipt_ms,
        stats.empty_frames - steady_start,
        stats.callbacks,
        stats.lane_nonzero_frames,
        budget.usage().peak_bytes,
        store.usage().map_err(|e| format!("{e:?}"))?.bytes
    );
    std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path.join("report.json"))
        .map_err(|e| e.to_string())?
        .write_all(report.as_bytes())
        .map_err(|e| e.to_string())?;
    phase("done");
    Ok(())
}
