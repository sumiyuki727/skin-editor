use eframe::egui;
use skin_editor_application::{BlobLease, RasterImportPort, values::*};
use skin_editor_infrastructure::Images;
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::thread::JoinHandle;

/// One decoder, four queued paths and four staged results; never a project writer.
pub struct ImportWorker {
    input: Option<SyncSender<ImportRequest>>,
    output: Option<Receiver<Result<ImportedAsset, String>>>,
    thread: Option<JoinHandle<()>>,
    join_ok: bool,
}
pub enum ImportRequest {
    Auto(String),
    Original(String),
    SampleSound,
}
pub struct ImportedAsset {
    pub blob: BlobRef,
    pub kind: AssetKind,
    pub label: String,
    pub lease: Arc<dyn BlobLease>,
}
pub(super) fn stage(images: &mut Images, request: ImportRequest) -> Result<ImportedAsset, String> {
    let (raw, label, kind) = match request {
        ImportRequest::Auto(path)
            if std::path::Path::new(&path)
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("png")) =>
        {
            let staged = images.stage_png(&path).map_err(|e| format!("{e:?}"))?;
            return Ok(ImportedAsset {
                blob: staged.blob,
                kind: AssetKind::RasterImage(staged.size),
                label: staged.label,
                lease: staged.lease,
            });
        }
        ImportRequest::Auto(path) => {
            if !std::path::Path::new(&path)
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("wav"))
            {
                return Err("仅探测 PNG／WAV；其他音频请勾选原始保留".into());
            }
            let raw = images
                .blobs()
                .stage_wav(std::path::Path::new(&path))
                .map_err(|e| format!("WAV: {e:?}"))?;
            let kind = AssetKind::Audio(raw.audio_metadata().expect("probed WAV"));
            (raw, path, kind)
        }
        ImportRequest::Original(path) => {
            let raw = images
                .blobs()
                .stage_original_file(std::path::Path::new(&path))
                .map_err(|e| format!("原始保留: {e:?}"))?;
            (raw, path, AssetKind::Opaque)
        }
        ImportRequest::SampleSound => {
            let raw = images
                .blobs()
                .stage_audio(
                    crate::audio_verification::tone(48000, 1, 440.0),
                    &skin_editor_infrastructure::WavDecoder,
                )
                .map_err(|e| format!("{e:?}"))?;
            let kind = AssetKind::Audio(raw.audio_metadata().unwrap());
            (raw, "自制示例.wav".into(), kind)
        }
    };
    let label = std::path::Path::new(&label)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    Ok(ImportedAsset {
        blob: raw.reference(),
        kind,
        label,
        lease: raw.lease(),
    })
}
impl ImportWorker {
    pub fn new(mut images: Images, ctx: egui::Context) -> Result<Self, String> {
        let (input, requests) = mpsc::sync_channel::<ImportRequest>(4);
        let (results, output) = mpsc::sync_channel(4);
        let thread = std::thread::Builder::new()
            .name("asset-import".into())
            .spawn(move || {
                while let Ok(request) = requests.recv() {
                    let result = stage(&mut images, request);
                    if results.send(result).is_err() {
                        break;
                    }
                    ctx.request_repaint();
                }
            })
            .map_err(|e| e.to_string())?;
        Ok(Self {
            input: Some(input),
            output: Some(output),
            thread: Some(thread),
            join_ok: true,
        })
    }
    pub fn enqueue(&self, path: String) -> Result<(), String> {
        self.enqueue_request(ImportRequest::Auto(path))
    }
    pub fn enqueue_request(&self, request: ImportRequest) -> Result<(), String> {
        self.input
            .as_ref()
            .unwrap()
            .try_send(request)
            .map_err(|_| "导入队列已满，请稍后重试".into())
    }
    pub fn drain(&self) -> impl Iterator<Item = Result<ImportedAsset, String>> + '_ {
        self.output.as_ref().unwrap().try_iter()
    }
    pub fn close(&mut self) {
        self.input.take();
        self.output.take();
        if let Some(thread) = self.thread.take() {
            self.join_ok &= thread.join().is_ok();
        }
    }
    pub fn closed(&self) -> bool {
        self.input.is_none() && self.output.is_none() && self.thread.is_none() && self.join_ok
    }
}
impl Drop for ImportWorker {
    fn drop(&mut self) {
        // Disconnect results before joining: a full channel cannot deadlock shutdown.
        self.close();
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn full_result_and_request_queues_can_be_closed_without_waiting_for_a_consumer() {
        let mut worker = ImportWorker::new(Images::default(), egui::Context::default()).unwrap();
        // Unsupported extension exits before any filesystem read.
        let mut full = false;
        for _ in 0..32 {
            full |= worker.enqueue("never-opened.jpg".into()).is_err();
        }
        assert!(full);
        assert!(!worker.closed());
        worker.close();
        assert!(worker.closed());
        worker.close();
        assert!(worker.closed());
    }
    #[test]
    fn wav_sample_is_attested_and_shares_the_image_raw_store() {
        let mut images = Images::default();
        let staged = stage(&mut images, ImportRequest::SampleSound).unwrap();
        assert!(
            matches!(staged.kind,AssetKind::Audio(m) if m.sample_rate()==48000 && m.channels()==1)
        );
        assert!(images.blobs().get(staged.blob).is_some());
        drop(staged);
        assert_eq!(images.blobs().usage().unwrap().bytes, 0);
    }
    #[test]
    fn unsupported_wav_requires_explicit_original_preservation_and_does_not_change_file() {
        use std::io::Write;
        let path =
            std::env::temp_dir().join(format!("skin-editor-a2-import-{}.wav", std::process::id()));
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .unwrap();
        let original = b"self authored unsupported audio bytes";
        file.write_all(original).unwrap();
        drop(file);
        let mut images = Images::default();
        assert!(
            stage(
                &mut images,
                ImportRequest::Auto(path.to_string_lossy().into_owned())
            )
            .is_err()
        );
        assert_eq!(images.blobs().usage().unwrap().bytes, 0);
        let staged = stage(
            &mut images,
            ImportRequest::Original(path.to_string_lossy().into_owned()),
        )
        .unwrap();
        assert_eq!(staged.kind, AssetKind::Opaque);
        assert_eq!(images.blobs().get(staged.blob).unwrap().bytes(), original);
        assert_eq!(std::fs::read(&path).unwrap(), original);
        drop(staged);
        assert_eq!(images.blobs().usage().unwrap().bytes, 0);
        std::fs::remove_file(&path).unwrap();
    }
}
