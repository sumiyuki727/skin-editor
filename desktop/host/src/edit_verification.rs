//! Explicit release-only measurements using self-authored files and the real supply path.
use crate::audio_verification::{component, submit};
use crate::load_verification::{consume, outcome, phase, stop, wait, wav};
use skin_editor_application::{values::*, *};
use skin_editor_audio::{Channel, Origin, PcmBudget, Preparation};
use skin_editor_infrastructure::{
    AdmissionState, AudioSupplyClient, BlobStore, NativeOutput, OutputControl, SupplyOutcome,
};
use std::io::Write;
use std::time::Instant;

fn error(e: impl std::fmt::Debug) -> String {
    format!("{e:?}")
}
fn draft(session: &mut ProjectSession, next: &mut u128, action: DraftAction) -> Result<(), String> {
    *next += 1;
    session
        .edit_draft(DraftRequest {
            api_version: API_VERSION,
            session_id: session.project_info().session_id,
            request_id: RequestId::new(*next).unwrap(),
            action,
        })
        .map_err(error)?;
    Ok(())
}
fn ds(session: &ProjectSession) -> DraftState {
    session
        .draft_info(session.project_info().session_id)
        .unwrap()
        .unwrap()
        .state
}
#[derive(Default)]
struct Capture(Option<Origin>);
impl SkinAudioPort for Capture {
    fn synchronize(
        &mut self,
        _: &SkinProject,
        s: SkinAudioSource,
        _: &[DraftId],
    ) -> Result<(), SkinAudioError> {
        self.0 = Some(Origin {
            session: s.session.get(),
            node: s.state.head.get(),
            revision: s.state.revision,
            navigation_revision: s.navigation_revision,
            draft: s.draft.map(|d| (d.id.get(), d.revision)),
        });
        Ok(())
    }
}
pub(super) fn source(session: &ProjectSession) -> Origin {
    let mut capture = Capture::default();
    session.synchronize_audio(&mut capture).unwrap();
    capture.0.unwrap()
}
fn ready(
    client: &mut AudioSupplyClient,
    output: &OutputControl,
    expected: Origin,
) -> Result<(), String> {
    let mut failure = None;
    wait(output, || {
        while let Some(r) = client.poll() {
            if let Err(e) = r.result {
                failure = Some(error(e));
            }
        }
        match client.status() {
            Ok(s) => {
                if let Some(fault) = s.fault {
                    failure = Some(fault);
                }
                failure.is_some()
                    || (s.origin == Some(expected)
                        && s.bindings.len() == 1
                        && s.bindings[0].1.desired == expected
                        && s.bindings[0].1.effective == Some(expected)
                        && s.bindings[0].1.preparation == Preparation::Ready)
            }
            Err(e) => {
                failure = Some(error(e));
                true
            }
        }
    })?;
    failure.map_or(Ok(()), Err)
}
/// Each row: application edit -> observed Ready, selection receipt, queue,
/// probe/allocation/decode, gain processing, worker result handoff (milliseconds).
fn prepare(
    client: &mut AudioSupplyClient,
    output: &OutputControl,
    session: &ProjectSession,
    started: Instant,
) -> Result<[f64; 6], String> {
    let expected = source(session);
    session.synchronize_audio(client).map_err(error)?;
    let mut admission = None;
    let mut failure = None;
    wait(output, || {
        while let Some(r) = client.poll() {
            match r.result {
                Ok(SupplyOutcome::Selected(a)) if a.len() == 1 => admission = Some(a[0]),
                other => failure = Some(format!("unexpected preparation receipt: {other:?}")),
            }
        }
        admission.is_some() || failure.is_some()
    })?;
    if let Some(e) = failure {
        return Err(e);
    }
    let selected_ms = started.elapsed().as_secs_f64() * 1000.0;
    let admitted = admission.unwrap();
    if admitted.status != Ok(AdmissionState::Queued) || admitted.request.origin != expected {
        return Err(format!(
            "expected a real cache miss for current source: {admitted:?}"
        ));
    }
    ready(client, output, expected)?;
    let elapsed = started.elapsed().as_secs_f64() * 1000.0;
    let observation = client
        .status()
        .map_err(error)?
        .last_preparation
        .ok_or("missing worker observation")?;
    if observation.request != admitted.request {
        return Err("worker observation source mismatch".into());
    }
    Ok([
        elapsed,
        selected_ms,
        observation.queue_wait.as_secs_f64() * 1000.0,
        observation.decode_and_allocate.as_secs_f64() * 1000.0,
        observation.process.as_secs_f64() * 1000.0,
        observation.result_handoff.as_secs_f64() * 1000.0,
    ])
}
fn trigger(
    client: &mut AudioSupplyClient,
    output: &OutputControl,
    session: &ProjectSession,
    started: Instant,
) -> Result<f64, String> {
    let baseline = output.stats().lane_nonzero_frames;
    let id = client
        .trigger(session, Channel::Audition, component())
        .map_err(error)?;
    match outcome(client, id)? {
        SupplyOutcome::Triggered(r)
            if r.origin == source(session) && !r.stale && r.voice.is_some() => {}
        other => return Err(format!("new source did not trigger: {other:?}")),
    }
    wait(output, || {
        output.stats().lane_nonzero_frames[1] > baseline[1]
    })?;
    if output.stats().lane_nonzero_frames[0] != baseline[0] {
        return Err("audition leaked into scene".into());
    }
    Ok(started.elapsed().as_secs_f64() * 1000.0)
}
fn settled_pcm(budget: &PcmBudget, native_bytes: usize, frames: u64) -> Result<(), String> {
    if budget.usage().bytes != native_bytes + frames as usize * 2 * 4 {
        return Err(format!(
            "old PCM lease retained at settled boundary: {:?}",
            budget.usage()
        ));
    }
    Ok(())
}
fn health(
    client: &AudioSupplyClient,
    output: &OutputControl,
    name: &str,
) -> Result<[u64; 4], String> {
    let s = output.stats();
    let values = [
        s.empty_frames,
        s.empty_callbacks,
        s.max_empty_callback_frames,
        client.status().map_err(error)?.max_supply_gap_micros,
    ];
    eprintln!(
        "{name} [empty_frames, empty_callbacks, max_empty_callback_frames, max_supply_gap_us]={values:?}"
    );
    Ok(values)
}
fn restored(before: &ProjectSnapshot, after: &ProjectSnapshot) -> bool {
    before.project == after.project
        && before.session_id == after.session_id
        && before.state.head == after.state.head
        && before.state.revision < after.state.revision
}
pub fn run(destination: &str) -> Result<(), String> {
    let path = std::path::Path::new(destination);
    if path.exists() {
        return Err("edit diagnostic output must be new".into());
    }
    std::fs::create_dir_all(path).map_err(error)?;
    phase("setup");
    let budget = PcmBudget::default();
    let store = BlobStore::default();
    let mut session =
        ProjectSession::with_resources(1, 1, Box::new(store.clone()), SessionLimits::default())
            .map_err(error)?;
    let mut next = 0;
    let mut originals = Vec::new();
    let mut commands = Vec::new();
    let mut files = Vec::new();
    for n in 0..16 {
        let file = path.join(format!("self-{n:02}.wav"));
        std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&file)
            .map_err(error)?
            .write_all(&wav(n))
            .map_err(error)?;
        // Explicit filesystem warming, outside every measurement. No cold-disk claim.
        drop(std::fs::read(&file).map_err(error)?);
        let raw = store.stage_wav(&file).map_err(error)?;
        commands.push(Command::RegisterAsset {
            asset: Asset::new(
                AssetId::new(u128::from(n) + 1).unwrap(),
                format!("self-{n:02}.wav"),
                raw.reference(),
                AssetKind::Audio(raw.audio_metadata().unwrap()),
            )
            .unwrap(),
        });
        originals.push(raw);
        files.push(file);
    }
    commands.push(Command::CreateAudioComponent {
        component: AudioComponent::new(
            component(),
            SoundSlotKey::new("self.edit".into()).unwrap(),
            AssetId::new(5).unwrap(),
        ),
    });
    submit(&mut session, &mut next, commands)?;
    let native = NativeOutput::open(&budget)?;
    let (rate, channels) = (native.rate, native.channels);
    let native_bytes = budget.usage().bytes;
    let period_frames = native.period_frames;
    let idle_reserve_frames = native.producer.idle_reserve_frames();
    let (stream, producer) = native.into_parts();
    let mut client =
        AudioSupplyClient::spawn(producer, rate, channels, budget.clone(), store.clone())
            .map_err(error)?;
    let output = client.output().map_err(|e| format!("{e:?}"))?;
    output.capture_startup()?;
    stream.play()?;
    prepare(&mut client, &output, &session, Instant::now())?;
    stop(&mut client, &output)?;
    let steady_start = output.stats().empty_frames;
    let mut phase_health = vec![health(&client, &output, "startup")?];
    phase("idle");
    consume(&output, rate, 3000)?;
    phase_health.push(health(&client, &output, "idle")?);
    let nodes = session.history_len();
    let expected = session.project_info().state;
    draft(&mut session, &mut next, DraftAction::Begin { expected })?;
    let mut hot = Vec::new();
    let mut hot_native = Vec::new();
    phase("hot_edits");
    for i in 0..300 {
        stop(&mut client, &output)?;
        let expected = ds(&session);
        let started = Instant::now();
        draft(
            &mut session,
            &mut next,
            DraftAction::Replace {
                expected,
                commands: vec![Command::SetAudioProperties {
                    component: component(),
                    gain: Gain::new(0.2 + (i + 1) as f64 / 500.0).unwrap(),
                    visible: true,
                }],
            },
        )?;
        hot.push(prepare(&mut client, &output, &session, started)?);
        hot_native.push(trigger(&mut client, &output, &session, started)?);
        stop(&mut client, &output)?;
        settled_pcm(&budget, native_bytes, 24000)?;
        if session.history_len() != nodes {
            return Err("draft produced a history node".into());
        }
    }
    let expected = ds(&session);
    draft(&mut session, &mut next, DraftAction::Commit { expected })?;
    session.synchronize_audio(&mut client).map_err(error)?;
    ready(&mut client, &output, source(&session))?;
    if session.history_len() != nodes + 1 {
        return Err("300 edits did not commit exactly once".into());
    }
    phase_health.push(health(&client, &output, "hot_edits")?);
    let mut warm = Vec::new();
    let mut reads = Vec::new();
    let mut warm_native = Vec::new();
    phase("warm_prepare");
    for i in 0..64 {
        stop(&mut client, &output)?;
        let n = i % 16;
        let read = Instant::now();
        let raw = store.stage_wav(&files[n]).map_err(error)?;
        reads.push(read.elapsed().as_secs_f64() * 1000.0);
        if raw.reference() != originals[n].reference() {
            return Err("warm raw reuse mismatch".into());
        }
        drop(raw);
        let started = Instant::now();
        submit(
            &mut session,
            &mut next,
            vec![
                Command::SetAudioAsset {
                    component: component(),
                    asset: AssetId::new(n as u128 + 1).unwrap(),
                },
                Command::SetAudioProperties {
                    component: component(),
                    gain: Gain::UNITY,
                    visible: true,
                },
            ],
        )?;
        warm.push(prepare(&mut client, &output, &session, started)?);
        warm_native.push(trigger(&mut client, &output, &session, started)?);
        stop(&mut client, &output)?;
        settled_pcm(
            &budget,
            native_bytes,
            originals[n].audio_metadata().unwrap().frames(),
        )?;
    }
    phase_health.push(health(&client, &output, "warm_prepare")?);
    phase("cleanup");
    let before = session.snapshot();
    let nodes = session.history_len();
    let expected = session.project_info().state;
    draft(&mut session, &mut next, DraftAction::Begin { expected })?;
    let expected = ds(&session);
    draft(
        &mut session,
        &mut next,
        DraftAction::Replace {
            expected,
            commands: vec![Command::SetAudioProperties {
                component: component(),
                gain: Gain::new(0.37).unwrap(),
                visible: true,
            }],
        },
    )?;
    prepare(&mut client, &output, &session, Instant::now())?;
    trigger(&mut client, &output, &session, Instant::now())?;
    let expected = ds(&session);
    draft(&mut session, &mut next, DraftAction::Cancel { expected })?;
    session.synchronize_audio(&mut client).map_err(error)?;
    ready(&mut client, &output, source(&session))?;
    wait(&output, || {
        let s = output.stats();
        s.lane_acknowledged == s.lane_epochs
    })?;
    let cancelled = output.stats().lane_nonzero_frames;
    consume(&output, rate, 100)?;
    if output.stats().lane_nonzero_frames != cancelled
        || client.status().map_err(error)?.voices != [0, 0]
        || session.snapshot() != before
        || session.history_len() != nodes
    {
        return Err("draft cancellation leaked".into());
    }
    for action in [HistoryAction::Undo, HistoryAction::Redo { child: None }] {
        trigger(&mut client, &output, &session, Instant::now())?;
        next += 1;
        session
            .navigate(HistoryRequest {
                api_version: API_VERSION,
                session_id: session.project_info().session_id,
                request_id: RequestId::new(next).unwrap(),
                expected: session.project_info().state,
                action,
            })
            .map_err(error)?;
        session.synchronize_audio(&mut client).map_err(error)?;
        ready(&mut client, &output, source(&session))?;
        wait(&output, || {
            let s = output.stats();
            s.lane_acknowledged == s.lane_epochs
        })?;
        let prior = output.stats().lane_nonzero_frames;
        consume(&output, rate, 100)?;
        if prior != output.stats().lane_nonzero_frames
            || client.status().map_err(error)?.voices != [0, 0]
        {
            return Err("history retained old sound".into());
        }
    }
    if !restored(&before, &session.snapshot()) || session.history_len() != nodes {
        return Err("history round trip changed content".into());
    }
    settled_pcm(&budget, native_bytes, 96000)?;
    consume(&output, rate, 1000)?;
    let retimed_idle_frames = output.stats().retimed_idle_frames;
    let startup_observations = output.startup_observations();
    let stats = output.stats();
    phase_health.push(health(&client, &output, "cleanup")?);
    if stats.failed {
        return Err(format!("callback failed/underflow: {stats:?}"));
    }
    let raw_bytes = store.usage().map_err(error)?.bytes;
    client.close();
    drop(client);
    drop(stream);
    drop(output);
    if budget.usage().bytes != 0 {
        return Err("PCM retained after close".into());
    }
    drop(originals);
    drop(session);
    if store.usage().map_err(error)?.bytes != 0 {
        return Err("raw leases retained after session close".into());
    }
    phase("closed");
    std::thread::sleep(std::time::Duration::from_secs(2));
    let report = format!(
        "{{\"schema\":1,\"period_frames\":{period_frames},\"idle_reserve_frames\":{idle_reserve_frames},\"retimed_idle_frames\":{retimed_idle_frames},\"startup_observations\":{startup_observations:?},\"rate\":{rate},\"channels\":{channels},\"hot_ms\":{hot:?},\"hot_native_upper_ms\":{hot_native:?},\"warm_ms\":{warm:?},\"warm_file_stage_ms\":{reads:?},\"warm_native_upper_ms\":{warm_native:?},\"queued_hot\":300,\"queued_warm\":64,\"new_origin_triggers\":364,\"draft_single_commit\":true,\"cancel_history_cleanup\":true,\"settled_old_pcm_released\":true,\"steady_empty_frames\":{},\"startup_empty_frames\":{steady_start},\"phase_health\":{phase_health:?},\"empty_callbacks\":{},\"max_empty_callback_frames\":{},\"pcm_peak_bytes\":{},\"pcm_final_bytes\":0,\"raw_peak_bytes\":{raw_bytes},\"raw_final_bytes\":0,\"callbacks\":{},\"device_consumption_or_acoustic_verified\":false}}\n",
        stats.empty_frames - steady_start,
        stats.empty_callbacks,
        stats.max_empty_callback_frames,
        budget.usage().peak_bytes,
        stats.callbacks,
    );
    std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path.join("report.json"))
        .map_err(error)?
        .write_all(report.as_bytes())
        .map_err(error)?;
    phase("done");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn history_restoration_checks_content_and_head_but_requires_a_new_revision() {
        let mut session = ProjectSession::new(1, 1).unwrap();
        let mut next = 0;
        submit(
            &mut session,
            &mut next,
            vec![Command::CreateDocument {
                document: ElementDocument::new(
                    DocumentId::new(1).unwrap(),
                    PixelSize::new(1, 1).unwrap(),
                    LayerTree::default(),
                ),
            }],
        )
        .unwrap();
        let before = session.snapshot();
        assert!(!restored(&before, &before));
        for action in [HistoryAction::Undo, HistoryAction::Redo { child: None }] {
            next += 1;
            session
                .navigate(HistoryRequest {
                    api_version: API_VERSION,
                    session_id: session.project_info().session_id,
                    request_id: RequestId::new(next).unwrap(),
                    expected: session.project_info().state,
                    action,
                })
                .unwrap();
            if next == 2 {
                assert!(!restored(&before, &session.snapshot()));
            }
        }
        let after = session.snapshot();
        assert_ne!(before, after);
        assert!(restored(&before, &after));
        let mut changed = after.clone();
        changed.project = ProjectSession::new(2, 2).unwrap().snapshot().project;
        assert!(!restored(&before, &changed));
        changed = after;
        changed.state.head = NodeId::new(1).unwrap();
        assert!(!restored(&before, &changed));
    }
}
