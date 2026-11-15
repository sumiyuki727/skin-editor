//! Fixed close intent. These values never authorize host shutdown or activation.
use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CloseRequest {
    pub api_version: u16,
    pub key: CloseAdmissionKey,
    pub source: CaptureSource,
    pub purpose: ClosePurpose,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CloseAdmission {
    Accepted(CloseTicket),
    Existing(CloseTicket),
}
impl CloseAdmission {
    pub fn ticket(self) -> CloseTicket {
        match self {
            Self::Accepted(t) | Self::Existing(t) => t,
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CloseChoice {
    Manual,
    Exit,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClosePhase {
    AwaitChoice,
    ResolveDraft,
    SettleExisting,
    FinalCapture,
    Protecting,
    Reconciling,
    ProtectionSatisfied,
    EmptyCreationBaselineSatisfied,
    Failed,
    Cancelled,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CloseActiveStatus {
    pub ticket: CloseTicket,
    pub choice: Option<CloseChoice>,
    pub phase: ClosePhase,
    pub finish: Option<CloseFinishPermit>,
    pub child: Option<SaveTicket>,
    pub error: Option<SaveCoordinationError>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CloseStatus {
    pub version: u16,
    pub next_key: Option<CloseAdmissionKey>,
    pub active: Option<CloseActiveStatus>,
}
