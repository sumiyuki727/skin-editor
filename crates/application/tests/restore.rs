use skin_editor_application::{values::*, *};
use std::sync::Arc;

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

struct Empty;
impl ResourceStore for Empty {
    fn pin(&self, _: BlobRef) -> Result<Arc<dyn BlobLease>, ResourceError> {
        Err(ResourceError::Missing)
    }
}
fn captured() -> (RestoreHeader, Vec<RestoreNode>, Driver) {
    let mut d = Driver::new();
    let doc = DocumentId::new(9).unwrap();
    d.edit(vec![
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
            record: record("A"),
        },
    ]);
    d.set("B");
    d.nav(HistoryAction::Undo);
    d.set("C");
    d.nav(HistoryAction::Undo);
    let mut c = d.capture(&mut Ids::new());
    let m = c.snapshot().unwrap().manifest().clone();
    let by_id = drain(&mut c)
        .into_iter()
        .map(|n| (n.info().id, n.project().clone()))
        .collect::<std::collections::BTreeMap<_, _>>();
    let nodes = m
        .nodes
        .iter()
        .map(|n| RestoreNode {
            id: n.id,
            parent: n.parent,
            alias: n.alias,
            coordinate: n.coordinate,
            depth: n.depth,
            operation: n.operation.clone(),
            origin: Arc::new(OriginalNodeRecord {
                record_version: 2,
                state: [1; 32],
                action: b"{}".to_vec(),
                checksum: [2; 32],
            }),
            project: by_id[&n.id].clone(),
        })
        .collect();
    (
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
            metadata: Arc::default(),
        },
        nodes,
        d,
    )
}
fn build(
    h: RestoreHeader,
    nodes: Vec<RestoreNode>,
    cancel: CaptureCancellation,
) -> Result<RestoreCandidate, RestoreError> {
    let mut b = RestoreBuilder::new(h, RestoreLimits::default(), cancel)?;
    for n in nodes {
        b.push(n)?;
    }
    b.finish(&Empty)
}
#[test]
fn restored_tree_preserves_ids_navigation_and_consumed_transient_object_ids() {
    let (h, nodes, d) = captured();
    let identities = h.identity.clone();
    let c = build(h, nodes, CaptureCancellation::default()).unwrap();
    let session = c
        .into_session(
            HistorySessionId::new(777).unwrap(),
            Box::new(Empty),
            SessionLimits::default(),
            CaptureCancellation::default(),
        )
        .unwrap();
    let mut restored = Driver {
        session,
        request: 0,
    };
    assert_eq!(
        restored.session.snapshot().project,
        d.session.snapshot().project
    );
    assert_eq!(restored.session.history_len(), 4);
    restored.nav(HistoryAction::Redo { child: None });
    assert_eq!(
        restored
            .session
            .snapshot()
            .project
            .configuration()
            .to_bytes(),
        b"C"
    );
    let old = d.session.project_info();
    assert!(
        restored
            .session
            .navigate(HistoryRequest {
                api_version: API_VERSION,
                session_id: old.session_id,
                request_id: RequestId::new(99).unwrap(),
                expected: old.state,
                action: HistoryAction::Undo
            })
            .is_err()
    );
    let i = restored.session.project_info();
    let before = i.state;
    assert!(
        restored
            .session
            .submit(CommandRequest {
                api_version: API_VERSION,
                session_id: i.session_id,
                request_id: RequestId::new(100).unwrap(),
                expected: i.state,
                commands: vec![Command::CreateDocument {
                    document: ElementDocument::new(
                        DocumentId::new(9).unwrap(),
                        PixelSize::new(2, 2).unwrap(),
                        LayerTree::default()
                    )
                }]
            })
            .is_err()
    );
    assert_eq!(restored.session.project_info().state, before);
    let c = restored.capture(&mut Ids::new());
    assert_eq!(c.snapshot().unwrap().manifest().identity, identities);
    assert!(
        c.snapshot()
            .unwrap()
            .manifest()
            .nodes
            .iter()
            .all(|n| n.origin.is_some())
    );
}
#[test]
fn detached_restore_rejects_topology_counter_and_navigation_corruption() {
    for case in 0..6 {
        let (mut h, mut nodes, _) = captured();
        match case {
            0 => nodes[2].parent = Some(nodes[2].id),
            1 => nodes[2].id = nodes[1].id,
            2 => h.next_alias = 2,
            3 => h.redo_preferences = vec![(nodes[0].id, nodes[3].id)],
            4 => h.node_count += 1,
            5 => nodes[1].depth += 1,
            _ => unreachable!(),
        }
        assert!(
            build(h, nodes, CaptureCancellation::default()).is_err(),
            "case {case}"
        );
    }
}
#[test]
fn restore_and_activation_cancellation_and_budget_fail_without_publishing_a_session() {
    let (h, nodes, d) = captured();
    let before = d.session.snapshot();
    let cancel = CaptureCancellation::default();
    let mut b = RestoreBuilder::new(h.clone(), RestoreLimits::default(), cancel.clone()).unwrap();
    b.push(nodes[0].clone()).unwrap();
    cancel.cancel();
    assert_eq!(b.push(nodes[1].clone()), Err(RestoreError::Cancelled));
    assert!(matches!(b.finish(&Empty), Err(RestoreError::Cancelled)));
    assert!(matches!(
        RestoreBuilder::new(
            h.clone(),
            RestoreLimits {
                nodes: 1,
                ..RestoreLimits::default()
            },
            CaptureCancellation::default()
        ),
        Err(RestoreError::Limit(_))
    ));
    let candidate = build(h, nodes, CaptureCancellation::default()).unwrap();
    let cancel = CaptureCancellation::default();
    cancel.cancel();
    assert!(matches!(
        candidate.into_session(
            HistorySessionId::new(777).unwrap(),
            Box::new(Empty),
            SessionLimits::default(),
            cancel
        ),
        Err(RestoreError::Cancelled)
    ));
    assert_eq!(d.session.snapshot().project, before.project);
    assert_eq!(d.session.project_info().state, before.state);
}
