//! The two concrete transports used by the same bounded supply controls.
use crate::{OutputControl, output_fence::OutputMark, separated_output};
#[derive(Clone)]
pub(crate) enum SupplyOutputControl {
    Premixed(OutputControl),
    Separated(separated_output::Control),
}
impl SupplyOutputControl {
    pub fn is_separated(&self) -> bool {
        matches!(self, Self::Separated(_))
    }
    pub fn premixed(&self) -> Result<OutputControl, String> {
        match self {
            Self::Premixed(c) => Ok(c.clone()),
            Self::Separated(_) => {
                Err("premixed diagnostics unavailable for separated transport".into())
            }
        }
    }
    pub fn capture_origin(&self) -> Option<std::time::Instant> {
        self.premixed().ok().and_then(|c| c.capture_origin())
    }
    pub fn mark(&self) -> OutputMark {
        match self {
            Self::Premixed(c) => c.mark(),
            Self::Separated(c) => c.mark(),
        }
    }
    pub fn invalidate_skin(&self, scene: bool, audition: bool) -> Result<OutputMark, String> {
        match self {
            Self::Premixed(c) => c.invalidate_skin(scene, audition),
            Self::Separated(c) => c.invalidate(scene, audition, false),
        }
    }
    pub fn invalidate_scene(&self) -> Result<OutputMark, String> {
        match self {
            Self::Premixed(c) => c.invalidate_scene(),
            Self::Separated(c) => c.invalidate(true, false, true),
        }
    }
    pub fn background_acknowledged(&self, mark: OutputMark) -> bool {
        match self {
            Self::Premixed(c) => c.background_acknowledged(mark),
            Self::Separated(c) => c.background_acknowledged(mark),
        }
    }
    pub fn acknowledged(&self, mark: OutputMark) -> bool {
        match self {
            Self::Premixed(_) => true,
            Self::Separated(c) => c.acknowledged(mark),
        }
    }
    pub fn background_next(&self) -> u64 {
        match self {
            Self::Premixed(c) => c.stats().background_next,
            Self::Separated(c) => c.background_next(),
        }
    }
    pub fn is_closed(&self) -> bool {
        match self {
            Self::Premixed(c) => c.is_closed(),
            Self::Separated(c) => c.is_closed(),
        }
    }
    pub fn close(&self) {
        match self {
            Self::Premixed(c) => c.close(),
            Self::Separated(c) => c.close(),
        }
    }
}
