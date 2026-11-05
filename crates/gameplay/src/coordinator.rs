//! Caller-stepped, bounded coordination; no project writer or native adapter.
use crate::{
    Circle, CoordinatorId, Limits, MAX_CIRCLES, MapPreparation, MapTime, RunError, RunSource,
    RunStatus, SyntheticMap, SyntheticRun, ValueError,
};
use core::cell::Cell;
use std::rc::Rc;

mod budget;
mod control;
mod task;
#[cfg(test)]
mod tests;
use budget::{PoolState, Reservation};
pub use budget::{PreparationCosts, PreparationLimits, ResourcePool, ResourceUsage};
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
    Budget {
        pool: ResourcePool,
        requested: usize,
        used: usize,
        limit: usize,
    },
}
impl From<ValueError> for PreparationError {
    fn from(value: ValueError) -> Self {
        Self::Invalid(value)
    }
}
impl From<RunError> for PreparationError {
    fn from(value: RunError) -> Self {
        Self::Runtime(value)
    }
}

struct CompiledSource {
    map: SyntheticMap,
    // The last shared content owner releases this reservation.
    _credit: Reservation,
}
struct ManagedRun {
    // Drop owned runtime/content before returning their reservations.
    run: Box<SyntheticRun>,
    content: Rc<CompiledSource>,
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
            live: Cell::new(None),
            base: Cell::new(None),
            alive: Cell::new(true),
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
    pub const fn costs() -> PreparationCosts {
        PreparationCosts {
            coordinator_bytes: core::mem::size_of::<Self>(),
            pool_bytes: core::mem::size_of::<PoolState>(),
            // The inline initializer coexists briefly with its boxed runtime.
            // Keep this known scratch reserved until the task owner is dropped.
            task_bytes: core::mem::size_of::<PreparationTask<'static>>()
                + core::mem::size_of::<SyntheticRun>(),
            run_bytes: SyntheticRun::static_storage_bytes(),
            source_bytes: core::mem::size_of::<CompiledSource>(),
        }
    }
    pub fn current(&self) -> Option<&SyntheticRun> {
        self.active.as_ref().map(|v| v.run.as_ref())
    }
    pub fn resources(&self) -> ResourceUsage {
        self.pool.usage.get()
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
        if input.len() > MAX_CIRCLES {
            return Err(ValueError::ObjectLimit.into());
        }
        let key = self.next_key(operation, source)?;
        let credits = PoolState::reserve_task(&self.pool, true)?;
        let preparation = MapPreparation::new(source.revision, input)?;
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
        let credits = PoolState::reserve_task(&self.pool, false)?;
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
