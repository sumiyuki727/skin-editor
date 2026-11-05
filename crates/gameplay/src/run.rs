use crate::{
    EventSequence, Evidence, InputEdge, InputEvent, InputSequence, MAX_CIRCLES, MAX_EVENTS,
    MAX_INPUTS, MapTime, ObjectId, ReleaseObservation, RunId, RunSource, SYNTHETIC_PROFILE_VERSION,
    SyntheticMap, TransportEpoch, ValueError, WorkBudget,
};

mod admission;
mod control;
mod engine;
mod inspection;
pub use control::{FreezeInfo, FreezeReport, RearmReport};
use inspection::InspectionCandidate;
pub use inspection::{InspectionCursor, InspectionInfo, InspectionKey, InspectionReport};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RunError {
    StaleSource,
    GateClosed,
    LateInput,
    InputBeforeStart,
    InputConflict,
    OldInputSequence,
    InputCapacity,
    InvalidTarget,
    BeyondSeal,
    Busy,
    StaleCursor,
    UnknownEvent,
    CounterExhausted,
    Faulted,
    BudgetLimit,
    RearmRequired,
    NotFrozen,
    NotFreezing,
    StaleInspection,
    InspectionIncomplete,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Failure {
    pub cause: RunError,
    pub cancelled_inputs: usize,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RunStatus {
    Prepared,
    Ready,
    Pending,
    Freezing,
    Frozen,
    Failed(Failure),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Judgement {
    Hit,
    Miss,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct JudgementRecord {
    pub object: ObjectId,
    pub at: MapTime,
    pub result: Judgement,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Phase {
    Future,
    VisibleUnjudged,
    ResolvedVisual,
    Retired,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CircleFrame {
    pub object: ObjectId,
    pub phase: Phase,
    pub judgement: Option<JudgementRecord>,
    pub unavailable: Option<UnavailableReason>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnavailableReason {
    InspectionSkipped,
}
/// Provenance of the ledger, not a calculated score/ACC or a HUD policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StatisticsOrigin {
    FromStart { start: MapTime },
    Inspection { at: MapTime },
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReadyFrame {
    pub source: RunSource,
    pub at: MapTime,
    pub evidence: Evidence,
    pub statistics_origin: StatisticsOrigin,
    objects: [Option<CircleFrame>; MAX_CIRCLES],
    len: usize,
}
impl ReadyFrame {
    pub fn object(self, index: usize) -> Option<CircleFrame> {
        self.objects.get(index).copied().flatten()
    }
    pub const fn len(self) -> usize {
        self.len
    }
    pub const fn is_empty(self) -> bool {
        self.len == 0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Observation {
    pub source: RunSource,
    pub evidence: Evidence,
    pub profile_version: u32,
    pub statistics_origin: StatisticsOrigin,
    pub status: RunStatus,
    pub completed_through: Option<MapTime>,
    pub last_committed_time: Option<MapTime>,
    pub active_target: Option<MapTime>,
    pub committed_judgements: [Option<JudgementRecord>; MAX_CIRCLES],
    pub ready_frame: Option<ReadyFrame>,
    pub pending_inputs: usize,
    pub pending_events: usize,
    pub held_slots: [bool; 4],
    pub armed_slots: [bool; 4],
    pub sealed_through: Option<MapTime>,
    pub freeze: Option<FreezeInfo>,
    pub cancelled_sound_through: Option<EventSequence>,
    pub inspection: Option<InspectionInfo>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EventKey {
    run: RunId,
    transport: TransportEpoch,
    sequence: EventSequence,
}
impl EventKey {
    pub const fn new(run: RunId, transport: TransportEpoch, sequence: EventSequence) -> Self {
        Self {
            run,
            transport,
            sequence,
        }
    }
    pub const fn run(self) -> RunId {
        self.run
    }
    pub const fn transport(self) -> TransportEpoch {
        self.transport
    }
    pub const fn sequence(self) -> EventSequence {
        self.sequence
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EventKind {
    Judgement(Judgement),
    SoundIntent,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GameEvent {
    pub key: EventKey,
    pub object: ObjectId,
    pub at: MapTime,
    pub kind: EventKind,
    pub evidence: Evidence,
    pub profile_version: u32,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Acknowledgement {
    Acknowledged {
        retired_through: Option<EventSequence>,
        removed: usize,
    },
    AlreadyRetired,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Admission {
    pub added: usize,
    pub repeated: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RequestKey {
    source: RunSource,
    id: u64,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AdvanceCursor {
    request: RequestKey,
    step: u64,
}
impl AdvanceCursor {
    pub const fn request(self) -> RequestKey {
        self.request
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PendingReason {
    WorkBudget,
    EventBackpressure,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdvanceReport {
    Complete {
        through: MapTime,
        steps: u16,
    },
    Pending {
        cursor: AdvanceCursor,
        reason: PendingReason,
        completed_through: Option<MapTime>,
        steps: u16,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Limits {
    inputs: usize,
    events: usize,
    replay: usize,
    max_work: u16,
    storage_bytes: usize,
}
impl Limits {
    pub const fn tiny() -> Self {
        Self {
            inputs: 4,
            events: 2,
            replay: 4,
            max_work: 1,
            storage_bytes: 64 * 1024,
        }
    }
    pub const fn reference() -> Self {
        Self {
            inputs: 64,
            events: 64,
            replay: 64,
            max_work: 256,
            storage_bytes: 8 * 1024 * 1024,
        }
    }
    pub fn try_new(
        inputs: usize,
        events: usize,
        replay: usize,
        max_work: u16,
        storage_bytes: usize,
    ) -> Result<Self, ValueError> {
        let limits = Self {
            inputs,
            events,
            replay,
            max_work,
            storage_bytes,
        };
        limits.validate()?;
        Ok(limits)
    }
    fn validate(self) -> Result<(), ValueError> {
        if self.inputs == 0
            || self.inputs > MAX_INPUTS
            || self.events < 2
            || self.events > MAX_EVENTS
            || self.replay == 0
            || self.replay > MAX_INPUTS
            || self.max_work == 0
            || self.max_work > 256
        {
            return Err(ValueError::InvalidLimits);
        }
        if self.storage_bytes < SyntheticRun::static_storage_bytes() {
            return Err(ValueError::StorageLimit);
        }
        Ok(())
    }
    pub fn budget(self) -> WorkBudget {
        WorkBudget::new(self.max_work).expect("validated work limit")
    }
}

#[derive(Clone, Copy, Debug)]
struct Active {
    key: RequestKey,
    target: MapTime,
}
#[derive(Clone, Copy, Debug)]
struct InputDraft {
    input: InputEvent,
    held: [bool; 4],
    next: usize,
    candidate: Option<usize>,
}
#[derive(Clone, Copy, Debug)]
enum Work {
    Select,
    Search(InputDraft),
    Commit(InputDraft),
    Publish {
        at: MapTime,
        next: usize,
        buffer: usize,
    },
}
#[derive(Clone, Copy, Debug)]
struct EventReservation {
    first: u64,
    last: u64,
    after: Option<u64>,
    count: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ControlMode {
    Running,
    Freezing,
    Frozen,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum InputRecord {
    Gameplay(InputEvent),
    Release(ReleaseObservation),
}
impl InputRecord {
    const fn sequence(self) -> InputSequence {
        match self {
            Self::Gameplay(event) => event.sequence,
            Self::Release(fact) => fact.sequence,
        }
    }
}
enum PumpReport {
    Complete { steps: u16 },
    Pending { reason: PendingReason, steps: u16 },
}

/// Owned preview state only; no project/session writer and no heap allocation.
pub struct SyntheticRun {
    source: RunSource,
    map: SyntheticMap,
    start: MapTime,
    statistics_origin: StatisticsOrigin,
    inspection_skipped: [bool; MAX_CIRCLES],
    inspection: Option<InspectionCandidate>,
    next_inspection: Option<u64>,
    limits: Limits,
    sealed: Option<MapTime>,
    highest_input: u64,
    inputs: [Option<InputEvent>; MAX_INPUTS],
    input_len: usize,
    replay: [Option<InputRecord>; MAX_INPUTS],
    replay_next: usize,
    held: [bool; 4],
    armed: [bool; 4],
    // A timed release cannot authorize a gameplay edge earlier than itself.
    // Initial/untimed arming has no extra floor beyond the ordinary seal.
    armed_from: [Option<MapTime>; 4],
    mode: ControlMode,
    freeze_info: Option<FreezeInfo>,
    cancelled_sound_through: u64,
    judgements: [Option<JudgementRecord>; MAX_CIRCLES],
    appeared: usize,
    expired: usize,
    retired: usize,
    events: [Option<GameEvent>; MAX_EVENTS],
    acknowledged: [bool; MAX_EVENTS],
    event_head: usize,
    event_len: usize,
    next_event: Option<u64>,
    issued_through: u64,
    retired_through: u64,
    frames: [ReadyFrame; 2],
    ready: Option<usize>,
    dirty: Option<MapTime>,
    last_committed: Option<MapTime>,
    active: Option<Active>,
    next_request: Option<u64>,
    step: u64,
    work: Work,
    failure: Option<Failure>,
}
impl SyntheticRun {
    /// Fixed owned storage plus bounded admission/candidate move scratch.
    /// This is not a process memory or native stack measurement.
    pub const fn static_storage_bytes() -> usize {
        core::mem::size_of::<Self>()
            + core::mem::size_of::<[Option<InputEvent>; MAX_INPUTS]>()
            + core::mem::size_of::<([bool; 4], [Option<MapTime>; 4])>()
            + core::mem::size_of::<InspectionCandidate>()
    }
    pub fn new(
        source: RunSource,
        map: SyntheticMap,
        start: MapTime,
        limits: Limits,
    ) -> Result<Self, ValueError> {
        limits.validate()?;
        if source.revision != map.revision {
            return Err(ValueError::RevisionMismatch);
        }
        if map.circle(0).is_some_and(|circle| start > circle.visible) {
            return Err(ValueError::InvalidStart);
        }
        let frame = ReadyFrame {
            source,
            at: start,
            evidence: Evidence::SyntheticTest,
            statistics_origin: StatisticsOrigin::FromStart { start },
            objects: [None; MAX_CIRCLES],
            len: map.len,
        };
        Ok(Self {
            source,
            map,
            start,
            statistics_origin: StatisticsOrigin::FromStart { start },
            inspection_skipped: [false; MAX_CIRCLES],
            inspection: None,
            next_inspection: Some(1),
            limits,
            sealed: None,
            highest_input: 0,
            inputs: [None; MAX_INPUTS],
            input_len: 0,
            replay: [None; MAX_INPUTS],
            replay_next: 0,
            held: [false; 4],
            armed: [true; 4],
            armed_from: [None; 4],
            mode: ControlMode::Running,
            freeze_info: None,
            cancelled_sound_through: 0,
            judgements: [None; MAX_CIRCLES],
            appeared: 0,
            expired: 0,
            retired: 0,
            events: [None; MAX_EVENTS],
            acknowledged: [false; MAX_EVENTS],
            event_head: 0,
            event_len: 0,
            next_event: Some(1),
            issued_through: 0,
            retired_through: 0,
            frames: [frame; 2],
            ready: None,
            dirty: None,
            last_committed: None,
            active: None,
            next_request: Some(1),
            step: 0,
            work: Work::Select,
            failure: None,
        })
    }
    pub const fn source(&self) -> RunSource {
        self.source
    }
    fn completed(&self) -> Option<MapTime> {
        self.ready.map(|index| self.frames[index].at)
    }
    pub fn observation(&self) -> Observation {
        let status = if let Some(failure) = self.failure {
            RunStatus::Failed(failure)
        } else if self.mode == ControlMode::Freezing {
            RunStatus::Freezing
        } else if self.mode == ControlMode::Frozen {
            RunStatus::Frozen
        } else if self.active.is_some() || self.dirty.is_some() {
            RunStatus::Pending
        } else if self.ready.is_some() {
            RunStatus::Ready
        } else {
            RunStatus::Prepared
        };
        Observation {
            source: self.source,
            evidence: Evidence::SyntheticTest,
            profile_version: SYNTHETIC_PROFILE_VERSION,
            statistics_origin: self.statistics_origin,
            status,
            completed_through: self.completed(),
            last_committed_time: self.last_committed,
            active_target: self.active.map(|v| v.target),
            committed_judgements: self.judgements,
            ready_frame: self.ready.map(|i| self.frames[i]),
            pending_inputs: self.input_len,
            pending_events: self.event_len,
            held_slots: self.held,
            armed_slots: self.armed,
            sealed_through: self.sealed,
            freeze: self.freeze_info,
            cancelled_sound_through: EventSequence::new(self.cancelled_sound_through).ok(),
            inspection: self.inspection.as_ref().map(InspectionCandidate::info),
        }
    }
    pub fn event_at(&self, index: usize) -> Option<GameEvent> {
        if index >= self.event_len {
            return None;
        }
        self.events[(self.event_head + index) % self.limits.events]
    }
    pub fn acknowledge(&mut self, key: EventKey) -> Result<Acknowledgement, RunError> {
        if key.run != self.source.run || key.transport != self.source.transport {
            return Err(RunError::StaleSource);
        }
        let sequence = key.sequence.get();
        if sequence <= self.retired_through {
            return Ok(Acknowledgement::AlreadyRetired);
        }
        if sequence > self.issued_through {
            return Err(RunError::UnknownEvent);
        }
        let Some(offset) = (0..self.event_len)
            .find(|offset| self.event_at(*offset).is_some_and(|event| event.key == key))
        else {
            return Err(RunError::UnknownEvent);
        };
        self.acknowledged[(self.event_head + offset) % self.limits.events] = true;
        let mut removed = 0;
        while self.event_len > 0 && self.acknowledged[self.event_head] {
            let event = self.events[self.event_head]
                .take()
                .expect("occupied event slot");
            self.retired_through = event.key.sequence.get();
            self.acknowledged[self.event_head] = false;
            self.event_head = (self.event_head + 1) % self.limits.events;
            self.event_len -= 1;
            removed += 1;
        }
        Ok(Acknowledgement::Acknowledged {
            retired_through: EventSequence::new(self.retired_through).ok(),
            removed,
        })
    }
    fn check_source(&self, source: RunSource) -> Result<(), RunError> {
        if source == self.source {
            Ok(())
        } else {
            Err(RunError::StaleSource)
        }
    }
    fn fail(&mut self, cause: RunError) -> RunError {
        if self.failure.is_none() {
            self.failure = Some(Failure {
                cause,
                cancelled_inputs: self.input_len,
            });
            self.inputs.fill(None);
            self.input_len = 0;
            self.active = None;
            self.inspection = None;
            self.work = Work::Select;
            self.armed.fill(false);
            self.armed_from.fill(None);
        }
        cause
    }
    fn has_freeze_cleanup(&self) -> bool {
        self.mode == ControlMode::Freezing
            && self
                .input_len
                .checked_sub(1)
                .and_then(|index| self.inputs[index])
                .is_some_and(|input| {
                    input.at > self.freeze_info.expect("freezing boundary").boundary
                })
    }
    fn reserve_events(&self, count: usize) -> Result<Option<EventReservation>, RunError> {
        if self.event_len + count > self.limits.events {
            return Ok(None);
        }
        let first = self.next_event.ok_or(RunError::CounterExhausted)?;
        let last = first
            .checked_add(u64::try_from(count - 1).expect("small nonzero event count"))
            .ok_or(RunError::CounterExhausted)?;
        Ok(Some(EventReservation {
            first,
            last,
            after: last.checked_add(1),
            count,
        }))
    }
    fn emit(&mut self, reservation: EventReservation, record: JudgementRecord) {
        for offset in 0..reservation.count {
            let sequence = reservation.first + u64::try_from(offset).expect("small event offset");
            let index = (self.event_head + self.event_len) % self.limits.events;
            self.events[index] = Some(GameEvent {
                key: EventKey::new(
                    self.source.run,
                    self.source.transport,
                    EventSequence::new(sequence).expect("reserved sequence"),
                ),
                object: record.object,
                at: record.at,
                kind: if offset == 0 {
                    EventKind::Judgement(record.result)
                } else {
                    EventKind::SoundIntent
                },
                evidence: Evidence::SyntheticTest,
                profile_version: SYNTHETIC_PROFILE_VERSION,
            });
            self.acknowledged[index] = false;
            self.event_len += 1;
        }
        self.next_event = reservation.after;
        self.issued_through = reservation.last;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Circle, InputEpoch, InputSequence, MapRevision, PhysicalSlot, Point};

    #[test]
    fn exhausted_event_identifiers_cannot_partially_commit_a_two_event_hit() {
        let source = RunSource {
            run: RunId::new(1).unwrap(),
            transport: TransportEpoch::new(1).unwrap(),
            input: InputEpoch::new(1).unwrap(),
            revision: MapRevision::new(1),
        };
        let at = MapTime::from_micros(0);
        let center = Point::new(0, 0).unwrap();
        let map = SyntheticMap::try_new(
            source.revision,
            &[Circle::new(ObjectId::new(0), at, center).unwrap()],
        )
        .unwrap();
        let mut run = SyntheticRun::new(
            source,
            map,
            MapTime::from_micros(-200_001),
            Limits::reference(),
        )
        .unwrap();
        run.submit_inputs(
            source,
            &[InputEvent {
                sequence: InputSequence::new(1).unwrap(),
                at,
                slot: PhysicalSlot::PrimaryOne,
                edge: InputEdge::Press(center),
            }],
        )
        .unwrap();
        // Inject the last identifier: a two-event transition must reject before
        // consuming the edge, publishing either event or changing the ledger.
        run.next_event = Some(u64::MAX);
        run.seal_through(source, at).unwrap();
        assert_eq!(
            run.advance(source, at, WorkBudget::full()),
            Err(RunError::CounterExhausted)
        );
        let result = run.observation();
        assert_eq!(
            result.status,
            RunStatus::Failed(Failure {
                cause: RunError::CounterExhausted,
                cancelled_inputs: 1
            })
        );
        assert_eq!(result.committed_judgements, [None; MAX_CIRCLES]);
        assert_eq!(result.held_slots, [false; 4]);
        assert_eq!(result.pending_events, 0);
        assert_eq!(run.event_at(0), None);
    }
}
