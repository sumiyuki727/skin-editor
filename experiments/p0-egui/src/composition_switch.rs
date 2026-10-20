//! P0 read-only document handoff. No project writes, commands or history ownership.
use crate::{
    composition_loading::{DocumentLoader, Event},
    composition_view::{LiveComposition, PendingComposition, PipelineTask, Pipelines},
};
use eframe::egui;
use std::path::PathBuf;
pub struct DocumentSwitch {
    state: eframe::egui_wgpu::RenderState,
    loader: DocumentLoader,
    requested: Option<(u64, usize)>,
    active: Option<(usize, LiveComposition)>,
    pending: Option<(u64, usize, PendingComposition)>,
    pub stale_reads: usize,
    pub stale_gpu: usize,
    pub accepted: usize,
    pub error: Option<String>,
    pub uploads: Vec<serde_json::Value>,
    budget: crate::loading::UploadBudget,
    pub profile: Option<Vec<serde_json::Value>>,
    warmup: Option<PipelineTask>,
    graphics: Option<Pipelines>,
    pub pipeline_warmup_ms: Option<f64>,
    warmup_failed: bool,
}
impl DocumentSwitch {
    pub fn new(state: eframe::egui_wgpu::RenderState, ctx: &egui::Context) -> Self {
        let wake = ctx.clone();
        Self {
            state,
            loader: DocumentLoader::new(move || wake.request_repaint()),
            requested: None,
            active: None,
            pending: None,
            stale_reads: 0,
            stale_gpu: 0,
            accepted: 0,
            error: None,
            uploads: vec![],
            budget: Default::default(),
            profile: None,
            warmup: None,
            graphics: None,
            pipeline_warmup_ms: None,
            warmup_failed: false,
        }
    }
    pub fn warm_pipelines(&mut self, ctx: &egui::Context) {
        assert!(
            self.warmup.is_none()
                && self.graphics.is_none()
                && self.active.is_none()
                && self.pending.is_none()
        );
        let wake = ctx.clone();
        self.warmup = Some(PipelineTask::start(self.state.device.clone(), move || {
            wake.request_repaint()
        }));
    }
    pub fn request(&mut self, root: PathBuf, document: usize) -> Result<u64, String> {
        if self.warmup_failed {
            return Err("Pipeline worker failed; create a new preview session".into());
        }
        let ticket = self.loader.request(root, document)?;
        self.requested = Some((ticket, document));
        self.error = None;
        Ok(ticket)
    }
    pub fn cancel(&mut self) -> Result<(), String> {
        self.loader.cancel()?;
        self.requested = None;
        Ok(())
    }
    pub fn close(&mut self) -> Result<(), String> {
        self.cancel()?;
        self.active = None;
        Ok(())
    }
    pub fn active(&self) -> Option<(usize, &LiveComposition)> {
        self.active.as_ref().map(|(d, v)| (*d, v))
    }
    pub fn active_mut(&mut self) -> Option<(usize, &mut LiveComposition)> {
        self.active.as_mut().map(|(d, v)| (*d, v))
    }
    pub fn pending(&self) -> bool {
        self.requested.is_some()
            || self.pending.is_some()
            || !self.loader.is_idle()
            || self.warmup.is_some()
    }
    pub fn preparing(&self) -> bool {
        self.pending.is_some()
    }
    pub fn poll(&mut self, frame: u64, ctx: &egui::Context) -> Result<(), String> {
        if let Some(task) = &mut self.warmup
            && let Some(result) = task.poll()
        {
            self.warmup = None;
            match result {
                Ok((graphics, ms)) => {
                    self.graphics = Some(graphics);
                    self.pipeline_warmup_ms = Some(ms);
                }
                Err(e) => {
                    self.warmup_failed = true;
                    self.error = Some(e);
                    self.cancel()?;
                }
            }
        }
        self.budget.begin(frame);
        let profile_start = self.profile.as_ref().map(|_| std::time::Instant::now());
        let mut row = if profile_start.is_some() {
            serde_json::json!({"frame":frame})
        } else {
            serde_json::Value::Null
        };
        let ready = if let Some((_, _, candidate)) = &mut self.pending {
            let done = candidate.advance(&mut self.budget, ctx)?;
            if profile_start.is_some() {
                row["advance"] = serde_json::json!(candidate.last_step);
            }
            done
        } else {
            false
        };
        if ready {
            let start = profile_start.map(|_| std::time::Instant::now());
            let (ticket, doc, candidate) = self.pending.take().unwrap();
            self.uploads.push(candidate.metrics());
            if self.requested == Some((ticket, doc)) {
                let view = candidate.finish()?;
                self.active = Some((doc, view));
                self.requested = None;
                self.accepted += 1;
            } else {
                self.stale_gpu += 1;
            }
            if let Some(s) = start {
                row["retire_or_swap_ms"] = serde_json::json!(s.elapsed().as_secs_f64() * 1000.);
            }
        }
        // Retire an older GPU candidate before preparing another. The worker's
        // rendezvous can retain one newer CPU document meanwhile (40 MiB).
        if self.pending.is_none()
            && self.warmup.is_none()
            && let Some(event) = self.loader.poll()
        {
            match event {
                Event::Stale { .. } => self.stale_reads += 1,
                Event::Ready { ticket, result } => {
                    if let Some((current, doc)) = self.requested
                        && current == ticket
                    {
                        match result.and_then(|loaded| {
                            PendingComposition::prepared(
                                self.state.clone(),
                                loaded,
                                profile_start.is_some(),
                                self.graphics.clone(),
                            )
                        }) {
                            Ok(candidate) => {
                                if profile_start.is_some() {
                                    row["preparation"] = serde_json::json!(candidate.preparation);
                                }
                                self.pending = Some((ticket, doc, candidate));
                            }
                            Err(error) => {
                                self.error = Some(error);
                                self.requested = None;
                            }
                        }
                    }
                }
            }
        }
        if self.pending() {
            ctx.request_repaint_after(std::time::Duration::from_millis(16));
        }
        if let Some(s) = profile_start {
            row["total_ms"] = serde_json::json!(s.elapsed().as_secs_f64() * 1000.);
            self.profile.as_mut().unwrap().push(row);
        }
        Ok(())
    }
}
