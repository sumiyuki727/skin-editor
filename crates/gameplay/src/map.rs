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

pub(crate) mod preparation;
use crate::storage::Buffer;
pub(crate) use preparation::IdKey;
pub use preparation::{MapPreparation, PreparationPhase, PreparationProgress};

/// Immutable compiled source. Moving it moves only fixed descriptors.
/// Debug prints metadata; object data requires explicit bounded reads.
pub struct SyntheticMap {
    pub(crate) revision: MapRevision,
    pub(crate) circles: Buffer<Circle>,
    pub(crate) len: usize,
    pub(crate) payload: Option<crate::coordinator::PayloadLease>,
}
impl SyntheticMap {
    /// Bounded convenience path. Large maps require MapPreparation::with_limits.
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
    pub fn capacity(&self) -> usize {
        self.circles.capacity()
    }
    pub fn circle(&self, index: usize) -> Option<Circle> {
        self.circles.get(index).copied()
    }
}
impl core::fmt::Debug for SyntheticMap {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SyntheticMap")
            .field("revision", &self.revision)
            .field("len", &self.len)
            .field("capacity", &self.capacity())
            .finish()
    }
}
impl PartialEq for SyntheticMap {
    fn eq(&self, other: &Self) -> bool {
        self.revision == other.revision && self.circles == other.circles
    }
}
impl Eq for SyntheticMap {}
