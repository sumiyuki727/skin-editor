//! Pure, validated project values. Active-state writes belong to application.
//!
//! Resource and document identities cannot be interchanged:
//! ```compile_fail
//! use skin_editor_domain::{AssetId, DocumentId};
//! let document = DocumentId::new(1).unwrap();
//! let _: AssetId = document;
//! ```
#![forbid(unsafe_code)]

mod assets;
mod components;
mod configuration;
mod forest;
mod geometry;
mod identity;
mod layers;
mod output;
mod project;

pub use assets::{Asset, AssetKind, AssetLibrary, BlobRef};
pub use components::{ComponentKind, ComponentNode, ComponentTree};
pub use configuration::{ConfigInterpretation, ConfigRecord, SkinConfiguration};
pub use geometry::{Affine2, Bounds, Opacity, PixelSize};
pub use identity::{AssetId, ComponentId, DocumentId, LayerId, OutputId};
pub use layers::{ElementDocument, LayerKind, LayerNode, LayerTree};
pub use output::{OutputMapping, OutputPath};
pub use project::{MODEL_VERSION, ProjectDefinition, SkinProject};

use core::num::NonZeroU128;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObjectId {
    Asset(AssetId),
    Document(DocumentId),
    Layer(LayerId),
    Component(ComponentId),
    Output(OutputId),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ModelError {
    InvalidValue(&'static str),
    DuplicateId(ObjectId),
    MissingReference(ObjectId),
    MultipleParents(ObjectId),
    Unreachable(ObjectId),
    Cycle,
    DuplicateMembership(DocumentId),
    OutputConflict(String),
    InvalidOutputPath(String),
    UnsupportedVersion(u16),
    NotInvertible,
    NumericOverflow,
}

/// A project identity supplied by the application; zero is never a valid ID.
///
/// Its representation cannot be changed by an adapter:
/// ```compile_fail
/// use skin_editor_domain::ProjectId;
/// let id = ProjectId::new(7).unwrap();
/// let _ = id.0;
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ProjectId(NonZeroU128);

impl ProjectId {
    /// Validates a resolved identity, without a clock, random generator or I/O.
    pub const fn new(value: u128) -> Option<Self> {
        match NonZeroU128::new(value) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }

    /// Returns the stable value, independently of a name, path or display order.
    pub const fn get(self) -> u128 {
        self.0.get()
    }
}

#[cfg(test)]
mod tests {
    use super::ProjectId;

    #[test]
    fn rejects_missing_identity() {
        assert_eq!(ProjectId::new(0), None);
    }

    #[test]
    fn preserves_full_width_resolved_identities() {
        let first = ProjectId::new(1).unwrap();
        let last = ProjectId::new(u128::MAX).unwrap();
        assert_ne!(first, last);
        assert_eq!(last.get(), u128::MAX);
        assert_eq!(ProjectId::new(last.get()), Some(last));
    }
}
