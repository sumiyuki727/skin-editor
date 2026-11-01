//! Control handles retain only atomic metadata, never a PCM/ring lease.
use crate::output_fence::{OutputFence, OutputMark};
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU64, Ordering::SeqCst},
};
pub(super) struct State {
    pub closed: AtomicBool,
    desired: [AtomicU64; 3],
    applied: [AtomicU64; 3],
    ack: [AtomicU64; 3],
    next: AtomicU64,
    running: AtomicBool,
}
impl State {
    pub fn new(position: u64) -> Arc<Self> {
        Arc::new(Self {
            closed: AtomicBool::new(false),
            desired: std::array::from_fn(|_| AtomicU64::new(1)),
            applied: std::array::from_fn(|_| AtomicU64::new(1)),
            ack: std::array::from_fn(|_| AtomicU64::new(0)),
            next: AtomicU64::new(position),
            running: AtomicBool::new(false),
        })
    }
    fn read(fields: &[AtomicU64; 3]) -> OutputMark {
        OutputMark {
            skin: OutputFence([fields[0].load(SeqCst), fields[1].load(SeqCst)]),
            background: fields[2].load(SeqCst),
        }
    }
    pub fn desired(&self) -> OutputMark {
        Self::read(&self.desired)
    }
    pub fn applied(&self) -> OutputMark {
        Self::read(&self.applied)
    }
    pub fn apply(&self, mark: OutputMark) {
        for (field, value) in
            self.applied
                .iter()
                .zip([mark.skin.0[0], mark.skin.0[1], mark.background])
        {
            field.store(value, SeqCst);
        }
    }
    pub fn acknowledge(&self, desired: OutputMark, applied: OutputMark, next: u64, running: bool) {
        self.next.store(next, SeqCst);
        self.running.store(running, SeqCst);
        for (field, value) in
            self.ack
                .iter()
                .zip([desired.skin.0[0], desired.skin.0[1], applied.background])
        {
            field.store(value, SeqCst);
        }
    }
}
#[derive(Clone)]
pub(crate) struct Control(pub(super) Arc<State>);
impl Control {
    pub fn is_closed(&self) -> bool {
        self.0.closed.load(SeqCst)
    }
    pub fn close(&self) {
        self.0.closed.store(true, SeqCst);
    }
    pub fn mark(&self) -> OutputMark {
        self.0.desired()
    }
    pub fn invalidate(
        &self,
        scene: bool,
        audition: bool,
        background: bool,
    ) -> Result<OutputMark, String> {
        if self.is_closed() {
            return Err("separated output closed".into());
        }
        for (field, change) in self.0.desired.iter().zip([scene, audition, background]) {
            if change
                && field
                    .fetch_update(SeqCst, SeqCst, |v| v.checked_add(1))
                    .is_err()
            {
                self.close();
                return Err("separated control version exhausted".into());
            }
        }
        Ok(self.mark())
    }
    pub fn acknowledged(&self, mark: OutputMark) -> bool {
        self.0
            .ack
            .iter()
            .zip([mark.skin.0[0], mark.skin.0[1], mark.background])
            .all(|(a, v)| a.load(SeqCst) >= v)
    }
    pub fn background_acknowledged(&self, mark: OutputMark) -> bool {
        self.acknowledged(mark) && !self.0.running.load(SeqCst)
    }
    pub fn background_next(&self) -> u64 {
        self.0.next.load(SeqCst)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exhausted_control_closes_without_wrapping_or_reopening() {
        let c = Control(State::new(0));
        c.0.desired[0].store(u64::MAX, SeqCst);
        assert!(c.invalidate(true, true, false).is_err());
        assert_eq!(c.mark().skin.0[0], u64::MAX);
        assert!(c.is_closed());
        assert!(c.invalidate(false, true, false).is_err());
        assert_eq!(c.mark().skin.0[1], 1);
    }
}
