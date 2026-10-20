//! GPU adapter for the P0 plan. Source buffers stay resident between dispatches.
use super::composition::{Image, Op, Plan, Sampling};
use eframe::wgpu;
use std::{
    sync::{Arc, mpsc},
    time::{Duration, Instant},
};
use wgpu::util::DeviceExt;

pub struct GpuPlan {
    group: wgpu::BindGroup,
    source: wgpu::Buffer,
    ops: wgpu::Buffer,
    assets: Arc<[Image]>,
    offsets: Vec<u32>,
    op_count: usize,
    output: wgpu::Buffer,
    read: wgpu::Buffer,
    queries: Option<(wgpu::QuerySet, wgpu::Buffer, wgpu::Buffer)>,
    size: [u32; 2],
    pub resident_buffer_bytes: u64,
}
pub struct Frame {
    pub image: Image,
    pub gpu_ms: Option<f64>,
    pub submit_wait_readback_ms: f64,
}
fn encode_ops(plan: &Plan<'_>, offsets: &[u32]) -> Vec<u8> {
    let mut commands = Vec::new();
    for op in plan.ops() {
        let mut meta = [0u32; 4];
        let mut x = [0f32; 4];
        let mut y = [0f32; 4];
        match op {
            Op::Begin => meta[0] = 1,
            Op::End(alpha) => {
                meta[0] = 2;
                x[3] = *alpha as f32;
            }
            Op::Draw {
                asset,
                inverse,
                opacity,
                sampling,
            } => {
                meta = [
                    if matches!(sampling, Sampling::Nearest) {
                        0
                    } else {
                        3
                    },
                    offsets[*asset],
                    plan.assets()[*asset].size[0],
                    plan.assets()[*asset].size[1],
                ];
                let [a, b, c, d, u, v] = inverse.0;
                x = [a as f32, c as f32, u as f32, *opacity as f32];
                y = [b as f32, d as f32, v as f32, 0.];
            }
        }
        commands.extend(meta.iter().flat_map(|v| v.to_le_bytes()));
        commands.extend(
            x.iter()
                .chain(y.iter())
                .chain([0f32; 4].iter())
                .flat_map(|v| v.to_le_bytes()),
        );
    }
    commands.resize(commands.len().max(64), 0);
    commands
}
fn check_update(
    assets: &Arc<[Image]>,
    size: [u32; 2],
    op_count: usize,
    plan: &Plan<'_>,
) -> Result<(), String> {
    if !std::ptr::eq(assets.as_ref(), plan.assets())
        || size != plan.size()
        || op_count != plan.ops().len()
    {
        return Err(
            "Update must preserve the exact immutable source set, canvas and operation count"
                .into(),
        );
    }
    Ok(())
}
/// Cannot dispatch or expose a partially uploaded source set.
pub struct PreparedPlan {
    job: GpuPlan,
    next: usize,
}
impl PreparedPlan {
    pub fn upload(
        &mut self,
        queue: &wgpu::Queue,
        budget: &mut crate::loading::UploadBudget,
    ) -> usize {
        let mut uploaded = 0;
        while let Some(image) = self.job.assets.get(self.next) {
            let bytes = image.rgba.len() * 4;
            if !budget.take(bytes) {
                break;
            }
            let pixels: Vec<_> = image.rgba.iter().flatten().copied().collect();
            queue.write_buffer(
                &self.job.source,
                u64::from(self.job.offsets[self.next]) * 4,
                &pixels,
            );
            self.next += 1;
            uploaded += bytes;
        }
        uploaded
    }
    pub fn complete(&self) -> bool {
        self.next == self.job.assets.len()
    }
    pub fn finish(self) -> Result<GpuPlan, String> {
        if !self.complete() {
            return Err("Source upload incomplete".into());
        }
        Ok(self.job)
    }
}
impl GpuPlan {
    pub fn new(
        device: &wgpu::Device,
        pipeline: &wgpu::ComputePipeline,
        assets: Arc<[Image]>,
        plan: &Plan<'_>,
    ) -> Result<Self, String> {
        Self::build(device, pipeline, assets, plan, false)
    }
    pub fn prepare(
        device: &wgpu::Device,
        pipeline: &wgpu::ComputePipeline,
        assets: Arc<[Image]>,
        plan: &Plan<'_>,
    ) -> Result<PreparedPlan, String> {
        Ok(PreparedPlan {
            job: Self::build(device, pipeline, assets, plan, true)?,
            next: 0,
        })
    }
    fn build(
        device: &wgpu::Device,
        pipeline: &wgpu::ComputePipeline,
        assets: Arc<[Image]>,
        plan: &Plan<'_>,
        deferred: bool,
    ) -> Result<Self, String> {
        check_update(&assets, plan.size(), plan.ops().len(), plan)?;
        let mut pixels = Vec::new();
        let mut offsets = Vec::new();
        let mut source_bytes = 0;
        for image in plan.assets() {
            offsets.push(source_bytes as u32 / 4);
            source_bytes += image.rgba.len() * 4;
            if !deferred {
                pixels.extend(image.rgba.iter().flatten().copied());
            }
        }
        source_bytes = source_bytes.max(4);
        if !deferred {
            pixels.resize(source_bytes, 0);
        }
        let commands = encode_ops(plan, &offsets);
        let size = plan.size();
        let parameters: Vec<u8> = [size[0], size[1], plan.ops().len() as u32, 0]
            .iter()
            .flat_map(|v| v.to_le_bytes())
            .collect();
        let buffer = |label, contents: &[u8], usage| {
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(label),
                contents,
                usage,
            })
        };
        if source_bytes as u64 > device.limits().max_storage_buffer_binding_size
            || commands.len() as u64 > device.limits().max_storage_buffer_binding_size
        {
            return Err("GPU storage binding budget exceeded".into());
        }
        let ops = buffer(
            "composition-ops",
            &commands,
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        );
        let source = if deferred {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("composition-deferred-source"),
                size: source_bytes as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        } else {
            buffer("composition-source", &pixels, wgpu::BufferUsages::STORAGE)
        };
        let params = buffer("composition-size", &parameters, wgpu::BufferUsages::UNIFORM);
        let bytes = u64::from(size[0]) * u64::from(size[1]) * 4;
        let output = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("composition-output"),
            size: bytes,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let read = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("composition-readback"),
            size: bytes,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("composition"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: ops.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: source.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: output.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: params.as_entire_binding(),
                },
            ],
        });
        let queries = device
            .features()
            .contains(wgpu::Features::TIMESTAMP_QUERY)
            .then(|| {
                let queries = device.create_query_set(&wgpu::QuerySetDescriptor {
                    label: Some("composition-timestamps"),
                    ty: wgpu::QueryType::Timestamp,
                    count: 2,
                });
                let resolve = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("composition-query-resolve"),
                    size: 16,
                    usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
                    mapped_at_creation: false,
                });
                let read = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("composition-query-read"),
                    size: 16,
                    usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
                (queries, resolve, read)
            });
        let resident_buffer_bytes = source_bytes as u64
            + commands.len() as u64
            + 16
            + bytes * 2
            + if queries.is_some() { 32 } else { 0 };
        Ok(Self {
            source,
            ops,
            assets,
            offsets,
            op_count: plan.ops().len(),
            group,
            output,
            read,
            queries,
            size,
            resident_buffer_bytes,
        })
    }
    pub fn update(&self, queue: &wgpu::Queue, plan: &Plan<'_>) -> Result<usize, String> {
        check_update(&self.assets, self.size, self.op_count, plan)?;
        let commands = encode_ops(plan, &self.offsets);
        queue.write_buffer(&self.ops, 0, &commands);
        Ok(commands.len())
    }
    pub fn present(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        pipeline: &wgpu::ComputePipeline,
        bridge: &wgpu::ComputePipeline,
        texture: &wgpu::TextureView,
    ) {
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("composition-presentation"),
            layout: &bridge.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.output.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(texture),
                },
            ],
        });
        let mut encoder = device.create_command_encoder(&Default::default());
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("live-composition"),
                timestamp_writes: None,
            });
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, Some(&self.group), &[]);
            pass.dispatch_workgroups(self.size[0].div_ceil(8), self.size[1].div_ceil(8), 1);
        }
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("opaque-editor-presentation"),
                timestamp_writes: None,
            });
            pass.set_pipeline(bridge);
            pass.set_bind_group(0, Some(&group), &[]);
            pass.dispatch_workgroups(self.size[0].div_ceil(8), self.size[1].div_ceil(8), 1);
        }
        queue.submit([encoder.finish()]);
    }
    pub fn run(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        pipeline: &wgpu::ComputePipeline,
    ) -> Result<Frame, String> {
        let start = Instant::now();
        let size = self.size;
        let bytes = u64::from(size[0]) * u64::from(size[1]) * 4;
        let mut encoder = device.create_command_encoder(&Default::default());
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("composition"),
                timestamp_writes: self.queries.as_ref().map(|(query_set, _, _)| {
                    wgpu::ComputePassTimestampWrites {
                        query_set,
                        beginning_of_pass_write_index: Some(0),
                        end_of_pass_write_index: Some(1),
                    }
                }),
            });
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, Some(&self.group), &[]);
            pass.dispatch_workgroups(size[0].div_ceil(8), size[1].div_ceil(8), 1);
        }
        encoder.copy_buffer_to_buffer(&self.output, 0, &self.read, 0, bytes);
        if let Some((queries, resolve, read)) = &self.queries {
            encoder.resolve_query_set(queries, 0..2, resolve, 0);
            encoder.copy_buffer_to_buffer(resolve, 0, read, 0, 16);
        }
        let submission = queue.submit([encoder.finish()]);
        let (tx, rx) = mpsc::channel();
        self.read
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |r| {
                let _ = tx.send(r);
            });
        let (query_tx, query_rx) = mpsc::channel();
        if let Some((_, _, read)) = &self.queries {
            read.slice(..).map_async(wgpu::MapMode::Read, move |r| {
                let _ = query_tx.send(r);
            });
        }
        device
            .poll(wgpu::PollType::Wait {
                submission_index: Some(submission),
                timeout: Some(Duration::from_secs(30)),
            })
            .map_err(|e| format!("{e:?}"))?;
        rx.recv_timeout(Duration::from_secs(1))
            .map_err(|e| e.to_string())?
            .map_err(|e| e.to_string())?;
        let view = self
            .read
            .slice(..)
            .get_mapped_range()
            .map_err(|e| e.to_string())?;
        let rgba = view
            .chunks_exact(4)
            .map(|c| [c[0], c[1], c[2], c[3]])
            .collect();
        drop(view);
        self.read.unmap();
        let gpu_ms = if let Some((_, _, read)) = &self.queries {
            query_rx
                .recv_timeout(Duration::from_secs(1))
                .map_err(|e| e.to_string())?
                .map_err(|e| e.to_string())?;
            let view = read
                .slice(..)
                .get_mapped_range()
                .map_err(|e| e.to_string())?;
            let a = u64::from_le_bytes(view[0..8].try_into().unwrap());
            let b = u64::from_le_bytes(view[8..16].try_into().unwrap());
            drop(view);
            read.unmap();
            Some(
                b.checked_sub(a).ok_or("GPU timestamp moved backward")? as f64
                    * f64::from(queue.get_timestamp_period())
                    / 1e6,
            )
        } else {
            None
        };
        Ok(Frame {
            image: Image { size, rgba },
            gpu_ms,
            submit_wait_readback_ms: start.elapsed().as_secs_f64() * 1000.,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::composition::{Affine, Node, Sampling, compile};
    #[test]
    fn updates_require_the_same_owned_sources_and_shape() {
        let sources: Arc<[Image]> = vec![Image {
            size: [1, 1],
            rgba: vec![[255; 4]],
        }]
        .into();
        let draw = Node::Layer {
            asset: 0,
            transform: Affine::IDENTITY,
            opacity: 1.,
            sampling: Sampling::Nearest,
        };
        let plan = compile([2, 2], &sources, std::slice::from_ref(&draw)).unwrap();
        assert!(check_update(&sources, [2, 2], 1, &plan).is_ok());
        let other: Arc<[Image]> = sources.to_vec().into();
        assert!(check_update(&other, [2, 2], 1, &plan).is_err());
        assert!(check_update(&sources, [1, 2], 1, &plan).is_err());
        assert!(check_update(&sources, [2, 2], 2, &plan).is_err());
        let bytes = encode_ops(&plan, &[0]);
        assert_eq!(bytes.len(), 64);
        let moved = Node::Layer {
            asset: 0,
            transform: Affine([1., 0., 0., 1., 2., 3.]),
            opacity: 1.,
            sampling: Sampling::Nearest,
        };
        let changed = compile([2, 2], &sources, &[moved]).unwrap();
        assert!(check_update(&sources, [2, 2], 1, &changed).is_ok());
        assert_ne!(encode_ops(&changed, &[0]), bytes);
    }
}
