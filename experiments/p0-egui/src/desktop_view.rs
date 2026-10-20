//! P0 desktop acceptance surface; synthetic clock, no device audio or project I/O.
use crate::{
    audio_transport::{AudioObservation, Intent, Phase, Transport},
    interaction_view::GestureView,
};
use eframe::egui;
use p0_preview_contract::{Probe, Tool};
use serde_json::{Value, json};

pub const PHRASE: &str = "皮肤 日本語 zx";
const TEXT_ID: &str = "desktop-ime-field";
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layout {
    Side,
    Bottom,
}
struct Clock {
    probe: Probe,
    request: u64,
    sequence: u64,
}
impl Default for Clock {
    fn default() -> Self {
        let mut probe = Probe::new(0);
        probe.pause(true, 0).unwrap();
        Self {
            probe,
            request: 0,
            sequence: 0,
        }
    }
}
impl Clock {
    fn control(&mut self, i: Intent, wall: u64) {
        if i.epoch == 0 && i.request > self.request {
            self.probe.pause(i.paused, wall).unwrap();
            self.request = i.request;
        }
    }
    fn poll(&mut self, wall: u64) -> AudioObservation {
        self.probe.tick(wall).unwrap();
        self.sequence += 1;
        let s = self.probe.observe();
        AudioObservation {
            epoch: 0,
            request: self.request,
            sequence: self.sequence,
            position_us: s.simulation_ms * 1000,
            paused: s.paused,
            fault: false,
        }
    }
}
pub struct DesktopView {
    pub gesture: GestureView,
    audio: Transport,
    clock: Clock,
    text: String,
    layout: Layout,
    visited: [bool; 2],
    last_frame: Option<u64>,
    focused: Option<bool>,
    composition: bool,
    pub fullscreen_target: Option<bool>,
    fullscreen: Option<bool>,
    fullscreen_entered: u64,
    fullscreen_returned: u64,
    key_presses: u64,
    pointer_presses: u64,
    focus_lost: u64,
    focus_returned: u64,
    focus_cancelled: u64,
    escape_cancelled: u64,
    keyboard_game_accepts: u64,
    ime_preedit: u64,
    ime_commit: u64,
    text_game_leaks: u64,
    text_zx: u64,
    text_zx_playing: u64,
    drop_accepted: u64,
    drop_rejected: u64,
    scales: Vec<Value>,
    last_scale: Option<(f32, f32)>,
    errors: Vec<String>,
    pub finish_requested: bool,
    pub checkpoint_requested: bool,
    human_font: bool,
    human_ime: bool,
    human_chinese_ime: bool,
    human_japanese_ime: bool,
    human_drop: bool,
    human_fullscreen: bool,
    japanese_unavailable: bool,
    dpi_unavailable: bool,
}
impl Default for DesktopView {
    fn default() -> Self {
        Self {
            gesture: GestureView::default(),
            audio: Transport::default(),
            clock: Clock::default(),
            text: String::new(),
            layout: Layout::Side,
            visited: [true, false],
            last_frame: None,
            focused: None,
            composition: false,
            fullscreen_target: None,
            fullscreen: None,
            fullscreen_entered: 0,
            fullscreen_returned: 0,
            key_presses: 0,
            pointer_presses: 0,
            focus_lost: 0,
            focus_returned: 0,
            focus_cancelled: 0,
            escape_cancelled: 0,
            keyboard_game_accepts: 0,
            ime_preedit: 0,
            ime_commit: 0,
            text_game_leaks: 0,
            text_zx: 0,
            text_zx_playing: 0,
            drop_accepted: 0,
            drop_rejected: 0,
            scales: vec![],
            last_scale: None,
            errors: vec![],
            finish_requested: false,
            checkpoint_requested: false,
            human_font: false,
            human_ime: false,
            human_chinese_ime: false,
            human_japanese_ime: false,
            human_drop: false,
            human_fullscreen: false,
            japanese_unavailable: false,
            dpi_unavailable: false,
        }
    }
}
impl DesktopView {
    pub fn set_layout(&mut self, layout: Layout) {
        if layout != self.layout {
            self.gesture.cancel();
            self.layout = layout;
        }
        self.visited[usize::from(layout == Layout::Bottom)] = true;
    }
    pub fn drop_result(&mut self, accepted: bool) {
        if accepted {
            self.drop_accepted += 1;
        } else {
            self.drop_rejected += 1;
        }
    }
    pub fn error(&mut self, error: String) {
        if self.errors.len() < 8 {
            self.errors.push(error);
        }
    }
    pub fn control(&mut self, paused: bool, wall: u64) {
        let i = self.audio.intent(paused);
        self.clock.control(i, wall);
    }
    pub fn show(
        &mut self,
        root: &mut egui::Ui,
        wall: u64,
        drop_folder: &str,
        texture: Option<egui::TextureId>,
    ) {
        self.audio.observe(self.clock.poll(wall));
        let ctx = root.ctx().clone();
        // IME cancellation can clear composition and TextEdit focus in this pass.
        // The same Escape still belongs to the editor that owned it on entry.
        let editor_was_active =
            self.composition || ctx.memory(|m| m.has_focus(egui::Id::unique(TEXT_ID)));
        let frame = ctx.cumulative_frame_nr();
        let fresh = self.last_frame != Some(frame);
        self.last_frame = Some(frame);
        let before_inputs = self.audio.state().accepted_inputs;
        let draft = self.gesture.position() != self.gesture.snapshot().position;
        let focused = ctx.input(|i| i.focused);
        let mut f11 = false;
        let mut escape = false;
        let mut text_key_count = 0;
        if fresh {
            if self.focused == Some(true) && !focused {
                self.focus_lost += 1;
                self.composition = false;
            }
            if self.focused == Some(false) && focused {
                self.focus_returned += 1;
            }
            self.focused = Some(focused);
            for e in ctx.input(|i| i.events.clone()) {
                match e {
                    egui::Event::Key {
                        key,
                        pressed: true,
                        repeat: false,
                        ..
                    } => {
                        if matches!(key, egui::Key::Z | egui::Key::X) {
                            self.key_presses += 1;
                            text_key_count += 1;
                        }
                        f11 |= key == egui::Key::F11;
                        escape |= key == egui::Key::Escape;
                    }
                    egui::Event::PointerButton { pressed: true, .. } => self.pointer_presses += 1,
                    egui::Event::Ime(egui::ImeEvent::Preedit { text, .. }) => {
                        self.composition = !text.is_empty();
                        if self.composition {
                            self.ime_preedit += 1;
                        }
                    }
                    egui::Event::Ime(egui::ImeEvent::Commit(_)) => {
                        self.composition = false;
                        self.ime_commit += 1;
                    }
                    _ => {}
                }
            }
            let full = ctx.input(|i| i.viewport().fullscreen);
            self.fullscreen = full;
            if full.is_some() && full == self.fullscreen_target {
                if full == Some(true) {
                    self.fullscreen_entered += 1;
                } else {
                    self.fullscreen_returned += 1;
                }
                self.fullscreen_target = None;
            }
            let native = ctx.native_pixels_per_point();
            let effective = ctx.pixels_per_point();
            if let Some(native) = native.filter(|n| n.is_finite() && *n > 0.)
                && effective.is_finite()
                && effective > 0.
                && self.last_scale != Some((native, effective))
                && self.scales.len() < 64
            {
                self.scales.push(json!({"native_pixels_per_point":native,"effective_pixels_per_point":effective,"ui_zoom":ctx.zoom_factor()}));
                self.last_scale = Some((native, effective));
            }
        }
        let mut finish = false;
        let mut save = false;
        let mut selected = self.layout;
        let mut panel = |ui: &mut egui::Ui| {
            ui.heading("Windows 桌面验收 / P0");
            ui.label("合成时钟，无音频；窗口事件与人工确认单独记录。");
            ui.horizontal(|ui| {
                ui.selectable_value(&mut selected, Layout::Side, "A 左侧");
                ui.selectable_value(&mut selected, Layout::Bottom, "B 底部");
            });
            egui::ScrollArea::vertical().max_height(460.).show(ui,|ui|{
                ui.label("1. 暂停 + Edit：拖动白框并释放；再拖动，按 Esc 取消。");
                ui.label("2. 拖动未松开时 Alt+Tab，松手后返回：草稿应取消。");
                ui.label("3. Interact + Resume：在预览区按 Z/X；输入框内输入不应增加游戏计数。");
                ui.label(format!("4. 用输入法输入：{PHRASE}（请勿粘贴来替代 IME）"));
                ui.add(egui::TextEdit::singleline(&mut self.text).id(egui::Id::unique(TEXT_ID)).char_limit(64).hint_text("在此测试中文、日文及 zx"));
                ui.label("只保存字数、目标匹配和 IME 次数，不保存输入文本。");
                ui.label(format!("IME preedit {} / commit {} | 输入框 Z/X {} | 游戏泄漏 {}",self.ime_preedit,self.ime_commit,self.text_zx,self.text_game_leaks));
                ui.label("5. F11 进/出全屏；输入框外按 Esc 返回。IME 候选时 Esc 不应退出全屏。");
                ui.label("6. 将此目录中的“素材 日本語.png”拖入窗口：");
                ui.monospace(drop_folder);
                if ui.button("复制测试目录路径").clicked(){ui.ctx().copy_text(drop_folder.to_owned());}
                ui.label(format!("拖放 接受 {} / 拒绝 {}",self.drop_accepted,self.drop_rejected));
                if let Some(texture)=texture {ui.image((texture,egui::vec2(96.,48.)));}
                ui.label("7. 调整窗口尺寸，比较 A/B；如有不同 DPI 的屏幕，移动窗口。UI 缩放不替代原生 DPI 验收。");
                let mut zoom=ctx.zoom_factor();if ui.add(egui::Slider::new(&mut zoom,0.75..=1.5).text("UI 缩放")).changed(){self.gesture.cancel();ctx.set_zoom_factor(zoom);}
                ui.checkbox(&mut self.human_font,"人工确认：中文/日文文字清晰，无方框");
                ui.checkbox(&mut self.human_ime,"人工确认：候选、提交与 Esc 正常");
                ui.checkbox(&mut self.human_chinese_ime,"已实际用中文输入法验证");
                if ui.checkbox(&mut self.human_japanese_ime,"已实际用日文输入法验证").changed() && self.human_japanese_ime {self.japanese_unavailable=false;}
                ui.checkbox(&mut self.human_drop,"人工确认：拖放后缩略图正确");
                ui.checkbox(&mut self.human_fullscreen,"人工确认：全屏返回后布局和操作正常");
                if ui.checkbox(&mut self.japanese_unavailable,"本机无日文输入法，日文 IME 暂未验收").changed() && self.japanese_unavailable {self.human_japanese_ime=false;}
                ui.checkbox(&mut self.dpi_unavailable,"无不同 DPI 环境，跨 DPI 暂未验收");
                ui.horizontal(|ui|{save=ui.button("保存进度").clicked();finish=ui.button("记录并结束").clicked();});
            });
        };
        match self.layout {
            Layout::Side => {
                egui::Panel::left("desktop-checks")
                    .default_size(340.)
                    .resizable(true)
                    .show(root, &mut panel);
            }
            Layout::Bottom => {
                egui::Panel::bottom("desktop-checks-bottom")
                    .default_size(260.)
                    .resizable(true)
                    .show(root, &mut panel);
            }
        }
        let editor_consumed = editor_was_active || ctx.text_edit_focused();
        self.set_layout(selected);
        if fresh
            && focused
            && (f11
                || (escape
                    && self.fullscreen == Some(true)
                    && !editor_consumed
                    && !editor_was_active
                    && !self.composition))
        {
            self.gesture.cancel();
            let target = if f11 {
                !self.fullscreen_target.or(self.fullscreen).unwrap_or(false)
            } else {
                false
            };
            self.fullscreen_target = Some(target);
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(target));
        }
        self.gesture
            .show_transport(root, None, &mut self.audio, editor_consumed);
        if let Some(i) = self.gesture.take_audio_intent() {
            self.clock.control(i, wall);
        }
        if fresh {
            if editor_consumed {
                self.text_zx += text_key_count;
                if self.audio.state().phase == Phase::Playing
                    && self.gesture.tool() == Tool::Interact
                {
                    self.text_zx_playing += text_key_count;
                }
                self.text_game_leaks += self.audio.state().accepted_inputs - before_inputs;
            } else if text_key_count > 0 {
                self.keyboard_game_accepts += self.audio.state().accepted_inputs - before_inputs;
            }
            if !focused && draft && self.gesture.position() == self.gesture.snapshot().position {
                self.focus_cancelled += 1;
            }
            if escape && draft && self.gesture.position() == self.gesture.snapshot().position {
                self.escape_cancelled += 1;
            }
        }
        self.finish_requested |= finish;
        self.checkpoint_requested |= save;
        if self.audio.state().phase == Phase::Playing {
            ctx.request_repaint_after(std::time::Duration::from_millis(16));
        }
    }
    pub fn report(&self, source: &str) -> Value {
        let s = self.audio.state();
        json!({"schema_version":1,"status":"recorded","source":source,
            "clock":"synthetic desktop fixture, not device audio or gameplay","simulation_us":s.simulation_us,"paused":s.phase!=Phase::Playing,
            "history_nodes":self.gesture.node_count(),"position":self.gesture.position(),"draft_active":self.gesture.position()!=self.gesture.snapshot().position,
            "accepted_game_inputs":s.accepted_inputs,"native_event_observations":{"zx_presses":self.key_presses,"pointer_presses":self.pointer_presses,
                "focus_lost":self.focus_lost,"focus_returned":self.focus_returned,"focus_cancelled_draft":self.focus_cancelled,"escape_cancelled_draft":self.escape_cancelled,
                "ime_preedit":self.ime_preedit,"ime_commit":self.ime_commit,"text_zx_presses":self.text_zx,"text_zx_while_playing":self.text_zx_playing,"keyboard_game_accepts":self.keyboard_game_accepts,"text_game_input_leaks":self.text_game_leaks,
                "fullscreen_entered":self.fullscreen_entered,"fullscreen_returned":self.fullscreen_returned,"drop_accepted":self.drop_accepted,"drop_rejected":self.drop_rejected},
            "text_summary":{"characters":self.text.chars().count(),"matches_fixture":self.text==PHRASE},
            "layouts_visited":self.visited,"scale_observations":self.scales,"pending_fullscreen":self.fullscreen_target,
            "human_confirmations":{"font_legibility":self.human_font,"ime_candidate_commit_escape":self.human_ime,"chinese_ime":self.human_chinese_ime,"japanese_ime":self.human_japanese_ime,"drop_thumbnail":self.human_drop,"fullscreen_return":self.human_fullscreen},
            "unavailable":{"japanese_ime":self.japanese_unavailable,"cross_native_dpi":self.dpi_unavailable},"errors":self.errors,
            "limits":["Native events need manual observation; smoke and synthetic tests cannot certify physical input or IME",
                "Two lightweight layouts only; no product UI decision, import Command or asset library implemented",
                "Native DPI and UI zoom are separate; one native scale does not prove DPI transitions",
                "No global keys, arbitrary text, foreign file bytes, system capture, audio latency or stable validation"]})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn raw(events: Vec<egui::Event>, focused: bool) -> egui::RawInput {
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1100., 760.),
            )),
            events,
            focused,
            ..Default::default()
        }
    }
    fn draw(ctx: &egui::Context, v: &mut DesktopView, input: egui::RawInput, wall: u64) {
        let _ = ctx.run_ui(input, |ui| v.show(ui, wall, "fixture only", None));
    }
    fn key(key: egui::Key, pressed: bool) -> egui::Event {
        egui::Event::Key {
            key,
            physical_key: None,
            pressed,
            repeat: false,
            modifiers: Default::default(),
        }
    }
    #[test]
    fn paused_clock_never_catches_up_and_report_does_not_persist_typed_text() {
        let mut c = Clock::default();
        assert_eq!(c.poll(10000).position_us, 0);
        c.control(
            Intent {
                epoch: 0,
                request: 1,
                paused: false,
            },
            10001,
        );
        assert_eq!(c.poll(10002).position_us, 1000);
        let mut v = DesktopView {
            text: "private accidental paste".into(),
            ..Default::default()
        };
        v.drop_result(false);
        let s = v.report("synthetic").to_string();
        assert!(!s.contains("private accidental"));
        assert!(!s.contains("passed"));
    }
    #[test]
    fn ime_and_editor_keys_do_not_reach_playing_preview_or_exit_fullscreen() {
        let ctx = egui::Context::default();
        let mut v = DesktopView::default();
        v.gesture.set_tool(Tool::Interact);
        v.control(false, 0);
        draw(&ctx, &mut v, raw(vec![], true), 1);
        ctx.memory_mut(|m| m.request_focus(egui::Id::unique(TEXT_ID)));
        let mut input = raw(
            vec![
                egui::Event::Ime(egui::ImeEvent::Preedit {
                    text: "pi".into(),
                    active_range_chars: None,
                }),
                key(egui::Key::Z, true),
            ],
            true,
        );
        input
            .viewports
            .get_mut(&egui::ViewportId::ROOT)
            .unwrap()
            .fullscreen = Some(true);
        draw(&ctx, &mut v, input, 2);
        assert_eq!(v.audio.state().accepted_inputs, 0);
        assert_eq!(v.ime_preedit, 1);
        let mut input = raw(
            vec![
                egui::Event::Ime(egui::ImeEvent::Preedit {
                    text: String::new(),
                    active_range_chars: None,
                }),
                key(egui::Key::Escape, true),
                key(egui::Key::X, true),
            ],
            true,
        );
        input
            .viewports
            .get_mut(&egui::ViewportId::ROOT)
            .unwrap()
            .fullscreen = Some(true);
        draw(&ctx, &mut v, input, 3);
        assert_eq!(v.fullscreen_target, None);
        assert_eq!(v.audio.state().accepted_inputs, 0);
        draw(
            &ctx,
            &mut v,
            raw(
                vec![egui::Event::Ime(egui::ImeEvent::Commit(PHRASE.into()))],
                true,
            ),
            4,
        );
        assert_eq!(v.ime_commit, 1);
        assert_eq!(v.text_game_leaks, 0);
    }
    #[test]
    fn focus_loss_and_layout_change_cancel_drafts_without_history() {
        let ctx = egui::Context::default();
        let mut v = DesktopView::default();
        draw(&ctx, &mut v, raw(vec![], true), 0);
        let button = |p, pressed| egui::Event::PointerButton {
            pos: p,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: Default::default(),
        };
        let p = v.gesture.target().center();
        draw(
            &ctx,
            &mut v,
            raw(vec![egui::Event::PointerMoved(p), button(p, true)], true),
            1,
        );
        draw(
            &ctx,
            &mut v,
            raw(
                vec![egui::Event::PointerMoved(p + egui::vec2(30., 20.))],
                true,
            ),
            2,
        );
        assert_ne!(v.gesture.position(), v.gesture.snapshot().position);
        draw(&ctx, &mut v, raw(vec![], false), 3);
        assert_eq!(v.focus_cancelled, 1);
        assert_eq!(v.gesture.node_count(), 1);
        v.set_layout(Layout::Bottom);
        assert_eq!(v.visited, [true, true]);
        assert_eq!(v.gesture.node_count(), 1);
    }
    #[test]
    fn fullscreen_requires_native_ack_and_scale_report_keeps_native_and_ui_separate() {
        let ctx = egui::Context::default();
        let mut v = DesktopView::default();
        let mut input = raw(vec![key(egui::Key::F11, true)], true);
        input
            .viewports
            .get_mut(&egui::ViewportId::ROOT)
            .unwrap()
            .fullscreen = Some(false);
        input
            .viewports
            .get_mut(&egui::ViewportId::ROOT)
            .unwrap()
            .native_pixels_per_point = Some(1.25);
        draw(&ctx, &mut v, input, 0);
        assert_eq!(v.fullscreen_entered, 0);
        assert_eq!(v.fullscreen_target, Some(true));
        let mut input = raw(vec![key(egui::Key::F11, false)], true);
        input
            .viewports
            .get_mut(&egui::ViewportId::ROOT)
            .unwrap()
            .fullscreen = Some(true);
        input
            .viewports
            .get_mut(&egui::ViewportId::ROOT)
            .unwrap()
            .native_pixels_per_point = Some(1.25);
        draw(&ctx, &mut v, input, 1);
        assert_eq!(v.fullscreen_entered, 1);
        ctx.set_zoom_factor(1.5);
        let mut input = raw(vec![], true);
        input
            .viewports
            .get_mut(&egui::ViewportId::ROOT)
            .unwrap()
            .native_pixels_per_point = Some(1.25);
        draw(&ctx, &mut v, input, 2);
        assert!(v.scales.iter().any(|s| s["ui_zoom"] == 1.5));
        assert_eq!(v.gesture.node_count(), 1);
    }
}
