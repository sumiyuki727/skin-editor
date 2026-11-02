use super::*;
use serde_json::json;
use skin_editor_application::values::*;

pub(super) fn encode(
    project: &SkinProject,
    limits: FormatLimits,
) -> Result<StateBundle, CodecError> {
    preflight(project, limits)?;
    let mut writer = Writer {
        limits,
        blocks: Vec::new(),
        seen: BTreeMap::new(),
        bytes: 0,
        decoded_bytes: 0,
    };
    let mut assets = Vec::new();
    for asset in project.assets().iter() {
        let kind = match asset.kind() {
            AssetKind::RasterImage(size) => {
                json!({"tag":"image","size":[size.width(),size.height()]})
            }
            AssetKind::Audio(info) => {
                json!({"tag":"audio","rate":info.sample_rate(),"channels":info.channels(),"frames":info.frames().to_string()})
            }
            AssetKind::Opaque => json!({"tag":"opaque"}),
        };
        let blob = asset.blob();
        assets.push(writer.block("skin-editor.asset", json!({"id":id(asset.id().get()),"name":asset.name(),
            "blob":{"sha256":hex(&blob.sha256()),"bytes":blob.byte_len().to_string()},"media":kind}), &[], &[blob.sha256()])?.hex());
    }
    let assets = writer.sequence("assets", assets, true)?;
    let mut documents = Vec::new();
    for document in project.documents() {
        let mut layers = Vec::new();
        for node in document.layers().iter() {
            let (kind, refs) = match &node.kind {
                LayerKind::Image { asset } => {
                    (json!({"tag":"image","asset":id(asset.get())}), Vec::new())
                }
                LayerKind::Group { children } => {
                    let children = writer.sequence(
                        "layer-ids",
                        children.iter().map(|n| id(n.get())).collect(),
                        false,
                    )?;
                    (
                        json!({"tag":"group","children":children.hex()}),
                        vec![children],
                    )
                }
            };
            layers.push(
                writer
                    .block(
                        "skin-editor.layer",
                        json!({"id":id(node.id.get()),"local":node.local.coefficients(),
                "opacity":f64::from(node.opacity.get()),"visible":node.visible,"node":kind}),
                        &refs,
                        &[],
                    )?
                    .hex(),
            );
        }
        let layers = writer.sequence("layers", layers, true)?;
        let roots = writer.sequence(
            "layer-ids",
            document
                .layers()
                .roots()
                .iter()
                .map(|n| id(n.get()))
                .collect(),
            false,
        )?;
        documents.push(writer.block("skin-editor.document", json!({"id":id(document.id().get()),
            "size":[document.canvas().width(),document.canvas().height()],"visible":document.visible(),
            "roots":roots.hex(),"layers":layers.hex()}), &[roots,layers], &[])?.hex());
    }
    let documents = writer.sequence("documents", documents, true)?;
    let mut components = Vec::new();
    for node in project.components().iter() {
        let (kind, refs) = match &node.kind {
            ComponentKind::Group { children } => {
                let children = writer.sequence(
                    "component-ids",
                    children.iter().map(|n| id(n.get())).collect(),
                    false,
                )?;
                (
                    json!({"tag":"group","children":children.hex()}),
                    vec![children],
                )
            }
            ComponentKind::Element {
                document,
                document_to_component,
            } => (
                json!({"tag":"element","document":id(document.get()),"mapping":document_to_component.coefficients()}),
                Vec::new(),
            ),
        };
        components.push(
            writer
                .block(
                    "skin-editor.component",
                    json!({"id":id(node.id.get()),"local":node.local.coefficients(),"node":kind}),
                    &refs,
                    &[],
                )?
                .hex(),
        );
    }
    let components = writer.sequence("components", components, true)?;
    let roots = writer.sequence(
        "component-ids",
        project
            .components()
            .roots()
            .iter()
            .map(|n| id(n.get()))
            .collect(),
        false,
    )?;
    let components = writer.block(
        "skin-editor.visual-tree",
        json!({"roots":roots.hex(),"nodes":components.hex()}),
        &[roots, components],
        &[],
    )?;
    let mut sounds = Vec::new();
    for sound in project.audio_components() {
        sounds.push(
            writer
                .block(
                    "skin-editor.sound",
                    json!({"id":id(sound.id().get()),"slot":sound.slot().as_str(),
            "asset":id(sound.asset().get()),"gain":sound.gain().get(),"visible":sound.visible()}),
                    &[],
                    &[],
                )?
                .hex(),
        );
    }
    let sounds = writer.sequence("sounds", sounds, true)?;
    let mut configuration = Vec::new();
    for record in project.configuration().records() {
        let range = |r: &std::ops::Range<usize>| json!([r.start.to_string(), r.end.to_string()]);
        let interpretation = match record.interpretation() {
            ConfigInterpretation::Opaque => json!({"tag":"opaque"}),
            ConfigInterpretation::Section { name } => json!({"tag":"section","name":range(name)}),
            ConfigInterpretation::Entry { key, value } => {
                json!({"tag":"entry","key":range(key),"value":range(value)})
            }
        };
        configuration.push(
            writer
                .block(
                    "skin-editor.config-record",
                    json!({"raw":hex(record.raw()),"interpretation":interpretation}),
                    &[],
                    &[],
                )?
                .hex(),
        );
    }
    let configuration = writer.sequence("configuration", configuration, true)?;
    let mut outputs = Vec::new();
    for output in project.outputs() {
        let source = match output.source() {
            OutputSource::Image(d) => json!({"tag":"image","id":id(d.get())}),
            OutputSource::Audio(a) => json!({"tag":"audio","id":id(a.get())}),
        };
        outputs.push(
            writer
                .block(
                    "skin-editor.output",
                    json!({"id":id(output.id().get()),"source":source,
            "key":output.element_key(),"path":output.path().as_str()}),
                    &[],
                    &[],
                )?
                .hex(),
        );
    }
    let outputs = writer.sequence("outputs", outputs, true)?;
    let root = writer.block("skin-editor.project", json!({"id":id(project.id().get()),"semantics":STATE_SEMANTICS,
        "assets":assets.hex(),"documents":documents.hex(),"components":components.hex(),"sounds":sounds.hex(),
        "configuration":configuration.hex(),"outputs":outputs.hex()}),
        &[assets,documents,components,sounds,configuration,outputs], &[])?;
    Ok(StateBundle {
        root,
        blocks: writer.blocks,
    })
}

fn preflight(project: &SkinProject, limits: FormatLimits) -> Result<(), CodecError> {
    let mut items = 0usize;
    let mut text = 0usize;
    let mut charge = |count: usize, bytes: usize| -> Result<(), CodecError> {
        items = items
            .checked_add(count)
            .ok_or(CodecError::ResourceLimit("items"))?;
        text = text
            .checked_add(bytes)
            .ok_or(CodecError::ResourceLimit("text"))?;
        if items > limits.items
            || bytes > limits.block_bytes.saturating_sub(512) / 6
            || text > limits.total_bytes / 6
        {
            return Err(CodecError::ResourceLimit("preflight"));
        }
        Ok(())
    };
    for asset in project.assets().iter() {
        charge(1, asset.name().len())?;
    }
    for document in project.documents() {
        charge(1 + document.layers().roots().len(), 0)?;
        for layer in document.layers().iter() {
            charge(1 + layer.children().len(), 0)?;
        }
    }
    charge(project.components().roots().len(), 0)?;
    for component in project.components().iter() {
        charge(1 + component.children().len(), 0)?;
    }
    for sound in project.audio_components() {
        charge(1, sound.slot().as_str().len())?;
    }
    for record in project.configuration().records() {
        charge(1, record.raw().len())?;
    }
    for output in project.outputs() {
        charge(1, output.element_key().len() + output.path().as_str().len())?;
    }
    Ok(())
}
