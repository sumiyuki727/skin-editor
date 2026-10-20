//! Opt-in file PCM and device-clock timeline proof; no window, judgement or project I/O.
#[cfg(windows)]
mod native {
    use p0_egui::{
        audio_output::Output,
        audio_transport::{Phase, Transport},
        windows_audio::WindowsOutput,
    };
    use p0_media::{
        audio,
        chart::{self, Frame, Timeline, Window},
    };
    use p0_wasapi::{
        Snapshot,
        pcm::{Clip, Packet, Stats},
    };
    use serde_json::{Value, json};
    use std::{
        collections::BTreeMap,
        fs::{self, File, OpenOptions},
        io::{Read, Write},
        path::{Path, PathBuf},
        sync::Arc,
        thread,
        time::{Duration, Instant},
    };

    fn load(root: &Path, name: &str, limit: usize) -> Result<Vec<u8>, String> {
        let path = root.join(name).canonicalize().map_err(|e| e.to_string())?;
        if path.parent() != Some(root) {
            return Err("Fixture resource escaped its directory".into());
        }
        let mut bytes = Vec::new();
        File::open(path)
            .map_err(|e| e.to_string())?
            .take(limit as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        if bytes.len() > limit {
            return Err("Fixture byte limit exceeded".into());
        }
        Ok(bytes)
    }
    fn packet(p: Option<Packet>) -> Value {
        p.map(|p|json!({"source_frame":p.source_frame,"frames":p.frames,"peak":p.peak,"hash":p.hash,"nonzero_samples":p.nonzero_samples,"probe":p.probe})).unwrap_or(Value::Null)
    }
    fn stats(s: Stats) -> Value {
        json!({"requested_us":s.requested_us,"source_origin_frame":s.source_origin_frame,"submitted_frames":s.submitted_frames,"packets":s.packets,"first":packet(s.first),"last":packet(s.last)})
    }
    fn frame(f: Frame) -> Value {
        json!({"position_us":f.position_us,"objects":f.objects.iter().map(|(id,s)|json!({"id":id.0,"stage":format!("{s:?}")})).collect::<Vec<_>>()})
    }
    struct Session {
        output: WindowsOutput,
        transport: Transport,
        timeline: Timeline,
        epoch: u64,
        last: Option<Snapshot>,
        samples: Vec<Value>,
        events: BTreeMap<u64, Vec<u32>>,
        pauses: Vec<Value>,
        checkpoints: Vec<Value>,
    }
    impl Session {
        fn poll(&mut self) -> Result<bool, String> {
            let s = self.output.0.snapshot();
            if self.last.is_some_and(|p| p.sequence == s.sequence) {
                return Ok(false);
            }
            let Some(o) = WindowsOutput::observation(s) else {
                return Ok(false);
            };
            if !self.transport.observe(o) {
                return Ok(false);
            }
            let state = self.transport.state();
            if state.phase == Phase::Faulted {
                return Err(format!(
                    "Native output fault: {:?} {:?}",
                    state.fault, s.error
                ));
            }
            let pcm = s.pcm.ok_or("Missing file PCM statistics")?;
            if state.simulation_us != pcm.requested_us + s.position_us {
                return Err("Timeline clock is not the device content position".into());
            }
            if state.epoch != self.epoch {
                self.timeline
                    .seek(state.simulation_us)
                    .map_err(|e| format!("{e:?}"))?;
                self.epoch = state.epoch;
            }
            let due = self
                .timeline
                .advance(state.simulation_us)
                .map_err(|e| format!("{e:?}"))?;
            self.events
                .entry(state.epoch)
                .or_default()
                .extend(due.iter().map(|id| id.0));
            if self.samples.len() >= 2048 {
                return Err("Native sample trace limit exceeded".into());
            }
            self.samples.push(json!({"epoch":state.epoch,"request":state.request,"sequence":s.sequence,"phase":format!("{:?}",state.phase),"device_us":s.position_us,"simulation_us":state.simulation_us,"pcm":stats(pcm)}));
            self.last = Some(s);
            Ok(true)
        }
        fn wait(&mut self, condition: impl Fn(&Transport) -> bool) -> Result<(), String> {
            let until = Instant::now() + Duration::from_secs(5);
            loop {
                self.poll()?;
                if condition(&self.transport) {
                    return Ok(());
                }
                if Instant::now() > until {
                    return Err("Output acknowledgement/progress timeout".into());
                }
                thread::sleep(Duration::from_millis(10));
            }
        }
        fn control(&mut self, paused: bool) -> Result<(), String> {
            let i = self.transport.intent(paused);
            self.output.control(i).map_err(|e| format!("{e:?}"))?;
            self.wait(|t| {
                t.state().phase
                    == if paused {
                        Phase::Paused
                    } else {
                        Phase::Playing
                    }
            })
        }
        fn freeze(&mut self) -> Result<(), String> {
            self.control(true)?;
            let before = self.timeline.frame();
            let sample = self.last.unwrap();
            let pcm = sample.pcm.unwrap();
            let start = Instant::now();
            let mut fresh = 0;
            while start.elapsed() < Duration::from_millis(350) {
                if self.poll()? {
                    fresh += 1;
                    let now = self.last.unwrap();
                    if now.position_us != sample.position_us
                        || now.pcm.unwrap().submitted_frames != pcm.submitted_frames
                        || self.timeline.frame() != before
                    {
                        return Err(
                            "Pause changed device position, PCM submission or chart frame".into(),
                        );
                    }
                }
                thread::sleep(Duration::from_millis(10));
            }
            if fresh < 3 {
                return Err("Insufficient fresh paused device samples".into());
            }
            self.pauses.push(json!({"epoch":self.epoch,"position_us":before.position_us,"device_us":sample.position_us,"fresh_samples":fresh,"wall_ms":start.elapsed().as_secs_f64()*1000.,"frame":frame(before)}));
            Ok(())
        }
        fn seek(&mut self, position: u64) -> Result<(), String> {
            let old = self.timeline.frame();
            let target = self
                .transport
                .seek(position)
                .ok_or("Seek requires confirmed pause")?;
            self.output.relocate(target).map_err(|e| format!("{e:?}"))?;
            if self.timeline.frame() != old {
                return Err("Timeline changed before content relocation acknowledgement".into());
            }
            self.wait(|t| t.state().phase == Phase::Paused)?;
            if self.transport.state().simulation_us != position {
                return Err("Relocation acknowledgement has wrong content position".into());
            }
            let s = self.last.unwrap();
            if s.pcm.unwrap().submitted_frames != 0 {
                return Err("Relocation submitted PCM before explicit play".into());
            }
            self.checkpoints.push(json!({"epoch":self.epoch,"position_us":position,"frame":frame(self.timeline.frame()),"pcm":stats(s.pcm.unwrap())}));
            Ok(())
        }
    }
    fn execute(decoded: audio::Clip, chart: chart::Chart) -> Result<Value, String> {
        let frames = decoded.frames();
        let duration = decoded.duration_us();
        let rate = decoded.rate();
        let channels = decoded.channels();
        if chart.objects().iter().any(|o| o.time_us > duration) {
            return Err("Chart event lies beyond audio content".into());
        }
        let clip = Arc::new(
            Clip::new(rate, channels, decoded.into_samples(), 0.04)
                .map_err(|e| format!("{e:?}"))?,
        );
        let output = WindowsOutput::spawn_clip(clip, || {}).map_err(|e| format!("{e:?}"))?;
        let mut s = Session {
            output,
            transport: Transport::default(),
            timeline: Timeline::new(
                chart,
                Window {
                    before_us: 150000,
                    after_us: 100000,
                },
            )
            .map_err(|e| format!("{e:?}"))?,
            epoch: 0,
            last: None,
            samples: vec![],
            events: BTreeMap::new(),
            pauses: vec![],
            checkpoints: vec![],
        };
        s.wait(|t| t.state().phase == Phase::Paused)?;
        if s.output.0.relocate(999, duration + 1).is_ok() {
            return Err("Out-of-content relocation was accepted".into());
        }
        s.transport.tool(true);
        s.control(false)?;
        if !s.transport.key(0, true, false, false) {
            return Err("Playing input was rejected".into());
        }
        s.wait(|t| t.state().simulation_us >= 600000)?;
        s.freeze()?;
        if s.transport.key(1, true, false, false) {
            return Err("Paused input leaked".into());
        }
        s.seek(1_100_000)?;
        s.control(false)?;
        s.wait(|t| t.state().simulation_us >= 1_600_000)?;
        s.freeze()?;
        s.seek(100000)?;
        s.control(false)?;
        if s.transport.key(1, true, true, false) {
            return Err("Held/repeat input replayed after relocation".into());
        }
        s.transport.key(1, false, false, false);
        if !s.transport.key(1, true, false, false) {
            return Err("Fresh input after relocation was rejected".into());
        }
        s.wait(|t| t.state().simulation_us >= 600000)?;
        s.freeze()?;
        let expected = BTreeMap::from([(0, vec![1, 2, 3]), (1, vec![5]), (2, vec![1, 2, 3])]);
        if s.events != expected {
            return Err(format!(
                "Unexpected timeline event crossings: {:?}",
                s.events
            ));
        }
        let format = s.last.unwrap().format.unwrap();
        Ok(
            json!({"schema_version":1,"status":"passed","source":"self_authored_wav_and_v14_circle_fixture","system_capture_used":false,"microphone_or_loopback_used":false,
        "decoded":{"rate":rate,"channels":channels,"frames":frames,"duration_us":duration,"payload_bytes":frames*usize::from(channels)*4,"gain":0.04},
        "audio_format":{"rate":format.rate,"channels":format.channels,"bits":format.bits,"buffer_frames":format.buffer_frames},"accepted_inputs":s.transport.state().accepted_inputs,
        "event_crossings":s.events,"fresh_samples":s.samples,"paused_checks":s.pauses,"relocations":s.checkpoints,"final_frame":frame(s.timeline.frame()),
        "checks":{"device_content_clock":true,"paused_timeline_and_pcm_frozen":true,"forward_seek_skips_events":true,"backward_seek_re_evaluates":true,"relocation_waits_for_ack":true,"pause_and_held_input_isolation":true,"out_of_content_seek_rejected":true},
        "limits":["WAV/PCM only; no compressed codec, streaming, resampling, hit judgement or skin rules","Timeline windows are fixture parameters, not stable AR or hit windows","Whole short clip is bounded and decoded before opening output; no UI-thread decoder","Native input here is scripted, not physical input-to-sound latency","Device acceptance is not acoustic verification or actual stable gameplay"]}),
        )
    }
    pub fn run() -> Result<(), String> {
        let args: Vec<_> = std::env::args().skip(1).collect();
        if args.len() != 2 {
            return Err(
                "Use new artifacts/p0 output and self-authored media-fixture directory".into(),
            );
        }
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("artifacts/p0")
            .canonicalize()
            .map_err(|e| e.to_string())?;
        let output = PathBuf::from(&args[0]);
        if output.exists()
            || output
                .parent()
                .ok_or("Output parent missing")?
                .canonicalize()
                .map_err(|e| e.to_string())?
                != root
        {
            return Err("Output must be a new direct artifacts/p0 child".into());
        }
        let fixture = PathBuf::from(&args[1])
            .canonicalize()
            .map_err(|e| e.to_string())?;
        if fixture.parent() != Some(root.as_path())
            || !fixture
                .file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("media-fixture-")
        {
            return Err("Only the generated P0 media fixture may be read".into());
        }
        let bytes = load(&fixture, "timeline.osu", chart::MAX_TEXT_BYTES)?;
        let text = std::str::from_utf8(&bytes).map_err(|e| e.to_string())?;
        let map = chart::parse(text).map_err(|e| format!("Chart: {e:?}"))?;
        let audio = load(&fixture, map.audio_file(), audio::MAX_ENCODED_BYTES)?;
        let start = Instant::now();
        let decoded = audio::decode_wav(audio).map_err(|e| format!("Decode: {e:?}"))?;
        let decode_ms = start.elapsed().as_secs_f64() * 1000.;
        fs::create_dir(&output).map_err(|e| e.to_string())?;
        let result = execute(decoded, map);
        let mut json = match &result {
            Ok(v) => v.clone(),
            Err(e) => json!({"schema_version":1,"status":"failed","error":e}),
        };
        json["decode_ms"] = json!(decode_ms);
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(output.join("result.json"))
            .map_err(|e| e.to_string())?;
        file.write_all(&serde_json::to_vec_pretty(&json).unwrap())
            .map_err(|e| e.to_string())?;
        result.map(|_| ())
    }
    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn fixture_reader_is_bounded_rejects_missing_or_escaped_resources_and_keeps_inputs() {
            let base = std::env::temp_dir().join(format!("p0-media-reader-{}", std::process::id()));
            fs::create_dir(&base).unwrap();
            let root = base.join("fixture");
            fs::create_dir(&root).unwrap();
            let root = root.canonicalize().unwrap();
            let input = root.join("sample.wav");
            fs::write(&input, b"owned source bytes").unwrap();
            let outside = base.join("outside.wav");
            fs::write(&outside, b"outside fixture sentinel").unwrap();
            assert_eq!(
                load(&root, "sample.wav", 64).unwrap(),
                b"owned source bytes"
            );
            assert!(load(&root, "missing.wav", 64).is_err());
            assert!(load(&root, "sample.wav", 4).is_err());
            assert!(load(&root, "../outside.wav", 64).is_err());
            assert_eq!(fs::read(&input).unwrap(), b"owned source bytes");
            assert_eq!(fs::read(&outside).unwrap(), b"outside fixture sentinel");
            fs::remove_file(input).unwrap();
            fs::remove_file(outside).unwrap();
            fs::remove_dir(root).unwrap();
            fs::remove_dir(base).unwrap();
        }
    }
}
fn main() {
    #[cfg(windows)]
    if let Err(e) = native::run() {
        eprintln!("Media proof: {e}");
        std::process::exit(1);
    }
    #[cfg(not(windows))]
    {
        eprintln!("Native file output proof requires Windows");
        std::process::exit(1);
    }
}
