//! Pure close intent coordination. No filesystem, shutdown or activation path.
use super::*;
use crate::{CommandError, DraftAction, DraftError, DraftRequest, DraftResult};

pub(super) struct CloseSlot {
    pub request: CloseRequest,
    pub status: CloseActiveStatus,
}
impl Coordinator {
    pub(super) fn close_active(&self) -> bool {
        self.close
            .slot
            .as_ref()
            .is_some_and(|s| s.status.phase != ClosePhase::Cancelled)
    }
    fn check_close(&self, ticket: CloseTicket) -> Result<(), SaveCoordinationError> {
        match &self.close.slot {
            Some(s) if s.status.ticket == ticket => Ok(()),
            Some(s) if s.status.ticket.key == ticket.key => Err(SaveCoordinationError::Conflict),
            _ => Err(SaveCoordinationError::Stale),
        }
    }
    pub(super) fn final_capture_matches(&self, ticket: CloseTicket, origin: SaveOrigin) -> bool {
        self.close.slot.as_ref().is_some_and(|s| {
            s.status.ticket == ticket
                && s.status.phase == ClosePhase::FinalCapture
                && s.status.finish.is_none()
                && s.status.child.is_none()
                && matches!(
                    (s.status.choice, origin),
                    (Some(CloseChoice::Manual), SaveOrigin::Manual)
                        | (Some(CloseChoice::Exit), SaveOrigin::Exit)
                )
        })
    }
    // Copy the bounded outcome BEFORE any SaveSlot reap. Exact parent + child
    // association prevents old completions from becoming a later close result.
    pub(super) fn sync_close_child(&mut self) {
        let Some(close) = &mut self.close.slot else {
            return;
        };
        if close.status.phase == ClosePhase::Cancelled {
            return;
        }
        let Some(save) = &self.save.slot else { return };
        if close.status.child != Some(save.ticket)
            || save.ticket.parent_close != Some(close.status.ticket.key)
        {
            return;
        }
        if self.facts.binding_conflict {
            close.status.phase = ClosePhase::Failed;
            close.status.error = Some(SaveCoordinationError::InvalidBinding);
            return;
        }
        if let Some(protocol) = &save.protocol {
            match protocol.outcome {
                Some(SaveProtocolOutcome::Confirmed(_)) => {
                    close.status.phase = ClosePhase::ProtectionSatisfied;
                    close.status.error = None;
                }
                Some(SaveProtocolOutcome::NotCommitted(d)) => {
                    close.status.phase = ClosePhase::Failed;
                    close.status.error = Some(SaveCoordinationError::Storage(d));
                }
                None => {
                    close.status.error = protocol.diagnostic.map(SaveCoordinationError::Storage);
                    close.status.phase = if protocol.unknown {
                        ClosePhase::Reconciling
                    } else {
                        ClosePhase::Protecting
                    }
                }
            }
        }
    }
}
impl ProjectSession {
    pub(crate) fn protection_identity_exhausted(&self) -> bool {
        self.save_coordination
            .as_ref()
            .is_some_and(|c| c.resident.save_exhausted || c.close.next_close.is_none())
    }
    pub(crate) fn close_active(&self) -> bool {
        self.save_coordination
            .as_ref()
            .is_some_and(|c| c.close_active())
    }
    pub(crate) fn close_finish_permit(&self) -> Option<CloseFinishPermit> {
        self.save_coordination
            .as_ref()?
            .close
            .slot
            .as_ref()?
            .status
            .finish
    }
    pub(crate) fn permits_final_capture(&self, permit: &FinalCapturePermit) -> bool {
        self.save_coordination.as_ref().is_some_and(|c| {
            c.final_capture_matches(permit.0, SaveOrigin::Manual)
                || c.final_capture_matches(permit.0, SaveOrigin::Exit)
        })
    }
    fn validate_close_ticket(&self, ticket: CloseTicket) -> Result<(), SaveCoordinationError> {
        if ticket.key.version != SAVE_COORDINATION_VERSION {
            return Err(SaveCoordinationError::UnsupportedVersion);
        }
        if ticket.key.session != self.session_id {
            return Err(SaveCoordinationError::WrongSession);
        }
        self.save_coordination
            .as_ref()
            .ok_or(SaveCoordinationError::Unavailable)?
            .check_close(ticket)
    }
    /// Detached fixed values. Never polls a worker or captures the history.
    pub fn close_status(&self, version: u16) -> Result<CloseStatus, SaveCoordinationError> {
        if version != SAVE_COORDINATION_VERSION {
            return Err(SaveCoordinationError::UnsupportedVersion);
        }
        let c = self
            .save_coordination
            .as_ref()
            .ok_or(SaveCoordinationError::Unavailable)?;
        Ok(CloseStatus {
            version,
            next_key: c.close.next_close.map(|sequence| CloseAdmissionKey {
                version,
                session: self.session_id,
                sequence,
            }),
            active: c.close.slot.as_ref().map(|s| s.status),
        })
    }
    pub fn request_close(
        &mut self,
        request: CloseRequest,
    ) -> Result<CloseAdmission, SaveCoordinationError> {
        use SaveCoordinationError as E;
        if request.api_version != API_VERSION || request.key.version != SAVE_COORDINATION_VERSION {
            return Err(E::UnsupportedVersion);
        }
        if request.key.session != self.session_id || request.source.session_id != self.session_id {
            return Err(E::WrongSession);
        }
        if request.key.sequence == 0 {
            return Err(E::InvalidIdentity);
        }
        let source = self.save_source();
        let c = self.save_coordination.as_mut().ok_or(E::Unavailable)?;
        if let Some(slot) = &c.close.slot
            && slot.request.key == request.key
        {
            return if slot.request == request {
                Ok(CloseAdmission::Existing(slot.status.ticket))
            } else {
                Err(E::Conflict)
            };
        }
        let next = c.close.next_close.ok_or(E::Stale)?;
        if request.key.sequence < next {
            return Err(E::Stale);
        }
        if c.close.slot.is_some() {
            return Err(E::Busy(SaveBusy::CloseOutstanding));
        }
        if request.key.sequence != next || request.source != source {
            return Err(E::Conflict);
        }
        c.drain_protocol(&mut self.archive_metadata);
        c.refresh(&self.capture_in_flight);
        if c.save
            .slot
            .as_ref()
            .is_some_and(|s| s.ticket.parent_close.is_some())
        {
            return Err(E::Busy(SaveBusy::SaveOutstanding));
        }
        if c.facts.binding_conflict {
            return Err(E::InvalidBinding);
        }
        let ticket = CloseTicket {
            key: request.key,
            purpose: request.purpose,
        };
        let finish = self.draft.as_ref().map(|d| {
            CloseFinishPermit::issue(CloseFinishContext {
                close: ticket,
                base: self.state,
                draft: d.state,
            })
        });
        c.close.slot = Some(CloseSlot {
            request,
            status: CloseActiveStatus {
                ticket,
                choice: None,
                phase: ClosePhase::AwaitChoice,
                finish,
                child: None,
                error: None,
            },
        });
        c.close.next_close = next.checked_add(1);
        Ok(CloseAdmission::Accepted(ticket))
    }
    pub fn choose_close(
        &mut self,
        ticket: CloseTicket,
        choice: CloseChoice,
    ) -> Result<(), SaveCoordinationError> {
        self.validate_close_ticket(ticket)?;
        let s = &mut self
            .save_coordination
            .as_mut()
            .expect("installed")
            .close
            .slot
            .as_mut()
            .expect("checked")
            .status;
        if s.phase == ClosePhase::Cancelled {
            return Err(SaveCoordinationError::NotPending);
        }
        if let Some(old) = s.choice {
            return if old == choice {
                Ok(())
            } else {
                Err(SaveCoordinationError::Conflict)
            };
        }
        s.choice = Some(choice);
        s.phase = if s.finish.is_some() {
            ClosePhase::ResolveDraft
        } else {
            ClosePhase::SettleExisting
        };
        Ok(())
    }
    /// The sole close-owned path to the ordinary atomic draft publisher. The
    /// current draft is checked before cached receipts; no gate is removed.
    pub fn finish_close_draft(
        &mut self,
        permit: CloseFinishPermit,
        request: DraftRequest,
    ) -> Result<DraftResult, DraftError> {
        if request.session_id != self.session_id {
            return Err(CommandError::WrongSession.into());
        }
        if request.api_version != API_VERSION {
            return Err(CommandError::UnsupportedVersion(request.api_version).into());
        }
        let context = permit.context();
        if !self.close_active()
            || self.validate_close_ticket(context.close).is_err()
            || self.close_finish_permit() != Some(permit)
            || context.base != self.state
            || self.draft.as_ref().map(|d| d.state) != Some(context.draft)
            || !matches!(request.action, DraftAction::Commit { expected } | DraftAction::Cancel { expected } if expected == context.draft)
        {
            return Err(CommandError::BlockedByClose.into());
        }
        let result = self.edit_draft_checked(request)?;
        // A failed attempt returns above, preserving the permit/cancel reserve.
        debug_assert!(self.draft.is_none());
        let s = &mut self
            .save_coordination
            .as_mut()
            .expect("installed")
            .close
            .slot
            .as_mut()
            .expect("checked")
            .status;
        s.finish = None;
        s.phase = if s.choice.is_some() {
            ClosePhase::SettleExisting
        } else {
            ClosePhase::AwaitChoice
        };
        Ok(result)
    }
    fn close_fail(&mut self, error: SaveCoordinationError) -> Result<(), SaveCoordinationError> {
        let s = &mut self
            .save_coordination
            .as_mut()
            .expect("installed")
            .close
            .slot
            .as_mut()
            .expect("active")
            .status;
        s.phase = ClosePhase::Failed;
        s.error = Some(error);
        Err(error)
    }
    /// Advance only existing intent; never retries a failed child implicitly.
    pub fn advance_close(
        &mut self,
        ticket: CloseTicket,
        identities: &mut dyn HistoryIdentitySource,
    ) -> Result<(), SaveCoordinationError> {
        use SaveCoordinationError as E;
        self.validate_close_ticket(ticket)?;
        self.advance_save_coordination(SAVE_COORDINATION_VERSION)?;
        let c = self.save_coordination.as_ref().expect("installed");
        let s = c.close.slot.as_ref().expect("checked").status;
        if s.phase == ClosePhase::Cancelled {
            return Ok(());
        }
        if c.facts.binding_conflict {
            return self.close_fail(E::InvalidBinding);
        }
        if matches!(
            s.phase,
            ClosePhase::AwaitChoice
                | ClosePhase::ResolveDraft
                | ClosePhase::ProtectionSatisfied
                | ClosePhase::EmptyCreationBaselineSatisfied
                | ClosePhase::Failed
        ) {
            return Ok(());
        }
        if s.child.is_some() {
            return Ok(());
        }
        // First record the independent task's actual fact, then use the exact
        // existing Ack path. Unknown and live consumers keep the slot occupied.
        if let Some(old) = c.save.slot.as_ref().map(|s| s.ticket) {
            match self.acknowledge_save(old) {
                Ok(SaveAcknowledgement::Released) => (),
                Ok(SaveAcknowledgement::WaitingResources) | Err(E::NotTerminal) => return Ok(()),
                Err(e) => return Err(e),
            }
        }
        if self.capture_in_flight.load(Ordering::Acquire) {
            return Ok(());
        }
        let choice = s.choice.expect("fixed choice");
        // Identity reservation by a previous unsaved capture is not an edit.
        let empty = !self.restored_history
            && self.history.nodes.len() == 1
            && self.used_ids.is_empty()
            && self.state.revision == 0
            && self.draft.is_none()
            && self.archive_metadata.as_ref() == &ArchiveMetadata::default();
        if choice == CloseChoice::Exit && empty {
            self.save_coordination
                .as_mut()
                .expect("installed")
                .close
                .slot
                .as_mut()
                .expect("checked")
                .status
                .phase = ClosePhase::EmptyCreationBaselineSatisfied;
            return Ok(());
        }
        let c = self.save_coordination.as_mut().expect("installed");
        let Some(config) = c.resident.config else {
            return self.close_fail(if choice == CloseChoice::Manual {
                E::NeedsTarget
            } else {
                E::RecoveryUnavailable
            });
        };
        if c.resident.protocol.is_none() || !c.resident.port.alive.load(Ordering::Acquire) {
            return self.close_fail(E::Unavailable);
        }
        if c.resident.save_exhausted {
            return self.close_fail(E::Limit(SaveLimit::SaveIdentity));
        }
        let sequence = c.resident.next_save;
        c.close.slot.as_mut().expect("checked").status.phase = ClosePhase::FinalCapture;
        let request = SaveRequest {
            api_version: API_VERSION,
            capture_version: HISTORY_CAPTURE_VERSION,
            key: SaveAdmissionKey {
                version: SAVE_COORDINATION_VERSION,
                session: self.session_id,
                sequence,
            },
            source: self.save_source(),
            binding: config.binding.key,
            origin: match choice {
                CloseChoice::Manual => SaveOrigin::Manual,
                CloseChoice::Exit => SaveOrigin::Exit,
            },
        };
        let child = match self.request_save_owned(request, identities, Some(ticket)) {
            Ok(a) => a.ticket(),
            Err(e) => return self.close_fail(e),
        };
        let s = &mut self
            .save_coordination
            .as_mut()
            .expect("installed")
            .close
            .slot
            .as_mut()
            .expect("checked")
            .status;
        s.child = Some(child);
        s.phase = ClosePhase::Protecting;
        Ok(())
    }
    pub fn retry_close(
        &mut self,
        ticket: CloseTicket,
        failed: SaveTicket,
        identities: &mut dyn HistoryIdentitySource,
    ) -> Result<(), SaveCoordinationError> {
        use SaveCoordinationError as E;
        self.validate_close_ticket(ticket)?;
        self.advance_save_coordination(SAVE_COORDINATION_VERSION)?;
        let c = self.save_coordination.as_ref().expect("installed");
        let s = c.close.slot.as_ref().expect("checked").status;
        if s.child != Some(failed) || failed.parent_close != Some(ticket.key) {
            return Err(E::Conflict);
        }
        if s.phase != ClosePhase::Failed {
            return Err(E::NotTerminal);
        }
        if c.facts.binding_conflict {
            return Err(E::InvalidBinding);
        }
        if c.save.slot.is_some() && self.acknowledge_save(failed)? != SaveAcknowledgement::Released
        {
            return Err(E::Busy(SaveBusy::SaveOutstanding));
        }
        // Ack drains a second observation cell. A completion may have arrived
        // since the first check; confirmed facts are never retried as failure.
        let c = self.save_coordination.as_ref().expect("installed");
        if c.facts.binding_conflict {
            return Err(E::InvalidBinding);
        }
        let latest = c.close.slot.as_ref().expect("checked").status;
        if latest.child != Some(failed) || latest.phase != ClosePhase::Failed {
            return Err(E::NotTerminal);
        }
        let s = &mut self
            .save_coordination
            .as_mut()
            .expect("installed")
            .close
            .slot
            .as_mut()
            .expect("checked")
            .status;
        s.child = None;
        s.error = None;
        s.phase = ClosePhase::SettleExisting;
        let result = self.advance_close(ticket, identities);
        if result.is_err() {
            // No replacement was admitted: keep the explicit failed-child
            // context so a later user retry need not consume another CloseKey.
            let s = &mut self
                .save_coordination
                .as_mut()
                .expect("installed")
                .close
                .slot
                .as_mut()
                .expect("checked")
                .status;
            if s.child.is_none() {
                s.child = Some(failed);
            }
        }
        result
    }
    pub fn cancel_close(&mut self, ticket: CloseTicket) -> Result<(), SaveCoordinationError> {
        self.validate_close_ticket(ticket)?;
        let c = self.save_coordination.as_mut().expect("installed");
        let s = &mut c.close.slot.as_mut().expect("checked").status;
        s.phase = ClosePhase::Cancelled;
        s.finish = None;
        let child = c
            .save
            .slot
            .as_ref()
            .filter(|save| save.ticket.parent_close == Some(ticket.key))
            .map(|save| save.ticket);
        if let Some(child) = child {
            self.cancel_save(child)?;
        }
        Ok(())
    }
    /// Only Cancelled is terminal in A3. Satisfied remains gated, without any
    /// host permission. Save/query/consumer ownership is independent of this POD.
    pub fn acknowledge_close(&mut self, ticket: CloseTicket) -> Result<(), SaveCoordinationError> {
        self.validate_close_ticket(ticket)?;
        let c = self.save_coordination.as_mut().expect("installed");
        if c.close.slot.as_ref().expect("checked").status.phase != ClosePhase::Cancelled {
            return Err(SaveCoordinationError::NotTerminal);
        }
        c.close.slot = None;
        Ok(())
    }
}
