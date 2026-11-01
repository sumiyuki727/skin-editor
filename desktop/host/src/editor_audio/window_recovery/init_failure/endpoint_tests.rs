use super::*;
use skin_editor_infrastructure::{
    BlobStore, WavDecoder,
    separated_native::{ControlledCallback, ReturnGate},
};

pub(super) fn rig_with_ui() -> (
    ProjectSession,
    EditorAudio,
    ControlledCallback,
    ReturnGate,
    PcmBudget,
    skin_editor_ui::EditorUi,
) {
    let store = BlobStore::default();
    let raw = store
        .stage_audio(crate::audio_verification::tone(48000, 2, 440.), &WavDecoder)
        .unwrap();
    let mut session =
        ProjectSession::with_resources(1, 91, Box::new(store.clone()), SessionLimits::default())
            .unwrap();
    let mut ui = skin_editor_ui::EditorUi::default();
    let asset = AssetId::new(1).unwrap();
    assert!(ui.submit(
        &mut session,
        vec![
                Command::RegisterAsset {
                    asset: Asset::new(
                        asset,
                        "controlled.wav".into(),
                        raw.reference(),
                        AssetKind::Audio(raw.audio_metadata().unwrap())
                    )
                    .unwrap(),
                },
                Command::CreateAudioComponent {
                    component: AudioComponent::new(
                        sound::component(),
                        SoundSlotKey::new("controlled".into()).unwrap(),
                        asset
                    ),
                }
            ]
    ));
    for label in [b"branch A".as_slice(), b"branch B".as_slice()] {
        assert!(ui.submit(
            &mut session,
            vec![Command::InsertConfigRecord {
                index: 0,
                record: ConfigRecord::new(label.to_vec(), ConfigInterpretation::Opaque).unwrap(),
            }]
        ));
        ui.navigation(&mut session, HistoryAction::Undo);
    }
    drop(raw);
    let mut a = EditorAudio::candidate(store.clone());
    let pool = a.budget.clone();
    let track = BackgroundTrack::new(pool.reserve(96000).unwrap().allocate(), 48000, 2).unwrap();
    let (client, endpoint) =
        AudioSupplyClient::spawn_separated(track, 48000, 2, pool.clone(), store).unwrap();
    let (native, mut callback, gate) = NativeExecutor::controlled_return(endpoint);
    a.client = Some(client);
    a.native = Some(native);
    a.format = (48000, 2);
    a.background_frames = 48000;
    session.synchronize_audio(&mut a).unwrap();
    callback.render(&mut [0.0; 2], 100);
    (session, a, callback, gate, pool, ui)
}
fn rig() -> (
    ProjectSession,
    EditorAudio,
    ControlledCallback,
    ReturnGate,
    PcmBudget,
) {
    let (s, a, c, g, p, _) = rig_with_ui();
    (s, a, c, g, p)
}

#[test]
fn pending_return_preserves_all_history_and_only_actual_delivery_releases_pcm() {
    let (session, mut a, mut callback, mut gate, pool) = rig();
    let before = Stamp::read(&session).unwrap();
    assert!(a.close().is_err());
    let held = pool.usage().bytes;
    assert!(held > 0);
    for _ in 0..3 {
        assert!(a.stop_pending && !a.close_failed);
        assert!(a.client.is_some() && a.native.is_some());
        for action in [
            AudioEditorAction::Enable,
            AudioEditorAction::SceneRunning(true),
            AudioEditorAction::Audition(sound::component()),
        ] {
            assert!(a.request(action).is_err());
        }
        assert!(session.synchronize_audio(&mut a).is_err());
        let state = a.poll();
        assert!(state.frozen && !state.enabled && state.error.is_some());
        let mut out = [1.0; 32];
        callback.render(&mut out, 200);
        assert_eq!(out, [0.0; 32]);
        assert_eq!(pool.usage().bytes, held);
        assert_eq!(Stamp::read(&session).unwrap(), before);
    }
    drop(callback);
    assert_eq!(pool.usage().bytes, held);
    gate.deliver().unwrap();
    assert_eq!(pool.usage().bytes, held); // Receipt alone is not owner.finish.
    a.close().unwrap();
    assert!(!a.stop_pending && !a.close_failed);
    assert!(a.client.is_none() && a.native.is_none());
    assert_eq!(pool.usage().bytes, 0);
    assert!(a.store.usage().unwrap().bytes > 0);
    assert_eq!(Stamp::read(&session).unwrap(), before);
    assert!(a.poll().frozen && !a.poll().enabled);
}

#[test]
fn disconnected_return_preserves_history_and_blocks_disable_enable_until_owner_exit() {
    let (session, mut a, mut callback, mut gate, pool) = rig();
    let before = Stamp::read(&session).unwrap();
    gate.disconnect();
    assert!(a.close().is_err());
    let held = pool.usage().bytes;
    assert!(held > 0);
    for _ in 0..3 {
        assert!(a.close_failed && !a.stop_pending);
        assert!(a.request(AudioEditorAction::Disable).is_err());
        assert!(a.request(AudioEditorAction::Enable).is_err());
        assert!(a.client.is_some() && a.native.is_some());
        let state = a.poll();
        assert!(state.frozen && !state.enabled && state.error.is_some());
        let mut out = [1.0; 32];
        callback.render(&mut out, 200);
        assert_eq!(out, [0.0; 32]);
        assert_eq!(pool.usage().bytes, held);
        assert_eq!(Stamp::read(&session).unwrap(), before);
    }
    drop(callback);
    assert!(gate.deliver().is_err());
    assert_eq!(pool.usage().bytes, held);
    drop(gate);
    assert!(pool.usage().bytes > 0); // retired supply still owns its allocations
    drop(a);
    assert_eq!(pool.usage().bytes, 0);
    assert_eq!(Stamp::read(&session).unwrap(), before);
}

#[test]
fn wrong_returned_endpoint_is_retained_instead_of_being_dropped_on_finish_error() {
    let (s1, mut a, cb1, mut gate1, pool1) = rig();
    let (s2, mut b, cb2, mut gate2, pool2) = rig();
    let before1 = Stamp::read(&s1).unwrap();
    let before2 = Stamp::read(&s2).unwrap();
    std::mem::swap(&mut a.native, &mut b.native);
    // Join the unrelated owner before comparing its retained allocation;
    // decoder progress is not a resource-release oracle.
    b.client.as_mut().unwrap().close();
    drop(cb2);
    gate2.deliver().unwrap();
    assert!(a.close().is_err());
    assert!(a.close_failed && a.stranded_endpoint.is_some() && a.client.is_some());
    let held = pool2.usage().bytes;
    assert!(held > 0);
    assert!(a.close().is_err());
    assert!(a.request(AudioEditorAction::Enable).is_err());
    assert_eq!(pool2.usage().bytes, held);
    drop(cb1);
    gate1.deliver().unwrap();
    assert!(b.close().is_err());
    assert!(b.stranded_endpoint.is_some());
    assert_eq!(Stamp::read(&s1).unwrap(), before1);
    assert_eq!(Stamp::read(&s2).unwrap(), before2);
    drop(a);
    drop(b);
    assert_eq!(pool1.usage().bytes, 0);
    assert_eq!(pool2.usage().bytes, 0);
}
