//! Small, deterministic SyntheticTest model. No stable compatibility claim.
//!
//! This first increment accepts at most four unambiguous precompiled circles.
//! The caller supplies absolute content time and serializes input admission.
//! Project editing, rendering, parsing, automatic drivers, inspection seek and
//! native devices are outside this increment.
//! Freeze/resume is caller-stepped and rearms inputs in a fresh input epoch.
//!
//! Runtime state cannot be changed through a projection:
//! ```compile_fail
//! use skin_editor_gameplay::SyntheticRun;
//! fn overwrite(run: &mut SyntheticRun) { run.judgements[0] = None; }
//! ```
#![forbid(unsafe_code)]

mod map;
mod run;
mod types;

pub use map::{Circle, MapPreparation, PreparationProgress, SyntheticMap};
pub use run::{
    Acknowledgement, Admission, AdvanceCursor, AdvanceReport, CircleFrame, EventKey, EventKind,
    Failure, FreezeInfo, FreezeReport, GameEvent, Judgement, JudgementRecord, Limits, Observation,
    PendingReason, Phase, ReadyFrame, RearmReport, RequestKey, RunError, RunStatus, SyntheticRun,
};
pub use types::{
    EventSequence, InputEdge, InputEpoch, InputEvent, InputSequence, MapRevision, MapTime,
    ObjectId, PhysicalSlot, Point, ReleaseObservation, RunId, RunSource, TransportEpoch,
    ValueError, WorkBudget,
};

pub const GAMEPLAY_API_VERSION: u16 = 2;
pub const SYNTHETIC_PROFILE_VERSION: u32 = 1;
pub const MAX_CIRCLES: usize = 4;
pub const MAX_INPUTS: usize = 64;
pub const MAX_EVENTS: usize = 64;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Evidence {
    SyntheticTest,
}
