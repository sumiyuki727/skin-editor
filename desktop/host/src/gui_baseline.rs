//! Explicit diagnostic controls. No product setting, project write or audio
//! policy. All modes keep the same host/audio loop and bootstrap full layout.
use eframe::egui;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Mode {
    Framework,
    Presentation,
    Editor,
}
impl Mode {
    pub fn from_flag(flag: &str) -> Option<Self> {
        match flag {
            "--verify-framework-audio-gui" => Some(Self::Framework),
            "--verify-presentation-audio-gui" => Some(Self::Presentation),
            "--verify-editor-baseline-gui" => Some(Self::Editor),
            _ => None,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::Framework => "framework",
            Self::Presentation => "presentation",
            Self::Editor => "editor",
        }
    }
    pub fn full_ui(self, stage: u8) -> bool {
        self == Self::Editor || stage == 0
    }
}

pub fn viewport(mode: Option<Mode>) -> egui::ViewportBuilder {
    let viewport = egui::ViewportBuilder::default().with_inner_size([1180., 800.]);
    if mode.is_some() {
        // A matched background comparison, without stealing user focus. Actual
        // input/focus is still observed and can invalidate the whole plan.
        viewport.with_active(false).with_mouse_passthrough(true)
    } else {
        viewport
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Geometry {
    pub window: [f32; 2],
    pub pixels_per_point: f32,
    pub preview: [f32; 4],
    pub size: [u32; 2],
    pub generation: u64,
}
impl Geometry {
    pub fn new(
        window: egui::Vec2,
        pixels_per_point: f32,
        preview: egui::Rect,
        size: [u32; 2],
        generation: u64,
    ) -> Result<Self, &'static str> {
        let value = Self {
            window: [window.x, window.y],
            pixels_per_point,
            preview: [preview.min.x, preview.min.y, preview.max.x, preview.max.y],
            size,
            generation,
        };
        if value.window.iter().any(|v| !v.is_finite() || *v <= 0.)
            || !pixels_per_point.is_finite()
            || pixels_per_point <= 0.
            || value.preview.iter().any(|v| !v.is_finite())
            || preview.min.x < 0.
            || preview.min.y < 0.
            || preview.width() <= 0.
            || preview.height() <= 0.
            || preview.max.x > window.x
            || preview.max.y > window.y
            || size.iter().any(|v| *v == 0 || *v > 1024)
            || generation == 0
        {
            return Err("invalid control geometry");
        }
        Ok(value)
    }
    pub fn rect(self) -> egui::Rect {
        egui::Rect::from_min_max(
            egui::pos2(self.preview[0], self.preview[1]),
            egui::pos2(self.preview[2], self.preview[3]),
        )
    }
    fn report(self) -> String {
        format!(
            "{{\"window\":{:?},\"pixels_per_point\":{},\"preview\":{:?},\"size\":{:?},\"generation\":{}}}",
            self.window, self.pixels_per_point, self.preview, self.size, self.generation
        )
    }
}

pub struct Control {
    pub mode: Mode,
    pub geometry: Option<Geometry>,
    pub changed: bool,
    // Per stage: host UI calls, full editor UI calls, preview paints, minimal calls.
    pub stages: [[u64; 4]; 18],
}
impl Control {
    pub fn new(mode: Mode) -> Self {
        Self {
            mode,
            geometry: None,
            changed: false,
            stages: [[0; 4]; 18],
        }
    }
    pub fn observe(&mut self, geometry: Geometry) -> Result<(), &'static str> {
        if self.geometry.is_some_and(|old| old != geometry) {
            self.changed = true;
            return Err("control geometry changed during fixed run");
        }
        self.geometry = Some(geometry);
        Ok(())
    }
    pub fn record(&mut self, stage: u8, full: bool, painted: bool) {
        let counts = &mut self.stages[usize::from(stage)];
        counts[0] += 1;
        counts[1] += u64::from(full);
        counts[2] += u64::from(painted);
        counts[3] += u64::from(!full);
    }
    pub fn report(&self) -> String {
        format!(
            "{{\"schema\":3,\"inactive_on_create\":true,\"mouse_passthrough\":true,\"mode\":\"{}\",\"geometry\":{},\"geometry_changed\":{},\"stages\":{:?},\"audio_policy\":\"unchanged-streamed-runtime\",\"nominal_playing_repaint_ms\":16}}",
            self.mode.name(),
            self.geometry
                .map_or_else(|| "null".into(), Geometry::report),
            self.changed,
            self.stages
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn geometry() -> Geometry {
        Geometry::new(
            egui::vec2(1180., 800.),
            1.,
            egui::Rect::from_min_max(egui::pos2(260., 40.), egui::pos2(930., 790.)),
            [670, 750],
            1,
        )
        .unwrap()
    }
    #[test]
    fn diagnostic_modes_do_not_activate_on_normal_flags_and_prime_same_layout() {
        assert_eq!(Mode::from_flag("--candidate-audio"), None);
        assert_eq!(Mode::from_flag("--verify-main-work-gui"), None);
        for mode in [Mode::Framework, Mode::Presentation, Mode::Editor] {
            assert!(mode.full_ui(0));
            assert_eq!(mode.full_ui(4), mode == Mode::Editor);
        }
    }
    #[test]
    fn only_explicit_controls_request_inactive_creation_without_changing_normal_windows() {
        assert_eq!(viewport(None).active, None);
        assert_eq!(viewport(None).mouse_passthrough, None);
        for mode in [Mode::Framework, Mode::Presentation, Mode::Editor] {
            let v = viewport(Some(mode));
            assert_eq!(v.active, Some(false));
            assert_eq!(v.mouse_passthrough, Some(true));
            assert_eq!(v.inner_size, Some(egui::vec2(1180., 800.)));
        }
    }
    #[test]
    fn viewport_scale_size_and_generation_changes_are_rejected_not_normalized() {
        let original = geometry();
        for case in 0..5 {
            let mut c = Control::new(Mode::Framework);
            c.observe(original).unwrap();
            let mut next = original;
            match case {
                0 => next.window[0] += 1.,
                1 => next.pixels_per_point = 2.,
                2 => next.preview[0] += 1.,
                3 => next.size[0] += 1,
                _ => next.generation += 1,
            }
            assert!(c.observe(next).is_err());
            assert!(c.changed);
            assert_eq!(c.geometry, Some(original));
        }
    }
    #[test]
    fn no_draw_editor_and_minimal_counts_remain_separate() {
        let mut c = Control::new(Mode::Framework);
        c.record(0, true, true);
        c.record(4, false, false);
        c.record(4, false, false);
        assert_eq!(c.stages[0], [1, 1, 1, 0]);
        assert_eq!(c.stages[4], [2, 0, 0, 2]);
    }
    #[test]
    fn invalid_or_outside_geometry_cannot_become_a_control_baseline() {
        for (window, ppp, rect, size, generation) in [
            (egui::vec2(0., 800.), 1., geometry().rect(), [670, 750], 1),
            (
                egui::vec2(1180., 800.),
                f32::NAN,
                geometry().rect(),
                [670, 750],
                1,
            ),
            (egui::vec2(100., 100.), 1., geometry().rect(), [670, 750], 1),
            (egui::vec2(1180., 800.), 1., geometry().rect(), [0, 750], 1),
            (
                egui::vec2(1180., 800.),
                1.,
                geometry().rect(),
                [670, 750],
                0,
            ),
        ] {
            assert!(Geometry::new(window, ppp, rect, size, generation).is_err());
        }
    }
}
