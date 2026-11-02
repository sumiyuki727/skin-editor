use skin_editor_application::{values::*, *};
use skin_editor_infrastructure::{
    BlobStore, RandomHistoryIdentities,
    project_format::{FormatLimits, decode_project, encode_project},
};

struct Ids(u128);
impl HistoryIdentitySource for Ids {
    fn next_identity(&mut self) -> Result<u128, IdentityError> {
        self.0 += 1;
        Ok(self.0)
    }
}
fn command(s: &mut ProjectSession, n: u128, commands: Vec<Command>) {
    let i = s.project_info();
    s.submit(CommandRequest {
        api_version: API_VERSION,
        session_id: i.session_id,
        request_id: RequestId::new(n).unwrap(),
        expected: i.state,
        commands,
    })
    .unwrap();
}
fn navigate(s: &mut ProjectSession, n: u128, action: HistoryAction) {
    let i = s.project_info();
    s.navigate(HistoryRequest {
        api_version: API_VERSION,
        session_id: i.session_id,
        request_id: RequestId::new(n).unwrap(),
        expected: i.state,
        action,
    })
    .unwrap();
}
fn request(s: &ProjectSession) -> CaptureRequest {
    let i = s.project_info();
    CaptureRequest {
        version: HISTORY_CAPTURE_VERSION,
        session_id: i.session_id,
        expected: i.state,
        limits: CaptureLimits::default(),
        cancellation: CaptureCancellation::default(),
    }
}
fn register(id: u128, blob: BlobRef) -> Command {
    Command::RegisterAsset {
        asset: Asset::new(
            AssetId::new(id).unwrap(),
            format!("opaque-{id}"),
            blob,
            AssetKind::Opaque,
        )
        .unwrap(),
    }
}

#[test]
fn all_branches_encode_real_states_and_keep_exact_original_bytes_until_final_release() {
    fn source<T: HistoryIdentitySource>() {}
    source::<RandomHistoryIdentities>();
    let store = BlobStore::with_limits(1024 * 1024, 1024 * 1024).unwrap();
    let bytes_a = b"unknown audio codec bytes: retain exactly";
    let bytes_b = b"unused original in the other branch";
    let a = store
        .stage_reader(std::io::Cursor::new(bytes_a), bytes_a.len() as u64)
        .unwrap();
    let b = store
        .stage_reader(std::io::Cursor::new(bytes_b), bytes_b.len() as u64)
        .unwrap();
    let reference_a = a.reference();
    let mut s =
        ProjectSession::with_resources(7, 70, Box::new(store.clone()), SessionLimits::default())
            .unwrap();
    command(
        &mut s,
        1,
        vec![register(1, a.reference()), register(2, a.reference())],
    );
    let first = s.project_info().state.head;
    command(
        &mut s,
        2,
        vec![
            Command::RemoveAsset {
                asset: AssetId::new(1).unwrap(),
            },
            Command::RemoveAsset {
                asset: AssetId::new(2).unwrap(),
            },
        ],
    );
    let removed = s.project_info().state.head;
    navigate(&mut s, 3, HistoryAction::Checkout { node: first });
    command(&mut s, 4, vec![register(3, b.reference())]);
    let fork = s.project_info().state.head;
    navigate(&mut s, 5, HistoryAction::Checkout { node: removed });
    let scope = s.project_info().session_id;
    let expected: std::collections::BTreeMap<_, _> =
        [NodeId::new(1).unwrap(), first, removed, fork]
            .into_iter()
            .map(|n| (n.get(), s.history_snapshot(scope, n).unwrap().project))
            .collect();
    let live = s.snapshot();
    let before = store.usage().unwrap();
    let mut limited = request(&s);
    limited.limits.originals = 1;
    assert!(matches!(
        s.capture_history(limited, &mut Ids(1000)),
        Err(CaptureError::ResourceLimit("originals"))
    ));
    let mut capture = s.capture_history(request(&s), &mut Ids(1000)).unwrap();
    assert_eq!(store.usage().unwrap().bytes, before.bytes); // independent pins, no media copies
    assert_eq!(store.usage().unwrap().peak_bytes, before.peak_bytes);
    let snapshot = capture.snapshot().unwrap();
    assert_eq!(snapshot.manifest().originals.len(), 2);
    assert_eq!(
        capture.statistics().original_bytes,
        (bytes_a.len() + bytes_b.len()) as u64
    );
    drop(a);
    drop(b);
    drop(s);
    let mut states = Vec::new();
    loop {
        match capture.poll(2).unwrap() {
            CapturePoll::Node(n) => {
                assert_eq!(n.project(), &expected[&n.info().alias]);
                let bundle = encode_project(n.project(), FormatLimits::default()).unwrap();
                assert_eq!(
                    decode_project(&bundle, FormatLimits::default()).unwrap(),
                    *n.project()
                );
                states.push(n);
            }
            CapturePoll::Pending => (),
            CapturePoll::Complete => break,
        }
    }
    assert_eq!(states.len(), 4);
    assert_eq!(store.get(reference_a).unwrap().bytes(), bytes_a);
    assert_eq!(store.usage().unwrap().bytes, before.bytes);
    drop(snapshot);
    drop(states);
    drop(capture);
    assert_eq!(store.usage().unwrap().bytes, 0);
    assert_eq!(store.usage().unwrap().entries, 0);
    assert_eq!(live.state.head, removed); // metadata value does not own original bytes
}

#[test]
fn cancellation_and_rejection_do_not_allocate_or_release_session_originals() {
    let store = BlobStore::with_limits(1024, 1024).unwrap();
    let a = store
        .stage_reader(std::io::Cursor::new([8u8; 128]), 128)
        .unwrap();
    let mut s =
        ProjectSession::with_resources(7, 70, Box::new(store.clone()), SessionLimits::default())
            .unwrap();
    command(&mut s, 1, vec![register(1, a.reference())]);
    drop(a);
    let mut ids = Ids(1000);
    let original = s.snapshot();
    let mut r = request(&s);
    r.limits.original_bytes = 127;
    assert!(matches!(
        s.capture_history(r, &mut ids),
        Err(CaptureError::ResourceLimit("original_bytes"))
    ));
    for _ in 0..20 {
        let mut capture = s.capture_history(request(&s), &mut ids).unwrap();
        drop(capture.poll(1).unwrap());
        capture.cancel();
        assert_eq!(store.usage().unwrap().bytes, 128);
    }
    assert_eq!(s.snapshot(), original);
    drop(s);
    assert_eq!(store.usage().unwrap().bytes, 0);
}
