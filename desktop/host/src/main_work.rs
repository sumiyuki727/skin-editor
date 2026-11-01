//! Opt-in main-thread boundary observations, not a CPU counter. No FFI or
//! framework patches. Python correlates validated clock pairs with ETW.
use std::time::{Instant, SystemTime, UNIX_EPOCH};

pub const FRAMEWORK: u8 = 0;
pub const LOGIC: u8 = 1;
pub const UI: u8 = 2;
pub const DIAGNOSTIC: u8 = 3;
pub const PRESENT_AND_EVENTS: u8 = 4;
pub const WIDGETS: u8 = 5;
pub const GRAPHICS_POLL: u8 = 6;
pub const PREVIEW_REQUEST: u8 = 7;
pub const UI_DISPATCH: u8 = 8;
const LIMIT: usize = 32768;

pub fn ui_category(phase: skin_editor_ui::UiPhase) -> u8 {
    use skin_editor_ui::UiPhase;
    match phase {
        UiPhase::Prelude => WIDGETS,
        UiPhase::SceneControls => 9,
        UiPhase::LayersAssets => 10,
        UiPhase::PropertiesHistory => 11,
        UiPhase::Canvas => 12,
        UiPhase::AudioPanel => 13,
        UiPhase::SettingsDiagnostics => 14,
        UiPhase::ProjectSnapshot => 15,
        UiPhase::SceneWidgets => 16,
        UiPhase::SceneReadout => 17,
        UiPhase::AudioDevice => 18,
        UiPhase::AudioStatus => 19,
        UiPhase::AudioLibrary => 20,
        UiPhase::AudioComponents => 21,
        UiPhase::AudioProperties => 22,
        UiPhase::AudioReadiness => 23,
        UiPhase::AudioActions => 24,
    }
}

pub struct MainWork {
    origin: Instant,
    // [UTC nanoseconds, monotonic elapsed nanoseconds, entered category]
    rows: Vec<[u64; 3]>,
    invalid: u64,
}
impl MainWork {
    pub fn new() -> Self {
        Self {
            origin: Instant::now(),
            rows: Vec::with_capacity(LIMIT),
            invalid: 0,
        }
    }
    pub fn edge(&mut self, category: u8) {
        if self.rows.len() == LIMIT {
            self.invalid += 1;
            return;
        }
        let elapsed = self.origin.elapsed().as_nanos();
        let epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|t| t.as_nanos());
        if let Ok(epoch) = epoch
            && let (Ok(epoch), Ok(elapsed)) = (u64::try_from(epoch), u64::try_from(elapsed))
        {
            self.record([epoch, elapsed, u64::from(category)]);
            return;
        }
        self.invalid += 1;
    }
    fn record(&mut self, row: [u64; 3]) {
        if self.rows.len() == LIMIT
            || row[2] > 24
            || self
                .rows
                .last()
                .is_some_and(|p| row[0] < p[0] || row[1] < p[1])
        {
            self.invalid += 1;
        } else {
            self.rows.push(row);
        }
    }
    pub fn report(&self) -> String {
        format!(
            "{{\"schema\":4,\"clock\":\"SystemTime-UTC-ns-and-Instant-elapsed-ns\",\"invalid\":{},\"rows\":{:?}}}",
            self.invalid, self.rows
        )
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn boundaries_remain_ordered_and_reject_invalid_clock_or_category() {
        let mut w = MainWork::new();
        w.record([100, 10, 0]);
        w.record([110, 20, 1]);
        w.record([109, 21, 0]);
        w.record([120, 19, 0]);
        w.record([120, 30, 25]);
        assert_eq!(w.rows, [[100, 10, 0], [110, 20, 1]]);
        assert_eq!(w.invalid, 3);
    }
    #[test]
    fn detailed_boundaries_share_one_bounded_ordered_partition() {
        let mut w = MainWork::new();
        for (at, category) in [
            UI,
            WIDGETS,
            UI_DISPATCH,
            PREVIEW_REQUEST,
            UI_DISPATCH,
            FRAMEWORK,
        ]
        .into_iter()
        .enumerate()
        {
            w.record([100 + at as u64, at as u64, u64::from(category)]);
        }
        assert_eq!(w.invalid, 0);
        assert_eq!(w.rows.len(), 6);
        assert!(w.report().starts_with("{\"schema\":4,"));
    }
    #[test]
    fn ui_boundaries_cannot_alias_framework_or_host_dispatch_categories() {
        use skin_editor_ui::UiPhase;
        let phases = [
            UiPhase::SceneControls,
            UiPhase::LayersAssets,
            UiPhase::PropertiesHistory,
            UiPhase::Canvas,
            UiPhase::AudioPanel,
            UiPhase::SettingsDiagnostics,
            UiPhase::ProjectSnapshot,
            UiPhase::SceneWidgets,
            UiPhase::SceneReadout,
            UiPhase::AudioDevice,
            UiPhase::AudioStatus,
            UiPhase::AudioLibrary,
            UiPhase::AudioComponents,
            UiPhase::AudioProperties,
            UiPhase::AudioReadiness,
            UiPhase::AudioActions,
        ];
        let mapped: std::collections::BTreeSet<_> = phases.map(ui_category).into_iter().collect();
        assert_eq!(mapped.len(), phases.len());
        assert!(mapped.iter().all(|v| (9..=24).contains(v)));
        assert_eq!(ui_category(UiPhase::Prelude), WIDGETS);
    }
    #[test]
    fn capacity_is_fixed_and_overflow_is_visible() {
        let mut w = MainWork::new();
        for _ in 0..LIMIT + 1 {
            w.record([100, 10, 0]);
        }
        assert_eq!(w.rows.len(), LIMIT);
        assert_eq!(w.invalid, 1);
    }
}
