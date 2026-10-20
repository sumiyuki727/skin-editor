//! One read-only document worker, latest-request mailbox, bounded rendezvous result.
use crate::composition_fixture::{LoadedDocument, read_document};
use std::{
    path::PathBuf,
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicU64, Ordering},
        mpsc,
    },
    thread::{self, JoinHandle},
};
struct Request {
    ticket: u64,
    root: PathBuf,
    document: usize,
}
#[derive(Default)]
struct Mailbox {
    next: Option<Request>,
    stop: bool,
    busy: bool,
}
struct Reply {
    ticket: u64,
    result: Result<LoadedDocument, String>,
}
pub enum Event {
    Ready {
        ticket: u64,
        result: Result<LoadedDocument, String>,
    },
    Stale {
        ticket: u64,
    },
}
pub struct DocumentLoader {
    generation: Arc<AtomicU64>,
    mailbox: Arc<(Mutex<Mailbox>, Condvar)>,
    rx: Option<mpsc::Receiver<Reply>>,
    worker: Option<JoinHandle<()>>,
}
impl DocumentLoader {
    pub fn new(wake: impl Fn() + Send + 'static) -> Self {
        Self::with_reader(wake, read_document)
    }
    fn with_reader(
        wake: impl Fn() + Send + 'static,
        read: impl Fn(&std::path::Path, usize, &dyn Fn() -> bool) -> Result<LoadedDocument, String>
        + Send
        + 'static,
    ) -> Self {
        let generation = Arc::new(AtomicU64::new(0));
        let mailbox = Arc::new((Mutex::new(Mailbox::default()), Condvar::new()));
        let token = generation.clone();
        let work = mailbox.clone();
        // The worker cannot begin another document while a result is unconsumed.
        let (tx, rx) = mpsc::sync_channel::<Reply>(0);
        let worker = thread::spawn(move || {
            loop {
                let request = {
                    let (mutex, cv) = &*work;
                    let mut state = mutex.lock().unwrap();
                    while !state.stop && state.next.is_none() {
                        state = cv.wait(state).unwrap();
                    }
                    if state.stop {
                        break;
                    }
                    state.busy = true;
                    state.next.take().unwrap()
                };
                let cancelled = || token.load(Ordering::Acquire) != request.ticket;
                let result = if cancelled() {
                    Err("Cancelled before read".into())
                } else {
                    read(&request.root, request.document, &cancelled)
                };
                let result = if cancelled() {
                    Err("Cancelled after read".into())
                } else {
                    result
                };
                // Notify before rendezvous. Consumers poll while a request is pending;
                // a repaint can race this send, so notification alone is not a latch.
                wake();
                if tx
                    .send(Reply {
                        ticket: request.ticket,
                        result,
                    })
                    .is_err()
                {
                    break;
                }
                work.0.lock().unwrap().busy = false;
            }
        });
        Self {
            generation,
            mailbox,
            rx: Some(rx),
            worker: Some(worker),
        }
    }
    pub fn request(&mut self, root: PathBuf, document: usize) -> Result<u64, String> {
        if document >= 10 {
            return Err("Document outside standard fixture".into());
        }
        let ticket = self.cancel()?;
        self.mailbox.0.lock().unwrap().next = Some(Request {
            ticket,
            root,
            document,
        });
        self.mailbox.1.notify_one();
        Ok(ticket)
    }
    pub fn cancel(&mut self) -> Result<u64, String> {
        let next = self
            .generation
            .load(Ordering::Acquire)
            .checked_add(1)
            .ok_or("Generation exhausted")?;
        self.generation.store(next, Ordering::Release);
        self.mailbox.0.lock().unwrap().next = None;
        Ok(next)
    }
    /// Cancellation invalidates a result; it does not complete physical I/O or
    /// release a worker blocked on result handoff. Keep polling until drained.
    pub fn is_idle(&self) -> bool {
        let state = self.mailbox.0.lock().unwrap();
        !state.busy && state.next.is_none()
    }
    pub fn poll(&mut self) -> Option<Event> {
        let reply = self.rx.as_ref()?.try_recv().ok()?;
        Some(if reply.ticket == self.generation.load(Ordering::Acquire) {
            Event::Ready {
                ticket: reply.ticket,
                result: reply.result,
            }
        } else {
            Event::Stale {
                ticket: reply.ticket,
            }
        })
    }
}
impl Drop for DocumentLoader {
    fn drop(&mut self) {
        self.generation.fetch_add(1, Ordering::Release);
        self.mailbox.0.lock().unwrap().stop = true;
        self.mailbox.1.notify_one();
        self.rx.take(); // Release a blocked result sender before join.
        if let Some(worker) = self.worker.take() {
            worker.join().expect("document worker panicked");
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::composition::Image;
    use std::time::{Duration, Instant};
    fn loaded() -> LoadedDocument {
        LoadedDocument {
            sources: vec![Image {
                size: [1, 1],
                rgba: vec![[200, 80, 40, 128]],
            }]
            .into(),
            nodes: vec![],
        }
    }
    fn event(loader: &mut DocumentLoader) -> Event {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(e) = loader.poll() {
                return e;
            }
            assert!(Instant::now() < deadline);
            thread::yield_now();
        }
    }
    #[test]
    fn newest_request_wins_and_invalid_request_does_not_cancel_it() {
        let (started, seen) = mpsc::channel();
        let (release, blocked) = mpsc::channel();
        let mut loader = DocumentLoader::with_reader(
            || {},
            move |_, doc, _| {
                started.send(doc).unwrap();
                if doc == 0 {
                    blocked.recv().unwrap();
                }
                Ok(loaded())
            },
        );
        let old = loader.request(PathBuf::new(), 0).unwrap();
        assert_eq!(seen.recv_timeout(Duration::from_secs(5)).unwrap(), 0);
        loader.request(PathBuf::new(), 1).unwrap();
        let newest = loader.request(PathBuf::new(), 2).unwrap();
        assert!(loader.request(PathBuf::new(), 10).is_err());
        release.send(()).unwrap();
        assert!(matches!(event(&mut loader), Event::Stale {ticket} if ticket==old));
        assert_eq!(seen.recv_timeout(Duration::from_secs(5)).unwrap(), 2);
        match event(&mut loader) {
            Event::Ready { ticket, result } => {
                assert_eq!(ticket, newest);
                assert_eq!(result.unwrap().sources[0].rgba[0], [200, 80, 40, 128]);
            }
            _ => panic!("new result discarded"),
        }
    }
    #[test]
    fn cancel_discards_pending_result_and_errors_are_not_success() {
        let (sent, waiting) = mpsc::channel();
        let mut loader = DocumentLoader::with_reader(
            move || {
                sent.send(()).unwrap();
            },
            |_, _, _| Err("missing PNG".into()),
        );
        let old = loader.request(PathBuf::new(), 0).unwrap();
        waiting.recv_timeout(Duration::from_secs(5)).unwrap();
        loader.cancel().unwrap();
        assert!(!loader.is_idle());
        assert!(matches!(event(&mut loader),Event::Stale {ticket} if ticket==old));
        let deadline = Instant::now() + Duration::from_secs(5);
        while !loader.is_idle() {
            assert!(Instant::now() < deadline);
            thread::yield_now();
        }
        loader.request(PathBuf::new(), 1).unwrap();
        assert!(matches!(
            event(&mut loader),
            Event::Ready { result: Err(_), .. }
        ));
    }
    #[test]
    fn result_backpressure_and_drop_release_worker_without_a_detached_thread() {
        let (started, seen) = mpsc::channel();
        let (wake, ready) = mpsc::channel();
        let mut loader = DocumentLoader::with_reader(
            move || {
                wake.send(()).unwrap();
            },
            move |_, doc, _| {
                started.send(doc).unwrap();
                Ok(loaded())
            },
        );
        loader.request(PathBuf::new(), 0).unwrap();
        seen.recv_timeout(Duration::from_secs(5)).unwrap();
        ready.recv_timeout(Duration::from_secs(5)).unwrap();
        loader.request(PathBuf::new(), 1).unwrap();
        assert!(seen.try_recv().is_err());
        drop(loader); // Sender is still waiting for the first result to be read.
        assert!(seen.try_recv().is_err());
    }
}
