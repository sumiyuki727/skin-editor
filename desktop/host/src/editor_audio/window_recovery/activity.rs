//! Explicit input/fault ordering in an unpublished real Editor; no OS capture.
use super::*;
use eframe::egui_wgpu::wgpu;
use skin_editor_ui::Selection;

pub(super) struct ResultRecord {
    pub record: String,
    pub held: egui::FullOutput,
    pub frame: FrameKey,
}
fn pass(
    e: &mut Editor,
    ctx: &egui::Context,
    events: Vec<egui::Event>,
) -> Result<([u32; 2], u64), String> {
    let mut output = None;
    let draw = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1180., 800.),
            )),
            focused: true,
            events,
            ..Default::default()
        },
        |root| output = Some(e.ui.show(root, &mut e.session, &mut e.host)),
    );
    let output = output.ok_or("no active input output")?;
    let viewport = output.viewport.ok_or("active viewport absent")?;
    e.complete_ui(ctx, output);
    drop(draw);
    Ok(viewport)
}
fn ready_frame(e: &mut Editor, viewport: ([u32; 2], u64)) -> Result<(), String> {
    crate::verification::frame(&mut e.host, &e.ui, &e.session, viewport.0, viewport.1)
}
fn button(pos: egui::Pos2, pressed: bool) -> egui::Event {
    egui::Event::PointerButton {
        pos,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: Default::default(),
    }
}
fn draft(e: &Editor) -> Result<DraftState, String> {
    e.session
        .draft_info(e.session.project_info().session_id)
        .map_err(error)?
        .map(|d| d.state)
        .ok_or("input gesture did not create a draft".into())
}
pub(super) fn exercise(
    e: &mut Editor,
    ctx: &egui::Context,
    audio: &Rc<RefCell<EditorAudio>>,
    observer: &Observations,
    plan: fault::Plan,
) -> Result<ResultRecord, String> {
    let before = init_failure::Stamp::read(&e.session)?;
    // Drain the preceding diagnostic preparation before the real input path.
    e.host.wait_preparation().map_err(error)?;
    e.host
        .device
        .poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: Some(Duration::from_secs(3)),
        })
        .map_err(error)?;
    e.host.poll().map_err(error)?;
    let viewport = pass(e, ctx, vec![])?;
    ready_frame(e, viewport)?;
    let viewport = pass(e, ctx, vec![])?;
    ready_frame(e, viewport)?;
    let canvas = e.ui.canvas_rect.ok_or("no input canvas")?;
    // Below the floating audio window, inside the actual canvas response.
    let point = egui::pos2(canvas.left() + 50., canvas.bottom() - 40.);
    pass(
        e,
        ctx,
        vec![egui::Event::PointerMoved(point), button(point, true)],
    )?;
    let viewport = pass(
        e,
        ctx,
        vec![egui::Event::PointerMoved(point + egui::vec2(24., -8.))],
    )?;
    let d = draft(e)?;
    let candidate = e
        .session
        .draft_snapshot(e.session.project_info().session_id, d)
        .map_err(error)?;
    let Some(Selection::Layer(document, layer)) = e.ui.selection() else {
        return Err("active fixture has no selected layer".into());
    };
    let transform = candidate
        .project
        .document(document)
        .and_then(|d| d.layers().node(layer))
        .ok_or("active layer absent")?
        .local;
    let base = e.session.snapshot().project;
    if base
        .document(document)
        .and_then(|d| d.layers().node(layer))
        .map(|n| n.local)
        == Some(transform)
    {
        return Err("canvas gesture did not change its visual transform".into());
    }
    if !e.ui.draft(
        &mut e.session,
        DraftAction::Replace {
            expected: d,
            commands: vec![
                Command::SetLayerTransform {
                    document,
                    layer,
                    transform,
                },
                Command::SetAudioProperties {
                    component: sound::component(),
                    gain: Gain::new(0.5).map_err(error)?,
                    visible: true,
                },
            ],
        },
    ) {
        return Err("combined visual/audio draft replacement failed".into());
    }
    let d = draft(e)?;
    if e.session
        .draft_snapshot(e.session.project_info().session_id, d)
        .map_err(error)?
        .project
        .audio_component(sound::component())
        == base.audio_component(sound::component())
    {
        return Err("audio draft properties did not change".into());
    }
    if !before.same_committed(&init_failure::Stamp::read(&e.session)?) {
        return Err("gesture modified committed history".into());
    }
    ready_frame(e, viewport)?;
    let draft_key = e.host.current().ok_or("draft frame absent")?.frame.key;
    let held = egui::Context::default().run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(64., 64.),
            )),
            ..Default::default()
        },
        |ui| e.host.paint(ui, draft_key, ui.max_rect()),
    );
    if held.shapes.is_empty() {
        return Err("draft draw did not hold a real target".into());
    }
    let end = Instant::now() + Duration::from_secs(5);
    loop {
        e.ui.refresh_audio(&e.session);
        if let Some(error) = &e.ui.audio_state.error {
            return Err(error.clone());
        }
        if e.ui.audio_state.source == Some(e.session.skin_audio_source())
            && e.ui
                .audio_state
                .components
                .iter()
                .all(|c| c.preparation == AudioPreparationState::Ready)
            && !e.ui.audio_state.pending_controls
        {
            break;
        }
        if Instant::now() >= end {
            return Err("active draft audio not ready".into());
        }
        std::thread::yield_now();
    }
    // Scene input stays frozen while the independent audition is active.
    if !e.ui.audio_state.frozen {
        return Err("active scene was not frozen".into());
    }
    let mut probes = Probes::default();
    sound::trigger(e, audio, &mut probes, Channel::Audition)?;
    let end = Instant::now() + Duration::from_secs(5);
    loop {
        sound::collect(audio, &mut probes, observer)?;
        if probes.empty() && probes.complete.len() == 1 {
            break;
        }
        if Instant::now() >= end {
            return Err("draft audition was not copied".into());
        }
        std::thread::yield_now();
    }
    let voices = audio
        .borrow()
        .client
        .as_ref()
        .ok_or("no live draft supply")?
        .status()
        .map_err(error)?
        .voices[1];
    if voices == 0 {
        return Err("draft audition already ended before injection".into());
    }
    let pending = e
        .session
        .request_preview(
            &mut e.host,
            PreviewContext {
                target: e.ui.target.ok_or("draft target absent")?,
                draft: Some(d),
                size: [321, 201],
                viewport_generation: 9_000_001,
                scene: 0,
                simulation_micros: 42,
            },
        )
        .map_err(error)?;
    e.host.wait_preparation().map_err(error)?;
    if !e.host.preparation_pending() {
        return Err("draft lacked late preparation".into());
    }
    let gpu = matches!(plan, fault::Plan::DraftGpu | fault::Plan::DraftBoth);
    let sound_fault = matches!(plan, fault::Plan::DraftAudio | fault::Plan::DraftBoth);
    if gpu {
        e.host.loss_signal().report();
        e.host.device.destroy();
        let _ = e.host.device.poll(wgpu::PollType::Poll);
        let _ = e.host.poll();
    }
    if sound_fault {
        audio
            .borrow()
            .native
            .as_ref()
            .ok_or("active native output absent")?
            .fault_signal()
            .report();
    }
    if !e.interrupt_for_fault() || !e.interrupt_for_fault() {
        return Err("active edit cancellation refused".into());
    }
    if e.session
        .draft_info(e.session.project_info().session_id)
        .map_err(error)?
        .is_some()
        || !before.same_committed(&init_failure::Stamp::read(&e.session)?)
    {
        return Err("fault did not preserve the committed project/history".into());
    }
    // Feed the original held pointer's move/release after the interruption.
    let viewport = pass(
        e,
        ctx,
        vec![
            egui::Event::PointerMoved(point + egui::vec2(80., 5.)),
            button(point + egui::vec2(80., 5.), false),
        ],
    )?;
    if !gpu {
        ready_frame(e, viewport)?;
        // Also exercise the no-valid-target admission gate with a real device:
        // invalidating a source must hide it without destroying the renderer.
        e.host.invalidate_source();
        if e.host.current().is_some() {
            return Err("invalidated source remained presentable".into());
        }
        ready_frame(e, viewport)?;
    }
    let stale = egui::Context::default().run_ui(egui::RawInput::default(), |ui| {
        e.host.paint(ui, draft_key, ui.max_rect());
        e.host.paint(ui, pending.key, ui.max_rect());
    });
    if !stale.shapes.is_empty()
        || !before.same_committed(&init_failure::Stamp::read(&e.session)?)
        || e.session
            .draft_info(e.session.project_info().session_id)
            .map_err(error)?
            .is_some()
        || !e.ui.paused
        || !e.preview.paused
    {
        return Err("late pointer/frame changed the cancelled draft".into());
    }
    let growth = init_failure::Stamp::read(&e.session)?
        .requests()
        .saturating_sub(before.requests());
    let upper = probes.complete[0].upper_ms;
    Ok(ResultRecord {
        frame: pending.key,
        held,
        record: format!(
            "{{\"input_path\":\"egui-canvas-drag\",\"gpu_fault\":{gpu},\"audio_fault\":{sound_fault},\"draft_active_before\":true,\"visual_and_audio_changed\":true,\"scene_frozen_before\":true,\"audition_voices_before\":{voices},\"audition_first_copy_upper_ms\":{upper},\"pending_draft_preparation\":true,\"owned_draft_cancelled\":true,\"interrupt_idempotent\":true,\"late_release_rejected\":true,\"old_draft_frames_rejected\":true,\"committed_project_exact\":true,\"history_exact\":true,\"requests_added\":{growth},\"paused\":true}}"
        ),
    })
}
