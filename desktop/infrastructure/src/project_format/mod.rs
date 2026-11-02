//! Versioned detached state blocks. No filesystem, SQLite, renderer or session writer.
//! This codec is not a project file or a complete persisted history.
pub(crate) mod json;
mod read;
mod write;

use sha2::{Digest, Sha256};
use skin_editor_application::values::{BlobRef, ModelError, SkinProject};
use std::collections::{BTreeMap, BTreeSet};

pub const PROJECT_STATE_SCHEMA: u16 = 1;
pub const STATE_SEMANTICS: &str = "editor-affine-audio-v1";
// A later in-memory model must supply a reviewed adapter, not silently inherit
// new defaults while continuing to claim this schema's old meaning.
const ADAPTED_MODEL_VERSION: u16 = 2;
const PAGE_ITEMS: usize = 256;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct StateHash([u8; 32]);
impl StateHash {
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
    pub const fn bytes(self) -> [u8; 32] {
        self.0
    }
    pub fn hex(self) -> String {
        hex(&self.0)
    }
}

/// Untrusted bytes. Decoding checks the hash, schema, complete closure and model.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StateBlock {
    pub hash: StateHash,
    pub bytes: Vec<u8>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StateBundle {
    pub root: StateHash,
    pub blocks: Vec<StateBlock>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum CodecError {
    InvalidLimits,
    ResourceLimit(&'static str),
    InvalidJson,
    InvalidField(&'static str),
    UnsupportedField,
    UnsupportedSchema,
    UnsupportedKind,
    DigestMismatch,
    MissingBlock,
    DuplicateBlock,
    ReferenceMismatch,
    UnusedBlock,
    Model(ModelError),
}
impl From<ModelError> for CodecError {
    fn from(value: ModelError) -> Self {
        Self::Model(value)
    }
}

/// Codec admission policies, not stable compatibility limits or whole-process budgets.
#[derive(Clone, Copy, Debug)]
pub struct FormatLimits {
    pub block_bytes: usize,
    pub total_bytes: usize,
    /// Sum of parsed block bytes, counting repeated references each time.
    pub decoded_bytes: usize,
    pub blocks: usize,
    pub items: usize,
    pub json_values: usize,
    pub json_depth: usize,
}
impl Default for FormatLimits {
    fn default() -> Self {
        Self {
            block_bytes: 1024 * 1024,
            total_bytes: 16 * 1024 * 1024,
            decoded_bytes: 32 * 1024 * 1024,
            blocks: 131072,
            items: 262144,
            json_values: 65536,
            json_depth: 32,
        }
    }
}
impl FormatLimits {
    fn validate(self) -> Result<Self, CodecError> {
        if self.block_bytes == 0
            || self.total_bytes == 0
            || self.decoded_bytes == 0
            || self.blocks == 0
            || self.items == 0
            || self.json_values == 0
            || self.json_depth == 0
            || self.json_depth > 64
        {
            return Err(CodecError::InvalidLimits);
        }
        Ok(self)
    }
}

pub fn encode_project(
    project: &SkinProject,
    limits: FormatLimits,
) -> Result<StateBundle, CodecError> {
    if project.model_version() != ADAPTED_MODEL_VERSION {
        return Err(CodecError::UnsupportedSchema);
    }
    write::encode(project, limits.validate()?)
}
/// Returns a detached, validated value. It cannot activate or replace a ProjectSession.
pub fn decode_project(
    bundle: &StateBundle,
    limits: FormatLimits,
) -> Result<SkinProject, CodecError> {
    read::decode(bundle, limits.validate()?)
}
/// Original bytes are external to the metadata bundle. Includes unused and opaque assets.
pub fn required_originals(project: &SkinProject) -> Vec<BlobRef> {
    project
        .assets()
        .iter()
        .map(|a| a.blob())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn hash(raw: &[u8]) -> StateHash {
    StateHash(Sha256::digest(raw).into())
}
fn hex(raw: &[u8]) -> String {
    const DIGITS: &[u8] = b"0123456789abcdef";
    let mut result = String::with_capacity(raw.len() * 2);
    for byte in raw {
        result.push(DIGITS[(byte >> 4) as usize] as char);
        result.push(DIGITS[(byte & 15) as usize] as char);
    }
    result
}
fn unhex(value: &str) -> Result<Vec<u8>, CodecError> {
    if !value.len().is_multiple_of(2) {
        return Err(CodecError::InvalidField("hex"));
    }
    fn digit(b: u8) -> Result<u8, CodecError> {
        match b {
            b'0'..=b'9' => Ok(b - b'0'),
            b'a'..=b'f' => Ok(b - b'a' + 10),
            _ => Err(CodecError::InvalidField("hex")),
        }
    }
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|v| Ok(digit(v[0])? * 16 + digit(v[1])?))
        .collect()
}
fn parse_hash(value: &str) -> Result<StateHash, CodecError> {
    if value.len() != 64 {
        return Err(CodecError::InvalidField("state_hash"));
    }
    Ok(StateHash(
        unhex(value)?
            .try_into()
            .map_err(|_| CodecError::InvalidField("state_hash"))?,
    ))
}
fn id(value: u128) -> String {
    format!("{value:032x}")
}
fn parse_id(value: &str) -> Result<u128, CodecError> {
    if value.len() != 32
        || !value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(CodecError::InvalidField("identity"));
    }
    u128::from_str_radix(value, 16)
        .ok()
        .filter(|n| *n != 0)
        .ok_or(CodecError::InvalidField("identity"))
}
fn decimal(value: &str) -> Result<u64, CodecError> {
    if value.is_empty()
        || (value.len() > 1 && value.starts_with('0'))
        || !value.bytes().all(|b| b.is_ascii_digit())
    {
        return Err(CodecError::InvalidField("decimal"));
    }
    value
        .parse()
        .map_err(|_| CodecError::InvalidField("decimal"))
}

struct Writer {
    limits: FormatLimits,
    blocks: Vec<StateBlock>,
    seen: BTreeMap<StateHash, usize>,
    bytes: usize,
    decoded_bytes: usize,
}
impl Writer {
    fn block(
        &mut self,
        kind: &str,
        value: serde_json::Value,
        refs: &[StateHash],
        blobs: &[[u8; 32]],
    ) -> Result<StateHash, CodecError> {
        let refs: BTreeSet<_> = refs.iter().copied().collect();
        let blobs: BTreeSet<_> = blobs.iter().copied().collect();
        let envelope = serde_json::json!({"kind":kind,"schema":1,
            "blocks":refs.iter().map(|h| h.hex()).collect::<Vec<_>>(),
            "blobs":blobs.iter().map(|h| hex(h)).collect::<Vec<_>>(),"value":value});
        let mut pending = vec![(0, &envelope)];
        let mut remaining = self.limits.json_values;
        while let Some((depth, value)) = pending.pop() {
            if depth > self.limits.json_depth || remaining == 0 {
                return Err(CodecError::ResourceLimit("json_structure"));
            }
            remaining -= 1;
            match value {
                serde_json::Value::Array(values) => {
                    pending.extend(values.iter().map(|v| (depth + 1, v)))
                }
                serde_json::Value::Object(values) => {
                    pending.extend(values.values().map(|v| (depth + 1, v)))
                }
                _ => {}
            }
        }
        // Bound the output before retaining it, including JSON escaping.
        let mut output = LimitedOutput {
            bytes: Vec::new(),
            limit: self.limits.block_bytes,
        };
        serde_json::to_writer(&mut output, &envelope)
            .map_err(|_| CodecError::ResourceLimit("block_bytes"))?;
        let raw = output.bytes;
        if raw.len() > self.limits.decoded_bytes.saturating_sub(self.decoded_bytes) {
            return Err(CodecError::ResourceLimit("decoded_bytes"));
        }
        self.decoded_bytes += raw.len();
        let key = hash(&raw);
        if let Some(index) = self.seen.get(&key) {
            if self.blocks[*index].bytes != raw {
                return Err(CodecError::DigestMismatch);
            }
            return Ok(key);
        }
        if self.blocks.len() >= self.limits.blocks
            || raw.len() > self.limits.total_bytes.saturating_sub(self.bytes)
        {
            return Err(CodecError::ResourceLimit("bundle"));
        }
        self.bytes += raw.len();
        self.seen.insert(key, self.blocks.len());
        self.blocks.push(StateBlock {
            hash: key,
            bytes: raw,
        });
        Ok(key)
    }
    fn sequence(
        &mut self,
        role: &str,
        values: Vec<String>,
        references: bool,
    ) -> Result<StateHash, CodecError> {
        if values.len() > self.limits.items {
            return Err(CodecError::ResourceLimit("sequence"));
        }
        let mut pages = Vec::new();
        for chunk in values.chunks(PAGE_ITEMS) {
            let refs = if references {
                chunk
                    .iter()
                    .map(|v| parse_hash(v))
                    .collect::<Result<Vec<_>, _>>()?
            } else {
                Vec::new()
            };
            pages.push(self.block(
                "skin-editor.page",
                serde_json::json!({"role":role,"items":chunk}),
                &refs,
                &[],
            )?);
        }
        self.block(
            "skin-editor.sequence",
            serde_json::json!({"role":role,"len":values.len().to_string(),
            "pages":pages.iter().map(|v| v.hex()).collect::<Vec<_>>()}),
            &pages,
            &[],
        )
    }
}
struct LimitedOutput {
    bytes: Vec<u8>,
    limit: usize,
}
impl std::io::Write for LimitedOutput {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.len() > self.limit.saturating_sub(self.bytes.len()) {
            return Err(std::io::Error::other("state block limit"));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
