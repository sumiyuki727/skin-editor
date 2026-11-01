use skin_editor_application::{values::*, *};
use std::sync::Arc;

fn aid(n: u128) -> AssetId {
    AssetId::new(n).unwrap()
}
fn sid(n: u128) -> AudioComponentId {
    AudioComponentId::new(n).unwrap()
}
fn did() -> DocumentId {
    DocumentId::new(1).unwrap()
}
fn metadata() -> AudioMetadata {
    AudioMetadata::new(48000, 2, 2).unwrap()
}
fn blob(n: u8) -> BlobRef {
    BlobRef::new([n; 32], 8)
}
struct Lease(BlobRef);
impl BlobLease for Lease {
    fn reference(&self) -> BlobRef {
        self.0
    }
}
struct Store;
impl ResourceStore for Store {
    fn pin(&self, blob: BlobRef) -> Result<Arc<dyn BlobLease>, ResourceError> {
        if [1, 2].iter().any(|n| blob == self::blob(*n)) {
            Ok(Arc::new(Lease(blob)))
        } else {
            Err(ResourceError::Missing)
        }
    }
    fn audio_metadata(&self, blob: BlobRef) -> Option<AudioMetadata> {
        (blob == self::blob(1)).then_some(metadata())
    }
}
struct Driver {
    session: ProjectSession,
    next: u128,
}
impl Driver {
    fn new() -> Self {
        Self {
            session: ProjectSession::with_resources(
                7,
                70,
                Box::new(Store),
                SessionLimits::default(),
            )
            .unwrap(),
            next: 1,
        }
    }
    fn id(&mut self) -> RequestId {
        let n = self.next;
        self.next += 1;
        RequestId::new(n).unwrap()
    }
    fn request(&mut self, commands: Vec<Command>) -> CommandRequest {
        CommandRequest {
            api_version: API_VERSION,
            session_id: self.scope(),
            request_id: self.id(),
            expected: self.session.project_info().state,
            commands,
        }
    }
    fn commit(&mut self, commands: Vec<Command>) -> CommandResult {
        let r = self.request(commands);
        self.session.submit(r).unwrap()
    }
    fn navigate(&mut self, action: HistoryAction) {
        let r = HistoryRequest {
            api_version: API_VERSION,
            session_id: self.scope(),
            request_id: self.id(),
            expected: self.session.project_info().state,
            action,
        };
        self.session.navigate(r).unwrap();
    }
    fn draft(&mut self, action: DraftAction) -> Result<DraftOutcome, DraftError> {
        let r = DraftRequest {
            api_version: API_VERSION,
            session_id: self.scope(),
            request_id: self.id(),
            action,
        };
        self.session.edit_draft(r).map(|r| r.outcome)
    }
    fn scope(&self) -> HistorySessionId {
        self.session.project_info().session_id
    }
    fn ds(&self) -> DraftState {
        self.session
            .draft_info(self.scope())
            .unwrap()
            .unwrap()
            .state
    }
    fn sound(&self, n: u128) -> AudioComponent {
        self.session
            .snapshot()
            .project
            .audio_component(sid(n))
            .unwrap()
            .clone()
    }
    fn baseline(&mut self) {
        self.commit(vec![
            register(1, AssetKind::Audio(metadata())),
            Command::CreateAudioComponent {
                component: sound(1, 1),
            },
            Command::CreateAudioComponent {
                component: sound(2, 1),
            },
        ]);
    }
}
fn register(n: u128, kind: AssetKind) -> Command {
    Command::RegisterAsset {
        asset: Asset::new(aid(n), format!("source-{n}"), blob(n as u8), kind).unwrap(),
    }
}
fn sound(n: u128, asset: u128) -> AudioComponent {
    AudioComponent::new(
        sid(n),
        SoundSlotKey::new(format!("fixture.{n}")).unwrap(),
        aid(asset),
    )
}
fn properties(n: u128, gain: f64, visible: bool) -> Command {
    Command::SetAudioProperties {
        component: sid(n),
        gain: Gain::new(gain).unwrap(),
        visible,
    }
}
fn mapping() -> OutputMapping {
    OutputMapping::for_audio(
        OutputId::new(1).unwrap(),
        sid(1),
        "fixture.hit".into(),
        OutputPath::new("hit.wav".into()).unwrap(),
    )
    .unwrap()
}

#[test]
fn sounds_are_independent_reusable_and_report_no_image_changes() {
    let mut d = Driver::new();
    d.baseline();
    let changed = d.commit(vec![properties(1, 0.5, false)]);
    assert_eq!(
        changed.changed_objects,
        vec![ObjectId::AudioComponent(sid(1))]
    );
    assert_eq!(d.sound(2), sound(2, 1));
    assert_eq!(d.session.snapshot().project.documents().len(), 0);
    assert_eq!(d.session.snapshot().project.components().iter().count(), 0);
    d.navigate(HistoryAction::Undo);
    assert_eq!(d.sound(1), sound(1, 1));
    d.navigate(HistoryAction::Redo { child: None });
    assert!(!d.sound(1).visible());
}
#[test]
fn referenced_asset_and_output_owner_require_explicit_unlinking() {
    let mut d = Driver::new();
    d.baseline();
    d.commit(vec![Command::SetOutputMapping { mapping: mapping() }]);
    let before = d.session.snapshot();
    let request = d.request(vec![Command::RemoveAsset { asset: aid(1) }]);
    assert!(matches!(
        d.session.submit(request),
        Err(CommandError::Command {
            reason: CommandFailure::AudioAssetInUse { .. },
            ..
        })
    ));
    let request = d.request(vec![Command::RemoveAudioComponent { component: sid(1) }]);
    assert!(matches!(
        d.session.submit(request),
        Err(CommandError::Command {
            reason: CommandFailure::AudioComponentInUse(_),
            ..
        })
    ));
    assert_eq!(d.session.snapshot(), before);
    d.commit(vec![
        Command::RemoveOutputMapping {
            output: OutputId::new(1).unwrap(),
        },
        Command::RemoveAudioComponent { component: sid(1) },
        Command::RemoveAudioComponent { component: sid(2) },
        Command::RemoveAsset { asset: aid(1) },
    ]);
    assert_eq!(d.session.snapshot().project.assets().iter().len(), 0);
    d.navigate(HistoryAction::Undo);
    assert_eq!(d.session.snapshot().project, before.project);
}
#[test]
fn typed_metadata_is_checked_even_for_already_pinned_opaque_bytes() {
    let mut d = Driver::new();
    d.commit(vec![register(2, AssetKind::Opaque)]);
    let before = d.session.snapshot();
    let forged = Asset::new(
        aid(3),
        "forged".into(),
        blob(2),
        AssetKind::Audio(metadata()),
    )
    .unwrap();
    let request = d.request(vec![
        Command::RegisterAsset { asset: forged },
        Command::CreateAudioComponent {
            component: sound(1, 3),
        },
    ]);
    assert_eq!(
        d.session.submit(request),
        Err(CommandError::Resource {
            blob: blob(2),
            error: ResourceError::MetadataMismatch
        })
    );
    assert_eq!(d.session.snapshot(), before);
    assert_eq!(d.session.history_len(), 2);
    let forged = Asset::new(
        aid(4),
        "wrong rate".into(),
        blob(1),
        AssetKind::Audio(AudioMetadata::new(44100, 2, 2).unwrap()),
    )
    .unwrap();
    let request = d.request(vec![Command::RegisterAsset { asset: forged }]);
    assert!(matches!(
        d.session.submit(request),
        Err(CommandError::Resource {
            error: ResourceError::MetadataMismatch,
            ..
        })
    ));
    d.commit(vec![register(1, AssetKind::Audio(metadata()))]);
}
#[test]
fn failed_batch_noop_replay_and_conflict_keep_atomic_history() {
    let mut d = Driver::new();
    d.baseline();
    d.commit(vec![Command::CreateDocument {
        document: ElementDocument::new(did(), PixelSize::new(1, 1).unwrap(), LayerTree::default()),
    }]);
    let before = d.session.snapshot();
    let len = d.session.history_len();
    let request = d.request(vec![
        Command::SetDocumentVisibility {
            document: did(),
            visible: false,
        },
        properties(1, 0.5, false),
        Command::SetAudioAsset {
            component: sid(2),
            asset: aid(99),
        },
    ]);
    assert!(d.session.submit(request).is_err());
    assert_eq!(d.session.snapshot(), before);
    assert_eq!(d.session.history_len(), len);
    assert_eq!(
        d.commit(vec![properties(1, 1.0, true)]).status,
        CommandStatus::NoChange
    );
    assert_eq!(d.session.history_len(), len);
    let stale = d.request(vec![properties(1, 0.75, true)]);
    let request = d.request(vec![properties(1, 0.5, true)]);
    let actual = d.session.submit(request.clone()).unwrap();
    assert_eq!(d.session.submit(request), Ok(actual));
    assert!(matches!(
        d.session.submit(stale),
        Err(CommandError::Conflict { .. })
    ));
    assert_eq!(d.session.history_len(), len + 1);
}
#[test]
fn branch_restore_retains_shared_and_replaced_raw_resources() {
    let mut d = Driver::new();
    d.baseline();
    let a = d.session.project_info().state.head;
    d.commit(vec![
        register(2, AssetKind::Opaque),
        Command::SetAudioAsset {
            component: sid(1),
            asset: aid(2),
        },
    ]);
    let b = d.session.project_info().state.head;
    let old_b = d.session.history_snapshot(d.scope(), b).unwrap();
    d.navigate(HistoryAction::Checkout { node: a });
    d.commit(vec![properties(1, 0.25, false)]);
    let c = d.session.project_info().state.head;
    for _ in 0..4 {
        d.navigate(HistoryAction::Checkout { node: b });
        assert_eq!(d.sound(1).asset(), aid(2));
        assert_eq!(d.sound(2).asset(), aid(1));
        d.navigate(HistoryAction::Checkout { node: c });
        assert_eq!(d.sound(1).gain(), Gain::new(0.25).unwrap());
        assert!(!d.sound(1).visible());
    }
    assert_eq!(d.session.history_snapshot(d.scope(), b).unwrap(), old_b);
    assert_eq!(
        d.session
            .history_statistics(d.scope())
            .unwrap()
            .retained_blobs,
        2
    );
}
#[test]
fn gain_draft_three_hundred_updates_commit_once_and_busy_is_preserved() {
    let mut d = Driver::new();
    d.baseline();
    let before = d.session.snapshot();
    let len = d.session.history_len();
    d.draft(DraftAction::Begin {
        expected: before.state,
    })
    .unwrap();
    for i in 0..300 {
        d.draft(DraftAction::Replace {
            expected: d.ds(),
            commands: vec![properties(1, i as f64 / 600.0, true)],
        })
        .unwrap();
    }
    assert_eq!(d.session.snapshot(), before);
    assert_eq!(d.session.history_len(), len);
    let request = d.request(vec![properties(2, 0.5, true)]);
    assert!(matches!(
        d.session.submit(request),
        Err(CommandError::Busy(_))
    ));
    d.draft(DraftAction::Commit { expected: d.ds() }).unwrap();
    assert_eq!(d.session.history_len(), len + 1);
    assert_eq!(d.sound(1).gain(), Gain::new(299.0 / 600.0).unwrap());
    d.navigate(HistoryAction::Undo);
    assert_eq!(d.session.snapshot().project, before.project);
}
#[test]
fn cancelled_failed_and_noop_audio_drafts_leave_no_project_or_node() {
    let mut d = Driver::new();
    d.baseline();
    let before = d.session.snapshot();
    let len = d.session.history_len();
    d.draft(DraftAction::Begin {
        expected: before.state,
    })
    .unwrap();
    let initial = d.ds();
    assert!(
        d.draft(DraftAction::Replace {
            expected: initial,
            commands: vec![
                properties(1, 0.5, false),
                Command::SetAudioAsset {
                    component: sid(1),
                    asset: aid(99)
                }
            ]
        })
        .is_err()
    );
    assert_eq!(d.ds(), initial);
    d.draft(DraftAction::Replace {
        expected: initial,
        commands: vec![properties(1, 0.5, false)],
    })
    .unwrap();
    d.draft(DraftAction::Cancel { expected: d.ds() }).unwrap();
    assert!(
        d.draft(DraftAction::Replace {
            expected: initial,
            commands: vec![properties(1, 0.0, false)]
        })
        .is_err()
    );
    d.draft(DraftAction::Begin {
        expected: before.state,
    })
    .unwrap();
    d.draft(DraftAction::Replace {
        expected: d.ds(),
        commands: vec![properties(1, 1.0, true)],
    })
    .unwrap();
    d.draft(DraftAction::Commit { expected: d.ds() }).unwrap();
    assert_eq!(d.session.snapshot(), before);
    assert_eq!(d.session.history_len(), len);
}
#[test]
fn whole_image_visibility_survives_layer_and_canvas_edits_and_history() {
    let mut d = Driver::new();
    d.commit(vec![
        Command::CreateDocument {
            document: ElementDocument::new(
                did(),
                PixelSize::new(2, 2).unwrap(),
                LayerTree::default(),
            ),
        },
        Command::SetDocumentVisibility {
            document: did(),
            visible: false,
        },
    ]);
    let hidden = d.session.snapshot().project;
    d.commit(vec![
        Command::ResizeDocument {
            document: did(),
            canvas: PixelSize::new(3, 3).unwrap(),
        },
        Command::AddLayer {
            document: did(),
            node: LayerNode {
                id: LayerId::new(1).unwrap(),
                local: Affine2::IDENTITY,
                opacity: Opacity::OPAQUE,
                visible: true,
                kind: LayerKind::Group { children: vec![] },
            },
            parent: None,
            index: 0,
        },
    ]);
    assert!(
        !d.session
            .snapshot()
            .project
            .document(did())
            .unwrap()
            .visible()
    );
    d.navigate(HistoryAction::Undo);
    assert_eq!(d.session.snapshot().project, hidden);
}
