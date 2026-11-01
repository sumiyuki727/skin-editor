use super::*;
use joint_selection::click;

fn click_size(h: &mut Harness, label: &str, size: [f32; 2]) {
    let p = h
        .labels
        .iter()
        .find(|(s, _)| s == label)
        .unwrap_or_else(|| panic!("missing {label}"))
        .1
        .center();
    for pressed in [true, false] {
        h.pass(
            vec![
                egui::Event::PointerMoved(p),
                egui::Event::PointerButton {
                    pos: p,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: Default::default(),
                },
            ],
            size,
            true,
        );
    }
}

fn branch(h: &mut Harness) -> (NodeId, NodeId, NodeId) {
    let c = h.session.project_info().state.head;
    click(h, "放大");
    let a = h.session.project_info().state.head;
    assert_ne!(a, c);
    click(h, "撤销");
    assert_eq!(h.session.project_info().state.head, c);
    click(h, "缩小");
    let b = h.session.project_info().state.head;
    assert_ne!(b, c);
    assert_ne!(b, a);
    click(h, "撤销");
    assert_eq!(h.session.project_info().state.head, c);
    (c, a, b)
}
#[test]
fn manual_two_branch_sequence_shows_both_children_and_navigates_them() {
    let mut h = Harness::new();
    let (c, a, b) = branch(&mut h);
    let info = h.session.project_info();
    let children = h
        .session
        .history_children(info.session_id, c, 0, 16)
        .unwrap()
        .entries;
    assert_eq!(children.iter().map(|n| n.id).collect::<Vec<_>>(), [a, b]);
    assert!(
        h.labels.iter().any(|(s, _)| s == &a.short_address()),
        "A not painted: {:?}",
        h.labels
    );
    assert!(
        h.labels.iter().any(|(s, _)| s == &b.short_address()),
        "B not painted: {:?}",
        h.labels
    );
    click(&mut h, "重做");
    assert_eq!(h.session.project_info().state.head, b);
    click(&mut h, "撤销");
    click(&mut h, &a.short_address());
    assert_eq!(h.session.project_info().state.head, a);
    click(&mut h, "撤销");
    click(&mut h, "重做");
    assert_eq!(h.session.project_info().state.head, a);
    let prefix = format!("{}  ", c.short_address());
    let ancestor = h
        .labels
        .iter()
        .find(|(s, _)| s.starts_with(&prefix))
        .unwrap()
        .0
        .clone();
    click(&mut h, &ancestor);
    assert_eq!(h.session.project_info().state.head, c);
    h.pass(vec![], [1180., 800.], true);
    click(&mut h, &b.short_address());
    assert_eq!(h.session.project_info().state.head, b);
}

#[test]
fn child_navigation_remains_visible_in_a_short_viewport_with_deep_ancestry() {
    let mut h = Harness::new();
    let selected = h.ui.selection().unwrap();
    for n in 0..18 {
        assert!(h.ui.submit(
            &mut h.session,
            vec![EditorUi::transform_command(
                selected,
                Affine2::new([1., 0., 0., 1., 128. + n as f64, 128.]).unwrap()
            )]
        ));
    }
    let c = h.session.project_info().state.head;
    assert!(h.ui.submit(
        &mut h.session,
        vec![EditorUi::transform_command(
            selected,
            Affine2::new([1.1, 0., 0., 1.1, 145., 128.]).unwrap()
        )]
    ));
    let a = h.session.project_info().state.head;
    h.ui.navigation(&mut h.session, HistoryAction::Undo);
    assert!(h.ui.submit(
        &mut h.session,
        vec![EditorUi::transform_command(
            selected,
            Affine2::new([0.9, 0., 0., 0.9, 145., 128.]).unwrap()
        )]
    ));
    let b = h.session.project_info().state.head;
    h.ui.navigation(&mut h.session, HistoryAction::Undo);
    assert_eq!(h.session.project_info().state.head, c);
    for _ in 0..3 {
        h.pass(vec![], [900., 480.], true);
    }
    assert!(
        h.labels.iter().any(|(s, _)| s == &a.short_address()),
        "first child hidden by ancestry"
    );
    assert!(
        h.labels.iter().any(|(s, _)| s == &b.short_address()),
        "second child hidden by ancestry"
    );
    let nodes = h.session.history_len();
    click_size(&mut h, &a.short_address(), [900., 480.]);
    assert_eq!(h.session.project_info().state.head, a);
    assert_eq!(h.session.history_len(), nodes);
}

#[test]
fn leaf_has_an_explicit_empty_state_instead_of_an_unexplained_blank() {
    let mut h = Harness::new();
    assert!(h.labels.iter().any(|(s, _)| s == "当前节点没有重做分支"));
    assert!(h.labels.iter().any(|(s, _)| s == "本页 0 项"));
    let before = h.session.snapshot();
    h.pass(vec![], [900., 480.], true);
    assert_eq!(h.session.snapshot(), before);
}

#[test]
fn a_held_child_pointer_cannot_click_an_ancestor_after_the_head_changes() {
    let mut h = Harness::new();
    let (_, a, b) = branch(&mut h);
    let p = h
        .labels
        .iter()
        .find(|(s, _)| s == &a.short_address())
        .unwrap()
        .1
        .center();
    h.pointer(p, Some(true));
    h.ui.navigation(&mut h.session, HistoryAction::Checkout { node: b });
    h.pass(vec![], [1180., 800.], true);
    let nodes = h.session.history_len();
    h.pointer(p, Some(false));
    assert_eq!(h.session.project_info().state.head, b);
    assert_eq!(h.session.history_len(), nodes);
}
