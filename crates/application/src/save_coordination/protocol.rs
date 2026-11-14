use super::*;
use protocol_work::{CompletionRelease, ExecutionRelease, QueryRelease};

/// Synthetic fixture admission only. It is not a file verifier or a production
/// fallback. Golden tests independently verify their restored/frozen contents.
#[cfg(test)]
pub(crate) fn synthetic_authority(
    session: &ProjectSession,
    opening: bool,
) -> SaveProtocolAuthority {
    SaveProtocolAuthority {
        profile: u16::MAX,
        opening: opening.then(|| protocol_values::OpeningBaseline {
            coverage: session.coverage_stamp(),
            files: SaveFileFacts::from(session.archive_metadata.as_ref()),
        }),
    }
}
#[cfg(test)]
pub(crate) fn test_query_attempt(session: &mut ProjectSession, next: u64) {
    assert!(next != 0);
    let c = session.save_coordination.as_mut().expect("installed");
    assert!(c.query.slot.is_none());
    c.save
        .slot
        .as_mut()
        .expect("save")
        .protocol
        .as_mut()
        .expect("protocol")
        .next_attempt = Some(next);
}
#[cfg(test)]
pub(crate) fn test_query_allocation(session: &ProjectSession) -> Option<usize> {
    let c = session.save_coordination.as_ref()?;
    let control = c.query.slot.as_ref().map(|q| &q.control).or_else(|| {
        c.save
            .slot
            .as_ref()?
            .protocol
            .as_ref()?
            .first_query
            .as_ref()
    })?;
    Some(Arc::as_ptr(control) as usize)
}
#[cfg(test)]
pub(crate) fn test_candidate_weak(
    session: &ProjectSession,
) -> Option<std::sync::Weak<ArchiveMetadata>> {
    let candidate = session
        .save_coordination
        .as_ref()?
        .save
        .slot
        .as_ref()?
        .control
        .protocol
        .as_ref()?
        .candidate
        .as_ref()?;
    Some(Arc::downgrade(&candidate.metadata))
}

#[derive(Clone, Copy)]
pub(super) struct ProtocolConfig {
    pub profile: u16,
    pub candidate: SaveCandidateLimits,
}

pub(super) struct ProtocolSlot {
    pub outcome: Option<SaveProtocolOutcome>,
    pub diagnostic: Option<SaveDiagnostic>,
    pub unknown: bool,
    pub main_seen: bool,
    // Distinguish a full file noncoverage proof from merely stopping before
    // MayWrite; the latter can legitimately be followed by late NoChange.
    pub verified_uncovered: bool,
    pub first_query: Option<Arc<QueryControl>>,
    pub next_attempt: Option<u64>,
}

pub(super) struct ReconcileSlot {
    pub ticket: ReconcileTicket,
    pub control: Arc<QueryControl>,
    pub dispatched: bool,
    pub observed: bool,
}

pub(super) fn prepare_task(
    config: ProtocolConfig,
    ticket: SaveTicket,
    snapshot: &CaptureSnapshot,
    control: &mut TaskControl,
) -> ProtocolSlot {
    let manifest = snapshot.manifest();
    let (candidate, capacity_diagnostic) =
        match candidate::prepare(manifest.archive_metadata.as_ref(), ticket, config.candidate) {
            Ok(candidate) => (Some(candidate), None),
            Err(diagnostic) => (None, Some(diagnostic)),
        };
    control.protocol = Some(ProtocolTask {
        profile: config.profile,
        witness: OnceLock::new(),
        completion: OnceLock::new(),
        may_have_written: AtomicBool::new(false),
        candidate,
        capacity_diagnostic,
        coverage: SaveCoverageStamp {
            source: manifest.source,
            nodes: manifest.nodes.len(),
            admissions: manifest.admissions.len(),
        },
        baseline: SaveFileFacts::from(manifest.archive_metadata.as_ref()),
    });
    ProtocolSlot {
        outcome: None,
        diagnostic: None,
        unknown: false,
        main_seen: false,
        verified_uncovered: false,
        first_query: Some(Arc::new(QueryControl::new(
            control.first_query_cancellation.clone(),
        ))),
        next_attempt: Some(1),
    }
}

impl ProjectSession {
    /// Consumes bounded qualified observations. This action does not capture,
    /// perform I/O, allocate confirmation metadata, or mutate content/history.
    pub fn advance_save_coordination(&mut self, version: u16) -> Result<(), SaveCoordinationError> {
        if version != SAVE_COORDINATION_VERSION {
            return Err(SaveCoordinationError::UnsupportedVersion);
        }
        let c = self
            .save_coordination
            .as_mut()
            .ok_or(SaveCoordinationError::Unavailable)?;
        c.drain_protocol(&mut self.archive_metadata);
        c.refresh(&self.capture_in_flight);
        Ok(())
    }

    pub fn begin_save_reconcile(
        &mut self,
        ticket: SaveTicket,
    ) -> Result<SaveReconcileAdmission, SaveCoordinationError> {
        use SaveCoordinationError as E;
        self.validate_save_ticket(ticket)?;
        let c = self.save_coordination.as_mut().ok_or(E::Unavailable)?;
        c.check_ticket(ticket)?;
        if c.resident.protocol.is_none() {
            return Err(E::Unavailable);
        }
        c.drain_protocol(&mut self.archive_metadata);
        if let Some(query) = &c.query.slot {
            return Ok(SaveReconcileAdmission::Existing(query.ticket));
        }
        let slot = c.save.slot.as_mut().expect("current ticket");
        let state = slot.protocol.as_mut().expect("coordinated slot");
        if state.outcome.is_some() || !state.unknown {
            return Err(E::NotUnknown);
        }
        if c.resident
            .port
            .writer_released_sequence
            .load(Ordering::Acquire)
            != ticket.key.sequence
        {
            return Err(E::Busy(SaveBusy::Writer));
        }
        if !c.resident.port.alive.load(Ordering::Acquire) {
            return Err(E::Unavailable);
        }
        if slot.snapshot.is_none() {
            return Err(E::NotPrepared);
        }
        let attempt = state
            .next_attempt
            .ok_or(E::Limit(SaveLimit::QueryIdentity))?;
        let control = match state.first_query.take() {
            Some(control) => control,
            None => Arc::new(QueryControl::new(CaptureCancellation::default())),
        };
        let ticket = ReconcileTicket {
            save: ticket,
            attempt,
        };
        state.next_attempt = attempt.checked_add(1);
        c.query.slot = Some(ReconcileSlot {
            ticket,
            control,
            dispatched: false,
            observed: false,
        });
        Ok(SaveReconcileAdmission::Accepted(ticket))
    }

    pub fn take_save_reconcile(
        &mut self,
        port: &SaveProtocolPort,
        ticket: ReconcileTicket,
    ) -> Result<SaveReconcileWork, SaveCoordinationError> {
        use SaveCoordinationError as E;
        self.validate_save_ticket(ticket.save)?;
        if ticket.attempt == 0 {
            return Err(E::InvalidIdentity);
        }
        let c = self.save_coordination.as_mut().ok_or(E::Unavailable)?;
        c.check_ticket(ticket.save)?;
        if !Arc::ptr_eq(&c.resident.port, &port.preparation.state) {
            return Err(E::WrongPort);
        }
        if !c.resident.port.alive.load(Ordering::Acquire) {
            return Err(E::Unavailable);
        }
        let query = c.query.slot.as_mut().ok_or(E::Stale)?;
        if query.ticket != ticket {
            return Err(E::Stale);
        }
        if query.dispatched || query.control.result.get().is_some() {
            return Err(E::NotPending);
        }
        let slot = c.save.slot.as_ref().expect("current ticket");
        let snapshot = slot.snapshot.as_ref().ok_or(E::NotPrepared)?.clone();
        query.dispatched = true;
        Ok(SaveReconcileWork {
            snapshot,
            task: slot.control.clone(),
            control: query.control.clone(),
            ticket,
            release: QueryRelease {
                port: c.resident.port.clone(),
                ticket,
            },
        })
    }

    pub fn cancel_save_reconcile(
        &mut self,
        ticket: ReconcileTicket,
    ) -> Result<(), SaveCoordinationError> {
        use SaveCoordinationError as E;
        self.validate_save_ticket(ticket.save)?;
        if ticket.attempt == 0 {
            return Err(E::InvalidIdentity);
        }
        let c = self.save_coordination.as_mut().ok_or(E::Unavailable)?;
        c.check_ticket(ticket.save)?;
        let query = c.query.slot.as_mut().ok_or(E::Stale)?;
        if query.ticket != ticket {
            return Err(E::Stale);
        }
        query.control.cancellation.cancel();
        if !query.dispatched {
            let _ = query
                .control
                .result
                .set(SaveStorageObservation::Unknown(SaveDiagnostic::new(
                    SaveDiagnosticCode::QueryCancelled,
                )));
            c.resident
                .port
                .query_released_attempt
                .store(ticket.attempt, Ordering::Release);
            c.resident
                .port
                .query_released_save
                .store(ticket.save.key.sequence, Ordering::Release);
        }
        c.drain_protocol(&mut self.archive_metadata);
        c.refresh(&self.capture_in_flight);
        Ok(())
    }

    fn coverage_stamp(&self) -> SaveCoverageStamp {
        SaveCoverageStamp {
            source: self.save_source(),
            nodes: self.history.nodes.len(),
            admissions: self.used_ids.len(),
        }
    }

    /// Composition only. There is deliberately no production authority issuer
    /// until the B adapter has a reviewed full-proof and file-validation path.
    pub fn install_save_protocol(
        &mut self,
        config: SavePreparationConfig,
        candidate: SaveCandidateLimits,
        authority: SaveProtocolAuthority,
    ) -> Result<SaveProtocolPort, SaveCoordinationError> {
        use SaveCoordinationError as E;
        if authority.profile == 0 {
            return Err(E::UnsupportedVersion);
        }
        if candidate.logical_bytes > SAVE_CANDIDATE_LOGICAL_BYTES
            || candidate.allocation_bytes > SAVE_CANDIDATE_ALLOCATION_BYTES
        {
            return Err(E::Limit(SaveLimit::WriteCapacity));
        }
        let files = SaveFileFacts::from(self.archive_metadata.as_ref());
        if let Some(opening) = authority.opening
            && (!self.restored_history
                || self.state.revision != 0
                || self.navigation_revision != 0
                || config.binding.phase != SaveTargetPhase::ExistingVerified
                || opening.coverage != self.coverage_stamp()
                || opening.files != files
                || !self.valid_opening_facts(files))
        {
            return Err(E::InvalidBinding);
        }
        let preparation = self.install_save_preparation(config)?;
        let c = self.save_coordination.as_mut().expect("installed");
        c.resident.protocol = Some(ProtocolConfig {
            profile: authority.profile,
            candidate,
        });
        c.facts.files = files;
        c.facts.opening = authority.opening;
        c.facts.covered = authority
            .opening
            .and_then(|b| b.files.protected.map(|_| b.coverage));
        Ok(SaveProtocolPort { preparation })
    }

    fn valid_opening_facts(&self, files: SaveFileFacts) -> bool {
        let Some(latest) = files.latest else {
            return false;
        };
        let Some(protected) = files.protected else {
            return false;
        };
        let Some(persistence) = &self.persistence else {
            return false;
        };
        if latest.generation == 0
            || protected.generation == 0
            || protected.generation > latest.generation
        {
            return false;
        }
        if [files.latest, files.manual, files.protected]
            .into_iter()
            .flatten()
            .any(|fact| {
                fact.generation == 0
                    || fact.generation > latest.generation
                    || !persistence.nodes.values().any(|head| *head == fact.head)
            })
        {
            return false;
        }
        if let Some(manual) = files.manual
            && (manual.origin != SaveOrigin::Manual
                || manual.generation > protected.generation
                || !self
                    .archive_metadata
                    .manual_marks
                    .contains(&(manual.commit, manual.head)))
        {
            return false;
        }
        true
    }

    /// Fixed read-only values. This never consumes a completion, captures a
    /// project, allocates metadata or traverses history/ancestors.
    pub fn save_protocol_status(
        &self,
        version: u16,
    ) -> Result<SaveProtocolStatus, SaveCoordinationError> {
        if version != SAVE_COORDINATION_VERSION {
            return Err(SaveCoordinationError::UnsupportedVersion);
        }
        let current = self.coverage_stamp();
        let mut result = SaveProtocolStatus {
            version,
            available: false,
            active: None,
            facts: SaveFactsStatus {
                version,
                current,
                files: SaveFileFacts::default(),
                covered: None,
                protection: SaveProtectionStatus::Unavailable,
                pending_draft: self.draft.is_some(),
                binding_conflict: false,
                installed_metadata: None,
            },
        };
        let Some(c) = &self.save_coordination else {
            return Ok(result);
        };
        if c.resident.protocol.is_none() {
            return Ok(result);
        }
        result.available = c.resident.port.alive.load(Ordering::Acquire);
        result.facts.files = c.facts.files;
        result.facts.covered = c.facts.covered;
        result.facts.binding_conflict = c.facts.binding_conflict;
        result.facts.installed_metadata = c.facts.installed_metadata;
        result.facts.protection = if c.facts.covered == Some(current) {
            SaveProtectionStatus::Covered
        } else {
            SaveProtectionStatus::NeedsProtection
        };
        if let Some(slot) = &c.save.slot {
            let state = slot.protocol.as_ref().expect("coordinated slot");
            let task = slot.control.protocol.as_ref().expect("coordinated task");
            result.active = Some(SaveProtocolActive {
                ticket: slot.ticket,
                phase: c.protocol_phase(slot),
                outcome: state.outcome,
                diagnostic: state.diagnostic,
                permission: task.permission(),
                witness_retained: task.witness.get().is_some(),
                candidate: task.candidate.as_ref().map(|c| c.cost),
                acknowledged: slot.acknowledged,
                writer_released: !slot.dispatched
                    || c.resident
                        .port
                        .writer_released_sequence
                        .load(Ordering::Acquire)
                        == slot.ticket.key.sequence,
                completion_released: !slot.dispatched
                    || c.resident
                        .port
                        .completion_released_sequence
                        .load(Ordering::Acquire)
                        == slot.ticket.key.sequence,
                capture_consumers_released: !self.capture_in_flight.load(Ordering::Acquire),
                query: c.query.slot.as_ref().map(|q| q.ticket),
                query_released: c.query.slot.as_ref().is_none_or(|q| c.query_released(q)),
            });
        }
        Ok(result)
    }

    pub fn take_save_execution(
        &mut self,
        port: &SaveProtocolPort,
        ticket: SaveTicket,
    ) -> Result<SaveExecution, SaveCoordinationError> {
        self.validate_save_ticket(ticket)?;
        let c = self
            .save_coordination
            .as_ref()
            .ok_or(SaveCoordinationError::Unavailable)?;
        c.check_ticket(ticket)?;
        if c.resident.protocol.is_none() {
            return Err(SaveCoordinationError::Unavailable);
        }
        let work = self.take_save_preparation(&port.preparation, ticket)?;
        let control = work.control.clone();
        Ok(SaveExecution {
            work,
            completion: Some(SaveCompletionSender {
                control,
                ticket,
                release: CompletionRelease {
                    port: port.preparation.state.clone(),
                    sequence: ticket.key.sequence,
                },
            }),
            release: ExecutionRelease {
                port: port.preparation.state.clone(),
                sequence: ticket.key.sequence,
            },
        })
    }
}

impl Coordinator {
    fn unresolved(&mut self, diagnostic: SaveDiagnostic, contradiction: bool) {
        let slot = self.save.slot.as_mut().expect("active save");
        let state = slot.protocol.as_mut().expect("coordinated slot");
        if matches!(state.outcome, Some(SaveProtocolOutcome::Confirmed(_))) {
            if contradiction
                || matches!(
                    diagnostic.code,
                    SaveDiagnosticCode::TargetConflict
                        | SaveDiagnosticCode::TargetMissing
                        | SaveDiagnosticCode::JournalConflict
                )
            {
                self.facts.binding_conflict = true;
            }
            return;
        }
        if state.outcome.is_some() && !contradiction {
            return;
        }
        state.outcome = None;
        state.unknown = true;
        state.diagnostic = Some(diagnostic);
        slot.acknowledged = false;
    }

    fn not_committed(&mut self, diagnostic: SaveDiagnostic) {
        let slot = self.save.slot.as_mut().expect("active save");
        let state = slot.protocol.as_mut().expect("coordinated slot");
        if matches!(state.outcome, Some(SaveProtocolOutcome::Confirmed(_))) {
            // A qualified, same-witness absence/noncoverage proof contradicts
            // already confirmed coverage. Preserve the fact and forbid reuse
            // of this binding; cancellation alone never calls this branch.
            self.facts.binding_conflict = true;
            return;
        }
        let outcome = SaveProtocolOutcome::NotCommitted(diagnostic);
        if state.outcome.is_some() && state.outcome != Some(outcome) {
            slot.acknowledged = false;
        }
        state.outcome = Some(outcome);
        state.unknown = false;
        state.diagnostic = None;
    }

    fn confirm_result(&mut self, confirmation: SaveConfirmation) {
        let slot = self.save.slot.as_mut().expect("active save");
        let state = slot.protocol.as_mut().expect("coordinated slot");
        self.facts.binding_conflict |= state.verified_uncovered;
        if let Some(SaveProtocolOutcome::Confirmed(previous)) = state.outcome
            && previous.confirmed_commit == confirmation.confirmed_commit
            && previous.coverage == confirmation.coverage
        {
            return;
        }
        if state.outcome.is_some() {
            slot.acknowledged = false;
        }
        state.outcome = Some(SaveProtocolOutcome::Confirmed(confirmation));
        state.unknown = false;
        state.diagnostic = None;
    }

    fn observe(
        &mut self,
        observation: SaveStorageObservation,
        metadata: &mut Arc<ArchiveMetadata>,
    ) {
        if let SaveStorageObservation::Unknown(diagnostic) = observation {
            self.unresolved(diagnostic, false);
            return;
        }
        let witness = match observation {
            SaveStorageObservation::Committed { witness, .. }
            | SaveStorageObservation::Replayed { witness, .. }
            | SaveStorageObservation::NoChange { witness, .. }
            | SaveStorageObservation::VerifiedNotCommitted { witness, .. } => witness,
            SaveStorageObservation::Unknown(_) => unreachable!(),
        };
        let slot = self.save.slot.as_ref().expect("active save");
        let ticket = slot.ticket;
        // Old task qualifications are ignored, including another runtime
        // session with the same project/head. They cannot settle this slot.
        if witness.info().ticket.key != ticket.key {
            return;
        }
        let task = slot.control.protocol.as_ref().expect("coordinated task");
        if task.witness.get() != Some(&witness) || witness.info().ticket != ticket {
            self.unresolved(SaveDiagnostic::new(SaveDiagnosticCode::AdapterFault), true);
            return;
        }
        let coverage = task.coverage;
        match observation {
            SaveStorageObservation::Committed {
                fact,
                binding_conflict,
                ..
            }
            | SaveStorageObservation::Replayed {
                fact,
                binding_conflict,
                ..
            } => {
                let Some(candidate) = &task.candidate else {
                    self.unresolved(SaveDiagnostic::new(SaveDiagnosticCode::AdapterFault), true);
                    return;
                };
                if !task.may_have_written.load(Ordering::Acquire) || candidate.fact != fact {
                    self.unresolved(SaveDiagnostic::new(SaveDiagnosticCode::AdapterFault), true);
                    return;
                }
                let conflicting_previous = slot.protocol.as_ref().is_some_and(|state| {
                    match state.outcome {
                        Some(SaveProtocolOutcome::Confirmed(c)) => c.confirmed_commit != Some(fact),
                        // MayWrite was issued and the writer exited before
                        // reconciliation. An earlier definite absence proof
                        // and this matching commit cannot both describe the
                        // same unchanged target. Keep the commit, lock binding.
                        Some(SaveProtocolOutcome::NotCommitted(_)) => true,
                        None => false,
                    }
                });
                let newer = self
                    .facts
                    .files
                    .latest
                    .is_some_and(|known| known.generation > fact.generation);
                if !newer {
                    // Arc handle transfer/clone cannot allocate. All variable
                    // metadata storage and the Manual mark already exist.
                    *metadata = candidate.metadata.clone();
                    self.facts.files = SaveFileFacts::from(metadata.as_ref());
                    self.facts.covered = Some(coverage);
                    self.facts.installed_metadata = Some(candidate.cost);
                    let binding = &mut self.resident.config.binding;
                    binding.phase = SaveTargetPhase::ExistingVerified;
                    binding.history = Some(ticket.history);
                    binding.expected = ExpectedFileCommit {
                        commit: Some(fact.commit),
                        generation: Some(fact.generation),
                    };
                }
                self.facts.binding_conflict |= binding_conflict || conflicting_previous || newer;
                let kind = if matches!(observation, SaveStorageObservation::Committed { .. }) {
                    SaveConfirmationKind::Committed
                } else {
                    SaveConfirmationKind::Replayed
                };
                self.confirm_result(SaveConfirmation {
                    kind,
                    files: self.facts.files,
                    coverage,
                    confirmed_commit: Some(fact),
                });
            }
            SaveStorageObservation::NoChange { files, basis, .. } => {
                let manual = files.manual;
                let intent = ticket.origin == SaveOrigin::Manual
                    && ticket.target.phase == SaveTargetPhase::ExistingVerified
                    && manual
                        .is_some_and(|m| m.origin == SaveOrigin::Manual && m.head == ticket.head)
                    && files.latest.is_some()
                    && files.protected.is_some();
                let basis_valid = basis == SaveNoChangeBasis::CurrentManual
                    || self
                        .facts
                        .opening
                        .is_some_and(|b| b.coverage == coverage && b.files == files);
                if files != task.baseline || !intent || !basis_valid {
                    self.unresolved(SaveDiagnostic::new(SaveDiagnosticCode::AdapterFault), true);
                    return;
                }
                if matches!(slot.protocol.as_ref().and_then(|s| s.outcome),
                    Some(SaveProtocolOutcome::Confirmed(c)) if c.confirmed_commit.is_some())
                {
                    self.facts.binding_conflict = true;
                    return;
                }
                // No new CommitId, generation, Manual mark or metadata object.
                self.facts.covered = Some(coverage);
                self.confirm_result(SaveConfirmation {
                    kind: SaveConfirmationKind::NoChange,
                    files: self.facts.files,
                    coverage,
                    confirmed_commit: None,
                });
            }
            SaveStorageObservation::VerifiedNotCommitted { diagnostic, .. } => {
                let diagnostic = task.capacity_diagnostic.unwrap_or(diagnostic);
                self.save
                    .slot
                    .as_mut()
                    .expect("active save")
                    .protocol
                    .as_mut()
                    .expect("coordinated slot")
                    .verified_uncovered = true;
                self.not_committed(diagnostic);
            }
            SaveStorageObservation::Unknown(_) => unreachable!(),
        }
    }

    pub(super) fn drain_protocol(&mut self, metadata: &mut Arc<ArchiveMetadata>) {
        let Some(slot) = &mut self.save.slot else {
            return;
        };
        let Some(state) = &mut slot.protocol else {
            return;
        };
        let task = slot.control.protocol.as_ref().expect("coordinated task");
        let main = if !state.main_seen {
            task.completion.get().copied()
        } else {
            None
        };
        if main.is_some() {
            state.main_seen = true;
        }
        if let Some(observation) = main {
            self.observe(observation, metadata);
        }

        let slot = self.save.slot.as_ref().expect("active save");
        let state = slot.protocol.as_ref().expect("coordinated slot");
        let task = slot.control.protocol.as_ref().expect("coordinated task");
        if state.outcome.is_none() && !state.unknown {
            if !slot.dispatched && slot.control.cancellation.is_cancelled() {
                self.not_committed(SaveDiagnostic::new(SaveDiagnosticCode::Cancelled));
            } else if slot.dispatched
                && self
                    .resident
                    .port
                    .writer_released_sequence
                    .load(Ordering::Acquire)
                    == slot.ticket.key.sequence
            {
                if task.may_have_written.load(Ordering::Acquire) {
                    self.unresolved(
                        SaveDiagnostic::new(SaveDiagnosticCode::OutcomeUnknown),
                        false,
                    );
                } else {
                    let diagnostic = match slot.control.result.get() {
                        Some(SavePreparationOutcome::StoppedNotSaved(d)) => *d,
                        _ if slot.control.cancellation.is_cancelled() => {
                            SaveDiagnostic::new(SaveDiagnosticCode::Cancelled)
                        }
                        _ => SaveDiagnostic::new(SaveDiagnosticCode::Abandoned),
                    };
                    self.not_committed(diagnostic);
                }
            }
        }
        let query_observation = self.query.slot.as_mut().and_then(|query| {
            if query.observed {
                return None;
            }
            let result = query.control.result.get().copied();
            if result.is_some() {
                query.observed = true;
            }
            result
        });
        if let Some(observation) = query_observation {
            self.observe(observation, metadata);
        }

        let settled = self
            .save
            .slot
            .as_ref()
            .and_then(|s| s.protocol.as_ref())
            .is_some_and(|s| s.outcome.is_some());
        if settled && let Some(query) = &mut self.query.slot {
            query.control.cancellation.cancel();
            if !query.dispatched {
                query.observed = true;
                self.resident
                    .port
                    .query_released_attempt
                    .store(query.ticket.attempt, Ordering::Release);
                self.resident
                    .port
                    .query_released_save
                    .store(query.ticket.save.key.sequence, Ordering::Release);
            }
        }
        if self
            .query
            .slot
            .as_ref()
            .is_some_and(|q| q.observed && self.query_released(q))
        {
            self.query.slot = None;
        }
    }

    pub(super) fn protocol_phase(&self, slot: &SaveSlot) -> SaveProtocolPhase {
        let state = slot.protocol.as_ref().expect("coordinated slot");
        let task = slot.control.protocol.as_ref().expect("coordinated task");
        if state.outcome.is_some() {
            SaveProtocolPhase::Settled
        } else if state.unknown
            || self.query.slot.is_some()
            || (self
                .resident
                .port
                .writer_released_sequence
                .load(Ordering::Acquire)
                == slot.ticket.key.sequence
                && task.may_have_written.load(Ordering::Acquire)
                && task.completion.get().is_none())
        {
            SaveProtocolPhase::Reconciling
        } else if slot.dispatched {
            SaveProtocolPhase::Working
        } else {
            SaveProtocolPhase::DispatchPending
        }
    }

    fn query_released(&self, query: &ReconcileSlot) -> bool {
        self.resident
            .port
            .query_released_save
            .load(Ordering::Acquire)
            == query.ticket.save.key.sequence
            && self
                .resident
                .port
                .query_released_attempt
                .load(Ordering::Acquire)
                == query.ticket.attempt
    }
}
