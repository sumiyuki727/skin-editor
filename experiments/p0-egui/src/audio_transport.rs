//! Pure P0 simulation/input gate driven only by read-only audio observations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Initializing,
    Pending,
    Paused,
    Playing,
    Faulted,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    Backend,
    BackwardsClock,
    ClockMovedWhilePaused,
    UnexpectedAcknowledgement,
    ClockOverflow,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Intent {
    pub epoch: u64,
    pub request: u64,
    pub paused: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Relocate {
    pub epoch: u64,
    pub position_us: u64,
}
#[derive(Clone, Copy, Debug)]
pub struct AudioObservation {
    pub epoch: u64,
    pub request: u64,
    pub sequence: u64,
    pub position_us: u64,
    pub paused: bool,
    pub fault: bool,
}
#[derive(Clone, Copy, Debug)]
pub struct State {
    pub epoch: u64,
    pub phase: Phase,
    pub request: u64,
    pub sequence: u64,
    pub simulation_us: u64,
    pub accepted_inputs: u64,
    pub fault: Option<Fault>,
}
pub struct Transport {
    state: State,
    desired_paused: bool,
    focused: bool,
    interact: bool,
    held: [bool; 3],
    base_us: u64,
    origin_us: u64,
    needs_anchor: bool,
}
impl Default for Transport {
    fn default() -> Self {
        Self {
            state: State {
                epoch: 0,
                phase: Phase::Initializing,
                request: 0,
                sequence: 0,
                simulation_us: 0,
                accepted_inputs: 0,
                fault: None,
            },
            desired_paused: true,
            focused: true,
            interact: false,
            held: [false; 3],
            base_us: 0,
            origin_us: 0,
            needs_anchor: false,
        }
    }
}
impl Transport {
    pub fn state(&self) -> State {
        self.state
    }
    pub fn desired_paused(&self) -> bool {
        self.desired_paused
    }
    pub fn intent(&mut self, paused: bool) -> Intent {
        if self.state.phase == Phase::Faulted || self.needs_anchor {
            self.desired_paused = true;
            return Intent {
                epoch: self.state.epoch,
                request: self.state.request,
                paused: true,
            };
        }
        self.state.request += 1;
        self.state.phase = Phase::Pending;
        self.desired_paused = paused;
        Intent {
            epoch: self.state.epoch,
            request: self.state.request,
            paused,
        }
    }
    pub fn fail(&mut self, fault: Fault) {
        self.state.phase = Phase::Faulted;
        self.state.fault = Some(fault);
        self.desired_paused = true;
    }
    /// Explicit recovery starts a distinct, paused stream. It never resumes playback.
    pub fn recover(&mut self) -> Option<Relocate> {
        if self.state.phase != Phase::Faulted {
            return None;
        }
        let epoch = self.state.epoch.checked_add(1)?;
        self.base_us = self.state.simulation_us;
        self.needs_anchor = true;
        self.state.epoch = epoch;
        self.state.request = 0;
        self.state.sequence = 0;
        self.state.phase = Phase::Initializing;
        self.state.fault = None;
        self.desired_paused = true;
        // Preserve held keys: release and a fresh press are required after recovery.
        Some(Relocate {
            epoch,
            position_us: self.base_us,
        })
    }
    /// An explicit seek is accepted only while paused. Keep the visible old time
    /// until the new stream acknowledges pause; seeking is not a project edit.
    pub fn seek(&mut self, position_us: u64) -> Option<Relocate> {
        if self.state.phase != Phase::Paused {
            return None;
        }
        let epoch = self.state.epoch.checked_add(1)?;
        self.base_us = position_us;
        self.needs_anchor = true;
        self.state.epoch = epoch;
        self.state.request = 0;
        self.state.sequence = 0;
        self.state.phase = Phase::Initializing;
        self.state.fault = None;
        self.desired_paused = true;
        Some(Relocate { epoch, position_us })
    }
    pub fn observe(&mut self, o: AudioObservation) -> bool {
        if o.epoch != self.state.epoch
            || o.sequence <= self.state.sequence
            || self.state.phase == Phase::Faulted
        {
            return false;
        }
        if o.fault {
            self.fail(Fault::Backend);
            return true;
        }
        if o.request != self.state.request {
            return false;
        }
        if o.paused != self.desired_paused || (self.needs_anchor && !o.paused) {
            self.fail(Fault::UnexpectedAcknowledgement);
            return true;
        }
        let anchoring = self.needs_anchor;
        if anchoring {
            self.origin_us = o.position_us;
            self.needs_anchor = false;
        }
        let Some(delta) = o.position_us.checked_sub(self.origin_us) else {
            self.fail(Fault::BackwardsClock);
            return true;
        };
        let Some(position_us) = self.base_us.checked_add(delta) else {
            self.fail(Fault::ClockOverflow);
            return true;
        };
        if position_us < self.state.simulation_us && !anchoring {
            self.fail(Fault::BackwardsClock);
            return true;
        }
        if self.state.phase == Phase::Paused && o.paused && position_us != self.state.simulation_us
        {
            self.fail(Fault::ClockMovedWhilePaused);
            return true;
        }
        self.state.sequence = o.sequence;
        self.state.simulation_us = position_us;
        self.state.phase = if o.paused {
            Phase::Paused
        } else {
            Phase::Playing
        };
        true
    }
    pub fn focus(&mut self, focused: bool) {
        self.focused = focused;
        if !focused {
            self.held = [false; 3];
        }
    }
    pub fn tool(&mut self, interact: bool) {
        self.interact = interact;
    }
    pub fn key(
        &mut self,
        index: usize,
        pressed: bool,
        repeat: bool,
        editor_consumed: bool,
    ) -> bool {
        let Some(held) = self.held.get_mut(index) else {
            return false;
        };
        let was_held = *held;
        *held = pressed;
        let accepted = pressed
            && !was_held
            && !repeat
            && !editor_consumed
            && self.focused
            && self.interact
            && self.state.phase == Phase::Playing;
        if accepted {
            self.state.accepted_inputs += 1;
        }
        accepted
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn event(request: u64, sequence: u64, position_us: u64, paused: bool) -> AudioObservation {
        AudioObservation {
            epoch: 0,
            request,
            sequence,
            position_us,
            paused,
            fault: false,
        }
    }
    #[test]
    fn only_fresh_acknowledged_device_samples_advance_simulation() {
        let mut t = Transport::default();
        t.tool(true);
        t.observe(event(0, 1, 0, true));
        let start = t.intent(false);
        assert!(!t.key(0, true, false, false));
        t.key(0, false, false, false);
        assert!(!t.observe(event(0, 2, 100, false)));
        assert_eq!(t.state().simulation_us, 0);
        assert!(t.observe(event(start.request, 3, 100, false)));
        assert!(t.key(0, true, false, false));
        assert!(!t.observe(event(start.request, 2, 90, false)));
        assert_eq!(t.state().simulation_us, 100);
        let pause = t.intent(true);
        assert!(!t.key(1, true, false, false));
        t.observe(event(pause.request, 4, 120, true));
        t.observe(event(pause.request, 5, 120, true));
        assert_eq!(t.state().simulation_us, 120);
        let resume = t.intent(false);
        t.observe(event(resume.request, 6, 120, false));
        t.observe(event(resume.request, 7, 130, false));
        assert_eq!(t.state().simulation_us, 130);
        // Held presses and repeats from before/during pause are never replayed.
        assert!(!t.key(0, true, true, false));
        assert!(!t.key(1, true, false, false));
        t.key(0, false, false, false);
        assert!(t.key(0, true, false, false));
        assert_eq!(t.state().accepted_inputs, 2);
    }
    #[test]
    fn superseded_pause_resume_ack_cannot_reopen_input_gate() {
        let mut t = Transport::default();
        t.tool(true);
        t.observe(event(0, 1, 0, true));
        t.intent(false);
        let pause = t.intent(true);
        assert!(!t.observe(event(1, 2, 20, false)));
        assert!(!t.key(0, true, false, false));
        assert!(t.observe(event(pause.request, 3, 20, true)));
        assert_eq!(t.state().phase, Phase::Paused);
    }
    #[test]
    fn matching_request_with_wrong_state_is_a_fault_not_a_resume() {
        let mut t = Transport::default();
        let start = t.intent(false);
        t.observe(event(start.request, 1, 0, true));
        assert_eq!(t.state().fault, Some(Fault::UnexpectedAcknowledgement));
    }
    #[test]
    fn editor_focus_tool_and_key_repeats_are_isolated() {
        let mut t = Transport::default();
        t.intent(false);
        t.observe(event(1, 1, 10, false));
        assert!(!t.key(0, true, false, false));
        t.key(0, false, false, false);
        t.tool(true);
        assert!(!t.key(0, true, false, true));
        t.key(0, false, false, false);
        t.focus(false);
        assert!(!t.key(0, true, false, false));
        t.focus(true);
        assert!(!t.key(0, true, true, false));
        t.key(0, false, false, false);
        assert!(t.key(0, true, false, false));
        assert!(!t.key(0, true, false, false));
        assert!(!t.key(9, true, false, false));
    }
    #[test]
    fn clock_regression_pause_drift_and_backend_failure_close_input() {
        for fault in [
            Fault::BackwardsClock,
            Fault::ClockMovedWhilePaused,
            Fault::Backend,
        ] {
            let mut t = Transport::default();
            t.tool(true);
            t.observe(event(0, 1, 20, true));
            let mut sample = event(0, 2, 20, true);
            match fault {
                Fault::BackwardsClock => sample.position_us = 19,
                Fault::ClockMovedWhilePaused => sample.position_us = 21,
                Fault::Backend => sample.fault = true,
                Fault::UnexpectedAcknowledgement | Fault::ClockOverflow => unreachable!(),
            }
            t.observe(sample);
            assert_eq!(t.state().fault, Some(fault));
            assert!(!t.key(0, true, false, false));
            let stale = t.intent(false);
            t.observe(event(stale.request, 3, 30, false));
            assert_eq!(t.state().phase, Phase::Faulted);
        }
    }
    #[test]
    fn recovery_preserves_time_rejects_old_stream_and_requires_explicit_resume() {
        let mut t = Transport::default();
        t.tool(true);
        assert_eq!(t.recover(), None);
        let play = t.intent(false);
        t.observe(event(play.request, 1, 400_000, false));
        assert!(t.key(0, true, false, false));
        t.fail(Fault::Backend);
        let epoch = t.recover().unwrap().epoch;
        let premature = t.intent(false);
        assert!(premature.paused);
        assert_eq!((premature.epoch, premature.request), (epoch, 0));
        assert_eq!(t.state().phase, Phase::Initializing);
        let sample = |sequence, position_us, paused| AudioObservation {
            epoch,
            request: 0,
            sequence,
            position_us,
            paused,
            fault: false,
        };
        let mut stale_fault = event(0, 99, 9_000_000, false);
        stale_fault.fault = true;
        assert!(!t.observe(stale_fault));
        assert!(t.observe(sample(1, 50, true)));
        assert_eq!(t.state().simulation_us, 400_000);
        assert_eq!(t.state().phase, Phase::Paused);
        t.observe(sample(2, 50, true));
        assert!(!t.key(1, true, false, false));
        let play = t.intent(false);
        let mut resumed = sample(3, 50, false);
        resumed.request = play.request;
        t.observe(resumed);
        assert_eq!(t.state().simulation_us, 400_000);
        assert!(!t.key(0, true, false, false));
        assert!(!t.key(1, true, true, false));
        t.key(0, false, false, false);
        assert!(t.key(0, true, false, false));
        resumed.sequence = 4;
        resumed.position_us = 250;
        t.observe(resumed);
        assert_eq!(t.state().simulation_us, 400_200);
        assert_eq!(t.state().accepted_inputs, 2);
    }
    #[test]
    fn failed_recovery_can_retry_without_advancing_or_accepting_stale_faults() {
        let mut t = Transport::default();
        t.observe(event(0, 1, 123, true));
        t.fail(Fault::Backend);
        let epoch = t.recover().unwrap().epoch;
        let failed = AudioObservation {
            epoch,
            fault: true,
            ..event(0, 1, 0, true)
        };
        t.observe(failed);
        assert_eq!(t.state().phase, Phase::Faulted);
        assert_eq!(t.state().simulation_us, 123);
        let next = t.recover().unwrap().epoch;
        assert!(!t.observe(failed));
        t.observe(AudioObservation {
            epoch: next,
            ..event(0, 1, 0, true)
        });
        assert_eq!(t.state().simulation_us, 123);
        assert_eq!(t.state().phase, Phase::Paused);
        // A duplicate old fault in this stream must not disable a healthy output.
        t.observe(AudioObservation {
            epoch: next,
            fault: true,
            ..event(0, 1, 0, true)
        });
        assert_eq!(t.state().phase, Phase::Paused);
    }
    #[test]
    fn recovery_rejects_playing_first_ack_and_checked_clock_overflow() {
        let mut t = Transport::default();
        t.observe(event(0, 1, u64::MAX - 1, true));
        t.fail(Fault::Backend);
        let epoch = t.recover().unwrap().epoch;
        t.observe(AudioObservation {
            epoch,
            ..event(0, 1, 0, false)
        });
        assert_eq!(t.state().fault, Some(Fault::UnexpectedAcknowledgement));
        let epoch = t.recover().unwrap().epoch;
        t.observe(AudioObservation {
            epoch,
            ..event(0, 1, 0, true)
        });
        t.intent(false);
        t.observe(AudioObservation {
            epoch,
            ..event(1, 2, 2, false)
        });
        assert_eq!(t.state().fault, Some(Fault::ClockOverflow));
        assert_eq!(t.state().simulation_us, u64::MAX - 1);
    }
    #[test]
    fn explicit_paused_seek_can_rewind_only_after_new_epoch_ack_without_replaying_held_keys() {
        let mut t = Transport::default();
        t.tool(true);
        t.intent(false);
        t.observe(event(1, 1, 900000, false));
        assert!(t.key(0, true, false, false));
        assert!(t.seek(50000).is_none());
        t.intent(true);
        t.observe(event(2, 2, 950000, true));
        let target = t.seek(50000).unwrap();
        assert_eq!(target.position_us, 50000);
        assert_eq!(t.state().simulation_us, 950000);
        assert!(t.seek(100000).is_none());
        assert!(t.intent(false).paused);
        assert!(!t.observe(event(2, 999, 990000, false)));
        t.observe(AudioObservation {
            epoch: target.epoch,
            ..event(0, 1, 0, true)
        });
        assert_eq!(t.state().simulation_us, 50000);
        assert_eq!(t.state().phase, Phase::Paused);
        t.intent(false);
        t.observe(AudioObservation {
            epoch: target.epoch,
            ..event(1, 2, 10000, false)
        });
        assert_eq!(t.state().simulation_us, 60000);
        assert!(!t.key(0, true, false, false));
        t.key(0, false, false, false);
        assert!(t.key(0, true, false, false));
        t.observe(AudioObservation {
            epoch: target.epoch,
            ..event(1, 3, 9999, false)
        });
        assert_eq!(t.state().fault, Some(Fault::BackwardsClock));
    }
    #[test]
    fn failed_seek_keeps_preseek_time_and_recovery_anchors_that_confirmed_position() {
        let mut t = Transport::default();
        t.observe(event(0, 1, 800000, true));
        let target = t.seek(200000).unwrap();
        t.observe(AudioObservation {
            epoch: target.epoch,
            fault: true,
            ..event(0, 1, 0, true)
        });
        assert_eq!(t.state().simulation_us, 800000);
        let epoch = t.recover().unwrap().epoch;
        t.observe(AudioObservation {
            epoch,
            ..event(0, 1, 0, true)
        });
        assert_eq!(t.state().simulation_us, 800000);
        assert_eq!(t.state().phase, Phase::Paused);
    }
}
