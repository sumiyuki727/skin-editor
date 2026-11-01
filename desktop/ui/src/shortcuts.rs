//! Application-owned bindings and modifier policies. Native text editing and
//! OS shortcuts remain owned by egui/the platform, never by this dispatcher.
use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EditorAction {
    Cancel,
    DeleteSelection,
    Undo,
    Redo,
    GamePrimary,
    GameSecondary,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShortcutScope {
    Editing,
    GameCanvas,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Chord {
    Any,
    Plain,
    Primary,
    PrimaryShift,
}
#[derive(Clone, Copy, Debug)]
pub struct ShortcutBinding {
    pub action: EditorAction,
    pub scope: ShortcutScope,
    pub display: &'static str,
    pub description: &'static str,
    key: egui::Key,
    chord: Chord,
}
use Chord::*;
use EditorAction::*;
use ShortcutScope::*;

/// One source for routing and the settings readout. No platform hook, project
/// command payload, persistence or user remapping is implied by this table.
pub const DEFAULT_SHORTCUTS: &[ShortcutBinding] = &[
    ShortcutBinding {
        action: Cancel,
        scope: Editing,
        key: egui::Key::Escape,
        chord: Any,
        display: "Esc",
        description: "取消本次编辑；空闲时解除画布选择",
    },
    ShortcutBinding {
        action: DeleteSelection,
        scope: Editing,
        key: egui::Key::Delete,
        chord: Plain,
        display: "Delete",
        description: "删除选中的图层／视觉组件，保留原始素材",
    },
    ShortcutBinding {
        action: Undo,
        scope: Editing,
        key: egui::Key::Z,
        chord: Primary,
        display: "Ctrl／Command + Z",
        description: "撤销项目操作",
    },
    ShortcutBinding {
        action: Redo,
        scope: Editing,
        key: egui::Key::Z,
        chord: PrimaryShift,
        display: "Ctrl／Command + Shift + Z",
        description: "重做项目操作",
    },
    ShortcutBinding {
        action: Redo,
        scope: Editing,
        key: egui::Key::Y,
        chord: Primary,
        display: "Ctrl／Command + Y",
        description: "重做项目操作（替代键）",
    },
    ShortcutBinding {
        action: GamePrimary,
        scope: GameCanvas,
        key: egui::Key::Z,
        chord: Plain,
        display: "Z",
        description: "交互模式下的占位游戏输入 1",
    },
    ShortcutBinding {
        action: GameSecondary,
        scope: GameCanvas,
        key: egui::Key::X,
        chord: Plain,
        display: "X",
        description: "交互模式下的占位游戏输入 2",
    },
];
pub const MODIFIER_HINTS: &[(&str, &str)] = &[
    ("Ctrl／Command + 点击", "追加或移除画布／面板选项"),
    ("Shift + 拖角点", "保持缩放比例"),
];
pub const TRANSFORM_HINT: &str = "画布平移；角点缩放（Shift 等比）；圆柄旋转；Esc 取消。";
pub const SELECTION_HINT: &str =
    "Ctrl／Command 点击图层或组件可多选；拖动中 Esc 取消本次，空闲时 Esc 解除选择。";

pub(super) fn additive(m: egui::Modifiers) -> bool {
    (m.command || m.ctrl) && !m.alt
}
pub(super) fn proportional(m: egui::Modifiers) -> bool {
    m.shift
}
impl Chord {
    fn matches(self, m: egui::Modifiers) -> bool {
        if self == Any {
            return true;
        }
        if m.alt {
            return false;
        }
        let primary = m.command || m.ctrl || m.mac_cmd;
        match self {
            Any => true,
            Plain => !primary && !m.shift,
            Primary => primary && !m.shift,
            PrimaryShift => primary && m.shift,
        }
    }
}
#[derive(Default)]
pub(super) struct Router {
    frame: Option<u64>,
    composing: bool,
    text_owned: bool,
}
pub(super) struct FrameInput {
    bindings: Vec<usize>,
    blocked: bool,
    text_owned: bool,
}
impl Router {
    pub fn begin(&mut self, ctx: &egui::Context) -> FrameInput {
        let mut frame = FrameInput {
            bindings: Vec::new(),
            blocked: true,
            text_owned: self.text_owned || ctx.text_edit_focused(),
        };
        if self.frame == Some(ctx.cumulative_frame_nr()) {
            return frame;
        }
        self.frame = Some(ctx.cumulative_frame_nr());
        frame.blocked = self.composing;
        ctx.input(|i| {
            if !i.focused {
                self.composing = false;
                frame.blocked = true;
                return;
            }
            for e in &i.events {
                if let egui::Event::Ime(ime) = e {
                    // Quarantine the entire commit/dismissal frame as well.
                    frame.blocked = true;
                    #[allow(deprecated)]
                    match ime {
                        egui::ImeEvent::Preedit { text, .. } => self.composing = !text.is_empty(),
                        egui::ImeEvent::Commit(_) => self.composing = false,
                        egui::ImeEvent::Disabled => self.composing = false,
                        _ => {}
                    }
                }
                if let egui::Event::Key {
                    key,
                    modifiers,
                    pressed: true,
                    repeat: false,
                    ..
                } = e
                    && let Some(n) = DEFAULT_SHORTCUTS
                        .iter()
                        .position(|b| b.key == *key && b.chord.matches(*modifiers))
                    && !frame.bindings.contains(&n)
                {
                    frame.bindings.push(n);
                }
            }
        });
        frame
    }
    pub fn finish(&mut self, ctx: &egui::Context) {
        self.text_owned = ctx.text_edit_focused();
    }
}
impl FrameInput {
    /// Escape must invalidate the gesture before a simultaneous pointer release.
    /// An owned numeric draft can also cancel, unless IME owns this frame.
    pub fn early_cancel(&mut self, ctx: &egui::Context, editing: bool, owned_draft: bool) -> bool {
        if !editing || self.blocked || (self.text_owned && !owned_draft) {
            return false;
        }
        let Some(at) = self
            .bindings
            .iter()
            .position(|n| DEFAULT_SHORTCUTS[*n].action == Cancel)
        else {
            return false;
        };
        let n = self.bindings.remove(at);
        self.consume(ctx, n);
        self.bindings.clear();
        true
    }
    fn consume(&self, ctx: &egui::Context, n: usize) {
        let b = DEFAULT_SHORTCUTS[n];
        ctx.input_mut(|i| i.events.retain(|e| !matches!(e,
            egui::Event::Key { key, modifiers, pressed: true, .. } if *key == b.key && b.chord.matches(*modifiers))));
    }
    pub fn actions(
        &self,
        ctx: &egui::Context,
        interaction: bool,
        canvas_focused: bool,
    ) -> Vec<EditorAction> {
        if self.blocked || self.text_owned || ctx.text_edit_focused() {
            return vec![];
        }
        let mut actions = Vec::new();
        for n in &self.bindings {
            let b = DEFAULT_SHORTCUTS[*n];
            if match b.scope {
                Editing => !interaction,
                GameCanvas => interaction && canvas_focused,
            } {
                self.consume(ctx, *n);
                if !actions.contains(&b.action) {
                    actions.push(b.action);
                }
                // At most one project/navigation action in a GUI frame.
                if b.scope == Editing {
                    break;
                }
            }
        }
        actions
    }
}
impl EditorUi {
    pub(super) fn sync_interaction_mode(&mut self, session: &mut ProjectSession) {
        if self.interaction && (!self.previous_interaction || !self.selected.is_empty()) {
            self.select(session, None);
        }
        self.previous_interaction = self.interaction;
    }
    /// Buttons and shortcuts use the same context-checked action path.
    pub(super) fn perform_action(
        &mut self,
        session: &mut ProjectSession,
        action: EditorAction,
    ) -> u32 {
        match action {
            GamePrimary | GameSecondary => u32::from(self.interaction),
            _ if self.interaction => 0,
            Cancel => {
                if self.owned_draft.is_some() {
                    self.cancel_owned(session);
                } else {
                    self.select(session, None);
                }
                0
            }
            Undo => {
                self.navigation(session, HistoryAction::Undo);
                0
            }
            Redo => {
                self.navigation(session, HistoryAction::Redo { child: None });
                0
            }
            DeleteSelection => {
                if Self::draft_state(session).is_some() {
                    self.message = "请先完成或取消当前编辑".into();
                    return 0;
                }
                let commands: Vec<_> =
                    selection::effective(&Self::project(session), &self.selected)
                        .into_iter()
                        .map(|item| match item {
                            Selection::Layer(document, layer) => {
                                Command::RemoveLayer { document, layer }
                            }
                            Selection::Component(component) => {
                                Command::RemoveComponent { component }
                            }
                        })
                        .collect();
                if !commands.is_empty() && self.submit(session, commands) {
                    self.select(session, None);
                }
                0
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_bindings_never_conflict_in_the_same_input_context() {
        for flags in 0..32 {
            let m = egui::Modifiers {
                alt: flags & 1 != 0,
                ctrl: flags & 2 != 0,
                shift: flags & 4 != 0,
                mac_cmd: flags & 8 != 0,
                command: flags & 16 != 0,
            };
            for b in DEFAULT_SHORTCUTS {
                assert!(
                    DEFAULT_SHORTCUTS
                        .iter()
                        .filter(|other| other.scope == b.scope
                            && other.key == b.key
                            && other.chord.matches(m))
                        .count()
                        <= 1
                );
            }
        }
    }
}
