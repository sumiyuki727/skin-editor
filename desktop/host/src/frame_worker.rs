//! One native-session preparation worker. No project/session or GUI writer crosses it.
use skin_editor_application::{FrameDescriptor, PreviewError};
use skin_editor_infrastructure::{Images, RawBlob};
use skin_editor_rendering::{Images as Rasters, Plan};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc::{self, Receiver, SyncSender},
};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

pub struct Job {
    pub descriptor: FrameDescriptor,
    pub plan: Plan,
    pub queued: Instant,
    pub started_ms: f64,
    pub plan_ms: f64,
    pub raw: Vec<RawBlob>,
    pub cancelled: Arc<AtomicBool>,
}
impl Job {
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }
}
pub struct Completion<T> {
    pub job: Job,
    pub started: Instant,
    pub finished: Instant,
    pub result: Result<Option<T>, PreviewError>,
}
/// The single credit stays occupied until the actual completion is received,
/// including cancellation. There is no speculative queue or early budget refund.
pub struct FrameWorker<T> {
    input: Option<SyncSender<Job>>,
    output: Option<Receiver<Completion<T>>>,
    thread: Option<JoinHandle<()>>,
    running: Option<(FrameDescriptor, Arc<AtomicBool>)>,
    received: Option<Completion<T>>,
    join_ok: bool,
}
impl<T: Send + 'static> FrameWorker<T> {
    pub fn new(
        prepare: impl Fn(&Job) -> Result<Option<T>, PreviewError> + Send + 'static,
        wake: impl Fn() + Send + 'static,
    ) -> Result<Self, PreviewError> {
        let (input, requests) = mpsc::sync_channel::<Job>(1);
        let (results, output) = mpsc::sync_channel(1);
        let thread = std::thread::Builder::new()
            .name("preview-prepare".into())
            .spawn(move || {
                while let Ok(job) = requests.recv() {
                    let started = Instant::now();
                    let result = if job.is_cancelled() {
                        Ok(None)
                    } else {
                        prepare(&job)
                    };
                    if results
                        .send(Completion {
                            job,
                            result,
                            started,
                            finished: Instant::now(),
                        })
                        .is_err()
                    {
                        break;
                    }
                    wake();
                }
            })
            .map_err(|e| PreviewError::Backend(e.to_string()))?;
        Ok(Self {
            input: Some(input),
            output: Some(output),
            thread: Some(thread),
            running: None,
            received: None,
            join_ok: true,
        })
    }
    pub fn current(&self) -> Option<FrameDescriptor> {
        self.running.as_ref().map(|r| r.0)
    }
    pub fn cancel(&self) {
        if let Some((_, flag)) = &self.running {
            flag.store(true, Ordering::Release);
        }
    }
    /// Fault teardown may be polled by the UI. Cancellation is not completion:
    /// keep the running credit until the thread actually exits; never join a
    /// gate-held decoder or driver call on this path.
    pub fn try_close(&mut self) -> bool {
        self.cancel();
        self.input.take();
        self.output.take();
        self.received.take();
        if self.thread.as_ref().is_some_and(|t| !t.is_finished()) {
            return false;
        }
        self.close();
        true
    }
    pub fn submit(&mut self, job: Job) -> Result<(), PreviewError> {
        if self.running.is_some() {
            return Err(PreviewError::ResourceLimit);
        }
        let running = (job.descriptor, job.cancelled.clone());
        self.input
            .as_ref()
            .ok_or_else(|| PreviewError::Backend("preview worker closed".into()))?
            .try_send(job)
            .map_err(|_| PreviewError::Backend("preview worker unavailable".into()))?;
        self.running = Some(running);
        Ok(())
    }
    pub fn poll(&mut self) -> Result<Option<Completion<T>>, PreviewError> {
        if self.running.is_none() {
            return Ok(None);
        }
        let result = if let Some(done) = self.received.take() {
            Some(done)
        } else {
            match self.output.as_ref().unwrap().try_recv() {
                Ok(done) => Some(done),
                Err(mpsc::TryRecvError::Empty) => None,
                Err(mpsc::TryRecvError::Disconnected) => {
                    return Err(PreviewError::Backend("preview worker stopped".into()));
                }
            }
        };
        if result.is_some() {
            self.running = None;
        }
        Ok(result)
    }
    /// Explicit diagnostic barrier only, never called from ordinary GUI updates.
    pub fn wait(&mut self) -> Result<(), PreviewError> {
        if self.running.is_some() && self.received.is_none() {
            self.received = Some(
                self.output
                    .as_ref()
                    .unwrap()
                    .recv_timeout(Duration::from_secs(20))
                    .map_err(|e| PreviewError::Backend(format!("preview barrier: {e}")))?,
            );
        }
        Ok(())
    }
    pub fn close(&mut self) {
        if let Some((_, flag)) = &self.running {
            flag.store(true, Ordering::Release);
        }
        self.input.take();
        self.output.take();
        self.received.take();
        if let Some(thread) = self.thread.take() {
            self.join_ok &= thread.join().is_ok();
        }
        self.running = None;
    }
    pub fn closed(&self) -> bool {
        self.input.is_none()
            && self.output.is_none()
            && self.thread.is_none()
            && self.running.is_none()
            && self.received.is_none()
            && self.join_ok
    }
}
impl<T> Drop for FrameWorker<T> {
    fn drop(&mut self) {
        if let Some((_, flag)) = &self.running {
            flag.store(true, Ordering::Release);
        }
        self.input.take();
        self.output.take();
        self.received.take();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
pub fn decode(job: &Job, images: &Images) -> Result<Option<(Rasters, f64)>, PreviewError> {
    let started = Instant::now();
    let mut rasters = Rasters::new();
    for raw in &job.raw {
        if job.is_cancelled() {
            return Ok(None);
        }
        let raster = images
            .load_raster(raw.reference())
            .map_err(|e| PreviewError::Backend(format!("image preparation: {e:?}")))?;
        rasters.insert(raw.reference(), raster);
    }
    Ok((!job.is_cancelled()).then(|| (rasters, started.elapsed().as_secs_f64() * 1000.)))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fault_poll_does_not_wait_or_refund_a_gate_held_job() {
        let store = skin_editor_infrastructure::BlobStore::default();
        let raw = store.stage_bytes(vec![1; 19]).unwrap();
        let (entered, wait) = mpsc::sync_channel(1);
        let (release, gate) = mpsc::sync_channel(1);
        let mut worker = FrameWorker::new(
            move |job| {
                entered.send(()).unwrap();
                gate.recv_timeout(Duration::from_secs(5)).unwrap();
                assert!(job.is_cancelled());
                Ok(Some(job.raw[0].clone()))
            },
            || {},
        )
        .unwrap();
        worker.submit(job(1, vec![raw])).unwrap();
        wait.recv_timeout(Duration::from_secs(5)).unwrap();
        assert!(!worker.try_close());
        assert!(!worker.closed());
        assert_eq!(store.usage().unwrap().bytes, 19);
        release.send(()).unwrap();
        worker.close();
        assert!(worker.try_close());
        assert!(worker.closed());
        assert_eq!(store.usage().unwrap().bytes, 0);
    }
    use skin_editor_application::{values::*, *};
    use skin_editor_rendering::Target;
    #[test]
    fn close_joins_held_work_before_success_and_detects_worker_panics() {
        let (entered_tx, entered) = mpsc::sync_channel(1);
        let (release, gate) = mpsc::sync_channel(1);
        let mut w = FrameWorker::new(
            move |_| {
                entered_tx.send(()).unwrap();
                gate.recv().unwrap();
                Ok(Some(()))
            },
            || {},
        )
        .unwrap();
        w.submit(job(1, vec![])).unwrap();
        entered.recv_timeout(Duration::from_secs(5)).unwrap();
        assert!(!w.closed());
        let t = std::thread::spawn(move || {
            w.close();
            assert!(w.closed());
            w.close();
            assert!(w.closed());
        });
        release.send(()).unwrap();
        t.join().unwrap();
        let mut failed =
            FrameWorker::<()>::new(|_| panic!("controlled preparation panic"), || {}).unwrap();
        failed.submit(job(2, vec![])).unwrap();
        assert!(failed.wait().is_err());
        failed.close();
        assert!(!failed.closed());
    }
    fn job(id: u64, raw: Vec<RawBlob>) -> Job {
        Job {
            descriptor: FrameDescriptor {
                api_version: PREVIEW_API_VERSION,
                key: FrameKey(id),
                source: FrameSource {
                    session: HistorySessionId::new(1).unwrap(),
                    state: ExpectedState {
                        head: NodeId::new(1).unwrap(),
                        revision: id,
                    },
                    context: PreviewContext {
                        target: PreviewTarget::Components,
                        draft: None,
                        size: [8, 8],
                        viewport_generation: 1,
                        scene: 0,
                        simulation_micros: 0,
                    },
                },
                status: FrameStatus::Pending,
            },
            plan: Plan::build(
                &ProjectSession::new(1, 1).unwrap().snapshot().project,
                Target::Components,
                PixelSize::new(8, 8).unwrap(),
            )
            .unwrap(),
            queued: Instant::now(),
            started_ms: 0.,
            plan_ms: 0.,
            raw,
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }
    #[test]
    fn cancellation_keeps_credit_and_raw_lease_until_actual_completion() {
        let store = skin_editor_infrastructure::BlobStore::default();
        let raw = store.stage_bytes(vec![1, 2, 3]).unwrap();
        let (entered_tx, entered) = mpsc::sync_channel(1);
        let (release, gate) = mpsc::sync_channel(1);
        let mut worker = FrameWorker::new(
            move |job| {
                entered_tx.send(()).unwrap();
                gate.recv().unwrap();
                assert!(job.is_cancelled());
                Ok(Some(job.raw[0].clone())) // even an uncooperative late success keeps its identity
            },
            || {},
        )
        .unwrap();
        worker.submit(job(1, vec![raw.clone()])).unwrap();
        drop(raw);
        entered.recv_timeout(Duration::from_secs(5)).unwrap();
        worker.cancel();
        let refused = worker.submit(job(2, vec![]));
        let still_running = worker.current().is_some();
        let still_pinned = store.usage().unwrap().bytes;
        release.send(()).unwrap();
        worker.wait().unwrap();
        let done = worker.poll().unwrap().unwrap();
        assert!(done.job.queued <= done.started && done.started <= done.finished);
        assert_eq!(refused, Err(PreviewError::ResourceLimit));
        assert!(still_running);
        assert_eq!(still_pinned, 3);
        assert_eq!(done.job.descriptor.key, FrameKey(1));
        assert!(done.job.is_cancelled());
        assert_eq!(store.usage().unwrap().bytes, 3);
        drop(done);
        assert_eq!(store.usage().unwrap().bytes, 0);
        worker.submit(job(2, vec![])).unwrap();
        // Release this admitted job before dropping/joining.
        worker.cancel();
        release.send(()).unwrap();
        worker.close();
    }
    #[test]
    fn fault_cancelled_draft_keeps_inflight_raw_until_late_completion_or_actual_join() {
        use skin_editor_application::values::{Asset, AssetId, AssetKind};
        use skin_editor_application::{Command, DraftAction, SessionLimits};
        use skin_editor_ui::EditorUi;
        for close in [false, true] {
            let store = skin_editor_infrastructure::BlobStore::default();
            let raw = store.stage_bytes(vec![1, 2, 3]).unwrap();
            let mut session = ProjectSession::with_resources(
                1,
                91,
                Box::new(store.clone()),
                SessionLimits::default(),
            )
            .unwrap();
            let mut ui = EditorUi::default();
            let before = session.snapshot();
            assert!(ui.draft(
                &mut session,
                DraftAction::Begin {
                    expected: before.state
                }
            ));
            let draft = session
                .draft_info(before.session_id)
                .unwrap()
                .unwrap()
                .state;
            assert!(ui.draft(
                &mut session,
                DraftAction::Replace {
                    expected: draft,
                    commands: vec![Command::RegisterAsset {
                            asset: Asset::new(
                                AssetId::new(1).unwrap(),
                                "draft-only".into(),
                                raw.reference(),
                                AssetKind::Opaque
                            )
                            .unwrap(),
                        }]
                }
            ));
            let draft = session
                .draft_info(before.session_id)
                .unwrap()
                .unwrap()
                .state;
            let mut request = job(1, vec![raw]);
            request.descriptor.source.session = before.session_id;
            request.descriptor.source.state = before.state;
            request.descriptor.source.context.draft = Some(draft);
            let old = request.descriptor.source;
            let (entered_tx, entered) = mpsc::sync_channel(1);
            let (release, gate) = mpsc::sync_channel(1);
            let mut worker = FrameWorker::new(
                move |job| {
                    entered_tx.send(()).unwrap();
                    gate.recv().unwrap();
                    Ok(Some(job.raw[0].clone()))
                },
                || {},
            )
            .unwrap();
            worker.submit(request).unwrap();
            entered.recv_timeout(Duration::from_secs(5)).unwrap();
            assert!(ui.interrupt_for_fault(&mut session));
            assert_eq!(session.snapshot(), before);
            assert!(session.draft_info(before.session_id).unwrap().is_none());
            worker.cancel();
            if close {
                assert!(!worker.try_close());
            }
            assert!(worker.current().is_some());
            assert_eq!(store.usage().unwrap().bytes, 3);
            release.send(()).unwrap();
            if !close {
                worker.wait().unwrap();
                let completion = worker.poll().unwrap().unwrap();
                let mut desired = old;
                desired.context.draft = None;
                assert!(!crate::presentation::candidate_current(
                    Some(desired),
                    completion.job.descriptor.source,
                    false
                ));
                assert!(!crate::presentation::candidate_current(
                    None,
                    completion.job.descriptor.source,
                    false
                ));
                assert!(completion.result.as_ref().unwrap().is_some());
                assert_eq!(store.usage().unwrap().bytes, 3);
                drop(completion);
            }
            worker.close();
            assert!(worker.closed());
            assert_eq!(store.usage().unwrap().bytes, 0);
        }
    }
    #[test]
    fn completed_result_and_disconnect_release_raw_ownership() {
        let store = skin_editor_infrastructure::BlobStore::default();
        let raw = store.stage_bytes(vec![7; 19]).unwrap();
        let (wake_tx, wake) = mpsc::sync_channel(1);
        let mut worker = FrameWorker::new(
            |job| Ok(Some(job.raw[0].clone())),
            move || {
                let _ = wake_tx.send(());
            },
        )
        .unwrap();
        worker.submit(job(1, vec![raw])).unwrap();
        wake.recv_timeout(Duration::from_secs(5)).unwrap();
        assert_eq!(store.usage().unwrap().bytes, 19);
        worker.close(); // result channel contains raw leases, no consumer needed
        assert_eq!(store.usage().unwrap().bytes, 0);
        assert!(worker.submit(job(2, vec![])).is_err());
    }
    #[test]
    fn real_png_reuse_cancellation_and_invalid_bytes_are_explicit() {
        let images = Images::default();
        let raw = images.blobs().stage_bytes(Images::sample_png()).unwrap();
        let request = job(1, vec![raw]);
        let (a, _) = decode(&request, &images).unwrap().unwrap();
        let used = images.rgba_bytes();
        let (b, _) = decode(&request, &images).unwrap().unwrap();
        assert!(Arc::ptr_eq(
            a.values().next().unwrap(),
            b.values().next().unwrap()
        ));
        assert_eq!(images.rgba_bytes(), used);
        request.cancelled.store(true, Ordering::Release);
        assert!(decode(&request, &images).unwrap().is_none());
        drop((a, b, request));
        assert_eq!(images.rgba_bytes(), 0);
        let invalid = images.blobs().stage_bytes(vec![1, 2]).unwrap();
        assert!(decode(&job(2, vec![invalid]), &images).is_err());
        assert_eq!(images.rgba_bytes(), 0);
    }
    #[test]
    fn preparation_failure_releases_credit_and_next_request_can_run() {
        let mut worker =
            FrameWorker::new(|_| Err::<Option<()>, _>(PreviewError::ResourceLimit), || {}).unwrap();
        for i in 1..=2 {
            worker.submit(job(i, vec![])).unwrap();
            worker.wait().unwrap();
            assert_eq!(
                worker.poll().unwrap().unwrap().result,
                Err(PreviewError::ResourceLimit)
            );
            assert!(worker.current().is_none());
        }
    }
}
