//! One bounded reusable upload allocation. No project, worker or UI ownership.
use crate::{RenderError, error};
use std::sync::{
    Arc,
    atomic::{AtomicU8, AtomicU64, Ordering},
};
use std::time::{Duration, Instant};

pub const MAX_STAGING_BYTES: u64 = 80 * 1024 * 1024;
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SourceUpload {
    #[default]
    QueueWrite,
    ReusableStaging,
}
#[derive(Clone, Copy, Debug, Default)]
pub struct StagingUsage {
    pub bytes: u64,
    pub peak_bytes: u64,
    pub allocations: u64,
    pub copies: u64,
    pub remaps: u64,
}
#[derive(Default)]
struct Budget {
    bytes: AtomicU64,
    peak: AtomicU64,
    allocations: AtomicU64,
    copies: AtomicU64,
    remaps: AtomicU64,
}
struct Reservation {
    budget: Arc<Budget>,
    bytes: u64,
}
impl Budget {
    fn reserve(self: &Arc<Self>, bytes: u64) -> Result<Reservation, RenderError> {
        if bytes == 0 || !bytes.is_multiple_of(4) || bytes > MAX_STAGING_BYTES {
            return Err(error("invalid staging size or budget exceeded"));
        }
        let old = self
            .bytes
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |used| {
                used.checked_add(bytes).filter(|n| *n <= MAX_STAGING_BYTES)
            })
            .map_err(|_| error("staging still leased or budget exceeded"))?;
        self.peak.fetch_max(old + bytes, Ordering::Relaxed);
        self.allocations.fetch_add(1, Ordering::Relaxed);
        Ok(Reservation {
            budget: self.clone(),
            bytes,
        })
    }
    fn usage(&self) -> StagingUsage {
        StagingUsage {
            bytes: self.bytes.load(Ordering::Acquire),
            peak_bytes: self.peak.load(Ordering::Relaxed),
            allocations: self.allocations.load(Ordering::Relaxed),
            copies: self.copies.load(Ordering::Relaxed),
            remaps: self.remaps.load(Ordering::Relaxed),
        }
    }
}
impl Drop for Reservation {
    fn drop(&mut self) {
        self.budget.bytes.fetch_sub(self.bytes, Ordering::AcqRel);
    }
}
struct Allocation {
    buffer: wgpu::Buffer, // release native handle before accounting its lease
    mapped: AtomicU8,
    _reservation: Reservation,
}
enum State {
    Mapped,
    Submitted(wgpu::SubmissionIndex),
    Mapping(wgpu::SubmissionIndex),
}
struct Slot {
    allocation: Arc<Allocation>,
    bytes: u64,
    state: State,
}
impl Slot {
    fn map(&mut self, device: &wgpu::Device, budget: &Budget) -> Result<(), RenderError> {
        if let State::Submitted(index) = &self.state {
            let index = index.clone();
            self.allocation.mapped.store(0, Ordering::Release);
            let lease = self.allocation.clone();
            self.allocation
                .buffer
                .slice(..)
                .map_async(wgpu::MapMode::Write, move |result| {
                    lease
                        .mapped
                        .store(if result.is_ok() { 1 } else { 2 }, Ordering::Release);
                });
            self.state = State::Mapping(index);
            budget.remaps.fetch_add(1, Ordering::Relaxed);
        }
        if let State::Mapping(index) = &self.state {
            // Executed only by the preparation caller, never the GUI/audio
            // callback. A timeout retains Mapping + its allocation/lease.
            device
                .poll(wgpu::PollType::Wait {
                    submission_index: Some(index.clone()),
                    timeout: Some(Duration::from_secs(3)),
                })
                .map_err(|e| error(&format!("staging map wait: {e}")))?;
            match self.allocation.mapped.load(Ordering::Acquire) {
                1 => self.state = State::Mapped,
                2 => return Err(error("staging remap failed")),
                _ => return Err(error("staging remap not completed")),
            }
        }
        Ok(())
    }
}
#[derive(Default)]
pub(crate) struct ReusableUpload {
    slot: Option<Slot>,
    budget: Arc<Budget>,
}
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct UploadTiming {
    pub wait_ms: f64,
    pub allocation_ms: f64,
    pub copy_submit_ms: f64,
}
impl ReusableUpload {
    pub fn usage(&self) -> StagingUsage {
        self.budget.usage()
    }
    pub fn clear(&mut self) {
        self.slot = None;
    } // submitted/map callbacks keep leases
    pub fn write(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        destination: &wgpu::Buffer,
        pixels: &[u8],
    ) -> Result<UploadTiming, RenderError> {
        let bytes = pixels.len() as u64;
        // Validate before waiting, evicting or allocating anything.
        if bytes == 0
            || !bytes.is_multiple_of(4)
            || bytes > MAX_STAGING_BYTES
            || bytes > destination.size()
        {
            return Err(error("invalid reusable upload size"));
        }
        let started = Instant::now();
        if let Some(slot) = &mut self.slot {
            slot.map(device, &self.budget)?;
        }
        let wait_ms = started.elapsed().as_secs_f64() * 1000.;
        let started = Instant::now();
        if self.slot.as_ref().is_none_or(|s| s.bytes < bytes) {
            self.slot = None; // old completion has actually returned before refund
            let reservation = self.budget.reserve(bytes)?;
            let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("bounded reusable upload"),
                size: bytes,
                usage: wgpu::BufferUsages::MAP_WRITE | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: true,
            });
            self.slot = Some(Slot {
                allocation: Arc::new(Allocation {
                    buffer,
                    mapped: AtomicU8::new(1),
                    _reservation: reservation,
                }),
                bytes,
                state: State::Mapped,
            });
        }
        let allocation_ms = started.elapsed().as_secs_f64() * 1000.;
        let started = Instant::now();
        let slot = self.slot.as_mut().expect("allocated staging");
        {
            let mut mapped = slot
                .allocation
                .buffer
                .slice(0..bytes)
                .get_mapped_range_mut()
                .map_err(|e| error(&format!("staging mapped view: {e}")))?;
            mapped.copy_from_slice(pixels);
        }
        slot.allocation.buffer.unmap();
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("bounded upload copy"),
        });
        encoder.copy_buffer_to_buffer(&slot.allocation.buffer, 0, destination, 0, bytes);
        let command = encoder.finish();
        let lease = slot.allocation.clone();
        command.on_submitted_work_done(move || drop(lease));
        slot.state = State::Submitted(queue.submit([command]));
        self.budget.copies.fetch_add(1, Ordering::Relaxed);
        Ok(UploadTiming {
            wait_ms,
            allocation_ms,
            copy_submit_ms: started.elapsed().as_secs_f64() * 1000.,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shared_completion_lease_prevents_early_refund_after_clear() {
        let budget = Arc::new(Budget::default());
        let cache = Arc::new(budget.reserve(40 * 1024 * 1024).unwrap());
        let completion = cache.clone();
        assert_eq!(budget.usage().bytes, 40 * 1024 * 1024);
        assert_eq!(budget.usage().allocations, 1);
        drop(cache);
        assert!(budget.reserve(MAX_STAGING_BYTES).is_err());
        assert_eq!(budget.usage().bytes, 40 * 1024 * 1024);
        drop(completion);
        let larger = budget.reserve(MAX_STAGING_BYTES).unwrap();
        assert_eq!(budget.usage().peak_bytes, MAX_STAGING_BYTES);
        drop(larger);
        assert_eq!(budget.usage().bytes, 0);
    }
    #[test]
    fn invalid_and_over_limit_reservations_do_not_change_usage() {
        let budget = Arc::new(Budget::default());
        for bytes in [0, 3, MAX_STAGING_BYTES + 4] {
            assert!(budget.reserve(bytes).is_err());
        }
        assert_eq!(budget.usage().bytes, 0);
        assert_eq!(budget.usage().allocations, 0);
        let full = budget.reserve(MAX_STAGING_BYTES).unwrap();
        assert!(budget.reserve(4).is_err());
        assert_eq!(budget.usage().bytes, MAX_STAGING_BYTES);
        drop(full);
        assert_eq!(budget.usage().bytes, 0);
    }
}
