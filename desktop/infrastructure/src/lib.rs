//! PNG derived cache and platform staging; raw bytes have a separate shared store.
#![forbid(unsafe_code)]
mod audio_playback;
mod audio_preparation;
mod audio_supply;
mod audio_worker;
mod background_input;
/// Bounded PCM16 WAV background reader; never used by the device callback.
pub mod background_stream;
mod callback_audit;
/// Versioned detached project-state blocks; no active-session or filesystem writer.
pub mod project_format;
mod project_identity;
pub use audio_supply::{
    AudioSupplyClient, MAX_SUPPLY_CONTROLS, SupplyError, SupplyOutcome, SupplyReceipt, SupplyStatus,
};
pub use project_identity::RandomHistoryIdentities;
mod native_audio;
mod output_fence;
/// CPAL executor used by the current Windows policy; other platforms remain unverified.
pub mod separated_native;
/// Independent background reserve and voices used by the Windows separated route.
pub mod separated_output;
mod submission_trace;
mod supply_output;
/// Bounded ready-voice transport used by separated output; not a project interface.
pub mod voice_handoff;
pub use submission_trace::{StreamClockSample, SubmissionTrace, VoiceSubmission};
mod wav;
pub use audio_playback::AudioPlayback;
pub use audio_worker::PreparationWorker;
pub use native_audio::*;
pub use wav::WavDecoder;
mod blobs;
mod byte_budget;
pub use audio_preparation::{
    Admission, AudioDecoder, AudioPreparationQueue, Cancellation, FinishedPreparation,
    MAX_PENDING_PREPARATIONS, PreparationError, PreparationObservation, PreparationUsage,
    RunningPreparation,
};
mod skin_audio;
pub use blobs::{BlobError, BlobStore, BlobUsage, RawBlob};
use byte_budget::{ByteBudget, ByteLease};
pub use byte_budget::{ByteObservation, ByteUsage};
use image::{ImageDecoder, ImageEncoder};
pub use skin_audio::{AdmissionState, BuildAdmission, SkinAudioController};
use skin_editor_application::{
    BlobLease, ImportError, RasterImportPort, ResourceError, ResourceStore, StagedRaster,
    values::{AudioMetadata, BlobRef, PixelSize},
};
use skin_editor_rendering::{Raster, RasterPixels};
use std::collections::BTreeMap;
use std::io::Cursor;
use std::sync::{Arc, Mutex};

struct CachedRaster {
    raster: Arc<Raster>,
}
#[derive(Clone)]
pub struct Images {
    blobs: BlobStore,
    entries: Arc<Mutex<BTreeMap<BlobRef, CachedRaster>>>,
    rgba: Arc<ByteBudget>,
    cache_target: usize,
}
impl Default for Images {
    fn default() -> Self {
        Self::with_store(BlobStore::default())
    }
}
impl ResourceStore for Images {
    fn pin(&self, blob: BlobRef) -> Result<Arc<dyn BlobLease>, ResourceError> {
        self.blobs.pin(blob)
    }
    fn audio_metadata(&self, blob: BlobRef) -> Option<AudioMetadata> {
        self.blobs.audio_metadata(blob)
    }
}
fn blob_import(error: BlobError) -> ImportError {
    match error {
        BlobError::ResourceLimit => ImportError::ResourceLimit,
        _ => ImportError::Io(format!("raw store: {error:?}")),
    }
}
impl Images {
    pub fn observe_rgba(&self) -> ByteObservation {
        self.rgba.observation()
    }
    pub fn with_store(blobs: BlobStore) -> Self {
        Self {
            blobs,
            entries: Arc::default(),
            rgba: ByteBudget::new(80 * 1024 * 1024),
            cache_target: 64 * 1024 * 1024,
        }
    }
    pub fn blobs(&self) -> &BlobStore {
        &self.blobs
    }
    fn prune(&self, entries: &mut BTreeMap<BlobRef, CachedRaster>) {
        entries.retain(|blob, c| {
            self.blobs.get(*blob).is_some()
                || Arc::strong_count(&c.raster) > 1
                || c.raster.pixels.is_shared()
        });
    }
    pub fn raster(&self, blob: BlobRef) -> Option<Arc<Raster>> {
        let mut entries = self.entries.lock().ok()?;
        self.prune(&mut entries);
        entries.get(&blob).map(|c| c.raster.clone())
    }
    pub fn rgba_bytes(&self) -> usize {
        if let Ok(mut entries) = self.entries.lock() {
            self.prune(&mut entries);
        }
        self.rgba.used()
    }
    pub fn rgba_peak_bytes(&self) -> usize {
        self.rgba.peak()
    }
    /// Rebuild an evicted derivative from history-pinned original bytes. Callers
    /// retain returned Arcs for the complete plan so admission cannot evict them.
    /// This bounded synchronous fallback is explicit; it is not a decode worker.
    pub fn load_raster(&self, blob: BlobRef) -> Result<Arc<Raster>, ImportError> {
        if let Some(raster) = self.raster(blob) {
            return Ok(raster);
        }
        let raw = self.blobs.get(blob).ok_or(ImportError::InvalidImage)?;
        self.prepare(raw, String::new())?;
        self.raster(blob).ok_or(ImportError::ResourceLimit)
    }
    fn reserve_pixels(&self, bytes: usize) -> Result<(ByteLease, ByteLease), ImportError> {
        let mut entries = self
            .entries
            .lock()
            .map_err(|_| ImportError::Io("image cache unavailable".into()))?;
        self.prune(&mut entries);
        while self.rgba.used().saturating_add(bytes) > self.cache_target {
            let victim = entries.iter().find_map(|(key, c)| {
                (Arc::strong_count(&c.raster) == 1 && !c.raster.pixels.is_shared()).then_some(*key)
            });
            if let Some(key) = victim {
                entries.remove(&key);
            } else {
                break;
            }
        }
        loop {
            if let Some(resident) = self.rgba.reserve(bytes) {
                if let Some(scratch) = self.rgba.reserve(bytes) {
                    return Ok((resident, scratch));
                }
                drop(resident);
            }
            // Raw-history leases do not pin reconstructible pixels. Never
            // evict a derivative still borrowed by a render/thumbnail caller.
            let victim = entries.iter().find_map(|(key, c)| {
                (Arc::strong_count(&c.raster) == 1 && !c.raster.pixels.is_shared()).then_some(*key)
            });
            match victim {
                Some(key) => {
                    entries.remove(&key);
                }
                None => return Err(ImportError::ResourceLimit),
            }
        }
    }
    pub fn stage_bytes(
        &mut self,
        bytes: Vec<u8>,
        label: String,
    ) -> Result<StagedRaster, ImportError> {
        let raw = self.blobs.stage_bytes(bytes).map_err(blob_import)?;
        self.prepare(raw, label)
    }
    fn prepare(&self, raw: RawBlob, label: String) -> Result<StagedRaster, ImportError> {
        let blob = raw.reference();
        if let Some(raster) = self.raster(blob) {
            return Ok(StagedRaster {
                blob,
                size: raster.size,
                label,
                lease: raw.lease(),
            });
        }
        let decoder = image::codecs::png::PngDecoder::new(Cursor::new(raw.bytes()))
            .map_err(|_| ImportError::InvalidImage)?;
        let (w, h) = decoder.dimensions();
        if w == 0
            || h == 0
            || w > 4096
            || h > 4096
            || u64::from(w) * u64::from(h) * 4 > 64 * 1024 * 1024
        {
            return Err(ImportError::ResourceLimit);
        }
        let bytes = (u64::from(w) * u64::from(h) * 4) as usize;
        // Vec -> Arc<[u8]> may briefly coexist; reserve the conversion copy too.
        let (resident, scratch) = self.reserve_pixels(bytes)?;
        let rgba = image::DynamicImage::from_decoder(decoder)
            .map_err(|_| ImportError::InvalidImage)?
            .into_rgba8();
        let size = PixelSize::new(w, h).map_err(|_| ImportError::InvalidImage)?;
        let raster = Arc::new(Raster {
            size,
            pixels: RasterPixels::with_residency(rgba.into_raw(), resident),
        });
        drop(scratch);
        let mut entries = self
            .entries
            .lock()
            .map_err(|_| ImportError::Io("image cache unavailable".into()))?;
        self.prune(&mut entries);
        entries.entry(blob).or_insert(CachedRaster { raster });
        Ok(StagedRaster {
            blob,
            size,
            label,
            lease: raw.lease(),
        })
    }
    pub fn sample_png() -> Vec<u8> {
        let mut pixels = vec![0u8; 64 * 64 * 4];
        for y in 0..64 {
            for x in 0..64 {
                let i = (y * 64 + x) * 4;
                let radius = (x as f32 - 31.5).hypot(y as f32 - 31.5);
                if radius < 27.0 {
                    pixels[i..i + 4].copy_from_slice(&[
                        255,
                        110,
                        165,
                        if radius > 23.0 { 255 } else { 150 },
                    ]);
                }
            }
        }
        let mut bytes = Vec::new();
        image::codecs::png::PngEncoder::new(&mut bytes)
            .write_image(&pixels, 64, 64, image::ExtendedColorType::Rgba8)
            .expect("self authored PNG");
        bytes
    }
}
/// Diagnostic output only: never replace a user file.
pub fn write_png(path: &std::path::Path, pixels: &[u8], size: PixelSize) -> Result<(), String> {
    let file = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    image::codecs::png::PngEncoder::new(file)
        .write_image(
            pixels,
            size.width(),
            size.height(),
            image::ExtendedColorType::Rgba8,
        )
        .map_err(|e| e.to_string())
}
impl RasterImportPort for Images {
    fn stage_png(&mut self, source: &str) -> Result<StagedRaster, ImportError> {
        let path = std::path::Path::new(source);
        if !path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("png"))
        {
            return Err(ImportError::Unsupported);
        }
        let file = std::fs::File::open(path).map_err(|e| ImportError::Io(e.to_string()))?;
        let len = file
            .metadata()
            .map_err(|e| ImportError::Io(e.to_string()))?
            .len();
        let raw = self.blobs.stage_reader(file, len).map_err(blob_import)?;
        self.prepare(
            raw,
            path.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned(),
        )
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};
    #[test]
    fn last_borrowed_pixels_keep_rgba_charge_after_the_cache_is_dropped() {
        let mut images = Images::default();
        let staged = images
            .stage_bytes(Images::sample_png(), "borrowed".into())
            .unwrap();
        let raster = images.load_raster(staged.blob).unwrap();
        let pixels = raster.pixels.clone();
        let quota = images.rgba.clone();
        drop((staged, raster, images));
        assert_eq!(
            quota.used(),
            64 * 64 * 4,
            "last pixel borrower still owns the allocation"
        );
        assert_eq!(pixels.len(), 64 * 64 * 4);
        drop(pixels);
        assert_eq!(quota.used(), 0);
    }
    #[test]
    fn counter_observers_do_not_keep_raw_or_rgba_storage_alive() {
        let mut images = Images::default();
        let raw = images.blobs().observe_bytes();
        let rgba = images.observe_rgba();
        let staged = images
            .stage_bytes(Images::sample_png(), "observed".into())
            .unwrap();
        let raster = images.load_raster(staged.blob).unwrap();
        let pixels = raster.pixels.clone();
        let alias = pixels.clone();
        let raw_bytes = raw.usage().bytes;
        drop((raster, images));
        assert_eq!(raw.usage().bytes, raw_bytes);
        assert_eq!(rgba.usage().bytes, 64 * 64 * 4);
        drop(staged);
        assert_eq!(raw.usage().bytes, 0);
        drop(pixels);
        assert_eq!(rgba.usage().bytes, 64 * 64 * 4);
        drop(alias);
        assert_eq!(rgba.usage().bytes, 0);
        assert!(raw.usage().peak_bytes > 0 && rgba.usage().peak_bytes >= 64 * 64 * 4);
    }
    fn changed_png(seed: u8) -> Vec<u8> {
        let pixels = vec![seed; 64 * 64 * 4];
        let mut bytes = vec![];
        image::codecs::png::PngEncoder::new(&mut bytes)
            .write_image(&pixels, 64, 64, image::ExtendedColorType::Rgba8)
            .unwrap();
        bytes
    }
    #[test]
    fn raw_history_survives_derivative_eviction_and_exact_reconstruction() {
        let mut images = Images {
            rgba: ByteBudget::new(3 * 64 * 64 * 4),
            ..Images::default()
        };
        let stages: Vec<_> = (1..=6)
            .map(|n| images.stage_bytes(changed_png(n), n.to_string()).unwrap())
            .collect();
        assert!(stages.iter().any(|s| images.raster(s.blob).is_none()));
        for (n, stage) in stages.iter().enumerate() {
            assert!(images.blobs.get(stage.blob).is_some());
            let raster = images.load_raster(stage.blob).unwrap();
            assert!(raster.pixels.iter().all(|p| *p == n as u8 + 1));
        }
        assert!(images.rgba_peak_bytes() <= 3 * 64 * 64 * 4);
        drop(stages);
        assert_eq!(images.rgba_bytes(), 0);
        assert_eq!(images.blobs.usage().unwrap().bytes, 0);
    }
    #[test]
    fn borrowed_pixels_cannot_be_evicted_or_escape_the_budget() {
        let mut images = Images {
            rgba: ByteBudget::new(2 * 64 * 64 * 4),
            ..Images::default()
        };
        let first = images.stage_bytes(changed_png(1), "first".into()).unwrap();
        let borrowed = images.load_raster(first.blob).unwrap();
        assert!(matches!(
            images.stage_bytes(changed_png(2), "second".into()),
            Err(ImportError::ResourceLimit)
        ));
        assert_eq!(images.rgba_bytes(), 64 * 64 * 4);
        assert!(images.raster(first.blob).is_some());
        drop(borrowed);
        let second = images.stage_bytes(changed_png(2), "second".into()).unwrap();
        assert!(images.raster(first.blob).is_none());
        drop((first, second));
        assert_eq!(images.rgba_bytes(), 0);
    }
    #[test]
    fn cache_target_is_soft_and_borrowed_working_sets_keep_the_hard_reservation_limit() {
        let mut images = Images {
            rgba: ByteBudget::new(4 * 64 * 64 * 4),
            cache_target: 2 * 64 * 64 * 4,
            ..Images::default()
        };
        let stages: Vec<_> = (1..=3)
            .map(|n| images.stage_bytes(changed_png(n), n.to_string()).unwrap())
            .collect();
        assert_eq!(images.rgba_bytes(), 2 * 64 * 64 * 4);
        let a = images.load_raster(stages[0].blob).unwrap();
        let b = images.load_raster(stages[1].blob).unwrap();
        let c = images.load_raster(stages[2].blob).unwrap();
        assert_eq!(images.rgba_bytes(), 3 * 64 * 64 * 4);
        assert!(images.rgba_peak_bytes() <= 4 * 64 * 64 * 4);
        drop((a, b, c, stages));
        assert_eq!(images.rgba_bytes(), 0);
    }
    #[test]
    fn sha256_png_round_trip_and_lease_lifetime_are_real() {
        let mut images = Images::default();
        let bytes = Images::sample_png();
        let stage = images
            .stage_bytes(bytes.clone(), "自制样例".into())
            .unwrap();
        assert_eq!(
            stage.blob.sha256(),
            <[u8; 32]>::from(Sha256::digest(&bytes))
        );
        assert_eq!(stage.size.width(), 64);
        let lease = images.pin(stage.blob).unwrap();
        let blob = stage.blob;
        drop(stage);
        assert!(images.raster(blob).is_some());
        drop(lease);
        assert!(images.raster(blob).is_none());
    }
    #[test]
    fn malformed_and_oversized_inputs_do_not_leave_entries() {
        let mut images = Images::default();
        assert!(images.stage_bytes(vec![1, 2, 3], "bad".into()).is_err());
        assert_eq!(images.entries.lock().unwrap().len(), 0);
        assert!(matches!(
            images.stage_png("image.jpg"),
            Err(ImportError::Unsupported)
        ));
    }
}
