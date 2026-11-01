use skin_editor_audio::*;
use skin_editor_domain::*;
fn plan(kind: AssetKind, gain: f64, visible: bool) -> SoundPlan {
    let mut definition = ProjectDefinition::empty(ProjectId::new(1).unwrap());
    let asset = AssetId::new(1).unwrap();
    let component = AudioComponentId::new(1).unwrap();
    definition.assets.push(
        Asset::new(
            asset,
            "unrecognised.codec".into(),
            BlobRef::new([7; 32], 8),
            kind,
        )
        .unwrap(),
    );
    definition.audio_components.push(
        AudioComponent::new(
            component,
            SoundSlotKey::new("fixture.result".into()).unwrap(),
            asset,
        )
        .with_properties(Gain::new(gain).unwrap(), visible),
    );
    SoundPlan::build(
        &SkinProject::from_definition(definition).unwrap(),
        component,
    )
    .unwrap()
}
fn known() -> AssetKind {
    AssetKind::Audio(AudioMetadata::new(48000, 2, 2).unwrap())
}
#[test]
fn undecodable_unedited_source_is_preserved_exactly_without_a_decoder() {
    let plan = plan(AssetKind::Opaque, 1.0, true);
    let original = [255, 0, 3, 12, 128, 5, 0, 99];
    let output = plan.original_bytes(plan.source, &original).unwrap();
    assert_eq!(output, original);
    assert!(std::ptr::eq(output.as_ptr(), original.as_ptr()));
    assert_eq!(plan.process_pcm(&[], 0), Err(AudioError::UnsupportedDecode));
    assert_eq!(
        plan.original_bytes(plan.source, &original[..7]),
        Err(AudioError::WrongSource)
    );
    assert_eq!(
        plan.original_bytes(BlobRef::new([8; 32], 8), &original),
        Err(AudioError::WrongSource)
    );
}
#[test]
fn edited_unsupported_source_is_diagnosed_instead_of_exported_with_ignored_edits() {
    let plan = plan(AssetKind::Opaque, 0.5, true);
    assert_eq!(plan.output_action(), Err(AudioError::UnsupportedDecode));
    assert_eq!(
        plan.original_bytes(plan.source, &[0; 8]),
        Err(AudioError::UnsupportedDecode)
    );
}
#[test]
fn hidden_unknown_audio_requests_valid_encoded_silence_and_never_original_or_omission() {
    let plan = plan(AssetKind::Opaque, 1.0, false);
    assert_eq!(plan.output_action(), Ok(OutputAction::EncodeSilence));
    assert_eq!(
        plan.original_bytes(plan.source, &[0; 8]),
        Err(AudioError::ProcessingRequired)
    );
}
#[test]
fn gain_matches_independent_reference_and_does_not_modify_source() {
    let source = [-1.0, -0.25, 0.5, 1.0];
    for gain in [0.0, 0.5, 1.0] {
        let plan = plan(known(), gain, true);
        let actual = plan.process_pcm(&source, 4).unwrap();
        let expected: Vec<_> = source.iter().map(|s| (*s as f64 * gain) as f32).collect();
        assert_eq!(actual, expected);
    }
    assert_eq!(source, [-1.0, -0.25, 0.5, 1.0]);
    assert_eq!(
        plan(known(), 1.0, false).process_pcm(&source, 4).unwrap(),
        [0.0; 4]
    );
}
#[test]
fn processing_rejects_invalid_pcm_and_excess_capacity_before_allocating() {
    let plan = plan(known(), 0.5, true);
    assert_eq!(
        plan.process_pcm(&[0.0; 4], 3),
        Err(AudioError::ResourceLimit)
    );
    assert_eq!(
        plan.process_pcm(&[0.0; 3], 4),
        Err(AudioError::InvalidSamples)
    );
    for value in [f32::NAN, f32::INFINITY, 2.0] {
        assert_eq!(
            plan.process_pcm(&[value; 4], 4),
            Err(AudioError::InvalidSamples)
        );
    }
    assert_eq!(
        plan.original_bytes(plan.source, &[0; 8]),
        Err(AudioError::ProcessingRequired)
    );
}
