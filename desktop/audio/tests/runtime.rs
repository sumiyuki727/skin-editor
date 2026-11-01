use skin_editor_audio::*;
use skin_editor_domain::*;

fn id() -> AudioComponentId {
    AudioComponentId::new(1).unwrap()
}
fn origin(revision: u64) -> Origin {
    Origin {
        session: 7,
        node: revision + 1,
        revision,
        navigation_revision: 0,
        draft: None,
    }
}
fn plan(gain: f64) -> SoundPlan {
    SoundPlan {
        target: (id()).into(),
        source: BlobRef::new([1; 32], 8),
        metadata: Some(AudioMetadata::new(48000, 1, 4).unwrap()),
        gain: Gain::new(gain).unwrap(),
        visible: true,
    }
}
fn runtime() -> Runtime {
    Runtime::new(48000, 1).unwrap()
}
fn select(r: &mut Runtime, o: Origin, p: SoundPlan) -> BuildRequest {
    r.synchronize(o, &[p]).unwrap().pop().unwrap()
}
fn prepared(r: &Runtime, job: BuildRequest, value: f32) -> PreparedSound {
    PreparedSound::from_source(job.plan, &[value; 4], &r.pcm_budget()).unwrap()
}
fn ready(r: &mut Runtime) -> BuildRequest {
    let job = select(r, origin(0), plan(1.0));
    r.complete(job, Ok(prepared(r, job, 0.25))).unwrap();
    job
}
fn fire(r: &mut Runtime, lane: Channel) -> TriggerReceipt {
    r.trigger(r.token(lane), id(), r.mix_frame()).unwrap()
}

#[test]
fn output_activity_covers_future_events_and_voices_but_not_the_independent_scene_clock() {
    let mut r = runtime();
    ready(&mut r);
    assert_eq!(r.output_activity(), [false, false]);
    r.set_scene_running(r.token(Channel::Scene), true).unwrap();
    assert_eq!(r.output_activity(), [false, false]);
    r.mix(&mut [0.0; 16]).unwrap();
    assert_eq!(r.scene_frame(), 16);
    r.trigger(r.token(Channel::Scene), id(), r.mix_frame() + 1024)
        .unwrap();
    assert_eq!(r.output_activity(), [true, false]);
    r.set_scene_running(r.token(Channel::Scene), false).unwrap();
    r.trigger(r.token(Channel::Audition), id(), 1024).unwrap();
    assert_eq!(r.output_activity(), [false, true]);
    r.stop_audition(r.token(Channel::Audition)).unwrap();
    assert_eq!(r.output_activity(), [false, false]);
    fire(&mut r, Channel::Audition);
    assert_eq!(r.output_activity(), [false, true]);
    r.mix(&mut [0.0; 4]).unwrap();
    assert_eq!(r.output_activity(), [false, false]);
}

#[test]
fn controlled_completion_order_rejects_late_success_failure_and_duplicate() {
    let mut r = runtime();
    let a = select(&mut r, origin(0), plan(1.0));
    assert_eq!(
        r.trigger(r.token(Channel::Audition), id(), 3),
        Err(RuntimeError::NotReady)
    );
    let b = select(&mut r, origin(1), plan(0.5));
    let old = prepared(&r, a, 0.75);
    let weak = old.pcm().downgrade();
    assert_eq!(r.complete(a, Ok(old)), Ok(Completion::Stale));
    assert!(!weak.is_alive());
    assert_eq!(
        r.complete(a, Err(PrepareFailure::Unsupported)),
        Ok(Completion::Stale)
    );
    let mut silence = [1.0; 2];
    r.mix(&mut silence).unwrap();
    assert_eq!(silence, [0.0; 2]);
    assert_eq!(
        r.complete(b, Ok(prepared(&r, b, 0.5))),
        Ok(Completion::Activated {
            effective_from_mix_frame: 2
        })
    );
    assert_eq!(r.voices().count(), 0);
    assert_eq!(r.queued(), 0);
    assert_eq!(
        r.complete(b, Err(PrepareFailure::InvalidPcm)),
        Ok(Completion::Stale)
    );
    let receipt = fire(&mut r, Channel::Audition);
    assert_eq!(receipt.origin, b.origin);
    assert!(!receipt.stale);
    let mut out = [0.0];
    r.mix(&mut out).unwrap();
    assert_eq!(out, [0.25]);
}
#[test]
fn activation_keeps_started_voices_but_drops_queued_old_versions() {
    let mut r = runtime();
    r.set_scene_running(r.token(Channel::Scene), true).unwrap();
    let a = ready(&mut r);
    fire(&mut r, Channel::Scene);
    let b = select(&mut r, origin(1), plan(0.5));
    let old = fire(&mut r, Channel::Scene);
    assert!(old.stale);
    assert_eq!(old.origin, a.origin);
    let queued = r.trigger(r.token(Channel::Scene), id(), 2).unwrap();
    assert!(queued.queued && queued.stale);
    r.complete(b, Ok(prepared(&r, b, 0.5))).unwrap();
    assert_eq!(r.queued(), 0);
    let new = fire(&mut r, Channel::Scene);
    assert!(!new.stale);
    assert_eq!(new.origin, b.origin);
    let captured: Vec<_> = r.voices().map(|v| v.origin).collect();
    assert_eq!(captured, [a.origin, a.origin, b.origin]);
    let mut out = [0.0; 4];
    r.mix(&mut out).unwrap();
    assert_eq!(out, [0.75; 4]);
    assert_eq!(r.voices().count(), 0);
}
#[test]
fn failure_preserves_explicit_stale_fallback_and_retry_is_serial_scoped() {
    let mut r = runtime();
    let a = ready(&mut r);
    let b = select(&mut r, origin(1), plan(0.5));
    assert_eq!(r.retry(id()), Err(RuntimeError::Busy));
    assert_eq!(
        r.complete(b, Err(PrepareFailure::Unsupported)),
        Ok(Completion::Failed(PrepareFailure::Unsupported))
    );
    let status = r.status(id()).unwrap();
    assert_eq!(status.effective, Some(a.origin));
    assert_eq!(
        status.preparation,
        Preparation::Failed(PrepareFailure::Unsupported)
    );
    assert!(fire(&mut r, Channel::Audition).stale);
    let retry = r.retry(id()).unwrap();
    assert_ne!(retry, b);
    assert_eq!(
        r.complete(b, Ok(prepared(&r, b, 0.8))),
        Ok(Completion::Stale)
    );
    r.complete(retry, Ok(prepared(&r, retry, 0.5))).unwrap();
    assert!(!fire(&mut r, Channel::Audition).stale);
}
#[test]
fn unrelated_content_edit_relabels_cached_pcm_without_repreparing_or_starting_audio() {
    let mut r = runtime();
    let a = ready(&mut r);
    assert!(r.synchronize(origin(1), &[a.plan]).unwrap().is_empty());
    assert_eq!(r.status(id()).unwrap().effective, Some(origin(1)));
    assert_eq!(r.voices().count(), 0);
    let mut out = [1.0; 4];
    r.mix(&mut out).unwrap();
    assert_eq!(out, [0.0; 4]);
    assert_eq!(fire(&mut r, Channel::Audition).origin, origin(1));
}
#[test]
fn scene_freeze_and_resume_neither_stop_audition_nor_replay_scene_events() {
    let mut r = runtime();
    ready(&mut r);
    r.set_scene_running(r.token(Channel::Scene), true).unwrap();
    fire(&mut r, Channel::Scene);
    fire(&mut r, Channel::Audition);
    r.trigger(r.token(Channel::Scene), id(), 2).unwrap();
    let scene_before = r.scene_frame();
    let old = r.token(Channel::Scene);
    r.set_scene_running(old, false).unwrap();
    assert_eq!(r.queued(), 0);
    assert_eq!(
        r.voices().map(|v| v.channel).collect::<Vec<_>>(),
        [Channel::Audition]
    );
    assert_eq!(
        r.trigger(r.token(Channel::Scene), id(), 0),
        Err(RuntimeError::Frozen)
    );
    let mut out = [0.0; 2];
    r.mix(&mut out).unwrap();
    assert_eq!(out, [0.25; 2]);
    assert_eq!(r.scene_frame(), scene_before);
    assert_eq!(r.trigger(old, id(), 2), Err(RuntimeError::Stale));
    r.set_scene_running(r.token(Channel::Scene), true).unwrap();
    r.stop_audition(r.token(Channel::Audition)).unwrap();
    let mut out = [1.0; 4];
    r.mix(&mut out).unwrap();
    assert_eq!(out, [0.0; 4]);
    assert_eq!(r.scene_frame(), scene_before + 4);
}
#[test]
fn stop_audition_and_seek_have_separate_scope_and_old_controls_fail() {
    let mut r = runtime();
    ready(&mut r);
    r.set_scene_running(r.token(Channel::Scene), true).unwrap();
    fire(&mut r, Channel::Scene);
    fire(&mut r, Channel::Audition);
    let stale = r.token(Channel::Audition);
    r.stop_audition(stale).unwrap();
    assert_eq!(
        r.voices().map(|v| v.channel).collect::<Vec<_>>(),
        [Channel::Scene]
    );
    assert_eq!(r.stop_audition(stale), Err(RuntimeError::Stale));
    fire(&mut r, Channel::Audition);
    let old = r.token(Channel::Scene);
    r.seek_scene(old, 123).unwrap();
    assert_eq!(r.scene_frame(), 123);
    assert_eq!(
        r.voices().map(|v| v.channel).collect::<Vec<_>>(),
        [Channel::Audition]
    );
    assert_eq!(r.set_scene_running(old, true), Err(RuntimeError::Stale));
    let mut out = [0.0];
    r.mix(&mut out).unwrap();
    assert_eq!(out, [0.25]);
    assert_eq!(r.scene_frame(), 123);
}
#[test]
fn cancel_draft_clears_all_its_revisions_voices_events_candidates_and_pcm_pins() {
    let mut r = runtime();
    let a = ready(&mut r);
    fire(&mut r, Channel::Audition);
    let draft = Origin {
        draft: Some((9, 1)),
        ..a.origin
    };
    let d = select(&mut r, draft, plan(0.5));
    let sound = prepared(&r, d, 0.8);
    let weak = sound.pcm().downgrade();
    r.complete(d, Ok(sound)).unwrap();
    fire(&mut r, Channel::Audition);
    let d2 = select(
        &mut r,
        Origin {
            draft: Some((9, 2)),
            ..draft
        },
        plan(0.75),
    );
    fire(&mut r, Channel::Audition);
    r.trigger(r.token(Channel::Audition), id(), 3).unwrap();
    let base = select(&mut r, a.origin, a.plan);
    assert_eq!(r.voices().map(|v| v.origin).collect::<Vec<_>>(), [a.origin]);
    assert_eq!(r.queued(), 0);
    assert!(!weak.is_alive());
    assert_eq!(
        r.complete(d2, Ok(prepared(&r, d2, 0.9))),
        Ok(Completion::Stale)
    );
    assert_eq!(
        r.trigger(r.token(Channel::Audition), id(), 0),
        Err(RuntimeError::NotReady)
    );
    r.complete(base, Ok(prepared(&r, base, 0.25))).unwrap();
    assert_eq!(fire(&mut r, Channel::Audition).origin, a.origin);
}
#[test]
fn draft_commit_can_finish_captured_voice_without_relabeling_its_origin() {
    let mut r = runtime();
    let a = ready(&mut r);
    let draft = Origin {
        draft: Some((1, 1)),
        ..a.origin
    };
    let d = select(&mut r, draft, plan(0.5));
    r.complete(d, Ok(prepared(&r, d, 0.5))).unwrap();
    fire(&mut r, Channel::Audition);
    assert!(r.synchronize(origin(1), &[d.plan]).unwrap().is_empty());
    assert_eq!(r.voices().next().unwrap().origin, draft);
    assert_eq!(fire(&mut r, Channel::Audition).origin, origin(1));
}
#[test]
fn history_return_to_same_node_still_clears_both_channels_and_old_results() {
    let mut r = runtime();
    let a = ready(&mut r);
    r.set_scene_running(r.token(Channel::Scene), true).unwrap();
    let mut tick = [0.0; 2];
    r.mix(&mut tick).unwrap();
    let position = r.scene_frame();
    fire(&mut r, Channel::Scene);
    fire(&mut r, Channel::Audition);
    r.trigger(r.token(Channel::Audition), id(), 4).unwrap();
    let old_gate = r.token(Channel::Audition);
    let back = Origin {
        revision: 2,
        navigation_revision: 2,
        ..a.origin
    };
    let target = select(&mut r, back, a.plan);
    assert_eq!(r.voices().count(), 0);
    assert_eq!(r.queued(), 0);
    assert_eq!(r.scene_frame(), position);
    assert_eq!(r.mix_frame(), 2);
    assert_eq!(r.trigger(old_gate, id(), 2), Err(RuntimeError::Stale));
    assert_eq!(
        r.complete(a, Ok(prepared(&r, a, 0.5))),
        Ok(Completion::Stale)
    );
    r.complete(target, Ok(prepared(&r, target, 0.25))).unwrap();
    assert_eq!(fire(&mut r, Channel::Audition).origin, back);
}
#[test]
fn queue_and_voice_limits_report_refusal_and_scheduled_drops() {
    let mut r = runtime();
    ready(&mut r);
    for _ in 0..MAX_EVENTS {
        r.trigger(r.token(Channel::Audition), id(), 2).unwrap();
    }
    assert_eq!(
        r.trigger(r.token(Channel::Audition), id(), 2),
        Err(RuntimeError::ResourceLimit)
    );
    let mut out = [0.0; 3];
    let report = r.mix(&mut out).unwrap();
    assert_eq!(report.started, 32);
    assert_eq!(report.dropped, 32);
    assert_eq!(out[..2], [0.0; 2]);
    assert_eq!(out[2], 1.0);
    assert_eq!(report.clipped_samples, 1);
    assert_eq!(r.voices().count(), MAX_VOICES);
    assert_eq!(
        r.trigger(r.token(Channel::Audition), id(), 3),
        Err(RuntimeError::ResourceLimit)
    );
    r.mix(&mut [0.0; 3]).unwrap();
    assert_eq!(r.voices().count(), 0);
    fire(&mut r, Channel::Audition);
}
#[test]
fn device_rebuild_requires_fresh_controls_and_does_not_start_either_channel() {
    let mut r = runtime();
    ready(&mut r);
    r.set_scene_running(r.token(Channel::Scene), true).unwrap();
    fire(&mut r, Channel::Scene);
    fire(&mut r, Channel::Audition);
    r.trigger(r.token(Channel::Audition), id(), 3).unwrap();
    let old = r.token(Channel::Scene);
    let epoch = r.device_epoch();
    let new = r.invalidate_device(epoch).unwrap();
    assert_eq!(r.voices().count(), 0);
    assert_eq!(r.queued(), 0);
    assert_eq!(r.rebuild_device(epoch), Err(RuntimeError::Stale));
    r.rebuild_device(new).unwrap();
    assert_eq!(r.set_scene_running(old, true), Err(RuntimeError::Stale));
    assert_eq!(
        r.trigger(r.token(Channel::Scene), id(), 0),
        Err(RuntimeError::Frozen)
    );
    let mut out = [1.0; 4];
    r.mix(&mut out).unwrap();
    assert_eq!(out, [0.0; 4]);
    fire(&mut r, Channel::Audition);
}
#[test]
fn prepared_format_and_recipe_mismatch_cannot_activate_wrong_samples() {
    let mut r = runtime();
    let a = select(&mut r, origin(0), plan(1.0));
    let wrong = PreparedSound::from_source(plan(0.5), &[0.25; 4], &r.pcm_budget()).unwrap();
    assert_eq!(r.complete(a, Ok(wrong)), Err(RuntimeError::InvalidPrepared));
    assert_eq!(r.status(id()).unwrap().preparation, Preparation::Pending);
    let mut p = plan(1.0);
    p.metadata = Some(AudioMetadata::new(44100, 1, 4).unwrap());
    let b = select(&mut r, origin(1), p);
    assert_eq!(
        r.complete(b, Ok(prepared(&r, b, 0.5))),
        Ok(Completion::Failed(PrepareFailure::FormatMismatch))
    );
    assert_eq!(
        r.trigger(r.token(Channel::Audition), id(), 2),
        Err(RuntimeError::NotReady)
    );
    assert_eq!(r.queued(), 0);
}
#[test]
fn hidden_component_is_silent_without_preparation_or_a_voice() {
    let mut r = runtime();
    let mut hidden = plan(1.0);
    hidden.visible = false;
    hidden.metadata = None;
    assert!(r.synchronize(origin(0), &[hidden]).unwrap().is_empty());
    let receipt = fire(&mut r, Channel::Audition);
    assert!(receipt.silent);
    assert!(!receipt.queued);
    assert_eq!(r.voices().count(), 0);
    let mut out = [1.0; 2];
    r.mix(&mut out).unwrap();
    assert_eq!(out, [0.0; 2]);
}
#[test]
fn removing_binding_retains_only_started_one_shots_until_their_end() {
    let mut r = runtime();
    r.set_scene_running(r.token(Channel::Scene), true).unwrap();
    let a = select(&mut r, origin(0), plan(1.0));
    let sound = prepared(&r, a, 0.5);
    let weak = sound.pcm().downgrade();
    r.complete(a, Ok(sound)).unwrap();
    fire(&mut r, Channel::Scene);
    r.trigger(r.token(Channel::Scene), id(), 3).unwrap();
    r.synchronize(origin(1), &[]).unwrap();
    assert!(weak.is_alive());
    assert_eq!(r.queued(), 0);
    assert_eq!(
        r.trigger(r.token(Channel::Scene), id(), 0),
        Err(RuntimeError::MissingComponent)
    );
    r.mix(&mut [0.0; 4]).unwrap();
    assert!(!weak.is_alive());
}
#[test]
fn invalid_projection_does_not_destroy_ready_sound_and_shutdown_releases_every_pin() {
    let mut r = runtime();
    let a = select(&mut r, origin(0), plan(1.0));
    let sound = prepared(&r, a, 0.5);
    let weak = sound.pcm().downgrade();
    r.complete(a, Ok(sound)).unwrap();
    assert_eq!(
        r.synchronize(a.origin, &[plan(0.5)]),
        Err(RuntimeError::InvalidOrigin)
    );
    assert_eq!(
        r.synchronize(origin(1), &[plan(1.0); MAX_BINDINGS + 1]),
        Err(RuntimeError::ResourceLimit)
    );
    assert_eq!(r.status(id()).unwrap().effective, Some(a.origin));
    fire(&mut r, Channel::Audition);
    r.trigger(r.token(Channel::Audition), id(), 1).unwrap();
    r.shutdown();
    assert!(!weak.is_alive());
    assert_eq!(r.voices().count(), 0);
    assert_eq!(r.queued(), 0);
    assert_eq!(
        r.complete(a, Ok(prepared(&r, a, 0.25))),
        Ok(Completion::Stale)
    );
    assert_eq!(r.mix(&mut [0.0]), Err(RuntimeError::Closed));
}

#[test]
fn library_candidate_has_distinct_identity_and_obeys_stop_and_source_generations() {
    let mut r = runtime();
    ready(&mut r);
    let project_plan = plan(1.0);
    let mut raw = project_plan;
    raw.target = SoundTarget::Library(AssetId::new(1).unwrap());
    assert_eq!(
        r.synchronize(origin(0), &[raw]),
        Err(RuntimeError::InvalidPrepared)
    );
    let late = r.request_library(raw).unwrap();
    let newest = r.request_library(raw).unwrap();
    assert_eq!(
        r.complete(late, Ok(prepared(&r, late, 0.5))),
        Ok(Completion::Stale)
    );
    r.complete(newest, Ok(prepared(&r, newest, 0.25))).unwrap();
    assert_eq!(r.status(id()).unwrap().preparation, Preparation::Ready);
    assert_eq!(
        r.trigger(r.token(Channel::Scene), raw.target, 0),
        Err(RuntimeError::Stale)
    );
    r.trigger(r.token(Channel::Audition), raw.target, 0)
        .unwrap();
    let mut out = [0.0; 1];
    r.mix(&mut out).unwrap();
    assert_eq!(out, [0.25]);
    r.stop_audition(r.token(Channel::Audition)).unwrap();
    assert!(r.library_status().is_none());
    assert_eq!(
        r.complete(newest, Ok(prepared(&r, newest, 0.5))),
        Ok(Completion::Stale)
    );
    let cancelled = r.request_library(raw).unwrap();
    r.synchronize(origin(1), &[project_plan]).unwrap();
    assert_eq!(
        r.complete(cancelled, Ok(prepared(&r, cancelled, 0.5))),
        Ok(Completion::Stale)
    );
    assert!(r.library_status().is_none());
    assert_eq!(r.voices().count(), 0);
    r.shutdown();
    assert_eq!(r.pcm_budget().usage().bytes, 0);
}
