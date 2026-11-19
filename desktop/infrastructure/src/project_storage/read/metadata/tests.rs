use super::*;

#[test]
fn latest_metadata_requires_exact_order_spelling_and_original_text() {
    let n = PersistentNodeId::new(47).unwrap();
    let nodes = BTreeSet::from([n]);
    let meta = ArchiveMetadata {
        tags: ["user:a", "user:雪"]
            .into_iter()
            .map(|name| ArchiveTag {
                node: n,
                name: name.into(),
                value: "e\u{301}\0\r\n".into(),
            })
            .collect(),
        extensions: vec![
            ArchiveExtension {
                name: "x".into(),
                bytes: vec![0, 255],
            },
            ArchiveExtension {
                name: "雪".into(),
                bytes: vec![],
            },
        ],
        ..ArchiveMetadata::default()
    };
    let tags: Vec<_> = meta
        .tags
        .iter()
        .map(|t| json!({"node":ptext(n),"name":t.name,"value":t.value}))
        .collect();
    let ext: Vec<_> = meta
        .extensions
        .iter()
        .map(|e| json!({"name":e.name,"sha":hex(&sha(&e.bytes)),"bytes":e.bytes.len().to_string()}))
        .collect();
    let mut features = write::features();
    features.push(json!({"name":"optional","version":"","required":false}));
    let validate = |tags: &[Value], ext: &[Value], f: &[Value]| {
        validate_past_metadata(
            &nodes,
            &meta,
            &features,
            &blocks::MetadataPages::from_rows(tags),
            &blocks::MetadataPages::from_rows(ext),
            &blocks::MetadataPages::from_rows(f),
            true,
        )
    };
    let verify = |tags: &[Value], ext: &[Value], f: &[Value]| {
        let project = |rows: &[Value]| {
            let mut rows = rows.to_vec();
            rows.iter_mut().for_each(retain_metadata_fields);
            rows
        };
        let expected = validate(tags, ext, f);
        assert_eq!(
            validate(&project(tags), &project(ext), &project(f)),
            expected
        );
        expected
    };
    assert_eq!(verify(&tags, &ext, &features), Ok(true));
    let mut reordered = tags.clone();
    reordered.reverse();
    assert_eq!(verify(&reordered, &ext, &features), Ok(false));
    reordered = ext.clone();
    reordered.reverse();
    assert_eq!(verify(&tags, &reordered, &features), Ok(false));
    reordered = features.clone();
    reordered.reverse();
    assert_eq!(verify(&tags, &ext, &reordered), Ok(false));
    let mut changed = tags.clone();
    changed[1]["value"] = json!("é\0\r\n");
    assert_eq!(verify(&changed, &ext, &features), Ok(false));
    assert_eq!(verify(&tags[..1], &ext[..1], &features[..4]), Ok(false));
    for (field, value, expected) in [
        ("node", json!("F".repeat(32)), "hex"),
        ("node", json!("00"), "fixed bytes"),
        ("node", json!("0".repeat(32)), "identity"),
        (
            "node",
            json!(ptext(PersistentNodeId::new(48).unwrap())),
            "historical tags",
        ),
        ("value", json!(false), "text"),
        ("name", json!([{"nested":"value"}]), "text"),
    ] {
        let mut changed = tags.clone();
        changed[0][field] = value;
        assert_eq!(
            verify(&changed, &ext, &features),
            Err(StorageError::Invalid(expected))
        );
    }
    for (field, value, expected) in [
        ("sha", json!("A".repeat(64)), "hex"),
        ("sha", json!("00"), "fixed bytes"),
        ("bytes", json!("02"), "decimal"),
        ("bytes", json!("18446744073709551616"), "decimal"),
        ("bytes", json!(2), "text"),
        ("sha", json!({"nested":[]}), "text"),
    ] {
        let mut changed = ext.clone();
        changed[0][field] = value;
        assert_eq!(
            verify(&tags, &changed, &features),
            Err(StorageError::Invalid(expected))
        );
    }
    let mut changed = features.clone();
    changed[0]["required"] = json!("true");
    changed[0]["extra"] = json!(true);
    assert_eq!(
        verify(&tags, &ext, &changed),
        Err(StorageError::Invalid("feature required"))
    );
    changed[0]["required"] = json!(true);
    assert_eq!(
        verify(&tags, &ext, &changed),
        Err(StorageError::Unsupported("metadata field"))
    );
    let mut bad_features = features.clone();
    bad_features[0]["required"] = json!([true]);
    bad_features[0]["extra"] = json!({"unknown": []});
    assert_eq!(
        verify(&tags, &ext, &bad_features),
        Err(StorageError::Invalid("feature required"))
    );
    bad_features[0].as_object_mut().unwrap().remove("version");
    assert_eq!(
        verify(&tags, &ext, &bad_features),
        Err(StorageError::Invalid("version"))
    );
    let mut bad_tags = tags.clone();
    bad_tags[0]["extra"] = json!([1, 2]);
    assert_eq!(
        verify(&bad_tags, &ext, &bad_features),
        Err(StorageError::Unsupported("metadata field"))
    );
    bad_tags[0] = json!([{"node": ptext(n)}]);
    assert_eq!(
        verify(&bad_tags, &ext, &bad_features),
        Err(StorageError::Invalid("object"))
    );
}

#[test]
fn duplicate_detection_preserves_source_error_order_across_fixed_and_wide_lists() {
    let n = PersistentNodeId::new(4).unwrap();
    let nodes = BTreeSet::from([n]);
    let meta = ArchiveMetadata::default();
    let features = write::features();
    let node = ptext(n);
    let tags: Vec<_> = (0..=METADATA_ITEMS)
        .map(|i| json!({"node":node,"name":format!("user:{i:04}"),"value":""}))
        .collect();
    let verify = |rows: &[Value]| {
        validate_past_metadata(
            &nodes,
            &meta,
            &features,
            &blocks::MetadataPages::from_rows(rows),
            &blocks::MetadataPages::from_rows(&[]),
            &blocks::MetadataPages::from_rows(&features),
            false,
        )
    };
    for count in [0, 1, 128, 129, 256, 257, METADATA_ITEMS, METADATA_ITEMS + 1] {
        assert_eq!(verify(&tags[..count]), Ok(false));
    }
    // Make the later duplicate sort first by key. Validation still reports
    // the earliest bad row by original index, before an unsupported field.
    for count in [4, 257, METADATA_ITEMS + 1] {
        let mut rows = tags[..count].to_vec();
        rows[1] = rows[0].clone();
        rows[0]["name"] = json!("user:zz");
        rows[1]["name"] = json!("user:zz");
        rows[2]["extra"] = json!(1);
        let last = rows.len() - 1;
        rows[last] = rows[2].clone();
        assert_eq!(verify(&rows), Err(StorageError::Invalid("historical tags")));
        rows[0]["extra"] = json!(1);
        assert_eq!(
            verify(&rows),
            Err(StorageError::Unsupported("metadata field"))
        );
        rows[0].as_object_mut().unwrap().remove("node");
        assert_eq!(verify(&rows), Err(StorageError::Invalid("node")));
    }
}
