use crate::{MAX_CIRCLES, MapRevision, MapTime, ObjectId, Point, ValueError, WorkBudget};

/// Immutable SyntheticTest-v1 circle. All offsets are engineering test rules.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Circle {
    pub(crate) id: ObjectId,
    pub(crate) center: Point,
    pub(crate) target: MapTime,
    pub(crate) visible: MapTime,
    pub(crate) open: MapTime,
    pub(crate) close: MapTime,
    pub(crate) miss: MapTime,
    pub(crate) retire: MapTime,
}
impl Circle {
    pub fn new(id: ObjectId, target: MapTime, center: Point) -> Result<Self, ValueError> {
        Ok(Self {
            id,
            center,
            target,
            visible: target.checked_offset(-200_000)?,
            open: target.checked_offset(-100_000)?,
            close: target.checked_offset(100_000)?,
            miss: target.checked_offset(100_001)?,
            retire: target.checked_offset(300_000)?,
        })
    }
    pub const fn id(self) -> ObjectId {
        self.id
    }
    pub const fn target(self) -> MapTime {
        self.target
    }
    pub const fn center(self) -> Point {
        self.center
    }
    pub const fn visible_from(self) -> MapTime {
        self.visible
    }
    pub const fn accepts_from(self) -> MapTime {
        self.open
    }
    pub const fn accepts_through(self) -> MapTime {
        self.close
    }
    pub const fn miss_at(self) -> MapTime {
        self.miss
    }
    pub const fn retires_at(self) -> MapTime {
        self.retire
    }
}

/// Deliberately small: no arbitrary-length or partially validated map escapes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SyntheticMap {
    pub(crate) revision: MapRevision,
    pub(crate) circles: [Option<Circle>; MAX_CIRCLES],
    pub(crate) len: usize,
}
impl SyntheticMap {
    /// Convenience for a fixed, at-most-four-element input. Larger preparations
    /// and compatibility profiles are not supported by this API.
    pub fn try_new(revision: MapRevision, circles: &[Circle]) -> Result<Self, ValueError> {
        let mut preparation = MapPreparation::new(revision, circles)?;
        preparation.step(WorkBudget::full())?;
        preparation.finish()
    }
    pub const fn revision(&self) -> MapRevision {
        self.revision
    }
    pub const fn len(&self) -> usize {
        self.len
    }
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }
    pub fn circle(&self, index: usize) -> Option<Circle> {
        self.circles.get(index).copied().flatten()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PreparationProgress {
    Pending { checked: usize, total: usize },
    Complete { checked: usize },
}

/// Synchronous, caller-stepped preparation. It has no reference to an active
/// runtime and can only yield a complete immutable map by consuming itself.
pub struct MapPreparation<'a> {
    input: &'a [Circle],
    candidate: SyntheticMap,
    pending: Option<Circle>,
    next_id: usize,
    cancelled: bool,
    error: Option<ValueError>,
}
impl<'a> MapPreparation<'a> {
    pub fn new(revision: MapRevision, input: &'a [Circle]) -> Result<Self, ValueError> {
        if input.len() > MAX_CIRCLES {
            return Err(ValueError::ObjectLimit);
        }
        Ok(Self {
            input,
            candidate: SyntheticMap {
                revision,
                circles: [None; MAX_CIRCLES],
                len: 0,
            },
            pending: None,
            next_id: 0,
            cancelled: false,
            error: None,
        })
    }
    pub fn step(&mut self, budget: WorkBudget) -> Result<PreparationProgress, ValueError> {
        if self.cancelled {
            return Err(ValueError::PreparationCancelled);
        }
        if let Some(error) = self.error {
            return Err(error);
        }
        for _ in 0..budget.get() {
            let index = self.candidate.len;
            let Some(circle) = self.pending else {
                let Some(circle) = self.input.get(index).copied() else {
                    break;
                };
                // One input-record inspection. Later calls retain this value
                // while inspecting at most one previous identity per unit.
                if index == 0 {
                    self.candidate.circles[0] = Some(circle);
                    self.candidate.len = 1;
                } else {
                    self.pending = Some(circle);
                    self.next_id = 0;
                }
                continue;
            };
            let previous = self.candidate.circles[self.next_id].expect("validated predecessor");
            let last = self.next_id + 1 == index;
            let error = if previous.id == circle.id {
                Some(ValueError::DuplicateObject)
            } else if last && circle.target < previous.target {
                Some(ValueError::UnsortedMap)
            } else if last && circle.open <= previous.close {
                Some(ValueError::UnsupportedOverlap)
            } else {
                None
            };
            if let Some(error) = error {
                self.error = Some(error);
                return Err(error);
            }
            if last {
                self.candidate.circles[index] = Some(circle);
                self.candidate.len += 1;
                self.pending = None;
            } else {
                self.next_id += 1;
            }
        }
        Ok(if self.candidate.len == self.input.len() {
            PreparationProgress::Complete {
                checked: self.candidate.len,
            }
        } else {
            PreparationProgress::Pending {
                checked: self.candidate.len,
                total: self.input.len(),
            }
        })
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
        if self.candidate.len != self.input.len() {
            return Err(ValueError::PreparationIncomplete);
        }
        Ok(self.candidate)
    }
}
