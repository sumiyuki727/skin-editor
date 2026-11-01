//! Explicit diagnostics only. Forward every official handler unchanged; no
//! custom message pump, repaint, control-flow, priority or timer policy.
use super::Trace;
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, DeviceId, StartCause, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    window::WindowId,
};
struct Observed<T> {
    inner: T,
    trace: Trace,
}
impl<T: ApplicationHandler<eframe::UserEvent>> ApplicationHandler<eframe::UserEvent>
    for Observed<T>
{
    fn new_events(&mut self, e: &ActiveEventLoop, cause: StartCause) {
        let _scope = self.trace.handler(19);
        self.inner.new_events(e, cause);
    }
    fn about_to_wait(&mut self, e: &ActiveEventLoop) {
        let _scope = self.trace.handler(20);
        self.inner.about_to_wait(e);
    }
    fn window_event(&mut self, e: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        let _scope = self.trace.handler(21);
        self.inner.window_event(e, id, event);
    }
    fn user_event(&mut self, e: &ActiveEventLoop, event: eframe::UserEvent) {
        let _scope = self.trace.handler(22);
        self.inner.user_event(e, event);
    }
    fn device_event(&mut self, e: &ActiveEventLoop, id: DeviceId, event: DeviceEvent) {
        let _scope = self.trace.handler(23);
        self.inner.device_event(e, id, event);
    }
    fn resumed(&mut self, e: &ActiveEventLoop) {
        let _scope = self.trace.handler(24);
        self.inner.resumed(e);
    }
    fn suspended(&mut self, e: &ActiveEventLoop) {
        let _scope = self.trace.handler(24);
        self.inner.suspended(e);
    }
    fn exiting(&mut self, e: &ActiveEventLoop) {
        let _scope = self.trace.handler(24);
        self.inner.exiting(e);
    }
    fn memory_warning(&mut self, e: &ActiveEventLoop) {
        let _scope = self.trace.handler(24);
        self.inner.memory_warning(e);
    }
}
pub fn run_native(
    name: &str,
    mut options: eframe::NativeOptions,
    creator: eframe::AppCreator<'_>,
) -> eframe::Result {
    use winit::platform::run_on_demand::EventLoopExtRunOnDemand;
    let trace = super::current().expect("explicit thread-scoped diagnostic required");
    let mut builder = EventLoop::<eframe::UserEvent>::with_user_event();
    if let Some(hook) = options.event_loop_builder.take() {
        hook(&mut builder);
    }
    let mut event_loop = builder.build()?;
    let inner = eframe::create_native(name, options, creator, &event_loop);
    event_loop.run_app_on_demand(&mut Observed { inner, trace })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn equal_repaint_delays_are_coalesced_but_earlier_delays_still_send_callbacks() {
        use std::{
            sync::{Arc, Mutex},
            time::Duration,
        };
        fn probe(delays: &[u64]) -> (Vec<Duration>, Duration) {
            let ctx = eframe::egui::Context::default();
            for _ in 0..3 {
                let mut out = ctx.run_ui(Default::default(), |_| {});
                out.textures_delta.clear();
            }
            let calls = Arc::new(Mutex::new(Vec::new()));
            let seen = calls.clone();
            ctx.set_request_repaint_callback(move |r| {
                if r.delay > Duration::ZERO {
                    seen.lock().unwrap().push(r.delay);
                }
            });
            let mut out = ctx.run_ui(Default::default(), |ui| {
                for ms in delays {
                    ui.ctx().request_repaint_after(Duration::from_millis(*ms));
                }
            });
            let delay = out.viewport_output[&eframe::egui::ViewportId::ROOT].repaint_delay;
            out.textures_delta.clear();
            let values = calls.lock().unwrap().clone();
            (values, delay)
        }
        let (old_calls, old_delay) = probe(&[20, 16, 16]);
        let (minimum_calls, minimum_delay) = probe(&[16]);
        assert_eq!(
            old_calls,
            vec![Duration::from_millis(20), Duration::from_millis(16)]
        );
        assert_eq!(minimum_calls, vec![Duration::from_millis(16)]);
        assert_eq!(old_delay, minimum_delay);
    }
    #[test]
    fn handler_boundaries_are_explicit_bounded_and_released() {
        let trace = Trace::new();
        trace.stage(4);
        for kind in 19..=24 {
            drop(trace.handler(kind));
        }
        let d = trace.0.data.lock().unwrap();
        assert_eq!(d.rows.len(), 6);
        assert_eq!(d.invalid, 0);
        assert!(d.slots.iter().all(|s| s.refs == 0 && s.entered.is_none()));
        assert_eq!(
            d.rows.iter().map(|r| r[1]).collect::<Vec<_>>(),
            (19..=24).collect::<Vec<_>>()
        );
    }
}
