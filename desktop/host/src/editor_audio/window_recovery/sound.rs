use super::*;
use crate::audio_verification::tone;
use skin_editor_infrastructure::{WavDecoder, background_stream::BufferedBackground};
use std::fs::File;

pub(super) fn component() -> AudioComponentId {
    AudioComponentId::new(42).unwrap()
}
pub(super) fn install(
    editor: &mut Editor,
    path: &Path,
    branched_history: bool,
) -> Result<Rc<RefCell<EditorAudio>>, String> {
    let mut audio = EditorAudio::candidate(editor.images.blobs().clone());
    let probe = PreparedDevice::probe()?;
    let format = probe.format();
    drop(probe);
    if format != (48000, 2) {
        return Err("window plan requires 48k stereo".into());
    }
    let staged = editor
        .images
        .blobs()
        .stage_audio(tone(48000, 2, 440.), &WavDecoder)
        .map_err(error)?;
    let asset = AssetId::new(10000).unwrap();
    if !editor.ui.submit(
        &mut editor.session,
        vec![
            Command::RegisterAsset {
                asset: Asset::new(
                    asset,
                    "self.window".into(),
                    staged.reference(),
                    AssetKind::Audio(staged.audio_metadata().unwrap()),
                )
                .map_err(error)?,
            },
            Command::CreateAudioComponent {
                component: AudioComponent::new(
                    component(),
                    SoundSlotKey::new("self.window".into()).map_err(error)?,
                    asset,
                ),
            },
        ],
    ) {
        return Err(editor.ui.message.clone());
    }
    drop(staged);
    if branched_history {
        init_failure::fixture(editor)?;
    }
    crate::stream_fixture::write(path, 48000, 2)?;
    audio.buffered = Some(
        BufferedBackground::open(
            File::open(path.join("self-background.wav")).map_err(error)?,
            &audio.budget,
        )
        .map_err(error)?,
    );
    let shared = Rc::new(RefCell::new(audio));
    editor.ui.set_audio_port(Box::new(Shared(shared.clone())));
    editor.ui.audio_component = Some(component());
    editor.ui.open_audio_editor();
    editor.ui.paused = true;
    editor
        .ui
        .audio_command(&editor.session, AudioEditorAction::MonitorVolume(200));
    editor
        .ui
        .audio_command(&editor.session, AudioEditorAction::Enable);
    Ok(shared)
}
pub(super) fn reopen(
    editor: &mut Editor,
    audio: &Rc<RefCell<EditorAudio>>,
    path: &Path,
) -> Result<u64, String> {
    let mut a = audio.borrow_mut();
    let anchor = a
        .recovery
        .ok_or("missing window audio anchor")?
        .copied_frame
        .ok_or("missing final copy")?;
    a.buffered = Some(
        BufferedBackground::open(
            File::open(path.join("self-background.wav")).map_err(error)?,
            &a.budget,
        )
        .map_err(error)?,
    );
    drop(a);
    editor.ui.paused = true;
    editor
        .ui
        .audio_command(&editor.session, AudioEditorAction::Enable);
    Ok(anchor)
}
pub(super) fn collect(
    audio: &Rc<RefCell<EditorAudio>>,
    probes: &mut Probes,
    observer: &Observations,
) -> Result<(), String> {
    let mut a = audio.borrow_mut();
    if let Some(c) = &mut a.client {
        let receipts: Vec<_> = std::iter::from_fn(|| c.poll()).collect();
        for receipt in receipts {
            if let SupplyOutcome::Triggered(r) = receipt.result.as_ref().map_err(error)? {
                probes.receipt(receipt.id, *r)?;
            }
            a.receive(receipt.id, &receipt.result);
        }
    }
    drop(a);
    if observer.overflow.load(Ordering::Acquire) {
        return Err("window native observer overflow".into());
    }
    probes.observe(&observer.rows.lock().map_err(error)?, Instant::now())
}
pub(super) fn trigger(
    editor: &mut Editor,
    audio: &Rc<RefCell<EditorAudio>>,
    probes: &mut Probes,
    lane: Channel,
) -> Result<(), String> {
    let at = Instant::now();
    let id = if lane == Channel::Audition {
        editor
            .ui
            .audio_command(&editor.session, AudioEditorAction::Audition(component()));
        audio
            .borrow()
            .last_trigger_request
            .ok_or("missing window audition request")?
    } else {
        audio
            .borrow_mut()
            .client
            .as_mut()
            .ok_or("missing window supply")?
            .trigger_selected(lane, component())
            .map_err(error)?
    };
    probes.begin(
        id,
        lane,
        at,
        crate::edit_verification::source(&editor.session),
        true,
    )
}
pub(super) fn rows(probes: &Probes) -> String {
    format!(
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
    )
}
