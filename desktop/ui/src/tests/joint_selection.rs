use super::*;

pub(super) fn pair(cross: bool, limit: usize) -> (Harness, Selection, Selection) {
    let mut ui = EditorUi::default();
    let mut session = ProjectSession::with_resources(
        1,
        2,
        Box::new(Store),
        SessionLimits {
            commands_per_request: limit,
            ..Default::default()
        },
    )
    .unwrap();
    let a = AssetId::new(ui.object_id()).unwrap();
    assert!(ui.submit(
        &mut session,
        vec![Command::RegisterAsset {
                asset: Asset::new(
                    a,
                    "joint".into(),
                    BlobRef::new([1; 32], 4),
                    AssetKind::RasterImage(PixelSize::new(64, 64).unwrap())
                )
                .unwrap()
            }]
    ));
    let d = DocumentId::new(ui.object_id()).unwrap();
    assert!(ui.submit(
        &mut session,
        vec![Command::CreateDocument {
            document: ElementDocument::new(
                d,
                PixelSize::new(512, 512).unwrap(),
                LayerTree::default()
            )
        }]
    ));
    let first = LayerId::new(ui.object_id()).unwrap();
    let e = if cross {
        let e = DocumentId::new(ui.object_id()).unwrap();
        assert!(ui.submit(
            &mut session,
            vec![Command::CreateDocument {
                document: ElementDocument::new(
                    e,
                    PixelSize::new(512, 512).unwrap(),
                    LayerTree::default()
                )
            }]
        ));
        e
    } else {
        d
    };
    let second = LayerId::new(ui.object_id()).unwrap();
    for (doc, id, x, index) in [(d, first, 128., 0), (e, second, 220., usize::from(!cross))] {
        assert!(ui.submit(
            &mut session,
            vec![Command::AddLayer {
                document: doc,
                node: LayerNode {
                    id,
                    local: Affine2::new([1., 0., 0., 1., x, 128.]).unwrap(),
                    opacity: Opacity::OPAQUE,
                    visible: true,
                    kind: LayerKind::Image { asset: a }
                },
                parent: None,
                index
            }]
        ));
    }
    ui.select(&mut session, Some(Selection::Layer(d, first)));
    ui.target = Some(PreviewTarget::Document(d));
    let mut h = Harness {
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
    (h, Selection::Layer(d, first), Selection::Layer(e, second))
}
fn joint(h: &mut Harness, second: Selection) {
    h.ui.pick(&mut h.session, second, true);
    for _ in 0..2 {
        h.pass(vec![], [1180., 800.], true);
    }
}
fn controls(h: &mut Harness) -> gestures::ControlBox {
    let g =
        h.ui.selection_geometry(&h.session, &EditorUi::project(&h.session))
            .unwrap();
    g.controls(h.presenter.0.unwrap(), h.ui.canvas_rect.unwrap())
        .unwrap()
}
pub(super) fn click(h: &mut Harness, label: &str) {
    let p = h
        .labels
        .iter()
        .find(|(s, _)| s == label)
        .unwrap_or_else(|| panic!("label {label} absent"))
        .1
        .center();
    h.pointer(p, Some(true));
    h.pointer(p, Some(false));
}
fn near(a: Affine2, b: Affine2) {
    for (x, y) in a.coefficients().into_iter().zip(b.coefficients()) {
        assert!((x - y).abs() < 1e-6, "{a:?} != {b:?}");
    }
}

#[test]
fn ctrl_selection_and_idle_escape_are_editor_state_without_component_nodes() {
    let (mut h, first, second) = pair(true, 1024);
    let before = h.session.snapshot();
    let nodes = h.session.history_len();
    let Selection::Layer(_, l) = second else {
        unreachable!()
    };
    h.modifiers.ctrl = true;
    h.modifiers.command = true;
    click(&mut h, &format!("图层 {}", l.get()));
    assert_eq!(h.ui.selections(), [first, second]);
    assert_eq!(h.ui.target, Some(PreviewTarget::Workspace));
    assert_eq!(h.session.snapshot(), before);
    assert_eq!(h.session.history_len(), nodes);
    assert_eq!(EditorUi::project(&h.session).components().iter().len(), 0);
    h.modifiers = Default::default();
    h.pass(vec![super::key(egui::Key::Escape)], [1180., 800.], true);
    assert!(h.ui.selections().is_empty() && h.ui.selection_frame.is_none());
    assert_eq!(h.session.snapshot(), before);
    assert_eq!(h.session.history_len(), nodes);
}

#[test]
fn joint_frame_survives_repeated_edits_and_clear_keeps_committed_transforms() {
    for cross in [false, true] {
        let (mut h, first, second) = pair(cross, 1024);
        joint(&mut h, second);
        let before = h.session.snapshot();
        let nodes = h.session.history_len();
        let a = EditorUi::transform(&before.project, first).unwrap();
        let b = EditorUi::transform(&before.project, second).unwrap();
        let p = controls(&mut h).center();
        h.pointer(p, Some(true));
        h.pointer(p + Vec2::new(30., 15.), None);
        assert_eq!(h.session.snapshot(), before);
        h.pointer(p + Vec2::new(30., 15.), Some(false));
        assert_eq!(h.session.history_len(), nodes + 1);
        assert_eq!(h.ui.selections().len(), 2);
        let p = controls(&mut h).corners[2];
        h.modifiers.shift = true;
        h.pointer(p, Some(true));
        h.pointer(p + Vec2::new(20., 12.), None);
        h.pointer(p + Vec2::new(20., 12.), Some(false));
        h.modifiers = Default::default();
        let after_scale = h.session.snapshot();
        assert_eq!(h.session.history_len(), nodes + 2);
        let canvas = h.ui.canvas_rect.unwrap();
        let p = gestures::rotation(controls(&mut h), canvas);
        h.pointer(p, Some(true));
        h.pointer(p + Vec2::new(20., 18.), None);
        h.pointer(p + Vec2::new(20., 18.), Some(false));
        let after = h.session.snapshot();
        assert_eq!(h.session.history_len(), nodes + 3);
        assert_eq!(h.ui.selections().len(), 2);
        let frame = controls(&mut h);
        assert!((frame.corners[1].y - frame.corners[0].y).abs() > 1.);
        near(
            EditorUi::transform(&after.project, first)
                .unwrap()
                .compose(a.inverse().unwrap())
                .unwrap(),
            EditorUi::transform(&after.project, second)
                .unwrap()
                .compose(b.inverse().unwrap())
                .unwrap(),
        );
        assert_eq!(after.project.components(), before.project.components());
        assert_eq!(after.project.assets(), before.project.assets());
        for doc in after.project.documents() {
            assert_eq!(
                doc.layers().roots(),
                before.project.document(doc.id()).unwrap().layers().roots()
            );
        }
        h.ui.navigation(&mut h.session, HistoryAction::Undo);
        assert_eq!(h.session.snapshot().project, after_scale.project);
        h.ui.navigation(&mut h.session, HistoryAction::Redo { child: None });
        assert_eq!(h.session.snapshot().project, after.project);
        h.pass(vec![], [1180., 800.], true);
        let recovered = controls(&mut h);
        assert!(
            (recovered.corners[1].y - recovered.corners[0].y).abs() > 1.,
            "history restore must not flatten joint orientation"
        );
        h.pass(vec![super::key(egui::Key::Escape)], [1180., 800.], true);
        assert!(h.ui.selections().is_empty());
        assert_eq!(h.session.snapshot().project, after.project);
        assert_eq!(h.session.history_len(), nodes + 3);
    }
}

#[test]
fn active_escape_keeps_joint_selection_and_late_release_cannot_commit() {
    let (mut h, _, second) = pair(true, 1024);
    joint(&mut h, second);
    let before = h.session.snapshot();
    let nodes = h.session.history_len();
    let frame = controls(&mut h);
    let start = frame.center();
    h.pointer(start, Some(true));
    h.pointer(start + Vec2::new(40., 20.), None);
    assert_ne!(EditorUi::project(&h.session), before.project);
    h.pass(vec![super::key(egui::Key::Escape)], [1180., 800.], true);
    assert_eq!(h.ui.selections().len(), 2);
    assert_eq!(h.session.snapshot(), before);
    h.pointer(start + Vec2::new(70., 30.), None);
    h.pointer(start + Vec2::new(70., 30.), Some(false));
    assert_eq!(h.session.snapshot(), before);
    assert_eq!(h.session.history_len(), nodes);
    assert_eq!(controls(&mut h).corners, frame.corners);
    let start = controls(&mut h).center();
    h.pointer(start, Some(true));
    h.pointer(start + Vec2::new(20., 10.), None);
    h.pointer(start + Vec2::new(20., 10.), Some(false));
    assert_eq!(h.session.history_len(), nodes + 1);
    assert_eq!(h.ui.selections().len(), 2);
}

#[test]
fn ancestor_selection_transforms_descendants_only_once() {
    for component in [false, true] {
        let mut h = super::gesture_controls::grouped(if component { 2 } else { 1 });
        let parent = h.ui.selection().unwrap();
        let p = EditorUi::project(&h.session);
        let doc = p.documents().next().unwrap();
        let leaf = doc
            .layers()
            .iter()
            .find(|n| matches!(n.kind, LayerKind::Image { .. }))
            .unwrap()
            .id;
        let child = Selection::Layer(doc.id(), leaf);
        joint(&mut h, child);
        assert_eq!(selection::effective(&p, h.ui.selections()), vec![parent]);
        let before = h.session.snapshot();
        let local = EditorUi::transform(&before.project, child).unwrap();
        let start = controls(&mut h).center();
        h.pointer(start, Some(true));
        h.pointer(start + Vec2::new(30., 15.), None);
        h.pointer(start + Vec2::new(30., 15.), Some(false));
        assert_eq!(
            EditorUi::transform(&h.session.snapshot().project, child),
            Some(local)
        );
        assert_ne!(
            EditorUi::transform(&h.session.snapshot().project, parent),
            EditorUi::transform(&before.project, parent)
        );
    }
}

#[test]
fn joint_replace_limit_failure_is_atomic_and_does_not_advance_frame() {
    let (mut h, _, second) = pair(true, 1);
    joint(&mut h, second);
    let before = h.session.snapshot();
    let nodes = h.session.history_len();
    let frame = controls(&mut h);
    let start = frame.center();
    h.pointer(start, Some(true));
    h.pointer(start + Vec2::new(40., 15.), None);
    assert_eq!(EditorUi::project(&h.session), before.project);
    assert!(h.ui.gesture.is_none() && EditorUi::draft_state(&h.session).is_none());
    h.pointer(start + Vec2::new(40., 15.), Some(false));
    assert_eq!(h.session.snapshot(), before);
    assert_eq!(h.session.history_len(), nodes);
    assert!(EditorUi::draft_state(&h.session).is_none());
    assert_eq!(controls(&mut h).corners, frame.corners);
    assert!(!h.ui.message.is_empty());
}

#[test]
fn clearing_selection_does_not_cancel_a_foreign_application_draft() {
    let (mut h, _, second) = pair(true, 1024);
    joint(&mut h, second);
    let info = h.session.project_info();
    let before = h.session.snapshot();
    h.session
        .edit_draft(DraftRequest {
            api_version: API_VERSION,
            session_id: info.session_id,
            request_id: RequestId::new(60000).unwrap(),
            action: DraftAction::Begin {
                expected: info.state,
            },
        })
        .unwrap();
    let draft = EditorUi::draft_state(&h.session).unwrap();
    h.pass(vec![super::key(egui::Key::Escape)], [1180., 800.], true);
    assert_eq!(EditorUi::draft_state(&h.session), Some(draft));
    assert!(h.ui.selections().is_empty());
    assert_eq!(h.session.snapshot(), before);
}

#[test]
fn component_panel_distinguishes_group_and_member_without_global_creation() {
    let mut h = super::gesture_controls::grouped(2);
    h.ctx
        .style_mut_of(egui::Theme::Dark, |s| s.animation_time = 0.);
    h.ctx
        .style_mut_of(egui::Theme::Light, |s| s.animation_time = 0.);
    let before = h.session.snapshot();
    let nodes = h.session.history_len();
    click(&mut h, "跨元素组件");
    h.pass(vec![], [1180., 800.], true);
    assert!(!h.labels.iter().any(|(s, _)| s.contains("建立包含所有元素")));
    assert_eq!(
        h.labels
            .iter()
            .filter(|(s, _)| s.starts_with("组件组 "))
            .count(),
        1,
        "{:?}",
        h.labels
    );
    assert_eq!(
        h.labels
            .iter()
            .filter(|(s, _)| s.starts_with("元素引用 "))
            .count(),
        1
    );
    assert_eq!(h.session.snapshot(), before);
    assert_eq!(h.session.history_len(), nodes);
}

#[test]
fn mapped_component_document_and_loose_layer_share_workspace_delta() {
    let (mut h, first, second) = pair(true, 1024);
    let Selection::Layer(document, _) = first else {
        unreachable!()
    };
    let group = ComponentId::new(h.ui.object_id()).unwrap();
    let member = ComponentId::new(h.ui.object_id()).unwrap();
    assert!(h.ui.submit(
        &mut h.session,
        vec![
            Command::AddComponent {
                node: ComponentNode {
                    id: group,
                    local: Affine2::new([0., 1., -1., 0., 300., 100.]).unwrap(),
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
                        document,
                        document_to_component: Affine2::new([2., 0., 0., 0.5, 10., 20.]).unwrap()
                    }
                },
                parent: Some(group),
                index: 0
            }
        ]
    ));
    joint(&mut h, second);
    let before = h.session.snapshot();
    let prefix = before
        .project
        .document_workspace_transform(document)
        .unwrap();
    let old_a = prefix
        .compose(EditorUi::transform(&before.project, first).unwrap())
        .unwrap();
    let old_b = EditorUi::transform(&before.project, second).unwrap();
    let start = controls(&mut h).center();
    h.pointer(start, Some(true));
    h.pointer(start + Vec2::new(15., 10.), None);
    h.pointer(start + Vec2::new(15., 10.), Some(false));
    let after = h.session.snapshot();
    let new_a = prefix
        .compose(EditorUi::transform(&after.project, first).unwrap())
        .unwrap();
    let new_b = EditorUi::transform(&after.project, second).unwrap();
    near(
        new_a.compose(old_a.inverse().unwrap()).unwrap(),
        new_b.compose(old_b.inverse().unwrap()).unwrap(),
    );
    assert_eq!(before.project.components(), after.project.components());
}

#[test]
fn singular_parent_rejects_joint_gesture_before_begin_without_partial_changes() {
    let (mut h, _, second) = pair(true, 1024);
    let Selection::Layer(document, layer) = second else {
        unreachable!()
    };
    let group = LayerId::new(h.ui.object_id()).unwrap();
    assert!(h.ui.submit(
        &mut h.session,
        vec![
            Command::AddLayer {
                document,
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
                document,
                layer,
                parent: Some(group),
                index: 0
            },
            Command::SetLayerTransform {
                document,
                layer: group,
                transform: Affine2::new([0., 0., 0., 0., 0., 0.]).unwrap()
            }
        ]
    ));
    joint(&mut h, second);
    let before = h.session.snapshot();
    let nodes = h.session.history_len();
    let start = controls(&mut h).center();
    h.pointer(start, Some(true));
    h.pointer(start + Vec2::new(15., 10.), None);
    h.pointer(start + Vec2::new(15., 10.), Some(false));
    assert_eq!(h.session.snapshot(), before);
    assert_eq!(h.session.history_len(), nodes);
    assert!(EditorUi::draft_state(&h.session).is_none());
}

#[test]
fn fault_cancels_joint_draft_and_late_release_preserves_the_project() {
    let (mut h, _, second) = pair(true, 1024);
    joint(&mut h, second);
    let before = h.session.snapshot();
    let nodes = h.session.history_len();
    let start = controls(&mut h).center();
    h.pointer(start, Some(true));
    h.pointer(start + Vec2::new(15., 10.), None);
    assert!(EditorUi::draft_state(&h.session).is_some());
    assert!(h.ui.interrupt_for_fault(&mut h.session));
    h.pointer(start + Vec2::new(40., 30.), None);
    h.pointer(start + Vec2::new(40., 30.), Some(false));
    assert_eq!(h.session.snapshot(), before);
    assert_eq!(h.session.history_len(), nodes);
    assert_eq!(h.ui.selections().len(), 2);
}

#[test]
fn selection_is_bounded_and_blank_click_releases_only_the_temporary_frame() {
    let (mut h, first, _) = pair(false, 1024);
    let Selection::Layer(document, _) = first else {
        unreachable!()
    };
    let asset = h
        .session
        .snapshot()
        .project
        .assets()
        .iter()
        .next()
        .unwrap()
        .id();
    let mut commands = Vec::new();
    let mut items = Vec::new();
    for index in 2..=selection::MAX_SELECTED + 2 {
        let id = LayerId::new(h.ui.object_id()).unwrap();
        commands.push(Command::AddLayer {
            document,
            node: LayerNode {
                id,
                local: Affine2::IDENTITY,
                opacity: Opacity::OPAQUE,
                visible: true,
                kind: LayerKind::Image { asset },
            },
            parent: None,
            index,
        });
        items.push(Selection::Layer(document, id));
    }
    assert!(h.ui.submit(&mut h.session, commands));
    let before = h.session.snapshot();
    let nodes = h.session.history_len();
    for item in items {
        h.ui.toggle_selection(&mut h.session, item);
    }
    assert_eq!(h.ui.selections().len(), selection::MAX_SELECTED);
    assert!(h.ui.message.contains("256"));
    assert_eq!(h.session.snapshot(), before);
    assert_eq!(h.session.history_len(), nodes);
    // A normal two-member selection also clears by clicking canvas whitespace.
    let (mut h, _, second) = pair(true, 1024);
    joint(&mut h, second);
    let before = h.session.snapshot();
    let nodes = h.session.history_len();
    let p = h.ui.canvas_rect.unwrap().left_bottom() + Vec2::new(20., -20.);
    h.pointer(p, Some(true));
    h.pointer(p, Some(false));
    assert!(h.ui.selections().is_empty());
    assert_eq!(h.session.snapshot(), before);
    assert_eq!(h.session.history_len(), nodes);
}

#[test]
fn ambiguous_layer_import_does_not_create_a_document_or_component() {
    let (mut h, _, second) = pair(true, 1024);
    joint(&mut h, second);
    let before = h.session.snapshot();
    let nodes = h.session.history_len();
    let asset = before.project.assets().iter().next().unwrap().id();
    h.ui.add_asset_layer(&mut h.session, asset);
    assert_eq!(h.session.snapshot(), before);
    assert_eq!(h.session.history_len(), nodes);
    assert!(!h.ui.message.is_empty());
}

#[test]
fn old_preview_descriptor_cannot_start_a_joint_edit() {
    let (mut h, _, second) = pair(true, 1024);
    joint(&mut h, second);
    let before = h.session.snapshot();
    let nodes = h.session.history_len();
    let start = controls(&mut h).center();
    h.presenter.0.as_mut().unwrap().frame.api_version = 1;
    h.pointer(start, Some(true));
    h.presenter.0.as_mut().unwrap().frame.api_version = 1;
    h.pointer(start + Vec2::new(20., 10.), None);
    h.pointer(start + Vec2::new(20., 10.), Some(false));
    assert_eq!(h.session.snapshot(), before);
    assert_eq!(h.session.history_len(), nodes);
    assert!(EditorUi::draft_state(&h.session).is_none());
}

#[test]
fn empty_group_has_no_bounds_but_remains_in_a_joint_transform() {
    let (mut h, first, _) = pair(false, 1024);
    let Selection::Layer(document, _) = first else {
        unreachable!()
    };
    let group = LayerId::new(h.ui.object_id()).unwrap();
    assert!(h.ui.submit(
        &mut h.session,
        vec![Command::AddLayer {
            document,
            node: LayerNode {
                id: group,
                local: Affine2::IDENTITY,
                opacity: Opacity::OPAQUE,
                visible: true,
                kind: LayerKind::Group { children: vec![] }
            },
            parent: None,
            index: 2
        }]
    ));
    joint(&mut h, Selection::Layer(document, group));
    let before = h.session.snapshot();
    let nodes = h.session.history_len();
    let start = controls(&mut h).center();
    h.pointer(start, Some(true));
    h.pointer(start + Vec2::new(20., 10.), None);
    h.pointer(start + Vec2::new(20., 10.), Some(false));
    let after = h.session.snapshot();
    assert_ne!(
        EditorUi::transform(&after.project, first),
        EditorUi::transform(&before.project, first)
    );
    assert_ne!(
        EditorUi::transform(&after.project, Selection::Layer(document, group)),
        Some(Affine2::IDENTITY)
    );
    assert!(
        after
            .project
            .document(document)
            .unwrap()
            .layers()
            .node(group)
            .unwrap()
            .children()
            .is_empty()
    );
    assert_eq!(h.session.history_len(), nodes + 1);
}

#[test]
fn replaced_application_draft_is_not_modified_by_old_joint_input_or_commit() {
    let (mut h, _, second) = pair(true, 1024);
    joint(&mut h, second);
    let before = h.session.snapshot();
    let nodes = h.session.history_len();
    let start = controls(&mut h).center();
    h.pointer(start, Some(true));
    h.pointer(start + Vec2::new(20., 10.), None);
    let old = EditorUi::draft_state(&h.session).unwrap();
    let info = h.session.project_info();
    h.session
        .edit_draft(DraftRequest {
            api_version: API_VERSION,
            session_id: info.session_id,
            request_id: RequestId::new(60000).unwrap(),
            action: DraftAction::Cancel { expected: old },
        })
        .unwrap();
    h.session
        .edit_draft(DraftRequest {
            api_version: API_VERSION,
            session_id: info.session_id,
            request_id: RequestId::new(60001).unwrap(),
            action: DraftAction::Begin {
                expected: info.state,
            },
        })
        .unwrap();
    let foreign = EditorUi::draft_state(&h.session).unwrap();
    h.pointer(start + Vec2::new(60., 30.), None);
    h.pointer(start + Vec2::new(60., 30.), Some(false));
    h.ui.commit(&mut h.session);
    assert_eq!(EditorUi::draft_state(&h.session), Some(foreign));
    assert_eq!(EditorUi::project(&h.session), before.project);
    assert_eq!(h.session.snapshot(), before);
    assert_eq!(h.session.history_len(), nodes);
}
