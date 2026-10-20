//! Same-process background document handoff with one real output clock.
#[cfg(windows)]
mod native {
    use eframe::{egui, wgpu};
    use p0_egui::{
        audio_transport::{AudioObservation, Fault, Phase, Transport},
        composition_switch::DocumentSwitch,
        interaction_view::GestureView,
    };
    use p0_preview_contract::Tool;
    use p0_wasapi::{Device, Phase as DevicePhase};
    use serde_json::{Value, json};
    use std::{
        path::PathBuf,
        sync::Arc,
        time::{Duration, Instant},
    };
    struct App {
        switch: DocumentSwitch,
        views: [GestureView; 10],
        audio: Transport,
        device: Device,
        root: PathBuf,
        output: PathBuf,
        start: Instant,
        phase_start: Instant,
        paused_start: Option<Instant>,
        step: u8,
        input_step: Option<u8>,
        origin: egui::Pos2,
        last_doc: usize,
        edits: usize,
        after_inputs: u8,
        before_sent: bool,
        frozen: Option<u64>,
        resumed: Option<u64>,
        second_frozen: Option<u64>,
        paused_wall_ms: f64,
        last_sequence: u64,
        samples: Vec<Value>,
        proofs: Vec<Value>,
        errors: Vec<String>,
        cancelled_draft: bool,
        failed_kept_old: bool,
        cancel_kept_old: bool,
        playing_switch: bool,
        closed_loading: bool,
        finished: bool,
    }
    fn key(key: egui::Key, pressed: bool, repeat: bool) -> egui::Event {
        egui::Event::Key {
            key,
            physical_key: None,
            pressed,
            repeat,
            modifiers: Default::default(),
        }
    }
    fn pointer(pos: egui::Pos2, pressed: bool) -> egui::Event {
        egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: Default::default(),
        }
    }
    impl App {
        fn new(
            cc: &eframe::CreationContext<'_>,
            root: PathBuf,
            output: PathBuf,
        ) -> Result<Self, String> {
            let wake = cc.egui_ctx.clone();
            let device =
                Device::spawn(move || wake.request_repaint()).map_err(|e| format!("{e:?}"))?;
            let mut switch = DocumentSwitch::new(
                cc.wgpu_render_state.clone().ok_or("wgpu required")?,
                &cc.egui_ctx,
            );
            switch.warm_pipelines(&cc.egui_ctx);
            switch.request(root.clone(), 0)?;
            let mut views = std::array::from_fn(|_| GestureView::default());
            views[0].set_tool(Tool::Interact);
            Ok(Self {
                switch,
                views,
                audio: Transport::default(),
                device,
                root,
                output,
                start: Instant::now(),
                phase_start: Instant::now(),
                paused_start: None,
                step: 0,
                input_step: None,
                origin: egui::Pos2::ZERO,
                last_doc: 0,
                edits: 0,
                after_inputs: 0,
                before_sent: false,
                frozen: None,
                resumed: None,
                second_frozen: None,
                paused_wall_ms: 0.,
                last_sequence: 0,
                samples: vec![],
                proofs: vec![],
                errors: vec![],
                cancelled_draft: false,
                failed_kept_old: false,
                cancel_kept_old: false,
                playing_switch: false,
                closed_loading: false,
                finished: false,
            })
        }
        fn control(&mut self, paused: bool) {
            let intent = self.audio.intent(paused);
            if let Err(e) = self.device.request(intent.request, intent.paused) {
                self.errors.push(format!("{e:?}"));
                self.audio.fail(Fault::Backend);
            }
        }
        fn request(&mut self, doc: usize, missing: bool) {
            if let Some((old, _)) = self.switch.active() {
                self.views[old].cancel();
            }
            let root = if missing {
                self.root.join("missing-audio-switch-fixture")
            } else {
                self.root.clone()
            };
            if let Err(e) = self.switch.request(root, doc) {
                self.errors.push(e);
            }
        }
        fn proof(&mut self, tag: u16) {
            if self.audio.state().phase != Phase::Paused {
                self.errors.push("Pixel diagnostic while playing".into());
                return;
            }
            if let Some((doc, v)) = self.switch.active() {
                match v.verify(&self.output, tag) {
                    Ok(mut p) => {
                        p["document"] = json!(doc);
                        p["simulation_us"] = json!(self.audio.state().simulation_us);
                        p["accepted_inputs"] = json!(self.audio.state().accepted_inputs);
                        self.proofs.push(p);
                    }
                    Err(e) => self.errors.push(e),
                }
            }
        }
        fn finish(&mut self, ctx: &egui::Context) {
            if self.finished {
                return;
            }
            self.finished = true;
            let s = self.audio.state();
            let paused: Vec<_> = self
                .samples
                .iter()
                .filter(|r| r["request"] == 2 && r["phase"] == "Paused")
                .collect();
            let span = paused
                .iter()
                .filter_map(|r| r["device_us"].as_u64())
                .min()
                .zip(paused.iter().filter_map(|r| r["device_us"].as_u64()).max())
                .map(|(a, b)| b - a);
            let nodes: Vec<_> = self.views.iter().map(|v| v.node_count()).collect();
            let checks = json!({"clock_advanced_before_pause":self.frozen.is_some_and(|n|n>=200000),"paused_fresh_samples":paused.len()>=3,"paused_span_zero":span==Some(0),
                "paused_wall_observed":self.paused_wall_ms>=300.,"resume_no_catchup":self.frozen.zip(self.resumed).is_some_and(|(f,r)|r>=f&&r-f<=50000),
                "playing_document_switch":self.playing_switch,"single_shared_clock":self.samples.iter().all(|r|r["device_us"]==r["simulation_us"]),
                "fresh_input_only":s.accepted_inputs==2,"document_draft_cancelled":self.cancelled_draft&&nodes[0]==1,"single_document_commit":nodes.iter().enumerate().all(|(i,n)|*n==if i==2{2}else{1}),
                "cancel_keeps_old":self.cancel_kept_old,"failed_keeps_old":self.failed_kept_old,"stale_gpu_discarded":self.switch.stale_gpu>=2,
                "close_during_upload":self.closed_loading&&self.switch.active().is_none()&&!self.switch.pending(),"final_pause_preserved":s.phase==Phase::Paused&&self.second_frozen==Some(s.simulation_us),
                "six_pixel_checks":self.proofs.len()==6,"thirty_draft_updates":self.edits==30,"no_fault":self.errors.is_empty()});
            let passed = checks.as_object().unwrap().values().all(|v| v == true);
            let format=self.device.snapshot().format.map(|f|json!({"rate":f.rate,"channels":f.channels,"bits":f.bits,"buffer_frames":f.buffer_frames,"clock_frequency":f.clock_frequency}));
            let result = json!({"schema_version":1,"status":if passed{"passed"}else{"failed"},"checks":checks,"errors":self.errors,"audio_format":format,
                "frozen_us":self.frozen,"resume_us":self.resumed,"second_frozen_us":self.second_frozen,"final_simulation_us":s.simulation_us,"paused_wall_ms":self.paused_wall_ms,"paused_span_us":span,"paused_samples":paused.len(),
                "accepted_inputs":s.accepted_inputs,"edit_count":self.edits,"nodes_per_document":nodes,"fresh_samples":self.samples,"proofs":self.proofs,
                "stale_gpu":self.switch.stale_gpu,"accepted_documents":self.switch.accepted,"uploads":self.switch.uploads,"pipeline_warmup_ms":self.switch.pipeline_warmup_ms,
                "system_capture_used":false,"limits":["Real default silent shared output; synthetic egui keys and pointer; no acoustic/physical input latency",
                    "One borrowed preview transport, ten independent gesture owners; fixture indices are not a product identity schema",
                    "Missing fixture failure and request cancellation, not physical device-unplug recovery",
                    "Pixel diagnostics block the UI only while paused; no frame/idle/whole-application budget claim",
                    "Obsolete GPU candidates drain all remaining bounded source writes; no preemption, stable rules or gameplay"]});
            std::fs::write(
                self.output.join("result.json"),
                serde_json::to_vec_pretty(&result).unwrap(),
            )
            .unwrap();
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        fn advance(&mut self, ctx: &egui::Context) {
            if self.input_step.take() != Some(self.step) {
                return;
            }
            let s = self.audio.state();
            let active = self.switch.active().map(|(d, _)| d);
            match self.step {
                0 if s.phase == Phase::Paused => {
                    self.control(false);
                    self.step = 1;
                }
                1 if s.phase == Phase::Playing => {
                    self.phase_start = Instant::now();
                    self.step = 2;
                }
                2 if self.before_sent
                    && active == Some(0)
                    && !self.switch.pending()
                    && self.phase_start.elapsed() >= Duration::from_millis(450) =>
                {
                    self.control(true);
                    self.step = 3;
                }
                3 if s.phase == Phase::Paused => {
                    self.frozen = Some(s.simulation_us);
                    self.paused_start = Some(Instant::now());
                    self.views[0].set_tool(Tool::Edit);
                    self.proof(0);
                    self.step = 4;
                }
                4 => self.step = 5,
                5 => {
                    self.cancelled_draft =
                        self.views[0].position() != self.views[0].snapshot().position;
                    self.request(1, false);
                    self.cancelled_draft &= self.views[0].position() == [40., 40.];
                    self.step = 6;
                }
                6 if self.switch.preparing() => {
                    self.proof(5);
                    self.request(2, false);
                    self.step = 7;
                }
                7 if active == Some(2) && !self.switch.pending() => {
                    self.proof(1);
                    self.step = 8;
                }
                8 => self.step = 9,
                9 => {
                    self.edits += 1;
                    if self.edits == 30 {
                        self.step = 10;
                    }
                }
                10 => {
                    self.proof(2);
                    self.request(3, true);
                    self.step = 11;
                }
                11 if self.switch.error.is_some() => {
                    self.failed_kept_old = active == Some(2) && self.views[2].node_count() == 2;
                    self.proof(3);
                    self.switch.error = None;
                    self.request(4, false);
                    self.switch.cancel().unwrap();
                    self.step = 12;
                }
                12 if !self.switch.pending() => {
                    self.cancel_kept_old = active == Some(2);
                    self.views[2].set_tool(Tool::Interact);
                    self.control(false);
                    self.step = 13;
                }
                13 if s.phase == Phase::Playing => {
                    self.resumed = Some(s.simulation_us);
                    self.paused_wall_ms =
                        self.paused_start.unwrap().elapsed().as_secs_f64() * 1000.;
                    self.phase_start = Instant::now();
                    self.step = 14;
                }
                14 => {
                    self.after_inputs = self.after_inputs.saturating_add(1);
                    if self.after_inputs >= 4
                        && self.phase_start.elapsed() >= Duration::from_millis(200)
                    {
                        self.request(5, false);
                        self.step = 15;
                    }
                }
                15 if active == Some(5)
                    && !self.switch.pending()
                    && s.phase == Phase::Playing
                    && self.phase_start.elapsed() >= Duration::from_millis(450)
                    && self.samples.iter().any(|r| {
                        r["request"] == 3 && r["phase"] == "Playing" && r["document"] == 5
                    }) =>
                {
                    self.playing_switch = s.simulation_us >= self.resumed.unwrap() + 200000;
                    self.control(true);
                    self.step = 16;
                }
                16 if s.phase == Phase::Paused => {
                    self.second_frozen = Some(s.simulation_us);
                    self.proof(4);
                    self.request(6, false);
                    self.step = 17;
                }
                17 if self.switch.preparing() => {
                    self.switch.close().unwrap();
                    self.closed_loading = true;
                    self.step = 18;
                }
                18 if !self.switch.pending() => self.finish(ctx),
                _ => {}
            }
        }
    }
    impl eframe::App for App {
        fn raw_input_hook(&mut self, _ctx: &egui::Context, raw: &mut egui::RawInput) {
            raw.events.clear();
            raw.focused = true;
            self.input_step = Some(self.step);
            raw.events = match self.step {
                2 if !self.before_sent => {
                    self.before_sent = true;
                    vec![key(egui::Key::Z, true, false)]
                }
                4 | 8 => {
                    self.origin = self.views[self.last_doc].target().center();
                    vec![
                        egui::Event::PointerMoved(self.origin),
                        pointer(self.origin, true),
                    ]
                }
                5 => vec![
                    egui::Event::PointerMoved(self.origin + egui::vec2(30., 20.)),
                    key(egui::Key::Z, true, true),
                ],
                6 => vec![
                    pointer(self.origin + egui::vec2(30., 20.), false),
                    key(egui::Key::Z, true, true),
                ],
                9 => vec![
                    egui::Event::PointerMoved(
                        self.origin
                            + if self.edits == 29 {
                                egui::vec2(60., 30.)
                            } else {
                                egui::vec2(60. + self.edits as f32, 30.)
                            },
                    ),
                    key(egui::Key::Z, true, true),
                ],
                10 => vec![pointer(self.origin + egui::vec2(60., 30.), false)],
                11 => vec![
                    key(egui::Key::X, true, false),
                    key(egui::Key::Z, true, true),
                ],
                14 => match self.after_inputs {
                    0 => vec![
                        key(egui::Key::Z, true, true),
                        key(egui::Key::X, true, false),
                    ],
                    1 => vec![
                        key(egui::Key::Z, false, false),
                        key(egui::Key::X, false, false),
                    ],
                    2 => vec![key(egui::Key::Z, true, false)],
                    3 => vec![key(egui::Key::Z, false, false)],
                    _ => vec![],
                },
                _ => vec![],
            };
        }
        fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
            if self.finished {
                return;
            }
            let sample = self.device.snapshot();
            if sample.phase != DevicePhase::Initializing {
                self.audio.observe(AudioObservation {
                    epoch: 0,
                    request: sample.request,
                    sequence: sample.sequence,
                    position_us: sample.position_us,
                    paused: sample.phase == DevicePhase::Paused,
                    fault: sample.phase == DevicePhase::Faulted,
                });
                let s = self.audio.state();
                if sample.sequence != self.last_sequence
                    && s.phase != Phase::Pending
                    && sample.request == s.request
                {
                    self.samples.push(json!({"request":sample.request,"sequence":sample.sequence,"phase":format!("{:?}",sample.phase),"device_us":sample.position_us,"simulation_us":s.simulation_us,"document":self.switch.active().map(|(d,_)|d)}));
                    self.last_sequence = sample.sequence;
                }
                if s.phase == Phase::Faulted {
                    self.errors
                        .push(format!("Audio failure {:?} / {:?}", sample.error, s.fault));
                    self.finish(ctx);
                    return;
                }
            }
            if let Err(e) = self.switch.poll(ctx.cumulative_frame_nr(), ctx) {
                self.errors.push(e);
                self.finish(ctx);
                return;
            }
            if self.start.elapsed() > Duration::from_secs(100) {
                self.errors.push("Combined proof timeout".into());
                self.finish(ctx);
                return;
            }
            ctx.request_repaint_after(Duration::from_millis(16));
        }
        fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
            if self.finished {
                return;
            }
            let texture = if let Some((doc, v)) = self.switch.active() {
                self.last_doc = doc;
                Some(v.texture_id)
            } else {
                None
            };
            self.views[self.last_doc].show_transport(ui, texture, &mut self.audio, false);
            if let Some((doc, v)) = self.switch.active_mut()
                && let Err(e) = v.refresh(self.views[doc].position())
            {
                self.errors.push(e);
            }
            if let Some(intent) = self.views[self.last_doc].take_audio_intent()
                && let Err(e) = self.device.request(intent.request, intent.paused)
            {
                self.errors.push(format!("{e:?}"));
                self.audio.fail(Fault::Backend);
            }
            self.advance(ui.ctx());
        }
    }
    pub fn run() -> eframe::Result {
        let mut args = std::env::args().skip(1);
        let output = PathBuf::from(args.next().expect("new output"));
        let root = PathBuf::from(args.next().expect("fixture"));
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
        std::fs::create_dir(&output).unwrap();
        let mut setup = eframe::egui_wgpu::WgpuSetupCreateNew::without_display_handle();
        setup.instance_descriptor.backends = wgpu::Backends::VULKAN;
        let descriptor = setup.device_descriptor;
        setup.device_descriptor = Arc::new(move |a| {
            let mut d = descriptor(a);
            d.memory_hints = wgpu::MemoryHints::MemoryUsage;
            d
        });
        eframe::run_native(
            "P0 shared output and document switching",
            eframe::NativeOptions {
                viewport: egui::ViewportBuilder::default()
                    .with_inner_size([1100., 720.])
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
                    App::new(cc, root, output).map_err(std::io::Error::other)?,
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
    eprintln!("P0 real output integration currently requires Windows");
    std::process::exit(1);
}
