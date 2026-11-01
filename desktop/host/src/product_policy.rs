//! Ordinary desktop assembly. Diagnostic controls keep their explicit policies.
use crate::editor_audio::EditorAudio;
use skin_editor_infrastructure::BlobStore;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Policy {
    pub separated_audio: bool,
    pub memory_usage: bool,
}
impl Policy {
    const fn for_windows(windows: bool) -> Self {
        Self {
            separated_audio: windows,
            memory_usage: windows,
        }
    }
    pub const CURRENT: Self = Self::for_windows(cfg!(target_os = "windows"));

    pub fn audio(self, store: BlobStore) -> EditorAudio {
        if self.separated_audio {
            EditorAudio::candidate(store)
        } else {
            EditorAudio::new(store)
        }
    }
    pub fn graphics(self) -> eframe::egui_wgpu::WgpuConfiguration {
        crate::gpu_observation::configuration(self.memory_usage)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_the_validated_platform_uses_the_separated_route() {
        assert_eq!(
            Policy::for_windows(true),
            Policy {
                separated_audio: true,
                memory_usage: true
            }
        );
        assert_eq!(
            Policy::for_windows(false),
            Policy {
                separated_audio: false,
                memory_usage: false
            }
        );
        assert_eq!(
            Policy::CURRENT,
            Policy::for_windows(cfg!(target_os = "windows"))
        );
    }
    #[test]
    fn ordinary_assembly_changes_only_the_allocator_hint() {
        let actual = Policy::CURRENT.graphics();
        let control = crate::gpu_observation::configuration(false);
        assert_eq!(actual.surface.present_mode, control.surface.present_mode);
        let eframe::egui_wgpu::WgpuSetup::CreateNew(actual) = actual.wgpu_setup else {
            panic!("new device")
        };
        let eframe::egui_wgpu::WgpuSetup::CreateNew(control) = control.wgpu_setup else {
            panic!("new device")
        };
        assert_eq!(actual.power_preference, control.power_preference);
    }
}
