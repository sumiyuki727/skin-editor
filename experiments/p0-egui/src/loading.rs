//! Bounded PNG experiment, not a project import/Command API.
//! One worker, bounded in-flight requests and result slots. Replacement invalidates
//! queued work and rejects completed old generations. An executing PNG decode is
//! not interruptible; cancellation takes effect before/after that decode.
use crate::Decoded;
use crate::Workload;
use std::{
    collections::{BTreeSet, VecDeque},
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
        mpsc,
    },
    thread::{self, JoinHandle},
    time::Instant,
};

pub const MAX_IMAGE_BYTES: usize = 4 * 1024 * 1024;
pub const UPLOAD_BYTES_PER_FRAME: usize = 8 * 1024 * 1024;
pub const UPLOADS_PER_FRAME: usize = 2;

/// A single bounded metadata read, completed before pixel work begins. No PNG
/// decoder is active alongside it. Dropping joins the read; OS file I/O cannot
/// be forcibly interrupted by this experiment.
pub struct MetadataTask {
    rx: Option<mpsc::Receiver<(Result<Workload, String>, f64)>>,
    worker: Option<JoinHandle<()>>,
}
impl MetadataTask {
    pub fn new(root: PathBuf, wake: impl FnOnce() + Send + 'static) -> Self {
        let (tx, rx) = mpsc::sync_channel(1);
        let worker = thread::spawn(move || {
            let start = Instant::now();
            let result = Workload::open(&root);
            if tx
                .send((result, start.elapsed().as_secs_f64() * 1000.0))
                .is_ok()
            {
                wake();
            }
        });
        Self {
            rx: Some(rx),
            worker: Some(worker),
        }
    }
    pub fn poll(&mut self) -> Option<(Result<Workload, String>, f64)> {
        self.rx.as_ref()?.try_recv().ok()
    }
}
impl Drop for MetadataTask {
    fn drop(&mut self) {
        self.rx.take();
        if let Some(worker) = self.worker.take() {
            worker.join().expect("metadata worker panicked");
        }
    }
}

#[derive(Clone, Debug)]
pub struct DecodeJob {
    pub id: usize,
    pub path: PathBuf,
    pub size: [usize; 2],
}
pub struct Ready {
    pub id: usize,
    pub image: Decoded,
}
#[derive(Default)]
pub struct DecodeStats {
    pub started: usize,
    pub decoded: usize,
    pub stale_results: usize,
    pub skipped_before_decode: usize,
    pub replaced_queued: usize,
    pub decode_ms: Vec<f64>,
}
struct Request {
    generation: u64,
    job: DecodeJob,
}
struct Reply {
    generation: u64,
    id: usize,
    result: Option<Result<Decoded, String>>,
    elapsed_ms: f64,
}
pub struct DecodeQueue {
    generation: Arc<AtomicU64>,
    todo: VecDeque<DecodeJob>,
    active: usize,
    limit: usize,
    tx: Option<mpsc::SyncSender<Request>>,
    rx: Option<mpsc::Receiver<Reply>>,
    worker: Option<JoinHandle<()>>,
    pub stats: DecodeStats,
}
impl DecodeQueue {
    pub fn new(wake: impl Fn() + Send + 'static) -> Self {
        Self::png_worker(wake, 1)
    }
    pub fn prefetched(wake: impl Fn() + Send + 'static) -> Self {
        Self::png_worker(wake, 2)
    }
    fn png_worker(wake: impl Fn() + Send + 'static, limit: usize) -> Self {
        Self::with_limit(
            wake,
            |job| {
                // Limit decoder dimensions as well as validating the returned size.
                // This fixture-only worker accepts at most one 1024² RGBA image.
                crate::decode_png_sized(&job.path, job.size).map(Arc::new)
            },
            limit,
        )
    }
    #[cfg(test)]
    fn with_decoder(
        wake: impl Fn() + Send + 'static,
        decode: impl Fn(&DecodeJob) -> Result<Decoded, String> + Send + 'static,
    ) -> Self {
        Self::with_limit(wake, decode, 1)
    }
    fn with_limit(
        wake: impl Fn() + Send + 'static,
        decode: impl Fn(&DecodeJob) -> Result<Decoded, String> + Send + 'static,
        limit: usize,
    ) -> Self {
        assert!((1..=2).contains(&limit));
        let generation = Arc::new(AtomicU64::new(0));
        let token = generation.clone();
        let (tx, requests) = mpsc::sync_channel::<Request>(limit);
        let (results, rx) = mpsc::sync_channel(limit);
        let worker = thread::spawn(move || {
            while let Ok(request) = requests.recv() {
                let start = Instant::now();
                let result = if request.generation != token.load(Ordering::Acquire) {
                    None
                } else {
                    Some(decode(&request.job).and_then(|image| {
                        if image.size != request.job.size
                            || image.pixels.len() * 4 > MAX_IMAGE_BYTES
                        {
                            Err("Decoded image differs from bounded request".into())
                        } else {
                            Ok(image)
                        }
                    }))
                };
                // Old pixels are dropped here, before handing a reply to the UI.
                let result = if request.generation != token.load(Ordering::Acquire) {
                    result.map(|_| Err("Cancelled decode".into()))
                } else {
                    result
                };
                if results
                    .send(Reply {
                        generation: request.generation,
                        id: request.job.id,
                        result,
                        elapsed_ms: start.elapsed().as_secs_f64() * 1000.0,
                    })
                    .is_err()
                {
                    break;
                }
                wake();
            }
        });
        Self {
            generation,
            todo: VecDeque::new(),
            active: 0,
            limit,
            tx: Some(tx),
            rx: Some(rx),
            worker: Some(worker),
            stats: DecodeStats::default(),
        }
    }
    /// Asset IDs refer to immutable fixture bytes in this experiment. A future
    /// mutable source must also include a content revision in its resource key.
    pub fn replace(&mut self, jobs: Vec<DecodeJob>) -> Result<(), String> {
        let mut ids = BTreeSet::new();
        if jobs.len() > 112
            || jobs.iter().any(|j| {
                !ids.insert(j.id)
                    || j.size.contains(&0)
                    || j.size.iter().any(|&n| n > 1024)
                    || j.size[0] * j.size[1] * 4 > MAX_IMAGE_BYTES
            })
        {
            return Err("Invalid or unbounded decode plan".into());
        }
        let next = self
            .generation
            .load(Ordering::Acquire)
            .checked_add(1)
            .ok_or("Generation exhausted")?;
        self.generation.store(next, Ordering::Release);
        self.stats.replaced_queued += self.todo.len();
        self.todo = jobs.into();
        self.dispatch();
        Ok(())
    }
    fn dispatch(&mut self) {
        while self.active < self.limit {
            let Some(job) = self.todo.pop_front() else {
                break;
            };
            self.tx
                .as_ref()
                .unwrap()
                .send(Request {
                    generation: self.generation.load(Ordering::Acquire),
                    job,
                })
                .expect("decode worker alive");
            self.active += 1;
            self.stats.started += 1;
        }
    }
    pub fn poll(&mut self) -> Option<Result<Ready, String>> {
        let reply = self.rx.as_ref().unwrap().try_recv().ok()?;
        self.active -= 1;
        if reply.result.is_some() {
            self.stats.decoded += 1;
            self.stats.decode_ms.push(reply.elapsed_ms);
        } else {
            self.stats.skipped_before_decode += 1;
        }
        let current = reply.generation == self.generation.load(Ordering::Acquire);
        if !current {
            self.stats.stale_results += 1;
        }
        self.dispatch();
        if current {
            reply.result.map(|r| {
                r.map(|image| Ready {
                    id: reply.id,
                    image,
                })
            })
        } else {
            None
        }
    }
    pub fn is_idle(&self) -> bool {
        self.active == 0 && self.todo.is_empty()
    }
    pub fn outstanding_limit(&self) -> usize {
        self.limit
    }
    pub fn outstanding(&self) -> usize {
        self.active
    }
}
impl Drop for DecodeQueue {
    fn drop(&mut self) {
        self.generation.fetch_add(1, Ordering::Release);
        self.tx.take();
        self.rx.take(); // Unblock a sender before joining; no detached worker.
        if let Some(worker) = self.worker.take() {
            worker.join().expect("decode worker panicked");
        }
    }
}

#[derive(Default)]
pub struct UploadBudget {
    frame: Option<u64>,
    pub bytes: usize,
    pub count: usize,
    pub peak_bytes: usize,
    pub peak_count: usize,
}
impl UploadBudget {
    pub fn begin(&mut self, frame: u64) {
        if self.frame != Some(frame) {
            self.frame = Some(frame);
            self.bytes = 0;
            self.count = 0;
        }
    }
    pub fn take(&mut self, bytes: usize) -> bool {
        if bytes > UPLOAD_BYTES_PER_FRAME - self.bytes || self.count == UPLOADS_PER_FRAME {
            return false;
        }
        self.bytes += bytes;
        self.count += 1;
        self.peak_bytes = self.peak_bytes.max(self.bytes);
        self.peak_count = self.peak_count.max(self.count);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    fn job(id: usize) -> DecodeJob {
        DecodeJob {
            id,
            path: PathBuf::from("unused"),
            size: [1, 1],
        }
    }
    fn image() -> Decoded {
        Arc::new(eframe::egui::ColorImage::new(
            [1, 1],
            vec![eframe::egui::Color32::WHITE],
        ))
    }
    fn poll_after_wake(
        q: &mut DecodeQueue,
        wake: &mpsc::Receiver<()>,
    ) -> Option<Result<Ready, String>> {
        wake.recv_timeout(Duration::from_secs(5)).unwrap();
        q.poll()
    }
    #[test]
    fn replacement_discards_running_result_and_queued_jobs() {
        let (begun, start) = mpsc::channel();
        let (release, barrier) = mpsc::channel();
        let (wake, awake) = mpsc::channel();
        let mut q = DecodeQueue::with_decoder(
            move || {
                wake.send(()).unwrap();
            },
            move |j| {
                begun.send(j.id).unwrap();
                barrier.recv().unwrap();
                Ok(image())
            },
        );
        q.replace(vec![job(0), job(1)]).unwrap();
        assert_eq!(start.recv_timeout(Duration::from_secs(5)).unwrap(), 0);
        q.replace(vec![job(2)]).unwrap();
        release.send(()).unwrap();
        assert!(poll_after_wake(&mut q, &awake).is_none());
        assert_eq!(start.recv_timeout(Duration::from_secs(5)).unwrap(), 2);
        release.send(()).unwrap();
        assert_eq!(poll_after_wake(&mut q, &awake).unwrap().unwrap().id, 2);
        assert!(q.is_idle());
        assert_eq!(
            (
                q.stats.started,
                q.stats.stale_results,
                q.stats.replaced_queued
            ),
            (2, 1, 1)
        );
    }
    #[test]
    fn backpressure_does_not_decode_the_next_image_until_consumed() {
        let (wake, awake) = mpsc::channel();
        let (decoded, seen) = mpsc::channel();
        let mut q = DecodeQueue::with_decoder(
            move || {
                wake.send(()).unwrap();
            },
            move |j| {
                decoded.send(j.id).unwrap();
                Ok(image())
            },
        );
        q.replace(vec![job(0), job(1)]).unwrap();
        awake.recv_timeout(Duration::from_secs(5)).unwrap();
        assert!(q.replace(vec![job(9), job(9)]).is_err());
        assert_eq!(seen.try_recv().unwrap(), 0);
        assert!(seen.try_recv().is_err());
        assert_eq!(q.poll().unwrap().unwrap().id, 0);
        assert_eq!(poll_after_wake(&mut q, &awake).unwrap().unwrap().id, 1);
    }
    #[test]
    fn prefetch_stops_at_two_and_rejects_both_obsolete_replies() {
        let (wake, awake) = mpsc::channel();
        let (decoded, seen) = mpsc::channel();
        let mut q = DecodeQueue::with_limit(
            move || {
                wake.send(()).unwrap();
            },
            move |j| {
                decoded.send(j.id).unwrap();
                Ok(image())
            },
            2,
        );
        q.replace(vec![job(0), job(1), job(2)]).unwrap();
        for id in [0, 1] {
            awake.recv_timeout(Duration::from_secs(5)).unwrap();
            assert_eq!(seen.recv_timeout(Duration::from_secs(5)).unwrap(), id);
        }
        assert!(seen.try_recv().is_err()); // No third decoded image while UI is idle.
        q.replace(vec![job(9)]).unwrap();
        assert!(q.poll().is_none());
        assert!(q.poll().is_none());
        assert_eq!(poll_after_wake(&mut q, &awake).unwrap().unwrap().id, 9);
        assert_eq!(q.stats.stale_results, 2);
        assert!(q.is_idle());
    }
    #[test]
    fn prefetched_queue_drop_with_full_result_slots_joins() {
        let (wake, awake) = mpsc::channel();
        let mut q = DecodeQueue::with_limit(
            move || {
                let _ = wake.send(());
            },
            |_| Ok(image()),
            2,
        );
        q.replace(vec![job(0), job(1), job(2)]).unwrap();
        for _ in 0..2 {
            awake.recv_timeout(Duration::from_secs(5)).unwrap();
        }
        drop(q);
    }
    #[test]
    fn errors_invalid_plans_and_cancel_are_observable() {
        let (wake, awake) = mpsc::channel();
        let mut q = DecodeQueue::with_decoder(
            move || {
                wake.send(()).unwrap();
            },
            |_| Err("bad PNG".into()),
        );
        assert!(q.replace(vec![job(0), job(0)]).is_err());
        let mut huge = job(1);
        huge.size = [usize::MAX, 1];
        assert!(q.replace(vec![huge]).is_err());
        q.replace(vec![job(0)]).unwrap();
        assert_eq!(
            poll_after_wake(&mut q, &awake).unwrap().err().unwrap(),
            "bad PNG"
        );
        q.replace(vec![job(1)]).unwrap();
        q.replace(Vec::new()).unwrap();
        assert!(poll_after_wake(&mut q, &awake).is_none());
        assert!(q.is_idle());
    }
    #[test]
    fn drop_with_unconsumed_result_joins_without_deadlock() {
        let (wake, awake) = mpsc::channel();
        let q = DecodeQueue::with_decoder(
            move || {
                let _ = wake.send(());
            },
            |_| Ok(image()),
        );
        let mut q = q;
        q.replace(vec![job(0), job(1)]).unwrap();
        awake.recv_timeout(Duration::from_secs(5)).unwrap();
        drop(q);
    }
    #[test]
    fn upload_limit_is_shared_by_all_passes_in_a_frame() {
        let mut b = UploadBudget::default();
        b.begin(1);
        assert!(b.take(MAX_IMAGE_BYTES));
        b.begin(1);
        assert!(b.take(MAX_IMAGE_BYTES));
        assert!(!b.take(1));
        b.begin(2);
        assert!(!b.take(UPLOAD_BYTES_PER_FRAME + 1));
        assert_eq!(b.bytes, 0);
        assert!(b.take(16));
        assert!(b.take(16));
        assert!(!b.take(16));
        assert_eq!((b.peak_bytes, b.peak_count), (UPLOAD_BYTES_PER_FRAME, 2));
    }
    #[test]
    fn native_worker_reports_missing_and_corrupt_png() {
        // No user directory, network, window, or actual client required.
        let (wake, awake) = mpsc::channel();
        let mut q = DecodeQueue::new(move || {
            let _ = wake.send(());
        });
        let dir = std::env::temp_dir().join(format!("p0-worker-{}", std::process::id()));
        std::fs::create_dir(&dir).unwrap();
        let path = dir.join("素材 日本語.png");
        std::fs::write(&path, b"not PNG").unwrap();
        let mut j = job(0);
        j.path = path.clone();
        q.replace(vec![j.clone()]).unwrap();
        assert!(poll_after_wake(&mut q, &awake).unwrap().is_err());
        std::fs::remove_file(path).unwrap();
        q.replace(vec![j.clone()]).unwrap();
        assert!(poll_after_wake(&mut q, &awake).unwrap().is_err());
        image::save_buffer(
            &j.path,
            &[255, 0, 0, 255, 0, 200, 0, 128],
            2,
            1,
            image::ColorType::Rgba8,
        )
        .unwrap();
        q.replace(vec![j.clone()]).unwrap();
        assert!(poll_after_wake(&mut q, &awake).unwrap().is_err()); // Changed dimensions.
        j.size = [2, 1];
        q.replace(vec![j.clone()]).unwrap();
        let ready = poll_after_wake(&mut q, &awake).unwrap().unwrap();
        assert_eq!(ready.image.size, [2, 1]);
        assert_eq!(ready.image.pixels[0].to_array(), [255, 0, 0, 255]);
        assert_eq!(ready.image.pixels[1].a(), 128);
        std::fs::remove_file(j.path).unwrap();
        std::fs::remove_dir(dir).unwrap();
    }
}
