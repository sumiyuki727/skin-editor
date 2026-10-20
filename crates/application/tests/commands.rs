use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use skin_editor_application::*;
use skin_editor_domain::*;

fn aid(n: u128) -> AssetId {
    AssetId::new(n).unwrap()
}
fn did(n: u128) -> DocumentId {
    DocumentId::new(n).unwrap()
}
fn lid(n: u128) -> LayerId {
    LayerId::new(n).unwrap()
}
fn cid(n: u128) -> ComponentId {
    ComponentId::new(n).unwrap()
}
fn oid(n: u128) -> OutputId {
    OutputId::new(n).unwrap()
}
fn size(w: u32, h: u32) -> PixelSize {
    PixelSize::new(w, h).unwrap()
}
fn transform(x: f64, y: f64) -> Affine2 {
    Affine2::new([1.0, 0.0, 0.0, 1.0, x, y]).unwrap()
}
fn blob(n: u8) -> BlobRef {
    BlobRef::new([n; 32], 4)
}
fn asset(id: u128, content: u8) -> Asset {
    Asset::new(
        aid(id),
        format!("source {id}"),
        blob(content),
        AssetKind::RasterImage(size(2, 2)),
    )
    .unwrap()
}
fn doc(id: u128) -> ElementDocument {
    ElementDocument::new(did(id), size(10, 20), LayerTree::default())
}
fn group(id: u128) -> LayerNode {
    LayerNode {
        id: lid(id),
        local: Affine2::IDENTITY,
        opacity: Opacity::OPAQUE,
        visible: true,
        kind: LayerKind::Group { children: vec![] },
    }
}
fn image(id: u128, source: u128) -> LayerNode {
    LayerNode {
        kind: LayerKind::Image { asset: aid(source) },
        ..group(id)
    }
}
fn cgroup(id: u128) -> ComponentNode {
    ComponentNode {
        id: cid(id),
        local: Affine2::IDENTITY,
        kind: ComponentKind::Group { children: vec![] },
    }
}
fn member(id: u128, document: u128) -> ComponentNode {
    ComponentNode {
        kind: ComponentKind::Element {
            document: did(document),
            document_to_component: Affine2::IDENTITY,
        },
        ..cgroup(id)
    }
}
fn record(raw: &[u8]) -> ConfigRecord {
    ConfigRecord::new(raw.to_vec(), ConfigInterpretation::Opaque).unwrap()
}
fn mapping(id: u128, document: u128, path: &str) -> OutputMapping {
    OutputMapping::new(
        oid(id),
        did(document),
        "explicit".into(),
        OutputPath::new(path.into()).unwrap(),
    )
    .unwrap()
}

struct Lease {
    reference: BlobRef,
    _bytes: Vec<u8>,
}
impl BlobLease for Lease {
    fn reference(&self) -> BlobRef {
        self.reference
    }
}
struct Store {
    data: BTreeMap<BlobRef, Arc<Lease>>,
    calls: Arc<AtomicUsize>,
}
impl ResourceStore for Store {
    fn pin(&self, blob: BlobRef) -> Result<Arc<dyn BlobLease>, ResourceError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.data
            .get(&blob)
            .cloned()
            .map(|p| p as Arc<dyn BlobLease>)
            .ok_or(ResourceError::Missing)
    }
}
fn session_with(
    contents: &[u8],
    limits: SessionLimits,
) -> (ProjectSession, Arc<AtomicUsize>, Vec<Arc<Lease>>) {
    let calls = Arc::new(AtomicUsize::new(0));
    let probes: Vec<_> = contents
        .iter()
        .map(|n| {
            Arc::new(Lease {
                reference: blob(*n),
                _bytes: vec![*n; 4],
            })
        })
        .collect();
    let data = probes.iter().map(|p| (p.reference, p.clone())).collect();
    let session = ProjectSession::with_resources(
        7,
        70,
        Box::new(Store {
            data,
            calls: calls.clone(),
        }),
        limits,
    )
    .unwrap();
    (session, calls, probes)
}
fn session() -> ProjectSession {
    session_with(&[1, 2], SessionLimits::default()).0
}
fn request(s: &ProjectSession, id: u128, commands: Vec<Command>) -> CommandRequest {
    let info = s.project_info();
    CommandRequest {
        api_version: API_VERSION,
        session_id: info.session_id,
        request_id: RequestId::new(id).unwrap(),
        expected: info.state,
        commands,
    }
}
fn commit(s: &mut ProjectSession, id: u128, commands: Vec<Command>) -> CommandResult {
    let r = request(s, id, commands);
    s.submit(r).unwrap()
}
fn baseline() -> Vec<Command> {
    vec![
        Command::RegisterAsset { asset: asset(1, 1) },
        Command::CreateDocument { document: doc(1) },
        Command::AddLayer {
            document: did(1),
            node: group(1),
            parent: None,
            index: 0,
        },
        Command::AddLayer {
            document: did(1),
            node: image(2, 1),
            parent: Some(lid(1)),
            index: 0,
        },
        Command::AddComponent {
            node: cgroup(1),
            parent: None,
            index: 0,
        },
        Command::AddComponent {
            node: member(2, 1),
            parent: Some(cid(1)),
            index: 0,
        },
        Command::SetOutputMapping {
            mapping: mapping(1, 1, "element.png"),
        },
        Command::InsertConfigRecord {
            index: 0,
            record: record(b"Unknown : keep\r\n"),
        },
    ]
}
fn assert_unchanged(s: &ProjectSession, before: &ProjectSnapshot, count: usize) {
    assert_eq!(&s.snapshot(), before);
    assert_eq!(s.history_len(), count);
}

#[test]
fn multi_command_success_publishes_exactly_one_node_and_owned_queries() {
    let mut s = session();
    let root = s.snapshot();
    let result = commit(&mut s, 1, baseline());
    assert_eq!(result.status, CommandStatus::Committed);
    assert_eq!(result.state.revision, 1);
    assert_eq!(s.history_len(), 2);
    let entry = s
        .history_entry(result.session_id, result.state.head)
        .unwrap();
    assert_eq!(entry.parent, Some(root.state.head));
    assert_eq!(entry.request_id, Some(result.request_id));
    assert!(entry.configuration_changed);
    assert!(entry.changed_objects.contains(&ObjectId::Layer(lid(2))));
    assert_eq!(s.snapshot().project.outputs().len(), 1);
    let old = s
        .history_snapshot(result.session_id, root.state.head)
        .unwrap();
    assert_eq!(old.project, root.project);
    assert_eq!(old.node, root.state.head);
    let mut detached = s.snapshot();
    detached.project = root.project;
    assert_eq!(s.snapshot().project.documents().len(), 1);
}

#[test]
fn every_step_is_validated_and_later_failure_rolls_back_all_prior_commands() {
    let mut s = session();
    let before = s.snapshot();
    let bad = request(
        &s,
        1,
        vec![
            Command::CreateDocument { document: doc(1) },
            Command::AddLayer {
                document: did(1),
                node: image(2, 99),
                parent: None,
                index: 0,
            },
        ],
    );
    assert!(matches!(
        s.submit(bad),
        Err(CommandError::Command {
            index: 1,
            reason: CommandFailure::Model(ModelError::MissingReference(_))
        })
    ));
    assert_unchanged(&s, &before, 1);
    // Failed candidates did not permanently reserve the new document/layer IDs.
    commit(&mut s, 2, baseline());
    assert_eq!(s.history_len(), 2);
}

#[test]
fn a_failure_at_the_first_step_does_not_publish_or_consume_node_ids() {
    let mut s = session();
    let before = s.snapshot();
    let bad = request(&s, 1, vec![Command::RemoveAsset { asset: aid(99) }]);
    assert!(matches!(
        s.submit(bad),
        Err(CommandError::Command { index: 0, .. })
    ));
    assert_unchanged(&s, &before, 1);
    assert_eq!(commit(&mut s, 2, baseline()).state.head.get(), 2);
}

#[test]
fn effective_noops_and_net_zero_batches_do_not_create_history() {
    let mut s = session();
    let original = s.snapshot();
    let result = commit(&mut s, 1, vec![]);
    assert_eq!(result.status, CommandStatus::NoChange);
    commit(
        &mut s,
        2,
        vec![
            Command::CreateDocument { document: doc(1) },
            Command::RemoveDocument { document: did(1) },
        ],
    );
    assert_unchanged(&s, &original, 1);
    commit(&mut s, 3, baseline());
    let before = s.snapshot();
    let result = commit(
        &mut s,
        4,
        vec![
            Command::SetLayerTransform {
                document: did(1),
                layer: lid(2),
                transform: Affine2::IDENTITY,
            },
            Command::MoveLayer {
                document: did(1),
                layer: lid(2),
                parent: Some(lid(1)),
                index: 0,
            },
            Command::SetLayerProperties {
                document: did(1),
                layer: lid(2),
                opacity: Opacity::OPAQUE,
                visible: true,
            },
            Command::RenameAsset {
                asset: aid(1),
                name: "source 1".into(),
            },
            Command::SetConfigRecord {
                index: 0,
                record: record(b"Unknown : keep\r\n"),
            },
            Command::SetOutputMapping {
                mapping: mapping(1, 1, "element.png"),
            },
        ],
    );
    assert_eq!(result.status, CommandStatus::NoChange);
    assert!(result.changed_objects.is_empty());
    assert_unchanged(&s, &before, 2);
}

#[test]
fn no_net_change_never_pins_transient_resources() {
    let (mut s, calls, _) = session_with(&[], SessionLimits::default());
    let result = commit(
        &mut s,
        1,
        vec![
            Command::RegisterAsset { asset: asset(1, 1) },
            Command::RemoveAsset { asset: aid(1) },
        ],
    );
    assert_eq!(result.status, CommandStatus::NoChange);
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    assert_eq!(s.history_len(), 1);
}

#[test]
fn revision_and_head_conflicts_are_checked_before_candidate_work() {
    let mut s = session();
    let stale = request(&s, 1, baseline());
    commit(
        &mut s,
        2,
        vec![Command::CreateDocument { document: doc(2) }],
    );
    let before = s.snapshot();
    assert!(matches!(
        s.submit(stale),
        Err(CommandError::Conflict { .. })
    ));
    let mut wrong_head = request(&s, 3, vec![]);
    wrong_head.expected.head = NodeId::new(1).unwrap();
    assert!(matches!(
        s.submit(wrong_head),
        Err(CommandError::Conflict { .. })
    ));
    let mut wrong_revision = request(&s, 4, vec![]);
    wrong_revision.expected.revision = 0;
    assert!(matches!(
        s.submit(wrong_revision),
        Err(CommandError::Conflict { .. })
    ));
    assert_unchanged(&s, &before, 2);
}

#[test]
fn retry_returns_original_result_even_after_another_commit_without_repinning() {
    let (mut s, calls, _) = session_with(&[1], SessionLimits::default());
    let first = request(&s, 1, baseline());
    let result = s.submit(first.clone()).unwrap();
    commit(
        &mut s,
        2,
        vec![Command::RenameAsset {
            asset: aid(1),
            name: "new".into(),
        }],
    );
    let before = s.snapshot();
    let call_count = calls.load(Ordering::SeqCst);
    assert_eq!(s.submit(first.clone()).unwrap(), result);
    assert_eq!(calls.load(Ordering::SeqCst), call_count);
    assert_unchanged(&s, &before, 3);
    let validation = s.validate(&first).unwrap();
    assert!(validation.cached);
    assert!(!validation.would_change);
    let mut changed = first;
    changed.commands.clear();
    assert_eq!(s.submit(changed), Err(CommandError::RequestIdReused));
    assert_unchanged(&s, &before, 3);
}

#[test]
fn rejected_and_no_change_requests_are_also_deduplicated() {
    let mut s = session();
    let failed = request(&s, 1, vec![Command::RemoveDocument { document: did(1) }]);
    let error = s.submit(failed.clone()).unwrap_err();
    let no_change = request(&s, 2, vec![]);
    let no_change_result = s.submit(no_change.clone()).unwrap();
    commit(
        &mut s,
        3,
        vec![Command::CreateDocument { document: doc(1) }],
    );
    let before = s.snapshot();
    assert_eq!(s.submit(failed), Err(error));
    assert_eq!(s.submit(no_change.clone()).unwrap(), no_change_result);
    let mut altered = no_change;
    altered.expected = before.state;
    assert_eq!(s.submit(altered), Err(CommandError::RequestIdReused));
    assert_unchanged(&s, &before, 2);
}

#[test]
fn wrong_session_and_old_api_are_rejected_and_do_not_reserve_request_id() {
    let mut s = session();
    let fresh = request(&s, 1, baseline());
    let before = s.snapshot();
    let mut wrong = fresh.clone();
    wrong.session_id = HistorySessionId::new(71).unwrap();
    assert_eq!(s.submit(wrong), Err(CommandError::WrongSession));
    let mut old = fresh.clone();
    old.api_version = API_VERSION - 1;
    assert_eq!(
        s.submit(old),
        Err(CommandError::UnsupportedVersion(API_VERSION - 1))
    );
    assert_unchanged(&s, &before, 1);
    s.submit(fresh).unwrap();
}

#[test]
fn preflight_is_read_only_and_submission_rechecks_a_changed_revision() {
    let (mut s, calls, _) = session_with(&[1], SessionLimits::default());
    let first = request(&s, 1, baseline());
    let before = s.snapshot();
    let validation = s.validate(&first).unwrap();
    assert!(validation.would_change && validation.resource_check_pending && !validation.cached);
    assert_unchanged(&s, &before, 1);
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    commit(
        &mut s,
        2,
        vec![Command::CreateDocument { document: doc(2) }],
    );
    assert!(matches!(
        s.submit(first),
        Err(CommandError::Conflict { .. })
    ));
    // IDs seen only by the read-only validation remain available.
    commit(&mut s, 3, baseline());
}

#[test]
fn unavailable_resource_rolls_back_and_releases_already_acquired_candidate_pins() {
    let (mut s, calls, probes) = session_with(&[1], SessionLimits::default());
    let before = s.snapshot();
    assert_eq!(Arc::strong_count(&probes[0]), 2);
    let failed = request(
        &s,
        1,
        vec![
            Command::RegisterAsset { asset: asset(1, 1) },
            Command::RegisterAsset { asset: asset(2, 2) },
        ],
    );
    assert_eq!(
        s.submit(failed),
        Err(CommandError::Resource {
            blob: blob(2),
            error: ResourceError::Missing
        })
    );
    assert_eq!(calls.load(Ordering::SeqCst), 2);
    assert_eq!(Arc::strong_count(&probes[0]), 2);
    assert_unchanged(&s, &before, 1);
    commit(
        &mut s,
        2,
        vec![Command::RegisterAsset { asset: asset(1, 1) }],
    );
}

#[test]
fn history_pins_shared_content_once_and_keeps_it_after_current_deletion() {
    let (mut s, calls, probes) = session_with(&[1], SessionLimits::default());
    let added = commit(
        &mut s,
        1,
        vec![
            Command::RegisterAsset { asset: asset(1, 1) },
            Command::RegisterAsset { asset: asset(2, 1) },
        ],
    );
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(Arc::strong_count(&probes[0]), 3);
    commit(
        &mut s,
        2,
        vec![
            Command::RemoveAsset { asset: aid(1) },
            Command::RemoveAsset { asset: aid(2) },
        ],
    );
    assert_eq!(s.snapshot().project.assets().iter().len(), 0);
    assert_eq!(
        s.history_snapshot(added.session_id, added.state.head)
            .unwrap()
            .project
            .assets()
            .iter()
            .len(),
        2
    );
    assert_eq!(Arc::strong_count(&probes[0]), 3);
    drop(s);
    assert_eq!(Arc::strong_count(&probes[0]), 1);
}

#[test]
fn a_store_returning_the_wrong_lease_is_rejected_atomically() {
    let calls = Arc::new(AtomicUsize::new(0));
    let mut data = BTreeMap::new();
    data.insert(
        blob(1),
        Arc::new(Lease {
            reference: blob(2),
            _bytes: vec![2; 4],
        }),
    );
    let mut s = ProjectSession::with_resources(
        7,
        70,
        Box::new(Store { data, calls }),
        SessionLimits::default(),
    )
    .unwrap();
    let before = s.snapshot();
    let bad = request(&s, 1, vec![Command::RegisterAsset { asset: asset(1, 1) }]);
    assert_eq!(
        s.submit(bad),
        Err(CommandError::Resource {
            blob: blob(1),
            error: ResourceError::ReferenceMismatch
        })
    );
    assert_unchanged(&s, &before, 1);
}

#[test]
fn layer_updates_reorder_reparent_and_subtree_delete_preserve_explicit_semantics() {
    let mut s = session();
    commit(&mut s, 1, baseline());
    commit(
        &mut s,
        2,
        vec![
            Command::RegisterAsset { asset: asset(2, 2) },
            Command::AddLayer {
                document: did(1),
                node: group(3),
                parent: None,
                index: 1,
            },
            Command::SetLayerTransform {
                document: did(1),
                layer: lid(3),
                transform: transform(10.0, 20.0),
            },
            Command::MoveLayer {
                document: did(1),
                layer: lid(2),
                parent: Some(lid(3)),
                index: 0,
            },
            Command::SetLayerAsset {
                document: did(1),
                layer: lid(2),
                asset: aid(2),
            },
            Command::SetLayerProperties {
                document: did(1),
                layer: lid(2),
                opacity: Opacity::new(0.5).unwrap(),
                visible: false,
            },
            Command::MoveLayer {
                document: did(1),
                layer: lid(3),
                parent: None,
                index: 0,
            },
            Command::ResizeDocument {
                document: did(1),
                canvas: size(30, 40),
            },
        ],
    );
    let snapshot = s.snapshot();
    let document = snapshot.project.document(did(1)).unwrap();
    assert_eq!(document.canvas(), size(30, 40));
    assert_eq!(document.layers().roots(), &[lid(3), lid(1)]);
    assert_eq!(
        document.layers().world_transform(lid(2)).unwrap(),
        Affine2::IDENTITY
    );
    assert_eq!(
        document.layers().node(lid(2)).unwrap().local,
        transform(-10.0, -20.0)
    );
    assert!(!document.layers().node(lid(2)).unwrap().visible);
    commit(
        &mut s,
        3,
        vec![
            Command::RemoveLayer {
                document: did(1),
                layer: lid(3),
            },
            Command::RemoveAsset { asset: aid(2) },
        ],
    );
    assert!(
        s.snapshot()
            .project
            .document(did(1))
            .unwrap()
            .layers()
            .node(lid(2))
            .is_none()
    );
}

#[test]
fn invalid_layer_moves_cycles_indices_and_asset_deletion_never_leak_partial_work() {
    let mut s = session();
    commit(&mut s, 1, baseline());
    let before = s.snapshot();
    let cases = vec![
        Command::MoveLayer {
            document: did(1),
            layer: lid(1),
            parent: Some(lid(2)),
            index: 0,
        },
        Command::MoveLayer {
            document: did(1),
            layer: lid(2),
            parent: Some(lid(2)),
            index: 0,
        },
        Command::MoveLayer {
            document: did(1),
            layer: lid(2),
            parent: None,
            index: 99,
        },
        Command::AddLayer {
            document: did(1),
            node: group(3),
            parent: Some(lid(2)),
            index: 0,
        },
        Command::RemoveAsset { asset: aid(1) },
    ];
    for (i, command) in cases.into_iter().enumerate() {
        let bad = request(
            &s,
            i as u128 + 2,
            vec![
                Command::RenameAsset {
                    asset: aid(1),
                    name: "temporary".into(),
                },
                command,
            ],
        );
        assert!(s.submit(bad).is_err());
        assert_unchanged(&s, &before, 2);
    }
}

#[test]
fn reparenting_to_a_singular_parent_rejects_the_whole_batch() {
    let mut s = session();
    commit(&mut s, 1, baseline());
    let before = s.snapshot();
    let bad = request(
        &s,
        2,
        vec![
            Command::AddLayer {
                document: did(1),
                node: group(3),
                parent: None,
                index: 1,
            },
            Command::SetLayerTransform {
                document: did(1),
                layer: lid(3),
                transform: Affine2::new([0.0; 6]).unwrap(),
            },
            Command::MoveLayer {
                document: did(1),
                layer: lid(2),
                parent: Some(lid(3)),
                index: 0,
            },
        ],
    );
    assert!(matches!(
        s.submit(bad),
        Err(CommandError::Command {
            index: 2,
            reason: CommandFailure::Model(ModelError::NotInvertible)
        })
    ));
    assert_unchanged(&s, &before, 2);
}

#[test]
fn component_changes_preserve_member_geometry_and_independent_outputs() {
    let mut s = session();
    commit(&mut s, 1, baseline());
    let result = commit(
        &mut s,
        2,
        vec![
            Command::CreateDocument { document: doc(2) },
            Command::AddComponent {
                node: member(3, 2),
                parent: Some(cid(1)),
                index: 1,
            },
            Command::SetOutputMapping {
                mapping: mapping(2, 2, "overlay.png"),
            },
            Command::SetComponentTransform {
                component: cid(1),
                transform: transform(20.0, 10.0),
            },
            Command::SetComponentMapping {
                component: cid(3),
                document_to_component: Affine2::new([2.0, 0.0, 0.0, 2.0, 0.0, 0.0]).unwrap(),
            },
            Command::AddComponent {
                node: cgroup(4),
                parent: None,
                index: 1,
            },
            Command::MoveComponent {
                component: cid(3),
                parent: Some(cid(4)),
                index: 0,
            },
        ],
    );
    assert!(result.changed_objects.contains(&ObjectId::Document(did(1))));
    assert!(result.changed_objects.contains(&ObjectId::Document(did(2))));
    let snapshot = s.snapshot();
    assert_eq!(
        snapshot.project.document_content_transform(did(1)).unwrap(),
        transform(20.0, 10.0)
    );
    assert_eq!(
        snapshot.project.document_content_transform(did(2)).unwrap(),
        transform(10.0, 5.0)
    );
    assert_eq!(snapshot.project.outputs().len(), 2);
    commit(
        &mut s,
        3,
        vec![Command::RemoveComponent { component: cid(4) }],
    );
    assert!(s.snapshot().project.document(did(2)).is_some());
    assert_eq!(s.snapshot().project.outputs().len(), 2);
}

#[test]
fn invalid_component_membership_cycles_and_mappings_roll_back() {
    let mut s = session();
    commit(&mut s, 1, baseline());
    let before = s.snapshot();
    let cases = vec![
        Command::AddComponent {
            node: member(3, 1),
            parent: None,
            index: 1,
        },
        Command::MoveComponent {
            component: cid(1),
            parent: Some(cid(2)),
            index: 0,
        },
        Command::SetComponentMapping {
            component: cid(2),
            document_to_component: Affine2::new([0.0; 6]).unwrap(),
        },
        Command::RemoveDocument { document: did(1) },
    ];
    for (i, command) in cases.into_iter().enumerate() {
        let bad = request(&s, i as u128 + 2, vec![command]);
        assert!(s.submit(bad).is_err());
        assert_unchanged(&s, &before, 2);
    }
}

#[test]
fn configuration_and_output_failures_preserve_unknown_bytes_and_old_node() {
    let mut s = session();
    let added = commit(&mut s, 1, baseline());
    let before = s.snapshot();
    let bad = request(
        &s,
        2,
        vec![
            Command::SetConfigRecord {
                index: 0,
                record: record(b"edited\n"),
            },
            Command::SetOutputMapping {
                mapping: mapping(2, 1, "ELEMENT.PNG"),
            },
        ],
    );
    assert!(s.submit(bad).is_err());
    assert_unchanged(&s, &before, 2);
    commit(
        &mut s,
        3,
        vec![
            Command::SetConfigRecord {
                index: 0,
                record: record(b"changed\n"),
            },
            Command::InsertConfigRecord {
                index: 1,
                record: record(b"\xffuntouched"),
            },
            Command::RemoveConfigRecord { index: 0 },
            Command::SetOutputMapping {
                mapping: mapping(1, 1, "renamed.png"),
            },
        ],
    );
    assert_eq!(
        s.snapshot().project.configuration().to_bytes(),
        b"\xffuntouched"
    );
    assert_eq!(
        s.history_snapshot(added.session_id, added.state.head)
            .unwrap()
            .project,
        before.project
    );
    commit(
        &mut s,
        4,
        vec![Command::CreateDocument { document: doc(2) }],
    );
    let remapped = commit(
        &mut s,
        5,
        vec![Command::SetOutputMapping {
            mapping: mapping(1, 2, "renamed.png"),
        }],
    );
    assert!(
        remapped
            .changed_objects
            .contains(&ObjectId::Document(did(1)))
    );
    assert!(
        remapped
            .changed_objects
            .contains(&ObjectId::Document(did(2)))
    );
}

#[test]
fn deleting_dependencies_in_explicit_order_is_atomic_and_preserves_old_history() {
    let mut s = session();
    let added = commit(&mut s, 1, baseline());
    commit(
        &mut s,
        2,
        vec![
            Command::RemoveComponent { component: cid(1) },
            Command::RemoveOutputMapping { output: oid(1) },
            Command::RemoveDocument { document: did(1) },
            Command::RemoveAsset { asset: aid(1) },
            Command::RemoveConfigRecord { index: 0 },
        ],
    );
    let snapshot = s.snapshot();
    assert_eq!(
        snapshot.project,
        SkinProject::empty(ProjectId::new(7).unwrap())
    );
    assert_eq!(snapshot.state.revision, 2);
    assert_eq!(s.history_len(), 3);
    assert_eq!(
        s.history_snapshot(added.session_id, added.state.head)
            .unwrap()
            .project
            .outputs()
            .len(),
        1
    );
    let reused = request(&s, 3, vec![Command::CreateDocument { document: doc(1) }]);
    assert!(matches!(
        s.submit(reused),
        Err(CommandError::Command {
            reason: CommandFailure::ReusedIdentity(_),
            ..
        })
    ));
}

#[test]
fn history_query_has_explicit_scope_and_never_changes_active_state() {
    let mut s = session();
    commit(&mut s, 1, baseline());
    let before = s.snapshot();
    assert_eq!(
        s.history_snapshot(HistorySessionId::new(71).unwrap(), NodeId::new(1).unwrap()),
        Err(HistoryQueryError::WrongSession)
    );
    assert!(matches!(
        s.history_entry(before.session_id, NodeId::new(99).unwrap()),
        Err(HistoryQueryError::UnknownNode(_))
    ));
    assert_unchanged(&s, &before, 2);
}

#[test]
fn explicit_caps_reject_without_eviction_or_project_publication() {
    let limits = SessionLimits {
        commands_per_request: 1,
        retained_requests: 2,
        history_nodes: 2,
    };
    let (mut s, _, _) = session_with(&[], limits);
    let too_many = request(
        &s,
        1,
        vec![
            Command::CreateDocument { document: doc(1) },
            Command::CreateDocument { document: doc(2) },
        ],
    );
    assert_eq!(
        s.submit(too_many),
        Err(CommandError::ResourceLimit("commands_per_request"))
    );
    let first = request(&s, 1, vec![Command::CreateDocument { document: doc(1) }]);
    let result = s.submit(first.clone()).unwrap();
    let before = s.snapshot();
    let more = request(&s, 2, vec![Command::CreateDocument { document: doc(2) }]);
    assert_eq!(
        s.submit(more),
        Err(CommandError::ResourceLimit("history_nodes"))
    );
    let fresh = request(&s, 3, vec![]);
    assert_eq!(
        s.submit(fresh),
        Err(CommandError::ResourceLimit("retained_requests"))
    );
    assert_eq!(s.submit(first).unwrap(), result);
    assert_unchanged(&s, &before, 2);
}
