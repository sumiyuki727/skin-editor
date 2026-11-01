# skin_editor_application：编译器符号清单

快照：2026-10-30。来自含私有项的 rustdoc；显示编译器规范化可见域。含 derive 生成的实现，不把它们视作类继承。来源行可能指向宏定义／展开点。
引用列只列 rustdoc 解析出的项目类型链接，不能充当完整调用图；无链接不表示无引用。所有方法保留源码位置供逐项复核。测试和未启用的 cfg 分支另见源码索引。

## skin_editor_application::API_VERSION

[源码:53](../../../crates/application/src/lib.rs#L53) · constant

```rust
pub const API_VERSION: u16 = 6;
```

## skin_editor_application::AUDIO_EDITOR_API_VERSION

[源码:4](../../../crates/application/src/audio_editor.rs#L4) · constant

```rust
pub const AUDIO_EDITOR_API_VERSION: u16 = 3;
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::audio_editor::AUDIO_EDITOR_API_VERSION`

## skin_editor_application::AddressError

[源码:311](../../../crates/application/src/contract.rs#L311) · enum

```rust
pub enum AddressError {
    TooLong,
    InvalidSyntax,
    Overflow,
    ZeroIdentity,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::contract::AddressError`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::AddressError::clone`, `skin_editor_application::AddressError::eq`, `skin_editor_application::BranchCoordinate::parse_address`, `skin_editor_application::HistoryQueryError (声明)`, `skin_editor_application::NodeId::parse_address`, `skin_editor_application::address::number (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for AddressError` — [源码:310](../../../crates/application/src/contract.rs#L310)
- `impl Debug for AddressError` — [源码:310](../../../crates/application/src/contract.rs#L310)
- `impl PartialEq for AddressError` — [源码:310](../../../crates/application/src/contract.rs#L310)
- `impl Copy for AddressError` — [源码:310](../../../crates/application/src/contract.rs#L310)
- `impl Eq for AddressError` — [源码:310](../../../crates/application/src/contract.rs#L310)
- `impl StructuralPartialEq for AddressError` — [源码:310](../../../crates/application/src/contract.rs#L310)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> AddressError` | impl Clone for AddressError | [源码:310](../../../crates/application/src/contract.rs#L310) | skin_editor_application::AddressError |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for AddressError | [源码:310](../../../crates/application/src/contract.rs#L310) |  |
| `fn eq(&self, other: &AddressError) -> bool` | impl PartialEq for AddressError | [源码:310](../../../crates/application/src/contract.rs#L310) | skin_editor_application::AddressError |

## skin_editor_application::AssetAudition

[源码:41](../../../crates/application/src/audio_editor.rs#L41) · struct

```rust
pub struct AssetAudition {
    pub source: SkinAudioSource,
    pub asset: AssetId,
    pub blob: BlobRef,
    pub metadata: Option<AudioMetadata>,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::audio_editor::AssetAudition`

声明引用：`skin_editor_application::SkinAudioSource`, `skin_editor_domain::AssetId`, `skin_editor_domain::AudioMetadata`, `skin_editor_domain::BlobRef`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::AssetAudition::clone`, `skin_editor_application::AssetAudition::eq`, `skin_editor_application::AudioEditorPort (声明)`, `skin_editor_application::AudioEditorPort::audition_asset`, `skin_editor_application::DisabledAudio::audition_asset`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for AssetAudition` — [源码:40](../../../crates/application/src/audio_editor.rs#L40)
- `impl Debug for AssetAudition` — [源码:40](../../../crates/application/src/audio_editor.rs#L40)
- `impl PartialEq for AssetAudition` — [源码:40](../../../crates/application/src/audio_editor.rs#L40)
- `impl Copy for AssetAudition` — [源码:40](../../../crates/application/src/audio_editor.rs#L40)
- `impl Eq for AssetAudition` — [源码:40](../../../crates/application/src/audio_editor.rs#L40)
- `impl StructuralPartialEq for AssetAudition` — [源码:40](../../../crates/application/src/audio_editor.rs#L40)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> AssetAudition` | impl Clone for AssetAudition | [源码:40](../../../crates/application/src/audio_editor.rs#L40) | skin_editor_application::AssetAudition |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for AssetAudition | [源码:40](../../../crates/application/src/audio_editor.rs#L40) |  |
| `fn eq(&self, other: &AssetAudition) -> bool` | impl PartialEq for AssetAudition | [源码:40](../../../crates/application/src/audio_editor.rs#L40) | skin_editor_application::AssetAudition |

## skin_editor_application::AssetAuditionState

[源码:48](../../../crates/application/src/audio_editor.rs#L48) · struct

```rust
pub struct AssetAuditionState {
    pub asset: AssetId,
    pub source: SkinAudioSource,
    pub preparation: AudioPreparationState,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::audio_editor::AssetAuditionState`

声明引用：`skin_editor_application::AudioPreparationState`, `skin_editor_application::SkinAudioSource`, `skin_editor_domain::AssetId`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::AssetAuditionState::clone`, `skin_editor_application::AssetAuditionState::eq`, `skin_editor_application::AudioEditorState (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for AssetAuditionState` — [源码:47](../../../crates/application/src/audio_editor.rs#L47)
- `impl Debug for AssetAuditionState` — [源码:47](../../../crates/application/src/audio_editor.rs#L47)
- `impl PartialEq for AssetAuditionState` — [源码:47](../../../crates/application/src/audio_editor.rs#L47)
- `impl Eq for AssetAuditionState` — [源码:47](../../../crates/application/src/audio_editor.rs#L47)
- `impl StructuralPartialEq for AssetAuditionState` — [源码:47](../../../crates/application/src/audio_editor.rs#L47)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> AssetAuditionState` | impl Clone for AssetAuditionState | [源码:47](../../../crates/application/src/audio_editor.rs#L47) | skin_editor_application::AssetAuditionState |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for AssetAuditionState | [源码:47](../../../crates/application/src/audio_editor.rs#L47) |  |
| `fn eq(&self, other: &AssetAuditionState) -> bool` | impl PartialEq for AssetAuditionState | [源码:47](../../../crates/application/src/audio_editor.rs#L47) | skin_editor_application::AssetAuditionState |

## skin_editor_application::AudioComponentState

[源码:34](../../../crates/application/src/audio_editor.rs#L34) · struct

```rust
pub struct AudioComponentState {
    pub component: AudioComponentId,
    pub desired: SkinAudioSource,
    pub effective: Option<SkinAudioSource>,
    pub preparation: AudioPreparationState,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::audio_editor::AudioComponentState`

声明引用：`skin_editor_application::AudioPreparationState`, `skin_editor_application::SkinAudioSource`, `skin_editor_domain::AudioComponentId`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::AudioComponentState::clone`, `skin_editor_application::AudioComponentState::eq`, `skin_editor_application::AudioEditorState (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for AudioComponentState` — [源码:33](../../../crates/application/src/audio_editor.rs#L33)
- `impl Debug for AudioComponentState` — [源码:33](../../../crates/application/src/audio_editor.rs#L33)
- `impl PartialEq for AudioComponentState` — [源码:33](../../../crates/application/src/audio_editor.rs#L33)
- `impl Eq for AudioComponentState` — [源码:33](../../../crates/application/src/audio_editor.rs#L33)
- `impl StructuralPartialEq for AudioComponentState` — [源码:33](../../../crates/application/src/audio_editor.rs#L33)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> AudioComponentState` | impl Clone for AudioComponentState | [源码:33](../../../crates/application/src/audio_editor.rs#L33) | skin_editor_application::AudioComponentState |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for AudioComponentState | [源码:33](../../../crates/application/src/audio_editor.rs#L33) |  |
| `fn eq(&self, other: &AudioComponentState) -> bool` | impl PartialEq for AudioComponentState | [源码:33](../../../crates/application/src/audio_editor.rs#L33) | skin_editor_application::AudioComponentState |

## skin_editor_application::AudioEditorAction

[源码:6](../../../crates/application/src/audio_editor.rs#L6) · enum

```rust
pub enum AudioEditorAction {
    Enable,
    Disable,
    SceneRunning(bool),
    Audition(AudioComponentId),
    AuditionAsset(AssetId),
    MonitorVolume(u16),
    StopAudition,
    Retry(AudioComponentId),
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::audio_editor::AudioEditorAction`

声明引用：`skin_editor_domain::AssetId`, `skin_editor_domain::AudioComponentId`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::AudioEditorAction::clone`, `skin_editor_application::AudioEditorAction::eq`, `skin_editor_application::AudioEditorPort (声明)`, `skin_editor_application::AudioEditorPort::request`, `skin_editor_application::DisabledAudio::request`, `skin_editor_application::ProjectSession::control_audio`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for AudioEditorAction` — [源码:5](../../../crates/application/src/audio_editor.rs#L5)
- `impl Debug for AudioEditorAction` — [源码:5](../../../crates/application/src/audio_editor.rs#L5)
- `impl PartialEq for AudioEditorAction` — [源码:5](../../../crates/application/src/audio_editor.rs#L5)
- `impl Copy for AudioEditorAction` — [源码:5](../../../crates/application/src/audio_editor.rs#L5)
- `impl Eq for AudioEditorAction` — [源码:5](../../../crates/application/src/audio_editor.rs#L5)
- `impl StructuralPartialEq for AudioEditorAction` — [源码:5](../../../crates/application/src/audio_editor.rs#L5)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> AudioEditorAction` | impl Clone for AudioEditorAction | [源码:5](../../../crates/application/src/audio_editor.rs#L5) | skin_editor_application::AudioEditorAction |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for AudioEditorAction | [源码:5](../../../crates/application/src/audio_editor.rs#L5) |  |
| `fn eq(&self, other: &AudioEditorAction) -> bool` | impl PartialEq for AudioEditorAction | [源码:5](../../../crates/application/src/audio_editor.rs#L5) | skin_editor_application::AudioEditorAction |

## skin_editor_application::AudioEditorError

[源码:17](../../../crates/application/src/audio_editor.rs#L17) · enum

```rust
pub enum AudioEditorError {
    Unavailable,
    Busy,
    MissingComponent,
    MissingAsset,
    ImageAsset,
    InvalidVolume,
    Synchronization(SkinAudioError),
    Backend(String),
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::audio_editor::AudioEditorError`

声明引用：`skin_editor_application::SkinAudioError`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::AudioEditorError::clone`, `skin_editor_application::AudioEditorError::eq`, `skin_editor_application::AudioEditorPort (声明)`, `skin_editor_application::AudioEditorPort::audition_asset`, `skin_editor_application::AudioEditorPort::request`, `skin_editor_application::DisabledAudio::audition_asset`, `skin_editor_application::DisabledAudio::request`, `skin_editor_application::ProjectSession::control_audio`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for AudioEditorError` — [源码:16](../../../crates/application/src/audio_editor.rs#L16)
- `impl Debug for AudioEditorError` — [源码:16](../../../crates/application/src/audio_editor.rs#L16)
- `impl PartialEq for AudioEditorError` — [源码:16](../../../crates/application/src/audio_editor.rs#L16)
- `impl Eq for AudioEditorError` — [源码:16](../../../crates/application/src/audio_editor.rs#L16)
- `impl StructuralPartialEq for AudioEditorError` — [源码:16](../../../crates/application/src/audio_editor.rs#L16)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> AudioEditorError` | impl Clone for AudioEditorError | [源码:16](../../../crates/application/src/audio_editor.rs#L16) | skin_editor_application::AudioEditorError |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for AudioEditorError | [源码:16](../../../crates/application/src/audio_editor.rs#L16) |  |
| `fn eq(&self, other: &AudioEditorError) -> bool` | impl PartialEq for AudioEditorError | [源码:16](../../../crates/application/src/audio_editor.rs#L16) | skin_editor_application::AudioEditorError |

## skin_editor_application::AudioEditorPort

[源码:83](../../../crates/application/src/audio_editor.rs#L83) · trait

```rust
pub trait AudioEditorPort: SkinAudioPort {
    // Required methods
    fn request(
        &mut self,
        action: AudioEditorAction,
    ) -> Result<(), AudioEditorError>;
    fn poll(&mut self) -> AudioEditorState;

    // Provided method
    fn audition_asset(
        &mut self,
        _: AssetAudition,
    ) -> Result<(), AudioEditorError> { ... }
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::audio_editor::AudioEditorPort`

声明引用：`skin_editor_application::AssetAudition`, `skin_editor_application::AudioEditorAction`, `skin_editor_application::AudioEditorError`, `skin_editor_application::AudioEditorState`, `skin_editor_application::SkinAudioPort`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::ProjectSession::control_audio`

实现／约束（显式或 derive，无类继承）：

- `impl AudioEditorPort for DisabledAudio` — [源码:102](../../../crates/application/src/audio_editor.rs#L102)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn request(&mut self, action: AudioEditorAction) -> Result<(), AudioEditorError>` |  | [源码:87](../../../crates/application/src/audio_editor.rs#L87) | skin_editor_application::AudioEditorAction, skin_editor_application::AudioEditorError |
| `fn poll(&mut self) -> AudioEditorState` |  | [源码:88](../../../crates/application/src/audio_editor.rs#L88) | skin_editor_application::AudioEditorState |
| `fn audition_asset(&mut self, _: AssetAudition) -> Result<(), AudioEditorError>` |  | [源码:84](../../../crates/application/src/audio_editor.rs#L84) | skin_editor_application::AssetAudition, skin_editor_application::AudioEditorError |

## skin_editor_application::AudioEditorState

[源码:66](../../../crates/application/src/audio_editor.rs#L66) · struct

```rust
pub struct AudioEditorState {
    pub submitted_nonzero_samples: u64,
    pub output: Option<AudioOutputObservation>,
    pub enabled: bool,
    pub monitor_volume: u16,
    pub asset_audition: Option<AssetAuditionState>,
    pub source: Option<SkinAudioSource>,
    pub components: Vec<AudioComponentState>,
    pub frozen: bool,
    pub freezing: bool,
    pub pending_controls: bool,
    pub voices: [usize; 2],
    pub last_audition: Option<(SkinAudioSource, bool)>,
    pub error: Option<String>,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::audio_editor::AudioEditorState`

声明引用：`skin_editor_application::AssetAuditionState`, `skin_editor_application::AudioComponentState`, `skin_editor_application::AudioOutputObservation`, `skin_editor_application::SkinAudioSource`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::AudioEditorPort (声明)`, `skin_editor_application::AudioEditorPort::poll`, `skin_editor_application::AudioEditorState::clone`, `skin_editor_application::AudioEditorState::default`, `skin_editor_application::AudioEditorState::eq`, `skin_editor_application::DisabledAudio::poll`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for AudioEditorState` — [源码:65](../../../crates/application/src/audio_editor.rs#L65)
- `impl Debug for AudioEditorState` — [源码:65](../../../crates/application/src/audio_editor.rs#L65)
- `impl Default for AudioEditorState` — [源码:65](../../../crates/application/src/audio_editor.rs#L65)
- `impl PartialEq for AudioEditorState` — [源码:65](../../../crates/application/src/audio_editor.rs#L65)
- `impl Eq for AudioEditorState` — [源码:65](../../../crates/application/src/audio_editor.rs#L65)
- `impl StructuralPartialEq for AudioEditorState` — [源码:65](../../../crates/application/src/audio_editor.rs#L65)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> AudioEditorState` | impl Clone for AudioEditorState | [源码:65](../../../crates/application/src/audio_editor.rs#L65) | skin_editor_application::AudioEditorState |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for AudioEditorState | [源码:65](../../../crates/application/src/audio_editor.rs#L65) |  |
| `fn default() -> AudioEditorState` | impl Default for AudioEditorState | [源码:65](../../../crates/application/src/audio_editor.rs#L65) | skin_editor_application::AudioEditorState |
| `fn eq(&self, other: &AudioEditorState) -> bool` | impl PartialEq for AudioEditorState | [源码:65](../../../crates/application/src/audio_editor.rs#L65) | skin_editor_application::AudioEditorState |

## skin_editor_application::AudioOutputObservation

[源码:54](../../../crates/application/src/audio_editor.rs#L54) · struct

```rust
pub struct AudioOutputObservation {
    pub candidate: bool,
    pub rate: u32,
    pub channels: u16,
    pub callbacks: u64,
    pub missing_frames: u64,
    pub generated_background: u64,
    pub copied_background: u64,
    pub background_frames: u64,
    pub pcm_bytes: usize,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::audio_editor::AudioOutputObservation`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::AudioEditorState (声明)`, `skin_editor_application::AudioOutputObservation::clone`, `skin_editor_application::AudioOutputObservation::default`, `skin_editor_application::AudioOutputObservation::eq`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for AudioOutputObservation` — [源码:53](../../../crates/application/src/audio_editor.rs#L53)
- `impl Debug for AudioOutputObservation` — [源码:53](../../../crates/application/src/audio_editor.rs#L53)
- `impl Default for AudioOutputObservation` — [源码:53](../../../crates/application/src/audio_editor.rs#L53)
- `impl PartialEq for AudioOutputObservation` — [源码:53](../../../crates/application/src/audio_editor.rs#L53)
- `impl Eq for AudioOutputObservation` — [源码:53](../../../crates/application/src/audio_editor.rs#L53)
- `impl StructuralPartialEq for AudioOutputObservation` — [源码:53](../../../crates/application/src/audio_editor.rs#L53)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> AudioOutputObservation` | impl Clone for AudioOutputObservation | [源码:53](../../../crates/application/src/audio_editor.rs#L53) | skin_editor_application::AudioOutputObservation |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for AudioOutputObservation | [源码:53](../../../crates/application/src/audio_editor.rs#L53) |  |
| `fn default() -> AudioOutputObservation` | impl Default for AudioOutputObservation | [源码:53](../../../crates/application/src/audio_editor.rs#L53) | skin_editor_application::AudioOutputObservation |
| `fn eq(&self, other: &AudioOutputObservation) -> bool` | impl PartialEq for AudioOutputObservation | [源码:53](../../../crates/application/src/audio_editor.rs#L53) | skin_editor_application::AudioOutputObservation |

## skin_editor_application::AudioPreparationState

[源码:28](../../../crates/application/src/audio_editor.rs#L28) · enum

```rust
pub enum AudioPreparationState {
    Pending,
    Ready,
    Failed(String),
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::audio_editor::AudioPreparationState`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::AssetAuditionState (声明)`, `skin_editor_application::AudioComponentState (声明)`, `skin_editor_application::AudioPreparationState::clone`, `skin_editor_application::AudioPreparationState::eq`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for AudioPreparationState` — [源码:27](../../../crates/application/src/audio_editor.rs#L27)
- `impl Debug for AudioPreparationState` — [源码:27](../../../crates/application/src/audio_editor.rs#L27)
- `impl PartialEq for AudioPreparationState` — [源码:27](../../../crates/application/src/audio_editor.rs#L27)
- `impl Eq for AudioPreparationState` — [源码:27](../../../crates/application/src/audio_editor.rs#L27)
- `impl StructuralPartialEq for AudioPreparationState` — [源码:27](../../../crates/application/src/audio_editor.rs#L27)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> AudioPreparationState` | impl Clone for AudioPreparationState | [源码:27](../../../crates/application/src/audio_editor.rs#L27) | skin_editor_application::AudioPreparationState |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for AudioPreparationState | [源码:27](../../../crates/application/src/audio_editor.rs#L27) |  |
| `fn eq(&self, other: &AudioPreparationState) -> bool` | impl PartialEq for AudioPreparationState | [源码:27](../../../crates/application/src/audio_editor.rs#L27) | skin_editor_application::AudioPreparationState |

## skin_editor_application::BlobLease

[源码:16](../../../crates/application/src/resources.rs#L16) · trait

```rust
pub trait BlobLease: Send + Sync {
    // Required method
    fn reference(&self) -> BlobRef;
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::resources::BlobLease`

声明引用：`skin_editor_domain::BlobRef`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::ProjectSession (声明)`, `skin_editor_application::ProjectSession::pin_candidate`, `skin_editor_application::ProjectSession::publish`, `skin_editor_application::ResourceStore (声明)`, `skin_editor_application::ResourceStore::pin`, `skin_editor_application::StagedRaster (声明)`, `skin_editor_application::draft::Draft (声明)`, `skin_editor_application::resources::NoResources::pin`

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn reference(&self) -> BlobRef` |  | [源码:17](../../../crates/application/src/resources.rs#L17) | skin_editor_domain::BlobRef |

## skin_editor_application::BranchCoordinate

[源码:291](../../../crates/application/src/contract.rs#L291) · struct

```rust
pub struct BranchCoordinate {
    branch: NonZeroU64,
    ordinal: u64,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::contract::BranchCoordinate`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::BranchCoordinate::clone`, `skin_editor_application::BranchCoordinate::cmp`, `skin_editor_application::BranchCoordinate::eq`, `skin_editor_application::BranchCoordinate::partial_cmp`, `skin_editor_application::HistoryQueryError (声明)`, `skin_editor_application::HistorySummary (声明)`, `skin_editor_application::history::AppendPlan (声明)`, `skin_editor_application::history::History (声明)`, `skin_editor_application::history::Node (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl BranchCoordinate` — [源码:55](../../../crates/application/src/address.rs#L55)
- `impl BranchCoordinate` — [源码:295](../../../crates/application/src/contract.rs#L295)
- `impl Clone for BranchCoordinate` — [源码:290](../../../crates/application/src/contract.rs#L290)
- `impl Debug for BranchCoordinate` — [源码:290](../../../crates/application/src/contract.rs#L290)
- `impl Ord for BranchCoordinate` — [源码:290](../../../crates/application/src/contract.rs#L290)
- `impl PartialEq for BranchCoordinate` — [源码:290](../../../crates/application/src/contract.rs#L290)
- `impl PartialOrd for BranchCoordinate` — [源码:290](../../../crates/application/src/contract.rs#L290)
- `impl Copy for BranchCoordinate` — [源码:290](../../../crates/application/src/contract.rs#L290)
- `impl Eq for BranchCoordinate` — [源码:290](../../../crates/application/src/contract.rs#L290)
- `impl StructuralPartialEq for BranchCoordinate` — [源码:290](../../../crates/application/src/contract.rs#L290)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn short_address(self) -> String` | impl BranchCoordinate | [源码:56](../../../crates/application/src/address.rs#L56) |  |
| `pub fn parse_address(text: &str) -> Result<Self, AddressError>` | impl BranchCoordinate | [源码:60](../../../crates/application/src/address.rs#L60) | skin_editor_application::AddressError |
| `pub const fn new(branch: u64, ordinal: u64) -> Option<Self>` | impl BranchCoordinate | [源码:296](../../../crates/application/src/contract.rs#L296) |  |
| `pub const fn branch(self) -> u64` | impl BranchCoordinate | [源码:302](../../../crates/application/src/contract.rs#L302) |  |
| `pub const fn ordinal(self) -> u64` | impl BranchCoordinate | [源码:305](../../../crates/application/src/contract.rs#L305) |  |
| `fn clone(&self) -> BranchCoordinate` | impl Clone for BranchCoordinate | [源码:290](../../../crates/application/src/contract.rs#L290) | skin_editor_application::BranchCoordinate |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for BranchCoordinate | [源码:290](../../../crates/application/src/contract.rs#L290) |  |
| `fn cmp(&self, other: &BranchCoordinate) -> Ordering` | impl Ord for BranchCoordinate | [源码:290](../../../crates/application/src/contract.rs#L290) | skin_editor_application::BranchCoordinate |
| `fn eq(&self, other: &BranchCoordinate) -> bool` | impl PartialEq for BranchCoordinate | [源码:290](../../../crates/application/src/contract.rs#L290) | skin_editor_application::BranchCoordinate |
| `fn partial_cmp(&self, other: &BranchCoordinate) -> Option<Ordering>` | impl PartialOrd for BranchCoordinate | [源码:290](../../../crates/application/src/contract.rs#L290) | skin_editor_application::BranchCoordinate |

## skin_editor_application::Command

[源码:64](../../../crates/application/src/contract.rs#L64) · enum

```rust
pub enum Command {
    RegisterAsset {
        asset: Asset,
    },
    RemoveAsset {
        asset: AssetId,
    },
    RenameAsset {
        asset: AssetId,
        name: String,
    },
    CreateAudioComponent {
        component: AudioComponent,
    },
    RemoveAudioComponent {
        component: AudioComponentId,
    },
    SetAudioAsset {
        component: AudioComponentId,
        asset: AssetId,
    },
    SetAudioProperties {
        component: AudioComponentId,
        gain: Gain,
        visible: bool,
    },
    SetDocumentVisibility {
        document: DocumentId,
        visible: bool,
    },
    CreateDocument {
        document: ElementDocument,
    },
    RemoveDocument {
        document: DocumentId,
    },
    ResizeDocument {
        document: DocumentId,
        canvas: PixelSize,
    },
    AddLayer {
        document: DocumentId,
        node: LayerNode,
        parent: Option<LayerId>,
        index: usize,
    },
    RemoveLayer {
        document: DocumentId,
        layer: LayerId,
    },
    MoveLayer {
        document: DocumentId,
        layer: LayerId,
        parent: Option<LayerId>,
        index: usize,
    },
    SetLayerTransform {
        document: DocumentId,
        layer: LayerId,
        transform: Affine2,
    },
    SetLayerProperties {
        document: DocumentId,
        layer: LayerId,
        opacity: Opacity,
        visible: bool,
    },
    SetLayerAsset {
        document: DocumentId,
        layer: LayerId,
        asset: AssetId,
    },
    AddComponent {
        node: ComponentNode,
        parent: Option<ComponentId>,
        index: usize,
    },
    RemoveComponent {
        component: ComponentId,
    },
    MoveComponent {
        component: ComponentId,
        parent: Option<ComponentId>,
        index: usize,
    },
    SetComponentTransform {
        component: ComponentId,
        transform: Affine2,
    },
    SetComponentMapping {
        component: ComponentId,
        document_to_component: Affine2,
    },
    SetOutputMapping {
        mapping: OutputMapping,
    },
    RemoveOutputMapping {
        output: OutputId,
    },
    InsertConfigRecord {
        index: usize,
        record: ConfigRecord,
    },
    SetConfigRecord {
        index: usize,
        record: ConfigRecord,
    },
    RemoveConfigRecord {
        index: usize,
    },
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::contract::Command`

声明引用：`skin_editor_domain::Affine2`, `skin_editor_domain::Asset`, `skin_editor_domain::AssetId`, `skin_editor_domain::AudioComponent`, `skin_editor_domain::AudioComponentId`, `skin_editor_domain::ComponentId`, `skin_editor_domain::ComponentNode`, `skin_editor_domain::ConfigRecord`, `skin_editor_domain::DocumentId`, `skin_editor_domain::ElementDocument`, `skin_editor_domain::Gain`, `skin_editor_domain::LayerId`, `skin_editor_domain::LayerNode`, `skin_editor_domain::Opacity`, `skin_editor_domain::OutputId`, `skin_editor_domain::OutputMapping`, `skin_editor_domain::PixelSize`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::Command::clone`, `skin_editor_application::Command::eq`, `skin_editor_application::CommandRequest (声明)`, `skin_editor_application::DraftAction (声明)`, `skin_editor_application::ProjectSession::prepare`, `skin_editor_application::draft::Draft (声明)`, `skin_editor_application::execute::apply (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for Command` — [源码:63](../../../crates/application/src/contract.rs#L63)
- `impl Debug for Command` — [源码:63](../../../crates/application/src/contract.rs#L63)
- `impl PartialEq for Command` — [源码:63](../../../crates/application/src/contract.rs#L63)
- `impl StructuralPartialEq for Command` — [源码:63](../../../crates/application/src/contract.rs#L63)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> Command` | impl Clone for Command | [源码:63](../../../crates/application/src/contract.rs#L63) | skin_editor_application::Command |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for Command | [源码:63](../../../crates/application/src/contract.rs#L63) |  |
| `fn eq(&self, other: &Command) -> bool` | impl PartialEq for Command | [源码:63](../../../crates/application/src/contract.rs#L63) | skin_editor_application::Command |

## skin_editor_application::CommandError

[源码:234](../../../crates/application/src/contract.rs#L234) · enum

```rust
pub enum CommandError {
    UnsupportedVersion(u16),
    WrongSession,
    RequestIdReused,
    Conflict {
        expected: ExpectedState,
        actual: ExpectedState,
    },
    Command {
        index: usize,
        reason: CommandFailure,
    },
    Resource {
        blob: BlobRef,
        error: ResourceError,
    },
    ResourceLimit(&'static str),
    History(HistoryQueryError),
    Busy(DraftId),
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::contract::CommandError`

声明引用：`skin_editor_application::CommandFailure`, `skin_editor_application::DraftId`, `skin_editor_application::ExpectedState`, `skin_editor_application::HistoryQueryError`, `skin_editor_application::ResourceError`, `skin_editor_domain::BlobRef`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::CommandError::clone`, `skin_editor_application::CommandError::eq`, `skin_editor_application::CommandError::from`, `skin_editor_application::DraftError (声明)`, `skin_editor_application::DraftError::from`, `skin_editor_application::ProjectSession::cached`, `skin_editor_application::ProjectSession::commit`, `skin_editor_application::ProjectSession::envelope`, `skin_editor_application::ProjectSession::pin_candidate`, `skin_editor_application::ProjectSession::prepare`, `skin_editor_application::ProjectSession::publish`, `skin_editor_application::ProjectSession::submit`, `skin_editor_application::ProjectSession::validate`, `skin_editor_application::Receipt (声明)`, `skin_editor_application::history::History::plan_append`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for CommandError` — [源码:233](../../../crates/application/src/contract.rs#L233)
- `impl Debug for CommandError` — [源码:233](../../../crates/application/src/contract.rs#L233)
- `impl From<CommandError> for DraftError` — [源码:92](../../../crates/application/src/draft_contract.rs#L92)
- `impl PartialEq for CommandError` — [源码:233](../../../crates/application/src/contract.rs#L233)
- `impl Eq for CommandError` — [源码:233](../../../crates/application/src/contract.rs#L233)
- `impl StructuralPartialEq for CommandError` — [源码:233](../../../crates/application/src/contract.rs#L233)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> CommandError` | impl Clone for CommandError | [源码:233](../../../crates/application/src/contract.rs#L233) | skin_editor_application::CommandError |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for CommandError | [源码:233](../../../crates/application/src/contract.rs#L233) |  |
| `fn from(error: CommandError) -> Self` | impl From<CommandError> for DraftError | [源码:93](../../../crates/application/src/draft_contract.rs#L93) | skin_editor_application::CommandError |
| `fn eq(&self, other: &CommandError) -> bool` | impl PartialEq for CommandError | [源码:233](../../../crates/application/src/contract.rs#L233) | skin_editor_application::CommandError |

## skin_editor_application::CommandFailure

[源码:204](../../../crates/application/src/contract.rs#L204) · enum

```rust
pub enum CommandFailure {
    Missing(ObjectId),
    ReusedIdentity(ObjectId),
    InvalidIndex {
        index: usize,
        len: usize,
    },
    InvalidOperation(&'static str),
    InvalidParent(ObjectId),
    WouldCreateCycle(ObjectId),
    AssetInUse {
        asset: AssetId,
        layers: Vec<LayerId>,
    },
    DocumentInUse(DocumentId),
    AudioAssetInUse {
        asset: AssetId,
        components: Vec<AudioComponentId>,
    },
    AudioComponentInUse(AudioComponentId),
    Model(ModelError),
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::contract::CommandFailure`

声明引用：`skin_editor_domain::AssetId`, `skin_editor_domain::AudioComponentId`, `skin_editor_domain::DocumentId`, `skin_editor_domain::LayerId`, `skin_editor_domain::ModelError`, `skin_editor_domain::ObjectId`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::CommandError (声明)`, `skin_editor_application::CommandFailure::clone`, `skin_editor_application::CommandFailure::eq`, `skin_editor_application::execute::apply (声明)`, `skin_editor_application::execute::component_children (声明)`, `skin_editor_application::execute::component_node (声明)`, `skin_editor_application::execute::components (声明)`, `skin_editor_application::execute::doc_index (声明)`, `skin_editor_application::execute::layer_children (声明)`, `skin_editor_application::execute::layer_node (声明)`, `skin_editor_application::execute::layers (声明)`, `skin_editor_application::execute::position (声明)`, `skin_editor_application::execute::reserve (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for CommandFailure` — [源码:203](../../../crates/application/src/contract.rs#L203)
- `impl Debug for CommandFailure` — [源码:203](../../../crates/application/src/contract.rs#L203)
- `impl From<ModelError> for CommandFailure` — [源码:227](../../../crates/application/src/contract.rs#L227)
- `impl PartialEq for CommandFailure` — [源码:203](../../../crates/application/src/contract.rs#L203)
- `impl Eq for CommandFailure` — [源码:203](../../../crates/application/src/contract.rs#L203)
- `impl StructuralPartialEq for CommandFailure` — [源码:203](../../../crates/application/src/contract.rs#L203)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> CommandFailure` | impl Clone for CommandFailure | [源码:203](../../../crates/application/src/contract.rs#L203) | skin_editor_application::CommandFailure |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for CommandFailure | [源码:203](../../../crates/application/src/contract.rs#L203) |  |
| `fn from(value: ModelError) -> Self` | impl From<ModelError> for CommandFailure | [源码:228](../../../crates/application/src/contract.rs#L228) | skin_editor_domain::ModelError |
| `fn eq(&self, other: &CommandFailure) -> bool` | impl PartialEq for CommandFailure | [源码:203](../../../crates/application/src/contract.rs#L203) | skin_editor_application::CommandFailure |

## skin_editor_application::CommandRequest

[源码:177](../../../crates/application/src/contract.rs#L177) · struct

```rust
pub struct CommandRequest {
    pub api_version: u16,
    pub session_id: HistorySessionId,
    pub request_id: RequestId,
    pub expected: ExpectedState,
    pub commands: Vec<Command>,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::contract::CommandRequest`

声明引用：`skin_editor_application::Command`, `skin_editor_application::ExpectedState`, `skin_editor_application::HistorySessionId`, `skin_editor_application::RequestId`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::CommandRequest::clone`, `skin_editor_application::CommandRequest::eq`, `skin_editor_application::ProjectSession::cached`, `skin_editor_application::ProjectSession::commit`, `skin_editor_application::ProjectSession::envelope`, `skin_editor_application::ProjectSession::submit`, `skin_editor_application::ProjectSession::validate`, `skin_editor_application::Receipt (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for CommandRequest` — [源码:176](../../../crates/application/src/contract.rs#L176)
- `impl Debug for CommandRequest` — [源码:176](../../../crates/application/src/contract.rs#L176)
- `impl PartialEq for CommandRequest` — [源码:176](../../../crates/application/src/contract.rs#L176)
- `impl StructuralPartialEq for CommandRequest` — [源码:176](../../../crates/application/src/contract.rs#L176)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> CommandRequest` | impl Clone for CommandRequest | [源码:176](../../../crates/application/src/contract.rs#L176) | skin_editor_application::CommandRequest |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for CommandRequest | [源码:176](../../../crates/application/src/contract.rs#L176) |  |
| `fn eq(&self, other: &CommandRequest) -> bool` | impl PartialEq for CommandRequest | [源码:176](../../../crates/application/src/contract.rs#L176) | skin_editor_application::CommandRequest |

## skin_editor_application::CommandResult

[源码:192](../../../crates/application/src/contract.rs#L192) · struct

```rust
pub struct CommandResult {
    pub api_version: u16,
    pub session_id: HistorySessionId,
    pub request_id: RequestId,
    pub state: ExpectedState,
    pub status: CommandStatus,
    pub changed_objects: Vec<ObjectId>,
    pub configuration_changed: bool,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::contract::CommandResult`

声明引用：`skin_editor_application::CommandStatus`, `skin_editor_application::ExpectedState`, `skin_editor_application::HistorySessionId`, `skin_editor_application::RequestId`, `skin_editor_domain::ObjectId`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::CommandResult::clone`, `skin_editor_application::CommandResult::eq`, `skin_editor_application::DraftOutcome (声明)`, `skin_editor_application::ProjectSession::cached`, `skin_editor_application::ProjectSession::commit`, `skin_editor_application::ProjectSession::publish`, `skin_editor_application::ProjectSession::submit`, `skin_editor_application::Receipt (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for CommandResult` — [源码:191](../../../crates/application/src/contract.rs#L191)
- `impl Debug for CommandResult` — [源码:191](../../../crates/application/src/contract.rs#L191)
- `impl PartialEq for CommandResult` — [源码:191](../../../crates/application/src/contract.rs#L191)
- `impl Eq for CommandResult` — [源码:191](../../../crates/application/src/contract.rs#L191)
- `impl StructuralPartialEq for CommandResult` — [源码:191](../../../crates/application/src/contract.rs#L191)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> CommandResult` | impl Clone for CommandResult | [源码:191](../../../crates/application/src/contract.rs#L191) | skin_editor_application::CommandResult |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for CommandResult | [源码:191](../../../crates/application/src/contract.rs#L191) |  |
| `fn eq(&self, other: &CommandResult) -> bool` | impl PartialEq for CommandResult | [源码:191](../../../crates/application/src/contract.rs#L191) | skin_editor_application::CommandResult |

## skin_editor_application::CommandStatus

[源码:186](../../../crates/application/src/contract.rs#L186) · enum

```rust
pub enum CommandStatus {
    Committed,
    NoChange,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::contract::CommandStatus`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::CommandResult (声明)`, `skin_editor_application::CommandStatus::clone`, `skin_editor_application::CommandStatus::eq`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for CommandStatus` — [源码:185](../../../crates/application/src/contract.rs#L185)
- `impl Debug for CommandStatus` — [源码:185](../../../crates/application/src/contract.rs#L185)
- `impl PartialEq for CommandStatus` — [源码:185](../../../crates/application/src/contract.rs#L185)
- `impl Copy for CommandStatus` — [源码:185](../../../crates/application/src/contract.rs#L185)
- `impl Eq for CommandStatus` — [源码:185](../../../crates/application/src/contract.rs#L185)
- `impl StructuralPartialEq for CommandStatus` — [源码:185](../../../crates/application/src/contract.rs#L185)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> CommandStatus` | impl Clone for CommandStatus | [源码:185](../../../crates/application/src/contract.rs#L185) | skin_editor_application::CommandStatus |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for CommandStatus | [源码:185](../../../crates/application/src/contract.rs#L185) |  |
| `fn eq(&self, other: &CommandStatus) -> bool` | impl PartialEq for CommandStatus | [源码:185](../../../crates/application/src/contract.rs#L185) | skin_editor_application::CommandStatus |

## skin_editor_application::DisabledAudio

[源码:91](../../../crates/application/src/audio_editor.rs#L91) · struct

```rust
pub struct DisabledAudio;
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::audio_editor::DisabledAudio`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::DisabledAudio::default`

实现／约束（显式或 derive，无类继承）：

- `impl AudioEditorPort for DisabledAudio` — [源码:102](../../../crates/application/src/audio_editor.rs#L102)
- `impl Default for DisabledAudio` — [源码:90](../../../crates/application/src/audio_editor.rs#L90)
- `impl SkinAudioPort for DisabledAudio` — [源码:92](../../../crates/application/src/audio_editor.rs#L92)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn request(&mut self, _: AudioEditorAction) -> Result<(), AudioEditorError>` | impl AudioEditorPort for DisabledAudio | [源码:103](../../../crates/application/src/audio_editor.rs#L103) | skin_editor_application::AudioEditorAction, skin_editor_application::AudioEditorError |
| `fn poll(&mut self) -> AudioEditorState` | impl AudioEditorPort for DisabledAudio | [源码:106](../../../crates/application/src/audio_editor.rs#L106) | skin_editor_application::AudioEditorState |
| `fn audition_asset(&mut self, _: AssetAudition) -> Result<(), AudioEditorError>` | impl AudioEditorPort for DisabledAudio | [源码:84](../../../crates/application/src/audio_editor.rs#L84) | skin_editor_application::AssetAudition, skin_editor_application::AudioEditorError |
| `fn default() -> DisabledAudio` | impl Default for DisabledAudio | [源码:90](../../../crates/application/src/audio_editor.rs#L90) | skin_editor_application::DisabledAudio |
| `fn synchronize( &mut self, _: &SkinProject, _: SkinAudioSource, _: &[DraftId], ) -> Result<(), SkinAudioError>` | impl SkinAudioPort for DisabledAudio | [源码:93](../../../crates/application/src/audio_editor.rs#L93) | skin_editor_application::DraftId, skin_editor_application::SkinAudioError, skin_editor_application::SkinAudioSource, skin_editor_domain::SkinProject |
| `fn retained_drafts(&self) -> Vec<DraftId>` | impl SkinAudioPort for DisabledAudio | [源码:24](../../../crates/application/src/audio_port.rs#L24) | skin_editor_application::DraftId |

## skin_editor_application::DraftAction

[源码:28](../../../crates/application/src/draft_contract.rs#L28) · enum

```rust
pub enum DraftAction {
    Begin {
        expected: ExpectedState,
    },
    Replace {
        expected: DraftState,
        commands: Vec<Command>,
    },
    Commit {
        expected: DraftState,
    },
    Cancel {
        expected: DraftState,
    },
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::draft_contract::DraftAction`

声明引用：`skin_editor_application::Command`, `skin_editor_application::DraftState`, `skin_editor_application::ExpectedState`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::DraftAction::clone`, `skin_editor_application::DraftAction::eq`, `skin_editor_application::DraftRequest (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for DraftAction` — [源码:27](../../../crates/application/src/draft_contract.rs#L27)
- `impl Debug for DraftAction` — [源码:27](../../../crates/application/src/draft_contract.rs#L27)
- `impl PartialEq for DraftAction` — [源码:27](../../../crates/application/src/draft_contract.rs#L27)
- `impl StructuralPartialEq for DraftAction` — [源码:27](../../../crates/application/src/draft_contract.rs#L27)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> DraftAction` | impl Clone for DraftAction | [源码:27](../../../crates/application/src/draft_contract.rs#L27) | skin_editor_application::DraftAction |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for DraftAction | [源码:27](../../../crates/application/src/draft_contract.rs#L27) |  |
| `fn eq(&self, other: &DraftAction) -> bool` | impl PartialEq for DraftAction | [源码:27](../../../crates/application/src/draft_contract.rs#L27) | skin_editor_application::DraftAction |

## skin_editor_application::DraftError

[源码:84](../../../crates/application/src/draft_contract.rs#L84) · enum

```rust
pub enum DraftError {
    Request(CommandError),
    NoActiveDraft,
    Conflict {
        expected: DraftState,
        actual: DraftState,
    },
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::draft_contract::DraftError`

声明引用：`skin_editor_application::CommandError`, `skin_editor_application::DraftState`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::DraftError::clone`, `skin_editor_application::DraftError::eq`, `skin_editor_application::ProjectSession::draft_info`, `skin_editor_application::ProjectSession::draft_once`, `skin_editor_application::ProjectSession::draft_snapshot`, `skin_editor_application::ProjectSession::edit_draft`, `skin_editor_application::ProjectSession::expected_draft`, `skin_editor_application::Receipt (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for DraftError` — [源码:83](../../../crates/application/src/draft_contract.rs#L83)
- `impl Debug for DraftError` — [源码:83](../../../crates/application/src/draft_contract.rs#L83)
- `impl From<CommandError> for DraftError` — [源码:92](../../../crates/application/src/draft_contract.rs#L92)
- `impl PartialEq for DraftError` — [源码:83](../../../crates/application/src/draft_contract.rs#L83)
- `impl Eq for DraftError` — [源码:83](../../../crates/application/src/draft_contract.rs#L83)
- `impl StructuralPartialEq for DraftError` — [源码:83](../../../crates/application/src/draft_contract.rs#L83)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> DraftError` | impl Clone for DraftError | [源码:83](../../../crates/application/src/draft_contract.rs#L83) | skin_editor_application::DraftError |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for DraftError | [源码:83](../../../crates/application/src/draft_contract.rs#L83) |  |
| `fn from(error: CommandError) -> Self` | impl From<CommandError> for DraftError | [源码:93](../../../crates/application/src/draft_contract.rs#L93) | skin_editor_application::CommandError |
| `fn eq(&self, other: &DraftError) -> bool` | impl PartialEq for DraftError | [源码:83](../../../crates/application/src/draft_contract.rs#L83) | skin_editor_application::DraftError |

## skin_editor_application::DraftId

[源码:7](../../../crates/application/src/draft_contract.rs#L7) · struct

```rust
pub struct DraftId(NonZeroU64);
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::draft_contract::DraftId`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::CommandError (声明)`, `skin_editor_application::DisabledAudio::retained_drafts`, `skin_editor_application::DisabledAudio::synchronize`, `skin_editor_application::DraftId::clone`, `skin_editor_application::DraftId::cmp`, `skin_editor_application::DraftId::eq`, `skin_editor_application::DraftId::partial_cmp`, `skin_editor_application::DraftState (声明)`, `skin_editor_application::HistoryError (声明)`, `skin_editor_application::ProjectSession (声明)`, `skin_editor_application::SkinAudioPort (声明)`, `skin_editor_application::SkinAudioPort::retained_drafts`, `skin_editor_application::SkinAudioPort::synchronize`

实现／约束（显式或 derive，无类继承）：

- `impl DraftId` — [源码:8](../../../crates/application/src/draft_contract.rs#L8)
- `impl Clone for DraftId` — [源码:6](../../../crates/application/src/draft_contract.rs#L6)
- `impl Debug for DraftId` — [源码:6](../../../crates/application/src/draft_contract.rs#L6)
- `impl Ord for DraftId` — [源码:6](../../../crates/application/src/draft_contract.rs#L6)
- `impl PartialEq for DraftId` — [源码:6](../../../crates/application/src/draft_contract.rs#L6)
- `impl PartialOrd for DraftId` — [源码:6](../../../crates/application/src/draft_contract.rs#L6)
- `impl Copy for DraftId` — [源码:6](../../../crates/application/src/draft_contract.rs#L6)
- `impl Eq for DraftId` — [源码:6](../../../crates/application/src/draft_contract.rs#L6)
- `impl StructuralPartialEq for DraftId` — [源码:6](../../../crates/application/src/draft_contract.rs#L6)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub const fn new(value: u64) -> Option<Self>` | impl DraftId | [源码:9](../../../crates/application/src/draft_contract.rs#L9) |  |
| `pub const fn get(self) -> u64` | impl DraftId | [源码:15](../../../crates/application/src/draft_contract.rs#L15) |  |
| `fn clone(&self) -> DraftId` | impl Clone for DraftId | [源码:6](../../../crates/application/src/draft_contract.rs#L6) | skin_editor_application::DraftId |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for DraftId | [源码:6](../../../crates/application/src/draft_contract.rs#L6) |  |
| `fn cmp(&self, other: &DraftId) -> Ordering` | impl Ord for DraftId | [源码:6](../../../crates/application/src/draft_contract.rs#L6) | skin_editor_application::DraftId |
| `fn eq(&self, other: &DraftId) -> bool` | impl PartialEq for DraftId | [源码:6](../../../crates/application/src/draft_contract.rs#L6) | skin_editor_application::DraftId |
| `fn partial_cmp(&self, other: &DraftId) -> Option<Ordering>` | impl PartialOrd for DraftId | [源码:6](../../../crates/application/src/draft_contract.rs#L6) | skin_editor_application::DraftId |

## skin_editor_application::DraftInfo

[源码:55](../../../crates/application/src/draft_contract.rs#L55) · struct

```rust
pub struct DraftInfo {
    pub api_version: u16,
    pub session_id: HistorySessionId,
    pub state: DraftState,
    pub has_changes: bool,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::draft_contract::DraftInfo`

声明引用：`skin_editor_application::DraftState`, `skin_editor_application::HistorySessionId`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::DraftInfo::clone`, `skin_editor_application::DraftInfo::eq`, `skin_editor_application::DraftOutcome (声明)`, `skin_editor_application::DraftSnapshot (声明)`, `skin_editor_application::ProjectSession::draft_info`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for DraftInfo` — [源码:54](../../../crates/application/src/draft_contract.rs#L54)
- `impl Debug for DraftInfo` — [源码:54](../../../crates/application/src/draft_contract.rs#L54)
- `impl PartialEq for DraftInfo` — [源码:54](../../../crates/application/src/draft_contract.rs#L54)
- `impl Eq for DraftInfo` — [源码:54](../../../crates/application/src/draft_contract.rs#L54)
- `impl StructuralPartialEq for DraftInfo` — [源码:54](../../../crates/application/src/draft_contract.rs#L54)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> DraftInfo` | impl Clone for DraftInfo | [源码:54](../../../crates/application/src/draft_contract.rs#L54) | skin_editor_application::DraftInfo |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for DraftInfo | [源码:54](../../../crates/application/src/draft_contract.rs#L54) |  |
| `fn eq(&self, other: &DraftInfo) -> bool` | impl PartialEq for DraftInfo | [源码:54](../../../crates/application/src/draft_contract.rs#L54) | skin_editor_application::DraftInfo |

## skin_editor_application::DraftOutcome

[源码:69](../../../crates/application/src/draft_contract.rs#L69) · enum

```rust
pub enum DraftOutcome {
    Active(DraftInfo),
    Committed(CommandResult),
    Cancelled(DraftState),
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::draft_contract::DraftOutcome`

声明引用：`skin_editor_application::CommandResult`, `skin_editor_application::DraftInfo`, `skin_editor_application::DraftState`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::DraftOutcome::clone`, `skin_editor_application::DraftOutcome::eq`, `skin_editor_application::DraftResult (声明)`, `skin_editor_application::ProjectSession::draft_once`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for DraftOutcome` — [源码:68](../../../crates/application/src/draft_contract.rs#L68)
- `impl Debug for DraftOutcome` — [源码:68](../../../crates/application/src/draft_contract.rs#L68)
- `impl PartialEq for DraftOutcome` — [源码:68](../../../crates/application/src/draft_contract.rs#L68)
- `impl Eq for DraftOutcome` — [源码:68](../../../crates/application/src/draft_contract.rs#L68)
- `impl StructuralPartialEq for DraftOutcome` — [源码:68](../../../crates/application/src/draft_contract.rs#L68)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> DraftOutcome` | impl Clone for DraftOutcome | [源码:68](../../../crates/application/src/draft_contract.rs#L68) | skin_editor_application::DraftOutcome |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for DraftOutcome | [源码:68](../../../crates/application/src/draft_contract.rs#L68) |  |
| `fn eq(&self, other: &DraftOutcome) -> bool` | impl PartialEq for DraftOutcome | [源码:68](../../../crates/application/src/draft_contract.rs#L68) | skin_editor_application::DraftOutcome |

## skin_editor_application::DraftRequest

[源码:47](../../../crates/application/src/draft_contract.rs#L47) · struct

```rust
pub struct DraftRequest {
    pub api_version: u16,
    pub session_id: HistorySessionId,
    pub request_id: RequestId,
    pub action: DraftAction,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::draft_contract::DraftRequest`

声明引用：`skin_editor_application::DraftAction`, `skin_editor_application::HistorySessionId`, `skin_editor_application::RequestId`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::DraftRequest::clone`, `skin_editor_application::DraftRequest::eq`, `skin_editor_application::ProjectSession::draft_once`, `skin_editor_application::ProjectSession::edit_draft`, `skin_editor_application::Receipt (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for DraftRequest` — [源码:46](../../../crates/application/src/draft_contract.rs#L46)
- `impl Debug for DraftRequest` — [源码:46](../../../crates/application/src/draft_contract.rs#L46)
- `impl PartialEq for DraftRequest` — [源码:46](../../../crates/application/src/draft_contract.rs#L46)
- `impl StructuralPartialEq for DraftRequest` — [源码:46](../../../crates/application/src/draft_contract.rs#L46)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> DraftRequest` | impl Clone for DraftRequest | [源码:46](../../../crates/application/src/draft_contract.rs#L46) | skin_editor_application::DraftRequest |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for DraftRequest | [源码:46](../../../crates/application/src/draft_contract.rs#L46) |  |
| `fn eq(&self, other: &DraftRequest) -> bool` | impl PartialEq for DraftRequest | [源码:46](../../../crates/application/src/draft_contract.rs#L46) | skin_editor_application::DraftRequest |

## skin_editor_application::DraftResult

[源码:76](../../../crates/application/src/draft_contract.rs#L76) · struct

```rust
pub struct DraftResult {
    pub api_version: u16,
    pub session_id: HistorySessionId,
    pub request_id: RequestId,
    pub outcome: DraftOutcome,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::draft_contract::DraftResult`

声明引用：`skin_editor_application::DraftOutcome`, `skin_editor_application::HistorySessionId`, `skin_editor_application::RequestId`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::DraftResult::clone`, `skin_editor_application::DraftResult::eq`, `skin_editor_application::ProjectSession::edit_draft`, `skin_editor_application::Receipt (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for DraftResult` — [源码:75](../../../crates/application/src/draft_contract.rs#L75)
- `impl Debug for DraftResult` — [源码:75](../../../crates/application/src/draft_contract.rs#L75)
- `impl PartialEq for DraftResult` — [源码:75](../../../crates/application/src/draft_contract.rs#L75)
- `impl Eq for DraftResult` — [源码:75](../../../crates/application/src/draft_contract.rs#L75)
- `impl StructuralPartialEq for DraftResult` — [源码:75](../../../crates/application/src/draft_contract.rs#L75)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> DraftResult` | impl Clone for DraftResult | [源码:75](../../../crates/application/src/draft_contract.rs#L75) | skin_editor_application::DraftResult |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for DraftResult | [源码:75](../../../crates/application/src/draft_contract.rs#L75) |  |
| `fn eq(&self, other: &DraftResult) -> bool` | impl PartialEq for DraftResult | [源码:75](../../../crates/application/src/draft_contract.rs#L75) | skin_editor_application::DraftResult |

## skin_editor_application::DraftSnapshot

[源码:63](../../../crates/application/src/draft_contract.rs#L63) · struct

```rust
pub struct DraftSnapshot {
    pub info: DraftInfo,
    pub project: SkinProject,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::draft_contract::DraftSnapshot`

声明引用：`skin_editor_application::DraftInfo`, `skin_editor_domain::SkinProject`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::DraftSnapshot::clone`, `skin_editor_application::DraftSnapshot::eq`, `skin_editor_application::ProjectSession::draft_snapshot`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for DraftSnapshot` — [源码:62](../../../crates/application/src/draft_contract.rs#L62)
- `impl Debug for DraftSnapshot` — [源码:62](../../../crates/application/src/draft_contract.rs#L62)
- `impl PartialEq for DraftSnapshot` — [源码:62](../../../crates/application/src/draft_contract.rs#L62)
- `impl StructuralPartialEq for DraftSnapshot` — [源码:62](../../../crates/application/src/draft_contract.rs#L62)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> DraftSnapshot` | impl Clone for DraftSnapshot | [源码:62](../../../crates/application/src/draft_contract.rs#L62) | skin_editor_application::DraftSnapshot |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for DraftSnapshot | [源码:62](../../../crates/application/src/draft_contract.rs#L62) |  |
| `fn eq(&self, other: &DraftSnapshot) -> bool` | impl PartialEq for DraftSnapshot | [源码:62](../../../crates/application/src/draft_contract.rs#L62) | skin_editor_application::DraftSnapshot |

## skin_editor_application::DraftState

[源码:21](../../../crates/application/src/draft_contract.rs#L21) · struct

```rust
pub struct DraftState {
    pub id: DraftId,
    pub revision: u64,
    pub base: ExpectedState,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::draft_contract::DraftState`

声明引用：`skin_editor_application::DraftId`, `skin_editor_application::ExpectedState`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::DraftAction (声明)`, `skin_editor_application::DraftError (声明)`, `skin_editor_application::DraftInfo (声明)`, `skin_editor_application::DraftOutcome (声明)`, `skin_editor_application::DraftState::clone`, `skin_editor_application::DraftState::eq`, `skin_editor_application::PreviewContext (声明)`, `skin_editor_application::ProjectSession::draft_snapshot`, `skin_editor_application::ProjectSession::expected_draft`, `skin_editor_application::SkinAudioSource (声明)`, `skin_editor_application::draft::Draft (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for DraftState` — [源码:20](../../../crates/application/src/draft_contract.rs#L20)
- `impl Debug for DraftState` — [源码:20](../../../crates/application/src/draft_contract.rs#L20)
- `impl PartialEq for DraftState` — [源码:20](../../../crates/application/src/draft_contract.rs#L20)
- `impl Copy for DraftState` — [源码:20](../../../crates/application/src/draft_contract.rs#L20)
- `impl Eq for DraftState` — [源码:20](../../../crates/application/src/draft_contract.rs#L20)
- `impl StructuralPartialEq for DraftState` — [源码:20](../../../crates/application/src/draft_contract.rs#L20)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> DraftState` | impl Clone for DraftState | [源码:20](../../../crates/application/src/draft_contract.rs#L20) | skin_editor_application::DraftState |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for DraftState | [源码:20](../../../crates/application/src/draft_contract.rs#L20) |  |
| `fn eq(&self, other: &DraftState) -> bool` | impl PartialEq for DraftState | [源码:20](../../../crates/application/src/draft_contract.rs#L20) | skin_editor_application::DraftState |

## skin_editor_application::ExpectedState

[源码:55](../../../crates/application/src/contract.rs#L55) · struct

```rust
pub struct ExpectedState {
    pub head: NodeId,
    pub revision: u64,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::contract::ExpectedState`

声明引用：`skin_editor_application::NodeId`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::CommandError (声明)`, `skin_editor_application::CommandRequest (声明)`, `skin_editor_application::CommandResult (声明)`, `skin_editor_application::DraftAction (声明)`, `skin_editor_application::DraftState (声明)`, `skin_editor_application::ExpectedState::clone`, `skin_editor_application::ExpectedState::eq`, `skin_editor_application::FrameSource (声明)`, `skin_editor_application::HistoryError (声明)`, `skin_editor_application::HistoryRequest (声明)`, `skin_editor_application::HistoryResult (声明)`, `skin_editor_application::ProjectInfo (声明)`, `skin_editor_application::ProjectSession (声明)`, `skin_editor_application::ProjectSession::prepare`, `skin_editor_application::ProjectSnapshot (声明)`, `skin_editor_application::SkinAudioSource (声明)`, `skin_editor_application::ValidationResult (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for ExpectedState` — [源码:54](../../../crates/application/src/contract.rs#L54)
- `impl Debug for ExpectedState` — [源码:54](../../../crates/application/src/contract.rs#L54)
- `impl PartialEq for ExpectedState` — [源码:54](../../../crates/application/src/contract.rs#L54)
- `impl Copy for ExpectedState` — [源码:54](../../../crates/application/src/contract.rs#L54)
- `impl Eq for ExpectedState` — [源码:54](../../../crates/application/src/contract.rs#L54)
- `impl StructuralPartialEq for ExpectedState` — [源码:54](../../../crates/application/src/contract.rs#L54)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> ExpectedState` | impl Clone for ExpectedState | [源码:54](../../../crates/application/src/contract.rs#L54) | skin_editor_application::ExpectedState |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for ExpectedState | [源码:54](../../../crates/application/src/contract.rs#L54) |  |
| `fn eq(&self, other: &ExpectedState) -> bool` | impl PartialEq for ExpectedState | [源码:54](../../../crates/application/src/contract.rs#L54) | skin_editor_application::ExpectedState |

## skin_editor_application::FrameDescriptor

[源码:37](../../../crates/application/src/preview_port.rs#L37) · struct

```rust
pub struct FrameDescriptor {
    pub api_version: u16,
    pub key: FrameKey,
    pub source: FrameSource,
    pub status: FrameStatus,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::preview_port::FrameDescriptor`

声明引用：`skin_editor_application::FrameKey`, `skin_editor_application::FrameSource`, `skin_editor_application::FrameStatus`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::FrameDescriptor::clone`, `skin_editor_application::FrameDescriptor::eq`, `skin_editor_application::PreviewPort (声明)`, `skin_editor_application::PreviewPort::request`, `skin_editor_application::ProjectSession::request_preview`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for FrameDescriptor` — [源码:36](../../../crates/application/src/preview_port.rs#L36)
- `impl Debug for FrameDescriptor` — [源码:36](../../../crates/application/src/preview_port.rs#L36)
- `impl PartialEq for FrameDescriptor` — [源码:36](../../../crates/application/src/preview_port.rs#L36)
- `impl Copy for FrameDescriptor` — [源码:36](../../../crates/application/src/preview_port.rs#L36)
- `impl Eq for FrameDescriptor` — [源码:36](../../../crates/application/src/preview_port.rs#L36)
- `impl StructuralPartialEq for FrameDescriptor` — [源码:36](../../../crates/application/src/preview_port.rs#L36)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> FrameDescriptor` | impl Clone for FrameDescriptor | [源码:36](../../../crates/application/src/preview_port.rs#L36) | skin_editor_application::FrameDescriptor |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for FrameDescriptor | [源码:36](../../../crates/application/src/preview_port.rs#L36) |  |
| `fn eq(&self, other: &FrameDescriptor) -> bool` | impl PartialEq for FrameDescriptor | [源码:36](../../../crates/application/src/preview_port.rs#L36) | skin_editor_application::FrameDescriptor |

## skin_editor_application::FrameKey

[源码:30](../../../crates/application/src/preview_port.rs#L30) · struct

```rust
pub struct FrameKey(pub u64);
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::preview_port::FrameKey`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::FrameDescriptor (声明)`, `skin_editor_application::FrameKey::clone`, `skin_editor_application::FrameKey::cmp`, `skin_editor_application::FrameKey::eq`, `skin_editor_application::FrameKey::partial_cmp`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for FrameKey` — [源码:29](../../../crates/application/src/preview_port.rs#L29)
- `impl Debug for FrameKey` — [源码:29](../../../crates/application/src/preview_port.rs#L29)
- `impl Ord for FrameKey` — [源码:29](../../../crates/application/src/preview_port.rs#L29)
- `impl PartialEq for FrameKey` — [源码:29](../../../crates/application/src/preview_port.rs#L29)
- `impl PartialOrd for FrameKey` — [源码:29](../../../crates/application/src/preview_port.rs#L29)
- `impl Copy for FrameKey` — [源码:29](../../../crates/application/src/preview_port.rs#L29)
- `impl Eq for FrameKey` — [源码:29](../../../crates/application/src/preview_port.rs#L29)
- `impl StructuralPartialEq for FrameKey` — [源码:29](../../../crates/application/src/preview_port.rs#L29)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> FrameKey` | impl Clone for FrameKey | [源码:29](../../../crates/application/src/preview_port.rs#L29) | skin_editor_application::FrameKey |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for FrameKey | [源码:29](../../../crates/application/src/preview_port.rs#L29) |  |
| `fn cmp(&self, other: &FrameKey) -> Ordering` | impl Ord for FrameKey | [源码:29](../../../crates/application/src/preview_port.rs#L29) | skin_editor_application::FrameKey |
| `fn eq(&self, other: &FrameKey) -> bool` | impl PartialEq for FrameKey | [源码:29](../../../crates/application/src/preview_port.rs#L29) | skin_editor_application::FrameKey |
| `fn partial_cmp(&self, other: &FrameKey) -> Option<Ordering>` | impl PartialOrd for FrameKey | [源码:29](../../../crates/application/src/preview_port.rs#L29) | skin_editor_application::FrameKey |

## skin_editor_application::FrameSource

[源码:24](../../../crates/application/src/preview_port.rs#L24) · struct

```rust
pub struct FrameSource {
    pub session: HistorySessionId,
    pub state: ExpectedState,
    pub context: PreviewContext,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::preview_port::FrameSource`

声明引用：`skin_editor_application::ExpectedState`, `skin_editor_application::HistorySessionId`, `skin_editor_application::PreviewContext`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::FrameDescriptor (声明)`, `skin_editor_application::FrameSource::clone`, `skin_editor_application::FrameSource::eq`, `skin_editor_application::PreviewPort (声明)`, `skin_editor_application::PreviewPort::request`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for FrameSource` — [源码:23](../../../crates/application/src/preview_port.rs#L23)
- `impl Debug for FrameSource` — [源码:23](../../../crates/application/src/preview_port.rs#L23)
- `impl PartialEq for FrameSource` — [源码:23](../../../crates/application/src/preview_port.rs#L23)
- `impl Copy for FrameSource` — [源码:23](../../../crates/application/src/preview_port.rs#L23)
- `impl Eq for FrameSource` — [源码:23](../../../crates/application/src/preview_port.rs#L23)
- `impl StructuralPartialEq for FrameSource` — [源码:23](../../../crates/application/src/preview_port.rs#L23)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> FrameSource` | impl Clone for FrameSource | [源码:23](../../../crates/application/src/preview_port.rs#L23) | skin_editor_application::FrameSource |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for FrameSource | [源码:23](../../../crates/application/src/preview_port.rs#L23) |  |
| `fn eq(&self, other: &FrameSource) -> bool` | impl PartialEq for FrameSource | [源码:23](../../../crates/application/src/preview_port.rs#L23) | skin_editor_application::FrameSource |

## skin_editor_application::FrameStatus

[源码:32](../../../crates/application/src/preview_port.rs#L32) · enum

```rust
pub enum FrameStatus {
    Pending,
    Ready,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::preview_port::FrameStatus`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::FrameDescriptor (声明)`, `skin_editor_application::FrameStatus::clone`, `skin_editor_application::FrameStatus::eq`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for FrameStatus` — [源码:31](../../../crates/application/src/preview_port.rs#L31)
- `impl Debug for FrameStatus` — [源码:31](../../../crates/application/src/preview_port.rs#L31)
- `impl PartialEq for FrameStatus` — [源码:31](../../../crates/application/src/preview_port.rs#L31)
- `impl Copy for FrameStatus` — [源码:31](../../../crates/application/src/preview_port.rs#L31)
- `impl Eq for FrameStatus` — [源码:31](../../../crates/application/src/preview_port.rs#L31)
- `impl StructuralPartialEq for FrameStatus` — [源码:31](../../../crates/application/src/preview_port.rs#L31)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> FrameStatus` | impl Clone for FrameStatus | [源码:31](../../../crates/application/src/preview_port.rs#L31) | skin_editor_application::FrameStatus |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for FrameStatus | [源码:31](../../../crates/application/src/preview_port.rs#L31) |  |
| `fn eq(&self, other: &FrameStatus) -> bool` | impl PartialEq for FrameStatus | [源码:31](../../../crates/application/src/preview_port.rs#L31) | skin_editor_application::FrameStatus |

## skin_editor_application::HistoricalSnapshot

[源码:85](../../../crates/application/src/lib.rs#L85) · struct

```rust
pub struct HistoricalSnapshot {
    pub api_version: u16,
    pub session_id: HistorySessionId,
    pub node: NodeId,
    pub project: SkinProject,
}
```

声明引用：`skin_editor_application::HistorySessionId`, `skin_editor_application::NodeId`, `skin_editor_domain::SkinProject`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::HistoricalSnapshot::clone`, `skin_editor_application::HistoricalSnapshot::eq`, `skin_editor_application::ProjectSession::history_snapshot`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for HistoricalSnapshot` — [源码:84](../../../crates/application/src/lib.rs#L84)
- `impl Debug for HistoricalSnapshot` — [源码:84](../../../crates/application/src/lib.rs#L84)
- `impl PartialEq for HistoricalSnapshot` — [源码:84](../../../crates/application/src/lib.rs#L84)
- `impl StructuralPartialEq for HistoricalSnapshot` — [源码:84](../../../crates/application/src/lib.rs#L84)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> HistoricalSnapshot` | impl Clone for HistoricalSnapshot | [源码:84](../../../crates/application/src/lib.rs#L84) | skin_editor_application::HistoricalSnapshot |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for HistoricalSnapshot | [源码:84](../../../crates/application/src/lib.rs#L84) |  |
| `fn eq(&self, other: &HistoricalSnapshot) -> bool` | impl PartialEq for HistoricalSnapshot | [源码:84](../../../crates/application/src/lib.rs#L84) | skin_editor_application::HistoricalSnapshot |

## skin_editor_application::HistoryAction

[源码:319](../../../crates/application/src/contract.rs#L319) · enum

```rust
pub enum HistoryAction {
    Undo,
    Redo {
        child: Option<NodeId>,
    },
    Checkout {
        node: NodeId,
    },
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::contract::HistoryAction`

声明引用：`skin_editor_application::NodeId`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::HistoryAction::clone`, `skin_editor_application::HistoryAction::eq`, `skin_editor_application::HistoryRequest (声明)`, `skin_editor_application::ProjectSession::navigation_target`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for HistoryAction` — [源码:318](../../../crates/application/src/contract.rs#L318)
- `impl Debug for HistoryAction` — [源码:318](../../../crates/application/src/contract.rs#L318)
- `impl PartialEq for HistoryAction` — [源码:318](../../../crates/application/src/contract.rs#L318)
- `impl Eq for HistoryAction` — [源码:318](../../../crates/application/src/contract.rs#L318)
- `impl StructuralPartialEq for HistoryAction` — [源码:318](../../../crates/application/src/contract.rs#L318)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> HistoryAction` | impl Clone for HistoryAction | [源码:318](../../../crates/application/src/contract.rs#L318) | skin_editor_application::HistoryAction |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for HistoryAction | [源码:318](../../../crates/application/src/contract.rs#L318) |  |
| `fn eq(&self, other: &HistoryAction) -> bool` | impl PartialEq for HistoryAction | [源码:318](../../../crates/application/src/contract.rs#L318) | skin_editor_application::HistoryAction |

## skin_editor_application::HistoryAncestorsPage

[源码:406](../../../crates/application/src/contract.rs#L406) · struct

```rust
pub struct HistoryAncestorsPage {
    pub api_version: u16,
    pub session_id: HistorySessionId,
    pub entries: Vec<HistorySummary>,
    pub next_ancestor: Option<NodeId>,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::contract::HistoryAncestorsPage`

声明引用：`skin_editor_application::HistorySessionId`, `skin_editor_application::HistorySummary`, `skin_editor_application::NodeId`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::HistoryAncestorsPage::clone`, `skin_editor_application::HistoryAncestorsPage::eq`, `skin_editor_application::ProjectSession::history_ancestors`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for HistoryAncestorsPage` — [源码:405](../../../crates/application/src/contract.rs#L405)
- `impl Debug for HistoryAncestorsPage` — [源码:405](../../../crates/application/src/contract.rs#L405)
- `impl PartialEq for HistoryAncestorsPage` — [源码:405](../../../crates/application/src/contract.rs#L405)
- `impl Eq for HistoryAncestorsPage` — [源码:405](../../../crates/application/src/contract.rs#L405)
- `impl StructuralPartialEq for HistoryAncestorsPage` — [源码:405](../../../crates/application/src/contract.rs#L405)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> HistoryAncestorsPage` | impl Clone for HistoryAncestorsPage | [源码:405](../../../crates/application/src/contract.rs#L405) | skin_editor_application::HistoryAncestorsPage |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for HistoryAncestorsPage | [源码:405](../../../crates/application/src/contract.rs#L405) |  |
| `fn eq(&self, other: &HistoryAncestorsPage) -> bool` | impl PartialEq for HistoryAncestorsPage | [源码:405](../../../crates/application/src/contract.rs#L405) | skin_editor_application::HistoryAncestorsPage |

## skin_editor_application::HistoryChildrenPage

[源码:397](../../../crates/application/src/contract.rs#L397) · struct

```rust
pub struct HistoryChildrenPage {
    pub api_version: u16,
    pub session_id: HistorySessionId,
    pub parent: NodeId,
    pub entries: Vec<HistorySummary>,
    pub next_offset: Option<usize>,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::contract::HistoryChildrenPage`

声明引用：`skin_editor_application::HistorySessionId`, `skin_editor_application::HistorySummary`, `skin_editor_application::NodeId`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::HistoryChildrenPage::clone`, `skin_editor_application::HistoryChildrenPage::eq`, `skin_editor_application::ProjectSession::history_children`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for HistoryChildrenPage` — [源码:396](../../../crates/application/src/contract.rs#L396)
- `impl Debug for HistoryChildrenPage` — [源码:396](../../../crates/application/src/contract.rs#L396)
- `impl PartialEq for HistoryChildrenPage` — [源码:396](../../../crates/application/src/contract.rs#L396)
- `impl Eq for HistoryChildrenPage` — [源码:396](../../../crates/application/src/contract.rs#L396)
- `impl StructuralPartialEq for HistoryChildrenPage` — [源码:396](../../../crates/application/src/contract.rs#L396)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> HistoryChildrenPage` | impl Clone for HistoryChildrenPage | [源码:396](../../../crates/application/src/contract.rs#L396) | skin_editor_application::HistoryChildrenPage |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for HistoryChildrenPage | [源码:396](../../../crates/application/src/contract.rs#L396) |  |
| `fn eq(&self, other: &HistoryChildrenPage) -> bool` | impl PartialEq for HistoryChildrenPage | [源码:396](../../../crates/application/src/contract.rs#L396) | skin_editor_application::HistoryChildrenPage |

## skin_editor_application::HistoryEntry

[源码:277](../../../crates/application/src/contract.rs#L277) · struct

```rust
pub struct HistoryEntry {
    pub api_version: u16,
    pub session_id: HistorySessionId,
    pub id: NodeId,
    pub parent: Option<NodeId>,
    pub request_id: Option<RequestId>,
    pub changed_objects: Vec<ObjectId>,
    pub configuration_changed: bool,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::contract::HistoryEntry`

声明引用：`skin_editor_application::HistorySessionId`, `skin_editor_application::NodeId`, `skin_editor_application::RequestId`, `skin_editor_domain::ObjectId`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::HistoryEntry::clone`, `skin_editor_application::HistoryEntry::eq`, `skin_editor_application::ProjectSession::history_entry`, `skin_editor_application::history::History::entry`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for HistoryEntry` — [源码:276](../../../crates/application/src/contract.rs#L276)
- `impl Debug for HistoryEntry` — [源码:276](../../../crates/application/src/contract.rs#L276)
- `impl PartialEq for HistoryEntry` — [源码:276](../../../crates/application/src/contract.rs#L276)
- `impl Eq for HistoryEntry` — [源码:276](../../../crates/application/src/contract.rs#L276)
- `impl StructuralPartialEq for HistoryEntry` — [源码:276](../../../crates/application/src/contract.rs#L276)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> HistoryEntry` | impl Clone for HistoryEntry | [源码:276](../../../crates/application/src/contract.rs#L276) | skin_editor_application::HistoryEntry |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for HistoryEntry | [源码:276](../../../crates/application/src/contract.rs#L276) |  |
| `fn eq(&self, other: &HistoryEntry) -> bool` | impl PartialEq for HistoryEntry | [源码:276](../../../crates/application/src/contract.rs#L276) | skin_editor_application::HistoryEntry |

## skin_editor_application::HistoryError

[源码:359](../../../crates/application/src/contract.rs#L359) · enum

```rust
pub enum HistoryError {
    UnsupportedVersion(u16),
    WrongSession,
    RequestIdReused,
    Conflict {
        expected: ExpectedState,
        actual: ExpectedState,
    },
    Query(HistoryQueryError),
    AmbiguousRedo {
        child_count: usize,
    },
    NotAChild {
        parent: NodeId,
        child: NodeId,
    },
    MissingPinnedResource(BlobRef),
    ResourceLimit(&'static str),
    Busy(DraftId),
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::contract::HistoryError`

声明引用：`skin_editor_application::DraftId`, `skin_editor_application::ExpectedState`, `skin_editor_application::HistoryQueryError`, `skin_editor_application::NodeId`, `skin_editor_domain::BlobRef`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::HistoryError::clone`, `skin_editor_application::HistoryError::eq`, `skin_editor_application::ProjectSession::navigate`, `skin_editor_application::ProjectSession::navigate_once`, `skin_editor_application::ProjectSession::navigation_target`, `skin_editor_application::Receipt (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for HistoryError` — [源码:358](../../../crates/application/src/contract.rs#L358)
- `impl Debug for HistoryError` — [源码:358](../../../crates/application/src/contract.rs#L358)
- `impl From<HistoryQueryError> for HistoryError` — [源码:379](../../../crates/application/src/contract.rs#L379)
- `impl PartialEq for HistoryError` — [源码:358](../../../crates/application/src/contract.rs#L358)
- `impl Eq for HistoryError` — [源码:358](../../../crates/application/src/contract.rs#L358)
- `impl StructuralPartialEq for HistoryError` — [源码:358](../../../crates/application/src/contract.rs#L358)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> HistoryError` | impl Clone for HistoryError | [源码:358](../../../crates/application/src/contract.rs#L358) | skin_editor_application::HistoryError |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for HistoryError | [源码:358](../../../crates/application/src/contract.rs#L358) |  |
| `fn from(error: HistoryQueryError) -> Self` | impl From<HistoryQueryError> for HistoryError | [源码:380](../../../crates/application/src/contract.rs#L380) | skin_editor_application::HistoryQueryError |
| `fn eq(&self, other: &HistoryError) -> bool` | impl PartialEq for HistoryError | [源码:358](../../../crates/application/src/contract.rs#L358) | skin_editor_application::HistoryError |

## skin_editor_application::HistoryQueryError

[源码:266](../../../crates/application/src/contract.rs#L266) · enum

```rust
pub enum HistoryQueryError {
    WrongSession,
    UnknownNode(NodeId),
    InvalidHistory(ModelError),
    InvalidAddress(AddressError),
    UnknownCoordinate(BranchCoordinate),
    InvalidPageLimit(usize),
    InvalidOffset {
        offset: usize,
        len: usize,
    },
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::contract::HistoryQueryError`

声明引用：`skin_editor_application::AddressError`, `skin_editor_application::BranchCoordinate`, `skin_editor_application::NodeId`, `skin_editor_domain::ModelError`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::CommandError (声明)`, `skin_editor_application::HistoryError (声明)`, `skin_editor_application::HistoryError::from`, `skin_editor_application::HistoryQueryError::clone`, `skin_editor_application::HistoryQueryError::eq`, `skin_editor_application::HistoryQueryError::from`, `skin_editor_application::ProjectSession::history_ancestors`, `skin_editor_application::ProjectSession::history_children`, `skin_editor_application::ProjectSession::history_entry`, `skin_editor_application::ProjectSession::history_scope`, `skin_editor_application::ProjectSession::history_snapshot`, `skin_editor_application::ProjectSession::history_statistics`, `skin_editor_application::ProjectSession::history_summary`, `skin_editor_application::ProjectSession::resolve_history_address`, `skin_editor_application::history::History::materialize`, `skin_editor_application::history::History::node`, `skin_editor_application::history::History::parent`, `skin_editor_application::history::History::summary`, `skin_editor_application::history::History::transition`, `skin_editor_application::history_queries::page_limit (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for HistoryQueryError` — [源码:265](../../../crates/application/src/contract.rs#L265)
- `impl Debug for HistoryQueryError` — [源码:265](../../../crates/application/src/contract.rs#L265)
- `impl From<HistoryQueryError> for HistoryError` — [源码:379](../../../crates/application/src/contract.rs#L379)
- `impl PartialEq for HistoryQueryError` — [源码:265](../../../crates/application/src/contract.rs#L265)
- `impl Eq for HistoryQueryError` — [源码:265](../../../crates/application/src/contract.rs#L265)
- `impl StructuralPartialEq for HistoryQueryError` — [源码:265](../../../crates/application/src/contract.rs#L265)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> HistoryQueryError` | impl Clone for HistoryQueryError | [源码:265](../../../crates/application/src/contract.rs#L265) | skin_editor_application::HistoryQueryError |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for HistoryQueryError | [源码:265](../../../crates/application/src/contract.rs#L265) |  |
| `fn from(error: HistoryQueryError) -> Self` | impl From<HistoryQueryError> for HistoryError | [源码:380](../../../crates/application/src/contract.rs#L380) | skin_editor_application::HistoryQueryError |
| `fn eq(&self, other: &HistoryQueryError) -> bool` | impl PartialEq for HistoryQueryError | [源码:265](../../../crates/application/src/contract.rs#L265) | skin_editor_application::HistoryQueryError |

## skin_editor_application::HistoryRequest

[源码:332](../../../crates/application/src/contract.rs#L332) · struct

```rust
pub struct HistoryRequest {
    pub api_version: u16,
    pub session_id: HistorySessionId,
    pub request_id: RequestId,
    pub expected: ExpectedState,
    pub action: HistoryAction,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::contract::HistoryRequest`

声明引用：`skin_editor_application::ExpectedState`, `skin_editor_application::HistoryAction`, `skin_editor_application::HistorySessionId`, `skin_editor_application::RequestId`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::HistoryRequest::clone`, `skin_editor_application::HistoryRequest::eq`, `skin_editor_application::ProjectSession::navigate`, `skin_editor_application::ProjectSession::navigate_once`, `skin_editor_application::Receipt (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for HistoryRequest` — [源码:331](../../../crates/application/src/contract.rs#L331)
- `impl Debug for HistoryRequest` — [源码:331](../../../crates/application/src/contract.rs#L331)
- `impl PartialEq for HistoryRequest` — [源码:331](../../../crates/application/src/contract.rs#L331)
- `impl Eq for HistoryRequest` — [源码:331](../../../crates/application/src/contract.rs#L331)
- `impl StructuralPartialEq for HistoryRequest` — [源码:331](../../../crates/application/src/contract.rs#L331)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> HistoryRequest` | impl Clone for HistoryRequest | [源码:331](../../../crates/application/src/contract.rs#L331) | skin_editor_application::HistoryRequest |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for HistoryRequest | [源码:331](../../../crates/application/src/contract.rs#L331) |  |
| `fn eq(&self, other: &HistoryRequest) -> bool` | impl PartialEq for HistoryRequest | [源码:331](../../../crates/application/src/contract.rs#L331) | skin_editor_application::HistoryRequest |

## skin_editor_application::HistoryResult

[源码:347](../../../crates/application/src/contract.rs#L347) · struct

```rust
pub struct HistoryResult {
    pub api_version: u16,
    pub session_id: HistorySessionId,
    pub request_id: RequestId,
    pub previous: ExpectedState,
    pub state: ExpectedState,
    pub status: HistoryStatus,
    pub changed_objects: Vec<ObjectId>,
    pub configuration_changed: bool,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::contract::HistoryResult`

声明引用：`skin_editor_application::ExpectedState`, `skin_editor_application::HistorySessionId`, `skin_editor_application::HistoryStatus`, `skin_editor_application::RequestId`, `skin_editor_domain::ObjectId`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::HistoryResult::clone`, `skin_editor_application::HistoryResult::eq`, `skin_editor_application::ProjectSession::navigate`, `skin_editor_application::ProjectSession::navigate_once`, `skin_editor_application::Receipt (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for HistoryResult` — [源码:346](../../../crates/application/src/contract.rs#L346)
- `impl Debug for HistoryResult` — [源码:346](../../../crates/application/src/contract.rs#L346)
- `impl PartialEq for HistoryResult` — [源码:346](../../../crates/application/src/contract.rs#L346)
- `impl Eq for HistoryResult` — [源码:346](../../../crates/application/src/contract.rs#L346)
- `impl StructuralPartialEq for HistoryResult` — [源码:346](../../../crates/application/src/contract.rs#L346)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> HistoryResult` | impl Clone for HistoryResult | [源码:346](../../../crates/application/src/contract.rs#L346) | skin_editor_application::HistoryResult |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for HistoryResult | [源码:346](../../../crates/application/src/contract.rs#L346) |  |
| `fn eq(&self, other: &HistoryResult) -> bool` | impl PartialEq for HistoryResult | [源码:346](../../../crates/application/src/contract.rs#L346) | skin_editor_application::HistoryResult |

## skin_editor_application::HistorySessionId

[源码:12](../../../crates/application/src/contract.rs#L12) · struct

```rust
pub struct HistorySessionId(NonZeroU128);
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::contract::HistorySessionId`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::CommandRequest (声明)`, `skin_editor_application::CommandResult (声明)`, `skin_editor_application::DraftInfo (声明)`, `skin_editor_application::DraftRequest (声明)`, `skin_editor_application::DraftResult (声明)`, `skin_editor_application::FrameSource (声明)`, `skin_editor_application::HistoricalSnapshot (声明)`, `skin_editor_application::HistoryAncestorsPage (声明)`, `skin_editor_application::HistoryChildrenPage (声明)`, `skin_editor_application::HistoryEntry (声明)`, `skin_editor_application::HistoryRequest (声明)`, `skin_editor_application::HistoryResult (声明)`, `skin_editor_application::HistorySessionId::clone`, `skin_editor_application::HistorySessionId::cmp`, `skin_editor_application::HistorySessionId::eq`, `skin_editor_application::HistorySessionId::partial_cmp`, `skin_editor_application::HistoryStatistics (声明)`, `skin_editor_application::ProjectInfo (声明)`, `skin_editor_application::ProjectSession (声明)`, `skin_editor_application::ProjectSession::draft_info`, `skin_editor_application::ProjectSession::draft_snapshot`, `skin_editor_application::ProjectSession::history_ancestors`, `skin_editor_application::ProjectSession::history_children`, `skin_editor_application::ProjectSession::history_entry`, `skin_editor_application::ProjectSession::history_scope`, `skin_editor_application::ProjectSession::history_snapshot`, `skin_editor_application::ProjectSession::history_statistics`, `skin_editor_application::ProjectSession::history_summary`, `skin_editor_application::ProjectSession::resolve_history_address`, `skin_editor_application::ProjectSnapshot (声明)`, `skin_editor_application::SkinAudioSource (声明)`, `skin_editor_application::history::History::entry`

实现／约束（显式或 derive，无类继承）：

- `impl HistorySessionId` — [源码:13](../../../crates/application/src/contract.rs#L13)
- `impl Clone for HistorySessionId` — [源码:11](../../../crates/application/src/contract.rs#L11)
- `impl Debug for HistorySessionId` — [源码:11](../../../crates/application/src/contract.rs#L11)
- `impl Ord for HistorySessionId` — [源码:11](../../../crates/application/src/contract.rs#L11)
- `impl PartialEq for HistorySessionId` — [源码:11](../../../crates/application/src/contract.rs#L11)
- `impl PartialOrd for HistorySessionId` — [源码:11](../../../crates/application/src/contract.rs#L11)
- `impl Copy for HistorySessionId` — [源码:11](../../../crates/application/src/contract.rs#L11)
- `impl Eq for HistorySessionId` — [源码:11](../../../crates/application/src/contract.rs#L11)
- `impl StructuralPartialEq for HistorySessionId` — [源码:11](../../../crates/application/src/contract.rs#L11)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub const fn new(value: u128) -> Option<Self>` | impl HistorySessionId | [源码:14](../../../crates/application/src/contract.rs#L14) |  |
| `pub const fn get(self) -> u128` | impl HistorySessionId | [源码:20](../../../crates/application/src/contract.rs#L20) |  |
| `fn clone(&self) -> HistorySessionId` | impl Clone for HistorySessionId | [源码:11](../../../crates/application/src/contract.rs#L11) | skin_editor_application::HistorySessionId |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for HistorySessionId | [源码:11](../../../crates/application/src/contract.rs#L11) |  |
| `fn cmp(&self, other: &HistorySessionId) -> Ordering` | impl Ord for HistorySessionId | [源码:11](../../../crates/application/src/contract.rs#L11) | skin_editor_application::HistorySessionId |
| `fn eq(&self, other: &HistorySessionId) -> bool` | impl PartialEq for HistorySessionId | [源码:11](../../../crates/application/src/contract.rs#L11) | skin_editor_application::HistorySessionId |
| `fn partial_cmp(&self, other: &HistorySessionId) -> Option<Ordering>` | impl PartialOrd for HistorySessionId | [源码:11](../../../crates/application/src/contract.rs#L11) | skin_editor_application::HistorySessionId |

## skin_editor_application::HistoryStatistics

[源码:416](../../../crates/application/src/contract.rs#L416) · struct

```rust
pub struct HistoryStatistics {
    pub api_version: u16,
    pub session_id: HistorySessionId,
    pub nodes: usize,
    pub edges: usize,
    pub coordinate_entries: usize,
    pub branches: usize,
    pub max_depth: u64,
    pub retained_blobs: usize,
    pub retained_requests: usize,
    pub redo_preference_entries: usize,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::contract::HistoryStatistics`

声明引用：`skin_editor_application::HistorySessionId`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::HistoryStatistics::clone`, `skin_editor_application::HistoryStatistics::eq`, `skin_editor_application::ProjectSession::history_statistics`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for HistoryStatistics` — [源码:415](../../../crates/application/src/contract.rs#L415)
- `impl Debug for HistoryStatistics` — [源码:415](../../../crates/application/src/contract.rs#L415)
- `impl PartialEq for HistoryStatistics` — [源码:415](../../../crates/application/src/contract.rs#L415)
- `impl Eq for HistoryStatistics` — [源码:415](../../../crates/application/src/contract.rs#L415)
- `impl StructuralPartialEq for HistoryStatistics` — [源码:415](../../../crates/application/src/contract.rs#L415)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> HistoryStatistics` | impl Clone for HistoryStatistics | [源码:415](../../../crates/application/src/contract.rs#L415) | skin_editor_application::HistoryStatistics |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for HistoryStatistics | [源码:415](../../../crates/application/src/contract.rs#L415) |  |
| `fn eq(&self, other: &HistoryStatistics) -> bool` | impl PartialEq for HistoryStatistics | [源码:415](../../../crates/application/src/contract.rs#L415) | skin_editor_application::HistoryStatistics |

## skin_editor_application::HistoryStatus

[源码:341](../../../crates/application/src/contract.rs#L341) · enum

```rust
pub enum HistoryStatus {
    Changed,
    NoChange,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::contract::HistoryStatus`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::HistoryResult (声明)`, `skin_editor_application::HistoryStatus::clone`, `skin_editor_application::HistoryStatus::eq`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for HistoryStatus` — [源码:340](../../../crates/application/src/contract.rs#L340)
- `impl Debug for HistoryStatus` — [源码:340](../../../crates/application/src/contract.rs#L340)
- `impl PartialEq for HistoryStatus` — [源码:340](../../../crates/application/src/contract.rs#L340)
- `impl Copy for HistoryStatus` — [源码:340](../../../crates/application/src/contract.rs#L340)
- `impl Eq for HistoryStatus` — [源码:340](../../../crates/application/src/contract.rs#L340)
- `impl StructuralPartialEq for HistoryStatus` — [源码:340](../../../crates/application/src/contract.rs#L340)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> HistoryStatus` | impl Clone for HistoryStatus | [源码:340](../../../crates/application/src/contract.rs#L340) | skin_editor_application::HistoryStatus |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for HistoryStatus | [源码:340](../../../crates/application/src/contract.rs#L340) |  |
| `fn eq(&self, other: &HistoryStatus) -> bool` | impl PartialEq for HistoryStatus | [源码:340](../../../crates/application/src/contract.rs#L340) | skin_editor_application::HistoryStatus |

## skin_editor_application::HistorySummary

[源码:388](../../../crates/application/src/contract.rs#L388) · struct

```rust
pub struct HistorySummary {
    pub id: NodeId,
    pub parent: Option<NodeId>,
    pub depth: u64,
    pub coordinate: BranchCoordinate,
    pub child_count: usize,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::contract::HistorySummary`

声明引用：`skin_editor_application::BranchCoordinate`, `skin_editor_application::NodeId`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::HistoryAncestorsPage (声明)`, `skin_editor_application::HistoryChildrenPage (声明)`, `skin_editor_application::HistorySummary::clone`, `skin_editor_application::HistorySummary::eq`, `skin_editor_application::ProjectSession::history_summary`, `skin_editor_application::history::History::summary`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for HistorySummary` — [源码:387](../../../crates/application/src/contract.rs#L387)
- `impl Debug for HistorySummary` — [源码:387](../../../crates/application/src/contract.rs#L387)
- `impl PartialEq for HistorySummary` — [源码:387](../../../crates/application/src/contract.rs#L387)
- `impl Eq for HistorySummary` — [源码:387](../../../crates/application/src/contract.rs#L387)
- `impl StructuralPartialEq for HistorySummary` — [源码:387](../../../crates/application/src/contract.rs#L387)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> HistorySummary` | impl Clone for HistorySummary | [源码:387](../../../crates/application/src/contract.rs#L387) | skin_editor_application::HistorySummary |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for HistorySummary | [源码:387](../../../crates/application/src/contract.rs#L387) |  |
| `fn eq(&self, other: &HistorySummary) -> bool` | impl PartialEq for HistorySummary | [源码:387](../../../crates/application/src/contract.rs#L387) | skin_editor_application::HistorySummary |

## skin_editor_application::ImportError

[源码:92](../../../crates/application/src/preview_port.rs#L92) · enum

```rust
pub enum ImportError {
    Unsupported,
    ResourceLimit,
    InvalidImage,
    Io(String),
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::preview_port::ImportError`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::ImportError::clone`, `skin_editor_application::ImportError::eq`, `skin_editor_application::RasterImportPort (声明)`, `skin_editor_application::RasterImportPort::stage_png`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for ImportError` — [源码:91](../../../crates/application/src/preview_port.rs#L91)
- `impl Debug for ImportError` — [源码:91](../../../crates/application/src/preview_port.rs#L91)
- `impl PartialEq for ImportError` — [源码:91](../../../crates/application/src/preview_port.rs#L91)
- `impl Eq for ImportError` — [源码:91](../../../crates/application/src/preview_port.rs#L91)
- `impl StructuralPartialEq for ImportError` — [源码:91](../../../crates/application/src/preview_port.rs#L91)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> ImportError` | impl Clone for ImportError | [源码:91](../../../crates/application/src/preview_port.rs#L91) | skin_editor_application::ImportError |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for ImportError | [源码:91](../../../crates/application/src/preview_port.rs#L91) |  |
| `fn eq(&self, other: &ImportError) -> bool` | impl PartialEq for ImportError | [源码:91](../../../crates/application/src/preview_port.rs#L91) | skin_editor_application::ImportError |

## skin_editor_application::MAX_AUDIO_DRAFT_REFERENCES

[源码:6](../../../crates/application/src/audio_port.rs#L6) · constant

```rust
pub const MAX_AUDIO_DRAFT_REFERENCES: usize = 1024;
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::audio_port::MAX_AUDIO_DRAFT_REFERENCES`

## skin_editor_application::MAX_BRANCH_ADDRESS

[源码:4](../../../crates/application/src/address.rs#L4) · constant

```rust
pub const MAX_BRANCH_ADDRESS: usize = 28;
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::address::MAX_BRANCH_ADDRESS`

## skin_editor_application::MAX_HISTORY_PAGE

[源码:385](../../../crates/application/src/contract.rs#L385) · constant

```rust
pub const MAX_HISTORY_PAGE: usize = 256;
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::contract::MAX_HISTORY_PAGE`

## skin_editor_application::MAX_NODE_ADDRESS

[源码:3](../../../crates/application/src/address.rs#L3) · constant

```rust
pub const MAX_NODE_ADDRESS: usize = 14;
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::address::MAX_NODE_ADDRESS`

## skin_editor_application::NodeId

[源码:41](../../../crates/application/src/contract.rs#L41) · struct

```rust
pub struct NodeId(NonZeroU64);
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::contract::NodeId`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::ExpectedState (声明)`, `skin_editor_application::HistoricalSnapshot (声明)`, `skin_editor_application::HistoryAction (声明)`, `skin_editor_application::HistoryAncestorsPage (声明)`, `skin_editor_application::HistoryChildrenPage (声明)`, `skin_editor_application::HistoryEntry (声明)`, `skin_editor_application::HistoryError (声明)`, `skin_editor_application::HistoryQueryError (声明)`, `skin_editor_application::HistorySummary (声明)`, `skin_editor_application::NodeId::clone`, `skin_editor_application::NodeId::cmp`, `skin_editor_application::NodeId::eq`, `skin_editor_application::NodeId::partial_cmp`, `skin_editor_application::ProjectSession (声明)`, `skin_editor_application::ProjectSession::history_ancestors`, `skin_editor_application::ProjectSession::history_children`, `skin_editor_application::ProjectSession::history_entry`, `skin_editor_application::ProjectSession::history_snapshot`, `skin_editor_application::ProjectSession::history_summary`, `skin_editor_application::ProjectSession::navigation_target`, `skin_editor_application::ProjectSession::resolve_history_address`, `skin_editor_application::history::AppendPlan (声明)`, `skin_editor_application::history::History (声明)`, `skin_editor_application::history::History::entry`, `skin_editor_application::history::History::materialize`, `skin_editor_application::history::History::node`, `skin_editor_application::history::History::parent`, `skin_editor_application::history::History::plan_append`, `skin_editor_application::history::History::summary`, `skin_editor_application::history::History::transition`, `skin_editor_application::history::Node (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl NodeId` — [源码:39](../../../crates/application/src/address.rs#L39)
- `impl NodeId` — [源码:42](../../../crates/application/src/contract.rs#L42)
- `impl Clone for NodeId` — [源码:40](../../../crates/application/src/contract.rs#L40)
- `impl Debug for NodeId` — [源码:40](../../../crates/application/src/contract.rs#L40)
- `impl Ord for NodeId` — [源码:40](../../../crates/application/src/contract.rs#L40)
- `impl PartialEq for NodeId` — [源码:40](../../../crates/application/src/contract.rs#L40)
- `impl PartialOrd for NodeId` — [源码:40](../../../crates/application/src/contract.rs#L40)
- `impl Copy for NodeId` — [源码:40](../../../crates/application/src/contract.rs#L40)
- `impl Eq for NodeId` — [源码:40](../../../crates/application/src/contract.rs#L40)
- `impl StructuralPartialEq for NodeId` — [源码:40](../../../crates/application/src/contract.rs#L40)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn short_address(self) -> String` | impl NodeId | [源码:40](../../../crates/application/src/address.rs#L40) |  |
| `pub fn parse_address(text: &str) -> Result<Self, AddressError>` | impl NodeId | [源码:46](../../../crates/application/src/address.rs#L46) | skin_editor_application::AddressError |
| `pub const fn new(value: u64) -> Option<Self>` | impl NodeId | [源码:43](../../../crates/application/src/contract.rs#L43) |  |
| `pub const fn get(self) -> u64` | impl NodeId | [源码:49](../../../crates/application/src/contract.rs#L49) |  |
| `fn clone(&self) -> NodeId` | impl Clone for NodeId | [源码:40](../../../crates/application/src/contract.rs#L40) | skin_editor_application::NodeId |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for NodeId | [源码:40](../../../crates/application/src/contract.rs#L40) |  |
| `fn cmp(&self, other: &NodeId) -> Ordering` | impl Ord for NodeId | [源码:40](../../../crates/application/src/contract.rs#L40) | skin_editor_application::NodeId |
| `fn eq(&self, other: &NodeId) -> bool` | impl PartialEq for NodeId | [源码:40](../../../crates/application/src/contract.rs#L40) | skin_editor_application::NodeId |
| `fn partial_cmp(&self, other: &NodeId) -> Option<Ordering>` | impl PartialOrd for NodeId | [源码:40](../../../crates/application/src/contract.rs#L40) | skin_editor_application::NodeId |

## skin_editor_application::PREVIEW_API_VERSION

[源码:6](../../../crates/application/src/preview_port.rs#L6) · constant

```rust
pub const PREVIEW_API_VERSION: u16 = 2;
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::preview_port::PREVIEW_API_VERSION`

## skin_editor_application::PreviewContext

[源码:15](../../../crates/application/src/preview_port.rs#L15) · struct

```rust
pub struct PreviewContext {
    pub target: PreviewTarget,
    pub draft: Option<DraftState>,
    pub size: [u32; 2],
    pub viewport_generation: u64,
    pub scene: u8,
    pub simulation_micros: u64,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::preview_port::PreviewContext`

声明引用：`skin_editor_application::DraftState`, `skin_editor_application::PreviewTarget`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::FrameSource (声明)`, `skin_editor_application::PreviewContext::clone`, `skin_editor_application::PreviewContext::eq`, `skin_editor_application::ProjectSession::request_preview`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for PreviewContext` — [源码:14](../../../crates/application/src/preview_port.rs#L14)
- `impl Debug for PreviewContext` — [源码:14](../../../crates/application/src/preview_port.rs#L14)
- `impl PartialEq for PreviewContext` — [源码:14](../../../crates/application/src/preview_port.rs#L14)
- `impl Copy for PreviewContext` — [源码:14](../../../crates/application/src/preview_port.rs#L14)
- `impl Eq for PreviewContext` — [源码:14](../../../crates/application/src/preview_port.rs#L14)
- `impl StructuralPartialEq for PreviewContext` — [源码:14](../../../crates/application/src/preview_port.rs#L14)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> PreviewContext` | impl Clone for PreviewContext | [源码:14](../../../crates/application/src/preview_port.rs#L14) | skin_editor_application::PreviewContext |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for PreviewContext | [源码:14](../../../crates/application/src/preview_port.rs#L14) |  |
| `fn eq(&self, other: &PreviewContext) -> bool` | impl PartialEq for PreviewContext | [源码:14](../../../crates/application/src/preview_port.rs#L14) | skin_editor_application::PreviewContext |

## skin_editor_application::PreviewError

[源码:44](../../../crates/application/src/preview_port.rs#L44) · enum

```rust
pub enum PreviewError {
    InvalidViewport,
    InvalidDraft,
    MissingDocument,
    ResourceLimit,
    Backend(String),
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::preview_port::PreviewError`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::PreviewError::clone`, `skin_editor_application::PreviewError::eq`, `skin_editor_application::PreviewPort (声明)`, `skin_editor_application::PreviewPort::request`, `skin_editor_application::ProjectSession::request_preview`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for PreviewError` — [源码:43](../../../crates/application/src/preview_port.rs#L43)
- `impl Debug for PreviewError` — [源码:43](../../../crates/application/src/preview_port.rs#L43)
- `impl PartialEq for PreviewError` — [源码:43](../../../crates/application/src/preview_port.rs#L43)
- `impl Eq for PreviewError` — [源码:43](../../../crates/application/src/preview_port.rs#L43)
- `impl StructuralPartialEq for PreviewError` — [源码:43](../../../crates/application/src/preview_port.rs#L43)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> PreviewError` | impl Clone for PreviewError | [源码:43](../../../crates/application/src/preview_port.rs#L43) | skin_editor_application::PreviewError |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for PreviewError | [源码:43](../../../crates/application/src/preview_port.rs#L43) |  |
| `fn eq(&self, other: &PreviewError) -> bool` | impl PartialEq for PreviewError | [源码:43](../../../crates/application/src/preview_port.rs#L43) | skin_editor_application::PreviewError |

## skin_editor_application::PreviewPort

[源码:51](../../../crates/application/src/preview_port.rs#L51) · trait

```rust
pub trait PreviewPort {
    // Required method
    fn request(
        &mut self,
        project: &SkinProject,
        source: FrameSource,
    ) -> Result<FrameDescriptor, PreviewError>;
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::preview_port::PreviewPort`

声明引用：`skin_editor_application::FrameDescriptor`, `skin_editor_application::FrameSource`, `skin_editor_application::PreviewError`, `skin_editor_domain::SkinProject`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::ProjectSession::request_preview`

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn request( &mut self, project: &SkinProject, source: FrameSource, ) -> Result<FrameDescriptor, PreviewError>` |  | [源码:52](../../../crates/application/src/preview_port.rs#L52) | skin_editor_application::FrameDescriptor, skin_editor_application::FrameSource, skin_editor_application::PreviewError, skin_editor_domain::SkinProject |

## skin_editor_application::PreviewTarget

[源码:8](../../../crates/application/src/preview_port.rs#L8) · enum

```rust
pub enum PreviewTarget {
    Document(DocumentId),
    Components,
    Workspace,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::preview_port::PreviewTarget`

声明引用：`skin_editor_domain::DocumentId`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::PreviewContext (声明)`, `skin_editor_application::PreviewTarget::clone`, `skin_editor_application::PreviewTarget::eq`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for PreviewTarget` — [源码:7](../../../crates/application/src/preview_port.rs#L7)
- `impl Debug for PreviewTarget` — [源码:7](../../../crates/application/src/preview_port.rs#L7)
- `impl PartialEq for PreviewTarget` — [源码:7](../../../crates/application/src/preview_port.rs#L7)
- `impl Copy for PreviewTarget` — [源码:7](../../../crates/application/src/preview_port.rs#L7)
- `impl Eq for PreviewTarget` — [源码:7](../../../crates/application/src/preview_port.rs#L7)
- `impl StructuralPartialEq for PreviewTarget` — [源码:7](../../../crates/application/src/preview_port.rs#L7)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> PreviewTarget` | impl Clone for PreviewTarget | [源码:7](../../../crates/application/src/preview_port.rs#L7) | skin_editor_application::PreviewTarget |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for PreviewTarget | [源码:7](../../../crates/application/src/preview_port.rs#L7) |  |
| `fn eq(&self, other: &PreviewTarget) -> bool` | impl PartialEq for PreviewTarget | [源码:7](../../../crates/application/src/preview_port.rs#L7) | skin_editor_application::PreviewTarget |

## skin_editor_application::ProjectInfo

[源码:66](../../../crates/application/src/lib.rs#L66) · struct

```rust
pub struct ProjectInfo {
    pub api_version: u16,
    pub project_id: u128,
    pub session_id: HistorySessionId,
    pub state: ExpectedState,
}
```

声明引用：`skin_editor_application::ExpectedState`, `skin_editor_application::HistorySessionId`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::ProjectInfo::clone`, `skin_editor_application::ProjectInfo::eq`, `skin_editor_application::ProjectSession::project_info`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for ProjectInfo` — [源码:65](../../../crates/application/src/lib.rs#L65)
- `impl Debug for ProjectInfo` — [源码:65](../../../crates/application/src/lib.rs#L65)
- `impl PartialEq for ProjectInfo` — [源码:65](../../../crates/application/src/lib.rs#L65)
- `impl Eq for ProjectInfo` — [源码:65](../../../crates/application/src/lib.rs#L65)
- `impl StructuralPartialEq for ProjectInfo` — [源码:65](../../../crates/application/src/lib.rs#L65)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> ProjectInfo` | impl Clone for ProjectInfo | [源码:65](../../../crates/application/src/lib.rs#L65) | skin_editor_application::ProjectInfo |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for ProjectInfo | [源码:65](../../../crates/application/src/lib.rs#L65) |  |
| `fn eq(&self, other: &ProjectInfo) -> bool` | impl PartialEq for ProjectInfo | [源码:65](../../../crates/application/src/lib.rs#L65) | skin_editor_application::ProjectInfo |

## skin_editor_application::ProjectSession

[源码:137](../../../crates/application/src/lib.rs#L137) · struct

```rust
pub struct ProjectSession {
    pub(crate) project: SkinProject,
    pub(crate) session_id: HistorySessionId,
    pub(crate) state: ExpectedState,
    pub(crate) navigation_revision: u64,
    pub(crate) history: History,
    pub(crate) redo_preferences: BTreeMap<NodeId, NodeId>,
    pub(crate) used_ids: BTreeSet<ObjectId>,
    pub(crate) receipts: BTreeMap<RequestId, Receipt>,
    pub(crate) pins: BTreeMap<BlobRef, Arc<dyn BlobLease>>,
    pub(crate) resources: Box<dyn ResourceStore>,
    pub(crate) limits: SessionLimits,
    pub(crate) draft: Option<Draft>,
    pub(crate) next_draft_id: u64,
    pub(crate) cancelled_drafts: BTreeSet<DraftId>,
}
```

声明引用：`skin_editor_application::BlobLease`, `skin_editor_application::DraftId`, `skin_editor_application::ExpectedState`, `skin_editor_application::HistorySessionId`, `skin_editor_application::NodeId`, `skin_editor_application::Receipt`, `skin_editor_application::RequestId`, `skin_editor_application::ResourceStore`, `skin_editor_application::SessionLimits`, `skin_editor_application::draft::Draft`, `skin_editor_application::history::History`, `skin_editor_domain::BlobRef`, `skin_editor_domain::ObjectId`, `skin_editor_domain::SkinProject`

实现／约束（显式或 derive，无类继承）：

- `impl ProjectSession` — [源码:110](../../../crates/application/src/audio_editor.rs#L110)
- `impl ProjectSession` — [源码:36](../../../crates/application/src/audio_port.rs#L36)
- `impl ProjectSession` — [源码:18](../../../crates/application/src/draft.rs#L18)
- `impl ProjectSession` — [源码:6](../../../crates/application/src/history_control.rs#L6)
- `impl ProjectSession` — [源码:14](../../../crates/application/src/history_queries.rs#L14)
- `impl ProjectSession` — [源码:58](../../../crates/application/src/preview_port.rs#L58)
- `impl ProjectSession` — [源码:158](../../../crates/application/src/lib.rs#L158)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn skin_audio_source(&self) -> SkinAudioSource` | impl ProjectSession | [源码:112](../../../crates/application/src/audio_editor.rs#L112) | skin_editor_application::SkinAudioSource |
| `pub fn control_audio( &self, port: &mut dyn AudioEditorPort, action: AudioEditorAction, ) -> Result<(), AudioEditorError>` | impl ProjectSession | [源码:120](../../../crates/application/src/audio_editor.rs#L120) | skin_editor_application::AudioEditorAction, skin_editor_application::AudioEditorError, skin_editor_application::AudioEditorPort |
| `pub fn synchronize_audio( &self, port: &mut dyn SkinAudioPort, ) -> Result<(), SkinAudioError>` | impl ProjectSession | [源码:41](../../../crates/application/src/audio_port.rs#L41) | skin_editor_application::SkinAudioError, skin_editor_application::SkinAudioPort |
| `pub fn draft_info( &self, session_id: HistorySessionId, ) -> Result<Option<DraftInfo>, DraftError>` | impl ProjectSession | [源码:19](../../../crates/application/src/draft.rs#L19) | skin_editor_application::DraftError, skin_editor_application::DraftInfo, skin_editor_application::HistorySessionId |
| `pub(crate) fn expected_draft( &self, expected: DraftState, ) -> Result<&Draft, DraftError>` | impl ProjectSession | [源码:34](../../../crates/application/src/draft.rs#L34) | skin_editor_application::DraftError, skin_editor_application::DraftState, skin_editor_application::draft::Draft |
| `pub fn draft_snapshot( &self, session_id: HistorySessionId, expected: DraftState, ) -> Result<DraftSnapshot, DraftError>` | impl ProjectSession | [源码:46](../../../crates/application/src/draft.rs#L46) | skin_editor_application::DraftError, skin_editor_application::DraftSnapshot, skin_editor_application::DraftState, skin_editor_application::HistorySessionId |
| `pub fn edit_draft( &mut self, request: DraftRequest, ) -> Result<DraftResult, DraftError>` | impl ProjectSession | [源码:63](../../../crates/application/src/draft.rs#L63) | skin_editor_application::DraftError, skin_editor_application::DraftRequest, skin_editor_application::DraftResult |
| `fn draft_once( &mut self, request: &DraftRequest, ) -> Result<DraftOutcome, DraftError>` | impl ProjectSession | [源码:121](../../../crates/application/src/draft.rs#L121) | skin_editor_application::DraftError, skin_editor_application::DraftOutcome, skin_editor_application::DraftRequest |
| `pub fn navigate( &mut self, request: HistoryRequest, ) -> Result<HistoryResult, HistoryError>` | impl ProjectSession | [源码:9](../../../crates/application/src/history_control.rs#L9) | skin_editor_application::HistoryError, skin_editor_application::HistoryRequest, skin_editor_application::HistoryResult |
| `fn navigation_target( &self, action: &HistoryAction, ) -> Result<NodeId, HistoryError>` | impl ProjectSession | [源码:42](../../../crates/application/src/history_control.rs#L42) | skin_editor_application::HistoryAction, skin_editor_application::HistoryError, skin_editor_application::NodeId |
| `fn navigate_once( &mut self, request: &HistoryRequest, ) -> Result<HistoryResult, HistoryError>` | impl ProjectSession | [源码:79](../../../crates/application/src/history_control.rs#L79) | skin_editor_application::HistoryError, skin_editor_application::HistoryRequest, skin_editor_application::HistoryResult |
| `fn history_scope( &self, session_id: HistorySessionId, ) -> Result<(), HistoryQueryError>` | impl ProjectSession | [源码:15](../../../crates/application/src/history_queries.rs#L15) | skin_editor_application::HistoryQueryError, skin_editor_application::HistorySessionId |
| `pub fn history_summary( &self, session_id: HistorySessionId, id: NodeId, ) -> Result<HistorySummary, HistoryQueryError>` | impl ProjectSession | [源码:22](../../../crates/application/src/history_queries.rs#L22) | skin_editor_application::HistoryQueryError, skin_editor_application::HistorySessionId, skin_editor_application::HistorySummary, skin_editor_application::NodeId |
| `pub fn resolve_history_address( &self, session_id: HistorySessionId, address: &str, ) -> Result<NodeId, HistoryQueryError>` | impl ProjectSession | [源码:31](../../../crates/application/src/history_queries.rs#L31) | skin_editor_application::HistoryQueryError, skin_editor_application::HistorySessionId, skin_editor_application::NodeId |
| `pub fn history_children( &self, session_id: HistorySessionId, parent: NodeId, offset: usize, limit: usize, ) -> Result<HistoryChildrenPage, HistoryQueryError>` | impl ProjectSession | [源码:56](../../../crates/application/src/history_queries.rs#L56) | skin_editor_application::HistoryChildrenPage, skin_editor_application::HistoryQueryError, skin_editor_application::HistorySessionId, skin_editor_application::NodeId |
| `pub fn history_ancestors( &self, session_id: HistorySessionId, start: NodeId, limit: usize, ) -> Result<HistoryAncestorsPage, HistoryQueryError>` | impl ProjectSession | [源码:87](../../../crates/application/src/history_queries.rs#L87) | skin_editor_application::HistoryAncestorsPage, skin_editor_application::HistoryQueryError, skin_editor_application::HistorySessionId, skin_editor_application::NodeId |
| `pub fn history_statistics( &self, session_id: HistorySessionId, ) -> Result<HistoryStatistics, HistoryQueryError>` | impl ProjectSession | [源码:113](../../../crates/application/src/history_queries.rs#L113) | skin_editor_application::HistoryQueryError, skin_editor_application::HistorySessionId, skin_editor_application::HistoryStatistics |
| `pub fn request_preview( &self, port: &mut dyn PreviewPort, context: PreviewContext, ) -> Result<FrameDescriptor, PreviewError>` | impl ProjectSession | [源码:59](../../../crates/application/src/preview_port.rs#L59) | skin_editor_application::FrameDescriptor, skin_editor_application::PreviewContext, skin_editor_application::PreviewError, skin_editor_application::PreviewPort |
| `pub fn new( resolved_project_id: u128, resolved_history_session_id: u128, ) -> Result<Self, SessionError>` | impl ProjectSession | [源码:160](../../../crates/application/src/lib.rs#L160) | skin_editor_application::SessionError |
| `pub fn with_resources( resolved_project_id: u128, resolved_history_session_id: u128, resources: Box<dyn ResourceStore>, limits: SessionLimits, ) -> Result<Self, SessionError>` | impl ProjectSession | [源码:174](../../../crates/application/src/lib.rs#L174) | skin_editor_application::ResourceStore, skin_editor_application::SessionError, skin_editor_application::SessionLimits |
| `pub fn project_info(&self) -> ProjectInfo` | impl ProjectSession | [源码:212](../../../crates/application/src/lib.rs#L212) | skin_editor_application::ProjectInfo |
| `pub fn snapshot(&self) -> ProjectSnapshot` | impl ProjectSession | [源码:221](../../../crates/application/src/lib.rs#L221) | skin_editor_application::ProjectSnapshot |
| `pub fn history_len(&self) -> usize` | impl ProjectSession | [源码:230](../../../crates/application/src/lib.rs#L230) |  |
| `pub fn history_entry( &self, session_id: HistorySessionId, id: NodeId, ) -> Result<HistoryEntry, HistoryQueryError>` | impl ProjectSession | [源码:234](../../../crates/application/src/lib.rs#L234) | skin_editor_application::HistoryEntry, skin_editor_application::HistoryQueryError, skin_editor_application::HistorySessionId, skin_editor_application::NodeId |
| `pub fn history_snapshot( &self, session_id: HistorySessionId, id: NodeId, ) -> Result<HistoricalSnapshot, HistoryQueryError>` | impl ProjectSession | [源码:248](../../../crates/application/src/lib.rs#L248) | skin_editor_application::HistoricalSnapshot, skin_editor_application::HistoryQueryError, skin_editor_application::HistorySessionId, skin_editor_application::NodeId |
| `pub(crate) fn envelope( &self, request: &CommandRequest, ) -> Result<(), CommandError>` | impl ProjectSession | [源码:269](../../../crates/application/src/lib.rs#L269) | skin_editor_application::CommandError, skin_editor_application::CommandRequest |
| `pub(crate) fn cached( &self, request: &CommandRequest, ) -> Result<Option<&Result<CommandResult, CommandError>>, CommandError>` | impl ProjectSession | [源码:282](../../../crates/application/src/lib.rs#L282) | skin_editor_application::CommandError, skin_editor_application::CommandRequest, skin_editor_application::CommandResult |
| `pub(crate) fn prepare( &self, expected: ExpectedState, commands: &[Command], ) -> Result<(SkinProject, BTreeSet<ObjectId>), CommandError>` | impl ProjectSession | [源码:296](../../../crates/application/src/lib.rs#L296) | skin_editor_application::Command, skin_editor_application::CommandError, skin_editor_application::ExpectedState, skin_editor_domain::ObjectId, skin_editor_domain::SkinProject |
| `pub fn validate( &self, request: &CommandRequest, ) -> Result<ValidationResult, CommandError>` | impl ProjectSession | [源码:331](../../../crates/application/src/lib.rs#L331) | skin_editor_application::CommandError, skin_editor_application::CommandRequest, skin_editor_application::ValidationResult |
| `pub fn submit( &mut self, request: CommandRequest, ) -> Result<CommandResult, CommandError>` | impl ProjectSession | [源码:379](../../../crates/application/src/lib.rs#L379) | skin_editor_application::CommandError, skin_editor_application::CommandRequest, skin_editor_application::CommandResult |
| `pub(crate) fn commit( &mut self, request: &CommandRequest, ) -> Result<CommandResult, CommandError>` | impl ProjectSession | [源码:402](../../../crates/application/src/lib.rs#L402) | skin_editor_application::CommandError, skin_editor_application::CommandRequest, skin_editor_application::CommandResult |
| `pub(crate) fn pin_candidate( &self, candidate: &SkinProject, temporary: &BTreeMap<BlobRef, Arc<dyn BlobLease>>, ) -> Result<BTreeMap<BlobRef, Arc<dyn BlobLease>>, CommandError>` | impl ProjectSession | [源码:407](../../../crates/application/src/lib.rs#L407) | skin_editor_application::BlobLease, skin_editor_application::CommandError, skin_editor_domain::BlobRef, skin_editor_domain::SkinProject |
| `pub(crate) fn publish( &mut self, request_id: RequestId, candidate: SkinProject, used: BTreeSet<ObjectId>, temporary: &BTreeMap<BlobRef, Arc<dyn BlobLease>>, ) -> Result<CommandResult, CommandError>` | impl ProjectSession | [源码:446](../../../crates/application/src/lib.rs#L446) | skin_editor_application::BlobLease, skin_editor_application::CommandError, skin_editor_application::CommandResult, skin_editor_application::RequestId, skin_editor_domain::BlobRef, skin_editor_domain::ObjectId, skin_editor_domain::SkinProject |

## skin_editor_application::ProjectSnapshot

[源码:77](../../../crates/application/src/lib.rs#L77) · struct

```rust
pub struct ProjectSnapshot {
    pub api_version: u16,
    pub session_id: HistorySessionId,
    pub state: ExpectedState,
    pub project: SkinProject,
}
```

声明引用：`skin_editor_application::ExpectedState`, `skin_editor_application::HistorySessionId`, `skin_editor_domain::SkinProject`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::ProjectSession::snapshot`, `skin_editor_application::ProjectSnapshot::clone`, `skin_editor_application::ProjectSnapshot::eq`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for ProjectSnapshot` — [源码:76](../../../crates/application/src/lib.rs#L76)
- `impl Debug for ProjectSnapshot` — [源码:76](../../../crates/application/src/lib.rs#L76)
- `impl PartialEq for ProjectSnapshot` — [源码:76](../../../crates/application/src/lib.rs#L76)
- `impl StructuralPartialEq for ProjectSnapshot` — [源码:76](../../../crates/application/src/lib.rs#L76)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> ProjectSnapshot` | impl Clone for ProjectSnapshot | [源码:76](../../../crates/application/src/lib.rs#L76) | skin_editor_application::ProjectSnapshot |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for ProjectSnapshot | [源码:76](../../../crates/application/src/lib.rs#L76) |  |
| `fn eq(&self, other: &ProjectSnapshot) -> bool` | impl PartialEq for ProjectSnapshot | [源码:76](../../../crates/application/src/lib.rs#L76) | skin_editor_application::ProjectSnapshot |

## skin_editor_application::RasterImportPort

[源码:104](../../../crates/application/src/preview_port.rs#L104) · trait

```rust
pub trait RasterImportPort {
    // Required method
    fn stage_png(&mut self, source: &str) -> Result<StagedRaster, ImportError>;
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::preview_port::RasterImportPort`

声明引用：`skin_editor_application::ImportError`, `skin_editor_application::StagedRaster`

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn stage_png(&mut self, source: &str) -> Result<StagedRaster, ImportError>` |  | [源码:106](../../../crates/application/src/preview_port.rs#L106) | skin_editor_application::ImportError, skin_editor_application::StagedRaster |

## skin_editor_application::Receipt

[源码:110](../../../crates/application/src/lib.rs#L110) · enum

```rust
pub(crate) enum Receipt {
    Command {
        request: CommandRequest,
        outcome: Result<CommandResult, CommandError>,
    },
    History {
        request: HistoryRequest,
        outcome: Result<HistoryResult, HistoryError>,
    },
    Draft {
        request: DraftRequest,
        outcome: Result<DraftResult, DraftError>,
    },
}
```

声明引用：`skin_editor_application::CommandError`, `skin_editor_application::CommandRequest`, `skin_editor_application::CommandResult`, `skin_editor_application::DraftError`, `skin_editor_application::DraftRequest`, `skin_editor_application::DraftResult`, `skin_editor_application::HistoryError`, `skin_editor_application::HistoryRequest`, `skin_editor_application::HistoryResult`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::ProjectSession (声明)`

## skin_editor_application::RequestId

[源码:26](../../../crates/application/src/contract.rs#L26) · struct

```rust
pub struct RequestId(NonZeroU128);
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::contract::RequestId`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::CommandRequest (声明)`, `skin_editor_application::CommandResult (声明)`, `skin_editor_application::DraftRequest (声明)`, `skin_editor_application::DraftResult (声明)`, `skin_editor_application::HistoryEntry (声明)`, `skin_editor_application::HistoryRequest (声明)`, `skin_editor_application::HistoryResult (声明)`, `skin_editor_application::ProjectSession (声明)`, `skin_editor_application::ProjectSession::publish`, `skin_editor_application::RequestId::clone`, `skin_editor_application::RequestId::cmp`, `skin_editor_application::RequestId::eq`, `skin_editor_application::RequestId::partial_cmp`, `skin_editor_application::history::History::append`, `skin_editor_application::history::Node (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl RequestId` — [源码:27](../../../crates/application/src/contract.rs#L27)
- `impl Clone for RequestId` — [源码:25](../../../crates/application/src/contract.rs#L25)
- `impl Debug for RequestId` — [源码:25](../../../crates/application/src/contract.rs#L25)
- `impl Ord for RequestId` — [源码:25](../../../crates/application/src/contract.rs#L25)
- `impl PartialEq for RequestId` — [源码:25](../../../crates/application/src/contract.rs#L25)
- `impl PartialOrd for RequestId` — [源码:25](../../../crates/application/src/contract.rs#L25)
- `impl Copy for RequestId` — [源码:25](../../../crates/application/src/contract.rs#L25)
- `impl Eq for RequestId` — [源码:25](../../../crates/application/src/contract.rs#L25)
- `impl StructuralPartialEq for RequestId` — [源码:25](../../../crates/application/src/contract.rs#L25)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub const fn new(value: u128) -> Option<Self>` | impl RequestId | [源码:28](../../../crates/application/src/contract.rs#L28) |  |
| `pub const fn get(self) -> u128` | impl RequestId | [源码:34](../../../crates/application/src/contract.rs#L34) |  |
| `fn clone(&self) -> RequestId` | impl Clone for RequestId | [源码:25](../../../crates/application/src/contract.rs#L25) | skin_editor_application::RequestId |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for RequestId | [源码:25](../../../crates/application/src/contract.rs#L25) |  |
| `fn cmp(&self, other: &RequestId) -> Ordering` | impl Ord for RequestId | [源码:25](../../../crates/application/src/contract.rs#L25) | skin_editor_application::RequestId |
| `fn eq(&self, other: &RequestId) -> bool` | impl PartialEq for RequestId | [源码:25](../../../crates/application/src/contract.rs#L25) | skin_editor_application::RequestId |
| `fn partial_cmp(&self, other: &RequestId) -> Option<Ordering>` | impl PartialOrd for RequestId | [源码:25](../../../crates/application/src/contract.rs#L25) | skin_editor_application::RequestId |

## skin_editor_application::ResourceError

[源码:6](../../../crates/application/src/resources.rs#L6) · enum

```rust
pub enum ResourceError {
    Missing,
    Unavailable,
    ReferenceMismatch,
    MetadataMismatch,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::resources::ResourceError`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::CommandError (声明)`, `skin_editor_application::ResourceError::clone`, `skin_editor_application::ResourceError::eq`, `skin_editor_application::ResourceStore (声明)`, `skin_editor_application::ResourceStore::pin`, `skin_editor_application::resources::NoResources::pin`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for ResourceError` — [源码:5](../../../crates/application/src/resources.rs#L5)
- `impl Debug for ResourceError` — [源码:5](../../../crates/application/src/resources.rs#L5)
- `impl PartialEq for ResourceError` — [源码:5](../../../crates/application/src/resources.rs#L5)
- `impl Eq for ResourceError` — [源码:5](../../../crates/application/src/resources.rs#L5)
- `impl StructuralPartialEq for ResourceError` — [源码:5](../../../crates/application/src/resources.rs#L5)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> ResourceError` | impl Clone for ResourceError | [源码:5](../../../crates/application/src/resources.rs#L5) | skin_editor_application::ResourceError |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for ResourceError | [源码:5](../../../crates/application/src/resources.rs#L5) |  |
| `fn eq(&self, other: &ResourceError) -> bool` | impl PartialEq for ResourceError | [源码:5](../../../crates/application/src/resources.rs#L5) | skin_editor_application::ResourceError |

## skin_editor_application::ResourceStore

[源码:20](../../../crates/application/src/resources.rs#L20) · trait

```rust
pub trait ResourceStore {
    // Required method
    fn pin(&self, blob: BlobRef) -> Result<Arc<dyn BlobLease>, ResourceError>;

    // Provided method
    fn audio_metadata(&self, _: BlobRef) -> Option<AudioMetadata> { ... }
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::resources::ResourceStore`

声明引用：`skin_editor_application::BlobLease`, `skin_editor_application::ResourceError`, `skin_editor_domain::AudioMetadata`, `skin_editor_domain::BlobRef`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::ProjectSession (声明)`, `skin_editor_application::ProjectSession::with_resources`

实现／约束（显式或 derive，无类继承）：

- `impl ResourceStore for NoResources` — [源码:33](../../../crates/application/src/resources.rs#L33)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn pin(&self, blob: BlobRef) -> Result<Arc<dyn BlobLease>, ResourceError>` |  | [源码:29](../../../crates/application/src/resources.rs#L29) | skin_editor_application::BlobLease, skin_editor_application::ResourceError, skin_editor_domain::BlobRef |
| `fn audio_metadata(&self, _: BlobRef) -> Option<AudioMetadata>` |  | [源码:22](../../../crates/application/src/resources.rs#L22) | skin_editor_domain::AudioMetadata, skin_editor_domain::BlobRef |

## skin_editor_application::SKIN_AUDIO_API_VERSION

[源码:5](../../../crates/application/src/audio_port.rs#L5) · constant

```rust
pub const SKIN_AUDIO_API_VERSION: u16 = 1;
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::audio_port::SKIN_AUDIO_API_VERSION`

## skin_editor_application::SessionError

[源码:57](../../../crates/application/src/lib.rs#L57) · enum

```rust
pub enum SessionError {
    InvalidProjectId,
    InvalidHistorySessionId,
    InvalidLimits,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::ProjectSession::new`, `skin_editor_application::ProjectSession::with_resources`, `skin_editor_application::SessionError::clone`, `skin_editor_application::SessionError::eq`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for SessionError` — [源码:56](../../../crates/application/src/lib.rs#L56)
- `impl Debug for SessionError` — [源码:56](../../../crates/application/src/lib.rs#L56)
- `impl PartialEq for SessionError` — [源码:56](../../../crates/application/src/lib.rs#L56)
- `impl Copy for SessionError` — [源码:56](../../../crates/application/src/lib.rs#L56)
- `impl Eq for SessionError` — [源码:56](../../../crates/application/src/lib.rs#L56)
- `impl StructuralPartialEq for SessionError` — [源码:56](../../../crates/application/src/lib.rs#L56)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> SessionError` | impl Clone for SessionError | [源码:56](../../../crates/application/src/lib.rs#L56) | skin_editor_application::SessionError |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for SessionError | [源码:56](../../../crates/application/src/lib.rs#L56) |  |
| `fn eq(&self, other: &SessionError) -> bool` | impl PartialEq for SessionError | [源码:56](../../../crates/application/src/lib.rs#L56) | skin_editor_application::SessionError |

## skin_editor_application::SessionLimits

[源码:95](../../../crates/application/src/lib.rs#L95) · struct

```rust
pub struct SessionLimits {
    pub commands_per_request: usize,
    pub retained_requests: usize,
    pub history_nodes: usize,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::ProjectSession (声明)`, `skin_editor_application::ProjectSession::with_resources`, `skin_editor_application::SessionLimits::clone`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for SessionLimits` — [源码:94](../../../crates/application/src/lib.rs#L94)
- `impl Debug for SessionLimits` — [源码:94](../../../crates/application/src/lib.rs#L94)
- `impl Default for SessionLimits` — [源码:100](../../../crates/application/src/lib.rs#L100)
- `impl Copy for SessionLimits` — [源码:94](../../../crates/application/src/lib.rs#L94)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> SessionLimits` | impl Clone for SessionLimits | [源码:94](../../../crates/application/src/lib.rs#L94) | skin_editor_application::SessionLimits |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for SessionLimits | [源码:94](../../../crates/application/src/lib.rs#L94) |  |
| `fn default() -> Self` | impl Default for SessionLimits | [源码:101](../../../crates/application/src/lib.rs#L101) |  |

## skin_editor_application::SkinAudioError

[源码:17](../../../crates/application/src/audio_port.rs#L17) · enum

```rust
pub enum SkinAudioError {
    ResourceLimit,
    Backend(String),
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::audio_port::SkinAudioError`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::AudioEditorError (声明)`, `skin_editor_application::DisabledAudio::synchronize`, `skin_editor_application::ProjectSession::synchronize_audio`, `skin_editor_application::SkinAudioError::clone`, `skin_editor_application::SkinAudioError::eq`, `skin_editor_application::SkinAudioPort (声明)`, `skin_editor_application::SkinAudioPort::synchronize`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for SkinAudioError` — [源码:16](../../../crates/application/src/audio_port.rs#L16)
- `impl Debug for SkinAudioError` — [源码:16](../../../crates/application/src/audio_port.rs#L16)
- `impl PartialEq for SkinAudioError` — [源码:16](../../../crates/application/src/audio_port.rs#L16)
- `impl Eq for SkinAudioError` — [源码:16](../../../crates/application/src/audio_port.rs#L16)
- `impl StructuralPartialEq for SkinAudioError` — [源码:16](../../../crates/application/src/audio_port.rs#L16)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> SkinAudioError` | impl Clone for SkinAudioError | [源码:16](../../../crates/application/src/audio_port.rs#L16) | skin_editor_application::SkinAudioError |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for SkinAudioError | [源码:16](../../../crates/application/src/audio_port.rs#L16) |  |
| `fn eq(&self, other: &SkinAudioError) -> bool` | impl PartialEq for SkinAudioError | [源码:16](../../../crates/application/src/audio_port.rs#L16) | skin_editor_application::SkinAudioError |

## skin_editor_application::SkinAudioPort

[源码:22](../../../crates/application/src/audio_port.rs#L22) · trait

```rust
pub trait SkinAudioPort {
    // Required method
    fn synchronize(
        &mut self,
        project: &SkinProject,
        source: SkinAudioSource,
        cancelled_drafts: &[DraftId],
    ) -> Result<(), SkinAudioError>;

    // Provided method
    fn retained_drafts(&self) -> Vec<DraftId> { ... }
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::audio_port::SkinAudioPort`

声明引用：`skin_editor_application::DraftId`, `skin_editor_application::SkinAudioError`, `skin_editor_application::SkinAudioSource`, `skin_editor_domain::SkinProject`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::AudioEditorPort (声明)`, `skin_editor_application::ProjectSession::synchronize_audio`

实现／约束（显式或 derive，无类继承）：

- `impl SkinAudioPort for DisabledAudio` — [源码:92](../../../crates/application/src/audio_editor.rs#L92)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn synchronize( &mut self, project: &SkinProject, source: SkinAudioSource, cancelled_drafts: &[DraftId], ) -> Result<(), SkinAudioError>` |  | [源码:28](../../../crates/application/src/audio_port.rs#L28) | skin_editor_application::DraftId, skin_editor_application::SkinAudioError, skin_editor_application::SkinAudioSource, skin_editor_domain::SkinProject |
| `fn retained_drafts(&self) -> Vec<DraftId>` |  | [源码:24](../../../crates/application/src/audio_port.rs#L24) | skin_editor_application::DraftId |

## skin_editor_application::SkinAudioSource

[源码:9](../../../crates/application/src/audio_port.rs#L9) · struct

```rust
pub struct SkinAudioSource {
    pub session: HistorySessionId,
    pub state: ExpectedState,
    pub navigation_revision: u64,
    pub draft: Option<DraftState>,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::audio_port::SkinAudioSource`

声明引用：`skin_editor_application::DraftState`, `skin_editor_application::ExpectedState`, `skin_editor_application::HistorySessionId`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::AssetAudition (声明)`, `skin_editor_application::AssetAuditionState (声明)`, `skin_editor_application::AudioComponentState (声明)`, `skin_editor_application::AudioEditorState (声明)`, `skin_editor_application::DisabledAudio::synchronize`, `skin_editor_application::ProjectSession::skin_audio_source`, `skin_editor_application::SkinAudioPort (声明)`, `skin_editor_application::SkinAudioPort::synchronize`, `skin_editor_application::SkinAudioSource::clone`, `skin_editor_application::SkinAudioSource::eq`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for SkinAudioSource` — [源码:8](../../../crates/application/src/audio_port.rs#L8)
- `impl Debug for SkinAudioSource` — [源码:8](../../../crates/application/src/audio_port.rs#L8)
- `impl PartialEq for SkinAudioSource` — [源码:8](../../../crates/application/src/audio_port.rs#L8)
- `impl Copy for SkinAudioSource` — [源码:8](../../../crates/application/src/audio_port.rs#L8)
- `impl Eq for SkinAudioSource` — [源码:8](../../../crates/application/src/audio_port.rs#L8)
- `impl StructuralPartialEq for SkinAudioSource` — [源码:8](../../../crates/application/src/audio_port.rs#L8)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> SkinAudioSource` | impl Clone for SkinAudioSource | [源码:8](../../../crates/application/src/audio_port.rs#L8) | skin_editor_application::SkinAudioSource |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for SkinAudioSource | [源码:8](../../../crates/application/src/audio_port.rs#L8) |  |
| `fn eq(&self, other: &SkinAudioSource) -> bool` | impl PartialEq for SkinAudioSource | [源码:8](../../../crates/application/src/audio_port.rs#L8) | skin_editor_application::SkinAudioSource |

## skin_editor_application::StagedRaster

[源码:98](../../../crates/application/src/preview_port.rs#L98) · struct

```rust
pub struct StagedRaster {
    pub blob: BlobRef,
    pub size: PixelSize,
    pub label: String,
    pub lease: Arc<dyn BlobLease>,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::preview_port::StagedRaster`

声明引用：`skin_editor_application::BlobLease`, `skin_editor_domain::BlobRef`, `skin_editor_domain::PixelSize`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::RasterImportPort (声明)`, `skin_editor_application::RasterImportPort::stage_png`

## skin_editor_application::ValidationResult

[源码:256](../../../crates/application/src/contract.rs#L256) · struct

```rust
pub struct ValidationResult {
    pub api_version: u16,
    pub state: ExpectedState,
    pub would_change: bool,
    pub resource_check_pending: bool,
    pub cached: bool,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::contract::ValidationResult`

声明引用：`skin_editor_application::ExpectedState`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::ProjectSession::validate`, `skin_editor_application::ValidationResult::clone`, `skin_editor_application::ValidationResult::eq`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for ValidationResult` — [源码:255](../../../crates/application/src/contract.rs#L255)
- `impl Debug for ValidationResult` — [源码:255](../../../crates/application/src/contract.rs#L255)
- `impl PartialEq for ValidationResult` — [源码:255](../../../crates/application/src/contract.rs#L255)
- `impl Eq for ValidationResult` — [源码:255](../../../crates/application/src/contract.rs#L255)
- `impl StructuralPartialEq for ValidationResult` — [源码:255](../../../crates/application/src/contract.rs#L255)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> ValidationResult` | impl Clone for ValidationResult | [源码:255](../../../crates/application/src/contract.rs#L255) | skin_editor_application::ValidationResult |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for ValidationResult | [源码:255](../../../crates/application/src/contract.rs#L255) |  |
| `fn eq(&self, other: &ValidationResult) -> bool` | impl PartialEq for ValidationResult | [源码:255](../../../crates/application/src/contract.rs#L255) | skin_editor_application::ValidationResult |

## skin_editor_application::address::encode

[源码:6](../../../crates/application/src/address.rs#L6) · fn

```rust
fn encode(value: u64) -> String
```

## skin_editor_application::address::number

[源码:26](../../../crates/application/src/address.rs#L26) · fn

```rust
fn number(text: &str) -> Result<u64, AddressError>
```

声明引用：`skin_editor_application::AddressError`

## skin_editor_application::draft::Draft

[源码:10](../../../crates/application/src/draft.rs#L10) · struct

```rust
pub(crate) struct Draft {
    pub state: DraftState,
    commands: Vec<Command>,
    pub(crate) project: SkinProject,
    used: BTreeSet<ObjectId>,
    pins: BTreeMap<BlobRef, Arc<dyn BlobLease>>,
}
```

声明引用：`skin_editor_application::BlobLease`, `skin_editor_application::Command`, `skin_editor_application::DraftState`, `skin_editor_domain::BlobRef`, `skin_editor_domain::ObjectId`, `skin_editor_domain::SkinProject`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::ProjectSession (声明)`, `skin_editor_application::ProjectSession::expected_draft`

## skin_editor_application::execute::ComponentNodes

[源码:71](../../../crates/application/src/execute.rs#L71) · type

```rust
type ComponentNodes = BTreeMap<ComponentId, ComponentNode>;
```

声明引用：`skin_editor_domain::ComponentId`, `skin_editor_domain::ComponentNode`

## skin_editor_application::execute::LayerNodes

[源码:32](../../../crates/application/src/execute.rs#L32) · type

```rust
type LayerNodes = BTreeMap<LayerId, LayerNode>;
```

声明引用：`skin_editor_domain::LayerId`, `skin_editor_domain::LayerNode`

## skin_editor_application::execute::apply

[源码:114](../../../crates/application/src/execute.rs#L114) · fn

```rust
pub(crate) fn apply(
    project: &SkinProject,
    command: &Command,
    used: &mut BTreeSet<ObjectId>,
) -> Result<SkinProject, CommandFailure>
```

声明引用：`skin_editor_application::Command`, `skin_editor_application::CommandFailure`, `skin_editor_domain::ObjectId`, `skin_editor_domain::SkinProject`

## skin_editor_application::execute::component_children

[源码:72](../../../crates/application/src/execute.rs#L72) · fn

```rust
fn component_children<'a>(
    roots: &'a mut Vec<ComponentId>,
    nodes: &'a mut BTreeMap<ComponentId, ComponentNode>,
    parent: Option<ComponentId>,
) -> Result<&'a mut Vec<ComponentId>, CommandFailure>
```

声明引用：`skin_editor_application::CommandFailure`, `skin_editor_domain::ComponentId`, `skin_editor_domain::ComponentNode`

## skin_editor_application::execute::component_node

[源码:89](../../../crates/application/src/execute.rs#L89) · fn

```rust
fn component_node(
    nodes: &mut BTreeMap<ComponentId, ComponentNode>,
    id: ComponentId,
) -> Result<&mut ComponentNode, CommandFailure>
```

声明引用：`skin_editor_application::CommandFailure`, `skin_editor_domain::ComponentId`, `skin_editor_domain::ComponentNode`

## skin_editor_application::execute::components

[源码:97](../../../crates/application/src/execute.rs#L97) · fn

```rust
fn components(
    input: &mut ProjectDefinition,
    edit: impl FnOnce(&ComponentTree, &mut Vec<ComponentId>, &mut BTreeMap<ComponentId, ComponentNode>) -> Result<(), CommandFailure>,
) -> Result<(), CommandFailure>
```

声明引用：`skin_editor_application::CommandFailure`, `skin_editor_domain::ComponentId`, `skin_editor_domain::ComponentNode`, `skin_editor_domain::ComponentTree`, `skin_editor_domain::ProjectDefinition`

## skin_editor_application::execute::doc_index

[源码:24](../../../crates/application/src/execute.rs#L24) · fn

```rust
fn doc_index(
    input: &ProjectDefinition,
    id: DocumentId,
) -> Result<usize, CommandFailure>
```

声明引用：`skin_editor_application::CommandFailure`, `skin_editor_domain::DocumentId`, `skin_editor_domain::ProjectDefinition`

## skin_editor_application::execute::layer_children

[源码:33](../../../crates/application/src/execute.rs#L33) · fn

```rust
fn layer_children<'a>(
    roots: &'a mut Vec<LayerId>,
    nodes: &'a mut BTreeMap<LayerId, LayerNode>,
    parent: Option<LayerId>,
) -> Result<&'a mut Vec<LayerId>, CommandFailure>
```

声明引用：`skin_editor_application::CommandFailure`, `skin_editor_domain::LayerId`, `skin_editor_domain::LayerNode`

## skin_editor_application::execute::layer_node

[源码:50](../../../crates/application/src/execute.rs#L50) · fn

```rust
fn layer_node(
    nodes: &mut BTreeMap<LayerId, LayerNode>,
    id: LayerId,
) -> Result<&mut LayerNode, CommandFailure>
```

声明引用：`skin_editor_application::CommandFailure`, `skin_editor_domain::LayerId`, `skin_editor_domain::LayerNode`

## skin_editor_application::execute::layers

[源码:55](../../../crates/application/src/execute.rs#L55) · fn

```rust
fn layers(
    input: &mut ProjectDefinition,
    document: DocumentId,
    edit: impl FnOnce(&LayerTree, &mut Vec<LayerId>, &mut BTreeMap<LayerId, LayerNode>) -> Result<(), CommandFailure>,
) -> Result<(), CommandFailure>
```

声明引用：`skin_editor_application::CommandFailure`, `skin_editor_domain::DocumentId`, `skin_editor_domain::LayerId`, `skin_editor_domain::LayerNode`, `skin_editor_domain::LayerTree`, `skin_editor_domain::ProjectDefinition`

## skin_editor_application::execute::position

[源码:18](../../../crates/application/src/execute.rs#L18) · fn

```rust
fn position(index: usize, len: usize) -> Result<(), CommandFailure>
```

声明引用：`skin_editor_application::CommandFailure`

## skin_editor_application::execute::reserve

[源码:12](../../../crates/application/src/execute.rs#L12) · fn

```rust
fn reserve(
    used: &mut BTreeSet<ObjectId>,
    id: ObjectId,
) -> Result<(), CommandFailure>
```

声明引用：`skin_editor_application::CommandFailure`, `skin_editor_domain::ObjectId`

## skin_editor_application::history::AppendPlan

[源码:213](../../../crates/application/src/history.rs#L213) · struct

```rust
pub(crate) struct AppendPlan {
    pub id: NodeId,
    parent: NodeId,
    depth: u64,
    coordinate: BranchCoordinate,
    new_branch: bool,
}
```

声明引用：`skin_editor_application::BranchCoordinate`, `skin_editor_application::NodeId`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::history::History::append`, `skin_editor_application::history::History::plan_append`

## skin_editor_application::history::Change

[源码:27](../../../crates/application/src/history.rs#L27) · struct

```rust
struct Change<I, V> {
    id: I,
    before: Option<V>,
    after: Option<V>,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::history::Change::clone`, `skin_editor_application::history::Patch (声明)`, `skin_editor_application::history::changes (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl<I: Clone, V: Clone> Clone for Change<I, V>` — [源码:26](../../../crates/application/src/history.rs#L26)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> Change<I, V>` | impl<I: Clone, V: Clone> Clone for Change<I, V> | [源码:26](../../../crates/application/src/history.rs#L26) | skin_editor_application::history::Change |

## skin_editor_application::history::History

[源码:203](../../../crates/application/src/history.rs#L203) · struct

```rust
pub(crate) struct History {
    pub root: SkinProject,
    pub nodes: BTreeMap<NodeId, Node>,
    pub next_id: u64,
    pub coordinates: BTreeMap<BranchCoordinate, NodeId>,
    pub next_branch_id: u64,
    pub branch_count: usize,
    pub max_depth: u64,
}
```

声明引用：`skin_editor_application::BranchCoordinate`, `skin_editor_application::NodeId`, `skin_editor_application::history::Node`, `skin_editor_domain::SkinProject`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::ProjectSession (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl History` — [源码:221](../../../crates/application/src/history.rs#L221)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn new(root: SkinProject) -> Self` | impl History | [源码:222](../../../crates/application/src/history.rs#L222) | skin_editor_domain::SkinProject |
| `pub fn plan_append(&self, parent: NodeId) -> Result<AppendPlan, CommandError>` | impl History | [源码:248](../../../crates/application/src/history.rs#L248) | skin_editor_application::CommandError, skin_editor_application::NodeId, skin_editor_application::history::AppendPlan |
| `pub fn append(&mut self, plan: AppendPlan, patch: Patch, request: RequestId)` | impl History | [源码:285](../../../crates/application/src/history.rs#L285) | skin_editor_application::RequestId, skin_editor_application::history::AppendPlan, skin_editor_application::history::Patch |
| `pub fn summary(&self, id: NodeId) -> Result<HistorySummary, HistoryQueryError>` | impl History | [源码:311](../../../crates/application/src/history.rs#L311) | skin_editor_application::HistoryQueryError, skin_editor_application::HistorySummary, skin_editor_application::NodeId |
| `pub fn node(&self, id: NodeId) -> Result<&Node, HistoryQueryError>` | impl History | [源码:322](../../../crates/application/src/history.rs#L322) | skin_editor_application::HistoryQueryError, skin_editor_application::NodeId, skin_editor_application::history::Node |
| `fn parent(&self, id: NodeId) -> Result<NodeId, HistoryQueryError>` | impl History | [源码:328](../../../crates/application/src/history.rs#L328) | skin_editor_application::HistoryQueryError, skin_editor_application::NodeId |
| `pub fn entry( &self, session_id: HistorySessionId, id: NodeId, ) -> Option<HistoryEntry>` | impl History | [源码:335](../../../crates/application/src/history.rs#L335) | skin_editor_application::HistoryEntry, skin_editor_application::HistorySessionId, skin_editor_application::NodeId |
| `pub fn materialize(&self, id: NodeId) -> Result<SkinProject, HistoryQueryError>` | impl History | [源码:353](../../../crates/application/src/history.rs#L353) | skin_editor_application::HistoryQueryError, skin_editor_application::NodeId, skin_editor_domain::SkinProject |
| `pub fn transition( &self, from: NodeId, to: NodeId, current: &SkinProject, ) -> Result<SkinProject, HistoryQueryError>` | impl History | [源码:359](../../../crates/application/src/history.rs#L359) | skin_editor_application::HistoryQueryError, skin_editor_application::NodeId, skin_editor_domain::SkinProject |

## skin_editor_application::history::Node

[源码:195](../../../crates/application/src/history.rs#L195) · struct

```rust
pub(crate) struct Node {
    pub parent: Option<NodeId>,
    pub patch: Option<Patch>,
    pub request: Option<RequestId>,
    pub children: Vec<NodeId>,
    pub depth: u64,
    pub coordinate: BranchCoordinate,
}
```

声明引用：`skin_editor_application::BranchCoordinate`, `skin_editor_application::NodeId`, `skin_editor_application::RequestId`, `skin_editor_application::history::Patch`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::history::History (声明)`, `skin_editor_application::history::History::node`

## skin_editor_application::history::Patch

[源码:56](../../../crates/application/src/history.rs#L56) · struct

```rust
pub(crate) struct Patch {
    assets: Vec<Change<AssetId, Asset>>,
    documents: Vec<Change<DocumentId, ElementDocument>>,
    audio_components: Vec<Change<AudioComponentId, AudioComponent>>,
    components: Option<(ComponentTree, ComponentTree)>,
    configuration: Option<(SkinConfiguration, SkinConfiguration)>,
    outputs: Option<(Vec<OutputMapping>, Vec<OutputMapping>)>,
    pub changed: Vec<ObjectId>,
}
```

声明引用：`skin_editor_application::history::Change`, `skin_editor_domain::Asset`, `skin_editor_domain::AssetId`, `skin_editor_domain::AudioComponent`, `skin_editor_domain::AudioComponentId`, `skin_editor_domain::ComponentTree`, `skin_editor_domain::DocumentId`, `skin_editor_domain::ElementDocument`, `skin_editor_domain::ObjectId`, `skin_editor_domain::OutputMapping`, `skin_editor_domain::SkinConfiguration`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::history::History::append`, `skin_editor_application::history::Node (声明)`, `skin_editor_application::history::Patch::clone`

实现／约束（显式或 derive，无类继承）：

- `impl Patch` — [源码:66](../../../crates/application/src/history.rs#L66)
- `impl Clone for Patch` — [源码:55](../../../crates/application/src/history.rs#L55)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn between(old: &SkinProject, new: &SkinProject) -> Self` | impl Patch | [源码:67](../../../crates/application/src/history.rs#L67) | skin_editor_domain::SkinProject |
| `pub fn configuration_changed(&self) -> bool` | impl Patch | [源码:135](../../../crates/application/src/history.rs#L135) |  |
| `fn apply_definition(&self, input: &mut ProjectDefinition, forward: bool)` | impl Patch | [源码:146](../../../crates/application/src/history.rs#L146) | skin_editor_domain::ProjectDefinition |
| `fn clone(&self) -> Patch` | impl Clone for Patch | [源码:55](../../../crates/application/src/history.rs#L55) | skin_editor_application::history::Patch |

## skin_editor_application::history::changes

[源码:33](../../../crates/application/src/history.rs#L33) · fn

```rust
fn changes<I: Copy + Ord, V: Clone + PartialEq>(
    before: BTreeMap<I, V>,
    after: BTreeMap<I, V>,
) -> Vec<Change<I, V>>
```

声明引用：`skin_editor_application::history::Change`

## skin_editor_application::history::definition

[源码:13](../../../crates/application/src/history.rs#L13) · fn

```rust
pub(crate) fn definition(project: &SkinProject) -> ProjectDefinition
```

声明引用：`skin_editor_domain::ProjectDefinition`, `skin_editor_domain::SkinProject`

## skin_editor_application::history_queries::page_limit

[源码:7](../../../crates/application/src/history_queries.rs#L7) · fn

```rust
fn page_limit(limit: usize) -> Result<(), HistoryQueryError>
```

声明引用：`skin_editor_application::HistoryQueryError`

## skin_editor_application::resources::NoResources

[源码:32](../../../crates/application/src/resources.rs#L32) · struct

```rust
pub(crate) struct NoResources;
```

实现／约束（显式或 derive，无类继承）：

- `impl ResourceStore for NoResources` — [源码:33](../../../crates/application/src/resources.rs#L33)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn pin(&self, _: BlobRef) -> Result<Arc<dyn BlobLease>, ResourceError>` | impl ResourceStore for NoResources | [源码:34](../../../crates/application/src/resources.rs#L34) | skin_editor_application::BlobLease, skin_editor_application::ResourceError, skin_editor_domain::BlobRef |
| `fn audio_metadata(&self, _: BlobRef) -> Option<AudioMetadata>` | impl ResourceStore for NoResources | [源码:22](../../../crates/application/src/resources.rs#L22) | skin_editor_domain::AudioMetadata, skin_editor_domain::BlobRef |

