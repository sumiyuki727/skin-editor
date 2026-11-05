use super::*;
use crate::{
    FreezeReport, InspectionCursor, InspectionKey, InspectionReport, PreparationProgress,
    WorkBudget,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PreparationStage {
    Validating,
    Initializing,
    Rebuilding,
    Ready,
    Activated,
    Failed,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PreparationCursor {
    key: PreparationKey,
    progress: u64,
}
impl PreparationCursor {
    pub const fn key(self) -> PreparationKey {
        self.key
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PreparationReport {
    Pending {
        cursor: PreparationCursor,
        stage: PreparationStage,
        steps: u16,
    },
    Ready {
        key: PreparationKey,
        source: RunSource,
        steps: u16,
    },
}
#[derive(Clone, Copy)]
enum Work {
    Validate,
    Initialize,
    FreezeStart,
    BeginInspection,
    Inspect(InspectionCursor),
    FinishInspection(InspectionKey),
    Ready,
    Activated,
    Failed,
}

/// Non-cloneable work/result owner. Cancellation revokes eligibility, while
/// storage remains reserved until this owner actually releases its values.
pub struct PreparationTask<'a> {
    preparation: Option<MapPreparation<'a>>,
    pub(super) run: Option<ManagedRun>,
    content: Option<Rc<CompiledSource>>,
    source_credit: Option<Reservation>,
    run_credit: Option<Reservation>,
    _workspace: Reservation,
    pub(super) pool: Rc<PoolState>,
    pub(super) key: PreparationKey,
    limits: Limits,
    progress: u64,
    work: Work,
}
impl<'a> PreparationTask<'a> {
    #[cfg(test)]
    pub(super) fn set_exhausted_progress(&mut self) {
        self.progress = u64::MAX;
    }
    pub(super) fn new(
        key: PreparationKey,
        pool: Rc<PoolState>,
        limits: Limits,
        preparation: Option<MapPreparation<'a>>,
        content: Option<Rc<CompiledSource>>,
        credits: (Reservation, Reservation, Option<Reservation>),
    ) -> Self {
        let (workspace, run_credit, source_credit) = credits;
        let work = if preparation.is_some() {
            Work::Validate
        } else {
            Work::Initialize
        };
        Self {
            preparation,
            run: None,
            content,
            source_credit,
            run_credit: Some(run_credit),
            _workspace: workspace,
            pool,
            key,
            limits,
            progress: 0,
            work,
        }
    }
    pub const fn key(&self) -> PreparationKey {
        self.key
    }
    pub const fn cursor(&self) -> PreparationCursor {
        PreparationCursor {
            key: self.key,
            progress: self.progress,
        }
    }
    pub fn resources(&self) -> ResourceUsage {
        self.pool.usage.get()
    }
    pub fn stage(&self) -> PreparationStage {
        match self.work {
            Work::Validate => PreparationStage::Validating,
            Work::Initialize => PreparationStage::Initializing,
            Work::Ready => PreparationStage::Ready,
            Work::Activated => PreparationStage::Activated,
            Work::Failed => PreparationStage::Failed,
            _ => PreparationStage::Rebuilding,
        }
    }
    pub(super) fn activated(&mut self) {
        self.work = Work::Activated;
    }
    pub(super) fn check_live(&self) -> Result<(), PreparationError> {
        if !self.pool.alive.get()
            || self.pool.live.get() != Some(self.key)
            || self.pool.base.get() != self.key.base
        {
            return Err(PreparationError::StaleRequest);
        }
        Ok(())
    }
    pub fn step(
        &mut self,
        cursor: PreparationCursor,
        budget: WorkBudget,
    ) -> Result<PreparationReport, PreparationError> {
        self.check_live()?;
        if cursor != self.cursor()
            || matches!(self.work, Work::Ready | Work::Activated | Work::Failed)
        {
            return Err(PreparationError::StaleCursor);
        }
        if budget.get() > self.pool.limits.max_work {
            return Err(PreparationError::BudgetLimit);
        }
        let mut steps = 0;
        while steps < budget.get() && !matches!(self.work, Work::Ready) {
            let outcome = self
                .progress
                .checked_add(1)
                .ok_or(PreparationError::CounterExhausted)
                .and_then(|next| {
                    self.step_one()?;
                    Ok(next)
                });
            match outcome {
                Ok(next) => {
                    self.progress = next;
                    steps += 1;
                }
                Err(error) => {
                    self.invalidate();
                    self.work = Work::Failed;
                    return Err(error);
                }
            }
        }
        Ok(if matches!(self.work, Work::Ready) {
            PreparationReport::Ready {
                key: self.key,
                source: self.key.target,
                steps,
            }
        } else {
            PreparationReport::Pending {
                cursor: self.cursor(),
                stage: self.stage(),
                steps,
            }
        })
    }
    fn step_one(&mut self) -> Result<(), PreparationError> {
        match self.work {
            Work::Validate => {
                if matches!(
                    self.preparation
                        .as_mut()
                        .expect("map preparation")
                        .step(WorkBudget::one())?,
                    PreparationProgress::Complete { .. }
                ) {
                    let map = self.preparation.take().expect("complete map").finish()?;
                    self.content = Some(Rc::new(CompiledSource {
                        map,
                        _credit: self.source_credit.take().expect("reserved source"),
                    }));
                    self.work = Work::Initialize;
                }
            }
            Work::Initialize => {
                let content = self.content.as_ref().expect("validated source");
                let (source, start) = match self.key.operation {
                    PreparationOperation::Create { start }
                    | PreparationOperation::Load { start } => (self.key.target, start),
                    PreparationOperation::Inspect { .. } => (
                        self.key.base.expect("inspection base").0,
                        content
                            .map
                            .circle(0)
                            .map_or(MapTime::from_micros(0), |v| v.visible_from()),
                    ),
                };
                let mut run = SyntheticRun::new_driven(
                    source,
                    content.map,
                    start,
                    self.limits,
                    self.key.driver,
                )?;
                if matches!(self.key.operation, PreparationOperation::Inspect { .. }) {
                    // The private start section has no accepted inputs/deadlines.
                    run.request_freeze(source, start)?;
                    run.seal_through(source, start)?;
                    self.work = Work::FreezeStart;
                } else {
                    self.work = Work::Ready;
                }
                self.run = Some(ManagedRun {
                    run: Box::new(run),
                    content: self.content.take().expect("validated source"),
                    _credit: self.run_credit.take().expect("reserved runtime"),
                });
            }
            Work::FreezeStart => {
                let run = &mut self.run.as_mut().expect("candidate run").run;
                if matches!(
                    run.settle_freeze(run.source(), WorkBudget::one())?,
                    FreezeReport::Frozen { .. }
                ) {
                    self.work = Work::BeginInspection;
                }
            }
            Work::BeginInspection => {
                let PreparationOperation::Inspect { at } = self.key.operation else {
                    unreachable!("inspection operation");
                };
                let run = &mut self.run.as_mut().expect("candidate run").run;
                self.work = Work::Inspect(run.request_inspection(run.source(), at)?);
            }
            Work::Inspect(cursor) => {
                let run = &mut self.run.as_mut().expect("candidate run").run;
                self.work = match run.step_inspection(cursor, WorkBudget::one())? {
                    InspectionReport::Pending { cursor, .. } => Work::Inspect(cursor),
                    InspectionReport::Ready { key, .. } => Work::FinishInspection(key),
                };
            }
            Work::FinishInspection(key) => {
                let source = self
                    .run
                    .as_mut()
                    .expect("candidate run")
                    .run
                    .activate_inspection(key)?;
                debug_assert_eq!(source, self.key.target);
                self.work = Work::Ready;
            }
            Work::Ready | Work::Activated | Work::Failed => unreachable!("checked step state"),
        }
        Ok(())
    }
    fn invalidate(&self) {
        if self.pool.live.get() == Some(self.key) {
            self.pool.live.set(None);
        }
    }
}
impl Drop for PreparationTask<'_> {
    fn drop(&mut self) {
        self.invalidate();
    }
}
