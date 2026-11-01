//! Explicit F32 Windows executor for the separated candidate. It does not
//! select the default product path or automatically rebuild devices.
use crate::separated_output::{BACKGROUND_RESERVE, OutputConsumer, StoppedOutput};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rtrb::{Consumer, Producer, RingBuffer};
use skin_editor_audio::Channel;
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU16, AtomicU64, Ordering},
    mpsc::{Receiver, SyncSender, sync_channel},
};

const OBSERVATIONS: usize = 128;
#[derive(Clone, Copy, Debug)]
pub struct FirstCopy {
    pub voice: u64,
    pub channel: Channel,
    pub callback_nanos: u64,
    pub callback_ordinal: u64,
    pub offset_frames: usize,
    pub callback_frames: usize,
}
#[derive(Clone, Copy, Debug, Default)]
pub struct NativeStats {
    pub callbacks: u64,
    pub frames: u64,
    pub max_callback_frames: u64,
    pub missing_frames: u64,
    pub render_errors: u64,
    pub observation_overflows: u64,
    pub nonzero_samples: u64,
    pub endpoint_returns: u64,
    pub device_fault: bool,
    pub callback_resource_releases: u64,
    pub endpoint_returns_in_render: u64,
    pub endpoint_returns_outside_render: u64,
}
// One writer, bounded readers. No partial combination of callback counters.
#[derive(Default)]
struct WorkCounters {
    stamp: AtomicU64,
    values: [AtomicU64; 6],
}
impl WorkCounters {
    fn publish(&self, values: [u64; 6]) {
        self.stamp.fetch_add(1, Ordering::SeqCst);
        for (cell, value) in self.values.iter().zip(values) {
            cell.store(value, Ordering::SeqCst);
        }
        self.stamp.fetch_add(1, Ordering::SeqCst);
    }
    fn read(&self) -> Option<[u64; 6]> {
        for _ in 0..3 {
            let before = self.stamp.load(Ordering::SeqCst);
            if before & 1 != 0 {
                continue;
            }
            let values = std::array::from_fn(|i| self.values[i].load(Ordering::SeqCst));
            if before == self.stamp.load(Ordering::SeqCst) {
                return Some(values);
            }
        }
        None
    }
}
struct Shared {
    control: Option<crate::separated_output::Control>,
    volume: AtomicU16,
    callbacks: AtomicU64,
    frames: AtomicU64,
    max_frames: AtomicU64,
    missing: AtomicU64,
    errors: AtomicU64,
    overflows: AtomicU64,
    nonzero: AtomicU64,
    returned: AtomicU64,
    fault: AtomicBool,
    closing: AtomicBool,
    releases: AtomicU64,
    returned_in: AtomicU64,
    returned_out: AtomicU64,
    work_enabled: AtomicBool,
    work: WorkCounters,
}
/// Sticky, endpoint-local device error. Contains counters only, no PCM owner.
/// A signal retained by a retired backend cannot fault its replacement.
#[derive(Clone)]
pub struct NativeFaultSignal(Arc<Shared>);
impl NativeFaultSignal {
    pub fn report(&self) {
        if let Some(control) = &self.0.control {
            control.close();
        }
        self.0.fault.store(true, Ordering::Release);
    }
}
impl Default for Shared {
    fn default() -> Self {
        Self {
            control: None,
            volume: AtomicU16::new(1000),
            callbacks: AtomicU64::new(0),
            frames: AtomicU64::new(0),
            max_frames: AtomicU64::new(0),
            missing: AtomicU64::new(0),
            errors: AtomicU64::new(0),
            overflows: AtomicU64::new(0),
            nonzero: AtomicU64::new(0),
            returned: AtomicU64::new(0),
            fault: AtomicBool::new(false),
            closing: AtomicBool::new(false),
            releases: AtomicU64::new(0),
            returned_in: AtomicU64::new(0),
            returned_out: AtomicU64::new(0),
            work_enabled: AtomicBool::new(false),
            work: WorkCounters::default(),
        }
    }
}
/// Optional diagnostics reader. Moving it does not move the OS stream or PCM.
pub struct FirstObserver(Consumer<FirstCopy>);
impl FirstObserver {
    pub fn next_first(&mut self) -> Option<FirstCopy> {
        self.0.pop().ok()
    }
}
impl Shared {
    fn stats(&self) -> NativeStats {
        NativeStats {
            callbacks: self.callbacks.load(Ordering::Acquire),
            frames: self.frames.load(Ordering::Acquire),
            max_callback_frames: self.max_frames.load(Ordering::Acquire),
            missing_frames: self.missing.load(Ordering::Acquire),
            render_errors: self.errors.load(Ordering::Acquire),
            observation_overflows: self.overflows.load(Ordering::Acquire),
            nonzero_samples: self.nonzero.load(Ordering::Acquire),
            endpoint_returns: self.returned.load(Ordering::Acquire),
            device_fault: self.fault.load(Ordering::Acquire),
            callback_resource_releases: self.releases.load(Ordering::Acquire),
            endpoint_returns_in_render: self.returned_in.load(Ordering::Acquire),
            endpoint_returns_outside_render: self.returned_out.load(Ordering::Acquire),
        }
    }
}
struct Callback {
    endpoint: Option<Box<OutputConsumer>>,
    returned: SyncSender<StoppedOutput>,
    first: Producer<FirstCopy>,
    shared: Arc<Shared>,
    channels: usize,
    work_accum: [u64; 6],
    direct_return: bool,
}
// This destructor is NOT a render callback. CPAL owns the closure; it transfers
// the endpoint through a precreated one-slot return channel when destroyed.
// NativeExecutor keeps the receiver alive across stream destruction. On WASAPI
// the control-thread Stream::drop joins the worker before we inspect the return.
impl Drop for Callback {
    fn drop(&mut self) {
        if let Some(endpoint) = self.endpoint.take() {
            if crate::callback_audit::active() {
                self.shared.returned_in.fetch_add(1, Ordering::Release);
            } else {
                self.shared.returned_out.fetch_add(1, Ordering::Release);
            }
            if self.returned.try_send(endpoint.stop()).is_ok() {
                if self.direct_return {
                    self.shared.returned.fetch_add(1, Ordering::Release);
                }
            } else {
                self.shared.fault.store(true, Ordering::Release);
            }
        }
    }
}
impl Callback {
    fn render(&mut self, output: &mut [f32], stamp: u64) {
        let started = self
            .shared
            .work_enabled
            .load(Ordering::Acquire)
            .then(std::time::Instant::now);
        let scope = crate::callback_audit::Scope::enter();
        let mut parts = [0u64; 3];
        self.render_inner(output, stamp, started.is_some(), &mut parts);
        self.shared
            .releases
            .fetch_add(scope.finish(), Ordering::Release);
        if let Some(started) = started {
            let values = [
                1,
                started.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64,
                parts[0],
                parts[1],
                parts[2],
                (output.len() / self.channels) as u64,
            ];
            for (total, delta) in self.work_accum.iter_mut().zip(values) {
                *total = total.saturating_add(delta);
            }
            self.shared.work.publish(self.work_accum);
        }
    }

    fn render_inner(
        &mut self,
        output: &mut [f32],
        stamp: u64,
        capture: bool,
        parts: &mut [u64; 3],
    ) {
        // Device errors invalidate the next copy immediately, independently of
        // GUI polling. Keep endpoint/PCM ownership for control-side reclamation.
        if self.shared.fault.load(Ordering::Acquire) || self.shared.closing.load(Ordering::Acquire)
        {
            output.fill(0.0);
            return;
        }
        let ordinal = self.shared.callbacks.fetch_add(1, Ordering::Relaxed) + 1;
        let frames = output.len() / self.channels;
        self.shared
            .frames
            .fetch_add(frames as u64, Ordering::Relaxed);
        self.shared
            .max_frames
            .fetch_max(frames as u64, Ordering::Relaxed);
        if !output.len().is_multiple_of(self.channels) {
            output.fill(0.0);
            self.shared.errors.fetch_add(1, Ordering::Release);
            return;
        }
        let volume = f32::from(self.shared.volume.load(Ordering::Acquire)) / 1000.0;
        for (chunk, out) in output
            .chunks_mut(BACKGROUND_RESERVE * self.channels)
            .enumerate()
        {
            let endpoint = self.endpoint.as_mut().expect("live executor");
            let result = if capture {
                endpoint.render_profiled(out).map(|(report, nanos)| {
                    for (total, delta) in parts.iter_mut().zip(nanos) {
                        *total = total.saturating_add(delta);
                    }
                    report
                })
            } else {
                endpoint.render(out)
            };
            match result {
                Ok(report) => {
                    for sample in out.iter_mut() {
                        *sample *= volume;
                    }
                    self.shared
                        .missing
                        .fetch_add(report.missing_frames as u64, Ordering::Relaxed);
                    for (voice, channel) in report.voices.runtime_first.into_iter().flatten() {
                        if self
                            .first
                            .push(FirstCopy {
                                voice,
                                channel,
                                callback_nanos: stamp,
                                callback_ordinal: ordinal,
                                offset_frames: chunk * BACKGROUND_RESERVE,
                                callback_frames: frames,
                            })
                            .is_err()
                        {
                            self.shared.overflows.fetch_add(1, Ordering::Release);
                        }
                    }
                }
                Err(_) => {
                    out.fill(0.0);
                    self.shared.errors.fetch_add(1, Ordering::Release);
                }
            }
        }
        let nonzero = output.iter().filter(|v| **v != 0.0).count() as u64;
        self.shared.nonzero.fetch_add(nonzero, Ordering::Release);
    }
}

pub struct PreparedDevice {
    device: cpal::Device,
    config: cpal::StreamConfig,
}
pub struct NativeExecutor {
    stream: Option<cpal::Stream>,
    returned: Receiver<StoppedOutput>,
    first: Option<FirstObserver>,
    shared: Arc<Shared>,
    build_error: Option<String>,
    period_frames: u32,
    return_received: bool,
}
#[cfg(test)]
fn executor(endpoint: Box<OutputConsumer>) -> (Callback, NativeExecutor) {
    let (callback, native, _) = executor_gated(endpoint, false);
    (callback, native)
}
fn executor_gated(
    endpoint: Box<OutputConsumer>,
    held: bool,
) -> (Callback, NativeExecutor, Option<ReturnGate>) {
    let channels = usize::from(endpoint.format().1);
    let (tx, rx) = sync_channel(1);
    let (first, observations) = RingBuffer::new(OBSERVATIONS);
    let shared = Arc::new(Shared {
        control: Some(endpoint.control()),
        ..Default::default()
    });
    let (sender, gate) = if held {
        let (sender, receiver) = sync_channel(1);
        (
            sender,
            Some(ReturnGate {
                receiver,
                destination: Some(tx),
                endpoint: None,
                shared: shared.clone(),
            }),
        )
    } else {
        (tx, None)
    };
    (
        Callback {
            endpoint: Some(endpoint),
            returned: sender,
            first,
            shared: shared.clone(),
            channels,
            work_accum: [0; 6],
            direct_return: !held,
        },
        NativeExecutor {
            stream: None,
            returned: rx,
            first: Some(FirstObserver(observations)),
            shared,
            build_error: None,
            period_frames: 0,
            return_received: false,
        },
        gate,
    )
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EndpointReturnError {
    Pending,
    Disconnected,
    AlreadyReceived,
}
/// Explicit diagnostic only. A second one-slot channel retains the actual
/// stopped endpoint; no render callback waits on the gate or owns its receiver.
pub struct ReturnGate {
    receiver: Receiver<StoppedOutput>,
    destination: Option<SyncSender<StoppedOutput>>,
    endpoint: Option<StoppedOutput>,
    shared: Arc<Shared>,
}
impl ReturnGate {
    pub fn disconnect(&mut self) {
        self.destination.take();
    }
    pub fn deliver(&mut self) -> Result<(), String> {
        if self.endpoint.is_none() {
            self.endpoint = Some(
                self.receiver
                    .try_recv()
                    .map_err(|e| format!("return gate: {e}"))?,
            );
        }
        let sender = self
            .destination
            .as_ref()
            .ok_or("return gate disconnected")?;
        match sender.try_send(self.endpoint.take().expect("retained endpoint")) {
            Ok(()) => {
                self.shared.returned.fetch_add(1, Ordering::Release);
                Ok(())
            }
            Err(e) => {
                self.endpoint = Some(match e {
                    std::sync::mpsc::TrySendError::Full(endpoint)
                    | std::sync::mpsc::TrySendError::Disconnected(endpoint) => endpoint,
                });
                Err("endpoint delivery refused".into())
            }
        }
    }
}
/// Device-free controlled boundary for cross-layer ownership regressions.
/// It executes the same bounded render code, not a simulated resource count.
pub struct ControlledCallback(Callback);
impl ControlledCallback {
    pub fn render(&mut self, output: &mut [f32], stamp: u64) {
        self.0.render(output, stamp);
    }
}
impl PreparedDevice {
    pub fn format(&self) -> (u32, u16) {
        (self.config.sample_rate, self.config.channels)
    }
    pub fn probe() -> Result<Self, String> {
        if !cfg!(windows) {
            return Err("separated native executor currently verified on Windows only".into());
        }
        let device = cpal::default_host()
            .default_output_device()
            .ok_or("no default output device")?;
        let supported = device.default_output_config().map_err(|e| e.to_string())?;
        if supported.sample_format() != cpal::SampleFormat::F32
            || !matches!(supported.sample_rate(), 44100 | 48000)
            || !matches!(supported.channels(), 1 | 2)
        {
            return Err("separated native candidate requires F32 44.1/48 kHz mono/stereo".into());
        }
        let mut config = supported.config();
        if let cpal::SupportedBufferSize::Range { min, max } = *supported.buffer_size() {
            config.buffer_size =
                cpal::BufferSize::Fixed(128u32.clamp(min.max(1), max.max(min).max(1)));
        }
        Ok(Self { device, config })
    }
    /// Even build failure retains a return receiver; caller must stop/finish to
    /// reclaim the supplied endpoint. Opening never starts playback.
    pub fn attach(self, endpoint: Box<OutputConsumer>) -> NativeExecutor {
        self.attach_inner(endpoint, false).0
    }
    pub fn attach_with_held_return(
        self,
        endpoint: Box<OutputConsumer>,
    ) -> (NativeExecutor, ReturnGate) {
        let (native, gate) = self.attach_inner(endpoint, true);
        (native, gate.expect("explicit return gate"))
    }
    fn attach_inner(
        self,
        endpoint: Box<OutputConsumer>,
        held: bool,
    ) -> (NativeExecutor, Option<ReturnGate>) {
        let matched = endpoint.format() == self.format();
        let (mut callback, mut native, gate) = executor_gated(endpoint, held);
        if !matched {
            native.build_error = Some("native endpoint format mismatch".into());
            drop(callback);
            return (native, gate);
        }
        let fault = native.fault_signal();
        let stream = self.device.build_output_stream(
            self.config,
            move |out: &mut [f32], info| match u64::try_from(info.timestamp().callback.as_nanos()) {
                Ok(stamp) => callback.render(out, stamp),
                Err(_) => {
                    out.fill(0.0);
                    callback.shared.errors.fetch_add(1, Ordering::Release);
                }
            },
            move |_| {
                fault.report();
            },
            Some(std::time::Duration::from_secs(3)),
        );
        match stream {
            Ok(stream) => {
                match stream.buffer_size() {
                    Ok(period) => native.period_frames = period,
                    Err(e) => native.build_error = Some(e.to_string()),
                }
                native.stream = Some(stream);
            }
            Err(e) => native.build_error = Some(e.to_string()),
        }
        (native, gate)
    }
}
impl NativeExecutor {
    pub fn controlled_return(
        endpoint: Box<OutputConsumer>,
    ) -> (Self, ControlledCallback, ReturnGate) {
        let (callback, native, gate) = executor_gated(endpoint, true);
        (
            native,
            ControlledCallback(callback),
            gate.expect("controlled return gate"),
        )
    }
    pub fn fault_signal(&self) -> NativeFaultSignal {
        NativeFaultSignal(self.shared.clone())
    }
    /// Optional fixed diagnostic counters; ordinary output does not time work.
    pub fn capture_work_cost(&self) {
        self.shared.work_enabled.store(true, Ordering::Release);
    }
    pub fn work_cost(&self) -> Option<[u64; 6]> {
        self.shared.work.read()
    }
    pub fn period_frames(&self) -> u32 {
        self.period_frames
    }
    pub fn play(&self) -> Result<(), String> {
        if let Some(e) = &self.build_error {
            return Err(e.clone());
        }
        self.stream
            .as_ref()
            .ok_or("no native stream")?
            .play()
            .map_err(|e| e.to_string())
    }
    pub fn stats(&self) -> NativeStats {
        self.shared.stats()
    }
    pub fn next_first(&mut self) -> Option<FirstCopy> {
        self.first.as_mut()?.next_first()
    }
    pub fn take_first_observer(&mut self) -> Option<FirstObserver> {
        self.first.take()
    }
    pub fn set_monitor_volume(&self, volume: u16) -> Result<(), String> {
        if volume > 1000 {
            return Err("invalid monitor volume".into());
        }
        self.shared.volume.store(volume, Ordering::Release);
        Ok(())
    }
    /// The returned endpoint proves the CPAL closure no longer owns it. On the
    /// validated WASAPI path stream Drop also joins. Neither proves acoustic stop.
    pub fn try_stop(&mut self) -> Result<(StoppedOutput, NativeStats), EndpointReturnError> {
        self.shared.closing.store(true, Ordering::Release);
        if let Some(control) = &self.shared.control {
            control.close();
        }
        drop(self.stream.take());
        if self.return_received {
            return Err(EndpointReturnError::AlreadyReceived);
        }
        let stopped = self.returned.try_recv().map_err(|e| match e {
            std::sync::mpsc::TryRecvError::Empty => EndpointReturnError::Pending,
            std::sync::mpsc::TryRecvError::Disconnected => EndpointReturnError::Disconnected,
        })?;
        self.return_received = true;
        Ok((stopped, self.shared.stats()))
    }
    /// Consuming convenience for standalone verification. Product teardown uses
    /// try_stop and retains this owner/receiver on failure.
    pub fn stop(mut self) -> Result<(StoppedOutput, NativeStats), String> {
        self.try_stop()
            .map_err(|e| format!("CPAL endpoint not returned after stream destruction: {e:?}"))
    }
}
impl Drop for NativeExecutor {
    fn drop(&mut self) {
        // Keep the return receiver alive while the worker/closure is destroyed.
        drop(self.stream.take());
    }
}
#[cfg(test)]
mod tests;
