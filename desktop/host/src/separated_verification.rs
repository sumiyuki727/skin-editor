//! Fixed, opt-in native short diagnostic. Generated references, no GUI/media.
use skin_editor_application::values::{AudioComponentId, AudioMetadata, BlobRef, Gain};
use skin_editor_audio::{
    BackgroundTrack, Channel, Origin, PcmBudget, PreparedSound, Runtime, SoundPlan,
};
use skin_editor_infrastructure::{
    separated_native::{NativeExecutor, PreparedDevice},
    separated_output::{self, BackgroundOwner, Phase},
};
use std::io::Write;
use std::time::{Duration, Instant};
fn origin(revision: u64) -> Origin {
    Origin {
        session: 1,
        node: 1,
        revision,
        navigation_revision: 1,
        draft: None,
    }
}
fn sound(r: &mut Runtime, pool: &PcmBudget, version: u64, frames: u64) -> Result<(), String> {
    let (rate, channels) = r.output_format();
    let plan = SoundPlan {
        target: AudioComponentId::new(1).unwrap().into(),
        source: BlobRef::new([version as u8; 32], frames),
        metadata: Some(AudioMetadata::new(rate, channels, frames).unwrap()),
        gain: Gain::UNITY,
        visible: true,
    };
    let job = r
        .synchronize(origin(version), &[plan])
        .map_err(|e| format!("{e:?}"))?[0];
    let mut pcm = pool
        .reserve(frames as usize * usize::from(channels))
        .map_err(|e| format!("{e:?}"))?
        .allocate();
    // Nonzero first sample: the descriptor's first copy is an effective sample.
    pcm.samples_mut().unwrap().fill(0.0001);
    let prepared = PreparedSound::from_decoded(plan, pcm).map_err(|e| format!("{e:?}"))?;
    r.complete(job, Ok(prepared))
        .map_err(|e| format!("{e:?}"))?;
    Ok(())
}
fn trigger(r: &mut Runtime, lane: Channel) -> Result<u64, String> {
    r.trigger(
        r.token(lane),
        AudioComponentId::new(1).unwrap(),
        r.mix_frame(),
    )
    .map_err(|e| format!("{e:?}"))?
    .voice
    .ok_or("expected non-silent voice".into())
}
fn tick(
    owner: &mut BackgroundOwner,
    r: &mut Runtime,
    native: &NativeExecutor,
) -> Result<(), String> {
    let s = native.stats();
    if s.device_fault || s.render_errors != 0 || s.observation_overflows != 0 {
        return Err(format!("native callback fault: {s:?}"));
    }
    owner.pump(r).map_err(|e| format!("pump: {e:?}"))?;
    if owner.position().phase != Phase::Freezing {
        owner.refill().map_err(|e| format!("refill: {e:?}"))?;
    }
    Ok(())
}
fn wait(
    owner: &mut BackgroundOwner,
    r: &mut Runtime,
    native: &NativeExecutor,
    mut ready: impl FnMut(&Runtime, &BackgroundOwner) -> bool,
) -> Result<(), String> {
    let guard = Instant::now() + Duration::from_secs(3);
    loop {
        tick(owner, r, native)?;
        if ready(r, owner) {
            return Ok(());
        }
        if Instant::now() >= guard {
            return Err("native candidate timeout".into());
        }
        std::thread::sleep(Duration::from_millis(2));
    }
}
fn duration(
    owner: &mut BackgroundOwner,
    r: &mut Runtime,
    native: &NativeExecutor,
    span: Duration,
) -> Result<(), String> {
    let until = Instant::now() + span;
    wait(owner, r, native, |_, _| Instant::now() >= until)
}
fn phase(name: &str) {
    println!("{{\"phase\":\"{name}\"}}");
    std::io::stdout().flush().unwrap();
}
pub fn run(destination: &str) -> Result<(), String> {
    let path = std::path::Path::new(destination);
    if path.exists() {
        return Err("candidate diagnostic destination must be new".into());
    }
    std::fs::create_dir_all(path).map_err(|e| e.to_string())?;
    let pool = PcmBudget::default();
    let device = PreparedDevice::probe()?;
    let (rate, channels) = device.format();
    let mut runtime =
        Runtime::with_budget(rate, channels, pool.clone()).map_err(|e| format!("{e:?}"))?;
    runtime
        .enable_voice_handoff()
        .map_err(|e| format!("{e:?}"))?;
    sound(&mut runtime, &pool, 1, u64::from(rate) / 100)?;
    let mut pcm = pool
        .reserve(rate as usize * usize::from(channels) * 10)
        .map_err(|e| format!("{e:?}"))?
        .allocate();
    for (frame, samples) in pcm
        .samples_mut()
        .unwrap()
        .chunks_exact_mut(usize::from(channels))
        .enumerate()
    {
        let v = (std::f32::consts::TAU * 220.0 * frame as f32 / rate as f32).sin() * 0.0001;
        samples.fill(v);
    }
    let track = BackgroundTrack::new(pcm, rate, channels).map_err(|e| format!("{e:?}"))?;
    let (mut owner, endpoint) =
        separated_output::boundary(&mut runtime, track, origin(1)).map_err(|e| format!("{e:?}"))?;
    let mut native = device.attach(endpoint);
    let period = native.period_frames();
    let result = (|| -> Result<String, String> {
        phase("idle");
        native.play()?;
        duration(
            &mut owner,
            &mut runtime,
            &native,
            Duration::from_millis(1100),
        )?;
        owner.resume(&mut runtime).map_err(|e| format!("{e:?}"))?;
        phase("voices_1");
        let mut probes = Vec::new();
        for round in 0..64 {
            tick(&mut owner, &mut runtime, &native)?;
            let mut pending = Vec::new();
            for lane in [Channel::Scene, Channel::Audition] {
                let at = Instant::now();
                let id = trigger(&mut runtime, lane)?;
                pending.push((id, lane, at));
            }
            let guard = Instant::now() + Duration::from_secs(3);
            while !pending.is_empty() {
                tick(&mut owner, &mut runtime, &native)?;
                while let Some(first) = native.next_first() {
                    let index = pending
                        .iter()
                        .position(|(id, lane, _)| *id == first.voice && *lane == first.channel)
                        .ok_or("unexpected or duplicate voice completion")?;
                    let (_, lane, request) = pending.remove(index);
                    let upper_ms = request.elapsed().as_secs_f64() * 1000.0;
                    probes.push(format!(
                        "[{}, {}, {}, {:.9}, {}, {}, {}, {}]",
                        round,
                        usize::from(lane == Channel::Audition),
                        first.voice,
                        upper_ms,
                        first.callback_nanos,
                        first.callback_ordinal,
                        first.offset_frames,
                        first.callback_frames
                    ));
                }
                if Instant::now() >= guard {
                    return Err("voice first-copy timeout".into());
                }
                if !pending.is_empty() {
                    std::thread::sleep(Duration::from_millis(2));
                }
            }
            // Deterministic plan, varied host admission phase; this sleep is
            // only the native load schedule, never a concurrency test oracle.
            duration(
                &mut owner,
                &mut runtime,
                &native,
                Duration::from_millis(18 + round % 7),
            )?;
        }
        wait(&mut owner, &mut runtime, &native, |r, _| {
            r.voices().count() == 0
        })?;
        let before = owner.position();
        sound(&mut runtime, &pool, 2, u64::from(rate) * 3)?;
        tick(&mut owner, &mut runtime, &native)?;
        let after = owner.position();
        let hot_continuity =
            before.epoch == after.epoch && after.timeline_frame >= before.timeline_frame;
        phase("voices_32");
        let mut burst = Vec::new();
        for i in 0..32 {
            burst.push(trigger(
                &mut runtime,
                if i % 2 == 0 {
                    Channel::Scene
                } else {
                    Channel::Audition
                },
            )?);
        }
        let refused = runtime
            .trigger(
                runtime.token(Channel::Audition),
                AudioComponentId::new(1).unwrap(),
                runtime.mix_frame(),
            )
            .is_err();
        tick(&mut owner, &mut runtime, &native)?;
        let peak = runtime.voices().count();
        duration(
            &mut owner,
            &mut runtime,
            &native,
            Duration::from_millis(2100),
        )?;
        let mut observed = Vec::new();
        while let Some(row) = native.next_first() {
            observed.push(row.voice);
        }
        burst.sort_unstable();
        observed.sort_unstable();
        let burst_exact = burst == observed;
        phase("controls");
        owner.pause(&mut runtime).map_err(|e| format!("{e:?}"))?;
        wait(&mut owner, &mut runtime, &native, |_, o| {
            o.position().phase == Phase::Frozen
        })?;
        let anchor = owner.position().timeline_frame;
        let audition_survives = runtime.voices().all(|v| v.channel == Channel::Audition)
            && runtime.voices().count() == 16;
        duration(&mut owner, &mut runtime, &native, Duration::from_millis(80))?;
        let pause_stable = owner.position().timeline_frame == anchor;
        runtime
            .stop_audition(runtime.token(Channel::Audition))
            .map_err(|e| format!("{e:?}"))?;
        wait(&mut owner, &mut runtime, &native, |r, _| {
            r.voices().count() == 0
        })?;
        owner.resume(&mut runtime).map_err(|e| format!("{e:?}"))?;
        duration(&mut owner, &mut runtime, &native, Duration::from_millis(80))?;
        let resumed = owner.position().timeline_frame > anchor;
        let pre_seek_not_eof = owner.position().generated_frame < u64::from(rate) * 10;
        owner
            .seek(&mut runtime, 100)
            .map_err(|e| format!("{e:?}"))?;
        wait(&mut owner, &mut runtime, &native, |_, o| {
            o.position().phase == Phase::Frozen
        })?;
        let seek_exact = owner.position().timeline_frame == 100;
        owner.resume(&mut runtime).map_err(|e| format!("{e:?}"))?;
        duration(&mut owner, &mut runtime, &native, Duration::from_millis(80))?;
        let seeking_resumed = owner.position().timeline_frame > 100;
        let source_not_eof =
            pre_seek_not_eof && owner.position().generated_frame < u64::from(rate) * 10;
        Ok(format!(
            "\"probes\":[{}],\"peak_voices\":{peak},\"refused_33\":{refused},\"burst_exact\":{burst_exact},\"hot_continuity\":{hot_continuity},\"audition_survives_pause\":{audition_survives},\"pause_stable\":{pause_stable},\"resumed\":{resumed},\"seek_exact\":{seek_exact},\"seeking_resumed\":{seeking_resumed},\"source_not_eof\":{source_not_eof}",
            probes.join(",")
        ))
    })();
    owner.close();
    let (stopped, stats) = native.stop()?;
    owner
        .finish(&mut runtime, stopped)
        .map_err(|_| "foreign native endpoint")?;
    let usage = pool.usage();
    phase("closed");
    std::thread::sleep(Duration::from_millis(1100));
    let details = match result {
        Ok(details) => details,
        Err(e) => {
            std::fs::write(
                path.join("failure.txt"),
                format!("{e}\n{stats:?}\n{usage:?}\n"),
            )
            .map_err(|e| e.to_string())?;
            return Err(e);
        }
    };
    let text = format!(
        "{{\"schema\":1,\"rate\":{rate},\"channels\":{channels},\"period_frames\":{period},{details},\"callbacks\":{},\"frames\":{},\"max_callback_frames\":{},\"missing_frames\":{},\"render_errors\":{},\"observation_overflows\":{},\"endpoint_returns\":{},\"device_fault\":{},\"nonzero_samples\":{},\"pcm_peak_bytes\":{},\"pcm_final_bytes\":{}}}",
        stats.callbacks,
        stats.frames,
        stats.max_callback_frames,
        stats.missing_frames,
        stats.render_errors,
        stats.observation_overflows,
        stats.endpoint_returns,
        stats.device_fault,
        stats.nonzero_samples,
        usage.peak_bytes,
        usage.bytes
    );
    std::fs::write(path.join("report.json"), text).map_err(|e| e.to_string())?;
    Ok(())
}
