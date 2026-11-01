//! Stack-scoped host demand merge. Does not replace egui's scheduler, intercept
//! external wakeups, retain a decision across passes, or control the audio worker.
use std::time::{Duration, Instant};

#[derive(Default)]
pub struct RepaintPass {
    pass: Option<u64>,
    published: Option<Instant>,
}
impl RepaintPass {
    // Always reset at logic entry, including logic-only calls without a UI pass.
    pub fn begin(&mut self, ctx: &eframe::egui::Context) {
        self.pass = Some(ctx.cumulative_pass_nr());
        self.published = None;
    }
    fn publish_at(&mut self, ctx: &eframe::egui::Context, cycle: RepaintCycle, now: Instant) {
        if self.pass != Some(ctx.cumulative_pass_nr()) {
            self.begin(ctx);
        }
        let Some(due) = cycle.due.map(|d| cycle.started + d) else {
            return;
        };
        if self.published.is_none_or(|previous| due < previous) {
            self.published = Some(due);
            ctx.request_repaint_after(due.saturating_duration_since(now));
        }
    }
}

pub struct RepaintCycle {
    started: Instant,
    due: Option<Duration>,
}
impl RepaintCycle {
    pub fn new() -> Self {
        Self {
            started: Instant::now(),
            due: None,
        }
    }
    pub fn after(&mut self, millis: u64) {
        self.offer_at(self.started.elapsed(), Duration::from_millis(millis));
    }
    fn offer_at(&mut self, elapsed: Duration, delay: Duration) {
        let due = elapsed.saturating_add(delay);
        self.due = Some(self.due.map_or(due, |previous| previous.min(due)));
    }
    #[cfg(test)]
    fn remaining_at(&self, elapsed: Duration) -> Option<Duration> {
        self.due.map(|due| due.saturating_sub(elapsed))
    }
    pub fn publish(self, ctx: &eframe::egui::Context, pass: &mut RepaintPass) {
        pass.publish_at(ctx, self, Instant::now());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui;
    use std::sync::{Arc, Mutex};
    fn ms(n: u64) -> Duration {
        Duration::from_millis(n)
    }

    #[test]
    fn selects_absolute_deadline_without_restarting_the_delay_at_publish() {
        let mut r = RepaintCycle::new();
        r.offer_at(ms(0), ms(100));
        r.offer_at(ms(1), ms(20));
        r.offer_at(ms(3), ms(16));
        r.offer_at(ms(15), ms(10)); // Later short delay still has a later deadline.
        assert_eq!(r.remaining_at(ms(4)), Some(ms(15)));
        assert_eq!(r.remaining_at(ms(19)), Some(Duration::ZERO));
        assert_eq!(r.remaining_at(ms(30)), Some(Duration::ZERO));
    }
    #[test]
    fn every_combination_keeps_the_original_earliest_policy_and_empty_stays_idle() {
        let delays = [100, 20, 16, 10];
        for mask in 0..16 {
            let mut r = RepaintCycle::new();
            for (i, d) in delays.into_iter().enumerate() {
                if mask & (1 << i) != 0 {
                    r.offer_at(ms(0), ms(d));
                }
            }
            let expected = delays
                .into_iter()
                .enumerate()
                .filter(|(i, _)| mask & (1 << i) != 0)
                .map(|(_, d)| ms(d))
                .min();
            assert_eq!(r.remaining_at(ms(0)), expected);
        }
    }
    fn mature() -> egui::Context {
        let ctx = egui::Context::default();
        for _ in 0..3 {
            let mut out = ctx.run_ui(Default::default(), |_| {});
            out.textures_delta.clear();
        }
        ctx
    }
    #[test]
    fn one_publication_reduces_actual_callbacks_without_replacing_external_callback() {
        let ctx = mature();
        let mut pass = RepaintPass::default();
        let calls = Arc::new(Mutex::new(Vec::new()));
        let seen = calls.clone();
        ctx.set_request_repaint_callback(move |r| {
            seen.lock().unwrap().push(r.delay);
        });
        let mut out = ctx.run_ui(Default::default(), |ui| {
            let mut r = RepaintCycle::new();
            r.after(20);
            r.after(16);
            r.after(16);
            r.publish(ui.ctx(), &mut pass);
            assert_eq!(calls.lock().unwrap().len(), 1);
            // A command's immediate external wakeup must still supersede it.
            ui.ctx().request_repaint();
            assert_eq!(calls.lock().unwrap().last(), Some(&Duration::ZERO));
        });
        out.textures_delta.clear();
    }
    #[test]
    fn paused_logic_cannot_suppress_post_command_work_or_next_pass_resume() {
        let ctx = mature();
        let mut pass = RepaintPass::default();
        let calls = Arc::new(Mutex::new(Vec::new()));
        let seen = calls.clone();
        ctx.set_request_repaint_callback(move |r| {
            if !r.delay.is_zero() {
                seen.lock().unwrap().push(r.delay);
            }
        });
        let base = Instant::now();
        for phase in 0..4 {
            let before = calls.lock().unwrap().len();
            let mut out = ctx.run_ui(Default::default(), |ui| {
                // Fresh logic cycle: phase 0/2 paused idle; phase 1/3 resume.
                let at = base + ms(phase as u64 * 100);
                let mut logic = RepaintCycle {
                    started: at,
                    due: None,
                };
                pass.begin(ui.ctx());
                if phase % 2 == 1 {
                    logic.offer_at(ms(0), ms(20));
                    logic.offer_at(ms(0), ms(16));
                }
                pass.publish_at(ui.ctx(), logic, at);
                // A paused command can create GPU work after logic, or shorten
                // an already published 16ms deadline. No cached "already sent".
                if phase < 2 {
                    let mut command = RepaintCycle {
                        started: at + ms(1),
                        due: None,
                    };
                    command.offer_at(ms(0), ms(20));
                    command.offer_at(ms(0), ms(10));
                    pass.publish_at(ui.ctx(), command, at + ms(1));
                }
            });
            out.textures_delta.clear();
            let values = calls.lock().unwrap();
            let expected = [1, 2, 0, 1][phase];
            assert_eq!(values.len() - before, expected);
            if expected > 0 {
                assert!(*values.last().unwrap() <= ms(if phase < 2 { 10 } else { 16 }));
            }
        }
    }
    #[test]
    fn absolute_pass_guard_ignores_later_ui_demand_but_accepts_new_earlier_work() {
        let ctx = mature();
        let calls = Arc::new(Mutex::new(Vec::new()));
        let seen = calls.clone();
        ctx.set_request_repaint_callback(move |r| {
            seen.lock().unwrap().push(r.delay);
        });
        let mut pass = RepaintPass::default();
        let base = Instant::now();
        let mut out = ctx.run_ui(Default::default(), |ui| {
            pass.begin(ui.ctx());
            let logic = RepaintCycle {
                started: base,
                due: Some(ms(16)),
            };
            pass.publish_at(ui.ctx(), logic, base);
            // Its RELATIVE remaining time is shorter (15ms), but its absolute
            // deadline is later. A new UI cycle must not post a second wakeup.
            let ui_readout = RepaintCycle {
                started: base + ms(2),
                due: Some(ms(16)),
            };
            pass.publish_at(ui.ctx(), ui_readout, base + ms(3));
            assert_eq!(*calls.lock().unwrap(), vec![ms(16)]);
            let command = RepaintCycle {
                started: base + ms(3),
                due: Some(ms(10)),
            };
            pass.publish_at(ui.ctx(), command, base + ms(3));
            assert_eq!(*calls.lock().unwrap(), vec![ms(16), ms(10)]);
        });
        out.textures_delta.clear();
    }
    #[test]
    fn pass_advance_and_explicit_logic_restart_both_reset_old_deadlines() {
        let ctx = mature();
        let mut pass = RepaintPass::default();
        let base = Instant::now();
        for i in 0..2 {
            let mut out = ctx.run_ui(Default::default(), |ui| {
                let cycle = RepaintCycle {
                    started: base + ms(i * 100),
                    due: Some(ms(20)),
                };
                pass.publish_at(ui.ctx(), cycle, base + ms(i * 100));
                assert_eq!(pass.published, Some(base + ms(i * 100 + 20)));
                pass.begin(ui.ctx()); // A logic-only cycle cannot retain it.
                assert!(pass.published.is_none());
                let cycle = RepaintCycle {
                    started: base + ms(i * 100),
                    due: Some(ms(16)),
                };
                pass.publish_at(ui.ctx(), cycle, base + ms(i * 100));
            });
            out.textures_delta.clear();
        }
    }
}
