use std::collections::BTreeMap;
use std::sync::Arc;

use skin_editor_domain::{
    Asset, AssetId, AudioComponent, AudioComponentId, ComponentTree, DocumentId, ElementDocument,
    ModelError, ObjectId, OutputMapping, ProjectDefinition, SkinConfiguration, SkinProject,
};

use crate::{
    API_VERSION, BranchCoordinate, CommandError, HistoryEntry, HistoryQueryError, HistorySessionId,
    HistorySummary, NodeId, RequestId,
};

pub(crate) fn definition(project: &SkinProject) -> ProjectDefinition {
    ProjectDefinition {
        model_version: project.model_version(),
        id: project.id(),
        assets: project.assets().iter().cloned().collect(),
        documents: project.documents().cloned().collect(),
        audio_components: project.audio_components().cloned().collect(),
        components: project.components().clone(),
        configuration: project.configuration().clone(),
        outputs: project.outputs().to_vec(),
    }
}

#[derive(Clone)]
struct Change<I, V> {
    id: I,
    before: Option<V>,
    after: Option<V>,
}

fn changes<I: Copy + Ord, V: Clone + PartialEq>(
    before: BTreeMap<I, V>,
    after: BTreeMap<I, V>,
) -> Vec<Change<I, V>> {
    let mut ids: Vec<_> = before.keys().chain(after.keys()).copied().collect();
    ids.sort();
    ids.dedup();
    ids.into_iter()
        .filter_map(|id| {
            let old = before.get(&id);
            let new = after.get(&id);
            (old != new).then(|| Change {
                id,
                before: old.cloned(),
                after: new.cloned(),
            })
        })
        .collect()
}

/// Reversible changed-metadata patch. Unchanged documents/assets and all raw
/// image bytes are absent. Document-sized patches are a measured P1-4 concern.
#[derive(Clone)]
pub(crate) struct Patch {
    assets: Vec<Change<AssetId, Asset>>,
    documents: Vec<Change<DocumentId, ElementDocument>>,
    audio_components: Vec<Change<AudioComponentId, AudioComponent>>,
    components: Option<(ComponentTree, ComponentTree)>,
    configuration: Option<(SkinConfiguration, SkinConfiguration)>,
    outputs: Option<(Vec<OutputMapping>, Vec<OutputMapping>)>,
    pub changed: Vec<ObjectId>,
}

impl Patch {
    pub(crate) fn originals(&self) -> impl Iterator<Item = skin_editor_domain::BlobRef> + '_ {
        self.assets
            .iter()
            .flat_map(|c| c.before.iter().chain(c.after.iter()))
            .map(Asset::blob)
    }

    pub(crate) fn capture_cost(&self) -> Result<usize, crate::CaptureError> {
        let mut c = crate::capture_cost::Cost::default();
        c.count(self.changed.len(), 64)?;
        for v in &self.assets {
            for a in v.before.iter().chain(v.after.iter()) {
                c.asset(a)?;
            }
        }
        for v in &self.documents {
            for d in v.before.iter().chain(v.after.iter()) {
                c.document(d)?;
            }
        }
        for v in &self.audio_components {
            for a in v.before.iter().chain(v.after.iter()) {
                c.sound(a)?;
            }
        }
        if let Some((a, b)) = &self.components {
            c.components(a)?;
            c.components(b)?;
        }
        if let Some((a, b)) = &self.configuration {
            c.configuration(a)?;
            c.configuration(b)?;
        }
        if let Some((a, b)) = &self.outputs {
            c.outputs(a)?;
            c.outputs(b)?;
        }
        Ok(c.bytes())
    }

    pub fn between(old: &SkinProject, new: &SkinProject) -> Self {
        let assets = changes(
            old.assets().iter().map(|a| (a.id(), a.clone())).collect(),
            new.assets().iter().map(|a| (a.id(), a.clone())).collect(),
        );
        let documents = changes(
            old.documents().map(|d| (d.id(), d.clone())).collect(),
            new.documents().map(|d| (d.id(), d.clone())).collect(),
        );
        let components = (old.components() != new.components())
            .then(|| (old.components().clone(), new.components().clone()));
        let audio_components = changes(
            old.audio_components()
                .map(|c| (c.id(), c.clone()))
                .collect(),
            new.audio_components()
                .map(|c| (c.id(), c.clone()))
                .collect(),
        );
        let configuration = (old.configuration() != new.configuration())
            .then(|| (old.configuration().clone(), new.configuration().clone()));
        let outputs = (old.outputs() != new.outputs())
            .then(|| (old.outputs().to_vec(), new.outputs().to_vec()));
        let mut changed: Vec<_> = assets
            .iter()
            .map(|c| ObjectId::Asset(c.id))
            .chain(documents.iter().map(|c| ObjectId::Document(c.id)))
            .chain(
                audio_components
                    .iter()
                    .map(|c| ObjectId::AudioComponent(c.id)),
            )
            .collect();
        for change in &documents {
            for doc in change.before.iter().chain(change.after.iter()) {
                changed.extend(doc.layers().iter().map(|n| ObjectId::Layer(n.id)));
            }
        }
        if let Some((a, b)) = &components {
            changed.extend(a.iter().chain(b.iter()).map(|n| ObjectId::Component(n.id)));
            changed.extend(a.iter().chain(b.iter()).filter_map(|node| match node.kind {
                skin_editor_domain::ComponentKind::Element { document, .. } => {
                    Some(ObjectId::Document(document))
                }
                _ => None,
            }));
        }
        if let Some((a, b)) = &outputs {
            changed.extend(a.iter().chain(b.iter()).map(|n| ObjectId::Output(n.id())));
            changed.extend(
                a.iter()
                    .chain(b.iter())
                    .map(|mapping| mapping.source().object_id()),
            );
        }
        changed.sort();
        changed.dedup();
        Self {
            assets,
            documents,
            audio_components,
            components,
            configuration,
            outputs,
            changed,
        }
    }

    pub fn configuration_changed(&self) -> bool {
        self.configuration.is_some()
    }

    #[cfg(test)]
    pub fn apply(&self, project: &SkinProject, forward: bool) -> Result<SkinProject, ModelError> {
        let mut input = definition(project);
        self.apply_definition(&mut input, forward);
        SkinProject::from_definition(input)
    }

    pub(crate) fn apply_definition(&self, input: &mut ProjectDefinition, forward: bool) {
        for change in &self.assets {
            input.assets.retain(|a| a.id() != change.id);
            input.assets.extend(
                if forward {
                    &change.after
                } else {
                    &change.before
                }
                .iter()
                .cloned(),
            );
        }
        for change in &self.documents {
            input.documents.retain(|d| d.id() != change.id);
            input.documents.extend(
                if forward {
                    &change.after
                } else {
                    &change.before
                }
                .iter()
                .cloned(),
            );
        }
        for change in &self.audio_components {
            input.audio_components.retain(|c| c.id() != change.id);
            input.audio_components.extend(
                if forward {
                    &change.after
                } else {
                    &change.before
                }
                .iter()
                .cloned(),
            );
        }
        if let Some((old, new)) = &self.components {
            input.components = if forward { new } else { old }.clone();
        }
        if let Some((old, new)) = &self.configuration {
            input.configuration = if forward { new } else { old }.clone();
        }
        if let Some((old, new)) = &self.outputs {
            input.outputs = if forward { new } else { old }.clone();
        }
    }
}

pub(crate) struct Node {
    pub parent: Option<NodeId>,
    pub patch: Option<Arc<Patch>>,
    pub operation: Arc<crate::CapturedOperation>,
    pub admitted: Arc<[ObjectId]>,
    pub request: Option<RequestId>,
    pub children: Vec<NodeId>,
    pub depth: u64,
    pub coordinate: BranchCoordinate,
}
pub(crate) struct History {
    pub root: SkinProject,
    pub nodes: BTreeMap<NodeId, Node>,
    pub next_id: u64,
    pub coordinates: BTreeMap<BranchCoordinate, NodeId>,
    pub next_branch_id: u64,
    pub branch_count: usize,
    pub max_depth: u64,
}

pub(crate) struct AppendPlan {
    pub id: NodeId,
    parent: NodeId,
    depth: u64,
    coordinate: BranchCoordinate,
    new_branch: bool,
}

impl History {
    pub fn new(root: SkinProject) -> Self {
        let root_id = NodeId::new(1).expect("nonzero root");
        let coordinate = BranchCoordinate::new(1, 0).expect("nonzero branch");
        let mut nodes = BTreeMap::new();
        nodes.insert(
            root_id,
            Node {
                parent: None,
                patch: None,
                operation: Arc::new(crate::CapturedOperation::root()),
                admitted: Arc::from([]),
                request: None,
                children: Vec::new(),
                depth: 0,
                coordinate,
            },
        );
        Self {
            root,
            nodes,
            next_id: 2,
            coordinates: BTreeMap::from([(coordinate, root_id)]),
            next_branch_id: 2,
            branch_count: 1,
            max_depth: 0,
        }
    }

    pub fn plan_append(&self, parent: NodeId) -> Result<AppendPlan, CommandError> {
        let id = NodeId::new(self.next_id).ok_or(CommandError::ResourceLimit("node_id"))?;
        let old = self.nodes.get(&parent).ok_or(CommandError::History(
            HistoryQueryError::UnknownNode(parent),
        ))?;
        let depth = old
            .depth
            .checked_add(1)
            .ok_or(CommandError::ResourceLimit("history_depth"))?;
        let new_branch = !old.children.is_empty();
        let coordinate = if new_branch {
            BranchCoordinate::new(self.next_branch_id, 0)
                .ok_or(CommandError::ResourceLimit("branch_id"))?
        } else {
            let ordinal = old
                .coordinate
                .ordinal()
                .checked_add(1)
                .ok_or(CommandError::ResourceLimit("branch_ordinal"))?;
            BranchCoordinate::new(old.coordinate.branch(), ordinal)
                .expect("existing nonzero branch")
        };
        if self.coordinates.contains_key(&coordinate) {
            return Err(CommandError::History(HistoryQueryError::InvalidHistory(
                ModelError::InvalidValue("duplicate_history_coordinate"),
            )));
        }
        Ok(AppendPlan {
            id,
            parent,
            depth,
            coordinate,
            new_branch,
        })
    }

    /// Called only after all validation and resource pinning succeeds.
    #[cfg(test)]
    pub fn append(&mut self, plan: AppendPlan, patch: Patch, request: RequestId) {
        self.append_record(
            plan,
            patch,
            request,
            crate::CapturedOperation::root(),
            Vec::new(),
        );
    }

    pub fn append_record(
        &mut self,
        plan: AppendPlan,
        patch: Patch,
        request: RequestId,
        operation: crate::CapturedOperation,
        admitted: Vec<ObjectId>,
    ) {
        self.nodes
            .get_mut(&plan.parent)
            .expect("validated parent")
            .children
            .push(plan.id);
        self.coordinates.insert(plan.coordinate, plan.id);
        self.nodes.insert(
            plan.id,
            Node {
                parent: Some(plan.parent),
                patch: Some(Arc::new(patch)),
                operation: Arc::new(operation),
                admitted: admitted.into(),
                request: Some(request),
                children: Vec::new(),
                depth: plan.depth,
                coordinate: plan.coordinate,
            },
        );
        self.next_id = plan.id.get().checked_add(1).unwrap_or(0);
        if plan.new_branch {
            self.next_branch_id = plan.coordinate.branch().checked_add(1).unwrap_or(0);
            self.branch_count += 1;
        }
        self.max_depth = self.max_depth.max(plan.depth);
    }

    pub fn summary(&self, id: NodeId) -> Result<HistorySummary, HistoryQueryError> {
        let node = self.node(id)?;
        Ok(HistorySummary {
            id,
            parent: node.parent,
            depth: node.depth,
            coordinate: node.coordinate,
            child_count: node.children.len(),
        })
    }

    pub fn node(&self, id: NodeId) -> Result<&Node, HistoryQueryError> {
        self.nodes
            .get(&id)
            .ok_or(HistoryQueryError::UnknownNode(id))
    }

    fn parent(&self, id: NodeId) -> Result<NodeId, HistoryQueryError> {
        self.node(id)?
            .parent
            .ok_or(HistoryQueryError::InvalidHistory(ModelError::InvalidValue(
                "history_parent",
            )))
    }
    pub fn entry(&self, session_id: HistorySessionId, id: NodeId) -> Option<HistoryEntry> {
        let node = self.nodes.get(&id)?;
        Some(HistoryEntry {
            api_version: API_VERSION,
            session_id,
            id,
            parent: node.parent,
            request_id: node.request,
            changed_objects: node
                .patch
                .as_ref()
                .map_or_else(Vec::new, |p| p.changed.clone()),
            configuration_changed: node
                .patch
                .as_ref()
                .is_some_and(|p| p.configuration_changed()),
        })
    }
    pub fn materialize(&self, id: NodeId) -> Result<SkinProject, HistoryQueryError> {
        self.transition(NodeId::new(1).expect("root"), id, &self.root)
    }

    /// Restore through the nearest common ancestor without publishing interim
    /// states or executing old commands/resource acquisition again.
    pub fn transition(
        &self,
        from: NodeId,
        to: NodeId,
        current: &SkinProject,
    ) -> Result<SkinProject, HistoryQueryError> {
        let mut left = from;
        let mut right = to;
        let mut undo = Vec::new();
        let mut redo = Vec::new();
        self.node(from)?;
        self.node(to)?;
        while left != right {
            if undo.len() > self.nodes.len() || redo.len() > self.nodes.len() {
                return Err(HistoryQueryError::InvalidHistory(ModelError::Cycle));
            }
            let left_depth = self.node(left)?.depth;
            let right_depth = self.node(right)?.depth;
            if left_depth >= right_depth {
                undo.push(left);
                left = self.parent(left)?;
            }
            if right_depth >= left_depth {
                redo.push(right);
                right = self.parent(right)?;
            }
        }
        let mut candidate = definition(current);
        for (id, forward) in undo
            .into_iter()
            .map(|id| (id, false))
            .chain(redo.into_iter().rev().map(|id| (id, true)))
        {
            let patch = self
                .node(id)?
                .patch
                .as_ref()
                .ok_or(HistoryQueryError::InvalidHistory(ModelError::InvalidValue(
                    "history_patch",
                )))?;
            patch.apply_definition(&mut candidate, forward);
        }
        // Published patches were already validated when committed. Replay is
        // private: validate the assembled destination once before exposing it.
        SkinProject::from_definition(candidate).map_err(HistoryQueryError::InvalidHistory)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use skin_editor_domain::{
        Affine2, AssetKind, BlobRef, ComponentId, ComponentKind, ComponentNode,
        ConfigInterpretation, ConfigRecord, LayerId, LayerKind, LayerNode, LayerTree, Opacity,
        OutputId, OutputPath, PixelSize, ProjectId,
    };

    #[test]
    fn invalid_restored_project_is_rejected_even_when_all_node_patches_exist() {
        let root = SkinProject::empty(ProjectId::new(1).unwrap());
        let document = ElementDocument::new(
            DocumentId::new(1).unwrap(),
            PixelSize::new(1, 1).unwrap(),
            LayerTree::new(
                vec![LayerId::new(1).unwrap()],
                vec![LayerNode {
                    id: LayerId::new(1).unwrap(),
                    local: Affine2::IDENTITY,
                    opacity: Opacity::OPAQUE,
                    visible: true,
                    kind: LayerKind::Image {
                        asset: AssetId::new(1).unwrap(),
                    },
                }],
            )
            .unwrap(),
        );
        let mut patch = Patch::between(&root, &root);
        patch.documents.push(Change {
            id: document.id(),
            before: None,
            after: Some(document),
        });
        let mut history = History::new(root.clone());
        let plan = history.plan_append(NodeId::new(1).unwrap()).unwrap();
        let id = plan.id;
        history.append(plan, patch, RequestId::new(1).unwrap());
        assert!(matches!(
            history.materialize(id),
            Err(HistoryQueryError::InvalidHistory(_))
        ));
        assert_eq!(history.root, root);
    }

    #[test]
    fn depth_ordinal_and_branch_exhaustion_reject_before_appending() {
        let root = NodeId::new(1).unwrap();
        let mut history = History::new(SkinProject::empty(ProjectId::new(1).unwrap()));
        history.nodes.get_mut(&root).unwrap().depth = u64::MAX;
        assert!(matches!(
            history.plan_append(root),
            Err(CommandError::ResourceLimit("history_depth"))
        ));
        history.nodes.get_mut(&root).unwrap().depth = 0;
        history.nodes.get_mut(&root).unwrap().coordinate =
            BranchCoordinate::new(1, u64::MAX).unwrap();
        assert!(matches!(
            history.plan_append(root),
            Err(CommandError::ResourceLimit("branch_ordinal"))
        ));
        history.nodes.get_mut(&root).unwrap().coordinate = BranchCoordinate::new(1, 0).unwrap();
        let plan = history.plan_append(root).unwrap();
        history.append(
            plan,
            Patch::between(&history.root, &history.root),
            RequestId::new(1).unwrap(),
        );
        history.next_branch_id = u64::MAX;
        let plan = history.plan_append(root).unwrap();
        history.append(
            plan,
            Patch::between(&history.root, &history.root),
            RequestId::new(2).unwrap(),
        );
        let children = history.nodes[&root].children.clone();
        let index = history.coordinates.clone();
        assert!(matches!(
            history.plan_append(root),
            Err(CommandError::ResourceLimit("branch_id"))
        ));
        assert_eq!(history.nodes[&root].children, children);
        assert_eq!(history.coordinates, index);
        assert_eq!(history.nodes.len(), 3);
        assert_eq!(history.next_id, 4);
    }

    #[test]
    fn patches_round_trip_assets_documents_components_outputs_and_configuration_together() {
        let empty = SkinProject::empty(ProjectId::new(1).unwrap());
        let mut input = definition(&empty);
        let asset_id = AssetId::new(1).unwrap();
        let doc_id = DocumentId::new(1).unwrap();
        let layer_id = LayerId::new(1).unwrap();
        let component_id = ComponentId::new(1).unwrap();
        let size = PixelSize::new(2, 2).unwrap();
        input.assets.push(
            Asset::new(
                asset_id,
                "source".into(),
                BlobRef::new([1; 32], 4),
                AssetKind::RasterImage(size),
            )
            .unwrap(),
        );
        input.documents.push(ElementDocument::new(
            doc_id,
            size,
            LayerTree::new(
                vec![layer_id],
                vec![LayerNode {
                    id: layer_id,
                    local: Affine2::IDENTITY,
                    opacity: Opacity::OPAQUE,
                    visible: true,
                    kind: LayerKind::Image { asset: asset_id },
                }],
            )
            .unwrap(),
        ));
        input.components = ComponentTree::new(
            vec![component_id],
            vec![ComponentNode {
                id: component_id,
                local: Affine2::IDENTITY,
                kind: ComponentKind::Element {
                    document: doc_id,
                    document_to_component: Affine2::IDENTITY,
                },
            }],
        )
        .unwrap();
        input.outputs.push(
            OutputMapping::new(
                OutputId::new(1).unwrap(),
                doc_id,
                "explicit".into(),
                OutputPath::new("test.png".into()).unwrap(),
            )
            .unwrap(),
        );
        input.configuration = SkinConfiguration::new(vec![
            ConfigRecord::new(b"unknown\r\n\xff".to_vec(), ConfigInterpretation::Opaque).unwrap(),
        ]);
        let populated = SkinProject::from_definition(input).unwrap();
        let add = Patch::between(&empty, &populated);
        assert_eq!(add.apply(&empty, true).unwrap(), populated);
        assert_eq!(add.apply(&populated, false).unwrap(), empty);
        let remove = Patch::between(&populated, &empty);
        assert_eq!(remove.apply(&populated, true).unwrap(), empty);
        assert_eq!(remove.apply(&empty, false).unwrap(), populated);
    }
}
