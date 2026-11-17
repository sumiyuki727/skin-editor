//! Pure packed-container/encoding probes, not complete file admission evidence.
use super::*;
fn origin() -> FileProfileOrigin {
    FileProfileOrigin {
        version: 1,
        file: FileFactsRequest {
            version: 1,
            key: FileFactsKey {
                session: HistorySessionId::new(88).unwrap(),
                sequence: 9,
            },
            binding: SaveTargetBinding {
                version: 1,
                key: SaveBindingKey {
                    id: SaveBindingId::new(11).unwrap(),
                    generation: 1,
                },
                target: SaveTargetId::new(12).unwrap(),
                project: ProjectId::new(7).unwrap(),
                phase: SaveTargetPhase::ExistingVerified,
                history: Some(SaveHistoryScope {
                    history: PersistentHistoryId::new(1).unwrap(),
                    lineage: LineageId::new(2).unwrap(),
                    address_epoch: AddressEpoch::new(3).unwrap(),
                }),
                expected: ExpectedFileCommit {
                    commit: Some(CommitId::new(2).unwrap()),
                    generation: Some(2),
                },
            },
            selection: RestoreSelection::Manual,
        },
        observation: FileObservationId {
            instance: 1,
            sequence: 1,
        },
        root: PersistentNodeId::new(1004).unwrap(),
        opened_head: PersistentNodeId::new(1005).unwrap(),
        features_root: [1; 32],
        source_sha256: [2; 32],
    }
}
fn receipt(v: &Value) -> Option<ArchiveReceipt> {
    if v.is_null() {
        return None;
    }
    Some(ArchiveReceipt {
        commit: CommitId::new(v["commit"].as_u64().unwrap() as u128).unwrap(),
        generation: v["generation"].as_u64().unwrap(),
        origin: match v["origin"].as_u64().unwrap() {
            1 => SaveOrigin::Manual,
            2 => SaveOrigin::Automatic,
            _ => SaveOrigin::Exit,
        },
        head: PersistentNodeId::new(v["head"].as_u64().unwrap() as u128).unwrap(),
    })
}
#[test]
fn packed_fields_encode_all_eight_independent_before_models_including_required_counterfactual() {
    let models: Value = serde_json::from_str(
        &std::fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/P3_2B1_2H_VECTORS.json"),
        )
        .unwrap(),
    )
    .unwrap();
    for v in models["cases"].as_array().unwrap() {
        let m = &v["model"];
        let mut features: Vec<Value> = m["features"]
            .as_array()
            .unwrap()
            .iter()
            .map(|f| json!({"name":f["name"],"version":f["version"],"required":f["required"]==1}))
            .collect();
        features.sort_unstable_by_key(|f| {
            (
                f["name"].as_str().unwrap().to_owned(),
                f["version"].as_str().unwrap().to_owned(),
            )
        });
        let text: usize = features
            .iter()
            .map(|f| f["name"].as_str().unwrap().len() + f["version"].as_str().unwrap().len())
            .sum();
        let budget = ByteBudget::new(fixed_bytes() + text);
        let source = create(
            &budget,
            origin(),
            &features,
            &CaptureCancellation::default(),
            Probe::default(),
        )
        .unwrap();
        let p = (source.as_ref() as &dyn Any)
            .downcast_ref::<Provider>()
            .unwrap();
        let meta = ArchiveMetadata {
            last_commit: receipt(&m["latest"]),
            manual: receipt(&m["manual"]),
            protected: receipt(&m["protected"]),
            manual_marks: m["marks"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| {
                    (
                        CommitId::new(v["commit"].as_u64().unwrap() as u128).unwrap(),
                        PersistentNodeId::new(v["node"].as_u64().unwrap() as u128).unwrap(),
                    )
                })
                .collect(),
            tags: m["tags"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| ArchiveTag {
                    node: PersistentNodeId::new(v["node"].as_u64().unwrap() as u128).unwrap(),
                    name: v["name"].as_str().unwrap().into(),
                    value: v["value"].as_str().unwrap().into(),
                })
                .collect(),
            extensions: m["extensions"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| ArchiveExtension {
                    name: v["name"].as_str().unwrap().into(),
                    bytes: v["bytes_hex"]
                        .as_str()
                        .unwrap()
                        .as_bytes()
                        .chunks_exact(2)
                        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
                        .collect(),
                })
                .collect(),
        };
        let facts = encode(
            &meta,
            &p.payload,
            &mut [0; 4096],
            &CaptureCancellation::default(),
            Probe::default(),
        )
        .unwrap();
        assert_eq!(hex(&facts.metadata), v["sha256"].as_str().unwrap());
        assert_eq!(facts.features, features.len());
        assert_eq!(budget.used(), fixed_bytes() + text);
        drop(source);
        assert_eq!(budget.used(), 0);
        let short = ByteBudget::new(fixed_bytes() + text - 1);
        assert!(matches!(
            create(
                &short,
                origin(),
                &features,
                &CaptureCancellation::default(),
                Probe::default()
            ),
            Err(FileFactsError::Limit(FileFactsLimit::Control))
        ));
        assert_eq!(short.used(), 0);
    }
}
#[test]
fn packed_container_boundaries_and_exact_last_owner_release_do_not_claim_real_maximum_admission() {
    let values: Vec<Value> = (0..256).map(|i| { let prefix=format!("{i:03}"); json!({"name":format!("{}{}",prefix,"x".repeat(253)),"version":"v".repeat(64),"required":false}) }).collect();
    let budget = ByteBudget::new(fixed_bytes() + 81920);
    let source = create(
        &budget,
        origin(),
        &values,
        &CaptureCancellation::default(),
        Probe::default(),
    )
    .unwrap();
    assert_eq!(budget.used(), fixed_bytes() + 81920);
    let other = source.clone();
    drop(source);
    assert_eq!(budget.used(), fixed_bytes() + 81920);
    drop(other);
    assert_eq!(budget.used(), 0);
    let mut too_many = values.clone();
    too_many.push(json!({"name":"z","version":"","required":false}));
    assert!(
        create(
            &budget,
            origin(),
            &too_many,
            &CaptureCancellation::default(),
            Probe::default()
        )
        .is_err()
    );
    let mut too_long = values;
    too_long[0]["version"] = json!("v".repeat(65));
    assert!(
        create(
            &budget,
            origin(),
            &too_long,
            &CaptureCancellation::default(),
            Probe::default()
        )
        .is_err()
    );
    assert_eq!(budget.used(), 0);
}
