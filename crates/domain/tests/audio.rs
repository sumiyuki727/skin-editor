use skin_editor_domain::*;

fn asset(id: u128, kind: AssetKind) -> Asset {
    Asset::new(
        AssetId::new(id).unwrap(),
        "source".into(),
        BlobRef::new([id as u8; 32], 8),
        kind,
    )
    .unwrap()
}
fn sound(id: u128, slot: &str, asset: u128) -> AudioComponent {
    AudioComponent::new(
        AudioComponentId::new(id).unwrap(),
        SoundSlotKey::new(slot.into()).unwrap(),
        AssetId::new(asset).unwrap(),
    )
}
fn input() -> ProjectDefinition {
    let mut input = ProjectDefinition::empty(ProjectId::new(1).unwrap());
    input.assets.push(asset(1, AssetKind::Opaque));
    input.audio_components = vec![sound(1, "fixture.hit", 1), sound(2, "fixture.result", 1)];
    input
}
#[test]
fn sound_components_share_a_source_without_visual_membership_or_shared_properties() {
    let mut definition = input();
    definition.audio_components[0] = definition.audio_components[0]
        .clone()
        .with_properties(Gain::new(0.5).unwrap(), false);
    let project = SkinProject::from_definition(definition).unwrap();
    assert_eq!(project.documents().len(), 0);
    assert_eq!(project.components().iter().count(), 0);
    let a = project
        .audio_component(AudioComponentId::new(1).unwrap())
        .unwrap();
    let b = project
        .audio_component(AudioComponentId::new(2).unwrap())
        .unwrap();
    assert_eq!(a.asset(), b.asset());
    assert!(!a.visible());
    assert_eq!(b.gain(), Gain::UNITY);
    assert!(b.visible());
}
#[test]
fn dangling_image_duplicate_slot_and_duplicate_identity_are_rejected() {
    let mut missing = input();
    missing.assets.clear();
    assert!(matches!(
        SkinProject::from_definition(missing),
        Err(ModelError::MissingReference(ObjectId::Asset(_)))
    ));
    let mut image = input();
    image.assets[0] = asset(1, AssetKind::RasterImage(PixelSize::new(1, 1).unwrap()));
    assert_eq!(
        SkinProject::from_definition(image),
        Err(ModelError::InvalidValue("audio_component_asset_kind"))
    );
    let mut duplicate = input();
    duplicate.audio_components[1] = sound(2, "fixture.hit", 1);
    assert_eq!(
        SkinProject::from_definition(duplicate),
        Err(ModelError::InvalidValue("duplicate_sound_slot"))
    );
    let mut duplicate = input();
    duplicate.audio_components[1] = sound(1, "fixture.other", 1);
    assert!(matches!(
        SkinProject::from_definition(duplicate),
        Err(ModelError::DuplicateId(ObjectId::AudioComponent(_)))
    ));
}
#[test]
fn numeric_metadata_and_slot_constraints_do_not_depend_on_a_decoder() {
    for gain in [f64::NAN, f64::INFINITY, -0.1, 1.1] {
        assert!(Gain::new(gain).is_err());
    }
    assert_eq!(Gain::new(-0.0).unwrap(), Gain::new(0.0).unwrap());
    for (rate, channels, frames) in [(0, 1, 1), (48000, 0, 1), (48000, 2, 0)] {
        assert!(AudioMetadata::new(rate, channels, frames).is_err());
    }
    for key in [
        "".to_owned(),
        " ".into(),
        "bad\nslot".into(),
        "x".repeat(129),
    ] {
        assert!(SoundSlotKey::new(key).is_err());
    }
}
#[test]
fn blob_aliases_cannot_disagree_about_media_metadata() {
    let info = AudioMetadata::new(48000, 2, 4).unwrap();
    let original = asset(1, AssetKind::Audio(info));
    let alias = |kind| {
        Asset::new(
            AssetId::new(2).unwrap(),
            "alias".into(),
            original.blob(),
            kind,
        )
        .unwrap()
    };
    assert!(AssetLibrary::new(vec![original.clone(), alias(AssetKind::Opaque)]).is_ok());
    for kind in [
        AssetKind::Audio(AudioMetadata::new(44100, 2, 4).unwrap()),
        AssetKind::RasterImage(PixelSize::new(1, 1).unwrap()),
    ] {
        assert!(AssetLibrary::new(vec![original.clone(), alias(kind.clone())]).is_err());
        assert!(AssetLibrary::new(vec![alias(kind), original.clone()]).is_err());
    }
}
#[test]
fn image_and_audio_outputs_share_one_collision_namespace() {
    let mut definition = input();
    let doc = DocumentId::new(1).unwrap();
    definition.documents.push(ElementDocument::new(
        doc,
        PixelSize::new(1, 1).unwrap(),
        LayerTree::default(),
    ));
    definition.outputs.push(
        OutputMapping::new(
            OutputId::new(1).unwrap(),
            doc,
            "image".into(),
            OutputPath::new("slot.wav".into()).unwrap(),
        )
        .unwrap(),
    );
    for path in ["SLOT.WAV", "slot.wav/child"] {
        let mut duplicate = definition.clone();
        duplicate.outputs.push(
            OutputMapping::for_audio(
                OutputId::new(2).unwrap(),
                AudioComponentId::new(1).unwrap(),
                "sound".into(),
                OutputPath::new(path.into()).unwrap(),
            )
            .unwrap(),
        );
        assert!(matches!(
            SkinProject::from_definition(duplicate),
            Err(ModelError::OutputConflict(_))
        ));
    }
    definition.outputs.push(
        OutputMapping::for_audio(
            OutputId::new(2).unwrap(),
            AudioComponentId::new(99).unwrap(),
            "sound".into(),
            OutputPath::new("other.wav".into()).unwrap(),
        )
        .unwrap(),
    );
    assert!(matches!(
        SkinProject::from_definition(definition),
        Err(ModelError::MissingReference(ObjectId::AudioComponent(_)))
    ));
}
#[test]
fn v1_upgrade_is_explicit_preserves_opaque_content_and_rejects_unknown_versions() {
    let mut old = ProjectDefinition::empty(ProjectId::new(7).unwrap());
    old.model_version = 1;
    old.assets.push(asset(1, AssetKind::Opaque));
    let document = DocumentId::new(3).unwrap();
    old.documents.push(ElementDocument::new(
        document,
        PixelSize::new(2, 2).unwrap(),
        LayerTree::default(),
    ));
    old.outputs.push(
        OutputMapping::new(
            OutputId::new(4).unwrap(),
            document,
            "image".into(),
            OutputPath::new("image.png".into()).unwrap(),
        )
        .unwrap(),
    );
    old.configuration = SkinConfiguration::new(vec![
        ConfigRecord::new(b"unknown=value\r\n".to_vec(), ConfigInterpretation::Opaque).unwrap(),
    ]);
    let before = old.clone();
    assert_eq!(
        SkinProject::from_definition(old.clone()),
        Err(ModelError::UnsupportedVersion(1))
    );
    let upgraded = old.upgrade_v1().unwrap();
    assert_eq!(upgraded.assets, before.assets);
    assert_eq!(upgraded.documents, before.documents);
    assert_eq!(upgraded.outputs, before.outputs);
    assert_eq!(upgraded.configuration, before.configuration);
    assert_eq!(upgraded.model_version, MODEL_VERSION);
    assert!(SkinProject::from_definition(upgraded).is_ok());
    let mut invalid = input();
    invalid.model_version = 1;
    assert_eq!(
        invalid.upgrade_v1(),
        Err(ModelError::InvalidValue("v1_contains_v2_content"))
    );
    let mut future = before;
    future.model_version = 99;
    assert_eq!(future.upgrade_v1(), Err(ModelError::UnsupportedVersion(99)));
}
