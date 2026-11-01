#![forbid(unsafe_code)]
mod audio;
mod gestures;
mod history;
mod picking;
mod selection;
mod shortcuts;
use selection::EditGesture;
pub use shortcuts::{
    DEFAULT_SHORTCUTS, EditorAction, MODIFIER_HINTS, ShortcutBinding, ShortcutScope,
};
mod observation;
pub use observation::UiPhase;
use observation::phase;
#[cfg(test)]
mod tests;
use egui::{Color32, Pos2, Rect, Sense, Stroke, Vec2};
use skin_editor_application::{values::*, *};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Selection {
    Layer(DocumentId, LayerId),
    Component(ComponentId),
}
#[derive(Clone, Copy)]
pub struct PresentedView {
    pub frame: FrameDescriptor,
    pub view: Affine2,
}
/// Only presentation; cannot read or mutate a project or expose GPU handles.
pub trait Presenter {
    fn current(&self) -> Option<PresentedView>;
    fn paint(&mut self, ui: &mut egui::Ui, key: FrameKey, rect: Rect);
    fn thumbnail(&mut self, ui: &mut egui::Ui, blob: BlobRef);
}
#[derive(Default)]
pub struct UiOutput {
    pub imports: Vec<String>,
    pub sample: bool,
    pub viewport: Option<([u32; 2], u64)>,
    pub game_inputs: u32,
    pub audio_imports: Vec<(String, bool)>,
    pub sample_sound: bool,
}
pub struct EditorUi {
    audio_port: Box<dyn AudioEditorPort>,
    pub audio_state: AudioEditorState,
    audio_sync_pending: bool,
    audio_sync_error: Option<String>,
    audio_window: bool,
    audio_labels: audio::CounterLabels,
    pub audio_asset: Option<AssetId>,
    pub pending_imports: usize,
    pub audio_component: Option<AudioComponentId>,
    audio_path: String,
    audio_original: bool,
    audio_slot: String,
    pub target: Option<PreviewTarget>,
    selected: Vec<Selection>,
    selection_frame: Option<selection::Frame>,
    pub paused: bool,
    pub interaction: bool,
    previous_interaction: bool,
    shortcuts: shortcuts::Router,
    pub scene: u8,
    pub auto_play: bool,
    pub fullscreen: bool,
    pub message: String,
    pub username: String,
    pub avatar: String,
    pub default_skin: String,
    pub stable: String,
    pub songs: String,
    pub simulation_micros: u64,
    pub accepted_inputs: u64,
    next_object: u128,
    next_request: u128,
    gesture: Option<EditGesture>,
    parameter_draft: bool,
    owned_draft: Option<(HistorySessionId, DraftId)>,
    blocked_parameter: bool,
    viewport_size: [u32; 2],
    viewport_generation: u64,
    path: String,
    settings: bool,
    pub canvas_rect: Option<Rect>,
}
impl Default for EditorUi {
    fn default() -> Self {
        Self {
            audio_port: Box::new(DisabledAudio),
            audio_state: AudioEditorState::default(),
            audio_sync_pending: false,
            audio_sync_error: None,
            audio_window: false,
            audio_labels: audio::CounterLabels::default(),
            audio_asset: None,
            pending_imports: 0,
            audio_component: None,
            audio_path: String::new(),
            audio_original: false,
            audio_slot: "自定义音效".into(),
            target: None,
            selected: Vec::new(),
            selection_frame: None,
            paused: true,
            interaction: false,
            previous_interaction: false,
            shortcuts: shortcuts::Router::default(),
            scene: 0,
            auto_play: true,
            fullscreen: false,
            message: String::new(),
            username: "Player".into(),
            avatar: String::new(),
            default_skin: String::new(),
            stable: String::new(),
            songs: String::new(),
            simulation_micros: 0,
            accepted_inputs: 0,
            next_object: 1,
            next_request: 1,
            gesture: None,
            parameter_draft: false,
            owned_draft: None,
            blocked_parameter: false,
            viewport_size: [0, 0],
            viewport_generation: 0,
            path: String::new(),
            settings: false,
            canvas_rect: None,
        }
    }
}
impl EditorUi {
    pub fn object_id(&mut self) -> u128 {
        let id = self.next_object;
        self.next_object += 1;
        id
    }
    fn request_id(&mut self) -> RequestId {
        let id = self.next_request;
        self.next_request += 1;
        RequestId::new(id).expect("request id")
    }
    pub fn submit(&mut self, session: &mut ProjectSession, commands: Vec<Command>) -> bool {
        let info = session.project_info();
        let id = self.request_id();
        match session.submit(CommandRequest {
            api_version: API_VERSION,
            session_id: info.session_id,
            request_id: id,
            expected: info.state,
            commands,
        }) {
            Ok(_) => {
                self.synchronize_audio(session);
                true
            }
            Err(e) => {
                self.message = format!("{e:?}");
                false
            }
        }
    }
    pub fn draft(&mut self, session: &mut ProjectSession, action: DraftAction) -> bool {
        let beginning = matches!(&action, DraftAction::Begin { .. });
        let id = self.request_id();
        match session.edit_draft(DraftRequest {
            api_version: API_VERSION,
            session_id: session.project_info().session_id,
            request_id: id,
            action,
        }) {
            Ok(_) => {
                if beginning {
                    self.owned_draft = Self::draft_state(session)
                        .map(|d| (session.project_info().session_id, d.id));
                } else if Self::draft_state(session).is_none() {
                    self.owned_draft = None;
                }
                self.synchronize_audio(session);
                true
            }
            Err(e) => {
                self.message = format!("{e:?}");
                false
            }
        }
    }
    fn draft_state(session: &ProjectSession) -> Option<DraftState> {
        session
            .draft_info(session.project_info().session_id)
            .ok()
            .flatten()
            .map(|d| d.state)
    }
    pub fn cancel(&mut self, session: &mut ProjectSession) {
        self.blocked_parameter |= self.parameter_draft;
        let before = self.gesture.take().map(|g| g.before);
        let mut cancelled = true;
        if let Some(expected) = Self::draft_state(session) {
            cancelled = self.draft(session, DraftAction::Cancel { expected });
        }
        self.selection_frame = None;
        if cancelled && let Some(geometry) = before {
            self.remember_frame(session, geometry);
        }
        self.parameter_draft = false;
    }
    fn cancel_owned(&mut self, session: &mut ProjectSession) {
        if self.owned_draft.is_some()
            && self.owned_draft
                == Self::draft_state(session).map(|d| (session.project_info().session_id, d.id))
        {
            self.cancel(session);
        } else {
            self.owned_draft = None;
            self.blocked_parameter |= self.parameter_draft;
            self.gesture = None;
            self.selection_frame = None;
            self.parameter_draft = false;
        }
    }
    /// Faults interrupt this UI's transient edit, never a foreign draft. The
    /// original pointer release must not commit or restart the interrupted edit.
    pub fn interrupt_for_fault(&mut self, session: &mut ProjectSession) -> bool {
        self.paused = true;
        self.cancel_owned(session);
        // An external adapter may have replaced the selected draft while this
        // UI retained an old gesture. Select the actual source, without cancelling it.
        self.synchronize_audio(session);
        self.audio_state = self.audio_port.poll();
        if self.audio_state.enabled {
            self.audio_command(session, AudioEditorAction::SceneRunning(false));
            self.audio_command(session, AudioEditorAction::StopAudition);
        }
        self.owned_draft.is_none()
    }
    pub fn commit(&mut self, session: &mut ProjectSession) {
        if self.owned_draft
            != Self::draft_state(session).map(|d| (session.project_info().session_id, d.id))
            || self.owned_draft.is_none()
        {
            self.cancel_owned(session);
            return;
        }
        let after = self.gesture.take().map(|g| g.after);
        let mut committed = false;
        if let Some(expected) = Self::draft_state(session) {
            committed = self.draft(session, DraftAction::Commit { expected });
        }
        self.selection_frame = None;
        if committed && let Some(geometry) = after {
            self.remember_frame(session, geometry);
        }
        self.parameter_draft = false;
    }
    fn parameter(
        &mut self,
        session: &mut ProjectSession,
        response: egui::Response,
        commands: Vec<Command>,
    ) {
        if self.blocked_parameter {
            if response.drag_stopped() {
                self.blocked_parameter = false;
            }
            return;
        }
        if response.drag_started() && Self::draft_state(session).is_none() {
            self.parameter_draft = self.draft(
                session,
                DraftAction::Begin {
                    expected: session.project_info().state,
                },
            );
        }
        if response.changed() {
            if self.parameter_draft {
                if let Some(expected) = Self::draft_state(session) {
                    self.draft(session, DraftAction::Replace { expected, commands });
                }
            } else {
                self.submit(session, commands);
            }
        }
        if response.drag_stopped() && self.parameter_draft {
            self.commit(session);
        }
    }
    pub fn navigation(&mut self, session: &mut ProjectSession, action: HistoryAction) {
        if Self::draft_state(session).is_some() {
            self.message = "请先完成或取消当前编辑".into();
            return;
        }
        let info = session.project_info();
        let id = self.request_id();
        if let Err(e) = session.navigate(HistoryRequest {
            api_version: API_VERSION,
            session_id: info.session_id,
            request_id: id,
            expected: info.state,
            action,
        }) {
            self.message = format!("{e:?}");
        }
        self.synchronize_audio(session);
        self.reconcile(session);
    }
    pub fn project(session: &ProjectSession) -> SkinProject {
        if let Some(state) = Self::draft_state(session)
            && let Ok(snapshot) = session.draft_snapshot(session.project_info().session_id, state)
        {
            return snapshot.project;
        }
        session.snapshot().project
    }
    pub fn reconcile(&mut self, session: &ProjectSession) {
        let project = Self::project(session);
        self.reconcile_project(&project);
    }
    fn reconcile_project(&mut self, project: &SkinProject) {
        let old_len = self.selected.len();
        self.selected.retain(|s| !match *s {
            Selection::Layer(d, l) => project
                .document(d)
                .is_none_or(|d| d.layers().node(l).is_none()),
            Selection::Component(c) => project.components().node(c).is_none(),
        });
        if self.selected.len() != old_len {
            self.selection_frame = None;
        }
        if matches!(self.target,Some(PreviewTarget::Document(id)) if project.document(id).is_none())
        {
            self.target = None;
        }
        if self.target.is_none() {
            self.target = project
                .documents()
                .next()
                .map(|d| PreviewTarget::Document(d.id()));
        }
    }
    pub fn add_document(&mut self, session: &mut ProjectSession) -> DocumentId {
        let id = DocumentId::new(self.object_id()).unwrap();
        if self.submit(
            session,
            vec![Command::CreateDocument {
                document: ElementDocument::new(
                    id,
                    PixelSize::new(512, 512).unwrap(),
                    LayerTree::default(),
                ),
            }],
        ) {
            self.select(session, None);
            self.target = Some(PreviewTarget::Document(id));
        }
        id
    }
    pub fn add_asset_layer(&mut self, session: &mut ProjectSession, asset: AssetId) {
        if self.selected.len() > 1
            || matches!(
                self.target,
                Some(PreviewTarget::Workspace | PreviewTarget::Components)
            )
        {
            self.message = "请先选择单个元素，再加入图层素材".into();
            return;
        }
        let mut commands = Vec::new();
        let doc = match self.target {
            Some(PreviewTarget::Document(id)) => id,
            _ => {
                let id = DocumentId::new(self.object_id()).unwrap();
                commands.push(Command::CreateDocument {
                    document: ElementDocument::new(
                        id,
                        PixelSize::new(512, 512).unwrap(),
                        LayerTree::default(),
                    ),
                });
                id
            }
        };
        let project = Self::project(session);
        let parent = match self.selection() {
            Some(Selection::Layer(d, l))
                if d == doc
                    && matches!(
                        project
                            .document(d)
                            .and_then(|d| d.layers().node(l))
                            .map(|n| &n.kind),
                        Some(LayerKind::Group { .. })
                    ) =>
            {
                Some(l)
            }
            _ => None,
        };
        let index = project.document(doc).map_or(0, |d| {
            parent.map_or(d.layers().roots().len(), |p| {
                d.layers().node(p).unwrap().children().len()
            })
        });
        let id = LayerId::new(self.object_id()).unwrap();
        commands.push(Command::AddLayer {
            document: doc,
            node: LayerNode {
                id,
                local: Affine2::new([1.0, 0.0, 0.0, 1.0, 128.0, 128.0]).unwrap(),
                opacity: Opacity::OPAQUE,
                visible: true,
                kind: LayerKind::Image { asset },
            },
            parent,
            index,
        });
        if self.submit(session, commands) {
            self.target = Some(PreviewTarget::Document(doc));
            self.select(session, Some(Selection::Layer(doc, id)));
        }
    }
    fn transform(project: &SkinProject, selection: Selection) -> Option<Affine2> {
        match selection {
            Selection::Layer(d, l) => Some(project.document(d)?.layers().node(l)?.local),
            Selection::Component(c) => Some(project.components().node(c)?.local),
        }
    }
    fn transform_command(selection: Selection, transform: Affine2) -> Command {
        match selection {
            Selection::Layer(document, layer) => Command::SetLayerTransform {
                document,
                layer,
                transform,
            },
            Selection::Component(component) => Command::SetComponentTransform {
                component,
                transform,
            },
        }
    }
    pub fn show(
        &mut self,
        root: &mut egui::Ui,
        session: &mut ProjectSession,
        presenter: &mut dyn Presenter,
    ) -> UiOutput {
        self.show_observed(root, session, presenter, None)
    }
    /// Same editor path, with an optional stack-scoped diagnostic observer.
    /// The observer receives only boundaries and cannot control the editor.
    pub fn show_observed(
        &mut self,
        root: &mut egui::Ui,
        session: &mut ProjectSession,
        presenter: &mut dyn Presenter,
        mut observer: Option<&mut dyn FnMut(UiPhase)>,
    ) -> UiOutput {
        let ctx = root.ctx().clone();
        let mut output = UiOutput::default();
        self.sync_interaction_mode(session);
        let mut keyboard = self.shortcuts.begin(&ctx);
        if self.owned_draft.is_some()
            && self.owned_draft
                != Self::draft_state(session).map(|d| (session.project_info().session_id, d.id))
        {
            self.cancel_owned(session);
        }
        if !ctx.input(|i| i.pointer.any_down()) {
            self.blocked_parameter = false;
        }
        self.reconcile_project(&Self::observed_project(
            session,
            &mut observer,
            UiPhase::Prelude,
        ));
        if keyboard.early_cancel(&ctx, !self.interaction, self.owned_draft.is_some()) {
            self.perform_action(session, EditorAction::Cancel);
        } else if !ctx.input(|i| i.focused) {
            self.cancel_owned(session);
        }
        phase(&mut observer, UiPhase::SceneControls);
        egui::Panel::top("scene_controls").show(root, |ui| {
            ui.horizontal(|ui| {
                phase(&mut observer, UiPhase::SceneWidgets);
                ui.strong("Skin Editor · 占位预览");
                ui.label("预览场景");
                ui.selectable_value(&mut self.scene, 0, "画布");
                ui.selectable_value(&mut self.scene, 1, "GUI 占位")
                    .on_hover_text("切换预览内容；操作工具保持不变");
                if ui.checkbox(&mut self.paused, "冻结模拟").changed() && self.audio_state.enabled
                {
                    self.audio_command(session, AudioEditorAction::SceneRunning(!self.paused));
                }
                if ui.button("音效编辑").clicked() {
                    self.audio_window = !self.audio_window;
                }
            });
            ui.horizontal(|ui| {
                ui.label("操作工具");
                ui.selectable_value(&mut self.interaction, false, "编辑");
                ui.selectable_value(&mut self.interaction, true, "交互")
                    .on_hover_text("输入交给预览；清除并关闭图层／组件选择");
                self.sync_interaction_mode(session);
                if ui
                    .add_enabled(!self.interaction, egui::Button::new("撤销"))
                    .clicked()
                {
                    self.perform_action(session, EditorAction::Undo);
                }
                if ui
                    .add_enabled(!self.interaction, egui::Button::new("重做"))
                    .clicked()
                {
                    self.perform_action(session, EditorAction::Redo);
                }
                if ui.button("设置").clicked() {
                    self.settings = true;
                }
                phase(&mut observer, UiPhase::SceneReadout);
                ui.label(format!(
                    "{:.3}s · 输入 {}",
                    self.simulation_micros as f64 / 1e6,
                    self.accepted_inputs
                ));
                phase(&mut observer, UiPhase::SceneControls);
            });
        });
        phase(&mut observer, UiPhase::LayersAssets);
        egui::Panel::left("layers_assets")
            .default_size(250.0)
            .resizable(true)
            .show(root, |ui| {
                ui.heading("元素与图层");
                if self.interaction {
                    ui.weak("交互工具：可切换预览元素，图层选择已关闭");
                }
                let project = Self::observed_project(session, &mut observer, UiPhase::LayersAssets);
                ui.horizontal(|ui| {
                    if ui.button("新建元素").clicked() {
                        self.add_document(session);
                    }
                    if ui
                        .add_enabled(self.selected.len() <= 1, egui::Button::new("新建组"))
                        .clicked()
                        && let Some(PreviewTarget::Document(doc)) = self.target
                    {
                        let id = LayerId::new(self.object_id()).unwrap();
                        let parent = match self.selection() {
                            Some(Selection::Layer(d, p))
                                if d == doc
                                    && project
                                        .document(d)
                                        .and_then(|d| d.layers().node(p))
                                        .is_some_and(|n| {
                                            matches!(n.kind, LayerKind::Group { .. })
                                        }) =>
                            {
                                Some(p)
                            }
                            _ => None,
                        };
                        let tree = project.document(doc).unwrap().layers();
                        let index = parent.map_or(tree.roots().len(), |p| {
                            tree.node(p).unwrap().children().len()
                        });
                        self.submit(
                            session,
                            vec![Command::AddLayer {
                                document: doc,
                                node: LayerNode {
                                    id,
                                    local: Affine2::IDENTITY,
                                    opacity: Opacity::OPAQUE,
                                    visible: true,
                                    kind: LayerKind::Group { children: vec![] },
                                },
                                parent,
                                index,
                            }],
                        );
                        self.select(session, Some(Selection::Layer(doc, id)));
                    }
                });
                egui::ScrollArea::vertical()
                    .id_salt("layers")
                    .max_height(290.0)
                    .show(ui, |ui| {
                        for doc in project.documents() {
                            if ui
                                .selectable_label(
                                    self.target == Some(PreviewTarget::Document(doc.id())),
                                    format!(
                                        "元素 {} · {}×{}",
                                        doc.id().get(),
                                        doc.canvas().width(),
                                        doc.canvas().height()
                                    ),
                                )
                                .clicked()
                            {
                                self.select(session, None);
                                self.target = Some(PreviewTarget::Document(doc.id()));
                            }
                            for node in doc.layers().iter() {
                                let mut depth = 0;
                                let mut parent = doc.layers().parent(node.id).flatten();
                                while let Some(p) = parent {
                                    depth += 1;
                                    parent = doc.layers().parent(p).flatten();
                                }
                                ui.horizontal(|ui| {
                                    ui.add_space(depth.min(12) as f32 * 12.0 + 12.0);
                                    let group = matches!(node.kind, LayerKind::Group { .. });
                                    if ui
                                        .add_enabled(
                                            !self.interaction,
                                            egui::Button::selectable(
                                                self.selected
                                                    .contains(&Selection::Layer(doc.id(), node.id)),
                                                format!(
                                                    "{} {}",
                                                    if group { "▾ 组" } else { "图层" },
                                                    node.id.get()
                                                ),
                                            ),
                                        )
                                        .clicked()
                                    {
                                        self.pick(
                                            session,
                                            Selection::Layer(doc.id(), node.id),
                                            ctx.input(|i| shortcuts::additive(i.modifiers)),
                                        );
                                    }
                                });
                            }
                        }
                    });
                ui.collapsing("跨元素组件", |ui| {
                    ui.label("Ctrl／Command 点击可联合选择；选择不会创建组件。");
                    for node in project.components().iter() {
                        let mut depth = 0;
                        let mut parent = project.components().parent(node.id).flatten();
                        while let Some(p) = parent {
                            depth += 1;
                            parent = project.components().parent(p).flatten();
                        }
                        let label = match node.kind {
                            ComponentKind::Group { .. } => format!("组件组 {}", node.id.get()),
                            ComponentKind::Element { document, .. } => {
                                format!("元素引用 {} · 元素 {}", node.id.get(), document.get())
                            }
                        };
                        ui.horizontal(|ui| {
                            ui.add_space(depth.min(12) as f32 * 12.0);
                            if ui
                                .add_enabled(
                                    !self.interaction,
                                    egui::Button::selectable(
                                        self.selected.contains(&Selection::Component(node.id)),
                                        label,
                                    ),
                                )
                                .clicked()
                            {
                                self.pick(
                                    session,
                                    Selection::Component(node.id),
                                    ctx.input(|i| shortcuts::additive(i.modifiers)),
                                );
                            }
                        });
                    }
                });
                ui.collapsing("素材库 · PNG", |ui| {
                    ui.text_edit_singleline(&mut self.path);
                    ui.horizontal(|ui| {
                        if ui.button("导入路径").clicked() && !self.path.trim().is_empty() {
                            output.imports.push(self.path.trim().into());
                        }
                        if ui.button("自制样例").clicked() {
                            output.sample = true;
                        }
                    });
                    ui.label("也可把 PNG 文件拖入窗口。导入后再加入图层。");
                    egui::ScrollArea::vertical()
                        .id_salt("assets")
                        .max_height(220.0)
                        .show(ui, |ui| {
                            for asset in project.assets().iter() {
                                ui.horizontal(|ui| {
                                    presenter.thumbnail(ui, asset.blob());
                                    ui.vertical(|ui| {
                                        ui.label(asset.name());
                                        if ui.small_button("加入当前元素").clicked() {
                                            self.add_asset_layer(session, asset.id());
                                        }
                                    });
                                });
                            }
                        });
                });
            });
        phase(&mut observer, UiPhase::PropertiesHistory);
        egui::Panel::right("parameters_history")
            .default_size(235.0)
            .resizable(true)
            .show(root, |ui| {
                ui.heading("参数");
                let project =
                    Self::observed_project(session, &mut observer, UiPhase::PropertiesHistory);
                let parameter_height = ui.available_height() * 0.45;
                egui::ScrollArea::vertical()
                    .id_salt("parameter_values")
                    .max_height(parameter_height)
                    .show(ui, |ui| {
                        if self.selected.len() == 1
                            && let Some(selection) = self.selection()
                            && let Some(transform) = Self::transform(&project, selection)
                        {
                            let mut values = transform.coefficients();
                            ui.horizontal(|ui| {
                                ui.label("X");
                                let x = ui.add(egui::DragValue::new(&mut values[4]).speed(1.0));
                                if let Ok(t) = Affine2::new(values) {
                                    self.parameter(
                                        session,
                                        x,
                                        vec![Self::transform_command(selection, t)],
                                    );
                                }
                                ui.label("Y");
                                let y = ui.add(egui::DragValue::new(&mut values[5]).speed(1.0));
                                if let Ok(t) = Affine2::new(values) {
                                    self.parameter(
                                        session,
                                        y,
                                        vec![Self::transform_command(selection, t)],
                                    );
                                }
                            });
                            ui.horizontal(|ui| {
                                for (label, scale) in [("缩小", 0.9), ("放大", 1.1)] {
                                    if ui.button(label).clicked() {
                                        let s = Affine2::new([scale, 0.0, 0.0, scale, 0.0, 0.0])
                                            .unwrap();
                                        if let Ok(next) = transform.compose(s) {
                                            self.submit(
                                                session,
                                                vec![Self::transform_command(selection, next)],
                                            );
                                        }
                                    }
                                }
                            });
                            if ui.button("旋转 +15°").clicked() {
                                let a = 15.0_f64.to_radians();
                                let r =
                                    Affine2::new([a.cos(), a.sin(), -a.sin(), a.cos(), 0.0, 0.0])
                                        .unwrap();
                                if let Ok(next) = transform.compose(r) {
                                    self.submit(
                                        session,
                                        vec![Self::transform_command(selection, next)],
                                    );
                                }
                            }
                            if let Selection::Layer(document, layer) = selection {
                                let node = project
                                    .document(document)
                                    .unwrap()
                                    .layers()
                                    .node(layer)
                                    .unwrap();
                                let mut visible = node.visible;
                                let mut opacity = node.opacity.get();
                                let c = ui.checkbox(&mut visible, "可见").changed();
                                if c {
                                    self.submit(
                                        session,
                                        vec![Command::SetLayerProperties {
                                            document,
                                            layer,
                                            opacity: Opacity::new(opacity).unwrap(),
                                            visible,
                                        }],
                                    );
                                }
                                let alpha = ui.add(
                                    egui::Slider::new(&mut opacity, 0.0..=1.0).text("不透明度"),
                                );
                                self.parameter(
                                    session,
                                    alpha,
                                    vec![Command::SetLayerProperties {
                                        document,
                                        layer,
                                        opacity: Opacity::new(opacity).unwrap(),
                                        visible,
                                    }],
                                );
                                if ui.button("移入选中元素的首个组").clicked()
                                    && let Some(parent) = project
                                        .document(document)
                                        .unwrap()
                                        .layers()
                                        .iter()
                                        .find(|n| {
                                            n.id != layer
                                                && matches!(n.kind, LayerKind::Group { .. })
                                        })
                                        .map(|n| n.id)
                                {
                                    self.submit(
                                        session,
                                        vec![Command::MoveLayer {
                                            document,
                                            layer,
                                            parent: Some(parent),
                                            index: 0,
                                        }],
                                    );
                                }
                                if ui.button("移到顶层").clicked() {
                                    self.submit(
                                        session,
                                        vec![Command::MoveLayer {
                                            document,
                                            layer,
                                            parent: None,
                                            index: 0,
                                        }],
                                    );
                                }
                            }
                            ui.label(shortcuts::TRANSFORM_HINT);
                        } else {
                            if self.selected.len() > 1 {
                                ui.label(format!("已选 {} 项 · 临时联合变换", self.selected.len()));
                            }
                            ui.label(shortcuts::SELECTION_HINT);
                        }
                        if ui
                            .add_enabled(
                                !self.interaction && !self.selected.is_empty(),
                                egui::Button::new("删除选中项"),
                            )
                            .clicked()
                        {
                            self.perform_action(session, EditorAction::DeleteSelection);
                        }
                    });
                ui.separator();
                self.history_panel(ui, session);
            });
        phase(&mut observer, UiPhase::Canvas);
        let mut canvas_focused = false;
        egui::CentralPanel::default().show(root, |ui| {
            ui.label("占位场景 · 非 osu!stable 内核");
            let available = ui.available_size().max(Vec2::splat(32.0));
            let (rect, response) = ui.allocate_exact_size(available, Sense::click_and_drag());
            self.canvas_rect = Some(rect);
            let ppp = ui.ctx().pixels_per_point();
            let ratio = (1024.0 / (available.x.max(available.y) * ppp)).min(1.0);
            let size = [
                (available.x * ppp * ratio).round().max(1.0) as u32,
                (available.y * ppp * ratio).round().max(1.0) as u32,
            ];
            if size != self.viewport_size {
                self.cancel_owned(session);
                self.viewport_size = size;
                self.viewport_generation += 1;
            }
            output.viewport = Some((size, self.viewport_generation));
            let shown = presenter.current();
            if let Some(view) = shown {
                presenter.paint(ui, view.frame.key, rect);
            } else {
                ui.painter()
                    .rect_filled(rect, 0.0, Color32::from_rgb(24, 27, 34));
            }
            let project = Self::observed_project(session, &mut observer, UiPhase::Canvas);
            let mut current = false;
            let mut transform_box = None;
            if let Some(view) = shown {
                current = view.frame.api_version == PREVIEW_API_VERSION
                    && view.frame.status == FrameStatus::Ready
                    && view.frame.source.session == session.project_info().session_id
                    && view.frame.source.context.size == self.viewport_size
                    && view.frame.source.state == session.project_info().state
                    && view.frame.source.context.draft == Self::draft_state(session)
                    && view.frame.source.context.viewport_generation == self.viewport_generation
                    && Some(view.frame.source.context.target) == self.target;
                if let Some(geometry) = self.selection_geometry(session, &project)
                    && let Some(controls) = geometry.controls(view, rect)
                {
                    transform_box = Some(controls);
                    ui.painter().add(egui::Shape::closed_line(
                        controls.corners.to_vec(),
                        Stroke::new(1.5, Color32::YELLOW),
                    ));
                    for p in controls.corners {
                        ui.painter().rect_filled(
                            Rect::from_center_size(p, Vec2::splat(6.0)),
                            0.0,
                            Color32::YELLOW,
                        );
                    }
                    if !self.interaction {
                        let handle = gestures::rotation(controls, rect);
                        ui.painter().line_segment(
                            [controls.top(), handle],
                            Stroke::new(1., Color32::YELLOW),
                        );
                        ui.painter().circle_filled(handle, 4., Color32::YELLOW);
                    }
                }
            }
            if !current {
                ui.painter().text(
                    rect.left_top() + Vec2::splat(10.0),
                    egui::Align2::LEFT_TOP,
                    "预览更新中",
                    egui::FontId::proportional(14.0),
                    Color32::WHITE,
                );
            }
            if response.clicked() {
                response.request_focus();
                if !self.interaction
                    && current
                    && let Some(p) = response.interact_pointer_pos()
                    && !transform_box.is_some_and(|b| {
                        (self.selected.len() > 1
                            && b.contains(p)
                            && !ctx.input(|i| shortcuts::additive(i.modifiers)))
                            || b.corners.iter().any(|c| c.distance(p) <= 9.)
                            || gestures::rotation(b, rect).distance(p) <= 10.
                    })
                    && let Some(view) = shown
                {
                    let item = picking::layer_at(&project, view, rect, p);
                    let additive = ctx.input(|i| shortcuts::additive(i.modifiers));
                    if let Some(item) = item {
                        if additive || self.selected != [item] {
                            let target = self.target;
                            self.pick(session, item, additive);
                            // Picking preserves the projection the pointer came from.
                            self.target = target;
                        }
                    } else if !additive {
                        self.select(session, None);
                    }
                }
            }
            if self.interaction && self.scene == 1 {
                let button = Rect::from_min_size(
                    rect.left_top() + Vec2::splat(20.0),
                    Vec2::new(130.0, 42.0),
                );
                ui.painter().rect_stroke(
                    button,
                    4.0,
                    Stroke::new(1.0, Color32::LIGHT_GREEN),
                    egui::StrokeKind::Inside,
                );
                ui.painter().text(
                    button.center(),
                    egui::Align2::CENTER_CENTER,
                    "占位 GUI 判定区",
                    egui::FontId::proportional(14.0),
                    Color32::LIGHT_GREEN,
                );
                if response.clicked()
                    && response
                        .interact_pointer_pos()
                        .is_some_and(|p| button.contains(p))
                {
                    output.game_inputs += 1;
                }
            }
            if self.scene == 1 {
                let phase = self.simulation_micros as f64 / 1_000_000.0;
                let dot = rect.right_top() + Vec2::new(-40.0 + phase.sin() as f32 * 12.0, 35.0);
                ui.painter().circle_filled(dot, 6.0, Color32::LIGHT_BLUE);
                ui.painter().text(
                    rect.right_bottom() - Vec2::splat(10.0),
                    egui::Align2::RIGHT_BOTTOM,
                    &self.username,
                    egui::FontId::proportional(16.0),
                    Color32::WHITE,
                );
            }
            canvas_focused = response.has_focus();
            if !self.interaction {
                if response.drag_started()
                    && current
                    && let (Some(view), Some(geometry)) =
                        (shown, self.selection_geometry(session, &project))
                {
                    let ratio = [
                        f64::from(size[0]) / f64::from(rect.width()),
                        f64::from(size[1]) / f64::from(rect.height()),
                    ];
                    let to_frame = |p: Pos2| {
                        [
                            f64::from(p.x - rect.left()) * ratio[0],
                            f64::from(p.y - rect.top()) * ratio[1],
                        ]
                    };
                    if let Some(press) = ctx.input(|i| i.pointer.press_origin())
                        && let Some(gesture) = EditGesture::new(
                            &project,
                            &self.selected,
                            geometry,
                            view,
                            ratio,
                            to_frame(press),
                            gestures::hit(transform_box, rect, press, to_frame),
                        )
                        && self.draft(
                            session,
                            DraftAction::Begin {
                                expected: session.project_info().state,
                            },
                        )
                    {
                        self.gesture = Some(gesture);
                    }
                }
                if (response.dragged() || response.drag_stopped())
                    && (current || response.drag_stopped())
                    && let (Some(g), Some(expected)) = (&self.gesture, Self::draft_state(session))
                    && let Some(pointer) = ctx.input(|i| i.pointer.latest_pos())
                    && let Some((values, geometry)) = g.values(
                        pointer,
                        rect.min,
                        ctx.input(|i| shortcuts::proportional(i.modifiers)),
                    )
                {
                    let commands: Vec<_> = values
                        .into_iter()
                        .filter(|(s, t)| Self::transform(&project, *s) != Some(*t))
                        .map(|(s, t)| Self::transform_command(s, t))
                        .collect();
                    if commands.is_empty()
                        || self.draft(session, DraftAction::Replace { expected, commands })
                    {
                        if let Some(g) = &mut self.gesture {
                            g.after = geometry;
                        }
                        self.remember_frame(session, geometry);
                    } else {
                        self.cancel_owned(session);
                    }
                }
                if response.drag_stopped() && self.gesture.is_some() {
                    self.commit(session);
                }
            }
        });
        for file in ctx.input(|i| i.raw.dropped_files.clone()) {
            output
                .imports
                .push(file.path().to_string_lossy().into_owned());
        }
        phase(&mut observer, UiPhase::AudioPanel);
        self.audio_panel(&ctx, session, &mut output, &mut observer);
        phase(&mut observer, UiPhase::SettingsDiagnostics);
        if self.settings {
            egui::Window::new("编辑器设置")
                .open(&mut self.settings)
                .show(&ctx, |ui| {
                    ui.label("本地展示资料，不写入 skin.ini 或项目历史");
                    ui.label("玩家名称");
                    ui.text_edit_singleline(&mut self.username);
                    for (label, value) in [
                        ("头像路径（预留）", &mut self.avatar),
                        ("外部默认皮肤（预留）", &mut self.default_skin),
                        ("stable 安装目录（预留）", &mut self.stable),
                        ("Songs 来源（预留）", &mut self.songs),
                    ] {
                        ui.label(label);
                        ui.text_edit_singleline(value);
                    }
                    ui.label("本轮仅保存于内存，不读取这些来源或包含原版素材。");
                    ui.checkbox(
                        &mut self.auto_play,
                        "嵌入预览默认自动演示（真实谱面未接入）",
                    );
                    ui.checkbox(&mut self.fullscreen, "全屏测试窗口");
                    ui.collapsing("快捷键（当前默认）", |ui| {
                        for b in DEFAULT_SHORTCUTS {
                            ui.label(format!("{} · {}", b.display, b.description));
                        }
                        for (key, description) in MODIFIER_HINTS {
                            ui.label(format!("{key} · {description}"));
                        }
                        ui.label("文本编辑与输入法优先；项目操作不响应按键自动重复。");
                    });
                });
        }
        if !self.message.is_empty() {
            egui::Window::new("诊断").show(&ctx, |ui| {
                ui.label(&self.message);
                if ui.button("关闭").clicked() {
                    self.message.clear();
                }
            });
        }
        for action in keyboard.actions(&ctx, self.interaction, canvas_focused) {
            output.game_inputs += self.perform_action(session, action);
        }
        self.shortcuts.finish(&ctx);
        output
    }
}
