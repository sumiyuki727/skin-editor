use super::*;
mod gesture_controls;
mod history_ui;
mod input;
mod interaction;
mod joint_selection;
use std::sync::Arc;
use std::sync::Mutex;
struct Lease(BlobRef);
impl BlobLease for Lease {
    fn reference(&self) -> BlobRef {
        self.0
    }
}
struct Store;

#[derive(Default)]
struct AudioTrace {
    sources: Vec<SkinAudioSource>,
    cancelled: Vec<DraftId>,
    actions: Vec<AudioEditorAction>,
    fail_once: bool,
    fail_enable: bool,
    device_fault: bool,
}
struct AudioFake(Arc<Mutex<AudioTrace>>);
impl SkinAudioPort for AudioFake {
    fn retained_drafts(&self) -> Vec<DraftId> {
        self.0
            .lock()
            .unwrap()
            .sources
            .last()
            .and_then(|s| s.draft)
            .map(|d| vec![d.id])
            .unwrap_or_default()
    }
    fn synchronize(
        &mut self,
        _: &SkinProject,
        s: SkinAudioSource,
        c: &[DraftId],
    ) -> Result<(), SkinAudioError> {
        let mut t = self.0.lock().unwrap();
        if t.fail_once {
            t.fail_once = false;
            return Err(SkinAudioError::ResourceLimit);
        }
        t.sources.push(s);
        t.cancelled.extend(c);
        Ok(())
    }
}
impl AudioEditorPort for AudioFake {
    fn request(&mut self, a: AudioEditorAction) -> Result<(), AudioEditorError> {
        let mut trace = self.0.lock().unwrap();
        trace.actions.push(a);
        if a == AudioEditorAction::Enable && trace.fail_enable {
            return Err(AudioEditorError::Backend("test no output".into()));
        }
        Ok(())
    }
    fn poll(&mut self) -> AudioEditorState {
        if self.0.lock().unwrap().device_fault {
            return AudioEditorState {
                frozen: true,
                error: Some("device stopped".into()),
                ..Default::default()
            };
        }
        AudioEditorState {
            enabled: true,
            source: self.0.lock().unwrap().sources.last().copied(),
            ..Default::default()
        }
    }
}
#[test]
fn failed_device_freezes_preview_and_reenable_does_not_inherit_old_play_intent() {
    let (mut ui, session, trace, _) = audio_fixture();
    let before = session.snapshot();
    let nodes = session.history_len();
    ui.paused = false;
    trace.lock().unwrap().device_fault = true;
    ui.refresh_audio(&session);
    assert!(ui.paused && ui.scene_is_frozen());
    trace.lock().unwrap().device_fault = false;
    ui.audio_command(&session, AudioEditorAction::Enable);
    assert_eq!(
        trace.lock().unwrap().actions,
        vec![
            AudioEditorAction::Enable,
            AudioEditorAction::SceneRunning(false)
        ]
    );
    assert_eq!(session.snapshot(), before);
    assert_eq!(session.history_len(), nodes);
}
fn audio_fixture() -> (
    EditorUi,
    ProjectSession,
    Arc<Mutex<AudioTrace>>,
    AudioComponentId,
) {
    let mut ui = EditorUi::default();
    let trace = Arc::new(Mutex::new(AudioTrace::default()));
    ui.set_audio_port(Box::new(AudioFake(trace.clone())));
    let mut session =
        ProjectSession::with_resources(1, 2, Box::new(Store), SessionLimits::default()).unwrap();
    let asset = AssetId::new(ui.object_id()).unwrap();
    assert!(ui.submit(
        &mut session,
        vec![Command::RegisterAsset {
                asset: Asset::new(
                    asset,
                    "original.ogg".into(),
                    BlobRef::new([7; 32], 9),
                    AssetKind::Opaque
                )
                .unwrap()
            }]
    ));
    let id = ui
        .create_audio_component(&mut session, asset, "self.test".into())
        .unwrap();
    (ui, session, trace, id)
}
#[test]
fn audio_edit_cancel_and_history_synchronize_before_return_without_a_gui_pass() {
    let (mut ui, mut session, trace, id) = audio_fixture();
    let base = session.snapshot();
    let nodes = session.history_len();
    ui.draft(
        &mut session,
        DraftAction::Begin {
            expected: base.state,
        },
    );
    let d = EditorUi::draft_state(&session).unwrap();
    ui.draft(
        &mut session,
        DraftAction::Replace {
            expected: d,
            commands: vec![Command::SetAudioProperties {
                component: id,
                gain: Gain::new(0.5).unwrap(),
                visible: true,
            }],
        },
    );
    assert_eq!(
        trace.lock().unwrap().sources.last(),
        Some(&session.skin_audio_source())
    );
    ui.cancel(&mut session);
    assert_eq!(session.snapshot(), base);
    assert_eq!(session.history_len(), nodes);
    trace.lock().unwrap().fail_enable = true;
    ui.audio_command(&session, AudioEditorAction::Enable);
    assert!(ui.message.contains("test no output"));
    assert_eq!(
        trace.lock().unwrap().actions.last(),
        Some(&AudioEditorAction::Enable)
    );
    assert!(trace.lock().unwrap().cancelled.contains(&d.id));
    ui.submit(
        &mut session,
        vec![Command::SetAudioProperties {
            component: id,
            gain: Gain::new(0.25).unwrap(),
            visible: true,
        }],
    );
    ui.navigation(&mut session, HistoryAction::Undo);
    assert_eq!(
        trace.lock().unwrap().sources.last(),
        Some(&session.skin_audio_source())
    );
    assert_eq!(session.snapshot().project, base.project);
}
#[test]
fn busy_audio_sync_retries_without_rolling_back_a_valid_project_or_replaying_audition() {
    let (mut ui, mut session, trace, id) = audio_fixture();
    trace.lock().unwrap().fail_once = true;
    assert!(ui.submit(
        &mut session,
        vec![Command::SetAudioProperties {
            component: id,
            gain: Gain::new(0.5).unwrap(),
            visible: true
        }]
    ));
    assert!(ui.audio_needs_poll());
    ui.refresh_audio(&session);
    assert!(!ui.audio_sync_pending);
    ui.paused = false;
    ui.audio_state.frozen = true;
    assert!(ui.scene_is_frozen());
    ui.audio_state.frozen = false;
    ui.audio_state.freezing = true;
    assert!(ui.scene_is_frozen());
    ui.audio_state.freezing = false;
    assert!(!ui.scene_is_frozen());
    let base = session.snapshot();
    let nodes = session.history_len();
    ui.audio_command(&session, AudioEditorAction::Audition(id));
    ui.refresh_audio(&session);
    assert_eq!(
        trace.lock().unwrap().actions,
        vec![AudioEditorAction::Audition(id)]
    );
    assert_eq!(session.snapshot(), base);
    assert_eq!(session.history_len(), nodes);
}
#[test]
fn three_hundred_audio_draft_updates_commit_once_and_duplicate_slot_failure_is_atomic() {
    let (mut ui, mut session, _, id) = audio_fixture();
    let n = session.history_len();
    let state = session.project_info().state;
    ui.draft(&mut session, DraftAction::Begin { expected: state });
    for i in 1..=300 {
        let expected = EditorUi::draft_state(&session).unwrap();
        assert!(ui.draft(
            &mut session,
            DraftAction::Replace {
                expected,
                commands: vec![Command::SetAudioProperties {
                    component: id,
                    gain: Gain::new(i as f64 / 600.0).unwrap(),
                    visible: true
                }]
            }
        ));
    }
    ui.commit(&mut session);
    assert_eq!(session.history_len(), n + 1);
    let base = session.snapshot();
    let asset = base.project.audio_component(id).unwrap().asset();
    assert!(
        ui.create_audio_component(&mut session, asset, "self.test".into())
            .is_none()
    );
    assert_eq!(session.snapshot(), base);
}
impl ResourceStore for Store {
    fn pin(&self, blob: BlobRef) -> Result<Arc<dyn BlobLease>, ResourceError> {
        Ok(Arc::new(Lease(blob)))
    }
}
#[derive(Default)]
struct Fake(Option<PresentedView>);
impl Presenter for Fake {
    fn current(&self) -> Option<PresentedView> {
        self.0
    }
    fn paint(&mut self, _: &mut egui::Ui, _: FrameKey, _: Rect) {}
    fn thumbnail(&mut self, _: &mut egui::Ui, _: BlobRef) {}
}
struct Harness {
    ctx: egui::Context,
    ui: EditorUi,
    session: ProjectSession,
    presenter: Fake,
    modifiers: egui::Modifiers,
    labels: Vec<(String, Rect)>,
}
impl Harness {
    fn new() -> Self {
        let mut session =
            ProjectSession::with_resources(1, 2, Box::new(Store), SessionLimits::default())
                .unwrap();
        let mut ui = EditorUi::default();
        let asset = AssetId::new(ui.object_id()).unwrap();
        assert!(ui.submit(
            &mut session,
            vec![Command::RegisterAsset {
                    asset: Asset::new(
                        asset,
                        "test".into(),
                        BlobRef::new([1; 32], 4),
                        AssetKind::RasterImage(PixelSize::new(64, 64).unwrap())
                    )
                    .unwrap()
                }]
        ));
        ui.add_asset_layer(&mut session, asset);
        let mut h = Self {
            ctx: egui::Context::default(),
            ui,
            session,
            presenter: Fake::default(),
            modifiers: Default::default(),
            labels: Vec::new(),
        };
        for _ in 0..3 {
            h.pass(vec![], [1180., 800.], true);
        }
        h
    }
    fn pass(&mut self, events: Vec<egui::Event>, size: [f32; 2], focused: bool) -> UiOutput {
        self.pass_recorded(events, size, focused, None)
    }
    fn pass_recorded(
        &mut self,
        mut events: Vec<egui::Event>,
        size: [f32; 2],
        focused: bool,
        mut phases: Option<&mut Vec<UiPhase>>,
    ) -> UiOutput {
        if self.ctx.input(|i| i.modifiers) != self.modifiers {
            events.insert(0, egui::Event::ModifiersChanged(self.modifiers));
        }
        let mut result = None;
        let mut frame = self.ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::from(size))),
                events,
                focused,
                ..Default::default()
            },
            |root| {
                result = Some(if let Some(phases) = &mut phases {
                    self.ui.show_observed(
                        root,
                        &mut self.session,
                        &mut self.presenter,
                        Some(&mut |phase| phases.push(phase)),
                    )
                } else {
                    self.ui.show(root, &mut self.session, &mut self.presenter)
                });
            },
        );
        fn labels(shape: &egui::Shape, output: &mut Vec<(String, Rect)>) {
            match shape {
                egui::Shape::Text(t) => output.push((
                    t.galley.text().to_owned(),
                    Rect::from_min_size(t.pos, t.galley.size()),
                )),
                egui::Shape::Vec(v) => {
                    for s in v {
                        labels(s, output)
                    }
                }
                _ => {}
            }
        }
        self.labels.clear();
        for shape in &frame.shapes {
            labels(&shape.shape, &mut self.labels);
        }
        frame.textures_delta.clear(); // No renderer in headless input tests.
        let output = result.unwrap();
        if let Some((size, generation)) = output.viewport {
            self.presenter.0 = Some(PresentedView {
                frame: FrameDescriptor {
                    api_version: PREVIEW_API_VERSION,
                    key: FrameKey(1),
                    status: FrameStatus::Ready,
                    source: FrameSource {
                        session: self.session.project_info().session_id,
                        state: self.session.project_info().state,
                        context: PreviewContext {
                            target: self.ui.target.unwrap(),
                            draft: EditorUi::draft_state(&self.session),
                            size,
                            viewport_generation: generation,
                            scene: self.ui.scene,
                            simulation_micros: self.ui.simulation_micros,
                        },
                    },
                },
                view: Affine2::IDENTITY,
            });
        }
        output
    }
    fn pointer(&mut self, point: Pos2, pressed: Option<bool>) {
        let mut events = vec![egui::Event::PointerMoved(point)];
        if let Some(pressed) = pressed {
            events.push(egui::Event::PointerButton {
                pos: point,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: self.modifiers,
            });
        }
        self.pass(events, [1180., 800.], true);
    }
    fn drag(&mut self) -> Pos2 {
        let point = self.ui.canvas_rect.unwrap().center();
        self.pointer(point, Some(true));
        self.pointer(point + Vec2::new(30., 0.), None);
        point
    }
    fn parameter_pass(&mut self, events: Vec<egui::Event>) -> Rect {
        let selection = self.ui.selection().unwrap();
        let mut values = EditorUi::transform(&EditorUi::project(&self.session), selection)
            .unwrap()
            .coefficients();
        let mut rect = Rect::NOTHING;
        let mut output = self.ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(500., 200.))),
                focused: true,
                events,
                ..Default::default()
            },
            |root| {
                let response = root.add(egui::DragValue::new(&mut values[4]).speed(1.));
                rect = response.rect;
                self.ui.parameter(
                    &mut self.session,
                    response,
                    vec![EditorUi::transform_command(
                        selection,
                        Affine2::new(values).unwrap(),
                    )],
                );
            },
        );
        output.textures_delta.clear();
        rect
    }
}
fn key(key: egui::Key) -> egui::Event {
    egui::Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Default::default(),
    }
}
#[test]
fn diagnostic_boundaries_preserve_project_layout_and_output() {
    let mut plain = Harness::new();
    let mut observed = Harness::new();
    let before = observed.session.snapshot();
    let nodes = observed.session.history_len();
    let mut phases = vec![];
    let a = plain.pass(vec![], [1180., 800.], true);
    let b = observed.pass_recorded(vec![], [1180., 800.], true, Some(&mut phases));
    assert_eq!(a.viewport, b.viewport);
    assert_eq!(a.game_inputs, b.game_inputs);
    assert_eq!(plain.ui.canvas_rect, observed.ui.canvas_rect);
    assert_eq!(observed.session.snapshot(), before);
    assert_eq!(observed.session.history_len(), nodes);
    assert_eq!(
        phases,
        [
            UiPhase::ProjectSnapshot,
            UiPhase::Prelude,
            UiPhase::SceneControls,
            UiPhase::SceneWidgets,
            UiPhase::SceneReadout,
            UiPhase::SceneControls,
            UiPhase::LayersAssets,
            UiPhase::ProjectSnapshot,
            UiPhase::LayersAssets,
            UiPhase::PropertiesHistory,
            UiPhase::ProjectSnapshot,
            UiPhase::PropertiesHistory,
            UiPhase::Canvas,
            UiPhase::ProjectSnapshot,
            UiPhase::Canvas,
            UiPhase::AudioPanel,
            UiPhase::SettingsDiagnostics,
        ]
    );
}
#[test]
fn audio_detail_observation_preserves_widgets_selection_and_project() {
    let mut h = Harness::new();
    let asset = AssetId::new(h.ui.object_id()).unwrap();
    assert!(h.ui.submit(
        &mut h.session,
        vec![Command::RegisterAsset {
                asset: Asset::new(
                    asset,
                    "sound.wav".into(),
                    BlobRef::new([7; 32], 9),
                    AssetKind::Opaque
                )
                .unwrap(),
            }]
    ));
    let id =
        h.ui.create_audio_component(&mut h.session, asset, "test.sound".into())
            .unwrap();
    h.ui.audio_component = Some(id);
    h.ui.open_audio_editor();
    let before = h.session.snapshot();
    let mut phases = vec![];
    for _ in 0..3 {
        phases.clear();
        h.pass_recorded(vec![], [1180., 800.], true, Some(&mut phases));
    }
    assert_eq!(h.session.snapshot(), before);
    assert_eq!(h.ui.audio_component, Some(id));
    let start = phases
        .iter()
        .position(|p| *p == UiPhase::AudioPanel)
        .unwrap();
    assert_eq!(
        &phases[start..],
        &[
            UiPhase::AudioPanel,
            UiPhase::AudioDevice,
            UiPhase::AudioStatus,
            UiPhase::AudioLibrary,
            UiPhase::AudioComponents,
            UiPhase::AudioProperties,
            UiPhase::AudioReadiness,
            UiPhase::AudioActions,
            UiPhase::AudioActions,
            UiPhase::AudioPanel,
            UiPhase::SettingsDiagnostics,
        ]
    );
}
#[test]
fn observed_escape_cancels_the_same_draft_without_a_history_node() {
    let mut h = Harness::new();
    let before = h.session.snapshot();
    let nodes = h.session.history_len();
    h.drag();
    assert!(EditorUi::draft_state(&h.session).is_some());
    let mut phases = vec![];
    h.pass_recorded(
        vec![key(egui::Key::Escape)],
        [1180., 800.],
        true,
        Some(&mut phases),
    );
    assert_eq!(h.session.snapshot(), before);
    assert_eq!(h.session.history_len(), nodes);
    assert!(EditorUi::draft_state(&h.session).is_none());
    assert_eq!(phases.last(), Some(&UiPhase::SettingsDiagnostics));
}
#[test]
fn fault_interrupt_cancels_canvas_drag_and_late_release_cannot_commit_it() {
    let mut h = Harness::new();
    let before = h.session.snapshot();
    let nodes = h.session.history_len();
    let p = h.drag();
    assert!(EditorUi::draft_state(&h.session).is_some());
    assert!(h.ui.interrupt_for_fault(&mut h.session));
    assert!(h.ui.interrupt_for_fault(&mut h.session));
    h.pointer(p + Vec2::new(80., 10.), None);
    h.pointer(p + Vec2::new(80., 10.), Some(false));
    assert!(h.ui.gesture.is_none() && h.ui.paused);
    assert!(EditorUi::draft_state(&h.session).is_none());
    assert_eq!(h.session.snapshot(), before);
    assert_eq!(h.session.history_len(), nodes);
    let p = h.drag();
    h.pointer(p + Vec2::new(40., 0.), Some(false));
    assert_eq!(h.session.history_len(), nodes + 1);
}
#[test]
fn fault_interrupt_keeps_numeric_drag_blocked_until_the_old_button_is_released() {
    let mut h = Harness::new();
    let before = h.session.snapshot();
    let point = h.parameter_pass(vec![]).center();
    let button = |pos, pressed| egui::Event::PointerButton {
        pos,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: Default::default(),
    };
    h.parameter_pass(vec![egui::Event::PointerMoved(point), button(point, true)]);
    h.parameter_pass(vec![egui::Event::PointerMoved(point + Vec2::new(20., 0.))]);
    assert!(h.ui.parameter_draft);
    assert!(h.ui.interrupt_for_fault(&mut h.session));
    h.parameter_pass(vec![egui::Event::PointerMoved(point + Vec2::new(70., 0.))]);
    h.parameter_pass(vec![button(point + Vec2::new(70., 0.), false)]);
    assert!(EditorUi::draft_state(&h.session).is_none());
    assert_eq!(h.session.snapshot(), before);
}
#[test]
fn fault_interrupt_does_not_cancel_an_external_draft_even_with_a_stale_ui_owner() {
    let mut h = Harness::new();
    h.drag();
    let old = EditorUi::draft_state(&h.session).unwrap();
    let mut other = ProjectSession::new(1, 999).unwrap();
    other
        .edit_draft(DraftRequest {
            api_version: API_VERSION,
            session_id: other.project_info().session_id,
            request_id: RequestId::new(5000).unwrap(),
            action: DraftAction::Begin {
                expected: other.project_info().state,
            },
        })
        .unwrap();
    let foreign = EditorUi::draft_state(&other).unwrap();
    let before = other.snapshot();
    assert_eq!(old.id, foreign.id);
    assert!(h.ui.interrupt_for_fault(&mut other));
    assert_eq!(EditorUi::draft_state(&other), Some(foreign));
    assert_eq!(other.snapshot(), before);
    assert!(h.ui.gesture.is_none());
}
#[test]
fn audio_backend_failure_during_cancel_does_not_keep_a_ui_draft_or_publish_a_node() {
    let (mut ui, mut session, trace, component) = audio_fixture();
    let before = session.snapshot();
    assert!(ui.draft(
        &mut session,
        DraftAction::Begin {
            expected: before.state
        }
    ));
    let expected = EditorUi::draft_state(&session).unwrap();
    assert!(ui.draft(
        &mut session,
        DraftAction::Replace {
            expected,
            commands: vec![Command::SetAudioProperties {
                component,
                gain: Gain::new(0.5).unwrap(),
                visible: true,
            }]
        }
    ));
    let cancelled = EditorUi::draft_state(&session).unwrap().id;
    trace.lock().unwrap().fail_once = true;
    trace.lock().unwrap().device_fault = true;
    assert!(ui.interrupt_for_fault(&mut session));
    assert!(EditorUi::draft_state(&session).is_none());
    assert_eq!(session.snapshot(), before);
    ui.refresh_audio(&session);
    assert!(trace.lock().unwrap().cancelled.contains(&cancelled));
    assert!(ui.paused && !ui.audio_state.enabled && ui.audio_state.frozen);
}
#[test]
fn continuous_canvas_drag_is_one_node_and_pause_does_not_block_edits() {
    let mut h = Harness::new();
    let before = h.session.snapshot();
    let p = h.drag();
    assert!(h.ui.paused);
    assert!(EditorUi::draft_state(&h.session).is_some());
    h.pointer(p + Vec2::new(80., 20.), None);
    assert_eq!(h.session.snapshot(), before);
    assert_ne!(EditorUi::project(&h.session), before.project);
    h.pointer(p + Vec2::new(80., 20.), Some(false));
    let after = h.session.snapshot();
    assert_ne!(after.project, before.project);
    let node = h
        .session
        .history_summary(after.session_id, after.state.head)
        .unwrap();
    assert_eq!(node.parent, Some(before.state.head));
    h.ui.navigation(&mut h.session, HistoryAction::Undo);
    assert_eq!(h.session.snapshot().project, before.project);
}
#[test]
fn escape_and_focus_loss_cancel_without_committed_changes() {
    for escape in [true, false] {
        let mut h = Harness::new();
        let before = h.session.snapshot();
        let p = h.drag();
        h.pass(
            if escape {
                vec![key(egui::Key::Escape)]
            } else {
                vec![]
            },
            [1180., 800.],
            escape,
        );
        h.pointer(p + Vec2::new(40., 0.), Some(false));
        assert_eq!(h.session.snapshot(), before);
        assert!(EditorUi::draft_state(&h.session).is_none());
    }
}
#[test]
fn stale_frame_blocks_new_drag_and_resize_cancels_active_drag() {
    let mut h = Harness::new();
    let before = h.session.snapshot();
    h.presenter
        .0
        .as_mut()
        .unwrap()
        .frame
        .source
        .context
        .viewport_generation = 0;
    let p = h.ui.canvas_rect.unwrap().center();
    h.pointer(p, Some(true));
    // Keep it stale for the drag threshold frame as well.
    h.presenter
        .0
        .as_mut()
        .unwrap()
        .frame
        .source
        .context
        .viewport_generation = 0;
    h.pointer(p + Vec2::new(30., 0.), None);
    assert!(EditorUi::draft_state(&h.session).is_none());
    h.pointer(p, Some(false));
    h.drag();
    assert!(EditorUi::draft_state(&h.session).is_some());
    h.pass(vec![], [1000., 700.], true);
    assert_eq!(h.session.snapshot(), before);
    assert!(EditorUi::draft_state(&h.session).is_none());
}
#[test]
fn canvas_focus_gates_keys_and_settings_do_not_enter_history() {
    let mut h = Harness::new();
    let before = h.session.snapshot();
    h.ui.interaction = true;
    h.ui.scene = 1;
    let output = h.pass(vec![key(egui::Key::Z)], [1180., 800.], true);
    assert_eq!(output.game_inputs, 0);
    let p = h.ui.canvas_rect.unwrap().center();
    h.pointer(p, Some(true));
    h.pointer(p, Some(false));
    let output = h.pass(vec![key(egui::Key::X)], [1180., 800.], true);
    assert_eq!(output.game_inputs, 1);
    h.ui.settings = true;
    h.ui.username = "本地玩家".into();
    h.ui.auto_play = false;
    h.pass(vec![], [1180., 800.], true);
    assert_eq!(h.session.snapshot(), before);
}
#[test]
fn creating_first_document_and_layer_is_atomic_on_asset_failure() {
    let mut session = ProjectSession::new(1, 2).unwrap();
    let mut ui = EditorUi::default();
    let before = session.snapshot();
    ui.add_asset_layer(&mut session, AssetId::new(99).unwrap());
    assert_eq!(session.snapshot(), before);
    assert!(ui.target.is_none());
    assert!(ui.selection().is_none());
}
#[test]
fn continuous_numeric_parameter_drag_uses_a_single_history_transaction() {
    let mut h = Harness::new();
    let before = h.session.snapshot();
    let p = h.parameter_pass(vec![]).center();
    let button = |point, pressed| egui::Event::PointerButton {
        pos: point,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: Default::default(),
    };
    h.parameter_pass(vec![egui::Event::PointerMoved(p), button(p, true)]);
    for delta in [20., 40., 60.] {
        h.parameter_pass(vec![egui::Event::PointerMoved(p + Vec2::new(delta, 0.))]);
    }
    assert_eq!(h.session.snapshot(), before);
    assert!(EditorUi::draft_state(&h.session).is_some());
    assert_ne!(EditorUi::project(&h.session), before.project);
    h.parameter_pass(vec![button(p + Vec2::new(60., 0.), false)]);
    let after = h.session.snapshot();
    assert_ne!(after.project, before.project);
    assert_eq!(
        h.session
            .history_summary(after.session_id, after.state.head)
            .unwrap()
            .parent,
        Some(before.state.head)
    );
}
#[test]
fn cancelled_numeric_drag_does_not_restart_as_immediate_commands_while_button_is_held() {
    let mut h = Harness::new();
    let before = h.session.snapshot();
    let p = h.parameter_pass(vec![]).center();
    let button = |point, pressed| egui::Event::PointerButton {
        pos: point,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: Default::default(),
    };
    h.parameter_pass(vec![egui::Event::PointerMoved(p), button(p, true)]);
    h.parameter_pass(vec![egui::Event::PointerMoved(p + Vec2::new(20., 0.))]);
    assert!(EditorUi::draft_state(&h.session).is_some());
    h.ui.cancel(&mut h.session);
    h.parameter_pass(vec![egui::Event::PointerMoved(p + Vec2::new(60., 0.))]);
    h.parameter_pass(vec![button(p + Vec2::new(60., 0.), false)]);
    assert_eq!(h.session.snapshot(), before);
}
#[test]
fn focus_loss_and_resize_do_not_cancel_a_draft_created_outside_the_ui() {
    let mut h = Harness::new();
    let session = h.session.project_info();
    h.session
        .edit_draft(DraftRequest {
            api_version: API_VERSION,
            session_id: session.session_id,
            request_id: RequestId::new(5000).unwrap(),
            action: DraftAction::Begin {
                expected: session.state,
            },
        })
        .unwrap();
    let state = EditorUi::draft_state(&h.session).unwrap();
    let before = h.session.snapshot();
    h.pass(vec![], [1000., 700.], false);
    assert_eq!(EditorUi::draft_state(&h.session), Some(state));
    assert_eq!(h.session.snapshot(), before);
}
#[test]
fn stale_ui_ownership_cannot_cancel_a_draft_in_another_session_with_the_same_id() {
    let mut h = Harness::new();
    h.drag();
    let old = EditorUi::draft_state(&h.session).unwrap();
    let mut other = ProjectSession::new(1, 999).unwrap();
    other
        .edit_draft(DraftRequest {
            api_version: API_VERSION,
            session_id: other.project_info().session_id,
            request_id: RequestId::new(5000).unwrap(),
            action: DraftAction::Begin {
                expected: other.project_info().state,
            },
        })
        .unwrap();
    let current = EditorUi::draft_state(&other).unwrap();
    assert_eq!(old.id, current.id);
    h.ui.cancel_owned(&mut other);
    assert_eq!(EditorUi::draft_state(&other), Some(current));
}
