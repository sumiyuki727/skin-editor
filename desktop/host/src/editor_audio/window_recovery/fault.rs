//! Explicit owned-device injection at real eframe callback boundaries.
//! No system/driver reset; handlers never hold the project or a renderer.
use super::*;
use eframe::egui_wgpu::{self, wgpu};
use skin_editor_infrastructure::separated_native::NativeFaultSignal;
use std::sync::{OnceLock, atomic::AtomicU64};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Plan {
    Planned,
    Prepare,
    Paint,
    RebuildFailure,
    InitConsumed,
    InitPrepared,
    InitAudio,
    CleanupHeld,
    EndpointHeld,
    DraftGpu,
    DraftAudio,
    DraftBoth,
}
impl Plan {
    pub fn code(self) -> u8 {
        match self {
            Self::Planned => 0,
            Self::Prepare => 1,
            Self::Paint => 2,
            Self::RebuildFailure => 3,
            Self::InitConsumed => 4,
            Self::InitPrepared => 5,
            Self::InitAudio => 6,
            Self::CleanupHeld => 7,
            Self::EndpointHeld => 8,
            Self::DraftGpu => 9,
            Self::DraftAudio => 10,
            Self::DraftBoth => 11,
        }
    }
    pub fn active_edit(self) -> bool {
        matches!(self, Self::DraftGpu | Self::DraftAudio | Self::DraftBoth)
    }
}
#[derive(Default)]
pub(super) struct Signals {
    pub fired: AtomicU64,
    pub prepares: AtomicU64,
    pub paints: AtomicU64,
    pub errors: AtomicU64,
    pub validation_errors: AtomicU64,
    pub internal_errors: AtomicU64,
    pub out_of_memory: AtomicU64,
    pub premature_errors: AtomicU64,
    pub surface_status: AtomicU64,
    pub noticed: AtomicU64,
    pub surface_faults: AtomicU64,
    pub device_notices: AtomicU64,
    pub logic_observed: AtomicU64,
    pub audio_closed_before_logic: AtomicU64,
    audio: OnceLock<NativeFaultSignal>,
    context: OnceLock<egui::Context>,
}
impl Signals {
    pub fn bind(&self, audio: NativeFaultSignal, context: egui::Context) -> Result<(), String> {
        self.audio
            .set(audio)
            .map_err(|_| "fault audio already bound")?;
        self.context
            .set(context)
            .map_err(|_| "fault context already bound".to_owned())
    }
    pub fn notify(&self) {
        self.noticed.fetch_add(1, Ordering::AcqRel);
        if let Some(audio) = self.audio.get() {
            audio.report();
        }
        if let Some(context) = self.context.get() {
            context.request_repaint();
        }
    }
    pub fn record_error(&self) {
        self.errors.fetch_add(1, Ordering::AcqRel);
        if self.fired.load(Ordering::Acquire) == 0 {
            self.premature_errors.fetch_add(1, Ordering::AcqRel);
        }
    }
    pub fn native_error(&self, error: &wgpu::Error) {
        self.record_error();
        let counter = match error {
            wgpu::Error::Validation { .. } => &self.validation_errors,
            wgpu::Error::Internal { .. } => &self.internal_errors,
            wgpu::Error::OutOfMemory { .. } => &self.out_of_memory,
        };
        counter.fetch_add(1, Ordering::AcqRel);
    }
    fn fire(&self, device: &wgpu::Device) {
        if self
            .fired
            .compare_exchange(0, 1, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
        {
            device.destroy();
        }
    }
    pub fn json(&self) -> String {
        let n = |a: &AtomicU64| a.load(Ordering::Acquire);
        format!(
            "{{\"fired\":{},\"prepares\":{},\"paints\":{},\"errors\":{},\"premature_errors\":{},\"surface_status\":{},\"noticed\":{},\"logic_observed\":{},\"audio_closed_before_logic\":{},\"validation_errors\":{},\"internal_errors\":{},\"out_of_memory\":{},\"surface_faults\":{},\"device_notices\":{}}}",
            n(&self.fired),
            n(&self.prepares),
            n(&self.paints),
            n(&self.errors),
            n(&self.premature_errors),
            n(&self.surface_status),
            n(&self.noticed),
            n(&self.logic_observed),
            n(&self.audio_closed_before_logic),
            n(&self.validation_errors),
            n(&self.internal_errors),
            n(&self.out_of_memory),
            n(&self.surface_faults),
            n(&self.device_notices)
        )
    }
}
struct Inject {
    signals: Arc<Signals>,
    plan: Plan,
    device: wgpu::Device,
}
impl egui_wgpu::CallbackTrait for Inject {
    fn prepare(
        &self,
        device: &wgpu::Device,
        _: &wgpu::Queue,
        _: &egui_wgpu::ScreenDescriptor,
        _: &mut wgpu::CommandEncoder,
        _: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        self.signals.prepares.fetch_add(1, Ordering::AcqRel);
        if self.plan != Plan::Paint {
            self.signals.fire(device);
        }
        Vec::new()
    }
    fn paint(
        &self,
        _: egui::PaintCallbackInfo,
        _: &mut wgpu::RenderPass<'static>,
        _: &egui_wgpu::CallbackResources,
    ) {
        self.signals.paints.fetch_add(1, Ordering::AcqRel);
        if self.plan == Plan::Paint {
            self.signals.fire(&self.device);
        }
    }
}
pub(super) fn append(ui: &egui::Ui, signals: Arc<Signals>, plan: Plan, device: wgpu::Device) {
    let rect = egui::Rect::from_min_size(ui.max_rect().min, egui::vec2(8., 8.));
    ui.ctx()
        .layer_painter(egui::LayerId::new(
            egui::Order::Foreground,
            egui::Id::unique("owned-device-injection"),
        ))
        .add(egui_wgpu::Callback::new_paint_callback(
            rect,
            Inject {
                signals,
                plan,
                device,
            },
        ));
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pre_injection_errors_cannot_be_reclassified_as_expected_loss() {
        let s = Signals::default();
        s.record_error();
        s.fired.store(1, Ordering::Release);
        s.record_error();
        assert_eq!(s.errors.load(Ordering::Acquire), 2);
        assert_eq!(s.premature_errors.load(Ordering::Acquire), 1);
    }
    #[test]
    fn repeated_notices_do_not_change_injection_identity_or_clear_failure() {
        let s = Signals::default();
        s.notify();
        s.notify();
        assert_eq!(s.noticed.load(Ordering::Acquire), 2);
        assert_eq!(s.fired.load(Ordering::Acquire), 0);
        assert!(s.json().contains("\"fired\":0"));
    }
}
