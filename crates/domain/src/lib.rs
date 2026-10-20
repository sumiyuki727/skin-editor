//! Pure domain value foundations. The editable project model comes in P1-1.
#![forbid(unsafe_code)]

use core::num::NonZeroU128;

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
