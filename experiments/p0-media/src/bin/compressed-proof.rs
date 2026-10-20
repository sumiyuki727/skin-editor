#[cfg(feature = "compressed")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use p0_media::{compressed::Stream, stream::MAX_BLOCK_FRAMES};
    use std::{
        fs::File,
        io::Write,
        path::PathBuf,
        time::{Duration, Instant},
    };
    let mut args = std::env::args().skip(1);
    let source = PathBuf::from(args.next().ok_or("source")?);
    let output = PathBuf::from(args.next().ok_or("new output")?);
    let allowed = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("artifacts/p0")
        .canonicalize()?;
    assert!(source.canonicalize()?.starts_with(&allowed));
    assert_eq!(output.parent().unwrap().canonicalize()?, allowed);
    assert!(!output.exists());
    assert!(args.next().is_none());
    std::fs::create_dir(&output)?;
    // Startup handshake keeps the first resource baseline observable.
    std::fs::write(output.join("ready"), b"1")?;
    let handshake = Instant::now();
    while !output.join("go").exists() {
        if handshake.elapsed() > Duration::from_secs(15) {
            return Err("Handshake timeout".into());
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    let start = Instant::now();
    let mut stream = Stream::open(File::open(source)?).map_err(|e| format!("{e:?}"))?;
    let open_ms = start.elapsed().as_secs_f64() * 1000.;
    let channels = stream.channels();
    let total = stream.total_frames();
    let mut pcm = File::create_new(output.join("decoded.f32"))?;
    let mut buffer = vec![0.; MAX_BLOCK_FRAMES * channels];
    let mut bytes = Vec::with_capacity(buffer.len() * 4);
    let start = Instant::now();
    let mut blocks = 0;
    loop {
        let b = stream.read(&mut buffer).map_err(|e| format!("{e:?}"))?;
        bytes.clear();
        for v in &buffer[..b.frames * channels] {
            bytes.extend(v.to_le_bytes());
        }
        pcm.write_all(&bytes)?;
        blocks += 1;
        if b.eof {
            break;
        }
    }
    let decode_ms = start.elapsed().as_secs_f64() * 1000.;
    let mut seeks = Vec::new();
    for target in [total / 2 + 7, 11, total.saturating_sub(9), total] {
        let start = Instant::now();
        let mut steps = 0;
        let mut max_step = Duration::ZERO;
        loop {
            let step_start = Instant::now();
            let ready = stream.seek_step(target).map_err(|e| format!("{e:?}"))?;
            max_step = max_step.max(step_start.elapsed());
            steps += 1;
            if ready {
                break;
            }
        }
        let mut samples = vec![0.; 257 * channels];
        let b = stream.read(&mut samples).map_err(|e| format!("{e:?}"))?;
        let ms = start.elapsed().as_secs_f64() * 1000.;
        let mut file = File::create_new(output.join(format!("seek-{}.f32", seeks.len())))?;
        for v in &samples[..b.frames * channels] {
            file.write_all(&v.to_le_bytes())?;
        }
        seeks.push(format!(
            "{{\"target\":{target},\"frames\":{},\"eof\":{},\"generation\":{},\"ms\":{ms},\"steps\":{steps},\"max_step_ms\":{}}}",
            b.frames, b.eof, b.generation, max_step.as_secs_f64() * 1000.
        ));
    }
    let stats = stream.stats();
    let result = format!(
        "{{\"schema_version\":1,\"rate\":{},\"channels\":{channels},\"frames\":{total},\"open_ms\":{open_ms},\"decode_ms\":{decode_ms},\"blocks\":{blocks},\"packets\":{},\"largest_packet_frames\":{},\"seeks\":[{}],\"audio_device_opened\":false}}\n",
        stream.rate(),
        stats.packets,
        stats.largest_packet_frames,
        seeks.join(",")
    );
    std::fs::write(output.join("result.json"), result)?;
    // Leave a post-decode sample window; no decoded file is retained in memory.
    std::thread::sleep(Duration::from_millis(350));
    Ok(())
}
#[cfg(not(feature = "compressed"))]
fn main() {
    eprintln!("Enable the optional compressed feature");
    std::process::exit(1);
}
