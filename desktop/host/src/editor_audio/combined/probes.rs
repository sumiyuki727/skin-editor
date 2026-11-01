//! Bounded diagnostic correlation; contains no audio data or device handles.
use skin_editor_audio::{Channel, Origin, TriggerReceipt};
use skin_editor_infrastructure::separated_native::FirstCopy;
use std::time::{Duration, Instant};
const LIMIT: usize = 2496;
pub(super) struct Pending {
    id: u64,
    lane: Channel,
    at: Instant,
    origin: Origin,
    voice: Option<u64>,
    measured: bool,
}
pub(in crate::editor_audio) struct Complete {
    pub id: u64,
    pub lane: Channel,
    pub voice: u64,
    pub upper_ms: f64,
    pub joined_ms: f64,
    pub first: FirstCopy,
    pub measured: bool,
}
pub(in crate::editor_audio) struct Probes {
    pending: Vec<Pending>,
    pub complete: Vec<Complete>,
}
impl Default for Probes {
    fn default() -> Self {
        Self {
            pending: Vec::with_capacity(16),
            complete: Vec::with_capacity(LIMIT),
        }
    }
}
impl Probes {
    pub fn begin(
        &mut self,
        id: u64,
        lane: Channel,
        at: Instant,
        origin: Origin,
        measured: bool,
    ) -> Result<(), String> {
        if self.pending.len() == 16
            || self.pending.len() + self.complete.len() == LIMIT
            || self.pending.iter().any(|p| p.id == id)
            || self.complete.iter().any(|p| p.id == id)
        {
            return Err("probe admission exhausted or duplicate request".into());
        }
        self.pending.push(Pending {
            id,
            lane,
            at,
            origin,
            voice: None,
            measured,
        });
        Ok(())
    }
    pub fn receipt(&mut self, id: u64, r: TriggerReceipt) -> Result<(), String> {
        let p = self
            .pending
            .iter_mut()
            .find(|p| p.id == id)
            .ok_or("trigger receipt lacks request")?;
        if p.voice.is_some() || r.origin != p.origin || r.stale || r.silent {
            return Err("duplicate, stale or silent trigger receipt".into());
        }
        p.voice = Some(r.voice.ok_or("normal trigger refused")?);
        Ok(())
    }
    pub fn observe(&mut self, rows: &[(FirstCopy, Instant)], now: Instant) -> Result<(), String> {
        for index in (0..self.pending.len()).rev() {
            let p = &self.pending[index];
            if let Some(voice) = p.voice
                && let Some((first, observed)) = rows
                    .iter()
                    .find(|(f, _)| f.voice == voice && f.channel == p.lane)
            {
                let upper = observed
                    .checked_duration_since(p.at)
                    .ok_or("copy predates request")?;
                let joined = now
                    .checked_duration_since(*observed)
                    .ok_or("join predates observation")?;
                self.complete.push(Complete {
                    id: p.id,
                    lane: p.lane,
                    voice,
                    upper_ms: upper.as_secs_f64() * 1000.,
                    joined_ms: (upper + joined).as_secs_f64() * 1000.,
                    first: *first,
                    measured: p.measured,
                });
                self.pending.swap_remove(index);
            } else if now
                .checked_duration_since(p.at)
                .ok_or("probe clock reversed")?
                > Duration::from_secs(5)
            {
                return Err("exact request/Voice native observation timeout".into());
            }
        }
        Ok(())
    }
    pub fn empty(&self) -> bool {
        self.pending.is_empty()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn origin() -> Origin {
        Origin {
            session: 1,
            node: 2,
            revision: 3,
            navigation_revision: 4,
            draft: None,
        }
    }
    fn receipt(voice: u64) -> TriggerReceipt {
        TriggerReceipt {
            origin: origin(),
            stale: false,
            voice: Some(voice),
            queued: false,
            silent: false,
        }
    }
    fn first(voice: u64, lane: Channel) -> FirstCopy {
        FirstCopy {
            voice,
            channel: lane,
            callback_nanos: 12,
            callback_ordinal: 1,
            offset_frames: 0,
            callback_frames: 480,
        }
    }
    #[test]
    fn exact_request_and_channel_match_even_if_copy_precedes_receipt() {
        let at = Instant::now();
        let mut p = Probes::default();
        p.begin(1, Channel::Scene, at, origin(), true).unwrap();
        let rows = [
            (first(9, Channel::Audition), at + Duration::from_millis(1)),
            (first(9, Channel::Scene), at + Duration::from_millis(12)),
        ];
        p.observe(&rows, at + Duration::from_millis(13)).unwrap();
        assert!(!p.empty());
        p.receipt(1, receipt(9)).unwrap();
        p.observe(&rows, at + Duration::from_millis(80)).unwrap();
        assert!(p.empty());
        assert_eq!(p.complete[0].upper_ms, 12.);
        assert_eq!(p.complete[0].joined_ms, 80.);
    }
    #[test]
    fn wrong_origin_duplicate_and_unrelated_newer_voice_are_not_completion() {
        let at = Instant::now();
        let mut p = Probes::default();
        p.begin(1, Channel::Scene, at, origin(), true).unwrap();
        let mut r = receipt(3);
        r.origin.navigation_revision += 1;
        assert!(p.receipt(1, r).is_err());
        p.receipt(1, receipt(3)).unwrap();
        assert!(p.receipt(1, receipt(3)).is_err());
        assert!(p.begin(1, Channel::Scene, at, origin(), true).is_err());
        assert!(
            p.observe(
                &[(first(4, Channel::Scene), at)],
                at + Duration::from_secs(6)
            )
            .is_err()
        );
        assert!(!p.empty());
    }
    #[test]
    fn admission_is_bounded_before_any_receipt_and_cancelled_source_is_rejected() {
        let at = Instant::now();
        let mut p = Probes::default();
        for id in 1..=16 {
            p.begin(id, Channel::Scene, at, origin(), false).unwrap();
        }
        assert!(p.begin(17, Channel::Audition, at, origin(), false).is_err());
        let mut r = receipt(1);
        r.stale = true;
        assert!(p.receipt(1, r).is_err());
    }
}
