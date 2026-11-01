//! A bounded, external PCM reference timeline, independent of skin bindings.
use crate::{PcmBuffer, PcmError};
pub struct BackgroundTrack {
    pcm: PcmBuffer,
    rate: u32,
    channels: u16,
    next: u64,
    running: bool,
}

impl BackgroundTrack {
    pub fn new(pcm: PcmBuffer, rate: u32, channels: u16) -> Result<Self, PcmError> {
        if !matches!(rate, 44100 | 48000)
            || !matches!(channels, 1 | 2)
            || !pcm.samples().len().is_multiple_of(usize::from(channels))
            || pcm
                .samples()
                .iter()
                .any(|v| !v.is_finite() || v.abs() > 1.0)
        {
            return Err(PcmError::ResourceLimit);
        }
        Ok(Self {
            pcm,
            rate,
            channels,
            next: 0,
            running: false,
        })
    }
    pub fn format(&self) -> (u32, u16) {
        (self.rate, self.channels)
    }
    pub fn pcm(&self) -> &PcmBuffer {
        &self.pcm
    }
    pub fn position(&self) -> u64 {
        self.next
    }
    pub fn frames(&self) -> u64 {
        (self.pcm.samples().len() / usize::from(self.channels)) as u64
    }
    pub fn is_running(&self) -> bool {
        self.running
    }
    pub fn set_running(&mut self, running: bool) {
        self.running = running;
    }
    /// Used after callback freeze confirmation, never for a skin content update.
    pub fn restore_position(&mut self, next: u64) -> Result<(), PcmError> {
        if next > self.frames() {
            return Err(PcmError::ResourceLimit);
        }
        self.next = next;
        Ok(())
    }
    pub fn next_frame(&mut self) -> Option<(u64, [f32; 2])> {
        if !self.running || self.next == self.frames() {
            return None;
        }
        let frame = self.next;
        let i = frame as usize * usize::from(self.channels);
        let sample = [
            self.pcm.samples()[i],
            self.pcm.samples()[i + usize::from(self.channels - 1)],
        ];
        self.next += 1;
        Some((frame, sample))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PcmBudget;
    #[test]
    fn pause_restore_and_eof_preserve_the_shared_reference() {
        let budget = PcmBudget::default();
        let mut pcm = budget.reserve(4).unwrap().allocate();
        pcm.samples_mut()
            .unwrap()
            .copy_from_slice(&[0.1, 0.2, 0.3, 0.4]);
        let original = pcm.clone();
        let mut track = BackgroundTrack::new(pcm, 48000, 2).unwrap();
        assert!(track.pcm().same_allocation(&original));
        assert_eq!(budget.usage().bytes, 16);
        assert_eq!(track.next_frame(), None);
        track.set_running(true);
        assert_eq!(track.next_frame(), Some((0, [0.1, 0.2])));
        track.set_running(false);
        assert_eq!(track.next_frame(), None);
        assert_eq!(track.position(), 1);
        assert!(track.restore_position(3).is_err());
        assert_eq!(track.position(), 1);
        track.restore_position(0).unwrap();
        track.set_running(true);
        assert_eq!(track.next_frame(), Some((0, [0.1, 0.2])));
        assert_eq!(track.next_frame(), Some((1, [0.3, 0.4])));
        assert_eq!(track.next_frame(), None);
        assert_eq!(track.position(), 2);
        drop(track);
        assert_eq!(budget.usage().bytes, 16);
        assert_eq!(original.samples(), [0.1, 0.2, 0.3, 0.4]);
        drop(original);
        assert_eq!(budget.usage().bytes, 0);
    }
    #[test]
    fn invalid_reference_never_becomes_a_timeline() {
        let budget = PcmBudget::default();
        for (rate, channels, values) in [
            (32000, 1, vec![0.0]),
            (48000, 0, vec![0.0]),
            (48000, 2, vec![0.0]),
            (48000, 1, vec![f32::NAN]),
            (48000, 1, vec![1.01]),
        ] {
            let mut pcm = budget.reserve(values.len()).unwrap().allocate();
            pcm.samples_mut().unwrap().copy_from_slice(&values);
            assert!(BackgroundTrack::new(pcm, rate, channels).is_err());
            assert_eq!(budget.usage().bytes, 0);
        }
        assert!(budget.reserve(4 * 1024 * 1024 / 4 + 1).is_err());
        let mut track =
            BackgroundTrack::new(budget.reserve(1).unwrap().allocate(), 44100, 1).unwrap();
        track.set_running(true);
        assert_eq!(track.next_frame(), Some((0, [0.0; 2])));
    }
}
