use super::*;
use serde_json::{Map, Value};
use skin_editor_application::values::*;

struct Fields(Map<String, Value>);
impl Fields {
    fn new(value: Value) -> Result<Self, CodecError> {
        match value {
            Value::Object(map) => Ok(Self(map)),
            _ => Err(CodecError::InvalidField("object")),
        }
    }
    fn take(&mut self, key: &'static str) -> Result<Value, CodecError> {
        self.0.remove(key).ok_or(CodecError::InvalidField(key))
    }
    fn text(&mut self, key: &'static str) -> Result<String, CodecError> {
        text(self.take(key)?)
    }
    fn flag(&mut self, key: &'static str) -> Result<bool, CodecError> {
        self.take(key)?
            .as_bool()
            .ok_or(CodecError::InvalidField(key))
    }
    fn identity(&mut self, key: &'static str) -> Result<u128, CodecError> {
        parse_id(&self.text(key)?)
    }
    fn reference(&mut self, key: &'static str) -> Result<StateHash, CodecError> {
        parse_hash(&self.text(key)?)
    }
    fn finish(self) -> Result<(), CodecError> {
        if self.0.is_empty() {
            Ok(())
        } else {
            Err(CodecError::UnsupportedField)
        }
    }
}
fn text(value: Value) -> Result<String, CodecError> {
    match value {
        Value::String(v) => Ok(v),
        _ => Err(CodecError::InvalidField("text")),
    }
}
fn array(value: Value) -> Result<Vec<Value>, CodecError> {
    match value {
        Value::Array(v) => Ok(v),
        _ => Err(CodecError::InvalidField("array")),
    }
}
fn integer(value: Value) -> Result<u64, CodecError> {
    value.as_u64().ok_or(CodecError::InvalidField("integer"))
}
fn number(value: Value) -> Result<f64, CodecError> {
    value
        .as_f64()
        .filter(|v| v.is_finite())
        .ok_or(CodecError::InvalidField("number"))
}
fn hashes(value: Value) -> Result<Vec<StateHash>, CodecError> {
    let values = array(value)?
        .into_iter()
        .map(|v| parse_hash(&text(v)?))
        .collect::<Result<Vec<_>, _>>()?;
    if values.windows(2).any(|p| p[0] >= p[1]) {
        return Err(CodecError::ReferenceMismatch);
    }
    Ok(values)
}
fn matrix(value: Value) -> Result<Affine2, CodecError> {
    let values = array(value)?
        .into_iter()
        .map(number)
        .collect::<Result<Vec<_>, _>>()?;
    Affine2::new(
        values
            .try_into()
            .map_err(|_| CodecError::InvalidField("matrix"))?,
    )
    .map_err(Into::into)
}
fn size(value: Value) -> Result<PixelSize, CodecError> {
    let values = array(value)?
        .into_iter()
        .map(|v| u32::try_from(integer(v)?).map_err(|_| CodecError::InvalidField("size")))
        .collect::<Result<Vec<_>, _>>()?;
    let [w, h]: [u32; 2] = values
        .try_into()
        .map_err(|_| CodecError::InvalidField("size"))?;
    Ok(PixelSize::new(w, h)?)
}
fn range(value: Value) -> Result<std::ops::Range<usize>, CodecError> {
    let values = array(value)?
        .into_iter()
        .map(|v| {
            usize::try_from(decimal(&text(v)?)?).map_err(|_| CodecError::ResourceLimit("range"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let [start, end]: [usize; 2] = values
        .try_into()
        .map_err(|_| CodecError::InvalidField("range"))?;
    Ok(start..end)
}

struct Record {
    fields: Fields,
    refs: Vec<StateHash>,
    blobs: Vec<StateHash>,
}
impl Record {
    fn finish(self, refs: &[StateHash], blobs: &[[u8; 32]]) -> Result<(), CodecError> {
        self.fields.finish()?;
        let expected: Vec<_> = refs
            .iter()
            .copied()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        let originals: Vec<_> = blobs
            .iter()
            .copied()
            .map(StateHash)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        if self.refs != expected || self.blobs != originals {
            return Err(CodecError::ReferenceMismatch);
        }
        Ok(())
    }
}
struct Reader<'a> {
    index: BTreeMap<StateHash, &'a StateBlock>,
    seen: BTreeSet<StateHash>,
    limits: FormatLimits,
    items: usize,
    decoded_bytes: usize,
}
impl Reader<'_> {
    fn record(&mut self, hash: StateHash, kind: &str) -> Result<Record, CodecError> {
        let block = self.index.get(&hash).ok_or(CodecError::MissingBlock)?;
        if block.bytes.len() > self.limits.decoded_bytes.saturating_sub(self.decoded_bytes) {
            return Err(CodecError::ResourceLimit("decoded_bytes"));
        }
        self.decoded_bytes += block.bytes.len();
        let mut fields = Fields::new(json::parse(&block.bytes, self.limits)?)?;
        if integer(fields.take("schema")?)? != u64::from(PROJECT_STATE_SCHEMA) {
            return Err(CodecError::UnsupportedSchema);
        }
        if fields.text("kind")? != kind {
            return Err(CodecError::UnsupportedKind);
        }
        let refs = hashes(fields.take("blocks")?)?;
        let blobs = hashes(fields.take("blobs")?)?;
        for reference in &refs {
            if !self.index.contains_key(reference) {
                return Err(CodecError::MissingBlock);
            }
        }
        let value = Fields::new(fields.take("value")?)?;
        fields.finish()?;
        self.seen.insert(hash);
        Ok(Record {
            fields: value,
            refs,
            blobs,
        })
    }
    fn sequence(
        &mut self,
        hash: StateHash,
        role: &str,
        references: bool,
    ) -> Result<Vec<String>, CodecError> {
        let mut seq = self.record(hash, "skin-editor.sequence")?;
        if seq.fields.text("role")? != role {
            return Err(CodecError::InvalidField("sequence_role"));
        }
        let count = usize::try_from(decimal(&seq.fields.text("len")?)?)
            .map_err(|_| CodecError::ResourceLimit("items"))?;
        if count > self.limits.items.saturating_sub(self.items) {
            return Err(CodecError::ResourceLimit("items"));
        }
        self.items += count;
        let pages = array(seq.fields.take("pages")?)?
            .into_iter()
            .map(|v| parse_hash(&text(v)?))
            .collect::<Result<Vec<_>, _>>()?;
        if pages.len() != count.div_ceil(PAGE_ITEMS) {
            return Err(CodecError::InvalidField("page_count"));
        }
        let mut result = Vec::with_capacity(count);
        for page in &pages {
            let mut record = self.record(*page, "skin-editor.page")?;
            if record.fields.text("role")? != role {
                return Err(CodecError::InvalidField("page_role"));
            }
            let items = array(record.fields.take("items")?)?;
            let expected = PAGE_ITEMS.min(count - result.len());
            if items.len() != expected {
                return Err(CodecError::InvalidField("page_items"));
            }
            let values = items.into_iter().map(text).collect::<Result<Vec<_>, _>>()?;
            let refs = if references {
                values
                    .iter()
                    .map(|v| parse_hash(v))
                    .collect::<Result<Vec<_>, _>>()?
            } else {
                for value in &values {
                    parse_id(value)?;
                }
                Vec::new()
            };
            record.finish(&refs, &[])?;
            result.extend(values);
        }
        seq.finish(&pages, &[])?;
        Ok(result)
    }
    fn refs(&mut self, hash: StateHash, role: &str) -> Result<Vec<StateHash>, CodecError> {
        self.sequence(hash, role, true)?
            .iter()
            .map(|v| parse_hash(v))
            .collect()
    }
    fn ids(&mut self, hash: StateHash, role: &str) -> Result<Vec<u128>, CodecError> {
        self.sequence(hash, role, false)?
            .iter()
            .map(|v| parse_id(v))
            .collect()
    }
    fn asset(&mut self, hash: StateHash) -> Result<Asset, CodecError> {
        let mut r = self.record(hash, "skin-editor.asset")?;
        let aid =
            AssetId::new(r.fields.identity("id")?).ok_or(CodecError::InvalidField("asset_id"))?;
        let name = r.fields.text("name")?;
        let mut blob = Fields::new(r.fields.take("blob")?)?;
        let sha = parse_hash(&blob.text("sha256")?)?.bytes();
        let len = decimal(&blob.text("bytes")?)?;
        blob.finish()?;
        let mut media = Fields::new(r.fields.take("media")?)?;
        let kind = match media.text("tag")?.as_str() {
            "image" => AssetKind::RasterImage(size(media.take("size")?)?),
            "audio" => AssetKind::Audio(AudioMetadata::new(
                u32::try_from(integer(media.take("rate")?)?)
                    .map_err(|_| CodecError::InvalidField("rate"))?,
                u16::try_from(integer(media.take("channels")?)?)
                    .map_err(|_| CodecError::InvalidField("channels"))?,
                decimal(&media.text("frames")?)?,
            )?),
            "opaque" => AssetKind::Opaque,
            _ => return Err(CodecError::UnsupportedKind),
        };
        media.finish()?;
        r.finish(&[], &[sha])?;
        Ok(Asset::new(aid, name, BlobRef::new(sha, len), kind)?)
    }
    fn layer(&mut self, hash: StateHash) -> Result<LayerNode, CodecError> {
        let mut r = self.record(hash, "skin-editor.layer")?;
        let id =
            LayerId::new(r.fields.identity("id")?).ok_or(CodecError::InvalidField("layer_id"))?;
        let local = matrix(r.fields.take("local")?)?;
        let scalar = number(r.fields.take("opacity")?)?;
        let opacity = scalar as f32;
        if f64::from(opacity).to_bits() != scalar.to_bits() {
            return Err(CodecError::InvalidField("opacity_precision"));
        }
        let opacity = Opacity::new(opacity)?;
        let visible = r.fields.flag("visible")?;
        let mut node = Fields::new(r.fields.take("node")?)?;
        let mut refs = Vec::new();
        let kind = match node.text("tag")?.as_str() {
            "image" => LayerKind::Image {
                asset: AssetId::new(node.identity("asset")?)
                    .ok_or(CodecError::InvalidField("asset_id"))?,
            },
            "group" => {
                let reference = node.reference("children")?;
                refs.push(reference);
                LayerKind::Group {
                    children: self
                        .ids(reference, "layer-ids")?
                        .into_iter()
                        .map(|n| LayerId::new(n).ok_or(CodecError::InvalidField("layer_id")))
                        .collect::<Result<_, _>>()?,
                }
            }
            _ => return Err(CodecError::UnsupportedKind),
        };
        node.finish()?;
        r.finish(&refs, &[])?;
        Ok(LayerNode {
            id,
            local,
            opacity,
            visible,
            kind,
        })
    }
    fn document(&mut self, hash: StateHash) -> Result<ElementDocument, CodecError> {
        let mut r = self.record(hash, "skin-editor.document")?;
        let id = DocumentId::new(r.fields.identity("id")?)
            .ok_or(CodecError::InvalidField("document_id"))?;
        let canvas = size(r.fields.take("size")?)?;
        let visible = r.fields.flag("visible")?;
        let roots = r.fields.reference("roots")?;
        let layers = r.fields.reference("layers")?;
        let roots_value = self
            .ids(roots, "layer-ids")?
            .into_iter()
            .map(|n| LayerId::new(n).ok_or(CodecError::InvalidField("layer_id")))
            .collect::<Result<_, _>>()?;
        let mut nodes = Vec::new();
        for reference in self.refs(layers, "layers")? {
            nodes.push(self.layer(reference)?);
        }
        r.finish(&[roots, layers], &[])?;
        Ok(
            ElementDocument::new(id, canvas, LayerTree::new(roots_value, nodes)?)
                .with_visibility(visible),
        )
    }
    fn components(&mut self, hash: StateHash) -> Result<ComponentTree, CodecError> {
        let mut r = self.record(hash, "skin-editor.visual-tree")?;
        let roots = r.fields.reference("roots")?;
        let nodes = r.fields.reference("nodes")?;
        let roots_value = self
            .ids(roots, "component-ids")?
            .into_iter()
            .map(|n| ComponentId::new(n).ok_or(CodecError::InvalidField("component_id")))
            .collect::<Result<_, _>>()?;
        let mut values = Vec::new();
        for reference in self.refs(nodes, "components")? {
            let mut component = self.record(reference, "skin-editor.component")?;
            let id = ComponentId::new(component.fields.identity("id")?)
                .ok_or(CodecError::InvalidField("component_id"))?;
            let local = matrix(component.fields.take("local")?)?;
            let mut node = Fields::new(component.fields.take("node")?)?;
            let mut refs = Vec::new();
            let kind = match node.text("tag")?.as_str() {
                "group" => {
                    let children = node.reference("children")?;
                    refs.push(children);
                    ComponentKind::Group {
                        children: self
                            .ids(children, "component-ids")?
                            .into_iter()
                            .map(|n| {
                                ComponentId::new(n).ok_or(CodecError::InvalidField("component_id"))
                            })
                            .collect::<Result<_, _>>()?,
                    }
                }
                "element" => ComponentKind::Element {
                    document: DocumentId::new(node.identity("document")?)
                        .ok_or(CodecError::InvalidField("document_id"))?,
                    document_to_component: matrix(node.take("mapping")?)?,
                },
                _ => return Err(CodecError::UnsupportedKind),
            };
            node.finish()?;
            component.finish(&refs, &[])?;
            values.push(ComponentNode { id, local, kind });
        }
        r.finish(&[roots, nodes], &[])?;
        Ok(ComponentTree::new(roots_value, values)?)
    }
    fn sound(&mut self, hash: StateHash) -> Result<AudioComponent, CodecError> {
        let mut r = self.record(hash, "skin-editor.sound")?;
        let id = AudioComponentId::new(r.fields.identity("id")?)
            .ok_or(CodecError::InvalidField("sound_id"))?;
        let slot = SoundSlotKey::new(r.fields.text("slot")?)?;
        let asset = AssetId::new(r.fields.identity("asset")?)
            .ok_or(CodecError::InvalidField("asset_id"))?;
        let gain = Gain::new(number(r.fields.take("gain")?)?)?;
        let visible = r.fields.flag("visible")?;
        r.finish(&[], &[])?;
        Ok(AudioComponent::new(id, slot, asset).with_properties(gain, visible))
    }
    fn configuration(&mut self, hash: StateHash) -> Result<SkinConfiguration, CodecError> {
        let mut result = Vec::new();
        for reference in self.refs(hash, "configuration")? {
            let mut r = self.record(reference, "skin-editor.config-record")?;
            let raw = unhex(&r.fields.text("raw")?)?;
            let mut interpretation = Fields::new(r.fields.take("interpretation")?)?;
            let value = match interpretation.text("tag")?.as_str() {
                "opaque" => ConfigInterpretation::Opaque,
                "section" => ConfigInterpretation::Section {
                    name: range(interpretation.take("name")?)?,
                },
                "entry" => ConfigInterpretation::Entry {
                    key: range(interpretation.take("key")?)?,
                    value: range(interpretation.take("value")?)?,
                },
                _ => return Err(CodecError::UnsupportedKind),
            };
            interpretation.finish()?;
            r.finish(&[], &[])?;
            result.push(ConfigRecord::new(raw, value)?);
        }
        Ok(SkinConfiguration::new(result))
    }
    fn output(&mut self, hash: StateHash) -> Result<OutputMapping, CodecError> {
        let mut r = self.record(hash, "skin-editor.output")?;
        let id =
            OutputId::new(r.fields.identity("id")?).ok_or(CodecError::InvalidField("output_id"))?;
        let mut source = Fields::new(r.fields.take("source")?)?;
        let tag = source.text("tag")?;
        let member = source.identity("id")?;
        source.finish()?;
        let key = r.fields.text("key")?;
        let path = OutputPath::new(r.fields.text("path")?)?;
        r.finish(&[], &[])?;
        Ok(match tag.as_str() {
            "image" => OutputMapping::new(
                id,
                DocumentId::new(member).ok_or(CodecError::InvalidField("document_id"))?,
                key,
                path,
            )?,
            "audio" => OutputMapping::for_audio(
                id,
                AudioComponentId::new(member).ok_or(CodecError::InvalidField("sound_id"))?,
                key,
                path,
            )?,
            _ => return Err(CodecError::UnsupportedKind),
        })
    }
}

pub(super) fn decode(
    bundle: &StateBundle,
    limits: FormatLimits,
) -> Result<SkinProject, CodecError> {
    if bundle.blocks.len() > limits.blocks {
        return Err(CodecError::ResourceLimit("blocks"));
    }
    let mut total = 0usize;
    let mut index = BTreeMap::new();
    for block in &bundle.blocks {
        total = total
            .checked_add(block.bytes.len())
            .ok_or(CodecError::ResourceLimit("bytes"))?;
        if block.bytes.len() > limits.block_bytes || total > limits.total_bytes {
            return Err(CodecError::ResourceLimit("bytes"));
        }
        if hash(&block.bytes) != block.hash {
            return Err(CodecError::DigestMismatch);
        }
        if index.insert(block.hash, block).is_some() {
            return Err(CodecError::DuplicateBlock);
        }
    }
    let mut reader = Reader {
        index,
        seen: BTreeSet::new(),
        limits,
        items: 0,
        decoded_bytes: 0,
    };
    let mut root = reader.record(bundle.root, "skin-editor.project")?;
    let project_id = ProjectId::new(root.fields.identity("id")?)
        .ok_or(CodecError::InvalidField("project_id"))?;
    if root.fields.text("semantics")? != STATE_SEMANTICS {
        return Err(CodecError::UnsupportedSchema);
    }
    let assets = root.fields.reference("assets")?;
    let documents = root.fields.reference("documents")?;
    let components = root.fields.reference("components")?;
    let sounds = root.fields.reference("sounds")?;
    let configuration = root.fields.reference("configuration")?;
    let outputs = root.fields.reference("outputs")?;
    root.finish(
        &[
            assets,
            documents,
            components,
            sounds,
            configuration,
            outputs,
        ],
        &[],
    )?;
    let mut project = ProjectDefinition::empty(project_id);
    if project.model_version != ADAPTED_MODEL_VERSION {
        return Err(CodecError::UnsupportedSchema);
    }
    for reference in reader.refs(assets, "assets")? {
        project.assets.push(reader.asset(reference)?);
    }
    for reference in reader.refs(documents, "documents")? {
        project.documents.push(reader.document(reference)?);
    }
    project.components = reader.components(components)?;
    for reference in reader.refs(sounds, "sounds")? {
        project.audio_components.push(reader.sound(reference)?);
    }
    project.configuration = reader.configuration(configuration)?;
    for reference in reader.refs(outputs, "outputs")? {
        project.outputs.push(reader.output(reference)?);
    }
    if reader.seen.len() != reader.index.len() {
        return Err(CodecError::UnusedBlock);
    }
    Ok(SkinProject::from_definition(project)?)
}
