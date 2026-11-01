//! Internal applied/desired transport versions. No project or resource identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OutputFence(pub(crate) [u64; 2]);
impl OutputFence {
    pub(crate) fn with_scene(self, other: Self) -> Self {
        Self([other.0[0], self.0[1]])
    }
    pub(crate) fn with_audition(self, other: Self) -> Self {
        Self([self.0[0], other.0[1]])
    }
}
#[derive(Clone, Copy)]
pub(crate) struct OutputMark {
    pub skin: OutputFence,
    pub background: u64,
}
