//! Pull-only, seekable canonical PCM16 WAV experiment. No output device or worker.
use std::io::{Read, Seek, SeekFrom};
use symphonia::core::{
    codecs::audio::{AudioDecoder, AudioDecoderOptions},
    common::Limit,
    formats::{FormatOptions, FormatReader, SeekMode, SeekTo, TrackType, probe::Hint},
    io::{MediaSource, MediaSourceStream, MediaSourceStreamOptions},
    meta::MetadataOptions,
    units::Timestamp,
};

pub const MAX_BLOCK_FRAMES: usize = 4096;
pub const MAX_FILE_BYTES: u64 = 128 * 1024 * 1024;
pub const MAX_SECONDS: u64 = 600;
pub const SOURCE_BUFFER_BYTES: usize = 64 * 1024;
pub const CACHE_BYTES: usize = MAX_BLOCK_FRAMES * 2 * 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    Io,
    Format,
    Limit,
    Decode,
    Request,
    Poisoned,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Block {
    pub generation: u64,
    pub first_frame: u64,
    pub frames: usize,
    pub eof: bool,
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Stats {
    pub packets: u64,
    pub decoded_frames: u64,
    pub largest_packet_frames: usize,
}
struct Source<S> {
    inner: S,
    len: u64,
}
impl<S: Read> Read for Source<S> {
    fn read(&mut self, output: &mut [u8]) -> std::io::Result<usize> {
        self.inner.read(output)
    }
}
impl<S: Seek> Seek for Source<S> {
    fn seek(&mut self, position: SeekFrom) -> std::io::Result<u64> {
        self.inner.seek(position)
    }
}
impl<S: Read + Seek + Send + Sync> MediaSource for Source<S> {
    fn is_seekable(&self) -> bool {
        true
    }
    fn byte_len(&self) -> Option<u64> {
        Some(self.len)
    }
}

pub struct Stream {
    reader: Box<dyn FormatReader>,
    decoder: Box<dyn AudioDecoder>,
    track_id: u32,
    rate: u32,
    channels: usize,
    total_frames: u64,
    position: u64,
    decode_head: u64,
    generation: u64,
    cache: Box<[f32]>,
    cached_frames: usize,
    cache_offset: usize,
    failed: bool,
    stats: Stats,
}
impl Stream {
    /// Only the self-authored 44-byte-header PCM16 layout is accepted in this slice.
    /// The caller supplies I/O; this library never opens a path or starts a thread.
    pub fn open<S: Read + Seek + Send + Sync + 'static>(mut input: S) -> Result<Self, Error> {
        let len = input.seek(SeekFrom::End(0)).map_err(|_| Error::Io)?;
        if len > MAX_FILE_BYTES {
            return Err(Error::Limit);
        }
        input.seek(SeekFrom::Start(0)).map_err(|_| Error::Io)?;
        let mut h = [0u8; 44];
        input.read_exact(&mut h).map_err(|_| Error::Io)?;
        let u16_at = |n| u16::from_le_bytes(h[n..n + 2].try_into().unwrap());
        let u32_at = |n| u32::from_le_bytes(h[n..n + 4].try_into().unwrap());
        let channels = usize::from(u16_at(22));
        let rate = u32_at(24);
        if &h[..4] != b"RIFF"
            || &h[8..16] != b"WAVEfmt "
            || u32_at(16) != 16
            || u16_at(20) != 1
            || !(1..=2).contains(&channels)
            || !(8000..=192000).contains(&rate)
            || u16_at(32) != channels as u16 * 2
            || u16_at(34) != 16
            || u32_at(28) != rate * channels as u32 * 2
            || &h[36..40] != b"data"
            || u64::from(u32_at(4)) + 8 != len
            || u64::from(u32_at(40)) + 44 != len
            || !(u32_at(40) as usize).is_multiple_of(channels * 2)
        {
            return Err(Error::Format);
        }
        let frames = u64::from(u32_at(40)) / (channels as u64 * 2);
        if frames == 0 || frames > u64::from(rate) * MAX_SECONDS {
            return Err(Error::Limit);
        }
        input.seek(SeekFrom::Start(0)).map_err(|_| Error::Io)?;
        let source = MediaSourceStream::new(
            Box::new(Source { inner: input, len }),
            MediaSourceStreamOptions {
                buffer_len: SOURCE_BUFFER_BYTES,
            },
        );
        let reader = symphonia::default::get_probe()
            .probe(
                &Hint::new(),
                source,
                FormatOptions::default(),
                MetadataOptions::default()
                    .limit_tag_bytes(Limit::Maximum(0))
                    .limit_visual_bytes(Limit::Maximum(0)),
            )
            .map_err(|_| Error::Decode)?;
        let track = reader
            .default_track(TrackType::Audio)
            .ok_or(Error::Format)?;
        let params = track
            .codec_params
            .as_ref()
            .and_then(|p| p.audio())
            .ok_or(Error::Format)?;
        if track.num_frames != Some(frames)
            || params.sample_rate != Some(rate)
            || params.channels.as_ref().map(|c| c.count()) != Some(channels)
        {
            return Err(Error::Format);
        }
        let track_id = track.id;
        let decoder = symphonia::default::get_codecs()
            .make_audio_decoder(params, &AudioDecoderOptions::default())
            .map_err(|_| Error::Decode)?;
        Ok(Self {
            reader,
            decoder,
            track_id,
            rate,
            channels,
            total_frames: frames,
            position: 0,
            decode_head: 0,
            generation: 0,
            cache: vec![0.; CACHE_BYTES / 4].into_boxed_slice(),
            cached_frames: 0,
            cache_offset: 0,
            failed: false,
            stats: Stats::default(),
        })
    }
    pub fn rate(&self) -> u32 {
        self.rate
    }
    pub fn channels(&self) -> usize {
        self.channels
    }
    pub fn total_frames(&self) -> u64 {
        self.total_frames
    }
    pub fn position(&self) -> u64 {
        self.position
    }
    pub fn generation(&self) -> u64 {
        self.generation
    }
    pub fn stats(&self) -> Stats {
        self.stats
    }

    /// Invalid requests leave the current state usable. I/O/decode failures poison it.
    pub fn seek(&mut self, frame: u64) -> Result<(), Error> {
        if self.failed {
            return Err(Error::Poisoned);
        }
        if frame > self.total_frames || self.generation == u64::MAX {
            return Err(Error::Request);
        }
        let result = if frame == self.total_frames {
            Ok(frame)
        } else {
            self.reader
                .seek(
                    SeekMode::Accurate,
                    SeekTo::Timestamp {
                        ts: Timestamp::new(frame as i64),
                        track_id: self.track_id,
                    },
                )
                .map_err(|_| Error::Io)
                .and_then(|s| {
                    if s.track_id != self.track_id
                        || s.required_ts.get() != frame as i64
                        || s.actual_ts.get() < 0
                        || s.actual_ts.get() > frame as i64
                    {
                        Err(Error::Decode)
                    } else {
                        Ok(s.actual_ts.get() as u64)
                    }
                })
        };
        match result {
            Ok(actual) => {
                self.decoder.reset();
                self.decode_head = actual;
                self.position = frame;
                self.cached_frames = 0;
                self.cache_offset = 0;
                self.generation += 1;
                Ok(())
            }
            Err(e) => {
                self.failed = true;
                Err(e)
            }
        }
    }
    /// Produces at most one caller-owned block. No read-ahead occurs between calls.
    /// A successful short EOF block is zero-padded. On decode/I/O failure no partial
    /// samples are published: output is cleared and this stream must be reopened.
    pub fn read(&mut self, output: &mut [f32]) -> Result<Block, Error> {
        if self.failed {
            output.fill(0.);
            return Err(Error::Poisoned);
        }
        if output.is_empty()
            || !output.len().is_multiple_of(self.channels)
            || output.len() / self.channels > MAX_BLOCK_FRAMES
        {
            return Err(Error::Request);
        }
        let first_frame = self.position;
        match self.read_inner(output) {
            Ok(frames) => Ok(Block {
                generation: self.generation,
                first_frame,
                frames,
                eof: self.position == self.total_frames,
            }),
            Err(e) => {
                output.fill(0.);
                self.failed = true;
                Err(e)
            }
        }
    }
    fn read_inner(&mut self, output: &mut [f32]) -> Result<usize, Error> {
        output.fill(0.);
        let mut written = 0;
        while written < output.len() / self.channels && self.position < self.total_frames {
            if self.cache_offset == self.cached_frames {
                self.refill()?;
            }
            let frames = (self.cached_frames - self.cache_offset)
                .min(output.len() / self.channels - written);
            output[written * self.channels..(written + frames) * self.channels].copy_from_slice(
                &self.cache[self.cache_offset * self.channels
                    ..(self.cache_offset + frames) * self.channels],
            );
            written += frames;
            self.cache_offset += frames;
            self.position += frames as u64;
        }
        Ok(written)
    }
    fn refill(&mut self) -> Result<(), Error> {
        loop {
            let packet = self
                .reader
                .next_packet()
                .map_err(|_| Error::Io)?
                .ok_or(Error::Decode)?;
            if packet.track_id != self.track_id
                || packet.pts.get() != self.decode_head as i64
                || packet.trim_start.get() != 0
                || packet.trim_end.get() != 0
                || packet.data.len() > MAX_BLOCK_FRAMES * self.channels * 2
            {
                return Err(Error::Decode);
            }
            let buffer = self.decoder.decode(&packet).map_err(|_| Error::Decode)?;
            let count = buffer.samples_interleaved();
            let frames = count / self.channels;
            if buffer.spec().rate() != self.rate
                || buffer.spec().channels().count() != self.channels
                || !count.is_multiple_of(self.channels)
                || frames == 0
                || frames > MAX_BLOCK_FRAMES
                || packet.dur.get() != frames as u64
                || self.decode_head + frames as u64 > self.total_frames
            {
                return Err(Error::Decode);
            }
            buffer.copy_to_slice_interleaved(&mut self.cache[..count]);
            if self.cache[..count]
                .iter()
                .any(|v| !v.is_finite() || v.abs() > 1.)
            {
                return Err(Error::Decode);
            }
            self.stats.packets += 1;
            self.stats.decoded_frames += frames as u64;
            self.stats.largest_packet_frames = self.stats.largest_packet_frames.max(frames);
            self.cache_offset = self
                .position
                .saturating_sub(self.decode_head)
                .min(frames as u64) as usize;
            self.cached_frames = frames;
            self.decode_head += frames as u64;
            if self.cache_offset < frames {
                return Ok(());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io::Cursor,
        sync::{
            Arc,
            atomic::{AtomicBool, AtomicU64, Ordering},
        },
    };
    pub fn wav(frames: usize, channels: u16) -> Vec<u8> {
        let mut b = Vec::new();
        let size = frames as u32 * u32::from(channels) * 2;
        b.extend(b"RIFF");
        b.extend((36 + size).to_le_bytes());
        b.extend(b"WAVEfmt ");
        b.extend(16u32.to_le_bytes());
        b.extend(1u16.to_le_bytes());
        b.extend(channels.to_le_bytes());
        b.extend(48000u32.to_le_bytes());
        b.extend((48000 * u32::from(channels) * 2).to_le_bytes());
        b.extend((channels * 2).to_le_bytes());
        b.extend(16u16.to_le_bytes());
        b.extend(b"data");
        b.extend(size.to_le_bytes());
        for i in 0..frames * usize::from(channels) {
            b.extend(((i as i32 % 32768 - 16384) as i16).to_le_bytes());
        }
        b
    }
    fn expected(frame: u64, channel: usize, channels: usize) -> f32 {
        (((frame as usize * channels + channel) % 32768) as i32 - 16384) as f32 / 32768.
    }
    #[test]
    fn arbitrary_pull_sizes_preserve_every_frame_channel_and_zero_pad_eof() {
        for channels in [1, 2] {
            let mut s = Stream::open(Cursor::new(wav(10001, channels))).unwrap();
            let mut frame = 0;
            for size in [1, 17, 4096, 3, 4000, 4096] {
                let mut out = vec![7.; size * channels as usize];
                let b = s.read(&mut out).unwrap();
                assert_eq!(b.first_frame, frame);
                assert_eq!(b.generation, 0);
                for i in 0..b.frames {
                    for ch in 0..channels as usize {
                        assert_eq!(
                            out[i * channels as usize + ch],
                            expected(frame + i as u64, ch, channels as usize)
                        );
                    }
                }
                assert!(out[b.frames * channels as usize..].iter().all(|v| *v == 0.));
                frame += b.frames as u64;
            }
            assert_eq!(frame, 10001);
            assert_eq!(s.read(&mut [3.; 2]).unwrap().frames, 0);
            assert!(s.stats().largest_packet_frames <= MAX_BLOCK_FRAMES);
        }
    }
    #[test]
    fn unaligned_forward_backward_and_end_seeks_never_reuse_old_cached_pcm() {
        let mut s = Stream::open(Cursor::new(wav(20000, 2))).unwrap();
        let mut out = [0.; 514];
        s.read(&mut out).unwrap();
        for (n, target) in [11003, 7, 19991, 20000, 1001].into_iter().enumerate() {
            s.seek(target).unwrap();
            let b = s.read(&mut out).unwrap();
            assert_eq!((b.first_frame, b.generation), (target, n as u64 + 1));
            for i in 0..b.frames {
                for ch in 0..2 {
                    assert_eq!(out[i * 2 + ch], expected(target + i as u64, ch, 2));
                }
            }
            assert!(out[b.frames * 2..].iter().all(|v| *v == 0.));
        }
    }
    #[test]
    fn invalid_requests_leave_position_generation_cache_and_output_unchanged() {
        let mut s = Stream::open(Cursor::new(wav(10000, 2))).unwrap();
        s.read(&mut [0.; 6]).unwrap();
        let before = (s.position(), s.generation(), s.stats());
        assert_eq!(s.seek(10001), Err(Error::Request));
        let mut odd = [9.; 3];
        assert_eq!(s.read(&mut odd), Err(Error::Request));
        assert_eq!(odd, [9.; 3]);
        assert_eq!(s.read(&mut []), Err(Error::Request));
        assert_eq!(
            s.read(&mut vec![0.; MAX_BLOCK_FRAMES * 2 + 2]),
            Err(Error::Request)
        );
        assert_eq!((s.position(), s.generation(), s.stats()), before);
        let b = s.read(&mut [0.; 2]).unwrap();
        assert_eq!(b.first_frame, 3);
    }
    struct Watched {
        data: Cursor<Vec<u8>>,
        reads: Arc<AtomicU64>,
        fail: Arc<AtomicBool>,
    }
    impl Read for Watched {
        fn read(&mut self, output: &mut [u8]) -> std::io::Result<usize> {
            self.reads.fetch_add(1, Ordering::Relaxed);
            if self.fail.load(Ordering::Relaxed) {
                return Err(std::io::ErrorKind::Other.into());
            }
            self.data.read(output)
        }
    }
    impl Seek for Watched {
        fn seek(&mut self, position: SeekFrom) -> std::io::Result<u64> {
            self.data.seek(position)
        }
    }
    #[test]
    fn consumer_backpressure_has_no_background_work_and_reads_only_a_bounded_prefix() {
        let reads = Arc::new(AtomicU64::new(0));
        let fail = Arc::new(AtomicBool::new(false));
        let mut s = Stream::open(Watched {
            data: Cursor::new(wav(48000 * 30, 2)),
            reads: reads.clone(),
            fail,
        })
        .unwrap();
        s.read(&mut [0.; 2]).unwrap();
        let before = (reads.load(Ordering::Relaxed), s.stats(), s.position());
        for _ in 0..100 {
            assert_eq!(
                (reads.load(Ordering::Relaxed), s.stats(), s.position()),
                before
            );
        }
        assert!(s.stats().decoded_frames <= MAX_BLOCK_FRAMES as u64);
    }
    #[test]
    fn source_failure_after_partial_work_clears_output_and_requires_reopen() {
        let fail = Arc::new(AtomicBool::new(false));
        let mut s = Stream::open(Watched {
            data: Cursor::new(wav(48000 * 3, 2)),
            reads: Arc::new(AtomicU64::new(0)),
            fail: fail.clone(),
        })
        .unwrap();
        fail.store(true, Ordering::Relaxed);
        let mut out = vec![9.; MAX_BLOCK_FRAMES * 2];
        for _ in 0..100 {
            if s.read(&mut out).is_err() {
                assert!(out.iter().all(|v| *v == 0.));
                assert_eq!(s.read(&mut out), Err(Error::Poisoned));
                assert_eq!(s.seek(0), Err(Error::Poisoned));
                return;
            }
        }
        panic!("source failure was not surfaced");
    }
    #[test]
    fn strict_layout_truncation_size_duration_and_format_limits_are_explicit() {
        for index in [0, 8, 12, 16, 20, 22, 28, 32, 34, 36, 40] {
            let mut b = wav(100, 2);
            b[index] ^= 7;
            assert!(Stream::open(Cursor::new(b)).is_err(), "{index}");
        }
        let mut short = wav(100, 2);
        short.pop();
        assert!(Stream::open(Cursor::new(short)).is_err());
        assert!(matches!(
            Stream::open(Cursor::new(wav(0, 1))),
            Err(Error::Limit)
        ));
        // A tiny source can advertise an oversized length; rejection precedes reads.
        struct Huge;
        impl Read for Huge {
            fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
                panic!("must reject before reading")
            }
        }
        impl Seek for Huge {
            fn seek(&mut self, _: SeekFrom) -> std::io::Result<u64> {
                Ok(MAX_FILE_BYTES + 1)
            }
        }
        assert!(matches!(Stream::open(Huge), Err(Error::Limit)));
        // Valid-looking small header with an over-limit duration is rejected
        // before any attempt to decode its advertised (absent) payload.
        struct HeaderOnly {
            data: Cursor<Vec<u8>>,
            len: u64,
        }
        impl Read for HeaderOnly {
            fn read(&mut self, output: &mut [u8]) -> std::io::Result<usize> {
                self.data.read(output)
            }
        }
        impl Seek for HeaderOnly {
            fn seek(&mut self, position: SeekFrom) -> std::io::Result<u64> {
                if position == SeekFrom::End(0) {
                    Ok(self.len)
                } else {
                    self.data.seek(position)
                }
            }
        }
        let mut h = wav(1, 1);
        h.truncate(44);
        let data_bytes = 48000u32 * 601 * 2;
        h[4..8].copy_from_slice(&(data_bytes + 36).to_le_bytes());
        h[40..44].copy_from_slice(&data_bytes.to_le_bytes());
        assert!(matches!(
            Stream::open(HeaderOnly {
                data: Cursor::new(h),
                len: u64::from(data_bytes) + 44
            }),
            Err(Error::Limit)
        ));
    }
}
