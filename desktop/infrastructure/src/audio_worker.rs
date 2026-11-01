//! One decoder thread. Admission and provenance stay with the control owner.
use crate::{AudioDecoder, FinishedPreparation, RunningPreparation};
use std::sync::mpsc::{self, Receiver, SyncSender, TrySendError};
use std::thread::{self, JoinHandle};

pub struct PreparationWorker {
    sender: Option<SyncSender<RunningPreparation>>,
    receiver: Receiver<FinishedPreparation>,
    thread: Option<JoinHandle<()>>,
    join_ok: bool,
}
impl PreparationWorker {
    pub fn new(mut decoder: impl AudioDecoder + Send + 'static) -> std::io::Result<Self> {
        let (sender, work) = mpsc::sync_channel::<RunningPreparation>(1);
        let (results, receiver) = mpsc::sync_channel(1);
        let thread = thread::Builder::new()
            .name("skin-audio-decode".into())
            .spawn(move || {
                while let Ok(job) = work.recv() {
                    if results.send(job.execute(&mut decoder)).is_err() {
                        break;
                    }
                }
            })?;
        Ok(Self {
            sender: Some(sender),
            receiver,
            thread: Some(thread),
            join_ok: true,
        })
    }
    pub fn try_submit(&self, job: RunningPreparation) -> Result<(), Box<RunningPreparation>> {
        let Some(sender) = &self.sender else {
            return Err(Box::new(job));
        };
        sender.try_send(job).map_err(|e| match e {
            TrySendError::Full(job) | TrySendError::Disconnected(job) => Box::new(job),
        })
    }
    pub fn try_result(&self) -> Option<FinishedPreparation> {
        self.receiver.try_recv().ok()
    }
    /// Diagnostics/tests may await an explicit completion; never use in a callback/UI tick.
    pub fn wait_result(&self, timeout: std::time::Duration) -> Option<FinishedPreparation> {
        self.receiver.recv_timeout(timeout).ok()
    }
    /// Close the preparation queue first so an active decoder observes cancellation.
    pub fn close(&mut self) {
        self.sender.take();
        // Drain while joining to prevent a full bounded result slot locking shutdown.
        if let Some(thread) = self.thread.take() {
            while !thread.is_finished() {
                let _ = self
                    .receiver
                    .recv_timeout(std::time::Duration::from_millis(10));
            }
            self.join_ok &= thread.join().is_ok();
        }
        while self.receiver.try_recv().is_ok() {}
    }
    pub fn closed(&self) -> bool {
        self.thread.is_none() && self.sender.is_none() && self.join_ok
    }
}
impl Drop for PreparationWorker {
    fn drop(&mut self) {
        self.close();
    }
}
