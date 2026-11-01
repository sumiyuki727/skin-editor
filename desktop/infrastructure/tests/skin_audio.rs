use skin_editor_application::{values::*, *};
use skin_editor_audio::*;
use skin_editor_infrastructure::SkinAudioController;
use std::sync::Arc;

fn component() -> AudioComponentId {
    AudioComponentId::new(1).unwrap()
}
fn metadata() -> AudioMetadata {
    AudioMetadata::new(48000, 1, 4).unwrap()
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
        Ok(Arc::new(Lease(blob)))
    }
    fn audio_metadata(&self, _: BlobRef) -> Option<AudioMetadata> {
        Some(metadata())
    }
}
struct Driver {
    session: ProjectSession,
    controller: SkinAudioController,
    next: u128,
}
impl Driver {
    fn new() -> Self {
        let mut d = Self {
            session: ProjectSession::with_resources(
                7,
                70,
                Box::new(Store),
                SessionLimits::default(),
            )
            .unwrap(),
            controller: SkinAudioController::new(48000, 1).unwrap(),
            next: 1,
        };
        let asset = AssetId::new(1).unwrap();
        d.commit(vec![
            Command::RegisterAsset {
                asset: Asset::new(
                    asset,
                    "fixture.wav".into(),
                    BlobRef::new([1; 32], 8),
                    AssetKind::Audio(metadata()),
                )
                .unwrap(),
            },
            Command::CreateAudioComponent {
                component: AudioComponent::new(
                    component(),
                    SoundSlotKey::new("fixture.hit".into()).unwrap(),
                    asset,
                ),
            },
        ]);
        d
    }
    fn id(&mut self) -> RequestId {
        let id = self.next;
        self.next += 1;
        RequestId::new(id).unwrap()
    }
    fn commit(&mut self, commands: Vec<Command>) {
        let request = CommandRequest {
            api_version: API_VERSION,
            session_id: self.session.project_info().session_id,
            request_id: self.id(),
            expected: self.session.project_info().state,
            commands,
        };
        self.session.submit(request).unwrap();
    }
    fn nav(&mut self, action: HistoryAction) {
        let request = HistoryRequest {
            api_version: API_VERSION,
            session_id: self.session.project_info().session_id,
            request_id: self.id(),
            expected: self.session.project_info().state,
            action,
        };
        self.session.navigate(request).unwrap();
    }
    fn draft(&mut self, action: DraftAction) {
        let request = DraftRequest {
            api_version: API_VERSION,
            session_id: self.session.project_info().session_id,
            request_id: self.id(),
            action,
        };
        self.session.edit_draft(request).unwrap();
    }
    fn ds(&self) -> DraftState {
        self.session
            .draft_info(self.session.project_info().session_id)
            .unwrap()
            .unwrap()
            .state
    }
    fn begin(&mut self) {
        self.draft(DraftAction::Begin {
            expected: self.session.project_info().state,
        });
    }
    fn sync(&mut self) {
        self.session
            .synchronize_audio(&mut self.controller)
            .unwrap();
    }
    fn job(&mut self) -> BuildRequest {
        self.sync();
        let mut jobs = self.controller.take_builds();
        assert_eq!(jobs.len(), 1);
        jobs.pop().unwrap()
    }
    fn accept(&mut self, job: BuildRequest) -> Completion {
        let sound = PreparedSound::from_source(
            job.plan,
            &[0.5; 4],
            &self.controller.runtime().pcm_budget(),
        )
        .unwrap();
        self.controller
            .accept_prepared(&self.session, job, Ok(sound))
            .unwrap()
    }
    fn fire(&mut self, lane: Channel) -> TriggerReceipt {
        self.sync();
        let runtime = self.controller.runtime();
        runtime
            .trigger(runtime.token(lane), component(), runtime.mix_frame())
            .unwrap()
    }
    fn ready(&mut self) -> BuildRequest {
        let job = self.job();
        self.accept(job);
        job
    }
}
fn properties(gain: f64, visible: bool) -> Command {
    Command::SetAudioProperties {
        component: component(),
        gain: Gain::new(gain).unwrap(),
        visible,
    }
}

#[test]
fn cancellation_is_seen_before_late_completion_without_an_intervening_ui_refresh() {
    let mut d = Driver::new();
    d.ready();
    let base = d.session.snapshot();
    let nodes = d.session.history_len();
    d.begin();
    d.draft(DraftAction::Replace {
        expected: d.ds(),
        commands: vec![properties(0.5, true)],
    });
    let first = d.job();
    let sound =
        PreparedSound::from_source(first.plan, &[0.5; 4], &d.controller.runtime().pcm_budget())
            .unwrap();
    let weak = sound.pcm().downgrade();
    d.controller
        .accept_prepared(&d.session, first, Ok(sound))
        .unwrap();
    d.fire(Channel::Audition);
    d.draft(DraftAction::Replace {
        expected: d.ds(),
        commands: vec![properties(0.25, true)],
    });
    let late = d.job();
    let runtime = d.controller.runtime();
    runtime
        .trigger(runtime.token(Channel::Audition), component(), 2)
        .unwrap();
    d.draft(DraftAction::Cancel { expected: d.ds() });
    assert_eq!(d.accept(late), Completion::Stale);
    assert!(!weak.is_alive());
    let runtime = d.controller.runtime();
    assert_eq!(runtime.queued(), 0);
    assert_eq!(runtime.voices().count(), 0);
    assert_eq!(runtime.status(component()).unwrap().desired.draft, None);
    assert_eq!(d.session.snapshot(), base);
    assert_eq!(d.session.history_len(), nodes);
    let restore = d.controller.take_builds().pop().unwrap();
    d.accept(restore);
    assert!(!d.fire(Channel::Audition).stale);
}
#[test]
fn history_change_then_new_branch_rejects_old_worker_and_restores_branch_recipe() {
    let mut d = Driver::new();
    let a = d.ready();
    let a_node = d.session.project_info().state.head;
    d.commit(vec![properties(0.5, true)]);
    let b = d.job();
    let b_node = d.session.project_info().state.head;
    d.fire(Channel::Audition);
    d.nav(HistoryAction::Checkout { node: a_node });
    d.commit(vec![properties(0.75, true)]);
    assert_eq!(d.accept(b), Completion::Stale);
    assert_eq!(d.controller.runtime().voices().count(), 0);
    let c = d.controller.take_builds().pop().unwrap();
    assert_eq!(c.plan.gain, Gain::new(0.75).unwrap());
    d.accept(c);
    d.nav(HistoryAction::Checkout { node: b_node });
    let back = d.job();
    assert_eq!(back.plan.gain, Gain::new(0.5).unwrap());
    d.accept(back);
    assert_eq!(d.fire(Channel::Audition).origin.node, b_node.get());
    assert_eq!(d.controller.runtime().scene_frame(), 0);
    assert_eq!(a.origin.node, a_node.get());
}
#[test]
fn returning_to_same_node_between_syncs_still_cleans_both_channels() {
    let mut d = Driver::new();
    let a = d.ready();
    let runtime = d.controller.runtime();
    runtime
        .set_scene_running(runtime.token(Channel::Scene), true)
        .unwrap();
    d.fire(Channel::Scene);
    d.fire(Channel::Audition);
    d.commit(vec![properties(0.5, true)]);
    d.nav(HistoryAction::Undo);
    let back = d.job();
    assert_eq!(back.origin.node, a.origin.node);
    assert_ne!(
        back.origin.navigation_revision,
        a.origin.navigation_revision
    );
    assert_eq!(d.controller.runtime().voices().count(), 0);
    d.accept(back);
    assert_eq!(d.fire(Channel::Audition).origin, back.origin);
}
#[test]
fn draft_commit_keeps_old_voice_origin_but_new_trigger_uses_committed_source() {
    let mut d = Driver::new();
    d.ready();
    let nodes = d.session.history_len();
    d.begin();
    d.draft(DraftAction::Replace {
        expected: d.ds(),
        commands: vec![properties(0.5, true)],
    });
    let draft = d.job();
    d.accept(draft);
    d.fire(Channel::Audition);
    d.draft(DraftAction::Commit { expected: d.ds() });
    d.sync();
    assert!(d.controller.take_builds().is_empty());
    assert_eq!(
        d.controller.runtime().voices().next().unwrap().origin,
        draft.origin
    );
    assert_eq!(d.fire(Channel::Audition).origin.draft, None);
    assert_eq!(d.session.history_len(), nodes + 1);
}
#[test]
fn frozen_scene_can_update_sound_and_audition_without_advancing_scene_or_project() {
    let mut d = Driver::new();
    d.ready();
    d.commit(vec![properties(0.5, true)]);
    let job = d.job();
    d.accept(job);
    let before = d.session.snapshot();
    d.fire(Channel::Audition);
    let runtime = d.controller.runtime();
    assert_eq!(
        runtime.trigger(runtime.token(Channel::Scene), component(), 0),
        Err(RuntimeError::Frozen)
    );
    let mut out = [0.0; 4];
    runtime.mix(&mut out).unwrap();
    assert_eq!(out, [0.25; 4]);
    assert_eq!(runtime.scene_frame(), 0);
    assert_eq!(runtime.mix_frame(), 4);
    assert_eq!(d.session.snapshot(), before);
}
#[test]
fn repeated_sync_keeps_untaken_jobs_and_hidden_edit_does_not_wait_for_decoder() {
    let mut d = Driver::new();
    d.sync();
    d.sync();
    let jobs = d.controller.take_builds();
    assert_eq!(jobs.len(), 1);
    d.commit(vec![properties(1.0, false)]);
    d.sync();
    assert!(d.controller.take_builds().is_empty());
    assert_eq!(d.accept(jobs[0]), Completion::Stale);
    assert!(d.fire(Channel::Audition).silent);
}

#[test]
fn cancel_followed_by_unrelated_commit_cannot_keep_a_cancelled_draft_voice() {
    let mut d = Driver::new();
    d.ready();
    d.begin();
    d.draft(DraftAction::Replace {
        expected: d.ds(),
        commands: vec![properties(0.5, true)],
    });
    let draft = d.job();
    d.accept(draft);
    d.fire(Channel::Audition);
    d.draft(DraftAction::Cancel { expected: d.ds() });
    // Deliberately skip synchronization until another command has committed.
    d.commit(vec![Command::CreateDocument {
        document: ElementDocument::new(
            DocumentId::new(1).unwrap(),
            PixelSize::new(1, 1).unwrap(),
            LayerTree::default(),
        ),
    }]);
    d.sync();
    assert_eq!(d.controller.runtime().voices().count(), 0);
    assert_eq!(d.accept(draft), Completion::Stale);
}
