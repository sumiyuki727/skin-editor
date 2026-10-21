use crate::{Command, CommandError, CommandResult, ExpectedState, HistorySessionId, RequestId};
use core::num::NonZeroU64;
use skin_editor_domain::SkinProject;

/// Session-local, never reused after completion or cancellation.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct DraftId(NonZeroU64);
impl DraftId {
    pub const fn new(value: u64) -> Option<Self> {
        match NonZeroU64::new(value) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }
    pub const fn get(self) -> u64 {
        self.0.get()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DraftState {
    pub id: DraftId,
    pub revision: u64,
    pub base: ExpectedState,
}

#[derive(Clone, Debug, PartialEq)]
pub enum DraftAction {
    Begin {
        expected: ExpectedState,
    },
    /// Replaces the complete batch relative to the committed base; not a delta
    /// appended to the previous candidate. Failed replacements keep that candidate.
    Replace {
        expected: DraftState,
        commands: Vec<Command>,
    },
    Commit {
        expected: DraftState,
    },
    Cancel {
        expected: DraftState,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct DraftRequest {
    pub api_version: u16,
    pub session_id: HistorySessionId,
    pub request_id: RequestId,
    pub action: DraftAction,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DraftInfo {
    pub api_version: u16,
    pub session_id: HistorySessionId,
    pub state: DraftState,
    pub has_changes: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DraftSnapshot {
    pub info: DraftInfo,
    pub project: SkinProject,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DraftOutcome {
    Active(DraftInfo),
    Committed(CommandResult),
    Cancelled(DraftState),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DraftResult {
    pub api_version: u16,
    pub session_id: HistorySessionId,
    pub request_id: RequestId,
    pub outcome: DraftOutcome,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DraftError {
    Request(CommandError),
    NoActiveDraft,
    Conflict {
        expected: DraftState,
        actual: DraftState,
    },
}
impl From<CommandError> for DraftError {
    fn from(error: CommandError) -> Self {
        Self::Request(error)
    }
}
