//! Rust 1.95 Global/RawVec proof: docs/K1_6B_STORAGE_PROOF.md.
//! Only this module can allocate or grow a variable object buffer.
use crate::coordinator::{AllocationKind, PayloadLease, PoolState, ResourcePool};
use crate::{MAX_CIRCLES, MAX_REFERENCE_CIRCLES, ValueError, WorkCounts};
use core::ops::{Index, IndexMut};
use std::rc::Rc;

pub(crate) fn capacity(objects: usize) -> Result<usize, ValueError> {
    if objects > MAX_REFERENCE_CIRCLES {
        return Err(ValueError::ObjectLimit);
    }
    Ok(objects.max(MAX_CIRCLES))
}
pub(crate) fn bytes<T>(capacity: usize) -> Result<usize, ValueError> {
    if core::mem::size_of::<T>() == 0 {
        return Err(ValueError::StorageLayout);
    }
    std::alloc::Layout::array::<T>(capacity)
        .map(|layout| layout.size())
        .map_err(|_| ValueError::StorageLimit)
}

/// Fixed capacity after a single reservation from an empty vector. No Clone,
/// resize, collect, shrink, whole-buffer formatting or unbounded append API.
pub(crate) struct Buffer<T: Copy> {
    data: Vec<T>,
    // Drop payload before returning its allocation accounting.
    _payload: Option<PayloadLease>,
    bound: usize,
}
impl<T: Copy> Buffer<T> {
    pub(crate) fn new(bound: usize) -> Self {
        Self {
            data: Vec::new(),
            _payload: None,
            bound,
        }
    }
    pub(crate) fn allocate(
        &mut self,
        stats: &mut WorkCounts,
        pool: Option<&Rc<PoolState>>,
        kind: AllocationKind,
        class: ResourcePool,
    ) -> Result<(), crate::PreparationError> {
        assert_eq!(self.data.capacity(), 0);
        let amount = bytes::<T>(self.bound)?;
        stats.allocation(amount);
        if let Some(pool) = pool {
            pool.check_allocation(kind)?;
        }
        self.data
            .try_reserve_exact(self.bound)
            .map_err(|_| crate::PreparationError::AllocationUnavailable(kind))?;
        // A defensive canary, not the pre-allocation upper-bound proof.
        if self.data.capacity() != self.bound {
            return Err(ValueError::StorageLayout.into());
        }
        if let Some(pool) = pool {
            self._payload = Some(PayloadLease::new(pool, class, amount));
        }
        Ok(())
    }
    pub(crate) fn push(&mut self, value: T) {
        assert!(self.data.len() < self.bound && self.data.capacity() == self.bound);
        self.data.push(value);
    }
    pub(crate) fn get(&self, index: usize) -> Option<&T> {
        self.data.get(index)
    }
    pub(crate) fn len(&self) -> usize {
        self.data.len()
    }
    pub(crate) fn capacity(&self) -> usize {
        self.data.capacity()
    }
}
impl<T: Copy> Index<usize> for Buffer<T> {
    type Output = T;
    fn index(&self, index: usize) -> &T {
        &self.data[index]
    }
}
impl<T: Copy> IndexMut<usize> for Buffer<T> {
    fn index_mut(&mut self, index: usize) -> &mut T {
        &mut self.data[index]
    }
}
impl<T: Copy> core::fmt::Debug for Buffer<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Buffer")
            .field("len", &self.len())
            .field("capacity", &self.capacity())
            .finish()
    }
}
impl<T: Copy + PartialEq> PartialEq for Buffer<T> {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}
impl<T: Copy + Eq> Eq for Buffer<T> {}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invalid_request_layouts_are_rejected_before_any_vector_reservation() {
        assert_eq!(bytes::<u64>(usize::MAX), Err(ValueError::StorageLimit));
        assert_eq!(bytes::<()>(4), Err(ValueError::StorageLayout));
        assert_eq!(capacity(usize::MAX), Err(ValueError::ObjectLimit));
    }
}
