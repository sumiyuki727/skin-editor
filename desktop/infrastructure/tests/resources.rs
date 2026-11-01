use skin_editor_application::{values::*, *};
use skin_editor_audio::*;
use skin_editor_infrastructure::*;
use std::io::{Cursor, Read, Seek, SeekFrom};

// Self-authored probe/decoder fake. This format is not WAV or stable audio.
struct Decoder {
    calls: usize,
    fail: bool,
}
impl AudioDecoder for Decoder {
    fn probe(&self, bytes: &[u8]) -> Result<AudioMetadata, PrepareFailure> {
        if bytes.len() != 8 || bytes[0] != 0xa1 {
            return Err(PrepareFailure::Unsupported);
        }
        AudioMetadata::new(48000, 1, u64::from(bytes[1])).map_err(|_| PrepareFailure::InvalidPcm)
    }
    fn decode_into(
        &mut self,
        _: &[u8],
        _: AudioMetadata,
        out: &mut [f32],
        cancel: &Cancellation,
    ) -> Result<usize, PrepareFailure> {
        self.calls += 1;
        if self.fail {
            return Err(PrepareFailure::InvalidPcm);
        }
        if cancel.is_cancelled() {
            return Err(PrepareFailure::Cancelled);
        }
        out.fill(0.25);
        Ok(out.len())
    }
}
fn decoder() -> Decoder {
    Decoder {
        calls: 0,
        fail: false,
    }
}
fn raw(store: &BlobStore, n: u8) -> RawBlob {
    store
        .stage_audio(vec![0xa1, 4, n, 0, 0, 0, 0, 0], &decoder())
        .unwrap()
}
fn origin(revision: u64) -> Origin {
    Origin {
        session: 1,
        node: revision + 1,
        revision,
        navigation_revision: 0,
        draft: None,
    }
}
fn jobs(raw: &RawBlob, n: usize, revision: u64) -> Vec<BuildRequest> {
    let plans: Vec<_> = (1..=n)
        .map(|n| SoundPlan {
            target: (AudioComponentId::new(n as u128).unwrap()).into(),
            source: raw.reference(),
            metadata: raw.audio_metadata(),
            gain: Gain::UNITY,
            visible: true,
        })
        .collect();
    Runtime::new(48000, 1)
        .unwrap()
        .synchronize(origin(revision), &plans)
        .unwrap()
}
#[test]
fn original_png_and_opaque_bytes_share_one_store_and_pins_do_not_own_rgba() {
    let store = BlobStore::with_limits(64 * 1024, 32 * 1024).unwrap();
    let mut images = Images::with_store(store.clone());
    let bytes = Images::sample_png();
    let png = images
        .stage_bytes(bytes.clone(), "self authored".into())
        .unwrap();
    let same = store.stage_bytes(bytes.clone()).unwrap();
    assert_eq!(same.reference(), png.blob);
    assert_eq!(same.bytes(), bytes);
    assert_eq!(store.usage().unwrap().bytes, bytes.len());
    let pin = images.pin(png.blob).unwrap();
    let blob = png.blob;
    drop(png);
    drop(same);
    assert!(images.rgba_bytes() > 0);
    drop(images);
    assert_eq!(store.get(blob).unwrap().bytes(), bytes);
    drop(pin);
    assert_eq!(store.usage().unwrap().bytes, 0);
    let opaque = store.stage_bytes(vec![255, 1, 7]).unwrap();
    assert_eq!(opaque.bytes(), [255, 1, 7]);
    assert_eq!(store.audio_metadata(opaque.reference()), None);
}
#[test]
fn full_raw_pool_reuses_known_bytes_and_reader_without_reallocating() {
    let store = BlobStore::with_limits(8, 8).unwrap();
    let a = store.stage_bytes(vec![1; 8]).unwrap();
    let b = store.stage_bytes(vec![1; 8]).unwrap();
    let c = store.stage_reader(Cursor::new(vec![1; 8]), 8).unwrap();
    assert!(a.same_allocation(&b));
    assert!(b.same_allocation(&c));
    assert_eq!(store.usage().unwrap().bytes, 8);
    assert!(matches!(
        store.stage_bytes(vec![2]),
        Err(BlobError::ResourceLimit)
    ));
    drop(a);
    drop(b);
    drop(c);
    assert_eq!(store.usage().unwrap().entries, 0);
    assert_eq!(store.usage().unwrap().bytes, 0);
}
struct ChangedReader {
    cursor: Cursor<Vec<u8>>,
    seeks: usize,
}
impl Read for ChangedReader {
    fn read(&mut self, out: &mut [u8]) -> std::io::Result<usize> {
        self.cursor.read(out)
    }
}
impl Seek for ChangedReader {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.seeks += 1;
        if self.seeks == 2 {
            self.cursor.get_mut()[0] ^= 1;
        }
        self.cursor.seek(pos)
    }
}
#[test]
fn changed_or_truncated_readers_return_reservations_without_registering_partial_bytes() {
    let store = BlobStore::with_limits(8, 8).unwrap();
    let reader = ChangedReader {
        cursor: Cursor::new(vec![1; 4]),
        seeks: 0,
    };
    assert!(matches!(
        store.stage_reader(reader, 4),
        Err(BlobError::SourceChanged)
    ));
    assert_eq!(store.usage().unwrap().bytes, 0);
    assert_eq!(store.usage().unwrap().entries, 0);
    assert!(matches!(
        store.stage_reader(Cursor::new(vec![1]), 2),
        Err(BlobError::Io)
    ));
    assert_eq!(store.usage().unwrap().bytes, 0);
    assert!(matches!(
        store.stage_reader(Cursor::new(vec![1; 9]), 9),
        Err(BlobError::ResourceLimit)
    ));
    assert_eq!(store.usage().unwrap().bytes, 0);
}
#[test]
fn adopted_capacity_counts_and_unknown_audio_is_preserved_without_false_metadata() {
    let store = BlobStore::with_limits(16, 16).unwrap();
    let mut over = Vec::with_capacity(32);
    over.push(1);
    assert!(matches!(
        store.stage_bytes(over),
        Err(BlobError::ResourceLimit)
    ));
    assert_eq!(store.usage().unwrap().bytes, 0);
    let unknown = vec![7, 8, 9];
    assert!(matches!(
        store.stage_audio(unknown.clone(), &decoder()),
        Err(BlobError::Unsupported)
    ));
    let raw = store.stage_bytes(unknown.clone()).unwrap();
    assert_eq!(raw.bytes(), unknown);
    assert_eq!(raw.audio_metadata(), None);
}
fn request(s: &ProjectSession, id: u128, commands: Vec<Command>) -> CommandRequest {
    CommandRequest {
        api_version: API_VERSION,
        session_id: s.project_info().session_id,
        request_id: RequestId::new(id).unwrap(),
        expected: s.project_info().state,
        commands,
    }
}
#[test]
fn history_pins_original_bytes_even_after_active_asset_removal_and_session_drop_releases() {
    let store = BlobStore::with_limits(8, 8).unwrap();
    let raw = store.stage_bytes(vec![1; 8]).unwrap();
    let reference = raw.reference();
    let mut session =
        ProjectSession::with_resources(1, 1, Box::new(store.clone()), SessionLimits::default())
            .unwrap();
    let asset = AssetId::new(1).unwrap();
    let r = request(
        &session,
        1,
        vec![Command::RegisterAsset {
            asset: Asset::new(asset, "opaque".into(), reference, AssetKind::Opaque).unwrap(),
        }],
    );
    session.submit(r).unwrap();
    drop(raw);
    let r = request(&session, 2, vec![Command::RemoveAsset { asset }]);
    session.submit(r).unwrap();
    assert_eq!(store.usage().unwrap().bytes, 8);
    assert!(store.stage_bytes(vec![2]).is_err());
    drop(session);
    assert_eq!(store.usage().unwrap().bytes, 0);
}
#[test]
fn one_worker_and_four_pending_tasks_refuse_excess_before_reserving_or_decoding() {
    let store = BlobStore::default();
    let raw = self::raw(&store, 1);
    let budget = PcmBudget::new(96, 16).unwrap();
    let mut queue = AudioPreparationQueue::new(budget.clone());
    let jobs = jobs(&raw, 6, 0);
    queue.submit(jobs[0], raw.clone()).unwrap();
    let running = queue.start_next().unwrap();
    for job in &jobs[1..5] {
        queue.submit(*job, raw.clone()).unwrap();
    }
    assert_eq!(queue.usage().active, 1);
    assert_eq!(queue.usage().pending, 4);
    assert!(queue.start_next().is_none());
    assert_eq!(budget.usage().bytes, 80);
    assert!(matches!(
        queue.submit(jobs[5], raw.clone()),
        Err(PreparationError::Busy)
    ));
    assert_eq!(budget.usage().bytes, 80);
    queue.close();
    assert_eq!(budget.usage().bytes, 16);
    drop(running);
    assert_eq!(budget.usage().bytes, 0);
    assert_eq!(queue.usage().active, 0);
}
#[test]
fn pcm_over_budget_rejects_before_decoder_and_replacement_refunds_pending_candidate() {
    let store = BlobStore::default();
    let raw = self::raw(&store, 1);
    let budget = PcmBudget::new(16, 16).unwrap();
    let mut queue = AudioPreparationQueue::new(budget.clone());
    let first = jobs(&raw, 2, 0);
    queue.submit(first[0], raw.clone()).unwrap();
    assert!(matches!(
        queue.submit(first[1], raw.clone()),
        Err(PreparationError::ResourceLimit)
    ));
    assert_eq!(queue.usage().pending, 1);
    let replacement = jobs(&raw, 1, 1)[0];
    queue.submit(replacement, raw.clone()).unwrap();
    assert_eq!(queue.usage().pending, 1);
    assert_eq!(budget.usage().bytes, 16);
    let job = queue.start_next().unwrap();
    assert_eq!(job.request(), replacement);
    drop(job);
    assert_eq!(budget.usage().bytes, 0);
    assert_eq!(queue.usage().active, 0);
}
#[test]
fn running_cancel_keeps_credit_until_ack_and_never_starts_a_second_worker() {
    let store = BlobStore::default();
    let raw = self::raw(&store, 1);
    let budget = PcmBudget::new(32, 16).unwrap();
    let mut queue = AudioPreparationQueue::new(budget.clone());
    let jobs = jobs(&raw, 2, 0);
    for job in &jobs {
        queue.submit(*job, raw.clone()).unwrap();
    }
    let running = queue.start_next().unwrap();
    queue.cancel_component(jobs[0].plan.target);
    assert_eq!(budget.usage().bytes, 32);
    assert!(queue.start_next().is_none());
    let mut decoder = decoder();
    let finished = running.execute(&mut decoder);
    assert_eq!(decoder.calls, 0);
    assert!(matches!(
        queue.finish(finished).unwrap().1,
        Err(PrepareFailure::Cancelled)
    ));
    assert_eq!(budget.usage().bytes, 16);
    drop(queue.start_next().unwrap());
    assert_eq!(budget.usage().bytes, 0);
}
#[test]
fn decode_failure_and_discarded_result_release_pcm_and_raw_pins() {
    let store = BlobStore::with_limits(16, 16).unwrap();
    let raw = self::raw(&store, 1);
    let reference = raw.reference();
    let budget = PcmBudget::new(16, 16).unwrap();
    let mut queue = AudioPreparationQueue::new(budget.clone());
    let job = jobs(&raw, 1, 0)[0];
    queue.submit(job, raw).unwrap();
    let mut decoder = Decoder {
        calls: 0,
        fail: true,
    };
    let finished = queue.start_next().unwrap().execute(&mut decoder);
    assert_eq!(budget.usage().bytes, 0);
    assert_eq!(store.usage().unwrap().bytes, 0);
    assert!(matches!(
        queue.finish(finished).unwrap().1,
        Err(PrepareFailure::InvalidPcm)
    ));
    assert!(store.get(reference).is_none());
    let raw = self::raw(&store, 2);
    let job = jobs(&raw, 1, 1)[0];
    queue.submit(job, raw).unwrap();
    let finished = queue.start_next().unwrap().execute(&mut self::decoder());
    assert_eq!(budget.usage().bytes, 16);
    drop(finished);
    assert_eq!(budget.usage().bytes, 0);
    assert_eq!(queue.usage().active, 0);
}
#[test]
fn ready_cache_reuses_pcm_across_components_at_full_pool_and_never_pins_it_forever() {
    let store = BlobStore::default();
    let raw = self::raw(&store, 1);
    let budget = PcmBudget::new(16, 16).unwrap();
    let mut queue = AudioPreparationQueue::new(budget.clone());
    let jobs = jobs(&raw, 2, 0);
    let mut decoder = decoder();
    queue.submit(jobs[0], raw.clone()).unwrap();
    let finished = queue.start_next().unwrap().execute(&mut decoder);
    let sound = queue.finish(finished).unwrap().1.unwrap();
    let reused = match queue.submit(jobs[1], raw.clone()).unwrap() {
        Admission::Cached(sound) => sound,
        _ => panic!("expected cache reuse"),
    };
    assert!(sound.pcm().same_allocation(reused.pcm()));
    assert_eq!(decoder.calls, 1);
    assert_eq!(budget.usage().bytes, 16);
    drop(sound);
    drop(reused);
    assert_eq!(budget.usage().bytes, 0);
    assert_eq!(queue.usage().cached, 0);
}
#[test]
fn missing_metadata_cannot_enter_typed_audio_preparation_or_modify_project_on_failure() {
    let store = BlobStore::default();
    let raw = self::raw(&store, 1);
    let mut session =
        ProjectSession::with_resources(1, 1, Box::new(store.clone()), SessionLimits::default())
            .unwrap();
    let before = session.snapshot();
    let fake = Asset::new(
        AssetId::new(1).unwrap(),
        "wrong metadata".into(),
        raw.reference(),
        AssetKind::Audio(AudioMetadata::new(44100, 1, 4).unwrap()),
    )
    .unwrap();
    let r = request(&session, 1, vec![Command::RegisterAsset { asset: fake }]);
    assert!(matches!(
        session.submit(r),
        Err(CommandError::Resource {
            error: ResourceError::MetadataMismatch,
            ..
        })
    ));
    assert_eq!(session.snapshot(), before);
    let opaque = store.stage_bytes(vec![1; 8]).unwrap();
    let mut job = jobs(&raw, 1, 0)[0];
    job.plan.source = opaque.reference();
    let mut queue = AudioPreparationQueue::new(PcmBudget::default());
    assert!(matches!(
        queue.submit(job, opaque),
        Err(PreparationError::MetadataMismatch)
    ));
    assert_eq!(queue.usage().pcm.bytes, 0);
}

#[test]
fn pending_cancellation_releases_raw_and_pcm_credit_immediately() {
    let store = BlobStore::with_limits(8, 8).unwrap();
    let raw = self::raw(&store, 1);
    let job = jobs(&raw, 1, 0)[0];
    let budget = PcmBudget::new(16, 16).unwrap();
    let mut queue = AudioPreparationQueue::new(budget.clone());
    queue.submit(job, raw).unwrap();
    assert_eq!(store.usage().unwrap().bytes, 8);
    assert_eq!(budget.usage().bytes, 16);
    queue.cancel_outdated(origin(1));
    assert_eq!(store.usage().unwrap().bytes, 0);
    assert_eq!(budget.usage().bytes, 0);
    assert_eq!(queue.usage().pending, 0);
}
struct CancelDuring<'a> {
    queue: &'a mut AudioPreparationQueue,
    component: AudioComponentId,
}
impl AudioDecoder for CancelDuring<'_> {
    fn probe(&self, bytes: &[u8]) -> Result<AudioMetadata, PrepareFailure> {
        decoder().probe(bytes)
    }
    fn decode_into(
        &mut self,
        _: &[u8],
        _: AudioMetadata,
        out: &mut [f32],
        cancel: &Cancellation,
    ) -> Result<usize, PrepareFailure> {
        out.fill(0.5);
        self.queue.cancel_component(self.component);
        assert!(cancel.is_cancelled());
        Ok(out.len())
    }
}
#[test]
fn cancellation_during_decode_discards_output_even_if_decoder_returns_success() {
    let store = BlobStore::default();
    let raw = self::raw(&store, 1);
    let job = jobs(&raw, 1, 0)[0];
    let budget = PcmBudget::new(16, 16).unwrap();
    let mut queue = AudioPreparationQueue::new(budget.clone());
    queue.submit(job, raw).unwrap();
    let running = queue.start_next().unwrap();
    let finished = running.execute(&mut CancelDuring {
        queue: &mut queue,
        component: match job.plan.target {
            skin_editor_audio::SoundTarget::Component(id) => id,
            _ => unreachable!(),
        },
    });
    assert_eq!(budget.usage().bytes, 0);
    assert!(matches!(
        queue.finish(finished).unwrap().1,
        Err(PrepareFailure::Cancelled)
    ));
    assert_eq!(queue.usage().cached, 0);
}
struct PartialDecoder;
impl AudioDecoder for PartialDecoder {
    fn probe(&self, bytes: &[u8]) -> Result<AudioMetadata, PrepareFailure> {
        decoder().probe(bytes)
    }
    fn decode_into(
        &mut self,
        _: &[u8],
        _: AudioMetadata,
        out: &mut [f32],
        _: &Cancellation,
    ) -> Result<usize, PrepareFailure> {
        let written = out.len() / 2;
        out[..written].fill(0.25);
        Ok(written)
    }
}
#[test]
fn partial_decode_success_is_rejected_and_refunds_all_resources() {
    let store = BlobStore::default();
    let raw = self::raw(&store, 1);
    let job = jobs(&raw, 1, 0)[0];
    let budget = PcmBudget::new(16, 16).unwrap();
    let mut queue = AudioPreparationQueue::new(budget.clone());
    queue.submit(job, raw).unwrap();
    let finished = queue.start_next().unwrap().execute(&mut PartialDecoder);
    assert_eq!(store.usage().unwrap().bytes, 0);
    assert_eq!(budget.usage().bytes, 0);
    assert!(matches!(
        queue.finish(finished).unwrap().1,
        Err(PrepareFailure::InvalidPcm)
    ));
    assert_eq!(queue.usage().cached, 0);
}
#[test]
fn foreign_completion_cannot_steal_an_active_slot_with_the_same_local_sequence() {
    let store = BlobStore::default();
    let raw = self::raw(&store, 1);
    let job = jobs(&raw, 1, 0)[0];
    let budget = PcmBudget::new(32, 16).unwrap();
    let mut a = AudioPreparationQueue::new(budget.clone());
    let mut b = AudioPreparationQueue::new(budget.clone());
    a.submit(job, raw.clone()).unwrap();
    b.submit(job, raw).unwrap();
    let running_a = a.start_next().unwrap();
    let foreign = b.start_next().unwrap().execute(&mut decoder());
    assert!(matches!(a.finish(foreign), Err(PreparationError::Stale)));
    assert_eq!(a.usage().active, 1);
    assert_eq!(b.usage().active, 0);
    assert_eq!(budget.usage().bytes, 16);
    let own = running_a.execute(&mut decoder());
    let sound = a.finish(own).unwrap().1.unwrap();
    assert_eq!(sound.samples(), [0.25; 4]);
    drop(sound);
    assert_eq!(budget.usage().bytes, 0);
}
fn project(
    components: usize,
    budget: PcmBudget,
) -> (BlobStore, ProjectSession, SkinAudioController) {
    let store = BlobStore::default();
    let raw = self::raw(&store, 1);
    let asset = AssetId::new(1).unwrap();
    let mut commands = vec![Command::RegisterAsset {
        asset: Asset::new(
            asset,
            "fake audio fixture".into(),
            raw.reference(),
            AssetKind::Audio(raw.audio_metadata().unwrap()),
        )
        .unwrap(),
    }];
    commands.extend((1..=components).map(|n| Command::CreateAudioComponent {
        component: AudioComponent::new(
            AudioComponentId::new(n as u128).unwrap(),
            SoundSlotKey::new(format!("fixture.{n}")).unwrap(),
            asset,
        ),
    }));
    let mut session =
        ProjectSession::with_resources(1, 1, Box::new(store.clone()), SessionLimits::default())
            .unwrap();
    let r = request(&session, 1, commands);
    session.submit(r).unwrap();
    (
        store,
        session,
        SkinAudioController::with_budget(48000, 1, budget).unwrap(),
    )
}
#[test]
fn real_controller_reports_each_admission_and_reuses_cached_pcm_on_explicit_retry() {
    let budget = PcmBudget::new(16, 16).unwrap();
    let (store, session, mut controller) = project(2, budget.clone());
    let before = session.snapshot();
    let admissions = controller.admit_builds(&session, &store).unwrap();
    assert_eq!(admissions.len(), 2);
    assert_eq!(admissions[0].status, Ok(AdmissionState::Queued));
    assert_eq!(admissions[1].status, Err(PreparationError::ResourceLimit));
    let mut decoder = decoder();
    let finished = controller
        .start_preparation()
        .unwrap()
        .execute(&mut decoder);
    assert!(matches!(
        controller.finish_preparation(&session, finished).unwrap(),
        Completion::Activated { .. }
    ));
    controller
        .retry_preparation(&session, AudioComponentId::new(2).unwrap())
        .unwrap();
    let retry = controller.admit_builds(&session, &store).unwrap();
    assert!(matches!(retry[0].status, Ok(AdmissionState::Cached(_))));
    assert_eq!(decoder.calls, 1);
    assert_eq!(budget.usage().bytes, 16);
    assert_eq!(session.snapshot(), before);
    controller.close();
    assert_eq!(budget.usage().bytes, 0);
    drop(session);
    assert_eq!(store.usage().unwrap().bytes, 0);
}
#[test]
fn real_draft_cancellation_rejects_finished_job_without_ui_tick_and_returns_pcm() {
    let budget = PcmBudget::new(16, 16).unwrap();
    let (store, mut session, mut controller) = project(1, budget.clone());
    let before = session.snapshot();
    let scope = session.project_info().session_id;
    session
        .edit_draft(DraftRequest {
            api_version: API_VERSION,
            session_id: scope,
            request_id: RequestId::new(2).unwrap(),
            action: DraftAction::Begin {
                expected: before.state,
            },
        })
        .unwrap();
    let ds = session.draft_info(scope).unwrap().unwrap().state;
    session
        .edit_draft(DraftRequest {
            api_version: API_VERSION,
            session_id: scope,
            request_id: RequestId::new(3).unwrap(),
            action: DraftAction::Replace {
                expected: ds,
                commands: vec![Command::SetAudioProperties {
                    component: AudioComponentId::new(1).unwrap(),
                    gain: Gain::new(0.5).unwrap(),
                    visible: true,
                }],
            },
        })
        .unwrap();
    controller.admit_builds(&session, &store).unwrap();
    let finished = controller
        .start_preparation()
        .unwrap()
        .execute(&mut decoder());
    assert_eq!(budget.usage().bytes, 16);
    let ds = session.draft_info(scope).unwrap().unwrap().state;
    session
        .edit_draft(DraftRequest {
            api_version: API_VERSION,
            session_id: scope,
            request_id: RequestId::new(4).unwrap(),
            action: DraftAction::Cancel { expected: ds },
        })
        .unwrap();
    assert_eq!(
        controller.finish_preparation(&session, finished).unwrap(),
        Completion::Stale
    );
    assert_eq!(budget.usage().bytes, 0);
    assert_eq!(session.snapshot(), before);
}

#[test]
fn original_bytes_registered_only_in_a_cancelled_draft_are_reclaimable() {
    let store = BlobStore::with_limits(8, 8).unwrap();
    let raw = store.stage_bytes(vec![1; 8]).unwrap();
    let reference = raw.reference();
    let mut session =
        ProjectSession::with_resources(1, 1, Box::new(store.clone()), SessionLimits::default())
            .unwrap();
    let before = session.snapshot();
    session
        .edit_draft(DraftRequest {
            api_version: API_VERSION,
            session_id: before.session_id,
            request_id: RequestId::new(1).unwrap(),
            action: DraftAction::Begin {
                expected: before.state,
            },
        })
        .unwrap();
    let ds = session
        .draft_info(before.session_id)
        .unwrap()
        .unwrap()
        .state;
    session
        .edit_draft(DraftRequest {
            api_version: API_VERSION,
            session_id: before.session_id,
            request_id: RequestId::new(2).unwrap(),
            action: DraftAction::Replace {
                expected: ds,
                commands: vec![Command::RegisterAsset {
                    asset: Asset::new(
                        AssetId::new(1).unwrap(),
                        "opaque draft input".into(),
                        reference,
                        AssetKind::Opaque,
                    )
                    .unwrap(),
                }],
            },
        })
        .unwrap();
    drop(raw);
    assert_eq!(store.usage().unwrap().bytes, 8);
    let ds = session
        .draft_info(before.session_id)
        .unwrap()
        .unwrap()
        .state;
    session
        .edit_draft(DraftRequest {
            api_version: API_VERSION,
            session_id: before.session_id,
            request_id: RequestId::new(3).unwrap(),
            action: DraftAction::Cancel { expected: ds },
        })
        .unwrap();
    assert_eq!(store.usage().unwrap().bytes, 0);
    assert_eq!(session.snapshot(), before);
    assert_eq!(session.history_len(), 1);
}
#[test]
fn png_file_import_keeps_source_unchanged_on_success_and_decode_failure() {
    use std::io::Write;
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "skin-editor-png-{}-{unique}.png",
        std::process::id()
    ));
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .unwrap();
    struct Cleanup(std::path::PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
        }
    }
    let _cleanup = Cleanup(path.clone());
    let bytes = Images::sample_png();
    file.write_all(&bytes).unwrap();
    drop(file);
    let mut images = Images::default();
    let stage = images.stage_png(path.to_str().unwrap()).unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    std::fs::write(&path, [1, 2, 3]).unwrap();
    assert!(matches!(
        images.stage_png(path.to_str().unwrap()),
        Err(ImportError::InvalidImage)
    ));
    assert_eq!(std::fs::read(&path).unwrap(), [1, 2, 3]);
    assert_eq!(images.blobs().get(stage.blob).unwrap().bytes(), bytes);
}
