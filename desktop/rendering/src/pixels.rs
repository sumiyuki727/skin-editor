//! Shared immutable pixels. Managed residency follows the last pixel borrower;
//! callers cannot extract an unleased inner Arc.
use std::fmt;
use std::ops::Deref;
use std::sync::Arc;
struct Allocation {
    bytes: Arc<[u8]>,
    // Declaration order releases bytes before refunding residency.
    _residency: Option<Box<dyn Send + Sync>>,
}
#[derive(Clone)]
pub struct RasterPixels(Arc<Allocation>);
impl RasterPixels {
    pub fn with_residency(bytes: Vec<u8>, residency: impl Send + Sync + 'static) -> Self {
        Self(Arc::new(Allocation {
            bytes: bytes.into(),
            _residency: Some(Box::new(residency)),
        }))
    }
    pub fn is_shared(&self) -> bool {
        Arc::strong_count(&self.0) > 1
    }
}
impl From<Vec<u8>> for RasterPixels {
    fn from(bytes: Vec<u8>) -> Self {
        Self(Arc::new(Allocation {
            bytes: bytes.into(),
            _residency: None,
        }))
    }
}
impl Deref for RasterPixels {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        &self.0.bytes
    }
}
impl AsRef<[u8]> for RasterPixels {
    fn as_ref(&self) -> &[u8] {
        self
    }
}
impl fmt::Debug for RasterPixels {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RasterPixels")
            .field("bytes", &self.len())
            .finish_non_exhaustive()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    struct Lease(Arc<AtomicUsize>);
    impl Drop for Lease {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }
    #[test]
    fn cloned_pixels_share_bytes_and_release_once_after_last_borrow() {
        let drops = Arc::new(AtomicUsize::new(0));
        let a = RasterPixels::with_residency(vec![1, 2, 3, 4], Lease(drops.clone()));
        let b = a.clone();
        assert_eq!(a.as_ptr(), b.as_ptr());
        assert!(a.is_shared());
        drop(a);
        assert_eq!(drops.load(Ordering::SeqCst), 0);
        assert!(!b.is_shared());
        assert_eq!(b.as_ref(), [1, 2, 3, 4]);
        drop(b);
        assert_eq!(drops.load(Ordering::SeqCst), 1);
    }
}
