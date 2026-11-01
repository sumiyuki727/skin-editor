//! Sequential, device-free protocol model, NOT a concurrent/native implementation.
//! Continuous PCM has a reserve; ready short voices are mixed at consumption.
//! Owner and callback steps are explicit: tests choose the interleaving/time.
use crate::{BackgroundTrack, Channel, MAX_VOICES, Origin, PcmBudget, PcmBuffer, PcmReservation};

const RESERVE: usize = 2048;
const PERIOD: usize = 480;
const RATE: u64 = 48_000;

fn lane(channel: Channel) -> usize {
    match channel {
        Channel::Scene => 0,
        Channel::Audition => 1,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Token {
    device: u64,
    content: u64,
    lane: u64,
    channel: Channel,
}
struct Gates {
    device: u64,
    content: u64,
    lanes: [u64; 2],
    running: bool,
    closed: bool,
    background: u64,
}
impl Gates {
    fn token(&self, channel: Channel) -> Token {
        Token {
            device: self.device,
            content: self.content,
            lane: self.lanes[lane(channel)],
            channel,
        }
    }
    fn permits(&self, token: Token) -> bool {
        !self.closed
            && token == self.token(token.channel)
            && (token.channel == Channel::Audition || self.running)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Key {
    slot: usize,
    generation: u64,
}
struct Voice {
    key: Key,
    token: Token,
    pcm: PcmBuffer,
    cursor: usize,
    started: bool,
}
struct Pin {
    key: Key,
    pcm: PcmBuffer,
}
// Validation happens during preparation, not each timed ready admission.
// Only prepared48kHz stereo is modeled; no format negotiation is implemented.
struct Ready(PcmBuffer);
impl Ready {
    fn new(pcm: PcmBuffer) -> Result<Self, Refusal> {
        if !pcm.samples().len().is_multiple_of(2)
            || pcm
                .samples()
                .iter()
                .any(|v| !v.is_finite() || v.abs() > 1.0)
        {
            return Err(Refusal::InvalidPcm);
        }
        Ok(Self(pcm))
    }
}
#[derive(Clone, Copy, Default)]
struct Frame {
    position: u64,
    samples: [f32; 2],
}
struct Reserve {
    frames: [Frame; RESERVE],
    read: usize,
    len: usize,
}
impl Reserve {
    fn push(&mut self, frame: Frame) {
        assert!(self.len < RESERVE);
        self.frames[(self.read + self.len) % RESERVE] = frame;
        self.len += 1;
    }
    fn pop(&mut self) -> Option<Frame> {
        if self.len == 0 {
            return None;
        }
        let frame = self.frames[self.read];
        self.read = (self.read + 1) % RESERVE;
        self.len -= 1;
        Some(frame)
    }
    fn clear(&mut self) {
        self.read = 0;
        self.len = 0;
    }
}

// Fixed slot credits cover pending + active + retired, not just active voices.
// The owner keeps a separate PCM pin until the retired descriptor returns.
// Coalesced gates and pause acknowledgements consume no voice/receipt credit.
struct Bridge {
    gates: Gates,
    pending: [Option<Voice>; MAX_VOICES],
    retired: [Option<Voice>; MAX_VOICES],
    reserve: Reserve,
    pause_ack: Option<(u64, u64)>, // background epoch, next submitted source frame
}
struct Owner {
    budget: PcmBudget,
    _reserve_payload: PcmReservation,
    source: Origin,
    serial: u64,
    pins: [Option<Pin>; MAX_VOICES],
    background: BackgroundTrack,
    background_epoch: u64,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Refusal {
    Busy,
    Stale,
    Frozen,
    Closed,
    WrongPool,
    InvalidPcm,
    AwaitingPauseAck,
    NotPrimed,
}
impl Owner {
    fn admit(
        &mut self,
        bridge: &mut Bridge,
        source: Origin,
        token: Token,
        ready: &Ready,
    ) -> Result<Key, Refusal> {
        let pcm = &ready.0;
        if bridge.gates.closed {
            return Err(Refusal::Closed);
        }
        if source != self.source || token != bridge.gates.token(token.channel) {
            return Err(Refusal::Stale);
        }
        if token.channel == Channel::Scene && !bridge.gates.running {
            return Err(Refusal::Frozen);
        }
        if !self.budget.same_pool(pcm.budget()) {
            return Err(Refusal::WrongPool);
        }
        let slot = self
            .pins
            .iter()
            .position(Option::is_none)
            .ok_or(Refusal::Busy)?;
        self.serial = self.serial.checked_add(1).expect("bounded experiment");
        let key = Key {
            slot,
            generation: self.serial,
        };
        self.pins[slot] = Some(Pin {
            key,
            pcm: pcm.clone(),
        });
        assert!(bridge.pending[slot].is_none() && bridge.retired[slot].is_none());
        bridge.pending[slot] = Some(Voice {
            key,
            token,
            pcm: pcm.clone(),
            cursor: 0,
            started: false,
        });
        Ok(key)
    }
    fn collect(&mut self, bridge: &mut Bridge) -> usize {
        let mut count = 0;
        for (slot, returned) in bridge.retired.iter_mut().enumerate() {
            if let Some(voice) = returned.take() {
                let pin = self.pins[slot].take().expect("credit remains pinned");
                assert_eq!(pin.key, voice.key);
                assert!(pin.pcm.same_allocation(&voice.pcm));
                // Both references, including the possible last owner, drop here.
                drop(voice);
                drop(pin);
                count += 1;
            }
        }
        count
    }
    fn refill(&mut self, bridge: &mut Bridge) -> Result<usize, Refusal> {
        if bridge.gates.closed {
            return Err(Refusal::Closed);
        }
        if self.background_epoch != bridge.gates.background {
            return Err(Refusal::AwaitingPauseAck);
        }
        let mut count = 0;
        while bridge.reserve.len < RESERVE {
            let Some((position, samples)) = self.background.next_frame() else {
                break;
            };
            bridge.reserve.push(Frame { position, samples });
            count += 1;
        }
        Ok(count)
    }
    fn reconcile_pause(&mut self, bridge: &Bridge) -> Result<(), Refusal> {
        if self.background_epoch == bridge.gates.background {
            return Err(Refusal::Stale);
        }
        let (epoch, next) = bridge.pause_ack.ok_or(Refusal::AwaitingPauseAck)?;
        if epoch != bridge.gates.background {
            return Err(Refusal::AwaitingPauseAck);
        }
        self.background.restore_position(next).unwrap();
        self.background_epoch = epoch;
        Ok(())
    }
    fn invalidate_source(&mut self, bridge: &mut Bridge, source: Origin) {
        self.source = source;
        bridge.gates.content += 1;
        // Deliberately does not alter background epoch, reserve or frontier.
    }
}

#[derive(Default)]
struct Report {
    first: [Option<Key>; MAX_VOICES],
    first_count: usize,
    missing: usize,
    background_frames: usize,
    voice_sample_adds: usize,
}
struct Callback {
    active: [Option<Voice>; MAX_VOICES],
    background_epoch: u64,
    next_background: u64,
}
impl Callback {
    // No owner/project/session reference; no Vec, clone, final PCM drop or waits.
    // Sequential mailbox access here is NOT a claim of atomic/SPSC correctness.
    fn output(&mut self, bridge: &mut Bridge, output: &mut [[f32; 2]]) -> Report {
        let mut report = Report::default();
        if self.background_epoch != bridge.gates.background {
            assert!(!bridge.gates.running);
            bridge.reserve.clear(); // Copy data only; this content was not submitted.
            self.background_epoch = bridge.gates.background;
            bridge.pause_ack = Some((self.background_epoch, self.next_background));
        }
        for slot in 0..MAX_VOICES {
            if let Some(voice) = bridge.pending[slot].take() {
                assert!(self.active[slot].is_none());
                self.active[slot] = Some(voice);
            }
            if self.active[slot]
                .as_ref()
                .is_some_and(|v| !bridge.gates.permits(v.token))
            {
                self.retire(bridge, slot);
            }
        }
        for frame in output {
            *frame = [0.0; 2];
            if bridge.gates.running && !bridge.gates.closed {
                if let Some(background) = bridge.reserve.pop() {
                    assert_eq!(background.position, self.next_background, "no skip/replay");
                    *frame = background.samples;
                    self.next_background += 1;
                    report.background_frames += 1;
                } else {
                    report.missing += 1; // Never relabel underrun as intentional silence.
                }
            }
            for slot in 0..MAX_VOICES {
                let Some(voice) = self.active[slot].as_mut() else {
                    continue;
                };
                if !voice.started {
                    report.first[report.first_count] = Some(voice.key);
                    report.first_count += 1;
                    voice.started = true;
                }
                for (channel, sample) in frame.iter_mut().enumerate() {
                    *sample += voice.pcm.samples()[voice.cursor + channel];
                    report.voice_sample_adds += 1;
                }
                voice.cursor += 2;
                if voice.cursor == voice.pcm.samples().len() {
                    self.retire(bridge, slot);
                }
            }
            for sample in frame {
                *sample = sample.clamp(-1.0, 1.0);
            }
        }
        report
    }
    fn retire(&mut self, bridge: &mut Bridge, slot: usize) {
        assert!(bridge.retired[slot].is_none());
        // Move, never drop: the owner has not yet returned this slot's credit.
        bridge.retired[slot] = self.active[slot].take();
    }
}

struct Model {
    owner: Owner,
    bridge: Bridge,
    callback: Callback,
}
impl Model {
    fn new() -> Self {
        let budget = PcmBudget::default();
        // Reserve Copy sample payload before constructing inline storage. Ring
        // position metadata and handoff tables are accounted separately.
        let reserve_payload = budget.reserve(RESERVE * 2).unwrap();
        let mut background = budget.reserve(16_384 * 2).unwrap().allocate();
        // Nonconstant reference catches gaps and repeats; binary-exact samples.
        for (frame, samples) in background
            .samples_mut()
            .unwrap()
            .chunks_exact_mut(2)
            .enumerate()
        {
            samples.copy_from_slice(&[(frame % 32) as f32 / 256.0, -(frame as f32 % 32.0) / 256.0]);
        }
        let mut background = BackgroundTrack::new(background, RATE as u32, 2).unwrap();
        background.set_running(true); // Generation is separate from device submission.
        let mut model = Self {
            owner: Owner {
                budget,
                _reserve_payload: reserve_payload,
                source: origin(),
                serial: 0,
                pins: std::array::from_fn(|_| None),
                background,
                background_epoch: 1,
            },
            bridge: Bridge {
                gates: Gates {
                    device: 1,
                    content: 1,
                    lanes: [1; 2],
                    running: false,
                    closed: false,
                    background: 1,
                },
                pending: std::array::from_fn(|_| None),
                retired: std::array::from_fn(|_| None),
                reserve: Reserve {
                    frames: [Frame::default(); RESERVE],
                    read: 0,
                    len: 0,
                },
                pause_ack: None,
            },
            callback: Callback {
                active: std::array::from_fn(|_| None),
                background_epoch: 1,
                next_background: 0,
            },
        };
        model.owner.refill(&mut model.bridge).unwrap();
        model.resume().unwrap();
        model
    }
    fn ready(&self, frames: usize, value: f32) -> Ready {
        let mut pcm = self.owner.budget.reserve(frames * 2).unwrap().allocate();
        pcm.samples_mut().unwrap().fill(value);
        Ready::new(pcm).unwrap()
    }
    fn admit(&mut self, channel: Channel, pcm: &Ready) -> Result<Key, Refusal> {
        let token = self.bridge_token(channel);
        self.owner
            .admit(&mut self.bridge, self.owner.source, token, pcm)
    }
    fn bridge_token(&self, channel: Channel) -> Token {
        self.bridge.gates.token(channel)
    }
    fn output(&mut self, frames: usize) -> (Vec<[f32; 2]>, Report) {
        // Test harness allocates before entering the modeled callback.
        let mut output = vec![[f32::NAN; 2]; frames];
        let before = self.owner.budget.usage();
        let report = self.callback.output(&mut self.bridge, &mut output);
        assert_eq!(
            self.owner.budget.usage(),
            before,
            "callback cannot release PCM credit"
        );
        (output, report)
    }
    fn pause(&mut self) {
        self.bridge.gates.running = false;
        self.bridge.gates.lanes[0] += 1;
        self.bridge.gates.background += 1;
        self.bridge.pause_ack = None;
    }
    fn resume(&mut self) -> Result<(), Refusal> {
        if self.bridge.gates.closed {
            return Err(Refusal::Closed);
        }
        if self.owner.background_epoch != self.bridge.gates.background {
            return Err(Refusal::AwaitingPauseAck);
        }
        if self.bridge.reserve.len < RESERVE {
            return Err(Refusal::NotPrimed);
        }
        // Requests sampled while frozen must not become valid on resume.
        self.bridge.gates.lanes[0] += 1;
        self.bridge.gates.running = true;
        Ok(())
    }
    fn close(&mut self) {
        self.pause();
        self.bridge.gates.device += 1;
        self.bridge.gates.closed = true;
    }
}
fn origin() -> Origin {
    Origin {
        session: 1,
        node: 1,
        revision: 1,
        navigation_revision: 1,
        draft: None,
    }
}

#[test]
fn refill_before_or_after_admission_never_places_ready_voice_behind_reserve() {
    for channel in [Channel::Scene, Channel::Audition] {
        for refill_first in [false, true] {
            let mut m = Model::new();
            m.output(PERIOD);
            let pcm = m.ready(16, 0.25);
            if refill_first {
                m.owner.refill(&mut m.bridge).unwrap();
            }
            let key = m.admit(channel, &pcm).unwrap();
            if !refill_first {
                m.owner.refill(&mut m.bridge).unwrap();
            }
            assert_eq!(m.bridge.reserve.len, RESERVE);
            let (samples, report) = m.output(PERIOD);
            assert_eq!(report.first[0], Some(key));
            assert_eq!(report.first_count, 1);
            assert_eq!(report.missing, 0);
            assert_eq!(samples[0], [0.25; 2]); // background frame480 is exactly0
            assert_eq!(m.callback.next_background, 960);
            assert_eq!(m.owner.collect(&mut m.bridge), 1);
        }
    }
}

#[test]
fn three_burst_callbacks_keep_background_and_existing_voices_without_owner_steps() {
    let mut m = Model::new();
    let pcm = m.ready(2000, 0.25);
    m.admit(Channel::Scene, &pcm).unwrap();
    for callback in 0..3 {
        let (samples, report) = m.output(PERIOD);
        assert_eq!(report.missing, 0);
        for (i, sample) in samples.iter().enumerate() {
            let reference = ((callback * PERIOD + i) % 32) as f32 / 256.0;
            assert_eq!(*sample, [reference + 0.25, -reference + 0.25]);
        }
    }
    assert_eq!(m.bridge.reserve.len, RESERVE - 3 * PERIOD);
    assert_eq!(
        m.callback.active[0].as_ref().unwrap().cursor,
        3 * PERIOD * 2
    );
    m.owner.refill(&mut m.bridge).unwrap();
    m.output(PERIOD);
    assert_eq!(m.callback.next_background, 1920);
}

#[test]
fn finite_reserve_and_premix_counterexamples_remain_failures() {
    // Same demand as the new native416 event. Healthy owner between bursts is
    // irrelevant if it cannot run between these three callback entries.
    let mut old_available = 1024_usize;
    let mut old_missing = 0;
    for _ in 0..3 {
        let taken = old_available.min(PERIOD);
        old_available -= taken;
        old_missing += PERIOD - taken;
    }
    assert_eq!(old_missing, 416);
    // New voice appended behind1280 premixed frames first fits callback3.
    // Entries at10/20/30ms: even zero callback work misses the20ms requirement.
    let premix_first_entry_us = (1280 / PERIOD + 1) * 10_000;
    assert_eq!(premix_first_entry_us, 30_000);
    let mut m = Model::new();
    let (_, report) = m.output(RESERVE + 1);
    assert_eq!(report.missing, 1); // No promise to survive arbitrary stalls/bursts.
    assert_eq!(m.callback.next_background, RESERVE as u64);
}

#[test]
fn pause_ack_restores_submission_anchor_and_audition_remains_independent() {
    let mut m = Model::new();
    let scene = m.ready(1000, 0.25);
    let audition = m.ready(1000, -0.125);
    m.admit(Channel::Scene, &scene).unwrap();
    m.admit(Channel::Audition, &audition).unwrap();
    m.output(128);
    let generated = m.owner.background.position();
    assert_eq!(generated, RESERVE as u64);
    m.pause();
    assert_eq!(m.resume(), Err(Refusal::AwaitingPauseAck));
    assert_eq!(
        m.owner.refill(&mut m.bridge),
        Err(Refusal::AwaitingPauseAck)
    );
    assert_eq!(m.admit(Channel::Scene, &scene), Err(Refusal::Frozen));
    let (samples, report) = m.output(128);
    assert!(samples.iter().all(|s| *s == [-0.125; 2]));
    assert_eq!(report.background_frames, 0);
    assert_eq!(report.missing, 0); // Intentional paused silence, not queue exhaustion.
    assert_eq!(m.callback.next_background, 128);
    m.owner.reconcile_pause(&m.bridge).unwrap();
    assert_eq!(m.owner.background.position(), 128);
    assert_eq!(m.resume(), Err(Refusal::NotPrimed));
    m.owner.refill(&mut m.bridge).unwrap();
    let refilled = m.owner.background.position();
    assert_eq!(m.owner.reconcile_pause(&m.bridge), Err(Refusal::Stale));
    assert_eq!(m.owner.background.position(), refilled);
    m.resume().unwrap();
    m.admit(Channel::Scene, &scene).unwrap();
    let (samples, report) = m.output(32);
    assert_eq!(report.missing, 0);
    for (i, sample) in samples.iter().enumerate() {
        let reference = (i % 32) as f32 / 256.0;
        assert_eq!(*sample, [reference + 0.125, -reference + 0.125]);
    }
    assert_eq!(m.callback.next_background, 160);
    assert_eq!(m.owner.background.position(), generated + 128);
}

#[test]
fn cancelled_pending_and_active_sounds_retire_without_releasing_pcm_in_callback() {
    for active in [false, true] {
        for transition in 0..3 {
            let mut m = Model::new();
            if transition == 0 {
                let mut draft = m.owner.source;
                draft.draft = Some((4, 8));
                m.owner.invalidate_source(&mut m.bridge, draft);
            }
            let pcm = m.ready(1000, 0.25);
            let weak = pcm.0.downgrade();
            m.admit(Channel::Scene, &pcm).unwrap();
            m.admit(Channel::Audition, &pcm).unwrap();
            drop(pcm);
            if active {
                m.output(1);
            }
            let submitted = m.callback.next_background;
            let generated = m.owner.background.position();
            let mut next = m.owner.source;
            match transition {
                0 => next.draft = None, // Cancel a real modeled draft back to its base.
                1 => {
                    next.node += 1;
                    next.revision += 1;
                    next.navigation_revision += 1;
                }
                _ => next.navigation_revision += 1, // Navigate away and back to the same node.
            }
            m.owner.invalidate_source(&mut m.bridge, next);
            let (samples, report) = m.output(1);
            assert_eq!(report.voice_sample_adds, 0);
            let reference = (submitted % 32) as f32 / 256.0;
            assert_eq!(samples[0], [reference, -reference]);
            assert!(weak.is_alive());
            assert_eq!(m.owner.background.position(), generated);
            assert_eq!(m.owner.collect(&mut m.bridge), 2);
            assert!(!weak.is_alive());
            assert_eq!(m.owner.budget.usage().bytes, (16_384 + RESERVE) * 2 * 4);
        }
    }
}

#[test]
fn complete_source_identity_and_each_gate_reject_late_requests() {
    let mut m = Model::new();
    let pcm = m.ready(1, 0.25);
    let token = m.bridge_token(Channel::Scene);
    let base = m.owner.source;
    let mut variants = [base; 5];
    variants[0].session += 1;
    variants[1].node += 1;
    variants[2].revision += 1;
    variants[3].navigation_revision += 1;
    variants[4].draft = Some((1, 1));
    for old in variants {
        assert_eq!(
            m.owner.admit(&mut m.bridge, old, token, &pcm),
            Err(Refusal::Stale)
        );
    }
    for old in [
        Token {
            device: token.device + 1,
            ..token
        },
        Token {
            content: token.content + 1,
            ..token
        },
        Token {
            lane: token.lane + 1,
            ..token
        },
    ] {
        assert_eq!(
            m.owner.admit(&mut m.bridge, base, old, &pcm),
            Err(Refusal::Stale)
        );
    }
    assert!(m.owner.pins.iter().all(Option::is_none));
    // Same node/revision reached by a new navigation still invalidates old work.
    let mut next = base;
    next.navigation_revision += 1;
    m.owner.invalidate_source(&mut m.bridge, next);
    assert_eq!(
        m.owner.admit(&mut m.bridge, base, token, &pcm),
        Err(Refusal::Stale)
    );
}

#[test]
fn exhausted_credits_do_not_block_stop_and_retirement_cannot_overwrite_or_reuse_keys() {
    for phase in 0..3 {
        // All credits pending, active, or awaiting collection.
        let mut m = Model::new();
        let pcm = m.ready(if phase == 2 { 1 } else { 1000 }, 0.25);
        let mut keys = Vec::new();
        for _ in 0..MAX_VOICES {
            keys.push(m.admit(Channel::Audition, &pcm).unwrap());
        }
        assert_eq!(m.admit(Channel::Scene, &pcm), Err(Refusal::Busy));
        if phase != 0 {
            let frames = if phase == 1 { PERIOD } else { 1 };
            let (samples, report) = m.output(frames);
            assert_eq!(samples[0], [1.0; 2]); // Sum32 voices, clip once at final mix.
            assert_eq!(report.voice_sample_adds, MAX_VOICES * 2 * frames);
            assert_eq!(report.first_count, MAX_VOICES);
            assert_eq!(m.admit(Channel::Audition, &pcm), Err(Refusal::Busy));
        }
        // Stop is a gate change, not another enqueued command needing credit.
        m.bridge.gates.lanes[1] += 1;
        let (_, report) = m.output(1);
        assert_eq!(report.voice_sample_adds, 0);
        assert_eq!(m.owner.collect(&mut m.bridge), MAX_VOICES);
        let new = m.admit(Channel::Audition, &pcm).unwrap();
        assert_eq!(new.slot, keys[0].slot);
        assert!(keys.iter().all(|old| *old != new));
        assert_eq!(m.output(1).1.first[0], Some(new));
    }
}

#[test]
fn voice_admitted_inside_burst_starts_next_callback_without_dropping_existing_audio() {
    let mut m = Model::new();
    let old = m.ready(2000, 0.25);
    let new = m.ready(16, -0.125);
    m.admit(Channel::Scene, &old).unwrap();
    m.output(PERIOD);
    let reserve_before = m.bridge.reserve.len;
    let key = m.admit(Channel::Audition, &new).unwrap();
    assert_eq!(m.bridge.reserve.len, reserve_before); // No refill/trim/catch-up.
    let (samples, report) = m.output(PERIOD);
    assert_eq!(report.first[0], Some(key));
    assert_eq!(samples[0], [0.125; 2]);
    assert_eq!(samples[16], [0.3125, 0.1875]);
    assert_eq!(m.output(PERIOD).1.missing, 0);
    assert_eq!(m.callback.next_background, 1440);
}

#[test]
fn stopping_audition_preserves_scene_and_pause_still_admits_a_new_explicit_audition() {
    let mut m = Model::new();
    let pcm = m.ready(1000, 0.25);
    m.admit(Channel::Scene, &pcm).unwrap();
    m.admit(Channel::Audition, &pcm).unwrap();
    m.bridge.gates.lanes[1] += 1;
    let (samples, report) = m.output(1);
    assert_eq!(samples[0], [0.25; 2]);
    assert_eq!(report.first_count, 1);
    m.pause();
    m.admit(Channel::Audition, &pcm).unwrap();
    let (samples, report) = m.output(1);
    assert_eq!(samples[0], [0.25; 2]);
    assert_eq!(report.background_frames, 0);
    assert_eq!(report.first_count, 1);
    assert_eq!(m.callback.next_background, 1);
}

#[test]
fn pause_before_or_after_callback_never_replays_already_submitted_voice_samples() {
    for submit_first in [false, true] {
        let mut m = Model::new();
        let pcm = m.ready(16, 0.25);
        let key = m.admit(Channel::Scene, &pcm).unwrap();
        if submit_first {
            let (samples, report) = m.output(1);
            assert_eq!(samples[0], [0.25; 2]);
            assert_eq!(report.first[0], Some(key));
        }
        m.pause();
        assert!(m.output(32).0.iter().all(|s| *s == [0.0; 2]));
        assert_eq!(m.owner.collect(&mut m.bridge), 1);
        m.owner.reconcile_pause(&m.bridge).unwrap();
        m.owner.refill(&mut m.bridge).unwrap();
        m.resume().unwrap();
        assert_eq!(m.output(16).1.voice_sample_adds, 0);
        assert_eq!(m.callback.next_background, u64::from(submit_first) + 16);
    }
}

#[test]
fn resume_rejects_frozen_scene_tokens_and_obsolete_pause_ack_without_invalidating_audition() {
    let mut m = Model::new();
    let pcm = m.ready(16, 0.25);
    let audition_token = m.bridge_token(Channel::Audition);
    m.pause();
    let frozen_scene = m.bridge_token(Channel::Scene);
    m.output(1);
    let old_ack = m.bridge.pause_ack;
    m.pause();
    m.bridge.pause_ack = old_ack; // A late acknowledgement from the first pause.
    assert_eq!(
        m.owner.reconcile_pause(&m.bridge),
        Err(Refusal::AwaitingPauseAck)
    );
    assert_eq!(m.resume(), Err(Refusal::AwaitingPauseAck));
    m.output(1);
    m.owner.reconcile_pause(&m.bridge).unwrap();
    m.owner.refill(&mut m.bridge).unwrap();
    let latest_frozen_scene = m.bridge_token(Channel::Scene);
    m.resume().unwrap();
    for stale in [frozen_scene, latest_frozen_scene] {
        assert_eq!(
            m.owner.admit(&mut m.bridge, m.owner.source, stale, &pcm),
            Err(Refusal::Stale)
        );
    }
    assert!(
        m.owner
            .admit(&mut m.bridge, m.owner.source, audition_token, &pcm)
            .is_ok()
    );
    assert_eq!(m.output(1).1.first_count, 1);
}

#[test]
fn repeated_retirement_reuses_slots_but_never_voice_identity_or_pcm_budget() {
    let mut m = Model::new();
    m.pause();
    m.output(1);
    let budget = m.owner.budget.clone();
    let base = budget.usage().bytes;
    let mut previous = None;
    for _ in 0..300 {
        let pcm = m.ready(1, 0.25);
        let weak = pcm.0.downgrade();
        for _ in 0..MAX_VOICES {
            let key = m.admit(Channel::Audition, &pcm).unwrap();
            assert!(previous.is_none_or(|old: Key| key.generation > old.generation));
            previous = Some(key);
        }
        drop(pcm);
        m.output(1);
        assert!(weak.is_alive());
        assert_eq!(budget.usage().bytes, base + 8);
        assert_eq!(m.owner.collect(&mut m.bridge), MAX_VOICES);
        assert!(!weak.is_alive());
        assert_eq!(budget.usage().bytes, base);
    }
    assert_eq!(m.callback.next_background, 0);
    drop(m);
    assert_eq!(budget.usage().bytes, 0);
}

#[test]
fn shared_pcm_has_one_charge_and_wrong_pool_or_invalid_prepared_audio_is_rejected() {
    let mut m = Model::new();
    let base = m.owner.budget.usage().bytes;
    let pcm = m.ready(4, 0.25);
    let weak = pcm.0.downgrade();
    for _ in 0..MAX_VOICES {
        m.admit(Channel::Audition, &pcm).unwrap();
    }
    drop(pcm);
    assert_eq!(m.owner.budget.usage().bytes, base + 4 * 2 * 4);
    m.output(4);
    assert!(weak.is_alive());
    assert_eq!(m.owner.collect(&mut m.bridge), MAX_VOICES);
    assert!(!weak.is_alive());
    assert_eq!(m.owner.budget.usage().bytes, base);
    let wrong = Ready::new(PcmBudget::default().reserve(2).unwrap().allocate()).unwrap();
    assert_eq!(m.admit(Channel::Scene, &wrong), Err(Refusal::WrongPool));
    let odd = m.owner.budget.reserve(1).unwrap().allocate();
    assert!(matches!(Ready::new(odd), Err(Refusal::InvalidPcm)));
    let mut bad = m.owner.budget.reserve(2).unwrap().allocate();
    bad.samples_mut().unwrap()[0] = f32::NAN;
    assert!(matches!(Ready::new(bad), Err(Refusal::InvalidPcm)));
    assert!(m.owner.pins.iter().all(Option::is_none));
}

#[test]
fn close_with_full_handoff_retires_every_lease_on_owner_and_stops_submission() {
    let mut m = Model::new();
    let budget = m.owner.budget.clone();
    let pcm = m.ready(1000, 0.25);
    let weak = pcm.0.downgrade();
    for _ in 0..MAX_VOICES {
        m.admit(Channel::Audition, &pcm).unwrap();
    }
    drop(pcm);
    m.close();
    let (samples, report) = m.output(PERIOD);
    assert!(samples.iter().all(|s| *s == [0.0; 2]));
    assert_eq!(report.missing, 0);
    assert_eq!(m.callback.next_background, 0);
    assert!(weak.is_alive());
    assert_eq!(m.owner.collect(&mut m.bridge), MAX_VOICES);
    assert!(!weak.is_alive());
    let another = m.ready(1, 0.25);
    assert_eq!(m.admit(Channel::Audition, &another), Err(Refusal::Closed));
    assert_eq!(m.owner.refill(&mut m.bridge), Err(Refusal::Closed));
    assert_eq!(m.resume(), Err(Refusal::Closed));
    drop(another);
    drop(m); // Device quiescent in this sequential test; background owner drops.
    assert_eq!(budget.usage().bytes, 0);
}

// Test driver only: no wall clock, sleeps, threads, devices or GUI.
fn scheduled_probe(
    channel: Channel,
    requested: u64,
    events: &[(u64, bool)],
) -> (u64, usize, usize) {
    let mut m = Model::new();
    let pcm = m.ready(16, 0.25);
    let mut accepted = None;
    let mut submitted = None;
    let mut missing = 0;
    for &(at, callback) in events {
        if callback {
            let (_, report) = m.output(PERIOD);
            missing += report.missing;
            if report.first_count > 0 {
                assert_eq!(report.first[0], accepted);
                assert!(submitted.is_none());
                submitted = Some(at + 1000); // Assumed completion upper bound, not observed CPU work.
            }
        } else {
            m.owner.collect(&mut m.bridge);
            // Deliberately refill BEFORE accepting new requests: the hard case.
            m.owner.refill(&mut m.bridge).unwrap();
            if at >= requested && accepted.is_none() {
                accepted = Some(m.admit(channel, &pcm).unwrap());
            }
        }
    }
    let budget = m.owner.budget.clone();
    let peak = budget.usage().peak_bytes;
    drop(pcm);
    drop(m); // Driver has no callback in flight; final owners are released here.
    assert_eq!(budget.usage().bytes, 0);
    (submitted.unwrap() - requested, missing, peak)
}

// Virtual microseconds are inputs, never wall-clock measurements. Callback work
// <=1ms, owner admission each2ms, regular callback period10ms are assumptions.
// Both orderings at tied timestamps are covered; bursts need no owner between.
#[test]
fn deterministic_phase_sweep_reports_submission_bounds_without_relaxing_twenty_ms() {
    let mut latencies = [Vec::new(), Vec::new()];
    let mut total_missing = 0;
    let mut total_probes = 0;
    let mut pcm_peak = 0;
    for channel in [Channel::Scene, Channel::Audition] {
        for burst in [false, true] {
            for callback_first in [false, true] {
                for requested in (0..=30_000_u64).step_by(100) {
                    let mut events = Vec::new();
                    for at in (0..=50_000_u64).step_by(2000) {
                        events.push((at, false));
                    }
                    for at in (10_000..=50_000_u64).step_by(10_000) {
                        events.push((at, true));
                    }
                    if burst {
                        events.extend([(30_148, true), (31_384, true)]);
                    }
                    events.sort_by_key(|(at, cb)| (*at, *cb != callback_first));
                    let (elapsed, missing, peak) = scheduled_probe(channel, requested, &events);
                    pcm_peak = pcm_peak.max(peak);
                    total_missing += missing;
                    latencies[lane(channel)].push(elapsed);
                    assert!(
                        elapsed <= 20_000,
                        "include admission, no queue wait subtracted"
                    );
                    total_probes += 1;
                }
            }
        }
    }
    let mut p95 = [0; 2];
    let mut maximum = [0; 2];
    for (i, values) in latencies.iter_mut().enumerate() {
        values.sort_unstable();
        p95[i] = values[(values.len() * 95).div_ceil(100) - 1];
        maximum[i] = *values.last().unwrap();
        assert!(p95[i] <= 20_000);
    }
    assert_eq!(total_missing, 0);
    println!(
        "MODEL_PROOF {{\"probes\":{total_probes},\"missing_frames\":{total_missing},\"scene_p95_us\":{},\"audition_p95_us\":{},\"scene_max_us\":{},\"audition_max_us\":{},\"reserve_frames\":{RESERVE},\"pcm_peak_bytes\":{pcm_peak},\"pcm_final_bytes\":0,\"model_inline_bytes\":{}}}",
        p95[0],
        p95[1],
        maximum[0],
        maximum[1],
        std::mem::size_of::<Model>(),
    );
}

#[test]
fn admission_stall_beyond_model_envelope_is_a_latency_failure_not_a_pass() {
    // Background reserve covers these callbacks; admission owner does not run
    // until25ms, so a request at0ms still misses20ms. Separation is not a cure
    // for arbitrary control-thread starvation.
    let events = [
        (10_000, true),
        (20_000, true),
        (25_000, false),
        (30_000, true),
    ];
    let (elapsed, missing, _) = scheduled_probe(Channel::Audition, 0, &events);
    assert_eq!(missing, 0);
    assert_eq!(elapsed, 31_000);
    assert!(elapsed > 20_000);
}
