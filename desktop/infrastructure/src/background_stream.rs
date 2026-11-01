//! Read/seek-only PCM16 background, independent of project assets and short WAV
//! preparation. After bounded metadata probing, only the dedicated worker touches
//! the reader. Callbacks see Copy frames in the existing device reserve, never
//! this type or an I/O handle.
use crate::byte_budget::{ByteBudget, ByteLease};
use rtrb::{Consumer, RingBuffer};
use skin_editor_audio::{PcmBudget, PcmReservation};
use std::{
    io::{Read, Seek, SeekFrom},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering::SeqCst},
    },
    thread::{self, JoinHandle},
};
pub const SOURCE_RESERVE: usize = 8192;
const READ_FRAMES: usize = 2048;
const READ_BYTES: usize = READ_FRAMES * 2 * 2;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StreamError {
    InvalidWave,
    Unsupported,
    Io,
    ResourceLimit,
    Closed,
    Protocol,
}
pub trait ReadSeek: Read + Seek + Send {}
impl<T: Read + Seek + Send> ReadSeek for T {}
struct Wave {
    reader: Box<dyn ReadSeek>,
    rate: u32,
    channels: u16,
    frames: u64,
    data: u64,
}
impl Wave {
    fn open(mut reader: Box<dyn ReadSeek>) -> Result<Self, StreamError> {
        let mut h = [0u8; 18];
        let length = reader.seek(SeekFrom::End(0)).map_err(|_| StreamError::Io)?;
        reader
            .seek(SeekFrom::Start(0))
            .map_err(|_| StreamError::Io)?;
        reader
            .read_exact(&mut h[..12])
            .map_err(|_| StreamError::InvalidWave)?;
        if &h[..4] != b"RIFF" || &h[8..12] != b"WAVE" {
            return Err(StreamError::Unsupported);
        }
        if u64::from(u32::from_le_bytes(h[4..8].try_into().unwrap())) + 8 != length {
            return Err(StreamError::InvalidWave);
        }
        let (mut pos, mut format, mut data) = (12u64, None, None);
        let mut chunks = 0;
        while pos < length {
            chunks += 1;
            if chunks > 4096 {
                return Err(StreamError::ResourceLimit);
            }
            reader
                .seek(SeekFrom::Start(pos))
                .map_err(|_| StreamError::Io)?;
            reader
                .read_exact(&mut h[..8])
                .map_err(|_| StreamError::InvalidWave)?;
            let len = u64::from(u32::from_le_bytes(h[4..8].try_into().unwrap()));
            let start = pos.checked_add(8).ok_or(StreamError::InvalidWave)?;
            let next = start
                .checked_add(len)
                .and_then(|v| v.checked_add(len & 1))
                .filter(|v| *v <= length)
                .ok_or(StreamError::InvalidWave)?;
            match &h[..4] {
                b"fmt " => {
                    if format.is_some() {
                        return Err(StreamError::InvalidWave);
                    }
                    if !matches!(len, 16 | 18) {
                        return Err(StreamError::Unsupported);
                    }
                    reader
                        .read_exact(&mut h[..len as usize])
                        .map_err(|_| StreamError::InvalidWave)?;
                    format = Some(crate::wav::pcm16_format(&h[..len as usize]).map_err(|e| {
                        if e == skin_editor_audio::PrepareFailure::Unsupported {
                            StreamError::Unsupported
                        } else {
                            StreamError::InvalidWave
                        }
                    })?);
                }
                b"data" => {
                    if data.is_some() {
                        return Err(StreamError::InvalidWave);
                    }
                    data = Some((start, len));
                }
                _ => {}
            }
            pos = next;
        }
        let (rate, channels) = format.ok_or(StreamError::InvalidWave)?;
        let (data, len) = data.ok_or(StreamError::InvalidWave)?;
        if len == 0 || len % (u64::from(channels) * 2) != 0 {
            return Err(StreamError::InvalidWave);
        }
        Ok(Self {
            reader,
            rate,
            channels,
            frames: len / (u64::from(channels) * 2),
            data,
        })
    }
}
struct State {
    stamp: AtomicU64,
    position: AtomicU64,
    stop: AtomicBool,
    fault: AtomicBool,
    bytes: AtomicU64,
    discarded: AtomicU64,
    seeks: AtomicU64,
}
impl State {
    fn requested(&self) -> Option<(u64, u64)> {
        let a = self.stamp.load(SeqCst);
        if a & 1 != 0 {
            return None;
        }
        let p = self.position.load(SeqCst);
        (self.stamp.load(SeqCst) == a).then_some((a, p))
    }
}
#[derive(Clone)]
pub struct StreamObservation {
    state: Arc<State>,
    raw: Arc<ByteBudget>,
}
#[derive(Clone, Copy, Debug)]
pub struct StreamStats {
    pub read_bytes: u64,
    pub discarded_frames: u64,
    pub seeks: u64,
    pub raw_bytes: usize,
    pub raw_peak_bytes: usize,
    pub fault: bool,
}
impl StreamObservation {
    pub fn stats(&self) -> StreamStats {
        StreamStats {
            read_bytes: self.state.bytes.load(SeqCst),
            discarded_frames: self.state.discarded.load(SeqCst),
            seeks: self.state.seeks.load(SeqCst),
            raw_bytes: self.raw.used(),
            raw_peak_bytes: self.raw.peak(),
            fault: self.state.fault.load(SeqCst),
        }
    }
}
const BLOCKS: usize = SOURCE_RESERVE / READ_FRAMES;
struct Block {
    version: u64,
    first: u64,
    frames: usize,
    pcm: skin_editor_audio::PcmBuffer,
}
pub struct BufferedBackground {
    state: Arc<State>,
    rx: Consumer<Block>,
    free: rtrb::Producer<Block>,
    current: Option<Block>,
    cursor: usize,
    thread: Option<JoinHandle<()>>,
    budget: PcmBudget,
    observation: StreamObservation,
    rate: u32,
    channels: u16,
    frames: u64,
    next: u64,
    version: u64,
}
impl BufferedBackground {
    /// Metadata is bounded/read synchronously before starting a worker. Caller
    /// supplies an explicitly selected read-only handle, never a project path.
    pub fn open(reader: impl ReadSeek + 'static, budget: &PcmBudget) -> Result<Self, StreamError> {
        Self::spawn(Wave::open(Box::new(reader))?, budget)
    }
    fn spawn(wave: Wave, budget: &PcmBudget) -> Result<Self, StreamError> {
        Self::spawn_inner(
            wave,
            budget,
            #[cfg(test)]
            None,
        )
    }
    fn spawn_inner(
        mut wave: Wave,
        budget: &PcmBudget,
        #[cfg(test)] mut wait_gate: Option<tests::WaitGate>,
    ) -> Result<Self, StreamError> {
        let reservations: Vec<PcmReservation> = (0..BLOCKS)
            .map(|_| budget.reserve(READ_FRAMES * usize::from(wave.channels)))
            .collect::<Result<_, _>>()
            .map_err(|_| StreamError::ResourceLimit)?;
        let (mut free, mut available) = RingBuffer::new(BLOCKS);
        let (mut ready, rx) = RingBuffer::new(BLOCKS);
        for lease in reservations {
            assert!(
                free.push(Block {
                    version: 0,
                    first: 0,
                    frames: 0,
                    pcm: lease.allocate()
                })
                .is_ok()
            );
        }
        let raw = ByteBudget::new(READ_BYTES);
        let raw_lease = raw.reserve(READ_BYTES).ok_or(StreamError::ResourceLimit)?;
        let raw_buffer = vec![0u8; READ_BYTES];
        let state = Arc::new(State {
            stamp: AtomicU64::new(2),
            position: AtomicU64::new(0),
            stop: AtomicBool::new(false),
            fault: AtomicBool::new(false),
            bytes: AtomicU64::new(0),
            discarded: AtomicU64::new(0),
            seeks: AtomicU64::new(0),
        });
        let observation = StreamObservation {
            state: state.clone(),
            raw,
        };
        let s = state.clone();
        let (rate, channels, frames) = (wave.rate, wave.channels, wave.frames);
        let thread = thread::Builder::new()
            .name("background-wav-reader".into())
            .spawn(move || {
                let _raw_lease: ByteLease = raw_lease;
                let mut bytes = raw_buffer;
                let (mut active, mut next) = (0, 0);
                let result = (|| -> Result<(), StreamError> {
                    while !s.stop.load(SeqCst) && !ready.is_abandoned() {
                        let Some((version, requested)) = s.requested() else {
                            thread::yield_now();
                            continue;
                        };
                        if active != version {
                            next = requested;
                            active = version;
                            wave.reader
                                .seek(SeekFrom::Start(wave.data + next * u64::from(channels) * 2))
                                .map_err(|_| StreamError::Io)?;
                            s.seeks.fetch_add(1, SeqCst);
                        }
                        if next == frames {
                            park_reader(
                                #[cfg(test)]
                                &mut wait_gate,
                            );
                            continue;
                        }
                        let Ok(mut block) = available.pop() else {
                            park_reader(
                                #[cfg(test)]
                                &mut wait_gate,
                            );
                            continue;
                        };
                        let count = READ_FRAMES.min((frames - next) as usize);
                        let length = count * usize::from(channels) * 2;
                        wave.reader
                            .read_exact(&mut bytes[..length])
                            .map_err(|_| StreamError::Io)?;
                        s.bytes.fetch_add(length as u64, SeqCst);
                        block.version = version;
                        block.first = next;
                        block.frames = 0;
                        if s.stop.load(SeqCst) {
                            break;
                        }
                        if s.stamp.load(SeqCst) == version {
                            for (sample, pair) in
                                block.pcm.samples_mut().map_err(|_| StreamError::Protocol)?
                                    [..count * usize::from(channels)]
                                    .iter_mut()
                                    .zip(bytes[..length].chunks_exact(2))
                            {
                                *sample =
                                    f32::from(i16::from_le_bytes([pair[0], pair[1]])) / 32768.0;
                            }
                            block.frames = count;
                            next += count as u64;
                        } else {
                            s.discarded.fetch_add(count as u64, SeqCst);
                        }
                        // A seek racing this publish is rejected again by the owner.
                        ready.push(block).map_err(|_| StreamError::Protocol)?;
                    }
                    Ok(())
                })();
                if result.is_err() {
                    s.fault.store(true, SeqCst);
                }
            })
            .map_err(|_| StreamError::ResourceLimit)?;
        Ok(Self {
            state,
            rx,
            free,
            current: None,
            cursor: 0,
            thread: Some(thread),
            budget: budget.clone(),
            observation,
            rate,
            channels,
            frames,
            next: 0,
            version: 2,
        })
    }
    pub fn observation(&self) -> StreamObservation {
        self.observation.clone()
    }
    pub fn format(&self) -> (u32, u16) {
        (self.rate, self.channels)
    }
    pub fn frames(&self) -> u64 {
        self.frames
    }
    pub fn position(&self) -> u64 {
        self.next
    }
    pub(crate) fn budget(&self) -> &PcmBudget {
        &self.budget
    }
    fn recycle(&mut self, block: Block) -> Result<(), StreamError> {
        self.free.push(block).map_err(|_| StreamError::Protocol)?;
        if let Some(t) = &self.thread {
            t.thread().unpark();
        }
        Ok(())
    }
    pub fn restore_position(&mut self, frame: u64) -> Result<(), StreamError> {
        if frame > self.frames {
            return Err(StreamError::InvalidWave);
        }
        if self.state.stop.load(SeqCst) {
            return Err(StreamError::Closed);
        }
        let Some(next) = self.version.checked_add(2) else {
            self.state.stop.store(true, SeqCst);
            if let Some(t) = &self.thread {
                t.thread().unpark();
            }
            return Err(StreamError::ResourceLimit);
        };
        if let Some(block) = self.current.take() {
            self.recycle(block)?;
        }
        self.state.stamp.store(next - 1, SeqCst);
        self.state.position.store(frame, SeqCst);
        self.state.stamp.store(next, SeqCst);
        self.version = next;
        self.next = frame;
        self.cursor = 0;
        if let Some(t) = &self.thread {
            t.thread().unpark();
        }
        Ok(())
    }
    pub(crate) fn next_frame(&mut self) -> Result<Option<(u64, [f32; 2])>, StreamError> {
        if self.state.fault.load(SeqCst) {
            return Err(StreamError::Io);
        }
        if self.state.stop.load(SeqCst) {
            return Err(StreamError::Closed);
        }
        for _ in 0..BLOCKS {
            if self.current.is_none() {
                let Ok(block) = self.rx.pop() else {
                    return Ok(None);
                };
                self.current = Some(block);
                self.cursor = 0;
            }
            let block = self.current.as_ref().unwrap();
            if block.version != self.version || block.frames == 0 {
                let block = self.current.take().unwrap();
                self.recycle(block)?;
                continue;
            }
            let frame = block.first + self.cursor as u64;
            if frame != self.next {
                return Err(StreamError::Protocol);
            }
            let i = self.cursor * usize::from(self.channels);
            let values = block.pcm.samples();
            let samples = [values[i], values[i + usize::from(self.channels - 1)]];
            self.cursor += 1;
            self.next += 1;
            if self.cursor == block.frames {
                let block = self.current.take().unwrap();
                self.recycle(block)?;
            }
            return Ok(Some((frame, samples)));
        }
        Ok(None)
    }
}
impl Drop for BufferedBackground {
    fn drop(&mut self) {
        crate::callback_audit::released();
        self.state.stop.store(true, SeqCst);
        if let Some(t) = self.thread.take() {
            t.thread().unpark();
            let _ = t.join();
        }
    }
}
fn park_reader(#[cfg(test)] gate: &mut Option<tests::WaitGate>) {
    #[cfg(test)]
    if let Some(gate) = gate.take() {
        gate.entered.try_send(()).unwrap();
        let until = std::time::Instant::now() + std::time::Duration::from_secs(5);
        // A channel receive may consume this very thread's park token. This
        // test latch must not park between notification and the tested park.
        while !gate.release.load(SeqCst) {
            assert!(std::time::Instant::now() < until, "park gate not released");
            thread::yield_now();
        }
    }
    // unpark is a retained token: recycle/seek/close just before this call
    // cannot be lost. Only the dedicated I/O thread may wait here.
    thread::park();
}
#[cfg(test)]
pub(crate) mod tests;
