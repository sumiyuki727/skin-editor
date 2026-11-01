//! Opt-in own-renderer readback. No OS capture, game, personal files or audio.
use super::*;
use eframe::{
    egui,
    egui_wgpu::{self, wgpu},
};
use skin_editor_infrastructure::write_png;
use skin_editor_rendering::{Plan, Target, compose};
use skin_editor_ui::{Presenter, Selection};
use std::path::Path;
use std::time::Duration;

fn wait(device: &wgpu::Device) -> Result<(), String> {
    device
        .poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: Some(Duration::from_secs(20)),
        })
        .map(|_| ())
        .map_err(|e| e.to_string())
}
pub(super) fn frame(
    host: &mut NativeHost,
    ui: &EditorUi,
    session: &ProjectSession,
    size: [u32; 2],
    generation: u64,
) -> Result<(), String> {
    for _ in 0..5 {
        let descriptor = session
            .request_preview(
                host,
                PreviewContext {
                    target: ui.target.ok_or("no target")?,
                    draft: session
                        .draft_info(session.project_info().session_id)
                        .map_err(|e| format!("{e:?}"))?
                        .map(|d| d.state),
                    size,
                    viewport_generation: generation,
                    scene: ui.scene,
                    simulation_micros: ui.simulation_micros,
                },
            )
            .map_err(|e| format!("{e:?}"))?;
        if descriptor.status == FrameStatus::Ready {
            return Ok(());
        }
        host.wait_preparation().map_err(|e| format!("{e:?}"))?;
        wait(&host.device)?;
    }
    Err("frame did not become ready".into())
}
pub(super) fn readback(
    host: &NativeHost,
    texture: &wgpu::Texture,
    size: [u32; 2],
) -> Result<Vec<u8>, String> {
    readback_observed(host, texture, size, &mut |_| Ok(()))
}
pub(super) fn readback_observed(
    host: &NativeHost,
    texture: &wgpu::Texture,
    size: [u32; 2],
    observe: &mut dyn FnMut(&str) -> Result<(), String>,
) -> Result<Vec<u8>, String> {
    let stride = (size[0] * 4).div_ceil(256) * 256;
    let buffer = host.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("own frame readback"),
        size: u64::from(stride) * u64::from(size[1]),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    observe("readback_buffer")?;
    let mut encoder = host.device.create_command_encoder(&Default::default());
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(stride),
                rows_per_image: None,
            },
        },
        wgpu::Extent3d {
            width: size[0],
            height: size[1],
            depth_or_array_layers: 1,
        },
    );
    host.queue.submit([encoder.finish()]);
    observe("readback_submitted")?;
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    buffer
        .slice(..)
        .map_async(wgpu::MapMode::Read, move |result| {
            let _ = tx.send(result);
        });
    wait(&host.device)?;
    rx.recv_timeout(Duration::from_secs(5))
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())?;
    observe("readback_mapped")?;
    let mapped = buffer
        .slice(..)
        .get_mapped_range()
        .map_err(|e| e.to_string())?;
    let mut pixels = Vec::with_capacity((size[0] * size[1] * 4) as usize);
    for row in mapped.chunks_exact(stride as usize) {
        pixels.extend_from_slice(&row[..size[0] as usize * 4]);
    }
    drop(mapped);
    buffer.unmap();
    Ok(pixels)
}
pub(super) fn cpu_pixels(
    session: &ProjectSession,
    ui: &EditorUi,
    images: &Images,
    size: [u32; 2],
) -> Result<Vec<u8>, String> {
    let target = match ui.target.unwrap() {
        PreviewTarget::Document(id) => Target::Document(id),
        PreviewTarget::Components => Target::Components,
        PreviewTarget::Workspace => Target::Workspace,
    };
    let plan = Plan::build(
        &EditorUi::project(session),
        target,
        PixelSize::new(size[0], size[1]).unwrap(),
    )
    .map_err(|e| e.0)?;
    let inputs = plan
        .sources()
        .into_iter()
        .map(|blob| {
            images
                .raster(blob)
                .map(|r| (blob, r))
                .ok_or("missing source".to_string())
        })
        .collect::<Result<_, _>>()?;
    compose(&plan, &inputs, true).map_err(|e| e.0)
}
fn compare(host: &NativeHost, cpu: &[u8], size: [u32; 2]) -> Result<(Vec<u8>, u8), String> {
    let gpu = readback(host, host.active_texture().ok_or("no GPU image")?, size)?;
    let error = cpu
        .iter()
        .zip(&gpu)
        .map(|(a, b)| a.abs_diff(*b))
        .max()
        .unwrap_or(255);
    if error > 2 {
        return Err(format!("CPU/GPU pixel error {error} exceeds 2"));
    }
    Ok((gpu, error))
}
fn verify_source_buffer_reuse(
    host: &NativeHost,
    session: &ProjectSession,
    ui: &EditorUi,
    images: &Images,
    upload: skin_editor_rendering::SourceUpload,
) -> Result<(), String> {
    use skin_editor_rendering::{GpuComposer, Raster};
    use std::sync::Arc;
    let target = match ui.target.ok_or("missing target")? {
        PreviewTarget::Document(id) => Target::Document(id),
        PreviewTarget::Components => Target::Components,
        PreviewTarget::Workspace => Target::Workspace,
    };
    let plan = Plan::build(
        &EditorUi::project(session),
        target,
        PixelSize::new(64, 64).unwrap(),
    )
    .map_err(|e| e.0)?;
    let original: skin_editor_rendering::Images = plan
        .sources()
        .into_iter()
        .map(|blob| (blob, images.raster(blob).unwrap()))
        .collect();
    if original.is_empty() {
        return Err("source reuse fixture empty".into());
    }
    // Detached renderer inputs deliberately use equal labels with distinct
    // allocations/pixels, exercising the cache's allocation identity check.
    let changed = original
        .iter()
        .map(|(blob, raster)| {
            let mut pixels = raster.pixels.to_vec();
            for pixel in pixels.chunks_exact_mut(4) {
                pixel.swap(0, 1);
            }
            (
                *blob,
                Arc::new(Raster {
                    size: raster.size,
                    pixels: pixels.into(),
                }),
            )
        })
        .collect();
    let composer = GpuComposer::with_upload(&host.device, upload);
    let first = composer
        .render(&host.device, &host.queue, &plan, &original, true, || {})
        .map_err(|e| e.0)?;
    // Submit a changed set before polling the first computation's completion.
    let second = composer
        .render(&host.device, &host.queue, &plan, &changed, true, || {})
        .map_err(|e| e.0)?;
    let larger = original
        .iter()
        .map(|(blob, raster)| {
            // Detached cache stress only: the same label with a larger allocation.
            let mut pixels = Vec::with_capacity(raster.pixels.len() * 4);
            for _ in 0..4 {
                pixels.extend_from_slice(&raster.pixels);
            }
            (
                *blob,
                Arc::new(Raster {
                    size: PixelSize::new(raster.size.width() * 2, raster.size.height() * 2)
                        .unwrap(),
                    pixels: pixels.into(),
                }),
            )
        })
        .collect();
    let grown = composer
        .render(&host.device, &host.queue, &plan, &larger, true, || {})
        .map_err(|e| e.0)?;
    let third = composer
        .render(&host.device, &host.queue, &plan, &original, true, || {})
        .map_err(|e| e.0)?;
    let fourth = composer
        .render(&host.device, &host.queue, &plan, &changed, true, || {})
        .map_err(|e| e.0)?;
    // Clear the cache before the final explicit GPU wait. Completion leases
    // retain any copy/map allocation that has not actually finished.
    composer.clear_sources();
    wait(&host.device)?;
    let one = readback(host, &first.texture, [64, 64])?;
    let two = readback(host, &second.texture, [64, 64])?;
    let three = readback(host, &third.texture, [64, 64])?;
    let four = readback(host, &fourth.texture, [64, 64])?;
    if one == two || one != three || two != four {
        return Err("source writes changed an earlier output".into());
    }
    for (frame, inputs) in [
        (&first, &original),
        (&second, &changed),
        (&grown, &larger),
        (&third, &original),
        (&fourth, &changed),
    ] {
        let actual = readback(host, &frame.texture, [64, 64])?;
        let expected = compose(&plan, inputs, true).map_err(|e| e.0)?;
        if expected.iter().zip(actual).any(|(a, b)| a.abs_diff(b) > 2) {
            return Err("source growth/reuse pixel error".into());
        }
    }
    if composer.source_allocations() != 2
        || composer.source_usage().0 != 5
        || composer.source_usage().1 != 0
    {
        return Err("source growth/shrink lifecycle failed".into());
    }
    let staging = composer.staging_usage();
    if staging.bytes != 0
        || (upload == skin_editor_rendering::SourceUpload::ReusableStaging
            && (staging.allocations != 2 || staging.copies != 5 || staging.remaps != 4))
    {
        return Err("staging reuse/growth/clear lifecycle failed".into());
    }
    Ok(())
}
fn ui_image(
    host: &mut NativeHost,
    ui: &mut EditorUi,
    session: &mut ProjectSession,
    ctx: &egui::Context,
) -> Result<Vec<u8>, String> {
    let size = [1180, 800];
    let mut renderer = egui_wgpu::Renderer::new(
        &host.device,
        wgpu::TextureFormat::Rgba8Unorm,
        Default::default(),
    );
    let texture = host.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("own UI screenshot"),
        size: wgpu::Extent3d {
            width: size[0],
            height: size[1],
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    for pass_index in 0..3 {
        let mut result = None;
        let mut output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1180., 800.),
                )),
                focused: true,
                ..Default::default()
            },
            |root| {
                result = Some(ui.show(root, session, host));
            },
        );
        let primitives = ctx.tessellate(output.shapes, output.pixels_per_point);
        if let Some(probe) = ctx.plugin_opt::<crate::gpu_path::Probe>() {
            probe.lock().after_tessellation();
        }
        for (id, deltas) in &output.textures_delta.set {
            for delta in deltas {
                renderer.update_texture(&host.device, &host.queue, *id, delta);
            }
        }
        let screen = egui_wgpu::ScreenDescriptor {
            size_in_pixels: size,
            pixels_per_point: output.pixels_per_point,
        };
        let mut encoder = host.device.create_command_encoder(&Default::default());
        let mut buffers = renderer.update_buffers(
            &host.device,
            &host.queue,
            &mut encoder,
            &primitives,
            &screen,
        );
        if !buffers.is_empty() {
            return Err("preview created auxiliary presentation command buffers".into());
        }
        if pass_index == 0 {
            // Prepare has encoded an old frame and acquired a completion lease,
            // but egui's command batch has not yet been submitted.
            let old_targets = host.live_targets();
            frame(host, ui, session, [300, 200], 100)?;
            if host.live_targets() <= old_targets {
                return Err("pending egui draw lost its old target lease".into());
            }
        }
        let view = texture.create_view(&Default::default());
        {
            let pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            renderer.render(&mut pass.forget_lifetime(), &primitives, &screen);
        }
        // CPU paint data is gone before submission; the completion lease is
        // now the sole host frame owner for the old pending draw.
        drop(primitives);
        if pass_index == 0 && host.live_targets() < 2 {
            return Err("completion lease did not outlive CPU paint data".into());
        }
        buffers.push(encoder.finish());
        host.queue.submit(buffers);
        host.wait_preparation().map_err(|e| format!("{e:?}"))?;
        wait(&host.device)?;
        for id in &output.textures_delta.free {
            renderer.free_texture(id);
        }
        output.textures_delta.clear();
        if let Some((size, generation)) = result.unwrap().viewport {
            frame(host, ui, session, size, generation)?;
        }
    }
    readback(host, &texture, size)
}
pub fn run(output: &str) -> Result<(), String> {
    run_with_audio(output, false)
}

/// Bounded 60 Hz pointer workload on the real UI/request/worker/GPU path.
/// No surface, OS input injection or per-update completion barrier. This counts
/// canonical frame publications, not physical display FPS or acoustic latency.
pub fn run_transform_cadence(output: &str) -> Result<(), String> {
    run_cadence(output, false)
}
pub fn run_joint_cadence(output: &str) -> Result<(), String> {
    run_cadence(output, true)
}
fn run_cadence(output: &str, joint: bool) -> Result<(), String> {
    let directory = Path::new(output);
    if directory.exists() {
        return Err("output directory already exists".into());
    }
    std::fs::create_dir_all(directory).map_err(|e| e.to_string())?;
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN,
        ..wgpu::InstanceDescriptor::new_without_display_handle()
    });
    let adapter = pollster::block_on(instance.request_adapter(&Default::default()))
        .map_err(|e| e.to_string())?;
    let mut descriptor = wgpu::DeviceDescriptor::default();
    crate::gpu_observation::apply_memory_hint(&mut descriptor, true);
    let (device, queue) =
        pollster::block_on(adapter.request_device(&descriptor)).map_err(|e| e.to_string())?;
    let ctx = egui::Context::default();
    let mut images = Images::default();
    let mut session = fresh_session(&images)?;
    let mut ui = EditorUi::default();
    sample(&mut ui, &mut session, &mut images);
    if joint {
        let first = ui.selection().ok_or("no first item")?;
        let asset = session
            .snapshot()
            .project
            .assets()
            .iter()
            .next()
            .ok_or("no asset")?
            .id();
        ui.add_document(&mut session);
        ui.add_asset_layer(&mut session, asset);
        let second = ui.selection().ok_or("no second item")?;
        if !ui.submit(
            &mut session,
            vec![match second {
                Selection::Layer(document, layer) => Command::SetLayerTransform {
                    document,
                    layer,
                    transform: Affine2::new([1., 0., 0., 1., 220., 128.]).unwrap(),
                },
                _ => return Err("not a layer".into()),
            }],
        ) {
            return Err(ui.message);
        }
        ui.toggle_selection(&mut session, first);
        if ui.selections().len() != 2 || session.snapshot().project.components().iter().len() != 0 {
            return Err("selection created project components".into());
        }
    }
    let mut host = NativeHost::with_upload(
        device,
        queue,
        wgpu::TextureFormat::Rgba8Unorm,
        images.clone(),
        ctx.clone(),
        skin_editor_rendering::SourceUpload::QueueWrite,
    )
    .map_err(|e| format!("{e:?}"))?;
    fn pass(
        ctx: &egui::Context,
        host: &mut NativeHost,
        ui: &mut EditorUi,
        session: &mut ProjectSession,
        events: Vec<egui::Event>,
    ) -> Result<([u32; 2], u64), String> {
        host.poll().map_err(|e| format!("{e:?}"))?;
        let mut output = None;
        let mut paint = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1180., 800.),
                )),
                focused: true,
                events,
                ..Default::default()
            },
            |root| output = Some(ui.show(root, session, host)),
        );
        paint.textures_delta.clear();
        let (size, generation) = output.unwrap().viewport.ok_or("no viewport")?;
        session
            .request_preview(
                host,
                PreviewContext {
                    target: ui.target.ok_or("no target")?,
                    draft: session
                        .draft_info(session.project_info().session_id)
                        .map_err(|e| format!("{e:?}"))?
                        .map(|d| d.state),
                    size,
                    viewport_generation: generation,
                    scene: ui.scene,
                    simulation_micros: 0,
                },
            )
            .map_err(|e| format!("{e:?}"))?;
        Ok((size, generation))
    }
    let mut viewport = ([1, 1], 0);
    for _ in 0..3 {
        viewport = pass(&ctx, &mut host, &mut ui, &mut session, vec![])?;
    }
    let (actual_size, generation) = viewport;
    let canvas = ui.canvas_rect.ok_or("no canvas")?;
    frame(&mut host, &ui, &session, actual_size, generation)?;
    let Selection::Layer(document, layer) = ui.selection().ok_or("no selection")? else {
        return Err("no layer".into());
    };
    let b = session
        .snapshot()
        .project
        .layer_bounds(document, layer)
        .map_err(|e| format!("{e:?}"))?
        .ok_or("no bounds")?;
    let view = host.current().ok_or("no frame")?;
    let p = view
        .view
        .point(
            (b.min()[0] + b.max()[0]) * 0.5,
            (b.min()[1] + b.max()[1]) * 0.5,
        )
        .map_err(|e| format!("{e:?}"))?;
    let start = canvas.min
        + egui::vec2(
            p[0] as f32 / actual_size[0] as f32 * canvas.width(),
            p[1] as f32 / actual_size[1] as f32 * canvas.height(),
        );
    let button = |pos, pressed| egui::Event::PointerButton {
        pos,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: Default::default(),
    };
    let nodes = session.history_len();
    pass(
        &ctx,
        &mut host,
        &mut ui,
        &mut session,
        vec![egui::Event::PointerMoved(start), button(start, true)],
    )?;
    host.enable_trace();
    let publications = host.publications;
    let begun = Instant::now();
    let mut rows =
        String::from("step,elapsed_ms,publications,draft_revision,active_draft_revision\n");
    let mut last = start;
    for n in 0..120 {
        // This is a declared performance stimulus, not race synchronization.
        let due = begun + Duration::from_nanos(16_666_667 * n);
        std::thread::sleep(due.saturating_duration_since(Instant::now()));
        last = start + egui::vec2(20. + n as f32 * 0.5, 10. + (n as f32 * 0.05).sin() * 10.);
        pass(
            &ctx,
            &mut host,
            &mut ui,
            &mut session,
            vec![egui::Event::PointerMoved(last)],
        )?;
        let draft = session
            .draft_info(session.project_info().session_id)
            .map_err(|e| format!("{e:?}"))?
            .ok_or("gesture did not start")?
            .state;
        let shown = host
            .current()
            .and_then(|v| v.frame.source.context.draft)
            .map_or(0, |d| d.revision);
        rows.push_str(&format!(
            "{n},{},{},{},{}\n",
            begun.elapsed().as_secs_f64() * 1000.,
            host.publications - publications,
            draft.revision,
            shown
        ));
    }
    let during = host.publications - publications;
    pass(
        &ctx,
        &mut host,
        &mut ui,
        &mut session,
        vec![button(last, false)],
    )?;
    frame(&mut host, &ui, &session, actual_size, generation)?;
    if session.history_len() != nodes + 1
        || host.current().ok_or("no final frame")?.frame.source.state
            != session.project_info().state
    {
        return Err("final gesture/history did not settle".into());
    }
    std::fs::write(directory.join("cadence.csv"), rows).map_err(|e| e.to_string())?;
    let cpu = cpu_pixels(&session, &ui, &images, actual_size)?;
    let (pixels, pixel_error) = compare(&host, &cpu, actual_size)?;
    std::fs::write(directory.join("final.rgba"), pixels).map_err(|e| e.to_string())?;
    let selected = ui.selections().len();
    let component_nodes = session.snapshot().project.components().iter().len();
    let before_clear = session.snapshot();
    ui.select(&mut session, None);
    if session.snapshot() != before_clear {
        return Err("clearing selection edited project".into());
    }
    host.shutdown();
    let report = format!(
        "{{\"schema\":1,\"scope\":\"offscreen-ui-worker-publications-no-surface\",\"steps\":120,\"requested_hz\":60,\"joint\":{joint},\"selected_before_clear\":{selected},\"component_nodes\":{component_nodes},\"clear_kept_project\":true,\"max_channel_error\":{pixel_error},\"during_publications\":{during},\"allocations\":{},\"source_uploads\":{},\"one_history_node\":true,\"final_source_current\":true,\"shutdown_targets\":{}}}\n",
        host.allocations,
        host.source_usage().0,
        host.live_targets()
    );
    std::fs::write(directory.join("report.json"), report).map_err(|e| e.to_string())?;
    Ok(())
}
pub fn run_memory(output: &str) -> Result<(), String> {
    run_mode(output, false, true, false, false)
}
pub fn run_markers(output: &str) -> Result<(), String> {
    run_mode(output, false, true, false, true)
}
pub fn run_with_audio(output: &str, audio_ui: bool) -> Result<(), String> {
    run_mode(output, audio_ui, false, false, false)
}
pub fn run_staging(output: &str, memory_usage: bool) -> Result<(), String> {
    run_mode(output, false, memory_usage, true, false)
}
fn run_mode(
    output: &str,
    audio_ui: bool,
    memory_usage: bool,
    reusable_staging: bool,
    markers: bool,
) -> Result<(), String> {
    let mut audio_ui_submitted_nonzero_samples = 0;
    let mut audio_ui_library_nonzero_samples = 0;
    let monitor_volume_permille = if audio_ui { "500" } else { "null" };
    let directory = Path::new(output);
    if directory.exists() {
        return Err("output directory already exists".into());
    }
    std::fs::create_dir_all(directory).map_err(|e| e.to_string())?;
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN,
        ..wgpu::InstanceDescriptor::new_without_display_handle()
    });
    let adapter = pollster::block_on(instance.request_adapter(&Default::default()))
        .map_err(|e| e.to_string())?;
    let adapter_name = adapter.get_info().name;
    let mut descriptor = wgpu::DeviceDescriptor::default();
    crate::gpu_observation::apply_memory_hint(&mut descriptor, memory_usage);
    let (device, queue) =
        pollster::block_on(adapter.request_device(&descriptor)).map_err(|e| e.to_string())?;
    let ctx = egui::Context::default();
    if markers {
        let mut probe = crate::gpu_path::Probe::new(crate::gpu_path::Mode::Markers, None)?;
        probe.stage = 4;
        ctx.add_plugin(probe);
    }
    // Optional installed OS font, never a fixture or a distributed resource.
    #[cfg(windows)]
    if let Ok(bytes) = std::fs::read("C:/Windows/Fonts/msyh.ttc") {
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "system-cjk".into(),
            std::sync::Arc::new(egui::FontData::from_owned(bytes)),
        );
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .push("system-cjk".into());
        ctx.set_fonts(fonts);
    }
    let mut images = Images::default();
    let mut session = fresh_session(&images)?;
    let mut ui = EditorUi::default();
    sample(&mut ui, &mut session, &mut images);
    // Include isolated group opacity in the actual GPU fixture.
    let Some(Selection::Layer(document, layer)) = ui.selection() else {
        return Err("sample layer missing".into());
    };
    let group = LayerId::new(ui.object_id()).unwrap();
    if !ui.submit(
        &mut session,
        vec![
            Command::AddLayer {
                document,
                node: LayerNode {
                    id: group,
                    local: Affine2::IDENTITY,
                    opacity: Opacity::new(0.6).unwrap(),
                    visible: true,
                    kind: LayerKind::Group { children: vec![] },
                },
                parent: None,
                index: 0,
            },
            Command::MoveLayer {
                document,
                layer,
                parent: Some(group),
                index: 0,
            },
        ],
    ) {
        return Err(ui.message);
    }
    let upload = if reusable_staging {
        skin_editor_rendering::SourceUpload::ReusableStaging
    } else {
        skin_editor_rendering::SourceUpload::QueueWrite
    };
    let mut host = NativeHost::with_upload(
        device,
        queue,
        wgpu::TextureFormat::Rgba8Unorm,
        images.clone(),
        ctx.clone(),
        upload,
    )
    .map_err(|e| format!("{e:?}"))?;
    let size = [256, 256];
    frame(&mut host, &ui, &session, size, 1)?;
    // Temporal metadata changes preserve old descriptor copies, immutable
    // pixels and target identity; they do not submit/reallocate a composition.
    let before_alias = host.current().ok_or("missing alias source")?;
    let old_time = ui.simulation_micros;
    let old_scene = ui.scene;
    let allocations = host.allocations;
    let uploads = host.source_usage().0;
    for n in 1..=64 {
        ui.simulation_micros = old_time + n * 1000;
        ui.scene = (n % 2) as u8;
        frame(&mut host, &ui, &session, size, 1)?;
        let now = host.current().ok_or("missing timed alias")?;
        if now.frame.key == before_alias.frame.key
            || now.frame.source.context.simulation_micros != ui.simulation_micros
            || before_alias.frame.source.context.simulation_micros != old_time
            || host.allocations != allocations
            || host.source_usage().0 != uploads
            || host.live_targets() != 1
        {
            return Err("temporal descriptor alias changed immutable target or old copy".into());
        }
    }
    ui.simulation_micros = old_time;
    ui.scene = old_scene;
    frame(&mut host, &ui, &session, size, 1)?;
    let (initial, mut max_error) =
        compare(&host, &cpu_pixels(&session, &ui, &images, size)?, size)?;
    // Fixed self-authored GPU frames for exact cross-policy comparison. This
    // diagnostic-only trace is separate from the no-readback performance run.
    let mut pixel_trace = Vec::with_capacity(2 * 1024 * 1024);
    pixel_trace.extend_from_slice(&initial);
    // Empty composition must be a real transparent target, with no source
    // decode/upload required. Keep the normal editor checker separate.
    ui.submit(
        &mut session,
        vec![Command::SetDocumentVisibility {
            document,
            visible: false,
        }],
    );
    let hidden = session.snapshot();
    let hidden_plan = Plan::build(
        &hidden.project,
        Target::Document(document),
        PixelSize::new(256, 256).unwrap(),
    )
    .map_err(|e| e.0)?;
    let blank = skin_editor_rendering::Images::new();
    let transparent_composer =
        skin_editor_rendering::GpuComposer::with_upload(&host.device, upload);
    let transparent = transparent_composer
        .render(
            &host.device,
            &host.queue,
            &hidden_plan,
            &blank,
            false,
            || {},
        )
        .map_err(|e| e.0)?;
    wait(&host.device)?;
    let transparent_pixels = readback(&host, &transparent.texture, size)?;
    pixel_trace.extend_from_slice(&transparent_pixels);
    if transparent_pixels.iter().any(|p| *p != 0)
        || compose(&hidden_plan, &blank, false).map_err(|e| e.0)? != transparent_pixels
    {
        return Err("hidden document was not transparent".into());
    }
    drop(transparent);
    ui.navigation(&mut session, HistoryAction::Undo);
    let original = session.snapshot();
    ui.draft(
        &mut session,
        DraftAction::Begin {
            expected: original.state,
        },
    );
    let state = session
        .draft_info(original.session_id)
        .unwrap()
        .unwrap()
        .state;
    ui.draft(
        &mut session,
        DraftAction::Replace {
            expected: state,
            commands: vec![Command::SetLayerTransform {
                document,
                layer,
                transform: Affine2::new([1., 0., 0., 1., 240., 128.]).unwrap(),
            }],
        },
    );
    frame(&mut host, &ui, &session, size, 1)?;
    let (changed, error) = compare(&host, &cpu_pixels(&session, &ui, &images, size)?, size)?;
    pixel_trace.extend_from_slice(&changed);
    max_error = max_error.max(error);
    if changed == initial || session.snapshot() != original || ui.simulation_micros != 0 {
        return Err("paused draft isolation failed".into());
    }
    ui.cancel(&mut session);
    frame(&mut host, &ui, &session, size, 1)?;
    let restored = readback(&host, host.active_texture().unwrap(), size)?;
    pixel_trace.extend_from_slice(&restored);
    if restored != initial || session.snapshot() != original {
        return Err("cancel did not restore exact frame".into());
    }
    let a = session.project_info().state.head;
    ui.submit(
        &mut session,
        vec![Command::SetLayerTransform {
            document,
            layer,
            transform: Affine2::new([1., 0., 0., 1., 240., 128.]).unwrap(),
        }],
    );
    let b = session.project_info().state.head;
    ui.navigation(&mut session, HistoryAction::Undo);
    ui.submit(
        &mut session,
        vec![Command::SetLayerProperties {
            document,
            layer,
            opacity: Opacity::new(0.3).unwrap(),
            visible: true,
        }],
    );
    let c = session.project_info().state.head;
    ui.navigation(&mut session, HistoryAction::Checkout { node: b });
    frame(&mut host, &ui, &session, size, 1)?;
    let (branch_b, error) = compare(&host, &cpu_pixels(&session, &ui, &images, size)?, size)?;
    pixel_trace.extend_from_slice(&branch_b);
    max_error = max_error.max(error);
    ui.navigation(&mut session, HistoryAction::Checkout { node: c });
    frame(&mut host, &ui, &session, size, 1)?;
    let (branch_c, error) = compare(&host, &cpu_pixels(&session, &ui, &images, size)?, size)?;
    pixel_trace.extend_from_slice(&branch_c);
    max_error = max_error.max(error);
    if a == b || b == c {
        return Err("branch identity failed".into());
    }
    // Competing viewport requests before the first submission completes.
    let mut context = host.current().unwrap().frame.source.context;
    context.size = [192, 128];
    context.viewport_generation = 2;
    session
        .request_preview(&mut host, context)
        .map_err(|e| format!("{e:?}"))?;
    context.size = [257, 193];
    context.viewport_generation = 3;
    session
        .request_preview(&mut host, context)
        .map_err(|e| format!("{e:?}"))?;
    frame(&mut host, &ui, &session, [257, 193], 3)?;
    if host.current().unwrap().frame.source.context.size != [257, 193] {
        return Err("stale viewport published".into());
    }
    let allocations = host.allocations;
    for _ in 0..8 {
        frame(&mut host, &ui, &session, [257, 193], 3)?;
    }
    if host.allocations != allocations {
        return Err("unchanged material was uploaded again".into());
    }
    let (pixels, error) = compare(
        &host,
        &cpu_pixels(&session, &ui, &images, [257, 193])?,
        [257, 193],
    )?;
    max_error = max_error.max(error);
    pixel_trace.extend_from_slice(&pixels);
    write_png(
        &directory.join("preview.png"),
        &pixels,
        PixelSize::new(257, 193).unwrap(),
    )?;
    if audio_ui {
        let staged = crate::import_worker::stage(
            &mut images,
            crate::import_worker::ImportRequest::SampleSound,
        )?;
        let asset = AssetId::new(ui.object_id()).unwrap();
        ui.set_audio_port(Box::new(crate::editor_audio::EditorAudio::new(
            images.blobs().clone(),
        )));
        if !ui.submit(
            &mut session,
            vec![Command::RegisterAsset {
                asset: Asset::new(asset, staged.label, staged.blob, staged.kind)
                    .map_err(|e| format!("{e:?}"))?,
            }],
        ) {
            return Err(ui.message);
        }
        drop(staged.lease);
        ui.audio_command(&session, AudioEditorAction::MonitorVolume(500));
        ui.audio_command(&session, AudioEditorAction::Enable);
        let raw_snapshot = session.snapshot();
        let raw_nodes = session.history_len();
        let raw_count = raw_snapshot.project.audio_components().len();
        let raw_baseline = ui.audio_state.submitted_nonzero_samples;
        ui.audio_command(&session, AudioEditorAction::AuditionAsset(asset));
        let raw_until = std::time::Instant::now() + std::time::Duration::from_secs(3);
        while ui
            .audio_state
            .asset_audition
            .as_ref()
            .is_none_or(|s| s.preparation != AudioPreparationState::Ready)
            || ui.audio_state.submitted_nonzero_samples <= raw_baseline
        {
            if std::time::Instant::now() >= raw_until {
                return Err(format!("library audition timeout: {:?}", ui.audio_state));
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
            ui.refresh_audio(&session);
        }
        audio_ui_library_nonzero_samples = ui.audio_state.submitted_nonzero_samples - raw_baseline;
        if session.snapshot() != raw_snapshot
            || session.history_len() != raw_nodes
            || raw_count != 0
            || ui.audio_state.monitor_volume != 500
            || !ui.audio_state.frozen
            || ui.audio_state.error.is_some()
        {
            return Err("library audition mutated source/history or scene".into());
        }
        ui.audio_command(&session, AudioEditorAction::StopAudition);
        let component = ui
            .create_audio_component(&mut session, asset, "self.preview".into())
            .ok_or("demo component rejected")?;
        ui.audio_asset = Some(asset);
        ui.audio_component = Some(component);
        ui.open_audio_editor();
        ui.audio_command(&session, AudioEditorAction::Enable);
        let until = std::time::Instant::now() + std::time::Duration::from_secs(3);
        loop {
            ui.refresh_audio(&session);
            if ui
                .audio_state
                .components
                .iter()
                .any(|c| c.component == component && c.preparation == AudioPreparationState::Ready)
            {
                break;
            }
            if !ui.audio_state.enabled || std::time::Instant::now() >= until {
                return Err(format!(
                    "audio UI prepare failed: {} {:?}",
                    ui.message, ui.audio_state
                ));
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        let before = session.snapshot();
        let nodes = session.history_len();
        let baseline = ui.audio_state.submitted_nonzero_samples;
        ui.audio_command(&session, AudioEditorAction::Audition(component));
        while ui.audio_state.last_audition.is_none()
            || ui.audio_state.submitted_nonzero_samples <= baseline
        {
            if std::time::Instant::now() >= until {
                return Err("audio UI audition receipt timeout".into());
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
            ui.refresh_audio(&session);
        }
        audio_ui_submitted_nonzero_samples = ui.audio_state.submitted_nonzero_samples - baseline;
        if ui.audio_state.error.is_some()
            || session.snapshot() != before
            || session.history_len() != nodes
            || !ui.audio_state.frozen
        {
            return Err("audio UI audition changed project or frozen scene".into());
        }
    }
    let pixels = ui_image(&mut host, &mut ui, &mut session, &ctx)?;
    if let Some(probe) = ctx.plugin_opt::<crate::gpu_path::Probe>() {
        probe.lock().stage = 0;
    }
    write_png(
        &directory.join("editor.png"),
        &pixels,
        PixelSize::new(1180, 800).unwrap(),
    )?;
    if audio_ui {
        ui.audio_command(&session, AudioEditorAction::Disable);
    }
    // Retired frames disappear after actual callbacks and CPU draw data disposal.
    host.poll().map_err(|e| format!("{e:?}"))?;
    if host.live_targets() != 1 {
        return Err("retired targets were retained after completion".into());
    }
    let source_uploads = host.source_usage().0;
    if source_uploads != 1 {
        return Err(format!(
            "unchanged decoded sources uploaded {source_uploads} times"
        ));
    }
    verify_source_buffer_reuse(&host, &session, &ui, &images, upload)?;
    let key = host.current().ok_or("missing final frame")?.frame.key;
    let held = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(256., 256.),
            )),
            ..Default::default()
        },
        |root| {
            host.paint(
                root,
                key,
                egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(256., 256.)),
            );
        },
    );
    host.shutdown();
    host.poll().map_err(|e| format!("{e:?}"))?;
    if host.live_targets() != 1 {
        return Err("shutdown lost tracking of a CPU-held frame".into());
    }
    drop(held);
    let _ = ctx.run_ui(egui::RawInput::default(), |_| {});
    host.poll().map_err(|e| format!("{e:?}"))?;
    if host.live_targets() != 0 || host.source_usage().1 != 0 || host.staging_usage().bytes != 0 {
        return Err("shutdown retained frame or source cache".into());
    }
    std::fs::write(directory.join("pixel_trace.rgba"), pixel_trace).map_err(|e| e.to_string())?;
    std::fs::write(
        directory.join("presentation_checks.json"),
        "{\"schema\":1,\"temporal_alias_exact\":true,\"auxiliary_command_buffers\":0,\"shared_encoder_lease_verified\":true}\n",
    ).map_err(|e| e.to_string())?;
    let report = format!(
        "{{\n  \"schema\": 1,\n  \"gpu_memory_usage_hint\": {memory_usage},\n  \"reusable_staging\": {reusable_staging},\n  \"pixel_trace_sizes\": [[256,256],[256,256],[256,256],[256,256],[256,256],[256,256],[257,193]],\n  \"adapter\": {adapter_name:?},\n  \"max_channel_error\": {max_error},\n  \"paused_draft_refresh\": true,\n  \"cancel_exact_restore\": true,\n  \"branch_restore\": true,\n  \"stale_viewport_rejected\": true,\n  \"unchanged_uploads\": 0,\n  \"audio_ui_native_audition\": {audio_ui},\n  \"audio_ui_submitted_nonzero_samples\": {audio_ui_submitted_nonzero_samples},\n  \"audio_ui_library_nonzero_samples\": {audio_ui_library_nonzero_samples},\n  \"audio_ui_library_readonly\": {audio_ui},\n  \"monitor_volume_permille\": {monitor_volume_permille},\n  \"source_uploads\": {source_uploads},\n  \"shutdown_held_frame_tracked\": true,\n  \"source_buffer_queue_order_verified\": true,\n  \"allocations\": {},\n  \"publications\": {}\n}}\n",
        host.allocations, host.publications
    );
    std::fs::write(directory.join("report.json"), &report).map_err(|e| e.to_string())?;
    host.shutdown();
    println!("{report}");
    if let Some(probe) = ctx.plugin_opt::<crate::gpu_path::Probe>() {
        let mut probe = probe.lock();
        probe.finish();
        std::fs::write(directory.join("gpu_shutdown.json"), probe.report())
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}
