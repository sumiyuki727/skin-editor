//! Manual P0 drag UI and opt-in synthetic egui-input/native-render proof.
use eframe::egui;
use p0_egui::{composition_view::LiveComposition, interaction_view::GestureView};
use serde_json::json;
use std::{
    path::PathBuf,
    sync::{Arc, mpsc},
    time::Instant,
};

struct App {
    view: GestureView,
    start: Instant,
    automatic: bool,
    output: Option<PathBuf>,
    step: u8,
    frames: u64,
    waiting: bool,
    input_step: Option<u8>,
    origin: egui::Pos2,
    tx: mpsc::Sender<(u16, Arc<egui::ColorImage>, Instant)>,
    rx: mpsc::Receiver<(u16, Arc<egui::ColorImage>, Instant)>,
    composition: Option<LiveComposition>,
    edit_readback: bool,
    edit_index: usize,
    input_started: Instant,
    edit_ui_ms: Vec<f64>,
    edit_readback_ms: Vec<f64>,
    pixel_checks: Vec<serde_json::Value>,
    crops: Vec<Vec<u8>>,
    observations: Vec<serde_json::Value>,
}
impl App {
    fn new(automatic: bool, output: Option<PathBuf>) -> Self {
        let (tx, rx) = mpsc::channel();
        let mut view = GestureView::default();
        view.pause(true, 0);
        Self {
            view,
            start: Instant::now(),
            automatic,
            output,
            step: 0,
            frames: 0,
            waiting: false,
            input_step: None,
            origin: egui::Pos2::ZERO,
            tx,
            rx,
            crops: Vec::new(),
            observations: Vec::new(),
            composition: None,
            edit_readback: false,
            edit_index: 0,
            input_started: Instant::now(),
            edit_ui_ms: Vec::new(),
            edit_readback_ms: Vec::new(),
            pixel_checks: Vec::new(),
        }
    }
    fn capture(&mut self, ctx: &egui::Context, tag: u16) {
        self.waiting = true;
        let tx = self.tx.clone();
        let wake = ctx.clone();
        ctx.request_screenshot(move |image| {
            let _ = tx.send((tag, image, Instant::now()));
            wake.request_repaint();
        });
    }
    fn advance_test(&mut self, ctx: &egui::Context) {
        // Screenshot completion can advance logic after raw_input_hook already
        // ran. Never consume a step whose input was not actually supplied.
        if self.waiting || self.input_step.take() != Some(self.step) {
            return;
        }
        match self.step {
            0 if self.frames >= 8 => self.step = 1,
            1 => self.capture(ctx, 0),
            4 => self.capture(ctx, 1),
            6 => self.capture(ctx, 2),
            11 => self.capture(ctx, 3),
            9 if self.composition.is_some() => {
                if self.edit_readback {
                    self.capture(ctx, 1000 + self.edit_index as u16);
                } else {
                    self.edit_index += 1;
                    if self.edit_index == 300 {
                        self.step = 10;
                    }
                }
            }
            2 | 3 | 5 | 7 | 8 | 9 | 10 => self.step += 1,
            _ => {}
        }
    }
}
fn button(p: egui::Pos2, pressed: bool) -> egui::Event {
    egui::Event::PointerButton {
        pos: p,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: Default::default(),
    }
}
fn escape(pressed: bool) -> egui::Event {
    egui::Event::Key {
        key: egui::Key::Escape,
        physical_key: None,
        pressed,
        repeat: false,
        modifiers: Default::default(),
    }
}
impl eframe::App for App {
    fn raw_input_hook(&mut self, _ctx: &egui::Context, raw: &mut egui::RawInput) {
        if !self.automatic {
            return;
        }
        // Only this hidden test window's input is replaced. No OS input injection.
        raw.events.clear();
        raw.focused = true;
        self.input_step = None;
        if self.waiting {
            return;
        }
        self.input_step = Some(self.step);
        if self.step == 9 {
            self.input_started = Instant::now();
        }
        raw.events = match self.step {
            2 | 8 => {
                self.origin = self.view.target().center();
                vec![
                    egui::Event::PointerMoved(self.origin),
                    button(self.origin, true),
                ]
            }
            3 => vec![egui::Event::PointerMoved(
                self.origin + egui::vec2(30., 20.),
            )],
            5 => vec![escape(true)],
            7 => vec![
                escape(false),
                button(self.origin + egui::vec2(30., 20.), false),
            ],
            9 => vec![egui::Event::PointerMoved(
                self.origin
                    + if self.composition.is_some() && self.edit_index < 299 {
                        egui::vec2(
                            60. + (self.edit_index % 50) as f32,
                            30. + (self.edit_index % 10) as f32,
                        )
                    } else {
                        egui::vec2(60., 30.)
                    },
            )],
            10 => vec![button(self.origin + egui::vec2(60., 30.), false)],
            _ => vec![],
        };
    }
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.automatic {
            return;
        }
        assert!(
            self.start.elapsed().as_secs() < if self.composition.is_some() { 90 } else { 25 },
            "Interaction proof timeout"
        );
        while let Ok((tag, image, completed)) = self.rx.try_recv() {
            if tag >= 1000 {
                self.edit_readback_ms
                    .push(completed.duration_since(self.input_started).as_secs_f64() * 1000.);
                if [0, 99, 199, 299].contains(&self.edit_index) {
                    let bytes: Vec<u8> = image.pixels.iter().flat_map(|p| p.to_array()).collect();
                    image::save_buffer(
                        self.output
                            .as_ref()
                            .unwrap()
                            .join(format!("edit-{}.png", self.edit_index)),
                        &bytes,
                        image.size[0] as u32,
                        image.size[1] as u32,
                        image::ColorType::Rgba8,
                    )
                    .unwrap();
                }
                self.edit_index += 1;
                self.waiting = false;
                if self.edit_index == 300 {
                    self.step = 10;
                }
                continue;
            }
            if let Some(c) = &self.composition {
                self.pixel_checks
                    .push(c.verify(self.output.as_ref().unwrap(), tag).unwrap());
            }
            let rgba: Vec<u8> = image.pixels.iter().flat_map(|p| p.to_array()).collect();
            let png = image::RgbaImage::from_raw(image.size[0] as u32, image.size[1] as u32, rgba)
                .unwrap();
            png.save(
                self.output
                    .as_ref()
                    .unwrap()
                    .join(format!("stage-{tag}.png")),
            )
            .unwrap();
            let scale = ctx.pixels_per_point();
            let rect = self.view.canvas();
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
            self.observations.push(json!({"stage":tag,"committed_node":self.view.snapshot().id,"nodes":self.view.node_count(),"visible_position":self.view.position(),"simulation_ms":self.view.simulation().simulation_ms,"accepted_game_inputs":self.view.simulation().accepted_game_inputs}));
            self.waiting = false;
            self.step += 1;
        }
        if self.step == 12 {
            let passed = self.crops.len() == 4
                && self.crops[0] != self.crops[1]
                && self.crops[0] == self.crops[2]
                && self.crops[0] != self.crops[3]
                && self
                    .view
                    .snapshot()
                    .position
                    .iter()
                    .zip([100., 70.])
                    .all(|(a, b)| {
                        (a - b).abs()
                            <= if self.composition.is_some() {
                                0.0001
                            } else {
                                0.
                            }
                    })
                && self.view.node_count() == 2
                && self.view.simulation().simulation_ms == 0
                && self.view.simulation().accepted_game_inputs == 0;
            let result = json!({"schema_version":1,"status":if passed {"passed"} else {"failed"},"observations":self.observations,
                "before_equals_cancelled_canvas":self.crops[0]==self.crops[2],"draft_changes_canvas":self.crops[0]!=self.crops[1],"commit_changes_canvas":self.crops[0]!=self.crops[3],
                "composition":self.composition.as_ref().map(|c|c.metrics()),"composition_pixel_checks":self.pixel_checks,
                "edit_count":self.edit_index,"edit_ui_host_ms":self.edit_ui_ms,"edit_readback_ms":self.edit_readback_ms,"per_edit_readback":self.edit_readback,
                "system_capture_used":false,"input":"synthetic egui RawInput through the same widget/command route as manual mode",
                "limits":["No product project or stable scenes; optional standard fixture composition","Physical mouse/keyboard, OS focus transitions, IME, full screen and DPI transitions remain manual checks","No audio integration"]});
            std::fs::write(
                self.output.as_ref().unwrap().join("result.json"),
                serde_json::to_vec_pretty(&result).unwrap(),
            )
            .unwrap();
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            self.step = 13;
        }
        ctx.request_repaint();
    }
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let wall = if self.automatic {
            self.frames * 10
        } else {
            self.start.elapsed().as_millis() as u64
        };
        let start = Instant::now();
        let edit = self.automatic
            && self.step == 9
            && self.input_step == Some(9)
            && self.composition.is_some();
        if let Some(c) = &mut self.composition {
            self.view.show_composition(ui, wall, c.texture_id);
            c.refresh(self.view.position()).unwrap();
        } else {
            self.view.show(ui, wall);
        }
        if edit {
            self.edit_ui_ms.push(start.elapsed().as_secs_f64() * 1000.);
        }
        self.frames += 1;
        if !self.automatic && !self.view.simulation().paused {
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_millis(16));
        }
        if self.automatic {
            self.advance_test(ui.ctx());
        }
    }
}
fn main() -> eframe::Result {
    let mut args = std::env::args().skip(1);
    let automatic = match args.next().as_deref() {
        Some("--self-test") => true,
        Some("--interactive") => false,
        _ => panic!("Use --interactive or --self-test <new artifacts/p0 directory>"),
    };
    let output = if automatic {
        let requested = PathBuf::from(args.next().expect("New output directory required"));
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("artifacts/p0")
            .canonicalize()
            .unwrap();
        assert_eq!(
            requested.parent().unwrap().canonicalize().unwrap(),
            root,
            "Output must be directly under artifacts/p0"
        );
        assert!(!requested.exists(), "Existing output must be preserved");
        std::fs::create_dir(&requested).unwrap();
        Some(requested)
    } else {
        None
    };
    let mut fixture = None;
    let mut edit_readback = false;
    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--fixture" => fixture = Some(PathBuf::from(args.next().expect("Fixture directory"))),
            "--edit-readback" => edit_readback = true,
            _ => panic!("Unexpected argument"),
        }
    }
    assert!(
        !edit_readback || (automatic && fixture.is_some()),
        "Edit readbacks require automatic fixture mode"
    );
    let mut setup = eframe::egui_wgpu::WgpuSetupCreateNew::without_display_handle();
    if fixture.is_some() {
        setup.instance_descriptor.backends = eframe::wgpu::Backends::VULKAN;
        let original = setup.device_descriptor;
        setup.device_descriptor = Arc::new(move |adapter| {
            let mut d = original(adapter);
            d.memory_hints = eframe::wgpu::MemoryHints::MemoryUsage;
            d
        });
    }
    eframe::run_native(
        "P0 cancellable gesture",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1100., 720.])
                .with_visible(!automatic),
            renderer: eframe::Renderer::Wgpu,
            wgpu_options: eframe::egui_wgpu::WgpuConfiguration {
                wgpu_setup: setup.into(),
                surface: eframe::egui_wgpu::SurfaceConfig {
                    present_mode: eframe::wgpu::PresentMode::AutoNoVsync,
                    ..eframe::egui_wgpu::SurfaceConfig::LOW_LATENCY
                },
                ..Default::default()
            },
            persist_window: false,
            ..Default::default()
        },
        Box::new(move |context| {
            let mut app = App::new(automatic, output);
            if let Some(fixture) = fixture {
                app.composition =
                    Some(LiveComposition::new(context, &fixture).map_err(std::io::Error::other)?);
            }
            app.edit_readback = edit_readback;
            Ok(Box::new(app))
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn screenshot_completion_between_input_and_ui_cannot_skip_the_next_press() {
        let ctx = egui::Context::default();
        let mut app = App::new(true, None);
        app.step = 1;
        app.waiting = true;
        let mut raw = egui::RawInput::default();
        eframe::App::raw_input_hook(&mut app, &ctx, &mut raw);
        assert!(raw.events.is_empty());
        // Completion in App::logic, after the input hook and before App::ui.
        app.step = 2;
        app.waiting = false;
        app.advance_test(&ctx);
        assert_eq!(app.step, 2);
        let _ = ctx.run_ui(Default::default(), |ui| app.view.show(ui, 0));
        eframe::App::raw_input_hook(&mut app, &ctx, &mut raw);
        assert!(
            raw.events
                .iter()
                .any(|e| matches!(e, egui::Event::PointerButton { pressed: true, .. }))
        );
        app.advance_test(&ctx);
        assert_eq!(app.step, 3);
        // A second UI pass must not consume movement without a new input hook.
        app.advance_test(&ctx);
        assert_eq!(app.step, 3);
    }
}
