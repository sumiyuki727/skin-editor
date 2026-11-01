//! Measurement boundaries only; never a product timer or a repaint limiter.
use std::time::Duration;

pub const SETTLE: Duration = Duration::from_secs(10);
pub const MEASURE: Duration = Duration::from_secs(65);

#[derive(Clone, Copy)]
pub struct Interval {
    start: Duration,
    draws: u64,
    inputs: u64,
    simulation: u64,
    background: u64,
}
pub struct Observation {
    pub elapsed: Duration,
    pub draws: u64,
    pub inputs: u64,
    pub frozen: bool,
}
impl Interval {
    pub fn begin(
        now: Duration,
        settled_since: Duration,
        draws: u64,
        inputs: u64,
        simulation: u64,
        background: u64,
    ) -> Result<Self, &'static str> {
        if now.checked_sub(settled_since).is_none_or(|d| d < SETTLE) {
            return Err("settling window incomplete");
        }
        Ok(Self {
            start: now,
            draws,
            inputs,
            simulation,
            background,
        })
    }
    pub fn finish(
        self,
        now: Duration,
        draws: u64,
        inputs: u64,
        simulation: u64,
        background: u64,
    ) -> Result<Observation, &'static str> {
        let elapsed = now.checked_sub(self.start).ok_or("clock reversal")?;
        if elapsed < MEASURE {
            return Err("measurement window incomplete");
        }
        Ok(Observation {
            elapsed,
            draws: draws
                .checked_sub(self.draws)
                .ok_or("draw counter reversed")?,
            inputs: inputs
                .checked_sub(self.inputs)
                .ok_or("input counter reversed")?,
            frozen: simulation == self.simulation && background == self.background,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn seconds(n: u64) -> Duration {
        Duration::from_secs(n)
    }
    #[test]
    fn settling_is_required_and_deadline_is_inclusive() {
        assert!(Interval::begin(seconds(9), Duration::ZERO, 0, 0, 0, 0).is_err());
        assert!(Interval::begin(seconds(1), seconds(2), 0, 0, 0, 0).is_err());
        assert!(Interval::begin(seconds(10), Duration::ZERO, 0, 0, 0, 0).is_ok());
    }
    #[test]
    fn full_window_is_required_without_cherry_picking_a_short_tail() {
        let i = Interval::begin(seconds(10), Duration::ZERO, 5, 7, 11, 13).unwrap();
        assert!(i.finish(seconds(74), 5, 7, 11, 13).is_err());
        let r = i.finish(seconds(75), 7, 7, 11, 13).unwrap();
        assert_eq!(r.elapsed, seconds(65));
        assert_eq!(r.draws, 2);
        assert_eq!(r.inputs, 0);
        assert!(r.frozen);
    }
    #[test]
    fn extra_draws_input_or_moving_clocks_remain_visible() {
        let i = Interval::begin(seconds(10), Duration::ZERO, 2, 0, 0, 0).unwrap();
        let r = i.finish(seconds(75), 5, 1, 0, 1).unwrap();
        assert_eq!(r.draws, 3);
        assert_eq!(r.inputs, 1);
        assert!(!r.frozen);
        assert!(!i.finish(seconds(75), 2, 0, 1, 0).unwrap().frozen);
    }
    #[test]
    fn counter_reversals_are_not_saturated_to_zero() {
        let i = Interval::begin(seconds(10), Duration::ZERO, 2, 3, 0, 0).unwrap();
        assert!(i.finish(seconds(75), 1, 3, 0, 0).is_err());
        assert!(i.finish(seconds(75), 2, 2, 0, 0).is_err());
        assert!(i.finish(seconds(9), 2, 3, 0, 0).is_err());
    }
}
