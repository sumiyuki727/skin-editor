use super::*;

// Loose columns exercise widths and storage classes rejected by the file
// schema. The production reader still processes all eighteen columns.
fn tables() -> Connection {
    let c = Connection::open_in_memory().unwrap();
    c.execute_batch(
        "CREATE TABLE header(family,profile,container_version,project_schema,history_schema,
                             project_id,history_id,lineage_id,root_id);
         CREATE TABLE control(generation,save_id,manual_id,protected_id,node_count,block_count,
                              blob_bytes,metadata_bytes,address_epoch);",
    )
    .unwrap();
    c.execute(
        "INSERT INTO header VALUES('skin-editor-project',?1,2,1,2,?2,?2,?2,?2)",
        params![FILE_PROFILE, 1u128.to_be_bytes()],
    )
    .unwrap();
    c.execute(
        "INSERT INTO control VALUES(?1,?2,NULL,?2,1,2,3,4,?2)",
        params![7u64.to_be_bytes(), 1u128.to_be_bytes()],
    )
    .unwrap();
    c
}

fn column_type() -> StorageError {
    StorageError::Sql {
        code: None,
        class: "column type",
    }
}

#[test]
fn type_errors_and_profile_precede_deferred_identity_checks() {
    for (sql, expected) in [
        ("UPDATE header SET family='x', root_id=0", column_type()),
        (
            "UPDATE header SET family='x', profile=CAST(x'ff' AS TEXT)",
            StorageError::Sql {
                code: None,
                class: "sqlite",
            },
        ),
        ("UPDATE header SET profile=zeroblob(524288)", column_type()),
        (
            "UPDATE header SET family=hex(zeroblob(262144)), history_id=x''; UPDATE control SET save_id=NULL",
            StorageError::Unsupported("profile/schema"),
        ),
        (
            "UPDATE header SET history_id=x''; UPDATE control SET save_id=NULL",
            column_type(),
        ),
        (
            "UPDATE header SET root_id=x''; UPDATE control SET protected_id=NULL",
            column_type(),
        ),
        (
            "UPDATE control SET generation=x'', manual_id=0",
            column_type(),
        ),
        (
            "UPDATE header SET history_id=zeroblob(16), lineage_id=x''",
            StorageError::Invalid("history id"),
        ),
        (
            "UPDATE header SET lineage_id=zeroblob(16), project_id=x''; UPDATE control SET address_epoch=x''",
            StorageError::Invalid("lineage"),
        ),
        (
            "UPDATE control SET address_epoch=zeroblob(16); UPDATE header SET project_id=x''",
            StorageError::Invalid("address epoch"),
        ),
        (
            "UPDATE header SET project_id=zeroblob(16), root_id=x''",
            StorageError::Invalid("project"),
        ),
        (
            "UPDATE header SET root_id=zeroblob(16)",
            StorageError::Invalid("node id"),
        ),
        (
            "UPDATE header SET history_id=zeroblob(524288)",
            StorageError::Invalid("fixed bytes"),
        ),
    ] {
        let c = tables();
        c.execute_batch(sql).unwrap();
        assert_eq!(read(&c).err(), Some(expected), "{sql}");
    }
}

#[test]
fn late_control_checks_keep_order_nullability_and_big_endian_values() {
    let commit = CommitId::new(1).unwrap();
    for (sql, manual, expected) in [
        ("", None, Ok(true)),
        ("", Some(commit), Ok(false)),
        (
            "UPDATE control SET manual_id=save_id",
            Some(commit),
            Ok(true),
        ),
        (
            "UPDATE control SET generation=x'', save_id=zeroblob(16)",
            None,
            Err(StorageError::Invalid("fixed bytes")),
        ),
        (
            "UPDATE control SET generation=x'0700000000000000', save_id=x''",
            None,
            Ok(false),
        ),
        (
            "UPDATE control SET save_id=zeroblob(16), manual_id=x''",
            None,
            Err(StorageError::Invalid("commit id")),
        ),
        (
            "UPDATE control SET save_id=zeroblob(524288)",
            None,
            Err(StorageError::Invalid("fixed bytes")),
        ),
        (
            "UPDATE control SET save_id=x'00000000000000000000000000000002', manual_id=x''",
            None,
            Ok(false),
        ),
        (
            "UPDATE control SET manual_id=zeroblob(16), protected_id=x''",
            None,
            Err(StorageError::Invalid("commit id")),
        ),
        (
            "UPDATE control SET manual_id=x''",
            None,
            Err(StorageError::Invalid("fixed bytes")),
        ),
        (
            "UPDATE control SET manual_id=save_id, protected_id=x''",
            None,
            Ok(false),
        ),
        (
            "UPDATE control SET protected_id=zeroblob(16)",
            None,
            Err(StorageError::Invalid("commit id")),
        ),
        (
            "UPDATE control SET protected_id=zeroblob(17)",
            None,
            Err(StorageError::Invalid("fixed bytes")),
        ),
    ] {
        let c = tables();
        c.execute_batch(sql).unwrap();
        let (_, control) = read(&c).unwrap();
        c.close().unwrap();
        assert_eq!(
            control.matches_commits(7, commit, manual),
            expected,
            "{sql}"
        );
    }
}

#[test]
fn deferred_widths_match_owned_conversions_after_the_row_closes() {
    let c = Connection::open_in_memory().unwrap();
    for width in [0, 7, 8, 9, 15, 16, 17, 524288] {
        let (old, seq, id) = c
            .query_row("SELECT zeroblob(?1)", [width], |r| {
                Ok((
                    r.get::<_, Vec<u8>>(0)?,
                    r.get::<_, DeferredBytes<8>>(0)?,
                    r.get::<_, DeferredBytes<16>>(0)?,
                ))
            })
            .unwrap();
        assert_eq!(seq.bytes().map(u64::from_be_bytes), u64blob(old.clone()));
        assert_eq!(id.commit(), commitblob(old));
    }
}

// Recheck the concrete layout used by the capacity proof on the audited ABI.
#[cfg(target_pointer_width = "64")]
#[test]
fn header_and_late_control_have_bounded_inline_layouts() {
    use std::mem::{needs_drop, size_of};
    assert_eq!(size_of::<HeaderIds>(), 68);
    assert_eq!(size_of::<DeferredBytes<16>>(), 17);
    assert_eq!(size_of::<Header>(), 80);
    assert_eq!(size_of::<Control>(), 96);
    assert!(!needs_drop::<(HeaderIds, DeferredBytes<16>, Header, Control)>());
}
