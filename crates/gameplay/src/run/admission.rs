use super::*;

impl SyntheticRun {
    /// Serial core admission only. The host must complete capture handoff before
    /// sealing; this method cannot infer unhanded platform events.
    pub fn submit_inputs(
        &mut self,
        source: RunSource,
        batch: &[InputEvent],
    ) -> Result<Admission, RunError> {
        self.submit_driver_inputs(source, InputDriver::Manual, batch)
    }
    /// External manual/regression input; automatic pulses have a private producer.
    pub fn submit_driver_inputs(
        &mut self,
        source: RunSource,
        driver: InputDriver,
        batch: &[InputEvent],
    ) -> Result<Admission, RunError> {
        self.check_external_driver(source, driver)?;
        self.admit_inputs(source, batch)
    }
    pub(super) fn admit_inputs(
        &mut self,
        source: RunSource,
        batch: &[InputEvent],
    ) -> Result<Admission, RunError> {
        self.check_source(source)?;
        if batch.len() > self.limits.inputs {
            if self.mode != ControlMode::Running || self.failure.is_some() {
                return Err(RunError::GateClosed);
            }
            return Err(self.fail(RunError::InputCapacity));
        }
        let mut added = [None; MAX_INPUTS];
        let mut new_len = 0;
        let mut repeated = 0;
        for input in batch.iter().copied() {
            if let Some(old) = self.replay[..self.limits.replay]
                .iter()
                .flatten()
                .find(|old| old.sequence() == input.sequence)
            {
                if *old != InputRecord::Gameplay(input) {
                    return Err(self.fail(RunError::InputConflict));
                }
                repeated += 1;
                continue;
            }
            if let Some(old) = added[..new_len]
                .iter()
                .flatten()
                .find(|old: &&InputEvent| old.sequence == input.sequence)
            {
                if *old != input {
                    return Err(self.fail(RunError::InputConflict));
                }
                repeated += 1;
                continue;
            }
            if input.sequence.get() <= self.highest_input {
                return Err(RunError::OldInputSequence);
            }
            if self.failure.is_some() || self.mode != ControlMode::Running {
                return Err(RunError::GateClosed);
            }
            added[new_len] = Some(input);
            new_len += 1;
        }

        // A bounded batch (at most 64), ordered before any queue/window write.
        for index in 0..new_len {
            let mut smallest = index;
            for next in index + 1..new_len {
                if added[next].expect("new input").sequence
                    < added[smallest].expect("new input").sequence
                {
                    smallest = next;
                }
            }
            added.swap(index, smallest);
        }
        // Admission permission is separate from the consumed held state. In a
        // batch, captured releases arm only later sequence numbers; rejection
        // cannot leave half the batch or half its arming changes behind.
        let mut armed_after = self.armed;
        let mut armed_from_after = self.armed_from;
        for input in added[..new_len].iter().flatten() {
            let slot = input.slot.index();
            // A quarantined key has no admission permission yet. Its repeat
            // must not turn a valid queued prefix into a time/capacity fault.
            if matches!(input.edge, InputEdge::Press(_))
                && (!armed_after[slot] || armed_from_after[slot].is_some_and(|at| input.at < at))
            {
                return Err(RunError::RearmRequired);
            }
            if self.sealed.is_some_and(|sealed| input.at <= sealed) {
                return Err(self.fail(RunError::LateInput));
            }
            if input.at < self.start {
                return Err(self.fail(RunError::InputBeforeStart));
            }
            if matches!(input.edge, InputEdge::Release) && !armed_after[slot] {
                armed_after[slot] = true;
                armed_from_after[slot] = Some(input.at);
            }
        }
        if self.input_len + new_len > self.limits.inputs {
            return Err(self.fail(RunError::InputCapacity));
        }
        for input in added[..new_len].iter().flatten().copied() {
            let mut at = self.input_len;
            while at > 0 {
                let previous = self.inputs[at - 1].expect("queued input");
                if (previous.at, previous.sequence) <= (input.at, input.sequence) {
                    break;
                }
                self.inputs[at] = self.inputs[at - 1];
                at -= 1;
            }
            self.inputs[at] = Some(input);
            self.input_len += 1;
            self.replay[self.replay_next] = Some(InputRecord::Gameplay(input));
            self.replay_next = (self.replay_next + 1) % self.limits.replay;
            self.highest_input = input.sequence.get();
        }
        self.armed = armed_after;
        self.armed_from = armed_from_after;
        Ok(Admission {
            added: new_len,
            repeated,
        })
    }

    pub fn seal_through(&mut self, source: RunSource, through: MapTime) -> Result<(), RunError> {
        self.check_source(source)?;
        if self.failure.is_some() {
            return Err(RunError::Faulted);
        }
        if through < self.start || self.sealed.is_some_and(|old| through < old) {
            return Err(RunError::InvalidTarget);
        }
        if self.driver == InputDriver::Automatic
            && self.mode == ControlMode::Running
            && self.automatic.ready.is_none_or(|ready| through > ready)
        {
            return Err(RunError::AutomaticInputsPending);
        }
        if self.mode != ControlMode::Running
            && through > self.freeze_info.expect("closed boundary").boundary
            && self.sealed != Some(through)
        {
            return Err(RunError::GateClosed);
        }
        self.sealed = Some(through);
        Ok(())
    }

    /// Cancellation uses the operation key, not a consumable result slot. It
    /// does not need event/input capacity or another request identifier.
    pub fn cancel_request(&mut self, key: RequestKey) -> Result<(), RunError> {
        self.check_source(key.source)?;
        if self.active.is_none_or(|active| active.key != key) {
            return Err(RunError::StaleCursor);
        }
        self.active = None;
        let keep_publication =
            matches!(self.work, Work::Publish { at, .. } if self.dirty == Some(at));
        if !keep_publication {
            self.work = Work::Select;
        }
        Ok(())
    }
}
