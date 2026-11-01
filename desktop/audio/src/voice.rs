//! Shared monitoring sampler and owner-authorised playback leases. No device I/O.
use crate::{PreparedSound, VoiceInfo};
use std::sync::{
    Arc, Weak,
    atomic::{AtomicBool, AtomicU32, Ordering},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MonitorFormat {
    rate: u32,
    channels: u16,
}
impl MonitorFormat {
    pub fn rate(self) -> u32 {
        self.rate
    }
    pub fn new(rate: u32, channels: u16) -> Option<Self> {
        (rate > 0 && matches!(channels, 1 | 2)).then_some(Self { rate, channels })
    }
}
impl PreparedSound {
    /// Add one bounded monitoring block, without clipping or changing PCM.
    /// Returns frames added; invalid target layout returns None before writing.
    /// Per-sample Voice addition order is the caller's responsibility.
    pub fn add_monitor_frames(
        &self,
        frame: u64,
        format: MonitorFormat,
        output: &mut [f32],
    ) -> Option<usize> {
        self.add_monitor_frames_with_gain(frame, format, output, self.monitor_gain())
    }
    pub fn add_monitor_frames_with_gain(
        &self,
        frame: u64,
        format: MonitorFormat,
        output: &mut [f32],
        gain: f32,
    ) -> Option<usize> {
        if !gain.is_finite() || !(0.0..=1.0).contains(&gain) {
            return None;
        }
        let channels = usize::from(format.channels);
        if !output.len().is_multiple_of(channels) {
            return None;
        }
        if output.is_empty() {
            return Some(0);
        }
        let (rate, source_channels) = self.format();
        if rate == format.rate && source_channels == format.channels {
            frame.checked_mul(u64::from(rate))?;
            let source = self.samples();
            let mut position = usize::try_from(frame).ok()?.checked_mul(channels)?;
            if position >= source.len() {
                return Some(0);
            }
            let before_overflow = (u64::MAX / u64::from(rate) - frame).saturating_add(1);
            let count = (output.len() / channels)
                .min((source.len() - position) / channels)
                .min(usize::try_from(before_overflow).unwrap_or(usize::MAX));
            let last = source.len() - channels;
            for target in output[..count * channels].chunks_exact_mut(channels) {
                let next = (position + channels).min(last);
                for (c, value) in target.iter_mut().enumerate() {
                    let a = source[position + c] * gain;
                    let b = source[next + c] * gain;
                    // Keep the zero-fraction interpolation, including signed zero.
                    *value += a + (b - a) * 0.0;
                }
                position += channels;
            }
            Some(count)
        } else {
            let mut count = 0;
            for target in output.chunks_exact_mut(channels) {
                let Some(samples) = frame
                    .checked_add(count as u64)
                    .and_then(|p| self.monitor_frame_with_gain(p, format, gain))
                else {
                    break;
                };
                for (c, value) in target.iter_mut().enumerate() {
                    *value += samples[c];
                }
                count += 1;
            }
            Some(count)
        }
    }
    #[inline]
    pub fn monitor_finished(&self, frame: u64, format: MonitorFormat) -> bool {
        let (rate, channels) = self.format();
        if !matches!(channels, 1 | 2) {
            return true;
        }
        frame.checked_mul(u64::from(rate)).is_none_or(|n| {
            let position = if rate == format.rate {
                frame
            } else {
                n / u64::from(format.rate)
            };
            position >= (self.samples().len() / usize::from(channels)) as u64
        })
    }
    /// Same linear interpolation / mono duplicate / stereo mean as the owner
    /// mixer. The output cursor counts frames, never source samples.
    #[inline]
    pub fn monitor_frame(&self, frame: u64, format: MonitorFormat) -> Option<[f32; 2]> {
        self.monitor_frame_with_gain(frame, format, self.monitor_gain())
    }
    #[inline]
    pub fn monitor_frame_with_gain(
        &self,
        frame: u64,
        format: MonitorFormat,
        gain: f32,
    ) -> Option<[f32; 2]> {
        if !gain.is_finite() || !(0.0..=1.0).contains(&gain) {
            return None;
        }
        let (rate, channels) = self.format();
        if !matches!(channels, 1 | 2) {
            return None;
        }
        let source_channels = usize::from(channels);
        let frames = self.samples().len() / source_channels;
        let numerator = frame.checked_mul(u64::from(rate))?;
        // Most ready skin sounds already match the device rate. Keep the same
        // overflow guard and interpolation arithmetic, without two divisions.
        let (position, fraction) = if rate == format.rate {
            (usize::try_from(frame).ok()?, 0.0)
        } else {
            (
                usize::try_from(numerator / u64::from(format.rate)).ok()?,
                (numerator % u64::from(format.rate)) as f32 / format.rate as f32,
            )
        };
        if position >= frames {
            return None;
        }
        let next = (position + 1).min(frames - 1);
        let sample = |channel: usize| {
            let c = channel.min(source_channels - 1);
            let a = self.samples()[position * source_channels + c] * gain;
            let b = self.samples()[next * source_channels + c] * gain;
            a + (b - a) * fraction
        };
        let mut output = [0.0; 2];
        for (c, value) in output
            .iter_mut()
            .take(usize::from(format.channels))
            .enumerate()
        {
            *value = if format.channels == 1 && channels == 2 {
                (sample(0) + sample(1)) * 0.5
            } else {
                sample(c)
            };
        }
        Some(output)
    }
}

/// Opaque identity of one Runtime execution choice, never a project/history ID.
#[derive(Clone, Debug)]
pub struct VoiceRuntimeKey(pub(crate) Weak<()>);
impl VoiceRuntimeKey {
    pub fn same_runtime(&self, other: &Self) -> bool {
        self.0.ptr_eq(&other.0)
    }
}

/// Only Runtime can revoke this lease; adapters can observe or share it. Arc
/// identity also prevents a completion for another Runtime reusing a Voice ID.
#[derive(Debug)]
pub(crate) struct VoiceControl {
    valid: AtomicBool,
    gain: AtomicU32,
}
#[derive(Debug)]
pub struct VoiceValidity(pub(crate) Arc<VoiceControl>);
impl Clone for VoiceValidity {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl VoiceValidity {
    pub(crate) fn new(gain: f32) -> Self {
        Self(Arc::new(VoiceControl {
            valid: AtomicBool::new(true),
            gain: AtomicU32::new(gain.to_bits()),
        }))
    }
    pub(crate) fn revoke(&self) {
        self.0.valid.store(false, Ordering::Release);
    }
    pub(crate) fn set_gain(&self, gain: f32) {
        self.0.gain.store(gain.to_bits(), Ordering::Release);
    }
    pub fn is_valid(&self) -> bool {
        self.0.valid.load(Ordering::Acquire)
    }
    pub fn gain(&self) -> f32 {
        f32::from_bits(self.0.gain.load(Ordering::Acquire))
    }
}
/// One immediate Voice accepted by the existing Runtime. Prepared PCM remains
/// immutable/shared; destruction and completion run on the owner, not rendering.
#[derive(Clone, Debug)]
pub struct VoiceHandoff {
    pub(crate) info: VoiceInfo,
    pub(crate) sound: PreparedSound,
    pub(crate) validity: VoiceValidity,
}
impl VoiceHandoff {
    pub fn info(&self) -> VoiceInfo {
        self.info
    }
    pub fn sound(&self) -> &PreparedSound {
        &self.sound
    }
    pub fn validity(&self) -> VoiceValidity {
        self.validity.clone()
    }
}

#[cfg(test)]
mod tests;
