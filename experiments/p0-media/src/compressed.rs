//! Optional bounded pull experiment; codec selection is not a stable rule.
use crate::stream::{self, Block, Error, MAX_BLOCK_FRAMES, MAX_FILE_BYTES, MAX_SECONDS, Stats};
use std::io::{Read, Seek, SeekFrom};
use symphonia::core::{
    codecs::audio::{AudioDecoder, AudioDecoderOptions},
    common::Limit,
    formats::{FormatOptions, FormatReader, SeekMode, SeekTo, TrackType, probe::Hint},
    io::{MediaSource, MediaSourceStream, MediaSourceStreamOptions},
    meta::MetadataOptions,
    units::Timestamp,
};
pub const MAX_PACKET_FRAMES: usize = 8192;
pub const MAX_PACKET_BYTES: usize = 256 * 1024;
pub const MAX_PACKETS: u64 = 120000;
pub const CACHE_BYTES: usize = MAX_PACKET_FRAMES * 2 * 4;
struct Source<S> {
    inner: S,
    len: u64,
}
impl<S: Read> Read for Source<S> {
    fn read(&mut self, out: &mut [u8]) -> std::io::Result<usize> {
        self.inner.read(out)
    }
}
impl<S: Seek> Seek for Source<S> {
    fn seek(&mut self, p: SeekFrom) -> std::io::Result<u64> {
        self.inner.seek(p)
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
/// WAV retains its original strict reader. Compressed packets use a separate cache.
pub struct Stream(Inner);
enum Inner {
    Wav(stream::Stream),
    Compressed(Box<Decoded>),
}
struct Decoded {
    reader: Box<dyn FormatReader>,
    decoder: Box<dyn AudioDecoder>,
    track: u32,
    vorbis: bool,
    seek_limit: i64,
    rate: u32,
    channels: usize,
    total: u64,
    position: u64,
    generation: u64,
    cache: Box<[f32]>,
    cached: usize,
    offset: usize,
    failed: bool,
    decoded_this_pass: u64,
    stats: Stats,
}
impl Stream {
    pub fn open<S: Read + Seek + Send + Sync + 'static>(mut source: S) -> Result<Self, Error> {
        let len = source.seek(SeekFrom::End(0)).map_err(|_| Error::Io)?;
        if len > MAX_FILE_BYTES {
            return Err(Error::Limit);
        }
        source.seek(SeekFrom::Start(0)).map_err(|_| Error::Io)?;
        let mut prefix = [0; 12];
        source.read_exact(&mut prefix).map_err(|_| Error::Io)?;
        source.seek(SeekFrom::Start(0)).map_err(|_| Error::Io)?;
        if &prefix[..4] == b"RIFF" {
            return stream::Stream::open(source).map(|s| Self(Inner::Wav(s)));
        }
        let reader = symphonia::default::get_probe()
            .probe(
                &Hint::new(),
                MediaSourceStream::new(
                    Box::new(Source { inner: source, len }),
                    MediaSourceStreamOptions {
                        buffer_len: stream::SOURCE_BUFFER_BYTES,
                    },
                ),
                FormatOptions::default(),
                MetadataOptions::default()
                    .limit_tag_bytes(Limit::Maximum(0))
                    .limit_visual_bytes(Limit::Maximum(0)),
            )
            .map_err(|_| Error::Format)?;
        if reader.tracks().len() != 1 {
            return Err(Error::Format);
        }
        let track = reader
            .default_track(TrackType::Audio)
            .ok_or(Error::Format)?;
        let params = track
            .codec_params
            .as_ref()
            .and_then(|p| p.audio())
            .ok_or(Error::Format)?;
        if !matches!(
            params.codec,
            symphonia::core::codecs::audio::well_known::CODEC_ID_MP3
                | symphonia::core::codecs::audio::well_known::CODEC_ID_VORBIS
        ) {
            return Err(Error::Format);
        }
        let rate = params.sample_rate.ok_or(Error::Format)?;
        let channels = params
            .channels
            .as_ref()
            .map(|c| c.count())
            .ok_or(Error::Format)?;
        let total = track.num_frames.ok_or(Error::Format)?;
        if !(8000..=192000).contains(&rate)
            || !(1..=2).contains(&channels)
            || total == 0
            || total > u64::from(rate) * MAX_SECONDS
        {
            return Err(Error::Limit);
        }
        let decoder = symphonia::default::get_codecs()
            .make_audio_decoder(params, &AudioDecoderOptions::default())
            .map_err(|_| Error::Decode)?;
        let id = track.id;
        let vorbis = params.codec == symphonia::core::codecs::audio::well_known::CODEC_ID_VORBIS;
        // Symphonia 0.6.1 checks seeks against start_ts + num_frames, while
        // Vorbis start_ts can include negative priming. Seek within that bound
        // and decode the remaining tail, without trimming or padding PCM.
        let seek_limit = track
            .start_ts
            .get()
            .checked_add(total as i64)
            .ok_or(Error::Limit)?;
        Ok(Self(Inner::Compressed(Box::new(Decoded {
            reader,
            decoder,
            track: id,
            vorbis,
            seek_limit,
            rate,
            channels,
            total,
            position: 0,
            generation: 0,
            cache: vec![0.; CACHE_BYTES / 4].into_boxed_slice(),
            cached: 0,
            offset: 0,
            failed: false,
            decoded_this_pass: 0,
            stats: Stats::default(),
        }))))
    }
    pub fn rate(&self) -> u32 {
        match &self.0 {
            Inner::Wav(s) => s.rate(),
            Inner::Compressed(s) => s.rate,
        }
    }
    pub fn channels(&self) -> usize {
        match &self.0 {
            Inner::Wav(s) => s.channels(),
            Inner::Compressed(s) => s.channels,
        }
    }
    pub fn total_frames(&self) -> u64 {
        match &self.0 {
            Inner::Wav(s) => s.total_frames(),
            Inner::Compressed(s) => s.total,
        }
    }
    pub fn position(&self) -> u64 {
        match &self.0 {
            Inner::Wav(s) => s.position(),
            Inner::Compressed(s) => s.position,
        }
    }
    pub fn generation(&self) -> u64 {
        match &self.0 {
            Inner::Wav(s) => s.generation(),
            Inner::Compressed(s) => s.generation,
        }
    }
    pub fn stats(&self) -> Stats {
        match &self.0 {
            Inner::Wav(s) => s.stats(),
            Inner::Compressed(s) => s.stats,
        }
    }
    pub fn read(&mut self, out: &mut [f32]) -> Result<Block, Error> {
        match &mut self.0 {
            Inner::Wav(s) => s.read(out),
            Inner::Compressed(s) => s.read(out),
        }
    }
    /// Vorbis uses accurate container seeking with decoder pre-roll; MP3 retains
    /// the prefix fallback until its reservoir behaviour is separately verified.
    pub fn seek(&mut self, frame: u64) -> Result<(), Error> {
        while !self.seek_step(frame)? {}
        Ok(())
    }
    /// Advance an MP3 prefix by at most MAX_BLOCK_FRAMES logical frames. False
    /// means the caller must step again before reading the requested position.
    /// The caller may abandon the stream or supply a new target between steps.
    /// WAV and Vorbis keep their existing synchronous seek paths. This bounds
    /// prefix decode work, not source I/O or container/decoder wall time.
    pub fn seek_step(&mut self, frame: u64) -> Result<bool, Error> {
        match &mut self.0 {
            Inner::Wav(s) => s.seek(frame).map(|()| true),
            Inner::Compressed(s) => s.seek(frame),
        }
    }
}

impl Decoded {
    fn seek(&mut self, frame: u64) -> Result<bool, Error> {
        if self.failed {
            return Err(Error::Poisoned);
        }
        if frame > self.total || self.generation == u64::MAX {
            return Err(Error::Request);
        }
        let result = (|| {
            if self.vorbis && frame == self.total {
                self.position = frame;
                self.cached = 0;
                self.offset = 0;
                return Ok(true);
            }
            if self.vorbis || frame < self.position {
                self.reader
                    .seek(
                        SeekMode::Accurate,
                        SeekTo::Timestamp {
                            ts: Timestamp::new(if self.vorbis {
                                (frame as i64).min(self.seek_limit)
                            } else {
                                0
                            }),
                            track_id: self.track,
                        },
                    )
                    .map_err(|_| Error::Io)?;
                self.decoder.reset();
                self.position = 0;
                self.cached = 0;
                self.offset = 0;
                self.decoded_this_pass = 0;
                if self.vorbis {
                    // The first packet after reset can produce no samples. Anchor
                    // to the first actual PCM packet, including its start trim.
                    let first = self.refill()?;
                    if first > frame {
                        return Err(Error::Decode);
                    }
                    self.position = first;
                }
            }
            let mut scratch = [0.; MAX_BLOCK_FRAMES * 2];
            while self.position < frame {
                let frames = (frame - self.position).min(MAX_BLOCK_FRAMES as u64) as usize;
                self.read(&mut scratch[..frames * self.channels])?;
                if !self.vorbis {
                    break;
                }
            }
            Ok(self.position == frame)
        })();
        if result.is_err() {
            self.failed = true;
        } else if result == Ok(true) {
            self.generation += 1;
        }
        result
    }
    fn read(&mut self, out: &mut [f32]) -> Result<Block, Error> {
        if self.failed {
            out.fill(0.);
            return Err(Error::Poisoned);
        }
        if out.is_empty()
            || !out.len().is_multiple_of(self.channels)
            || out.len() / self.channels > MAX_BLOCK_FRAMES
        {
            return Err(Error::Request);
        }
        let first = self.position;
        let result = (|| {
            out.fill(0.);
            let mut written = 0;
            while written < out.len() / self.channels && self.position < self.total {
                if self.offset == self.cached {
                    self.refill()?;
                }
                let frames = (self.cached - self.offset)
                    .min(out.len() / self.channels - written)
                    .min((self.total - self.position) as usize);
                out[written * self.channels..(written + frames) * self.channels].copy_from_slice(
                    &self.cache
                        [self.offset * self.channels..(self.offset + frames) * self.channels],
                );
                written += frames;
                self.offset += frames;
                self.position += frames as u64;
            }
            Ok(Block {
                generation: self.generation,
                first_frame: first,
                frames: written,
                eof: self.position == self.total,
            })
        })();
        if result.is_err() {
            out.fill(0.);
            self.failed = true;
        }
        result
    }
    fn refill(&mut self) -> Result<u64, Error> {
        loop {
            let p = self
                .reader
                .next_packet()
                .map_err(|_| Error::Io)?
                .ok_or(Error::Decode)?;
            if self.stats.packets >= MAX_PACKETS
                || p.track_id != self.track
                || p.data.len() > MAX_PACKET_BYTES
            {
                return Err(Error::Limit);
            }
            self.stats.packets += 1;
            let b = self.decoder.decode(&p).map_err(|_| Error::Decode)?;
            let count = b.samples_interleaved();
            let frames = count / self.channels;
            if b.spec().rate() != self.rate
                || b.spec().channels().count() != self.channels
                || !count.is_multiple_of(self.channels)
                || frames > MAX_PACKET_FRAMES
                || self.decoded_this_pass + frames as u64 > self.total
            {
                return Err(Error::Decode);
            }
            self.stats.decoded_frames += frames as u64;
            self.decoded_this_pass += frames as u64;
            self.stats.largest_packet_frames = self.stats.largest_packet_frames.max(frames);
            if frames == 0 {
                continue;
            }
            b.copy_to_slice_interleaved(&mut self.cache[..count]);
            if self.cache[..count]
                .iter()
                .any(|v| !v.is_finite() || v.abs() > 1.)
            {
                return Err(Error::Decode);
            }
            self.cached = frames;
            self.offset = 0;
            if !self.vorbis {
                return Ok(0);
            }
            let first = p
                .pts
                .get()
                .checked_add(p.trim_start.get() as i64)
                .ok_or(Error::Decode)?;
            return u64::try_from(first).map_err(|_| Error::Decode);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    const MP3: &[u8] = include_bytes!("../tests/fixtures/audio-0_37.mp3");
    const OGG: &[u8] = include_bytes!("../tests/fixtures/audio-0_37.ogg");
    fn decode(data: &'static [u8]) -> Vec<f32> {
        let mut s = Stream::open(Cursor::new(data)).unwrap();
        let expected_frames = s.total_frames();
        let mut result = Vec::new();
        let mut block = vec![0.; MAX_BLOCK_FRAMES * 2];
        loop {
            let b = s.read(&mut block).unwrap();
            result.extend_from_slice(&block[..b.frames * 2]);
            if b.eof {
                break;
            }
        }
        assert_eq!(result.len(), expected_frames as usize * 2);
        result
    }
    #[test]
    fn mp3_gapless_samples_match_independent_ffmpeg_reference() {
        let actual = decode(MP3);
        let expected = include_bytes!("../tests/fixtures/audio-0_37.mp3.reference.f32");
        assert_eq!(actual.len() * 4, expected.len());
        for (a, b) in actual.iter().zip(expected.chunks_exact(4)) {
            assert!((a - f32::from_le_bytes(b.try_into().unwrap())).abs() <= 0.00001);
        }
    }
    #[test]
    fn short_vorbis_including_tail_matches_independent_libsndfile() {
        let actual = decode(OGG);
        let reference = include_bytes!("../tests/fixtures/audio-0_37.ogg.sndfile.f32");
        assert_eq!(actual.len(), 17760 * 2);
        assert_eq!(actual.len() * 4, reference.len());
        for (a, b) in actual.iter().zip(reference.chunks_exact(4)) {
            assert!((a - f32::from_le_bytes(b.try_into().unwrap())).abs() <= 0.00001);
        }
    }
    #[test]
    fn vorbis_multipage_random_seeks_are_exact_without_decoding_the_prefix() {
        let data = include_bytes!("../tests/fixtures/seek-3.ogg");
        let reference = decode(data);
        let mut s = Stream::open(Cursor::new(data.as_slice())).unwrap();
        s.seek(96007).unwrap();
        assert!(s.stats().decoded_frames < 24000);
        for target in [
            96007, 0, 48001, 143999, 127, 4095, 4096, 4097, 71999, 144000, 1,
        ] {
            s.seek(target)
                .unwrap_or_else(|e| panic!("seek {target}: {e:?}"));
            let mut out = [9.; 514];
            let block = s.read(&mut out).unwrap();
            assert_eq!(block.first_frame, target);
            assert_eq!(block.frames, (144000 - target).min(257) as usize);
            assert_eq!(
                &out[..block.frames * 2],
                &reference[target as usize * 2..(target as usize + block.frames) * 2]
            );
            assert!(out[block.frames * 2..].iter().all(|v| *v == 0.));
        }
    }
    #[test]
    fn both_codecs_seek_after_eof_forward_and_backward_match_linear_decode() {
        for data in [MP3, OGG] {
            let reference = decode(data);
            let mut s = Stream::open(Cursor::new(data)).unwrap();
            s.seek(17760).unwrap();
            assert!(s.read(&mut [0.; 16]).unwrap().eof);
            for (n, target) in [8887, 11, 17751, 17760, 0].into_iter().enumerate() {
                s.seek(target)
                    .unwrap_or_else(|e| panic!("seek {target}: {e:?}"));
                let mut samples = vec![9.; 514];
                let b = s.read(&mut samples).unwrap();
                assert_eq!(b.generation, n as u64 + 2);
                assert_eq!(b.first_frame, target);
                assert_eq!(
                    &samples[..b.frames * 2],
                    &reference[target as usize * 2..(target as usize + b.frames) * 2]
                );
                assert!(samples[b.frames * 2..].iter().all(|v| *v == 0.));
            }
        }
    }
    #[test]
    fn invalid_requests_preserve_stream_position_and_do_not_clear_caller_data() {
        let mut s = Stream::open(Cursor::new(MP3)).unwrap();
        s.seek(123).unwrap();
        let g = s.generation();
        assert_eq!(s.seek(17761), Err(Error::Request));
        assert_eq!((s.position(), s.generation()), (123, g));
        let mut bad = vec![9.; (MAX_BLOCK_FRAMES + 1) * 2];
        assert_eq!(s.read(&mut bad), Err(Error::Request));
        assert!(bad.iter().all(|v| *v == 9.));
        assert_eq!(s.read(&mut [0.; 2]).unwrap().first_frame, 123);
    }
    #[test]
    fn mp3_seek_steps_are_bounded_and_preserve_exact_samples() {
        let reference = decode(MP3);
        let mut s = Stream::open(Cursor::new(MP3)).unwrap();
        let mut steps = 0;
        loop {
            let before = s.position();
            let decoded = s.stats().decoded_frames;
            let ready = s.seek_step(15847).unwrap();
            assert!(s.position() - before <= MAX_BLOCK_FRAMES as u64);
            assert!(
                s.stats().decoded_frames - decoded <= (MAX_BLOCK_FRAMES + MAX_PACKET_FRAMES) as u64
            );
            steps += 1;
            assert_eq!(s.generation(), u64::from(ready));
            if ready {
                break;
            }
        }
        assert_eq!(steps, 4);
        let mut samples = [0.; 514];
        let b = s.read(&mut samples).unwrap();
        assert_eq!(b.first_frame, 15847);
        assert_eq!(samples, reference[15847 * 2..(15847 + 257) * 2]);
    }
    #[test]
    fn partial_mp3_seek_can_be_replaced_and_invalid_target_is_non_mutating() {
        let reference = decode(MP3);
        let mut s = Stream::open(Cursor::new(MP3)).unwrap();
        assert!(!s.seek_step(16000).unwrap());
        assert_eq!(s.position(), 4096);
        let before = (s.position(), s.generation(), s.stats());
        assert_eq!(s.seek_step(17761), Err(Error::Request));
        assert_eq!((s.position(), s.generation(), s.stats()), before);
        assert!(s.seek_step(37).unwrap());
        assert_eq!((s.position(), s.generation()), (37, 1));
        let mut samples = [0.; 514];
        s.read(&mut samples).unwrap();
        assert_eq!(samples, reference[37 * 2..(37 + 257) * 2]);
        assert!(!s.seek_step(16000).unwrap());
        // Abandoning an incomplete request requires no completion or rollback.
        drop(s);
    }
    #[test]
    fn partial_mp3_seek_failure_stays_poisoned_and_cannot_complete_later() {
        let mut s = Stream::open(Cursor::new(&MP3[..MP3.len() / 2])).unwrap();
        let mut failed = false;
        for _ in 0..5 {
            match s.seek_step(16000) {
                Ok(ready) => assert!(!ready),
                Err(_) => {
                    failed = true;
                    break;
                }
            }
        }
        assert!(failed);
        assert_eq!(s.generation(), 0);
        assert_eq!(s.seek_step(0), Err(Error::Poisoned));
        let mut out = [9.; 16];
        assert_eq!(s.read(&mut out), Err(Error::Poisoned));
        assert_eq!(out, [0.; 16]);
    }
    #[test]
    fn truncated_packets_fail_without_delivering_a_partial_block_and_stay_poisoned() {
        let mut s = Stream::open(Cursor::new(&MP3[..MP3.len() / 2])).unwrap();
        let mut block = vec![9.; MAX_BLOCK_FRAMES * 2];
        let mut failed = false;
        for _ in 0..10 {
            match s.read(&mut block) {
                Err(_) => {
                    failed = true;
                    assert!(block.iter().all(|v| *v == 0.));
                    break;
                }
                Ok(b) => assert!(!b.eof),
            }
        }
        assert!(failed);
        assert_eq!(s.read(&mut block), Err(Error::Poisoned));
        assert_eq!(s.seek(0), Err(Error::Poisoned));
    }
    #[test]
    fn malformed_and_oversized_sources_are_rejected() {
        assert!(Stream::open(Cursor::new(vec![0; 128])).is_err());
        struct Huge;
        impl Read for Huge {
            fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
                Err(std::io::Error::other("must reject before reading"))
            }
        }
        impl Seek for Huge {
            fn seek(&mut self, _: SeekFrom) -> std::io::Result<u64> {
                Ok(MAX_FILE_BYTES + 1)
            }
        }
        assert!(matches!(Stream::open(Huge), Err(Error::Limit)));
    }
}
