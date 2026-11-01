use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use skin_editor_application::{values::*, *};

fn record(value: &str) -> ConfigRecord {
    ConfigRecord::new(value.as_bytes().to_vec(), ConfigInterpretation::Opaque).unwrap()
}
fn value(session: &ProjectSession) -> Vec<u8> {
    session.snapshot().project.configuration().to_bytes()
}

#[test]
fn plain_undo_redo_remembers_each_fork_in_the_undone_path() {
    let mut d = Driver::new();
    let a = d.set("A").state.head;
    let b = d.set("B").state.head;
    let c = d.set("C").state.head;
    d.checkout(a);
    let other = d.set("other").state.head;
    d.checkout(b);
    let newer = d.set("newer").state.head;
    let saved = d.session.history_snapshot(d.scope(), c).unwrap();
    d.checkout(c);
    d.navigate(HistoryAction::Undo);
    d.navigate(HistoryAction::Undo);
    assert_eq!(d.session.project_info().state.head, a);
    assert_eq!(
        d.session
            .history_statistics(d.scope())
            .unwrap()
            .redo_preference_entries,
        2
    );
    assert_eq!(
        d.navigate(HistoryAction::Redo { child: None }).state.head,
        b
    );
    assert_eq!(
        d.navigate(HistoryAction::Redo { child: None }).state.head,
        c
    );
    assert_ne!(c, newer);
    d.checkout(other);
    d.navigate(HistoryAction::Undo);
    assert_eq!(
        d.navigate(HistoryAction::Redo { child: None }).state.head,
        other
    );
    assert_eq!(d.session.history_snapshot(d.scope(), c).unwrap(), saved);
    assert_eq!(d.session.history_len(), 6);
}

#[test]
fn failed_no_change_and_cached_navigation_do_not_replace_the_remembered_child() {
    let mut d = Driver::new();
    let a = d.set("A").state.head;
    let b = d.set("B").state.head;
    d.checkout(a);
    let c = d.set("C").state.head;
    d.checkout(b);
    let undo = d.history_request(HistoryAction::Undo);
    let old = d.session.navigate(undo.clone()).unwrap();
    assert_eq!(d.checkout(a).status, HistoryStatus::NoChange);
    let mut conflict = d.history_request(HistoryAction::Checkout { node: c });
    conflict.expected.revision += 1;
    assert!(matches!(
        d.session.navigate(conflict),
        Err(HistoryError::Conflict { .. })
    ));
    assert_eq!(
        d.navigate(HistoryAction::Redo { child: None }).state.head,
        b
    );
    d.checkout(c);
    d.navigate(HistoryAction::Undo);
    let before = d.session.snapshot();
    assert_eq!(d.session.navigate(undo).unwrap(), old);
    assert_eq!(d.session.snapshot(), before);
    assert_eq!(
        d.navigate(HistoryAction::Redo { child: None }).state.head,
        c
    );
    d.checkout(a);
    assert_eq!(
        d.session
            .history_statistics(d.scope())
            .unwrap()
            .redo_preference_entries,
        0
    );
    let redo = d.history_request(HistoryAction::Redo { child: None });
    assert_eq!(
        d.session.navigate(redo),
        Err(HistoryError::AmbiguousRedo { child_count: 2 })
    );
}

#[test]
fn ten_thousand_linear_undo_redo_steps_need_no_fork_preference_or_ancestor_address() {
    let mut d = Driver::new();
    let root = d.session.project_info().state.head;
    for i in 0..10_000 {
        d.set(&format!("step-{i}"));
    }
    let end = d.session.project_info().state.head;
    for _ in 0..10_000 {
        d.navigate(HistoryAction::Undo);
    }
    assert_eq!(d.session.project_info().state.head, root);
    assert_eq!(
        d.session
            .history_statistics(d.scope())
            .unwrap()
            .redo_preference_entries,
        0
    );
    for _ in 0..10_000 {
        d.navigate(HistoryAction::Redo { child: None });
    }
    assert_eq!(d.session.project_info().state.head, end);
    assert_eq!(
        d.session
            .history_statistics(d.scope())
            .unwrap()
            .redo_preference_entries,
        0
    );
    assert_eq!(d.session.history_len(), 10_001);
    assert!(end.short_address().len() <= MAX_NODE_ADDRESS);
}
struct Driver {
    session: ProjectSession,
    next: u128,
}
impl Driver {
    fn new() -> Self {
        Self::with_session(ProjectSession::new(7, 70).unwrap())
    }
    fn with_session(session: ProjectSession) -> Self {
        Self { session, next: 1 }
    }
    fn command_request(&mut self, commands: Vec<Command>) -> CommandRequest {
        let info = self.session.project_info();
        let request_id = RequestId::new(self.next).unwrap();
        self.next += 1;
        CommandRequest {
            api_version: API_VERSION,
            session_id: info.session_id,
            request_id,
            expected: info.state,
            commands,
        }
    }
    fn history_request(&mut self, action: HistoryAction) -> HistoryRequest {
        let info = self.session.project_info();
        let request_id = RequestId::new(self.next).unwrap();
        self.next += 1;
        HistoryRequest {
            api_version: API_VERSION,
            session_id: info.session_id,
            request_id,
            expected: info.state,
            action,
        }
    }
    fn commit(&mut self, commands: Vec<Command>) -> CommandResult {
        let request = self.command_request(commands);
        self.session.submit(request).unwrap()
    }
    fn set(&mut self, text: &str) -> CommandResult {
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
        self.commit(vec![command])
    }
    fn navigate(&mut self, action: HistoryAction) -> HistoryResult {
        let request = self.history_request(action);
        self.session.navigate(request).unwrap()
    }
    fn checkout(&mut self, node: NodeId) -> HistoryResult {
        self.navigate(HistoryAction::Checkout { node })
    }
    fn scope(&self) -> HistorySessionId {
        self.session.project_info().session_id
    }
}

#[test]
fn undo_redo_and_checkout_preserve_old_descendants_when_editing_a_branch() {
    let mut d = Driver::new();
    let root = d.session.snapshot();
    let a = d.set("A").state.head;
    let b = d.set("B").state.head;
    let c = d.set("C").state.head;
    let c_before = d.session.history_snapshot(d.scope(), c).unwrap();
    let b_summary = d.session.history_summary(d.scope(), b).unwrap();
    assert_eq!(d.navigate(HistoryAction::Undo).state.head, b);
    assert_eq!(value(&d.session), b"B");
    assert_eq!(
        d.navigate(HistoryAction::Redo { child: None }).state.head,
        c
    );
    d.checkout(a);
    let fork = d.set("fork").state.head;
    assert_eq!(d.session.history_len(), 5);
    let children = d.session.history_children(d.scope(), a, 0, 10).unwrap();
    assert_eq!(
        children.entries.iter().map(|e| e.id).collect::<Vec<_>>(),
        [b, fork]
    );
    assert_eq!(d.session.history_summary(d.scope(), b).unwrap(), b_summary);
    assert_eq!(d.session.history_snapshot(d.scope(), c).unwrap(), c_before);
    d.checkout(a);
    let before = d.session.snapshot();
    let ambiguous = d.history_request(HistoryAction::Redo { child: None });
    assert_eq!(
        d.session.navigate(ambiguous),
        Err(HistoryError::AmbiguousRedo { child_count: 2 })
    );
    assert_eq!(d.session.snapshot(), before);
    let skipped = d.history_request(HistoryAction::Redo { child: Some(c) });
    assert_eq!(
        d.session.navigate(skipped),
        Err(HistoryError::NotAChild {
            parent: a,
            child: c
        })
    );
    d.navigate(HistoryAction::Redo { child: Some(b) });
    d.navigate(HistoryAction::Redo { child: None });
    assert_eq!(value(&d.session), b"C");
    d.checkout(fork);
    assert_eq!(value(&d.session), b"fork");
    assert_eq!(
        d.session
            .history_snapshot(d.scope(), root.state.head)
            .unwrap()
            .project,
        root.project
    );
}

#[test]
fn navigation_increments_revision_even_between_equal_contents_without_new_nodes() {
    let mut d = Driver::new();
    let root = d.session.project_info().state.head;
    let a = d.set("same").state.head;
    d.checkout(root);
    let b = d.set("same").state.head;
    let before = d.session.snapshot();
    let count = d.session.history_len();
    let result = d.checkout(a);
    assert_eq!(result.status, HistoryStatus::Changed);
    assert!(result.changed_objects.is_empty());
    assert!(!result.configuration_changed);
    assert_eq!(result.state.revision, before.state.revision + 1);
    assert_eq!(d.session.snapshot().project, before.project);
    assert_ne!(a, b);
    assert_eq!(d.session.history_len(), count);
    let state = d.session.project_info().state;
    let result = d.checkout(a);
    assert_eq!(result.status, HistoryStatus::NoChange);
    assert_eq!(result.state, state);
}

#[test]
fn root_undo_leaf_redo_and_read_queries_do_not_change_head_or_revision() {
    let mut d = Driver::new();
    let before = d.session.snapshot();
    assert_eq!(
        d.navigate(HistoryAction::Undo).status,
        HistoryStatus::NoChange
    );
    assert_eq!(
        d.navigate(HistoryAction::Redo { child: None }).status,
        HistoryStatus::NoChange
    );
    assert_eq!(d.session.snapshot(), before);
    let leaf = d.set("leaf").state.head;
    let before = d.session.snapshot();
    assert_eq!(
        d.navigate(HistoryAction::Redo { child: None }).status,
        HistoryStatus::NoChange
    );
    d.session
        .history_snapshot(d.scope(), NodeId::new(1).unwrap())
        .unwrap();
    d.session.history_ancestors(d.scope(), leaf, 10).unwrap();
    d.session.history_children(d.scope(), leaf, 0, 10).unwrap();
    assert_eq!(d.session.snapshot(), before);
    assert_eq!(d.session.history_len(), 2);
}

#[test]
fn navigation_scope_expected_state_and_unknown_targets_are_validated_atomically() {
    let mut d = Driver::new();
    let stale = d.history_request(HistoryAction::Undo);
    d.set("new");
    let before = d.session.snapshot();
    assert!(matches!(
        d.session.navigate(stale),
        Err(HistoryError::Conflict { .. })
    ));
    let mut wrong = d.history_request(HistoryAction::Undo);
    wrong.session_id = HistorySessionId::new(71).unwrap();
    assert_eq!(d.session.navigate(wrong), Err(HistoryError::WrongSession));
    let mut old = d.history_request(HistoryAction::Undo);
    old.api_version = API_VERSION - 1;
    assert_eq!(
        d.session.navigate(old),
        Err(HistoryError::UnsupportedVersion(API_VERSION - 1))
    );
    let unknown = d.history_request(HistoryAction::Checkout {
        node: NodeId::new(99).unwrap(),
    });
    assert_eq!(
        d.session.navigate(unknown),
        Err(HistoryError::Query(HistoryQueryError::UnknownNode(
            NodeId::new(99).unwrap()
        )))
    );
    let mut wrong_head = d.history_request(HistoryAction::Undo);
    wrong_head.expected.head = NodeId::new(1).unwrap();
    assert!(matches!(
        d.session.navigate(wrong_head),
        Err(HistoryError::Conflict { .. })
    ));
    assert_eq!(d.session.snapshot(), before);
    assert_eq!(d.session.history_len(), 2);
}

#[test]
fn navigation_receipts_are_idempotent_and_share_ids_with_content_commands() {
    let mut d = Driver::new();
    let command = d.command_request(vec![Command::InsertConfigRecord {
        index: 0,
        record: record("A"),
    }]);
    d.session.submit(command.clone()).unwrap();
    d.set("B");
    let undo = d.history_request(HistoryAction::Undo);
    let result = d.session.navigate(undo.clone()).unwrap();
    d.set("fork");
    let before = d.session.snapshot();
    assert_eq!(d.session.navigate(undo.clone()).unwrap(), result);
    assert_eq!(d.session.snapshot(), before);
    let mut altered = undo.clone();
    altered.action = HistoryAction::Checkout {
        node: NodeId::new(1).unwrap(),
    };
    assert_eq!(
        d.session.navigate(altered),
        Err(HistoryError::RequestIdReused)
    );
    let mut control = d.history_request(HistoryAction::Undo);
    control.request_id = command.request_id;
    assert_eq!(
        d.session.navigate(control),
        Err(HistoryError::RequestIdReused)
    );
    let mut content = d.command_request(vec![]);
    content.request_id = undo.request_id;
    assert_eq!(
        d.session.submit(content),
        Err(CommandError::RequestIdReused)
    );
    assert_eq!(d.session.snapshot(), before);
    // Replaying an old content receipt after branching never changes the head.
    d.session.submit(command).unwrap();
    assert_eq!(d.session.snapshot(), before);
}

#[test]
fn failed_and_no_change_navigation_receipts_keep_the_original_outcome() {
    let mut d = Driver::new();
    let noop = d.history_request(HistoryAction::Undo);
    let noop_result = d.session.navigate(noop.clone()).unwrap();
    let a = d.set("A").state.head;
    let b = d.set("B").state.head;
    d.checkout(a);
    d.set("C");
    d.checkout(a);
    let ambiguous = d.history_request(HistoryAction::Redo { child: None });
    let error = d.session.navigate(ambiguous.clone()).unwrap_err();
    d.navigate(HistoryAction::Redo { child: Some(b) });
    let before = d.session.snapshot();
    assert_eq!(d.session.navigate(noop).unwrap(), noop_result);
    assert_eq!(d.session.navigate(ambiguous), Err(error));
    assert_eq!(d.session.snapshot(), before);
}

#[test]
fn canonical_addresses_have_fixed_upper_bounds_and_reject_noncanonical_inputs() {
    let max = NodeId::new(u64::MAX).unwrap();
    let coordinate = BranchCoordinate::new(u64::MAX, u64::MAX).unwrap();
    assert_eq!(max.short_address().len(), MAX_NODE_ADDRESS);
    assert_eq!(coordinate.short_address().len(), MAX_BRANCH_ADDRESS);
    assert_eq!(NodeId::parse_address(&max.short_address()).unwrap(), max);
    assert_eq!(
        BranchCoordinate::parse_address(&coordinate.short_address()).unwrap(),
        coordinate
    );
    assert_eq!(NodeId::parse_address("n0"), Err(AddressError::ZeroIdentity));
    assert_eq!(
        BranchCoordinate::parse_address("b0:0"),
        Err(AddressError::ZeroIdentity)
    );
    for bad in ["", "1", "N1", "n01", "n-1", "n+1", "n1 ", "n1/2", "n中"] {
        assert!(NodeId::parse_address(bad).is_err(), "{bad}");
    }
    for bad in ["b1", "b01:1", "b1:01", "b1:-1", "b1:1:2", "B1:0", "b1:中"] {
        assert!(BranchCoordinate::parse_address(bad).is_err(), "{bad}");
    }
    assert_eq!(
        NodeId::parse_address("nzzzzzzzzzzzzz"),
        Err(AddressError::Overflow)
    );
    assert_eq!(
        NodeId::parse_address(&format!("n{}", "1".repeat(14))),
        Err(AddressError::TooLong)
    );
    assert_eq!(
        BranchCoordinate::parse_address(&"b".repeat(29)),
        Err(AddressError::TooLong)
    );
    assert_eq!(std::mem::size_of::<NodeId>(), 8);
    assert_eq!(std::mem::size_of::<BranchCoordinate>(), 16);
}

#[test]
fn branch_coordinates_resolve_stably_after_checkout_and_display_sorting() {
    let mut d = Driver::new();
    let a = d.set("A").state.head;
    let b = d.set("B").state.head;
    let old = d.session.history_summary(d.scope(), b).unwrap();
    d.checkout(a);
    let c = d.set("C").state.head;
    let fork = d.session.history_summary(d.scope(), c).unwrap();
    assert_eq!(old.coordinate.short_address(), "b1:2");
    assert_eq!(fork.coordinate.short_address(), "b2:0");
    let mut display = d.session.history_children(d.scope(), a, 0, 10).unwrap();
    display.entries.reverse();
    for summary in &display.entries {
        assert_eq!(
            d.session
                .resolve_history_address(d.scope(), &summary.coordinate.short_address())
                .unwrap(),
            summary.id
        );
        assert_eq!(
            d.session
                .resolve_history_address(d.scope(), &summary.id.short_address())
                .unwrap(),
            summary.id
        );
    }
    d.checkout(b);
    d.set("continuation");
    assert_eq!(
        d.session.history_summary(d.scope(), b).unwrap().coordinate,
        old.coordinate
    );
    assert!(matches!(
        d.session.resolve_history_address(d.scope(), "b9:9"),
        Err(HistoryQueryError::UnknownCoordinate(_))
    ));
    assert!(matches!(
        d.session.resolve_history_address(d.scope(), "nzzz"),
        Err(HistoryQueryError::UnknownNode(_))
    ));
    assert_eq!(
        d.session
            .resolve_history_address(HistorySessionId::new(71).unwrap(), "n1"),
        Err(HistoryQueryError::WrongSession)
    );
    let other = ProjectSession::new(7, 71).unwrap();
    assert_eq!(
        other.resolve_history_address(d.scope(), "n1"),
        Err(HistoryQueryError::WrongSession)
    );
}

#[test]
fn history_pages_enforce_limits_offsets_and_session_scope() {
    let mut d = Driver::new();
    let leaf = d.set("leaf").state.head;
    let root = NodeId::new(1).unwrap();
    for limit in [0, MAX_HISTORY_PAGE + 1, usize::MAX] {
        assert_eq!(
            d.session.history_children(d.scope(), root, 0, limit),
            Err(HistoryQueryError::InvalidPageLimit(limit))
        );
        assert_eq!(
            d.session.history_ancestors(d.scope(), leaf, limit),
            Err(HistoryQueryError::InvalidPageLimit(limit))
        );
    }
    assert!(matches!(
        d.session.history_children(d.scope(), root, usize::MAX, 1),
        Err(HistoryQueryError::InvalidOffset { .. })
    ));
    let empty = d.session.history_children(d.scope(), root, 1, 1).unwrap();
    assert!(empty.entries.is_empty());
    assert_eq!(empty.next_offset, None);
    let first = d.session.history_ancestors(d.scope(), leaf, 1).unwrap();
    assert_eq!(first.next_ancestor, Some(root));
    let last = d.session.history_ancestors(d.scope(), root, 1).unwrap();
    assert_eq!(last.next_ancestor, None);
    assert_eq!(
        d.session
            .history_summary(HistorySessionId::new(71).unwrap(), root),
        Err(HistoryQueryError::WrongSession)
    );
}

struct Lease {
    blob: BlobRef,
    _bytes: Vec<u8>,
}
impl BlobLease for Lease {
    fn reference(&self) -> BlobRef {
        self.blob
    }
}
struct Store {
    blobs: BTreeMap<BlobRef, Arc<Lease>>,
    calls: Arc<AtomicUsize>,
    available: Arc<AtomicBool>,
}
impl ResourceStore for Store {
    fn pin(&self, blob: BlobRef) -> Result<Arc<dyn BlobLease>, ResourceError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        if !self.available.load(Ordering::SeqCst) {
            return Err(ResourceError::Unavailable);
        }
        self.blobs
            .get(&blob)
            .cloned()
            .map(|p| p as Arc<dyn BlobLease>)
            .ok_or(ResourceError::Missing)
    }
}

#[test]
fn cross_branch_resources_and_document_state_survive_deletion_and_store_unavailability() {
    let calls = Arc::new(AtomicUsize::new(0));
    let available = Arc::new(AtomicBool::new(true));
    let blobs: Vec<_> = (1..=2u8)
        .map(|n| {
            Arc::new(Lease {
                blob: BlobRef::new([n; 32], 4),
                _bytes: vec![n; 4],
            })
        })
        .collect();
    let store = Store {
        blobs: blobs.iter().map(|p| (p.blob, p.clone())).collect(),
        calls: calls.clone(),
        available: available.clone(),
    };
    let mut d = Driver::with_session(
        ProjectSession::with_resources(7, 70, Box::new(store), SessionLimits::default()).unwrap(),
    );
    let aid = AssetId::new(1).unwrap();
    let other = AssetId::new(2).unwrap();
    let doc = DocumentId::new(1).unwrap();
    let layer = LayerId::new(1).unwrap();
    let asset = |id, index: usize| {
        Asset::new(
            id,
            format!("source {index}"),
            blobs[index].blob,
            AssetKind::RasterImage(PixelSize::new(2, 2).unwrap()),
        )
        .unwrap()
    };
    let a = d
        .commit(vec![
            Command::RegisterAsset {
                asset: asset(aid, 0),
            },
            Command::CreateDocument {
                document: ElementDocument::new(
                    doc,
                    PixelSize::new(2, 2).unwrap(),
                    LayerTree::new(
                        vec![layer],
                        vec![LayerNode {
                            id: layer,
                            local: Affine2::IDENTITY,
                            opacity: Opacity::OPAQUE,
                            visible: true,
                            kind: LayerKind::Image { asset: aid },
                        }],
                    )
                    .unwrap(),
                ),
            },
        ])
        .state
        .head;
    let b = d
        .commit(vec![
            Command::RegisterAsset {
                asset: asset(other, 1),
            },
            Command::SetLayerAsset {
                document: doc,
                layer,
                asset: other,
            },
            Command::RemoveAsset { asset: aid },
        ])
        .state
        .head;
    let b_before = d.session.history_snapshot(d.scope(), b).unwrap();
    available.store(false, Ordering::SeqCst);
    d.checkout(a);
    let c = d
        .commit(vec![
            Command::RenameAsset {
                asset: aid,
                name: "branch C".into(),
            },
            Command::SetLayerTransform {
                document: doc,
                layer,
                transform: Affine2::new([1.0, 0.0, 0.0, 1.0, 7.0, 0.0]).unwrap(),
            },
        ])
        .state
        .head;
    d.checkout(b);
    assert!(d.session.snapshot().project.assets().get(aid).is_none());
    assert_eq!(d.session.history_snapshot(d.scope(), b).unwrap(), b_before);
    d.checkout(c);
    let snapshot = d.session.snapshot();
    assert_eq!(
        snapshot.project.assets().get(aid).unwrap().name(),
        "branch C"
    );
    assert_eq!(
        snapshot
            .project
            .document(doc)
            .unwrap()
            .layers()
            .node(layer)
            .unwrap()
            .local
            .coefficients()[4],
        7.0
    );
    d.checkout(a);
    assert_eq!(
        d.session
            .snapshot()
            .project
            .assets()
            .get(aid)
            .unwrap()
            .name(),
        "source 0"
    );
    let bad = d.command_request(vec![Command::RegisterAsset {
        asset: asset(other, 1),
    }]);
    assert!(matches!(
        d.session.submit(bad),
        Err(CommandError::Command {
            reason: CommandFailure::ReusedIdentity(_),
            ..
        })
    ));
    assert_eq!(calls.load(Ordering::SeqCst), 2);
    assert_eq!(
        d.session
            .history_statistics(d.scope())
            .unwrap()
            .retained_blobs,
        2
    );
    for blob in &blobs {
        assert_eq!(Arc::strong_count(blob), 3);
    }
    drop(d);
    for blob in &blobs {
        assert_eq!(Arc::strong_count(blob), 1);
    }
}

#[test]
fn scale_ten_thousand_deep_history_queries_and_addresses_are_bounded() {
    let mut d = Driver::new();
    for index in 0..10_000 {
        d.set(&index.to_string());
    }
    let head = d.session.project_info().state.head;
    let before = d.session.snapshot();
    for number in 1..=10_001u64 {
        let id = NodeId::new(number).unwrap();
        let summary = d.session.history_summary(d.scope(), id).unwrap();
        let address = id.short_address();
        let coordinate = summary.coordinate.short_address();
        assert!(address.len() <= MAX_NODE_ADDRESS && coordinate.len() <= MAX_BRANCH_ADDRESS);
        assert_eq!(
            d.session
                .resolve_history_address(d.scope(), &address)
                .unwrap(),
            id
        );
        assert_eq!(
            d.session
                .resolve_history_address(d.scope(), &coordinate)
                .unwrap(),
            id
        );
    }
    let mut cursor = Some(head);
    let mut count = 0;
    let mut seen = BTreeSet::new();
    while let Some(start) = cursor {
        let page = d.session.history_ancestors(d.scope(), start, 127).unwrap();
        assert!(page.entries.len() <= 127);
        for summary in &page.entries {
            assert!(seen.insert(summary.id));
            count += 1;
        }
        cursor = page.next_ancestor;
    }
    assert_eq!(count, 10_001);
    let middle = d
        .session
        .history_snapshot(d.scope(), NodeId::new(5_001).unwrap())
        .unwrap();
    assert_eq!(middle.project.configuration().to_bytes(), b"4999");
    assert_eq!(d.session.snapshot(), before);
    d.checkout(NodeId::new(1).unwrap());
    assert_eq!(value(&d.session), b"");
    d.checkout(head);
    assert_eq!(value(&d.session), b"9999");
    d.navigate(HistoryAction::Undo);
    assert_eq!(value(&d.session), b"9998");
    d.navigate(HistoryAction::Redo { child: None });
    assert_eq!(value(&d.session), b"9999");
    let stats = d.session.history_statistics(d.scope()).unwrap();
    assert_eq!(
        (
            stats.nodes,
            stats.edges,
            stats.coordinate_entries,
            stats.branches,
            stats.max_depth
        ),
        (10_001, 10_000, 10_001, 1, 10_000)
    );
    eprintln!("P1_HISTORY_DEEP {stats:?}");
}

#[test]
fn scale_ten_thousand_siblings_keep_old_coordinates_and_stable_paging() {
    let mut d = Driver::new();
    let base = d.set("base").state.head;
    let mut leaves = Vec::new();
    for index in 0..10_000 {
        if d.session.project_info().state.head != base {
            d.checkout(base);
        }
        leaves.push(d.set(&format!("leaf-{index}")).state.head);
    }
    let first_coordinate = d
        .session
        .history_summary(d.scope(), leaves[0])
        .unwrap()
        .coordinate;
    let mut offset = 0;
    let mut listed = Vec::new();
    loop {
        let page = d
            .session
            .history_children(d.scope(), base, offset, MAX_HISTORY_PAGE)
            .unwrap();
        assert!(page.entries.len() <= MAX_HISTORY_PAGE);
        for summary in page.entries {
            let address = summary.coordinate.short_address();
            assert!(address.len() <= MAX_BRANCH_ADDRESS);
            assert_eq!(
                d.session
                    .resolve_history_address(d.scope(), &address)
                    .unwrap(),
                summary.id
            );
            listed.push(summary.id);
        }
        match page.next_offset {
            Some(next) => offset = next,
            None => break,
        }
    }
    assert_eq!(listed, leaves);
    d.checkout(base);
    let ambiguous = d.history_request(HistoryAction::Redo { child: None });
    assert_eq!(
        d.session.navigate(ambiguous),
        Err(HistoryError::AmbiguousRedo {
            child_count: 10_000
        })
    );
    d.navigate(HistoryAction::Redo {
        child: Some(leaves[0]),
    });
    assert_eq!(value(&d.session), b"leaf-0");
    assert_eq!(
        d.session
            .history_summary(d.scope(), leaves[0])
            .unwrap()
            .coordinate,
        first_coordinate
    );
    let stats = d.session.history_statistics(d.scope()).unwrap();
    assert_eq!(
        (
            stats.nodes,
            stats.edges,
            stats.coordinate_entries,
            stats.branches,
            stats.max_depth
        ),
        (10_002, 10_001, 10_002, 10_000, 2)
    );
    eprintln!("P1_HISTORY_WIDE {stats:?}");
}

#[test]
fn seeded_branch_navigation_matches_an_independent_snapshot_model() {
    struct Reference {
        id: NodeId,
        parent: Option<usize>,
        children: Vec<usize>,
        value: Vec<u8>,
    }
    let mut d = Driver::new();
    let root = d.session.project_info().state.head;
    let mut nodes = vec![Reference {
        id: root,
        parent: None,
        children: Vec::new(),
        value: Vec::new(),
    }];
    let mut current = 0;
    let mut revision = 0;
    let mut seed = 0x1a2b3c4d_u64;
    for step in 0..1_500 {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        match (seed >> 32) % 4 {
            0 => {
                let text = format!("value-{}", (seed >> 8) % 31);
                let result = d.set(&text);
                if nodes[current].value != text.as_bytes() {
                    let index = nodes.len();
                    nodes[current].children.push(index);
                    nodes.push(Reference {
                        id: result.state.head,
                        parent: Some(current),
                        children: Vec::new(),
                        value: text.into_bytes(),
                    });
                    current = index;
                    revision += 1;
                } else {
                    assert_eq!(result.status, CommandStatus::NoChange);
                }
            }
            1 => {
                d.navigate(HistoryAction::Undo);
                if let Some(parent) = nodes[current].parent {
                    current = parent;
                    revision += 1;
                }
            }
            2 => {
                if nodes[current].children.is_empty() {
                    d.navigate(HistoryAction::Redo { child: None });
                } else {
                    let child = nodes[current].children
                        [(seed >> 16) as usize % nodes[current].children.len()];
                    d.navigate(HistoryAction::Redo {
                        child: Some(nodes[child].id),
                    });
                    current = child;
                    revision += 1;
                }
            }
            _ => {
                let target = (seed >> 16) as usize % nodes.len();
                d.checkout(nodes[target].id);
                if target != current {
                    current = target;
                    revision += 1;
                }
            }
        }
        assert_eq!(
            d.session.project_info().state,
            ExpectedState {
                head: nodes[current].id,
                revision
            }
        );
        assert_eq!(value(&d.session), nodes[current].value);
        assert_eq!(d.session.history_len(), nodes.len());
        if step % 50 == 0 {
            let index = (seed >> 24) as usize % nodes.len();
            assert_eq!(
                d.session
                    .history_snapshot(d.scope(), nodes[index].id)
                    .unwrap()
                    .project
                    .configuration()
                    .to_bytes(),
                nodes[index].value
            );
        }
    }
}
