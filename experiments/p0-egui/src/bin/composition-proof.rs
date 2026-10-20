//! Headless GPU differential test for the bounded P0 composition semantics.
use eframe::wgpu;
use p0_egui::composition::{
    self, Affine, Image, Limits, Node, Sampling, compile, compile_with_limits, render,
};
use p0_egui::composition_fixture::{remap, standard_nodes};
use p0_egui::composition_gpu::GpuPlan;
use serde_json::json;
use std::{path::PathBuf, time::Instant};

fn layer(asset: usize, transform: Affine, opacity: f64, sampling: Sampling) -> Node {
    Node::Layer {
        asset,
        transform,
        opacity,
        sampling,
    }
}
fn group(children: Vec<Node>, transform: Affine, opacity: f64) -> Node {
    Node::Group {
        children,
        transform,
        opacity,
    }
}
fn fixtures() -> (Vec<Image>, Vec<(&'static str, Vec<Node>)>) {
    let mut assets = Vec::new();
    for (width, height, seed) in [(7, 11, 1), (16, 5, 3), (3, 9, 5)] {
        let rgba = (0..height)
            .flat_map(|y| {
                (0..width).map(move |x| {
                    [
                        (x * 33 + seed * 21) as u8,
                        (y * 47 + seed * 13) as u8,
                        ((x + y) * 19) as u8,
                        if (x + y) % 5 == 0 {
                            0
                        } else {
                            ((x * 17 + y * 23) % 255) as u8
                        },
                    ]
                })
            })
            .collect();
        assets.push(Image {
            size: [width, height],
            rgba,
        });
    }
    assets.push(Image {
        size: [1, 1],
        rgba: vec![[255, 0, 0, 255]],
    });
    assets.push(Image {
        size: [1, 1],
        rgba: vec![[0, 0, 255, 255]],
    });
    let id = Affine::IDENTITY;
    let near = Sampling::Nearest;
    let linear = Sampling::Bilinear;
    let cases = vec![
        ("empty", vec![]),
        ("identity", vec![layer(0, id, 1., near)]),
        (
            "affine",
            vec![layer(
                0,
                Affine([1.7, 0.4, -0.3, 1.2, 9.3, 7.1]),
                0.73,
                linear,
            )],
        ),
        (
            "reflection",
            vec![layer(
                1,
                Affine([-1.4, 0.3, 0.2, 2.1, 25.2, 3.4]),
                0.83,
                near,
            )],
        ),
        (
            "isolated-overlap",
            vec![group(
                vec![
                    layer(3, Affine([15., 0., 0., 15., 2., 2.]), 1., near),
                    layer(4, Affine([15., 0., 0., 15., 2., 2.]), 1., near),
                ],
                id,
                0.5,
            )],
        ),
        (
            "nested",
            vec![
                layer(1, Affine([2., 0., 0., 3., 0., 0.]), 0.6, linear),
                group(
                    vec![
                        layer(0, id, 0.8, linear),
                        group(
                            vec![layer(2, Affine([1., 0.5, 0., 1., 2., 3.]), 1., linear)],
                            Affine([0.8, 0.6, -0.6, 0.8, 8., 2.]),
                            0.5,
                        ),
                    ],
                    Affine([1.3, 0.2, 0.1, 1.7, 3.2, 5.7]),
                    0.65,
                ),
            ],
        ),
        (
            "outside-clip",
            vec![layer(0, Affine([1.3, 0., 0., 1.1, -4.7, -3.3]), 1., linear)],
        ),
        (
            "zero-opacity",
            vec![group(vec![layer(0, id, 1., linear)], id, 0.)],
        ),
        (
            "boundary-snap",
            vec![layer(3, Affine([1., 0., 0., 1., 0.500001, 0.]), 1., near)],
        ),
    ];
    (assets, cases)
}
fn save(path: PathBuf, image: &Image) -> Result<(), String> {
    let bytes: Vec<u8> = image.rgba.iter().flatten().copied().collect();
    image::save_buffer(
        &path,
        &bytes,
        image.size[0],
        image.size[1],
        image::ColorType::Rgba8,
    )
    .map_err(|e| e.to_string())?;
    let restored = image::open(&path).map_err(|e| e.to_string())?.into_rgba8();
    if restored.as_raw() != &bytes {
        return Err("PNG roundtrip changed pixels".into());
    }
    Ok(())
}
fn scale(
    fixture: PathBuf,
    output: &std::path::Path,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    pipeline: &wgpu::ComputePipeline,
) -> Result<serde_json::Value, String> {
    let fixture = fixture.canonicalize().map_err(|e| e.to_string())?;
    let bytes = std::fs::read(fixture.join("workload.json")).map_err(|e| e.to_string())?;
    if bytes.len() > 2 * 1024 * 1024 {
        return Err("Fixture manifest too large".into());
    }
    let manifest: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    let sources = manifest["assets"].as_array().ok_or("Missing assets")?;
    let documents = manifest["documents"]
        .as_array()
        .ok_or("Missing documents")?;
    if manifest["schema_version"] != 1 || sources.len() != 100 || documents.len() != 10 {
        return Err("Expected standard P0 fixture v1: 100 sources, 10 documents".into());
    }
    let mut results = Vec::new();
    let mut passed = true;
    for (document, value) in documents.iter().enumerate() {
        let mut ids = std::collections::BTreeSet::new();
        let mut nodes = standard_nodes(&value["children"], &mut ids, 0, &mut 0)?;
        let ids: Vec<_> = ids.into_iter().collect();
        if ids.len() != 10 {
            return Err("Expected 10 active sources per document".into());
        }
        let start = Instant::now();
        let mut assets = Vec::new();
        for &id in &ids {
            if sources[id]["id"].as_u64() != Some(id as u64) {
                return Err("Fixture asset identity mismatch".into());
            }
            let path = fixture
                .join(sources[id]["path"].as_str().ok_or("Missing source path")?)
                .canonicalize()
                .map_err(|e| e.to_string())?;
            if !path.starts_with(&fixture)
                || image::image_dimensions(&path).map_err(|e| e.to_string())? != (1024, 1024)
            {
                return Err("Source outside fixture or incorrect dimensions".into());
            }
            let png = image::open(path).map_err(|e| e.to_string())?.into_rgba8();
            assets.push(Image {
                size: [1024, 1024],
                rgba: png
                    .as_raw()
                    .chunks_exact(4)
                    .map(|p| [p[0], p[1], p[2], p[3]])
                    .collect(),
            });
        }
        let decode_ms = start.elapsed().as_secs_f64() * 1000.;
        remap(&mut nodes, &ids);
        let assets: std::sync::Arc<[Image]> = assets.into();
        let plan = compile_with_limits([1024, 1024], &assets, &nodes, Limits::STANDARD)?;
        let layers = plan
            .ops()
            .iter()
            .filter(|n| matches!(n, composition::Op::Draw { .. }))
            .count();
        if layers != 50 {
            return Err("Expected 50 layers per document".into());
        }
        let start = Instant::now();
        let cpu = render(&plan);
        let cpu_ms = start.elapsed().as_secs_f64() * 1000.;
        let start = Instant::now();
        let job = GpuPlan::new(device, pipeline, assets.clone(), &plan)?;
        let setup_ms = start.elapsed().as_secs_f64() * 1000.;
        let first = job.run(device, queue, pipeline)?;
        let mut max_error = 0;
        let mut outside = 0;
        let mut differing = 0;
        for (a, b) in cpu.rgba.iter().zip(&first.image.rgba) {
            let error = a.iter().zip(b).map(|(a, b)| a.abs_diff(*b)).max().unwrap();
            max_error = max_error.max(error);
            outside += usize::from(error > 2);
            differing += usize::from(error > 0);
        }
        passed &= outside == 0;
        save(output.join(format!("document-{document}-cpu.png")), &cpu)?;
        save(
            output.join(format!("document-{document}-gpu.png")),
            &first.image,
        )?;
        let mut gpu_ms = Vec::new();
        let mut wall_ms = Vec::new();
        for _ in 0..30 {
            let frame = job.run(device, queue, pipeline)?;
            if frame.image.rgba != first.image.rgba {
                return Err("Resident GPU plan produced unstable pixels".into());
            }
            gpu_ms.push(frame.gpu_ms);
            wall_ms.push(frame.submit_wait_readback_ms);
        }
        results.push(json!({"document":document,"source_ids":ids,"source_bytes":40*1024*1024,"layers":layers,"groups":10,"ops":plan.ops().len(),"pixels":cpu.rgba.len(),
            "max_channel_error":max_error,"differing_pixels":differing,"pixels_outside_tolerance":outside,"decode_ms":decode_ms,"cpu_wall_ms":cpu_ms,"setup_ms":setup_ms,
            "first_gpu_ms":first.gpu_ms,"first_submit_wait_readback_ms":first.submit_wait_readback_ms,"warm_gpu_ms":gpu_ms,"warm_submit_wait_readback_ms":wall_ms,
            "resident_buffer_bytes":job.resident_buffer_bytes}));
        println!(
            "Composed standard document {document}: max error {max_error}, CPU {cpu_ms:.1} ms"
        );
    }
    Ok(
        json!({"schema_version":1,"semantics_version":1,"status":if passed {"passed"} else {"failed"},"cases":results,"tolerance_per_channel":2,
        "pixel_dimensions":[1024,1024],"project_assets":100,"project_documents":10,"project_layers":500,"warm_dispatches":300,"window_used":false,"system_capture_used":false,
        "limits":["Same generated PNGs and hierarchy as standard workload; group opacity uses composition semantics v1 instead of historical inherited layer opacity",
            "Warm dispatches reuse the same immutable plan, not 300 interactive edits or UI frames","Resident bytes count explicit buffers only; exclude driver, staging, query metadata and private shader stack/spills",
            "GPU timestamps cover the compute pass, not source upload, readback, PNG encoding or editor latency","No stable, masks, effects, ICC, mipmaps or physical input/audio verification"]}),
    )
}
fn main() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let requested = PathBuf::from(args.next().ok_or("new output directory required")?);
    let scale_fixture = match args.next().as_deref() {
        None => None,
        Some("--scale") => Some(PathBuf::from(
            args.next().ok_or("Missing fixture directory")?,
        )),
        _ => return Err("Unknown composition option".into()),
    };
    if args.next().is_some() {
        return Err("Unexpected composition arguments".into());
    }
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("artifacts/p0")
        .canonicalize()
        .map_err(|e| e.to_string())?;
    let parent = requested
        .parent()
        .ok_or("missing output parent")?
        .canonicalize()
        .map_err(|e| e.to_string())?;
    if parent != root || requested.exists() {
        return Err("Output must be a new immediate child of artifacts/p0".into());
    }
    let output = parent.join(requested.file_name().ok_or("missing output name")?);
    std::fs::create_dir(&output).map_err(|e| e.to_string())?;
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN,
        ..wgpu::InstanceDescriptor::new_without_display_handle()
    });
    let adapter =
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
            .map_err(|e| e.to_string())?;
    let info = adapter.get_info();
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("P0 composition proof"),
        required_features: adapter.features() & wgpu::Features::TIMESTAMP_QUERY,
        memory_hints: wgpu::MemoryHints::MemoryUsage,
        ..Default::default()
    }))
    .map_err(|e| e.to_string())?;
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("P0 composition"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../composition.wgsl").into()),
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("P0 composition"),
        layout: None,
        module: &shader,
        entry_point: Some("main"),
        compilation_options: Default::default(),
        cache: None,
    });
    if let Some(fixture) = scale_fixture {
        let mut result = scale(fixture, &output, &device, &queue, &pipeline)?;
        result["adapter"] = json!({"name":info.name,"backend":format!("{:?}",info.backend),"driver":info.driver,"driver_info":info.driver_info});
        std::fs::write(
            output.join("result.json"),
            serde_json::to_vec_pretty(&result).unwrap(),
        )
        .map_err(|e| e.to_string())?;
        if result["status"] != "passed" {
            return Err("Standard composition pixel tolerance exceeded".into());
        }
        return Ok(());
    }
    let (generated, cases) = fixtures();
    let mut assets = Vec::new();
    for (index, image) in generated.iter().enumerate() {
        let path = output.join(format!("asset-{index}.png"));
        save(path.clone(), image)?;
        let png = image::open(path).map_err(|e| e.to_string())?.into_rgba8();
        assets.push(Image {
            size: [png.width(), png.height()],
            rgba: png
                .as_raw()
                .chunks_exact(4)
                .map(|p| [p[0], p[1], p[2], p[3]])
                .collect(),
        });
    }
    let assets: std::sync::Arc<[Image]> = assets.into();
    let mut results = Vec::new();
    let mut passed = true;
    for (name, nodes) in cases {
        let plan = compile([40, 36], &assets, &nodes)?;
        let start = Instant::now();
        let cpu = render(&plan);
        let cpu_ms = start.elapsed().as_secs_f64() * 1000.;
        let start = Instant::now();
        let job = GpuPlan::new(&device, &pipeline, assets.clone(), &plan)?;
        let frame = job.run(&device, &queue, &pipeline)?;
        let gpu = frame.image;
        let gpu_wall_ms = start.elapsed().as_secs_f64() * 1000.;
        let mut max_error = 0;
        let mut differing = 0;
        let mut outside = 0;
        for (a, b) in cpu.rgba.iter().zip(&gpu.rgba) {
            let error = a.iter().zip(b).map(|(a, b)| a.abs_diff(*b)).max().unwrap();
            max_error = max_error.max(error);
            differing += usize::from(error > 0);
            outside += usize::from(error > 2);
        }
        passed &= outside == 0;
        save(output.join(format!("{name}-cpu.png")), &cpu)?;
        save(output.join(format!("{name}-gpu.png")), &gpu)?;
        results.push(json!({"case":name,"pixels":cpu.rgba.len(),"max_channel_error":max_error,"differing_pixels":differing,"pixels_outside_tolerance":outside,
            "cpu_wall_ms":cpu_ms,"gpu_setup_submit_readback_wall_ms":gpu_wall_ms,"ops":plan.ops().len(),"gpu_execution_ms":frame.gpu_ms,"submit_wait_readback_ms":frame.submit_wait_readback_ms,"resident_buffer_bytes":job.resident_buffer_bytes}));
    }
    let result = json!({"schema_version":1,"semantics_version":1,"status":if passed{"passed"}else{"failed"},"adapter":{"name":info.name,"backend":format!("{:?}",info.backend),"driver":info.driver,"driver_info":info.driver_info},
        "tolerance_per_channel":2,"pixel_dimensions":[40,36],"cases":results,"window_used":false,"system_capture_used":false,
        "limits":["Small generated fixtures, not standard-load performance or final composition backend","sRGB linear-light policy is an editor experiment, not verified stable behavior","No masks, effects, color profiles, project edits, audio or game logic","GPU wall time includes setup and readback; not GPU execution timing"]});
    std::fs::write(
        output.join("result.json"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .map_err(|e| e.to_string())?;
    if !passed {
        return Err("CPU/GPU pixel tolerance exceeded".into());
    }
    println!("Composition evidence: {}", output.display());
    Ok(())
}

#[cfg(test)]
mod fixture_tests {
    use super::*;
    #[test]
    fn fixture_keeps_isolated_groups_remaps_shared_sources_and_rejects_bad_references() {
        let leaf = |id| json!({"asset":id,"x":0,"y":0,"scale":1,"opacity":1});
        let nested = |children| json!({"x":0,"y":0,"scale":1,"opacity":0.5,"children":children});
        let fixture = json!([nested(json!([nested(json!([leaf(3), leaf(99)]))]))]);
        let mut ids = std::collections::BTreeSet::new();
        let mut nodes = standard_nodes(&fixture, &mut ids, 0, &mut 0).unwrap();
        let ids: Vec<_> = ids.into_iter().collect();
        assert_eq!(ids, vec![3, 99]);
        remap(&mut nodes, &ids);
        let assets = [
            Image {
                size: [1, 1],
                rgba: vec![[255, 0, 0, 255]],
            },
            Image {
                size: [1, 1],
                rgba: vec![[0, 0, 255, 255]],
            },
        ];
        assert_eq!(
            render(&compile([1, 1], &assets, &nodes).unwrap()).rgba,
            vec![[0, 0, 255, 64]]
        );
        assert!(standard_nodes(&json!([leaf(100)]), &mut Default::default(), 0, &mut 0).is_err());
        let mut excessive = leaf(0);
        for _ in 0..9 {
            excessive = nested(json!([excessive]));
        }
        assert!(standard_nodes(&json!([excessive]), &mut Default::default(), 0, &mut 0).is_err());
    }
}
