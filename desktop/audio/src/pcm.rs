//! One shared byte budget. Reservation and resident samples use the same lease.
use std::sync::{
    Arc, Weak,
    atomic::{AtomicUsize, Ordering},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PcmUsage {
    pub bytes: usize,
    pub peak_bytes: usize,
    pub limit: usize,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PcmError {
    ResourceLimit,
    SharedBuffer,
}
#[derive(Debug)]
struct Budget {
    limit: usize,
    per_sound: usize,
    used: AtomicUsize,
    peak: AtomicUsize,
}
#[derive(Clone, Debug)]
pub struct PcmBudget(Arc<Budget>);
impl Default for PcmBudget {
    fn default() -> Self {
        Self::new(32 * 1024 * 1024, 4 * 1024 * 1024).expect("positive defaults")
    }
}
impl PcmBudget {
    pub fn new(limit: usize, per_sound: usize) -> Result<Self, PcmError> {
        if limit == 0 || per_sound == 0 || per_sound > limit {
            return Err(PcmError::ResourceLimit);
        }
        Ok(Self(Arc::new(Budget {
            limit,
            per_sound,
            used: AtomicUsize::new(0),
            peak: AtomicUsize::new(0),
        })))
    }
    pub fn reserve(&self, samples: usize) -> Result<PcmReservation, PcmError> {
        let bytes = samples
            .checked_mul(std::mem::size_of::<f32>())
            .filter(|n| *n > 0 && *n <= self.0.per_sound)
            .ok_or(PcmError::ResourceLimit)?;
        let previous = self
            .0
            .used
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |n| {
                n.checked_add(bytes).filter(|n| *n <= self.0.limit)
            })
            .map_err(|_| PcmError::ResourceLimit)?;
        self.0.peak.fetch_max(previous + bytes, Ordering::Relaxed);
        Ok(PcmReservation {
            budget: self.clone(),
            samples,
        })
    }
    pub fn usage(&self) -> PcmUsage {
        PcmUsage {
            bytes: self.0.used.load(Ordering::Acquire),
            peak_bytes: self.0.peak.load(Ordering::Relaxed),
            limit: self.0.limit,
        }
    }
    pub fn same_pool(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
#[derive(Debug)]
pub struct PcmReservation {
    budget: PcmBudget,
    samples: usize,
}
impl PcmReservation {
    pub fn allocate(self) -> PcmBuffer {
        let samples = vec![0.0; self.samples];
        PcmBuffer(Arc::new(Allocation {
            samples,
            _lease: self,
        }))
    }
}
impl Drop for PcmReservation {
    fn drop(&mut self) {
        self.budget
            .0
            .used
            .fetch_sub(self.samples * std::mem::size_of::<f32>(), Ordering::AcqRel);
    }
}
#[derive(Debug)]
struct Allocation {
    samples: Vec<f32>,
    _lease: PcmReservation,
}
#[derive(Clone, Debug)]
pub struct PcmBuffer(Arc<Allocation>);
impl PcmBuffer {
    pub fn samples(&self) -> &[f32] {
        &self.0.samples
    }
    pub fn samples_mut(&mut self) -> Result<&mut [f32], PcmError> {
        Arc::get_mut(&mut self.0)
            .map(|a| a.samples.as_mut_slice())
            .ok_or(PcmError::SharedBuffer)
    }
    pub fn budget(&self) -> &PcmBudget {
        &self.0._lease.budget
    }
    pub fn same_allocation(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
    pub fn downgrade(&self) -> PcmWeak {
        PcmWeak(Arc::downgrade(&self.0))
    }
}
#[derive(Clone, Debug)]
pub struct PcmWeak(Weak<Allocation>);
impl PcmWeak {
    pub fn is_alive(&self) -> bool {
        self.0.strong_count() > 0
    }
    pub(crate) fn upgrade(&self) -> Option<PcmBuffer> {
        self.0.upgrade().map(PcmBuffer)
    }
}
