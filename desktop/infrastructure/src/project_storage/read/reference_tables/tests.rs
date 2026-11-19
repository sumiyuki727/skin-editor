use super::*;
use rusqlite::types::Value as SqlValue;

#[test]
fn borrowed_digest_preserves_native_blob_errors_and_uses_the_current_row() {
    let c = Connection::open_in_memory().unwrap();
    let mut q = c
        .prepare(
            "SELECT zeroblob(0),zeroblob(31),?1,zeroblob(33),zeroblob(524288),
        '0123456789abcdef0123456789abcdef',CAST(x'ff' AS TEXT),NULL,32,0.5",
        )
        .unwrap();
    let mut rows = q.query([[0xa5_u8; 32]]).unwrap();
    let r = rows.next().unwrap().unwrap();
    for i in 0..10 {
        let old = r
            .get::<_, Vec<u8>>(i)
            .map_err(StorageError::from)
            .and_then(fixed::<32>);
        let borrowed = fixed_blob_column::<32>(r, i);
        assert_eq!(borrowed.as_ref().copied(), old.as_ref());
        if let Ok(bytes) = borrowed {
            assert_eq!(
                bytes.as_ptr(),
                r.get_ref(i).unwrap().as_blob().unwrap().as_ptr()
            );
        }
    }
}

fn expected(with_blocks: bool, with_blobs: bool) -> BTreeMap<StateHash, blocks::Row> {
    let hashes = |enabled| if enabled { vec![hex(&[2; 32])] } else { vec![] };
    let bytes = serde_json::to_vec(
        &json!({"kind":"fixture","schema":1,"blocks":hashes(with_blocks),
        "blobs":hashes(with_blobs),"value":{}}),
    )
    .unwrap();
    let key = StateHash::from_bytes(sha(&bytes));
    let row = blocks::parse(StateBlock { hash: key, bytes }, FormatLimits::default())
        .unwrap()
        .row;
    BTreeMap::from([(key, row)])
}

fn tables() -> Connection {
    let c = Connection::open_in_memory().unwrap();
    c.execute_batch("CREATE TABLE block_refs(owner,target); CREATE TABLE blob_refs(owner,target)")
        .unwrap();
    c
}

#[test]
fn both_reference_tables_keep_column_short_circuits_and_tail_errors() {
    for (block, table, missing, mismatch, extra) in [
        (
            true,
            "block_refs",
            "missing block index",
            "block index",
            "extra block index",
        ),
        (
            false,
            "blob_refs",
            "missing blob index",
            "blob index",
            "extra blob index",
        ),
    ] {
        let cache = expected(block, !block);
        let owner = SqlValue::Blob(cache.first_key_value().unwrap().0.bytes().to_vec());
        let target = SqlValue::Blob(vec![2; 32]);
        let wrong = SqlValue::Blob(vec![7; 32]);
        let cancel = CaptureCancellation::default();
        let c = tables();
        let insert = format!("INSERT INTO {table} VALUES(?1,?2)");
        assert_eq!(
            verify(&c, &cache, &cancel),
            Err(StorageError::Invalid(missing))
        );
        for (pair, error) in [
            (
                (SqlValue::Null, SqlValue::Null),
                StorageError::Sql {
                    code: None,
                    class: "column type",
                },
            ),
            (
                (SqlValue::Blob(vec![1; 31]), SqlValue::Null),
                StorageError::Invalid("fixed bytes"),
            ),
            (
                (wrong.clone(), SqlValue::Null),
                StorageError::Invalid(mismatch),
            ),
            (
                (owner.clone(), SqlValue::Text("x".into())),
                StorageError::Sql {
                    code: None,
                    class: "column type",
                },
            ),
            (
                (owner.clone(), SqlValue::Blob(vec![2; 33])),
                StorageError::Invalid("fixed bytes"),
            ),
            ((owner.clone(), wrong), StorageError::Invalid(mismatch)),
        ] {
            c.execute(&insert, params![pair.0, pair.1]).unwrap();
            assert_eq!(verify(&c, &cache, &cancel), Err(error));
            c.execute(&format!("DELETE FROM {table}"), []).unwrap();
        }
        c.execute(&insert, params![owner, target]).unwrap();
        verify(&c, &cache, &cancel).unwrap();
        // Sort after every valid owner. Neither malformed column is decoded.
        c.execute(&insert, params![vec![255u8; 33], SqlValue::Null])
            .unwrap();
        assert_eq!(
            verify(&c, &cache, &cancel),
            Err(StorageError::Invalid(extra))
        );
        c.close().unwrap();
    }
}

#[test]
fn shared_targets_are_checked_for_each_owner_through_the_last_digest_byte() {
    let mut both = expected(true, true);
    let mut block_only = expected(true, false);
    let mut blob_only = expected(false, true);
    let key = |map: &BTreeMap<StateHash, blocks::Row>| map.first_key_value().unwrap().0.bytes();
    let shared = key(&both);
    let block_owner = key(&block_only);
    let blob_owner = key(&blob_only);
    both.append(&mut block_only);
    both.append(&mut blob_only);
    let c = tables();
    let cancel = CaptureCancellation::default();
    let target = [2u8; 32];
    for (table, other) in [("block_refs", block_owner), ("blob_refs", blob_owner)] {
        let insert = format!("INSERT INTO {table} VALUES(?1,?2)");
        c.execute(&insert, params![shared.max(other), target])
            .unwrap();
        c.execute(&insert, params![shared.min(other), target])
            .unwrap();
    }
    verify(&c, &both, &cancel).unwrap();
    let mut wrong = target;
    wrong[31] ^= 1;
    for (table, last, error) in [
        ("blob_refs", shared.max(blob_owner), "blob index"),
        ("block_refs", shared.max(block_owner), "block index"),
    ] {
        c.execute(
            &format!("UPDATE {table} SET target=?1 WHERE owner=?2"),
            params![wrong, last],
        )
        .unwrap();
        assert_eq!(
            verify(&c, &both, &cancel),
            Err(StorageError::Invalid(error))
        );
    }
}

#[test]
fn block_refs_finish_before_blob_refs_and_cancellation_precedes_expected_rows_only() {
    let c = tables();
    let both = expected(true, true);
    c.execute_batch("DROP TABLE blob_refs").unwrap();
    assert_eq!(
        verify(&c, &both, &CaptureCancellation::default()),
        Err(StorageError::Invalid("missing block index"))
    );
    let cancel = CaptureCancellation::default();
    cancel.cancel();
    assert_eq!(verify(&c, &both, &cancel), Err(StorageError::Cancelled));
    c.execute_batch("CREATE TABLE blob_refs(owner,target)")
        .unwrap();
    assert_eq!(
        verify(&c, &expected(false, true), &cancel),
        Err(StorageError::Cancelled)
    );
    let empty = BTreeMap::new();
    verify(&c, &empty, &cancel).unwrap();
    c.execute_batch("INSERT INTO blob_refs VALUES(NULL,NULL)")
        .unwrap();
    assert_eq!(
        verify(&c, &empty, &cancel),
        Err(StorageError::Invalid("extra blob index"))
    );
    c.execute_batch("INSERT INTO block_refs VALUES(NULL,NULL)")
        .unwrap();
    assert_eq!(
        verify(&c, &empty, &cancel),
        Err(StorageError::Invalid("extra block index"))
    );
    c.close().unwrap();
}
