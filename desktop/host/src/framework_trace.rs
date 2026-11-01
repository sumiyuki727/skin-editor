//! Feature-gated official profiling/tracing backend. Thread-scoped subscriber,
//! no framework fork, global logger, project state, device or audio callback.
use std::{
    fmt,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU8, Ordering},
    },
    time::{Instant, SystemTime, UNIX_EPOCH},
};
use tracing::{
    Event, Metadata, Subscriber,
    field::{Field, Visit},
    span::{Attributes, Id, Record},
};
const LIMIT: usize = 32768;
// 0 ignored; 1 acquire, 2 finish, 3 submit, 4 present; 5 redraw;
// 6 pointer, 7 keyboard/IME, 8 focus, 9 geometry, 10 other window;
// 11 repaint user event, 12 device event, 13 integration, 14 logic;
// schema 2: 15 whole painter, 16 upload buffers, 17 upload texture, 18 encode render.
fn candidate(m: &Metadata<'_>) -> bool {
    (m.target() == "egui_wgpu::winit"
        && matches!(
            m.name(),
            "get_current_texture"
                | "CommandEncoder::finish"
                | "Queue::submit"
                | "present"
                | "function_scope"
        ))
        || (matches!(
            m.target(),
            "eframe::native::run" | "eframe::native::wgpu_integration" | "egui_wgpu::renderer"
        ) && m.name() == "function_scope")
}
struct Text {
    bytes: [u8; 256],
    used: usize,
}
impl Default for Text {
    fn default() -> Self {
        Self {
            bytes: [0; 256],
            used: 0,
        }
    }
}
impl fmt::Write for Text {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if self.used + s.len() > self.bytes.len() {
            return Err(fmt::Error);
        }
        self.bytes[self.used..self.used + s.len()].copy_from_slice(s.as_bytes());
        self.used += s.len();
        Ok(())
    }
}
impl Text {
    fn as_str(&self) -> &str {
        std::str::from_utf8(&self.bytes[..self.used])
            .unwrap_or("")
            .trim_matches('"')
    }
}
#[derive(Default)]
struct Tags {
    message: Text,
    tag: Text,
}
impl Visit for Tags {
    fn record_debug(&mut self, f: &Field, value: &dyn fmt::Debug) {
        use fmt::Write;
        // Never serialize fields. Only classify fixed framework function/event
        // labels; no coordinates, paths, typed text, keys or arbitrary values.
        if f.name() == "message" {
            let _ = write!(&mut self.message, "{value:?}");
        }
        if f.name() == "tag" {
            let _ = write!(&mut self.tag, "{value:?}");
        }
    }
}
fn kind(name: &str, function: &str, tag: &str) -> u8 {
    match name {
        "get_current_texture" => 1,
        "CommandEncoder::finish" => 2,
        "Queue::submit" => 3,
        "present" => 4,
        "function_scope" if function.ends_with("::window_event") => match tag {
            "WindowEvent::RedrawRequested" => 5,
            "WindowEvent::CursorMoved"
            | "WindowEvent::CursorEntered"
            | "WindowEvent::CursorLeft"
            | "WindowEvent::MouseWheel"
            | "WindowEvent::MouseInput" => 6,
            "WindowEvent::KeyboardInput" | "WindowEvent::Ime" | "WindowEvent::ModifiersChanged" => {
                7
            }
            "WindowEvent::Focused" => 8,
            "WindowEvent::Resized" | "WindowEvent::ScaleFactorChanged" | "WindowEvent::Moved" => 9,
            _ => 10,
        },
        "function_scope"
            if function.ends_with("::user_event") && tag == "UserEvent::RequestRepaint" =>
        {
            11
        }
        "function_scope" if function.ends_with("::device_event") => 12,
        "function_scope" if function.ends_with("::run_ui_and_paint") => 13,
        "function_scope" if function.ends_with("::paint_and_update_textures") => 15,
        "function_scope" if function.ends_with("::update_buffers") => 16,
        "function_scope" if function.ends_with("::update_texture") => 17,
        "function_scope" if function.ends_with("::render") => 18,
        _ => 0,
    }
}
struct Slot {
    kind: u8,
    refs: usize,
    entered: Option<[u64; 3]>,
}
struct Data {
    slots: Vec<Slot>,
    rows: Vec<[u64; 8]>,
    invalid: u64,
    ignored: u64,
    passes: Vec<Pass>,
    sources: Vec<(&'static str, u32)>,
}
struct Pass {
    row: [u64; 7],
    causes: [usize; 8],
    count: usize,
}
impl Data {
    fn new() -> Self {
        Self {
            slots: Vec::with_capacity(LIMIT),
            rows: Vec::with_capacity(LIMIT),
            invalid: 0,
            ignored: 0,
            passes: Vec::with_capacity(4096),
            sources: Vec::with_capacity(128),
        }
    }
    fn open(&mut self, kind: u8) -> u64 {
        if kind == 0 {
            self.ignored += 1;
            return 1;
        }
        if self.slots.len() == LIMIT {
            self.invalid += 1;
            return 1;
        }
        self.slots.push(Slot {
            kind,
            refs: 1,
            entered: None,
        });
        self.slots.len() as u64 + 1
    }
    fn enter(&mut self, id: u64, at: [u64; 3]) {
        if id == 1 {
            return;
        }
        let Some(s) = self.slots.get_mut(id.saturating_sub(2) as usize) else {
            self.invalid += 1;
            return;
        };
        if s.refs == 0 || s.entered.is_some() {
            self.invalid += 1;
            return;
        }
        s.entered = Some(at);
    }
    fn exit(&mut self, id: u64, at: [u64; 3]) {
        if id == 1 {
            return;
        }
        let Some(s) = self.slots.get_mut(id.saturating_sub(2) as usize) else {
            self.invalid += 1;
            return;
        };
        let Some(start) = s.entered.take() else {
            self.invalid += 1;
            return;
        };
        if self.rows.len() == LIMIT || at[0] < start[0] || at[1] < start[1] {
            self.invalid += 1;
            return;
        }
        self.rows.push([
            id,
            u64::from(s.kind),
            start[0],
            at[0],
            start[1],
            at[1],
            start[2],
            at[2],
        ]);
    }
    fn close(&mut self, id: u64) -> bool {
        if id == 1 {
            return true;
        }
        let Some(s) = self.slots.get_mut(id.saturating_sub(2) as usize) else {
            self.invalid += 1;
            return false;
        };
        if s.refs == 0 {
            self.invalid += 1;
            return false;
        }
        s.refs -= 1;
        if s.refs == 0 && s.entered.is_some() {
            self.invalid += 1;
        }
        s.refs == 0
    }
}
struct Inner {
    at: Instant,
    stage: AtomicU8,
    data: Mutex<Data>,
}
#[derive(Clone)]
pub struct Trace(Arc<Inner>);
impl Trace {
    fn new() -> Self {
        Self(Arc::new(Inner {
            at: Instant::now(),
            stage: AtomicU8::new(0),
            data: Mutex::new(Data::new()),
        }))
    }
    fn stamp(&self) -> [u64; 3] {
        [
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(0, |v| v.as_nanos() as u64),
            self.0.at.elapsed().as_nanos() as u64,
            u64::from(self.0.stage.load(Ordering::Relaxed)),
        ]
    }
    pub fn stage(&self, v: u8) {
        self.0.stage.store(v, Ordering::Relaxed);
    }
    pub fn logic(&self) -> Scope {
        let id = self.0.data.lock().unwrap().open(14);
        self.0.data.lock().unwrap().enter(id, self.stamp());
        Scope(self.clone(), id)
    }
    fn report(&self) -> String {
        let d = self.0.data.lock().unwrap();
        let active = d.slots.iter().filter(|s| s.entered.is_some()).count();
        let owners = d.slots.iter().map(|s| s.refs).sum::<usize>();
        let passes = d
            .passes
            .iter()
            .map(|p| {
                format!(
                    "{{\"row\":{:?},\"causes\":{:?}}}",
                    p.row,
                    &p.causes[..p.count]
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        let sources = d
            .sources
            .iter()
            .map(|(file, line)| {
                let parts: Vec<_> = file.rsplit(['/', '\\']).take(3).collect();
                let file = parts.into_iter().rev().collect::<Vec<_>>().join("/");
                format!("{{\"file_utf8\":{:?},\"line\":{line}}}", file.as_bytes())
            })
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "{{\"schema\":3,\"invalid\":{},\"ignored\":{},\"active\":{},\"owners\":{},\"spans\":{},\"rows\":{:?},\"passes\":[{}],\"discard_sources\":[{}]}}",
            d.invalid,
            d.ignored,
            active,
            owners,
            d.slots.len(),
            d.rows,
            passes,
            sources
        )
    }
}
pub struct Scope(Trace, u64);
pub struct Passes(pub Trace);
impl eframe::egui::plugin::Plugin for Passes {
    fn debug_name(&self) -> &'static str {
        "bounded-pass-discard-observer"
    }
    fn output_hook(&mut self, ctx: &eframe::egui::Context, out: &mut eframe::egui::FullOutput) {
        let [epoch, mono, stage] = self.0.stamp();
        // output_hook runs AFTER Context has taken/reset its own output.
        // Context::will_discard() is therefore no longer this pass's answer.
        let discard = out.platform_output.requested_discard()
            && out.platform_output.num_completed_passes < ctx.options(|o| o.max_passes.get());
        let row = [
            epoch,
            mono,
            stage,
            ctx.cumulative_frame_nr(),
            ctx.cumulative_pass_nr(),
            out.platform_output.num_completed_passes as u64,
            u64::from(discard),
        ];
        let root = ctx.viewport_id() == eframe::egui::ViewportId::ROOT;
        let mut d = self.0.0.data.lock().unwrap();
        if !root || d.passes.len() == 4096 || out.platform_output.request_discard_reasons.len() > 8
        {
            d.invalid += 1;
            return;
        }
        let mut p = Pass {
            row,
            causes: [0; 8],
            count: 0,
        };
        for cause in &out.platform_output.request_discard_reasons {
            let key = (cause.file, cause.line);
            let id = if let Some(i) = d.sources.iter().position(|s| *s == key) {
                i
            } else {
                if d.sources.len() == 128 {
                    d.invalid += 1;
                    return;
                }
                let i = d.sources.len();
                d.sources.push(key);
                i
            };
            p.causes[p.count] = id;
            p.count += 1;
        }
        d.passes.push(p);
    }
}
impl Drop for Scope {
    fn drop(&mut self) {
        let at = self.0.stamp();
        let mut d = self.0.0.data.lock().unwrap();
        d.exit(self.1, at);
        d.close(self.1);
    }
}
mod event_loop;
pub use event_loop::run_native;
impl Trace {
    fn handler(&self, kind: u8) -> Scope {
        let id = self.0.data.lock().unwrap().open(kind);
        self.0.data.lock().unwrap().enter(id, self.stamp());
        Scope(self.clone(), id)
    }
}
struct Recorder(Trace);
impl Subscriber for Recorder {
    fn enabled(&self, m: &Metadata<'_>) -> bool {
        candidate(m)
    }
    fn new_span(&self, a: &Attributes<'_>) -> Id {
        let mut t = Tags::default();
        a.record(&mut t);
        Id::from_u64(self.0.0.data.lock().unwrap().open(kind(
            a.metadata().name(),
            t.message.as_str(),
            t.tag.as_str(),
        )))
    }
    fn record(&self, _: &Id, _: &Record<'_>) {}
    fn record_follows_from(&self, _: &Id, _: &Id) {}
    fn event(&self, _: &Event<'_>) {}
    fn enter(&self, id: &Id) {
        let at = self.0.stamp();
        self.0.0.data.lock().unwrap().enter(id.into_u64(), at);
    }
    fn exit(&self, id: &Id) {
        let at = self.0.stamp();
        self.0.0.data.lock().unwrap().exit(id.into_u64(), at);
    }
    fn clone_span(&self, id: &Id) -> Id {
        if id.into_u64() != 1 {
            let mut d = self.0.0.data.lock().unwrap();
            if let Some(s) = d.slots.get_mut(id.into_u64().saturating_sub(2) as usize) {
                s.refs += 1;
            } else {
                d.invalid += 1;
            }
        }
        id.clone()
    }
    fn try_close(&self, id: Id) -> bool {
        self.0.0.data.lock().unwrap().close(id.into_u64())
    }
}
pub fn current() -> Option<Trace> {
    tracing::dispatcher::get_default(|d| d.downcast_ref::<Recorder>().map(|r| r.0.clone()))
}
pub fn run(destination: &str, main_work: bool) -> Result<(), String> {
    let trace = Trace::new();
    let result = tracing::subscriber::with_default(Recorder(trace.clone()), || {
        super::editor_audio::gui_verification::dispatch(destination, main_work)
    });
    if std::path::Path::new(destination).is_dir() {
        std::fs::write(
            std::path::Path::new(destination).join("framework_trace.json"),
            trace.report(),
        )
        .map_err(|e| e.to_string())?;
    }
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn concrete_framework_labels_and_unknown_payloads_have_no_text_output() {
        assert_eq!(
            kind("function_scope", "x::paint_and_update_textures", ""),
            15
        );
        assert_eq!(kind("function_scope", "x::update_buffers", ""), 16);
        assert_eq!(kind("function_scope", "x::update_texture", ""), 17);
        assert_eq!(kind("function_scope", "x::render", ""), 18);
        assert_eq!(kind("Queue::submit", "", ""), 3);
        assert_eq!(
            kind(
                "function_scope",
                "x::window_event",
                "WindowEvent::RedrawRequested"
            ),
            5
        );
        assert_eq!(
            kind(
                "function_scope",
                "x::window_event",
                "WindowEvent::CursorMoved"
            ),
            6
        );
        assert_eq!(
            kind(
                "function_scope",
                "x::user_event",
                "UserEvent::RequestRepaint"
            ),
            11
        );
        assert_eq!(kind("function_scope", "unrelated", "private text"), 0);
        assert_eq!(
            kind("function_scope", "x::window_event", "private text"),
            10
        );
    }
    #[test]
    fn invalid_duplicate_reversed_and_closed_span_edges_remain_visible() {
        let mut d = Data::new();
        let id = d.open(3);
        d.enter(id, [10, 5, 4]);
        d.enter(id, [10, 5, 4]);
        d.exit(id, [9, 6, 4]);
        d.exit(id, [11, 7, 4]);
        assert!(d.close(id));
        d.enter(id, [12, 8, 4]);
        assert!(!d.close(id));
        assert_eq!(d.invalid, 5);
    }
    #[test]
    fn recording_is_bounded_and_has_distinct_monotonic_and_epoch_clocks() {
        let mut d = Data::new();
        let id = d.open(4);
        d.enter(id, [100, 10, 4]);
        d.exit(id, [120, 30, 5]);
        d.close(id);
        assert_eq!(d.rows, [[id, 4, 100, 120, 10, 30, 4, 5]]);
        for _ in 1..LIMIT {
            d.open(1);
        }
        assert_eq!(d.open(1), 1);
        assert_eq!(d.invalid, 1);
    }
    #[test]
    fn thread_scoped_official_spans_and_clones_release_all_owners() {
        let t = Trace::new();
        assert!(current().is_none());
        tracing::subscriber::with_default(Recorder(t.clone()), || {
            assert!(current().is_some());
            let s = tracing::span!(target:"egui_wgpu::winit",tracing::Level::INFO,"Queue::submit");
            let c = s.clone();
            {
                let _g = s.enter();
            }
            drop(c);
            drop(s);
            let _g = t.logic();
        });
        assert!(current().is_none());
        let d = t.0.data.lock().unwrap();
        assert_eq!(d.invalid, 0);
        assert_eq!(d.rows.len(), 2);
        assert!(d.slots.iter().all(|s| s.refs == 0 && s.entered.is_none()));
    }
    #[test]
    fn fixed_text_buffer_rejects_overflow_without_growth() {
        use fmt::Write;
        let mut t = Text::default();
        assert!(t.write_str(&"a".repeat(256)).is_ok());
        assert!(t.write_str("b").is_err());
        assert_eq!(t.used, 256);
    }
    #[test]
    fn discarded_layout_passes_keep_frame_identity_and_source_without_reason_text() {
        use eframe::egui;
        let t = Trace::new();
        let ctx = egui::Context::default();
        ctx.add_plugin(Passes(t.clone()));
        ctx.options_mut(|o| o.max_passes = 2.try_into().unwrap());
        let mut calls = 0;
        let mut out = ctx.run_ui(egui::RawInput::default(), |root| {
            if calls == 0 {
                root.ctx().request_discard("private discard text");
            }
            calls += 1;
        });
        out.textures_delta.clear();
        {
            let d = t.0.data.lock().unwrap();
            assert_eq!(d.invalid, 0);
            assert_eq!(d.passes.len(), 2);
            assert_eq!(d.passes[0].row[3], d.passes[1].row[3]);
            assert_eq!(d.passes[0].row[6], 1);
            assert_eq!(d.passes[1].row[6], 0);
            assert_eq!(d.sources.len(), 1);
        }
        assert!(!t.report().contains("private discard text"));
    }
}
