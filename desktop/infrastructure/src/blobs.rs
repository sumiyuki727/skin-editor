//! Shared original bytes, independent of image/PCM caches and project metadata.
use crate::byte_budget::{ByteBudget, ByteLease};
use sha2::{Digest, Sha256};
use skin_editor_application::{
    BlobLease, ResourceError, ResourceStore,
    values::{AudioMetadata, BlobRef},
};
use std::collections::BTreeMap;
use std::io::{Read, Seek, SeekFrom};
use std::sync::{Arc, Mutex, Weak};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BlobError {
    ResourceLimit,
    Unavailable,
    Io,
    SourceChanged,
    HashCollision,
    MetadataMismatch,
    Unsupported,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BlobUsage {
    pub bytes: usize,
    pub peak_bytes: usize,
    pub entries: usize,
}
struct Entry {
    reference: BlobRef,
    bytes: Vec<u8>,
    metadata: Mutex<Option<AudioMetadata>>,
    _lease: ByteLease,
}
impl BlobLease for Entry {
    fn reference(&self) -> BlobRef {
        self.reference
    }
}
#[derive(Clone)]
pub struct RawBlob(Arc<Entry>);
impl RawBlob {
    pub fn reference(&self) -> BlobRef {
        self.0.reference
    }
    pub fn bytes(&self) -> &[u8] {
        &self.0.bytes
    }
    pub fn lease(&self) -> Arc<dyn BlobLease> {
        self.0.clone()
    }
    pub fn same_allocation(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
    pub fn audio_metadata(&self) -> Option<AudioMetadata> {
        *self.0.metadata.lock().ok()?
    }
    pub(crate) fn attest_audio(&self, metadata: AudioMetadata) -> Result<(), BlobError> {
        let mut current = self.0.metadata.lock().map_err(|_| BlobError::Unavailable)?;
        if current.is_some_and(|old| old != metadata) {
            return Err(BlobError::MetadataMismatch);
        }
        *current = Some(metadata);
        Ok(())
    }
}
#[derive(Clone)]
pub struct BlobStore {
    entries: Arc<Mutex<BTreeMap<BlobRef, Weak<Entry>>>>,
    budget: Arc<ByteBudget>,
    per_blob: usize,
}
impl Default for BlobStore {
    fn default() -> Self {
        Self::with_limits(128 * 1024 * 1024, 32 * 1024 * 1024).expect("positive defaults")
    }
}
impl BlobStore {
    pub fn observe_bytes(&self) -> crate::ByteObservation {
        self.budget.observation()
    }
    /// Explicit original-byte import; does not claim codec support.
    pub fn stage_original_file(&self, path: &std::path::Path) -> Result<RawBlob, BlobError> {
        let file = std::fs::File::open(path).map_err(|_| BlobError::Io)?;
        let len = file.metadata().map_err(|_| BlobError::Io)?.len();
        self.stage_reader(file, len)
    }
    pub fn with_limits(total: usize, per_blob: usize) -> Result<Self, BlobError> {
        if total == 0 || per_blob == 0 || per_blob > total {
            return Err(BlobError::ResourceLimit);
        }
        Ok(Self {
            entries: Arc::default(),
            budget: ByteBudget::new(total),
            per_blob,
        })
    }
    pub fn get(&self, reference: BlobRef) -> Option<RawBlob> {
        self.entries
            .lock()
            .ok()?
            .get(&reference)?
            .upgrade()
            .map(RawBlob)
    }
    pub fn usage(&self) -> Result<BlobUsage, BlobError> {
        let mut entries = self.entries.lock().map_err(|_| BlobError::Unavailable)?;
        entries.retain(|_, v| v.strong_count() > 0);
        Ok(BlobUsage {
            bytes: self.budget.used(),
            peak_bytes: self.budget.peak(),
            entries: entries.len(),
        })
    }
    fn insert(
        &self,
        bytes: Vec<u8>,
        reference: BlobRef,
        lease: ByteLease,
    ) -> Result<RawBlob, BlobError> {
        let mut entries = self.entries.lock().map_err(|_| BlobError::Unavailable)?;
        entries.retain(|_, v| v.strong_count() > 0);
        if let Some(old) = entries.get(&reference).and_then(Weak::upgrade) {
            if old.bytes != bytes {
                return Err(BlobError::HashCollision);
            }
            return Ok(RawBlob(old));
        }
        if entries.len() >= 4096 {
            return Err(BlobError::ResourceLimit);
        }
        // Moving Vec into Arc<Entry> does not copy its byte allocation.
        let entry = Arc::new(Entry {
            reference,
            bytes,
            metadata: Mutex::new(None),
            _lease: lease,
        });
        entries.insert(reference, Arc::downgrade(&entry));
        Ok(RawBlob(entry))
    }
    /// Adopts caller-owned input. Its capacity, including spare capacity, counts.
    /// New file reads should use stage_reader to reserve before allocation.
    pub fn stage_bytes(&self, bytes: Vec<u8>) -> Result<RawBlob, BlobError> {
        if bytes.len() > self.per_blob {
            return Err(BlobError::ResourceLimit);
        }
        let reference = BlobRef::new(Sha256::digest(&bytes).into(), bytes.len() as u64);
        if let Some(old) = self.get(reference) {
            if old.bytes() != bytes {
                return Err(BlobError::HashCollision);
            }
            return Ok(old);
        }
        let lease = self
            .budget
            .reserve(bytes.capacity())
            .ok_or(BlobError::ResourceLimit)?;
        self.insert(bytes, reference, lease)
    }
    /// Two bounded passes permit deduplication at a full pool. Fixed 16 KiB
    /// hash scratch is separate from stored-byte capacity; no unbounded read.
    pub fn stage_reader(
        &self,
        mut reader: impl Read + Seek,
        byte_len: u64,
    ) -> Result<RawBlob, BlobError> {
        let len = usize::try_from(byte_len).map_err(|_| BlobError::ResourceLimit)?;
        if len > self.per_blob {
            return Err(BlobError::ResourceLimit);
        }
        reader.seek(SeekFrom::Start(0)).map_err(|_| BlobError::Io)?;
        let mut hasher = Sha256::new();
        let mut scratch = [0u8; 16 * 1024];
        let mut left = len;
        while left > 0 {
            let n = left.min(scratch.len());
            reader
                .read_exact(&mut scratch[..n])
                .map_err(|_| BlobError::Io)?;
            hasher.update(&scratch[..n]);
            left -= n;
        }
        let mut extra = [0];
        if reader.read(&mut extra).map_err(|_| BlobError::Io)? != 0 {
            return Err(BlobError::SourceChanged);
        }
        let reference = BlobRef::new(hasher.finalize().into(), byte_len);
        if let Some(old) = self.get(reference) {
            return Ok(old);
        }
        let lease = self.budget.reserve(len).ok_or(BlobError::ResourceLimit)?;
        reader.seek(SeekFrom::Start(0)).map_err(|_| BlobError::Io)?;
        let mut bytes = vec![0; len];
        reader.read_exact(&mut bytes).map_err(|_| BlobError::Io)?;
        if reader.read(&mut extra).map_err(|_| BlobError::Io)? != 0
            || <[u8; 32]>::from(Sha256::digest(&bytes)) != reference.sha256()
        {
            return Err(BlobError::SourceChanged);
        }
        self.insert(bytes, reference, lease)
    }
}
impl ResourceStore for BlobStore {
    fn pin(&self, blob: BlobRef) -> Result<Arc<dyn BlobLease>, ResourceError> {
        self.get(blob)
            .map(|b| b.lease())
            .ok_or(ResourceError::Missing)
    }
    fn audio_metadata(&self, blob: BlobRef) -> Option<AudioMetadata> {
        self.get(blob)?.audio_metadata()
    }
}
