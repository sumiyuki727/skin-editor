use crate::upload::ReusableUpload;
use crate::{Images, Op, Plan, RenderError, SourceUpload, StagingUsage, error};
use std::sync::{
    Arc, Mutex, OnceLock, Weak,
    atomic::{AtomicBool, AtomicU64, Ordering},
};
use std::time::Instant;
use wgpu::util::DeviceExt;
#[derive(Clone, Copy, Debug, Default)]
pub struct GpuPreparation {
    pub source_lock_ms: f64,
    pub submission_lock_ms: f64,
    pub staging_wait_ms: f64,
    pub staging_alloc_ms: f64,
    pub staging_copy_ms: f64,
    pub write_ms: f64,
    pub flush_ms: f64,
    pub create_ms: f64,
    pub pack_ms: f64,
    pub upload_ms: f64,
    pub encode_submit_ms: f64,
    pub reused: bool,
}
pub struct GpuImage {
    pub texture: wgpu::Texture,
    pub ready: Arc<AtomicBool>,
    /// Host-clock observation of GPU completion notification, before ready publication.
    pub completed: Arc<OnceLock<Instant>>,
    pub size: [u32; 2],
    pub preparation: GpuPreparation,
}
pub struct GpuComposer {
    upload: SourceUpload,
    staging: Mutex<ReusableUpload>,
    submission: Mutex<()>,
    pipeline: wgpu::ComputePipeline,
    source: Mutex<Option<CachedSource>>,
    uploads: AtomicU64,
    allocations: AtomicU64,
}
type PixelOffsets = std::collections::BTreeMap<skin_editor_domain::BlobRef, u32>;
type SourceIdentity = Vec<(skin_editor_domain::BlobRef, Weak<crate::Raster>)>;
struct CachedSource {
    identity: SourceIdentity,
    buffer: wgpu::Buffer,
    offsets: PixelOffsets,
    bytes: usize,
}
fn same_sources(identity: &SourceIdentity, plan: &Plan, images: &Images) -> bool {
    let sources = plan.sources();
    identity.len() == sources.len()
        && identity.iter().zip(sources).all(|((blob, weak), key)| {
            *blob == key
                && images
                    .get(blob)
                    .is_some_and(|r| weak.upgrade().is_some_and(|old| Arc::ptr_eq(&old, r)))
        })
}
fn pack_pixels(
    plan: &Plan,
    images: &Images,
    limit: usize,
) -> Result<(Vec<u8>, PixelOffsets), RenderError> {
    let sources = plan.sources();
    let mut total = 0usize;
    for blob in &sources {
        let image = images
            .get(blob)
            .ok_or_else(|| error("missing decoded source"))?;
        if image.pixels.len() as u64
            != u64::from(image.size.width()) * u64::from(image.size.height()) * 4
        {
            return Err(error("invalid RGBA length"));
        }
        total = total
            .checked_add(image.pixels.len())
            .filter(|n| *n <= limit)
            .ok_or_else(|| error("GPU source budget exceeded"))?;
    }
    let capacity = total.max(4);
    if capacity > limit {
        return Err(error("GPU source budget exceeded"));
    }
    // Validate before allocating; geometric Vec growth otherwise reserves
    // 64 MiB for the standard 40 MiB source set and retains unnecessary pages.
    let mut pixels = Vec::new();
    pixels
        .try_reserve_exact(capacity)
        .map_err(|_| error("GPU source staging allocation failed"))?;
    let mut offsets = PixelOffsets::new();
    for blob in sources {
        offsets.insert(blob, (pixels.len() / 4) as u32);
        pixels.extend_from_slice(&images[&blob].pixels);
    }
    if pixels.is_empty() {
        pixels.resize(4, 0);
    }
    Ok((pixels, offsets))
}
impl GpuComposer {
    pub fn new(device: &wgpu::Device) -> Self {
        Self::with_upload(device, SourceUpload::QueueWrite)
    }
    pub fn with_upload(device: &wgpu::Device, upload: SourceUpload) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shared composition"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("composition"),
            layout: None,
            module: &shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });
        Self {
            upload,
            staging: Mutex::new(ReusableUpload::default()),
            submission: Mutex::new(()),
            pipeline,
            source: Mutex::new(None),
            uploads: AtomicU64::new(0),
            allocations: AtomicU64::new(0),
        }
    }
    pub fn staging_usage(&self) -> StagingUsage {
        self.staging.lock().expect("staging usage").usage()
    }
    pub fn clear_sources(&self) {
        if let Ok(mut source) = self.source.lock() {
            *source = None;
        }
        if let Ok(mut staging) = self.staging.lock() {
            staging.clear();
        }
    }
    pub fn source_usage(&self) -> (u64, usize) {
        (
            self.uploads.load(Ordering::Relaxed),
            self.source
                .lock()
                .ok()
                .and_then(|s| s.as_ref().map(|s| s.bytes))
                .unwrap_or(0),
        )
    }
    pub fn source_allocations(&self) -> u64 {
        self.allocations.load(Ordering::Relaxed)
    }
    fn source_buffer(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        plan: &Plan,
        images: &Images,
    ) -> Result<(wgpu::Buffer, PixelOffsets, GpuPreparation), RenderError> {
        let locked = Instant::now();
        let mut slot = self
            .source
            .lock()
            .map_err(|_| error("source cache unavailable"))?;
        let source_lock_ms = locked.elapsed().as_secs_f64() * 1000.;
        if let Some(source) = slot
            .as_ref()
            .filter(|s| same_sources(&s.identity, plan, images))
        {
            return Ok((
                source.buffer.clone(),
                source.offsets.clone(),
                GpuPreparation {
                    source_lock_ms,
                    reused: true,
                    ..Default::default()
                },
            ));
        }
        // The cache owns one source set. Submission closures separately retain
        // in-flight buffers; weak raster identities never pin CPU pixel leases.
        let previous = slot.take();
        let started = Instant::now();
        let (pixels, offsets) = pack_pixels(
            plan,
            images,
            (80 * 1024 * 1024).min(device.limits().max_storage_buffer_binding_size as usize),
        )?;
        let pack_ms = started.elapsed().as_secs_f64() * 1000.;
        let started = Instant::now();
        let (mut write_ms, mut flush_ms, mut create_ms) = (0., 0., 0.);
        let mut staging_timing = crate::upload::UploadTiming::default();
        let buffer = if self.upload == SourceUpload::ReusableStaging {
            let buffer = if let Some(previous) =
                previous.filter(|s| s.buffer.size() >= pixels.len() as u64)
            {
                previous.buffer
            } else {
                let stamp = Instant::now();
                let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("source pixels"),
                    size: pixels.len() as u64,
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
                create_ms = stamp.elapsed().as_secs_f64() * 1000.;
                self.allocations.fetch_add(1, Ordering::Relaxed);
                buffer
            };
            staging_timing = self
                .staging
                .lock()
                .map_err(|_| error("staging unavailable"))?
                .write(device, queue, &buffer, &pixels)?;
            buffer
        } else {
            if let Some(previous) = previous.filter(|s| s.buffer.size() >= pixels.len() as u64) {
                // All uses and writes are serialized through submission below.
                // Queue writes precede this render and follow earlier submitted
                // computations. Published output textures never change in place.
                let stamp = Instant::now();
                queue.write_buffer(&previous.buffer, 0, &pixels);
                write_ms = stamp.elapsed().as_secs_f64() * 1000.;
                // Flush even if later operation encoding returns a diagnostic;
                // a failed candidate must not leave a staging write pending.
                let stamp = Instant::now();
                queue.submit([]);
                flush_ms = stamp.elapsed().as_secs_f64() * 1000.;
                previous.buffer
            } else {
                self.allocations.fetch_add(1, Ordering::Relaxed);
                let stamp = Instant::now();
                let created = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("source pixels"),
                    contents: &pixels,
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                });
                create_ms = stamp.elapsed().as_secs_f64() * 1000.;
                created
            }
        };
        let identity = plan
            .sources()
            .into_iter()
            .map(|blob| (blob, Arc::downgrade(&images[&blob])))
            .collect();
        *slot = Some(CachedSource {
            identity,
            buffer: buffer.clone(),
            offsets: offsets.clone(),
            bytes: buffer.size() as usize,
        });
        self.uploads.fetch_add(1, Ordering::Relaxed);
        Ok((
            buffer,
            offsets,
            GpuPreparation {
                source_lock_ms,
                staging_wait_ms: staging_timing.wait_ms,
                staging_alloc_ms: staging_timing.allocation_ms,
                staging_copy_ms: staging_timing.copy_submit_ms,
                write_ms,
                flush_ms,
                create_ms,
                pack_ms,
                upload_ms: started.elapsed().as_secs_f64() * 1000.,
                ..Default::default()
            },
        ))
    }
    pub fn render(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        plan: &Plan,
        images: &Images,
        background: bool,
        wake: impl FnOnce() + Send + 'static,
    ) -> Result<GpuImage, RenderError> {
        let locked = Instant::now();
        let _submission = self
            .submission
            .lock()
            .map_err(|_| error("GPU submission unavailable"))?;
        let submission_lock_ms = locked.elapsed().as_secs_f64() * 1000.;
        let (source, offsets, mut preparation) = self.source_buffer(device, queue, plan, images)?;
        preparation.submission_lock_ms = submission_lock_ms;
        let started = Instant::now();
        let mut commands = Vec::new();
        fn floats(
            bytes: &mut Vec<u8>,
            values: impl IntoIterator<Item = f64>,
        ) -> Result<(), RenderError> {
            for value in values {
                let f = value as f32;
                if !f.is_finite() {
                    return Err(error("GPU coordinate range exceeded"));
                }
                bytes.extend_from_slice(&f.to_le_bytes());
            }
            Ok(())
        }
        for op in &plan.ops {
            let (kind, offset, size, inverse, opacity, clip, canvas) = match op {
                Op::Push(alpha) => (0, 0, [0, 0], [0.0; 6], *alpha, [0.0; 6], [0, 0]),
                Op::Pop => (2, 0, [0, 0], [0.0; 6], 0.0, [0.0; 6], [0, 0]),
                Op::Image {
                    blob,
                    inverse,
                    opacity,
                    clip_inverse,
                    canvas,
                } => (
                    1,
                    offsets[blob],
                    [images[blob].size.width(), images[blob].size.height()],
                    inverse.coefficients(),
                    *opacity,
                    clip_inverse.coefficients(),
                    [canvas.width(), canvas.height()],
                ),
            };
            for v in [kind, offset, size[0], size[1]] {
                commands.extend_from_slice(&v.to_le_bytes());
            }
            floats(&mut commands, inverse[..4].iter().copied())?;
            floats(
                &mut commands,
                [inverse[4], inverse[5], f64::from(opacity), 0.0],
            )?;
            floats(&mut commands, clip[..4].iter().copied())?;
            floats(
                &mut commands,
                [clip[4], clip[5], f64::from(canvas[0]), f64::from(canvas[1])],
            )?;
        }
        if commands.is_empty() {
            commands.resize(80, 0);
        }
        if commands.len() as u64 > device.limits().max_storage_buffer_binding_size {
            return Err(error("GPU operation buffer budget exceeded"));
        }
        let mut params = Vec::new();
        for value in [
            plan.size.width(),
            plan.size.height(),
            plan.ops.len() as u32,
            u32::from(background),
        ] {
            params.extend_from_slice(&value.to_le_bytes());
        }
        let buffer = |label, data: &[u8], usage| {
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(label),
                contents: data,
                usage,
            })
        };
        let ops = buffer("composition ops", &commands, wgpu::BufferUsages::STORAGE);
        let params = buffer("composition params", &params, wgpu::BufferUsages::UNIFORM);
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("candidate preview"),
            size: wgpu::Extent3d {
                width: plan.size.width(),
                height: plan.size.height(),
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
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: source.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: ops.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: params.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
            ],
        });
        let ready = Arc::new(AtomicBool::new(false));
        let done = ready.clone();
        let completed = Arc::new(OnceLock::new());
        let stamp = completed.clone();
        let texture_lease = texture.clone();
        let mut encoder = device.create_command_encoder(&Default::default());
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: None,
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &group, &[]);
            pass.dispatch_workgroups(
                plan.size.width().div_ceil(8),
                plan.size.height().div_ceil(8),
                1,
            );
        }
        let command = encoder.finish();
        command.on_submitted_work_done(move || {
            drop((source, ops, params, group, texture_lease));
            let _ = stamp.set(Instant::now());
            done.store(true, Ordering::Release);
            wake();
        });
        queue.submit([command]);
        preparation.encode_submit_ms = started.elapsed().as_secs_f64() * 1000.;
        Ok(GpuImage {
            texture,
            ready,
            completed,
            size: [plan.size.width(), plan.size.height()],
            preparation,
        })
    }
}
const SHADER: &str = r#"
struct Op { info:vec4<u32>, inverse0:vec4<f32>, inverse1:vec4<f32>, clip0:vec4<f32>, clip1:vec4<f32> }
struct Params { size:vec2<u32>, count:u32, background:u32 }
@group(0) @binding(0) var<storage,read> pixels:array<u32>;
@group(0) @binding(1) var<storage,read> ops:array<Op>;
@group(0) @binding(2) var<uniform> params:Params;
@group(0) @binding(3) var output:texture_storage_2d<rgba8unorm,write>;
fn point(a:vec4<f32>,b:vec4<f32>,p:vec2<f32>)->vec2<f32> {return vec2(a.x*p.x+a.z*p.y+b.x,a.y*p.x+a.w*p.y+b.y);}
fn linear(c:vec3<f32>)->vec3<f32> {return select(pow((c+vec3(0.055))/1.055,vec3(2.4)),c/12.92,c<=vec3(0.04045));}
fn gamma(c:vec3<f32>)->vec3<f32> {return select(1.055*pow(c,vec3(1.0/2.4))-vec3(0.055),c*12.92,c<=vec3(0.0031308));}
fn over(back:vec4<f32>,front:vec4<f32>)->vec4<f32> {return front+back*(1.0-front.a);}
@compute @workgroup_size(8,8) fn main(@builtin(global_invocation_id) id:vec3<u32>) {
 if any(id.xy>=params.size) {return;}
 let p=vec2<f32>(id.xy)+vec2(0.5);var color=vec4<f32>(0.0);var stack:array<vec4<f32>,32>;var weights:array<f32,32>;var depth=0u;
 for(var i=0u;i<params.count;i+=1u) {
  let op=ops[i];
  if op.info.x==0u {stack[depth]=color;weights[depth]=op.inverse1.z;depth+=1u;color=vec4(0.0);}
  else if op.info.x==2u {depth-=1u;color=over(stack[depth],color*weights[depth]);}
  else {
   let clip=point(op.clip0,op.clip1,p);let at=point(op.inverse0,op.inverse1,p);
   if all(clip>=vec2(0.0)) && all(clip<op.clip1.zw) && all(at>=vec2(0.0)) && all(at<vec2<f32>(op.info.zw)) {
    let pos=vec2<u32>(floor(at));let rgba=pixels[op.info.y+pos.y*op.info.z+pos.x];
    let value=vec4<f32>(f32(rgba&255u),f32((rgba>>8u)&255u),f32((rgba>>16u)&255u),f32(rgba>>24u))/255.0;
    let alpha=value.a*op.inverse1.z;color=over(color,vec4(linear(value.rgb)*alpha,alpha));
   }
  }
 }
 if params.background!=0u {let grey=select(0.055,0.035,((id.x/16u+id.y/16u)%2u)==0u);color=over(vec4(vec3(grey),1.0),color);}
 var rgb=vec3<f32>(0.0);if color.a>0.0 {rgb=gamma(clamp(color.rgb/color.a,vec3(0.0),vec3(1.0)));}
 textureStore(output,vec2<i32>(id.xy),vec4(rgb,color.a));
}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use skin_editor_domain::{Affine2, BlobRef, PixelSize};
    #[test]
    fn gpu_source_identity_is_weak_and_cannot_confuse_equal_blob_labels() {
        let blob = BlobRef::new([1; 32], 1);
        let raster = Arc::new(crate::Raster {
            size: PixelSize::new(1, 1).unwrap(),
            pixels: vec![1; 4].into(),
        });
        let identity = vec![(blob, Arc::downgrade(&raster))];
        let mut images = Images::from([(blob, raster.clone())]);
        let plan = Plan {
            size: raster.size,
            ops: vec![Op::Image {
                blob,
                inverse: Affine2::IDENTITY,
                opacity: 1.,
                clip_inverse: Affine2::IDENTITY,
                canvas: raster.size,
            }],
            view: Affine2::IDENTITY,
        };
        assert!(same_sources(&identity, &plan, &images));
        images.insert(blob, Arc::new((*raster).clone()));
        assert!(!same_sources(&identity, &plan, &images));
        drop(raster);
        assert!(identity[0].1.upgrade().is_none());
    }
    #[test]
    fn repeated_sources_are_packed_once_and_over_budget_inputs_are_rejected() {
        let blob = BlobRef::new([1; 32], 1);
        let op = Op::Image {
            blob,
            inverse: Affine2::IDENTITY,
            opacity: 1.,
            clip_inverse: Affine2::IDENTITY,
            canvas: PixelSize::new(2, 2).unwrap(),
        };
        let plan = Plan {
            size: PixelSize::new(2, 2).unwrap(),
            ops: vec![op.clone(), op],
            view: Affine2::IDENTITY,
        };
        let images = Images::from([(
            blob,
            Arc::new(crate::Raster {
                size: plan.size,
                pixels: vec![7; 16].into(),
            }),
        )]);
        let (packed, offsets) = pack_pixels(&plan, &images, 16).unwrap();
        assert_eq!(packed, vec![7; 16]);
        assert_eq!(offsets.len(), 1);
        assert_eq!(offsets[&blob], 0);
        assert!(pack_pixels(&plan, &images, 15).is_err());
        assert!(pack_pixels(&plan, &Images::new(), 16).is_err());
    }
    #[test]
    fn empty_and_malformed_rasters_keep_explicit_upload_bounds() {
        let mut plan = Plan {
            size: PixelSize::new(1, 1).unwrap(),
            ops: vec![],
            view: Affine2::IDENTITY,
        };
        assert_eq!(pack_pixels(&plan, &Images::new(), 4).unwrap().0, vec![0; 4]);
        assert!(pack_pixels(&plan, &Images::new(), 3).is_err());
        let blob = BlobRef::new([1; 32], 1);
        plan.ops.push(Op::Image {
            blob,
            inverse: Affine2::IDENTITY,
            opacity: 1.,
            clip_inverse: Affine2::IDENTITY,
            canvas: plan.size,
        });
        let images = Images::from([(
            blob,
            Arc::new(crate::Raster {
                size: plan.size,
                pixels: vec![0; 3].into(),
            }),
        )]);
        assert!(pack_pixels(&plan, &images, 16).is_err());
    }
}
