use crate::{
    API_VERSION, ExpectedState, HistoryAction, HistoryError, HistoryRequest, HistoryResult,
    HistoryStatus, NodeId, ProjectSession, Receipt, history::Patch,
};

impl ProjectSession {
    /// Changes the active historical head, never stored node content. Uses the
    /// same receipt namespace, expected state and serial owner as submit.
    pub fn navigate(&mut self, request: HistoryRequest) -> Result<HistoryResult, HistoryError> {
        if request.session_id != self.session_id {
            return Err(HistoryError::WrongSession);
        }
        if request.api_version != API_VERSION {
            return Err(HistoryError::UnsupportedVersion(request.api_version));
        }
        if let Some(receipt) = self.receipts.get(&request.request_id) {
            return match receipt {
                Receipt::History {
                    request: old,
                    outcome,
                } if old == &request => outcome.clone(),
                _ => Err(HistoryError::RequestIdReused),
            };
        }
        if self.receipts.len() >= self.limits.retained_requests {
            return Err(HistoryError::ResourceLimit("retained_requests"));
        }
        let outcome = self.navigate_once(&request);
        self.receipts.insert(
            request.request_id,
            Receipt::History {
                request,
                outcome: outcome.clone(),
            },
        );
        outcome
    }

    fn navigation_target(&self, action: &HistoryAction) -> Result<NodeId, HistoryError> {
        let current = self.history.node(self.state.head)?;
        match action {
            HistoryAction::Undo => Ok(current.parent.unwrap_or(self.state.head)),
            HistoryAction::Checkout { node } => {
                self.history.node(*node)?;
                Ok(*node)
            }
            HistoryAction::Redo { child: Some(child) } => {
                if self.history.node(*child)?.parent != Some(self.state.head) {
                    return Err(HistoryError::NotAChild {
                        parent: self.state.head,
                        child: *child,
                    });
                }
                Ok(*child)
            }
            HistoryAction::Redo { child: None } => match current.children.as_slice() {
                [] => Ok(self.state.head),
                [child] => Ok(*child),
                _ => Err(HistoryError::AmbiguousRedo {
                    child_count: current.children.len(),
                }),
            },
        }
    }

    fn navigate_once(&mut self, request: &HistoryRequest) -> Result<HistoryResult, HistoryError> {
        if request.expected != self.state {
            return Err(HistoryError::Conflict {
                expected: request.expected,
                actual: self.state,
            });
        }
        let target = self.navigation_target(&request.action)?;
        let previous = self.state;
        if target == self.state.head {
            return Ok(HistoryResult {
                api_version: API_VERSION,
                session_id: self.session_id,
                request_id: request.request_id,
                previous,
                state: previous,
                status: HistoryStatus::NoChange,
                changed_objects: Vec::new(),
                configuration_changed: false,
            });
        }
        let revision = self
            .state
            .revision
            .checked_add(1)
            .ok_or(HistoryError::ResourceLimit("revision"))?;
        let candidate = self
            .history
            .transition(self.state.head, target, &self.project)?;
        for asset in candidate.assets().iter() {
            if !self.pins.contains_key(&asset.blob()) {
                return Err(HistoryError::MissingPinnedResource(asset.blob()));
            }
        }
        let change = Patch::between(&self.project, &candidate);
        let result = HistoryResult {
            api_version: API_VERSION,
            session_id: self.session_id,
            request_id: request.request_id,
            previous,
            state: ExpectedState {
                head: target,
                revision,
            },
            status: HistoryStatus::Changed,
            changed_objects: change.changed.clone(),
            configuration_changed: change.configuration_changed(),
        };
        // No fallible work after publishing. IDs, leases and descendants stay.
        self.project = candidate;
        self.state = result.state;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Command, CommandRequest, HistoryQueryError, RequestId, SessionLimits, values::*};

    fn edit(session: &mut ProjectSession, id: u128) -> NodeId {
        session
            .submit(CommandRequest {
                api_version: API_VERSION,
                session_id: session.session_id,
                request_id: RequestId::new(id).unwrap(),
                expected: session.state,
                commands: vec![Command::InsertConfigRecord {
                    index: 0,
                    record: ConfigRecord::new(vec![id as u8], ConfigInterpretation::Opaque)
                        .unwrap(),
                }],
            })
            .unwrap()
            .state
            .head
    }
    fn request(session: &ProjectSession, id: u128, action: HistoryAction) -> HistoryRequest {
        HistoryRequest {
            api_version: API_VERSION,
            session_id: session.session_id,
            request_id: RequestId::new(id).unwrap(),
            expected: session.state,
            action,
        }
    }

    #[test]
    fn revision_exhaustion_preserves_active_head_but_allows_no_change() {
        let mut session = ProjectSession::new(7, 70).unwrap();
        edit(&mut session, 1);
        session.state.revision = u64::MAX;
        let before = session.snapshot();
        let undo = request(&session, 2, HistoryAction::Undo);
        assert_eq!(
            session.navigate(undo),
            Err(HistoryError::ResourceLimit("revision"))
        );
        assert_eq!(session.snapshot(), before);
        let same = request(
            &session,
            3,
            HistoryAction::Checkout {
                node: session.state.head,
            },
        );
        assert_eq!(
            session.navigate(same).unwrap().status,
            HistoryStatus::NoChange
        );
        assert_eq!(session.snapshot(), before);
        assert_eq!(session.history_len(), 2);
    }

    #[test]
    fn failed_replay_discards_intermediate_candidate_and_caches_failure() {
        let mut session = ProjectSession::new(7, 70).unwrap();
        let root = session.state.head;
        edit(&mut session, 1);
        let middle = edit(&mut session, 2);
        edit(&mut session, 3);
        let before = session.snapshot();
        // Inject corruption only inside the owner. Undoing the leaf succeeds
        // privately before this missing middle patch makes the checkout fail.
        let patch = session.history.nodes.get_mut(&middle).unwrap().patch.take();
        let checkout = request(&session, 4, HistoryAction::Checkout { node: root });
        let failure = session.navigate(checkout.clone());
        assert!(matches!(
            failure,
            Err(HistoryError::Query(HistoryQueryError::InvalidHistory(_)))
        ));
        assert_eq!(session.snapshot(), before);
        assert_eq!(session.history_len(), 4);
        session.history.nodes.get_mut(&middle).unwrap().patch = patch;
        let retry = request(&session, 5, HistoryAction::Checkout { node: root });
        session.navigate(retry).unwrap();
        let after = session.snapshot();
        assert_eq!(session.navigate(checkout), failure);
        assert_eq!(session.snapshot(), after);
    }

    #[test]
    fn commands_and_navigation_share_receipt_capacity_without_evicting_replays() {
        let mut session = ProjectSession::with_resources(
            7,
            70,
            Box::new(crate::resources::NoResources),
            SessionLimits {
                retained_requests: 2,
                ..Default::default()
            },
        )
        .unwrap();
        edit(&mut session, 1);
        let undo = request(&session, 2, HistoryAction::Undo);
        let result = session.navigate(undo.clone()).unwrap();
        let before = session.snapshot();
        let redo = request(&session, 3, HistoryAction::Redo { child: None });
        assert_eq!(
            session.navigate(redo),
            Err(HistoryError::ResourceLimit("retained_requests"))
        );
        assert_eq!(session.navigate(undo), Ok(result));
        assert_eq!(session.snapshot(), before);
        assert_eq!(session.receipts.len(), 2);
    }

    #[test]
    fn missing_internal_resource_pin_prevents_restoring_head() {
        use std::sync::Arc;
        struct Lease(BlobRef);
        impl crate::BlobLease for Lease {
            fn reference(&self) -> BlobRef {
                self.0
            }
        }
        struct Store;
        impl crate::ResourceStore for Store {
            fn pin(
                &self,
                blob: BlobRef,
            ) -> Result<Arc<dyn crate::BlobLease>, crate::ResourceError> {
                Ok(Arc::new(Lease(blob)))
            }
        }
        let mut session =
            ProjectSession::with_resources(7, 70, Box::new(Store), SessionLimits::default())
                .unwrap();
        let blob = BlobRef::new([1; 32], 4);
        let result = session
            .submit(CommandRequest {
                api_version: API_VERSION,
                session_id: session.session_id,
                request_id: RequestId::new(1).unwrap(),
                expected: session.state,
                commands: vec![Command::RegisterAsset {
                    asset: Asset::new(
                        AssetId::new(1).unwrap(),
                        "test".into(),
                        blob,
                        AssetKind::RasterImage(PixelSize::new(1, 1).unwrap()),
                    )
                    .unwrap(),
                }],
            })
            .unwrap();
        let undo = request(&session, 2, HistoryAction::Undo);
        session.navigate(undo).unwrap();
        // Simulate an internal lifetime invariant failure, with no public setter.
        session.pins.clear();
        let before = session.snapshot();
        let redo = request(
            &session,
            3,
            HistoryAction::Checkout {
                node: result.state.head,
            },
        );
        assert_eq!(
            session.navigate(redo),
            Err(HistoryError::MissingPinnedResource(blob))
        );
        assert_eq!(session.snapshot(), before);
        assert_eq!(session.history_len(), 2);
    }
}
