//! egui adapter for the P0 gesture owner. Physical input remains a manual check.
use crate::audio_transport::{AudioObservation, Fault, Intent, Phase, State, Transport};
use crate::interaction::{Command, Outcome, Owner, Snapshot};
use eframe::egui;
use p0_preview_contract::{Observation, Probe, Tool};

struct Gesture {
    session: u64,
    base: [f64; 2],
    origin: egui::Pos2,
}
pub struct GestureView {
    owner: Owner,
    gesture: Option<Gesture>,
    probe: Probe,
    tool: Tool,
    zoom: f32,
    target: egui::Rect,
    canvas: egui::Rect,
    audio: Option<Transport>,
    audio_intent: Option<Intent>,
}
impl Default for GestureView {
    fn default() -> Self {
        Self {
            owner: Owner::default(),
            gesture: None,
            probe: Probe::new(0),
            tool: Tool::Edit,
            zoom: 1.,
            target: egui::Rect::NOTHING,
            canvas: egui::Rect::NOTHING,
            audio: None,
            audio_intent: None,
        }
    }
}
impl GestureView {
    pub fn tool(&self) -> Tool {
        self.tool
    }
    pub fn snapshot(&self) -> Snapshot {
        self.owner.committed()
    }
    pub fn node_count(&self) -> usize {
        self.owner.snapshots().len()
    }
    pub fn position(&self) -> [f64; 2] {
        self.owner.visible_position()
    }
    pub fn target(&self) -> egui::Rect {
        self.target
    }
    pub fn canvas(&self) -> egui::Rect {
        self.canvas
    }
    pub fn simulation(&self) -> Observation {
        self.simulation_for(self.audio.as_ref())
    }
    fn simulation_for(&self, audio: Option<&Transport>) -> Observation {
        if let Some(t) = audio {
            let s = t.state();
            let mut result = self.probe.observe();
            result.simulation_ms = s.simulation_us / 1000;
            result.audio_cursor_ms = result.simulation_ms;
            result.paused = s.phase != Phase::Playing;
            result.accepted_game_inputs = s.accepted_inputs;
            return result;
        }
        self.probe.observe()
    }
    pub fn enable_audio(&mut self) {
        let mut transport = Transport::default();
        transport.tool(self.tool == Tool::Interact);
        self.audio = Some(transport);
    }
    pub fn observe_audio(&mut self, o: AudioObservation) -> bool {
        self.audio.as_mut().is_some_and(|a| a.observe(o))
    }
    pub fn audio_state(&self) -> Option<State> {
        self.audio.as_ref().map(Transport::state)
    }
    pub fn take_audio_intent(&mut self) -> Option<Intent> {
        self.audio_intent.take()
    }
    pub fn fail_audio(&mut self) {
        if let Some(a) = &mut self.audio {
            a.fail(Fault::Backend);
        }
    }
    pub fn cancel(&mut self) {
        if let Some(g) = self.gesture.take() {
            self.owner
                .execute(Command::Cancel { session: g.session })
                .unwrap();
        }
    }
    pub fn set_tool(&mut self, tool: Tool) {
        self.cancel();
        self.tool = tool;
        self.probe.select_tool(tool);
        if let Some(a) = &mut self.audio {
            a.tool(tool == Tool::Interact);
        }
    }
    pub fn set_zoom(&mut self, zoom: f32) {
        if zoom.is_finite() && (0.5..=2.).contains(&zoom) {
            self.cancel();
            self.zoom = zoom;
        }
    }
    pub fn pause(&mut self, paused: bool, wall_ms: u64) {
        if let Some(a) = &mut self.audio {
            self.audio_intent = Some(a.intent(paused));
            return;
        }
        self.probe.pause(paused, wall_ms).unwrap();
    }
    pub fn show(&mut self, root: &mut egui::Ui, wall_ms: u64) {
        self.show_impl(root, wall_ms, None);
    }
    pub fn show_composition(
        &mut self,
        root: &mut egui::Ui,
        wall_ms: u64,
        texture: egui::TextureId,
    ) {
        self.show_impl(root, wall_ms, Some(texture));
    }
    fn show_impl(&mut self, root: &mut egui::Ui, wall_ms: u64, texture: Option<egui::TextureId>) {
        let mut owned = self.audio.take();
        let editor_keyboard_owned = root.ctx().text_edit_focused();
        self.show_inner(
            root,
            wall_ms,
            texture,
            owned.as_mut(),
            editor_keyboard_owned,
        );
        self.audio = owned;
    }
    /// Borrow the single preview transport; retain editor ownership across this UI pass.
    pub fn show_transport(
        &mut self,
        root: &mut egui::Ui,
        texture: Option<egui::TextureId>,
        transport: &mut Transport,
        editor_keyboard_owned: bool,
    ) {
        assert!(
            self.audio.is_none(),
            "A view cannot own and borrow audio transports together"
        );
        self.show_inner(root, 0, texture, Some(transport), editor_keyboard_owned);
    }
    fn show_inner(
        &mut self,
        root: &mut egui::Ui,
        wall_ms: u64,
        texture: Option<egui::TextureId>,
        mut audio: Option<&mut Transport>,
        editor_keyboard_owned: bool,
    ) {
        if audio.is_none() {
            self.probe.tick(wall_ms).unwrap();
        }
        let ctx = root.ctx().clone();
        let focused = ctx.input(|i| i.focused);
        self.probe.set_focus(focused);
        if let Some(a) = &mut audio {
            a.focus(focused);
        }
        let interrupted = !focused || ctx.input(|i| i.key_pressed(egui::Key::Escape));
        if interrupted {
            self.cancel();
        }
        egui::Panel::top("gesture-tools").show(root,|ui| {
            ui.horizontal(|ui| {
                let desired=audio.as_deref().map_or(self.probe.observe().paused,Transport::desired_paused);
                if ui.button(if desired {"Resume"} else {"Pause"}).clicked() {if let Some(a)=audio.as_deref_mut(){self.audio_intent=Some(a.intent(!desired));}else{self.pause(!desired,wall_ms);}}
                if ui.selectable_label(self.tool==Tool::Edit,"Edit").clicked() {self.set_tool(Tool::Edit);}
                if ui.selectable_label(self.tool==Tool::Interact,"Interact").clicked() {self.set_tool(Tool::Interact);}
                let mut zoom=self.zoom;
                if ui.add(egui::Slider::new(&mut zoom,0.5..=2.).text("View zoom")).changed() {self.set_zoom(zoom);}
                ui.label("Drag square; Escape cancels; focus/tool/zoom changes cancel");
            });
            ui.horizontal(|ui| {
                for node in self.owner.snapshots().to_vec() {
                    if ui.selectable_label(node.id==self.snapshot().id,format!("Node {} (parent {:?})",node.id,node.parent)).clicked() {
                        self.cancel(); self.owner.execute(Command::Checkout {node:node.id}).unwrap();
                    }
                }
            });
            ui.label(format!("P0 only | simulation {} ms | paused {} | accepted game inputs {} | node {} | draft {:?}",self.simulation_for(audio.as_deref()).simulation_ms,self.simulation_for(audio.as_deref()).paused,self.simulation_for(audio.as_deref()).accepted_game_inputs,self.snapshot().id,self.position()));
        });
        if let Some(a) = &mut audio {
            a.tool(self.tool == Tool::Interact);
            // Ordinary buttons can retain keyboard focus without editing text.
            let consumed = editor_keyboard_owned || ctx.text_edit_focused();
            for event in ctx.input(|i| i.events.clone()) {
                if let egui::Event::Key {
                    key,
                    pressed,
                    repeat,
                    ..
                } = event
                {
                    let index = match key {
                        egui::Key::Z => Some(0),
                        egui::Key::X => Some(1),
                        _ => None,
                    };
                    if let Some(index) = index {
                        a.key(index, pressed, repeat, consumed);
                    }
                }
            }
        }
        egui::CentralPanel::default().show(root, |ui| {
            let canvas = ui.available_rect_before_wrap();
            self.canvas = canvas;
            ui.painter()
                .rect_filled(canvas, 0., egui::Color32::from_rgb(18, 22, 30));
            if let Some(id) = texture {
                ui.painter().image(
                    id,
                    egui::Rect::from_min_size(canvas.min, egui::vec2(1024., 1024.) * self.zoom),
                    egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1., 1.)),
                    egui::Color32::WHITE,
                );
            }
            let origin = if texture.is_some() {
                egui::vec2(40., 40.)
            } else {
                egui::Vec2::ZERO
            };
            let side = if texture.is_some() {
                egui::vec2(102.4, 102.4)
            } else {
                egui::vec2(90., 70.)
            };
            let p = self.position();
            let rect = egui::Rect::from_min_size(
                canvas.min + (egui::vec2(p[0] as f32, p[1] as f32) - origin) * self.zoom,
                side * self.zoom,
            );
            let response = ui.interact(
                rect,
                ui.make_persistent_id("gesture-target"),
                egui::Sense::click_and_drag(),
            );
            if self.tool == Tool::Edit && !interrupted {
                if response.drag_started_by(egui::PointerButton::Primary) {
                    let base = self.snapshot().position;
                    let origin = response.interact_pointer_pos().unwrap()
                        - response.total_drag_delta().unwrap_or_default();
                    if let Outcome::Started(session) = self
                        .owner
                        .execute(Command::Begin {
                            expected_node: self.snapshot().id,
                        })
                        .unwrap()
                    {
                        self.gesture = Some(Gesture {
                            session,
                            base,
                            origin,
                        });
                    }
                }
                if let Some(g) = &self.gesture {
                    if let Some(pointer) = ctx.input(|i| i.pointer.interact_pos()) {
                        let delta = (pointer - g.origin) / self.zoom;
                        // Invalid/nonfinite/out-of-bounds drafts are rejected atomically.
                        let _ = self.owner.execute(Command::Move {
                            session: g.session,
                            position: [
                                g.base[0] + f64::from(delta.x),
                                g.base[1] + f64::from(delta.y),
                            ],
                        });
                    }
                    if response.drag_stopped_by(egui::PointerButton::Primary) {
                        self.owner
                            .execute(Command::Commit { session: g.session })
                            .unwrap();
                        self.gesture = None;
                    } else if !ctx.input(|i| i.pointer.primary_down()) {
                        self.cancel();
                    }
                }
            } else if self.tool == Tool::Interact
                && response.clicked_by(egui::PointerButton::Primary)
            {
                if let Some(a) = &mut audio {
                    a.key(2, true, false, false);
                    a.key(2, false, false, false);
                } else {
                    self.probe.game_press();
                }
            }
            let p = self.position();
            self.target = egui::Rect::from_min_size(
                canvas.min + (egui::vec2(p[0] as f32, p[1] as f32) - origin) * self.zoom,
                side * self.zoom,
            );
            if texture.is_none() {
                ui.painter()
                    .rect_filled(self.target, 0., egui::Color32::from_rgb(65, 160, 215));
            }
            ui.painter().rect_stroke(
                self.target,
                0.,
                egui::Stroke::new(2., egui::Color32::WHITE),
                egui::StrokeKind::Inside,
            );
            if audio.is_some() {
                let x = 600.
                    + (self.simulation_for(audio.as_deref()).simulation_ms % 1000) as f32 * 0.3;
                ui.painter().circle_filled(
                    canvas.min + egui::vec2(x, 25.),
                    8.,
                    egui::Color32::LIGHT_GREEN,
                );
            }
            ui.allocate_space(canvas.size());
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn focused_non_text_control_does_not_steal_playing_preview_keys() {
        let ctx = egui::Context::default();
        let mut view = GestureView::default();
        view.set_tool(Tool::Interact);
        let mut audio = Transport::default();
        audio.observe(AudioObservation {
            epoch: 0,
            request: 0,
            sequence: 1,
            position_us: 0,
            paused: true,
            fault: false,
        });
        let intent = audio.intent(false);
        audio.observe(AudioObservation {
            epoch: 0,
            request: intent.request,
            sequence: 2,
            position_us: 0,
            paused: false,
            fault: false,
        });
        let input = egui::RawInput {
            events: vec![egui::Event::Key {
                key: egui::Key::Z,
                physical_key: Some(egui::Key::Z),
                pressed: true,
                repeat: false,
                modifiers: Default::default(),
            }],
            ..Default::default()
        };
        let _ = ctx.run_ui(input, |ui| {
            // Native button clicks retain keyboard focus, unlike set_tool() alone.
            ui.button("Ordinary toolbar control").request_focus();
            view.show_transport(ui, None, &mut audio, false);
        });
        assert_eq!(audio.state().accepted_inputs, 1);
    }
    #[test]
    fn document_views_borrow_one_transport_without_resetting_clock_keys_or_faults() {
        let ctx = egui::Context::default();
        let mut first = GestureView::default();
        let mut second = GestureView::default();
        let mut audio = Transport::default();
        let observation = |request, sequence, position_us, paused| AudioObservation {
            epoch: 0,
            request,
            sequence,
            position_us,
            paused,
            fault: false,
        };
        audio.intent(false);
        audio.observe(observation(1, 1, 400000, false));
        let key = |pressed, repeat| egui::Event::Key {
            key: egui::Key::Z,
            physical_key: None,
            pressed,
            repeat,
            modifiers: Default::default(),
        };
        let draw = |v: &mut GestureView, t: &mut Transport, events| {
            let _ = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(1000., 600.),
                    )),
                    events,
                    focused: true,
                    ..Default::default()
                },
                |ui| v.show_transport(ui, None, t, false),
            );
        };
        first.set_tool(Tool::Interact);
        draw(&mut first, &mut audio, vec![key(true, false)]);
        assert_eq!(audio.state().accepted_inputs, 1);
        audio.intent(true);
        audio.observe(observation(2, 2, 460000, true));
        draw(&mut second, &mut audio, vec![key(true, true)]);
        let p = second.target().center();
        draw(
            &mut second,
            &mut audio,
            vec![egui::Event::PointerMoved(p), button(p, true)],
        );
        draw(
            &mut second,
            &mut audio,
            vec![egui::Event::PointerMoved(p + egui::vec2(30., 20.))],
        );
        assert_eq!(second.position(), [70., 60.]);
        second.cancel();
        assert_eq!(second.position(), [40., 40.]);
        assert_eq!(first.node_count(), 1);
        assert_eq!(second.node_count(), 1);
        assert_eq!(audio.state().simulation_us, 460000);
        assert_eq!(audio.state().request, 2);
        second.set_tool(Tool::Interact);
        audio.intent(false);
        audio.observe(observation(3, 3, 460000, false));
        draw(&mut second, &mut audio, vec![key(true, true)]);
        assert_eq!(audio.state().accepted_inputs, 1);
        draw(&mut second, &mut audio, vec![key(false, false)]);
        draw(&mut second, &mut audio, vec![key(true, false)]);
        assert_eq!(audio.state().accepted_inputs, 2);
        audio.fail(Fault::Backend);
        draw(&mut first, &mut audio, vec![key(true, true)]);
        assert_eq!(audio.state().phase, Phase::Faulted);
        assert_eq!(audio.state().simulation_us, 460000);
        assert_eq!(audio.state().accepted_inputs, 2);
    }
    #[test]
    fn acknowledged_audio_clock_replaces_wall_time_and_routes_real_widget_keys() {
        let ctx = egui::Context::default();
        let mut v = GestureView::default();
        v.enable_audio();
        v.set_tool(Tool::Interact);
        let sample = |request, sequence, position_us, paused| AudioObservation {
            epoch: 0,
            request,
            sequence,
            position_us,
            paused,
            fault: false,
        };
        v.observe_audio(sample(0, 1, 0, true));
        v.pause(false, 0);
        let intent = v.take_audio_intent().unwrap();
        let z = |pressed, repeat| egui::Event::Key {
            key: egui::Key::Z,
            physical_key: None,
            pressed,
            repeat,
            modifiers: Default::default(),
        };
        run(&ctx, &mut v, vec![z(true, false)], 1000, true);
        assert_eq!(v.simulation().accepted_game_inputs, 0);
        v.observe_audio(sample(intent.request, 2, 100000, false));
        run(&ctx, &mut v, vec![z(true, true)], 2000, true);
        assert_eq!(v.simulation().simulation_ms, 100);
        assert_eq!(v.simulation().accepted_game_inputs, 0);
        run(&ctx, &mut v, vec![z(false, false)], 2010, true);
        run(&ctx, &mut v, vec![z(true, false)], 2020, true);
        assert_eq!(v.simulation().accepted_game_inputs, 1);
        v.pause(true, 2030);
        let intent = v.take_audio_intent().unwrap();
        v.observe_audio(sample(intent.request, 3, 125000, true));
        v.set_tool(Tool::Edit);
        let q = start(&ctx, &mut v, 3000);
        assert_ne!(v.position(), v.snapshot().position);
        run(
            &ctx,
            &mut v,
            vec![egui::Event::PointerMoved(q)],
            10000000,
            true,
        );
        assert_eq!(v.simulation().simulation_ms, 125);
        assert_eq!(v.node_count(), 1);
        assert_eq!(v.simulation().accepted_game_inputs, 1);
        v.cancel();
        v.pause(false, 10000001);
        let intent = v.take_audio_intent().unwrap();
        v.observe_audio(sample(intent.request, 4, 125000, false));
        v.set_tool(Tool::Interact);
        run(&ctx, &mut v, vec![z(true, true)], 10000002, true);
        assert_eq!(v.simulation().accepted_game_inputs, 1);
        run(&ctx, &mut v, vec![z(false, false)], 10000003, true);
        run(&ctx, &mut v, vec![z(true, false)], 10000004, true);
        assert_eq!(v.simulation().accepted_game_inputs, 2);
    }
    fn run(
        ctx: &egui::Context,
        v: &mut GestureView,
        events: Vec<egui::Event>,
        time: u64,
        focused: bool,
    ) {
        let _ = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1000., 600.),
                )),
                events,
                focused,
                time: Some(time as f64 / 1000.),
                ..Default::default()
            },
            |ui| v.show(ui, time),
        );
    }
    fn button(p: egui::Pos2, pressed: bool) -> egui::Event {
        egui::Event::PointerButton {
            pos: p,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: Default::default(),
        }
    }
    fn start(ctx: &egui::Context, v: &mut GestureView, time: u64) -> egui::Pos2 {
        run(ctx, v, vec![], time, true);
        let p = v.target().center();
        run(
            ctx,
            v,
            vec![egui::Event::PointerMoved(p), button(p, true)],
            time + 1,
            true,
        );
        let q = p + egui::vec2(30., 20.);
        run(ctx, v, vec![egui::Event::PointerMoved(q)], time + 2, true);
        q
    }
    #[test]
    fn egui_pointer_route_updates_paused_draft_then_commits_release_position_once() {
        let ctx = egui::Context::default();
        let mut v = GestureView::default();
        v.pause(true, 0);
        v.set_zoom(2.);
        let q = start(&ctx, &mut v, 100);
        assert_eq!(v.position(), [55., 50.]);
        assert_eq!(v.snapshot().position, [40., 40.]);
        assert_eq!(v.node_count(), 1);
        let q = q + egui::vec2(10., 10.);
        run(
            &ctx,
            &mut v,
            vec![egui::Event::PointerMoved(q), button(q, false)],
            200,
            true,
        );
        assert_eq!(v.position(), [60., 55.]);
        assert_eq!(v.snapshot().position, [60., 55.]);
        assert_eq!(v.node_count(), 2);
        run(&ctx, &mut v, vec![], 300, true);
        assert_eq!(v.node_count(), 2);
        assert_eq!(v.simulation().simulation_ms, 0);
        assert_eq!(v.simulation().accepted_game_inputs, 0);
        v.pause(false, 400);
        run(&ctx, &mut v, vec![], 410, true);
        assert_eq!(v.simulation().simulation_ms, 10);
    }
    #[test]
    fn escape_focus_tool_and_zoom_cancellation_never_commit_on_later_release() {
        for reason in 0..4 {
            let ctx = egui::Context::default();
            let mut v = GestureView::default();
            let q = start(&ctx, &mut v, 10);
            assert_ne!(v.position(), v.snapshot().position);
            match reason {
                0 => run(
                    &ctx,
                    &mut v,
                    vec![egui::Event::Key {
                        key: egui::Key::Escape,
                        physical_key: None,
                        pressed: true,
                        repeat: false,
                        modifiers: Default::default(),
                    }],
                    20,
                    true,
                ),
                1 => run(
                    &ctx,
                    &mut v,
                    vec![egui::Event::WindowFocused(false)],
                    20,
                    false,
                ),
                2 => v.set_tool(Tool::Interact),
                _ => v.set_zoom(1.5),
            }
            run(&ctx, &mut v, vec![button(q, false)], 30, true);
            assert_eq!(v.position(), [40., 40.]);
            assert_eq!(v.node_count(), 1);
        }
    }
    #[test]
    fn interact_clicks_are_discarded_during_pause_and_do_not_leak_into_edit_history() {
        let ctx = egui::Context::default();
        let mut v = GestureView::default();
        v.set_tool(Tool::Interact);
        v.pause(true, 0);
        for step in 0..2 {
            let t = 10 + step * 10;
            if step == 1 {
                v.pause(false, t);
            }
            run(&ctx, &mut v, vec![], t, true);
            let p = v.target().center();
            run(
                &ctx,
                &mut v,
                vec![egui::Event::PointerMoved(p), button(p, true)],
                t + 1,
                true,
            );
            run(&ctx, &mut v, vec![button(p, false)], t + 2, true);
            assert_eq!(v.simulation().accepted_game_inputs, step);
            assert_eq!(v.node_count(), 1);
        }
    }
    #[test]
    fn repeated_widget_pass_does_not_replay_keyboard_release_press_pair() {
        let ctx = egui::Context::default();
        let mut v = GestureView::default();
        v.enable_audio();
        v.set_tool(Tool::Interact);
        v.observe_audio(AudioObservation {
            epoch: 0,
            request: 0,
            sequence: 1,
            position_us: 0,
            paused: true,
            fault: false,
        });
        v.pause(false, 0);
        let intent = v.take_audio_intent().unwrap();
        v.observe_audio(AudioObservation {
            epoch: 0,
            request: intent.request,
            sequence: 2,
            position_us: 1000,
            paused: false,
            fault: false,
        });
        let key = |pressed| egui::Event::Key {
            key: egui::Key::Z,
            physical_key: None,
            pressed,
            repeat: false,
            modifiers: Default::default(),
        };
        let mut passes = 0;
        let _ = ctx.run_ui(
            egui::RawInput {
                focused: true,
                events: vec![key(false), key(true)],
                ..Default::default()
            },
            |ui| {
                v.show(ui, 0);
                passes += 1;
                if ui.ctx().current_pass_index() == 0 {
                    ui.ctx().request_discard("repeat physical key route");
                }
            },
        );
        assert_eq!(passes, 2);
        assert_eq!(v.simulation().accepted_game_inputs, 1);
    }
    #[test]
    fn repeated_egui_pass_does_not_restart_a_gesture_or_toggle_pause_twice() {
        let ctx = egui::Context::default();
        let mut v = GestureView::default();
        let mut tick = 0;
        let mut frame = |events: Vec<egui::Event>| {
            tick += 1;
            let _ = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(1000., 600.),
                    )),
                    events,
                    ..Default::default()
                },
                |ui| {
                    v.show(ui, tick);
                    if ui.ctx().current_pass_index() == 0 {
                        ui.ctx().request_discard("exercise repeated widget pass");
                    }
                },
            );
        };
        frame(vec![]);
        let pause = egui::pos2(25., 12.);
        frame(vec![egui::Event::PointerMoved(pause), button(pause, true)]);
        frame(vec![button(pause, false)]);
        assert!(v.simulation().paused);
        let p = v.target().center();
        for (events, time) in [
            (vec![egui::Event::PointerMoved(p), button(p, true)], 10),
            (
                vec![egui::Event::PointerMoved(p + egui::vec2(30., 20.))],
                11,
            ),
        ] {
            let _ = ctx.run_ui(
                egui::RawInput {
                    events,
                    ..Default::default()
                },
                |ui| {
                    v.show(ui, time);
                    if ui.ctx().current_pass_index() == 0 {
                        ui.ctx().request_discard("repeat drag start");
                    }
                },
            );
        }
        let q = p + egui::vec2(30., 20.);
        assert_eq!(v.position(), [70., 60.]);
        let _ = ctx.run_ui(
            egui::RawInput {
                events: vec![egui::Event::PointerMoved(q + egui::vec2(10., 0.))],
                ..Default::default()
            },
            |ui| {
                v.show(ui, 20);
                if ui.ctx().current_pass_index() == 0 {
                    ui.ctx().request_discard("repeat active drag");
                }
            },
        );
        assert_eq!(v.position(), [80., 60.]);
        assert_eq!(v.node_count(), 1);
        run(&ctx, &mut v, vec![button(q, false)], 21, true);
        assert_eq!(v.node_count(), 2);
    }
}
