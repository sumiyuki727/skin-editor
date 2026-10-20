//! Bounded value-only PCM handoff. No decoder, files, UI or native pointers.
use crate::{Error, pcm};
use std::{
    collections::VecDeque,
    sync::{Arc, Condvar, Mutex},
    time::Duration,
};

pub const BLOCK_FRAMES: usize = 4096;
pub const QUEUE_BLOCKS: usize = 4;
pub const POOL_BLOCKS: usize = QUEUE_BLOCKS + 1;
pub const TRACE_LIMIT: usize = 8192;

#[derive(Clone, Copy, Debug)]
pub struct Config {
    pub rate: u32,
    pub channels: usize,
    pub total_frames: u64,
    pub gain: f32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Activate,
    Pause,
    Retire,
    Submit,
    Stale,
    Underrun,
}
#[derive(Clone, Copy, Debug)]
pub struct Event {
    pub kind: Kind,
    pub generation: u64,
    pub first_frame: u64,
    pub frames: u64,
    pub native_discarded_frames: u64,
    pub packet: Option<pcm::Packet>,
}
#[derive(Clone, Copy, Debug, Default)]
pub struct Stats {
    pub generation: u64,
    pub active: bool,
    pub queued_blocks: usize,
    pub queued_frames: u64,
    pub in_flight: bool,
    pub free_blocks: usize,
    pub max_queued_blocks: usize,
    pub accepted_blocks: u64,
    pub stale_blocks: u64,
    pub discarded_blocks: u64,
    pub discarded_frames: u64,
    pub native_discarded_frames: u64,
    pub underruns: u64,
    pub decode_failures: u64,
    pub trace_overflow: bool,
    pub pcm: pcm::Stats,
}
pub struct Work {
    generation: u64,
    first_frame: u64,
    samples: Box<[f32]>,
}
impl Work {
    pub fn generation(&self) -> u64 {
        self.generation
    }
    pub fn first_frame(&self) -> u64 {
        self.first_frame
    }
    pub fn samples_mut(&mut self) -> &mut [f32] {
        &mut self.samples
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Completion {
    Accepted,
    Stale,
}
#[derive(Clone, Copy, Debug)]
pub struct Prepared {
    pub packet: pcm::Packet,
    generation: u64,
}
struct Queued {
    first: u64,
    frames: usize,
    offset: usize,
    samples: Box<[f32]>,
}
struct State {
    stats: Stats,
    closed: bool,
    failed: bool,
    eof: bool,
    next_frame: u64,
    queue: VecDeque<Queued>,
    pool: Vec<Box<[f32]>>,
    trace: Vec<Event>,
}
struct Shared {
    config: Config,
    state: Mutex<State>,
    changed: Condvar,
}
#[derive(Clone)]
pub struct Input(Arc<Shared>);
/// Single consumer capability; the producer cannot activate or consume a queue.
pub struct Receiver(pub(crate) Input);
pub fn channel(config: Config) -> Result<(Input, Receiver), Error> {
    let input = Input::new(config)?;
    Ok((input.clone(), Receiver(input)))
}
impl Receiver {
    pub fn config(&self) -> Config {
        self.0.config()
    }
    pub fn duration_us(&self) -> u64 {
        self.0.duration_us()
    }
    pub fn activate(&self, position_us: u64) -> Result<(), Error> {
        self.0.restart(position_us, Kind::Activate, 0)
    }
    pub fn pause_at(&self, position_us: u64, native_discarded: u64) -> Result<(), Error> {
        self.0.restart(position_us, Kind::Pause, native_discarded)
    }
    pub fn ready(&self, frames: u32) -> Result<bool, Error> {
        self.0.ready(frames)
    }
    pub fn prepare(&self, frames: u32, output: &mut [u8]) -> Result<Option<Prepared>, Error> {
        self.0.prepare(frames, output)
    }
    pub fn commit(&self, packet: Prepared) -> Result<(), Error> {
        self.0.commit(packet)
    }
}
fn invalid(operation: &'static str) -> Error {
    Error {
        operation,
        code: None,
    }
}
impl Input {
    fn new(config: Config) -> Result<Self, Error> {
        if !(8000..=192000).contains(&config.rate)
            || !(1..=2).contains(&config.channels)
            || config.total_frames == 0
            || config.total_frames > u64::from(config.rate) * 600
            || !config.gain.is_finite()
            || !(0.0..=pcm::AMPLITUDE).contains(&config.gain)
        {
            return Err(invalid("Invalid buffered PCM configuration"));
        }
        let pool = (0..POOL_BLOCKS)
            .map(|_| vec![0.; BLOCK_FRAMES * config.channels].into_boxed_slice())
            .collect();
        Ok(Self(Arc::new(Shared {
            config,
            state: Mutex::new(State {
                stats: Stats::default(),
                closed: false,
                failed: false,
                eof: false,
                next_frame: 0,
                queue: VecDeque::with_capacity(QUEUE_BLOCKS),
                pool,
                trace: Vec::with_capacity(TRACE_LIMIT),
            }),
            changed: Condvar::new(),
        })))
    }
    pub fn config(&self) -> Config {
        self.0.config
    }
    pub fn duration_us(&self) -> u64 {
        self.0.config.total_frames * 1_000_000 / u64::from(self.0.config.rate)
    }
    pub fn stats(&self) -> Stats {
        let s = self.0.state.lock().unwrap_or_else(|e| e.into_inner());
        Stats {
            queued_blocks: s.queue.len(),
            queued_frames: s.queue.iter().map(|b| (b.frames - b.offset) as u64).sum(),
            free_blocks: s.pool.len(),
            ..s.stats
        }
    }
    pub fn trace(&self) -> Vec<Event> {
        self.0
            .state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .trace
            .clone()
    }
    /// Move out recorded diagnostics, retaining the bounded producer allocation.
    /// The caller reuses its own buffer. Overflow remains sticky after a drain.
    pub fn drain_trace(&self, output: &mut Vec<Event>) {
        let mut s = self.0.state.lock().unwrap_or_else(|e| e.into_inner());
        output.clear();
        output.append(&mut s.trace);
    }
    pub fn is_closed(&self) -> bool {
        self.0
            .state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .closed
    }
    /// Read-only cancellation check; a later generation can still race with a
    /// producer, so complete/fail must retain their authoritative stale checks.
    pub fn is_current(&self, generation: u64) -> bool {
        let s = self.0.state.lock().unwrap_or_else(|e| e.into_inner());
        !s.closed && s.stats.active && s.stats.generation == generation
    }
    pub fn wake(&self) {
        self.0.changed.notify_all();
    }
    pub fn close(&self) {
        let mut s = self.0.state.lock().unwrap_or_else(|e| e.into_inner());
        s.closed = true;
        Self::discard(&mut s);
        self.0.changed.notify_all();
    }
    /// One producer lease, at most four queued blocks. All I/O occurs after return.
    pub fn work(&self, timeout: Duration) -> Option<Work> {
        let mut s = self.0.state.lock().unwrap_or_else(|e| e.into_inner());
        let blocked = |s: &State| {
            !s.closed
                && (!s.stats.active
                    || s.failed
                    || s.eof
                    || s.stats.in_flight
                    || s.queue.len() == QUEUE_BLOCKS)
        };
        if blocked(&s) {
            s = self
                .0
                .changed
                .wait_timeout_while(s, timeout, |state| blocked(state))
                .unwrap_or_else(|e| e.into_inner())
                .0;
        }
        if s.closed || blocked(&s) {
            return None;
        }
        let samples = s.pool.pop()?;
        s.stats.in_flight = true;
        Some(Work {
            generation: s.stats.generation,
            first_frame: s.next_frame,
            samples,
        })
    }
    pub fn complete(&self, work: Work, frames: usize, eof: bool) -> Result<Completion, Error> {
        let c = self.config();
        let valid = frames > 0
            && frames <= BLOCK_FRAMES
            && frames as u64
                == (c.total_frames.saturating_sub(work.first_frame)).min(BLOCK_FRAMES as u64)
            && eof == (work.first_frame + frames as u64 == c.total_frames)
            && work.samples[..frames * c.channels]
                .iter()
                .all(|v| v.is_finite() && v.abs() <= 1.);
        let mut s = self.0.state.lock().unwrap_or_else(|e| e.into_inner());
        s.stats.in_flight = false;
        if s.closed || !s.stats.active || work.generation != s.stats.generation {
            s.stats.stale_blocks += 1;
            Self::record(
                &mut s,
                Event {
                    kind: Kind::Stale,
                    generation: work.generation,
                    first_frame: work.first_frame,
                    frames: frames as u64,
                    native_discarded_frames: 0,
                    packet: None,
                },
            );
            s.pool.push(work.samples);
            self.wake();
            return Ok(Completion::Stale);
        }
        if !valid || work.first_frame != s.next_frame || s.queue.len() == QUEUE_BLOCKS {
            s.failed = true;
            s.stats.decode_failures += 1;
            s.pool.push(work.samples);
            self.wake();
            return Err(invalid("Invalid buffered PCM completion"));
        }
        s.next_frame += frames as u64;
        s.eof = eof;
        s.queue.push_back(Queued {
            first: work.first_frame,
            frames,
            offset: 0,
            samples: work.samples,
        });
        s.stats.accepted_blocks += 1;
        s.stats.max_queued_blocks = s.stats.max_queued_blocks.max(s.queue.len());
        self.wake();
        Ok(Completion::Accepted)
    }
    pub fn fail(&self, work: Option<Work>) {
        let mut s = self.0.state.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(w) = work {
            s.stats.in_flight = false;
            if w.generation != s.stats.generation || s.closed || !s.stats.active {
                s.stats.stale_blocks += 1;
                Self::record(
                    &mut s,
                    Event {
                        kind: Kind::Stale,
                        generation: w.generation,
                        first_frame: w.first_frame,
                        frames: 0,
                        native_discarded_frames: 0,
                        packet: None,
                    },
                );
                s.pool.push(w.samples);
                self.wake();
                return;
            }
            s.pool.push(w.samples);
        }
        s.failed = true;
        s.stats.decode_failures += 1;
        self.wake();
    }
    fn record(s: &mut State, event: Event) {
        if s.trace.len() < TRACE_LIMIT {
            s.trace.push(event);
        } else {
            s.stats.trace_overflow = true;
        }
    }
    fn discard(s: &mut State) {
        while let Some(b) = s.queue.pop_front() {
            s.stats.discarded_blocks += 1;
            s.stats.discarded_frames += (b.frames - b.offset) as u64;
            s.pool.push(b.samples);
        }
    }
    pub(crate) fn restart(
        &self,
        position_us: u64,
        kind: Kind,
        native_discarded: u64,
    ) -> Result<(), Error> {
        if position_us > self.duration_us() {
            return Err(invalid("Buffered PCM origin outside content"));
        }
        let mut s = self.0.state.lock().unwrap_or_else(|e| e.into_inner());
        if s.closed {
            return Err(invalid("Buffered PCM input closed"));
        }
        let generation = s
            .stats
            .generation
            .checked_add(1)
            .ok_or_else(|| invalid("Buffered PCM generation overflow"))?;
        let frame = position_us * u64::from(self.config().rate) / 1_000_000;
        let discarded = s.queue.iter().map(|b| (b.frames - b.offset) as u64).sum();
        Self::discard(&mut s);
        s.failed = false;
        s.eof = frame == self.config().total_frames;
        s.next_frame = frame;
        s.stats.generation = generation;
        s.stats.active = true;
        s.stats.native_discarded_frames += native_discarded;
        s.stats.pcm = pcm::Stats {
            requested_us: position_us,
            source_origin_frame: frame,
            ..Default::default()
        };
        Self::record(
            &mut s,
            Event {
                kind,
                generation,
                first_frame: frame,
                frames: discarded,
                native_discarded_frames: native_discarded,
                packet: None,
            },
        );
        self.wake();
        Ok(())
    }
    pub(crate) fn retire(&self) {
        let mut s = self.0.state.lock().unwrap_or_else(|e| e.into_inner());
        Self::discard(&mut s);
        s.stats.active = false;
        s.stats.generation = s.stats.generation.saturating_add(1);
        let generation = s.stats.generation;
        let first_frame = s.next_frame;
        Self::record(
            &mut s,
            Event {
                kind: Kind::Retire,
                generation,
                first_frame,
                frames: 0,
                native_discarded_frames: 0,
                packet: None,
            },
        );
        self.wake();
    }
    pub(crate) fn ready(&self, frames: u32) -> Result<bool, Error> {
        let s = self.0.state.lock().unwrap_or_else(|e| e.into_inner());
        if s.failed || s.closed || !s.stats.active {
            return Err(invalid("Buffered PCM source unavailable"));
        }
        let position = s.stats.pcm.source_origin_frame + s.stats.pcm.submitted_frames;
        let needed = u64::from(frames).min(self.config().total_frames.saturating_sub(position));
        Ok(s.queue
            .iter()
            .map(|b| (b.frames - b.offset) as u64)
            .sum::<u64>()
            >= needed)
    }
    pub(crate) fn underrun(&self) -> Error {
        let mut s = self.0.state.lock().unwrap_or_else(|e| e.into_inner());
        s.stats.underruns += 1;
        let generation = s.stats.generation;
        let first_frame = s.stats.pcm.source_origin_frame + s.stats.pcm.submitted_frames;
        Self::record(
            &mut s,
            Event {
                kind: Kind::Underrun,
                generation,
                first_frame,
                frames: 0,
                native_discarded_frames: 0,
                packet: None,
            },
        );
        invalid("Streamed PCM queue underrun")
    }
    /// Stage without consuming. The native backend commits only after ReleaseBuffer.
    pub(crate) fn prepare(&self, frames: u32, out: &mut [u8]) -> Result<Option<Prepared>, Error> {
        let c = self.config();
        if frames == 0 || out.len() != frames as usize * c.channels * 4 || out.len() > 1024 * 1024 {
            return Err(invalid("Invalid buffered native packet"));
        }
        let s = self.0.state.lock().unwrap_or_else(|e| e.into_inner());
        if s.failed || s.closed || !s.stats.active {
            return Err(invalid("Buffered PCM source unavailable"));
        }
        let start = s.stats.pcm.source_origin_frame + s.stats.pcm.submitted_frames;
        let needed = u64::from(frames).min(c.total_frames.saturating_sub(start)) as usize;
        if s.queue.iter().map(|b| b.frames - b.offset).sum::<usize>() < needed {
            return Ok(None);
        }
        out.fill(0);
        let mut written = 0usize;
        for b in &s.queue {
            if b.first + b.offset as u64 != start + written as u64 {
                return Err(invalid("Discontinuous buffered PCM"));
            }
            let n = (b.frames - b.offset).min(needed - written);
            for (sample, bytes) in b.samples[b.offset * c.channels..(b.offset + n) * c.channels]
                .iter()
                .zip(
                    out[written * c.channels * 4..(written + n) * c.channels * 4]
                        .chunks_exact_mut(4),
                )
            {
                bytes.copy_from_slice(&(*sample * c.gain).to_le_bytes());
            }
            written += n;
            if written == needed {
                break;
            }
        }
        let mut packet = pcm::Packet {
            source_frame: start,
            frames,
            hash: 0xcbf29ce484222325,
            ..Default::default()
        };
        for (n, bytes) in out.chunks_exact(4).enumerate() {
            let v = f32::from_le_bytes(bytes.try_into().unwrap());
            packet.peak = packet.peak.max(v.abs());
            packet.nonzero_samples += u64::from(v != 0.);
            if n.is_multiple_of(c.channels) && n / c.channels < 8 {
                packet.probe[n / c.channels] = v;
            }
        }
        for b in out {
            packet.hash = (packet.hash ^ u64::from(*b)).wrapping_mul(0x100000001b3);
        }
        Ok(Some(Prepared {
            packet,
            generation: s.stats.generation,
        }))
    }
    pub(crate) fn commit(&self, prepared: Prepared) -> Result<(), Error> {
        let mut s = self.0.state.lock().unwrap_or_else(|e| e.into_inner());
        let packet = prepared.packet;
        let c = self.config();
        let start = s.stats.pcm.source_origin_frame + s.stats.pcm.submitted_frames;
        if prepared.generation != s.stats.generation
            || !s.stats.active
            || s.closed
            || packet.source_frame != start
            || packet.frames == 0
        {
            return Err(invalid("Stale buffered native commit"));
        }
        let mut remaining =
            u64::from(packet.frames).min(c.total_frames.saturating_sub(start)) as usize;
        if s.queue.iter().map(|b| b.frames - b.offset).sum::<usize>() < remaining {
            return Err(invalid("Missing buffered commit data"));
        }
        while remaining != 0 {
            let b = s.queue.front_mut().unwrap();
            let n = remaining.min(b.frames - b.offset);
            b.offset += n;
            remaining -= n;
            if b.offset == b.frames {
                let b = s.queue.pop_front().unwrap();
                s.pool.push(b.samples);
            }
        }
        let p = &mut s.stats.pcm;
        p.submitted_frames += u64::from(packet.frames);
        p.packets += 1;
        p.nonzero_samples += packet.nonzero_samples;
        p.first.get_or_insert(packet);
        p.last = Some(packet);
        let generation = s.stats.generation;
        Self::record(
            &mut s,
            Event {
                kind: Kind::Submit,
                generation,
                first_frame: start,
                frames: u64::from(packet.frames),
                native_discarded_frames: 0,
                packet: Some(packet),
            },
        );
        self.wake();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn input(frames: u64) -> Input {
        Input::new(Config {
            rate: 48000,
            channels: 2,
            total_frames: frames,
            gain: 0.02,
        })
        .unwrap()
    }
    fn fill(i: &Input) {
        let mut w = i.work(Duration::ZERO).unwrap();
        let count = (i.config().total_frames - w.first_frame()).min(BLOCK_FRAMES as u64) as usize;
        for (n, v) in w.samples_mut().iter_mut().enumerate() {
            *v = if n % 2 == 0 { 0.5 } else { -0.25 };
        }
        let eof = w.first_frame() + count as u64 == i.config().total_frames;
        i.complete(w, count, eof).unwrap();
    }
    #[test]
    fn pool_queue_and_inflight_are_bounded_and_backpressure_releases_on_commit() {
        let i = input(48000);
        i.restart(0, Kind::Activate, 0).unwrap();
        for _ in 0..QUEUE_BLOCKS {
            fill(&i);
        }
        assert!(i.work(Duration::ZERO).is_none());
        assert_eq!((i.stats().queued_blocks, i.stats().free_blocks), (4, 1));
        let mut bytes = vec![0; 4096 * 8];
        let p = i.prepare(4096, &mut bytes).unwrap().unwrap();
        i.commit(p).unwrap();
        let w = i.work(Duration::ZERO).unwrap();
        assert!(i.work(Duration::ZERO).is_none());
        assert_eq!(
            i.stats().free_blocks + i.stats().queued_blocks + usize::from(i.stats().in_flight),
            POOL_BLOCKS
        );
        i.complete(w, 4096, false).unwrap();
    }
    #[test]
    fn current_generation_check_rejects_replacement_retirement_and_close() {
        let i = input(48000);
        assert!(!i.is_current(0));
        i.restart(0, Kind::Activate, 0).unwrap();
        let w = i.work(Duration::ZERO).unwrap();
        assert!(i.is_current(w.generation()));
        i.restart(500, Kind::Pause, 0).unwrap();
        assert!(!i.is_current(w.generation()));
        i.fail(Some(w));
        assert_eq!(i.stats().decode_failures, 0);
        assert!(i.is_current(2));
        i.retire();
        assert!(!i.is_current(i.stats().generation));
        i.restart(0, Kind::Activate, 0).unwrap();
        assert!(i.is_current(i.stats().generation));
        i.close();
        assert!(!i.is_current(i.stats().generation));
        assert_eq!(i.stats().free_blocks, POOL_BLOCKS);
    }
    #[test]
    fn draining_long_trace_preserves_every_packet_and_pool_bounds() {
        let i = input(48000 * 600);
        i.restart(0, Kind::Activate, 0).unwrap();
        let mut events = Vec::with_capacity(TRACE_LIMIT);
        let mut bytes = vec![0; 8];
        let mut cursor = 0;
        let mut activates = 0;
        for n in 0..TRACE_LIMIT * 2 {
            if i.stats().queued_frames == 0 {
                fill(&i);
            }
            let p = i.prepare(1, &mut bytes).unwrap().unwrap();
            i.commit(p).unwrap();
            if n % 100 == 0 || n + 1 == TRACE_LIMIT * 2 {
                i.drain_trace(&mut events);
                for event in &events {
                    if event.kind == Kind::Submit {
                        assert_eq!(event.first_frame, cursor);
                        cursor += event.frames;
                    } else {
                        assert_eq!(event.kind, Kind::Activate);
                        activates += 1;
                    }
                }
            }
        }
        assert_eq!((cursor, activates), ((TRACE_LIMIT * 2) as u64, 1));
        i.drain_trace(&mut events);
        assert!(events.is_empty());
        assert!(!i.stats().trace_overflow);
        assert_eq!(i.stats().free_blocks + i.stats().queued_blocks, POOL_BLOCKS);
    }
    #[test]
    fn draining_cannot_hide_a_preexisting_trace_overflow() {
        let i = input(48000);
        for _ in 0..=TRACE_LIMIT {
            i.restart(0, Kind::Pause, 0).unwrap();
        }
        let mut events = Vec::new();
        i.drain_trace(&mut events);
        assert_eq!(events.len(), TRACE_LIMIT);
        assert!(i.stats().trace_overflow);
    }
    #[test]
    fn prepare_is_retryable_and_only_successful_native_commit_consumes_frames() {
        let i = input(48000);
        i.restart(0, Kind::Activate, 0).unwrap();
        fill(&i);
        fill(&i);
        let mut bytes = vec![0; 4800 * 8];
        let p = i.prepare(4800, &mut bytes).unwrap().unwrap();
        let original = bytes.clone();
        assert_eq!(i.stats().pcm.submitted_frames, 0);
        assert_eq!(
            i.prepare(4800, &mut bytes).unwrap().unwrap().packet.hash,
            p.packet.hash
        );
        assert_eq!(bytes, original);
        assert_eq!(f32::from_le_bytes(bytes[..4].try_into().unwrap()), 0.01);
        assert_eq!(f32::from_le_bytes(bytes[4..8].try_into().unwrap()), -0.005);
        i.commit(p).unwrap();
        assert_eq!(i.stats().pcm.submitted_frames, 4800);
        assert!(i.commit(p).is_err());
        assert_eq!(i.stats().queued_frames, 8192 - 4800);
    }
    #[test]
    fn pause_and_seek_drop_queued_and_inflight_old_generations_and_restart_at_exact_origin() {
        let i = input(48000);
        i.restart(0, Kind::Activate, 0).unwrap();
        fill(&i);
        let old = i.work(Duration::ZERO).unwrap();
        i.restart(100001, Kind::Pause, 100).unwrap();
        assert_eq!(i.stats().queued_frames, 0);
        assert_eq!(i.complete(old, 4096, false).unwrap(), Completion::Stale);
        let w = i.work(Duration::ZERO).unwrap();
        assert_eq!((w.generation(), w.first_frame()), (2, 4800));
        i.restart(7, Kind::Activate, 0).unwrap();
        assert_eq!(i.complete(w, 4096, false).unwrap(), Completion::Stale);
        let w = i.work(Duration::ZERO).unwrap();
        assert_eq!(w.first_frame(), 0);
        assert_eq!(i.stats().stale_blocks, 2);
        assert_eq!(i.stats().native_discarded_frames, 100);
        i.complete(w, 4096, false).unwrap();
        assert_eq!(i.stats().pcm.submitted_frames, 0);
    }
    #[test]
    fn eof_padding_is_distinct_from_starvation_and_does_not_repeat_source() {
        let i = input(7);
        i.restart(0, Kind::Activate, 0).unwrap();
        assert!(!i.ready(4800).unwrap());
        fill(&i);
        assert!(i.ready(4800).unwrap());
        let mut bytes = vec![9; 16 * 8];
        let p = i.prepare(16, &mut bytes).unwrap().unwrap();
        assert!(bytes[7 * 8..].iter().all(|b| *b == 0));
        i.commit(p).unwrap();
        let p = i.prepare(16, &mut bytes).unwrap().unwrap();
        assert_eq!(p.packet.source_frame, 16);
        assert!(bytes.iter().all(|b| *b == 0));
        assert!(i.work(Duration::ZERO).is_none());
    }
    #[test]
    fn corrupt_completions_fail_closed_and_explicit_restart_is_required() {
        let i = input(48000);
        i.restart(0, Kind::Activate, 0).unwrap();
        let mut w = i.work(Duration::ZERO).unwrap();
        w.samples_mut()[0] = f32::NAN;
        assert!(i.complete(w, 4096, false).is_err());
        assert!(i.ready(1).is_err());
        assert_eq!(i.stats().decode_failures, 1);
        i.restart(0, Kind::Activate, 0).unwrap();
        fill(&i);
        assert!(i.ready(4096).unwrap());
        assert_eq!(i.stats().underruns, 0);
        assert_eq!(i.underrun().operation, "Streamed PCM queue underrun");
        assert_eq!(i.stats().underruns, 1);
    }
    #[test]
    fn invalid_config_origin_and_packet_do_not_modify_valid_queue() {
        assert!(
            Input::new(Config {
                rate: 48000,
                channels: 3,
                total_frames: 1,
                gain: 0.02
            })
            .is_err()
        );
        assert!(
            Input::new(Config {
                rate: 48000,
                channels: 2,
                total_frames: 1,
                gain: 1.
            })
            .is_err()
        );
        let i = input(48000);
        i.restart(0, Kind::Activate, 0).unwrap();
        fill(&i);
        assert!(i.restart(1000001, Kind::Pause, 0).is_err());
        assert!(i.prepare(1, &mut [0; 1]).is_err());
        assert_eq!(i.stats().generation, 1);
        assert_eq!(i.stats().queued_frames, 4096);
    }
    #[test]
    fn retirement_and_close_reject_late_completions_and_recycle_every_buffer() {
        let i = input(48000);
        i.restart(0, Kind::Activate, 0).unwrap();
        fill(&i);
        let w = i.work(Duration::ZERO).unwrap();
        i.retire();
        assert_eq!(i.complete(w, 4096, false).unwrap(), Completion::Stale);
        assert!(i.work(Duration::ZERO).is_none());
        i.restart(0, Kind::Activate, 0).unwrap();
        let w = i.work(Duration::ZERO).unwrap();
        i.close();
        i.complete(w, 4096, false).unwrap();
        assert_eq!(i.stats().free_blocks, POOL_BLOCKS);
        assert!(i.is_closed());
        assert!(i.restart(0, Kind::Activate, 0).is_err());
    }
    #[test]
    fn same_origin_new_generation_rejects_old_prepared_packet_and_old_decode_error() {
        let i = input(48000);
        i.restart(0, Kind::Activate, 0).unwrap();
        fill(&i);
        let prepared = i.prepare(32, &mut [0; 256]).unwrap().unwrap();
        let old = i.work(Duration::ZERO).unwrap();
        i.restart(0, Kind::Pause, 0).unwrap();
        i.fail(Some(old));
        assert_eq!(i.stats().decode_failures, 0);
        assert_eq!(i.stats().stale_blocks, 1);
        fill(&i);
        assert!(i.commit(prepared).is_err());
        assert_eq!(i.stats().pcm.submitted_frames, 0);
        assert!(i.ready(32).unwrap());
    }
}
