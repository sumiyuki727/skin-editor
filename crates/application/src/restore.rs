use crate::archive::{restore_cancel, session_limits};
use crate::capture_identity::PersistenceMap;
use crate::history::{History, Node, Patch};
use crate::values::{AssetKind, BlobRef, SkinProject};
use crate::{
    BlobLease, CaptureCancellation, CapturedAdmission, ExpectedState, HistorySessionId, NodeId,
    ObjectId, OperationOrigin, ProjectSession, ResourceStore, RestoreError, RestoreHeader,
    RestoreLimits, RestoreNode, SessionLimits,
};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

pub struct RestoreBuilder {
    header: RestoreHeader,
    nodes: Vec<RestoreNode>,
    ids: BTreeMap<crate::PersistentNodeId, usize>,
    aliases: BTreeSet<u64>,
    coordinates: BTreeSet<crate::BranchCoordinate>,
    limits: RestoreLimits,
    cancel: CaptureCancellation,
    bytes: usize,
}
/// Fully checked detached history. Activation creates a new session; there is
/// no setter that can overwrite an existing ProjectSession or bypass commands.
pub struct RestoreCandidate {
    header: RestoreHeader,
    nodes: Vec<RestoreNode>,
    pins: BTreeMap<BlobRef, Arc<dyn BlobLease>>,
}
impl std::fmt::Debug for RestoreCandidate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RestoreCandidate")
            .field("nodes", &self.nodes.len())
            .finish_non_exhaustive()
    }
}
pub(crate) fn objects(p: &SkinProject) -> Vec<ObjectId> {
    let mut ids = Vec::new();
    ids.extend(p.assets().iter().map(|a| ObjectId::Asset(a.id())));
    for d in p.documents() {
        ids.push(ObjectId::Document(d.id()));
        ids.extend(d.layers().iter().map(|n| ObjectId::Layer(n.id)));
    }
    ids.extend(p.components().iter().map(|n| ObjectId::Component(n.id)));
    ids.extend(
        p.audio_components()
            .map(|n| ObjectId::AudioComponent(n.id())),
    );
    ids.extend(p.outputs().iter().map(|n| ObjectId::Output(n.id())));
    ids
}
impl RestoreBuilder {
    pub fn new(
        header: RestoreHeader,
        limits: RestoreLimits,
        cancel: CaptureCancellation,
    ) -> Result<Self, RestoreError> {
        restore_cancel(&cancel)?;
        if limits.nodes == 0
            || limits.object_ids == 0
            || limits.metadata_bytes == 0
            || limits.original_bytes == 0
        {
            return Err(RestoreError::Invalid("limits"));
        }
        let namespaces = [
            header.identity.history.get(),
            header.identity.lineage.get(),
            header.identity.address_epoch.get(),
        ];
        if namespaces.into_iter().collect::<BTreeSet<_>>().len() != 3 {
            return Err(RestoreError::Invalid("identity collision"));
        }
        if header.admissions.len() > limits.object_ids || header.node_count > limits.nodes {
            return Err(RestoreError::Limit("identities/nodes"));
        }
        let bytes = header
            .metadata
            .logical_bytes()
            .map_err(|_| RestoreError::Limit("metadata"))?
            .checked_add(
                header
                    .admissions
                    .len()
                    .checked_mul(128)
                    .ok_or(RestoreError::Limit("metadata"))?,
            )
            .and_then(|n| n.checked_add(header.redo_preferences.len().checked_mul(64)?))
            .ok_or(RestoreError::Limit("metadata"))?;
        if bytes > limits.metadata_bytes {
            return Err(RestoreError::Limit("metadata"));
        }
        Ok(Self {
            header,
            nodes: Vec::new(),
            ids: BTreeMap::new(),
            aliases: BTreeSet::new(),
            coordinates: BTreeSet::new(),
            limits,
            cancel,
            bytes,
        })
    }
    pub fn push(&mut self, node: RestoreNode) -> Result<(), RestoreError> {
        restore_cancel(&self.cancel)?;
        let cost = crate::capture_cost::project(&node.project)
            .map_err(|_| RestoreError::Limit("metadata"))?;
        let bytes = self
            .bytes
            .checked_add(cost.checked_mul(3).ok_or(RestoreError::Limit("metadata"))?)
            .and_then(|n| n.checked_add(512 + node.origin.action.len()))
            .ok_or(RestoreError::Limit("metadata"))?;
        if self.nodes.len() >= self.limits.nodes || bytes > self.limits.metadata_bytes {
            return Err(RestoreError::Limit("history"));
        }
        if node.project.id() != self.header.project
            || node.alias == 0
            || [
                self.header.identity.history.get(),
                self.header.identity.lineage.get(),
                self.header.identity.address_epoch.get(),
            ]
            .contains(&node.id.get())
            || self.ids.contains_key(&node.id)
            || self.aliases.contains(&node.alias)
            || self.coordinates.contains(&node.coordinate)
        {
            return Err(RestoreError::Invalid("node identity"));
        }
        if self.nodes.last().is_some_and(|n| n.alias >= node.alias) {
            return Err(RestoreError::Invalid("append order"));
        }
        if let Some(parent) = node.parent {
            let i = *self
                .ids
                .get(&parent)
                .ok_or(RestoreError::Invalid("parent"))?;
            if self.nodes[i].depth.checked_add(1) != Some(node.depth)
                || node.operation.origin == OperationOrigin::Root
            {
                return Err(RestoreError::Invalid("depth/operation"));
            }
        } else if !self.nodes.is_empty()
            || node.id != self.header.root
            || node.alias != 1
            || node.depth != 0
            || node.operation.origin != OperationOrigin::Root
        {
            return Err(RestoreError::Invalid("root"));
        }
        if node.origin.record_version != 2
            || node.origin.action.is_empty()
            || node.operation.version != 1
            || node
                .operation
                .steps
                .iter()
                .any(|s| s.is_empty() || s.len() > 1024)
        {
            return Err(RestoreError::Invalid("operation"));
        }
        self.bytes = bytes;
        self.ids.insert(node.id, self.nodes.len());
        self.aliases.insert(node.alias);
        self.coordinates.insert(node.coordinate);
        self.nodes.push(node);
        Ok(())
    }
    pub fn finish(self, resources: &dyn ResourceStore) -> Result<RestoreCandidate, RestoreError> {
        restore_cancel(&self.cancel)?;
        if self.nodes.len() != self.header.node_count
            || self.nodes.is_empty()
            || !self.ids.contains_key(&self.header.head)
        {
            return Err(RestoreError::Invalid("head/count"));
        }
        let last = self.nodes.last().expect("nonempty").alias;
        let branch = self
            .coordinates
            .iter()
            .map(|c| c.branch())
            .max()
            .expect("nonempty");
        if !counter(self.header.next_alias, last) || !counter(self.header.next_branch, branch) {
            return Err(RestoreError::Invalid("counters"));
        }
        let mut children = vec![Vec::new(); self.nodes.len()];
        for (i, node) in self.nodes.iter().enumerate() {
            if let Some(p) = node.parent {
                children[self.ids[&p]].push(i);
            }
        }
        let mut ranges = vec![(0, 0); self.nodes.len()];
        let mut stack = vec![(0, false)];
        let mut time = 0;
        while let Some((i, exit)) = stack.pop() {
            restore_cancel(&self.cancel)?;
            if exit {
                ranges[i].1 = time;
            } else {
                ranges[i].0 = time;
                time += 1;
                stack.push((i, true));
                stack.extend(children[i].iter().rev().map(|i| (*i, false)));
            }
        }
        let mut admissions = BTreeMap::new();
        for CapturedAdmission { object, first_node } in &self.header.admissions {
            let first = *self
                .ids
                .get(first_node)
                .ok_or(RestoreError::Invalid("first admission"))?;
            if admissions.insert(*object, first).is_some() {
                return Err(RestoreError::Invalid("duplicate admission"));
            }
        }
        let mut refs = BTreeSet::new();
        for (i, node) in self.nodes.iter().enumerate() {
            restore_cancel(&self.cancel)?;
            for object in objects(&node.project) {
                let first = *admissions
                    .get(&object)
                    .ok_or(RestoreError::Invalid("missing admission"))?;
                if !(ranges[first].0 <= ranges[i].0 && ranges[i].1 <= ranges[first].1) {
                    return Err(RestoreError::Invalid("cross branch identity"));
                }
            }
            for a in node.project.assets().iter() {
                refs.insert(a.blob());
                if let AssetKind::Audio(meta) = a.kind()
                    && resources.audio_metadata(a.blob()) != Some(*meta)
                {
                    return Err(RestoreError::Invalid("audio metadata"));
                }
            }
        }
        let declared: BTreeSet<_> = self.header.originals.iter().copied().collect();
        if declared.len() != self.header.originals.len() || declared != refs {
            return Err(RestoreError::Invalid("original closure"));
        }
        let total = refs
            .iter()
            .try_fold(0u64, |sum, b| sum.checked_add(b.byte_len()))
            .ok_or(RestoreError::Limit("originals"))?;
        if total > self.limits.original_bytes {
            return Err(RestoreError::Limit("originals"));
        }
        let mut pins = BTreeMap::new();
        for b in refs {
            restore_cancel(&self.cancel)?;
            let p = resources.pin(b).map_err(|e| RestoreError::Resource(b, e))?;
            if p.reference() != b {
                return Err(RestoreError::Invalid("original lease"));
            }
            pins.insert(b, p);
        }
        let mut preferences = BTreeSet::new();
        for (p, c) in &self.header.redo_preferences {
            let i = *self.ids.get(c).ok_or(RestoreError::Invalid("redo child"))?;
            if self.nodes[i].parent != Some(*p) || !preferences.insert(*p) {
                return Err(RestoreError::Invalid("redo preference"));
            }
        }
        let mut tags = BTreeSet::new();
        for tag in &self.header.metadata.tags {
            if !self.ids.contains_key(&tag.node)
                || !tag.name.starts_with("user:")
                || !tags.insert((tag.node, &tag.name))
            {
                return Err(RestoreError::Invalid("tag"));
            }
        }
        for (_, node) in &self.header.metadata.manual_marks {
            if !self.ids.contains_key(node) {
                return Err(RestoreError::Invalid("manual mark"));
            }
        }
        restore_cancel(&self.cancel)?;
        Ok(RestoreCandidate {
            header: self.header,
            nodes: self.nodes,
            pins,
        })
    }
}
fn counter(next: u64, last: u64) -> bool {
    if last == u64::MAX {
        next == 0
    } else {
        next > last
    }
}
impl RestoreCandidate {
    pub fn header(&self) -> &RestoreHeader {
        &self.header
    }
    pub fn nodes(&self) -> impl ExactSizeIterator<Item = &RestoreNode> {
        self.nodes.iter()
    }
    pub fn into_session(
        self,
        epoch: HistorySessionId,
        resources: Box<dyn ResourceStore>,
        limits: SessionLimits,
        cancel: CaptureCancellation,
    ) -> Result<ProjectSession, RestoreError> {
        if !session_limits(limits) || self.nodes.len() > limits.history_nodes {
            return Err(RestoreError::Limit("session"));
        }
        let mut activation_pins = BTreeMap::new();
        for blob in self.pins.keys() {
            restore_cancel(&cancel)?;
            let p = resources
                .pin(*blob)
                .map_err(|e| RestoreError::Resource(*blob, e))?;
            if p.reference() != *blob {
                return Err(RestoreError::Invalid("activation resource"));
            }
            activation_pins.insert(*blob, p);
        }
        for n in &self.nodes {
            for a in n.project.assets().iter() {
                if let AssetKind::Audio(meta) = a.kind()
                    && resources.audio_metadata(a.blob()) != Some(*meta)
                {
                    return Err(RestoreError::Invalid("activation audio"));
                }
            }
        }
        let mut session = ProjectSession::with_resources(
            self.header.project.get(),
            epoch.get(),
            resources,
            limits,
        )
        .map_err(|_| RestoreError::Invalid("session"))?;
        let local: BTreeMap<_, _> = self
            .nodes
            .iter()
            .map(|n| (n.id, NodeId::new(n.alias).expect("checked")))
            .collect();
        let by_id: BTreeMap<_, _> = self.nodes.iter().map(|n| (n.id, n)).collect();
        let mut history = History::new(self.nodes[0].project.clone());
        history.nodes.clear();
        history.coordinates.clear();
        let mut admitted: BTreeMap<_, Vec<_>> = BTreeMap::new();
        for a in &self.header.admissions {
            admitted.entry(a.first_node).or_default().push(a.object);
        }
        for n in &self.nodes {
            restore_cancel(&cancel)?;
            let patch = n
                .parent
                .map(|p| Arc::new(Patch::between(&by_id[&p].project, &n.project)));
            history.nodes.insert(
                local[&n.id],
                Node {
                    parent: n.parent.map(|p| local[&p]),
                    patch,
                    operation: n.operation.clone(),
                    admitted: admitted.remove(&n.id).unwrap_or_default().into(),
                    origin: Some(n.origin.clone()),
                    request: None,
                    children: Vec::new(),
                    depth: n.depth,
                    coordinate: n.coordinate,
                },
            );
            history.coordinates.insert(n.coordinate, local[&n.id]);
        }
        for n in &self.nodes {
            if let Some(p) = n.parent {
                history
                    .nodes
                    .get_mut(&local[&p])
                    .expect("parent")
                    .children
                    .push(local[&n.id]);
            }
        }
        history.branch_count = self
            .nodes
            .iter()
            .map(|n| n.coordinate.branch())
            .collect::<BTreeSet<_>>()
            .len();
        history.max_depth = self.nodes.iter().map(|n| n.depth).max().unwrap_or(0);
        history.next_id = self.header.next_alias;
        history.next_branch_id = self.header.next_branch;
        session.project = by_id[&self.header.head].project.clone();
        session.history = history;
        session.state = ExpectedState {
            head: local[&self.header.head],
            revision: 0,
        };
        session.redo_preferences = self
            .header
            .redo_preferences
            .iter()
            .map(|(p, c)| (local[p], local[c]))
            .collect();
        session.used_ids = self.header.admissions.iter().map(|a| a.object).collect();
        session.pins = activation_pins;
        session.persistence = Some(PersistenceMap {
            identity: self.header.identity,
            nodes: local.into_iter().map(|(p, l)| (l, p)).collect(),
        });
        session.archive_metadata = self.header.metadata;
        session.restored_history = true;
        restore_cancel(&cancel)?;
        Ok(session)
    }
}
