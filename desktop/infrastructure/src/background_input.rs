use crate::{background_stream::BufferedBackground, separated_output::OutputError};
use skin_editor_audio::{BackgroundTrack, PcmBudget};
pub(crate) enum BackgroundInput {
    Memory(BackgroundTrack),
    Buffered(BufferedBackground),
}
impl From<BackgroundTrack> for BackgroundInput {
    fn from(t: BackgroundTrack) -> Self {
        Self::Memory(t)
    }
}
impl From<BufferedBackground> for BackgroundInput {
    fn from(t: BufferedBackground) -> Self {
        Self::Buffered(t)
    }
}
impl BackgroundInput {
    pub fn format(&self) -> (u32, u16) {
        match self {
            Self::Memory(t) => t.format(),
            Self::Buffered(t) => t.format(),
        }
    }
    pub fn frames(&self) -> u64 {
        match self {
            Self::Memory(t) => t.frames(),
            Self::Buffered(t) => t.frames(),
        }
    }
    pub fn position(&self) -> u64 {
        match self {
            Self::Memory(t) => t.position(),
            Self::Buffered(t) => t.position(),
        }
    }
    pub fn budget(&self) -> &PcmBudget {
        match self {
            Self::Memory(t) => t.pcm().budget(),
            Self::Buffered(t) => t.budget(),
        }
    }
    pub fn restore_position(&mut self, frame: u64) -> Result<(), OutputError> {
        match self {
            Self::Memory(t) => t
                .restore_position(frame)
                .map_err(|_| OutputError::InvalidPosition),
            Self::Buffered(t) => t
                .restore_position(frame)
                .map_err(|_| OutputError::BackgroundFault),
        }
    }
    pub fn set_running(&mut self, running: bool) {
        if let Self::Memory(t) = self {
            t.set_running(running);
        }
    }
    pub fn next_frame(&mut self) -> Result<Option<(u64, [f32; 2])>, OutputError> {
        match self {
            Self::Memory(t) => Ok(t.next_frame()),
            Self::Buffered(t) => t.next_frame().map_err(|_| OutputError::BackgroundFault),
        }
    }
}
