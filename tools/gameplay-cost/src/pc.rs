//! Fixed prepared/cancel-held/Drop batches. No product or native integration.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::driver::fixture;

    #[test]
    fn bounded_heap_initialization_counts_capacity_and_rejects_partial_construction() {
        assert!(storage_bytes() <= query::STORAGE_LIMIT);
        let s = Storage::try_new().unwrap();
        assert_eq!(s.slots.capacity(), 128);
        assert_eq!(s.slots.len(), 128);
        assert!(!s.owners_alive());
        assert!(s.slots.iter().all(|x| x.task.is_none() && x.key.is_none()));
        assert!(Storage::with_limit(storage_bytes() - 1, None).is_err());
        assert!(Storage::check_capacity(129, query::STORAGE_LIMIT).is_err());
        assert!(Storage::check_capacity(usize::MAX, query::STORAGE_LIMIT).is_err());
        for at in [0, 63, 127] {
            assert!(Storage::with_limit(query::STORAGE_LIMIT, Some(at)).is_err());
        }
        let a = admit(10000, false, storage_bytes(), AGGREGATE_LIMIT).unwrap();
        assert_eq!(
            a.aggregate_required,
            128 * (a.costs.source_bytes
                + a.costs.pool_bytes
                + a.costs.coordinator_bytes
                + a.costs.task_bytes
                + 2 * a.costs.run_bytes)
        );
        assert!(admit(10000, false, storage_bytes(), a.aggregate_required).is_ok());
        assert!(admit(10000, false, storage_bytes(), a.aggregate_required - 1).is_err());
        let mut overflow = a.costs;
        overflow.source_bytes = usize::MAX;
        assert!(admission(overflow, a.pool_limit, AGGREGATE_LIMIT).is_err());
        println!(
            "PC128 bounded layout: container={}, slots={}, slot={}, aggregate={}",
            storage_bytes(),
            s.slots.capacity(),
            std::mem::size_of::<Slot>(),
            a.aggregate_required
        );
    }

    #[test]
    fn partial_setup_only_reports_created_owners_and_observed_resources() {
        for at in [0, 63, 127] {
            let mut s = Storage::new();
            s.fault = Some(("setup", at));
            let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                s.establish(4, Trace::G0, WorkBudget::one(), &fixture(4), 0, true);
            }));
            assert!(outcome.is_err());
            assert_eq!(s.slots.iter().filter(|x| x.driver.is_some()).count(), at);
            s.cleanup();
            assert_eq!(s.cleanup_owners, at);
            assert_eq!(s.cleanup_tasks, 0);
            assert!(s.cleanup_after_tasks[..at].iter().all(Option::is_some));
            assert!(s.cleanup_after_tasks[at..].iter().all(Option::is_none));
            assert!(!s.owners_alive());
            s.cleanup();
            assert_eq!(s.cleanup_owners, at);
        }
    }

    #[test]
    fn every_round_failure_discards_old_results_and_releases_actual_remaining_tasks() {
        for round in [0, 1, 15] {
            for at in [0, 63, 127] {
                for stage in ["request", "work", "cancel", "drop"] {
                    let mut s = Storage::new();
                    s.establish(4, Trace::A1, WorkBudget::one(), &fixture(4), 0, true);
                    for r in 0..round {
                        s.prepare(r, false);
                        s.cancel(r, false);
                        s.drop_tasks(r, false);
                    }
                    let baseline: Vec<_> =
                        s.slots.iter().map(|x| x.baseline.unwrap().usage).collect();
                    s.fault = Some((stage, at));
                    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        s.prepare(round, false);
                        s.cancel(round, false);
                        s.drop_tasks(round, false);
                    }));
                    assert!(outcome.is_err(), "{round}/{at}/{stage}");
                    let held = match stage {
                        "request" => at,
                        "work" => at + 1,
                        "cancel" => OWNERS,
                        _ => OWNERS - at - 1,
                    };
                    assert_eq!(s.slots.iter().filter(|x| x.task.is_some()).count(), held);
                    assert_eq!(s.attempt.round, Some(round));
                    let completed = match stage {
                        "request" | "work" => [round; 3],
                        "cancel" => [round + 1, round, round],
                        _ => [round + 1, round + 1, round],
                    };
                    assert_eq!(s.lengths, completed);
                    let interrupted_work = s.slots[at].task.as_ref().map(|t| t.work_counts());
                    if matches!(stage, "request" | "work") {
                        assert!(s.slots.iter().all(|x| x.cancel.is_none()));
                        let unrequested = if stage == "request" { at } else { at + 1 };
                        assert!(s.slots[unrequested..].iter().all(|x| x.task_work.is_none()
                            && x.key.is_none()
                            && !x.checked
                            && !x.fresh));
                    } else if stage == "cancel" {
                        assert!(s.slots[..=at].iter().all(|x| x.cancel == Some(Ok(()))));
                        assert!(s.slots[at + 1..].iter().all(|x| x.cancel.is_none()));
                    }
                    s.cleanup();
                    assert_eq!(s.lengths, completed);
                    if stage == "work" {
                        assert_eq!(s.slots[at].task_work, interrupted_work);
                        assert!(s.slots[at].task_work.unwrap().management_steps > 0);
                    }
                    assert_eq!(s.cleanup_tasks, held);
                    assert_eq!(s.cleanup_owners, OWNERS);
                    assert!(
                        s.slots
                            .iter()
                            .all(|x| x.task.is_none() && x.driver.is_none())
                    );
                    for (i, original) in baseline.iter().enumerate() {
                        assert_eq!(s.cleanup_after_tasks[i], Some(*original));
                    }
                    s.cleanup();
                    assert_eq!(s.cleanup_tasks, held);
                    assert_eq!(s.cleanup_owners, OWNERS);
                }
            }
        }
    }
    #[test]
    fn cleanup_reads_actual_task_work_even_when_the_last_step_receipt_was_interrupted() {
        let mut s = Storage::new();
        s.establish(4, Trace::G0, WorkBudget::one(), &fixture(4), 0, true);
        let baseline = s.slots[0].baseline.unwrap().usage;
        {
            let slot = &mut s.slots[0];
            let d = slot.driver.as_mut().unwrap();
            let task = d.owner.request_inspection(d.source(), t(0)).unwrap();
            slot.key = Some(task.key());
            slot.task = Some(task);
        }
        // Core work has happened, but an enclosing diagnostic fails before its
        // cached work receipt is refreshed. No batch has completed.
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let slot = &mut s.slots[0];
            let task = slot.task.as_mut().unwrap();
            slot.steps += 1;
            task.step(task.cursor(), WorkBudget::one()).unwrap();
            panic!("interrupted diagnostic receipt");
        }));
        assert!(result.is_err());
        let actual = s.slots[0].task.as_ref().unwrap().work_counts();
        assert!(actual.valid && actual.management_steps > 0);
        assert!(s.slots[0].task_work.is_none());
        s.cleanup();
        assert_eq!(s.slots[0].task_work, Some(actual));
        assert_eq!(s.cleanup_after_tasks[0], Some(baseline));
        assert_eq!(s.cleanup_tasks, 1);
        assert_eq!(s.cleanup_owners, OWNERS);
        assert_eq!(s.slots[0].cleanup_cancel, Some(Ok(())));
        assert!(
            s.slots[1..]
                .iter()
                .all(|slot| slot.cleanup_cancel.is_none())
        );
        s.cleanup();
        assert_eq!(s.cleanup_tasks, 1);
        assert_eq!(s.cleanup_owners, OWNERS);
        assert_eq!(s.lengths, [0; 3]);
        assert!(s.slots.iter().all(|slot| slot.cancel.is_none()));
        let mut out = Vec::new();
        s.write_failure(&mut out).unwrap();
        assert!(out.len() < FAILURE_LIMIT);
        let receipt = String::from_utf8(out).unwrap();
        assert_eq!(
            receipt.matches("\"cleanup_cancel_success\":true").count(),
            1
        );
        assert_eq!(
            receipt.matches("\"cleanup_cancel_success\":null").count(),
            OWNERS - 1
        );
        assert_eq!(receipt.matches("\"cancel_success\":null").count(), OWNERS);
    }

    #[test]
    fn admission_is_checked_before_any_owner_and_counts_independent_sources() {
        let a = admit(10000, false, query::STORAGE_LIMIT, AGGREGATE_LIMIT).unwrap();
        assert_eq!(
            a.aggregate_required,
            OWNERS
                * (a.costs.source_bytes
                    + a.costs.pool_bytes
                    + a.costs.coordinator_bytes
                    + a.costs.task_bytes
                    + 2 * a.costs.run_bytes)
        );
        assert!(admit(10000, false, query::STORAGE_LIMIT, a.aggregate_required - 1).is_err());
        assert!(admit(10000, false, storage_bytes() - 1, AGGREGATE_LIMIT).is_err());
        assert!(admit(5, true, query::STORAGE_LIMIT, AGGREGATE_LIMIT).is_err());
        assert!(admit(10001, false, query::STORAGE_LIMIT, AGGREGATE_LIMIT).is_err());
        let mut costs = a.costs;
        costs.run_bytes = usize::MAX;
        assert!(admission(costs, a.pool_limit, AGGREGATE_LIMIT).is_err());
    }
    #[test]
    fn actual_tiny_all_traces_reuse_owners_but_never_keys_or_candidate_resources() {
        for trace in [Trace::G0, Trace::G1, Trace::A1] {
            let mut s = Storage::new();
            s.establish(4, trace, WorkBudget::one(), &fixture(4), 0, true);
            let baseline = s.aggregate;
            for r in 0..REPEATS {
                s.prepare(r, false);
                assert_eq!(s.aggregate[0], baseline[0]);
                assert_eq!(s.aggregate[4], baseline[4]);
                assert_eq!(s.aggregate[5], baseline[5] + OWNERS * 3);
                s.cancel(r, false);
                let held = s.aggregate;
                assert!(held[1] > baseline[1]);
                s.drop_tasks(r, false);
                assert_eq!(s.aggregate, baseline);
                if r + 1 < REPEATS {
                    let old = s.slots[0].key.unwrap();
                    let d = s.slots[0].driver.as_mut().unwrap();
                    assert!(d.owner.cancel(old).is_err());
                }
            }
            s.references(false);
            s.release();
            assert!(!s.owners_alive());
        }
    }
    #[test]
    fn failure_cleanup_needs_no_ack_or_sample_slot_and_preserves_actual_partial_results() {
        for stage in 0..3 {
            let mut s = Storage::new();
            s.establish(4, Trace::A1, WorkBudget::one(), &fixture(4), 0, true);
            let baseline = s.slots[0].baseline.unwrap().usage;
            s.prepare(0, false);
            if stage >= 1 {
                s.cancel(0, false);
            }
            if stage == 2 {
                s.drop_tasks(0, false);
            }
            s.cleanup();
            assert_eq!(s.cleanup_tasks, if stage == 2 { 0 } else { OWNERS });
            assert_eq!(s.cleanup_owners, OWNERS);
            assert!(!s.owners_alive());
            assert_eq!(s.cleanup_after_tasks[0], Some(baseline));
            assert_eq!(
                s.lengths,
                match stage {
                    0 => [1, 0, 0],
                    1 => [1, 1, 0],
                    _ => [1, 1, 1],
                }
            );
            let mut row = Vec::new();
            s.write_failure(&mut row).unwrap();
            assert!(row.len() < FAILURE_LIMIT);
        }
        let mut s = Storage::new();
        s.establish(4, Trace::G0, WorkBudget::one(), &fixture(4), 0, true);
        s.prepare(0, false);
        // A stale/cross-owner key must fail before the cancel timer/receipt.
        s.slots[0].key = s.slots[1].key;
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| s.cancel(0, false))).is_err()
        );
        assert_eq!(s.lengths, [1, 0, 0]);
        assert_eq!(
            s.attempt,
            Attempt {
                round: Some(0),
                phase: "cancel"
            }
        );
        assert!(s.slots.iter().all(|x| x.cancel.is_none()));
        s.cleanup();
        assert_eq!(s.cleanup_tasks, OWNERS);
    }
}

use crate::driver::{Driver, Setup, Trace, bound, t, target};
use crate::meter::{Meter, NAMES};
use crate::{query, write_work};
use skin_editor_gameplay::*;
use std::hint::black_box;
use std::io::{self, Write};
use std::time::Instant;

pub const PLAN: &str = "CostClosure-PC128-v1";
pub const OWNERS: usize = 128;
pub const REPEATS: usize = 16;
pub const IDS: [&str; 3] = ["PC-PREPARE-128", "PC-CANCEL-128", "PC-DROP-128"];
pub const AGGREGATE_LIMIT: usize = 256 * 1024 * 1024;
pub const FAILURE_LIMIT: usize = 512 * 1024;
pub const USAGE: [&str; 6] = [
    "source_reserved",
    "runtime_reserved",
    "source_payload",
    "runtime_payload",
    "source_allocations",
    "runtime_allocations",
];
type Usage = [usize; 6];

#[derive(Clone, Copy)]
struct Snapshot {
    observation: Observation,
    work: WorkCounts,
    usage: Usage,
}
struct Slot {
    driver: Option<Driver>,
    task: Option<PreparationTask<'static>>,
    key: Option<PreparationKey>,
    previous: Option<PreparationKey>,
    baseline: Option<Snapshot>,
    current: Option<Snapshot>,
    create_work: WorkCounts,
    play_work: WorkCounts,
    inspection_work: WorkCounts,
    calls: [u64; 24],
    task_work: Option<WorkCounts>,
    steps: usize,
    checked: bool,
    fresh: bool,
    cancel: Option<Result<(), PreparationError>>,
    cleanup_cancel: Option<Result<(), PreparationError>>,
}
impl Slot {
    fn empty() -> Self {
        Self {
            driver: None,
            task: None,
            key: None,
            previous: None,
            baseline: None,
            current: None,
            create_work: WorkCounts::default(),
            play_work: WorkCounts::default(),
            inspection_work: WorkCounts::default(),
            calls: [0; 24],
            task_work: None,
            steps: 0,
            checked: false,
            fresh: false,
            cancel: None,
            cleanup_cancel: None,
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Admission {
    costs: PreparationCosts,
    pool_limit: usize,
    aggregate_required: usize,
}
// A conservative requested-storage bound, including known constructor/move
// scratch, not an OS peak or a complete compiler-stack/allocator accounting.
// Four control structures cover construction, Result, Option and caller moves;
// only one heap slot allocation exists. Two empty Slot temporaries cover
// construction/push. Three query bounds include construction/move and its
// fixed caller buffers. Extra Driver/Context/Attempt scratch is explicit.
pub const fn storage_bytes() -> usize {
    std::mem::size_of::<Storage>()
        .checked_mul(4)
        .unwrap()
        .checked_add(OWNERS.checked_mul(std::mem::size_of::<Slot>()).unwrap())
        .unwrap()
        .checked_add(std::mem::size_of::<Slot>().checked_mul(2).unwrap())
        .unwrap()
        .checked_add(query::storage_bytes().checked_mul(3).unwrap())
        .unwrap()
        .checked_add(std::mem::size_of::<Driver>().checked_mul(2).unwrap())
        .unwrap()
        .checked_add(
            std::mem::size_of::<crate::Context>()
                .checked_mul(2)
                .unwrap(),
        )
        .unwrap()
        .checked_add(std::mem::size_of::<Attempt>().checked_mul(2).unwrap())
        .unwrap()
}
const _: () = assert!(storage_bytes() <= query::STORAGE_LIMIT);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Attempt {
    pub round: Option<usize>,
    pub phase: &'static str,
}
impl Attempt {
    pub const fn setup() -> Self {
        Self {
            round: None,
            phase: "setup",
        }
    }
}
fn admission(
    costs: PreparationCosts,
    pool_limit: usize,
    aggregate: usize,
) -> Result<Admission, &'static str> {
    let runtime = costs
        .pool_bytes
        .checked_add(costs.coordinator_bytes)
        .and_then(|x| x.checked_add(costs.task_bytes))
        .and_then(|x| {
            costs
                .run_bytes
                .checked_mul(2)
                .and_then(|r| x.checked_add(r))
        })
        .ok_or("PC arithmetic")?;
    let aggregate_required = costs
        .source_bytes
        .checked_add(runtime)
        .and_then(|x| x.checked_mul(OWNERS))
        .ok_or("PC aggregate arithmetic")?;
    if costs.source_bytes > pool_limit
        || runtime > pool_limit
        || costs.run_bytes > pool_limit
        || aggregate_required > aggregate
        || aggregate > AGGREGATE_LIMIT
    {
        return Err("PC pool/aggregate admission");
    }
    Ok(Admission {
        costs,
        pool_limit,
        aggregate_required,
    })
}
pub fn admit(
    n: usize,
    tiny: bool,
    container: usize,
    aggregate: usize,
) -> Result<Admission, &'static str> {
    if (tiny && n != 4)
        || (!tiny && n != 10000)
        || storage_bytes() > container
        || container > query::STORAGE_LIMIT
    {
        return Err("PC configuration/container admission");
    }
    query::admit(n, tiny, query::STORAGE_LIMIT)?;
    admission(
        SyntheticCoordinator::costs_for_objects(n).map_err(|_| "PC costs")?,
        if tiny { 64 * 1024 } else { 8 * 1024 * 1024 },
        aggregate,
    )
}
fn usage(owner: &SyntheticCoordinator) -> Usage {
    let r = owner.resources();
    let a = owner.allocations();
    [
        r.precompiled_bytes,
        r.runtime_bytes,
        a.precompiled_bytes,
        a.runtime_bytes,
        a.precompiled_allocations,
        a.runtime_allocations,
    ]
}
fn ns(begin: Option<Instant>) -> u64 {
    begin.map_or(0, |b| u64::try_from(b.elapsed().as_nanos()).unwrap())
}
#[derive(Clone, Copy)]
pub enum Receipt {
    Ready,
    Prepared(usize),
    Held(usize),
    Dropped(usize),
    Released,
    Summary,
}
pub struct Storage {
    pub attempt: Attempt,
    #[cfg(test)]
    fault: Option<(&'static str, usize)>,
    slots: Vec<Slot>,
    live: [Usage; OWNERS],
    aggregate: Usage,
    times: [[u64; REPEATS]; 3],
    references: [[u64; REPEATS]; 3],
    lengths: [usize; 3],
    reference_lengths: [usize; 3],
    reference_slots: [Option<PreparationKey>; OWNERS],
    admission: Option<Admission>,
    n: usize,
    ordinal: usize,
    released: bool,
    cleanup_tasks: usize,
    cleanup_owners: usize,
    cleanup_before: [Option<Usage>; OWNERS],
    cleanup_after_tasks: [Option<Usage>; OWNERS],
}
impl Storage {
    #[cfg(test)]
    pub(crate) fn maximum_failure_fixture(&mut self) {
        self.lengths = [REPEATS; 3];
        self.reference_lengths = [REPEATS; 3];
        self.times = [[u64::MAX; REPEATS]; 3];
        self.references = self.times;
        self.cleanup_tasks = OWNERS;
        self.cleanup_owners = OWNERS;
        self.cleanup_before = [Some([usize::MAX; 6]); OWNERS];
        self.cleanup_after_tasks = self.cleanup_before;
        for slot in &mut self.slots {
            slot.steps = usize::MAX;
            slot.task_work = Some(crate::maximum_work());
            slot.cancel = Some(Err(PreparationError::StaleRequest));
            slot.cleanup_cancel = slot.cancel;
        }
    }

    pub fn owners_alive(&self) -> bool {
        self.slots.iter().any(|s| s.driver.is_some())
    }
    #[cfg(test)]
    pub fn new() -> Self {
        Self::try_new().unwrap()
    }

    pub fn try_new() -> Result<Self, &'static str> {
        Self::with_limit(query::STORAGE_LIMIT, None)
    }
    fn check_capacity(capacity: usize, limit: usize) -> Result<(), &'static str> {
        if capacity != OWNERS || storage_bytes() > limit || limit > query::STORAGE_LIMIT {
            return Err("PC actual container capacity");
        }
        Ok(())
    }
    // The optional stop is used only by initialization failure tests; production
    // always passes None. No coordinator, task or source exists in this phase.
    fn with_limit(limit: usize, stop: Option<usize>) -> Result<Self, &'static str> {
        Self::check_capacity(OWNERS, limit)?;
        let mut slots = Vec::new();
        slots
            .try_reserve_exact(OWNERS)
            .map_err(|_| "PC slot reservation")?;
        Self::check_capacity(slots.capacity(), limit)?;
        for i in 0..OWNERS {
            if stop == Some(i) {
                return Err("PC slot initialization");
            }
            slots.push(Slot::empty());
        }
        let mut value = Self {
            attempt: Attempt::setup(),
            #[cfg(test)]
            fault: None,
            slots,
            live: [[0; 6]; OWNERS],
            aggregate: [0; 6],
            times: [[1; REPEATS]; 3],
            references: [[1; REPEATS]; 3],
            lengths: [0; 3],
            reference_lengths: [0; 3],
            reference_slots: [None; OWNERS],
            admission: None,
            n: 0,
            ordinal: 0,
            released: false,
            cleanup_tasks: 0,
            cleanup_owners: 0,
            cleanup_before: [None; OWNERS],
            cleanup_after_tasks: [None; OWNERS],
        };
        black_box(&value);
        value.times = [[0; REPEATS]; 3];
        value.references = [[0; REPEATS]; 3];
        Ok(value)
    }
    #[cfg(test)]
    fn fail(&self, stage: &'static str, index: usize) {
        assert_ne!(self.fault, Some((stage, index)), "injected tool boundary");
    }
    fn observe_resources(&mut self, i: usize) {
        let now = usage(&self.slots[i].driver.as_ref().unwrap().owner);
        let a = self.admission.unwrap();
        assert!(now[2] <= now[0] && now[3] <= now[1]);
        assert!(now[0] <= a.pool_limit && now[1] <= a.pool_limit);
        for (k, value) in now.iter().enumerate() {
            self.aggregate[k] = self.aggregate[k]
                .checked_sub(self.live[i][k])
                .unwrap()
                .checked_add(*value)
                .unwrap();
        }
        self.live[i] = now;
        assert!(self.aggregate[0].checked_add(self.aggregate[1]).unwrap() <= AGGREGATE_LIMIT);
    }
    fn capture(&mut self, i: usize) {
        self.observe_resources(i);
        let slot = &mut self.slots[i];
        let d = slot.driver.as_ref().unwrap();
        let run = d.owner.current().unwrap();
        let observation = run.observation();
        assert_eq!(observation.status, RunStatus::Frozen);
        assert_eq!(observation.pending_inputs, 0);
        assert_eq!(observation.pending_events, 0);
        assert_eq!(
            observation.statistics_origin,
            StatisticsOrigin::Inspection {
                at: t(target(self.n / 2))
            }
        );
        assert_eq!(
            observation.committed_counts,
            FrameCounts {
                hits: 0,
                misses: 0,
                inspection_skipped: self.n / 2
            }
        );
        assert_eq!(observation.ready_frame.unwrap().at, t(target(self.n / 2)));
        if let Some(base) = slot.baseline {
            assert_eq!(observation, base.observation);
        }
        let snap = Snapshot {
            observation,
            work: run.work_counts(),
            usage: self.live[i],
        };
        assert!(snap.work.valid);
        slot.current = Some(snap);
    }
    pub fn establish(
        &mut self,
        n: usize,
        trace: Trace,
        budget: WorkBudget,
        data: &[Circle],
        ordinal: usize,
        tiny: bool,
    ) {
        assert!(self.admission.is_none() && self.slots.iter().all(|s| s.driver.is_none()));
        self.admission = Some(admit(n, tiny, query::STORAGE_LIMIT, AGGREGATE_LIMIT).unwrap());
        self.n = n;
        self.ordinal = ordinal;
        let base = query::base_id(ordinal);
        self.attempt = Attempt::setup();
        for i in 0..OWNERS {
            #[cfg(test)]
            self.fail("setup", i);
            let mut meter = Meter::unmeasured();
            self.slots[i].driver = Some(Driver::create_with(
                n,
                trace,
                budget,
                data,
                &mut meter,
                Setup {
                    id: base.checked_add(100 + i as u64).unwrap(),
                    tiny,
                },
            ));
            {
                let slot = &mut self.slots[i];
                let d = slot.driver.as_mut().unwrap();
                slot.create_work = d.prepared_work;
                d.play(&mut meter);
                slot.play_work = d.owner.current().unwrap().work_counts();
                slot.inspection_work = d.inspect(&mut meter);
                slot.calls = *meter.calls();
                assert!(d.owner.candidate().is_none());
            }
            self.capture(i);
            self.slots[i].baseline = self.slots[i].current;
        }
        self.assert_aggregate();
    }
    fn assert_aggregate(&self) {
        let mut sum = [0_usize; 6];
        for u in self.live {
            for (s, value) in sum.iter_mut().zip(u) {
                *s = s.checked_add(value).unwrap();
            }
        }
        assert_eq!(sum, self.aggregate);
    }
    fn check_keys(&mut self) {
        for (i, slot) in self.slots.iter_mut().enumerate() {
            let d = slot.driver.as_ref().unwrap();
            let key = slot.key.unwrap();
            assert_eq!(Some(key), d.owner.candidate());
            assert_eq!(key, slot.task.as_ref().unwrap().key());
            assert_eq!(
                key.owner().get(),
                u128::from(query::base_id(self.ordinal) + 100 + i as u64)
            );
            assert_eq!(key.base(), Some(d.source()));
            assert_eq!(key.operation(), PreparationOperation::Inspect { at: t(0) });
            assert_eq!(key.driver(), d.owner.current().unwrap().driver());
            assert_eq!(key.target().run, d.source().run);
            assert_eq!(key.target().transport.get(), d.source().transport.get() + 1);
            assert_eq!(key.target().input.get(), d.source().input.get() + 1);
            assert_eq!(key.target().revision, d.source().revision);
            assert_ne!(Some(key), slot.previous);
            slot.checked = true;
            slot.fresh = true;
        }
    }
    pub fn prepare(&mut self, r: usize, timed: bool) {
        self.attempt = Attempt {
            round: (r < REPEATS).then_some(r),
            phase: "prepare",
        };
        assert!(r < REPEATS && self.lengths == [r; 3] && !self.released);
        for slot in &self.slots {
            assert!(
                slot.task.is_none() && slot.driver.as_ref().unwrap().owner.candidate().is_none()
            );
            assert_eq!(
                usage(&slot.driver.as_ref().unwrap().owner),
                slot.baseline.unwrap().usage
            );
        }
        let begin = timed.then(Instant::now);
        for slot in &mut self.slots {
            slot.key = None;
            slot.steps = 0;
            slot.task_work = None;
            slot.cancel = None;
            slot.cleanup_cancel = None;
            slot.checked = false;
            slot.fresh = false;
        }
        for i in 0..OWNERS {
            #[cfg(test)]
            self.fail("request", i);
            {
                let slot = &mut self.slots[i];
                let d = slot.driver.as_mut().unwrap();
                let task = d.owner.request_inspection(d.source(), t(0)).unwrap();
                slot.key = Some(task.key());
                slot.task = Some(task);
            }
            self.observe_resources(i);
            let mut reached = false;
            for _ in 0..bound(self.n) {
                let slot = &mut self.slots[i];
                let task = slot.task.as_mut().unwrap();
                slot.steps += 1;
                let report = task.step(task.cursor(), WorkBudget::one());
                #[cfg(test)]
                assert_ne!(self.fault, Some(("work", i)), "interrupted work receipt");
                let w = task.work_counts();
                slot.task_work = Some(w);
                assert!(w.valid);
                assert!(matches!(
                    report.unwrap(),
                    PreparationReport::Pending { steps: 1, .. }
                ));
                reached = w.allocation_attempts == 3
                    && w.state_initializations == (self.n + self.n / 2) as u64
                    && w.inspection_objects == (self.n / 2) as u64;
                if reached {
                    assert_eq!(task.stage(), PreparationStage::Rebuilding);
                }
                self.observe_resources(i);
                if reached {
                    break;
                }
            }
            assert!(reached, "PC exact half-way checkpoint");
        }
        self.check_keys();
        self.times[0][r] = ns(begin);
        self.lengths[0] += 1;
        for i in 0..OWNERS {
            self.capture(i);
        }
        self.assert_aggregate();
    }
    pub fn cancel(&mut self, r: usize, timed: bool) {
        self.attempt = Attempt {
            round: (r < REPEATS).then_some(r),
            phase: "cancel",
        };
        assert!(r < REPEATS && self.lengths == [r + 1, r, r]);
        self.check_keys();
        let begin = timed.then(Instant::now);
        for i in 0..OWNERS {
            let slot = &mut self.slots[i];
            slot.cancel = Some(black_box(
                slot.driver
                    .as_mut()
                    .unwrap()
                    .owner
                    .cancel(black_box(slot.key.unwrap())),
            ));
            #[cfg(test)]
            self.fail("cancel", i);
        }
        self.times[1][r] = ns(begin);
        self.lengths[1] += 1;
        for i in 0..OWNERS {
            let slot = &self.slots[i];
            assert_eq!(slot.cancel, Some(Ok(())));
            assert!(slot.driver.as_ref().unwrap().owner.candidate().is_none());
            assert_eq!(
                slot.task.as_ref().unwrap().work_counts(),
                slot.task_work.unwrap()
            );
            assert_eq!(
                usage(&slot.driver.as_ref().unwrap().owner),
                slot.current.unwrap().usage
            );
            self.capture(i);
        }
        self.assert_aggregate();
    }
    pub fn drop_tasks(&mut self, r: usize, timed: bool) {
        self.attempt = Attempt {
            round: (r < REPEATS).then_some(r),
            phase: "drop",
        };
        assert!(r < REPEATS && self.lengths == [r + 1, r + 1, r]);
        for slot in &self.slots {
            assert_eq!(slot.cancel, Some(Ok(())));
            assert!(
                slot.task.is_some() && slot.driver.as_ref().unwrap().owner.candidate().is_none()
            );
        }
        let begin = timed.then(Instant::now);
        for i in 0..OWNERS {
            drop(black_box(self.slots[i].task.take()));
            #[cfg(test)]
            self.fail("drop", i);
        }
        self.times[2][r] = ns(begin);
        self.lengths[2] += 1;
        for i in 0..OWNERS {
            let slot = &mut self.slots[i];
            assert_eq!(
                usage(&slot.driver.as_ref().unwrap().owner),
                slot.baseline.unwrap().usage
            );
            slot.previous = slot.key;
            self.capture(i);
        }
        self.assert_aggregate();
    }
    pub fn references(&mut self, timed: bool) {
        self.attempt = Attempt {
            round: None,
            phase: "reference",
        };
        assert_eq!(self.lengths, [REPEATS; 3]);
        assert_eq!(self.reference_lengths, [0; 3]);
        for kind in 0..3 {
            for r in 0..REPEATS {
                for (dst, slot) in self.reference_slots.iter_mut().zip(&self.slots) {
                    *dst = slot.key;
                }
                let begin = timed.then(Instant::now);
                // Only the fixed outer slots/arguments/barriers. No core call,
                // preparation step or real destructor is represented by this reference.
                for i in 0..OWNERS {
                    black_box(i);
                    if kind == 2 {
                        black_box(self.reference_slots[i].take());
                    } else {
                        self.reference_slots[i] = black_box(black_box(self.slots[i].key));
                    }
                }
                self.references[kind][r] = ns(begin);
                self.reference_lengths[kind] += 1;
            }
        }
    }
    pub fn release(&mut self) {
        self.attempt = Attempt {
            round: None,
            phase: "release",
        };
        assert_eq!(self.lengths, [REPEATS; 3]);
        assert_eq!(self.reference_lengths, [REPEATS; 3]);
        for slot in &mut self.slots {
            assert!(slot.task.is_none());
            assert_eq!(
                usage(&slot.driver.as_ref().unwrap().owner),
                slot.baseline.unwrap().usage
            );
            drop(slot.driver.take());
        }
        self.released = true;
    }
    pub fn cleanup(&mut self) {
        for (i, slot) in self.slots.iter_mut().enumerate() {
            // A step/diagnostic may unwind before updating the cached receipt.
            // Read the surviving task before cancellation/Drop; no new work or timer.
            if let Some(task) = slot.task.as_ref() {
                slot.task_work = Some(task.work_counts());
            }
            if let Some(d) = slot.driver.as_mut() {
                self.cleanup_before[i] = Some(usage(&d.owner));
                if let Some(key) = d.owner.candidate() {
                    slot.cleanup_cancel = Some(d.owner.cancel(key));
                }
                if let Some(task) = slot.task.take() {
                    drop(task);
                    self.cleanup_tasks += 1;
                }
                self.cleanup_after_tasks[i] = Some(usage(&d.owner));
            } else if let Some(task) = slot.task.take() {
                drop(task);
                self.cleanup_tasks += 1;
            }
            if let Some(d) = slot.driver.take() {
                drop(d);
                self.cleanup_owners += 1;
            }
        }
    }
    pub fn write(&self, out: &mut impl Write, receipt: Receipt) -> io::Result<()> {
        if let Receipt::Summary = receipt {
            assert_eq!(self.lengths, [REPEATS; 3]);
            assert!(self.released);
            write!(
                out,
                "{{\"kind\":\"summary\",\"scope\":\"Complete\",\"product_verdict\":\"NotEvaluated\",\"batches\":["
            )?;
            for (kind, id) in IDS.iter().enumerate() {
                if kind > 0 {
                    write!(out, ",")?;
                }
                write!(out, "{{\"id\":\"{}\",\"timing\":", id)?;
                query::write_times(out, &self.times[kind])?;
                write!(out, ",\"reference\":")?;
                query::write_times(out, &self.references[kind])?;
                write!(out, "}}")?;
            }
            return write!(out, "]}}");
        }
        if let Receipt::Released = receipt {
            assert!(
                self.released
                    && self
                        .slots
                        .iter()
                        .all(|s| s.driver.is_none() && s.task.is_none())
            );
            write!(
                out,
                "{{\"kind\":\"released\",\"owners_dropped\":{OWNERS},\"tasks_held\":0,\"owners\":null,\"aggregate\":null,\"reference_scope\":\"outer_loop_only\",\"reference_ns\":["
            )?;
            for kind in 0..3 {
                if kind > 0 {
                    write!(out, ",")?;
                }
                query::write_times(out, &self.references[kind])?;
            }
            return write!(out, "]}}");
        }
        let (kind, r) = match receipt {
            Receipt::Ready => ("ready", None),
            Receipt::Prepared(r) => ("prepared", Some(r)),
            Receipt::Held(r) => ("cancelled-held", Some(r)),
            Receipt::Dropped(r) => ("dropped", Some(r)),
            _ => unreachable!(),
        };
        write!(out, "{{\"kind\":\"{kind}\"")?;
        if let Some(r) = r {
            let k = match receipt {
                Receipt::Prepared(_) => 0,
                Receipt::Held(_) => 1,
                _ => 2,
            };
            write!(out, ",\"r\":{r},\"action_ns\":{}", self.times[k][r])?;
        }
        let a = self.admission.unwrap();
        let c = a.costs;
        write!(
            out,
            ",\"admission\":{{\"source_bytes\":{},\"run_bytes\":{},\"task_bytes\":{},\"pool_bytes\":{},\"coordinator_bytes\":{},\"pool_limit\":{},\"aggregate_limit\":{AGGREGATE_LIMIT},\"aggregate_required\":{},\"container_bytes\":{}}},\"aggregate\":",
            c.source_bytes,
            c.run_bytes,
            c.task_bytes,
            c.pool_bytes,
            c.coordinator_bytes,
            a.pool_limit,
            a.aggregate_required,
            storage_bytes()
        )?;
        write_usage(out, self.aggregate)?;
        write!(out, ",\"owners\":[")?;
        for (i, slot) in self.slots.iter().enumerate() {
            if i > 0 {
                write!(out, ",")?;
            }
            let snap = slot.current.unwrap();
            let d = slot.driver.as_ref().unwrap();
            write!(out, "{{\"i\":{i},\"frame\":")?;
            write_frame(out, snap.observation, d.owner.current().unwrap().driver())?;
            write!(out, ",\"resources\":")?;
            write_usage(out, snap.usage)?;
            write!(out, ",\"active_work\":")?;
            write_work(out, snap.work)?;
            write!(out, ",\"create_work\":")?;
            write_work(out, slot.create_work)?;
            write!(out, ",\"play_work\":")?;
            write_work(out, slot.play_work)?;
            write!(out, ",\"inspection_work\":")?;
            write_work(out, slot.inspection_work)?;
            write!(out, ",\"calls\":{{")?;
            for (j, name) in NAMES.iter().enumerate() {
                if j > 0 {
                    write!(out, ",")?;
                }
                write!(out, "\"{name}\":{}", slot.calls[j])?;
            }
            let p = d.peaks;
            write!(
                out,
                "}},\"peaks\":{{\"inputs\":{},\"events\":{},\"source_reserved\":{},\"runtime_reserved\":{},\"source_payload\":{},\"runtime_payload\":{}}},\"task_held\":{},\"eligible\":{},\"steps\":{},\"task_work\":",
                p.inputs,
                p.events,
                p.source,
                p.runtime,
                p.payload_source,
                p.payload_runtime,
                slot.task.is_some(),
                d.owner.candidate().is_some(),
                slot.steps
            )?;
            if let Some(w) = slot.task_work {
                write_work(out, w)?;
            } else {
                write!(out, "null")?;
            }
            write!(out, ",\"cancel_success\":")?;
            if let Some(result) = slot.cancel {
                write!(out, "{}", result.is_ok())?;
            } else {
                write!(out, "null")?;
            }
            write!(out, ",\"key\":")?;
            if let Some(key) = slot.key {
                let r = r.expect("candidate sample");
                write!(
                    out,
                    "{{\"handle\":[{},{r},{i}],\"owner\":{},\"base\":",
                    self.ordinal,
                    key.owner().get()
                )?;
                write_source(out, key.base().unwrap())?;
                write!(out, ",\"target\":")?;
                write_source(out, key.target())?;
                write!(
                    out,
                    ",\"operation\":\"Inspect\",\"at\":0,\"driver\":\"{}\",\"full_eq_checked\":{},\"fresh\":{}}}",
                    driver_name(key.driver()),
                    slot.checked,
                    slot.fresh
                )?;
            } else {
                write!(out, "null")?;
            }
            write!(out, "}}")?;
        }
        write!(out, "]}}")
    }
    pub fn write_failure(&self, out: &mut impl Write) -> io::Result<()> {
        write!(
            out,
            "{{\"completed\":[{},{},{}],\"tasks_dropped\":{},\"owners_dropped\":{},\"post_owner_resources\":null,\"retained_ns\":[",
            self.lengths[0],
            self.lengths[1],
            self.lengths[2],
            self.cleanup_tasks,
            self.cleanup_owners
        )?;
        for k in 0..3 {
            if k > 0 {
                write!(out, ",")?;
            }
            write!(out, "[")?;
            for (i, n) in self.times[k].iter().take(self.lengths[k]).enumerate() {
                if i > 0 {
                    write!(out, ",")?;
                }
                write!(out, "{n}")?;
            }
            write!(out, "]")?;
        }
        write!(
            out,
            "],\"reference_completed\":[{},{},{}],\"retained_reference_ns\":[",
            self.reference_lengths[0], self.reference_lengths[1], self.reference_lengths[2]
        )?;
        for k in 0..3 {
            if k > 0 {
                write!(out, ",")?;
            }
            write!(out, "[")?;
            for (i, n) in self.references[k]
                .iter()
                .take(self.reference_lengths[k])
                .enumerate()
            {
                if i > 0 {
                    write!(out, ",")?;
                }
                write!(out, "{n}")?;
            }
            write!(out, "]")?;
        }
        write!(out, "],\"owners\":[")?;
        for i in 0..OWNERS {
            if i > 0 {
                write!(out, ",")?;
            }
            write!(
                out,
                "{{\"i\":{i},\"steps\":{},\"key_eq_checked\":{},\"fresh\":{},\"before\":",
                self.slots[i].steps, self.slots[i].checked, self.slots[i].fresh
            )?;
            if let Some(u) = self.cleanup_before[i] {
                write_usage(out, u)?;
            } else {
                write!(out, "null")?;
            }
            write!(out, ",\"after_tasks\":")?;
            if let Some(u) = self.cleanup_after_tasks[i] {
                write_usage(out, u)?;
            } else {
                write!(out, "null")?;
            }
            write!(out, ",\"cancel_success\":")?;
            if let Some(r) = self.slots[i].cancel {
                write!(out, "{}", r.is_ok())?;
            } else {
                write!(out, "null")?;
            }
            write!(out, ",\"cleanup_cancel_success\":")?;
            if let Some(r) = self.slots[i].cleanup_cancel {
                write!(out, "{}", r.is_ok())?;
            } else {
                write!(out, "null")?;
            }
            write!(out, ",\"task_work\":")?;
            if let Some(w) = self.slots[i].task_work {
                write_work(out, w)?;
            } else {
                write!(out, "null")?;
            }
            write!(out, "}}")?;
        }
        write!(out, "]}}")
    }
}
fn write_usage(out: &mut impl Write, u: Usage) -> io::Result<()> {
    write!(out, "{{")?;
    for (i, name) in USAGE.iter().enumerate() {
        if i > 0 {
            write!(out, ",")?;
        }
        write!(out, "\"{name}\":{}", u[i])?;
    }
    write!(out, "}}")
}
fn write_source(out: &mut impl Write, s: RunSource) -> io::Result<()> {
    write!(
        out,
        "{{\"run\":{},\"transport\":{},\"input\":{},\"revision\":{}}}",
        s.run.get(),
        s.transport.get(),
        s.input.get(),
        s.revision.get()
    )
}
fn driver_name(d: InputDriver) -> &'static str {
    match d {
        InputDriver::Manual => "Manual",
        InputDriver::Automatic => "Automatic",
        InputDriver::Regression => "Regression",
    }
}
fn write_frame(out: &mut impl Write, o: Observation, d: InputDriver) -> io::Result<()> {
    let h = o.ready_frame.unwrap();
    let StatisticsOrigin::Inspection { at } = o.statistics_origin else {
        panic!("PC inspection origin");
    };
    write!(out, "{{\"source\":")?;
    write_source(out, h.source)?;
    write!(
        out,
        ",\"at\":{},\"publication\":{},\"objects\":{},\"hits\":{},\"misses\":{},\"skipped\":{},\"statistics_at\":{},\"driver\":\"{}\",\"status\":\"Frozen\",\"pending_inputs\":{},\"pending_events\":{},\"evidence\":\"SyntheticTest\",\"profile_version\":1}}",
        h.at.micros(),
        h.key().publication(),
        h.len(),
        h.counts.hits,
        h.counts.misses,
        h.counts.inspection_skipped,
        at.micros(),
        driver_name(d),
        o.pending_inputs,
        o.pending_events
    )
}
