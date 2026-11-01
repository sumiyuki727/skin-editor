//! Opt-in, bounded parent/child checkpoints. Never enabled by the editor.
use eframe::egui_wgpu::wgpu;
use std::{
    io::Write,
    path::Path,
    time::{Duration, Instant},
};

fn ack_matches(bytes: &[u8], sequence: u32) -> bool {
    bytes == sequence.to_string().as_bytes()
}

pub fn run(destination: &str, memory_usage: bool) -> Result<(), String> {
    let root = Path::new(destination);
    if !root.is_absolute() {
        return Err("absolute fresh diagnostic directory required".into());
    }
    std::fs::create_dir(root).map_err(|e| e.to_string())?;
    let mut sequence = 0;
    let mut checkpoint = |stage: &str, device: Option<&wgpu::Device>| {
        sequence += 1;
        if sequence > 40 {
            return Err("checkpoint bound exceeded".into());
        }
        let gpu = match device {
            Some(d) => crate::gpu_observation::snapshot(d, stage)?,
            None => "null".into(),
        };
        // Parent acknowledges only after collecting metadata from its own child.
        // Unique per-sequence files avoid partial overwrite and stale reuse.
        let line =
            format!("{{\"checkpoint\":{sequence},\"stage\":\"{stage}\",\"allocator\":{gpu}}}");
        println!("{line}");
        std::io::stdout().flush().map_err(|e| e.to_string())?;
        let ack = root.join(format!("ack-{sequence}"));
        let deadline = Instant::now() + Duration::from_secs(20);
        loop {
            match std::fs::read(&ack) {
                Ok(bytes) if ack_matches(&bytes, sequence) => break,
                Ok(_) => return Err("wrong checkpoint acknowledgement".into()),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(e.to_string()),
            }
            if Instant::now() >= deadline {
                return Err("checkpoint acknowledgement timed out".into());
            }
            std::thread::sleep(Duration::from_millis(1)); // bounded IPC polling, not a race oracle
        }
        Ok(())
    };
    crate::graphics_faults::run_observed(
        root.join("native").to_str().ok_or("invalid output path")?,
        memory_usage,
        &mut checkpoint,
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn checkpoint_ack_is_exact_and_cannot_replay_another_sequence() {
        assert!(super::ack_matches(b"3", 3));
        for bytes in [b"2".as_slice(), b"03", b"3\n", b"", b"4"] {
            assert!(!super::ack_matches(bytes, 3));
        }
    }
    #[test]
    fn stages_require_fresh_absolute_destination() {
        assert!(super::run("relative-stages", false).is_err());
        let root = std::env::current_dir().unwrap();
        assert!(super::run(root.to_str().unwrap(), true).is_err());
    }
}
