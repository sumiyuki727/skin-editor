//! Deliberately small RIFF/WAVE PCM16 adapter. No stable naming rules.
use crate::{AudioDecoder, BlobError, BlobStore, Cancellation, RawBlob};
use skin_editor_application::values::AudioMetadata;
use skin_editor_audio::PrepareFailure;

#[derive(Default)]
pub struct WavDecoder;
struct Wave<'a> {
    metadata: AudioMetadata,
    data: &'a [u8],
}
fn u16le(bytes: &[u8]) -> u16 {
    u16::from_le_bytes(bytes[..2].try_into().expect("checked field"))
}
fn u32le(bytes: &[u8]) -> u32 {
    u32::from_le_bytes(bytes[..4].try_into().expect("checked field"))
}
pub(crate) fn pcm16_format(chunk: &[u8]) -> Result<(u32, u16), PrepareFailure> {
    use PrepareFailure::{InvalidPcm, Unsupported};
    if chunk.len() < 16 {
        return Err(InvalidPcm);
    }
    if u16le(chunk) != 1 || u16le(&chunk[14..]) != 16 {
        return Err(Unsupported);
    }
    if chunk.len() != 16 && !(chunk.len() == 18 && u16le(&chunk[16..]) == 0) {
        return Err(Unsupported);
    }
    let channels = u16le(&chunk[2..]);
    let rate = u32le(&chunk[4..]);
    if !matches!(channels, 1 | 2) || !matches!(rate, 44100 | 48000) {
        return Err(Unsupported);
    }
    if u16le(&chunk[12..]) != channels * 2 || u32le(&chunk[8..]) != rate * u32::from(channels) * 2 {
        return Err(InvalidPcm);
    }
    Ok((rate, channels))
}
fn parse(bytes: &[u8]) -> Result<Wave<'_>, PrepareFailure> {
    use PrepareFailure::{InvalidPcm, Unsupported};
    if bytes.len() < 12 || &bytes[..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err(Unsupported);
    }
    if u32le(&bytes[4..8]) as u64 + 8 != bytes.len() as u64 {
        return Err(InvalidPcm);
    }
    let mut position = 12usize;
    let mut format = None;
    let mut data = None;
    let mut chunks = 0;
    while position < bytes.len() {
        chunks += 1;
        if chunks > 4096 {
            return Err(PrepareFailure::ResourceLimit);
        }
        let header = bytes.get(position..position + 8).ok_or(InvalidPcm)?;
        let length = u32le(&header[4..]) as usize;
        position += 8;
        let end = position.checked_add(length).ok_or(InvalidPcm)?;
        let chunk = bytes.get(position..end).ok_or(InvalidPcm)?;
        match &header[..4] {
            b"fmt " => {
                if format.is_some() || chunk.len() < 16 {
                    return Err(InvalidPcm);
                }
                format = Some(pcm16_format(chunk)?);
            }
            b"data" if data.is_some() => return Err(InvalidPcm),
            b"data" => data = Some(chunk),
            _ => {}
        }
        position = end.checked_add(length & 1).ok_or(InvalidPcm)?;
        if position > bytes.len() {
            return Err(InvalidPcm);
        }
    }
    let (rate, channels) = format.ok_or(InvalidPcm)?;
    let data = data.ok_or(InvalidPcm)?;
    let stride = usize::from(channels) * 2;
    if data.is_empty() || data.len() % stride != 0 {
        return Err(InvalidPcm);
    }
    let metadata =
        AudioMetadata::new(rate, channels, (data.len() / stride) as u64).map_err(|_| InvalidPcm)?;
    Ok(Wave { metadata, data })
}
impl AudioDecoder for WavDecoder {
    fn probe(&self, bytes: &[u8]) -> Result<AudioMetadata, PrepareFailure> {
        Ok(parse(bytes)?.metadata)
    }
    fn decode_into(
        &mut self,
        bytes: &[u8],
        metadata: AudioMetadata,
        output: &mut [f32],
        cancel: &Cancellation,
    ) -> Result<usize, PrepareFailure> {
        let wave = parse(bytes)?;
        if wave.metadata != metadata || output.len() != wave.data.len() / 2 {
            return Err(PrepareFailure::MetadataMismatch);
        }
        for (i, (sample, raw)) in output.iter_mut().zip(wave.data.chunks_exact(2)).enumerate() {
            if i % 1024 == 0 && cancel.is_cancelled() {
                return Err(PrepareFailure::Cancelled);
            }
            *sample = f32::from(i16::from_le_bytes([raw[0], raw[1]])) / 32768.0;
        }
        Ok(output.len())
    }
}
impl BlobStore {
    /// Explicit path only, read-only; unsupported bytes may separately be kept as Opaque.
    pub fn stage_wav(&self, path: &std::path::Path) -> Result<RawBlob, BlobError> {
        let file = std::fs::File::open(path).map_err(|_| BlobError::Io)?;
        let len = file.metadata().map_err(|_| BlobError::Io)?.len();
        let raw = self.stage_reader(file, len)?;
        let metadata = WavDecoder
            .probe(raw.bytes())
            .map_err(|_| BlobError::Unsupported)?;
        raw.attest_audio(metadata)?;
        Ok(raw)
    }
}
