//! Explicit host diagnostics. Markers observe the REAL draw without issuing
//! commands; upload replay uses a SEPARATE renderer, never the live UI buffers.
//! All times are monotonic CPU wall observations, not GPU timestamps/present.
use eframe::{
    egui,
    egui_wgpu::{self, wgpu},
};
use std::{
    sync::{
        Arc,
        atomic::{AtomicU64, AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};

const FRAMES: usize = 512;
const UPLOADS: usize = 16;
const NONE: usize = usize::MAX;
const ORDINALS: [u64; 4] = [1, 33, 65, 97];
#[derive(Clone, Copy, PartialEq)]
pub enum Mode {
    Markers,
    Upload,
}
struct Trace {
    origin: Instant,
    rows: Vec<[AtomicU64; 8]>,
    done: [AtomicU64; UPLOADS],
    active: AtomicUsize,
    invalid: AtomicU64,
}
impl Trace {
    fn new() -> Self {
        Self {
            origin: Instant::now(),
            rows: (0..FRAMES)
                .map(|_| std::array::from_fn(|_| AtomicU64::new(0)))
                .collect(),
            done: std::array::from_fn(|_| AtomicU64::new(0)),
            active: AtomicUsize::new(NONE),
            invalid: AtomicU64::new(0),
        }
    }
    fn now(&self) -> u64 {
        self.origin.elapsed().as_nanos() as u64 + 1
    }
    fn set(&self, slot: usize, field: usize, time: u64) {
        if slot >= FRAMES
            || field >= 8
            || time == 0
            || self.rows[slot][field]
                .compare_exchange(0, time, Ordering::AcqRel, Ordering::Acquire)
                .is_err()
        {
            self.invalid.fetch_add(1, Ordering::Relaxed);
        }
    }
    fn stamp(&self, slot: usize, field: usize) {
        self.set(slot, field, self.now());
    }
    fn values(&self, count: usize) -> Vec<[u64; 8]> {
        self.rows[..count]
            .iter()
            .map(|r| std::array::from_fn(|i| r[i].load(Ordering::Acquire)))
            .collect()
    }
}
struct Marker {
    trace: Arc<Trace>,
    slot: usize,
    first: bool,
}
impl egui_wgpu::CallbackTrait for Marker {
    fn prepare(
        &self,
        _: &wgpu::Device,
        _: &wgpu::Queue,
        _: &egui_wgpu::ScreenDescriptor,
        _: &mut wgpu::CommandEncoder,
        _: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        if self.first {
            self.trace.stamp(self.slot, 3);
        }
        vec![]
    }
    fn finish_prepare(
        &self,
        _: &wgpu::Device,
        _: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        _: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        if !self.first {
            self.trace.stamp(self.slot, 4);
            let trace = self.trace.clone();
            let slot = self.slot;
            // Only fixed scalar trace storage crosses this completion. No GPU,
            // project, audio resource or repaint request is retained/issued.
            encoder.on_submitted_work_done(move || trace.stamp(slot, 7));
        }
        vec![]
    }
    fn paint(
        &self,
        _: egui::epaint::PaintCallbackInfo,
        _: &mut wgpu::RenderPass<'static>,
        _: &egui_wgpu::CallbackResources,
    ) {
        self.trace.stamp(self.slot, if self.first { 5 } else { 6 });
    }
}
fn bracket(output: &mut egui::FullOutput, trace: &Arc<Trace>, slot: usize, rect: egui::Rect) {
    let marker = |first| egui::epaint::ClippedShape {
        clip_rect: rect,
        transform_after_tessellation: Default::default(),
        shape: egui::Shape::Callback(egui_wgpu::Callback::new_paint_callback(
            rect,
            Marker {
                trace: trace.clone(),
                slot,
                first,
            },
        )),
    };
    output.shapes.insert(0, marker(true));
    output.shapes.push(marker(false));
}
fn admit(shapes: &[egui::epaint::ClippedShape]) -> bool {
    fn walk(shape: &egui::Shape, depth: usize, nodes: &mut usize) -> bool {
        *nodes += 1;
        if depth > 32 || *nodes > 2048 {
            return false;
        }
        match shape {
            egui::Shape::Vec(v) => v.iter().all(|s| walk(s, depth + 1, nodes)),
            egui::Shape::Mesh(m) => m.vertices.len() <= 32768 && m.indices.len() <= 131072,
            _ => true,
        }
    }
    let mut nodes = 0;
    shapes.iter().all(|s| walk(&s.shape, 0, &mut nodes))
}
struct Upload {
    renderer: Option<egui_wgpu::Renderer>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    samples: Vec<[u64; 7]>,
    rows: Vec<[u64; 7]>,
    snapshots: Vec<String>,
}
pub struct Probe {
    pub stage: u8,
    mode: Mode,
    trace: Arc<Trace>,
    seen: u64,
    last_pass: Option<u64>,
    frames: usize,
    discarded: u64,
    error: Option<String>,
    upload: Option<Upload>,
    released: bool,
}
impl Probe {
    pub fn new(mode: Mode, state: Option<&egui_wgpu::RenderState>) -> Result<Self, String> {
        let upload = if mode == Mode::Upload {
            let s = state.ok_or("GPU state required for upload diagnostic")?;
            let before = crate::gpu_observation::snapshot(&s.device, "before_probe")?;
            let renderer = egui_wgpu::Renderer::new(&s.device, s.target_format, Default::default());
            Some(Upload {
                renderer: Some(renderer),
                device: s.device.clone(),
                queue: s.queue.clone(),
                samples: Vec::with_capacity(4),
                rows: Vec::with_capacity(UPLOADS),
                snapshots: vec![
                    before,
                    crate::gpu_observation::snapshot(&s.device, "created_probe")?,
                ],
            })
        } else {
            None
        };
        Ok(Self {
            stage: 0,
            mode,
            trace: Arc::new(Trace::new()),
            seen: 0,
            last_pass: None,
            frames: 0,
            discarded: 0,
            error: None,
            upload,
            released: false,
        })
    }
    pub fn after_tessellation(&self) {
        let slot = self.trace.active.load(Ordering::Acquire);
        if slot != NONE {
            self.trace.stamp(slot, 2);
        }
    }
    fn upload(
        &mut self,
        ctx: &egui::Context,
        output: &egui::FullOutput,
        pass: u64,
    ) -> Result<(), String> {
        let u = self.upload.as_mut().ok_or("missing upload renderer")?;
        let started = Instant::now();
        let mut meshes = ctx.tessellate(output.shapes.clone(), output.pixels_per_point);
        let tess_ns = started.elapsed().as_nanos() as u64;
        let mut vertices = 0;
        let mut indices = 0;
        let mut callbacks = 0;
        meshes.retain(|p| match &p.primitive {
            egui::epaint::Primitive::Mesh(m) => {
                vertices += m.vertices.len();
                indices += m.indices.len();
                true
            }
            egui::epaint::Primitive::Callback(_) => {
                callbacks += 1;
                false
            }
        });
        if vertices > 32768 || indices > 131072 || meshes.len() > 2048 {
            return Err("upload mesh capacity".into());
        }
        let rect = ctx.input(|i| i.viewport_rect());
        let size = rect.size() * output.pixels_per_point;
        if !size.is_finite() || size.x < 1. || size.y < 1. || size.x > 8192. || size.y > 8192. {
            return Err("upload viewport capacity".into());
        }
        let screen = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [size.x as u32, size.y as u32],
            pixels_per_point: output.pixels_per_point,
        };
        let sample = u.samples.len() as u64;
        u.samples.push([
            pass,
            self.seen,
            tess_ns,
            meshes.len() as u64,
            vertices as u64,
            indices as u64,
            callbacks,
        ]);
        for repeat in 0..4 {
            if u.rows.len() >= UPLOADS {
                return Err("upload sample capacity".into());
            }
            let at = Instant::now();
            let mut encoder = u.device.create_command_encoder(&Default::default());
            let create_ns = at.elapsed().as_nanos() as u64;
            let at = Instant::now();
            let buffers = u
                .renderer
                .as_mut()
                .ok_or("released renderer")?
                .update_buffers(&u.device, &u.queue, &mut encoder, &meshes, &screen);
            let update_ns = at.elapsed().as_nanos() as u64;
            if !buffers.is_empty() {
                return Err("upload-only replay executed a callback".into());
            }
            let trace = self.trace.clone();
            let index = u.rows.len();
            encoder.on_submitted_work_done(move || {
                if trace.done[index]
                    .compare_exchange(0, trace.now(), Ordering::AcqRel, Ordering::Acquire)
                    .is_err()
                {
                    trace.invalid.fetch_add(1, Ordering::Relaxed);
                }
            });
            let at = Instant::now();
            let command = encoder.finish();
            let finish_ns = at.elapsed().as_nanos() as u64;
            let begin = self.trace.now();
            let at = Instant::now();
            u.queue.submit([command]);
            let submit_ns = at.elapsed().as_nanos() as u64;
            u.rows.push([
                sample, repeat, create_ns, update_ns, finish_ns, begin, submit_ns,
            ]);
        }
        Ok(())
    }
    pub fn finish(&mut self) {
        if self.released {
            return;
        }
        self.trace.active.store(NONE, Ordering::Release);
        if let Some(u) = &mut self.upload {
            if let Err(e) = u.device.poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: Some(Duration::from_secs(3)),
            }) {
                self.error = Some(e.to_string());
            }
            u.renderer.take();
            if let Err(e) = u.device.poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: Some(Duration::from_secs(3)),
            }) {
                self.error = Some(e.to_string());
            }
            match crate::gpu_observation::snapshot(&u.device, "released_probe") {
                Ok(s) => u.snapshots.push(s),
                Err(e) => self.error = Some(e),
            }
        }
        self.released = true;
    }
    pub fn report(&self) -> String {
        let (samples, uploads, snapshots) = self.upload.as_ref().map_or_else(
            || (vec![], vec![], String::new()),
            |u| {
                let rows: Vec<[u64; 8]> = u
                    .rows
                    .iter()
                    .enumerate()
                    .map(|(i, r)| {
                        let mut v = [0; 8];
                        v[..7].copy_from_slice(r);
                        v[7] = self.trace.done[i].load(Ordering::Acquire);
                        v
                    })
                    .collect();
                (u.samples.clone(), rows, u.snapshots.join(","))
            },
        );
        format!(
            "{{\"schema\":1,\"mode\":\"{}\",\"invalid\":{},\"discarded\":{},\"error\":{},\"released\":{},\"frames\":{:?},\"samples\":{:?},\"uploads\":{:?},\"snapshots\":[{}]}}",
            if self.mode == Mode::Markers {
                "markers"
            } else {
                "upload"
            },
            self.trace.invalid.load(Ordering::Acquire),
            self.discarded,
            self.error
                .as_ref()
                .map_or_else(|| "null".into(), |e| format!("{e:?}")),
            self.released,
            self.trace.values(self.frames),
            samples,
            uploads,
            snapshots
        )
    }
}
impl egui::plugin::Plugin for Probe {
    fn debug_name(&self) -> &'static str {
        "explicit-gpu-path-observation"
    }
    fn output_hook(&mut self, ctx: &egui::Context, output: &mut egui::FullOutput) {
        self.trace.active.store(NONE, Ordering::Release);
        if self.stage != 4 || self.error.is_some() || self.released {
            return;
        }
        if ctx.will_discard() {
            self.discarded += 1;
            return;
        }
        let pass = ctx.cumulative_pass_nr();
        if self.last_pass.is_some_and(|p| p >= pass)
            || ctx.viewport_id() != egui::ViewportId::ROOT
            || !admit(&output.shapes)
        {
            self.error = Some("unordered, secondary or excessive output".into());
            return;
        }
        self.last_pass = Some(pass);
        self.seen += 1;
        if self.mode == Mode::Upload {
            if ORDINALS.contains(&self.seen)
                && let Err(e) = self.upload(ctx, output, pass)
            {
                self.error = Some(e);
            }
        } else if self.frames < FRAMES {
            let slot = self.frames;
            self.frames += 1;
            self.trace.set(slot, 0, pass + 1);
            self.trace.stamp(slot, 1);
            self.trace.active.store(slot, Ordering::Release);
            let rect = ctx.input(|i| i.viewport_rect());
            bracket(output, &self.trace, slot, rect);
        } else {
            self.error = Some("frame trace capacity".into());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounded_slots_and_duplicate_edges_fail_visibly() {
        let t = Trace::new();
        t.set(0, 0, 1);
        t.set(0, 1, 2);
        t.set(0, 1, 3);
        t.set(FRAMES, 1, 4);
        t.set(0, 8, 4);
        t.set(0, 2, 0);
        assert_eq!(t.invalid.load(Ordering::Relaxed), 4);
        assert_eq!(t.values(1)[0][1], 2);
    }
    #[test]
    fn late_completion_keeps_original_slot_and_owner() {
        let t = Arc::new(Trace::new());
        let weak = Arc::downgrade(&t);
        t.active.store(1, Ordering::Release);
        let callback = t.clone();
        t.active.store(2, Ordering::Release);
        callback.set(1, 7, 90);
        assert_eq!(t.values(3)[2][7], 0);
        assert_eq!(t.values(3)[1][7], 90);
        drop(t);
        assert!(weak.upgrade().is_some());
        drop(callback);
        assert!(weak.upgrade().is_none());
    }
    #[test]
    fn marker_bracket_preserves_original_shapes_textures_and_order() {
        let mut out = egui::FullOutput::default();
        let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(30., 20.));
        out.shapes.push(egui::epaint::ClippedShape {
            clip_rect: rect,
            shape: egui::Shape::Noop,
            transform_after_tessellation: Default::default(),
        });
        let original = out.shapes.clone();
        let t = Arc::new(Trace::new());
        let n = Arc::strong_count(&t);
        bracket(&mut out, &t, 0, rect);
        assert_eq!(out.shapes[1], original[0]);
        assert!(out.textures_delta.is_empty());
        assert_eq!(out.shapes.len(), 3);
        assert_eq!(Arc::strong_count(&t), n + 2);
        drop(out);
        assert_eq!(Arc::strong_count(&t), n);
    }
    #[test]
    fn inactive_and_released_probes_cannot_create_markers() {
        use egui::plugin::Plugin;
        let ctx = egui::Context::default();
        let mut out = egui::FullOutput::default();
        let mut p = Probe::new(Mode::Markers, None).unwrap();
        p.output_hook(&ctx, &mut out);
        assert!(out.shapes.is_empty());
        p.stage = 4;
        p.finish();
        p.output_hook(&ctx, &mut out);
        assert!(out.shapes.is_empty());
        assert!(p.released);
    }
    #[test]
    fn source_recursion_and_primitive_count_are_bounded() {
        let mut s = egui::Shape::Noop;
        for _ in 0..34 {
            s = egui::Shape::Vec(vec![s]);
        }
        let wrap = |shape| egui::epaint::ClippedShape {
            clip_rect: egui::Rect::EVERYTHING,
            shape,
            transform_after_tessellation: Default::default(),
        };
        assert!(!admit(&[wrap(s)]));
        assert!(!admit(&vec![wrap(egui::Shape::Noop); 2049]));
        assert!(admit(&vec![wrap(egui::Shape::Noop); 2048]));
    }
}
