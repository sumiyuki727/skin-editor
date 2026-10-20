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
        buffered::{self, Config, Input, Kind},
    };
    use serde_json::{Value, json};
    use std::{
        fs::{self, File},
        path::PathBuf,
        thread,
        time::{Duration, Instant},
    };

    fn packet(p: p0_wasapi::pcm::Packet) -> Value {
        json!({"source_frame":p.source_frame,"frames":p.frames,"hash":p.hash,"peak":p.peak,"nonzero_samples":p.nonzero_samples,"probe":p.probe})
    }
    fn stats(s: buffered::Stats) -> Value {
        json!({"generation":s.generation,"active":s.active,"queued_blocks":s.queued_blocks,"queued_frames":s.queued_frames,
        "in_flight":s.in_flight,"free_blocks":s.free_blocks,"max_queued_blocks":s.max_queued_blocks,"accepted_blocks":s.accepted_blocks,
        "stale_blocks":s.stale_blocks,"discarded_blocks":s.discarded_blocks,"discarded_frames":s.discarded_frames,"native_discarded_frames":s.native_discarded_frames,
        "underruns":s.underruns,"decode_failures":s.decode_failures,"trace_overflow":s.trace_overflow,
        "pcm":{"requested_us":s.pcm.requested_us,"source_origin_frame":s.pcm.source_origin_frame,"submitted_frames":s.pcm.submitted_frames,"packets":s.pcm.packets}})
    }
    struct Session {
        output: WindowsOutput,
        transport: Transport,
        input: Input,
        gate: DiagnosticGate,
        last: Option<Snapshot>,
        samples: Vec<Value>,
        checkpoints: Vec<Value>,
    }
    impl Session {
        fn poll(&mut self) -> Result<bool, String> {
            let sample = self.output.0.snapshot();
            if self.last.is_some_and(|s| s.sequence == sample.sequence) {
                return Ok(false);
            }
            self.last = Some(sample);
            if let Some(observation) = WindowsOutput::observation(sample) {
                self.transport.observe(observation);
            }
            if self.samples.len() == 4096 {
                return Err("Observation limit".into());
            }
            self.samples.push(json!({"epoch":sample.epoch,"sequence":sample.sequence,"phase":format!("{:?}",sample.phase),"request":sample.request,"device_us":sample.position_us,
                "simulation_us":self.transport.state().simulation_us,"transport_phase":format!("{:?}",self.transport.state().phase),"buffered":sample.buffered.map(stats)}));
            Ok(true)
        }
        fn wait(
            &mut self,
            seconds: u64,
            expected_fault: bool,
            mut predicate: impl FnMut(&Self) -> bool,
        ) -> Result<(), String> {
            let deadline = Instant::now() + Duration::from_secs(seconds);
            loop {
                self.poll()?;
                if self.transport.state().phase == Phase::Faulted && !expected_fault {
                    return Err(format!("Unexpected audio fault: {:?}", self.last));
                }
                if predicate(self) {
                    return Ok(());
                }
                if Instant::now() >= deadline {
                    return Err(format!("Timed out: {:?}", self.last));
                }
                thread::sleep(Duration::from_millis(10));
            }
        }
        fn control(&mut self, paused: bool) -> Result<(), String> {
            let intent = self.transport.intent(paused);
            self.output.control(intent).map_err(|e| format!("{e:?}"))?;
            self.wait(4, false, |s| {
                s.transport.state().phase
                    == if paused {
                        Phase::Paused
                    } else {
                        Phase::Playing
                    }
            })
        }
        fn freeze(&mut self, name: &str) -> Result<(), String> {
            let sample = self.last.ok_or("Missing pause sample")?;
            let before = self.transport.state().simulation_us;
            let native = sample.position_us;
            let generation = self.input.stats().generation;
            let start = Instant::now();
            let mut fresh = 0;
            while start.elapsed() < Duration::from_millis(350) {
                if self.poll()? {
                    fresh += 1;
                }
                if self.transport.state().phase != Phase::Paused
                    || self.transport.state().simulation_us != before
                    || self.last.unwrap().position_us != native
                    || self.input.stats().pcm.submitted_frames != 0
                    || self.input.stats().generation != generation
                {
                    return Err("Paused stream submitted data or moved time/generation".into());
                }
                thread::sleep(Duration::from_millis(10));
            }
            if fresh < 3 {
                return Err("Missing fresh pause observations".into());
            }
            self.checkpoints.push(json!({"kind":"pause","name":name,"epoch":sample.epoch,"position_us":before,"device_us":native,"generation":generation,"fresh_samples":fresh,
                "wall_ms":start.elapsed().as_secs_f64()*1000.,"stats":stats(self.input.stats())}));
            Ok(())
        }
        fn seek(&mut self, position_us: u64) -> Result<(), String> {
            let target = self
                .transport
                .seek(position_us)
                .ok_or("Seek requires pause")?;
            self.output.relocate(target).map_err(|e| format!("{e:?}"))?;
            self.wait(4, false, |s| s.transport.state().phase == Phase::Paused)?;
            if self.transport.state().simulation_us != position_us
                || self.input.stats().pcm.submitted_frames != 0
            {
                return Err("Seek advanced before explicit play".into());
            }
            self.checkpoints.push(json!({"kind":"seek","epoch":target.epoch,"position_us":position_us,"generation":self.input.stats().generation}));
            Ok(())
        }
    }
    fn execute(path: PathBuf) -> Result<Value, String> {
        let (input, receiver) = buffered::channel(Config {
            rate: 48000,
            channels: 2,
            total_frames: 48000 * 30,
            gain: 0.02,
        })
        .map_err(|e| format!("{e:?}"))?;
        let gate = DiagnosticGate::default();
        let worker = DecoderWorker::spawn(input.clone(), gate.clone(), move || File::open(&path))
            .map_err(|e| e.to_string())?;
        let output =
            WindowsOutput::spawn_buffered(receiver, || {}).map_err(|e| format!("{e:?}"))?;
        let mut s = Session {
            output,
            transport: Transport::default(),
            input: input.clone(),
            gate,
            last: None,
            samples: vec![],
            checkpoints: vec![],
        };
        s.wait(4, false, |s| s.transport.state().phase == Phase::Paused)?;
        s.transport.tool(true);
        s.control(false)?;
        if !s.transport.key(0, true, false, false) {
            return Err("Playing input rejected".into());
        }
        eprintln!("continuous 10-second device playback");
        s.wait(15, false, |s| {
            s.transport.state().simulation_us >= 10_000_000
        })?;
        if input.stats().underruns != 0 {
            return Err("Continuous playback underrun".into());
        }
        s.checkpoints.push(json!({"kind":"continuous","position_us":s.transport.state().simulation_us,"stats":stats(input.stats())}));
        eprintln!("pause while one decoded completion is deliberately held");
        s.gate.hold();
        s.wait(2, false, |s| s.gate.held().is_some())?;
        let held_pause = s.gate.held().unwrap();
        let old_stale = input.stats().stale_blocks;
        s.control(true)?;
        s.gate.release();
        s.wait(3, false, |_| {
            input.stats().stale_blocks > old_stale && input.stats().queued_blocks >= 2
        })?;
        s.freeze("discard_inflight_on_pause")?;
        if s.transport.key(1, true, false, false) {
            return Err("Paused input leaked".into());
        }
        let resumed_from = s.transport.state().simulation_us;
        s.control(false)?;
        s.wait(3, false, |s| {
            s.transport.state().simulation_us >= resumed_from + 500000
        })?;
        s.control(true)?;
        s.seek(20_000_000)?;
        s.control(false)?;
        s.wait(3, false, |s| {
            s.transport.state().simulation_us >= 20_500_000
        })?;
        s.control(true)?;
        eprintln!("rapid forward/back seek rejects held old completion");
        s.gate.hold();
        s.seek(25_000_000)?;
        s.wait(2, false, |s| s.gate.held().is_some())?;
        let held_seek = s.gate.held().unwrap();
        let old_stale = input.stats().stale_blocks;
        s.seek(500000)?;
        s.gate.release();
        s.wait(3, false, |_| {
            input.stats().stale_blocks > old_stale && input.stats().queued_blocks >= 2
        })?;
        s.freeze("discard_inflight_on_seek")?;
        s.control(false)?;
        if s.transport.key(1, true, true, false) {
            return Err("Held input replayed after seek".into());
        }
        s.transport.key(1, false, false, false);
        if !s.transport.key(1, true, false, false) {
            return Err("Fresh post-seek input rejected".into());
        }
        s.wait(3, false, |s| s.transport.state().simulation_us >= 1_000_000)?;
        eprintln!("controlled decoder starvation, then explicit recovery");
        s.gate.hold();
        s.wait(2, false, |s| s.gate.held().is_some())?;
        let held_underrun = s.gate.held().unwrap();
        s.wait(3, true, |s| s.transport.state().phase == Phase::Faulted)?;
        if s.last
            .unwrap()
            .error
            .is_none_or(|e| e.operation != "Streamed PCM queue underrun")
            || input.stats().underruns != 1
        {
            return Err("Starvation did not report a single explicit underrun".into());
        }
        let frozen = s.transport.state().simulation_us;
        let fault_start = Instant::now();
        while fault_start.elapsed() < Duration::from_millis(350) {
            s.poll()?;
            if s.transport.state().simulation_us != frozen || s.transport.key(2, true, false, false)
            {
                return Err("Fault allowed time or game input".into());
            }
            thread::sleep(Duration::from_millis(10));
        }
        s.checkpoints.push(json!({"kind":"underrun","position_us":frozen,"wall_ms":fault_start.elapsed().as_secs_f64()*1000.,"stats":stats(input.stats())}));
        let old_stale = input.stats().stale_blocks;
        s.gate.release();
        s.wait(3, true, |_| input.stats().stale_blocks > old_stale)?;
        let recovery = s.transport.recover().ok_or("Recovery not available")?;
        s.output.relocate(recovery).map_err(|e| format!("{e:?}"))?;
        s.wait(4, false, |s| s.transport.state().phase == Phase::Paused)?;
        s.freeze("recovery_requires_explicit_play")?;
        if s.transport.state().simulation_us != frozen {
            return Err("Recovery changed confirmed content time".into());
        }
        s.control(false)?;
        if s.transport.key(0, true, true, false) {
            return Err("Held input replayed after recovery".into());
        }
        s.transport.key(0, false, false, false);
        if !s.transport.key(0, true, false, false) {
            return Err("Fresh post-recovery input rejected".into());
        }
        s.wait(3, false, |s| {
            s.transport.state().simulation_us >= frozen + 500000
        })?;
        s.control(true)?;
        s.freeze("final")?;
        let accepted_inputs = s.transport.state().accepted_inputs;
        let final_time = s.transport.state().simulation_us;
        let format = s.last.unwrap().format.unwrap();
        let samples = std::mem::take(&mut s.samples);
        let checkpoints = std::mem::take(&mut s.checkpoints);
        drop(s);
        drop(worker);
        let final_stats = input.stats();
        if final_stats.free_blocks != buffered::POOL_BLOCKS
            || final_stats.in_flight
            || final_stats.underruns != 1
            || final_stats.decode_failures != 0
            || final_stats.trace_overflow
        {
            return Err(
                "Pipeline shutdown, failure counts or bounded trace invariant failed".into(),
            );
        }
        let events: Vec<_> = input.trace().into_iter().map(|e| json!({"kind":match e.kind {Kind::Activate=>"activate",Kind::Pause=>"pause",Kind::Retire=>"retire",Kind::Submit=>"submit",Kind::Stale=>"stale",Kind::Underrun=>"underrun"},
            "generation":e.generation,"first_frame":e.first_frame,"frames":e.frames,"native_discarded_frames":e.native_discarded_frames,"packet":e.packet.map(packet)})).collect();
        Ok(
            json!({"schema_version":1,"status":"passed","source":"self_authored_30_second_wav_stream","audio_format":{"rate":format.rate,"channels":format.channels,"bits":format.bits,"buffer_frames":format.buffer_frames},
            "config":{"rate":48000,"channels":2,"frames":1440000,"gain":0.02,"block_frames":buffered::BLOCK_FRAMES,"queue_blocks":buffered::QUEUE_BLOCKS,"pool_blocks":buffered::POOL_BLOCKS,"trace_limit":buffered::TRACE_LIMIT},
            "samples":samples,"checkpoints":checkpoints,"events":events,"held_pause":held_pause,"held_seek":held_seek,"held_underrun":held_underrun,"accepted_inputs":accepted_inputs,
            "final_simulation_us":final_time,"final_stats":stats(final_stats),"system_capture_used":false,"microphone_or_loopback_used":false,"physical_input_or_acoustic_latency_measured":false,
            "checks":{"continuous_device_playback":true,"pause_flushes_native_and_queued_pcm":true,"stale_inflight_on_pause_and_seek_rejected":true,"controlled_underrun_faults_and_freezes_input":true,"recovery_requires_explicit_resume":true,"pool_recycled_on_shutdown":true}}),
        )
    }
    pub fn run() -> Result<(), String> {
        let args: Vec<_> = std::env::args().skip(1).collect();
        if args.len() != 2 {
            return Err(
                "Expected new artifacts/p0 output and owned stream-output-fixture directory".into(),
            );
        }
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("artifacts/p0")
            .canonicalize()
            .map_err(|e| e.to_string())?;
        let out = PathBuf::from(&args[0]);
        if out.exists()
            || out
                .parent()
                .ok_or("Missing output parent")?
                .canonicalize()
                .map_err(|e| e.to_string())?
                != root
        {
            return Err("Output must be a new direct artifacts/p0 child".into());
        }
        let fixture = PathBuf::from(&args[1])
            .canonicalize()
            .map_err(|e| e.to_string())?;
        if fixture.parent() != Some(root.as_path())
            || !fixture
                .file_name()
                .and_then(|s| s.to_str())
                .is_some_and(|s| s.starts_with("stream-output-fixture-"))
        {
            return Err("Not an owned stream output fixture".into());
        }
        let audio = fixture
            .join("stream.wav")
            .canonicalize()
            .map_err(|e| e.to_string())?;
        if audio.parent() != Some(fixture.as_path())
            || audio.metadata().map_err(|e| e.to_string())?.len() != 5760044
        {
            return Err("Escaped or unexpected fixture audio".into());
        }
        fs::create_dir(&out).map_err(|e| e.to_string())?;
        let result = execute(audio);
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
        eprintln!("stream output proof: {e}");
        std::process::exit(1);
    }
    #[cfg(not(windows))]
    {
        eprintln!("Native stream output proof is Windows-only");
        std::process::exit(1);
    }
}
