//! Bounded byte-to-PCM adapter. No filesystem, native audio, UI or project writes.
use std::{io::Cursor, sync::Arc};
use symphonia::core::{
    codecs::audio::AudioDecoderOptions,
    common::Limit,
    formats::{FormatOptions, TrackType, probe::Hint},
    io::MediaSourceStream,
    meta::MetadataOptions,
};

pub const MAX_ENCODED_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_DECODED_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_DURATION_US: u64 = 6_000_000;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecodeError {
    Limit,
    Container,
    Unsupported,
    Decode,
    InvalidSamples,
}
#[derive(Clone, Debug)]
pub struct Clip {
    rate: u32,
    channels: u16,
    samples: Arc<[f32]>,
}
impl Clip {
    pub fn rate(&self) -> u32 {
        self.rate
    }
    pub fn channels(&self) -> u16 {
        self.channels
    }
    pub fn into_samples(self) -> Arc<[f32]> {
        self.samples
    }
    pub fn frames(&self) -> usize {
        self.samples.len() / usize::from(self.channels)
    }
    pub fn duration_us(&self) -> u64 {
        self.frames() as u64 * 1_000_000 / u64::from(self.rate)
    }
}

/// Only RIFF/WAVE in this slice. Errors never yield a partial playable clip.
pub fn decode_wav(bytes: Vec<u8>) -> Result<Clip, DecodeError> {
    if bytes.len() > MAX_ENCODED_BYTES {
        return Err(DecodeError::Limit);
    }
    if bytes.len() < 12
        || &bytes[..4] != b"RIFF"
        || &bytes[8..12] != b"WAVE"
        || u64::from(u32::from_le_bytes(bytes[4..8].try_into().unwrap())) + 8 != bytes.len() as u64
    {
        return Err(DecodeError::Container);
    }
    let source = MediaSourceStream::new(Box::new(Cursor::new(bytes)), Default::default());
    let metadata = MetadataOptions::default()
        .limit_tag_bytes(Limit::Maximum(0))
        .limit_visual_bytes(Limit::Maximum(0));
    let mut reader = symphonia::default::get_probe()
        .probe(&Hint::new(), source, FormatOptions::default(), metadata)
        .map_err(|_| DecodeError::Decode)?;
    let track = reader
        .default_track(TrackType::Audio)
        .ok_or(DecodeError::Unsupported)?;
    let params = track
        .codec_params
        .as_ref()
        .and_then(|p| p.audio())
        .ok_or(DecodeError::Unsupported)?;
    let rate = params.sample_rate.ok_or(DecodeError::Unsupported)?;
    let channels = params
        .channels
        .as_ref()
        .ok_or(DecodeError::Unsupported)?
        .count();
    if !(8000..=192000).contains(&rate) || !(1..=2).contains(&channels) {
        return Err(DecodeError::Unsupported);
    }
    let maximum_frames = u64::from(rate) * MAX_DURATION_US / 1_000_000;
    // RIFF/PCM has an exact frame count. Do not treat a truncated tail as clean EOF.
    let expected_frames = track.num_frames.ok_or(DecodeError::Unsupported)?;
    if expected_frames == 0
        || expected_frames > maximum_frames
        || expected_frames * channels as u64 * 4 > MAX_DECODED_BYTES as u64
    {
        return Err(DecodeError::Limit);
    }
    let id = track.id;
    let mut decoder = symphonia::default::get_codecs()
        .make_audio_decoder(params, &AudioDecoderOptions::default())
        .map_err(|_| DecodeError::Unsupported)?;
    let mut samples = Vec::<f32>::new();
    let mut packets = 0usize;
    while let Some(packet) = reader.next_packet().map_err(|_| DecodeError::Decode)? {
        packets += 1;
        if packets > 65536 {
            return Err(DecodeError::Limit);
        }
        if packet.track_id != id {
            return Err(DecodeError::Unsupported);
        }
        let buffer = decoder.decode(&packet).map_err(|_| DecodeError::Decode)?;
        if buffer.spec().rate() != rate || buffer.spec().channels().count() != channels {
            return Err(DecodeError::Unsupported);
        }
        let total = samples
            .len()
            .checked_add(buffer.samples_interleaved())
            .ok_or(DecodeError::Limit)?;
        if total > MAX_DECODED_BYTES / 4 || total > expected_frames as usize * channels {
            return Err(DecodeError::Limit);
        }
        let old = samples.len();
        samples
            .try_reserve(buffer.samples_interleaved())
            .map_err(|_| DecodeError::Limit)?;
        samples.resize(total, 0.);
        buffer.copy_to_slice_interleaved(&mut samples[old..]);
        if samples[old..]
            .iter()
            .any(|v| !v.is_finite() || v.abs() > 1.)
        {
            return Err(DecodeError::InvalidSamples);
        }
    }
    if samples.len() != expected_frames as usize * channels {
        return Err(DecodeError::Decode);
    }
    Ok(Clip {
        rate,
        channels: channels as u16,
        samples: samples.into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn wav(data: &[i16], rate: u32, channels: u16) -> Vec<u8> {
        let length = (data.len() * 2) as u32;
        let mut b = Vec::new();
        b.extend(b"RIFF");
        b.extend((36 + length).to_le_bytes());
        b.extend(b"WAVEfmt ");
        b.extend(16u32.to_le_bytes());
        b.extend(1u16.to_le_bytes());
        b.extend(channels.to_le_bytes());
        b.extend(rate.to_le_bytes());
        b.extend((rate * u32::from(channels) * 2).to_le_bytes());
        b.extend((channels * 2).to_le_bytes());
        b.extend(16u16.to_le_bytes());
        b.extend(b"data");
        b.extend(length.to_le_bytes());
        for s in data {
            b.extend(s.to_le_bytes());
        }
        b
    }
    #[test]
    fn pcm16_preserves_channel_order_normalization_and_source_bytes() {
        let b = wav(&[0, 32767, -32768, 16384], 48000, 2);
        let original = b.clone();
        let c = decode_wav(b).unwrap();
        assert_eq!((c.rate, c.channels, c.frames()), (48000, 2, 2));
        assert_eq!(c.samples.as_ref(), &[0., 32767. / 32768., -1., 0.5]);
        assert_eq!(decode_wav(original).unwrap().samples, c.samples);
    }
    #[test]
    fn corrupt_truncated_empty_and_unsupported_shapes_never_publish_partial_pcm() {
        let good = wav(&[1; 80], 8000, 1);
        assert!(decode_wav(good.clone()).is_ok());
        let mut truncated = good;
        truncated.pop();
        assert_eq!(decode_wav(truncated).unwrap_err(), DecodeError::Container);
        let mut short = wav(&[1; 80], 8000, 1);
        short.truncate(short.len() - 2);
        let size = (short.len() - 8) as u32;
        short[4..8].copy_from_slice(&size.to_le_bytes());
        assert!(decode_wav(short).is_err());
        for b in [
            b"not audio".to_vec(),
            wav(&[], 8000, 1),
            wav(&[1; 12], 48000, 3),
            wav(&[1; 80], 4000, 1),
        ] {
            assert!(decode_wav(b).is_err());
        }
    }
    #[test]
    fn limits_reject_oversized_input_and_duration_before_pcm_collection() {
        assert_eq!(
            decode_wav(vec![0; MAX_ENCODED_BYTES + 1]).unwrap_err(),
            DecodeError::Limit
        );
        assert_eq!(
            decode_wav(wav(&vec![0; 8000 * 7], 8000, 1)).unwrap_err(),
            DecodeError::Limit
        );
    }
}
