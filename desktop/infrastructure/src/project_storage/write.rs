use super::*;

pub(super) fn features() -> Vec<Value> {
    vec![
        json!({"name":"history.delta-index","version":"1","required":true}),
        json!({"name":"history.summary","version":"1","required":true}),
        json!({"name":"state.editor","version":"1","required":true}),
        json!({"name":"storage.core","version":"2","required":true}),
    ]
}
pub(super) fn cost(cache: &Cache) -> Result<usize, StorageError> {
    let mut n = 256usize;
    let mut add = |v: usize| {
        n = n.checked_add(v).ok_or(StorageError::Limit("metadata"))?;
        Ok::<_, StorageError>(())
    };
    for b in cache.blocks.values() {
        add(b.block.bytes.len())?;
    }
    for node in cache.nodes.values() {
        add(128 + node.info.origin.as_ref().expect("record").action.len())?;
    }
    add(cache
        .admissions
        .len()
        .checked_mul(64)
        .ok_or(StorageError::Limit("metadata"))?)?;
    add(cache.originals.len() * 64)?;
    add(cache.checkpoints.len() * 128)?;
    for t in &cache.metadata.tags {
        add(64 + t.name.len() + t.value.len())?;
    }
    for e in &cache.metadata.extensions {
        add(64 + e.name.len() + e.bytes.len())?;
    }
    for f in &cache.features {
        add(32
            + f["name"]
                .as_str()
                .ok_or(StorageError::Invalid("feature"))?
                .len()
            + f["version"]
                .as_str()
                .ok_or(StorageError::Invalid("feature"))?
                .len())?;
    }
    Ok(n)
}
pub(super) fn header_json(h: &Header) -> Value {
    json!({"project":format!("{:032x}",h.project.get()),"history":format!("{:032x}",h.identity.history.get()),"lineage":format!("{:032x}",h.identity.lineage.get()),"address_epoch":format!("{:032x}",h.identity.address_epoch.get()),"root":ptext(h.root)})
}
fn readonly_metadata(
    c: &Connection,
    cache: &mut Cache,
    limits: FileLimits,
) -> Result<(StateHash, StateHash, StateHash), StorageError> {
    let tags = cache
        .metadata
        .tags
        .iter()
        .map(|t| json!({"node":ptext(t.node),"name":t.name,"value":t.value}))
        .collect();
    let ext = cache
        .metadata
        .extensions
        .iter()
        .map(|e| json!({"name":e.name,"sha":hex(&sha(&e.bytes)),"bytes":e.bytes.len().to_string()}))
        .collect();
    let tags = blocks::paged(c, cache, "tags", tags, limits)?;
    let extensions = blocks::paged(c, cache, "extensions", ext, limits)?;
    let features = blocks::paged(c, cache, "features", cache.features.clone(), limits)?;
    Ok((tags, extensions, features))
}
pub(super) struct RequestFrame<'a> {
    pub commit: CommitId,
    pub previous: Option<CommitId>,
    pub origin: SaveOrigin,
    pub head: PersistentNodeId,
    pub prefix: [u8; 32],
    pub node_count: usize,
    pub next_alias: u64,
    pub next_branch: u64,
    pub redo: &'a [(PersistentNodeId, PersistentNodeId)],
}
pub(super) fn request_hash(header: &Header, frame: RequestFrame<'_>) -> [u8; 32] {
    let RequestFrame {
        commit,
        previous,
        origin,
        head,
        prefix,
        node_count,
        next_alias,
        next_branch,
        redo,
    } = frame;
    let mut h = Sha256::new();
    h.update(b"skin-editor.archive-request.v1\0");
    h.update(commit.get().to_be_bytes());
    if let Some(p) = previous {
        h.update([1]);
        h.update(p.get().to_be_bytes());
    } else {
        h.update([0]);
    }
    let name = save_name(origin);
    h.update((name.len() as u64).to_be_bytes());
    h.update(name);
    for id in [
        header.project.get(),
        header.identity.history.get(),
        header.identity.lineage.get(),
        header.identity.address_epoch.get(),
        header.root.get(),
        head.get(),
    ] {
        h.update(id.to_be_bytes());
    }
    h.update(prefix);
    for n in [node_count as u64, next_alias, next_branch] {
        h.update(n.to_be_bytes());
    }
    let redo: BTreeMap<_, _> = redo.iter().copied().collect();
    h.update((redo.len() as u64).to_be_bytes());
    for (p, c) in redo {
        h.update(p.get().to_be_bytes());
        h.update(c.get().to_be_bytes());
    }
    h.finalize().into()
}
fn node_item(n: &CapturedNodeInfo) -> Value {
    let record = n.origin.as_ref().expect("record");
    json!({"id":ptext(n.id),"checksum":hex(&record.checksum),"address":hex(&address_checksum(n)),"state":hex(&record.state)})
}
fn admission_item((o, p): (ObjectId, PersistentNodeId)) -> Value {
    let (kind, id) = object(o);
    json!({"kind":kind,"id":format!("{id:032x}"),"first":ptext(p)})
}
fn insert_originals(
    c: &Connection,
    cache: &mut Cache,
    source: &BlobStore,
    refs: &[BlobRef],
    limits: FileLimits,
    cancel: &CaptureCancellation,
) -> Result<Vec<BlobRef>, StorageError> {
    let mut new = Vec::new();
    let mut total = cache.originals.iter().map(|b| b.byte_len()).sum::<u64>();
    for b in refs {
        check(cancel)?;
        if cache.originals.contains(b) {
            continue;
        }
        total = total
            .checked_add(b.byte_len())
            .ok_or(StorageError::Limit("original bytes"))?;
        if total > limits.original_bytes || cache.originals.len() >= limits.originals {
            return Err(StorageError::Limit("originals"));
        }
        let raw = source.get(*b).ok_or(StorageError::MissingOriginal(*b))?;
        if raw.bytes().len() as u64 != b.byte_len() || sha(raw.bytes()) != b.sha256() {
            return Err(StorageError::Invalid("original bytes"));
        }
        c.execute(
            "INSERT INTO blobs VALUES(?1,?2)",
            params![
                b.sha256(),
                i64::try_from(b.byte_len()).map_err(|_| StorageError::Limit("original"))?
            ],
        )?;
        for (part, bytes) in raw.bytes().chunks(65536).enumerate() {
            check(cancel)?;
            c.execute(
                "INSERT INTO blob_chunks VALUES(?1,?2,?3)",
                params![b.sha256(), part as i64, bytes],
            )?;
        }
        cache.originals.insert(*b);
        cache.metadata_bytes += 64;
        new.push(*b);
    }
    Ok(new)
}
struct WriteCheck<'a> {
    probe: Probe,
    guard: Option<(&'a FileStamp, &'a Path, i64)>,
}
fn run(
    c: &mut Connection,
    cache: &mut Cache,
    source: &BlobStore,
    limits: FileLimits,
    capture: &mut HistoryCapture,
    request: &ArchiveWriteRequest,
    checks: WriteCheck<'_>,
) -> Result<ArchiveWriteOutcome, StorageError> {
    let probe = checks.probe;
    check(&request.cancellation)?;
    if request.version != ARCHIVE_PORT_VERSION {
        return Err(StorageError::Unsupported("archive port"));
    }
    let snapshot = capture.snapshot()?;
    let m = snapshot.manifest();
    if m.project_id != cache.header.project
        || m.identity != cache.header.identity
        || m.root != cache.header.root
    {
        return Err(StorageError::Conflict);
    }
    if m.archive_metadata.tags != cache.metadata.tags
        || m.archive_metadata.extensions != cache.metadata.extensions
    {
        return Err(StorageError::Unsupported("metadata editing"));
    }
    let replay = cache
        .checkpoints
        .iter()
        .find(|p| p.receipt.commit == request.commit)
        .cloned();
    let previous = cache.checkpoints.last().map(|p| p.receipt.commit);
    if replay.is_none() && request.expected != previous {
        return Err(StorageError::Conflict);
    }
    let input_ids: BTreeSet<_> = m.nodes.iter().map(|n| n.id).collect();
    if replay.is_none() && cache.nodes.keys().any(|id| !input_ids.contains(id)) {
        return Err(StorageError::Invalid("history shrink"));
    }
    if m.nodes.len() > limits.restore.nodes {
        return Err(StorageError::Limit("nodes"));
    }
    let flag = request.cancellation.clone();
    c.progress_handler(1000, Some(move || flag.is_cancelled()))?;
    let mut tx = Transaction::begin(c, true)?;
    if let Some((stamp, path, version)) = checks.guard
        && (!stamp.matches(path)?
            || tx
                .conn
                .query_row("PRAGMA data_version", [], |r| r.get::<_, i64>(0))?
                != version)
    {
        return Err(StorageError::ExternalChange);
    }
    let old_count = cache.order.len();
    let new_originals = if replay.is_none() {
        insert_originals(
            tx.conn,
            cache,
            source,
            &m.originals,
            limits,
            &request.cancellation,
        )?
    } else {
        vec![]
    };
    probe.step(Stage::Originals, &request.cancellation)?;
    let mut input: BTreeMap<PersistentNodeId, CachedNode> = BTreeMap::new();
    loop {
        check(&request.cancellation)?;
        match capture.poll(64)? {
            CapturePoll::Pending => (),
            CapturePoll::Complete => break,
            CapturePoll::Node(n) => {
                let mut info = n.info().clone();
                let bundle = encode_project(n.project(), limits.codec)?;
                if let Some(old) = cache.nodes.get(&info.id) {
                    if old.canonical != bundle.root
                        || old.info.parent != info.parent
                        || old.info.alias != info.alias
                        || old.info.coordinate != info.coordinate
                        || old.info.depth != info.depth
                        || old.info.operation != info.operation
                    {
                        return Err(StorageError::Invalid("immutable node conflict"));
                    }
                    if info
                        .origin
                        .as_ref()
                        .is_some_and(|origin| Some(origin) != old.info.origin.as_ref())
                    {
                        return Err(StorageError::Invalid("origin binding"));
                    }
                    info.origin = old.info.origin.clone();
                    input.insert(
                        info.id,
                        CachedNode {
                            info,
                            canonical: bundle.root,
                        },
                    );
                } else {
                    if replay.is_some() || info.origin.is_some() {
                        return Err(StorageError::Unsupported(
                            "missing original history; verified copy required",
                        ));
                    }
                    for block in bundle.blocks {
                        blocks::add(tx.conn, cache, block, limits)?;
                    }
                    let action = operation_bytes(&info.operation)?;
                    let checksum = node_checksum(info.id, info.parent, bundle.root, &action);
                    info.origin = Some(Arc::new(OriginalNodeRecord {
                        record_version: 2,
                        state: bundle.root.bytes(),
                        action,
                        checksum,
                    }));
                    input.insert(
                        info.id,
                        CachedNode {
                            info,
                            canonical: bundle.root,
                        },
                    );
                }
            }
        }
    }
    if !snapshot.is_complete() || input.len() != m.nodes.len() {
        return Err(StorageError::Invalid("incomplete capture"));
    }
    probe.step(Stage::States, &request.cancellation)?;
    let mut by_first: BTreeMap<_, Vec<_>> = BTreeMap::new();
    let mut all_admissions = BTreeMap::new();
    for a in &m.admissions {
        by_first
            .entry(a.first_node)
            .or_insert_with(Vec::new)
            .push((a.object, a.first_node));
        all_admissions.insert(a.object, a.first_node);
    }
    for facts in by_first.values_mut() {
        facts.sort_by_key(|(o, _)| object(*o));
    }
    let mut prefix = initial_prefix();
    let mut new_nodes = Vec::new();
    for info in &m.nodes {
        let n = input
            .get(&info.id)
            .ok_or(StorageError::Invalid("capture node"))?;
        prefix = extend_prefix(
            prefix,
            &n.info,
            by_first.get(&info.id).map_or(&[], Vec::as_slice),
        );
        if !cache.nodes.contains_key(&info.id) {
            new_nodes.push(n.clone());
        }
    }
    let digest = request_hash(
        &cache.header,
        RequestFrame {
            commit: request.commit,
            previous: request.expected,
            origin: request.origin,
            head: m.head,
            prefix,
            node_count: m.nodes.len(),
            next_alias: m.next_alias,
            next_branch: m.next_branch,
            redo: &m.redo_preferences,
        },
    );
    if let Some(old) = replay {
        let parsed = read::checkpoint(cache, old.navigation, limits)?;
        if parsed.request != digest {
            return Err(StorageError::Invalid("commit id reused"));
        }
        check(&request.cancellation)?;
        tx.commit()?;
        return Ok(ArchiveWriteOutcome::Replayed(old.receipt));
    }
    if cache
        .admissions
        .iter()
        .any(|(id, p)| all_admissions.get(id) != Some(p))
    {
        return Err(StorageError::Invalid("admission shrink/change"));
    }
    if cache.originals.iter().any(|b| !m.originals.contains(b)) {
        return Err(StorageError::Invalid("resource shrink"));
    }
    if new_nodes.is_empty() && new_originals.is_empty() {
        let old = cache.checkpoints.last().expect("existing history");
        let manual = cache.metadata.manual.as_ref().and_then(|r| {
            cache
                .checkpoints
                .iter()
                .find(|c| c.receipt.commit == r.commit)
        });
        let same_manual = manual
            .is_some_and(|p| p.receipt.head == m.head && same_redo(&p.redo, &m.redo_preferences));
        if (request.origin != SaveOrigin::Manual && m.source.state.revision == 0)
            || (request.origin == SaveOrigin::Manual && same_manual)
            || (request.origin != SaveOrigin::Manual
                && old.receipt.head == m.head
                && same_redo(&old.redo, &m.redo_preferences))
        {
            check(&request.cancellation)?;
            tx.commit()?;
            return Ok(ArchiveWriteOutcome::NoChange(old.receipt.clone()));
        }
    }
    new_nodes.sort_by_key(|n| n.info.alias);
    if new_nodes.first().is_some_and(|n| {
        cache
            .order
            .last()
            .is_some_and(|old| cache.nodes[old].info.alias >= n.info.alias)
    }) {
        return Err(StorageError::Invalid("new alias order"));
    }
    let new_ids: BTreeSet<_> = new_nodes.iter().map(|n| n.info.id).collect();
    let mut delta_admissions = Vec::new();
    for (o, p) in all_admissions {
        if cache.admissions.contains_key(&o) {
            continue;
        }
        if !new_ids.contains(&p) {
            return Err(StorageError::Invalid("late admission"));
        }
        cache.admissions.insert(o, p);
        cache.metadata_bytes += 64;
        delta_admissions.push((o, p));
    }
    for n in &new_nodes {
        let record = n.info.origin.as_ref().expect("record");
        tx.conn.execute(
            "INSERT INTO nodes VALUES(?1,?2,?3,?4,?5,?6,2)",
            params![
                n.info.id.get().to_be_bytes(),
                n.info.parent.map(|p| p.get().to_be_bytes()),
                record.state,
                n.info.depth as i64,
                record.action,
                record.checksum
            ],
        )?;
        tx.conn.execute(
            "INSERT INTO addresses VALUES(?1,?2,?3,?4,?5)",
            params![
                n.info.id.get().to_be_bytes(),
                bu64(n.info.alias),
                bu64(n.info.coordinate.branch()),
                bu64(n.info.coordinate.ordinal()),
                address_checksum(&n.info)
            ],
        )?;
        cache.metadata_bytes += 128 + record.action.len();
        cache.order.push(n.info.id);
        cache.nodes.insert(n.info.id, n.clone());
    }
    if cache.order.len() != m.nodes.len() || old_count + new_nodes.len() != m.nodes.len() {
        return Err(StorageError::Invalid("node closure"));
    }
    for (o, p) in &delta_admissions {
        let (kind, id) = object(*o);
        tx.conn.execute(
            "INSERT INTO object_ids VALUES(?1,?2,?3)",
            params![kind, id.to_be_bytes(), p.get().to_be_bytes()],
        )?;
    }
    probe.step(Stage::Nodes, &request.cancellation)?;
    let nodes = blocks::paged(
        tx.conn,
        cache,
        "nodes",
        new_nodes.iter().map(|n| node_item(&n.info)).collect(),
        limits,
    )?;
    let admissions = blocks::paged(
        tx.conn,
        cache,
        "admissions",
        delta_admissions.into_iter().map(admission_item).collect(),
        limits,
    )?;
    let originals = blocks::paged(
        tx.conn,
        cache,
        "originals",
        new_originals
            .iter()
            .map(|b| json!({"sha":hex(&b.sha256()),"bytes":b.byte_len().to_string()}))
            .collect(),
        limits,
    )?;
    let redo = blocks::paged(
        tx.conn,
        cache,
        "redo",
        m.redo_preferences
            .iter()
            .map(|(p, c)| json!({"parent":ptext(*p),"child":ptext(*c)}))
            .collect(),
        limits,
    )?;
    let (tags, extensions, features) = readonly_metadata(tx.conn, cache, limits)?;
    let generation = cache
        .checkpoints
        .last()
        .map_or(Some(1), |p| p.receipt.generation.checked_add(1))
        .ok_or(StorageError::Limit("generation"))?;
    let nav = blocks::make(
        tx.conn,
        cache,
        "skin-editor.checkpoint",
        json!({"header":header_json(&cache.header),"commit":format!("{:032x}",request.commit.get()),"previous":request.expected.map(|p|format!("{:032x}",p.get())),"generation":generation.to_string(),"origin":save_name(request.origin),"head":ptext(m.head),"node_count":m.nodes.len().to_string(),"prefix":hex(&prefix),"next_alias":m.next_alias.to_string(),"next_branch":m.next_branch.to_string(),"request":hex(&digest),"nodes":nodes.hex(),"admissions":admissions.hex(),"originals":originals.hex(),"redo":redo.hex(),"tags":tags.hex(),"extensions":extensions.hex(),"features":features.hex()}),
        vec![
            nodes, admissions, originals, redo, tags, extensions, features,
        ],
        vec![],
        limits,
    )?;
    let receipt = ArchiveReceipt {
        commit: request.commit,
        generation,
        head: m.head,
        origin: request.origin,
    };
    tx.conn.execute(
        "INSERT INTO commits VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",
        params![
            request.commit.get().to_be_bytes(),
            request.expected.map(|p| p.get().to_be_bytes()),
            bu64(generation),
            save_name(request.origin),
            m.head.get().to_be_bytes(),
            nav.bytes(),
            digest,
            cache.header.identity.address_epoch.get().to_be_bytes()
        ],
    )?;
    let mut meta = (*cache.metadata).clone();
    meta.last_commit = Some(receipt.clone());
    meta.protected = Some(receipt.clone());
    if request.origin == SaveOrigin::Manual {
        tx.conn.execute(
            "INSERT INTO manual_marks VALUES(?1,?2)",
            params![
                request.commit.get().to_be_bytes(),
                m.head.get().to_be_bytes()
            ],
        )?;
        meta.manual = Some(receipt.clone());
        meta.manual_marks.push((request.commit, m.head));
    }
    cache.checkpoints.push(Checkpoint {
        receipt: receipt.clone(),
        previous: request.expected,
        navigation: nav,
        prefix,
        node_count: m.nodes.len(),
        next_alias: m.next_alias,
        next_branch: m.next_branch,
        redo: m.redo_preferences.clone(),
    });
    cache.prefix = prefix;
    cache.metadata = Arc::new(meta);
    cache.metadata_bytes = cost(cache)?;
    if cache.metadata_bytes > limits.metadata_bytes {
        return Err(StorageError::Limit("metadata"));
    }
    let blobs = cache.originals.iter().map(|b| b.byte_len()).sum::<u64>();
    tx.conn.execute("INSERT INTO control VALUES(1,?1,?2,?3,?4,?5,?6,?7,?8,?9) ON CONFLICT(singleton) DO UPDATE SET generation=excluded.generation,save_id=excluded.save_id,manual_id=excluded.manual_id,protected_id=excluded.protected_id,node_count=excluded.node_count,block_count=excluded.block_count,blob_bytes=excluded.blob_bytes,metadata_bytes=excluded.metadata_bytes,address_epoch=excluded.address_epoch",params![bu64(generation),request.commit.get().to_be_bytes(),cache.metadata.manual.as_ref().map(|r|r.commit.get().to_be_bytes()),request.commit.get().to_be_bytes(),cache.nodes.len()as i64,cache.blocks.len()as i64,blobs as i64,cache.metadata_bytes as i64,cache.header.identity.address_epoch.get().to_be_bytes()])?;
    let pages = tx
        .conn
        .query_row("PRAGMA page_count", [], |r| r.get::<_, i64>(0))?;
    if u64::try_from(pages)
        .map_err(|_| StorageError::Invalid("page count"))?
        .saturating_mul(4096)
        > limits.file_bytes
    {
        return Err(StorageError::Limit("file bytes"));
    }
    probe.step(Stage::BeforeCommit, &request.cancellation)?;
    tx.commit()
        .map_err(|_| StorageError::OutcomeUnknown(request.commit))?;
    if probe
        .step(Stage::AfterCommit, &request.cancellation)
        .is_err()
    {
        return Err(StorageError::OutcomeUnknown(request.commit));
    }
    Ok(ArchiveWriteOutcome::Committed(receipt))
}
fn same_redo(
    a: &[(PersistentNodeId, PersistentNodeId)],
    b: &[(PersistentNodeId, PersistentNodeId)],
) -> bool {
    a.iter().copied().collect::<BTreeMap<_, _>>() == b.iter().copied().collect()
}
pub(super) fn existing(
    file: &mut SqliteProjectFile,
    capture: &mut HistoryCapture,
    request: ArchiveWriteRequest,
    probe: Probe,
) -> Result<ArchiveWriteOutcome, StorageError> {
    let result = run(
        &mut file.conn,
        &mut file.cache,
        &file.originals,
        file.limits,
        capture,
        &request,
        WriteCheck {
            probe,
            guard: Some((&file.stamp, &file.path, file.data_version)),
        },
    );
    if result.is_ok() {
        let stamp = FileStamp::read(&file.path)
            .map_err(|_| StorageError::OutcomeUnknown(request.commit))?;
        if stamp.handle != file.stamp.handle {
            return Err(StorageError::OutcomeUnknown(request.commit));
        }
        file.stamp = stamp;
        if file
            .conn
            .query_row("PRAGMA data_version", [], |r| r.get::<_, i64>(0))?
            != file.data_version
        {
            return Err(StorageError::OutcomeUnknown(request.commit));
        }
    }
    if request.cancellation.is_cancelled()
        && result.is_err()
        && !matches!(result, Err(StorageError::OutcomeUnknown(_)))
    {
        return Err(StorageError::Cancelled);
    }
    result
}

struct Staging(PathBuf);
impl Drop for Staging {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}
impl SqliteProjectFile {
    pub fn publish_new(
        path: impl AsRef<Path>,
        source: BlobStore,
        limits: FileLimits,
        capture: &mut HistoryCapture,
        request: ArchiveWriteRequest,
    ) -> Result<(Self, ArchiveWriteOutcome), StorageError> {
        let result = publish(
            path.as_ref(),
            source,
            limits,
            capture,
            &request,
            Probe::default(),
        );
        if result.is_err() {
            capture.cancel();
        }
        result
    }
}
pub(super) fn publish(
    path: &Path,
    source: BlobStore,
    limits: FileLimits,
    capture: &mut HistoryCapture,
    request: &ArchiveWriteRequest,
    probe: Probe,
) -> Result<(SqliteProjectFile, ArchiveWriteOutcome), StorageError> {
    check(&request.cancellation)?;
    if std::fs::symlink_metadata(path).is_ok() {
        return Err(StorageError::TargetExists);
    }
    let snapshot = capture.snapshot()?;
    let m = snapshot.manifest();
    if request.expected.is_some() || m.nodes.iter().any(|n| n.origin.is_some()) {
        return Err(StorageError::Unsupported(
            "new target requires fresh history; verified copy not implemented",
        ));
    }
    let mut nonce = [0u8; 16];
    getrandom::fill(&mut nonce).map_err(|_| StorageError::Io)?;
    let name = path
        .file_name()
        .ok_or(StorageError::Invalid("target filename"))?
        .to_string_lossy();
    let temp = Staging(path.with_file_name(format!(".{name}.{}.tmp", hex(&nonce))));
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp.0)
        .map_err(io)?;
    let mut conn =
        Connection::open_with_flags(&temp.0, rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE)?;
    configure(&mut conn, limits, &request.cancellation)?;
    conn.execute_batch(&format!("PRAGMA page_size=4096; PRAGMA application_id={APP_ID}; PRAGMA user_version={CONTAINER_VERSION}; PRAGMA journal_mode=DELETE; PRAGMA synchronous=EXTRA; PRAGMA max_page_count={};",limits.file_bytes/4096))?;
    conn.execute_batch(&schema::script())?;
    let header = Header {
        project: m.project_id,
        identity: m.identity.clone(),
        root: m.root,
    };
    conn.execute(
        "INSERT INTO header VALUES(1,'skin-editor-project',?1,2,1,2,?2,?3,?4,?5)",
        params![
            FILE_PROFILE,
            header.project.get().to_be_bytes(),
            header.identity.history.get().to_be_bytes(),
            header.identity.lineage.get().to_be_bytes(),
            header.root.get().to_be_bytes()
        ],
    )?;
    let fs = features();
    for f in &fs {
        conn.execute(
            "INSERT INTO features VALUES(?1,?2,1)",
            params![f["name"].as_str(), f["version"].as_str()],
        )?;
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
        features: fs,
        metadata_bytes: 0,
        prefix: initial_prefix(),
    };
    cache.metadata_bytes = cost(&cache)?;
    let outcome = run(
        &mut conn,
        &mut cache,
        &source,
        limits,
        capture,
        request,
        WriteCheck { probe, guard: None },
    )?;
    let (cache, verified_candidate) = read::full(
        &mut conn,
        &source,
        limits,
        RestoreSelection::Latest,
        &request.cancellation,
    )?;
    drop(verified_candidate);
    conn.close().map_err(|_| StorageError::Io)?;
    std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(&temp.0)
        .map_err(io)?
        .sync_all()
        .map_err(io)?;
    probe.step(Stage::BeforePublish, &request.cancellation)?;
    std::fs::hard_link(&temp.0, path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::AlreadyExists {
            StorageError::TargetExists
        } else {
            io(e)
        }
    })?;
    if probe
        .step(Stage::AfterPublish, &request.cancellation)
        .is_err()
    {
        return Err(StorageError::OutcomeUnknown(request.commit));
    }
    std::fs::remove_file(&temp.0).map_err(|_| StorageError::OutcomeUnknown(request.commit))?;
    let stamp = FileStamp::read(path).map_err(|_| StorageError::OutcomeUnknown(request.commit))?;
    let mut conn = Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE)
        .map_err(|_| StorageError::OutcomeUnknown(request.commit))?;
    configure(&mut conn, limits, &CaptureCancellation::default())
        .map_err(|_| StorageError::OutcomeUnknown(request.commit))?;
    conn.progress_handler(0, None::<fn() -> bool>)?;
    let data_version = conn.query_row("PRAGMA data_version", [], |r| r.get(0))?;
    Ok((
        SqliteProjectFile {
            conn,
            path: path.to_path_buf(),
            stamp,
            data_version,
            cache,
            originals: source,
            limits,
            invalid: false,
        },
        outcome,
    ))
}
