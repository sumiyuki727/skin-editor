use super::*;

/// Managed allocation boundary. A diagnostic failure is not an OS OOM recovery guarantee.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AllocationKind {
    SourceMap,
    SourceOwner,
    Runtime,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResourcePool {
    Precompiled,
    Runtime,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Reserved managed capacity, including planned payloads and known scratch.
/// Allocation headers and the complete native call stack are outside this sum.
pub struct ResourceUsage {
    pub precompiled_bytes: usize,
    pub runtime_bytes: usize,
}
/// Quantified payload/owned-workspace reservations, not allocator overhead or
/// process memory. Shared source payloads are charged once per allocation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PreparationCosts {
    pub coordinator_bytes: usize,
    pub pool_bytes: usize,
    pub task_bytes: usize,
    pub run_bytes: usize,
    pub source_bytes: usize,
    pub source_map_bytes: usize,
    pub source_owner_bytes: usize,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PreparationLimits {
    precompiled_bytes: usize,
    runtime_bytes: usize,
    pub(super) max_work: u16,
}
impl PreparationLimits {
    pub const fn tiny() -> Self {
        Self {
            precompiled_bytes: 64 * 1024,
            runtime_bytes: 64 * 1024,
            max_work: 1,
        }
    }
    pub const fn reference() -> Self {
        Self {
            precompiled_bytes: 8 * 1024 * 1024,
            runtime_bytes: 8 * 1024 * 1024,
            max_work: 256,
        }
    }
    pub fn try_new(
        precompiled_bytes: usize,
        runtime_bytes: usize,
        max_work: u16,
    ) -> Result<Self, ValueError> {
        let limits = Self {
            precompiled_bytes,
            runtime_bytes,
            max_work,
        };
        limits.validate()?;
        Ok(limits)
    }
    pub(super) fn validate(self) -> Result<(), ValueError> {
        if self.max_work == 0 || self.max_work > 256 {
            return Err(ValueError::InvalidLimits);
        }
        let cost = SyntheticCoordinator::costs();
        if self.precompiled_bytes < cost.source_bytes
            || self.runtime_bytes
                < cost.pool_bytes + cost.coordinator_bytes + cost.task_bytes + cost.run_bytes
        {
            return Err(ValueError::StorageLimit);
        }
        Ok(())
    }
}
pub(super) struct PoolState {
    pub(super) limits: PreparationLimits,
    pub(super) usage: Cell<ResourceUsage>,
    pub(super) live: Cell<Option<PreparationKey>>,
    pub(super) base: Cell<Option<(RunSource, RunStatus)>>,
    pub(super) alive: Cell<bool>,
    #[cfg(test)]
    pub(super) allocation_failure: Cell<Option<AllocationKind>>,
}
pub(super) struct Reservation {
    pool: Rc<PoolState>,
    kind: ResourcePool,
    bytes: usize,
}
impl PoolState {
    pub(super) fn check_allocation(&self, kind: AllocationKind) -> Result<(), PreparationError> {
        #[cfg(test)]
        if self.allocation_failure.get() == Some(kind) {
            self.allocation_failure.set(None);
            return Err(PreparationError::AllocationUnavailable(kind));
        }
        let _ = kind;
        Ok(())
    }

    pub(super) fn reserve_owner(pool: &Rc<Self>) -> Reservation {
        let bytes = SyntheticCoordinator::costs().coordinator_bytes;
        let mut usage = pool.usage.get();
        usage.runtime_bytes += bytes;
        pool.usage.set(usage);
        Reservation {
            pool: Rc::clone(pool),
            kind: ResourcePool::Runtime,
            bytes,
        }
    }
    pub(super) fn reserve_task(
        pool: &Rc<Self>,
        content: bool,
    ) -> Result<(Reservation, Reservation, Option<Reservation>), PreparationError> {
        let costs = SyntheticCoordinator::costs();
        let source_bytes = if content { costs.source_bytes } else { 0 };
        let runtime_bytes = costs.task_bytes + costs.run_bytes;
        let used = pool.usage.get();
        let check = |kind, amount, used: usize, limit| {
            used.checked_add(amount)
                .filter(|total| *total <= limit)
                .ok_or(PreparationError::Budget {
                    pool: kind,
                    requested: amount,
                    used,
                    limit,
                })
        };
        // Both checks precede any counter, generation or allocation change.
        let next_content = check(
            ResourcePool::Precompiled,
            source_bytes,
            used.precompiled_bytes,
            pool.limits.precompiled_bytes,
        )?;
        let next_runtime = check(
            ResourcePool::Runtime,
            runtime_bytes,
            used.runtime_bytes,
            pool.limits.runtime_bytes,
        )?;
        pool.usage.set(ResourceUsage {
            precompiled_bytes: next_content,
            runtime_bytes: next_runtime,
        });
        let credit = |kind, bytes| Reservation {
            pool: Rc::clone(pool),
            kind,
            bytes,
        };
        Ok((
            credit(ResourcePool::Runtime, costs.task_bytes),
            credit(ResourcePool::Runtime, costs.run_bytes),
            content.then(|| credit(ResourcePool::Precompiled, costs.source_bytes)),
        ))
    }
}
impl Drop for Reservation {
    fn drop(&mut self) {
        let mut used = self.pool.usage.get();
        let current = match self.kind {
            ResourcePool::Precompiled => &mut used.precompiled_bytes,
            ResourcePool::Runtime => &mut used.runtime_bytes,
        };
        *current = current.checked_sub(self.bytes).expect("owned reservation");
        self.pool.usage.set(used);
    }
}
