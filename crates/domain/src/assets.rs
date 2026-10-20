use std::collections::BTreeMap;

use crate::{AssetId, ModelError, ObjectId, PixelSize};

/// Content address asserted by the byte store, not computed or verified here.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BlobRef {
    sha256: [u8; 32],
    byte_len: u64,
}

impl BlobRef {
    pub const fn new(sha256: [u8; 32], byte_len: u64) -> Self {
        Self { sha256, byte_len }
    }

    pub const fn sha256(self) -> [u8; 32] {
        self.sha256
    }

    pub const fn byte_len(self) -> u64 {
        self.byte_len
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AssetKind {
    RasterImage(PixelSize),
    /// Uninterpreted resource; cannot be used as an image layer.
    Opaque,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Asset {
    id: AssetId,
    name: String,
    blob: BlobRef,
    kind: AssetKind,
}

impl Asset {
    pub fn new(
        id: AssetId,
        name: String,
        blob: BlobRef,
        kind: AssetKind,
    ) -> Result<Self, ModelError> {
        if name.trim().is_empty() {
            return Err(ModelError::InvalidValue("asset_name"));
        }
        Ok(Self {
            id,
            name,
            blob,
            kind,
        })
    }

    pub const fn id(&self) -> AssetId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub const fn blob(&self) -> BlobRef {
        self.blob
    }

    pub fn kind(&self) -> &AssetKind {
        &self.kind
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AssetLibrary {
    assets: BTreeMap<AssetId, Asset>,
}

impl AssetLibrary {
    pub fn new(assets: Vec<Asset>) -> Result<Self, ModelError> {
        let mut result = BTreeMap::new();
        let mut blobs = BTreeMap::new();
        for asset in assets {
            let id = asset.id;
            let metadata = blobs
                .entry(asset.blob.sha256)
                .or_insert((asset.blob.byte_len, None));
            if metadata.0 != asset.blob.byte_len {
                return Err(ModelError::InvalidValue("conflicting_blob_length"));
            }
            if let AssetKind::RasterImage(size) = asset.kind {
                if metadata.1.is_some_and(|old| old != size) {
                    return Err(ModelError::InvalidValue("conflicting_image_dimensions"));
                }
                metadata.1 = Some(size);
            }
            if result.insert(id, asset).is_some() {
                return Err(ModelError::DuplicateId(ObjectId::Asset(id)));
            }
        }
        Ok(Self { assets: result })
    }

    pub fn get(&self, id: AssetId) -> Option<&Asset> {
        self.assets.get(&id)
    }

    pub fn iter(&self) -> impl ExactSizeIterator<Item = &Asset> {
        self.assets.values()
    }
}
