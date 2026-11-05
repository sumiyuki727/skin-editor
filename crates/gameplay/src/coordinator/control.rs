use super::*;
use crate::{
    Acknowledgement, Admission, AdvanceCursor, AdvanceReport, AutomaticReport, EventKey,
    FreezeReport, InputEvent, RearmReport, ReleaseObservation, RequestKey, WorkBudget,
};

impl SyntheticCoordinator {
    pub fn select_driver(
        &mut self,
        source: RunSource,
        driver: InputDriver,
    ) -> Result<RunSource, PreparationError> {
        self.check_base(source)?;
        if self.pool.live.get().is_some() {
            return Err(PreparationError::Busy);
        }
        self.control(|run| run.select_driver(source, driver))
    }
    pub fn submit_driver_inputs(
        &mut self,
        source: RunSource,
        driver: InputDriver,
        batch: &[InputEvent],
    ) -> Result<Admission, PreparationError> {
        self.control(|run| run.submit_driver_inputs(source, driver, batch))
    }
    pub fn observe_driver_release(
        &mut self,
        source: RunSource,
        driver: InputDriver,
        fact: ReleaseObservation,
    ) -> Result<RearmReport, PreparationError> {
        self.control(|run| run.observe_driver_release(source, driver, fact))
    }
    pub fn prepare_automatic_inputs(
        &mut self,
        source: RunSource,
        through: MapTime,
        budget: WorkBudget,
    ) -> Result<AutomaticReport, PreparationError> {
        self.control(|run| run.prepare_automatic_inputs(source, through, budget))
    }
    fn control<T>(
        &mut self,
        action: impl FnOnce(&mut SyntheticRun) -> Result<T, RunError>,
    ) -> Result<T, PreparationError> {
        let result = action(
            &mut self
                .active
                .as_mut()
                .ok_or(PreparationError::NoActiveRun)?
                .run,
        )
        .map_err(PreparationError::Runtime);
        let base = self.current().map(|v| (v.source(), v.observation().status));
        self.pool.base.set(base);
        if self.pool.live.get().is_some_and(|key| key.base != base) {
            self.pool.live.set(None);
        }
        result
    }
    pub fn submit_inputs(
        &mut self,
        source: RunSource,
        batch: &[InputEvent],
    ) -> Result<Admission, PreparationError> {
        self.control(|run| run.submit_inputs(source, batch))
    }
    pub fn observe_release(
        &mut self,
        source: RunSource,
        fact: ReleaseObservation,
    ) -> Result<RearmReport, PreparationError> {
        self.control(|run| run.observe_release(source, fact))
    }
    pub fn seal_through(&mut self, source: RunSource, at: MapTime) -> Result<(), PreparationError> {
        self.control(|run| run.seal_through(source, at))
    }
    pub fn advance(
        &mut self,
        source: RunSource,
        at: MapTime,
        budget: WorkBudget,
    ) -> Result<AdvanceReport, PreparationError> {
        self.control(|run| run.advance(source, at, budget))
    }
    pub fn continue_advance(
        &mut self,
        cursor: AdvanceCursor,
        budget: WorkBudget,
    ) -> Result<AdvanceReport, PreparationError> {
        self.control(|run| run.continue_advance(cursor, budget))
    }
    pub fn cancel_advance(&mut self, key: RequestKey) -> Result<(), PreparationError> {
        self.control(|run| run.cancel_request(key))
    }
    pub fn acknowledge(&mut self, key: EventKey) -> Result<Acknowledgement, PreparationError> {
        self.control(|run| run.acknowledge(key))
    }
    pub fn request_freeze(
        &mut self,
        source: RunSource,
        at: MapTime,
    ) -> Result<(), PreparationError> {
        self.control(|run| run.request_freeze(source, at))
    }
    pub fn settle_freeze(
        &mut self,
        source: RunSource,
        budget: WorkBudget,
    ) -> Result<FreezeReport, PreparationError> {
        self.control(|run| run.settle_freeze(source, budget))
    }
    pub fn resume(&mut self, source: RunSource) -> Result<RunSource, PreparationError> {
        self.check_base(source)?;
        if self.pool.live.get().is_some() {
            return Err(PreparationError::Busy);
        }
        self.control(|run| run.resume(source))
    }
}
