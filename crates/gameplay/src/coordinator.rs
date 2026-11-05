//! Caller-stepped, bounded coordination; no project writer or native adapter.
use crate::{
    Circle, CoordinatorId, InputDriver, Limits, MapPreparation, MapTime, RunError, RunSource,
    RunStatus, SyntheticMap, SyntheticRun, ValueError,
};
use core::cell::Cell;
use std::rc::Rc;

mod budget;
mod control;
mod task;
#[cfg(test)]
mod tests;
use budget::Reservation;
pub use budget::{
    AllocationKind, AllocationUsage, PreparationCosts, PreparationLimits, ResourcePool,
    ResourceUsage,
};
pub(crate) use budget::{PayloadLease, PoolState};
pub use task::{PreparationCursor, PreparationReport, PreparationStage, PreparationTask};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PreparationOperation {
    Create { start: MapTime },
    Load { start: MapTime },
    Inspect { at: MapTime },
}

/// Complete request identity, distinct from map/advance/inspection cursors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PreparationKey {
    owner: CoordinatorId,
    generation: u64,
    base: Option<(RunSource, RunStatus)>,
    operation: PreparationOperation,
    target: RunSource,
    driver: InputDriver,
}
impl PreparationKey {
    pub const fn owner(self) -> CoordinatorId {
        self.owner
    }
    pub const fn base(self) -> Option<RunSource> {
        match self.base {
            Some((source, _)) => Some(source),
            None => None,
        }
    }
    pub const fn operation(self) -> PreparationOperation {
        self.operation
    }
    pub const fn target(self) -> RunSource {
        self.target
    }
    pub const fn driver(self) -> InputDriver {
        self.driver
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PreparationError {
    Invalid(ValueError),
    Runtime(RunError),
    Busy,
    StaleRequest,
    StaleCursor,
    NotReady,
    NoActiveRun,
    InvalidOperation,
    RunIdReuse,
    CounterExhausted,
    BudgetLimit,
    AllocationUnavailable(AllocationKind),
    Budget {
        pool: ResourcePool,
        requested: usize,
        used: usize,
        limit: usize,
    },
}
impl From<ValueError> for PreparationError {
    fn from(value: ValueError) -> Self {
        if let ValueError::AllocationUnavailable(kind) = value {
            Self::AllocationUnavailable(kind)
        } else {
            Self::Invalid(value)
        }
    }
}
impl From<RunError> for PreparationError {
    fn from(value: RunError) -> Self {
        Self::Runtime(value)
    }
}

struct CompiledSource {
    map: Rc<SyntheticMap>,
    _payload: PayloadLease,
    // The last shared content owner releases this reservation.
    _credit: Reservation,
}
struct ManagedRun {
    // Drop owned runtime/content before returning their reservations.
    run: Box<SyntheticRun>,
    content: Rc<CompiledSource>,
    _payload: PayloadLease,
    _credit: Reservation,
}

/// Owns its active run. `current` is deliberately read-only:
/// ```compile_fail
/// use skin_editor_gameplay::SyntheticCoordinator;
/// fn bypass(owner: &mut SyntheticCoordinator) {
///     let run = owner.current().unwrap();
///     run.resume(run.source()).unwrap();
/// }
/// ```
pub struct SyntheticCoordinator {
    active: Option<ManagedRun>,
    _credit: Reservation,
    pool: Rc<PoolState>,
    id: CoordinatorId,
    next_request: Option<u64>,
    run_limits: Limits,
}
impl SyntheticCoordinator {
    pub fn new(
        id: CoordinatorId,
        run_limits: Limits,
        limits: PreparationLimits,
    ) -> Result<Self, PreparationError> {
        run_limits.validate()?;
        limits.validate()?;
        let pool = Rc::new(PoolState {
            limits,
            usage: Cell::new(ResourceUsage {
                precompiled_bytes: 0,
                runtime_bytes: Self::costs().pool_bytes,
            }),
            allocated: Cell::new(AllocationUsage {
                runtime_bytes: Self::costs().pool_bytes,
                runtime_allocations: 1,
                ..AllocationUsage::default()
            }),
            live: Cell::new(None),
            base: Cell::new(None),
            alive: Cell::new(true),
            #[cfg(test)]
            allocation_failure: Cell::new(None),
        });
        let credit = PoolState::reserve_owner(&pool);
        Ok(Self {
            active: None,
            _credit: credit,
            pool,
            id,
            next_request: Some(1),
            run_limits,
        })
    }
    /// Four-slot minimum, retained for small fixtures. Variable runs must use
    /// costs_for_objects before either pool is reserved.
    pub fn costs() -> PreparationCosts {
        Self::costs_for_objects(crate::MAX_CIRCLES).expect("four-slot layout")
    }
    pub fn costs_for_objects(objects: usize) -> Result<PreparationCosts, ValueError> {
        let capacity = crate::storage::capacity(objects)?;
        let circles = crate::storage::bytes::<Circle>(capacity)?;
        let indices = crate::storage::bytes::<crate::map::IdKey>(capacity)?
            .checked_mul(2)
            .ok_or(ValueError::StorageLimit)?;
        let states = crate::storage::bytes::<crate::run::ObjectState>(capacity)?;
        let source_map_bytes = core::mem::size_of::<SyntheticMap>()
            .checked_add(circles)
            .ok_or(ValueError::StorageLimit)?;
        let source_owner_bytes = core::mem::size_of::<CompiledSource>();
        let source_bytes = source_map_bytes
            .checked_add(source_owner_bytes)
            .ok_or(ValueError::StorageLimit)?;
        let task_bytes = core::mem::size_of::<PreparationTask<'static>>()
            .checked_add(core::mem::size_of::<SyntheticRun>())
            .and_then(|v| v.checked_add(core::mem::size_of::<SyntheticMap>()))
            .and_then(|v| v.checked_add(source_owner_bytes))
            .and_then(|v| v.checked_add(indices))
            .ok_or(ValueError::StorageLimit)?;
        Ok(PreparationCosts {
            coordinator_bytes: core::mem::size_of::<Self>(),
            pool_bytes: core::mem::size_of::<PoolState>(),
            // The inline initializer coexists briefly with its boxed runtime.
            // Keep this known scratch reserved until the task owner is dropped.
            task_bytes,
            run_bytes: SyntheticRun::storage_bytes_for_objects(objects)?,
            source_bytes,
            source_map_bytes,
            source_owner_bytes,
            object_capacity: capacity,
            index_bytes: indices,
            state_bytes: states,
        })
    }
    pub fn current(&self) -> Option<&SyntheticRun> {
        self.active.as_ref().map(|v| v.run.as_ref())
    }
    pub fn resources(&self) -> ResourceUsage {
        self.pool.usage.get()
    }
    pub fn allocations(&self) -> AllocationUsage {
        self.pool.allocated.get()
    }
    pub fn candidate(&self) -> Option<PreparationKey> {
        self.pool.live.get()
    }

    pub fn request_create<'a>(
        &mut self,
        source: RunSource,
        input: &'a [Circle],
        start: MapTime,
    ) -> Result<PreparationTask<'a>, PreparationError> {
        if self.active.is_some() {
            return Err(PreparationError::InvalidOperation);
        }
        self.request_map(source, input, PreparationOperation::Create { start })
    }
    /// A pure precompiled replacement, not a file/beatmap loader. The old run
    /// must be frozen or already failed; failure never publishes a replacement.
    pub fn request_load<'a>(
        &mut self,
        base: RunSource,
        source: RunSource,
        input: &'a [Circle],
        start: MapTime,
    ) -> Result<PreparationTask<'a>, PreparationError> {
        self.check_base(base)?;
        match self.current().expect("checked base").observation().status {
            RunStatus::Frozen | RunStatus::Failed(_) => {}
            _ => return Err(RunError::NotFrozen.into()),
        }
        if source.run == base.run {
            return Err(PreparationError::RunIdReuse);
        }
        self.request_map(source, input, PreparationOperation::Load { start })
    }
    fn request_map<'a>(
        &mut self,
        source: RunSource,
        input: &'a [Circle],
        operation: PreparationOperation,
    ) -> Result<PreparationTask<'a>, PreparationError> {
        if input.len() > self.run_limits.object_capacity() {
            return Err(ValueError::ObjectLimit.into());
        }
        let key = self.next_key(operation, source)?;
        let costs = Self::costs_for_objects(input.len())?;
        if self.run_limits.storage_bytes() < costs.run_bytes {
            return Err(ValueError::StorageLimit.into());
        }
        let credits = PoolState::reserve_task(&self.pool, true, costs)?;
        let preparation = MapPreparation::managed(
            source.revision,
            input,
            self.run_limits,
            Rc::clone(&self.pool),
        )?;
        let task = PreparationTask::new(
            key,
            Rc::clone(&self.pool),
            self.run_limits,
            Some(preparation),
            None,
            credits,
        );
        self.publish_request(key);
        Ok(task)
    }
    pub fn request_inspection(
        &mut self,
        base: RunSource,
        at: MapTime,
    ) -> Result<PreparationTask<'static>, PreparationError> {
        self.check_base(base)?;
        match self.current().expect("checked base").observation().status {
            RunStatus::Frozen => {}
            RunStatus::Failed(_) => return Err(RunError::Faulted.into()),
            _ => return Err(RunError::NotFrozen.into()),
        }
        let target = RunSource {
            transport: base
                .transport
                .checked_next()
                .ok_or(PreparationError::CounterExhausted)?,
            input: base
                .input
                .checked_next()
                .ok_or(PreparationError::CounterExhausted)?,
            ..base
        };
        let key = self.next_key(PreparationOperation::Inspect { at }, target)?;
        let costs =
            Self::costs_for_objects(self.current().expect("base").observation().object_count)?;
        let credits = PoolState::reserve_task(&self.pool, false, costs)?;
        let content = Rc::clone(&self.active.as_ref().expect("checked base").content);
        let task = PreparationTask::new(
            key,
            Rc::clone(&self.pool),
            self.run_limits,
            None,
            Some(content),
            credits,
        );
        self.publish_request(key);
        Ok(task)
    }
    fn next_key(
        &self,
        operation: PreparationOperation,
        target: RunSource,
    ) -> Result<PreparationKey, PreparationError> {
        if self.pool.live.get().is_some() {
            return Err(PreparationError::Busy);
        }
        Ok(PreparationKey {
            owner: self.id,
            generation: self
                .next_request
                .ok_or(PreparationError::CounterExhausted)?,
            base: self.pool.base.get(),
            operation,
            target,
            driver: match operation {
                PreparationOperation::Inspect { .. } => {
                    self.current().expect("inspection base").driver()
                }
                _ => InputDriver::Manual,
            },
        })
    }
    fn publish_request(&mut self, key: PreparationKey) {
        self.next_request = key.generation.checked_add(1);
        self.pool.live.set(Some(key));
    }
    fn check_base(&self, source: RunSource) -> Result<(), PreparationError> {
        let run = self.current().ok_or(PreparationError::NoActiveRun)?;
        if run.source() != source {
            return Err(RunError::StaleSource.into());
        }
        Ok(())
    }
    fn check_key(&self, key: PreparationKey) -> Result<(), PreparationError> {
        if key.owner != self.id
            || self.pool.live.get() != Some(key)
            || self.pool.base.get() != key.base
        {
            return Err(PreparationError::StaleRequest);
        }
        Ok(())
    }
    pub fn cancel(&mut self, key: PreparationKey) -> Result<(), PreparationError> {
        self.check_key(key)?;
        self.pool.live.set(None);
        Ok(())
    }
    /// Incomplete/foreign results leave both the run and eligible task intact.
    /// After success the task still owns its workspace until it is dropped.
    pub fn activate(
        &mut self,
        key: PreparationKey,
        task: &mut PreparationTask<'_>,
    ) -> Result<RunSource, PreparationError> {
        self.check_key(key)?;
        if !Rc::ptr_eq(&self.pool, &task.pool) || task.key != key {
            return Err(PreparationError::StaleRequest);
        }
        task.check_live()?;
        if task.stage() != PreparationStage::Ready {
            return Err(PreparationError::NotReady);
        }
        let ready = task.run.take().expect("complete runtime");
        debug_assert_eq!(ready.run.source(), key.target);
        let old = self.active.replace(ready);
        self.pool
            .base
            .set(self.current().map(|v| (v.source(), v.observation().status)));
        self.pool.live.set(None);
        task.activated();
        drop(old);
        Ok(key.target)
    }
}
impl Drop for SyntheticCoordinator {
    fn drop(&mut self) {
        self.pool.alive.set(false);
        self.pool.live.set(None);
    }
}
