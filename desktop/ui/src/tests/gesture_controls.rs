use super::Harness;
use super::*;

pub(super) fn grouped(kind: u8) -> Harness {
    let mut h = Harness::new();
    let Selection::Layer(doc, layer) = h.ui.selection().unwrap() else {
        panic!("layer")
    };
    if kind == 1 {
        let group = LayerId::new(h.ui.object_id()).unwrap();
        assert!(h.ui.submit(
            &mut h.session,
            vec![
                Command::AddLayer {
                    document: doc,
                    node: LayerNode {
                        id: group,
                        local: Affine2::IDENTITY,
                        opacity: Opacity::OPAQUE,
                        visible: true,
                        kind: LayerKind::Group { children: vec![] }
                    },
                    parent: None,
                    index: 1
                },
                Command::MoveLayer {
                    document: doc,
                    layer,
                    parent: Some(group),
                    index: 0
                }
            ]
        ));
        h.ui.select(&mut h.session, Some(Selection::Layer(doc, group)));
    } else if kind == 2 {
        let group = ComponentId::new(h.ui.object_id()).unwrap();
        let member = ComponentId::new(h.ui.object_id()).unwrap();
        assert!(h.ui.submit(
            &mut h.session,
            vec![
                Command::AddComponent {
                    node: ComponentNode {
                        id: group,
                        local: Affine2::IDENTITY,
                        kind: ComponentKind::Group { children: vec![] }
                    },
                    parent: None,
                    index: 0
                },
                Command::AddComponent {
                    node: ComponentNode {
                        id: member,
                        local: Affine2::IDENTITY,
                        kind: ComponentKind::Element {
                            document: doc,
                            document_to_component: Affine2::IDENTITY
                        }
                    },
                    parent: Some(group),
                    index: 0
                }
            ]
        ));
        h.ui.target = Some(PreviewTarget::Components);
        h.ui.select(&mut h.session, Some(Selection::Component(group)));
    }
    for _ in 0..3 {
        h.pass(vec![], [1180., 800.], true);
    }
    h
}
fn bounds(h: &Harness) -> Rect {
    let p = EditorUi::project(&h.session);
    let b = match h.ui.selection().unwrap() {
        Selection::Layer(d, l) => p.layer_bounds(d, l).unwrap().unwrap(),
        Selection::Component(c) => p.component_bounds(c).unwrap().unwrap(),
    };
    let r = h.ui.canvas_rect.unwrap();
    let size = h.presenter.0.unwrap().frame.source.context.size;
    let map = |p: [f64; 2]| {
        r.min
            + Vec2::new(
                p[0] as f32 / size[0] as f32 * r.width(),
                p[1] as f32 / size[1] as f32 * r.height(),
            )
    };
    Rect::from_min_max(map(b.min()), map(b.max()))
}
#[test]
fn corner_drag_is_one_transaction_for_layer_nested_group_and_component_group() {
    for kind in 0..3 {
        let mut h = grouped(kind);
        let before = h.session.snapshot();
        let nodes = h.session.history_len();
        let rect = bounds(&h);
        h.pointer(rect.max, Some(true));
        h.pointer(rect.max + Vec2::new(32., 24.), None);
        assert_eq!(h.session.snapshot(), before);
        assert!(EditorUi::draft_state(&h.session).is_some());
        let candidate = bounds(&h);
        assert!(candidate.min.distance(rect.min) < 0.01);
        assert!(candidate.width() > rect.width() && candidate.height() > rect.height());
        h.pointer(rect.max + Vec2::new(50., 35.), None);
        h.pointer(rect.max + Vec2::new(50., 35.), Some(false));
        assert_eq!(h.session.history_len(), nodes + 1);
        h.ui.navigation(&mut h.session, HistoryAction::Undo);
        assert_eq!(h.session.snapshot().project, before.project);
        h.ui.navigation(&mut h.session, HistoryAction::Redo { child: None });
        assert_ne!(h.session.snapshot().project, before.project);
    }
}
#[test]
fn rotation_cancel_and_late_release_preserve_the_original_node() {
    for fault in [false, true] {
        let mut h = grouped(2);
        let before = h.session.snapshot();
        let nodes = h.session.history_len();
        let rect = bounds(&h);
        let press = gestures::rotation(rect, h.ui.canvas_rect.unwrap());
        h.pointer(press, Some(true));
        h.pointer(press + Vec2::new(30., 15.), None);
        let after = EditorUi::transform(&EditorUi::project(&h.session), h.ui.selection().unwrap())
            .unwrap()
            .coefficients();
        assert!(
            after[1].abs() > 0.01,
            "{press:?} {rect:?} {:?} {after:?}",
            h.ui.gesture
        );
        if fault {
            assert!(h.ui.interrupt_for_fault(&mut h.session));
        } else {
            h.pass(vec![super::key(egui::Key::Escape)], [1180., 800.], true);
        }
        h.pointer(press + Vec2::new(70., 20.), None);
        h.pointer(press + Vec2::new(70., 20.), Some(false));
        assert_eq!(h.session.snapshot(), before);
        assert_eq!(h.session.history_len(), nodes);
        assert!(EditorUi::draft_state(&h.session).is_none());
    }
}

#[derive(Clone, Copy, Debug)]
enum Control {
    Move,
    Scale,
    Rotate,
}
fn press_point(h: &Harness, control: Control) -> Pos2 {
    let r = bounds(h);
    match control {
        Control::Move => r.center(),
        Control::Scale => r.max,
        Control::Rotate => gestures::rotation(r, h.ui.canvas_rect.unwrap()),
    }
}

#[test]
fn held_canvas_gestures_keep_the_candidate_and_accumulate_motion_across_frames() {
    for kind in 0..3 {
        for control in [Control::Move, Control::Scale, Control::Rotate] {
            let mut h = grouped(kind);
            let before = h.session.snapshot();
            let nodes = h.session.history_len();
            let rect = bounds(&h);
            let start = press_point(&h, control);
            let base = EditorUi::transform(&before.project, h.ui.selection().unwrap()).unwrap();
            let size = h.presenter.0.unwrap().frame.source.context.size;
            let canvas = h.ui.canvas_rect.unwrap();
            let ratio = [
                size[0] as f64 / canvas.width() as f64,
                size[1] as f64 / canvas.height() as f64,
            ];
            let old_frame = h.presenter.0;
            h.pointer(start, Some(true));
            h.pointer(start + Vec2::new(30., 15.), None);
            let candidate = EditorUi::project(&h.session);
            let ready_candidate = h.presenter.0;
            assert_ne!(candidate, before.project);
            // Real pointer input has many no-motion frames while a button is
            // held. Keep displaying an old frame too: GPU readiness is separate.
            let requests = h
                .session
                .history_statistics(before.session_id)
                .unwrap()
                .retained_requests;
            for _ in 0..8 {
                h.pass(vec![], [1180., 800.], true);
                assert_eq!(
                    EditorUi::project(&h.session),
                    candidate,
                    "kind={kind} control={control:?}: stationary hold reset the candidate"
                );
                h.presenter.0 = old_frame;
            }
            assert_eq!(
                h.session
                    .history_statistics(before.session_id)
                    .unwrap()
                    .retained_requests,
                requests
            );
            h.presenter.0 = ready_candidate;
            h.pointer(start + Vec2::new(45., 22.), None);
            h.pointer(start + Vec2::new(60., 30.), None);
            let project = EditorUi::project(&h.session);
            let value = EditorUi::transform(&project, h.ui.selection().unwrap())
                .unwrap()
                .coefficients();
            match control {
                Control::Move => {
                    let base = base.coefficients();
                    assert!((value[4] - base[4] - 60. * ratio[0]).abs() < 1e-6);
                    assert!((value[5] - base[5] - 30. * ratio[1]).abs() < 1e-6);
                }
                Control::Scale => {
                    let next = bounds(&h);
                    assert!(next.min.distance(rect.min) < 0.01);
                    assert!(next.max.distance(rect.max + Vec2::new(60., 30.)) < 0.01);
                }
                Control::Rotate => {
                    let from = start - rect.center();
                    let to = start + Vec2::new(60., 30.) - rect.center();
                    let expected = (f64::from(to.y) * ratio[1]).atan2(f64::from(to.x) * ratio[0])
                        - (f64::from(from.y) * ratio[1]).atan2(f64::from(from.x) * ratio[0]);
                    assert!((value[1].atan2(value[0]) - expected).abs() < 1e-6);
                    assert!(bounds(&h).center().distance(rect.center()) < 0.01);
                }
            }
            for _ in 0..8 {
                h.pass(vec![], [1180., 800.], true);
                assert_eq!(EditorUi::project(&h.session), project);
            }
            assert_eq!(h.session.snapshot(), before);
            assert_eq!(h.session.history_len(), nodes);
            h.pointer(start + Vec2::new(60., 30.), Some(false));
            assert_eq!(h.session.snapshot().project, project);
            assert_eq!(h.session.history_len(), nodes + 1);
            h.ui.navigation(&mut h.session, HistoryAction::Undo);
            assert_eq!(h.session.snapshot().project, before.project);
            h.ui.navigation(&mut h.session, HistoryAction::Redo { child: None });
            assert_eq!(h.session.snapshot().project, project);
        }
    }
}

#[test]
fn held_canvas_return_to_press_origin_is_a_noop() {
    for kind in 0..3 {
        for control in [Control::Move, Control::Scale, Control::Rotate] {
            let mut h = grouped(kind);
            let before = h.session.snapshot();
            let nodes = h.session.history_len();
            let start = press_point(&h, control);
            h.pointer(start, Some(true));
            h.pointer(start + Vec2::new(30., 15.), None);
            h.pointer(start + Vec2::new(60., 30.), None);
            h.pointer(start, None);
            assert_eq!(EditorUi::project(&h.session), before.project);
            for _ in 0..8 {
                h.pass(vec![], [1180., 800.], true);
                assert_eq!(EditorUi::project(&h.session), before.project);
            }
            h.pointer(start, Some(false));
            assert_eq!(h.session.snapshot(), before);
            assert_eq!(h.session.history_len(), nodes);
            assert!(EditorUi::draft_state(&h.session).is_none());
        }
    }
}

#[test]
fn canvas_release_includes_the_last_pointer_motion() {
    let mut h = grouped(0);
    let before = h.session.snapshot();
    let start = press_point(&h, Control::Move);
    let selection = h.ui.selection().unwrap();
    let base = EditorUi::transform(&before.project, selection)
        .unwrap()
        .coefficients();
    let canvas = h.ui.canvas_rect.unwrap();
    let size = h.presenter.0.unwrap().frame.source.context.size;
    h.pointer(start, Some(true));
    h.pointer(start + Vec2::new(30., 15.), None);
    h.pointer(start + Vec2::new(60., 30.), Some(false));
    let value = EditorUi::transform(&h.session.snapshot().project, selection)
        .unwrap()
        .coefficients();
    assert!((value[4] - base[4] - 60. * size[0] as f64 / canvas.width() as f64).abs() < 1e-6);
    assert!((value[5] - base[5] - 30. * size[1] as f64 / canvas.height() as f64).abs() < 1e-6);
    let node = h
        .session
        .history_summary(before.session_id, h.session.project_info().state.head)
        .unwrap();
    assert_eq!(node.parent, Some(before.state.head));
}

#[test]
fn pending_preview_coalesces_pointer_motion_then_flushes_when_ready() {
    for kind in 0..3 {
        let mut h = grouped(kind);
        let start = bounds(&h).center();
        let old = h.presenter.0;
        let before = h.session.snapshot();
        let nodes = h.session.history_len();
        h.pointer(start, Some(true));
        h.pointer(start + Vec2::new(25., 10.), None);
        let ready = h.presenter.0;
        let first = EditorUi::project(&h.session);
        let requests = h
            .session
            .history_statistics(before.session_id)
            .unwrap()
            .retained_requests;
        // The renderer has not published the first candidate. More input must
        // not keep invalidating its source; retain only the latest pointer.
        for n in 26..86 {
            h.presenter.0 = old;
            h.pointer(start + Vec2::new(n as f32, 20.), None);
            assert_eq!(
                EditorUi::project(&h.session),
                first,
                "pending input invalidated its in-flight source"
            );
        }
        assert_eq!(
            h.session
                .history_statistics(before.session_id)
                .unwrap()
                .retained_requests,
            requests
        );
        h.presenter.0 = ready;
        h.pass(vec![], [1180., 800.], true);
        let latest = EditorUi::project(&h.session);
        assert_ne!(
            latest, first,
            "latest pointer must flush even without a new motion event"
        );
        assert_eq!(h.session.snapshot(), before);
        h.pointer(start + Vec2::new(85., 20.), Some(false));
        assert_eq!(h.session.snapshot().project, latest);
        assert_eq!(h.session.history_len(), nodes + 1);
    }
}

fn controls(h: &Harness) -> gestures::ControlBox {
    let (b, world) =
        gestures::geometry(&EditorUi::project(&h.session), h.ui.selection().unwrap()).unwrap();
    let view = h.presenter.0.unwrap();
    let map = view.view.compose(world).unwrap();
    let canvas = h.ui.canvas_rect.unwrap();
    let point = |x, y| {
        let p = map.point(x, y).unwrap();
        canvas.min
            + Vec2::new(
                p[0] as f32 / view.frame.source.context.size[0] as f32 * canvas.width(),
                p[1] as f32 / view.frame.source.context.size[1] as f32 * canvas.height(),
            )
    };
    let (low, high) = (b.min(), b.max());
    gestures::ControlBox {
        corners: [
            point(low[0], low[1]),
            point(high[0], low[1]),
            point(high[0], high[1]),
            point(low[0], high[1]),
        ],
    }
}

#[test]
fn rotated_layer_nested_group_and_component_controls_scale_in_their_own_axes() {
    for kind in 0..3 {
        for shift in [false, true] {
            let mut h = grouped(kind);
            let angle: f64 = 0.27;
            let base = Affine2::new([
                0.6 * angle.cos(),
                0.6 * angle.sin(),
                -0.6 * angle.sin(),
                0.6 * angle.cos(),
                100.,
                40.,
            ])
            .unwrap();
            let selection = h.ui.selection().unwrap();
            assert!(h.ui.submit(
                &mut h.session,
                vec![EditorUi::transform_command(selection, base)]
            ));
            h.pass(vec![], [1180., 800.], true);
            let before = h.session.snapshot();
            let nodes = h.session.history_len();
            let box_before = controls(&h);
            // Local top edge must rotate; horizontal world AABBs cannot pass.
            assert!((box_before.corners[1].y - box_before.corners[0].y).abs() > 1.);
            let start = box_before.corners[2];
            h.modifiers.shift = shift;
            h.pointer(start, Some(true));
            h.pointer(start + Vec2::new(30., 10.), None);
            let candidate = EditorUi::project(&h.session);
            let after = EditorUi::transform(&candidate, selection).unwrap();
            let relative = base
                .inverse()
                .unwrap()
                .compose(after)
                .unwrap()
                .coefficients();
            assert!(
                relative[1].abs() < 1e-6 && relative[2].abs() < 1e-6,
                "scale must follow local axes: {relative:?}"
            );
            if shift {
                assert!((relative[0] - relative[3]).abs() < 1e-6);
            } else {
                assert!((relative[0] - relative[3]).abs() > 0.01);
            }
            assert!(controls(&h).corners[0].distance(box_before.corners[0]) < 0.01);
            assert_eq!(h.session.snapshot(), before);
            h.pointer(start + Vec2::new(30., 10.), Some(false));
            assert_eq!(h.session.history_len(), nodes + 1);
            h.ui.navigation(&mut h.session, HistoryAction::Undo);
            assert_eq!(h.session.snapshot().project, before.project);
            h.ui.navigation(&mut h.session, HistoryAction::Redo { child: None });
            assert_eq!(h.session.snapshot().project, candidate);
        }
    }
}

#[test]
fn shift_can_toggle_at_a_stationary_pointer_and_release_flushes_pending_final_pose() {
    let mut h = grouped(0);
    let before = h.session.snapshot();
    let nodes = h.session.history_len();
    let start = controls(&h).corners[2];
    h.pointer(start, Some(true));
    h.pointer(start + Vec2::new(30., 10.), None);
    let free = EditorUi::project(&h.session);
    h.modifiers.shift = true;
    h.pass(vec![], [1180., 800.], true);
    let locked = EditorUi::project(&h.session);
    assert_ne!(locked, free);
    let a = EditorUi::transform(&locked, h.ui.selection().unwrap())
        .unwrap()
        .coefficients();
    assert!((a[0] - a[3]).abs() < 1e-6);
    h.modifiers.shift = false;
    h.pass(vec![], [1180., 800.], true);
    assert_eq!(EditorUi::project(&h.session), free);
    let stale = h.presenter.0;
    h.pointer(start + Vec2::new(40., 15.), None);
    h.presenter.0 = stale;
    h.modifiers.shift = true;
    h.pointer(start + Vec2::new(50., 20.), Some(false));
    let after = h.session.snapshot();
    let a = EditorUi::transform(&after.project, h.ui.selection().unwrap())
        .unwrap()
        .coefficients();
    assert!((a[0] - a[3]).abs() < 1e-6);
    assert_eq!(h.session.history_len(), nodes + 1);
    assert!(EditorUi::draft_state(&h.session).is_none());
    h.ui.navigation(&mut h.session, HistoryAction::Undo);
    assert_eq!(h.session.snapshot().project, before.project);
}

#[test]
fn control_geometry_unions_children_before_selected_transform_and_keeps_output_mapping() {
    let mut h = grouped(1);
    let Selection::Layer(document, group) = h.ui.selection().unwrap() else {
        panic!("layer group")
    };
    let project = EditorUi::project(&h.session);
    let asset = project.assets().iter().next().unwrap().id();
    let extra = LayerId::new(h.ui.object_id()).unwrap();
    let world = Affine2::new([0., 2., -3., 0., 400., 500.]).unwrap();
    assert!(h.ui.submit(
        &mut h.session,
        vec![
            Command::AddLayer {
                document,
                node: LayerNode {
                    id: extra,
                    local: Affine2::new([2., 0., 0., -0.5, 200., 132.]).unwrap(),
                    visible: false,
                    opacity: Opacity::new(0.).unwrap(),
                    kind: LayerKind::Image { asset }
                },
                parent: Some(group),
                index: 1
            },
            Command::SetLayerTransform {
                document,
                layer: group,
                transform: world
            }
        ]
    ));
    let (b, map) =
        gestures::geometry(&EditorUi::project(&h.session), h.ui.selection().unwrap()).unwrap();
    assert_eq!(b.min(), [128., 100.]);
    assert_eq!(b.max(), [328., 192.]);
    assert_eq!(map, world);
    let mut h = grouped(2);
    let project = EditorUi::project(&h.session);
    let Selection::Component(group) = h.ui.selection().unwrap() else {
        panic!("component group")
    };
    let member = project.components().node(group).unwrap().children()[0];
    assert!(h.ui.submit(
        &mut h.session,
        vec![
            Command::SetComponentTransform {
                component: group,
                transform: world
            },
            Command::SetComponentTransform {
                component: member,
                transform: Affine2::new([1., 0., 0., 1., 40., 50.]).unwrap()
            },
            Command::SetComponentMapping {
                component: member,
                document_to_component: Affine2::new([0., 1., -1., 0., 512., 0.]).unwrap()
            }
        ]
    ));
    let (b, map) =
        gestures::geometry(&EditorUi::project(&h.session), h.ui.selection().unwrap()).unwrap();
    assert_eq!(b.min(), [40., 50.]);
    assert_eq!(b.max(), [552., 562.]);
    assert_eq!(map, world);
}

#[test]
fn held_canvas_gestures_cancel_without_a_node_or_late_release_commit() {
    for control in [Control::Move, Control::Scale, Control::Rotate] {
        for fault in [false, true] {
            let mut h = grouped(2);
            let before = h.session.snapshot();
            let nodes = h.session.history_len();
            let start = press_point(&h, control);
            h.pointer(start, Some(true));
            h.pointer(start + Vec2::new(30., 15.), None);
            h.pointer(start + Vec2::new(60., 30.), None);
            let candidate = EditorUi::project(&h.session);
            for _ in 0..8 {
                h.pass(vec![], [1180., 800.], true);
                assert_eq!(EditorUi::project(&h.session), candidate);
            }
            if fault {
                assert!(h.ui.interrupt_for_fault(&mut h.session));
            } else {
                h.pass(vec![super::key(egui::Key::Escape)], [1180., 800.], true);
            }
            h.pointer(start + Vec2::new(90., 45.), None);
            h.pointer(start + Vec2::new(90., 45.), Some(false));
            assert_eq!(h.session.snapshot(), before);
            assert_eq!(h.session.history_len(), nodes);
            assert!(EditorUi::draft_state(&h.session).is_none());
        }
    }
}
