use skin_editor_application::{values::*, *};

#[derive(Default)]
struct Capture {
    source: Option<SkinAudioSource>,
    bytes: Vec<u8>,
    fail: bool,
    retained: Vec<DraftId>,
    cancelled: Vec<DraftId>,
}
impl SkinAudioPort for Capture {
    fn retained_drafts(&self) -> Vec<DraftId> {
        self.retained.clone()
    }
    fn synchronize(
        &mut self,
        project: &SkinProject,
        source: SkinAudioSource,
        cancelled: &[DraftId],
    ) -> Result<(), SkinAudioError> {
        self.source = Some(source);
        self.cancelled = cancelled.to_vec();
        self.bytes = project.configuration().to_bytes();
        if self.fail {
            Err(SkinAudioError::ResourceLimit)
        } else {
            Ok(())
        }
    }
}
struct Driver {
    session: ProjectSession,
    next: u128,
}
impl Driver {
    fn new() -> Self {
        Self {
            session: ProjectSession::new(7, 70).unwrap(),
            next: 1,
        }
    }
    fn id(&mut self) -> RequestId {
        let id = self.next;
        self.next += 1;
        RequestId::new(id).unwrap()
    }
    fn commit(&mut self) {
        let request = CommandRequest {
            api_version: API_VERSION,
            session_id: self.session.project_info().session_id,
            request_id: self.id(),
            expected: self.session.project_info().state,
            commands: vec![insert()],
        };
        self.session.submit(request).unwrap();
    }
    fn nav_request(&mut self, action: HistoryAction) -> HistoryRequest {
        HistoryRequest {
            api_version: API_VERSION,
            session_id: self.session.project_info().session_id,
            request_id: self.id(),
            expected: self.session.project_info().state,
            action,
        }
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
    fn source(&self) -> SkinAudioSource {
        let mut capture = Capture::default();
        self.session.synchronize_audio(&mut capture).unwrap();
        capture.source.unwrap()
    }
}
fn insert() -> Command {
    Command::InsertConfigRecord {
        index: 0,
        record: ConfigRecord::new(b"fixture".to_vec(), ConfigInterpretation::Opaque).unwrap(),
    }
}

#[test]
fn navigation_source_changes_even_when_returning_to_the_same_node_between_syncs() {
    let mut d = Driver::new();
    d.commit();
    let a = d.source();
    assert_eq!(a.navigation_revision, 0);
    d.commit();
    let undo = d.nav_request(HistoryAction::Undo);
    d.session.navigate(undo.clone()).unwrap();
    let back = d.source();
    assert_eq!(back.state.head, a.state.head);
    assert_ne!(back.state.revision, a.state.revision);
    assert_eq!(back.navigation_revision, back.state.revision);
    let cached = d.session.navigate(undo).unwrap();
    assert_eq!(cached.state, back.state);
    assert_eq!(d.source(), back);
    d.commit();
    assert_eq!(d.source().navigation_revision, back.navigation_revision);
}
#[test]
fn noop_and_failed_navigation_do_not_invalidate_audio_selection() {
    let mut d = Driver::new();
    let before = d.source();
    let noop = d.nav_request(HistoryAction::Undo);
    assert_eq!(
        d.session.navigate(noop).unwrap().status,
        HistoryStatus::NoChange
    );
    assert_eq!(d.source(), before);
    let missing = d.nav_request(HistoryAction::Checkout {
        node: NodeId::new(999).unwrap(),
    });
    assert!(d.session.navigate(missing).is_err());
    assert_eq!(d.source(), before);
    d.commit();
    let committed = d.source();
    let redo = d.nav_request(HistoryAction::Redo { child: None });
    d.session.navigate(redo).unwrap();
    assert_eq!(d.source(), committed);
}
#[test]
fn port_selects_actual_draft_and_adapter_failure_never_changes_project_or_history() {
    let mut d = Driver::new();
    let before = d.session.snapshot();
    d.draft(DraftAction::Begin {
        expected: before.state,
    });
    d.draft(DraftAction::Replace {
        expected: d.ds(),
        commands: vec![insert()],
    });
    let mut capture = Capture {
        fail: true,
        ..Default::default()
    };
    assert_eq!(
        d.session.synchronize_audio(&mut capture),
        Err(SkinAudioError::ResourceLimit)
    );
    assert_eq!(capture.bytes, b"fixture");
    assert_eq!(capture.source.unwrap().draft, Some(d.ds()));
    assert_eq!(d.session.snapshot(), before);
    assert_eq!(d.session.history_len(), 1);
    d.draft(DraftAction::Cancel { expected: d.ds() });
    capture.fail = false;
    d.session.synchronize_audio(&mut capture).unwrap();
    assert_eq!(capture.source.unwrap().draft, None);
    assert!(capture.bytes.is_empty());
    assert_eq!(d.session.snapshot(), before);
}

#[test]
fn cancellation_facts_survive_other_edits_but_do_not_label_committed_drafts_cancelled() {
    let mut d = Driver::new();
    d.draft(DraftAction::Begin {
        expected: d.session.project_info().state,
    });
    let cancelled = d.ds();
    d.draft(DraftAction::Cancel {
        expected: cancelled,
    });
    d.commit();
    d.draft(DraftAction::Begin {
        expected: d.session.project_info().state,
    });
    d.draft(DraftAction::Replace {
        expected: d.ds(),
        commands: vec![insert()],
    });
    let committed = d.ds();
    d.draft(DraftAction::Commit {
        expected: committed,
    });
    let mut capture = Capture {
        retained: vec![cancelled.id, committed.id, DraftId::new(999).unwrap()],
        ..Default::default()
    };
    d.session.synchronize_audio(&mut capture).unwrap();
    assert_eq!(capture.cancelled, [cancelled.id]);
    capture.retained = vec![cancelled.id; MAX_AUDIO_DRAFT_REFERENCES + 1];
    assert_eq!(
        d.session.synchronize_audio(&mut capture),
        Err(SkinAudioError::ResourceLimit)
    );
}
