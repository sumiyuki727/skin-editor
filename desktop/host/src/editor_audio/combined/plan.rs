//! Fixed normal workload, independent of a GUI clock or device.
pub(super) const SECONDS: u64 = 300;
pub(super) const COUNTS: [usize; 4] = [2400, 64, 300, 20];
#[derive(Default)]
pub(super) struct StageClock {
    pub stage: u8,
    entered: std::time::Duration,
}
impl StageClock {
    pub fn enter(&mut self, stage: u8, now: std::time::Duration) {
        self.stage = stage;
        self.entered = now;
    }
    pub fn age(&self, now: std::time::Duration) -> Result<std::time::Duration, &'static str> {
        now.checked_sub(self.entered).ok_or("stage clock reversed")
    }
    pub fn check(&self, now: std::time::Duration) -> Result<(), &'static str> {
        let age = self.age(now)?;
        if matches!(self.stage, 13 | 15) && age >= std::time::Duration::from_secs(1) {
            return Err("old voice cleanup exceeded guard before natural end");
        }
        if age > std::time::Duration::from_secs(30) && !matches!(self.stage, 3 | 20 | 21) {
            return Err("control/preparation stage expired");
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Event {
    Scene,
    Audition,
    Edit,
    Switch,
}
impl Event {
    pub(super) fn index(self) -> usize {
        match self {
            Self::Scene => 0,
            Self::Audition => 1,
            Self::Edit => 2,
            Self::Switch => 3,
        }
    }
}
#[derive(Default)]
pub(super) struct Plan {
    pub counts: [usize; 4],
    pub max_late_us: u64,
}
impl Plan {
    fn due(&self, e: Event) -> Option<u64> {
        let n = self.counts[e.index()];
        (n < COUNTS[e.index()]).then(|| match e {
            Event::Scene => n as u64 * 125_000,
            Event::Audition => 31_250 + n as u64 * 4_000_000,
            // Place commands between sound requests; don't invalidate a trigger
            // in the same GUI callback just to hit an arbitrary clock phase.
            Event::Edit => 562_500 + n as u64 * 1_000_000,
            Event::Switch => 1_062_500 + n as u64 * 10_000_000,
        })
    }
    pub fn next(&self) -> Option<(Event, u64)> {
        [Event::Scene, Event::Audition, Event::Edit, Event::Switch]
            .into_iter()
            .filter_map(|e| self.due(e).map(|at| (e, at)))
            .min_by_key(|(_, at)| *at)
    }
    pub fn take(&mut self, now_us: u64) -> Result<Option<Event>, &'static str> {
        let Some((event, at)) = self.next() else {
            return Ok(None);
        };
        if now_us < at {
            return Ok(None);
        }
        let late = now_us - at;
        if late > 125_000 {
            return Err("normal workload missed deadline; no catch-up burst");
        }
        self.max_late_us = self.max_late_us.max(late);
        self.counts[event.index()] += 1;
        Ok(Some(event))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    #[test]
    fn transition_after_a_full_playback_gets_a_fresh_control_deadline() {
        let mut c = StageClock::default();
        c.enter(3, Duration::from_secs(1));
        c.check(Duration::from_secs(301)).unwrap();
        c.enter(4, Duration::from_secs(301));
        c.check(Duration::from_secs(301)).unwrap();
        assert_eq!(c.age(Duration::from_secs(301)).unwrap(), Duration::ZERO);
        c.check(Duration::from_secs(331)).unwrap();
        assert!(c.check(Duration::from_secs(332)).is_err());
    }
    #[test]
    fn stale_time_is_rejected_and_each_control_transition_resets_watchdog() {
        let mut c = StageClock::default();
        for stage in (4..=16).filter(|s| !matches!(s, 13 | 15)) {
            let now = Duration::from_secs(300 + u64::from(stage) * 20);
            c.enter(stage, now);
            assert!(c.age(now - Duration::from_secs(1)).is_err());
            c.check(now + Duration::from_secs(29)).unwrap();
            assert!(c.check(now + Duration::from_secs(31)).is_err());
        }
    }
    #[test]
    fn cleanup_deadline_is_checked_before_a_ready_natural_end_can_pass() {
        let mut c = StageClock::default();
        for stage in [13, 15] {
            c.enter(stage, Duration::from_secs(300));
            c.check(Duration::from_millis(300999)).unwrap();
            assert!(c.check(Duration::from_secs(301)).is_err());
            assert!(c.check(Duration::from_secs(302)).is_err());
        }
    }
    #[test]
    fn full_three_hundred_seconds_has_exact_counts_and_no_phase_collisions() {
        let mut plan = Plan::default();
        let mut seen = [0; 4];
        let mut previous = None;
        while let Some((event, at)) = plan.next() {
            assert!(at < SECONDS * 1_000_000);
            assert!(previous.is_none_or(|p| p < at));
            assert_eq!(
                plan.take(at - u64::from(at != 0)).unwrap(),
                (at == 0).then_some(event)
            );
            if at != 0 {
                assert_eq!(plan.take(at).unwrap(), Some(event));
            }
            seen[event.index()] += 1;
            previous = Some(at);
        }
        assert_eq!(seen, COUNTS);
        assert_eq!(plan.counts, COUNTS);
        assert_eq!(plan.take(SECONDS * 1_000_000).unwrap(), None);
    }
    #[test]
    fn stalled_host_fails_without_consuming_an_event_or_bursting() {
        let mut p = Plan::default();
        assert!(p.take(125_001).is_err());
        assert_eq!(p.counts, [0; 4]);
        assert_eq!(p.next(), Some((Event::Scene, 0)));
    }
    #[test]
    fn bounded_jitter_preserves_all_operations_and_records_lateness() {
        let mut p = Plan::default();
        while let Some((event, at)) = p.next() {
            assert_eq!(p.take(at + 5_000).unwrap(), Some(event));
        }
        assert_eq!(p.counts, COUNTS);
        assert_eq!(p.max_late_us, 5_000);
    }
}
