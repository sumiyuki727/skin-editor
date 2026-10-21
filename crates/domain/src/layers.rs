use std::collections::BTreeMap;
use std::sync::Arc;

use crate::forest::{ForestIndex, index_forest};
use crate::{Affine2, AssetId, DocumentId, LayerId, ModelError, ObjectId, Opacity, PixelSize};

#[derive(Clone, Debug, PartialEq)]
pub enum LayerKind {
    Image {
        asset: AssetId,
    },
    /// Children have explicit back-to-front order. Opacity applies to the group result.
    Group {
        children: Vec<LayerId>,
    },
}

/// Detached definition; changing it cannot mutate a validated tree or session.
#[derive(Clone, Debug, PartialEq)]
pub struct LayerNode {
    pub id: LayerId,
    pub local: Affine2,
    pub opacity: Opacity,
    pub visible: bool,
    pub kind: LayerKind,
}

impl LayerNode {
    pub fn children(&self) -> &[LayerId] {
        match &self.kind {
            LayerKind::Image { .. } => &[],
            LayerKind::Group { children } => children,
        }
    }
}

/// Lookups never expose a mutable node, even to the owner of a detached tree:
/// ```compile_fail
/// use skin_editor_domain::{LayerId, LayerTree};
/// let mut tree = LayerTree::default();
/// tree.node(LayerId::new(1).unwrap()).unwrap().visible = false;
/// ```
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LayerTree {
    data: Arc<LayerStorage>,
}

// Flat, validated and immutable. Candidate/history clones share all indexes;
// editing reconstructs a new tree through new(), never mutates this storage.
#[derive(Debug, Default, PartialEq)]
struct LayerStorage {
    roots: Vec<LayerId>,
    nodes: BTreeMap<LayerId, LayerNode>,
    index: ForestIndex<LayerId>,
    worlds: BTreeMap<LayerId, Affine2>,
}

impl LayerTree {
    /// Validates a complete detached forest, not an editing entry point.
    pub fn new(roots: Vec<LayerId>, definitions: Vec<LayerNode>) -> Result<Self, ModelError> {
        let mut nodes = BTreeMap::new();
        for node in definitions {
            let id = node.id;
            if nodes.insert(id, node).is_some() {
                return Err(ModelError::DuplicateId(ObjectId::Layer(id)));
            }
        }
        let edges = nodes
            .iter()
            .map(|(id, n)| (*id, n.children().to_vec()))
            .collect();
        let index = index_forest(&roots, &edges, ObjectId::Layer)?;
        let mut worlds = BTreeMap::new();
        for id in &index.preorder {
            let parent_world = index.parents[id].map_or(Affine2::IDENTITY, |p| worlds[&p]);
            worlds.insert(*id, parent_world.compose(nodes[id].local)?);
        }
        Ok(Self {
            data: Arc::new(LayerStorage {
                roots,
                nodes,
                index,
                worlds,
            }),
        })
    }

    pub fn roots(&self) -> &[LayerId] {
        &self.data.roots
    }

    pub fn node(&self, id: LayerId) -> Option<&LayerNode> {
        self.data.nodes.get(&id)
    }

    pub fn parent(&self, id: LayerId) -> Option<Option<LayerId>> {
        self.data.index.parents.get(&id).copied()
    }

    pub fn world_transform(&self, id: LayerId) -> Option<Affine2> {
        self.data.worlds.get(&id).copied()
    }

    pub fn iter(&self) -> impl ExactSizeIterator<Item = &LayerNode> {
        self.data
            .index
            .preorder
            .iter()
            .map(|id| &self.data.nodes[id])
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ElementDocument {
    id: DocumentId,
    canvas: PixelSize,
    layers: LayerTree,
}

impl ElementDocument {
    pub const fn new(id: DocumentId, canvas: PixelSize, layers: LayerTree) -> Self {
        Self { id, canvas, layers }
    }

    pub const fn id(&self) -> DocumentId {
        self.id
    }

    pub const fn canvas(&self) -> PixelSize {
        self.canvas
    }

    pub fn layers(&self) -> &LayerTree {
        &self.layers
    }
}
