use std::collections::{BTreeMap, BTreeSet};

use skin_editor_domain::{
    Affine2, Asset, AudioComponent, ComponentId, ComponentKind, ComponentNode, ComponentTree,
    DocumentId, ElementDocument, LayerId, LayerKind, LayerNode, LayerTree, ObjectId, OutputSource,
    ProjectDefinition, SkinConfiguration, SkinProject,
};

use crate::history::definition;
use crate::{Command, CommandFailure};

fn reserve(used: &mut BTreeSet<ObjectId>, id: ObjectId) -> Result<(), CommandFailure> {
    if !used.insert(id) {
        return Err(CommandFailure::ReusedIdentity(id));
    }
    Ok(())
}
fn position(index: usize, len: usize) -> Result<(), CommandFailure> {
    if index > len {
        return Err(CommandFailure::InvalidIndex { index, len });
    }
    Ok(())
}
fn doc_index(input: &ProjectDefinition, id: DocumentId) -> Result<usize, CommandFailure> {
    input
        .documents
        .iter()
        .position(|d| d.id() == id)
        .ok_or(CommandFailure::Missing(ObjectId::Document(id)))
}

type LayerNodes = BTreeMap<LayerId, LayerNode>;
fn layer_children<'a>(
    roots: &'a mut Vec<LayerId>,
    nodes: &'a mut LayerNodes,
    parent: Option<LayerId>,
) -> Result<&'a mut Vec<LayerId>, CommandFailure> {
    match parent {
        None => Ok(roots),
        Some(id) => match &mut nodes
            .get_mut(&id)
            .ok_or(CommandFailure::Missing(ObjectId::Layer(id)))?
            .kind
        {
            LayerKind::Group { children } => Ok(children),
            _ => Err(CommandFailure::InvalidParent(ObjectId::Layer(id))),
        },
    }
}
fn layer_node(nodes: &mut LayerNodes, id: LayerId) -> Result<&mut LayerNode, CommandFailure> {
    nodes
        .get_mut(&id)
        .ok_or(CommandFailure::Missing(ObjectId::Layer(id)))
}
fn layers(
    input: &mut ProjectDefinition,
    document: DocumentId,
    edit: impl FnOnce(&LayerTree, &mut Vec<LayerId>, &mut LayerNodes) -> Result<(), CommandFailure>,
) -> Result<(), CommandFailure> {
    let index = doc_index(input, document)?;
    let old = &input.documents[index];
    let mut roots = old.layers().roots().to_vec();
    let mut nodes = old.layers().iter().map(|n| (n.id, n.clone())).collect();
    edit(old.layers(), &mut roots, &mut nodes)?;
    let tree = LayerTree::new(roots, nodes.into_values().collect())?;
    input.documents[index] =
        ElementDocument::new(document, old.canvas(), tree).with_visibility(old.visible());
    Ok(())
}

type ComponentNodes = BTreeMap<ComponentId, ComponentNode>;
fn component_children<'a>(
    roots: &'a mut Vec<ComponentId>,
    nodes: &'a mut ComponentNodes,
    parent: Option<ComponentId>,
) -> Result<&'a mut Vec<ComponentId>, CommandFailure> {
    match parent {
        None => Ok(roots),
        Some(id) => match &mut nodes
            .get_mut(&id)
            .ok_or(CommandFailure::Missing(ObjectId::Component(id)))?
            .kind
        {
            ComponentKind::Group { children } => Ok(children),
            _ => Err(CommandFailure::InvalidParent(ObjectId::Component(id))),
        },
    }
}
fn component_node(
    nodes: &mut ComponentNodes,
    id: ComponentId,
) -> Result<&mut ComponentNode, CommandFailure> {
    nodes
        .get_mut(&id)
        .ok_or(CommandFailure::Missing(ObjectId::Component(id)))
}
fn components(
    input: &mut ProjectDefinition,
    edit: impl FnOnce(
        &ComponentTree,
        &mut Vec<ComponentId>,
        &mut ComponentNodes,
    ) -> Result<(), CommandFailure>,
) -> Result<(), CommandFailure> {
    let old = &input.components;
    let mut roots = old.roots().to_vec();
    let mut nodes = old.iter().map(|n| (n.id, n.clone())).collect();
    edit(old, &mut roots, &mut nodes)?;
    input.components = ComponentTree::new(roots, nodes.into_values().collect())?;
    Ok(())
}

/// Only used by the application owner's isolated candidate execution.
pub(crate) fn apply(
    project: &SkinProject,
    command: &Command,
    used: &mut BTreeSet<ObjectId>,
) -> Result<SkinProject, CommandFailure> {
    let mut input = definition(project);
    match command {
        Command::RegisterAsset { asset } => {
            reserve(used, ObjectId::Asset(asset.id()))?;
            input.assets.push(asset.clone());
        }
        Command::RemoveAsset { asset } => {
            let index = input
                .assets
                .iter()
                .position(|a| a.id() == *asset)
                .ok_or(CommandFailure::Missing(ObjectId::Asset(*asset)))?;
            let references: Vec<_> = input
                .documents
                .iter()
                .flat_map(|d| d.layers().iter())
                .filter_map(|n| {
                    matches!(n.kind, LayerKind::Image { asset: id } if id == *asset).then_some(n.id)
                })
                .collect();
            if !references.is_empty() {
                return Err(CommandFailure::AssetInUse {
                    asset: *asset,
                    layers: references,
                });
            }
            let references: Vec<_> = input
                .audio_components
                .iter()
                .filter(|c| c.asset() == *asset)
                .map(|c| c.id())
                .collect();
            if !references.is_empty() {
                return Err(CommandFailure::AudioAssetInUse {
                    asset: *asset,
                    components: references,
                });
            }
            input.assets.remove(index);
        }
        Command::RenameAsset { asset, name } => {
            let old = input
                .assets
                .iter_mut()
                .find(|a| a.id() == *asset)
                .ok_or(CommandFailure::Missing(ObjectId::Asset(*asset)))?;
            *old = Asset::new(*asset, name.clone(), old.blob(), old.kind().clone())?;
        }
        Command::CreateDocument { document } => {
            reserve(used, ObjectId::Document(document.id()))?;
            for node in document.layers().iter() {
                reserve(used, ObjectId::Layer(node.id))?;
            }
            input.documents.push(document.clone());
        }
        Command::CreateAudioComponent { component } => {
            reserve(used, ObjectId::AudioComponent(component.id()))?;
            input.audio_components.push(component.clone());
        }
        Command::RemoveAudioComponent { component } => {
            let index = input
                .audio_components
                .iter()
                .position(|c| c.id() == *component)
                .ok_or(CommandFailure::Missing(ObjectId::AudioComponent(
                    *component,
                )))?;
            if input
                .outputs
                .iter()
                .any(|m| m.source() == OutputSource::Audio(*component))
            {
                return Err(CommandFailure::AudioComponentInUse(*component));
            }
            input.audio_components.remove(index);
        }
        Command::SetAudioAsset { component, asset } => {
            let old = input
                .audio_components
                .iter_mut()
                .find(|c| c.id() == *component)
                .ok_or(CommandFailure::Missing(ObjectId::AudioComponent(
                    *component,
                )))?;
            *old = AudioComponent::new(*component, old.slot().clone(), *asset)
                .with_properties(old.gain(), old.visible());
        }
        Command::SetAudioProperties {
            component,
            gain,
            visible,
        } => {
            let old = input
                .audio_components
                .iter_mut()
                .find(|c| c.id() == *component)
                .ok_or(CommandFailure::Missing(ObjectId::AudioComponent(
                    *component,
                )))?;
            *old = old.clone().with_properties(*gain, *visible);
        }
        Command::SetDocumentVisibility { document, visible } => {
            let index = doc_index(&input, *document)?;
            input.documents[index] = input.documents[index].clone().with_visibility(*visible);
        }
        Command::RemoveDocument { document } => {
            let index = doc_index(&input, *document)?;
            if input.components.member(*document).is_some()
                || input
                    .outputs
                    .iter()
                    .any(|m| m.document() == Some(*document))
            {
                return Err(CommandFailure::DocumentInUse(*document));
            }
            input.documents.remove(index);
        }
        Command::ResizeDocument { document, canvas } => {
            let index = doc_index(&input, *document)?;
            input.documents[index] =
                ElementDocument::new(*document, *canvas, input.documents[index].layers().clone())
                    .with_visibility(input.documents[index].visible());
        }
        Command::AddLayer {
            document,
            node,
            parent,
            index,
        } => {
            reserve(used, ObjectId::Layer(node.id))?;
            if !node.children().is_empty() {
                return Err(CommandFailure::InvalidOperation("new_group_must_be_empty"));
            }
            layers(&mut input, *document, |_, roots, nodes| {
                let children = layer_children(roots, nodes, *parent)?;
                position(*index, children.len())?;
                children.insert(*index, node.id);
                nodes.insert(node.id, node.clone());
                Ok(())
            })?;
        }
        Command::RemoveLayer { document, layer } => {
            layers(&mut input, *document, |old, roots, nodes| {
                let parent = old
                    .parent(*layer)
                    .ok_or(CommandFailure::Missing(ObjectId::Layer(*layer)))?;
                layer_children(roots, nodes, parent)?.retain(|id| id != layer);
                let mut stack = vec![*layer];
                while let Some(id) = stack.pop() {
                    let node = nodes
                        .remove(&id)
                        .ok_or(CommandFailure::Missing(ObjectId::Layer(id)))?;
                    stack.extend(node.children().iter().copied());
                }
                Ok(())
            })?;
        }
        Command::MoveLayer {
            document,
            layer,
            parent,
            index,
        } => {
            layers(&mut input, *document, |old, roots, nodes| {
                let old_parent = old
                    .parent(*layer)
                    .ok_or(CommandFailure::Missing(ObjectId::Layer(*layer)))?;
                let mut ancestor = *parent;
                while let Some(id) = ancestor {
                    if id == *layer {
                        return Err(CommandFailure::WouldCreateCycle(ObjectId::Layer(id)));
                    }
                    ancestor = old
                        .parent(id)
                        .ok_or(CommandFailure::Missing(ObjectId::Layer(id)))?;
                }
                let local = if old_parent == *parent {
                    layer_node(nodes, *layer)?.local
                } else {
                    let world = old
                        .world_transform(*layer)
                        .ok_or(CommandFailure::Missing(ObjectId::Layer(*layer)))?;
                    let target = parent.map_or(Ok(Affine2::IDENTITY), |p| {
                        old.world_transform(p)
                            .ok_or(CommandFailure::Missing(ObjectId::Layer(p)))
                    })?;
                    Affine2::rebased(world, target)?
                };
                layer_children(roots, nodes, old_parent)?.retain(|id| id != layer);
                let children = layer_children(roots, nodes, *parent)?;
                position(*index, children.len())?;
                children.insert(*index, *layer);
                layer_node(nodes, *layer)?.local = local;
                Ok(())
            })?;
        }
        Command::SetLayerTransform {
            document,
            layer,
            transform,
        } => {
            layers(&mut input, *document, |_, _, nodes| {
                layer_node(nodes, *layer)?.local = *transform;
                Ok(())
            })?;
        }
        Command::SetLayerProperties {
            document,
            layer,
            opacity,
            visible,
        } => {
            layers(&mut input, *document, |_, _, nodes| {
                let node = layer_node(nodes, *layer)?;
                node.opacity = *opacity;
                node.visible = *visible;
                Ok(())
            })?;
        }
        Command::SetLayerAsset {
            document,
            layer,
            asset,
        } => {
            layers(&mut input, *document, |_, _, nodes| {
                match &mut layer_node(nodes, *layer)?.kind {
                    LayerKind::Image { asset: old } => *old = *asset,
                    _ => return Err(CommandFailure::InvalidOperation("not_an_image_layer")),
                }
                Ok(())
            })?;
        }
        Command::AddComponent {
            node,
            parent,
            index,
        } => {
            reserve(used, ObjectId::Component(node.id))?;
            if !node.children().is_empty() {
                return Err(CommandFailure::InvalidOperation("new_group_must_be_empty"));
            }
            components(&mut input, |_, roots, nodes| {
                let children = component_children(roots, nodes, *parent)?;
                position(*index, children.len())?;
                children.insert(*index, node.id);
                nodes.insert(node.id, node.clone());
                Ok(())
            })?;
        }
        Command::RemoveComponent { component } => {
            components(&mut input, |old, roots, nodes| {
                let parent = old
                    .parent(*component)
                    .ok_or(CommandFailure::Missing(ObjectId::Component(*component)))?;
                component_children(roots, nodes, parent)?.retain(|id| id != component);
                let mut stack = vec![*component];
                while let Some(id) = stack.pop() {
                    let node = nodes
                        .remove(&id)
                        .ok_or(CommandFailure::Missing(ObjectId::Component(id)))?;
                    stack.extend(node.children().iter().copied());
                }
                Ok(())
            })?;
        }
        Command::MoveComponent {
            component,
            parent,
            index,
        } => {
            components(&mut input, |old, roots, nodes| {
                let old_parent = old
                    .parent(*component)
                    .ok_or(CommandFailure::Missing(ObjectId::Component(*component)))?;
                let mut ancestor = *parent;
                while let Some(id) = ancestor {
                    if id == *component {
                        return Err(CommandFailure::WouldCreateCycle(ObjectId::Component(id)));
                    }
                    ancestor = old
                        .parent(id)
                        .ok_or(CommandFailure::Missing(ObjectId::Component(id)))?;
                }
                let local = if old_parent == *parent {
                    component_node(nodes, *component)?.local
                } else {
                    let world = old
                        .world_transform(*component)
                        .ok_or(CommandFailure::Missing(ObjectId::Component(*component)))?;
                    let target = parent.map_or(Ok(Affine2::IDENTITY), |p| {
                        old.world_transform(p)
                            .ok_or(CommandFailure::Missing(ObjectId::Component(p)))
                    })?;
                    Affine2::rebased(world, target)?
                };
                component_children(roots, nodes, old_parent)?.retain(|id| id != component);
                let children = component_children(roots, nodes, *parent)?;
                position(*index, children.len())?;
                children.insert(*index, *component);
                component_node(nodes, *component)?.local = local;
                Ok(())
            })?;
        }
        Command::SetComponentTransform {
            component,
            transform,
        } => {
            components(&mut input, |_, _, nodes| {
                component_node(nodes, *component)?.local = *transform;
                Ok(())
            })?;
        }
        Command::SetComponentMapping {
            component,
            document_to_component,
        } => {
            components(&mut input, |_, _, nodes| {
                match &mut component_node(nodes, *component)?.kind {
                    ComponentKind::Element {
                        document_to_component: map,
                        ..
                    } => *map = *document_to_component,
                    _ => return Err(CommandFailure::InvalidOperation("not_a_component_member")),
                }
                Ok(())
            })?;
        }
        Command::SetOutputMapping { mapping } => {
            if let Some(old) = input.outputs.iter_mut().find(|m| m.id() == mapping.id()) {
                *old = mapping.clone();
            } else {
                reserve(used, ObjectId::Output(mapping.id()))?;
                input.outputs.push(mapping.clone());
            }
        }
        Command::RemoveOutputMapping { output } => {
            let index = input
                .outputs
                .iter()
                .position(|m| m.id() == *output)
                .ok_or(CommandFailure::Missing(ObjectId::Output(*output)))?;
            input.outputs.remove(index);
        }
        Command::InsertConfigRecord { index, record } => {
            let mut records = input.configuration.records().to_vec();
            position(*index, records.len())?;
            records.insert(*index, record.clone());
            input.configuration = SkinConfiguration::new(records);
        }
        Command::SetConfigRecord { index, record } => {
            let mut records = input.configuration.records().to_vec();
            let len = records.len();
            *records
                .get_mut(*index)
                .ok_or(CommandFailure::InvalidIndex { index: *index, len })? = record.clone();
            input.configuration = SkinConfiguration::new(records);
        }
        Command::RemoveConfigRecord { index } => {
            let mut records = input.configuration.records().to_vec();
            if *index >= records.len() {
                return Err(CommandFailure::InvalidIndex {
                    index: *index,
                    len: records.len(),
                });
            }
            records.remove(*index);
            input.configuration = SkinConfiguration::new(records);
        }
    }
    Ok(SkinProject::from_definition(input)?)
}
