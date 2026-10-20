//! Opt-in native proof: real silent output clock, input gate and GPU editing.
#[cfg(windows)]
mod native {
    use eframe::egui;
    use p0_egui::{
        audio_transport::{AudioObservation, Phase},
        composition_view::LiveComposition,
        interaction_view::GestureView,
    };
    use p0_preview_contract::Tool;
    use p0_wasapi::{Device, Phase as DevicePhase};
    use serde_json::{Value, json};
    use std::{
        path::PathBuf,
        sync::{Arc, mpsc},
        time::{Duration, Instant},
    };
    struct App {
        view: GestureView,
        device: Device,
        composition: LiveComposition,
        output: PathBuf,
        start: Instant,
        phase_start: Instant,
        step: u8,
        input_step: Option<u8>,
        origin: egui::Pos2,
        edits: usize,
        before_sent: bool,
        after_inputs: u8,
        received: [bool; 4],
        finished: bool,
        tx: mpsc::Sender<(usize, Arc<egui::ColorImage>)>,
        rx: mpsc::Receiver<(usize, Arc<egui::ColorImage>)>,
        crops: Vec<Vec<u8>>,
        captures: Vec<Value>,
        pixels: Vec<Value>,
        samples: Vec<Value>,
        last_sequence: u64,
        frozen_us: Option<u64>,
        resume_us: Option<u64>,
        paused_start: Option<Instant>,
        paused_wall_ms: f64,
        errors: Vec<String>,
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
            fixture: PathBuf,
            output: PathBuf,
        ) -> Result<Self, String> {
            let composition = LiveComposition::new(cc, &fixture)?;
            let wake = cc.egui_ctx.clone();
            let device =
                Device::spawn(move || wake.request_repaint()).map_err(|e| format!("{e:?}"))?;
            let mut view = GestureView::default();
            view.enable_audio();
            view.set_tool(Tool::Interact);
            let (tx, rx) = mpsc::channel();
            Ok(Self {
                view,
                device,
                composition,
                output,
                start: Instant::now(),
                phase_start: Instant::now(),
                step: 0,
                input_step: None,
                origin: egui::Pos2::ZERO,
                edits: 0,
                before_sent: false,
                after_inputs: 0,
                received: [false; 4],
                finished: false,
                tx,
                rx,
                crops: Vec::new(),
                captures: Vec::new(),
                pixels: Vec::new(),
                samples: Vec::new(),
                last_sequence: 0,
                frozen_us: None,
                resume_us: None,
                paused_start: None,
                paused_wall_ms: 0.,
                errors: Vec::new(),
            })
        }
        fn capture(&self, ctx: &egui::Context, tag: usize) {
            let tx = self.tx.clone();
            let wake = ctx.clone();
            ctx.request_screenshot(move |image| {
                let _ = tx.send((tag, image));
                wake.request_repaint();
            });
        }
        fn finish(&mut self, ctx: &egui::Context) {
            if self.finished {
                return;
            }
            self.finished = true;
            let state = self.view.audio_state().unwrap();
            let paused: Vec<_> = self
                .samples
                .iter()
                .filter(|s| s["phase"] == "Paused" && s["request"] == 2)
                .collect();
            let positions: Vec<_> = paused
                .iter()
                .filter_map(|s| s["device_us"].as_u64())
                .collect();
            let span = positions
                .iter()
                .max()
                .zip(positions.iter().min())
                .map(|(a, b)| a - b);
            let capture_ok = self.crops.len() == 4;
            let crop_eq = |a, b| capture_ok && self.crops[a] == self.crops[b];
            let capture_nodes: Vec<_> = self
                .captures
                .iter()
                .filter_map(|c| c["nodes"].as_u64())
                .collect();
            let capture_inputs: Vec<_> = self
                .captures
                .iter()
                .filter_map(|c| c["accepted_inputs"].as_u64())
                .collect();
            let checks = json!({"audio_clock_was_advancing":self.frozen_us.is_some_and(|n|n>=200000),
                "fresh_paused_device_samples":positions.len()>=3,"paused_device_span_is_zero":span==Some(0),
                "paused_wall_at_least_300_ms":self.paused_wall_ms>=300.,
                "draft_changes_canvas":capture_ok && !crop_eq(0,1),"cancel_restores_canvas":crop_eq(0,2),"commit_changes_canvas":capture_ok && !crop_eq(0,3),
                "single_commit":capture_nodes==[1,1,1,2] && self.view.node_count()==2,
                "paused_inputs_discarded":capture_inputs==[1,1,1,1],"fresh_press_after_resume_only":state.accepted_inputs==2,
                "resume_does_not_catch_up":self.resume_us.zip(self.frozen_us).is_some_and(|(r,f)|r>=f && r-f<=50000),
                "simulation_advances_after_resume":self.resume_us.is_some_and(|r|state.simulation_us>=r+200000),
                "simulation_uses_device_clock":self.samples.iter().all(|s|s["device_us"]==s["simulation_us"]),
                "three_hundred_edits":self.edits==300,"four_composition_proofs":self.pixels.len()==4,
                "no_fault":self.errors.is_empty() && state.phase!=Phase::Faulted});
            let passed = checks.as_object().unwrap().values().all(|v| v == true);
            let format=self.device.snapshot().format.map(|f|json!({"rate":f.rate,"channels":f.channels,"bits":f.bits,"buffer_frames":f.buffer_frames,"clock_frequency":f.clock_frequency}));
            let result = json!({"schema_version":1,"status":if passed {"passed"}else{"failed"},"checks":checks,"errors":self.errors,
                "audio_format":format,"captures":self.captures,"composition_pixel_checks":self.pixels,"composition":self.composition.metrics(),
                "fresh_samples":self.samples,"paused_device_samples":positions.len(),"paused_device_span_us":span,
                "paused_wall_ms":self.paused_wall_ms,"frozen_us":self.frozen_us,"resume_us":self.resume_us,"final_simulation_us":state.simulation_us,"accepted_inputs":state.accepted_inputs,"edit_count":self.edits,
                "system_capture_used":false,"input":"Synthetic egui Z/X and pointer events through the same view/transport gate as native input",
                "limits":["Real default shared output, silent PCM only; not acoustic latency or confirmed physical endpoint","100 ms buffer / polling adapter, not a selected production audio backend",
                    "CPU/GPU/display spot checks block UI while the device is paused; not edit-latency or full application budgets","No actual stable rules, hit judgement, physical input, IME or cross-platform verification"]});
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
            let state = self.view.audio_state().unwrap();
            match self.step {
                0 if state.phase == Phase::Paused => {
                    self.view.pause(false, 0);
                    self.step = 1;
                }
                1 if state.phase == Phase::Playing => {
                    self.phase_start = Instant::now();
                    self.step = 2;
                }
                2 if self.before_sent
                    && self.phase_start.elapsed() >= Duration::from_millis(450) =>
                {
                    self.view.pause(true, 0);
                    self.step = 3;
                }
                3 if state.phase == Phase::Paused => {
                    self.frozen_us = Some(state.simulation_us);
                    self.paused_start = Some(Instant::now());
                    self.view.set_tool(Tool::Edit);
                    self.capture(ctx, 0);
                    self.step = 4;
                }
                4 if self.received[0] => self.step = 5,
                5 => self.step = 6,
                6 => {
                    self.capture(ctx, 1);
                    self.step = 7;
                }
                7 if self.received[1] => self.step = 8,
                8 => {
                    self.capture(ctx, 2);
                    self.step = 9;
                }
                9 if self.received[2] => self.step = 10,
                10 => self.step = 11,
                11 => self.step = 12,
                12 => {
                    if self.view.node_count() != 1 {
                        self.errors.push("Draft was published".into());
                    }
                    self.edits += 1;
                    if self.edits == 300 {
                        self.step = 13;
                    }
                }
                13 => {
                    self.capture(ctx, 3);
                    self.step = 14;
                }
                14 if self.received[3] => {
                    self.view.set_tool(Tool::Interact);
                    self.view.pause(false, 0);
                    self.step = 15;
                }
                15 if state.phase == Phase::Playing => {
                    self.resume_us = Some(state.simulation_us);
                    self.paused_wall_ms =
                        self.paused_start.unwrap().elapsed().as_secs_f64() * 1000.;
                    self.phase_start = Instant::now();
                    self.step = 16;
                }
                16 => {
                    self.after_inputs = self.after_inputs.saturating_add(1);
                    if self.phase_start.elapsed() >= Duration::from_millis(450) {
                        self.finish(ctx);
                    }
                }
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
                5 | 11 => {
                    self.origin = self.view.target().center();
                    vec![
                        egui::Event::PointerMoved(self.origin),
                        pointer(self.origin, true),
                    ]
                }
                6 => vec![
                    egui::Event::PointerMoved(self.origin + egui::vec2(30., 20.)),
                    key(egui::Key::Z, true, true),
                ],
                8 => vec![key(egui::Key::Escape, true, false)],
                10 => vec![
                    key(egui::Key::Escape, false, false),
                    pointer(self.origin + egui::vec2(30., 20.), false),
                    key(egui::Key::X, true, false),
                ],
                12 => vec![
                    egui::Event::PointerMoved(
                        self.origin
                            + if self.edits == 299 {
                                egui::vec2(60., 30.)
                            } else {
                                egui::vec2(
                                    60. + (self.edits % 50) as f32,
                                    30. + (self.edits % 10) as f32,
                                )
                            },
                    ),
                    key(egui::Key::Z, true, true),
                ],
                13 => vec![pointer(self.origin + egui::vec2(60., 30.), false)],
                16 => match self.after_inputs {
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
                self.view.observe_audio(AudioObservation {
                    epoch: 0,
                    request: sample.request,
                    sequence: sample.sequence,
                    position_us: sample.position_us,
                    paused: sample.phase == DevicePhase::Paused,
                    fault: sample.phase == DevicePhase::Faulted,
                });
                let state = self.view.audio_state().unwrap();
                if sample.sequence != self.last_sequence
                    && state.phase != Phase::Pending
                    && sample.request == state.request
                {
                    self.samples.push(json!({"phase":format!("{:?}",sample.phase),"request":sample.request,"sequence":sample.sequence,"device_us":sample.position_us,"simulation_us":state.simulation_us,"observed_elapsed_us":sample.observed_elapsed_us}));
                    self.last_sequence = sample.sequence;
                }
                if state.phase == Phase::Faulted {
                    self.errors.push(format!(
                        "Output failure: {:?}; transport {:?}",
                        sample.error, state.fault
                    ));
                    self.finish(ctx);
                    return;
                }
            }
            while let Ok((tag, image)) = self.rx.try_recv() {
                let rgba: Vec<u8> = image.pixels.iter().flat_map(|p| p.to_array()).collect();
                let png =
                    image::RgbaImage::from_raw(image.size[0] as u32, image.size[1] as u32, rgba)
                        .unwrap();
                png.save(self.output.join(format!("stage-{tag}.png")))
                    .unwrap();
                let rect = self.view.canvas();
                let scale = ctx.pixels_per_point();
                let x = (rect.min.x * scale).ceil() as u32;
                let y = (rect.min.y * scale).ceil() as u32;
                let w = (rect.width() * scale).floor() as u32;
                let h = (rect.height() * scale).floor() as u32;
                self.crops.push(
                    image::imageops::crop_imm(
                        &png,
                        x,
                        y,
                        w.min(png.width() - x),
                        h.min(png.height() - y),
                    )
                    .to_image()
                    .into_raw(),
                );
                let state = self.view.audio_state().unwrap();
                self.captures.push(json!({"stage":tag,"simulation_us":state.simulation_us,"accepted_inputs":state.accepted_inputs,"nodes":self.view.node_count(),"position":self.view.position()}));
                match self.composition.verify(&self.output, tag as u16) {
                    Ok(proof) => self.pixels.push(proof),
                    Err(error) => self.errors.push(error),
                }
                self.received[tag] = true;
            }
            if self.start.elapsed() > Duration::from_secs(90) {
                self.errors.push("Transport proof timeout".into());
                self.finish(ctx);
                return;
            }
            ctx.request_repaint_after(Duration::from_millis(16));
        }
        fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
            self.view
                .show_composition(ui, 0, self.composition.texture_id);
            if let Err(error) = self.composition.refresh(self.view.position()) {
                self.errors.push(error);
            }
            if !self.finished {
                self.advance(ui.ctx());
            }
            if let Some(intent) = self.view.take_audio_intent()
                && let Err(error) = self.device.request(intent.request, intent.paused)
            {
                self.errors.push(format!("{error:?}"));
                self.view.fail_audio();
            }
        }
    }
    pub fn run() -> eframe::Result {
        let mut args = std::env::args().skip(1);
        let output = PathBuf::from(args.next().expect("new artifacts/p0 output directory"));
        let fixture = PathBuf::from(args.next().expect("standard fixture directory"));
        assert!(args.next().is_none());
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("artifacts/p0")
            .canonicalize()
            .unwrap();
        assert_eq!(
            output.parent().unwrap().canonicalize().unwrap(),
            root,
            "Output must be directly under artifacts/p0"
        );
        assert!(!output.exists(), "Preserve existing outputs");
        std::fs::create_dir(&output).unwrap();
        let mut setup = eframe::egui_wgpu::WgpuSetupCreateNew::without_display_handle();
        setup.instance_descriptor.backends = eframe::wgpu::Backends::VULKAN;
        let descriptor = setup.device_descriptor;
        setup.device_descriptor = Arc::new(move |adapter| {
            let mut d = descriptor(adapter);
            d.memory_hints = eframe::wgpu::MemoryHints::MemoryUsage;
            d
        });
        eframe::run_native(
            "P0 device-clock transport",
            eframe::NativeOptions {
                viewport: egui::ViewportBuilder::default()
                    .with_inner_size([1100., 720.])
                    .with_visible(false),
                renderer: eframe::Renderer::Wgpu,
                wgpu_options: eframe::egui_wgpu::WgpuConfiguration {
                    wgpu_setup: setup.into(),
                    ..Default::default()
                },
                persist_window: false,
                ..Default::default()
            },
            Box::new(move |cc| {
                Ok(Box::new(
                    App::new(cc, fixture, output).map_err(std::io::Error::other)?,
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
    eprintln!("Optional P0 device output proof requires Windows");
    std::process::exit(1);
}
