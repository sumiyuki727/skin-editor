//! File-side metadata from the old verifier's borrowed, validated checkpoint.
use super::*;
const ITEMS: usize = 8192;
type Workspace = [u16; ITEMS];
#[derive(Clone, Copy)]
pub(super) struct Point {
    digest: [u8; 32],
    features_root: [u8; 32],
    counts: [u16; 5], // tags, extensions, features, required features, marks
    extension_bytes: u32,
}
pub(super) struct MetadataData {
    points: [Point; MAX_FILE_FACTS_COMMITS],
    len: usize,
}
impl MetadataData {
    pub const fn empty() -> Self {
        Self {
            points: [Point {
                digest: [0; 32],
                features_root: [0; 32],
                counts: [0; 5],
                extension_bytes: 0,
            }; MAX_FILE_FACTS_COMMITS],
            len: 0,
        }
    }
    pub fn matches_latest(&self, cache: &Cache, digest: [u8; 32]) -> bool {
        let Some(p) = self.points.get(self.len.wrapping_sub(1)) else {
            return false;
        };
        self.len == cache.checkpoints.len()
            && p.digest == digest
            && usize::from(p.counts[0]) == cache.metadata.tags.len()
            && usize::from(p.counts[1]) == cache.metadata.extensions.len()
            && usize::from(p.counts[2]) == cache.features.len()
            && usize::from(p.counts[3])
                == cache
                    .features
                    .iter()
                    .filter(|f| f["required"].as_bool() == Some(true))
                    .count()
            && usize::from(p.counts[4]) == cache.metadata.manual_marks.len()
            && Some(u64::from(p.extension_bytes))
                == cache
                    .metadata
                    .extensions
                    .iter()
                    .try_fold(0u64, |sum, e| sum.checked_add(e.bytes.len() as u64))
    }
}
// Two literal workspace copies may coexist during construction. MetadataData
// producer/destination overlap is separately named in 2Data's static bound.
pub(super) const WORK_BYTES: usize = 2 * std::mem::size_of::<Workspace>()
    + 2 * std::mem::size_of::<proof::Encoder>()
    + 2 * std::mem::size_of::<Point>()
    + 2 * std::mem::size_of::<read::MetadataView<'static>>()
    + std::mem::size_of::<[usize; 64]>()
    + 2 * std::mem::size_of::<[u8; 32]>();
const _: () = assert!(
    std::mem::size_of::<Point>() <= 80 && WORK_BYTES <= 40 * 1024 && ITEMS <= u16::MAX as usize
);

#[derive(Clone, Copy)]
enum Kind {
    Tags,
    Extensions,
    Features,
}
fn text<'a>(value: &'a Value, key: &str) -> Result<&'a str, StorageError> {
    value[key]
        .as_str()
        .ok_or(StorageError::Invalid("metadata text"))
}
fn key(value: &Value, kind: Kind) -> Result<(u128, &str, &str), StorageError> {
    let name = text(value, "name")?;
    Ok(match kind {
        Kind::Tags => {
            let node = u128::from_str_radix(text(value, "node")?, 16)
                .map_err(|_| StorageError::Invalid("metadata node"))?;
            text(value, "value")?;
            (node, name, "")
        }
        Kind::Extensions => (0, name, ""),
        Kind::Features => (0, name, text(value, "version")?),
    })
}
fn order<'a>(
    values: &[Value],
    kind: Kind,
    work: &'a mut Workspace,
    cancel: &CaptureCancellation,
) -> Result<&'a [u16], StorageError> {
    if values.len() > ITEMS {
        return Err(StorageError::Limit("historical metadata items"));
    }
    for (i, v) in values.iter().enumerate() {
        check(cancel)?;
        key(v, kind)?;
        work[i] = u16::try_from(i).map_err(|_| StorageError::Limit("metadata index"))?;
    }
    work[..values.len()].sort_unstable_by_key(|i| {
        key(&values[usize::from(*i)], kind).expect("prechecked borrowed key")
    });
    check(cancel)?;
    Ok(&work[..values.len()])
}
fn encode(
    view: &read::MetadataView<'_>,
    work: &mut Workspace,
    cancel: &CaptureCancellation,
) -> Result<Point, StorageError> {
    check(cancel)?;
    if view.extensions.len() > 1024
        || view.features.len() > 256
        || view.metadata.manual_marks.len() > MAX_FILE_FACTS_COMMITS
    {
        return Err(StorageError::Limit("historical metadata profile"));
    }
    let mut h = proof::metadata_header(view.metadata);
    h.number(view.tags.len() as u64);
    for i in order(view.tags, Kind::Tags, work, cancel)? {
        check(cancel)?;
        let v = &view.tags[usize::from(*i)];
        let (node, name, _) = key(v, Kind::Tags)?;
        proof::metadata_tag(&mut h, node, name, text(v, "value")?);
    }
    let mut extension_bytes = 0u32;
    h.number(view.extensions.len() as u64);
    for i in order(view.extensions, Kind::Extensions, work, cancel)? {
        check(cancel)?;
        let name = text(&view.extensions[usize::from(*i)], "name")?;
        // The old verifier just bound this declaration's SHA and length to
        // these actual global bytes. Borrow only the declared member.
        let pos = view
            .metadata
            .extensions
            .binary_search_by(|e| e.name.as_str().cmp(name))
            .map_err(|_| StorageError::Invalid("metadata extension source"))?;
        let e = &view.metadata.extensions[pos];
        extension_bytes = extension_bytes
            .checked_add(
                u32::try_from(e.bytes.len())
                    .map_err(|_| StorageError::Limit("metadata extension bytes"))?,
            )
            .ok_or(StorageError::Limit("metadata extension bytes"))?;
        proof::metadata_extension(&mut h, name, &e.bytes);
    }
    let mut required = 0u16;
    h.number(view.features.len() as u64);
    for i in order(view.features, Kind::Features, work, cancel)? {
        check(cancel)?;
        let v = &view.features[usize::from(*i)];
        proof::metadata_feature(&mut h, v)?;
        required += u16::from(
            v["required"]
                .as_bool()
                .ok_or(StorageError::Invalid("metadata required"))?,
        );
    }
    let count = |n| u16::try_from(n).map_err(|_| StorageError::Limit("metadata count"));
    Ok(Point {
        digest: h.finish()?,
        features_root: view.features_root.bytes(),
        counts: [
            count(view.tags.len())?,
            count(view.extensions.len())?,
            count(view.features.len())?,
            required,
            count(view.metadata.manual_marks.len())?,
        ],
        extension_bytes,
    })
}
pub(super) struct Collector<'a> {
    data: &'a mut MetadataData,
    work: Workspace,
}
impl<'a> Collector<'a> {
    pub fn new(data: &'a mut MetadataData) -> Self {
        Self {
            data,
            work: [0; ITEMS],
        }
    }
}
impl read::MetadataObserver for Collector<'_> {
    fn checkpoint(
        &mut self,
        view: read::MetadataView<'_>,
        cancel: &CaptureCancellation,
        probe: Probe,
    ) -> Result<(), StorageError> {
        if self.data.len >= MAX_FILE_FACTS_COMMITS
            || view.receipt.generation != self.data.len as u64 + 1
            || view.metadata.last_commit.as_ref() != Some(view.receipt)
            || view.metadata.protected.as_ref() != Some(view.receipt)
        {
            return Err(StorageError::Invalid("metadata checkpoint scope"));
        }
        let point = encode(&view, &mut self.work, cancel)?;
        self.data.points[self.data.len] = point;
        self.data.len += 1;
        // Actual completed work, but no partially validated result is public.
        probe.step(Stage::ReadMetadataSummary, cancel)
    }
}
impl FileHistoricalMetadataPort for InspectedFileFacts {
    fn historical_metadata(
        &self,
        request: FileHistoricalMetadataRequest,
    ) -> Result<FileHistoricalMetadataResult, FileFactsError> {
        if request.version != FILE_HISTORICAL_METADATA_VERSION
            || request.file.version != FILE_FACTS_PORT_VERSION
        {
            return Err(FileFactsError::UnsupportedVersion);
        }
        let s = self.summary();
        if request.observation != s.observation || request.file != s.request {
            return Err(FileFactsError::StalePage);
        }
        let facts = self.data.commits[..s.commits]
            .iter()
            .position(|c| c.is_some_and(|c| c.fact.commit == request.commit))
            .map(|i| {
                let p = self.data.metadata.points[i];
                FileHistoricalMetadataFacts {
                    proof_version: FILE_COORD_PROOF_VERSION,
                    commit: self.data.commits[i].expect("validated fixed commit"),
                    metadata: p.digest,
                    features_root: p.features_root,
                    tags: usize::from(p.counts[0]),
                    extensions: usize::from(p.counts[1]),
                    features: usize::from(p.counts[2]),
                    required_features: usize::from(p.counts[3]),
                    manual_marks: usize::from(p.counts[4]),
                    extension_bytes: u64::from(p.extension_bytes),
                }
            });
        Ok(FileHistoricalMetadataResult {
            version: FILE_HISTORICAL_METADATA_VERSION,
            request,
            assurance: FileFactsAssurance::ExperimentalValidation,
            facts,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn receipt(v: &Value) -> Option<ArchiveReceipt> {
        (!v.is_null()).then(|| ArchiveReceipt {
            commit: CommitId::new(v["commit"].as_u64().unwrap() as u128).unwrap(),
            generation: v["generation"].as_u64().unwrap(),
            origin: match v["origin"].as_u64().unwrap() {
                1 => SaveOrigin::Manual,
                2 => SaveOrigin::Automatic,
                _ => SaveOrigin::Exit,
            },
            head: PersistentNodeId::new(v["head"].as_u64().unwrap() as u128).unwrap(),
        })
    }
    #[test]
    fn historical_domain_three_matches_all_independent_models_without_latest_substitution() {
        let vectors: Value = serde_json::from_str(
            &std::fs::read_to_string(
                Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/P3_2B1_2G_VECTORS.json"),
            )
            .unwrap(),
        )
        .unwrap();
        for case in vectors["cases"].as_array().unwrap() {
            let m = &case["model"];
            let mut meta = ArchiveMetadata {
                last_commit: receipt(&m["latest"]),
                manual: receipt(&m["manual"]),
                protected: receipt(&m["protected"]),
                manual_marks: m["marks"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| {
                        (
                            CommitId::new(v["commit"].as_u64().unwrap() as u128).unwrap(),
                            PersistentNodeId::new(v["node"].as_u64().unwrap() as u128).unwrap(),
                        )
                    })
                    .collect(),
                // Deliberately unrelated global tags: only borrowed old tags count.
                tags: vec![ArchiveTag {
                    node: PersistentNodeId::new(1).unwrap(),
                    name: "user:wrong-latest".into(),
                    value: "wrong".into(),
                }],
                extensions: m["extensions"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| ArchiveExtension {
                        name: v["name"].as_str().unwrap().into(),
                        bytes: unhex(v["bytes_hex"].as_str().unwrap()).unwrap(),
                    })
                    .collect(),
            };
            meta.extensions.sort_by(|a, b| a.name.cmp(&b.name));
            let tags: Vec<_> = m["tags"].as_array().unwrap().iter().map(|v| json!({"node":format!("{:032x}",v["node"].as_u64().unwrap()),"name":v["name"],"value":v["value"]})).collect();
            let extensions: Vec<_> = m["extensions"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| {
                    let raw = unhex(v["bytes_hex"].as_str().unwrap()).unwrap();
                    json!({"name":v["name"],"sha":hex(&sha(&raw)),"bytes":raw.len().to_string()})
                })
                .collect();
            let features: Vec<_> = m["features"].as_array().unwrap().iter().map(|v|json!({"name":v["name"],"version":v["version"],"required":v["required"].as_u64().unwrap()!=0})).collect();
            let p = encode(
                &read::MetadataView {
                    receipt: meta.last_commit.as_ref().unwrap(),
                    features_root: StateHash::from_bytes([0; 32]),
                    metadata: &meta,
                    tags: &tags,
                    extensions: &extensions,
                    features: &features,
                },
                &mut [0; ITEMS],
                &CaptureCancellation::default(),
            )
            .unwrap();
            assert_eq!(hex(&p.digest), case["sha256"].as_str().unwrap());
            assert_eq!(usize::from(p.counts[0]), tags.len());
            assert_eq!(usize::from(p.counts[1]), extensions.len());
            assert_eq!(usize::from(p.counts[2]), features.len());
            assert_eq!(usize::from(p.counts[4]), meta.manual_marks.len());
        }
    }
    #[test]
    fn fixed_order_workspace_covers_all_8192_historical_tags_and_rejects_overflow_or_cancel() {
        // Local ordering boundary only; not a claim of whole-file admission.
        let mut tags: Vec<_> = (0..ITEMS).rev().map(|i|json!({"node":"00000000000000000000000000000001","name":format!("user:{i:04}"),"value":""})).collect();
        let mut work = [0; ITEMS];
        let order = order(
            &tags,
            Kind::Tags,
            &mut work,
            &CaptureCancellation::default(),
        )
        .unwrap();
        assert_eq!(order.len(), ITEMS);
        assert_eq!(tags[usize::from(order[0])]["name"], "user:0000");
        tags.push(tags[0].clone());
        assert!(matches!(
            super::order(
                &tags,
                Kind::Tags,
                &mut work,
                &CaptureCancellation::default()
            ),
            Err(StorageError::Limit("historical metadata items"))
        ));
        let cancel = CaptureCancellation::default();
        cancel.cancel();
        assert!(matches!(
            super::order(&tags[..ITEMS], Kind::Tags, &mut work, &cancel),
            Err(StorageError::Cancelled)
        ));
    }
}
