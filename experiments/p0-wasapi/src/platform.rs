//! All COM and returned native pointers stay on the worker's MTA thread.
use super::{Content, Error, Format, buffered, pcm};
use std::time::{Duration, Instant};
use windows::{
    Win32::{Foundation::S_OK, Media::Audio::*, System::Com::*},
    core::Interface,
};
fn error(operation: &'static str, e: windows::core::Error) -> Error {
    Error {
        operation,
        code: Some(e.code().0),
    }
}
struct Apartment;
impl Apartment {
    fn new() -> Result<Self, Error> {
        // This new worker thread has no previous COM apartment; balanced in Drop.
        unsafe { CoInitializeEx(None, COINIT_MULTITHREADED).ok() }
            .map_err(|e| error("CoInitializeEx", e))?;
        Ok(Self)
    }
}
impl Drop for Apartment {
    fn drop(&mut self) {
        unsafe { CoUninitialize() };
    }
}
struct Mix(*mut WAVEFORMATEX);
impl Drop for Mix {
    fn drop(&mut self) {
        unsafe { CoTaskMemFree(Some(self.0.cast())) };
    }
}
pub struct Stream {
    // Interfaces drop before apartment teardown; Stream never leaves this thread.
    clock: IAudioClock,
    render: IAudioRenderClient,
    client: IAudioClient,
    pub format: Format,
    running: bool,
    pcm: Option<pcm::Cursor>,
    buffered: Option<buffered::Input>,
    scratch: Vec<u8>,
    content_origin_us: u64,
    position_bias_us: u64,
    monitor: Option<(IMMDeviceEnumerator, Vec<u16>, Instant)>,
    _apartment: Apartment,
}
impl Stream {
    pub fn open_content(content: &Content, position_us: u64) -> Result<Self, Error> {
        let apartment = Apartment::new()?;
        // Generated bindings enforce interface signatures. No user pointers enter.
        let enumerator: IMMDeviceEnumerator =
            unsafe { CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_INPROC_SERVER) }
                .map_err(|e| error("Create endpoint enumerator", e))?;
        let device = unsafe { enumerator.GetDefaultAudioEndpoint(eRender, eMultimedia) }
            .map_err(|e| error("Get default output", e))?;
        let client: IAudioClient = unsafe { device.Activate(CLSCTX_INPROC_SERVER, None) }
            .map_err(|e| error("Activate IAudioClient", e))?;
        let mix = Mix(unsafe { client.GetMixFormat() }.map_err(|e| error("GetMixFormat", e))?);
        if mix.0.is_null() {
            return Err(Error {
                operation: "Null mix format",
                code: None,
            });
        }
        // GetMixFormat owns this valid WAVEFORMATEX allocation until Mix::drop.
        let mix_raw = unsafe { *mix.0 };
        let mix_rate = mix_raw.nSamplesPerSec;
        let mix_channels = mix_raw.nChannels;
        if !(8000..=384000).contains(&mix_rate) || !(1..=32).contains(&mix_channels) {
            return Err(Error {
                operation: "Unexpected engine mix format",
                code: None,
            });
        }
        let converting = matches!(content, Content::Buffered { convert: true, .. });
        let requested = if let Content::Buffered {
            input,
            convert: true,
        } = content
        {
            let c = input.config();
            Some(float_format(c.rate, c.channels)?)
        } else {
            None
        };
        let raw = requested.unwrap_or(mix_raw);
        let rate = raw.nSamplesPerSec;
        let channels = raw.nChannels;
        if !(8000..=384000).contains(&rate) || !(1..=32).contains(&channels) {
            return Err(Error {
                operation: "Unexpected mix format",
                code: None,
            });
        }
        let pcm = if !matches!(content, Content::Silent) {
            let subformat = if raw.wFormatTag == 0xfffe && raw.cbSize >= 22 {
                // GetMixFormat supplies WAVEFORMATEX plus cbSize extension bytes.
                let extended = unsafe { mix.0.cast::<WAVEFORMATEXTENSIBLE>().read_unaligned() };
                let valid = unsafe { extended.Samples.wValidBitsPerSample };
                Some((extended.SubFormat, valid))
            } else {
                None
            };
            validate_float(raw, subformat)?;
            Some(match content {
                Content::Generated => pcm::Cursor::new(rate, channels, position_us)?,
                Content::Clip(clip) => {
                    pcm::Cursor::from_clip(rate, channels, position_us, clip.clone())?
                }
                Content::Buffered { input, .. } => {
                    let c = input.config();
                    if c.rate != rate || c.channels != usize::from(channels) {
                        return Err(Error {
                            operation: "Buffered PCM format must match output",
                            code: None,
                        });
                    }
                    // The bounded queue has its own long-content cursor.
                    pcm::Cursor::new(rate, channels, 0)?
                }
                Content::Silent => unreachable!(),
            })
        } else {
            None
        };
        let flags = if converting {
            AUDCLNT_STREAMFLAGS_AUTOCONVERTPCM | AUDCLNT_STREAMFLAGS_SRC_DEFAULT_QUALITY
        } else {
            0
        };
        // The optional stack format stays alive through Initialize; no format
        // pointer or COM interface escapes this worker. Legacy mode uses GetMixFormat.
        let format_ptr = requested
            .as_ref()
            .map_or(mix.0.cast_const(), |r| r as *const WAVEFORMATEX);
        unsafe {
            client.Initialize(
                AUDCLNT_SHAREMODE_SHARED,
                flags,
                1_000_000,
                0,
                format_ptr,
                None,
            )
        }
        .map_err(|e| error("Initialize shared output", e))?;
        let frames = unsafe { client.GetBufferSize() }.map_err(|e| error("GetBufferSize", e))?;
        if matches!(content, Content::Buffered { .. })
            && frames as usize > buffered::BLOCK_FRAMES * buffered::QUEUE_BLOCKS
        {
            return Err(Error {
                operation: "Native buffer exceeds bounded PCM queue",
                code: None,
            });
        }
        if let Some(cursor) = &pcm {
            cursor.packet_bytes(frames)?;
        }
        let render = unsafe { client.GetService::<IAudioRenderClient>() }
            .map_err(|e| error("Get render service", e))?;
        let clock = unsafe { client.GetService::<IAudioClock>() }
            .map_err(|e| error("Get clock service", e))?;
        let frequency = unsafe { clock.GetFrequency() }.map_err(|e| error("GetFrequency", e))?;
        if frequency == 0 {
            return Err(Error {
                operation: "Zero device frequency",
                code: None,
            });
        }
        let format = Format {
            rate: raw.nSamplesPerSec,
            channels: raw.nChannels,
            bits: raw.wBitsPerSample,
            buffer_frames: frames,
            clock_frequency: frequency,
            mix_rate: mix_raw.nSamplesPerSec,
            mix_channels: mix_raw.nChannels,
            conversion_enabled: converting,
        };
        let monitor = if converting {
            Some((enumerator, endpoint_id(&device)?, Instant::now()))
        } else {
            None
        };
        let buffered = if let Content::Buffered { input, .. } = content {
            input.activate(position_us)?;
            Some(input.0.clone())
        } else {
            None
        };
        let scratch = if buffered.is_some() {
            vec![0; frames as usize * usize::from(channels) * 4]
        } else {
            Vec::new()
        };
        Ok(Self {
            clock,
            render,
            client,
            format,
            running: false,
            pcm: if buffered.is_some() { None } else { pcm },
            buffered,
            scratch,
            content_origin_us: position_us,
            position_bias_us: 0,
            monitor,
            _apartment: apartment,
        })
    }
    pub fn check_device(&mut self) -> Result<(), Error> {
        if let Some((enumerator, opened_id, last)) = &mut self.monitor
            && last.elapsed() >= Duration::from_millis(250)
        {
            *last = Instant::now();
            let current = unsafe { enumerator.GetDefaultAudioEndpoint(eRender, eMultimedia) }
                .map_err(|e| error("Default output unavailable", e))?;
            if endpoint_id(&current)? != *opened_id {
                return Err(Error {
                    operation: "Default output changed",
                    code: None,
                });
            }
            // Also exercise a native client operation during pause, so a same-ID
            // invalidation is not hidden by a stopped clock's cached position.
            unsafe { self.client.GetCurrentPadding() }
                .map_err(|e| error("Paused device health", e))?;
        }
        Ok(())
    }
    pub fn feed(&mut self) -> Result<(), Error> {
        if self.buffered.is_some() {
            return self.feed_buffered();
        }
        if self.running
            && let Some(cursor) = &self.pcm
        {
            cursor.check_playhead(self.position()?)?;
        }
        let padding = unsafe { self.client.GetCurrentPadding() }
            .map_err(|e| error("GetCurrentPadding", e))?;
        let frames = self
            .format
            .buffer_frames
            .checked_sub(padding)
            .ok_or(Error {
                operation: "Padding exceeds buffer",
                code: None,
            })?;
        if frames != 0 {
            // Validate length before acquiring a native packet. Byte writes avoid
            // assuming pointer alignment; format validation establishes 4 bytes/channel.
            let bytes = self
                .pcm
                .as_ref()
                .map(|c| c.packet_bytes(frames))
                .transpose()?;
            let buffer =
                unsafe { self.render.GetBuffer(frames) }.map_err(|e| error("GetBuffer", e))?;
            if let (Some(cursor), Some(bytes)) = (&mut self.pcm, bytes) {
                let prepared = if buffer.is_null() {
                    Err(Error {
                        operation: "Null render packet",
                        code: None,
                    })
                } else {
                    // WASAPI guarantees requested frames * nBlockAlign writable
                    // bytes until the paired ReleaseBuffer on this same MTA thread.
                    cursor.prepare(frames, unsafe {
                        std::slice::from_raw_parts_mut(buffer, bytes)
                    })
                };
                let packet = match prepared {
                    Ok(value) => value,
                    Err(error) => {
                        let _ = unsafe {
                            self.render
                                .ReleaseBuffer(frames, AUDCLNT_BUFFERFLAGS_SILENT.0 as u32)
                        };
                        return Err(error);
                    }
                };
                unsafe { self.render.ReleaseBuffer(frames, 0) }
                    .map_err(|e| error("Release PCM packet", e))?;
                cursor.commit(packet)?;
                return Ok(());
            }
            // Default path remains silent and never dereferences the native pointer.
            unsafe {
                self.render
                    .ReleaseBuffer(frames, AUDCLNT_BUFFERFLAGS_SILENT.0 as u32)
            }
            .map_err(|e| error("ReleaseBuffer", e))?;
        }
        Ok(())
    }
    pub fn pcm_stats(&self) -> Option<pcm::Stats> {
        self.pcm.as_ref().map(pcm::Cursor::stats)
    }
    pub fn buffered_stats(&self) -> Option<buffered::Stats> {
        self.buffered.as_ref().map(buffered::Input::stats)
    }
    pub fn ready(&self) -> Result<bool, Error> {
        self.buffered
            .as_ref()
            .map_or(Ok(true), |input| input.ready(self.format.buffer_frames))
    }
    fn feed_buffered(&mut self) -> Result<(), Error> {
        let input = self.buffered.as_ref().unwrap();
        if self.running {
            let consumed =
                u128::from(self.raw_position()?) * u128::from(self.format.rate) / 1_000_000;
            if consumed > u128::from(input.stats().pcm.submitted_frames) + 1 {
                return Err(input.underrun());
            }
        }
        let padding = unsafe { self.client.GetCurrentPadding() }
            .map_err(|e| error("GetCurrentPadding", e))?;
        let frames = self
            .format
            .buffer_frames
            .checked_sub(padding)
            .ok_or(Error {
                operation: "Padding exceeds buffer",
                code: None,
            })?;
        if frames == 0 {
            return Ok(());
        }
        let bytes = frames as usize * usize::from(self.format.channels) * 4;
        let Some(packet) = input.prepare(frames, &mut self.scratch[..bytes])? else {
            if self.running && padding == 0 {
                return Err(input.underrun());
            }
            return Ok(());
        };
        let buffer = unsafe { self.render.GetBuffer(frames) }
            .map_err(|e| error("Get streamed buffer", e))?;
        if buffer.is_null() {
            let _ = unsafe {
                self.render
                    .ReleaseBuffer(frames, AUDCLNT_BUFFERFLAGS_SILENT.0 as u32)
            };
            return Err(Error {
                operation: "Null streamed buffer",
                code: None,
            });
        }
        // Decoding and queue waiting finished before acquiring this native packet.
        // Same validated float layout, full initialization and paired release as PCM.
        unsafe { std::slice::from_raw_parts_mut(buffer, bytes) }
            .copy_from_slice(&self.scratch[..bytes]);
        unsafe { self.render.ReleaseBuffer(frames, 0) }
            .map_err(|e| error("Release streamed packet", e))?;
        input.commit(packet)
    }
    pub fn pause(&mut self, paused: bool) -> Result<(), Error> {
        if paused && self.running {
            unsafe { self.client.Stop() }.map_err(|e| error("Stop output", e))?;
            self.running = false;
        }
        if paused && let Some(input) = &self.buffered {
            let position = self.position()?;
            let raw = self.raw_position()?;
            let consumed = u64::try_from(
                u128::from(raw) * u128::from(self.format.rate) / 1_000_000,
            )
            .map_err(|_| Error {
                operation: "Buffered position overflow",
                code: None,
            })?;
            let discarded = input.stats().pcm.submitted_frames.saturating_sub(consumed);
            // Reset is legal only while stopped; it flushes unread endpoint data
            // and returns the device clock to zero. The logical bias stays frozen.
            unsafe { self.client.Reset() }.map_err(|e| error("Reset paused streamed output", e))?;
            self.position_bias_us = position;
            input.restart(
                (self.content_origin_us + position).min(input.duration_us()),
                buffered::Kind::Pause,
                discarded,
            )?;
        }
        if !paused && !self.running {
            self.feed()?;
            unsafe { self.client.Start() }.map_err(|e| error("Start output", e))?;
            self.running = true;
        }
        Ok(())
    }
    pub fn position(&self) -> Result<u64, Error> {
        self.position_bias_us
            .checked_add(self.raw_position()?)
            .ok_or(Error {
                operation: "Buffered clock overflow",
                code: None,
            })
    }
    fn raw_position(&self) -> Result<u64, Error> {
        let mut position = 0;
        let mut qpc = 0;
        // Direct HRESULT preserves S_FALSE as an imprecise reading rather than
        // erasing it through Result<()>. Both output pointers are stack-owned.
        let hr = unsafe {
            (Interface::vtable(&self.clock).GetPosition)(
                Interface::as_raw(&self.clock),
                &mut position,
                &mut qpc,
            )
        };
        precise(hr)?;
        micros(position, self.format.clock_frequency)
    }
}
fn endpoint_id(device: &IMMDevice) -> Result<Vec<u16>, Error> {
    let id = unsafe { device.GetId() }.map_err(|e| error("Get endpoint identity", e))?;
    if id.is_null() {
        return Err(Error {
            operation: "Null endpoint identity",
            code: None,
        });
    }
    // GetId supplies a NUL-terminated CoTaskMem allocation. Copy only the opaque
    // identifier for comparison; do not log names/IDs or retain native pointers.
    let value = unsafe { id.as_wide() }.to_vec();
    unsafe { CoTaskMemFree(Some(id.0.cast())) };
    Ok(value)
}
fn float_format(rate: u32, channels: usize) -> Result<WAVEFORMATEX, Error> {
    if !(8000..=192000).contains(&rate) || !(1..=2).contains(&channels) {
        return Err(Error {
            operation: "Unsupported conversion source format",
            code: None,
        });
    }
    let align = channels as u16 * 4;
    Ok(WAVEFORMATEX {
        wFormatTag: 3,
        nChannels: channels as u16,
        nSamplesPerSec: rate,
        nAvgBytesPerSec: rate * u32::from(align),
        nBlockAlign: align,
        wBitsPerSample: 32,
        cbSize: 0,
    })
}
fn validate_float(
    raw: WAVEFORMATEX,
    extended: Option<(windows::core::GUID, u16)>,
) -> Result<(), Error> {
    // SDK identity also present in pinned windows 0.62.2 Media::Multimedia;
    // using the value avoids enabling an unrelated binding feature for one GUID.
    let float = windows::core::GUID::from_u128(0x00000003_0000_0010_8000_00aa00389b71);
    let encoding = raw.wFormatTag == 3
        || (raw.wFormatTag == 0xfffe && raw.cbSize >= 22 && extended == Some((float, 32)));
    let channels = raw.nChannels;
    if !encoding
        || raw.wBitsPerSample != 32
        || !(1..=2).contains(&channels)
        || raw.nBlockAlign != channels * 4
        || Some(raw.nAvgBytesPerSec) != raw.nSamplesPerSec.checked_mul(u32::from(raw.nBlockAlign))
    {
        return Err(Error {
            operation: "PCM proof requires mono/stereo IEEE float32 mix format",
            code: None,
        });
    }
    Ok(())
}
fn precise(hr: windows::core::HRESULT) -> Result<(), Error> {
    if hr == S_OK {
        Ok(())
    } else {
        Err(Error {
            operation: "GetPosition precise reading",
            code: Some(hr.0),
        })
    }
}
fn micros(position: u64, frequency: u64) -> Result<u64, Error> {
    if frequency == 0 {
        return Err(Error {
            operation: "Zero device frequency",
            code: None,
        });
    }
    (u128::from(position) * 1_000_000 / u128::from(frequency))
        .try_into()
        .map_err(|_| Error {
            operation: "Device clock overflow",
            code: None,
        })
}
impl Drop for Stream {
    fn drop(&mut self) {
        if self.running {
            let _ = unsafe { self.client.Stop() };
        }
        let _ = unsafe { self.client.Reset() };
        if let Some(input) = &self.buffered {
            input.retire();
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn conversion_describes_source_frames_without_reinterpreting_rate_or_channels() {
        for (rate, channels) in [(44100, 1), (32000, 2), (96000, 2), (48000, 1)] {
            let raw = float_format(rate, channels).unwrap();
            assert!(validate_float(raw, None).is_ok());
            let actual_rate = raw.nSamplesPerSec;
            let actual_channels = raw.nChannels;
            let alignment = raw.nBlockAlign;
            assert_eq!(
                (actual_rate, actual_channels, alignment),
                (rate, channels as u16, channels as u16 * 4)
            );
        }
        for (rate, channels) in [(0, 1), (192001, 2), (48000, 0), (48000, 6)] {
            assert!(float_format(rate, channels).is_err());
        }
    }
    #[test]
    fn clock_units_need_not_be_sample_frames() {
        assert_eq!(micros(192000, 384000).unwrap(), 500000);
        assert_eq!(micros(0, 384000).unwrap(), 0);
        assert!(micros(1, 0).is_err());
        assert!(micros(u64::MAX, 1).is_err());
        assert!(precise(S_OK).is_ok());
        assert_eq!(
            precise(windows::Win32::Foundation::S_FALSE)
                .unwrap_err()
                .code,
            Some(1)
        );
    }
    #[test]
    fn pcm_format_requires_float_identity_valid_bits_and_exact_frame_layout() {
        let raw = WAVEFORMATEX {
            wFormatTag: 3,
            nChannels: 2,
            nSamplesPerSec: 48000,
            nAvgBytesPerSec: 384000,
            nBlockAlign: 8,
            wBitsPerSample: 32,
            cbSize: 0,
        };
        assert!(validate_float(raw, None).is_ok());
        let ext = WAVEFORMATEX {
            wFormatTag: 0xfffe,
            cbSize: 22,
            ..raw
        };
        let float = windows::core::GUID::from_u128(0x00000003_0000_0010_8000_00aa00389b71);
        assert!(validate_float(ext, Some((float, 32))).is_ok());
        for bad in [
            WAVEFORMATEX {
                wFormatTag: 1,
                ..raw
            },
            WAVEFORMATEX {
                nBlockAlign: 4,
                ..raw
            },
            WAVEFORMATEX {
                nAvgBytesPerSec: 0,
                ..raw
            },
            WAVEFORMATEX { cbSize: 21, ..ext },
        ] {
            assert!(validate_float(bad, Some((float, 32))).is_err());
        }
        assert!(validate_float(ext, Some((float, 24))).is_err());
        assert!(validate_float(ext, None).is_err());
        assert!(validate_float(ext, Some((windows::core::GUID::zeroed(), 32))).is_err());
    }
}
