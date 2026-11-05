/// Local raw work, separate from elapsed time and simulation observations.
/// An overflowing field keeps its old value and sets `valid = false`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorkCounts {
    pub valid: bool,
    pub record_reads: u64,
    pub record_copies: u64,
    pub index_initializations: u64,
    pub id_comparisons: u64,
    pub index_moves: u64,
    pub state_initializations: u64,
    pub search_objects: u64,
    pub inspection_objects: u64,
    pub automatic_objects: u64,
    pub publications: u64,
    pub summaries: u64,
    pub frame_objects: u64,
    pub ledger_objects: u64,
    pub management_steps: u64,
    pub allocation_attempts: u64,
    pub allocation_bytes: u64,
}

impl Default for WorkCounts {
    fn default() -> Self {
        Self {
            valid: true,
            record_reads: 0,
            record_copies: 0,
            index_initializations: 0,
            id_comparisons: 0,
            index_moves: 0,
            state_initializations: 0,
            search_objects: 0,
            inspection_objects: 0,
            automatic_objects: 0,
            publications: 0,
            summaries: 0,
            frame_objects: 0,
            ledger_objects: 0,
            management_steps: 0,
            allocation_attempts: 0,
            allocation_bytes: 0,
        }
    }
}
macro_rules! fields {
    ($action:ident, $left:expr, $right:expr) => {
        $action!($left, $right; record_reads, record_copies, index_initializations,
            id_comparisons, index_moves, state_initializations, search_objects,
            inspection_objects, automatic_objects, publications, summaries,
            frame_objects, ledger_objects, management_steps, allocation_attempts, allocation_bytes)
    };
}
impl WorkCounts {
    pub(crate) fn merge(&mut self, other: Self) {
        macro_rules! add {
            ($a:expr, $b:expr; $($field:ident),*) => { $(
                match $a.$field.checked_add($b.$field) {
                    Some(value) => $a.$field = value,
                    None => $a.valid = false,
                }
            )* };
        }
        self.valid &= other.valid;
        fields!(add, self, other);
    }
    pub(crate) fn allocation(&mut self, bytes: usize) {
        let mut delta = Self {
            allocation_attempts: 1,
            ..Self::default()
        };
        match u64::try_from(bytes) {
            Ok(bytes) => delta.allocation_bytes = bytes,
            Err(_) => delta.valid = false,
        }
        self.merge(delta);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn diagnostics_never_wrap_and_overflow_is_explicitly_invalid() {
        let mut counts = WorkCounts {
            frame_objects: u64::MAX,
            ..WorkCounts::default()
        };
        counts.merge(WorkCounts {
            frame_objects: 1,
            ..WorkCounts::default()
        });
        assert_eq!(counts.frame_objects, u64::MAX);
        assert!(!counts.valid);
        counts.merge(WorkCounts::default());
        assert!(!counts.valid);
    }
}
