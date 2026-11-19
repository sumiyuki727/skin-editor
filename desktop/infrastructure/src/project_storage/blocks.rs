use super::*;
mod metadata_pages;
pub(super) use metadata_pages::MetadataPages;
pub(super) const PAGE_ITEMS: usize = 256;
pub(super) struct Row {
    pub block: StateBlock,
    pub kind: String,
    pub refs: Vec<StateHash>,
    pub blobs: Vec<[u8; 32]>,
}
fn digests(v: Value) -> Result<Vec<StateHash>, StorageError> {
    let values = array(v)?
        .into_iter()
        .map(|v| hash(&text(v)?))
        .collect::<Result<Vec<_>, _>>()?;
    if values.windows(2).any(|p| p[0] >= p[1]) {
        return Err(StorageError::Invalid("reference ordering"));
    }
    Ok(values)
}
pub(super) fn parse(block: StateBlock, limits: FormatLimits) -> Result<Row, StorageError> {
    if sha(&block.bytes) != block.hash.bytes() {
        return Err(StorageError::Invalid("block hash"));
    }
    let mut f = Fields::new(crate::project_format::json::parse(&block.bytes, limits)?)?;
    let kind = f.text("kind")?;
    if f.take("schema")?.as_u64() != Some(1) {
        return Err(StorageError::Unsupported("block schema"));
    }
    let refs = digests(f.take("blocks")?)?;
    let blobs = digests(f.take("blobs")?)?
        .into_iter()
        .map(StateHash::bytes)
        .collect();
    let _value = Fields::new(f.take("value")?)?;
    f.finish()?;
    Ok(Row {
        block,
        kind,
        refs,
        blobs,
    })
}
pub(super) fn value(row: &Row, limits: FormatLimits) -> Result<Fields, StorageError> {
    value_fields(crate::project_format::json::parse(
        &row.block.bytes,
        limits,
    )?)
}
fn compact_page_value(row: &Row, limits: FormatLimits) -> Result<Fields, StorageError> {
    value_fields(crate::project_format::json::parse_compact_page(
        &row.block.bytes,
        limits,
    )?)
}
fn value_fields(parsed: Value) -> Result<Fields, StorageError> {
    let mut f = Fields::new(parsed)?;
    Fields::new(f.take("value")?)
}
pub(super) fn add(
    c: &Connection,
    cache: &mut Cache,
    block: StateBlock,
    limits: FileLimits,
) -> Result<StateHash, StorageError> {
    if let Some(old) = cache.blocks.get(&block.hash) {
        if old.block.bytes != block.bytes {
            return Err(StorageError::Invalid("hash collision"));
        }
        return Ok(block.hash);
    }
    let sequence = (cache.blocks.len() as u64)
        .checked_add(1)
        .ok_or(StorageError::Limit("blocks"))?;
    let row = parse(block, limits.codec)?;
    if cache.blocks.len() >= limits.blocks
        || row.block.bytes.len() > limits.metadata_bytes.saturating_sub(cache.metadata_bytes)
    {
        return Err(StorageError::Limit("metadata/blocks"));
    }
    for target in &row.refs {
        if !cache.blocks.contains_key(target) {
            return Err(StorageError::Invalid("block order"));
        }
    }
    for target in &row.blobs {
        if !cache.originals.iter().any(|b| b.sha256() == *target) {
            return Err(StorageError::Invalid("blob reference"));
        }
    }
    let project = if row.kind == "skin-editor.project" {
        Some(value(&row, limits.codec)?.text("id")?)
    } else {
        None
    };
    c.execute(
        "INSERT INTO blocks VALUES(?1,?2,?3,?4,1,?5)",
        params![
            row.block.hash.bytes(),
            bu64(sequence),
            row.block.bytes,
            row.kind,
            project
        ],
    )?;
    for target in &row.refs {
        c.execute(
            "INSERT INTO block_refs VALUES(?1,?2)",
            params![row.block.hash.bytes(), target.bytes()],
        )?;
    }
    for target in &row.blobs {
        c.execute(
            "INSERT INTO blob_refs VALUES(?1,?2)",
            params![row.block.hash.bytes(), target],
        )?;
    }
    let key = row.block.hash;
    cache.metadata_bytes += row.block.bytes.len();
    cache.blocks.insert(key, row);
    Ok(key)
}
pub(super) fn make(
    c: &Connection,
    cache: &mut Cache,
    kind: &str,
    v: Value,
    refs: Vec<StateHash>,
    blobs: Vec<[u8; 32]>,
    limits: FileLimits,
) -> Result<StateHash, StorageError> {
    let refs: BTreeSet<_> = refs.into_iter().collect();
    let blobs: BTreeSet<_> = blobs.into_iter().collect();
    let raw=serde_json::to_vec(&json!({"kind":kind,"schema":1,"blocks":refs.iter().map(|h|h.hex()).collect::<Vec<_>>(),"blobs":blobs.iter().map(|h|hex(h)).collect::<Vec<_>>(),"value":v})).map_err(|_|StorageError::Invalid("json encode"))?;
    if raw.len() > limits.codec.block_bytes {
        return Err(StorageError::Limit("block"));
    }
    let hash = StateHash::from_bytes(sha(&raw));
    add(c, cache, StateBlock { hash, bytes: raw }, limits)
}
pub(super) fn paged(
    c: &Connection,
    cache: &mut Cache,
    role: &str,
    items: Vec<Value>,
    limits: FileLimits,
) -> Result<StateHash, StorageError> {
    let mut pages = Vec::new();
    for page in items.chunks(PAGE_ITEMS) {
        let (refs, blobs) = page_refs(role, page)?;
        pages.push(make(
            c,
            cache,
            "skin-editor.history-page",
            json!({"role":role,"items":page}),
            refs,
            blobs,
            limits,
        )?);
    }
    make(
        c,
        cache,
        "skin-editor.history-index",
        json!({"role":role,"len":items.len().to_string(),"pages":pages.iter().map(|h|h.hex()).collect::<Vec<_>>()}),
        pages,
        vec![],
        limits,
    )
}
fn page_refs(role: &str, items: &[Value]) -> Result<(Vec<StateHash>, Vec<[u8; 32]>), StorageError> {
    let mut refs = Vec::new();
    let mut blobs = Vec::new();
    for v in items {
        if role == "nodes" {
            refs.push(hash(
                v["state"]
                    .as_str()
                    .ok_or(StorageError::Invalid("index state"))?,
            )?);
        }
        if role == "originals" {
            blobs.push(
                hash(
                    v["sha"]
                        .as_str()
                        .ok_or(StorageError::Invalid("index blob"))?,
                )?
                .bytes(),
            );
        }
    }
    Ok((refs, blobs))
}
pub(super) fn read_paged(
    cache: &Cache,
    root: StateHash,
    role: &str,
    limits: FileLimits,
) -> Result<Vec<Value>, StorageError> {
    let mut rows = Vec::new();
    visit_pages(cache, root, role, limits, value, |page| rows.extend(page))?;
    Ok(rows)
}

fn visit_pages(
    cache: &Cache,
    root: StateHash,
    role: &str,
    limits: FileLimits,
    read_page: fn(&Row, FormatLimits) -> Result<Fields, StorageError>,
    mut accept: impl FnMut(Vec<Value>),
) -> Result<(), StorageError> {
    let row = cache
        .blocks
        .get(&root)
        .ok_or(StorageError::Invalid("index root"))?;
    if row.kind != "skin-editor.history-index" || !row.blobs.is_empty() {
        return Err(StorageError::Invalid("index kind"));
    }
    let mut f = value(row, limits.codec)?;
    if f.text("role")? != role {
        return Err(StorageError::Invalid("index role"));
    }
    let len = usize::try_from(f.num("len")?).map_err(|_| StorageError::Limit("index items"))?;
    if len > limits.codec.items {
        return Err(StorageError::Limit("index items"));
    }
    let pages = array(f.take("pages")?)?
        .into_iter()
        .map(|v| hash(&text(v)?))
        .collect::<Result<Vec<_>, _>>()?;
    f.finish()?;
    if pages.len() != len.div_ceil(PAGE_ITEMS)
        || pages
            .iter()
            .copied()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>()
            != row.refs
    {
        return Err(StorageError::Invalid("index refs"));
    }
    let mut remaining = len;
    for h in pages {
        let page = cache.blocks.get(&h).ok_or(StorageError::Invalid("page"))?;
        if page.kind != "skin-editor.history-page" {
            return Err(StorageError::Invalid("page kind"));
        }
        let mut f = read_page(page, limits.codec)?;
        if f.text("role")? != role {
            return Err(StorageError::Invalid("page role"));
        }
        let items = array(f.take("items")?)?;
        f.finish()?;
        if items.len() != PAGE_ITEMS.min(remaining) {
            return Err(StorageError::Invalid("page count"));
        }
        let (refs, blobs) = page_refs(role, &items)?;
        if refs
            .into_iter()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>()
            != page.refs
            || blobs
                .into_iter()
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>()
                != page.blobs
        {
            return Err(StorageError::Invalid("page refs"));
        }
        remaining -= items.len();
        accept(items);
    }
    Ok(())
}
pub(super) fn decoded_state_blocks<'a>(
    cache: &'a Cache,
    root: StateHash,
    seen: &mut BTreeSet<StateHash>,
    limits: FileLimits,
    remaining_decode_bytes: &mut usize,
) -> Result<impl ExactSizeIterator<Item = &'a StateBlock> + use<'a>, StorageError> {
    let mut local = BTreeSet::new();
    let mut todo = vec![root];
    let mut bytes = 0usize;
    while let Some(h) = todo.pop() {
        if !local.insert(h) {
            continue;
        }
        let r = cache
            .blocks
            .get(&h)
            .ok_or(StorageError::Invalid("closure"))?;
        bytes = bytes
            .checked_add(r.block.bytes.len())
            .ok_or(StorageError::Limit("closure"))?;
        if bytes > limits.codec.total_bytes {
            return Err(StorageError::Limit("closure"));
        }
        if bytes > *remaining_decode_bytes {
            return Err(StorageError::Limit("decoded bytes"));
        }
        todo.extend(&r.refs);
    }
    // All distinct payload lengths are known before copying the first block.
    *remaining_decode_bytes -= bytes;
    seen.extend(&local);
    Ok(local.into_iter().map(|h| &cache.blocks[&h].block))
}

pub(super) fn mark(
    cache: &Cache,
    root: StateHash,
    seen: &mut BTreeSet<StateHash>,
) -> Result<(), StorageError> {
    let mut todo = vec![root];
    while let Some(h) = todo.pop() {
        if !seen.insert(h) {
            continue;
        }
        let row = cache
            .blocks
            .get(&h)
            .ok_or(StorageError::Invalid("closure"))?;
        todo.extend(&row.refs);
    }
    Ok(())
}
