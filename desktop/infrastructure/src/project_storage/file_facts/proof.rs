//! FileCoord-Proof-v1 field stream; no runtime ticket or write qualification.
use super::super::*;

pub(super) struct Encoder {
    hash: Sha256,
    tag: u16,
    failed: bool,
}
impl Encoder {
    pub fn new(domain: u16) -> Self {
        Self::scoped(b"skin-editor/FileCoord-Proof/v1\0", domain)
    }
    pub fn scoped(prefix: &[u8], domain: u16) -> Self {
        let mut hash = Sha256::new();
        hash.update(prefix);
        hash.update(domain.to_be_bytes());
        Self {
            hash,
            tag: 0,
            failed: false,
        }
    }
    pub fn bytes(&mut self, value: &[u8]) {
        let Some(tag) = self.tag.checked_add(1) else {
            self.failed = true;
            return;
        };
        self.tag = tag;
        self.hash.update(tag.to_be_bytes());
        self.hash.update((value.len() as u64).to_be_bytes());
        self.hash.update(value);
    }
    pub fn number(&mut self, value: u64) {
        self.bytes(&value.to_be_bytes());
    }
    pub fn id(&mut self, value: u128) {
        self.bytes(&value.to_be_bytes());
    }
    pub fn optional_id(&mut self, value: Option<u128>) {
        if let Some(value) = value {
            let mut encoded = [0u8; 17];
            encoded[0] = 1;
            encoded[1..].copy_from_slice(&value.to_be_bytes());
            self.bytes(&encoded);
        } else {
            self.bytes(&[0]);
        }
    }
    pub fn receipt(&mut self, value: &ArchiveReceipt) {
        self.id(value.commit.get());
        self.number(value.generation);
        self.number(match value.origin {
            SaveOrigin::Manual => 1,
            SaveOrigin::Automatic => 2,
            SaveOrigin::Exit => 3,
        });
        self.id(value.head.get());
    }
    pub fn optional_receipt(&mut self, value: Option<&ArchiveReceipt>) {
        self.number(u64::from(value.is_some()));
        if let Some(value) = value {
            self.receipt(value);
        }
    }
    pub fn finish(self) -> Result<[u8; 32], StorageError> {
        if self.failed {
            Err(StorageError::Limit("proof fields"))
        } else {
            Ok(self.hash.finalize().into())
        }
    }
}

pub(super) fn metadata(
    metadata: &ArchiveMetadata,
    features: &[Value],
) -> Result<[u8; 32], StorageError> {
    let mut h = Encoder::new(3);
    h.optional_receipt(metadata.last_commit.as_ref());
    h.optional_receipt(metadata.manual.as_ref());
    h.optional_receipt(metadata.protected.as_ref());
    h.number(metadata.manual_marks.len() as u64);
    for (commit, node) in &metadata.manual_marks {
        h.id(commit.get());
        h.id(node.get());
    }
    h.number(metadata.tags.len() as u64);
    for tag in &metadata.tags {
        h.id(tag.node.get());
        h.bytes(tag.name.as_bytes());
        h.bytes(tag.value.as_bytes());
    }
    h.number(metadata.extensions.len() as u64);
    for ext in &metadata.extensions {
        h.bytes(ext.name.as_bytes());
        h.bytes(&ext.bytes);
    }
    h.number(features.len() as u64);
    for feature in features {
        h.bytes(
            feature["name"]
                .as_str()
                .ok_or(StorageError::Invalid("feature name"))?
                .as_bytes(),
        );
        h.bytes(
            feature["version"]
                .as_str()
                .ok_or(StorageError::Invalid("feature version"))?
                .as_bytes(),
        );
        h.number(u64::from(
            feature["required"]
                .as_bool()
                .ok_or(StorageError::Invalid("feature required"))?,
        ));
    }
    h.finish()
}

pub(super) const CONTENT_WORK_BYTES: usize = 2 * std::mem::size_of::<Encoder>()
    + std::mem::size_of::<[u128; 5]>()
    + std::mem::size_of::<[u64; 5]>()
    + std::mem::size_of::<[u8; 17]>()
    + 3 * std::mem::size_of::<[u8; 32]>();

pub(super) fn content_node(
    h: &mut Encoder,
    n: &CapturedNodeInfo,
    canonical: StateHash,
) -> Result<(), StorageError> {
    let record = n.origin.as_ref().ok_or(StorageError::Invalid("record"))?;
    h.id(n.id.get());
    h.optional_id(n.parent.map(PersistentNodeId::get));
    for value in [
        n.alias,
        n.coordinate.branch(),
        n.coordinate.ordinal(),
        n.depth,
        u64::from(record.record_version),
    ] {
        h.number(value);
    }
    h.bytes(&record.state);
    h.bytes(&record.action);
    h.bytes(&record.checksum);
    h.bytes(&canonical.bytes());
    Ok(())
}
pub(super) fn content(
    cache: &Cache,
    nodes: usize,
    cancel: &CaptureCancellation,
) -> Result<([u8; 32], usize), StorageError> {
    check(cancel)?;
    let prefix = cache
        .order
        .get(..nodes)
        .filter(|p| !p.is_empty())
        .ok_or(StorageError::Invalid("content prefix"))?;
    let mut admissions = 0usize;
    for first in cache.admissions.values() {
        check(cancel)?;
        if prefix.contains(first) {
            admissions = admissions
                .checked_add(1)
                .ok_or(StorageError::Limit("content admissions"))?;
        }
    }
    let mut h = Encoder::new(1);
    for id in [
        cache.header.project.get(),
        cache.header.identity.history.get(),
        cache.header.identity.lineage.get(),
        cache.header.identity.address_epoch.get(),
        cache.header.root.get(),
    ] {
        h.id(id);
    }
    h.number(u64::try_from(nodes).map_err(|_| StorageError::Limit("content nodes"))?);
    for id in prefix {
        check(cancel)?;
        let node = &cache.nodes[id];
        content_node(&mut h, &node.info, node.canonical)?;
    }
    h.number(u64::try_from(admissions).map_err(|_| StorageError::Limit("content admissions"))?);
    for kind in ["asset", "component", "document", "layer", "output", "sound"] {
        for (value, first) in &cache.admissions {
            check(cancel)?;
            let (found, id) = object(*value);
            if found == kind && prefix.contains(first) {
                h.bytes(kind.as_bytes());
                h.id(id);
                h.id(first.get());
            }
        }
    }
    Ok((h.finish()?, admissions))
}

pub(super) fn build(
    cache: &Cache,
    selected: &RestoreHeader,
    selection: RestoreSelection,
    limits: FileLimits,
    cancel: &CaptureCancellation,
) -> Result<FileFactsProof, StorageError> {
    let (content, _) = content(cache, cache.order.len(), cancel)?;
    let (originals, _) = original_set(
        cache.originals.iter().copied(),
        cache.originals.len(),
        cancel,
    )?;
    let metadata = metadata(&cache.metadata, &cache.features)?;
    let mut h = Encoder::new(4);
    h.number(cache.checkpoints.len() as u64);
    for point in &cache.checkpoints {
        h.receipt(&point.receipt);
        h.optional_id(point.previous.map(CommitId::get));
        h.bytes(&read::checkpoint(cache, point.navigation, limits)?.request);
        h.bytes(&point.navigation.bytes());
        h.bytes(&point.prefix);
        h.number(point.node_count as u64);
        h.number(point.next_alias);
        h.number(point.next_branch);
        // Parsed checkpoint redo is validated but not stored sorted in all inputs.
        let mut redo = point.redo.clone();
        redo.sort_unstable();
        h.number(redo.len() as u64);
        for (p, c) in redo {
            h.id(p.get());
            h.id(c.get());
        }
    }
    let request = h.finish()?;
    let mut h = Encoder::new(5);
    for field in [&content, &originals, &metadata, &request] {
        h.bytes(field);
    }
    h.number(match selection {
        RestoreSelection::Manual => 1,
        RestoreSelection::Latest => 2,
    });
    h.id(selected.head.get());
    h.number(selected.redo_preferences.len() as u64);
    for (p, c) in &selected.redo_preferences {
        h.id(p.get());
        h.id(c.get());
    }
    Ok(FileFactsProof {
        version: FILE_COORD_PROOF_VERSION,
        content,
        originals,
        metadata,
        request,
        baseline: h.finish()?,
    })
}

pub(super) const ORIGINAL_WORK_BYTES: usize = 2 * std::mem::size_of::<Encoder>()
    + 2 * std::mem::size_of::<Option<BlobRef>>()
    + 2 * std::mem::size_of::<[u8; 32]>()
    + std::mem::size_of::<[u64; 4]>()
    + std::mem::size_of::<[usize; 8]>();
pub(super) fn original_set(
    originals: impl Iterator<Item = BlobRef>,
    count: usize,
    cancel: &CaptureCancellation,
) -> Result<([u8; 32], u64), StorageError> {
    check(cancel)?;
    let mut h = Encoder::new(2);
    h.number(u64::try_from(count).map_err(|_| StorageError::Limit("original proof count"))?);
    let mut previous: Option<BlobRef> = None;
    let mut seen = 0usize;
    let mut bytes = 0u64;
    for original in originals {
        check(cancel)?;
        if seen == count || previous.is_some_and(|p| p.sha256() >= original.sha256()) {
            return Err(StorageError::Invalid("original proof order/count"));
        }
        bytes = bytes
            .checked_add(original.byte_len())
            .ok_or(StorageError::Limit("original proof bytes"))?;
        h.bytes(&original.sha256());
        h.number(original.byte_len());
        seen += 1;
        previous = Some(original);
    }
    if seen != count {
        return Err(StorageError::Invalid("original proof count"));
    }
    Ok((h.finish()?, bytes))
}

#[cfg(test)]
mod original_tests {
    use super::*;
    #[test]
    fn independent_original_vectors_and_invalid_streams_keep_exact_domain_two() {
        let vectors: Value = serde_json::from_str(
            &std::fs::read_to_string(
                Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/P3_2B1_2E_VECTORS.json"),
            )
            .unwrap(),
        )
        .unwrap();
        for case in vectors["cases"].as_array().unwrap() {
            let refs: Vec<_> = case["entries"]
                .as_array()
                .unwrap()
                .iter()
                .map(|b| {
                    BlobRef::new(
                        hash(b["sha256"].as_str().unwrap()).unwrap().bytes(),
                        b["byte_len"].as_u64().unwrap(),
                    )
                })
                .collect();
            let (digest, bytes) = original_set(
                refs.into_iter(),
                case["len"].as_u64().unwrap() as usize,
                &CaptureCancellation::default(),
            )
            .unwrap();
            assert_eq!(hex(&digest), case["sha256"]);
            assert_eq!(bytes, case["bytes"]);
        }
        let a = BlobRef::new([1; 32], u64::MAX);
        let b = BlobRef::new([2; 32], 1);
        let cancel = CaptureCancellation::default();
        assert!(original_set([a, a].into_iter(), 2, &cancel).is_err());
        assert!(original_set([b, a].into_iter(), 2, &cancel).is_err());
        assert!(original_set([b].into_iter(), 0, &cancel).is_err());
        assert!(original_set([b].into_iter(), 2, &cancel).is_err());
        assert_eq!(
            original_set([a, b].into_iter(), 2, &cancel),
            Err(StorageError::Limit("original proof bytes"))
        );
        cancel.cancel();
        assert_eq!(
            original_set(std::iter::empty(), 0, &cancel),
            Err(StorageError::Cancelled)
        );
    }
}
