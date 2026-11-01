//! Controlled errors after the existing session has moved into an Editor.
use super::*;
use restore::{Attempt, Owner, Slot, State};
#[cfg(test)]
mod activity_tests;
#[cfg(test)]
mod endpoint_tests;

#[derive(Clone, Debug, PartialEq)]
pub(super) struct Stamp {
    snapshot: ProjectSnapshot,
    entries: Vec<HistorySummary>,
    projects: Vec<HistoricalSnapshot>,
    statistics: HistoryStatistics,
}
impl Stamp {
    pub fn same_committed(&self, other: &Self) -> bool {
        let mut same = self.clone();
        // Begin/Replace/Cancel are genuine API requests, not project nodes.
        same.statistics.retained_requests = other.statistics.retained_requests;
        same == *other
    }
    pub fn requests(&self) -> usize {
        self.statistics.retained_requests
    }
    pub fn read(session: &ProjectSession) -> Result<Self, String> {
        let info = session.project_info();
        let statistics = session.history_statistics(info.session_id).map_err(error)?;
        if statistics.nodes > 32 {
            return Err("initialization fixture history exceeded bound".into());
        }
        let mut entries = Vec::new();
        let mut projects = Vec::new();
        for id in 1..=statistics.nodes as u64 {
            let node = NodeId::new(id).ok_or("invalid fixture node")?;
            let entry = session
                .history_summary(info.session_id, node)
                .map_err(error)?;
            for address in [entry.id.short_address(), entry.coordinate.short_address()] {
                if session
                    .resolve_history_address(info.session_id, &address)
                    .map_err(error)?
                    != node
                {
                    return Err("history address changed during initialization".into());
                }
            }
            projects.push(
                session
                    .history_snapshot(info.session_id, node)
                    .map_err(error)?,
            );
            entries.push(entry);
        }
        Ok(Self {
            snapshot: session.snapshot(),
            entries,
            projects,
            statistics,
        })
    }
}
pub(super) fn fixture(editor: &mut Editor) -> Result<(), String> {
    for label in [
        b"// retry branch A\n".as_slice(),
        b"// retry branch B\n".as_slice(),
    ] {
        if !editor.ui.submit(
            &mut editor.session,
            vec![Command::InsertConfigRecord {
                index: 0,
                record: ConfigRecord::new(label.to_vec(), ConfigInterpretation::Opaque)
                    .map_err(error)?,
            }],
        ) {
            return Err("history fixture command failed".into());
        }
        editor
            .ui
            .navigation(&mut editor.session, HistoryAction::Undo);
    }
    let stamp = Stamp::read(&editor.session)?;
    if stamp.statistics.branches < 1 || stamp.statistics.redo_preference_entries != 1 {
        return Err("history fixture lacks a remembered fork".into());
    }
    Ok(())
}
pub(super) fn verify_redo(seed: &mut Seed) -> Result<(), String> {
    let before = Stamp::read(&seed.session)?;
    let info = seed.session.project_info();
    let children = seed
        .session
        .history_children(info.session_id, info.state.head, 0, 32)
        .map_err(error)?;
    if children.entries.len() != 2 {
        return Err("retry lost a branch".into());
    }
    let expected = children.entries[1].id;
    seed.ui
        .navigation(&mut seed.session, HistoryAction::Redo { child: None });
    if seed.session.project_info().state.head != expected {
        return Err("retry lost remembered redo path".into());
    }
    seed.ui.navigation(&mut seed.session, HistoryAction::Undo);
    let after = Stamp::read(&seed.session)?;
    if seed.session.project_info().state.head != info.state.head
        || before.entries != after.entries
        || before.projects != after.projects
        || before.snapshot.project != after.snapshot.project
    {
        return Err("history round trip changed content or branches".into());
    }
    Ok(())
}
pub(super) struct Recovered {
    pub seed: Seed,
    pub record: String,
    pub frame: Option<FrameKey>,
    pub loss: crate::presentation::DeviceLoss,
    pub audio: Option<skin_editor_infrastructure::separated_native::NativeFaultSignal>,
}
pub(super) fn run(
    seed: Seed,
    audio: Rc<RefCell<EditorAudio>>,
    path: &Path,
    plan: fault::Plan,
) -> Result<Recovered, String> {
    crate::load_verification::phase("window_initialization_failure");
    let before = Stamp::read(&seed.session)?;
    let anchor = audio
        .borrow()
        .recovery
        .and_then(|r| r.copied_frame)
        .ok_or("missing prior copy anchor")?;
    let slot = Rc::new(RefCell::new(Slot::new(State::seed(seed))));
    let returned = slot.clone();
    let backend = slot
        .borrow()
        .get()
        .ok_or("missing cleanup slot")?
        .open_backend()?;
    let witness = backend.renderer_gone.clone();
    let notice = backend.notice.clone();
    let signal = backend.loss.clone();
    let path = path.to_path_buf();
    let sound = audio.clone();
    let reached = Arc::new(AtomicU8::new(0));
    let observed = reached.clone();
    let failed_key = Arc::new(std::sync::atomic::AtomicU64::new(0));
    let retained_key = failed_key.clone();
    let failed_audio = Rc::new(RefCell::new(None));
    let late_audio = failed_audio.clone();
    let held_outside = Rc::new(RefCell::new(None));
    let outside = held_outside.clone();
    let endpoint_gate = Rc::new(RefCell::new(None));
    let delayed_endpoint = endpoint_gate.clone();
    let activity_record = Rc::new(RefCell::new(None));
    let active_record = activity_record.clone();
    let result = eframe::run_native(
        "Skin Editor - post-handoff initialization failure",
        backend.options,
        Box::new(move |context| {
            let state = context
                .wgpu_render_state
                .as_ref()
                .ok_or_else(|| std::io::Error::other("no failed initialization renderer"))?;
            state
                .renderer
                .write()
                .callback_resources
                .insert(backend::RendererWitness(witness));
            let mut slot = returned.borrow_mut();
            let mut attempt = Attempt::begin(&mut slot, &context.egui_ctx, state, notice)
                .map_err(std::io::Error::other)?;
            attempt.attach_audio(sound.clone());
            attempt
                .editor()
                .host
                .bind_loss(signal)
                .map_err(|e| std::io::Error::other(error(e)))?;
            observed.store(1, Ordering::Release); // the ticket was taken and an Editor owns the session
            if plan != fault::Plan::InitConsumed {
                let e = attempt.editor();
                crate::verification::frame(&mut e.host, &e.ui, &e.session, [128, 128], 8_000_000)
                    .map_err(std::io::Error::other)?;
                let key = e
                    .host
                    .current()
                    .ok_or_else(|| std::io::Error::other("missing failed-init frame"))?
                    .frame
                    .key;
                let held = egui::Context::default().run_ui(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(64., 64.),
                        )),
                        ..Default::default()
                    },
                    |ui| e.host.paint(ui, key, ui.max_rect()),
                );
                let pending = e
                    .session
                    .request_preview(
                        &mut e.host,
                        PreviewContext {
                            target: e.ui.target.ok_or_else(|| {
                                std::io::Error::other("missing initialization target")
                            })?,
                            draft: None,
                            size: [320, 200],
                            viewport_generation: 8_000_001,
                            scene: 0,
                            simulation_micros: 0,
                        },
                    )
                    .map_err(|e| std::io::Error::other(error(e)))?;
                retained_key.store(pending.key.0, Ordering::Release);
                e.host
                    .wait_preparation()
                    .map_err(|e| std::io::Error::other(error(e)))?;
                if !e.host.preparation_pending() {
                    return Err(std::io::Error::other("failed init has no pending resource").into());
                }
                attempt.held = Some(held);
                if plan == fault::Plan::CleanupHeld {
                    // Real pending CPU draw data owns the old GPU target. The
                    // cleanup attempt cannot release another owner's lease.
                    *outside.borrow_mut() = attempt.held.clone();
                    attempt.cleanup_once = true;
                }
                attempt.prepared();
                observed.store(2, Ordering::Release);
            }
            if plan.active_edit()
                || matches!(
                    plan,
                    fault::Plan::InitAudio | fault::Plan::CleanupHeld | fault::Plan::EndpointHeld
                )
            {
                sound.borrow_mut().hold_return = plan == fault::Plan::EndpointHeld;
                let e = attempt.editor();
                sound::reopen(e, &sound, &path).map_err(std::io::Error::other)?;
                let end = Instant::now() + Duration::from_secs(5);
                loop {
                    e.ui.refresh_audio(&e.session);
                    if let Some(error) = &e.ui.audio_state.error {
                        return Err(std::io::Error::other(error.clone()).into());
                    }
                    let a = sound.borrow();
                    let ready = e.ui.audio_state.enabled
                        && e.ui.audio_state.frozen
                        && !e.ui.audio_state.pending_controls
                        && e.ui
                            .audio_state
                            .components
                            .iter()
                            .all(|c| c.preparation == AudioPreparationState::Ready)
                        && a.native.as_ref().is_some_and(|n| n.stats().callbacks > 0);
                    drop(a);
                    if ready {
                        break;
                    }
                    if Instant::now() >= end {
                        return Err(std::io::Error::other("failed init audio not ready").into());
                    }
                    std::thread::yield_now();
                }
                let reader = sound
                    .borrow_mut()
                    .native
                    .as_mut()
                    .ok_or_else(|| std::io::Error::other("failed init native absent"))?
                    .take_first_observer()
                    .ok_or_else(|| std::io::Error::other("failed init observer already taken"))?;
                *late_audio.borrow_mut() = sound.borrow().native.as_ref().map(|n| n.fault_signal());
                attempt.observer =
                    Some(Observations::with_capacity(reader, 8).map_err(std::io::Error::other)?);
                if plan == fault::Plan::EndpointHeld {
                    let mut a = sound.borrow_mut();
                    a.hold_return = false;
                    *delayed_endpoint.borrow_mut() = Some(
                        a.return_gate
                            .take()
                            .ok_or_else(|| std::io::Error::other("missing native return gate"))?,
                    );
                    attempt.cleanup_once = true;
                }
                observed.store(3, Ordering::Release);
            }
            if plan.active_edit() {
                let observer = attempt
                    .observer
                    .take()
                    .ok_or_else(|| std::io::Error::other("active edit observer absent"))?;
                let result = activity::exercise(
                    attempt.editor(),
                    &context.egui_ctx,
                    &sound,
                    &observer,
                    plan,
                );
                attempt.observer = Some(observer);
                let result = result.map_err(std::io::Error::other)?;
                retained_key.store(result.frame.0, Ordering::Release);
                attempt.held = Some(result.held);
                *active_record.borrow_mut() = Some(result.record);
            }
            Err(std::io::Error::other("controlled post-handoff initialization failure").into())
        }),
    );
    if !matches!(result,Err(eframe::Error::AppCreation(ref e)) if e.to_string()=="controlled post-handoff initialization failure")
    {
        return Err(format!(
            "initialization failure did not reach requested stage: {result:?}"
        ));
    }
    let endpoint_evidence = if plan == fault::Plan::EndpointHeld {
        let mut local = slot.borrow_mut();
        let mut loan = local.take().ok_or("missing endpoint cleanup owner")?;
        let value = loan.get_mut();
        let retained_pcm = audio.borrow().budget.usage().bytes;
        let bytes = match &value.owner {
            Some(Owner::Editor(e)) => e.images.blobs().observe_bytes().usage().bytes,
            _ => 0,
        };
        for _ in 0..3 {
            if value.retry_cleanup().is_ok() || value.retry_ready() || value.open_backend().is_ok()
            {
                return Err("endpoint not returned but native retry was admitted".into());
            }
            let Some(Owner::Editor(editor)) = &value.owner else {
                return Err("endpoint wait lost project owner".into());
            };
            if Stamp::read(&editor.session)? != before
                || !editor.ui.paused
                || !editor.preview.paused
                || editor.images.blobs().observe_bytes().usage().bytes != bytes
            {
                return Err("endpoint wait changed original project/history".into());
            }
            let mut a = audio.borrow_mut();
            if a.request(AudioEditorAction::Enable).is_ok()
                || !a.stop_pending
                || a.close_failed
                || a.client.is_none()
                || a.native.is_none()
                || retained_pcm == 0
                || a.budget.usage().bytes != retained_pcm
                || bytes == 0
                || a.native.as_ref().unwrap().stats().endpoint_returns != 0
            {
                return Err("endpoint wait released owners/budget or reopened audio".into());
            }
        }
        endpoint_gate
            .borrow_mut()
            .as_mut()
            .ok_or("no retained native endpoint")?
            .deliver()?;
        value.settle(false);
        if !value.runtime_retired()
            || value.retry_ready()
            || value.open_backend().is_ok()
            || value.snapshot() != before.snapshot
            || audio.borrow().budget.usage().bytes != 0
        {
            return Err("late endpoint completion did not retire the original runtime".into());
        }
        format!(
            "{{\"pending_detected\":true,\"supply_retained\":true,\"receiver_retained\":true,\"new_audio_rejected\":true,\"new_backend_rejected\":true,\"repeated_attempts\":3,\"project_exact_while_pending\":true,\"history_exact_while_pending\":true,\"pcm_retained_bytes\":{retained_pcm},\"raw_retained_bytes\":{bytes},\"returns_before_delivery\":0,\"actual_late_delivery\":true,\"pcm_after_delivery\":0,\"framework_gate_after_runtime\":true}}"
        )
    } else {
        "null".into()
    };
    let cleanup_evidence = if plan == fault::Plan::CleanupHeld {
        let mut local = slot.borrow_mut();
        let mut loan = local.take().ok_or("no quarantined owner")?;
        let value = loan.get_mut();
        let Owner::Editor(editor) = value.owner.as_ref().ok_or("lost cleanup owner")? else {
            return Err("cleanup returned a seed before old draw exited".into());
        };
        let retained = init_failure::Stamp::read(&editor.session)?;
        let targets = editor.host.live_targets();
        let raw = editor.images.blobs().observe_bytes();
        let rgba = editor.images.observe_rgba();
        let bytes = raw.usage().bytes;
        let failed_gate =
            value.error.is_some() && !value.retry_ready() && value.open_backend().is_err();
        // Repeat while the exact external owner remains live, without timing
        // guesses or allowing a new Device to be created.
        let refused = value.retry_cleanup().is_err() && value.retry_cleanup().is_err();
        let Owner::Editor(editor) = value.owner.as_ref().ok_or("cleanup lost editor")? else {
            return Err("cleanup advanced despite held draw".into());
        };
        let unchanged = retained == before && Stamp::read(&editor.session)? == before;
        let held_live = editor.host.live_targets() == targets && targets > 0;
        let raw_retained = bytes > 0 && raw.usage().bytes == bytes;
        let rgba_retained = rgba.usage().bytes;
        let paused = editor.ui.paused && editor.preview.paused;
        let renderer = backend.renderer_gone.load(Ordering::Acquire);
        let resources = backend
            .instance
            .generate_report()
            .ok_or("no quarantined registry")?;
        let runtime_live = !resources.hub.devices.is_empty() && !resources.hub.textures.is_empty();
        let audio_closed = {
            let a = audio.borrow();
            !a.close_failed
                && a.native.is_none()
                && a.client.is_none()
                && a.budget.usage().bytes == 0
        };
        if !failed_gate
            || !refused
            || !unchanged
            || !held_live
            || !raw_retained
            || !paused
            || !renderer
            || !runtime_live
            || !audio_closed
        {
            return Err("native cleanup quarantine did not retain owners and block retry".into());
        }
        held_outside.borrow_mut().take();
        value.settle(false);
        if !value.runtime_retired()
            || value.retry_ready()
            || value.open_backend().is_ok()
            || value.error.is_some()
            || value.snapshot() != before.snapshot
        {
            return Err("cleanup completion did not unlock the same session".into());
        }
        format!(
            "{{\"retry_blocked\":true,\"repeated_refusal\":true,\"project_exact_while_blocked\":true,\"history_exact_while_blocked\":true,\"targets_retained\":{targets},\"raw_bytes_retained\":{bytes},\"rgba_bytes_while_blocked\":{rgba_retained},\"renderer_released_before_cleanup\":{renderer},\"runtime_live_before_release\":{runtime_live},\"paused\":{paused},\"audio_closed\":{audio_closed},\"framework_gate_after_runtime\":true,\"retry_ready_after_release\":true}}"
        )
    } else {
        "null".into()
    };
    let report = backend
        .instance
        .generate_report()
        .ok_or("no failed-init registry")?;
    let renderer = backend.renderer_gone.load(Ordering::Acquire);
    let empty = report.surfaces.is_empty()
        && report.hub.devices.is_empty()
        && report.hub.queues.is_empty()
        && report.hub.textures.is_empty()
        && report.hub.buffers.is_empty();
    let mut slot = slot.borrow_mut();
    let value = slot.get().ok_or("failed init lost project owner")?;
    if let Some(e) = &value.error {
        return Err(format!("initialization cleanup blocked retry: {e}"));
    }
    let Some(Owner::Seed(seed)) = &value.owner else {
        return Err("failed initialization still owns runtime".into());
    };
    let after = Stamp::read(&seed.session)?;
    if value.snapshot() != before.snapshot {
        return Err("active session identity changed".into());
    }
    let a = audio.borrow();
    let audio_ok = !a.close_failed
        && a.native.is_none()
        && a.client.is_none()
        && a.budget.usage().bytes == 0
        && a.recovery.and_then(|r| r.copied_frame) == Some(anchor);
    let count = reached.load(Ordering::Acquire);
    let audio_stats = if plan.active_edit()
        || matches!(
            plan,
            fault::Plan::InitAudio | fault::Plan::CleanupHeld | fault::Plan::EndpointHeld
        ) {
        let stats = a
            .closed_stats
            .ok_or("failed initialization omitted native close receipt")?;
        if stats.callbacks == 0
            || stats.missing_frames != 0
            || stats.render_errors != 0
            || stats.observation_overflows != 0
            || stats.endpoint_returns != 1
            || stats.endpoint_returns_outside_render != 1
            || stats.endpoint_returns_in_render != 0
            || stats.callback_resource_releases != 0
        {
            return Err("failed initialization audio health/ownership failed".into());
        }
        format!(
            "{{\"callbacks\":{},\"missing_frames\":{},\"render_errors\":{},\"observation_overflows\":{},\"endpoint_returns\":{},\"endpoint_returns_outside_render\":{},\"endpoint_returns_in_render\":{},\"callback_resource_releases\":{}}}",
            stats.callbacks,
            stats.missing_frames,
            stats.render_errors,
            stats.observation_overflows,
            stats.endpoint_returns,
            stats.endpoint_returns_outside_render,
            stats.endpoint_returns_in_render,
            stats.callback_resource_releases
        )
    } else {
        "null".into()
    };
    let expected = match plan {
        fault::Plan::InitConsumed => 1,
        fault::Plan::InitPrepared => 2,
        fault::Plan::InitAudio => 3,
        fault::Plan::CleanupHeld => 3,
        fault::Plan::EndpointHeld => 3,
        p if p.active_edit() => 3,
        _ => 0,
    };
    if !renderer
        || !empty
        || if plan.active_edit() {
            !before.same_committed(&after) || after.requests() <= before.requests()
        } else {
            before != after
        }
        || !audio_ok
        || count != expected
        || (expected >= 2 && (!value.prepared || !value.held_blocked))
        || !value.observer_closed
    {
        return Err(format!(
            "post-consumption init retention failed: stage={count} renderer={renderer} empty={empty} audio={audio_ok}"
        ));
    }
    let last_key = failed_key.load(Ordering::Acquire);
    let record = format!(
        "{{\"schema\":1,\"stage\":{count},\"consumed\":true,\"editor_constructed\":true,\"project_exact\":true,\"history_exact\":true,\"nodes\":{},\"branches\":{},\"remembered_paths\":{},\"prepared\":{},\"held_draw_blocked\":{},\"observer_closed\":{},\"audio_closed\":{audio_ok},\"anchor\":{anchor},\"renderer_released\":{renderer},\"registries_empty\":{empty},\"instance_retired\":true,\"last_frame_key\":{last_key},\"audio_stats\":{audio_stats}}}",
        before.statistics.nodes,
        before.statistics.branches,
        before.statistics.redo_preference_entries,
        value.prepared,
        value.held_blocked,
        value.observer_closed
    );
    let record = format!(
        "{},\"cleanup\":{cleanup_evidence},\"endpoint_cleanup\":{endpoint_evidence},\"activity\":{},\"draft_request_growth\":{}}}",
        &record[..record.len() - 1],
        activity_record.borrow().as_deref().unwrap_or("null"),
        after.requests().saturating_sub(before.requests())
    );
    drop(a);
    drop(backend.instance);
    let mut ticket = slot.take().ok_or("missing retry ticket")?;
    ticket.get_mut().framework_retired()?;
    if !ticket.get_mut().retry_ready() {
        return Err("framework retirement did not permit retry".into());
    }
    let owned = ticket.finish();
    let Some(Owner::Seed(seed)) = owned.owner else {
        return Err("no GPU-free retry ticket".into());
    };
    let late_audio = failed_audio.borrow_mut().take();
    Ok(Recovered {
        seed: *seed,
        record,
        frame: (last_key > 0).then_some(FrameKey(last_key)),
        loss: backend.loss,
        audio: late_audio,
    })
}
