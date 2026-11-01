use skin_editor_application::{values::*, *};
#[derive(Default)]
struct Port {
    steps: Vec<&'static str>,
    sources: Vec<SkinAudioSource>,
    fail: bool,
    assets: Vec<AssetAudition>,
}
impl SkinAudioPort for Port {
    fn synchronize(
        &mut self,
        _: &SkinProject,
        source: SkinAudioSource,
        _: &[DraftId],
    ) -> Result<(), SkinAudioError> {
        self.steps.push("sync");
        self.sources.push(source);
        if self.fail {
            Err(SkinAudioError::ResourceLimit)
        } else {
            Ok(())
        }
    }
}
impl AudioEditorPort for Port {
    fn audition_asset(&mut self, a: AssetAudition) -> Result<(), AudioEditorError> {
        self.steps.push("asset");
        self.assets.push(a);
        Ok(())
    }
    fn request(&mut self, a: AudioEditorAction) -> Result<(), AudioEditorError> {
        self.steps.push(if a == AudioEditorAction::Enable {
            "enable"
        } else {
            "control"
        });
        Ok(())
    }
    fn poll(&mut self) -> AudioEditorState {
        Default::default()
    }
}
#[test]
fn device_controls_are_read_only_and_enable_selects_source_before_playback() {
    let session = ProjectSession::new(1, 2).unwrap();
    let mut port = Port::default();
    let snapshot = session.snapshot();
    let nodes = session.history_len();
    session
        .control_audio(&mut port, AudioEditorAction::Enable)
        .unwrap();
    session
        .control_audio(&mut port, AudioEditorAction::SceneRunning(false))
        .unwrap();
    assert_eq!(port.steps, vec!["enable", "sync", "sync", "control"]);
    assert_eq!(port.sources[0], session.skin_audio_source());
    assert_eq!(session.snapshot(), snapshot);
    assert_eq!(session.history_len(), nodes);
}
#[test]
fn failed_sync_and_missing_component_do_not_trigger_and_disable_remains_available() {
    let session = ProjectSession::new(1, 2).unwrap();
    let mut port = Port {
        fail: true,
        ..Default::default()
    };
    assert!(
        session
            .control_audio(&mut port, AudioEditorAction::SceneRunning(true))
            .is_err()
    );
    assert_eq!(port.steps, vec!["sync"]);
    session
        .control_audio(&mut port, AudioEditorAction::Disable)
        .unwrap();
    assert_eq!(port.steps, vec!["sync", "control"]);
    port.fail = false;
    port.steps.clear();
    assert_eq!(
        session.control_audio(
            &mut port,
            AudioEditorAction::Audition(AudioComponentId::new(9).unwrap())
        ),
        Err(AudioEditorError::MissingComponent)
    );
    assert_eq!(port.steps, vec!["sync"]);
}
#[test]
fn readonly_source_query_tracks_drafts_and_history_navigation() {
    let mut session = ProjectSession::new(1, 2).unwrap();
    let base = session.skin_audio_source();
    session
        .edit_draft(DraftRequest {
            api_version: API_VERSION,
            session_id: base.session,
            request_id: RequestId::new(1).unwrap(),
            action: DraftAction::Begin {
                expected: base.state,
            },
        })
        .unwrap();
    let draft = session.skin_audio_source();
    assert!(draft.draft.is_some());
    assert_eq!(draft.state, base.state);
    session
        .edit_draft(DraftRequest {
            api_version: API_VERSION,
            session_id: base.session,
            request_id: RequestId::new(2).unwrap(),
            action: DraftAction::Cancel {
                expected: draft.draft.unwrap(),
            },
        })
        .unwrap();
    assert_eq!(session.skin_audio_source(), base);
}

#[test]
fn monitor_and_stop_are_readonly_and_not_blocked_by_project_sync_failure() {
    let session = ProjectSession::new(1, 2).unwrap();
    let snapshot = session.snapshot();
    let mut port = Port {
        fail: true,
        ..Default::default()
    };
    assert_eq!(
        session.control_audio(&mut port, AudioEditorAction::MonitorVolume(1001)),
        Err(AudioEditorError::InvalidVolume)
    );
    assert!(port.steps.is_empty());
    session
        .control_audio(&mut port, AudioEditorAction::MonitorVolume(250))
        .unwrap();
    session
        .control_audio(&mut port, AudioEditorAction::StopAudition)
        .unwrap();
    assert_eq!(port.steps, ["control", "control"]);
    port.fail = false;
    assert_eq!(
        session.control_audio(
            &mut port,
            AudioEditorAction::AuditionAsset(AssetId::new(1).unwrap())
        ),
        Err(AudioEditorError::MissingAsset)
    );
    assert_eq!(session.snapshot(), snapshot);
    assert_eq!(session.history_len(), 1);
}

#[test]
fn library_control_resolves_actual_asset_and_source_without_component_or_history_edits() {
    use std::sync::Arc;
    struct Lease(BlobRef);
    impl BlobLease for Lease {
        fn reference(&self) -> BlobRef {
            self.0
        }
    }
    struct Store;
    impl ResourceStore for Store {
        fn pin(&self, b: BlobRef) -> Result<Arc<dyn BlobLease>, ResourceError> {
            Ok(Arc::new(Lease(b)))
        }
    }
    let mut session =
        ProjectSession::with_resources(1, 2, Box::new(Store), SessionLimits::default()).unwrap();
    let asset = AssetId::new(1).unwrap();
    let image = AssetId::new(2).unwrap();
    let blob = BlobRef::new([1; 32], 8);
    session
        .submit(CommandRequest {
            api_version: API_VERSION,
            session_id: session.project_info().session_id,
            request_id: RequestId::new(1).unwrap(),
            expected: session.project_info().state,
            commands: vec![
                Command::RegisterAsset {
                    asset: Asset::new(asset, "raw.unknown".into(), blob, AssetKind::Opaque)
                        .unwrap(),
                },
                Command::RegisterAsset {
                    asset: Asset::new(
                        image,
                        "image".into(),
                        BlobRef::new([2; 32], 16),
                        AssetKind::RasterImage(PixelSize::new(2, 2).unwrap()),
                    )
                    .unwrap(),
                },
            ],
        })
        .unwrap();
    let snapshot = session.snapshot();
    let nodes = session.history_len();
    let mut port = Port::default();
    session
        .control_audio(&mut port, AudioEditorAction::AuditionAsset(asset))
        .unwrap();
    assert_eq!(port.steps, ["sync", "asset"]);
    assert_eq!(
        port.assets,
        [AssetAudition {
            source: session.skin_audio_source(),
            asset,
            blob,
            metadata: None
        }]
    );
    assert_eq!(
        session.control_audio(&mut port, AudioEditorAction::AuditionAsset(image)),
        Err(AudioEditorError::ImageAsset)
    );
    assert_eq!(port.assets.len(), 1);
    assert_eq!(session.snapshot(), snapshot);
    assert_eq!(session.history_len(), nodes);
    assert_eq!(snapshot.project.audio_components().len(), 0);
}
