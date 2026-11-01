use skin_editor_audio::*;
use skin_editor_domain::*;
#[test]
fn monitoring_conversion_refuses_unreviewed_multichannel_layouts() {
    let mut runtime = Runtime::new(48000, 2).unwrap();
    runtime.enable_minimum_conversion();
    let mut p = plan();
    p.metadata = Some(AudioMetadata::new(48000, 3, 4).unwrap());
    let job = runtime.synchronize(origin(0), &[p]).unwrap()[0];
    let sound = PreparedSound::from_source(p, &[0.25; 12], &runtime.pcm_budget()).unwrap();
    assert_eq!(
        runtime.complete(job, Ok(sound)),
        Ok(Completion::Failed(PrepareFailure::FormatMismatch))
    );
}
fn plan() -> SoundPlan {
    SoundPlan {
        target: (AudioComponentId::new(1).unwrap()).into(),
        source: BlobRef::new([1; 32], 8),
        metadata: Some(AudioMetadata::new(48000, 1, 4).unwrap()),
        gain: Gain::UNITY,
        visible: true,
    }
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
#[test]
fn reservation_precedes_allocation_and_cancel_returns_the_exact_credit() {
    let pool = PcmBudget::new(32, 16).unwrap();
    let a = pool.reserve(4).unwrap();
    assert_eq!(pool.usage().bytes, 16);
    let b = pool.reserve(4).unwrap();
    assert_eq!(pool.usage().bytes, 32);
    assert!(matches!(pool.reserve(1), Err(PcmError::ResourceLimit)));
    drop(b);
    assert_eq!(pool.usage().bytes, 16);
    let buffer = a.allocate();
    assert_eq!(buffer.samples(), [0.0; 4]);
    assert_eq!(pool.usage().bytes, 16);
    drop(buffer);
    assert_eq!(pool.usage().bytes, 0);
}
#[test]
fn clones_and_weak_cache_references_count_one_allocation_until_the_last_owner() {
    let pool = PcmBudget::new(16, 16).unwrap();
    let a = pool.reserve(4).unwrap().allocate();
    let b = a.clone();
    let weak = a.downgrade();
    assert!(a.same_allocation(&b));
    assert_eq!(pool.usage().bytes, 16);
    drop(a);
    assert!(weak.is_alive());
    drop(b);
    assert!(!weak.is_alive());
    assert_eq!(pool.usage().bytes, 0);
}
#[test]
fn default_pool_enforces_real_32_mib_total_and_4_mib_single_sound() {
    let pool = PcmBudget::default();
    assert!(pool.reserve(MAX_SOUND_SAMPLES + 1).is_err());
    assert!(pool.reserve(usize::MAX).is_err());
    assert_eq!(pool.usage().peak_bytes, 0);
    let allocations: Vec<_> = (0..8)
        .map(|_| pool.reserve(MAX_SOUND_SAMPLES).unwrap().allocate())
        .collect();
    assert_eq!(pool.usage().bytes, 32 * 1024 * 1024);
    assert!(pool.reserve(1).is_err());
    assert_eq!(pool.usage().peak_bytes, 32 * 1024 * 1024);
    drop(allocations);
    assert_eq!(pool.usage().bytes, 0);
}
#[test]
fn shared_pcm_cannot_mutate_or_create_an_unaccounted_copy_on_write() {
    let pool = PcmBudget::new(16, 16).unwrap();
    let mut a = pool.reserve(4).unwrap().allocate();
    let b = a.clone();
    assert_eq!(a.samples_mut(), Err(PcmError::SharedBuffer));
    assert_eq!(pool.usage().bytes, 16);
    drop(b);
    a.samples_mut().unwrap()[0] = 0.5;
    assert_eq!(a.samples()[0], 0.5);
}
#[test]
fn runtime_rejects_samples_paid_for_by_an_unrelated_pool() {
    let own = PcmBudget::new(16, 16).unwrap();
    let other = PcmBudget::new(16, 16).unwrap();
    let mut runtime = Runtime::with_budget(48000, 1, own.clone()).unwrap();
    let job = runtime.synchronize(origin(0), &[plan()]).unwrap()[0];
    let sound = PreparedSound::from_source(plan(), &[0.25; 4], &other).unwrap();
    assert_eq!(
        runtime.complete(job, Ok(sound)),
        Err(RuntimeError::InvalidPrepared)
    );
    assert_eq!(other.usage().bytes, 0);
    assert_eq!(own.usage().bytes, 0);
}
#[test]
fn started_voice_keeps_credit_after_binding_removal_until_actual_completion() {
    let pool = PcmBudget::new(16, 16).unwrap();
    let mut runtime = Runtime::with_budget(48000, 1, pool.clone()).unwrap();
    let job = runtime.synchronize(origin(0), &[plan()]).unwrap()[0];
    let sound = PreparedSound::from_source(plan(), &[0.25; 4], &pool).unwrap();
    runtime.complete(job, Ok(sound)).unwrap();
    runtime
        .set_scene_running(runtime.token(Channel::Scene), true)
        .unwrap();
    runtime
        .trigger(runtime.token(Channel::Scene), plan().target, 0)
        .unwrap();
    runtime.synchronize(origin(1), &[]).unwrap();
    assert_eq!(pool.usage().bytes, 16);
    assert!(pool.reserve(1).is_err());
    runtime.mix(&mut [0.0; 4]).unwrap();
    assert_eq!(pool.usage().bytes, 0);
}
#[test]
fn cached_recipe_reuses_full_pool_across_components_and_checks_gain_and_pool_identity() {
    let pool = PcmBudget::new(16, 16).unwrap();
    let a = PreparedSound::from_source(plan(), &[0.25; 4], &pool).unwrap();
    let weak = a.downgrade();
    let mut other = plan();
    other.target = AudioComponentId::new(2).unwrap().into();
    let b = weak.upgrade_for(other, &pool).unwrap();
    assert!(a.pcm().same_allocation(b.pcm()));
    assert_eq!(pool.usage().bytes, 16);
    other.gain = Gain::new(0.5).unwrap();
    assert!(weak.upgrade_for(other, &pool).is_none());
    assert!(weak.upgrade_for(plan(), &PcmBudget::default()).is_none());
    drop(a);
    drop(b);
    assert_eq!(pool.usage().bytes, 0);
    assert!(!weak.is_alive());
}

#[test]
fn output_blocks_share_the_voice_budget_and_cannot_be_modified_when_shared() {
    let pool = PcmBudget::new(32, 16).unwrap();
    let mut runtime = Runtime::with_budget(48000, 1, pool.clone()).unwrap();
    let job = runtime.synchronize(origin(0), &[plan()]).unwrap()[0];
    runtime
        .complete(
            job,
            Ok(PreparedSound::from_source(plan(), &[0.25; 4], &pool).unwrap()),
        )
        .unwrap();
    let mut output = pool.reserve(4).unwrap().allocate();
    assert_eq!(pool.usage().bytes, 32);
    assert!(pool.reserve(1).is_err());
    let shared = output.clone();
    assert_eq!(
        runtime.mix_buffer(&mut output),
        Err(RuntimeError::InvalidBlock)
    );
    drop(shared);
    runtime
        .trigger(runtime.token(Channel::Audition), plan().target, 0)
        .unwrap();
    runtime.mix_buffer(&mut output).unwrap();
    assert_eq!(output.samples(), [0.25; 4]);
    drop(output);
    assert_eq!(pool.usage().bytes, 16);
    runtime.shutdown();
    assert_eq!(pool.usage().bytes, 0);
}
