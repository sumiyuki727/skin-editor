use std::time::Duration;
#[derive(Clone, Copy)]
pub(super) struct Mode {
    pub smoke: bool,
}
impl Mode {
    pub fn rounds(self) -> usize {
        if self.smoke { 2 } else { 20 }
    }
    pub fn counts(self) -> [usize; 3] {
        if self.smoke { [4, 4, 2] } else { [32, 32, 16] }
    }
    pub fn play(self) -> Duration {
        Duration::from_millis(if self.smoke { 1200 } else { 6000 })
    }
    pub fn settle(self) -> Duration {
        Duration::from_millis(if self.smoke { 200 } else { 4000 })
    }
    pub fn baseline(self) -> Duration {
        Duration::from_millis(if self.smoke { 1000 } else { 4000 })
    }
    pub fn final_settle(self) -> Duration {
        Duration::from_millis(if self.smoke { 200 } else { 10000 })
    }
    pub fn final_observe(self) -> Duration {
        Duration::from_millis(if self.smoke { 1500 } else { 65000 })
    }
}
pub(super) fn next(mode: Mode, counts: [usize; 3]) -> Option<(usize, u64)> {
    (0..3)
        .filter(|i| counts[*i] < mode.counts()[*i])
        .map(|i| {
            (
                i,
                match i {
                    0 => counts[i] as u64 * 160_000,
                    1 => 80_000 + counts[i] as u64 * 160_000,
                    _ => 40_000 + counts[i] as u64 * 320_000,
                },
            )
        })
        .min_by_key(|(_, t)| *t)
}
pub(super) fn released(
    usage: [usize; 5],
    workers: [bool; 3],
    endpoint_returns: u64,
    background_bytes: usize,
) -> bool {
    usage == [0; 5] && workers == [true; 3] && endpoint_returns == 1 && background_bytes == 0
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn full_and_smoke_schedules_are_distinct_bounded_and_finish_inside_playback() {
        for smoke in [false, true] {
            let mode = Mode { smoke };
            let mut counts = [0; 3];
            let mut last = None;
            while let Some((lane, t)) = next(mode, counts) {
                assert!(last.is_none_or(|p| p < t));
                assert!(u128::from(t) < mode.play().as_micros());
                counts[lane] += 1;
                last = Some(t);
            }
            assert_eq!(counts, mode.counts());
            assert_eq!(mode.rounds(), if smoke { 2 } else { 20 });
        }
    }
    #[test]
    fn a_missing_join_or_any_live_lease_prevents_the_next_resource_cycle() {
        assert!(released([0; 5], [true; 3], 1, 0));
        for i in 0..5 {
            let mut u = [0; 5];
            u[i] = 1;
            assert!(!released(u, [true; 3], 1, 0));
        }
        for i in 0..3 {
            let mut w = [true; 3];
            w[i] = false;
            assert!(!released([0; 5], w, 1, 0));
        }
        assert!(!released([0; 5], [true; 3], 0, 0));
        assert!(!released([0; 5], [true; 3], 2, 0));
        assert!(!released([0; 5], [true; 3], 1, 8192));
    }
}
