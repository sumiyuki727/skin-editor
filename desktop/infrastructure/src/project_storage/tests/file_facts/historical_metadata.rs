use super::*;
fn vectors() -> Value {
    serde_json::from_str(
        &std::fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/P3_2B1_2G_VECTORS.json"),
        )
        .unwrap(),
    )
    .unwrap()
}
fn query(file: &InspectedFileFacts, id: u128) -> FileHistoricalMetadataRequest {
    FileHistoricalMetadataRequest {
        version: FILE_HISTORICAL_METADATA_VERSION,
        observation: file.summary().observation,
        file: file.summary().request,
        commit: CommitId::new(id).unwrap(),
    }
}
fn check(file: &InspectedFileFacts, id: u128, name: &str) -> FileHistoricalMetadataFacts {
    let request = query(file, id);
    let result = file.historical_metadata(request).unwrap();
    assert_eq!(result.version, FILE_HISTORICAL_METADATA_VERSION);
    assert_eq!(result.request, request);
    assert_eq!(result.assurance, FileFactsAssurance::ExperimentalValidation);
    let f = result.facts.unwrap();
    assert_eq!(f.commit.fact.commit, request.commit);
    assert_eq!(f.proof_version, FILE_COORD_PROOF_VERSION);
    let v = vectors();
    let case = v["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == name)
        .unwrap();
    let m = &case["model"];
    assert_eq!(hex(&f.metadata), case["sha256"].as_str().unwrap());
    assert_eq!(
        (
            f.tags,
            f.extensions,
            f.features,
            f.required_features,
            f.manual_marks
        ),
        (
            m["tags"].as_array().unwrap().len(),
            m["extensions"].as_array().unwrap().len(),
            m["features"].as_array().unwrap().len(),
            m["features"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|v| v["required"] == 1)
                .count(),
            m["marks"].as_array().unwrap().len()
        )
    );
    assert_eq!(
        f.extension_bytes,
        m["extensions"]
            .as_array()
            .unwrap()
            .iter()
            .map(|e| bytes(e["bytes_hex"].as_str().unwrap()).len() as u64)
            .sum::<u64>()
    );
    f
}
fn tag_values(model: &Value) -> Vec<Value> {
    model["tags"].as_array().unwrap().iter().map(|v| json!({"node":format!("{:032x}",v["node"].as_u64().unwrap()),"name":v["name"],"value":v["value"]})).collect()
}
fn feature_values(model: &Value) -> Vec<Value> {
    model["features"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| json!({"name":v["name"],"version":v["version"],"required":v["required"]==1}))
        .collect()
}
fn extension_values(model: &Value) -> Vec<Value> {
    model["extensions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| {
            let raw = bytes(v["bytes_hex"].as_str().unwrap());
            json!({"name":v["name"],"sha":hex(&sha(&raw)),"bytes":raw.len().to_string()})
        })
        .collect()
}
// Edits ONLY a freshly self-authored test archive. Rebuild hashed metadata
// indexes/checkpoints, prune only fixture-construction orphans and restore all
// original guards before inspection. No product migration/cleanup capability.
fn metadata_fixture(bad: Option<&str>) -> (Directory, FileFactsRequest, [[u8; 32]; 2]) {
    let dir = Directory::new();
    let store = BlobStore::default();
    let mut d = Driver::new(store.clone());
    d.set("G old");
    let (mut a, _) = SqliteProjectFile::publish_new(
        dir.file(),
        store,
        FileLimits::default(),
        &mut d.capture(),
        request(900, None, SaveOrigin::Manual),
    )
    .unwrap();
    d.set("G new");
    a.write_capture(
        &mut d.capture(),
        request(
            2,
            Some(900),
            if bad == Some("future-feature") {
                SaveOrigin::Exit
            } else {
                SaveOrigin::Manual
            },
        ),
    )
    .unwrap();
    let req = FileFactsRequest {
        version: FILE_FACTS_PORT_VERSION,
        key: FileFactsKey {
            session: d.session.project_info().session_id,
            sequence: 1,
        },
        binding: SaveTargetBinding {
            version: SAVE_COORDINATION_VERSION,
            key: SaveBindingKey {
                id: SaveBindingId::new(11).unwrap(),
                generation: 1,
            },
            target: SaveTargetId::new(12).unwrap(),
            project: a.cache.header.project,
            phase: SaveTargetPhase::ExistingVerified,
            history: Some((&a.cache.header.identity).into()),
            expected: ExpectedFileCommit {
                commit: Some(CommitId::new(2).unwrap()),
                generation: Some(2),
            },
        },
        selection: RestoreSelection::Manual,
    };
    let v = vectors();
    let old = &v["cases"][4]["model"];
    let latest = &v["cases"][5]["model"];
    let mut tags = [tag_values(old), tag_values(latest)];
    let mut ext = [extension_values(old), extension_values(latest)];
    let mut features = [feature_values(old), feature_values(latest)];
    let mut global_features = features[1].clone();
    match bad {
        Some("required") => {
            let f = global_features
                .iter_mut()
                .find(|f| f["name"] == "future.雪")
                .unwrap();
            f["required"] = json!(true);
            features[1] = global_features.clone();
        }
        Some("missing") => {
            global_features.retain(|f| f["name"] != "storage.core");
            features[1] = global_features.clone();
        }
        Some("historical-feature") => {
            features[0].push(json!({"name":"past.only","version":"1","required":false}))
        }
        Some("future-feature") => {
            features[1].push(json!({"name":"future.unlisted","version":"1","required":false}))
        }
        Some("required-conflict") => {
            let mut f = features[0][0].clone();
            f["required"] = json!(false);
            features[0].push(f);
        }
        Some("extension-sha") => ext[0][0]["sha"] = json!("00".repeat(32)),
        Some("extension-length") => ext[0][0]["bytes"] = json!("1"),
        Some("tag-duplicate") => {
            let duplicate = tags[0][0].clone();
            tags[0].push(duplicate);
        }
        None => {}
        _ => panic!("unknown self-authored variant"),
    }
    let guards: Vec<(String, String)> = a
        .conn
        .prepare("SELECT name,sql FROM sqlite_schema WHERE type='trigger' AND name LIKE 'guard_%'")
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    a.conn.execute_batch("PRAGMA foreign_keys=OFF").unwrap();
    for (name, _) in &guards {
        a.conn
            .execute_batch(&format!("DROP TRIGGER {name}"))
            .unwrap();
    }
    a.conn.execute("DELETE FROM features", []).unwrap();
    for f in &global_features {
        a.conn
            .execute(
                "INSERT INTO features VALUES(?1,?2,?3)",
                params![
                    f["name"].as_str().unwrap(),
                    f["version"].as_str().unwrap(),
                    i64::from(f["required"].as_bool().unwrap())
                ],
            )
            .unwrap();
    }
    a.cache.features = global_features;
    let mut meta = (*a.cache.metadata).clone();
    meta.tags = latest["tags"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| ArchiveTag {
            node: PersistentNodeId::new(v["node"].as_u64().unwrap() as u128).unwrap(),
            name: v["name"].as_str().unwrap().into(),
            value: v["value"].as_str().unwrap().into(),
        })
        .collect();
    meta.extensions = latest["extensions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| ArchiveExtension {
            name: v["name"].as_str().unwrap().into(),
            bytes: bytes(v["bytes_hex"].as_str().unwrap()),
        })
        .collect();
    for t in &meta.tags {
        a.conn
            .execute(
                "INSERT INTO tags VALUES(?1,?2,?3)",
                params![t.node.get().to_be_bytes(), t.name, t.value],
            )
            .unwrap();
    }
    for e in &meta.extensions {
        a.conn
            .execute(
                "INSERT INTO extensions VALUES(?1,?2)",
                params![e.name, e.bytes],
            )
            .unwrap();
    }
    a.cache.metadata = Arc::new(meta);
    let mut roots = [[0; 32]; 2];
    let mut removed = Vec::new();
    for (i, p) in a.cache.checkpoints.clone().iter().enumerate() {
        let mut body: Value =
            serde_json::from_slice(&a.cache.blocks[&p.navigation].block.bytes).unwrap();
        let mut value = body["value"].take();
        for (role, items) in [
            ("tags", tags[i].clone()),
            ("extensions", ext[i].clone()),
            ("features", features[i].clone()),
        ] {
            let root = blocks::paged(&a.conn, &mut a.cache, role, items, a.limits).unwrap();
            value[role] = json!(root.hex());
            if role == "features" {
                roots[i] = root.bytes();
            }
        }
        let refs = [
            "nodes",
            "admissions",
            "originals",
            "redo",
            "tags",
            "extensions",
            "features",
        ]
        .into_iter()
        .map(|k| hash(value[k].as_str().unwrap()).unwrap())
        .collect();
        let root = blocks::make(
            &a.conn,
            &mut a.cache,
            "skin-editor.checkpoint",
            value,
            refs,
            vec![],
            a.limits,
        )
        .unwrap();
        a.conn
            .execute(
                "UPDATE commits SET navigation=?1 WHERE id=?2",
                params![root.bytes(), p.receipt.commit.get().to_be_bytes()],
            )
            .unwrap();
        a.cache.checkpoints[i].navigation = root;
        removed.push(p.navigation);
    }
    for root in removed {
        a.cache.blocks.remove(&root);
        a.conn
            .execute("DELETE FROM block_refs WHERE owner=?1", [root.bytes()])
            .unwrap();
        a.conn
            .execute("DELETE FROM blocks WHERE sha=?1", [root.bytes()])
            .unwrap();
    }
    let mut reachable = BTreeSet::new();
    for p in &a.cache.checkpoints {
        blocks::mark(&a.cache, p.navigation, &mut reachable).unwrap();
    }
    for n in a.cache.nodes.values() {
        blocks::mark(
            &a.cache,
            StateHash::from_bytes(n.info.origin.as_ref().unwrap().state),
            &mut reachable,
        )
        .unwrap();
        blocks::mark(&a.cache, n.canonical, &mut reachable).unwrap();
    }
    let unused: Vec<_> = a
        .cache
        .blocks
        .keys()
        .filter(|h| !reachable.contains(h))
        .copied()
        .collect();
    for h in unused {
        a.cache.blocks.remove(&h);
        a.conn
            .execute("DELETE FROM block_refs WHERE owner=?1", [h.bytes()])
            .unwrap();
        a.conn
            .execute("DELETE FROM blob_refs WHERE owner=?1", [h.bytes()])
            .unwrap();
        a.conn
            .execute("DELETE FROM blocks WHERE sha=?1", [h.bytes()])
            .unwrap();
    }
    for (i, h) in a.cache.blocks.keys().enumerate() {
        a.conn
            .execute(
                "UPDATE blocks SET sequence=?1 WHERE sha=?2",
                params![bu64((10000 + i) as u64), h.bytes()],
            )
            .unwrap();
    }
    let mut done = BTreeSet::new();
    while done.len() < a.cache.blocks.len() {
        let before = done.len();
        for (h, row) in &a.cache.blocks {
            if !done.contains(h) && row.refs.iter().all(|h| done.contains(h)) {
                a.conn
                    .execute(
                        "UPDATE blocks SET sequence=?1 WHERE sha=?2",
                        params![bu64(done.len() as u64 + 1), h.bytes()],
                    )
                    .unwrap();
                done.insert(*h);
            }
        }
        assert!(done.len() > before);
    }
    a.conn
        .execute(
            "UPDATE control SET block_count=?1,metadata_bytes=?2",
            params![
                a.cache.blocks.len() as i64,
                write::cost(&a.cache).unwrap() as i64
            ],
        )
        .unwrap();
    for (_, sql) in guards {
        a.conn.execute_batch(&sql).unwrap();
    }
    a.conn.execute_batch("PRAGMA foreign_keys=ON").unwrap();
    schema::verify(&a.conn).unwrap();
    drop(a);
    (dir, req, roots)
}

#[test]
fn file_metadata_tracks_four_real_generations_without_granting_q_metadata() {
    let (dir, store, d, _port, work, mut req) = save::fixture_on(true);
    let ticket = work.ticket();
    let scratch = Directory::new();
    let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    let q = reader
        .prepare_save(work, req.selection, &store, CaptureCancellation::default())
        .unwrap();
    let file = reader
        .inspect(&dir.file(), req, CaptureCancellation::default())
        .unwrap();
    let a = check(&file, 400, "golden_400");
    let b = check(&file, 401, "golden_401");
    assert_eq!(a.features_root, b.features_root);
    assert_eq!(b.metadata, file.summary().proof.metadata);
    let missing = query(&file, 999);
    assert_eq!(file.historical_metadata(missing).unwrap().facts, None);
    assert_eq!(
        file.compare_historical_save(
            &q,
            FileHistoricalSaveContext {
                version: FILE_HISTORICAL_SAVE_VERSION,
                observation: file.summary().observation,
                file: req,
                commit: CommitId::new(401).unwrap(),
                ticket
            }
        )
        .unwrap()
        .metadata,
        FileHistoricalSaveMetadata::NotCompared
    );
    drop(file);
    save::committed(&dir.file(), &store, ticket);
    original_query::future_before_old(&dir.file(), &store, ticket);
    req.binding.expected = ExpectedFileCommit {
        commit: Some(CommitId::new(3001).unwrap()),
        generation: Some(4),
    };
    let file = reader
        .inspect(&dir.file(), req, CaptureCancellation::default())
        .unwrap();
    check(&file, 400, "golden_400");
    check(&file, 401, "golden_401");
    check(&file, ticket.commit.get(), "golden_2001");
    let last = check(&file, 3001, "golden_3001");
    assert_eq!(last.metadata, file.summary().proof.metadata);
    assert_eq!(
        d.session
            .save_coordination_status(1)
            .unwrap()
            .active
            .unwrap()
            .outcome,
        Some(SavePreparationOutcome::CompleteNotSaved)
    );
}
#[test]
fn historical_metadata_uses_old_lists_actual_extension_bytes_and_generation_ordered_marks() {
    let (dir, req, roots) = metadata_fixture(None);
    let source = std::fs::read(dir.file()).unwrap();
    let store = BlobStore::default();
    let (legacy, candidate) = open(&dir.file(), store, RestoreSelection::Manual);
    assert_eq!(
        candidate
            .header()
            .metadata
            .manual_marks
            .iter()
            .map(|m| m.0.get())
            .collect::<Vec<_>>(),
        [900, 2]
    );
    drop(candidate);
    drop(legacy);
    let scratch = Directory::new();
    let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    let file = reader
        .inspect(&dir.file(), req, CaptureCancellation::default())
        .unwrap();
    let held = reader.usage();
    let old = check(&file, 900, "custom_900");
    let latest = check(&file, 2, "custom_2");
    assert_eq!(old.features_root, roots[0]);
    assert_eq!(latest.features_root, roots[1]);
    assert_ne!(old.features_root, latest.features_root);
    assert_ne!(old.metadata, latest.metadata);
    assert_eq!(latest.metadata, file.summary().proof.metadata);
    assert_eq!(reader.usage().managed_bytes, held.managed_bytes);
    assert_eq!(reader.usage().managed_peak_bytes, held.managed_peak_bytes);
    drop(file);
    assert_eq!(std::fs::read(dir.file()).unwrap(), source);
}
#[test]
fn unsupported_or_unbound_past_and_future_metadata_rejects_the_whole_file() {
    for bad in [
        "required",
        "missing",
        "historical-feature",
        "future-feature",
        "required-conflict",
        "extension-sha",
        "extension-length",
        "tag-duplicate",
    ] {
        let (dir, req, _) = metadata_fixture(Some(bad));
        let source = std::fs::read(dir.file()).unwrap();
        let legacy = SqliteProjectFile::load(
            dir.file(),
            BlobStore::default(),
            FileLimits::default(),
            RestoreSelection::Manual,
            CaptureCancellation::default(),
        )
        .err()
        .unwrap();
        match bad {
            "required" => assert!(matches!(
                legacy,
                StorageError::Unsupported("required feature")
            )),
            "missing" => assert!(matches!(
                legacy,
                StorageError::Unsupported("missing required feature")
            )),
            "historical-feature" | "future-feature" | "required-conflict" => assert!(matches!(
                legacy,
                StorageError::Invalid("historical features")
            )),
            "extension-sha" | "extension-length" => assert!(matches!(
                legacy,
                StorageError::Invalid("historical extension binding")
            )),
            "tag-duplicate" => assert!(matches!(legacy, StorageError::Invalid("historical tags"))),
            _ => unreachable!(),
        }
        let scratch = Directory::new();
        let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
        let held = reader.usage().managed_bytes;
        assert_eq!(
            reader
                .inspect(&dir.file(), req, CaptureCancellation::default())
                .err(),
            Some(if matches!(bad, "required" | "missing") {
                FileFactsError::Unsupported
            } else {
                FileFactsError::ValidationFailed
            }),
            "{bad}"
        );
        assert_eq!(reader.usage().managed_bytes, held);
        assert_eq!(reader.usage().scratch_reserved_bytes, 0);
        assert_eq!(std::fs::read(dir.file()).unwrap(), source);
    }
}
#[test]
fn metadata_query_checks_complete_version_request_and_observation_without_reusing_old_pages() {
    let (dir, _d, req) = golden();
    let copy = Directory::new();
    std::fs::copy(dir.file(), copy.file()).unwrap();
    let scratch = Directory::new();
    let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    let file = reader
        .inspect(&dir.file(), req, CaptureCancellation::default())
        .unwrap();
    let request = query(&file, 401);
    let mut bad = request;
    bad.version += 1;
    assert_eq!(
        file.historical_metadata(bad),
        Err(FileFactsError::UnsupportedVersion)
    );
    bad = request;
    bad.file.version += 1;
    assert_eq!(
        file.historical_metadata(bad),
        Err(FileFactsError::UnsupportedVersion)
    );
    let mut variants = Vec::new();
    bad = request;
    bad.observation.instance += 1;
    variants.push(bad);
    bad = request;
    bad.observation.sequence += 1;
    variants.push(bad);
    bad = request;
    bad.file.key.session = HistorySessionId::new(999).unwrap();
    variants.push(bad);
    bad = request;
    bad.file.key.sequence += 1;
    variants.push(bad);
    bad = request;
    bad.file.binding.version += 1;
    variants.push(bad);
    bad = request;
    bad.file.binding.key.id = SaveBindingId::new(999).unwrap();
    variants.push(bad);
    bad = request;
    bad.file.binding.key.generation += 1;
    variants.push(bad);
    bad = request;
    bad.file.binding.target = SaveTargetId::new(999).unwrap();
    variants.push(bad);
    bad = request;
    bad.file.binding.project = ProjectId::new(999).unwrap();
    variants.push(bad);
    bad = request;
    bad.file.binding.phase = SaveTargetPhase::NewUnpublished;
    variants.push(bad);
    bad = request;
    bad.file.binding.history = None;
    variants.push(bad);
    bad = request;
    bad.file.binding.expected.commit = Some(CommitId::new(400).unwrap());
    variants.push(bad);
    bad = request;
    bad.file.binding.expected.generation = Some(1);
    variants.push(bad);
    bad = request;
    bad.file.selection = RestoreSelection::Latest;
    variants.push(bad);
    for bad in variants {
        assert_eq!(
            file.historical_metadata(bad),
            Err(FileFactsError::StalePage)
        );
    }
    let old = file.historical_metadata(request).unwrap();
    let newer = reader
        .inspect(&copy.file(), req, CaptureCancellation::default())
        .unwrap();
    assert_eq!(
        newer.historical_metadata(request),
        Err(FileFactsError::StalePage)
    );
    assert_eq!(file.historical_metadata(request).unwrap(), old);
    drop(newer);
    drop(file);
    let scratch2 = Directory::new();
    let mut reader2 = FileFactsReader::open(&scratch2.path, FileFactsLimits::default()).unwrap();
    let unrelated = reader2
        .inspect(&dir.file(), req, CaptureCancellation::default())
        .unwrap();
    assert_eq!(
        unrelated.historical_metadata(request),
        Err(FileFactsError::StalePage)
    );
}
#[test]
fn cancellation_after_first_real_metadata_digest_keeps_old_pages_q_and_actual_release() {
    let (dir, store, mut d, _port, work, req) = save::fixture_on(true);
    let copy = Directory::new();
    std::fs::copy(dir.file(), copy.file()).unwrap();
    let source = std::fs::read(dir.file()).unwrap();
    let scratch = Directory::new();
    let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    let q = reader
        .prepare_save(work, req.selection, &store, CaptureCancellation::default())
        .unwrap();
    let ticket = q.summary().ticket;
    let file = reader
        .inspect(&dir.file(), req, CaptureCancellation::default())
        .unwrap();
    let request = query(&file, 400);
    let old = file.historical_metadata(request).unwrap();
    let p = file.commits(page(file.summary(), 0, 1)).unwrap();
    let held = reader.usage().managed_bytes;
    let counter = support::control_observation(&reader);
    support::probe(
        &mut reader,
        Probe {
            cancel_at: Some(Stage::ReadMetadataSummary),
            ..Probe::default()
        },
    );
    let cancel = CaptureCancellation::default();
    assert_eq!(
        reader.inspect(&copy.file(), req, cancel.clone()).err(),
        Some(FileFactsError::Cancelled)
    );
    assert!(cancel.is_cancelled());
    assert_eq!(reader.usage().managed_bytes, held);
    assert_eq!(reader.usage().scratch_reserved_bytes, 0);
    assert_eq!(file.historical_metadata(request).unwrap(), old);
    assert_eq!(file.commits(page(file.summary(), 0, 1)).unwrap(), p);
    let c = FileHistoricalSaveContext {
        version: FILE_HISTORICAL_SAVE_VERSION,
        observation: file.summary().observation,
        file: req,
        commit: CommitId::new(401).unwrap(),
        ticket,
    };
    let relation = file.compare_historical_save(&q, c).unwrap();
    assert_eq!(relation.metadata, FileHistoricalSaveMetadata::NotCompared);
    assert_eq!(
        relation.conclusion,
        FileCaptureConclusion::InsufficientForSaveReconciliation
    );
    assert_eq!(std::fs::read(copy.file()).unwrap(), source);
    drop(reader);
    assert_eq!(file.historical_metadata(request).unwrap(), old);
    assert_eq!(file.compare_historical_save(&q, c).unwrap(), relation);
    assert!(counter.usage().bytes > 0);
    drop(file);
    assert_eq!(std::fs::read(dir.file()).unwrap(), source);
    assert!(counter.usage().bytes > 0);
    assert_eq!(
        d.session.acknowledge_save(ticket),
        Ok(SaveAcknowledgement::WaitingResources)
    );
    drop(q);
    assert_eq!(counter.usage().bytes, 0);
    assert_eq!(
        d.session.acknowledge_save(ticket),
        Ok(SaveAcknowledgement::Released)
    );
}
#[test]
fn metadata_fixed_layout_preserves_two_observations_and_real_q_at_exact_shared_budget() {
    let (point, work_bytes) = support::metadata_layout();
    assert!(point <= 80 && work_bytes <= 40 * 1024);
    let (retained, _) = support::capture_controls();
    let (extra, _) = support::save_controls();
    let required = support::inspect_controls(2) + retained + extra;
    assert!(required <= FileFactsLimits::default().control_bytes);
    for exact in [true, false] {
        let (dir, store, mut d, _port, work, req) = save::fixture_on(true);
        let copy = Directory::new();
        std::fs::copy(dir.file(), copy.file()).unwrap();
        let ticket = work.ticket();
        let scratch = Directory::new();
        let mut reader = FileFactsReader::open(
            &scratch.path,
            FileFactsLimits {
                control_bytes: required - usize::from(!exact),
                ..FileFactsLimits::default()
            },
        )
        .unwrap();
        let q = reader
            .prepare_save(work, req.selection, &store, CaptureCancellation::default())
            .unwrap();
        let first = reader
            .inspect(&dir.file(), req, CaptureCancellation::default())
            .unwrap();
        let c = query(&first, 401);
        let old = first.historical_metadata(c).unwrap();
        let held = reader.usage().managed_bytes;
        let next = reader.inspect(&copy.file(), req, CaptureCancellation::default());
        if exact {
            let second = next.unwrap();
            assert_eq!(reader.usage().managed_peak_bytes, required);
            let held = reader.usage().managed_bytes;
            check(&second, 401, "golden_401");
            assert_eq!(reader.usage().managed_bytes, held);
            assert_eq!(reader.usage().managed_peak_bytes, required);
            drop(second);
        } else {
            assert_eq!(
                next.err(),
                Some(FileFactsError::Limit(FileFactsLimit::Control))
            );
        }
        assert_eq!(reader.usage().managed_bytes, held);
        assert_eq!(first.historical_metadata(c).unwrap(), old);
        drop(reader);
        assert_eq!(first.historical_metadata(c).unwrap(), old);
        drop(first);
        assert_eq!(
            d.session.acknowledge_save(ticket),
            Ok(SaveAcknowledgement::WaitingResources)
        );
        drop(q);
        assert_eq!(
            d.session.acknowledge_save(ticket),
            Ok(SaveAcknowledgement::Released)
        );
    }
}
