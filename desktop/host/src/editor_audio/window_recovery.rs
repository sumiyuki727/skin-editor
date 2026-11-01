//! Explicit same-process eframe window replacement, not in-place device repair.
use super::{
    combined::probes::Probes,
    gui_verification::{Observations, Shared},
    *,
};
use crate::{Editor, presentation::RebuildSeed};
use eframe::{App, egui};
use skin_editor_ui::Presenter;
use std::{
    cell::RefCell,
    path::{Path, PathBuf},
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicU8, Ordering},
        mpsc,
    },
    time::{Duration, Instant},
};
mod activity;
mod backend;
mod fault;
mod init_failure;
mod report;
mod restore;
mod sound;
fn error(e: impl std::fmt::Debug) -> String {
    format!("{e:?}")
}
struct Seed {
    session: ProjectSession,
    ui: skin_editor_ui::EditorUi,
    images: skin_editor_infrastructure::Images,
    preview: skin_editor_preview::PreviewState,
    gpu: RebuildSeed,
}
impl Seed {
    fn restore(
        self,
        ctx: &egui::Context,
        state: &eframe::egui_wgpu::RenderState,
    ) -> Result<Editor, (String, Box<Self>)> {
        let mut importer =
            match crate::import_worker::ImportWorker::new(self.images.clone(), ctx.clone()) {
                Ok(importer) => importer,
                Err(error) => return Err((error, Box::new(self))),
            };
        let host = match self.gpu.build_retained(
            state.device.clone(),
            state.queue.clone(),
            state.target_format,
            ctx.clone(),
        ) {
            Ok(host) => host,
            Err(err) => {
                importer.close();
                return Err((error(err), Box::new(self)));
            }
        };
        Ok(Editor {
            session: self.session,
            ui: self.ui,
            images: self.images,
            preview: self.preview,
            host,
            importer,
            last: Instant::now(),
            smoke_started: None,
            ui_passes: 0,
            main_work: None,
            repaint_pass: crate::repaint_deadline::RepaintPass::default(),
        })
    }
}
struct Returned {
    frame_key: Option<FrameKey>,
    surface_failures: usize,
    editor: Editor,
    audio: Rc<RefCell<EditorAudio>>,
    audio_signal: skin_editor_infrastructure::separated_native::NativeFaultSignal,
    held: Option<egui::FullOutput>,
    record: String,
    canonical: Vec<u8>,
    failure: Option<String>,
}
struct Run {
    plan: fault::Plan,
    fault: Option<Arc<fault::Signals>>,
    late_frame_rejected: bool,
    held_key: Option<FrameKey>,
    previous_key: Option<FrameKey>,
    previous_frame_rejected: bool,
    editor: Option<Editor>,
    audio: Rc<RefCell<EditorAudio>>,
    observer: Option<Observations>,
    probes: Probes,
    path: PathBuf,
    index: usize,
    phase: u8,
    started: Instant,
    playing: Option<Instant>,
    issued: usize,
    pause: u64,
    restore_anchor: Option<u64>,
    screens: Vec<[usize; 4]>,
    expected_screens: Vec<[usize; 2]>,
    shot_tx: mpsc::SyncSender<Arc<egui::ColorImage>>,
    shot_rx: mpsc::Receiver<Arc<egui::ColorImage>>,
    software_scale: bool,
    native_before: f32,
    native_after: f32,
    physical_dpi: bool,
    held: Option<egui::FullOutput>,
    canonical: Vec<u8>,
    max_pixel_error: u8,
    notice: Arc<AtomicU8>,
    failure: Option<String>,
    out: Rc<RefCell<Option<Returned>>>,
    old_voice: usize,
    pending: bool,
    baseline: ProjectSnapshot,
    audio_signal: skin_editor_infrastructure::separated_native::NativeFaultSignal,
}
impl Run {
    fn shot(&mut self, ctx: &egui::Context) {
        let size = ctx.viewport_rect().size() * ctx.pixels_per_point();
        self.expected_screens
            .push([size.x.round() as usize, size.y.round() as usize]);
        let tx = self.shot_tx.clone();
        let ctx = ctx.clone();
        ctx.clone().request_screenshot(move |image| {
            let _ = tx.try_send(image);
            ctx.request_repaint();
        });
    }
    fn screenshots(&mut self) -> Result<(), String> {
        while let Ok(image) = self.shot_rx.try_recv() {
            if self.screens.len() == 4 || image.pixels.len() != image.width() * image.height() {
                return Err("screenshot bound/size".into());
            }
            let mut colors = std::collections::BTreeSet::new();
            for p in &image.pixels {
                colors.insert(p.to_array());
                if colors.len() == 64 {
                    break;
                }
            }
            if image.width() < 600 || image.height() < 400 || colors.len() < 16 {
                return Err("renderer screenshot is blank/truncated".into());
            }
            let expected = self
                .expected_screens
                .get(self.screens.len())
                .ok_or("unexpected renderer readback")?;
            if image.width().abs_diff(expected[0]) > 2 || image.height().abs_diff(expected[1]) > 2 {
                return Err("surface pixels do not match viewport/DPI".into());
            }
            let pixels: Vec<u8> = image.pixels.iter().flat_map(|p| p.to_array()).collect();
            skin_editor_infrastructure::write_png(
                &self
                    .path
                    .join(format!("ui-{}-{}.png", self.index, self.screens.len())),
                &pixels,
                PixelSize::new(image.width() as u32, image.height() as u32).map_err(error)?,
            )?;
            self.screens.push([
                image.width(),
                image.height(),
                image.pixels.len(),
                colors.len(),
            ]);
        }
        Ok(())
    }
    fn step(&mut self, ctx: &egui::Context) -> Result<(), String> {
        if self.started.elapsed() > Duration::from_secs(40) {
            return Err(format!("window phase {} deadline", self.phase));
        }
        self.screenshots()?;
        let e = self.editor.as_mut().ok_or("editor missing")?;
        let state = &e.ui.audio_state;
        if self.phase < 14
            && let Some(err) = &state.error
        {
            return Err(err.clone());
        }
        let ready = state.enabled
            && !state.pending_controls
            && state.components.len() == 1
            && state.components[0].preparation == AudioPreparationState::Ready;
        match self.phase {
            0 if ready && e.host.current().is_some() && e.ui_passes >= 2 => {
                if let Some(key) = self.previous_key {
                    let output = egui::Context::default().run_ui(egui::RawInput::default(), |ui| {
                        e.host.paint(ui, key, ui.max_rect());
                    });
                    self.previous_frame_rejected = output.shapes.is_empty();
                    if !self.previous_frame_rejected {
                        return Err("new device accepted old FrameKey".into());
                    }
                }
                if !state.frozen || state.voices != [0, 0] {
                    return Err("window reopened playing or with old voices".into());
                }
                if let Some(anchor) = self.restore_anchor
                    && state
                        .output
                        .as_ref()
                        .ok_or("no audio clock")?
                        .copied_background
                        != anchor
                {
                    return Err("background reset across window".into());
                }
                if e.host
                    .bind_loss(crate::presentation::DeviceLoss::default())
                    .is_ok()
                {
                    return Err("used device fence replaced".into());
                }
                self.shot(ctx);
                self.phase = 1;
            }
            1 if self.screens.len() == 1 => {
                e.ui.paused = false;
                e.ui.audio_command(&e.session, AudioEditorAction::SceneRunning(true));
                self.phase = 2;
                crate::load_verification::phase(if self.index == 0 {
                    "window_before_play"
                } else {
                    "window_after_play"
                });
            }
            2 if ready && !state.frozen && !state.freezing => {
                let start = *self.playing.get_or_insert_with(Instant::now);
                if self.issued < 64
                    && start.elapsed() >= Duration::from_millis(self.issued as u64 * 80)
                {
                    let lane = if self.issued.is_multiple_of(2) {
                        Channel::Scene
                    } else {
                        Channel::Audition
                    };
                    sound::trigger(e, &self.audio, &mut self.probes, lane)?;
                    self.issued += 1;
                }
                if self.issued == 64 && self.probes.empty() {
                    e.ui.paused = true;
                    e.ui.audio_command(&e.session, AudioEditorAction::SceneRunning(false));
                    self.phase = 3;
                }
            }
            3 if ready && state.frozen && !state.freezing => {
                self.pause = state
                    .output
                    .as_ref()
                    .ok_or("pause clock absent")?
                    .copied_background;
                if self.pause <= self.restore_anchor.unwrap_or(0) {
                    return Err("background failed to resume".into());
                }
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(980., 700.)));
                self.phase = 4;
            }
            4 if size(ctx, [980., 700.]) => {
                self.shot(ctx);
                self.phase = 5;
            }
            5 if self.screens.len() == 2 => {
                ctx.set_zoom_factor(1.25);
                self.phase = 6;
            }
            6 if (ctx.zoom_factor() - 1.25).abs() < 0.001 => {
                self.software_scale = true;
                self.shot(ctx);
                self.phase = 7;
            }
            7 if self.screens.len() == 3 => {
                ctx.set_zoom_factor(1.);
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(1180., 800.)));
                self.phase = 8;
            }
            8 if size(ctx, [1180., 800.]) && (ctx.zoom_factor() - 1.).abs() < 0.001 => {
                self.native_before = ctx
                    .input(|i| i.viewport().native_pixels_per_point)
                    .ok_or("native DPI unknown")?;
                println!("{{\"phase\":\"window_dpi\",\"window\":{}}}", self.index);
                self.phase = 9;
            }
            9 => {
                let ack = self.path.join(format!("dpi-{}.ok", self.index));
                if let Ok(bytes) = std::fs::read(ack) {
                    self.native_after = ctx
                        .input(|i| i.viewport().native_pixels_per_point)
                        .ok_or("restored native DPI unknown")?;
                    if bytes == b"moved" && self.native_after == self.native_before {
                        return Ok(());
                    }
                    if bytes != b"moved" && bytes != b"skipped" {
                        return Err("wrong DPI acknowledgement".into());
                    }
                    self.physical_dpi = bytes == b"moved";
                    ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                    println!(
                        "{{\"phase\":\"window_minimize\",\"window\":{}}}",
                        self.index
                    );
                    self.phase = 10;
                }
            }
            10 if std::fs::read(self.path.join(format!("restore-{}.ok", self.index)))
                .ok()
                .as_deref()
                == Some(self.index.to_string().as_bytes())
                && ctx.input(|i| i.viewport().minimized) != Some(true) =>
            {
                self.shot(ctx);
                self.phase = 11;
            }
            11 if self.screens.len() == 4 && ready => {
                if state
                    .output
                    .as_ref()
                    .ok_or("restored clock absent")?
                    .copied_background
                    != self.pause
                {
                    return Err("paused background advanced during window changes".into());
                }
                crate::verification::frame(
                    &mut e.host,
                    &e.ui,
                    &e.session,
                    [128, 128],
                    1_000_000 + self.index as u64,
                )?;
                self.canonical = crate::verification::readback(
                    &e.host,
                    e.host.active_texture().ok_or("no canonical frame")?,
                    [128, 128],
                )?;
                let cpu =
                    crate::verification::cpu_pixels(&e.session, &e.ui, &e.images, [128, 128])?;
                self.max_pixel_error = self
                    .canonical
                    .iter()
                    .zip(cpu)
                    .map(|(a, b)| a.abs_diff(b))
                    .max()
                    .unwrap_or(255);
                if self.max_pixel_error > 2 {
                    return Err("window preview pixel mismatch".into());
                }
                e.ui.audio_command(&e.session, AudioEditorAction::Audition(sound::component()));
                if self.plan != fault::Plan::Planned {
                    e.ui.paused = false;
                    e.ui.audio_command(&e.session, AudioEditorAction::SceneRunning(true));
                }
                self.phase = 12;
            }
            12 if state.voices[1] > 0 => {
                if self.plan != fault::Plan::Planned && (state.frozen || state.freezing) {
                    return Ok(());
                }
                self.old_voice = state.voices[1];
                // UI may still own an older GPU candidate after the canonical
                // readback. Drain that actual submission before asking for the
                // specific preparation we will invalidate; wait() alone is not
                // evidence that a new request was admitted.
                e.host.wait_preparation().map_err(error)?;
                e.host
                    .device
                    .poll(eframe::egui_wgpu::wgpu::PollType::Wait {
                        submission_index: None,
                        timeout: Some(Duration::from_secs(3)),
                    })
                    .map_err(error)?;
                e.host.poll().map_err(error)?;
                let key = e
                    .host
                    .current()
                    .ok_or("missing active frame at loss")?
                    .frame
                    .key;
                self.held_key = Some(key);
                self.held = Some(egui::Context::default().run_ui(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(64., 64.),
                        )),
                        ..Default::default()
                    },
                    |ui| e.host.paint(ui, key, ui.max_rect()),
                ));
                if self.plan != fault::Plan::Planned {
                    self.phase = 13;
                    return Ok(());
                }
                e.session
                    .request_preview(
                        &mut e.host,
                        PreviewContext {
                            target: e.ui.target.ok_or("target missing")?,
                            draft: None,
                            size: [320, 200],
                            viewport_generation: 2_000_000 + self.index as u64,
                            scene: 0,
                            simulation_micros: 42,
                        },
                    )
                    .map_err(error)?;
                e.host.wait_preparation().map_err(error)?;
                self.pending = e.host.preparation_pending();
                if !self.pending {
                    return Err("window loss lacked in-flight preparation".into());
                }
                e.host.loss_signal().report();
                self.audio
                    .borrow()
                    .native
                    .as_ref()
                    .ok_or("native missing at loss")?
                    .fault_signal()
                    .report();
                e.ui.refresh_audio(&e.session);
                e.ui.paused = true;
                if e.ui.audio_state.enabled || !e.ui.audio_state.frozen {
                    return Err("window loss did not freeze sound".into());
                }
                self.phase = 14;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            _ => {}
        }
        Ok(())
    }
}
fn size(ctx: &egui::Context, want: [f32; 2]) -> bool {
    ctx.input(|i| i.viewport().inner_rect)
        .is_some_and(|r| (r.width() - want[0]).abs() < 2. && (r.height() - want[1]).abs() < 2.)
}
impl App for Run {
    fn logic(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if self.phase == 13
            && let Some(fault) = &self.fault
            && fault.fired.load(Ordering::Acquire) != 0
            && fault.noticed.load(Ordering::Acquire) == 0
            && let Some(editor) = &mut self.editor
        {
            // wgpu may defer DeviceLost dispatch until an owner poll. This is
            // nonblocking and runs before Editor logic/any new preparation.
            let _ = editor.host.poll();
            if fault.noticed.load(Ordering::Acquire) == 0 {
                ctx.request_repaint_after(Duration::from_millis(16));
                return;
            }
        }
        if self.phase == 13
            && let Some(fault) = &self.fault
            && fault.fired.load(Ordering::Acquire) != 0
        {
            fault.logic_observed.fetch_add(1, Ordering::AcqRel);
            if self
                .audio
                .borrow()
                .native
                .as_ref()
                .is_some_and(|n| n.stats().device_fault)
            {
                fault.audio_closed_before_logic.store(1, Ordering::Release);
            } else {
                self.failure = Some("GPU notice did not close audio before GUI logic".into());
            }
            self.phase = 14;
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        if self.failure.is_none() && self.phase < 14 {
            if self.phase <= 3
                && let Some(observer) = &self.observer
                && let Err(e) = sound::collect(&self.audio, &mut self.probes, observer)
            {
                self.failure = Some(e);
            }
            if let Some(e) = &mut self.editor {
                e.logic(ctx, frame);
            }
        }
        ctx.request_repaint_after(Duration::from_millis(16));
    }
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        if self.phase == 13
            && self
                .fault
                .as_ref()
                .is_some_and(|f| f.fired.load(Ordering::Acquire) != 0)
        {
            // Let the actual surface/device status reach the owner. No new
            // Editor work or second injection is admitted during this gap.
            ui.ctx().request_repaint_after(Duration::from_millis(16));
            return;
        }
        if self.failure.is_none()
            && let Err(e) = self.step(ui.ctx())
        {
            self.failure = Some(e);
        }
        if self.failure.is_none() && self.phase < 14 {
            if let Some(e) = &mut self.editor {
                e.ui(ui, frame);
                if self.phase == 13
                    && let Some(fault) = &self.fault
                {
                    // Arm after complete_ui: its normal preview request must
                    // not consume the specific result we intend to invalidate.
                    let mut arm = || -> Result<(), String> {
                        e.host.wait_preparation().map_err(error)?;
                        e.host
                            .device
                            .poll(eframe::egui_wgpu::wgpu::PollType::Wait {
                                submission_index: None,
                                timeout: Some(Duration::from_secs(3)),
                            })
                            .map_err(error)?;
                        e.host.poll().map_err(error)?;
                        e.session
                            .request_preview(
                                &mut e.host,
                                PreviewContext {
                                    target: e.ui.target.ok_or("target missing at live loss")?,
                                    draft: None,
                                    size: [320, 200],
                                    viewport_generation: 2_000_000 + self.index as u64,
                                    scene: 0,
                                    simulation_micros: 42,
                                },
                            )
                            .map_err(error)?;
                        e.host.wait_preparation().map_err(error)?;
                        Ok(())
                    };
                    if let Err(err) = arm() {
                        self.failure = Some(err);
                    } else {
                        self.pending = e.host.preparation_pending();
                        if !self.pending {
                            self.failure = Some("live fault lacked in-flight preparation".into());
                        } else {
                            fault::append(ui, fault.clone(), self.plan, e.host.device.clone());
                        }
                    }
                }
            }
        } else {
            if self.plan == fault::Plan::Planned {
                ui.label("Closing graphics generation; project stays in memory.");
            }
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
    fn on_exit(&mut self) {
        if let Some(mut observer) = self.observer.take()
            && !observer.close()
        {
            self.failure = Some("window observation thread did not join successfully".into());
        }
        if let Some(mut e) = self.editor.take() {
            if !e.interrupt_for_fault() {
                self.failure = Some("window UI draft cancellation refused".into());
            }
            e.importer.close();
            e.host.loss_signal().report();
            if let Some(native) = self.audio.borrow().native.as_ref() {
                native.fault_signal().report();
            }
            e.ui.refresh_audio(&e.session);
            if let Err(err) = backend::destroy(&e.host.device, &self.notice) {
                self.failure = Some(err);
            }
            let _ = e.host.poll();
            self.late_frame_rejected = e.host.current().is_none();
            if let Some(key) = self.held_key {
                let output = egui::Context::default().run_ui(egui::RawInput::default(), |ui| {
                    e.host.paint(ui, key, ui.max_rect());
                });
                self.late_frame_rejected &= output.shapes.is_empty();
            }
            if self.plan != fault::Plan::Planned {
                self.pause = self
                    .audio
                    .borrow()
                    .recovery
                    .and_then(|r| r.copied_frame)
                    .unwrap_or(0);
            }
            if e.session.snapshot() != self.baseline {
                self.failure = Some("project changed during window control".into());
            }
            let record = report::window(self, &self.audio.borrow(), e.ui_passes);
            *self.out.borrow_mut() = Some(Returned {
                frame_key: self.held_key,
                surface_failures: self
                    .fault
                    .as_ref()
                    .map_or(0, |f| f.surface_faults.load(Ordering::Acquire) as usize),
                editor: e,
                audio: self.audio.clone(),
                audio_signal: self.audio_signal.clone(),
                held: self.held.take(),
                record,
                canonical: std::mem::take(&mut self.canonical),
                failure: self.failure.take(),
            });
        }
    }
}
pub fn run(destination: &str) -> Result<(), String> {
    report::run(destination, fault::Plan::Planned)
}
pub fn run_fault(destination: &str, plan: &str) -> Result<(), String> {
    let plan = match plan {
        "prepare" => fault::Plan::Prepare,
        "paint" => fault::Plan::Paint,
        "rebuild-failure" => fault::Plan::RebuildFailure,
        "init-consumed" => fault::Plan::InitConsumed,
        "init-prepared" => fault::Plan::InitPrepared,
        "init-audio" => fault::Plan::InitAudio,
        "cleanup-held" => fault::Plan::CleanupHeld,
        "endpoint-held" => fault::Plan::EndpointHeld,
        "draft-gpu" => fault::Plan::DraftGpu,
        "draft-audio" => fault::Plan::DraftAudio,
        "draft-both" => fault::Plan::DraftBoth,
        _ => return Err("unknown present fault plan".into()),
    };
    report::run(destination, plan)
}
