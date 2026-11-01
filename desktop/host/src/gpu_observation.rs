//! Opt-in allocator observations, never a replacement for OS process counters.
use eframe::{egui, egui_wgpu::wgpu};
use std::time::{Duration, Instant};

/// Explicit single-field policy control; ordinary assembly selects via Policy.
pub fn configuration(memory_usage: bool) -> eframe::egui_wgpu::WgpuConfiguration {
    // NativeOptions uses LOW_LATENCY; standalone WgpuConfiguration defaults
    // to HIGH_THROUGHPUT. Preserve the actual host's presentation policy.
    let mut config = eframe::NativeOptions::default().wgpu_options;
    if memory_usage {
        let eframe::egui_wgpu::WgpuSetup::CreateNew(setup) = &mut config.wgpu_setup else {
            unreachable!()
        };
        let descriptor = setup.device_descriptor.clone();
        setup.device_descriptor = std::sync::Arc::new(move |adapter| {
            let mut value = descriptor(adapter);
            apply_memory_hint(&mut value, true);
            value
        });
    }
    config
}

/// The offscreen pixel control uses the same single-field change as eframe.
pub fn apply_memory_hint(descriptor: &mut wgpu::DeviceDescriptor<'_>, memory_usage: bool) {
    if memory_usage {
        descriptor.memory_hints = wgpu::MemoryHints::MemoryUsage;
    }
}

/// Allocator reports omit swapchain images and allocations outside this device.
/// Keep labels as UTF-8 bytes, just like the existing diagnostic adapter identity.
pub fn snapshot(device: &wgpu::Device, stage: &str) -> Result<String, String> {
    let started = Instant::now();
    encode(device.generate_allocator_report(), stage, started)
}
fn encode(
    report: Option<wgpu::AllocatorReport>,
    stage: &str,
    started: Instant,
) -> Result<String, String> {
    let Some(report) = report else {
        return Ok(format!("{{\"stage\":\"{stage}\",\"supported\":false}}"));
    };
    if report.allocations.len() > 512 || report.blocks.len() > 512 {
        return Err("GPU allocator observation overflow".into());
    }
    let allocations = report
        .allocations
        .iter()
        .map(|a| {
            format!(
                "{{\"name_utf8\":{:?},\"offset\":{},\"bytes\":{}}}",
                a.name.as_bytes(),
                a.offset,
                a.size
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let blocks = report
        .blocks
        .iter()
        .map(|b| {
            format!(
                "{{\"bytes\":{},\"begin\":{},\"end\":{}}}",
                b.size, b.allocations.start, b.allocations.end
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    Ok(format!(
        "{{\"stage\":\"{stage}\",\"supported\":true,\"allocated_bytes\":{},\"reserved_bytes\":{},\"allocations\":[{allocations}],\"blocks\":[{blocks}],\"capture_ms\":{}}}",
        report.total_allocated_bytes,
        report.total_reserved_bytes,
        started.elapsed().as_secs_f64() * 1000.
    ))
}

/// Separate empty eframe control: no project, preview composer, audio or PNG.
/// Two viewport sizes distinguish area-dependent presentation costs. The window
/// is real; no OS window capture or renderer readback is involved.
pub fn baseline(path: &str, large: bool, memory_usage: bool) -> Result<(), String> {
    let path = std::path::PathBuf::from(path);
    if path.exists() || !path.is_absolute() {
        return Err("new absolute output directory required".into());
    }
    std::fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    struct Baseline {
        path: std::path::PathBuf,
        started: Instant,
        snapshots: Vec<String>,
        done: bool,
        memory_usage: bool,
        adapter: String,
    }
    impl eframe::App for Baseline {
        fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
            ui.label("P2-A3 empty presentation control");
            if self.done {
                return;
            }
            if self.started.elapsed() >= Duration::from_secs(3) {
                let device = &frame.wgpu_render_state().unwrap().device;
                let result = snapshot(device, "presented").and_then(|s| {
                    self.snapshots.push(s);
                    let size = ui
                        .ctx()
                        .input(|i| i.viewport().inner_rect.unwrap().size() * i.pixels_per_point);
                    std::fs::write(
                        self.path.join("report.json"),
                        format!(
                            "{{\"gpu_memory_usage_hint\":{},\"adapter_utf8\":{:?},\"physical_size\":{:?},\"snapshots\":[{}]}}\n",
                            self.memory_usage,
                            self.adapter.as_bytes(),
                            [size.x, size.y],
                            self.snapshots.join(",")
                        ),
                    )
                    .map_err(|e| e.to_string())
                });
                if let Err(e) = result {
                    eprintln!("GPU baseline failed: {e}");
                }
                self.done = true;
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            }
            ui.ctx().request_repaint_after(Duration::from_millis(50));
        }
    }
    let report = path.join("report.json");
    eframe::run_native(
        "P2-A3 empty GPU cost control",
        eframe::NativeOptions {
            renderer: eframe::Renderer::Wgpu,
            wgpu_options: configuration(memory_usage),
            viewport: egui::ViewportBuilder::default().with_inner_size(if large {
                [1480., 1000.]
            } else {
                [1180., 800.]
            }),
            ..Default::default()
        },
        Box::new(move |context| {
            let snapshot = snapshot(
                &context.wgpu_render_state.as_ref().unwrap().device,
                "created",
            )
            .map_err(std::io::Error::other)?;
            println!("{{\"phase\":\"blank\"}}");
            Ok(Box::new(Baseline {
                path,
                started: Instant::now(),
                snapshots: vec![snapshot],
                done: false,
                memory_usage,
                adapter: format!(
                    "{:?}",
                    context
                        .wgpu_render_state
                        .as_ref()
                        .unwrap()
                        .adapter
                        .get_info()
                ),
            }))
        }),
    )
    .map_err(|e| e.to_string())?;
    if !report.is_file() {
        return Err("GPU baseline incomplete".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn memory_policy_keeps_requested_device_features_and_limits() {
        let mut descriptor = wgpu::DeviceDescriptor {
            label: Some("pixel-policy-control"),
            required_features: wgpu::Features::TIMESTAMP_QUERY,
            ..Default::default()
        };
        descriptor.required_limits.max_texture_dimension_2d = 2048;
        let before = format!("{descriptor:?}");
        apply_memory_hint(&mut descriptor, false);
        assert_eq!(format!("{descriptor:?}"), before);
        apply_memory_hint(&mut descriptor, true);
        assert!(matches!(
            descriptor.memory_hints,
            wgpu::MemoryHints::MemoryUsage
        ));
        descriptor.memory_hints = wgpu::MemoryHints::Performance;
        assert_eq!(format!("{descriptor:?}"), before);
    }
    #[test]
    fn diagnostic_memory_hint_preserves_native_presentation_policy() {
        let original = eframe::NativeOptions::default().wgpu_options;
        assert_eq!(configuration(false).surface, original.surface);
        assert_eq!(configuration(true).surface, original.surface);
    }
    #[test]
    fn absent_and_overflowing_reports_are_not_fabricated_as_zero() {
        let absent = encode(None, "test", Instant::now()).unwrap();
        assert!(absent.contains("\"supported\":false"));
        assert!(!absent.contains("allocated_bytes"));
        let mut report = wgpu::AllocatorReport {
            allocations: vec![wgpu::wgt::AllocationReport {
                name: "\"\\\n".into(),
                offset: 0,
                size: 4,
            }],
            blocks: vec![wgpu::wgt::MemoryBlockReport {
                size: 64,
                allocations: 0..1,
            }],
            total_allocated_bytes: 4,
            total_reserved_bytes: 64,
        };
        let encoded = encode(Some(report.clone()), "test", Instant::now()).unwrap();
        assert!(encoded.contains("\"name_utf8\":[34, 92, 10]"));
        assert!(encoded.contains("\"allocated_bytes\":4,\"reserved_bytes\":64"));
        report
            .allocations
            .resize(513, report.allocations[0].clone());
        assert!(encode(Some(report), "test", Instant::now()).is_err());
    }
}
