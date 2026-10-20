#[cfg(windows)]
mod native {
    use p0_egui::{
        audio_output::Output,
        audio_transport::{Phase, Transport},
        streaming_decode::{DecoderWorker, DiagnosticGate},
        windows_audio::WindowsOutput,
    };
    use p0_wasapi::{
        Snapshot,
        buffered::{self, Config, Input},
    };
    use serde_json::{Value, json};
    use std::{
        fs::{self, File},
        path::PathBuf,
        thread,
        time::{Duration, Instant},
    };
    fn stats(s: buffered::Stats) -> Value {
        json!({"generation":s.generation,"active":s.active,"free_blocks":s.free_blocks,
        "queued_blocks":s.queued_blocks,"in_flight":s.in_flight,"max_queued_blocks":s.max_queued_blocks,"accepted_blocks":s.accepted_blocks,
        "stale_blocks":s.stale_blocks,"underruns":s.underruns,"decode_failures":s.decode_failures,"trace_overflow":s.trace_overflow,
        "native_discarded_frames":s.native_discarded_frames,"discarded_frames":s.discarded_frames,
        "source_origin_frame":s.pcm.source_origin_frame,"submitted_frames":s.pcm.submitted_frames,"packets":s.pcm.packets})
    }
    // Field drop order also stops output before joining decode on early failure.
    struct Run {
        output: WindowsOutput,
        worker: DecoderWorker,
        input: Input,
        audio: Transport,
        last: Option<Snapshot>,
        start: Instant,
        last_record: Instant,
        samples: Vec<Value>,
        checkpoints: Vec<Value>,
        latencies: Vec<Value>,
    }
    impl Run {
        fn poll(&mut self) -> Result<(), String> {
            let s = self.output.0.snapshot();
            if let Some(o) = WindowsOutput::observation(s) {
                self.audio.observe(o);
            }
            if self.audio.state().phase == Phase::Faulted {
                return Err(format!("Native fault: {s:?}"));
            }
            if self.last_record.elapsed() >= Duration::from_millis(50)
                && self
                    .last
                    .is_none_or(|last| last.sequence != s.sequence || last.epoch != s.epoch)
            {
                if self.samples.len() >= 4096 {
                    return Err("Observation limit".into());
                }
                self.samples.push(json!({"wall_seconds":self.start.elapsed().as_secs_f64(),"epoch":s.epoch,"sequence":s.sequence,
                    "phase":format!("{:?}",s.phase),"device_us":s.position_us,"simulation_us":self.audio.state().simulation_us,"stats":stats(self.input.stats())}));
                self.last_record = Instant::now();
            }
            self.last = Some(s);
            Ok(())
        }
        fn wait(
            &mut self,
            seconds: u64,
            mut condition: impl FnMut(&Self) -> bool,
        ) -> Result<(), String> {
            let end = Instant::now() + Duration::from_secs(seconds);
            loop {
                self.poll()?;
                if condition(self) {
                    return Ok(());
                }
                if Instant::now() > end {
                    return Err(format!("Timeout: {:?}", self.last));
                }
                thread::sleep(Duration::from_millis(10));
            }
        }
        fn control(&mut self, paused: bool) -> Result<(), String> {
            let start = Instant::now();
            let intent = self.audio.intent(paused);
            self.output.control(intent).map_err(|e| format!("{e:?}"))?;
            self.wait(5, |s| {
                s.audio.state().phase
                    == if paused {
                        Phase::Paused
                    } else {
                        Phase::Playing
                    }
            })?;
            self.latencies.push(json!({"kind":if paused{"pause_ack"}else{"play_ready_ack"},"ms":start.elapsed().as_secs_f64()*1000.,"content_us":self.audio.state().simulation_us}));
            Ok(())
        }
        fn freeze(&mut self, name: &str) -> Result<(), String> {
            let first = self.last.ok_or("No pause observation")?;
            let t = Instant::now();
            let position = self.audio.state().simulation_us;
            while t.elapsed() < Duration::from_millis(650) {
                self.poll()?;
                if self.audio.state().simulation_us != position
                    || self.input.stats().pcm.packets != 0
                {
                    return Err("Pause changed content or submitted PCM".into());
                }
                thread::sleep(Duration::from_millis(10));
            }
            let last = self.last.unwrap();
            if last.sequence <= first.sequence || last.position_us != first.position_us {
                return Err("Pause observation stale or advanced".into());
            }
            self.checkpoints.push(json!({"kind":"pause","name":name,"epoch":first.epoch,"first_sequence":first.sequence,"last_sequence":last.sequence,
                "device_us":last.position_us,"position_us":position,"wall_ms":t.elapsed().as_secs_f64()*1000.,"stats":stats(self.input.stats())}));
            Ok(())
        }
        fn seek(&mut self, position_us: u64) -> Result<(), String> {
            let start = Instant::now();
            let request = self.audio.seek(position_us).ok_or("Seek requires pause")?;
            self.output
                .relocate(request)
                .map_err(|e| format!("{e:?}"))?;
            self.wait(5, |s| s.audio.state().phase == Phase::Paused)?;
            if self.audio.state().simulation_us != position_us
                || self.input.stats().pcm.packets != 0
            {
                return Err("Seek moved before play".into());
            }
            self.latencies.push(json!({"kind":"seek_ack","ms":start.elapsed().as_secs_f64()*1000.,"content_us":position_us}));
            self.wait(5, |s| s.input.stats().queued_frames >= 4800)?;
            self.latencies.push(json!({"kind":"seek_prefill","ms":start.elapsed().as_secs_f64()*1000.,"content_us":position_us}));
            self.checkpoints.push(json!({"kind":"seek","epoch":request.epoch,"position_us":position_us,"generation":self.input.stats().generation}));
            Ok(())
        }
    }
    fn execute(source: PathBuf) -> Result<Value, String> {
        let (input, receiver) = buffered::channel(Config {
            rate: 48000,
            channels: 2,
            total_frames: 48000 * 180,
            gain: 0.002,
        })
        .map_err(|e| format!("{e:?}"))?;
        let worker = DecoderWorker::spawn(input.clone(), DiagnosticGate::default(), move || {
            File::open(&source)
        })
        .map_err(|e| e.to_string())?;
        let output =
            WindowsOutput::spawn_buffered(receiver, || {}).map_err(|e| format!("{e:?}"))?;
        let mut r = Run {
            output,
            worker,
            input: input.clone(),
            audio: Transport::default(),
            last: None,
            start: Instant::now(),
            last_record: Instant::now(),
            samples: Vec::with_capacity(1000),
            checkpoints: vec![],
            latencies: vec![],
        };
        r.wait(5, |s| s.audio.state().phase == Phase::Paused)?;
        r.audio.tool(true);
        r.control(false)?;
        if !r.audio.key(0, true, false, false) {
            return Err("Playing input rejected".into());
        }
        r.wait(36, |s| s.audio.state().simulation_us >= 30_000_000)?;
        r.checkpoints.push(json!({"kind":"continuous","position_us":r.audio.state().simulation_us,"wall_seconds":r.start.elapsed().as_secs_f64(),"stats":stats(input.stats())}));
        r.control(true)?;
        r.freeze("after_continuous")?;
        if r.audio.key(1, true, false, false) {
            return Err("Paused input accepted".into());
        }
        for (name, target) in [("forward", 90_000_000), ("backward", 500_000)] {
            r.seek(target)?;
            r.freeze(name)?;
            r.control(false)?;
            if r.audio.key(1, true, true, false) {
                return Err("Held input replayed".into());
            }
            r.wait(5, |s| s.audio.state().simulation_us >= target + 1_000_000)?;
            r.control(true)?;
        }
        r.freeze("final")?;
        let format = r.last.and_then(|s| s.format).ok_or("No audio format")?;
        drop(r.output);
        drop(r.worker);
        let final_stats = input.stats();
        if final_stats.free_blocks != 5
            || final_stats.queued_blocks != 0
            || final_stats.in_flight
            || final_stats.active
            || final_stats.underruns != 0
            || final_stats.decode_failures != 0
            || final_stats.trace_overflow
        {
            return Err(format!("Pool/fault: {final_stats:?}"));
        }
        let events:Vec<_>=input.trace().into_iter().map(|e|json!({"kind":format!("{:?}",e.kind),"generation":e.generation,"first_frame":e.first_frame,
            "frames":e.frames,"native_discarded_frames":e.native_discarded_frames,"packet":e.packet.map(|p|json!({"source_frame":p.source_frame,"frames":p.frames,"hash":p.hash,"peak":p.peak,"nonzero_samples":p.nonzero_samples,"probe":p.probe}))})).collect();
        Ok(
            json!({"schema_version":1,"status":"passed","config":{"rate":48000,"channels":2,"frames":8640000,"gain":0.002},
            "audio_format":{"rate":format.rate,"channels":format.channels,"bits":format.bits,"buffer_frames":format.buffer_frames},
            "events":events,"observations":r.samples,"checkpoints":r.checkpoints,"latencies":r.latencies,"final_stats":stats(final_stats),
            "accepted_inputs":1,"system_capture_used":false,"microphone_or_loopback_used":false,"physical_or_acoustic_latency_measured":false}),
        )
    }
    pub fn run() -> Result<(), String> {
        let mut args = std::env::args().skip(1);
        let out = PathBuf::from(args.next().ok_or("new output")?);
        let source = PathBuf::from(args.next().ok_or("owned input")?);
        if args.next().is_some() {
            return Err("Too many arguments".into());
        }
        let allowed = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("artifacts/p0")
            .canonicalize()
            .map_err(|e| e.to_string())?;
        if out.exists()
            || out
                .parent()
                .ok_or("No parent")?
                .canonicalize()
                .map_err(|e| e.to_string())?
                != allowed
            || !source
                .canonicalize()
                .map_err(|e| e.to_string())?
                .starts_with(&allowed)
        {
            return Err("Output/input boundary".into());
        }
        fs::create_dir(&out).map_err(|e| e.to_string())?;
        fs::write(out.join("ready"), b"1").map_err(|e| e.to_string())?;
        let start = Instant::now();
        while !out.join("go").exists() {
            if start.elapsed() > Duration::from_secs(15) {
                return Err("Handshake timeout".into());
            }
            thread::sleep(Duration::from_millis(5));
        }
        let result = execute(source);
        let value = match &result {
            Ok(v) => v.clone(),
            Err(e) => json!({"schema_version":1,"status":"failed","error":e}),
        };
        fs::write(
            out.join("result.json"),
            serde_json::to_vec_pretty(&value).unwrap(),
        )
        .map_err(|e| e.to_string())?;
        result.map(|_| ())
    }
}
fn main() {
    #[cfg(windows)]
    if let Err(e) = native::run() {
        eprintln!("compressed output: {e}");
        std::process::exit(1);
    }
    #[cfg(not(windows))]
    {
        eprintln!("Windows native proof only");
        std::process::exit(1);
    }
}
