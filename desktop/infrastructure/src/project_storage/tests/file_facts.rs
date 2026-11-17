use super::super::file_facts::test_support as support;
use super::*;

#[cfg(windows)]
mod capture;
mod preflight;
#[cfg(windows)]
mod save;

fn vectors() -> Value {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/P3_2B1_PROOF_VECTORS.json");
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}
fn bytes(text: &str) -> Vec<u8> {
    text.as_bytes()
        .chunks_exact(2)
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect()
}
#[test]
fn independent_framing_and_complete_field_vectors_do_not_use_runtime_generation() {
    let vectors = vectors();
    let fields: Vec<_> = vectors["fields"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| bytes(v["payload_hex"].as_str().unwrap()))
        .collect();
    for v in vectors["envelope_vectors"].as_array().unwrap() {
        assert_eq!(
            hex(&support::framing(
                v["domain"].as_u64().unwrap() as u16,
                &fields
            )),
            v["sha256"].as_str().unwrap()
        );
    }
    for v in vectors["file_case"]["domains"]
        .as_array()
        .unwrap()
        .iter()
        .chain(std::iter::once(&vectors["metadata_field_case"]["domain"]))
    {
        let fields: Vec<_> = v["fields"]
            .as_array()
            .unwrap()
            .iter()
            .map(|f| bytes(f["payload_hex"].as_str().unwrap()))
            .collect();
        assert_eq!(
            hex(&support::framing(
                v["domain"].as_u64().unwrap() as u16,
                &fields
            )),
            v["sha256"].as_str().unwrap()
        );
    }
    let changed = [b"ab".to_vec(), b"c".to_vec()];
    assert_ne!(
        support::framing(1, &changed),
        support::framing(1, &[b"a".to_vec(), b"bc".to_vec()])
    );
    assert_ne!(support::framing(1, &changed), support::framing(2, &changed));
}

#[cfg(windows)]
fn golden() -> (Directory, Driver, FileFactsRequest) {
    golden_variant("Unknown:雪!\r\n")
}
#[cfg(windows)]
fn golden_variant(last: &str) -> (Directory, Driver, FileFactsRequest) {
    let dir = Directory::new();
    let store = BlobStore::default();
    let mut d = Driver::new(store.clone());
    let raw = store.stage_bytes(b"opaque\0original\r\n".to_vec()).unwrap();
    d.command(vec![Command::RegisterAsset {
        asset: Asset::new(
            AssetId::new(1).unwrap(),
            "opaque".into(),
            raw.reference(),
            AssetKind::Opaque,
        )
        .unwrap(),
    }]);
    d.set("Unknown:雪\r\n");
    let (mut file, _) = SqliteProjectFile::publish_new(
        dir.file(),
        store,
        FileLimits::default(),
        &mut d.capture(),
        request(400, None, SaveOrigin::Manual),
    )
    .unwrap();
    d.set(last);
    file.write_capture(&mut d.capture(), request(401, Some(400), SaveOrigin::Exit))
        .unwrap();
    let request = FileFactsRequest {
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
            project: file.cache.header.project,
            phase: SaveTargetPhase::ExistingVerified,
            history: Some((&file.cache.header.identity).into()),
            expected: ExpectedFileCommit {
                commit: Some(CommitId::new(401).unwrap()),
                generation: Some(2),
            },
        },
        selection: RestoreSelection::Manual,
    };
    drop(file);
    (dir, d, request)
}
#[cfg(windows)]
fn page(summary: FileFactsSummary, offset: usize, limit: usize) -> FileFactsPageRequest {
    FileFactsPageRequest {
        version: FILE_FACTS_PORT_VERSION,
        observation: summary.observation,
        key: summary.request.key,
        binding: summary.request.binding,
        offset,
        limit,
    }
}
#[cfg(windows)]
#[test]
fn complete_file_proof_matches_independent_model_and_keeps_original_read_only() {
    let (dir, _d, req) = golden();
    let scratch = Directory::new();
    let before = std::fs::read(dir.file()).unwrap();
    let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    let baseline = reader.usage();
    let view = reader
        .inspect(&dir.file(), req, CaptureCancellation::default())
        .unwrap();
    let summary = view.summary();
    assert_eq!(
        (
            summary.nodes,
            summary.admissions,
            summary.originals,
            summary.original_bytes,
            summary.commits
        ),
        (4, 1, 1, 17, 2)
    );
    assert_eq!(
        summary.assurance,
        FileFactsAssurance::ExperimentalValidation
    );
    assert_eq!(summary.selected_head, summary.files.manual.unwrap().head);
    assert_ne!(summary.selected_head, summary.files.protected.unwrap().head);
    let v = vectors();
    for (i, hash) in [
        summary.proof.content,
        summary.proof.originals,
        summary.proof.metadata,
        summary.proof.request,
        summary.proof.baseline,
    ]
    .iter()
    .enumerate()
    {
        assert_eq!(
            hex(hash),
            v["file_case"]["domains"][i]["sha256"].as_str().unwrap()
        );
    }
    let first = view.commits(page(summary, 0, 1)).unwrap();
    assert_eq!((first.len, first.next), (1, Some(1)));
    assert_eq!(first.entries[0].unwrap().fact.commit.get(), 400);
    let second = view.commits(page(summary, 1, 16)).unwrap();
    assert_eq!((second.len, second.next), (1, None));
    assert_eq!(
        second.entries[0].unwrap().previous,
        Some(CommitId::new(400).unwrap())
    );
    assert_eq!(view.commits(page(summary, 2, 16)).unwrap().len, 0);
    assert_eq!(
        view.commits(page(summary, 3, 1)),
        Err(FileFactsError::InvalidPage)
    );
    assert_eq!(
        view.commits(page(summary, 0, 0)),
        Err(FileFactsError::InvalidPage)
    );
    assert_eq!(std::fs::read_dir(&scratch.path).unwrap().count(), 1);
    assert_eq!(reader.usage().scratch_reserved_bytes, 0);
    assert!(reader.usage().managed_bytes > baseline.managed_bytes);
    drop(view);
    assert_eq!(reader.usage().managed_bytes, baseline.managed_bytes);
    assert_eq!(std::fs::read(dir.file()).unwrap(), before);
}
#[cfg(windows)]
#[test]
fn stale_observation_pages_are_rejected_even_when_the_caller_reuses_its_request() {
    let (dir, _d, req) = golden();
    let scratch = Directory::new();
    let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    let view = reader
        .inspect(&dir.file(), req, CaptureCancellation::default())
        .unwrap();
    let old = view.summary();
    drop(view);
    let view = reader
        .inspect(&dir.file(), req, CaptureCancellation::default())
        .unwrap();
    assert_ne!(old.observation, view.summary().observation);
    assert_eq!(
        view.commits(page(old, 0, 1)),
        Err(FileFactsError::StalePage)
    );
    let mut wrong = page(view.summary(), 0, 1);
    wrong.version = 2;
    assert_eq!(view.commits(wrong), Err(FileFactsError::UnsupportedVersion));
}
#[cfg(windows)]
#[test]
fn all_sidecars_including_empty_files_are_refused_without_recovery_or_mutation() {
    let (dir, _d, req) = golden();
    let scratch = Directory::new();
    let before = std::fs::read(dir.file()).unwrap();
    let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    for suffix in ["-journal", "-wal", "-shm"] {
        for value in [b"".as_slice(), b"untrusted journal"] {
            let path = dir.file().with_file_name(format!("test.oskproj{suffix}"));
            std::fs::write(&path, value).unwrap();
            assert!(matches!(
                reader.inspect(&dir.file(), req, CaptureCancellation::default()),
                Err(FileFactsError::Unsupported)
            ));
            assert_eq!(std::fs::read(&path).unwrap(), value);
            std::fs::remove_file(&path).unwrap();
            assert_eq!(std::fs::read(dir.file()).unwrap(), before);
            assert_eq!(reader.usage().scratch_reserved_bytes, 0);
        }
    }
}
#[cfg(windows)]
#[test]
fn source_and_parent_guard_survive_until_the_last_observation_owner_exits() {
    use std::fs::OpenOptions;
    let (dir, _d, req) = golden();
    let scratch = Directory::new();
    let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    let view = reader
        .inspect(&dir.file(), req, CaptureCancellation::default())
        .unwrap();
    assert!(OpenOptions::new().write(true).open(dir.file()).is_err());
    assert!(std::fs::rename(&dir.path, dir.path.with_extension("renamed")).is_err());
    assert!(std::fs::remove_file(dir.file()).is_err());
    assert!(matches!(
        FileFactsReader::open(&scratch.path, FileFactsLimits::default()),
        Err(FileFactsError::Busy)
    ));
    drop(view);
    assert!(OpenOptions::new().write(true).open(dir.file()).is_ok());
}
#[cfg(windows)]
#[test]
fn cleanup_failure_retains_disk_ownership_and_blocks_reuse_until_actual_removal() {
    let (dir, _d, req) = golden();
    let scratch = Directory::new();
    let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    support::cleanup(&mut reader, true);
    let token = match reader.inspect(&dir.file(), req, CaptureCancellation::default()) {
        Err(FileFactsError::Cleanup { token }) => token,
        _ => panic!("expected retained cleanup"),
    };
    assert_eq!(reader.pending_cleanup(), Some(token));
    assert!(reader.usage().scratch_reserved_bytes > 0);
    assert!(
        matches!(reader.inspect(&dir.file(),req,CaptureCancellation::default()),Err(FileFactsError::Cleanup{token:t}) if t==token)
    );
    support::cleanup(&mut reader, false);
    reader.retry_cleanup().unwrap();
    assert_eq!(reader.usage().scratch_reserved_bytes, 0);
    assert_eq!(std::fs::read_dir(&scratch.path).unwrap().count(), 1);
    drop(
        reader
            .inspect(&dir.file(), req, CaptureCancellation::default())
            .unwrap(),
    );
    support::cleanup(&mut reader, true);
    assert!(
        reader
            .inspect(&dir.file(), req, CaptureCancellation::default())
            .is_err()
    );
    drop(reader);
    assert!(matches!(
        FileFactsReader::open(&scratch.path, FileFactsLimits::default()),
        Err(FileFactsError::Unsupported)
    ));
}
#[cfg(windows)]
#[test]
fn wrong_versions_binding_and_pre_cancel_do_not_create_scratch_or_a_qualification() {
    let (dir, _d, req) = golden();
    let scratch = Directory::new();
    let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    let baseline = reader.usage().managed_bytes;
    let mut bad = req;
    bad.version = 2;
    assert!(matches!(
        reader.inspect(
            std::path::Path::new("does-not-exist"),
            bad,
            CaptureCancellation::default()
        ),
        Err(FileFactsError::UnsupportedVersion)
    ));
    bad = req;
    bad.key.sequence = 0;
    assert!(matches!(
        reader.inspect(&dir.file(), bad, CaptureCancellation::default()),
        Err(FileFactsError::InvalidRequest)
    ));
    bad = req;
    bad.binding.expected.generation = Some(1);
    assert!(matches!(
        reader.inspect(&dir.file(), bad, CaptureCancellation::default()),
        Err(FileFactsError::TargetConflict)
    ));
    let cancel = CaptureCancellation::default();
    cancel.cancel();
    assert!(matches!(
        reader.inspect(&dir.file(), req, cancel),
        Err(FileFactsError::Cancelled)
    ));
    assert_eq!(reader.usage().managed_bytes, baseline);
    assert_eq!(std::fs::read_dir(&scratch.path).unwrap().count(), 1);
}
#[cfg(windows)]
#[test]
fn exact_file_scratch_and_new_wrapper_capacity_boundaries_are_independent() {
    let (dir, _d, req) = golden();
    let file_bytes = std::fs::metadata(dir.file()).unwrap().len();
    for less in [false, true] {
        for field in 0..3 {
            let scratch = Directory::new();
            let mut limits = FileFactsLimits::default();
            match field {
                0 => limits.file_bytes = file_bytes - u64::from(less),
                1 => limits.scratch_bytes = file_bytes - u64::from(less),
                _ => limits.control_bytes = support::controls() - usize::from(less),
            }
            let mut reader = FileFactsReader::open(&scratch.path, limits).unwrap();
            let before = reader.usage().managed_bytes;
            let result = reader.inspect(&dir.file(), req, CaptureCancellation::default());
            if less {
                assert!(matches!(result, Err(FileFactsError::Limit(_))));
            } else {
                drop(result.unwrap());
            }
            assert_eq!(reader.usage().managed_bytes, before);
            assert_eq!(reader.usage().scratch_reserved_bytes, 0);
        }
    }
}
#[cfg(windows)]
#[test]
fn cancellation_after_copy_or_inside_full_validation_releases_source_and_owned_scratch() {
    let (dir, _d, req) = golden();
    let original = std::fs::read(dir.file()).unwrap();
    for stage in [Stage::CopyMain, Stage::ReadOriginal, Stage::ReadNodes] {
        let scratch = Directory::new();
        let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
        let before = reader.usage().managed_bytes;
        support::probe(
            &mut reader,
            Probe {
                cancel_at: Some(stage),
                ..Probe::default()
            },
        );
        assert!(matches!(
            reader.inspect(&dir.file(), req, CaptureCancellation::default()),
            Err(FileFactsError::Cancelled)
        ));
        assert_eq!(reader.usage().managed_bytes, before);
        assert_eq!(reader.usage().scratch_reserved_bytes, 0);
        assert_eq!(std::fs::read(dir.file()).unwrap(), original);
    }
}
#[cfg(not(windows))]
#[test]
fn unsupported_snapshot_platform_does_not_claim_a_validated_file() {
    let scratch = Directory::new();
    assert!(matches!(
        FileFactsReader::open(&scratch.path, FileFactsLimits::default()),
        Err(FileFactsError::Unsupported)
    ));
}

#[test]
fn metadata_encoder_matches_the_independent_none_and_extension_field_case() {
    let v = vectors();
    let m = &v["metadata_field_case"]["model"];
    let receipt = |value: &Value| -> Option<ArchiveReceipt> {
        if value.is_null() {
            None
        } else {
            Some(ArchiveReceipt {
                commit: CommitId::new(value["commit"].as_u64().unwrap() as u128).unwrap(),
                generation: value["generation"].as_u64().unwrap(),
                origin: match value["origin"].as_u64().unwrap() {
                    1 => SaveOrigin::Manual,
                    2 => SaveOrigin::Automatic,
                    3 => SaveOrigin::Exit,
                    _ => panic!("origin"),
                },
                head: PersistentNodeId::new(value["head"].as_u64().unwrap() as u128).unwrap(),
            })
        }
    };
    let metadata = ArchiveMetadata {
        last_commit: receipt(&m["latest"]),
        manual: receipt(&m["manual"]),
        protected: receipt(&m["protected"]),
        extensions: m["extensions"]
            .as_array()
            .unwrap()
            .iter()
            .map(|e| ArchiveExtension {
                name: e["name"].as_str().unwrap().into(),
                bytes: bytes(e["bytes"].as_str().unwrap()),
            })
            .collect(),
        ..ArchiveMetadata::default()
    };
    assert_eq!(
        hex(&support::metadata(&metadata)),
        v["metadata_field_case"]["domain"]["sha256"]
            .as_str()
            .unwrap()
    );
}

#[cfg(windows)]
#[test]
fn original_byte_limit_accepts_the_actual_total_and_rejects_one_less() {
    let (dir, _d, req) = golden();
    let original = std::fs::read(dir.file()).unwrap();
    for limit in [17, 16] {
        let scratch = Directory::new();
        let limits = FileFactsLimits {
            original_bytes: limit,
            ..FileFactsLimits::default()
        };
        let mut reader = FileFactsReader::open(&scratch.path, limits).unwrap();
        let result = reader.inspect(&dir.file(), req, CaptureCancellation::default());
        if limit == 17 {
            assert_eq!(result.unwrap().summary().original_bytes, 17);
        } else {
            assert!(matches!(
                result,
                Err(FileFactsError::Limit(FileFactsLimit::Validation))
            ));
        }
        assert_eq!(reader.usage().scratch_reserved_bytes, 0);
        assert_eq!(std::fs::read(dir.file()).unwrap(), original);
    }
}

#[cfg(windows)]
#[test]
fn cleanup_retry_and_reader_drop_never_delete_a_replacement_at_the_old_scratch_path() {
    let (dir, _d, req) = golden();
    let scratch = Directory::new();
    let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    support::cleanup(&mut reader, true);
    assert!(
        reader
            .inspect(&dir.file(), req, CaptureCancellation::default())
            .is_err()
    );
    let path = std::fs::read_dir(&scratch.path)
        .unwrap()
        .map(|e| e.unwrap().path())
        .find(|p| p.extension().is_some_and(|s| s == "scratch"))
        .unwrap();
    let moved = scratch.path.join("retained-original");
    std::fs::rename(&path, &moved).unwrap();
    std::fs::write(&path, b"replacement must remain").unwrap();
    let charged = reader.usage().scratch_reserved_bytes;
    support::cleanup(&mut reader, false);
    assert!(matches!(
        reader.retry_cleanup(),
        Err(FileFactsError::Cleanup { .. })
    ));
    assert_eq!(reader.usage().scratch_reserved_bytes, charged);
    assert!(moved.exists());
    drop(reader);
    assert_eq!(std::fs::read(path).unwrap(), b"replacement must remain");
    assert!(moved.exists());
}

#[cfg(windows)]
#[test]
fn multiple_observations_share_one_control_budget_and_survive_reader_drop() {
    let (first, _d, req) = golden();
    let (second, _e, other) = golden_variant("Unknown:雨!\r\n");
    for less in [false, true] {
        let scratch = Directory::new();
        let limits = FileFactsLimits {
            control_bytes: support::aggregate_controls(2) - usize::from(less),
            ..FileFactsLimits::default()
        };
        let mut reader = FileFactsReader::open(&scratch.path, limits).unwrap();
        let a = reader
            .inspect(&first.file(), req, CaptureCancellation::default())
            .unwrap();
        let b = reader.inspect(&second.file(), other, CaptureCancellation::default());
        if less {
            assert!(matches!(
                b,
                Err(FileFactsError::Limit(FileFactsLimit::Control))
            ));
            assert_eq!(a.commits(page(a.summary(), 0, 1)).unwrap().len, 1);
            drop(a);
            drop(
                reader
                    .inspect(&second.file(), other, CaptureCancellation::default())
                    .unwrap(),
            );
        } else {
            let b = b.unwrap();
            assert_ne!(a.summary().proof.content, b.summary().proof.content);
            assert_eq!(a.summary().nodes, b.summary().nodes);
            assert_eq!(
                b.commits(page(a.summary(), 0, 1)),
                Err(FileFactsError::StalePage)
            );
            drop(reader);
            assert!(
                std::fs::OpenOptions::new()
                    .write(true)
                    .open(first.file())
                    .is_err()
            );
            drop(a);
            assert!(
                std::fs::OpenOptions::new()
                    .write(true)
                    .open(first.file())
                    .is_ok()
            );
            drop(b);
        }
    }
}

#[cfg(windows)]
#[test]
fn manual_absence_requires_explicit_latest_and_does_not_manufacture_manual_facts() {
    let dir = Directory::new();
    let store = BlobStore::default();
    let mut d = Driver::new(store.clone());
    d.set("Exit only\r\n");
    let (file, _) = SqliteProjectFile::publish_new(
        dir.file(),
        store,
        FileLimits::default(),
        &mut d.capture(),
        request(400, None, SaveOrigin::Exit),
    )
    .unwrap();
    let mut req = FileFactsRequest {
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
            project: file.cache.header.project,
            phase: SaveTargetPhase::ExistingVerified,
            history: Some((&file.cache.header.identity).into()),
            expected: ExpectedFileCommit {
                commit: Some(CommitId::new(400).unwrap()),
                generation: Some(1),
            },
        },
        selection: RestoreSelection::Manual,
    };
    drop(file);
    let before = std::fs::read(dir.file()).unwrap();
    let scratch = Directory::new();
    let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    assert!(matches!(
        reader.inspect(&dir.file(), req, CaptureCancellation::default()),
        Err(FileFactsError::NeedsExplicitRecovery)
    ));
    req.selection = RestoreSelection::Latest;
    let view = reader
        .inspect(&dir.file(), req, CaptureCancellation::default())
        .unwrap();
    assert_eq!(view.summary().files.manual, None);
    assert_eq!(
        view.summary().selected_head,
        view.summary().files.protected.unwrap().head
    );
    drop(view);
    assert_eq!(std::fs::read(dir.file()).unwrap(), before);
}

#[cfg(windows)]
fn branched_golden() -> (Directory, Driver, FileFactsRequest) {
    let (dir, mut d, mut req) = golden();
    let (mut file, _) = open(&dir.file(), BlobStore::default(), RestoreSelection::Manual);
    // Move the real Manual point to B first. Otherwise saving the old Manual A
    // correctly returns NoChange, and cannot be used as a new commit fixture.
    file.write_capture(
        &mut d.capture(),
        request(402, Some(401), SaveOrigin::Manual),
    )
    .unwrap();
    d.nav(HistoryAction::Checkout {
        node: NodeId::new(3).unwrap(),
    });
    file.write_capture(
        &mut d.capture(),
        request(403, Some(402), SaveOrigin::Manual),
    )
    .unwrap();
    d.set("D branch only 雪\r\n");
    file.write_capture(&mut d.capture(), request(404, Some(403), SaveOrigin::Exit))
        .unwrap();
    req.binding.expected = ExpectedFileCommit {
        commit: Some(CommitId::new(404).unwrap()),
        generation: Some(5),
    };
    drop(file);
    (dir, d, req)
}

#[cfg(windows)]
#[test]
fn full_branch_default_manual_proof_includes_the_independently_expected_derived_redo() {
    let (dir, d, req) = branched_golden();
    let before = d.session.snapshot();
    let scratch = Directory::new();
    let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    let view = reader
        .inspect(&dir.file(), req, CaptureCancellation::default())
        .unwrap();
    let s = view.summary();
    assert_eq!(s.nodes, 5);
    assert_eq!(s.files.manual.unwrap().commit.get(), 403);
    assert_eq!(s.files.protected.unwrap().commit.get(), 404);
    let fields = vec![
        s.proof.content.to_vec(),
        s.proof.originals.to_vec(),
        s.proof.metadata.to_vec(),
        s.proof.request.to_vec(),
        1u64.to_be_bytes().to_vec(),
        s.selected_head.get().to_be_bytes().to_vec(),
        1u64.to_be_bytes().to_vec(),
        1006u128.to_be_bytes().to_vec(),
        1007u128.to_be_bytes().to_vec(),
    ];
    assert_eq!(s.proof.baseline, support::framing(5, &fields));
    assert_eq!(d.session.snapshot(), before);
}

#[cfg(windows)]
#[test]
fn bad_unselected_branch_original_or_commit_chain_never_yields_partial_facts() {
    for (table, sql) in [
        (
            "nodes",
            "UPDATE nodes SET checksum=zeroblob(32) WHERE id=(SELECT node_id FROM addresses ORDER BY alias DESC LIMIT 1)",
        ),
        (
            "blob_chunks",
            "UPDATE blob_chunks SET bytes=zeroblob(length(bytes))",
        ),
        (
            "commits",
            "UPDATE commits SET request_hash=zeroblob(32) WHERE generation=(SELECT max(generation) FROM commits)",
        ),
    ] {
        let (dir, _d, req) = branched_golden();
        corrupt(&dir.file(), table, sql);
        let before = std::fs::read(dir.file()).unwrap();
        let scratch = Directory::new();
        let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
        let baseline = reader.usage().managed_bytes;
        assert!(matches!(
            reader.inspect(&dir.file(), req, CaptureCancellation::default()),
            Err(FileFactsError::ValidationFailed)
        ));
        assert_eq!(reader.usage().managed_bytes, baseline);
        assert_eq!(reader.usage().scratch_reserved_bytes, 0);
        assert_eq!(std::fs::read(dir.file()).unwrap(), before);
    }
}

#[cfg(windows)]
#[test]
fn full_page_binding_limits_and_observation_sequence_do_not_silently_wrap() {
    let (dir, _d, req) = golden();
    let scratch = Directory::new();
    let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    support::sequence(&mut reader, u64::MAX - 1);
    let view = reader
        .inspect(&dir.file(), req, CaptureCancellation::default())
        .unwrap();
    assert_eq!(view.summary().observation.sequence, u64::MAX);
    let mut wrong = page(view.summary(), 0, 1);
    wrong.binding.target = SaveTargetId::new(99).unwrap();
    assert_eq!(view.commits(wrong), Err(FileFactsError::StalePage));
    drop(view);
    assert!(matches!(
        reader.inspect(&dir.file(), req, CaptureCancellation::default()),
        Err(FileFactsError::Limit(FileFactsLimit::Sequence))
    ));
    for limits in [
        FileFactsLimits {
            nodes: 0,
            ..FileFactsLimits::default()
        },
        FileFactsLimits {
            file_bytes: 32 * 1024 * 1024 + 1,
            ..FileFactsLimits::default()
        },
        FileFactsLimits {
            commits: 129,
            ..FileFactsLimits::default()
        },
    ] {
        assert!(matches!(
            FileFactsReader::open(std::path::Path::new("does-not-exist"), limits),
            Err(FileFactsError::InvalidLimits)
        ));
    }
    for path in [
        std::path::PathBuf::from("relative"),
        std::path::PathBuf::from(format!("C:\\{}", "x".repeat(1024))),
        std::path::PathBuf::from(format!("C:\\{}", vec!["a"; 17].join("\\"))),
        std::path::PathBuf::from("C:\\x\\..\\source"),
        std::path::PathBuf::from("\\\\server\\share\\x"),
    ] {
        assert_eq!(support::path(&path), Err(FileFactsError::Unsupported));
    }
}

#[cfg(windows)]
#[test]
fn parent_junction_is_rejected_without_following_it_or_mutating_the_source() {
    use std::os::windows::process::CommandExt;
    let (dir, _d, req) = golden();
    let links = Directory::new();
    let scratch = Directory::new();
    let junction = links.path.join("link");
    let result=std::process::Command::new("powershell.exe").args(["-NoProfile","-NonInteractive","-Command","New-Item -ItemType Junction -Path $env:SKIN_EDITOR_TEST_LINK -Target $env:SKIN_EDITOR_TEST_TARGET -ErrorAction Stop | Out-Null"]).env("SKIN_EDITOR_TEST_LINK",&junction).env("SKIN_EDITOR_TEST_TARGET",&dir.path).creation_flags(0x0800_0000).output().unwrap();
    assert!(
        result.status.success(),
        "self-owned junction fixture creation failed"
    );
    let before = std::fs::read(dir.file()).unwrap();
    let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    assert!(matches!(
        reader.inspect(
            &junction.join("test.oskproj"),
            req,
            CaptureCancellation::default()
        ),
        Err(FileFactsError::Unsupported)
    ));
    std::fs::remove_dir(junction).unwrap();
    assert_eq!(std::fs::read(dir.file()).unwrap(), before);
}
