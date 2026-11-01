//! Opt-in full Editor stable windows. The only diagnostic wake is a phase end.
use super::{gui_verification::Shared, *};
use crate::{
    Editor, fresh_session,
    stable_window::{Interval, MEASURE, SETTLE},
};
use eframe::egui;
use std::{
    cell::RefCell,
    path::PathBuf,
    rc::Rc,
    time::{Duration, Instant},
};

struct Run {
    product: bool,
    editor: Editor,
    audio: Option<Rc<RefCell<EditorAudio>>>,
    source: Option<skin_editor_infrastructure::background_stream::StreamObservation>,
    path: PathBuf,
    resources_case: bool,
    origin: Instant,
    since: Instant,
    due: Instant,
    stage: u8,
    interval: Option<Interval>,
    before: [usize; 5],
    rows: Vec<String>,
    inputs: u64,
    visible: bool,
    geometry: [f32; 3],
    shape: [usize; 3],
    completed: bool,
    failure: Option<String>,
    adapter: String,
}
fn error(e: impl std::fmt::Debug) -> String {
    format!("{e:?}")
}
impl Run {
    fn new(
        c: &eframe::CreationContext<'_>,
        path: PathBuf,
        resources_case: bool,
        product: bool,
    ) -> Result<Self, String> {
        let state = c.wgpu_render_state.as_ref().ok_or("wgpu unavailable")?;
        let editor = Editor::with_initial_content(
            &c.egui_ctx,
            state,
            false,
            skin_editor_rendering::SourceUpload::QueueWrite,
            false,
        )?;
        let now = Instant::now();
        Ok(Self {
            product,
            editor,
            audio: None,
            source: None,
            path,
            resources_case,
            origin: now,
            since: now,
            due: now + SETTLE,
            stage: 0,
            interval: None,
            before: [0; 5],
            rows: vec![],
            inputs: 0,
            visible: true,
            geometry: [0.; 3],
            shape: [0; 3],
            completed: false,
            failure: None,
            adapter: format!("{:?}", state.adapter.get_info()),
        })
    }
    fn usage(&self) -> Result<[usize; 5], String> {
        Ok([
            self.audio
                .as_ref()
                .map_or(0, |a| a.borrow().budget.usage().bytes),
            self.editor.images.blobs().usage().map_err(error)?.bytes,
            self.editor.images.rgba_bytes(),
            self.editor.host.live_targets(),
            self.editor.host.source_usage().1,
        ])
    }
    fn background(&self) -> u64 {
        self.editor
            .ui
            .audio_state
            .output
            .as_ref()
            .map_or(0, |o| o.copied_background)
    }
    fn shape(&self) -> [usize; 3] {
        let s = self.editor.session.snapshot();
        [
            s.project
                .assets()
                .iter()
                .filter(|a| matches!(a.kind(), AssetKind::RasterImage(_)))
                .count(),
            s.project.documents().len(),
            s.project.documents().map(|d| d.layers().iter().len()).sum(),
        ]
    }
    fn begin(&mut self, name: &str, stage: u8, ctx: &egui::Context) -> Result<(), String> {
        let now = Instant::now();
        self.visible = ctx.input(|i| i.viewport().visible() != Some(false));
        self.geometry = [
            ctx.viewport_rect().width(),
            ctx.viewport_rect().height(),
            ctx.pixels_per_point(),
        ];
        self.before = self.usage()?;
        self.interval = Some(
            Interval::begin(
                now.duration_since(self.origin),
                self.since.duration_since(self.origin),
                self.editor.ui_passes,
                self.inputs,
                self.editor.ui.simulation_micros,
                self.background(),
            )
            .map_err(str::to_owned)?,
        );
        crate::load_verification::phase(name);
        self.stage = stage;
        self.since = now;
        self.due = now + MEASURE;
        Ok(())
    }
    fn action(&mut self, action: AudioEditorAction) -> Result<(), String> {
        self.editor.ui.audio_command(&self.editor.session, action);
        if let Some(error) = &self.editor.ui.audio_state.error {
            return Err(error.clone());
        }
        Ok(())
    }
    fn load(&mut self) -> Result<(), String> {
        crate::load_verification::phase("loading");
        crate::combined_verification::visual_fixture(&mut self.editor, &self.path)?;
        let snapshot = self.editor.session.snapshot();
        self.shape = [
            snapshot
                .project
                .assets()
                .iter()
                .filter(|a| matches!(a.kind(), AssetKind::RasterImage(_)))
                .count(),
            snapshot.project.documents().len(),
            snapshot
                .project
                .documents()
                .map(|d| d.layers().iter().len())
                .sum(),
        ];
        if self.shape != [100, 10, 500] {
            return Err("wrong visual fixture".into());
        }
        drop(snapshot);
        let store = self.editor.images.blobs().clone();
        let mut adapter = if self.product {
            crate::product_policy::Policy::CURRENT.audio(store.clone())
        } else {
            EditorAudio::candidate(store.clone())
        };
        let format = PreparedDevice::probe()?.format();
        let (source, observation) =
            crate::stream_fixture::open(&self.path, format.0, format.1, &adapter.budget)?;
        adapter.buffered = Some(source);
        self.source = Some(observation);
        let audio = Rc::new(RefCell::new(adapter));
        self.editor
            .ui
            .set_audio_port(Box::new(Shared(audio.clone())));
        self.audio = Some(audio);
        let raw = store
            .stage_audio(
                crate::load_verification::wav(0),
                &skin_editor_infrastructure::WavDecoder,
            )
            .map_err(error)?;
        let asset = AssetId::new(2000).unwrap();
        let component = AudioComponentId::new(1).unwrap();
        if !self.editor.ui.submit(
            &mut self.editor.session,
            vec![
                Command::RegisterAsset {
                    asset: Asset::new(
                        asset,
                        "self stable.wav".into(),
                        raw.reference(),
                        AssetKind::Audio(raw.audio_metadata().ok_or("WAV metadata")?),
                    )
                    .map_err(error)?,
                },
                Command::CreateAudioComponent {
                    component: AudioComponent::new(
                        component,
                        SoundSlotKey::new("self.stable".into()).map_err(error)?,
                        asset,
                    ),
                },
            ],
        ) {
            return Err(self.editor.ui.message.clone());
        }
        self.editor.ui.audio_component = Some(component);
        self.editor.ui.open_audio_editor();
        self.action(AudioEditorAction::Enable)?;
        self.stage = 1;
        self.since = Instant::now();
        Ok(())
    }
    // Called before the deadline frame is drawn. Beginning is recorded after
    // its setup frame, so neither edge paint is invented as stable work.
    fn finish_due(&mut self, ctx: &egui::Context) -> Result<(), String> {
        if self.interval.is_none() || Instant::now() < self.due {
            return Ok(());
        }
        let name = match self.stage {
            5 => "paused",
            7 => "closed",
            8 => "empty",
            _ => return Err("invalid stable stage".into()),
        };
        let result = self
            .interval
            .take()
            .unwrap()
            .finish(
                self.origin.elapsed(),
                self.editor.ui_passes,
                self.inputs,
                self.editor.ui.simulation_micros,
                self.background(),
            )
            .map_err(str::to_owned)?;
        let after = self.usage()?;
        let view_valid = self.visible
            && ctx.input(|i| i.viewport().visible() != Some(false))
            && self.geometry
                == [
                    ctx.viewport_rect().width(),
                    ctx.viewport_rect().height(),
                    ctx.pixels_per_point(),
                ];
        self.rows.push(format!("{{\"phase\":\"{name}\",\"span_us\":{},\"draws\":{},\"inputs\":{},\"frozen\":{},\"view_valid\":{view_valid},\"geometry\":{:?},\"before\":{:?},\"after\":{after:?},\"shape\":{:?}}}",
            result.elapsed.as_micros(), result.draws, result.inputs, result.frozen, self.geometry, self.before, self.shape()));
        if self.stage == 5 {
            self.action(AudioEditorAction::Disable)?;
            self.editor.host.shutdown();
            self.editor.session = fresh_session(&self.editor.images)?;
            self.editor.ui.target = None;
            self.editor.ui.select(&mut self.editor.session, None);
            self.editor.ui.audio_component = None;
            self.editor.ui.audio_asset = None;
            self.stage = 6;
            self.since = Instant::now();
            self.due = self.since + SETTLE;
            crate::load_verification::phase("closed_settle");
        } else {
            self.completed = true;
            crate::load_verification::phase("finished");
        }
        Ok(())
    }
    fn after_draw(&mut self, ctx: &egui::Context) -> Result<(), String> {
        let now = Instant::now();
        if self.origin.elapsed() > Duration::from_secs(if self.resources_case { 220 } else { 95 }) {
            return Err("stable window watchdog expired".into());
        }
        match self.stage {
            0 if now >= self.due => {
                if self.resources_case {
                    self.load()?;
                } else {
                    self.shape = self.shape();
                    if self.shape != [0; 3] {
                        return Err("empty editor contains sample content".into());
                    }
                    self.begin("empty", 8, ctx)?;
                }
            }
            1 => {
                let s = &self.editor.ui.audio_state;
                if let Some(e) = &s.error {
                    return Err(e.clone());
                }
                if s.enabled
                    && !s.pending_controls
                    && s.components.len() == 1
                    && s.components[0].preparation == AudioPreparationState::Ready
                    && !self.editor.host.needs_work()
                {
                    self.action(AudioEditorAction::SceneRunning(true))?;
                    self.editor.ui.paused = false;
                    self.stage = 2;
                    self.due = now + Duration::from_secs(1);
                }
            }
            2 if now >= self.due && self.background() > 0 && !self.editor.ui.audio_state.frozen => {
                self.editor.ui.paused = true;
                self.action(AudioEditorAction::SceneRunning(false))?;
                self.stage = 3;
            }
            3 if self.editor.ui.audio_state.frozen
                && !self.editor.ui.audio_state.freezing
                && !self.editor.ui.audio_needs_poll()
                && !self.editor.host.needs_work() =>
            {
                self.stage = 4;
                self.since = now;
                self.due = now + SETTLE;
                crate::load_verification::phase("paused_settle");
            }
            4 if now >= self.due => self.begin("paused", 5, ctx)?,
            6 if now >= self.due => self.begin("closed", 7, ctx)?,
            _ => {}
        }
        Ok(())
    }
    fn report(&self) -> String {
        let stats = self
            .audio
            .as_ref()
            .and_then(|a| a.borrow().closed_stats)
            .unwrap_or_default();
        let raw_final = self.source.as_ref().map_or(0, |o| o.stats().raw_bytes);
        format!(
            "{{\"schema\":1,\"scope\":\"full-editor-stable-windows\",\"production_policy\":{},\"gpu_memory_usage_hint\":{},\"resources_case\":{},\"completed\":{},\"shape\":{:?},\"rows\":[{}],\"adapter_utf8\":{:?},\"source_raw_final\":{raw_final},\"native\":{{\"callbacks\":{},\"missing_frames\":{},\"render_errors\":{},\"observation_overflows\":{},\"device_fault\":{},\"callback_resource_releases\":{},\"endpoint_returns_in_render\":{},\"endpoint_returns_outside_render\":{}}}}}",
            self.product,
            self.product && crate::product_policy::Policy::CURRENT.memory_usage,
            self.resources_case,
            self.completed,
            self.shape,
            self.rows.join(","),
            self.adapter.as_bytes(),
            stats.callbacks,
            stats.missing_frames,
            stats.render_errors,
            stats.observation_overflows,
            stats.device_fault,
            stats.callback_resource_releases,
            stats.endpoint_returns_in_render,
            stats.endpoint_returns_outside_render
        )
    }
}
impl eframe::App for Run {
    fn raw_input_hook(&mut self, _: &egui::Context, input: &mut egui::RawInput) {
        self.inputs += input.events.iter().filter(|e| physical_input(e)).count() as u64;
        if self.interval.is_some() && input.viewport().visible() == Some(false) {
            self.visible = false;
        }
    }
    fn logic(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.editor.logic(ctx, frame);
        if matches!(self.stage, 1..=3) {
            ctx.request_repaint_after(Duration::from_millis(20));
        } else {
            ctx.request_repaint_after(self.due.saturating_duration_since(Instant::now()));
        }
    }
    fn ui(&mut self, root: &mut egui::Ui, frame: &mut eframe::Frame) {
        if self.failure.is_none()
            && let Err(e) = self.finish_due(root.ctx())
        {
            self.failure = Some(e);
        }
        if !self.completed && self.failure.is_none() {
            self.editor.ui(root, frame);
            if let Err(e) = self.after_draw(root.ctx()) {
                self.failure = Some(e);
            }
            if matches!(self.stage, 1..=3) {
                root.ctx().request_repaint_after(Duration::from_millis(20));
            } else {
                root.ctx()
                    .request_repaint_after(self.due.saturating_duration_since(Instant::now()));
            }
        }
        if self.completed || self.failure.is_some() {
            let _ = std::fs::write(
                self.path.join(if self.completed {
                    "report.json"
                } else {
                    "partial-report.json"
                }),
                self.report(),
            );
            if let Some(e) = &self.failure {
                let _ = std::fs::write(self.path.join("failure.txt"), e);
            }
            root.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
    fn on_exit(&mut self) {
        self.editor.on_exit();
        if !self.completed {
            let _ = std::fs::write(self.path.join("partial-report.json"), self.report());
        }
    }
}
fn physical_input(event: &egui::Event) -> bool {
    !matches!(
        event,
        egui::Event::WindowFocused(_) | egui::Event::PointerGone
    )
}
pub fn run(destination: &str, resources_case: bool) -> Result<(), String> {
    run_policy(destination, resources_case, false)
}
pub fn run_policy(destination: &str, resources_case: bool, product: bool) -> Result<(), String> {
    let path = PathBuf::from(destination);
    if path.exists() {
        return Err("stable output must be new".into());
    }
    std::fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    let report = path.join("report.json");
    eframe::run_native(
        "Skin Editor - stable window verification",
        eframe::NativeOptions {
            renderer: eframe::Renderer::Wgpu,
            wgpu_options: if product {
                crate::product_policy::Policy::CURRENT.graphics()
            } else {
                crate::gpu_observation::configuration(false)
            },
            viewport: crate::gui_baseline::viewport(None),
            ..Default::default()
        },
        Box::new(move |c| {
            Run::new(c, path, resources_case, product)
                .map(|a| Box::new(a) as Box<dyn eframe::App>)
                .map_err(|e| std::io::Error::other(e).into())
        }),
    )
    .map_err(|e| e.to_string())?;
    if !report.is_file() {
        return Err("stable verification incomplete; retained partial report".into());
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn focus_notifications_do_not_hide_real_pointer_or_text_input() {
        assert!(!physical_input(&egui::Event::WindowFocused(false)));
        assert!(!physical_input(&egui::Event::PointerGone));
        assert!(physical_input(&egui::Event::PointerMoved(egui::pos2(
            1., 2.
        ))));
        assert!(physical_input(&egui::Event::Text("self fixture".into())));
    }
}
