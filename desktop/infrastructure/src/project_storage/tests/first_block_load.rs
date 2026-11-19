use super::*;

fn small_codec() -> FormatLimits {
    FormatLimits {
        items: read::METADATA_ITEMS,
        json_values: 8192,
        block_bytes: 256 * 1024,
        json_depth: 32,
        ..FormatLimits::default()
    }
}

fn block(bytes: Vec<u8>) -> StateBlock {
    StateBlock {
        hash: StateHash::from_bytes(sha(&bytes)),
        bytes,
    }
}

#[test]
fn full_size_block_moves_its_bytes_and_keeps_both_digest_lists_exact() {
    let limits = small_codec();
    let hashes: Vec<_> = (0..3840).map(|i| format!("{i:064x}")).collect();
    let body = json!({"kind":"line\n雪", "schema":1, "blocks":hashes[..2049],
        "blobs":hashes[2049..], "value":{"nested":[{"unused":true}]}});
    let mut raw = serde_json::to_vec(&body).unwrap();
    assert!(raw.len() < limits.block_bytes);
    raw.resize(limits.block_bytes, b' ');
    let source = block(raw);
    let ptr = source.bytes.as_ptr();
    let parsed = blocks::parse(source, limits).unwrap().row;
    assert_eq!(parsed.block.bytes.as_ptr(), ptr);
    assert_eq!(parsed.kind, "line\n雪");
    assert_eq!(parsed.kind.capacity(), parsed.kind.len());
    assert_eq!((parsed.refs.len(), parsed.refs.capacity()), (2049, 2049));
    assert_eq!((parsed.blobs.len(), parsed.blobs.capacity()), (1791, 1791));
    assert_eq!(
        parsed.refs.last().unwrap().bytes(),
        hash(&hashes[2048]).unwrap().bytes()
    );
    assert_eq!(parsed.blobs[0], hash(&hashes[2049]).unwrap().bytes());
    let mut oversize = parsed.block.bytes;
    oversize.push(b' ');
    assert_eq!(
        blocks::parse(block(oversize), limits).err(),
        Some(StorageError::Codec(CodecError::ResourceLimit(
            "block_bytes"
        )))
    );
}

#[test]
fn impossible_digest_counts_keep_field_and_ordering_errors_in_both_profiles() {
    let low = "00".repeat(32);
    let high = "ff".repeat(32);
    // Three valid but unordered entries, a late bad entry, then short
    // values. The declared length cannot fit as many encoded digests.
    for (blobs, bad, expected) in [
        (false, json!(0), "text"),
        (true, json!("GG"), "hex"),
        (true, json!("00"), "fixed bytes"),
    ] {
        let mut values = vec![json!(high), json!(low), json!(low), bad];
        values.resize(200, Value::Null);
        let mut body = json!({"kind":"fixture","schema":1,"blocks":[low],"blobs":[],"value":{}});
        body[if blobs { "blobs" } else { "blocks" }] = json!(values);
        let raw = serde_json::to_vec(&body).unwrap();
        assert!(200 > raw.len() / blocks::MIN_DIGEST_JSON_BYTES);
        for limits in [small_codec(), FormatLimits::default()] {
            assert_eq!(
                blocks::parse(block(raw.clone()), limits).err(),
                Some(StorageError::Invalid(expected))
            );
        }
    }
    let malformed = br#"{"kind":"x","schema":1,"blocks":[],"blobs":[],"value":{"x":1,"\u0078":2}}"#;
    let mut wrong_hash = block(malformed.to_vec());
    wrong_hash.bytes.push(b' ');
    assert_eq!(
        blocks::parse(wrong_hash, small_codec()).err(),
        Some(StorageError::Invalid("block hash"))
    );
    assert_eq!(
        blocks::parse(block(malformed.to_vec()), small_codec()).err(),
        Some(StorageError::Codec(CodecError::InvalidJson))
    );
}

#[test]
fn escaped_digest_and_structure_boundaries_precede_schema_validation() {
    let raw = format!(
        r#"{{"kind":"x","schema":1,"blocks":["\u0030{}"],"blobs":[],"value":{{"a":[0]}}}}"#,
        "0".repeat(63)
    );
    let invalid_schema = raw.replace(r#""schema":1"#, r#""schema":2"#);
    for items in [read::METADATA_ITEMS, FormatLimits::default().items] {
        let exact = FormatLimits {
            items,
            json_values: 9,
            json_depth: 3,
            ..small_codec()
        };
        let row = blocks::parse(block(raw.as_bytes().to_vec()), exact)
            .unwrap()
            .row;
        assert_eq!(row.refs[0].bytes(), [0; 32]);
        assert_eq!(row.refs.capacity(), 1);
        for short in [
            FormatLimits {
                json_values: 8,
                ..exact
            },
            FormatLimits {
                json_depth: 2,
                ..exact
            },
        ] {
            assert_eq!(
                blocks::parse(block(invalid_schema.as_bytes().to_vec()), short).err(),
                Some(StorageError::Codec(CodecError::ResourceLimit(
                    "json_structure"
                )))
            );
        }
        assert_eq!(
            blocks::parse(block(invalid_schema.as_bytes().to_vec()), exact).err(),
            Some(StorageError::Unsupported("block schema"))
        );
    }
}

#[test]
fn failed_first_decode_after_valid_sqlite_rows_releases_original_leases_on_each_open() {
    let (dir, _driver, file) = fixture();
    let seq = file.cache.blocks.len() as u64 + 1;
    assert!(seq > 2);
    let hashes = ["00".repeat(32), "ff".repeat(32)];
    let raw = serde_json::to_vec(&json!({"kind":"fixture","schema":1,"blocks":hashes,
        "blobs":[hashes[1],hashes[0],"GG"],"value":{}}))
    .unwrap();
    // Append to this test database after all healthy blocks. The digest
    // conversion failure precedes the deliberately mismatched SQL kind.
    file.conn
        .execute(
            "INSERT INTO blocks VALUES(?1,?2,?3,'other',1,NULL)",
            params![sha(&raw), bu64(seq), raw],
        )
        .unwrap();
    drop(file);
    let before = digest(&dir.file());
    let store = BlobStore::default();
    let limits = FileLimits {
        codec: small_codec(),
        metadata_bytes: 1024 * 1024,
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
        assert_eq!(result.err(), Some(StorageError::Invalid("hex")));
        let usage = store.usage().unwrap();
        assert!(usage.peak_bytes > 0);
        assert_eq!(usage.bytes, 0);
        assert_eq!(digest(&dir.file()), before);
    }
}
