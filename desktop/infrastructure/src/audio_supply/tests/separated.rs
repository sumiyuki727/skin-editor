use super::*;
use crate::separated_output;
use skin_editor_audio::PreparedSound;
mod streaming;
fn idle(core: &mut SupplyCore, separated: bool) -> bool {
    let voices = core.voice_counts();
    core.can_idle(separated, voices)
}

#[test]
fn idle_wait_requires_frozen_primed_settled_controls_and_no_voice_or_preparation() {
    let (pool, store, mut core, mut out, cb) = setup();
    assert!(!idle(&mut core, true));
    out.pump(&mut core, 0).unwrap();
    assert!(idle(&mut core, true));
    assert!(!idle(&mut core, false));
    for flag in 0..3 {
        let old = (
            core.frozen,
            core.separated_anchor_ready,
            core.separated_primed,
        );
        match flag {
            0 => core.frozen = false,
            1 => core.separated_anchor_ready = false,
            _ => core.separated_primed = false,
        }
        assert!(!idle(&mut core, true));
        (
            core.frozen,
            core.separated_anchor_ready,
            core.separated_primed,
        ) = old;
    }
    core.freezing = Some(request(Control::Barrier));
    assert!(!idle(&mut core, true));
    core.freezing.take();
    core.library_intent = Some(AssetId::new(1).unwrap());
    assert!(!idle(&mut core, true));
    core.library_intent = None;
    core.external_drafts.push(DraftId::new(1).unwrap());
    assert!(!idle(&mut core, true));
    core.external_drafts.clear();
    let pending = request(Control::Barrier);
    core.acknowledgments.push((
        core.mark,
        SupplyReceipt {
            id: pending.id,
            result: Ok(SupplyOutcome::Observed),
            _credit: pending.credit,
            retained: vec![],
            retained_revision: 0,
        },
    ));
    assert!(!idle(&mut core, true));
    core.acknowledgments.clear();
    let plan = SoundPlan {
        target: id().into(),
        source: skin_editor_application::values::BlobRef::new([1; 32], 16),
        metadata: Some(
            skin_editor_application::values::AudioMetadata::new(48000, 1, 4096).unwrap(),
        ),
        gain: skin_editor_application::values::Gain::UNITY,
        visible: true,
    };
    let jobs = core
        .controller
        .runtime()
        .synchronize(origin(1), &[plan])
        .unwrap();
    core.components = vec![id()];
    core.selected = Some(origin(1));
    assert!(!idle(&mut core, true));
    core.controller
        .runtime()
        .complete(
            jobs[0],
            Ok(PreparedSound::from_source(plan, &[0.125; 4096], &pool).unwrap()),
        )
        .unwrap();
    assert!(idle(&mut core, true));
    trigger(&mut core, &mut out, &store, Channel::Audition);
    assert!(!idle(&mut core, true));
    finish(pool, core, out, cb);
}
type Setup = (
    PcmBudget,
    BlobStore,
    SupplyCore,
    SupplyOutput,
    Box<separated_output::OutputConsumer>,
);
fn setup() -> Setup {
    let pool = PcmBudget::default();
    let (core, owner, consumer) =
        SupplyCore::new_separated(reference(&pool), 48000, 1, pool.clone()).unwrap();
    (
        pool,
        BlobStore::default(),
        core,
        SupplyOutput::Separated(Box::new(owner)),
        consumer,
    )
}
fn apply_output(
    core: &mut SupplyCore,
    out: &mut SupplyOutput,
    store: &BlobStore,
    c: Control,
) -> Option<SupplyReceipt> {
    core.apply_output(out, request(c), store)
}
fn ready(core: &mut SupplyCore, source: Origin, pool: &PcmBudget) {
    let p = SoundPlan {
        target: id().into(),
        source: skin_editor_application::values::BlobRef::new([source.revision as u8; 32], 16),
        metadata: Some(
            skin_editor_application::values::AudioMetadata::new(48000, 1, 4096).unwrap(),
        ),
        gain: skin_editor_application::values::Gain::UNITY,
        visible: true,
    };
    let r = core.controller.runtime();
    let job = r.synchronize(source, &[p]).unwrap()[0];
    r.complete(
        job,
        Ok(PreparedSound::from_source(p, &[0.125; 4096], pool).unwrap()),
    )
    .unwrap();
    core.selected = Some(source);
    core.components = vec![id()];
}
fn scene(
    core: &mut SupplyCore,
    out: &mut SupplyOutput,
    store: &BlobStore,
    running: bool,
) -> Option<SupplyReceipt> {
    let mark = core.output.invalidate_scene().unwrap();
    apply_output(core, out, store, Control::Scene { running, mark })
}
fn start(core: &mut SupplyCore, out: &mut SupplyOutput, store: &BlobStore) {
    out.pump(core, 0).unwrap();
    assert!(matches!(
        scene(core, out, store, true).unwrap().result,
        Ok(SupplyOutcome::Running)
    ));
}
fn trigger(core: &mut SupplyCore, out: &mut SupplyOutput, store: &BlobStore, lane: Channel) {
    let source = core.selected.unwrap();
    assert!(matches!(
        apply_output(
            core,
            out,
            store,
            Control::Trigger {
                origin: source,
                lane,
                component: id(),
                stop_library: None
            }
        )
        .unwrap()
        .result,
        Ok(SupplyOutcome::Triggered(_))
    ));
    out.pump(core, 0).unwrap();
}
fn finish(
    pool: PcmBudget,
    mut core: SupplyCore,
    out: SupplyOutput,
    cb: Box<separated_output::OutputConsumer>,
) {
    let SupplyOutput::Separated(mut owner) = out else {
        unreachable!()
    };
    owner.finish(core.controller.runtime(), cb.stop()).unwrap();
    core.worker.close();
    drop(core);
    assert_eq!(pool.usage().bytes, 0);
}
#[test]
fn queued_skin_invalidation_mutes_old_voice_without_cutting_background() {
    let (pool, store, mut core, mut out, mut cb) = setup();
    ready(&mut core, origin(1), &pool);
    start(&mut core, &mut out, &store);
    trigger(&mut core, &mut out, &store, Channel::Scene);
    let mut values = [0.0; 32];
    cb.render(&mut values).unwrap();
    assert_eq!(values[0], 0.125);
    core.output.invalidate_skin(true, true).unwrap(); // Worker has not handled Select.
    cb.render(&mut values).unwrap();
    assert_eq!(values[0], 32.0 / 32768.0);
    let mark = core.output.mark();
    apply_output(
        &mut core,
        &mut out,
        &store,
        Control::Select(Projection {
            origin: origin(2),
            plans: Vec::new(),
            cancelled: Vec::new(),
            mark,
        }),
    )
    .unwrap()
    .result
    .unwrap();
    out.pump(&mut core, 0).unwrap();
    cb.render(&mut values).unwrap();
    assert_eq!(values[0], 64.0 / 32768.0);
    finish(pool, core, out, cb);
}
#[test]
fn pause_request_fences_before_worker_then_acknowledges_exact_copy_anchor() {
    let (pool, store, mut core, mut out, mut cb) = setup();
    ready(&mut core, origin(1), &pool);
    start(&mut core, &mut out, &store);
    trigger(&mut core, &mut out, &store, Channel::Audition);
    cb.render(&mut [0.0; 32]).unwrap();
    out.pump(&mut core, 0).unwrap();
    let mark = core.output.invalidate_scene().unwrap();
    let mut values = [0.0; 32];
    cb.render(&mut values).unwrap();
    assert_eq!(values, [0.125; 32]); // Frozen background, independent audition.
    assert!(
        apply_output(
            &mut core,
            &mut out,
            &store,
            Control::Scene {
                running: false,
                mark
            }
        )
        .is_none()
    );
    out.pump(&mut core, 0).unwrap();
    assert!(core.finish_freeze().unwrap().is_none());
    cb.render(&mut values).unwrap();
    // Copy acknowledgement alone must not publish completion before the owner
    // has restored the generation frontier/Runtime anchor.
    assert!(core.finish_freeze().unwrap().is_none());
    out.pump(&mut core, 0).unwrap();
    let receipt = core.finish_freeze().unwrap().unwrap();
    assert!(matches!(
        receipt.result,
        Ok(SupplyOutcome::Frozen {
            next_background_frame: 32
        })
    ));
    let mark = core.output.invalidate_skin(false, true).unwrap();
    apply_output(&mut core, &mut out, &store, Control::StopAudition(mark));
    start(&mut core, &mut out, &store);
    cb.render(&mut values).unwrap();
    assert_eq!(values[0], 32.0 / 32768.0);
    finish(pool, core, out, cb);
}
#[test]
fn stop_and_barrier_receipts_wait_for_native_copy_ack_and_do_not_drop_credit() {
    let (pool, store, mut core, mut out, mut cb) = setup();
    ready(&mut core, origin(1), &pool);
    start(&mut core, &mut out, &store);
    trigger(&mut core, &mut out, &store, Channel::Audition);
    let mark = core.output.invalidate_skin(false, true).unwrap();
    assert!(apply_output(&mut core, &mut out, &store, Control::StopAudition(mark)).is_none());
    assert!(apply_output(&mut core, &mut out, &store, Control::Barrier).is_none());
    let mut receipts = Vec::new();
    core.finish_acknowledgments(&mut receipts);
    assert!(receipts.is_empty());
    let mut values = [0.0; 32];
    let report = cb.render(&mut values).unwrap();
    assert_eq!(values[0], 0.0);
    assert!(report.voices.runtime_first.iter().all(Option::is_none));
    out.pump(&mut core, 0).unwrap();
    core.finish_acknowledgments(&mut receipts);
    assert_eq!(receipts.len(), 2);
    assert!(matches!(
        receipts[0].result,
        Ok(SupplyOutcome::AuditionStopped)
    ));
    assert!(matches!(receipts[1].result, Ok(SupplyOutcome::Observed)));
    finish(pool, core, out, cb);
}
#[test]
fn cancelled_draft_remains_reported_until_its_transport_pcm_is_collected() {
    let (pool, store, mut core, mut out, mut cb) = setup();
    let mut source = origin(1);
    source.draft = Some((9, 1));
    ready(&mut core, source, &pool);
    start(&mut core, &mut out, &store);
    trigger(&mut core, &mut out, &store, Channel::Audition);
    let draft = DraftId::new(9).unwrap();
    assert!(core.retained_drafts().contains(&draft));
    let mark = core.output.invalidate_skin(true, true).unwrap();
    apply_output(
        &mut core,
        &mut out,
        &store,
        Control::Select(Projection {
            origin: origin(2),
            plans: Vec::new(),
            cancelled: vec![draft],
            mark,
        }),
    )
    .unwrap()
    .result
    .unwrap();
    out.pump(&mut core, 0).unwrap();
    assert!(core.retained_drafts().contains(&draft));
    cb.render(&mut [0.0; 32]).unwrap();
    assert!(out.pump(&mut core, 0).unwrap()); // Publish retirement without a new request.
    assert!(!core.retained_drafts().contains(&draft));
    finish(pool, core, out, cb);
}
#[test]
fn history_switch_rejects_old_trigger_and_does_not_reset_background() {
    let (pool, store, mut core, mut out, mut cb) = setup();
    let old = origin(1);
    ready(&mut core, old, &pool);
    start(&mut core, &mut out, &store);
    trigger(&mut core, &mut out, &store, Channel::Scene);
    cb.render(&mut [0.0; 32]).unwrap();
    let mut source = old;
    source.navigation_revision += 1;
    let mark = core.output.invalidate_skin(true, true).unwrap();
    apply_output(
        &mut core,
        &mut out,
        &store,
        Control::Select(Projection {
            origin: source,
            plans: Vec::new(),
            cancelled: Vec::new(),
            mark,
        }),
    )
    .unwrap()
    .result
    .unwrap();
    assert!(matches!(
        apply_output(
            &mut core,
            &mut out,
            &store,
            Control::Trigger {
                origin: old,
                lane: Channel::Scene,
                component: id(),
                stop_library: None
            }
        )
        .unwrap()
        .result,
        Err(SupplyError::Stale)
    ));
    out.pump(&mut core, 0).unwrap();
    let mut values = [0.0; 32];
    cb.render(&mut values).unwrap();
    assert_eq!(values[0], 32.0 / 32768.0);
    finish(pool, core, out, cb);
}
#[test]
fn busy_resume_preserves_pending_pause_and_a_retry_recovers_without_old_time() {
    let (pool, store, mut core, mut out, mut cb) = setup();
    start(&mut core, &mut out, &store);
    cb.render(&mut [0.0; 32]).unwrap();
    assert!(scene(&mut core, &mut out, &store, false).is_none());
    assert!(matches!(
        scene(&mut core, &mut out, &store, true).unwrap().result,
        Err(SupplyError::Busy)
    ));
    cb.render(&mut [0.0; 32]).unwrap();
    out.pump(&mut core, 0).unwrap();
    assert!(core.finish_freeze().unwrap().is_some());
    start(&mut core, &mut out, &store);
    let mut values = [0.0; 32];
    cb.render(&mut values).unwrap();
    assert_eq!(values[0], 32.0 / 32768.0);
    finish(pool, core, out, cb);
}
#[test]
fn real_worker_full_unread_control_window_can_always_close_and_finish() {
    let pool = PcmBudget::default();
    let (mut client, cb) = AudioSupplyClient::spawn_separated(
        reference(&pool),
        48000,
        1,
        pool.clone(),
        BlobStore::default(),
    )
    .unwrap();
    assert!(client.output().is_err());
    for _ in 0..MAX_SUPPLY_CONTROLS {
        client.barrier().unwrap();
    }
    assert_eq!(client.barrier(), Err(SupplyError::Busy));
    client.close();
    assert_eq!(client.credits.load(Ordering::Acquire), 0);
    assert!(pool.usage().bytes > 0);
    client.finish_separated(cb.stop()).unwrap();
    assert_eq!(pool.usage().bytes, 0);
}
#[test]
fn foreign_stopped_endpoint_does_not_consume_either_retired_worker() {
    let a = PcmBudget::default();
    let b = PcmBudget::default();
    let (mut c, cb) = AudioSupplyClient::spawn_separated(
        reference(&a),
        48000,
        1,
        a.clone(),
        BlobStore::default(),
    )
    .unwrap();
    let (mut d, db) = AudioSupplyClient::spawn_separated(
        reference(&b),
        48000,
        1,
        b.clone(),
        BlobStore::default(),
    )
    .unwrap();
    c.close();
    d.close();
    let foreign = c.finish_separated(db.stop()).unwrap_err();
    d.finish_separated(foreign).unwrap();
    c.finish_separated(cb.stop()).unwrap();
    assert_eq!((a.usage().bytes, b.usage().bytes), (0, 0));
}
#[test]
fn cancelling_one_draft_keeps_an_unrelated_active_scene_voice() {
    let (pool, store, mut core, mut out, mut cb) = setup();
    ready(&mut core, origin(1), &pool);
    start(&mut core, &mut out, &store);
    trigger(&mut core, &mut out, &store, Channel::Scene);
    let mut draft_source = origin(2);
    draft_source.draft = Some((9, 1));
    ready(&mut core, draft_source, &pool);
    trigger(&mut core, &mut out, &store, Channel::Audition);
    let mark = core.output.invalidate_skin(true, true).unwrap();
    apply_output(
        &mut core,
        &mut out,
        &store,
        Control::Select(Projection {
            origin: origin(3),
            plans: Vec::new(),
            cancelled: vec![DraftId::new(9).unwrap()],
            mark,
        }),
    )
    .unwrap()
    .result
    .unwrap();
    out.pump(&mut core, 0).unwrap();
    let mut values = [0.0; 32];
    cb.render(&mut values).unwrap();
    assert_eq!(values[0], 0.125);
    assert_eq!(core.voice_counts(), [1, 0]);
    finish(pool, core, out, cb);
}
#[test]
fn idle_status_publication_advances_copied_position_without_another_control() {
    let (pool, store, mut core, mut out, mut cb) = setup();
    start(&mut core, &mut out, &store);
    let mut view = core.status();
    assert_eq!(view.copied_background, 0);
    cb.render(&mut [0.0; 32]).unwrap();
    out.pump(&mut core, 0).unwrap();
    core.publish_progress(&mut view);
    assert_eq!(view.copied_background, 32);
    let frontier = view.generated_background;
    cb.render(&mut [0.0; 32]).unwrap();
    core.publish_progress(&mut view);
    assert_eq!(view.copied_background, 64);
    assert_eq!(view.generated_background, frontier);
    finish(pool, core, out, cb);
}
