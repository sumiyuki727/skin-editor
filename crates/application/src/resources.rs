use std::sync::Arc;

use skin_editor_domain::BlobRef;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResourceError {
    Missing,
    Unavailable,
    ReferenceMismatch,
}

/// An immutable-byte lifetime pin, not necessarily resident image/file bytes.
/// The adapter must have verified hash/length and prevent reclamation while
/// this lease lives. It has no mutable project reference or active-state access.
pub trait BlobLease: Send + Sync {
    fn reference(&self) -> BlobRef;
}

pub trait ResourceStore {
    /// Pin an already staged, verified blob using the store's lifetime index.
    /// No file decoding, hash computation, network or blocking byte I/O here;
    /// staging those resources is a separate import use case before submission.
    fn pin(&self, blob: BlobRef) -> Result<Arc<dyn BlobLease>, ResourceError>;
}

pub(crate) struct NoResources;
impl ResourceStore for NoResources {
    fn pin(&self, _: BlobRef) -> Result<Arc<dyn BlobLease>, ResourceError> {
        Err(ResourceError::Missing)
    }
}
