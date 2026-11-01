# skin_editor_audio：编译器符号清单

快照：2026-10-30。来自含私有项的 rustdoc；显示编译器规范化可见域。含 derive 生成的实现，不把它们视作类继承。来源行可能指向宏定义／展开点。
引用列只列 rustdoc 解析出的项目类型链接，不能充当完整调用图；无链接不表示无引用。所有方法保留源码位置供逐项复核。测试和未启用的 cfg 分支另见源码索引。

## skin_editor_audio::AudioError

[源码:53](../../../desktop/audio/src/lib.rs#L53) · enum

```rust
pub enum AudioError {
    MissingComponent,
    MissingAsset,
    ImageAsset,
    UnsupportedDecode,
    InvalidSamples,
    ResourceLimit,
    WrongSource,
    ProcessingRequired,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_audio::AudioError::clone`, `skin_editor_audio::AudioError::eq`, `skin_editor_audio::SoundPlan::build`, `skin_editor_audio::SoundPlan::original_bytes`, `skin_editor_audio::SoundPlan::output_action`, `skin_editor_audio::SoundPlan::process_pcm`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for AudioError` — [源码:52](../../../desktop/audio/src/lib.rs#L52)
- `impl Debug for AudioError` — [源码:52](../../../desktop/audio/src/lib.rs#L52)
- `impl PartialEq for AudioError` — [源码:52](../../../desktop/audio/src/lib.rs#L52)
- `impl Copy for AudioError` — [源码:52](../../../desktop/audio/src/lib.rs#L52)
- `impl Eq for AudioError` — [源码:52](../../../desktop/audio/src/lib.rs#L52)
- `impl StructuralPartialEq for AudioError` — [源码:52](../../../desktop/audio/src/lib.rs#L52)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> AudioError` | impl Clone for AudioError | [源码:52](../../../desktop/audio/src/lib.rs#L52) | skin_editor_audio::AudioError |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for AudioError | [源码:52](../../../desktop/audio/src/lib.rs#L52) |  |
| `fn eq(&self, other: &AudioError) -> bool` | impl PartialEq for AudioError | [源码:52](../../../desktop/audio/src/lib.rs#L52) | skin_editor_audio::AudioError |

## skin_editor_audio::AuditionChange

[源码:11](../../../desktop/audio/src/audition_edit.rs#L11) · struct

```rust
pub struct AuditionChange {
    pub gain_edit: bool,
    pub preserve_cancel: bool,
    pub replacement: bool,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_audio::audition_edit::AuditionChange`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_audio::AuditionChange::clone`, `skin_editor_audio::AuditionChange::default`, `skin_editor_audio::AuditionEdits::select`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for AuditionChange` — [源码:10](../../../desktop/audio/src/audition_edit.rs#L10)
- `impl Default for AuditionChange` — [源码:10](../../../desktop/audio/src/audition_edit.rs#L10)
- `impl Copy for AuditionChange` — [源码:10](../../../desktop/audio/src/audition_edit.rs#L10)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> AuditionChange` | impl Clone for AuditionChange | [源码:10](../../../desktop/audio/src/audition_edit.rs#L10) | skin_editor_audio::AuditionChange |
| `fn default() -> AuditionChange` | impl Default for AuditionChange | [源码:10](../../../desktop/audio/src/audition_edit.rs#L10) | skin_editor_audio::AuditionChange |

## skin_editor_audio::AuditionEdits

[源码:17](../../../desktop/audio/src/audition_edit.rs#L17) · struct

```rust
pub struct AuditionEdits {
    last: Option<Origin>,
    plans: Vec<SoundPlan>,
    gain_draft: Option<u64>,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_audio::audition_edit::AuditionEdits`

声明引用：`skin_editor_audio::Origin`, `skin_editor_audio::SoundPlan`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_audio::AuditionEdits::clone`, `skin_editor_audio::AuditionEdits::default`, `skin_editor_audio::Runtime (声明)`, `skin_editor_infrastructure::AudioSupplyClient (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl AuditionEdits` — [源码:22](../../../desktop/audio/src/audition_edit.rs#L22)
- `impl Clone for AuditionEdits` — [源码:16](../../../desktop/audio/src/audition_edit.rs#L16)
- `impl Default for AuditionEdits` — [源码:16](../../../desktop/audio/src/audition_edit.rs#L16)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn select( &mut self, origin: Origin, plans: &[SoundPlan], cancelled: &[u64], ) -> AuditionChange` | impl AuditionEdits | [源码:24](../../../desktop/audio/src/audition_edit.rs#L24) | skin_editor_audio::AuditionChange, skin_editor_audio::Origin, skin_editor_audio::SoundPlan |
| `fn clone(&self) -> AuditionEdits` | impl Clone for AuditionEdits | [源码:16](../../../desktop/audio/src/audition_edit.rs#L16) | skin_editor_audio::AuditionEdits |
| `fn default() -> AuditionEdits` | impl Default for AuditionEdits | [源码:16](../../../desktop/audio/src/audition_edit.rs#L16) | skin_editor_audio::AuditionEdits |

## skin_editor_audio::BackgroundTrack

[源码:3](../../../desktop/audio/src/background.rs#L3) · struct

```rust
pub struct BackgroundTrack {
    pcm: PcmBuffer,
    rate: u32,
    channels: u16,
    next: u64,
    running: bool,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_audio::background::BackgroundTrack`

声明引用：`skin_editor_audio::PcmBuffer`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::editor_audio::EditorAudio (声明)`, `skin_editor_desktop::editor_audio::EditorAudio::take_reference`, `skin_editor_infrastructure::AudioSupplyClient::set_background`, `skin_editor_infrastructure::AudioSupplyClient::spawn_separated`, `skin_editor_infrastructure::OutputProducer::pump_timeline`, `skin_editor_infrastructure::audio_supply::Control (声明)`, `skin_editor_infrastructure::audio_supply::SupplyCore (声明)`, `skin_editor_infrastructure::background_input::BackgroundInput (声明)`, `skin_editor_infrastructure::background_input::BackgroundInput::from`, `skin_editor_infrastructure::separated_output::boundary (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl BackgroundTrack` — [源码:11](../../../desktop/audio/src/background.rs#L11)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn new(pcm: PcmBuffer, rate: u32, channels: u16) -> Result<Self, PcmError>` | impl BackgroundTrack | [源码:12](../../../desktop/audio/src/background.rs#L12) | skin_editor_audio::PcmBuffer, skin_editor_audio::PcmError |
| `pub fn format(&self) -> (u32, u16)` | impl BackgroundTrack | [源码:31](../../../desktop/audio/src/background.rs#L31) |  |
| `pub fn pcm(&self) -> &PcmBuffer` | impl BackgroundTrack | [源码:34](../../../desktop/audio/src/background.rs#L34) | skin_editor_audio::PcmBuffer |
| `pub fn position(&self) -> u64` | impl BackgroundTrack | [源码:37](../../../desktop/audio/src/background.rs#L37) |  |
| `pub fn frames(&self) -> u64` | impl BackgroundTrack | [源码:40](../../../desktop/audio/src/background.rs#L40) |  |
| `pub fn is_running(&self) -> bool` | impl BackgroundTrack | [源码:43](../../../desktop/audio/src/background.rs#L43) |  |
| `pub fn set_running(&mut self, running: bool)` | impl BackgroundTrack | [源码:46](../../../desktop/audio/src/background.rs#L46) |  |
| `pub fn restore_position(&mut self, next: u64) -> Result<(), PcmError>` | impl BackgroundTrack | [源码:50](../../../desktop/audio/src/background.rs#L50) | skin_editor_audio::PcmError |
| `pub fn next_frame(&mut self) -> Option<(u64, [f32; 2])>` | impl BackgroundTrack | [源码:57](../../../desktop/audio/src/background.rs#L57) |  |

## skin_editor_audio::BindingStatus

[源码:87](../../../desktop/audio/src/runtime.rs#L87) · struct

```rust
pub struct BindingStatus {
    pub desired: Origin,
    pub effective: Option<Origin>,
    pub preparation: Preparation,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_audio::runtime::BindingStatus`

声明引用：`skin_editor_audio::Origin`, `skin_editor_audio::Preparation`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_audio::BindingStatus::clone`, `skin_editor_audio::BindingStatus::eq`, `skin_editor_audio::Runtime::library_status`, `skin_editor_audio::Runtime::status`, `skin_editor_infrastructure::AudioPlayback::status`, `skin_editor_infrastructure::SupplyStatus (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for BindingStatus` — [源码:86](../../../desktop/audio/src/runtime.rs#L86)
- `impl Debug for BindingStatus` — [源码:86](../../../desktop/audio/src/runtime.rs#L86)
- `impl PartialEq for BindingStatus` — [源码:86](../../../desktop/audio/src/runtime.rs#L86)
- `impl Copy for BindingStatus` — [源码:86](../../../desktop/audio/src/runtime.rs#L86)
- `impl Eq for BindingStatus` — [源码:86](../../../desktop/audio/src/runtime.rs#L86)
- `impl StructuralPartialEq for BindingStatus` — [源码:86](../../../desktop/audio/src/runtime.rs#L86)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> BindingStatus` | impl Clone for BindingStatus | [源码:86](../../../desktop/audio/src/runtime.rs#L86) | skin_editor_audio::BindingStatus |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for BindingStatus | [源码:86](../../../desktop/audio/src/runtime.rs#L86) |  |
| `fn eq(&self, other: &BindingStatus) -> bool` | impl PartialEq for BindingStatus | [源码:86](../../../desktop/audio/src/runtime.rs#L86) | skin_editor_audio::BindingStatus |

## skin_editor_audio::BuildRequest

[源码:43](../../../desktop/audio/src/runtime.rs#L43) · struct

```rust
pub struct BuildRequest {
    pub origin: Origin,
    pub plan: SoundPlan,
    serial: u64,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_audio::runtime::BuildRequest`

声明引用：`skin_editor_audio::Origin`, `skin_editor_audio::SoundPlan`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_audio::BuildRequest::clone`, `skin_editor_audio::BuildRequest::eq`, `skin_editor_audio::Runtime::complete`, `skin_editor_audio::Runtime::request_library`, `skin_editor_audio::Runtime::retry`, `skin_editor_audio::Runtime::synchronize`, `skin_editor_audio::Runtime::synchronize_cancelled`, `skin_editor_audio::runtime::Binding (声明)`, `skin_editor_infrastructure::AudioPreparationQueue::finish`, `skin_editor_infrastructure::AudioPreparationQueue::submit`, `skin_editor_infrastructure::BuildAdmission (声明)`, `skin_editor_infrastructure::FinishedPreparation (声明)`, `skin_editor_infrastructure::PreparationObservation (声明)`, `skin_editor_infrastructure::RunningPreparation::request`, `skin_editor_infrastructure::SkinAudioController (声明)`, `skin_editor_infrastructure::SkinAudioController::accept_prepared`, `skin_editor_infrastructure::SkinAudioController::take_builds`, `skin_editor_infrastructure::audio_preparation::Active (声明)`, `skin_editor_infrastructure::audio_preparation::Pending (声明)`, `skin_editor_infrastructure::audio_preparation::PreparationResult (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for BuildRequest` — [源码:42](../../../desktop/audio/src/runtime.rs#L42)
- `impl Debug for BuildRequest` — [源码:42](../../../desktop/audio/src/runtime.rs#L42)
- `impl PartialEq for BuildRequest` — [源码:42](../../../desktop/audio/src/runtime.rs#L42)
- `impl Copy for BuildRequest` — [源码:42](../../../desktop/audio/src/runtime.rs#L42)
- `impl StructuralPartialEq for BuildRequest` — [源码:42](../../../desktop/audio/src/runtime.rs#L42)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> BuildRequest` | impl Clone for BuildRequest | [源码:42](../../../desktop/audio/src/runtime.rs#L42) | skin_editor_audio::BuildRequest |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for BuildRequest | [源码:42](../../../desktop/audio/src/runtime.rs#L42) |  |
| `fn eq(&self, other: &BuildRequest) -> bool` | impl PartialEq for BuildRequest | [源码:42](../../../desktop/audio/src/runtime.rs#L42) | skin_editor_audio::BuildRequest |

## skin_editor_audio::Channel

[源码:22](../../../desktop/audio/src/runtime.rs#L22) · enum

```rust
pub enum Channel {
    Scene,
    Audition,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_audio::runtime::Channel`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_audio::Channel::clone`, `skin_editor_audio::Channel::eq`, `skin_editor_audio::GateToken (声明)`, `skin_editor_audio::Runtime::clear_lane`, `skin_editor_audio::Runtime::token`, `skin_editor_audio::VoiceInfo (声明)`, `skin_editor_audio::runtime::Voice (声明)`, `skin_editor_desktop::editor_audio::combined::Run::trigger`, `skin_editor_desktop::editor_audio::combined::probes::Complete (声明)`, `skin_editor_desktop::editor_audio::combined::probes::Pending (声明)`, `skin_editor_desktop::editor_audio::combined::probes::Probes::begin`, `skin_editor_desktop::editor_audio::cycles::Round::trigger`, `skin_editor_desktop::editor_audio::gui_verification::Probe (声明)`, `skin_editor_desktop::editor_audio::window_recovery::sound::trigger (声明)`, `skin_editor_desktop::separated_verification::trigger (声明)`, `skin_editor_desktop::supply_verification::trigger (声明)`, `skin_editor_desktop::worker_verification::Probe (声明)`, `skin_editor_infrastructure::AudioPlayback::trigger`, `skin_editor_infrastructure::AudioSupplyClient::diagnostic_recovery_probe`, `skin_editor_infrastructure::AudioSupplyClient::trigger`, `skin_editor_infrastructure::AudioSupplyClient::trigger_batch_selected`, `skin_editor_infrastructure::AudioSupplyClient::trigger_selected`, `skin_editor_infrastructure::audio_supply::Control (声明)`, `skin_editor_infrastructure::separated_native::FirstCopy (声明)`, `skin_editor_infrastructure::voice_handoff::HandoffReport (声明)`, `skin_editor_infrastructure::voice_handoff::VoiceOwner::token`, `skin_editor_infrastructure::voice_handoff::VoiceToken (声明)`, `skin_editor_infrastructure::voice_handoff::epoch (声明)`, `skin_editor_infrastructure::voice_handoff::index (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Channel` — [源码:26](../../../desktop/audio/src/runtime.rs#L26)
- `impl Clone for Channel` — [源码:21](../../../desktop/audio/src/runtime.rs#L21)
- `impl Debug for Channel` — [源码:21](../../../desktop/audio/src/runtime.rs#L21)
- `impl PartialEq for Channel` — [源码:21](../../../desktop/audio/src/runtime.rs#L21)
- `impl Copy for Channel` — [源码:21](../../../desktop/audio/src/runtime.rs#L21)
- `impl Eq for Channel` — [源码:21](../../../desktop/audio/src/runtime.rs#L21)
- `impl StructuralPartialEq for Channel` — [源码:21](../../../desktop/audio/src/runtime.rs#L21)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn index(self) -> usize` | impl Channel | [源码:27](../../../desktop/audio/src/runtime.rs#L27) |  |
| `fn clone(&self) -> Channel` | impl Clone for Channel | [源码:21](../../../desktop/audio/src/runtime.rs#L21) | skin_editor_audio::Channel |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for Channel | [源码:21](../../../desktop/audio/src/runtime.rs#L21) |  |
| `fn eq(&self, other: &Channel) -> bool` | impl PartialEq for Channel | [源码:21](../../../desktop/audio/src/runtime.rs#L21) | skin_editor_audio::Channel |

## skin_editor_audio::Completion

[源码:75](../../../desktop/audio/src/runtime.rs#L75) · enum

```rust
pub enum Completion {
    Activated {
        effective_from_mix_frame: u64,
    },
    Failed(PrepareFailure),
    Stale,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_audio::runtime::Completion`

声明引用：`skin_editor_audio::PrepareFailure`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_audio::Completion::clone`, `skin_editor_audio::Completion::eq`, `skin_editor_audio::Runtime::complete`, `skin_editor_infrastructure::AdmissionState (声明)`, `skin_editor_infrastructure::SkinAudioController::accept_prepared`, `skin_editor_infrastructure::SkinAudioController::finish_preparation`, `skin_editor_infrastructure::SkinAudioController::finish_selected`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for Completion` — [源码:74](../../../desktop/audio/src/runtime.rs#L74)
- `impl Debug for Completion` — [源码:74](../../../desktop/audio/src/runtime.rs#L74)
- `impl PartialEq for Completion` — [源码:74](../../../desktop/audio/src/runtime.rs#L74)
- `impl Copy for Completion` — [源码:74](../../../desktop/audio/src/runtime.rs#L74)
- `impl Eq for Completion` — [源码:74](../../../desktop/audio/src/runtime.rs#L74)
- `impl StructuralPartialEq for Completion` — [源码:74](../../../desktop/audio/src/runtime.rs#L74)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> Completion` | impl Clone for Completion | [源码:74](../../../desktop/audio/src/runtime.rs#L74) | skin_editor_audio::Completion |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for Completion | [源码:74](../../../desktop/audio/src/runtime.rs#L74) |  |
| `fn eq(&self, other: &Completion) -> bool` | impl PartialEq for Completion | [源码:74](../../../desktop/audio/src/runtime.rs#L74) | skin_editor_audio::Completion |

## skin_editor_audio::GateToken

[源码:36](../../../desktop/audio/src/runtime.rs#L36) · struct

```rust
pub struct GateToken {
    device: u64,
    channel: u64,
    selection: u64,
    lane: Channel,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_audio::runtime::GateToken`

声明引用：`skin_editor_audio::Channel`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_audio::GateToken::clone`, `skin_editor_audio::GateToken::eq`, `skin_editor_audio::Runtime::seek_scene`, `skin_editor_audio::Runtime::set_scene_running`, `skin_editor_audio::Runtime::stop_audition`, `skin_editor_audio::Runtime::token`, `skin_editor_audio::Runtime::trigger`, `skin_editor_audio::Runtime::valid`, `skin_editor_audio::runtime::Event (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for GateToken` — [源码:35](../../../desktop/audio/src/runtime.rs#L35)
- `impl Debug for GateToken` — [源码:35](../../../desktop/audio/src/runtime.rs#L35)
- `impl PartialEq for GateToken` — [源码:35](../../../desktop/audio/src/runtime.rs#L35)
- `impl Copy for GateToken` — [源码:35](../../../desktop/audio/src/runtime.rs#L35)
- `impl Eq for GateToken` — [源码:35](../../../desktop/audio/src/runtime.rs#L35)
- `impl StructuralPartialEq for GateToken` — [源码:35](../../../desktop/audio/src/runtime.rs#L35)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> GateToken` | impl Clone for GateToken | [源码:35](../../../desktop/audio/src/runtime.rs#L35) | skin_editor_audio::GateToken |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for GateToken | [源码:35](../../../desktop/audio/src/runtime.rs#L35) |  |
| `fn eq(&self, other: &GateToken) -> bool` | impl PartialEq for GateToken | [源码:35](../../../desktop/audio/src/runtime.rs#L35) | skin_editor_audio::GateToken |

## skin_editor_audio::MAX_BINDINGS

[源码:8](../../../desktop/audio/src/runtime.rs#L8) · constant

```rust
pub const MAX_BINDINGS: usize = 256;
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_audio::runtime::MAX_BINDINGS`

## skin_editor_audio::MAX_EVENTS

[源码:7](../../../desktop/audio/src/runtime.rs#L7) · constant

```rust
pub const MAX_EVENTS: usize = 64;
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_audio::runtime::MAX_EVENTS`

## skin_editor_audio::MAX_SOUND_SAMPLES

[源码:9](../../../desktop/audio/src/runtime.rs#L9) · constant

```rust
pub const MAX_SOUND_SAMPLES: usize = _; // 1_048_576usize
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_audio::runtime::MAX_SOUND_SAMPLES`

## skin_editor_audio::MAX_VOICES

[源码:6](../../../desktop/audio/src/runtime.rs#L6) · constant

```rust
pub const MAX_VOICES: usize = 32;
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_audio::runtime::MAX_VOICES`

## skin_editor_audio::MIX_CONVERSION_VERSION

[源码:11](../../../desktop/audio/src/runtime.rs#L11) · constant

```rust
pub const MIX_CONVERSION_VERSION: u16 = 1;
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_audio::runtime::MIX_CONVERSION_VERSION`

## skin_editor_audio::MixReport

[源码:273](../../../desktop/audio/src/runtime.rs#L273) · struct

```rust
pub struct MixReport {
    pub started: u32,
    pub dropped: u32,
    pub clipped_samples: u32,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_audio::runtime::MixReport`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_audio::MixReport::clone`, `skin_editor_audio::MixReport::default`, `skin_editor_audio::MixReport::eq`, `skin_editor_audio::Runtime::mix`, `skin_editor_audio::Runtime::mix_buffer`, `skin_editor_audio::Runtime::mix_frames`, `skin_editor_audio::Runtime::mix_split_buffers`, `skin_editor_audio::Runtime::mix_split_buffers_marked`, `skin_editor_audio::Runtime::split_buffers`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for MixReport` — [源码:272](../../../desktop/audio/src/runtime.rs#L272)
- `impl Debug for MixReport` — [源码:272](../../../desktop/audio/src/runtime.rs#L272)
- `impl Default for MixReport` — [源码:272](../../../desktop/audio/src/runtime.rs#L272)
- `impl PartialEq for MixReport` — [源码:272](../../../desktop/audio/src/runtime.rs#L272)
- `impl Copy for MixReport` — [源码:272](../../../desktop/audio/src/runtime.rs#L272)
- `impl Eq for MixReport` — [源码:272](../../../desktop/audio/src/runtime.rs#L272)
- `impl StructuralPartialEq for MixReport` — [源码:272](../../../desktop/audio/src/runtime.rs#L272)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> MixReport` | impl Clone for MixReport | [源码:272](../../../desktop/audio/src/runtime.rs#L272) | skin_editor_audio::MixReport |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for MixReport | [源码:272](../../../desktop/audio/src/runtime.rs#L272) |  |
| `fn default() -> MixReport` | impl Default for MixReport | [源码:272](../../../desktop/audio/src/runtime.rs#L272) | skin_editor_audio::MixReport |
| `fn eq(&self, other: &MixReport) -> bool` | impl PartialEq for MixReport | [源码:272](../../../desktop/audio/src/runtime.rs#L272) | skin_editor_audio::MixReport |

## skin_editor_audio::MonitorFormat

[源码:9](../../../desktop/audio/src/voice.rs#L9) · struct

```rust
pub struct MonitorFormat {
    rate: u32,
    channels: u16,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_audio::voice::MonitorFormat`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_audio::MonitorFormat::clone`, `skin_editor_audio::MonitorFormat::eq`, `skin_editor_audio::PreparedSound::add_monitor_frames`, `skin_editor_audio::PreparedSound::add_monitor_frames_with_gain`, `skin_editor_audio::PreparedSound::monitor_finished`, `skin_editor_audio::PreparedSound::monitor_frame`, `skin_editor_audio::PreparedSound::monitor_frame_with_gain`, `skin_editor_infrastructure::voice_handoff::VoiceConsumer (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl MonitorFormat` — [源码:13](../../../desktop/audio/src/voice.rs#L13)
- `impl Clone for MonitorFormat` — [源码:8](../../../desktop/audio/src/voice.rs#L8)
- `impl Debug for MonitorFormat` — [源码:8](../../../desktop/audio/src/voice.rs#L8)
- `impl PartialEq for MonitorFormat` — [源码:8](../../../desktop/audio/src/voice.rs#L8)
- `impl Copy for MonitorFormat` — [源码:8](../../../desktop/audio/src/voice.rs#L8)
- `impl Eq for MonitorFormat` — [源码:8](../../../desktop/audio/src/voice.rs#L8)
- `impl StructuralPartialEq for MonitorFormat` — [源码:8](../../../desktop/audio/src/voice.rs#L8)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn rate(self) -> u32` | impl MonitorFormat | [源码:14](../../../desktop/audio/src/voice.rs#L14) |  |
| `pub fn new(rate: u32, channels: u16) -> Option<Self>` | impl MonitorFormat | [源码:17](../../../desktop/audio/src/voice.rs#L17) |  |
| `fn clone(&self) -> MonitorFormat` | impl Clone for MonitorFormat | [源码:8](../../../desktop/audio/src/voice.rs#L8) | skin_editor_audio::MonitorFormat |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for MonitorFormat | [源码:8](../../../desktop/audio/src/voice.rs#L8) |  |
| `fn eq(&self, other: &MonitorFormat) -> bool` | impl PartialEq for MonitorFormat | [源码:8](../../../desktop/audio/src/voice.rs#L8) | skin_editor_audio::MonitorFormat |

## skin_editor_audio::Origin

[源码:14](../../../desktop/audio/src/runtime.rs#L14) · struct

```rust
pub struct Origin {
    pub session: u128,
    pub node: u64,
    pub revision: u64,
    pub navigation_revision: u64,
    pub draft: Option<(u64, u64)>,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_audio::runtime::Origin`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_audio::AuditionEdits (声明)`, `skin_editor_audio::AuditionEdits::select`, `skin_editor_audio::BindingStatus (声明)`, `skin_editor_audio::BuildRequest (声明)`, `skin_editor_audio::Origin::clone`, `skin_editor_audio::Origin::eq`, `skin_editor_audio::Runtime (声明)`, `skin_editor_audio::Runtime::synchronize`, `skin_editor_audio::Runtime::synchronize_cancelled`, `skin_editor_audio::TriggerReceipt (声明)`, `skin_editor_audio::VoiceInfo (声明)`, `skin_editor_audio::runtime::Effective (声明)`, `skin_editor_desktop::edit_verification::Capture (声明)`, `skin_editor_desktop::edit_verification::ready (声明)`, `skin_editor_desktop::edit_verification::source (声明)`, `skin_editor_desktop::editor_audio::combined::probes::Pending (声明)`, `skin_editor_desktop::editor_audio::combined::probes::Probes::begin`, `skin_editor_desktop::editor_audio::source (声明)`, `skin_editor_desktop::separated_verification::origin (声明)`, `skin_editor_infrastructure::AudioPreparationQueue::cancel_outdated`, `skin_editor_infrastructure::SkinAudioController (声明)`, `skin_editor_infrastructure::SkinAudioController::select`, `skin_editor_infrastructure::SupplyStatus (声明)`, `skin_editor_infrastructure::audio_supply::Control (声明)`, `skin_editor_infrastructure::audio_supply::Projection (声明)`, `skin_editor_infrastructure::audio_supply::SupplyCore (声明)`, `skin_editor_infrastructure::audio_supply::origin (声明)`, `skin_editor_infrastructure::separated_output::boundary (声明)`, `skin_editor_infrastructure::separated_output::boundary_input (声明)`, `skin_editor_infrastructure::voice_handoff::VoiceOwner (声明)`, `skin_editor_infrastructure::voice_handoff::VoiceOwner::admit`, `skin_editor_infrastructure::voice_handoff::VoiceOwner::invalidate_source`, `skin_editor_infrastructure::voice_handoff::VoiceOwner::publish`, `skin_editor_infrastructure::voice_handoff::boundary (声明)`, `skin_editor_infrastructure::voice_handoff::valid_origin (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for Origin` — [源码:13](../../../desktop/audio/src/runtime.rs#L13)
- `impl Debug for Origin` — [源码:13](../../../desktop/audio/src/runtime.rs#L13)
- `impl PartialEq for Origin` — [源码:13](../../../desktop/audio/src/runtime.rs#L13)
- `impl Copy for Origin` — [源码:13](../../../desktop/audio/src/runtime.rs#L13)
- `impl Eq for Origin` — [源码:13](../../../desktop/audio/src/runtime.rs#L13)
- `impl StructuralPartialEq for Origin` — [源码:13](../../../desktop/audio/src/runtime.rs#L13)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> Origin` | impl Clone for Origin | [源码:13](../../../desktop/audio/src/runtime.rs#L13) | skin_editor_audio::Origin |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for Origin | [源码:13](../../../desktop/audio/src/runtime.rs#L13) |  |
| `fn eq(&self, other: &Origin) -> bool` | impl PartialEq for Origin | [源码:13](../../../desktop/audio/src/runtime.rs#L13) | skin_editor_audio::Origin |

## skin_editor_audio::OutputAction

[源码:44](../../../desktop/audio/src/lib.rs#L44) · enum

```rust
pub enum OutputAction {
    CopyOriginal(BlobRef),
    ProcessPcm,
    EncodeSilence,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_audio::OutputAction::clone`, `skin_editor_audio::OutputAction::eq`, `skin_editor_audio::SoundPlan::output_action`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for OutputAction` — [源码:43](../../../desktop/audio/src/lib.rs#L43)
- `impl Debug for OutputAction` — [源码:43](../../../desktop/audio/src/lib.rs#L43)
- `impl PartialEq for OutputAction` — [源码:43](../../../desktop/audio/src/lib.rs#L43)
- `impl Copy for OutputAction` — [源码:43](../../../desktop/audio/src/lib.rs#L43)
- `impl StructuralPartialEq for OutputAction` — [源码:43](../../../desktop/audio/src/lib.rs#L43)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> OutputAction` | impl Clone for OutputAction | [源码:43](../../../desktop/audio/src/lib.rs#L43) | skin_editor_audio::OutputAction |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for OutputAction | [源码:43](../../../desktop/audio/src/lib.rs#L43) |  |
| `fn eq(&self, other: &OutputAction) -> bool` | impl PartialEq for OutputAction | [源码:43](../../../desktop/audio/src/lib.rs#L43) | skin_editor_audio::OutputAction |

## skin_editor_audio::PcmBudget

[源码:26](../../../desktop/audio/src/pcm.rs#L26) · struct

```rust
pub struct PcmBudget(Arc<Budget>);
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_audio::pcm::PcmBudget`

声明引用：`skin_editor_audio::pcm::Budget`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_audio::PcmBudget::clone`, `skin_editor_audio::PcmBuffer::budget`, `skin_editor_audio::PcmReservation (声明)`, `skin_editor_audio::PreparedSound::from_source`, `skin_editor_audio::PreparedWeak::upgrade_for`, `skin_editor_audio::Runtime (声明)`, `skin_editor_audio::Runtime::pcm_budget`, `skin_editor_audio::Runtime::with_budget`, `skin_editor_desktop::combined_verification::Combined (声明)`, `skin_editor_desktop::edit_verification::settled_pcm (声明)`, `skin_editor_desktop::editor_audio::EditorAudio (声明)`, `skin_editor_desktop::editor_audio::cycles::Counters (声明)`, `skin_editor_desktop::separated_verification::sound (声明)`, `skin_editor_desktop::stream_fixture::open (声明)`, `skin_editor_infrastructure::AudioPlayback::new`, `skin_editor_infrastructure::AudioPreparationQueue (声明)`, `skin_editor_infrastructure::AudioPreparationQueue::new`, `skin_editor_infrastructure::AudioSupplyClient::spawn`, `skin_editor_infrastructure::AudioSupplyClient::spawn_separated`, `skin_editor_infrastructure::AudioSupplyClient::spawn_streamed`, `skin_editor_infrastructure::NativeOutput::open`, `skin_editor_infrastructure::SkinAudioController::with_budget`, `skin_editor_infrastructure::audio_supply::SupplyCore::new`, `skin_editor_infrastructure::audio_supply::SupplyCore::new_separated`, `skin_editor_infrastructure::background_input::BackgroundInput::budget`, `skin_editor_infrastructure::background_stream::BufferedBackground (声明)`, `skin_editor_infrastructure::background_stream::BufferedBackground::budget`, `skin_editor_infrastructure::background_stream::BufferedBackground::open`, `skin_editor_infrastructure::background_stream::BufferedBackground::spawn`, `skin_editor_infrastructure::background_stream::BufferedBackground::spawn_inner`, `skin_editor_infrastructure::output_boundary (声明)`, `skin_editor_infrastructure::voice_handoff::VoiceOwner (声明)`, `skin_editor_infrastructure::voice_handoff::boundary (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl PcmBudget` — [源码:32](../../../desktop/audio/src/pcm.rs#L32)
- `impl Clone for PcmBudget` — [源码:25](../../../desktop/audio/src/pcm.rs#L25)
- `impl Debug for PcmBudget` — [源码:25](../../../desktop/audio/src/pcm.rs#L25)
- `impl Default for PcmBudget` — [源码:27](../../../desktop/audio/src/pcm.rs#L27)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn new(limit: usize, per_sound: usize) -> Result<Self, PcmError>` | impl PcmBudget | [源码:33](../../../desktop/audio/src/pcm.rs#L33) | skin_editor_audio::PcmError |
| `pub fn reserve(&self, samples: usize) -> Result<PcmReservation, PcmError>` | impl PcmBudget | [源码:44](../../../desktop/audio/src/pcm.rs#L44) | skin_editor_audio::PcmError, skin_editor_audio::PcmReservation |
| `pub fn usage(&self) -> PcmUsage` | impl PcmBudget | [源码:62](../../../desktop/audio/src/pcm.rs#L62) | skin_editor_audio::PcmUsage |
| `pub fn same_pool(&self, other: &Self) -> bool` | impl PcmBudget | [源码:69](../../../desktop/audio/src/pcm.rs#L69) |  |
| `fn clone(&self) -> PcmBudget` | impl Clone for PcmBudget | [源码:25](../../../desktop/audio/src/pcm.rs#L25) | skin_editor_audio::PcmBudget |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for PcmBudget | [源码:25](../../../desktop/audio/src/pcm.rs#L25) |  |
| `fn default() -> Self` | impl Default for PcmBudget | [源码:28](../../../desktop/audio/src/pcm.rs#L28) |  |

## skin_editor_audio::PcmBuffer

[源码:101](../../../desktop/audio/src/pcm.rs#L101) · struct

```rust
pub struct PcmBuffer(Arc<Allocation>);
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_audio::pcm::PcmBuffer`

声明引用：`skin_editor_audio::pcm::Allocation`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_audio::BackgroundTrack (声明)`, `skin_editor_audio::BackgroundTrack::new`, `skin_editor_audio::BackgroundTrack::pcm`, `skin_editor_audio::PcmBuffer::clone`, `skin_editor_audio::PcmReservation::allocate`, `skin_editor_audio::PcmWeak::upgrade`, `skin_editor_audio::PreparedSound (声明)`, `skin_editor_audio::PreparedSound::from_decoded`, `skin_editor_audio::PreparedSound::pcm`, `skin_editor_audio::Runtime::mix_buffer`, `skin_editor_audio::Runtime::mix_split_buffers`, `skin_editor_audio::Runtime::mix_split_buffers_marked`, `skin_editor_audio::Runtime::split_buffers`, `skin_editor_infrastructure::OutputProducer (声明)`, `skin_editor_infrastructure::background_stream::Block (声明)`, `skin_editor_infrastructure::separated_output::OutputConsumer (声明)`, `skin_editor_infrastructure::voice_handoff::Pin (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl PcmBuffer` — [源码:102](../../../desktop/audio/src/pcm.rs#L102)
- `impl Clone for PcmBuffer` — [源码:100](../../../desktop/audio/src/pcm.rs#L100)
- `impl Debug for PcmBuffer` — [源码:100](../../../desktop/audio/src/pcm.rs#L100)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn samples(&self) -> &[f32]` | impl PcmBuffer | [源码:103](../../../desktop/audio/src/pcm.rs#L103) |  |
| `pub fn samples_mut(&mut self) -> Result<&mut [f32], PcmError>` | impl PcmBuffer | [源码:106](../../../desktop/audio/src/pcm.rs#L106) | skin_editor_audio::PcmError |
| `pub fn budget(&self) -> &PcmBudget` | impl PcmBuffer | [源码:111](../../../desktop/audio/src/pcm.rs#L111) | skin_editor_audio::PcmBudget |
| `pub fn same_allocation(&self, other: &Self) -> bool` | impl PcmBuffer | [源码:114](../../../desktop/audio/src/pcm.rs#L114) |  |
| `pub fn downgrade(&self) -> PcmWeak` | impl PcmBuffer | [源码:117](../../../desktop/audio/src/pcm.rs#L117) | skin_editor_audio::PcmWeak |
| `fn clone(&self) -> PcmBuffer` | impl Clone for PcmBuffer | [源码:100](../../../desktop/audio/src/pcm.rs#L100) | skin_editor_audio::PcmBuffer |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for PcmBuffer | [源码:100](../../../desktop/audio/src/pcm.rs#L100) |  |

## skin_editor_audio::PcmError

[源码:14](../../../desktop/audio/src/pcm.rs#L14) · enum

```rust
pub enum PcmError {
    ResourceLimit,
    SharedBuffer,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_audio::pcm::PcmError`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_audio::BackgroundTrack::new`, `skin_editor_audio::BackgroundTrack::restore_position`, `skin_editor_audio::PcmBudget::new`, `skin_editor_audio::PcmBudget::reserve`, `skin_editor_audio::PcmBuffer::samples_mut`, `skin_editor_audio::PcmError::clone`, `skin_editor_audio::PcmError::eq`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for PcmError` — [源码:13](../../../desktop/audio/src/pcm.rs#L13)
- `impl Debug for PcmError` — [源码:13](../../../desktop/audio/src/pcm.rs#L13)
- `impl PartialEq for PcmError` — [源码:13](../../../desktop/audio/src/pcm.rs#L13)
- `impl Copy for PcmError` — [源码:13](../../../desktop/audio/src/pcm.rs#L13)
- `impl Eq for PcmError` — [源码:13](../../../desktop/audio/src/pcm.rs#L13)
- `impl StructuralPartialEq for PcmError` — [源码:13](../../../desktop/audio/src/pcm.rs#L13)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> PcmError` | impl Clone for PcmError | [源码:13](../../../desktop/audio/src/pcm.rs#L13) | skin_editor_audio::PcmError |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for PcmError | [源码:13](../../../desktop/audio/src/pcm.rs#L13) |  |
| `fn eq(&self, other: &PcmError) -> bool` | impl PartialEq for PcmError | [源码:13](../../../desktop/audio/src/pcm.rs#L13) | skin_editor_audio::PcmError |

## skin_editor_audio::PcmReservation

[源码:74](../../../desktop/audio/src/pcm.rs#L74) · struct

```rust
pub struct PcmReservation {
    budget: PcmBudget,
    samples: usize,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_audio::pcm::PcmReservation`

声明引用：`skin_editor_audio::PcmBudget`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_audio::PcmBudget::reserve`, `skin_editor_audio::pcm::Allocation (声明)`, `skin_editor_infrastructure::audio_preparation::Pending (声明)`, `skin_editor_infrastructure::native_audio::Shared (声明)`, `skin_editor_infrastructure::separated_output::Shared (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl PcmReservation` — [源码:78](../../../desktop/audio/src/pcm.rs#L78)
- `impl Debug for PcmReservation` — [源码:73](../../../desktop/audio/src/pcm.rs#L73)
- `impl Drop for PcmReservation` — [源码:87](../../../desktop/audio/src/pcm.rs#L87)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn allocate(self) -> PcmBuffer` | impl PcmReservation | [源码:79](../../../desktop/audio/src/pcm.rs#L79) | skin_editor_audio::PcmBuffer |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for PcmReservation | [源码:73](../../../desktop/audio/src/pcm.rs#L73) |  |
| `fn drop(&mut self)` | impl Drop for PcmReservation | [源码:88](../../../desktop/audio/src/pcm.rs#L88) |  |

## skin_editor_audio::PcmUsage

[源码:8](../../../desktop/audio/src/pcm.rs#L8) · struct

```rust
pub struct PcmUsage {
    pub bytes: usize,
    pub peak_bytes: usize,
    pub limit: usize,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_audio::pcm::PcmUsage`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_audio::PcmBudget::usage`, `skin_editor_audio::PcmUsage::clone`, `skin_editor_audio::PcmUsage::eq`, `skin_editor_infrastructure::PreparationUsage (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for PcmUsage` — [源码:7](../../../desktop/audio/src/pcm.rs#L7)
- `impl Debug for PcmUsage` — [源码:7](../../../desktop/audio/src/pcm.rs#L7)
- `impl PartialEq for PcmUsage` — [源码:7](../../../desktop/audio/src/pcm.rs#L7)
- `impl Copy for PcmUsage` — [源码:7](../../../desktop/audio/src/pcm.rs#L7)
- `impl Eq for PcmUsage` — [源码:7](../../../desktop/audio/src/pcm.rs#L7)
- `impl StructuralPartialEq for PcmUsage` — [源码:7](../../../desktop/audio/src/pcm.rs#L7)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> PcmUsage` | impl Clone for PcmUsage | [源码:7](../../../desktop/audio/src/pcm.rs#L7) | skin_editor_audio::PcmUsage |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for PcmUsage | [源码:7](../../../desktop/audio/src/pcm.rs#L7) |  |
| `fn eq(&self, other: &PcmUsage) -> bool` | impl PartialEq for PcmUsage | [源码:7](../../../desktop/audio/src/pcm.rs#L7) | skin_editor_audio::PcmUsage |

## skin_editor_audio::PcmWeak

[源码:122](../../../desktop/audio/src/pcm.rs#L122) · struct

```rust
pub struct PcmWeak(Weak<Allocation>);
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_audio::pcm::PcmWeak`

声明引用：`skin_editor_audio::pcm::Allocation`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_audio::PcmBuffer::downgrade`, `skin_editor_audio::PcmWeak::clone`, `skin_editor_audio::PreparedWeak (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl PcmWeak` — [源码:123](../../../desktop/audio/src/pcm.rs#L123)
- `impl Clone for PcmWeak` — [源码:121](../../../desktop/audio/src/pcm.rs#L121)
- `impl Debug for PcmWeak` — [源码:121](../../../desktop/audio/src/pcm.rs#L121)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn is_alive(&self) -> bool` | impl PcmWeak | [源码:124](../../../desktop/audio/src/pcm.rs#L124) |  |
| `pub(crate) fn upgrade(&self) -> Option<PcmBuffer>` | impl PcmWeak | [源码:127](../../../desktop/audio/src/pcm.rs#L127) | skin_editor_audio::PcmBuffer |
| `fn clone(&self) -> PcmWeak` | impl Clone for PcmWeak | [源码:121](../../../desktop/audio/src/pcm.rs#L121) | skin_editor_audio::PcmWeak |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for PcmWeak | [源码:121](../../../desktop/audio/src/pcm.rs#L121) |  |

## skin_editor_audio::Preparation

[源码:81](../../../desktop/audio/src/runtime.rs#L81) · enum

```rust
pub enum Preparation {
    Pending,
    Ready,
    Failed(PrepareFailure),
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_audio::runtime::Preparation`

声明引用：`skin_editor_audio::PrepareFailure`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_audio::BindingStatus (声明)`, `skin_editor_audio::Preparation::clone`, `skin_editor_audio::Preparation::eq`, `skin_editor_audio::runtime::Binding (声明)`, `skin_editor_desktop::editor_audio::preparation (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for Preparation` — [源码:80](../../../desktop/audio/src/runtime.rs#L80)
- `impl Debug for Preparation` — [源码:80](../../../desktop/audio/src/runtime.rs#L80)
- `impl PartialEq for Preparation` — [源码:80](../../../desktop/audio/src/runtime.rs#L80)
- `impl Copy for Preparation` — [源码:80](../../../desktop/audio/src/runtime.rs#L80)
- `impl Eq for Preparation` — [源码:80](../../../desktop/audio/src/runtime.rs#L80)
- `impl StructuralPartialEq for Preparation` — [源码:80](../../../desktop/audio/src/runtime.rs#L80)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> Preparation` | impl Clone for Preparation | [源码:80](../../../desktop/audio/src/runtime.rs#L80) | skin_editor_audio::Preparation |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for Preparation | [源码:80](../../../desktop/audio/src/runtime.rs#L80) |  |
| `fn eq(&self, other: &Preparation) -> bool` | impl PartialEq for Preparation | [源码:80](../../../desktop/audio/src/runtime.rs#L80) | skin_editor_audio::Preparation |

## skin_editor_audio::PrepareFailure

[源码:49](../../../desktop/audio/src/runtime.rs#L49) · enum

```rust
pub enum PrepareFailure {
    Busy,
    Cancelled,
    MetadataMismatch,
    Unsupported,
    InvalidPcm,
    FormatMismatch,
    ResourceLimit,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_audio::runtime::PrepareFailure`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_audio::Completion (声明)`, `skin_editor_audio::Preparation (声明)`, `skin_editor_audio::PrepareFailure::clone`, `skin_editor_audio::PrepareFailure::eq`, `skin_editor_audio::PreparedKey::new`, `skin_editor_audio::PreparedSound::from_decoded`, `skin_editor_audio::PreparedSound::from_source`, `skin_editor_audio::Runtime::complete`, `skin_editor_audio::Runtime::fail_library`, `skin_editor_infrastructure::AudioDecoder (声明)`, `skin_editor_infrastructure::AudioDecoder::decode_into`, `skin_editor_infrastructure::AudioDecoder::probe`, `skin_editor_infrastructure::AudioPreparationQueue::finish`, `skin_editor_infrastructure::FinishedPreparation (声明)`, `skin_editor_infrastructure::SkinAudioController::accept_prepared`, `skin_editor_infrastructure::WavDecoder::decode_into`, `skin_editor_infrastructure::WavDecoder::probe`, `skin_editor_infrastructure::audio_preparation::PreparationResult (声明)`, `skin_editor_infrastructure::wav::parse (声明)`, `skin_editor_infrastructure::wav::pcm16_format (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for PrepareFailure` — [源码:48](../../../desktop/audio/src/runtime.rs#L48)
- `impl Debug for PrepareFailure` — [源码:48](../../../desktop/audio/src/runtime.rs#L48)
- `impl PartialEq for PrepareFailure` — [源码:48](../../../desktop/audio/src/runtime.rs#L48)
- `impl Copy for PrepareFailure` — [源码:48](../../../desktop/audio/src/runtime.rs#L48)
- `impl Eq for PrepareFailure` — [源码:48](../../../desktop/audio/src/runtime.rs#L48)
- `impl StructuralPartialEq for PrepareFailure` — [源码:48](../../../desktop/audio/src/runtime.rs#L48)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> PrepareFailure` | impl Clone for PrepareFailure | [源码:48](../../../desktop/audio/src/runtime.rs#L48) | skin_editor_audio::PrepareFailure |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for PrepareFailure | [源码:48](../../../desktop/audio/src/runtime.rs#L48) |  |
| `fn eq(&self, other: &PrepareFailure) -> bool` | impl PartialEq for PrepareFailure | [源码:48](../../../desktop/audio/src/runtime.rs#L48) | skin_editor_audio::PrepareFailure |

## skin_editor_audio::PreparedKey

[源码:101](../../../desktop/audio/src/runtime.rs#L101) · struct

```rust
pub struct PreparedKey {
    blob: BlobRef,
    rate: u32,
    channels: u16,
    frames: u64,
    gain: u64,
    visible: bool,
    version: u16,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_audio::runtime::PreparedKey`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_audio::PreparedKey::clone`, `skin_editor_audio::PreparedKey::cmp`, `skin_editor_audio::PreparedKey::eq`, `skin_editor_audio::PreparedKey::partial_cmp`, `skin_editor_audio::PreparedSound::cache_key`, `skin_editor_audio::PreparedWeak (声明)`, `skin_editor_infrastructure::AudioPreparationQueue (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl PreparedKey` — [源码:110](../../../desktop/audio/src/runtime.rs#L110)
- `impl Clone for PreparedKey` — [源码:100](../../../desktop/audio/src/runtime.rs#L100)
- `impl Debug for PreparedKey` — [源码:100](../../../desktop/audio/src/runtime.rs#L100)
- `impl Ord for PreparedKey` — [源码:100](../../../desktop/audio/src/runtime.rs#L100)
- `impl PartialEq for PreparedKey` — [源码:100](../../../desktop/audio/src/runtime.rs#L100)
- `impl PartialOrd for PreparedKey` — [源码:100](../../../desktop/audio/src/runtime.rs#L100)
- `impl Copy for PreparedKey` — [源码:100](../../../desktop/audio/src/runtime.rs#L100)
- `impl Eq for PreparedKey` — [源码:100](../../../desktop/audio/src/runtime.rs#L100)
- `impl StructuralPartialEq for PreparedKey` — [源码:100](../../../desktop/audio/src/runtime.rs#L100)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn new(plan: SoundPlan) -> Result<Self, PrepareFailure>` | impl PreparedKey | [源码:111](../../../desktop/audio/src/runtime.rs#L111) | skin_editor_audio::PrepareFailure, skin_editor_audio::SoundPlan |
| `fn clone(&self) -> PreparedKey` | impl Clone for PreparedKey | [源码:100](../../../desktop/audio/src/runtime.rs#L100) | skin_editor_audio::PreparedKey |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for PreparedKey | [源码:100](../../../desktop/audio/src/runtime.rs#L100) |  |
| `fn cmp(&self, other: &PreparedKey) -> Ordering` | impl Ord for PreparedKey | [源码:100](../../../desktop/audio/src/runtime.rs#L100) | skin_editor_audio::PreparedKey |
| `fn eq(&self, other: &PreparedKey) -> bool` | impl PartialEq for PreparedKey | [源码:100](../../../desktop/audio/src/runtime.rs#L100) | skin_editor_audio::PreparedKey |
| `fn partial_cmp(&self, other: &PreparedKey) -> Option<Ordering>` | impl PartialOrd for PreparedKey | [源码:100](../../../desktop/audio/src/runtime.rs#L100) | skin_editor_audio::PreparedKey |

## skin_editor_audio::PreparedSound

[源码:94](../../../desktop/audio/src/runtime.rs#L94) · struct

```rust
pub struct PreparedSound {
    plan: SoundPlan,
    rate: u32,
    channels: u16,
    pcm: PcmBuffer,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_audio::runtime::PreparedSound`

声明引用：`skin_editor_audio::PcmBuffer`, `skin_editor_audio::SoundPlan`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_audio::PreparedSound::clone`, `skin_editor_audio::PreparedWeak::upgrade_for`, `skin_editor_audio::Runtime::complete`, `skin_editor_audio::Runtime::library_sound`, `skin_editor_audio::VoiceHandoff (声明)`, `skin_editor_audio::VoiceHandoff::sound`, `skin_editor_audio::runtime::Effective (声明)`, `skin_editor_infrastructure::Admission (声明)`, `skin_editor_infrastructure::AudioPreparationQueue::finish`, `skin_editor_infrastructure::FinishedPreparation (声明)`, `skin_editor_infrastructure::SkinAudioController::accept_prepared`, `skin_editor_infrastructure::audio_preparation::PreparationResult (声明)`, `skin_editor_infrastructure::voice_handoff::Packet (声明)`, `skin_editor_infrastructure::voice_handoff::VoiceOwner::admit`, `skin_editor_infrastructure::voice_handoff::VoiceOwner::publish`

实现／约束（显式或 derive，无类继承）：

- `impl PreparedSound` — [源码:149](../../../desktop/audio/src/runtime.rs#L149)
- `impl PreparedSound` — [源码:21](../../../desktop/audio/src/voice.rs#L21)
- `impl Clone for PreparedSound` — [源码:93](../../../desktop/audio/src/runtime.rs#L93)
- `impl Debug for PreparedSound` — [源码:93](../../../desktop/audio/src/runtime.rs#L93)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn from_source( plan: SoundPlan, samples: &[f32], budget: &PcmBudget, ) -> Result<Self, PrepareFailure>` | impl PreparedSound | [源码:152](../../../desktop/audio/src/runtime.rs#L152) | skin_editor_audio::PcmBudget, skin_editor_audio::PrepareFailure, skin_editor_audio::SoundPlan |
| `pub fn from_decoded( plan: SoundPlan, pcm: PcmBuffer, ) -> Result<Self, PrepareFailure>` | impl PreparedSound | [源码:169](../../../desktop/audio/src/runtime.rs#L169) | skin_editor_audio::PcmBuffer, skin_editor_audio::PrepareFailure, skin_editor_audio::SoundPlan |
| `pub fn samples(&self) -> &[f32]` | impl PreparedSound | [源码:195](../../../desktop/audio/src/runtime.rs#L195) |  |
| `pub fn pcm(&self) -> &PcmBuffer` | impl PreparedSound | [源码:198](../../../desktop/audio/src/runtime.rs#L198) | skin_editor_audio::PcmBuffer |
| `pub fn monitor_gain(&self) -> f32` | impl PreparedSound | [源码:201](../../../desktop/audio/src/runtime.rs#L201) |  |
| `pub fn format(&self) -> (u32, u16)` | impl PreparedSound | [源码:209](../../../desktop/audio/src/runtime.rs#L209) |  |
| `pub fn cache_key(&self) -> PreparedKey` | impl PreparedSound | [源码:212](../../../desktop/audio/src/runtime.rs#L212) | skin_editor_audio::PreparedKey |
| `pub fn downgrade(&self) -> PreparedWeak` | impl PreparedSound | [源码:215](../../../desktop/audio/src/runtime.rs#L215) | skin_editor_audio::PreparedWeak |
| `pub fn add_monitor_frames( &self, frame: u64, format: MonitorFormat, output: &mut [f32], ) -> Option<usize>` | impl PreparedSound | [源码:25](../../../desktop/audio/src/voice.rs#L25) | skin_editor_audio::MonitorFormat |
| `pub fn add_monitor_frames_with_gain( &self, frame: u64, format: MonitorFormat, output: &mut [f32], gain: f32, ) -> Option<usize>` | impl PreparedSound | [源码:33](../../../desktop/audio/src/voice.rs#L33) | skin_editor_audio::MonitorFormat |
| `pub fn monitor_finished(&self, frame: u64, format: MonitorFormat) -> bool` | impl PreparedSound | [源码:92](../../../desktop/audio/src/voice.rs#L92) | skin_editor_audio::MonitorFormat |
| `pub fn monitor_frame( &self, frame: u64, format: MonitorFormat, ) -> Option<[f32; 2]>` | impl PreparedSound | [源码:109](../../../desktop/audio/src/voice.rs#L109) | skin_editor_audio::MonitorFormat |
| `pub fn monitor_frame_with_gain( &self, frame: u64, format: MonitorFormat, gain: f32, ) -> Option<[f32; 2]>` | impl PreparedSound | [源码:113](../../../desktop/audio/src/voice.rs#L113) | skin_editor_audio::MonitorFormat |
| `fn clone(&self) -> PreparedSound` | impl Clone for PreparedSound | [源码:93](../../../desktop/audio/src/runtime.rs#L93) | skin_editor_audio::PreparedSound |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for PreparedSound | [源码:93](../../../desktop/audio/src/runtime.rs#L93) |  |

## skin_editor_audio::PreparedWeak

[源码:125](../../../desktop/audio/src/runtime.rs#L125) · struct

```rust
pub struct PreparedWeak {
    key: PreparedKey,
    pcm: PcmWeak,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_audio::runtime::PreparedWeak`

声明引用：`skin_editor_audio::PcmWeak`, `skin_editor_audio::PreparedKey`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_audio::PreparedSound::downgrade`, `skin_editor_audio::PreparedWeak::clone`, `skin_editor_infrastructure::AudioPreparationQueue (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl PreparedWeak` — [源码:129](../../../desktop/audio/src/runtime.rs#L129)
- `impl Clone for PreparedWeak` — [源码:124](../../../desktop/audio/src/runtime.rs#L124)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn is_alive(&self) -> bool` | impl PreparedWeak | [源码:130](../../../desktop/audio/src/runtime.rs#L130) |  |
| `pub fn upgrade_for( &self, plan: SoundPlan, budget: &PcmBudget, ) -> Option<PreparedSound>` | impl PreparedWeak | [源码:133](../../../desktop/audio/src/runtime.rs#L133) | skin_editor_audio::PcmBudget, skin_editor_audio::PreparedSound, skin_editor_audio::SoundPlan |
| `fn clone(&self) -> PreparedWeak` | impl Clone for PreparedWeak | [源码:124](../../../desktop/audio/src/runtime.rs#L124) | skin_editor_audio::PreparedWeak |

## skin_editor_audio::Runtime

[源码:279](../../../desktop/audio/src/runtime.rs#L279) · struct

```rust
pub struct Runtime {
    budget: PcmBudget,
    rate: u32,
    channels: u16,
    origin: Option<Origin>,
    minimum_conversion: bool,
    external_voices: Option<Arc<()>>,
    bindings: BTreeMap<SoundTarget, Binding>,
    voices: [Option<Voice>; 32],
    events: VecDeque<Event>,
    selection: u64,
    device: u64,
    gates: [u64; 2],
    device_ready: bool,
    scene_running: bool,
    closed: bool,
    next_build: u64,
    next_voice: u64,
    mix_frame: u64,
    scene_frame: u64,
    audition_edits: AuditionEdits,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_audio::runtime::Runtime`

声明引用：`skin_editor_audio::AuditionEdits`, `skin_editor_audio::Origin`, `skin_editor_audio::PcmBudget`, `skin_editor_audio::SoundTarget`, `skin_editor_audio::runtime::Binding`, `skin_editor_audio::runtime::Event`, `skin_editor_audio::runtime::Voice`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::separated_verification::duration (声明)`, `skin_editor_desktop::separated_verification::sound (声明)`, `skin_editor_desktop::separated_verification::tick (声明)`, `skin_editor_desktop::separated_verification::trigger (声明)`, `skin_editor_desktop::separated_verification::wait (声明)`, `skin_editor_infrastructure::OutputProducer::prefill`, `skin_editor_infrastructure::OutputProducer::pump`, `skin_editor_infrastructure::OutputProducer::pump_timeline`, `skin_editor_infrastructure::OutputProducer::pump_to`, `skin_editor_infrastructure::SkinAudioController (声明)`, `skin_editor_infrastructure::SkinAudioController::runtime`, `skin_editor_infrastructure::separated_output::BackgroundOwner::finish`, `skin_editor_infrastructure::separated_output::BackgroundOwner::freeze`, `skin_editor_infrastructure::separated_output::BackgroundOwner::pause`, `skin_editor_infrastructure::separated_output::BackgroundOwner::pump`, `skin_editor_infrastructure::separated_output::BackgroundOwner::reclaim_abandoned`, `skin_editor_infrastructure::separated_output::BackgroundOwner::resume`, `skin_editor_infrastructure::separated_output::BackgroundOwner::seek`, `skin_editor_infrastructure::separated_output::BackgroundOwner::validate`, `skin_editor_infrastructure::separated_output::boundary (声明)`, `skin_editor_infrastructure::separated_output::boundary_input (声明)`, `skin_editor_infrastructure::voice_handoff::VoiceOwner::finish_runtime`, `skin_editor_infrastructure::voice_handoff::VoiceOwner::pump_runtime`

实现／约束（显式或 derive，无类继承）：

- `impl Runtime` — [源码:304](../../../desktop/audio/src/runtime.rs#L304)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn new(rate: u32, channels: u16) -> Result<Self, RuntimeError>` | impl Runtime | [源码:305](../../../desktop/audio/src/runtime.rs#L305) | skin_editor_audio::RuntimeError |
| `pub fn pcm_budget(&self) -> PcmBudget` | impl Runtime | [源码:308](../../../desktop/audio/src/runtime.rs#L308) | skin_editor_audio::PcmBudget |
| `pub fn with_budget( rate: u32, channels: u16, budget: PcmBudget, ) -> Result<Self, RuntimeError>` | impl Runtime | [源码:311](../../../desktop/audio/src/runtime.rs#L311) | skin_editor_audio::PcmBudget, skin_editor_audio::RuntimeError |
| `pub fn enable_minimum_conversion(&mut self)` | impl Runtime | [源码:344](../../../desktop/audio/src/runtime.rs#L344) |  |
| `pub fn output_activity(&self) -> [bool; 2]` | impl Runtime | [源码:351](../../../desktop/audio/src/runtime.rs#L351) |  |
| `pub fn output_format(&self) -> (u32, u16)` | impl Runtime | [源码:362](../../../desktop/audio/src/runtime.rs#L362) |  |
| `pub fn synchronize( &mut self, origin: Origin, plans: &[SoundPlan], ) -> Result<Vec<BuildRequest>, RuntimeError>` | impl Runtime | [源码:368](../../../desktop/audio/src/runtime.rs#L368) | skin_editor_audio::BuildRequest, skin_editor_audio::Origin, skin_editor_audio::RuntimeError, skin_editor_audio::SoundPlan |
| `pub fn synchronize_cancelled( &mut self, origin: Origin, plans: &[SoundPlan], cancelled_drafts: &[u64], ) -> Result<Vec<BuildRequest>, RuntimeError>` | impl Runtime | [源码:376](../../../desktop/audio/src/runtime.rs#L376) | skin_editor_audio::BuildRequest, skin_editor_audio::Origin, skin_editor_audio::RuntimeError, skin_editor_audio::SoundPlan |
| `pub fn request_library( &mut self, plan: SoundPlan, ) -> Result<BuildRequest, RuntimeError>` | impl Runtime | [源码:515](../../../desktop/audio/src/runtime.rs#L515) | skin_editor_audio::BuildRequest, skin_editor_audio::RuntimeError, skin_editor_audio::SoundPlan |
| `pub fn library_sound(&self) -> Option<PreparedSound>` | impl Runtime | [源码:545](../../../desktop/audio/src/runtime.rs#L545) | skin_editor_audio::PreparedSound |
| `pub fn library_status(&self) -> Option<(AssetId, BindingStatus)>` | impl Runtime | [源码:554](../../../desktop/audio/src/runtime.rs#L554) | skin_editor_audio::BindingStatus |
| `pub fn fail_library(&mut self, error: PrepareFailure)` | impl Runtime | [源码:569](../../../desktop/audio/src/runtime.rs#L569) | skin_editor_audio::PrepareFailure |
| `fn clear_library(&mut self)` | impl Runtime | [源码:578](../../../desktop/audio/src/runtime.rs#L578) |  |
| `pub fn complete( &mut self, request: BuildRequest, result: Result<PreparedSound, PrepareFailure>, ) -> Result<Completion, RuntimeError>` | impl Runtime | [源码:593](../../../desktop/audio/src/runtime.rs#L593) | skin_editor_audio::BuildRequest, skin_editor_audio::Completion, skin_editor_audio::PrepareFailure, skin_editor_audio::PreparedSound, skin_editor_audio::RuntimeError |
| `pub fn retry( &mut self, component: AudioComponentId, ) -> Result<BuildRequest, RuntimeError>` | impl Runtime | [源码:644](../../../desktop/audio/src/runtime.rs#L644) | skin_editor_audio::BuildRequest, skin_editor_audio::RuntimeError |
| `pub fn status(&self, component: AudioComponentId) -> Option<BindingStatus>` | impl Runtime | [源码:670](../../../desktop/audio/src/runtime.rs#L670) | skin_editor_audio::BindingStatus |
| `pub fn retained_drafts(&self) -> Vec<u64>` | impl Runtime | [源码:680](../../../desktop/audio/src/runtime.rs#L680) |  |
| `pub fn invalidate_drafts(&mut self, ids: &[u64])` | impl Runtime | [源码:708](../../../desktop/audio/src/runtime.rs#L708) |  |
| `pub fn token(&self, lane: Channel) -> GateToken` | impl Runtime | [源码:732](../../../desktop/audio/src/runtime.rs#L732) | skin_editor_audio::Channel, skin_editor_audio::GateToken |
| `fn valid(&self, token: GateToken) -> Result<(), RuntimeError>` | impl Runtime | [源码:740](../../../desktop/audio/src/runtime.rs#L740) | skin_editor_audio::GateToken, skin_editor_audio::RuntimeError |
| `fn clear_lane(&mut self, lane: Channel)` | impl Runtime | [源码:752](../../../desktop/audio/src/runtime.rs#L752) | skin_editor_audio::Channel |
| `pub fn set_scene_running( &mut self, token: GateToken, running: bool, ) -> Result<GateToken, RuntimeError>` | impl Runtime | [源码:760](../../../desktop/audio/src/runtime.rs#L760) | skin_editor_audio::GateToken, skin_editor_audio::RuntimeError |
| `pub fn stop_audition( &mut self, token: GateToken, ) -> Result<GateToken, RuntimeError>` | impl Runtime | [源码:775](../../../desktop/audio/src/runtime.rs#L775) | skin_editor_audio::GateToken, skin_editor_audio::RuntimeError |
| `pub fn seek_scene( &mut self, token: GateToken, frame: u64, ) -> Result<GateToken, RuntimeError>` | impl Runtime | [源码:786](../../../desktop/audio/src/runtime.rs#L786) | skin_editor_audio::GateToken, skin_editor_audio::RuntimeError |
| `pub fn invalidate_device(&mut self, expected: u64) -> Result<u64, RuntimeError>` | impl Runtime | [源码:791](../../../desktop/audio/src/runtime.rs#L791) | skin_editor_audio::RuntimeError |
| `pub fn rebuild_device(&mut self, expected: u64) -> Result<(), RuntimeError>` | impl Runtime | [源码:806](../../../desktop/audio/src/runtime.rs#L806) | skin_editor_audio::RuntimeError |
| `pub fn device_epoch(&self) -> u64` | impl Runtime | [源码:816](../../../desktop/audio/src/runtime.rs#L816) |  |
| `fn start(&mut self, event: Event) -> Result<TriggerReceipt, RuntimeError>` | impl Runtime | [源码:820](../../../desktop/audio/src/runtime.rs#L820) | skin_editor_audio::RuntimeError, skin_editor_audio::TriggerReceipt, skin_editor_audio::runtime::Event |
| `pub fn trigger( &mut self, token: GateToken, component: impl Into<SoundTarget>, at_mix_frame: u64, ) -> Result<TriggerReceipt, RuntimeError>` | impl Runtime | [源码:851](../../../desktop/audio/src/runtime.rs#L851) | skin_editor_audio::GateToken, skin_editor_audio::RuntimeError, skin_editor_audio::SoundTarget, skin_editor_audio::TriggerReceipt |
| `pub fn mix(&mut self, output: &mut [f32]) -> Result<MixReport, RuntimeError>` | impl Runtime | [源码:918](../../../desktop/audio/src/runtime.rs#L918) | skin_editor_audio::MixReport, skin_editor_audio::RuntimeError |
| `pub fn mix_split_buffers( &mut self, scene: &mut PcmBuffer, audition: &mut PcmBuffer, ) -> Result<MixReport, RuntimeError>` | impl Runtime | [源码:930](../../../desktop/audio/src/runtime.rs#L930) | skin_editor_audio::MixReport, skin_editor_audio::PcmBuffer, skin_editor_audio::RuntimeError |
| `pub fn mix_split_buffers_marked( &mut self, scene: &mut PcmBuffer, audition: &mut PcmBuffer, marks: &mut [[u64; 2]], ) -> Result<MixReport, RuntimeError>` | impl Runtime | [源码:939](../../../desktop/audio/src/runtime.rs#L939) | skin_editor_audio::MixReport, skin_editor_audio::PcmBuffer, skin_editor_audio::RuntimeError |
| `fn split_buffers( &mut self, scene: &mut PcmBuffer, audition: &mut PcmBuffer, marks: Option<&mut [[u64; 2]]>, ) -> Result<MixReport, RuntimeError>` | impl Runtime | [源码:947](../../../desktop/audio/src/runtime.rs#L947) | skin_editor_audio::MixReport, skin_editor_audio::PcmBuffer, skin_editor_audio::RuntimeError |
| `fn mix_frames( &mut self, frames: usize, write: impl FnMut(usize, [f32; 2], [f32; 2], [u64; 2]), ) -> Result<MixReport, RuntimeError>` | impl Runtime | [源码:982](../../../desktop/audio/src/runtime.rs#L982) | skin_editor_audio::MixReport, skin_editor_audio::RuntimeError |
| `pub fn mix_buffer( &mut self, output: &mut PcmBuffer, ) -> Result<MixReport, RuntimeError>` | impl Runtime | [源码:1065](../../../desktop/audio/src/runtime.rs#L1065) | skin_editor_audio::MixReport, skin_editor_audio::PcmBuffer, skin_editor_audio::RuntimeError |
| `pub fn voices(&self) -> impl Iterator<Item = VoiceInfo> + '_` | impl Runtime | [源码:1076](../../../desktop/audio/src/runtime.rs#L1076) | skin_editor_audio::VoiceInfo |
| `pub fn queued(&self) -> usize` | impl Runtime | [源码:1085](../../../desktop/audio/src/runtime.rs#L1085) |  |
| `pub fn mix_frame(&self) -> u64` | impl Runtime | [源码:1088](../../../desktop/audio/src/runtime.rs#L1088) |  |
| `pub fn scene_frame(&self) -> u64` | impl Runtime | [源码:1091](../../../desktop/audio/src/runtime.rs#L1091) |  |
| `pub fn enable_voice_handoff(&mut self) -> Result<(), RuntimeError>` | impl Runtime | [源码:1097](../../../desktop/audio/src/runtime.rs#L1097) | skin_editor_audio::RuntimeError |
| `pub fn voice_handoff_enabled(&self) -> bool` | impl Runtime | [源码:1110](../../../desktop/audio/src/runtime.rs#L1110) |  |
| `pub fn is_closed(&self) -> bool` | impl Runtime | [源码:1113](../../../desktop/audio/src/runtime.rs#L1113) |  |
| `pub fn synchronize_handoff_clock( &mut self, output: u64, scene: u64, ) -> Result<(), RuntimeError>` | impl Runtime | [源码:1118](../../../desktop/audio/src/runtime.rs#L1118) | skin_editor_audio::RuntimeError |
| `pub fn voice_runtime_key(&self) -> Option<VoiceRuntimeKey>` | impl Runtime | [源码:1136](../../../desktop/audio/src/runtime.rs#L1136) | skin_editor_audio::VoiceRuntimeKey |
| `pub fn scene_running(&self) -> bool` | impl Runtime | [源码:1141](../../../desktop/audio/src/runtime.rs#L1141) |  |
| `pub fn handoff_waiting(&self) -> usize` | impl Runtime | [源码:1144](../../../desktop/audio/src/runtime.rs#L1144) |  |
| `pub fn take_voice_handoff( &mut self, ) -> Result<Option<VoiceHandoff>, RuntimeError>` | impl Runtime | [源码:1154](../../../desktop/audio/src/runtime.rs#L1154) | skin_editor_audio::RuntimeError, skin_editor_audio::VoiceHandoff |
| `pub fn finish_voice_handoff(&mut self, returned: &VoiceHandoff) -> bool` | impl Runtime | [源码:1181](../../../desktop/audio/src/runtime.rs#L1181) | skin_editor_audio::VoiceHandoff |
| `pub fn retry_voice_handoff(&mut self, returned: &VoiceHandoff) -> bool` | impl Runtime | [源码:1201](../../../desktop/audio/src/runtime.rs#L1201) | skin_editor_audio::VoiceHandoff |
| `pub fn shutdown(&mut self)` | impl Runtime | [源码:1215](../../../desktop/audio/src/runtime.rs#L1215) |  |

## skin_editor_audio::RuntimeError

[源码:59](../../../desktop/audio/src/runtime.rs#L59) · enum

```rust
pub enum RuntimeError {
    Busy,
    Stale,
    NotReady,
    Frozen,
    DeviceUnavailable,
    Closed,
    MissingComponent,
    ResourceLimit,
    InvalidBlock,
    InvalidOrigin,
    InvalidPrepared,
    ExecutionMode,
    UnsupportedScheduling,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_audio::runtime::RuntimeError`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_audio::Runtime::complete`, `skin_editor_audio::Runtime::enable_voice_handoff`, `skin_editor_audio::Runtime::invalidate_device`, `skin_editor_audio::Runtime::mix`, `skin_editor_audio::Runtime::mix_buffer`, `skin_editor_audio::Runtime::mix_frames`, `skin_editor_audio::Runtime::mix_split_buffers`, `skin_editor_audio::Runtime::mix_split_buffers_marked`, `skin_editor_audio::Runtime::new`, `skin_editor_audio::Runtime::rebuild_device`, `skin_editor_audio::Runtime::request_library`, `skin_editor_audio::Runtime::retry`, `skin_editor_audio::Runtime::seek_scene`, `skin_editor_audio::Runtime::set_scene_running`, `skin_editor_audio::Runtime::split_buffers`, `skin_editor_audio::Runtime::start`, `skin_editor_audio::Runtime::stop_audition`, `skin_editor_audio::Runtime::synchronize`, `skin_editor_audio::Runtime::synchronize_cancelled`, `skin_editor_audio::Runtime::synchronize_handoff_clock`, `skin_editor_audio::Runtime::take_voice_handoff`, `skin_editor_audio::Runtime::trigger`, `skin_editor_audio::Runtime::valid`, `skin_editor_audio::Runtime::with_budget`, `skin_editor_audio::RuntimeError::clone`, `skin_editor_audio::RuntimeError::eq`, `skin_editor_audio::runtime::next (声明)`, `skin_editor_infrastructure::SkinAudioController::new`, `skin_editor_infrastructure::SkinAudioController::with_budget`, `skin_editor_infrastructure::SupplyOutcome (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for RuntimeError` — [源码:58](../../../desktop/audio/src/runtime.rs#L58)
- `impl Debug for RuntimeError` — [源码:58](../../../desktop/audio/src/runtime.rs#L58)
- `impl PartialEq for RuntimeError` — [源码:58](../../../desktop/audio/src/runtime.rs#L58)
- `impl Copy for RuntimeError` — [源码:58](../../../desktop/audio/src/runtime.rs#L58)
- `impl Eq for RuntimeError` — [源码:58](../../../desktop/audio/src/runtime.rs#L58)
- `impl StructuralPartialEq for RuntimeError` — [源码:58](../../../desktop/audio/src/runtime.rs#L58)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> RuntimeError` | impl Clone for RuntimeError | [源码:58](../../../desktop/audio/src/runtime.rs#L58) | skin_editor_audio::RuntimeError |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for RuntimeError | [源码:58](../../../desktop/audio/src/runtime.rs#L58) |  |
| `fn eq(&self, other: &RuntimeError) -> bool` | impl PartialEq for RuntimeError | [源码:58](../../../desktop/audio/src/runtime.rs#L58) | skin_editor_audio::RuntimeError |

## skin_editor_audio::SOUND_PLAN_VERSION

[源码:21](../../../desktop/audio/src/lib.rs#L21) · constant

```rust
pub const SOUND_PLAN_VERSION: u16 = 2;
```

## skin_editor_audio::SoundPlan

[源码:34](../../../desktop/audio/src/lib.rs#L34) · struct

```rust
pub struct SoundPlan {
    pub target: SoundTarget,
    pub source: BlobRef,
    pub metadata: Option<AudioMetadata>,
    pub gain: Gain,
    pub visible: bool,
}
```

声明引用：`skin_editor_audio::SoundTarget`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_audio::AuditionEdits (声明)`, `skin_editor_audio::AuditionEdits::select`, `skin_editor_audio::BuildRequest (声明)`, `skin_editor_audio::PreparedKey::new`, `skin_editor_audio::PreparedSound (声明)`, `skin_editor_audio::PreparedSound::from_decoded`, `skin_editor_audio::PreparedSound::from_source`, `skin_editor_audio::PreparedWeak::upgrade_for`, `skin_editor_audio::Runtime::request_library`, `skin_editor_audio::Runtime::synchronize`, `skin_editor_audio::Runtime::synchronize_cancelled`, `skin_editor_audio::SoundPlan::clone`, `skin_editor_audio::SoundPlan::eq`, `skin_editor_audio::audition_edit::same_content (声明)`, `skin_editor_audio::runtime::Binding (声明)`, `skin_editor_infrastructure::SkinAudioController::request_library`, `skin_editor_infrastructure::SkinAudioController::select`, `skin_editor_infrastructure::audio_supply::Control (声明)`, `skin_editor_infrastructure::audio_supply::Projection (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl SoundPlan` — [源码:64](../../../desktop/audio/src/lib.rs#L64)
- `impl Clone for SoundPlan` — [源码:33](../../../desktop/audio/src/lib.rs#L33)
- `impl Debug for SoundPlan` — [源码:33](../../../desktop/audio/src/lib.rs#L33)
- `impl PartialEq for SoundPlan` — [源码:33](../../../desktop/audio/src/lib.rs#L33)
- `impl Copy for SoundPlan` — [源码:33](../../../desktop/audio/src/lib.rs#L33)
- `impl StructuralPartialEq for SoundPlan` — [源码:33](../../../desktop/audio/src/lib.rs#L33)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn build( project: &SkinProject, component: AudioComponentId, ) -> Result<Self, AudioError>` | impl SoundPlan | [源码:65](../../../desktop/audio/src/lib.rs#L65) | skin_editor_audio::AudioError |
| `pub fn output_action(&self) -> Result<OutputAction, AudioError>` | impl SoundPlan | [源码:87](../../../desktop/audio/src/lib.rs#L87) | skin_editor_audio::AudioError, skin_editor_audio::OutputAction |
| `pub fn original_bytes<'a>( &self, reference: BlobRef, bytes: &'a [u8], ) -> Result<&'a [u8], AudioError>` | impl SoundPlan | [源码:102](../../../desktop/audio/src/lib.rs#L102) | skin_editor_audio::AudioError |
| `pub fn process_pcm( &self, samples: &[f32], max_samples: usize, ) -> Result<Vec<f32>, AudioError>` | impl SoundPlan | [源码:119](../../../desktop/audio/src/lib.rs#L119) | skin_editor_audio::AudioError |
| `fn clone(&self) -> SoundPlan` | impl Clone for SoundPlan | [源码:33](../../../desktop/audio/src/lib.rs#L33) | skin_editor_audio::SoundPlan |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for SoundPlan | [源码:33](../../../desktop/audio/src/lib.rs#L33) |  |
| `fn eq(&self, other: &SoundPlan) -> bool` | impl PartialEq for SoundPlan | [源码:33](../../../desktop/audio/src/lib.rs#L33) | skin_editor_audio::SoundPlan |

## skin_editor_audio::SoundTarget

[源码:24](../../../desktop/audio/src/lib.rs#L24) · enum

```rust
pub enum SoundTarget {
    Component(AudioComponentId),
    Library(AssetId),
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_audio::Runtime (声明)`, `skin_editor_audio::Runtime::trigger`, `skin_editor_audio::SoundPlan (声明)`, `skin_editor_audio::SoundTarget::clone`, `skin_editor_audio::SoundTarget::cmp`, `skin_editor_audio::SoundTarget::eq`, `skin_editor_audio::SoundTarget::partial_cmp`, `skin_editor_audio::VoiceInfo (声明)`, `skin_editor_audio::runtime::Event (声明)`, `skin_editor_audio::runtime::Voice (声明)`, `skin_editor_infrastructure::AudioPreparationQueue::cancel_component`, `skin_editor_infrastructure::AudioPreparationQueue::cancel_target`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for SoundTarget` — [源码:23](../../../desktop/audio/src/lib.rs#L23)
- `impl Debug for SoundTarget` — [源码:23](../../../desktop/audio/src/lib.rs#L23)
- `impl From<AudioComponentId> for SoundTarget` — [源码:28](../../../desktop/audio/src/lib.rs#L28)
- `impl Ord for SoundTarget` — [源码:23](../../../desktop/audio/src/lib.rs#L23)
- `impl PartialEq for SoundTarget` — [源码:23](../../../desktop/audio/src/lib.rs#L23)
- `impl PartialOrd for SoundTarget` — [源码:23](../../../desktop/audio/src/lib.rs#L23)
- `impl Copy for SoundTarget` — [源码:23](../../../desktop/audio/src/lib.rs#L23)
- `impl Eq for SoundTarget` — [源码:23](../../../desktop/audio/src/lib.rs#L23)
- `impl StructuralPartialEq for SoundTarget` — [源码:23](../../../desktop/audio/src/lib.rs#L23)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> SoundTarget` | impl Clone for SoundTarget | [源码:23](../../../desktop/audio/src/lib.rs#L23) | skin_editor_audio::SoundTarget |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for SoundTarget | [源码:23](../../../desktop/audio/src/lib.rs#L23) |  |
| `fn from(id: AudioComponentId) -> Self` | impl From<AudioComponentId> for SoundTarget | [源码:29](../../../desktop/audio/src/lib.rs#L29) |  |
| `fn cmp(&self, other: &SoundTarget) -> Ordering` | impl Ord for SoundTarget | [源码:23](../../../desktop/audio/src/lib.rs#L23) | skin_editor_audio::SoundTarget |
| `fn eq(&self, other: &SoundTarget) -> bool` | impl PartialEq for SoundTarget | [源码:23](../../../desktop/audio/src/lib.rs#L23) | skin_editor_audio::SoundTarget |
| `fn partial_cmp(&self, other: &SoundTarget) -> Option<Ordering>` | impl PartialOrd for SoundTarget | [源码:23](../../../desktop/audio/src/lib.rs#L23) | skin_editor_audio::SoundTarget |

## skin_editor_audio::TriggerReceipt

[源码:257](../../../desktop/audio/src/runtime.rs#L257) · struct

```rust
pub struct TriggerReceipt {
    pub origin: Origin,
    pub stale: bool,
    pub voice: Option<u64>,
    pub queued: bool,
    pub silent: bool,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_audio::runtime::TriggerReceipt`

声明引用：`skin_editor_audio::Origin`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_audio::Runtime::start`, `skin_editor_audio::Runtime::trigger`, `skin_editor_audio::TriggerReceipt::clone`, `skin_editor_audio::TriggerReceipt::eq`, `skin_editor_desktop::editor_audio::EditorAudio (声明)`, `skin_editor_desktop::editor_audio::combined::probes::Probes::receipt`, `skin_editor_infrastructure::AudioPlayback::trigger`, `skin_editor_infrastructure::SupplyOutcome (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for TriggerReceipt` — [源码:256](../../../desktop/audio/src/runtime.rs#L256)
- `impl Debug for TriggerReceipt` — [源码:256](../../../desktop/audio/src/runtime.rs#L256)
- `impl PartialEq for TriggerReceipt` — [源码:256](../../../desktop/audio/src/runtime.rs#L256)
- `impl Copy for TriggerReceipt` — [源码:256](../../../desktop/audio/src/runtime.rs#L256)
- `impl Eq for TriggerReceipt` — [源码:256](../../../desktop/audio/src/runtime.rs#L256)
- `impl StructuralPartialEq for TriggerReceipt` — [源码:256](../../../desktop/audio/src/runtime.rs#L256)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> TriggerReceipt` | impl Clone for TriggerReceipt | [源码:256](../../../desktop/audio/src/runtime.rs#L256) | skin_editor_audio::TriggerReceipt |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for TriggerReceipt | [源码:256](../../../desktop/audio/src/runtime.rs#L256) |  |
| `fn eq(&self, other: &TriggerReceipt) -> bool` | impl PartialEq for TriggerReceipt | [源码:256](../../../desktop/audio/src/runtime.rs#L256) | skin_editor_audio::TriggerReceipt |

## skin_editor_audio::VoiceHandoff

[源码:211](../../../desktop/audio/src/voice.rs#L211) · struct

```rust
pub struct VoiceHandoff {
    pub(crate) info: VoiceInfo,
    pub(crate) sound: PreparedSound,
    pub(crate) validity: VoiceValidity,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_audio::voice::VoiceHandoff`

声明引用：`skin_editor_audio::PreparedSound`, `skin_editor_audio::VoiceInfo`, `skin_editor_audio::VoiceValidity`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_audio::Runtime::finish_voice_handoff`, `skin_editor_audio::Runtime::retry_voice_handoff`, `skin_editor_audio::Runtime::take_voice_handoff`, `skin_editor_audio::VoiceHandoff::clone`, `skin_editor_infrastructure::voice_handoff::Pin (声明)`, `skin_editor_infrastructure::voice_handoff::VoiceOwner::collect_with`, `skin_editor_infrastructure::voice_handoff::VoiceOwner::publish`

实现／约束（显式或 derive，无类继承）：

- `impl VoiceHandoff` — [源码:216](../../../desktop/audio/src/voice.rs#L216)
- `impl Clone for VoiceHandoff` — [源码:210](../../../desktop/audio/src/voice.rs#L210)
- `impl Debug for VoiceHandoff` — [源码:210](../../../desktop/audio/src/voice.rs#L210)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn info(&self) -> VoiceInfo` | impl VoiceHandoff | [源码:217](../../../desktop/audio/src/voice.rs#L217) | skin_editor_audio::VoiceInfo |
| `pub fn sound(&self) -> &PreparedSound` | impl VoiceHandoff | [源码:220](../../../desktop/audio/src/voice.rs#L220) | skin_editor_audio::PreparedSound |
| `pub fn validity(&self) -> VoiceValidity` | impl VoiceHandoff | [源码:223](../../../desktop/audio/src/voice.rs#L223) | skin_editor_audio::VoiceValidity |
| `fn clone(&self) -> VoiceHandoff` | impl Clone for VoiceHandoff | [源码:210](../../../desktop/audio/src/voice.rs#L210) | skin_editor_audio::VoiceHandoff |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for VoiceHandoff | [源码:210](../../../desktop/audio/src/voice.rs#L210) |  |

## skin_editor_audio::VoiceInfo

[源码:265](../../../desktop/audio/src/runtime.rs#L265) · struct

```rust
pub struct VoiceInfo {
    pub id: u64,
    pub target: SoundTarget,
    pub channel: Channel,
    pub origin: Origin,
    pub cursor: usize,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_audio::runtime::VoiceInfo`

声明引用：`skin_editor_audio::Channel`, `skin_editor_audio::Origin`, `skin_editor_audio::SoundTarget`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_audio::Runtime::voices`, `skin_editor_audio::VoiceHandoff (声明)`, `skin_editor_audio::VoiceHandoff::info`, `skin_editor_audio::VoiceInfo::clone`, `skin_editor_audio::VoiceInfo::eq`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for VoiceInfo` — [源码:264](../../../desktop/audio/src/runtime.rs#L264)
- `impl Debug for VoiceInfo` — [源码:264](../../../desktop/audio/src/runtime.rs#L264)
- `impl PartialEq for VoiceInfo` — [源码:264](../../../desktop/audio/src/runtime.rs#L264)
- `impl Copy for VoiceInfo` — [源码:264](../../../desktop/audio/src/runtime.rs#L264)
- `impl Eq for VoiceInfo` — [源码:264](../../../desktop/audio/src/runtime.rs#L264)
- `impl StructuralPartialEq for VoiceInfo` — [源码:264](../../../desktop/audio/src/runtime.rs#L264)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> VoiceInfo` | impl Clone for VoiceInfo | [源码:264](../../../desktop/audio/src/runtime.rs#L264) | skin_editor_audio::VoiceInfo |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for VoiceInfo | [源码:264](../../../desktop/audio/src/runtime.rs#L264) |  |
| `fn eq(&self, other: &VoiceInfo) -> bool` | impl PartialEq for VoiceInfo | [源码:264](../../../desktop/audio/src/runtime.rs#L264) | skin_editor_audio::VoiceInfo |

## skin_editor_audio::VoiceRuntimeKey

[源码:167](../../../desktop/audio/src/voice.rs#L167) · struct

```rust
pub struct VoiceRuntimeKey(pub(crate) Weak<()>);
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_audio::voice::VoiceRuntimeKey`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_audio::Runtime::voice_runtime_key`, `skin_editor_audio::VoiceRuntimeKey::clone`, `skin_editor_infrastructure::separated_output::BackgroundOwner (声明)`, `skin_editor_infrastructure::voice_handoff::VoiceOwner (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl VoiceRuntimeKey` — [源码:168](../../../desktop/audio/src/voice.rs#L168)
- `impl Clone for VoiceRuntimeKey` — [源码:166](../../../desktop/audio/src/voice.rs#L166)
- `impl Debug for VoiceRuntimeKey` — [源码:166](../../../desktop/audio/src/voice.rs#L166)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn same_runtime(&self, other: &Self) -> bool` | impl VoiceRuntimeKey | [源码:169](../../../desktop/audio/src/voice.rs#L169) |  |
| `fn clone(&self) -> VoiceRuntimeKey` | impl Clone for VoiceRuntimeKey | [源码:166](../../../desktop/audio/src/voice.rs#L166) | skin_editor_audio::VoiceRuntimeKey |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for VoiceRuntimeKey | [源码:166](../../../desktop/audio/src/voice.rs#L166) |  |

## skin_editor_audio::VoiceValidity

[源码:182](../../../desktop/audio/src/voice.rs#L182) · struct

```rust
pub struct VoiceValidity(pub(crate) Arc<VoiceControl>);
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_audio::voice::VoiceValidity`

声明引用：`skin_editor_audio::voice::VoiceControl`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_audio::VoiceHandoff (声明)`, `skin_editor_audio::VoiceHandoff::validity`, `skin_editor_audio::runtime::Voice (声明)`, `skin_editor_infrastructure::voice_handoff::Packet (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl VoiceValidity` — [源码:188](../../../desktop/audio/src/voice.rs#L188)
- `impl Clone for VoiceValidity` — [源码:183](../../../desktop/audio/src/voice.rs#L183)
- `impl Debug for VoiceValidity` — [源码:181](../../../desktop/audio/src/voice.rs#L181)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub(crate) fn new(gain: f32) -> Self` | impl VoiceValidity | [源码:189](../../../desktop/audio/src/voice.rs#L189) |  |
| `pub(crate) fn revoke(&self)` | impl VoiceValidity | [源码:195](../../../desktop/audio/src/voice.rs#L195) |  |
| `pub(crate) fn set_gain(&self, gain: f32)` | impl VoiceValidity | [源码:198](../../../desktop/audio/src/voice.rs#L198) |  |
| `pub fn is_valid(&self) -> bool` | impl VoiceValidity | [源码:201](../../../desktop/audio/src/voice.rs#L201) |  |
| `pub fn gain(&self) -> f32` | impl VoiceValidity | [源码:204](../../../desktop/audio/src/voice.rs#L204) |  |
| `fn clone(&self) -> Self` | impl Clone for VoiceValidity | [源码:184](../../../desktop/audio/src/voice.rs#L184) |  |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for VoiceValidity | [源码:181](../../../desktop/audio/src/voice.rs#L181) |  |

## skin_editor_audio::audition_edit::same_content

[源码:4](../../../desktop/audio/src/audition_edit.rs#L4) · fn

```rust
pub(crate) fn same_content(a: SoundPlan, b: SoundPlan) -> bool
```

声明引用：`skin_editor_audio::SoundPlan`

## skin_editor_audio::pcm::Allocation

[源码:96](../../../desktop/audio/src/pcm.rs#L96) · struct

```rust
struct Allocation {
    samples: Vec<f32>,
    _lease: PcmReservation,
}
```

声明引用：`skin_editor_audio::PcmReservation`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_audio::PcmBuffer (声明)`, `skin_editor_audio::PcmWeak (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Debug for Allocation` — [源码:95](../../../desktop/audio/src/pcm.rs#L95)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for Allocation | [源码:95](../../../desktop/audio/src/pcm.rs#L95) |  |

## skin_editor_audio::pcm::Budget

[源码:19](../../../desktop/audio/src/pcm.rs#L19) · struct

```rust
struct Budget {
    limit: usize,
    per_sound: usize,
    used: AtomicUsize,
    peak: AtomicUsize,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_audio::PcmBudget (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Debug for Budget` — [源码:18](../../../desktop/audio/src/pcm.rs#L18)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for Budget | [源码:18](../../../desktop/audio/src/pcm.rs#L18) |  |

## skin_editor_audio::runtime::Binding

[源码:227](../../../desktop/audio/src/runtime.rs#L227) · struct

```rust
struct Binding {
    plan: SoundPlan,
    pending: Option<BuildRequest>,
    effective: Option<Effective>,
    preparation: Preparation,
}
```

声明引用：`skin_editor_audio::BuildRequest`, `skin_editor_audio::Preparation`, `skin_editor_audio::SoundPlan`, `skin_editor_audio::runtime::Effective`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_audio::Runtime (声明)`

## skin_editor_audio::runtime::Effective

[源码:223](../../../desktop/audio/src/runtime.rs#L223) · struct

```rust
struct Effective {
    origin: Origin,
    sound: Option<PreparedSound>,
}
```

声明引用：`skin_editor_audio::Origin`, `skin_editor_audio::PreparedSound`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_audio::runtime::Binding (声明)`, `skin_editor_audio::runtime::Effective::clone`, `skin_editor_audio::runtime::Event (声明)`, `skin_editor_audio::runtime::Voice (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for Effective` — [源码:222](../../../desktop/audio/src/runtime.rs#L222)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> Effective` | impl Clone for Effective | [源码:222](../../../desktop/audio/src/runtime.rs#L222) | skin_editor_audio::runtime::Effective |

## skin_editor_audio::runtime::Event

[源码:233](../../../desktop/audio/src/runtime.rs#L233) · struct

```rust
struct Event {
    target: SoundTarget,
    gate: GateToken,
    at: u64,
    effective: Effective,
    stale: bool,
}
```

声明引用：`skin_editor_audio::GateToken`, `skin_editor_audio::SoundTarget`, `skin_editor_audio::runtime::Effective`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_audio::Runtime (声明)`, `skin_editor_audio::Runtime::start`

## skin_editor_audio::runtime::Voice

[源码:240](../../../desktop/audio/src/runtime.rs#L240) · struct

```rust
struct Voice {
    id: u64,
    target: SoundTarget,
    lane: Channel,
    effective: Effective,
    cursor: usize,
    gain: f32,
    handoff: Option<VoiceValidity>,
}
```

声明引用：`skin_editor_audio::Channel`, `skin_editor_audio::SoundTarget`, `skin_editor_audio::VoiceValidity`, `skin_editor_audio::runtime::Effective`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_audio::Runtime (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Drop for Voice` — [源码:249](../../../desktop/audio/src/runtime.rs#L249)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn drop(&mut self)` | impl Drop for Voice | [源码:250](../../../desktop/audio/src/runtime.rs#L250) |  |

## skin_editor_audio::runtime::next

[源码:301](../../../desktop/audio/src/runtime.rs#L301) · fn

```rust
fn next(value: u64) -> Result<u64, RuntimeError>
```

声明引用：`skin_editor_audio::RuntimeError`

## skin_editor_audio::voice::VoiceControl

[源码:177](../../../desktop/audio/src/voice.rs#L177) · struct

```rust
pub(crate) struct VoiceControl {
    valid: AtomicBool,
    gain: AtomicU32,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_audio::VoiceValidity (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Debug for VoiceControl` — [源码:176](../../../desktop/audio/src/voice.rs#L176)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for VoiceControl | [源码:176](../../../desktop/audio/src/voice.rs#L176) |  |

