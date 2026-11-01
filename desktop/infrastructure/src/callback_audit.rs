//! Safe, constant TLS audit of managed object destruction in our data callback.
//! Does not intercept the global allocator or CPAL/driver internals.
use std::cell::Cell;
#[derive(Clone, Copy, Default)]
struct State {
    depth: u32,
    releases: u64,
}
thread_local! {static STATE:Cell<State>=const {Cell::new(State{depth:0,releases:0})};}
pub(crate) struct Scope {
    previous: State,
    finished: bool,
}
pub(crate) fn active() -> bool {
    STATE.with(|s| s.get().depth > 0)
}
pub(crate) fn released() {
    STATE.with(|s| {
        let mut v = s.get();
        if v.depth > 0 {
            v.releases = v.releases.saturating_add(1);
            s.set(v);
        }
    });
}
impl Scope {
    pub fn enter() -> Self {
        let previous = STATE.with(|s| {
            let v = s.get();
            s.set(State {
                depth: v.depth.saturating_add(1),
                releases: 0,
            });
            v
        });
        Self {
            previous,
            finished: false,
        }
    }
    pub fn finish(mut self) -> u64 {
        let count = STATE.with(|s| {
            let v = s.get();
            s.set(State {
                releases: self.previous.releases.saturating_add(v.releases),
                ..self.previous
            });
            v.releases
        });
        self.finished = true;
        count
    }
}
impl Drop for Scope {
    fn drop(&mut self) {
        if !self.finished {
            STATE.with(|s| s.set(self.previous));
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn resource_drops_are_scoped_and_other_owner_threads_do_not_create_false_callback_violations() {
        released();
        let s = Scope::enter();
        std::thread::spawn(released).join().unwrap();
        assert_eq!(s.finish(), 0);
        let s = Scope::enter();
        released();
        released();
        assert_eq!(s.finish(), 2);
        assert!(!active());
    }
    #[test]
    fn unwind_restores_the_previous_thread_scope() {
        let _ = std::panic::catch_unwind(|| {
            let _s = Scope::enter();
            panic!("controlled");
        });
        assert!(!active());
    }
}
