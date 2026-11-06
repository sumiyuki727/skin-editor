use super::*;
use crate::coordinator::{PayloadLease, PoolState};
use crate::{AllocationKind, ResourcePool};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RunPreparationProgress {
    Pending { initialized: usize, total: usize },
    Complete,
}

/// Explicit stepped initialization. Ready does not imply a completed game frame.
pub struct RunPreparation {
    run: Option<Box<SyntheticRun>>,
    // The boxed value and buffers die before its payload accounting.
    payload: Option<PayloadLease>,
    pool: Option<Rc<PoolState>>,
    allocated: bool,
    ready: bool,
    error: Option<ValueError>,
}
impl RunPreparation {
    pub fn new(
        source: RunSource,
        map: SyntheticMap,
        start: MapTime,
        limits: Limits,
    ) -> Result<Self, ValueError> {
        Self::new_driven(source, map, start, limits, InputDriver::Manual)
    }
    pub fn new_driven(
        source: RunSource,
        map: SyntheticMap,
        start: MapTime,
        limits: Limits,
        driver: InputDriver,
    ) -> Result<Self, ValueError> {
        SyntheticRun::validate_map(
            source,
            &map,
            start,
            limits,
            driver,
            crate::MAX_REFERENCE_CIRCLES,
        )?;
        if limits.storage_bytes < SyntheticRun::standalone_bytes_for_objects(map.len)? {
            return Err(ValueError::StorageLimit);
        }
        let mut allocation_counts = WorkCounts::default();
        allocation_counts.allocation(core::mem::size_of::<SyntheticMap>());
        let map = Rc::new(map);
        let mut result = Self::managed(
            source,
            map,
            start,
            limits,
            driver,
            None,
            &mut allocation_counts,
        )
        .map_err(crate::map::preparation::value_error)?;
        result
            .run
            .as_mut()
            .expect("new run")
            .count(allocation_counts);
        Ok(result)
    }
    pub(crate) fn managed(
        source: RunSource,
        map: Rc<SyntheticMap>,
        start: MapTime,
        limits: Limits,
        driver: InputDriver,
        pool: Option<Rc<PoolState>>,
        allocation_counts: &mut WorkCounts,
    ) -> Result<Self, crate::PreparationError> {
        let mut run = SyntheticRun::uninitialized_shared(source, map, start, limits, driver)?;
        run.pool = pool.clone();
        run.count(WorkCounts {
            management_steps: 1,
            ..WorkCounts::default()
        });
        // Validation above can fail before any allocation boundary is reached.
        // The owner retains this attempted-allocation receipt even on rejection.
        allocation_counts.allocation(core::mem::size_of::<SyntheticRun>());
        if let Some(pool) = &pool {
            pool.check_allocation(AllocationKind::Runtime)?;
        }
        let run = Box::new(run);
        let payload = pool.as_ref().map(|pool| {
            PayloadLease::new(
                pool,
                ResourcePool::Runtime,
                core::mem::size_of::<SyntheticRun>(),
            )
        });
        Ok(Self {
            run: Some(run),
            payload,
            pool,
            allocated: false,
            ready: false,
            error: None,
        })
    }
    pub fn work_counts(&self) -> WorkCounts {
        self.run
            .as_ref()
            .expect("owned initialization")
            .work_counts()
    }
    pub fn step(&mut self, budget: WorkBudget) -> Result<RunPreparationProgress, ValueError> {
        if let Some(error) = self.error {
            return Err(error);
        }
        let run = self.run.as_mut().expect("owned initialization");
        if budget.get() > run.limits.max_work {
            return Err(ValueError::InvalidBudget);
        }
        for _ in 0..budget.get() {
            if self.ready {
                break;
            }
            if !self.allocated {
                let mut stats = run.stats.get();
                let outcome = run.objects.allocate(
                    &mut stats,
                    self.pool.as_ref(),
                    AllocationKind::ObjectStates,
                    ResourcePool::Runtime,
                );
                stats.merge(WorkCounts {
                    management_steps: 1,
                    ..WorkCounts::default()
                });
                run.stats.set(stats);
                if let Err(error) = outcome {
                    let error = crate::map::preparation::value_error(error);
                    self.error = Some(error);
                    return Err(error);
                }
                self.allocated = true;
            } else if run.objects.len() < run.map.len {
                run.objects.push(ObjectState::default());
                run.count(WorkCounts {
                    state_initializations: 1,
                    ..WorkCounts::default()
                });
            }
            self.ready = self.allocated && run.objects.len() == run.map.len;
        }
        Ok(if self.ready {
            RunPreparationProgress::Complete
        } else {
            RunPreparationProgress::Pending {
                initialized: run.objects.len(),
                total: run.map.len,
            }
        })
    }
    pub(crate) fn take_ready(&mut self) -> Option<(Box<SyntheticRun>, Option<PayloadLease>)> {
        if !self.ready {
            return None;
        }
        Some((self.run.take().expect("ready run"), self.payload.take()))
    }
    pub fn finish(mut self) -> Result<SyntheticRun, ValueError> {
        if let Some(error) = self.error {
            return Err(error);
        }
        let (run, _payload) = self.take_ready().ok_or(ValueError::PreparationIncomplete)?;
        Ok(*run)
    }
}
