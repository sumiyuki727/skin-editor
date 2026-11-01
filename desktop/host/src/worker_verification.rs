//! Fixed native candidate through the existing bounded controls + WAV worker.
use crate::audio_verification::{component, submit};
use crate::load_verification::phase;
use crate::supply_verification::draft;
use skin_editor_application::{values::*, *};
use skin_editor_audio::{BackgroundTrack, Channel, PcmBudget};
use skin_editor_infrastructure::{
    AudioSupplyClient, BlobStore, SupplyOutcome, SupplyReceipt, WavDecoder,
    separated_native::{FirstCopy, NativeExecutor, PreparedDevice},
};
use std::time::{Duration, Instant};
fn backend(e: impl std::fmt::Debug) -> String {
    format!("{e:?}")
}
fn wav(rate: u32, channels: u16, frames: u32) -> Vec<u8> {
    let size = frames * u32::from(channels) * 2;
    let mut b = Vec::with_capacity(size as usize + 44);
    b.extend(b"RIFF");
    b.extend((36 + size).to_le_bytes());
    b.extend(b"WAVEfmt ");
    b.extend(16u32.to_le_bytes());
    b.extend(1u16.to_le_bytes());
    b.extend(channels.to_le_bytes());
    b.extend(rate.to_le_bytes());
    b.extend((rate * u32::from(channels) * 2).to_le_bytes());
    b.extend((channels * 2).to_le_bytes());
    b.extend(16u16.to_le_bytes());
    b.extend(b"data");
    b.extend(size.to_le_bytes());
    for _ in 0..frames * u32::from(channels) {
        b.extend(4i16.to_le_bytes());
    }
    b
}
fn healthy(client: &AudioSupplyClient, native: &NativeExecutor) -> Result<(), String> {
    if let Some(error) = client.status().map_err(backend)?.fault {
        return Err(error);
    }
    let s = native.stats();
    if s.device_fault || s.render_errors > 0 || s.observation_overflows > 0 {
        return Err(format!("native fault: {s:?}"));
    }
    Ok(())
}
fn receipt(client: &mut AudioSupplyClient, id: u64) -> Result<SupplyReceipt, String> {
    let until = Instant::now() + Duration::from_secs(3);
    loop {
        let r = client
            .wait_receipt(Duration::from_millis(100))
            .map_err(backend)?;
        if let Some(r) = r {
            if r.id == id {
                return Ok(r);
            }
            r.result.map_err(backend)?;
        }
        if Instant::now() >= until {
            return Err("worker receipt timeout".into());
        }
    }
}
fn outcome(client: &mut AudioSupplyClient, id: u64) -> Result<SupplyOutcome, String> {
    receipt(client, id)?.result.map_err(backend)
}
fn ready(
    client: &mut AudioSupplyClient,
    session: &ProjectSession,
    native: &NativeExecutor,
) -> Result<(), String> {
    client.synchronize(session).map_err(backend)?;
    let until = Instant::now() + Duration::from_secs(3);
    loop {
        while let Some(r) = client.poll() {
            r.result.map_err(backend)?;
        }
        healthy(client, native)?;
        if client.status().map_err(backend)?.ready_components == 1 && !client.pending_controls() {
            return Ok(());
        }
        if Instant::now() >= until {
            return Err("worker WAV readiness timeout".into());
        }
        std::thread::sleep(Duration::from_millis(1));
    }
}
fn barrier(client: &mut AudioSupplyClient) -> Result<(), String> {
    let id = client.barrier().map_err(backend)?;
    outcome(client, id)?;
    Ok(())
}
fn stop(client: &mut AudioSupplyClient) -> Result<(), String> {
    let id = client.stop_audition().map_err(backend)?;
    outcome(client, id)?;
    barrier(client)
}
struct Probe {
    request: u64,
    lane: Channel,
    at: Instant,
    voice: Option<u64>,
    receipt_ms: f64,
}
pub fn run(destination: &str) -> Result<(), String> {
    run_mode(destination, false)
}
pub fn streamed(destination: &str) -> Result<(), String> {
    run_mode(destination, true)
}
fn run_mode(destination: &str, streamed: bool) -> Result<(), String> {
    let path = std::path::Path::new(destination);
    if path.exists() {
        return Err("worker diagnostic destination must be new".into());
    }
    std::fs::create_dir_all(path).map_err(|e| e.to_string())?;
    let pool = PcmBudget::default();
    let store = BlobStore::default();
    let device = PreparedDevice::probe()?;
    let (rate, channels) = device.format();
    let mut session =
        ProjectSession::with_resources(1, 1, Box::new(store.clone()), SessionLimits::default())
            .map_err(backend)?;
    let mut next = 0;
    let mut originals = Vec::new();
    let mut commands = Vec::new();
    for (n, frames) in [(1, rate / 100), (2, rate * 3)] {
        let raw = store
            .stage_audio(wav(rate, channels, frames), &WavDecoder)
            .map_err(backend)?;
        commands.push(Command::RegisterAsset {
            asset: Asset::new(
                AssetId::new(n).unwrap(),
                format!("self worker {n}.wav"),
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
            SoundSlotKey::new("self.worker".into()).unwrap(),
            AssetId::new(1).unwrap(),
        ),
    });
    submit(&mut session, &mut next, commands)?;
    let (mut client, consumer, stream) = if streamed {
        let (source, observation) = crate::stream_fixture::open(path, rate, channels, &pool)?;
        let (client, consumer) =
            AudioSupplyClient::spawn_streamed(source, rate, channels, pool.clone(), store)
                .map_err(backend)?;
        (client, consumer, Some(observation))
    } else {
        let mut pcm = pool
            .reserve(rate as usize * usize::from(channels) * 10)
            .map_err(backend)?
            .allocate();
        for (frame, values) in pcm
            .samples_mut()
            .unwrap()
            .chunks_exact_mut(usize::from(channels))
            .enumerate()
        {
            values.fill(
                (std::f32::consts::TAU * 220.0 * frame as f32 / rate as f32).sin() * 0.000025,
            );
        }
        let track = BackgroundTrack::new(pcm, rate, channels).map_err(backend)?;
        let (client, consumer) =
            AudioSupplyClient::spawn_separated(track, rate, channels, pool.clone(), store)
                .map_err(backend)?;
        (client, consumer, None)
    };
    let mut native = device.attach(consumer);
    let period = native.period_frames();
    let result = (|| -> Result<String, String> {
        native.play()?;
        phase("idle");
        ready(&mut client, &session, &native)?;
        std::thread::sleep(Duration::from_millis(1100));
        prime(&client)?;
        let id = client.scene_running(true).map_err(backend)?;
        outcome(&mut client, id)?;
        phase("voices_1");
        let mut rows = Vec::new();
        for round in 0..64 {
            let mut pending = Vec::new();
            let mut firsts: Vec<FirstCopy> = Vec::new();
            for lane in [Channel::Scene, Channel::Audition] {
                let at = Instant::now();
                let request = client
                    .trigger(&session, lane, component())
                    .map_err(backend)?;
                pending.push(Probe {
                    request,
                    lane,
                    at,
                    voice: None,
                    receipt_ms: 0.0,
                });
            }
            let until = Instant::now() + Duration::from_secs(3);
            while !pending.is_empty() {
                while let Some(r) = client.poll() {
                    let p = pending
                        .iter_mut()
                        .find(|p| p.request == r.id)
                        .ok_or("unexpected control receipt")?;
                    let SupplyOutcome::Triggered(t) = r.result.map_err(backend)? else {
                        return Err("expected trigger receipt".into());
                    };
                    p.voice = Some(t.voice.ok_or("expected non-silent voice")?);
                    p.receipt_ms = p.at.elapsed().as_secs_f64() * 1000.0;
                }
                while let Some(first) = native.next_first() {
                    firsts.push(first);
                }
                let mut i = 0;
                while i < pending.len() {
                    if let Some(j) = firsts.iter().position(|f| {
                        Some(f.voice) == pending[i].voice && f.channel == pending[i].lane
                    }) {
                        let p = pending.remove(i);
                        let f = firsts.remove(j);
                        let upper = p.at.elapsed().as_secs_f64() * 1000.0;
                        rows.push(format!(
                            "[{round},{},{},{},{:.9},{upper:.9},{},{},{},{}]",
                            usize::from(p.lane == Channel::Audition),
                            p.request,
                            f.voice,
                            p.receipt_ms,
                            f.callback_nanos,
                            f.callback_ordinal,
                            f.offset_frames,
                            f.callback_frames
                        ));
                    } else {
                        i += 1;
                    }
                }
                healthy(&client, &native)?;
                if Instant::now() >= until {
                    return Err("full request to copy timeout".into());
                }
                if !pending.is_empty() {
                    std::thread::sleep(Duration::from_millis(1));
                }
            }
            if !firsts.is_empty() {
                return Err("unexpected or duplicate native voice".into());
            }
            std::thread::sleep(Duration::from_millis(18 + round % 7));
        }
        let seeks_before = stream.as_ref().map(|o| o.stats().seeks);
        let before = client.status().map_err(backend)?.copied_background;
        submit(
            &mut session,
            &mut next,
            vec![Command::SetAudioAsset {
                component: component(),
                asset: AssetId::new(2).unwrap(),
            }],
        )?;
        ready(&mut client, &session, &native)?;
        let hot_continuity = client.status().map_err(backend)?.copied_background >= before
            && stream.as_ref().map(|o| o.stats().seeks) == seeks_before;
        phase("voices_32");
        let entries: Vec<_> = (0..32)
            .map(|i| {
                (
                    if i % 2 == 0 {
                        Channel::Scene
                    } else {
                        Channel::Audition
                    },
                    component(),
                )
            })
            .collect();
        let id = client.trigger_batch_selected(&entries).map_err(backend)?;
        let SupplyOutcome::BatchTriggered { results, .. } = outcome(&mut client, id)? else {
            return Err("missing batch receipt".into());
        };
        let mut burst = Vec::new();
        for r in results {
            burst.push(r.map_err(backend)?.voice.ok_or("missing batch voice")?);
        }
        let peak = client
            .status()
            .map_err(backend)?
            .voices
            .iter()
            .sum::<usize>();
        let id = client
            .trigger(&session, Channel::Audition, component())
            .map_err(backend)?;
        let refused = matches!(receipt(&mut client,id)?.result,Err(skin_editor_infrastructure::SupplyError::Backend(e)) if e=="ResourceLimit");
        std::thread::sleep(Duration::from_millis(2100));
        let mut actual = Vec::new();
        while let Some(f) = native.next_first() {
            actual.push(f.voice);
        }
        burst.sort_unstable();
        actual.sort_unstable();
        let burst_exact = burst == actual;
        phase("controls");
        let id = client.scene_running(false).map_err(backend)?;
        let SupplyOutcome::Frozen {
            next_background_frame: anchor,
        } = outcome(&mut client, id)?
        else {
            return Err("missing freeze receipt".into());
        };
        std::thread::sleep(Duration::from_millis(80));
        let s = client.status().map_err(backend)?;
        let pause_isolated = s.frozen && s.copied_background == anchor && s.voices == [0, 16];
        stop(&mut client)?;
        let quiet = native.stats().nonzero_samples;
        std::thread::sleep(Duration::from_millis(80));
        let stop_quiet = native.stats().nonzero_samples == quiet
            && client.status().map_err(backend)?.voices == [0, 0];
        prime(&client)?;
        let id = client.scene_running(true).map_err(backend)?;
        outcome(&mut client, id)?;
        std::thread::sleep(Duration::from_millis(40));
        let resumed = client.status().map_err(backend)?.copied_background > anchor;
        let baseline_pcm = pool.usage().bytes;
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
        ready(&mut client, &session, &native)?;
        let id = client
            .trigger(&session, Channel::Audition, component())
            .map_err(backend)?;
        outcome(&mut client, id)?;
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
        ready(&mut client, &session, &native)?;
        barrier(&mut client)?;
        let until = Instant::now() + Duration::from_secs(3);
        while pool.usage().bytes > baseline_pcm {
            if Instant::now() >= until {
                return Err("cancelled PCM not released".into());
            }
            barrier(&mut client)?;
            std::thread::sleep(Duration::from_millis(1));
        }
        let cancel_clean = session.snapshot() == snapshot
            && session.history_len() == nodes
            && client.status().map_err(backend)?.voices == [0, 0]
            && client.status().map_err(backend)?.retained_drafts.is_empty();
        let before_history = client.status().map_err(backend)?.copied_background;
        let id = client
            .trigger(&session, Channel::Audition, component())
            .map_err(backend)?;
        outcome(&mut client, id)?;
        next += 1;
        session
            .navigate(HistoryRequest {
                api_version: API_VERSION,
                session_id: session.project_info().session_id,
                request_id: RequestId::new(next).unwrap(),
                expected: session.project_info().state,
                action: HistoryAction::Undo,
            })
            .map_err(backend)?;
        ready(&mut client, &session, &native)?;
        barrier(&mut client)?;
        let s = client.status().map_err(backend)?;
        let history_clean = s.voices == [0, 0] && s.copied_background >= before_history;
        let source_not_eof =
            s.generated_background < u64::from(rate) * if streamed { 360 } else { 10 };
        let mut streamed_checks = String::new();
        if streamed {
            // Location is frozen and explicit. A three-second audition must survive it.
            submit(
                &mut session,
                &mut next,
                vec![Command::SetAudioAsset {
                    component: component(),
                    asset: AssetId::new(2).unwrap(),
                }],
            )?;
            ready(&mut client, &session, &native)?;
            let id = client
                .trigger(&session, Channel::Audition, component())
                .map_err(backend)?;
            outcome(&mut client, id)?;
            let before = s.copied_background;
            let target = u64::from(rate) * 120 + 7;
            let id = client.seek_background(target).map_err(backend)?;
            let located = matches!(outcome(&mut client,id)?,SupplyOutcome::Located{frame,previous_copy_anchor} if frame==target&&previous_copy_anchor>=before&&previous_copy_anchor<target);
            location_visible(&client, target)?;
            let s = client.status().map_err(backend)?;
            let seek_isolated = s.frozen && s.background_position == target && s.voices == [0, 1];
            stop(&mut client)?;
            prime(&client)?;
            let id = client.scene_running(true).map_err(backend)?;
            outcome(&mut client, id)?;
            // A Located receipt rebases generation; it does not pretend that a
            // new native copy has occurred. Wait for the first copy in this epoch.
            let until = Instant::now() + Duration::from_secs(3);
            while client.status().map_err(backend)?.copied_background < target {
                healthy(&client, &native)?;
                if Instant::now() >= until {
                    return Err("new location was not copied".into());
                }
                std::thread::sleep(Duration::from_millis(1));
            }
            phase("streaming");
            let start = Instant::now();
            let mut previous = target;
            let mut monotonic = true;
            while start.elapsed() < Duration::from_secs(20) {
                healthy(&client, &native)?;
                let now = client.status().map_err(backend)?.copied_background;
                monotonic &= now >= previous;
                previous = now;
                std::thread::sleep(Duration::from_millis(10));
            }
            let seconds = start.elapsed().as_secs_f64();
            let supplied = previous - target;
            phase("controls");
            let id = client.seek_background(31).map_err(backend)?;
            let reply = outcome(&mut client, id)?;
            location_visible(&client, 31)?;
            let backwards = matches!(reply, SupplyOutcome::Located { frame: 31, .. })
                && client.status().map_err(backend)?.background_position == 31;
            prime(&client)?;
            let id = client.scene_running(true).map_err(backend)?;
            outcome(&mut client, id)?;
            std::thread::sleep(Duration::from_millis(40));
            let position = client.status().map_err(backend)?.copied_background;
            let backwards = backwards && position > 31 && position < u64::from(rate);
            streamed_checks = format!(
                ",\"located\":{located},\"seek_isolated\":{seek_isolated},\"stream_monotonic\":{monotonic},\"stream_seconds\":{seconds},\"stream_frames\":{supplied},\"backwards\":{backwards}"
            );
        }
        healthy(&client, &native)?;
        Ok(format!(
            "\"probes\":[{}],\"peak_voices\":{peak},\"refused_33\":{refused},\"burst_exact\":{burst_exact},\"hot_continuity\":{hot_continuity},\"pause_isolated\":{pause_isolated},\"stop_quiet\":{stop_quiet},\"resumed\":{resumed},\"cancel_clean\":{cancel_clean},\"history_clean\":{history_clean},\"source_not_eof\":{source_not_eof},\"max_owner_gap_micros\":{}",
            rows.join(","),
            s.max_supply_gap_micros
        ) + &streamed_checks)
    })();
    client.close();
    let (stopped, stats) = native.stop()?;
    client
        .finish_separated(stopped)
        .map_err(|_| "foreign worker endpoint")?;
    drop(client);
    let usage = pool.usage();
    phase("closed");
    std::thread::sleep(Duration::from_millis(1100));
    let details = match result {
        Ok(d) => d,
        Err(e) => {
            std::fs::write(
                path.join("failure.txt"),
                format!("{e}\n{stats:?}\n{usage:?}"),
            )
            .map_err(|e| e.to_string())?;
            return Err(e);
        }
    };
    let stream_fields=stream.map_or_else(||String::from("\"streamed\":false"),|o|{let s=o.stats();format!("\"streamed\":true,\"source_frames\":{},\"source_read_bytes\":{},\"source_seeks\":{},\"source_raw_peak\":{},\"source_raw_final\":{},\"source_fault\":{}",u64::from(rate)*360,s.read_bytes,s.seeks,s.raw_peak_bytes,s.raw_bytes,s.fault)});
    let report = format!(
        "{{{stream_fields},\"callback_resource_releases\":{},\"endpoint_returns_in_render\":{},\"endpoint_returns_outside_render\":{},\"schema\":1,\"scope\":\"bounded-worker-full-request\",\"rate\":{rate},\"channels\":{channels},\"period_frames\":{period},{details},\"callbacks\":{},\"missing_frames\":{},\"render_errors\":{},\"observation_overflows\":{},\"endpoint_returns\":{},\"device_fault\":{},\"nonzero_samples\":{},\"pcm_peak_bytes\":{},\"pcm_final_bytes\":{}}}",
        stats.callback_resource_releases,
        stats.endpoint_returns_in_render,
        stats.endpoint_returns_outside_render,
        stats.callbacks,
        stats.missing_frames,
        stats.render_errors,
        stats.observation_overflows,
        stats.endpoint_returns,
        stats.device_fault,
        stats.nonzero_samples,
        usage.peak_bytes,
        usage.bytes
    );
    std::fs::write(path.join("report.json"), report).map_err(|e| e.to_string())?;
    Ok(())
}

fn prime(client: &AudioSupplyClient) -> Result<(), String> {
    let until = Instant::now() + Duration::from_secs(3);
    while !client.status().map_err(backend)?.background_primed {
        if Instant::now() >= until {
            return Err("background readiness timeout".into());
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    Ok(())
}
fn location_visible(client: &AudioSupplyClient, frame: u64) -> Result<(), String> {
    let until = Instant::now() + Duration::from_secs(3);
    loop {
        let s = client.status().map_err(backend)?;
        if s.frozen && !s.freezing && s.background_position == frame {
            return Ok(());
        }
        if Instant::now() >= until {
            return Err("location status publication timeout".into());
        }
        std::thread::sleep(Duration::from_millis(1));
    }
}
