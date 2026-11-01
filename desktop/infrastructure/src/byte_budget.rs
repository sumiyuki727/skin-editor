use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

pub(crate) struct ByteBudget {
    limit: usize,
    used: AtomicUsize,
    peak: AtomicUsize,
}
/// Counters only: this does not retain caches, resources or their bytes.
#[derive(Clone)]
pub struct ByteObservation(Arc<ByteBudget>);
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ByteUsage {
    pub bytes: usize,
    pub peak_bytes: usize,
}
impl ByteObservation {
    pub fn usage(&self) -> ByteUsage {
        ByteUsage {
            bytes: self.0.used(),
            peak_bytes: self.0.peak(),
        }
    }
}
impl ByteBudget {
    pub fn observation(self: &Arc<Self>) -> ByteObservation {
        ByteObservation(self.clone())
    }
    pub fn new(limit: usize) -> Arc<Self> {
        Arc::new(Self {
            limit,
            used: AtomicUsize::new(0),
            peak: AtomicUsize::new(0),
        })
    }
    pub fn reserve(self: &Arc<Self>, bytes: usize) -> Option<ByteLease> {
        let previous = self
            .used
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |n| {
                n.checked_add(bytes).filter(|n| *n <= self.limit)
            })
            .ok()?;
        self.peak.fetch_max(previous + bytes, Ordering::Relaxed);
        Some(ByteLease {
            budget: self.clone(),
            bytes,
        })
    }
    pub fn used(&self) -> usize {
        self.used.load(Ordering::Acquire)
    }
    pub fn peak(&self) -> usize {
        self.peak.load(Ordering::Relaxed)
    }
}
pub(crate) struct ByteLease {
    budget: Arc<ByteBudget>,
    bytes: usize,
}
impl Drop for ByteLease {
    fn drop(&mut self) {
        self.budget.used.fetch_sub(self.bytes, Ordering::AcqRel);
    }
}
