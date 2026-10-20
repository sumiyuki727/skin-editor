//! Platform-neutral P0 port. No native device, UI, project or host clock values.
use crate::audio_transport::{AudioObservation, Intent, Relocate};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutputError {
    ControlUnavailable,
}

/// One output session, with asynchronous, explicit replacement of a failed stream.
/// A rebuilt epoch starts paused with request zero and stream-relative microseconds.
/// None means opening, not an invitation to advance simulation using wall time.
pub trait Output {
    fn snapshot(&self) -> Option<AudioObservation>;
    fn control(&self, intent: Intent) -> Result<(), OutputError>;
    /// Recovery and seek both carry an explicit content origin, so replacing a
    /// stream cannot silently restart the source while the simulation continues.
    fn relocate(&self, target: Relocate) -> Result<(), OutputError>;
}
