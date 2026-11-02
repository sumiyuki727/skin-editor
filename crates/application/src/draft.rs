use crate::{
    API_VERSION, BlobLease, Command, CommandError, DraftAction, DraftError, DraftId, DraftInfo,
    DraftOutcome, DraftRequest, DraftResult, DraftSnapshot, DraftState, HistorySessionId,
    ProjectSession, Receipt,
};
use skin_editor_domain::{BlobRef, ObjectId, SkinProject};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

pub(crate) struct Draft {
    pub state: DraftState,
    commands: Vec<Command>,
    pub(super) project: SkinProject,
    used: BTreeSet<ObjectId>,
    pins: BTreeMap<BlobRef, Arc<dyn BlobLease>>,
}

impl ProjectSession {
    pub fn draft_info(
        &self,
        session_id: HistorySessionId,
    ) -> Result<Option<DraftInfo>, DraftError> {
        if session_id != self.session_id {
            return Err(CommandError::WrongSession.into());
        }
        Ok(self.draft.as_ref().map(|draft| DraftInfo {
            api_version: API_VERSION,
            session_id,
            state: draft.state,
            has_changes: draft.project != self.project,
        }))
    }

    pub(super) fn expected_draft(&self, expected: DraftState) -> Result<&Draft, DraftError> {
        let draft = self.draft.as_ref().ok_or(DraftError::NoActiveDraft)?;
        if expected != draft.state {
            return Err(DraftError::Conflict {
                expected,
                actual: draft.state,
            });
        }
        Ok(draft)
    }

    /// Detached candidate, explicitly labelled with its committed base and draft revision.
    pub fn draft_snapshot(
        &self,
        session_id: HistorySessionId,
        expected: DraftState,
    ) -> Result<DraftSnapshot, DraftError> {
        let info = self
            .draft_info(session_id)?
            .ok_or(DraftError::NoActiveDraft)?;
        let draft = self.expected_draft(expected)?;
        Ok(DraftSnapshot {
            info,
            project: draft.project.clone(),
        })
    }

    /// One owner and one active draft. All requests share the command/history
    /// receipt namespace. Busy is transient and never consumes a receipt.
    pub fn edit_draft(&mut self, request: DraftRequest) -> Result<DraftResult, DraftError> {
        if request.session_id != self.session_id {
            return Err(CommandError::WrongSession.into());
        }
        if request.api_version != API_VERSION {
            return Err(CommandError::UnsupportedVersion(request.api_version).into());
        }
        if let DraftAction::Replace { commands, .. } = &request.action
            && commands.len() > self.limits.commands_per_request
        {
            return Err(CommandError::ResourceLimit("commands_per_request").into());
        }
        if let Some(receipt) = self.receipts.get(&request.request_id) {
            return match receipt {
                Receipt::Draft {
                    request: old,
                    outcome,
                } if old == &request => outcome.clone(),
                _ => Err(CommandError::RequestIdReused.into()),
            };
        }
        if matches!(request.action, DraftAction::Begin { .. })
            && let Some(draft) = &self.draft
        {
            return Err(CommandError::Busy(draft.state.id).into());
        }
        // Keep the final slot for a valid cancel even after a failed commit.
        // Begin needs room for its receipt, one attempt, and that cancel.
        let minimum = match request.action {
            DraftAction::Begin { .. } => 3,
            DraftAction::Cancel { expected } if self.expected_draft(expected).is_ok() => 1,
            _ if self.draft.is_some() => 2,
            _ => 1,
        };
        if self
            .limits
            .retained_requests
            .saturating_sub(self.receipts.len())
            < minimum
        {
            return Err(CommandError::ResourceLimit("retained_requests").into());
        }
        let outcome = self.draft_once(&request).map(|outcome| DraftResult {
            api_version: API_VERSION,
            session_id: self.session_id,
            request_id: request.request_id,
            outcome,
        });
        self.receipts.insert(
            request.request_id,
            Receipt::Draft {
                request,
                outcome: outcome.clone(),
            },
        );
        outcome
    }

    fn draft_once(&mut self, request: &DraftRequest) -> Result<DraftOutcome, DraftError> {
        match &request.action {
            DraftAction::Begin { expected } => {
                if *expected != self.state {
                    return Err(CommandError::Conflict {
                        expected: *expected,
                        actual: self.state,
                    }
                    .into());
                }
                let id = DraftId::new(self.next_draft_id)
                    .ok_or(CommandError::ResourceLimit("draft_id"))?;
                self.draft = Some(Draft {
                    state: DraftState {
                        id,
                        revision: 0,
                        base: self.state,
                    },
                    commands: Vec::new(),
                    project: self.project.clone(),
                    used: self.used_ids.clone(),
                    pins: BTreeMap::new(),
                });
                self.next_draft_id = id.get().checked_add(1).unwrap_or(0);
            }
            DraftAction::Replace { expected, commands } => {
                let draft = self.expected_draft(*expected)?;
                if draft.commands != *commands {
                    let revision = draft
                        .state
                        .revision
                        .checked_add(1)
                        .ok_or(CommandError::ResourceLimit("draft_revision"))?;
                    let (project, used) = self.prepare(expected.base, commands)?;
                    let pins = self.pin_candidate(&project, &draft.pins)?;
                    self.draft = Some(Draft {
                        state: DraftState {
                            revision,
                            ..*expected
                        },
                        commands: commands.clone(),
                        project,
                        used,
                        pins,
                    });
                } else if expected.base != self.state {
                    return Err(CommandError::Conflict {
                        expected: expected.base,
                        actual: self.state,
                    }
                    .into());
                }
            }
            DraftAction::Commit { expected } => {
                let draft = self.expected_draft(*expected)?;
                if expected.base != self.state {
                    return Err(CommandError::Conflict {
                        expected: expected.base,
                        actual: self.state,
                    }
                    .into());
                }
                let (project, used, pins, operation) = (
                    draft.project.clone(),
                    draft.used.clone(),
                    draft.pins.clone(),
                    crate::CapturedOperation::transaction(
                        &draft.commands,
                        crate::OperationOrigin::Draft,
                    ),
                );
                let result = self.publish(request.request_id, project, used, &pins, operation)?;
                self.draft = None;
                return Ok(DraftOutcome::Committed(result));
            }
            DraftAction::Cancel { expected } => {
                self.expected_draft(*expected)?;
                self.cancelled_drafts.insert(expected.id);
                self.draft = None;
                return Ok(DraftOutcome::Cancelled(*expected));
            }
        }
        Ok(DraftOutcome::Active(
            self.draft_info(self.session_id)?.expect("active draft"),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RequestId;
    fn apply(
        session: &mut ProjectSession,
        id: u128,
        action: DraftAction,
    ) -> Result<DraftResult, DraftError> {
        session.edit_draft(DraftRequest {
            api_version: API_VERSION,
            session_id: session.session_id,
            request_id: RequestId::new(id).unwrap(),
            action,
        })
    }
    fn begin(session: &mut ProjectSession) -> DraftState {
        apply(
            session,
            1,
            DraftAction::Begin {
                expected: session.state,
            },
        )
        .unwrap();
        session.draft.as_ref().unwrap().state
    }
    fn command() -> Command {
        Command::InsertConfigRecord {
            index: 0,
            record: crate::values::ConfigRecord::new(
                b"test".to_vec(),
                crate::values::ConfigInterpretation::Opaque,
            )
            .unwrap(),
        }
    }
    #[test]
    fn draft_id_and_revision_exhaustion_never_change_committed_state() {
        let mut session = ProjectSession::new(7, 70).unwrap();
        let before = session.snapshot();
        session.next_draft_id = 0;
        assert_eq!(
            apply(
                &mut session,
                1,
                DraftAction::Begin {
                    expected: before.state
                }
            ),
            Err(DraftError::Request(CommandError::ResourceLimit("draft_id")))
        );
        assert!(session.draft.is_none());
        assert_eq!(session.snapshot(), before);
        session.next_draft_id = 1;
        apply(
            &mut session,
            2,
            DraftAction::Begin {
                expected: before.state,
            },
        )
        .unwrap();
        session.draft.as_mut().unwrap().state.revision = u64::MAX;
        let expected = session.draft.as_ref().unwrap().state;
        assert_eq!(
            apply(
                &mut session,
                3,
                DraftAction::Replace {
                    expected,
                    commands: vec![command()]
                }
            ),
            Err(DraftError::Request(CommandError::ResourceLimit(
                "draft_revision"
            )))
        );
        apply(&mut session, 4, DraftAction::Cancel { expected }).unwrap();
        assert_eq!(session.snapshot(), before);
        assert_eq!(session.history_len(), 1);
    }
    #[test]
    fn unexpected_base_change_rejects_commit_and_still_allows_cancel() {
        let mut session = ProjectSession::new(7, 70).unwrap();
        let state = begin(&mut session);
        apply(
            &mut session,
            2,
            DraftAction::Replace {
                expected: state,
                commands: vec![command()],
            },
        )
        .unwrap();
        let expected = session.draft.as_ref().unwrap().state;
        // Only internal fault injection can change the base while the owner is Busy.
        session.state.revision += 1;
        let before = session.snapshot();
        assert!(matches!(
            apply(&mut session, 3, DraftAction::Commit { expected }),
            Err(DraftError::Request(CommandError::Conflict { .. }))
        ));
        assert_eq!(session.snapshot(), before);
        apply(&mut session, 4, DraftAction::Cancel { expected }).unwrap();
        assert_eq!(session.snapshot(), before);
    }
    #[test]
    fn failed_publication_keeps_draft_for_cancel_or_new_request_retry() {
        let mut session = ProjectSession::new(7, 70).unwrap();
        let state = begin(&mut session);
        apply(
            &mut session,
            2,
            DraftAction::Replace {
                expected: state,
                commands: vec![command()],
            },
        )
        .unwrap();
        let expected = session.draft.as_ref().unwrap().state;
        let before = session.snapshot();
        session.limits.history_nodes = 1;
        let failure = apply(&mut session, 3, DraftAction::Commit { expected });
        assert_eq!(
            failure,
            Err(DraftError::Request(CommandError::ResourceLimit(
                "history_nodes"
            )))
        );
        assert_eq!(session.snapshot(), before);
        assert_eq!(session.history_len(), 1);
        assert_eq!(session.draft.as_ref().unwrap().state, expected);
        session.limits.history_nodes = 10;
        assert_eq!(
            apply(&mut session, 3, DraftAction::Commit { expected }),
            failure
        );
        apply(&mut session, 4, DraftAction::Commit { expected }).unwrap();
        assert_eq!(session.history_len(), 2);
        assert!(session.draft.is_none());
    }
}
