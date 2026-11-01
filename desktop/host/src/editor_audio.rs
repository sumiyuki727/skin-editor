//! Desktop-injected application port; no GUI dependency in the supply runtime.
use skin_editor_application::{values::*, *};
use skin_editor_audio::{BackgroundTrack, Channel, Origin, PcmBudget, Preparation, TriggerReceipt};
use skin_editor_infrastructure::{
    AudioSupplyClient, BlobStore, NativeOutput, NativeStream, SupplyError, SupplyOutcome,
    separated_native::{EndpointReturnError, NativeExecutor, PreparedDevice},
};
pub(crate) mod combined;
pub(crate) mod cycles;
pub(crate) mod faults;
pub(crate) mod gui_verification;
pub(crate) mod stability;
pub(crate) mod window_recovery;
#[cfg(test)]
mod product_tests {
    use super::*;
    #[test]
    fn ordinary_audio_is_disabled_and_lazy_but_selects_the_platform_route() {
        let store = BlobStore::default();
        let a = crate::product_policy::Policy::CURRENT.audio(store.clone());
        assert_eq!(a.candidate, cfg!(target_os = "windows"));
        assert!(!a.goal && !a.stop_pending && !a.close_failed);
        assert!(a.client.is_none() && a.native.is_none() && a.stream.is_none());
        assert_eq!(a.budget.usage().bytes, 0);
        assert_eq!(store.usage().unwrap().bytes, 0);
    }
}
pub struct EditorAudio {
    store: BlobStore,
    budget: PcmBudget,
    client: Option<AudioSupplyClient>,
    stream: Option<NativeStream>,
    native: Option<NativeExecutor>,
    candidate: bool,
    reference: Option<BackgroundTrack>,
    buffered: Option<skin_editor_infrastructure::background_stream::BufferedBackground>,
    format: (u32, u16),
    background_frames: u64,
    last_triggered: Option<(u64, TriggerReceipt)>,
    last_trigger_request: Option<u64>,
    last_batch: Option<(u64, Result<Vec<u64>, String>)>,
    last_failed: Option<(u64, SupplyError)>,
    closed_stats: Option<skin_editor_infrastructure::separated_native::NativeStats>,
    goal: bool,
    monitor_volume: u16,
    transport: Option<u64>,
    error: Option<String>,
    last_audition: Option<(SkinAudioSource, bool)>,
    recovery: Option<Recovery>,
    close_failed: bool,
    stop_pending: bool,
    stranded_endpoint: Option<skin_editor_infrastructure::separated_output::StoppedOutput>,
    hold_return: bool,
    return_gate: Option<skin_editor_infrastructure::separated_native::ReturnGate>,
}
#[derive(Clone, Copy)]
struct Recovery {
    format: (u32, u16),
    copied_frame: Option<u64>,
    frames: u64,
}

fn preparation(p: Preparation) -> AudioPreparationState {
    match p {
        Preparation::Pending => AudioPreparationState::Pending,
        Preparation::Ready => AudioPreparationState::Ready,
        Preparation::Failed(e) => AudioPreparationState::Failed(format!("{e:?}")),
    }
}
fn source(o: Origin) -> SkinAudioSource {
    SkinAudioSource {
        session: HistorySessionId::new(o.session).expect("validated session"),
        state: ExpectedState {
            head: NodeId::new(o.node).expect("validated node"),
            revision: o.revision,
        },
        navigation_revision: o.navigation_revision,
        draft: o.draft.map(|(id, revision)| DraftState {
            id: DraftId::new(id).expect("validated draft"),
            revision,
            base: ExpectedState {
                head: NodeId::new(o.node).expect("validated node"),
                revision: o.revision,
            },
        }),
    }
}
impl EditorAudio {
    fn receive(&mut self, id: u64, result: &Result<SupplyOutcome, SupplyError>) {
        let transport = self.transport == Some(id);
        if transport {
            self.transport = None;
        }
        if transport && matches!(result, Err(SupplyError::Busy)) {
            return; // Desired state is retried once owner readiness is published.
        }
        match result {
            Ok(SupplyOutcome::Triggered(r)) => {
                self.last_audition = Some((source(r.origin), r.stale));
                self.last_triggered = Some((id, *r));
            }
            Ok(SupplyOutcome::BatchTriggered { results, .. }) => {
                self.last_batch = Some((
                    id,
                    results
                        .iter()
                        .map(|r| {
                            r.as_ref()
                                .map_err(|e| format!("{e:?}"))?
                                .voice
                                .ok_or("silent batch voice".into())
                        })
                        .collect(),
                ));
            }
            Err(e) => {
                self.error = Some(format!("{e:?}"));
                self.last_failed = Some((id, e.clone()));
            }
            _ => {}
        }
    }
    pub fn new(store: BlobStore) -> Self {
        Self {
            store,
            budget: PcmBudget::default(),
            client: None,
            stream: None,
            native: None,
            candidate: false,
            reference: None,
            buffered: None,
            format: (0, 0),
            background_frames: 0,
            last_triggered: None,
            last_trigger_request: None,
            last_batch: None,
            last_failed: None,
            closed_stats: None,
            goal: false,
            monitor_volume: 1000,
            transport: None,
            error: None,
            last_audition: None,
            recovery: None,
            close_failed: false,
            stop_pending: false,
            stranded_endpoint: None,
            hold_return: false,
            return_gate: None,
        }
    }
    pub fn candidate(store: BlobStore) -> Self {
        let mut adapter = Self::new(store);
        adapter.candidate = true;
        adapter
    }
    fn take_reference(&mut self, rate: u32, channels: u16) -> Result<BackgroundTrack, String> {
        // No song loaded in the ordinary editor yet. Empty background is explicit
        // EOF, not an invented continuous timeline. Diagnostics inject a reference.
        let (track, frames) = match self.reference.take() {
            Some(track) => {
                let frames = track.frames();
                (track, frames)
            }
            None => {
                let mut track = BackgroundTrack::new(
                    self.budget
                        .reserve(usize::from(channels))
                        .map_err(|e| format!("{e:?}"))?
                        .allocate(),
                    rate,
                    channels,
                )
                .map_err(|e| format!("{e:?}"))?;
                track.restore_position(1).map_err(|e| format!("{e:?}"))?;
                (track, 0)
            }
        };
        self.background_frames = frames;
        Ok(track)
    }
    fn enable_candidate(&mut self) -> Result<(), String> {
        let device = PreparedDevice::probe()?;
        let (rate, channels) = device.format();
        self.restore_background((rate, channels))?;
        let pair = if let Some(source) = self.buffered.take() {
            self.background_frames = source.frames();
            AudioSupplyClient::spawn_streamed(
                source,
                rate,
                channels,
                self.budget.clone(),
                self.store.clone(),
            )
        } else {
            let track = self.take_reference(rate, channels)?;
            AudioSupplyClient::spawn_separated(
                track,
                rate,
                channels,
                self.budget.clone(),
                self.store.clone(),
            )
        };
        let (client, endpoint) = pair.map_err(|e| format!("{e:?}"))?;
        let native = if self.hold_return {
            let (native, gate) = device.attach_with_held_return(endpoint);
            self.return_gate = Some(gate);
            native
        } else {
            device.attach(endpoint)
        };
        self.format = (rate, channels);
        self.client = Some(client);
        self.native = Some(native);
        if let Err(e) = self
            .native
            .as_ref()
            .expect("attached output")
            .set_monitor_volume(self.monitor_volume)
            .and_then(|_| self.native.as_ref().expect("attached output").play())
        {
            let cleanup = self.close();
            return Err(format!("{e}; cleanup: {cleanup:?}"));
        }
        self.recovery = None;
        Ok(())
    }
    fn restore_background(&mut self, format: (u32, u16)) -> Result<(), String> {
        let Some(anchor) = self.recovery else {
            return Ok(());
        };
        if anchor.frames == 0 {
            return Ok(());
        }
        if anchor.format != format {
            return Err("device format changed; background conversion is not implemented".into());
        }
        let copied = anchor
            .copied_frame
            .ok_or("final copy anchor unavailable; recovery refused")?;
        if let Some(source) = &mut self.buffered {
            if source.format() != format || source.frames() != anchor.frames {
                return Err("reopened background does not match recovery source".into());
            }
            source
                .restore_position(copied)
                .map_err(|e| format!("{e:?}"))
        } else if let Some(track) = &mut self.reference {
            if track.format() != format || track.frames() != anchor.frames {
                return Err("reopened background does not match recovery source".into());
            }
            track.restore_position(copied).map_err(|e| format!("{e:?}"))
        } else {
            Err("background must be reopened at the retained copy anchor before recovery".into())
        }
    }
    fn handle_native_fault(&mut self) {
        if self.stop_pending {
            if let Err(error) = self.close() {
                self.error = Some(error);
            }
            return;
        }
        let Some(stats) = self.native.as_ref().map(NativeExecutor::stats) else {
            return;
        };
        if !stats.device_fault && stats.render_errors == 0 && stats.observation_overflows == 0 {
            return;
        }
        self.recovery = Some(Recovery {
            format: self.format,
            copied_frame: None,
            frames: self.background_frames,
        });
        // No implicit resume after rebuilding. Old receipts and controls stay
        // inside the closed owner; the replacement needs a fresh projection.
        self.goal = false;
        let closed = self.close();
        self.error = Some(format!(
            "native device stopped: {stats:?}; cleanup: {closed:?}"
        ));
    }
    fn close(&mut self) -> Result<(), String> {
        self.goal = false;
        let mut result = Ok(());
        if let Some(client) = &mut self.client {
            client.close();
        }
        if let Some(native) = &mut self.native {
            match native.try_stop() {
                Err(EndpointReturnError::Pending) => {
                    self.stop_pending = true;
                    result = Err("audio endpoint return pending; owner retained".into());
                }
                Err(e) => {
                    self.close_failed = true;
                    self.stop_pending = false;
                    result = Err(format!(
                        "audio endpoint return failed: {e:?}; owner retained"
                    ));
                }
                Ok((stopped, stats)) => {
                    self.native.take();
                    self.stop_pending = false;
                    self.closed_stats = Some(stats);
                    let copied = stopped.copy_clock();
                    if let Some(anchor) = &mut self.recovery {
                        anchor.copied_frame = copied.map(|c| c.next_background);
                    }
                    let anchor_valid = self.recovery.is_none() || copied.is_some();
                    match self.client.as_mut().filter(|_| anchor_valid) {
                        Some(client) => {
                            if let Err(stopped) = client.finish_separated(stopped) {
                                self.stranded_endpoint = Some(stopped);
                                result =
                                    Err("audio worker/endpoint cleanup failed; endpoint retained"
                                        .into());
                            } else {
                                self.client.take();
                            }
                        }
                        None => {
                            self.stranded_endpoint = Some(stopped);
                            result = Err(
                                "final copy anchor or audio owner unavailable; endpoint retained"
                                    .into(),
                            );
                        }
                    }
                    self.close_failed |= result.is_err();
                }
            }
        } else if self.stranded_endpoint.is_some() {
            result = Err("audio endpoint cleanup failed; endpoint retained".into());
        } else {
            self.client.take();
        }
        self.stream.take();
        self.transport = None;
        self.last_audition = None;
        self.last_triggered = None;
        self.last_trigger_request = None;
        self.last_batch = None;
        self.last_failed = None;
        result
    }
    fn transport(&mut self) -> Result<(), AudioEditorError> {
        if self.transport.is_none()
            && let Some(client) = &mut self.client
        {
            let s = client
                .status()
                .map_err(|e| AudioEditorError::Backend(format!("{e:?}")))?;
            if !s.freezing
                && self.goal == s.frozen
                && (!self.candidate || !self.goal || s.background_primed)
            {
                self.transport = Some(
                    client
                        .scene_running(self.goal)
                        .map_err(|e| AudioEditorError::Backend(format!("{e:?}")))?,
                );
            }
        }
        Ok(())
    }
}
impl Drop for EditorAudio {
    fn drop(&mut self) {
        let _ = self.close();
    }
}
impl SkinAudioPort for EditorAudio {
    fn retained_drafts(&self) -> Vec<DraftId> {
        self.client
            .as_ref()
            .map_or_else(Vec::new, SkinAudioPort::retained_drafts)
    }
    fn synchronize(
        &mut self,
        p: &SkinProject,
        s: SkinAudioSource,
        c: &[DraftId],
    ) -> Result<(), SkinAudioError> {
        if self.stop_pending || self.close_failed {
            return Err(SkinAudioError::Backend("audio cleanup incomplete".into()));
        }
        if let Some(client) = &mut self.client {
            SkinAudioPort::synchronize(client, p, s, c)?;
        }
        Ok(())
    }
}
impl AudioEditorPort for EditorAudio {
    fn audition_asset(&mut self, request: AssetAudition) -> Result<(), AudioEditorError> {
        if self.stop_pending || self.close_failed {
            return Err(AudioEditorError::Backend("audio cleanup incomplete".into()));
        }
        let result = self
            .client
            .as_mut()
            .ok_or(AudioEditorError::Unavailable)
            .and_then(|c| {
                c.audition_asset(request)
                    .map(|_| ())
                    .map_err(|e| AudioEditorError::Backend(format!("{e:?}")))
            });
        self.error = result.as_ref().err().map(|e| format!("{e:?}"));
        result
    }

    fn request(&mut self, action: AudioEditorAction) -> Result<(), AudioEditorError> {
        self.handle_native_fault();
        let result = (|| {
            if (self.stop_pending || self.close_failed)
                && !matches!(action, AudioEditorAction::Disable)
            {
                return Err(AudioEditorError::Backend(
                    "audio cleanup incomplete; retained output cannot accept new work".into(),
                ));
            }
            match action {
                AudioEditorAction::Enable => {
                    if self.close_failed {
                        return Err(AudioEditorError::Backend("audio cleanup incomplete; replace the stopped output owner before enabling".into()));
                    }
                    if self.client.is_none() {
                        if self.candidate {
                            return self.enable_candidate().map_err(AudioEditorError::Backend);
                        }
                        let native =
                            NativeOutput::open(&self.budget).map_err(AudioEditorError::Backend)?;
                        let (rate, channels) = (native.rate, native.channels);
                        let (stream, output) = native.into_parts();
                        let client = AudioSupplyClient::spawn(
                            output,
                            rate,
                            channels,
                            self.budget.clone(),
                            self.store.clone(),
                        )
                        .map_err(|e| AudioEditorError::Backend(format!("{e:?}")))?;
                        client
                            .output()
                            .map_err(|e| AudioEditorError::Backend(format!("{e:?}")))?
                            .set_monitor_volume(self.monitor_volume)
                            .map_err(AudioEditorError::Backend)?;
                        stream.play().map_err(AudioEditorError::Backend)?;
                        self.client = Some(client);
                        self.stream = Some(stream);
                    }
                }
                AudioEditorAction::Disable => self.close().map_err(AudioEditorError::Backend)?,
                AudioEditorAction::AuditionAsset(_) => return Err(AudioEditorError::MissingAsset),
                AudioEditorAction::MonitorVolume(volume) => {
                    if volume > 1000 {
                        return Err(AudioEditorError::InvalidVolume);
                    }
                    if let Some(native) = &self.native {
                        native
                            .set_monitor_volume(volume)
                            .map_err(AudioEditorError::Backend)?;
                    } else if let Some(client) = &self.client {
                        client
                            .output()
                            .map_err(|e| AudioEditorError::Backend(format!("{e:?}")))?
                            .set_monitor_volume(volume)
                            .map_err(AudioEditorError::Backend)?;
                    }
                    self.monitor_volume = volume;
                }
                AudioEditorAction::SceneRunning(goal) => {
                    self.goal = goal;
                    let client = self.client.as_mut().ok_or(AudioEditorError::Unavailable)?;
                    if self.candidate
                        && goal
                        && !client
                            .status()
                            .map_err(|e| AudioEditorError::Backend(format!("{e:?}")))?
                            .background_primed
                    {
                        return Ok(());
                    }
                    self.transport = Some(
                        client
                            .scene_running(goal)
                            .map_err(|e| AudioEditorError::Backend(format!("{e:?}")))?,
                    );
                }
                AudioEditorAction::Audition(component) => {
                    self.last_trigger_request = Some(
                        self.client
                            .as_mut()
                            .ok_or(AudioEditorError::Unavailable)?
                            .trigger_selected(Channel::Audition, component)
                            .map_err(|e| AudioEditorError::Backend(format!("{e:?}")))?,
                    );
                }
                AudioEditorAction::StopAudition => {
                    self.client
                        .as_mut()
                        .ok_or(AudioEditorError::Unavailable)?
                        .stop_audition()
                        .map_err(|e| AudioEditorError::Backend(format!("{e:?}")))?;
                }
                AudioEditorAction::Retry(component) => {
                    self.client
                        .as_mut()
                        .ok_or(AudioEditorError::Unavailable)?
                        .retry_selected(component)
                        .map_err(|e| AudioEditorError::Backend(format!("{e:?}")))?;
                }
            }
            Ok(())
        })();
        self.error = result.as_ref().err().map(|e| format!("{e:?}"));
        result
    }
    fn poll(&mut self) -> AudioEditorState {
        self.handle_native_fault();
        if self.stop_pending || self.close_failed {
            return AudioEditorState {
                frozen: true,
                monitor_volume: self.monitor_volume,
                error: self
                    .error
                    .clone()
                    .or_else(|| Some("audio cleanup incomplete".into())),
                ..Default::default()
            };
        }
        let Some(client) = &mut self.client else {
            return AudioEditorState {
                monitor_volume: self.monitor_volume,
                error: self.error.clone(),
                frozen: true,
                ..Default::default()
            };
        };
        let receipts: Vec<_> = std::iter::from_fn(|| client.poll()).collect();
        for receipt in receipts {
            self.receive(receipt.id, &receipt.result);
        }
        if let Err(e) = self.transport() {
            self.error = Some(format!("{e:?}"));
        }
        let client = self.client.as_ref().unwrap();
        let s = match client.status() {
            Ok(s) => s,
            Err(e) => {
                return AudioEditorState {
                    enabled: true,
                    error: Some(format!("{e:?}")),
                    ..Default::default()
                };
            }
        };
        let native = self.native.as_ref().map(NativeExecutor::stats);
        // The normal GUI drains observations even when no benchmark is attached.
        if let Some(native) = &mut self.native {
            while native.next_first().is_some() {}
        }
        let (nonzero, observation, device_error) = if self.candidate {
            let n = native.unwrap_or_default();
            (
                n.nonzero_samples,
                Some(AudioOutputObservation {
                    candidate: true,
                    rate: self.format.0,
                    channels: self.format.1,
                    callbacks: n.callbacks,
                    missing_frames: n.missing_frames,
                    generated_background: s.generated_background,
                    copied_background: s.copied_background,
                    background_frames: self.background_frames,
                    pcm_bytes: self.budget.usage().bytes,
                }),
                (n.device_fault || n.render_errors > 0 || n.observation_overflows > 0)
                    .then(|| format!("native output: {n:?}")),
            )
        } else {
            match client.output() {
                Ok(o) => (o.stats().nonzero_samples, None, None),
                Err(e) => (0, None, Some(format!("{e:?}"))),
            }
        };
        AudioEditorState {
            submitted_nonzero_samples: nonzero,
            output: observation,
            monitor_volume: self.monitor_volume,
            asset_audition: s.library.map(|(asset, s)| AssetAuditionState {
                asset,
                source: source(s.desired),
                preparation: preparation(s.preparation),
            }),
            enabled: true,
            source: s.origin.map(source),
            components: s
                .bindings
                .into_iter()
                .map(|(component, s)| AudioComponentState {
                    component,
                    desired: source(s.desired),
                    effective: s.effective.map(source),
                    preparation: preparation(s.preparation),
                })
                .collect(),
            frozen: s.frozen,
            freezing: s.freezing || !self.goal && !s.frozen,
            pending_controls: client.pending_controls() || self.transport.is_some(),
            voices: s.voices,
            last_audition: self
                .last_audition
                .map(|(origin, old)| (origin, old || Some(origin) != s.origin.map(source))),
            error: s.fault.or(device_error).or_else(|| self.error.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn failed_cleanup_cannot_be_cleared_by_disable_then_reenable() {
        let mut a = EditorAudio::candidate(BlobStore::default());
        a.close_failed = true;
        for disable in [false, true] {
            if disable {
                a.request(AudioEditorAction::Disable).unwrap();
            }
            assert!(
                matches!(a.request(AudioEditorAction::Enable),Err(AudioEditorError::Backend(e)) if e.contains("cleanup incomplete"))
            );
            assert!(a.client.is_none() && a.native.is_none());
            assert_eq!(a.budget.usage().bytes, 0);
            assert!(a.poll().frozen);
        }
    }
    use skin_editor_infrastructure::{OutputConsumer, WavDecoder, output_boundary};
    use skin_editor_ui::EditorUi;
    use std::{cell::RefCell, rc::Rc, time::Duration};
    struct Shared(Rc<RefCell<EditorAudio>>);
    #[test]
    fn transient_transport_busy_keeps_resume_goal_retryable_without_poisoning_editor_error() {
        let mut adapter = EditorAudio::candidate(BlobStore::default());
        adapter.goal = true;
        adapter.transport = Some(7);
        adapter.receive(7, &Err(SupplyError::Busy));
        assert!(adapter.goal);
        assert!(adapter.transport.is_none());
        assert!(adapter.error.is_none());
        // A full sound-admission window still produces an explicit action error.
        adapter.receive(8, &Err(SupplyError::Busy));
        assert_eq!(adapter.last_failed, Some((8, SupplyError::Busy)));
        assert!(adapter.error.is_some());
    }
    #[test]
    fn drained_receipts_keep_bounded_request_identity_for_gui_observation() {
        let mut a = EditorAudio::candidate(BlobStore::default());
        let origin = Origin {
            session: 1,
            node: 1,
            revision: 0,
            navigation_revision: 0,
            draft: None,
        };
        let trigger = TriggerReceipt {
            origin,
            stale: false,
            voice: Some(7),
            queued: false,
            silent: false,
        };
        a.receive(
            2,
            &Ok(SupplyOutcome::BatchTriggered {
                mix_frame: 0,
                results: vec![Ok(trigger)],
            }),
        );
        assert_eq!(a.last_batch, Some((2, Ok(vec![7]))));
        a.receive(3, &Err(SupplyError::Backend("ResourceLimit".into())));
        assert_eq!(
            a.last_failed,
            Some((3, SupplyError::Backend("ResourceLimit".into())))
        );
        a.receive(4, &Ok(SupplyOutcome::Triggered(trigger)));
        assert_eq!(a.last_triggered, Some((4, trigger)));
        assert_eq!(a.last_batch, Some((2, Ok(vec![7]))));
        a.close().unwrap();
        assert!(a.last_triggered.is_none() && a.last_batch.is_none() && a.last_failed.is_none());
    }
    #[test]
    fn candidate_without_song_is_explicit_eof_and_only_reserves_one_silent_frame() {
        let mut a = EditorAudio::candidate(BlobStore::default());
        let mut track = a.take_reference(48000, 2).unwrap();
        assert_eq!(a.background_frames, 0);
        assert_eq!((track.frames(), track.position()), (1, 1));
        track.set_running(true);
        assert!(track.next_frame().is_none());
        assert_eq!(a.budget.usage().bytes, 8);
        drop(track);
        assert_eq!(a.budget.usage().bytes, 0);
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
        fn audition_asset(&mut self, a: AssetAudition) -> Result<(), AudioEditorError> {
            self.0.borrow_mut().audition_asset(a)
        }
        fn request(&mut self, a: AudioEditorAction) -> Result<(), AudioEditorError> {
            self.0.borrow_mut().request(a)
        }
        fn poll(&mut self) -> AudioEditorState {
            self.0.borrow_mut().poll()
        }
    }
    fn barrier(shared: &Rc<RefCell<EditorAudio>>) {
        let mut adapter = shared.borrow_mut();
        let id = adapter.client.as_mut().unwrap().barrier().unwrap();
        loop {
            let r = adapter
                .client
                .as_mut()
                .unwrap()
                .wait_receipt(Duration::from_secs(5))
                .unwrap()
                .unwrap();
            if r.id == id {
                r.result.as_ref().unwrap();
            }
            adapter.receive(r.id, &r.result);
            if r.id == id {
                break;
            }
        }
    }
    fn settle(ui: &mut EditorUi, session: &ProjectSession, shared: &Rc<RefCell<EditorAudio>>) {
        for _ in 0..256 {
            barrier(shared);
            ui.refresh_audio(session);
            if ui.audio_state.source == Some(session.skin_audio_source())
                && ui
                    .audio_state
                    .asset_audition
                    .as_ref()
                    .is_none_or(|s| s.preparation != AudioPreparationState::Pending)
                && ui
                    .audio_state
                    .components
                    .iter()
                    .all(|s| s.preparation != AudioPreparationState::Pending)
            {
                return;
            }
        }
        panic!("preparation did not complete");
    }
    fn sound(store: &BlobStore, value: i16) -> skin_editor_infrastructure::RawBlob {
        let mut bytes = crate::audio_verification::tone(48000, 1, 440.0);
        for s in bytes[44..].chunks_exact_mut(2) {
            s.copy_from_slice(&value.to_le_bytes());
        }
        store.stage_audio(bytes, &WavDecoder).unwrap()
    }
    fn flush(callback: &mut OutputConsumer) {
        callback.render_f32(&mut [0.0; 2048]);
    }
    #[test]
    fn opaque_preparation_failure_is_visible_and_retry_never_autoplays_or_changes_history() {
        let store = BlobStore::default();
        let raw = store
            .stage_bytes(b"self authored unknown codec".to_vec())
            .unwrap();
        let budget = PcmBudget::default();
        let (output, mut callback) = output_boundary(&budget, 48000, 1).unwrap();
        let mut adapter = EditorAudio::new(store.clone());
        adapter.budget = budget.clone();
        adapter.client = Some(
            AudioSupplyClient::spawn(output, 48000, 1, budget.clone(), store.clone()).unwrap(),
        );
        let shared = Rc::new(RefCell::new(adapter));
        let mut ui = EditorUi::default();
        ui.set_audio_port(Box::new(Shared(shared.clone())));
        let mut session =
            ProjectSession::with_resources(1, 2, Box::new(store.clone()), SessionLimits::default())
                .unwrap();
        let asset = AssetId::new(ui.object_id()).unwrap();
        ui.submit(
            &mut session,
            vec![Command::RegisterAsset {
                asset: Asset::new(
                    asset,
                    "unknown.ogg".into(),
                    raw.reference(),
                    AssetKind::Opaque,
                )
                .unwrap(),
            }],
        );
        let component = ui
            .create_audio_component(&mut session, asset, "unknown".into())
            .unwrap();
        settle(&mut ui, &session, &shared);
        assert!(
            matches!(&ui.audio_state.components[0].preparation,AudioPreparationState::Failed(e) if e.contains("Unsupported"))
        );
        let snapshot = session.snapshot();
        let nodes = session.history_len();
        ui.audio_command(&session, AudioEditorAction::Audition(component));
        barrier(&shared);
        ui.refresh_audio(&session);
        assert!(
            ui.audio_state
                .error
                .as_ref()
                .is_some_and(|s| s.contains("NotReady"))
        );
        ui.audio_command(&session, AudioEditorAction::AuditionAsset(asset));
        settle(&mut ui, &session, &shared);
        assert!(
            matches!(&ui.audio_state.asset_audition.as_ref().unwrap().preparation, AudioPreparationState::Failed(e) if e.contains("Unsupported"))
        );
        ui.audio_command(&session, AudioEditorAction::StopAudition);
        barrier(&shared);
        ui.audio_command(&session, AudioEditorAction::Retry(component));
        settle(&mut ui, &session, &shared);
        let mut samples = [1.0; 2048];
        callback.render_f32(&mut samples);
        assert!(samples.iter().all(|v| *v == 0.0));
        assert_eq!(ui.audio_state.last_audition, None);
        assert_eq!(session.snapshot(), snapshot);
        assert_eq!(session.history_len(), nodes);
        assert_eq!(store.get(raw.reference()).unwrap().bytes(), raw.bytes());
        ui.audio_command(&session, AudioEditorAction::Disable);
        drop(ui);
        drop(shared);
        drop(callback);
        assert_eq!(budget.usage().bytes, 0);
    }
    #[test]
    fn ui_port_real_wav_hot_replace_cancel_and_history_invalidate_without_ui_frame() {
        let store = BlobStore::default();
        let budget = PcmBudget::default();
        let a = sound(&store, 4096);
        let b = sound(&store, 8192);
        let mut session =
            ProjectSession::with_resources(1, 2, Box::new(store.clone()), SessionLimits::default())
                .unwrap();
        let (output, mut callback) = output_boundary(&budget, 48000, 1).unwrap();
        let mut adapter = EditorAudio::new(store.clone());
        adapter.budget = budget.clone();
        adapter.client =
            Some(AudioSupplyClient::spawn(output, 48000, 1, budget.clone(), store).unwrap());
        let shared = Rc::new(RefCell::new(adapter));
        let mut ui = EditorUi::default();
        ui.set_audio_port(Box::new(Shared(shared.clone())));
        let aid = AssetId::new(ui.object_id()).unwrap();
        let bid = AssetId::new(ui.object_id()).unwrap();
        for (id, raw) in [(aid, &a), (bid, &b)] {
            assert!(ui.submit(
                &mut session,
                vec![Command::RegisterAsset {
                        asset: Asset::new(
                            id,
                            "self.wav".into(),
                            raw.reference(),
                            AssetKind::Audio(raw.audio_metadata().unwrap())
                        )
                        .unwrap()
                    }]
            ));
        }
        let component = ui
            .create_audio_component(&mut session, aid, "self".into())
            .unwrap();
        settle(&mut ui, &session, &shared);
        assert!(ui.paused && ui.audio_state.frozen);
        flush(&mut callback);
        let base = session.snapshot();
        let nodes = session.history_len();
        ui.audio_command(&session, AudioEditorAction::Audition(component));
        barrier(&shared);
        let mut out = [0.0; 768];
        callback.render_f32(&mut out);
        assert!(out.contains(&0.125));
        ui.refresh_audio(&session);
        assert!(ui.audio_state.submitted_nonzero_samples > 0);
        assert_eq!(session.snapshot(), base);
        assert_eq!(session.history_len(), nodes);
        ui.audio_command(&session, AudioEditorAction::StopAudition);
        barrier(&shared);
        flush(&mut callback);
        ui.submit(
            &mut session,
            vec![Command::SetAudioAsset {
                component,
                asset: bid,
            }],
        );
        settle(&mut ui, &session, &shared);
        flush(&mut callback);
        ui.audio_command(&session, AudioEditorAction::Audition(component));
        barrier(&shared);
        callback.render_f32(&mut out);
        assert!(out.contains(&0.25));
        ui.audio_command(&session, AudioEditorAction::StopAudition);
        barrier(&shared);
        flush(&mut callback);
        let base = session.snapshot();
        let nodes = session.history_len();
        ui.draft(
            &mut session,
            DraftAction::Begin {
                expected: base.state,
            },
        );
        let expected = session.skin_audio_source().draft.unwrap();
        ui.draft(
            &mut session,
            DraftAction::Replace {
                expected,
                commands: vec![Command::SetAudioProperties {
                    component,
                    gain: Gain::new(0.5).unwrap(),
                    visible: true,
                }],
            },
        );
        settle(&mut ui, &session, &shared);
        flush(&mut callback);
        ui.audio_command(&session, AudioEditorAction::Audition(component));
        barrier(&shared);
        callback.render_f32(&mut out[..128]);
        assert!(out[..128].contains(&0.125));
        ui.cancel(&mut session);
        callback.render_f32(&mut out[..128]);
        assert_eq!(out[..128], [0.0; 128]);
        assert_eq!(session.snapshot(), base);
        assert_eq!(session.history_len(), nodes);
        settle(&mut ui, &session, &shared);
        flush(&mut callback);
        ui.audio_command(&session, AudioEditorAction::Audition(component));
        barrier(&shared);
        callback.render_f32(&mut out[..128]);
        assert!(out[..128].contains(&0.25));
        ui.navigation(&mut session, HistoryAction::Undo);
        callback.render_f32(&mut out[..128]);
        assert_eq!(out[..128], [0.0; 128]);
        settle(&mut ui, &session, &shared);
        assert_eq!(ui.audio_state.source, Some(session.skin_audio_source()));
        ui.audio_command(&session, AudioEditorAction::SceneRunning(true));
        ui.audio_command(&session, AudioEditorAction::SceneRunning(false));
        flush(&mut callback);
        barrier(&shared);
        ui.refresh_audio(&session);
        assert!(ui.audio_state.frozen);
        ui.audio_command(&session, AudioEditorAction::Disable);
        drop(ui);
        drop(shared);
        drop(callback);
        assert_eq!(budget.usage().bytes, 0);
    }
    #[test]
    fn library_audition_without_component_is_frozen_readonly_and_cleaned_outside_ui_frames() {
        let store = BlobStore::default();
        let raw = sound(&store, 8192);
        let budget = PcmBudget::default();
        let (output, mut callback) = output_boundary(&budget, 48000, 1).unwrap();
        let mut adapter = EditorAudio::new(store.clone());
        adapter.budget = budget.clone();
        adapter.client = Some(
            AudioSupplyClient::spawn(output, 48000, 1, budget.clone(), store.clone()).unwrap(),
        );
        let shared = Rc::new(RefCell::new(adapter));
        let mut ui = EditorUi::default();
        ui.set_audio_port(Box::new(Shared(shared.clone())));
        let mut session =
            ProjectSession::with_resources(1, 2, Box::new(store.clone()), SessionLimits::default())
                .unwrap();
        let asset = AssetId::new(ui.object_id()).unwrap();
        assert!(ui.submit(
            &mut session,
            vec![Command::RegisterAsset {
                    asset: Asset::new(
                        asset,
                        "library.wav".into(),
                        raw.reference(),
                        AssetKind::Audio(raw.audio_metadata().unwrap())
                    )
                    .unwrap()
                }]
        ));
        settle(&mut ui, &session, &shared);
        let snapshot = session.snapshot();
        let nodes = session.history_len();
        assert_eq!(session.snapshot().project.audio_components().len(), 0);
        ui.audio_command(&session, AudioEditorAction::MonitorVolume(500));
        ui.audio_command(&session, AudioEditorAction::AuditionAsset(asset));
        settle(&mut ui, &session, &shared);
        assert_eq!(ui.audio_state.asset_audition.as_ref().unwrap().asset, asset);
        assert_eq!(
            ui.audio_state.asset_audition.as_ref().unwrap().preparation,
            AudioPreparationState::Ready
        );
        assert!(ui.audio_state.frozen);
        assert!(ui.audio_state.error.is_none());
        flush(&mut callback);
        barrier(&shared);
        let mut samples = [0.0; 128];
        callback.render_f32(&mut samples);
        assert_eq!(samples, [0.125; 128]);
        ui.audio_command(&session, AudioEditorAction::StopAudition);
        samples.fill(1.0);
        callback.render_f32(&mut samples);
        assert_eq!(samples, [0.0; 128]);
        settle(&mut ui, &session, &shared);
        assert!(ui.audio_state.asset_audition.is_none());
        assert_eq!(ui.audio_state.voices, [0, 0]);
        assert_eq!(session.snapshot(), snapshot);
        assert_eq!(session.history_len(), nodes);
        // A source change cancels the explicit library intent; no later frame may resurrect it.
        ui.audio_command(&session, AudioEditorAction::AuditionAsset(asset));
        settle(&mut ui, &session, &shared);
        let expected = session.snapshot().state;
        ui.draft(&mut session, DraftAction::Begin { expected });
        samples.fill(1.0);
        callback.render_f32(&mut samples);
        assert_eq!(samples, [0.0; 128]);
        ui.cancel(&mut session);
        settle(&mut ui, &session, &shared);
        assert!(ui.audio_state.asset_audition.is_none());
        ui.audio_command(&session, AudioEditorAction::AuditionAsset(asset));
        settle(&mut ui, &session, &shared);
        ui.navigation(&mut session, HistoryAction::Undo);
        samples.fill(1.0);
        callback.render_f32(&mut samples);
        assert_eq!(samples, [0.0; 128]);
        settle(&mut ui, &session, &shared);
        assert!(ui.audio_state.asset_audition.is_none());
        ui.audio_command(&session, AudioEditorAction::Disable);
        drop(callback);
        drop(shared);
        drop(ui);
        assert_eq!(budget.usage().bytes, 0);
    }
    struct CandidateRig {
        ui: EditorUi,
        session: ProjectSession,
        shared: Rc<RefCell<EditorAudio>>,
        callback: skin_editor_infrastructure::separated_output::OutputConsumer,
        budget: PcmBudget,
        component: AudioComponentId,
    }
    impl CandidateRig {
        fn new() -> Self {
            let store = BlobStore::default();
            let raw = sound(&store, 8192);
            let budget = PcmBudget::default();
            let mut track =
                BackgroundTrack::new(budget.reserve(1).unwrap().allocate(), 48000, 1).unwrap();
            track.restore_position(1).unwrap();
            let (client, callback) =
                AudioSupplyClient::spawn_separated(track, 48000, 1, budget.clone(), store.clone())
                    .unwrap();
            let mut adapter = EditorAudio::candidate(store.clone());
            adapter.budget = budget.clone();
            adapter.client = Some(client);
            adapter.format = (48000, 1);
            let shared = Rc::new(RefCell::new(adapter));
            let mut ui = EditorUi::default();
            ui.set_audio_port(Box::new(Shared(shared.clone())));
            let mut session =
                ProjectSession::with_resources(1, 2, Box::new(store), SessionLimits::default())
                    .unwrap();
            let asset = AssetId::new(ui.object_id()).unwrap();
            assert!(ui.submit(
                &mut session,
                vec![Command::RegisterAsset {
                        asset: Asset::new(
                            asset,
                            "self.wav".into(),
                            raw.reference(),
                            AssetKind::Audio(raw.audio_metadata().unwrap())
                        )
                        .unwrap()
                    }]
            ));
            let component = ui
                .create_audio_component(&mut session, asset, "self".into())
                .unwrap();
            Self {
                ui,
                session,
                shared,
                callback: *callback,
                budget,
                component,
            }
        }
        // Real worker with a manually advanced software endpoint. Completion
        // predicates, receipts and explicit frames determine progress; no sleeps.
        fn drive(&mut self, done: impl Fn(&Self) -> bool) {
            let until = std::time::Instant::now() + Duration::from_secs(5);
            loop {
                self.callback.render(&mut [0.0; 128]).unwrap();
                {
                    let mut a = self.shared.borrow_mut();
                    if let Some(r) = a
                        .client
                        .as_mut()
                        .unwrap()
                        .wait_receipt(Duration::from_millis(2))
                        .unwrap()
                    {
                        r.result.as_ref().unwrap();
                        a.receive(r.id, &r.result);
                    }
                }
                self.ui.refresh_audio(&self.session);
                if done(self) {
                    break;
                }
                assert!(
                    std::time::Instant::now() < until,
                    "candidate progress: {:?}",
                    self.ui.audio_state
                );
            }
            assert!(
                self.ui.audio_state.error.is_none(),
                "{:?}",
                self.ui.audio_state.error
            );
        }
        fn ready(&mut self) {
            self.drive(|r| {
                r.ui.audio_state.source == Some(r.session.skin_audio_source())
                    && r.ui.audio_state.components.len() == 1
                    && r.ui.audio_state.components[0].preparation == AudioPreparationState::Ready
                    && !r.ui.audio_state.pending_controls
            });
        }
    }
    #[test]
    fn candidate_gui_observes_preparation_freeze_and_independent_audition_without_native_device() {
        let mut r = CandidateRig::new();
        r.ready();
        let snapshot = r.session.snapshot();
        let nodes = r.session.history_len();
        let observation = r.ui.audio_state.output.as_ref().unwrap();
        assert!(observation.candidate);
        assert_eq!(
            (
                observation.rate,
                observation.channels,
                observation.background_frames
            ),
            (48000, 1, 0)
        );
        r.ui.audio_command(&r.session, AudioEditorAction::SceneRunning(true));
        r.drive(|r| !r.ui.audio_state.frozen && !r.ui.audio_state.pending_controls);
        r.ui.audio_command(&r.session, AudioEditorAction::Audition(r.component));
        r.drive(|r| r.ui.audio_state.voices == [0, 1]);
        r.ui.audio_command(&r.session, AudioEditorAction::SceneRunning(false));
        r.drive(|r| {
            r.ui.audio_state.frozen
                && !r.ui.audio_state.freezing
                && !r.ui.audio_state.pending_controls
        });
        assert_eq!(r.ui.audio_state.voices, [0, 1]);
        let mut samples = [0.0; 128];
        r.callback.render(&mut samples).unwrap();
        assert_eq!(samples, [0.25; 128]);
        r.ui.audio_command(&r.session, AudioEditorAction::StopAudition);
        samples.fill(1.0);
        r.callback.render(&mut samples).unwrap();
        assert_eq!(samples, [0.0; 128]);
        r.drive(|r| r.ui.audio_state.voices == [0, 0] && !r.ui.audio_state.pending_controls);
        assert_eq!(r.session.snapshot(), snapshot);
        assert_eq!(r.session.history_len(), nodes);
        let pool = r.budget.clone();
        r.ui.audio_command(&r.session, AudioEditorAction::Disable);
        assert!(!r.ui.audio_state.enabled);
        drop(r);
        assert_eq!(pool.usage().bytes, 0);
    }
    #[test]
    fn candidate_gui_cancel_and_navigation_reject_old_samples_and_label_late_observation() {
        let mut r = CandidateRig::new();
        r.ready();
        let snapshot = r.session.snapshot();
        let nodes = r.session.history_len();
        r.ui.draft(
            &mut r.session,
            DraftAction::Begin {
                expected: snapshot.state,
            },
        );
        let expected = r
            .session
            .draft_info(r.session.project_info().session_id)
            .unwrap()
            .unwrap()
            .state;
        r.ui.draft(
            &mut r.session,
            DraftAction::Replace {
                expected,
                commands: vec![Command::SetAudioProperties {
                    component: r.component,
                    gain: Gain::new(0.5).unwrap(),
                    visible: true,
                }],
            },
        );
        r.ready();
        r.ui.audio_command(&r.session, AudioEditorAction::Audition(r.component));
        r.drive(|r| r.ui.audio_state.voices == [0, 1]);
        let late = r.shared.borrow().last_triggered.unwrap();
        r.ui.cancel(&mut r.session);
        let mut samples = [1.0; 128];
        r.callback.render(&mut samples).unwrap();
        assert_eq!(samples, [0.0; 128]);
        r.ready();
        r.drive(|r| {
            r.ui.audio_state.voices == [0, 0] && r.shared.borrow().retained_drafts().is_empty()
        });
        assert_eq!(r.session.snapshot(), snapshot);
        assert_eq!(r.session.history_len(), nodes);
        r.shared
            .borrow_mut()
            .receive(late.0, &Ok(SupplyOutcome::Triggered(late.1)));
        r.ui.refresh_audio(&r.session);
        assert_eq!(
            r.ui.audio_state.last_audition,
            Some((source(late.1.origin), true))
        );
        assert!(r.ui.submit(
            &mut r.session,
            vec![Command::SetAudioProperties {
                component: r.component,
                gain: Gain::new(0.5).unwrap(),
                visible: true
            }]
        ));
        r.ready();
        r.ui.audio_command(&r.session, AudioEditorAction::Audition(r.component));
        r.drive(|r| r.ui.audio_state.voices == [0, 1]);
        r.ui.navigation(&mut r.session, HistoryAction::Undo);
        samples.fill(1.0);
        r.callback.render(&mut samples).unwrap();
        assert_eq!(samples, [0.0; 128]);
        r.ready();
        assert_eq!(r.ui.audio_state.voices, [0, 0]);
        assert!(r.ui.audio_state.last_audition.unwrap().1);
        r.ui.navigation(&mut r.session, HistoryAction::Redo { child: None });
        r.ready();
        assert_eq!(r.ui.audio_state.voices, [0, 0]);
        let pool = r.budget.clone();
        drop(r);
        assert_eq!(pool.usage().bytes, 0);
    }

    #[test]
    fn candidate_gui_live_gain_cancel_and_binding_replacement_use_the_existing_worker() {
        let mut r = CandidateRig::new();
        r.ready();
        let snapshot = r.session.snapshot();
        let nodes = r.session.history_len();
        r.ui.audio_command(&r.session, AudioEditorAction::Audition(r.component));
        barrier(&r.shared);
        let mut samples = [0.; 128];
        r.callback.render(&mut samples).unwrap();
        assert_eq!(samples, [0.25; 128]);
        assert!(r.ui.draft(
            &mut r.session,
            DraftAction::Begin {
                expected: snapshot.state
            }
        ));
        for gain in [0., 0.5, 1.] {
            let expected = r
                .session
                .draft_info(r.session.project_info().session_id)
                .unwrap()
                .unwrap()
                .state;
            assert!(r.ui.draft(
                &mut r.session,
                DraftAction::Replace {
                    expected,
                    commands: vec![Command::SetAudioProperties {
                        component: r.component,
                        gain: Gain::new(gain).unwrap(),
                        visible: true
                    }]
                }
            ));
            barrier(&r.shared);
            r.callback.render(&mut samples).unwrap();
            assert_eq!(samples, [0.25 * gain as f32; 128]);
        }
        r.ui.cancel(&mut r.session);
        barrier(&r.shared);
        r.callback.render(&mut samples).unwrap();
        assert_eq!(samples, [0.25; 128]);
        assert_eq!(r.session.snapshot(), snapshot);
        assert_eq!(r.session.history_len(), nodes);
        let store = r.shared.borrow().store.clone();
        let raw = sound(&store, 16384);
        let asset = AssetId::new(r.ui.object_id()).unwrap();
        assert!(r.ui.submit(
            &mut r.session,
            vec![Command::RegisterAsset {
                    asset: Asset::new(
                        asset,
                        "replacement.wav".into(),
                        raw.reference(),
                        AssetKind::Audio(raw.audio_metadata().unwrap())
                    )
                    .unwrap()
                }]
        ));
        barrier(&r.shared);
        assert!(r.ui.submit(
            &mut r.session,
            vec![Command::SetAudioAsset {
                component: r.component,
                asset
            }]
        ));
        // The client fence must reject old data before worker/UI progress.
        r.callback.render(&mut samples).unwrap();
        assert_eq!(samples, [0.; 128]);
        r.ready();
        assert_eq!(r.ui.audio_state.voices, [0, 0]);
        r.ui.audio_command(&r.session, AudioEditorAction::Audition(r.component));
        barrier(&r.shared);
        r.callback.render(&mut samples).unwrap();
        assert_eq!(samples, [0.5; 128]);
        let pool = r.budget.clone();
        drop(r);
        drop(raw);
        assert_eq!(pool.usage().bytes, 0);
    }
}
