use super::*;
use rusqlite::types::Value as SqlValue;

#[test]
fn block_row_header_sequence_borrows_eight_bytes_and_keeps_big_endian_values() {
    let c = Connection::open_in_memory().unwrap();
    let mut statement = c
        .prepare(
            "SELECT zeroblob(0),zeroblob(7),x'0102030405060708',zeroblob(9),
        zeroblob(524288),'12345678',CAST(x'ff' AS TEXT),NULL,1,0.5,x'ffffffffffffffff'",
        )
        .unwrap();
    let mut rows = statement.query([]).unwrap();
    let row = rows.next().unwrap().unwrap();
    for i in 0..11 {
        let old = row
            .get::<_, Vec<u8>>(i)
            .map_err(StorageError::from)
            .and_then(u64blob);
        let borrowed = fixed_blob_column::<8>(row, i);
        if let Ok(bytes) = borrowed {
            assert_eq!(
                bytes.as_ptr(),
                row.get_ref(i).unwrap().as_blob().unwrap().as_ptr()
            );
        }
        assert_eq!(borrowed.map(|bytes| u64::from_be_bytes(*bytes)), old);
    }
    assert_eq!(
        u64::from_be_bytes(*fixed_blob_column(row, 2).unwrap()),
        0x0102_0304_0506_0708
    );
    assert_eq!(
        u64::from_be_bytes(*fixed_blob_column(row, 10).unwrap()),
        u64::MAX
    );
}

#[test]
fn borrowed_columns_match_native_text_conversion_and_use_sqlite_bytes() {
    let c = Connection::open_in_memory().unwrap();
    let mut q = c
        .prepare("SELECT ?1,CAST(?2 AS TEXT),NULL,x'4100',3,0.5,''")
        .unwrap();
    let long = "x\0雪".repeat(60_000);
    let mut invalid = long.as_bytes().to_vec();
    invalid.push(255);
    let mut rows = q.query(params![long, invalid]).unwrap();
    let r = rows.next().unwrap().unwrap();
    for i in 0..7 {
        for nullable in [false, true] {
            let original = if nullable {
                r.get::<_, Option<String>>(i)
            } else {
                r.get::<_, String>(i).map(Some)
            }
            .map_err(StorageError::from);
            let borrowed = text_column(r, i, nullable);
            assert_eq!(
                borrowed.as_ref().copied(),
                original.as_ref().map(|s| s.as_deref())
            );
            if let Ok(Some(s)) = borrowed {
                assert_eq!(s.as_ptr(), r.get_ref(i).unwrap().as_str().unwrap().as_ptr());
            }
        }
    }
}

fn check_columns(
    conn: &Connection,
    kind: &str,
    body: Value,
    stored_kind: SqlValue,
    schema: SqlValue,
    stored_id: SqlValue,
) -> Result<(), StorageError> {
    let bytes = serde_json::to_vec(
        &json!({"kind":kind, "schema":1, "blocks":[], "blobs":[], "value":body}),
    )
    .unwrap();
    let parsed = blocks::parse(
        StateBlock {
            hash: StateHash::from_bytes(sha(&bytes)),
            bytes,
        },
        FormatLimits {
            items: METADATA_ITEMS,
            ..FormatLimits::default()
        },
    )?;
    let mut q = conn.prepare("SELECT NULL,NULL,NULL,NULL,?1,?2,?3")?;
    let mut rows = q.query(params![stored_kind, schema, stored_id])?;
    verify(rows.next()?.unwrap(), parsed).map(|_| ())
}

#[test]
fn schema_and_project_id_errors_keep_their_place_among_column_checks() {
    let c = Connection::open_in_memory().unwrap();
    let kind = "skin-editor.project";
    let check = |body, k, s, id| check_columns(&c, kind, body, k, s, id);
    let stored_kind = || SqlValue::Text(kind.into());
    let bad = || SqlValue::Blob(vec![1]);
    let ok_schema = SqlValue::Integer(1);
    assert_eq!(
        check(json!({}), SqlValue::Text("different".into()), bad(), bad()),
        Err(StorageError::Invalid("block derived fields"))
    );
    assert_eq!(
        check(json!({}), stored_kind(), bad(), bad()),
        Err(StorageError::Sql {
            code: None,
            class: "column type"
        })
    );
    assert_eq!(
        check(json!({"id":{}}), stored_kind(), SqlValue::Integer(2), bad()),
        Err(StorageError::Invalid("block derived fields"))
    );
    assert_eq!(
        check(json!({}), stored_kind(), ok_schema.clone(), bad()),
        Err(StorageError::Invalid("id"))
    );
    assert_eq!(
        check(json!({"id":[0]}), stored_kind(), ok_schema.clone(), bad()),
        Err(StorageError::Invalid("text"))
    );
    assert_eq!(
        check(json!({"id":"a"}), stored_kind(), ok_schema.clone(), bad()),
        Err(StorageError::Sql {
            code: None,
            class: "column type"
        })
    );
    assert_eq!(
        check(
            json!({"id":"a"}),
            stored_kind(),
            ok_schema.clone(),
            SqlValue::Null
        ),
        Err(StorageError::Invalid("block project"))
    );
    let id = "é\0雪\n".repeat(2000);
    assert_eq!(
        check(
            json!({"id":id, "other":{"id":0}}),
            stored_kind(),
            ok_schema.clone(),
            SqlValue::Text(id.clone())
        ),
        Ok(())
    );
    assert_eq!(
        check(
            json!({"id":id}),
            stored_kind(),
            ok_schema.clone(),
            SqlValue::Text(id.trim_end().into())
        ),
        Err(StorageError::Invalid("block project"))
    );
    assert_eq!(
        check_columns(
            &c,
            "fixture",
            json!({"id":{}}),
            SqlValue::Text("fixture".into()),
            ok_schema,
            SqlValue::Null
        ),
        Ok(())
    );
}
