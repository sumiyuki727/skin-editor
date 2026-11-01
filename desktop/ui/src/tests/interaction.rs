use super::*;
use joint_selection::{click, pair};

#[test]
fn interaction_tool_blocks_layer_panel_selection_and_edit_tool_reenables_it() {
    let (mut h, a, b) = pair(false, 1024);
    let Selection::Layer(_, layer) = b else {
        unreachable!()
    };
    let before = h.session.snapshot();
    let nodes = h.session.history_len();
    click(&mut h, "交互");
    assert!(h.ui.interaction);
    assert!(h.ui.selections().is_empty());
    for additive in [false, true] {
        h.modifiers = egui::Modifiers {
            ctrl: additive,
            command: additive,
            ..Default::default()
        };
        click(&mut h, &format!("图层 {}", layer.get()));
        assert!(
            h.ui.selections().is_empty(),
            "interaction tool must block the layer panel, including additive selection"
        );
    }
    let rect = h.ui.canvas_rect.unwrap();
    let size = h.presenter.0.unwrap().frame.source.context.size;
    let p = rect.min
        + Vec2::new(
            160. / size[0] as f32 * rect.width(),
            160. / size[1] as f32 * rect.height(),
        );
    h.pointer(p, Some(true));
    h.pointer(p, Some(false));
    assert!(
        h.ui.selections().is_empty(),
        "interaction canvas must not acquire editor selection"
    );
    assert_eq!(h.session.snapshot(), before);
    assert_eq!(h.session.history_len(), nodes);
    h.modifiers = Default::default();
    click(&mut h, "编辑");
    assert!(h.ui.selections().is_empty());
    click(&mut h, &format!("图层 {}", layer.get()));
    assert_eq!(h.ui.selections(), [b]);
    assert_ne!(h.ui.selections(), [a]);
    assert_eq!(h.session.snapshot(), before);
}

#[test]
fn interaction_tool_blocks_nested_group_and_component_rows() {
    for kind in [1, 2] {
        let mut h = gesture_controls::grouped(kind);
        h.ctx
            .style_mut_of(egui::Theme::Dark, |s| s.animation_time = 0.);
        h.ctx
            .style_mut_of(egui::Theme::Light, |s| s.animation_time = 0.);
        if kind == 2 {
            click(&mut h, "跨元素组件");
            h.pass(vec![], [1180., 800.], true);
        }
        let labels: Vec<_> = h
            .labels
            .iter()
            .filter(|(s, _)| {
                s.starts_with("▾ 组 ")
                    || s.starts_with("图层 ")
                    || s.starts_with("组件组 ")
                    || s.starts_with("元素引用 ")
            })
            .map(|(s, _)| s.clone())
            .collect();
        assert!(labels.len() >= 2);
        let before = h.session.snapshot();
        let target = h.ui.target;
        click(&mut h, "交互");
        for label in labels {
            click(&mut h, &label);
            assert!(h.ui.selections().is_empty(), "blocked row: {label}");
            assert_eq!(h.ui.target, target);
        }
        assert_eq!(h.session.snapshot(), before);
    }
}

#[test]
fn interaction_selection_apis_are_noops_and_do_not_cancel_an_owned_draft() {
    let (mut h, a, b) = pair(false, 1024);
    click(&mut h, "交互");
    let before = h.session.snapshot();
    let nodes = h.session.history_len();
    assert!(h.ui.draft(
        &mut h.session,
        DraftAction::Begin {
            expected: before.state
        }
    ));
    let draft = EditorUi::draft_state(&h.session);
    for item in [a, b] {
        h.ui.select(&mut h.session, Some(item));
        h.ui.toggle_selection(&mut h.session, item);
        assert!(h.ui.selections().is_empty());
        assert_eq!(
            EditorUi::draft_state(&h.session),
            draft,
            "blocked selection must not cancel an unrelated edit"
        );
    }
    h.pass(vec![], [1180., 800.], true);
    assert_eq!(EditorUi::draft_state(&h.session), draft);
    assert_eq!(h.session.snapshot(), before);
    assert_eq!(h.session.history_len(), nodes);
    h.ui.cancel(&mut h.session);
}

#[test]
fn interaction_never_exposes_transform_geometry_even_before_the_next_ui_pass() {
    let (mut h, _, _) = pair(false, 1024);
    let project = EditorUi::project(&h.session);
    assert!(h.ui.selection_geometry(&h.session, &project).is_some());
    h.ui.interaction = true;
    assert!(h.ui.selection_geometry(&h.session, &project).is_none());
    h.pass(vec![], [1180., 800.], true);
    assert!(h.ui.selections().is_empty());
}

#[test]
fn preview_scene_and_operation_tool_have_independent_selection_semantics() {
    let (mut h, a, _) = pair(false, 1024);
    let before = h.session.snapshot();
    let nodes = h.session.history_len();
    click(&mut h, "GUI 占位");
    assert_eq!(h.ui.scene, 1);
    assert!(!h.ui.interaction);
    assert_eq!(h.ui.selections(), [a]);
    click(&mut h, "交互");
    assert!(h.ui.interaction);
    assert!(h.ui.selections().is_empty());
    click(&mut h, "画布");
    assert_eq!(h.ui.scene, 0);
    assert!(h.ui.interaction);
    assert!(h.ui.selections().is_empty());
    click(&mut h, "编辑");
    assert!(!h.ui.interaction);
    assert!(h.ui.selections().is_empty());
    assert_eq!(h.session.snapshot(), before);
    assert_eq!(h.session.history_len(), nodes);
}

#[test]
fn adding_an_image_while_interacting_does_not_reopen_visual_selection() {
    let (mut h, a, _) = pair(false, 1024);
    let Selection::Layer(d, l) = a else {
        unreachable!()
    };
    let project = h.session.snapshot().project;
    let LayerKind::Image { asset } = project.document(d).unwrap().layers().node(l).unwrap().kind
    else {
        unreachable!()
    };
    click(&mut h, "交互");
    let nodes = h.session.history_len();
    h.ui.add_asset_layer(&mut h.session, asset);
    assert_eq!(h.session.history_len(), nodes + 1);
    assert!(h.ui.selections().is_empty());
    assert_eq!(h.session.snapshot().project.assets(), project.assets());
    assert_eq!(
        h.session
            .snapshot()
            .project
            .document(d)
            .unwrap()
            .layers()
            .iter()
            .len(),
        3
    );
    h.pass(vec![], [1180., 800.], true);
    assert!(h.ui.selections().is_empty());
    click(&mut h, "编辑");
    assert!(h.ui.selections().is_empty());
}
