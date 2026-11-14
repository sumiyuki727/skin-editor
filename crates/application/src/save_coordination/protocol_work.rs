use super::*;
use crate::{CapturePoll, CaptureStatistics};

/// Opaque composition capability. A2 supplies no real-file authority issuer.
pub struct SaveProtocolPort {
    pub(super) preparation: SavePreparationPort,
}

pub(super) struct ProtocolTask {
    pub profile: u16,
    pub witness: OnceLock<PreparedWitness>,
    pub completion: OnceLock<SaveStorageObservation>,
    pub may_have_written: AtomicBool,
    pub candidate: Option<candidate::PreparedCandidate>,
    pub capacity_diagnostic: Option<SaveDiagnostic>,
    pub coverage: SaveCoverageStamp,
    pub baseline: SaveFileFacts,
}
impl ProtocolTask {
    pub fn permission(&self) -> SaveWritePermission {
        if self.candidate.is_some() {
            SaveWritePermission::MayWrite
        } else {
            SaveWritePermission::VerifyOnly
        }
    }
}

pub(super) struct QueryControl {
    pub result: OnceLock<SaveStorageObservation>,
    pub cancellation: CaptureCancellation,
}
impl QueryControl {
    pub fn new(cancellation: CaptureCancellation) -> Self {
        Self {
            result: OnceLock::new(),
            cancellation,
        }
    }
}

/// One private completion qualification, separately owned from writer scratch.
/// It is not Clone. Sending a value does not release this qualification.
pub struct SaveCompletionSender {
    pub(super) control: Arc<TaskControl>,
    pub(super) ticket: SaveTicket,
    // Persistent port flag follows the per-task handle in field drop order.
    pub(super) release: CompletionRelease,
}
impl SaveCompletionSender {
    pub fn ticket(&self) -> SaveTicket {
        self.ticket
    }
    pub fn publish(
        &self,
        observation: SaveStorageObservation,
    ) -> Result<(), SaveCoordinationError> {
        self.control
            .protocol
            .as_ref()
            .expect("qualified execution")
            .completion
            .set(observation)
            .map_err(|_| SaveCoordinationError::NotPending)
    }
}
impl Drop for SaveCompletionSender {
    fn drop(&mut self) {
        debug_assert_eq!(self.release.sequence, self.ticket.key.sequence);
    }
}
pub(super) struct CompletionRelease {
    pub port: Arc<PortState>,
    pub sequence: u64,
}
impl Drop for CompletionRelease {
    fn drop(&mut self) {
        self.port
            .completion_released_sequence
            .store(self.sequence, Ordering::Release);
    }
}

pub(super) struct ExecutionRelease {
    pub port: Arc<PortState>,
    pub sequence: u64,
}
impl Drop for ExecutionRelease {
    fn drop(&mut self) {
        self.port
            .writer_released_sequence
            .store(self.sequence, Ordering::Release);
    }
}

/// The proof and confirmation candidate already belong to the shared task.
/// Captured states are streamed through the real capture, not copied into it.
pub struct SaveExecution {
    pub(super) work: SavePreparationWork,
    pub(super) completion: Option<SaveCompletionSender>,
    // Must be last, after capture owners and an undetached completion sender.
    pub(super) release: ExecutionRelease,
}
impl SaveExecution {
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
    pub fn is_cancelled(&self) -> bool {
        self.work.control.cancellation.is_cancelled()
    }
    pub fn permission(&self) -> SaveWritePermission {
        self.work
            .control
            .protocol
            .as_ref()
            .expect("qualified execution")
            .permission()
    }
    pub fn candidate_metadata(&self) -> Option<&ArchiveMetadata> {
        self.work
            .control
            .protocol
            .as_ref()
            .expect("qualified execution")
            .candidate
            .as_ref()
            .map(|c| c.metadata.as_ref())
    }
    /// The qualified adapter must first verify every actual emitted state,
    /// original scope and request against this profile's independent proof.
    /// Complete is necessary but by itself is not sufficient file evidence.
    pub fn publish_witness(
        &self,
        info: PreparedWitnessInfo,
    ) -> Result<PreparedWitness, SaveCoordinationError> {
        let protocol = self
            .work
            .control
            .protocol
            .as_ref()
            .expect("qualified execution");
        if info.version != SAVE_COORDINATION_VERSION
            || info.capture_version != HISTORY_CAPTURE_VERSION
            || info.proof_profile != protocol.profile
        {
            return Err(SaveCoordinationError::UnsupportedVersion);
        }
        if info.ticket != self.ticket() {
            return Err(SaveCoordinationError::Conflict);
        }
        if !self.snapshot().is_complete() {
            return Err(SaveCoordinationError::NotTerminal);
        }
        let witness = PreparedWitness::prepared(info);
        match protocol.witness.set(witness) {
            Ok(()) => Ok(witness),
            Err(_) if protocol.witness.get() == Some(&witness) => Ok(witness),
            Err(_) => Err(SaveCoordinationError::Conflict),
        }
    }
    /// One permission boundary; no file operation occurs in application.
    pub fn begin_write(&self) -> Result<SaveWritePermit<'_>, SaveCoordinationError> {
        let protocol = self
            .work
            .control
            .protocol
            .as_ref()
            .expect("qualified execution");
        if protocol.completion.get().is_some() {
            return Err(SaveCoordinationError::NotPending);
        }
        if protocol.candidate.is_none() {
            return Err(SaveCoordinationError::Limit(SaveLimit::WriteCapacity));
        }
        if self.is_cancelled() {
            return Err(SaveCoordinationError::Capture(SaveDiagnostic::new(
                SaveDiagnosticCode::Cancelled,
            )));
        }
        let witness = protocol
            .witness
            .get()
            .copied()
            .ok_or(SaveCoordinationError::NotPrepared)?;
        if protocol.may_have_written.swap(true, Ordering::AcqRel) {
            return Err(SaveCoordinationError::NotPending);
        }
        Ok(SaveWritePermit {
            execution: self,
            witness,
        })
    }
    pub fn take_completion(&mut self) -> Result<SaveCompletionSender, SaveCoordinationError> {
        self.completion
            .take()
            .ok_or(SaveCoordinationError::NotPending)
    }
}
impl Drop for SaveExecution {
    fn drop(&mut self) {
        // Only the final guard publishes writer exit. A still-live detached
        // sender may legitimately confirm while a read-only query is running.
        debug_assert_eq!(self.release.sequence, self.work.ticket.key.sequence);
    }
}

/// Borrowed permission cannot outlive the writer whose actual exit gates query.
///
/// ```compile_fail
/// use skin_editor_application::{SaveExecution, SaveWritePermit};
/// fn escape(work: SaveExecution) -> SaveWritePermit<'static> {
///     work.begin_write().unwrap()
/// }
/// ```
pub struct SaveWritePermit<'work> {
    execution: &'work SaveExecution,
    witness: PreparedWitness,
}
impl SaveWritePermit<'_> {
    pub fn ticket(&self) -> SaveTicket {
        self.execution.ticket()
    }
    pub fn witness(&self) -> PreparedWitness {
        self.witness
    }
}

pub(super) struct QueryRelease {
    pub port: Arc<PortState>,
    pub ticket: ReconcileTicket,
}
impl Drop for QueryRelease {
    fn drop(&mut self) {
        self.port
            .query_released_attempt
            .store(self.ticket.attempt, Ordering::Release);
        self.port
            .query_released_save
            .store(self.ticket.save.key.sequence, Ordering::Release);
    }
}

pub struct SaveReconcileWork {
    pub(super) snapshot: CaptureSnapshot,
    pub(super) task: Arc<TaskControl>,
    pub(super) control: Arc<QueryControl>,
    pub(super) ticket: ReconcileTicket,
    // Must remain last: a query result is not the end of its ownership.
    pub(super) release: QueryRelease,
}
impl SaveReconcileWork {
    pub fn ticket(&self) -> ReconcileTicket {
        self.ticket
    }
    pub fn snapshot(&self) -> &CaptureSnapshot {
        &self.snapshot
    }
    pub fn witness(&self) -> Option<PreparedWitness> {
        self.task
            .protocol
            .as_ref()
            .expect("qualified query")
            .witness
            .get()
            .copied()
    }
    pub fn is_cancelled(&self) -> bool {
        self.control.cancellation.is_cancelled()
    }
    pub fn publish(
        &self,
        observation: SaveStorageObservation,
    ) -> Result<(), SaveCoordinationError> {
        self.control
            .result
            .set(observation)
            .map_err(|_| SaveCoordinationError::NotPending)
    }
}
impl Drop for SaveReconcileWork {
    fn drop(&mut self) {
        if self.control.result.get().is_none() {
            let code = if self.is_cancelled() {
                SaveDiagnosticCode::QueryCancelled
            } else {
                SaveDiagnosticCode::Abandoned
            };
            let _ = self
                .control
                .result
                .set(SaveStorageObservation::Unknown(SaveDiagnostic::new(code)));
        }
        debug_assert_eq!(self.release.ticket, self.ticket);
    }
}
