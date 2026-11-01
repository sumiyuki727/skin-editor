//! Optional host diagnostic. The observer never replaces eframe's scheduler,
//! calls Context, waits on a lock, or grows storage in a requesting thread.
use eframe::egui;
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
};
use std::time::{SystemTime, UNIX_EPOCH};

const REQUEST_LIMIT: usize = 32768;
const PASS_LIMIT: usize = 4096;
const SOURCE_LIMIT: usize = 128;
const CAUSES_PER_PASS: usize = 64;
fn utc() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|v| u64::try_from(v.as_nanos()).ok())
        .unwrap_or(0)
}
struct Slot {
    at: AtomicU64,
    pass: AtomicU64,
    delay: AtomicU64,
    ready: AtomicBool,
}
struct Requests {
    next: AtomicUsize,
    rows: Box<[Slot]>,
}
impl Requests {
    fn new(limit: usize) -> Self {
        Self {
            next: AtomicUsize::new(0),
            rows: (0..limit)
                .map(|_| Slot {
                    at: AtomicU64::new(0),
                    pass: AtomicU64::new(0),
                    delay: AtomicU64::new(0),
                    ready: AtomicBool::new(false),
                })
                .collect(),
        }
    }
    fn push(&self, at: u64, pass: u64, delay: u64) {
        let i = self.next.fetch_add(1, Ordering::Relaxed);
        if let Some(row) = self.rows.get(i) {
            row.at.store(at, Ordering::Relaxed);
            row.pass.store(pass, Ordering::Relaxed);
            row.delay.store(delay, Ordering::Relaxed);
            row.ready.store(true, Ordering::Release);
        }
    }
    fn snapshot(&self) -> (Vec<[u64; 3]>, usize, usize) {
        let count = self.next.load(Ordering::Acquire);
        let mut rows = Vec::with_capacity(count.min(self.rows.len()));
        let mut pending = 0;
        for r in self.rows.iter().take(count) {
            if r.ready.load(Ordering::Acquire) {
                rows.push([
                    r.at.load(Ordering::Relaxed),
                    r.pass.load(Ordering::Relaxed),
                    r.delay.load(Ordering::Relaxed),
                ]);
            } else {
                pending += 1;
            }
        }
        (rows, count.saturating_sub(self.rows.len()), pending)
    }
}
struct Source {
    file: String,
    line: u32,
}
struct Pass {
    at: u64,
    pass: u64,
    frame: u64,
    causes: Vec<usize>,
}
pub struct Repaints {
    requests: Arc<Requests>,
    sources: Vec<Source>,
    passes: Vec<Pass>,
    inputs: Vec<[u64; 9]>,
    invalid: u64,
}
fn source_name(file: &str) -> String {
    // Compile-time locations only; never persist an absolute developer path.
    let file = file.replace('\\', "/");
    file.split('/')
        .rev()
        .take(3)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join("/")
}
impl Repaints {
    pub fn install(ctx: &egui::Context) -> Self {
        let requests = Arc::new(Requests::new(REQUEST_LIMIT));
        let observer = requests.clone();
        ctx.set_repaint_observer(move |info| {
            observer.push(
                utc(),
                info.current_cumulative_pass_nr,
                info.delay.as_nanos().min(u128::from(u64::MAX)) as u64,
            );
        });
        Self {
            requests,
            sources: Vec::with_capacity(SOURCE_LIMIT),
            passes: Vec::with_capacity(PASS_LIMIT),
            inputs: Vec::with_capacity(PASS_LIMIT),
            invalid: 0,
        }
    }
    fn source(&mut self, file: &str, line: u32) -> Option<usize> {
        let file = source_name(file);
        if let Some(i) = self
            .sources
            .iter()
            .position(|s| s.file == file && s.line == line)
        {
            return Some(i);
        }
        if self.sources.len() == SOURCE_LIMIT || file.len() > 160 {
            self.invalid += 1;
            return None;
        }
        self.sources.push(Source { file, line });
        Some(self.sources.len() - 1)
    }
    pub fn pass(&mut self, ctx: &egui::Context) {
        let pass = ctx.cumulative_pass_nr();
        if self.passes.last().is_some_and(|p| p.pass == pass) {
            return;
        }
        if self.passes.len() == PASS_LIMIT {
            self.invalid += 1;
            return;
        }
        // Public API returns previous-pass causes, not the cause of each
        // individual observer event; keep those identities distinct.
        let causes = ctx.repaint_causes();
        if causes.len() > CAUSES_PER_PASS {
            self.invalid += 1;
            return;
        }
        let ids = causes
            .iter()
            .filter_map(|c| self.source(c.file, c.line))
            .collect();
        self.passes.push(Pass {
            at: utc(),
            pass,
            frame: ctx.cumulative_frame_nr(),
            causes: ids,
        });
    }
    pub fn input(&mut self, ctx: &egui::Context, input: &egui::RawInput) {
        if self.inputs.len() == PASS_LIMIT {
            self.invalid += 1;
            return;
        }
        let mut counts = [0; 9];
        counts[0] = utc();
        counts[1] = ctx.cumulative_pass_nr();
        counts[8] = u64::from(input.focused);
        for event in &input.events {
            let i = match event {
                egui::Event::PointerMoved(_) => 2,
                egui::Event::MouseMoved(_) => 3,
                egui::Event::MouseWheel { .. } => 4,
                egui::Event::Key { .. } | egui::Event::PointerButton { .. } => 5,
                egui::Event::WindowFocused(_) => 6,
                _ => 7,
            };
            counts[i] += 1;
        }
        self.inputs.push(counts);
    }
    pub fn report(&self) -> String {
        let (requests, overflow, pending) = self.requests.snapshot();
        let sources = self
            .sources
            .iter()
            .map(|s| {
                format!(
                    "{{\"file_utf8\":{:?},\"line\":{}}}",
                    s.file.as_bytes(),
                    s.line
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        let passes = self
            .passes
            .iter()
            .map(|p| {
                format!(
                    "{{\"at\":{},\"pass\":{},\"frame\":{},\"causes\":{:?}}}",
                    p.at, p.pass, p.frame, p.causes
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "{{\"schema\":1,\"invalid\":{},\"overflow\":{},\"pending\":{},\"requests\":{:?},\"sources\":[{}],\"passes\":[{}],\"inputs\":{:?}}}",
            self.invalid, overflow, pending, requests, sources, passes, self.inputs
        )
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounded_requests_preserve_delay_and_report_overflow() {
        let r = Requests::new(1);
        r.push(10, 2, 0);
        r.push(11, 2, 16_000_000);
        assert_eq!(r.snapshot(), (vec![[10, 2, 0]], 1, 0));
    }
    #[test]
    fn unfinished_slot_is_reported_not_read_as_a_completed_request() {
        let r = Requests::new(1);
        r.next.store(1, Ordering::Release);
        assert_eq!(r.snapshot(), (vec![], 0, 1));
    }
    #[test]
    fn observer_preserves_framework_callback_and_observes_requests() {
        let ctx = egui::Context::default();
        let calls = Arc::new(AtomicUsize::new(0));
        let c = calls.clone();
        ctx.set_request_repaint_callback(move |_| {
            c.fetch_add(1, Ordering::Relaxed);
        });
        let r = Repaints::install(&ctx);
        ctx.request_repaint();
        ctx.request_repaint_after(std::time::Duration::from_millis(16));
        assert!(calls.load(Ordering::Relaxed) > 0);
        let (rows, o, p) = r.requests.snapshot();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0][2], 0);
        assert_eq!(rows[1][2], 16_000_000);
        assert_eq!((o, p), (0, 0));
    }
    #[test]
    fn sources_are_bounded_and_remove_absolute_path_prefix() {
        assert_eq!(
            source_name("C:\\private\\egui-0.37.0\\src\\context.rs"),
            "egui-0.37.0/src/context.rs"
        );
        let mut r = Repaints::install(&egui::Context::default());
        for i in 0..SOURCE_LIMIT {
            assert_eq!(r.source("host/src/main.rs", i as u32), Some(i));
        }
        assert_eq!(r.source("host/src/main.rs", 0), Some(0));
        assert_eq!(r.source("host/src/main.rs", 1000), None);
        assert_eq!(r.invalid, 1);
    }
    #[test]
    fn input_diagnostic_counts_kinds_without_storing_input_values() {
        let ctx = egui::Context::default();
        let mut r = Repaints::install(&ctx);
        let mut i = egui::RawInput::default();
        i.events
            .push(egui::Event::PointerMoved(egui::pos2(123., 456.)));
        i.events.push(egui::Event::Text("private input".into()));
        r.input(&ctx, &i);
        assert_eq!(r.inputs[0][2], 1);
        assert_eq!(r.inputs[0][7], 1);
        assert!(!r.report().contains("private input"));
    }

    #[test]
    fn repeated_delayed_requests_do_not_multiply_scheduler_callbacks() {
        let ctx = egui::Context::default();
        let calls = Arc::new(AtomicUsize::new(0));
        let c = calls.clone();
        ctx.set_request_repaint_callback(move |_| {
            c.fetch_add(1, Ordering::Relaxed);
        });
        let r = Repaints::install(&ctx);
        for _ in 0..3 {
            ctx.request_repaint_after(std::time::Duration::from_millis(16));
        }
        ctx.request_repaint_after(std::time::Duration::from_millis(20));
        assert_eq!(r.requests.snapshot().0.len(), 4);
        assert_eq!(calls.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn raw_mouse_event_requests_immediate_repaint_without_application_widgets() {
        let ctx = egui::Context::default();
        let r = Repaints::install(&ctx);
        for _ in 0..3 {
            let mut output = ctx.run_ui(egui::RawInput::default(), |_| {});
            output.textures_delta.clear();
        }
        let before = r.requests.snapshot().0.len();
        let mut input = egui::RawInput::default();
        input
            .events
            .push(egui::Event::MouseMoved(egui::vec2(2., 3.)));
        let mut output = ctx.run_ui(input, |_| {});
        output.textures_delta.clear();
        assert!(r.requests.snapshot().0[before..].iter().any(|r| r[2] == 0));
    }
}
