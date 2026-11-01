//! Same-device complete resource lifetimes, separate from ordinary 300 s runs.
//! No readback, native supply changes, OS trimming or forced collection.
use super::*;
use eframe::App;
use eframe::egui_wgpu::RenderState;

const CYCLES: usize = 20;
const SETTLE: Duration = Duration::from_secs(10);
const STABLE: Duration = Duration::from_secs(65);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stage {
    Playing,
    PauseSettle,
    Paused,
    CloseSettle,
    Closed,
    CycleGap,
}
impl Stage {
    fn stable(self) -> bool {
        matches!(self, Self::Paused | Self::Closed)
    }
}
struct Recovery {
    ctx: egui::Context,
    render: RenderState,
    active: Option<Combined>,
    path: PathBuf,
    cycle: usize,
    stage: Stage,
    deadline: Option<Instant>,
    frames: [u64; 3],
    snapshots: Vec<String>,
    failed: Option<String>,
    done: bool,
}
impl Recovery {
    fn open(&mut self) -> Result<(), String> {
        if self.active.is_some() || self.cycle >= CYCLES {
            return Err("invalid lifecycle admission".into());
        }
        let path = self.path.join(format!("cycle-{:02}", self.cycle + 1));
        std::fs::create_dir(&path).map_err(error)?;
        phase(&format!("cycle_{:02}_loading", self.cycle + 1));
        // Each cycle owns a fresh project, raw/RGBA budgets, import/frame workers,
        // composer/staging cache, PCM preparation/supply owner and native stream.
        // Only eframe's Device/Queue, presentation surface and egui are retained.
        let mut active = Combined::create(
            &self.ctx,
            &self.render,
            path,
            self.cycle != 0,
            true,
            false,
            true,
        )?;
        active.hold_completion = true;
        active.diagnostic_cycle = Some(self.cycle);
        self.active = Some(active);
        self.stage = Stage::Playing;
        self.deadline = None;
        Ok(())
    }
    fn begin(&mut self, stage: Stage, duration: Duration, label: &str) {
        self.stage = stage;
        self.deadline = Some(Instant::now() + duration);
        phase(label);
    }
    fn snapshot(&mut self, suffix: &str) -> Result<(), String> {
        if self.snapshots.len() >= 44 {
            return Err("lifecycle allocator trace overflow".into());
        }
        self.snapshots.push(crate::gpu_observation::snapshot(
            &self.render.device,
            &format!("cycle_{:02}_{suffix}", self.cycle + 1),
        )?);
        Ok(())
    }
    fn close(&mut self) -> Result<(), String> {
        let active = self.active.as_mut().ok_or("missing active lifecycle")?;
        active.finish()?;
        active.report("report.json")?;
        if !active.cleanup || active.editor.host.needs_work() {
            return Err("resource leases or frame work survived shutdown".into());
        }
        // Drop the entire Editor (including the importer and disabled UI audio
        // adapter), not just its project or native stream. Drops join workers.
        drop(self.active.take());
        self.snapshot("closed_start")?;
        if self.cycle == 0 || self.cycle + 1 == CYCLES {
            self.begin(
                Stage::CloseSettle,
                SETTLE,
                &format!("cycle_{:02}_settling", self.cycle + 1),
            );
        } else {
            self.begin(
                Stage::CycleGap,
                Duration::from_secs(2),
                &format!("cycle_{:02}_closed", self.cycle + 1),
            );
        }
        Ok(())
    }
    fn tick(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) -> Result<(), String> {
        if self.stage == Stage::Playing {
            let active = self.active.as_mut().ok_or("missing playing lifecycle")?;
            active.logic(ctx, frame);
            if let Some(e) = &active.failed {
                return Err(e.clone());
            }
            if active.completed {
                active.playing_frames =
                    active.output.as_ref().unwrap().stats().frames - active.frames_at_start;
                if self.cycle == 0 {
                    // Freeze both channels without dropping prepared assets or
                    // the active source/output/staging resources.
                    stop(
                        active.client.as_mut().unwrap(),
                        active.output.as_ref().unwrap(),
                    )?;
                    self.snapshot("paused_start")?;
                    self.begin(Stage::PauseSettle, SETTLE, "pause_settling");
                } else {
                    self.close()?;
                }
            }
            return Ok(());
        }
        if self
            .deadline
            .is_some_and(|deadline| Instant::now() < deadline)
        {
            return Ok(());
        }
        match self.stage {
            Stage::PauseSettle => self.begin(Stage::Paused, STABLE, "paused"),
            Stage::Paused => {
                self.snapshot("paused_end")?;
                self.close()?;
            }
            Stage::CloseSettle => self.begin(
                Stage::Closed,
                STABLE,
                if self.cycle == 0 {
                    "closed_initial"
                } else {
                    "closed_final"
                },
            ),
            Stage::Closed | Stage::CycleGap => {
                self.snapshot("closed_end")?;
                self.cycle += 1;
                if self.cycle == CYCLES {
                    let text = format!(
                        "{{\"schema\":1,\"cycles\":{CYCLES},\"gpu_memory_usage_hint\":true,\"reusable_staging\":true,\"settle_seconds\":10,\"stable_seconds\":65,\"stable_ui_frames\":{:?},\"gpu_allocator_snapshots\":[{}]}}\n",
                        self.frames,
                        self.snapshots.join(",")
                    );
                    std::fs::write(self.path.join("report.json"), text).map_err(error)?;
                    self.done = true;
                } else {
                    self.open()?;
                }
            }
            Stage::Playing => unreachable!(),
        }
        Ok(())
    }
}
impl eframe::App for Recovery {
    fn logic(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if let Err(e) = self.tick(ctx, frame) {
            self.failed = Some(e);
        }
        if self.failed.is_some() || self.done {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        } else if let Some(deadline) = self.deadline {
            ctx.request_repaint_after(deadline.saturating_duration_since(Instant::now()));
        }
    }
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        if self.stage == Stage::Playing {
            if let Some(active) = &mut self.active {
                active.ui(ui, frame);
            }
        } else {
            if self.stage.stable() {
                let index = if self.stage == Stage::Paused {
                    0
                } else if self.cycle == 0 {
                    1
                } else {
                    2
                };
                self.frames[index] += 1;
            }
            ui.label("P2-A3 resource recovery: waiting for lifecycle deadline.");
        }
    }
    fn on_exit(&mut self) {
        if !self.done {
            if let Some(active) = &mut self.active {
                active.on_exit();
            }
            let _ = std::fs::write(
                self.path.join("failure.txt"),
                self.failed
                    .as_deref()
                    .unwrap_or("lifecycle closed before completion"),
            );
        }
    }
}
pub fn run(destination: &str) -> Result<(), String> {
    let path = PathBuf::from(destination);
    if !path.is_absolute() || path.exists() {
        return Err("new absolute lifecycle output required".into());
    }
    std::fs::create_dir_all(&path).map_err(error)?;
    let report = path.join("report.json");
    eframe::run_native(
        "P2-A3 staging recovery (self-authored resources)",
        eframe::NativeOptions {
            renderer: eframe::Renderer::Wgpu,
            wgpu_options: crate::gpu_observation::configuration(true),
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1180., 800.])
                .with_min_inner_size([820., 600.]),
            ..Default::default()
        },
        Box::new(move |context| {
            let mut app = Recovery {
                ctx: context.egui_ctx.clone(),
                render: context.wgpu_render_state.as_ref().unwrap().clone(),
                active: None,
                path,
                cycle: 0,
                stage: Stage::Playing,
                deadline: None,
                frames: [0; 3],
                snapshots: vec![],
                failed: None,
                done: false,
            };
            app.open().map_err(std::io::Error::other)?;
            Ok(Box::new(app))
        }),
    )
    .map_err(error)?;
    if !report.is_file() {
        return Err("lifecycle incomplete; inspect failure.txt".into());
    }
    phase("done");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stable_observation_excludes_settling_and_quick_cycle_gaps() {
        assert_eq!(CYCLES, 20);
        assert_eq!(SETTLE.as_secs(), 10);
        assert!(STABLE.as_secs() >= 65);
        for stage in [
            Stage::Playing,
            Stage::PauseSettle,
            Stage::CloseSettle,
            Stage::CycleGap,
        ] {
            assert!(!stage.stable());
        }
        assert!(Stage::Paused.stable() && Stage::Closed.stable());
    }
}
