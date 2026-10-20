//! Opt-in, bounded same-process streaming + real PNG/GPU long observation.
#[cfg(windows)]
mod native {
    use eframe::{egui, wgpu};
    use p0_egui::{
        audio_output::Output,
        audio_transport::{Phase, Transport},
        composition_switch::DocumentSwitch,
        interaction::{Command, Outcome, Owner},
        streaming_decode::{DecoderWorker, DiagnosticGate},
        windows_audio::WindowsOutput,
    };
    use p0_wasapi::{Snapshot, buffered};
    use serde_json::{Value, json};
    use std::{
        fs::{self, File},
        io::{BufWriter, Write},
        path::PathBuf,
        sync::Arc,
        time::{Duration, Instant},
    };

    fn stats(s: buffered::Stats) -> Value {
        json!({"generation":s.generation,"active":s.active,"queued_blocks":s.queued_blocks,
            "queued_frames":s.queued_frames,"in_flight":s.in_flight,"free_blocks":s.free_blocks,
            "max_queued_blocks":s.max_queued_blocks,"accepted_blocks":s.accepted_blocks,
            "stale_blocks":s.stale_blocks,"discarded_frames":s.discarded_frames,
            "native_discarded_frames":s.native_discarded_frames,"underruns":s.underruns,
            "decode_failures":s.decode_failures,"trace_overflow":s.trace_overflow,
            "submitted_frames":s.pcm.submitted_frames,"packets":s.pcm.packets})
    }
    struct App {
        root: PathBuf,
        output: PathBuf,
        start: Instant,
        phase_start: Instant,
        phase: &'static str,
        seconds: u64,
        idle_seconds: u64,
        output_device: Option<WindowsOutput>,
        decoder: Option<DecoderWorker>,
        input: buffered::Input,
        audio: Transport,
        last: Option<Snapshot>,
        switch: DocumentSwitch,
        owners: Vec<Owner>,
        sessions: Vec<Option<u64>>,
        changes: u64,
        switches: usize,
        requested: Option<(usize, Instant)>,
        switch_ms: Vec<f64>,
        trace: BufWriter<File>,
        events: Vec<buffered::Event>,
        event_count: usize,
        submit_count: usize,
        last_drain: Instant,
        samples: Vec<Value>,
        phases: Vec<Value>,
        frame_ms: Vec<f64>,
        frame_intervals_ms: Vec<f64>,
        last_frame: Option<Instant>,
        previous_playing: bool,
        idle_frames: usize,
        closed_frames: usize,
        frozen_us: u64,
        continuous_us: u64,
        pixel_checks: Vec<Value>,
        errors: Vec<String>,
        finished: bool,
        adapter: String,
    }
    impl App {
        fn new(
            cc: &eframe::CreationContext<'_>,
            root: PathBuf,
            output: PathBuf,
            source: PathBuf,
            seconds: u64,
        ) -> Result<Self, String> {
            let (input, receiver) = buffered::channel(buffered::Config {
                rate: 48000,
                channels: 2,
                total_frames: 48000 * 600,
                gain: 0.002,
            })
            .map_err(|e| format!("{e:?}"))?;
            let decoder =
                DecoderWorker::spawn(input.clone(), DiagnosticGate::default(), move || {
                    File::open(&source)
                })
                .map_err(|e| e.to_string())?;
            let wake = cc.egui_ctx.clone();
            let device = WindowsOutput::spawn_buffered(receiver, move || wake.request_repaint())
                .map_err(|e| format!("{e:?}"))?;
            let state = cc.wgpu_render_state.clone().ok_or("wgpu required")?;
            let adapter = format!("{:?}", state.adapter.get_info());
            let mut switch = DocumentSwitch::new(state, &cc.egui_ctx);
            switch.warm_pipelines(&cc.egui_ctx);
            switch.request(root.clone(), 0)?;
            let mut app = Self {
                root,
                output: output.clone(),
                start: Instant::now(),
                phase_start: Instant::now(),
                phase: "Warmup",
                seconds,
                idle_seconds: if seconds == 3 { 2 } else { 65 },
                output_device: Some(device),
                decoder: Some(decoder),
                input,
                audio: Transport::default(),
                last: None,
                switch,
                owners: (0..10).map(|_| Owner::default()).collect(),
                sessions: vec![None; 10],
                changes: 0,
                switches: 0,
                requested: None,
                switch_ms: vec![],
                trace: BufWriter::new(
                    File::create(output.join("packets.jsonl")).map_err(|e| e.to_string())?,
                ),
                events: Vec::with_capacity(buffered::TRACE_LIMIT),
                event_count: 0,
                submit_count: 0,
                last_drain: Instant::now(),
                samples: Vec::with_capacity(700),
                phases: vec![],
                frame_ms: Vec::with_capacity(40000),
                frame_intervals_ms: Vec::with_capacity(40000),
                last_frame: None,
                previous_playing: false,
                idle_frames: 0,
                closed_frames: 0,
                frozen_us: 0,
                continuous_us: 0,
                pixel_checks: vec![],
                errors: vec![],
                finished: false,
                adapter,
            };
            app.mark("Warmup")?;
            Ok(app)
        }
        fn mark(&mut self, phase: &'static str) -> Result<(), String> {
            self.phase = phase;
            self.phase_start = Instant::now();
            let row = json!({"phase":phase,"elapsed_seconds":self.start.elapsed().as_secs_f64(),
                "simulation_us":self.audio.state().simulation_us,"changes":self.changes});
            self.phases.push(row.clone());
            fs::write(self.output.join("phase.json"), row.to_string()).map_err(|e| e.to_string())
        }
        fn control(&mut self, paused: bool) -> Result<(), String> {
            let intent = self.audio.intent(paused);
            self.output_device
                .as_mut()
                .ok_or("No output")?
                .control(intent)
                .map_err(|e| format!("{e:?}"))
        }
        fn drain(&mut self) -> Result<(), String> {
            self.input.drain_trace(&mut self.events);
            for e in &self.events {
                let p = e.packet.map(|p| json!({"source_frame":p.source_frame,"frames":p.frames,
                    "hash":p.hash,"peak":p.peak,"nonzero_samples":p.nonzero_samples,"probe":p.probe}));
                let row = json!({"kind":format!("{:?}",e.kind),"generation":e.generation,
                    "first_frame":e.first_frame,"frames":e.frames,"native_discarded_frames":e.native_discarded_frames,"packet":p});
                serde_json::to_writer(&mut self.trace, &row).map_err(|e| e.to_string())?;
                self.trace.write_all(b"\n").map_err(|e| e.to_string())?;
                self.submit_count += usize::from(e.kind == buffered::Kind::Submit);
            }
            self.event_count += self.events.len();
            self.events.clear();
            self.trace.flush().map_err(|e| e.to_string())?;
            if self.samples.len() >= 700 {
                return Err("Observation limit".into());
            }
            self.samples.push(json!({"elapsed_seconds":self.start.elapsed().as_secs_f64(),"phase":self.phase,
                "simulation_us":self.audio.state().simulation_us,"stats":stats(self.input.stats()),
                "device":self.last.map(|s|json!({"sequence":s.sequence,"phase":format!("{:?}",s.phase),"position_us":s.position_us}))}));
            self.last_drain = Instant::now();
            Ok(())
        }
        fn commit(&mut self, doc: usize) -> Result<(), String> {
            if let Some(session) = self.sessions[doc].take() {
                self.owners[doc]
                    .execute(Command::Commit { session })
                    .map_err(|e| format!("{e:?}"))?;
            }
            Ok(())
        }
        fn edit(&mut self, doc: usize) -> Result<(), String> {
            if self.sessions[doc].is_none() {
                let expected_node = self.owners[doc].committed().id;
                let Outcome::Started(session) = self.owners[doc]
                    .execute(Command::Begin { expected_node })
                    .map_err(|e| format!("{e:?}"))?
                else {
                    return Err("No command session".into());
                };
                self.sessions[doc] = Some(session);
            }
            let n = self.changes as f64;
            let position = [40. + 24. * (n * 0.04).sin(), 40. + 16. * (n * 0.03).cos()];
            self.owners[doc]
                .execute(Command::Move {
                    session: self.sessions[doc].unwrap(),
                    position,
                })
                .map_err(|e| format!("{e:?}"))?;
            self.changes += 1;
            Ok(())
        }
        fn poll(&mut self, ctx: &egui::Context) -> Result<(), String> {
            if let Some(output) = &self.output_device {
                let s = output.0.snapshot();
                if let Some(observation) = WindowsOutput::observation(s) {
                    self.audio.observe(observation);
                }
                self.last = Some(s);
                if self.audio.state().phase == Phase::Faulted {
                    return Err(format!("Unexpected audio fault: {s:?}"));
                }
            }
            self.switch.poll(ctx.cumulative_frame_nr(), ctx)?;
            if let Some(e) = &self.switch.error {
                return Err(e.clone());
            }
            if let Some((doc, requested)) = self.requested
                && self.switch.active().is_some_and(|(d, _)| d == doc)
                && !self.switch.pending()
            {
                self.switch_ms
                    .push(requested.elapsed().as_secs_f64() * 1000.);
                self.requested = None;
            }
            let s = self.input.stats();
            if s.underruns != 0 || s.decode_failures != 0 || s.trace_overflow {
                return Err(format!("Audio queue failed: {s:?}"));
            }
            if self.last_drain.elapsed() >= Duration::from_secs(1)
                && matches!(self.phase, "Warmup" | "Start" | "Playing" | "Pausing")
            {
                self.drain()?;
            }
            match self.phase {
                "Warmup"
                    if self.switch.active().is_some()
                        && !self.switch.pending()
                        && self.audio.state().phase == Phase::Paused =>
                {
                    self.control(false)?;
                    self.mark("Start")?;
                }
                "Start" if self.audio.state().phase == Phase::Playing => {
                    self.mark("Playing")?;
                }
                "Playing" => {
                    if self.audio.state().simulation_us >= self.seconds * 1_000_000 {
                        self.continuous_us = self.audio.state().simulation_us;
                        for doc in 0..10 {
                            self.commit(doc)?;
                        }
                        self.control(true)?;
                        self.mark("Pausing")?;
                    } else {
                        let spacing = if self.seconds == 3 { 0.5 } else { 10. };
                        if self.switches < if self.seconds == 3 { 2 } else { 20 }
                            && self.phase_start.elapsed().as_secs_f64()
                                >= (self.switches + 1) as f64 * spacing
                            && !self.switch.pending()
                        {
                            if let Some((doc, _)) = self.switch.active() {
                                self.commit(doc)?;
                            }
                            self.switches += 1;
                            let doc = self.switches % 10;
                            self.switch.request(self.root.clone(), doc)?;
                            self.requested = Some((doc, Instant::now()));
                        }
                    }
                }
                "Pausing"
                    if self.audio.state().phase == Phase::Paused && !self.switch.pending() =>
                {
                    self.frozen_us = self.audio.state().simulation_us;
                    self.mark("Diagnostic")?;
                    let doc = self.switch.active().ok_or("Missing active document")?.0;
                    self.pixel_checks
                        .push(self.switch.active().unwrap().1.verify(&self.output, 0)?);
                    self.edit(doc)?;
                    let position = self.owners[doc].visible_position();
                    self.switch.active_mut().unwrap().1.refresh(position)?;
                    self.commit(doc)?;
                    self.pixel_checks
                        .push(self.switch.active().unwrap().1.verify(&self.output, 1)?);
                    self.drain()?;
                    self.mark("PausedSettle")?;
                }
                "PausedSettle"
                    if self.phase_start.elapsed().as_secs()
                        >= if self.seconds == 3 { 1 } else { 10 } =>
                {
                    self.mark("PausedIdle")?;
                }
                "PausedIdle" => {
                    self.idle_frames += 1;
                    if self.audio.state().simulation_us != self.frozen_us {
                        return Err("Paused content clock changed".into());
                    }
                    if self.phase_start.elapsed() >= Duration::from_secs(self.idle_seconds) {
                        self.mark("Closing")?;
                        self.output_device.take();
                        self.decoder.take();
                        self.switch.close()?;
                        self.drain()?;
                        self.mark("ClosedSettle")?;
                    }
                }
                "ClosedSettle"
                    if !self.switch.pending()
                        && self.phase_start.elapsed().as_secs()
                            >= if self.seconds == 3 { 1 } else { 10 } =>
                {
                    self.mark("ClosedIdle")?;
                }
                "ClosedIdle" => {
                    self.closed_frames += 1;
                    if self.phase_start.elapsed() >= Duration::from_secs(self.idle_seconds) {
                        self.mark("Done")?;
                        self.finish(ctx);
                    }
                }
                _ => {}
            }
            if self.start.elapsed().as_secs() > self.seconds + 230 {
                return Err("Soak timeout".into());
            }
            if matches!(self.phase, "PausedIdle" | "ClosedIdle") {
                ctx.request_repaint_after(
                    Duration::from_secs(self.idle_seconds)
                        .saturating_sub(self.phase_start.elapsed()),
                );
            } else if matches!(self.phase, "PausedSettle" | "ClosedSettle")
                && !self.switch.pending()
            {
                ctx.request_repaint_after(
                    Duration::from_secs(if self.seconds == 3 { 1 } else { 10 })
                        .saturating_sub(self.phase_start.elapsed()),
                );
            } else {
                ctx.request_repaint_after(Duration::from_millis(16));
            }
            Ok(())
        }
        fn finish(&mut self, ctx: &egui::Context) {
            if self.finished {
                return;
            }
            self.output_device.take();
            self.decoder.take();
            if let Err(e) = self.drain() {
                self.errors.push(e);
            }
            let s = self.input.stats();
            if s.free_blocks != 5 || s.queued_blocks != 0 || s.in_flight || s.active {
                self.errors.push("PCM pool not retired".into());
            }
            let result = json!({"schema_version":1,"status":if self.errors.is_empty(){"passed"}else{"failed"},
                "errors":self.errors,"seconds":self.seconds,"idle_seconds":self.idle_seconds,
                "continuous_device_us":self.continuous_us,"switches":self.switches,"accepted_documents":self.switch.accepted,
                "changes":self.changes,"command_nodes":self.owners.iter().map(|o|o.snapshots().len()).sum::<usize>(),
                "switch_ms":self.switch_ms,"uploads":self.switch.uploads,"pipeline_warmup_ms":self.switch.pipeline_warmup_ms,
                "adapter":self.adapter,"audio_format":self.last.and_then(|s|s.format).map(|f|format!("{f:?}")),
                "final_stats":stats(s),"event_count":self.event_count,"submit_count":self.submit_count,
                "observations":self.samples,"phases":self.phases,"eframe_cpu_frame_ms_playing":self.frame_ms,
                "frame_intervals_ms_playing":self.frame_intervals_ms,"paused_idle_frames":self.idle_frames,
                "closed_idle_frames":self.closed_frames,"pixel_checks":self.pixel_checks,
                "system_capture_used":false,"physical_or_acoustic_latency_measured":false,
                "compressed_feature_enabled":cfg!(feature = "compressed-proof"),
                "normal_path_cpu_pixel_readbacks":0});
            fs::write(
                self.output.join("result.json"),
                serde_json::to_vec_pretty(&result).unwrap(),
            )
            .unwrap();
            self.finished = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
    impl eframe::App for App {
        fn raw_input_hook(&mut self, _ctx: &egui::Context, raw: &mut egui::RawInput) {
            raw.events.clear();
        }
        fn logic(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
            if self.finished {
                return;
            }
            let now = Instant::now();
            if self.phase == "Playing" && self.previous_playing {
                if self.frame_ms.len() >= 40000 {
                    self.errors.push("Frame sample limit".into());
                    self.finish(ctx);
                    return;
                }
                if let Some(cpu) = frame.info().cpu_usage {
                    self.frame_ms.push(f64::from(cpu) * 1000.);
                }
                if let Some(last) = self.last_frame {
                    self.frame_intervals_ms
                        .push(now.duration_since(last).as_secs_f64() * 1000.);
                }
            }
            self.previous_playing = self.phase == "Playing";
            self.last_frame = Some(now);
            if let Err(e) = self.poll(ctx) {
                self.errors.push(e);
                self.finish(ctx);
            }
        }
        fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
            if self.finished {
                return;
            }
            ui.label(format!(
                "P0 {} — 100 PNG / 10 documents / 500 layers",
                self.phase
            ));
            ui.label(format!(
                "Device content {:.3}s; document switches {}; draft updates {}",
                self.audio.state().simulation_us as f64 / 1_000_000.,
                self.switches,
                self.changes
            ));
            if let Some((doc, _)) = self.switch.active() {
                if self.phase == "Playing"
                    && let Err(e) = self.edit(doc)
                {
                    self.errors.push(e);
                }
                let position = self.owners[doc].visible_position();
                let (_, live) = self.switch.active_mut().unwrap();
                if let Err(e) = live.refresh(position) {
                    self.errors.push(e);
                }
                ui.add(egui::Image::new((live.texture_id, egui::vec2(640., 640.))));
            }
            if !self.errors.is_empty() {
                self.finish(ui.ctx());
            }
        }
    }
    pub fn run() -> eframe::Result {
        let mut args = std::env::args().skip(1);
        let output = PathBuf::from(args.next().expect("new output"));
        let root = PathBuf::from(args.next().expect("PNG fixture"));
        let source = PathBuf::from(args.next().expect("600-second audio fixture"));
        let seconds: u64 = args
            .next()
            .expect("300 or smoke 3 seconds")
            .parse()
            .unwrap();
        assert!(matches!(seconds, 3 | 300));
        assert!(args.next().is_none());
        let allowed = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("artifacts/p0")
            .canonicalize()
            .unwrap();
        assert_eq!(output.parent().unwrap().canonicalize().unwrap(), allowed);
        assert!(!output.exists());
        assert!(root.canonicalize().unwrap().starts_with(&allowed));
        assert!(source.canonicalize().unwrap().starts_with(&allowed));
        fs::create_dir(&output).unwrap();
        let mut setup = eframe::egui_wgpu::WgpuSetupCreateNew::without_display_handle();
        setup.instance_descriptor.backends = wgpu::Backends::VULKAN;
        let descriptor = setup.device_descriptor;
        setup.device_descriptor = Arc::new(move |a| {
            let mut d = descriptor(a);
            d.memory_hints = wgpu::MemoryHints::MemoryUsage;
            d
        });
        eframe::run_native(
            "P0 streaming and graphics soak",
            eframe::NativeOptions {
                viewport: egui::ViewportBuilder::default()
                    .with_inner_size([1100., 760.])
                    .with_visible(false),
                renderer: eframe::Renderer::Wgpu,
                persist_window: false,
                wgpu_options: eframe::egui_wgpu::WgpuConfiguration {
                    wgpu_setup: setup.into(),
                    ..Default::default()
                },
                ..Default::default()
            },
            Box::new(move |cc| {
                Ok(Box::new(
                    App::new(cc, root, output, source, seconds).map_err(std::io::Error::other)?,
                ))
            }),
        )
    }
}
#[cfg(windows)]
fn main() -> eframe::Result {
    native::run()
}
#[cfg(not(windows))]
fn main() {
    eprintln!("Windows-only P0 soak; other native platforms untested");
    std::process::exit(1);
}
