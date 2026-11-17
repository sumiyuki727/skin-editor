//! Extra strict admission for experiments, never historical save authority.
use super::*;

const BLOCKS: usize = 8192;
const ORIGINALS: usize = 8;
type OriginalRoots = [StateHash; MAX_FILE_FACTS_COMMITS];
struct Work<'a> {
    rows: [Option<&'a blocks::Row>; BLOCKS],
    queue: [u16; BLOCKS],
    enqueued: [u8; BLOCKS / 8],
    originals: [BlobRef; ORIGINALS],
    len: usize,
    originals_len: usize,
    head: usize,
    tail: usize,
    actual: u8,
}
// Literal arrays and their destination may coexist. This helper returns unit,
// never a Work. The additional 128 usize slots cover bounded local scalars,
// hash copies, iterator cursors and helper frames; old parser/model work is not
// claimed here. Roots are owned by validate's caller, under the same lease.
pub(super) const WORK_BYTES: usize = 2 * std::mem::size_of::<Work<'static>>()
    + 2 * std::mem::size_of::<OriginalRoots>()
    + 2 * std::mem::size_of::<[BlobRef; ORIGINALS]>()
    + std::mem::size_of::<[usize; 128]>();
const _: () = assert!(BLOCKS <= u16::MAX as usize && WORK_BYTES <= 192 * 1024);

impl Work<'_> {
    fn index(&self, hash: StateHash) -> Result<usize, StorageError> {
        self.rows[..self.len]
            .binary_search_by_key(&hash, |row| row.expect("initialized row").block.hash)
            .map_err(|_| StorageError::Invalid("original closure block"))
    }
    fn enqueue(&mut self, hash: StateHash) -> Result<(), StorageError> {
        let i = self.index(hash)?;
        let bit = 1u8 << (i % 8);
        if self.enqueued[i / 8] & bit == 0 {
            if self.tail == self.queue.len() {
                return Err(StorageError::Limit("original closure queue"));
            }
            self.queue[self.tail] =
                u16::try_from(i).map_err(|_| StorageError::Limit("original closure index"))?;
            self.enqueued[i / 8] |= bit;
            self.tail += 1;
        }
        Ok(())
    }
    fn original(&self, hash: [u8; 32]) -> Result<u8, StorageError> {
        self.originals[..self.originals_len]
            .iter()
            .position(|b| b.sha256() == hash)
            .map(|i| 1u8 << i)
            .ok_or(StorageError::Invalid("original closure dictionary"))
    }
    fn extend(
        &mut self,
        root: StateHash,
        cancel: &CaptureCancellation,
        probe: Probe,
    ) -> Result<(), StorageError> {
        self.head = 0;
        self.tail = 0;
        self.enqueue(root)?;
        while self.head < self.tail {
            check(cancel)?;
            let row = self.rows[usize::from(self.queue[self.head])].expect("initialized row");
            self.head += 1;
            for hash in &row.blobs {
                check(cancel)?;
                self.actual |= self.original(*hash)?;
            }
            for hash in &row.refs {
                check(cancel)?;
                self.enqueue(*hash)?;
            }
            if !row.refs.is_empty() || !row.blobs.is_empty() {
                // Fired only after processing actual graph edges/references.
                probe.step(Stage::ReadOriginalClosure, cancel)?;
            }
        }
        Ok(())
    }
    fn declaration(
        &self,
        root: StateHash,
        cancel: &CaptureCancellation,
    ) -> Result<u8, StorageError> {
        check(cancel)?;
        let index = self.rows[self.index(root)?].expect("initialized row");
        if index.kind != "skin-editor.history-index" || !index.blobs.is_empty() {
            return Err(StorageError::Invalid("original closure index kind"));
        }
        let mut mask = 0;
        for hash in &index.refs {
            check(cancel)?;
            let page = self.rows[self.index(*hash)?].expect("initialized row");
            if page.kind != "skin-editor.history-page" || !page.refs.is_empty() {
                return Err(StorageError::Invalid("original closure page kind"));
            }
            for hash in &page.blobs {
                check(cancel)?;
                mask |= self.original(*hash)?;
            }
        }
        Ok(mask)
    }
}

pub(super) fn validate(
    cache: &Cache,
    roots: &[StateHash],
    cancel: &CaptureCancellation,
    probe: Probe,
) -> Result<(), StorageError> {
    if cache.blocks.len() > BLOCKS
        || cache.originals.len() > ORIGINALS
        || cache.order.len() > MAX_FILE_CAPTURE_NODES
        || roots.len() != cache.checkpoints.len()
        || roots.len() > MAX_FILE_FACTS_COMMITS
        || roots.is_empty()
    {
        return Err(StorageError::Limit("original closure workspace"));
    }
    let mut work = Work {
        rows: [None; BLOCKS],
        queue: [0; BLOCKS],
        enqueued: [0; BLOCKS / 8],
        originals: [BlobRef::new([0; 32], 0); ORIGINALS],
        len: cache.blocks.len(),
        originals_len: cache.originals.len(),
        head: 0,
        tail: 0,
        actual: 0,
    };
    for (i, (hash, row)) in cache.blocks.iter().enumerate() {
        check(cancel)?;
        if *hash != row.block.hash {
            return Err(StorageError::Invalid("original closure hash"));
        }
        work.rows[i] = Some(row);
    }
    for (i, blob) in cache.originals.iter().enumerate() {
        check(cancel)?;
        work.originals[i] = *blob;
    }
    let mut nodes = 0;
    let mut declared = 0;
    for (point, root) in cache.checkpoints.iter().zip(roots) {
        check(cancel)?;
        if point.node_count < nodes || point.node_count == 0 || point.node_count > cache.order.len()
        {
            return Err(StorageError::Invalid("original closure prefix"));
        }
        for id in &cache.order[nodes..point.node_count] {
            check(cancel)?;
            let node = cache
                .nodes
                .get(id)
                .ok_or(StorageError::Invalid("original closure node"))?;
            let original = node
                .info
                .origin
                .as_ref()
                .ok_or(StorageError::Invalid("original closure record"))?;
            work.extend(StateHash::from_bytes(original.state), cancel, probe)?;
        }
        nodes = point.node_count;
        declared |= work.declaration(*root, cancel)?;
        if work.actual != declared {
            return Err(StorageError::Invalid("historical original closure"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn key(id: u64) -> StateHash {
        let mut bytes = [0; 32];
        bytes[24..].copy_from_slice(&id.to_be_bytes());
        StateHash::from_bytes(bytes)
    }
    fn row(id: u64, refs: Vec<StateHash>, blobs: Vec<[u8; 32]>, kind: &str) -> blocks::Row {
        blocks::Row {
            block: StateBlock {
                hash: key(id),
                bytes: vec![],
            },
            kind: kind.to_owned(),
            refs,
            blobs,
        }
    }
    // Deliberately synthetic graph metadata; only the separate real-file tests
    // establish the prerequisite full decoder/record/declaration validation.
    fn model(v: &Value) -> (Cache, Vec<StateHash>) {
        let identity = PersistentHistoryIdentity {
            history: PersistentHistoryId::new(10).unwrap(),
            lineage: LineageId::new(11).unwrap(),
            address_epoch: AddressEpoch::new(1).unwrap(),
        };
        let template = CapturedNodeInfo {
            id: PersistentNodeId::new(1).unwrap(),
            parent: None,
            alias: 1,
            coordinate: BranchCoordinate::new(1, 0).unwrap(),
            depth: 0,
            operation: Arc::new(CapturedOperation {
                version: 1,
                origin: OperationOrigin::Root,
                steps: vec![],
            }),
            changed_objects: vec![],
            origin: None,
        };
        let originals: BTreeSet<_> = v["catalog"]
            .as_array()
            .unwrap()
            .iter()
            .map(|b| {
                let bytes = unhex(b["raw_hex"].as_str().unwrap()).unwrap();
                assert_eq!(hex(&sha(&bytes)), b["sha256"]);
                BlobRef::new(sha(&bytes), b["byte_len"].as_u64().unwrap())
            })
            .collect();
        let dictionary: BTreeMap<_, _> = v["catalog"]
            .as_array()
            .unwrap()
            .iter()
            .map(|b| {
                (
                    b["name"].as_str().unwrap(),
                    hash(b["sha256"].as_str().unwrap()).unwrap().bytes(),
                )
            })
            .collect();
        let mut cache = Cache {
            header: Header {
                project: ProjectId::new(7).unwrap(),
                identity,
                root: template.id,
            },
            blocks: BTreeMap::new(),
            nodes: BTreeMap::new(),
            order: vec![],
            admissions: BTreeMap::new(),
            originals,
            checkpoints: vec![],
            metadata: Arc::default(),
            features: vec![],
            metadata_bytes: 0,
            prefix: [0; 32],
        };
        for b in v["graph"].as_array().unwrap() {
            let id = b["id"].as_u64().unwrap();
            let refs = b["refs"]
                .as_array()
                .unwrap()
                .iter()
                .map(|h| key(h.as_u64().unwrap()))
                .collect();
            let blobs = b["blobs"]
                .as_array()
                .unwrap()
                .iter()
                .map(|n| dictionary[n.as_str().unwrap()])
                .collect();
            cache
                .blocks
                .insert(key(id), row(id, refs, blobs, "synthetic-state"));
        }
        for (i, root) in v["node_roots"].as_array().unwrap().iter().enumerate() {
            let mut info = template.clone();
            info.id = PersistentNodeId::new(i as u128 + 1).unwrap();
            info.origin = Some(Arc::new(OriginalNodeRecord {
                record_version: 2,
                state: key(root.as_u64().unwrap()).bytes(),
                action: vec![],
                checksum: [0; 32],
            }));
            cache.order.push(info.id);
            cache.nodes.insert(
                info.id,
                CachedNode {
                    info,
                    canonical: key(v["canonical_absent"].as_u64().unwrap()),
                },
            );
        }
        let mut roots = vec![];
        for (i, count) in v["node_counts"].as_array().unwrap().iter().enumerate() {
            let id = 100000 + 2 * i as u64;
            let mut blobs: Vec<_> = v["declarations"][i]
                .as_array()
                .unwrap()
                .iter()
                .map(|n| dictionary[n.as_str().unwrap()])
                .collect();
            blobs.sort();
            cache.blocks.insert(
                key(id),
                row(id, vec![key(id + 1)], vec![], "skin-editor.history-index"),
            );
            cache.blocks.insert(
                key(id + 1),
                row(id + 1, vec![], blobs, "skin-editor.history-page"),
            );
            roots.push(key(id));
            cache.checkpoints.push(Checkpoint {
                receipt: ArchiveReceipt {
                    commit: CommitId::new(i as u128 + 1).unwrap(),
                    generation: i as u64 + 1,
                    head: cache.order[0],
                    origin: SaveOrigin::Manual,
                },
                previous: None,
                navigation: key(id),
                prefix: [0; 32],
                node_count: count.as_u64().unwrap() as usize,
                next_alias: 1,
                next_branch: 1,
                redo: vec![],
            });
        }
        (cache, roots)
    }
    fn work(cache: &Cache) -> Work<'_> {
        let mut w = Work {
            rows: [None; BLOCKS],
            queue: [0; BLOCKS],
            enqueued: [0; BLOCKS / 8],
            originals: [BlobRef::new([0; 32], 0); ORIGINALS],
            len: cache.blocks.len(),
            originals_len: cache.originals.len(),
            head: 0,
            tail: 0,
            actual: 0,
        };
        for (i, row) in cache.blocks.values().enumerate() {
            w.rows[i] = Some(row);
        }
        for (i, blob) in cache.originals.iter().enumerate() {
            w.originals[i] = *blob;
        }
        w
    }
    fn vectors() -> Value {
        serde_json::from_str(
            &std::fs::read_to_string(
                Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/P3_2B1_2D_VECTORS.json"),
            )
            .unwrap(),
        )
        .unwrap()
    }
    #[test]
    fn independent_cumulative_models_use_original_roots_and_keep_zero_length_members() {
        let vectors = vectors();
        for v in vectors["cases"].as_array().unwrap() {
            let (cache, roots) = model(v);
            let cancel = CaptureCancellation::default();
            assert!(
                !cache
                    .blocks
                    .contains_key(&key(v["canonical_absent"].as_u64().unwrap()))
            );
            let mut w = work(&cache);
            let mut nodes = 0;
            let mut declared = 0;
            for (i, point) in cache.checkpoints.iter().enumerate() {
                for id in &cache.order[nodes..point.node_count] {
                    w.extend(
                        StateHash::from_bytes(cache.nodes[id].info.origin.as_ref().unwrap().state),
                        &cancel,
                        Probe::default(),
                    )
                    .unwrap();
                }
                nodes = point.node_count;
                declared |= w.declaration(roots[i], &cancel).unwrap();
                assert_eq!(u64::from(w.actual), v["expected"][i]["mask"]);
                assert_eq!(u64::from(declared), v["expected"][i]["declared_mask"]);
                assert_eq!(u64::from(w.actual.count_ones()), v["expected"][i]["count"]);
                let bytes: u64 = cache
                    .originals
                    .iter()
                    .enumerate()
                    .filter(|(j, _)| w.actual & (1u8 << j) != 0)
                    .map(|(_, b)| b.byte_len())
                    .sum();
                assert_eq!(bytes, v["expected"][i]["bytes"]);
            }
            assert_eq!(
                validate(&cache, &roots, &cancel, Probe::default()).is_ok(),
                v["admitted"].as_bool().unwrap()
            );
        }
    }
    #[test]
    fn dense_shared_graph_marks_on_enqueue_instead_of_filling_the_queue_with_duplicates() {
        let v = &vectors()["cases"][0];
        let (mut cache, _) = model(v);
        cache.blocks.clear();
        let raw = cache.originals.first().unwrap().sha256();
        for i in 1..=130 {
            cache.blocks.insert(
                key(i),
                row(
                    i,
                    (1..i).map(key).collect(),
                    if i == 1 { vec![raw] } else { vec![] },
                    "synthetic-state",
                ),
            );
        }
        let mut w = work(&cache);
        w.extend(key(130), &CaptureCancellation::default(), Probe::default())
            .unwrap();
        assert_eq!(w.tail, 130);
        assert_eq!(w.actual, 1);
        assert_eq!(w.enqueued.iter().map(|b| b.count_ones()).sum::<u32>(), 130);
        w.extend(key(130), &CaptureCancellation::default(), Probe::default())
            .unwrap();
        assert_eq!(w.tail, 0);
        assert_eq!(w.actual, 1);
    }
    #[test]
    fn fixed_graph_accepts_8192_and_rejects_overflow_missing_roots_and_dictionary_entries() {
        let v = &vectors()["cases"][0];
        let (mut cache, roots) = model(v);
        cache
            .blocks
            .retain(|_, r| r.kind.starts_with("skin-editor.history-"));
        // Keep exactly one checkpoint index/page and 8190 state blocks.
        cache.checkpoints.truncate(1);
        let roots = &roots[..1];
        cache
            .blocks
            .retain(|h, _| *h == roots[0] || *h == key(100001));
        let raw = cache.originals.first().unwrap().sha256();
        cache.blocks.get_mut(&key(100001)).unwrap().blobs = vec![raw];
        for i in 1..=8190 {
            cache.blocks.insert(
                key(i),
                row(
                    i,
                    if i > 1 { vec![key(i - 1)] } else { vec![] },
                    if i == 1 { vec![raw] } else { vec![] },
                    "synthetic-state",
                ),
            );
        }
        cache.order.truncate(1);
        let node = cache.nodes.get_mut(&cache.order[0]).unwrap();
        Arc::make_mut(node.info.origin.as_mut().unwrap()).state = key(8190).bytes();
        let cancel = CaptureCancellation::default();
        assert_eq!(cache.blocks.len(), 8192);
        assert_eq!(validate(&cache, roots, &cancel, Probe::default()), Ok(()));
        for i in 0u8..7 {
            let raw = vec![i; i as usize];
            cache
                .originals
                .insert(BlobRef::new(sha(&raw), raw.len() as u64));
        }
        assert_eq!(cache.originals.len(), 8);
        let all: Vec<_> = cache.originals.iter().map(|b| b.sha256()).collect();
        cache.blocks.get_mut(&key(1)).unwrap().blobs = all.clone();
        cache.blocks.get_mut(&key(100001)).unwrap().blobs = all;
        assert_eq!(validate(&cache, roots, &cancel, Probe::default()), Ok(()));
        let extra = BlobRef::new(sha(b"ninth"), 5);
        cache.originals.insert(extra);
        assert_eq!(
            validate(&cache, roots, &cancel, Probe::default()),
            Err(StorageError::Limit("original closure workspace"))
        );
        cache.originals.remove(&extra);
        cache
            .blocks
            .insert(key(999998), row(999998, vec![], vec![], "synthetic-state"));
        assert_eq!(
            validate(&cache, roots, &cancel, Probe::default()),
            Err(StorageError::Limit("original closure workspace"))
        );
        cache.blocks.remove(&key(999998));
        cache.blocks.remove(&key(8190));
        assert!(validate(&cache, roots, &cancel, Probe::default()).is_err());
        cache.blocks.insert(
            key(8190),
            row(8190, vec![key(8189)], vec![], "synthetic-state"),
        );
        cache.originals.clear();
        assert!(validate(&cache, roots, &cancel, Probe::default()).is_err());
        assert!(validate(&cache, &[], &cancel, Probe::default()).is_err());
    }
}
