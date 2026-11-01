//! Single project writer: value commands, isolated transactions and history.
#![forbid(unsafe_code)]

mod address;
mod audio_editor;
mod audio_port;
mod contract;
mod draft;
mod draft_contract;
mod execute;
mod history;
mod history_control;
mod history_queries;
mod preview_port;
mod resources;
pub use audio_editor::*;
pub use audio_port::*;
pub use preview_port::*;

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use skin_editor_domain::{BlobRef, ObjectId};
use skin_editor_domain::{ProjectId, SkinProject};

pub use address::{MAX_BRANCH_ADDRESS, MAX_NODE_ADDRESS};
pub use contract::{
    AddressError, BranchCoordinate, Command, CommandError, CommandFailure, CommandRequest,
    CommandResult, CommandStatus, ExpectedState, HistoryAction, HistoryAncestorsPage,
    HistoryChildrenPage, HistoryEntry, HistoryError, HistoryQueryError, HistoryRequest,
    HistoryResult, HistorySessionId, HistoryStatistics, HistoryStatus, HistorySummary,
    MAX_HISTORY_PAGE, NodeId, RequestId, ValidationResult,
};
pub use draft_contract::{
    DraftAction, DraftError, DraftId, DraftInfo, DraftOutcome, DraftRequest, DraftResult,
    DraftSnapshot, DraftState,
};
pub use resources::{BlobLease, ResourceError, ResourceStore};

/// Pure values needed to form commands through the application dependency.
/// No active-state setter, storage implementation or project-replacement API.
pub mod values {
    pub use skin_editor_domain::{
        Affine2, Asset, AssetId, AssetKind, AudioComponent, AudioComponentId, AudioMetadata,
        BlobRef, Bounds, ComponentId, ComponentKind, ComponentNode, ConfigInterpretation,
        ConfigRecord, DocumentId, ElementDocument, Gain, LayerId, LayerKind, LayerNode, LayerTree,
        ModelError, ObjectId, Opacity, OutputId, OutputMapping, OutputPath, OutputSource,
        PixelSize, SkinProject, SoundSlotKey,
    };
}

/// Current version of the owned application query values.
pub const API_VERSION: u16 = 6;

/// A structured lifecycle validation error, usable without UI types.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SessionError {
    /// Zero is reserved and cannot identify a project.
    InvalidProjectId,
    InvalidHistorySessionId,
    InvalidLimits,
}

/// An owned query value, independent of mutable session storage.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectInfo {
    /// Version of this query contract, not the future project file format.
    pub api_version: u16,
    /// Stable resolved project identity.
    pub project_id: u128,
    pub session_id: HistorySessionId,
    pub state: ExpectedState,
}

/// An owned, validated snapshot; this is not a project-replacement request.
#[derive(Clone, Debug, PartialEq)]
pub struct ProjectSnapshot {
    pub api_version: u16,
    pub session_id: HistorySessionId,
    pub state: ExpectedState,
    pub project: SkinProject,
}

#[derive(Clone, Debug, PartialEq)]
pub struct HistoricalSnapshot {
    pub api_version: u16,
    pub session_id: HistorySessionId,
    pub node: NodeId,
    pub project: SkinProject,
}

/// Explicit session safeguards, not stable compatibility or product budgets.
/// Receipts are never silently evicted: a full session rejects new request IDs.
#[derive(Clone, Copy, Debug)]
pub struct SessionLimits {
    pub commands_per_request: usize,
    pub retained_requests: usize,
    pub history_nodes: usize,
}
impl Default for SessionLimits {
    fn default() -> Self {
        Self {
            commands_per_request: 1024,
            retained_requests: 65_536,
            history_nodes: 65_536,
        }
    }
}

enum Receipt {
    Command {
        request: CommandRequest,
        outcome: Result<CommandResult, CommandError>,
    },
    History {
        request: HistoryRequest,
        outcome: Result<HistoryResult, HistoryError>,
    },
    Draft {
        request: DraftRequest,
        outcome: Result<DraftResult, DraftError>,
    },
}

/// Owns the active validated project. No whole-state replacement is exposed.
///
/// Adapters cannot obtain or replace its internal storage:
/// ```compile_fail
/// use skin_editor_application::ProjectSession;
/// let mut session = ProjectSession::new(7, 70).unwrap();
/// session.project = session.project;
/// ```
/// Candidate execution is private; adapters cannot call it as another writer:
/// ```compile_fail
/// use skin_editor_application::execute::apply;
/// ```
pub struct ProjectSession {
    project: SkinProject,
    session_id: HistorySessionId,
    state: ExpectedState,
    /// Last effective history navigation, independent of content-node identity.
    navigation_revision: u64,
    history: history::History,
    /// Only forks need a remembered direct child. Navigation metadata, never
    /// node content or a copy of its ancestor path. Explicit checkout resets it.
    redo_preferences: BTreeMap<NodeId, NodeId>,
    used_ids: BTreeSet<ObjectId>,
    receipts: BTreeMap<RequestId, Receipt>,
    pins: BTreeMap<BlobRef, Arc<dyn BlobLease>>,
    resources: Box<dyn ResourceStore>,
    limits: SessionLimits,
    draft: Option<draft::Draft>,
    next_draft_id: u64,
    /// Cancellation tombstones are bounded by the admitted Begin requests.
    cancelled_drafts: BTreeSet<DraftId>,
}

impl ProjectSession {
    /// Creates an empty project root; this does not load external content.
    pub fn new(
        resolved_project_id: u128,
        resolved_history_session_id: u128,
    ) -> Result<Self, SessionError> {
        Self::with_resources(
            resolved_project_id,
            resolved_history_session_id,
            Box::new(resources::NoResources),
            SessionLimits::default(),
        )
    }

    /// Desktop composition supplies the store; commands never carry a store,
    /// closure, filesystem path or mutable project. Creates only an empty root.
    pub fn with_resources(
        resolved_project_id: u128,
        resolved_history_session_id: u128,
        resources: Box<dyn ResourceStore>,
        limits: SessionLimits,
    ) -> Result<Self, SessionError> {
        let identity = ProjectId::new(resolved_project_id).ok_or(SessionError::InvalidProjectId)?;
        let session_id = HistorySessionId::new(resolved_history_session_id)
            .ok_or(SessionError::InvalidHistorySessionId)?;
        if limits.commands_per_request == 0
            || limits.retained_requests == 0
            || limits.history_nodes == 0
        {
            return Err(SessionError::InvalidLimits);
        }
        let project = SkinProject::empty(identity);
        Ok(Self {
            history: history::History::new(project.clone()),
            project,
            session_id,
            state: ExpectedState {
                head: NodeId::new(1).expect("nonzero root"),
                revision: 0,
            },
            used_ids: BTreeSet::new(),
            navigation_revision: 0,
            redo_preferences: BTreeMap::new(),
            receipts: BTreeMap::new(),
            resources,
            pins: BTreeMap::new(),
            limits,
            draft: None,
            next_draft_id: 1,
            cancelled_drafts: BTreeSet::new(),
        })
    }

    /// Returns a detached value; editing the returned value cannot change state.
    pub fn project_info(&self) -> ProjectInfo {
        ProjectInfo {
            api_version: API_VERSION,
            project_id: self.project.id().get(),
            session_id: self.session_id,
            state: self.state,
        }
    }

    pub fn snapshot(&self) -> ProjectSnapshot {
        ProjectSnapshot {
            api_version: API_VERSION,
            session_id: self.session_id,
            state: self.state,
            project: self.project.clone(),
        }
    }

    pub fn history_len(&self) -> usize {
        self.history.nodes.len()
    }

    pub fn history_entry(
        &self,
        session_id: HistorySessionId,
        id: NodeId,
    ) -> Result<HistoryEntry, HistoryQueryError> {
        if session_id != self.session_id {
            return Err(HistoryQueryError::WrongSession);
        }
        self.history
            .entry(session_id, id)
            .ok_or(HistoryQueryError::UnknownNode(id))
    }

    /// Materialises an old value without checking out or changing active state.
    pub fn history_snapshot(
        &self,
        session_id: HistorySessionId,
        id: NodeId,
    ) -> Result<HistoricalSnapshot, HistoryQueryError> {
        if session_id != self.session_id {
            return Err(HistoryQueryError::WrongSession);
        }
        let project = if id == self.state.head {
            self.project.clone()
        } else {
            self.history.materialize(id)?
        };
        Ok(HistoricalSnapshot {
            api_version: API_VERSION,
            session_id,
            node: id,
            project,
        })
    }

    fn envelope(&self, request: &CommandRequest) -> Result<(), CommandError> {
        if request.session_id != self.session_id {
            return Err(CommandError::WrongSession);
        }
        if request.api_version != API_VERSION {
            return Err(CommandError::UnsupportedVersion(request.api_version));
        }
        if request.commands.len() > self.limits.commands_per_request {
            return Err(CommandError::ResourceLimit("commands_per_request"));
        }
        Ok(())
    }

    fn cached(
        &self,
        request: &CommandRequest,
    ) -> Result<Option<&Result<CommandResult, CommandError>>, CommandError> {
        match self.receipts.get(&request.request_id) {
            Some(Receipt::Command {
                request: old,
                outcome,
            }) if old == request => Ok(Some(outcome)),
            Some(_) => Err(CommandError::RequestIdReused),
            None => Ok(None),
        }
    }

    fn prepare(
        &self,
        expected: ExpectedState,
        commands: &[Command],
    ) -> Result<(SkinProject, BTreeSet<ObjectId>), CommandError> {
        if expected != self.state {
            return Err(CommandError::Conflict {
                expected,
                actual: self.state,
            });
        }
        if self.receipts.len() >= self.limits.retained_requests {
            return Err(CommandError::ResourceLimit("retained_requests"));
        }
        let mut candidate = self.project.clone();
        let mut used = self.used_ids.clone();
        for (index, command) in commands.iter().enumerate() {
            candidate = execute::apply(&candidate, command, &mut used)
                .map_err(|reason| CommandError::Command { index, reason })?;
        }
        if candidate != self.project {
            if self.history.nodes.len() >= self.limits.history_nodes {
                return Err(CommandError::ResourceLimit("history_nodes"));
            }
            self.history.plan_append(self.state.head)?;
            self.state
                .revision
                .checked_add(1)
                .ok_or(CommandError::ResourceLimit("revision"))?;
        }
        Ok((candidate, used))
    }

    /// Read-only metadata preflight. It does not pin bytes, reserve IDs, cache
    /// receipts or guarantee later commit against a changed revision/store.
    pub fn validate(&self, request: &CommandRequest) -> Result<ValidationResult, CommandError> {
        self.envelope(request)?;
        if let Some(receipt) = self.cached(request)? {
            receipt.as_ref().map_err(Clone::clone)?;
            return Ok(ValidationResult {
                api_version: API_VERSION,
                state: self.state,
                would_change: false,
                resource_check_pending: false,
                cached: true,
            });
        }
        let (candidate, _) = self.prepare(request.expected, &request.commands)?;
        let would_change = candidate != self.project;
        Ok(ValidationResult {
            api_version: API_VERSION,
            state: self.state,
            would_change,
            resource_check_pending: would_change
                && candidate
                    .assets()
                    .iter()
                    .any(|a| !self.pins.contains_key(&a.blob())),
            cached: false,
        })
    }

    /// Commits a content batch through the same publication path as draft
    /// completion. An effective batch publishes one revision and history node.
    ///
    /// Adapters need only the application's public contract:
    /// ```
    /// use skin_editor_application::{
    ///     API_VERSION, Command, CommandRequest, ProjectSession, RequestId,
    ///     values::{DocumentId, ElementDocument, LayerTree, PixelSize},
    /// };
    /// let mut session = ProjectSession::new(7, 70).unwrap();
    /// let info = session.project_info();
    /// session.submit(CommandRequest {
    ///     api_version: API_VERSION, session_id: info.session_id,
    ///     request_id: RequestId::new(1).unwrap(), expected: info.state,
    ///     commands: vec![Command::CreateDocument { document: ElementDocument::new(
    ///         DocumentId::new(1).unwrap(), PixelSize::new(64, 64).unwrap(),
    ///         LayerTree::default(),
    ///     ) }],
    /// }).unwrap();
    /// assert_eq!(session.history_len(), 2);
    /// ```
    pub fn submit(&mut self, request: CommandRequest) -> Result<CommandResult, CommandError> {
        self.envelope(&request)?;
        if let Some(receipt) = self.cached(&request)? {
            return receipt.clone();
        }
        if let Some(draft) = &self.draft {
            return Err(CommandError::Busy(draft.state.id));
        }
        // Never exceed the receipt cap even when recording rejection.
        if self.receipts.len() >= self.limits.retained_requests {
            return Err(CommandError::ResourceLimit("retained_requests"));
        }
        let outcome = self.commit(&request);
        self.receipts.insert(
            request.request_id,
            Receipt::Command {
                request,
                outcome: outcome.clone(),
            },
        );
        outcome
    }

    fn commit(&mut self, request: &CommandRequest) -> Result<CommandResult, CommandError> {
        let (candidate, used) = self.prepare(request.expected, &request.commands)?;
        self.publish(request.request_id, candidate, used, &BTreeMap::new())
    }

    fn pin_candidate(
        &self,
        candidate: &SkinProject,
        temporary: &BTreeMap<BlobRef, Arc<dyn BlobLease>>,
    ) -> Result<BTreeMap<BlobRef, Arc<dyn BlobLease>>, CommandError> {
        let mut pins = BTreeMap::new();
        for asset in candidate.assets().iter() {
            let blob = asset.blob();
            // Existing identical typed references were checked on admission.
            // New aliases must still be verified even if the bytes are pinned.
            if let skin_editor_domain::AssetKind::Audio(metadata) = asset.kind()
                && self.project.assets().get(asset.id()).map(|old| old.kind()) != Some(asset.kind())
                && self.resources.audio_metadata(blob) != Some(*metadata)
            {
                return Err(CommandError::Resource {
                    blob,
                    error: ResourceError::MetadataMismatch,
                });
            }
            if !self.pins.contains_key(&blob) && !pins.contains_key(&blob) {
                let pin = match temporary.get(&blob) {
                    Some(pin) => pin.clone(),
                    None => self
                        .resources
                        .pin(blob)
                        .map_err(|error| CommandError::Resource { blob, error })?,
                };
                if pin.reference() != blob {
                    return Err(CommandError::Resource {
                        blob,
                        error: ResourceError::ReferenceMismatch,
                    });
                }
                pins.insert(blob, pin);
            }
        }
        Ok(pins)
    }

    fn publish(
        &mut self,
        request_id: RequestId,
        candidate: SkinProject,
        used: BTreeSet<ObjectId>,
        temporary: &BTreeMap<BlobRef, Arc<dyn BlobLease>>,
    ) -> Result<CommandResult, CommandError> {
        if candidate == self.project {
            return Ok(CommandResult {
                api_version: API_VERSION,
                session_id: self.session_id,
                request_id,
                state: self.state,
                status: CommandStatus::NoChange,
                changed_objects: Vec::new(),
                configuration_changed: false,
            });
        }
        if self.history.nodes.len() >= self.limits.history_nodes {
            return Err(CommandError::ResourceLimit("history_nodes"));
        }
        let plan = self.history.plan_append(self.state.head)?;
        let id = plan.id;
        let revision = self
            .state
            .revision
            .checked_add(1)
            .ok_or(CommandError::ResourceLimit("revision"))?;
        let patch = history::Patch::between(&self.project, &candidate);
        let new_pins = self.pin_candidate(&candidate, temporary)?;
        let result = CommandResult {
            api_version: API_VERSION,
            session_id: self.session_id,
            request_id,
            state: ExpectedState { head: id, revision },
            status: CommandStatus::Committed,
            changed_objects: patch.changed.clone(),
            configuration_changed: patch.configuration_changed(),
        };
        // All fallible validation/acquisition is complete before publishing.
        self.history.append(plan, patch, request_id);
        self.project = candidate;
        self.state = result.state;
        self.used_ids = used;
        self.pins.extend(new_pins);
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::{API_VERSION, ProjectSession, SessionError};

    #[test]
    fn rejects_an_invalid_session_identity() {
        assert!(matches!(
            ProjectSession::new(0, 70),
            Err(SessionError::InvalidProjectId)
        ));
    }

    #[test]
    fn queries_keep_the_version_and_identity() {
        let first = ProjectSession::new(7, 70).unwrap();
        let other = ProjectSession::new(8, 80).unwrap();
        assert_eq!(first.project_info().api_version, API_VERSION);
        assert_eq!(first.project_info().project_id, 7);
        assert_ne!(first.project_info(), other.project_info());
        assert_eq!(first.project_info(), first.project_info());
    }

    #[test]
    fn modifying_a_query_value_cannot_modify_the_active_session() {
        let session = ProjectSession::new(7, 70).unwrap();
        let mut view = session.project_info();
        view.project_id = 100;
        view.api_version = 999;
        assert_ne!(view, session.project_info());
        assert_eq!(session.project_info().project_id, 7);
        assert_eq!(session.project_info().api_version, API_VERSION);
    }

    #[test]
    fn new_session_owns_a_valid_empty_project() {
        let session = ProjectSession::new(7, 70).unwrap();
        let snapshot = session.snapshot();
        assert_eq!(snapshot.api_version, API_VERSION);
        assert_eq!(snapshot.project.id().get(), 7);
        assert_eq!(snapshot.project.documents().len(), 0);
        assert_eq!(snapshot.project.assets().iter().len(), 0);
    }

    #[test]
    fn snapshot_values_are_detached_even_for_populated_project_content() {
        use skin_editor_domain::{
            ConfigInterpretation, ConfigRecord, DocumentId, ElementDocument, LayerTree, PixelSize,
            ProjectDefinition, ProjectId, SkinConfiguration, SkinProject,
        };
        let mut definition = ProjectDefinition::empty(ProjectId::new(7).unwrap());
        definition.documents.push(ElementDocument::new(
            DocumentId::new(10).unwrap(),
            PixelSize::new(8, 8).unwrap(),
            LayerTree::default(),
        ));
        definition.configuration = SkinConfiguration::new(vec![
            ConfigRecord::new(
                b"unknown: preserve\r\n".to_vec(),
                ConfigInterpretation::Opaque,
            )
            .unwrap(),
        ]);
        // A unit fixture inside the owner, not a public restoration/write API.
        let project = SkinProject::from_definition(definition).unwrap();
        let mut session = ProjectSession::new(7, 70).unwrap();
        session.history = super::history::History::new(project.clone());
        session.project = project;
        let mut snapshot = session.snapshot();
        assert_eq!(snapshot.project.documents().len(), 1);
        snapshot.api_version = 999;
        snapshot.project = SkinProject::empty(ProjectId::new(99).unwrap());
        let actual = session.snapshot();
        assert_eq!(actual.api_version, API_VERSION);
        assert_eq!(actual.project.id().get(), 7);
        assert_eq!(actual.project.documents().len(), 1);
        assert_eq!(
            actual.project.configuration().to_bytes(),
            b"unknown: preserve\r\n"
        );
    }

    #[test]
    fn session_scope_and_safeguards_are_validated_at_creation() {
        assert!(matches!(
            ProjectSession::new(7, 0),
            Err(SessionError::InvalidHistorySessionId)
        ));
        let limits = super::SessionLimits {
            history_nodes: 0,
            ..Default::default()
        };
        assert!(matches!(
            ProjectSession::with_resources(7, 70, Box::new(super::resources::NoResources), limits),
            Err(SessionError::InvalidLimits)
        ));
        assert!(super::RequestId::new(0).is_none());
        assert!(super::NodeId::new(0).is_none());
        assert_eq!(super::NodeId::new(u64::MAX).unwrap().get(), u64::MAX);
    }

    fn config_request(session: &ProjectSession, id: u128) -> super::CommandRequest {
        super::CommandRequest {
            api_version: API_VERSION,
            session_id: session.session_id,
            request_id: super::RequestId::new(id).unwrap(),
            expected: session.state,
            commands: vec![super::Command::InsertConfigRecord {
                index: 0,
                record: skin_editor_domain::ConfigRecord::new(
                    b"test".to_vec(),
                    skin_editor_domain::ConfigInterpretation::Opaque,
                )
                .unwrap(),
            }],
        }
    }

    #[test]
    fn reversible_patch_restores_exact_values_without_reexecuting_commands() {
        let mut session = ProjectSession::new(7, 70).unwrap();
        let root = session.project.clone();
        let request = config_request(&session, 1);
        let result = session.submit(request).unwrap();
        let patch = session.history.nodes[&result.state.head]
            .patch
            .as_ref()
            .unwrap();
        assert_eq!(patch.apply(&root, true).unwrap(), session.project);
        assert_eq!(patch.apply(&session.project, false).unwrap(), root);
    }

    #[test]
    fn node_and_revision_exhaustion_reject_before_publishing() {
        let mut session = ProjectSession::new(7, 70).unwrap();
        session.history.next_id = u64::MAX;
        let request = config_request(&session, 1);
        assert_eq!(session.submit(request).unwrap().state.head.get(), u64::MAX);
        let before = session.snapshot();
        let request = config_request(&session, 2);
        assert_eq!(
            session.submit(request),
            Err(super::CommandError::ResourceLimit("node_id"))
        );
        assert_eq!(session.snapshot(), before);
        assert_eq!(session.history.nodes.len(), 2);
        let mut session = ProjectSession::new(7, 71).unwrap();
        session.state.revision = u64::MAX;
        let before = session.snapshot();
        let request = config_request(&session, 1);
        assert_eq!(
            session.submit(request),
            Err(super::CommandError::ResourceLimit("revision"))
        );
        assert_eq!(session.snapshot(), before);
        assert_eq!(session.history.nodes.len(), 1);
    }
}
