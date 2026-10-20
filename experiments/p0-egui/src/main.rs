//! Independently authored native candidate experiment, not the product editor.
//! Hidden self-tests read back this app's own render target, never the desktop.

use eframe::egui;
use p0_preview_contract::{Probe, Scene, Tool};
use serde_json::json;
use std::{
    path::PathBuf,
    sync::mpsc,
    time::{Duration, Instant},
};

struct Candidate {
    probe: Probe,
    started: Instant,
    output: PathBuf,
    automatic: bool,
    phase: u8,
    frozen_ms: Option<u64>,
    texture: Option<egui::TextureHandle>,
    samples: Vec<f64>,
    frames: u64,
    zoom: f32,
    canvas: egui::Rect,
    before_pixels: Option<Vec<u8>>,
    captures: Vec<String>,
    adapter: serde_json::Value,
    tx: mpsc::Sender<(u8, std::sync::Arc<egui::ColorImage>)>,
    rx: mpsc::Receiver<(u8, std::sync::Arc<egui::ColorImage>)>,
    completed: bool,
}

impl Candidate {
    fn new(cc: &eframe::CreationContext<'_>, output: PathBuf, automatic: bool) -> Self {
        let (tx, rx) = mpsc::channel();
        let adapter = cc.wgpu_render_state.as_ref().map_or(json!(null), |state| {
            let info = state.adapter.get_info();
            json!({ "name": info.name, "backend": format!("{:?}", info.backend),
                "device_type": format!("{:?}", info.device_type), "driver": info.driver })
        });
        Self {
            probe: Probe::new(0),
            started: Instant::now(),
            output,
            automatic,
            phase: 0,
            frozen_ms: None,
            texture: None,
            samples: Vec::new(),
            frames: 0,
            zoom: 1.0,
            canvas: egui::Rect::NOTHING,
            before_pixels: None,
            captures: Vec::new(),
            adapter,
            tx,
            rx,
            completed: false,
        }
    }

    fn capture(&self, ctx: &egui::Context, stage: u8) {
        let sender = self.tx.clone();
        let wake = ctx.clone();
        ctx.request_screenshot(move |pixels| {
            let _ = sender.send((stage, pixels));
            wake.request_repaint();
        });
    }

    fn finish(&mut self, ctx: &egui::Context, error: Option<String>) {
        if self.completed {
            return;
        }
        self.completed = true;
        let mut sorted = self.samples.clone();
        sorted.sort_by(f64::total_cmp);
        let p95 = if sorted.is_empty() {
            None
        } else {
            Some(sorted[((sorted.len() - 1) * 95) / 100])
        };
        let observation = self.probe.observe();
        let result = json!({
            "schema_version": 1, "experiment": "p0-egui", "eframe_version": "0.37.0",
            "status": if error.is_none() { "passed" } else { "failed" }, "error": error,
            "adapter": self.adapter, "ui_frames": self.frames,
            "ui_cpu_ms_p95": p95, "cpu_sample_count": self.samples.len(),
            "wall_seconds": self.started.elapsed().as_secs_f64(),
            "frozen_simulation_ms": self.frozen_ms,
            "final_simulation_ms": observation.simulation_ms,
            "content_revision": observation.content_revision,
            "render_target_readbacks": self.captures,
            "system_window_capture_used": false,
            "workload": "50 synthetic textured quads using one 32x32 source; not the standard project benchmark",
            "not_tested": ["stable compatibility", "real audio", "physical input", "IME", "full project memory budget", "macOS", "Linux"]
        });
        if let Err(error) = std::fs::write(self.output.join("result.json"), result.to_string()) {
            eprintln!("Cannot write experiment result: {error}");
        }
        println!("{result}");
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }

    fn receive_captures(&mut self, ctx: &egui::Context) {
        while let Ok((stage, pixels)) = self.rx.try_recv() {
            let rgba: Vec<u8> = pixels
                .pixels
                .iter()
                .flat_map(egui::Color32::to_array)
                .collect();
            let name = format!("render-{stage}.png");
            if let Err(error) = image::save_buffer(
                self.output.join(&name),
                &rgba,
                pixels.size[0] as u32,
                pixels.size[1] as u32,
                image::ColorType::Rgba8,
            ) {
                self.finish(ctx, Some(format!("PNG readback write failed: {error}")));
                return;
            }
            self.captures.push(name);
            let scale = ctx.pixels_per_point();
            let min_x = (self.canvas.min.x * scale).max(0.0) as usize;
            let max_x = ((self.canvas.max.x * scale) as usize).min(pixels.size[0]);
            let min_y = (self.canvas.min.y * scale).max(0.0) as usize;
            let max_y = ((self.canvas.max.y * scale) as usize).min(pixels.size[1]);
            let mut canvas_pixels = Vec::new();
            for y in min_y..max_y {
                canvas_pixels.extend_from_slice(
                    &rgba[(y * pixels.size[0] + min_x) * 4..(y * pixels.size[0] + max_x) * 4],
                );
            }
            if canvas_pixels.is_empty() {
                self.finish(ctx, Some("Readback did not include a valid canvas".into()));
                return;
            }
            if stage == 0 {
                self.before_pixels = Some(canvas_pixels);
                self.probe
                    .replace_material(0, [190, 70, 120])
                    .expect("script revision");
                self.phase = 2;
                ctx.request_repaint();
            } else {
                let changed = self
                    .before_pixels
                    .as_ref()
                    .is_some_and(|before| *before != canvas_pixels);
                let frozen = self.frozen_ms == Some(self.probe.observe().simulation_ms);
                self.finish(
                    ctx,
                    if changed && frozen {
                        None
                    } else {
                        Some(format!(
                            "Contract mismatch: canvas_changed={changed}, time_frozen={frozen}"
                        ))
                    },
                );
            }
        }
    }
}

impl eframe::App for Candidate {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let now = self.started.elapsed().as_millis() as u64;
        self.probe.tick(now).expect("monotonic experiment clock");
        self.receive_captures(ctx);
        if self.completed {
            return;
        }
        if self.automatic && self.started.elapsed() > Duration::from_secs(25) {
            self.finish(ctx, Some("Internal render test timed out".into()));
            return;
        }
        if self.automatic && self.phase == 0 && self.frames >= 120 {
            self.probe.pause(true, now).expect("freeze simulation");
            self.frozen_ms = Some(self.probe.observe().simulation_ms);
            self.phase = 1;
            self.capture(ctx, 0);
        }
        if self.automatic || !self.probe.observe().paused {
            ctx.send_viewport_cmd(egui::ViewportCommand::RequestPaintWhileHidden);
            ctx.request_repaint_after(Duration::from_millis(16));
        }
    }

    fn ui(&mut self, root: &mut egui::Ui, frame: &mut eframe::Frame) {
        let ctx = root.ctx().clone();
        if self.automatic && self.frames == 0 {
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
        }
        if let Some(seconds) = frame.info().cpu_usage {
            self.samples.push(f64::from(seconds) * 1000.0);
        }
        self.frames += 1;
        if let Some(request) = self.probe.render_request() {
            let rendered = request.render();
            let pixels = egui::ColorImage::from_rgb([32, 32], rendered.rgb());
            if let Some(texture) = &mut self.texture {
                texture.set(pixels, egui::TextureOptions::NEAREST);
            } else {
                self.texture = Some(ctx.load_texture(
                    "synthetic-material",
                    pixels,
                    egui::TextureOptions::NEAREST,
                ));
            }
            self.probe.present(&rendered);
        }
        egui::Panel::top("scene-controls").show(root, |ui| {
            ui.horizontal(|ui| {
                ui.strong("P0 native candidate / 独立实验");
                if ui.button("Scene A").clicked() {
                    self.probe.select_scene(Scene::First);
                }
                if ui.button("Scene B").clicked() {
                    self.probe.select_scene(Scene::Second);
                }
                if ui.button("Freeze / Resume").clicked() {
                    self.probe
                        .pause(
                            !self.probe.observe().paused,
                            self.started.elapsed().as_millis() as u64,
                        )
                        .unwrap();
                }
            });
            ui.horizontal(|ui| {
                if ui.button("Edit").clicked() {
                    self.probe.select_tool(Tool::Edit);
                }
                if ui.button("Interact").clicked() {
                    self.probe.select_tool(Tool::Interact);
                }
                ui.label("Placeholder scenes; no stable rules or gameplay");
            });
        });
        egui::Panel::left("asset-library")
            .default_size(180.0)
            .show(root, |ui| {
                ui.heading("Asset library");
                egui::CollapsingHeader::new("Synthetic samples")
                    .default_open(true)
                    .show(ui, |ui| {
                        for number in 0..10 {
                            ui.label(format!("Sample {number:02}"));
                        }
                    });
                ui.label("中文 / 日本語 font display");
            });
        egui::Panel::right("layers-parameters")
            .default_size(210.0)
            .show(root, |ui| {
                ui.heading("Layers");
                egui::ScrollArea::vertical()
                    .max_height(230.0)
                    .show(ui, |ui| {
                        for number in 0..50 {
                            ui.label(format!("Layer {number:02}"));
                        }
                    });
                ui.separator();
                ui.heading("Preview parameters");
                ui.add(egui::Slider::new(&mut self.zoom, 0.5..=1.5).text("View zoom"));
                if ui.button("Change sample material").clicked() {
                    let revision = self.probe.observe().content_revision;
                    let color = if revision.is_multiple_of(2) {
                        [190, 70, 120]
                    } else {
                        [40, 80, 120]
                    };
                    self.probe.replace_material(revision, color).unwrap();
                }
                let state = self.probe.observe();
                ui.label(format!("Simulation: {} ms", state.simulation_ms));
                ui.label(format!("Frozen: {}", state.paused));
                ui.label(format!("Material revision: {}", state.content_revision));
            });
        egui::CentralPanel::default().show(root, |ui| {
            let (response, painter) =
                ui.allocate_painter(ui.available_size(), egui::Sense::click());
            self.canvas = response.rect;
            painter.rect_filled(self.canvas, 0.0, egui::Color32::from_rgb(18, 22, 30));
            if let Some(texture) = &self.texture {
                for number in 0..50 {
                    let column = (number % 10) as f32;
                    let row = (number / 10) as f32;
                    let side = 52.0 * self.zoom;
                    let rect = egui::Rect::from_min_size(
                        self.canvas.min + egui::vec2(18.0 + column * 48.0, 24.0 + row * 52.0),
                        egui::vec2(side, side),
                    );
                    painter.image(
                        texture.id(),
                        rect,
                        egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
                        egui::Color32::from_white_alpha(220),
                    );
                }
            }
            if response.clicked() {
                self.probe.game_press();
            }
        });
        if self.automatic && self.phase == 2 {
            self.phase = 3;
            self.capture(&ctx, 1);
        }
    }
}

fn main() -> eframe::Result {
    let mut args = std::env::args().skip(1);
    let automatic = args.next().as_deref() == Some("--self-test");
    let output = PathBuf::from(
        args.next()
            .unwrap_or_else(|| "artifacts/p0/egui-native".into()),
    );
    let artifact_root = std::env::current_dir()
        .expect("run from repository root")
        .join("artifacts/p0");
    std::fs::create_dir_all(&artifact_root).expect("create P0 artifact root");
    let artifact_root = artifact_root.canonicalize().expect("resolve artifact root");
    let parent = output.parent().expect("output parent").canonicalize();
    assert_eq!(
        parent.as_ref().ok(),
        Some(&artifact_root),
        "Output must be a new direct child of artifacts/p0"
    );
    let output = artifact_root.join(output.file_name().expect("output directory name"));
    if output.exists() {
        panic!("Use a new output directory; existing artifacts are preserved");
    }
    std::fs::create_dir_all(&output).expect("create experiment output");
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 720.0])
            .with_visible(!automatic),
        renderer: eframe::Renderer::Wgpu,
        persist_window: false,
        ..Default::default()
    };
    eframe::run_native(
        "P0 egui candidate",
        options,
        Box::new(move |context| Ok(Box::new(Candidate::new(context, output, automatic)))),
    )
}
