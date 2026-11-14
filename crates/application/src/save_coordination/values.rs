use crate::{
    AddressEpoch, CaptureError, CaptureLimits, CaptureSource, CommitId, DraftState, ExpectedState,
    HistorySessionId, LineageId, PersistentHistoryId, PersistentHistoryIdentity, PersistentNodeId,
    SaveOrigin, values::ProjectId,
};
use core::num::NonZeroU128;

pub const SAVE_COORDINATION_VERSION: u16 = 1;
pub const SAVE_CONTROL_PROFILE_VERSION: u16 = 1;
pub const SAVE_CONTROL_BYTES: usize = 512 * 1024;
pub const SAVE_CONTROL_LANE_BYTES: usize = 64 * 1024;

macro_rules! identity {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub struct $name(NonZeroU128);
        impl $name {
            pub const fn new(value: u128) -> Option<Self> {
                match NonZeroU128::new(value) {
                    Some(value) => Some(Self(value)),
                    None => None,
                }
            }
            pub const fn get(self) -> u128 {
                self.0.get()
            }
        }
    };
}
identity!(SaveTargetId);
identity!(SaveBindingId);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SaveHistoryScope {
    pub history: PersistentHistoryId,
    pub lineage: LineageId,
    pub address_epoch: AddressEpoch,
}
impl From<&PersistentHistoryIdentity> for SaveHistoryScope {
    fn from(value: &PersistentHistoryIdentity) -> Self {
        Self {
            history: value.history,
            lineage: value.lineage,
            address_epoch: value.address_epoch,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SaveBindingKey {
    pub id: SaveBindingId,
    pub generation: u64,
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ExpectedFileCommit {
    pub commit: Option<CommitId>,
    pub generation: Option<u64>,
}
impl ExpectedFileCommit {
    pub(super) fn valid(self) -> bool {
        matches!(
            (self.commit, self.generation),
            (None, None) | (Some(_), Some(1..))
        )
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SaveTargetPhase {
    NewUnpublished,
    ExistingVerified,
}
/// Composition-supplied opaque preparation context, never a path or a file
/// verification result. A1 checks it against the real session's provenance.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SaveTargetBinding {
    pub version: u16,
    pub key: SaveBindingKey,
    pub target: SaveTargetId,
    pub project: ProjectId,
    pub phase: SaveTargetPhase,
    pub history: Option<SaveHistoryScope>,
    pub expected: ExpectedFileCommit,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SaveControlProfile {
    pub version: u16,
    /// Available managed control capacity; lanes never borrow from each other.
    pub available_bytes: usize,
}
impl Default for SaveControlProfile {
    fn default() -> Self {
        Self {
            version: SAVE_CONTROL_PROFILE_VERSION,
            available_bytes: SAVE_CONTROL_BYTES,
        }
    }
}
/// Small, logical capture fixture. This does not change Session defaults or
/// measure the allocator, original-byte store or the process footprint.
pub const fn save_fixture_capture_limits() -> CaptureLimits {
    CaptureLimits {
        nodes: 64,
        object_ids: 256,
        originals: 8,
        original_bytes: 256 * 1024,
        retained_metadata_bytes: 1024 * 1024,
        state_bytes: 256 * 1024,
        total_state_bytes: 16 * 1024 * 1024,
    }
}
pub(super) fn valid_fixture_limits(limits: CaptureLimits) -> bool {
    let max = save_fixture_capture_limits();
    limits.valid()
        && limits.nodes <= max.nodes
        && limits.object_ids <= max.object_ids
        && limits.originals <= max.originals
        && limits.original_bytes <= max.original_bytes
        && limits.retained_metadata_bytes <= max.retained_metadata_bytes
        && limits.state_bytes <= max.state_bytes
        && limits.total_state_bytes <= max.total_state_bytes
}
#[derive(Clone, Copy, Debug)]
pub struct SavePreparationConfig {
    pub version: u16,
    pub binding: SaveTargetBinding,
    pub control: SaveControlProfile,
    pub capture: CaptureLimits,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SaveAdmissionKey {
    pub version: u16,
    pub session: HistorySessionId,
    pub sequence: u64,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SaveRequest {
    pub api_version: u16,
    pub capture_version: u16,
    pub key: SaveAdmissionKey,
    pub source: CaptureSource,
    pub binding: SaveBindingKey,
    pub origin: SaveOrigin,
}
/// Separate runtime namespaces, not digests or persistent identities.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SaveCaptureId(pub u64);
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SaveCoverageKey(pub u64);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CloseAdmissionKey {
    pub version: u16,
    pub session: HistorySessionId,
    pub sequence: u64,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClosePurpose {
    Exit,
    NewProject,
    Open,
    Reopen,
}
/// Reserved value contract only. A1 has no close admission/finish operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CloseTicket {
    pub key: CloseAdmissionKey,
    pub purpose: ClosePurpose,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CloseFinishContext {
    pub close: CloseTicket,
    pub base: ExpectedState,
    pub draft: DraftState,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CloseFinishPermit(CloseFinishContext);
impl CloseFinishPermit {
    pub fn context(self) -> CloseFinishContext {
        self.0
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SaveTicket {
    pub key: SaveAdmissionKey,
    pub capture_version: u16,
    pub target: SaveTargetBinding,
    pub history: SaveHistoryScope,
    pub capture: SaveCaptureId,
    pub source: CaptureSource,
    pub coverage: SaveCoverageKey,
    pub head: PersistentNodeId,
    pub commit: CommitId,
    pub origin: SaveOrigin,
    pub parent_close: Option<CloseAdmissionKey>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReconcileTicket {
    pub save: SaveTicket,
    pub attempt: u64,
}
/// Reserved bounded carrier. No A1 constructor, publication, verification or
/// MayWrite authority exists. The proof profile must be supplied in A2/B.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PreparedWitnessInfo {
    pub version: u16,
    pub ticket: SaveTicket,
    pub capture_version: u16,
    pub proof_profile: u16,
    pub content: [u8; 32],
    pub originals: [u8; 32],
    pub metadata: [u8; 32],
    pub request: [u8; 32],
    pub baseline: [u8; 32],
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PreparedWitness(PreparedWitnessInfo);
impl PreparedWitness {
    pub fn info(self) -> PreparedWitnessInfo {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SaveDiagnosticCode {
    Cancelled,
    Abandoned,
    InvalidWorkLimit,
    IdentityUnavailable,
    InvalidIdentity,
    IdentityCollision,
    CaptureLimit,
    MissingOriginal,
    InvalidOriginal,
    InvalidHistory,
    CaptureRejected,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SaveDiagnostic {
    pub code: SaveDiagnosticCode,
    pub numbers: [u128; 4],
}
impl SaveDiagnostic {
    pub(super) const fn new(code: SaveDiagnosticCode) -> Self {
        Self {
            code,
            numbers: [0; 4],
        }
    }
    pub(super) fn capture(error: &CaptureError) -> Self {
        use SaveDiagnosticCode as C;
        let code = match error {
            CaptureError::Cancelled => C::Cancelled,
            CaptureError::InvalidWorkLimit => C::InvalidWorkLimit,
            CaptureError::IdentityUnavailable => C::IdentityUnavailable,
            CaptureError::InvalidIdentity => C::InvalidIdentity,
            CaptureError::IdentityCollision => C::IdentityCollision,
            CaptureError::ResourceLimit(_) => C::CaptureLimit,
            CaptureError::MissingOriginal(_) => C::MissingOriginal,
            CaptureError::InvalidOriginal(_) => C::InvalidOriginal,
            CaptureError::History(_) => C::InvalidHistory,
            _ => C::CaptureRejected,
        };
        let mut result = Self::new(code);
        match error {
            CaptureError::MissingOriginal(blob) | CaptureError::InvalidOriginal(blob) => {
                let hash = blob.sha256();
                result.numbers[0] = u128::from_be_bytes(hash[..16].try_into().expect("half hash"));
                result.numbers[1] = u128::from_be_bytes(hash[16..].try_into().expect("half hash"));
                result.numbers[2] = u128::from(blob.byte_len());
            }
            CaptureError::ResourceLimit(name) => {
                result.numbers[0] = match *name {
                    "nodes" => 1,
                    "object_ids" => 2,
                    "originals" => 3,
                    "original_bytes" => 4,
                    "retained_metadata_bytes" => 5,
                    "capture_metadata" => 6,
                    "metadata" => 7,
                    "state_bytes" => 8,
                    _ => 0,
                };
            }
            _ => (),
        }
        result
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SaveBusy {
    SaveOutstanding,
    Draft,
    CaptureConsumers,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SaveLimit {
    Control,
    Fixture,
    FinalIdentityReserved,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SaveCoordinationError {
    UnsupportedVersion,
    Unavailable,
    AlreadyInstalled,
    UnsupportedOrigin,
    NeedsClose,
    WrongSession,
    InvalidIdentity,
    InvalidBinding,
    Conflict,
    Stale,
    Busy(SaveBusy),
    Limit(SaveLimit),
    Capture(SaveDiagnostic),
    WrongPort,
    NotPending,
    NotTerminal,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SaveAdmission {
    Accepted(SaveTicket),
    Existing(SaveTicket),
}
impl SaveAdmission {
    pub fn ticket(self) -> SaveTicket {
        match self {
            Self::Accepted(t) | Self::Existing(t) => t,
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SavePreparationOutcome {
    /// The real frozen traversal finished. Nothing has been written or saved.
    CompleteNotSaved,
    StoppedNotSaved(SaveDiagnostic),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SavePreparationPhase {
    DispatchPending,
    Working,
    Settled,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SaveActiveStatus {
    pub ticket: SaveTicket,
    pub phase: SavePreparationPhase,
    pub outcome: Option<SavePreparationOutcome>,
    pub acknowledged: bool,
    pub work_released: bool,
    pub capture_consumers_released: bool,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SaveAvailability {
    Unavailable,
    /// No storage completion, VerifyOnly, MayWrite or close permission.
    PreparationOnly,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SaveCoordinationStatus {
    pub version: u16,
    pub source: CaptureSource,
    pub availability: SaveAvailability,
    pub next_key: Option<SaveAdmissionKey>,
    pub active: Option<SaveActiveStatus>,
    pub reserved_control_bytes: usize,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SaveAcknowledgement {
    WaitingResources,
    Released,
}
