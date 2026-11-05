//! Owned four-object semantic snapshots for the pre-API6 golden tests only.
//! Public production observations/frames never materialize these arrays.
use skin_editor_gameplay::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FrameSnapshot {
    pub source: RunSource,
    pub at: MapTime,
    pub evidence: Evidence,
    pub statistics_origin: StatisticsOrigin,
    objects: [Option<CircleFrame>; MAX_CIRCLES],
    len: usize,
}
impl FrameSnapshot {
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
pub struct Snapshot {
    pub source: RunSource,
    pub driver: InputDriver,
    pub evidence: Evidence,
    pub profile_version: u32,
    pub statistics_origin: StatisticsOrigin,
    pub status: RunStatus,
    pub completed_through: Option<MapTime>,
    pub last_committed_time: Option<MapTime>,
    pub active_target: Option<MapTime>,
    pub committed_judgements: [Option<JudgementRecord>; MAX_CIRCLES],
    pub ready_frame: Option<FrameSnapshot>,
    pub pending_inputs: usize,
    pub pending_events: usize,
    pub held_slots: [bool; 4],
    pub armed_slots: [bool; 4],
    pub sealed_through: Option<MapTime>,
    pub freeze: Option<FreezeInfo>,
    pub cancelled_sound_through: Option<EventSequence>,
    pub inspection: Option<InspectionInfo>,
}
pub trait SnapshotExt {
    fn snapshot(&self) -> Snapshot;
}
impl SnapshotExt for SyntheticRun {
    fn snapshot(&self) -> Snapshot {
        let s = self.observation();
        let ready_frame = self.ready_frame(s.source).unwrap().map(|frame| {
            let h = frame.header();
            FrameSnapshot {
                source: h.source,
                at: h.at,
                evidence: h.evidence,
                statistics_origin: h.statistics_origin,
                len: h.len(),
                objects: core::array::from_fn(|i| frame.object(i)),
            }
        });
        Snapshot {
            source: s.source,
            driver: s.driver,
            evidence: s.evidence,
            profile_version: s.profile_version,
            statistics_origin: s.statistics_origin,
            status: s.status,
            completed_through: s.completed_through,
            last_committed_time: s.last_committed_time,
            active_target: s.active_target,
            committed_judgements: core::array::from_fn(|i| {
                self.committed_judgement(s.source, i).unwrap()
            }),
            ready_frame,
            pending_inputs: s.pending_inputs,
            pending_events: s.pending_events,
            held_slots: s.held_slots,
            armed_slots: s.armed_slots,
            sealed_through: s.sealed_through,
            freeze: s.freeze,
            cancelled_sound_through: s.cancelled_sound_through,
            inspection: s.inspection,
        }
    }
}
