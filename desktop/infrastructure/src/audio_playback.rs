//! Control-side assembly; the native consumer never sees ProjectSession.
use crate::{
    BlobStore, BuildAdmission, OutputFence, OutputProducer, PreparationWorker, SkinAudioController,
    WavDecoder,
};
use skin_editor_application::{ProjectSession, values::AudioComponentId};
use skin_editor_audio::{Channel, PcmBudget, TriggerReceipt};

pub struct AudioPlayback {
    controller: SkinAudioController,
    worker: PreparationWorker,
    output_invalidation: u64,
    output_safety: Option<crate::native_audio::OutputSafety>,
}
impl AudioPlayback {
    pub fn new(rate: u32, channels: u16, budget: PcmBudget) -> Result<Self, String> {
        let mut controller = SkinAudioController::with_budget(rate, channels, budget)
            .map_err(|e| format!("{e:?}"))?;
        controller.runtime().enable_minimum_conversion();
        Ok(Self {
            controller,
            worker: PreparationWorker::new(WavDecoder).map_err(|e| e.to_string())?,
            output_invalidation: 0,
            output_safety: None,
        })
    }
    fn refresh(&mut self, session: &ProjectSession, output: &OutputProducer) -> Result<(), String> {
        if let Some(safety) = &self.output_safety {
            if !safety.is_for(output) {
                return Err("playback is bound to another output".into());
            }
        } else {
            self.output_safety = Some(output.safety());
        }
        if output.stats().failed {
            self.close(output)?;
            return Err("native output failed; explicit rebuild required".into());
        }
        if let Err(error) = session.synchronize_audio(&mut self.controller) {
            output.invalidate(true, true)?;
            return Err(format!("{error:?}"));
        }
        self.fence_if_changed(output)
    }
    fn fence_if_changed(&mut self, output: &OutputProducer) -> Result<(), String> {
        let revision = self.controller.output_invalidation();
        if revision != self.output_invalidation {
            output.invalidate(true, true)?;
            self.output_invalidation = revision;
        }
        Ok(())
    }
    /// Control tick is nonblocking, bounded to the one active decode and four queue slots.
    pub fn step(
        &mut self,
        session: &ProjectSession,
        store: &BlobStore,
        output: &mut OutputProducer,
    ) -> Result<Vec<BuildAdmission>, String> {
        if output.stats().failed {
            self.close(output)?;
            return Err("native output failed; explicit rebuild required".into());
        }
        self.refresh(session, output)?;
        if let Some(result) = self.worker.try_result() {
            self.controller
                .finish_preparation(session, result)
                .map_err(|e| format!("{e:?}"))?;
        }
        let admissions = self
            .controller
            .admit_builds(session, store)
            .map_err(|e| format!("{e:?}"))?;
        self.fence_if_changed(output)?;
        if let Some(job) = self.controller.start_preparation()
            && let Err(job) = self.worker.try_submit(job)
        {
            let request = job.request();
            drop(job);
            self.controller
                .accept_prepared(
                    session,
                    request,
                    Err(skin_editor_audio::PrepareFailure::Busy),
                )
                .map_err(|e| format!("{e:?}"))?;
        }
        output.pump(self.controller.runtime())?;
        Ok(admissions)
    }
    pub fn trigger(
        &mut self,
        session: &ProjectSession,
        output: &OutputProducer,
        lane: Channel,
        component: AudioComponentId,
    ) -> Result<TriggerReceipt, String> {
        self.refresh(session, output)?;
        let runtime = self.controller.runtime();
        runtime
            .trigger(runtime.token(lane), component, runtime.mix_frame())
            .map_err(|e| format!("{e:?}"))
    }
    pub fn scene_running(
        &mut self,
        output: &OutputProducer,
        running: bool,
    ) -> Result<OutputFence, String> {
        let runtime = self.controller.runtime();
        runtime
            .set_scene_running(runtime.token(Channel::Scene), running)
            .map_err(|e| format!("{e:?}"))?;
        output.invalidate(true, false)
    }
    pub fn stop_audition(&mut self, output: &OutputProducer) -> Result<OutputFence, String> {
        let runtime = self.controller.runtime();
        runtime
            .stop_audition(runtime.token(Channel::Audition))
            .map_err(|e| format!("{e:?}"))?;
        output.invalidate(false, true)
    }
    pub fn status(
        &mut self,
        component: AudioComponentId,
    ) -> Option<skin_editor_audio::BindingStatus> {
        self.controller.runtime().status(component)
    }
    /// Bounded startup/diagnostic wait. Interactive hosts use nonblocking step instead.
    pub fn wait_preparation(
        &mut self,
        session: &ProjectSession,
        output: &OutputProducer,
        timeout: std::time::Duration,
    ) -> Result<bool, String> {
        self.refresh(session, output)?;
        let Some(result) = self.worker.wait_result(timeout) else {
            return Ok(false);
        };
        self.controller
            .finish_preparation(session, result)
            .map_err(|e| format!("{e:?}"))?;
        self.fence_if_changed(output)?;
        Ok(true)
    }
    pub fn scene_frame(&mut self) -> u64 {
        self.controller.runtime().scene_frame()
    }
    /// Explicit initial prefill before native play; not an interactive steady-state target.
    pub fn prefill(&mut self, output: &mut OutputProducer) -> Result<(), String> {
        output.prefill(self.controller.runtime())?;
        Ok(())
    }
    pub fn close(&mut self, output: &OutputProducer) -> Result<OutputFence, String> {
        let fence = output.invalidate(true, true)?;
        self.controller.close();
        self.worker.close();
        Ok(fence)
    }
}
impl Drop for AudioPlayback {
    fn drop(&mut self) {
        if let Some(safety) = &self.output_safety {
            safety.close();
        }
        self.controller.close();
        self.worker.close();
    }
}
