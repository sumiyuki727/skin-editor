//! Independent skin sound slots. No visual membership or decoder dependency.
use crate::{AssetId, AudioComponentId, ModelError};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AudioMetadata {
    sample_rate: u32,
    channels: u16,
    frames: u64,
}
impl AudioMetadata {
    pub fn new(sample_rate: u32, channels: u16, frames: u64) -> Result<Self, ModelError> {
        if sample_rate == 0 || channels == 0 || frames == 0 {
            return Err(ModelError::InvalidValue("audio_metadata"));
        }
        Ok(Self {
            sample_rate,
            channels,
            frames,
        })
    }
    pub const fn sample_rate(self) -> u32 {
        self.sample_rate
    }
    pub const fn channels(self) -> u16 {
        self.channels
    }
    pub const fn frames(self) -> u64 {
        self.frames
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Gain(f64);
impl Gain {
    pub const UNITY: Self = Self(1.0);
    pub fn new(value: f64) -> Result<Self, ModelError> {
        if !value.is_finite() || !(0.0..=1.0).contains(&value) {
            return Err(ModelError::InvalidValue("audio_gain"));
        }
        Ok(Self(if value == 0.0 { 0.0 } else { value }))
    }
    pub const fn get(self) -> f64 {
        self.0
    }
}
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SoundSlotKey(String);
impl SoundSlotKey {
    /// An editor identity, not a filename or a stable compatibility assertion.
    pub fn new(value: String) -> Result<Self, ModelError> {
        if value.trim().is_empty() || value.len() > 128 || value.chars().any(char::is_control) {
            return Err(ModelError::InvalidValue("sound_slot"));
        }
        Ok(Self(value))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct AudioComponent {
    id: AudioComponentId,
    slot: SoundSlotKey,
    asset: AssetId,
    gain: Gain,
    visible: bool,
}
impl AudioComponent {
    pub fn new(id: AudioComponentId, slot: SoundSlotKey, asset: AssetId) -> Self {
        Self {
            id,
            slot,
            asset,
            gain: Gain::UNITY,
            visible: true,
        }
    }
    /// Detached value construction; never an active project setter.
    pub fn with_properties(mut self, gain: Gain, visible: bool) -> Self {
        self.gain = gain;
        self.visible = visible;
        self
    }
    pub const fn id(&self) -> AudioComponentId {
        self.id
    }
    pub fn slot(&self) -> &SoundSlotKey {
        &self.slot
    }
    pub const fn asset(&self) -> AssetId {
        self.asset
    }
    pub const fn gain(&self) -> Gain {
        self.gain
    }
    pub const fn visible(&self) -> bool {
        self.visible
    }
}
