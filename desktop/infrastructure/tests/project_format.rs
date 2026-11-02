use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use skin_editor_application::{ProjectSession, values::*};
use skin_editor_infrastructure::project_format::*;
use std::collections::{BTreeMap, BTreeSet};

fn a(n: u128) -> AssetId {
    AssetId::new(u128::MAX - n).unwrap()
}
fn d(n: u128) -> DocumentId {
    DocumentId::new(n).unwrap()
}
fn l(n: u128) -> LayerId {
    LayerId::new(n).unwrap()
}
fn c(n: u128) -> ComponentId {
    ComponentId::new(n).unwrap()
}
fn s(n: u128) -> AudioComponentId {
    AudioComponentId::new(n).unwrap()
}
fn o(n: u128) -> OutputId {
    OutputId::new(n).unwrap()
}
fn affine(v: [f64; 6]) -> Affine2 {
    Affine2::new(v).unwrap()
}
fn config(raw: &[u8], interpretation: ConfigInterpretation) -> ConfigRecord {
    ConfigRecord::new(raw.to_vec(), interpretation).unwrap()
}
fn project() -> SkinProject {
    let mut input = ProjectDefinition::empty(ProjectId::new(u128::MAX).unwrap());
    let size = PixelSize::new(8, 9).unwrap();
    input.assets = vec![
        Asset::new(
            a(1),
            "原图 🙂".into(),
            BlobRef::new([1; 32], 64),
            AssetKind::RasterImage(size),
        )
        .unwrap(),
        Asset::new(
            a(2),
            "same bytes".into(),
            BlobRef::new([1; 32], 64),
            AssetKind::RasterImage(size),
        )
        .unwrap(),
        Asset::new(
            a(3),
            "sound".into(),
            BlobRef::new([2; 32], u64::MAX),
            AssetKind::Audio(AudioMetadata::new(48000, 2, u64::MAX).unwrap()),
        )
        .unwrap(),
        Asset::new(
            a(4),
            "unknown codec".into(),
            BlobRef::new([3; 32], 13),
            AssetKind::Opaque,
        )
        .unwrap(),
        Asset::new(
            a(5),
            "unused original".into(),
            BlobRef::new([4; 32], 7),
            AssetKind::Opaque,
        )
        .unwrap(),
    ];
    input.documents = vec![
        ElementDocument::new(
            d(1),
            size,
            LayerTree::new(
                vec![l(1)],
                vec![
                    LayerNode {
                        id: l(1),
                        local: affine([1., 0.125, -0.25, 1., 2., 3.]),
                        opacity: Opacity::new(0.5).unwrap(),
                        visible: false,
                        kind: LayerKind::Group {
                            children: vec![l(2)],
                        },
                    },
                    LayerNode {
                        id: l(2),
                        local: Affine2::IDENTITY,
                        opacity: Opacity::OPAQUE,
                        visible: true,
                        kind: LayerKind::Group {
                            children: vec![l(4), l(3)],
                        },
                    },
                    LayerNode {
                        id: l(3),
                        local: affine([1., 0., 0., 1., -0.0, 1.2345678901234567]),
                        opacity: Opacity::new(0.2).unwrap(),
                        visible: true,
                        kind: LayerKind::Image { asset: a(1) },
                    },
                    LayerNode {
                        id: l(4),
                        local: Affine2::IDENTITY,
                        opacity: Opacity::OPAQUE,
                        visible: false,
                        kind: LayerKind::Image { asset: a(2) },
                    },
                ],
            )
            .unwrap(),
        )
        .with_visibility(false),
        ElementDocument::new(
            d(2),
            size,
            LayerTree::new(
                vec![l(5)],
                vec![LayerNode {
                    id: l(5),
                    local: Affine2::IDENTITY,
                    opacity: Opacity::OPAQUE,
                    visible: true,
                    kind: LayerKind::Image { asset: a(1) },
                }],
            )
            .unwrap(),
        ),
    ];
    input.components = ComponentTree::new(
        vec![c(1)],
        vec![
            ComponentNode {
                id: c(1),
                local: affine([0., 1., -1., 0., 8., 10.]),
                kind: ComponentKind::Group {
                    children: vec![c(2), c(3)],
                },
            },
            ComponentNode {
                id: c(2),
                local: Affine2::IDENTITY,
                kind: ComponentKind::Element {
                    document: d(1),
                    document_to_component: affine([2., 0., 0., 0.5, 6., 9.]),
                },
            },
            ComponentNode {
                id: c(3),
                local: Affine2::IDENTITY,
                kind: ComponentKind::Group {
                    children: vec![c(4)],
                },
            },
            ComponentNode {
                id: c(4),
                local: Affine2::IDENTITY,
                kind: ComponentKind::Element {
                    document: d(2),
                    document_to_component: Affine2::IDENTITY,
                },
            },
        ],
    )
    .unwrap();
    input.audio_components = vec![
        AudioComponent::new(s(1), SoundSlotKey::new("editor:one".into()).unwrap(), a(3))
            .with_properties(Gain::new(0.1234567890123456).unwrap(), false),
        AudioComponent::new(s(2), SoundSlotKey::new("editor:two".into()).unwrap(), a(3)),
        AudioComponent::new(
            s(3),
            SoundSlotKey::new("editor:opaque".into()).unwrap(),
            a(4),
        ),
    ];
    input.configuration = SkinConfiguration::new(vec![
        config(b"#\xff\0\r\n", ConfigInterpretation::Opaque),
        config(
            b"[General]\r\n",
            ConfigInterpretation::Section { name: 1..8 },
        ),
        config(
            b"Key: \xff\0\n",
            ConfigInterpretation::Entry {
                key: 0..3,
                value: 5..7,
            },
        ),
        config(
            b"Key: \xff\0\n",
            ConfigInterpretation::Entry {
                key: 0..3,
                value: 5..7,
            },
        ),
        config(b"", ConfigInterpretation::Opaque),
    ]);
    input.outputs = vec![
        OutputMapping::new(
            o(1),
            d(1),
            "unverified-image-slot".into(),
            OutputPath::new("image.png".into()).unwrap(),
        )
        .unwrap(),
        OutputMapping::for_audio(
            o(2),
            s(3),
            "unverified-audio-slot".into(),
            OutputPath::new("unknown-audio.bin".into()).unwrap(),
        )
        .unwrap(),
    ];
    SkinProject::from_definition(input).unwrap()
}
fn definition(project: &SkinProject) -> ProjectDefinition {
    ProjectDefinition {
        model_version: project.model_version(),
        id: project.id(),
        assets: project.assets().iter().cloned().collect(),
        documents: project.documents().cloned().collect(),
        audio_components: project.audio_components().cloned().collect(),
        components: project.components().clone(),
        configuration: project.configuration().clone(),
        outputs: project.outputs().to_vec(),
    }
}
fn bundle(project: &SkinProject) -> StateBundle {
    encode_project(project, FormatLimits::default()).unwrap()
}
fn digest(bytes: &[u8]) -> StateHash {
    StateHash::from_bytes(Sha256::digest(bytes).into())
}
fn kind(bundle: &StateBundle, name: &str) -> StateHash {
    bundle
        .blocks
        .iter()
        .find(|b| serde_json::from_slice::<Value>(&b.bytes).unwrap()["kind"] == name)
        .unwrap()
        .hash
}
fn replace_references(value: &mut Value, changes: &BTreeMap<String, String>) {
    match value {
        Value::String(s) => {
            if let Some(new) = changes.get(s) {
                *s = new.clone();
            }
        }
        Value::Array(v) => {
            for x in v {
                replace_references(x, changes)
            }
        }
        Value::Object(v) => {
            for x in v.values_mut() {
                replace_references(x, changes)
            }
        }
        _ => {}
    }
}
// Rebuild a tampered detached fixture's ancestor hashes, so failures test
// semantic validation rather than only detecting an intentionally stale SHA.
fn rewrite(bundle: &StateBundle, target: StateHash, change: impl Fn(&mut Value)) -> StateBundle {
    let mut result = bundle.clone();
    let mut changes = BTreeMap::new();
    for block in &mut result.blocks {
        let old = block.hash;
        let mut value: Value = serde_json::from_slice(&block.bytes).unwrap();
        if old == target {
            change(&mut value);
        }
        replace_references(&mut value, &changes);
        let refs = value["blocks"].as_array_mut().unwrap();
        refs.sort_by_key(|x| x.as_str().unwrap().to_owned());
        refs.dedup();
        block.bytes = serde_json::to_vec(&value).unwrap();
        block.hash = digest(&block.bytes);
        changes.insert(old.hex(), block.hash.hex());
    }
    result.root = StateHash::from_bytes(
        result
            .blocks
            .iter()
            .find(|b| b.hash.hex() == changes[&bundle.root.hex()])
            .unwrap()
            .hash
            .bytes(),
    );
    result
}

#[test]
fn all_current_model_content_roundtrips_without_normalizing_configuration_or_media() {
    let p = project();
    let encoded = bundle(&p);
    let decoded = decode_project(&encoded, FormatLimits::default()).unwrap();
    assert_eq!(decoded, p);
    assert_eq!(
        decoded.configuration().to_bytes(),
        p.configuration().to_bytes()
    );
    assert_eq!(
        decoded
            .document(d(1))
            .unwrap()
            .layers()
            .node(l(2))
            .unwrap()
            .children(),
        &[l(4), l(3)]
    );
    assert_eq!(
        decoded
            .document(d(1))
            .unwrap()
            .layers()
            .node(l(3))
            .unwrap()
            .local
            .coefficients()[4]
            .to_bits(),
        (-0.0_f64).to_bits()
    );
    assert_eq!(required_originals(&decoded).len(), 4);
    assert!(required_originals(&decoded).contains(&BlobRef::new([4; 32], 7)));
    let metadata = String::from_utf8(
        encoded
            .blocks
            .iter()
            .flat_map(|b| b.bytes.clone())
            .collect(),
    )
    .unwrap();
    assert!(metadata.contains(&format!("{:032x}", u128::MAX)));
    assert!(metadata.contains(&format!("\"{}\"", u64::MAX)));
    assert!(!metadata.contains("Arc"));
}

#[test]
fn one_layer_edit_reuses_unrelated_blocks_and_keeps_old_bundle_unchanged() {
    let p = project();
    let old = bundle(&p);
    let original = old.clone();
    let mut input = definition(&p);
    let doc = &input.documents[0];
    let mut layers: Vec<_> = doc.layers().iter().cloned().collect();
    layers.iter_mut().find(|n| n.id == l(3)).unwrap().local = affine([1., 0., 0., 1., 55., 88.]);
    input.documents[0] = ElementDocument::new(
        doc.id(),
        doc.canvas(),
        LayerTree::new(doc.layers().roots().to_vec(), layers).unwrap(),
    )
    .with_visibility(doc.visible());
    let p2 = SkinProject::from_definition(input).unwrap();
    let next = bundle(&p2);
    let existing: BTreeSet<_> = old.blocks.iter().map(|b| b.hash).collect();
    let added = next
        .blocks
        .iter()
        .filter(|b| !existing.contains(&b.hash))
        .count();
    assert!(added > 0 && added < next.blocks.len() / 2, "{added}");
    assert_eq!(
        kind(&old, "skin-editor.asset"),
        kind(&next, "skin-editor.asset")
    );
    assert_eq!(
        kind(&old, "skin-editor.sound"),
        kind(&next, "skin-editor.sound")
    );
    assert_eq!(old, original);
    assert_eq!(decode_project(&old, FormatLimits::default()).unwrap(), p);
    assert_eq!(decode_project(&next, FormatLimits::default()).unwrap(), p2);
}

#[test]
fn missing_corrupt_duplicate_and_unreachable_blocks_are_rejected() {
    let b = bundle(&project());
    let mut missing = b.clone();
    missing.blocks.remove(0);
    assert_eq!(
        decode_project(&missing, FormatLimits::default()),
        Err(CodecError::MissingBlock)
    );
    let mut bad = b.clone();
    bad.blocks[0].bytes.push(b' ');
    assert_eq!(
        decode_project(&bad, FormatLimits::default()),
        Err(CodecError::DigestMismatch)
    );
    let mut duplicate = b.clone();
    duplicate.blocks.push(duplicate.blocks[0].clone());
    assert_eq!(
        decode_project(&duplicate, FormatLimits::default()),
        Err(CodecError::DuplicateBlock)
    );
    let mut extra = b.clone();
    let bytes = br#"{"unused":true}"#.to_vec();
    extra.blocks.push(StateBlock {
        hash: digest(&bytes),
        bytes,
    });
    assert_eq!(
        decode_project(&extra, FormatLimits::default()),
        Err(CodecError::UnusedBlock)
    );
}

#[test]
fn unknown_schema_semantics_and_fields_never_produce_a_writable_candidate() {
    let b = bundle(&project());
    for changed in [
        rewrite(&b, b.root, |v| v["schema"] = json!(2)),
        rewrite(&b, b.root, |v| v["value"]["semantics"] = json!("future")),
    ] {
        assert_eq!(
            decode_project(&changed, FormatLimits::default()),
            Err(CodecError::UnsupportedSchema)
        );
    }
    let changed = rewrite(&b, kind(&b, "skin-editor.layer"), |v| {
        v["value"]["future-blur"] = json!(10)
    });
    assert_eq!(
        decode_project(&changed, FormatLimits::default()),
        Err(CodecError::UnsupportedField)
    );
    let changed = rewrite(&b, kind(&b, "skin-editor.sound"), |v| {
        v["kind"] = json!("future-sound")
    });
    assert_eq!(
        decode_project(&changed, FormatLimits::default()),
        Err(CodecError::UnsupportedKind)
    );
}

#[test]
fn reference_manifests_and_model_invariants_are_both_validated() {
    let b = bundle(&project());
    let changed = rewrite(&b, kind(&b, "skin-editor.asset"), |v| {
        v["blobs"] = json!([])
    });
    assert_eq!(
        decode_project(&changed, FormatLimits::default()),
        Err(CodecError::ReferenceMismatch)
    );
    let changed = rewrite(&b, kind(&b, "skin-editor.sound"), |v| {
        v["value"]["asset"] = json!(format!("{:032x}", 1234))
    });
    assert!(matches!(
        decode_project(&changed, FormatLimits::default()),
        Err(CodecError::Model(ModelError::MissingReference(_)))
    ));
    let changed = rewrite(&b, b.root, |v| {
        v["value"]["id"] = json!("00000000000000000000000000000000")
    });
    assert_eq!(
        decode_project(&changed, FormatLimits::default()),
        Err(CodecError::InvalidField("identity"))
    );
}

#[test]
fn json_duplicates_integer_loss_invalid_utf8_and_trailing_values_are_rejected() {
    let b = bundle(&project());
    let original = b.blocks.iter().find(|v| v.hash == b.root).unwrap();
    for bytes in [
        [b"{\"schema\":1,".as_slice(), &original.bytes[1..]].concat(),
        [b"{\"sche\\u006da\":1,".as_slice(), &original.bytes[1..]].concat(),
        original
            .bytes
            .iter()
            .copied()
            .chain(b" {}".iter().copied())
            .collect(),
        br#"{"schema":9007199254740993}"#.to_vec(),
        br#"{"schema":9007199254740992}"#.to_vec(),
        br#"{"schema":-9007199254740992}"#.to_vec(),
        br#"{"schema":9999999999999999999999999999999999999999999}"#.to_vec(),
        vec![b'{', b'"', 0xff, b'"', b':', b'0', b'}'],
    ] {
        let mut bad = b.clone();
        let hash = digest(&bytes);
        bad.root = hash;
        *bad.blocks
            .iter_mut()
            .find(|v| v.hash == original.hash)
            .unwrap() = StateBlock { hash, bytes };
        assert_eq!(
            decode_project(&bad, FormatLimits::default()),
            Err(CodecError::InvalidJson)
        );
    }
}

#[test]
fn decoder_accepts_different_whitespace_and_block_order_without_rewriting_input() {
    let mut b = bundle(&project());
    let original = b.clone();
    let block = b
        .blocks
        .iter_mut()
        .find(|b| b.hash == original.root)
        .unwrap();
    block.bytes = [b" \n".as_slice(), &block.bytes, b"\n"].concat();
    block.hash = digest(&block.bytes);
    b.root = block.hash;
    b.blocks.reverse();
    let unchanged = b.clone();
    assert_eq!(
        decode_project(&b, FormatLimits::default()).unwrap(),
        project()
    );
    assert_eq!(b, unchanged);
}

#[test]
fn budgets_fail_without_a_partial_result_or_source_mutation() {
    let p = project();
    let b = bundle(&p);
    let before = b.clone();
    for limits in [
        FormatLimits {
            blocks: 1,
            ..FormatLimits::default()
        },
        FormatLimits {
            total_bytes: 100,
            ..FormatLimits::default()
        },
        FormatLimits {
            items: 1,
            ..FormatLimits::default()
        },
        FormatLimits {
            block_bytes: 100,
            ..FormatLimits::default()
        },
    ] {
        assert!(matches!(
            encode_project(&p, limits),
            Err(CodecError::ResourceLimit(_))
        ));
        assert!(matches!(
            decode_project(&b, limits),
            Err(CodecError::ResourceLimit(_))
        ));
    }
    assert_eq!(
        encode_project(
            &p,
            FormatLimits {
                json_depth: 0,
                ..FormatLimits::default()
            }
        ),
        Err(CodecError::InvalidLimits)
    );
    assert_eq!(b, before);
    let shallow = FormatLimits {
        json_depth: 1,
        ..FormatLimits::default()
    };
    assert!(matches!(
        encode_project(&p, shallow),
        Err(CodecError::ResourceLimit("json_structure"))
    ));
    assert_eq!(
        decode_project(&b, shallow),
        Err(CodecError::ResourceLimit("json_structure"))
    );
}

#[test]
fn independent_golden_state_decodes_without_rust_memory_layout_or_numeric_ids() {
    let fixture: Value =
        serde_json::from_str(include_str!("fixtures/project-state-v1-empty.json")).unwrap();
    fn sha(text: &str) -> StateHash {
        let bytes = (0..32)
            .map(|i| u8::from_str_radix(&text[i * 2..i * 2 + 2], 16).unwrap())
            .collect::<Vec<_>>();
        StateHash::from_bytes(bytes.try_into().unwrap())
    }
    let b = StateBundle {
        root: sha(fixture["root"].as_str().unwrap()),
        blocks: fixture["blocks"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| StateBlock {
                hash: sha(v["sha256"].as_str().unwrap()),
                bytes: v["raw_utf8"].as_str().unwrap().as_bytes().to_vec(),
            })
            .collect(),
    };
    assert_eq!(
        decode_project(&b, FormatLimits::default()).unwrap(),
        SkinProject::empty(ProjectId::new(42).unwrap())
    );
}

#[test]
fn affine_and_gain_bits_roundtrip_without_silent_quantization() {
    let p = project();
    for value in [
        -0.0,
        f64::MIN_POSITIVE,
        1e-300,
        -1e-100,
        1.0000000000000002,
        -123.45678901234567,
        1e100,
    ] {
        let mut input = definition(&p);
        let doc = &input.documents[0];
        let mut nodes: Vec<_> = doc.layers().iter().cloned().collect();
        nodes.iter_mut().find(|n| n.id == l(3)).unwrap().local =
            affine([1., 0., 0., 1., value, 2.]);
        input.documents[0] = ElementDocument::new(
            doc.id(),
            doc.canvas(),
            LayerTree::new(doc.layers().roots().to_vec(), nodes).unwrap(),
        )
        .with_visibility(doc.visible());
        let project = SkinProject::from_definition(input).unwrap();
        let decoded = decode_project(&bundle(&project), FormatLimits::default()).unwrap();
        assert_eq!(
            decoded
                .document(d(1))
                .unwrap()
                .layers()
                .node(l(3))
                .unwrap()
                .local
                .coefficients()[4]
                .to_bits(),
            value.to_bits()
        );
    }
    let b = bundle(&p);
    let changed = rewrite(&b, kind(&b, "skin-editor.layer"), |v| {
        v["value"]["opacity"] = json!(0.1_f64)
    });
    assert_eq!(
        decode_project(&changed, FormatLimits::default()),
        Err(CodecError::InvalidField("opacity_precision"))
    );
}

#[test]
fn invalid_configuration_ranges_and_oversized_sequence_counts_are_rejected() {
    let b = bundle(&project());
    let changed = rewrite(&b, kind(&b, "skin-editor.config-record"), |v| {
        v["value"]["interpretation"] = json!({"tag":"section","name":["0","999999"]})
    });
    assert!(matches!(
        decode_project(&changed, FormatLimits::default()),
        Err(CodecError::Model(ModelError::InvalidValue(_)))
    ));
    let changed = rewrite(&b, kind(&b, "skin-editor.sequence"), |v| {
        v["value"]["len"] = json!(u64::MAX.to_string())
    });
    assert_eq!(
        decode_project(&changed, FormatLimits::default()),
        Err(CodecError::ResourceLimit("items"))
    );
}

#[test]
fn paged_flat_trees_preserve_thousands_of_roots_and_deep_groups() {
    let mut input = ProjectDefinition::empty(ProjectId::new(77).unwrap());
    let size = PixelSize::new(2, 2).unwrap();
    input.assets.push(
        Asset::new(
            a(1),
            "source".into(),
            BlobRef::new([9; 32], 1),
            AssetKind::RasterImage(size),
        )
        .unwrap(),
    );
    let mut nodes = Vec::new();
    for n in 1..=3000 {
        nodes.push(LayerNode {
            id: l(n),
            local: Affine2::IDENTITY,
            opacity: Opacity::OPAQUE,
            visible: true,
            kind: LayerKind::Image { asset: a(1) },
        });
    }
    input.documents.push(ElementDocument::new(
        d(1),
        size,
        LayerTree::new((1..=3000).rev().map(l).collect(), nodes).unwrap(),
    ));
    let p = SkinProject::from_definition(input).unwrap();
    let b = bundle(&p);
    assert!(b.blocks.len() > 3000);
    assert_eq!(decode_project(&b, FormatLimits::default()).unwrap(), p);
    let mut input = definition(&p);
    let mut nodes = Vec::new();
    for n in 1..=2000 {
        nodes.push(LayerNode {
            id: l(n),
            local: Affine2::IDENTITY,
            opacity: Opacity::OPAQUE,
            visible: true,
            kind: if n == 2000 {
                LayerKind::Image { asset: a(1) }
            } else {
                LayerKind::Group {
                    children: vec![l(n + 1)],
                }
            },
        });
    }
    input.documents[0] =
        ElementDocument::new(d(1), size, LayerTree::new(vec![l(1)], nodes).unwrap());
    let p = SkinProject::from_definition(input).unwrap();
    assert_eq!(
        decode_project(&bundle(&p), FormatLimits::default()).unwrap(),
        p
    );
}

#[test]
fn detached_decode_cannot_change_active_session_or_history() {
    let session = ProjectSession::new(7, 70).unwrap();
    let before = session.project_info();
    let result = decode_project(&bundle(&project()), FormatLimits::default()).unwrap();
    assert_ne!(result.id().get(), 7);
    assert_eq!(session.project_info(), before);
}

#[test]
fn shared_records_cannot_expand_past_the_decode_budget() {
    let mut input = ProjectDefinition::empty(ProjectId::new(1).unwrap());
    let record = ConfigRecord::new(vec![0xaa; 20000], ConfigInterpretation::Opaque).unwrap();
    input.configuration = SkinConfiguration::new(vec![record; 32]);
    let project = SkinProject::from_definition(input).unwrap();
    let b = bundle(&project);
    assert!(b.blocks.iter().map(|b| b.bytes.len()).sum::<usize>() < 100000);
    assert_eq!(
        decode_project(&b, FormatLimits::default()).unwrap(),
        project
    );
    let small = FormatLimits {
        decoded_bytes: 100000,
        ..FormatLimits::default()
    };
    assert_eq!(
        decode_project(&b, small),
        Err(CodecError::ResourceLimit("decoded_bytes"))
    );
    assert_eq!(
        encode_project(&project, small),
        Err(CodecError::ResourceLimit("decoded_bytes"))
    );
}
