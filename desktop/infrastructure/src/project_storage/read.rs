use super::*;
use std::io::{Read, Seek, SeekFrom};
pub(super) struct ParsedCheckpoint {
    pub checkpoint: Checkpoint,
    pub request: [u8; 32],
    roots: BTreeMap<String, StateHash>,
}
pub(super) fn checkpoint(
    cache: &Cache,
    root: StateHash,
    limits: FileLimits,
) -> Result<ParsedCheckpoint, StorageError> {
    let row = cache
        .blocks
        .get(&root)
        .ok_or(StorageError::Invalid("checkpoint"))?;
    if row.kind != "skin-editor.checkpoint" || !row.blobs.is_empty() {
        return Err(StorageError::Invalid("checkpoint kind"));
    }
    let mut f = blocks::value(row, limits.codec)?;
    if f.take("header")? != write::header_json(&cache.header) {
        return Err(StorageError::Invalid("header binding"));
    }
    let commit = CommitId::new(ident(&f.text("commit")?)?).expect("nonzero");
    let previous = match f.take("previous")? {
        Value::Null => None,
        v => Some(CommitId::new(ident(&text(v)?)?).expect("nonzero")),
    };
    let generation = f.num("generation")?;
    let origin = save_origin(&f.text("origin")?)?;
    let head = f.id("head")?;
    let node_count =
        usize::try_from(f.num("node_count")?).map_err(|_| StorageError::Limit("nodes"))?;
    let prefix = f.hash("prefix")?.bytes();
    let next_alias = f.num("next_alias")?;
    let next_branch = f.num("next_branch")?;
    let request = f.hash("request")?.bytes();
    let mut roots = BTreeMap::new();
    for name in [
        "nodes",
        "admissions",
        "originals",
        "redo",
        "tags",
        "extensions",
        "features",
    ] {
        roots.insert(name.to_owned(), f.hash(name)?);
    }
    f.finish()?;
    if roots
        .values()
        .copied()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>()
        != row.refs
    {
        return Err(StorageError::Invalid("checkpoint refs"));
    }
    let mut redo = Vec::new();
    let mut parents = BTreeSet::new();
    for v in blocks::read_paged(cache, roots["redo"], "redo", limits)? {
        let mut f = Fields::new(v)?;
        let p = f.id("parent")?;
        let c = f.id("child")?;
        f.finish()?;
        if !parents.insert(p) || cache.nodes.get(&c).and_then(|n| n.info.parent) != Some(p) {
            return Err(StorageError::Invalid("redo"));
        }
        redo.push((p, c));
    }
    Ok(ParsedCheckpoint {
        checkpoint: Checkpoint {
            receipt: ArchiveReceipt {
                commit,
                generation,
                head,
                origin,
            },
            previous,
            navigation: root,
            prefix,
            node_count,
            next_alias,
            next_branch,
            redo,
        },
        request,
        roots,
    })
}
struct BlobReader<'a> {
    conn: &'a Connection,
    reference: BlobRef,
    position: u64,
    chunk: Vec<u8>,
    part: Option<u64>,
    cancel: CaptureCancellation,
}
impl Read for BlobReader<'_> {
    fn read(&mut self, out: &mut [u8]) -> std::io::Result<usize> {
        if self.cancel.is_cancelled() {
            return Err(std::io::Error::other("cancelled"));
        }
        if self.position >= self.reference.byte_len() || out.is_empty() {
            return Ok(0);
        }
        let part = self.position / 65536;
        if self.part != Some(part) {
            self.chunk = self
                .conn
                .query_row(
                    "SELECT bytes FROM blob_chunks WHERE sha=?1 AND part=?2",
                    params![self.reference.sha256(), part as i64],
                    |r| r.get(0),
                )
                .map_err(std::io::Error::other)?;
            let expected = (self.reference.byte_len() - part * 65536).min(65536) as usize;
            if self.chunk.len() != expected {
                return Err(std::io::Error::other("chunk length"));
            }
            self.part = Some(part);
        }
        let start = (self.position % 65536) as usize;
        let n = out.len().min(self.chunk.len() - start);
        out[..n].copy_from_slice(&self.chunk[start..start + n]);
        self.position += n as u64;
        Ok(n)
    }
}
impl Seek for BlobReader<'_> {
    fn seek(&mut self, p: SeekFrom) -> std::io::Result<u64> {
        let next = match p {
            SeekFrom::Start(n) => i128::from(n),
            SeekFrom::Current(n) => i128::from(self.position) + i128::from(n),
            SeekFrom::End(n) => i128::from(self.reference.byte_len()) + i128::from(n),
        };
        if next < 0 || next > i128::from(self.reference.byte_len()) {
            return Err(std::io::Error::other("seek"));
        }
        self.position = next as u64;
        Ok(self.position)
    }
}
fn metadata_probe(project: &SkinProject, store: &BlobStore) -> Result<(), StorageError> {
    use crate::AudioDecoder;
    use image::ImageDecoder;
    for a in project.assets().iter() {
        let raw = store
            .get(a.blob())
            .ok_or(StorageError::MissingOriginal(a.blob()))?;
        match a.kind() {
            AssetKind::Audio(meta) => {
                let found = crate::WavDecoder
                    .probe(raw.bytes())
                    .map_err(|_| StorageError::Invalid("typed WAV"))?;
                if found != *meta {
                    return Err(StorageError::Invalid("audio metadata"));
                }
                raw.attest_audio(found)
                    .map_err(|_| StorageError::Invalid("audio attestation"))?;
            }
            AssetKind::RasterImage(size) => {
                let png = image::codecs::png::PngDecoder::new(std::io::Cursor::new(raw.bytes()))
                    .map_err(|_| StorageError::Invalid("typed PNG"))?;
                if png.dimensions() != (size.width(), size.height()) {
                    return Err(StorageError::Invalid("image metadata"));
                }
            }
            AssetKind::Opaque => (),
        }
    }
    Ok(())
}
fn table_count(c: &Connection, table: &str, max: usize) -> Result<usize, StorageError> {
    let n = c.query_row(&format!("SELECT count(*) FROM {table}"), [], |r| {
        r.get::<_, i64>(0)
    })?;
    let n = usize::try_from(n).map_err(|_| StorageError::Invalid("count"))?;
    if n > max {
        return Err(StorageError::Limit("table rows"));
    }
    Ok(n)
}
fn metadata_refs(
    c: &Connection,
    cache: &Cache,
    cancel: &CaptureCancellation,
) -> Result<(), StorageError> {
    let mut q = c.prepare("SELECT owner,target FROM block_refs ORDER BY owner,target")?;
    let mut rows = q.query([])?;
    for (owner, row) in &cache.blocks {
        for target in &row.refs {
            check(cancel)?;
            let r = rows
                .next()?
                .ok_or(StorageError::Invalid("missing block index"))?;
            if fixed::<32>(r.get(0)?)? != owner.bytes() || fixed::<32>(r.get(1)?)? != target.bytes()
            {
                return Err(StorageError::Invalid("block index"));
            }
        }
    }
    if rows.next()?.is_some() {
        return Err(StorageError::Invalid("extra block index"));
    }
    let mut q = c.prepare("SELECT owner,target FROM blob_refs ORDER BY owner,target")?;
    let mut rows = q.query([])?;
    for (owner, row) in &cache.blocks {
        for target in &row.blobs {
            check(cancel)?;
            let r = rows
                .next()?
                .ok_or(StorageError::Invalid("missing blob index"))?;
            if fixed::<32>(r.get(0)?)? != owner.bytes() || fixed::<32>(r.get(1)?)? != *target {
                return Err(StorageError::Invalid("blob index"));
            }
        }
    }
    if rows.next()?.is_some() {
        return Err(StorageError::Invalid("extra blob index"));
    }
    Ok(())
}
fn load_blocks(
    c: &Connection,
    cache: &mut Cache,
    limits: FileLimits,
    cancel: &CaptureCancellation,
) -> Result<(), StorageError> {
    table_count(c, "blocks", limits.blocks)?;
    let mut q=c.prepare("SELECT sha,sequence,length(bytes),bytes,kind,schema_version,project_id FROM blocks ORDER BY sequence")?;
    let mut rows = q.query([])?;
    while let Some(r) = rows.next()? {
        check(cancel)?;
        let len = usize::try_from(r.get::<_, i64>(2)?)
            .map_err(|_| StorageError::Invalid("block length"))?;
        if len > limits.codec.block_bytes
            || len > limits.metadata_bytes.saturating_sub(cache.metadata_bytes)
        {
            return Err(StorageError::Limit("metadata bytes"));
        }
        let hash = StateHash::from_bytes(fixed(r.get(0)?)?);
        let sequence = u64blob(r.get(1)?)?;
        if sequence != cache.blocks.len() as u64 + 1 {
            return Err(StorageError::Invalid("block sequence"));
        }
        let row = blocks::parse(
            StateBlock {
                hash,
                bytes: r.get(3)?,
            },
            limits.codec,
        )?;
        if row.kind != r.get::<_, String>(4)? || r.get::<_, i64>(5)? != 1 {
            return Err(StorageError::Invalid("block derived fields"));
        }
        let expected_project = if row.kind == "skin-editor.project" {
            Some(blocks::value(&row, limits.codec)?.text("id")?)
        } else {
            None
        };
        if r.get::<_, Option<String>>(6)? != expected_project {
            return Err(StorageError::Invalid("block project"));
        }
        if row.refs.iter().any(|h| !cache.blocks.contains_key(h))
            || row
                .blobs
                .iter()
                .any(|h| !cache.originals.iter().any(|b| b.sha256() == *h))
        {
            return Err(StorageError::Invalid("block dependencies"));
        }
        cache.metadata_bytes += len;
        if cache.blocks.insert(hash, row).is_some() {
            return Err(StorageError::Invalid("duplicate block"));
        }
    }
    metadata_refs(c, cache, cancel)
}
fn bounded_feature(v: &Value) -> Result<(), StorageError> {
    let mut f = Fields::new(v.clone())?;
    let name = f.text("name")?;
    let version = f.text("version")?;
    let needed = f
        .take("required")?
        .as_bool()
        .ok_or(StorageError::Invalid("feature required"))?;
    f.finish()?;
    if name.len() > 256 || version.len() > 64 || name.is_empty() {
        return Err(StorageError::Limit("feature"));
    }
    if needed && !write::features().contains(v) {
        return Err(StorageError::Unsupported("required feature"));
    }
    Ok(())
}
fn node_info(
    c: &Connection,
    cache: &mut Cache,
    limits: FileLimits,
    cancel: &CaptureCancellation,
) -> Result<Vec<(CapturedNodeInfo, StateHash)>, StorageError> {
    table_count(c, "nodes", limits.restore.nodes)?;
    let mut q=c.prepare("SELECT n.id,n.parent,n.state,n.depth,length(n.action),n.action,n.checksum,n.record_version,a.alias,a.branch,a.ordinal,a.checksum FROM nodes n JOIN addresses a ON n.id=a.node_id ORDER BY a.alias")?;
    let mut rows = q.query([])?;
    let mut infos = Vec::new();
    while let Some(r) = rows.next()? {
        check(cancel)?;
        let len = usize::try_from(r.get::<_, i64>(4)?)
            .map_err(|_| StorageError::Invalid("action length"))?;
        if len > 65536 || len + 128 > limits.metadata_bytes.saturating_sub(cache.metadata_bytes) {
            return Err(StorageError::Limit("actions"));
        }
        let id = nodeblob(r.get(0)?)?;
        let parent = r.get::<_, Option<Vec<u8>>>(1)?.map(nodeblob).transpose()?;
        let state = StateHash::from_bytes(fixed(r.get(2)?)?);
        let depth =
            u64::try_from(r.get::<_, i64>(3)?).map_err(|_| StorageError::Invalid("depth"))?;
        let raw: Vec<u8> = r.get(5)?;
        let checksum: [u8; 32] = fixed(r.get(6)?)?;
        if r.get::<_, i64>(7)? != 2 || node_checksum(id, parent, state, &raw) != checksum {
            return Err(StorageError::Invalid("node record"));
        }
        let coordinate = BranchCoordinate::new(u64blob(r.get(9)?)?, u64blob(r.get(10)?)?)
            .ok_or(StorageError::Invalid("coordinate"))?;
        let info = CapturedNodeInfo {
            id,
            parent,
            alias: u64blob(r.get(8)?)?,
            coordinate,
            depth,
            operation: operation(&raw, limits.codec)?,
            changed_objects: Vec::new(),
            origin: Some(Arc::new(OriginalNodeRecord {
                record_version: 2,
                state: state.bytes(),
                action: raw,
                checksum,
            })),
        };
        if address_checksum(&info) != fixed::<32>(r.get(11)?)? {
            return Err(StorageError::Invalid("address binding"));
        }
        cache.metadata_bytes += len + 128;
        infos.push((info, state));
    }
    if infos.len() != table_count(c, "nodes", limits.restore.nodes)?
        || infos.len() != table_count(c, "addresses", limits.restore.nodes)?
    {
        return Err(StorageError::Invalid("node/address coverage"));
    }
    Ok(infos)
}
pub(super) fn full(
    c: &mut Connection,
    store: &BlobStore,
    limits: FileLimits,
    selection: RestoreSelection,
    cancel: &CaptureCancellation,
) -> Result<(Cache, RestoreCandidate), StorageError> {
    full_probe(c, store, limits, selection, cancel, Probe::default())
}
pub(super) fn full_probe(
    c: &mut Connection,
    store: &BlobStore,
    limits: FileLimits,
    selection: RestoreSelection,
    cancel: &CaptureCancellation,
    probe: Probe,
) -> Result<(Cache, RestoreCandidate), StorageError> {
    let result = full_inner(c, store, limits, selection, cancel, probe);
    if cancel.is_cancelled() && result.is_err() {
        Err(StorageError::Cancelled)
    } else {
        result
    }
}
fn full_inner(
    c: &mut Connection,
    store: &BlobStore,
    limits: FileLimits,
    selection: RestoreSelection,
    cancel: &CaptureCancellation,
    probe: Probe,
) -> Result<(Cache, RestoreCandidate), StorageError> {
    check(cancel)?;
    configure(c, limits, cancel)?;
    let mut tx = Transaction::begin(c, false)?;
    let result = read_snapshot(tx.conn, store, limits, selection, cancel, probe)?;
    check(cancel)?;
    tx.commit()?;
    Ok(result)
}
fn read_snapshot(
    c: &Connection,
    store: &BlobStore,
    limits: FileLimits,
    selection: RestoreSelection,
    cancel: &CaptureCancellation,
    probe: Probe,
) -> Result<(Cache, RestoreCandidate), StorageError> {
    if c.query_row("PRAGMA application_id", [], |r| r.get::<_, u32>(0))? != APP_ID
        || c.query_row("PRAGMA user_version", [], |r| r.get::<_, u32>(0))? != CONTAINER_VERSION
    {
        return Err(StorageError::Unsupported("container"));
    }
    schema::verify(c)?;
    if c.query_row("PRAGMA journal_mode", [], |r| r.get::<_, String>(0))? != "delete" {
        return Err(StorageError::Unsupported("journal mode"));
    }
    if table_count(c, "header", 1)? != 1 || table_count(c, "control", 1)? != 1 {
        return Err(StorageError::Invalid("header/control"));
    }
    let header=c.query_row("SELECT family,profile,container_version,project_schema,history_schema,project_id,history_id,lineage_id,root_id FROM header",[],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,i64>(2)?,r.get::<_,i64>(3)?,r.get::<_,i64>(4)?,r.get::<_,Vec<u8>>(5)?,r.get::<_,Vec<u8>>(6)?,r.get::<_,Vec<u8>>(7)?,r.get::<_,Vec<u8>>(8)?)))?;
    if header.0 != "skin-editor-project"
        || header.1 != FILE_PROFILE
        || header.2 != 2
        || header.3 != 1
        || header.4 != 2
    {
        return Err(StorageError::Unsupported("profile/schema"));
    }
    let control=c.query_row("SELECT generation,save_id,manual_id,protected_id,node_count,block_count,blob_bytes,metadata_bytes,address_epoch FROM control",[],|r|Ok((r.get::<_,Vec<u8>>(0)?,r.get::<_,Vec<u8>>(1)?,r.get::<_,Option<Vec<u8>>>(2)?,r.get::<_,Vec<u8>>(3)?,r.get::<_,i64>(4)?,r.get::<_,i64>(5)?,r.get::<_,i64>(6)?,r.get::<_,i64>(7)?,r.get::<_,Vec<u8>>(8)?)))?;
    let identity = PersistentHistoryIdentity {
        history: PersistentHistoryId::new(u128::from_be_bytes(fixed(header.6)?))
            .ok_or(StorageError::Invalid("history id"))?,
        lineage: LineageId::new(u128::from_be_bytes(fixed(header.7)?))
            .ok_or(StorageError::Invalid("lineage"))?,
        address_epoch: AddressEpoch::new(u128::from_be_bytes(fixed(control.8)?))
            .ok_or(StorageError::Invalid("address epoch"))?,
    };
    let header = Header {
        project: ProjectId::new(u128::from_be_bytes(fixed(header.5)?))
            .ok_or(StorageError::Invalid("project"))?,
        identity,
        root: nodeblob(header.8)?,
    };
    if table_count(c, "state_views", limits.blocks)? != 0 {
        return Err(StorageError::Unsupported("state migrations"));
    }
    let mut cache = Cache {
        header,
        blocks: BTreeMap::new(),
        nodes: BTreeMap::new(),
        order: Vec::new(),
        admissions: BTreeMap::new(),
        originals: BTreeSet::new(),
        checkpoints: Vec::new(),
        metadata: Arc::default(),
        features: Vec::new(),
        metadata_bytes: 256,
        prefix: initial_prefix(),
    };
    table_count(c, "features", 256)?;
    let mut q = c.prepare("SELECT name,version,required FROM features ORDER BY name,version")?;
    let mut rows = q.query([])?;
    while let Some(r) = rows.next()? {
        let required = r.get::<_, i64>(2)?;
        if !matches!(required, 0 | 1) {
            return Err(StorageError::Invalid("feature"));
        }
        let v = json!({"name":r.get::<_,String>(0)?,"version":r.get::<_,String>(1)?,"required":required==1});
        bounded_feature(&v)?;
        cache.features.push(v);
    }
    for required in write::features() {
        if !cache.features.contains(&required) {
            return Err(StorageError::Unsupported("missing required feature"));
        }
    }
    table_count(c, "blobs", limits.originals)?;
    let mut q = c.prepare("SELECT sha,byte_len FROM blobs ORDER BY sha")?;
    let mut rows = q.query([])?;
    let mut total = 0u64;
    let mut raw_pins = Vec::<RawBlob>::new();
    while let Some(r) = rows.next()? {
        check(cancel)?;
        let reference = BlobRef::new(
            fixed(r.get(0)?)?,
            u64::try_from(r.get::<_, i64>(1)?).map_err(|_| StorageError::Invalid("blob length"))?,
        );
        total = total
            .checked_add(reference.byte_len())
            .ok_or(StorageError::Limit("original bytes"))?;
        if total > limits.original_bytes {
            return Err(StorageError::Limit("original bytes"));
        }
        let count = c.query_row(
            "SELECT count(*) FROM blob_chunks WHERE sha=?1",
            params![reference.sha256()],
            |r| r.get::<_, i64>(0),
        )?;
        if count as u64 != reference.byte_len().div_ceil(65536) {
            return Err(StorageError::Invalid("chunk count"));
        }
        let raw = store
            .stage_reader(
                BlobReader {
                    conn: c,
                    reference,
                    position: 0,
                    chunk: Vec::new(),
                    part: None,
                    cancel: cancel.clone(),
                },
                reference.byte_len(),
            )
            .map_err(|e| match e {
                crate::BlobError::ResourceLimit => StorageError::Limit("original admission"),
                _ => StorageError::Invalid("original read"),
            })?;
        if raw.reference() != reference {
            return Err(StorageError::Invalid("original digest"));
        }
        raw_pins.push(raw);
        cache.originals.insert(reference);
        probe.step(Stage::ReadOriginal, cancel)?;
    }
    load_blocks(c, &mut cache, limits, cancel)?;
    let infos = node_info(c, &mut cache, limits, cancel)?;
    let mut reachable = BTreeSet::new();
    for (info, root) in &infos {
        cache.order.push(info.id);
        cache.nodes.insert(
            info.id,
            CachedNode {
                info: info.clone(),
                canonical: *root,
            },
        );
    }
    table_count(c, "object_ids", limits.restore.object_ids)?;
    let mut q = c.prepare("SELECT kind,id,first_node FROM object_ids ORDER BY kind,id")?;
    let mut rows = q.query([])?;
    while let Some(r) = rows.next()? {
        let kind = r.get::<_, String>(0)?;
        let id = u128::from_be_bytes(fixed(r.get(1)?)?);
        let first = nodeblob(r.get(2)?)?;
        if cache
            .admissions
            .insert(object_id(&kind, id)?, first)
            .is_some()
        {
            return Err(StorageError::Invalid("admissions"));
        }
    }
    let (tags, extensions) = metadata_rows(c, limits, cancel)?;
    let mut meta = ArchiveMetadata {
        tags,
        extensions,
        ..Default::default()
    };
    table_count(c, "commits", 65536)?;
    let mut q=c.prepare("SELECT id,previous,generation,origin,head,navigation,request_hash,address_epoch FROM commits ORDER BY generation")?;
    let mut rows = q.query([])?;
    let mut seen_nodes = BTreeSet::new();
    let mut seen_admissions = BTreeMap::new();
    let mut seen_originals = BTreeSet::new();
    let mut prefix = initial_prefix();
    let mut last_tags = Vec::new();
    let mut last_ext = Vec::new();
    let mut last_features = Vec::new();
    let mut max_alias = 0;
    let mut max_branch = 0;
    while let Some(r) = rows.next()? {
        check(cancel)?;
        let root = StateHash::from_bytes(fixed(r.get(5)?)?);
        let p = checkpoint(&cache, root, limits)?;
        let cp = &p.checkpoint;
        if commitblob(r.get(0)?)? != cp.receipt.commit
            || r.get::<_, Option<Vec<u8>>>(1)?
                .map(commitblob)
                .transpose()?
                != cp.previous
            || u64blob(r.get(2)?)? != cp.receipt.generation
            || save_origin(&r.get::<_, String>(3)?)? != cp.receipt.origin
            || nodeblob(r.get(4)?)? != cp.receipt.head
            || fixed::<32>(r.get(6)?)? != p.request
            || u128::from_be_bytes(fixed(r.get(7)?)?) != cache.header.identity.address_epoch.get()
        {
            return Err(StorageError::Invalid("commit binding"));
        }
        if cp.previous != cache.checkpoints.last().map(|p| p.receipt.commit)
            || cp.receipt.generation != cache.checkpoints.len() as u64 + 1
            || !cache.nodes.contains_key(&cp.receipt.head)
        {
            return Err(StorageError::Invalid("commit chain"));
        }
        let mut added = Vec::new();
        for v in blocks::read_paged(&cache, p.roots["nodes"], "nodes", limits)? {
            let mut f = Fields::new(v)?;
            let id = f.id("id")?;
            let checksum = f.hash("checksum")?.bytes();
            let address = f.hash("address")?.bytes();
            let state = f.hash("state")?.bytes();
            f.finish()?;
            let n = cache
                .nodes
                .get(&id)
                .ok_or(StorageError::Invalid("indexed node"))?;
            if cache.order.get(seen_nodes.len()) != Some(&id)
                || !seen_nodes.insert(id)
                || n.info.origin.as_ref().expect("record").checksum != checksum
                || address_checksum(&n.info) != address
                || n.info.origin.as_ref().expect("record").state != state
            {
                return Err(StorageError::Invalid("node index attestation"));
            }
            max_alias = max_alias.max(n.info.alias);
            max_branch = max_branch.max(n.info.coordinate.branch());
            added.push(id);
        }
        let added_set: BTreeSet<_> = added.iter().copied().collect();
        let mut by_first: BTreeMap<_, Vec<_>> = BTreeMap::new();
        for v in blocks::read_paged(&cache, p.roots["admissions"], "admissions", limits)? {
            let mut f = Fields::new(v)?;
            let kind = f.text("kind")?;
            let id = ident(&f.text("id")?)?;
            let first = f.id("first")?;
            f.finish()?;
            let object = object_id(&kind, id)?;
            if !added_set.contains(&first)
                || cache.admissions.get(&object) != Some(&first)
                || seen_admissions.insert(object, first).is_some()
            {
                return Err(StorageError::Invalid("admission attestation"));
            }
            by_first.entry(first).or_default().push((object, first));
        }
        for facts in by_first.values_mut() {
            facts.sort_by_key(|(o, _)| object(*o));
        }
        for id in added {
            prefix = extend_prefix(
                prefix,
                &cache.nodes[&id].info,
                by_first.get(&id).map_or(&[], Vec::as_slice),
            );
        }
        for v in blocks::read_paged(&cache, p.roots["originals"], "originals", limits)? {
            let mut f = Fields::new(v)?;
            let sha = f.hash("sha")?.bytes();
            let len = f.num("bytes")?;
            f.finish()?;
            let b = BlobRef::new(sha, len);
            if !cache.originals.contains(&b) || !seen_originals.insert(b) {
                return Err(StorageError::Invalid("original index"));
            }
        }
        let valid_counter = |next, last| {
            if last == u64::MAX {
                next == 0
            } else {
                next > last
            }
        };
        if !seen_nodes.contains(&cp.receipt.head)
            || cp
                .redo
                .iter()
                .any(|(p, c)| !seen_nodes.contains(p) || !seen_nodes.contains(c))
            || !valid_counter(cp.next_alias, max_alias)
            || !valid_counter(cp.next_branch, max_branch)
            || prefix != cp.prefix
            || cp.node_count != seen_nodes.len()
            || write::request_hash(
                &cache.header,
                write::RequestFrame {
                    commit: cp.receipt.commit,
                    previous: cp.previous,
                    origin: cp.receipt.origin,
                    head: cp.receipt.head,
                    prefix: cp.prefix,
                    node_count: cp.node_count,
                    next_alias: cp.next_alias,
                    next_branch: cp.next_branch,
                    redo: &cp.redo,
                },
            ) != p.request
        {
            return Err(StorageError::Invalid("snapshot attestation"));
        }
        blocks::mark(&cache, root, &mut reachable)?;
        last_tags = blocks::read_paged(&cache, p.roots["tags"], "tags", limits)?;
        last_ext = blocks::read_paged(&cache, p.roots["extensions"], "extensions", limits)?;
        last_features = blocks::read_paged(&cache, p.roots["features"], "features", limits)?;
        validate_past_metadata(
            &seen_nodes,
            &meta,
            &cache.features,
            &last_tags,
            &last_ext,
            &last_features,
        )?;
        if cp.receipt.origin == SaveOrigin::Manual {
            meta.manual = Some(cp.receipt.clone());
            meta.manual_marks.push((cp.receipt.commit, cp.receipt.head));
        }
        meta.last_commit = Some(cp.receipt.clone());
        meta.protected = Some(cp.receipt.clone());
        cache.checkpoints.push(p.checkpoint);
    }
    if seen_nodes.len() != cache.nodes.len()
        || seen_admissions != cache.admissions
        || seen_originals != cache.originals
        || reachable.len() != cache.blocks.len()
    {
        return Err(StorageError::Invalid("complete closure"));
    }
    verify_metadata_indexes(&cache, &meta, last_tags, last_ext, last_features)?;
    let mut actual_marks = BTreeMap::new();
    let mut q = c.prepare("SELECT commit_id,node_id FROM manual_marks")?;
    let mut rows = q.query([])?;
    while let Some(r) = rows.next()? {
        actual_marks.insert(commitblob(r.get(0)?)?, nodeblob(r.get(1)?)?);
    }
    if actual_marks != meta.manual_marks.iter().copied().collect() {
        return Err(StorageError::Invalid("manual marks"));
    }
    let last = cache
        .checkpoints
        .last()
        .ok_or(StorageError::Invalid("missing commit"))?;
    if u64blob(control.0)? != last.receipt.generation
        || commitblob(control.1)? != last.receipt.commit
        || control.2.map(commitblob).transpose()? != meta.manual.as_ref().map(|r| r.commit)
        || commitblob(control.3)? != last.receipt.commit
        || usize::try_from(control.4).ok() != Some(cache.nodes.len())
        || usize::try_from(control.5).ok() != Some(cache.blocks.len())
        || u64::try_from(control.6).ok() != Some(total)
    {
        return Err(StorageError::Invalid("control"));
    }
    cache.metadata = Arc::new(meta);
    cache.metadata_bytes = write::cost(&cache)?;
    cache.prefix = prefix;
    if usize::try_from(control.7).ok() != Some(cache.metadata_bytes)
        || cache.metadata_bytes > limits.metadata_bytes
    {
        return Err(StorageError::Invalid("metadata counters"));
    }
    let selected = match selection {
        RestoreSelection::Latest => last,
        RestoreSelection::Manual => cache
            .metadata
            .manual
            .as_ref()
            .and_then(|r| {
                cache
                    .checkpoints
                    .iter()
                    .find(|p| p.receipt.commit == r.commit)
            })
            .unwrap_or(last),
    };
    // A checkpoint's old unique successor stays the ordinary redo route if
    // branches were added later. Preserve ambiguity when it already existed.
    let saved: BTreeSet<_> = cache
        .order
        .iter()
        .take(selected.node_count)
        .copied()
        .collect();
    let mut children: BTreeMap<_, Vec<_>> = BTreeMap::new();
    for n in cache.nodes.values() {
        if let Some(p) = n.info.parent {
            children.entry(p).or_default().push(n.info.id);
        }
    }
    let mut restored_redo: BTreeMap<_, _> = selected.redo.iter().copied().collect();
    for (parent, all) in children {
        if all.len() > 1 && !restored_redo.contains_key(&parent) {
            let prior: Vec<_> = all.into_iter().filter(|n| saved.contains(n)).collect();
            if prior.len() == 1 {
                restored_redo.insert(parent, prior[0]);
            }
        }
    }
    let header = RestoreHeader {
        node_count: cache.nodes.len(),
        project: cache.header.project,
        identity: cache.header.identity.clone(),
        root: cache.header.root,
        head: selected.receipt.head,
        next_alias: last.next_alias,
        next_branch: last.next_branch,
        admissions: cache
            .admissions
            .iter()
            .map(|(o, p)| CapturedAdmission {
                object: *o,
                first_node: *p,
            })
            .collect(),
        redo_preferences: restored_redo.into_iter().collect(),
        originals: cache.originals.iter().copied().collect(),
        metadata: cache.metadata.clone(),
    };
    let mut builder = RestoreBuilder::new(header, limits.restore, cancel.clone())?;
    let mut decoded = 0usize;
    for (info, root) in infos {
        check(cancel)?;
        let bundle = blocks::closure(&cache, root, &mut reachable, limits)?;
        decoded = decoded
            .checked_add(bundle.blocks.iter().map(|b| b.bytes.len()).sum::<usize>())
            .ok_or(StorageError::Limit("decoded bytes"))?;
        if decoded > limits.decoded_bytes {
            return Err(StorageError::Limit("decoded bytes"));
        }
        let project = decode_project(&bundle, limits.codec)?;
        metadata_probe(&project, store)?;
        let canonical = encode_project(&project, limits.codec)?.root;
        cache.nodes.get_mut(&info.id).expect("indexed").canonical = canonical;
        builder.push(RestoreNode {
            id: info.id,
            parent: info.parent,
            alias: info.alias,
            coordinate: info.coordinate,
            depth: info.depth,
            operation: info.operation,
            origin: info.origin.expect("record"),
            project,
        })?;
        probe.step(Stage::ReadNodes, cancel)?;
    }
    let candidate = builder.finish(store)?;
    drop(raw_pins);
    check(cancel)?;
    Ok((cache, candidate))
}
fn metadata_rows(
    c: &Connection,
    limits: FileLimits,
    cancel: &CaptureCancellation,
) -> Result<(Vec<ArchiveTag>, Vec<ArchiveExtension>), StorageError> {
    table_count(c, "tags", limits.restore.nodes.saturating_mul(64))?;
    let mut tags = Vec::new();
    let mut bytes = 0usize;
    let mut q = c.prepare("SELECT node_id,name,value FROM tags ORDER BY node_id,name")?;
    let mut rows = q.query([])?;
    while let Some(r) = rows.next()? {
        check(cancel)?;
        let t = ArchiveTag {
            node: nodeblob(r.get(0)?)?,
            name: r.get(1)?,
            value: r.get(2)?,
        };
        if !t.name.starts_with("user:") || t.name.len() > 256 || t.value.len() > 4096 {
            return Err(StorageError::Invalid("tag"));
        }
        bytes = bytes
            .checked_add(64 + t.name.len() + t.value.len())
            .ok_or(StorageError::Limit("tags"))?;
        if bytes > limits.metadata_bytes {
            return Err(StorageError::Limit("tags"));
        }
        tags.push(t);
    }
    table_count(c, "extensions", 1024)?;
    let mut ext = Vec::new();
    let mut q = c.prepare("SELECT name,length(bytes),bytes FROM extensions ORDER BY name")?;
    let mut rows = q.query([])?;
    while let Some(r) = rows.next()? {
        check(cancel)?;
        let name = r.get::<_, String>(0)?;
        let len = usize::try_from(r.get::<_, i64>(1)?)
            .map_err(|_| StorageError::Invalid("extension length"))?;
        bytes = bytes
            .checked_add(64 + name.len())
            .and_then(|n| n.checked_add(len))
            .ok_or(StorageError::Limit("extensions"))?;
        if name.is_empty() || name.len() > 256 || bytes > limits.metadata_bytes {
            return Err(StorageError::Limit("extensions"));
        }
        ext.push(ArchiveExtension {
            name,
            bytes: r.get(2)?,
        });
    }
    Ok((tags, ext))
}
fn verify_metadata_indexes(
    cache: &Cache,
    meta: &ArchiveMetadata,
    tags: Vec<Value>,
    ext: Vec<Value>,
    features: Vec<Value>,
) -> Result<(), StorageError> {
    let expected_tags: Vec<_> = meta
        .tags
        .iter()
        .map(|t| json!({"node":ptext(t.node),"name":t.name,"value":t.value}))
        .collect();
    let expected_ext: Vec<_> = meta
        .extensions
        .iter()
        .map(|e| json!({"name":e.name,"sha":hex(&sha(&e.bytes)),"bytes":e.bytes.len().to_string()}))
        .collect();
    if tags != expected_tags || ext != expected_ext || features != cache.features {
        return Err(StorageError::Invalid("metadata indexes"));
    }
    Ok(())
}

fn validate_past_metadata(
    nodes: &BTreeSet<PersistentNodeId>,
    meta: &ArchiveMetadata,
    known_features: &[Value],
    tags: &[Value],
    extensions: &[Value],
    features: &[Value],
) -> Result<(), StorageError> {
    let mut keys = BTreeSet::new();
    for v in tags {
        let mut f = Fields::new(v.clone())?;
        let node = f.id("node")?;
        let name = f.text("name")?;
        let value = f.text("value")?;
        f.finish()?;
        if !nodes.contains(&node)
            || !name.starts_with("user:")
            || name.len() > 256
            || value.len() > 4096
            || !keys.insert((node, name))
        {
            return Err(StorageError::Invalid("historical tags"));
        }
    }
    let by_name: BTreeMap<_, _> = meta
        .extensions
        .iter()
        .map(|e| (e.name.as_str(), e))
        .collect();
    let mut names = BTreeSet::new();
    for v in extensions {
        let mut f = Fields::new(v.clone())?;
        let name = f.text("name")?;
        let digest = f.hash("sha")?.bytes();
        let len = f.num("bytes")?;
        f.finish()?;
        let e = by_name
            .get(name.as_str())
            .ok_or(StorageError::Invalid("historical extension"))?;
        if !names.insert(name) || sha(&e.bytes) != digest || e.bytes.len() as u64 != len {
            return Err(StorageError::Invalid("historical extension binding"));
        }
    }
    let mut keys = BTreeSet::new();
    for v in features {
        bounded_feature(v)?;
        if !known_features.contains(v) || !keys.insert((v["name"].as_str(), v["version"].as_str()))
        {
            return Err(StorageError::Invalid("historical features"));
        }
    }
    if write::features().iter().any(|v| !features.contains(v)) {
        return Err(StorageError::Unsupported("historical required feature"));
    }
    Ok(())
}

#[cfg(test)]
mod metadata_tests {
    use super::*;
    #[test]
    fn historical_metadata_is_validated_even_when_latest_records_are_valid() {
        let node = PersistentNodeId::new(100).unwrap();
        let nodes = BTreeSet::from([node]);
        let meta = ArchiveMetadata::default();
        let features = write::features();
        let tags = vec![json!({"node":ptext(node),"name":"user:note","value":"old"})];
        assert!(validate_past_metadata(&nodes, &meta, &features, &tags, &[], &features).is_ok());
        assert!(
            validate_past_metadata(&BTreeSet::new(), &meta, &features, &tags, &[], &features)
                .is_err()
        );
        assert!(
            validate_past_metadata(
                &nodes,
                &meta,
                &features,
                &[tags[0].clone(), tags[0].clone()],
                &[],
                &features
            )
            .is_err()
        );
        assert!(validate_past_metadata(&nodes, &meta, &features, &[], &[], &[]).is_err());
        assert!(
            validate_past_metadata(
                &nodes,
                &meta,
                &features,
                &[],
                &[json!({"name":"missing","sha":"00".repeat(32),"bytes":"1"})],
                &features
            )
            .is_err()
        );
    }
}
