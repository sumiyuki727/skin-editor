use std::collections::{BTreeMap, BTreeSet};

use crate::{
    Affine2, Asset, AssetKind, AssetLibrary, Bounds, ComponentId, ComponentKind, ComponentTree,
    DocumentId, ElementDocument, LayerId, LayerKind, ModelError, ObjectId, OutputMapping,
    ProjectId, SkinConfiguration,
};

/// Version of the in-memory definition contract, not a frozen disk file format.
pub const MODEL_VERSION: u16 = 1;

/// Untrusted detached input. It cannot replace an application's active project.
#[derive(Clone, Debug, PartialEq)]
pub struct ProjectDefinition {
    pub model_version: u16,
    pub id: ProjectId,
    pub assets: Vec<Asset>,
    pub documents: Vec<ElementDocument>,
    pub components: ComponentTree,
    pub configuration: SkinConfiguration,
    pub outputs: Vec<OutputMapping>,
}

impl ProjectDefinition {
    pub fn empty(id: ProjectId) -> Self {
        Self {
            model_version: MODEL_VERSION,
            id,
            assets: Vec::new(),
            documents: Vec::new(),
            components: ComponentTree::default(),
            configuration: SkinConfiguration::default(),
            outputs: Vec::new(),
        }
    }
}

/// Validated immutable project value. No mutable collections or setters escape.
///
/// ```compile_fail
/// use skin_editor_domain::{ProjectId, SkinProject};
/// let mut project = SkinProject::empty(ProjectId::new(1).unwrap());
/// project.documents.clear();
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct SkinProject {
    id: ProjectId,
    assets: AssetLibrary,
    documents: BTreeMap<DocumentId, ElementDocument>,
    components: ComponentTree,
    configuration: SkinConfiguration,
    outputs: Vec<OutputMapping>,
}

impl SkinProject {
    /// The initial empty state has no references or outputs to invalidate.
    pub fn empty(id: ProjectId) -> Self {
        Self {
            id,
            assets: AssetLibrary::default(),
            documents: BTreeMap::new(),
            components: ComponentTree::default(),
            configuration: SkinConfiguration::default(),
            outputs: Vec::new(),
        }
    }

    pub fn from_definition(definition: ProjectDefinition) -> Result<Self, ModelError> {
        if definition.model_version != MODEL_VERSION {
            return Err(ModelError::UnsupportedVersion(definition.model_version));
        }
        let assets = AssetLibrary::new(definition.assets)?;
        let mut documents = BTreeMap::new();
        let mut layer_ids = BTreeSet::new();
        for document in definition.documents {
            for layer in document.layers().iter() {
                if !layer_ids.insert(layer.id) {
                    return Err(ModelError::DuplicateId(ObjectId::Layer(layer.id)));
                }
                if let LayerKind::Image { asset } = layer.kind {
                    let source = assets
                        .get(asset)
                        .ok_or(ModelError::MissingReference(ObjectId::Asset(asset)))?;
                    let AssetKind::RasterImage(size) = source.kind() else {
                        return Err(ModelError::InvalidValue("image_layer_asset_kind"));
                    };
                    Bounds::transformed(
                        *size,
                        document
                            .layers()
                            .world_transform(layer.id)
                            .ok_or(ModelError::MissingReference(ObjectId::Layer(layer.id)))?,
                    )?;
                }
            }
            let id = document.id();
            if documents.insert(id, document).is_some() {
                return Err(ModelError::DuplicateId(ObjectId::Document(id)));
            }
        }
        for node in definition.components.iter() {
            if let ComponentKind::Element {
                document,
                document_to_component,
            } = node.kind
            {
                let member = documents
                    .get(&document)
                    .ok_or(ModelError::MissingReference(ObjectId::Document(document)))?;
                let world = definition
                    .components
                    .world_transform(node.id)
                    .ok_or(ModelError::MissingReference(ObjectId::Component(node.id)))?;
                Bounds::transformed(member.canvas(), world.compose(document_to_component)?)?;
                let content = definition
                    .components
                    .document_content_transform(document)
                    .ok_or(ModelError::MissingReference(ObjectId::Document(document)))?;
                Bounds::transformed(member.canvas(), content)?;
            }
        }
        for document in documents.values() {
            let content = definition
                .components
                .document_content_transform(document.id())
                .unwrap_or(Affine2::IDENTITY);
            for layer in document.layers().iter() {
                let world = document
                    .layers()
                    .world_transform(layer.id)
                    .ok_or(ModelError::MissingReference(ObjectId::Layer(layer.id)))?;
                let effective = content.compose(world)?;
                if let LayerKind::Image { asset } = layer.kind
                    && let Some(AssetKind::RasterImage(size)) = assets.get(asset).map(Asset::kind)
                {
                    Bounds::transformed(*size, effective)?;
                }
            }
        }
        let mut output_ids = BTreeSet::new();
        let mut paths = BTreeSet::<String>::new();
        for output in &definition.outputs {
            if !documents.contains_key(&output.document()) {
                return Err(ModelError::MissingReference(ObjectId::Document(
                    output.document(),
                )));
            }
            if !output_ids.insert(output.id()) {
                return Err(ModelError::DuplicateId(ObjectId::Output(output.id())));
            }
            let path = output.path().collision_key();
            if !paths.insert(path.clone()) {
                return Err(ModelError::OutputConflict(
                    output.path().as_str().to_owned(),
                ));
            }
        }
        // Check every prefix, not just adjacent lexical entries (a, a-b, a/b).
        for path in &paths {
            for (index, _) in path.match_indices('/') {
                if paths.contains(&path[..index]) {
                    return Err(ModelError::OutputConflict(path.clone()));
                }
            }
        }
        Ok(Self {
            id: definition.id,
            assets,
            documents,
            components: definition.components,
            configuration: definition.configuration,
            outputs: definition.outputs,
        })
    }

    pub const fn id(&self) -> ProjectId {
        self.id
    }

    pub const fn model_version(&self) -> u16 {
        MODEL_VERSION
    }

    pub fn assets(&self) -> &AssetLibrary {
        &self.assets
    }

    pub fn document(&self, id: DocumentId) -> Option<&ElementDocument> {
        self.documents.get(&id)
    }

    pub fn documents(&self) -> impl ExactSizeIterator<Item = &ElementDocument> {
        self.documents.values()
    }

    pub fn components(&self) -> &ComponentTree {
        &self.components
    }

    pub fn configuration(&self) -> &SkinConfiguration {
        &self.configuration
    }

    pub fn outputs(&self) -> &[OutputMapping] {
        &self.outputs
    }

    /// Shared geometry input for future composition plans, in document pixels.
    pub fn document_content_transform(&self, id: DocumentId) -> Result<Affine2, ModelError> {
        if !self.documents.contains_key(&id) {
            return Err(ModelError::MissingReference(ObjectId::Document(id)));
        }
        Ok(self
            .components
            .document_content_transform(id)
            .unwrap_or(Affine2::IDENTITY))
    }

    /// Includes full source image rectangles, regardless of visibility/alpha.
    /// Bounds are in document space before the component content transform;
    /// group opacity is not flattened and canvas clipping is not applied here.
    pub fn layer_bounds(
        &self,
        document: DocumentId,
        layer: LayerId,
    ) -> Result<Option<Bounds>, ModelError> {
        let tree = self
            .document(document)
            .ok_or(ModelError::MissingReference(ObjectId::Document(document)))?
            .layers();
        if tree.node(layer).is_none() {
            return Err(ModelError::MissingReference(ObjectId::Layer(layer)));
        }
        let mut stack = vec![layer];
        let mut result: Option<Bounds> = None;
        while let Some(id) = stack.pop() {
            let node = tree
                .node(id)
                .ok_or(ModelError::MissingReference(ObjectId::Layer(id)))?;
            stack.extend(node.children().iter().copied());
            if let LayerKind::Image { asset } = node.kind {
                let source = self
                    .assets
                    .get(asset)
                    .ok_or(ModelError::MissingReference(ObjectId::Asset(asset)))?;
                let AssetKind::RasterImage(size) = source.kind() else {
                    return Err(ModelError::InvalidValue("image_layer_asset_kind"));
                };
                let world = tree
                    .world_transform(id)
                    .ok_or(ModelError::MissingReference(ObjectId::Layer(id)))?;
                let bounds = Bounds::transformed(*size, world)?;
                result = Some(result.map_or(bounds, |old| old.union(bounds)));
            }
        }
        Ok(result)
    }

    /// Component geometry uses mapped document canvases, not gameplay hit areas.
    pub fn component_bounds(&self, component: ComponentId) -> Result<Option<Bounds>, ModelError> {
        if self.components.node(component).is_none() {
            return Err(ModelError::MissingReference(ObjectId::Component(component)));
        }
        let mut stack = vec![component];
        let mut result: Option<Bounds> = None;
        while let Some(id) = stack.pop() {
            let node = self
                .components
                .node(id)
                .ok_or(ModelError::MissingReference(ObjectId::Component(id)))?;
            stack.extend(node.children().iter().copied());
            if let ComponentKind::Element {
                document,
                document_to_component,
            } = node.kind
            {
                let member = self
                    .document(document)
                    .ok_or(ModelError::MissingReference(ObjectId::Document(document)))?;
                let world = self
                    .components
                    .world_transform(id)
                    .ok_or(ModelError::MissingReference(ObjectId::Component(id)))?;
                let bounds =
                    Bounds::transformed(member.canvas(), world.compose(document_to_component)?)?;
                result = Some(result.map_or(bounds, |old| old.union(bounds)));
            }
        }
        Ok(result)
    }
}
