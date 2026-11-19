use super::*;

#[test]
fn wrong_directory_length_still_checks_every_hash_then_unknown_fields() {
    let (_dir, _driver, mut file) = fixture();
    let mut small = file.limits;
    small.codec.items = read::METADATA_ITEMS;
    let mut wide = file.limits;
    wide.codec.json_depth = 12;
    small.codec.json_depth = wide.codec.json_depth;
    let valid = "00".repeat(32);
    for (last, extra, expected) in [
        (json!([]), true, StorageError::Invalid("text")),
        (json!("GG"), true, StorageError::Invalid("hex")),
        (json!("00"), true, StorageError::Invalid("fixed bytes")),
        (
            json!(valid),
            true,
            StorageError::Unsupported("metadata field"),
        ),
        (json!(valid), false, StorageError::Invalid("index refs")),
    ] {
        let mut pages = vec![json!(valid); 33];
        pages.push(last);
        let mut body = json!({"role":"tags", "len":"1", "pages":pages});
        if extra {
            body["extra"] = Value::Null;
        }
        let root = blocks::make(
            &file.conn,
            &mut file.cache,
            "skin-editor.history-index",
            body,
            vec![],
            vec![],
            file.limits,
        )
        .unwrap();
        for limits in [small, wide] {
            assert_eq!(
                blocks::MetadataPages::read(&file.cache, root, "tags", limits)
                    .err()
                    .as_ref(),
                Some(&expected)
            );
        }
    }
    // The index DOM itself must pass bounded decoding before any role,
    // field, count or hash check, even if the body would later be rejected.
    let mut nested = Value::Null;
    for _ in 0..12 {
        nested = json!([nested]);
    }
    let root = blocks::make(
        &file.conn,
        &mut file.cache,
        "skin-editor.history-index",
        json!({"role":"wrong","pages":[nested]}),
        vec![],
        vec![],
        file.limits,
    )
    .unwrap();
    for limits in [small, wide] {
        assert_eq!(
            blocks::MetadataPages::read(&file.cache, root, "tags", limits).err(),
            Some(StorageError::Codec(CodecError::ResourceLimit(
                "json_structure"
            )))
        );
    }
}

#[test]
fn reference_set_equality_keeps_nonadjacent_order_and_rejects_extra_or_unsorted_refs() {
    let (_dir, _driver, mut file) = fixture();
    let mut small = file.limits;
    small.codec.items = read::METADATA_ITEMS;
    let root = blocks::paged(
        &file.conn,
        &mut file.cache,
        "tags",
        [7, 3, 7]
            .into_iter()
            .flat_map(|v| std::iter::repeat_n(json!(v), blocks::PAGE_ITEMS))
            .collect(),
        file.limits,
    )
    .unwrap();
    let read = |cache: &Cache| blocks::MetadataPages::read(cache, root, "tags", small);
    let rows = read(&file.cache).unwrap();
    assert_eq!(rows.len(), 3 * blocks::PAGE_ITEMS);
    assert_eq!(rows[0], 7);
    assert_eq!(rows[blocks::PAGE_ITEMS], 3);
    assert!(std::ptr::eq(&rows[0], &rows[2 * blocks::PAGE_ITEMS]));
    let refs = &file.cache.blocks[&root].refs;
    assert_eq!(refs.len(), 2);
    let (a, b) = (refs[0], refs[1]);
    for wrong in [vec![a], vec![b, a], vec![a, b, b], vec![a, b, root]] {
        file.cache.blocks.get_mut(&root).unwrap().refs = wrong;
        assert_eq!(
            read(&file.cache).err(),
            Some(StorageError::Invalid("index refs"))
        );
    }
}

#[test]
fn sqlite_loaded_blocks_hold_exact_raw_kind_and_digest_capacities() {
    let (dir, _driver, file) = fixture();
    drop(file);
    let (loaded, _candidate) = open(&dir.file(), BlobStore::default(), RestoreSelection::Manual);
    let mut raw_bytes = 0;
    let mut block_refs = 0;
    let mut blob_refs = 0;
    for row in loaded.cache.blocks.values() {
        let raw = &row.block.bytes;
        assert_eq!(raw.capacity(), raw.len());
        assert_eq!(row.kind.capacity(), row.kind.len());
        assert_eq!(row.refs.capacity(), row.refs.len());
        assert_eq!(row.blobs.capacity(), row.blobs.len());
        assert!(
            row.kind.capacity() + 32 * (row.refs.capacity() + row.blobs.capacity()) <= raw.len()
        );
        raw_bytes += raw.len();
        block_refs += row.refs.len();
        blob_refs += row.blobs.len();
    }
    assert!(raw_bytes <= loaded.cache.metadata_bytes);
    assert!(block_refs > 0 && blob_refs > 0);
}

#[test]
fn escaped_kind_and_non_power_of_two_digest_lists_keep_capacity_and_error_order() {
    let hashes: Vec<_> = (0..17u8).map(|n| hex(&[n; 32])).collect();
    let parse = |refs: &[String], blobs: &[String]| {
        let body = json!({"kind":"esc\n雪", "schema":1, "blocks":refs, "blobs":blobs, "value":{}});
        let bytes = serde_json::to_vec(&body).unwrap();
        blocks::parse(
            StateBlock {
                hash: StateHash::from_bytes(sha(&bytes)),
                bytes,
            },
            FormatLimits::default(),
        )
    };
    let held = parse(&hashes, &hashes[..5]).unwrap().row;
    assert_eq!(held.kind, "esc\n雪");
    assert_eq!(held.kind.capacity(), held.kind.len());
    assert_eq!((held.refs.len(), held.refs.capacity()), (17, 17));
    assert_eq!((held.blobs.len(), held.blobs.capacity()), (5, 5));
    assert_eq!(held.refs[16].bytes(), [16; 32]);
    assert_eq!(held.blobs[4], [4; 32]);
    let mut invalid = hashes;
    invalid.swap(0, 1);
    assert_eq!(
        parse(&[], &invalid).err(),
        Some(StorageError::Invalid("reference ordering"))
    );
    invalid[16] = "GG".into();
    assert_eq!(
        parse(&invalid, &[]).err(),
        Some(StorageError::Invalid("hex"))
    );
    assert_eq!(
        parse(&[], &invalid).err(),
        Some(StorageError::Invalid("hex"))
    );
}
