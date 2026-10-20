//! Standard-load experiment. Immutable fixtures and scripted view changes, no SkinProject writes.
use eframe::egui;
use p0_egui::loading::{
    DecodeJob, DecodeQueue, MAX_IMAGE_BYTES, MetadataTask, Ready, UPLOAD_BYTES_PER_FRAME,
    UPLOADS_PER_FRAME, UploadBudget,
};
use p0_egui::texture_cache::TextureCache;
use p0_egui::upload_fence::UploadFence;
use p0_egui::view::{show_resource_list, wake_loading};
use p0_egui::{ByteCache, CACHE_BYTES, Decoded, Workload, decode_png};
use p0_preview_contract::Probe;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::Write,
    path::PathBuf,
    sync::{Arc, mpsc},
    time::{Duration, Instant},
};

#[cfg(feature = "heap-profile")]
#[global_allocator]
static GLOBAL: &stats_alloc::StatsAlloc<std::alloc::System> = &stats_alloc::INSTRUMENTED_SYSTEM;

fn heap_snapshot() -> Value {
    #[cfg(feature = "heap-profile")]
    {
        let s = GLOBAL.stats();
        // Reallocation growth/shrink is already included in these byte counters.
        json!({"live_requested_bytes":s.bytes_allocated.checked_sub(s.bytes_deallocated),
            "bytes_allocated":s.bytes_allocated,"bytes_deallocated":s.bytes_deallocated,
            "allocations":s.allocations,"deallocations":s.deallocations,"reallocations":s.reallocations,
            "note":"Non-atomic Rust global allocator counters; requested bytes, not heap commit; excludes native/driver allocations"})
    }
    #[cfg(not(feature = "heap-profile"))]
    {
        Value::Null
    }
}

const PERIOD: Duration = Duration::from_nanos(16_666_667);
const EDITS: usize = 300;
#[derive(Clone, Copy, PartialEq, Debug)]
enum Phase {
    Empty,
    Loading,
    Warm,
    Edit,
    Before,
    After,
    IdleSettle,
    Idle,
    RapidSwitch,
    Cycle,
    ClosedSettle,
    Closed,
    Done,
}
struct PendingLoad {
    // None means the paused material replacement; documents keep the old view
    // until all required textures are available.
    document: Option<usize>,
    remaining: BTreeSet<usize>,
    started: Instant,
}
struct Readback {
    tag: usize,
    pixels: Arc<egui::ColorImage>,
    finished: Instant,
}
struct Bench {
    root: PathBuf,
    output: PathBuf,
    events: File,
    started: Instant,
    phase: Phase,
    entered: Instant,
    workload: Option<Workload>,
    cpu: ByteCache<Decoded>,
    gpu: TextureCache,
    thumbs: Vec<egui::TextureHandle>,
    material_override: Option<egui::TextureHandle>,
    current: usize,
    probe: Probe,
    adapter: Value,
    next_frame: Instant,
    frames: u64,
    ui_passes: u64,
    last_ui_frame: Option<u64>,
    visibility_counts: [u64; 3],
    idle_frames: u64,
    edit: usize,
    in_flight: bool,
    edit_started: Option<Instant>,
    latencies: Vec<f64>,
    edit_intervals: Vec<f64>,
    last_edit: Option<Instant>,
    previous_edit_pixels: Option<Vec<egui::Color32>>,
    frame_cpu: Vec<f64>,
    decode_ms: Vec<f64>,
    switch_ms: Vec<f64>,
    decode_count: usize,
    first_decoded: usize,
    cycle: usize,
    before: Option<Vec<u8>>,
    frozen: Option<u64>,
    captures: Vec<String>,
    canvas: egui::Rect,
    tx: mpsc::Sender<Readback>,
    rx: mpsc::Receiver<Readback>,
    completed: bool,
    phase_rows: Vec<Value>,
    diagnostic: bool,
    readback_edits: bool,
    readbacks_enabled: bool,
    memory_usage: bool,
    entered_frame: u64,
    pause_verified: Option<bool>,
    phase_frame_bins: BTreeMap<String, BTreeMap<u64, u64>>,
    repaint_causes: BTreeMap<String, BTreeMap<String, u64>>,
    render_state: Option<eframe::egui_wgpu::RenderState>,
    issued_textures: BTreeSet<egui::TextureId>,
    decoded_refs: Vec<std::sync::Weak<egui::ColorImage>>,
    readback_requests: usize,
    decoder: Option<DecodeQueue>,
    pending: Option<PendingLoad>,
    ready: Option<Ready>,
    upload_budget: UploadBudget,
    uploads: BTreeMap<u64, (usize, usize)>,
    loading_frame_cpu: Vec<f64>,
    previous_loading_frame: bool,
    request_ms: Vec<f64>,
    request_count: usize,
    rapid_switch_verified: Option<bool>,
    metadata_ms: f64,
    loading_work_this_frame: bool,
    cache_only_reload: bool,
    metadata_task: Option<MetadataTask>,
    initial_started: Option<Instant>,
    initial_load_ms: Option<f64>,
    paced_loading: bool,
    profile_memory: bool,
    no_original_upload: bool,
    no_original_draw: bool,
    memory_trace: Option<File>,
    next_memory_trace: Instant,
    reuse_textures: bool,
    texture_budget: usize,
    drain_uploads: bool,
    drain_pending: bool,
    drain_frame: u64,
    drains: Vec<Value>,
    texture_proof: bool,
    proof_pending: bool,
    texture_proofs: Vec<Value>,
    bounded_uploads: bool,
    observe_uploads: bool,
    fence: UploadFence,
    fence_tx: mpsc::Sender<(u64, Instant)>,
    fence_rx: mpsc::Receiver<(u64, Instant)>,
    lifetime: Option<File>,
    fence_wait_frames: usize,
}
fn stats(values: &[f64]) -> Value {
    if values.is_empty() {
        return json!(null);
    }
    let mut v = values.to_vec();
    v.sort_by(f64::total_cmp);
    json!({"count":v.len(),"min":v[0],"mean":v.iter().sum::<f64>()/v.len() as f64,"p50":v[((v.len()-1)*50)/100],"p95":v[((v.len()-1)*95)/100],"max":v[v.len()-1]})
}
impl Bench {
    fn new(
        cc: &eframe::CreationContext<'_>,
        root: PathBuf,
        output: PathBuf,
        diagnostic: bool,
        readback_edits: bool,
        readbacks_enabled: bool,
        flags: &[String],
    ) -> Self {
        // This fixture needs 10 current + 10 incoming originals pinned simultaneously.
        let texture_budget = if flags.iter().any(|f| f == "--small-texture-cache") {
            80 * 1024 * 1024
        } else {
            CACHE_BYTES
        };
        let paced_loading = flags.iter().any(|f| f == "--paced-loading");
        let profile_memory = flags.iter().any(|f| f == "--profile-memory");
        let memory_trace = profile_memory
            .then(|| File::create(output.join("memory-trace.jsonl")).expect("memory trace"));
        let (tx, rx) = mpsc::channel();
        let (fence_tx, fence_rx) = mpsc::channel();
        let lifetime = flags
            .iter()
            .any(|f| f == "--lifetime-trace")
            .then(|| File::create(output.join("lifetime.jsonl")).expect("lifetime file"));
        let now = Instant::now();
        let anchor = json!({"unix_seconds":std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs_f64()});
        std::fs::write(output.join("clock-anchor.json"), anchor.to_string()).expect("clock anchor");
        cc.egui_ctx
            .send_viewport_cmd(egui::ViewportCommand::RequestPaintWhileHidden);
        let adapter=cc.wgpu_render_state.as_ref().map_or(json!(null),|s|{let i=s.adapter.get_info();json!({"name":i.name,"backend":format!("{:?}",i.backend),"driver":i.driver,"driver_info":i.driver_info})});
        let events = File::create(output.join("phases.jsonl")).expect("phase journal");
        let mut app = Self {
            root,
            output,
            events,
            started: now,
            phase: Phase::Empty,
            entered: now,
            workload: None,
            cpu: ByteCache::new(CACHE_BYTES),
            gpu: TextureCache::new(
                texture_budget,
                [1024, 1024],
                flags.iter().any(|f| f == "--reuse-textures"),
            ),
            thumbs: Vec::new(),
            material_override: None,
            current: 0,
            probe: Probe::new(0),
            adapter,
            next_frame: now,
            frames: 0,
            ui_passes: 0,
            last_ui_frame: None,
            visibility_counts: [0; 3],
            idle_frames: 0,
            edit: 0,
            in_flight: false,
            edit_started: None,
            latencies: Vec::new(),
            edit_intervals: Vec::new(),
            last_edit: None,
            previous_edit_pixels: None,
            frame_cpu: Vec::new(),
            decode_ms: Vec::new(),
            switch_ms: Vec::new(),
            decode_count: 0,
            first_decoded: 0,
            cycle: 0,
            before: None,
            frozen: None,
            captures: Vec::new(),
            canvas: egui::Rect::NOTHING,
            tx,
            rx,
            completed: false,
            phase_rows: Vec::new(),
            diagnostic,
            readback_edits,
            readbacks_enabled,
            memory_usage: flags.iter().any(|f| f == "--memory-usage"),
            entered_frame: 0,
            pause_verified: None,
            phase_frame_bins: BTreeMap::new(),
            repaint_causes: BTreeMap::new(),
            render_state: cc.wgpu_render_state.clone(),
            issued_textures: BTreeSet::new(),
            decoded_refs: Vec::new(),
            readback_requests: 0,
            decoder: flags.iter().any(|f| f == "--async-load").then(|| {
                let ctx = cc.egui_ctx.clone();
                if paced_loading {
                    DecodeQueue::prefetched(move || wake_loading(&ctx))
                } else {
                    DecodeQueue::new(move || ctx.request_repaint())
                }
            }),
            pending: None,
            ready: None,
            upload_budget: UploadBudget::default(),
            uploads: BTreeMap::new(),
            loading_frame_cpu: Vec::new(),
            previous_loading_frame: false,
            request_ms: Vec::new(),
            request_count: 0,
            rapid_switch_verified: None,
            metadata_ms: 0.0,
            loading_work_this_frame: false,
            cache_only_reload: false,
            metadata_task: None,
            initial_started: None,
            initial_load_ms: None,
            paced_loading,
            profile_memory,
            no_original_upload: flags.iter().any(|f| f == "--no-original-upload"),
            no_original_draw: flags.iter().any(|f| f == "--no-original-draw"),
            memory_trace,
            next_memory_trace: now,
            texture_budget,
            reuse_textures: flags.iter().any(|f| f == "--reuse-textures"),
            drain_uploads: flags.iter().any(|f| f == "--drain-uploads"),
            drain_pending: false,
            drain_frame: 0,
            drains: Vec::new(),
            texture_proof: flags.iter().any(|f| f == "--texture-proof"),
            proof_pending: false,
            texture_proofs: Vec::new(),
            bounded_uploads: flags.iter().any(|f| f == "--bounded-uploads"),
            observe_uploads: flags
                .iter()
                .any(|f| f == "--bounded-uploads" || f == "--lifetime-trace"),
            fence: UploadFence::default(),
            fence_tx,
            fence_rx,
            lifetime,
            fence_wait_frames: 0,
        };
        app.event("Empty");
        app
    }
    fn lifetime_event(&mut self, kind: &str, detail: Value) {
        if let Some(file) = &mut self.lifetime {
            let row = json!({"elapsed_seconds":self.started.elapsed().as_secs_f64(),"kind":kind,
                "phase":format!("{:?}",self.phase),"cycle":self.cycle,"detail":detail,
                "gpu_payload_bytes":self.gpu.bytes(),"outstanding_upload_bytes":self.fence.bytes()});
            writeln!(file, "{row}").expect("lifetime write");
            file.flush().expect("lifetime flush");
        }
    }
    fn progress_uploads(&mut self, ctx: &egui::Context) -> Result<(), String> {
        if !self.observe_uploads {
            return Ok(());
        }
        let state = self.render_state.as_ref().ok_or("No GPU state")?;
        for ticket in self.fence.arm_before(ctx.cumulative_frame_nr()) {
            let tx = self.fence_tx.clone();
            let wake = ctx.clone();
            state.queue.on_submitted_work_done(move || {
                let _ = tx.send((ticket, Instant::now()));
                wake_loading(&wake);
            });
        }
        if self.fence.bytes() > 0 {
            state
                .device
                .poll(eframe::wgpu::PollType::Poll)
                .map_err(|e| format!("GPU poll: {e:?}"))?;
            ctx.request_repaint_after(PERIOD);
        }
        while let Ok((ticket, time)) = self.fence_rx.try_recv() {
            let bytes = self.fence.complete(ticket)?;
            self.lifetime_event(
                "UploadComplete",
                json!({"ticket":ticket,"bytes":bytes,
                "callback_elapsed_seconds":time.duration_since(self.started).as_secs_f64()}),
            );
        }
        Ok(())
    }
    fn event(&mut self, label: &str) {
        let allocator=self.render_state.as_ref().and_then(|s|s.device.generate_allocator_report()).map(|report|{
            let mut groups:BTreeMap<String,u64>=BTreeMap::new();
            for allocation in &report.allocations {
                *groups.entry(allocation.name.clone()).or_default()+=allocation.size;
            }
            json!({"allocated_bytes":report.total_allocated_bytes,"reserved_bytes":report.total_reserved_bytes,
                "allocation_count":report.allocations.len(),"block_count":report.blocks.len(),"allocations_by_label":groups})
        });
        let resources=self.render_state.as_ref().and_then(|s|s.instance.generate_report()).map(|report|json!({
            "textures_kept":report.hub.textures.num_kept_from_user,"textures_released":report.hub.textures.num_released_from_user,
            "buffers_kept":report.hub.buffers.num_kept_from_user,"buffers_released":report.hub.buffers.num_released_from_user}));
        let owned_textures = self.render_state.as_ref().map(|s| {
            let renderer = s.renderer.read();
            self.issued_textures
                .iter()
                .filter(|id| renderer.texture(id).is_some())
                .count()
        });
        let value = json!({"phase":label,"elapsed_seconds":self.started.elapsed().as_secs_f64(),"ui_frames":self.frames,
            "cpu_cache_bytes":self.cpu.bytes,"gpu_texture_payload_bytes":self.gpu.bytes(),"cycle":self.cycle,"rust_heap":heap_snapshot(),"idle_texture_pool_bytes":self.gpu.idle_bytes(),
            "wgpu_allocator_before_current_frame":allocator,"wgpu_resources_before_current_frame":resources,
            "owned_textures_in_renderer":owned_textures,"owned_texture_ids_issued":self.issued_textures.len(),
            "decode_worker_idle":self.decoder.as_ref().map(DecodeQueue::is_idle),"pending_load":self.pending.is_some(),"ready_decoded_bytes":self.ready.as_ref().map_or(0,|r|r.image.pixels.len()*4),
            "original_decoded_images_live":self.decoded_refs.iter().filter(|image|image.strong_count()>0).count()});
        writeln!(self.events, "{value}").expect("phase journal write");
        self.events.flush().expect("flush phase");
        self.phase_rows.push(value);
    }
    fn enter(&mut self, phase: Phase) {
        if phase == Phase::Warm {
            self.initial_load_ms = self
                .initial_started
                .map(|s| s.elapsed().as_secs_f64() * 1000.0);
        }
        self.phase = phase;
        self.entered = Instant::now();
        self.entered_frame = self.frames;
        self.event(&format!("{phase:?}"));
    }
    fn trace_memory(&mut self) {
        if self.memory_trace.is_none() || Instant::now() < self.next_memory_trace {
            return;
        }
        self.next_memory_trace = Instant::now() + Duration::from_millis(100);
        let allocator=self.render_state.as_ref().and_then(|s|s.device.generate_allocator_report()).map(|report| {
            let mut categories:BTreeMap<&str,u64>=BTreeMap::new();
            for allocation in &report.allocations {
                let category=if allocation.name.contains("Staging") { "staging" }
                    else if allocation.name.contains("egui_texid") { "textures" } else { "other" };
                *categories.entry(category).or_default()+=allocation.size;
            }
            json!({"allocated_bytes":report.total_allocated_bytes,"reserved_bytes":report.total_reserved_bytes,"categories":categories})
        });
        let live_bytes: usize = self
            .decoded_refs
            .iter()
            .filter_map(std::sync::Weak::upgrade)
            .map(|i| i.pixels.len() * 4)
            .sum();
        let row = json!({"elapsed_seconds":self.started.elapsed().as_secs_f64(),"phase":format!("{:?}",self.phase),"cycle":self.cycle,
            "ui_frames":self.frames,"cpu_cache_bytes":self.cpu.bytes,"gpu_texture_payload_bytes":self.gpu.bytes(),
            "live_accepted_original_pixel_bytes":live_bytes,"ready_pixel_bytes":self.ready.as_ref().map_or(0,|r|r.image.pixels.len()*4),
            "rust_heap":heap_snapshot(),"idle_texture_pool_bytes":self.gpu.idle_bytes(),
            "outstanding_requests":self.decoder.as_ref().map(DecodeQueue::outstanding),"wgpu_allocator":allocator});
        let trace = self.memory_trace.as_mut().unwrap();
        writeln!(trace, "{row}").expect("memory trace write");
        trace.flush().expect("memory trace flush");
    }
    fn load(&mut self, ctx: &egui::Context, document: usize) -> Result<(), String> {
        let start = Instant::now();
        self.request_count += 1;
        self.loading_work_this_frame = true;
        if self.decoder.is_some() {
            self.begin_async(Some(document))?;
            self.request_ms.push(start.elapsed().as_secs_f64() * 1000.0);
            return Ok(());
        }
        let workload = self.workload.as_ref().ok_or("No workload")?;
        let pins: BTreeSet<_> = workload.documents[document]
            .iter()
            .map(|l| l.asset)
            .collect();
        for &id in &pins {
            let decoded = if let Some(image) = self.cpu.get(id) {
                image.clone()
            } else {
                let now = Instant::now();
                let image = Arc::new(decode_png(&workload.assets[id].path)?);
                self.decoded_refs.push(Arc::downgrade(&image));
                self.decode_ms.push(now.elapsed().as_secs_f64() * 1000.0);
                self.decode_count += 1;
                self.cpu
                    .insert(id, image.clone(), image.pixels.len() * 4, &pins)?;
                image
            };
            if self.gpu.get(id).is_none() {
                // The queued upload shares immutable pixels with the CPU cache.
                let texture = self.gpu.upload(ctx, id, decoded.clone(), &pins)?;
                self.issued_textures.insert(texture);
                let row = self.uploads.entry(ctx.cumulative_frame_nr()).or_default();
                row.0 += decoded.pixels.len() * 4;
                row.1 += 1;
            }
        }
        self.current = document;
        self.switch_ms.push(start.elapsed().as_secs_f64() * 1000.0);
        Ok(())
    }
    fn begin_async(&mut self, document: Option<usize>) -> Result<(), String> {
        let workload = self.workload.as_ref().ok_or("No workload")?;
        let mut remaining: BTreeSet<_> = document.map_or_else(
            || BTreeSet::from([112]),
            |d| {
                workload.documents[d]
                    .iter()
                    .map(|l| l.asset)
                    .filter(|&id| self.gpu.get(id).is_none())
                    .collect()
            },
        );
        if self.thumbs.is_empty() && document.is_some() {
            remaining.extend(100..112);
        }
        let jobs = remaining
            .iter()
            .filter(|&&id| id >= 100 || self.cpu.get(id).is_none())
            .map(|&id| {
                let (path, size) = if id < 100 {
                    (&workload.assets[id].path, [1024, 1024])
                } else if id < 112 {
                    (&workload.assets[id - 100].thumbnail, [64, 64])
                } else {
                    (&workload.assets[99].path, [1024, 1024])
                };
                DecodeJob {
                    id,
                    path: path.clone(),
                    size,
                }
            })
            .collect();
        self.decoder.as_mut().unwrap().replace(jobs)?;
        self.ready = None;
        self.pending = Some(PendingLoad {
            document,
            remaining,
            started: Instant::now(),
        });
        Ok(())
    }
    fn pump_loading(&mut self, ctx: &egui::Context) -> Result<(), String> {
        if self.bounded_uploads && !self.fence.allows(ctx.cumulative_frame_nr()) {
            self.fence_wait_frames += 1;
            return Ok(());
        }
        let Some(decoder) = &mut self.decoder else {
            return Ok(());
        };
        self.upload_budget.begin(ctx.cumulative_frame_nr());
        for _ in 0..if self.paced_loading { 2 } else { 1 } {
            if self.ready.is_none() {
                self.ready = decoder.poll().transpose()?;
            }
            let Some(pending) = &mut self.pending else {
                self.ready = None;
                return Ok(());
            };
            let workload = self.workload.as_ref().ok_or("No workload")?;
            let mut pins: BTreeSet<_> = workload.documents[self.current]
                .iter()
                .map(|l| l.asset)
                .collect();
            if let Some(d) = pending.document {
                pins.extend(workload.documents[d].iter().map(|l| l.asset));
            }
            if let Some(ready) = &self.ready
                && ready.id < 100
                && self.cpu.get(ready.id).is_none()
            {
                self.decoded_refs.push(Arc::downgrade(&ready.image));
                self.cpu.insert(
                    ready.id,
                    ready.image.clone(),
                    ready.image.pixels.len() * 4,
                    &pins,
                )?;
                self.decode_count += 1;
            }
            // Cached pixels and freshly decoded pixels use the same frame quota.
            let mut made_progress = false;
            for id in pending.remaining.clone() {
                let image = self
                    .ready
                    .as_ref()
                    .filter(|r| r.id == id)
                    .map(|r| r.image.clone())
                    .or_else(|| self.cpu.get(id).cloned());
                let Some(image) = image else {
                    continue;
                };
                let bytes = image.pixels.len() * 4;
                if !self.upload_budget.take(bytes) {
                    break;
                }
                if id < 100 && self.no_original_upload {
                    pending.remaining.remove(&id);
                    if self.ready.as_ref().is_some_and(|r| r.id == id) {
                        self.ready = None;
                    }
                    made_progress = true;
                    continue;
                }
                if id < 100 {
                    let texture = self.gpu.upload(ctx, id, image, &pins)?;
                    self.issued_textures.insert(texture);
                } else {
                    let handle = ctx.load_texture(
                        format!("async-{id}"),
                        image,
                        egui::TextureOptions::LINEAR,
                    );
                    self.issued_textures.insert(handle.id());
                    if id < 112 {
                        self.thumbs.push(handle);
                    } else {
                        self.material_override = Some(handle);
                    }
                }
                let row = self.uploads.entry(ctx.cumulative_frame_nr()).or_default();
                row.0 += bytes;
                row.1 += 1;
                if self.observe_uploads {
                    let ticket = self.fence.queued(ctx.cumulative_frame_nr(), bytes);
                    if let Some(file) = &mut self.lifetime {
                        writeln!(file,"{}",json!({"elapsed_seconds":self.started.elapsed().as_secs_f64(),
                            "kind":"UploadQueued","ticket":ticket,"bytes":bytes,"asset":id,"cycle":self.cycle,
                            "outstanding_upload_bytes":self.fence.bytes()})).expect("upload event");
                        file.flush().expect("upload event flush");
                    }
                }
                pending.remaining.remove(&id);
                made_progress = true;
                if self.ready.as_ref().is_some_and(|r| r.id == id) {
                    self.ready = None;
                }
            }
            if !made_progress || self.upload_budget.count == UPLOADS_PER_FRAME {
                break;
            }
        }
        let Some(pending) = &self.pending else {
            return Ok(());
        };
        if pending.remaining.is_empty() {
            let pending = self.pending.take().unwrap();
            if let Some(document) = pending.document {
                self.current = document;
                self.switch_ms
                    .push(pending.started.elapsed().as_secs_f64() * 1000.0);
                match self.phase {
                    Phase::Loading => {
                        self.first_decoded = self.decode_count;
                        self.enter(Phase::Warm);
                    }
                    Phase::Cycle => {
                        self.complete_cycle();
                    }
                    Phase::RapidSwitch => {
                        if self.current != 9 {
                            return Err("Obsolete document replaced rapid-switch winner".into());
                        }
                        self.rapid_switch_verified = Some(true);
                        if !self.cache_only_reload {
                            self.cache_only_reload = true;
                            self.gpu.reset_bindings();
                            self.load(ctx, 9)?;
                        } else {
                            self.enter(Phase::Cycle);
                        }
                    }
                    _ => {}
                }
            } else {
                self.enter(Phase::After);
            }
        }
        Ok(())
    }
    fn complete_cycle(&mut self) {
        self.cycle += 1;
        self.next_frame = Instant::now() + Duration::from_millis(50);
        if self.cycle.is_multiple_of(10) {
            self.event("RoundEnd");
            if self.drain_uploads {
                self.drain_pending = true;
                self.drain_frame = self.frames;
            }
        }
        self.proof_pending =
            self.texture_proof && (self.cycle <= 11 || self.cycle == self.cycle_count());
    }
    fn capture(&mut self, ctx: &egui::Context, tag: usize) {
        self.readback_requests += 1;
        self.in_flight = true;
        let tx = self.tx.clone();
        let wake = ctx.clone();
        ctx.request_screenshot(move |pixels| {
            let _ = tx.send(Readback {
                tag,
                pixels,
                finished: Instant::now(),
            });
            wake.request_repaint();
        });
    }
    fn finish(&mut self, ctx: &egui::Context, error: Option<String>) {
        if self.completed {
            return;
        }
        self.completed = true;
        self.enter(Phase::Done);
        let mut result = json!({"schema_version":6,"experiment":"p0-egui-standard-load","status":if error.is_none(){"passed"}else{"failed"},"error":error,
            "adapter":self.adapter,"viewport_points":[1100,720],"target_hz":60,"vsync":false,"window":"visible native wgpu viewport; timed edit script, own render-target readback",
            "workload":{"assets":100,"dimensions":[1024,1024],"documents":10,"layers":500,"current_layers":50,"visible_thumbnails":12,"active_unique_originals":10,"nested_group_depth":2},
            "first_document_original_decode_count":self.first_decoded,"original_decode_count":self.decode_count,
            "decode_ms":stats(&self.decode_ms),"document_load_cpu_ms":if self.decoder.is_none(){stats(&self.switch_ms)}else{Value::Null},
            "edit_count":self.edit,"edit_to_gpu_readback_ms":stats(&self.latencies),"eframe_cpu_frame_ms_during_edits":stats(&self.frame_cpu),
            "edit_interval_ms":stats(&self.edit_intervals),
            "gpu_timestamp_ms":null,"gpu_timestamp_note":"eframe frame CPU excludes vsync; no GPU timestamp queries in this experiment",
            "cpu_cache_budget_bytes":CACHE_BYTES,"cpu_cache_peak_bytes":self.cpu.peak_bytes});
        let details = json!({
            "diagnostic":self.diagnostic,"per_edit_gpu_readback":self.readback_edits,"repaint_causes_by_phase":self.repaint_causes,
            "gpu_readbacks_enabled":self.readbacks_enabled,"memory_hint":if self.memory_usage{"MemoryUsage"}else{"Performance"},
            "gpu_readback_requests":self.readback_requests,
            "heap_profile_build":cfg!(feature="heap-profile"),"texture_reuse":self.reuse_textures,
            "original_textures_created":self.gpu.created,"original_textures_updated":self.gpu.updated,
            "bounded_uploads":self.bounded_uploads,"upload_receipts_observed":self.observe_uploads,
            "outstanding_upload_peak_bytes":if self.observe_uploads{Some(self.fence.peak_bytes)}else{None},
            "outstanding_upload_last_bytes":if self.observe_uploads{Some(self.fence.bytes())}else{None},
            "upload_batches_completed":self.fence.completed,"upload_fence_wait_frames":self.fence_wait_frames,
            "gpu_drains":self.drains,"texture_proofs":self.texture_proofs,
            "phase_frame_bins_100ms":self.phase_frame_bins,
            "gpu_texture_cache_budget_bytes":self.texture_budget,"gpu_texture_payload_peak_bytes":self.gpu.peak_bytes,
            "extra_frozen_override_texture_bytes":4*1024*1024,"thumbnail_texture_payload_bytes":12*64*64*4,
            "gpu_memory_note":"Payload excludes driver residency, staging, targets, fonts and thumbnails; OS process GPU counters are recorded separately by runner",
            "idle_seconds":self.idle_duration().as_secs_f64(),"closed_seconds":self.closed_duration().as_secs_f64(),"idle_ui_frames":self.idle_frames,"cycles_completed":self.cycle/10,
            "frozen_simulation_ms":self.frozen,"pause_png_refresh_verified":self.pause_verified,
            "render_target_readbacks":self.captures,"phases":self.phase_rows,"ui_frames":self.frames,
            "wall_seconds":self.started.elapsed().as_secs_f64(),"system_window_capture_used":false,"ui_passes":self.ui_passes,
            "viewport_visibility_counts":{"visible":self.visibility_counts[0],"occluded_or_minimized":self.visibility_counts[1],"unknown":self.visibility_counts[2]},
            "not_tested":["stable behavior","physical drag or input latency","IME/high DPI","GPU execution timestamps","isolated group compositing/effects","real audio/gameplay","history","macOS/Linux"]});
        result
            .as_object_mut()
            .unwrap()
            .extend(details.as_object().unwrap().clone());
        let loader = self.decoder.as_ref().map(|q| json!({
            "worker_count":1,"max_in_flight":q.outstanding_limit(),"result_channel_capacity":q.outstanding_limit(),
            "max_result_bytes":MAX_IMAGE_BYTES,"max_result_plus_ui_handoff_bytes":(q.outstanding_limit()+1)*MAX_IMAGE_BYTES,
            "upload_bytes_limit_per_frame":UPLOAD_BYTES_PER_FRAME,"upload_count_limit_per_frame":UPLOADS_PER_FRAME,
            "started":q.stats.started,"decoded_including_thumbnails_and_override":q.stats.decoded,
            "stale_results_rejected":q.stats.stale_results,"skipped_before_decode":q.stats.skipped_before_decode,
            "replaced_queued_jobs":q.stats.replaced_queued,"decode_including_thumbnails_ms":stats(&q.stats.decode_ms),"idle_at_finish":q.is_idle(),
            "cancellation":"Generation replacement; running PNG decoder finishes before cancellation is observed; no thread detach",
            "bounds_note":"Decoded output/handoff payload only; compressed bytes, decoder scratch, cache, egui upload references and GPU staging are separate"}));
        result.as_object_mut().unwrap().extend(json!({
            "async_png_loading":self.decoder.is_some(),"loader":loader,
            "paced_loading":self.paced_loading,"profile_memory":self.profile_memory,"no_original_upload":self.no_original_upload,"no_original_draw":self.no_original_draw,
            "document_load_wall_ms":stats(&self.switch_ms),"document_request_cpu_ms":stats(&self.request_ms),"document_requests":self.request_count,
            "loading_cpu_frame_ms":stats(&self.loading_frame_cpu),"metadata_validation_wall_ms":self.metadata_ms,
            "initial_load_request_to_textures_queued_ms":self.initial_load_ms,
            "max_upload_bytes_in_one_frame":self.uploads.values().map(|r|r.0).max().unwrap_or(0),
            "max_upload_count_in_one_frame":self.uploads.values().map(|r|r.1).max().unwrap_or(0),
            "rapid_switch_winner_verified":self.rapid_switch_verified,
            "initial_metadata_io":if self.decoder.is_some(){"One background metadata task before PNG decoding; UI thread validates reference workload shape and queues textures"}else{"Synchronous manifest/path/PNG header validation"},
            "document_ready_note":"All required texture deltas queued; not GPU execution/presentation completion"
        }).as_object().unwrap().clone());
        let raw = json!({"schema_version":1,"loading_cpu_frame_ms":self.loading_frame_cpu,
            "edit_cpu_frame_ms":self.frame_cpu,"document_load_wall_ms":self.switch_ms,"document_request_cpu_ms":self.request_ms,
            "edit_to_gpu_readback_ms":self.latencies,"edit_interval_ms":self.edit_intervals,
            "uploads_by_frame":self.uploads.iter().map(|(frame,(bytes,count))|json!({"frame":frame,"bytes":bytes,"count":count})).collect::<Vec<_>>()});
        std::fs::write(
            self.output.join("timing-samples.json"),
            serde_json::to_vec_pretty(&raw).unwrap(),
        )
        .expect("timing samples");
        std::fs::write(
            self.output.join("result.json"),
            serde_json::to_vec_pretty(&result).unwrap(),
        )
        .expect("write result");
        println!("Standard-load result: {}", self.output.display());
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }
    fn readbacks(&mut self, ctx: &egui::Context) -> Result<(), String> {
        while let Ok(readback) = self.rx.try_recv() {
            self.in_flight = false;
            if readback.tag < EDITS {
                let scale = ctx.pixels_per_point();
                let mut strip = Vec::new();
                for y in 0..40 {
                    for x in 0..100 {
                        let px = ((self.canvas.min.x + x as f32) * scale) as usize;
                        let py = ((self.canvas.min.y + y as f32) * scale) as usize;
                        strip.push(
                            *readback
                                .pixels
                                .pixels
                                .get(py * readback.pixels.size[0] + px)
                                .ok_or("Edit strip outside render target")?,
                        );
                    }
                }
                if self.previous_edit_pixels.as_ref() == Some(&strip) {
                    return Err("Edit readback repeated old transform pixels".into());
                }
                self.previous_edit_pixels = Some(strip);
                self.latencies.push(
                    readback
                        .finished
                        .duration_since(self.edit_started.take().ok_or("Unexpected edit readback")?)
                        .as_secs_f64()
                        * 1000.0,
                );
            } else if readback.tag >= 1000 {
                let image = readback.pixels;
                let scale = ctx.pixels_per_point();
                let x = (self.canvas.min.x * scale) as usize;
                let y = (self.canvas.min.y * scale) as usize;
                let width = ((self.canvas.width() * scale) as usize).min(image.size[0] - x);
                // Exclude time-dependent marker at y=570; all textured quads are above y=520.
                let height = ((520.0 * scale) as usize).min(image.size[1] - y);
                let mut rgba = Vec::with_capacity(width * height * 4);
                for row in y..y + height {
                    rgba.extend(
                        image.pixels[row * image.size[0] + x..row * image.size[0] + x + width]
                            .iter()
                            .flat_map(egui::Color32::to_array),
                    );
                }
                let name = format!("texture-proof-{}.png", readback.tag - 1000);
                image::save_buffer(
                    self.output.join(&name),
                    &rgba,
                    width as u32,
                    height as u32,
                    image::ColorType::Rgba8,
                )
                .map_err(|e| e.to_string())?;
                self.texture_proofs.push(json!({"cycle":readback.tag-1000,"document":self.current,"file":name,"size":[width,height]}));
            } else {
                let image = readback.pixels;
                let scale = ctx.pixels_per_point();
                let min_x = (self.canvas.min.x * scale).max(0.0) as usize;
                let max_x = ((self.canvas.max.x * scale) as usize).min(image.size[0]);
                let min_y = (self.canvas.min.y * scale).max(0.0) as usize;
                let max_y = ((self.canvas.max.y * scale) as usize).min(image.size[1]);
                let mut crop = Vec::new();
                for y in min_y..max_y {
                    crop.extend_from_slice(
                        &image.pixels[y * image.size[0] + min_x..y * image.size[0] + max_x],
                    );
                }
                if crop.is_empty() {
                    return Err("Empty readback canvas".into());
                }
                let rgba: Vec<_> = image
                    .pixels
                    .iter()
                    .flat_map(egui::Color32::to_array)
                    .collect();
                let name = format!("frozen-{}.png", readback.tag - EDITS);
                image::save_buffer(
                    self.output.join(&name),
                    &rgba,
                    image.size[0] as u32,
                    image.size[1] as u32,
                    image::ColorType::Rgba8,
                )
                .map_err(|e| e.to_string())?;
                self.captures.push(name);
                let crop: Vec<_> = crop.iter().flat_map(egui::Color32::to_array).collect();
                if self.phase == Phase::Before {
                    self.before = Some(crop);
                    // A separate view override keeps immutable source/cache identities intact.
                    self.apply_override(ctx)?;
                } else if self.phase == Phase::After {
                    if self.before.as_ref() == Some(&crop)
                        || self.frozen != Some(self.probe.observe().simulation_ms)
                    {
                        return Err(
                            "Frozen PNG update did not change pixels or advanced time".into()
                        );
                    }
                    self.pause_verified = Some(true);
                    self.enter(Phase::IdleSettle);
                } else {
                    return Err("Unexpected proof readback".into());
                }
            }
        }
        Ok(())
    }
    fn step(&mut self, ctx: &egui::Context) -> Result<(), String> {
        self.progress_uploads(ctx)?;
        self.probe
            .tick(self.started.elapsed().as_millis() as u64)
            .map_err(|e| format!("{e:?}"))?;
        if let Some((result, elapsed)) = self.metadata_task.as_mut().and_then(MetadataTask::poll) {
            self.metadata_ms = elapsed;
            self.metadata_task = None;
            self.install_workload(ctx, result?)?;
        }
        self.loading_work_this_frame |= self.pending.is_some();
        self.pump_loading(ctx)?;
        self.readbacks(ctx)?;
        if self.drain_pending && self.frames > self.drain_frame {
            self.drain_pending = false;
            self.event("DrainBefore");
            let start = Instant::now();
            let status = self
                .render_state
                .as_ref()
                .unwrap()
                .device
                .poll(eframe::wgpu::PollType::Wait {
                    submission_index: None,
                    timeout: Some(Duration::from_secs(2)),
                })
                .map_err(|e| format!("GPU diagnostic drain failed: {e:?}"))?;
            self.drains.push(json!({"cycle":self.cycle,"wall_ms":start.elapsed().as_secs_f64()*1000.0,"status":format!("{status:?}")}));
            self.event("DrainAfter");
        }
        match self.phase {
            Phase::Empty if self.entered.elapsed() >= self.empty_duration() => {
                self.event("InitialLoadStart");
                self.initial_started = Some(Instant::now());
                if self.decoder.is_some() {
                    let ctx = ctx.clone();
                    let paced = self.paced_loading;
                    self.metadata_task = Some(MetadataTask::new(self.root.clone(), move || {
                        if paced {
                            wake_loading(&ctx);
                        } else {
                            ctx.request_repaint();
                        }
                    }));
                    self.enter(Phase::Loading);
                    return Ok(());
                }
                let metadata_start = Instant::now();
                let workload = Workload::open(&self.root)?;
                self.metadata_ms = metadata_start.elapsed().as_secs_f64() * 1000.0;
                self.install_workload(ctx, workload)?;
            }
            Phase::Warm if self.entered.elapsed() >= self.warm_duration() => {
                self.enter(Phase::Edit)
            }
            Phase::Edit if self.edit == self.edit_count() && !self.in_flight => {
                self.probe
                    .pause(true, self.started.elapsed().as_millis() as u64)
                    .unwrap();
                self.frozen = Some(self.probe.observe().simulation_ms);
                self.enter(Phase::Before);
            }
            Phase::IdleSettle if self.entered.elapsed() >= Duration::from_secs(2) => {
                self.enter(Phase::Idle);
            }
            Phase::Before
                if !self.readbacks_enabled
                    && self.frames > self.entered_frame
                    && self.pending.is_none() =>
            {
                self.apply_override(ctx)?;
            }
            Phase::After if !self.readbacks_enabled && self.frames > self.entered_frame => {
                self.enter(Phase::IdleSettle)
            }
            Phase::Idle if self.entered.elapsed() >= self.idle_duration() => {
                self.material_override = None;
                self.enter(Phase::RapidSwitch);
                // Same request sequence for both paths, with no artificial worker delay.
                // The async path must invalidate obsolete plans and retain the winner.
                for document in [1, 2, 9] {
                    self.load(ctx, document)?;
                }
                if self.decoder.is_none() {
                    self.rapid_switch_verified = Some(self.current == 9);
                    self.gpu.reset_bindings();
                    self.load(ctx, 9)?;
                    self.cache_only_reload = true;
                    self.enter(Phase::Cycle);
                }
            }
            Phase::Cycle
                if self.pending.is_none()
                    && !self.in_flight
                    && !self.proof_pending
                    && Instant::now() >= self.next_frame =>
            {
                if self.cycle == self.cycle_count() {
                    self.lifetime_event(
                        "TexturesClear",
                        json!({"created":self.gpu.created,"updates":self.gpu.updated}),
                    );
                    self.cpu.clear();
                    self.gpu.clear();
                    self.thumbs.clear();
                    self.workload = None;
                    self.before = None;
                    self.previous_edit_pixels = None;
                    self.ready = None;
                    if let Some(decoder) = &mut self.decoder {
                        decoder.replace(Vec::new())?;
                    }
                    self.enter(Phase::ClosedSettle);
                    // Drain egui's deferred texture frees before observing the closed plateau.
                    ctx.request_repaint();
                } else {
                    let document = self.cycle % 10;
                    if document == 0 {
                        self.cpu.clear();
                        self.gpu.reset_bindings();
                        self.lifetime_event(
                            "BindingsReset",
                            json!({"physical_textures_retained":self.reuse_textures}),
                        );
                    }
                    self.load(ctx, document)?;
                    if self.decoder.is_none() {
                        self.complete_cycle();
                    }
                }
            }
            Phase::ClosedSettle if self.entered.elapsed() >= Duration::from_secs(2) => {
                self.enter(Phase::Closed)
            }
            Phase::Closed if self.entered.elapsed() >= self.closed_duration() => {
                self.finish(ctx, None)
            }
            _ => {}
        }
        if self.started.elapsed() > Duration::from_secs(240) {
            return Err("Benchmark timeout".into());
        }
        Ok(())
    }
    fn install_workload(&mut self, ctx: &egui::Context, workload: Workload) -> Result<(), String> {
        if workload.assets.len() != 100
            || workload.documents.len() != 10
            || workload.documents.iter().any(|d| d.len() != 50)
            || workload.assets.iter().any(|a| a.size != [1024, 1024])
        {
            return Err("Not the reference workload".into());
        }
        if self.decoder.is_none() {
            for asset in workload.assets.iter().take(12) {
                let image = decode_png(&asset.thumbnail)?;
                let bytes = image.pixels.len() * 4;
                let handle = ctx.load_texture("thumbnail", image, egui::TextureOptions::LINEAR);
                self.issued_textures.insert(handle.id());
                let row = self.uploads.entry(ctx.cumulative_frame_nr()).or_default();
                row.0 += bytes;
                row.1 += 1;
                self.thumbs.push(handle);
            }
        }
        self.workload = Some(workload);
        self.load(ctx, 0)?;
        if self.decoder.is_some() {
            self.enter(Phase::Loading);
        } else {
            self.first_decoded = self.decode_count;
            self.enter(Phase::Warm);
        }
        Ok(())
    }
    fn empty_duration(&self) -> Duration {
        if self.diagnostic || self.profile_memory {
            Duration::from_millis(100)
        } else {
            Duration::from_secs(3)
        }
    }
    fn closed_duration(&self) -> Duration {
        if self.lifetime.is_some() {
            return Duration::from_secs(10);
        }
        if self.diagnostic || self.profile_memory {
            Duration::from_secs(3)
        } else {
            Duration::from_secs(60)
        }
    }
    fn apply_override(&mut self, ctx: &egui::Context) -> Result<(), String> {
        if self.decoder.is_some() {
            self.begin_async(None)?;
            return Ok(());
        }
        let path = &self.workload.as_ref().ok_or("No workload")?.assets[99].path;
        let replacement = decode_png(path)?;
        self.material_override = Some(ctx.load_texture(
            "frozen-view-override",
            replacement,
            egui::TextureOptions::LINEAR,
        ));
        self.issued_textures
            .insert(self.material_override.as_ref().unwrap().id());
        let row = self.uploads.entry(ctx.cumulative_frame_nr()).or_default();
        row.0 += 4 * 1024 * 1024;
        row.1 += 1;
        self.enter(Phase::After);
        Ok(())
    }
    fn warm_duration(&self) -> Duration {
        if self.diagnostic || self.profile_memory {
            Duration::from_millis(100)
        } else {
            Duration::from_secs(1)
        }
    }
    fn idle_duration(&self) -> Duration {
        if self.diagnostic || self.profile_memory {
            Duration::from_secs(1)
        } else {
            Duration::from_secs(60)
        }
    }
    fn edit_count(&self) -> usize {
        if self.diagnostic || self.profile_memory {
            30
        } else {
            EDITS
        }
    }
    fn cycle_count(&self) -> usize {
        if self.diagnostic { 10 } else { 200 }
    }
}
impl eframe::App for Bench {
    fn logic(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if self.completed {
            return;
        }
        if self.previous_loading_frame {
            if let Some(seconds) = frame.info().cpu_usage {
                self.loading_frame_cpu.push(f64::from(seconds) * 1000.0);
            }
            self.previous_loading_frame = false;
        }
        if let Err(error) = self.step(ctx) {
            self.finish(ctx, Some(error));
            return;
        }
        self.trace_memory();
        if self.lifetime.is_some() && matches!(self.phase, Phase::Closed | Phase::ClosedSettle) {
            self.lifetime_event("ClosedCheckpoint", json!({}));
            ctx.request_repaint_after(Duration::from_millis(100));
        }
        let remaining = match self.phase {
            Phase::Idle => self.idle_duration().saturating_sub(self.entered.elapsed()),
            Phase::IdleSettle | Phase::ClosedSettle => {
                Duration::from_secs(2).saturating_sub(self.entered.elapsed())
            }
            Phase::Empty => self.empty_duration().saturating_sub(self.entered.elapsed()),
            Phase::Closed => self
                .closed_duration()
                .saturating_sub(self.entered.elapsed()),
            // Active deadlines are scheduled after UI processing, once their new
            // deadline is known. Scheduling an expired deadline here adds busy frames.
            _ => return,
        };
        ctx.request_repaint_after(remaining);
    }
    fn ui(&mut self, root: &mut egui::Ui, frame: &mut eframe::Frame) {
        let ctx = root.ctx().clone();
        self.ui_passes += 1;
        let frame_nr = ctx.cumulative_frame_nr();
        let first_pass = self.last_ui_frame != Some(frame_nr);
        if first_pass {
            let bin = (self.entered.elapsed().as_millis() / 100) as u64;
            *self
                .phase_frame_bins
                .entry(format!("{:?}", self.phase))
                .or_default()
                .entry(bin)
                .or_default() += 1;
            let causes = self
                .repaint_causes
                .entry(format!("{:?}", self.phase))
                .or_default();
            for cause in ctx.repaint_causes() {
                let file = cause.file.replace('\\', "/");
                let suffix = file.split("/src/").last().unwrap_or(&file);
                *causes
                    .entry(format!("{}:{} {}", suffix, cause.line, cause.reason))
                    .or_default() += 1;
            }
            self.last_ui_frame = Some(frame_nr);
            self.previous_loading_frame = self.loading_work_this_frame
                || matches!(
                    self.phase,
                    Phase::Loading | Phase::RapidSwitch | Phase::Cycle
                );
            self.loading_work_this_frame = false;
            self.frames += 1;
            if self.phase == Phase::Idle {
                self.idle_frames += 1;
            }
            if self.phase == Phase::Warm {
                self.next_frame = Instant::now() + PERIOD;
            }
            let visibility = match ctx.input(|i| i.viewport().visible()) {
                Some(true) => 0,
                Some(false) => 1,
                None => 2,
            };
            self.visibility_counts[visibility] += 1;
        }
        if self.phase == Phase::Edit && first_pass {
            if let Some(seconds) = frame.info().cpu_usage {
                self.frame_cpu.push(f64::from(seconds) * 1000.0);
            }
            if self.edit < self.edit_count()
                && !self.in_flight
                && self.last_edit.is_none_or(|time| time.elapsed() >= PERIOD)
            {
                let now = Instant::now();
                if let Some(previous) = self.last_edit {
                    self.edit_intervals
                        .push(now.duration_since(previous).as_secs_f64() * 1000.0);
                }
                self.last_edit = Some(now);
                self.edit_started = Some(now);
                self.next_frame = now + PERIOD;
                self.edit += 1;
                if self.readback_edits && self.readbacks_enabled {
                    self.capture(&ctx, self.edit - 1);
                }
            }
        }
        egui::Panel::top("scene").show(root, |ui| {
            ui.label(format!(
                "P0 standard PNG load / {:?} / 60 Hz target / editor freeze {}",
                self.phase,
                self.probe.observe().paused
            ));
        });
        egui::Panel::left("library")
            .default_size(180.)
            .show(root, |ui| {
                ui.heading("素材 / 日本語 PNG");
                ui.label("100 originals / 12 thumbnails");
                show_resource_list(ui, "assets", self.thumbs.is_empty(), |ui| {
                    for (id, texture) in self.thumbs.iter().enumerate() {
                        ui.horizontal(|ui| {
                            ui.image((texture.id(), egui::vec2(40., 40.)));
                            ui.label(format!("Asset {id:03}"));
                        });
                    }
                });
            });
        egui::Panel::right("layers")
            .default_size(210.)
            .show(root, |ui| {
                ui.heading("Layers / parameters");
                ui.label(format!("Document {} / edit {}", self.current, self.edit));
                ui.label(format!("CPU decoded: {} MiB", self.cpu.bytes / 1024 / 1024));
                ui.label(format!(
                    "Texture payload: {} MiB",
                    self.gpu.bytes() / 1024 / 1024
                ));
                show_resource_list(ui, "layers", self.workload.is_none(), |ui| {
                    if let Some(w) = &self.workload {
                        for (id, l) in w.documents[self.current].iter().enumerate() {
                            ui.label(format!("{id:02} → asset {} / α {:.2}", l.asset, l.opacity));
                        }
                    }
                });
            });
        egui::CentralPanel::default().show(root, |ui| {
            let (_, painter) = ui.allocate_painter(ui.available_size(), egui::Sense::hover());
            self.canvas = painter.clip_rect();
            painter.rect_filled(self.canvas, 0., egui::Color32::from_rgb(18, 22, 30));
            if let Some(w) = &self.workload {
                for (id, layer) in w.documents[self.current].iter().enumerate() {
                    if self.no_original_draw {
                        continue;
                    }
                    let texture = if layer.asset == 0 && self.material_override.is_some() {
                        self.material_override.as_ref()
                    } else {
                        self.gpu.get(layer.asset)
                    };
                    if let Some(texture) = texture {
                        let offset = if id == 0 { (self.edit % 31) as f32 } else { 0. };
                        let rect = egui::Rect::from_min_size(
                            self.canvas.min + egui::vec2(layer.x + offset, layer.y),
                            egui::Vec2::splat(1024. * layer.scale),
                        );
                        painter.image(
                            texture.id(),
                            rect,
                            egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1., 1.)),
                            egui::Color32::from_white_alpha((layer.opacity * 255.).round() as u8),
                        );
                    }
                }
                let x = (self.probe.observe().simulation_ms % 500) as f32;
                painter.circle_filled(
                    self.canvas.min + egui::vec2(x, 570.),
                    4.,
                    egui::Color32::WHITE,
                );
            }
        });
        if self.readbacks_enabled
            && matches!(self.phase, Phase::Before | Phase::After)
            && !self.in_flight
            && self.pending.is_none()
        {
            self.capture(&ctx, EDITS + usize::from(self.phase == Phase::After));
        }
        if self.proof_pending && !self.in_flight {
            self.proof_pending = false;
            self.capture(&ctx, 1000 + self.cycle);
        }
        if self.pending.is_some() || self.metadata_task.is_some() {
            // Wake on results as well; this deadline keeps the loading view responsive.
            ctx.request_repaint_after(PERIOD);
        } else if matches!(
            self.phase,
            Phase::Warm | Phase::Edit | Phase::Before | Phase::After | Phase::Cycle
        ) {
            ctx.request_repaint_after(
                self.next_frame
                    .saturating_duration_since(Instant::now())
                    .max(Duration::from_millis(1)),
            );
        }
    }
}
fn main() -> eframe::Result {
    let mut args = std::env::args().skip(1);
    let root = PathBuf::from(args.next().expect("fixture directory"));
    let output = PathBuf::from(args.next().expect("new artifact directory"));
    let flags: Vec<_> = args.collect();
    assert!(
        flags.iter().all(|f| matches!(
            f.as_str(),
            "--diagnostic"
                | "--no-edit-readback"
                | "--no-readback"
                | "--memory-usage"
                | "--async-load"
                | "--paced-loading"
                | "--profile-memory"
                | "--no-original-upload"
                | "--no-original-draw"
                | "--reuse-textures"
                | "--small-texture-cache"
                | "--texture-proof"
                | "--drain-uploads"
                | "--bounded-uploads"
                | "--lifetime-trace"
                | "--backend-dx12"
                | "--backend-vulkan"
        )),
        "Unknown benchmark flag"
    );
    assert!(
        !flags
            .iter()
            .any(|f| f == "--bounded-uploads" || f == "--lifetime-trace")
            || flags.iter().any(|f| f == "--async-load"),
        "Receipts require async loading"
    );
    assert!(
        !flags.iter().any(|f| f == "--lifetime-trace")
            || flags.iter().any(|f| f == "--profile-memory"),
        "Lifetime logging is diagnostic only"
    );
    let diagnostic = flags.iter().any(|f| f == "--diagnostic");
    let readbacks_enabled = !flags.iter().any(|f| f == "--no-readback");
    let readback_edits = readbacks_enabled && !flags.iter().any(|f| f == "--no-edit-readback");
    let memory_usage = flags.iter().any(|f| f == "--memory-usage");
    assert!(
        !flags.iter().any(|f| f == "--paced-loading") || flags.iter().any(|f| f == "--async-load"),
        "Pacing requires async loading"
    );
    assert!(
        !flags.iter().any(|f| f == "--no-original-upload")
            || (flags.iter().any(|f| f == "--profile-memory")
                && !readbacks_enabled
                && flags.iter().any(|f| f == "--async-load")),
        "Upload ablation is diagnostic only, without readbacks"
    );
    assert!(
        !flags.iter().any(|f| f == "--no-original-draw")
            || (flags.iter().any(|f| f == "--profile-memory") && !readbacks_enabled),
        "Draw ablation is diagnostic only"
    );
    assert!(
        !flags.iter().any(|f| f == "--drain-uploads")
            || flags.iter().any(|f| f == "--profile-memory"),
        "GPU drain requires profiling"
    );
    assert!(
        !flags.iter().any(|f| f == "--texture-proof") || readbacks_enabled,
        "Texture proof requires readback"
    );
    assert!(
        !(flags.iter().any(|f| f == "--backend-dx12")
            && flags.iter().any(|f| f == "--backend-vulkan")),
        "Choose one backend"
    );
    let artifacts = std::env::current_dir()
        .unwrap()
        .join("artifacts/p0")
        .canonicalize()
        .unwrap();
    assert_eq!(
        output.parent().and_then(|p| p.canonicalize().ok()),
        Some(artifacts.clone()),
        "Output must be direct child of artifacts/p0"
    );
    assert!(!output.exists(), "Preserve existing artifacts");
    std::fs::create_dir(&output).unwrap();
    let mut setup = eframe::egui_wgpu::WgpuSetupCreateNew::without_display_handle();
    if flags.iter().any(|f| f == "--backend-dx12") {
        setup.instance_descriptor.backends = eframe::wgpu::Backends::DX12;
    }
    if flags.iter().any(|f| f == "--backend-vulkan") {
        setup.instance_descriptor.backends = eframe::wgpu::Backends::VULKAN;
    }
    let original_descriptor = setup.device_descriptor;
    setup.device_descriptor = Arc::new(move |adapter| {
        let mut descriptor = original_descriptor(adapter);
        descriptor.memory_hints = if memory_usage {
            eframe::wgpu::MemoryHints::MemoryUsage
        } else {
            eframe::wgpu::MemoryHints::Performance
        };
        descriptor
    });
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100., 720.])
            .with_visible(true),
        renderer: eframe::Renderer::Wgpu,
        wgpu_options: eframe::egui_wgpu::WgpuConfiguration {
            wgpu_setup: setup.into(),
            surface: eframe::egui_wgpu::SurfaceConfig {
                present_mode: eframe::wgpu::PresentMode::AutoNoVsync,
                ..eframe::egui_wgpu::SurfaceConfig::LOW_LATENCY
            },
            ..Default::default()
        },
        persist_window: false,
        ..Default::default()
    };
    eframe::run_native(
        "P0 standard PNG benchmark",
        options,
        Box::new(move |cc| {
            Ok(Box::new(Bench::new(
                cc,
                root,
                output,
                diagnostic,
                readback_edits,
                readbacks_enabled,
                &flags,
            )))
        }),
    )
}
