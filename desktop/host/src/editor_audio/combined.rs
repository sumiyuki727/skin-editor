//! Standard graphics with the current GUI adapter, separated native voices and
//! bounded continuous background. Opt-in only; never promotes the default route.
use super::{
    gui_verification::{Observations, Shared},
    *,
};
use crate::{Editor, fresh_session, load_verification::phase};
use eframe::egui;
use skin_editor_infrastructure::{
    background_stream::StreamObservation, separated_native::NativeStats,
};
use skin_editor_ui::Presenter;
use std::{
    cell::RefCell,
    path::PathBuf,
    rc::Rc,
    sync::atomic::Ordering,
    time::{Duration, Instant},
};
mod plan;
pub(super) mod probes;
mod report;
use plan::{COUNTS, Event, Plan, SECONDS};
use probes::Probes;
fn error(e: impl std::fmt::Debug) -> String {
    format!("{e:?}")
}
fn document(n: usize) -> DocumentId {
    DocumentId::new(n as u128 + 1000).unwrap()
}
fn layer(n: usize) -> LayerId {
    LayerId::new(n as u128 + 10000).unwrap()
}
fn sound(n: usize) -> AudioComponentId {
    AudioComponentId::new(n as u128 + 1).unwrap()
}
struct VisualPending {
    at: Instant,
    draft: Option<DraftState>,
    target: PreviewTarget,
    edit: bool,
}
struct Run {
    product: bool,
    editor: Editor,
    audio: Rc<RefCell<EditorAudio>>,
    source: StreamObservation,
    observer: Option<Observations>,
    probes: Probes,
    path: PathBuf,
    origin: Instant,
    playing: Option<Instant>,
    clock: plan::StageClock,
    plan: Plan,
    pending_visual: Option<VisualPending>,
    edits_ms: Vec<f64>,
    switches_ms: Vec<f64>,
    opacities: [f32; 10],
    frame_ms: Vec<f64>,
    last_frame: Option<u64>,
    bg_start: u64,
    bg_end: u64,
    bg_previous: u64,
    frames_start: u64,
    playing_frames: u64,
    playing_us: u64,
    anchor: u64,
    quiet: u64,
    baseline_pcm: usize,
    snapshot: Option<ProjectSnapshot>,
    nodes: usize,
    peaks: [usize; 5],
    final_usage: [usize; 5],
    max_voices: usize,
    max_supply_gap: u64,
    stats: NativeStats,
    controls: [bool; 6],
    continuity: Vec<[u64; 3]>,
    adapter: String,
    format: (u32, u16),
    period: u32,
    geometry: Option<[f32; 3]>,
    view_valid: bool,
    physical_inputs: u64,
    completed: bool,
    failure: Option<String>,
}
impl Run {
    fn new(c: &eframe::CreationContext<'_>, path: PathBuf, product: bool) -> Result<Self, String> {
        let state = c.wgpu_render_state.as_ref().ok_or("wgpu unavailable")?;
        let mut editor = Editor::with_initial_content(
            &c.egui_ctx,
            state,
            false,
            skin_editor_rendering::SourceUpload::QueueWrite,
            false,
        )?;
        editor.host.enable_trace();
        crate::combined_verification::fixture(&mut editor, &path)?;
        let device = PreparedDevice::probe()?;
        let format = device.format();
        let mut adapter = if product {
            crate::product_policy::Policy::CURRENT.audio(editor.images.blobs().clone())
        } else {
            EditorAudio::candidate(editor.images.blobs().clone())
        };
        let (background, source) =
            crate::stream_fixture::open(&path, format.0, format.1, &adapter.budget)?;
        adapter.buffered = Some(background);
        let audio = Rc::new(RefCell::new(adapter));
        editor.ui.set_audio_port(Box::new(Shared(audio.clone())));
        editor.ui.audio_component = Some(sound(0));
        editor.ui.open_audio_editor();
        editor
            .ui
            .audio_command(&editor.session, AudioEditorAction::MonitorVolume(500));
        editor
            .ui
            .audio_command(&editor.session, AudioEditorAction::Enable);
        let mut a = audio.borrow_mut();
        let n = a
            .native
            .as_mut()
            .ok_or("separated native output unavailable")?;
        let period = n.period_frames();
        let observer = Observations::with_capacity(
            n.take_first_observer().ok_or("first-copy reader taken")?,
            4096,
        )?;
        drop(a);
        let now = Instant::now();
        Ok(Self {
            product,
            editor,
            audio,
            source,
            observer: Some(observer),
            probes: Probes::default(),
            path,
            origin: now,
            playing: None,
            clock: plan::StageClock::default(),
            plan: Plan::default(),
            pending_visual: None,
            edits_ms: Vec::with_capacity(300),
            switches_ms: Vec::with_capacity(20),
            opacities: [0.9; 10],
            frame_ms: Vec::with_capacity(25000),
            last_frame: None,
            bg_start: 0,
            bg_end: 0,
            bg_previous: 0,
            frames_start: 0,
            playing_frames: 0,
            playing_us: 0,
            anchor: 0,
            quiet: 0,
            baseline_pcm: 0,
            snapshot: None,
            nodes: 0,
            peaks: [0; 5],
            final_usage: [0; 5],
            max_voices: 0,
            max_supply_gap: 0,
            stats: NativeStats::default(),
            controls: [false; 6],
            continuity: Vec::with_capacity(300),
            adapter: format!("{:?}", state.adapter.get_info()),
            format,
            period,
            geometry: None,
            view_valid: true,
            physical_inputs: 0,
            completed: false,
            failure: None,
        })
    }
    fn advance(&mut self, stage: u8) {
        self.clock.enter(stage, self.origin.elapsed());
    }
    fn usage(&self) -> Result<[usize; 5], String> {
        Ok([
            self.audio.borrow().budget.usage().bytes,
            self.editor.images.blobs().usage().map_err(error)?.bytes,
            self.editor.images.rgba_bytes(),
            self.editor.host.live_targets(),
            self.editor.host.source_usage().1,
        ])
    }
    fn action(&mut self, action: AudioEditorAction) -> Result<(), String> {
        if let AudioEditorAction::SceneRunning(goal) = action {
            self.editor.ui.paused = !goal;
        }
        self.editor.ui.audio_command(&self.editor.session, action);
        if let Some(e) = &self.editor.ui.audio_state.error {
            return Err(e.clone());
        }
        Ok(())
    }
    fn draft(&mut self, action: DraftAction) -> Result<(), String> {
        if !self.editor.ui.draft(&mut self.editor.session, action) {
            return Err(self.editor.ui.message.clone());
        }
        Ok(())
    }
    fn draft_state(&self) -> Result<DraftState, String> {
        Ok(self
            .editor
            .session
            .draft_info(self.editor.session.project_info().session_id)
            .map_err(error)?
            .ok_or("missing load draft")?
            .state)
    }
    fn ready(&mut self) -> Result<bool, String> {
        let s = &self.editor.ui.audio_state;
        if let Some(e) = &s.error {
            return Err(e.clone());
        }
        let failed = s
            .components
            .iter()
            .find(|c| matches!(c.preparation, AudioPreparationState::Failed(_)));
        if let Some(c) = failed {
            if matches!(&c.preparation,AudioPreparationState::Failed(e) if e.contains("Busy")) {
                let id = c.component;
                self.action(AudioEditorAction::Retry(id))?;
                return Ok(false);
            }
            return Err(format!("preparation failure: {:?}", c.preparation));
        }
        Ok(s.enabled
            && !s.pending_controls
            && s.source == Some(self.editor.session.skin_audio_source())
            && s.components.len() == 16
            && s.components.iter().all(|c| {
                c.preparation == AudioPreparationState::Ready && c.effective == Some(c.desired)
            }))
    }
    fn background(&self) -> u64 {
        self.editor
            .ui
            .audio_state
            .output
            .as_ref()
            .map_or(0, |o| o.copied_background)
    }
    fn trigger(
        &mut self,
        lane: Channel,
        component: AudioComponentId,
        measured: bool,
    ) -> Result<(), String> {
        let at = Instant::now();
        let id = if lane == Channel::Audition {
            self.action(AudioEditorAction::Audition(component))?;
            self.audio
                .borrow()
                .last_trigger_request
                .ok_or("audition request missing")?
        } else {
            self.audio
                .borrow_mut()
                .client
                .as_mut()
                .ok_or("client closed")?
                .trigger_selected(lane, component)
                .map_err(error)?
        };
        self.probes.begin(
            id,
            lane,
            at,
            crate::edit_verification::source(&self.editor.session),
            measured,
        )
    }
    fn collect(&mut self) -> Result<(), String> {
        let mut a = self.audio.borrow_mut();
        if let Some(c) = &mut a.client {
            let receipts: Vec<_> = std::iter::from_fn(|| c.poll()).collect();
            for receipt in receipts {
                let result = receipt.result.as_ref().map_err(error)?;
                if let SupplyOutcome::Triggered(r) = result {
                    self.probes.receipt(receipt.id, *r)?;
                }
                a.receive(receipt.id, &receipt.result);
            }
        }
        drop(a);
        if let Some(o) = &self.observer {
            if o.overflow.load(Ordering::Acquire) {
                return Err("first-copy observation trace overflow".into());
            }
            self.probes
                .observe(&o.rows.lock().map_err(error)?, Instant::now())?;
        }
        Ok(())
    }
    fn visual(&mut self, edit: bool) -> Result<(), String> {
        if self.pending_visual.is_some() {
            return Err("graphics command overlapped unpublished material".into());
        }
        self.pending_visual = Some(VisualPending {
            at: Instant::now(),
            draft: Some(self.draft_state()?),
            target: self.editor.ui.target.ok_or("target missing")?,
            edit,
        });
        Ok(())
    }
    fn observe_visual(&mut self) -> Result<(), String> {
        if let Some(p) = &self.pending_visual
            && let Some(v) = self.editor.host.current()
            && v.frame.source.context.draft == p.draft
            && v.frame.source.context.target == p.target
        {
            let ms = p.at.elapsed().as_secs_f64() * 1000.;
            if p.edit {
                self.edits_ms.push(ms);
            } else {
                self.switches_ms.push(ms);
            }
            self.pending_visual = None;
        }
        if self.editor.host.trace_overflow {
            return Err("preparation trace overflow".into());
        }
        Ok(())
    }
    fn event(&mut self, e: Event) -> Result<(), String> {
        match e {
            Event::Scene | Event::Audition => {
                if !self.ready()? {
                    return Err("scheduled sound was not ready".into());
                }
                let lane = if e == Event::Scene {
                    Channel::Scene
                } else {
                    Channel::Audition
                };
                let n = if lane == Channel::Scene {
                    (self.plan.counts[0] - 1) % 16
                } else {
                    0
                };
                self.trigger(lane, sound(n), true)?;
            }
            Event::Edit => {
                self.visual(true)?;
                let n = self.plan.counts[2] - 1;
                self.opacities[self.plan.counts[3] % 10] = 0.5 + (n % 40) as f32 / 100.;
                let mut commands: Vec<_> = self
                    .opacities
                    .iter()
                    .enumerate()
                    .map(|(d, o)| Command::SetLayerProperties {
                        document: document(d),
                        layer: layer(d * 50),
                        opacity: Opacity::new(*o).unwrap(),
                        visible: true,
                    })
                    .collect();
                commands.push(Command::SetAudioProperties {
                    component: sound(0),
                    gain: Gain::new(0.4 + (n % 50) as f64 / 100.).unwrap(),
                    visible: true,
                });
                self.draft(DraftAction::Replace {
                    expected: self.draft_state()?,
                    commands,
                })?;
                self.pending_visual.as_mut().unwrap().draft = Some(self.draft_state()?);
                self.continuity.push([n as u64, self.background(), 0]);
            }
            Event::Switch => {
                self.editor.ui.target =
                    Some(PreviewTarget::Document(document(self.plan.counts[3] % 10)));
                self.visual(false)?;
            }
        }
        Ok(())
    }
    fn step(&mut self) -> Result<(), String> {
        self.clock
            .check(self.origin.elapsed())
            .map_err(str::to_owned)?;
        if self.origin.elapsed() > Duration::from_secs(390) {
            return Err("combined watchdog expired".into());
        }
        let usage = self.usage()?;
        for (p, u) in self.peaks.iter_mut().zip(usage) {
            *p = (*p).max(u);
        }
        self.peaks[0] = self.peaks[0].max(self.audio.borrow().budget.usage().peak_bytes);
        self.peaks[1] = self.peaks[1].max(
            self.editor
                .images
                .blobs()
                .usage()
                .map_err(error)?
                .peak_bytes,
        );
        self.peaks[2] = self.peaks[2].max(self.editor.images.rgba_peak_bytes());
        self.max_voices = self
            .max_voices
            .max(self.editor.ui.audio_state.voices.iter().sum());
        if let Some(c) = &self.audio.borrow().client {
            self.max_supply_gap = self
                .max_supply_gap
                .max(c.status().map_err(error)?.max_supply_gap_micros);
        }
        self.observe_visual()?;
        let ready = self.ready()?;
        let bg = self.background();
        if self.clock.stage >= 3 && self.clock.stage < 20 {
            if bg < self.bg_previous {
                return Err("background regressed without explicit seek".into());
            }
            self.bg_previous = bg;
        }
        if self.clock.stage == 3
            && ready
            && let Some(row) = self.continuity.last_mut()
            && row[2] == 0
        {
            row[2] = bg;
        }
        let age = self
            .clock
            .age(self.origin.elapsed())
            .map_err(str::to_owned)?;
        match self.clock.stage {
            0 if ready && !self.editor.host.needs_work() && self.editor.host.publications > 0 => {
                self.nodes = self.editor.session.history_len();
                self.draft(DraftAction::Begin {
                    expected: self.editor.session.project_info().state,
                })?;
                self.advance(1);
            }
            1 if ready && !self.editor.host.needs_work() => {
                self.action(AudioEditorAction::SceneRunning(true))?;
                self.advance(2);
            }
            2 if !self.editor.ui.audio_state.frozen
                && !self.editor.ui.audio_state.pending_controls =>
            {
                self.bg_start = bg;
                self.bg_previous = bg;
                self.frames_start = self.audio.borrow().native.as_ref().unwrap().stats().frames;
                self.playing = Some(Instant::now());
                phase("playing");
                self.advance(3);
            }
            3 => {
                let elapsed = self.playing.unwrap().elapsed();
                // At most four scheduled actions per GUI pass; an overdue host
                // fails Plan instead of silently replaying a missed burst.
                for _ in 0..4 {
                    match self
                        .plan
                        .take(elapsed.as_micros() as u64)
                        .map_err(str::to_owned)?
                    {
                        Some(e) => self.event(e)?,
                        None => break,
                    }
                }
                let frames =
                    self.audio.borrow().native.as_ref().unwrap().stats().frames - self.frames_start;
                if elapsed >= Duration::from_secs(SECONDS)
                    && bg - self.bg_start >= u64::from(self.format.0) * SECONDS
                    && frames >= u64::from(self.format.0) * SECONDS
                {
                    if self.plan.counts != COUNTS
                        || !self.probes.empty()
                        || self.edits_ms.len() != 300
                        || self.switches_ms.len() != 20
                    {
                        return Err("incomplete normal schedule/publications/probes".into());
                    }
                    self.playing_us = elapsed.as_micros() as u64;
                    self.playing_frames = frames;
                    self.bg_end = bg;
                    phase("controls");
                    self.action(AudioEditorAction::SceneRunning(false))?;
                    self.advance(4);
                }
            }
            4 if ready
                && self.editor.ui.audio_state.frozen
                && !self.editor.ui.audio_state.freezing =>
            {
                self.anchor = bg;
                self.draft(DraftAction::Commit {
                    expected: self.draft_state()?,
                })?;
                if self.editor.session.history_len() != self.nodes + 1 {
                    return Err("300 updates did not commit exactly once".into());
                }
                self.controls[0] = true;
                self.advance(5);
            }
            5 if ready && self.editor.ui.audio_state.voices == [0, 0] => {
                self.trigger(Channel::Audition, sound(15), false)?;
                self.advance(6);
            }
            6 if self.probes.empty() && self.editor.ui.audio_state.voices == [0, 1] => {
                self.controls[1] = bg == self.anchor;
                self.action(AudioEditorAction::StopAudition)?;
                self.advance(7);
            }
            7 if !self.editor.ui.audio_state.pending_controls
                && self.editor.ui.audio_state.voices == [0, 0] =>
            {
                self.quiet = self.editor.ui.audio_state.submitted_nonzero_samples;
                self.advance(8);
            }
            8 if age >= Duration::from_millis(80) => {
                self.controls[2] = self.editor.ui.audio_state.submitted_nonzero_samples
                    == self.quiet
                    && bg == self.anchor;
                self.action(AudioEditorAction::SceneRunning(true))?;
                self.advance(9);
            }
            9 if !self.editor.ui.audio_state.frozen && bg > self.anchor => {
                self.controls[3] = true;
                self.action(AudioEditorAction::SceneRunning(false))?;
                self.advance(10);
            }
            10 if ready
                && self.editor.ui.audio_state.frozen
                && !self.editor.ui.audio_state.freezing =>
            {
                self.anchor = bg;
                self.baseline_pcm = self.audio.borrow().budget.usage().bytes;
                self.snapshot = Some(self.editor.session.snapshot());
                self.nodes = self.editor.session.history_len();
                self.draft(DraftAction::Begin {
                    expected: self.editor.session.project_info().state,
                })?;
                self.draft(DraftAction::Replace {
                    expected: self.draft_state()?,
                    commands: vec![Command::SetAudioProperties {
                        component: sound(15),
                        gain: Gain::new(0.51).unwrap(),
                        visible: true,
                    }],
                })?;
                self.advance(11);
            }
            11 if ready => {
                self.trigger(Channel::Audition, sound(15), false)?;
                self.advance(12);
            }
            12 if self.probes.empty() && self.editor.ui.audio_state.voices == [0, 1] => {
                let d = self.draft_state()?;
                if self
                    .audio
                    .borrow()
                    .last_triggered
                    .is_none_or(|(_, r)| r.origin.draft != Some((d.id.get(), d.revision)))
                {
                    return Err("cleanup did not start a draft voice".into());
                }
                self.editor.ui.cancel(&mut self.editor.session);
                self.advance(13);
            }
            13 if ready
                && self.editor.ui.audio_state.voices == [0, 0]
                && self.audio.borrow().retained_drafts().is_empty()
                && self.audio.borrow().budget.usage().bytes <= self.baseline_pcm =>
            {
                self.controls[4] = self.snapshot.as_ref() == Some(&self.editor.session.snapshot())
                    && self.editor.session.history_len() == self.nodes
                    && bg == self.anchor;
                self.trigger(Channel::Audition, sound(15), false)?;
                self.advance(14);
            }
            13 if age >= Duration::from_secs(1) => {
                return Err(
                    "draft cleanup exceeded diagnostic guard before natural voice end".into(),
                );
            }
            14 if self.probes.empty() && self.editor.ui.audio_state.voices == [0, 1] => {
                self.editor
                    .ui
                    .navigation(&mut self.editor.session, HistoryAction::Undo);
                self.advance(15);
            }
            15 if ready && self.editor.ui.audio_state.voices == [0, 0] => {
                self.editor.ui.navigation(
                    &mut self.editor.session,
                    HistoryAction::Redo { child: None },
                );
                self.advance(16);
            }
            15 if age >= Duration::from_secs(1) => {
                return Err("history old voice not retired before natural end".into());
            }
            16 if ready
                && self.editor.ui.audio_state.voices == [0, 0]
                && self.audio.borrow().retained_drafts().is_empty() =>
            {
                self.controls[5] = self.snapshot.as_ref().unwrap().project
                    == self.editor.session.snapshot().project
                    && bg == self.anchor
                    && self.editor.session.history_len() == self.nodes;
                self.action(AudioEditorAction::Disable)?;
                self.stats = self
                    .audio
                    .borrow()
                    .closed_stats
                    .ok_or("missing native stopped ownership")?;
                self.observer.take();
                self.editor.host.shutdown();
                self.snapshot.take();
                self.editor.session = fresh_session(&self.editor.images)?;
                self.editor.ui.target = None;
                self.editor.ui.select(&mut self.editor.session, None);
                self.editor.ui.audio_component = None;
                self.editor.ui.audio_asset = None;
                phase("closed");
                self.advance(20);
            }
            20 if age >= Duration::from_secs(2) => {
                self.final_usage = self.usage()?;
                self.completed = true;
                phase("finished");
                self.advance(21);
            }
            _ => {}
        }
        Ok(())
    }
}
impl eframe::App for Run {
    fn raw_input_hook(&mut self, _: &egui::Context, input: &mut egui::RawInput) {
        self.physical_inputs += input
            .events
            .iter()
            .filter(|e| !matches!(e, egui::Event::WindowFocused(_) | egui::Event::PointerGone))
            .count() as u64;
        // The script supplies focus to exercise draft editing independent of OS
        // focus grants. Actual physical events remain counted, never hidden.
        input.focused = true;
        for v in input.viewports.values_mut() {
            v.focused = Some(true);
        }
    }
    fn logic(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if self.failure.is_none() && self.clock.stage < 20 {
            if let Err(e) = self.collect() {
                self.failure = Some(e);
            }
            self.editor.logic(ctx, frame);
        } else {
            let _ = self.editor.host.poll();
        }
        ctx.request_repaint_after(Duration::from_millis(if self.clock.stage >= 20 {
            50
        } else {
            16
        }));
    }
    fn ui(&mut self, root: &mut egui::Ui, frame: &mut eframe::Frame) {
        let ctx = root.ctx().clone();
        let current = [
            ctx.viewport_rect().width(),
            ctx.viewport_rect().height(),
            ctx.pixels_per_point(),
        ];
        if self.clock.stage == 3 {
            if self.geometry.is_none() {
                self.geometry = Some(current);
            }
            self.view_valid &= self.geometry == Some(current)
                && ctx.input(|i| i.viewport().visible() != Some(false));
            let n = ctx.cumulative_frame_nr();
            if self.last_frame != Some(n) {
                self.last_frame = Some(n);
                if let Some(v) = frame.info().cpu_usage {
                    if !v.is_finite() || v < 0. || self.frame_ms.len() == 25000 {
                        self.failure = Some("invalid/unbounded framework CPU frame trace".into());
                    } else {
                        self.frame_ms.push(f64::from(v) * 1000.);
                    }
                }
            }
        }
        if self.failure.is_none()
            && let Err(e) = self.step()
        {
            self.failure = Some(e);
        }
        if self.clock.stage < 20 && self.failure.is_none() {
            self.editor.ui(root, frame);
        } else {
            root.label("P2 current-route load: resources closed.");
        }
        if self.failure.is_some() || self.completed {
            let name = if self.completed {
                "report.json"
            } else {
                "partial-report.json"
            };
            if let Err(e) = std::fs::write(self.path.join(name), self.report()) {
                self.failure = Some(e.to_string());
            }
            if let Some(e) = &self.failure {
                let _ = std::fs::write(
                    self.path.join("failure.txt"),
                    format!("stage {}: {e}", self.clock.stage),
                );
            }
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
    fn on_exit(&mut self) {
        self.observer.take();
        self.editor.on_exit();
    }
}
pub fn run(destination: &str) -> Result<(), String> {
    run_policy(destination, false)
}
pub fn run_policy(destination: &str, product: bool) -> Result<(), String> {
    let path = PathBuf::from(destination);
    if path.exists() {
        return Err("combined output must be new".into());
    }
    std::fs::create_dir_all(&path).map_err(error)?;
    let report = path.join("report.json");
    phase("loading");
    eframe::run_native(
        "Skin Editor - current separated audio 300s load",
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
            Run::new(c, path, product)
                .map(|r| Box::new(r) as Box<dyn eframe::App>)
                .map_err(|e| std::io::Error::other(e).into())
        }),
    )
    .map_err(error)?;
    if !report.is_file() {
        return Err("current-route combined incomplete; partial report retained".into());
    }
    Ok(())
}
