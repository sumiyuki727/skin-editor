use skin_editor_domain::*;

fn aid(n: u128) -> AssetId {
    AssetId::new(n).unwrap()
}
fn did(n: u128) -> DocumentId {
    DocumentId::new(n).unwrap()
}
fn lid(n: u128) -> LayerId {
    LayerId::new(n).unwrap()
}
fn cid(n: u128) -> ComponentId {
    ComponentId::new(n).unwrap()
}
fn oid(n: u128) -> OutputId {
    OutputId::new(n).unwrap()
}
fn size(w: u32, h: u32) -> PixelSize {
    PixelSize::new(w, h).unwrap()
}
fn affine(v: [f64; 6]) -> Affine2 {
    Affine2::new(v).unwrap()
}
fn translation(x: f64, y: f64) -> Affine2 {
    affine([1.0, 0.0, 0.0, 1.0, x, y])
}
fn scale(x: f64, y: f64) -> Affine2 {
    affine([x, 0.0, 0.0, y, 0.0, 0.0])
}
fn close(a: [f64; 2], b: [f64; 2]) {
    for i in 0..2 {
        assert!((a[i] - b[i]).abs() < 1e-9, "{a:?} != {b:?}");
    }
}
fn image(id: u128, asset: u128, local: Affine2) -> LayerNode {
    LayerNode {
        id: lid(id),
        local,
        opacity: Opacity::OPAQUE,
        visible: true,
        kind: LayerKind::Image { asset: aid(asset) },
    }
}
fn group(id: u128, children: &[u128], local: Affine2) -> LayerNode {
    LayerNode {
        id: lid(id),
        local,
        opacity: Opacity::OPAQUE,
        visible: true,
        kind: LayerKind::Group {
            children: children.iter().map(|n| lid(*n)).collect(),
        },
    }
}
fn cgroup(id: u128, children: &[u128], local: Affine2) -> ComponentNode {
    ComponentNode {
        id: cid(id),
        local,
        kind: ComponentKind::Group {
            children: children.iter().map(|n| cid(*n)).collect(),
        },
    }
}
fn member(id: u128, doc: u128, map: Affine2) -> ComponentNode {
    ComponentNode {
        id: cid(id),
        local: Affine2::IDENTITY,
        kind: ComponentKind::Element {
            document: did(doc),
            document_to_component: map,
        },
    }
}
fn document(id: u128, roots: &[u128], nodes: Vec<LayerNode>) -> ElementDocument {
    ElementDocument::new(
        did(id),
        size(10, 20),
        LayerTree::new(roots.iter().map(|n| lid(*n)).collect(), nodes).unwrap(),
    )
}
fn asset(id: u128) -> Asset {
    Asset::new(
        aid(id),
        format!("source {id}"),
        BlobRef::new([1; 32], 100),
        AssetKind::RasterImage(size(10, 20)),
    )
    .unwrap()
}
fn definition() -> ProjectDefinition {
    let mut input = ProjectDefinition::empty(ProjectId::new(7).unwrap());
    input.assets = vec![asset(1)];
    input.documents = vec![document(1, &[1], vec![image(1, 1, Affine2::IDENTITY)])];
    input
}

#[test]
fn workspace_document_query_preserves_membership_and_includes_member_mapping() {
    let plain = SkinProject::from_definition(definition()).unwrap();
    assert_eq!(
        plain.document_workspace_transform(did(1)),
        Ok(Affine2::IDENTITY)
    );
    assert_eq!(plain.components().iter().len(), 0);
    assert!(plain.document_workspace_transform(did(99)).is_err());
    let mut input = definition();
    let world = affine([0., 2., -3., 0., 40., 50.]);
    let mapping = affine([2., 0., 0., 0.5, 10., 20.]);
    input.components = ComponentTree::new(
        vec![cid(1)],
        vec![
            ComponentNode {
                id: cid(1),
                local: world,
                kind: ComponentKind::Group {
                    children: vec![cid(2)],
                },
            },
            ComponentNode {
                id: cid(2),
                local: translation(3., 4.),
                kind: ComponentKind::Element {
                    document: did(1),
                    document_to_component: mapping,
                },
            },
        ],
    )
    .unwrap();
    let project = SkinProject::from_definition(input).unwrap();
    let before = project.clone();
    assert_eq!(
        project.document_workspace_transform(did(1)),
        Ok(world
            .compose(translation(3., 4.))
            .unwrap()
            .compose(mapping)
            .unwrap())
    );
    assert_eq!(project, before);
}
fn output(id: u128, doc: u128, name: &str) -> OutputMapping {
    OutputMapping::new(
        oid(id),
        did(doc),
        "explicit-element".into(),
        OutputPath::new(name.into()).unwrap(),
    )
    .unwrap()
}

#[test]
fn object_id_types_preserve_full_width_and_reject_zero() {
    assert!(AssetId::new(0).is_none());
    assert!(DocumentId::new(0).is_none());
    assert!(LayerId::new(0).is_none());
    assert!(ComponentId::new(0).is_none());
    assert!(OutputId::new(0).is_none());
    assert_eq!(aid(u128::MAX).get(), u128::MAX);
    assert_ne!(ObjectId::Asset(aid(1)), ObjectId::Document(did(1)));
}

#[test]
fn numeric_values_reject_nonfinite_invalid_extents_and_byte_overflow() {
    for value in [f32::NAN, f32::INFINITY, -0.1, 1.1] {
        assert!(Opacity::new(value).is_err());
    }
    assert_eq!(Opacity::new(0.0).unwrap().get(), 0.0);
    assert!(PixelSize::new(0, 1).is_err());
    assert!(PixelSize::new(1, 0).is_err());
    assert_eq!(
        PixelSize::new(u32::MAX, u32::MAX),
        Err(ModelError::NumericOverflow)
    );
    assert!(Affine2::new([f64::NAN; 6]).is_err());
    assert!(Affine2::new([f64::INFINITY; 6]).is_err());
    assert!(scale(1e200, 1.0).compose(scale(1e200, 1.0)).is_err());
    assert!(scale(1e308, 1.0).point(10.0, 0.0).is_err());
}

#[test]
fn rebase_preserves_world_geometry_including_shear_and_reflection() {
    let old = affine([0.0, 2.0, -3.0, 0.5, 15.0, -8.0]);
    let parent = affine([-2.0, 0.25, 0.5, 3.0, 4.0, 7.0]);
    let local = Affine2::rebased(old, parent).unwrap();
    let restored = parent.compose(local).unwrap();
    close(restored.point(3.0, -7.0).unwrap(), [36.0, -5.5]);
    close(restored.point(0.0, 0.0).unwrap(), [15.0, -8.0]);
    assert_eq!(
        Affine2::rebased(old, scale(0.0, 1.0)),
        Err(ModelError::NotInvertible)
    );
    assert!(scale(1e-300, 1e-300).inverse().is_err());
}

#[test]
fn layer_groups_keep_order_local_parameters_and_composed_geometry() {
    let mut child = image(3, 1, translation(2.0, 4.0));
    child.opacity = Opacity::new(0.8).unwrap();
    let mut parent = group(1, &[2, 4], translation(10.0, 20.0));
    parent.opacity = Opacity::new(0.5).unwrap();
    let tree = LayerTree::new(
        vec![lid(1)],
        vec![
            child.clone(),
            parent,
            image(4, 1, Affine2::IDENTITY),
            group(2, &[3], scale(2.0, 3.0)),
        ],
    )
    .unwrap();
    assert_eq!(
        tree.iter().map(|n| n.id.get()).collect::<Vec<_>>(),
        [1, 2, 3, 4]
    );
    assert_eq!(tree.parent(lid(3)), Some(Some(lid(2))));
    assert_eq!(tree.parent(lid(1)), Some(None));
    assert_eq!(tree.parent(lid(99)), None);
    close(
        tree.world_transform(lid(3))
            .unwrap()
            .point(1.0, 1.0)
            .unwrap(),
        [16.0, 35.0],
    );
    assert_eq!(tree.node(lid(3)).unwrap(), &child);
    // Group opacity remains isolated metadata, not multiplied into each child.
    assert_eq!(tree.node(lid(1)).unwrap().opacity.get(), 0.5);
    assert_eq!(tree.node(lid(3)).unwrap().opacity.get(), 0.8);
}

#[test]
fn layer_forests_reject_duplicate_ids_roots_and_missing_edges() {
    let a = group(1, &[], Affine2::IDENTITY);
    assert!(matches!(
        LayerTree::new(vec![lid(1)], vec![a.clone(), a.clone()]),
        Err(ModelError::DuplicateId(_))
    ));
    assert!(matches!(
        LayerTree::new(vec![lid(1), lid(1)], vec![a.clone()]),
        Err(ModelError::MultipleParents(_))
    ));
    assert!(matches!(
        LayerTree::new(vec![lid(2)], vec![a.clone()]),
        Err(ModelError::MissingReference(_))
    ));
    assert!(matches!(
        LayerTree::new(vec![lid(1)], vec![group(1, &[2], Affine2::IDENTITY)]),
        Err(ModelError::MissingReference(_))
    ));
    assert!(matches!(
        LayerTree::new(vec![], vec![a]),
        Err(ModelError::Unreachable(_))
    ));
}

#[test]
fn layer_forests_reject_multiple_parents_and_duplicate_children() {
    for nodes in [
        vec![
            group(1, &[3], Affine2::IDENTITY),
            group(2, &[3], Affine2::IDENTITY),
            group(3, &[], Affine2::IDENTITY),
        ],
        vec![
            group(1, &[3, 3], Affine2::IDENTITY),
            group(2, &[], Affine2::IDENTITY),
            group(3, &[], Affine2::IDENTITY),
        ],
    ] {
        assert!(matches!(
            LayerTree::new(vec![lid(1), lid(2)], nodes),
            Err(ModelError::MultipleParents(_))
        ));
    }
    assert!(matches!(
        LayerTree::new(
            vec![lid(1), lid(2)],
            vec![
                group(1, &[2], Affine2::IDENTITY),
                group(2, &[], Affine2::IDENTITY)
            ]
        ),
        Err(ModelError::MultipleParents(_))
    ));
}

#[test]
fn disconnected_and_self_cycles_are_rejected() {
    assert_eq!(
        LayerTree::new(vec![], vec![group(1, &[1], Affine2::IDENTITY)]),
        Err(ModelError::Cycle)
    );
    assert_eq!(
        LayerTree::new(
            vec![lid(3)],
            vec![
                group(1, &[2], Affine2::IDENTITY),
                group(2, &[1], Affine2::IDENTITY),
                group(3, &[], Affine2::IDENTITY)
            ]
        ),
        Err(ModelError::Cycle)
    );
}

#[test]
fn accumulated_world_overflow_is_rejected_before_project_publication() {
    assert_eq!(
        LayerTree::new(
            vec![lid(1)],
            vec![
                group(1, &[2], scale(1e200, 1.0)),
                group(2, &[], scale(1e200, 1.0))
            ]
        ),
        Err(ModelError::NumericOverflow)
    );
}

#[test]
fn deep_layer_tree_build_traversal_clone_and_drop_are_iterative() {
    let nodes = (1..=10_000u128)
        .map(|id| {
            group(
                id,
                &if id == 10_000 { vec![] } else { vec![id + 1] },
                translation(1.0, 0.0),
            )
        })
        .collect();
    let tree = LayerTree::new(vec![lid(1)], nodes).unwrap();
    assert_eq!(tree.iter().len(), 10_000);
    close(
        tree.world_transform(lid(10_000))
            .unwrap()
            .point(0.0, 0.0)
            .unwrap(),
        [10_000.0, 0.0],
    );
    assert_eq!(tree.clone(), tree);
}

#[test]
fn library_shares_blob_content_but_keeps_asset_identity() {
    let library = AssetLibrary::new(vec![asset(1), asset(2)]).unwrap();
    assert_eq!(library.iter().len(), 2);
    assert_eq!(
        library.get(aid(1)).unwrap().blob(),
        library.get(aid(2)).unwrap().blob()
    );
    assert_ne!(
        library.get(aid(1)).unwrap().name(),
        library.get(aid(2)).unwrap().name()
    );
}

#[test]
fn library_rejects_duplicate_identity_and_contradictory_blob_metadata() {
    assert!(matches!(
        AssetLibrary::new(vec![asset(1), asset(1)]),
        Err(ModelError::DuplicateId(_))
    ));
    let bad_length = Asset::new(
        aid(2),
        "b".into(),
        BlobRef::new([1; 32], 101),
        AssetKind::Opaque,
    )
    .unwrap();
    assert!(AssetLibrary::new(vec![asset(1), bad_length]).is_err());
    let bad_size = Asset::new(
        aid(2),
        "b".into(),
        BlobRef::new([1; 32], 100),
        AssetKind::RasterImage(size(20, 10)),
    )
    .unwrap();
    assert!(AssetLibrary::new(vec![asset(1), bad_size]).is_err());
    let opaque = Asset::new(
        aid(2),
        "uninterpreted".into(),
        BlobRef::new([1; 32], 100),
        AssetKind::Opaque,
    )
    .unwrap();
    assert!(AssetLibrary::new(vec![opaque, asset(1)]).is_ok());
}

#[test]
fn shared_image_layers_have_independent_parameters_and_full_geometric_bounds() {
    let mut input = definition();
    let mut second = image(2, 1, affine([0.0, 1.0, -1.0, 0.0, 40.0, 0.0]));
    second.visible = false;
    second.opacity = Opacity::new(0.25).unwrap();
    input.documents = vec![document(
        1,
        &[3],
        vec![
            group(3, &[1, 2], Affine2::IDENTITY),
            image(1, 1, Affine2::IDENTITY),
            second,
        ],
    )];
    let project = SkinProject::from_definition(input).unwrap();
    let bounds = project.layer_bounds(did(1), lid(3)).unwrap().unwrap();
    close(bounds.min(), [0.0, 0.0]);
    close(bounds.max(), [40.0, 20.0]);
    assert_eq!(project.assets().get(aid(1)).unwrap().blob().byte_len(), 100);
    assert_eq!(
        project
            .document(did(1))
            .unwrap()
            .layers()
            .node(lid(1))
            .unwrap()
            .opacity,
        Opacity::OPAQUE
    );
}

#[test]
fn empty_groups_have_no_bounds_but_keep_their_transform() {
    let mut input = definition();
    input.documents = vec![document(
        1,
        &[1],
        vec![group(1, &[], translation(7.0, 8.0))],
    )];
    input.components =
        ComponentTree::new(vec![cid(1)], vec![cgroup(1, &[], scale(0.0, 1.0))]).unwrap();
    let project = SkinProject::from_definition(input).unwrap();
    assert_eq!(project.layer_bounds(did(1), lid(1)).unwrap(), None);
    assert_eq!(project.component_bounds(cid(1)).unwrap(), None);
    assert_eq!(
        project
            .document(did(1))
            .unwrap()
            .layers()
            .world_transform(lid(1)),
        Some(translation(7.0, 8.0))
    );
}

#[test]
fn project_rejects_missing_image_sources_and_nonimage_layers() {
    let mut input = definition();
    input.assets.clear();
    assert_eq!(
        SkinProject::from_definition(input),
        Err(ModelError::MissingReference(ObjectId::Asset(aid(1))))
    );
    let mut input = definition();
    input.assets = vec![
        Asset::new(
            aid(1),
            "opaque".into(),
            BlobRef::new([1; 32], 100),
            AssetKind::Opaque,
        )
        .unwrap(),
    ];
    assert!(SkinProject::from_definition(input).is_err());
}

#[test]
fn project_rejects_duplicate_documents_and_project_wide_layer_ids() {
    let mut input = definition();
    input.documents.push(document(1, &[], vec![]));
    assert_eq!(
        SkinProject::from_definition(input),
        Err(ModelError::DuplicateId(ObjectId::Document(did(1))))
    );
    let mut input = definition();
    input
        .documents
        .push(document(2, &[1], vec![image(1, 1, Affine2::IDENTITY)]));
    assert_eq!(
        SkinProject::from_definition(input),
        Err(ModelError::DuplicateId(ObjectId::Layer(lid(1))))
    );
}

#[test]
fn changing_a_detached_definition_never_changes_the_validated_value() {
    let mut input = definition();
    let project = SkinProject::from_definition(input.clone()).unwrap();
    input.assets.clear();
    input.documents.clear();
    assert_eq!(project.assets().iter().len(), 1);
    assert_eq!(project.documents().len(), 1);
}

#[test]
fn nested_component_transform_affects_two_distinct_document_outputs() {
    let mut input = definition();
    input
        .documents
        .push(document(2, &[2], vec![image(2, 1, Affine2::IDENTITY)]));
    input.components = ComponentTree::new(
        vec![cid(1)],
        vec![
            cgroup(1, &[2], translation(20.0, 12.0)),
            cgroup(2, &[3, 4], scale(2.0, 3.0)),
            member(3, 1, translation(-5.0, -10.0)),
            member(4, 2, Affine2::IDENTITY),
        ],
    )
    .unwrap();
    input.outputs = vec![output(1, 1, "circle.png"), output(2, 2, "overlay.png")];
    let project = SkinProject::from_definition(input).unwrap();
    close(
        project
            .document_content_transform(did(1))
            .unwrap()
            .point(2.0, 4.0)
            .unwrap(),
        [19.0, 4.0],
    );
    close(
        project
            .document_content_transform(did(2))
            .unwrap()
            .point(2.0, 4.0)
            .unwrap(),
        [24.0, 24.0],
    );
    let bounds = project.component_bounds(cid(1)).unwrap().unwrap();
    close(bounds.min(), [10.0, -18.0]);
    close(bounds.max(), [40.0, 72.0]);
    assert_eq!(project.outputs().len(), 2);
    assert_ne!(
        project.outputs()[0].document(),
        project.outputs()[1].document()
    );
    assert_eq!(project.components().parent(cid(3)), Some(Some(cid(2))));
    assert_eq!(project.components().member(did(1)), Some(cid(3)));
}

#[test]
fn component_membership_is_unique_even_in_separate_roots() {
    assert_eq!(
        ComponentTree::new(
            vec![cid(1), cid(2)],
            vec![
                member(1, 1, Affine2::IDENTITY),
                member(2, 1, Affine2::IDENTITY)
            ]
        ),
        Err(ModelError::DuplicateMembership(did(1)))
    );
}

#[test]
fn components_require_invertible_member_maps_but_allow_zero_scale_edits() {
    assert_eq!(
        ComponentTree::new(vec![cid(1)], vec![member(1, 1, scale(0.0, 1.0))]),
        Err(ModelError::NotInvertible)
    );
    let tree = ComponentTree::new(
        vec![cid(1)],
        vec![
            cgroup(1, &[2], scale(0.0, 1.0)),
            member(2, 1, Affine2::IDENTITY),
        ],
    )
    .unwrap();
    assert_eq!(
        tree.document_content_transform(did(1)),
        Some(scale(0.0, 1.0))
    );
}

#[test]
fn component_forests_reject_cycles_missing_children_and_multiple_parents() {
    assert_eq!(
        ComponentTree::new(
            vec![],
            vec![
                cgroup(1, &[2], Affine2::IDENTITY),
                cgroup(2, &[1], Affine2::IDENTITY)
            ]
        ),
        Err(ModelError::Cycle)
    );
    assert!(matches!(
        ComponentTree::new(vec![cid(1)], vec![cgroup(1, &[2], Affine2::IDENTITY)]),
        Err(ModelError::MissingReference(_))
    ));
    assert!(matches!(
        ComponentTree::new(
            vec![cid(1), cid(2)],
            vec![
                cgroup(1, &[3], Affine2::IDENTITY),
                cgroup(2, &[3], Affine2::IDENTITY),
                cgroup(3, &[], Affine2::IDENTITY)
            ]
        ),
        Err(ModelError::MultipleParents(_))
    ));
    assert!(matches!(
        ComponentTree::new(vec![], vec![cgroup(1, &[], Affine2::IDENTITY)]),
        Err(ModelError::Unreachable(_))
    ));
    assert!(matches!(
        ComponentTree::new(
            vec![cid(1)],
            vec![
                cgroup(1, &[], Affine2::IDENTITY),
                cgroup(1, &[], Affine2::IDENTITY)
            ]
        ),
        Err(ModelError::DuplicateId(_))
    ));
}

#[test]
fn component_member_references_must_exist_in_project() {
    let mut input = definition();
    input.components =
        ComponentTree::new(vec![cid(1)], vec![member(1, 99, Affine2::IDENTITY)]).unwrap();
    assert_eq!(
        SkinProject::from_definition(input),
        Err(ModelError::MissingReference(ObjectId::Document(did(99))))
    );
}

#[test]
fn ungrouped_documents_have_identity_component_transform() {
    let project = SkinProject::from_definition(definition()).unwrap();
    assert_eq!(
        project.document_content_transform(did(1)).unwrap(),
        Affine2::IDENTITY
    );
    assert!(project.document_content_transform(did(99)).is_err());
}

#[test]
fn deep_components_validate_and_drop_without_recursive_nodes() {
    let nodes = (1..=10_000u128)
        .map(|id| {
            if id == 10_000 {
                member(id, 1, Affine2::IDENTITY)
            } else {
                cgroup(id, &[id + 1], translation(1.0, 0.0))
            }
        })
        .collect();
    let tree = ComponentTree::new(vec![cid(1)], nodes).unwrap();
    assert_eq!(tree.iter().len(), 10_000);
    close(
        tree.document_content_transform(did(1))
            .unwrap()
            .point(0.0, 0.0)
            .unwrap(),
        [9_999.0, 0.0],
    );
    assert_eq!(tree.clone(), tree);
}

#[test]
fn composed_document_and_component_extents_cannot_overflow() {
    let mut input = definition();
    input.documents = vec![document(
        1,
        &[1],
        vec![image(1, 1, translation(1e200, 0.0))],
    )];
    input.components = ComponentTree::new(
        vec![cid(1)],
        vec![
            cgroup(1, &[2], scale(1e200, 1.0)),
            member(2, 1, Affine2::IDENTITY),
        ],
    )
    .unwrap();
    assert_eq!(
        SkinProject::from_definition(input),
        Err(ModelError::NumericOverflow)
    );
}

#[test]
fn configuration_preserves_unknown_bytes_duplicates_and_line_endings() {
    let raw = |b: &[u8]| ConfigRecord::new(b.to_vec(), ConfigInterpretation::Opaque).unwrap();
    let section = ConfigRecord::new(
        b"[Future]\r\n".to_vec(),
        ConfigInterpretation::Section { name: 1..7 },
    )
    .unwrap();
    let entry = ConfigRecord::new(
        b"Key : 1\n".to_vec(),
        ConfigInterpretation::Entry {
            key: 0..3,
            value: 6..7,
        },
    )
    .unwrap();
    let config = SkinConfiguration::new(vec![
        raw(b"\xef\xbb\xbf; comment\r\n"),
        section,
        entry.clone(),
        entry,
        raw(b"\xff???\rLast:"),
    ]);
    assert_eq!(
        config.to_bytes(),
        b"\xef\xbb\xbf; comment\r\n[Future]\r\nKey : 1\nKey : 1\n\xff???\rLast:"
    );
    assert_eq!(config.records().len(), 5);
    assert_eq!(config.records()[2].raw(), b"Key : 1\n");
}

#[test]
fn configuration_ranges_reject_out_of_bounds_empty_keys_and_overlap() {
    for interpretation in [
        ConfigInterpretation::Section { name: 0..99 },
        ConfigInterpretation::Section { name: 1..1 },
        ConfigInterpretation::Entry {
            key: 0..2,
            value: 1..3,
        },
        ConfigInterpretation::Entry {
            key: 1..1,
            value: 2..3,
        },
        ConfigInterpretation::Entry {
            key: 0..1,
            value: 8..9,
        },
    ] {
        assert!(ConfigRecord::new(b"A:1".to_vec(), interpretation).is_err());
    }
    assert!(
        ConfigRecord::new(
            b"A:".to_vec(),
            ConfigInterpretation::Entry {
                key: 0..1,
                value: 2..2
            }
        )
        .is_ok()
    );
}

#[test]
fn output_paths_are_relative_and_do_not_guess_element_from_filename() {
    for path in [
        "", "/root", "../x", "a/../b", "a/./b", "a//b", "C:/x", "a\\b", "a/", "a.", "a ", "a\n",
        "a?", "a*",
    ] {
        assert!(OutputPath::new(path.into()).is_err(), "{path}");
    }
    let mapping = output(1, 1, "自定义/anything.png");
    assert_eq!(mapping.element_key(), "explicit-element");
    assert_eq!(mapping.path().as_str(), "自定义/anything.png");
}

#[test]
fn output_conflicts_include_case_and_file_directory_prefixes() {
    for names in [
        vec!["a.png", "A.PNG"],
        vec!["a", "a-b", "a/b.png"],
        vec!["Dir/x", "dir/X"],
    ] {
        let mut input = definition();
        input.outputs = names
            .iter()
            .enumerate()
            .map(|(i, name)| output(i as u128 + 1, 1, name))
            .collect();
        assert!(matches!(
            SkinProject::from_definition(input),
            Err(ModelError::OutputConflict(_))
        ));
    }
}

#[test]
fn output_identity_and_document_references_are_checked() {
    let mut input = definition();
    input.outputs = vec![output(1, 99, "a.png")];
    assert!(matches!(
        SkinProject::from_definition(input),
        Err(ModelError::MissingReference(_))
    ));
    let mut input = definition();
    input.outputs = vec![output(1, 1, "a.png"), output(1, 1, "b.png")];
    assert!(matches!(
        SkinProject::from_definition(input),
        Err(ModelError::DuplicateId(_))
    ));
    let mut input = definition();
    input.outputs = vec![output(1, 1, "a.png"), output(2, 1, "a@2x.png")];
    assert_eq!(
        SkinProject::from_definition(input).unwrap().outputs().len(),
        2
    );
}

#[test]
fn unsupported_model_version_is_explicit_and_empty_root_is_valid() {
    let mut input = definition();
    input.model_version = MODEL_VERSION + 1;
    assert_eq!(
        SkinProject::from_definition(input),
        Err(ModelError::UnsupportedVersion(MODEL_VERSION + 1))
    );
    let empty = SkinProject::empty(ProjectId::new(7).unwrap());
    assert_eq!(empty.model_version(), MODEL_VERSION);
    assert_eq!(empty.documents().len(), 0);
    assert_eq!(empty.assets().iter().len(), 0);
    assert!(empty.outputs().is_empty());
}
