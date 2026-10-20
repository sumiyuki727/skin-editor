use std::collections::BTreeMap;

use skin_editor_domain::{
    Asset, AssetId, ComponentTree, DocumentId, ElementDocument, ModelError, ObjectId,
    OutputMapping, ProjectDefinition, SkinConfiguration, SkinProject,
};

use crate::{API_VERSION, HistoryEntry, HistoryQueryError, HistorySessionId, NodeId, RequestId};

pub(crate) fn definition(project: &SkinProject) -> ProjectDefinition {
    ProjectDefinition {
        model_version: project.model_version(),
        id: project.id(),
        assets: project.assets().iter().cloned().collect(),
        documents: project.documents().cloned().collect(),
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
    components: Option<(ComponentTree, ComponentTree)>,
    configuration: Option<(SkinConfiguration, SkinConfiguration)>,
    outputs: Option<(Vec<OutputMapping>, Vec<OutputMapping>)>,
    pub changed: Vec<ObjectId>,
}

impl Patch {
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
        let configuration = (old.configuration() != new.configuration())
            .then(|| (old.configuration().clone(), new.configuration().clone()));
        let outputs = (old.outputs() != new.outputs())
            .then(|| (old.outputs().to_vec(), new.outputs().to_vec()));
        let mut changed: Vec<_> = assets
            .iter()
            .map(|c| ObjectId::Asset(c.id))
            .chain(documents.iter().map(|c| ObjectId::Document(c.id)))
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
                    .map(|mapping| ObjectId::Document(mapping.document())),
            );
        }
        changed.sort();
        changed.dedup();
        Self {
            assets,
            documents,
            components,
            configuration,
            outputs,
            changed,
        }
    }

    pub fn configuration_changed(&self) -> bool {
        self.configuration.is_some()
    }

    pub fn apply(&self, project: &SkinProject, forward: bool) -> Result<SkinProject, ModelError> {
        let mut input = definition(project);
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
        if let Some((old, new)) = &self.components {
            input.components = if forward { new } else { old }.clone();
        }
        if let Some((old, new)) = &self.configuration {
            input.configuration = if forward { new } else { old }.clone();
        }
        if let Some((old, new)) = &self.outputs {
            input.outputs = if forward { new } else { old }.clone();
        }
        SkinProject::from_definition(input)
    }
}

pub(crate) struct Node {
    pub parent: Option<NodeId>,
    pub patch: Option<Patch>,
    pub request: Option<RequestId>,
}
pub(crate) struct History {
    pub root: SkinProject,
    pub nodes: BTreeMap<NodeId, Node>,
    pub next_id: u64,
}

impl History {
    pub fn new(root: SkinProject) -> Self {
        let mut nodes = BTreeMap::new();
        nodes.insert(
            NodeId::new(1).expect("nonzero root"),
            Node {
                parent: None,
                patch: None,
                request: None,
            },
        );
        Self {
            root,
            nodes,
            next_id: 2,
        }
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
                .is_some_and(Patch::configuration_changed),
        })
    }
    pub fn materialize(&self, id: NodeId) -> Result<SkinProject, HistoryQueryError> {
        let mut cursor = id;
        let mut patches = Vec::new();
        loop {
            let node = self
                .nodes
                .get(&cursor)
                .ok_or(HistoryQueryError::UnknownNode(cursor))?;
            if let Some(patch) = &node.patch {
                patches.push(patch);
            }
            match node.parent {
                Some(parent) => cursor = parent,
                None => break,
            }
        }
        let mut project = self.root.clone();
        for patch in patches.into_iter().rev() {
            project = patch
                .apply(&project, true)
                .map_err(HistoryQueryError::InvalidHistory)?;
        }
        Ok(project)
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
