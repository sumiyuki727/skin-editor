use skin_editor_application::{values::*, *};
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};

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
    fn request(&mut self, action: DraftAction) -> DraftRequest {
        let id = self.next;
        self.next += 1;
        DraftRequest {
            api_version: API_VERSION,
            session_id: self.session.project_info().session_id,
            request_id: RequestId::new(id).unwrap(),
            action,
        }
    }
    fn apply(&mut self, action: DraftAction) -> DraftOutcome {
        let request = self.request(action);
        self.session.edit_draft(request).unwrap().outcome
    }
    fn state(&self) -> DraftState {
        self.session
            .draft_info(self.scope())
            .unwrap()
            .unwrap()
            .state
    }
    fn begin(&mut self) -> DraftState {
        self.apply(DraftAction::Begin {
            expected: self.session.project_info().state,
        });
        self.state()
    }
    fn replace(&mut self, commands: Vec<Command>) -> DraftState {
        self.apply(DraftAction::Replace {
            expected: self.state(),
            commands,
        });
        self.state()
    }
    fn scope(&self) -> HistorySessionId {
        self.session.project_info().session_id
    }
    fn command_request(&mut self, commands: Vec<Command>) -> CommandRequest {
        let info = self.session.project_info();
        let id = self.next;
        self.next += 1;
        CommandRequest {
            api_version: API_VERSION,
            session_id: info.session_id,
            request_id: RequestId::new(id).unwrap(),
            expected: info.state,
            commands,
        }
    }
}
fn insert(text: &str) -> Command {
    Command::InsertConfigRecord {
        index: 0,
        record: ConfigRecord::new(text.as_bytes().to_vec(), ConfigInterpretation::Opaque).unwrap(),
    }
}
fn bytes(d: &Driver) -> Vec<u8> {
    d.session.snapshot().project.configuration().to_bytes()
}

#[test]
fn replacements_are_relative_to_base_and_one_commit_creates_one_node() {
    let mut d = Driver::new();
    let before = d.session.snapshot();
    d.begin();
    for i in 0..300 {
        d.replace(vec![insert(&format!("drag-{i}"))]);
    }
    assert_eq!(d.session.snapshot(), before);
    assert_eq!(d.session.history_len(), 1);
    let state = d.state();
    let mut detached = d.session.draft_snapshot(d.scope(), state).unwrap();
    assert_eq!(detached.project.configuration().to_bytes(), b"drag-299");
    detached.info.has_changes = false;
    assert!(
        d.session
            .draft_info(d.scope())
            .unwrap()
            .unwrap()
            .has_changes
    );
    let result = match d.apply(DraftAction::Commit { expected: state }) {
        DraftOutcome::Committed(result) => result,
        _ => panic!(),
    };
    assert_eq!(result.status, CommandStatus::Committed);
    assert_eq!(bytes(&d), b"drag-299");
    assert_eq!(d.session.history_len(), 2);
    assert_eq!(result.state.revision, 1);
    assert_eq!(d.session.draft_info(d.scope()).unwrap(), None);
    let info = d.session.project_info();
    d.session
        .navigate(HistoryRequest {
            api_version: API_VERSION,
            session_id: info.session_id,
            request_id: RequestId::new(900).unwrap(),
            expected: info.state,
            action: HistoryAction::Undo,
        })
        .unwrap();
    assert_eq!(d.session.snapshot().project, before.project);
    let info = d.session.project_info();
    d.session
        .navigate(HistoryRequest {
            api_version: API_VERSION,
            session_id: info.session_id,
            request_id: RequestId::new(901).unwrap(),
            expected: info.state,
            action: HistoryAction::Redo { child: None },
        })
        .unwrap();
    assert_eq!(bytes(&d), b"drag-299");
}

#[test]
fn cancel_keeps_content_revision_history_and_candidate_id_admission_unchanged() {
    let mut d = Driver::new();
    let before = d.session.snapshot();
    let id = d.begin().id;
    let command = Command::CreateDocument {
        document: ElementDocument::new(
            DocumentId::new(1).unwrap(),
            PixelSize::new(2, 2).unwrap(),
            LayerTree::default(),
        ),
    };
    d.replace(vec![command.clone()]);
    d.apply(DraftAction::Cancel {
        expected: d.state(),
    });
    assert_eq!(d.session.snapshot(), before);
    assert_eq!(d.session.history_len(), 1);
    assert_ne!(d.begin().id, id);
    d.apply(DraftAction::Cancel {
        expected: d.state(),
    });
    let request = d.command_request(vec![command]);
    d.session.submit(request).unwrap();
}

#[test]
fn empty_and_zero_net_change_drafts_finish_without_history_or_revision() {
    let mut d = Driver::new();
    let before = d.session.snapshot();
    d.begin();
    d.apply(DraftAction::Commit {
        expected: d.state(),
    });
    d.begin();
    d.replace(vec![
        insert("temporary"),
        Command::RemoveConfigRecord { index: 0 },
    ]);
    assert!(
        !d.session
            .draft_info(d.scope())
            .unwrap()
            .unwrap()
            .has_changes
    );
    let result = d.apply(DraftAction::Commit {
        expected: d.state(),
    });
    assert!(matches!(
        result,
        DraftOutcome::Committed(CommandResult {
            status: CommandStatus::NoChange,
            ..
        })
    ));
    assert_eq!(d.session.snapshot(), before);
    assert_eq!(d.session.history_len(), 1);
}

#[test]
fn failed_replacement_keeps_previous_valid_candidate_and_does_not_publish() {
    let mut d = Driver::new();
    let before = d.session.snapshot();
    d.begin();
    let state = d.replace(vec![insert("valid")]);
    let old = d.session.draft_snapshot(d.scope(), state).unwrap();
    let request = d.request(DraftAction::Replace {
        expected: state,
        commands: vec![insert("partial"), Command::RemoveConfigRecord { index: 99 }],
    });
    let failure = d.session.edit_draft(request.clone());
    assert!(matches!(
        failure,
        Err(DraftError::Request(CommandError::Command { index: 1, .. }))
    ));
    assert_eq!(d.session.draft_snapshot(d.scope(), state).unwrap(), old);
    assert_eq!(d.session.edit_draft(request), failure);
    assert_eq!(d.session.snapshot(), before);
    assert_eq!(d.session.history_len(), 1);
    d.apply(DraftAction::Cancel { expected: state });
    assert_eq!(d.session.snapshot(), before);
}

#[test]
fn busy_blocks_new_writes_navigation_and_second_begin_but_not_queries_or_retries() {
    let mut d = Driver::new();
    let command = d.command_request(vec![insert("base")]);
    let original = d.session.submit(command.clone()).unwrap();
    let draft = d.begin();
    let blocked = d.command_request(vec![insert("blocked")]);
    assert_eq!(
        d.session.submit(blocked.clone()),
        Err(CommandError::Busy(draft.id))
    );
    let info = d.session.project_info();
    let navigation = HistoryRequest {
        api_version: API_VERSION,
        session_id: d.scope(),
        request_id: RequestId::new(999).unwrap(),
        expected: info.state,
        action: HistoryAction::Undo,
    };
    assert_eq!(
        d.session.navigate(navigation.clone()),
        Err(HistoryError::Busy(draft.id))
    );
    let second = d.request(DraftAction::Begin {
        expected: info.state,
    });
    assert_eq!(
        d.session.edit_draft(second),
        Err(DraftError::Request(CommandError::Busy(draft.id)))
    );
    assert_eq!(d.session.submit(command), Ok(original));
    d.session
        .history_snapshot(d.scope(), info.state.head)
        .unwrap();
    d.session
        .history_ancestors(d.scope(), info.state.head, 10)
        .unwrap();
    d.session.validate(&blocked).unwrap();
    d.apply(DraftAction::Cancel { expected: draft });
    // Busy was transient and did not cache a rejection or consume these IDs.
    d.session.submit(blocked).unwrap();
    let mut navigation = navigation;
    navigation.expected = d.session.project_info().state;
    d.session.navigate(navigation).unwrap();
}

#[test]
fn stale_base_draft_revision_identity_and_wrong_scope_are_rejected() {
    let mut d = Driver::new();
    let initial = d.session.project_info().state;
    let command = d.command_request(vec![insert("base")]);
    d.session.submit(command).unwrap();
    let begin = d.request(DraftAction::Begin { expected: initial });
    assert!(matches!(
        d.session.edit_draft(begin),
        Err(DraftError::Request(CommandError::Conflict { .. }))
    ));
    let old = d.begin();
    let current = d.replace(vec![insert("next")]);
    let before = d.session.snapshot();
    for action in [
        DraftAction::Commit { expected: old },
        DraftAction::Cancel { expected: old },
        DraftAction::Replace {
            expected: old,
            commands: vec![],
        },
    ] {
        let request = d.request(action);
        assert_eq!(
            d.session.edit_draft(request),
            Err(DraftError::Conflict {
                expected: old,
                actual: current
            })
        );
    }
    assert!(matches!(
        d.session.draft_snapshot(d.scope(), old),
        Err(DraftError::Conflict { .. })
    ));
    let mut wrong = d.request(DraftAction::Cancel { expected: current });
    wrong.session_id = HistorySessionId::new(71).unwrap();
    assert_eq!(
        d.session.edit_draft(wrong),
        Err(DraftError::Request(CommandError::WrongSession))
    );
    assert!(
        d.session
            .draft_info(HistorySessionId::new(71).unwrap())
            .is_err()
    );
    let mut version = d.request(DraftAction::Cancel { expected: current });
    version.api_version -= 1;
    assert!(matches!(
        d.session.edit_draft(version),
        Err(DraftError::Request(CommandError::UnsupportedVersion(_)))
    ));
    assert_eq!(d.session.snapshot(), before);
    d.apply(DraftAction::Cancel { expected: current });
    let new = d.begin();
    assert_ne!(new.id, old.id);
    let closed = d.request(DraftAction::Commit { expected: current });
    assert_eq!(
        d.session.edit_draft(closed),
        Err(DraftError::Conflict {
            expected: current,
            actual: new
        })
    );
}

#[test]
fn shared_receipts_replay_begin_update_commit_cancel_without_repeating_side_effects() {
    let mut d = Driver::new();
    let begin = d.request(DraftAction::Begin {
        expected: d.session.project_info().state,
    });
    let begun = d.session.edit_draft(begin.clone()).unwrap();
    let update = d.request(DraftAction::Replace {
        expected: d.state(),
        commands: vec![insert("done")],
    });
    let updated = d.session.edit_draft(update.clone()).unwrap();
    let commit = d.request(DraftAction::Commit {
        expected: d.state(),
    });
    let committed = d.session.edit_draft(commit.clone()).unwrap();
    let before = d.session.snapshot();
    assert_eq!(d.session.edit_draft(begin), Ok(begun));
    assert_eq!(d.session.edit_draft(update), Ok(updated));
    assert_eq!(d.session.edit_draft(commit.clone()), Ok(committed));
    assert_eq!(d.session.snapshot(), before);
    assert_eq!(d.session.history_len(), 2);
    assert!(d.session.draft_info(d.scope()).unwrap().is_none());
    let mut altered = commit.clone();
    altered.action = DraftAction::Begin {
        expected: before.state,
    };
    assert_eq!(
        d.session.edit_draft(altered),
        Err(DraftError::Request(CommandError::RequestIdReused))
    );
    let mut command = d.command_request(vec![]);
    command.request_id = commit.request_id;
    assert_eq!(
        d.session.submit(command.clone()),
        Err(CommandError::RequestIdReused)
    );
    command.request_id = RequestId::new(500).unwrap();
    d.session.submit(command.clone()).unwrap();
    let mut request = d.request(DraftAction::Begin {
        expected: before.state,
    });
    request.request_id = command.request_id;
    assert_eq!(
        d.session.edit_draft(request),
        Err(DraftError::Request(CommandError::RequestIdReused))
    );
    d.begin();
    let cancel = d.request(DraftAction::Cancel {
        expected: d.state(),
    });
    let cancelled = d.session.edit_draft(cancel.clone()).unwrap();
    d.begin();
    let current = d.state();
    assert_eq!(d.session.edit_draft(cancel), Ok(cancelled));
    assert_eq!(d.state(), current);
}

#[test]
fn identical_replacement_preserves_draft_revision_but_new_payload_advances_it() {
    let mut d = Driver::new();
    d.begin();
    let commands = vec![insert("same")];
    let state = d.replace(commands.clone());
    assert_eq!(d.replace(commands), state);
    let changed = d.replace(vec![
        insert("same"),
        insert("removed"),
        Command::RemoveConfigRecord { index: 0 },
    ]);
    assert_eq!(changed.revision, state.revision + 1);
    assert_eq!(
        d.session
            .draft_snapshot(d.scope(), changed)
            .unwrap()
            .project
            .configuration()
            .to_bytes(),
        b"same"
    );
}

#[test]
fn reserved_receipt_allows_cancel_at_capacity_and_never_evicts_receipts() {
    let mut d = Driver {
        session: ProjectSession::with_resources(
            7,
            70,
            Box::new(TestStore::default()),
            SessionLimits {
                retained_requests: 3,
                ..Default::default()
            },
        )
        .unwrap(),
        next: 1,
    };
    d.begin();
    d.replace(vec![insert("last candidate")]);
    let commit = d.request(DraftAction::Commit {
        expected: d.state(),
    });
    assert_eq!(
        d.session.edit_draft(commit),
        Err(DraftError::Request(CommandError::ResourceLimit(
            "retained_requests"
        )))
    );
    let cancel = d.request(DraftAction::Cancel {
        expected: d.state(),
    });
    d.session.edit_draft(cancel.clone()).unwrap();
    d.session.edit_draft(cancel).unwrap();
    assert_eq!(d.session.history_len(), 1);
    assert!(bytes(&d).is_empty());
    assert_eq!(
        d.session
            .history_statistics(d.scope())
            .unwrap()
            .retained_requests,
        3
    );
}

#[test]
fn missing_draft_oversized_payload_and_insufficient_start_capacity_are_safe() {
    let mut d = Driver::new();
    let before = d.session.snapshot();
    let fake = DraftState {
        id: DraftId::new(1).unwrap(),
        revision: 0,
        base: before.state,
    };
    let request = d.request(DraftAction::Commit { expected: fake });
    assert_eq!(
        d.session.edit_draft(request),
        Err(DraftError::NoActiveDraft)
    );
    d.begin();
    let state = d.state();
    let request = d.request(DraftAction::Replace {
        expected: state,
        commands: vec![insert("too many"); 1025],
    });
    assert_eq!(
        d.session.edit_draft(request),
        Err(DraftError::Request(CommandError::ResourceLimit(
            "commands_per_request"
        )))
    );
    assert_eq!(d.state(), state);
    d.apply(DraftAction::Cancel { expected: state });
    assert_eq!(d.session.snapshot(), before);
    assert_eq!(d.session.history_len(), 1);
    let mut small = Driver {
        session: ProjectSession::with_resources(
            7,
            71,
            Box::new(TestStore::default()),
            SessionLimits {
                retained_requests: 2,
                ..Default::default()
            },
        )
        .unwrap(),
        next: 1,
    };
    let request = small.request(DraftAction::Begin {
        expected: small.session.project_info().state,
    });
    assert_eq!(
        small.session.edit_draft(request),
        Err(DraftError::Request(CommandError::ResourceLimit(
            "retained_requests"
        )))
    );
    assert!(small.session.draft_info(small.scope()).unwrap().is_none());
}

#[derive(Default)]
struct TestStore {
    live: Arc<AtomicUsize>,
    calls: Arc<AtomicUsize>,
    fail_two: Arc<AtomicBool>,
}
struct Lease {
    blob: BlobRef,
    live: Arc<AtomicUsize>,
    _bytes: Vec<u8>,
}
impl BlobLease for Lease {
    fn reference(&self) -> BlobRef {
        self.blob
    }
}
impl Drop for Lease {
    fn drop(&mut self) {
        self.live.fetch_sub(1, Ordering::SeqCst);
    }
}
impl ResourceStore for TestStore {
    fn pin(&self, blob: BlobRef) -> Result<Arc<dyn BlobLease>, ResourceError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        if self.fail_two.load(Ordering::SeqCst) && blob == BlobRef::new([2; 32], 4) {
            return Err(ResourceError::Unavailable);
        }
        self.live.fetch_add(1, Ordering::SeqCst);
        Ok(Arc::new(Lease {
            blob,
            live: self.live.clone(),
            _bytes: vec![0; 4],
        }))
    }
}
fn asset(n: u8) -> Command {
    Command::RegisterAsset {
        asset: Asset::new(
            AssetId::new(n as u128).unwrap(),
            format!("self authored {n}"),
            BlobRef::new([n; 32], 4),
            AssetKind::RasterImage(PixelSize::new(1, 1).unwrap()),
        )
        .unwrap(),
    }
}

#[test]
fn draft_resource_replacement_cancel_and_commit_have_distinct_lifetimes() {
    let store = TestStore::default();
    let live = store.live.clone();
    let calls = store.calls.clone();
    let mut d = Driver {
        session: ProjectSession::with_resources(7, 70, Box::new(store), SessionLimits::default())
            .unwrap(),
        next: 1,
    };
    d.begin();
    d.replace(vec![asset(1)]);
    assert_eq!(live.load(Ordering::SeqCst), 1);
    d.replace(vec![asset(2)]);
    assert_eq!(live.load(Ordering::SeqCst), 1);
    d.apply(DraftAction::Cancel {
        expected: d.state(),
    });
    assert_eq!(live.load(Ordering::SeqCst), 0);
    assert_eq!(
        d.session
            .history_statistics(d.scope())
            .unwrap()
            .retained_blobs,
        0
    );
    d.begin();
    d.replace(vec![asset(1)]);
    let count = calls.load(Ordering::SeqCst);
    d.apply(DraftAction::Commit {
        expected: d.state(),
    });
    assert_eq!(calls.load(Ordering::SeqCst), count);
    assert_eq!(live.load(Ordering::SeqCst), 1);
    let remove = d.command_request(vec![Command::RemoveAsset {
        asset: AssetId::new(1).unwrap(),
    }]);
    d.session.submit(remove).unwrap();
    assert_eq!(live.load(Ordering::SeqCst), 1);
    drop(d);
    assert_eq!(live.load(Ordering::SeqCst), 0);
}

#[test]
fn resource_failure_releases_new_leases_and_keeps_previous_draft_and_project() {
    let store = TestStore::default();
    let live = store.live.clone();
    let fail = store.fail_two.clone();
    fail.store(true, Ordering::SeqCst);
    let mut d = Driver {
        session: ProjectSession::with_resources(7, 70, Box::new(store), SessionLimits::default())
            .unwrap(),
        next: 1,
    };
    let before = d.session.snapshot();
    let state = d.begin();
    let request = d.request(DraftAction::Replace {
        expected: state,
        commands: vec![asset(1), asset(2)],
    });
    assert!(matches!(
        d.session.edit_draft(request.clone()),
        Err(DraftError::Request(CommandError::Resource { .. }))
    ));
    assert_eq!(live.load(Ordering::SeqCst), 0);
    assert_eq!(d.state(), state);
    assert_eq!(d.session.snapshot(), before);
    fail.store(false, Ordering::SeqCst);
    assert!(d.session.edit_draft(request).is_err());
    d.replace(vec![asset(1), asset(2)]);
    assert_eq!(live.load(Ordering::SeqCst), 2);
    d.apply(DraftAction::Cancel {
        expected: d.state(),
    });
    assert_eq!(live.load(Ordering::SeqCst), 0);
    assert_eq!(d.session.history_len(), 1);
}

#[test]
fn draft_commit_at_old_node_preserves_other_branch_content() {
    let mut d = Driver::new();
    let a = d.command_request(vec![insert("A")]);
    let a = d.session.submit(a).unwrap().state.head;
    let b = d.command_request(vec![insert("B")]);
    let b = d.session.submit(b).unwrap().state.head;
    let old = d.session.history_snapshot(d.scope(), b).unwrap();
    let info = d.session.project_info();
    d.session
        .navigate(HistoryRequest {
            api_version: API_VERSION,
            session_id: d.scope(),
            request_id: RequestId::new(700).unwrap(),
            expected: info.state,
            action: HistoryAction::Checkout { node: a },
        })
        .unwrap();
    d.begin();
    d.replace(vec![insert("fork")]);
    d.apply(DraftAction::Commit {
        expected: d.state(),
    });
    assert_eq!(d.session.history_snapshot(d.scope(), b).unwrap(), old);
    assert_eq!(
        d.session
            .history_children(d.scope(), a, 0, 10)
            .unwrap()
            .entries
            .len(),
        2
    );
}
