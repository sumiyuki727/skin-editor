use super::*;

fn database() -> Connection {
    let c = Connection::open_in_memory().unwrap();
    c.execute_batch(&schema::script()).unwrap();
    c
}
#[test]
fn raw_preflight_has_fixed_order_and_empty_aggregates_without_claiming_a_valid_file() {
    let c = database();
    let value = support::preflight(&c, FileFactsLimits::default()).unwrap();
    assert_eq!(value.version, FILE_READ_PREFLIGHT_VERSION);
    assert!(value.catalog.rows > 16 && value.catalog.rows <= 128);
    assert_eq!(value.tables, [FileReadAggregate::default(); 16]);
    // Empty tables do not form a healthy archive; preflight is only admission.
    assert!(
        read::full(
            &mut { c },
            &BlobStore::default(),
            FileLimits::default(),
            RestoreSelection::Latest,
            &CaptureCancellation::default()
        )
        .is_err()
    );
}
#[test]
fn raw_preflight_counts_utf8_and_embedded_nul_bytes() {
    let c = database();
    c.execute(
        "INSERT INTO features VALUES(?1,?2,0)",
        params!["雪\0x", "β"],
    )
    .unwrap();
    let value = support::preflight(&c, FileFactsLimits::default()).unwrap();
    assert_eq!(
        value.tables[1],
        FileReadAggregate {
            rows: 1,
            bytes: 7,
            largest: 7
        }
    );
}
#[test]
fn non_utf8_encoding_is_explicitly_unsupported_without_changing_the_database() {
    let c = Connection::open_in_memory().unwrap();
    c.execute_batch("PRAGMA encoding='UTF-16le'; CREATE TABLE marker(x TEXT);")
        .unwrap();
    assert_eq!(
        support::preflight(&c, FileFactsLimits::default()),
        Err(FileFactsError::Unsupported)
    );
    assert_eq!(
        c.query_row("PRAGMA encoding", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "UTF-16le"
    );
}
#[test]
fn large_catalog_is_rejected_before_full_catalog_copy() {
    let c = database();
    let name = "雪".repeat(22000);
    c.execute_batch(&format!("CREATE INDEX \"{name}\" ON nodes(id);"))
        .unwrap();
    assert_eq!(
        support::preflight(&c, FileFactsLimits::default()),
        Err(FileFactsError::Limit(FileFactsLimit::Validation))
    );
    assert_eq!(
        c.query_row(
            "SELECT count(*) FROM sqlite_schema WHERE name=?1",
            [&name],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        1
    );
}
#[test]
fn raw_size_limit_boundary_overflow_and_non_integer_declarations_fail_closed() {
    for (value, second, expected) in [
        (17i64, false, None),
        (
            18,
            false,
            Some(FileFactsError::Limit(FileFactsLimit::Validation)),
        ),
        (i64::MAX, true, Some(FileFactsError::ValidationFailed)),
    ] {
        let c = database();
        c.execute(
            "INSERT INTO blobs VALUES(?1,?2)",
            params![[1u8; 32].as_slice(), value],
        )
        .unwrap();
        if second {
            c.execute(
                "INSERT INTO blobs VALUES(?1,?2)",
                params![[2u8; 32].as_slice(), value],
            )
            .unwrap();
        }
        let limits = FileFactsLimits {
            original_bytes: 17,
            ..FileFactsLimits::default()
        };
        let result = support::preflight(&c, limits);
        if let Some(error) = expected {
            assert_eq!(result, Err(error));
        } else {
            assert_eq!(result.unwrap().tables[2].bytes, 17);
        }
    }
    let c = database();
    c.execute(
        "INSERT INTO blobs VALUES(?1,?2)",
        params![[1u8; 32].as_slice(), 0.5f64],
    )
    .unwrap();
    assert_eq!(
        support::preflight(&c, FileFactsLimits::default()),
        Err(FileFactsError::ValidationFailed)
    );
}
#[test]
fn oversized_text_and_actual_lowered_count_limits_cannot_pass_preflight() {
    let c = database();
    c.execute("INSERT INTO features VALUES(?1,'1',0)", ["x".repeat(4097)])
        .unwrap();
    assert_eq!(
        support::preflight(&c, FileFactsLimits::default()),
        Err(FileFactsError::Limit(FileFactsLimit::Validation))
    );
    let c = database();
    for n in 1..=2 {
        c.execute(
            "INSERT INTO blobs VALUES(?1,0)",
            params![[n as u8; 32].as_slice()],
        )
        .unwrap();
    }
    assert_eq!(
        support::preflight(
            &c,
            FileFactsLimits {
                originals: 1,
                ..FileFactsLimits::default()
            }
        ),
        Err(FileFactsError::Limit(FileFactsLimit::Validation))
    );
}

#[test]
fn declared_blob_columns_still_count_actual_utf8_text_and_nul_bytes() {
    let c = database();
    c.execute(
        "INSERT INTO blobs VALUES(?1,5)",
        params![[1u8; 32].as_slice()],
    )
    .unwrap();
    c.execute(
        "INSERT INTO blob_chunks VALUES(?1,0,?2)",
        params![[1u8; 32].as_slice(), "雪\0x"],
    )
    .unwrap();
    c.execute(
        "INSERT INTO blocks VALUES(?1,?2,x'','unused',1,NULL)",
        params![[2u8; 32].as_slice(), 1u64.to_be_bytes().as_slice()],
    )
    .unwrap();
    c.execute(
        "INSERT INTO nodes VALUES(?1,NULL,?2,0,?3,?4,2)",
        params![
            [3u8; 16].as_slice(),
            [2u8; 32].as_slice(),
            "雪\0x",
            [4u8; 32].as_slice()
        ],
    )
    .unwrap();
    let p = support::preflight(&c, FileFactsLimits::default()).unwrap();
    assert_eq!(p.tables[3].bytes, 5);
    assert_eq!(p.tables[7].bytes, 5);
}
#[test]
fn sqlite_interrupts_at_early_and_later_steps_report_cancellation() {
    for threshold in [1, 1000] {
        let mut c = database();
        let cancel = CaptureCancellation::default();
        let trigger = cancel.clone();
        let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let counter = calls.clone();
        c.progress_handler(
            1,
            Some(move || {
                if counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed) >= threshold {
                    trigger.cancel();
                    true
                } else {
                    false
                }
            }),
        )
        .unwrap();
        assert_eq!(
            support::preflight_cancel(&c, &cancel),
            Err(FileFactsError::Cancelled)
        );
        assert!(cancel.is_cancelled());
        assert!(calls.load(std::sync::atomic::Ordering::Relaxed) > threshold);
    }
}

#[cfg(windows)]
#[test]
fn rejected_real_file_preflight_keeps_source_and_cleans_only_its_owned_scratch() {
    for fail_cleanup in [false, true] {
        let (dir, _d, req) = golden();
        let c = Connection::open(dir.file()).unwrap();
        c.execute_batch(&format!(
            "CREATE INDEX \"{}\" ON nodes(id)",
            "雪".repeat(22000)
        ))
        .unwrap();
        drop(c);
        let before = std::fs::read(dir.file()).unwrap();
        let scratch = Directory::new();
        let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
        let baseline = reader.usage().managed_bytes;
        support::cleanup(&mut reader, fail_cleanup);
        let error = reader
            .inspect(&dir.file(), req, CaptureCancellation::default())
            .err()
            .unwrap();
        if fail_cleanup {
            assert!(matches!(error, FileFactsError::Cleanup { .. }));
            assert_eq!(reader.usage().scratch_reserved_bytes, before.len() as u64);
            assert!(reader.pending_cleanup().is_some());
            support::cleanup(&mut reader, false);
            reader.retry_cleanup().unwrap();
        } else {
            assert_eq!(error, FileFactsError::Limit(FileFactsLimit::Validation));
        }
        assert_eq!(reader.usage().managed_bytes, baseline);
        assert_eq!(reader.usage().scratch_reserved_bytes, 0);
        assert_eq!(std::fs::read(dir.file()).unwrap(), before);
        assert_eq!(std::fs::read_dir(&scratch.path).unwrap().count(), 1);
    }
}
