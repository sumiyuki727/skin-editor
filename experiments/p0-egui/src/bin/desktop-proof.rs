//! Native desktop session recorder. It never captures other windows or input.
use eframe::{egui, wgpu};
use p0_egui::desktop_view::{DesktopView, Layout};
use serde_json::json;
use std::{
    fs::{self, OpenOptions},
    io::{Cursor, Write},
    path::{Path, PathBuf},
    sync::{Arc, mpsc},
    time::{Duration, Instant},
};

const FIXTURE: &str = "素材 日本語.png";
fn fixture_image() -> image::RgbaImage {
    image::RgbaImage::from_fn(32, 16, |x, y| {
        image::Rgba([
            if x < 16 { 230 } else { 30 },
            (y * 15) as u8,
            if x < 16 { 40 } else { 220 },
            255,
        ])
    })
}
fn write_new(path: &Path, value: &serde_json::Value) -> Result<(), String> {
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    f.write_all(&serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())
}
/// Compare canonical names before reading bytes; never use a foreign handle's bytes().
fn accept_drop(dropped: &Path, expected: &Path) -> Result<egui::ColorImage, ()> {
    let canonical = expected.canonicalize().map_err(|_| ())?;
    let parent = expected
        .parent()
        .ok_or(())?
        .canonicalize()
        .map_err(|_| ())?;
    if dropped.canonicalize().map_err(|_| ())? != canonical
        || canonical.parent() != Some(parent.as_path())
        || !expected.is_file()
    {
        return Err(());
    }
    if fs::metadata(expected).map_err(|_| ())?.len() > 65536 {
        return Err(());
    }
    let bytes = fs::read(expected).map_err(|_| ())?;
    let mut reader = image::ImageReader::with_format(Cursor::new(bytes), image::ImageFormat::Png);
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(32);
    limits.max_image_height = Some(16);
    limits.max_alloc = Some(65536);
    reader.limits(limits);
    let image = reader.decode().map_err(|_| ())?.into_rgba8();
    if image != fixture_image() {
        return Err(());
    }
    Ok(egui::ColorImage::from_rgba_unmultiplied(
        [32, 16],
        image.as_raw(),
    ))
}
struct App {
    view: DesktopView,
    output: PathBuf,
    fixture: PathBuf,
    start: Instant,
    automatic: bool,
    frames: u64,
    drop_texture: Option<egui::TextureHandle>,
    saved: bool,
    checkpoints: u64,
    capture_pending: bool,
    tx: mpsc::Sender<Arc<egui::ColorImage>>,
    rx: mpsc::Receiver<Arc<egui::ColorImage>>,
}
impl App {
    fn new(output: PathBuf, automatic: bool) -> Self {
        let fixture = output.join(FIXTURE);
        fixture_image().save(&fixture).unwrap();
        let (tx, rx) = mpsc::channel();
        Self {
            view: DesktopView::default(),
            output,
            fixture,
            start: Instant::now(),
            automatic,
            frames: 0,
            drop_texture: None,
            saved: false,
            checkpoints: 0,
            capture_pending: false,
            tx,
            rx,
        }
    }
    fn save(&mut self, final_record: bool) -> Result<(), String> {
        if final_record && self.saved {
            return Ok(());
        }
        if final_record {
            self.view.gesture.cancel();
        }
        let source = if self.automatic {
            "native_smoke_no_manual_input"
        } else {
            "native_user_session"
        };
        let mut result = self.view.report(source);
        result["elapsed_ms"] = json!(self.start.elapsed().as_millis());
        result["ended_by"] = json!(if self.view.finish_requested {
            "record_button"
        } else if final_record {
            "window_close_or_smoke"
        } else {
            "checkpoint"
        });
        result["system_capture_used"] = json!(false);
        result["input_text_persisted"] = json!(false);
        result["foreign_file_bytes_read"] = json!(false);
        let name = if final_record {
            "result.json".into()
        } else {
            self.checkpoints += 1;
            format!("checkpoint-{}.json", self.checkpoints)
        };
        write_new(&self.output.join(name), &result)?;
        if final_record {
            self.saved = true;
        }
        Ok(())
    }
}
impl eframe::App for App {
    fn raw_input_hook(&mut self, ctx: &egui::Context, raw: &mut egui::RawInput) {
        // Smoke ignores this window's input, so its own screenshot can never
        // persist accidentally typed text. Interactive mode preserves native events.
        if self.automatic {
            raw.events.clear();
            raw.dropped_files.clear();
            return;
        }
        for file in &raw.dropped_files {
            match accept_drop(file.path(), &self.fixture) {
                Ok(image) => {
                    self.drop_texture =
                        Some(ctx.load_texture("accepted-own-drop", image, Default::default()));
                    self.view.drop_result(true);
                }
                Err(()) => self.view.drop_result(false),
            }
        }
    }
    fn logic(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        if let Ok(image) = self.rx.try_recv() {
            let bytes: Vec<_> = image.pixels.iter().flat_map(|p| p.to_array()).collect();
            image::save_buffer(
                self.output.join("smoke-own-renderer.png"),
                &bytes,
                image.size[0] as u32,
                image.size[1] as u32,
                image::ColorType::Rgba8,
            )
            .unwrap();
            self.save(true).unwrap();
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        if self.automatic && self.start.elapsed() > Duration::from_secs(30) {
            self.view.error("Native smoke timeout".into());
            self.save(true).unwrap();
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        if self.saved {
            return;
        }
        let wall = self.start.elapsed().as_millis() as u64;
        self.view.show(
            ui,
            wall,
            &self.output.to_string_lossy(),
            self.drop_texture.as_ref().map(|t| t.id()),
        );
        if std::mem::take(&mut self.view.checkpoint_requested)
            && let Err(e) = self.save(false)
        {
            self.view.error(e);
        }
        if self.view.finish_requested {
            self.save(true).unwrap();
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        }
        if self.automatic {
            self.frames += 1;
            if self.frames == 15 {
                self.view.set_layout(Layout::Bottom);
            }
            if self.frames >= 30 && !self.capture_pending {
                self.capture_pending = true;
                let tx = self.tx.clone();
                let ctx = ui.ctx().clone();
                ui.ctx().request_screenshot(move |image| {
                    let _ = tx.send(image);
                    ctx.request_repaint();
                });
            }
            ui.ctx().request_repaint_after(Duration::from_millis(16));
        }
    }
    fn on_exit(&mut self) {
        if let Err(e) = self.save(true) {
            eprintln!("Desktop record failed: {e}");
        }
    }
}
fn main() -> eframe::Result {
    let mut args = std::env::args().skip(1);
    let automatic = match args.next().as_deref() {
        Some("--self-test") => true,
        Some("--interactive") => false,
        _ => panic!("Use --self-test or --interactive with new output"),
    };
    let output = PathBuf::from(args.next().expect("new artifacts/p0 output"));
    assert!(args.next().is_none());
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("artifacts/p0")
        .canonicalize()
        .unwrap();
    assert_eq!(output.parent().unwrap().canonicalize().unwrap(), root);
    assert!(!output.exists());
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
        "P0 Windows desktop checks - no audio",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1200., 820.])
                .with_min_inner_size([900., 600.])
                .with_visible(!automatic),
            renderer: eframe::Renderer::Wgpu,
            persist_window: false,
            wgpu_options: eframe::egui_wgpu::WgpuConfiguration {
                wgpu_setup: setup.into(),
                ..Default::default()
            },
            ..Default::default()
        },
        Box::new(move |_| Ok(Box::new(App::new(output, automatic)))),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn drop_gate_only_reads_exact_owned_fixture_and_never_imports_or_writes_foreign_files() {
        let dir = std::env::temp_dir().join(format!("p0-desktop-drop-{}", std::process::id()));
        fs::create_dir(&dir).unwrap();
        let expected = dir.join(FIXTURE);
        fixture_image().save(&expected).unwrap();
        assert!(accept_drop(&expected, &expected).is_ok());
        let foreign = dir.join("foreign.png");
        fs::write(&foreign, b"not our fixture").unwrap();
        assert!(accept_drop(&foreign, &expected).is_err());
        assert_eq!(fs::read(&foreign).unwrap(), b"not our fixture");
        fs::write(&expected, b"damaged").unwrap();
        assert!(accept_drop(&expected, &expected).is_err());
        fs::remove_file(&expected).unwrap();
        assert!(accept_drop(&expected, &expected).is_err());
        fs::remove_file(foreign).unwrap();
        fs::remove_dir(dir).unwrap();
    }
    #[test]
    fn checkpoints_do_not_overwrite_existing_evidence() {
        let path =
            std::env::temp_dir().join(format!("p0-desktop-record-{}.json", std::process::id()));
        write_new(&path, &json!({"status":"recorded"})).unwrap();
        assert!(write_new(&path, &json!({"status":"passed"})).is_err());
        assert!(fs::read_to_string(&path).unwrap().contains("recorded"));
        fs::remove_file(path).unwrap();
    }
}
