use super::*;

#[test]
fn failed_opens_release_their_originals_at_each_control_validation_stage() {
    let (dir, _driver, file) = fixture();
    drop(file);
    for (sql, expected, acquired_originals) in [
        (
            "UPDATE control SET generation=zeroblob(8)",
            StorageError::Invalid("control"),
            true,
        ),
        (
            "INSERT INTO blocks VALUES(zeroblob(32),x'ffffffffffffffff',x'21','fixture',1,NULL)",
            StorageError::Invalid("block sequence"),
            true,
        ),
        (
            "UPDATE control SET save_id=NULL",
            StorageError::Sql {
                code: None,
                class: "column type",
            },
            false,
        ),
    ] {
        // Faults accumulate. The malformed block wins over the delayed
        // generation mismatch, then an SQL type error wins over both.
        Connection::open(dir.file())
            .unwrap()
            .execute_batch(sql)
            .unwrap();
        let before = digest(&dir.file());
        let store = BlobStore::default();
        let independent = store
            .stage_bytes(b"independent control lease".to_vec())
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
            assert_eq!(usage.peak_bytes > baseline, acquired_originals);
            assert_eq!(usage.bytes, baseline);
            assert_eq!(digest(&dir.file()), before);
        }
        drop(independent);
        assert_eq!(store.usage().unwrap().bytes, 0);
    }
}
