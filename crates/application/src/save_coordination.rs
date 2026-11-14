//! Headless preparation and qualified storage-fact coordination. No file I/O.
mod budget;
mod candidate;
mod protocol;
mod protocol_values;
mod protocol_work;
mod values;
mod work;
pub use budget::*;
pub use candidate::{SAVE_CANDIDATE_ALLOCATION_BYTES, SAVE_CANDIDATE_LOGICAL_BYTES};
#[cfg(test)]
pub(crate) use protocol::{
    synthetic_authority, test_candidate_weak, test_query_allocation, test_query_attempt,
};
pub use protocol_values::*;
pub use protocol_work::{
    SaveCompletionSender, SaveExecution, SaveProtocolPort, SaveReconcileWork, SaveWritePermit,
};
pub use values::*;
pub use work::{SavePreparationPort, SavePreparationWork};

use crate::{
    API_VERSION, ArchiveMetadata, CaptureCancellation, CaptureError, CaptureRequest,
    CaptureSnapshot, CaptureSource, CommitId, HISTORY_CAPTURE_VERSION, HistoryCapture,
    HistoryIdentitySource, ProjectSession, SaveOrigin,
};
use protocol::{ProtocolConfig, ProtocolSlot, ReconcileSlot};
use protocol_work::{ProtocolTask, QueryControl};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};
use work::{PortState, TaskControl, WorkRelease};

// Eight independently bounded lanes. Each is aligned so Coordinator has no
// unaccounted inter-lane padding. The reserved lanes cannot back ordinary work.
#[repr(C, align(16))]
struct Resident {
    config: SavePreparationConfig,
    port: Arc<PortState>,
    next_save: u64,
    last_preparation_commit: Option<CommitId>,
    protocol: Option<ProtocolConfig>,
}
struct SaveSlot {
    request: SaveRequest,
    ticket: SaveTicket,
    control: Arc<TaskControl>,
    snapshot: Option<CaptureSnapshot>,
    dispatched: bool,
    acknowledged: bool,
    protocol: Option<ProtocolSlot>,
}
#[repr(C, align(16))]
struct SaveLane {
    slot: Option<SaveSlot>,
}
#[repr(C, align(16))]
struct Mailbox {
    pending: Option<HistoryCapture>,
}
#[repr(C, align(16))]
struct Facts {
    files: SaveFileFacts,
    covered: Option<SaveCoverageStamp>,
    opening: Option<protocol_values::OpeningBaseline>,
    binding_conflict: bool,
    installed_metadata: Option<SaveCandidateCost>,
}
#[repr(C, align(16))]
struct CloseLane {
    _ticket: Option<CloseTicket>,
    _permit: Option<CloseFinishPermit>,
    _next_close: u64,
}
#[repr(C, align(16))]
struct QueryLane {
    slot: Option<ReconcileSlot>,
}
#[repr(C, align(16))]
struct CleanupLane {
    _save_terminal: Option<SavePreparationOutcome>,
    _close_terminal: Option<CloseTicket>,
    _ack: Option<SaveTicket>,
}
#[repr(C, align(16))]
struct FinalLane {
    _request: Option<SaveRequest>,
    _ticket: Option<SaveTicket>,
    _close: Option<CloseTicket>,
}
#[repr(C)]
pub(crate) struct Coordinator {
    resident: Resident,
    save: SaveLane,
    mailbox: Mailbox,
    facts: Facts,
    close: CloseLane,
    query: QueryLane,
    cleanup: CleanupLane,
    final_handoff: FinalLane,
}
impl Coordinator {
    fn released(&self, slot: &SaveSlot) -> bool {
        !slot.dispatched
            || self.resident.port.released_sequence.load(Ordering::Acquire)
                == slot.ticket.key.sequence
    }
    fn check_ticket(&self, ticket: SaveTicket) -> Result<(), SaveCoordinationError> {
        match &self.save.slot {
            Some(slot) if slot.ticket == ticket => Ok(()),
            Some(slot) if slot.ticket.key == ticket.key => Err(SaveCoordinationError::Conflict),
            _ => Err(SaveCoordinationError::Stale),
        }
    }
    fn refresh(&mut self, capture_busy: &AtomicBool) {
        if let Some(slot) = &mut self.save.slot {
            let can_release = if let Some(state) = &slot.protocol {
                let task = slot.control.protocol.as_ref().expect("qualified task");
                state.outcome.is_some()
                    && (!slot.dispatched
                        || self
                            .resident
                            .port
                            .completion_released_sequence
                            .load(Ordering::Acquire)
                            == slot.ticket.key.sequence
                        || (state.main_seen && task.completion.get().is_some()))
            } else {
                slot.control.result.get().is_some()
            };
            if can_release {
                slot.snapshot = None;
            }
        }
        let can_reap = self.save.slot.as_ref().is_some_and(|slot| {
            let terminal_and_released = if let Some(state) = &slot.protocol {
                let task = slot.control.protocol.as_ref().expect("qualified task");
                state.outcome.is_some()
                    && (task.completion.get().is_none() || state.main_seen)
                    && self.query.slot.is_none()
                    && (!slot.dispatched
                        || (self
                            .resident
                            .port
                            .writer_released_sequence
                            .load(Ordering::Acquire)
                            == slot.ticket.key.sequence
                            && self
                                .resident
                                .port
                                .completion_released_sequence
                                .load(Ordering::Acquire)
                                == slot.ticket.key.sequence))
            } else {
                slot.control.result.get().is_some()
            };
            slot.acknowledged
                && terminal_and_released
                && self.released(slot)
                && !capture_busy.load(Ordering::Acquire)
        });
        if can_reap {
            debug_assert!(self.mailbox.pending.is_none());
            self.save.slot = None;
        }
    }
}
impl Drop for Coordinator {
    fn drop(&mut self) {
        self.resident.port.alive.store(false, Ordering::Release);
        if let Some(slot) = &self.save.slot {
            slot.control.cancellation.cancel();
            slot.control.first_query_cancellation.cancel();
        }
        if let Some(query) = &self.query.slot {
            query.control.cancellation.cancel();
        }
    }
}

impl ProjectSession {
    pub fn save_source(&self) -> CaptureSource {
        CaptureSource {
            session_id: self.session_id,
            state: self.state,
            navigation_revision: self.navigation_revision,
        }
    }
    pub(crate) fn save_outstanding(&self) -> bool {
        self.save_coordination
            .as_ref()
            .is_some_and(|c| c.save.slot.is_some())
    }
    fn validate_save_binding(
        &self,
        binding: SaveTargetBinding,
    ) -> Result<(), SaveCoordinationError> {
        use SaveCoordinationError as E;
        if binding.version != SAVE_COORDINATION_VERSION {
            return Err(E::UnsupportedVersion);
        }
        if binding.project != self.project.id()
            || binding.key.generation == 0
            || !binding.expected.valid()
        {
            return Err(E::InvalidBinding);
        }
        match binding.phase {
            SaveTargetPhase::NewUnpublished => {
                // A permanent map from an unsaved capture is NOT publication.
                // Conversely restored provenance cannot be erased by origin=None
                // or by a legacy archive with no last_commit metadata.
                if self.restored_history
                    || self.archive_metadata.last_commit.is_some()
                    || binding.history.is_some()
                    || binding.expected != ExpectedFileCommit::default()
                {
                    return Err(E::InvalidBinding);
                }
            }
            SaveTargetPhase::ExistingVerified => {
                let identity = self.persistence.as_ref().ok_or(E::InvalidBinding)?;
                let last = self
                    .archive_metadata
                    .last_commit
                    .as_ref()
                    .ok_or(E::InvalidBinding)?;
                if binding.history != Some(SaveHistoryScope::from(&identity.identity))
                    || binding.expected.commit != Some(last.commit)
                    || binding.expected.generation != Some(last.generation)
                {
                    return Err(E::InvalidBinding);
                }
            }
        }
        Ok(())
    }
    /// Explicit composition opt-in. There is no default fake or file adapter.
    /// One installation only; target rebinding/publication is outside A1.
    pub fn install_save_preparation(
        &mut self,
        config: SavePreparationConfig,
    ) -> Result<SavePreparationPort, SaveCoordinationError> {
        use SaveCoordinationError as E;
        if config.version != SAVE_COORDINATION_VERSION
            || config.control.version != SAVE_CONTROL_PROFILE_VERSION
        {
            return Err(E::UnsupportedVersion);
        }
        if self.save_coordination.is_some() {
            return Err(E::AlreadyInstalled);
        }
        if config.control.available_bytes < SAVE_CONTROL_BYTES || !save_control_layout().fits() {
            return Err(E::Limit(SaveLimit::Control));
        }
        if !values::valid_fixture_limits(config.capture) {
            return Err(E::Limit(SaveLimit::Fixture));
        }
        self.validate_save_binding(config.binding)?;
        // The entire profile is reserved before any new control allocation.
        let port = Arc::new(PortState {
            alive: AtomicBool::new(true),
            released_sequence: AtomicU64::new(0),
            writer_released_sequence: AtomicU64::new(0),
            completion_released_sequence: AtomicU64::new(0),
            query_released_save: AtomicU64::new(0),
            query_released_attempt: AtomicU64::new(0),
        });
        self.save_coordination = Some(Box::new(Coordinator {
            resident: Resident {
                config,
                port: port.clone(),
                next_save: 1,
                last_preparation_commit: None,
                protocol: None,
            },
            save: SaveLane { slot: None },
            mailbox: Mailbox { pending: None },
            facts: Facts {
                files: SaveFileFacts::default(),
                covered: None,
                opening: None,
                binding_conflict: false,
                installed_metadata: None,
            },
            close: CloseLane {
                _ticket: None,
                _permit: None,
                _next_close: 1,
            },
            query: QueryLane { slot: None },
            cleanup: CleanupLane {
                _save_terminal: None,
                _close_terminal: None,
                _ack: None,
            },
            final_handoff: FinalLane {
                _request: None,
                _ticket: None,
                _close: None,
            },
        }));
        Ok(SavePreparationPort { state: port })
    }

    /// O(1) detached values, no polling, capture, full-tree copy or file query.
    pub fn save_coordination_status(
        &self,
        version: u16,
    ) -> Result<SaveCoordinationStatus, SaveCoordinationError> {
        if version != SAVE_COORDINATION_VERSION {
            return Err(SaveCoordinationError::UnsupportedVersion);
        }
        let mut result = SaveCoordinationStatus {
            version,
            source: self.save_source(),
            availability: SaveAvailability::Unavailable,
            next_key: None,
            active: None,
            reserved_control_bytes: 0,
        };
        if let Some(c) = &self.save_coordination {
            result.reserved_control_bytes = SAVE_CONTROL_BYTES;
            if c.resident.port.alive.load(Ordering::Acquire) {
                result.availability = if c.resident.protocol.is_some() {
                    SaveAvailability::Coordinated
                } else {
                    SaveAvailability::PreparationOnly
                };
                if c.resident.next_save < u64::MAX {
                    result.next_key = Some(SaveAdmissionKey {
                        version,
                        session: self.session_id,
                        sequence: c.resident.next_save,
                    });
                }
            }
            result.active = c.save.slot.as_ref().map(|slot| {
                let outcome = if slot.protocol.is_some() {
                    None
                } else {
                    slot.control.result.get().copied()
                };
                SaveActiveStatus {
                    ticket: slot.ticket,
                    phase: if slot.protocol.is_some() {
                        match c.protocol_phase(slot) {
                            SaveProtocolPhase::DispatchPending => {
                                SavePreparationPhase::DispatchPending
                            }
                            SaveProtocolPhase::Working => SavePreparationPhase::Working,
                            SaveProtocolPhase::Reconciling => SavePreparationPhase::Reconciling,
                            SaveProtocolPhase::Settled => SavePreparationPhase::Settled,
                        }
                    } else if outcome.is_some() {
                        SavePreparationPhase::Settled
                    } else if slot.dispatched {
                        SavePreparationPhase::Working
                    } else {
                        SavePreparationPhase::DispatchPending
                    },
                    outcome,
                    acknowledged: slot.acknowledged,
                    work_released: c.released(slot),
                    capture_consumers_released: !self.capture_in_flight.load(Ordering::Acquire),
                }
            });
        }
        Ok(result)
    }
    pub fn request_save(
        &mut self,
        request: SaveRequest,
        identities: &mut dyn HistoryIdentitySource,
    ) -> Result<SaveAdmission, SaveCoordinationError> {
        use SaveCoordinationError as E;
        if request.api_version != API_VERSION
            || request.capture_version != HISTORY_CAPTURE_VERSION
            || request.key.version != SAVE_COORDINATION_VERSION
        {
            return Err(E::UnsupportedVersion);
        }
        if request.key.session != self.session_id || request.source.session_id != self.session_id {
            return Err(E::WrongSession);
        }
        if request.key.sequence == 0 {
            return Err(E::InvalidIdentity);
        }
        let c = self.save_coordination.as_mut().ok_or(E::Unavailable)?;
        c.refresh(&self.capture_in_flight);
        // Repeat matching MUST precede current state, draft and port liveness.
        if let Some(slot) = &c.save.slot
            && slot.request.key == request.key
        {
            return if slot.request == request {
                Ok(SaveAdmission::Existing(slot.ticket))
            } else {
                Err(E::Conflict)
            };
        }
        if request.key.sequence < c.resident.next_save {
            return Err(E::Stale);
        }
        if c.save.slot.is_some() {
            return Err(E::Busy(SaveBusy::SaveOutstanding));
        }
        if c.facts.binding_conflict {
            return Err(E::InvalidBinding);
        }
        if !c.resident.port.alive.load(Ordering::Acquire) {
            return Err(E::Unavailable);
        }
        if request.key.sequence != c.resident.next_save {
            return Err(E::Conflict);
        }
        // Ordinary admissions leave the last value for one final protection.
        if c.resident.next_save == u64::MAX {
            return Err(E::Limit(SaveLimit::FinalIdentityReserved));
        }
        match request.origin {
            SaveOrigin::Manual => (),
            SaveOrigin::Automatic => return Err(E::UnsupportedOrigin),
            SaveOrigin::Exit => return Err(E::NeedsClose),
        }
        let config = c.resident.config;
        let protocol_config = c.resident.protocol;
        let previous_commit = c.resident.last_preparation_commit;
        if request.binding != config.binding.key {
            return Err(E::InvalidBinding);
        }
        if request.source != self.save_source() {
            return Err(E::Conflict);
        }
        self.validate_save_binding(config.binding)?;
        if self.draft.is_some() {
            return Err(E::Busy(SaveBusy::Draft));
        }
        if self.capture_in_flight.load(Ordering::Acquire) {
            return Err(E::Busy(SaveBusy::CaptureConsumers));
        }

        let mut control = Arc::new(TaskControl::new());
        // SaveSlot is still empty. The public capture3 entry applies its normal
        // checks; there is no public bypass flag, mutable reference or fake q.
        let capture = self
            .capture_history(
                CaptureRequest {
                    version: HISTORY_CAPTURE_VERSION,
                    session_id: self.session_id,
                    expected: request.source.state,
                    limits: config.capture,
                    cancellation: control.cancellation.clone(),
                },
                identities,
            )
            .map_err(|e| E::Capture(SaveDiagnostic::capture(&e)))?;
        let snapshot = capture
            .snapshot()
            .map_err(|e| E::Capture(SaveDiagnostic::capture(&e)))?;
        let manifest = snapshot.manifest();
        let candidate = identities.next_identity().map_err(|_| {
            E::Capture(SaveDiagnostic::new(SaveDiagnosticCode::IdentityUnavailable))
        })?;
        let commit = CommitId::new(candidate).ok_or(E::InvalidIdentity)?;
        if Some(commit) == previous_commit
            || config.binding.expected.commit == Some(commit)
            || candidate == manifest.identity.history.get()
            || candidate == manifest.identity.lineage.get()
            || candidate == manifest.identity.address_epoch.get()
            || manifest.nodes.iter().any(|n| n.id.get() == candidate)
            || manifest
                .archive_metadata
                .manual_marks
                .iter()
                .any(|(id, _)| *id == commit)
            || [
                &manifest.archive_metadata.last_commit,
                &manifest.archive_metadata.manual,
                &manifest.archive_metadata.protected,
            ]
            .into_iter()
            .flatten()
            .any(|r| r.commit == commit)
        {
            return Err(E::Capture(SaveDiagnostic::new(
                SaveDiagnosticCode::IdentityCollision,
            )));
        }
        let ticket = SaveTicket {
            key: request.key,
            capture_version: HISTORY_CAPTURE_VERSION,
            target: config.binding,
            history: SaveHistoryScope::from(&manifest.identity),
            capture: SaveCaptureId(request.key.sequence),
            source: manifest.source,
            coverage: SaveCoverageKey(request.key.sequence),
            head: manifest.head,
            commit,
            origin: request.origin,
            parent_close: None,
        };
        let protocol_slot = protocol_config.map(|config| {
            protocol::prepare_task(
                config,
                ticket,
                &snapshot,
                Arc::get_mut(&mut control).expect("not handed off"),
            )
        });
        let c = self.save_coordination.as_mut().expect("installed owner");
        c.resident.next_save = c
            .resident
            .next_save
            .checked_add(1)
            .expect("final value reserved");
        c.resident.last_preparation_commit = Some(commit);
        c.mailbox.pending = Some(capture);
        c.save.slot = Some(SaveSlot {
            request,
            ticket,
            control,
            snapshot: Some(snapshot),
            dispatched: false,
            acknowledged: false,
            protocol: protocol_slot,
        });
        Ok(SaveAdmission::Accepted(ticket))
    }

    pub fn take_save_preparation(
        &mut self,
        port: &SavePreparationPort,
        ticket: SaveTicket,
    ) -> Result<SavePreparationWork, SaveCoordinationError> {
        use SaveCoordinationError as E;
        self.validate_save_ticket(ticket)?;
        let c = self.save_coordination.as_mut().ok_or(E::Unavailable)?;
        c.check_ticket(ticket)?;
        if !Arc::ptr_eq(&c.resident.port, &port.state) {
            return Err(E::WrongPort);
        }
        if !c.resident.port.alive.load(Ordering::Acquire) {
            return Err(E::Unavailable);
        }
        let slot = c.save.slot.as_mut().expect("checked ticket");
        if slot.dispatched || slot.control.result.get().is_some() {
            return Err(E::NotPending);
        }
        let capture = c.mailbox.pending.take().ok_or(E::NotPending)?;
        slot.dispatched = true;
        Ok(SavePreparationWork {
            capture,
            snapshot: slot.snapshot.as_ref().expect("early snapshot").clone(),
            control: slot.control.clone(),
            ticket,
            release: WorkRelease {
                port: c.resident.port.clone(),
                sequence: ticket.key.sequence,
            },
        })
    }
    pub fn cancel_save(&mut self, ticket: SaveTicket) -> Result<(), SaveCoordinationError> {
        self.validate_save_ticket(ticket)?;
        let c = self
            .save_coordination
            .as_mut()
            .ok_or(SaveCoordinationError::Unavailable)?;
        c.check_ticket(ticket)?;
        c.drain_protocol(&mut self.archive_metadata);
        let slot = c.save.slot.as_ref().expect("checked ticket");
        slot.control.cancellation.cancel();
        if !slot.dispatched {
            c.mailbox.pending = None;
            slot.control
                .stop(SaveDiagnostic::new(SaveDiagnosticCode::Cancelled));
        }
        c.drain_protocol(&mut self.archive_metadata);
        c.refresh(&self.capture_in_flight);
        Ok(())
    }
    pub fn acknowledge_save(
        &mut self,
        ticket: SaveTicket,
    ) -> Result<SaveAcknowledgement, SaveCoordinationError> {
        use SaveCoordinationError as E;
        self.validate_save_ticket(ticket)?;
        let c = self.save_coordination.as_mut().ok_or(E::Unavailable)?;
        c.check_ticket(ticket)?;
        c.drain_protocol(&mut self.archive_metadata);
        let slot = c.save.slot.as_mut().expect("checked ticket");
        let terminal = slot.protocol.as_ref().map_or_else(
            || slot.control.result.get().is_some(),
            |s| s.outcome.is_some(),
        );
        if !terminal {
            return Err(E::NotTerminal);
        }
        slot.acknowledged = true;
        c.refresh(&self.capture_in_flight);
        Ok(if c.save.slot.is_none() {
            SaveAcknowledgement::Released
        } else {
            SaveAcknowledgement::WaitingResources
        })
    }
    fn validate_save_ticket(&self, ticket: SaveTicket) -> Result<(), SaveCoordinationError> {
        if ticket.key.version != SAVE_COORDINATION_VERSION
            || ticket.target.version != SAVE_COORDINATION_VERSION
            || ticket.capture_version != HISTORY_CAPTURE_VERSION
        {
            return Err(SaveCoordinationError::UnsupportedVersion);
        }
        if ticket.key.session != self.session_id || ticket.source.session_id != self.session_id {
            return Err(SaveCoordinationError::WrongSession);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{HistorySessionId, IdentityError, values::ProjectId};

    struct Ids(u128);
    impl HistoryIdentitySource for Ids {
        fn next_identity(&mut self) -> Result<u128, IdentityError> {
            self.0 += 1;
            Ok(self.0)
        }
    }
    fn setup() -> (ProjectSession, SavePreparationPort, SaveBindingKey) {
        let mut s = ProjectSession::new(7, 70).unwrap();
        let binding = SaveTargetBinding {
            version: 1,
            key: SaveBindingKey {
                id: SaveBindingId::new(10).unwrap(),
                generation: 1,
            },
            target: SaveTargetId::new(20).unwrap(),
            project: ProjectId::new(7).unwrap(),
            phase: SaveTargetPhase::NewUnpublished,
            history: None,
            expected: ExpectedFileCommit::default(),
        };
        let port = s
            .install_save_preparation(SavePreparationConfig {
                version: 1,
                binding,
                control: SaveControlProfile::default(),
                capture: save_fixture_capture_limits(),
            })
            .unwrap();
        (s, port, binding.key)
    }
    fn request(s: &ProjectSession, binding: SaveBindingKey, sequence: u64) -> SaveRequest {
        SaveRequest {
            api_version: API_VERSION,
            capture_version: HISTORY_CAPTURE_VERSION,
            key: SaveAdmissionKey {
                version: 1,
                session: HistorySessionId::new(70).unwrap(),
                sequence,
            },
            source: s.save_source(),
            binding,
            origin: SaveOrigin::Manual,
        }
    }
    #[test]
    fn final_save_identity_is_reserved_without_wrap_and_cleanup_needs_no_new_identity() {
        let (mut s, _port, binding) = setup();
        s.save_coordination.as_mut().unwrap().resident.next_save = u64::MAX - 1;
        let mut ids = Ids(1000);
        let r = request(&s, binding, u64::MAX - 1);
        let t = s.request_save(r, &mut ids).unwrap().ticket();
        assert!(s.save_coordination_status(1).unwrap().next_key.is_none());
        let consumed = ids.0;
        s.cancel_save(t).unwrap();
        assert_eq!(s.acknowledge_save(t), Ok(SaveAcknowledgement::Released));
        assert_eq!(
            s.request_save(r, &mut ids),
            Err(SaveCoordinationError::Stale)
        );
        assert_eq!(
            s.request_save(request(&s, binding, u64::MAX), &mut ids),
            Err(SaveCoordinationError::Limit(
                SaveLimit::FinalIdentityReserved
            ))
        );
        assert_eq!(ids.0, consumed);
        assert_eq!(
            s.save_coordination.as_ref().unwrap().resident.next_save,
            u64::MAX
        );
    }
    #[test]
    fn first_query_token_is_preallocated_and_independent_not_a_reset_cancel_token() {
        let (mut s, _port, binding) = setup();
        let mut ids = Ids(1000);
        let t = s
            .request_save(request(&s, binding, 1), &mut ids)
            .unwrap()
            .ticket();
        s.cancel_save(t).unwrap();
        let slot = s
            .save_coordination
            .as_ref()
            .unwrap()
            .save
            .slot
            .as_ref()
            .unwrap();
        assert!(slot.control.cancellation.is_cancelled());
        assert!(!slot.control.first_query_cancellation.is_cancelled());
        assert_eq!(s.acknowledge_save(t), Ok(SaveAcknowledgement::Released));
        let _t = s
            .request_save(request(&s, binding, 2), &mut ids)
            .unwrap()
            .ticket();
        let slot = s
            .save_coordination
            .as_ref()
            .unwrap()
            .save
            .slot
            .as_ref()
            .unwrap();
        assert!(!slot.control.cancellation.is_cancelled());
        assert!(!slot.control.first_query_cancellation.is_cancelled());
    }
}
