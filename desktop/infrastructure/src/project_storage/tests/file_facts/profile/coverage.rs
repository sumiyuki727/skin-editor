use super::*;

pub(super) fn check_case(name: &str, evidence: FileNoChangeCoverage) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/P3_2B1_2J_CASES.json");
    let matrix: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let row = matrix["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v[0] == name)
        .unwrap();
    assert_eq!(
        row[1].as_str().unwrap().to_ascii_lowercase(),
        save_name(evidence.context.ticket.origin)
    );
    let covered = evidence.history_equal
        && evidence.originals_equal
        && evidence.counters_equal
        && evidence.metadata_equal;
    assert_eq!(Some(covered), row[2].as_bool(), "{name}");
    assert_eq!(
        evidence.manual_navigation_equal.unwrap_or(false),
        row[3].as_bool().unwrap(),
        "{name}"
    );
    assert_eq!(
        Some(evidence.protection_navigation_equal),
        row[4].as_bool(),
        "{name}"
    );
    assert_eq!(Some(evidence.unchanged_opening), row[5].as_bool(), "{name}");
    for (i, basis) in [
        SaveNoChangeBasis::CurrentManual,
        SaveNoChangeBasis::CurrentProtection,
        SaveNoChangeBasis::VerifiedOpening,
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(
            evidence.matches_basis(basis),
            row[6 + i].as_bool(),
            "{name} {basis:?}"
        );
    }
    assert_eq!(
        evidence.conclusion,
        FileCaptureConclusion::InsufficientForSaveReconciliation
    );
}

#[test]
fn manual_opening_uses_derived_redo_and_retains_both_file_positions() {
    for branched in [false, true] {
        let (dir, _, req) = if branched {
            branched_golden()
        } else {
            golden()
        };
        let bytes = std::fs::read(dir.file()).unwrap();
        let scratch = Directory::new();
        let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
        let store = BlobStore::default();
        let (d, req) = restore(&mut reader, &dir, req, &store);
        let (mut d, _port, work, req) = save::attach_work(d, req);
        let ticket = work.ticket();
        let q = reader
            .prepare_save(work, req.selection, &store, CaptureCancellation::default())
            .unwrap();
        let view = reader
            .inspect(&dir.file(), req, CaptureCancellation::default())
            .unwrap();
        let c = context(&view, &q, ticket.target.expected.commit.unwrap().get());
        let compare = || {
            reader
                .compare_no_change_coverage(&view, &q, c, CaptureCancellation::default())
                .unwrap()
                .unwrap()
        };
        let result = compare();
        check_case(
            if branched {
                "branch_derived_redo"
            } else {
                "linear_manual_A_exit_C"
            },
            result,
        );
        assert_ne!(
            result.files.manual.unwrap().head,
            result.files.protected.unwrap().head
        );
        assert_eq!(result.files, view.summary().files);
        assert_eq!(result.origin.opened_head, ticket.head);
        assert_eq!(result.origin.file.key.sequence, 9);
        assert_eq!(c.file.key.sequence, 1);
        if !branched {
            // Counterfactual intent on the same detached output; no Exit Work is issued.
            let mut example = result;
            example.context.ticket.origin = SaveOrigin::Exit;
            check_case("default_A_protected_C", example);
        }
        d.set("edit after q was captured");
        let late = d.session.snapshot();
        assert_eq!(compare(), result);
        assert_eq!(d.session.snapshot(), late);
        assert_ne!(d.session.save_source(), ticket.source);
        drop(view);
        assert_eq!(std::fs::read(dir.file()).unwrap(), bytes);
        // The writer gets an independently restored copy with exactly q's original navigation.
        let copy = Directory::new();
        std::fs::write(copy.file(), &bytes).unwrap();
        let mut writer = save::load_driver(&copy.file(), &store, 90);
        let (mut file, _) = open(&copy.file(), store.clone(), req.selection);
        let written = file
            .write_capture(
                &mut writer.capture(),
                request(
                    ticket.commit.get(),
                    ticket.target.expected.commit.map(CommitId::get),
                    ticket.origin,
                ),
            )
            .unwrap();
        drop(file);
        if branched {
            assert!(
                matches!(written,ArchiveWriteOutcome::Committed(ref r) if r.commit==ticket.commit && r.head==ticket.head)
            );
            let mut after = req;
            after.binding.expected = ExpectedFileCommit {
                commit: Some(ticket.commit),
                generation: Some(6),
            };
            let observed = reader
                .inspect(&copy.file(), after, CaptureCancellation::default())
                .unwrap();
            let saved = observed
                .compare_save(
                    &q,
                    FileSaveCompareContext {
                        version: FILE_SAVE_INTENT_VERSION,
                        ticket,
                        observation: observed.summary().observation,
                    },
                )
                .unwrap()
                .commit
                .unwrap();
            assert!(saved.legacy_fields_equal && saved.generation_equal && saved.navigation_equal);
        } else {
            assert!(
                matches!(written,ArchiveWriteOutcome::NoChange(ref r) if Some(SaveCommitFact::from(r))==result.files.protected)
            );
            assert_eq!(std::fs::read(copy.file()).unwrap(), bytes);
        }
        assert_eq!(std::fs::read(dir.file()).unwrap(), bytes);
    }
}

#[test]
fn navigation_round_trip_and_additional_history_at_the_same_manual_head_are_distinct() {
    for new_asset in [false, true] {
        let (dir, req) = copy_golden_file();
        let scratch = Directory::new();
        let store = BlobStore::default();
        let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
        let (mut d, req) = restore(&mut reader, &dir, req, &store);
        if new_asset {
            let raw = store
                .stage_bytes(b"unreferenced new original".to_vec())
                .unwrap();
            d.command(vec![Command::RegisterAsset {
                asset: Asset::new(
                    AssetId::new(999).unwrap(),
                    "unused".into(),
                    raw.reference(),
                    AssetKind::Opaque,
                )
                .unwrap(),
            }]);
        } else {
            d.nav(HistoryAction::Checkout {
                node: NodeId::new(4).unwrap(),
            });
        }
        d.nav(HistoryAction::Checkout {
            node: NodeId::new(3).unwrap(),
        });
        let (_d, _port, work, req) = save::attach_work(d, req);
        let q = reader
            .prepare_save(work, req.selection, &store, CaptureCancellation::default())
            .unwrap();
        let view = reader
            .inspect(&dir.file(), req, CaptureCancellation::default())
            .unwrap();
        let result = reader
            .compare_no_change_coverage(
                &view,
                &q,
                context(&view, &q, 401),
                CaptureCancellation::default(),
            )
            .unwrap()
            .unwrap();
        check_case(
            if new_asset {
                "new_branch_then_A"
            } else {
                "linear_checkout_C_then_A"
            },
            result,
        );
        assert_eq!(result.history_equal, !new_asset);
        assert_eq!(result.originals_equal, !new_asset);
        assert!(result.metadata_equal);
        if !new_asset {
            let mut example = result;
            example.context.ticket.origin = SaveOrigin::Exit;
            check_case("navigation_returned_to_A", example);
        }
    }
}

#[test]
fn explicit_latest_without_manual_is_protected_but_cannot_satisfy_a_manual_request() {
    let (dir, mut req) = exit_only_file();
    req.selection = RestoreSelection::Latest;
    let scratch = Directory::new();
    let store = BlobStore::default();
    let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    let (d, req) = restore(&mut reader, &dir, req, &store);
    let (_d, _port, work, req) = save::attach_work(d, req);
    let q = reader
        .prepare_save(work, req.selection, &store, CaptureCancellation::default())
        .unwrap();
    let view = reader
        .inspect(&dir.file(), req, CaptureCancellation::default())
        .unwrap();
    let result = reader
        .compare_no_change_coverage(
            &view,
            &q,
            context(&view, &q, 400),
            CaptureCancellation::default(),
        )
        .unwrap()
        .unwrap();
    assert_eq!(result.manual_navigation_equal, None);
    check_case("explicit_latest_C_no_manual", result);
    // Pure intent classification of held facts, not an admitted Exit q.
    let mut example = result;
    example.context.ticket.origin = SaveOrigin::Exit;
    assert_eq!(
        example.matches_basis(SaveNoChangeBasis::VerifiedOpening),
        Some(true)
    );
    example.unchanged_opening = false;
    check_case("protection_C", example);
}

#[test]
fn original_open_selection_is_required_even_when_both_choices_open_the_same_checkpoint() {
    let (dir, req, _) = historical_metadata::profile_fixture(None, None, false);
    let scratch = Directory::new();
    let store = BlobStore::default();
    let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    let (d, mut req) = restore(&mut reader, &dir, req, &store);
    assert_eq!(req.selection, RestoreSelection::Manual);
    req.selection = RestoreSelection::Latest;
    let (_d, _port, work, req) = save::attach_work(d, req);
    let q = reader
        .prepare_save(work, req.selection, &store, CaptureCancellation::default())
        .unwrap();
    let view = reader
        .inspect(&dir.file(), req, CaptureCancellation::default())
        .unwrap();
    let c = context(&view, &q, 2);
    let e = reader
        .compare_no_change_coverage(&view, &q, c, CaptureCancellation::default())
        .unwrap()
        .unwrap();
    assert_eq!(e.origin.opened_head, view.summary().selected_head);
    assert_eq!(
        q.summary().navigation,
        view.intent_facts().opening_navigation
    );
    assert_eq!(
        (
            c.ticket.source.state.revision,
            c.ticket.source.navigation_revision
        ),
        (0, 0)
    );
    assert_eq!(
        e.matches_basis(SaveNoChangeBasis::CurrentManual),
        Some(true)
    );
    assert_eq!(
        e.matches_basis(SaveNoChangeBasis::VerifiedOpening),
        Some(false)
    );
}

#[test]
fn full_commit_table_and_exact_workspace_still_allow_read_only_manual_coverage() {
    let (dir, mut author, mut req) = golden();
    let store = BlobStore::default();
    let (mut file, _) = open(&dir.file(), store.clone(), RestoreSelection::Manual);
    let mut previous = 401;
    for generation in 3..=128 {
        let head = if generation % 2 == 1 { 4 } else { 3 };
        author.nav(HistoryAction::Checkout {
            node: NodeId::new(head).unwrap(),
        });
        let id = 10000 + generation;
        assert!(
            matches!(file.write_capture(&mut author.capture(),request(id,Some(previous),SaveOrigin::Manual)).unwrap(),ArchiveWriteOutcome::Committed(r) if u128::from(r.generation)==generation)
        );
        previous = id;
    }
    drop(file);
    req.binding.expected = ExpectedFileCommit {
        commit: CommitId::new(previous),
        generation: Some(128),
    };
    let bytes = std::fs::read(dir.file()).unwrap();
    let scratch = Directory::new();
    let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    let (d, req) = restore(&mut reader, &dir, req, &store);
    let (mut d, _port, work, req) = save::attach_work(d, req);
    let ticket = work.ticket();
    let q = reader
        .prepare_save(work, req.selection, &store, CaptureCancellation::default())
        .unwrap();
    let view = reader
        .inspect(&dir.file(), req, CaptureCancellation::default())
        .unwrap();
    let c = context(&view, &q, previous);
    assert_eq!(view.summary().commits, 128);
    assert_eq!(
        reader.project_fresh_append_metadata(&view, &q, c, CaptureCancellation::default()),
        Err(FileFactsError::Limit(FileFactsLimit::Commits))
    );
    let kept = reader.usage().managed_bytes;
    for short in [1, 0] {
        let holder = reader.hold_coverage_workspace_margin(short);
        let held = reader.usage().managed_bytes;
        let result =
            reader.compare_no_change_coverage(&view, &q, c, CaptureCancellation::default());
        if short == 1 {
            assert_eq!(result, Err(FileFactsError::Limit(FileFactsLimit::Control)));
        } else {
            let evidence = result.unwrap().unwrap();
            assert_eq!(
                evidence.matches_basis(SaveNoChangeBasis::CurrentManual),
                Some(true)
            );
            // Independently remove each necessary fact on detached values.
            for mut incomplete in [
                FileNoChangeCoverage {
                    history_equal: false,
                    ..evidence
                },
                FileNoChangeCoverage {
                    originals_equal: false,
                    ..evidence
                },
                FileNoChangeCoverage {
                    metadata_equal: false,
                    ..evidence
                },
                FileNoChangeCoverage {
                    counters_equal: false,
                    ..evidence
                },
            ] {
                assert_eq!(
                    incomplete.matches_basis(SaveNoChangeBasis::CurrentManual),
                    Some(false)
                );
                assert_eq!(
                    incomplete.matches_basis(SaveNoChangeBasis::VerifiedOpening),
                    Some(false)
                );
                incomplete.context.ticket.origin = SaveOrigin::Exit;
                assert_eq!(
                    incomplete.matches_basis(SaveNoChangeBasis::CurrentProtection),
                    Some(false)
                );
                assert_eq!(
                    incomplete.matches_basis(SaveNoChangeBasis::VerifiedOpening),
                    Some(false)
                );
            }
            let mut example = evidence;
            example.context.ticket.origin = SaveOrigin::Automatic;
            check_case("automatic_not_admitted", example);
        }
        assert_eq!(reader.usage().managed_bytes, held);
        drop(holder);
        assert_eq!(reader.usage().managed_bytes, kept);
    }
    support::probe(
        &mut reader,
        Probe {
            cancel_at: Some(Stage::ReadNoChangeCoverage),
            ..Default::default()
        },
    );
    assert_eq!(
        reader.compare_no_change_coverage(&view, &q, c, CaptureCancellation::default()),
        Err(FileFactsError::Cancelled)
    );
    assert_eq!(reader.usage().managed_bytes, kept);
    support::probe(&mut reader, Probe::default());
    let mut stale = c;
    stale.ticket.source.navigation_revision += 1;
    assert_eq!(
        reader.compare_no_change_coverage(&view, &q, stale, CaptureCancellation::default()),
        Err(FileFactsError::StalePage)
    );
    let other_root = Directory::new();
    let other = FileFactsReader::open(&other_root.path, FileFactsLimits::default()).unwrap();
    assert_eq!(
        other.compare_no_change_coverage(&view, &q, c, CaptureCancellation::default()),
        Err(FileFactsError::StalePage)
    );
    assert!(
        reader
            .compare_no_change_coverage(&view, &q, c, CaptureCancellation::default())
            .unwrap()
            .unwrap()
            .unchanged_opening
    );
    assert_eq!(reader.usage().managed_bytes, kept);
    drop(view);
    drop(q);
    assert_eq!(
        d.session.acknowledge_save(ticket),
        Ok(SaveAcknowledgement::Released)
    );
    drop(d);
    assert_eq!(reader.usage().managed_bytes, 16384);
    registered_coverage::check_full_table(&mut reader, &dir, req, &store);
    assert_eq!(reader.usage().managed_bytes, 16384);
    assert_eq!(std::fs::read(dir.file()).unwrap(), bytes);
}
