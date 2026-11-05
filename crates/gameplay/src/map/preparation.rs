use super::*;
use crate::coordinator::PoolState;
use crate::{AllocationKind, Limits, PreparationError, ResourcePool, WorkCounts};
use std::rc::Rc;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PreparationProgress {
    Pending { checked: usize, total: usize },
    Complete { checked: usize },
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PreparationPhase {
    Allocating,
    Records,
    IndexInitialization,
    Sorting,
    DuplicateCheck,
    Complete,
}
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct IdKey {
    id: ObjectId,
    position: usize,
}
#[derive(Clone, Copy)]
struct Merge {
    width: usize,
    left: usize,
    middle: usize,
    right: usize,
    end: usize,
    out: usize,
    from_a: bool,
}
#[derive(Clone, Copy)]
enum Stage {
    Allocate(u8),
    Records,
    Initialize,
    Begin {
        width: usize,
        start: usize,
        from_a: bool,
    },
    Choose(Merge),
    Write(Merge, IdKey),
    Duplicates {
        next: usize,
        from_a: bool,
    },
    Done,
}

/// Bounded, caller-stepped preparation. No map or partially validated source escapes.
pub struct MapPreparation<'a> {
    input: &'a [Circle],
    map: SyntheticMap,
    first: Buffer<IdKey>,
    second: Buffer<IdKey>,
    pool: Option<Rc<PoolState>>,
    stage: Stage,
    stats: WorkCounts,
    cancelled: bool,
    error: Option<ValueError>,
    earliest: Option<(usize, u8, ValueError)>,
    max_work: u16,
    previous: Option<(MapTime, MapTime)>,
}
impl<'a> MapPreparation<'a> {
    pub fn new(revision: MapRevision, input: &'a [Circle]) -> Result<Self, ValueError> {
        if input.len() > MAX_CIRCLES {
            return Err(ValueError::ObjectLimit);
        }
        Self::build(revision, input, Limits::reference(), None)
    }
    pub fn with_limits(
        revision: MapRevision,
        input: &'a [Circle],
        limits: Limits,
    ) -> Result<Self, ValueError> {
        Self::build(revision, input, limits, None)
    }
    pub(crate) fn managed(
        revision: MapRevision,
        input: &'a [Circle],
        limits: Limits,
        pool: Rc<PoolState>,
    ) -> Result<Self, ValueError> {
        Self::build(revision, input, limits, Some(pool))
    }
    fn build(
        revision: MapRevision,
        input: &'a [Circle],
        limits: Limits,
        pool: Option<Rc<PoolState>>,
    ) -> Result<Self, ValueError> {
        limits.validate()?;
        if input.len() > limits.object_capacity() {
            return Err(ValueError::ObjectLimit);
        }
        let c = crate::storage::capacity(input.len())?;
        let bytes = Self::storage_bytes(input.len())?;
        if pool.is_none() && bytes > limits.storage_bytes() {
            return Err(ValueError::StorageLimit);
        }
        Ok(Self {
            input,
            map: SyntheticMap {
                revision,
                circles: Buffer::new(c),
                len: 0,
                payload: None,
            },
            first: Buffer::new(c),
            second: Buffer::new(c),
            pool,
            stage: Stage::Allocate(0),
            stats: WorkCounts::default(),
            cancelled: false,
            error: None,
            earliest: None,
            max_work: limits.budget().get(),
            previous: None,
        })
    }
    pub fn storage_bytes(objects: usize) -> Result<usize, ValueError> {
        let c = crate::storage::capacity(objects)?;
        core::mem::size_of::<Self>()
            .checked_add(crate::storage::bytes::<Circle>(c)?)
            .and_then(|v| v.checked_add(crate::storage::bytes::<IdKey>(c).ok()?.checked_mul(2)?))
            .ok_or(ValueError::StorageLimit)
    }
    pub fn work_counts(&self) -> WorkCounts {
        self.stats
    }
    pub fn failed_record_index(&self) -> Option<usize> {
        self.error.and_then(|_| self.earliest.map(|v| v.0))
    }
    pub fn capacities(&self) -> [usize; 3] {
        [
            self.map.capacity(),
            self.first.capacity(),
            self.second.capacity(),
        ]
    }
    pub fn phase(&self) -> PreparationPhase {
        match self.stage {
            Stage::Allocate(_) => PreparationPhase::Allocating,
            Stage::Records => PreparationPhase::Records,
            Stage::Initialize => PreparationPhase::IndexInitialization,
            Stage::Begin { .. } | Stage::Choose(_) | Stage::Write(_, _) => {
                PreparationPhase::Sorting
            }
            Stage::Duplicates { .. } => PreparationPhase::DuplicateCheck,
            Stage::Done => PreparationPhase::Complete,
        }
    }
    pub fn step(&mut self, budget: WorkBudget) -> Result<PreparationProgress, ValueError> {
        if self.cancelled {
            return Err(ValueError::PreparationCancelled);
        }
        if let Some(error) = self.error {
            return Err(error);
        }
        if budget.get() > self.max_work {
            return Err(ValueError::InvalidBudget);
        }
        for _ in 0..budget.get() {
            if matches!(self.stage, Stage::Done) {
                break;
            }
            if let Err(error) = self.one() {
                self.error = Some(error);
                return Err(error);
            }
        }
        Ok(if matches!(self.stage, Stage::Done) {
            PreparationProgress::Complete {
                checked: self.map.len,
            }
        } else {
            PreparationProgress::Pending {
                checked: self.map.len,
                total: self.input.len(),
            }
        })
    }
    fn note_error(&mut self, at: usize, rank: u8, error: ValueError) {
        if self.earliest.is_none_or(|old| (at, rank) < (old.0, old.1)) {
            self.earliest = Some((at, rank, error));
        }
    }
    fn key(&self, first: bool, at: usize) -> IdKey {
        if first {
            self.first[at]
        } else {
            self.second[at]
        }
    }
    fn one(&mut self) -> Result<(), ValueError> {
        let mut delta = WorkCounts::default();
        match self.stage {
            Stage::Allocate(which) => {
                let (buffer, kind, class) = match which {
                    0 => {
                        self.map
                            .circles
                            .allocate(
                                &mut self.stats,
                                self.pool.as_ref(),
                                AllocationKind::Circles,
                                ResourcePool::Precompiled,
                            )
                            .map_err(value_error)?;
                        self.stage = Stage::Allocate(1);
                        self.stats.merge(WorkCounts {
                            management_steps: 1,
                            ..WorkCounts::default()
                        });
                        return Ok(());
                    }
                    1 => (
                        &mut self.first,
                        AllocationKind::IndexFirst,
                        ResourcePool::Runtime,
                    ),
                    _ => (
                        &mut self.second,
                        AllocationKind::IndexSecond,
                        ResourcePool::Runtime,
                    ),
                };
                buffer
                    .allocate(&mut self.stats, self.pool.as_ref(), kind, class)
                    .map_err(value_error)?;
                self.stage = if which == 1 {
                    Stage::Allocate(2)
                } else {
                    Stage::Records
                };
                delta.management_steps = 1;
            }
            Stage::Records => {
                let i = self.map.len;
                if let Some(circle) = self.input.get(i).copied() {
                    delta.record_reads = 1;
                    delta.record_copies = 1;
                    delta.index_initializations = 1;
                    if let Some((target, close)) = self.previous {
                        if circle.target < target {
                            self.note_error(i, 1, ValueError::UnsortedMap);
                        } else if circle.open <= close {
                            self.note_error(i, 2, ValueError::UnsupportedOverlap);
                        }
                    }
                    self.map.circles.push(circle);
                    self.previous = Some((circle.target, circle.close));
                    self.first.push(IdKey {
                        id: circle.id,
                        position: i,
                    });
                    self.map.len += 1;
                } else {
                    self.stage = Stage::Initialize;
                    delta.management_steps = 1;
                }
            }
            Stage::Initialize => {
                if self.second.len() < self.map.len {
                    self.second.push(IdKey {
                        id: ObjectId::new(0),
                        position: 0,
                    });
                    delta.index_initializations = 1;
                } else {
                    self.stage = Stage::Begin {
                        width: 1,
                        start: 0,
                        from_a: true,
                    };
                    delta.management_steps = 1;
                }
            }
            Stage::Begin {
                width,
                start,
                from_a,
            } => {
                delta.management_steps = 1;
                self.stage = if width >= self.map.len {
                    Stage::Duplicates { next: 1, from_a }
                } else if start >= self.map.len {
                    Stage::Begin {
                        width: width.checked_mul(2).ok_or(ValueError::StorageLimit)?,
                        start: 0,
                        from_a: !from_a,
                    }
                } else {
                    let middle = start
                        .checked_add(width)
                        .ok_or(ValueError::StorageLimit)?
                        .min(self.map.len);
                    let end = middle
                        .checked_add(width)
                        .ok_or(ValueError::StorageLimit)?
                        .min(self.map.len);
                    Stage::Choose(Merge {
                        width,
                        left: start,
                        middle,
                        right: middle,
                        end,
                        out: start,
                        from_a,
                    })
                };
            }
            Stage::Choose(mut m) => {
                delta.management_steps = 1;
                if m.out == m.end {
                    self.stage = Stage::Begin {
                        width: m.width,
                        start: m.end,
                        from_a: m.from_a,
                    };
                    delta.management_steps = 1;
                } else {
                    let left = if m.right == m.end {
                        true
                    } else if m.left == m.middle {
                        false
                    } else {
                        delta.id_comparisons = 1;
                        self.key(m.from_a, m.left) <= self.key(m.from_a, m.right)
                    };
                    let key = self.key(m.from_a, if left { m.left } else { m.right });
                    if left {
                        m.left += 1;
                    } else {
                        m.right += 1;
                    }
                    self.stage = Stage::Write(m, key);
                }
            }
            Stage::Write(mut m, key) => {
                if m.from_a {
                    self.second[m.out] = key;
                } else {
                    self.first[m.out] = key;
                }
                m.out += 1;
                self.stage = Stage::Choose(m);
                delta.index_moves = 1;
            }
            Stage::Duplicates { next, from_a } => {
                if next < self.map.len {
                    delta.id_comparisons = 1;
                    let a = self.key(from_a, next - 1);
                    let b = self.key(from_a, next);
                    if a.id == b.id {
                        self.note_error(b.position, 0, ValueError::DuplicateObject);
                    }
                    self.stage = Stage::Duplicates {
                        next: next + 1,
                        from_a,
                    };
                } else {
                    delta.management_steps = 1;
                    self.stats.merge(delta);
                    if let Some((_, _, error)) = self.earliest {
                        return Err(error);
                    }
                    self.stage = Stage::Done;
                    return Ok(());
                }
            }
            Stage::Done => return Ok(()),
        }
        self.stats.merge(delta);
        Ok(())
    }
    pub fn cancel(&mut self) {
        self.cancelled = true;
    }
    pub fn finish(self) -> Result<SyntheticMap, ValueError> {
        if self.cancelled {
            return Err(ValueError::PreparationCancelled);
        }
        if let Some(error) = self.error {
            return Err(error);
        }
        if !matches!(self.stage, Stage::Done) {
            return Err(ValueError::PreparationIncomplete);
        }
        Ok(self.map)
    }
}
pub(crate) fn value_error(error: PreparationError) -> ValueError {
    match error {
        PreparationError::Invalid(error) => error,
        PreparationError::AllocationUnavailable(kind) => ValueError::AllocationUnavailable(kind),
        _ => ValueError::StorageLayout,
    }
}
