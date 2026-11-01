//! Offscreen real-device destroy/rebuild and retained egui callback verification.
//! Does not reset the user's GPU or replace eframe's device in a live window.
use crate::{NativeHost, fresh_session, sample, verification};
use eframe::{egui, egui_wgpu::wgpu};
use skin_editor_application::*;
use skin_editor_infrastructure::Images;
use skin_editor_rendering::SourceUpload;
use skin_editor_ui::{EditorUi, Presenter};
use std::{path::Path, sync::mpsc, time::Duration};
type StageObserver<'a> = dyn FnMut(&str, Option<&wgpu::Device>) -> Result<(), String> + 'a;
fn error(e: impl std::fmt::Debug) -> String {
    format!("{e:?}")
}
fn wait(device: &wgpu::Device) -> Result<(), String> {
    device
        .poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: Some(Duration::from_secs(8)),
        })
        .map(|_| ())
        .map_err(error)
}
fn descriptor(memory_usage: bool) -> wgpu::DeviceDescriptor<'static> {
    let mut descriptor = wgpu::DeviceDescriptor::default();
    crate::gpu_observation::apply_memory_hint(&mut descriptor, memory_usage);
    descriptor
}
pub fn run(destination: &str, memory_usage: bool) -> Result<(), String> {
    run_observed(destination, memory_usage, &mut |_, _| Ok(()))
}
pub(super) fn run_observed(
    destination: &str,
    memory_usage: bool,
    observe: &mut StageObserver<'_>,
) -> Result<(), String> {
    let path = Path::new(destination);
    std::fs::create_dir(path).map_err(error)?;
    crate::load_verification::phase("graphics_setup");
    observe("before_instance", None)?;
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN,
        ..wgpu::InstanceDescriptor::new_without_display_handle()
    });
    let adapter =
        pollster::block_on(instance.request_adapter(&Default::default())).map_err(error)?;
    let adapter_name = adapter.get_info().name;
    observe("instance_adapter", None)?;
    let first_descriptor = descriptor(memory_usage);
    let first_hint = matches!(
        first_descriptor.memory_hints,
        wgpu::MemoryHints::MemoryUsage
    );
    let (device, queue) =
        pollster::block_on(adapter.request_device(&first_descriptor)).map_err(error)?;
    observe("device", Some(&device))?;
    let ctx = egui::Context::default();
    let mut images = Images::default();
    let rgba = images.observe_rgba();
    let raw = images.blobs().observe_bytes();
    let mut session = fresh_session(&images)?;
    let mut ui = EditorUi::default();
    sample(&mut ui, &mut session, &mut images);
    ui.paused = true;
    let snapshot = session.snapshot();
    observe("model", Some(&device))?;
    let mut host = NativeHost::with_upload(
        device,
        queue,
        wgpu::TextureFormat::Rgba8Unorm,
        images.clone(),
        ctx.clone(),
        SourceUpload::QueueWrite,
    )
    .map_err(error)?;
    observe("pipelines", Some(&host.device))?;
    let loss = host.loss_signal();
    let signal = loss.clone();
    let (tx, rx) = mpsc::sync_channel(1);
    host.device.set_device_lost_callback(move |reason, _| {
        signal.report();
        let _ = tx.try_send(format!("{reason:?}"));
    });
    verification::frame(&mut host, &ui, &session, [64, 64], 1)?;
    observe("upload_dispatch", Some(&host.device))?;
    let first_key = host.current().ok_or("no first frame")?.frame.key;
    crate::load_verification::phase("graphics_active");
    std::thread::sleep(Duration::from_secs(2)); // Fixed PID-budget observation, not a race oracle.
    let original = verification::readback_observed(
        &host,
        host.active_texture().unwrap(),
        [64, 64],
        &mut |stage| observe(stage, Some(&host.device)),
    )?;
    observe("readback_released", Some(&host.device))?;
    let held = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(64., 64.),
            )),
            ..Default::default()
        },
        |root| {
            host.paint(root, first_key, root.max_rect());
        },
    );
    if held.shapes.is_empty() {
        return Err("no held draw".into());
    }
    // Last request remains in the real worker/completion slot at loss; no poll
    // can publish its result after the device callback invalidates this host.
    session
        .request_preview(
            &mut host,
            PreviewContext {
                target: ui.target.unwrap(),
                draft: None,
                size: [128, 96],
                viewport_generation: 2,
                scene: 1,
                simulation_micros: 42,
            },
        )
        .map_err(error)?;
    host.wait_preparation().map_err(error)?;
    if !host.preparation_pending() {
        return Err("loss did not exercise pending preparation".into());
    }
    observe("pending_upload", Some(&host.device))?;
    crate::load_verification::phase("device_destroyed");
    host.device.destroy();
    let notice_deadline = std::time::Instant::now() + Duration::from_secs(5);
    let lost_reason = loop {
        // In pinned wgpu 30.0.1, destroy invalidates immediately but the lost
        // notification waits for queued operations and poll/maintain progress.
        let _ = host.device.poll(wgpu::PollType::Poll);
        match rx.try_recv() {
            Ok(reason) => break reason,
            Err(mpsc::TryRecvError::Disconnected) => {
                return Err("device notice channel disconnected".into());
            }
            Err(mpsc::TryRecvError::Empty) => {}
        }
        if std::time::Instant::now() >= notice_deadline {
            return Err("device loss notice deadline".into());
        }
        std::thread::sleep(Duration::from_millis(1));
    };
    if host.current().is_some() || host.active_texture().is_some() {
        return Err("old frame remained presentable after loss".into());
    }
    if host.poll().is_ok() {
        return Err("lost host accepted a poll".into());
    }
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    while !host.preparation_closed() {
        if std::time::Instant::now() >= deadline {
            return Err("lost worker did not exit".into());
        }
        std::thread::sleep(Duration::from_millis(1)); // Actual exit observation, not assumed completion.
        let _ = host.poll();
    }
    if host.source_usage().1 != 0 {
        return Err("lost device did not drain/close prepare owner".into());
    }
    let held_targets = host.live_targets();
    if held_targets == 0 {
        return Err("held CPU draw lease was refunded early".into());
    }
    host = match host.into_rebuild_seed() {
        Ok(_) => return Err("retirement ignored the held draw lease".into()),
        Err(owner) => *owner,
    };
    // A replacement Renderer must never receive the old CPU paint output.
    // Keep the old lease until that output is really discarded, then rebuild.
    drop(held);
    let rejected = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(64., 64.),
            )),
            ..Default::default()
        },
        |root| {
            host.paint(root, first_key, root.max_rect());
        },
    );
    if rejected
        .shapes
        .iter()
        .any(|s| matches!(s.shape, egui::epaint::Shape::Callback(_)))
    {
        return Err("lost frame accepted a new draw".into());
    }
    drop(rejected);
    if host.live_targets() != 0 {
        return Err("old draw/GPU completion retained after release".into());
    }
    let seed = host
        .into_rebuild_seed()
        .map_err(|_| "old device owner could not retire")?;
    observe("host_retired", None)?;
    // Parent context owns backend registries/caches too. Retire the complete
    // old context, not just its Device, before opening the next one.
    drop(adapter);
    drop(instance);
    observe("parents_retired", None)?;
    crate::load_verification::phase("graphics_rebuilt");
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN,
        ..wgpu::InstanceDescriptor::new_without_display_handle()
    });
    let adapter =
        pollster::block_on(instance.request_adapter(&Default::default())).map_err(error)?;
    if adapter.get_info().name != adapter_name {
        return Err("adapter changed during context rebuild".into());
    }
    observe("rebuilt_instance_adapter", None)?;
    let rebuilt_descriptor = descriptor(memory_usage);
    let rebuilt_hint = matches!(
        rebuilt_descriptor.memory_hints,
        wgpu::MemoryHints::MemoryUsage
    );
    let (device, queue) =
        pollster::block_on(adapter.request_device(&rebuilt_descriptor)).map_err(error)?;
    observe("rebuilt_device", Some(&device))?;
    let mut host = seed
        .build(device, queue, wgpu::TextureFormat::Rgba8Unorm, ctx.clone())
        .map_err(error)?;
    observe("rebuilt_pipelines", Some(&host.device))?;
    loss.report(); // a late old-device callback cannot invalidate the replacement
    verification::frame(&mut host, &ui, &session, [64, 64], 3)?;
    observe("rebuilt_upload_dispatch", Some(&host.device))?;
    let rebuilt_key = host.current().ok_or("no rebuilt frame")?.frame.key;
    if rebuilt_key <= first_key {
        return Err("frame key reused after rebuild".into());
    }
    let rebuilt = verification::readback_observed(
        &host,
        host.active_texture().unwrap(),
        [64, 64],
        &mut |stage| observe(stage, Some(&host.device)),
    )?;
    observe("rebuilt_readback_released", Some(&host.device))?;
    if rebuilt != original {
        return Err("device rebuild changed pixels".into());
    }
    std::thread::sleep(Duration::from_secs(2)); // Same fixed observation after device rebuild.
    // Rapid size/viewport-generation requests. This is not physical DPI/minimize.
    for n in 4..=35 {
        let size = if n % 2 == 0 { [96, 128] } else { [128, 96] };
        session
            .request_preview(
                &mut host,
                PreviewContext {
                    target: ui.target.unwrap(),
                    draft: None,
                    size,
                    viewport_generation: n,
                    scene: 0,
                    simulation_micros: 42,
                },
            )
            .map_err(error)?;
    }
    verification::frame(&mut host, &ui, &session, [64, 64], 36)?;
    observe("viewport_storm", Some(&host.device))?;
    if host
        .current()
        .unwrap()
        .frame
        .source
        .context
        .viewport_generation
        != 36
        || session.snapshot() != snapshot
    {
        return Err("stale viewport or project mutation".into());
    }
    drop(snapshot);
    host.shutdown();
    wait(&host.device)?;
    host.poll().map_err(error)?;
    let final_targets = host.live_targets();
    let worker_closed = host.preparation_closed();
    let shutdown_ok = host.shutdown_succeeded();
    if final_targets != 0 || !worker_closed || !shutdown_ok {
        return Err("rebuilt host failed normal shutdown".into());
    }
    observe("shutdown", Some(&host.device))?;
    drop(host);
    observe("final_host_retired", None)?;
    drop(adapter);
    drop(instance);
    observe("final_parents_retired", None)?;
    drop(session);
    drop(ui);
    drop(images);
    let _ = ctx.run_ui(Default::default(), |_| {});
    if rgba.usage().bytes != 0 || raw.usage().bytes != 0 {
        return Err("graphics leases survived final owner drop".into());
    }
    observe("final_leases_released", None)?;
    crate::load_verification::phase("closed");
    std::fs::write(path.join("first.rgba"), &original).map_err(error)?;
    std::fs::write(path.join("rebuilt.rgba"), &rebuilt).map_err(error)?;
    std::fs::write(path.join("report.json"),format!("{{\"schema\":5,\"gpu_memory_usage_hint\":{memory_usage},\"device_memory_usage_hints\":[{first_hint},{rebuilt_hint}],\"adapter_utf8\":{:?},\"scope\":\"offscreen-own-device-destroy-rebuild\",\"full_editor\":false,\"physical_dpi_minimize\":false,\"device_lost_reason\":\"{lost_reason}\",\"inflight_prepare\":true,\"held_targets_after_loss\":{held_targets},\"old_cpu_output_discarded\":true,\"device_owner_released_before_open\":true,\"parent_context_released_before_open\":true,\"late_draw_request_rejected\":true,\"old_signal_rejected\":true,\"frame_keys\":[{},{}],\"pixels_exact\":true,\"viewport_requests\":32,\"final_targets\":{final_targets},\"worker_closed\":{worker_closed},\"shutdown_ok\":{shutdown_ok},\"raw_final\":{},\"rgba_final\":{},\"raw_peak\":{},\"rgba_peak\":{}}}",adapter_name.as_bytes(),first_key.0,rebuilt_key.0,raw.usage().bytes,rgba.usage().bytes,raw.usage().peak_bytes,rgba.usage().peak_bytes)).map_err(error)
}

#[cfg(test)]
mod tests {
    #[test]
    fn both_incarnations_keep_defaults_except_explicit_memory_hint() {
        for enabled in [false, true] {
            for mut value in [super::descriptor(enabled), super::descriptor(enabled)] {
                assert_eq!(
                    matches!(value.memory_hints, super::wgpu::MemoryHints::MemoryUsage),
                    enabled
                );
                value.memory_hints = super::wgpu::DeviceDescriptor::default().memory_hints;
                assert_eq!(
                    format!("{value:?}"),
                    format!("{:?}", super::wgpu::DeviceDescriptor::default())
                );
            }
        }
    }
}
