//! Opt-in native background composition handoff; own renderer, no client/capture APIs.
use eframe::{egui, wgpu};
use p0_egui::{composition_switch::DocumentSwitch, interaction_view::GestureView};
use serde_json::{Value, json};
use std::{
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};
struct App {
    switch: DocumentSwitch,
    views: [GestureView; 10],
    root: PathBuf,
    output: PathBuf,
    pixels: bool,
    observe: bool,
    start: Instant,
    phase_start: Instant,
    step: u8,
    cycle: usize,
    requested: Instant,
    latencies: Vec<f64>,
    request_cpu: Vec<f64>,
    frame_cpu: Vec<f64>,
    proofs: Vec<Value>,
    errors: Vec<String>,
    last_frame: Option<u64>,
    idle_frames: usize,
    closed_frames: usize,
    retained_frames: usize,
    failed_preserved: bool,
    cancel_preserved: bool,
    finished: bool,
    trace: Option<Vec<Value>>,
    first_document_ms: Option<f64>,
}
impl App {
    fn new(
        cc: &eframe::CreationContext<'_>,
        root: PathBuf,
        output: PathBuf,
        pixels: bool,
        observe: bool,
        profile: bool,
        warmup: bool,
    ) -> Self {
        let mut app = Self {
            switch: DocumentSwitch::new(cc.wgpu_render_state.clone().expect("wgpu"), &cc.egui_ctx),
            views: std::array::from_fn(|_| GestureView::default()),
            root,
            output,
            pixels,
            observe,
            start: Instant::now(),
            phase_start: Instant::now(),
            step: 0,
            cycle: 0,
            requested: Instant::now(),
            latencies: vec![],
            request_cpu: vec![],
            frame_cpu: vec![],
            proofs: vec![],
            errors: vec![],
            last_frame: None,
            idle_frames: 0,
            closed_frames: 0,
            retained_frames: 0,
            failed_preserved: false,
            cancel_preserved: false,
            finished: false,
            trace: profile.then(Vec::new),
            first_document_ms: None,
        };
        if profile {
            app.switch.profile = Some(Vec::new());
        }
        if warmup {
            app.switch.warm_pipelines(&cc.egui_ctx);
        }
        app.phase("Loading");
        app.request(0, false);
        app
    }
    fn phase(&mut self, name: &str) {
        self.phase_start = Instant::now();
        std::fs::write(
            self.output.join("phase.json"),
            json!({"phase":name}).to_string(),
        )
        .unwrap();
    }
    fn request(&mut self, doc: usize, missing: bool) {
        if let Some((old, _)) = self.switch.active() {
            self.views[old].cancel();
        }
        self.requested = Instant::now();
        let root = if missing {
            self.root.join("missing-document-fixture")
        } else {
            self.root.clone()
        };
        if let Err(e) = self.switch.request(root, doc) {
            self.errors.push(e);
        }
        self.request_cpu
            .push(self.requested.elapsed().as_secs_f64() * 1000.);
    }
    fn check_pixels(&mut self, tag: u16) {
        if self.pixels
            && let Some((doc, view)) = self.switch.active()
        {
            match view.verify(&self.output, tag) {
                Ok(mut proof) => {
                    proof["document"] = json!(doc);
                    self.proofs.push(proof);
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
        let checks = json!({"stale_gpu_discarded":self.switch.stale_gpu>=1,"failed_keeps_old":self.failed_preserved,
            "cancel_keeps_old":self.cancel_preserved,"twenty_switches":self.latencies.len()==20,
            "old_view_retained":self.retained_frames>0,"all_views_paused":self.views.iter().all(|v|v.simulation().simulation_ms==0&&v.simulation().accepted_game_inputs==0&&v.node_count()==1),
            "closed_without_active_or_pending":self.switch.active().is_none()&&!self.switch.pending(),
            "bounded_source_upload":!self.switch.uploads.is_empty()&&self.switch.uploads.iter().all(|v|v["uploaded_bytes"]==40*1024*1024&&v["upload_frames"]==5&&v["max_source_bytes_per_frame"].as_u64().unwrap()<=8*1024*1024&&v["max_sources_per_frame"].as_u64().unwrap()<=2),
            "pixel_proofs":!self.pixels||self.proofs.len()==4,"no_errors":self.errors.is_empty()});
        let passed = checks.as_object().unwrap().values().all(|v| v == true);
        let result = json!({"schema_version":1,"status":if passed{"passed"}else{"failed"},"checks":checks,"errors":self.errors,
            "pixel_diagnostic":self.pixels,"long_observation":self.observe,"pipeline_warmup_ms":self.switch.pipeline_warmup_ms,"first_document_wall_ms":self.first_document_ms,"switch_ms":self.latencies,"request_cpu_ms":self.request_cpu,"eframe_cpu_frame_ms_loading":self.frame_cpu,
            "stale_reads":self.switch.stale_reads,"stale_gpu":self.switch.stale_gpu,"accepted_documents":self.switch.accepted,"uploads":self.switch.uploads,
            "retained_frames":self.retained_frames,"idle_ui_frames":self.idle_frames,"closed_ui_frames":self.closed_frames,"proofs":self.proofs,
            "simulation_ms":0,"accepted_game_inputs":0,"system_capture_used":false,
            "limits":["Read-only standard fixture; synthetic paused editor state, no actual gameplay or physical input",
                "Source byte writes bounded per egui frame; allocation/zero-initialization, operations, display work, staging and driver retention are separate costs",
                "At most active, GPU candidate and one newer worker result CPU sets (120 MiB original payload), plus decoder/upload temporaries",
                "Missing-resource check uses a missing fixture directory; corrupt/individual missing PNGs are headless cases",
                "Only long non-pixel runs are idle/close observations; pixel diagnostics include blocking CPU references"]});
        std::fs::write(
            self.output.join("result.json"),
            serde_json::to_vec_pretty(&result).unwrap(),
        )
        .unwrap();
        if let Some(rows) = &self.trace {
            std::fs::write(
                self.output.join("frame-trace.json"),
                serde_json::to_vec_pretty(
                    &json!({"schema_version":1,"frames":rows,"switch_steps":self.switch.profile}),
                )
                .unwrap(),
            )
            .unwrap();
        }
        self.phase("Finished");
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }
}
impl eframe::App for App {
    fn logic(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if self.finished {
            return;
        }
        if self.start.elapsed() > Duration::from_secs(200) {
            self.errors.push("Timeout".into());
            self.finish(ctx);
            return;
        }
        let trace_start = self.trace.as_ref().map(|_| Instant::now());
        if self.step < 7
            && !self.pixels
            && let Some(seconds) = frame.info().cpu_usage
        {
            self.frame_cpu.push((seconds * 1000.) as f64);
        }
        if let Err(e) = self.switch.poll(ctx.cumulative_frame_nr(), ctx) {
            self.errors.push(e);
            self.finish(ctx);
            return;
        }
        let active = self.switch.active().map(|(d, _)| d);
        match self.step {
            0 if active == Some(0) && !self.switch.pending() => {
                self.first_document_ms = Some(self.start.elapsed().as_secs_f64() * 1000.);
                self.check_pixels(0);
                self.request(1, false);
                self.step = 1;
            }
            1 if self.switch.preparing() => {
                if active != Some(0) {
                    self.errors.push("Old view replaced too early".into());
                }
                self.request(2, false);
                self.step = 2;
            }
            2 => {
                if active == Some(0) {
                    self.retained_frames += 1;
                } else if active != Some(2) {
                    self.errors
                        .push("Obsolete GPU candidate became visible".into());
                }
                if active == Some(2) && !self.switch.pending() {
                    self.check_pixels(1);
                    self.request(3, false);
                    self.switch.cancel().unwrap();
                    self.step = 3;
                }
            }
            3 => {
                self.cancel_preserved = active == Some(2);
                self.request(4, true);
                self.step = 4;
            }
            4 if self.switch.error.is_some() => {
                self.failed_preserved = active == Some(2);
                self.check_pixels(3);
                self.switch.error = None;
                self.phase("Cycle");
                self.request(0, false);
                self.step = 5;
            }
            5 if !self.switch.pending() => {
                if active != Some(self.cycle % 10) {
                    self.errors.push("Wrong document accepted".into());
                }
                self.latencies
                    .push(self.requested.elapsed().as_secs_f64() * 1000.);
                self.cycle += 1;
                if self.cycle == 20 {
                    self.check_pixels(2);
                    self.phase("IdleSettle");
                    self.step = 6;
                } else {
                    self.request(self.cycle % 10, false);
                }
            }
            6 if self.phase_start.elapsed() >= Duration::from_secs(3) => {
                self.phase("Idle");
                self.step = 7;
            }
            7 if self.phase_start.elapsed()
                >= Duration::from_secs(if self.observe { 60 } else { 1 }) =>
            {
                self.switch.close().unwrap();
                self.phase("ClosedSettle");
                self.step = 8;
            }
            8 if self.phase_start.elapsed() >= Duration::from_secs(3) => {
                self.phase("Closed");
                self.step = 9;
            }
            9 if self.phase_start.elapsed()
                >= Duration::from_secs(if self.observe { 60 } else { 1 }) =>
            {
                self.finish(ctx)
            }
            _ => {}
        }
        let delay = match self.step {
            6 | 8 => Duration::from_secs(3).saturating_sub(self.phase_start.elapsed()),
            7 | 9 => Duration::from_secs(if self.observe { 60 } else { 1 })
                .saturating_sub(self.phase_start.elapsed()),
            _ => Duration::from_millis(16),
        };
        ctx.request_repaint_after(delay);
        if let Some(s) = trace_start {
            self.trace.as_mut().unwrap().push(json!({"frame":ctx.cumulative_frame_nr(),"step":self.step,"logic_ms":s.elapsed().as_secs_f64()*1000.,"previous_eframe_cpu_ms":frame.info().cpu_usage.map(|s|s as f64*1000.)}));
        }
    }
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let number = ui.ctx().cumulative_frame_nr();
        if self.last_frame != Some(number) {
            if self.step == 7 {
                self.idle_frames += 1;
            }
            if self.step == 9 {
                self.closed_frames += 1;
            }
            self.last_frame = Some(number);
        }
        let start = self.trace.as_ref().map(|_| Instant::now());
        if let Some((doc, view)) = self.switch.active_mut() {
            self.views[doc].show_composition(ui, 0, view.texture_id);
            if let Err(e) = view.refresh(self.views[doc].position()) {
                self.errors.push(e);
            }
        } else {
            egui::CentralPanel::default().show(ui, |ui| {
                ui.label("P0 background document proof / no active document");
            });
        }
        if let Some(s) = start {
            self.trace.as_mut().unwrap().last_mut().unwrap()["ui_ms"] =
                json!(s.elapsed().as_secs_f64() * 1000.);
        }
    }
}
fn main() -> eframe::Result {
    let mut args = std::env::args().skip(1);
    let output = PathBuf::from(args.next().expect("new output directory"));
    let root = PathBuf::from(args.next().expect("fixture"));
    let flags: Vec<_> = args.collect();
    assert!(
        flags
            .iter()
            .all(|f| f == "--pixels" || f == "--observe" || f == "--profile" || f == "--warmup")
    );
    let warmup = flags.iter().any(|f| f == "--warmup");
    let profile = flags.iter().any(|f| f == "--profile");
    let pixels = flags.iter().any(|f| f == "--pixels");
    let observe = flags.iter().any(|f| f == "--observe");
    assert!(!(pixels && observe));
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
        "P0 background composition",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1100., 720.])
                .with_visible(false),
            renderer: eframe::Renderer::Wgpu,
            persist_window: false,
            wgpu_options: eframe::egui_wgpu::WgpuConfiguration {
                wgpu_setup: setup.into(),
                surface: eframe::egui_wgpu::SurfaceConfig {
                    present_mode: wgpu::PresentMode::AutoNoVsync,
                    ..eframe::egui_wgpu::SurfaceConfig::LOW_LATENCY
                },
                ..Default::default()
            },
            ..Default::default()
        },
        Box::new(move |cc| {
            Ok(Box::new(App::new(
                cc, root, output, pixels, observe, profile, warmup,
            )))
        }),
    )
}
