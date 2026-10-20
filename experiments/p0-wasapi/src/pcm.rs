//! Bounded, platform-free diagnostic PCM; not an audio-file decoder or game mixer.
use crate::Error;
use std::sync::Arc;

pub const DURATION_US: u64 = 6_000_000;
pub const AMPLITUDE: f32 = 0.04;
const MAX_PACKET_BYTES: usize = 1024 * 1024;

/// Validated decoded samples. The adapter receives values, never a file or decoder.
pub struct Clip {
    rate: u32,
    channels: u16,
    samples: Arc<[f32]>,
    gain: f32,
}
impl Clip {
    pub fn new(rate: u32, channels: u16, samples: Arc<[f32]>, gain: f32) -> Result<Self, Error> {
        if !(8000..=384000).contains(&rate)
            || !(1..=2).contains(&channels)
            || samples.is_empty()
            || !samples.len().is_multiple_of(usize::from(channels))
            || !gain.is_finite()
            || !(0.0..=AMPLITUDE).contains(&gain)
            || samples.len() as u128 * 1_000_000
                > u128::from(DURATION_US) * u128::from(rate) * u128::from(channels)
            || samples.iter().any(|s| !s.is_finite() || s.abs() > 1.)
        {
            return Err(invalid());
        }
        Ok(Self {
            rate,
            channels,
            samples,
            gain,
        })
    }
    pub fn duration_us(&self) -> u64 {
        self.samples.len() as u64 / u64::from(self.channels) * 1_000_000 / u64::from(self.rate)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Packet {
    pub source_frame: u64,
    pub frames: u32,
    pub nonzero_samples: u64,
    pub hash: u64,
    pub peak: f32,
    pub probe: [f32; 8],
}
#[derive(Clone, Copy, Debug, Default)]
pub struct Stats {
    pub requested_us: u64,
    pub source_origin_frame: u64,
    pub submitted_frames: u64,
    pub packets: u64,
    pub nonzero_samples: u64,
    pub first: Option<Packet>,
    pub last: Option<Packet>,
}
pub struct Cursor {
    rate: u32,
    channels: u16,
    stats: Stats,
    clip: Option<Arc<Clip>>,
}
fn invalid() -> Error {
    Error {
        operation: "Invalid diagnostic PCM request",
        code: None,
    }
}
impl Cursor {
    pub fn new(rate: u32, channels: u16, position_us: u64) -> Result<Self, Error> {
        if !(8000..=384000).contains(&rate)
            || !(1..=2).contains(&channels)
            || position_us > DURATION_US
        {
            return Err(invalid());
        }
        Ok(Self {
            rate,
            channels,
            clip: None,
            stats: Stats {
                requested_us: position_us,
                source_origin_frame: (u128::from(position_us) * u128::from(rate) / 1_000_000)
                    as u64,
                ..Default::default()
            },
        })
    }
    /// Exact format match only; resampling and downmixing are separate future work.
    pub fn from_clip(
        rate: u32,
        channels: u16,
        position_us: u64,
        clip: Arc<Clip>,
    ) -> Result<Self, Error> {
        if rate != clip.rate || channels != clip.channels || position_us > clip.duration_us() {
            return Err(invalid());
        }
        let mut cursor = Self::new(rate, channels, position_us)?;
        cursor.clip = Some(clip);
        Ok(cursor)
    }
    pub fn stats(&self) -> Stats {
        self.stats
    }
    /// A stream clock that has passed all submitted frames has lost content/time
    /// alignment. One frame allows integer conversion at the clock boundary.
    pub fn check_playhead(&self, device_us: u64) -> Result<(), Error> {
        let consumed = u128::from(device_us) * u128::from(self.rate) / 1_000_000;
        if consumed > u128::from(self.stats.submitted_frames) + 1 {
            return Err(Error {
                operation: "Diagnostic PCM submission underrun",
                code: None,
            });
        }
        Ok(())
    }
    pub fn packet_bytes(&self, frames: u32) -> Result<usize, Error> {
        let bytes = usize::try_from(frames)
            .ok()
            .and_then(|n| n.checked_mul(usize::from(self.channels) * 4))
            .ok_or_else(invalid)?;
        if bytes == 0 || bytes > MAX_PACKET_BYTES {
            return Err(invalid());
        }
        Ok(bytes)
    }
    /// Initialize the entire provided packet. State advances only after backend acceptance.
    pub fn prepare(&self, frames: u32, output: &mut [u8]) -> Result<Packet, Error> {
        if output.len() != self.packet_bytes(frames)? {
            return Err(invalid());
        }
        let start = self
            .stats
            .source_origin_frame
            .checked_add(self.stats.submitted_frames)
            .ok_or_else(invalid)?;
        start.checked_add(u64::from(frames)).ok_or_else(invalid)?;
        let mut packet = Packet {
            source_frame: start,
            frames,
            hash: 0xcbf29ce484222325,
            ..Default::default()
        };
        for (frame, bytes) in output
            .chunks_exact_mut(usize::from(self.channels) * 4)
            .enumerate()
        {
            let generated = if self.clip.is_none() {
                sample(start + frame as u64, self.rate)
            } else {
                0.
            };
            for (channel, bytes) in bytes.chunks_exact_mut(4).enumerate() {
                let value = if let Some(clip) = &self.clip {
                    let index = (start + frame as u64)
                        .checked_mul(u64::from(self.channels))
                        .and_then(|n| n.checked_add(channel as u64))
                        .and_then(|n| usize::try_from(n).ok());
                    index
                        .and_then(|i| clip.samples.get(i))
                        .copied()
                        .unwrap_or(0.)
                        * clip.gain
                } else {
                    generated
                };
                if channel == 0 && frame < packet.probe.len() {
                    packet.probe[frame] = value;
                }
                packet.peak = packet.peak.max(value.abs());
                bytes.copy_from_slice(&value.to_le_bytes());
                packet.nonzero_samples += u64::from(value != 0.);
            }
        }
        for byte in output {
            packet.hash = (packet.hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3);
        }
        Ok(packet)
    }
    pub fn commit(&mut self, packet: Packet) -> Result<(), Error> {
        if packet.source_frame != self.stats.source_origin_frame + self.stats.submitted_frames
            || packet.nonzero_samples > u64::from(packet.frames) * u64::from(self.channels)
            || !packet.peak.is_finite()
            || !(0.0..=AMPLITUDE).contains(&packet.peak)
        {
            return Err(invalid());
        }
        self.packet_bytes(packet.frames)?;
        let submitted = self
            .stats
            .submitted_frames
            .checked_add(u64::from(packet.frames))
            .ok_or_else(invalid)?;
        let packets = self.stats.packets.checked_add(1).ok_or_else(invalid)?;
        let nonzero = self
            .stats
            .nonzero_samples
            .checked_add(packet.nonzero_samples)
            .ok_or_else(invalid)?;
        self.stats.submitted_frames = submitted;
        self.stats.packets = packets;
        self.stats.nonzero_samples = nonzero;
        self.stats.first.get_or_insert(packet);
        self.stats.last = Some(packet);
        Ok(())
    }
}
/// Three half-second regions identify location: 440/660/880 Hz, 200 ms tone
/// followed by 300 ms quiet, with 5 ms ramps. Six seconds, then zero padding.
fn sample(frame: u64, rate: u32) -> f32 {
    let t = frame as f64 / f64::from(rate);
    if t >= 6. {
        return 0.;
    }
    let local = t % 0.5;
    if local >= 0.2 {
        return 0.;
    }
    let frequency = [440., 660., 880.][((t / 0.5) as usize) % 3];
    let envelope = (local / 0.005).min((0.2 - local) / 0.005).clamp(0., 1.);
    (f64::from(AMPLITUDE) * envelope * (std::f64::consts::TAU * frequency * t).sin()) as f32
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn decoded_clip_preserves_stereo_gain_seek_eof_and_failed_submission() {
        let clip = Arc::new(Clip::new(8000, 2, Arc::from([0.5, -0.25, 1., -1.]), 0.04).unwrap());
        let mut c = Cursor::from_clip(8000, 2, 0, clip.clone()).unwrap();
        let mut b = [0; 24];
        let p = c.prepare(3, &mut b).unwrap();
        let values: Vec<_> = b
            .chunks_exact(4)
            .map(|v| f32::from_le_bytes(v.try_into().unwrap()))
            .collect();
        assert_eq!(values, vec![0.02, -0.01, 0.04, -0.04, 0., 0.]);
        assert_eq!(p.nonzero_samples, 4);
        assert_eq!(c.stats().submitted_frames, 0);
        assert_eq!(c.prepare(3, &mut b).unwrap().hash, p.hash);
        c.commit(p).unwrap();
        assert!(c.commit(p).is_err());
        let c = Cursor::from_clip(8000, 2, 125, clip.clone()).unwrap();
        let mut b = [0; 8];
        c.prepare(1, &mut b).unwrap();
        assert_eq!(f32::from_le_bytes(b[..4].try_into().unwrap()), 0.04);
        let c = Cursor::from_clip(8000, 2, 250, clip.clone()).unwrap();
        c.prepare(1, &mut b).unwrap();
        assert_eq!(b, [0; 8]);
        assert!(Cursor::from_clip(48000, 2, 0, clip.clone()).is_err());
        assert!(Cursor::from_clip(8000, 1, 0, clip.clone()).is_err());
        assert!(Cursor::from_clip(8000, 2, 251, clip).is_err());
    }
    #[test]
    fn decoded_clip_rejects_nonfinite_shape_gain_and_duration() {
        for samples in [
            vec![],
            vec![0., 0., 0.],
            vec![f32::NAN, 0.],
            vec![1.1, 0.],
            vec![0.; 8000 * 2 * 7],
        ] {
            assert!(Clip::new(8000, 2, samples.into(), 0.04).is_err());
        }
        for gain in [-1., f32::NAN, 0.041] {
            assert!(Clip::new(8000, 1, Arc::from([0.5]), gain).is_err());
        }
    }
    #[test]
    fn known_sample_channel_order_and_quiet_regions() {
        let c = Cursor::new(48000, 2, 12500).unwrap();
        let mut bytes = [0; 8];
        let p = c.prepare(1, &mut bytes).unwrap();
        assert_eq!(&bytes[..4], &bytes[4..]);
        // 440 Hz * 12.5 ms = 5.5 cycles, a zero crossing.
        assert!(f32::from_le_bytes(bytes[..4].try_into().unwrap()).abs() < 1e-6);
        assert!(p.peak <= AMPLITUDE);
        let c = Cursor::new(48000, 2, 18750).unwrap();
        c.prepare(1, &mut bytes).unwrap();
        assert!((f32::from_le_bytes(bytes[..4].try_into().unwrap()) - 0.04).abs() < 1e-7);
        for us in [200000, 300000, 450000, 6000000] {
            let c = Cursor::new(48000, 1, us).unwrap();
            let mut bytes = vec![1; 100 * 4];
            c.prepare(100, &mut bytes).unwrap();
            assert!(bytes.iter().all(|n| *n == 0));
        }
    }
    #[test]
    fn accepted_packets_are_contiguous_failed_submission_does_not_skip_and_seek_is_sample_quantized()
     {
        let mut c = Cursor::new(44100, 2, 123456).unwrap();
        let initial = c.stats();
        assert_eq!(initial.source_origin_frame, 5444);
        let mut bytes = vec![0; c.packet_bytes(4096).unwrap()];
        let first = c.prepare(4096, &mut bytes).unwrap();
        let retry = c.prepare(4096, &mut bytes).unwrap();
        assert_eq!(first.hash, retry.hash);
        assert_eq!(c.stats().submitted_frames, 0);
        c.commit(first).unwrap();
        let next = c.prepare(4096, &mut bytes).unwrap();
        assert_eq!(next.source_frame, first.source_frame + 4096);
        assert!(c.commit(first).is_err());
        c.commit(next).unwrap();
        assert_eq!(c.stats().submitted_frames, 8192);
        assert!(c.stats().nonzero_samples > 0);
        let relocated = Cursor::new(44100, 2, 500000).unwrap();
        assert_eq!(relocated.stats().source_origin_frame, 22050);
    }
    #[test]
    fn rejects_bad_shape_rate_position_and_allocation_before_writing() {
        for (rate, ch, us) in [
            (0, 2, 0),
            (48000, 0, 0),
            (48000, 3, 0),
            (48000, 2, DURATION_US + 1),
        ] {
            assert!(Cursor::new(rate, ch, us).is_err());
        }
        let c = Cursor::new(48000, 2, 0).unwrap();
        let mut bytes = [9; 7];
        assert!(c.prepare(1, &mut bytes).is_err());
        assert_eq!(bytes, [9; 7]);
        assert!(c.packet_bytes(0).is_err());
        assert!(c.packet_bytes(u32::MAX).is_err());
    }
    #[test]
    fn submission_frontier_is_not_playhead_and_underflow_requires_recovery() {
        let mut c = Cursor::new(48000, 2, 500000).unwrap();
        let mut bytes = vec![0; c.packet_bytes(4800).unwrap()];
        c.commit(c.prepare(4800, &mut bytes).unwrap()).unwrap();
        assert!(c.check_playhead(0).is_ok());
        assert!(c.check_playhead(50000).is_ok());
        assert!(c.check_playhead(100000).is_ok());
        assert!(c.check_playhead(100100).is_err());
        assert_eq!(c.stats().submitted_frames, 4800);
    }
}
