use skin_editor_application::{values::*, *};
use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

struct Empty;
impl ResourceStore for Empty {
    fn pin(&self, _: BlobRef) -> Result<Arc<dyn BlobLease>, ResourceError> {
        Err(ResourceError::Missing)
    }
}
struct Ids {
    next: u128,
    calls: usize,
}
impl Default for Ids {
    fn default() -> Self {
        Self {
            next: 1000,
            calls: 0,
        }
    }
}
impl HistoryIdentitySource for Ids {
    fn next_identity(&mut self) -> Result<u128, IdentityError> {
        self.calls += 1;
        let result = self.next;
        self.next += 1;
        Ok(result)
    }
}
fn limits() -> SessionLimits {
    SessionLimits {
        history_nodes: 64,
        retained_requests: 128,
        commands_per_request: 16,
    }
}
fn config(s: &ProjectSession) -> SavePreparationConfig {
    SavePreparationConfig {
        version: SAVE_COORDINATION_VERSION,
        binding: SaveTargetBinding {
            version: SAVE_COORDINATION_VERSION,
            key: SaveBindingKey {
                id: SaveBindingId::new(10).unwrap(),
                generation: 1,
            },
            target: SaveTargetId::new(20).unwrap(),
            project: ProjectId::new(s.project_info().project_id).unwrap(),
            phase: SaveTargetPhase::NewUnpublished,
            history: None,
            expected: ExpectedFileCommit::default(),
        },
        control: SaveControlProfile::default(),
        capture: save_fixture_capture_limits(),
    }
}
fn request(s: &ProjectSession, binding: SaveBindingKey) -> SaveRequest {
    SaveRequest {
        api_version: API_VERSION,
        capture_version: HISTORY_CAPTURE_VERSION,
        key: s
            .save_coordination_status(1)
            .unwrap()
            .next_key
            .unwrap_or(SaveAdmissionKey {
                version: 1,
                session: s.project_info().session_id,
                sequence: 1,
            }),
        source: s.save_source(),
        binding,
        origin: SaveOrigin::Manual,
    }
}
fn raw_request(s: &ProjectSession) -> CaptureRequest {
    CaptureRequest {
        version: HISTORY_CAPTURE_VERSION,
        session_id: s.project_info().session_id,
        expected: s.project_info().state,
        limits: save_fixture_capture_limits(),
        cancellation: CaptureCancellation::default(),
    }
}
fn record(s: &str) -> ConfigRecord {
    ConfigRecord::new(s.as_bytes().to_vec(), ConfigInterpretation::Opaque).unwrap()
}
struct Driver {
    session: ProjectSession,
    next: u128,
}
impl Driver {
    fn new() -> Self {
        Self {
            session: ProjectSession::with_resources(7, 70, Box::new(Empty), limits()).unwrap(),
            next: 1,
        }
    }
    fn edit(&mut self, commands: Vec<Command>) -> NodeId {
        let i = self.session.project_info();
        let request = CommandRequest {
            api_version: API_VERSION,
            session_id: i.session_id,
            request_id: RequestId::new(self.next).unwrap(),
            expected: i.state,
            commands,
        };
        self.next += 1;
        self.session.submit(request).unwrap().state.head
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
        self.edit(vec![command])
    }
    fn nav(&mut self, action: HistoryAction) {
        let i = self.session.project_info();
        self.session
            .navigate(HistoryRequest {
                api_version: API_VERSION,
                session_id: i.session_id,
                request_id: RequestId::new(self.next).unwrap(),
                expected: i.state,
                action,
            })
            .unwrap();
        self.next += 1;
    }
    fn draft(&mut self, action: DraftAction) -> DraftOutcome {
        let result = self
            .session
            .edit_draft(DraftRequest {
                api_version: API_VERSION,
                session_id: self.session.project_info().session_id,
                request_id: RequestId::new(self.next).unwrap(),
                action,
            })
            .unwrap();
        self.next += 1;
        result.outcome
    }
    fn install(&mut self) -> (SavePreparationPort, SaveBindingKey) {
        let config = config(&self.session);
        (
            self.session.install_save_preparation(config).unwrap(),
            config.binding.key,
        )
    }
    fn stats(&self) -> HistoryStatistics {
        self.session
            .history_statistics(self.session.project_info().session_id)
            .unwrap()
    }
    fn active(&self) -> SaveActiveStatus {
        self.session
            .save_coordination_status(1)
            .unwrap()
            .active
            .unwrap()
    }
}
fn drain(work: &mut SavePreparationWork) -> Vec<CapturedNode> {
    let mut nodes = Vec::new();
    loop {
        match work.poll(1).unwrap() {
            CapturePoll::Pending => (),
            CapturePoll::Node(n) => nodes.push(*n),
            CapturePoll::Complete => break,
        }
    }
    nodes
}

#[test]
fn versions_unavailable_and_unsupported_origins_do_not_capture_or_consume_commands() {
    assert_eq!(
        (
            API_VERSION,
            HISTORY_CAPTURE_VERSION,
            SAVE_COORDINATION_VERSION,
            ARCHIVE_PORT_VERSION
        ),
        (7, 3, 1, 1)
    );
    let mut d = Driver::new();
    let mut ids = Ids::default();
    let cfg = config(&d.session);
    let r = request(&d.session, cfg.binding.key);
    let before = d.stats();
    assert_eq!(
        d.session.request_save(r, &mut ids),
        Err(SaveCoordinationError::Unavailable)
    );
    for (api, capture, coord) in [(6, 3, 1), (7, 2, 1), (7, 3, 2)] {
        let mut bad = r;
        bad.api_version = api;
        bad.capture_version = capture;
        bad.key.version = coord;
        assert_eq!(
            d.session.request_save(bad, &mut ids),
            Err(SaveCoordinationError::UnsupportedVersion)
        );
    }
    let _port = d.session.install_save_preparation(cfg).unwrap();
    let mut bad = r;
    bad.origin = SaveOrigin::Automatic;
    assert_eq!(
        d.session.request_save(bad, &mut ids),
        Err(SaveCoordinationError::UnsupportedOrigin)
    );
    bad.origin = SaveOrigin::Exit;
    assert_eq!(
        d.session.request_save(bad, &mut ids),
        Err(SaveCoordinationError::NeedsClose)
    );
    let mut raw = raw_request(&d.session);
    raw.version = 2;
    assert!(matches!(
        d.session.capture_history(raw, &mut ids),
        Err(CaptureError::UnsupportedVersion)
    ));
    let i = d.session.project_info();
    assert!(matches!(
        d.session.submit(CommandRequest {
            api_version: 6,
            session_id: i.session_id,
            request_id: RequestId::new(1).unwrap(),
            expected: i.state,
            commands: vec![]
        }),
        Err(CommandError::UnsupportedVersion(6))
    ));
    assert_eq!(ids.calls, 0);
    assert_eq!(d.stats(), before);
}

#[test]
fn invalid_profiles_and_binding_pairs_reject_before_installation() {
    let mut d = Driver::new();
    for field in 0..8 {
        let mut c = config(&d.session);
        match field {
            0 => c.version += 1,
            1 => c.control.version += 1,
            2 => c.binding.version += 1,
            3 => c.control.available_bytes -= 1,
            4 => c.capture.nodes += 1,
            5 => c.capture.state_bytes = 0,
            6 => c.binding.key.generation = 0,
            _ => c.binding.project = ProjectId::new(99).unwrap(),
        }
        assert!(d.session.install_save_preparation(c).is_err());
        assert_eq!(
            d.session
                .save_coordination_status(1)
                .unwrap()
                .reserved_control_bytes,
            0
        );
    }
    for expected in [
        ExpectedFileCommit {
            commit: Some(CommitId::new(1).unwrap()),
            generation: None,
        },
        ExpectedFileCommit {
            commit: None,
            generation: Some(1),
        },
        ExpectedFileCommit {
            commit: Some(CommitId::new(1).unwrap()),
            generation: Some(0),
        },
    ] {
        let mut c = config(&d.session);
        c.binding.expected = expected;
        assert!(matches!(
            d.session.install_save_preparation(c),
            Err(SaveCoordinationError::InvalidBinding)
        ));
    }
    assert!(SaveTargetId::new(0).is_none());
    assert!(SaveBindingId::new(0).is_none());
    let (_port, _) = d.install();
    assert_eq!(
        d.session
            .save_coordination_status(1)
            .unwrap()
            .reserved_control_bytes,
        512 * 1024
    );
    assert!(matches!(
        d.session.install_save_preparation(config(&d.session)),
        Err(SaveCoordinationError::AlreadyInstalled)
    ));
}

#[test]
fn zero_revision_is_legal_but_wrong_source_and_target_are_not() {
    let mut d = Driver::new();
    let (_port, binding) = d.install();
    let r = request(&d.session, binding);
    assert_eq!(
        (r.source.state.revision, r.source.navigation_revision),
        (0, 0)
    );
    let mut ids = Ids::default();
    for field in 0..5 {
        let mut bad = r;
        match field {
            0 => bad.key.sequence = 0,
            1 => bad.key.session = HistorySessionId::new(99).unwrap(),
            2 => bad.source.state.revision += 1,
            3 => bad.source.navigation_revision += 1,
            _ => bad.binding.generation += 1,
        }
        assert!(d.session.request_save(bad, &mut ids).is_err());
        assert!(
            d.session
                .save_coordination_status(1)
                .unwrap()
                .active
                .is_none()
        );
    }
    assert_eq!(ids.calls, 0);
    let ticket = d.session.request_save(r, &mut ids).unwrap().ticket();
    assert_eq!(ticket.source, r.source);
    assert_eq!(ticket.capture, SaveCaptureId(1));
    assert_eq!(ticket.coverage, SaveCoverageKey(1));
    assert_eq!(ticket.target.expected, ExpectedFileCommit::default());
}

#[test]
fn repeats_match_original_payload_before_live_revision_draft_or_port_liveness() {
    let mut d = Driver::new();
    let (port, binding) = d.install();
    let mut ids = Ids::default();
    let r = request(&d.session, binding);
    let ticket = d.session.request_save(r, &mut ids).unwrap().ticket();
    let calls = ids.calls;
    d.set("q+1");
    d.draft(DraftAction::Begin {
        expected: d.session.project_info().state,
    });
    let draft = d.session.draft_info(r.key.session).unwrap().unwrap();
    let stats = d.stats();
    drop(port);
    assert_eq!(
        d.session.request_save(r, &mut ids),
        Ok(SaveAdmission::Existing(ticket))
    );
    let mut changed = r;
    changed.source = d.session.save_source();
    assert_eq!(
        d.session.request_save(changed, &mut ids),
        Err(SaveCoordinationError::Conflict)
    );
    let mut second = r;
    second.key.sequence += 1;
    assert_eq!(
        d.session.request_save(second, &mut ids),
        Err(SaveCoordinationError::Busy(SaveBusy::SaveOutstanding))
    );
    assert_eq!(
        d.session.acknowledge_save(ticket),
        Err(SaveCoordinationError::NotTerminal)
    );
    assert_eq!(ids.calls, calls);
    assert_eq!(d.stats(), stats);
    assert_eq!(d.session.draft_info(r.key.session).unwrap().unwrap(), draft);
    d.session.cancel_save(ticket).unwrap();
    assert_eq!(
        d.session.acknowledge_save(ticket),
        Ok(SaveAcknowledgement::Released)
    );
    assert_eq!(
        d.session.request_save(r, &mut ids),
        Err(SaveCoordinationError::Stale)
    );
}

#[test]
fn pending_cancel_is_not_saved_and_does_not_require_a_worker_or_new_request_id() {
    let mut d = Driver::new();
    d.set("keep");
    let (port, binding) = d.install();
    let mut ids = Ids::default();
    let r = request(&d.session, binding);
    let before = d.stats();
    let ticket = d.session.request_save(r, &mut ids).unwrap().ticket();
    assert!(matches!(
        d.session.capture_history(raw_request(&d.session), &mut ids),
        Err(CaptureError::SaveOutstanding)
    ));
    d.session.cancel_save(ticket).unwrap();
    d.session.cancel_save(ticket).unwrap();
    assert!(matches!(
        d.active().outcome,
        Some(SavePreparationOutcome::StoppedNotSaved(SaveDiagnostic {
            code: SaveDiagnosticCode::Cancelled,
            ..
        }))
    ));
    assert!(matches!(
        d.session.take_save_preparation(&port, ticket),
        Err(SaveCoordinationError::NotPending)
    ));
    assert_eq!(
        d.session.acknowledge_save(ticket),
        Ok(SaveAcknowledgement::Released)
    );
    assert_eq!(d.stats(), before);
    assert_eq!(
        d.session.request_save(r, &mut ids),
        Err(SaveCoordinationError::Stale)
    );
    let raw = d
        .session
        .capture_history(raw_request(&d.session), &mut ids)
        .unwrap();
    assert_eq!(
        raw.snapshot().unwrap().manifest().archive_metadata.as_ref(),
        &ArchiveMetadata::default()
    );
}

#[test]
fn ack_waits_for_worker_snapshot_and_node_owners_then_old_keys_cannot_release_new_slot() {
    let mut d = Driver::new();
    let (port, binding) = d.install();
    let mut ids = Ids::default();
    let r = request(&d.session, binding);
    let ticket = d.session.request_save(r, &mut ids).unwrap().ticket();
    let mut work = d.session.take_save_preparation(&port, ticket).unwrap();
    let snapshot = work.snapshot().clone();
    let nodes = drain(&mut work);
    assert!(snapshot.is_complete());
    assert_eq!(
        d.active().outcome,
        Some(SavePreparationOutcome::CompleteNotSaved)
    );
    assert_eq!(
        d.session.acknowledge_save(ticket),
        Ok(SaveAcknowledgement::WaitingResources)
    );
    assert!(!d.active().work_released);
    drop(work);
    assert!(d.active().work_released);
    assert_eq!(
        d.session.acknowledge_save(ticket),
        Ok(SaveAcknowledgement::WaitingResources)
    );
    drop(snapshot);
    assert_eq!(
        d.session.acknowledge_save(ticket),
        Ok(SaveAcknowledgement::WaitingResources)
    );
    drop(nodes);
    assert_eq!(
        d.session.acknowledge_save(ticket),
        Ok(SaveAcknowledgement::Released)
    );
    let second = d
        .session
        .request_save(request(&d.session, binding), &mut ids)
        .unwrap()
        .ticket();
    assert_ne!(second.key, ticket.key);
    assert_ne!(second.commit, ticket.commit);
    assert_eq!(
        d.session.cancel_save(ticket),
        Err(SaveCoordinationError::Stale)
    );
    assert_eq!(
        d.session.acknowledge_save(ticket),
        Err(SaveCoordinationError::Stale)
    );
    assert_eq!(d.active().ticket, second);
}

#[test]
fn worker_drop_before_ack_releases_capture_but_keeps_save_outstanding() {
    let mut d = Driver::new();
    let (port, binding) = d.install();
    let mut ids = Ids::default();
    let ticket = d
        .session
        .request_save(request(&d.session, binding), &mut ids)
        .unwrap()
        .ticket();
    let mut work = d.session.take_save_preparation(&port, ticket).unwrap();
    drop(drain(&mut work));
    drop(work);
    // Cancel is an allocation-free control action; a terminal outcome is stable.
    d.session.cancel_save(ticket).unwrap();
    assert!(d.active().capture_consumers_released);
    assert_eq!(
        d.active().outcome,
        Some(SavePreparationOutcome::CompleteNotSaved)
    );
    assert!(matches!(
        d.session.capture_history(raw_request(&d.session), &mut ids),
        Err(CaptureError::SaveOutstanding)
    ));
    assert_eq!(
        d.session
            .request_save(request(&d.session, binding), &mut ids),
        Err(SaveCoordinationError::Busy(SaveBusy::SaveOutstanding))
    );
    assert_eq!(
        d.session.acknowledge_save(ticket),
        Ok(SaveAcknowledgement::Released)
    );
}

#[test]
fn working_cancel_abandon_and_invalid_poll_are_finite_not_saved_results() {
    for mode in 0..3 {
        let mut d = Driver::new();
        let (port, binding) = d.install();
        let mut ids = Ids::default();
        let ticket = d
            .session
            .request_save(request(&d.session, binding), &mut ids)
            .unwrap()
            .ticket();
        let mut work = d.session.take_save_preparation(&port, ticket).unwrap();
        let expected = match mode {
            0 => {
                d.session.cancel_save(ticket).unwrap();
                assert_eq!(
                    d.session.acknowledge_save(ticket),
                    Err(SaveCoordinationError::NotTerminal)
                );
                assert!(matches!(work.poll(1), Err(CaptureError::Cancelled)));
                SaveDiagnosticCode::Cancelled
            }
            1 => SaveDiagnosticCode::Abandoned,
            _ => {
                assert!(matches!(work.poll(0), Err(CaptureError::InvalidWorkLimit)));
                SaveDiagnosticCode::InvalidWorkLimit
            }
        };
        drop(work);
        assert!(
            matches!(d.active().outcome, Some(SavePreparationOutcome::StoppedNotSaved(SaveDiagnostic { code, .. })) if code == expected)
        );
        assert_eq!(
            d.session.acknowledge_save(ticket),
            Ok(SaveAcknowledgement::Released)
        );
        let capture = d
            .session
            .capture_history(raw_request(&d.session), &mut ids)
            .unwrap();
        assert!(
            capture
                .snapshot()
                .unwrap()
                .manifest()
                .archive_metadata
                .last_commit
                .is_none()
        );
    }
}

#[test]
fn wrong_port_ticket_and_session_do_not_change_the_active_task() {
    let mut d = Driver::new();
    let (port, binding) = d.install();
    let mut other = Driver::new();
    let (other_port, _) = other.install();
    let mut ids = Ids::default();
    let ticket = d
        .session
        .request_save(request(&d.session, binding), &mut ids)
        .unwrap()
        .ticket();
    assert!(matches!(
        d.session.take_save_preparation(&other_port, ticket),
        Err(SaveCoordinationError::WrongPort)
    ));
    let before = d.active();
    for field in 0..16 {
        let mut bad = ticket;
        match field {
            0 => bad.key.session = HistorySessionId::new(71).unwrap(),
            1 => bad.key.sequence += 1,
            2 => bad.target.key.generation += 1,
            3 => bad.capture.0 += 1,
            4 => bad.coverage.0 += 1,
            5 => bad.source.navigation_revision += 1,
            6 => bad.commit = CommitId::new(9999).unwrap(),
            7 => bad.origin = SaveOrigin::Exit,
            8 => bad.history.lineage = LineageId::new(9999).unwrap(),
            9 => bad.head = PersistentNodeId::new(9999).unwrap(),
            10 => bad.target.target = SaveTargetId::new(9999).unwrap(),
            11 => bad.target.expected.generation = Some(1),
            12 => bad.capture_version = 2,
            13 => bad.key.version = 2,
            14 => bad.target.version = 2,
            _ => {
                bad.parent_close = Some(CloseAdmissionKey {
                    version: 1,
                    session: ticket.key.session,
                    sequence: 1,
                })
            }
        }
        assert!(d.session.cancel_save(bad).is_err());
        assert!(d.session.acknowledge_save(bad).is_err());
        assert!(d.session.take_save_preparation(&port, bad).is_err());
        assert_eq!(d.active(), before);
    }
    drop(d.session.take_save_preparation(&port, ticket).unwrap());
    assert_eq!(
        d.session.acknowledge_save(ticket),
        Ok(SaveAcknowledgement::Released)
    );
}

#[test]
fn two_runtime_sessions_with_the_same_project_and_head_reject_each_others_controls() {
    let mut first = Driver::new();
    let mut second = Driver {
        session: ProjectSession::with_resources(7, 71, Box::new(Empty), limits()).unwrap(),
        next: 1,
    };
    let (_first_port, first_binding) = first.install();
    let (_second_port, second_binding) = second.install();
    let a = first
        .session
        .request_save(request(&first.session, first_binding), &mut Ids::default())
        .unwrap()
        .ticket();
    let b = second
        .session
        .request_save(
            request(&second.session, second_binding),
            &mut Ids {
                next: 2000,
                calls: 0,
            },
        )
        .unwrap()
        .ticket();
    assert_eq!(a.target.project, b.target.project);
    assert_eq!(a.source.state.head, b.source.state.head);
    assert_eq!(
        second.session.cancel_save(a),
        Err(SaveCoordinationError::WrongSession)
    );
    assert_eq!(
        first.session.acknowledge_save(b),
        Err(SaveCoordinationError::WrongSession)
    );
    assert_eq!(first.active().ticket, a);
    assert_eq!(second.active().ticket, b);
}

#[test]
fn active_draft_and_external_capture_are_busy_without_identity_or_receipt_cost() {
    let mut d = Driver::new();
    let (_port, binding) = d.install();
    let mut ids = Ids::default();
    let mut external = d
        .session
        .capture_history(raw_request(&d.session), &mut ids)
        .unwrap();
    let snapshot = external.snapshot().unwrap();
    external.cancel();
    drop(external);
    let calls = ids.calls;
    assert_eq!(
        d.session
            .request_save(request(&d.session, binding), &mut ids),
        Err(SaveCoordinationError::Busy(SaveBusy::CaptureConsumers))
    );
    assert_eq!(ids.calls, calls);
    drop(snapshot);
    d.draft(DraftAction::Begin {
        expected: d.session.project_info().state,
    });
    let before = d.stats();
    assert_eq!(
        d.session
            .request_save(request(&d.session, binding), &mut ids),
        Err(SaveCoordinationError::Busy(SaveBusy::Draft))
    );
    assert_eq!(d.stats(), before);
    assert_eq!(ids.calls, calls);
}

#[test]
fn capture_limit_failure_is_atomic_and_does_not_consume_the_admission_key() {
    let mut d = Driver::new();
    d.set("A");
    let mut cfg = config(&d.session);
    cfg.capture.nodes = 1;
    let _port = d.session.install_save_preparation(cfg).unwrap();
    let r = request(&d.session, cfg.binding.key);
    let mut ids = Ids::default();
    let snapshot = d.session.snapshot();
    let stats = d.stats();
    assert!(matches!(
        d.session.request_save(r, &mut ids),
        Err(SaveCoordinationError::Capture(SaveDiagnostic {
            code: SaveDiagnosticCode::CaptureLimit,
            numbers: [1, 0, 0, 0]
        }))
    ));
    assert_eq!(ids.calls, 0);
    assert_eq!(d.session.snapshot(), snapshot);
    assert_eq!(d.stats(), stats);
    assert!(
        d.session
            .save_coordination_status(1)
            .unwrap()
            .active
            .is_none()
    );
    assert_eq!(
        d.session.save_coordination_status(1).unwrap().next_key,
        Some(r.key)
    );
    assert!(
        d.session
            .capture_history(raw_request(&d.session), &mut ids)
            .is_ok()
    );
}

#[test]
fn an_unsaved_permanent_map_is_not_mistaken_for_a_published_file() {
    let mut d = Driver::new();
    let mut ids = Ids::default();
    let capture = d
        .session
        .capture_history(raw_request(&d.session), &mut ids)
        .unwrap();
    let identity = capture.snapshot().unwrap().manifest().identity.clone();
    drop(capture);
    let (_port, binding) = d.install();
    let t = d
        .session
        .request_save(request(&d.session, binding), &mut ids)
        .unwrap()
        .ticket();
    assert_eq!(t.history, SaveHistoryScope::from(&identity));
    assert_eq!(t.target.phase, SaveTargetPhase::NewUnpublished);
}

fn restored_fixture(with_receipt: bool) -> (ProjectSession, SaveTargetBinding) {
    let mut d = Driver::new();
    let mut ids = Ids::default();
    d.set("restored opaque bytes\r\n");
    let mut capture = d
        .session
        .capture_history(raw_request(&d.session), &mut ids)
        .unwrap();
    let m = capture.snapshot().unwrap().manifest().clone();
    let history = SaveHistoryScope::from(&m.identity);
    let receipt = ArchiveReceipt {
        commit: CommitId::new(42).unwrap(),
        generation: u64::MAX,
        head: m.head,
        origin: SaveOrigin::Manual,
    };
    let metadata = if with_receipt {
        ArchiveMetadata {
            last_commit: Some(receipt.clone()),
            manual: Some(receipt.clone()),
            protected: Some(receipt.clone()),
            manual_marks: vec![(receipt.commit, receipt.head)],
            ..ArchiveMetadata::default()
        }
    } else {
        ArchiveMetadata::default()
    };
    let mut builder = RestoreBuilder::new(
        RestoreHeader {
            node_count: m.nodes.len(),
            project: m.project_id,
            identity: m.identity,
            root: m.root,
            head: m.head,
            next_alias: m.next_alias,
            next_branch: m.next_branch,
            admissions: m.admissions,
            redo_preferences: m.redo_preferences,
            originals: m.originals,
            metadata: Arc::new(metadata),
        },
        RestoreLimits::default(),
        CaptureCancellation::default(),
    )
    .unwrap();
    loop {
        match capture.poll(1).unwrap() {
            CapturePoll::Pending => (),
            CapturePoll::Complete => break,
            CapturePoll::Node(n) => {
                let i = n.info();
                builder
                    .push(RestoreNode {
                        id: i.id,
                        parent: i.parent,
                        alias: i.alias,
                        coordinate: i.coordinate,
                        depth: i.depth,
                        operation: i.operation.clone(),
                        origin: Arc::new(OriginalNodeRecord {
                            record_version: 2,
                            state: [1; 32],
                            action: b"{}".to_vec(),
                            checksum: [2; 32],
                        }),
                        project: n.project().clone(),
                    })
                    .unwrap();
            }
        }
    }
    let restored = builder
        .finish(&Empty)
        .unwrap()
        .into_session(
            HistorySessionId::new(71).unwrap(),
            Box::new(Empty),
            limits(),
            CaptureCancellation::default(),
        )
        .unwrap();
    let mut binding = config(&restored).binding;
    binding.phase = SaveTargetPhase::ExistingVerified;
    binding.history = Some(history);
    binding.expected = if with_receipt {
        ExpectedFileCommit {
            commit: Some(receipt.commit),
            generation: Some(receipt.generation),
        }
    } else {
        ExpectedFileCommit::default()
    };
    (restored, binding)
}

#[test]
fn restored_history_without_commit_metadata_cannot_be_published_as_new() {
    let (mut restored, _) = restored_fixture(false);
    assert!(matches!(
        restored.install_save_preparation(config(&restored)),
        Err(SaveCoordinationError::InvalidBinding)
    ));
    assert!(
        restored
            .save_coordination_status(1)
            .unwrap()
            .active
            .is_none()
    );
}

#[test]
fn existing_binding_requires_full_restored_scope_and_exact_expected_chain() {
    let (mut restored, binding) = restored_fixture(true);
    assert!(matches!(
        restored.install_save_preparation(config(&restored)),
        Err(SaveCoordinationError::InvalidBinding)
    ));
    for field in 0..4 {
        let mut c = config(&restored);
        c.binding = binding;
        match field {
            0 => c.binding.history.as_mut().unwrap().lineage = LineageId::new(999).unwrap(),
            1 => c.binding.expected.commit = Some(CommitId::new(999).unwrap()),
            2 => c.binding.expected.generation = Some(1),
            _ => c.binding.history = None,
        }
        assert!(matches!(
            restored.install_save_preparation(c),
            Err(SaveCoordinationError::InvalidBinding)
        ));
    }
    let mut c = config(&restored);
    c.binding = binding;
    let port = restored.install_save_preparation(c).unwrap();
    let before = restored.snapshot();
    let t = restored
        .request_save(
            request(&restored, binding.key),
            &mut Ids {
                next: 2000,
                calls: 0,
            },
        )
        .unwrap()
        .ticket();
    assert_eq!(t.history, binding.history.unwrap());
    // The maximum file generation can be inspected; A1 cannot issue a write.
    assert_eq!(t.target.expected.generation, Some(u64::MAX));
    let mut work = restored.take_save_preparation(&port, t).unwrap();
    let metadata = work.snapshot().manifest().archive_metadata.clone();
    drop(drain(&mut work));
    drop(work);
    assert_eq!(
        restored.acknowledge_save(t),
        Ok(SaveAcknowledgement::Released)
    );
    assert_eq!(restored.snapshot(), before);
    assert_eq!(metadata.manual_marks.len(), 1);
    assert_eq!(
        metadata.last_commit.as_ref().unwrap().commit,
        CommitId::new(42).unwrap()
    );
}

#[test]
fn failed_commit_candidate_releases_capture_and_allows_the_same_unconsumed_key() {
    struct Fixed(u128);
    impl HistoryIdentitySource for Fixed {
        fn next_identity(&mut self) -> Result<u128, IdentityError> {
            Ok(self.0)
        }
    }
    let mut d = Driver::new();
    let mut ids = Ids::default();
    let raw = d
        .session
        .capture_history(raw_request(&d.session), &mut ids)
        .unwrap();
    let history_id = raw.snapshot().unwrap().manifest().identity.history.get();
    drop(raw);
    let (_port, binding) = d.install();
    let r = request(&d.session, binding);
    let stats = d.stats();
    let before = d.session.snapshot();
    for candidate in [0, history_id] {
        assert!(d.session.request_save(r, &mut Fixed(candidate)).is_err());
        assert!(
            d.session
                .save_coordination_status(1)
                .unwrap()
                .active
                .is_none()
        );
        assert_eq!(
            d.session.save_coordination_status(1).unwrap().next_key,
            Some(r.key)
        );
        assert_eq!(d.stats(), stats);
        assert_eq!(d.session.snapshot(), before);
    }
    let t = d.session.request_save(r, &mut ids).unwrap().ticket();
    d.session.cancel_save(t).unwrap();
    d.session.acknowledge_save(t).unwrap();
    let second = request(&d.session, binding);
    assert!(matches!(
        d.session.request_save(second, &mut Fixed(t.commit.get())),
        Err(SaveCoordinationError::Capture(SaveDiagnostic {
            code: SaveDiagnosticCode::IdentityCollision,
            ..
        }))
    ));
    assert_eq!(
        d.session.save_coordination_status(1).unwrap().next_key,
        Some(second.key)
    );
    assert!(d.session.request_save(second, &mut ids).is_ok());
}

#[test]
fn ordinary_receipt_exhaustion_does_not_disable_cancel_ack_or_change_old_history() {
    let mut d = Driver {
        session: ProjectSession::with_resources(
            7,
            70,
            Box::new(Empty),
            SessionLimits {
                retained_requests: 2,
                ..limits()
            },
        )
        .unwrap(),
        next: 1,
    };
    d.set("A");
    let (_port, binding) = d.install();
    let mut ids = Ids::default();
    let t = d
        .session
        .request_save(request(&d.session, binding), &mut ids)
        .unwrap()
        .ticket();
    d.set("B");
    let i = d.session.project_info();
    let before = d.session.snapshot();
    assert_eq!(d.stats().retained_requests, 2);
    assert!(matches!(
        d.session.submit(CommandRequest {
            api_version: API_VERSION,
            session_id: i.session_id,
            expected: i.state,
            request_id: RequestId::new(3).unwrap(),
            commands: vec![Command::SetConfigRecord {
                index: 0,
                record: record("C")
            }]
        }),
        Err(CommandError::ResourceLimit("retained_requests"))
    ));
    d.session.cancel_save(t).unwrap();
    assert_eq!(
        d.session.acknowledge_save(t),
        Ok(SaveAcknowledgement::Released)
    );
    assert_eq!(d.session.snapshot(), before);
    assert_eq!(d.stats().retained_requests, 2);
    assert_eq!(d.session.history_len(), 3);
}

#[test]
fn late_state_limit_failure_is_terminal_but_waits_for_actual_worker_drop() {
    let mut d = Driver::new();
    let mut c = config(&d.session);
    c.capture.state_bytes = 1;
    let port = d.session.install_save_preparation(c).unwrap();
    let mut ids = Ids::default();
    let t = d
        .session
        .request_save(request(&d.session, c.binding.key), &mut ids)
        .unwrap()
        .ticket();
    let mut work = d.session.take_save_preparation(&port, t).unwrap();
    assert!(matches!(
        work.poll(1),
        Err(CaptureError::ResourceLimit("state_bytes"))
    ));
    assert!(matches!(
        d.active().outcome,
        Some(SavePreparationOutcome::StoppedNotSaved(SaveDiagnostic {
            code: SaveDiagnosticCode::CaptureLimit,
            ..
        }))
    ));
    assert_eq!(
        d.session.acknowledge_save(t),
        Ok(SaveAcknowledgement::WaitingResources)
    );
    drop(work);
    assert_eq!(
        d.session.acknowledge_save(t),
        Ok(SaveAcknowledgement::Released)
    );
    assert_eq!(d.session.history_len(), 1);
}

#[test]
fn fixture_full_node_capacity_and_one_less_limit_have_atomic_distinct_outcomes() {
    for admitted_nodes in [63, 64] {
        let mut d = Driver::new();
        for n in 1..64 {
            d.set(&format!("state {n}"));
        }
        assert_eq!(d.session.history_len(), 64);
        let mut c = config(&d.session);
        c.capture.nodes = admitted_nodes;
        let port = d.session.install_save_preparation(c).unwrap();
        let mut ids = Ids::default();
        let before = d.session.snapshot();
        let stats = d.stats();
        let result = d
            .session
            .request_save(request(&d.session, c.binding.key), &mut ids);
        if admitted_nodes == 63 {
            assert!(matches!(
                result,
                Err(SaveCoordinationError::Capture(SaveDiagnostic {
                    code: SaveDiagnosticCode::CaptureLimit,
                    ..
                }))
            ));
            assert_eq!(ids.calls, 0);
        } else {
            let t = result.unwrap().ticket();
            let mut work = d.session.take_save_preparation(&port, t).unwrap();
            assert_eq!(drain(&mut work).len(), 64);
            drop(work);
            assert_eq!(
                d.session.acknowledge_save(t),
                Ok(SaveAcknowledgement::Released)
            );
        }
        assert_eq!(d.session.snapshot(), before);
        assert_eq!(d.stats(), stats);
    }
}

#[test]
fn dropping_session_cancels_detached_work_without_access_to_live_project() {
    let mut d = Driver::new();
    let mut ids = Ids::default();
    let (port, binding) = d.install();
    let t = d
        .session
        .request_save(request(&d.session, binding), &mut ids)
        .unwrap()
        .ticket();
    let mut work = d.session.take_save_preparation(&port, t).unwrap();
    drop(d);
    assert!(matches!(work.poll(1), Err(CaptureError::Cancelled)));
    assert_eq!(work.snapshot().manifest().source, t.source);
}

#[test]
fn static_control_layout_accounts_for_all_lanes_and_bounds_public_carriers() {
    let proof = save_control_layout();
    println!(
        "SaveCoord-Control-v1 static layout: {proof:?}; total={}",
        proof.total_bound()
    );
    assert!(proof.fits());
    assert_eq!(proof.lane_bounds.len(), 8);
    assert!(proof.lane_bounds[..4].iter().sum::<usize>() <= 256 * 1024);
    assert!(proof.lane_bounds[4..].iter().sum::<usize>() <= 256 * 1024);
    assert!(proof.total_bound() <= 512 * 1024);
    assert!(proof.total_bound() > proof.owner_bytes + proof.shared_control_bytes);
    assert!(proof.lane_bounds.iter().all(|n| *n > 0 && *n <= 64 * 1024));
}

struct Pin {
    blob: BlobRef,
    bytes: &'static [u8],
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
struct Pool {
    originals: BTreeMap<BlobRef, Arc<Pin>>,
}
impl ResourceStore for Pool {
    fn pin(&self, blob: BlobRef) -> Result<Arc<dyn BlobLease>, ResourceError> {
        self.originals
            .get(&blob)
            .map(|p| p.clone() as Arc<dyn BlobLease>)
            .ok_or(ResourceError::Missing)
    }
}

#[test]
fn real_frozen_golden_tree_keeps_branches_admissions_originals_and_raw_configuration_after_q() {
    let drops = Arc::new(AtomicUsize::new(0));
    let bytes: [&'static [u8]; 3] = [
        b"unreferenced original",
        b"opaque audio\0\xff",
        b"D-only original",
    ];
    let originals: BTreeMap<_, _> = bytes
        .iter()
        .enumerate()
        .map(|(i, bytes)| {
            assert!(bytes.len() <= 64 * 1024);
            let blob = BlobRef::new([i as u8 + 1; 32], bytes.len() as u64);
            (
                blob,
                Arc::new(Pin {
                    blob,
                    bytes,
                    drops: drops.clone(),
                }),
            )
        })
        .collect();
    let blobs: Vec<_> = originals.keys().copied().collect();
    let weak: Vec<_> = originals.values().map(Arc::downgrade).collect();
    let mut d = Driver {
        session: ProjectSession::with_resources(7, 70, Box::new(Pool { originals }), limits())
            .unwrap(),
        next: 1,
    };
    let asset = |id, blob, name: &str| Command::RegisterAsset {
        asset: Asset::new(
            AssetId::new(id).unwrap(),
            name.into(),
            blob,
            AssetKind::Opaque,
        )
        .unwrap(),
    };
    let doc = DocumentId::new(9).unwrap();
    let a = d.edit(vec![
        Command::CreateDocument {
            document: ElementDocument::new(
                doc,
                PixelSize::new(2, 2).unwrap(),
                LayerTree::default(),
            ),
        },
        Command::RemoveDocument { document: doc },
        asset(10, blobs[0], "unreferenced"),
        asset(11, blobs[1], "unknown.wav"),
        Command::InsertConfigRecord {
            index: 0,
            record: record("// keep\r\n[Unknown]\r\nX :  raw\r\n"),
        },
    ]);
    let b = d.set("B\n");
    d.set("C\r\n");
    d.nav(HistoryAction::Checkout { node: a });
    let branch_d = d.edit(vec![
        asset(12, blobs[2], "D only"),
        Command::SetConfigRecord {
            index: 0,
            record: record("D\r\n"),
        },
    ]);
    d.nav(HistoryAction::Undo);
    let before = d.session.snapshot();
    let golden: BTreeMap<_, _> = (1..=d.session.history_len())
        .map(|n| {
            let node = NodeId::new(n as u64).unwrap();
            (
                n as u64,
                (
                    d.session.history_entry(before.session_id, node).unwrap(),
                    d.session
                        .history_snapshot(before.session_id, node)
                        .unwrap()
                        .project,
                ),
            )
        })
        .collect();
    let (port, binding) = d.install();
    let mut ids = Ids::default();
    let r = request(&d.session, binding);
    let t = d.session.request_save(r, &mut ids).unwrap().ticket();
    // The real worker starts only AFTER a new branch, resource and draft exist.
    let e = d.edit(vec![
        asset(13, blobs[0], "shared new admission"),
        Command::SetConfigRecord {
            index: 0,
            record: record("E"),
        },
    ]);
    d.nav(HistoryAction::Checkout { node: b });
    d.draft(DraftAction::Begin {
        expected: d.session.project_info().state,
    });
    let draft = d.session.draft_info(before.session_id).unwrap();
    let live = d.session.snapshot();
    let stats = d.stats();
    let mut work = d.session.take_save_preparation(&port, t).unwrap();
    let snapshot = work.snapshot().clone();
    let manifest = snapshot.manifest().clone();
    assert_eq!(manifest.source, r.source);
    assert_eq!(t.history, SaveHistoryScope::from(&manifest.identity));
    assert_eq!(t.head, manifest.head);
    assert_eq!(manifest.nodes.len(), 5);
    assert_eq!(manifest.originals, blobs);
    assert_eq!(manifest.admissions.len(), 4); // transient document + three assets
    assert!(
        manifest
            .admissions
            .iter()
            .any(|a| a.object == ObjectId::Document(doc))
    );
    assert!(!manifest.nodes.iter().any(|n| n.alias == e.get()));
    let permanent_a = manifest
        .nodes
        .iter()
        .find(|n| n.alias == a.get())
        .unwrap()
        .id;
    let permanent_d = manifest
        .nodes
        .iter()
        .find(|n| n.alias == branch_d.get())
        .unwrap()
        .id;
    assert_eq!(manifest.redo_preferences, vec![(permanent_a, permanent_d)]);
    let nodes = drain(&mut work);
    let by_alias: BTreeMap<_, _> = manifest.nodes.iter().map(|n| (n.alias, n.id)).collect();
    for n in &nodes {
        let (entry, project) = &golden[&n.info().alias];
        assert_eq!(n.project(), project);
        assert_eq!(n.info().parent, entry.parent.map(|p| by_alias[&p.get()]));
    }
    for (i, p) in weak.iter().enumerate() {
        let p = p.upgrade().unwrap();
        assert_eq!(p.bytes, bytes[i]);
        assert_eq!(p.blob.byte_len() as usize, p.bytes.len());
    }
    assert_eq!(d.session.snapshot(), live);
    assert_eq!(d.stats(), stats);
    assert_eq!(d.session.draft_info(before.session_id).unwrap(), draft);
    assert_eq!(
        manifest.archive_metadata.as_ref(),
        &ArchiveMetadata::default()
    );
    assert_eq!(
        d.session.acknowledge_save(t),
        Ok(SaveAcknowledgement::WaitingResources)
    );
    drop(work);
    drop(snapshot);
    drop(nodes);
    assert_eq!(
        d.session.acknowledge_save(t),
        Ok(SaveAcknowledgement::Released)
    );
    assert_eq!(drops.load(Ordering::SeqCst), 0); // history/store still pin the originals
    assert_eq!(
        d.session
            .history_snapshot(before.session_id, a)
            .unwrap()
            .project
            .configuration()
            .to_bytes(),
        b"// keep\r\n[Unknown]\r\nX :  raw\r\n"
    );
    drop(d);
    assert_eq!(drops.load(Ordering::SeqCst), 3);
    assert!(weak.iter().all(|p| p.upgrade().is_none()));
}
