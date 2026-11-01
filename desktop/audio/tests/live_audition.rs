use skin_editor_audio::*;
use skin_editor_domain::{AudioComponentId, AudioMetadata, BlobRef, Gain};

fn origin(revision: u64, draft: Option<(u64, u64)>) -> Origin {
    Origin {
        session: 1,
        node: 1,
        revision,
        navigation_revision: 0,
        draft,
    }
}
fn plan(gain: f64) -> SoundPlan {
    SoundPlan {
        target: AudioComponentId::new(1).unwrap().into(),
        source: BlobRef::new([1; 32], 256),
        metadata: Some(AudioMetadata::new(48000, 1, 128).unwrap()),
        gain: Gain::new(gain).unwrap(),
        visible: true,
    }
}
fn ready() -> Runtime {
    let mut r = Runtime::new(48000, 1).unwrap();
    let p = plan(0.25);
    let job = r.synchronize(origin(1, None), &[p]).unwrap()[0];
    let sound = PreparedSound::from_source(p, &[0.8; 128], &r.pcm_budget()).unwrap();
    r.complete(job, Ok(sound)).unwrap();
    r
}
fn mix(r: &mut Runtime) -> f32 {
    let mut samples = [0.; 4];
    r.mix(&mut samples).unwrap();
    assert!(samples.iter().all(|v| (*v - samples[0]).abs() < 0.000001));
    samples[0]
}
fn trigger(r: &mut Runtime, lane: Channel) {
    r.trigger(
        r.token(lane),
        AudioComponentId::new(1).unwrap(),
        r.mix_frame(),
    )
    .unwrap();
}
#[test]
fn binding_replacement_revokes_active_audition_without_replaying_new_sound() {
    let mut r = ready();
    trigger(&mut r, Channel::Audition);
    assert!((mix(&mut r) - 0.2).abs() < 0.000001);
    let mut replacement = plan(0.25);
    replacement.source = BlobRef::new([2; 32], 256);
    r.synchronize(origin(2, None), &[replacement]).unwrap();
    assert_eq!(mix(&mut r), 0.0);
    assert_eq!(r.voices().count(), 0);
}
#[test]
fn gain_drag_zero_restore_and_cancel_keep_the_same_audition_cursor() {
    let mut r = ready();
    trigger(&mut r, Channel::Audition);
    assert!((mix(&mut r) - 0.2).abs() < 0.000001);
    let first = r.voices().next().unwrap();
    r.synchronize(origin(1, Some((8, 1))), &[plan(0.)]).unwrap();
    assert_eq!(mix(&mut r), 0.0);
    r.synchronize(origin(1, Some((8, 2))), &[plan(1.)]).unwrap();
    assert!((mix(&mut r) - 0.8).abs() < 0.000001);
    r.synchronize(origin(1, None), &[plan(0.25)]).unwrap();
    assert!((mix(&mut r) - 0.2).abs() < 0.000001);
    let last = r.voices().next().unwrap();
    assert_eq!(first.id, last.id);
    assert_eq!(last.cursor, first.cursor + 12);
    assert_eq!(last.origin, origin(1, None));
}

#[test]
fn audition_started_at_zero_can_become_audible_without_another_decode_or_voice() {
    let mut r = Runtime::new(48000, 1).unwrap();
    let p = plan(0.);
    let job = r.synchronize(origin(1, None), &[p]).unwrap()[0];
    let sound = PreparedSound::from_source(p, &[0.8; 128], &r.pcm_budget()).unwrap();
    assert_eq!(sound.samples(), [0.8; 128]);
    r.complete(job, Ok(sound)).unwrap();
    trigger(&mut r, Channel::Audition);
    assert_eq!(mix(&mut r), 0.);
    let id = r.voices().next().unwrap().id;
    r.synchronize(origin(2, None), &[plan(1.)]).unwrap();
    assert!((mix(&mut r) - 0.8).abs() < 0.000001);
    assert_eq!(r.voices().next().unwrap().id, id);
    assert_eq!(r.pcm_budget().usage().bytes, 512);
}

#[test]
fn live_gain_does_not_exempt_history_navigation_or_late_preparation() {
    let mut r = ready();
    trigger(&mut r, Channel::Audition);
    let job = r
        .synchronize(origin(1, Some((8, 1))), &[plan(0.5)])
        .unwrap()[0];
    let next = Origin {
        navigation_revision: 1,
        ..origin(1, None)
    };
    r.synchronize_cancelled(next, &[plan(0.25)], &[8]).unwrap();
    assert_eq!(mix(&mut r), 0.);
    let old = PreparedSound::from_source(job.plan, &[0.8; 128], &r.pcm_budget()).unwrap();
    assert_eq!(r.complete(job, Ok(old)), Ok(Completion::Stale));
    assert_eq!(r.voices().count(), 0);
    r.shutdown();
    assert_eq!(r.pcm_budget().usage().bytes, 0);
}
