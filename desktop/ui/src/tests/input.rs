use super::*;
use joint_selection::{click, pair};

fn press(h: &mut Harness, key: egui::Key, modifiers: egui::Modifiers, repeat: bool) -> UiOutput {
    h.modifiers = modifiers;
    h.pass(
        vec![egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat,
            modifiers,
        }],
        [1180., 800.],
        true,
    )
}
fn release(h: &mut Harness, key: egui::Key) {
    h.pass(
        vec![egui::Event::Key {
            key,
            physical_key: None,
            pressed: false,
            repeat: false,
            modifiers: h.modifiers,
        }],
        [1180., 800.],
        true,
    );
}
fn primary() -> egui::Modifiers {
    egui::Modifiers {
        ctrl: true,
        command: true,
        ..Default::default()
    }
}
fn point(h: &Harness, p: [f64; 2]) -> Pos2 {
    let canvas = h.ui.canvas_rect.unwrap();
    let view = h.presenter.0.unwrap();
    let p = view.view.point(p[0], p[1]).unwrap();
    canvas.min
        + Vec2::new(
            p[0] as f32 / view.frame.source.context.size[0] as f32 * canvas.width(),
            p[1] as f32 / view.frame.source.context.size[1] as f32 * canvas.height(),
        )
}
fn select_at(h: &mut Harness, p: [f64; 2]) {
    let p = point(h, p);
    h.pointer(p, Some(true));
    h.pointer(p, Some(false));
}
fn move_layer(h: &mut Harness, s: Selection, x: f64) {
    assert!(h.ui.submit(
        &mut h.session,
        vec![EditorUi::transform_command(
            s,
            Affine2::new([1., 0., 0., 1., x, 128.]).unwrap()
        )]
    ));
    h.pass(vec![], [1180., 800.], true);
}

#[test]
fn delete_is_atomic_keeps_assets_and_uses_the_button_action_with_undo_redo() {
    let (mut h, a, b) = pair(false, 1024);
    h.ui.toggle_selection(&mut h.session, b);
    let before = h.session.snapshot();
    let n = h.session.history_len();
    press(&mut h, egui::Key::Delete, Default::default(), false);
    assert!(h.ui.selections().is_empty());
    assert_eq!(h.session.history_len(), n + 1);
    assert_eq!(
        h.session.snapshot().project.assets(),
        before.project.assets()
    );
    for s in [a, b] {
        assert!(EditorUi::transform(&h.session.snapshot().project, s).is_none());
    }
    press(&mut h, egui::Key::Z, primary(), false);
    assert_eq!(h.session.snapshot().project, before.project);
    release(&mut h, egui::Key::Z);
    press(
        &mut h,
        egui::Key::Z,
        egui::Modifiers {
            shift: true,
            ..primary()
        },
        false,
    );
    assert!(EditorUi::transform(&h.session.snapshot().project, a).is_none());
    release(&mut h, egui::Key::Z);
    press(&mut h, egui::Key::Z, primary(), false);
    h.ui.select(&mut h.session, Some(a));
    h.pass(vec![], [1180., 800.], true);
    click(&mut h, "删除选中项");
    assert!(EditorUi::transform(&h.session.snapshot().project, a).is_none());
    assert!(EditorUi::transform(&h.session.snapshot().project, b).is_some());
}
#[test]
fn delete_limit_failure_and_busy_leave_selection_project_and_history_intact() {
    let (mut h, a, b) = pair(false, 1);
    h.ui.toggle_selection(&mut h.session, b);
    let before = h.session.snapshot();
    let n = h.session.history_len();
    press(&mut h, egui::Key::Delete, Default::default(), false);
    assert_eq!(h.session.snapshot(), before);
    assert_eq!(h.session.history_len(), n);
    assert_eq!(h.ui.selections(), [a, b]);
    assert!(!h.ui.message.is_empty());
    release(&mut h, egui::Key::Delete);
    let req = DraftRequest {
        api_version: API_VERSION,
        session_id: before.session_id,
        request_id: RequestId::new(99000).unwrap(),
        action: DraftAction::Begin {
            expected: before.state,
        },
    };
    h.session.edit_draft(req).unwrap();
    let draft = EditorUi::draft_state(&h.session);
    press(&mut h, egui::Key::Delete, Default::default(), false);
    assert_eq!(EditorUi::draft_state(&h.session), draft);
    assert_eq!(h.session.snapshot(), before);
    assert_eq!(h.ui.selections(), [a, b]);
}
#[test]
fn ancestor_delete_is_once_and_undo_restores_its_subtree() {
    let mut h = gesture_controls::grouped(1);
    let group = h.ui.selection().unwrap();
    let Selection::Layer(d, _) = group else {
        unreachable!()
    };
    let child = h
        .session
        .snapshot()
        .project
        .document(d)
        .unwrap()
        .layers()
        .iter()
        .find(|n| matches!(n.kind, LayerKind::Image { .. }))
        .unwrap()
        .id;
    h.ui.toggle_selection(&mut h.session, Selection::Layer(d, child));
    let before = h.session.snapshot().project;
    let n = h.session.history_len();
    press(&mut h, egui::Key::Delete, Default::default(), false);
    assert_eq!(h.session.history_len(), n + 1);
    assert!(
        h.session
            .snapshot()
            .project
            .document(d)
            .unwrap()
            .layers()
            .node(child)
            .is_none()
    );
    press(&mut h, egui::Key::Z, primary(), false);
    assert_eq!(h.session.snapshot().project, before);
}
#[test]
fn repeated_delete_empty_selection_and_modified_delete_cannot_create_edits() {
    let (mut h, a, b) = pair(false, 1024);
    let before = h.session.snapshot();
    press(
        &mut h,
        egui::Key::Delete,
        egui::Modifiers {
            alt: true,
            ..Default::default()
        },
        false,
    );
    assert_eq!(h.session.snapshot(), before);
    // egui derives repeat from key-down state. Holding a modified Delete and
    // releasing Alt must not reinterpret the repeated key as a fresh delete.
    press(&mut h, egui::Key::Delete, Default::default(), true);
    assert_eq!(h.session.snapshot(), before);
    release(&mut h, egui::Key::Delete);
    press(&mut h, egui::Key::Delete, Default::default(), false);
    h.ui.select(&mut h.session, Some(b));
    let after = h.session.snapshot();
    press(&mut h, egui::Key::Delete, Default::default(), true);
    assert_eq!(h.session.snapshot(), after);
    h.ui.select(&mut h.session, None);
    release(&mut h, egui::Key::Delete);
    press(&mut h, egui::Key::Delete, Default::default(), false);
    assert_eq!(h.session.snapshot(), after);
    assert!(EditorUi::transform(&after.project, a).is_none());
}
#[test]
fn ime_composition_commit_and_focus_loss_are_quarantined() {
    let (mut h, _, _) = pair(false, 1024);
    let before = h.session.snapshot();
    h.pass(
        vec![egui::Event::Ime(egui::ImeEvent::Preedit {
            text: "候选".into(),
            active_range_chars: None,
        })],
        [1180., 800.],
        true,
    );
    press(&mut h, egui::Key::Delete, Default::default(), false);
    assert_eq!(h.session.snapshot(), before);
    release(&mut h, egui::Key::Delete);
    h.pass(
        vec![
            egui::Event::Ime(egui::ImeEvent::Commit("选".into())),
            key(egui::Key::Delete),
        ],
        [1180., 800.],
        true,
    );
    assert_eq!(h.session.snapshot(), before);
    release(&mut h, egui::Key::Delete);
    h.pass(vec![key(egui::Key::Delete)], [1180., 800.], false);
    assert_eq!(h.session.snapshot(), before);
    release(&mut h, egui::Key::Delete);
    press(&mut h, egui::Key::Delete, Default::default(), false);
    assert_ne!(h.session.snapshot(), before);
}
#[test]
fn text_widget_owns_delete_undo_and_escape_but_a_focused_button_does_not() {
    let (mut h, a, _) = pair(false, 1024);
    let before = h.session.snapshot();
    let id = egui::Id::unique("focused text fixture");
    let mut text = "abc".to_owned();
    // A real egui TextEdit state, rather than treating every focused widget as text.
    let text_pass = |h: &mut Harness, events: Vec<egui::Event>, text: &mut String| {
        let mut frame = h.ctx.run_ui(
            egui::RawInput {
                events,
                focused: true,
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1180., 800.))),
                ..Default::default()
            },
            |root| {
                egui::Panel::top("text fixture").show(root, |ui| {
                    ui.add(egui::TextEdit::singleline(text).id(id))
                        .request_focus();
                });
                h.ui.show(root, &mut h.session, &mut h.presenter);
            },
        );
        frame.textures_delta.clear();
    };
    text_pass(&mut h, vec![], &mut text);
    assert!(h.ctx.text_edit_focused());
    for (k, m) in [
        (egui::Key::Delete, Default::default()),
        (egui::Key::Z, primary()),
        (egui::Key::Escape, Default::default()),
    ] {
        text_pass(
            &mut h,
            vec![egui::Event::Key {
                key: k,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: m,
            }],
            &mut text,
        );
        assert_eq!(h.session.snapshot(), before);
        assert_eq!(h.ui.selections(), [a]);
        text_pass(
            &mut h,
            vec![egui::Event::Key {
                key: k,
                physical_key: None,
                pressed: false,
                repeat: false,
                modifiers: m,
            }],
            &mut text,
        );
    }
    h.ctx
        .memory_mut(|m| m.request_focus(egui::Id::unique("ordinary button")));
    h.pass(vec![], [1180., 800.], true);
    press(&mut h, egui::Key::Delete, Default::default(), false);
    assert_ne!(h.session.snapshot(), before);
}
#[test]
fn switching_to_interaction_clears_joint_selection_and_keeps_committed_edits() {
    let (mut h, a, b) = pair(true, 1024);
    h.ui.toggle_selection(&mut h.session, b);
    let before = h.session.snapshot();
    let n = h.session.history_len();
    click(&mut h, "交互");
    assert!(h.ui.interaction);
    assert!(h.ui.selections().is_empty());
    assert_eq!(h.session.snapshot(), before);
    assert_eq!(h.session.history_len(), n);
    press(&mut h, egui::Key::Delete, Default::default(), false);
    press(&mut h, egui::Key::Z, primary(), false);
    assert_eq!(h.session.snapshot(), before);
    click(&mut h, "编辑");
    assert!(h.ui.selections().is_empty());
    h.ui.select(&mut h.session, Some(a));
    h.ui.interaction = true;
    h.pass(vec![], [1180., 800.], true);
    assert!(h.ui.selections().is_empty());
    assert_eq!(h.session.snapshot(), before);
}
#[test]
fn mode_change_cancels_owned_gesture_and_late_release_but_not_foreign_draft() {
    let mut h = Harness::new();
    let before = h.session.snapshot();
    let p = h.drag();
    assert!(EditorUi::draft_state(&h.session).is_some());
    h.ui.interaction = true;
    h.pass(vec![], [1180., 800.], true);
    h.pointer(p, Some(false));
    assert!(EditorUi::draft_state(&h.session).is_none());
    assert_eq!(h.session.snapshot(), before);
    h.ui.interaction = false;
    h.pass(vec![], [1180., 800.], true);
    h.session
        .edit_draft(DraftRequest {
            api_version: API_VERSION,
            session_id: before.session_id,
            request_id: RequestId::new(99999).unwrap(),
            action: DraftAction::Begin {
                expected: before.state,
            },
        })
        .unwrap();
    let draft = EditorUi::draft_state(&h.session);
    h.ui.interaction = true;
    h.pass(vec![], [1180., 800.], true);
    assert_eq!(EditorUi::draft_state(&h.session), draft);
    assert_eq!(h.session.snapshot(), before);
}
#[test]
fn topmost_canvas_pick_obeys_paint_order_and_does_not_edit_history() {
    let (mut h, _, b) = pair(false, 1024);
    move_layer(&mut h, b, 128.);
    let before = h.session.snapshot();
    let n = h.session.history_len();
    select_at(&mut h, [160., 160.]);
    assert_eq!(h.ui.selections(), [b]);
    assert_eq!(h.session.snapshot(), before);
    assert_eq!(h.session.history_len(), n);
    select_at(&mut h, [400., 400.]);
    assert!(h.ui.selections().is_empty());
    select_at(&mut h, [160., 160.]);
    assert_eq!(h.ui.selections(), [b]);
}
#[test]
fn hidden_layers_groups_zero_opacity_and_document_clip_do_not_pick() {
    let (mut h, a, b) = pair(false, 1024);
    move_layer(&mut h, b, 128.);
    let Selection::Layer(d, l) = b else {
        unreachable!()
    };
    for (visible, opacity) in [(false, 1.), (true, 0.)] {
        assert!(h.ui.submit(
            &mut h.session,
            vec![Command::SetLayerProperties {
                document: d,
                layer: l,
                visible,
                opacity: Opacity::new(opacity).unwrap()
            }]
        ));
        h.pass(vec![], [1180., 800.], true);
        select_at(&mut h, [160., 160.]);
        assert_eq!(h.ui.selections(), [a]);
    }
    move_layer(&mut h, a, 500.);
    select_at(&mut h, [530., 160.]);
    assert!(h.ui.selections().is_empty());
    let mut h = gesture_controls::grouped(1);
    let group = h.ui.selection().unwrap();
    let Selection::Layer(d, l) = group else {
        unreachable!()
    };
    assert!(h.ui.submit(
        &mut h.session,
        vec![Command::SetLayerProperties {
            document: d,
            layer: l,
            visible: false,
            opacity: Opacity::OPAQUE
        }]
    ));
    let p =
        h.ui.selection_geometry(&h.session, &EditorUi::project(&h.session))
            .unwrap()
            .controls(h.presenter.0.unwrap(), h.ui.canvas_rect.unwrap())
            .unwrap()
            .corners[0]
            + Vec2::splat(20.);
    h.pass(vec![], [1180., 800.], true);
    h.pointer(p, Some(true));
    h.pointer(p, Some(false));
    assert!(h.ui.selections().is_empty());
}
#[test]
fn canvas_ctrl_multi_pick_keeps_workspace_and_its_common_frame() {
    let (mut h, a, b) = pair(true, 1024);
    h.ui.target = Some(PreviewTarget::Workspace);
    h.pass(vec![], [1180., 800.], true);
    select_at(&mut h, [160., 160.]);
    assert_eq!(h.ui.selections(), [a]);
    assert_eq!(h.ui.target, Some(PreviewTarget::Workspace));
    h.modifiers = primary();
    select_at(&mut h, [250., 160.]);
    assert_eq!(h.ui.selections(), [a, b]);
    h.modifiers = Default::default();
    select_at(&mut h, [200., 160.]);
    assert_eq!(h.ui.selections(), [a, b]);
    h.modifiers = primary();
    select_at(&mut h, [160., 160.]);
    assert_eq!(h.ui.selections(), [b]);
}
#[test]
fn stale_preview_cannot_select_and_game_keys_cannot_be_history_shortcuts() {
    let (mut h, a, _) = pair(false, 1024);
    let before = h.session.snapshot();
    let p = point(&h, [400., 400.]);
    h.pointer(p, Some(true));
    h.presenter.0.as_mut().unwrap().frame.source.state.revision += 1;
    h.pointer(p, Some(false));
    assert_eq!(h.ui.selections(), [a]);
    h.ui.interaction = true;
    h.pass(vec![], [1180., 800.], true);
    h.pointer(p, Some(true));
    h.pointer(p, Some(false));
    assert_eq!(press(&mut h, egui::Key::Z, primary(), false).game_inputs, 0);
    release(&mut h, egui::Key::Z);
    assert_eq!(
        press(&mut h, egui::Key::Z, Default::default(), false).game_inputs,
        1
    );
    assert_eq!(
        press(&mut h, egui::Key::X, Default::default(), false).game_inputs,
        1
    );
    assert_eq!(h.session.snapshot(), before);
}

#[test]
fn escape_repeat_and_simultaneous_release_cancel_once_without_clearing_selection() {
    let mut h = Harness::new();
    let selection = h.ui.selections().to_vec();
    let before = h.session.snapshot();
    let p = h.drag();
    h.modifiers.shift = true;
    h.pass(
        vec![
            egui::Event::Key {
                key: egui::Key::Escape,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: h.modifiers,
            },
            egui::Event::PointerButton {
                pos: p,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: Default::default(),
            },
        ],
        [1180., 800.],
        true,
    );
    assert_eq!(h.session.snapshot(), before);
    assert_eq!(h.ui.selections(), selection);
    press(&mut h, egui::Key::Escape, Default::default(), true);
    assert_eq!(h.ui.selections(), selection);
    assert!(EditorUi::draft_state(&h.session).is_none());
    release(&mut h, egui::Key::Escape);
    press(&mut h, egui::Key::Escape, Default::default(), false);
    assert!(h.ui.selections().is_empty());
    assert_eq!(h.session.snapshot(), before);
}
#[test]
fn multiple_egui_passes_execute_delete_once_and_command_redo_aliases_work() {
    let (mut h, a, b) = pair(false, 1024);
    let before = h.session.snapshot().project;
    let nodes = h.session.history_len();
    let mut passes = 0;
    let mut frame = h.ctx.run_ui(
        egui::RawInput {
            events: vec![key(egui::Key::Delete)],
            focused: true,
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1180., 800.))),
            ..Default::default()
        },
        |root| {
            h.ui.show(root, &mut h.session, &mut h.presenter);
            passes += 1;
            if passes == 1 {
                h.ui.select(&mut h.session, Some(b));
                root.ctx().request_discard("controlled second pass");
            }
        },
    );
    frame.textures_delta.clear();
    assert_eq!(passes, 2);
    assert_eq!(h.session.history_len(), nodes + 1);
    assert!(EditorUi::transform(&h.session.snapshot().project, b).is_some());
    assert!(EditorUi::transform(&h.session.snapshot().project, a).is_none());
    let command = egui::Modifiers {
        command: true,
        mac_cmd: true,
        ..Default::default()
    };
    press(&mut h, egui::Key::Z, command, false);
    assert_eq!(h.session.snapshot().project, before);
    press(&mut h, egui::Key::Y, command, false);
    assert!(EditorUi::transform(&h.session.snapshot().project, a).is_none());
}
#[test]
fn picking_applies_member_mapping_rotation_and_visibility_without_changing_target() {
    let mut h = gesture_controls::grouped(2);
    let Selection::Component(group) = h.ui.selection().unwrap() else {
        unreachable!()
    };
    let project = h.session.snapshot().project;
    let doc = project.documents().next().unwrap().id();
    let member = project.components().member(doc).unwrap();
    let layer = project
        .document(doc)
        .unwrap()
        .layers()
        .iter()
        .find(|l| matches!(l.kind, LayerKind::Image { .. }))
        .unwrap()
        .id;
    assert!(h.ui.submit(
        &mut h.session,
        vec![
            Command::SetComponentMapping {
                component: member,
                document_to_component: Affine2::new([1.2, 0., 0., 1.2, -10., 20.]).unwrap()
            },
            Command::SetComponentTransform {
                component: group,
                transform: Affine2::new([0., 1., -1., 0., 450., 0.]).unwrap()
            }
        ]
    ));
    h.pass(vec![], [1180., 800.], true);
    let p = h
        .session
        .snapshot()
        .project
        .document_workspace_transform(doc)
        .unwrap()
        .compose(
            h.session
                .snapshot()
                .project
                .document(doc)
                .unwrap()
                .layers()
                .world_transform(layer)
                .unwrap(),
        )
        .unwrap()
        .point(32., 32.)
        .unwrap();
    select_at(&mut h, p);
    assert_eq!(h.ui.selections(), [Selection::Layer(doc, layer)]);
    assert_eq!(h.ui.target, Some(PreviewTarget::Components));
    let controls =
        h.ui.selection_geometry(&h.session, &EditorUi::project(&h.session))
            .unwrap()
            .controls(h.presenter.0.unwrap(), h.ui.canvas_rect.unwrap())
            .unwrap();
    let center = controls.corners[0] + (controls.corners[2] - controls.corners[0]) * 0.5;
    let pointer = point(&h, p);
    assert!(
        center.distance(pointer) < 0.001,
        "picked layer controls must use the presented component space"
    );
    let delta = Vec2::new(25., 10.);
    h.pointer(pointer, Some(true));
    h.pointer(pointer + delta, None);
    h.pointer(pointer + delta, Some(false));
    let project = h.session.snapshot().project;
    let moved = project
        .document_workspace_transform(doc)
        .unwrap()
        .compose(
            project
                .document(doc)
                .unwrap()
                .layers()
                .world_transform(layer)
                .unwrap(),
        )
        .unwrap()
        .point(32., 32.)
        .unwrap();
    let ratio = h.presenter.0.unwrap().frame.source.context.size;
    let rect = h.ui.canvas_rect.unwrap();
    for (axis, offset) in [
        delta.x * ratio[0] as f32 / rect.width(),
        delta.y * ratio[1] as f32 / rect.height(),
    ]
    .into_iter()
    .enumerate()
    {
        assert!((moved[axis] - p[axis] - f64::from(offset)).abs() < 0.001);
    }
    assert!(h.ui.submit(
        &mut h.session,
        vec![Command::SetDocumentVisibility {
            document: doc,
            visible: false
        }]
    ));
    h.pass(vec![], [1180., 800.], true);
    select_at(&mut h, p);
    assert!(h.ui.selections().is_empty());
}
