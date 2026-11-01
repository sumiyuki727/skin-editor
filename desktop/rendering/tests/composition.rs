use skin_editor_domain::*;
use skin_editor_rendering::*;
use std::collections::BTreeMap;
use std::sync::Arc;
fn fixture() -> (SkinProject, Images) {
    let mut input = ProjectDefinition::empty(ProjectId::new(1).unwrap());
    let blob = BlobRef::new([1; 32], 4);
    input.assets.push(
        Asset::new(
            AssetId::new(1).unwrap(),
            "pixel".into(),
            blob,
            AssetKind::RasterImage(PixelSize::new(1, 1).unwrap()),
        )
        .unwrap(),
    );
    let layers = LayerTree::new(
        vec![LayerId::new(1).unwrap()],
        vec![
            LayerNode {
                id: LayerId::new(1).unwrap(),
                local: Affine2::IDENTITY,
                opacity: Opacity::new(0.5).unwrap(),
                visible: true,
                kind: LayerKind::Group {
                    children: vec![LayerId::new(2).unwrap(), LayerId::new(3).unwrap()],
                },
            },
            LayerNode {
                id: LayerId::new(2).unwrap(),
                local: Affine2::IDENTITY,
                opacity: Opacity::OPAQUE,
                visible: true,
                kind: LayerKind::Image {
                    asset: AssetId::new(1).unwrap(),
                },
            },
            LayerNode {
                id: LayerId::new(3).unwrap(),
                local: Affine2::IDENTITY,
                opacity: Opacity::OPAQUE,
                visible: true,
                kind: LayerKind::Image {
                    asset: AssetId::new(1).unwrap(),
                },
            },
        ],
    )
    .unwrap();
    input.documents.push(ElementDocument::new(
        DocumentId::new(1).unwrap(),
        PixelSize::new(1, 1).unwrap(),
        layers,
    ));
    (
        SkinProject::from_definition(input).unwrap(),
        BTreeMap::from([(
            blob,
            Arc::new(Raster {
                size: PixelSize::new(1, 1).unwrap(),
                pixels: vec![255, 0, 0, 255].into(),
            }),
        )]),
    )
}
#[test]
fn group_opacity_is_isolated_and_not_applied_to_each_child() {
    let (project, images) = fixture();
    let plan = Plan::build(
        &project,
        Target::Document(DocumentId::new(1).unwrap()),
        PixelSize::new(1, 1).unwrap(),
    )
    .unwrap();
    assert_eq!(compose(&plan, &images, false).unwrap(), [255, 0, 0, 128]);
    assert_eq!(compose(&plan, &images, true).unwrap()[3], 255);
}

#[test]
fn workspace_draws_unassociated_documents_without_creating_components_or_changing_outputs() {
    let (project, images) = fixture();
    let before = project.clone();
    let size = PixelSize::new(512, 512).unwrap();
    let components = compose(
        &Plan::build(&project, Target::Components, size).unwrap(),
        &images,
        false,
    )
    .unwrap();
    assert!(components.iter().all(|v| *v == 0));
    let workspace = compose(
        &Plan::build(&project, Target::Workspace, size).unwrap(),
        &images,
        false,
    )
    .unwrap();
    assert_eq!(&workspace[..4], &[255, 0, 0, 128]);
    let document = compose(
        &Plan::build(
            &project,
            Target::Document(DocumentId::new(1).unwrap()),
            PixelSize::new(1, 1).unwrap(),
        )
        .unwrap(),
        &images,
        false,
    )
    .unwrap();
    assert_eq!(&workspace[..4], document.as_slice());
    assert_eq!(project, before);
    assert_eq!(project.components().iter().len(), 0);
}
#[test]
fn missing_source_and_preview_budget_fail_instead_of_rendering_partial_pixels() {
    let (project, _) = fixture();
    let target = Target::Document(DocumentId::new(1).unwrap());
    let plan = Plan::build(&project, target, PixelSize::new(1, 1).unwrap()).unwrap();
    assert!(compose(&plan, &BTreeMap::new(), false).is_err());
    assert!(Plan::build(&project, target, PixelSize::new(2048, 2048).unwrap()).is_err());
}

#[test]
fn hidden_child_removes_only_its_contribution_and_hidden_group_removes_subtree() {
    let (project, images) = fixture();
    let old = project.documents().next().unwrap();
    let mut nodes: Vec<_> = old.layers().iter().cloned().collect();
    let second = nodes
        .iter_mut()
        .find(|n| n.id == LayerId::new(3).unwrap())
        .unwrap();
    second.local = Affine2::new([1., 0., 0., 1., 1., 0.]).unwrap();
    second.visible = false;
    let mut definition = ProjectDefinition::empty(project.id());
    definition.assets = project.assets().iter().cloned().collect();
    definition.documents.push(ElementDocument::new(
        old.id(),
        PixelSize::new(2, 1).unwrap(),
        LayerTree::new(old.layers().roots().to_vec(), nodes.clone()).unwrap(),
    ));
    let project = SkinProject::from_definition(definition.clone()).unwrap();
    let plan = Plan::build(
        &project,
        Target::Document(old.id()),
        PixelSize::new(2, 1).unwrap(),
    )
    .unwrap();
    assert_eq!(
        compose(&plan, &images, false).unwrap(),
        [255, 0, 0, 128, 0, 0, 0, 0]
    );
    nodes
        .iter_mut()
        .find(|n| n.id == LayerId::new(1).unwrap())
        .unwrap()
        .visible = false;
    definition.documents[0] = ElementDocument::new(
        old.id(),
        PixelSize::new(2, 1).unwrap(),
        LayerTree::new(old.layers().roots().to_vec(), nodes).unwrap(),
    );
    let project = SkinProject::from_definition(definition).unwrap();
    let plan = Plan::build(
        &project,
        Target::Document(old.id()),
        PixelSize::new(2, 1).unwrap(),
    )
    .unwrap();
    assert_eq!(compose(&plan, &Images::new(), false).unwrap(), [0; 8]);
}

#[test]
fn hidden_element_keeps_output_and_canvas_but_needs_no_source_pixels() {
    let (project, _) = fixture();
    let old = project.documents().next().unwrap();
    let mut definition = ProjectDefinition::empty(project.id());
    definition.assets = project.assets().iter().cloned().collect();
    definition
        .documents
        .push(old.clone().with_visibility(false));
    definition.outputs.push(
        OutputMapping::new(
            OutputId::new(1).unwrap(),
            old.id(),
            "fixture.image".into(),
            OutputPath::new("image.png".into()).unwrap(),
        )
        .unwrap(),
    );
    let project = SkinProject::from_definition(definition).unwrap();
    let plan = Plan::build(
        &project,
        Target::Document(old.id()),
        PixelSize::new(1, 1).unwrap(),
    )
    .unwrap();
    assert_eq!(compose(&plan, &Images::new(), false).unwrap(), [0; 4]);
    assert_eq!(compose(&plan, &Images::new(), true).unwrap()[3], 255);
    assert_eq!(project.outputs().len(), 1);
    assert_eq!(project.document(old.id()).unwrap().canvas(), old.canvas());
}

#[test]
fn component_group_translation_changes_document_pixels_but_preserves_output_clip() {
    let blob = BlobRef::new([2; 32], 4);
    let mut definition = ProjectDefinition::empty(ProjectId::new(2).unwrap());
    definition.assets.push(
        Asset::new(
            AssetId::new(1).unwrap(),
            "pixel".into(),
            blob,
            AssetKind::RasterImage(PixelSize::new(1, 1).unwrap()),
        )
        .unwrap(),
    );
    let document = DocumentId::new(1).unwrap();
    let layer = LayerId::new(1).unwrap();
    definition.documents.push(ElementDocument::new(
        document,
        PixelSize::new(4, 1).unwrap(),
        LayerTree::new(
            vec![layer],
            vec![LayerNode {
                id: layer,
                local: Affine2::IDENTITY,
                opacity: Opacity::OPAQUE,
                visible: true,
                kind: LayerKind::Image {
                    asset: AssetId::new(1).unwrap(),
                },
            }],
        )
        .unwrap(),
    ));
    let group = ComponentId::new(1).unwrap();
    let member = ComponentId::new(2).unwrap();
    definition.components = ComponentTree::new(
        vec![group],
        vec![
            ComponentNode {
                id: group,
                local: Affine2::new([1., 0., 0., 1., 2., 0.]).unwrap(),
                kind: ComponentKind::Group {
                    children: vec![member],
                },
            },
            ComponentNode {
                id: member,
                local: Affine2::IDENTITY,
                kind: ComponentKind::Element {
                    document,
                    document_to_component: Affine2::IDENTITY,
                },
            },
        ],
    )
    .unwrap();
    let project = SkinProject::from_definition(definition).unwrap();
    let images = BTreeMap::from([(
        blob,
        Arc::new(Raster {
            size: PixelSize::new(1, 1).unwrap(),
            pixels: vec![20, 80, 200, 255].into(),
        }),
    )]);
    let plan = Plan::build(
        &project,
        Target::Document(document),
        PixelSize::new(4, 1).unwrap(),
    )
    .unwrap();
    assert_eq!(
        compose(&plan, &images, false).unwrap(),
        [0, 0, 0, 0, 0, 0, 0, 0, 20, 80, 200, 255, 0, 0, 0, 0]
    );
    // The same content transform participates in the component plan.
    let plan = Plan::build(
        &project,
        Target::Components,
        PixelSize::new(512, 512).unwrap(),
    )
    .unwrap();
    let pixels = compose(&plan, &images, false).unwrap();
    assert_eq!(&pixels[8..12], &[20, 80, 200, 255]);
    assert_eq!(
        pixels
            .iter()
            .skip(3)
            .step_by(4)
            .filter(|a| **a != 0)
            .count(),
        1
    );
}

#[test]
fn excessive_group_depth_is_explicit_error() {
    let mut definition = ProjectDefinition::empty(ProjectId::new(3).unwrap());
    let document = DocumentId::new(1).unwrap();
    let nodes = (1..=33)
        .map(|i| LayerNode {
            id: LayerId::new(i).unwrap(),
            local: Affine2::IDENTITY,
            opacity: Opacity::OPAQUE,
            visible: true,
            kind: LayerKind::Group {
                children: if i < 33 {
                    vec![LayerId::new(i + 1).unwrap()]
                } else {
                    vec![]
                },
            },
        })
        .collect();
    definition.documents.push(ElementDocument::new(
        document,
        PixelSize::new(1, 1).unwrap(),
        LayerTree::new(vec![LayerId::new(1).unwrap()], nodes).unwrap(),
    ));
    let project = SkinProject::from_definition(definition).unwrap();
    assert!(
        Plan::build(
            &project,
            Target::Document(document),
            PixelSize::new(1, 1).unwrap()
        )
        .unwrap_err()
        .0
        .contains("depth")
    );
}
#[test]
fn excessive_operation_count_does_not_publish_a_truncated_image() {
    let mut definition = ProjectDefinition::empty(ProjectId::new(4).unwrap());
    let ids: Vec<_> = (1..=32769).map(|id| LayerId::new(id).unwrap()).collect();
    let nodes = ids
        .iter()
        .map(|id| LayerNode {
            id: *id,
            local: Affine2::IDENTITY,
            opacity: Opacity::OPAQUE,
            visible: true,
            kind: LayerKind::Group { children: vec![] },
        })
        .collect();
    definition.documents.push(ElementDocument::new(
        DocumentId::new(1).unwrap(),
        PixelSize::new(1, 1).unwrap(),
        LayerTree::new(ids, nodes).unwrap(),
    ));
    let project = SkinProject::from_definition(definition).unwrap();
    assert!(
        Plan::build(
            &project,
            Target::Document(DocumentId::new(1).unwrap()),
            PixelSize::new(1, 1).unwrap()
        )
        .unwrap_err()
        .0
        .contains("budget")
    );
}
