use super::*;
use serde_json::Map;
pub(super) fn sha(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}
pub(super) fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
pub(super) fn unhex(s: &str) -> Result<Vec<u8>, StorageError> {
    if !s.len().is_multiple_of(2)
        || !s
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(StorageError::Invalid("hex"));
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|_| StorageError::Invalid("hex")))
        .collect()
}
pub(super) fn fixed<const N: usize>(bytes: Vec<u8>) -> Result<[u8; N], StorageError> {
    bytes
        .try_into()
        .map_err(|_| StorageError::Invalid("fixed bytes"))
}
pub(super) fn hash(s: &str) -> Result<StateHash, StorageError> {
    Ok(StateHash::from_bytes(fixed(unhex(s)?)?))
}
pub(super) fn ident(s: &str) -> Result<u128, StorageError> {
    let b: [u8; 16] = fixed(unhex(s)?)?;
    let n = u128::from_be_bytes(b);
    if n == 0 {
        Err(StorageError::Invalid("identity"))
    } else {
        Ok(n)
    }
}
pub(super) fn pid(s: &str) -> Result<PersistentNodeId, StorageError> {
    Ok(PersistentNodeId::new(ident(s)?).expect("nonzero"))
}
pub(super) fn ptext(p: PersistentNodeId) -> String {
    format!("{:032x}", p.get())
}
pub(super) fn decimal(s: &str) -> Result<u64, StorageError> {
    if s.is_empty() || (s.len() > 1 && s.starts_with('0')) || !s.bytes().all(|b| b.is_ascii_digit())
    {
        return Err(StorageError::Invalid("decimal"));
    }
    s.parse().map_err(|_| StorageError::Invalid("decimal"))
}
pub(super) fn bu64(n: u64) -> [u8; 8] {
    n.to_be_bytes()
}
pub(super) fn u64blob(v: Vec<u8>) -> Result<u64, StorageError> {
    Ok(u64::from_be_bytes(fixed(v)?))
}
pub(super) fn nodeblob(v: Vec<u8>) -> Result<PersistentNodeId, StorageError> {
    PersistentNodeId::new(u128::from_be_bytes(fixed(v)?)).ok_or(StorageError::Invalid("node id"))
}
pub(super) fn commitblob(v: Vec<u8>) -> Result<CommitId, StorageError> {
    CommitId::new(u128::from_be_bytes(fixed(v)?)).ok_or(StorageError::Invalid("commit id"))
}
pub(super) fn text(v: Value) -> Result<String, StorageError> {
    v.as_str()
        .map(str::to_owned)
        .ok_or(StorageError::Invalid("text"))
}
pub(super) fn array(v: Value) -> Result<Vec<Value>, StorageError> {
    match v {
        Value::Array(v) => Ok(v),
        _ => Err(StorageError::Invalid("array")),
    }
}
pub(super) struct Fields(Map<String, Value>);
impl Fields {
    pub fn new(v: Value) -> Result<Self, StorageError> {
        match v {
            Value::Object(m) => Ok(Self(m)),
            _ => Err(StorageError::Invalid("object")),
        }
    }
    pub fn take(&mut self, k: &'static str) -> Result<Value, StorageError> {
        self.0.remove(k).ok_or(StorageError::Invalid(k))
    }
    pub fn text(&mut self, k: &'static str) -> Result<String, StorageError> {
        text(self.take(k)?)
    }
    pub fn num(&mut self, k: &'static str) -> Result<u64, StorageError> {
        decimal(&self.text(k)?)
    }
    pub fn id(&mut self, k: &'static str) -> Result<PersistentNodeId, StorageError> {
        pid(&self.text(k)?)
    }
    pub fn hash(&mut self, k: &'static str) -> Result<StateHash, StorageError> {
        hash(&self.text(k)?)
    }
    pub fn finish(self) -> Result<(), StorageError> {
        if self.0.is_empty() {
            Ok(())
        } else {
            Err(StorageError::Unsupported("metadata field"))
        }
    }
}
pub(super) fn save_name(o: SaveOrigin) -> &'static str {
    match o {
        SaveOrigin::Manual => "manual",
        SaveOrigin::Automatic => "automatic",
        SaveOrigin::Exit => "exit",
    }
}
pub(super) fn save_origin(s: &str) -> Result<SaveOrigin, StorageError> {
    match s {
        "manual" => Ok(SaveOrigin::Manual),
        "automatic" => Ok(SaveOrigin::Automatic),
        "exit" => Ok(SaveOrigin::Exit),
        _ => Err(StorageError::Unsupported("save origin")),
    }
}
pub(super) fn operation_bytes(o: &CapturedOperation) -> Result<Vec<u8>, StorageError> {
    let name = match o.origin {
        OperationOrigin::Root => "root",
        OperationOrigin::Command => "command",
        OperationOrigin::Draft => "draft",
    };
    serde_json::to_vec(&json!({"version":o.version,"origin":name,"steps":o.steps}))
        .map_err(|_| StorageError::Invalid("operation"))
}
pub(super) fn operation(
    raw: &[u8],
    limits: FormatLimits,
) -> Result<Arc<CapturedOperation>, StorageError> {
    if raw.len() > 65536 {
        return Err(StorageError::Limit("operation"));
    }
    let mut f = Fields::new(crate::project_format::json::parse(raw, limits)?)?;
    if f.take("version")?.as_u64() != Some(1) {
        return Err(StorageError::Unsupported("operation version"));
    }
    let origin = match f.text("origin")?.as_str() {
        "root" => OperationOrigin::Root,
        "command" => OperationOrigin::Command,
        "draft" => OperationOrigin::Draft,
        _ => return Err(StorageError::Unsupported("operation origin")),
    };
    let steps = array(f.take("steps")?)?
        .into_iter()
        .map(text)
        .collect::<Result<Vec<_>, _>>()?;
    if steps.len() > 1024 || steps.iter().any(|s| s.is_empty() || s.len() > 1024) {
        return Err(StorageError::Limit("operation steps"));
    }
    // Extra descriptive fields stay in OriginalNodeRecord.action, never rewritten.
    Ok(Arc::new(CapturedOperation {
        version: 1,
        origin,
        steps,
    }))
}
pub(super) fn object(o: ObjectId) -> (&'static str, u128) {
    match o {
        ObjectId::Asset(n) => ("asset", n.get()),
        ObjectId::Document(n) => ("document", n.get()),
        ObjectId::Layer(n) => ("layer", n.get()),
        ObjectId::Component(n) => ("component", n.get()),
        ObjectId::AudioComponent(n) => ("sound", n.get()),
        ObjectId::Output(n) => ("output", n.get()),
    }
}
pub(super) fn object_id(kind: &str, id: u128) -> Result<ObjectId, StorageError> {
    Ok(match kind {
        "asset" => ObjectId::Asset(AssetId::new(id).ok_or(StorageError::Invalid("object id"))?),
        "document" => {
            ObjectId::Document(DocumentId::new(id).ok_or(StorageError::Invalid("object id"))?)
        }
        "layer" => ObjectId::Layer(LayerId::new(id).ok_or(StorageError::Invalid("object id"))?),
        "component" => {
            ObjectId::Component(ComponentId::new(id).ok_or(StorageError::Invalid("object id"))?)
        }
        "sound" => ObjectId::AudioComponent(
            AudioComponentId::new(id).ok_or(StorageError::Invalid("object id"))?,
        ),
        "output" => ObjectId::Output(OutputId::new(id).ok_or(StorageError::Invalid("object id"))?),
        _ => return Err(StorageError::Unsupported("object kind")),
    })
}
pub(super) fn node_checksum(
    id: PersistentNodeId,
    parent: Option<PersistentNodeId>,
    state: StateHash,
    action: &[u8],
) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(b"skin-editor.node.v2\0");
    h.update(id.get().to_be_bytes());
    if let Some(p) = parent {
        h.update([1]);
        h.update(p.get().to_be_bytes());
    } else {
        h.update([0]);
    }
    h.update(state.bytes());
    h.update((action.len() as u64).to_be_bytes());
    h.update(action);
    h.finalize().into()
}
pub(super) fn address_checksum(n: &CapturedNodeInfo) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(b"skin-editor.address.v1\0");
    h.update(n.id.get().to_be_bytes());
    for x in [n.alias, n.coordinate.branch(), n.coordinate.ordinal()] {
        h.update(x.to_be_bytes());
    }
    h.finalize().into()
}
pub(super) fn extend_prefix(
    previous: [u8; 32],
    node: &CapturedNodeInfo,
    admissions: &[(ObjectId, PersistentNodeId)],
) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(b"skin-editor.history-prefix.v1\0");
    h.update(previous);
    h.update(node.origin.as_ref().expect("bound record").checksum);
    h.update(address_checksum(node));
    h.update((admissions.len() as u64).to_be_bytes());
    for (o, first) in admissions {
        let (kind, id) = object(*o);
        h.update((kind.len() as u64).to_be_bytes());
        h.update(kind);
        h.update(id.to_be_bytes());
        h.update(first.get().to_be_bytes());
    }
    h.finalize().into()
}
pub(super) fn initial_prefix() -> [u8; 32] {
    sha(b"skin-editor.history-prefix.v1.empty")
}
