use super::*;
use std::io::{self, Cursor};
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};
use std::time::Duration;

pub(super) struct WaitGate {
    pub entered: SyncSender<()>,
    pub release: Arc<AtomicBool>,
}
fn gated(frames: u32, pool: &PcmBudget) -> (BufferedBackground, Receiver<()>, Arc<AtomicBool>) {
    let (entered, seen) = sync_channel(1);
    let release = Arc::new(AtomicBool::new(false));
    let wave = Wave::open(Box::new(Generated::new(frames, 1))).unwrap();
    (
        BufferedBackground::spawn_inner(
            wave,
            pool,
            Some(WaitGate {
                entered,
                release: release.clone(),
            }),
        )
        .unwrap(),
        seen,
        release,
    )
}
#[test]
fn recycled_block_wakes_reader_even_before_it_parks_and_preserves_every_frame() {
    let pool = PcmBudget::default();
    let frames = READ_FRAMES * (BLOCKS + 1);
    let (mut source, entered, release) = gated(frames as u32, &pool);
    entered.recv_timeout(Duration::from_secs(5)).unwrap(); // All free blocks exhausted.
    for i in 0..READ_FRAMES {
        let (at, values) = next(&mut source);
        assert_eq!(at, i as u64);
        assert_eq!(values[0], Generated::sample(at));
    }
    release.store(true, SeqCst); // recycle already sent the unpark token.
    for i in READ_FRAMES..frames {
        assert_eq!(next(&mut source).0, i as u64);
    }
    drop(source);
    assert_eq!(pool.usage().bytes, 0);
}
#[test]
fn eof_seek_before_park_is_not_lost_and_close_releases_the_idle_reader() {
    let pool = PcmBudget::default();
    let (mut source, entered, release) = gated(17, &pool);
    entered.recv_timeout(Duration::from_secs(5)).unwrap(); // EOF, before park.
    source.restore_position(7).unwrap();
    release.store(true, SeqCst);
    assert_eq!(next(&mut source), (7, [Generated::sample(7); 2]));
    let observation = source.observation();
    drop(source);
    assert_eq!(pool.usage().bytes, 0);
    assert_eq!(observation.stats().raw_bytes, 0);
}
#[test]
fn exhausted_seek_version_wakes_and_releases_reader_storage_before_owner_drop() {
    let pool = PcmBudget::default();
    let (mut source, entered, release) = gated(17, &pool);
    entered.recv_timeout(Duration::from_secs(5)).unwrap();
    source.version = u64::MAX - 1;
    assert_eq!(source.restore_position(0), Err(StreamError::ResourceLimit));
    release.store(true, SeqCst);
    let observation = source.observation();
    let until = std::time::Instant::now() + Duration::from_secs(5);
    while observation.stats().raw_bytes != 0 {
        assert!(
            std::time::Instant::now() < until,
            "idle reader failed to stop"
        );
        thread::yield_now();
    }
    assert_eq!(source.next_frame(), Err(StreamError::Closed));
    assert!(pool.usage().bytes > 0); // queued PCM remains charged until its owner drops.
    drop(source);
    assert_eq!(pool.usage().bytes, 0);
}
pub(crate) struct Generated {
    header: [u8; 44],
    pos: u64,
    frames: u64,
    channels: u16,
}
impl Generated {
    pub fn new(frames: u32, channels: u16) -> Self {
        let len = frames * u32::from(channels) * 2;
        let mut h = Vec::new();
        h.extend(b"RIFF");
        h.extend((36 + len).to_le_bytes());
        h.extend(b"WAVEfmt ");
        h.extend(16u32.to_le_bytes());
        h.extend(1u16.to_le_bytes());
        h.extend(channels.to_le_bytes());
        h.extend(48000u32.to_le_bytes());
        h.extend((48000 * u32::from(channels) * 2).to_le_bytes());
        h.extend((channels * 2).to_le_bytes());
        h.extend(16u16.to_le_bytes());
        h.extend(b"data");
        h.extend(len.to_le_bytes());
        Self {
            header: h.try_into().unwrap(),
            pos: 0,
            frames: u64::from(frames),
            channels,
        }
    }
    fn length(&self) -> u64 {
        44 + self.frames * u64::from(self.channels) * 2
    }
    pub fn sample(frame: u64) -> f32 {
        ((frame % 10000) as i16 - 5000) as f32 / 32768.0
    }
}
impl Read for Generated {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        let count = out.len().min((self.length() - self.pos) as usize);
        for (i, v) in out[..count].iter_mut().enumerate() {
            let p = self.pos + i as u64;
            *v = if p < 44 {
                self.header[p as usize]
            } else {
                let q = p - 44;
                let frame = q / (u64::from(self.channels) * 2);
                let s = (frame % 10000) as i16 - 5000;
                s.to_le_bytes()[(q % 2) as usize]
            };
        }
        self.pos += count as u64;
        Ok(count)
    }
}
impl Seek for Generated {
    fn seek(&mut self, from: SeekFrom) -> io::Result<u64> {
        let p = match from {
            SeekFrom::Start(v) => i128::from(v),
            SeekFrom::Current(v) => i128::from(self.pos) + i128::from(v),
            SeekFrom::End(v) => i128::from(self.length()) + i128::from(v),
        };
        if p < 0 || p > i128::from(self.length()) {
            return Err(io::ErrorKind::InvalidInput.into());
        }
        self.pos = p as u64;
        Ok(self.pos)
    }
}
fn next(stream: &mut BufferedBackground) -> (u64, [f32; 2]) {
    let until = std::time::Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(f) = stream.next_frame().unwrap() {
            return f;
        }
        assert!(
            std::time::Instant::now() < until,
            "stream did not produce requested frame"
        );
        thread::yield_now();
    }
}
struct Gate {
    wave: Generated,
    entered: Option<SyncSender<()>>,
    release: Receiver<()>,
}
impl Read for Gate {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        if self.wave.pos >= 44
            && let Some(tx) = self.entered.take()
        {
            tx.send(()).unwrap();
            self.release.recv().unwrap();
        }
        self.wave.read(out)
    }
}
impl Seek for Gate {
    fn seek(&mut self, p: SeekFrom) -> io::Result<u64> {
        self.wave.seek(p)
    }
}
#[test]
fn ten_minute_source_consumes_across_ring_wraps_with_constant_pcm_and_raw_storage() {
    let budget = PcmBudget::new(SOURCE_RESERVE * 8, SOURCE_RESERVE * 8).unwrap();
    let mut stream = BufferedBackground::open(Generated::new(48000 * 600, 2), &budget).unwrap();
    let observer = stream.observation();
    assert_eq!(stream.frames(), 48000 * 600);
    assert_eq!(budget.usage().bytes, SOURCE_RESERVE * 8);
    for frame in 0..50000 {
        assert_eq!(next(&mut stream), (frame, [Generated::sample(frame); 2]));
    }
    stream.restore_position(48000 * 500 + 7).unwrap();
    assert_eq!(
        next(&mut stream),
        (48000 * 500 + 7, [Generated::sample(48000 * 500 + 7); 2])
    );
    assert_eq!(budget.usage().peak_bytes, SOURCE_RESERVE * 8);
    assert_eq!(observer.stats().raw_peak_bytes, READ_BYTES);
    drop(stream);
    assert_eq!(budget.usage().bytes, 0);
    assert_eq!(observer.stats().raw_bytes, 0);
    assert!(!observer.stats().fault);
}
#[test]
fn late_read_is_discarded_after_seek_without_publishing_old_position() {
    let pool = PcmBudget::default();
    let (tx, entered) = sync_channel(0);
    let (release, rx) = sync_channel(0);
    let mut stream = BufferedBackground::open(
        Gate {
            wave: Generated::new(48000, 1),
            entered: Some(tx),
            release: rx,
        },
        &pool,
    )
    .unwrap();
    entered.recv_timeout(Duration::from_secs(5)).unwrap();
    stream.restore_position(12345).unwrap();
    release.send(()).unwrap();
    assert_eq!(next(&mut stream), (12345, [Generated::sample(12345); 2]));
    assert!(stream.observation().stats().discarded_frames > 0);
    drop(stream);
    assert_eq!(pool.usage().bytes, 0);
}
#[test]
fn cancellation_waits_for_actual_reader_exit_before_returning_resource_credits() {
    let pool = PcmBudget::default();
    let (tx, entered) = sync_channel(0);
    let (release, rx) = sync_channel(0);
    let stream = BufferedBackground::open(
        Gate {
            wave: Generated::new(48000, 2),
            entered: Some(tx),
            release: rx,
        },
        &pool,
    )
    .unwrap();
    let observer = stream.observation();
    let state = stream.state.clone();
    entered.recv_timeout(Duration::from_secs(5)).unwrap();
    let closer = thread::spawn(move || drop(stream));
    let until = std::time::Instant::now() + Duration::from_secs(5);
    while !state.stop.load(SeqCst) {
        assert!(std::time::Instant::now() < until);
        thread::yield_now();
    }
    assert_eq!(pool.usage().bytes, SOURCE_RESERVE * 8);
    assert_eq!(observer.stats().raw_bytes, READ_BYTES);
    release.send(()).unwrap();
    closer.join().unwrap();
    assert_eq!(pool.usage().bytes, 0);
    assert_eq!(observer.stats().raw_bytes, 0);
}
#[test]
fn eof_and_invalid_seek_do_not_loop_or_reset_the_source() {
    let pool = PcmBudget::default();
    let mut stream = BufferedBackground::open(Generated::new(37, 1), &pool).unwrap();
    for frame in 0..37 {
        assert_eq!(next(&mut stream).0, frame);
    }
    assert_eq!(stream.next_frame().unwrap(), None);
    let version = stream.version;
    assert_eq!(stream.restore_position(38), Err(StreamError::InvalidWave));
    assert_eq!(stream.version, version);
    stream.restore_position(37).unwrap();
    assert_eq!(stream.position(), 37);
    assert_eq!(stream.next_frame().unwrap(), None);
    drop(stream);
    assert_eq!(pool.usage().bytes, 0);
}
#[test]
fn over_budget_open_does_not_start_a_reader_or_retain_partial_reservations() {
    let pool = PcmBudget::new(SOURCE_RESERVE * 8 - 1, SOURCE_RESERVE * 8 - 1).unwrap();
    assert!(matches!(
        BufferedBackground::open(Generated::new(48000, 2), &pool),
        Err(StreamError::ResourceLimit)
    ));
    assert_eq!(pool.usage().bytes, 0);
}
#[test]
fn malformed_and_unsupported_wave_headers_are_rejected_without_worker_storage() {
    for kind in 0..5 {
        let mut wave = Generated::new(4, 1);
        let mut bytes = vec![0; 52];
        wave.read_exact(&mut bytes).unwrap();
        match kind {
            0 => bytes[0] = 0,
            1 => bytes[4] = 0,
            2 => bytes[20] = 3,
            3 => bytes[22] = 3,
            _ => bytes[40] = 9,
        };
        let pool = PcmBudget::default();
        assert!(BufferedBackground::open(Cursor::new(bytes), &pool).is_err());
        assert_eq!(pool.usage().bytes, 0);
    }
}
#[test]
fn exhausted_source_version_cannot_wrap_to_an_old_request() {
    let pool = PcmBudget::default();
    let mut stream = BufferedBackground::open(Generated::new(100, 1), &pool).unwrap();
    stream.version = u64::MAX - 1;
    assert_eq!(stream.restore_position(0), Err(StreamError::ResourceLimit));
    drop(stream);
    assert_eq!(pool.usage().bytes, 0);
}
#[test]
fn both_minimum_rates_and_channel_counts_produce_exact_pcm_without_full_file_storage() {
    for rate in [44100u32, 48000] {
        for channels in [1u16, 2] {
            let mut wave = Generated::new(33, channels);
            wave.header[24..28].copy_from_slice(&rate.to_le_bytes());
            wave.header[28..32].copy_from_slice(&(rate * u32::from(channels) * 2).to_le_bytes());
            let pool = PcmBudget::default();
            let mut source = BufferedBackground::open(wave, &pool).unwrap();
            assert_eq!(source.format(), (rate, channels));
            for i in 0..33 {
                assert_eq!(next(&mut source), (i, [Generated::sample(i); 2]));
            }
            drop(source);
            assert_eq!(pool.usage().bytes, 0);
        }
    }
}
#[test]
fn repeated_locations_before_read_completion_only_publish_the_latest_position() {
    let pool = PcmBudget::default();
    let (tx, entered) = sync_channel(0);
    let (release, rx) = sync_channel(0);
    let mut stream = BufferedBackground::open(
        Gate {
            wave: Generated::new(48000, 2),
            entered: Some(tx),
            release: rx,
        },
        &pool,
    )
    .unwrap();
    entered.recv_timeout(Duration::from_secs(5)).unwrap();
    for position in [42000, 200, 900, 37] {
        stream.restore_position(position).unwrap();
    }
    release.send(()).unwrap();
    assert_eq!(next(&mut stream), (37, [Generated::sample(37); 2]));
    drop(stream);
    assert_eq!(pool.usage().bytes, 0);
}
struct Broken(Generated);
impl Read for Broken {
    fn read(&mut self, b: &mut [u8]) -> io::Result<usize> {
        if self.0.pos >= 44 {
            return Err(io::ErrorKind::UnexpectedEof.into());
        }
        self.0.read(b)
    }
}
impl Seek for Broken {
    fn seek(&mut self, p: SeekFrom) -> io::Result<u64> {
        self.0.seek(p)
    }
}
#[test]
fn reader_failure_is_reported_instead_of_inventing_eof_or_silence_and_close_releases_all_storage() {
    let pool = PcmBudget::default();
    let mut source = BufferedBackground::open(Broken(Generated::new(100, 1)), &pool).unwrap();
    let observer = source.observation();
    let until = std::time::Instant::now() + Duration::from_secs(5);
    while !observer.stats().fault {
        assert!(std::time::Instant::now() < until);
        thread::yield_now();
    }
    assert_eq!(source.next_frame(), Err(StreamError::Io));
    drop(source);
    assert_eq!(pool.usage().bytes, 0);
    assert_eq!(observer.stats().raw_bytes, 0);
}
