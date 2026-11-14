//! Fixed values for A2. None of these values alone installs a storage fact.
use super::*;
use crate::ArchiveReceipt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SaveCommitFact {
    pub commit: CommitId,
    pub generation: u64,
    pub head: crate::PersistentNodeId,
    pub origin: SaveOrigin,
}
impl From<&ArchiveReceipt> for SaveCommitFact {
    fn from(value: &ArchiveReceipt) -> Self {
        Self {
            commit: value.commit,
            generation: value.generation,
            head: value.head,
            origin: value.origin,
        }
    }
}
impl SaveCommitFact {
    pub(super) fn receipt(self) -> ArchiveReceipt {
        ArchiveReceipt {
            commit: self.commit,
            generation: self.generation,
            head: self.head,
            origin: self.origin,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SaveFileFacts {
    pub latest: Option<SaveCommitFact>,
    pub manual: Option<SaveCommitFact>,
    pub protected: Option<SaveCommitFact>,
}
impl From<&ArchiveMetadata> for SaveFileFacts {
    fn from(value: &ArchiveMetadata) -> Self {
        Self {
            latest: value.last_commit.as_ref().map(Into::into),
            manual: value.manual.as_ref().map(Into::into),
            protected: value.protected.as_ref().map(Into::into),
        }
    }
}

/// Association with an append-only history and its exact navigation source.
/// This is neither a digest nor independent proof of file coverage.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SaveCoverageStamp {
    pub source: CaptureSource,
    pub nodes: usize,
    pub admissions: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SaveProtectionStatus {
    Unavailable,
    NeedsProtection,
    Covered,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SaveFactsStatus {
    pub version: u16,
    pub current: SaveCoverageStamp,
    pub files: SaveFileFacts,
    pub covered: Option<SaveCoverageStamp>,
    pub protection: SaveProtectionStatus,
    pub pending_draft: bool,
    pub binding_conflict: bool,
    /// Ownership transferred to active metadata, not freed by task Ack.
    pub installed_metadata: Option<SaveCandidateCost>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SaveCandidateLimits {
    pub logical_bytes: usize,
    pub allocation_bytes: usize,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SaveCandidateCost {
    pub logical_bytes: usize,
    /// Actual retained capacities and the pinned shared-block layout, not RSS.
    pub allocation_bytes: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SaveWritePermission {
    VerifyOnly,
    MayWrite,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SaveConfirmationKind {
    Committed,
    Replayed,
    NoChange,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SaveConfirmation {
    pub kind: SaveConfirmationKind,
    pub files: SaveFileFacts,
    pub coverage: SaveCoverageStamp,
    pub confirmed_commit: Option<SaveCommitFact>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SaveProtocolOutcome {
    Confirmed(SaveConfirmation),
    NotCommitted(SaveDiagnostic),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SaveProtocolPhase {
    DispatchPending,
    Working,
    Reconciling,
    Settled,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SaveReconcileAdmission {
    Accepted(ReconcileTicket),
    Existing(ReconcileTicket),
}
impl SaveReconcileAdmission {
    pub fn ticket(self) -> ReconcileTicket {
        match self {
            Self::Accepted(ticket) | Self::Existing(ticket) => ticket,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SaveProtocolActive {
    pub ticket: SaveTicket,
    pub phase: SaveProtocolPhase,
    pub outcome: Option<SaveProtocolOutcome>,
    pub diagnostic: Option<SaveDiagnostic>,
    pub permission: SaveWritePermission,
    pub witness_retained: bool,
    pub candidate: Option<SaveCandidateCost>,
    pub acknowledged: bool,
    pub writer_released: bool,
    pub completion_released: bool,
    pub capture_consumers_released: bool,
    pub query: Option<ReconcileTicket>,
    pub query_released: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SaveProtocolStatus {
    pub version: u16,
    pub available: bool,
    pub active: Option<SaveProtocolActive>,
    pub facts: SaveFactsStatus,
}

/// No public constructor in A2: a real qualified adapter is a B prerequisite.
/// The only A2 authority issuer is the deterministic, test-only fixture.
///
/// ```compile_fail
/// use skin_editor_application::SaveProtocolAuthority;
/// let authority = SaveProtocolAuthority { profile: 1, opening: None };
/// ```
pub struct SaveProtocolAuthority {
    pub(super) profile: u16,
    pub(super) opening: Option<OpeningBaseline>,
}

#[derive(Clone, Copy)]
pub(super) struct OpeningBaseline {
    pub coverage: SaveCoverageStamp,
    pub files: SaveFileFacts,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SaveNoChangeBasis {
    /// The complete file proof includes this exact current Manual state and
    /// navigation, not merely its head or node count.
    CurrentManual,
    /// Independently verified default-open equivalence, including derived Redo.
    VerifiedOpening,
}

/// Only a qualified execution/completion capability can deliver this value.
/// A matching witness is mandatory even for a read-only NoChange observation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SaveStorageObservation {
    Committed {
        witness: PreparedWitness,
        fact: SaveCommitFact,
        binding_conflict: bool,
    },
    Replayed {
        witness: PreparedWitness,
        fact: SaveCommitFact,
        binding_conflict: bool,
    },
    NoChange {
        witness: PreparedWitness,
        files: SaveFileFacts,
        basis: SaveNoChangeBasis,
    },
    /// Complete same-target verification excludes the original commit and
    /// proves that the frozen request is not covered. Absence alone is Unknown.
    VerifiedNotCommitted {
        witness: PreparedWitness,
        diagnostic: SaveDiagnostic,
    },
    Unknown(SaveDiagnostic),
}
