//! Full per-project owner replacement inside one persistent eframe GPU shell.
use super::{
    combined::probes::Probes,
    gui_verification::{Observations, Shared},
    *,
};
use crate::{Editor, fresh_session, load_verification::phase};
use eframe::egui;
use skin_editor_infrastructure::{
    ByteObservation,
    background_stream::{BufferedBackground, StreamObservation},
    separated_native::NativeStats,
};
use skin_editor_ui::Presenter;
use std::{
    cell::RefCell,
    fs::File,
    path::PathBuf,
    rc::Rc,
    sync::atomic::Ordering,
    time::{Duration, Instant},
};
mod plan;
mod report;
use plan::Mode;
fn error(e: impl std::fmt::Debug) -> String {
    format!("{e:?}")
}
fn sound(i: usize) -> AudioComponentId {
    AudioComponentId::new(i as u128 + 1).unwrap()
}
struct Counters {
    pcm: PcmBudget,
    raw: ByteObservation,
    rgba: ByteObservation,
    background: StreamObservation,
}
impl Counters {
    fn bytes(&self) -> [usize; 3] {
        [
            self.pcm.usage().bytes,
            self.raw.usage().bytes,
            self.rgba.usage().bytes,
        ]
    }
}
struct Round {
    editor: Box<Editor>,
    audio: Rc<RefCell<EditorAudio>>,
    observer: Option<Observations>,
    probes: Probes,
    counters: Counters,
    opened: Instant,
    ready_ms: f64,
    session: u128,
    target: usize,
    counts: [usize; 3],
    playing: Option<Instant>,
    bg_start: u64,
    bg_previous: u64,
    bg_end: u64,
    frame_start: u64,
    playing_frames: u64,
    playing_us: u64,
    max_late_us: u64,
    pending_image: Option<DraftState>,
    images_published: usize,
    frame_ms: Vec<f64>,
    last_frame: Option<u64>,
    peaks: [usize; 5],
    max_voices: usize,
    stats: NativeStats,
    workers: [bool; 3],
    pending_at_close: bool,
    voices_at_close: [usize; 2],
    closed_started: Option<Instant>,
    close_ms: f64,
    geometry: Option<[f32; 3]>,
    view_valid: bool,
}
impl Round {
    fn new(
        ctx: &egui::Context,
        state: &eframe::egui_wgpu::RenderState,
        path: &std::path::Path,
        index: usize,
        format: (u32, u16),
        product: bool,
    ) -> Result<Self, String> {
        let opened = Instant::now();
        let mut editor = Box::new(Editor::with_initial_content(
            ctx,
            state,
            false,
            skin_editor_rendering::SourceUpload::QueueWrite,
            false,
        )?);
        if index == 0 {
            crate::combined_verification::fixture(&mut editor, path)?;
        } else {
            crate::combined_verification::reopen_fixture(&mut editor, path)?;
        }
        let target = index % 10;
        editor.ui.target = Some(PreviewTarget::Document(
            DocumentId::new(target as u128 + 1000).unwrap(),
        ));
        let s = editor.session.snapshot();
        if s.project
            .assets()
            .iter()
            .filter(|a| matches!(a.kind(), AssetKind::RasterImage(_)))
            .count()
            != 100
            || s.project.documents().len() != 10
            || s.project
                .documents()
                .map(|d| d.layers().iter().len())
                .sum::<usize>()
                != 500
            || s.project.audio_components().len() != 16
        {
            return Err("wrong full-cycle fixture".into());
        }
        drop(s);
        let mut a = if product {
            crate::product_policy::Policy::CURRENT.audio(editor.images.blobs().clone())
        } else {
            EditorAudio::candidate(editor.images.blobs().clone())
        };
        let source = BufferedBackground::open(
            File::open(path.join("self-background.wav")).map_err(error)?,
            &a.budget,
        )
        .map_err(error)?;
        let observation = source.observation();
        a.buffered = Some(source);
        let counters = Counters {
            pcm: a.budget.clone(),
            raw: editor.images.blobs().observe_bytes(),
            rgba: editor.images.observe_rgba(),
            background: observation,
        };
        let audio = Rc::new(RefCell::new(a));
        editor.ui.set_audio_port(Box::new(Shared(audio.clone())));
        editor.ui.audio_component = Some(sound(0));
        editor.ui.open_audio_editor();
        editor
            .ui
            .audio_command(&editor.session, AudioEditorAction::MonitorVolume(500));
        editor
            .ui
            .audio_command(&editor.session, AudioEditorAction::Enable);
        if audio.borrow().format != format {
            return Err("device format changed during resource cycles".into());
        }
        let observer = Observations::with_capacity(
            audio
                .borrow_mut()
                .native
                .as_mut()
                .ok_or("native missing")?
                .take_first_observer()
                .ok_or("reader taken")?,
            192,
        )?;
        let session = editor.session.project_info().session_id.get();
        Ok(Self {
            editor,
            audio,
            observer: Some(observer),
            probes: Probes::default(),
            counters,
            opened,
            ready_ms: 0.,
            session,
            target,
            counts: [0; 3],
            playing: None,
            bg_start: 0,
            bg_previous: 0,
            bg_end: 0,
            frame_start: 0,
            playing_frames: 0,
            playing_us: 0,
            max_late_us: 0,
            pending_image: None,
            images_published: 0,
            frame_ms: Vec::with_capacity(1500),
            last_frame: None,
            peaks: [0; 5],
            max_voices: 0,
            stats: NativeStats::default(),
            workers: [false; 3],
            pending_at_close: false,
            voices_at_close: [0; 2],
            closed_started: None,
            close_ms: 0.,
            geometry: None,
            view_valid: true,
        })
    }
    fn usage(&self) -> [usize; 5] {
        let _ = self.editor.images.rgba_bytes();
        let b = self.counters.bytes();
        [
            b[0],
            b[1],
            b[2],
            self.editor.host.live_targets(),
            self.editor.host.source_usage().1,
        ]
    }
    fn observe(&mut self) -> Result<(), String> {
        let u = self.usage();
        for (p, v) in self.peaks.iter_mut().zip(u) {
            *p = (*p).max(v);
        }
        self.peaks[0] = self.peaks[0].max(self.counters.pcm.usage().peak_bytes);
        self.peaks[1] = self.peaks[1].max(self.counters.raw.usage().peak_bytes);
        self.peaks[2] = self.peaks[2].max(self.counters.rgba.usage().peak_bytes);
        self.max_voices = self
            .max_voices
            .max(self.editor.ui.audio_state.voices.iter().sum());
        if let Some(p) = self.pending_image
            && let Some(v) = self.editor.host.current()
            && v.frame.source.context.draft == Some(p)
        {
            self.images_published += 1;
            self.pending_image = None;
        }
        Ok(())
    }
    fn action(&mut self, a: AudioEditorAction) -> Result<(), String> {
        if let AudioEditorAction::SceneRunning(g) = a {
            self.editor.ui.paused = !g;
        }
        self.editor.ui.audio_command(&self.editor.session, a);
        if let Some(e) = &self.editor.ui.audio_state.error {
            return Err(e.clone());
        }
        Ok(())
    }
    fn ready(&mut self) -> Result<bool, String> {
        let s = &self.editor.ui.audio_state;
        if let Some(e) = &s.error {
            return Err(e.clone());
        }
        if let Some(c) = s
            .components
            .iter()
            .find(|c| matches!(c.preparation, AudioPreparationState::Failed(_)))
        {
            if matches!(&c.preparation,AudioPreparationState::Failed(e) if e.contains("Busy")) {
                let id = c.component;
                self.action(AudioEditorAction::Retry(id))?;
                return Ok(false);
            }
            return Err(format!("preparation failed: {:?}", c.preparation));
        }
        Ok(s.enabled
            && !s.pending_controls
            && s.source == Some(self.editor.session.skin_audio_source())
            && s.components.len() == 16
            && s.components.iter().all(|c| {
                c.preparation == AudioPreparationState::Ready && c.effective == Some(c.desired)
            }))
    }
    fn draft_state(&self) -> Result<DraftState, String> {
        Ok(self
            .editor
            .session
            .draft_info(self.editor.session.project_info().session_id)
            .map_err(error)?
            .ok_or("draft missing")?
            .state)
    }
    fn draft(&mut self, a: DraftAction) -> Result<(), String> {
        if !self.editor.ui.draft(&mut self.editor.session, a) {
            return Err(self.editor.ui.message.clone());
        }
        Ok(())
    }
    fn image(&mut self, opacity: f32) -> Result<(), String> {
        if self.pending_image.is_some() {
            return Err("image update overtook publication".into());
        }
        self.draft(DraftAction::Replace {
            expected: self.draft_state()?,
            commands: vec![Command::SetLayerProperties {
                document: DocumentId::new(self.target as u128 + 1000).unwrap(),
                layer: LayerId::new((self.target * 50) as u128 + 10000).unwrap(),
                opacity: Opacity::new(opacity).unwrap(),
                visible: true,
            }],
        })?;
        self.pending_image = Some(self.draft_state()?);
        Ok(())
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
                .ok_or("supply missing")?
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
            for r in receipts {
                if let SupplyOutcome::Triggered(t) = r.result.as_ref().map_err(error)? {
                    self.probes.receipt(r.id, *t)?;
                }
                a.receive(r.id, &r.result);
            }
        }
        drop(a);
        if let Some(o) = &self.observer {
            if o.overflow.load(Ordering::Acquire) {
                return Err("cycle first-copy overflow".into());
            }
            self.probes
                .observe(&o.rows.lock().map_err(error)?, Instant::now())?;
        }
        Ok(())
    }
    fn background(&self) -> u64 {
        self.editor
            .ui
            .audio_state
            .output
            .as_ref()
            .map_or(0, |o| o.copied_background)
    }
    fn begin_close(&mut self) -> Result<(), String> {
        self.closed_started = Some(Instant::now());
        self.voices_at_close = self.editor.ui.audio_state.voices;
        if self.voices_at_close.contains(&0) {
            return Err("close did not start with both channels live".into());
        }
        self.image(0.731)?;
        let mut context = self
            .editor
            .host
            .current()
            .ok_or("no preview before close")?
            .frame
            .source
            .context;
        context.draft = Some(self.draft_state()?);
        self.editor
            .session
            .request_preview(&mut self.editor.host, context)
            .map_err(error)?;
        self.pending_at_close = self.editor.host.preparation_pending();
        if !self.pending_at_close {
            return Err("no pending graphics work at close".into());
        }
        self.action(AudioEditorAction::Disable)?;
        self.stats = self
            .audio
            .borrow()
            .closed_stats
            .ok_or("native ownership not returned")?;
        self.observer.take();
        self.editor.importer.close();
        self.editor.host.shutdown();
        self.workers = [
            self.editor.host.preparation_closed() && self.editor.host.shutdown_succeeded(),
            self.editor.importer.closed(),
            self.audio.borrow().client.is_none() && self.audio.borrow().native.is_none(),
        ];
        self.editor.session = fresh_session(&self.editor.images)?;
        self.editor.ui.target = None;
        self.editor.ui.select(&mut self.editor.session, None);
        self.editor.ui.audio_component = None;
        self.editor.ui.audio_asset = None;
        Ok(())
    }
}
struct Run {
    product: bool,
    state: eframe::egui_wgpu::RenderState,
    path: PathBuf,
    fixture: PathBuf,
    mode: Mode,
    format: (u32, u16),
    period: u32,
    adapter: String,
    active: Option<Round>,
    retired: Option<Counters>,
    phase: u8,
    entered: Instant,
    started: Instant,
    index: usize,
    rows: Vec<String>,
    completed: bool,
    failure: Option<String>,
    physical_inputs: u64,
    phase_step: u64,
}
impl Run {
    fn new(
        c: &eframe::CreationContext<'_>,
        path: PathBuf,
        smoke: bool,
        product: bool,
    ) -> Result<Self, String> {
        let state = c.wgpu_render_state.clone().ok_or("wgpu unavailable")?;
        let format = PreparedDevice::probe()?.format();
        let fixture = path.join("fixture");
        std::fs::create_dir(&fixture).map_err(error)?;
        crate::stream_fixture::write(&fixture, format.0, format.1)?;
        let now = Instant::now();
        phase("initial_settle");
        Ok(Self {
            product,
            adapter: format!("{:?}", state.adapter.get_info()),
            state,
            path,
            fixture,
            mode: Mode { smoke },
            format,
            period: 0,
            active: None,
            retired: None,
            phase: 0,
            entered: now,
            started: now,
            index: 0,
            rows: Vec::with_capacity(20),
            completed: false,
            failure: None,
            physical_inputs: 0,
            phase_step: 0,
        })
    }
    fn publish_phase(&mut self, name: &str) {
        self.phase_step += 1;
        let value = format!("{{\"step\":{},\"phase\":\"{name}\"}}", self.phase_step);
        if let Err(e) = std::fs::write(self.path.join("phase.json"), value) {
            self.failure = Some(format!("phase identity write: {e}"));
        }
        phase(name);
    }
    fn advance(&mut self, p: u8) {
        self.phase = p;
        self.entered = Instant::now();
        let name = match p {
            1 => "initial_closed".into(),
            2 => format!("cycle_{:02}_loading", self.index + 1),
            3 => format!("cycle_{:02}_playing", self.index + 1),
            4 | 5 => format!("cycle_{:02}_tail", self.index + 1),
            6 => format!("cycle_{:02}_closing", self.index + 1),
            7 => format!("cycle_{:02}_closed_settle", self.index + 1),
            8 => format!("cycle_{:02}_closed", self.index + 1),
            9 => "final_settle".into(),
            10 => "final_closed".into(),
            _ => "finished".into(),
        };
        self.publish_phase(&name);
    }
    fn step(&mut self, ctx: &egui::Context) -> Result<(), String> {
        if self.started.elapsed() > Duration::from_secs(800) {
            return Err("resource-cycle outer watchdog".into());
        }
        let age = self.entered.elapsed();
        // Check before taking a success branch; never reuse a preceding phase's age.
        if matches!(self.phase, 2 | 6) && age > Duration::from_secs(30) {
            return Err("open/close watchdog".into());
        }
        if matches!(self.phase, 4 | 5) && age > Duration::from_secs(1) {
            return Err("tail voice guard expired before natural end".into());
        }
        if let Some(r) = &mut self.active {
            r.observe()?;
        }
        match self.phase {
            0 if age >= self.mode.final_settle() => self.advance(1),
            1 if age >= self.mode.baseline() => {
                self.advance(2);
                self.active = Some(Round::new(
                    ctx,
                    &self.state,
                    &self.fixture,
                    self.index,
                    self.format,
                    self.product,
                )?);
            }
            2 => {
                let r = self.active.as_mut().unwrap();
                if r.ready()? && !r.editor.host.needs_work() && r.editor.host.publications > 0 {
                    if r.editor.session.skin_audio_source().draft.is_none() {
                        r.draft(DraftAction::Begin {
                            expected: r.editor.session.project_info().state,
                        })?;
                    } else if r.editor.ui.audio_state.frozen {
                        r.action(AudioEditorAction::SceneRunning(true))?;
                    } else {
                        r.ready_ms = r.opened.elapsed().as_secs_f64() * 1000.;
                        r.playing = Some(Instant::now());
                        r.bg_start = r.background();
                        r.bg_previous = r.bg_start;
                        r.frame_start = r.audio.borrow().native.as_ref().unwrap().stats().frames;
                        self.period = r.audio.borrow().native.as_ref().unwrap().period_frames();
                        self.advance(3);
                    }
                }
            }
            3 => {
                let r = self.active.as_mut().unwrap();
                let elapsed = r.playing.unwrap().elapsed();
                let bg = r.background();
                if bg < r.bg_previous {
                    return Err("background reset during cycle edits".into());
                }
                r.bg_previous = bg;
                for _ in 0..3 {
                    let Some((lane, due)) = plan::next(self.mode, r.counts) else {
                        break;
                    };
                    if elapsed.as_micros() < u128::from(due) {
                        break;
                    }
                    let late = (elapsed.as_micros() as u64) - due;
                    r.max_late_us = r.max_late_us.max(late);
                    if late > 80_000 {
                        return Err("cycle schedule missed deadline; no catchup".into());
                    }
                    r.counts[lane] += 1;
                    if lane < 2 {
                        if !r.ready()? {
                            return Err("cycle trigger not ready".into());
                        }
                        r.trigger(
                            if lane == 0 {
                                Channel::Scene
                            } else {
                                Channel::Audition
                            },
                            sound(if lane == 0 { (r.counts[0] - 1) % 16 } else { 0 }),
                            true,
                        )?;
                    } else {
                        r.image(0.45 + (r.counts[2] - 1) as f32 * 0.02)?;
                    }
                }
                let frames =
                    r.audio.borrow().native.as_ref().unwrap().stats().frames - r.frame_start;
                let minimum = (self.mode.play().as_secs_f64() * f64::from(self.format.0)) as u64;
                if elapsed >= self.mode.play() && bg - r.bg_start >= minimum && frames >= minimum {
                    if r.counts != self.mode.counts()
                        || !r.probes.empty()
                        || r.pending_image.is_some()
                        || r.images_published != self.mode.counts()[2]
                    {
                        return Err("incomplete cycle load".into());
                    }
                    r.bg_end = bg;
                    r.playing_us = elapsed.as_micros() as u64;
                    r.playing_frames = frames;
                    r.draft(DraftAction::Replace {
                        expected: r.draft_state()?,
                        commands: vec![Command::SetAudioProperties {
                            component: sound(15),
                            gain: Gain::new(0.51).unwrap(),
                            visible: true,
                        }],
                    })?;
                    self.advance(4);
                }
            }
            4 => {
                let r = self.active.as_mut().unwrap();
                if r.ready()? {
                    r.trigger(Channel::Scene, sound(15), false)?;
                    r.trigger(Channel::Audition, sound(15), false)?;
                    self.advance(5);
                }
            }
            5 => {
                let r = self.active.as_mut().unwrap();
                if r.probes.empty() && r.editor.ui.audio_state.voices.iter().all(|n| *n > 0) {
                    self.advance(6);
                    self.active.as_mut().unwrap().begin_close()?;
                }
            }
            6 => {
                let r = self.active.as_mut().unwrap();
                r.editor.host.poll().map_err(error)?;
                let usage = r.usage();
                if plan::released(
                    usage,
                    r.workers,
                    r.stats.endpoint_returns,
                    r.counters.background.stats().raw_bytes,
                ) {
                    r.close_ms = r.closed_started.unwrap().elapsed().as_secs_f64() * 1000.;
                    let record = r.record(self.index + 1, usage, true);
                    let summary = r.summary(self.index + 1);
                    let counters = Counters {
                        pcm: r.counters.pcm.clone(),
                        raw: r.counters.raw.clone(),
                        rgba: r.counters.rgba.clone(),
                        background: r.counters.background.clone(),
                    };
                    self.active.take();
                    self.state
                        .device
                        .poll(eframe::egui_wgpu::wgpu::PollType::Wait {
                            submission_index: None,
                            timeout: Some(Duration::from_secs(3)),
                        })
                        .map_err(error)?;
                    if counters.bytes() != [0; 3] || counters.background.stats().raw_bytes != 0 {
                        return Err("resource survived final editor drop".into());
                    }
                    use std::io::Write;
                    let mut file = std::fs::OpenOptions::new()
                        .write(true)
                        .create_new(true)
                        .open(self.path.join(format!("cycle-{:02}.json", self.index + 1)))
                        .map_err(error)?;
                    file.write_all(record.as_bytes()).map_err(error)?;
                    self.rows.push(summary);
                    self.retired = Some(counters);
                    self.advance(7);
                }
            }
            7 if age >= self.mode.settle() => self.advance(8),
            8 if age >= self.mode.baseline() => {
                if self.retired.as_ref().unwrap().bytes() != [0; 3] {
                    return Err("old cycle allocated after close".into());
                }
                self.retired.take();
                self.index += 1;
                if self.index == self.mode.rounds() {
                    self.advance(9);
                } else {
                    self.advance(2);
                    self.active = Some(Round::new(
                        ctx,
                        &self.state,
                        &self.fixture,
                        self.index,
                        self.format,
                        self.product,
                    )?);
                }
            }
            9 if age >= self.mode.final_settle() => self.advance(10),
            10 if age >= self.mode.final_observe() => {
                self.completed = true;
                self.advance(11);
            }
            _ => {}
        }
        Ok(())
    }
    fn delay(&self) -> Duration {
        let total = match self.phase {
            0 | 9 => self.mode.final_settle(),
            1 | 8 => self.mode.baseline(),
            7 => self.mode.settle(),
            10 => self.mode.final_observe(),
            _ => return Duration::from_millis(16),
        };
        total
            .saturating_sub(self.entered.elapsed())
            .max(Duration::from_millis(1))
    }
}
impl eframe::App for Run {
    fn raw_input_hook(&mut self, _: &egui::Context, input: &mut egui::RawInput) {
        self.physical_inputs += input
            .events
            .iter()
            .filter(|e| !matches!(e, egui::Event::WindowFocused(_) | egui::Event::PointerGone))
            .count() as u64;
        input.focused = true;
        for v in input.viewports.values_mut() {
            v.focused = Some(true);
        }
    }
    fn logic(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if let Some(r) = &mut self.active {
            if self.phase < 6 {
                if let Err(e) = r.collect() {
                    self.failure = Some(e);
                }
                r.editor.logic(ctx, frame);
            } else {
                let _ = r.editor.host.poll();
            }
        }
        ctx.request_repaint_after(self.delay());
    }
    fn ui(&mut self, root: &mut egui::Ui, frame: &mut eframe::Frame) {
        if let Some(r) = &mut self.active
            && self.phase == 3
        {
            let g = [
                root.ctx().viewport_rect().width(),
                root.ctx().viewport_rect().height(),
                root.ctx().pixels_per_point(),
            ];
            if r.geometry.is_none() {
                r.geometry = Some(g);
            }
            r.view_valid &= r.geometry == Some(g)
                && root.ctx().input(|i| i.viewport().visible() != Some(false));
            let f = root.ctx().cumulative_frame_nr();
            if r.last_frame != Some(f) {
                r.last_frame = Some(f);
                if let Some(v) = frame.info().cpu_usage {
                    if !v.is_finite() || v < 0. || r.frame_ms.len() == 1500 {
                        self.failure = Some("cycle frame trace invalid/overflow".into());
                    } else {
                        r.frame_ms.push(f64::from(v) * 1000.);
                    }
                }
            }
        }
        if self.failure.is_none()
            && let Err(e) = self.step(root.ctx())
        {
            self.failure = Some(e);
        }
        if let Some(r) = &mut self.active
            && self.phase < 6
            && self.failure.is_none()
        {
            r.editor.ui(root, frame);
        } else {
            root.label("Resource lifecycle observation; editor resources closed.");
        }
        if self.failure.is_some() || self.completed {
            let _ = std::fs::write(
                self.path.join(if self.completed {
                    "report.json"
                } else {
                    "partial-report.json"
                }),
                self.report(),
            );
            if let Some(e) = &self.failure {
                let _ = std::fs::write(
                    self.path.join("failure.txt"),
                    format!("cycle {} phase {}: {e}", self.index + 1, self.phase),
                );
                if let Some(r) = &self.active {
                    let _ = std::fs::write(
                        self.path.join("partial-cycle.json"),
                        r.record(self.index + 1, r.usage(), false),
                    );
                }
            }
            root.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
    fn on_exit(&mut self) {
        if let Some(r) = &mut self.active {
            r.observer.take();
            r.editor.on_exit();
        }
    }
}
pub fn run(destination: &str, smoke: bool) -> Result<(), String> {
    run_policy(destination, smoke, false)
}
pub fn run_policy(destination: &str, smoke: bool, product: bool) -> Result<(), String> {
    let path = PathBuf::from(destination);
    if path.exists() {
        return Err("resource-cycle destination must be new".into());
    }
    std::fs::create_dir_all(&path).map_err(error)?;
    let report = path.join("report.json");
    eframe::run_native(
        "Skin Editor - full resource cycles",
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
            Run::new(c, path, smoke, product)
                .map(|r| Box::new(r) as Box<dyn eframe::App>)
                .map_err(|e| std::io::Error::other(e).into())
        }),
    )
    .map_err(error)?;
    if !report.is_file() {
        return Err("resource cycles incomplete; partial evidence retained".into());
    }
    Ok(())
}
