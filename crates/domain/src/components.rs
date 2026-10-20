use std::collections::BTreeMap;

use crate::forest::{ForestIndex, index_forest};
use crate::{Affine2, ComponentId, DocumentId, ModelError, ObjectId};

#[derive(Clone, Debug, PartialEq)]
pub enum ComponentKind {
    Group {
        children: Vec<ComponentId>,
    },
    Element {
        document: DocumentId,
        /// Invertible map from document pixels into the untransformed component basis.
        /// It is an editing mapping, not a guessed stable anchor.
        document_to_component: Affine2,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct ComponentNode {
    pub id: ComponentId,
    pub local: Affine2,
    pub kind: ComponentKind,
}

impl ComponentNode {
    pub fn children(&self) -> &[ComponentId] {
        match &self.kind {
            ComponentKind::Group { children } => children,
            ComponentKind::Element { .. } => &[],
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ComponentTree {
    roots: Vec<ComponentId>,
    nodes: BTreeMap<ComponentId, ComponentNode>,
    index: ForestIndex<ComponentId>,
    worlds: BTreeMap<ComponentId, Affine2>,
    members: BTreeMap<DocumentId, ComponentId>,
    content_transforms: BTreeMap<DocumentId, Affine2>,
}

impl ComponentTree {
    pub fn new(
        roots: Vec<ComponentId>,
        definitions: Vec<ComponentNode>,
    ) -> Result<Self, ModelError> {
        let mut nodes = BTreeMap::new();
        let mut members = BTreeMap::new();
        for node in definitions {
            if let ComponentKind::Element {
                document,
                document_to_component,
            } = node.kind
            {
                document_to_component.inverse()?;
                if members.insert(document, node.id).is_some() {
                    return Err(ModelError::DuplicateMembership(document));
                }
            }
            let id = node.id;
            if nodes.insert(id, node).is_some() {
                return Err(ModelError::DuplicateId(ObjectId::Component(id)));
            }
        }
        let edges = nodes
            .iter()
            .map(|(id, n)| (*id, n.children().to_vec()))
            .collect();
        let index = index_forest(&roots, &edges, ObjectId::Component)?;
        let mut worlds = BTreeMap::new();
        let mut content_transforms = BTreeMap::new();
        for id in &index.preorder {
            let parent_world = index.parents[id].map_or(Affine2::IDENTITY, |p| worlds[&p]);
            let world = parent_world.compose(nodes[id].local)?;
            worlds.insert(*id, world);
            if let ComponentKind::Element {
                document,
                document_to_component: map,
            } = nodes[id].kind
            {
                // Same document-space content transform for preview and export:
                // map^-1 * ancestor/local transforms * map.
                let content = map.inverse()?.compose(world)?.compose(map)?;
                content_transforms.insert(document, content);
            }
        }
        Ok(Self {
            roots,
            nodes,
            index,
            worlds,
            members,
            content_transforms,
        })
    }

    pub fn roots(&self) -> &[ComponentId] {
        &self.roots
    }

    pub fn node(&self, id: ComponentId) -> Option<&ComponentNode> {
        self.nodes.get(&id)
    }

    pub fn parent(&self, id: ComponentId) -> Option<Option<ComponentId>> {
        self.index.parents.get(&id).copied()
    }

    pub fn world_transform(&self, id: ComponentId) -> Option<Affine2> {
        self.worlds.get(&id).copied()
    }

    pub fn member(&self, document: DocumentId) -> Option<ComponentId> {
        self.members.get(&document).copied()
    }

    pub fn document_content_transform(&self, document: DocumentId) -> Option<Affine2> {
        self.content_transforms.get(&document).copied()
    }

    pub fn iter(&self) -> impl ExactSizeIterator<Item = &ComponentNode> {
        self.index.preorder.iter().map(|id| &self.nodes[id])
    }
}
