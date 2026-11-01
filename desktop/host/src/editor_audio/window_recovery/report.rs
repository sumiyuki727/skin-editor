use super::*;
pub(super) fn window(r: &Run, a: &EditorAudio, passes: u64) -> String {
    let stats = a.closed_stats.unwrap_or_default();
    let base = format!(
        "{{\"index\":{},\"phase\":{},\"full_ui_passes\":{},\"issued\":{},\"rows\":{},\"screens\":{:?},\"expected_screens\":{:?},\"software_scale\":{},\"native_before\":{},\"native_after\":{},\"physical_dpi\":{},\"pause_anchor\":{},\"restore_anchor\":{},\"old_voices\":{},\"pending_prepare\":{},\"pixel_error\":{},\"notice\":{},\"pcm_after_fault\":{},\"callbacks\":{},\"missing_frames\":{},\"render_errors\":{},\"observation_overflows\":{},\"endpoint_returns\":{},\"callback_resource_releases\":{},\"endpoint_returns_in_render\":{},\"endpoint_returns_outside_render\":{}}}",
        r.index,
        r.phase,
        passes,
        r.issued,
        sound::rows(&r.probes),
        r.screens,
        r.expected_screens,
        r.software_scale,
        r.native_before,
        r.native_after,
        r.physical_dpi,
        r.pause,
        r.restore_anchor.map_or("null".into(), |n| n.to_string()),
        r.old_voice,
        r.pending,
        r.max_pixel_error,
        r.notice.load(Ordering::Acquire),
        a.budget.usage().bytes,
        stats.callbacks,
        stats.missing_frames,
        stats.render_errors,
        stats.observation_overflows,
        stats.endpoint_returns,
        stats.callback_resource_releases,
        stats.endpoint_returns_in_render,
        stats.endpoint_returns_outside_render
    );
    format!(
        "{},\"fault_plan\":{},\"fault\":{},\"late_frame_rejected\":{},\"previous_frame_rejected\":{}}}",
        &base[..base.len() - 1],
        r.plan.code(),
        r.fault.as_ref().map_or("null".into(), |s| s.json()),
        r.late_frame_rejected,
        r.previous_frame_rejected
    )
}
pub(super) fn run(destination: &str, plan: fault::Plan) -> Result<(), String> {
    let path = PathBuf::from(destination);
    if !path.is_absolute() {
        return Err("new absolute window proof directory required".into());
    }
    std::fs::create_dir(&path).map_err(error)?;
    let mut seed: Option<Seed> = None;
    let mut shared_audio: Option<Rc<RefCell<EditorAudio>>> = None;
    let mut baseline: Option<ProjectSnapshot> = None;
    let mut first_pixels: Option<Vec<u8>> = None;
    let mut records = Vec::new();
    let mut adapters = Vec::new();
    let mut retirements = Vec::new();
    let mut old_loss: Option<crate::presentation::DeviceLoss> = None;
    let mut old_audio: Option<skin_editor_infrastructure::separated_native::NativeFaultSignal> =
        None;
    let mut raw = None;
    let mut rgba = None;
    let mut budget = None;
    let mut failed_creation = false;
    let mut old_frame = None;
    for index in 0..2 {
        if index == 1 && plan == fault::Plan::RebuildFailure {
            reject_creation(seed.as_ref().ok_or("missing retained seed")?)?;
            failed_creation = true;
        }
        if index == 1 && plan.code() >= 4 {
            let original = seed
                .take()
                .ok_or("missing consumed initialization handoff")?;
            let recovered = init_failure::run(
                original,
                shared_audio
                    .as_ref()
                    .ok_or("missing retained audio")?
                    .clone(),
                &path,
                plan,
            )?;
            seed = Some(recovered.seed);
            if let Some(key) = recovered.frame {
                old_frame = Some(key);
            }
            old_loss = Some(recovered.loss);
            if let Some(signal) = recovered.audio {
                old_audio = Some(signal);
            }
            std::fs::write(path.join("initialization.json"), recovered.record).map_err(error)?;
        }
        crate::load_verification::phase(if index == 0 {
            "window_initializing"
        } else {
            "window_rebuilding"
        });
        let local_plan = if index == 0 {
            plan
        } else {
            fault::Plan::Planned
        };
        let fault =
            (local_plan != fault::Plan::Planned).then(|| Arc::new(fault::Signals::default()));
        let backend = backend::Backend::with_fault(fault.clone())?;
        let backend::Backend {
            instance,
            loss,
            notice,
            renderer_gone,
            options,
            adapter_name,
        } = backend;
        adapters.push(adapter_name);
        let out = Rc::new(RefCell::new(None));
        let result = out.clone();
        let path_app = path.clone();
        let initialization = Rc::new(RefCell::new(match seed.take() {
            Some(seed) => restore::Slot::new(restore::State::seed(seed)),
            None => restore::Slot::vacant(),
        }));
        let initial_slot = initialization.clone();
        let initial_audio = shared_audio.clone();
        let fence = loss.clone();
        let signal = notice.clone();
        let witness = renderer_gone.clone();
        let previous = old_loss.clone();
        let previous_audio = old_audio.clone();
        eframe::run_native(
            &format!("Skin Editor - C4 recovery {index}"),
            options,
            Box::new(move |context| {
                let state = context
                    .wgpu_render_state
                    .as_ref()
                    .ok_or_else(|| std::io::Error::other("no native renderer"))?;
                state
                    .renderer
                    .write()
                    .callback_resources
                    .insert(backend::RendererWitness(witness));
                let mut slot = initial_slot.borrow_mut();
                let mut initializing = if slot.get().is_some() {
                    restore::Attempt::begin(&mut slot, &context.egui_ctx, state, signal.clone())
                        .map_err(std::io::Error::other)?
                } else {
                    let editor = Editor::with_render_state(
                        &context.egui_ctx,
                        state,
                        false,
                        skin_editor_rendering::SourceUpload::QueueWrite,
                    )
                    .map_err(std::io::Error::other)?;
                    restore::Attempt::fresh(&mut slot, editor, signal.clone())
                };
                let editor = initializing.editor();
                editor
                    .host
                    .bind_loss(fence)
                    .map_err(|e| std::io::Error::other(error(e)))?;
                if let Some(previous) = previous {
                    previous.report();
                    editor
                        .host
                        .poll()
                        .map_err(|e| std::io::Error::other(error(e)))?;
                }
                let (audio, anchor) = match initial_audio {
                    Some(a) => {
                        let anchor =
                            sound::reopen(editor, &a, &path_app).map_err(std::io::Error::other)?;
                        (a, Some(anchor))
                    }
                    None => (
                        sound::install(editor, &path_app, local_plan.code() >= 4)
                            .map_err(std::io::Error::other)?,
                        None,
                    ),
                };
                initializing.attach_audio(audio.clone());
                let editor = initializing.editor();
                if let Some(signal) = previous_audio {
                    signal.report();
                    if audio
                        .borrow()
                        .native
                        .as_ref()
                        .is_none_or(|n| n.stats().device_fault)
                    {
                        return Err(
                            std::io::Error::other("old audio signal faulted new endpoint").into(),
                        );
                    }
                }
                let audio_signal = audio
                    .borrow()
                    .native
                    .as_ref()
                    .ok_or_else(|| std::io::Error::other("native missing"))?
                    .fault_signal();
                if let Some(fault) = &fault {
                    fault
                        .bind(audio_signal.clone(), context.egui_ctx.clone())
                        .map_err(std::io::Error::other)?;
                }
                let reader = audio
                    .borrow_mut()
                    .native
                    .as_mut()
                    .ok_or_else(|| std::io::Error::other("native not enabled"))?
                    .take_first_observer()
                    .ok_or_else(|| std::io::Error::other("observer already taken"))?;
                let baseline = editor.session.snapshot();
                initializing.observer =
                    Some(Observations::with_capacity(reader, 192).map_err(std::io::Error::other)?);
                let (tx, rx) = mpsc::sync_channel(1);
                let baseline = if local_plan.code() >= 4 {
                    initializing.editor().session.snapshot()
                } else {
                    baseline
                };
                let observer = initializing
                    .observer
                    .take()
                    .ok_or_else(|| std::io::Error::other("initialization observer missing"))?;
                Ok(Box::new(Run {
                    plan: local_plan,
                    fault,
                    late_frame_rejected: false,
                    held_key: None,
                    previous_key: old_frame,
                    previous_frame_rejected: false,
                    editor: Some(initializing.commit()),
                    audio,
                    observer: Some(observer),
                    probes: Probes::default(),
                    path: path_app,
                    index,
                    phase: 0,
                    started: Instant::now(),
                    playing: None,
                    issued: 0,
                    pause: 0,
                    restore_anchor: anchor,
                    screens: Vec::with_capacity(4),
                    expected_screens: Vec::with_capacity(4),
                    shot_tx: tx,
                    shot_rx: rx,
                    software_scale: false,
                    native_before: 0.,
                    native_after: 0.,
                    physical_dpi: false,
                    held: None,
                    canonical: Vec::new(),
                    max_pixel_error: 255,
                    notice: signal,
                    failure: None,
                    out: result,
                    old_voice: 0,
                    pending: false,
                    baseline,
                    audio_signal,
                }))
            }),
        )
        .map_err(error)?;
        let mut returned: Returned = out
            .borrow_mut()
            .take()
            .ok_or("window ended without handoff")?;
        if let Some(e) = returned.failure {
            std::fs::write(path.join("failure.txt"), &e).map_err(error)?;
            std::fs::write(path.join("partial-window.json"), returned.record).map_err(error)?;
            return Err(e);
        }
        std::fs::write(path.join(format!("window-{index}.json")), &returned.record)
            .map_err(error)?;
        records.push(returned.record);
        old_frame = returned.frame_key;
        if let Some(snapshot) = &baseline {
            if returned.editor.session.snapshot() != *snapshot {
                return Err("project changed during window replacement".into());
            }
        } else {
            baseline = Some(returned.editor.session.snapshot());
            raw = Some(returned.editor.images.blobs().observe_bytes());
            rgba = Some(returned.editor.images.observe_rgba());
        }
        if let Some(first) = &first_pixels {
            if returned.canonical != *first {
                return Err("preview pixels changed across eframe contexts".into());
            }
        } else {
            first_pixels = Some(returned.canonical.clone());
        }
        std::fs::write(
            path.join(format!("preview-{index}.rgba")),
            &returned.canonical,
        )
        .map_err(error)?;
        let surfaces = instance
            .generate_report()
            .ok_or("registry report unavailable")?
            .surfaces
            .is_empty();
        let renderer = renderer_gone.load(Ordering::Acquire);
        // The held old CPU callback must block project-renderer retirement,
        // even after eframe's separate Renderer/Surface owners have exited.
        let Editor {
            session,
            ui,
            images,
            host,
            preview,
            importer,
            ..
        } = returned.editor;
        let host = match host.into_rebuild_seed() {
            Ok(_) => return Err("held window draw did not block retirement".into()),
            Err(owner) => *owner,
        };
        drop(returned.held.take());
        let worker = host.preparation_closed();
        let targets = host.live_targets();
        let imported = importer.closed();
        let gpu = host
            .into_rebuild_seed()
            .map_err(|_| "framework output/worker still pins the old target")?;
        drop(importer);
        let report = instance
            .generate_report()
            .ok_or("registry report unavailable after retirement")?;
        let devices = report.hub.devices.is_empty()
            && report.hub.queues.is_empty()
            && report.hub.textures.is_empty()
            && report.hub.buffers.is_empty();
        let hub = &report.hub;
        let others_empty = [
            hub.adapters,
            hub.devices,
            hub.queues,
            hub.buffers,
            hub.pipeline_layouts,
            hub.shader_modules,
            hub.bind_group_layouts,
            hub.bind_groups,
            hub.command_encoders,
            hub.command_buffers,
            hub.render_bundles,
            hub.render_pipelines,
            hub.compute_pipelines,
            hub.pipeline_caches,
            hub.query_sets,
            hub.texture_views,
            hub.external_textures,
            hub.samplers,
            hub.render_passes,
            hub.compute_passes,
            hub.render_bundle_encoders,
        ]
        .iter()
        .all(|r| r.is_empty());
        let texture_ids = hub.textures.num_allocated;
        let texture_kept = hub.textures.num_kept_from_user;
        let known_surface_ids = others_empty
            && backend::surface_ids_only(texture_ids, texture_kept, returned.surface_failures);
        if !backend::retired(
            renderer,
            surfaces,
            devices || known_surface_ids,
            targets,
            worker,
            imported,
        ) {
            return Err(format!(
                "window owner still alive: renderer={renderer} surfaces={surfaces} devices={devices} targets={targets} worker={worker} importer={imported}; {report:?}"
            ));
        }
        old_loss = Some(loss);
        drop(instance);
        retirements.push(format!("{{\"renderer_released\":{renderer},\"surfaces_empty\":{surfaces},\"device_resources_empty\":{devices},\"targets\":{targets},\"worker_closed\":{worker},\"importer_closed\":{imported},\"held_draw_blocked\":true,\"texture_ids\":{texture_ids},\"texture_kept\":{texture_kept},\"surface_failures\":{},\"other_registries_empty\":{others_empty},\"instance_retired\":true}}",returned.surface_failures));
        old_audio = Some(returned.audio_signal);
        shared_audio = Some(returned.audio);
        budget = shared_audio.as_ref().map(|a| a.borrow().budget.clone());
        seed = Some(Seed {
            session,
            ui,
            images,
            preview,
            gpu,
        });
        crate::load_verification::phase("window_retired");
    }
    let history_navigation = if plan.code() >= 4 {
        init_failure::verify_redo(seed.as_mut().ok_or("no final history owner")?)?;
        true
    } else {
        false
    };
    drop(seed);
    drop(baseline);
    drop(shared_audio);
    let raw = raw.ok_or("no raw observation")?.usage();
    let rgba = rgba.ok_or("no RGBA observation")?.usage();
    let pcm = budget.ok_or("no PCM observation")?.usage();
    if raw.bytes != 0 || rgba.bytes != 0 || pcm.bytes != 0 {
        return Err("window final resource leases survived".into());
    }
    std::fs::write(path.join("report.json"),format!("{{\"schema\":1,\"scope\":\"same-process-eframe-window-replacement\",\"full_editor\":true,\"memory_usage\":true,\"adapters_utf8\":{adapters:?},\"in_place_device_reset\":false,\"fault_plan\":{},\"failed_creation_retired\":{failed_creation},\"history_navigation_verified\":{history_navigation},\"windows\":[{}],\"retirements\":[{}],\"pixels_exact\":true,\"project_unchanged\":true,\"raw_final\":{},\"rgba_final\":{},\"pcm_final\":{},\"raw_peak\":{},\"rgba_peak\":{},\"pcm_peak\":{}}}",plan.code(),records.join(","),retirements.join(","),raw.bytes,rgba.bytes,pcm.bytes,raw.peak_bytes,rgba.peak_bytes,pcm.peak_bytes)).map_err(error)?;
    crate::load_verification::phase("closed");
    Ok(())
}

fn reject_creation(seed: &Seed) -> Result<(), String> {
    crate::load_verification::phase("window_rebuild_failure");
    let snapshot = seed.session.snapshot();
    let backend = backend::Backend::new()?;
    let witness = backend.renderer_gone.clone();
    let result = eframe::run_native(
        "Skin Editor - controlled creation failure",
        backend.options,
        Box::new(move |context| {
            context
                .wgpu_render_state
                .as_ref()
                .ok_or_else(|| std::io::Error::other("missing failed-creation renderer"))?
                .renderer
                .write()
                .callback_resources
                .insert(backend::RendererWitness(witness));
            Err(std::io::Error::other("controlled app creation failure").into())
        }),
    );
    if !matches!(result,Err(eframe::Error::AppCreation(ref e)) if e.to_string()=="controlled app creation failure")
    {
        return Err("expected creation failure was not returned".into());
    }
    let report = backend
        .instance
        .generate_report()
        .ok_or("failed creation registry unavailable")?;
    if !backend.renderer_gone.load(Ordering::Acquire)
        || !report.surfaces.is_empty()
        || !report.hub.devices.is_empty()
        || !report.hub.queues.is_empty()
        || !report.hub.textures.is_empty()
        || !report.hub.buffers.is_empty()
        || seed.session.snapshot() != snapshot
    {
        return Err(format!(
            "failed creation retained renderer/surface/device or changed project: {report:?}"
        ));
    }
    Ok(())
}
