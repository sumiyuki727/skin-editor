//! Controlled device-error notification, native stop/reopen and final copy anchor.
//! Not a physical unplug test; no GUI or project mutation outside commands.
use super::combined::probes::Probes;
use super::gui_verification::Observations;
use super::*;
use crate::audio_verification::{component, submit, tone};
use crate::load_verification::phase;
use skin_editor_infrastructure::{WavDecoder, background_stream::BufferedBackground};
use std::{
    fs::File,
    path::Path,
    time::{Duration, Instant},
};
fn error(e: impl std::fmt::Debug) -> String {
    format!("{e:?}")
}
fn until(mut poll: impl FnMut() -> Result<bool, String>) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_secs(8);
    loop {
        if poll()? {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err("fault verification state deadline".into());
        }
        // Hardware-observation cadence only, never a race-test oracle.
        std::thread::sleep(Duration::from_millis(1));
    }
}
fn selected(a: &mut EditorAudio, s: &ProjectSession) -> Result<(), String> {
    s.synchronize_audio(a).map_err(error)?;
    until(|| {
        let state = a.poll();
        if let Some(e) = state.error {
            return Err(e);
        }
        Ok(state.frozen
            && !state.pending_controls
            && state.components.len() == 1
            && state.components[0].preparation == AudioPreparationState::Ready
            && state
                .output
                .is_some_and(|o| o.generated_background >= o.copied_background))
    })
}
fn transport(a: &mut EditorAudio, running: bool) -> Result<(), String> {
    a.request(AudioEditorAction::SceneRunning(running))
        .map_err(error)?;
    until(|| {
        let state = a.poll();
        if let Some(e) = state.error {
            return Err(e);
        }
        Ok(state.frozen != running && !state.freezing && !state.pending_controls)
    })
}
fn measure(
    a: &mut EditorAudio,
    s: &ProjectSession,
    observer: &Observations,
    count: u64,
) -> Result<String, String> {
    let mut probes = Probes::default();
    let started = Instant::now();
    for index in 0..count {
        for lane in [Channel::Scene, Channel::Audition] {
            let due = started
                + Duration::from_millis((index * 2 + u64::from(lane == Channel::Audition)) * 80);
            std::thread::sleep(due.saturating_duration_since(Instant::now()));
            let at = Instant::now();
            let client = a.client.as_mut().ok_or("client missing")?;
            let id = client.trigger_selected(lane, component()).map_err(error)?;
            let origin = client
                .status()
                .map_err(error)?
                .origin
                .ok_or("origin missing")?;
            probes.begin(id, lane, at, origin, true)?;
            until(|| {
                let client = a.client.as_mut().ok_or("client missing")?;
                while let Some(r) = client.poll() {
                    if let Ok(SupplyOutcome::Triggered(t)) = &r.result {
                        probes.receipt(r.id, *t)?;
                    } else if let Err(e) = &r.result {
                        return Err(error(e));
                    }
                }
                let rows = observer.rows.lock().map_err(error)?;
                probes.observe(&rows, Instant::now())?;
                if observer.overflow.load(std::sync::atomic::Ordering::Acquire) {
                    return Err("observer overflow".into());
                }
                Ok(probes.empty())
            })?;
        }
    }
    if s.skin_audio_source()
        != source(
            a.client
                .as_ref()
                .unwrap()
                .status()
                .map_err(error)?
                .origin
                .ok_or("source missing")?,
        )
    {
        return Err("source changed".into());
    }
    Ok(format!(
        "[{}]",
        probes
            .complete
            .iter()
            .map(|p| format!(
                "[{},{},{},{},{},{},{},{},{}]",
                p.id,
                usize::from(p.lane == Channel::Audition),
                p.voice,
                p.upper_ms,
                p.joined_ms,
                p.first.callback_nanos,
                p.first.callback_ordinal,
                p.first.offset_frames,
                p.first.callback_frames
            ))
            .collect::<Vec<_>>()
            .join(",")
    ))
}
pub fn run(destination: &str) -> Result<(), String> {
    let path = Path::new(destination);
    std::fs::create_dir(path).map_err(error)?;
    let store = BlobStore::default();
    let raw = store.observe_bytes();
    let mut session =
        ProjectSession::with_resources(1, 91, Box::new(store.clone()), SessionLimits::default())
            .map_err(error)?;
    let mut a = EditorAudio::candidate(store.clone());
    let budget = a.budget.clone();
    let device = PreparedDevice::probe()?;
    let format = device.format();
    drop(device);
    if format != (48000, 2) {
        return Err("fixed fault plan requires 48kHz stereo".into());
    }
    let blob = store
        .stage_audio(tone(format.0, format.1, 440.), &WavDecoder)
        .map_err(error)?;
    let asset = AssetId::new(1).unwrap();
    let mut request = 0;
    submit(
        &mut session,
        &mut request,
        vec![
            Command::RegisterAsset {
                asset: Asset::new(
                    asset,
                    "self-authored".into(),
                    blob.reference(),
                    AssetKind::Audio(blob.audio_metadata().unwrap()),
                )
                .map_err(error)?,
            },
            Command::CreateAudioComponent {
                component: AudioComponent::new(
                    component(),
                    SoundSlotKey::new("self.fault".into()).map_err(error)?,
                    asset,
                ),
            },
        ],
    )?;
    drop(blob);
    crate::stream_fixture::write(path, format.0, format.1)?;
    let open = |a: &EditorAudio| {
        BufferedBackground::open(
            File::open(path.join("self-background.wav")).map_err(error)?,
            &a.budget,
        )
        .map_err(error)
    };
    let input = open(&a)?;
    let first_source = input.observation();
    let mut sources = vec![first_source.clone()];
    a.buffered = Some(input);
    a.request(AudioEditorAction::MonitorVolume(200))
        .map_err(error)?;
    a.request(AudioEditorAction::Enable).map_err(error)?;
    let retired_signal = a.native.as_ref().unwrap().fault_signal();
    let mut rows = Vec::new();
    let mut stats = Vec::new();
    let mut anchors = Vec::new();
    for iteration in 0..2 {
        selected(&mut a, &session)?;
        let native = a.native.as_mut().unwrap();
        let observer = Observations::with_capacity(
            native.take_first_observer().ok_or("observer taken")?,
            192,
        )?;
        let signal = native.fault_signal();
        phase(if iteration == 0 {
            "before_fault"
        } else {
            "after_rebuild"
        });
        let anchor = a.recovery.map(|r| r.copied_frame);
        if anchor.is_some() {
            return Err("recovery anchor not committed".into());
        }
        transport(&mut a, true)?;
        rows.push(measure(&mut a, &session, &observer, 32)?);
        transport(&mut a, false)?;
        let paused = a
            .client
            .as_ref()
            .unwrap()
            .status()
            .map_err(error)?
            .copied_background;
        // Explicit audition remains available while the scene is frozen.
        a.request(AudioEditorAction::Audition(component()))
            .map_err(error)?;
        let barrier = a.client.as_mut().unwrap().barrier().map_err(error)?;
        until(|| {
            let c = a.client.as_mut().unwrap();
            while let Some(r) = c.poll() {
                if r.id == barrier {
                    return r.result.map(|_| true).map_err(error);
                }
            }
            Ok(false)
        })?;
        if a.client
            .as_ref()
            .unwrap()
            .status()
            .map_err(error)?
            .copied_background
            != paused
        {
            return Err("paused background advanced".into());
        }
        transport(&mut a, true)?;
        // Fault with active sounds and unread bounded controls, not after quiet close.
        for lane in [Channel::Scene, Channel::Audition] {
            a.client
                .as_mut()
                .unwrap()
                .trigger_selected(lane, component())
                .map_err(error)?;
        }
        signal.report();
        drop(observer);
        let stopped = a.poll();
        if a.close_failed
            || stopped.enabled
            || !stopped.frozen
            || stopped.error.is_none()
            || a.client.is_some()
            || a.native.is_some()
            || a.goal
        {
            return Err("fault did not close/freeze adapter".into());
        }
        let closed = a
            .closed_stats
            .ok_or("missing stopped endpoint statistics")?;
        if closed.endpoint_returns != 1
            || closed.missing_frames != 0
            || closed.callback_resource_releases != 0
            || closed.endpoint_returns_in_render != 0
            || closed.endpoint_returns_outside_render != 1
        {
            return Err(format!("fault cleanup counters: {closed:?}"));
        }
        let copy = a
            .recovery
            .ok_or("missing recovery checkpoint")?
            .copied_frame
            .ok_or("missing exact final copy anchor")?;
        if copy < paused || copy == 0 || budget.usage().bytes != 0 {
            return Err("fault retained PCM or lost final copy anchor".into());
        }
        anchors.push(copy);
        stats.push(format!("{{\"callbacks\":{},\"missing_frames\":{},\"render_errors\":{},\"observation_overflows\":{},\"device_fault\":{},\"endpoint_returns\":{},\"callback_resource_releases\":{},\"endpoint_returns_in_render\":{},\"endpoint_returns_outside_render\":{}}}",closed.callbacks,closed.missing_frames,closed.render_errors,closed.observation_overflows,closed.device_fault,closed.endpoint_returns,closed.callback_resource_releases,closed.endpoint_returns_in_render,closed.endpoint_returns_outside_render));
        if iteration == 0 {
            if first_source.stats().raw_bytes != 0 {
                return Err("old background reader retained".into());
            }
            phase("rebuilding_frozen");
            if a.request(AudioEditorAction::Enable).is_ok() {
                return Err("background silently reset to empty".into());
            }
            let input = open(&a)?;
            sources.push(input.observation());
            a.buffered = Some(input);
            a.request(AudioEditorAction::Enable).map_err(error)?;
            retired_signal.report();
            if a.native.as_ref().unwrap().stats().device_fault {
                return Err("old signal faulted replacement".into());
            }
            selected(&mut a, &session)?;
            let state = a.client.as_ref().unwrap().status().map_err(error)?;
            if !state.frozen || state.copied_background != copy || state.voices != [0, 0] {
                return Err("rebuild resumed or retained old voices/position".into());
            }
        }
    }
    a.recovery = None;
    drop(a);
    drop(session);
    drop(store);
    if budget.usage().bytes != 0
        || raw.usage().bytes != 0
        || sources.len() != 2
        || sources
            .iter()
            .any(|s| s.stats().raw_bytes != 0 || s.stats().fault)
    {
        return Err("final audio owners survived".into());
    }
    phase("closed");
    std::fs::write(path.join("report.json"),format!("{{\"schema\":1,\"scope\":\"controlled-native-device-error-reopen\",\"physical_unplug\":false,\"full_editor\":false,\"rounds\":2,\"rate\":48000,\"channels\":2,\"rows\":[{}],\"stats\":[{}],\"copy_anchors\":{anchors:?},\"pcm_peak\":{},\"pcm_final\":{},\"raw_peak\":{},\"raw_final\":{},\"old_reader_raw_final\":{},\"old_signal_rejected\":true,\"frozen_rebuild\":true}}",rows.join(","),stats.join(","),budget.usage().peak_bytes,budget.usage().bytes,raw.usage().peak_bytes,raw.usage().bytes,first_source.stats().raw_bytes)).map_err(error)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn recovery_requires_a_confirmed_matching_background_and_keeps_the_copy_position() {
        let mut a = EditorAudio::candidate(BlobStore::default());
        a.recovery = Some(Recovery {
            format: (48000, 2),
            copied_frame: Some(333),
            frames: 1000,
        });
        assert!(a.restore_background((48000, 2)).is_err());
        a.reference = Some(
            BackgroundTrack::new(a.budget.reserve(2000).unwrap().allocate(), 48000, 2).unwrap(),
        );
        assert!(a.restore_background((44100, 2)).is_err());
        assert_eq!(a.reference.as_ref().unwrap().position(), 0);
        a.recovery.as_mut().unwrap().copied_frame = None;
        assert!(a.restore_background((48000, 2)).is_err());
        a.recovery.as_mut().unwrap().copied_frame = Some(333);
        a.restore_background((48000, 2)).unwrap();
        assert_eq!(a.reference.as_ref().unwrap().position(), 333);
        assert!(!a.reference.as_ref().unwrap().is_running());
        a.recovery.as_mut().unwrap().frames = 2000;
        assert!(a.restore_background((48000, 2)).is_err());
        assert_eq!(a.reference.as_ref().unwrap().position(), 333);
        let budget = a.budget.clone();
        drop(a);
        assert_eq!(budget.usage().bytes, 0);
    }
}
