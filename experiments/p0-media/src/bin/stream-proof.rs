//! Headless, no-device resource probe. Only opens runner-owned WAV fixtures.
use p0_media::stream::{CACHE_BYTES, MAX_BLOCK_FRAMES, SOURCE_BUFFER_BYTES, Stream};
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom, Write},
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};

struct Counted {
    file: File,
    bytes: Arc<AtomicU64>,
}
impl Read for Counted {
    fn read(&mut self, out: &mut [u8]) -> std::io::Result<usize> {
        let n = self.file.read(out)?;
        self.bytes.fetch_add(n as u64, Ordering::Relaxed);
        Ok(n)
    }
}
impl Seek for Counted {
    fn seek(&mut self, position: SeekFrom) -> std::io::Result<u64> {
        self.file.seek(position)
    }
}
fn hash(mut h: u64, samples: &[f32]) -> u64 {
    for sample in samples {
        for b in sample.to_le_bytes() {
            h = (h ^ u64::from(b)).wrapping_mul(0x100000001b3);
        }
    }
    h
}
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() != 3 {
        return Err("expected mode and runner fixture".into());
    }
    let mode = args[1].to_str().ok_or("invalid mode")?;
    if mode != "stream" && mode != "eager" {
        return Err("invalid mode".into());
    }
    let fixture = PathBuf::from(&args[2]).canonicalize()?;
    let allowed = std::env::current_dir()?
        .join("artifacts/p0")
        .canonicalize()?;
    let parent = fixture.parent().ok_or("missing fixture directory")?;
    if parent.parent() != Some(allowed.as_path())
        || !parent
            .file_name()
            .and_then(|s| s.to_str())
            .is_some_and(|n| n.starts_with("stream-fixture-"))
        || !["short.wav", "long.wav"]
            .iter()
            .any(|n| fixture.file_name().is_some_and(|f| f == *n))
    {
        return Err("fixture is outside runner-owned inputs".into());
    }
    // Wait for the runner's explicit baseline; metrics initialization may be slow.
    println!("{{\"phase\":\"ready\"}}");
    std::io::stdout().flush()?;
    let mut command = String::new();
    std::io::stdin().read_line(&mut command)?;
    if command != "run\n" {
        return Err("missing runner handshake".into());
    }
    let bytes = Arc::new(AtomicU64::new(0));
    let encoded_bytes = fixture.metadata()?.len();
    let open_start = Instant::now();
    let mut stream = Stream::open(Counted {
        file: File::open(fixture)?,
        bytes: bytes.clone(),
    })
    .map_err(|_| "open failed")?;
    let open_ms = open_start.elapsed().as_secs_f64() * 1000.;
    let (rate, channels, total) = (stream.rate(), stream.channels(), stream.total_frames());
    let open_read_bytes = bytes.load(Ordering::Relaxed);
    let mut out = vec![0.; MAX_BLOCK_FRAMES * channels];
    let mut retained = if mode == "eager" {
        Vec::with_capacity(total as usize * channels)
    } else {
        Vec::new()
    };
    let mut pulls_us = Vec::new();
    let (mut pcm_hash, mut frames) = (0xcbf29ce484222325, 0u64);
    let work = Instant::now();
    loop {
        let start = Instant::now();
        let b = stream.read(&mut out).map_err(|_| "read failed")?;
        pulls_us.push(start.elapsed().as_secs_f64() * 1_000_000.);
        if b.first_frame != frames || b.generation != 0 {
            return Err("discontinuous block".into());
        }
        frames += b.frames as u64;
        let samples = &out[..b.frames * channels];
        pcm_hash = hash(pcm_hash, samples);
        if mode == "eager" {
            retained.extend_from_slice(samples);
        }
        if b.eof {
            break;
        }
    }
    let work_ms = work.elapsed().as_secs_f64() * 1000.;
    if frames != total {
        return Err("incomplete output".into());
    }
    let sequential_read_bytes = bytes.load(Ordering::Relaxed);
    let sequential_stats = stream.stats();
    let mut seeks = Vec::new();
    for target in [total / 2 + 7, 11, total - 9, total] {
        let start = Instant::now();
        stream.seek(target).map_err(|_| "seek failed")?;
        let b = stream
            .read(&mut out[..257 * channels])
            .map_err(|_| "post-seek read failed")?;
        let elapsed = start.elapsed().as_secs_f64() * 1_000_000.;
        if b.first_frame != target || b.generation != seeks.len() as u64 + 1 {
            return Err("stale block after seek".into());
        }
        if out[b.frames * channels..257 * channels]
            .iter()
            .any(|v| *v != 0.)
        {
            return Err("nonzero EOF tail".into());
        }
        seeks.push(format!("{{\"target\":{target},\"generation\":{},\"frames\":{},\"eof\":{},\"hash\":{},\"wall_us\":{elapsed}}}", b.generation, b.frames, b.eof, hash(0xcbf29ce484222325, &out[..b.frames * channels])));
    }
    let idle = (
        bytes.load(Ordering::Relaxed),
        stream.position(),
        stream.stats(),
    );
    std::thread::sleep(Duration::from_millis(350));
    let idle_unchanged = idle
        == (
            bytes.load(Ordering::Relaxed),
            stream.position(),
            stream.stats(),
        );
    if !idle_unchanged {
        return Err("work continued without consumer demand".into());
    }
    std::hint::black_box(&retained);
    let sum_us: f64 = pulls_us.iter().sum();
    let first_pull_us = pulls_us[0];
    pulls_us.sort_by(f64::total_cmp);
    let p95_us = pulls_us[(pulls_us.len() * 95 / 100).min(pulls_us.len() - 1)];
    let max_us = pulls_us[pulls_us.len() - 1];
    println!(
        "{{\"schema_version\":1,\"status\":\"passed\",\"mode\":\"{mode}\",\"source\":\"self_authored_canonical_pcm16_wav\",\"rate\":{rate},\"channels\":{channels},\"frames\":{frames},\"encoded_bytes\":{encoded_bytes},\"decoded_payload_bytes\":{},\"retained_pcm_bytes\":{},\"caller_buffer_bytes\":{},\"cache_bytes\":{CACHE_BYTES},\"source_buffer_bytes\":{SOURCE_BUFFER_BYTES},\"open_read_bytes\":{open_read_bytes},\"sequential_read_bytes\":{sequential_read_bytes},\"pcm_hash\":{pcm_hash},\"open_ms\":{open_ms},\"first_pull_us\":{first_pull_us},\"read_sum_us\":{sum_us},\"read_p95_us\":{p95_us},\"read_max_us\":{max_us},\"work_ms\":{work_ms},\"sequential_packets\":{},\"sequential_decoded_frames\":{},\"largest_packet_frames\":{},\"idle_ms\":350,\"idle_unchanged\":{idle_unchanged},\"seeks\":[{}],\"audio_device_opened\":false,\"system_capture_used\":false}}",
        frames * channels as u64 * 4,
        retained.len() * 4,
        out.len() * 4,
        sequential_stats.packets,
        sequential_stats.decoded_frames,
        sequential_stats.largest_packet_frames,
        seeks.join(",")
    );
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("stream proof: {error}");
        std::process::exit(1);
    }
}
