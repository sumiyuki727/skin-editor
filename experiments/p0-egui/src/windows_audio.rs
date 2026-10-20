//! Desktop assembly adapter; pure transport never imports WASAPI types.
use crate::{
    audio_output::{Output, OutputError},
    audio_transport::{AudioObservation, Intent, Relocate},
};
use p0_wasapi::{Device, Error, Phase, Snapshot};

pub struct WindowsOutput(pub Device);
impl WindowsOutput {
    pub fn spawn(wake: impl Fn() + Send + Sync + 'static) -> Result<Self, Error> {
        Device::spawn(wake).map(Self)
    }
    pub fn spawn_pcm(wake: impl Fn() + Send + Sync + 'static) -> Result<Self, Error> {
        Device::spawn_pcm(wake).map(Self)
    }
    pub fn spawn_clip(
        clip: std::sync::Arc<p0_wasapi::pcm::Clip>,
        wake: impl Fn() + Send + Sync + 'static,
    ) -> Result<Self, Error> {
        Device::spawn_clip(clip, wake).map(Self)
    }
    pub fn spawn_buffered(
        input: p0_wasapi::buffered::Receiver,
        wake: impl Fn() + Send + Sync + 'static,
    ) -> Result<Self, Error> {
        Device::spawn_buffered(input, wake).map(Self)
    }
    pub fn spawn_buffered_converting(
        input: p0_wasapi::buffered::Receiver,
        wake: impl Fn() + Send + Sync + 'static,
    ) -> Result<Self, Error> {
        Device::spawn_buffered_converting(input, wake).map(Self)
    }
    /// Diagnostic assembly can correlate native metadata with exactly this sample.
    pub fn observation(s: Snapshot) -> Option<AudioObservation> {
        (s.phase != Phase::Initializing).then_some(AudioObservation {
            epoch: s.epoch,
            request: s.request,
            sequence: s.sequence,
            position_us: s.position_us,
            paused: s.phase == Phase::Paused,
            fault: s.phase == Phase::Faulted,
        })
    }
}
impl Output for WindowsOutput {
    fn snapshot(&self) -> Option<AudioObservation> {
        Self::observation(self.0.snapshot())
    }
    fn control(&self, i: Intent) -> Result<(), OutputError> {
        self.0
            .request_in(i.epoch, i.request, i.paused)
            .map_err(|_| OutputError::ControlUnavailable)
    }
    fn relocate(&self, target: Relocate) -> Result<(), OutputError> {
        self.0
            .relocate(target.epoch, target.position_us)
            .map_err(|_| OutputError::ControlUnavailable)
    }
}
