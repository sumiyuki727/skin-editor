use crate::presentation::DeviceLoss;
use eframe::{
    NativeOptions,
    egui_wgpu::{self, wgpu},
};
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU8, Ordering},
};
use std::time::Duration;

/// Never owns a Renderer, Surface, Device, or project. Lives in callback resources.
pub(super) struct RendererWitness(pub Arc<AtomicBool>);
impl Drop for RendererWitness {
    fn drop(&mut self) {
        self.0.store(true, Ordering::Release);
    }
}
pub(super) struct Backend {
    pub instance: wgpu::Instance,
    pub loss: DeviceLoss,
    pub notice: Arc<AtomicU8>,
    pub renderer_gone: Arc<AtomicBool>,
    pub options: NativeOptions,
    pub adapter_name: Vec<u8>,
}
impl Backend {
    pub fn new() -> Result<Self, String> {
        Self::with_fault(None)
    }
    pub fn with_fault(fault: Option<Arc<super::fault::Signals>>) -> Result<Self, String> {
        let mut config = crate::gpu_observation::configuration(true);
        let egui_wgpu::WgpuSetup::CreateNew(setup) = &config.wgpu_setup else {
            return Err("new setup required".into());
        };
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN,
            ..wgpu::InstanceDescriptor::new_without_display_handle()
        });
        let adapter = pollster::block_on(instance.request_adapter(&Default::default()))
            .map_err(|e| e.to_string())?;
        let adapter_name = adapter.get_info().name.into_bytes();
        let descriptor = (setup.device_descriptor)(&adapter);
        if !matches!(descriptor.memory_hints, wgpu::MemoryHints::MemoryUsage) {
            return Err("memory hint lost".into());
        }
        let (device, queue) =
            pollster::block_on(adapter.request_device(&descriptor)).map_err(|e| e.to_string())?;
        let loss = DeviceLoss::default();
        let notice = Arc::new(AtomicU8::new(0));
        let (signal, n) = (loss.clone(), notice.clone());
        // Existing setup: this test owns callback installation before eframe sees the device.
        let hook = fault.clone();
        device.set_device_lost_callback(move |reason, _| {
            signal.report();
            if let Some(fault) = &hook {
                fault.device_notices.fetch_add(1, Ordering::AcqRel);
                fault.notify();
            }
            n.store(
                if reason == wgpu::DeviceLostReason::Destroyed {
                    1
                } else {
                    2
                },
                Ordering::Release,
            );
        });
        if let Some(fault) = fault {
            let errors = fault.clone();
            device.on_uncaptured_error(Arc::new(move |e| errors.native_error(&e)));
            let previous = config.on_surface_status.clone();
            let surface_loss = loss.clone();
            config.on_surface_status = Arc::new(move |status| {
                if matches!(
                    status,
                    wgpu::CurrentSurfaceTexture::Lost | wgpu::CurrentSurfaceTexture::Validation
                ) {
                    fault.surface_status.fetch_or(
                        if matches!(status, wgpu::CurrentSurfaceTexture::Lost) {
                            8
                        } else {
                            16
                        },
                        Ordering::AcqRel,
                    );
                    // A surface can expose loss before wgpu dispatches the
                    // DeviceLost callback. Close audio here, not on a later UI poll.
                    if fault.fired.load(Ordering::Acquire) == 0 {
                        fault.premature_errors.fetch_add(1, Ordering::AcqRel);
                    }
                    fault.surface_faults.fetch_add(1, Ordering::AcqRel);
                    surface_loss.report();
                    fault.notify();
                    return egui_wgpu::SurfaceErrorAction::SkipFrame;
                }
                if fault.fired.load(Ordering::Acquire) == 0 {
                    return previous(status);
                }
                let flag = match status {
                    wgpu::CurrentSurfaceTexture::Timeout => 1,
                    wgpu::CurrentSurfaceTexture::Occluded => 2,
                    wgpu::CurrentSurfaceTexture::Outdated => 4,
                    wgpu::CurrentSurfaceTexture::Lost => 8,
                    wgpu::CurrentSurfaceTexture::Validation => 16,
                    _ => 32,
                };
                fault.surface_status.fetch_or(flag, Ordering::AcqRel);
                // A dead Device cannot be repaired by reconfiguring its Surface.
                egui_wgpu::SurfaceErrorAction::SkipFrame
            });
        }
        config.wgpu_setup = egui_wgpu::WgpuSetupExisting {
            instance: instance.clone(),
            adapter,
            device,
            queue,
        }
        .into();
        Ok(Self {
            adapter_name,
            instance,
            loss,
            notice,
            renderer_gone: Arc::new(AtomicBool::new(false)),
            options: NativeOptions {
                renderer: eframe::Renderer::Wgpu,
                wgpu_options: config,
                viewport: crate::gui_baseline::viewport(None)
                    .with_active(false)
                    .with_mouse_passthrough(true),
                persist_window: false,
                ..Default::default()
            },
        })
    }
}
pub(super) fn destroy(device: &wgpu::Device, notice: &AtomicU8) -> Result<(), String> {
    device.destroy();
    let end = std::time::Instant::now() + Duration::from_secs(5);
    while notice.load(Ordering::Acquire) == 0 {
        let _ = device.poll(wgpu::PollType::Poll);
        if std::time::Instant::now() >= end {
            return Err("framework device notice timeout".into());
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    if notice.load(Ordering::Acquire) != 1 {
        return Err("unexpected device notice".into());
    }
    Ok(())
}
pub(super) fn retired(
    renderer_gone: bool,
    surfaces: bool,
    devices: bool,
    targets: usize,
    worker: bool,
    importer: bool,
) -> bool {
    renderer_gone && surfaces && devices && targets == 0 && worker && importer
}
/// wgpu-core 30.0.1 reserves a FutureId before surface acquisition; its error
/// path can leave an unassigned ID. Never call that an empty registry. Only
/// accept exactly the observed failed acquisitions, with no user-kept object,
/// and retire the entire Instance before opening another device.
pub(super) fn surface_ids_only(allocated: usize, kept: usize, failures: usize) -> bool {
    failures > 0 && allocated == failures && kept == 0
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn witness_reports_actual_owner_drop_and_does_not_pin_the_owner() {
        let flag = Arc::new(AtomicBool::new(false));
        let w = RendererWitness(flag.clone());
        assert!(!flag.load(Ordering::Acquire));
        drop(w);
        assert!(flag.load(Ordering::Acquire));
    }
    #[test]
    fn next_window_requires_every_actual_retirement_condition() {
        assert!(retired(true, true, true, 0, true, true));
        for i in 0..5 {
            let mut flags = [true; 5];
            flags[i] = false;
            assert!(!retired(
                flags[0], flags[1], flags[2], 0, flags[3], flags[4]
            ));
        }
        assert!(!retired(true, true, true, 1, true, true));
    }
    #[test]
    fn unassigned_surface_ids_require_exact_failure_evidence_and_no_live_object() {
        assert!(surface_ids_only(3, 0, 3));
        assert!(!surface_ids_only(3, 0, 0));
        assert!(!surface_ids_only(3, 0, 2));
        assert!(!surface_ids_only(3, 1, 3));
    }
}
