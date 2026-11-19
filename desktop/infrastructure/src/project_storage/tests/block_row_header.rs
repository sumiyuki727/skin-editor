use super::*;
use rusqlite::types::Value as SqlValue;

#[test]
fn length_identity_sequence_and_payload_errors_preserve_the_loaded_prefix() {
    let (_dir, _driver, mut file) = fixture();
    let count = file.cache.blocks.len();
    let retained = file.cache.metadata_bytes;
    let (key, old) = file.cache.blocks.first_key_value().unwrap();
    let key = *key;
    let original = old.block.bytes.as_ptr();
    let c = Connection::open_in_memory().unwrap();
    // Isolate the next row after an admitted prefix. This table also admits
    // malformed widths which an ordinary project's constraints reject.
    c.execute_batch("CREATE TABLE blocks(sha,sequence,bytes,kind,schema_version,project_id)")
        .unwrap();
    let limits = FileLimits {
        metadata_bytes: retained + 1,
        ..FileFactsLimits::default().file()
    };
    let column_type = || StorageError::Sql {
        code: None,
        class: "column type",
    };
    let wrong_width = || StorageError::Invalid("fixed bytes");
    let bad_json = || SqlValue::Blob(b"!".to_vec());
    let sha_bytes = sha(b"!");
    let valid_hash = || SqlValue::Blob(sha_bytes.to_vec());
    let mut changed = sha_bytes;
    changed[31] ^= 1;
    let next = count as u64 + 1;
    let sequence = || SqlValue::Blob(next.to_be_bytes().to_vec());
    let wrong_sequence = || SqlValue::Blob(next.to_le_bytes().to_vec());
    let cancel = CaptureCancellation::default();
    for (hash, seq, bytes, expected) in [
        (
            SqlValue::Blob(vec![]),
            SqlValue::Null,
            SqlValue::Null,
            column_type(),
        ),
        (
            SqlValue::Null,
            SqlValue::Null,
            SqlValue::Blob(vec![0; 2]),
            StorageError::Limit("metadata bytes"),
        ),
        (
            SqlValue::Text("x".repeat(32)),
            SqlValue::Blob(vec![]),
            bad_json(),
            column_type(),
        ),
        (
            SqlValue::Blob(vec![0; 31]),
            SqlValue::Null,
            bad_json(),
            wrong_width(),
        ),
        (
            SqlValue::Blob(vec![0; 33]),
            wrong_sequence(),
            bad_json(),
            wrong_width(),
        ),
        (
            valid_hash(),
            SqlValue::Integer(1),
            bad_json(),
            column_type(),
        ),
        (
            valid_hash(),
            SqlValue::Blob(vec![0; 9]),
            SqlValue::Text("!".into()),
            wrong_width(),
        ),
        (
            SqlValue::Blob(changed.to_vec()),
            wrong_sequence(),
            SqlValue::Text("!".into()),
            StorageError::Invalid("block sequence"),
        ),
        (
            SqlValue::Blob(changed.to_vec()),
            sequence(),
            SqlValue::Text("!".into()),
            column_type(),
        ),
        (
            SqlValue::Blob(changed.to_vec()),
            sequence(),
            bad_json(),
            StorageError::Invalid("block hash"),
        ),
        (
            valid_hash(),
            sequence(),
            bad_json(),
            StorageError::Codec(CodecError::InvalidJson),
        ),
    ] {
        c.execute(
            "INSERT INTO blocks VALUES(?1,?2,?3,'fixture',1,NULL)",
            params![hash, seq, bytes],
        )
        .unwrap();
        assert_eq!(
            read::load_blocks(&c, &mut file.cache, limits, &cancel),
            Err(expected)
        );
        assert_eq!(file.cache.blocks.len(), count);
        assert_eq!(file.cache.metadata_bytes, retained);
        assert_eq!(file.cache.blocks[&key].block.bytes.as_ptr(), original);
        c.execute("DELETE FROM blocks", []).unwrap();
    }
    c.close().unwrap();
}

#[test]
fn late_hash_type_and_sequence_gap_release_only_the_failed_openings_leases() {
    for (hash, expected) in [
        (
            SqlValue::Text("雪".repeat(32)),
            StorageError::Sql {
                code: None,
                class: "column type",
            },
        ),
        (
            SqlValue::Blob(vec![0; 32]),
            StorageError::Invalid("block sequence"),
        ),
    ] {
        let (dir, _driver, file) = fixture();
        assert!(file.cache.blocks.len() > 1);
        // The genuine schema accepts 32 TEXT characters. Both cases sort
        // last and have the same out-of-sequence, well-formed eight bytes.
        file.conn
            .execute(
                "INSERT INTO blocks VALUES(?1,?2,?3,'fixture',1,NULL)",
                params![hash, bu64(u64::MAX), b"!"],
            )
            .unwrap();
        drop(file);
        let before = digest(&dir.file());
        let store = BlobStore::default();
        let independent = store
            .stage_bytes(b"separate block header lease".to_vec())
            .unwrap();
        let baseline = store.usage().unwrap().bytes;
        for _ in 0..2 {
            let result = SqliteProjectFile::load(
                dir.file(),
                store.clone(),
                FileFactsLimits::default().file(),
                RestoreSelection::Manual,
                CaptureCancellation::default(),
            );
            assert_eq!(result.err().as_ref(), Some(&expected));
            let usage = store.usage().unwrap();
            assert!(usage.peak_bytes > baseline);
            assert_eq!(usage.bytes, baseline);
            assert_eq!(digest(&dir.file()), before);
        }
        drop(independent);
        assert_eq!(store.usage().unwrap().bytes, 0);
    }
}
