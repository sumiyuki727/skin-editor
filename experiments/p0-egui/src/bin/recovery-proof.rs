//! Controlled retirement of our own silent stream, not a physical unplug test.
#[cfg(windows)]
mod native {
    use eframe::{egui, wgpu};
    use p0_egui::{
        audio_output::Output,
        audio_transport::{AudioObservation, Fault, Phase, Transport},
        composition_view::LiveComposition,
        interaction_view::GestureView,
        windows_audio::WindowsOutput,
    };
    use p0_preview_contract::Tool;
    use serde_json::{Value, json};
    use std::{
        collections::BTreeMap,
        path::PathBuf,
        sync::Arc,
        time::{Duration, Instant},
    };

    struct App {
        view: GestureView,
        audio: Transport,
        output_device: WindowsOutput,
        composition: LiveComposition,
        output: PathBuf,
        start: Instant,
        since: Instant,
        step: u8,
        input_step: Option<u8>,
        origin: egui::Pos2,
        edits: usize,
        input_pass: usize,
        sent: bool,
        last_sample: (u64, u64),
        samples: Vec<Value>,
        checkpoints: Vec<Value>,
        proofs: Vec<Value>,
        anchors: BTreeMap<u64, (u64, u64)>,
        frozen: Vec<u64>,
        recovered: Vec<u64>,
        stale_rejected: usize,
        errors: Vec<String>,
        finished: bool,
    }
    fn key(pressed: bool, repeat: bool) -> egui::Event {
        egui::Event::Key {
            key: egui::Key::Z,
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
            let output_device = WindowsOutput::spawn(move || wake.request_repaint())
                .map_err(|e| format!("{e:?}"))?;
            let mut view = GestureView::default();
            view.set_tool(Tool::Interact);
            Ok(Self {
                view,
                audio: Transport::default(),
                output_device,
                composition,
                output,
                start: Instant::now(),
                since: Instant::now(),
                step: 0,
                input_step: None,
                origin: egui::Pos2::ZERO,
                edits: 0,
                input_pass: 0,
                sent: false,
                last_sample: (u64::MAX, 0),
                samples: vec![],
                checkpoints: vec![],
                proofs: vec![],
                anchors: BTreeMap::from([(0, (0, 0))]),
                frozen: vec![],
                recovered: vec![],
                stale_rejected: 0,
                errors: vec![],
                finished: false,
            })
        }
        fn control(&mut self, paused: bool) {
            let intent = self.audio.intent(paused);
            if let Err(e) = self.output_device.control(intent) {
                self.errors.push(format!("control: {e:?}"));
            }
        }
        fn invalidate(&mut self) {
            if let Err(e) = self.output_device.0.invalidate(self.audio.state().epoch) {
                self.errors.push(format!("invalidate: {e:?}"));
            }
        }
        fn recover(&mut self) {
            let target = self
                .audio
                .recover()
                .expect("explicit recovery only from fault");
            let epoch = target.epoch;
            if let Err(e) = self.output_device.relocate(target) {
                self.errors.push(format!("rebuild: {e:?}"));
            }
            // An old stream can have a numerically larger position/sequence or fault.
            let old = AudioObservation {
                epoch: epoch - 1,
                request: 0,
                sequence: u64::MAX,
                position_us: u64::MAX,
                paused: false,
                fault: true,
            };
            if !self.audio.observe(old) {
                self.stale_rejected += 1;
            }
        }
        fn proof(&mut self, tag: u16) {
            let s = self.audio.state();
            if !matches!(s.phase, Phase::Paused | Phase::Faulted) {
                self.errors.push("Pixel check during playback".into());
                return;
            }
            match self.composition.verify(&self.output, tag) {
                Ok(p) => self.proofs.push(p),
                Err(e) => self.errors.push(e),
            }
            self.checkpoints.push(json!({"stage":tag,"epoch":s.epoch,"phase":format!("{:?}",s.phase),
                "simulation_us":s.simulation_us,"nodes":self.view.node_count(),"inputs":s.accepted_inputs,"position":self.view.position()}));
        }
        fn paused_samples(&self, epoch: u64, request: u64) -> usize {
            self.samples
                .iter()
                .filter(|s| {
                    s["epoch"] == epoch && s["request"] == request && s["phase"] == "Paused"
                })
                .count()
        }
        fn advance(&mut self, ctx: &egui::Context) {
            if self.input_step.take() != Some(self.step) {
                return;
            }
            let s = self.audio.state();
            match self.step {
                0 if s.phase == Phase::Paused => {
                    self.control(false);
                    self.step = 1;
                }
                1 if s.phase == Phase::Playing => {
                    self.since = Instant::now();
                    self.step = 2;
                }
                2 if self.sent && self.since.elapsed() >= Duration::from_millis(350) => {
                    self.invalidate();
                    self.step = 3;
                }
                3 if s.phase == Phase::Faulted => {
                    self.frozen.push(s.simulation_us);
                    self.since = Instant::now();
                    self.view.set_tool(Tool::Edit);
                    self.proof(0);
                    self.step = 4;
                }
                4 => self.step = 5,
                5 => {
                    self.edits += 1;
                    if self.edits == 30 {
                        self.step = 6;
                    }
                }
                6 => self.step = 7,
                7 => {
                    self.proof(1);
                    if s.simulation_us != self.frozen[0] {
                        self.errors.push("Time changed while faulted".into());
                    }
                    self.recover();
                    self.step = 8;
                }
                8 if s.epoch == 1 && s.phase == Phase::Paused && self.paused_samples(1, 0) >= 3 => {
                    self.recovered.push(s.simulation_us);
                    self.proof(2);
                    self.view.set_tool(Tool::Interact);
                    self.control(false);
                    self.step = 9;
                }
                9 if s.phase == Phase::Playing => {
                    self.since = Instant::now();
                    self.input_pass = 0;
                    self.step = 10;
                }
                10 => {
                    self.input_pass += 1;
                    if self.input_pass >= 4 && self.since.elapsed() >= Duration::from_millis(350) {
                        self.invalidate();
                        self.step = 11;
                    }
                }
                11 if s.phase == Phase::Faulted => {
                    self.frozen.push(s.simulation_us);
                    self.recover();
                    self.step = 12;
                }
                12 if s.epoch == 2
                    && s.phase == Phase::Paused
                    && self.paused_samples(2, 0) >= 3 =>
                {
                    self.recovered.push(s.simulation_us);
                    self.proof(3);
                    self.control(false);
                    self.step = 13;
                }
                13 if s.phase == Phase::Playing => {
                    self.since = Instant::now();
                    self.step = 14;
                }
                14 if self.since.elapsed() >= Duration::from_millis(350) => {
                    self.control(true);
                    self.step = 15;
                }
                15 if s.phase == Phase::Paused && self.paused_samples(2, 2) >= 3 => {
                    self.finish(ctx)
                }
                _ => {}
            }
        }
        fn finish(&mut self, ctx: &egui::Context) {
            if self.finished {
                return;
            }
            self.finished = true;
            let s = self.audio.state();
            let native = self.output_device.0.snapshot();
            let equations = self.samples.iter().all(|r| {
                r["simulation_us"].as_u64()
                    == r["base_us"]
                        .as_u64()
                        .zip(
                            r["device_us"]
                                .as_u64()
                                .zip(r["origin_us"].as_u64())
                                .and_then(|(d, o)| d.checked_sub(o)),
                        )
                        .and_then(|(b, d)| b.checked_add(d))
            });
            let checks = json!({"two_recoveries":s.epoch==2&&self.frozen.len()==2&&self.recovered==self.frozen,
                "clock_progress_each_stream":self.frozen.first().is_some_and(|f|*f>=200000)&&self.frozen.get(1).zip(self.frozen.first()).is_some_and(|(b,a)|*b>=*a+200000)&&self.frozen.last().is_some_and(|f|s.simulation_us>=*f+200000),
                "recovered_paused_samples":self.paused_samples(1,0)>=3&&self.paused_samples(2,0)>=3,"old_epoch_faults_ignored":self.stale_rejected==2,
                "anchored_clock":equations,"fresh_input_only":s.accepted_inputs==2,"fault_edit_one_commit":self.edits==30&&self.view.node_count()==2,
                "four_pixel_checks":self.proofs.len()==4,"final_pause":s.phase==Phase::Paused&&native.epoch==2&&native.phase==p0_wasapi::Phase::Paused,
                "no_unexpected_error":self.errors.is_empty()});
            let passed = checks.as_object().unwrap().values().all(|v| v == true);
            let result = json!({"schema_version":1,"status":if passed{"passed"}else{"failed"},"checks":checks,"errors":self.errors,
                "fault_source":"explicit retirement of this process's silent stream; not a physical disconnect or injected driver HRESULT",
                "frozen_us":self.frozen,"recovered_us":self.recovered,"final_simulation_us":s.simulation_us,"accepted_inputs":s.accepted_inputs,
                "edit_count":self.edits,"nodes":self.view.node_count(),"fresh_samples":self.samples,"checkpoints":self.checkpoints,"proofs":self.proofs,
                "stale_rejected":self.stale_rejected,"composition":self.composition.metrics(),"system_capture_used":false,
                "limits":["Silent shared default output, synthetic egui input, 100 ms buffer/polling; no acoustic latency or endpoint identity claim",
                    "Actual stream retirement/reopen on dedicated worker; physical unplug, default-device changes and driver-blocking shutdown not verified",
                    "Failed open/runtime error paths use headless fake streams; no system settings or client files modified; real PCM repositioning not implemented",
                    "Blocking CPU pixel references run while faulted or paused; no frame or resource-budget claim; no stable gameplay"]});
            std::fs::write(
                self.output.join("result.json"),
                serde_json::to_vec_pretty(&result).unwrap(),
            )
            .unwrap();
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
    impl eframe::App for App {
        fn raw_input_hook(&mut self, _: &egui::Context, raw: &mut egui::RawInput) {
            raw.events.clear();
            raw.focused = true;
            self.input_step = Some(self.step);
            raw.events = match self.step {
                2 if !self.sent => {
                    self.sent = true;
                    vec![key(true, false)]
                }
                4 => {
                    self.origin = self.view.target().center();
                    vec![
                        egui::Event::PointerMoved(self.origin),
                        pointer(self.origin, true),
                    ]
                }
                5 => vec![
                    egui::Event::PointerMoved(
                        self.origin + egui::vec2(30. + self.edits as f32, 20.),
                    ),
                    key(true, true),
                ],
                6 => vec![pointer(self.origin + egui::vec2(60., 30.), false)],
                10 => match self.input_pass {
                    0 => vec![key(true, true)],
                    1 => vec![key(false, false)],
                    2 => vec![key(true, false)],
                    _ => vec![],
                },
                14 => vec![key(true, true)],
                _ => vec![],
            };
        }
        fn logic(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
            if self.finished {
                return;
            }
            if let Some(o) = self.output_device.snapshot() {
                let accepted = self.audio.observe(o);
                let s = self.audio.state();
                if accepted && !o.fault && (o.epoch, o.sequence) != self.last_sample {
                    let (origin, base) = *self
                        .anchors
                        .entry(o.epoch)
                        .or_insert((o.position_us, s.simulation_us));
                    self.samples.push(json!({"epoch":o.epoch,"sequence":o.sequence,"request":o.request,"phase":format!("{:?}",s.phase),
                        "device_us":o.position_us,"origin_us":origin,"base_us":base,"simulation_us":s.simulation_us}));
                    self.last_sample = (o.epoch, o.sequence);
                }
                let expected_fault = matches!(self.step, 3..=7 | 11)
                    && o.fault
                    && s.fault == Some(Fault::Backend)
                    && self
                        .output_device
                        .0
                        .snapshot()
                        .error
                        .is_some_and(|e| e.operation == "explicit stream invalidation");
                if s.phase == Phase::Faulted && !expected_fault {
                    self.errors.push(format!(
                        "Unexpected fault {:?}, native {:?}",
                        s.fault,
                        self.output_device.0.snapshot().error
                    ));
                    self.finish(ctx);
                    return;
                }
            }
            if self.start.elapsed() > Duration::from_secs(60) {
                self.errors.push(format!("Timeout at {}", self.step));
                self.finish(ctx);
                return;
            }
            ctx.request_repaint_after(Duration::from_millis(16));
        }
        fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
            if self.finished {
                return;
            }
            self.view.show_transport(
                ui,
                Some(self.composition.texture_id),
                &mut self.audio,
                false,
            );
            if let Err(e) = self.composition.refresh(self.view.position()) {
                self.errors.push(e);
            }
            if let Some(i) = self.view.take_audio_intent()
                && let Err(e) = self.output_device.control(i)
            {
                self.errors.push(format!("{e:?}"));
            }
            self.advance(ui.ctx());
        }
    }
    pub fn run() -> eframe::Result {
        let mut args = std::env::args().skip(1);
        let output = PathBuf::from(args.next().expect("new output"));
        let fixture = PathBuf::from(args.next().expect("fixture"));
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
            "P0 output recovery",
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
    eprintln!("Real output recovery currently requires Windows");
    std::process::exit(1);
}
