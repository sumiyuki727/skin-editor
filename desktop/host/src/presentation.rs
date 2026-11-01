use eframe::{
    egui,
    egui_wgpu::{self, wgpu},
};
use skin_editor_application::{values::*, *};
use skin_editor_infrastructure::Images;
use skin_editor_rendering::{GpuComposer, GpuImage, Plan, Target};
use skin_editor_ui::{PresentedView, Presenter};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{
    Arc, OnceLock, Weak,
    atomic::{AtomicBool, Ordering},
};
use std::time::Instant;

pub struct PreparationTrace {
    pub source: FrameSource,
    pub started_ms: f64,
    pub plan_ms: f64,
    pub decode_ms: f64,
    pub worker_started_ms: f64,
    pub worker_finished_ms: f64,
    pub queue_ms: f64,
    pub received_ms: f64,
    pub gpu_done_ms: Option<f64>,
    gpu_done: Option<Arc<OnceLock<Instant>>>,
    pub gpu: skin_editor_rendering::GpuPreparation,
    pub published_ms: Option<f64>,
}

struct NativeTarget {
    allocation: u64,
    image: GpuImage,
    group: wgpu::BindGroup,
}
struct Frame {
    descriptor: FrameDescriptor,
    view: Affine2,
    target: Arc<NativeTarget>,
    trace: Option<usize>,
    publishable: bool,
}
struct Prepared {
    image: GpuImage,
    view: Affine2,
    decode_ms: f64,
}
/// One incarnation only. The framework owns callback installation; standalone
/// verification installs it on its own device, never overwriting eframe's hook.
#[derive(Clone, Default)]
pub(crate) struct DeviceLoss(Arc<AtomicBool>);
impl DeviceLoss {
    pub fn report(&self) {
        self.0.store(true, Ordering::Release);
    }
    fn is_lost(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}
pub struct NativeHost {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    composer: Arc<GpuComposer>,
    worker: crate::frame_worker::FrameWorker<Prepared>,
    layout: wgpu::BindGroupLayout,
    pipeline: Arc<wgpu::RenderPipeline>,
    active: Option<Frame>,
    candidate: Option<Frame>,
    retired: Vec<Weak<NativeTarget>>,
    submitted: Vec<Arc<NativeTarget>>,
    desired: Option<FrameDescriptor>,
    next: u64,
    next_allocation: u64,
    images: Images,
    thumbs: BTreeMap<BlobRef, egui::TextureHandle>,
    ctx: egui::Context,
    pending: Arc<AtomicBool>,
    pub allocations: u64,
    pub publications: u64,
    trace_started: Option<Instant>,
    pub traces: Vec<PreparationTrace>,
    pub trace_overflow: bool,
    shutdown_ok: bool,
    loss: DeviceLoss,
    loss_binding_open: bool,
    lost_cleaned: bool,
    upload: skin_editor_rendering::SourceUpload,
}
/// No Device/Queue/pipeline/texture or pending draw may cross this handoff.
pub(crate) struct RebuildSeed {
    images: Images,
    upload: skin_editor_rendering::SourceUpload,
    next: u64,
    next_allocation: u64,
}
impl RebuildSeed {
    pub fn build(
        self,
        device: wgpu::Device,
        queue: wgpu::Queue,
        format: wgpu::TextureFormat,
        ctx: egui::Context,
    ) -> Result<NativeHost, PreviewError> {
        self.build_retained(device, queue, format, ctx)
    }
    /// A failed constructor must not consume the only retry recipe. This
    /// contains no GPU owner; the outer window owner still gates device reuse.
    pub(crate) fn build_retained(
        &self,
        device: wgpu::Device,
        queue: wgpu::Queue,
        format: wgpu::TextureFormat,
        ctx: egui::Context,
    ) -> Result<NativeHost, PreviewError> {
        let mut host =
            NativeHost::with_upload(device, queue, format, self.images.clone(), ctx, self.upload)?;
        host.next = self.next;
        host.next_allocation = self.next_allocation;
        Ok(host)
    }
}
fn material_equal(a: FrameSource, b: FrameSource) -> bool {
    a.session == b.session
        && a.state == b.state
        && a.context.target == b.context.target
        && a.context.draft == b.context.draft
        && a.context.size == b.context.size
        && a.context.viewport_generation == b.context.viewport_generation
}
pub(crate) fn candidate_current(
    desired: Option<FrameSource>,
    source: FrameSource,
    cancelled: bool,
) -> bool {
    !cancelled && desired.is_some_and(|d| material_equal(d, source))
}
fn ready_descriptor(
    desired: Option<FrameDescriptor>,
    source: FrameSource,
    publishable: bool,
) -> Option<FrameDescriptor> {
    desired
        .filter(|d| candidate_current(Some(d.source), source, !publishable))
        .map(|d| FrameDescriptor {
            status: FrameStatus::Ready,
            ..d
        })
}
impl NativeHost {
    /// The native owner installs its device callback before the first request.
    /// Binding may not replace the fence of an already-used incarnation.
    pub(crate) fn bind_loss(&mut self, loss: DeviceLoss) -> Result<(), PreviewError> {
        if !self.loss_binding_open {
            return Err(PreviewError::Backend(
                "device loss fence already in use".into(),
            ));
        }
        self.loss = loss;
        self.loss_binding_open = false;
        Ok(())
    }
    pub fn with_upload(
        device: wgpu::Device,
        queue: wgpu::Queue,
        format: wgpu::TextureFormat,
        images: Images,
        ctx: egui::Context,
        upload: skin_editor_rendering::SourceUpload,
    ) -> Result<Self, PreviewError> {
        let composer = Arc::new(GpuComposer::with_upload(&device, upload));
        let worker = {
            let (device, queue, composer, images, ctx) = (
                device.clone(),
                queue.clone(),
                composer.clone(),
                images.clone(),
                ctx.clone(),
            );
            let wake = ctx.clone();
            crate::frame_worker::FrameWorker::new(
                move |job| {
                    let Some((rasters, decode_ms)) = crate::frame_worker::decode(job, &images)?
                    else {
                        return Ok(None);
                    };
                    let ctx = ctx.clone();
                    // Once submitted, even a cancelled image must reach the host
                    // for GPU completion tracking. Never discard an in-flight lease.
                    let image = composer
                        .render(&device, &queue, &job.plan, &rasters, true, move || {
                            ctx.request_repaint()
                        })
                        .map_err(|e| PreviewError::Backend(e.0))?;
                    Ok(Some(Prepared {
                        image,
                        view: job.plan.view,
                        decode_ms,
                    }))
                },
                move || wake.request_repaint(),
            )?
        };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("opaque frame presentation"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            }],
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("preview presentation"),
            source: wgpu::ShaderSource::Wgsl(
                DISPLAY
                    .replace(
                        "TARGET_SRGB",
                        if format.is_srgb() { "true" } else { "false" },
                    )
                    .into(),
            ),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("frame bridge"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            multiview_mask: None,
            cache: None,
        });
        Ok(Self {
            device,
            queue,
            composer,
            worker,
            layout,
            pipeline: Arc::new(pipeline),
            active: None,
            candidate: None,
            retired: vec![],
            submitted: vec![],
            desired: None,
            next: 1,
            next_allocation: 1,
            images,
            thumbs: BTreeMap::new(),
            ctx,
            pending: Arc::new(AtomicBool::new(false)),
            allocations: 0,
            publications: 0,
            trace_started: None,
            traces: vec![],
            trace_overflow: false,
            shutdown_ok: true,
            loss: DeviceLoss::default(),
            loss_binding_open: true,
            lost_cleaned: false,
            upload,
        })
    }
    pub fn enable_trace(&mut self) {
        if self.trace_started.is_some() {
            return;
        }
        self.trace_started = Some(Instant::now());
        self.traces = Vec::with_capacity(512);
    }
    fn retire(&mut self, frame: Frame) {
        self.retired.push(Arc::downgrade(&frame.target));
        if !frame.target.image.ready.load(Ordering::Acquire) {
            self.submitted.push(frame.target);
        }
    }
    pub fn wait_preparation(&mut self) -> Result<(), PreviewError> {
        self.worker.wait()
    }
    pub fn poll(&mut self) -> Result<(), PreviewError> {
        if self.loss.is_lost() {
            self.clean_lost_device();
            return Err(PreviewError::Backend(
                "preview device stopped; explicit rebuild required".into(),
            ));
        }
        let result = self.poll_live();
        if self.loss.is_lost() {
            self.clean_lost_device();
            return Err(PreviewError::Backend(
                "preview device stopped during poll".into(),
            ));
        }
        result
    }
    fn poll_live(&mut self) -> Result<(), PreviewError> {
        // Dispatch device loss before accepting a completed preparation. Its
        // buffers may belong to a dead device even though the worker finished.
        self.device
            .poll(wgpu::PollType::Poll)
            .map_err(|e| PreviewError::Backend(e.to_string()))?;
        if self.loss.is_lost() {
            return Ok(());
        }
        if let Some(done) = self.worker.poll()? {
            self.accept_prepared(done)?;
        }
        self.retired.retain(|f| f.strong_count() > 0);
        self.submitted
            .retain(|t| !t.image.ready.load(Ordering::Acquire));
        if self
            .candidate
            .as_ref()
            .is_some_and(|f| f.target.image.ready.load(Ordering::Acquire))
        {
            let frame = self.candidate.take().unwrap();
            if let Some(descriptor) =
                ready_descriptor(self.desired, frame.descriptor.source, frame.publishable)
            {
                let ready = Frame {
                    descriptor,
                    view: frame.view,
                    target: frame.target.clone(),
                    trace: frame.trace,
                    publishable: true,
                };
                if let (Some(start), Some(i)) = (self.trace_started, frame.trace) {
                    let trace = &mut self.traces[i];
                    trace.gpu_done_ms = trace
                        .gpu_done
                        .as_ref()
                        .and_then(|r| r.get())
                        .map(|at| at.duration_since(start).as_secs_f64() * 1000.);
                    trace.published_ms = Some(start.elapsed().as_secs_f64() * 1000.);
                }
                if let Some(old) = self.active.replace(ready) {
                    self.retire(old);
                }
                self.publications += 1;
            }
        }
        Ok(())
    }
    pub fn loss_signal(&self) -> DeviceLoss {
        self.loss.clone()
    }
    pub fn is_lost(&self) -> bool {
        self.loss.is_lost()
    }
    /// No valid target remains after a cancelled edit. Reject old completions
    /// without pretending their tasks, queued draws or leases have finished.
    pub fn invalidate_source(&mut self) {
        self.desired = None;
        self.worker.cancel();
        for frame in [self.active.take(), self.candidate.take()]
            .into_iter()
            .flatten()
        {
            self.retired.push(Arc::downgrade(&frame.target));
        }
        self.pending
            .store(self.worker.current().is_some(), Ordering::Release);
    }
    fn clean_lost_device(&mut self) {
        if self.lost_cleaned {
            return;
        }
        self.desired = None;
        self.worker.cancel();
        // A completion may contain an old-device image. Drain/drop it without
        // creating bindings or waiting for an old GPU acknowledgement.
        let finished = self.worker.try_close();
        for frame in [self.active.take(), self.candidate.take()]
            .into_iter()
            .flatten()
        {
            self.retired.push(Arc::downgrade(&frame.target));
        }
        for target in self.submitted.drain(..) {
            self.retired.push(Arc::downgrade(&target));
        }
        self.thumbs.clear();
        if finished {
            self.composer.clear_sources();
        }
        self.pending.store(!finished, Ordering::Release);
        self.shutdown_ok = false;
        self.lost_cleaned = finished;
    }
    /// Return a GPU-free handoff only after actual task/draw retirement.
    /// Err retains the old owner; the caller must keep it and continue polling.
    /// A replacement device must be opened only after this consumes/drops self.
    pub fn into_rebuild_seed(mut self) -> Result<RebuildSeed, Box<Self>> {
        if !self.loss.is_lost() {
            return Err(Box::new(self));
        }
        self.clean_lost_device();
        if !self.worker.closed() || self.live_targets() != 0 {
            return Err(Box::new(self));
        }
        let seed = RebuildSeed {
            images: self.images.clone(),
            upload: self.upload,
            next: self.next,
            next_allocation: self.next_allocation,
        };
        drop(self);
        Ok(seed)
    }
    pub fn needs_work(&self) -> bool {
        self.pending.load(Ordering::Acquire)
    }
    pub fn preparation_closed(&self) -> bool {
        self.worker.closed()
    }
    pub fn preparation_pending(&self) -> bool {
        self.worker.current().is_some()
    }
    pub fn shutdown_succeeded(&self) -> bool {
        self.shutdown_ok && self.worker.closed()
    }
    pub fn shutdown(&mut self) {
        if self.loss.is_lost() {
            self.clean_lost_device();
            return;
        }
        self.desired = None;
        self.worker.cancel();
        let _ = self.worker.wait();
        let _ = self.poll();
        self.worker.close();
        if let Some(frame) = self.active.take() {
            self.retire(frame);
        }
        if let Some(frame) = self.candidate.take() {
            self.retire(frame);
        }
        self.thumbs.clear();
        self.composer.clear_sources();
        self.pending.store(false, Ordering::Release);
        self.shutdown_ok &= self
            .device
            .poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: Some(std::time::Duration::from_secs(3)),
            })
            .is_ok();
        let _ = self.poll();
    }
    pub fn active_texture(&self) -> Option<&wgpu::Texture> {
        if self.loss.is_lost() {
            return None;
        }
        Some(&self.active.as_ref()?.target.image.texture)
    }
    pub fn source_usage(&self) -> (u64, usize) {
        self.composer.source_usage()
    }
    pub fn staging_usage(&self) -> skin_editor_rendering::StagingUsage {
        self.composer.staging_usage()
    }
    pub fn source_allocations(&self) -> u64 {
        self.composer.source_allocations()
    }
    pub fn live_targets(&self) -> usize {
        self.retired
            .iter()
            .filter_map(Weak::upgrade)
            .chain(self.active.iter().map(|f| f.target.clone()))
            .chain(self.candidate.iter().map(|f| f.target.clone()))
            .chain(self.submitted.iter().cloned())
            .map(|t| t.allocation)
            .collect::<BTreeSet<_>>()
            .len()
    }
}
impl PreviewPort for NativeHost {
    fn request(
        &mut self,
        project: &SkinProject,
        source: FrameSource,
    ) -> Result<FrameDescriptor, PreviewError> {
        self.loss_binding_open = false;
        if self.desired.is_none_or(|d| d.source != source) {
            let key = FrameKey(self.next);
            self.next = self
                .next
                .checked_add(1)
                .ok_or(PreviewError::ResourceLimit)?;
            self.desired = Some(FrameDescriptor {
                api_version: PREVIEW_API_VERSION,
                key,
                source,
                status: FrameStatus::Pending,
            });
        }
        if self
            .worker
            .current()
            .is_some_and(|d| !material_equal(d.source, source))
        {
            self.worker.cancel();
        }
        self.poll()?;
        let desired = self.desired.unwrap();
        if let Some(active) = &mut self.active
            && material_equal(active.descriptor.source, source)
        {
            if active.descriptor.source != source {
                // UI receives descriptor copies; pending GPU draws own only
                // the immutable target. Time aliases need no Frame allocation
                // or retirement, and cannot mutate an old target lease.
                active.descriptor = FrameDescriptor {
                    status: FrameStatus::Ready,
                    ..desired
                };
            }
            self.pending.store(false, Ordering::Release);
            return Ok(self.active.as_ref().unwrap().descriptor);
        }
        self.pending.store(true, Ordering::Release);
        if self.candidate.is_some() || self.worker.current().is_some() {
            return Ok(desired);
        }
        let mut targets = BTreeSet::new();
        for target in self
            .retired
            .iter()
            .filter_map(Weak::upgrade)
            .chain(self.active.iter().map(|f| f.target.clone()))
        {
            targets.insert(target.allocation);
        }
        if targets.len() >= 8 {
            return Ok(desired);
        }
        let target = match source.context.target {
            PreviewTarget::Document(id) => Target::Document(id),
            PreviewTarget::Components => Target::Components,
            PreviewTarget::Workspace => Target::Workspace,
        };
        let size = PixelSize::new(source.context.size[0], source.context.size[1])
            .map_err(|_| PreviewError::InvalidViewport)?;
        let started_ms = self
            .trace_started
            .map_or(0., |s| s.elapsed().as_secs_f64() * 1000.);
        let started = Instant::now();
        let plan = Plan::build(project, target, size).map_err(|e| PreviewError::Backend(e.0))?;
        let plan_ms = started.elapsed().as_secs_f64() * 1000.;
        let raw = plan
            .sources()
            .into_iter()
            .map(|blob| {
                self.images
                    .blobs()
                    .get(blob)
                    .ok_or_else(|| PreviewError::Backend("missing original image".into()))
            })
            .collect::<Result<_, _>>()?;
        self.worker.submit(crate::frame_worker::Job {
            descriptor: desired,
            plan,
            queued: Instant::now(),
            started_ms,
            plan_ms,
            raw,
            cancelled: Arc::new(AtomicBool::new(false)),
        })?;
        Ok(desired)
    }
}
impl NativeHost {
    fn accept_prepared(
        &mut self,
        done: crate::frame_worker::Completion<Prepared>,
    ) -> Result<(), PreviewError> {
        if self.loss.is_lost() {
            return Ok(());
        }
        let matches = candidate_current(
            self.desired.map(|d| d.source),
            done.job.descriptor.source,
            done.job.is_cancelled(),
        );
        let prepared = match done.result {
            Ok(Some(prepared)) => prepared,
            Ok(None) => return Ok(()),
            Err(error) => return if matches { Err(error) } else { Ok(()) },
        };
        let received = Instant::now();
        let Prepared {
            image,
            view: transform,
            decode_ms,
        } = prepared;
        // Static material may alias only time/scene. Cancelled or different
        // material keeps its original identity and can never become active.
        let desired = if matches {
            self.desired.unwrap()
        } else {
            done.job.descriptor
        };
        let source = done.job.descriptor.source;
        let (started_ms, plan_ms) = (done.job.started_ms, done.job.plan_ms);
        let view = image.texture.create_view(&Default::default());
        let group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("frame-only binding"),
            layout: &self.layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            }],
        });
        let allocation = self.next_allocation;
        self.next_allocation += 1;
        self.allocations += 1;
        let trace = self.trace_started.and_then(|_| {
            if self.traces.len() == 512 {
                self.trace_overflow = true;
                None
            } else {
                let index = self.traces.len();
                self.traces.push(PreparationTrace {
                    source,
                    started_ms,
                    plan_ms,
                    decode_ms,
                    worker_started_ms: done
                        .started
                        .duration_since(self.trace_started.unwrap())
                        .as_secs_f64()
                        * 1000.,
                    worker_finished_ms: done
                        .finished
                        .duration_since(self.trace_started.unwrap())
                        .as_secs_f64()
                        * 1000.,
                    queue_ms: done.started.duration_since(done.job.queued).as_secs_f64() * 1000.,
                    received_ms: received
                        .duration_since(self.trace_started.unwrap())
                        .as_secs_f64()
                        * 1000.,
                    gpu_done_ms: None,
                    gpu_done: Some(image.completed.clone()),
                    gpu: image.preparation,
                    published_ms: None,
                });
                Some(index)
            }
        });
        self.candidate = Some(Frame {
            descriptor: desired,
            view: transform,
            trace,
            publishable: matches,
            target: Arc::new(NativeTarget {
                allocation,
                image,
                group,
            }),
        });
        Ok(())
    }
}
struct Draw {
    target: Arc<NativeTarget>,
    pipeline: Arc<wgpu::RenderPipeline>,
    pending: Arc<AtomicBool>,
    ctx: egui::Context,
    loss: DeviceLoss,
}
impl egui_wgpu::CallbackTrait for Draw {
    fn prepare(
        &self,
        _: &wgpu::Device,
        _: &wgpu::Queue,
        _: &egui_wgpu::ScreenDescriptor,
        encoder: &mut wgpu::CommandEncoder,
        _: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        if self.loss.is_lost() {
            return Vec::new();
        }
        let lease = self.target.clone();
        let pending = self.pending.clone();
        let ctx = self.ctx.clone();
        // The callback belongs to egui's actual draw submission. No auxiliary
        // empty encoder/buffer is needed to keep the target alive until done.
        encoder.on_submitted_work_done(move || {
            drop(lease);
            if pending.load(Ordering::Acquire) {
                ctx.request_repaint();
            }
        });
        Vec::new()
    }
    fn paint(
        &self,
        _: egui::PaintCallbackInfo,
        pass: &mut wgpu::RenderPass<'static>,
        _: &egui_wgpu::CallbackResources,
    ) {
        if self.loss.is_lost() {
            return;
        }
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.target.group, &[]);
        pass.draw(0..6, 0..1);
    }
}
impl Presenter for NativeHost {
    fn current(&self) -> Option<PresentedView> {
        if self.loss.is_lost() {
            return None;
        }
        let frame = self.active.as_ref()?;
        Some(PresentedView {
            frame: frame.descriptor,
            view: frame.view,
        })
    }
    fn paint(&mut self, ui: &mut egui::Ui, key: FrameKey, rect: egui::Rect) {
        if !self.loss.is_lost()
            && let Some(frame) = &self.active
            && frame.descriptor.key == key
        {
            ui.painter().add(egui_wgpu::Callback::new_paint_callback(
                rect,
                Draw {
                    target: frame.target.clone(),
                    pipeline: self.pipeline.clone(),
                    pending: self.pending.clone(),
                    ctx: self.ctx.clone(),
                    loss: self.loss.clone(),
                },
            ));
        }
    }
    fn thumbnail(&mut self, ui: &mut egui::Ui, blob: BlobRef) {
        if self.loss.is_lost() {
            return;
        }
        let (rect, _) = ui.allocate_exact_size(egui::Vec2::splat(40.0), egui::Sense::hover());
        if !ui.is_rect_visible(rect) {
            return;
        }
        if !self.thumbs.contains_key(&blob)
            && let Some(image) = self.images.raster(blob)
        {
            let mut pixels = vec![0u8; 40 * 40 * 4];
            let scale = (40.0 / image.size.width() as f32).min(40.0 / image.size.height() as f32);
            let width = (image.size.width() as f32 * scale).round().max(1.0) as usize;
            let height = (image.size.height() as f32 * scale).round().max(1.0) as usize;
            for y in 0..height {
                for x in 0..width {
                    let source = ((y as f32 / scale) as usize * image.size.width() as usize
                        + (x as f32 / scale) as usize)
                        * 4;
                    let dest = ((y + (40 - height) / 2) * 40 + x + (40 - width) / 2) * 4;
                    pixels[dest..dest + 4].copy_from_slice(&image.pixels[source..source + 4]);
                }
            }
            if self.thumbs.len() >= 128 {
                self.thumbs.pop_first();
            }
            self.thumbs.insert(
                blob,
                ui.ctx().load_texture(
                    format!("thumbnail {:?}", blob.sha256()),
                    egui::ColorImage::from_rgba_unmultiplied([40, 40], &pixels),
                    egui::TextureOptions::NEAREST,
                ),
            );
        }
        if let Some(texture) = self.thumbs.get(&blob) {
            ui.painter().image(
                texture.id(),
                rect,
                egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );
        }
    }
}
const DISPLAY: &str = r#"
@group(0) @binding(0) var source:texture_2d<f32>;
struct Vertex { @builtin(position) position:vec4<f32>, @location(0) uv:vec2<f32> }
@vertex fn vs_main(@builtin(vertex_index) i:u32)->Vertex {
 let p=array<vec2<f32>,6>(vec2(-1.0,1.0),vec2(1.0,1.0),vec2(-1.0,-1.0),vec2(-1.0,-1.0),vec2(1.0,1.0),vec2(1.0,-1.0));
 var out:Vertex;out.position=vec4(p[i],0.0,1.0);out.uv=(p[i]*vec2(1.0,-1.0)+vec2(1.0))/2.0;return out;
}
@fragment fn fs_main(v:Vertex)->@location(0) vec4<f32> {
 let size=textureDimensions(source);let p=clamp(vec2<i32>(v.uv*vec2<f32>(size)),vec2(0),vec2<i32>(size)-vec2(1));var color=textureLoad(source,p,0);
 if TARGET_SRGB {color=vec4(select(pow((color.rgb+vec3(0.055))/1.055,vec3(2.4)),color.rgb/12.92,color.rgb<=vec3(0.04045)),color.a);}
 return color;
}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn timed_candidate_publication_uses_current_descriptor_without_discarding_identical_material() {
        let source = FrameSource {
            session: HistorySessionId::new(1).unwrap(),
            state: ExpectedState {
                head: NodeId::new(1).unwrap(),
                revision: 1,
            },
            context: PreviewContext {
                target: PreviewTarget::Components,
                draft: None,
                size: [64, 64],
                viewport_generation: 1,
                scene: 0,
                simulation_micros: 0,
            },
        };
        for n in 1..10000 {
            let mut now = source;
            now.context.simulation_micros = n;
            now.context.scene = 1;
            let desired = FrameDescriptor {
                api_version: PREVIEW_API_VERSION,
                key: FrameKey(n),
                source: now,
                status: FrameStatus::Pending,
            };
            assert_eq!(
                ready_descriptor(Some(desired), source, true),
                Some(FrameDescriptor {
                    status: FrameStatus::Ready,
                    ..desired
                })
            );
            assert_eq!(ready_descriptor(Some(desired), source, false), None);
            for kind in 0..7 {
                let mut stale = source;
                match kind {
                    0 => stale.session = HistorySessionId::new(2).unwrap(),
                    1 => stale.state.head = NodeId::new(2).unwrap(),
                    2 => stale.state.revision += 1,
                    3 => stale.context.size = [32, 32],
                    4 => stale.context.viewport_generation += 1,
                    5 => {
                        stale.context.target = PreviewTarget::Document(DocumentId::new(2).unwrap())
                    }
                    _ => {
                        stale.context.draft = Some(DraftState {
                            id: DraftId::new(1).unwrap(),
                            revision: 1,
                            base: source.state,
                        })
                    }
                };
                assert_eq!(ready_descriptor(Some(desired), stale, true), None);
            }
        }
        assert_eq!(ready_descriptor(None, source, true), None);
    }
    #[test]
    fn time_alias_cannot_hide_a_project_draft_or_viewport_change() {
        let source = FrameSource {
            session: HistorySessionId::new(1).unwrap(),
            state: ExpectedState {
                head: NodeId::new(1).unwrap(),
                revision: 0,
            },
            context: PreviewContext {
                target: PreviewTarget::Components,
                draft: None,
                size: [64, 64],
                viewport_generation: 1,
                scene: 0,
                simulation_micros: 0,
            },
        };
        let mut next = source;
        next.context.simulation_micros = 100;
        assert!(material_equal(source, next));
        next.state.revision = 1;
        assert!(!material_equal(source, next));
        assert!(candidate_current(Some(source), source, false));
        assert!(!candidate_current(Some(source), source, true));
        assert!(!candidate_current(None, source, false));
        let mut variants = [source; 7];
        variants[0].session = HistorySessionId::new(2).unwrap();
        variants[1].state.head = NodeId::new(2).unwrap();
        variants[2].state.revision += 1;
        variants[3].context.target = PreviewTarget::Document(DocumentId::new(2).unwrap());
        variants[4].context.size = [32, 32];
        variants[5].context.viewport_generation += 1;
        variants[6].context.draft = Some(DraftState {
            id: DraftId::new(1).unwrap(),
            revision: 1,
            base: source.state,
        });
        for changed in variants {
            assert!(!candidate_current(Some(changed), source, false));
        }
        let draft = variants[6];
        let mut edited = draft;
        edited.context.draft.as_mut().unwrap().revision += 1;
        assert!(!candidate_current(Some(edited), draft, false));
        edited = draft;
        edited.context.draft = None;
        assert!(!candidate_current(Some(edited), draft, false));
        next = source;
        next.context.viewport_generation += 1;
        assert!(!material_equal(source, next));
    }
}
