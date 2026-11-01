//! Detached bounded controls -> one supply owner. No ProjectSession crosses threads.
use crate::native_audio::OutputMark;
use crate::separated_output::{self, BackgroundOwner, StoppedOutput};
use crate::supply_output::SupplyOutputControl;
use crate::{
    BlobStore, BuildAdmission, OUTPUT_AHEAD_FRAMES, OUTPUT_QUEUE_FRAMES, OutputControl,
    OutputProducer, PreparationWorker, SkinAudioController, WavDecoder,
};
use crate::{background_input::BackgroundInput, background_stream::BufferedBackground};
use skin_editor_application::{
    AssetAudition, DraftId, ProjectSession, SkinAudioError, SkinAudioPort, SkinAudioSource,
    values::{AssetId, AudioComponentId, Gain, SkinProject},
};
use skin_editor_audio::{
    BackgroundTrack, Channel, MAX_BINDINGS, Origin, PcmBudget, SoundPlan, SoundTarget,
    TriggerReceipt,
};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicUsize, Ordering},
    mpsc::{self, Receiver, SyncSender},
};
use std::thread::{self, JoinHandle};
pub const MAX_SUPPLY_CONTROLS: usize = 16;
#[derive(Clone, Debug, PartialEq)]
pub enum SupplyError {
    Busy,
    Closed,
    Stale,
    Unsupported,
    ResourceLimit,
    InvalidPosition,
    Backend(String),
}
#[derive(Debug)]
struct Credit(Arc<AtomicUsize>);
impl Drop for Credit {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::AcqRel);
    }
}
#[derive(Debug)]
pub enum SupplyOutcome {
    Selected(Vec<BuildAdmission>),
    Triggered(TriggerReceipt),
    BatchTriggered {
        mix_frame: u64,
        results: Vec<Result<TriggerReceipt, skin_editor_audio::RuntimeError>>,
    },
    Frozen {
        next_background_frame: u64,
    },
    Located {
        frame: u64,
        previous_copy_anchor: u64,
    },
    Running,
    BackgroundSet,
    AuditionStopped,
    Observed,
}
#[derive(Debug)]
pub struct SupplyReceipt {
    pub id: u64,
    pub result: Result<SupplyOutcome, SupplyError>,
    _credit: Credit,
    retained: Vec<DraftId>,
    retained_revision: u64,
}
#[derive(Clone, Debug, Default)]
pub struct SupplyStatus {
    retained_revision: u64,
    pub origin: Option<Origin>,
    pub generated_background: u64,
    pub copied_background: u64,
    pub background_position: u64,
    pub background_frames: Option<u64>,
    pub background_primed: bool,
    pub frozen: bool,
    pub freezing: bool,
    pub ready_components: usize,
    pub retained_drafts: Vec<DraftId>,
    pub fault: Option<String>,
    pub bindings: Vec<(AudioComponentId, skin_editor_audio::BindingStatus)>,
    pub voices: [usize; 2],
    pub library: Option<(AssetId, skin_editor_audio::BindingStatus)>,
    pub last_preparation: Option<crate::PreparationObservation>,
    pub max_supply_gap_micros: u64,
}
struct Projection {
    origin: Origin,
    plans: Vec<SoundPlan>,
    cancelled: Vec<DraftId>,
    mark: OutputMark,
}
enum Control {
    Select(Projection),
    Background(BackgroundTrack),
    Scene {
        running: bool,
        mark: OutputMark,
    },
    Seek {
        frame: u64,
        mark: OutputMark,
    },
    StopAudition(OutputMark),
    Retry(AudioComponentId),
    Library {
        origin: Origin,
        plan: SoundPlan,
        mark: OutputMark,
    },
    Barrier,
    DiagnosticRecovery {
        trigger: Option<(Origin, Channel, AudioComponentId)>,
    },
    TriggerBatch {
        origin: Origin,
        entries: Vec<(Channel, AudioComponentId)>,
        stop_library: Option<OutputMark>,
    },
    Trigger {
        origin: Origin,
        lane: Channel,
        component: AudioComponentId,
        stop_library: Option<OutputMark>,
    },
}
struct Request {
    id: u64,
    control: Control,
    credit: Credit,
}
pub struct AudioSupplyClient {
    sender: SyncSender<Request>,
    receiver: Receiver<SupplyReceipt>,
    credits: Arc<AtomicUsize>,
    status: Arc<Mutex<SupplyStatus>>,
    output: SupplyOutputControl,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<Option<RetiredSupply>>>,
    retired: Option<RetiredSupply>,
    next: u64,
    pending: BTreeMap<u64, Option<DraftId>>,
    last: Option<SkinAudioSource>,
    audition_edits: skin_editor_audio::AuditionEdits,
    scene_goal: bool,
    library_active: bool,
    draft_audition: Option<DraftId>,
    reported_retained: Vec<DraftId>,
    reported_revision: u64,
    background_frames: Option<u64>,
}
struct RetiredSupply {
    core: SupplyCore,
    owner: Box<BackgroundOwner>,
}
enum SupplyOutput {
    Premixed(Box<OutputProducer>),
    Separated(Box<BackgroundOwner>),
}
impl SupplyOutput {
    fn pump(&mut self, core: &mut SupplyCore, ahead: usize) -> Result<bool, SupplyError> {
        let mut retained_changed = false;
        match self {
            Self::Premixed(output) => {
                output
                    .pump_timeline(
                        core.controller.runtime(),
                        core.background.as_mut(),
                        core.mark,
                        ahead,
                    )
                    .map_err(backend)?;
            }
            Self::Separated(owner) => {
                owner.pump(core.controller.runtime()).map_err(backend)?;
                if owner.position().phase != separated_output::Phase::Freezing {
                    owner.refill().map_err(backend)?;
                }
                core.separated_generated = owner.position().generated_frame;
                core.separated_position = owner.position().timeline_frame;
                core.separated_primed = owner.is_primed();
                core.separated_anchor_ready =
                    owner.position().phase == separated_output::Phase::Frozen;
                retained_changed = !owner
                    .retained_drafts()
                    .filter_map(DraftId::new)
                    .eq(core.external_drafts.iter().copied());
                core.external_drafts.clear();
                core.external_drafts
                    .extend(owner.retained_drafts().filter_map(DraftId::new));
            }
        }
        Ok(retained_changed)
    }
}
fn backend(error: impl std::fmt::Debug) -> SupplyError {
    SupplyError::Backend(format!("{error:?}"))
}
fn origin(source: SkinAudioSource) -> Origin {
    Origin {
        session: source.session.get(),
        node: source.state.head.get(),
        revision: source.state.revision,
        navigation_revision: source.navigation_revision,
        draft: source.draft.map(|d| (d.id.get(), d.revision)),
    }
}
impl AudioSupplyClient {
    /// Explicit capture-only probe, not an OS stall or a project command.
    /// A trigger shares this owner turn; None observes the refill before a later trigger.
    pub fn diagnostic_recovery_probe(
        &mut self,
        trigger: Option<(Channel, AudioComponentId)>,
    ) -> Result<u64, SupplyError> {
        if self.output.capture_origin().is_none() || self.library_active {
            return Err(SupplyError::Unsupported);
        }
        let trigger = trigger
            .map(|(lane, component)| {
                Ok((
                    origin(self.last.ok_or(SupplyError::Stale)?),
                    lane,
                    component,
                ))
            })
            .transpose()?;
        let credit = self.reserve()?;
        self.send(Control::DiagnosticRecovery { trigger }, credit, None)
    }
    /// Bounded diagnostic barrier, acknowledged after the owner pumps; no playback action.
    pub fn barrier(&mut self) -> Result<u64, SupplyError> {
        let credit = self.reserve()?;
        self.send(Control::Barrier, credit, None)
    }
    pub fn spawn(
        mut output: OutputProducer,
        rate: u32,
        channels: u16,
        budget: PcmBudget,
        store: BlobStore,
    ) -> Result<Self, SupplyError> {
        let mut core = SupplyCore::new(&output, rate, channels, budget)?;
        output
            .pump_timeline(
                core.controller.runtime(),
                None,
                core.mark,
                OUTPUT_QUEUE_FRAMES,
            )
            .map_err(backend)?;
        Self::spawn_owned(SupplyOutput::Premixed(Box::new(output)), core, store)
    }
    /// Explicit candidate; background and consumer share the same PCM budget.
    /// The default spawn and GUI continue to select the premixed transport.
    pub fn spawn_separated(
        background: BackgroundTrack,
        rate: u32,
        channels: u16,
        budget: PcmBudget,
        store: BlobStore,
    ) -> Result<(Self, Box<separated_output::OutputConsumer>), SupplyError> {
        let (core, owner, consumer) =
            SupplyCore::new_separated(background, rate, channels, budget)?;
        let client = Self::spawn_owned(SupplyOutput::Separated(Box::new(owner)), core, store)?;
        Ok((client, consumer))
    }
    pub fn spawn_streamed(
        background: BufferedBackground,
        rate: u32,
        channels: u16,
        budget: PcmBudget,
        store: BlobStore,
    ) -> Result<(Self, Box<separated_output::OutputConsumer>), SupplyError> {
        let (core, owner, consumer) =
            SupplyCore::new_separated(background, rate, channels, budget)?;
        let client = Self::spawn_owned(SupplyOutput::Separated(Box::new(owner)), core, store)?;
        Ok((client, consumer))
    }
    fn spawn_owned(
        mut output: SupplyOutput,
        mut core: SupplyCore,
        store: BlobStore,
    ) -> Result<Self, SupplyError> {
        let output_control = core.output.clone();
        let background_frames = core.background_frames;
        let (sender, requests) = mpsc::sync_channel(MAX_SUPPLY_CONTROLS);
        let (receipts, receiver) = mpsc::sync_channel(MAX_SUPPLY_CONTROLS);
        let status = Arc::new(Mutex::new(core.status()));
        let shared = status.clone();
        let stop = Arc::new(AtomicBool::new(false));
        let stopping = stop.clone();
        let safety = output_control.clone();
        let thread = thread::Builder::new()
            .name("skin-audio-supply".into())
            .spawn(move || {
                let result = (|| -> Result<(), SupplyError> {
                    let mut outbound = Vec::with_capacity(MAX_SUPPLY_CONTROLS);
                    let mut dirty = true;
                    let mut retained_revision = 0u64;
                    let mut previous = std::time::Instant::now();
                    while !stopping.load(Ordering::Acquire) {
                        let now = std::time::Instant::now();
                        core.max_supply_gap_micros = core.max_supply_gap_micros.max(
                            now.duration_since(previous)
                                .as_micros()
                                .min(u128::from(u64::MAX)) as u64,
                        );
                        previous = now;
                        if let SupplyOutput::Premixed(output) = &mut output {
                            output.owner_woke(now);
                        }
                        let mut changed = false;
                        for _ in 0..MAX_SUPPLY_CONTROLS {
                            let Ok(request) = requests.try_recv() else {
                                break;
                            };
                            changed = true;
                            if let Some(receipt) = core.apply_output(&mut output, request, &store) {
                                outbound.push(receipt);
                            }
                        }
                        if matches!(output, SupplyOutput::Premixed(_))
                            && let Some(receipt) = core.finish_freeze()?
                        {
                            outbound.push(receipt);
                            changed = true;
                        }
                        changed |= core.prepare()?;
                        changed |= output.pump(&mut core, OUTPUT_AHEAD_FRAMES)?;
                        if matches!(output, SupplyOutput::Separated(_))
                            && let Some(receipt) = core.finish_freeze()?
                        {
                            outbound.push(receipt);
                            changed = true;
                        }
                        core.finish_acknowledgments(&mut outbound);
                        let voices = core.voice_counts();
                        if shared.try_lock().is_ok_and(|s| s.voices != voices) {
                            changed = true;
                        }
                        if changed {
                            retained_revision = retained_revision
                                .checked_add(1)
                                .ok_or(SupplyError::ResourceLimit)?;
                        }
                        dirty |= changed;
                        let snapshot = dirty.then(|| core.status());
                        // A GUI query cannot stall the supply owner.
                        if let Ok(mut published) = shared.try_lock() {
                            if let Some(snapshot) = snapshot {
                                *published = snapshot;
                                published.retained_revision = retained_revision;
                                dirty = false;
                            } else {
                                core.publish_progress(&mut published);
                            }
                        }
                        for mut receipt in outbound.drain(..) {
                            receipt.retained = core.retained_drafts();
                            receipt.retained_revision = retained_revision;
                            receipts
                                .try_send(receipt)
                                .map_err(|_| SupplyError::Closed)?;
                        }
                        if let SupplyOutput::Premixed(output) = &mut output {
                            output.owner_sleeping();
                        }
                        if !dirty
                            && core.can_idle(matches!(output, SupplyOutput::Separated(_)), voices)
                        {
                            // Control enqueue and close both unpark this owner. A
                            // bounded housekeeping heartbeat remains while idle.
                            thread::park_timeout(std::time::Duration::from_millis(50));
                        } else {
                            thread::sleep(std::time::Duration::from_millis(2));
                        }
                    }
                    Ok(())
                })();
                safety.close();
                core.controller.close();
                core.worker.close();
                core.freezing = None;
                core.acknowledgments.clear();
                if let Ok(mut published) = shared.lock()
                    && !stopping.load(Ordering::Acquire)
                    && let Err(error) = result
                {
                    published.fault = Some(format!("{error:?}"));
                }
                match output {
                    SupplyOutput::Premixed(_) => None,
                    SupplyOutput::Separated(mut owner) => {
                        owner.close();
                        Some(RetiredSupply { core, owner })
                    }
                }
            })
            .map_err(backend)?;
        Ok(Self {
            sender,
            receiver,
            credits: Arc::new(AtomicUsize::new(0)),
            status,
            output: output_control,
            stop,
            thread: Some(thread),
            retired: None,
            next: 1,
            pending: BTreeMap::new(),
            last: None,
            audition_edits: skin_editor_audio::AuditionEdits::default(),
            scene_goal: false,
            library_active: false,
            draft_audition: None,
            reported_retained: Vec::new(),
            reported_revision: 0,
            background_frames,
        })
    }
    fn reserve(&self) -> Result<Credit, SupplyError> {
        if self.stop.load(Ordering::Acquire) || self.output.is_closed() {
            return Err(SupplyError::Closed);
        }
        self.credits
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |n| {
                (n < MAX_SUPPLY_CONTROLS).then_some(n + 1)
            })
            .map_err(|_| SupplyError::Busy)?;
        Ok(Credit(self.credits.clone()))
    }
    fn send(
        &mut self,
        control: Control,
        credit: Credit,
        draft: Option<DraftId>,
    ) -> Result<u64, SupplyError> {
        let id = self.next;
        self.next = self.next.checked_add(1).ok_or(SupplyError::Closed)?;
        self.sender
            .try_send(Request {
                id,
                control,
                credit,
            })
            .map_err(|_| {
                self.output.close();
                SupplyError::Closed
            })?;
        self.pending.insert(id, draft);
        if let Some(t) = &self.thread {
            t.thread().unpark();
        }
        Ok(id)
    }
    pub fn synchronize(&mut self, session: &ProjectSession) -> Result<(), SkinAudioError> {
        session.synchronize_audio(self)
    }
    pub fn set_background(&mut self, background: BackgroundTrack) -> Result<u64, SupplyError> {
        let credit = self.reserve()?;
        self.send(Control::Background(background), credit, None)
    }
    pub fn scene_running(&mut self, running: bool) -> Result<u64, SupplyError> {
        let credit = self.reserve()?;
        let mark = if running != self.scene_goal {
            self.output.invalidate_scene().map_err(backend)?
        } else {
            self.output.mark()
        };
        let id = self.send(Control::Scene { running, mark }, credit, None)?;
        self.scene_goal = running;
        Ok(id)
    }
    /// Location is preview control, never a project command. A successful reply
    /// waits for native freeze and owner rebase, and leaves the scene frozen.
    pub fn seek_background(&mut self, frame: u64) -> Result<u64, SupplyError> {
        let limit = self.background_frames.ok_or(SupplyError::Unsupported)?;
        if frame > limit {
            return Err(SupplyError::InvalidPosition);
        }
        let credit = self.reserve()?;
        let mark = self.output.invalidate_scene().map_err(backend)?;
        let id = self.send(Control::Seek { frame, mark }, credit, None)?;
        self.scene_goal = false;
        Ok(id)
    }
    pub fn audition_asset(&mut self, request: AssetAudition) -> Result<u64, SupplyError> {
        if self.last != Some(request.source) {
            return Err(SupplyError::Stale);
        }
        let credit = self.reserve()?;
        let mark = self.output.invalidate_skin(false, true).map_err(backend)?;
        let plan = SoundPlan {
            target: SoundTarget::Library(request.asset),
            source: request.blob,
            metadata: request.metadata,
            gain: Gain::UNITY,
            visible: true,
        };
        let id = self.send(
            Control::Library {
                origin: origin(request.source),
                plan,
                mark,
            },
            credit,
            request.source.draft.map(|d| d.id),
        )?;
        self.library_active = true;
        Ok(id)
    }
    pub fn stop_audition(&mut self) -> Result<u64, SupplyError> {
        let credit = self.reserve()?;
        let mark = self.output.invalidate_skin(false, true).map_err(backend)?;
        self.library_active = false;
        self.send(Control::StopAudition(mark), credit, None)
    }
    pub fn retry_selected(&mut self, component: AudioComponentId) -> Result<u64, SupplyError> {
        let credit = self.reserve()?;
        self.send(Control::Retry(component), credit, None)
    }
    pub fn trigger(
        &mut self,
        session: &ProjectSession,
        lane: Channel,
        component: AudioComponentId,
    ) -> Result<u64, SupplyError> {
        self.synchronize(session).map_err(backend)?;
        self.trigger_selected(lane, component)
    }
    /// Only after the application control boundary selected the actual source.
    pub fn trigger_selected(
        &mut self,
        lane: Channel,
        component: AudioComponentId,
    ) -> Result<u64, SupplyError> {
        let credit = self.reserve()?;
        let origin = origin(self.last.ok_or(SupplyError::Stale)?);
        let stop_library = if lane == Channel::Audition && self.library_active {
            self.library_active = false;
            Some(self.output.invalidate_skin(false, true).map_err(backend)?)
        } else {
            None
        };
        let result = self.send(
            Control::Trigger {
                origin,
                lane,
                component,
                stop_library,
            },
            credit,
            None,
        );
        if result.is_ok() && lane == Channel::Audition {
            self.draft_audition = origin.draft.and_then(|d| DraftId::new(d.0));
        }
        result
    }
    /// One bounded control; all accepted voices start at the same mix frame.
    /// Each entry reports refusal separately; this is not a project transaction.
    pub fn trigger_batch_selected(
        &mut self,
        entries: &[(Channel, AudioComponentId)],
    ) -> Result<u64, SupplyError> {
        if entries.is_empty() || entries.len() > skin_editor_audio::MAX_VOICES {
            return Err(SupplyError::ResourceLimit);
        }
        let origin = origin(self.last.ok_or(SupplyError::Stale)?);
        let credit = self.reserve()?;
        let stop_library =
            if self.library_active && entries.iter().any(|(lane, _)| *lane == Channel::Audition) {
                self.library_active = false;
                Some(self.output.invalidate_skin(false, true).map_err(backend)?)
            } else {
                None
            };
        let result = self.send(
            Control::TriggerBatch {
                origin,
                entries: entries.to_vec(),
                stop_library,
            },
            credit,
            None,
        );
        if result.is_ok() && entries.iter().any(|(lane, _)| *lane == Channel::Audition) {
            self.draft_audition = origin.draft.and_then(|d| DraftId::new(d.0));
        }
        result
    }
    pub fn poll(&mut self) -> Option<SupplyReceipt> {
        let receipt = self.receiver.try_recv().ok()?;
        Some(self.accept_receipt(receipt))
    }
    /// Diagnostics/headless callers only; UI callers should use `poll`.
    pub fn wait_receipt(
        &mut self,
        timeout: std::time::Duration,
    ) -> Result<Option<SupplyReceipt>, SupplyError> {
        match self.receiver.recv_timeout(timeout) {
            Ok(receipt) => Ok(Some(self.accept_receipt(receipt))),
            Err(mpsc::RecvTimeoutError::Timeout) => Ok(None),
            Err(mpsc::RecvTimeoutError::Disconnected) => Err(SupplyError::Closed),
        }
    }
    fn accept_receipt(&mut self, receipt: SupplyReceipt) -> SupplyReceipt {
        self.pending.remove(&receipt.id);
        self.reported_retained.clone_from(&receipt.retained);
        self.reported_revision = receipt.retained_revision;
        receipt
    }
    pub fn status(&self) -> Result<SupplyStatus, SupplyError> {
        self.status.lock().map(|s| s.clone()).map_err(backend)
    }
    pub fn pending_controls(&self) -> bool {
        !self.pending.is_empty()
    }
    pub fn output(&self) -> Result<OutputControl, SupplyError> {
        self.output.premixed().map_err(backend)
    }
    pub fn close(&mut self) {
        self.output.close();
        self.stop.store(true, Ordering::Release);
        if let Some(thread) = self.thread.take() {
            thread.thread().unpark();
            self.retired = thread.join().ok().flatten();
        }
        while self.poll().is_some() {}
        self.pending.clear();
    }
    /// After close/join and native stream destruction, return the exact endpoint.
    pub fn finish_separated(&mut self, stopped: StoppedOutput) -> Result<(), StoppedOutput> {
        let Some(retired) = self.retired.as_mut() else {
            return Err(stopped);
        };
        if !retired.core.worker.closed() {
            return Err(stopped);
        }
        retired
            .owner
            .finish(retired.core.controller.runtime(), stopped)?;
        self.retired = None;
        Ok(())
    }
}
impl Drop for AudioSupplyClient {
    fn drop(&mut self) {
        self.close();
    }
}
impl SkinAudioPort for AudioSupplyClient {
    fn retained_drafts(&self) -> Vec<DraftId> {
        let mut ids: BTreeSet<_> = self.pending.values().flatten().copied().collect();
        if let Some(id) = self.last.and_then(|s| s.draft.map(|d| d.id)) {
            ids.insert(id);
        }
        // A newer published retirement replaces the receipt snapshot. A
        // contended/failed publication must still conservatively retain it.
        match self.status.lock() {
            Ok(status) => {
                ids.extend(status.retained_drafts.iter().copied());
                if status.retained_revision < self.reported_revision {
                    ids.extend(self.reported_retained.iter().copied());
                }
            }
            Err(_) => ids.extend(self.reported_retained.iter().copied()),
        }
        ids.into_iter().collect()
    }
    fn synchronize(
        &mut self,
        project: &SkinProject,
        source: SkinAudioSource,
        cancelled: &[DraftId],
    ) -> Result<(), SkinAudioError> {
        if self.output.is_closed() {
            return Err(SkinAudioError::Backend("output closed".into()));
        }
        if self.last == Some(source) && cancelled.is_empty() {
            return Ok(());
        }
        if project.audio_components().len() > MAX_BINDINGS {
            let _ = self.output.invalidate_skin(true, true);
            return Err(SkinAudioError::ResourceLimit);
        }
        let plans = project
            .audio_components()
            .map(|c| SoundPlan::build(project, c.id()))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| SkinAudioError::Backend(format!("{e:?}")))?;
        let mut audition_edits = self.audition_edits.clone();
        let change = audition_edits.select(
            origin(source),
            &plans,
            &cancelled.iter().map(|d| d.get()).collect::<Vec<_>>(),
        );
        let cancelled_audition = self
            .draft_audition
            .is_some_and(|id| cancelled.contains(&id));
        let reset = (!cancelled.is_empty() && (!change.preserve_cancel || cancelled_audition))
            || self.last.is_some_and(|old| {
                old.session != source.session
                    || old.navigation_revision != source.navigation_revision
            });
        let raw_reset = self.library_active && self.last != Some(source);
        let mark = if reset {
            self.output
                .invalidate_skin(true, true)
                .map_err(SkinAudioError::Backend)?
        } else if raw_reset || change.replacement {
            self.output
                .invalidate_skin(false, true)
                .map_err(SkinAudioError::Backend)?
        } else {
            self.output.mark()
        };
        if raw_reset {
            self.library_active = false;
        }
        let credit = self.reserve().map_err(|_| {
            let _ = self.output.invalidate_skin(true, true);
            SkinAudioError::ResourceLimit
        })?;
        self.send(
            Control::Select(Projection {
                origin: origin(source),
                plans,
                cancelled: cancelled.to_vec(),
                mark,
            }),
            credit,
            source.draft.map(|d| d.id),
        )
        .map_err(|e| SkinAudioError::Backend(format!("{e:?}")))?;
        self.last = Some(source);
        self.audition_edits = audition_edits;
        if reset || change.replacement || source.draft.is_none() {
            self.draft_audition = None;
        }
        Ok(())
    }
}
struct SupplyCore {
    controller: SkinAudioController,
    worker: PreparationWorker,
    background: Option<BackgroundTrack>,
    output: SupplyOutputControl,
    mark: OutputMark,
    selected: Option<Origin>,
    frozen: bool,
    freezing: Option<Request>,
    components: Vec<AudioComponentId>,
    library_intent: Option<AssetId>,
    last_preparation: Option<crate::PreparationObservation>,
    max_supply_gap_micros: u64,
    separated_generated: u64,
    separated_position: u64,
    separated_primed: bool,
    background_frames: Option<u64>,
    separated_anchor_ready: bool,
    acknowledgments: Vec<(OutputMark, SupplyReceipt)>,
    external_drafts: Vec<DraftId>,
}
impl SupplyCore {
    fn can_idle(&mut self, separated: bool, voices: [usize; 2]) -> bool {
        if !separated
            || !self.frozen
            || self.freezing.is_some()
            || !self.separated_anchor_ready
            || !self.separated_primed
            || !self.acknowledgments.is_empty()
            || self.library_intent.is_some()
            || !self.external_drafts.is_empty()
            || voices != [0, 0]
        {
            return false;
        }
        let runtime = self.controller.runtime();
        !self.components.iter().any(|id| {
            runtime
                .status(*id)
                .is_some_and(|s| s.preparation == skin_editor_audio::Preparation::Pending)
        }) && !runtime
            .library_status()
            .is_some_and(|(_, s)| s.preparation == skin_editor_audio::Preparation::Pending)
    }
    fn new_separated(
        background: impl Into<BackgroundInput>,
        rate: u32,
        channels: u16,
        budget: PcmBudget,
    ) -> Result<(Self, BackgroundOwner, Box<separated_output::OutputConsumer>), SupplyError> {
        let background = background.into();
        let background_frames = background.frames();
        let mut controller =
            SkinAudioController::with_budget(rate, channels, budget).map_err(backend)?;
        controller.runtime().enable_minimum_conversion();
        controller
            .runtime()
            .enable_voice_handoff()
            .map_err(backend)?;
        let source = Origin {
            session: 1,
            node: 1,
            revision: 0,
            navigation_revision: 0,
            draft: None,
        };
        let (owner, consumer) =
            separated_output::boundary_input(controller.runtime(), background, source)
                .map_err(backend)?;
        let output = SupplyOutputControl::Separated(owner.control());
        let mark = output.mark();
        let core = Self {
            controller,
            worker: PreparationWorker::new(WavDecoder).map_err(backend)?,
            background: None,
            output,
            mark,
            selected: None,
            frozen: true,
            freezing: None,
            components: Vec::new(),
            library_intent: None,
            last_preparation: None,
            max_supply_gap_micros: 0,
            separated_generated: 0,
            separated_position: 0,
            separated_primed: false,
            background_frames: Some(background_frames),
            separated_anchor_ready: true,
            acknowledgments: Vec::with_capacity(MAX_SUPPLY_CONTROLS),
            external_drafts: Vec::new(),
        };
        Ok((core, owner, consumer))
    }
    fn generated_background(&self) -> u64 {
        if self.output.is_separated() {
            self.separated_generated
        } else {
            self.background
                .as_ref()
                .map_or(0, BackgroundTrack::position)
        }
    }
    fn publish_progress(&self, published: &mut SupplyStatus) {
        published.max_supply_gap_micros = self.max_supply_gap_micros;
        published.generated_background = self.generated_background();
        published.copied_background = self.output.background_next();
        published.background_position = self.separated_position;
        published.background_primed = self.separated_primed;
    }
    fn retained_drafts(&mut self) -> Vec<DraftId> {
        self.controller
            .runtime()
            .retained_drafts()
            .into_iter()
            .filter_map(DraftId::new)
            .chain(self.external_drafts.iter().copied())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }
    fn apply_output(
        &mut self,
        output: &mut SupplyOutput,
        request: Request,
        store: &BlobStore,
    ) -> Option<SupplyReceipt> {
        let SupplyOutput::Separated(owner) = output else {
            let SupplyOutput::Premixed(output) = output else {
                unreachable!()
            };
            return self.apply_owned(output, request, store);
        };
        let wait_for_ack = matches!(request.control, Control::Barrier | Control::StopAudition(_));
        if let Control::Select(p) = &request.control
            && p.cancelled.is_empty()
        {
            // A client-side reset (including backpressure/cancel recovery) must
            // revoke old instances before a newer fence can label any samples.
            // Ordinary hot activation has unchanged fences. Selective draft
            // cancellation belongs to Runtime; unrelated voices must survive.
            let runtime = self.controller.runtime();
            if p.mark.skin.0[0] != self.mark.skin.0[0] {
                let running = runtime.scene_running();
                if let Err(e) = runtime.set_scene_running(runtime.token(Channel::Scene), running) {
                    return Some(SupplyReceipt {
                        id: request.id,
                        result: Err(backend(e)),
                        _credit: request.credit,
                        retained: Vec::new(),
                        retained_revision: 0,
                    });
                }
            }
            if p.mark.skin.0[1] != self.mark.skin.0[1] {
                self.library_intent = None;
                if let Err(e) = runtime.stop_audition(runtime.token(Channel::Audition)) {
                    return Some(SupplyReceipt {
                        id: request.id,
                        result: Err(backend(e)),
                        _credit: request.credit,
                        retained: Vec::new(),
                        retained_revision: 0,
                    });
                }
            }
        }
        let receipt = if let Control::Seek { frame, mark } = &request.control {
            let (frame, mark) = (*frame, *mark);
            let result = if self.freezing.is_some() {
                Err(SupplyError::Busy)
            } else {
                owner
                    .seek(self.controller.runtime(), frame)
                    .map_err(backend)
            };
            // The version only invalidates old copied data, even if the requested
            // location is Busy. It must not leave the original freeze unfinishable.
            self.mark.background = mark.background;
            self.mark.skin = self.mark.skin.with_scene(mark.skin);
            owner.apply_mark(self.mark);
            if result.is_ok() {
                self.frozen = true;
                self.separated_anchor_ready = false;
                self.freezing = Some(request);
                return None;
            }
            Some(SupplyReceipt {
                id: request.id,
                result: result.map(|_| SupplyOutcome::Running),
                _credit: request.credit,
                retained: Vec::new(),
                retained_revision: 0,
            })
        } else if let Control::Scene { running, mark } = &request.control {
            let (running, mark) = (*running, *mark);
            let result = if self.freezing.is_some() {
                Err(SupplyError::Busy)
            } else if running != self.frozen {
                Ok(if running {
                    SupplyOutcome::Running
                } else {
                    SupplyOutcome::Frozen {
                        next_background_frame: self.output.background_next(),
                    }
                })
            } else if running {
                owner
                    .refill()
                    .map_err(backend)
                    .and_then(|_| {
                        owner.resume(self.controller.runtime()).map_err(|e| {
                            if matches!(
                                e,
                                separated_output::OutputError::NotPrimed
                                    | separated_output::OutputError::AwaitingAck
                            ) {
                                SupplyError::Busy
                            } else {
                                backend(e)
                            }
                        })
                    })
                    .map(|_| SupplyOutcome::Running)
            } else {
                owner
                    .pause(self.controller.runtime())
                    .map_err(backend)
                    .map(|_| SupplyOutcome::Running)
            };
            self.mark.background = mark.background;
            self.mark.skin = self.mark.skin.with_scene(mark.skin);
            if result.is_ok() {
                self.frozen = !running;
                self.separated_anchor_ready =
                    owner.position().phase == separated_output::Phase::Frozen;
                if !running && owner.position().phase == separated_output::Phase::Freezing {
                    self.freezing = Some(request);
                    owner.apply_mark(self.mark);
                    return None;
                }
            }
            Some(SupplyReceipt {
                id: request.id,
                result,
                _credit: request.credit,
                retained: Vec::new(),
                retained_revision: 0,
            })
        } else if matches!(request.control, Control::Background(_)) {
            Some(SupplyReceipt {
                id: request.id,
                result: Err(SupplyError::Unsupported),
                _credit: request.credit,
                retained: Vec::new(),
                retained_revision: 0,
            })
        } else {
            self.apply(request, store)
        };
        owner.apply_mark(self.mark);
        if wait_for_ack && receipt.as_ref().is_some_and(|r| r.result.is_ok()) {
            self.acknowledgments.push((self.mark, receipt.unwrap()));
            None
        } else {
            receipt
        }
    }
    fn finish_acknowledgments(&mut self, outbound: &mut Vec<SupplyReceipt>) {
        let mut i = 0;
        while i < self.acknowledgments.len() {
            if self.output.acknowledged(self.acknowledgments[i].0) {
                outbound.push(self.acknowledgments.remove(i).1);
            } else {
                i += 1;
            }
        }
    }
    fn apply_owned(
        &mut self,
        output: &mut OutputProducer,
        mut request: Request,
        store: &BlobStore,
    ) -> Option<SupplyReceipt> {
        let Control::DiagnosticRecovery { trigger } = &request.control else {
            return self.apply(request, store);
        };
        let trigger = *trigger;
        let invalid = if trigger.is_some_and(|(origin, _, _)| self.selected != Some(origin)) {
            Some(SupplyError::Stale)
        } else if trigger.is_none()
            && self.controller.runtime().output_activity() == [false, false]
            && self.background.as_ref().is_none_or(|b| !b.is_running())
        {
            Some(SupplyError::Unsupported)
        } else {
            output.arm_diagnostic_recovery().err().map(backend)
        };
        if let Some(error) = invalid {
            return Some(SupplyReceipt {
                id: request.id,
                result: Err(error),
                _credit: request.credit,
                retained: Vec::new(),
                retained_revision: 0,
            });
        }
        request.control = trigger.map_or(Control::Barrier, |(origin, lane, component)| {
            Control::Trigger {
                origin,
                lane,
                component,
                stop_library: None,
            }
        });
        let receipt = self.apply(request, store);
        if receipt.as_ref().is_none_or(|r| r.result.is_err()) {
            output.cancel_diagnostic_recovery();
        }
        receipt
    }
    fn new(
        output: &OutputProducer,
        rate: u32,
        channels: u16,
        budget: PcmBudget,
    ) -> Result<Self, SupplyError> {
        let mut controller =
            SkinAudioController::with_budget(rate, channels, budget).map_err(backend)?;
        controller.runtime().enable_minimum_conversion();
        Ok(Self {
            controller,
            worker: PreparationWorker::new(WavDecoder).map_err(backend)?,
            background: None,
            output: SupplyOutputControl::Premixed(output.control()),
            mark: output.control().mark(),
            selected: None,
            frozen: true,
            freezing: None,
            components: Vec::new(),
            library_intent: None,
            last_preparation: None,
            max_supply_gap_micros: 0,
            separated_generated: 0,
            separated_position: 0,
            separated_primed: false,
            background_frames: None,
            separated_anchor_ready: false,
            acknowledgments: Vec::with_capacity(MAX_SUPPLY_CONTROLS),
            external_drafts: Vec::new(),
        })
    }
    fn apply(&mut self, request: Request, store: &BlobStore) -> Option<SupplyReceipt> {
        let Request {
            id,
            control,
            credit,
        } = request;
        let result = (|| -> Result<SupplyOutcome, SupplyError> {
            match control {
                Control::DiagnosticRecovery { .. } | Control::Seek { .. } => {
                    Err(SupplyError::Unsupported)
                }
                Control::Barrier => Ok(SupplyOutcome::Observed),
                Control::TriggerBatch {
                    origin,
                    entries,
                    stop_library,
                } => {
                    if self.selected != Some(origin) {
                        return Err(SupplyError::Stale);
                    }
                    if entries.is_empty() || entries.len() > skin_editor_audio::MAX_VOICES {
                        return Err(SupplyError::ResourceLimit);
                    }
                    if let Some(mark) = stop_library {
                        self.library_intent = None;
                        self.controller.cancel_library();
                        let r = self.controller.runtime();
                        r.stop_audition(r.token(Channel::Audition))
                            .map_err(backend)?;
                        self.mark.skin = self.mark.skin.with_audition(mark.skin);
                    }
                    let r = self.controller.runtime();
                    let mix_frame = r.mix_frame();
                    let results = entries
                        .into_iter()
                        .map(|(lane, id)| r.trigger(r.token(lane), id, mix_frame))
                        .collect();
                    Ok(SupplyOutcome::BatchTriggered { mix_frame, results })
                }
                Control::Select(p) => {
                    if self.selected != Some(p.origin) {
                        self.library_intent = None;
                    }
                    self.controller
                        .select(p.origin, &p.plans, &p.cancelled)
                        .map_err(backend)?;
                    self.mark.skin = p.mark.skin;
                    self.selected = Some(p.origin);
                    self.components = p
                        .plans
                        .iter()
                        .filter_map(|p| match p.target {
                            SoundTarget::Component(id) => Some(id),
                            _ => None,
                        })
                        .collect();
                    Ok(SupplyOutcome::Selected(
                        self.controller.admit_selected(store).map_err(backend)?,
                    ))
                }
                Control::Background(track) => {
                    if self.background.is_some()
                        || !self.frozen
                        || self.freezing.is_some()
                        || track.format() != self.controller.runtime().output_format()
                        || !track
                            .pcm()
                            .budget()
                            .same_pool(&self.controller.runtime().pcm_budget())
                    {
                        return Err(SupplyError::Unsupported);
                    }
                    self.background = Some(track);
                    Ok(SupplyOutcome::BackgroundSet)
                }
                Control::Scene { running, mark } => {
                    if self.freezing.is_some() {
                        return Err(SupplyError::Busy);
                    }
                    if running != self.frozen {
                        return Ok(if running {
                            SupplyOutcome::Running
                        } else {
                            SupplyOutcome::Frozen {
                                next_background_frame: self.output.background_next(),
                            }
                        });
                    }
                    let runtime = self.controller.runtime();
                    runtime
                        .set_scene_running(runtime.token(Channel::Scene), running)
                        .map_err(backend)?;
                    self.mark.background = mark.background;
                    self.mark.skin = self.mark.skin.with_scene(mark.skin);
                    if let Some(track) = &mut self.background {
                        track.set_running(running);
                    }
                    self.frozen = !running;
                    Ok(SupplyOutcome::Running)
                }
                Control::StopAudition(mark) => {
                    self.library_intent = None;
                    self.controller.cancel_library();
                    let runtime = self.controller.runtime();
                    runtime
                        .stop_audition(runtime.token(Channel::Audition))
                        .map_err(backend)?;
                    self.mark.skin = self.mark.skin.with_audition(mark.skin);
                    Ok(SupplyOutcome::AuditionStopped)
                }
                Control::Retry(component) => {
                    self.controller.retry_selected(component).map_err(backend)?;
                    Ok(SupplyOutcome::Selected(
                        self.controller.admit_selected(store).map_err(backend)?,
                    ))
                }
                Control::Library { origin, plan, mark } => {
                    if self.selected != Some(origin) {
                        return Err(SupplyError::Stale);
                    }
                    let SoundTarget::Library(asset) = plan.target else {
                        return Err(SupplyError::Unsupported);
                    };
                    self.library_intent = None;
                    self.mark.skin = self.mark.skin.with_audition(mark.skin);
                    self.controller
                        .request_library(plan, store)
                        .map_err(backend)?;
                    self.library_intent = Some(asset);
                    self.activate_library()?;
                    Ok(SupplyOutcome::Observed)
                }
                Control::Trigger {
                    origin,
                    lane,
                    component,
                    stop_library,
                } => {
                    if self.selected != Some(origin) {
                        return Err(SupplyError::Stale);
                    }
                    if let Some(mark) = stop_library {
                        self.library_intent = None;
                        self.controller.cancel_library();
                        let runtime = self.controller.runtime();
                        runtime
                            .stop_audition(runtime.token(Channel::Audition))
                            .map_err(backend)?;
                        self.mark.skin = self.mark.skin.with_audition(mark.skin);
                    }
                    let runtime = self.controller.runtime();
                    Ok(SupplyOutcome::Triggered(
                        runtime
                            .trigger(runtime.token(lane), component, runtime.mix_frame())
                            .map_err(backend)?,
                    ))
                }
            }
        })();
        if matches!(&result, Ok(SupplyOutcome::Running)) && self.frozen {
            self.freezing = Some(Request {
                id,
                control: Control::Scene {
                    running: false,
                    mark: self.mark,
                },
                credit,
            });
            None
        } else {
            Some(SupplyReceipt {
                id,
                result,
                _credit: credit,
                retained: Vec::new(),
                retained_revision: 0,
            })
        }
    }
    fn finish_freeze(&mut self) -> Result<Option<SupplyReceipt>, SupplyError> {
        if self.freezing.is_none()
            || (self.output.is_separated() && !self.separated_anchor_ready)
            || !self.output.background_acknowledged(self.mark)
        {
            return Ok(None);
        }
        let next = self.output.background_next();
        if let Some(Request {
            control: Control::Seek { frame, .. },
            ..
        }) = &self.freezing
        {
            let frame = *frame;
            let request = self.freezing.take().unwrap();
            return Ok(Some(SupplyReceipt {
                id: request.id,
                result: Ok(SupplyOutcome::Located {
                    frame,
                    previous_copy_anchor: next,
                }),
                _credit: request.credit,
                retained: Vec::new(),
                retained_revision: 0,
            }));
        }
        if let Some(track) = &mut self.background {
            track.restore_position(next).map_err(backend)?;
        }
        let runtime = self.controller.runtime();
        runtime
            .seek_scene(runtime.token(Channel::Scene), next)
            .map_err(backend)?;
        let request = self.freezing.take().expect("checked freeze");
        Ok(Some(SupplyReceipt {
            id: request.id,
            result: Ok(SupplyOutcome::Frozen {
                next_background_frame: next,
            }),
            _credit: request.credit,
            retained: Vec::new(),
            retained_revision: 0,
        }))
    }
    fn prepare(&mut self) -> Result<bool, SupplyError> {
        let mut changed = false;
        if let Some(result) = self.worker.try_result() {
            self.last_preparation = Some(result.observation());
            self.controller.finish_selected(result).map_err(backend)?;
            self.activate_library()?;
            changed = true;
        }
        if let Some(job) = self.controller.start_preparation() {
            self.worker
                .try_submit(job)
                .map_err(|_| SupplyError::Closed)?;
        }
        Ok(changed)
    }
    fn activate_library(&mut self) -> Result<(), SupplyError> {
        let runtime = self.controller.runtime();
        if let Some(asset) = self.library_intent
            && let Some((id, status)) = runtime.library_status()
            && id == asset
            && status.preparation != skin_editor_audio::Preparation::Pending
        {
            self.library_intent = None;
            if status.preparation == skin_editor_audio::Preparation::Ready
                && runtime
                    .trigger(
                        runtime.token(Channel::Audition),
                        SoundTarget::Library(asset),
                        runtime.mix_frame(),
                    )
                    .is_err()
            {
                runtime.fail_library(skin_editor_audio::PrepareFailure::ResourceLimit);
            }
        }
        Ok(())
    }
    fn status(&mut self) -> SupplyStatus {
        let voices = self.voice_counts();
        SupplyStatus {
            retained_revision: 0,
            origin: self.selected,
            generated_background: self.generated_background(),
            copied_background: self.output.background_next(),
            background_position: self.separated_position,
            background_frames: self.background_frames,
            background_primed: self.separated_primed,
            frozen: self.frozen && self.freezing.is_none(),
            freezing: self.freezing.is_some(),
            ready_components: self
                .components
                .iter()
                .filter(|id| {
                    self.controller
                        .runtime()
                        .status(**id)
                        .is_some_and(|s| s.preparation == skin_editor_audio::Preparation::Ready)
                })
                .count(),
            retained_drafts: self.retained_drafts(),
            fault: None,
            bindings: self
                .components
                .iter()
                .filter_map(|id| self.controller.runtime().status(*id).map(|s| (*id, s)))
                .collect(),
            voices,
            library: self.controller.runtime().library_status(),
            last_preparation: self.last_preparation,
            max_supply_gap_micros: self.max_supply_gap_micros,
        }
    }
    fn voice_counts(&mut self) -> [usize; 2] {
        let mut counts = [0; 2];
        for voice in self.controller.runtime().voices() {
            counts[usize::from(voice.channel == Channel::Audition)] += 1;
        }
        counts
    }
}
impl Drop for SupplyCore {
    fn drop(&mut self) {
        self.output.close();
        self.controller.close();
        self.worker.close();
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn accepted_control_wakes_owner_even_if_notification_precedes_park() {
        let budget = skin_editor_audio::PcmBudget::default();
        let (output, callback) = crate::output_boundary(&budget, 48000, 1).unwrap();
        let mut client = super::AudioSupplyClient::spawn(
            output,
            48000,
            1,
            budget.clone(),
            crate::BlobStore::default(),
        )
        .unwrap();
        client.close();
        let (sender, requests) = std::sync::mpsc::sync_channel::<super::Request>(1);
        let (entered, seen) = std::sync::mpsc::sync_channel(1);
        let release = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let resume = release.clone();
        let (sent, observed) = std::sync::mpsc::sync_channel(1);
        client.sender = sender;
        client.thread = Some(std::thread::spawn(move || {
            entered.try_send(()).unwrap();
            let until = std::time::Instant::now() + std::time::Duration::from_secs(5);
            while !resume.load(std::sync::atomic::Ordering::Acquire) {
                assert!(
                    std::time::Instant::now() < until,
                    "controlled park gate not released"
                );
                std::thread::yield_now();
            }
            std::thread::park();
            let request = requests.try_recv().unwrap();
            sent.send(request.id).unwrap();
            None
        }));
        seen.recv_timeout(std::time::Duration::from_secs(5))
            .unwrap();
        let credit = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(1));
        let id = client
            .send(super::Control::Barrier, super::Credit(credit.clone()), None)
            .unwrap();
        release.store(true, std::sync::atomic::Ordering::Release);
        let result = observed.recv_timeout(std::time::Duration::from_secs(5));
        client.close(); // Also releases a deliberately broken wake path on failure.
        assert_eq!(result.unwrap(), id);
        assert_eq!(credit.load(std::sync::atomic::Ordering::Acquire), 0);
        drop((client, callback));
        assert_eq!(budget.usage().bytes, 0);
    }
    #[test]
    fn retirement_observation_requires_a_status_at_least_as_new_as_the_receipt() {
        let budget = skin_editor_audio::PcmBudget::default();
        let (output, callback) = crate::output_boundary(&budget, 48000, 1).unwrap();
        let mut client = super::AudioSupplyClient::spawn(
            output,
            48000,
            1,
            budget.clone(),
            crate::BlobStore::default(),
        )
        .unwrap();
        client.close(); // Freeze the producer for deterministic metadata ordering.
        let old = skin_editor_application::DraftId::new(8).unwrap();
        let other = skin_editor_application::DraftId::new(9).unwrap();
        client.reported_retained = vec![old];
        client.reported_revision = 2;
        {
            let mut s = client.status.lock().unwrap();
            s.retained_revision = 1;
            s.retained_drafts = vec![other];
        }
        assert_eq!(
            skin_editor_application::SkinAudioPort::retained_drafts(&client),
            vec![old, other]
        );
        {
            let mut s = client.status.lock().unwrap();
            s.retained_revision = 2;
            s.retained_drafts = vec![other];
        }
        assert_eq!(
            skin_editor_application::SkinAudioPort::retained_drafts(&client),
            vec![other]
        );
        {
            let mut s = client.status.lock().unwrap();
            s.retained_revision = 3;
            s.retained_drafts.clear();
        }
        assert!(skin_editor_application::SkinAudioPort::retained_drafts(&client).is_empty());
        drop(client);
        drop(callback);
        assert_eq!(budget.usage().bytes, 0);
    }
    mod separated;
    #[test]
    fn owned_recovery_probe_uses_the_same_trigger_gate_and_returns_control_credit() {
        let budget = PcmBudget::default();
        let store = BlobStore::default();
        let (mut output, mut callback) = output_boundary(&budget, 48000, 1).unwrap();
        output.configure_period(480).unwrap();
        output
            .configure_supply_policy(crate::SupplyPolicy::RecoverLateWake)
            .unwrap();
        output.control().capture_submissions().unwrap();
        let mut core = SupplyCore::new(&output, 48000, 1, budget.clone()).unwrap();
        let (_raw, plan) = raw_plan(&store);
        let mark = core.output.mark();
        select(&mut core, &store, origin(0), vec![plan], vec![], mark);
        finish(&mut core);
        let request = request(Control::DiagnosticRecovery {
            trigger: Some((origin(0), Channel::Audition, id())),
        });
        let credit = request.credit.0.clone();
        let receipt = core.apply_owned(&mut output, request, &store).unwrap();
        assert!(matches!(receipt.result, Ok(SupplyOutcome::Triggered(r)) if r.voice.is_some()));
        assert_eq!(credit.load(Ordering::Acquire), 1);
        pump(&mut core, &mut output);
        assert_eq!(output.stats().diagnostic_refill_passes, 1);
        callback.render_f32(&mut [0.; 128]);
        drop(receipt);
        assert_eq!(credit.load(Ordering::Acquire), 0);
        drop((core, output, callback));
        assert_eq!(budget.usage().bytes, 0);
    }
    #[test]
    fn idle_stale_and_failed_trigger_probes_cannot_leave_an_armed_recovery() {
        let budget = PcmBudget::default();
        let store = BlobStore::default();
        let (mut output, _) = output_boundary(&budget, 48000, 1).unwrap();
        output.configure_period(480).unwrap();
        output
            .configure_supply_policy(crate::SupplyPolicy::RecoverLateWake)
            .unwrap();
        output.control().capture_submissions().unwrap();
        let mut core = SupplyCore::new(&output, 48000, 1, budget).unwrap();
        for control in [
            Control::DiagnosticRecovery { trigger: None },
            Control::DiagnosticRecovery {
                trigger: Some((origin(9), Channel::Audition, id())),
            },
        ] {
            assert!(
                core.apply_owned(&mut output, request(control), &store)
                    .unwrap()
                    .result
                    .is_err()
            );
        }
        let (_raw, plan) = raw_plan(&store);
        let mark = core.output.mark();
        select(&mut core, &store, origin(0), vec![plan], vec![], mark);
        let invalid = AudioComponentId::new(99).unwrap();
        assert!(
            core.apply_owned(
                &mut output,
                request(Control::DiagnosticRecovery {
                    trigger: Some((origin(0), Channel::Audition, invalid))
                }),
                &store
            )
            .unwrap()
            .result
            .is_err()
        );
        pump(&mut core, &mut output);
        assert_eq!(output.stats().diagnostic_recovery_passes, 0);
    }
    use super::*;
    use crate::{OutputConsumer, RawBlob, output_boundary};
    use skin_editor_application::{values::*, *};
    use skin_editor_audio::TriggerReceipt;
    use std::time::Duration;
    #[test]
    fn controls_keep_order_credit_backpressure_and_close_release() {
        let budget = PcmBudget::default();
        let (producer, callback) = output_boundary(&budget, 48000, 1).unwrap();
        let mut client =
            AudioSupplyClient::spawn(producer, 48000, 1, budget.clone(), BlobStore::default())
                .unwrap();
        let ids: Vec<_> = (0..MAX_SUPPLY_CONTROLS)
            .map(|_| client.barrier().unwrap())
            .collect();
        assert_eq!(client.barrier(), Err(SupplyError::Busy));
        let receipts: Vec<_> = (0..MAX_SUPPLY_CONTROLS)
            .map(|_| {
                client
                    .receiver
                    .recv_timeout(Duration::from_secs(5))
                    .unwrap()
            })
            .collect();
        assert_eq!(receipts.iter().map(|r| r.id).collect::<Vec<_>>(), ids);
        assert!(
            receipts
                .iter()
                .all(|r| matches!(r.result, Ok(SupplyOutcome::Observed)))
        );
        assert_eq!(client.barrier(), Err(SupplyError::Busy)); // receipts still own all credits
        drop(receipts);
        for _ in 0..MAX_SUPPLY_CONTROLS {
            client.barrier().unwrap();
        }
        client.close();
        assert_eq!(client.credits.load(Ordering::Acquire), 0);
        drop((client, callback));
        assert_eq!(budget.usage().bytes, 0);
    }
    fn origin(revision: u64) -> Origin {
        Origin {
            session: 1,
            node: revision + 1,
            revision,
            navigation_revision: 0,
            draft: None,
        }
    }
    fn id() -> AudioComponentId {
        AudioComponentId::new(1).unwrap()
    }
    fn request(control: Control) -> Request {
        Request {
            id: 1,
            control,
            credit: Credit(Arc::new(AtomicUsize::new(1))),
        }
    }
    fn apply(core: &mut SupplyCore, store: &BlobStore, control: Control) -> SupplyOutcome {
        core.apply(request(control), store).unwrap().result.unwrap()
    }
    fn reference(budget: &PcmBudget) -> BackgroundTrack {
        let mut pcm = budget.reserve(8192).unwrap().allocate();
        for (i, value) in pcm.samples_mut().unwrap().iter_mut().enumerate() {
            *value = i as f32 / 32768.0;
        }
        BackgroundTrack::new(pcm, 48000, 1).unwrap()
    }
    fn pump(core: &mut SupplyCore, output: &mut OutputProducer) {
        output
            .pump_timeline(
                core.controller.runtime(),
                core.background.as_mut(),
                core.mark,
                OUTPUT_AHEAD_FRAMES,
            )
            .unwrap();
    }
    fn render_reference(callback: &mut OutputConsumer, first: usize, count: usize) {
        let mut actual = vec![0.0; count];
        callback.render_f32(&mut actual);
        for (offset, value) in actual.iter().enumerate() {
            assert_eq!(*value, (first + offset) as f32 / 32768.0);
        }
    }
    fn select(
        core: &mut SupplyCore,
        store: &BlobStore,
        origin: Origin,
        plans: Vec<SoundPlan>,
        cancelled: Vec<DraftId>,
        mark: OutputMark,
    ) {
        apply(
            core,
            store,
            Control::Select(Projection {
                origin,
                plans,
                cancelled,
                mark,
            }),
        );
    }
    fn wave() -> Vec<u8> {
        let size = 8192u32;
        let mut bytes = Vec::new();
        bytes.extend(b"RIFF");
        bytes.extend((36 + size).to_le_bytes());
        bytes.extend(b"WAVEfmt ");
        bytes.extend(16u32.to_le_bytes());
        bytes.extend(1u16.to_le_bytes());
        bytes.extend(1u16.to_le_bytes());
        bytes.extend(48000u32.to_le_bytes());
        bytes.extend(96000u32.to_le_bytes());
        bytes.extend(2u16.to_le_bytes());
        bytes.extend(16u16.to_le_bytes());
        bytes.extend(b"data");
        bytes.extend(size.to_le_bytes());
        for _ in 0..4096 {
            bytes.extend(4096i16.to_le_bytes());
        }
        bytes
    }
    fn raw_plan(store: &BlobStore) -> (RawBlob, SoundPlan) {
        let raw = store.stage_audio(wave(), &WavDecoder).unwrap();
        let plan = SoundPlan {
            target: (id()).into(),
            source: raw.reference(),
            metadata: raw.audio_metadata(),
            gain: Gain::UNITY,
            visible: true,
        };
        (raw, plan)
    }
    fn finish(core: &mut SupplyCore) {
        while let Some(job) = core.controller.start_preparation() {
            let done = job.execute(&mut WavDecoder);
            core.controller.finish_selected(done).unwrap();
        }
    }
    #[test]
    fn skin_replacement_navigation_and_cancel_never_reset_background() {
        for policy in [
            crate::SupplyPolicy::FixedAhead,
            crate::SupplyPolicy::RecoverLateWake,
        ] {
            let budget = PcmBudget::default();
            let store = BlobStore::default();
            let (mut output, mut callback) = output_boundary(&budget, 48000, 1).unwrap();
            output.configure_period(480).unwrap();
            output.configure_supply_policy(policy).unwrap();
            let wake = std::time::Instant::now();
            output.owner_woke(wake);
            output.owner_woke(wake + std::time::Duration::from_millis(34));
            let mut core = SupplyCore::new(&output, 48000, 1, budget.clone()).unwrap();
            apply(&mut core, &store, Control::Background(reference(&budget)));
            let (_raw, mut plan) = raw_plan(&store);
            let mark = core.output.mark();
            select(&mut core, &store, origin(0), vec![plan], vec![], mark);
            finish(&mut core);
            let mark = core.output.invalidate_scene().unwrap();
            apply(
                &mut core,
                &store,
                Control::Scene {
                    running: true,
                    mark,
                },
            );
            if policy == crate::SupplyPolicy::RecoverLateWake {
                output.control().capture_submissions().unwrap();
                output.arm_diagnostic_recovery().unwrap();
            }
            pump(&mut core, &mut output);
            assert_eq!(
                output.stats().recovery_passes > 0,
                policy == crate::SupplyPolicy::RecoverLateWake
            );
            render_reference(&mut callback, 0, 128);
            let generated = core.background.as_ref().unwrap().position();
            plan.gain = Gain::new(0.5).unwrap();
            let mark = core.output.mark();
            select(&mut core, &store, origin(1), vec![plan], vec![], mark);
            finish(&mut core);
            assert_eq!(core.background.as_ref().unwrap().position(), generated);
            render_reference(&mut callback, 128, 128);
            for (index, cancelled) in [false, true].into_iter().enumerate() {
                let mark = core.output.invalidate_skin(true, true).unwrap();
                let mut source = origin(2 + index as u64);
                source.navigation_revision = 1;
                let drafts = if cancelled {
                    vec![DraftId::new(7).unwrap()]
                } else {
                    vec![]
                };
                select(&mut core, &store, source, vec![plan], drafts, mark);
                pump(&mut core, &mut output);
                render_reference(&mut callback, 256 + index * 128, 128);
            }
            assert_eq!(core.output.premixed().unwrap().stats().background_gaps, 0);
            assert_eq!(core.output.premixed().unwrap().stats().background_next, 512);
            drop(core);
            drop(output);
            drop(callback);
            assert_eq!(budget.usage().bytes, 0);
        }
    }
    #[test]
    fn scene_hot_activation_keeps_started_one_shot_gain() {
        let budget = PcmBudget::default();
        let store = BlobStore::default();
        let (mut output, mut callback) = output_boundary(&budget, 48000, 1).unwrap();
        let mut core = SupplyCore::new(&output, 48000, 1, budget.clone()).unwrap();
        apply(&mut core, &store, Control::Background(reference(&budget)));
        let (_raw, mut plan) = raw_plan(&store);
        let mark = core.output.mark();
        select(&mut core, &store, origin(0), vec![plan], vec![], mark);
        finish(&mut core);
        let mark = core.output.invalidate_scene().unwrap();
        apply(
            &mut core,
            &store,
            Control::Scene {
                running: true,
                mark,
            },
        );
        apply(
            &mut core,
            &store,
            Control::Trigger {
                origin: origin(0),
                lane: Channel::Scene,
                component: id(),
                stop_library: None,
            },
        );
        pump(&mut core, &mut output);
        plan.gain = Gain::new(0.5).unwrap();
        let mark = core.output.mark();
        select(&mut core, &store, origin(1), vec![plan], vec![], mark);
        finish(&mut core);
        let receipt = apply(
            &mut core,
            &store,
            Control::Trigger {
                origin: origin(1),
                lane: Channel::Scene,
                component: id(),
                stop_library: None,
            },
        );
        assert!(
            matches!(receipt, SupplyOutcome::Triggered(TriggerReceipt { origin: o, stale: false, .. }) if o == origin(1))
        );
        let mut old = [0.0; OUTPUT_AHEAD_FRAMES];
        callback.render_f32(&mut old);
        for (i, value) in old.iter().enumerate() {
            assert_eq!(*value, 0.125 + i as f32 / 32768.0);
        }
        pump(&mut core, &mut output);
        let mut new = [0.0; 128];
        callback.render_f32(&mut new);
        for (i, value) in new.iter().enumerate() {
            assert_eq!(*value, 0.1875 + (OUTPUT_AHEAD_FRAMES + i) as f32 / 32768.0);
        }
        assert_eq!(core.output.premixed().unwrap().stats().background_gaps, 0);
        assert_eq!(core.controller.runtime().voices().count(), 2);
    }
    #[test]
    fn freeze_waits_for_callback_rewinds_unused_ahead_and_resumes_exact_sample() {
        for policy in [
            crate::SupplyPolicy::FixedAhead,
            crate::SupplyPolicy::RecoverLateWake,
        ] {
            let budget = PcmBudget::default();
            let store = BlobStore::default();
            let (mut output, mut callback) = output_boundary(&budget, 48000, 1).unwrap();
            output.configure_period(480).unwrap();
            output.configure_supply_policy(policy).unwrap();
            let wake = std::time::Instant::now();
            output.owner_woke(wake);
            output.owner_woke(wake + std::time::Duration::from_millis(34));
            let mut core = SupplyCore::new(&output, 48000, 1, budget.clone()).unwrap();
            apply(&mut core, &store, Control::Background(reference(&budget)));
            let (_raw, plan) = raw_plan(&store);
            let mark = core.output.mark();
            select(&mut core, &store, origin(0), vec![plan], vec![], mark);
            finish(&mut core);
            let mark = core.output.invalidate_scene().unwrap();
            apply(
                &mut core,
                &store,
                Control::Scene {
                    running: true,
                    mark,
                },
            );
            if policy == crate::SupplyPolicy::RecoverLateWake {
                output.control().capture_submissions().unwrap();
                output.arm_diagnostic_recovery().unwrap();
            }
            pump(&mut core, &mut output);
            assert_eq!(
                output.stats().recovery_passes > 0,
                policy == crate::SupplyPolicy::RecoverLateWake
            );
            render_reference(&mut callback, 0, 64);
            assert!(core.background.as_ref().unwrap().position() > 64);
            let mark = core.output.invalidate_scene().unwrap();
            assert!(
                core.apply(
                    request(Control::Scene {
                        running: false,
                        mark
                    }),
                    &store
                )
                .is_none()
            );
            assert!(!core.status().frozen);
            assert!(core.finish_freeze().unwrap().is_none());
            let mut silent = [1.0; OUTPUT_QUEUE_FRAMES];
            callback.render_f32(&mut silent);
            assert!(silent.iter().all(|v| *v == 0.0));
            let receipt = core.finish_freeze().unwrap().unwrap();
            assert!(matches!(
                receipt.result,
                Ok(SupplyOutcome::Frozen {
                    next_background_frame: 64
                })
            ));
            assert!(core.status().frozen);
            assert_eq!(core.background.as_ref().unwrap().position(), 64);
            // Audition is live while the separate background and scene clocks stay frozen.
            apply(
                &mut core,
                &store,
                Control::Trigger {
                    origin: origin(0),
                    lane: Channel::Audition,
                    component: id(),
                    stop_library: None,
                },
            );
            pump(&mut core, &mut output);
            let mut audition = [0.0; 128];
            callback.render_f32(&mut audition);
            assert_eq!(audition, [0.125; 128]);
            assert_eq!(core.output.premixed().unwrap().stats().background_next, 64);
            assert_eq!(core.controller.runtime().scene_frame(), 64);
            let mark = core.output.invalidate_skin(false, true).unwrap();
            apply(&mut core, &store, Control::StopAudition(mark));
            callback.render_f32(&mut [0.0; OUTPUT_QUEUE_FRAMES]);
            let mark = core.output.invalidate_scene().unwrap();
            apply(
                &mut core,
                &store,
                Control::Scene {
                    running: true,
                    mark,
                },
            );
            pump(&mut core, &mut output);
            render_reference(&mut callback, 64, 128);
            assert_eq!(core.output.premixed().unwrap().stats().background_gaps, 0);
        }
    }
    #[test]
    fn desired_fence_cannot_relabel_old_runtime_samples_or_cut_background() {
        let budget = PcmBudget::default();
        let store = BlobStore::default();
        let (mut output, mut callback) = output_boundary(&budget, 48000, 1).unwrap();
        let mut core = SupplyCore::new(&output, 48000, 1, budget.clone()).unwrap();
        apply(&mut core, &store, Control::Background(reference(&budget)));
        let (_raw, plan) = raw_plan(&store);
        let mark = core.output.mark();
        let mut draft = origin(0);
        draft.draft = Some((7, 1));
        select(&mut core, &store, draft, vec![plan], vec![], mark);
        finish(&mut core);
        let mark = core.output.invalidate_scene().unwrap();
        apply(
            &mut core,
            &store,
            Control::Scene {
                running: true,
                mark,
            },
        );
        apply(
            &mut core,
            &store,
            Control::Trigger {
                origin: draft,
                lane: Channel::Audition,
                component: id(),
                stop_library: None,
            },
        );
        let runtime = core.controller.runtime();
        runtime
            .trigger(
                runtime.token(Channel::Audition),
                id(),
                runtime.mix_frame() + 10000,
            )
            .unwrap();
        assert_eq!(runtime.queued(), 1);
        // Control caller invalidates now, but supply has not received its projection yet.
        let mark = core.output.invalidate_skin(true, true).unwrap();
        pump(&mut core, &mut output);
        render_reference(&mut callback, 0, 128);
        select(
            &mut core,
            &store,
            origin(1),
            vec![plan],
            vec![DraftId::new(7).unwrap()],
            mark,
        );
        assert_eq!(core.controller.runtime().voices().count(), 0);
        assert_eq!(core.controller.runtime().queued(), 0);
        let old = core
            .apply(
                request(Control::Trigger {
                    origin: draft,
                    lane: Channel::Audition,
                    component: id(),
                    stop_library: None,
                }),
                &store,
            )
            .unwrap();
        assert_eq!(old.result.err(), Some(SupplyError::Stale));
        render_reference(&mut callback, 128, 128);
        assert!(core.output.premixed().unwrap().stats().stale_lane_frames > 0);
    }
    #[test]
    fn incompatible_background_or_supply_budget_is_rejected_and_reservations_return() {
        let budget = PcmBudget::default();
        let foreign = PcmBudget::default();
        let store = BlobStore::default();
        let (output, callback) = output_boundary(&budget, 48000, 1).unwrap();
        let mut core = SupplyCore::new(&output, 48000, 1, budget.clone()).unwrap();
        for track in [
            reference(&foreign),
            BackgroundTrack::new(budget.reserve(8).unwrap().allocate(), 44100, 1).unwrap(),
        ] {
            let receipt = core
                .apply(request(Control::Background(track)), &store)
                .unwrap();
            assert_eq!(receipt.result.err(), Some(SupplyError::Unsupported));
            assert!(core.background.is_none());
        }
        assert_eq!(foreign.usage().bytes, 0);
        drop(core);
        drop(output);
        drop(callback);
        assert_eq!(budget.usage().bytes, 0);
        let (output, callback) = output_boundary(&budget, 48000, 1).unwrap();
        assert!(AudioSupplyClient::spawn(output, 48000, 1, foreign.clone(), store).is_err());
        drop(callback);
        assert_eq!(budget.usage().bytes, 0);
        assert_eq!(foreign.usage().bytes, 0);
    }
    fn receive(client: &mut AudioSupplyClient, id: u64) -> SupplyOutcome {
        loop {
            let receipt = client
                .wait_receipt(Duration::from_secs(5))
                .unwrap()
                .expect("supply receipt");
            if receipt.id == id {
                return receipt.result.unwrap();
            }
            receipt.result.as_ref().unwrap();
        }
    }
    #[test]
    fn independent_owner_supplies_without_gui_ticks_and_redundant_resume_is_harmless() {
        let budget = PcmBudget::default();
        let store = BlobStore::default();
        let (output, mut callback) = output_boundary(&budget, 48000, 1).unwrap();
        let mut client = AudioSupplyClient::spawn(output, 48000, 1, budget.clone(), store).unwrap();
        let request = client.set_background(reference(&budget)).unwrap();
        receive(&mut client, request);
        callback.render_f32(&mut [0.0; OUTPUT_QUEUE_FRAMES]);
        let request = client.scene_running(true).unwrap();
        receive(&mut client, request);
        // Receipt is a barrier after owner pumping; no GUI frame or pump call here.
        render_reference(&mut callback, 0, 128);
        let request = client.scene_running(true).unwrap();
        receive(&mut client, request);
        render_reference(&mut callback, 128, 128);
        assert_eq!(client.output.premixed().unwrap().stats().background_gaps, 0);
        client.close();
        let mut silence = [1.0; 128];
        callback.render_f32(&mut silence);
        assert_eq!(silence, [0.0; 128]);
        drop(client);
        drop(callback);
        assert_eq!(budget.usage().bytes, 0);
    }
    #[test]
    fn control_credit_covers_unread_and_retained_replies_and_close_bypasses_full_queue() {
        let budget = PcmBudget::default();
        let (output, callback) = output_boundary(&budget, 48000, 1).unwrap();
        let mut client =
            AudioSupplyClient::spawn(output, 48000, 1, budget.clone(), BlobStore::default())
                .unwrap();
        for _ in 0..MAX_SUPPLY_CONTROLS {
            client.stop_audition().unwrap();
        }
        assert_eq!(client.stop_audition(), Err(SupplyError::Busy));
        let held = client
            .wait_receipt(Duration::from_secs(5))
            .unwrap()
            .unwrap();
        assert_eq!(client.stop_audition(), Err(SupplyError::Busy));
        drop(held);
        client.stop_audition().unwrap();
        client.close();
        assert_eq!(client.stop_audition(), Err(SupplyError::Closed));
        assert_eq!(client.credits.load(Ordering::Acquire), 0);
        drop(client);
        drop(callback);
        assert_eq!(budget.usage().bytes, 0);
    }
    #[test]
    fn projection_uses_application_cancel_facts_even_without_status_polling() {
        let budget = PcmBudget::default();
        let store = BlobStore::default();
        let (raw, _) = raw_plan(&store);
        let mut session =
            ProjectSession::with_resources(1, 1, Box::new(store.clone()), SessionLimits::default())
                .unwrap();
        session
            .submit(CommandRequest {
                api_version: API_VERSION,
                session_id: session.project_info().session_id,
                request_id: RequestId::new(1).unwrap(),
                expected: session.project_info().state,
                commands: vec![
                    Command::RegisterAsset {
                        asset: Asset::new(
                            AssetId::new(1).unwrap(),
                            "self.wav".into(),
                            raw.reference(),
                            AssetKind::Audio(raw.audio_metadata().unwrap()),
                        )
                        .unwrap(),
                    },
                    Command::CreateAudioComponent {
                        component: AudioComponent::new(
                            id(),
                            SoundSlotKey::new("self".into()).unwrap(),
                            AssetId::new(1).unwrap(),
                        ),
                    },
                ],
            })
            .unwrap();
        let base = session.snapshot();
        let nodes = session.history_len();
        let (output, callback) = output_boundary(&budget, 48000, 1).unwrap();
        let mut client = AudioSupplyClient::spawn(output, 48000, 1, budget.clone(), store).unwrap();
        session
            .edit_draft(DraftRequest {
                api_version: API_VERSION,
                session_id: session.project_info().session_id,
                request_id: RequestId::new(2).unwrap(),
                action: DraftAction::Begin {
                    expected: session.project_info().state,
                },
            })
            .unwrap();
        client.synchronize(&session).unwrap();
        // Pending projection retains the draft before any asynchronous status query.
        assert_eq!(client.retained_drafts().len(), 1);
        let state = session
            .draft_info(session.project_info().session_id)
            .unwrap()
            .unwrap()
            .state;
        session
            .edit_draft(DraftRequest {
                api_version: API_VERSION,
                session_id: session.project_info().session_id,
                request_id: RequestId::new(3).unwrap(),
                action: DraftAction::Cancel { expected: state },
            })
            .unwrap();
        client.synchronize(&session).unwrap();
        let barrier = client.stop_audition().unwrap();
        receive(&mut client, barrier);
        assert_eq!(session.snapshot(), base);
        assert_eq!(session.history_len(), nodes);
        assert!(client.retained_drafts().is_empty());
        // Exercise actual single decoder; each explicit receipt is a bounded scheduling barrier.
        for _ in 0..256 {
            let barrier = client.stop_audition().unwrap();
            receive(&mut client, barrier);
            if client.status().unwrap().ready_components == 1 {
                break;
            }
        }
        assert_eq!(client.status().unwrap().ready_components, 1);
        let trigger = client.trigger(&session, Channel::Audition, id()).unwrap();
        assert!(matches!(
            receive(&mut client, trigger),
            SupplyOutcome::Triggered(TriggerReceipt {
                voice: Some(_),
                stale: false,
                ..
            })
        ));
        assert_eq!(client.status().unwrap().origin.unwrap().draft, None);
        client.close();
        drop(client);
        drop(callback);
        drop(raw);
        drop(session);
        assert_eq!(budget.usage().bytes, 0);
    }
    #[test]
    fn late_library_decode_after_stop_or_navigation_cannot_activate_or_retain_pcm() {
        for stop in [true, false] {
            let budget = PcmBudget::default();
            let store = BlobStore::default();
            let (mut output, mut callback) = output_boundary(&budget, 48000, 1).unwrap();
            let mut core = SupplyCore::new(&output, 48000, 1, budget.clone()).unwrap();
            let (_raw, mut plan) = raw_plan(&store);
            plan.target = SoundTarget::Library(AssetId::new(1).unwrap());
            let mark = core.output.mark();
            select(&mut core, &store, origin(0), vec![], vec![], mark);
            let mark = core.output.invalidate_skin(false, true).unwrap();
            apply(
                &mut core,
                &store,
                Control::Library {
                    origin: origin(0),
                    plan,
                    mark,
                },
            );
            let running = core.controller.start_preparation().unwrap();
            // Hold a real decoded result until after cancellation, no sleep/race guess.
            let finished = running.execute(&mut WavDecoder);
            assert!(core.controller.preparation_usage().pcm.bytes > 0);
            let mark = core.output.invalidate_skin(false, true).unwrap();
            if stop {
                apply(&mut core, &store, Control::StopAudition(mark));
            } else {
                let mut o = origin(0);
                o.navigation_revision = 1;
                select(&mut core, &store, o, vec![], vec![], mark);
            }
            assert_eq!(
                core.controller.finish_selected(finished).unwrap(),
                skin_editor_audio::Completion::Stale
            );
            core.activate_library().unwrap();
            assert!(core.library_intent.is_none());
            assert!(core.status().library.is_none());
            assert_eq!(core.voice_counts(), [0, 0]);
            pump(&mut core, &mut output);
            let mut samples = [1.0; 128];
            callback.render_f32(&mut samples);
            assert_eq!(samples, [0.0; 128]);
            drop(core);
            drop(output);
            drop(callback);
            assert_eq!(budget.usage().bytes, 0);
        }
    }
    #[test]
    fn library_voice_limit_is_visible_and_does_not_close_supply_or_stop_scene() {
        let budget = PcmBudget::default();
        let store = BlobStore::default();
        let (output, callback) = output_boundary(&budget, 48000, 1).unwrap();
        let mut core = SupplyCore::new(&output, 48000, 1, budget.clone()).unwrap();
        let (_raw, mut plan) = raw_plan(&store);
        let mark = core.output.mark();
        select(&mut core, &store, origin(0), vec![plan], vec![], mark);
        finish(&mut core);
        let r = core.controller.runtime();
        r.set_scene_running(r.token(Channel::Scene), true).unwrap();
        for _ in 0..skin_editor_audio::MAX_VOICES {
            r.trigger(r.token(Channel::Scene), id(), r.mix_frame())
                .unwrap();
        }
        plan.target = SoundTarget::Library(AssetId::new(1).unwrap());
        let mark = core.output.invalidate_skin(false, true).unwrap();
        apply(
            &mut core,
            &store,
            Control::Library {
                origin: origin(0),
                plan,
                mark,
            },
        );
        finish(&mut core);
        core.activate_library().unwrap();
        assert_eq!(
            core.status().library.unwrap().1.preparation,
            skin_editor_audio::Preparation::Failed(
                skin_editor_audio::PrepareFailure::ResourceLimit
            )
        );
        assert_eq!(core.voice_counts(), [32, 0]);
        assert!(!core.output.is_closed());
        drop(core);
        drop(output);
        drop(callback);
        assert_eq!(budget.usage().bytes, 0);
    }
    #[test]
    fn repeated_library_audition_reuses_pcm_without_a_component_or_another_decode() {
        let budget = PcmBudget::default();
        let store = BlobStore::default();
        let (output, callback) = output_boundary(&budget, 48000, 1).unwrap();
        let mut core = SupplyCore::new(&output, 48000, 1, budget.clone()).unwrap();
        let boundary_bytes = budget.usage().bytes;
        let (_raw, mut plan) = raw_plan(&store);
        plan.target = SoundTarget::Library(AssetId::new(1).unwrap());
        let mark = core.output.mark();
        select(&mut core, &store, origin(0), vec![], vec![], mark);
        let mark = core.output.invalidate_skin(false, true).unwrap();
        apply(
            &mut core,
            &store,
            Control::Library {
                origin: origin(0),
                plan,
                mark,
            },
        );
        finish(&mut core);
        core.activate_library().unwrap();
        let first_bytes = budget.usage().bytes;
        assert!(first_bytes > boundary_bytes);
        // Different library id, identical immutable bytes and recipe.
        plan.target = SoundTarget::Library(AssetId::new(2).unwrap());
        let mark = core.output.invalidate_skin(false, true).unwrap();
        apply(
            &mut core,
            &store,
            Control::Library {
                origin: origin(0),
                plan,
                mark,
            },
        );
        assert!(core.controller.start_preparation().is_none());
        assert_eq!(
            core.status().library.unwrap().1.preparation,
            skin_editor_audio::Preparation::Ready
        );
        assert_eq!(core.voice_counts(), [0, 1]);
        assert_eq!(budget.usage().bytes, first_bytes);
        let mark = core.output.invalidate_skin(false, true).unwrap();
        apply(&mut core, &store, Control::StopAudition(mark));
        assert_eq!(budget.usage().bytes, boundary_bytes);
        drop(core);
        drop(output);
        drop(callback);
        assert_eq!(budget.usage().bytes, 0);
    }
    #[test]
    fn bounded_batch_starts_32_voices_at_one_frame_and_refuses_33rd_without_stealing() {
        let budget = PcmBudget::default();
        let store = BlobStore::default();
        let (mut output, mut callback) = output_boundary(&budget, 48000, 1).unwrap();
        let mut core = SupplyCore::new(&output, 48000, 1, budget.clone()).unwrap();
        let (_raw, plan) = raw_plan(&store);
        let mark = core.output.mark();
        select(&mut core, &store, origin(0), vec![plan], vec![], mark);
        finish(&mut core);
        let r = core.controller.runtime();
        r.set_scene_running(r.token(Channel::Scene), true).unwrap();
        let entries: Vec<_> = (0..32)
            .map(|i| {
                (
                    if i % 2 == 0 {
                        Channel::Scene
                    } else {
                        Channel::Audition
                    },
                    id(),
                )
            })
            .collect();
        let result = apply(
            &mut core,
            &store,
            Control::TriggerBatch {
                origin: origin(0),
                entries,
                stop_library: None,
            },
        );
        let SupplyOutcome::BatchTriggered { mix_frame, results } = result else {
            panic!("batch receipt missing")
        };
        assert_eq!(mix_frame, 0);
        assert_eq!(results.len(), 32);
        assert!(
            results
                .iter()
                .all(|r| r.as_ref().is_ok_and(|r| r.voice.is_some()))
        );
        assert!(core.controller.runtime().voices().all(|v| v.cursor == 0));
        let prior: Vec<_> = core.controller.runtime().voices().map(|v| v.id).collect();
        let result = apply(
            &mut core,
            &store,
            Control::TriggerBatch {
                origin: origin(0),
                entries: vec![(Channel::Audition, id())],
                stop_library: None,
            },
        );
        assert!(
            matches!(result,SupplyOutcome::BatchTriggered {results,..} if results==[Err(skin_editor_audio::RuntimeError::ResourceLimit)])
        );
        assert_eq!(
            prior,
            core.controller
                .runtime()
                .voices()
                .map(|v| v.id)
                .collect::<Vec<_>>()
        );
        let rejected = core
            .apply(
                request(Control::TriggerBatch {
                    origin: origin(0),
                    entries: vec![(Channel::Scene, id()); 33],
                    stop_library: None,
                }),
                &store,
            )
            .unwrap();
        assert_eq!(rejected.result.unwrap_err(), SupplyError::ResourceLimit);
        pump(&mut core, &mut output);
        callback.render_f32(&mut [0.0; 128]);
        assert_eq!(
            core.output.premixed().unwrap().stats().lane_nonzero_frames,
            [128, 128]
        );
        drop(core);
        drop(output);
        drop(callback);
        assert_eq!(budget.usage().bytes, 0);
    }
}
