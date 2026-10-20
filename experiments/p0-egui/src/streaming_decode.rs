//! P0 assembly only: one decoder worker, bounded PCM values to the output adapter.
#[cfg(feature = "compressed-proof")]
use p0_media::compressed::Stream;
#[cfg(not(feature = "compressed-proof"))]
use p0_media::stream::Stream;
use p0_wasapi::buffered::Input;
use std::{
    io::{Read, Seek},
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

#[derive(Default)]
struct GateState {
    blocked: bool,
    held: Option<(u64, u64)>,
}
/// Explicit diagnostic stall after decoding, before publishing. Never a game state.
#[derive(Clone, Default)]
pub struct DiagnosticGate(Arc<(Mutex<GateState>, Condvar)>);
impl DiagnosticGate {
    pub fn hold(&self) {
        let mut s = self.0.0.lock().unwrap();
        s.blocked = true;
        s.held = None;
    }
    pub fn release(&self) {
        self.0.0.lock().unwrap().blocked = false;
        self.0.1.notify_all();
    }
    pub fn held(&self) -> Option<(u64, u64)> {
        self.0.0.lock().unwrap().held
    }
    fn wait(&self, generation: u64, frame: u64, stop: &AtomicBool) {
        let mut s = self.0.0.lock().unwrap();
        if s.blocked {
            s.held = Some((generation, frame));
        }
        while s.blocked && !stop.load(Ordering::Acquire) {
            s = self
                .0
                .1
                .wait_timeout(s, Duration::from_millis(100))
                .unwrap()
                .0;
        }
    }
}
pub struct DecoderWorker {
    input: Input,
    gate: DiagnosticGate,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}
impl DecoderWorker {
    /// The immutable source is reopened on generation changes, including failures.
    /// Factory executes only on this worker; no callback or file enters the output.
    pub fn spawn<S, F>(input: Input, gate: DiagnosticGate, mut open: F) -> std::io::Result<Self>
    where
        S: Read + Seek + Send + Sync + 'static,
        F: FnMut() -> std::io::Result<S> + Send + 'static,
    {
        let stop = Arc::new(AtomicBool::new(false));
        let quitting = stop.clone();
        let feed = input.clone();
        let barrier = gate.clone();
        let thread = thread::Builder::new()
            .name("p0-stream-decoder".into())
            .spawn(move || {
                let mut decoder = None;
                let mut generation = None;
                while !quitting.load(Ordering::Acquire) && !feed.is_closed() {
                    let Some(mut work) = feed.work(Duration::from_millis(100)) else {
                        continue;
                    };
                    let work_generation = work.generation();
                    let current =
                        || !quitting.load(Ordering::Acquire) && feed.is_current(work_generation);
                    if !current() {
                        feed.fail(Some(work));
                        continue;
                    }
                    let config = feed.config();
                    if generation != Some(work.generation()) {
                        decoder = open()
                            .ok()
                            .and_then(|source| Stream::open(source).ok())
                            .filter(|s| {
                                s.rate() == config.rate
                                    && s.channels() == config.channels
                                    && s.total_frames() == config.total_frames
                            });
                        generation = Some(work.generation());
                        let seek = decoder.as_mut().is_some_and(|s| {
                            #[cfg(feature = "compressed-proof")]
                            while current() {
                                match s.seek_step(work.first_frame()) {
                                    Ok(true) => return true,
                                    Ok(false) => (),
                                    Err(_) => return false,
                                }
                            }
                            #[cfg(not(feature = "compressed-proof"))]
                            if current() {
                                return s.seek(work.first_frame()).is_ok();
                            }
                            false
                        });
                        if !seek || !current() {
                            feed.fail(Some(work));
                            decoder = None;
                            continue;
                        }
                    }
                    let result = decoder
                        .as_mut()
                        .and_then(|s| s.read(work.samples_mut()).ok());
                    let Some(block) = result else {
                        feed.fail(Some(work));
                        decoder = None;
                        continue;
                    };
                    if block.first_frame != work.first_frame() {
                        feed.fail(Some(work));
                        decoder = None;
                        continue;
                    }
                    if !current() {
                        feed.fail(Some(work));
                        decoder = None;
                        continue;
                    }
                    barrier.wait(work.generation(), work.first_frame(), &quitting);
                    let _ = feed.complete(work, block.frames, block.eof);
                }
            })?;
        Ok(Self {
            input,
            gate,
            stop,
            thread: Some(thread),
        })
    }
}
impl Drop for DecoderWorker {
    fn drop(&mut self) {
        self.input.close();
        self.stop.store(true, Ordering::Release);
        self.gate.release();
        self.input.wake();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use p0_wasapi::buffered::{self, Config, Receiver};
    use std::{io::Cursor, sync::atomic::AtomicUsize, time::Instant};
    fn wav() -> Vec<u8> {
        let size = 48000u32 * 2 * 2;
        let mut b = Vec::new();
        b.extend(b"RIFF");
        b.extend((size + 36).to_le_bytes());
        b.extend(b"WAVEfmt ");
        b.extend(16u32.to_le_bytes());
        b.extend(1u16.to_le_bytes());
        b.extend(2u16.to_le_bytes());
        b.extend(48000u32.to_le_bytes());
        b.extend(192000u32.to_le_bytes());
        b.extend(4u16.to_le_bytes());
        b.extend(16u16.to_le_bytes());
        b.extend(b"data");
        b.extend(size.to_le_bytes());
        for n in 0..48000i32 {
            b.extend(((n % 16384) as i16).to_le_bytes());
            b.extend((-(n % 16384) as i16).to_le_bytes());
        }
        b
    }
    fn channel() -> (Input, Receiver) {
        buffered::channel(Config {
            rate: 48000,
            channels: 2,
            total_frames: 48000,
            gain: 0.02,
        })
        .unwrap()
    }
    fn wait(mut condition: impl FnMut() -> bool) {
        let deadline = Instant::now() + Duration::from_secs(3);
        while !condition() {
            assert!(Instant::now() < deadline);
            thread::sleep(Duration::from_millis(1));
        }
    }
    #[test]
    fn decoder_thread_backpressure_reuses_pool_and_shutdown_releases_waiter() {
        let (input, output) = channel();
        let gate = DiagnosticGate::default();
        let bytes = wav();
        let worker =
            DecoderWorker::spawn(input.clone(), gate, move || Ok(Cursor::new(bytes.clone())))
                .unwrap();
        output.activate(0).unwrap();
        wait(|| input.stats().queued_blocks == 4);
        let before = input.stats().accepted_blocks;
        thread::sleep(Duration::from_millis(25));
        assert_eq!(input.stats().accepted_blocks, before);
        let mut out = vec![0; 4096 * 8];
        let p = output.prepare(4096, &mut out).unwrap().unwrap();
        output.commit(p).unwrap();
        wait(|| input.stats().accepted_blocks > before);
        drop(worker);
        assert!(input.is_closed());
        assert_eq!(input.stats().free_blocks, buffered::POOL_BLOCKS);
    }
    #[test]
    fn pause_and_seek_reject_stalled_completions_and_new_source_frame_is_exact() {
        let (input, output) = channel();
        let gate = DiagnosticGate::default();
        let bytes = wav();
        let worker = DecoderWorker::spawn(input.clone(), gate.clone(), move || {
            Ok(Cursor::new(bytes.clone()))
        })
        .unwrap();
        gate.hold();
        output.activate(0).unwrap();
        wait(|| gate.held().is_some());
        output.pause_at(100001, 0).unwrap();
        gate.release();
        wait(|| input.stats().stale_blocks == 1 && input.stats().queued_blocks == 4);
        let mut out = vec![0; 16 * 8];
        let p = output.prepare(16, &mut out).unwrap().unwrap();
        assert_eq!(p.packet.source_frame, 4800);
        assert_eq!(
            f32::from_le_bytes(out[..4].try_into().unwrap()),
            4800. / 32768. * 0.02
        );
        gate.hold();
        output.activate(700000).unwrap();
        wait(|| gate.held().is_some());
        output.activate(7).unwrap();
        gate.release();
        wait(|| input.stats().stale_blocks == 2 && input.stats().queued_blocks == 4);
        let p = output.prepare(16, &mut out).unwrap().unwrap();
        assert_eq!(p.packet.source_frame, 0);
        assert_eq!(out[..8], [0; 8]);
        drop(worker);
    }
    #[test]
    fn source_open_failure_remains_faulted_until_new_generation_and_can_reopen() {
        let (input, output) = channel();
        let attempts = Arc::new(AtomicUsize::new(0));
        let count = attempts.clone();
        let bytes = wav();
        let worker = DecoderWorker::spawn(input.clone(), DiagnosticGate::default(), move || {
            if count.fetch_add(1, Ordering::SeqCst) == 0 {
                Err(std::io::ErrorKind::NotFound.into())
            } else {
                Ok(Cursor::new(bytes.clone()))
            }
        })
        .unwrap();
        output.activate(0).unwrap();
        wait(|| input.stats().decode_failures == 1);
        assert!(output.ready(1).is_err());
        output.activate(250000).unwrap();
        wait(|| input.stats().queued_blocks == 4);
        assert!(output.ready(4800).unwrap());
        assert_eq!(attempts.load(Ordering::SeqCst), 2);
        drop(worker);
    }

    // Stall only the first source, after probing and during MP3 prefix decode.
    // A short read prevents the source buffer from hiding the whole fixture.
    #[cfg(feature = "compressed-proof")]
    #[derive(Default)]
    struct ReadGate {
        held: bool,
        released: bool,
        last_read_end: u64,
    }
    #[cfg(feature = "compressed-proof")]
    struct GatedMp3 {
        bytes: Cursor<&'static [u8]>,
        gate: Option<Arc<(Mutex<ReadGate>, Condvar)>>,
    }
    #[cfg(feature = "compressed-proof")]
    impl Read for GatedMp3 {
        fn read(&mut self, out: &mut [u8]) -> std::io::Result<usize> {
            let count = out.len().min(128);
            if let Some(gate) = &self.gate {
                let mut s = gate.0.lock().unwrap();
                if self.bytes.position() >= 6144 && !s.released {
                    s.held = true;
                    gate.1.notify_all();
                    let (state, timeout) = gate
                        .1
                        .wait_timeout_while(s, Duration::from_secs(3), |s| !s.released)
                        .unwrap();
                    s = state;
                    assert!(!timeout.timed_out(), "test did not release source I/O");
                }
                let n = self.bytes.read(&mut out[..count])?;
                s.last_read_end = self.bytes.position();
                return Ok(n);
            }
            self.bytes.read(&mut out[..count])
        }
    }
    #[cfg(feature = "compressed-proof")]
    impl Seek for GatedMp3 {
        fn seek(&mut self, position: std::io::SeekFrom) -> std::io::Result<u64> {
            self.bytes.seek(position)
        }
    }
    #[cfg(feature = "compressed-proof")]
    fn cancel_prefix(action: u8) {
        let data = include_bytes!("../../p0-media/tests/fixtures/audio-0_37.mp3");
        let probe = Arc::new((
            Mutex::new(ReadGate {
                released: true,
                ..Default::default()
            }),
            Condvar::new(),
        ));
        let opened = Stream::open(GatedMp3 {
            bytes: Cursor::new(data.as_slice()),
            gate: Some(probe.clone()),
        })
        .unwrap();
        assert!(probe.0.lock().unwrap().last_read_end < 6144);
        assert_eq!(opened.stats().decoded_frames, 0);
        drop(opened);
        let (input, output) = buffered::channel(Config {
            rate: 48000,
            channels: 2,
            total_frames: 17760,
            gain: 0.02,
        })
        .unwrap();
        let gate = Arc::new((Mutex::new(ReadGate::default()), Condvar::new()));
        let source_gate = gate.clone();
        let mut first = true;
        let worker = DecoderWorker::spawn(input.clone(), DiagnosticGate::default(), move || {
            let gate = first.then(|| source_gate.clone());
            first = false;
            Ok(GatedMp3 {
                bytes: Cursor::new(data.as_slice()),
                gate,
            })
        })
        .unwrap();
        output.activate(330000).unwrap();
        wait(|| gate.0.lock().unwrap().held);
        assert_eq!(input.stats().accepted_blocks, 0);
        match action {
            0 => {
                // Coalesce several requests while the old source is in I/O.
                output.activate(100000).unwrap();
                output.activate(250000).unwrap();
                output.activate(1000).unwrap();
            }
            1 => output.pause_at(1000, 0).unwrap(),
            2 => input.close(),
            _ => unreachable!(),
        }
        gate.0.lock().unwrap().released = true;
        gate.1.notify_all();
        if action != 2 {
            wait(|| input.stats().queued_blocks == 4);
            let mut actual = [0; 16 * 8];
            let packet = output.prepare(16, &mut actual).unwrap().unwrap();
            assert_eq!(packet.packet.source_frame, 48);
            let mut reference = Stream::open(Cursor::new(data.as_slice())).unwrap();
            reference.seek(48).unwrap();
            let mut expected = [0.; 16 * 2];
            reference.read(&mut expected).unwrap();
            for (a, e) in actual.chunks_exact(4).zip(expected) {
                assert_eq!(f32::from_le_bytes(a.try_into().unwrap()), e * 0.02);
            }
        }
        drop(worker);
        let stats = input.stats();
        assert_eq!(stats.stale_blocks, 1);
        assert_eq!(stats.decode_failures, 0);
        assert_eq!(stats.free_blocks, buffered::POOL_BLOCKS);
        assert!(!stats.in_flight);
        // The obsolete seek must stop within one PCM step instead of reading
        // almost all 10368 compressed bytes to reach its old target.
        assert!(gate.0.lock().unwrap().last_read_end <= 8192);
    }
    #[cfg(feature = "compressed-proof")]
    #[test]
    fn replacement_cancels_mp3_prefix_and_only_latest_request_is_published() {
        cancel_prefix(0);
    }
    #[cfg(feature = "compressed-proof")]
    #[test]
    fn pause_cancels_mp3_prefix_without_faulting_or_publishing_old_pcm() {
        cancel_prefix(1);
    }
    #[cfg(feature = "compressed-proof")]
    #[test]
    fn shutdown_cancels_mp3_prefix_and_returns_all_pool_leases() {
        cancel_prefix(2);
    }
}
