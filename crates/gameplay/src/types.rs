use core::num::NonZeroU64;
use core::num::NonZeroU128;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValueError {
    ZeroIdentity,
    TimeOverflow,
    CoordinateRange,
    ObjectLimit,
    DuplicateObject,
    UnsortedMap,
    UnsupportedOverlap,
    InvalidStart,
    RevisionMismatch,
    InvalidLimits,
    StorageLimit,
    InvalidBudget,
    PreparationIncomplete,
    PreparationCancelled,
    StorageLayout,
    AllocationUnavailable(crate::AllocationKind),
}

macro_rules! identity {
    ($name:ident, $raw:ty, $nonzero:ty) => {
        #[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
        pub struct $name($nonzero);
        impl $name {
            pub const fn new(value: $raw) -> Result<Self, ValueError> {
                match <$nonzero>::new(value) {
                    Some(value) => Ok(Self(value)),
                    None => Err(ValueError::ZeroIdentity),
                }
            }
            pub const fn get(self) -> $raw {
                self.0.get()
            }
        }
    };
}
identity!(RunId, u128, NonZeroU128);
identity!(CoordinatorId, u128, NonZeroU128);
identity!(TransportEpoch, u64, NonZeroU64);
identity!(InputEpoch, u64, NonZeroU64);
identity!(InputSequence, u64, NonZeroU64);
identity!(EventSequence, u64, NonZeroU64);

impl TransportEpoch {
    pub fn checked_next(self) -> Option<Self> {
        self.get().checked_add(1).and_then(|v| Self::new(v).ok())
    }
}
impl InputEpoch {
    pub fn checked_next(self) -> Option<Self> {
        self.get().checked_add(1).and_then(|v| Self::new(v).ok())
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ObjectId(u32);
impl ObjectId {
    pub const fn new(value: u32) -> Self {
        Self(value)
    }
    pub const fn get(self) -> u32 {
        self.0
    }
}

/// An opaque synthetic revision token, not a file digest or project identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MapRevision(u128);
impl MapRevision {
    pub const fn new(value: u128) -> Self {
        Self(value)
    }
    pub const fn get(self) -> u128 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct MapTime(i64);
impl MapTime {
    pub const fn from_micros(value: i64) -> Self {
        Self(value)
    }
    pub const fn micros(self) -> i64 {
        self.0
    }
    pub fn checked_offset(self, offset: i64) -> Result<Self, ValueError> {
        self.0
            .checked_add(offset)
            .map(Self)
            .ok_or(ValueError::TimeOverflow)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Point {
    x: i32,
    y: i32,
}
impl Point {
    pub fn new(x: i32, y: i32) -> Result<Self, ValueError> {
        if !(-1024..=1024).contains(&x) || !(-1024..=1024).contains(&y) {
            return Err(ValueError::CoordinateRange);
        }
        Ok(Self { x, y })
    }
    pub const fn x(self) -> i32 {
        self.x
    }
    pub const fn y(self) -> i32 {
        self.y
    }
    pub(crate) fn within_synthetic_radius(self, center: Self) -> bool {
        let dx = i64::from(self.x) - i64::from(center.x);
        let dy = i64::from(self.y) - i64::from(center.y);
        dx * dx + dy * dy <= 100
    }
}

/// The assembler must not reuse a RunId while its inputs, tasks, events or
/// detached frame keys/cursors can still arrive. Equal revision tokens alone
/// do not identify shared storage or an independent run.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RunSource {
    pub run: RunId,
    pub transport: TransportEpoch,
    pub input: InputEpoch,
    pub revision: MapRevision,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PhysicalSlot {
    PrimaryOne,
    PrimaryTwo,
    SecondaryOne,
    SecondaryTwo,
}
impl PhysicalSlot {
    pub(crate) const fn index(self) -> usize {
        match self {
            Self::PrimaryOne => 0,
            Self::PrimaryTwo => 1,
            Self::SecondaryOne => 2,
            Self::SecondaryTwo => 3,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InputDriver {
    Manual,
    Automatic,
    Regression,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InputEdge {
    Press(Point),
    Release,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InputEvent {
    pub sequence: InputSequence,
    pub at: MapTime,
    pub slot: PhysicalSlot,
    pub edge: InputEdge,
}

/// A current-input-epoch release fact, not a timestamped gameplay edge.
/// Its sequence shares the admission fence with gameplay inputs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReleaseObservation {
    pub sequence: InputSequence,
    pub slot: PhysicalSlot,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorkBudget(u16);
impl WorkBudget {
    pub const fn new(value: u16) -> Result<Self, ValueError> {
        if value == 0 || value > 256 {
            Err(ValueError::InvalidBudget)
        } else {
            Ok(Self(value))
        }
    }
    pub const fn one() -> Self {
        Self(1)
    }
    pub const fn full() -> Self {
        Self(256)
    }
    pub const fn get(self) -> u16 {
        self.0
    }
}
