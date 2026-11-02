use crate::capture_identity::PersistenceMap;
use crate::history::{Patch, definition};
use crate::{
    AddressEpoch, BlobLease, CaptureCancellation, CaptureError, CaptureLimits, CaptureManifest,
    CapturePoll, CaptureRequest, CaptureSource, CaptureStatistics, CapturedAdmission, CapturedNode,
    CapturedNodeInfo, HISTORY_CAPTURE_VERSION, HistoryIdentitySource, LineageId, NodeId,
    PersistentHistoryId, PersistentHistoryIdentity, PersistentNodeId, ProjectSession,
};
use skin_editor_domain::{BlobRef, ProjectDefinition, SkinProject};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

struct Permit(Arc<AtomicBool>);
impl Drop for Permit {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}
struct Resources {
    complete: AtomicBool,
    // Drop real pins before allowing another capture, including on rejection.
    _pins: Vec<Arc<dyn BlobLease>>,
    _permit: Permit,
}

/// Read-only fixed metadata and independent byte lifetime ownership. Cloning it
/// shares both allocations; the single capture slot lasts until the final owner.
#[derive(Clone)]
pub struct CaptureSnapshot {
    manifest: Arc<CaptureManifest>,
    _resources: Arc<Resources>,
}
impl CaptureSnapshot {
    pub fn manifest(&self) -> &CaptureManifest {
        &self.manifest
    }
    /// All captured states were emitted and the traversal validated. This is
    /// not a filesystem commit or proof that a consumer stored every block.
    pub fn is_complete(&self) -> bool {
        self._resources.complete.load(Ordering::Acquire)
    }
}
impl std::fmt::Debug for CaptureSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CaptureSnapshot")
            .field("source", &self.manifest.source)
            .field("nodes", &self.manifest.nodes.len())
            .finish_non_exhaustive()
    }
}

struct FrozenNode {
    patch: Option<Arc<Patch>>,
    children: Vec<usize>,
}
struct Frame {
    node: usize,
    next: usize,
}
struct Frozen {
    nodes: Vec<FrozenNode>,
    current: ProjectDefinition,
    stack: Vec<Frame>,
    root_pending: bool,
    snapshot: CaptureSnapshot,
}

/// Detached, movable traversal; no ProjectSession, ResourceStore or old command
/// executor is retained. Output states are validated, private patches never escape.
pub struct HistoryCapture {
    inner: Option<Frozen>,
    cancellation: CaptureCancellation,
    limits: CaptureLimits,
    statistics: CaptureStatistics,
    terminal: Option<CaptureError>,
    complete: bool,
}
impl std::fmt::Debug for HistoryCapture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HistoryCapture")
            .field("statistics", &self.statistics)
            .finish_non_exhaustive()
    }
}

fn cancelled(c: &CaptureCancellation) -> Result<(), CaptureError> {
    if c.is_cancelled() {
        Err(CaptureError::Cancelled)
    } else {
        Ok(())
    }
}
fn invalid() -> CaptureError {
    CaptureError::History(crate::HistoryQueryError::InvalidHistory(
        crate::values::ModelError::InvalidValue("capture_history"),
    ))
}

impl ProjectSession {
    /// Freezes all branches and their original-byte leases. Identity reservation
    /// is atomic, not a content edit/save mark. No active-state writer is exposed.
    pub fn capture_history(
        &mut self,
        request: CaptureRequest,
        identities: &mut dyn HistoryIdentitySource,
    ) -> Result<HistoryCapture, CaptureError> {
        if request.version != HISTORY_CAPTURE_VERSION {
            return Err(CaptureError::UnsupportedVersion);
        }
        if request.session_id != self.session_id {
            return Err(CaptureError::WrongSession);
        }
        if request.expected != self.state {
            return Err(CaptureError::Conflict {
                expected: request.expected,
                actual: self.state,
            });
        }
        if let Some(d) = &self.draft {
            return Err(CaptureError::Busy(d.state.id));
        }
        if !request.limits.valid() {
            return Err(CaptureError::InvalidLimits);
        }
        cancelled(&request.cancellation)?;
        self.capture_in_flight
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .map_err(|_| CaptureError::InFlight)?;
        let permit = Permit(self.capture_in_flight.clone());
        let limits = request.limits;
        if self.history.nodes.len() > limits.nodes {
            return Err(CaptureError::ResourceLimit("nodes"));
        }
        if self.history.coordinates.len() != self.history.nodes.len() {
            return Err(invalid());
        }
        if self.used_ids.len() > limits.object_ids {
            return Err(CaptureError::ResourceLimit("object_ids"));
        }

        let mut cost = crate::capture_cost::Cost::default();
        cost.count(self.history.nodes.len(), 512)?;
        cost.count(self.used_ids.len(), 128)?;
        cost.count(self.redo_preferences.len(), 64)?;
        cost.add(self.archive_metadata.logical_bytes()?)?;
        cost.count(3, crate::capture_cost::project(&self.history.root)?)?;
        let mut originals = BTreeSet::new();
        for a in self.history.root.assets().iter() {
            originals.insert(a.blob());
        }
        let mut edge_count = 0usize;
        let mut linked = BTreeSet::new();
        for (id, node) in &self.history.nodes {
            cancelled(&request.cancellation)?;
            if self.history.coordinates.get(&node.coordinate) != Some(id) {
                return Err(invalid());
            }
            if let Some(parent) = node.parent {
                if parent >= *id || node.patch.is_none() {
                    return Err(invalid());
                }
                if !self.history.nodes.contains_key(&parent) {
                    return Err(invalid());
                }
                if self.history.nodes[&parent].depth.checked_add(1) != Some(node.depth) {
                    return Err(invalid());
                }
            } else if id.get() != 1 || node.patch.is_some() || node.depth != 0 {
                return Err(invalid());
            }
            for child in &node.children {
                if !linked.insert(*child) {
                    return Err(invalid());
                }
                if self.history.nodes.get(child).and_then(|n| n.parent) != Some(*id) {
                    return Err(invalid());
                }
            }
            edge_count = edge_count
                .checked_add(node.children.len())
                .ok_or(invalid())?;
            cost.count(node.operation.steps.len(), 32)?;
            for step in &node.operation.steps {
                cost.add(step.len())?;
            }
            if let Some(origin) = &node.origin {
                cost.add(origin.action.len())?;
                cost.add(128)?;
            }
            cost.count(node.admitted.len(), 32)?;
            cost.count(node.children.len(), 16)?;
            if let Some(patch) = &node.patch {
                cost.count(2, patch.capture_cost()?)?;
                for blob in patch.originals() {
                    originals.insert(blob);
                    if originals.len() > limits.originals {
                        return Err(CaptureError::ResourceLimit("originals"));
                    }
                }
            }
            if cost.bytes() > limits.retained_metadata_bytes {
                return Err(CaptureError::ResourceLimit("retained_metadata_bytes"));
            }
        }
        if edge_count != self.history.nodes.len().saturating_sub(1) {
            return Err(invalid());
        }
        if !self.history.nodes.contains_key(&self.state.head)
            || !self
                .history
                .nodes
                .contains_key(&NodeId::new(1).expect("root"))
        {
            return Err(invalid());
        }
        cost.count(originals.len(), 128)?;
        if originals.len() > limits.originals || cost.bytes() > limits.retained_metadata_bytes {
            return Err(CaptureError::ResourceLimit("capture_metadata"));
        }
        let original_bytes = originals
            .iter()
            .try_fold(0u64, |sum, b| sum.checked_add(b.byte_len()))
            .filter(|v| *v <= limits.original_bytes)
            .ok_or(CaptureError::ResourceLimit("original_bytes"))?;
        let mut pins = Vec::new();
        let mut previous = None;
        for blob in &originals {
            cancelled(&request.cancellation)?;
            if previous.is_some_and(|old: BlobRef| old.sha256() == blob.sha256()) {
                return Err(CaptureError::InvalidOriginal(*blob));
            }
            let pin = self
                .pins
                .get(blob)
                .ok_or(CaptureError::MissingOriginal(*blob))?;
            if pin.reference() != *blob {
                return Err(CaptureError::InvalidOriginal(*blob));
            }
            pins.push(pin.clone());
            previous = Some(*blob);
        }

        let mapping = self.capture_identities(identities, &request.cancellation)?;
        let index: BTreeMap<_, _> = self
            .history
            .nodes
            .keys()
            .enumerate()
            .map(|(i, id)| (*id, i))
            .collect();
        let mut admissions = BTreeMap::new();
        let mut nodes = Vec::new();
        let mut frozen = Vec::new();
        for (local, node) in &self.history.nodes {
            cancelled(&request.cancellation)?;
            let id = mapping.nodes[local];
            for object in node.admitted.iter() {
                if !self.used_ids.contains(object) || admissions.insert(*object, id).is_some() {
                    return Err(invalid());
                }
            }
            nodes.push(CapturedNodeInfo {
                id,
                parent: node.parent.map(|p| mapping.nodes[&p]),
                alias: local.get(),
                coordinate: node.coordinate,
                depth: node.depth,
                operation: node.operation.clone(),
                changed_objects: node
                    .patch
                    .as_ref()
                    .map_or_else(Vec::new, |p| p.changed.clone()),
                origin: node.origin.clone(),
            });
            frozen.push(FrozenNode {
                patch: node.patch.clone(),
                children: node.children.iter().map(|id| index[id]).collect(),
            });
        }
        if admissions.len() != self.used_ids.len() {
            return Err(invalid());
        }
        let mut redo_preferences = Vec::new();
        for (parent, child) in &self.redo_preferences {
            if self.history.nodes.get(child).and_then(|n| n.parent) != Some(*parent) {
                return Err(invalid());
            }
            redo_preferences.push((mapping.nodes[parent], mapping.nodes[child]));
        }
        let manifest = CaptureManifest {
            version: HISTORY_CAPTURE_VERSION,
            project_id: self.project.id(),
            identity: mapping.identity.clone(),
            source: CaptureSource {
                session_id: self.session_id,
                state: self.state,
                navigation_revision: self.navigation_revision,
            },
            root: mapping.nodes[&NodeId::new(1).expect("root")],
            head: mapping.nodes[&self.state.head],
            nodes,
            admissions: admissions
                .into_iter()
                .map(|(object, first_node)| CapturedAdmission { object, first_node })
                .collect(),
            redo_preferences,
            originals: originals.into_iter().collect(),
            next_alias: self.history.next_id,
            next_branch: self.history.next_branch_id,
            archive_metadata: self.archive_metadata.clone(),
        };
        let current = definition(&self.history.root);
        cancelled(&request.cancellation)?;
        let snapshot = CaptureSnapshot {
            manifest: Arc::new(manifest),
            _resources: Arc::new(Resources {
                complete: AtomicBool::new(false),
                _pins: pins,
                _permit: permit,
            }),
        };
        // Only permanent identity reservations change, never content/navigation.
        self.persistence = Some(mapping);
        Ok(HistoryCapture {
            inner: Some(Frozen {
                nodes: frozen,
                current,
                stack: vec![Frame { node: 0, next: 0 }],
                root_pending: true,
                snapshot,
            }),
            cancellation: request.cancellation,
            limits,
            statistics: CaptureStatistics {
                retained_metadata_bytes: cost.bytes(),
                original_bytes,
                ..Default::default()
            },
            terminal: None,
            complete: false,
        })
    }

    fn capture_identities(
        &self,
        source: &mut dyn HistoryIdentitySource,
        cancellation: &CaptureCancellation,
    ) -> Result<PersistenceMap, CaptureError> {
        let mut used = BTreeSet::new();
        if let Some(old) = &self.persistence {
            used.extend([
                old.identity.history.get(),
                old.identity.lineage.get(),
                old.identity.address_epoch.get(),
            ]);
            used.extend(old.nodes.values().map(|id| id.get()));
        }
        let mut fresh = || {
            cancelled(cancellation)?;
            let value = source
                .next_identity()
                .map_err(|_| CaptureError::IdentityUnavailable)?;
            if value == 0 {
                return Err(CaptureError::InvalidIdentity);
            }
            if !used.insert(value) {
                return Err(CaptureError::IdentityCollision);
            }
            Ok(value)
        };
        let mut mapping = match &self.persistence {
            Some(old) => old.clone(),
            None => PersistenceMap {
                identity: PersistentHistoryIdentity {
                    history: PersistentHistoryId::new(fresh()?).expect("nonzero"),
                    lineage: LineageId::new(fresh()?).expect("nonzero"),
                    address_epoch: AddressEpoch::new(fresh()?).expect("nonzero"),
                },
                nodes: BTreeMap::new(),
            },
        };
        for id in self.history.nodes.keys() {
            if let std::collections::btree_map::Entry::Vacant(entry) = mapping.nodes.entry(*id) {
                entry.insert(PersistentNodeId::new(fresh()?).expect("nonzero"));
            }
        }
        cancelled(cancellation)?;
        Ok(mapping)
    }
}

impl HistoryCapture {
    pub fn statistics(&self) -> CaptureStatistics {
        self.statistics
    }
    pub fn snapshot(&self) -> Result<CaptureSnapshot, CaptureError> {
        cancelled(&self.cancellation)?;
        self.inner
            .as_ref()
            .map(|f| f.snapshot.clone())
            .ok_or(CaptureError::Terminated)
    }
    /// Idempotent, has no receipt/queue admission requirement. Returned nodes
    /// keep their own ownership; cancellation cannot revoke consumers' leases.
    pub fn cancel(&mut self) {
        if self.inner.is_some() {
            self.cancellation.cancel();
            self.abort(CaptureError::Cancelled);
        }
    }
    fn abort(&mut self, error: CaptureError) {
        self.inner = None;
        self.terminal = Some(error);
    }
    /// At most 1024 edge transitions per call. Pending permits cooperative
    /// scheduling while backing out of a deep branch; no recursion or sleeps.
    pub fn poll(&mut self, max_edges: usize) -> Result<CapturePoll, CaptureError> {
        if self.complete {
            return Ok(CapturePoll::Complete);
        }
        if let Some(error) = &self.terminal {
            return Err(error.clone());
        }
        if max_edges == 0 || max_edges > 1024 {
            self.abort(CaptureError::InvalidWorkLimit);
            return Err(CaptureError::InvalidWorkLimit);
        }
        let result = self.poll_once(max_edges);
        if let Err(error) = &result {
            self.abort(error.clone());
        }
        if matches!(result, Ok(CapturePoll::Complete)) {
            if self.statistics.nodes_emitted != self.inner.as_ref().expect("active").nodes.len() {
                self.abort(invalid());
                return Err(invalid());
            }
            self.inner
                .as_ref()
                .expect("active")
                .snapshot
                ._resources
                .complete
                .store(true, Ordering::Release);
            self.inner = None;
            self.complete = true;
        }
        result
    }
    fn poll_once(&mut self, max_edges: usize) -> Result<CapturePoll, CaptureError> {
        cancelled(&self.cancellation)?;
        let f = self.inner.as_mut().ok_or(CaptureError::Terminated)?;
        if f.root_pending {
            f.root_pending = false;
            return self.emit(0);
        }
        for _ in 0..max_edges {
            cancelled(&self.cancellation)?;
            let Some(frame) = f.stack.last_mut() else {
                return Ok(CapturePoll::Complete);
            };
            if let Some(child) = f.nodes[frame.node].children.get(frame.next).copied() {
                frame.next += 1;
                f.nodes[child]
                    .patch
                    .as_ref()
                    .ok_or_else(invalid)?
                    .apply_definition(&mut f.current, true);
                self.statistics.forward_edges += 1;
                f.stack.push(Frame {
                    node: child,
                    next: 0,
                });
                return self.emit(child);
            }
            let node = f.stack.pop().expect("frame").node;
            if f.stack.is_empty() {
                return Ok(CapturePoll::Complete);
            }
            f.nodes[node]
                .patch
                .as_ref()
                .ok_or_else(invalid)?
                .apply_definition(&mut f.current, false);
            self.statistics.reverse_edges += 1;
        }
        Ok(CapturePoll::Pending)
    }
    fn emit(&mut self, index: usize) -> Result<CapturePoll, CaptureError> {
        let f = self.inner.as_ref().ok_or(CaptureError::Terminated)?;
        let bytes = crate::capture_cost::definition(&f.current)?;
        let total = self
            .statistics
            .state_bytes
            .checked_add(bytes)
            .ok_or(CaptureError::ResourceLimit("state_bytes"))?;
        if bytes > self.limits.state_bytes || total > self.limits.total_state_bytes {
            return Err(CaptureError::ResourceLimit("state_bytes"));
        }
        let project = SkinProject::from_definition(f.current.clone())
            .map_err(|e| CaptureError::History(crate::HistoryQueryError::InvalidHistory(e)))?;
        cancelled(&self.cancellation)?;
        self.statistics.nodes_emitted += 1;
        self.statistics.state_bytes = total;
        Ok(CapturePoll::Node(Box::new(CapturedNode {
            project,
            snapshot: f.snapshot.clone(),
            index,
        })))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{API_VERSION, Command, CommandRequest, IdentityError, RequestId, values::*};
    struct Ids(u128);
    impl HistoryIdentitySource for Ids {
        fn next_identity(&mut self) -> Result<u128, IdentityError> {
            self.0 += 1;
            Ok(self.0)
        }
    }
    fn session() -> ProjectSession {
        let mut s = ProjectSession::new(7, 70).unwrap();
        let i = s.project_info();
        s.submit(CommandRequest {
            api_version: API_VERSION,
            session_id: i.session_id,
            request_id: RequestId::new(1).unwrap(),
            expected: i.state,
            commands: vec![Command::InsertConfigRecord {
                index: 0,
                record: ConfigRecord::new(b"immutable".to_vec(), ConfigInterpretation::Opaque)
                    .unwrap(),
            }],
        })
        .unwrap();
        s
    }
    fn request(s: &ProjectSession) -> CaptureRequest {
        CaptureRequest {
            version: HISTORY_CAPTURE_VERSION,
            session_id: s.session_id,
            expected: s.state,
            limits: CaptureLimits::default(),
            cancellation: CaptureCancellation::default(),
        }
    }
    #[test]
    fn corrupt_topology_is_rejected_without_omitting_nodes_or_publishing_identities() {
        let mut s = session();
        let root = NodeId::new(1).unwrap();
        let head = s.state.head;
        s.history.nodes.get_mut(&root).unwrap().children.push(head);
        let before = s.snapshot();
        assert!(matches!(
            s.capture_history(request(&s), &mut Ids(1000)),
            Err(CaptureError::History(_))
        ));
        assert!(s.persistence.is_none());
        assert!(!s.capture_in_flight.load(Ordering::Acquire));
        assert_eq!(s.snapshot(), before);
        s.history.nodes.get_mut(&root).unwrap().children.pop();
        let mut c = s.capture_history(request(&s), &mut Ids(1000)).unwrap();
        c.cancel();
    }
    #[test]
    fn worker_releases_shared_patch_payloads_on_cancel_and_terminal_failure() {
        for fail in [false, true] {
            let mut s = session();
            let weak = Arc::downgrade(s.history.nodes[&s.state.head].patch.as_ref().unwrap());
            let mut r = request(&s);
            if fail {
                r.limits.state_bytes = 1;
            }
            let mut c = s.capture_history(r, &mut Ids(1000)).unwrap();
            drop(s);
            assert!(weak.upgrade().is_some());
            if fail {
                assert!(c.poll(1).is_err());
            } else {
                c.cancel();
            }
            assert!(weak.upgrade().is_none());
        }
    }
    #[test]
    fn missing_or_changed_original_lease_rejects_before_identity_publication() {
        struct Lease(BlobRef, AtomicBool);
        impl BlobLease for Lease {
            fn reference(&self) -> BlobRef {
                if self.1.load(Ordering::Acquire) {
                    self.0
                } else {
                    BlobRef::new([99; 32], 1)
                }
            }
        }
        struct Store(Arc<Lease>);
        impl crate::ResourceStore for Store {
            fn pin(&self, _: BlobRef) -> Result<Arc<dyn BlobLease>, crate::ResourceError> {
                Ok(self.0.clone())
            }
        }
        let blob = BlobRef::new([1; 32], 10);
        let lease = Arc::new(Lease(blob, AtomicBool::new(true)));
        let mut s = ProjectSession::with_resources(
            7,
            70,
            Box::new(Store(lease.clone())),
            crate::SessionLimits::default(),
        )
        .unwrap();
        let i = s.project_info();
        s.submit(CommandRequest {
            api_version: API_VERSION,
            session_id: i.session_id,
            request_id: RequestId::new(1).unwrap(),
            expected: i.state,
            commands: vec![Command::RegisterAsset {
                asset: Asset::new(
                    AssetId::new(1).unwrap(),
                    "original".into(),
                    blob,
                    AssetKind::Opaque,
                )
                .unwrap(),
            }],
        })
        .unwrap();
        let old = s.snapshot();
        let pin = s.pins.remove(&blob).unwrap();
        assert_eq!(
            s.capture_history(request(&s), &mut Ids(1000)).unwrap_err(),
            CaptureError::MissingOriginal(blob)
        );
        s.pins.insert(blob, pin);
        lease.1.store(false, Ordering::Release);
        let strong = Arc::strong_count(&lease);
        assert_eq!(
            s.capture_history(request(&s), &mut Ids(1000)).unwrap_err(),
            CaptureError::InvalidOriginal(blob)
        );
        assert_eq!(Arc::strong_count(&lease), strong);
        assert!(s.persistence.is_none());
        assert_eq!(s.snapshot(), old);
        assert!(!s.capture_in_flight.load(Ordering::Acquire));
        lease.1.store(true, Ordering::Release);
        let mut c = s.capture_history(request(&s), &mut Ids(1000)).unwrap();
        let snapshot = c.snapshot().unwrap();
        c.cancel();
        assert!(!snapshot.is_complete());
        drop(snapshot);
    }
}
