//! Opt-in generated PCM output with seek, frozen editing and content relocation.
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
    use p0_wasapi::pcm::{Packet, Stats};
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
        device: WindowsOutput,
        gpu: LiveComposition,
        output: PathBuf,
        start: Instant,
        since: Instant,
        step: u8,
        input_step: Option<u8>,
        origin: egui::Pos2,
        edits: usize,
        input_pass: usize,
        sent: bool,
        samples: Vec<Value>,
        proofs: Vec<Value>,
        checkpoints: Vec<Value>,
        errors: Vec<String>,
        bases: BTreeMap<u64, u64>,
        last: (u64, u64),
        frozen: Option<u64>,
        recovery_frozen: Option<u64>,
        resume: Option<u64>,
        pause_stats: Option<Stats>,
        pause_stable: bool,
        stale_rejected: usize,
        finished: bool,
    }
    fn packet(p: Option<Packet>) -> Value {
        p.map(|p| {
            json!({"source_frame":p.source_frame,"frames":p.frames,
        "nonzero_samples":p.nonzero_samples,"hash":p.hash,"peak":p.peak,"probe":p.probe})
        })
        .unwrap_or(Value::Null)
    }
    fn pcm(s: Option<Stats>) -> Value {
        s.map(|s|json!({"requested_us":s.requested_us,"source_origin_frame":s.source_origin_frame,
        "submitted_frames":s.submitted_frames,"packets":s.packets,"nonzero_samples":s.nonzero_samples,
        "first":packet(s.first),"last":packet(s.last)})).unwrap_or(Value::Null)
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
            let gpu = LiveComposition::new(cc, &fixture)?;
            let wake = cc.egui_ctx.clone();
            let device = WindowsOutput::spawn_pcm(move || wake.request_repaint())
                .map_err(|e| format!("{e:?}"))?;
            let mut view = GestureView::default();
            view.set_tool(Tool::Interact);
            Ok(Self {
                view,
                audio: Transport::default(),
                device,
                gpu,
                output,
                start: Instant::now(),
                since: Instant::now(),
                step: 0,
                input_step: None,
                origin: egui::Pos2::ZERO,
                edits: 0,
                input_pass: 0,
                sent: false,
                samples: vec![],
                proofs: vec![],
                checkpoints: vec![],
                errors: vec![],
                bases: BTreeMap::from([(0, 0)]),
                last: (u64::MAX, 0),
                frozen: None,
                recovery_frozen: None,
                resume: None,
                pause_stats: None,
                pause_stable: true,
                stale_rejected: 0,
                finished: false,
            })
        }
        fn control(&mut self, paused: bool) {
            let i = self.audio.intent(paused);
            if let Err(e) = self.device.control(i) {
                self.errors.push(format!("{e:?}"));
            }
        }
        fn relocate(&mut self, position_us: u64, recovery: bool) {
            let target = if recovery {
                self.audio.recover().unwrap()
            } else {
                self.audio
                    .seek(position_us)
                    .expect("seek only while paused")
            };
            self.bases.insert(target.epoch, position_us);
            if let Err(e) = self.device.relocate(target) {
                self.errors.push(format!("relocate {e:?}"));
                self.audio.fail(Fault::Backend);
            }
            // A late old fault must not kill the replacement session.
            if !self.audio.observe(AudioObservation {
                epoch: target.epoch - 1,
                request: 0,
                sequence: u64::MAX,
                position_us: u64::MAX,
                paused: false,
                fault: true,
            }) {
                self.stale_rejected += 1;
            }
        }
        fn paused_samples(&self, epoch: u64, request: u64) -> usize {
            self.samples
                .iter()
                .filter(|r| {
                    r["epoch"] == epoch && r["request"] == request && r["phase"] == "Paused"
                })
                .count()
        }
        fn proof(&mut self, tag: u16) {
            let s = self.audio.state();
            if !matches!(s.phase, Phase::Paused | Phase::Faulted) {
                self.errors.push("Pixel check during playback".into());
                return;
            }
            match self.gpu.verify(&self.output, tag) {
                Ok(p) => self.proofs.push(p),
                Err(e) => self.errors.push(e),
            }
            self.checkpoints.push(json!({"stage":tag,"epoch":s.epoch,"simulation_us":s.simulation_us,"nodes":self.view.node_count(),
                "accepted_inputs":s.accepted_inputs,"pcm":pcm(self.device.0.snapshot().pcm)}));
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
                    self.control(true);
                    self.step = 3;
                }
                3 if s.phase == Phase::Paused && self.paused_samples(0, 2) >= 3 => {
                    self.frozen = Some(s.simulation_us);
                    self.pause_stats = self.device.0.snapshot().pcm;
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
                    self.pause_stable = self
                        .device
                        .0
                        .snapshot()
                        .pcm
                        .zip(self.pause_stats)
                        .is_some_and(|(a, b)| {
                            a.submitted_frames == b.submitted_frames && a.packets == b.packets
                        });
                    self.view.set_tool(Tool::Interact);
                    self.control(false);
                    self.step = 8;
                }
                8 if s.phase == Phase::Playing => {
                    self.resume = Some(s.simulation_us);
                    self.since = Instant::now();
                    self.input_pass = 0;
                    self.step = 9;
                }
                9 => {
                    self.input_pass += 1;
                    if self.input_pass >= 4 && self.since.elapsed() >= Duration::from_millis(350) {
                        self.control(true);
                        self.step = 10;
                    }
                }
                10 if s.phase == Phase::Paused && self.paused_samples(0, 4) >= 3 => {
                    self.relocate(1_050_000, false);
                    self.step = 11;
                }
                11 if s.epoch == 1
                    && s.phase == Phase::Paused
                    && self.paused_samples(1, 0) >= 3 =>
                {
                    self.proof(2);
                    self.control(false);
                    self.step = 12;
                }
                12 if s.phase == Phase::Playing => {
                    self.since = Instant::now();
                    self.step = 13;
                }
                13 if self.since.elapsed() >= Duration::from_millis(350) => {
                    self.control(true);
                    self.step = 14;
                }
                14 if s.phase == Phase::Paused && self.paused_samples(1, 2) >= 3 => {
                    self.relocate(50_000, false);
                    self.step = 15;
                }
                15 if s.epoch == 2
                    && s.phase == Phase::Paused
                    && self.paused_samples(2, 0) >= 3 =>
                {
                    self.control(false);
                    self.step = 16;
                }
                16 if s.phase == Phase::Playing => {
                    self.since = Instant::now();
                    self.step = 17;
                }
                17 if self.since.elapsed() >= Duration::from_millis(350) => {
                    if let Err(e) = self.device.0.invalidate(s.epoch) {
                        self.errors.push(format!("{e:?}"));
                    }
                    self.step = 18;
                }
                18 if s.phase == Phase::Faulted => {
                    self.recovery_frozen = Some(s.simulation_us);
                    self.relocate(s.simulation_us, true);
                    self.step = 19;
                }
                19 if s.epoch == 3
                    && s.phase == Phase::Paused
                    && self.paused_samples(3, 0) >= 3 =>
                {
                    self.proof(3);
                    self.control(false);
                    self.step = 20;
                }
                20 if s.phase == Phase::Playing => {
                    self.since = Instant::now();
                    self.step = 21;
                }
                21 if self.since.elapsed() >= Duration::from_millis(350) => {
                    self.control(true);
                    self.step = 22;
                }
                22 if s.phase == Phase::Paused && self.paused_samples(3, 2) >= 3 => {
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
            let native = self.device.0.snapshot();
            let epochs: Vec<_> = (0..=3)
                .filter_map(|epoch| {
                    self.samples
                        .iter()
                        .rev()
                        .find(|r| r["epoch"] == epoch)
                        .cloned()
                })
                .collect();
            let checks = json!({"four_content_epochs":epochs.len()==4&&s.epoch==3,"all_epochs_nonzero_pcm":epochs.iter().all(|r|r["pcm"]["nonzero_samples"].as_u64().is_some_and(|n|n>0)),
                "paused_submission_frozen":self.pause_stable,"fresh_input_only":s.accepted_inputs==2,"thirty_updates_one_commit":self.edits==30&&self.view.node_count()==2,
                "four_pixel_proofs":self.proofs.len()==4,"old_epochs_rejected":self.stale_rejected==3,
                "normal_resume_no_catchup":self.frozen.zip(self.resume).is_some_and(|(f,r)|r>=f&&r-f<=50000),
                "clock_from_device":self.samples.iter().all(|r|r["simulation_us"].as_u64()==r["base_us"].as_u64().zip(r["device_us"].as_u64()).and_then(|(a,b)|a.checked_add(b))),
                "final_paused":s.phase==Phase::Paused&&self.paused_samples(3,2)>=3,"no_error":self.errors.is_empty()});
            let passed = checks.as_object().unwrap().values().all(|v| v == true);
            let format=native.format.map(|f|json!({"rate":f.rate,"channels":f.channels,"bits":f.bits,"buffer_frames":f.buffer_frames,"clock_frequency":f.clock_frequency}));
            let result = json!({"schema_version":1,"status":if passed{"passed"}else{"failed"},"checks":checks,"errors":self.errors,"audio_format":format,
                "frozen_us":self.frozen,"normal_resume_us":self.resume,"recovery_frozen_us":self.recovery_frozen,"final_simulation_us":s.simulation_us,
                "accepted_inputs":s.accepted_inputs,"edit_count":self.edits,"nodes":self.view.node_count(),"stale_rejected":self.stale_rejected,"fresh_samples":self.samples,"epoch_last_samples":epochs,
                "checkpoints":self.checkpoints,"proofs":self.proofs,"composition":self.gpu.metrics(),"system_capture_used":false,"microphone_or_loopback_used":false,
                "human_audibility":"unconfirmed","pcm_source":"generated six-second mono tone pattern duplicated into validated float32 output channels, peak <= 0.04",
                "limits":["WASAPI accepted initialized nonzero PCM; physical endpoint/hearing and acoustic latency need separate evidence",
                    "100 ms polling shared buffer, no low latency, underrun or physical input acceptance",
                    "Explicit retire/reopen/seek, not physical unplug; starts paused; recovery may replay audio beyond last UI-confirmed device position",
                    "Seek positions quantized down to one sample; no audio-file decoding, resampling, mixing, chart parsing or stable judgement",
                    "Pixel references block only while paused; not full resource or frame-budget measurements"]});
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
                9 => match self.input_pass {
                    0 => vec![key(true, true)],
                    1 => vec![key(false, false)],
                    2 => vec![key(true, false)],
                    _ => vec![],
                },
                13 | 17 | 21 => vec![key(true, true)],
                _ => vec![],
            };
        }
        fn logic(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
            if self.finished {
                return;
            }
            let native = self.device.0.snapshot();
            if let Some(o) = WindowsOutput::observation(native) {
                let accepted = self.audio.observe(o);
                let s = self.audio.state();
                if accepted && !o.fault && (o.epoch, o.sequence) != self.last {
                    if let Some(base) = self.bases.get(&o.epoch) {
                        self.samples.push(json!({"epoch":o.epoch,"sequence":o.sequence,"request":o.request,
                        "phase":format!("{:?}",s.phase),"device_us":o.position_us,"base_us":base,"simulation_us":s.simulation_us,"pcm":pcm(native.pcm)}));
                    }
                    self.last = (o.epoch, o.sequence);
                }
                let expected = self.step == 18
                    && o.fault
                    && s.fault == Some(Fault::Backend)
                    && native
                        .error
                        .is_some_and(|e| e.operation == "explicit stream invalidation");
                if s.phase == Phase::Faulted && !expected {
                    self.errors.push(format!(
                        "Unexpected {:?}, native {:?}",
                        s.fault, native.error
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
            self.view
                .show_transport(ui, Some(self.gpu.texture_id), &mut self.audio, false);
            if let Err(e) = self.gpu.refresh(self.view.position()) {
                self.errors.push(e);
            }
            if let Some(i) = self.view.take_audio_intent()
                && let Err(e) = self.device.control(i)
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
            "P0 generated PCM transport",
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
    eprintln!("PCM device proof currently requires Windows");
    std::process::exit(1);
}
