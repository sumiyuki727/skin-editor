use super::*;

fn register(d: &mut Driver, store: &BlobStore, id: u128, bytes: &[u8]) -> BlobRef {
    let raw = store.stage_bytes(bytes.to_vec()).unwrap();
    let reference = raw.reference();
    d.command(vec![Command::RegisterAsset {
        asset: Asset::new(
            AssetId::new(id).unwrap(),
            format!("unused-{id}"),
            reference,
            AssetKind::Opaque,
        )
        .unwrap(),
    }]);
    reference
}
fn temporal_fixture(late: bool) -> (Directory, BlobStore, Driver, FileFactsRequest) {
    let dir = Directory::new();
    let store = BlobStore::default();
    let mut d = Driver::new(store.clone());
    if late {
        register(&mut d, &store, 1, b"temporal-original");
    }
    let (mut archive, _) = SqliteProjectFile::publish_new(
        dir.file(),
        store.clone(),
        FileLimits::default(),
        &mut d.capture(),
        request(400, None, SaveOrigin::Manual),
    )
    .unwrap();
    if late {
        d.set("second node with no new original");
    } else {
        register(&mut d, &store, 1, b"temporal-original");
    }
    archive
        .write_capture(&mut d.capture(), request(401, Some(400), SaveOrigin::Exit))
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
            project: archive.cache.header.project,
            phase: SaveTargetPhase::ExistingVerified,
            history: Some((&archive.cache.header.identity).into()),
            expected: ExpectedFileCommit {
                commit: Some(CommitId::new(401).unwrap()),
                generation: Some(2),
            },
        },
        selection: RestoreSelection::Manual,
    };
    drop(archive);
    (dir, store, d, req)
}
// Counterfeit ONLY a self-authored test archive. All state/index blocks, raw
// bytes and history nodes remain. Replace two checkpoint blocks, their owners
// and navigation keys, then restore every guard and topological sequence.
fn swap_declarations(path: &Path) {
    let conn = Connection::open(path).unwrap();
    conn.execute_batch("PRAGMA foreign_keys=OFF").unwrap();
    let mut q = conn
        .prepare("SELECT sha,bytes FROM blocks ORDER BY sequence")
        .unwrap();
    let mut blocks = q
        .query_map([], |r| {
            Ok((r.get::<_, Vec<u8>>(0)?, r.get::<_, Vec<u8>>(1)?))
        })
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    drop(q);
    let positions: Vec<_> = blocks
        .iter()
        .enumerate()
        .filter_map(|(i, (_, raw))| {
            let v: Value = serde_json::from_slice(raw).unwrap();
            (v["kind"] == "skin-editor.checkpoint").then_some(i)
        })
        .collect();
    assert_eq!(positions.len(), 2);
    let roots: Vec<_> = positions
        .iter()
        .map(|i| {
            let v: Value = serde_json::from_slice(&blocks[*i].1).unwrap();
            v["value"]["originals"].as_str().unwrap().to_owned()
        })
        .collect();
    assert_ne!(roots[0], roots[1]);
    let mut guards = Vec::new();
    for table in ["blocks", "block_refs", "commits"] {
        let name = format!("guard_{table}_UPDATE");
        let sql: String = conn
            .query_row(
                "SELECT sql FROM sqlite_schema WHERE name=?1",
                [&name],
                |r| r.get(0),
            )
            .unwrap();
        conn.execute_batch(&format!("DROP TRIGGER {name}")).unwrap();
        guards.push(sql);
    }
    for (n, i) in positions.into_iter().enumerate() {
        let (old, raw) = &blocks[i];
        let mut value: Value = serde_json::from_slice(raw).unwrap();
        value["value"]["originals"] = Value::String(roots[1 - n].clone());
        let mut refs = value["blocks"].as_array().unwrap().clone();
        let old_root = Value::String(roots[n].clone());
        let replacement = Value::String(roots[1 - n].clone());
        *refs.iter_mut().find(|r| **r == old_root).unwrap() = replacement.clone();
        refs.sort_by(|a, b| a.as_str().cmp(&b.as_str()));
        refs.dedup();
        value["blocks"] = Value::Array(refs);
        let new_raw = serde_json::to_vec(&value).unwrap();
        assert_eq!(new_raw.len(), raw.len());
        let new = sha(&new_raw);
        conn.execute(
            "UPDATE blocks SET sha=?1,bytes=?2 WHERE sha=?3",
            params![new, new_raw, old],
        )
        .unwrap();
        conn.execute(
            "UPDATE block_refs SET owner=?1 WHERE owner=?2",
            params![new, old],
        )
        .unwrap();
        conn.execute(
            "UPDATE block_refs SET target=?1 WHERE owner=?2 AND target=?3",
            params![
                bytes(replacement.as_str().unwrap()),
                new,
                bytes(old_root.as_str().unwrap())
            ],
        )
        .unwrap();
        conn.execute(
            "UPDATE commits SET navigation=?1 WHERE navigation=?2",
            params![new, old],
        )
        .unwrap();
        blocks[i] = (new.to_vec(), new_raw);
    }
    // Hash changes do not alter byte counts; order must still put dependencies
    // before owners. A unique temporary range prevents sequence collisions.
    for (i, (hash, _)) in blocks.iter().enumerate() {
        conn.execute(
            "UPDATE blocks SET sequence=?1 WHERE sha=?2",
            params![bu64((blocks.len() + i + 1) as u64), hash],
        )
        .unwrap();
    }
    let mut done = BTreeSet::new();
    while done.len() < blocks.len() {
        let before = done.len();
        for (hash, raw) in &blocks {
            if done.contains(hash) {
                continue;
            }
            let value: Value = serde_json::from_slice(raw).unwrap();
            if value["blocks"]
                .as_array()
                .unwrap()
                .iter()
                .all(|h| done.contains(&bytes(h.as_str().unwrap())))
            {
                conn.execute(
                    "UPDATE blocks SET sequence=?1 WHERE sha=?2",
                    params![bu64(done.len() as u64 + 1), hash],
                )
                .unwrap();
                done.insert(hash.clone());
            }
        }
        assert!(done.len() > before, "test graph must be acyclic");
    }
    for sql in guards {
        conn.execute_batch(&sql).unwrap();
    }
    schema::verify(&conn).unwrap();
}

#[test]
fn early_and_late_original_declarations_pass_old_full_but_fail_strict_observation() {
    for late in [false, true] {
        let (dir, store, _d, req) = temporal_fixture(late);
        swap_declarations(&dir.file());
        let before = std::fs::read(dir.file()).unwrap();
        let (archive, candidate) = open(&dir.file(), store.clone(), RestoreSelection::Manual);
        assert_eq!(archive.cache.originals.len(), 1);
        assert_eq!(archive.cache.checkpoints.len(), 2);
        drop(candidate);
        drop(archive);
        let scratch = Directory::new();
        let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
        let baseline = reader.usage().managed_bytes;
        assert_eq!(
            reader
                .inspect(&dir.file(), req, CaptureCancellation::default())
                .err(),
            Some(FileFactsError::ValidationFailed)
        );
        assert_eq!(reader.usage().managed_bytes, baseline);
        assert_eq!(reader.usage().scratch_reserved_bytes, 0);
        assert_eq!(std::fs::read(dir.file()).unwrap(), before);
    }
}

#[test]
fn unused_shared_deleted_and_old_branch_originals_still_admit_complete_observation() {
    let (dir, store, mut d, mut req) = temporal_fixture(false);
    let shared = register(&mut d, &store, 2, b"temporal-original");
    assert_eq!(
        shared,
        store
            .stage_bytes(b"temporal-original".to_vec())
            .unwrap()
            .reference()
    );
    register(&mut d, &store, 3, b"");
    d.command(vec![
        Command::RemoveAsset {
            asset: AssetId::new(1).unwrap(),
        },
        Command::RemoveAsset {
            asset: AssetId::new(2).unwrap(),
        },
    ]);
    let node = d.session.project_info().state.head;
    d.nav(HistoryAction::Checkout {
        node: NodeId::new(1).unwrap(),
    });
    d.set("separate branch without assets");
    let (mut archive, _) = open(&dir.file(), store.clone(), RestoreSelection::Manual);
    archive
        .write_capture(&mut d.capture(), request(402, Some(401), SaveOrigin::Exit))
        .unwrap();
    assert_eq!(archive.cache.originals.len(), 2);
    drop(archive);
    req.binding.expected = ExpectedFileCommit {
        commit: Some(CommitId::new(402).unwrap()),
        generation: Some(3),
    };
    let scratch = Directory::new();
    let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    let file = reader
        .inspect(&dir.file(), req, CaptureCancellation::default())
        .unwrap();
    assert_eq!(file.summary().originals, 2);
    assert_eq!(file.summary().original_bytes, shared.byte_len());
    let old = file
        .historical_content(FileHistoricalContentRequest {
            version: FILE_HISTORICAL_CONTENT_VERSION,
            observation: file.summary().observation,
            file: req,
            commit: CommitId::new(400).unwrap(),
        })
        .unwrap();
    assert_eq!(old.facts.unwrap().commit.nodes, 1);
    assert_ne!(node, d.session.project_info().state.head);
}

#[test]
fn real_closure_cancellation_releases_only_temporary_work_and_keeps_old_pages_and_work() {
    let (dir, store, mut d, _port, work, req) = save::fixture_on(true);
    let copy = Directory::new();
    std::fs::copy(dir.file(), copy.file()).unwrap();
    let scratch = Directory::new();
    let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    let q = reader
        .prepare_save(work, req.selection, &store, CaptureCancellation::default())
        .unwrap();
    let view = reader
        .inspect(&dir.file(), req, CaptureCancellation::default())
        .unwrap();
    let saved_page = view.commits(page(view.summary(), 0, 1)).unwrap();
    let before = reader.usage().managed_bytes;
    let before_bytes = std::fs::read(copy.file()).unwrap();
    support::probe(
        &mut reader,
        Probe {
            cancel_at: Some(Stage::ReadOriginalClosure),
            ..Probe::default()
        },
    );
    let cancel = CaptureCancellation::default();
    assert_eq!(
        reader.inspect(&copy.file(), req, cancel.clone()).err(),
        Some(FileFactsError::Cancelled)
    );
    assert!(cancel.is_cancelled());
    assert_eq!(reader.usage().managed_bytes, before);
    assert_eq!(reader.usage().scratch_reserved_bytes, 0);
    assert_eq!(
        view.commits(page(view.summary(), 0, 1)).unwrap(),
        saved_page
    );
    assert_eq!(std::fs::read(copy.file()).unwrap(), before_bytes);
    let counter = support::control_observation(&reader);
    let ticket = q.summary().ticket;
    drop(reader);
    assert!(counter.usage().bytes > 0);
    drop(view);
    assert!(counter.usage().bytes > 0);
    assert!(
        !d.session
            .save_coordination_status(1)
            .unwrap()
            .active
            .unwrap()
            .work_released
    );
    drop(q);
    assert_eq!(counter.usage().bytes, 0);
    assert_eq!(
        d.session.acknowledge_save(ticket),
        Ok(SaveAcknowledgement::Released)
    );
}
