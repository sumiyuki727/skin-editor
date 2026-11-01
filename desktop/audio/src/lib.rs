//! Device-free recipe shared by preview preparation and eventual export.
//! Slot/file lookup, codecs and playback are adapter concerns.
#![forbid(unsafe_code)]
mod background;
mod pcm;
pub use background::BackgroundTrack;
mod audition_edit;
mod runtime;
pub use audition_edit::{AuditionChange, AuditionEdits};
mod voice;
pub use voice::{MonitorFormat, VoiceHandoff, VoiceRuntimeKey, VoiceValidity};
// Executable design experiment only; no alternative product runtime/API yet.
#[cfg(test)]
mod separated_model;
pub use pcm::*;
pub use runtime::*;
use skin_editor_domain::{
    AssetId, AssetKind, AudioComponentId, AudioMetadata, BlobRef, Gain, SkinProject,
};

pub const SOUND_PLAN_VERSION: u16 = 2;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum SoundTarget {
    Component(AudioComponentId),
    Library(AssetId),
}
impl From<AudioComponentId> for SoundTarget {
    fn from(id: AudioComponentId) -> Self {
        Self::Component(id)
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SoundPlan {
    pub target: SoundTarget,
    pub source: BlobRef,
    pub metadata: Option<AudioMetadata>,
    pub gain: Gain,
    pub visible: bool,
}

/// No omit result: missing, hidden and unsupported are different states.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OutputAction {
    CopyOriginal(BlobRef),
    ProcessPcm,
    /// Requires a valid encoded silent file, never an absent/zero-byte file.
    /// A verified format/slot policy must be supplied by the export adapter.
    EncodeSilence,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AudioError {
    MissingComponent,
    MissingAsset,
    ImageAsset,
    UnsupportedDecode,
    InvalidSamples,
    ResourceLimit,
    WrongSource,
    ProcessingRequired,
}

impl SoundPlan {
    pub fn build(project: &SkinProject, component: AudioComponentId) -> Result<Self, AudioError> {
        let component = project
            .audio_component(component)
            .ok_or(AudioError::MissingComponent)?;
        let asset = project
            .assets()
            .get(component.asset())
            .ok_or(AudioError::MissingAsset)?;
        let metadata = match asset.kind() {
            AssetKind::Audio(info) => Some(*info),
            AssetKind::Opaque => None,
            AssetKind::RasterImage(_) => return Err(AudioError::ImageAsset),
        };
        Ok(Self {
            target: SoundTarget::Component(component.id()),
            source: asset.blob(),
            metadata,
            gain: component.gain(),
            visible: component.visible(),
        })
    }

    pub fn output_action(&self) -> Result<OutputAction, AudioError> {
        if !self.visible {
            return Ok(OutputAction::EncodeSilence);
        }
        if self.gain == Gain::UNITY {
            return Ok(OutputAction::CopyOriginal(self.source));
        }
        if self.metadata.is_none() {
            return Err(AudioError::UnsupportedDecode);
        }
        Ok(OutputAction::ProcessPcm)
    }

    /// Returns exact staged bytes without decoding/re-encoding. Infrastructure
    /// must verify the reference/hash before calling, as with all blob leases.
    pub fn original_bytes<'a>(
        &self,
        reference: BlobRef,
        bytes: &'a [u8],
    ) -> Result<&'a [u8], AudioError> {
        if self.output_action()? != OutputAction::CopyOriginal(self.source) {
            return Err(AudioError::ProcessingRequired);
        }
        if reference != self.source || bytes.len() as u64 != self.source.byte_len() {
            return Err(AudioError::WrongSource);
        }
        Ok(bytes)
    }

    /// Shared preview/export gain on decoded interleaved source samples.
    /// Device volume, conversion and quantisation are separate policies.
    /// Check the caller's allocation budget before creating a processed copy.
    pub fn process_pcm(&self, samples: &[f32], max_samples: usize) -> Result<Vec<f32>, AudioError> {
        let metadata = self.metadata.ok_or(AudioError::UnsupportedDecode)?;
        let expected = metadata
            .frames()
            .checked_mul(u64::from(metadata.channels()))
            .ok_or(AudioError::ResourceLimit)?;
        if expected > max_samples as u64 {
            return Err(AudioError::ResourceLimit);
        }
        if expected != samples.len() as u64
            || samples.iter().any(|s| !s.is_finite() || s.abs() > 1.0)
        {
            return Err(AudioError::InvalidSamples);
        }
        let gain = if self.visible {
            self.gain.get() as f32
        } else {
            0.0
        };
        Ok(samples.iter().map(|sample| sample * gain).collect())
    }
}
