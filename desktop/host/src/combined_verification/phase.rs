//! Explicit phase experiment, separate from standard graphics acceptance.
use super::*;
use skin_editor_infrastructure::{BlobStore, WavDecoder};
const BINS: usize = 20;
const REPEATS: usize = 8;
const COUNT: usize = BINS * REPEATS;
const REFERENCE_ATTEMPTS: usize = 32;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RecoveryProbeMode {
    Natural,
    WithTrigger,
    AfterRefill,
}

fn reference(
    mut read: impl FnMut() -> Option<[u64; 4]>,
    mut pause: impl FnMut() -> Result<(), String>,
) -> Result<([u64; 4], usize), String> {
    for attempt in 0..REFERENCE_ATTEMPTS {
        if let Some(row) = read() {
            return Ok((row, attempt));
        }
        if attempt + 1 < REFERENCE_ATTEMPTS {
            pause()?;
        }
    }
    Err("callback reference remained incomplete within bounded reads".into())
}
fn callback_reference(output: &OutputControl) -> Result<([u64; 4], usize), String> {
    reference(
        || output.callback_progress(),
        || {
            if output.is_closed() {
                return Err("phase output closed".into());
            }
            std::thread::sleep(Duration::from_millis(1));
            Ok(())
        },
    )
}

fn future_phase(now: u64, entry: u64, period: u64, bin: usize) -> Result<u64, String> {
    if period == 0 || period > 1_000_000_000 || entry > now || bin >= BINS {
        return Err("invalid phase reference".into());
    }
    let cycles = ((now - entry) / period)
        .checked_add(3)
        .ok_or("phase overflow")?;
    entry
        .checked_add(cycles.checked_mul(period).ok_or("phase overflow")?)
        .and_then(|v| v.checked_add(period * bin as u64 / BINS as u64))
        .ok_or("phase overflow".into())
}
pub fn run(destination: &str) -> Result<(), String> {
    run_with_policy(
        destination,
        skin_editor_infrastructure::SupplyPolicy::FixedAhead,
        RecoveryProbeMode::Natural,
    )
}
pub fn run_recovery(destination: &str) -> Result<(), String> {
    run_with_policy(
        destination,
        skin_editor_infrastructure::SupplyPolicy::RecoverLateWake,
        RecoveryProbeMode::Natural,
    )
}
pub fn run_controlled(destination: &str, after_refill: bool) -> Result<(), String> {
    run_with_policy(
        destination,
        skin_editor_infrastructure::SupplyPolicy::RecoverLateWake,
        if after_refill {
            RecoveryProbeMode::AfterRefill
        } else {
            RecoveryProbeMode::WithTrigger
        },
    )
}
fn run_with_policy(
    destination: &str,
    policy: skin_editor_infrastructure::SupplyPolicy,
    mode: RecoveryProbeMode,
) -> Result<(), String> {
    let path = PathBuf::from(destination);
    if !path.is_absolute() || path.exists() {
        return Err("new absolute phase directory required".into());
    }
    std::fs::create_dir_all(&path).map_err(error)?;
    phase("setup");
    let budget = PcmBudget::default();
    let store = BlobStore::default();
    let mut session =
        ProjectSession::with_resources(1, 1, Box::new(store.clone()), SessionLimits::default())
            .map_err(error)?;
    let mut next = 0;
    let mut commands = vec![];
    let mut pins = vec![];
    for n in 0..16 {
        let raw = store
            .stage_audio(wav(n as u32), &WavDecoder)
            .map_err(error)?;
        let asset = AssetId::new(n as u128 + 1).unwrap();
        commands.push(Command::RegisterAsset {
            asset: Asset::new(
                asset,
                format!("self {n}.wav"),
                raw.reference(),
                AssetKind::Audio(raw.audio_metadata().unwrap()),
            )
            .map_err(error)?,
        });
        commands.push(Command::CreateAudioComponent {
            component: AudioComponent::new(
                sound(n),
                SoundSlotKey::new(format!("self.phase{n}")).map_err(error)?,
                asset,
            ),
        });
        pins.push(raw);
    }
    submit(&mut session, &mut next, commands)?;
    let native = NativeOutput::open(&budget)?;
    let (rate, channels, period) = (native.rate, native.channels, native.period_frames);
    let (stream, mut producer) = native.into_parts();
    producer.configure_supply_policy(policy)?;
    let mut client =
        AudioSupplyClient::spawn(producer, rate, channels, budget.clone(), store.clone())
            .map_err(error)?;
    let output = client.output().map_err(|e| format!("{e:?}"))?;
    output.capture_submissions()?;
    output.capture_startup()?;
    output.set_diagnostic_stage(1)?;
    phase("stream_start");
    stream.play()?;
    output.set_diagnostic_stage(2)?;
    phase("prepare");
    session.synchronize_audio(&mut client).map_err(error)?;
    ready_all(&mut client, &output, &session)?;
    output.set_diagnostic_stage(3)?;
    phase("ready");
    let id = client.scene_running(true).map_err(error)?;
    outcome(&mut client, id)?;
    let mut observer = Observer::with_capacity(output.clone(), COUNT)?;
    let mut keepalive = Instant::now() - Duration::from_secs(2);
    let mut rows = [Vec::with_capacity(COUNT), Vec::with_capacity(COUNT)];
    let mut references = [Vec::with_capacity(COUNT), Vec::with_capacity(COUNT)];
    let mut reference_retries = 0;
    let mut recovery_rows = [Vec::with_capacity(COUNT), Vec::with_capacity(COUNT)];
    output.set_diagnostic_stage(4)?;
    phase("phase_scan");
    for index in 0..COUNT {
        for (lane_index, lane) in [Channel::Scene, Channel::Audition].into_iter().enumerate() {
            if keepalive.elapsed() >= Duration::from_millis(750) {
                let id = client
                    .trigger_selected(Channel::Scene, sound(12))
                    .map_err(error)?;
                outcome(&mut client, id)?;
                keepalive = Instant::now();
            }
            let (progress, retries) = callback_reference(&output)?;
            reference_retries += retries;
            let period_ns = u64::from(period) * 1_000_000_000 / u64::from(rate);
            let requested = (index * 7) % BINS;
            let anchor = Instant::now();
            let clock = stream.clock_sample()?;
            let target = future_phase(clock.stream_nanos, progress[1], period_ns, requested)?;
            let bounds = clock.callback_bounds_ms(target, anchor);
            let due = anchor + Duration::from_secs_f64((bounds[0] + bounds[1]) / 2000.);
            // Native phase experiment, not a deterministic race oracle. Record
            // achieved phase and overshoot; no spin/yield or pacing change.
            while Instant::now() < due {
                if output.is_closed() {
                    return Err("phase output closed".into());
                }
                std::thread::sleep(Duration::from_millis(1));
            }
            let recovery_before = output.stats();
            if mode == RecoveryProbeMode::AfterRefill {
                let request = client.diagnostic_recovery_probe(None).map_err(error)?;
                if !matches!(outcome(&mut client, request)?, SupplyOutcome::Observed) {
                    return Err("unexpected recovery barrier outcome".into());
                }
            }
            let (reference, retries) = callback_reference(&output)?;
            reference_retries += retries;
            let clock = stream.clock_sample()?;
            let started = Instant::now();
            let id = if mode == RecoveryProbeMode::WithTrigger {
                client.diagnostic_recovery_probe(Some((lane, sound(0))))
            } else {
                client.trigger_selected(lane, sound(0))
            }
            .map_err(error)?;
            let voice = match outcome(&mut client, id)? {
                SupplyOutcome::Triggered(r) if !r.stale => {
                    r.voice.ok_or("phase trigger refused")?
                }
                other => return Err(format!("phase trigger: {other:?}")),
            };
            let recovery_after = output.stats();
            recovery_rows[lane_index].push([
                recovery_before.diagnostic_recovery_passes,
                recovery_after.diagnostic_recovery_passes,
                recovery_before.diagnostic_refill_passes,
                recovery_after.diagnostic_refill_passes,
                recovery_before.diagnostic_refill_frames,
                recovery_after.diagnostic_refill_frames,
            ]);
            let (done, ack) = mpsc::sync_channel(1);
            observer
                .sender
                .as_ref()
                .unwrap()
                .try_send(Probe {
                    callback_reference: Some(reference),
                    completed: Some(done),
                    started,
                    receipt_ms: started.elapsed().as_secs_f64() * 1000.,
                    started_in_play_ms: 0.,
                    voice,
                    lane: lane_index,
                    clock: Some(clock),
                })
                .map_err(error)?;
            let age = clock.callback_bounds_ms(reference[1], started);
            rows[lane_index].push([
                requested as f64,
                -age[1],
                -age[0],
                started.saturating_duration_since(due).as_secs_f64() * 1000.,
                period_ns as f64 / 1_000_000.,
            ]);
            references[lane_index].push(reference);
            // Specific Voice plus matching producer telemetry, not a guessed
            // delay before the next trigger/keepalive can overwrite the row.
            ack.recv_timeout(Duration::from_secs(5)).map_err(error)?;
        }
    }
    output.set_diagnostic_stage(5)?;
    phase("collect");
    let observed = observer.finish()?;
    output.set_diagnostic_stage(6)?;
    phase("stop");
    stop(&mut client, &output)?;
    let cleanup_stopped = client.status().map_err(error)?.voices == [0, 0];
    let max_gap = client.status().map_err(error)?.max_supply_gap_micros;
    let alignment = clock_alignment(&output, &stream)?;
    output.set_diagnostic_stage(7)?;
    phase("close");
    client.close();
    drop((client, stream));
    // Stream drop ends callback writes. Only now are totals and all stage rows
    // quiescent; a live mixed-atomic snapshot is not an exact stage boundary.
    let stats = output.stats();
    let stages = output.diagnostic_stage_counts();
    let startup = output.startup_observations();
    let underruns = output.underrun_observations();
    let contexts = output
        .underrun_contexts()
        .iter()
        .map(|row| row.map_or_else(|| "null".to_string(), |v| format!("{v:?}")))
        .collect::<Vec<_>>()
        .join(",");
    let scheduling = scheduling_json(&output, alignment, true);
    drop((output, observer, session, pins));
    let pcm = budget.usage();
    let raw = store.usage().map_err(error)?;
    phase("closed");
    std::thread::sleep(Duration::from_secs(2));
    let report = format!(
        concat!(
            "{{\"schema\":2,\"count_per_lane\":{},\"requested_bins\":{},\"repeats\":{},\"rate\":{},\"channels\":{},\"period_frames\":{},",
            "\"supply_policy\":\"{:?}\",\"recovery_passes\":{},\"max_recovery_target\":{},",
            "\"recovery_probe_mode\":\"{:?}\",\"diagnostic_recovery_passes\":{},\"diagnostic_refill_passes\":{},\"diagnostic_refill_frames\":{},\"recovery_rows\":{:?},",
            "\"reference_retries\":{},\"reference_attempt_limit\":{},",
            "\"cleanup_stopped\":{},",
            "\"phase_rows\":{:?},\"callback_references\":{:?},\"latency_upper_bound_ms\":{:?},\"submission_intervals_ms\":{:?},\"submission_causal_ms\":{:?},",
            "\"exact_voice_intervals_ms\":{:?},\"completion_markers\":{:?},\"requested_voice_ids\":{:?},\"max_probe_poll_gap_ms\":{},\"max_supply_gap_micros\":{},",
            "\"empty_frames\":{},\"empty_callbacks\":{},\"callbacks\":{},\"frames\":{},\"max_callback_frames\":{},\"max_empty_callback_frames\":{},\"retimed_idle_frames\":{},",
            "\"stage_counts\":{:?},\"startup_observations\":{:?},\"underrun_observations\":{:?},\"underrun_contexts\":[{}],\"scheduling\":{},",
            "\"pcm_peak_bytes\":{},\"pcm_final_bytes\":{},\"raw_peak_bytes\":{},\"raw_final_bytes\":{}}}\n"
        ),
        COUNT,
        BINS,
        REPEATS,
        rate,
        channels,
        period,
        policy,
        stats.recovery_passes,
        stats.max_recovery_target,
        mode,
        stats.diagnostic_recovery_passes,
        stats.diagnostic_refill_passes,
        stats.diagnostic_refill_frames,
        recovery_rows,
        reference_retries,
        REFERENCE_ATTEMPTS,
        cleanup_stopped,
        rows,
        references,
        observed.latencies,
        observed.intervals,
        observed.causal,
        observed.exact,
        observed.markers,
        observed.requested,
        observed.max_poll_gap_ms,
        max_gap,
        stats.empty_frames,
        stats.empty_callbacks,
        stats.callbacks,
        stats.frames,
        stats.max_callback_frames,
        stats.max_empty_callback_frames,
        stats.retimed_idle_frames,
        stages,
        startup,
        underruns,
        contexts,
        scheduling,
        pcm.peak_bytes,
        pcm.bytes,
        raw.peak_bytes,
        raw.bytes
    );
    std::fs::write(path.join("report.json"), report).map_err(error)?;
    phase("done");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn incomplete_reference_is_retried_and_returns_the_actual_later_snapshot() {
        let row = [7, 100, 480, 768];
        let mut reads = [None, None, Some(row)].into_iter();
        let mut pauses = 0;
        assert_eq!(
            reference(
                || reads.next().unwrap(),
                || {
                    pauses += 1;
                    Ok(())
                }
            )
            .unwrap(),
            (row, 2)
        );
        assert_eq!(pauses, 2);
    }
    #[test]
    fn reference_retry_is_bounded_and_closed_output_stops_immediately() {
        let mut reads = 0;
        let mut pauses = 0;
        assert!(
            reference(
                || {
                    reads += 1;
                    None
                },
                || {
                    pauses += 1;
                    Ok(())
                }
            )
            .is_err()
        );
        assert_eq!(reads, REFERENCE_ATTEMPTS);
        assert_eq!(pauses, REFERENCE_ATTEMPTS - 1);
        assert_eq!(
            reference(|| None, || Err("closed".into())),
            Err("closed".into())
        );
    }
    #[test]
    fn phase_deadlines_are_future_and_cover_all_bins_without_overflow() {
        let mut bins = [0; BINS];
        for index in 0..COUNT {
            let bin = index * 7 % BINS;
            bins[bin] += 1;
            let t = future_phase(123_000_000, 120_000_000, 10_000_000, bin).unwrap();
            assert_eq!(t, 150_000_000 + bin as u64 * 500_000);
        }
        assert_eq!(bins, [REPEATS; BINS]);
        for args in [
            (1, 2, 10, 0),
            (1, 0, 0, 0),
            (1, 0, 10, BINS),
            (u64::MAX - 1, u64::MAX - 2, 10, 0),
        ] {
            assert!(future_phase(args.0, args.1, args.2, args.3).is_err());
        }
    }
}
