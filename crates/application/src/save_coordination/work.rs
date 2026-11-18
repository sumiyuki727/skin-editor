use super::*;
use crate::{CapturePoll, CaptureStatistics};

/// A composition capability, not a storage fake. It cannot publish file facts.
/// It is intentionally not Clone and never gives access to the live session.
pub struct SavePreparationPort {
    pub(super) state: Arc<PortState>,
}
impl Drop for SavePreparationPort {
    fn drop(&mut self) {
        self.state.alive.store(false, Ordering::Release);
    }
}

pub(super) struct PortState {
    pub alive: AtomicBool,
    pub released_sequence: AtomicU64,
    pub writer_released_sequence: AtomicU64,
    pub completion_released_sequence: AtomicU64,
    pub query_released_save: AtomicU64,
    pub query_released_attempt: AtomicU64,
}
pub(super) struct TaskControl {
    pub result: OnceLock<SavePreparationOutcome>,
    pub cancellation: CaptureCancellation,
    // Allocated before admission, independent of the work cancellation. No A1
    // query is dispatched and neither token can ever be reset or reused.
    pub first_query_cancellation: CaptureCancellation,
    pub protocol: Option<ProtocolTask>,
}
impl TaskControl {
    pub fn new() -> Self {
        Self {
            result: OnceLock::new(),
            cancellation: CaptureCancellation::default(),
            first_query_cancellation: CaptureCancellation::default(),
            protocol: None,
        }
    }
    pub fn stop(&self, diagnostic: SaveDiagnostic) {
        let _ = self
            .result
            .set(SavePreparationOutcome::StoppedNotSaved(diagnostic));
    }
}

/// Its Drop runs AFTER capture, snapshot and shared task fields. A completion
/// message alone must not report the worker's ownership as released.
pub(super) struct WorkRelease {
    pub port: Arc<PortState>,
    pub sequence: u64,
}
impl Drop for WorkRelease {
    fn drop(&mut self) {
        self.port
            .released_sequence
            .store(self.sequence, Ordering::Release);
    }
}

/// Detached, single-consumer real traversal. Consumers own emitted nodes and
/// snapshots under the existing capture lease; they must actually drop them.
///
/// The underlying capture cannot be handed to a file writer through this API:
/// ```compile_fail
/// use skin_editor_application::SavePreparationWork;
/// fn bypass(work: SavePreparationWork) { let _ = work.capture; }
/// ```
pub struct SavePreparationWork {
    pub(super) capture: HistoryCapture,
    pub(super) snapshot: CaptureSnapshot,
    pub(super) control: Arc<TaskControl>,
    pub(super) ticket: SaveTicket,
    // Must remain last: ordinary Rust field drop order is part of the protocol.
    pub(super) release: WorkRelease,
}

/// Version of the borrowed save traversal, independent of admission and storage.
pub const SAVE_CAPTURE_CURSOR_VERSION: u16 = 1;

/// Exclusive traversal of the owner's existing capture. Ending the borrow leaves
/// both the traversal position and the owner's release obligations intact.
///
/// ```compile_fail
/// use skin_editor_application::SaveCaptureCursor;
/// fn write(cursor: SaveCaptureCursor<'_>) { cursor.begin_write(); }
/// ```
/// ```compile_fail
/// use skin_editor_application::SaveCaptureCursor;
/// fn complete(mut cursor: SaveCaptureCursor<'_>) { cursor.take_completion(); }
/// ```
/// ```compile_fail
/// use skin_editor_application::{PreparedWitnessInfo, SaveCaptureCursor};
/// fn publish(cursor: SaveCaptureCursor<'_>, info: PreparedWitnessInfo) {
///     cursor.publish_witness(info);
/// }
/// ```
/// ```compile_fail
/// use skin_editor_application::{SaveCaptureCursor, SavePreparationWork};
/// fn escape(work: &mut SavePreparationWork) -> SaveCaptureCursor<'static> {
///     work.capture_cursor(1).unwrap()
/// }
/// ```
/// ```compile_fail
/// use skin_editor_application::SaveExecution;
/// fn overlap(work: &mut SaveExecution) {
///     let mut cursor = work.capture_cursor(1).unwrap();
///     let _ = work.take_completion();
///     let _ = cursor.poll(1);
/// }
/// ```
pub struct SaveCaptureCursor<'work> {
    work: &'work mut SavePreparationWork,
}

impl SaveCaptureCursor<'_> {
    pub fn ticket(&self) -> SaveTicket {
        self.work.ticket()
    }
    pub fn snapshot(&self) -> &CaptureSnapshot {
        self.work.snapshot()
    }
    pub fn statistics(&self) -> CaptureStatistics {
        self.work.statistics()
    }
    pub fn poll(&mut self, max_edges: usize) -> Result<CapturePoll, CaptureError> {
        self.work.poll(max_edges)
    }
}

impl SavePreparationWork {
    pub fn capture_cursor(
        &mut self,
        version: u16,
    ) -> Result<SaveCaptureCursor<'_>, SaveCoordinationError> {
        if version != SAVE_CAPTURE_CURSOR_VERSION {
            return Err(SaveCoordinationError::UnsupportedVersion);
        }
        Ok(SaveCaptureCursor { work: self })
    }
    pub fn ticket(&self) -> SaveTicket {
        self.ticket
    }
    /// Retained before the first poll; unlike HistoryCapture::snapshot this is
    /// still available after Complete. It is metadata/leases, not full states.
    pub fn snapshot(&self) -> &CaptureSnapshot {
        &self.snapshot
    }
    pub fn statistics(&self) -> CaptureStatistics {
        self.capture.statistics()
    }
    pub fn poll(&mut self, max_edges: usize) -> Result<CapturePoll, CaptureError> {
        let result = self.capture.poll(max_edges);
        match &result {
            Ok(CapturePoll::Complete) => {
                let _ = self
                    .control
                    .result
                    .set(SavePreparationOutcome::CompleteNotSaved);
            }
            Err(error) => self.control.stop(SaveDiagnostic::capture(error)),
            _ => (),
        }
        result
    }
}
impl Drop for SavePreparationWork {
    fn drop(&mut self) {
        if self.control.result.get().is_none() {
            let code = if self.control.cancellation.is_cancelled() {
                SaveDiagnosticCode::Cancelled
            } else {
                SaveDiagnosticCode::Abandoned
            };
            self.capture.cancel();
            self.control.stop(SaveDiagnostic::new(code));
        }
        // Read for the invariant without publishing an early release. The final
        // field destructor, not this body, performs the release notification.
        debug_assert_eq!(self.release.sequence, self.ticket.key.sequence);
    }
}
