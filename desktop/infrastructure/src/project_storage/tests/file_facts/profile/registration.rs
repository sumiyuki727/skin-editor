use super::*;

pub(super) fn opening(mut file: FileFactsRequest) -> FileProfileRestoreRequest {
    file.key = FileFactsKey {
        session: HistorySessionId::new(88).unwrap(),
        sequence: 9,
    };
    FileProfileRestoreRequest {
        version: FILE_PROFILE_VERSION,
        file,
    }
}

fn bind(
    r: &mut FileTargetRegistry,
    reader: &mut FileFactsReader,
    stamp: FileTargetStamp,
    path: &Path,
    req: FileProfileRestoreRequest,
    store: &BlobStore,
) -> Result<RestoreCandidate, FileFactsError> {
    r.bind(
        reader,
        stamp,
        path,
        req,
        store,
        CaptureCancellation::default(),
    )
}

fn work(
    candidate: RestoreCandidate,
    req: FileProfileRestoreRequest,
    store: &BlobStore,
) -> (
    Driver,
    SavePreparationPort,
    SavePreparationWork,
    FileFactsRequest,
) {
    save::attach_work(activate(candidate, req.file, store), req.file)
}

fn observe(
    r: &mut FileTargetRegistry,
    stamp: FileTargetStamp,
    work: &mut SavePreparationWork,
    ticket: SaveTicket,
) -> Result<FileFactsSummary, FileFactsError> {
    let cursor = work.capture_cursor(SAVE_CAPTURE_CURSOR_VERSION).unwrap();
    r.opening_for_capture(stamp, &cursor, ticket, &CaptureCancellation::default())
        .map(|v| v.summary())
}

#[test]
fn one_original_handle_survives_restore_reader_drop_and_cancellation_until_registry_release() {
    let (dir, req) = copy_golden_file();
    let req = opening(req);
    let source = dir.file();
    let bytes = dir.closed_main_bytes();
    let alias = dir.path.join("target-link.db");
    let moved = dir.path.join("target-moved.db");
    std::fs::hard_link(&source, &alias).unwrap();
    let scratch = Directory::new();
    let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    let store = BlobStore::default();
    assert_eq!(
        FileTargetRegistry::new(&reader, 2).err(),
        Some(FileFactsError::UnsupportedVersion)
    );
    let mut r = FileTargetRegistry::new(&reader, FILE_TARGET_REGISTRY_VERSION).unwrap();
    let empty = r.stamp();
    let candidate = bind(&mut r, &mut reader, empty, &source, req, &store).unwrap();
    let origin = candidate.file_profile(1).unwrap().unwrap().origin();
    let occupied = r.stamp();
    assert_ne!(occupied, empty);
    for name in [&source, &alias] {
        assert_eq!(
            bind(&mut r, &mut reader, occupied, name, req, &store).err(),
            Some(FileFactsError::Busy)
        );
        assert_eq!(r.stamp(), occupied);
    }
    drop(reader);
    let (mut d, _port, mut w, _) = work(candidate, req, &store);
    let ticket = w.ticket();
    assert_ne!(ticket.key.sequence, req.file.key.sequence);
    let initial = w.statistics();
    let summary = observe(&mut r, occupied, &mut w, ticket).unwrap();
    assert_eq!(
        (summary.observation, summary.request),
        (origin.observation, origin.file)
    );
    assert_eq!(
        std::fs::rename(&source, &moved).unwrap_err().raw_os_error(),
        Some(32)
    );
    assert_eq!(
        std::fs::OpenOptions::new()
            .write(true)
            .open(&alias)
            .unwrap_err()
            .raw_os_error(),
        Some(32)
    );
    assert_eq!(w.statistics(), initial);
    {
        let mut cursor = w.capture_cursor(SAVE_CAPTURE_CURSOR_VERSION).unwrap();
        let held = r
            .opening_for_capture(occupied, &cursor, ticket, &CaptureCancellation::default())
            .unwrap();
        cursor.poll(1).unwrap();
        assert_eq!(held.summary(), summary);
    }
    let mut substituted = ticket;
    substituted.coverage.0 += 1;
    assert_eq!(
        observe(&mut r, occupied, &mut w, substituted),
        Err(FileFactsError::StalePage)
    );
    {
        let cancelled = CaptureCancellation::default();
        cancelled.cancel();
        let cursor = w.capture_cursor(SAVE_CAPTURE_CURSOR_VERSION).unwrap();
        assert_eq!(
            r.opening_for_capture(occupied, &cursor, ticket, &cancelled)
                .err(),
            Some(FileFactsError::Cancelled)
        );
    }
    d.session.cancel_save(ticket).unwrap();
    assert_eq!(
        observe(&mut r, occupied, &mut w, ticket),
        Err(FileFactsError::Cancelled)
    );
    assert!(
        d.session
            .save_protocol_status(1)
            .unwrap()
            .facts
            .installed_metadata
            .is_none()
    );
    drop(w);
    drop(d);
    assert!(std::fs::File::open(&source).is_err());
    r.release(occupied).unwrap();
    assert_eq!(r.release(occupied), Err(FileFactsError::StalePage));
    std::fs::rename(&source, &moved).unwrap();
    assert_eq!(std::fs::read(&moved).unwrap(), bytes);
    assert_eq!(store.usage().unwrap().bytes, 0);
}

#[test]
fn replacement_checks_registrar_generation_and_authentic_opening_even_after_aba() {
    let (a, req) = copy_golden_file();
    let b = Directory::from_main_bytes(&a.closed_main_bytes());
    let req = opening(req);
    let scratch = Directory::new();
    let second_root = Directory::new();
    let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    let mut foreign = FileFactsReader::open(&second_root.path, FileFactsLimits::default()).unwrap();
    let store = BlobStore::default();
    let mut r = FileTargetRegistry::new(&reader, 1).unwrap();
    let mut other = FileTargetRegistry::new(&reader, 1).unwrap();
    let empty = r.stamp();
    assert_eq!(
        bind(&mut r, &mut foreign, empty, &a.file(), req, &store).err(),
        Some(FileFactsError::StalePage)
    );
    assert_eq!(
        bind(&mut r, &mut reader, other.stamp(), &a.file(), req, &store).err(),
        Some(FileFactsError::StalePage)
    );
    let candidate = bind(&mut r, &mut reader, empty, &a.file(), req, &store).unwrap();
    let first = r.stamp();
    let (_d, _port, mut old, _) = work(candidate, req, &store);
    let ticket = old.ticket();
    let other_empty = other.stamp();
    let second = bind(&mut other, &mut reader, other_empty, &b.file(), req, &store).unwrap();
    let other_full = other.stamp();
    assert_ne!(first, other_full);
    assert_eq!(other.release(first), Err(FileFactsError::StalePage));
    assert_eq!(
        observe(&mut other, other_full, &mut old, ticket),
        Err(FileFactsError::TargetConflict)
    );
    other.release(other_full).unwrap();
    drop(second);
    let b_candidate = bind(&mut r, &mut reader, first, &b.file(), req, &store).unwrap();
    let b_stamp = r.stamp();
    assert_eq!(r.release(first), Err(FileFactsError::StalePage));
    assert_eq!(
        observe(&mut r, b_stamp, &mut old, ticket),
        Err(FileFactsError::TargetConflict)
    );
    let (_d2, _p2, mut new_work, _) = work(b_candidate, req, &store);
    let new_ticket = new_work.ticket();
    assert!(observe(&mut r, b_stamp, &mut new_work, new_ticket).is_ok());
    assert!(std::fs::File::open(a.file()).is_ok());
    r.release(b_stamp).unwrap();
    let vacant = r.stamp();
    assert_ne!(vacant, empty);
    assert_eq!(
        bind(&mut r, &mut reader, empty, &a.file(), req, &store).err(),
        Some(FileFactsError::StalePage)
    );
    let again = bind(&mut r, &mut reader, vacant, &a.file(), req, &store).unwrap();
    let latest = r.stamp();
    assert_eq!(
        observe(&mut r, first, &mut old, ticket),
        Err(FileFactsError::StalePage)
    );
    assert_eq!(
        observe(&mut r, latest, &mut old, ticket),
        Err(FileFactsError::TargetConflict)
    );
    drop(r);
    assert!(std::fs::File::open(a.file()).is_ok());
    assert!(again.file_profile(1).unwrap().is_some());
}

#[test]
fn replacement_peak_is_admitted_before_releasing_old_target_and_cleans_up_on_failures() {
    let (a, req) = copy_golden_file();
    let b = Directory::from_main_bytes(&a.closed_main_bytes());
    let req = opening(req);
    let mut required = FileFactsLimits::default().control_bytes;
    for pass in 0..3 {
        let scratch = Directory::new();
        let limits = FileFactsLimits {
            control_bytes: required - usize::from(pass == 2),
            ..FileFactsLimits::default()
        };
        let mut reader = FileFactsReader::open(&scratch.path, limits).unwrap();
        let idle = reader.usage().managed_bytes;
        let store = BlobStore::default();
        let mut r = FileTargetRegistry::new(&reader, 1).unwrap();
        let empty = r.stamp();
        let old = bind(&mut r, &mut reader, empty, &a.file(), req, &store).unwrap();
        let first = r.stamp();
        let held = reader.usage().managed_bytes;
        if pass == 0 {
            support::probe(
                &mut reader,
                Probe {
                    cancel_at: Some(Stage::ReadProfileSource),
                    ..Probe::default()
                },
            );
            assert_eq!(
                bind(&mut r, &mut reader, first, &b.file(), req, &store).err(),
                Some(FileFactsError::Cancelled)
            );
            support::probe(&mut reader, Probe::default());
            support::cleanup(&mut reader, true);
            assert!(matches!(
                bind(&mut r, &mut reader, first, &b.file(), req, &store),
                Err(FileFactsError::Cleanup { .. })
            ));
            support::cleanup(&mut reader, false);
            reader.retry_cleanup().unwrap();
            assert_eq!(reader.usage().managed_bytes, held);
            assert_eq!(r.stamp(), first);
            assert!(std::fs::File::open(a.file()).is_err());
        }
        let next = bind(&mut r, &mut reader, first, &b.file(), req, &store);
        if pass == 2 {
            assert_eq!(
                next.err(),
                Some(FileFactsError::Limit(FileFactsLimit::Control))
            );
            assert_eq!(r.stamp(), first);
            assert_eq!(reader.usage().managed_bytes, held);
            assert!(std::fs::File::open(a.file()).is_err());
        } else {
            drop(next.unwrap());
            required = reader.usage().managed_peak_bytes;
            assert!(std::fs::File::open(a.file()).is_ok());
        }
        drop(r);
        drop(old);
        assert_eq!(reader.usage().managed_bytes, idle);
        assert_eq!(store.usage().unwrap().bytes, 0);
    }
}

#[test]
fn exhausted_registration_generation_preserves_release() {
    let (dir, req) = copy_golden_file();
    let scratch = Directory::new();
    let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    let store = BlobStore::default();
    let mut r = FileTargetRegistry::new(&reader, 1).unwrap();
    r.set_generation_for_test(u64::MAX - 1);
    let before = r.stamp();
    let req = opening(req);
    let candidate = bind(&mut r, &mut reader, before, &dir.file(), req, &store).unwrap();
    let last = r.stamp();
    assert_eq!(
        bind(&mut r, &mut reader, last, &dir.file(), req, &store).err(),
        Some(FileFactsError::Limit(FileFactsLimit::Sequence))
    );
    r.release(last).unwrap();
    let released = r.stamp();
    assert_ne!(last, released);
    assert_eq!(
        bind(&mut r, &mut reader, released, &dir.file(), req, &store).err(),
        Some(FileFactsError::Limit(FileFactsLimit::Sequence))
    );
    assert!(std::fs::File::open(dir.file()).is_ok());
    drop(candidate);
}

#[test]
fn copied_public_origin_on_another_restore_cannot_borrow_the_registered_handle() {
    let (a, req) = copy_golden_file();
    let b = Directory::from_main_bytes(&a.closed_main_bytes());
    let req = opening(req);
    let scratch = Directory::new();
    let mut reader = FileFactsReader::open(&scratch.path, FileFactsLimits::default()).unwrap();
    let store = BlobStore::default();
    let mut r = FileTargetRegistry::new(&reader, 1).unwrap();
    let empty = r.stamp();
    let original = bind(&mut r, &mut reader, empty, &a.file(), req, &store).unwrap();
    let origin = original.file_profile(1).unwrap().unwrap().origin();
    let stamp = r.stamp();
    for spoof in [false, true] {
        let (file, mut clone) = open(&b.file(), store.clone(), req.file.selection);
        drop(file);
        if spoof {
            clone
                .attach_file_profile(1, Arc::new(Fake { origin, bytes: 1 }))
                .unwrap();
        }
        let (_d, _p, mut w, _) = work(clone, req, &store);
        let ticket = w.ticket();
        assert_eq!(
            observe(&mut r, stamp, &mut w, ticket),
            Err(FileFactsError::TargetConflict)
        );
    }
    assert_eq!(r.stamp(), stamp);
}
