//! Optional Windows-only P0 output adapter; no UI, project or game dependencies.
pub mod buffered;
pub mod pcm;
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Initializing,
    Paused,
    Playing,
    Faulted,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Error {
    pub operation: &'static str,
    pub code: Option<i32>,
}
#[derive(Clone, Copy, Debug)]
pub struct Snapshot {
    pub epoch: u64,
    pub phase: Phase,
    pub request: u64,
    pub sequence: u64,
    pub position_us: u64,
    pub observed_elapsed_us: u64,
    pub error: Option<Error>,
    pub format: Option<Format>,
    pub pcm: Option<pcm::Stats>,
    pub buffered: Option<buffered::Stats>,
}
#[derive(Clone, Copy, Debug)]
pub struct Format {
    /// Client buffer format; source frames and padding use this rate.
    pub rate: u32,
    pub channels: u16,
    pub bits: u16,
    pub buffer_frames: u32,
    pub clock_frequency: u64,
    pub mix_rate: u32,
    pub mix_channels: u16,
    pub conversion_enabled: bool,
}
impl Default for Snapshot {
    fn default() -> Self {
        Self {
            epoch: 0,
            phase: Phase::Initializing,
            request: 0,
            sequence: 0,
            position_us: 0,
            observed_elapsed_us: 0,
            error: None,
            format: None,
            pcm: None,
            buffered: None,
        }
    }
}
enum Control {
    State {
        epoch: u64,
        request: u64,
        paused: bool,
    },
    Rebuild {
        epoch: u64,
        position_us: u64,
    },
    Invalidate {
        epoch: u64,
    },
}
enum Content {
    Silent,
    Generated,
    Clip(Arc<pcm::Clip>),
    Buffered {
        input: buffered::Receiver,
        convert: bool,
    },
}
impl Content {
    fn duration_us(&self) -> u64 {
        match self {
            Self::Clip(c) => c.duration_us(),
            Self::Buffered { input, .. } => input.duration_us(),
            _ => pcm::DURATION_US,
        }
    }
}
pub struct Device {
    tx: mpsc::SyncSender<Control>,
    snapshot: Arc<Mutex<Snapshot>>,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
    duration_us: u64,
}
impl Device {
    pub fn spawn(wake: impl Fn() + Send + Sync + 'static) -> Result<Self, Error> {
        Self::spawn_mode(wake, Content::Silent)
    }
    /// Explicit opt-in diagnostic output, low-amplitude generated float PCM.
    pub fn spawn_pcm(wake: impl Fn() + Send + Sync + 'static) -> Result<Self, Error> {
        Self::spawn_mode(wake, Content::Generated)
    }
    pub fn spawn_clip(
        clip: Arc<pcm::Clip>,
        wake: impl Fn() + Send + Sync + 'static,
    ) -> Result<Self, Error> {
        Self::spawn_mode(wake, Content::Clip(clip))
    }
    pub fn spawn_buffered(
        input: buffered::Receiver,
        wake: impl Fn() + Send + Sync + 'static,
    ) -> Result<Self, Error> {
        Self::spawn_mode(
            wake,
            Content::Buffered {
                input,
                convert: false,
            },
        )
    }
    /// Opt-in Windows shared-engine conversion and default-endpoint monitoring.
    /// No system format, endpoint or volume settings are changed.
    pub fn spawn_buffered_converting(
        input: buffered::Receiver,
        wake: impl Fn() + Send + Sync + 'static,
    ) -> Result<Self, Error> {
        Self::spawn_mode(
            wake,
            Content::Buffered {
                input,
                convert: true,
            },
        )
    }
    fn spawn_mode(
        wake: impl Fn() + Send + Sync + 'static,
        content: Content,
    ) -> Result<Self, Error> {
        let duration_us = content.duration_us();
        let (tx, rx) = mpsc::sync_channel(8);
        let snapshot = Arc::new(Mutex::new(Snapshot::default()));
        let stop = Arc::new(AtomicBool::new(false));
        let output = snapshot.clone();
        let quitting = stop.clone();
        let thread = thread::Builder::new()
            .name("p0-audio-output".into())
            .spawn(move || {
                let started = Instant::now();
                let publish = |mut s: Snapshot| {
                    let mut current = output.lock().unwrap_or_else(|e| e.into_inner());
                    let notify = current.phase != s.phase
                        || current.epoch != s.epoch
                        || current.request != s.request
                        || current.error != s.error
                        || (s.phase == Phase::Paused && current.position_us != s.position_us);
                    s.sequence = current.sequence + 1;
                    s.observed_elapsed_us = started.elapsed().as_micros() as u64;
                    *current = s;
                    drop(current);
                    if notify {
                        wake();
                    }
                };
                if let Err(error) = worker(rx, quitting, &publish, content) {
                    let mut s = *output.lock().unwrap_or_else(|e| e.into_inner());
                    s.phase = Phase::Faulted;
                    s.error = Some(error);
                    publish(s);
                }
            })
            .map_err(|_| Error {
                operation: "spawn output worker",
                code: None,
            })?;
        Ok(Self {
            tx,
            snapshot,
            stop,
            thread: Some(thread),
            duration_us,
        })
    }
    pub fn request(&self, request: u64, paused: bool) -> Result<(), Error> {
        self.request_in(0, request, paused)
    }
    pub fn request_in(&self, epoch: u64, request: u64, paused: bool) -> Result<(), Error> {
        self.send(Control::State {
            epoch,
            request,
            paused,
        })
    }
    /// Retire the owned stream and open a newer epoch paused, on the same worker.
    pub fn rebuild(&self, epoch: u64) -> Result<(), Error> {
        self.relocate(epoch, 0)
    }
    pub fn relocate(&self, epoch: u64, position_us: u64) -> Result<(), Error> {
        if position_us > self.duration_us {
            return Err(Error {
                operation: "PCM relocation outside fixture",
                code: None,
            });
        }
        self.send(Control::Rebuild { epoch, position_us })
    }
    /// Explicitly retire our stream, for controlled lifecycle diagnostics.
    /// This does not simulate a driver HRESULT or change system endpoint settings.
    pub fn invalidate(&self, epoch: u64) -> Result<(), Error> {
        self.send(Control::Invalidate { epoch })
    }
    fn send(&self, control: Control) -> Result<(), Error> {
        self.tx.try_send(control).map_err(|_| Error {
            operation: "output control queue full or closed",
            code: None,
        })
    }
    pub fn snapshot(&self) -> Snapshot {
        *self.snapshot.lock().unwrap_or_else(|e| e.into_inner())
    }
}
impl Drop for Device {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
#[cfg(windows)]
fn worker(
    rx: mpsc::Receiver<Control>,
    stop: Arc<AtomicBool>,
    publish: &impl Fn(Snapshot),
    content: Content,
) -> Result<(), Error> {
    run_worker(rx, stop, publish, |position| {
        platform::Stream::open_content(&content, position)
    });
    if let Content::Buffered { input, .. } = &content {
        input.0.close();
    }
    Ok(())
}

#[cfg(any(windows, test))]
trait StreamPort {
    fn format(&self) -> Format;
    fn pause(&mut self, paused: bool) -> Result<(), Error>;
    fn feed(&mut self) -> Result<(), Error>;
    fn position(&self) -> Result<u64, Error>;
    fn check_device(&mut self) -> Result<(), Error> {
        Ok(())
    }
    fn ready(&self) -> Result<bool, Error> {
        Ok(true)
    }
    fn buffered(&self) -> Option<buffered::Stats> {
        None
    }
    fn pcm(&self) -> Option<pcm::Stats> {
        None
    }
}
#[cfg(windows)]
impl StreamPort for platform::Stream {
    fn check_device(&mut self) -> Result<(), Error> {
        self.check_device()
    }
    fn format(&self) -> Format {
        self.format
    }
    fn pause(&mut self, paused: bool) -> Result<(), Error> {
        self.pause(paused)
    }
    fn feed(&mut self) -> Result<(), Error> {
        self.feed()
    }
    fn position(&self) -> Result<u64, Error> {
        self.position()
    }
    fn pcm(&self) -> Option<pcm::Stats> {
        self.pcm_stats()
    }
    fn ready(&self) -> Result<bool, Error> {
        self.ready()
    }
    fn buffered(&self) -> Option<buffered::Stats> {
        self.buffered_stats()
    }
}

#[cfg(any(windows, test))]
fn run_worker<S: StreamPort>(
    rx: mpsc::Receiver<Control>,
    stop: Arc<AtomicBool>,
    publish: &impl Fn(Snapshot),
    mut open: impl FnMut(u64) -> Result<S, Error>,
) {
    let mut snapshot = Snapshot::default();
    let mut stream = None;
    let mut pending_play = None;
    let fault = |s: &mut Snapshot, stream: &mut Option<S>, error| {
        // Close the input gate before potentially slow native cleanup. The same
        // worker still finishes releasing this stream before opening its successor.
        s.phase = Phase::Faulted;
        s.error = Some(error);
        s.buffered = stream.as_ref().and_then(StreamPort::buffered);
        publish(*s);
        *stream = None;
    };
    let opening = |s: &mut Snapshot,
                   stream: &mut Option<S>,
                   position,
                   open: &mut dyn FnMut(u64) -> Result<S, Error>| {
        match open(position) {
            Ok(value) => {
                s.phase = Phase::Paused;
                s.format = Some(value.format());
                s.pcm = value.pcm();
                s.buffered = value.buffered();
                *stream = Some(value);
                publish(*s);
            }
            Err(error) => fault(s, stream, error),
        }
    };
    opening(&mut snapshot, &mut stream, 0, &mut open);
    while !stop.load(Ordering::Acquire) {
        let timeout = if snapshot.phase == Phase::Playing || pending_play.is_some() {
            Duration::from_millis(5)
        } else {
            Duration::from_millis(100)
        };
        match rx.recv_timeout(timeout) {
            Ok(Control::Rebuild { epoch, position_us }) if epoch > snapshot.epoch => {
                pending_play = None;
                stream = None;
                snapshot = Snapshot {
                    epoch,
                    ..Default::default()
                };
                publish(snapshot);
                opening(&mut snapshot, &mut stream, position_us, &mut open);
            }
            Ok(Control::Invalidate { epoch }) if epoch == snapshot.epoch => {
                pending_play = None;
                fault(
                    &mut snapshot,
                    &mut stream,
                    Error {
                        operation: "explicit stream invalidation",
                        code: None,
                    },
                );
            }
            Ok(Control::State {
                epoch,
                request,
                paused,
            }) if epoch == snapshot.epoch
                && request
                    > pending_play
                        .unwrap_or(snapshot.request)
                        .max(snapshot.request) =>
            {
                if let Some(value) = &mut stream {
                    pending_play = None;
                    if !paused {
                        pending_play = Some(request);
                    } else if let Err(error) = value.pause(true) {
                        fault(&mut snapshot, &mut stream, error);
                    } else {
                        snapshot.request = request;
                        snapshot.phase = Phase::Paused;
                    }
                }
            }
            Ok(_) | Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
        if let Some(value) = &mut stream {
            let result = (|| {
                // Check even while paused. Device changes must never enable input
                // or implicitly resume a replacement stream.
                value.check_device()?;
                if let Some(request) = pending_play
                    && value.ready()?
                {
                    value.pause(false)?;
                    snapshot.request = request;
                    snapshot.phase = Phase::Playing;
                    pending_play = None;
                }
                if snapshot.phase == Phase::Playing {
                    value.feed()?;
                }
                snapshot.position_us = value.position()?;
                snapshot.pcm = value.pcm();
                snapshot.buffered = value.buffered();
                Ok(())
            })();
            match result {
                Ok(()) => publish(snapshot),
                Err(error) => {
                    pending_play = None;
                    fault(&mut snapshot, &mut stream, error);
                }
            }
        }
    }
}
#[cfg(not(windows))]
fn worker(
    _: mpsc::Receiver<Control>,
    _: Arc<AtomicBool>,
    _: &impl Fn(Snapshot),
    _: Content,
) -> Result<(), Error> {
    Err(Error {
        operation: "Windows WASAPI unavailable",
        code: None,
    })
}

#[cfg(windows)]
mod platform;

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;
    struct Fake {
        drops: Arc<AtomicUsize>,
        playing: bool,
        broken: bool,
        drop_gate: Option<mpsc::Receiver<()>>,
    }
    impl Drop for Fake {
        fn drop(&mut self) {
            if let Some(gate) = &self.drop_gate {
                gate.recv_timeout(Duration::from_secs(5)).unwrap();
            }
            self.drops.fetch_add(1, Ordering::SeqCst);
        }
    }
    impl StreamPort for Fake {
        fn format(&self) -> Format {
            Format {
                rate: 48000,
                channels: 2,
                bits: 32,
                buffer_frames: 4800,
                clock_frequency: 48000,
                mix_rate: 48000,
                mix_channels: 2,
                conversion_enabled: false,
            }
        }
        fn pause(&mut self, paused: bool) -> Result<(), Error> {
            self.playing = !paused;
            Ok(())
        }
        fn feed(&mut self) -> Result<(), Error> {
            if self.broken {
                Err(Error {
                    operation: "fake feed failure",
                    code: Some(-1),
                })
            } else {
                Ok(())
            }
        }
        fn position(&self) -> Result<u64, Error> {
            Ok(if self.playing { 100 } else { 0 })
        }
    }
    fn receive(rx: &mpsc::Receiver<Snapshot>, predicate: impl Fn(&Snapshot) -> bool) -> Snapshot {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            let s = rx
                .recv_timeout(deadline.saturating_duration_since(Instant::now()))
                .unwrap();
            if predicate(&s) {
                return s;
            }
        }
    }
    #[test]
    fn changed_device_faults_during_pause_or_play_and_failed_reopen_never_resumes() {
        struct Monitored {
            inner: Fake,
            changed: Arc<AtomicBool>,
            mix_rate: u32,
        }
        impl StreamPort for Monitored {
            fn format(&self) -> Format {
                Format {
                    mix_rate: self.mix_rate,
                    mix_channels: 1,
                    conversion_enabled: true,
                    ..self.inner.format()
                }
            }
            fn pause(&mut self, paused: bool) -> Result<(), Error> {
                self.inner.pause(paused)
            }
            fn feed(&mut self) -> Result<(), Error> {
                self.inner.feed()
            }
            fn position(&self) -> Result<u64, Error> {
                self.inner.position()
            }
            fn check_device(&mut self) -> Result<(), Error> {
                if self.changed.load(Ordering::Acquire) {
                    Err(Error {
                        operation: "fake changed endpoint",
                        code: None,
                    })
                } else {
                    Ok(())
                }
            }
        }
        for playing in [false, true] {
            let (tx, rx) = mpsc::channel();
            let (out, states) = mpsc::channel();
            let changed = Arc::new(AtomicBool::new(false));
            let flag = changed.clone();
            let drops = Arc::new(AtomicUsize::new(0));
            let releases = drops.clone();
            let worker = thread::spawn(move || {
                let mut opens = 0;
                run_worker(
                    rx,
                    Arc::new(AtomicBool::new(false)),
                    &|s| out.send(s).unwrap(),
                    |position| {
                        opens += 1;
                        if opens > 1 {
                            assert_eq!(position, 123456);
                            assert_eq!(releases.load(Ordering::SeqCst), 1);
                        }
                        if opens == 2 {
                            return Err(Error {
                                operation: "fake endpoint still absent",
                                code: None,
                            });
                        }
                        Ok(Monitored {
                            inner: Fake {
                                drops: releases.clone(),
                                playing: false,
                                broken: false,
                                drop_gate: None,
                            },
                            changed: flag.clone(),
                            mix_rate: if opens == 1 { 48000 } else { 44100 },
                        })
                    },
                );
            });
            receive(&states, |s| s.phase == Phase::Paused);
            if playing {
                tx.send(Control::State {
                    epoch: 0,
                    request: 1,
                    paused: false,
                })
                .unwrap();
                receive(&states, |s| s.phase == Phase::Playing);
            }
            changed.store(true, Ordering::Release);
            receive(&states, |s| s.phase == Phase::Faulted);
            tx.send(Control::State {
                epoch: 0,
                request: 99,
                paused: false,
            })
            .unwrap();
            tx.send(Control::Rebuild {
                epoch: 1,
                position_us: 123456,
            })
            .unwrap();
            receive(&states, |s| s.epoch == 1 && s.phase == Phase::Faulted);
            changed.store(false, Ordering::Release);
            tx.send(Control::Rebuild {
                epoch: 2,
                position_us: 123456,
            })
            .unwrap();
            let paused = receive(&states, |s| s.epoch == 2 && s.phase == Phase::Paused);
            assert_eq!(paused.request, 0);
            assert_eq!(paused.format.unwrap().mix_rate, 44100);
            tx.send(Control::State {
                epoch: 0,
                request: 100,
                paused: false,
            })
            .unwrap();
            tx.send(Control::State {
                epoch: 2,
                request: 1,
                paused: true,
            })
            .unwrap();
            let still = receive(&states, |s| s.epoch == 2 && s.request == 1);
            assert_eq!(still.phase, Phase::Paused);
            tx.send(Control::State {
                epoch: 2,
                request: 2,
                paused: false,
            })
            .unwrap();
            receive(&states, |s| s.epoch == 2 && s.phase == Phase::Playing);
            drop(tx);
            worker.join().unwrap();
            assert_eq!(drops.load(Ordering::SeqCst), 2);
        }
    }
    #[test]
    fn rebuilding_releases_stream_before_open_and_rejects_old_epoch_controls() {
        let (tx, rx) = mpsc::channel();
        let (out, states) = mpsc::channel();
        let stop = Arc::new(AtomicBool::new(false));
        let quitting = stop.clone();
        let drops = Arc::new(AtomicUsize::new(0));
        let releases = drops.clone();
        let worker = thread::spawn(move || {
            let mut opens = 0;
            run_worker(rx, quitting, &|s| out.send(s).unwrap(), |position| {
                assert_eq!(releases.load(Ordering::SeqCst), opens);
                assert_eq!(position, if opens == 0 { 0 } else { 375000 });
                opens += 1;
                Ok(Fake {
                    drops: releases.clone(),
                    playing: false,
                    broken: false,
                    drop_gate: None,
                })
            });
        });
        receive(&states, |s| s.phase == Phase::Paused);
        tx.send(Control::State {
            epoch: 0,
            request: 1,
            paused: false,
        })
        .unwrap();
        receive(&states, |s| s.phase == Phase::Playing);
        tx.send(Control::Invalidate { epoch: 0 }).unwrap();
        receive(&states, |s| s.phase == Phase::Faulted);
        tx.send(Control::Rebuild {
            epoch: 1,
            position_us: 375000,
        })
        .unwrap();
        receive(&states, |s| s.epoch == 1 && s.phase == Phase::Paused);
        tx.send(Control::State {
            epoch: 0,
            request: 999,
            paused: false,
        })
        .unwrap();
        tx.send(Control::Invalidate { epoch: 0 }).unwrap();
        tx.send(Control::Rebuild {
            epoch: 0,
            position_us: 0,
        })
        .unwrap();
        let s = receive(&states, |s| s.epoch == 1 && s.phase == Phase::Paused);
        assert_eq!((s.request, s.position_us), (0, 0));
        tx.send(Control::State {
            epoch: 1,
            request: 1,
            paused: false,
        })
        .unwrap();
        receive(&states, |s| s.epoch == 1 && s.phase == Phase::Playing);
        stop.store(true, Ordering::Release);
        drop(tx);
        worker.join().unwrap();
        assert_eq!(drops.load(Ordering::SeqCst), 2);
    }
    #[test]
    fn failed_open_and_runtime_failure_stay_faulted_until_explicit_retry() {
        let (tx, rx) = mpsc::channel();
        let (out, states) = mpsc::channel();
        let stop = Arc::new(AtomicBool::new(false));
        let quitting = stop.clone();
        let drops = Arc::new(AtomicUsize::new(0));
        let releases = drops.clone();
        let worker = thread::spawn(move || {
            let mut opens = 0;
            run_worker(rx, quitting, &|s| out.send(s).unwrap(), |_| {
                opens += 1;
                if opens <= 2 {
                    return Err(Error {
                        operation: "fake open failure",
                        code: Some(-2),
                    });
                }
                Ok(Fake {
                    drops: releases.clone(),
                    playing: false,
                    broken: opens == 3,
                    drop_gate: None,
                })
            });
        });
        receive(&states, |s| s.epoch == 0 && s.phase == Phase::Faulted);
        tx.send(Control::Rebuild {
            epoch: 1,
            position_us: 0,
        })
        .unwrap();
        receive(&states, |s| s.epoch == 1 && s.phase == Phase::Faulted);
        tx.send(Control::Rebuild {
            epoch: 2,
            position_us: 0,
        })
        .unwrap();
        receive(&states, |s| s.epoch == 2 && s.phase == Phase::Paused);
        tx.send(Control::State {
            epoch: 2,
            request: 1,
            paused: false,
        })
        .unwrap();
        let failure = receive(&states, |s| s.epoch == 2 && s.phase == Phase::Faulted);
        assert_eq!(failure.error.unwrap().operation, "fake feed failure");
        tx.send(Control::Rebuild {
            epoch: 3,
            position_us: 0,
        })
        .unwrap();
        receive(&states, |s| s.epoch == 3 && s.phase == Phase::Paused);
        drop(tx);
        worker.join().unwrap();
        assert_eq!(drops.load(Ordering::SeqCst), 2);
    }
    #[test]
    fn fault_is_observable_while_old_native_stream_cleanup_is_blocked() {
        let (tx, rx) = mpsc::channel();
        let (out, states) = mpsc::channel();
        let (release, gate) = mpsc::channel();
        let drops = Arc::new(AtomicUsize::new(0));
        let releases = drops.clone();
        let worker = thread::spawn(move || {
            let mut gate = Some(gate);
            run_worker(
                rx,
                Arc::new(AtomicBool::new(false)),
                &|s| out.send(s).unwrap(),
                |_| {
                    Ok(Fake {
                        drops: releases.clone(),
                        playing: false,
                        broken: false,
                        drop_gate: gate.take(),
                    })
                },
            );
        });
        receive(&states, |s| s.phase == Phase::Paused);
        tx.send(Control::Invalidate { epoch: 0 }).unwrap();
        receive(&states, |s| s.phase == Phase::Faulted);
        assert_eq!(drops.load(Ordering::SeqCst), 0);
        // The caller remains free to process edits or queue recovery while cleanup waits.
        tx.send(Control::Rebuild {
            epoch: 1,
            position_us: 0,
        })
        .unwrap();
        release.send(()).unwrap();
        receive(&states, |s| s.epoch == 1 && s.phase == Phase::Paused);
        assert_eq!(drops.load(Ordering::SeqCst), 1);
        drop(tx);
        worker.join().unwrap();
        assert_eq!(drops.load(Ordering::SeqCst), 2);
    }
    #[test]
    fn play_waits_for_prefill_and_newer_pause_cancels_pending_start() {
        struct Waiting {
            ready: Arc<AtomicBool>,
            playing: bool,
        }
        impl StreamPort for Waiting {
            fn format(&self) -> Format {
                Format {
                    rate: 48000,
                    channels: 2,
                    bits: 32,
                    buffer_frames: 4800,
                    clock_frequency: 48000,
                    mix_rate: 48000,
                    mix_channels: 2,
                    conversion_enabled: false,
                }
            }
            fn pause(&mut self, paused: bool) -> Result<(), Error> {
                self.playing = !paused;
                Ok(())
            }
            fn ready(&self) -> Result<bool, Error> {
                Ok(self.ready.load(Ordering::Acquire))
            }
            fn feed(&mut self) -> Result<(), Error> {
                Ok(())
            }
            fn position(&self) -> Result<u64, Error> {
                Ok(u64::from(self.playing))
            }
        }
        let (tx, rx) = mpsc::channel();
        let (out, states) = mpsc::channel();
        let ready = Arc::new(AtomicBool::new(false));
        let gate = ready.clone();
        let worker = thread::spawn(move || {
            run_worker(
                rx,
                Arc::new(AtomicBool::new(false)),
                &|s| out.send(s).unwrap(),
                |_| {
                    Ok(Waiting {
                        ready: gate.clone(),
                        playing: false,
                    })
                },
            )
        });
        receive(&states, |s| s.phase == Phase::Paused);
        tx.send(Control::State {
            epoch: 0,
            request: 1,
            paused: false,
        })
        .unwrap();
        let s = receive(&states, |s| s.phase == Phase::Paused);
        assert_eq!((s.request, s.position_us), (0, 0));
        tx.send(Control::State {
            epoch: 0,
            request: 2,
            paused: true,
        })
        .unwrap();
        receive(&states, |s| s.request == 2);
        ready.store(true, Ordering::Release);
        tx.send(Control::State {
            epoch: 0,
            request: 1,
            paused: false,
        })
        .unwrap();
        let s = receive(&states, |s| s.request == 2);
        assert_eq!(s.phase, Phase::Paused);
        tx.send(Control::State {
            epoch: 0,
            request: 3,
            paused: false,
        })
        .unwrap();
        receive(&states, |s| s.request == 3 && s.phase == Phase::Playing);
        drop(tx);
        worker.join().unwrap();
    }
}
