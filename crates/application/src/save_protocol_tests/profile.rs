//! Synthetic A-confirmation fixture only, not a production file authority.
use super::*;
struct Observed(FileProfileOrigin);
impl FileProfileSource for Observed {
    fn origin(&self) -> FileProfileOrigin {
        self.0
    }
    fn logical_bytes(&self) -> usize {
        1024
    }
}
#[test]
fn confirmed_expected_advances_without_rewriting_an_immutable_profile_or_locking_the_next_capture()
{
    let mut d = restored(true, RestoreSelection::Manual, 2);
    let binding = config(&d.session).binding;
    let persistent = d.session.persistence.as_ref().unwrap();
    let origin = FileProfileOrigin {
        version: 1,
        file: FileFactsRequest {
            version: 1,
            key: FileFactsKey {
                session: d.session.session_id,
                sequence: 9,
            },
            binding,
            selection: RestoreSelection::Manual,
        },
        observation: FileObservationId {
            instance: 1,
            sequence: 1,
        },
        root: persistent.nodes[&NodeId::new(1).unwrap()],
        opened_head: persistent.nodes[&d.session.state.head],
        features_root: [1; 32],
        source_sha256: [2; 32],
    };
    d.session.file_profile =
        Some(crate::file_profile::ProfileAttachment::new(Arc::new(Observed(origin))).unwrap());
    let golden = d.golden();
    let port = d.install(SaveCandidateLimits::default(), false);
    let ticket = d.admit();
    let mut work = d.session.take_save_execution(&port, ticket).unwrap();
    let old_snapshot = work.snapshot().clone();
    let witness = prepare(&mut work, &golden, &d.pool).unwrap();
    let fact = fact(&work);
    let sender = work.take_completion().unwrap();
    work.begin_write().unwrap();
    sender.publish(committed(witness, fact)).unwrap();
    d.advance();
    assert_ne!(
        d.session
            .archive_metadata
            .last_commit
            .as_ref()
            .unwrap()
            .commit,
        origin.file.binding.expected.commit.unwrap()
    );
    assert_eq!(
        old_snapshot.file_profile(1).unwrap().unwrap().origin(),
        origin
    );
    drop(work);
    drop(sender);
    drop(old_snapshot);
    assert_eq!(
        d.session.acknowledge_save(ticket),
        Ok(SaveAcknowledgement::Released)
    );
    let next = d.admit();
    assert_eq!(next.target.expected.commit, Some(fact.commit));
    assert_ne!(next.target.expected, origin.file.binding.expected);
    let next_work = d.session.take_save_execution(&port, next).unwrap();
    assert_eq!(
        next_work
            .snapshot()
            .file_profile(1)
            .unwrap()
            .unwrap()
            .origin(),
        origin
    );
    drop(next_work);
}

#[test]
fn detached_attachment_and_capture_each_require_source_plus_handle_at_the_exact_boundary() {
    let mut d = restored(true, RestoreSelection::Manual, 2);
    let info = d.session.project_info();
    let mut capture = d
        .session
        .capture_history(
            CaptureRequest {
                version: 3,
                session_id: info.session_id,
                expected: info.state,
                limits: CaptureLimits::default(),
                cancellation: CaptureCancellation::default(),
            },
            &mut d.ids,
        )
        .unwrap();
    let base_capture = capture.statistics().retained_metadata_bytes;
    let snapshot = capture.snapshot().unwrap();
    let m = snapshot.manifest();
    let header = RestoreHeader {
        node_count: m.nodes.len(),
        project: m.project_id,
        identity: m.identity.clone(),
        root: m.root,
        head: m.head,
        next_alias: m.next_alias,
        next_branch: m.next_branch,
        admissions: m.admissions.clone(),
        redo_preferences: m.redo_preferences.clone(),
        originals: m.originals.clone(),
        metadata: m.archive_metadata.clone(),
    };
    let mut nodes = Vec::new();
    loop {
        match capture.poll(1).unwrap() {
            CapturePoll::Pending => (),
            CapturePoll::Complete => break,
            CapturePoll::Node(n) => {
                let i = n.info();
                nodes.push(RestoreNode {
                    id: i.id,
                    parent: i.parent,
                    alias: i.alias,
                    coordinate: i.coordinate,
                    depth: i.depth,
                    operation: i.operation.clone(),
                    origin: i.origin.clone().unwrap(),
                    project: n.project().clone(),
                });
            }
        }
    }
    drop(snapshot);
    drop(capture);
    let build = |bytes| -> Result<RestoreCandidate, RestoreError> {
        let mut builder = RestoreBuilder::new(
            header.clone(),
            RestoreLimits {
                metadata_bytes: bytes,
                ..RestoreLimits::default()
            },
            CaptureCancellation::default(),
        )?;
        for node in &nodes {
            builder.push(node.clone())?;
        }
        builder.finish(&d.pool)
    };
    // Find the existing restore's public admission boundary; add the separately
    // specified source+handle layout, not a budget inferred from attachment.
    let (mut low, mut high) = (1, 1024 * 1024);
    while low < high {
        let mid = low + (high - low) / 2;
        if build(mid).is_ok() {
            high = mid;
        } else {
            low = mid + 1;
        }
    }
    let source_cost = 1024 + std::mem::size_of::<crate::file_profile::ProfileAttachment>();
    let origin = FileProfileOrigin {
        version: 1,
        file: FileFactsRequest {
            version: 1,
            key: FileFactsKey {
                session: HistorySessionId::new(77).unwrap(),
                sequence: 9,
            },
            binding: config(&d.session).binding,
            selection: RestoreSelection::Manual,
        },
        observation: FileObservationId {
            instance: 1,
            sequence: 1,
        },
        root: header.root,
        opened_head: header.head,
        features_root: [1; 32],
        source_sha256: [2; 32],
    };
    let mut short = build(low + source_cost - 1).unwrap();
    assert_eq!(
        short.attach_file_profile(1, Arc::new(Observed(origin))),
        Err(RestoreError::Limit("profile metadata"))
    );
    assert!(short.file_profile(1).unwrap().is_none());
    drop(short);
    let mut exact = build(low + source_cost).unwrap();
    exact
        .attach_file_profile(1, Arc::new(Observed(origin)))
        .unwrap();
    let mut session = exact
        .into_session(
            origin.file.key.session,
            Box::new(d.pool.clone()),
            SessionLimits::default(),
            CaptureCancellation::default(),
        )
        .unwrap();
    let capture_request = |bytes| CaptureRequest {
        version: 3,
        session_id: session.project_info().session_id,
        expected: session.project_info().state,
        limits: CaptureLimits {
            retained_metadata_bytes: bytes,
            ..CaptureLimits::default()
        },
        cancellation: CaptureCancellation::default(),
    };
    let too_small = capture_request(base_capture + source_cost - 1);
    let exact_request = capture_request(base_capture + source_cost);
    assert!(matches!(
        session.capture_history(too_small, &mut d.ids),
        Err(CaptureError::ResourceLimit("retained_metadata_bytes"))
    ));
    let capture = session.capture_history(exact_request, &mut d.ids).unwrap();
    assert_eq!(
        capture.statistics().retained_metadata_bytes,
        base_capture + source_cost
    );
    assert_eq!(
        capture
            .snapshot()
            .unwrap()
            .file_profile(1)
            .unwrap()
            .unwrap()
            .origin(),
        origin
    );
}
