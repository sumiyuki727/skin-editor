//! Diagnostic only: requested repaint versus the next actual logic call.
pub(super) struct Cadence {
    // begin, tick end, logic end, repaint request, requested delay, UI begin/end.
    current: [u64; 7],
    pub calls: u64,
    pub late_count: u64,
    pub rows: Vec<[u64; 11]>,
}
impl Default for Cadence {
    fn default() -> Self {
        Self {
            current: [0; 7],
            calls: 0,
            late_count: 0,
            rows: Vec::with_capacity(64),
        }
    }
}
impl Cadence {
    pub fn begin(&mut self, now: u64) {
        self.calls += 1;
        let previous = self.current;
        self.current = [now, 0, 0, 0, 0, 0, 0];
        let target = previous[3].saturating_add(previous[4]);
        if previous[3] > 0 && now.saturating_sub(target) > 20_000_000 {
            self.late_count += 1;
            if self.rows.len() < 64 {
                let mut row = [0; 11];
                row[..7].copy_from_slice(&previous);
                row[7] = now;
                row[8] = now - target;
                self.rows.push(row);
            }
        }
    }
    pub fn tick_end(&mut self, now: u64) {
        self.current[1] = now;
        if let Some(row) = self.rows.last_mut().filter(|r| r[7] == self.current[0]) {
            row[9] = now;
        }
    }
    pub fn end(&mut self, now: u64) {
        self.current[2] = now;
        if let Some(row) = self.rows.last_mut().filter(|r| r[7] == self.current[0]) {
            row[10] = now;
        }
    }
    pub fn repaint(&mut self, now: u64, delay: u64) {
        self.current[3..5].copy_from_slice(&[now, delay]);
    }
    pub fn ui(&mut self, now: u64, end: bool) {
        self.current[if end { 6 } else { 5 }] = now;
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn late_wakeup_is_distinct_from_work_and_survives_early_failure() {
        let mut c = Cadence::default();
        c.begin(1);
        c.tick_end(2);
        c.repaint(3, 125_000_000);
        c.end(4);
        c.ui(5, false);
        c.ui(6, true);
        c.begin(545_500_003);
        c.tick_end(546_000_003);
        c.end(546_000_004);
        assert_eq!(
            c.rows[0],
            [
                1,
                2,
                4,
                3,
                125_000_000,
                5,
                6,
                545_500_003,
                420_500_000,
                546_000_003,
                546_000_004
            ]
        );
        assert_eq!(c.late_count, 1);
        for _ in 0..70 {
            c.repaint(1, 1);
            c.begin(100_000_000);
        }
        assert_eq!(c.rows.len(), 64);
        assert_eq!(c.late_count, 71);
    }
    #[test]
    fn on_time_and_unrequested_calls_do_not_invent_lateness() {
        let mut c = Cadence::default();
        c.begin(1);
        c.begin(50_000_000);
        c.repaint(50_000_001, 125_000_000);
        c.begin(175_000_001);
        assert_eq!(c.late_count, 0);
        assert!(c.rows.is_empty());
    }
}
