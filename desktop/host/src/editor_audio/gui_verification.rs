//! Opt-in real eframe/EditorUi diagnostic. Actions originate in a controlled GUI
//! callback, not physical clicks. A bounded diagnostic reader timestamps native
//! observations independently of the next GUI repaint; it never owns PCM.
use super::*;
use crate::{Editor, load_verification::phase};
use eframe::egui;
use skin_editor_infrastructure::{
    WavDecoder,
    separated_native::{FirstCopy, FirstObserver, NativeStats},
};
use std::{
    cell::RefCell,
    path::PathBuf,
    rc::Rc,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::JoinHandle,
    time::{Duration, Instant},
};
pub(super) struct Shared(pub(super) Rc<RefCell<EditorAudio>>);
#[derive(Clone, Copy, PartialEq)]
enum OutputProbe {
    None,
    Text,
    Mesh,
    GpuPath,
    Upload,
    RepaintOnly,
}
impl SkinAudioPort for Shared {
    fn retained_drafts(&self) -> Vec<DraftId> {
        self.0.borrow().retained_drafts()
    }
    fn synchronize(
        &mut self,
        p: &SkinProject,
        s: SkinAudioSource,
        c: &[DraftId],
    ) -> Result<(), SkinAudioError> {
        self.0.borrow_mut().synchronize(p, s, c)
    }
}
impl AudioEditorPort for Shared {
    fn request(&mut self, a: AudioEditorAction) -> Result<(), AudioEditorError> {
        self.0.borrow_mut().request(a)
    }
    fn audition_asset(&mut self, a: AssetAudition) -> Result<(), AudioEditorError> {
        self.0.borrow_mut().audition_asset(a)
    }
    fn poll(&mut self) -> AudioEditorState {
        self.0.borrow_mut().poll()
    }
}
pub(super) struct Observations {
    stop: Arc<AtomicBool>,
    pub(super) rows: Arc<Mutex<Vec<(FirstCopy, Instant)>>>,
    pub(super) overflow: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
    join_ok: bool,
}
impl Observations {
    fn new(reader: FirstObserver) -> Result<Self, String> {
        Self::with_capacity(reader, 192)
    }
    pub(super) fn with_capacity(
        mut reader: FirstObserver,
        capacity: usize,
    ) -> Result<Self, String> {
        if !(1..=4096).contains(&capacity) {
            return Err("invalid bounded first-copy observation capacity".into());
        }
        let stop = Arc::new(AtomicBool::new(false));
        let rows = Arc::new(Mutex::new(Vec::with_capacity(capacity)));
        let overflow = Arc::new(AtomicBool::new(false));
        let (s, r, o) = (stop.clone(), rows.clone(), overflow.clone());
        let thread = std::thread::Builder::new()
            .name("gui-copy-observer".into())
            .spawn(move || {
                while !s.load(Ordering::Acquire) {
                    while let Some(first) = reader.next_first() {
                        let at = Instant::now();
                        let mut rows = r.lock().unwrap();
                        if rows.len() == capacity {
                            o.store(true, Ordering::Release);
                        } else {
                            rows.push((first, at));
                        }
                    }
                    std::thread::sleep(Duration::from_millis(1));
                }
            })
            .map_err(|e| e.to_string())?;
        Ok(Self {
            stop,
            rows,
            overflow,
            thread: Some(thread),
            join_ok: true,
        })
    }
    pub(super) fn close(&mut self) -> bool {
        self.stop.store(true, Ordering::Release);
        if let Some(t) = self.thread.take() {
            self.join_ok &= t.join().is_ok();
        }
        self.join_ok
    }
}
impl Drop for Observations {
    fn drop(&mut self) {
        self.close();
    }
}
fn wav(rate: u32, channels: u16, frames: u32) -> Vec<u8> {
    let size = frames * u32::from(channels) * 2;
    let mut b = Vec::with_capacity(size as usize + 44);
    b.extend(b"RIFF");
    b.extend((36 + size).to_le_bytes());
    b.extend(b"WAVEfmt ");
    b.extend(16u32.to_le_bytes());
    b.extend(1u16.to_le_bytes());
    b.extend(channels.to_le_bytes());
    b.extend(rate.to_le_bytes());
    b.extend((rate * u32::from(channels) * 2).to_le_bytes());
    b.extend((channels * 2).to_le_bytes());
    b.extend(16u16.to_le_bytes());
    b.extend(b"data");
    b.extend(size.to_le_bytes());
    for _ in 0..frames * u32::from(channels) {
        b.extend(4i16.to_le_bytes());
    }
    b
}
struct Probe {
    at: Instant,
    request: u64,
    lane: Channel,
}
#[derive(Default)]
struct GuiWork {
    logic: [[u64; 3]; 18],
    frame_wall: [[u64; 3]; 18],
    frame: Option<u64>,
    submitted_stage: Option<u8>,
    invalid: u64,
}
impl GuiWork {
    fn add(row: &mut [u64; 3], ns: u64) {
        row[0] += 1;
        row[1] = row[1].saturating_add(ns);
        row[2] = row[2].max(ns);
    }
    fn previous_frame(&mut self, frame: u64, seconds: Option<f32>) {
        if self.frame == Some(frame) {
            return;
        }
        self.frame = Some(frame);
        if let (Some(stage), Some(seconds)) = (self.submitted_stage.take(), seconds) {
            if seconds.is_finite() && seconds >= 0.0 {
                Self::add(
                    &mut self.frame_wall[usize::from(stage)],
                    (f64::from(seconds) * 1e9) as u64,
                );
            } else {
                self.invalid += 1;
            }
        }
    }
}
struct Run {
    editor: Editor,
    audio: Rc<RefCell<EditorAudio>>,
    path: PathBuf,
    observer: Option<Observations>,
    started: Instant,
    stage: u8,
    stage_at: Instant,
    pending: Option<Probe>,
    rows: Vec<String>,
    batch: Vec<u64>,
    batch_request: Option<u64>,
    refusal: Option<u64>,
    refused: bool,
    anchor: u64,
    quiet: u64,
    before_hot: u64,
    before_history: u64,
    baseline_pcm: usize,
    snapshot: Option<ProjectSnapshot>,
    nodes: usize,
    stats: NativeStats,
    pcm_peak: usize,
    checks: [bool; 8],
    failure: Option<String>,
    completed: bool,
    lifecycle: Vec<String>,
    visual: Vec<String>,
    ui_cost: [[u64; 3]; 18],
    max_live_targets: usize,
    rate: u32,
    channels: u16,
    period: u32,
    stream: Option<skin_editor_infrastructure::background_stream::StreamObservation>,
    profile_work: bool,
    work_edges: [[u64; 6]; 2],
    gui_work: Option<GuiWork>,
    repaints: Option<crate::repaint_observation::Repaints>,
    adapter: String,
    control: Option<crate::gui_baseline::Control>,
    target_format: String,
    text_work: Option<egui::plugin::TypedPluginHandle<crate::text_work::Census>>,
    mesh_replay: Option<egui::plugin::TypedPluginHandle<crate::mesh_replay::Replay>>,
    #[cfg(feature = "framework-trace")]
    framework_trace: Option<crate::framework_trace::Trace>,
    gpu_path: Option<egui::plugin::TypedPluginHandle<crate::gpu_path::Probe>>,
}
impl Run {
    fn new(
        context: &eframe::CreationContext<'_>,
        path: PathBuf,
        streamed: bool,
        profile_work: bool,
        main_work: bool,
        mode: Option<crate::gui_baseline::Mode>,
        probe: OutputProbe,
    ) -> Result<Self, String> {
        let mut editor = Editor::new(context, false)?;
        editor.main_work = main_work.then(|| RefCell::new(crate::main_work::MainWork::new()));
        let store = editor.images.blobs().clone();
        let device = PreparedDevice::probe()?;
        let (rate, channels) = device.format();
        let pool = PcmBudget::default();
        let mut adapter = crate::product_policy::Policy::CURRENT.audio(store.clone());
        adapter.budget = pool.clone();
        let stream = if streamed {
            let (source, observation) = crate::stream_fixture::open(&path, rate, channels, &pool)?;
            adapter.buffered = Some(source);
            Some(observation)
        } else {
            let mut pcm = pool
                .reserve(rate as usize * usize::from(channels) * 10)
                .map_err(|e| format!("{e:?}"))?
                .allocate();
            for (i, v) in pcm
                .samples_mut()
                .unwrap()
                .chunks_exact_mut(usize::from(channels))
                .enumerate()
            {
                v.fill((std::f32::consts::TAU * 220.0 * i as f32 / rate as f32).sin() * 0.000025);
            }
            adapter.reference =
                Some(BackgroundTrack::new(pcm, rate, channels).map_err(|e| format!("{e:?}"))?);
            None
        };
        let audio = Rc::new(RefCell::new(adapter));
        editor.ui.set_audio_port(Box::new(Shared(audio.clone())));
        for (id, frames) in [(1, rate / 100), (2, rate * 3)] {
            let raw = store
                .stage_audio(wav(rate, channels, frames), &WavDecoder)
                .map_err(|e| format!("{e:?}"))?;
            let asset = AssetId::new(editor.ui.object_id()).unwrap();
            if !editor.ui.submit(
                &mut editor.session,
                vec![Command::RegisterAsset {
                    asset: Asset::new(
                        asset,
                        format!("self gui {id}.wav"),
                        raw.reference(),
                        AssetKind::Audio(raw.audio_metadata().unwrap()),
                    )
                    .unwrap(),
                }],
            ) {
                return Err(editor.ui.message);
            }
            if id == 1 {
                let component = editor
                    .ui
                    .create_audio_component(&mut editor.session, asset, "self.gui".into())
                    .ok_or("component creation")?;
                editor.ui.audio_component = Some(component);
            } else {
                editor.ui.audio_asset = Some(asset);
            }
        }
        editor.ui.open_audio_editor();
        editor.host.enable_trace();
        let text_work = (probe == OutputProbe::Text).then(|| {
            context.egui_ctx.add_plugin(crate::text_work::Census::new());
            context.egui_ctx.plugin::<crate::text_work::Census>()
        });
        let mesh_replay = (probe == OutputProbe::Mesh).then(|| {
            context
                .egui_ctx
                .add_plugin(crate::mesh_replay::Replay::new());
            context.egui_ctx.plugin::<crate::mesh_replay::Replay>()
        });
        let gpu_path = if matches!(probe, OutputProbe::GpuPath | OutputProbe::Upload) {
            context.egui_ctx.add_plugin(crate::gpu_path::Probe::new(
                if probe == OutputProbe::Upload {
                    crate::gpu_path::Mode::Upload
                } else {
                    crate::gpu_path::Mode::Markers
                },
                context.wgpu_render_state.as_ref(),
            )?);
            Some(context.egui_ctx.plugin::<crate::gpu_path::Probe>())
        } else {
            None
        };
        let now = Instant::now();
        #[cfg(feature = "framework-trace")]
        let framework_trace = crate::framework_trace::current();
        #[cfg(feature = "framework-trace")]
        if let Some(t) = &framework_trace {
            context
                .egui_ctx
                .add_plugin(crate::framework_trace::Passes(t.clone()));
        }
        Ok(Self {
            editor,
            audio,
            path,
            observer: None,
            started: now,
            stage: 0,
            stage_at: now,
            pending: None,
            rows: Vec::with_capacity(128),
            batch: Vec::new(),
            batch_request: None,
            refusal: None,
            refused: false,
            anchor: 0,
            quiet: 0,
            before_hot: 0,
            before_history: 0,
            baseline_pcm: 0,
            snapshot: None,
            nodes: 0,
            stats: NativeStats::default(),
            pcm_peak: 0,
            checks: [false; 8],
            failure: None,
            completed: false,
            lifecycle: Vec::with_capacity(17),
            visual: Vec::with_capacity(17),
            ui_cost: [[0; 3]; 18],
            max_live_targets: 0,
            rate,
            channels,
            period: 0,
            stream,
            profile_work,
            work_edges: [[0; 6]; 2],
            gui_work: profile_work.then(GuiWork::default),
            repaints: (main_work || probe == OutputProbe::RepaintOnly)
                .then(|| crate::repaint_observation::Repaints::install(&context.egui_ctx)),
            adapter: format!(
                "{:?}",
                context
                    .wgpu_render_state
                    .as_ref()
                    .unwrap()
                    .adapter
                    .get_info()
            ),
            control: mode.map(crate::gui_baseline::Control::new),
            target_format: format!(
                "{:?}",
                context.wgpu_render_state.as_ref().unwrap().target_format
            ),
            text_work,
            mesh_replay,
            #[cfg(feature = "framework-trace")]
            framework_trace,
            gpu_path,
        })
    }
    fn component(&self) -> AudioComponentId {
        self.editor.ui.audio_component.unwrap()
    }
    fn action(&mut self, a: AudioEditorAction) {
        if let AudioEditorAction::SceneRunning(running) = a {
            self.editor.ui.paused = !running;
        }
        self.editor.ui.audio_command(&self.editor.session, a);
    }
    fn advance(&mut self, stage: u8) {
        use skin_editor_ui::Presenter;
        let view = self.editor.host.current();
        let (generation, size) = view.map_or((0, [0, 0]), |v| {
            (
                v.frame.source.context.viewport_generation,
                v.frame.source.context.size,
            )
        });
        self.visual.push(format!(
            "[{stage},{},{},{},{},{generation},{},{},{},{}]",
            self.editor.ui_passes,
            self.editor.host.allocations,
            self.editor.host.publications,
            self.editor.host.traces.len(),
            size[0],
            size[1],
            self.editor.session.skin_audio_source().draft.is_some(),
            self.editor
                .ui
                .audio_state
                .source
                .is_some_and(|s| s.draft.is_some())
        ));
        let s = &self.editor.ui.audio_state;
        let (copied, generated) = s
            .output
            .as_ref()
            .map_or((0, 0), |o| (o.copied_background, o.generated_background));
        self.lifecycle.push(format!(
            "[{stage},{:.6},{},{},{copied},{generated},{},{},{}]",
            self.started.elapsed().as_secs_f64() * 1000.0,
            s.voices[0],
            s.voices[1],
            s.frozen,
            s.freezing,
            self.audio.borrow().budget.usage().bytes
        ));
        self.stage = stage;
        self.stage_at = Instant::now();
    }
    fn ready(&self) -> bool {
        let s = &self.editor.ui.audio_state;
        s.enabled
            && !s.pending_controls
            && s.source == Some(self.editor.session.skin_audio_source())
            && s.components.len() == 1
            && s.components[0].preparation == AudioPreparationState::Ready
    }
    fn drain(&mut self) -> Result<(), String> {
        let mut a = self.audio.borrow_mut();
        if let Some((id, result)) = &a.last_batch
            && self.batch_request == Some(*id)
        {
            self.batch = result.clone()?;
            self.batch_request = None;
        }
        if let Some((id, error)) = &a.last_failed
            && self.refusal == Some(*id)
        {
            self.refused = matches!(error,SupplyError::Backend(e) if e=="ResourceLimit");
            if !self.refused {
                return Err("33rd voice not rejected".into());
            }
            self.refusal = None;
            a.error = None;
            self.editor.ui.audio_state.error = None;
        }
        Ok(())
    }
    fn step(&mut self) -> Result<(), String> {
        self.drain()?;
        if self.started.elapsed() > Duration::from_secs(28) {
            return Err("GUI diagnostic deadline".into());
        }
        if let Some(error) = &self.editor.ui.audio_state.error {
            return Err(error.clone());
        }
        if let Some(o) = &self.observer
            && o.overflow.load(Ordering::Acquire)
        {
            return Err("observer overflow".into());
        }
        let age = self.stage_at.elapsed();
        match self.stage {
            0 if age >= Duration::from_secs(1) && !self.editor.host.needs_work() => {
                self.action(AudioEditorAction::MonitorVolume(500));
                self.action(AudioEditorAction::Enable);
                let reader = {
                    let mut a = self.audio.borrow_mut();
                    let n = a.native.as_mut().ok_or("GUI native not enabled")?;
                    if self.profile_work {
                        n.capture_work_cost();
                    }
                    self.period = n.period_frames();
                    n.take_first_observer().ok_or("reader already taken")?
                };
                self.observer = Some(Observations::new(reader)?);
                phase("idle");
                self.advance(1);
            }
            1 if self.ready() && age >= Duration::from_millis(1100) => {
                self.action(AudioEditorAction::SceneRunning(true));
                phase("voices_1");
                self.advance(2);
            }
            2 if self.ready() && !self.editor.ui.audio_state.frozen => {
                if let Some(p) = &self.pending {
                    let a = self.audio.borrow();
                    if let Some((request, r)) = a.last_triggered
                        && request == p.request
                    {
                        let voice = r.voice.ok_or("missing GUI voice")?;
                        let rows = self.observer.as_ref().unwrap().rows.lock().unwrap();
                        if let Some((f, observed)) = rows
                            .iter()
                            .find(|(f, _)| f.voice == voice && f.channel == p.lane)
                        {
                            let upper = observed
                                .checked_duration_since(p.at)
                                .ok_or("copy predates GUI request")?
                                .as_secs_f64()
                                * 1000.0;
                            let joined = p.at.elapsed().as_secs_f64() * 1000.0;
                            self.rows.push(format!(
                                "[{},{},{},{},{upper:.9},{joined:.9},{},{},{},{}]",
                                self.rows.len() / 2,
                                usize::from(p.lane == Channel::Audition),
                                p.request,
                                voice,
                                f.callback_nanos,
                                f.callback_ordinal,
                                f.offset_frames,
                                f.callback_frames
                            ));
                            self.pending = None;
                        }
                    }
                }
                if self.pending.is_none() && self.rows.len() < 128 {
                    let lane = if self.rows.len().is_multiple_of(2) {
                        Channel::Scene
                    } else {
                        Channel::Audition
                    };
                    let at = Instant::now();
                    let request = if lane == Channel::Audition {
                        self.action(AudioEditorAction::Audition(self.component()));
                        self.audio
                            .borrow()
                            .last_trigger_request
                            .ok_or("GUI audition not queued")?
                    } else {
                        // Preview fixture event in this real GUI callback. Uses the same
                        // adapter/client; no additional project write or Runtime access.
                        self.editor
                            .session
                            .synchronize_audio(&mut *self.audio.borrow_mut())
                            .map_err(|e| format!("{e:?}"))?;
                        self.audio
                            .borrow_mut()
                            .client
                            .as_mut()
                            .unwrap()
                            .trigger_selected(lane, self.component())
                            .map_err(|e| format!("{e:?}"))?
                    };
                    self.pending = Some(Probe { at, request, lane });
                } else if self.pending.is_none() {
                    self.before_hot = self
                        .editor
                        .ui
                        .audio_state
                        .output
                        .as_ref()
                        .unwrap()
                        .copied_background;
                    let asset = self.editor.ui.audio_asset.unwrap();
                    let component = self.component();
                    if !self.editor.ui.submit(
                        &mut self.editor.session,
                        vec![Command::SetAudioAsset { component, asset }],
                    ) {
                        return Err(self.editor.ui.message.clone());
                    }
                    self.advance(3);
                }
            }
            3 if self.ready() => {
                self.checks[0] = self
                    .editor
                    .ui
                    .audio_state
                    .output
                    .as_ref()
                    .unwrap()
                    .copied_background
                    >= self.before_hot;
                phase("voices_32");
                let entries: Vec<_> = (0..32)
                    .map(|i| {
                        (
                            if i % 2 == 0 {
                                Channel::Scene
                            } else {
                                Channel::Audition
                            },
                            self.component(),
                        )
                    })
                    .collect();
                self.batch_request = Some(
                    self.audio
                        .borrow_mut()
                        .client
                        .as_mut()
                        .unwrap()
                        .trigger_batch_selected(&entries)
                        .map_err(|e| format!("{e:?}"))?,
                );
                self.work_edges[0] = self
                    .audio
                    .borrow()
                    .native
                    .as_ref()
                    .unwrap()
                    .work_cost()
                    .ok_or("partial work snapshot")?;
                self.advance(4);
            }
            4 if self.batch.len() == 32 && self.refusal.is_none() && !self.refused => {
                if self.editor.ui.audio_state.voices != [16, 16] {
                    return Err("GUI 32 voice count".into());
                }
                self.refusal = Some(
                    self.audio
                        .borrow_mut()
                        .client
                        .as_mut()
                        .unwrap()
                        .trigger_selected(Channel::Audition, self.component())
                        .map_err(|e| format!("{e:?}"))?,
                );
            }
            4 if age >= Duration::from_millis(2100) && self.refused => {
                self.work_edges[1] = self
                    .audio
                    .borrow()
                    .native
                    .as_ref()
                    .unwrap()
                    .work_cost()
                    .ok_or("partial work snapshot")?;
                let actual: Vec<_> = self
                    .observer
                    .as_ref()
                    .unwrap()
                    .rows
                    .lock()
                    .unwrap()
                    .iter()
                    .filter(|(f, _)| self.batch.contains(&f.voice))
                    .map(|(f, _)| f.voice)
                    .collect();
                let mut expected = self.batch.clone();
                let mut actual = actual;
                expected.sort_unstable();
                actual.sort_unstable();
                self.checks[1] = actual == expected;
                phase("controls");
                self.action(AudioEditorAction::SceneRunning(false));
                self.advance(5);
            }
            5 if self.editor.ui.audio_state.frozen && !self.editor.ui.audio_state.freezing => {
                self.anchor = self
                    .editor
                    .ui
                    .audio_state
                    .output
                    .as_ref()
                    .unwrap()
                    .copied_background;
                self.advance(6);
            }
            6 if age >= Duration::from_millis(80) => {
                self.checks[2] = self.editor.ui.audio_state.voices == [0, 16]
                    && self
                        .editor
                        .ui
                        .audio_state
                        .output
                        .as_ref()
                        .unwrap()
                        .copied_background
                        == self.anchor;
                self.action(AudioEditorAction::StopAudition);
                self.advance(7);
            }
            7 if !self.editor.ui.audio_state.pending_controls
                && self.editor.ui.audio_state.voices == [0, 0] =>
            {
                self.quiet = self.editor.ui.audio_state.submitted_nonzero_samples;
                self.advance(8);
            }
            8 if age >= Duration::from_millis(80) => {
                self.checks[3] = self.editor.ui.audio_state.submitted_nonzero_samples == self.quiet;
                self.action(AudioEditorAction::SceneRunning(true));
                self.advance(9);
            }
            9 if !self.editor.ui.audio_state.frozen
                && self
                    .editor
                    .ui
                    .audio_state
                    .output
                    .as_ref()
                    .unwrap()
                    .copied_background
                    > self.anchor =>
            {
                self.checks[4] = true;
                self.action(AudioEditorAction::SceneRunning(false));
                self.advance(10);
            }
            10 if self.ready()
                && self.editor.ui.audio_state.frozen
                && !self.editor.ui.audio_state.freezing =>
            {
                self.baseline_pcm = self.audio.borrow().budget.usage().bytes;
                self.snapshot = Some(self.editor.session.snapshot());
                self.nodes = self.editor.session.history_len();
                let expected = self.editor.session.project_info().state;
                if !self
                    .editor
                    .ui
                    .draft(&mut self.editor.session, DraftAction::Begin { expected })
                {
                    return Err(self.editor.ui.message.clone());
                }
                let expected = self
                    .editor
                    .session
                    .draft_info(self.editor.session.project_info().session_id)
                    .unwrap()
                    .unwrap()
                    .state;
                let component = self.component();
                if !self.editor.ui.draft(
                    &mut self.editor.session,
                    DraftAction::Replace {
                        expected,
                        commands: vec![Command::SetAudioProperties {
                            component,
                            gain: Gain::new(0.5).unwrap(),
                            visible: true,
                        }],
                    },
                ) {
                    return Err(self.editor.ui.message.clone());
                }
                self.advance(11);
            }
            11 if self.ready() => {
                if self.editor.session.skin_audio_source().draft.is_none()
                    || self
                        .editor
                        .ui
                        .audio_state
                        .source
                        .is_none_or(|s| s.draft.is_none())
                {
                    return Err("GUI draft was cancelled before audition".into());
                }
                self.action(AudioEditorAction::Audition(self.component()));
                self.advance(12);
            }
            12 if self.editor.ui.audio_state.voices == [0, 1] => {
                if self.editor.session.skin_audio_source().draft.is_none() {
                    return Err("GUI draft disappeared before explicit cancel".into());
                }
                let d = self.editor.session.skin_audio_source().draft.unwrap();
                if self
                    .audio
                    .borrow()
                    .last_triggered
                    .is_none_or(|(_, r)| r.origin.draft != Some((d.id.get(), d.revision)))
                {
                    return Err("GUI audition was not a draft voice".into());
                }
                self.editor.ui.cancel(&mut self.editor.session);
                self.advance(13);
            }
            13 if self.ready()
                && self.editor.ui.audio_state.voices == [0, 0]
                && self.audio.borrow().retained_drafts().is_empty()
                && self.audio.borrow().budget.usage().bytes <= self.baseline_pcm =>
            {
                self.checks[5] = self.snapshot.as_ref() == Some(&self.editor.session.snapshot())
                    && self.editor.session.history_len() == self.nodes;
                self.before_history = self
                    .editor
                    .ui
                    .audio_state
                    .output
                    .as_ref()
                    .unwrap()
                    .copied_background;
                self.action(AudioEditorAction::Audition(self.component()));
                self.advance(14);
            }
            13 if age >= Duration::from_secs(1) => {
                return Err("cancel did not clear the verified 3-second draft voice within the diagnostic guard".into());
            }
            14 if self.editor.ui.audio_state.voices == [0, 1] => {
                self.editor
                    .ui
                    .navigation(&mut self.editor.session, HistoryAction::Undo);
                self.advance(15);
            }
            15 if self.ready() && self.editor.ui.audio_state.voices == [0, 0] => {
                self.checks[6] = self
                    .editor
                    .ui
                    .audio_state
                    .output
                    .as_ref()
                    .unwrap()
                    .copied_background
                    == self.before_history;
                self.editor.ui.navigation(
                    &mut self.editor.session,
                    HistoryAction::Redo { child: None },
                );
                self.advance(16);
            }
            16 if self.ready() => {
                let o = self.editor.ui.audio_state.output.as_ref().unwrap();
                self.checks[6] &= o.copied_background == self.before_history
                    && self.editor.ui.audio_state.voices == [0, 0];
                self.checks[7] = o.generated_background < self.audio.borrow().background_frames;
                self.stats = self.audio.borrow().native.as_ref().unwrap().stats();
                self.pcm_peak = self.audio.borrow().budget.usage().peak_bytes;
                self.observer.take();
                self.action(AudioEditorAction::Disable);
                // close() checks exact endpoint ownership; count comes from the
                // returned native stats, not an invented callback acknowledgement.
                self.stats = self
                    .audio
                    .borrow()
                    .closed_stats
                    .ok_or("missing stopped GUI stats")?;
                if self.audio.borrow().budget.usage().bytes != 0 {
                    return Err("GUI PCM retained after close".into());
                }
                phase("closed");
                self.advance(17);
            }
            _ => {}
        }
        Ok(())
    }
    fn report(&self) -> String {
        let s = self.stats;
        let work_fields = format!(
            "\"gpu_path\":{},\"mesh_replay\":{},\"text_work\":{},\"gui_control\":{},\"target_format_utf8\":{:?},\"profile_work\":{},\"repaints\":{},\"main_work\":{},\"work_edges\":{:?},\"adapter_utf8\":{:?},\"gui_work\":{}",
            self.gpu_path.as_ref().map_or_else(|| "null".into(), |w| w.lock().report()),
            self.mesh_replay.as_ref().map_or_else(|| "null".into(), |w| w.lock().report()),
            self.text_work.as_ref().map_or_else(|| "null".into(), |w| w.lock().report()),
            self.control.as_ref().map_or_else(|| "null".into(), |c| c.report()),
            self.target_format.as_bytes(),
            self.profile_work,
            self.repaints.as_ref().map_or_else(|| "null".into(), |r| r.report()),
            self.editor.main_work.as_ref().map_or_else(|| "null".into(), |w| w.borrow().report()),
            self.work_edges,
            self.adapter.as_bytes(),
            self.gui_work.as_ref().map_or_else(
                || "null".into(),
                |w| format!(
                    "{{\"logic\":{:?},\"frame_wall\":{:?},\"invalid\":{},\"scope\":\"previous-framework-frame-wall-excluding-reported-vsync\"}}",
                    w.logic, w.frame_wall, w.invalid
                )
            )
        );
        let stream_fields=self.stream.as_ref().map_or_else(||String::from("\"streamed\":false"),|o|{let v=o.stats();format!("\"streamed\":true,\"source_frames\":{},\"source_read_bytes\":{},\"source_seeks\":{},\"source_raw_peak\":{},\"source_raw_final\":{},\"source_fault\":{}",u64::from(self.rate)*360,v.read_bytes,v.seeks,v.raw_peak_bytes,v.raw_bytes,v.fault)});
        let costs = self
            .ui_cost
            .iter()
            .map(|row| format!("{:?}", row))
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "{{{work_fields},{stream_fields},\"callback_resource_releases\":{},\"endpoint_returns_in_render\":{},\"endpoint_returns_outside_render\":{},\"schema\":2,\"controlled_focus\":true,\"scope\":\"gui-action-dispatch-to-native-copy\",\"physical_input\":false,\"rate\":{},\"channels\":{},\"period_frames\":{},\"probes\":[{}],\"callbacks\":{},\"missing_frames\":{},\"render_errors\":{},\"observation_overflows\":{},\"endpoint_returns\":{},\"device_fault\":{},\"nonzero_samples\":{},\"pcm_peak_bytes\":{},\"pcm_final_bytes\":{},\"peak_voices\":32,\"refused_33\":{},\"hot_continuity\":{},\"burst_exact\":{},\"pause_isolated\":{},\"stop_quiet\":{},\"resumed\":{},\"cancel_clean\":{},\"history_clean\":{},\"source_not_eof\":{},\"gui_passes\":{},\"lifecycle\":[{}],\"visual\":[{}],\"ui_cost\":[{}],\"max_live_targets\":{}}}",
            s.callback_resource_releases,
            s.endpoint_returns_in_render,
            s.endpoint_returns_outside_render,
            self.rate,
            self.channels,
            self.period,
            self.rows.join(","),
            s.callbacks,
            s.missing_frames,
            s.render_errors,
            s.observation_overflows,
            s.endpoint_returns,
            s.device_fault,
            s.nonzero_samples,
            self.pcm_peak,
            self.audio.borrow().budget.usage().bytes,
            self.refused,
            self.checks[0],
            self.checks[1],
            self.checks[2],
            self.checks[3],
            self.checks[4],
            self.checks[5],
            self.checks[6],
            self.checks[7],
            self.editor.ui_passes,
            self.lifecycle.join(","),
            self.visual.join(","),
            costs,
            self.max_live_targets
        )
    }
    fn draw_control(
        &mut self,
        root: &mut egui::Ui,
        frame: &mut eframe::Frame,
    ) -> Result<(), String> {
        use skin_editor_ui::Presenter;
        let full = self
            .control
            .as_ref()
            .is_none_or(|c| c.mode.full_ui(self.stage));
        let painted;
        if full {
            painted = self.editor.host.current().is_some();
            eframe::App::ui(&mut self.editor, root, frame);
        } else {
            let geometry = self
                .control
                .as_ref()
                .and_then(|c| c.geometry)
                .ok_or("control layout not ready before measurement")?;
            self.editor.ui_passes += 1;
            self.editor.work_edge(crate::main_work::WIDGETS);
            // A real quad keeps the ordinary framework render/submit path active,
            // even in the control without our preview PaintCallback.
            root.painter()
                .rect_filled(root.max_rect(), 0., egui::Color32::from_gray(27));
            painted =
                self.control.as_ref().unwrap().mode == crate::gui_baseline::Mode::Presentation;
            if painted {
                let view = self
                    .editor
                    .host
                    .current()
                    .ok_or("control preview disappeared")?;
                self.editor
                    .host
                    .paint(root, view.frame.key, geometry.rect());
            }
            self.editor.work_edge(crate::main_work::UI_DISPATCH);
            self.editor.complete_ui(
                root.ctx(),
                skin_editor_ui::UiOutput {
                    viewport: Some((geometry.size, geometry.generation)),
                    ..Default::default()
                },
            );
        }
        if let Some(c) = &mut self.control {
            if let (Some(rect), Some(view)) =
                (self.editor.ui.canvas_rect, self.editor.host.current())
            {
                let geometry = crate::gui_baseline::Geometry::new(
                    root.ctx().viewport_rect().size(),
                    root.ctx().pixels_per_point(),
                    rect,
                    view.frame.source.context.size,
                    view.frame.source.context.viewport_generation,
                )
                .map_err(str::to_owned)?;
                c.observe(geometry).map_err(str::to_owned)?;
            }
            c.record(self.stage, full, painted);
        }
        Ok(())
    }
}
impl eframe::App for Run {
    fn raw_input_hook(&mut self, ctx: &egui::Context, input: &mut egui::RawInput) {
        if let Some(r) = &mut self.repaints {
            r.input(ctx, input);
        }
        if let Some(w) = &self.editor.main_work {
            w.borrow_mut().edge(crate::main_work::FRAMEWORK);
        }
        // Controlled focused GUI input, not proof that Windows granted focus.
        // Real focus-loss cancellation is covered by the device-free UI harness.
        input.focused = true;
    }
    fn logic(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        #[cfg(feature = "framework-trace")]
        let _logic_span = self.framework_trace.as_ref().map(|t| t.logic());
        if let Some(r) = &mut self.repaints {
            r.pass(ctx);
        }
        if let Some(w) = &self.editor.main_work {
            w.borrow_mut().edge(crate::main_work::LOGIC);
        }
        let at = self.gui_work.as_ref().map(|_| Instant::now());
        if self.stage < 17 && self.failure.is_none() {
            self.editor.logic(ctx, frame);
        }
        if let (Some(work), Some(at)) = (&mut self.gui_work, at) {
            GuiWork::add(
                &mut work.logic[usize::from(self.stage)],
                at.elapsed().as_nanos() as u64,
            );
        }
        ctx.request_repaint_after(Duration::from_millis(
            if self.stage >= 17 || self.stage == 1 {
                50
            } else {
                16
            },
        ));
        if let Some(w) = &self.editor.main_work {
            w.borrow_mut().edge(crate::main_work::FRAMEWORK);
        }
    }
    fn ui(&mut self, root: &mut egui::Ui, frame: &mut eframe::Frame) {
        if let Some(work) = &mut self.gui_work {
            work.previous_frame(root.ctx().cumulative_frame_nr(), frame.info().cpu_usage);
        }
        if self.failure.is_none() && self.stage < 17 {
            if let Some(w) = &self.editor.main_work {
                w.borrow_mut().edge(crate::main_work::DIAGNOSTIC);
            }
            if let Err(e) = self.step() {
                self.failure = Some(e);
            }
            if let Some(w) = &self.editor.main_work {
                w.borrow_mut().edge(crate::main_work::UI);
            }
            let at = Instant::now();
            if let Some(text) = &self.text_work {
                text.lock().stage = self.stage;
            }
            if let Some(mesh) = &self.mesh_replay {
                mesh.lock().stage = self.stage;
            }
            if let Some(gpu) = &self.gpu_path {
                gpu.lock().stage = self.stage;
            }
            #[cfg(feature = "framework-trace")]
            if let Some(trace) = &self.framework_trace {
                trace.stage(self.stage);
            }
            if let Err(e) = self.draw_control(root, frame)
                && self.failure.is_none()
            {
                self.failure = Some(e);
            }
            if let Some(w) = &self.editor.main_work {
                w.borrow_mut().edge(crate::main_work::FRAMEWORK);
            }
            self.max_live_targets = self.max_live_targets.max(self.editor.host.live_targets());
            let elapsed = at.elapsed().as_nanos() as u64;
            let cost = &mut self.ui_cost[usize::from(self.stage)];
            cost[0] += 1;
            cost[1] += elapsed;
            cost[2] = cost[2].max(elapsed);
        }
        if let Some(work) = &mut self.gui_work {
            work.submitted_stage = Some(self.stage);
        }
        if self.failure.is_some()
            || self.stage == 17 && self.stage_at.elapsed() >= Duration::from_millis(1100)
        {
            if let Some(e) = &self.failure {
                let _ = std::fs::write(self.path.join("partial-report.json"), self.report());
                let _ = std::fs::write(
                    self.path.join("failure.txt"),
                    format!(
                        "stage {}: {e}\n{:?}",
                        self.stage, self.editor.ui.audio_state
                    ),
                );
            } else {
                let _ = std::fs::write(self.path.join("report.json"), self.report());
                self.completed = true;
            }
            root.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
    fn clear_color(&self, visuals: &egui::Visuals) -> [f32; 4] {
        if let Some(gpu) = &self.gpu_path {
            gpu.lock().after_tessellation();
        }
        if let Some(w) = &self.editor.main_work {
            w.borrow_mut().edge(crate::main_work::PRESENT_AND_EVENTS);
        }
        // Delegate to our existing Editor's default App implementation.
        eframe::App::clear_color(&self.editor, visuals)
    }
    fn on_exit(&mut self) {
        self.observer.take();
        self.editor.on_exit();
        if let Some(gpu) = &self.gpu_path {
            let mut gpu = gpu.lock();
            gpu.finish();
            let _ = std::fs::write(self.path.join("gpu_shutdown.json"), gpu.report());
        }
        if let Some(text) = &self.text_work {
            let mut text = text.lock();
            text.clear_identities();
            let _ = std::fs::write(self.path.join("text_shutdown.json"), text.report());
        }
        if let Some(mesh) = &self.mesh_replay {
            let _ = std::fs::write(self.path.join("mesh_shutdown.json"), mesh.lock().report());
        }
        if !self.completed && self.failure.is_none() {
            let _ = std::fs::write(
                self.path.join("failure.txt"),
                "GUI closed before completion",
            );
        }
    }
}
pub fn run(destination: &str) -> Result<(), String> {
    run_mode(destination, false, false, false)
}
pub fn streamed(destination: &str) -> Result<(), String> {
    run_mode(destination, true, false, false)
}
pub fn profiled(destination: &str) -> Result<(), String> {
    run_mode(destination, true, true, false)
}
pub fn main_work(destination: &str) -> Result<(), String> {
    run_mode(destination, true, false, true)
}
pub fn repaint_only(destination: &str) -> Result<(), String> {
    run_control(
        destination,
        true,
        false,
        false,
        None,
        OutputProbe::RepaintOnly,
    )
}
#[cfg(feature = "framework-trace")]
pub fn dispatch(destination: &str, main_work: bool) -> Result<(), String> {
    run_control(
        destination,
        true,
        false,
        main_work,
        None,
        OutputProbe::RepaintOnly,
    )
}
pub fn baseline(destination: &str, mode: crate::gui_baseline::Mode) -> Result<(), String> {
    run_control(
        destination,
        true,
        false,
        true,
        Some(mode),
        OutputProbe::None,
    )
}
pub fn text_work(destination: &str) -> Result<(), String> {
    run_control(destination, true, false, true, None, OutputProbe::Text)
}
pub fn mesh_replay(destination: &str) -> Result<(), String> {
    run_control(destination, true, false, true, None, OutputProbe::Mesh)
}
pub fn gpu_path(destination: &str, upload: bool) -> Result<(), String> {
    run_control(
        destination,
        true,
        false,
        true,
        None,
        if upload {
            OutputProbe::Upload
        } else {
            OutputProbe::GpuPath
        },
    )
}
fn run_mode(
    destination: &str,
    streamed: bool,
    profile_work: bool,
    main_work: bool,
) -> Result<(), String> {
    run_control(
        destination,
        streamed,
        profile_work,
        main_work,
        None,
        OutputProbe::None,
    )
}
fn run_control(
    destination: &str,
    streamed: bool,
    profile_work: bool,
    main_work: bool,
    mode: Option<crate::gui_baseline::Mode>,
    probe: OutputProbe,
) -> Result<(), String> {
    let path = PathBuf::from(destination);
    if path.exists() {
        return Err("GUI diagnostic destination must be new".into());
    }
    std::fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    let report = path.join("report.json");
    phase("loading");
    #[cfg(feature = "framework-trace")]
    let run_native = if crate::framework_trace::current().is_some() {
        crate::framework_trace::run_native
    } else {
        eframe::run_native
    };
    #[cfg(not(feature = "framework-trace"))]
    let run_native = eframe::run_native;
    run_native(
        "Skin Editor - candidate audio GUI short verification",
        eframe::NativeOptions {
            renderer: eframe::Renderer::Wgpu,
            wgpu_options: crate::gpu_observation::configuration(true),
            viewport: crate::gui_baseline::viewport(mode),
            ..Default::default()
        },
        Box::new(move |c| {
            Run::new(c, path, streamed, profile_work, main_work, mode, probe)
                .map(|a| Box::new(a) as Box<dyn eframe::App>)
                .map_err(|e| std::io::Error::other(e).into())
        }),
    )
    .map_err(|e| e.to_string())?;
    if !report.is_file() {
        return Err("GUI verification incomplete; inspect failure.txt".into());
    }
    Ok(())
}

#[cfg(test)]
mod gui_work_tests {
    use super::GuiWork;

    #[test]
    fn previous_frame_belongs_to_submitted_stage_and_is_not_counted_twice() {
        let mut w = GuiWork::default();
        w.previous_frame(0, None);
        w.submitted_stage = Some(4);
        w.previous_frame(1, Some(0.001));
        w.submitted_stage = Some(5);
        w.previous_frame(1, Some(0.001));
        assert_eq!(w.frame_wall[4][0], 1);
        assert_eq!(w.frame_wall[5][0], 0);
        w.previous_frame(2, Some(0.002));
        assert_eq!(w.frame_wall[5][0], 1);
        assert!(w.frame_wall[5][1] > w.frame_wall[4][1]);
    }

    #[test]
    fn absent_first_frame_and_invalid_values_are_not_cpu_samples() {
        let mut w = GuiWork::default();
        for (i, value) in [None, Some(f32::NAN), Some(-1.0), Some(f32::INFINITY)]
            .into_iter()
            .enumerate()
        {
            w.submitted_stage = Some(4);
            w.previous_frame(i as u64, value);
        }
        assert_eq!(w.frame_wall[4], [0; 3]);
        assert_eq!(w.invalid, 3);
    }
}

#[cfg(test)]
mod observation_close_tests {
    use super::*;
    #[test]
    fn observer_panic_is_a_sticky_failed_join() {
        let mut o = Observations {
            stop: Arc::new(AtomicBool::new(false)),
            rows: Arc::new(Mutex::new(Vec::new())),
            overflow: Arc::new(AtomicBool::new(false)),
            thread: Some(std::thread::spawn(|| panic!("controlled observer failure"))),
            join_ok: true,
        };
        assert!(!o.close());
        assert!(!o.close());
        assert!(o.thread.is_none());
    }
}
