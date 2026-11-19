use super::*;

fn rows(count: usize) -> Vec<StateBlock> {
    let mut blocks: Vec<_> = (0..count)
        .map(|n| {
            let bytes = serde_json::to_vec(&json!({"kind":"split","schema":1,"blocks":[],
            "blobs":[],"value":{"n":n}}))
            .unwrap();
            StateBlock {
                hash: StateHash::from_bytes(sha(&bytes)),
                bytes,
            }
        })
        .collect();
    blocks.sort_unstable_by_key(|b| b.hash);
    blocks
}

fn append(c: &mut Connection, existing: usize, blocks: &[StateBlock]) {
    let tx = c.transaction().unwrap();
    let mut insert = tx
        .prepare("INSERT INTO blocks VALUES(?1,?2,?3,'split',1,NULL)")
        .unwrap();
    for (i, block) in blocks.iter().enumerate() {
        insert
            .execute(params![
                block.hash.bytes(),
                bu64((existing + i + 1) as u64),
                block.bytes
            ])
            .unwrap();
    }
    drop(insert);
    tx.commit().unwrap();
}

#[test]
fn small_row_limit_admits_multilevel_splits_in_both_key_orders() {
    let (_dir, _driver, mut file) = fixture();
    let limits = FileFactsLimits::default().file();
    let mut expected = rows(limits.blocks);
    let raw: usize = expected.iter().map(|b| b.bytes.len()).sum();
    assert!(256 + raw <= limits.metadata_bytes);
    assert_eq!(expected.len(), 8192);
    for descending in [false, true] {
        if descending {
            expected.reverse();
        }
        let mut c = Connection::open_in_memory().unwrap();
        c.execute_batch(&schema::script()).unwrap();
        append(&mut c, 0, &expected);
        file.cache.blocks = BTreeMap::new();
        file.cache.metadata_bytes = 256;
        let cancel = CaptureCancellation::default();
        read::load_blocks(&c, &mut file.cache, limits, &cancel).unwrap();
        assert_eq!(file.cache.blocks.len(), expected.len());
        assert_eq!(file.cache.metadata_bytes, 256 + raw);
        for b in &expected {
            let row = &file.cache.blocks[&b.hash];
            assert_eq!(row.block.bytes, b.bytes);
            assert_eq!(row.block.bytes.capacity(), b.bytes.len());
            assert_eq!(row.kind, "split");
        }
        c.execute(
            "INSERT INTO blocks VALUES(?1,?2,?3,'extra',1,NULL)",
            params![sha(b"extra"), bu64(limits.blocks as u64 + 1), b"extra"],
        )
        .unwrap();
        file.cache.blocks = BTreeMap::new();
        file.cache.metadata_bytes = 256;
        assert_eq!(
            read::load_blocks(&c, &mut file.cache, limits, &cancel),
            Err(StorageError::Limit("table rows"))
        );
        assert!(file.cache.blocks.is_empty());
        assert_eq!(file.cache.metadata_bytes, 256);
    }
}

#[test]
fn duplicate_block_keeps_the_prior_owner_and_byte_ledger_at_the_original_error_position() {
    let (_dir, _driver, mut file) = fixture();
    // Feed an existing key to the insertion boundary. A verified on-disk
    // blocks table has a unique SHA and the real reader starts empty.
    let r = file
        .cache
        .blocks
        .values()
        .find(|r| !r.refs.is_empty() && r.kind != "skin-editor.project")
        .unwrap();
    let key = r.block.hash;
    let dependency = r.refs[0];
    let original = r.block.bytes.as_ptr();
    let count = file.cache.blocks.len();
    let retained = file.cache.metadata_bytes;
    let c = Connection::open_in_memory().unwrap();
    c.execute_batch("CREATE TABLE blocks(sha,sequence,bytes,kind,schema_version,project_id)")
        .unwrap();
    c.execute(
        "INSERT INTO blocks VALUES(?1,?2,?3,?4,1,NULL)",
        params![key.bytes(), bu64(count as u64 + 1), r.block.bytes, r.kind],
    )
    .unwrap();
    let cancel = CaptureCancellation::default();
    let limits = FileFactsLimits::default().file();
    let stopped = CaptureCancellation::default();
    stopped.cancel();
    assert_eq!(
        read::load_blocks(&c, &mut file.cache, limits, &stopped),
        Err(StorageError::Cancelled)
    );
    assert_eq!(
        read::load_blocks(&c, &mut file.cache, limits, &cancel),
        Err(StorageError::Invalid("duplicate block"))
    );
    assert_eq!(file.cache.blocks[&key].block.bytes.as_ptr(), original);
    assert_eq!(file.cache.metadata_bytes, retained);
    assert_eq!(file.cache.blocks.len(), count);

    file.cache.blocks.remove(&dependency).unwrap();
    c.execute("UPDATE blocks SET sequence=?1", [bu64(count as u64)])
        .unwrap();
    assert_eq!(
        read::load_blocks(&c, &mut file.cache, limits, &cancel),
        Err(StorageError::Invalid("block dependencies"))
    );
    c.execute("UPDATE blocks SET schema_version=2", []).unwrap();
    assert_eq!(
        read::load_blocks(&c, &mut file.cache, limits, &cancel),
        Err(StorageError::Invalid("block derived fields"))
    );
    assert_eq!(file.cache.blocks[&key].block.bytes.as_ptr(), original);
    assert_eq!(file.cache.metadata_bytes, retained);
}

#[test]
fn late_load_error_after_internal_splits_releases_this_openings_pins() {
    let (dir, _driver, mut file) = fixture();
    let prefix = file.cache.blocks.len();
    let fresh = rows(160);
    append(&mut file.conn, prefix, &fresh);
    // With 11 root keys and 12 leaves of at most 11 keys, height one
    // cannot retain this prefix. The following row fails before insertion.
    file.conn
        .execute(
            "INSERT INTO blocks VALUES(?1,?2,?3,'bad-kind',1,NULL)",
            params![
                sha(b"broken"),
                bu64((prefix + fresh.len() + 1) as u64),
                b"broken"
            ],
        )
        .unwrap();
    drop(file);
    let before = digest(&dir.file());
    let store = BlobStore::default();
    let held = store.stage_bytes(b"independent owner".to_vec()).unwrap();
    let retained = store.usage().unwrap().bytes;
    let limits = FileFactsLimits::default().file();
    for _ in 0..2 {
        let result = SqliteProjectFile::load(
            dir.file(),
            store.clone(),
            limits,
            RestoreSelection::Manual,
            CaptureCancellation::default(),
        );
        assert_eq!(
            result.err(),
            Some(StorageError::Codec(CodecError::InvalidJson))
        );
        let usage = store.usage().unwrap();
        assert!(usage.peak_bytes > retained);
        assert_eq!(usage.bytes, retained);
        assert_eq!(digest(&dir.file()), before);
    }
    drop(held);
    assert_eq!(store.usage().unwrap().bytes, 0);
}
