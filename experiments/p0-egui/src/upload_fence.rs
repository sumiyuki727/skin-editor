//! P0 frame receipts; no GPU types or document state.
use std::collections::BTreeMap;
#[derive(Default)]
pub struct UploadFence {
    next: u64,
    pending: BTreeMap<u64, (u64, usize, bool)>,
    pub peak_bytes: usize,
    pub completed: usize,
}
impl UploadFence {
    pub fn bytes(&self) -> usize {
        self.pending.values().map(|v| v.1).sum()
    }
    pub fn allows(&self, frame: u64) -> bool {
        self.pending.values().all(|v| v.0 == frame)
    }
    pub fn queued(&mut self, frame: u64, bytes: usize) -> u64 {
        assert!(bytes > 0);
        let existing = self
            .pending
            .iter()
            .find(|(_, v)| v.0 == frame)
            .map(|(&k, _)| k);
        let ticket = existing.unwrap_or_else(|| {
            self.next += 1;
            self.pending.insert(self.next, (frame, 0, false));
            self.next
        });
        let item = self.pending.get_mut(&ticket).unwrap();
        assert!(!item.2);
        item.1 += bytes;
        self.peak_bytes = self.peak_bytes.max(self.bytes());
        ticket
    }
    pub fn arm_before(&mut self, frame: u64) -> Vec<u64> {
        self.pending
            .iter_mut()
            .filter_map(|(&ticket, v)| {
                if v.0 < frame && !v.2 {
                    v.2 = true;
                    Some(ticket)
                } else {
                    None
                }
            })
            .collect()
    }
    pub fn complete(&mut self, ticket: u64) -> Result<usize, &'static str> {
        if !self.pending.get(&ticket).is_some_and(|v| v.2) {
            return Err("Unknown, duplicate or unsubmitted receipt");
        }
        let bytes = self.pending.remove(&ticket).unwrap().1;
        self.completed += 1;
        Ok(bytes)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn blocks_later_frames_until_completion() {
        let mut f = UploadFence::default();
        let t = f.queued(10, 4);
        assert_eq!(f.queued(10, 4), t);
        assert!(f.allows(10));
        assert!(!f.allows(11));
        assert!(f.arm_before(10).is_empty());
        assert!(f.complete(t).is_err());
        assert_eq!(f.arm_before(11), vec![t]);
        assert!(f.arm_before(12).is_empty());
        assert!(!f.allows(12));
        assert_eq!(f.complete(t), Ok(8));
        assert!(f.allows(12));
        assert!(f.complete(t).is_err());
    }
    #[test]
    fn cancellation_cannot_erase_physical_receipts() {
        let mut f = UploadFence::default();
        let a = f.queued(1, 8);
        let b = f.queued(2, 4);
        assert_eq!(f.arm_before(3), vec![a, b]);
        assert!(f.complete(999).is_err());
        assert_eq!(f.complete(b), Ok(4));
        assert_eq!(f.bytes(), 8);
        assert!(!f.allows(3));
        assert_eq!(f.complete(a), Ok(8));
        assert_eq!((f.bytes(), f.peak_bytes, f.completed), (0, 12, 2));
    }
}
