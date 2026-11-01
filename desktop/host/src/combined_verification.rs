//! Ordinary eframe presentation + independent native supply. No pixel readback.
use crate::audio_verification::submit;
use crate::edit_verification::source;
use crate::load_verification::{outcome, phase, stop, wait, wav};
use crate::{Editor, fresh_session};
use eframe::egui;
use skin_editor_application::{values::*, *};
use skin_editor_audio::{Channel, PcmBudget, Preparation, PrepareFailure};
use skin_editor_infrastructure::{
    AudioSupplyClient, NativeOutput, NativeStream, OutputControl, OutputStats, SupplyOutcome,
    write_png,
};
use std::io::Write;
use std::path::PathBuf;
use std::sync::mpsc::{self, SyncSender};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

const SECONDS: u64 = 300;
const TRIGGERS: usize = 2400;
const EDITS: usize = 300;
const SWITCHES: usize = 20;
mod recovery;
pub use recovery::run as recovery_run;
mod phase;
pub use phase::run as phase_run;
pub use phase::run_controlled as phase_controlled_run;
pub use phase::run_recovery as phase_recovery_run;
mod cadence;
fn optional_rows<const N: usize>(rows: impl IntoIterator<Item = Option<[u64; N]>>) -> String {
    rows.into_iter()
        .map(|r| r.map_or_else(|| "null".into(), |v| format!("{v:?}")))
        .collect::<Vec<_>>()
        .join(",")
}
fn clock_alignment(output: &OutputControl, stream: &NativeStream) -> Result<[u64; 3], String> {
    let origin = output.capture_origin().ok_or("capture not armed")?;
    let clock = stream.clock_sample()?;
    Ok([
        clock.stream_nanos,
        clock.before.saturating_duration_since(origin).as_nanos() as u64,
        clock.after.saturating_duration_since(origin).as_nanos() as u64,
    ])
}
fn scheduling_json(output: &OutputControl, alignment: [u64; 3], quiescent: bool) -> String {
    let s = output.supply_timing();
    format!(
        "{{\"schema\":1,\"rounding_ns\":{},\"alignment\":{alignment:?},\"quiescent\":{quiescent},\"completed_pass\":{},\"recent\":[{}],\"slow_count\":{},\"slow\":[{}],\"event_count\":{},\"events\":[{}]}}",
        if cfg!(windows) { 100 } else { 0 },
        s.completed_pass,
        optional_rows(s.recent),
        s.slow_count,
        optional_rows(s.slow),
        s.event_count,
        optional_rows(s.events)
    )
}
fn error(e: impl std::fmt::Debug) -> String {
    format!("{e:?}")
}
fn document(index: usize) -> DocumentId {
    DocumentId::new(index as u128 + 1000).unwrap()
}
fn sound(index: usize) -> AudioComponentId {
    AudioComponentId::new(index as u128 + 1).unwrap()
}
fn layer(index: usize) -> LayerId {
    LayerId::new(index as u128 + 10000).unwrap()
}
fn ready_all(
    client: &mut AudioSupplyClient,
    output: &OutputControl,
    session: &ProjectSession,
) -> Result<(), String> {
    let expected = source(session);
    let mut failure = None;
    wait(output, || {
        while let Some(r) = client.poll() {
            if let Err(e) = r.result {
                failure = Some(error(e));
            }
        }
        if let Ok(status) = client.status() {
            for (id, b) in status.bindings {
                if b.preparation == Preparation::Failed(PrepareFailure::Busy) {
                    match client
                        .retry_selected(id)
                        .map_err(error)
                        .and_then(|id| outcome(client, id))
                    {
                        Ok(_) => {}
                        Err(e) => failure = Some(e),
                    }
                }
            }
        }
        failure.is_some()
            || client.status().is_ok_and(|s| {
                s.origin == Some(expected)
                    && s.ready_components == 16
                    && s.bindings
                        .iter()
                        .all(|(_, b)| b.desired == expected && b.effective == Some(expected))
            })
    })?;
    failure.map_or(Ok(()), Err)
}
fn play_old(
    client: &mut AudioSupplyClient,
    output: &OutputControl,
    session: &ProjectSession,
) -> Result<(), String> {
    ready_all(client, output, session)?;
    let id = client
        .trigger_selected(Channel::Audition, sound(12))
        .map_err(error)?;
    match outcome(client, id)? {
        SupplyOutcome::Triggered(r) if r.voice.is_some() && !r.stale => Ok(()),
        other => Err(format!("old-voice setup failed: {other:?}")),
    }
}
fn cleared(client: &mut AudioSupplyClient, output: &OutputControl) -> Result<bool, String> {
    let id = client.barrier().map_err(error)?;
    outcome(client, id)?;
    let mark = output.stats().lane_epochs;
    wait(output, || {
        output
            .stats()
            .lane_acknowledged
            .iter()
            .zip(mark)
            .all(|(a, e)| *a >= e)
    })?;
    Ok(client.status().map_err(error)?.voices == [0, 0])
}

/// Latency observation runs independently of egui's 16 ms refresh. A scene
/// probe uses a specific Voice from the overlapping standard triggers;
/// audition probes are separate short sounds, never attributed to scene audio.
/// At most one probe exists. The callback still contains no clock or allocation.
struct Probe {
    callback_reference: Option<[u64; 4]>,
    completed: Option<SyncSender<()>>,
    started: Instant,
    receipt_ms: f64,
    started_in_play_ms: f64,
    voice: u64,
    lane: usize,
    clock: Option<skin_editor_infrastructure::StreamClockSample>,
}
struct Observer {
    sender: Option<SyncSender<Probe>>,
    thread: Option<JoinHandle<Result<ProbeObservations, String>>>,
}
struct ProbeObservations {
    latencies: [Vec<f64>; 2],
    /// Per Voice: last negative before-read, first positive after-read,
    /// request-to-receipt, first poll, request offset in the playing phase.
    intervals: [Vec<[f64; 5]>; 2],
    causal: [Vec<[f64; 13]>; 2],
    exact: [Vec<[f64; 2]>; 2],
    markers: [Vec<[u64; 7]>; 2],
    requested: [Vec<u64>; 2],
    request_phases: [Vec<[f64; 2]>; 2],
    request_refs: [Vec<[u64; 4]>; 2],
    max_poll_gap_ms: f64,
}
#[derive(Default)]
struct SubmissionBracket {
    lower_ms: f64,
    positive_ms: Option<f64>,
}
impl SubmissionBracket {
    fn completion(
        &mut self,
        before: f64,
        after: f64,
        value: skin_editor_infrastructure::VoiceSubmission,
    ) -> Result<Option<[f64; 2]>, String> {
        use skin_editor_infrastructure::VoiceSubmission;
        match value {
            VoiceSubmission::Complete(_) => Ok(self.observe(before, after, true)),
            VoiceSubmission::Pending => Ok(self.observe(before, after, false)),
            VoiceSubmission::Unknown => Ok(self.positive_ms.map(|upper| [self.lower_ms, upper])),
            VoiceSubmission::Superseded | VoiceSubmission::Invalidated => {
                Err(format!("probe completion lost identity: {value:?}"))
            }
        }
    }
    fn observe(&mut self, before_ms: f64, after_ms: f64, submitted: bool) -> Option<[f64; 2]> {
        if let Some(upper) = self.positive_ms {
            Some([self.lower_ms, upper])
        } else if submitted {
            self.positive_ms = Some(after_ms);
            Some([self.lower_ms, after_ms])
        } else {
            // The atomic observation happened somewhere inside the read. Only
            // its BEFORE timestamp is a conservative lower bound on submission.
            self.lower_ms = before_ms;
            None
        }
    }
}
impl Observer {
    fn new(output: OutputControl) -> Self {
        Self::with_capacity(output, 64).expect("fixed observer capacity")
    }
    fn with_capacity(output: OutputControl, capacity: usize) -> Result<Self, String> {
        if !(1..=256).contains(&capacity) {
            return Err("invalid bounded observer capacity".into());
        }
        let (tx, rx) = mpsc::sync_channel::<Probe>(1);
        let thread = std::thread::spawn(move || {
            let mut observations = [Vec::with_capacity(capacity), Vec::with_capacity(capacity)];
            let mut intervals = [Vec::with_capacity(capacity), Vec::with_capacity(capacity)];
            let mut causal = [Vec::with_capacity(capacity), Vec::with_capacity(capacity)];
            let mut exact = [Vec::with_capacity(capacity), Vec::with_capacity(capacity)];
            let mut markers = [Vec::with_capacity(capacity), Vec::with_capacity(capacity)];
            let mut requested = [Vec::with_capacity(capacity), Vec::with_capacity(capacity)];
            let mut request_phases = [Vec::with_capacity(capacity), Vec::with_capacity(capacity)];
            let mut request_refs = [Vec::with_capacity(capacity), Vec::with_capacity(capacity)];
            let mut max_poll_gap_ms: f64 = 0.;
            while let Ok(probe) = rx.recv() {
                let until = Instant::now() + Duration::from_secs(5);
                let mut previous = Instant::now();
                let first_poll_ms = probe.started.elapsed().as_secs_f64() * 1000.;
                let mut bracket = SubmissionBracket::default();
                let mut precise = SubmissionBracket::default();
                let mut completion = None;
                let (bounds, trace, exact_bounds) = loop {
                    let now = Instant::now();
                    max_poll_gap_ms =
                        max_poll_gap_ms.max(now.duration_since(previous).as_secs_f64() * 1000.);
                    previous = now;
                    let before_ms = now.duration_since(probe.started).as_secs_f64() * 1000.;
                    let submitted = output.stats().submitted_voice_ids[probe.lane] >= probe.voice;
                    let after_ms = probe.started.elapsed().as_secs_f64() * 1000.;
                    let bounds = bracket.observe(before_ms, after_ms, submitted);
                    let exact_bounds = if probe.clock.is_some() {
                        let before = probe.started.elapsed().as_secs_f64() * 1000.;
                        let value = output.voice_submission(probe.lane, probe.voice)?;
                        let after = probe.started.elapsed().as_secs_f64() * 1000.;
                        use skin_editor_infrastructure::VoiceSubmission;
                        if let VoiceSubmission::Complete(row) = value {
                            completion = Some(row);
                        }
                        precise.completion(before, after, value)?
                    } else {
                        bounds
                    };
                    if let (Some(bounds), Some(exact_bounds)) = (bounds, exact_bounds) {
                        let trace = output.submission_trace(probe.lane, probe.voice);
                        if probe.clock.is_none() || trace.is_some() {
                            break (bounds, trace, exact_bounds);
                        }
                        // Callback may finish before the producer publishes its
                        // after-commit telemetry. Preserve the FIRST positive;
                        // waiting for diagnostics must not inflate the gate.
                    }
                    if output.is_closed() || now >= until {
                        return Err("native probe timeout".into());
                    }
                    // Control-side observation only. Keep the original bounded
                    // sleep cadence and report its actual polling uncertainty.
                    std::thread::sleep(Duration::from_millis(1));
                };
                if observations[probe.lane].len() >= capacity {
                    return Err("probe trace overflow".into());
                }
                if let Some(clock) = probe.clock {
                    let trace = trace.ok_or("exact Voice trace unavailable")?;
                    let p = trace.producer;
                    let c = trace.callback;
                    if completion != Some(c) {
                        return Err("completion/producer Voice epoch serial mismatch".into());
                    }
                    exact[probe.lane].push(exact_bounds);
                    markers[probe.lane].push(c);
                    requested[probe.lane].push(probe.voice);
                    let reference = probe
                        .callback_reference
                        .ok_or("missing request callback reference")?;
                    let reference_bounds = clock.callback_bounds_ms(reference[1], probe.started);
                    request_phases[probe.lane].push([-reference_bounds[1], -reference_bounds[0]]);
                    request_refs[probe.lane].push(reference);
                    let offset = probe.started.duration_since(trace.origin).as_secs_f64() * 1000.;
                    let relative = |ns: u64| ns as f64 / 1_000_000. - offset;
                    let cb = clock.callback_bounds_ms(c[3], probe.started);
                    if cb[0] > bounds[1] || relative(p[4]) > bounds[1] {
                        return Err("stream clock/commit outside observation".into());
                    }
                    causal[probe.lane].push([
                        relative(p[3]),
                        relative(p[4]),
                        relative(p[5]),
                        cb[0],
                        cb[1],
                        p[6] as f64,
                        p[7] as f64,
                        p[8] as f64,
                        c[4] as f64,
                        c[5] as f64,
                        c[2] as f64,
                        c[1] as f64,
                        c[6] as f64,
                    ]);
                }
                observations[probe.lane].push(bounds[1]);
                intervals[probe.lane].push([
                    bounds[0],
                    bounds[1],
                    probe.receipt_ms,
                    first_poll_ms,
                    probe.started_in_play_ms,
                ]);
                if let Some(done) = probe.completed {
                    done.try_send(()).map_err(error)?;
                }
            }
            Ok(ProbeObservations {
                latencies: observations,
                intervals,
                causal,
                exact,
                markers,
                requested,
                request_phases,
                request_refs,
                max_poll_gap_ms,
            })
        });
        Ok(Self {
            sender: Some(tx),
            thread: Some(thread),
        })
    }
    fn finish(&mut self) -> Result<ProbeObservations, String> {
        self.sender.take();
        self.thread
            .take()
            .ok_or("observer already closed")?
            .join()
            .map_err(|_| "observer panic")?
    }
}
impl Drop for Observer {
    fn drop(&mut self) {
        self.sender.take();
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

pub(crate) fn fixture(editor: &mut Editor, path: &std::path::Path) -> Result<u128, String> {
    build_fixture(editor, path, true, true)
}
pub(crate) fn visual_fixture(editor: &mut Editor, path: &std::path::Path) -> Result<u128, String> {
    build_fixture(editor, path, false, true)
}
pub(crate) fn reopen_fixture(editor: &mut Editor, path: &std::path::Path) -> Result<u128, String> {
    build_fixture(editor, path, true, false)
}
fn build_fixture(
    editor: &mut Editor,
    path: &std::path::Path,
    include_audio: bool,
    create_images: bool,
) -> Result<u128, String> {
    editor.session = fresh_session(&editor.images)?;
    let mut commands = Vec::new();
    let mut pins = Vec::new();
    for i in 0..100usize {
        if create_images {
            let mut pixels = vec![0u8; 1024 * 1024 * 4];
            for (n, p) in pixels.chunks_exact_mut(4).enumerate() {
                let x = n % 1024;
                let y = n / 1024;
                p.copy_from_slice(&[
                    (i * 2) as u8,
                    (x / 8) as u8,
                    (y / 8) as u8,
                    if (x / 64 + y / 64 + i).is_multiple_of(3) {
                        180
                    } else {
                        96
                    },
                ]);
            }
            let file = path.join(format!("self-{i:03}.png"));
            write_png(&file, &pixels, PixelSize::new(1024, 1024).unwrap())?;
        }
        let file = path.join(format!("self-{i:03}.png"));
        let staged = editor
            .images
            .stage_png(file.to_str().ok_or("fixture path encoding")?)
            .map_err(error)?;
        commands.push(Command::RegisterAsset {
            asset: Asset::new(
                AssetId::new(i as u128 + 1000).unwrap(),
                format!("self {i}.png"),
                staged.blob,
                AssetKind::RasterImage(staged.size),
            )
            .map_err(error)?,
        });
        pins.push(staged.lease);
    }
    for d in 0..10 {
        commands.push(Command::CreateDocument {
            document: ElementDocument::new(
                document(d),
                PixelSize::new(1024, 1024).unwrap(),
                LayerTree::default(),
            ),
        });
        for n in 0..50 {
            let group = n < 2;
            commands.push(Command::AddLayer {
                document: document(d),
                node: LayerNode {
                    id: layer(d * 50 + n),
                    local: if group {
                        Affine2::IDENTITY
                    } else {
                        Affine2::new([
                            0.35,
                            0.,
                            0.,
                            0.35,
                            ((n % 4) * 200) as f64,
                            ((n / 4 % 4) * 200) as f64,
                        ])
                        .unwrap()
                    },
                    opacity: Opacity::new(if group { 0.9 } else { 0.4 }).unwrap(),
                    visible: true,
                    kind: if group {
                        LayerKind::Group { children: vec![] }
                    } else {
                        LayerKind::Image {
                            asset: AssetId::new((1000 + d * 10 + (n - 2) % 10) as u128).unwrap(),
                        }
                    },
                },
                parent: (n > 0).then(|| layer(d * 50 + usize::from(n > 1))),
                index: if n > 1 { n - 2 } else { 0 },
            });
        }
    }
    let blobs = editor.images.blobs();
    for n in 0..if include_audio { 16 } else { 0 } {
        let raw = blobs
            .stage_audio(wav(n as u32), &skin_editor_infrastructure::WavDecoder)
            .map_err(error)?;
        let asset = AssetId::new(n as u128 + 1).unwrap();
        commands.push(Command::RegisterAsset {
            asset: Asset::new(
                asset,
                format!("self {n}.wav"),
                raw.reference(),
                AssetKind::Audio(raw.audio_metadata().unwrap()),
            )
            .map_err(error)?,
        });
        commands.push(Command::CreateAudioComponent {
            component: AudioComponent::new(
                sound(n),
                SoundSlotKey::new(format!("self.combined{n}")).map_err(error)?,
                asset,
            ),
        });
        pins.push(raw.lease());
    }
    let mut next = 100_000;
    submit(&mut editor.session, &mut next, commands)?;
    editor.ui.target = Some(PreviewTarget::Document(document(0)));
    editor.ui.select(&mut editor.session, None);
    Ok(next)
}

struct Combined {
    timing_origin: Instant,
    timing: String,
    cadence: cadence::Cadence,
    late_supply: Vec<String>,
    diagnostic_cycle: Option<usize>,
    hold_completion: bool,
    completed: bool,
    control_only: bool,
    cold_control: bool,
    memory_usage: bool,
    reusable_staging: bool,
    editor: Editor,
    path: PathBuf,
    budget: PcmBudget,
    client: Option<AudioSupplyClient>,
    stream: Option<NativeStream>,
    output: Option<OutputControl>,
    final_stats: OutputStats,
    playing_frames: u64,
    observer: Observer,
    next: u128,
    started: Option<Instant>,
    frames_at_start: u64,
    loading: Instant,
    closing: Option<Instant>,
    failed: Option<String>,
    done: bool,
    triggers: usize,
    edits: usize,
    switches: usize,
    probes: usize,
    edit_pending: Option<Instant>,
    edit_ms: Vec<f64>,
    frame_ms: Vec<f64>,
    slow_ui: Vec<[f64; 4]>,
    latencies: [Vec<f64>; 2],
    intervals: [Vec<[f64; 5]>; 2],
    causal: [Vec<[f64; 13]>; 2],
    gpu_snapshots: Vec<String>,
    exact_intervals: [Vec<[f64; 2]>; 2],
    completion_markers: [Vec<[u64; 7]>; 2],
    requested_voices: [Vec<u64>; 2],
    request_phases: [Vec<[f64; 2]>; 2],
    request_refs: [Vec<[u64; 4]>; 2],
    presentation: Option<String>,
    max_probe_poll_gap_ms: f64,
    max_supply_gap_micros: u64,
    underrun_observations: Vec<[u64; 6]>,
    max_targets: usize,
    max_rgba: usize,
    max_voices: usize,
    max_trigger_lateness_ms: f64,
    rate: u32,
    channels: u16,
    period: u32,
    adapter: String,
    cleanup: bool,
    isolation: bool,
    draft_history_cleanup: bool,
    opacities: [f32; 10],
}
impl Combined {
    fn new(
        context: &eframe::CreationContext<'_>,
        path: PathBuf,
        control_only: bool,
        memory_usage: bool,
        cold_control: bool,
        reusable_staging: bool,
    ) -> Result<Self, String> {
        Self::create(
            &context.egui_ctx,
            context.wgpu_render_state.as_ref().unwrap(),
            path,
            control_only,
            memory_usage,
            cold_control,
            reusable_staging,
        )
    }
    fn create(
        ctx: &egui::Context,
        state: &eframe::egui_wgpu::RenderState,
        path: PathBuf,
        control_only: bool,
        memory_usage: bool,
        cold_control: bool,
        reusable_staging: bool,
    ) -> Result<Self, String> {
        let device = &state.device;
        let mut gpu_snapshots = vec![crate::gpu_observation::snapshot(device, "before_editor")?];
        let mut editor = Editor::with_render_state(
            ctx,
            state,
            false,
            if reusable_staging {
                skin_editor_rendering::SourceUpload::ReusableStaging
            } else {
                skin_editor_rendering::SourceUpload::QueueWrite
            },
        )?;
        gpu_snapshots.push(crate::gpu_observation::snapshot(device, "editor_created")?);
        editor.host.enable_trace();
        let next = fixture(&mut editor, &path)?;
        let budget = PcmBudget::default();
        let native = NativeOutput::open(&budget)?;
        let (rate, channels, period) = (native.rate, native.channels, native.period_frames);
        let (stream, producer) = native.into_parts();
        let mut client = AudioSupplyClient::spawn(
            producer,
            rate,
            channels,
            budget.clone(),
            editor.images.blobs().clone(),
        )
        .map_err(error)?;
        let output = client.output().map_err(|e| format!("{e:?}"))?;
        let timing_origin = output.capture_submissions()?;
        stream.play()?;
        editor
            .session
            .synchronize_audio(&mut client)
            .map_err(error)?;
        let adapter = format!("{:?}", state.adapter.get_info());
        Ok(Self {
            timing_origin,
            timing: "null".into(),
            cadence: cadence::Cadence::default(),
            late_supply: Vec::with_capacity(8),
            diagnostic_cycle: None,
            hold_completion: false,
            completed: false,
            control_only,
            cold_control,
            memory_usage,
            reusable_staging,
            editor,
            path,
            budget,
            client: Some(client),
            stream: Some(stream),
            observer: Observer::new(output.clone()),
            output: Some(output),
            final_stats: OutputStats::default(),
            playing_frames: 0,
            next,
            started: None,
            frames_at_start: 0,
            loading: Instant::now(),
            closing: None,
            failed: None,
            done: false,
            triggers: 0,
            edits: 0,
            switches: 0,
            probes: 0,
            edit_pending: None,
            edit_ms: Vec::with_capacity(300),
            frame_ms: Vec::with_capacity(25000),
            slow_ui: Vec::with_capacity(256),
            latencies: [vec![], vec![]],
            intervals: [vec![], vec![]],
            causal: [vec![], vec![]],
            exact_intervals: [vec![], vec![]],
            completion_markers: [vec![], vec![]],
            requested_voices: [vec![], vec![]],
            request_phases: [vec![], vec![]],
            request_refs: [vec![], vec![]],
            gpu_snapshots,
            presentation: None,
            max_probe_poll_gap_ms: 0.,
            max_supply_gap_micros: 0,
            underrun_observations: vec![],
            max_targets: 0,
            max_rgba: 0,
            max_voices: 0,
            max_trigger_lateness_ms: 0.,
            rate,
            channels,
            period,
            adapter,
            cleanup: false,
            isolation: false,
            draft_history_cleanup: false,
            opacities: [0.9; 10],
        })
    }
    fn tick(&mut self) -> Result<(), String> {
        if self.closing.is_some() {
            return Ok(());
        }
        let client = self.client.as_mut().unwrap();
        while let Some(r) = client.poll() {
            match r.result.map_err(error)? {
                SupplyOutcome::Triggered(r) if r.voice.is_none() || r.stale => {
                    return Err(format!("standard trigger refused: {r:?}"));
                }
                _ => {}
            }
        }
        let status = client.status().map_err(error)?;
        self.max_supply_gap_micros = self.max_supply_gap_micros.max(status.max_supply_gap_micros);
        let expected_origin = source(&self.editor.session);
        let ready = status.ready_components == 16
            && status.origin == Some(expected_origin)
            && status.bindings.iter().all(|(_, b)| {
                b.desired == expected_origin
                    && b.effective == Some(expected_origin)
                    && b.preparation == Preparation::Ready
            });
        if let Some(fault) = status.fault {
            return Err(fault);
        }
        if self.output.as_ref().unwrap().is_closed() {
            return Err("native output failed".into());
        }
        self.max_voices = self.max_voices.max(status.voices.iter().sum());
        for (id, b) in &status.bindings {
            if b.preparation == Preparation::Failed(PrepareFailure::Busy) {
                let request = client.retry_selected(*id).map_err(error)?;
                outcome(client, request)?;
            }
        }
        if self.started.is_none() {
            if self.loading.elapsed() > Duration::from_secs(30) {
                return Err(format!(
                    "combined preparation timeout: status={status:?}, draft={:?}, ui_passes={}, publications={}, ui_message={}",
                    self.editor
                        .session
                        .draft_info(self.editor.session.project_info().session_id),
                    self.editor.ui_passes,
                    self.editor.host.publications,
                    self.editor.ui.message
                ));
            }
            if ready && self.editor.host.publications > 0 && self.editor.ui_passes >= 5 {
                if self
                    .editor
                    .session
                    .draft_info(self.editor.session.project_info().session_id)
                    .map_err(error)?
                    .is_none()
                {
                    self.next += 1;
                    self.editor
                        .session
                        .edit_draft(DraftRequest {
                            api_version: API_VERSION,
                            session_id: self.editor.session.project_info().session_id,
                            request_id: RequestId::new(self.next).unwrap(),
                            action: DraftAction::Begin {
                                expected: self.editor.session.project_info().state,
                            },
                        })
                        .map_err(error)?;
                    self.editor
                        .session
                        .synchronize_audio(client)
                        .map_err(error)?;
                    return Ok(());
                }
                let id = client.scene_running(true).map_err(error)?;
                outcome(client, id)?;
                self.frames_at_start = self.output.as_ref().unwrap().stats().frames;
                self.started = Some(Instant::now());
                self.gpu_snapshots.push(crate::gpu_observation::snapshot(
                    &self.editor.host.device,
                    "playing_start",
                )?);
                if let Some(cycle) = self.diagnostic_cycle.filter(|c| *c > 0) {
                    phase(&format!("cycle_{:02}_playing", cycle + 1));
                } else {
                    phase("playing");
                }
            }
            return Ok(());
        }
        let elapsed = self.started.unwrap().elapsed();
        // Five fixed checkpoints, outside the callback and outside preparation.
        // This observes allocator capacity separately from OS GPU process usage.
        for (index, (seconds, stage)) in [
            (2, "playing_2s"),
            (60, "playing_60s"),
            (200, "playing_200s"),
            (299, "playing_299s"),
        ]
        .into_iter()
        .enumerate()
        {
            if self.gpu_snapshots.len() == index + 3 && elapsed >= Duration::from_secs(seconds) {
                self.gpu_snapshots.push(crate::gpu_observation::snapshot(
                    &self.editor.host.device,
                    stage,
                )?);
            }
        }
        if self.control_only
            && elapsed >= Duration::from_secs(if self.cold_control { 14 } else { 3 })
        {
            if self.hold_completion {
                self.completed = true;
            } else {
                self.finish()?;
            }
            return Ok(());
        }
        if ready && let Some(start) = self.edit_pending.take() {
            self.edit_ms.push(start.elapsed().as_secs_f64() * 1000.);
        }
        if self.probes < 64 && elapsed >= Duration::from_millis(250 + self.probes as u64 * 4000) {
            if !ready {
                return Err("audition probe was not ready".into());
            }
            let callback_reference = self.output.as_ref().unwrap().callback_progress();
            let clock = Some(self.stream.as_ref().unwrap().clock_sample()?);
            let started = Instant::now();
            let id = client
                .trigger_selected(Channel::Audition, sound(0))
                .map_err(error)?;
            let voice = match outcome(client, id)? {
                SupplyOutcome::Triggered(r) if !r.stale => {
                    r.voice.ok_or("audition probe refused")?
                }
                other => return Err(format!("audition probe failed: {other:?}")),
            };
            self.observer
                .sender
                .as_ref()
                .unwrap()
                .try_send(Probe {
                    callback_reference,
                    completed: None,
                    started,
                    receipt_ms: started.elapsed().as_secs_f64() * 1000.,
                    started_in_play_ms: started.duration_since(self.started.unwrap()).as_secs_f64()
                        * 1000.,
                    voice,
                    lane: 1,
                    clock,
                })
                .map_err(error)?;
            self.probes += 1;
        }
        while self.triggers < TRIGGERS
            && elapsed >= Duration::from_millis(self.triggers as u64 * 125)
        {
            let lateness = elapsed.as_secs_f64() * 1000. - self.triggers as f64 * 125.;
            self.max_trigger_lateness_ms = self.max_trigger_lateness_ms.max(lateness);
            if lateness > 125. {
                return Err(format!("missed standard trigger deadline: {lateness} ms"));
            }
            if !ready {
                return Err("scene trigger during incomplete preparation".into());
            }
            let observe = self.triggers % 32 == 5 && self.triggers / 32 < 64;
            let callback_reference = if observe {
                self.output.as_ref().unwrap().callback_progress()
            } else {
                None
            };
            let clock = if observe {
                Some(self.stream.as_ref().unwrap().clock_sample()?)
            } else {
                None
            };
            let started = Instant::now();
            let id = client
                .trigger_selected(Channel::Scene, sound(self.triggers % 16))
                .map_err(error)?;
            if observe {
                let voice = match outcome(client, id)? {
                    SupplyOutcome::Triggered(r) if !r.stale => {
                        r.voice.ok_or("scene probe refused")?
                    }
                    other => return Err(format!("scene probe failed: {other:?}")),
                };
                self.observer
                    .sender
                    .as_ref()
                    .unwrap()
                    .try_send(Probe {
                        callback_reference,
                        completed: None,
                        started,
                        receipt_ms: started.elapsed().as_secs_f64() * 1000.,
                        started_in_play_ms: started
                            .duration_since(self.started.unwrap())
                            .as_secs_f64()
                            * 1000.,
                        voice,
                        lane: 0,
                        clock,
                    })
                    .map_err(error)?;
            }
            self.triggers += 1;
        }
        if self.edits < EDITS && elapsed >= Duration::from_millis(500 + self.edits as u64 * 1000) {
            if self.edit_pending.is_some() {
                return Err("edit activation exceeded next edit".into());
            }
            let d = (self.switches % 10).min(9);
            let started = Instant::now();
            self.opacities[d] = 0.5 + (self.edits % 40) as f32 / 100.;
            let mut commands: Vec<_> = self
                .opacities
                .iter()
                .enumerate()
                .map(|(d, alpha)| Command::SetLayerProperties {
                    document: document(d),
                    layer: layer(d * 50),
                    opacity: Opacity::new(*alpha).unwrap(),
                    visible: true,
                })
                .collect();
            commands.push(Command::SetAudioProperties {
                component: sound(0),
                gain: Gain::new(0.4 + (self.edits % 50) as f64 / 100.).unwrap(),
                visible: true,
            });
            let expected = self
                .editor
                .session
                .draft_info(self.editor.session.project_info().session_id)
                .map_err(error)?
                .ok_or("missing load draft")?
                .state;
            self.next += 1;
            self.editor
                .session
                .edit_draft(DraftRequest {
                    api_version: API_VERSION,
                    session_id: self.editor.session.project_info().session_id,
                    request_id: RequestId::new(self.next).unwrap(),
                    action: DraftAction::Replace { expected, commands },
                })
                .map_err(error)?;
            self.editor
                .session
                .synchronize_audio(client)
                .map_err(error)?;
            self.edit_pending = Some(started);
            self.edits += 1;
        }
        if self.switches < SWITCHES
            && elapsed
                >= Duration::from_millis(if self.cold_control {
                    600 * (1 + self.switches as u64)
                } else {
                    1000 + self.switches as u64 * 10000
                })
        {
            self.switches += 1;
            self.editor.ui.target = Some(PreviewTarget::Document(document(self.switches % 10)));
        }
        if elapsed >= Duration::from_secs(SECONDS)
            && self.output.as_ref().unwrap().stats().frames - self.frames_at_start
                >= u64::from(self.rate) * SECONDS
        {
            if self.triggers != TRIGGERS
                || self.edits != EDITS
                || self.switches != SWITCHES
                || self.edit_ms.len() != EDITS
            {
                return Err("incomplete standard workload".into());
            }
            if self.hold_completion {
                self.completed = true;
            } else {
                self.finish()?;
            }
        }
        Ok(())
    }
    fn finish(&mut self) -> Result<(), String> {
        if !self.hold_completion || self.playing_frames == 0 {
            self.playing_frames =
                self.output.as_ref().unwrap().stats().frames - self.frames_at_start;
        }
        phase("cleanup");
        let observed = self.observer.finish()?;
        self.latencies = observed.latencies;
        self.intervals = observed.intervals;
        self.causal = observed.causal;
        self.exact_intervals = observed.exact;
        self.completion_markers = observed.markers;
        self.requested_voices = observed.requested;
        self.request_phases = observed.request_phases;
        self.request_refs = observed.request_refs;
        self.max_probe_poll_gap_ms = observed.max_poll_gap_ms;
        let client = self.client.as_mut().unwrap();
        stop(client, self.output.as_ref().unwrap())?;
        let before_commit = self.editor.session.history_len();
        let expected = self
            .editor
            .session
            .draft_info(self.editor.session.project_info().session_id)
            .map_err(error)?
            .ok_or("missing load draft")?
            .state;
        self.next += 1;
        self.editor
            .session
            .edit_draft(DraftRequest {
                api_version: API_VERSION,
                session_id: self.editor.session.project_info().session_id,
                request_id: RequestId::new(self.next).unwrap(),
                action: DraftAction::Commit { expected },
            })
            .map_err(error)?;
        if before_commit != 2 || self.editor.session.history_len() != 3 {
            return Err("300 draft updates did not commit exactly once".into());
        }
        self.editor
            .session
            .synchronize_audio(client)
            .map_err(error)?;
        ready_all(client, self.output.as_ref().unwrap(), &self.editor.session)?;
        let before = self.output.as_ref().unwrap().stats();
        let id = client
            .trigger_selected(Channel::Audition, sound(0))
            .map_err(error)?;
        outcome(client, id)?;
        wait(self.output.as_ref().unwrap(), || {
            self.output.as_ref().unwrap().stats().lane_nonzero_frames[1]
                > before.lane_nonzero_frames[1]
        })?;
        self.isolation = self.output.as_ref().unwrap().stats().lane_nonzero_frames[0]
            == before.lane_nonzero_frames[0];
        stop(client, self.output.as_ref().unwrap())?;
        // Draft cancellation and history movement invalidate the native epoch
        // immediately, including already-generated Copy frames; inspect ack.
        let base = self.editor.session.snapshot();
        let count = self.editor.session.history_len();
        self.next += 1;
        self.editor
            .session
            .edit_draft(DraftRequest {
                api_version: API_VERSION,
                session_id: base.session_id,
                request_id: RequestId::new(self.next).unwrap(),
                action: DraftAction::Begin {
                    expected: base.state,
                },
            })
            .map_err(error)?;
        self.editor
            .session
            .synchronize_audio(client)
            .map_err(error)?;
        play_old(client, self.output.as_ref().unwrap(), &self.editor.session)?;
        let expected = self
            .editor
            .session
            .draft_info(base.session_id)
            .map_err(error)?
            .unwrap()
            .state;
        self.next += 1;
        self.editor
            .session
            .edit_draft(DraftRequest {
                api_version: API_VERSION,
                session_id: base.session_id,
                request_id: RequestId::new(self.next).unwrap(),
                action: DraftAction::Cancel { expected },
            })
            .map_err(error)?;
        self.editor
            .session
            .synchronize_audio(client)
            .map_err(error)?;
        let mut cleaned = cleared(client, self.output.as_ref().unwrap())?;
        for action in [HistoryAction::Undo, HistoryAction::Redo { child: None }] {
            play_old(client, self.output.as_ref().unwrap(), &self.editor.session)?;
            self.next += 1;
            self.editor
                .session
                .navigate(HistoryRequest {
                    api_version: API_VERSION,
                    session_id: base.session_id,
                    request_id: RequestId::new(self.next).unwrap(),
                    expected: self.editor.session.project_info().state,
                    action,
                })
                .map_err(error)?;
            self.editor
                .session
                .synchronize_audio(client)
                .map_err(error)?;
            cleaned &= cleared(client, self.output.as_ref().unwrap())?;
        }
        stop(client, self.output.as_ref().unwrap())?;
        let now = self.output.as_ref().unwrap().stats();
        self.draft_history_cleanup = cleaned
            && self.editor.session.snapshot().project == base.project
            && self.editor.session.history_len() == count
            && now
                .lane_acknowledged
                .iter()
                .zip(now.lane_epochs)
                .all(|(a, e)| *a >= e)
            && client.status().map_err(error)?.voices == [0, 0];
        self.final_stats = self.output.as_ref().unwrap().stats();
        self.underrun_observations = self.output.as_ref().unwrap().underrun_observations();
        self.max_supply_gap_micros = self
            .max_supply_gap_micros
            .max(client.status().map_err(error)?.max_supply_gap_micros);
        client.close();
        self.client.take();
        let alignment =
            clock_alignment(self.output.as_ref().unwrap(), self.stream.as_ref().unwrap())?;
        self.stream.take();
        self.timing = scheduling_json(self.output.as_ref().unwrap(), alignment, true);
        self.output.take();
        drop(base);
        self.editor.host.shutdown();
        self.gpu_snapshots.push(crate::gpu_observation::snapshot(
            &self.editor.host.device,
            "after_shutdown",
        )?);
        self.editor.session = fresh_session(&self.editor.images)?;
        self.editor.ui.target = None;
        self.editor.ui.select(&mut self.editor.session, None);
        self.closing = Some(Instant::now());
        phase("closed");
        Ok(())
    }
    fn report(&mut self, file: &str) -> Result<(), String> {
        self.gpu_snapshots.push(crate::gpu_observation::snapshot(
            &self.editor.host.device,
            "report",
        )?);
        if self.gpu_snapshots.len() > 16 {
            return Err("GPU snapshot trace overflow".into());
        }
        self.editor.host.poll().map_err(error)?;
        let raw = self.editor.images.blobs().usage().map_err(error)?;
        let rgba = self.editor.images.rgba_bytes();
        let staging = self.editor.host.staging_usage();
        self.cleanup = staging.bytes == 0
            && self.budget.usage().bytes == 0
            && raw.bytes == 0
            && rgba == 0
            && self.editor.host.live_targets() == 0;
        let s = self.final_stats;
        let preparation = self.editor.host.traces.iter().map(|t| format!(
            "{{\"source_utf8\":{:?},\"started_ms\":{},\"plan_ms\":{},\"decode_ms\":{},\"pack_ms\":{},\"upload_ms\":{},\"encode_submit_ms\":{},\"reused\":{},\"queue_ms\":{},\"worker_started_ms\":{},\"worker_finished_ms\":{},\"received_ms\":{},\"gpu_done_ms\":{},\"source_lock_ms\":{},\"submission_lock_ms\":{},\"staging_wait_ms\":{},\"staging_alloc_ms\":{},\"staging_copy_ms\":{},\"write_ms\":{},\"flush_ms\":{},\"create_ms\":{},\"published_ms\":{}}}",
            format!("{:?}", t.source).as_bytes(), t.started_ms, t.plan_ms, t.decode_ms,
            t.gpu.pack_ms, t.gpu.upload_ms, t.gpu.encode_submit_ms, t.gpu.reused,
            t.queue_ms,t.worker_started_ms,t.worker_finished_ms,t.received_ms,t.gpu_done_ms.map_or("null".into(),|v| v.to_string()),
            t.gpu.source_lock_ms,t.gpu.submission_lock_ms,t.gpu.staging_wait_ms,t.gpu.staging_alloc_ms,t.gpu.staging_copy_ms,t.gpu.write_ms,t.gpu.flush_ms,t.gpu.create_ms,
            t.published_ms.map_or("null".into(), |v| v.to_string())
        )).collect::<Vec<_>>().join(",");
        // Adapter is diagnostic text encoded as bytes, avoiding a second JSON
        // implementation or an unreviewed serde dependency in the product.
        let report = format!(
            concat!(
                "{{\"schema\":1,\"gpu_memory_usage_hint\":{},\"reusable_staging\":{},\"seconds\":{},\"rate\":{},\"channels\":{},\"period_frames\":{},",
                "\"original_pngs\":100,\"documents\":10,\"layers\":500,\"triggers\":{},\"edits\":{},\"switches\":{},",
                "\"native_play_frames\":{},\"empty_frames\":{},\"empty_callbacks\":{},\"max_empty_callback_frames\":{},\"callbacks\":{},\"max_callback_frames\":{},",
                "\"latency_upper_bound_ms\":{:?},\"submission_intervals_ms\":{:?},\"submission_causal_ms\":{:?},\"gpu_allocator_snapshots\":[{}],\"presentation\":{},\"edit_activation_ms\":{:?},\"cpu_frame_ms\":{:?},",
                "\"exact_voice_intervals_ms\":{:?},\"completion_markers\":{:?},",
                "\"requested_voice_ids\":{:?},",
                "\"request_phase_age_ms\":{:?},\"request_callback_refs\":{:?},",
                "\"peak_voices\":{},\"max_trigger_lateness_ms\":{},\"pcm_peak_bytes\":{},\"pcm_final_bytes\":{},",
                "\"raw_peak_bytes\":{},\"raw_final_bytes\":{},\"rgba_peak_bytes\":{},\"rgba_final_bytes\":{},",
                "\"max_targets\":{},\"final_targets\":{},\"allocations\":{},\"publications\":{},",
                "\"source_uploads\":{},\"source_cache_final_bytes\":{},\"source_allocations\":{},\"staging_peak_bytes\":{},\"staging_final_bytes\":{},\"staging_allocations\":{},\"staging_copies\":{},\"staging_remaps\":{},",
                "\"max_probe_poll_gap_ms\":{},\"max_supply_gap_micros\":{},\"underrun_observations\":{:?},",
                "\"preparation_trace\":[{}],\"slow_ui_ms\":{:?},\"trace_overflow\":{},",
                "\"scheduling\":{},\"host_cadence\":{{\"calls\":{},\"late_count\":{},\"rows\":{:?},\"playing_origin_ns\":{},\"late_supply\":[{}]}},",
                "\"dual_channel_isolation\":{},\"draft_history_cleanup\":{},\"resources_released\":{},\"adapter_utf8\":{:?}}}\n"
            ),
            self.memory_usage,
            self.reusable_staging,
            if self.cold_control {
                14
            } else if self.control_only {
                3
            } else {
                SECONDS
            },
            self.rate,
            self.channels,
            self.period,
            self.triggers,
            self.edits,
            self.switches,
            self.playing_frames,
            s.empty_frames,
            s.empty_callbacks,
            s.max_empty_callback_frames,
            s.callbacks,
            s.max_callback_frames,
            self.latencies,
            self.intervals,
            self.causal,
            self.gpu_snapshots.join(","),
            self.presentation.as_deref().unwrap_or("null"),
            self.edit_ms,
            self.frame_ms,
            self.exact_intervals,
            self.completion_markers,
            self.requested_voices,
            self.request_phases,
            self.request_refs,
            self.max_voices,
            self.max_trigger_lateness_ms,
            self.budget.usage().peak_bytes,
            self.budget.usage().bytes,
            raw.peak_bytes,
            raw.bytes,
            self.editor.images.rgba_peak_bytes(),
            rgba,
            self.max_targets,
            self.editor.host.live_targets(),
            self.editor.host.allocations,
            self.editor.host.publications,
            self.editor.host.source_usage().0,
            self.editor.host.source_usage().1,
            self.editor.host.source_allocations(),
            staging.peak_bytes,
            staging.bytes,
            staging.allocations,
            staging.copies,
            staging.remaps,
            self.max_probe_poll_gap_ms,
            self.max_supply_gap_micros,
            self.underrun_observations,
            preparation,
            self.slow_ui,
            self.editor.host.trace_overflow,
            self.timing,
            self.cadence.calls,
            self.cadence.late_count,
            self.cadence.rows,
            self.started.map_or(
                0,
                |t| t.saturating_duration_since(self.timing_origin).as_nanos() as u64
            ),
            self.late_supply.join(","),
            self.isolation,
            self.draft_history_cleanup,
            self.cleanup,
            self.adapter.as_bytes()
        );
        std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(self.path.join(file))
            .map_err(error)?
            .write_all(report.as_bytes())
            .map_err(error)?;
        self.done = file == "report.json";
        Ok(())
    }
}
impl Combined {
    fn ns(&self) -> u64 {
        self.timing_origin.elapsed().as_nanos() as u64
    }
    fn logic_inner(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if self.failed.is_some() || self.done {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        let result = self.tick();
        self.cadence.tick_end(self.ns());
        if let Err(e) = result {
            self.failed = Some(e);
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        if let Some(started) = self.closing {
            let _ = self.editor.host.poll();
            if started.elapsed() >= Duration::from_secs(2) {
                if let Err(e) = self.report("report.json") {
                    self.failed = Some(e);
                }
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        } else {
            eframe::App::logic(&mut self.editor, ctx, frame);
        }
        // Static canvas: use real control deadlines and normal GPU wakeups,
        // rather than adding a synthetic 60 Hz repaint absent from Editor.
        let delay = if let Some(start) = self.started {
            let elapsed = start.elapsed();
            let mut next = Duration::from_secs(SECONDS);
            if self.triggers < TRIGGERS {
                next = next.min(Duration::from_millis(self.triggers as u64 * 125));
            }
            if self.edits < EDITS {
                next = next.min(Duration::from_millis(500 + self.edits as u64 * 1000));
            }
            if self.probes < 64 {
                next = next.min(Duration::from_millis(250 + self.probes as u64 * 4000));
            }
            if self.edit_pending.is_some() {
                next = next.min(elapsed + Duration::from_millis(5));
            }
            next.saturating_sub(elapsed).max(Duration::from_millis(1))
        } else {
            Duration::from_millis(5)
        };
        let requested_delay = if self.closing.is_some() {
            Duration::from_millis(50)
        } else {
            delay
        };
        self.cadence
            .repaint(self.ns(), requested_delay.as_nanos() as u64);
        ctx.request_repaint_after(requested_delay);
    }
}
impl eframe::App for Combined {
    fn logic(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let before = self.cadence.late_count;
        let now = self.ns();
        self.cadence.begin(now);
        self.logic_inner(ctx, frame);
        let sample_at = self.ns();
        if self.cadence.late_count > before
            && self.started.is_some()
            && self.closing.is_none()
            && self.late_supply.len() < 8
            && self.cadence.rows.last().is_some_and(|r| r[7] == now)
            && let (Some(output), Some(stream)) = (&self.output, &self.stream)
            && let Ok(alignment) = clock_alignment(output, stream)
        {
            let timing = scheduling_json(output, alignment, false);
            let sample_end = self.ns();
            self.late_supply.push(format!(
                "{{\"at_ns\":{now},\"sample_at_ns\":{sample_at},\"sample_end_ns\":{sample_end},\"timing\":{timing}}}"
            ));
        }
        self.cadence.end(self.ns());
    }
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        self.cadence.ui(self.ns(), false);
        if self.presentation.is_none() {
            let size = ui
                .ctx()
                .input(|i| i.viewport().inner_rect.unwrap().size() * i.pixels_per_point);
            self.presentation = Some(format!(
                "{{\"physical_size\":{:?},\"surface_config_utf8\":{:?}}}",
                [size.x, size.y],
                format!("{:?}", frame.wgpu_surface_config()).as_bytes()
            ));
        }
        if self.closing.is_none() {
            let started = Instant::now();
            let before = self.editor.host.traces.len();
            self.editor.ui(ui, frame);
            let elapsed_ms = started.elapsed().as_secs_f64() * 1000.;
            if let Some(start) = self.started
                && elapsed_ms > 8.
            {
                if self.slow_ui.len() == 256 {
                    self.failed = Some("slow UI trace overflow".into());
                } else {
                    self.slow_ui.push([
                        start.elapsed().as_secs_f64() * 1000.,
                        elapsed_ms,
                        before as f64,
                        self.editor.host.traces.len() as f64,
                    ]);
                }
            }
            if self.editor.host.trace_overflow {
                self.failed = Some("preparation trace overflow".into());
            }
            if self.started.is_some() {
                if self.frame_ms.len() >= 25000 {
                    self.failed = Some("frame trace overflow".into());
                } else if let Some(seconds) = frame.info().cpu_usage {
                    self.frame_ms.push(f64::from(seconds) * 1000.);
                }
            }
            self.max_targets = self.max_targets.max(self.editor.host.live_targets());
            self.max_rgba = self.max_rgba.max(self.editor.images.rgba_bytes());
        } else {
            ui.label("P2-A3: resources closed; sampling retained host.");
        }
        self.cadence.ui(self.ns(), true);
    }
    fn on_exit(&mut self) {
        if let Some(client) = &mut self.client {
            client.close();
        }
        if let Some(failure) = &self.failed {
            eprintln!("combined load failed: {failure}");
        }
        if !self.done {
            if let Some(output) = &self.output {
                self.final_stats = output.stats();
                self.underrun_observations = output.underrun_observations();
            }
            if let (Some(output), Some(stream)) = (&self.output, &self.stream)
                && let Ok(alignment) = clock_alignment(output, stream)
            {
                self.stream.take();
                self.timing = scheduling_json(self.output.as_ref().unwrap(), alignment, true);
            }
            if self.observer.thread.is_some()
                && let Ok(values) = self.observer.finish()
            {
                self.latencies = values.latencies;
                self.intervals = values.intervals;
                self.causal = values.causal;
                self.exact_intervals = values.exact;
                self.completion_markers = values.markers;
                self.requested_voices = values.requested;
                self.request_phases = values.request_phases;
                self.request_refs = values.request_refs;
                self.max_probe_poll_gap_ms = values.max_poll_gap_ms;
            }
            let _ = self.report("partial-report.json");
            let _ = std::fs::write(
                self.path.join("failure.txt"),
                self.failed
                    .as_deref()
                    .unwrap_or("window closed before completion"),
            );
        }
    }
}
pub fn run(destination: &str) -> Result<(), String> {
    run_mode(destination, false, false, false, false)
}
pub fn memory_run(destination: &str) -> Result<(), String> {
    run_mode(destination, false, true, false, false)
}
pub fn control_run(destination: &str) -> Result<(), String> {
    run_mode(destination, true, false, false, false)
}
pub fn memory_control_run(destination: &str) -> Result<(), String> {
    run_mode(destination, true, true, false, false)
}
pub fn cold_run(destination: &str, memory_usage: bool) -> Result<(), String> {
    run_mode(destination, true, memory_usage, true, false)
}
pub fn staging_run(destination: &str, cold_control: bool) -> Result<(), String> {
    run_mode(destination, cold_control, true, cold_control, true)
}
fn run_mode(
    destination: &str,
    control_only: bool,
    memory_usage: bool,
    cold_control: bool,
    reusable_staging: bool,
) -> Result<(), String> {
    let path = PathBuf::from(destination);
    if path.exists() {
        return Err("combined output must be new".into());
    }
    std::fs::create_dir_all(&path).map_err(error)?;
    phase("loading");
    let report = path.join("report.json");
    eframe::run_native(
        "Skin Editor P2-A3 - self-authored 300s load",
        eframe::NativeOptions {
            renderer: eframe::Renderer::Wgpu,
            wgpu_options: crate::gpu_observation::configuration(memory_usage),
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1180., 800.])
                .with_min_inner_size([820., 600.]),
            ..Default::default()
        },
        Box::new(move |context| {
            Combined::new(
                context,
                path,
                control_only,
                memory_usage,
                cold_control,
                reusable_staging,
            )
            .map(|app| Box::new(app) as Box<dyn eframe::App>)
            .map_err(|e| std::io::Error::other(e).into())
        }),
    )
    .map_err(error)?;
    if !report.is_file() {
        return Err("combined run incomplete; inspect failure.txt".into());
    }
    phase("done");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unknown_marker_cannot_advance_negative_or_late_telemetry_move_positive() {
        use skin_editor_infrastructure::VoiceSubmission::*;
        let mut b = SubmissionBracket::default();
        assert_eq!(b.completion(2., 3., Pending).unwrap(), None);
        assert_eq!(b.completion(25., 26., Unknown).unwrap(), None);
        assert_eq!(
            b.completion(30., 31., Complete([1; 7])).unwrap(),
            Some([2., 31.])
        );
        assert_eq!(b.completion(40., 41., Unknown).unwrap(), Some([2., 31.]));
        assert_eq!(
            b.completion(50., 51., Complete([1; 7])).unwrap(),
            Some([2., 31.])
        );
        assert!(b.completion(52., 53., Superseded).is_err());
        assert!(b.completion(52., 53., Invalidated).is_err());
    }
    #[test]
    fn invalid_probe_capacity_is_rejected_without_thread_or_device() {
        let budget = PcmBudget::default();
        let (p, c) = skin_editor_infrastructure::output_boundary(&budget, 48000, 1).unwrap();
        for capacity in [0, 257] {
            assert!(Observer::with_capacity(p.control(), capacity).is_err());
        }
        drop((p, c));
        assert_eq!(budget.usage().bytes, 0);
    }
    #[test]
    fn submission_bounds_cover_atomic_read_races_and_delayed_observer() {
        let mut bracket = SubmissionBracket::default();
        // A long stats read may straddle the actual submission. The last
        // negative's AFTER time (21 ms) would be an invalid lower bound.
        assert_eq!(bracket.observe(18., 21., false), None);
        assert_eq!(bracket.observe(22., 23., true), Some([18., 23.]));
        // A delayed causal row must not replace the first positive upper bound.
        assert_eq!(bracket.observe(25., 26., true), Some([18., 23.]));
        // A late first positive gives no lower bound beyond request time.
        assert_eq!(
            SubmissionBracket::default().observe(25., 26., true),
            Some([0., 26.])
        );
        let mut bracket = SubmissionBracket::default();
        assert_eq!(bracket.observe(21., 21.1, false), None);
        assert_eq!(bracket.observe(22., 22.1, true), Some([21., 22.1]));
    }
    #[test]
    fn native_probe_close_and_overflow_are_explicit_without_sleep_or_device() {
        use skin_editor_infrastructure::output_boundary;
        let budget = PcmBudget::default();
        let (producer, callback) = output_boundary(&budget, 48000, 1).unwrap();
        let output = producer.control();
        let mut observer = Observer::new(output.clone());
        output.close();
        observer
            .sender
            .as_ref()
            .unwrap()
            .send(Probe {
                callback_reference: None,
                completed: None,
                started: Instant::now(),
                receipt_ms: 0.,
                started_in_play_ms: 0.,
                voice: 1,
                lane: 0,
                clock: None,
            })
            .unwrap();
        assert!(observer.finish().is_err());
        let mut observer = Observer::new(output);
        for _ in 0..65 {
            observer
                .sender
                .as_ref()
                .unwrap()
                .send(Probe {
                    callback_reference: None,
                    completed: None,
                    started: Instant::now(),
                    receipt_ms: 0.,
                    started_in_play_ms: 0.,
                    voice: 0,
                    lane: 0,
                    clock: None,
                })
                .unwrap();
        }
        assert!(observer.finish().is_err());
        drop((observer, producer, callback));
        assert_eq!(budget.usage().bytes, 0);
    }
    #[test]
    fn standard_schedule_covers_300_seconds_without_unbounded_catch_up() {
        assert_eq!(TRIGGERS as u64 * 125, SECONDS * 1000);
        assert_eq!(EDITS as u64, SECONDS);
        assert!(1 + (SWITCHES as u64 - 1) * 10 < 200);
        assert_eq!(
            (0..SWITCHES)
                .map(|n| (n + 1) % 10)
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            10
        );
    }
}
