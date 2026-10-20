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
        io::{BufWriter, Write},
        path::PathBuf,
        thread,
        time::{Duration, Instant},
    };

    fn stats(s: buffered::Stats) -> Value {
        json!({"generation":s.generation,"active":s.active,"free_blocks":s.free_blocks,
            "queued_blocks":s.queued_blocks,"queued_frames":s.queued_frames,"in_flight":s.in_flight,
            "stale_blocks":s.stale_blocks,"discarded_frames":s.discarded_frames,
            "native_discarded_frames":s.native_discarded_frames,"underruns":s.underruns,
            "decode_failures":s.decode_failures,"trace_overflow":s.trace_overflow,
            "origin_frame":s.pcm.source_origin_frame,"packets":s.pcm.packets})
    }
    struct Run {
        // Output is retired before the decoder is joined, also on an error return.
        output: WindowsOutput,
        worker: DecoderWorker,
        input: Input,
        gate: DiagnosticGate,
        audio: Transport,
        out: PathBuf,
        started: Instant,
        last_record: Instant,
        last: Snapshot,
        allow_fault: bool,
        samples: Vec<Value>,
        checks: Vec<Value>,
        latencies: Vec<Value>,
        trace: BufWriter<File>,
        drained: Vec<buffered::Event>,
        last_drain: Instant,
    }
    impl Run {
        fn stage(&self, name: &str) -> Result<(), String> {
            fs::write(
                self.out.join("phase.json"),
                json!({"stage":name}).to_string(),
            )
            .map_err(|e| e.to_string())
        }
        fn drain(&mut self) -> Result<(), String> {
            self.input.drain_trace(&mut self.drained);
            for e in &self.drained {
                let row = json!({"kind":format!("{:?}",e.kind),"generation":e.generation,
                    "first_frame":e.first_frame,"frames":e.frames,"native_discarded_frames":e.native_discarded_frames,
                    "packet":e.packet.map(|p|json!({"source_frame":p.source_frame,"frames":p.frames,
                        "hash":p.hash,"peak":p.peak,"nonzero_samples":p.nonzero_samples,"probe":p.probe}))});
                writeln!(self.trace, "{row}").map_err(|e| e.to_string())?;
            }
            self.last_drain = Instant::now();
            Ok(())
        }
        fn poll(&mut self) -> Result<(), String> {
            self.last = self.output.0.snapshot();
            if let Some(o) = WindowsOutput::observation(self.last) {
                self.audio.observe(o);
            }
            if !self.allow_fault && self.audio.state().phase == Phase::Faulted {
                return Err(format!("Unexpected native fault: {:?}", self.last));
            }
            if self.last_record.elapsed() >= Duration::from_millis(50) {
                if self.samples.len() >= 16000 {
                    return Err("Observation limit".into());
                }
                let f = self.last.format;
                self.samples.push(json!({"wall_seconds":self.started.elapsed().as_secs_f64(),
                    "epoch":self.last.epoch,"sequence":self.last.sequence,"phase":format!("{:?}",self.last.phase),
                    "device_us":self.last.position_us,"simulation_us":self.audio.state().simulation_us,
                    "format":f.map(|f|json!({"rate":f.rate,"channels":f.channels,"mix_rate":f.mix_rate,
                        "mix_channels":f.mix_channels,"conversion_enabled":f.conversion_enabled,
                        "buffer_frames":f.buffer_frames,"clock_frequency":f.clock_frequency})),"stats":stats(self.input.stats())}));
                self.last_record = Instant::now();
            }
            if self.last_drain.elapsed() >= Duration::from_millis(500) {
                self.drain()?;
            }
            Ok(())
        }
        fn wait(
            &mut self,
            seconds: u64,
            mut condition: impl FnMut(&Self) -> bool,
        ) -> Result<(), String> {
            let until = Instant::now() + Duration::from_secs(seconds);
            loop {
                self.poll()?;
                if condition(self) {
                    return Ok(());
                }
                if Instant::now() > until {
                    return Err(format!("Timeout: {:?}", self.last));
                }
                thread::sleep(Duration::from_millis(10));
            }
        }
        fn checkpoint(&mut self, name: &str) {
            self.checks.push(json!({"name":name,"epoch":self.last.epoch,"simulation_us":self.audio.state().simulation_us,"stats":stats(self.input.stats())}));
        }
        fn control(&mut self, paused: bool) -> Result<(), String> {
            let start = Instant::now();
            let i = self.audio.intent(paused);
            self.output.control(i).map_err(|e| format!("{e:?}"))?;
            self.wait(5, |r| {
                r.audio.state().phase
                    == if paused {
                        Phase::Paused
                    } else {
                        Phase::Playing
                    }
            })?;
            self.latencies.push(json!({"kind":if paused{"pause"}else{"play_ready"},"ms":start.elapsed().as_secs_f64()*1000.}));
            Ok(())
        }
        fn freeze(&mut self, name: &str, millis: u64) -> Result<(), String> {
            let expected = self.audio.state();
            let first = self.last.sequence;
            let start = Instant::now();
            while start.elapsed() < Duration::from_millis(millis) {
                self.poll()?;
                if self.audio.state().simulation_us != expected.simulation_us
                    || self.audio.key(1, true, false, false)
                {
                    return Err("Frozen clock/input changed".into());
                }
                if expected.phase == Phase::Paused && self.input.stats().pcm.packets != 0 {
                    return Err("Paused native submission".into());
                }
                thread::sleep(Duration::from_millis(10));
            }
            if expected.phase == Phase::Paused && self.last.sequence <= first {
                return Err("Stale pause observations".into());
            }
            self.checkpoint(name);
            Ok(())
        }
        fn play_for(&mut self, micros: u64) -> Result<(), String> {
            self.control(false)?;
            let first = self.audio.state().simulation_us;
            self.wait(micros / 1_000_000 + 6, |r| {
                r.audio.state().simulation_us >= first + micros
            })
        }
        fn recover(&mut self) -> Result<(), String> {
            let before = self.audio.state().simulation_us;
            let old = self.last;
            let target = self.audio.recover().ok_or("Recovery without fault")?;
            self.output.relocate(target).map_err(|e| format!("{e:?}"))?;
            // Early play is constrained to pause until a new, paused anchor arrives.
            let early = self.audio.intent(false);
            if !early.paused {
                return Err("Early recovery play".into());
            }
            self.output.control(early).map_err(|e| format!("{e:?}"))?;
            self.wait(6, |r| r.audio.state().phase == Phase::Paused)?;
            self.allow_fault = false;
            if self.audio.state().simulation_us != before {
                return Err("Recovery position changed".into());
            }
            if self
                .audio
                .observe(WindowsOutput::observation(old).ok_or("Old observation")?)
            {
                return Err("Old epoch accepted".into());
            }
            self.output
                .0
                .request_in(old.epoch, 999, false)
                .map_err(|e| format!("{e:?}"))?;
            self.freeze("recovered_paused", 450)?;
            Ok(())
        }
    }
    fn execute(
        out: PathBuf,
        source: PathBuf,
        convert: bool,
        idle: u64,
        manual: bool,
    ) -> Result<Value, String> {
        if source.metadata().map_err(|e| e.to_string())?.len() > 128 * 1024 * 1024 {
            return Err("Source byte limit".into());
        }
        let stream =
            p0_media::compressed::Stream::open(File::open(&source).map_err(|e| e.to_string())?)
                .map_err(|e| format!("{e:?}"))?;
        let rate = stream.rate();
        let channels = stream.channels();
        let total_frames = stream.total_frames();
        drop(stream);
        let (input, receiver) = buffered::channel(Config {
            rate,
            channels,
            total_frames,
            gain: 0.002,
        })
        .map_err(|e| format!("{e:?}"))?;
        let gate = DiagnosticGate::default();
        let worker = DecoderWorker::spawn(input.clone(), gate.clone(), move || File::open(&source))
            .map_err(|e| e.to_string())?;
        let output = if convert {
            WindowsOutput::spawn_buffered_converting(receiver, || {})
        } else {
            WindowsOutput::spawn_buffered(receiver, || {})
        }
        .map_err(|e| format!("{e:?}"))?;
        let mut r = Run {
            output,
            worker,
            input: input.clone(),
            gate,
            audio: Transport::default(),
            out: out.clone(),
            started: Instant::now(),
            last_record: Instant::now(),
            last: Snapshot::default(),
            allow_fault: false,
            samples: Vec::new(),
            checks: Vec::new(),
            latencies: Vec::new(),
            trace: BufWriter::new(
                File::create(out.join("packets.jsonl")).map_err(|e| e.to_string())?,
            ),
            drained: Vec::with_capacity(buffered::TRACE_LIMIT),
            last_drain: Instant::now(),
        };
        r.wait(6, |r| r.audio.state().phase == Phase::Paused)?;
        r.audio.tool(true);
        r.stage("Playing")?;
        r.play_for(8_000_000)?;
        if !r.audio.key(0, true, false, false) {
            return Err("Fresh playing input rejected".into());
        }
        r.checkpoint("continuous");
        r.stage("Transitions")?;
        // Hold an old decoded block across pause, then verify its late completion
        // is rejected. This is a diagnostic barrier, not real storage starvation.
        r.gate.hold();
        r.wait(2, |r| r.gate.held().is_some())?;
        let stale = input.stats().stale_blocks;
        r.control(true)?;
        r.freeze("pause_with_old_inflight", 350)?;
        r.gate.release();
        r.wait(4, |r| {
            r.input.stats().stale_blocks > stale && r.input.stats().queued_blocks == 4
        })?;
        r.checkpoint("old_block_rejected");
        r.play_for(500_000)?;
        r.control(true)?;
        let target = r.audio.seek(1_234_567).ok_or("Seek not paused")?;
        r.output.relocate(target).map_err(|e| format!("{e:?}"))?;
        r.wait(5, |r| r.audio.state().phase == Phase::Paused)?;
        if r.audio.state().simulation_us != target.position_us {
            return Err("Seek anchor mismatch".into());
        }
        r.freeze("seek_paused", 350)?;
        r.play_for(500_000)?;
        r.allow_fault = true;
        let mut device_change_observed = false;
        if manual {
            r.stage("await_unplug_playing")?;
            r.wait(120, |r| {
                r.audio.state().phase == Phase::Faulted || r.out.join("no_change").exists()
            })?;
            device_change_observed = r.audio.state().phase == Phase::Faulted;
            if !device_change_observed {
                r.output
                    .0
                    .invalidate(r.audio.state().epoch)
                    .map_err(|e| format!("{e:?}"))?;
                r.wait(5, |r| r.audio.state().phase == Phase::Faulted)?;
            }
        } else {
            r.output
                .0
                .invalidate(r.audio.state().epoch)
                .map_err(|e| format!("{e:?}"))?;
            r.wait(5, |r| r.audio.state().phase == Phase::Faulted)?;
        }
        let fault = r
            .last
            .error
            .map(|e| json!({"operation":e.operation,"code":e.code}));
        r.freeze("fault_frozen", 450)?;
        if manual {
            r.stage("await_replug_recover")?;
            r.wait(180, |r| r.out.join("recover").exists())?;
        }
        r.recover()?;
        if manual {
            r.stage("recovered_wait_explicit_play")?;
            r.wait(180, |r| r.out.join("play").exists())?;
        }
        if r.audio.key(0, true, true, false) || r.audio.key(1, true, true, false) {
            return Err("Held key replay".into());
        }
        r.play_for(1_000_000)?;
        if r.audio.key(0, true, true, false) {
            return Err("Held key accepted after resume".into());
        }
        r.audio.key(0, false, false, false);
        if !r.audio.key(0, true, false, false) {
            return Err("Fresh resumed input rejected".into());
        }
        r.control(true)?;
        r.stage("PausedIdle")?;
        r.freeze("final_pause", idle * 1000)?;
        let accepted = r.audio.state().accepted_inputs;
        let checks = r.checks.clone();
        let observations = r.samples.clone();
        let latencies = r.latencies.clone();
        drop(r.output);
        drop(r.worker);
        input.drain_trace(&mut r.drained);
        for e in &r.drained {
            let row = json!({"kind":format!("{:?}",e.kind),"generation":e.generation,"first_frame":e.first_frame,
                "frames":e.frames,"native_discarded_frames":e.native_discarded_frames,"packet":e.packet.map(|p|json!({"source_frame":p.source_frame,
                    "frames":p.frames,"hash":p.hash,"peak":p.peak,"nonzero_samples":p.nonzero_samples,"probe":p.probe}))});
            writeln!(r.trace, "{row}").map_err(|e| e.to_string())?;
        }
        r.trace.flush().map_err(|e| e.to_string())?;
        let final_stats = input.stats();
        if final_stats.active
            || final_stats.in_flight
            || final_stats.queued_blocks != 0
            || final_stats.free_blocks != 5
            || final_stats.underruns != 0
            || final_stats.decode_failures != 0
            || final_stats.trace_overflow
            || accepted != 2
        {
            return Err(format!("Final pool/stats: {final_stats:?}"));
        }
        fs::write(
            out.join("phase.json"),
            json!({"stage":"ClosedIdle"}).to_string(),
        )
        .map_err(|e| e.to_string())?;
        thread::sleep(Duration::from_secs(idle));
        Ok(
            json!({"schema_version":1,"status":"passed","source_format":{"rate":rate,"channels":channels,"frames":total_frames,"gain":0.002},
            "conversion_requested":convert,"manual_device_change":manual,"device_change_observed":device_change_observed,"fault":fault,"accepted_inputs":accepted,
            "checkpoints":checks,"observations":observations,"latencies":latencies,"final_stats":stats(final_stats),
            "system_capture_used":false,"recording_used":false,"engine_output_samples_verified":false}),
        )
    }
    pub fn run() -> Result<(), String> {
        let args: Vec<_> = std::env::args().skip(1).collect();
        if args.len() != 5 {
            return Err(
                "new output, owned source, strict/convert, idle seconds, auto/manual".into(),
            );
        }
        let out = PathBuf::from(&args[0]);
        let source = PathBuf::from(&args[1])
            .canonicalize()
            .map_err(|e| e.to_string())?;
        let convert = match args[2].as_str() {
            "strict" => false,
            "convert" => true,
            _ => return Err("Output mode".into()),
        };
        let idle: u64 = args[3].parse().map_err(|_| "Idle seconds")?;
        if ![3, 65].contains(&idle) {
            return Err("Idle range".into());
        }
        let manual = match args[4].as_str() {
            "auto" => false,
            "manual" => true,
            _ => return Err("Device mode".into()),
        };
        if manual && !convert {
            return Err("Manual requires monitored conversion mode".into());
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
            || !source.starts_with(&allowed)
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
        let result = execute(out.clone(), source, convert, idle, manual);
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
        eprintln!("Format/device proof: {e}");
        std::process::exit(1);
    }
    #[cfg(not(windows))]
    {
        eprintln!("Windows proof only");
        std::process::exit(1);
    }
}
