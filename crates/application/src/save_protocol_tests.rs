//! Deterministic A2 adapter fixtures. No production authority, file, clock,
//! device, thread, global test allocator or fabricated file-format proof.
use crate::save_coordination::{
    synthetic_authority, test_candidate_weak, test_query_allocation, test_query_attempt,
};
use crate::values::*;
use crate::*;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

#[derive(Clone, Default)]
struct Pool(BTreeMap<BlobRef, Arc<Original>>);
struct Original {
    blob: BlobRef,
    bytes: Vec<u8>,
}
impl BlobLease for Original {
    fn reference(&self) -> BlobRef {
        self.blob
    }
}
impl ResourceStore for Pool {
    fn pin(&self, blob: BlobRef) -> Result<Arc<dyn BlobLease>, ResourceError> {
        self.0
            .get(&blob)
            .map(|p| p.clone() as Arc<dyn BlobLease>)
            .ok_or(ResourceError::Missing)
    }
}
struct Ids(u128);
impl HistoryIdentitySource for Ids {
    fn next_identity(&mut self) -> Result<u128, IdentityError> {
        self.0 += 1;
        Ok(self.0)
    }
}
fn limits() -> SessionLimits {
    SessionLimits {
        history_nodes: 64,
        retained_requests: 128,
        commands_per_request: 16,
    }
}
fn record(text: &str) -> ConfigRecord {
    ConfigRecord::new(text.as_bytes().to_vec(), ConfigInterpretation::Opaque).unwrap()
}
fn config(s: &ProjectSession) -> SavePreparationConfig {
    let metadata = s.archive_metadata.as_ref();
    let last = metadata.last_commit.as_ref();
    SavePreparationConfig {
        version: 1,
        binding: SaveTargetBinding {
            version: 1,
            key: SaveBindingKey {
                id: SaveBindingId::new(10).unwrap(),
                generation: 1,
            },
            target: SaveTargetId::new(20).unwrap(),
            project: ProjectId::new(s.project_info().project_id).unwrap(),
            phase: if last.is_some() {
                SaveTargetPhase::ExistingVerified
            } else {
                SaveTargetPhase::NewUnpublished
            },
            history: last.and_then(|_| {
                s.persistence
                    .as_ref()
                    .map(|p| SaveHistoryScope::from(&p.identity))
            }),
            expected: ExpectedFileCommit {
                commit: last.map(|r| r.commit),
                generation: last.map(|r| r.generation),
            },
        },
        control: SaveControlProfile::default(),
        capture: save_fixture_capture_limits(),
    }
}
fn request(s: &ProjectSession) -> SaveRequest {
    SaveRequest {
        api_version: API_VERSION,
        capture_version: HISTORY_CAPTURE_VERSION,
        key: s.save_coordination_status(1).unwrap().next_key.unwrap(),
        source: s.save_source(),
        binding: config(s).binding.key,
        origin: SaveOrigin::Manual,
    }
}

struct Driver {
    session: ProjectSession,
    pool: Pool,
    next: u128,
    ids: Ids,
    expected: BTreeMap<u64, SkinProject>,
    file: Option<Golden>,
}
impl Driver {
    fn new() -> Self {
        Self::with_pool(Pool::default())
    }
    fn with_pool(pool: Pool) -> Self {
        let session =
            ProjectSession::with_resources(7, 70, Box::new(pool.clone()), limits()).unwrap();
        let expected = BTreeMap::from([(1, session.snapshot().project)]);
        Self {
            session,
            pool,
            next: 1,
            ids: Ids(1000),
            expected,
            file: None,
        }
    }
    fn edit(&mut self, commands: Vec<Command>) -> NodeId {
        let i = self.session.project_info();
        let result = self
            .session
            .submit(CommandRequest {
                api_version: API_VERSION,
                session_id: i.session_id,
                request_id: RequestId::new(self.next).unwrap(),
                expected: i.state,
                commands,
            })
            .unwrap();
        self.next += 1;
        self.expected
            .insert(result.state.head.get(), self.session.snapshot().project);
        result.state.head
    }
    fn set(&mut self, text: &str) -> NodeId {
        let command = if self
            .session
            .snapshot()
            .project
            .configuration()
            .records()
            .is_empty()
        {
            Command::InsertConfigRecord {
                index: 0,
                record: record(text),
            }
        } else {
            Command::SetConfigRecord {
                index: 0,
                record: record(text),
            }
        };
        self.edit(vec![command])
    }
    fn nav(&mut self, action: HistoryAction) {
        let i = self.session.project_info();
        self.session
            .navigate(HistoryRequest {
                api_version: API_VERSION,
                session_id: i.session_id,
                request_id: RequestId::new(self.next).unwrap(),
                expected: i.state,
                action,
            })
            .unwrap();
        self.next += 1;
    }
    fn install(&mut self, candidate: SaveCandidateLimits, opening: bool) -> SaveProtocolPort {
        let authority = synthetic_authority(&self.session, opening);
        self.session
            .install_save_protocol(config(&self.session), candidate, authority)
            .unwrap()
    }
    fn admit(&mut self) -> SaveTicket {
        self.session
            .request_save(request(&self.session), &mut self.ids)
            .unwrap()
            .ticket()
    }
    fn advance(&mut self) {
        self.session.advance_save_coordination(1).unwrap();
    }
    fn status(&self) -> SaveProtocolStatus {
        self.session.save_protocol_status(1).unwrap()
    }
    fn active(&self) -> SaveProtocolActive {
        self.status().active.unwrap()
    }
    fn golden(&self) -> Golden {
        let source = self.session.save_source();
        let parents = self
            .expected
            .keys()
            .map(|alias| {
                let e = self
                    .session
                    .history_entry(source.session_id, NodeId::new(*alias).unwrap())
                    .unwrap();
                (*alias, e.parent.map(|n| n.get()))
            })
            .collect();
        Golden {
            source,
            states: self.expected.clone(),
            parents,
            originals: self
                .pool
                .0
                .iter()
                .map(|(b, p)| (*b, p.bytes.clone()))
                .collect(),
            metadata: self.session.archive_metadata.clone(),
            admissions: self.session.used_ids.clone(),
            redo: self
                .session
                .redo_preferences
                .iter()
                .map(|(a, b)| (a.get(), b.get()))
                .collect(),
            manual_alias: self
                .session
                .archive_metadata
                .manual
                .as_ref()
                .and_then(|manual| {
                    self.session
                        .persistence
                        .as_ref()?
                        .nodes
                        .iter()
                        .find(|(_, p)| **p == manual.head)
                        .map(|(n, _)| n.get())
                }),
        }
    }
}

#[derive(Clone)]
struct Golden {
    source: CaptureSource,
    states: BTreeMap<u64, SkinProject>,
    parents: BTreeMap<u64, Option<u64>>,
    originals: BTreeMap<BlobRef, Vec<u8>>,
    metadata: Arc<ArchiveMetadata>,
    admissions: BTreeSet<ObjectId>,
    redo: Vec<(u64, u64)>,
    manual_alias: Option<u64>,
}

fn verify_no_change_file(
    q: &Golden,
    file: &Golden,
    basis: SaveNoChangeBasis,
) -> Result<(), &'static str> {
    if q.states != file.states
        || q.parents != file.parents
        || q.originals != file.originals
        || q.admissions != file.admissions
        || q.metadata != file.metadata
    {
        return Err("full file coverage");
    }
    if q.manual_alias != Some(q.source.state.head.get()) || q.redo != file.redo {
        return Err("manual/navigation intent");
    }
    match basis {
        SaveNoChangeBasis::VerifiedOpening if q.source == file.source => Ok(()),
        SaveNoChangeBasis::CurrentManual
            if q.source.state.head == file.source.state.head
                && file.metadata.last_commit.as_ref().map(|r| r.head)
                    == file.metadata.manual.as_ref().map(|r| r.head) =>
        {
            Ok(())
        }
        _ => Err("opening/file navigation basis"),
    }
}
// Bounded synthetic commitments supplement (never replace) the independent
// whole-value comparison below. Debug spelling is deliberately test-profile
// material, not a persistent encoding, real digest algorithm or security proof.
struct SyntheticCommitment([u64; 4]);
impl SyntheticCommitment {
    fn new() -> Self {
        Self([17, 43, 101, 211])
    }
    fn absorb(&mut self, bytes: &[u8]) {
        for byte in bytes {
            for (lane, value) in self.0.iter_mut().enumerate() {
                *value = value
                    .rotate_left(5 + lane as u32)
                    .wrapping_mul(0x100000001b3 + lane as u64 * 2)
                    .wrapping_add(u64::from(*byte) + 1);
            }
        }
        for value in &mut self.0 {
            *value ^= bytes.len() as u64;
        }
    }
    fn finish(self) -> [u8; 32] {
        let mut result = [0; 32];
        for (chunk, value) in result.chunks_exact_mut(8).zip(self.0) {
            chunk.copy_from_slice(&value.to_le_bytes());
        }
        result
    }
}
fn synthetic_commitment(bytes: &[u8]) -> [u8; 32] {
    let mut value = SyntheticCommitment::new();
    value.absorb(bytes);
    value.finish()
}
// An independent fixture comparison, not a production hash or file codec.
// It streams actual captured states and reads the self-authored original pool.
fn prepare(
    work: &mut SaveExecution,
    expected: &Golden,
    pool: &Pool,
) -> Result<PreparedWitness, &'static str> {
    let m = work.snapshot().manifest();
    if m.source != expected.source
        || m.nodes.len() != expected.states.len()
        || m.archive_metadata.as_ref() != expected.metadata.as_ref()
    {
        return Err("source/metadata");
    }
    let aliases: BTreeMap<_, _> = m.nodes.iter().map(|n| (n.id, n.alias)).collect();
    if aliases[&m.head] != expected.source.state.head.get()
        || m.nodes
            .iter()
            .any(|n| expected.parents.get(&n.alias).copied() != Some(n.parent.map(|p| aliases[&p])))
    {
        return Err("relations/head");
    }
    if m.admissions
        .iter()
        .map(|a| a.object)
        .collect::<BTreeSet<_>>()
        != expected.admissions
    {
        return Err("admissions");
    }
    if m.redo_preferences
        .iter()
        .map(|(a, b)| (aliases[a], aliases[b]))
        .collect::<Vec<_>>()
        != expected.redo
    {
        return Err("navigation");
    }
    if m.originals != expected.originals.keys().copied().collect::<Vec<_>>() {
        return Err("original scope");
    }
    let mut original_commitment = SyntheticCommitment::new();
    for blob in &m.originals {
        if pool.0.get(blob).map(|p| &p.bytes) != expected.originals.get(blob) {
            return Err("original bytes");
        }
        original_commitment.absorb(format!("{blob:?}").as_bytes());
        original_commitment.absorb(&pool.0[blob].bytes);
    }
    let mut content = SyntheticCommitment::new();
    content.absorb(
        format!(
            "{:?}{:?}{:?}{:?}{:?}{}{}",
            m.identity, m.nodes, m.admissions, m.root, m.head, m.next_alias, m.next_branch
        )
        .as_bytes(),
    );
    let metadata = synthetic_commitment(
        format!("{:?}{:?}", m.archive_metadata, m.redo_preferences).as_bytes(),
    );
    let request = synthetic_commitment(format!("{:?}", work.ticket()).as_bytes());
    let baseline = synthetic_commitment(
        format!(
            "{:?}{:?}{:?}",
            m.source,
            m.archive_metadata,
            work.ticket().target.expected
        )
        .as_bytes(),
    );
    let mut seen = [false; 64];
    loop {
        match work.poll(1).map_err(|_| "capture")? {
            CapturePoll::Pending => (),
            CapturePoll::Node(node) => {
                let alias = node.info().alias;
                if alias == 0
                    || alias > 64
                    || seen[alias as usize - 1]
                    || expected.states.get(&alias) != Some(node.project())
                {
                    return Err("frozen state");
                }
                seen[alias as usize - 1] = true;
                content.absorb(format!("{:?}{:?}", node.info(), node.project()).as_bytes());
            }
            CapturePoll::Complete => break,
        }
    }
    if seen.iter().filter(|s| **s).count() != expected.states.len() {
        return Err("state coverage");
    }
    work.publish_witness(PreparedWitnessInfo {
        version: 1,
        ticket: work.ticket(),
        capture_version: 3,
        proof_profile: u16::MAX,
        content: content.finish(),
        originals: original_commitment.finish(),
        metadata,
        request,
        baseline,
    })
    .map_err(|_| "publication")
}
fn fact(work: &SaveExecution) -> SaveCommitFact {
    SaveCommitFact::from(
        work.candidate_metadata()
            .unwrap()
            .last_commit
            .as_ref()
            .unwrap(),
    )
}
fn diagnostic(code: SaveDiagnosticCode) -> SaveDiagnostic {
    SaveDiagnostic {
        code,
        numbers: [0; 4],
    }
}
fn committed(witness: PreparedWitness, fact: SaveCommitFact) -> SaveStorageObservation {
    SaveStorageObservation::Committed {
        witness,
        fact,
        binding_conflict: false,
    }
}

fn restored(manual: bool, selection: RestoreSelection, generation: u64) -> Driver {
    let mut d = Driver::new();
    let a = d.set("A opaque\r\n");
    let b = d.set("B\n");
    let c = d.set("C\r\n");
    d.nav(HistoryAction::Checkout { node: a });
    let branch = d.set("D branch\r\n");
    d.nav(HistoryAction::Checkout { node: c });
    let mut capture = d
        .session
        .capture_history(
            CaptureRequest {
                version: 3,
                session_id: d.session.project_info().session_id,
                expected: d.session.project_info().state,
                limits: save_fixture_capture_limits(),
                cancellation: CaptureCancellation::default(),
            },
            &mut d.ids,
        )
        .unwrap();
    let snapshot = capture.snapshot().unwrap();
    let m = snapshot.manifest();
    let persistent = |node: NodeId| m.nodes.iter().find(|n| n.alias == node.get()).unwrap().id;
    let manual_fact = ArchiveReceipt {
        commit: CommitId::new(41).unwrap(),
        generation: 1,
        head: persistent(a),
        origin: SaveOrigin::Manual,
    };
    let latest = ArchiveReceipt {
        commit: CommitId::new(42).unwrap(),
        generation,
        head: persistent(c),
        origin: if manual {
            SaveOrigin::Exit
        } else {
            SaveOrigin::Automatic
        },
    };
    let metadata = Arc::new(ArchiveMetadata {
        last_commit: Some(latest.clone()),
        manual: manual.then_some(manual_fact.clone()),
        protected: Some(latest),
        manual_marks: if manual {
            vec![(manual_fact.commit, manual_fact.head)]
        } else {
            vec![]
        },
        tags: vec![ArchiveTag {
            node: persistent(branch),
            name: "user:fixture".into(),
            value: "keep\r\n雪".into(),
        }],
        extensions: vec![ArchiveExtension {
            name: "future:opaque".into(),
            bytes: vec![0, 255, 13, 10, 7],
        }],
    });
    let (head, redo) = match selection {
        RestoreSelection::Manual => (persistent(a), vec![(persistent(a), persistent(b))]),
        RestoreSelection::Latest => (persistent(c), vec![]),
    };
    let mut builder = RestoreBuilder::new(
        RestoreHeader {
            node_count: m.nodes.len(),
            project: m.project_id,
            identity: m.identity.clone(),
            root: m.root,
            head,
            next_alias: m.next_alias,
            next_branch: m.next_branch,
            admissions: m.admissions.clone(),
            redo_preferences: redo,
            originals: m.originals.clone(),
            metadata,
        },
        RestoreLimits::default(),
        CaptureCancellation::default(),
    )
    .unwrap();
    loop {
        match capture.poll(1).unwrap() {
            CapturePoll::Pending => (),
            CapturePoll::Complete => break,
            CapturePoll::Node(node) => {
                let i = node.info();
                builder
                    .push(RestoreNode {
                        id: i.id,
                        parent: i.parent,
                        alias: i.alias,
                        coordinate: i.coordinate,
                        depth: i.depth,
                        operation: i.operation.clone(),
                        origin: Arc::new(OriginalNodeRecord {
                            record_version: 2,
                            state: [1; 32],
                            action: b"synthetic record".to_vec(),
                            checksum: [2; 32],
                        }),
                        project: node.project().clone(),
                    })
                    .unwrap();
            }
        }
    }
    let session = builder
        .finish(&d.pool)
        .unwrap()
        .into_session(
            HistorySessionId::new(71).unwrap(),
            Box::new(d.pool.clone()),
            limits(),
            CaptureCancellation::default(),
        )
        .unwrap();
    let mut result = Driver {
        session,
        pool: d.pool,
        next: 1,
        ids: Ids(5000),
        expected: d.expected,
        file: None,
    };
    result.file = Some(result.golden());
    result
}

fn publish_no_change(
    d: &mut Driver,
    port: &SaveProtocolPort,
    ticket: SaveTicket,
    basis: SaveNoChangeBasis,
) {
    let golden = d.golden();
    verify_no_change_file(&golden, d.file.as_ref().expect("synthetic file"), basis).unwrap();
    let mut work = d.session.take_save_execution(port, ticket).unwrap();
    let witness = prepare(&mut work, &golden, &d.pool).unwrap();
    let sender = work.take_completion().unwrap();
    sender
        .publish(SaveStorageObservation::NoChange {
            witness,
            files: SaveFileFacts::from(golden.metadata.as_ref()),
            basis,
        })
        .unwrap();
    d.advance();
    drop(work);
    drop(sender);
}

#[test]
fn default_manual_opening_no_change_preserves_newer_protection_and_derived_redo() {
    let mut d = restored(true, RestoreSelection::Manual, 2);
    let before = d.session.snapshot();
    let metadata = d.session.archive_metadata.clone();
    assert_eq!(d.session.redo_preferences.len(), 1);
    let port = d.install(SaveCandidateLimits::default(), true);
    assert_eq!(d.status().facts.protection, SaveProtectionStatus::Covered);
    let ticket = d.admit();
    let unused_candidate = test_candidate_weak(&d.session).unwrap();
    publish_no_change(&mut d, &port, ticket, SaveNoChangeBasis::VerifiedOpening);
    assert!(
        matches!(d.active().outcome,Some(SaveProtocolOutcome::Confirmed(c)) if c.kind==SaveConfirmationKind::NoChange && c.confirmed_commit.is_none())
    );
    assert!(Arc::ptr_eq(&metadata, &d.session.archive_metadata));
    assert_ne!(
        d.status().facts.files.manual.unwrap().head,
        d.status().facts.files.protected.unwrap().head
    );
    assert_eq!(d.session.snapshot(), before);
    assert_eq!(
        d.session.acknowledge_save(ticket).unwrap(),
        SaveAcknowledgement::Released
    );
    assert!(unused_candidate.upgrade().is_none());
    assert_eq!(d.session.archive_metadata.manual_marks.len(), 1);
}

#[test]
fn real_navigation_cannot_reuse_the_unchanged_opening_equivalence() {
    let mut d = restored(true, RestoreSelection::Manual, 2);
    let port = d.install(SaveCandidateLimits::default(), true);
    d.nav(HistoryAction::Checkout {
        node: NodeId::new(4).unwrap(),
    });
    d.nav(HistoryAction::Checkout {
        node: NodeId::new(2).unwrap(),
    });
    assert_eq!(
        d.status().facts.protection,
        SaveProtectionStatus::NeedsProtection
    );
    let old = d.status().facts.files;
    let ticket = d.admit();
    let golden = d.golden();
    assert!(
        verify_no_change_file(
            &golden,
            d.file.as_ref().unwrap(),
            SaveNoChangeBasis::VerifiedOpening
        )
        .is_err()
    );
    // Fault injection: the independently checked fake would reject this claim.
    let mut work = d.session.take_save_execution(&port, ticket).unwrap();
    let witness = prepare(&mut work, &golden, &d.pool).unwrap();
    let sender = work.take_completion().unwrap();
    sender
        .publish(SaveStorageObservation::NoChange {
            witness,
            files: SaveFileFacts::from(golden.metadata.as_ref()),
            basis: SaveNoChangeBasis::VerifiedOpening,
        })
        .unwrap();
    drop(work);
    drop(sender);
    d.advance();
    assert_eq!(d.active().phase, SaveProtocolPhase::Reconciling);
    assert_eq!(
        d.active().diagnostic.unwrap().code,
        SaveDiagnosticCode::AdapterFault
    );
    assert_eq!(d.status().facts.files, old);
    assert_eq!(
        d.status().facts.protection,
        SaveProtectionStatus::NeedsProtection
    );
}

#[test]
fn first_manual_after_latest_without_manual_requires_a_real_new_manual_fact() {
    let mut d = restored(false, RestoreSelection::Latest, 2);
    let golden = d.golden();
    let port = d.install(SaveCandidateLimits::default(), true);
    assert!(d.status().facts.files.manual.is_none());
    let ticket = d.admit();
    let mut work = d.session.take_save_execution(&port, ticket).unwrap();
    let witness = prepare(&mut work, &golden, &d.pool).unwrap();
    let receipt = fact(&work);
    let sender = work.take_completion().unwrap();
    work.begin_write().unwrap();
    sender.publish(committed(witness, receipt)).unwrap();
    d.advance();
    assert_eq!(d.status().facts.files.manual, Some(receipt));
    assert_eq!(receipt.generation, 3);
    assert_eq!(
        d.session.archive_metadata.manual_marks,
        vec![(ticket.commit, ticket.head)]
    );
    assert_eq!(
        d.session.archive_metadata.extensions,
        golden.metadata.extensions
    );
    drop(work);
    drop(sender);
    assert_eq!(
        d.session.acknowledge_save(ticket).unwrap(),
        SaveAcknowledgement::Released
    );
}

#[test]
fn existing_no_change_survives_candidate_limits_and_generation_exhaustion_without_write_permission()
{
    let mut exact = restored(true, RestoreSelection::Manual, 2);
    let _port = exact.install(SaveCandidateLimits::default(), true);
    exact.admit();
    let cost = exact.active().candidate.unwrap();
    for (generation, limits) in [
        (
            2,
            SaveCandidateLimits {
                logical_bytes: cost.logical_bytes - 1,
                allocation_bytes: SAVE_CANDIDATE_ALLOCATION_BYTES,
            },
        ),
        (
            2,
            SaveCandidateLimits {
                logical_bytes: SAVE_CANDIDATE_LOGICAL_BYTES,
                allocation_bytes: cost.allocation_bytes - 1,
            },
        ),
        (u64::MAX, SaveCandidateLimits::default()),
    ] {
        let mut d = restored(true, RestoreSelection::Manual, generation);
        let golden = d.golden();
        let old = d.session.archive_metadata.clone();
        let port = d.install(limits, true);
        let ticket = d.admit();
        assert_eq!(d.active().permission, SaveWritePermission::VerifyOnly);
        assert!(d.active().candidate.is_none());
        let mut work = d.session.take_save_execution(&port, ticket).unwrap();
        let witness = prepare(&mut work, &golden, &d.pool).unwrap();
        verify_no_change_file(
            &golden,
            d.file.as_ref().unwrap(),
            SaveNoChangeBasis::VerifiedOpening,
        )
        .unwrap();
        assert!(matches!(
            work.begin_write(),
            Err(SaveCoordinationError::Limit(SaveLimit::WriteCapacity))
        ));
        let sender = work.take_completion().unwrap();
        sender
            .publish(SaveStorageObservation::NoChange {
                witness,
                files: SaveFileFacts::from(old.as_ref()),
                basis: SaveNoChangeBasis::VerifiedOpening,
            })
            .unwrap();
        d.advance();
        assert!(matches!(
            d.active().outcome,
            Some(SaveProtocolOutcome::Confirmed(_))
        ));
        assert!(Arc::ptr_eq(&old, &d.session.archive_metadata));
        drop(work);
        drop(sender);
        assert_eq!(
            d.session.acknowledge_save(ticket).unwrap(),
            SaveAcknowledgement::Released
        );
    }
    let mut d = restored(true, RestoreSelection::Manual, 2);
    let _port = d.install(
        SaveCandidateLimits {
            logical_bytes: cost.logical_bytes,
            allocation_bytes: cost.allocation_bytes,
        },
        true,
    );
    d.admit();
    assert_eq!(d.active().permission, SaveWritePermission::MayWrite);
    assert_eq!(d.active().candidate, Some(cost));
}

#[test]
fn verify_only_requires_write_and_read_failure_have_distinct_outcomes() {
    for read_failure in [false, true] {
        let mut d = Driver::new();
        let golden = d.golden();
        let port = d.install(
            SaveCandidateLimits {
                logical_bytes: 0,
                allocation_bytes: 0,
            },
            false,
        );
        let ticket = d.admit();
        let mut work = d.session.take_save_execution(&port, ticket).unwrap();
        let witness = prepare(&mut work, &golden, &d.pool).unwrap();
        let sender = work.take_completion().unwrap();
        let result = if read_failure {
            SaveStorageObservation::Unknown(diagnostic(SaveDiagnosticCode::ReadFailed))
        } else {
            SaveStorageObservation::VerifiedNotCommitted {
                witness,
                diagnostic: diagnostic(SaveDiagnosticCode::Abandoned),
            }
        };
        sender.publish(result).unwrap();
        drop(work);
        drop(sender);
        d.advance();
        assert_eq!(d.status().facts.files, SaveFileFacts::default());
        if read_failure {
            assert_eq!(d.active().phase, SaveProtocolPhase::Reconciling);
            assert_eq!(
                d.active().diagnostic.unwrap().code,
                SaveDiagnosticCode::ReadFailed
            );
            assert_eq!(
                d.session.acknowledge_save(ticket),
                Err(SaveCoordinationError::NotTerminal)
            );
        } else {
            assert!(
                matches!(d.active().outcome,Some(SaveProtocolOutcome::NotCommitted(error)) if error.code==SaveDiagnosticCode::RequiresWriteCapacity)
            );
            assert_eq!(
                d.session.acknowledge_save(ticket).unwrap(),
                SaveAcknowledgement::Released
            );
        }
    }
}

#[test]
fn q_confirmation_keeps_new_branch_navigation_and_new_draft_pending() {
    let mut d = Driver::new();
    let a = d.set("A");
    d.set("B");
    d.nav(HistoryAction::Checkout { node: a });
    let golden = d.golden();
    let port = d.install(SaveCandidateLimits::default(), false);
    let ticket = d.admit();
    let mut work = d.session.take_save_execution(&port, ticket).unwrap();
    let witness = prepare(&mut work, &golden, &d.pool).unwrap();
    let receipt = fact(&work);
    let sender = work.take_completion().unwrap();
    work.begin_write().unwrap();
    d.set("new E branch\r\n");
    d.nav(HistoryAction::Checkout { node: a });
    let info = d.session.project_info();
    d.session
        .edit_draft(DraftRequest {
            api_version: API_VERSION,
            session_id: info.session_id,
            request_id: RequestId::new(d.next).unwrap(),
            action: DraftAction::Begin {
                expected: info.state,
            },
        })
        .unwrap();
    d.next += 1;
    let draft = d.session.draft_info(info.session_id).unwrap();
    let live = d.session.snapshot();
    sender.publish(committed(witness, receipt)).unwrap();
    d.advance();
    assert_eq!(d.session.snapshot(), live);
    assert_eq!(d.session.draft_info(info.session_id).unwrap(), draft);
    assert_eq!(
        d.status().facts.protection,
        SaveProtectionStatus::NeedsProtection
    );
    assert!(d.status().facts.pending_draft);
    assert_eq!(d.status().facts.covered.unwrap().nodes, 3);
    assert_eq!(d.status().facts.current.nodes, 4);
    drop(work);
    drop(sender);
    d.session.acknowledge_save(ticket).unwrap();
    let draft = draft.unwrap();
    d.session
        .edit_draft(DraftRequest {
            api_version: API_VERSION,
            session_id: info.session_id,
            request_id: RequestId::new(d.next).unwrap(),
            action: DraftAction::Cancel {
                expected: draft.state,
            },
        })
        .unwrap();
    assert!(!d.status().facts.pending_draft);
    assert_eq!(
        d.status().facts.protection,
        SaveProtectionStatus::NeedsProtection
    );
}

#[test]
fn protocol_is_unavailable_without_qualified_authority_and_no_default_success() {
    let mut d = Driver::new();
    assert!(!d.status().available);
    assert_eq!(
        d.status().facts.protection,
        SaveProtectionStatus::Unavailable
    );
    let _port = d
        .session
        .install_save_preparation(config(&d.session))
        .unwrap();
    assert!(!d.status().available);
    assert_eq!(d.status().facts.files, SaveFileFacts::default());
}

#[test]
fn first_manual_commit_installs_prepared_metadata_without_content_or_receipt_changes() {
    let mut d = Driver::new();
    d.set("// keep\r\n[Unknown]\r\nX : raw\r\n");
    let golden = d.golden();
    let before = d.session.snapshot();
    let stats = d.session.history_statistics(before.session_id).unwrap();
    let port = d.install(SaveCandidateLimits::default(), false);
    let ticket = d.admit();
    let weak = test_candidate_weak(&d.session).unwrap();
    let mut work = d.session.take_save_execution(&port, ticket).unwrap();
    let old = work.snapshot().clone();
    let witness = prepare(&mut work, &golden, &d.pool).unwrap();
    let receipt = fact(&work);
    let sender = work.take_completion().unwrap();
    {
        let permit = work.begin_write().unwrap();
        assert_eq!(permit.witness(), witness);
    }
    sender.publish(committed(witness, receipt)).unwrap();
    // Read-only status does not install queued observations.
    assert_eq!(d.status().facts.files.latest, None);
    d.advance();
    assert_eq!(d.session.snapshot(), before);
    assert_eq!(
        d.session.history_statistics(before.session_id).unwrap(),
        stats
    );
    assert_eq!(d.status().facts.protection, SaveProtectionStatus::Covered);
    assert_eq!(
        d.session.archive_metadata.manual_marks,
        vec![(ticket.commit, ticket.head)]
    );
    assert!(old.manifest().archive_metadata.manual_marks.is_empty());
    assert_eq!(
        d.session.acknowledge_save(ticket).unwrap(),
        SaveAcknowledgement::WaitingResources
    );
    drop(work);
    drop(sender);
    assert_eq!(
        d.session.acknowledge_save(ticket).unwrap(),
        SaveAcknowledgement::WaitingResources
    );
    drop(old);
    assert_eq!(
        d.session.acknowledge_save(ticket).unwrap(),
        SaveAcknowledgement::Released
    );
    // The candidate allocation was transferred to active metadata, not freed.
    assert!(weak.upgrade().is_some());
    assert!(d.status().facts.installed_metadata.is_some());
    let next = d.admit();
    assert_eq!(next.target.key, ticket.target.key);
    assert_eq!(next.target.phase, SaveTargetPhase::ExistingVerified);
    assert_eq!(next.target.expected.commit, Some(ticket.commit));
    d.session.cancel_save(next).unwrap();
    assert_eq!(
        d.session.acknowledge_save(next).unwrap(),
        SaveAcknowledgement::Released
    );
}

#[test]
fn writer_loss_after_permission_is_unknown_and_query_waits_for_actual_writer_exit() {
    let mut d = Driver::new();
    let golden = d.golden();
    let port = d.install(SaveCandidateLimits::default(), false);
    let ticket = d.admit();
    let reserved = test_query_allocation(&d.session).unwrap();
    let mut work = d.session.take_save_execution(&port, ticket).unwrap();
    let witness = prepare(&mut work, &golden, &d.pool).unwrap();
    let sender = work.take_completion().unwrap();
    work.begin_write().unwrap();
    sender
        .publish(SaveStorageObservation::Unknown(diagnostic(
            SaveDiagnosticCode::ReadFailed,
        )))
        .unwrap();
    d.advance();
    assert_eq!(
        d.session.begin_save_reconcile(ticket),
        Err(SaveCoordinationError::Busy(SaveBusy::Writer))
    );
    assert_eq!(
        d.session.acknowledge_save(ticket),
        Err(SaveCoordinationError::NotTerminal)
    );
    d.session.cancel_save(ticket).unwrap();
    drop(work);
    drop(sender);
    let query_ticket = d.session.begin_save_reconcile(ticket).unwrap().ticket();
    assert_eq!(test_query_allocation(&d.session), Some(reserved));
    let query = d.session.take_save_reconcile(&port, query_ticket).unwrap();
    assert!(!query.is_cancelled());
    assert_eq!(query.witness(), Some(witness));
    query
        .publish(SaveStorageObservation::VerifiedNotCommitted {
            witness,
            diagnostic: diagnostic(SaveDiagnosticCode::Cancelled),
        })
        .unwrap();
    d.advance();
    assert!(matches!(
        d.active().outcome,
        Some(SaveProtocolOutcome::NotCommitted(_))
    ));
    assert_eq!(
        d.session.acknowledge_save(ticket).unwrap(),
        SaveAcknowledgement::WaitingResources
    );
    drop(query);
    assert_eq!(
        d.session.acknowledge_save(ticket).unwrap(),
        SaveAcknowledgement::Released
    );
    assert_eq!(
        d.status().facts.protection,
        SaveProtectionStatus::NeedsProtection
    );
}

#[test]
fn late_main_commit_confirms_during_query_but_query_drop_still_gates_reuse() {
    let mut d = Driver::new();
    let golden = d.golden();
    let port = d.install(SaveCandidateLimits::default(), false);
    let ticket = d.admit();
    let mut work = d.session.take_save_execution(&port, ticket).unwrap();
    let witness = prepare(&mut work, &golden, &d.pool).unwrap();
    let receipt = fact(&work);
    let sender = work.take_completion().unwrap();
    work.begin_write().unwrap();
    drop(work);
    d.advance();
    assert_eq!(d.active().phase, SaveProtocolPhase::Reconciling);
    let qt = d.session.begin_save_reconcile(ticket).unwrap().ticket();
    let query = d.session.take_save_reconcile(&port, qt).unwrap();
    sender.publish(committed(witness, receipt)).unwrap();
    d.advance();
    assert!(query.is_cancelled());
    assert_eq!(d.status().facts.files.manual, Some(receipt));
    drop(sender);
    assert_eq!(
        d.session.acknowledge_save(ticket).unwrap(),
        SaveAcknowledgement::WaitingResources
    );
    assert_eq!(
        d.session.request_save(request(&d.session), &mut d.ids),
        Err(SaveCoordinationError::Busy(SaveBusy::SaveOutstanding))
    );
    query
        .publish(SaveStorageObservation::Unknown(diagnostic(
            SaveDiagnosticCode::QueryCancelled,
        )))
        .unwrap();
    drop(query);
    d.advance();
    assert!(d.status().active.is_none());
    assert_eq!(d.status().facts.files.manual, Some(receipt));
    assert_eq!(d.session.archive_metadata.manual_marks.len(), 1);
}

#[test]
fn query_cancellation_retry_and_attempt_exhaustion_preserve_unknown_and_old_context() {
    let mut d = Driver::new();
    let golden = d.golden();
    let port = d.install(SaveCandidateLimits::default(), false);
    let ticket = d.admit();
    let mut work = d.session.take_save_execution(&port, ticket).unwrap();
    prepare(&mut work, &golden, &d.pool).unwrap();
    work.begin_write().unwrap();
    drop(work);
    d.advance();
    let first = d.session.begin_save_reconcile(ticket).unwrap().ticket();
    let query = d.session.take_save_reconcile(&port, first).unwrap();
    d.session.cancel_save_reconcile(first).unwrap();
    assert!(query.is_cancelled());
    assert_eq!(
        d.session.begin_save_reconcile(ticket).unwrap(),
        SaveReconcileAdmission::Existing(first)
    );
    drop(query);
    d.advance();
    test_query_attempt(&mut d.session, u64::MAX);
    let last = d.session.begin_save_reconcile(ticket).unwrap().ticket();
    assert_eq!(last.attempt, u64::MAX);
    let query = d.session.take_save_reconcile(&port, last).unwrap();
    assert!(!query.is_cancelled());
    assert_eq!(
        d.session.cancel_save_reconcile(first),
        Err(SaveCoordinationError::Stale)
    );
    drop(query);
    d.advance();
    assert_eq!(
        d.session.begin_save_reconcile(ticket),
        Err(SaveCoordinationError::Limit(SaveLimit::QueryIdentity))
    );
    assert_eq!(d.active().phase, SaveProtocolPhase::Reconciling);
    assert!(d.active().witness_retained);
    assert!(!d.active().capture_consumers_released);
}

#[test]
fn cancel_before_permission_keeps_candidate_until_actual_owners_exit() {
    let mut d = Driver::new();
    let golden = d.golden();
    let port = d.install(SaveCandidateLimits::default(), false);
    let ticket = d.admit();
    let candidate = test_candidate_weak(&d.session).unwrap();
    let mut work = d.session.take_save_execution(&port, ticket).unwrap();
    prepare(&mut work, &golden, &d.pool).unwrap();
    let snapshot = work.snapshot().clone();
    let sender = work.take_completion().unwrap();
    d.session.cancel_save(ticket).unwrap();
    assert!(
        matches!(work.begin_write(),Err(SaveCoordinationError::Capture(e)) if e.code==SaveDiagnosticCode::Cancelled)
    );
    assert_eq!(
        d.session.acknowledge_save(ticket),
        Err(SaveCoordinationError::NotTerminal)
    );
    drop(work);
    d.advance();
    assert!(
        matches!(d.active().outcome,Some(SaveProtocolOutcome::NotCommitted(e)) if e.code==SaveDiagnosticCode::Cancelled)
    );
    assert_eq!(
        d.session.acknowledge_save(ticket).unwrap(),
        SaveAcknowledgement::WaitingResources
    );
    assert!(candidate.upgrade().is_some());
    drop(sender);
    d.advance();
    assert!(candidate.upgrade().is_some());
    drop(snapshot);
    d.advance();
    assert!(d.status().active.is_none());
    assert!(candidate.upgrade().is_none());
    assert_eq!(d.status().facts.files, SaveFileFacts::default());
}

#[test]
fn query_read_failures_missing_target_and_absence_without_proof_never_settle_unknown() {
    let mut d = Driver::new();
    let golden = d.golden();
    let port = d.install(SaveCandidateLimits::default(), false);
    let ticket = d.admit();
    let candidate = test_candidate_weak(&d.session).unwrap();
    let mut work = d.session.take_save_execution(&port, ticket).unwrap();
    prepare(&mut work, &golden, &d.pool).unwrap();
    work.begin_write().unwrap();
    drop(work);
    d.advance();
    let before = d.session.snapshot();
    for code in [
        Some(SaveDiagnosticCode::ReadFailed),
        Some(SaveDiagnosticCode::TargetMissing),
        Some(SaveDiagnosticCode::TargetConflict),
        Some(SaveDiagnosticCode::JournalConflict),
        Some(SaveDiagnosticCode::AdapterFault),
        None,
    ] {
        let query_ticket = d.session.begin_save_reconcile(ticket).unwrap().ticket();
        let query = d.session.take_save_reconcile(&port, query_ticket).unwrap();
        if let Some(code) = code {
            query
                .publish(SaveStorageObservation::Unknown(diagnostic(code)))
                .unwrap();
        }
        // None is represented by a lost observation, never by a fake absence proof.
        drop(query);
        d.advance();
        assert_eq!(d.active().phase, SaveProtocolPhase::Reconciling);
        assert!(d.active().witness_retained);
        assert!(candidate.upgrade().is_some());
        assert_eq!(
            d.session.acknowledge_save(ticket),
            Err(SaveCoordinationError::NotTerminal)
        );
        assert_eq!(d.session.snapshot(), before);
        assert_eq!(d.status().facts.files, SaveFileFacts::default());
    }
}

#[test]
fn query_found_original_commit_and_later_binding_conflict_are_both_retained() {
    let mut d = Driver::new();
    let golden = d.golden();
    let port = d.install(SaveCandidateLimits::default(), false);
    let ticket = d.admit();
    let mut work = d.session.take_save_execution(&port, ticket).unwrap();
    let witness = prepare(&mut work, &golden, &d.pool).unwrap();
    let receipt = fact(&work);
    work.begin_write().unwrap();
    drop(work);
    d.advance();
    let qt = d.session.begin_save_reconcile(ticket).unwrap().ticket();
    let query = d.session.take_save_reconcile(&port, qt).unwrap();
    query
        .publish(SaveStorageObservation::Replayed {
            witness,
            fact: receipt,
            binding_conflict: true,
        })
        .unwrap();
    d.advance();
    assert_eq!(d.status().facts.files.manual, Some(receipt));
    assert!(d.status().facts.binding_conflict);
    d.session.cancel_save(ticket).unwrap();
    assert_eq!(d.status().facts.files.manual, Some(receipt));
    assert_eq!(
        d.session.acknowledge_save(ticket).unwrap(),
        SaveAcknowledgement::WaitingResources
    );
    drop(query);
    d.advance();
    assert!(d.status().active.is_none());
    assert_eq!(
        d.session.request_save(request(&d.session), &mut d.ids),
        Err(SaveCoordinationError::InvalidBinding)
    );
    assert_eq!(d.session.archive_metadata.manual_marks.len(), 1);
}

#[test]
fn query_no_change_keeps_manual_a_and_protected_c_and_drops_unused_candidate() {
    let mut d = restored(true, RestoreSelection::Manual, 2);
    let golden = d.golden();
    verify_no_change_file(
        &golden,
        d.file.as_ref().unwrap(),
        SaveNoChangeBasis::VerifiedOpening,
    )
    .unwrap();
    let before = d.session.archive_metadata.clone();
    let port = d.install(SaveCandidateLimits::default(), true);
    let ticket = d.admit();
    let candidate = test_candidate_weak(&d.session).unwrap();
    let mut work = d.session.take_save_execution(&port, ticket).unwrap();
    let witness = prepare(&mut work, &golden, &d.pool).unwrap();
    work.begin_write().unwrap();
    drop(work);
    d.advance();
    let qt = d.session.begin_save_reconcile(ticket).unwrap().ticket();
    let query = d.session.take_save_reconcile(&port, qt).unwrap();
    query
        .publish(SaveStorageObservation::NoChange {
            witness,
            files: SaveFileFacts::from(before.as_ref()),
            basis: SaveNoChangeBasis::VerifiedOpening,
        })
        .unwrap();
    d.advance();
    assert!(Arc::ptr_eq(&before, &d.session.archive_metadata));
    assert_eq!(
        d.session.acknowledge_save(ticket).unwrap(),
        SaveAcknowledgement::WaitingResources
    );
    drop(query);
    d.advance();
    assert!(d.status().active.is_none());
    assert!(candidate.upgrade().is_none());
    assert_eq!(d.status().facts.protection, SaveProtectionStatus::Covered);
}

#[test]
fn replay_and_late_main_completion_append_the_manual_mark_only_once() {
    let mut d = Driver::new();
    d.set("A");
    let golden = d.golden();
    let port = d.install(SaveCandidateLimits::default(), false);
    let ticket = d.admit();
    let mut work = d.session.take_save_execution(&port, ticket).unwrap();
    let witness = prepare(&mut work, &golden, &d.pool).unwrap();
    let receipt = fact(&work);
    let sender = work.take_completion().unwrap();
    work.begin_write().unwrap();
    drop(work);
    d.advance();
    let qt = d.session.begin_save_reconcile(ticket).unwrap().ticket();
    let query = d.session.take_save_reconcile(&port, qt).unwrap();
    query
        .publish(SaveStorageObservation::Replayed {
            witness,
            fact: receipt,
            binding_conflict: false,
        })
        .unwrap();
    drop(query);
    d.advance();
    assert_eq!(
        d.session.acknowledge_save(ticket).unwrap(),
        SaveAcknowledgement::WaitingResources
    );
    let installed = d.session.archive_metadata.clone();
    sender.publish(committed(witness, receipt)).unwrap();
    assert_eq!(
        sender.publish(committed(witness, receipt)),
        Err(SaveCoordinationError::NotPending)
    );
    d.advance();
    assert!(d.active().acknowledged);
    assert!(Arc::ptr_eq(&installed, &d.session.archive_metadata));
    drop(sender);
    d.advance();
    assert!(d.status().active.is_none());
    assert_eq!(installed.manual_marks.len(), 1);
    let newer = d.admit();
    let before = d.status();
    assert_eq!(
        d.session.acknowledge_save(ticket),
        Err(SaveCoordinationError::Stale)
    );
    assert_eq!(d.status(), before);
    d.session.cancel_save(newer).unwrap();
    d.session.acknowledge_save(newer).unwrap();
}

#[test]
fn prepared_proof_is_immutable_and_all_commitment_fields_are_bound() {
    let mut d = Driver::new();
    d.set("raw\r\n");
    let golden = d.golden();
    let port = d.install(SaveCandidateLimits::default(), false);
    let ticket = d.admit();
    let mut work = d.session.take_save_execution(&port, ticket).unwrap();
    assert!(matches!(
        work.begin_write(),
        Err(SaveCoordinationError::NotPrepared)
    ));
    let proof = prepare(&mut work, &golden, &d.pool).unwrap();
    for index in 0..5 {
        let mut changed = proof.info();
        let field = match index {
            0 => &mut changed.content,
            1 => &mut changed.originals,
            2 => &mut changed.metadata,
            3 => &mut changed.request,
            _ => &mut changed.baseline,
        };
        field[0] ^= 1;
        assert_eq!(
            work.publish_witness(changed),
            Err(SaveCoordinationError::Conflict)
        );
    }
    assert_eq!(work.publish_witness(proof.info()).unwrap(), proof);
    drop(work);
    d.advance();
    assert!(matches!(
        d.active().outcome,
        Some(SaveProtocolOutcome::NotCommitted(_))
    ));
    d.session.acknowledge_save(ticket).unwrap();
}

#[test]
fn same_head_counts_and_ticket_cannot_substitute_another_frozen_content_proof() {
    let mut left = Driver::new();
    left.set("left raw\r\n");
    let lg = left.golden();
    let mut right = Driver::new();
    right.set("right raw\n");
    let rg = right.golden();
    // Deliberately reuse runtime IDs to test full proof/context checking; C,
    // not this fixture, will guarantee global host epoch freshness.
    let lp = left.install(SaveCandidateLimits::default(), false);
    let lt = left.admit();
    let rp = right.install(SaveCandidateLimits::default(), false);
    let rt = right.admit();
    assert_eq!(lt, rt);
    let mut lw = left.session.take_save_execution(&lp, lt).unwrap();
    let lproof = prepare(&mut lw, &lg, &left.pool).unwrap();
    let mut rw = right.session.take_save_execution(&rp, rt).unwrap();
    let rproof = prepare(&mut rw, &rg, &right.pool).unwrap();
    assert_ne!(lproof.info().content, rproof.info().content);
    let sender = lw.take_completion().unwrap();
    let receipt = fact(&lw);
    lw.begin_write().unwrap();
    sender.publish(committed(rproof, receipt)).unwrap();
    left.advance();
    assert_eq!(left.active().phase, SaveProtocolPhase::Reconciling);
    assert_eq!(
        left.active().diagnostic.unwrap().code,
        SaveDiagnosticCode::AdapterFault
    );
    assert_eq!(left.status().facts.files, SaveFileFacts::default());
    drop(lw);
    drop(rw);
    drop(sender);
}

#[test]
fn wrong_generation_or_request_context_cannot_install_a_receipt() {
    for mutation in 0..4 {
        let mut d = Driver::new();
        let golden = d.golden();
        let port = d.install(SaveCandidateLimits::default(), false);
        let ticket = d.admit();
        let mut work = d.session.take_save_execution(&port, ticket).unwrap();
        let witness = prepare(&mut work, &golden, &d.pool).unwrap();
        let mut bad = fact(&work);
        match mutation {
            0 => bad.generation = 0,
            1 => bad.generation += 1,
            2 => bad.commit = CommitId::new(99999).unwrap(),
            _ => bad.origin = SaveOrigin::Exit,
        }
        let sender = work.take_completion().unwrap();
        work.begin_write().unwrap();
        sender.publish(committed(witness, bad)).unwrap();
        drop(work);
        drop(sender);
        d.advance();
        assert_eq!(d.active().phase, SaveProtocolPhase::Reconciling);
        assert!(d.session.archive_metadata.manual_marks.is_empty());
        assert_eq!(
            d.session.acknowledge_save(ticket),
            Err(SaveCoordinationError::NotTerminal)
        );
    }
}

fn resource_golden() -> Driver {
    let bytes: [&[u8]; 3] = [
        b"unreferenced original",
        b"opaque audio\0\xff",
        b"D-only original",
    ];
    let pool = Pool(
        bytes
            .iter()
            .enumerate()
            .map(|(i, bytes)| {
                assert!(bytes.len() <= 64 * 1024);
                let blob = BlobRef::new([i as u8 + 1; 32], bytes.len() as u64);
                (
                    blob,
                    Arc::new(Original {
                        blob,
                        bytes: bytes.to_vec(),
                    }),
                )
            })
            .collect(),
    );
    let blobs: Vec<_> = pool.0.keys().copied().collect();
    let mut d = Driver::with_pool(pool);
    let transient = DocumentId::new(9).unwrap();
    let a = d.edit(vec![
        Command::CreateDocument {
            document: ElementDocument::new(
                transient,
                PixelSize::new(2, 2).unwrap(),
                LayerTree::default(),
            ),
        },
        Command::RemoveDocument {
            document: transient,
        },
        register(10, blobs[0], "unreferenced"),
        register(11, blobs[1], "opaque.wav"),
        Command::InsertConfigRecord {
            index: 0,
            record: record("// keep\r\n[Unknown]\r\nX : raw\r\n"),
        },
    ]);
    d.set("B\n");
    d.set("C\r\n");
    d.nav(HistoryAction::Checkout { node: a });
    d.edit(vec![
        register(12, blobs[2], "D only"),
        Command::SetConfigRecord {
            index: 0,
            record: record("D\r\n"),
        },
    ]);
    d.nav(HistoryAction::Undo);
    d
}
fn register(id: u128, blob: BlobRef, name: &str) -> Command {
    Command::RegisterAsset {
        asset: Asset::new(
            AssetId::new(id).unwrap(),
            name.into(),
            blob,
            AssetKind::Opaque,
        )
        .unwrap(),
    }
}

#[test]
fn full_resource_golden_confirms_only_q_and_capture_release_does_not_free_history_originals() {
    let mut d = resource_golden();
    let golden = d.golden();
    assert_eq!(golden.states.len(), 5);
    assert_eq!(golden.admissions.len(), 4);
    assert_eq!(golden.originals.len(), 3);
    assert!(golden.originals.values().map(Vec::len).sum::<usize>() < 256 * 1024);
    let weak: Vec<_> = d.pool.0.values().map(Arc::downgrade).collect();
    let port = d.install(SaveCandidateLimits::default(), false);
    let ticket = d.admit();
    let blob = *d.pool.0.keys().next().unwrap();
    d.edit(vec![
        register(13, blob, "new admission"),
        Command::SetConfigRecord {
            index: 0,
            record: record("E"),
        },
    ]);
    d.nav(HistoryAction::Checkout {
        node: NodeId::new(2).unwrap(),
    });
    let live = d.session.snapshot();
    let stats = d.session.history_statistics(live.session_id).unwrap();
    let mut work = d.session.take_save_execution(&port, ticket).unwrap();
    let snapshot = work.snapshot().clone();
    let proof = prepare(&mut work, &golden, &d.pool).unwrap();
    let receipt = fact(&work);
    let sender = work.take_completion().unwrap();
    work.begin_write().unwrap();
    sender.publish(committed(proof, receipt)).unwrap();
    drop(work);
    drop(sender);
    d.advance();
    assert_eq!(d.session.snapshot(), live);
    assert_eq!(
        d.session.history_statistics(live.session_id).unwrap(),
        stats
    );
    assert_eq!(d.status().facts.covered.unwrap().nodes, 5);
    assert_eq!(d.status().facts.current.nodes, 6);
    assert_eq!(
        d.status().facts.protection,
        SaveProtectionStatus::NeedsProtection
    );
    assert_eq!(
        d.session.acknowledge_save(ticket).unwrap(),
        SaveAcknowledgement::WaitingResources
    );
    drop(snapshot);
    d.advance();
    assert!(d.status().active.is_none());
    drop(d.pool);
    assert!(weak.iter().all(|w| w.upgrade().is_some()));
    drop(d.session);
    assert!(weak.iter().all(|w| w.upgrade().is_none()));
}

#[test]
fn golden_verifier_rejects_equal_count_mutations_before_publishing_proof() {
    for mutation in 0..7 {
        let mut d = resource_golden();
        let mut wrong = d.golden();
        match mutation {
            0 => {
                wrong.states.insert(2, wrong.states[&3].clone());
            }
            1 => {
                wrong.originals.values_mut().next().unwrap()[0] ^= 1;
            }
            2 => {
                Arc::make_mut(&mut wrong.metadata)
                    .extensions
                    .push(ArchiveExtension {
                        name: "unknown".into(),
                        bytes: vec![255],
                    });
            }
            3 => {
                wrong.source.navigation_revision += 1;
            }
            4 => {
                wrong.redo.clear();
            }
            5 => {
                wrong
                    .admissions
                    .remove(&ObjectId::Document(DocumentId::new(9).unwrap()));
                wrong
                    .admissions
                    .insert(ObjectId::Document(DocumentId::new(99).unwrap()));
            }
            _ => {
                wrong.parents.insert(5, Some(3));
            }
        }
        let port = d.install(SaveCandidateLimits::default(), false);
        let ticket = d.admit();
        let mut work = d.session.take_save_execution(&port, ticket).unwrap();
        assert!(
            prepare(&mut work, &wrong, &d.pool).is_err(),
            "mutation {mutation}"
        );
        assert!(!d.active().witness_retained);
        assert!(matches!(
            work.begin_write(),
            Err(SaveCoordinationError::NotPrepared)
        ));
        drop(work);
        d.advance();
        d.session.acknowledge_save(ticket).unwrap();
        assert!(d.session.archive_metadata.manual_marks.is_empty());
    }
}

#[test]
fn candidate_logical_and_managed_allocation_caps_are_independent_exact_boundaries() {
    let mut baseline = restored(true, RestoreSelection::Latest, 2);
    let _port = baseline.install(SaveCandidateLimits::default(), true);
    let ticket = baseline.admit();
    let cost = baseline.active().candidate.unwrap();
    assert!(cost.logical_bytes > 0 && cost.allocation_bytes > 0);
    baseline.session.cancel_save(ticket).unwrap();
    baseline.session.acknowledge_save(ticket).unwrap();
    for (limits, permission) in [
        (
            SaveCandidateLimits {
                logical_bytes: cost.logical_bytes,
                allocation_bytes: cost.allocation_bytes,
            },
            SaveWritePermission::MayWrite,
        ),
        (
            SaveCandidateLimits {
                logical_bytes: cost.logical_bytes - 1,
                allocation_bytes: cost.allocation_bytes,
            },
            SaveWritePermission::VerifyOnly,
        ),
        (
            SaveCandidateLimits {
                logical_bytes: cost.logical_bytes,
                allocation_bytes: cost.allocation_bytes - 1,
            },
            SaveWritePermission::VerifyOnly,
        ),
    ] {
        let mut d = restored(true, RestoreSelection::Latest, 2);
        let old = d.session.archive_metadata.clone();
        let golden = d.golden();
        let port = d.install(limits, true);
        let ticket = d.admit();
        let mut work = d.session.take_save_execution(&port, ticket).unwrap();
        prepare(&mut work, &golden, &d.pool).unwrap();
        assert_eq!(work.permission(), permission);
        if permission == SaveWritePermission::MayWrite {
            let candidate = work.candidate_metadata().unwrap();
            assert_eq!(candidate.tags, old.tags);
            assert_eq!(candidate.extensions, old.extensions);
            assert_eq!(
                &candidate.manual_marks[..old.manual_marks.len()],
                old.manual_marks
            );
            assert_eq!(d.active().candidate, Some(cost));
        } else {
            assert!(work.candidate_metadata().is_none());
            assert!(matches!(
                work.begin_write(),
                Err(SaveCoordinationError::Limit(SaveLimit::WriteCapacity))
            ));
        }
        d.session.cancel_save(ticket).unwrap();
        drop(work);
        d.advance();
        d.session.acknowledge_save(ticket).unwrap();
        assert!(Arc::ptr_eq(&old, &d.session.archive_metadata));
    }
    let mut d = Driver::new();
    let before = d.session.snapshot();
    let authority = synthetic_authority(&d.session, false);
    assert!(matches!(
        d.session.install_save_protocol(
            config(&d.session),
            SaveCandidateLimits {
                logical_bytes: SAVE_CANDIDATE_LOGICAL_BYTES + 1,
                ..SaveCandidateLimits::default()
            },
            authority
        ),
        Err(SaveCoordinationError::Limit(SaveLimit::WriteCapacity))
    ));
    assert_eq!(d.session.snapshot(), before);
    assert!(!d.status().available);
}

#[test]
fn versions_missing_witness_and_new_empty_root_cannot_manufacture_no_change() {
    let mut d = Driver::new();
    let golden = d.golden();
    let port = d.install(SaveCandidateLimits::default(), false);
    let ticket = d.admit();
    let mut work = d.session.take_save_execution(&port, ticket).unwrap();
    assert!(matches!(
        work.begin_write(),
        Err(SaveCoordinationError::NotPrepared)
    ));
    let proof = prepare(&mut work, &golden, &d.pool).unwrap();
    for mutation in 0..4 {
        let mut info = proof.info();
        match mutation {
            0 => info.version += 1,
            1 => info.capture_version -= 1,
            2 => info.proof_profile -= 1,
            _ => info.ticket.source.navigation_revision += 1,
        }
        let expected = if mutation == 3 {
            SaveCoordinationError::Conflict
        } else {
            SaveCoordinationError::UnsupportedVersion
        };
        assert_eq!(work.publish_witness(info), Err(expected));
    }
    let sender = work.take_completion().unwrap();
    sender
        .publish(SaveStorageObservation::NoChange {
            witness: proof,
            files: SaveFileFacts::default(),
            basis: SaveNoChangeBasis::CurrentManual,
        })
        .unwrap();
    drop(work);
    drop(sender);
    d.advance();
    assert_eq!(d.active().phase, SaveProtocolPhase::Reconciling);
    assert_eq!(d.status().facts.files, SaveFileFacts::default());
    assert_eq!(
        d.session.acknowledge_save(ticket),
        Err(SaveCoordinationError::NotTerminal)
    );
}

#[test]
fn plain_queries_and_exhausted_ordinary_receipts_do_not_block_confirmation_or_cleanup() {
    let mut d = Driver::new();
    let golden = d.golden();
    let port = d.install(SaveCandidateLimits::default(), false);
    let ticket = d.admit();
    let mut work = d.session.take_save_execution(&port, ticket).unwrap();
    let proof = prepare(&mut work, &golden, &d.pool).unwrap();
    let receipt = fact(&work);
    let sender = work.take_completion().unwrap();
    work.begin_write().unwrap();
    for _ in 0..128 {
        d.edit(vec![]);
    }
    let before = d
        .session
        .history_statistics(d.session.project_info().session_id)
        .unwrap();
    let info = d.session.project_info();
    assert!(
        d.session
            .submit(CommandRequest {
                api_version: API_VERSION,
                session_id: info.session_id,
                request_id: RequestId::new(129).unwrap(),
                expected: info.state,
                commands: vec![]
            })
            .is_err()
    );
    let stable = d.status();
    let stats = work.statistics();
    for _ in 0..128 {
        assert_eq!(d.status(), stable);
        assert_eq!(work.statistics(), stats);
    }
    sender.publish(committed(proof, receipt)).unwrap();
    assert_eq!(d.status(), stable); // status neither drains nor captures
    drop(work);
    drop(sender);
    d.advance();
    assert_eq!(d.status().facts.protection, SaveProtectionStatus::Covered);
    assert_eq!(
        d.session.history_statistics(info.session_id).unwrap(),
        before
    );
    d.session.cancel_save(ticket).unwrap();
    d.session.acknowledge_save(ticket).unwrap();
    assert!(d.status().active.is_none());
    assert_eq!(d.status().facts.files.manual, Some(receipt));
}

#[test]
fn undispatched_cancel_drop_before_ack_and_session_drop_keep_real_ownership_rules() {
    let mut d = Driver::new();
    let port = d.install(SaveCandidateLimits::default(), false);
    let ticket = d.admit();
    let candidate = test_candidate_weak(&d.session).unwrap();
    d.session.cancel_save(ticket).unwrap();
    assert!(candidate.upgrade().is_some());
    assert!(d.active().writer_released);
    d.session.acknowledge_save(ticket).unwrap();
    assert!(candidate.upgrade().is_none());
    let ticket = d.admit();
    let golden = d.golden();
    let mut work = d.session.take_save_execution(&port, ticket).unwrap();
    prepare(&mut work, &golden, &d.pool).unwrap();
    work.begin_write().unwrap();
    drop(work);
    d.advance();
    let qt = d.session.begin_save_reconcile(ticket).unwrap().ticket();
    let query = d.session.take_save_reconcile(&port, qt).unwrap();
    let candidate = test_candidate_weak(&d.session).unwrap();
    let snapshot = query.snapshot().clone();
    drop(d.session);
    assert!(query.is_cancelled());
    assert!(candidate.upgrade().is_some());
    assert!(snapshot.is_complete());
    drop(query);
    assert!(candidate.upgrade().is_none());
    drop(snapshot);
}

#[test]
fn foreign_port_and_old_runtime_ticket_cannot_take_current_execution() {
    let mut first = Driver::new();
    let p1 = first.install(SaveCandidateLimits::default(), false);
    let t1 = first.admit();
    let mut second = restored(true, RestoreSelection::Manual, 2);
    let p2 = second.install(SaveCandidateLimits::default(), true);
    let t2 = second.admit();
    let before = second.status();
    assert!(second.session.take_save_execution(&p1, t1).is_err());
    assert!(matches!(
        second.session.take_save_execution(&p1, t2),
        Err(SaveCoordinationError::WrongPort)
    ));
    assert_eq!(second.status(), before);
    let work = second.session.take_save_execution(&p2, t2).unwrap();
    drop(second.session);
    assert!(work.is_cancelled());
    drop(work);
    first.session.cancel_save(t1).unwrap();
    first.session.acknowledge_save(t1).unwrap();
}

#[test]
fn candidate_ceiling_does_not_borrow_the_separate_capture_manifest_budget() {
    for extra in [0, 1] {
        let mut d = restored(true, RestoreSelection::Latest, 2);
        let metadata = Arc::make_mut(&mut d.session.archive_metadata);
        let existing = metadata.logical_bytes().unwrap();
        let grow = SAVE_CANDIDATE_LOGICAL_BYTES - 64 - existing + extra;
        metadata.extensions[0]
            .bytes
            .extend(std::iter::repeat_n(0x9d, grow));
        let before = d.session.snapshot();
        let old = d.session.archive_metadata.clone();
        let _port = d.install(SaveCandidateLimits::default(), false);
        let request = request(&d.session);
        // The capture's 1 MiB includes the complete manifest as well as this
        // metadata. The two ceilings are not simultaneously spendable pools.
        assert!(
            matches!(d.session.request_save(request,&mut d.ids),Err(SaveCoordinationError::Capture(e)) if e.code==SaveDiagnosticCode::CaptureLimit)
        );
        assert!(d.status().active.is_none());
        assert_eq!(d.session.snapshot(), before);
        assert!(Arc::ptr_eq(&old, &d.session.archive_metadata));
        assert_eq!(
            d.session.save_coordination_status(1).unwrap().next_key,
            Some(request.key)
        );
    }
}

#[test]
fn contradictory_late_query_cannot_erase_confirmed_fact_or_unlock_binding() {
    for case in 0..4 {
        let mut d = if case == 3 {
            restored(true, RestoreSelection::Manual, 2)
        } else {
            Driver::new()
        };
        let golden = d.golden();
        let files = SaveFileFacts::from(d.session.archive_metadata.as_ref());
        let port = d.install(SaveCandidateLimits::default(), case == 3);
        let ticket = d.admit();
        let mut work = d.session.take_save_execution(&port, ticket).unwrap();
        let proof = prepare(&mut work, &golden, &d.pool).unwrap();
        let receipt = fact(&work);
        let sender = work.take_completion().unwrap();
        work.begin_write().unwrap();
        drop(work);
        d.advance();
        let qt = d.session.begin_save_reconcile(ticket).unwrap().ticket();
        let query = d.session.take_save_reconcile(&port, qt).unwrap();
        let observation = if case != 0 {
            SaveStorageObservation::VerifiedNotCommitted {
                witness: proof,
                diagnostic: diagnostic(SaveDiagnosticCode::Cancelled),
            }
        } else {
            SaveStorageObservation::Unknown(diagnostic(SaveDiagnosticCode::TargetConflict))
        };
        let main = if case == 3 {
            verify_no_change_file(
                &golden,
                d.file.as_ref().unwrap(),
                SaveNoChangeBasis::VerifiedOpening,
            )
            .unwrap();
            SaveStorageObservation::NoChange {
                witness: proof,
                files,
                basis: SaveNoChangeBasis::VerifiedOpening,
            }
        } else {
            committed(proof, receipt)
        };
        if case >= 2 {
            query.publish(observation).unwrap();
            d.advance();
            assert!(matches!(
                d.active().outcome,
                Some(SaveProtocolOutcome::NotCommitted(_))
            ));
            sender.publish(main).unwrap();
            d.advance();
        } else {
            sender.publish(main).unwrap();
            d.advance();
            let installed = d.session.archive_metadata.clone();
            query.publish(observation).unwrap();
            d.advance();
            assert!(Arc::ptr_eq(&installed, &d.session.archive_metadata));
        }
        d.advance();
        assert_eq!(
            d.status().facts.files.manual,
            if case == 3 {
                files.manual
            } else {
                Some(receipt)
            }
        );
        assert!(d.status().facts.binding_conflict);
        d.session.acknowledge_save(ticket).unwrap();
        drop(query);
        drop(sender);
        d.advance();
        assert!(d.status().active.is_none());
        assert_eq!(
            d.session.request_save(request(&d.session), &mut d.ids),
            Err(SaveCoordinationError::InvalidBinding)
        );
    }
}
