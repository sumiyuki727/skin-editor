//! Managed layout bound under the pinned Rust 1.95 toolchain. Not allocator
//! metadata, fragmentation, stack-spill, RSS or capture-payload measurement.
use super::*;
use core::mem::{align_of, size_of};
use std::sync::atomic::AtomicUsize;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SaveControlLayout {
    pub version: u16,
    /// Resident, save, mailbox, facts; close, query, cleanup, final handoff.
    pub lane_bounds: [usize; 8],
    pub owner_bytes: usize,
    pub shared_control_bytes: usize,
    pub ticket_bytes: usize,
    pub witness_bytes: usize,
    pub status_bytes: usize,
    pub diagnostic_bytes: usize,
    pub work_carrier_bytes: usize,
    pub completion_carrier_bytes: usize,
}
impl SaveControlLayout {
    pub const fn fits(self) -> bool {
        let mut i = 0;
        while i < 8 {
            if self.lane_bounds[i] > SAVE_CONTROL_LANE_BYTES {
                return false;
            }
            i += 1;
        }
        self.ticket_bytes <= 1024
            && self.witness_bytes <= 4096
            && self.status_bytes <= 4096
            && self.diagnostic_bytes <= 128
            && self.work_carrier_bytes <= 16 * 1024
            && self.completion_carrier_bytes <= 16 * 1024
    }
    pub const fn total_bound(self) -> usize {
        let mut sum = 0;
        let mut i = 0;
        while i < 8 {
            sum += self.lane_bounds[i];
            i += 1;
        }
        sum
    }
}

const fn max(a: usize, b: usize) -> usize {
    if a > b { a } else { b }
}
const fn aligned(n: usize, a: usize) -> usize {
    n.div_ceil(a) * a
}
// Pinned primary source: alloc/src/sync.rs, commit
// 59807616e1fa2540724bfbac14d7976d7e4a3860. ArcInner is repr(C, align(2)),
// two atomic usize counters followed by T. This is a reviewed layout formula,
// NOT an assertion of a stable private ABI or a copy of allocation code.
const fn arc_bytes<T>() -> usize {
    let alignment = max(2, max(align_of::<AtomicUsize>(), align_of::<T>()));
    aligned(
        aligned(2 * size_of::<AtomicUsize>(), align_of::<T>()) + size_of::<T>(),
        alignment,
    )
}
#[repr(C)]
struct CompletionCarrier {
    version: u16,
    ticket: SaveTicket,
    result: SavePreparationOutcome,
}
#[repr(C)]
struct WorkCarrier {
    version: u16,
    work: SavePreparationWork,
}

/// All control allocations: owner box, persistent port, one task and two
/// independent cancellation blocks. Shared allocations are counted once;
/// explicit handles/producer-mailbox-consumer value copies are counted below.
/// Captured models, manifest/resource allocations and a future metadata
/// candidate have separate budgets. Their handles ARE included here.
pub const fn save_control_layout() -> SaveControlLayout {
    let port = arc_bytes::<PortState>();
    let task = arc_bytes::<TaskControl>();
    let cancellation = arc_bytes::<AtomicBool>();
    let work = size_of::<WorkCarrier>();
    let completion = size_of::<CompletionCarrier>();
    SaveControlLayout {
        version: SAVE_CONTROL_PROFILE_VERSION,
        lane_bounds: [
            size_of::<Resident>()
                + size_of::<Option<Box<Coordinator>>>()
                + port
                + size_of::<SavePreparationPort>()
                + 2 * size_of::<SavePreparationConfig>()
                + 2 * size_of::<SaveControlLayout>(),
            size_of::<SaveLane>()
                + task
                + cancellation
                + 3 * size_of::<SaveRequest>()
                + 3 * size_of::<SaveAdmission>(),
            // Conservative coexistence: producer, mailbox and consumer carriers,
            // although the non-Clone worker itself transfers by move.
            size_of::<Mailbox>() + 3 * work + 3 * completion,
            size_of::<Facts>()
                + 3 * size_of::<SaveCoordinationStatus>()
                + 2 * size_of::<PreparedWitness>(),
            size_of::<CloseLane>()
                + 3 * size_of::<CloseTicket>()
                + 3 * size_of::<CloseFinishPermit>(),
            size_of::<QueryLane>()
                + cancellation
                + 3 * size_of::<ReconcileTicket>()
                + 2 * size_of::<CaptureCancellation>(),
            size_of::<CleanupLane>() + 3 * completion + 3 * size_of::<SaveAcknowledgement>(),
            size_of::<FinalLane>() + 3 * size_of::<SaveRequest>() + 3 * size_of::<SaveTicket>(),
        ],
        owner_bytes: size_of::<Coordinator>(),
        shared_control_bytes: port + task + 2 * cancellation,
        ticket_bytes: max(size_of::<SaveTicket>(), size_of::<ReconcileTicket>()),
        witness_bytes: size_of::<PreparedWitness>(),
        status_bytes: size_of::<SaveCoordinationStatus>(),
        diagnostic_bytes: size_of::<SaveDiagnostic>(),
        work_carrier_bytes: work,
        completion_carrier_bytes: completion,
    }
}

const _: () = {
    assert!(save_control_layout().fits());
    assert!(save_control_layout().total_bound() <= SAVE_CONTROL_BYTES);
    assert!(
        size_of::<Coordinator>()
            == size_of::<Resident>()
                + size_of::<SaveLane>()
                + size_of::<Mailbox>()
                + size_of::<Facts>()
                + size_of::<CloseLane>()
                + size_of::<QueryLane>()
                + size_of::<CleanupLane>()
                + size_of::<FinalLane>()
    );
};
