use super::*;

fn project_block(body: Value, refs: &[String], blobs: &[String]) -> StateBlock {
    let bytes = serde_json::to_vec(&json!({"kind":"skin-editor.project","schema":1,
        "blocks":refs,"blobs":blobs,"value":body}))
    .unwrap();
    StateBlock {
        hash: StateHash::from_bytes(sha(&bytes)),
        bytes,
    }
}

#[test]
fn full_small_block_transfers_exact_id_and_keeps_envelope_errors_first() {
    let limits = FormatLimits {
        items: read::METADATA_ITEMS,
        block_bytes: 256 * 1024,
        ..FormatLimits::default()
    };
    let overhead = project_block(json!({"id":""}), &[], &[]).bytes.len();
    let id = "a".repeat(limits.block_bytes - overhead);
    let block = project_block(json!({"id":id}), &[], &[]);
    assert_eq!(block.bytes.len(), limits.block_bytes);
    let parsed = blocks::parse(block, limits).unwrap();
    let decoded = parsed.project_id.unwrap().unwrap();
    assert_eq!(decoded, id);
    assert_eq!(decoded.capacity(), decoded.len());
    for items in [limits.items, FormatLimits::default().items] {
        let bytes = br#"{"kind":"skin-editor.project","schema":1,"blocks":[],"blobs":[],"value":{"id":false},"future":0}"#.to_vec();
        let source = StateBlock {
            hash: StateHash::from_bytes(sha(&bytes)),
            bytes,
        };
        assert_eq!(
            blocks::parse(source, FormatLimits { items, ..limits }).err(),
            Some(StorageError::Unsupported("metadata field"))
        );
    }
}

#[test]
fn deferred_id_preserves_add_order_and_reuses_the_stored_block() {
    let (_dir, _driver, mut file) = fixture();
    let limits = file.limits;
    let full = FileLimits {
        blocks: 0,
        ..limits
    };
    let missing = ["ed".repeat(32)];
    let count = file.cache.blocks.len();
    let metadata = file.cache.metadata_bytes;
    for (budget, refs, blobs, expected) in [
        (
            full,
            &missing[..],
            &missing[..],
            StorageError::Limit("metadata/blocks"),
        ),
        (
            limits,
            &missing[..],
            &missing[..],
            StorageError::Invalid("block order"),
        ),
        (
            limits,
            &[][..],
            &missing[..],
            StorageError::Invalid("blob reference"),
        ),
        (limits, &[][..], &[][..], StorageError::Invalid("id")),
    ] {
        let bad = project_block(json!({}), refs, blobs);
        assert_eq!(
            blocks::add(&file.conn, &mut file.cache, bad, budget),
            Err(expected)
        );
        assert_eq!(file.cache.blocks.len(), count);
        assert_eq!(file.cache.metadata_bytes, metadata);
        let rows = file
            .conn
            .query_row("SELECT count(*) FROM blocks", [], |r| r.get::<_, i64>(0))
            .unwrap();
        assert_eq!(rows, count as i64);
    }
    let id = "雪\0another\n";
    let good = project_block(json!({"id":id}), &[], &[]);
    let key = blocks::add(&file.conn, &mut file.cache, good.clone(), limits).unwrap();
    let saved = file
        .conn
        .query_row(
            "SELECT project_id FROM blocks WHERE sha=?1",
            [key.bytes()],
            |r| r.get::<_, String>(0),
        )
        .unwrap();
    assert_eq!(saved, id);
    assert_eq!(
        blocks::add(&file.conn, &mut file.cache, good, full),
        Ok(key)
    );
    assert_eq!(file.cache.blocks.len(), count + 1);
}

#[test]
fn late_sql_id_mismatch_precedes_dependencies_and_releases_only_opening_leases() {
    let (dir, _driver, file) = fixture();
    let seq = file.cache.blocks.len() as u64 + 1;
    assert!(seq > 2);
    let bad = project_block(json!({"id":"expected"}), &["ed".repeat(32)], &[]);
    file.conn
        .execute(
            "INSERT INTO blocks VALUES(?1,?2,?3,'skin-editor.project',1,'other')",
            params![bad.hash.bytes(), bu64(seq), bad.bytes],
        )
        .unwrap();
    drop(file);
    let before = digest(&dir.file());
    let store = BlobStore::default();
    let held = store
        .stage_bytes(b"keep this independent lease".to_vec())
        .unwrap();
    let retained = store.usage().unwrap().bytes;
    assert!(retained > 0);
    let limits = FileLimits {
        codec: FormatLimits {
            items: read::METADATA_ITEMS,
            ..FormatLimits::default()
        },
        ..FileLimits::default()
    };
    for _ in 0..2 {
        let result = SqliteProjectFile::load(
            dir.file(),
            store.clone(),
            limits,
            RestoreSelection::Manual,
            CaptureCancellation::default(),
        );
        assert_eq!(result.err(), Some(StorageError::Invalid("block project")));
        let usage = store.usage().unwrap();
        assert!(usage.peak_bytes > retained);
        assert_eq!(usage.bytes, retained);
        assert_eq!(digest(&dir.file()), before);
    }
    drop(held);
    assert_eq!(store.usage().unwrap().bytes, 0);
}
