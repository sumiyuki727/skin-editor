use super::*;

const INDEPENDENT: &[u8] = b"independent descriptor lease";

fn descriptor(
    c: &Connection,
    sql: &str,
    params: impl rusqlite::Params,
    prior: u64,
    limit: u64,
) -> Result<(BlobRef, u64), StorageError> {
    let mut statement = c.prepare(sql)?;
    let mut rows = statement.query(params)?;
    read::original_descriptor(c, rows.next()?.unwrap(), prior, limit)
}

#[test]
fn hash_length_and_total_fail_in_order_before_the_chunk_query() {
    // No chunk table: an early count query would itself fail.
    let c = Connection::open_in_memory().unwrap();
    let column_type = || StorageError::Sql {
        code: None,
        class: "column type",
    };
    for (sql, prior, limit, expected) in [
        ("SELECT 'not a blob',-1", 0, 0, column_type()),
        ("SELECT NULL,-1", 0, 0, column_type()),
        (
            "SELECT zeroblob(0),NULL",
            0,
            0,
            StorageError::Invalid("fixed bytes"),
        ),
        (
            "SELECT zeroblob(31),'invalid'",
            0,
            0,
            StorageError::Invalid("fixed bytes"),
        ),
        (
            "SELECT zeroblob(33),-1",
            0,
            0,
            StorageError::Invalid("fixed bytes"),
        ),
        (
            "SELECT zeroblob(524288),1",
            0,
            0,
            StorageError::Invalid("fixed bytes"),
        ),
        ("SELECT zeroblob(32),'1'", 0, 0, column_type()),
        ("SELECT zeroblob(32),1.5", 0, 0, column_type()),
        ("SELECT zeroblob(32),NULL", 0, 0, column_type()),
        (
            "SELECT zeroblob(32),9223372036854775807",
            0,
            262144,
            StorageError::Limit("original bytes"),
        ),
        (
            "SELECT zeroblob(32),-1",
            u64::MAX,
            0,
            StorageError::Invalid("blob length"),
        ),
        (
            "SELECT zeroblob(32),1",
            7,
            7,
            StorageError::Limit("original bytes"),
        ),
        (
            "SELECT zeroblob(32),1",
            u64::MAX,
            u64::MAX,
            StorageError::Limit("original bytes"),
        ),
    ] {
        assert_eq!(
            descriptor(&c, sql, [], prior, limit),
            Err(expected),
            "{sql}"
        );
    }
}

#[test]
fn exact_budget_and_chunk_boundaries_require_only_matching_chunk_counts() {
    let c = Connection::open_in_memory().unwrap();
    // No payload or part column is available. Other originals cannot supply
    // this descriptor's chunks.
    c.execute_batch("CREATE TABLE blob_chunks(sha); INSERT INTO blob_chunks VALUES(zeroblob(32));")
        .unwrap();
    let digest = std::array::from_fn::<_, 32, _>(|i| i as u8);
    for chunk_count in 0..=2 {
        if chunk_count > 0 {
            c.execute("INSERT INTO blob_chunks VALUES(?1)", [digest])
                .unwrap();
        }
        for (len, required) in [
            (0, 0),
            (1, 1),
            (65535, 1),
            (65536, 1),
            (65537, 2),
            (131072, 2),
            (131073, 3),
        ] {
            let total = 9 + len;
            let got = descriptor(&c, "SELECT ?1,?2", params![digest, len as i64], 9, total);
            if chunk_count == required {
                assert_eq!(got, Ok((BlobRef::new(digest, len), total)));
            } else {
                assert_eq!(got, Err(StorageError::Invalid("chunk count")));
            }
        }
    }
    #[cfg(target_pointer_width = "64")]
    {
        assert_eq!(size_of::<BlobRef>(), 40);
        assert_eq!(size_of::<(BlobRef, u64)>(), 48);
    }
}

fn assert_failed_open(
    path: &Path,
    limits: FileLimits,
    expected: StorageError,
    acquired_peak_bytes: usize,
) {
    let before = digest(path);
    let marker = INDEPENDENT.to_vec();
    let budget = marker.capacity() + acquired_peak_bytes;
    let store = BlobStore::with_limits(budget, budget).unwrap();
    let independent = store.stage_bytes(marker).unwrap();
    let baseline = store.usage().unwrap();
    for _ in 0..2 {
        let error = SqliteProjectFile::load(
            path,
            store.clone(),
            limits,
            RestoreSelection::Manual,
            CaptureCancellation::default(),
        )
        .err();
        assert_eq!(error.as_ref(), Some(&expected));
        let usage = store.usage().unwrap();
        assert_eq!(usage.peak_bytes, baseline.bytes + acquired_peak_bytes);
        assert_eq!(
            (usage.bytes, usage.entries),
            (baseline.bytes, baseline.entries)
        );
        assert!(
            store
                .get(independent.reference())
                .unwrap()
                .same_allocation(&independent)
        );
        assert_eq!(digest(path), before);
    }
    drop(independent);
    assert_eq!(store.usage().unwrap().bytes, 0);
}

#[test]
fn table_limit_and_sql_hash_type_reject_before_any_original_is_acquired() {
    let (dir, _driver, file) = fixture();
    // SQLite counts characters for this TEXT primary key and sorts it before
    // the fixture's BLOB key. It satisfies the real schema's length check.
    file.conn
        .execute(
            "INSERT INTO blobs(sha,byte_len) VALUES(?1,3)",
            ["雪".repeat(32)],
        )
        .unwrap();
    drop(file);
    let limits = FileFactsLimits::default().file();
    assert_failed_open(
        &dir.file(),
        FileLimits {
            originals: 1,
            ..limits
        },
        StorageError::Limit("table rows"),
        0,
    );
    assert_failed_open(
        &dir.file(),
        limits,
        StorageError::Sql {
            code: None,
            class: "column type",
        },
        0,
    );
}

#[test]
fn later_descriptor_rejections_release_the_prefix_and_preserve_independent_leases() {
    let (dir, _driver, file) = fixture();
    assert_eq!(file.cache.originals.len(), 1);
    let first = file.cache.originals.first().unwrap();
    let later = [0xff; 32];
    assert!(first.sha256() < later);
    let prefix = usize::try_from(first.byte_len()).unwrap();
    let payload = [0x61_u8; 128];
    assert!(INDEPENDENT.len() + prefix < payload.len());
    file.conn
        .execute(
            "INSERT INTO blobs(sha,byte_len) VALUES(?1,?2)",
            params![later, payload.len() as i64],
        )
        .unwrap();
    drop(file);
    let limits = FileLimits {
        original_bytes: (prefix + payload.len()) as u64,
        ..FileFactsLimits::default().file()
    };
    assert_failed_open(
        &dir.file(),
        FileLimits {
            original_bytes: limits.original_bytes - 1,
            ..limits
        },
        StorageError::Limit("original bytes"),
        prefix,
    );
    assert_failed_open(
        &dir.file(),
        limits,
        StorageError::Invalid("chunk count"),
        prefix,
    );
    // Once the count is correct the pool's admission error becomes visible.
    // With room for the payload its wrong digest rejects the acquired lease.
    Connection::open(dir.file())
        .unwrap()
        .execute(
            "INSERT INTO blob_chunks(sha,part,bytes) VALUES(?1,0,?2)",
            params![later, payload],
        )
        .unwrap();
    assert_failed_open(
        &dir.file(),
        limits,
        StorageError::Limit("original admission"),
        prefix,
    );
    assert_failed_open(
        &dir.file(),
        limits,
        StorageError::Invalid("original digest"),
        prefix + payload.len(),
    );
}
