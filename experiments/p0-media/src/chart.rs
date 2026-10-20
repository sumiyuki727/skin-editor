//! Read-only v14 circle fixture and audio-position timeline, without hit judgement.
pub const MAX_TEXT_BYTES: usize = 64 * 1024;
pub const MAX_OBJECTS: usize = 1024;
pub const MAX_TIME_US: u64 = 6_000_000;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    Limit,
    Syntax,
    Unsupported,
    Order,
    Backwards,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Error {
    pub line: usize,
    pub kind: ErrorKind,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObjectId(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Circle {
    pub id: ObjectId,
    pub position: [i32; 2],
    pub time_us: u64,
}
#[derive(Clone, Debug)]
pub struct Chart {
    audio_file: String,
    objects: Vec<Circle>,
}
impl Chart {
    pub fn audio_file(&self) -> &str {
        &self.audio_file
    }
    pub fn objects(&self) -> &[Circle] {
        &self.objects
    }
}
fn failure(line: usize, kind: ErrorKind) -> Error {
    Error { line, kind }
}
pub fn parse(text: &str) -> Result<Chart, Error> {
    if text.len() > MAX_TEXT_BYTES {
        return Err(failure(0, ErrorKind::Limit));
    }
    let mut lines = text.trim_start_matches('\u{feff}').lines().enumerate();
    if lines.next().map(|(_, l)| l.trim()) != Some("osu file format v14") {
        return Err(failure(1, ErrorKind::Unsupported));
    }
    let mut section = "";
    let mut audio = None;
    let mut mode = false;
    let mut lead = false;
    let mut seen_objects = false;
    let mut general = false;
    let mut objects = Vec::new();
    for (i, line) in lines {
        let line_no = i + 1;
        let line = line.trim();
        let invalid = |kind| failure(line_no, kind);
        if line.is_empty() || line.starts_with("//") {
            continue;
        }
        if line.starts_with('[') {
            match line {
                "[General]" if !general => {
                    general = true;
                    section = "general";
                }
                "[HitObjects]" if !seen_objects => {
                    seen_objects = true;
                    section = "objects";
                }
                _ => return Err(invalid(ErrorKind::Unsupported)),
            }
            continue;
        }
        match section {
            "general" => {
                let (k, v) = line.split_once(':').ok_or(invalid(ErrorKind::Syntax))?;
                let v = v.trim();
                match k.trim() {
                    "AudioFilename" if audio.is_none() => {
                        if v.is_empty()
                            || v == "."
                            || v == ".."
                            || v.chars()
                                .any(|c| c.is_control() || "/\\:*?\"<>|".contains(c))
                        {
                            return Err(invalid(ErrorKind::Unsupported));
                        }
                        audio = Some(v.to_owned());
                    }
                    "Mode" if !mode && v == "0" => {
                        mode = true;
                    }
                    "AudioLeadIn" if !lead && v == "0" => {
                        lead = true;
                    }
                    _ => return Err(invalid(ErrorKind::Unsupported)),
                }
            }
            "objects" => {
                if objects.len() >= MAX_OBJECTS {
                    return Err(invalid(ErrorKind::Limit));
                }
                let f: Vec<_> = line.split(',').collect();
                if !(5..=6).contains(&f.len()) {
                    return Err(invalid(ErrorKind::Unsupported));
                }
                let number = |s: &str| {
                    s.trim()
                        .parse::<i64>()
                        .map_err(|_| invalid(ErrorKind::Syntax))
                };
                let x = i32::try_from(number(f[0])?).map_err(|_| invalid(ErrorKind::Limit))?;
                let y = i32::try_from(number(f[1])?).map_err(|_| invalid(ErrorKind::Limit))?;
                let t = u64::try_from(number(f[2])?)
                    .ok()
                    .and_then(|v| v.checked_mul(1000))
                    .filter(|t| *t <= MAX_TIME_US)
                    .ok_or(invalid(ErrorKind::Limit))?;
                if number(f[3])? != 1
                    || number(f[4])? != 0
                    || (f.len() == 6 && f[5].trim() != "0:0:0:0:")
                {
                    return Err(invalid(ErrorKind::Unsupported));
                }
                if objects.last().is_some_and(|o: &Circle| o.time_us > t) {
                    return Err(invalid(ErrorKind::Order));
                }
                objects.push(Circle {
                    id: ObjectId(objects.len() as u32 + 1),
                    position: [x, y],
                    time_us: t,
                });
            }
            _ => return Err(invalid(ErrorKind::Syntax)),
        }
    }
    if !general || !mode || !seen_objects || objects.is_empty() {
        return Err(failure(0, ErrorKind::Syntax));
    }
    Ok(Chart {
        audio_file: audio.ok_or(failure(0, ErrorKind::Syntax))?,
        objects,
    })
}

/// These windows are fixture parameters, never AR or hit-window compatibility rules.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Window {
    pub before_us: u64,
    pub after_us: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    Pending,
    Visible,
    Past,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Frame {
    pub position_us: u64,
    pub objects: Vec<(ObjectId, Stage)>,
}
pub struct Timeline {
    chart: Chart,
    window: Window,
    position: u64,
    next: usize,
}
impl Timeline {
    pub fn new(chart: Chart, window: Window) -> Result<Self, Error> {
        if window.before_us > MAX_TIME_US || window.after_us > MAX_TIME_US {
            return Err(failure(0, ErrorKind::Limit));
        }
        Ok(Self {
            chart,
            window,
            position: 0,
            next: 0,
        })
    }
    pub fn frame(&self) -> Frame {
        Frame {
            position_us: self.position,
            objects: self
                .chart
                .objects
                .iter()
                .map(|o| {
                    let stage = if self.position < o.time_us.saturating_sub(self.window.before_us) {
                        Stage::Pending
                    } else if self.position > o.time_us + self.window.after_us {
                        Stage::Past
                    } else {
                        Stage::Visible
                    };
                    (o.id, stage)
                })
                .collect(),
        }
    }
    pub fn advance(&mut self, position: u64) -> Result<Vec<ObjectId>, Error> {
        if position < self.position {
            return Err(failure(0, ErrorKind::Backwards));
        }
        if position > MAX_TIME_US {
            return Err(failure(0, ErrorKind::Limit));
        }
        let end = self
            .chart
            .objects
            .partition_point(|o| o.time_us <= position);
        let events = self.chart.objects[self.next..end]
            .iter()
            .map(|o| o.id)
            .collect();
        self.position = position;
        self.next = end;
        Ok(events)
    }
    /// Explicit content relocation resets evaluation; skipped events are not replayed.
    pub fn seek(&mut self, position: u64) -> Result<(), Error> {
        if position > MAX_TIME_US {
            return Err(failure(0, ErrorKind::Limit));
        }
        self.position = position;
        self.next = self
            .chart
            .objects
            .partition_point(|o| o.time_us <= position);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn source(objects: &str) -> String {
        format!(
            "osu file format v14\n[General]\nAudioFilename: 音频 日本語.wav\nMode: 0\nAudioLeadIn: 0\n[HitObjects]\n{objects}"
        )
    }
    #[test]
    fn public_v14_fields_keep_unicode_names_object_identity_and_equal_timestamps() {
        let c = parse(&source(
            "40,50,250,1,0\n60,70,500,1,0,0:0:0:0:\n80,90,500,1,0",
        ))
        .unwrap();
        assert_eq!(c.audio_file(), "音频 日本語.wav");
        assert_eq!(
            c.objects()[0],
            Circle {
                id: ObjectId(1),
                position: [40, 50],
                time_us: 250000
            }
        );
        assert_ne!(c.objects()[1].id, c.objects()[2].id);
    }
    #[test]
    fn invalid_and_unsupported_input_fails_without_silently_discarding_content() {
        for s in [
            source("0,0,500,1,0\n0,0,250,1,0"),
            source("0,0,0,2,0,B|40:50,1,20"),
            source("0,0,0,1,2"),
            source("0,0,-1,1,0"),
            source("0,0,7000,1,0"),
            source("0,0,250,1,0").replace("Mode: 0", "Mode: 1"),
            source("0,0,250,1,0").replace("AudioLeadIn: 0", "AudioLeadIn: 2"),
            source("0,0,250,1,0").replace("Mode: 0", "Mode: 0\nUnknown: 7"),
            source("0,0,250,1,0").replace("音频 日本語.wav", "../foreign.wav"),
            source("0,0,250,1,0").replace("v14", "v128"),
        ] {
            assert!(parse(&s).is_err(), "{s}");
        }
        assert!(parse(&"x".repeat(MAX_TEXT_BYTES + 1)).is_err());
        assert!(parse(&source(&"0,0,250,1,0\n".repeat(MAX_OBJECTS + 1))).is_err());
    }
    #[test]
    fn device_positions_emit_once_pause_is_idempotent_and_seek_does_not_replay_skipped_events() {
        let c = parse(&source(
            "40,50,250,1,0\n60,70,500,1,0\n80,90,500,1,0\n20,40,900,1,0",
        ))
        .unwrap();
        let mut t = Timeline::new(
            c,
            Window {
                before_us: 100000,
                after_us: 50000,
            },
        )
        .unwrap();
        assert_eq!(t.frame().objects[0].1, Stage::Pending);
        assert!(t.advance(150000).unwrap().is_empty());
        assert_eq!(t.frame().objects[0].1, Stage::Visible);
        assert_eq!(
            t.advance(500000).unwrap(),
            vec![ObjectId(1), ObjectId(2), ObjectId(3)]
        );
        let frozen = t.frame();
        for _ in 0..100 {
            assert!(t.advance(500000).unwrap().is_empty());
            assert_eq!(t.frame(), frozen);
        }
        assert!(t.advance(499999).is_err());
        assert_eq!(t.frame(), frozen);
        t.seek(950000).unwrap();
        assert!(t.advance(960000).unwrap().is_empty());
        t.seek(100000).unwrap();
        assert_eq!(
            t.advance(500000).unwrap(),
            vec![ObjectId(1), ObjectId(2), ObjectId(3)]
        );
        let old = t.frame();
        assert!(t.seek(MAX_TIME_US + 1).is_err());
        assert_eq!(t.frame(), old);
    }
}
