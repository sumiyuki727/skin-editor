use super::*;

#[test]
fn missing_last_blob_reference_releases_opening_leases_and_preserves_the_file() {
    let (dir, _driver, mut file) = fixture();
    let tx = file.conn.transaction().unwrap();
    let guard: String = tx
        .query_row(
            "SELECT sql FROM sqlite_schema WHERE name='guard_blob_refs_DELETE'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    tx.execute_batch("DROP TRIGGER guard_blob_refs_DELETE")
        .unwrap();
    let deleted = tx
        .execute(
            "DELETE FROM blob_refs WHERE (owner,target) =
        (SELECT owner,target FROM blob_refs ORDER BY owner DESC,target DESC LIMIT 1)",
            [],
        )
        .unwrap();
    assert_eq!(deleted, 1);
    tx.execute_batch(&guard).unwrap();
    tx.commit().unwrap();
    drop(file);
    let before = digest(&dir.file());
    let store = BlobStore::default();
    let independent = store.stage_bytes(b"separate lease".to_vec()).unwrap();
    let baseline = store.usage().unwrap().bytes;
    for _ in 0..2 {
        let result = SqliteProjectFile::load(
            dir.file(),
            store.clone(),
            FileFactsLimits::default().file(),
            RestoreSelection::Manual,
            CaptureCancellation::default(),
        );
        assert_eq!(
            result.err(),
            Some(StorageError::Invalid("missing blob index"))
        );
        let usage = store.usage().unwrap();
        assert!(usage.peak_bytes > baseline);
        assert_eq!(usage.bytes, baseline);
        assert_eq!(digest(&dir.file()), before);
    }
    drop(independent);
    assert_eq!(store.usage().unwrap().bytes, 0);
}
