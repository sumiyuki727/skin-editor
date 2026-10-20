//! Native P0 presentation adapter. Receives only a read-only gesture projection.
use crate::{
    composition::{Image, Limits, Node, compile_with_limits, render},
    composition_fixture::{load_document, translated_first},
    composition_gpu::GpuPlan,
};
use eframe::{egui, wgpu};
use std::{path::Path, sync::Arc, time::Instant};
fn opaque_reference(p: [u8; 4]) -> [u8; 4] {
    let alpha = f64::from(p[3]) / 255.;
    let mut out = [0; 4];
    for (i, bg) in [18., 22., 30.].into_iter().enumerate() {
        let value = crate::composition::linear(f64::from(p[i]) / 255.) * alpha
            + crate::composition::linear(bg / 255.) * (1. - alpha);
        let encoded = if value <= 0.0031308 {
            12.92 * value
        } else {
            1.055 * value.powf(1. / 2.4) - 0.055
        };
        out[i] = (encoded * 255.).round() as u8;
    }
    out[3] = 255;
    out
}
fn pipelines(device: &wgpu::Device) -> (wgpu::ComputePipeline, wgpu::ComputePipeline) {
    let make = |label, source: &'static str| {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(label),
            source: wgpu::ShaderSource::Wgsl(source.into()),
        });
        device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some(label),
            layout: None,
            module: &shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        })
    };
    let pipeline = make("live-composition", include_str!("composition.wgsl"));
    let bridge = make(
        "editor-presentation",
        include_str!("composition_present.wgsl"),
    );
    (pipeline, bridge)
}

#[derive(Clone)]
pub struct Pipelines(wgpu::ComputePipeline, wgpu::ComputePipeline);
/// Optional one-shot GPU adapter task. No document, file or project dependencies.
pub struct PipelineTask {
    rx: Option<std::sync::mpsc::Receiver<Result<(Pipelines, f64), String>>>,
    thread: Option<std::thread::JoinHandle<()>>,
}
impl PipelineTask {
    pub fn start(device: wgpu::Device, wake: impl FnOnce() + Send + 'static) -> Self {
        Self::build(
            move || {
                let (a, b) = pipelines(&device);
                Ok(Pipelines(a, b))
            },
            wake,
        )
    }
    fn build(
        build: impl FnOnce() -> Result<Pipelines, String> + Send + 'static,
        wake: impl FnOnce() + Send + 'static,
    ) -> Self {
        let (tx, rx) = std::sync::mpsc::sync_channel(1);
        let thread = std::thread::spawn(move || {
            let start = Instant::now();
            let result = build().map(|p| (p, start.elapsed().as_secs_f64() * 1000.));
            let _ = tx.send(result);
            wake();
        });
        Self {
            rx: Some(rx),
            thread: Some(thread),
        }
    }
    pub fn poll(&mut self) -> Option<Result<(Pipelines, f64), String>> {
        match self.rx.as_ref()?.try_recv() {
            Ok(r) => Some(r),
            Err(std::sync::mpsc::TryRecvError::Empty) => None,
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                Some(Err("Pipeline worker disconnected".into()))
            }
        }
    }
}
impl Drop for PipelineTask {
    fn drop(&mut self) {
        self.rx.take();
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

pub struct LiveComposition {
    state: eframe::egui_wgpu::RenderState,
    sources: Arc<[Image]>,
    nodes: Vec<Node>,
    job: GpuPlan,
    pipeline: wgpu::ComputePipeline,
    bridge: wgpu::ComputePipeline,
    _texture: wgpu::Texture,
    view: wgpu::TextureView,
    pub texture_id: egui::TextureId,
    position: Option<[f64; 2]>,
    pub updates: usize,
    pub operation_upload_bytes: usize,
}
impl LiveComposition {
    pub fn new(context: &eframe::CreationContext<'_>, fixture: &Path) -> Result<Self, String> {
        let state = context.wgpu_render_state.clone().ok_or("wgpu required")?;
        let (sources, nodes) = load_document(fixture, 0)?;
        let (pipeline, bridge) = pipelines(&state.device);
        let plan = compile_with_limits([1024, 1024], &sources, &nodes, Limits::STANDARD)?;
        let job = GpuPlan::new(&state.device, &pipeline, sources.clone(), &plan)?;
        Ok(Self::assemble(state, sources, nodes, job, pipeline, bridge))
    }
    fn assemble(
        state: eframe::egui_wgpu::RenderState,
        sources: Arc<[Image]>,
        nodes: Vec<Node>,
        job: GpuPlan,
        pipeline: wgpu::ComputePipeline,
        bridge: wgpu::ComputePipeline,
    ) -> Self {
        let device = &state.device;
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("composition-display"),
            size: wgpu::Extent3d {
                width: 1024,
                height: 1024,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = texture.create_view(&Default::default());
        let texture_id = state.renderer.write().register_native_texture(
            device,
            &view,
            wgpu::FilterMode::Nearest,
        );
        Self {
            state,
            sources,
            nodes,
            job,
            pipeline,
            bridge,
            _texture: texture,
            view,
            texture_id,
            position: None,
            updates: 0,
            operation_upload_bytes: 0,
        }
    }
    pub fn refresh(&mut self, position: [f64; 2]) -> Result<bool, String> {
        if self.position == Some(position) {
            return Ok(false);
        }
        let nodes = translated_first(&self.nodes, [position[0] - 40., position[1] - 40.])?;
        let plan = compile_with_limits([1024, 1024], &self.sources, &nodes, Limits::STANDARD)?;
        self.operation_upload_bytes += self.job.update(&self.state.queue, &plan)?;
        self.job.present(
            &self.state.device,
            &self.state.queue,
            &self.pipeline,
            &self.bridge,
            &self.view,
        );
        self.position = Some(position);
        self.updates += 1;
        Ok(true)
    }
    pub fn verify(&self, output: &Path, tag: u16) -> Result<serde_json::Value, String> {
        let start = Instant::now();
        let p = self.position.ok_or("No presented plan")?;
        let nodes = translated_first(&self.nodes, [p[0] - 40., p[1] - 40.])?;
        let plan = compile_with_limits([1024, 1024], &self.sources, &nodes, Limits::STANDARD)?;
        let cpu = render(&plan);
        let gpu = self
            .job
            .run(&self.state.device, &self.state.queue, &self.pipeline)?;
        // Diagnostic only: independently check the opaque presentation conversion.
        let display = self.read_display()?;
        image::save_buffer(
            output.join(format!("composition-{tag}-display.png")),
            &display,
            1024,
            1024,
            image::ColorType::Rgba8,
        )
        .map_err(|e| e.to_string())?;
        let mut display_max = 0;
        let mut display_outside = 0;
        for (p, actual) in gpu.image.rgba.iter().zip(display.chunks_exact(4)) {
            let expected = opaque_reference(*p);
            let error = expected
                .iter()
                .zip(actual)
                .map(|(a, b)| a.abs_diff(*b))
                .max()
                .unwrap();
            display_max = display_max.max(error);
            display_outside += usize::from(error > 2);
        }
        let mut max = 0;
        let mut outside = 0;
        for (a, b) in cpu.rgba.iter().zip(&gpu.image.rgba) {
            let error = a.iter().zip(b).map(|(a, b)| a.abs_diff(*b)).max().unwrap();
            max = max.max(error);
            outside += usize::from(error > 2);
        }
        for (name, img) in [("cpu", &cpu), ("gpu", &gpu.image)] {
            let bytes: Vec<_> = img.rgba.iter().flatten().copied().collect();
            image::save_buffer(
                output.join(format!("composition-{tag}-{name}.png")),
                &bytes,
                1024,
                1024,
                image::ColorType::Rgba8,
            )
            .map_err(|e| e.to_string())?;
        }
        if outside != 0 || display_outside != 0 {
            return Err(format!(
                "Composition proof exceeds tolerance: {outside} pixels, max {max}"
            ));
        }
        Ok(
            serde_json::json!({"stage":tag,"max_channel_error":max,"pixels_outside_tolerance":outside,"pixels":cpu.rgba.len(),"display_max_channel_error":display_max,"display_pixels_outside_tolerance":display_outside,"diagnostic_wall_ms":start.elapsed().as_secs_f64()*1000.}),
        )
    }
    fn read_display(&self) -> Result<Vec<u8>, String> {
        let device = &self.state.device;
        let read = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("display-diagnostic-read"),
            size: 4 * 1024 * 1024,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = device.create_command_encoder(&Default::default());
        encoder.copy_texture_to_buffer(
            self._texture.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &read,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4096),
                    rows_per_image: Some(1024),
                },
            },
            wgpu::Extent3d {
                width: 1024,
                height: 1024,
                depth_or_array_layers: 1,
            },
        );
        let submission = self.state.queue.submit([encoder.finish()]);
        let (tx, rx) = std::sync::mpsc::channel();
        read.slice(..).map_async(wgpu::MapMode::Read, move |r| {
            let _ = tx.send(r);
        });
        device
            .poll(wgpu::PollType::Wait {
                submission_index: Some(submission),
                timeout: Some(std::time::Duration::from_secs(10)),
            })
            .map_err(|e| format!("{e:?}"))?;
        rx.recv_timeout(std::time::Duration::from_secs(1))
            .map_err(|e| e.to_string())?
            .map_err(|e| e.to_string())?;
        let view = read
            .slice(..)
            .get_mapped_range()
            .map_err(|e| e.to_string())?;
        let pixels = view.to_vec();
        drop(view);
        read.unmap();
        Ok(pixels)
    }
    pub fn metrics(&self) -> serde_json::Value {
        serde_json::json!({"source_upload_batches":1,"source_upload_bytes":40*1024*1024,"updates":self.updates,"operation_upload_bytes":self.operation_upload_bytes,
            "resident_buffer_bytes":self.job.resident_buffer_bytes,"display_texture_bytes":4*1024*1024,"adapter":format!("{:?}",self.state.adapter.get_info()),
            "normal_path_cpu_pixel_readbacks":0,"normal_path_cpu_texture_uploads":0})
    }
}
impl Drop for LiveComposition {
    fn drop(&mut self) {
        self.state.renderer.write().free_texture(&self.texture_id);
    }
}

/// Holds an unpublished candidate until source writes and its first compute complete.
pub struct PendingComposition {
    state: eframe::egui_wgpu::RenderState,
    sources: Arc<[Image]>,
    nodes: Vec<Node>,
    upload: Option<crate::composition_gpu::PreparedPlan>,
    graphics: Option<(wgpu::ComputePipeline, wgpu::ComputePipeline)>,
    view: Option<LiveComposition>,
    completed: Arc<std::sync::atomic::AtomicBool>,
    budget: crate::loading::UploadBudget,
    pub uploaded_bytes: usize,
    pub upload_frames: usize,
    pub preparation: Option<serde_json::Value>,
    pub last_step: Option<serde_json::Value>,
}
impl PendingComposition {
    pub fn new(
        state: eframe::egui_wgpu::RenderState,
        loaded: crate::composition_fixture::LoadedDocument,
    ) -> Result<Self, String> {
        Self::profiled(state, loaded, false)
    }
    pub fn profiled(
        state: eframe::egui_wgpu::RenderState,
        loaded: crate::composition_fixture::LoadedDocument,
        profile: bool,
    ) -> Result<Self, String> {
        Self::prepared(state, loaded, profile, None)
    }
    pub fn prepared(
        state: eframe::egui_wgpu::RenderState,
        loaded: crate::composition_fixture::LoadedDocument,
        profile: bool,
        cached: Option<Pipelines>,
    ) -> Result<Self, String> {
        let start = profile.then(Instant::now);
        let (pipeline, bridge) = cached.map_or_else(|| pipelines(&state.device), |p| (p.0, p.1));
        let pipelines_ms = start.map(|s| s.elapsed().as_secs_f64() * 1000.);
        let compile_start = profile.then(Instant::now);
        let plan = compile_with_limits(
            [1024, 1024],
            &loaded.sources,
            &loaded.nodes,
            Limits::STANDARD,
        )?;
        let compile_ms = compile_start.map(|s| s.elapsed().as_secs_f64() * 1000.);
        let buffers_start = profile.then(Instant::now);
        let upload = GpuPlan::prepare(&state.device, &pipeline, loaded.sources.clone(), &plan)?;
        let preparation=start.map(|s|serde_json::json!({"pipelines_ms":pipelines_ms,"compile_ms":compile_ms,"buffers_ms":buffers_start.unwrap().elapsed().as_secs_f64()*1000.,"total_ms":s.elapsed().as_secs_f64()*1000.}));
        Ok(Self {
            state,
            sources: loaded.sources,
            nodes: loaded.nodes,
            upload: Some(upload),
            graphics: Some((pipeline, bridge)),
            view: None,
            completed: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            budget: Default::default(),
            uploaded_bytes: 0,
            upload_frames: 0,
            preparation,
            last_step: None,
        })
    }
    pub fn advance(
        &mut self,
        budget: &mut crate::loading::UploadBudget,
        ctx: &egui::Context,
    ) -> Result<bool, String> {
        use std::sync::atomic::Ordering;
        let start = self.preparation.as_ref().map(|_| Instant::now());
        let mut upload_ms = 0.;
        let mut first_frame_ms = 0.;
        if let Some(upload) = &mut self.upload {
            let upload_start = start.map(|_| Instant::now());
            let bytes = upload.upload(&self.state.queue, budget);
            if let Some(s) = upload_start {
                upload_ms = s.elapsed().as_secs_f64() * 1000.;
            }
            self.budget.peak_bytes = self.budget.peak_bytes.max(budget.bytes);
            self.budget.peak_count = self.budget.peak_count.max(budget.count);
            self.uploaded_bytes += bytes;
            self.upload_frames += usize::from(bytes != 0);
            if upload.complete() {
                let first_start = start.map(|_| Instant::now());
                let job = self.upload.take().unwrap().finish()?;
                let (pipeline, bridge) = self.graphics.take().unwrap();
                let mut view = LiveComposition::assemble(
                    self.state.clone(),
                    self.sources.clone(),
                    std::mem::take(&mut self.nodes),
                    job,
                    pipeline,
                    bridge,
                );
                view.refresh([40., 40.])?;
                let completed = self.completed.clone();
                let wake = ctx.clone();
                self.state.queue.on_submitted_work_done(move || {
                    completed.store(true, Ordering::Release);
                    wake.request_repaint();
                });
                self.view = Some(view);
                if let Some(s) = first_start {
                    first_frame_ms = s.elapsed().as_secs_f64() * 1000.;
                }
            }
        }
        let poll_start = start.map(|_| Instant::now());
        self.state
            .device
            .poll(wgpu::PollType::Poll)
            .map_err(|e| format!("{e:?}"))?;
        self.last_step=start.map(|s|serde_json::json!({"upload_ms":upload_ms,"first_frame_ms":first_frame_ms,"poll_ms":poll_start.unwrap().elapsed().as_secs_f64()*1000.,"total_ms":s.elapsed().as_secs_f64()*1000.}));
        Ok(self.completed.load(Ordering::Acquire))
    }
    pub fn metrics(&self) -> serde_json::Value {
        serde_json::json!({"uploaded_bytes":self.uploaded_bytes,"upload_frames":self.upload_frames,
            "max_source_bytes_per_frame":self.budget.peak_bytes,"max_sources_per_frame":self.budget.peak_count})
    }
    pub fn finish(mut self) -> Result<LiveComposition, String> {
        if !self.completed.load(std::sync::atomic::Ordering::Acquire) {
            return Err("GPU candidate not complete".into());
        }
        self.view.take().ok_or("Missing GPU view".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pipeline_task_reports_errors_and_joins_even_if_result_is_unconsumed() {
        let mut task = PipelineTask::build(|| Err("bad pipeline".into()), || {});
        let end = Instant::now() + std::time::Duration::from_secs(5);
        loop {
            if let Some(r) = task.poll() {
                assert!(matches!(r,Err(e) if e=="bad pipeline"));
                break;
            }
            assert!(Instant::now() < end);
            std::thread::yield_now();
        }
        let finished = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let completed = finished.clone();
        let task = PipelineTask::build(
            move || {
                completed.store(true, std::sync::atomic::Ordering::Release);
                Err("cancelled observer".into())
            },
            || {},
        );
        drop(task);
        assert!(finished.load(std::sync::atomic::Ordering::Acquire));
    }
    #[test]
    fn presentation_uses_linear_light_and_keeps_the_export_pixel_unchanged() {
        assert_eq!(opaque_reference([255, 0, 0, 0]), [18, 22, 30, 255]);
        assert_eq!(opaque_reference([20, 40, 60, 255]), [20, 40, 60, 255]);
        let p = [255, 255, 255, 128];
        assert_eq!(opaque_reference(p), [188, 189, 189, 255]);
        assert_eq!(p, [255, 255, 255, 128]);
    }
}
