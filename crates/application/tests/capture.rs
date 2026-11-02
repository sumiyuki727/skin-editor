use skin_editor_application::{values::*, *};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

struct Ids {
    next: u128,
    calls: usize,
    stop: Option<(usize, CaptureCancellation)>,
}
impl Ids {
    fn new() -> Self {
        Self {
            next: 1000,
            calls: 0,
            stop: None,
        }
    }
}
impl HistoryIdentitySource for Ids {
    fn next_identity(&mut self) -> Result<u128, IdentityError> {
        self.calls += 1;
        if let Some((n, c)) = &self.stop
            && *n == self.calls
        {
            c.cancel();
        }
        let id = self.next;
        self.next += 1;
        Ok(id)
    }
}
struct Driver {
    session: ProjectSession,
    request: u128,
}
impl Driver {
    fn new() -> Self {
        Self {
            session: ProjectSession::new(7, 70).unwrap(),
            request: 0,
        }
    }
    fn edit(&mut self, commands: Vec<Command>) -> CommandResult {
        self.request += 1;
        let i = self.session.project_info();
        self.session
            .submit(CommandRequest {
                api_version: API_VERSION,
                session_id: i.session_id,
                request_id: RequestId::new(self.request).unwrap(),
                expected: i.state,
                commands,
            })
            .unwrap()
    }
    fn set(&mut self, text: &str) -> NodeId {
        let command = if self
            .session
            .snapshot()
            .project
            .configuration()
            .records()
            .is_empty()
        {
            Command::InsertConfigRecord {
                index: 0,
                record: record(text),
            }
        } else {
            Command::SetConfigRecord {
                index: 0,
                record: record(text),
            }
        };
        self.edit(vec![command]).state.head
    }
    fn nav(&mut self, action: HistoryAction) {
        self.request += 1;
        let i = self.session.project_info();
        self.session
            .navigate(HistoryRequest {
                api_version: API_VERSION,
                session_id: i.session_id,
                request_id: RequestId::new(self.request).unwrap(),
                expected: i.state,
                action,
            })
            .unwrap();
    }
    fn capture(&mut self, ids: &mut Ids) -> HistoryCapture {
        let request = request(&self.session);
        self.session.capture_history(request, ids).unwrap()
    }
}
fn record(s: &str) -> ConfigRecord {
    ConfigRecord::new(s.as_bytes().to_vec(), ConfigInterpretation::Opaque).unwrap()
}
fn request(s: &ProjectSession) -> CaptureRequest {
    let i = s.project_info();
    CaptureRequest {
        version: HISTORY_CAPTURE_VERSION,
        session_id: i.session_id,
        expected: i.state,
        limits: CaptureLimits::default(),
        cancellation: CaptureCancellation::default(),
    }
}
fn drain(c: &mut HistoryCapture) -> Vec<CapturedNode> {
    let mut nodes = Vec::new();
    loop {
        match c.poll(1).unwrap() {
            CapturePoll::Node(n) => nodes.push(*n),
            CapturePoll::Pending => (),
            CapturePoll::Complete => break,
        }
    }
    nodes
}

#[test]
fn fixed_capture_keeps_every_branch_navigation_and_old_states_during_live_edits() {
    let mut d = Driver::new();
    let mut ids = Ids::new();
    let a = d.set("A");
    let b = d.set("B");
    d.nav(HistoryAction::Checkout { node: a });
    let c = d.set("C");
    d.nav(HistoryAction::Undo);
    let before = d.session.snapshot();
    let expected: BTreeMap<_, _> = (1..=4)
        .map(|n| {
            let id = NodeId::new(n).unwrap();
            (
                n,
                d.session
                    .history_snapshot(before.session_id, id)
                    .unwrap()
                    .project,
            )
        })
        .collect();
    let mut capture = d.capture(&mut ids);
    let snapshot = capture.snapshot().unwrap();
    assert!(!snapshot.is_complete());
    let m = snapshot.manifest().clone();
    assert_eq!(d.session.snapshot(), before);
    assert_eq!(m.nodes.len(), 4);
    assert_eq!(m.source.state, before.state);
    let pa = m.nodes.iter().find(|n| n.alias == a.get()).unwrap().id;
    let pc = m.nodes.iter().find(|n| n.alias == c.get()).unwrap().id;
    assert_eq!(m.redo_preferences, vec![(pa, pc)]);
    assert_eq!(m.nodes.iter().filter(|n| n.parent == Some(pa)).count(), 2);
    d.set("new live branch");
    d.nav(HistoryAction::Checkout { node: b });
    let live = d.session.snapshot();
    let captured = drain(&mut capture);
    assert!(snapshot.is_complete());
    for n in &captured {
        assert_eq!(n.project(), &expected[&n.info().alias]);
    }
    assert_eq!(captured.len(), 4);
    assert_eq!(d.session.snapshot(), live);
    assert_eq!(snapshot.manifest(), &m);
    assert_eq!(capture.statistics().forward_edges, 3);
    assert_eq!(capture.statistics().reverse_edges, 3);
    let calls = ids.calls;
    assert!(matches!(
        d.session.capture_history(request(&d.session), &mut ids),
        Err(CaptureError::InFlight)
    ));
    drop(captured);
    drop(snapshot);
    let mut second = d.capture(&mut ids);
    let updated = second.snapshot().unwrap();
    for old in &m.nodes {
        assert_eq!(
            updated
                .manifest()
                .nodes
                .iter()
                .find(|n| n.alias == old.alias)
                .unwrap()
                .id,
            old.id
        );
    }
    assert_eq!(updated.manifest().identity, m.identity);
    assert_eq!(ids.calls, calls + 1);
    second.cancel();
    drop(updated);
    assert_eq!(d.session.history_len(), 5);
}

#[test]
fn admissions_include_transient_and_removed_objects_without_reusing_ids() {
    let mut d = Driver::new();
    let mut ids = Ids::new();
    let doc = DocumentId::new(9).unwrap();
    let node = d
        .edit(vec![
            Command::CreateDocument {
                document: ElementDocument::new(
                    doc,
                    PixelSize::new(2, 2).unwrap(),
                    LayerTree::default(),
                ),
            },
            Command::RemoveDocument { document: doc },
            Command::InsertConfigRecord {
                index: 0,
                record: record("effective"),
            },
        ])
        .state
        .head;
    let capture = d.capture(&mut ids);
    let s = capture.snapshot().unwrap();
    assert_eq!(s.manifest().admissions.len(), 1);
    let first = s
        .manifest()
        .nodes
        .iter()
        .find(|n| n.alias == node.get())
        .unwrap();
    assert_eq!(
        s.manifest().admissions[0],
        CapturedAdmission {
            object: ObjectId::Document(doc),
            first_node: first.id
        }
    );
    assert_eq!(
        first.operation.steps,
        vec!["document.create", "document.remove", "config.insert"]
    );
    assert_eq!(first.operation.origin, OperationOrigin::Command);
    drop(capture);
    drop(s);
    let i = d.session.project_info();
    let error = d
        .session
        .submit(CommandRequest {
            api_version: API_VERSION,
            session_id: i.session_id,
            request_id: RequestId::new(99).unwrap(),
            expected: i.state,
            commands: vec![Command::CreateDocument {
                document: ElementDocument::new(
                    doc,
                    PixelSize::new(1, 1).unwrap(),
                    LayerTree::default(),
                ),
            }],
        })
        .unwrap_err();
    assert!(matches!(
        error,
        CommandError::Command {
            reason: CommandFailure::ReusedIdentity(_),
            ..
        }
    ));
    assert_eq!(d.session.history_len(), 2);
}

#[test]
fn cancellation_and_all_start_limits_leave_history_and_admission_slot_usable() {
    let mut d = Driver::new();
    d.set("A");
    let mut ids = Ids::new();
    let before = d.session.snapshot();
    for limits in [
        CaptureLimits {
            nodes: 1,
            ..Default::default()
        },
        CaptureLimits {
            retained_metadata_bytes: 1,
            ..Default::default()
        },
        CaptureLimits {
            nodes: 0,
            ..Default::default()
        },
    ] {
        let mut r = request(&d.session);
        r.limits = limits;
        assert!(d.session.capture_history(r, &mut ids).is_err());
    }
    assert_eq!(ids.calls, 0);
    let r = request(&d.session);
    r.cancellation.cancel();
    assert!(matches!(
        d.session.capture_history(r, &mut ids),
        Err(CaptureError::Cancelled)
    ));
    let r = request(&d.session);
    let token = r.cancellation.clone();
    ids.stop = Some((2, token));
    assert!(matches!(
        d.session.capture_history(r, &mut ids),
        Err(CaptureError::Cancelled)
    ));
    ids.stop = None;
    let mut c = d.capture(&mut ids);
    assert_eq!(ids.calls, 7); // failed candidates never published
    c.cancel();
    c.cancel();
    assert!(matches!(c.poll(1), Err(CaptureError::Cancelled)));
    let calls = ids.calls;
    let mut next = d.capture(&mut ids);
    assert_eq!(ids.calls, calls);
    next.cancel();
    assert_eq!(d.session.snapshot(), before);
    assert_eq!(d.session.history_len(), 2);
}

#[test]
fn identity_zero_collision_and_unavailability_are_atomic_and_do_not_change_old_ids() {
    struct Fixed(Result<u128, IdentityError>);
    impl HistoryIdentitySource for Fixed {
        fn next_identity(&mut self) -> Result<u128, IdentityError> {
            self.0
        }
    }
    let mut d = Driver::new();
    d.set("A");
    for (value, error) in [
        (Ok(0), CaptureError::InvalidIdentity),
        (Ok(22), CaptureError::IdentityCollision),
        (
            Err(IdentityError::Unavailable),
            CaptureError::IdentityUnavailable,
        ),
    ] {
        assert_eq!(
            d.session
                .capture_history(request(&d.session), &mut Fixed(value))
                .unwrap_err(),
            error
        );
    }
    let mut ids = Ids::new();
    let mut old = d.capture(&mut ids);
    let manifest = old.snapshot().unwrap().manifest().clone();
    old.cancel();
    d.set("B");
    let reused = manifest.nodes[0].id.get();
    assert_eq!(
        d.session
            .capture_history(request(&d.session), &mut Fixed(Ok(reused)))
            .unwrap_err(),
        CaptureError::IdentityCollision
    );
    let next = d.capture(&mut ids);
    assert_eq!(
        next.snapshot().unwrap().manifest().nodes[0].id,
        manifest.nodes[0].id
    );
    assert_eq!(d.session.history_len(), 3);
}

#[test]
fn poll_failure_releases_worker_and_external_cancellation_needs_no_receipt_slot() {
    let mut d = Driver::new();
    d.set("A");
    let mut ids = Ids::new();
    let mut r = request(&d.session);
    r.limits.total_state_bytes = 256;
    let mut capture = d.session.capture_history(r, &mut ids).unwrap();
    drop(capture.poll(1).unwrap());
    assert!(matches!(
        capture.poll(1),
        Err(CaptureError::ResourceLimit("state_bytes"))
    ));
    let mut r = request(&d.session);
    let token = r.cancellation.clone();
    r.limits.state_bytes = 1;
    let mut capture = d.session.capture_history(r, &mut ids).unwrap();
    assert!(matches!(
        capture.poll(1),
        Err(CaptureError::ResourceLimit("state_bytes"))
    ));
    let mut capture = d.capture(&mut ids);
    let output = capture.poll(1).unwrap();
    drop(output);
    token.cancel(); // belongs to the earlier capture, cannot cancel a new one
    assert!(matches!(capture.poll(1), Ok(CapturePoll::Node(_))));
    capture.cancel();
    let r = request(&d.session);
    let token = r.cancellation.clone();
    let mut capture = d.session.capture_history(r, &mut ids).unwrap();
    token.cancel();
    assert!(matches!(capture.poll(1), Err(CaptureError::Cancelled)));
    let mut capture = d.capture(&mut ids);
    assert!(matches!(
        capture.poll(0),
        Err(CaptureError::InvalidWorkLimit)
    ));
    let mut capture = d.capture(&mut ids);
    capture.cancel();
    assert_eq!(d.session.history_len(), 2);
}

#[test]
fn stale_source_busy_and_no_change_never_create_nodes_or_reserve_identities() {
    let mut d = Driver::new();
    let mut ids = Ids::new();
    let stale = request(&d.session);
    d.set("A");
    assert!(matches!(
        d.session.capture_history(stale, &mut ids),
        Err(CaptureError::Conflict { .. })
    ));
    let mut r = request(&d.session);
    r.session_id = HistorySessionId::new(999).unwrap();
    assert!(matches!(
        d.session.capture_history(r, &mut ids),
        Err(CaptureError::WrongSession)
    ));
    let mut r = request(&d.session);
    r.version += 1;
    assert!(matches!(
        d.session.capture_history(r, &mut ids),
        Err(CaptureError::UnsupportedVersion)
    ));
    let info = d.session.project_info();
    let begun = d
        .session
        .edit_draft(DraftRequest {
            api_version: API_VERSION,
            session_id: info.session_id,
            request_id: RequestId::new(10).unwrap(),
            action: DraftAction::Begin {
                expected: info.state,
            },
        })
        .unwrap();
    let DraftOutcome::Active(active) = begun.outcome else {
        panic!()
    };
    assert!(matches!(
        d.session.capture_history(request(&d.session), &mut ids),
        Err(CaptureError::Busy(_))
    ));
    assert_eq!(ids.calls, 0);
    d.session
        .edit_draft(DraftRequest {
            api_version: API_VERSION,
            session_id: info.session_id,
            request_id: RequestId::new(11).unwrap(),
            action: DraftAction::Cancel {
                expected: active.state,
            },
        })
        .unwrap();
    assert_eq!(d.set("A"), info.state.head);
    let capture = d.capture(&mut ids);
    assert_eq!(capture.snapshot().unwrap().manifest().nodes.len(), 2);
}

#[test]
fn ten_thousand_deep_nodes_use_two_edge_passes_and_bounded_addresses() {
    fn send<T: Send>() {}
    send::<HistoryCapture>();
    send::<CaptureSnapshot>();
    let mut d = Driver::new();
    let mut ids = Ids::new();
    for i in 0..10000 {
        d.set(&format!("step{i}"));
    }
    let mut capture = d.capture(&mut ids);
    let mut seen = BTreeSet::new();
    let mut pending = 0;
    loop {
        match capture.poll(7).unwrap() {
            CapturePoll::Node(n) => {
                assert!(seen.insert(n.info().id));
                assert!(n.info().id.short_key().len() <= 26);
                assert_eq!(
                    PersistentNodeId::parse_short_key(&n.info().id.short_key()),
                    Some(n.info().id)
                );
            }
            CapturePoll::Pending => pending += 1,
            CapturePoll::Complete => break,
        }
    }
    assert_eq!(seen.len(), 10001);
    let stat = capture.statistics();
    assert_eq!(stat.forward_edges, 10000);
    assert_eq!(stat.reverse_edges, 10000);
    assert!(pending > 1000);
    assert!(stat.retained_metadata_bytes < CaptureLimits::default().retained_metadata_bytes);
    assert_eq!(d.session.history_len(), 10001);
}

#[test]
fn full_width_permanent_keys_are_canonical_and_not_local_aliases() {
    for n in [1, 36, u64::MAX as u128, u128::MAX] {
        let id = PersistentNodeId::new(n).unwrap();
        let key = id.short_key();
        assert!(key.len() <= 26);
        assert_eq!(PersistentNodeId::parse_short_key(&key), Some(id));
    }
    for bad in ["k0", "k01", "kA", "n1", "k", "kzzzzzzzzzzzzzzzzzzzzzzzzz"] {
        assert_eq!(PersistentNodeId::parse_short_key(bad), None);
    }
    let reference = PersistentNodeReference {
        project: ProjectId::new(u128::MAX).unwrap(),
        history: PersistentHistoryId::new(70).unwrap(),
        node: PersistentNodeId::new(u128::MAX).unwrap(),
    };
    let text = reference.text();
    assert_eq!(text.len(), 102);
    assert_eq!(PersistentNodeReference::parse(&text), Some(reference));
    assert_eq!(PersistentNodeReference::parse(&text.to_uppercase()), None);
    assert_eq!(
        PersistentNodeReference::parse(&text.replace(
            "ffffffffffffffffffffffffffffffff",
            "00000000000000000000000000000000"
        )),
        None
    );
}

#[test]
fn independent_lease_ownership_survives_session_drop_and_waits_for_last_output() {
    struct Pin {
        blob: BlobRef,
        drops: Arc<AtomicUsize>,
    }
    impl BlobLease for Pin {
        fn reference(&self) -> BlobRef {
            self.blob
        }
    }
    impl Drop for Pin {
        fn drop(&mut self) {
            self.drops.fetch_add(1, Ordering::SeqCst);
        }
    }
    struct Store {
        pin: Arc<Pin>,
    }
    impl ResourceStore for Store {
        fn pin(&self, _: BlobRef) -> Result<Arc<dyn BlobLease>, ResourceError> {
            Ok(self.pin.clone())
        }
    }
    let drops = Arc::new(AtomicUsize::new(0));
    let blob = BlobRef::new([11; 32], 123);
    let pin = Arc::new(Pin {
        blob,
        drops: drops.clone(),
    });
    let weak = Arc::downgrade(&pin);
    let mut d = Driver {
        session: ProjectSession::with_resources(
            7,
            70,
            Box::new(Store { pin }),
            SessionLimits::default(),
        )
        .unwrap(),
        request: 0,
    };
    d.edit(vec![Command::RegisterAsset {
        asset: Asset::new(
            AssetId::new(1).unwrap(),
            "opaque".into(),
            blob,
            AssetKind::Opaque,
        )
        .unwrap(),
    }]);
    let old = d.session.project_info().state.head;
    d.edit(vec![Command::RemoveAsset {
        asset: AssetId::new(1).unwrap(),
    }]);
    let mut ids = Ids::new();
    let mut r = request(&d.session);
    r.limits.original_bytes = 122;
    assert!(matches!(
        d.session.capture_history(r, &mut ids),
        Err(CaptureError::ResourceLimit("original_bytes"))
    ));
    let mut capture = d.capture(&mut ids);
    let snapshot = capture.snapshot().unwrap();
    assert_eq!(snapshot.manifest().originals, vec![blob]);
    drop(d);
    let output = loop {
        if let CapturePoll::Node(n) = capture.poll(1).unwrap()
            && n.info().alias == old.get()
        {
            break n;
        }
    };
    capture.cancel();
    drop(capture);
    drop(snapshot);
    assert!(weak.upgrade().is_some());
    assert_eq!(drops.load(Ordering::SeqCst), 0);
    drop(output);
    assert!(weak.upgrade().is_none());
    assert_eq!(drops.load(Ordering::SeqCst), 1);
}

#[test]
fn draft_capture_records_only_the_final_committed_operation_and_all_reserved_objects() {
    let mut d = Driver::new();
    let mut ids = Ids::new();
    let i = d.session.project_info();
    let begin = d
        .session
        .edit_draft(DraftRequest {
            api_version: API_VERSION,
            session_id: i.session_id,
            request_id: RequestId::new(1).unwrap(),
            action: DraftAction::Begin { expected: i.state },
        })
        .unwrap();
    let DraftOutcome::Active(first) = begin.outcome else {
        panic!()
    };
    let replace = d
        .session
        .edit_draft(DraftRequest {
            api_version: API_VERSION,
            session_id: i.session_id,
            request_id: RequestId::new(2).unwrap(),
            action: DraftAction::Replace {
                expected: first.state,
                commands: vec![Command::InsertConfigRecord {
                    index: 0,
                    record: record("final"),
                }],
            },
        })
        .unwrap();
    let DraftOutcome::Active(last) = replace.outcome else {
        panic!()
    };
    assert!(matches!(
        d.session.capture_history(request(&d.session), &mut ids),
        Err(CaptureError::Busy(_))
    ));
    d.session
        .edit_draft(DraftRequest {
            api_version: API_VERSION,
            session_id: i.session_id,
            request_id: RequestId::new(3).unwrap(),
            action: DraftAction::Commit {
                expected: last.state,
            },
        })
        .unwrap();
    let mut capture = d.capture(&mut ids);
    let states = drain(&mut capture);
    assert_eq!(states.len(), 2);
    assert_eq!(states[1].info().operation.origin, OperationOrigin::Draft);
    assert_eq!(states[1].info().operation.steps, vec!["config.insert"]);
    assert_eq!(states[1].project().configuration().to_bytes(), b"final");
}

#[test]
fn all_object_ids_are_admitted_before_capture_clones_or_identity_allocation() {
    let mut d = Driver::new();
    let mut ids = Ids::new();
    for id in [1, 2] {
        d.edit(vec![Command::CreateDocument {
            document: ElementDocument::new(
                DocumentId::new(id).unwrap(),
                PixelSize::new(1, 1).unwrap(),
                LayerTree::default(),
            ),
        }]);
    }
    let mut r = request(&d.session);
    r.limits.object_ids = 1;
    assert!(matches!(
        d.session.capture_history(r, &mut ids),
        Err(CaptureError::ResourceLimit("object_ids"))
    ));
    assert_eq!(ids.calls, 0);
    let mut capture = d.capture(&mut ids);
    capture.cancel();
}

#[test]
fn capture_cancel_is_available_when_the_command_receipt_capacity_is_full() {
    let mut d = Driver {
        session: ProjectSession::with_resources(
            7,
            70,
            Box::new(Empty),
            SessionLimits {
                retained_requests: 1,
                ..Default::default()
            },
        )
        .unwrap(),
        request: 0,
    };
    struct Empty;
    impl ResourceStore for Empty {
        fn pin(&self, _: BlobRef) -> Result<Arc<dyn BlobLease>, ResourceError> {
            Err(ResourceError::Missing)
        }
    }
    d.set("A");
    let before = d.session.snapshot();
    let mut ids = Ids::new();
    for _ in 0..3 {
        let mut capture = d.capture(&mut ids);
        capture.cancel();
    }
    assert_eq!(d.session.snapshot(), before);
    assert_eq!(
        d.session
            .history_statistics(before.session_id)
            .unwrap()
            .retained_requests,
        1
    );
}
