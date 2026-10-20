use core::num::{NonZeroU64, NonZeroU128};

use skin_editor_domain::{
    Affine2, Asset, AssetId, ComponentId, ComponentNode, ConfigRecord, DocumentId, ElementDocument,
    LayerId, LayerNode, ModelError, ObjectId, Opacity, OutputId, OutputMapping, PixelSize,
};

/// Resolved by the host, separately from the project identity. Never reuse for
/// another open history session; no clock/global generator is hidden in core.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct HistorySessionId(NonZeroU128);
impl HistorySessionId {
    pub const fn new(value: u128) -> Option<Self> {
        match NonZeroU128::new(value) {
            Some(v) => Some(Self(v)),
            None => None,
        }
    }
    pub const fn get(self) -> u128 {
        self.0.get()
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RequestId(NonZeroU128);
impl RequestId {
    pub const fn new(value: u128) -> Option<Self> {
        match NonZeroU128::new(value) {
            Some(v) => Some(Self(v)),
            None => None,
        }
    }
    pub const fn get(self) -> u128 {
        self.0.get()
    }
}

/// Fixed-size local identity, meaningful only within HistorySessionId.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct NodeId(NonZeroU64);
impl NodeId {
    pub const fn new(value: u64) -> Option<Self> {
        match NonZeroU64::new(value) {
            Some(v) => Some(Self(v)),
            None => None,
        }
    }
    pub const fn get(self) -> u64 {
        self.0.get()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExpectedState {
    pub head: NodeId,
    pub revision: u64,
}

/// v2 is a closed value protocol. New-object IDs are explicit candidates;
/// application admits them once per session, and never reuses a committed ID.
/// Positions are zero-based; moves index the destination AFTER detaching.
#[derive(Clone, Debug, PartialEq)]
pub enum Command {
    RegisterAsset {
        asset: Asset,
    },
    RemoveAsset {
        asset: AssetId,
    },
    RenameAsset {
        asset: AssetId,
        name: String,
    },
    CreateDocument {
        document: ElementDocument,
    },
    RemoveDocument {
        document: DocumentId,
    },
    ResizeDocument {
        document: DocumentId,
        canvas: PixelSize,
    },
    AddLayer {
        document: DocumentId,
        node: LayerNode,
        parent: Option<LayerId>,
        index: usize,
    },
    RemoveLayer {
        document: DocumentId,
        layer: LayerId,
    },
    MoveLayer {
        document: DocumentId,
        layer: LayerId,
        parent: Option<LayerId>,
        index: usize,
    },
    SetLayerTransform {
        document: DocumentId,
        layer: LayerId,
        transform: Affine2,
    },
    SetLayerProperties {
        document: DocumentId,
        layer: LayerId,
        opacity: Opacity,
        visible: bool,
    },
    SetLayerAsset {
        document: DocumentId,
        layer: LayerId,
        asset: AssetId,
    },
    AddComponent {
        node: ComponentNode,
        parent: Option<ComponentId>,
        index: usize,
    },
    RemoveComponent {
        component: ComponentId,
    },
    MoveComponent {
        component: ComponentId,
        parent: Option<ComponentId>,
        index: usize,
    },
    SetComponentTransform {
        component: ComponentId,
        transform: Affine2,
    },
    SetComponentMapping {
        component: ComponentId,
        document_to_component: Affine2,
    },
    SetOutputMapping {
        mapping: OutputMapping,
    },
    RemoveOutputMapping {
        output: OutputId,
    },
    InsertConfigRecord {
        index: usize,
        record: ConfigRecord,
    },
    SetConfigRecord {
        index: usize,
        record: ConfigRecord,
    },
    RemoveConfigRecord {
        index: usize,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct CommandRequest {
    pub api_version: u16,
    pub session_id: HistorySessionId,
    pub request_id: RequestId,
    pub expected: ExpectedState,
    pub commands: Vec<Command>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommandStatus {
    Committed,
    NoChange,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandResult {
    pub api_version: u16,
    pub session_id: HistorySessionId,
    pub request_id: RequestId,
    /// State at the original execution, even when replaying a cached result.
    pub state: ExpectedState,
    pub status: CommandStatus,
    pub changed_objects: Vec<ObjectId>,
    pub configuration_changed: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommandFailure {
    Missing(ObjectId),
    ReusedIdentity(ObjectId),
    InvalidIndex {
        index: usize,
        len: usize,
    },
    InvalidOperation(&'static str),
    InvalidParent(ObjectId),
    WouldCreateCycle(ObjectId),
    AssetInUse {
        asset: AssetId,
        layers: Vec<LayerId>,
    },
    DocumentInUse(DocumentId),
    Model(ModelError),
}

impl From<ModelError> for CommandFailure {
    fn from(value: ModelError) -> Self {
        Self::Model(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommandError {
    UnsupportedVersion(u16),
    WrongSession,
    RequestIdReused,
    Conflict {
        expected: ExpectedState,
        actual: ExpectedState,
    },
    Command {
        index: usize,
        reason: CommandFailure,
    },
    Resource {
        blob: skin_editor_domain::BlobRef,
        error: crate::ResourceError,
    },
    ResourceLimit(&'static str),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidationResult {
    pub api_version: u16,
    pub state: ExpectedState,
    pub would_change: bool,
    /// Metadata validation only. Submission must still acquire resource pins.
    pub resource_check_pending: bool,
    pub cached: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HistoryQueryError {
    WrongSession,
    UnknownNode(NodeId),
    InvalidHistory(ModelError),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HistoryEntry {
    pub api_version: u16,
    pub session_id: HistorySessionId,
    pub id: NodeId,
    pub parent: Option<NodeId>,
    pub request_id: Option<RequestId>,
    pub changed_objects: Vec<ObjectId>,
    pub configuration_changed: bool,
}
