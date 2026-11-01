//! Bounded writer for an explicit self-authored long native diagnostic only.
use skin_editor_audio::PcmBudget;
use skin_editor_infrastructure::background_stream::{BufferedBackground, StreamObservation};
use std::{
    fs::{File, OpenOptions},
    io::Write,
    path::Path,
};
pub const SECONDS: u32 = 360;
pub fn open(
    path: &Path,
    rate: u32,
    channels: u16,
    pool: &PcmBudget,
) -> Result<(BufferedBackground, StreamObservation), String> {
    let file = write(path, rate, channels)?;
    let source = BufferedBackground::open(File::open(file).map_err(|e| e.to_string())?, pool)
        .map_err(|e| format!("{e:?}"))?;
    let observer = source.observation();
    Ok((source, observer))
}
pub fn write(path: &Path, rate: u32, channels: u16) -> Result<std::path::PathBuf, String> {
    let file = path.join("self-background.wav");
    let size = SECONDS * rate * u32::from(channels) * 2;
    let mut writer = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&file)
        .map_err(|e| e.to_string())?;
    writer
        .write_all(b"RIFF")
        .and_then(|_| writer.write_all(&(36 + size).to_le_bytes()))
        .map_err(|e| e.to_string())?;
    writer.write_all(b"WAVEfmt ").map_err(|e| e.to_string())?;
    for b in [
        16u32.to_le_bytes().to_vec(),
        1u16.to_le_bytes().to_vec(),
        channels.to_le_bytes().to_vec(),
        rate.to_le_bytes().to_vec(),
        (rate * u32::from(channels) * 2).to_le_bytes().to_vec(),
        (channels * 2).to_le_bytes().to_vec(),
        16u16.to_le_bytes().to_vec(),
        b"data".to_vec(),
        size.to_le_bytes().to_vec(),
    ] {
        writer.write_all(&b).map_err(|e| e.to_string())?;
    }
    let mut block = [0u8; 8192];
    for pair in block.chunks_exact_mut(2) {
        pair.copy_from_slice(&1i16.to_le_bytes());
    }
    let mut left = size as usize;
    while left > 0 {
        let n = left.min(block.len());
        writer.write_all(&block[..n]).map_err(|e| e.to_string())?;
        left -= n;
    }
    writer.flush().map_err(|e| e.to_string())?;
    drop(writer);
    Ok(file)
}
