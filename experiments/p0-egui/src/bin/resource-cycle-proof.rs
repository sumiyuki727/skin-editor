//! P0 only: repeated document + decoder + native stream ownership cycles.
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
        path::{Path, PathBuf},
        sync::Arc,
        time::{Duration, Instant},
    };
    fn stats(s: buffered::Stats) -> Value {
        json!({"active":s.active,"generation":s.generation,"free_blocks":s.free_blocks,
            "queued_blocks":s.queued_blocks,"in_flight":s.in_flight,"max_queued_blocks":s.max_queued_blocks,
            "underruns":s.underruns,"decode_failures":s.decode_failures,"trace_overflow":s.trace_overflow,
            "stale_blocks":s.stale_blocks,"accepted_blocks":s.accepted_blocks,
            "native_discarded_frames":s.native_discarded_frames,"discarded_frames":s.discarded_frames,
            "packets":s.pcm.packets,"submitted_frames":s.pcm.submitted_frames})
    }
    fn observation(s: Snapshot) -> Value {
        json!({"sequence":s.sequence,"phase":format!("{:?}",s.phase),"position_us":s.position_us})
    }
    struct Session {
        device: Option<WindowsOutput>,
        decoder: Option<DecoderWorker>,
        input: buffered::Input,
        audio: Transport,
        last: Option<Snapshot>,
        trace: BufWriter<File>,
        events: Vec<buffered::Event>,
        event_count: usize,
        submit_count: usize,
        last_drain: Instant,
    }
    impl Session {
        fn new(source: PathBuf, trace_path: &Path, ctx: &egui::Context) -> Result<Self, String> {
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
            let wake = ctx.clone();
            let device = WindowsOutput::spawn_buffered(receiver, move || wake.request_repaint())
                .map_err(|e| format!("{e:?}"))?;
            Ok(Self {
                device: Some(device),
                decoder: Some(decoder),
                input,
                audio: Transport::default(),
                last: None,
                trace: BufWriter::new(File::create_new(trace_path).map_err(|e| e.to_string())?),
                events: Vec::with_capacity(buffered::TRACE_LIMIT),
                event_count: 0,
                submit_count: 0,
                last_drain: Instant::now(),
            })
        }
        fn poll(&mut self) -> Result<(), String> {
            let s = self
                .device
                .as_ref()
                .ok_or("Closed session polled")?
                .0
                .snapshot();
            if let Some(o) = WindowsOutput::observation(s) {
                self.audio.observe(o);
            }
            self.last = Some(s);
            let q = self.input.stats();
            if self.audio.state().phase == Phase::Faulted
                || q.underruns != 0
                || q.decode_failures != 0
                || q.trace_overflow
            {
                return Err(format!("Audio failure: {s:?} / {q:?}"));
            }
            if self.last_drain.elapsed() >= Duration::from_secs(1) {
                self.drain()?;
            }
            Ok(())
        }
        fn control(&mut self, paused: bool) -> Result<(), String> {
            let intent = self.audio.intent(paused);
            self.device
                .as_mut()
                .ok_or("No output")?
                .control(intent)
                .map_err(|e| format!("{e:?}"))
        }
        fn drain(&mut self) -> Result<(), String> {
            self.input.drain_trace(&mut self.events);
            for e in &self.events {
                let p = e.packet.map(|p| {
                    json!({"source_frame":p.source_frame,"frames":p.frames,"hash":p.hash,
                    "peak":p.peak,"nonzero_samples":p.nonzero_samples,"probe":p.probe})
                });
                serde_json::to_writer(&mut self.trace,&json!({"kind":format!("{:?}",e.kind),"generation":e.generation,
                    "first_frame":e.first_frame,"frames":e.frames,"native_discarded_frames":e.native_discarded_frames,"packet":p}))
                    .map_err(|e|e.to_string())?;
                self.trace.write_all(b"\n").map_err(|e| e.to_string())?;
                self.submit_count += usize::from(e.kind == buffered::Kind::Submit);
            }
            self.event_count += self.events.len();
            self.events.clear();
            self.trace.flush().map_err(|e| e.to_string())?;
            self.last_drain = Instant::now();
            Ok(())
        }
        fn close(mut self) -> Result<Value, String> {
            self.device.take();
            self.decoder.take();
            self.drain()?;
            let s = self.input.stats();
            if !self.input.is_closed()
                || s.active
                || s.in_flight
                || s.free_blocks != 5
                || s.queued_blocks != 0
                || s.underruns != 0
                || s.decode_failures != 0
                || s.trace_overflow
            {
                return Err(format!("Unretired session: {s:?}"));
            }
            Ok(
                json!({"final_stats":stats(s),"event_count":self.event_count,"submit_count":self.submit_count,
                "audio_format":self.last.and_then(|s|s.format).map(|f|format!("{f:?}"))}),
            )
        }
    }
    // Device must stop before joining the producer even on an early error.
    impl Drop for Session {
        fn drop(&mut self) {
            self.device.take();
            self.decoder.take();
        }
    }
    struct App {
        root: PathBuf,
        source: PathBuf,
        output: PathBuf,
        smoke: bool,
        start: Instant,
        phase_start: Instant,
        opened: Instant,
        phase: &'static str,
        step: u64,
        cycle: usize,
        target: usize,
        switch: DocumentSwitch,
        session: Option<Session>,
        owner: Owner,
        draft: Option<u64>,
        changes: usize,
        last_edit_frame: Option<u64>,
        continuous_us: u64,
        frozen: Option<Snapshot>,
        pause_start: Option<Instant>,
        open_ms: f64,
        cycle_frames_begin: usize,
        frame_ms: Vec<f64>,
        intervals_ms: Vec<f64>,
        last_frame: Option<Instant>,
        previous_play: bool,
        phases: Vec<Value>,
        cycles: Vec<Value>,
        closing: Option<Value>,
        pixel_checks: Vec<Value>,
        final_idle_frames: usize,
        errors: Vec<String>,
        finished: bool,
        adapter: String,
    }
    impl App {
        fn new(
            cc: &eframe::CreationContext<'_>,
            root: PathBuf,
            source: PathBuf,
            output: PathBuf,
            smoke: bool,
        ) -> Result<Self, String> {
            let state = cc.wgpu_render_state.clone().ok_or("wgpu required")?;
            let adapter = format!("{:?}", state.adapter.get_info());
            let mut switch = DocumentSwitch::new(state, &cc.egui_ctx);
            switch.warm_pipelines(&cc.egui_ctx);
            let mut app = Self {
                root,
                source,
                output,
                smoke,
                start: Instant::now(),
                phase_start: Instant::now(),
                opened: Instant::now(),
                phase: "Open",
                step: 0,
                cycle: 0,
                target: if smoke { 2 } else { 20 },
                switch,
                session: None,
                owner: Owner::default(),
                draft: None,
                changes: 0,
                last_edit_frame: None,
                continuous_us: 0,
                frozen: None,
                pause_start: None,
                open_ms: 0.,
                cycle_frames_begin: 0,
                frame_ms: Vec::with_capacity(20000),
                intervals_ms: Vec::with_capacity(20000),
                last_frame: None,
                previous_play: false,
                phases: Vec::with_capacity(200),
                cycles: Vec::with_capacity(20),
                closing: None,
                pixel_checks: vec![],
                final_idle_frames: 0,
                errors: vec![],
                finished: false,
                adapter,
            };
            app.open(&cc.egui_ctx)?;
            Ok(app)
        }
        fn mark(&mut self, phase: &'static str) -> Result<(), String> {
            self.phase = phase;
            self.phase_start = Instant::now();
            self.step += 1;
            if self.phases.len() >= 200 {
                return Err("Phase limit".into());
            }
            let row = json!({"phase":phase,"cycle":self.cycle,"step":self.step,"elapsed_seconds":self.start.elapsed().as_secs_f64()});
            self.phases.push(row.clone());
            fs::write(self.output.join("phase.json"), row.to_string()).map_err(|e| e.to_string())
        }
        fn open(&mut self, ctx: &egui::Context) -> Result<(), String> {
            if self.session.is_some()
                || self.switch.active().is_some()
                || (self.cycle != 0 && self.switch.pending())
            {
                return Err("Previous resources still owned".into());
            }
            self.opened = Instant::now();
            self.owner = Owner::default();
            self.draft = None;
            self.changes = 0;
            self.frozen = None;
            self.pause_start = None;
            self.cycle_frames_begin = self.frame_ms.len();
            self.continuous_us = 0;
            self.last_edit_frame = None;
            self.session = Some(Session::new(
                self.source.clone(),
                &self
                    .output
                    .join(format!("cycle-{:02}-packets.jsonl", self.cycle)),
                ctx,
            )?);
            self.switch.request(self.root.clone(), self.cycle % 10)?;
            self.mark("Open")
        }
        fn edit(&mut self) -> Result<(), String> {
            if self.draft.is_none() {
                let expected_node = self.owner.committed().id;
                let Outcome::Started(id) = self
                    .owner
                    .execute(Command::Begin { expected_node })
                    .map_err(|e| format!("{e:?}"))?
                else {
                    return Err("No draft".into());
                };
                self.draft = Some(id);
            }
            let n = self.changes as f64;
            self.owner
                .execute(Command::Move {
                    session: self.draft.unwrap(),
                    position: [40. + 20. * (n * 0.04).sin(), 40. + 14. * (n * 0.03).cos()],
                })
                .map_err(|e| format!("{e:?}"))?;
            self.changes += 1;
            Ok(())
        }
        fn commit(&mut self) -> Result<(), String> {
            if let Some(session) = self.draft.take() {
                self.owner
                    .execute(Command::Commit { session })
                    .map_err(|e| format!("{e:?}"))?;
            }
            Ok(())
        }
        fn poll(&mut self, ctx: &egui::Context) -> Result<(), String> {
            self.switch.poll(ctx.cumulative_frame_nr(), ctx)?;
            if let Some(e) = &self.switch.error {
                return Err(e.clone());
            }
            if let Some(s) = &mut self.session {
                s.poll()?;
            }
            match self.phase {
                "Open"
                    if !self.switch.pending()
                        && self.switch.active().is_some()
                        && self.session.as_ref().unwrap().audio.state().phase == Phase::Paused =>
                {
                    self.open_ms = self.opened.elapsed().as_secs_f64() * 1000.;
                    self.session.as_mut().unwrap().control(false)?;
                    self.mark("Starting")?;
                }
                "Starting"
                    if self.session.as_ref().unwrap().audio.state().phase == Phase::Playing =>
                {
                    self.mark("Play")?;
                }
                "Play" => {
                    let s = self.session.as_ref().unwrap();
                    if s.audio.state().simulation_us
                        >= if self.smoke { 1_000_000 } else { 6_000_000 }
                        && self.changes >= if self.smoke { 20 } else { 300 }
                    {
                        self.continuous_us = s.audio.state().simulation_us;
                        self.commit()?;
                        self.session.as_mut().unwrap().control(true)?;
                        self.mark("Pause")?;
                    }
                }
                "Pause" if self.session.as_ref().unwrap().audio.state().phase == Phase::Paused => {
                    self.frozen = self.session.as_ref().unwrap().last;
                    self.pause_start = Some(Instant::now());
                    self.mark("Paused")?;
                }
                "Paused" if self.phase_start.elapsed() >= Duration::from_secs(1) => {
                    let first = self.frozen.ok_or("No pause start")?;
                    let s = self.session.as_ref().unwrap();
                    let last = s.last.ok_or("No pause end")?;
                    if last.sequence <= first.sequence
                        || last.position_us != first.position_us
                        || s.input.stats().pcm.packets != 0
                    {
                        return Err("Paused native content changed or observation stale".into());
                    }
                    let pause = json!({"first":observation(first),"last":observation(last),"wall_seconds":self.pause_start.unwrap().elapsed().as_secs_f64()});
                    if self.cycle == 0 || self.cycle + 1 == self.target {
                        self.mark("Diagnostic")?;
                        self.edit()?;
                        self.switch
                            .active_mut()
                            .unwrap()
                            .1
                            .refresh(self.owner.visible_position())?;
                        self.commit()?;
                        let proof = self
                            .switch
                            .active()
                            .unwrap()
                            .1
                            .verify(&self.output, self.cycle as u16)?;
                        self.pixel_checks.push(proof);
                    }
                    self.mark("Closing")?;
                    let closed = self.session.take().ok_or("Missing session")?.close()?;
                    self.switch.close()?;
                    self.closing = Some(
                        json!({"cycle":self.cycle,"document":self.cycle%10,"open_ms":self.open_ms,
                        "continuous_device_us":self.continuous_us,"changes":self.changes,"command_nodes":self.owner.snapshots().len(),
                        "frame_samples":self.frame_ms.len()-self.cycle_frames_begin,"pause":pause,"closed":closed}),
                    );
                    self.mark("ClosedSettle")?;
                }
                "ClosedSettle"
                    if !self.switch.pending()
                        && self.phase_start.elapsed()
                            >= Duration::from_secs(if self.smoke { 1 } else { 4 }) =>
                {
                    if self.switch.active().is_some() || self.session.is_some() {
                        return Err("Closed resources still active".into());
                    }
                    self.mark("Closed")?;
                }
                "Closed"
                    if self.phase_start.elapsed()
                        >= Duration::from_secs(if self.smoke { 1 } else { 4 }) =>
                {
                    let row = self.closing.take().ok_or("Missing closed cycle")?;
                    let file =
                        File::create_new(self.output.join(format!("cycle-{:02}.json", self.cycle)))
                            .map_err(|e| e.to_string())?;
                    serde_json::to_writer_pretty(file, &row).map_err(|e| e.to_string())?;
                    self.cycles.push(row);
                    if self.cycle + 1 == self.target {
                        self.mark("FinalIdle")?;
                    } else {
                        self.cycle += 1;
                        self.open(ctx)?;
                    }
                }
                "FinalIdle" => {
                    self.final_idle_frames += 1;
                    if self.phase_start.elapsed()
                        >= Duration::from_secs(if self.smoke { 2 } else { 65 })
                    {
                        self.mark("Done")?;
                        self.finish(ctx);
                    }
                }
                _ => {}
            }
            if self.start.elapsed().as_secs() > if self.smoke { 80 } else { 600 } {
                return Err("Cycle proof timeout".into());
            }
            if matches!(
                self.phase,
                "ClosedSettle" | "Closed" | "Paused" | "FinalIdle"
            ) && !self.switch.pending()
            {
                let wait = match self.phase {
                    "Paused" => 1,
                    "FinalIdle" => {
                        if self.smoke {
                            2
                        } else {
                            65
                        }
                    }
                    _ => {
                        if self.smoke {
                            1
                        } else {
                            4
                        }
                    }
                };
                ctx.request_repaint_after(
                    Duration::from_secs(wait).saturating_sub(self.phase_start.elapsed()),
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
            if let Some(s) = self.session.take()
                && let Err(e) = s.close()
            {
                self.errors.push(e);
            }
            if let Err(e) = self.switch.close() {
                self.errors.push(e);
            }
            let result = json!({"schema_version":1,"status":if self.errors.is_empty(){"passed"}else{"failed"},"errors":self.errors,
                "smoke":self.smoke,"target_cycles":self.target,"cycles":self.cycles,"phases":self.phases,
                "accepted_documents":self.switch.accepted,"uploads":self.switch.uploads,"adapter":self.adapter,
                "pipeline_warmup_ms":self.switch.pipeline_warmup_ms,"eframe_cpu_frame_ms_playing":self.frame_ms,
                "frame_intervals_ms_playing":self.intervals_ms,"final_idle_frames":self.final_idle_frames,"pixel_checks":self.pixel_checks,
                "session_owned_at_finish":self.session.is_some(),"active_document_at_finish":self.switch.active().is_some(),
                "background_pending_at_finish":self.switch.pending(),"system_capture_used":false,
                "normal_path_cpu_pixel_readbacks":0,"physical_or_acoustic_latency_measured":false});
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
            if self.phase == "Play" && self.previous_play {
                if self.frame_ms.len() >= 20000 || self.intervals_ms.len() >= 20000 {
                    self.errors.push("Frame sample limit".into());
                    self.finish(ctx);
                    return;
                }
                if let Some(cpu) = frame.info().cpu_usage {
                    self.frame_ms.push(f64::from(cpu) * 1000.);
                }
                if let Some(last) = self.last_frame {
                    self.intervals_ms
                        .push(now.duration_since(last).as_secs_f64() * 1000.);
                }
            }
            self.previous_play = self.phase == "Play";
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
                "P0 resource cycle {}/{} — {}",
                self.cycle + 1,
                self.target,
                self.phase
            ));
            ui.label("Standard 100 PNG / 10 documents / 500 layers; one active 50-layer document");
            if self.phase == "Play" && self.last_edit_frame != Some(ui.ctx().cumulative_frame_nr())
            {
                self.last_edit_frame = Some(ui.ctx().cumulative_frame_nr());
                if let Err(e) = self.edit() {
                    self.errors.push(e);
                }
            }
            if let Some((_, live)) = self.switch.active_mut() {
                if let Err(e) = live.refresh(self.owner.visible_position()) {
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
        let source = PathBuf::from(args.next().expect("WAV source"));
        let mode = args.next().expect("full or smoke");
        assert!(matches!(mode.as_str(), "full" | "smoke"));
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
            "P0 resource ownership cycles",
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
                    App::new(cc, root, source, output, mode == "smoke")
                        .map_err(std::io::Error::other)?,
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
    eprintln!("Windows-only P0 resource cycles; other native platforms untested");
    std::process::exit(1);
}
