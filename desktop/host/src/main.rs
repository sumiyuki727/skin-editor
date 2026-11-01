#![forbid(unsafe_code)]
mod audio_verification;
mod combined_verification;
mod edit_verification;
mod editor_audio;
mod frame_worker;
#[cfg(feature = "framework-trace")]
mod framework_trace;
mod gpu_observation;
mod gpu_path;
mod graphics_faults;
mod gui_baseline;
mod import_worker;
mod load_verification;
mod main_work;
mod mesh_replay;
mod presentation;
mod product_policy;
mod rebuild_stages;
mod repaint_deadline;
mod repaint_observation;
mod separated_verification;
mod stable_window;
mod stream_fixture;
mod supply_verification;
mod text_work;
mod verification;
mod worker_verification;
use presentation::NativeHost;
use skin_editor_application::{values::*, *};
use skin_editor_infrastructure::Images;
use skin_editor_preview::{PreviewState, Scene};
use skin_editor_ui::EditorUi;
use std::time::Instant;

struct Editor {
    session: ProjectSession,
    ui: EditorUi,
    images: Images,
    host: NativeHost,
    preview: PreviewState,
    last: Instant,
    importer: import_worker::ImportWorker,
    smoke_started: Option<Instant>,
    ui_passes: u64,
    // Only installed by the explicit native diagnostic. Never enters project data.
    main_work: Option<std::cell::RefCell<main_work::MainWork>>,
    repaint_pass: repaint_deadline::RepaintPass,
}
fn fresh_session(images: &Images) -> Result<ProjectSession, String> {
    let mut bytes = [0; 16];
    getrandom::fill(&mut bytes).map_err(|e| e.to_string())?;
    ProjectSession::with_resources(
        1,
        u128::from_le_bytes(bytes).max(1),
        Box::new(images.clone()),
        SessionLimits::default(),
    )
    .map_err(|e| format!("{e:?}"))
}
fn import(
    ui: &mut EditorUi,
    session: &mut ProjectSession,
    staged: StagedRaster,
) -> Option<AssetId> {
    let id = AssetId::new(ui.object_id()).unwrap();
    let asset = Asset::new(
        id,
        staged.label,
        staged.blob,
        AssetKind::RasterImage(staged.size),
    )
    .ok()?;
    let result = ui.submit(session, vec![Command::RegisterAsset { asset }]);
    drop(staged.lease);
    result.then_some(id)
}
fn sample(ui: &mut EditorUi, session: &mut ProjectSession, images: &mut Images) {
    match images.stage_bytes(Images::sample_png(), "自制示例 · 圆环".into()) {
        Ok(staged) => {
            if let Some(asset) = import(ui, session, staged) {
                ui.add_asset_layer(session, asset);
            }
        }
        Err(e) => ui.message = format!("{e:?}"),
    }
}
impl Editor {
    fn new(context: &eframe::CreationContext<'_>, smoke: bool) -> Result<Self, String> {
        Self::with_upload(
            context,
            smoke,
            skin_editor_rendering::SourceUpload::QueueWrite,
        )
    }
    fn with_upload(
        context: &eframe::CreationContext<'_>,
        smoke: bool,
        upload: skin_editor_rendering::SourceUpload,
    ) -> Result<Self, String> {
        let state = context
            .wgpu_render_state
            .as_ref()
            .ok_or("wgpu renderer unavailable")?;
        Self::with_render_state(&context.egui_ctx, state, smoke, upload)
    }
    fn with_render_state(
        ctx: &eframe::egui::Context,
        state: &eframe::egui_wgpu::RenderState,
        smoke: bool,
        upload: skin_editor_rendering::SourceUpload,
    ) -> Result<Self, String> {
        Self::with_initial_content(ctx, state, smoke, upload, true)
    }
    fn with_initial_content(
        ctx: &eframe::egui::Context,
        state: &eframe::egui_wgpu::RenderState,
        smoke: bool,
        upload: skin_editor_rendering::SourceUpload,
        include_sample: bool,
    ) -> Result<Self, String> {
        let mut images = Images::default();
        let mut session = fresh_session(&images)?;
        let mut ui = EditorUi::default();
        ui.set_audio_port(Box::new(
            product_policy::Policy::CURRENT.audio(images.blobs().clone()),
        ));
        if include_sample {
            sample(&mut ui, &mut session, &mut images);
        }
        let host = NativeHost::with_upload(
            state.device.clone(),
            state.queue.clone(),
            state.target_format,
            images.clone(),
            ctx.clone(),
            upload,
        )
        .map_err(|e| format!("{e:?}"))?;
        let importer = import_worker::ImportWorker::new(images.clone(), ctx.clone())?;
        Ok(Self {
            session,
            ui,
            images,
            host,
            preview: PreviewState::default(),
            last: Instant::now(),
            importer,
            smoke_started: smoke.then(Instant::now),
            ui_passes: 0,
            main_work: None,
            repaint_pass: repaint_deadline::RepaintPass::default(),
        })
    }
    fn work_edge(&self, category: u8) {
        if let Some(work) = &self.main_work {
            work.borrow_mut().edge(category);
        }
    }
}
impl eframe::App for Editor {
    fn logic(&mut self, ctx: &eframe::egui::Context, _: &mut eframe::Frame) {
        self.repaint_pass.begin(ctx);
        let mut repaint = repaint_deadline::RepaintCycle::new();
        if let Some(started) = self.smoke_started {
            if started.elapsed() >= std::time::Duration::from_secs(3) {
                ctx.send_viewport_cmd(eframe::egui::ViewportCommand::Close);
            } else {
                repaint.after(100);
            }
        }
        let now = Instant::now();
        let delta = now
            .duration_since(self.last)
            .as_micros()
            .min(u128::from(u64::MAX)) as u64;
        self.last = now;
        self.ui.refresh_audio(&self.session);
        let audio_fault = !self.ui.audio_state.enabled
            && self.ui.audio_state.frozen
            && self.ui.audio_state.error.is_some();
        if audio_fault {
            self.interrupt_for_fault();
        }
        self.work_edge(main_work::GRAPHICS_POLL);
        if let Err(e) = self.host.poll() {
            self.ui.message = format!("GPU: {e:?}；请重启编辑器");
        }
        if self.host.is_lost() && !audio_fault {
            self.interrupt_for_fault();
        }
        self.preview.advance(delta);
        self.ui.simulation_micros = self.preview.micros();
        self.ui.accepted_inputs = self.preview.accepted_inputs();
        if self.ui.audio_needs_poll() {
            repaint.after(20);
        }
        if !self.preview.paused {
            repaint.after(16);
        }
        if self.host.needs_work() {
            repaint.after(10);
        }
        repaint.publish(ctx, &mut self.repaint_pass);
        self.work_edge(main_work::LOGIC);
    }
    fn ui(&mut self, root: &mut eframe::egui::Ui, _: &mut eframe::Frame) {
        self.ui_passes += 1;
        let ctx = root.ctx().clone();
        // Keep staged leases alive while a draft owns the project write lane.
        if self
            .session
            .draft_info(self.session.project_info().session_id)
            .ok()
            .flatten()
            .is_none()
        {
            for result in self.importer.drain() {
                self.ui.pending_imports = self.ui.pending_imports.saturating_sub(1);
                match result {
                    Ok(staged) => {
                        let sound = !matches!(staged.kind, AssetKind::RasterImage(_));
                        let id = AssetId::new(self.ui.object_id()).unwrap();
                        match Asset::new(id, staged.label, staged.blob, staged.kind) {
                            Ok(asset) => {
                                if self.ui.submit(
                                    &mut self.session,
                                    vec![Command::RegisterAsset { asset }],
                                ) && sound
                                {
                                    self.ui.audio_asset = Some(id);
                                }
                            }
                            Err(e) => self.ui.message = format!("素材登记: {e:?}"),
                        }
                        drop(staged.lease);
                    }
                    Err(e) => self.ui.message = format!("素材导入: {e}"),
                }
            }
        }
        self.work_edge(main_work::WIDGETS);
        let output = if let Some(work) = &self.main_work {
            let mut observer = |phase| work.borrow_mut().edge(main_work::ui_category(phase));
            self.ui
                .show_observed(root, &mut self.session, &mut self.host, Some(&mut observer))
        } else {
            self.ui.show(root, &mut self.session, &mut self.host)
        };
        self.work_edge(main_work::UI_DISPATCH);
        self.complete_ui(&ctx, output);
    }
    fn on_exit(&mut self) {
        self.ui.cancel(&mut self.session);
        self.ui
            .audio_command(&self.session, AudioEditorAction::Disable);
        self.host.shutdown();
        if self.smoke_started.is_some() {
            println!(
                "native-window-smoke ui_passes={} publications={}",
                self.ui_passes, self.host.publications
            );
        }
    }
}
impl Editor {
    fn interrupt_for_fault(&mut self) -> bool {
        self.preview.paused = true;
        let cancelled = self.ui.interrupt_for_fault(&mut self.session);
        // On an audio-only fault the GPU is still usable. Select the cancelled
        // draft's replacement before another poll can accept its late result.
        if !self.host.is_lost()
            && let Some(view) = skin_editor_ui::Presenter::current(&self.host)
        {
            let mut context = view.frame.source.context;
            context.draft = self
                .session
                .draft_info(self.session.project_info().session_id)
                .ok()
                .flatten()
                .map(|d| d.state);
            context.simulation_micros = self.preview.micros();
            if let Err(error) = self.session.request_preview(&mut self.host, context) {
                self.host.invalidate_source();
                self.ui.message = format!("故障后预览: {error:?}");
            }
        }
        cancelled
    }
    // Shared post-UI path. Diagnostic controls replace drawing only, keeping
    // audio, preview source updates and repaint policy on this existing path.
    fn complete_ui(&mut self, ctx: &eframe::egui::Context, output: skin_editor_ui::UiOutput) {
        for path in output.imports {
            if let Err(e) = self.importer.enqueue(path) {
                self.ui.message = e;
            } else {
                self.ui.pending_imports += 1;
            }
        }
        for (path, original) in output.audio_imports {
            let request = if original {
                import_worker::ImportRequest::Original(path)
            } else {
                import_worker::ImportRequest::Auto(path)
            };
            if let Err(e) = self.importer.enqueue_request(request) {
                self.ui.message = e;
            } else {
                self.ui.pending_imports += 1;
            }
        }
        if output.sample_sound {
            match self
                .importer
                .enqueue_request(import_worker::ImportRequest::SampleSound)
            {
                Ok(()) => self.ui.pending_imports += 1,
                Err(e) => self.ui.message = e,
            }
        }
        if output.sample {
            sample(&mut self.ui, &mut self.session, &mut self.images);
        }
        self.preview.paused = self.ui.scene_is_frozen();
        self.preview.scene = if self.ui.scene == 0 {
            Scene::Canvas
        } else {
            Scene::Playground
        };
        self.preview.auto_play = self.ui.auto_play;
        for _ in 0..output.game_inputs {
            self.preview.input(self.ui.interaction, true);
        }
        if self.preview.fullscreen != self.ui.fullscreen {
            self.preview.fullscreen = self.ui.fullscreen;
            ctx.send_viewport_cmd(eframe::egui::ViewportCommand::Fullscreen(
                self.ui.fullscreen,
            ));
        }
        if let (Some(target), Some((size, generation))) = (self.ui.target, output.viewport) {
            let draft = self
                .session
                .draft_info(self.session.project_info().session_id)
                .ok()
                .flatten()
                .map(|d| d.state);
            self.work_edge(main_work::PREVIEW_REQUEST);
            if let Err(e) = self.session.request_preview(
                &mut self.host,
                PreviewContext {
                    target,
                    draft,
                    size,
                    viewport_generation: generation,
                    scene: self.ui.scene,
                    simulation_micros: self.preview.micros(),
                },
            ) {
                self.ui.message = format!("预览: {e:?}");
            }
            self.work_edge(main_work::UI_DISPATCH);
        }
        // Commands may create new work after logic has already published its
        // deadline. Use a fresh cycle; never suppress these post-UI demands.
        let mut repaint = repaint_deadline::RepaintCycle::new();
        if self.ui.audio_needs_poll() {
            repaint.after(20);
        }
        if self.host.needs_work() {
            repaint.after(10);
        }
        if !self.preview.paused {
            repaint.after(16);
        }
        repaint.publish(ctx, &mut self.repaint_pass);
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    if args.get(1).is_some_and(|a| a == "--verify-joint-cadence") {
        verification::run_joint_cadence(args.get(2).ok_or("output directory required")?)?;
        return Ok(());
    }
    if args
        .get(1)
        .is_some_and(|a| a == "--verify-transform-cadence")
    {
        verification::run_transform_cadence(args.get(2).ok_or("output directory required")?)?;
        return Ok(());
    }
    if args.get(1).is_some_and(|a| a == "--verify-present-fault") {
        editor_audio::window_recovery::run_fault(
            args.get(2).ok_or("output directory required")?,
            args.get(3).ok_or("fault plan required")?,
        )?;
        return Ok(());
    }
    if args.get(1).is_some_and(|a| a == "--verify-window-recovery") {
        editor_audio::window_recovery::run(args.get(2).ok_or("output directory required")?)?;
        return Ok(());
    }
    if args
        .get(1)
        .is_some_and(|a| a == "--verify-graphics-stages" || a == "--verify-graphics-stages-memory")
    {
        rebuild_stages::run(
            args.get(2).ok_or("output directory required")?,
            args[1].ends_with("-memory"),
        )?;
        return Ok(());
    }
    if args.get(1).is_some_and(|a| {
        a == "--verify-graphics-faults"
            || a == "--verify-graphics-faults-memory"
            || a == "--verify-audio-faults"
    }) {
        let path = args.get(2).ok_or("output directory required")?;
        if args[1].starts_with("--verify-graphics-faults") {
            graphics_faults::run(path, args[1].ends_with("-memory"))?;
        } else {
            editor_audio::faults::run(path)?;
        }
        return Ok(());
    }
    if args
        .get(1)
        .is_some_and(|a| a == "--verify-resource-cycles" || a == "--verify-resource-cycles-smoke")
    {
        editor_audio::cycles::run(
            args.get(2).ok_or("output directory required")?,
            args[1].ends_with("smoke"),
        )?;
        return Ok(());
    }
    if args
        .get(1)
        .is_some_and(|a| a == "--verify-streamed-combined")
    {
        editor_audio::combined::run(args.get(2).ok_or("output directory required")?)?;
        return Ok(());
    }
    if args
        .get(1)
        .is_some_and(|a| a == "--verify-stable-empty" || a == "--verify-stable-resources")
    {
        editor_audio::stability::run(
            args.get(2).ok_or("output directory required")?,
            args[1] == "--verify-stable-resources",
        )?;
        return Ok(());
    }
    if let Some(mode) = args.get(1).and_then(|s| gui_baseline::Mode::from_flag(s)) {
        editor_audio::gui_verification::baseline(
            args.get(2).ok_or("output directory required")?,
            mode,
        )?;
        return Ok(());
    }
    if args.get(1).is_some_and(|a| {
        a == "--verify-recovery-with-trigger" || a == "--verify-recovery-after-refill"
    }) {
        combined_verification::phase_controlled_run(
            args.get(2).ok_or("output directory required")?,
            args[1] == "--verify-recovery-after-refill",
        )?;
        return Ok(());
    }
    if args
        .get(1)
        .is_some_and(|a| a == "--verify-voice-phase-recovery")
    {
        combined_verification::phase_recovery_run(args.get(2).ok_or("output directory required")?)?;
        return Ok(());
    }
    if args.get(1).is_some_and(|a| a == "--verify-voice-phase") {
        combined_verification::phase_run(args.get(2).ok_or("output directory required")?)?;
        return Ok(());
    }
    if args
        .get(1)
        .is_some_and(|a| a == "--verify-staging-recovery")
    {
        combined_verification::recovery_run(args.get(2).ok_or("output directory required")?)?;
        return Ok(());
    }
    if args
        .get(1)
        .is_some_and(|a| a == "--verify-cold" || a == "--verify-cold-memory")
    {
        return combined_verification::cold_run(
            args.get(2).ok_or("output directory required")?,
            args[1].ends_with("memory"),
        )
        .map_err(Into::into);
    }
    if args.get(1).is_some_and(|a| {
        a == "--verify-combined-staging-memory" || a == "--verify-cold-staging-memory"
    }) {
        return combined_verification::staging_run(
            args.get(2).ok_or("output directory required")?,
            args[1].starts_with("--verify-cold"),
        )
        .map_err(Into::into);
    }
    if args
        .get(1)
        .is_some_and(|a| a == "--verify-gpu-staging" || a == "--verify-gpu-staging-memory")
    {
        return verification::run_staging(
            args.get(2).ok_or("output directory required")?,
            args[1].ends_with("memory"),
        )
        .map_err(Into::into);
    }
    if args.get(1).is_some_and(|a| a == "--verify-combined-memory") {
        return combined_verification::memory_run(args.get(2).ok_or("output directory required")?)
            .map_err(Into::into);
    }
    if args.get(1).is_some_and(|a| a == "--verify-gpu-memory") {
        return verification::run_memory(args.get(2).ok_or("output directory required")?)
            .map_err(Into::into);
    }
    if args.get(1).is_some_and(|a| {
        a == "--verify-gpu-cost"
            || a == "--verify-gpu-cost-large"
            || a == "--verify-gpu-cost-memory"
    }) {
        return gpu_observation::baseline(
            args.get(2).ok_or("output directory required")?,
            args[1].ends_with("large"),
            args[1].ends_with("memory"),
        )
        .map_err(Into::into);
    }
    if args
        .get(1)
        .is_some_and(|a| a == "--verify-combined-control-memory")
    {
        return combined_verification::memory_control_run(
            args.get(2).ok_or("output directory required")?,
        )
        .map_err(Into::into);
    }
    if args
        .get(1)
        .is_some_and(|a| a == "--verify-combined-control")
    {
        return combined_verification::control_run(args.get(2).ok_or("output directory required")?)
            .map_err(Into::into);
    }
    if args.get(1).is_some_and(|a| a == "--verify-combined") {
        return combined_verification::run(args.get(2).ok_or("output directory required")?)
            .map_err(Into::into);
    }
    let smoke = args.get(1).is_some_and(|a| a == "--smoke-window");
    if args.get(1).is_some_and(|a| a == "--verify-audio-startup") {
        return load_verification::startup(args.get(2).ok_or("output directory required")?)
            .map_err(Into::into);
    }
    if args.get(1).is_some_and(|a| a == "--verify-audio-edit") {
        return edit_verification::run(args.get(2).ok_or("output directory required")?)
            .map_err(Into::into);
    }
    if args.get(1).is_some_and(|a| a == "--verify-audio-load") {
        return load_verification::run(args.get(2).ok_or("output directory required")?)
            .map_err(Into::into);
    }
    if args.get(1).is_some_and(|a| a == "--verify-audio") {
        return audio_verification::run(args.get(2).ok_or("output directory required")?)
            .map_err(Into::into);
    }
    if args.get(1).is_some_and(|a| a == "--verify-audio-supply") {
        return supply_verification::run(args.get(2).ok_or("output directory required")?)
            .map_err(Into::into);
    }
    if args
        .get(1)
        .is_some_and(|a| a == "--verify-separated-native")
    {
        return separated_verification::run(args.get(2).ok_or("output directory required")?)
            .map_err(Into::into);
    }
    if args
        .get(1)
        .is_some_and(|a| a == "--verify-separated-worker")
    {
        return worker_verification::run(args.get(2).ok_or("output directory required")?)
            .map_err(Into::into);
    }
    if args.get(1).is_some_and(|a| a == "--verify-main-work-gui") {
        editor_audio::gui_verification::main_work(args.get(2).ok_or("output directory required")?)?;
        return Ok(());
    }
    if args.get(1).is_some_and(|a| a == "--verify-text-work-gui") {
        editor_audio::gui_verification::text_work(args.get(2).ok_or("output directory required")?)?;
        return Ok(());
    }
    if args
        .get(1)
        .is_some_and(|a| a == "--verify-dispatch-gui" || a == "--verify-dispatch-repaint-gui")
    {
        #[cfg(feature = "framework-trace")]
        return framework_trace::run(
            args.get(2).ok_or("output directory required")?,
            args[1] == "--verify-dispatch-gui",
        )
        .map_err(Into::into);
        #[cfg(not(feature = "framework-trace"))]
        return Err("explicit framework-trace diagnostic build required".into());
    }
    if args
        .get(1)
        .is_some_and(|a| a == "--verify-repaint-only-gui")
    {
        editor_audio::gui_verification::repaint_only(
            args.get(2).ok_or("output directory required")?,
        )?;
        return Ok(());
    }
    if args
        .get(1)
        .is_some_and(|a| a == "--verify-gpu-path-gui" || a == "--verify-upload-api-gui")
    {
        editor_audio::gui_verification::gpu_path(
            args.get(2).ok_or("output directory required")?,
            args[1] == "--verify-upload-api-gui",
        )?;
        return Ok(());
    }
    if args.get(1).is_some_and(|a| a == "--verify-gpu-markers") {
        verification::run_markers(args.get(2).ok_or("output directory required")?)?;
        return Ok(());
    }
    if args.get(1).is_some_and(|a| a == "--verify-mesh-replay-gui") {
        editor_audio::gui_verification::mesh_replay(
            args.get(2).ok_or("output directory required")?,
        )?;
        return Ok(());
    }
    if args.get(1).is_some_and(|a| a == "--verify-profiled-gui") {
        editor_audio::gui_verification::profiled(args.get(2).ok_or("output directory required")?)?;
        return Ok(());
    }
    if args.get(1).is_some_and(|a| a == "--verify-streamed-gui") {
        editor_audio::gui_verification::streamed(args.get(2).ok_or("output directory required")?)?;
        return Ok(());
    }
    if args.get(1).is_some_and(|a| a == "--verify-streamed-worker") {
        worker_verification::streamed(args.get(2).ok_or("output directory required")?)?;
        return Ok(());
    }
    if args.get(1).is_some_and(|a| a == "--verify-candidate-gui") {
        return editor_audio::gui_verification::run(
            args.get(2).ok_or("output directory required")?,
        )
        .map_err(Into::into);
    }
    if let Some(mode) = args
        .get(1)
        .and_then(|a| a.strip_prefix("--verify-product-"))
    {
        let destination = args.get(2).ok_or("output directory required")?;
        if !product_policy::Policy::CURRENT.separated_audio {
            return Err("product acceptance currently requires Windows".into());
        }
        match mode {
            "combined" => editor_audio::combined::run_policy(destination, true)?,
            "cycles" => editor_audio::cycles::run_policy(destination, false, true)?,
            "cycles-smoke" => editor_audio::cycles::run_policy(destination, true, true)?,
            "empty" => editor_audio::stability::run_policy(destination, false, true)?,
            "stable" => editor_audio::stability::run_policy(destination, true, true)?,
            _ => return Err("unknown product verification".into()),
        }
        return Ok(());
    }
    let candidate_audio = args.get(1).is_some_and(|a| a == "--candidate-audio");
    if args.get(1).is_some_and(|a| a == "--verify-gpu") {
        return verification::run(args.get(2).ok_or("output directory required")?)
            .map_err(Into::into);
    }
    if args.get(1).is_some_and(|a| a == "--verify-audio-ui") {
        return verification::run_with_audio(args.get(2).ok_or("output directory required")?, true)
            .map_err(Into::into);
    }
    let options = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        wgpu_options: product_policy::Policy::CURRENT.graphics(),
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1180.0, 800.0])
            .with_min_inner_size([820.0, 600.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Skin Editor · P2",
        options,
        Box::new(move |context| {
            Editor::new(context, smoke)
                .map(|mut app| {
                    if candidate_audio {
                        app.ui
                            .set_audio_port(Box::new(editor_audio::EditorAudio::candidate(
                                app.images.blobs().clone(),
                            )));
                    }
                    app
                })
                .map(|app| Box::new(app) as Box<dyn eframe::App>)
                .map_err(|e| std::io::Error::other(e).into())
        }),
    )?;
    Ok(())
}
