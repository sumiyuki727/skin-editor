# skin_editor_infrastructure：编译器符号清单

快照：2026-10-30。来自含私有项的 rustdoc；显示编译器规范化可见域。含 derive 生成的实现，不把它们视作类继承。来源行可能指向宏定义／展开点。
引用列只列 rustdoc 解析出的项目类型链接，不能充当完整调用图；无链接不表示无引用。所有方法保留源码位置供逐项复核。测试和未启用的 cfg 分支另见源码索引。

## skin_editor_infrastructure::Admission

[源码:160](../../../desktop/infrastructure/src/audio_preparation.rs#L160) · enum

```rust
pub enum Admission {
    Queued,
    Cached(PreparedSound),
    AlreadyQueued,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_infrastructure::audio_preparation::Admission`

声明引用：`skin_editor_audio::PreparedSound`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::AudioPreparationQueue::submit`

## skin_editor_infrastructure::AdmissionState

[源码:253](../../../desktop/infrastructure/src/skin_audio.rs#L253) · enum

```rust
pub enum AdmissionState {
    Queued,
    AlreadyQueued,
    Cached(Completion),
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_infrastructure::skin_audio::AdmissionState`

声明引用：`skin_editor_audio::Completion`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::AdmissionState::clone`, `skin_editor_infrastructure::AdmissionState::eq`, `skin_editor_infrastructure::BuildAdmission (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for AdmissionState` — [源码:252](../../../desktop/infrastructure/src/skin_audio.rs#L252)
- `impl Debug for AdmissionState` — [源码:252](../../../desktop/infrastructure/src/skin_audio.rs#L252)
- `impl PartialEq for AdmissionState` — [源码:252](../../../desktop/infrastructure/src/skin_audio.rs#L252)
- `impl Copy for AdmissionState` — [源码:252](../../../desktop/infrastructure/src/skin_audio.rs#L252)
- `impl Eq for AdmissionState` — [源码:252](../../../desktop/infrastructure/src/skin_audio.rs#L252)
- `impl StructuralPartialEq for AdmissionState` — [源码:252](../../../desktop/infrastructure/src/skin_audio.rs#L252)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> AdmissionState` | impl Clone for AdmissionState | [源码:252](../../../desktop/infrastructure/src/skin_audio.rs#L252) | skin_editor_infrastructure::AdmissionState |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for AdmissionState | [源码:252](../../../desktop/infrastructure/src/skin_audio.rs#L252) |  |
| `fn eq(&self, other: &AdmissionState) -> bool` | impl PartialEq for AdmissionState | [源码:252](../../../desktop/infrastructure/src/skin_audio.rs#L252) | skin_editor_infrastructure::AdmissionState |

## skin_editor_infrastructure::AudioDecoder

[源码:29](../../../desktop/infrastructure/src/audio_preparation.rs#L29) · trait

```rust
pub trait AudioDecoder {
    // Required methods
    fn probe(&self, bytes: &[u8]) -> Result<AudioMetadata, PrepareFailure>;
    fn decode_into(
        &mut self,
        bytes: &[u8],
        metadata: AudioMetadata,
        output: &mut [f32],
        cancel: &Cancellation,
    ) -> Result<usize, PrepareFailure>;
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_infrastructure::audio_preparation::AudioDecoder`

声明引用：`skin_editor_audio::PrepareFailure`, `skin_editor_infrastructure::Cancellation`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::BlobStore::stage_audio`, `skin_editor_infrastructure::PreparationWorker::new`, `skin_editor_infrastructure::RunningPreparation::execute`

实现／约束（显式或 derive，无类继承）：

- `impl AudioDecoder for WavDecoder` — [源码:87](../../../desktop/infrastructure/src/wav.rs#L87)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn probe(&self, bytes: &[u8]) -> Result<AudioMetadata, PrepareFailure>` |  | [源码:31](../../../desktop/infrastructure/src/audio_preparation.rs#L31) | skin_editor_audio::PrepareFailure |
| `fn decode_into( &mut self, bytes: &[u8], metadata: AudioMetadata, output: &mut [f32], cancel: &Cancellation, ) -> Result<usize, PrepareFailure>` |  | [源码:33](../../../desktop/infrastructure/src/audio_preparation.rs#L33) | skin_editor_audio::PrepareFailure, skin_editor_infrastructure::Cancellation |

## skin_editor_infrastructure::AudioPlayback

[源码:9](../../../desktop/infrastructure/src/audio_playback.rs#L9) · struct

```rust
pub struct AudioPlayback {
    controller: SkinAudioController,
    worker: PreparationWorker,
    output_invalidation: u64,
    output_safety: Option<OutputSafety>,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_infrastructure::audio_playback::AudioPlayback`

声明引用：`skin_editor_infrastructure::PreparationWorker`, `skin_editor_infrastructure::SkinAudioController`, `skin_editor_infrastructure::native_audio::OutputSafety`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::audio_verification::phase (声明)`, `skin_editor_desktop::audio_verification::quiet (声明)`, `skin_editor_desktop::audio_verification::settle (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl AudioPlayback` — [源码:15](../../../desktop/infrastructure/src/audio_playback.rs#L15)
- `impl Drop for AudioPlayback` — [源码:160](../../../desktop/infrastructure/src/audio_playback.rs#L160)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn new(rate: u32, channels: u16, budget: PcmBudget) -> Result<Self, String>` | impl AudioPlayback | [源码:16](../../../desktop/infrastructure/src/audio_playback.rs#L16) | skin_editor_audio::PcmBudget |
| `fn refresh( &mut self, session: &ProjectSession, output: &OutputProducer, ) -> Result<(), String>` | impl AudioPlayback | [源码:27](../../../desktop/infrastructure/src/audio_playback.rs#L27) | skin_editor_infrastructure::OutputProducer |
| `fn fence_if_changed(&mut self, output: &OutputProducer) -> Result<(), String>` | impl AudioPlayback | [源码:45](../../../desktop/infrastructure/src/audio_playback.rs#L45) | skin_editor_infrastructure::OutputProducer |
| `pub fn step( &mut self, session: &ProjectSession, store: &BlobStore, output: &mut OutputProducer, ) -> Result<Vec<BuildAdmission>, String>` | impl AudioPlayback | [源码:54](../../../desktop/infrastructure/src/audio_playback.rs#L54) | skin_editor_infrastructure::BlobStore, skin_editor_infrastructure::BuildAdmission, skin_editor_infrastructure::OutputProducer |
| `pub fn trigger( &mut self, session: &ProjectSession, output: &OutputProducer, lane: Channel, component: AudioComponentId, ) -> Result<TriggerReceipt, String>` | impl AudioPlayback | [源码:91](../../../desktop/infrastructure/src/audio_playback.rs#L91) | skin_editor_audio::Channel, skin_editor_audio::TriggerReceipt, skin_editor_infrastructure::OutputProducer |
| `pub fn scene_running( &mut self, output: &OutputProducer, running: bool, ) -> Result<OutputFence, String>` | impl AudioPlayback | [源码:104](../../../desktop/infrastructure/src/audio_playback.rs#L104) | skin_editor_infrastructure::OutputFence, skin_editor_infrastructure::OutputProducer |
| `pub fn stop_audition( &mut self, output: &OutputProducer, ) -> Result<OutputFence, String>` | impl AudioPlayback | [源码:115](../../../desktop/infrastructure/src/audio_playback.rs#L115) | skin_editor_infrastructure::OutputFence, skin_editor_infrastructure::OutputProducer |
| `pub fn status(&mut self, component: AudioComponentId) -> Option<BindingStatus>` | impl AudioPlayback | [源码:122](../../../desktop/infrastructure/src/audio_playback.rs#L122) | skin_editor_audio::BindingStatus |
| `pub fn wait_preparation( &mut self, session: &ProjectSession, output: &OutputProducer, timeout: Duration, ) -> Result<bool, String>` | impl AudioPlayback | [源码:129](../../../desktop/infrastructure/src/audio_playback.rs#L129) | skin_editor_infrastructure::OutputProducer |
| `pub fn scene_frame(&mut self) -> u64` | impl AudioPlayback | [源码:145](../../../desktop/infrastructure/src/audio_playback.rs#L145) |  |
| `pub fn prefill(&mut self, output: &mut OutputProducer) -> Result<(), String>` | impl AudioPlayback | [源码:149](../../../desktop/infrastructure/src/audio_playback.rs#L149) | skin_editor_infrastructure::OutputProducer |
| `pub fn close(&mut self, output: &OutputProducer) -> Result<OutputFence, String>` | impl AudioPlayback | [源码:153](../../../desktop/infrastructure/src/audio_playback.rs#L153) | skin_editor_infrastructure::OutputFence, skin_editor_infrastructure::OutputProducer |
| `fn drop(&mut self)` | impl Drop for AudioPlayback | [源码:161](../../../desktop/infrastructure/src/audio_playback.rs#L161) |  |

## skin_editor_infrastructure::AudioPreparationQueue

[源码:173](../../../desktop/infrastructure/src/audio_preparation.rs#L173) · struct

```rust
pub struct AudioPreparationQueue {
    budget: PcmBudget,
    pending: VecDeque<Pending>,
    active: Option<Active>,
    cache: BTreeMap<PreparedKey, PreparedWeak>,
    next_id: u64,
    closed: bool,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_infrastructure::audio_preparation::AudioPreparationQueue`

声明引用：`skin_editor_audio::PcmBudget`, `skin_editor_audio::PreparedKey`, `skin_editor_audio::PreparedWeak`, `skin_editor_infrastructure::audio_preparation::Active`, `skin_editor_infrastructure::audio_preparation::Pending`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::SkinAudioController (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl AudioPreparationQueue` — [源码:181](../../../desktop/infrastructure/src/audio_preparation.rs#L181)
- `impl Drop for AudioPreparationQueue` — [源码:371](../../../desktop/infrastructure/src/audio_preparation.rs#L371)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn new(budget: PcmBudget) -> Self` | impl AudioPreparationQueue | [源码:182](../../../desktop/infrastructure/src/audio_preparation.rs#L182) | skin_editor_audio::PcmBudget |
| `fn prune(&mut self)` | impl AudioPreparationQueue | [源码:192](../../../desktop/infrastructure/src/audio_preparation.rs#L192) |  |
| `pub fn usage(&mut self) -> PreparationUsage` | impl AudioPreparationQueue | [源码:202](../../../desktop/infrastructure/src/audio_preparation.rs#L202) | skin_editor_infrastructure::PreparationUsage |
| `pub fn submit( &mut self, request: BuildRequest, raw: RawBlob, ) -> Result<Admission, PreparationError>` | impl AudioPreparationQueue | [源码:211](../../../desktop/infrastructure/src/audio_preparation.rs#L211) | skin_editor_audio::BuildRequest, skin_editor_infrastructure::Admission, skin_editor_infrastructure::PreparationError, skin_editor_infrastructure::RawBlob |
| `pub fn start_next(&mut self) -> Option<RunningPreparation>` | impl AudioPreparationQueue | [源码:272](../../../desktop/infrastructure/src/audio_preparation.rs#L272) | skin_editor_infrastructure::RunningPreparation |
| `pub fn finish( &mut self, finished: FinishedPreparation, ) -> Result<(BuildRequest, Result<PreparedSound, PrepareFailure>), PreparationError>` | impl AudioPreparationQueue | [源码:291](../../../desktop/infrastructure/src/audio_preparation.rs#L291) | skin_editor_audio::BuildRequest, skin_editor_audio::PrepareFailure, skin_editor_audio::PreparedSound, skin_editor_infrastructure::FinishedPreparation, skin_editor_infrastructure::PreparationError |
| `pub fn cancel_component(&mut self, component: impl Into<SoundTarget>)` | impl AudioPreparationQueue | [源码:316](../../../desktop/infrastructure/src/audio_preparation.rs#L316) | skin_editor_audio::SoundTarget |
| `pub fn cancel_target(&mut self, target: SoundTarget)` | impl AudioPreparationQueue | [源码:319](../../../desktop/infrastructure/src/audio_preparation.rs#L319) | skin_editor_audio::SoundTarget |
| `pub fn cancel_library(&mut self)` | impl AudioPreparationQueue | [源码:330](../../../desktop/infrastructure/src/audio_preparation.rs#L330) |  |
| `pub fn cancel_outdated(&mut self, origin: Origin)` | impl AudioPreparationQueue | [源码:351](../../../desktop/infrastructure/src/audio_preparation.rs#L351) | skin_editor_audio::Origin |
| `pub fn close(&mut self)` | impl AudioPreparationQueue | [源码:362](../../../desktop/infrastructure/src/audio_preparation.rs#L362) |  |
| `fn drop(&mut self)` | impl Drop for AudioPreparationQueue | [源码:372](../../../desktop/infrastructure/src/audio_preparation.rs#L372) |  |

## skin_editor_infrastructure::AudioSupplyClient

[源码:136](../../../desktop/infrastructure/src/audio_supply.rs#L136) · struct

```rust
pub struct AudioSupplyClient {
    sender: SyncSender<Request>,
    receiver: Receiver<SupplyReceipt>,
    credits: Arc<AtomicUsize>,
    status: Arc<Mutex<SupplyStatus>>,
    output: SupplyOutputControl,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<Option<RetiredSupply>>>,
    retired: Option<RetiredSupply>,
    next: u64,
    pending: BTreeMap<u64, Option<DraftId>>,
    last: Option<SkinAudioSource>,
    audition_edits: AuditionEdits,
    scene_goal: bool,
    library_active: bool,
    draft_audition: Option<DraftId>,
    reported_retained: Vec<DraftId>,
    reported_revision: u64,
    background_frames: Option<u64>,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_infrastructure::audio_supply::AudioSupplyClient`

声明引用：`skin_editor_audio::AuditionEdits`, `skin_editor_infrastructure::SupplyReceipt`, `skin_editor_infrastructure::SupplyStatus`, `skin_editor_infrastructure::audio_supply::Request`, `skin_editor_infrastructure::audio_supply::RetiredSupply`, `skin_editor_infrastructure::supply_output::SupplyOutputControl`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::combined_verification::Combined (声明)`, `skin_editor_desktop::combined_verification::cleared (声明)`, `skin_editor_desktop::combined_verification::play_old (声明)`, `skin_editor_desktop::combined_verification::ready_all (声明)`, `skin_editor_desktop::edit_verification::health (声明)`, `skin_editor_desktop::edit_verification::prepare (声明)`, `skin_editor_desktop::edit_verification::ready (声明)`, `skin_editor_desktop::edit_verification::trigger (声明)`, `skin_editor_desktop::editor_audio::EditorAudio (声明)`, `skin_editor_desktop::load_verification::control (声明)`, `skin_editor_desktop::load_verification::outcome (声明)`, `skin_editor_desktop::load_verification::stop (声明)`, `skin_editor_desktop::supply_verification::outcome (声明)`, `skin_editor_desktop::supply_verification::ready (声明)`, `skin_editor_desktop::supply_verification::trigger (声明)`, `skin_editor_desktop::worker_verification::barrier (声明)`, `skin_editor_desktop::worker_verification::healthy (声明)`, `skin_editor_desktop::worker_verification::location_visible (声明)`, `skin_editor_desktop::worker_verification::outcome (声明)`, `skin_editor_desktop::worker_verification::prime (声明)`, `skin_editor_desktop::worker_verification::ready (声明)`, `skin_editor_desktop::worker_verification::receipt (声明)`, `skin_editor_desktop::worker_verification::stop (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl AudioSupplyClient` — [源码:212](../../../desktop/infrastructure/src/audio_supply.rs#L212)
- `impl Drop for AudioSupplyClient` — [源码:651](../../../desktop/infrastructure/src/audio_supply.rs#L651)
- `impl SkinAudioPort for AudioSupplyClient` — [源码:656](../../../desktop/infrastructure/src/audio_supply.rs#L656)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn diagnostic_recovery_probe( &mut self, trigger: Option<(Channel, AudioComponentId)>, ) -> Result<u64, SupplyError>` | impl AudioSupplyClient | [源码:215](../../../desktop/infrastructure/src/audio_supply.rs#L215) | skin_editor_audio::Channel, skin_editor_infrastructure::SupplyError |
| `pub fn barrier(&mut self) -> Result<u64, SupplyError>` | impl AudioSupplyClient | [源码:235](../../../desktop/infrastructure/src/audio_supply.rs#L235) | skin_editor_infrastructure::SupplyError |
| `pub fn spawn( output: OutputProducer, rate: u32, channels: u16, budget: PcmBudget, store: BlobStore, ) -> Result<Self, SupplyError>` | impl AudioSupplyClient | [源码:239](../../../desktop/infrastructure/src/audio_supply.rs#L239) | skin_editor_audio::PcmBudget, skin_editor_infrastructure::BlobStore, skin_editor_infrastructure::OutputProducer, skin_editor_infrastructure::SupplyError |
| `pub fn spawn_separated( background: BackgroundTrack, rate: u32, channels: u16, budget: PcmBudget, store: BlobStore, ) -> Result<(Self, Box<OutputConsumer>), SupplyError>` | impl AudioSupplyClient | [源码:259](../../../desktop/infrastructure/src/audio_supply.rs#L259) | skin_editor_audio::BackgroundTrack, skin_editor_audio::PcmBudget, skin_editor_infrastructure::BlobStore, skin_editor_infrastructure::SupplyError, skin_editor_infrastructure::separated_output::OutputConsumer |
| `pub fn spawn_streamed( background: BufferedBackground, rate: u32, channels: u16, budget: PcmBudget, store: BlobStore, ) -> Result<(Self, Box<OutputConsumer>), SupplyError>` | impl AudioSupplyClient | [源码:271](../../../desktop/infrastructure/src/audio_supply.rs#L271) | skin_editor_audio::PcmBudget, skin_editor_infrastructure::BlobStore, skin_editor_infrastructure::SupplyError, skin_editor_infrastructure::background_stream::BufferedBackground, skin_editor_infrastructure::separated_output::OutputConsumer |
| `fn spawn_owned( output: SupplyOutput, core: SupplyCore, store: BlobStore, ) -> Result<Self, SupplyError>` | impl AudioSupplyClient | [源码:283](../../../desktop/infrastructure/src/audio_supply.rs#L283) | skin_editor_infrastructure::BlobStore, skin_editor_infrastructure::SupplyError, skin_editor_infrastructure::audio_supply::SupplyCore, skin_editor_infrastructure::audio_supply::SupplyOutput |
| `fn reserve(&self) -> Result<Credit, SupplyError>` | impl AudioSupplyClient | [源码:425](../../../desktop/infrastructure/src/audio_supply.rs#L425) | skin_editor_infrastructure::SupplyError, skin_editor_infrastructure::audio_supply::Credit |
| `fn send( &mut self, control: Control, credit: Credit, draft: Option<DraftId>, ) -> Result<u64, SupplyError>` | impl AudioSupplyClient | [源码:436](../../../desktop/infrastructure/src/audio_supply.rs#L436) | skin_editor_infrastructure::SupplyError, skin_editor_infrastructure::audio_supply::Control, skin_editor_infrastructure::audio_supply::Credit |
| `pub fn synchronize( &mut self, session: &ProjectSession, ) -> Result<(), SkinAudioError>` | impl AudioSupplyClient | [源码:460](../../../desktop/infrastructure/src/audio_supply.rs#L460) |  |
| `pub fn set_background( &mut self, background: BackgroundTrack, ) -> Result<u64, SupplyError>` | impl AudioSupplyClient | [源码:463](../../../desktop/infrastructure/src/audio_supply.rs#L463) | skin_editor_audio::BackgroundTrack, skin_editor_infrastructure::SupplyError |
| `pub fn scene_running(&mut self, running: bool) -> Result<u64, SupplyError>` | impl AudioSupplyClient | [源码:467](../../../desktop/infrastructure/src/audio_supply.rs#L467) | skin_editor_infrastructure::SupplyError |
| `pub fn seek_background(&mut self, frame: u64) -> Result<u64, SupplyError>` | impl AudioSupplyClient | [源码:480](../../../desktop/infrastructure/src/audio_supply.rs#L480) | skin_editor_infrastructure::SupplyError |
| `pub fn audition_asset( &mut self, request: AssetAudition, ) -> Result<u64, SupplyError>` | impl AudioSupplyClient | [源码:491](../../../desktop/infrastructure/src/audio_supply.rs#L491) | skin_editor_infrastructure::SupplyError |
| `pub fn stop_audition(&mut self) -> Result<u64, SupplyError>` | impl AudioSupplyClient | [源码:516](../../../desktop/infrastructure/src/audio_supply.rs#L516) | skin_editor_infrastructure::SupplyError |
| `pub fn retry_selected( &mut self, component: AudioComponentId, ) -> Result<u64, SupplyError>` | impl AudioSupplyClient | [源码:522](../../../desktop/infrastructure/src/audio_supply.rs#L522) | skin_editor_infrastructure::SupplyError |
| `pub fn trigger( &mut self, session: &ProjectSession, lane: Channel, component: AudioComponentId, ) -> Result<u64, SupplyError>` | impl AudioSupplyClient | [源码:526](../../../desktop/infrastructure/src/audio_supply.rs#L526) | skin_editor_audio::Channel, skin_editor_infrastructure::SupplyError |
| `pub fn trigger_selected( &mut self, lane: Channel, component: AudioComponentId, ) -> Result<u64, SupplyError>` | impl AudioSupplyClient | [源码:536](../../../desktop/infrastructure/src/audio_supply.rs#L536) | skin_editor_audio::Channel, skin_editor_infrastructure::SupplyError |
| `pub fn trigger_batch_selected( &mut self, entries: &[(Channel, AudioComponentId)], ) -> Result<u64, SupplyError>` | impl AudioSupplyClient | [源码:566](../../../desktop/infrastructure/src/audio_supply.rs#L566) | skin_editor_audio::Channel, skin_editor_infrastructure::SupplyError |
| `pub fn poll(&mut self) -> Option<SupplyReceipt>` | impl AudioSupplyClient | [源码:596](../../../desktop/infrastructure/src/audio_supply.rs#L596) | skin_editor_infrastructure::SupplyReceipt |
| `pub fn wait_receipt( &mut self, timeout: Duration, ) -> Result<Option<SupplyReceipt>, SupplyError>` | impl AudioSupplyClient | [源码:601](../../../desktop/infrastructure/src/audio_supply.rs#L601) | skin_editor_infrastructure::SupplyError, skin_editor_infrastructure::SupplyReceipt |
| `fn accept_receipt(&mut self, receipt: SupplyReceipt) -> SupplyReceipt` | impl AudioSupplyClient | [源码:611](../../../desktop/infrastructure/src/audio_supply.rs#L611) | skin_editor_infrastructure::SupplyReceipt |
| `pub fn status(&self) -> Result<SupplyStatus, SupplyError>` | impl AudioSupplyClient | [源码:617](../../../desktop/infrastructure/src/audio_supply.rs#L617) | skin_editor_infrastructure::SupplyError, skin_editor_infrastructure::SupplyStatus |
| `pub fn pending_controls(&self) -> bool` | impl AudioSupplyClient | [源码:620](../../../desktop/infrastructure/src/audio_supply.rs#L620) |  |
| `pub fn output(&self) -> Result<OutputControl, SupplyError>` | impl AudioSupplyClient | [源码:623](../../../desktop/infrastructure/src/audio_supply.rs#L623) | skin_editor_infrastructure::OutputControl, skin_editor_infrastructure::SupplyError |
| `pub fn close(&mut self)` | impl AudioSupplyClient | [源码:626](../../../desktop/infrastructure/src/audio_supply.rs#L626) |  |
| `pub fn finish_separated( &mut self, stopped: StoppedOutput, ) -> Result<(), StoppedOutput>` | impl AudioSupplyClient | [源码:637](../../../desktop/infrastructure/src/audio_supply.rs#L637) | skin_editor_infrastructure::separated_output::StoppedOutput |
| `fn drop(&mut self)` | impl Drop for AudioSupplyClient | [源码:652](../../../desktop/infrastructure/src/audio_supply.rs#L652) |  |
| `fn retained_drafts(&self) -> Vec<DraftId>` | impl SkinAudioPort for AudioSupplyClient | [源码:657](../../../desktop/infrastructure/src/audio_supply.rs#L657) |  |
| `fn synchronize( &mut self, project: &SkinProject, source: SkinAudioSource, cancelled: &[DraftId], ) -> Result<(), SkinAudioError>` | impl SkinAudioPort for AudioSupplyClient | [源码:675](../../../desktop/infrastructure/src/audio_supply.rs#L675) |  |

## skin_editor_infrastructure::BlobError

[源码:13](../../../desktop/infrastructure/src/blobs.rs#L13) · enum

```rust
pub enum BlobError {
    ResourceLimit,
    Unavailable,
    Io,
    SourceChanged,
    HashCollision,
    MetadataMismatch,
    Unsupported,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_infrastructure::blobs::BlobError`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::BlobError::clone`, `skin_editor_infrastructure::BlobError::eq`, `skin_editor_infrastructure::BlobStore::insert`, `skin_editor_infrastructure::BlobStore::stage_audio`, `skin_editor_infrastructure::BlobStore::stage_bytes`, `skin_editor_infrastructure::BlobStore::stage_original_file`, `skin_editor_infrastructure::BlobStore::stage_reader`, `skin_editor_infrastructure::BlobStore::stage_wav`, `skin_editor_infrastructure::BlobStore::usage`, `skin_editor_infrastructure::BlobStore::with_limits`, `skin_editor_infrastructure::RawBlob::attest_audio`, `skin_editor_infrastructure::blob_import (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for BlobError` — [源码:12](../../../desktop/infrastructure/src/blobs.rs#L12)
- `impl Debug for BlobError` — [源码:12](../../../desktop/infrastructure/src/blobs.rs#L12)
- `impl PartialEq for BlobError` — [源码:12](../../../desktop/infrastructure/src/blobs.rs#L12)
- `impl Copy for BlobError` — [源码:12](../../../desktop/infrastructure/src/blobs.rs#L12)
- `impl Eq for BlobError` — [源码:12](../../../desktop/infrastructure/src/blobs.rs#L12)
- `impl StructuralPartialEq for BlobError` — [源码:12](../../../desktop/infrastructure/src/blobs.rs#L12)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> BlobError` | impl Clone for BlobError | [源码:12](../../../desktop/infrastructure/src/blobs.rs#L12) | skin_editor_infrastructure::BlobError |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for BlobError | [源码:12](../../../desktop/infrastructure/src/blobs.rs#L12) |  |
| `fn eq(&self, other: &BlobError) -> bool` | impl PartialEq for BlobError | [源码:12](../../../desktop/infrastructure/src/blobs.rs#L12) | skin_editor_infrastructure::BlobError |

## skin_editor_infrastructure::BlobStore

[源码:67](../../../desktop/infrastructure/src/blobs.rs#L67) · struct

```rust
pub struct BlobStore {
    entries: Arc<Mutex<BTreeMap<BlobRef, Weak<Entry>>>>,
    budget: Arc<ByteBudget>,
    per_blob: usize,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_infrastructure::blobs::BlobStore`

声明引用：`skin_editor_infrastructure::blobs::Entry`, `skin_editor_infrastructure::byte_budget::ByteBudget`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::audio_verification::phase (声明)`, `skin_editor_desktop::audio_verification::quiet (声明)`, `skin_editor_desktop::audio_verification::settle (声明)`, `skin_editor_desktop::editor_audio::EditorAudio (声明)`, `skin_editor_desktop::editor_audio::EditorAudio::candidate`, `skin_editor_desktop::editor_audio::EditorAudio::new`, `skin_editor_desktop::product_policy::Policy::audio`, `skin_editor_infrastructure::AudioPlayback::step`, `skin_editor_infrastructure::AudioSupplyClient::spawn`, `skin_editor_infrastructure::AudioSupplyClient::spawn_owned`, `skin_editor_infrastructure::AudioSupplyClient::spawn_separated`, `skin_editor_infrastructure::AudioSupplyClient::spawn_streamed`, `skin_editor_infrastructure::BlobStore::clone`, `skin_editor_infrastructure::Images (声明)`, `skin_editor_infrastructure::Images::blobs`, `skin_editor_infrastructure::Images::with_store`, `skin_editor_infrastructure::SkinAudioController::admit_builds`, `skin_editor_infrastructure::SkinAudioController::admit_selected`, `skin_editor_infrastructure::SkinAudioController::request_library`, `skin_editor_infrastructure::audio_supply::SupplyCore::apply`, `skin_editor_infrastructure::audio_supply::SupplyCore::apply_output`, `skin_editor_infrastructure::audio_supply::SupplyCore::apply_owned`

实现／约束（显式或 derive，无类继承）：

- `impl BlobStore` — [源码:41](../../../desktop/infrastructure/src/audio_preparation.rs#L41)
- `impl BlobStore` — [源码:111](../../../desktop/infrastructure/src/wav.rs#L111)
- `impl BlobStore` — [源码:77](../../../desktop/infrastructure/src/blobs.rs#L77)
- `impl Clone for BlobStore` — [源码:66](../../../desktop/infrastructure/src/blobs.rs#L66)
- `impl Default for BlobStore` — [源码:72](../../../desktop/infrastructure/src/blobs.rs#L72)
- `impl ResourceStore for BlobStore` — [源码:203](../../../desktop/infrastructure/src/blobs.rs#L203)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn stage_audio( &self, bytes: Vec<u8>, decoder: &dyn AudioDecoder, ) -> Result<RawBlob, BlobError>` | impl BlobStore | [源码:44](../../../desktop/infrastructure/src/audio_preparation.rs#L44) | skin_editor_infrastructure::AudioDecoder, skin_editor_infrastructure::BlobError, skin_editor_infrastructure::RawBlob |
| `pub fn stage_wav(&self, path: &Path) -> Result<RawBlob, BlobError>` | impl BlobStore | [源码:113](../../../desktop/infrastructure/src/wav.rs#L113) | skin_editor_infrastructure::BlobError, skin_editor_infrastructure::RawBlob |
| `pub fn observe_bytes(&self) -> ByteObservation` | impl BlobStore | [源码:78](../../../desktop/infrastructure/src/blobs.rs#L78) | skin_editor_infrastructure::ByteObservation |
| `pub fn stage_original_file(&self, path: &Path) -> Result<RawBlob, BlobError>` | impl BlobStore | [源码:82](../../../desktop/infrastructure/src/blobs.rs#L82) | skin_editor_infrastructure::BlobError, skin_editor_infrastructure::RawBlob |
| `pub fn with_limits(total: usize, per_blob: usize) -> Result<Self, BlobError>` | impl BlobStore | [源码:87](../../../desktop/infrastructure/src/blobs.rs#L87) | skin_editor_infrastructure::BlobError |
| `pub fn get(&self, reference: BlobRef) -> Option<RawBlob>` | impl BlobStore | [源码:97](../../../desktop/infrastructure/src/blobs.rs#L97) | skin_editor_infrastructure::RawBlob |
| `pub fn usage(&self) -> Result<BlobUsage, BlobError>` | impl BlobStore | [源码:105](../../../desktop/infrastructure/src/blobs.rs#L105) | skin_editor_infrastructure::BlobError, skin_editor_infrastructure::BlobUsage |
| `fn insert( &self, bytes: Vec<u8>, reference: BlobRef, lease: ByteLease, ) -> Result<RawBlob, BlobError>` | impl BlobStore | [源码:114](../../../desktop/infrastructure/src/blobs.rs#L114) | skin_editor_infrastructure::BlobError, skin_editor_infrastructure::RawBlob, skin_editor_infrastructure::byte_budget::ByteLease |
| `pub fn stage_bytes(&self, bytes: Vec<u8>) -> Result<RawBlob, BlobError>` | impl BlobStore | [源码:143](../../../desktop/infrastructure/src/blobs.rs#L143) | skin_editor_infrastructure::BlobError, skin_editor_infrastructure::RawBlob |
| `pub fn stage_reader( &self, reader: impl Read + Seek, byte_len: u64, ) -> Result<RawBlob, BlobError>` | impl BlobStore | [源码:162](../../../desktop/infrastructure/src/blobs.rs#L162) | skin_editor_infrastructure::BlobError, skin_editor_infrastructure::RawBlob |
| `fn clone(&self) -> BlobStore` | impl Clone for BlobStore | [源码:66](../../../desktop/infrastructure/src/blobs.rs#L66) | skin_editor_infrastructure::BlobStore |
| `fn default() -> Self` | impl Default for BlobStore | [源码:73](../../../desktop/infrastructure/src/blobs.rs#L73) |  |
| `fn pin(&self, blob: BlobRef) -> Result<Arc<dyn BlobLease>, ResourceError>` | impl ResourceStore for BlobStore | [源码:204](../../../desktop/infrastructure/src/blobs.rs#L204) |  |
| `fn audio_metadata(&self, blob: BlobRef) -> Option<AudioMetadata>` | impl ResourceStore for BlobStore | [源码:209](../../../desktop/infrastructure/src/blobs.rs#L209) |  |

## skin_editor_infrastructure::BlobUsage

[源码:23](../../../desktop/infrastructure/src/blobs.rs#L23) · struct

```rust
pub struct BlobUsage {
    pub bytes: usize,
    pub peak_bytes: usize,
    pub entries: usize,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_infrastructure::blobs::BlobUsage`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::BlobStore::usage`, `skin_editor_infrastructure::BlobUsage::clone`, `skin_editor_infrastructure::BlobUsage::eq`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for BlobUsage` — [源码:22](../../../desktop/infrastructure/src/blobs.rs#L22)
- `impl Debug for BlobUsage` — [源码:22](../../../desktop/infrastructure/src/blobs.rs#L22)
- `impl PartialEq for BlobUsage` — [源码:22](../../../desktop/infrastructure/src/blobs.rs#L22)
- `impl Copy for BlobUsage` — [源码:22](../../../desktop/infrastructure/src/blobs.rs#L22)
- `impl Eq for BlobUsage` — [源码:22](../../../desktop/infrastructure/src/blobs.rs#L22)
- `impl StructuralPartialEq for BlobUsage` — [源码:22](../../../desktop/infrastructure/src/blobs.rs#L22)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> BlobUsage` | impl Clone for BlobUsage | [源码:22](../../../desktop/infrastructure/src/blobs.rs#L22) | skin_editor_infrastructure::BlobUsage |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for BlobUsage | [源码:22](../../../desktop/infrastructure/src/blobs.rs#L22) |  |
| `fn eq(&self, other: &BlobUsage) -> bool` | impl PartialEq for BlobUsage | [源码:22](../../../desktop/infrastructure/src/blobs.rs#L22) | skin_editor_infrastructure::BlobUsage |

## skin_editor_infrastructure::BuildAdmission

[源码:259](../../../desktop/infrastructure/src/skin_audio.rs#L259) · struct

```rust
pub struct BuildAdmission {
    pub request: BuildRequest,
    pub status: Result<AdmissionState, PreparationError>,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_infrastructure::skin_audio::BuildAdmission`

声明引用：`skin_editor_audio::BuildRequest`, `skin_editor_infrastructure::AdmissionState`, `skin_editor_infrastructure::PreparationError`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::AudioPlayback::step`, `skin_editor_infrastructure::BuildAdmission::clone`, `skin_editor_infrastructure::BuildAdmission::eq`, `skin_editor_infrastructure::SkinAudioController::admit_builds`, `skin_editor_infrastructure::SkinAudioController::admit_selected`, `skin_editor_infrastructure::SupplyOutcome (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for BuildAdmission` — [源码:258](../../../desktop/infrastructure/src/skin_audio.rs#L258)
- `impl Debug for BuildAdmission` — [源码:258](../../../desktop/infrastructure/src/skin_audio.rs#L258)
- `impl PartialEq for BuildAdmission` — [源码:258](../../../desktop/infrastructure/src/skin_audio.rs#L258)
- `impl Copy for BuildAdmission` — [源码:258](../../../desktop/infrastructure/src/skin_audio.rs#L258)
- `impl StructuralPartialEq for BuildAdmission` — [源码:258](../../../desktop/infrastructure/src/skin_audio.rs#L258)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> BuildAdmission` | impl Clone for BuildAdmission | [源码:258](../../../desktop/infrastructure/src/skin_audio.rs#L258) | skin_editor_infrastructure::BuildAdmission |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for BuildAdmission | [源码:258](../../../desktop/infrastructure/src/skin_audio.rs#L258) |  |
| `fn eq(&self, other: &BuildAdmission) -> bool` | impl PartialEq for BuildAdmission | [源码:258](../../../desktop/infrastructure/src/skin_audio.rs#L258) | skin_editor_infrastructure::BuildAdmission |

## skin_editor_infrastructure::ByteObservation

[源码:13](../../../desktop/infrastructure/src/byte_budget.rs#L13) · struct

```rust
pub struct ByteObservation(Arc<ByteBudget>);
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_infrastructure::byte_budget::ByteObservation`

声明引用：`skin_editor_infrastructure::byte_budget::ByteBudget`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::editor_audio::cycles::Counters (声明)`, `skin_editor_infrastructure::BlobStore::observe_bytes`, `skin_editor_infrastructure::ByteObservation::clone`, `skin_editor_infrastructure::Images::observe_rgba`, `skin_editor_infrastructure::byte_budget::ByteBudget::observation`

实现／约束（显式或 derive，无类继承）：

- `impl ByteObservation` — [源码:19](../../../desktop/infrastructure/src/byte_budget.rs#L19)
- `impl Clone for ByteObservation` — [源码:12](../../../desktop/infrastructure/src/byte_budget.rs#L12)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn usage(&self) -> ByteUsage` | impl ByteObservation | [源码:20](../../../desktop/infrastructure/src/byte_budget.rs#L20) | skin_editor_infrastructure::ByteUsage |
| `fn clone(&self) -> ByteObservation` | impl Clone for ByteObservation | [源码:12](../../../desktop/infrastructure/src/byte_budget.rs#L12) | skin_editor_infrastructure::ByteObservation |

## skin_editor_infrastructure::ByteUsage

[源码:15](../../../desktop/infrastructure/src/byte_budget.rs#L15) · struct

```rust
pub struct ByteUsage {
    pub bytes: usize,
    pub peak_bytes: usize,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_infrastructure::byte_budget::ByteUsage`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::ByteObservation::usage`, `skin_editor_infrastructure::ByteUsage::clone`, `skin_editor_infrastructure::ByteUsage::eq`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for ByteUsage` — [源码:14](../../../desktop/infrastructure/src/byte_budget.rs#L14)
- `impl Debug for ByteUsage` — [源码:14](../../../desktop/infrastructure/src/byte_budget.rs#L14)
- `impl PartialEq for ByteUsage` — [源码:14](../../../desktop/infrastructure/src/byte_budget.rs#L14)
- `impl Copy for ByteUsage` — [源码:14](../../../desktop/infrastructure/src/byte_budget.rs#L14)
- `impl Eq for ByteUsage` — [源码:14](../../../desktop/infrastructure/src/byte_budget.rs#L14)
- `impl StructuralPartialEq for ByteUsage` — [源码:14](../../../desktop/infrastructure/src/byte_budget.rs#L14)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> ByteUsage` | impl Clone for ByteUsage | [源码:14](../../../desktop/infrastructure/src/byte_budget.rs#L14) | skin_editor_infrastructure::ByteUsage |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for ByteUsage | [源码:14](../../../desktop/infrastructure/src/byte_budget.rs#L14) |  |
| `fn eq(&self, other: &ByteUsage) -> bool` | impl PartialEq for ByteUsage | [源码:14](../../../desktop/infrastructure/src/byte_budget.rs#L14) | skin_editor_infrastructure::ByteUsage |

## skin_editor_infrastructure::CachedRaster

[源码:52](../../../desktop/infrastructure/src/lib.rs#L52) · struct

```rust
pub(crate) struct CachedRaster {
    pub(crate) raster: Arc<Raster>,
}
```

声明引用：`skin_editor_rendering::Raster`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::Images (声明)`, `skin_editor_infrastructure::Images::prune`

## skin_editor_infrastructure::Cancellation

[源码:60](../../../desktop/infrastructure/src/audio_preparation.rs#L60) · struct

```rust
pub struct Cancellation(Arc<JobLife>);
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_infrastructure::audio_preparation::Cancellation`

声明引用：`skin_editor_infrastructure::audio_preparation::JobLife`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::AudioDecoder (声明)`, `skin_editor_infrastructure::AudioDecoder::decode_into`, `skin_editor_infrastructure::WavDecoder::decode_into`

实现／约束（显式或 derive，无类继承）：

- `impl Cancellation` — [源码:61](../../../desktop/infrastructure/src/audio_preparation.rs#L61)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn is_cancelled(&self) -> bool` | impl Cancellation | [源码:62](../../../desktop/infrastructure/src/audio_preparation.rs#L62) |  |

## skin_editor_infrastructure::FinishedPreparation

[源码:82](../../../desktop/infrastructure/src/audio_preparation.rs#L82) · struct

```rust
pub struct FinishedPreparation {
    id: u64,
    request: BuildRequest,
    result: Result<PreparedSound, PrepareFailure>,
    life: Arc<JobLife>,
    observation: PreparationObservation,
    finished: Instant,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_infrastructure::audio_preparation::FinishedPreparation`

声明引用：`skin_editor_audio::BuildRequest`, `skin_editor_audio::PrepareFailure`, `skin_editor_audio::PreparedSound`, `skin_editor_infrastructure::PreparationObservation`, `skin_editor_infrastructure::audio_preparation::JobLife`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::AudioPreparationQueue::finish`, `skin_editor_infrastructure::PreparationWorker (声明)`, `skin_editor_infrastructure::PreparationWorker::try_result`, `skin_editor_infrastructure::PreparationWorker::wait_result`, `skin_editor_infrastructure::RunningPreparation::execute`, `skin_editor_infrastructure::SkinAudioController::finish_preparation`, `skin_editor_infrastructure::SkinAudioController::finish_selected`

实现／约束（显式或 derive，无类继承）：

- `impl FinishedPreparation` — [源码:99](../../../desktop/infrastructure/src/audio_preparation.rs#L99)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn observation(&self) -> PreparationObservation` | impl FinishedPreparation | [源码:100](../../../desktop/infrastructure/src/audio_preparation.rs#L100) | skin_editor_infrastructure::PreparationObservation |

## skin_editor_infrastructure::Images

[源码:56](../../../desktop/infrastructure/src/lib.rs#L56) · struct

```rust
pub struct Images {
    pub(crate) blobs: BlobStore,
    pub(crate) entries: Arc<Mutex<BTreeMap<BlobRef, CachedRaster>>>,
    pub(crate) rgba: Arc<ByteBudget>,
    pub(crate) cache_target: usize,
}
```

声明引用：`skin_editor_infrastructure::BlobStore`, `skin_editor_infrastructure::CachedRaster`, `skin_editor_infrastructure::byte_budget::ByteBudget`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::Editor (声明)`, `skin_editor_desktop::editor_audio::window_recovery::Seed (声明)`, `skin_editor_desktop::frame_worker::decode (声明)`, `skin_editor_desktop::fresh_session (声明)`, `skin_editor_desktop::import_worker::ImportWorker::new`, `skin_editor_desktop::import_worker::stage (声明)`, `skin_editor_desktop::presentation::NativeHost (声明)`, `skin_editor_desktop::presentation::NativeHost::with_upload`, `skin_editor_desktop::presentation::RebuildSeed (声明)`, `skin_editor_desktop::sample (声明)`, `skin_editor_desktop::verification::cpu_pixels (声明)`, `skin_editor_desktop::verification::verify_source_buffer_reuse (声明)`, `skin_editor_infrastructure::Images::clone`

实现／约束（显式或 derive，无类继承）：

- `impl Images` — [源码:81](../../../desktop/infrastructure/src/lib.rs#L81)
- `impl Clone for Images` — [源码:55](../../../desktop/infrastructure/src/lib.rs#L55)
- `impl Default for Images` — [源码:62](../../../desktop/infrastructure/src/lib.rs#L62)
- `impl RasterImportPort for Images` — [源码:257](../../../desktop/infrastructure/src/lib.rs#L257)
- `impl ResourceStore for Images` — [源码:67](../../../desktop/infrastructure/src/lib.rs#L67)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn observe_rgba(&self) -> ByteObservation` | impl Images | [源码:82](../../../desktop/infrastructure/src/lib.rs#L82) | skin_editor_infrastructure::ByteObservation |
| `pub fn with_store(blobs: BlobStore) -> Self` | impl Images | [源码:85](../../../desktop/infrastructure/src/lib.rs#L85) | skin_editor_infrastructure::BlobStore |
| `pub fn blobs(&self) -> &BlobStore` | impl Images | [源码:93](../../../desktop/infrastructure/src/lib.rs#L93) | skin_editor_infrastructure::BlobStore |
| `pub(crate) fn prune(&self, entries: &mut BTreeMap<BlobRef, CachedRaster>)` | impl Images | [源码:96](../../../desktop/infrastructure/src/lib.rs#L96) | skin_editor_infrastructure::CachedRaster |
| `pub fn raster(&self, blob: BlobRef) -> Option<Arc<Raster>>` | impl Images | [源码:103](../../../desktop/infrastructure/src/lib.rs#L103) | skin_editor_rendering::Raster |
| `pub fn rgba_bytes(&self) -> usize` | impl Images | [源码:108](../../../desktop/infrastructure/src/lib.rs#L108) |  |
| `pub fn rgba_peak_bytes(&self) -> usize` | impl Images | [源码:114](../../../desktop/infrastructure/src/lib.rs#L114) |  |
| `pub fn load_raster(&self, blob: BlobRef) -> Result<Arc<Raster>, ImportError>` | impl Images | [源码:120](../../../desktop/infrastructure/src/lib.rs#L120) | skin_editor_rendering::Raster |
| `pub(crate) fn reserve_pixels( &self, bytes: usize, ) -> Result<(ByteLease, ByteLease), ImportError>` | impl Images | [源码:128](../../../desktop/infrastructure/src/lib.rs#L128) | skin_editor_infrastructure::byte_budget::ByteLease |
| `pub fn stage_bytes( &mut self, bytes: Vec<u8>, label: String, ) -> Result<StagedRaster, ImportError>` | impl Images | [源码:164](../../../desktop/infrastructure/src/lib.rs#L164) |  |
| `pub(crate) fn prepare( &self, raw: RawBlob, label: String, ) -> Result<StagedRaster, ImportError>` | impl Images | [源码:172](../../../desktop/infrastructure/src/lib.rs#L172) | skin_editor_infrastructure::RawBlob |
| `pub fn sample_png() -> Vec<u8> ⓘ` | impl Images | [源码:218](../../../desktop/infrastructure/src/lib.rs#L218) |  |
| `fn clone(&self) -> Images` | impl Clone for Images | [源码:55](../../../desktop/infrastructure/src/lib.rs#L55) | skin_editor_infrastructure::Images |
| `fn default() -> Self` | impl Default for Images | [源码:63](../../../desktop/infrastructure/src/lib.rs#L63) |  |
| `fn stage_png(&mut self, source: &str) -> Result<StagedRaster, ImportError>` | impl RasterImportPort for Images | [源码:258](../../../desktop/infrastructure/src/lib.rs#L258) |  |
| `fn pin(&self, blob: BlobRef) -> Result<Arc<dyn BlobLease>, ResourceError>` | impl ResourceStore for Images | [源码:68](../../../desktop/infrastructure/src/lib.rs#L68) |  |
| `fn audio_metadata(&self, blob: BlobRef) -> Option<AudioMetadata>` | impl ResourceStore for Images | [源码:71](../../../desktop/infrastructure/src/lib.rs#L71) |  |

## skin_editor_infrastructure::MAX_PENDING_PREPARATIONS

[源码:16](../../../desktop/infrastructure/src/audio_preparation.rs#L16) · constant

```rust
pub const MAX_PENDING_PREPARATIONS: usize = 4;
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_infrastructure::audio_preparation::MAX_PENDING_PREPARATIONS`

## skin_editor_infrastructure::MAX_SUPPLY_CONTROLS

[源码:25](../../../desktop/infrastructure/src/audio_supply.rs#L25) · constant

```rust
pub const MAX_SUPPLY_CONTROLS: usize = 16;
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_infrastructure::audio_supply::MAX_SUPPLY_CONTROLS`

## skin_editor_infrastructure::NativeOutput

[源码:990](../../../desktop/infrastructure/src/native_audio.rs#L990) · struct

```rust
pub struct NativeOutput {
    stream: Stream,
    pub producer: OutputProducer,
    pub rate: u32,
    pub channels: u16,
    pub sample_format: SampleFormat,
    pub period_frames: u32,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_infrastructure::native_audio::NativeOutput`

声明引用：`skin_editor_infrastructure::OutputProducer`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::audio_verification::phase (声明)`, `skin_editor_desktop::audio_verification::quiet (声明)`, `skin_editor_desktop::audio_verification::settle (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl NativeOutput` — [源码:1019](../../../desktop/infrastructure/src/native_audio.rs#L1019)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn into_parts(self) -> (NativeStream, OutputProducer)` | impl NativeOutput | [源码:1020](../../../desktop/infrastructure/src/native_audio.rs#L1020) | skin_editor_infrastructure::NativeStream, skin_editor_infrastructure::OutputProducer |
| `pub fn open(budget: &PcmBudget) -> Result<Self, String>` | impl NativeOutput | [源码:1023](../../../desktop/infrastructure/src/native_audio.rs#L1023) | skin_editor_audio::PcmBudget |
| `pub fn play(&self) -> Result<(), String>` | impl NativeOutput | [源码:1077](../../../desktop/infrastructure/src/native_audio.rs#L1077) |  |

## skin_editor_infrastructure::NativeStream

[源码:999](../../../desktop/infrastructure/src/native_audio.rs#L999) · struct

```rust
pub struct NativeStream(Stream);
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_infrastructure::native_audio::NativeStream`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::combined_verification::Combined (声明)`, `skin_editor_desktop::combined_verification::clock_alignment (声明)`, `skin_editor_desktop::editor_audio::EditorAudio (声明)`, `skin_editor_infrastructure::NativeOutput::into_parts`

实现／约束（显式或 derive，无类继承）：

- `impl NativeStream` — [源码:1000](../../../desktop/infrastructure/src/native_audio.rs#L1000)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn clock_sample(&self) -> Result<StreamClockSample, String>` | impl NativeStream | [源码:1002](../../../desktop/infrastructure/src/native_audio.rs#L1002) | skin_editor_infrastructure::StreamClockSample |
| `pub fn play(&self) -> Result<(), String>` | impl NativeStream | [源码:1015](../../../desktop/infrastructure/src/native_audio.rs#L1015) |  |

## skin_editor_infrastructure::OUTPUT_AHEAD_FRAMES

[源码:12](../../../desktop/infrastructure/src/native_audio.rs#L12) · constant

```rust
pub const OUTPUT_AHEAD_FRAMES: usize = 768;
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_infrastructure::native_audio::OUTPUT_AHEAD_FRAMES`

## skin_editor_infrastructure::OUTPUT_BLOCK_FRAMES

[源码:13](../../../desktop/infrastructure/src/native_audio.rs#L13) · constant

```rust
pub const OUTPUT_BLOCK_FRAMES: usize = 128;
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_infrastructure::native_audio::OUTPUT_BLOCK_FRAMES`

## skin_editor_infrastructure::OUTPUT_QUEUE_FRAMES

[源码:11](../../../desktop/infrastructure/src/native_audio.rs#L11) · constant

```rust
pub const OUTPUT_QUEUE_FRAMES: usize = 2048;
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_infrastructure::native_audio::OUTPUT_QUEUE_FRAMES`

## skin_editor_infrastructure::OutputConsumer

[源码:285](../../../desktop/infrastructure/src/native_audio.rs#L285) · struct

```rust
pub struct OutputConsumer {
    consumer: Consumer<Frame>,
    shared: Arc<Shared>,
    channels: u16,
    idle_cutover: u64,
    idle_guard: u64,
    traced_voices: [u64; 2],
    context: [[u64; 11]; 4],
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_infrastructure::native_audio::OutputConsumer`

声明引用：`skin_editor_infrastructure::native_audio::Frame`, `skin_editor_infrastructure::native_audio::Shared`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::output_boundary (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl OutputConsumer` — [源码:764](../../../desktop/infrastructure/src/native_audio.rs#L764)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn render( &mut self, samples: usize, stamp: Option<u64>, write: impl FnMut(usize, f32), )` | impl OutputConsumer | [源码:765](../../../desktop/infrastructure/src/native_audio.rs#L765) |  |
| `pub fn render_f32(&mut self, output: &mut [f32])` | impl OutputConsumer | [源码:980](../../../desktop/infrastructure/src/native_audio.rs#L980) |  |

## skin_editor_infrastructure::OutputControl

[源码:104](../../../desktop/infrastructure/src/native_audio.rs#L104) · struct

```rust
pub struct OutputControl(Arc<Shared>);
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_infrastructure::native_audio::OutputControl`

声明引用：`skin_editor_infrastructure::native_audio::Shared`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::combined_verification::Combined (声明)`, `skin_editor_desktop::combined_verification::Observer::new`, `skin_editor_desktop::combined_verification::Observer::with_capacity`, `skin_editor_desktop::combined_verification::cleared (声明)`, `skin_editor_desktop::combined_verification::clock_alignment (声明)`, `skin_editor_desktop::combined_verification::phase::callback_reference (声明)`, `skin_editor_desktop::combined_verification::play_old (声明)`, `skin_editor_desktop::combined_verification::ready_all (声明)`, `skin_editor_desktop::combined_verification::scheduling_json (声明)`, `skin_editor_desktop::edit_verification::health (声明)`, `skin_editor_desktop::edit_verification::prepare (声明)`, `skin_editor_desktop::edit_verification::ready (声明)`, `skin_editor_desktop::edit_verification::trigger (声明)`, `skin_editor_desktop::load_verification::consume (声明)`, `skin_editor_desktop::load_verification::stop (声明)`, `skin_editor_desktop::load_verification::wait (声明)`, `skin_editor_infrastructure::AudioSupplyClient::output`, `skin_editor_infrastructure::OutputControl::clone`, `skin_editor_infrastructure::OutputProducer::control`, `skin_editor_infrastructure::supply_output::SupplyOutputControl (声明)`, `skin_editor_infrastructure::supply_output::SupplyOutputControl::premixed`

实现／约束（显式或 derive，无类继承）：

- `impl OutputControl` — [源码:119](../../../desktop/infrastructure/src/native_audio.rs#L119)
- `impl Clone for OutputControl` — [源码:103](../../../desktop/infrastructure/src/native_audio.rs#L103)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn capture_origin(&self) -> Option<Instant>` | impl OutputControl | [源码:120](../../../desktop/infrastructure/src/native_audio.rs#L120) |  |
| `pub fn supply_timing(&self) -> SupplyTimingSnapshot` | impl OutputControl | [源码:123](../../../desktop/infrastructure/src/native_audio.rs#L123) | skin_editor_infrastructure::SupplyTimingSnapshot |
| `pub fn capture_submissions(&self) -> Result<Instant, String>` | impl OutputControl | [源码:127](../../../desktop/infrastructure/src/native_audio.rs#L127) |  |
| `pub fn submission_trace( &self, lane: usize, voice: u64, ) -> Option<SubmissionTrace>` | impl OutputControl | [源码:133](../../../desktop/infrastructure/src/native_audio.rs#L133) | skin_editor_infrastructure::SubmissionTrace |
| `pub fn voice_submission( &self, lane: usize, voice: u64, ) -> Result<VoiceSubmission, String>` | impl OutputControl | [源码:138](../../../desktop/infrastructure/src/native_audio.rs#L138) | skin_editor_infrastructure::VoiceSubmission |
| `pub fn callback_progress(&self) -> Option<[u64; 4]>` | impl OutputControl | [源码:155](../../../desktop/infrastructure/src/native_audio.rs#L155) |  |
| `pub fn set_diagnostic_stage(&self, stage: u64) -> Result<(), String>` | impl OutputControl | [源码:160](../../../desktop/infrastructure/src/native_audio.rs#L160) |  |
| `pub fn diagnostic_stage_counts(&self) -> [[u64; 5]; 8]` | impl OutputControl | [源码:165](../../../desktop/infrastructure/src/native_audio.rs#L165) |  |
| `pub fn underrun_contexts(&self) -> Vec<Option<[[u64; 11]; 4]>>` | impl OutputControl | [源码:172](../../../desktop/infrastructure/src/native_audio.rs#L172) |  |
| `pub fn underrun_observations(&self) -> Vec<[u64; 6]>` | impl OutputControl | [源码:180](../../../desktop/infrastructure/src/native_audio.rs#L180) |  |
| `pub fn capture_startup(&self) -> Result<(), String>` | impl OutputControl | [源码:192](../../../desktop/infrastructure/src/native_audio.rs#L192) |  |
| `pub fn startup_observations(&self) -> Vec<[u64; 4]>` | impl OutputControl | [源码:200](../../../desktop/infrastructure/src/native_audio.rs#L200) |  |
| `pub fn set_monitor_volume(&self, permille: u16) -> Result<(), String>` | impl OutputControl | [源码:211](../../../desktop/infrastructure/src/native_audio.rs#L211) |  |
| `pub fn is_closed(&self) -> bool` | impl OutputControl | [源码:221](../../../desktop/infrastructure/src/native_audio.rs#L221) |  |
| `pub(crate) fn mark(&self) -> OutputMark` | impl OutputControl | [源码:224](../../../desktop/infrastructure/src/native_audio.rs#L224) | skin_editor_infrastructure::output_fence::OutputMark |
| `pub(crate) fn invalidate_skin( &self, scene: bool, audition: bool, ) -> Result<OutputMark, String>` | impl OutputControl | [源码:232](../../../desktop/infrastructure/src/native_audio.rs#L232) | skin_editor_infrastructure::output_fence::OutputMark |
| `pub(crate) fn invalidate_scene(&self) -> Result<OutputMark, String>` | impl OutputControl | [源码:246](../../../desktop/infrastructure/src/native_audio.rs#L246) | skin_editor_infrastructure::output_fence::OutputMark |
| `pub(crate) fn background_acknowledged(&self, mark: OutputMark) -> bool` | impl OutputControl | [源码:253](../../../desktop/infrastructure/src/native_audio.rs#L253) | skin_editor_infrastructure::output_fence::OutputMark |
| `pub fn stats(&self) -> OutputStats` | impl OutputControl | [源码:257](../../../desktop/infrastructure/src/native_audio.rs#L257) | skin_editor_infrastructure::OutputStats |
| `pub fn close(&self)` | impl OutputControl | [源码:260](../../../desktop/infrastructure/src/native_audio.rs#L260) |  |
| `fn clone(&self) -> OutputControl` | impl Clone for OutputControl | [源码:103](../../../desktop/infrastructure/src/native_audio.rs#L103) | skin_editor_infrastructure::OutputControl |

## skin_editor_infrastructure::OutputFence

[源码:3](../../../desktop/infrastructure/src/output_fence.rs#L3) · struct

```rust
pub struct OutputFence(pub(crate) [u64; 2]);
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_infrastructure::output_fence::OutputFence`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::AudioPlayback::close`, `skin_editor_infrastructure::AudioPlayback::scene_running`, `skin_editor_infrastructure::AudioPlayback::stop_audition`, `skin_editor_infrastructure::OutputFence::clone`, `skin_editor_infrastructure::OutputFence::eq`, `skin_editor_infrastructure::OutputProducer::acknowledged`, `skin_editor_infrastructure::OutputProducer::invalidate`, `skin_editor_infrastructure::output_fence::OutputMark (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl OutputFence` — [源码:4](../../../desktop/infrastructure/src/output_fence.rs#L4)
- `impl Clone for OutputFence` — [源码:2](../../../desktop/infrastructure/src/output_fence.rs#L2)
- `impl Debug for OutputFence` — [源码:2](../../../desktop/infrastructure/src/output_fence.rs#L2)
- `impl PartialEq for OutputFence` — [源码:2](../../../desktop/infrastructure/src/output_fence.rs#L2)
- `impl Copy for OutputFence` — [源码:2](../../../desktop/infrastructure/src/output_fence.rs#L2)
- `impl Eq for OutputFence` — [源码:2](../../../desktop/infrastructure/src/output_fence.rs#L2)
- `impl StructuralPartialEq for OutputFence` — [源码:2](../../../desktop/infrastructure/src/output_fence.rs#L2)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub(crate) fn with_scene(self, other: Self) -> Self` | impl OutputFence | [源码:5](../../../desktop/infrastructure/src/output_fence.rs#L5) |  |
| `pub(crate) fn with_audition(self, other: Self) -> Self` | impl OutputFence | [源码:8](../../../desktop/infrastructure/src/output_fence.rs#L8) |  |
| `fn clone(&self) -> OutputFence` | impl Clone for OutputFence | [源码:2](../../../desktop/infrastructure/src/output_fence.rs#L2) | skin_editor_infrastructure::OutputFence |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for OutputFence | [源码:2](../../../desktop/infrastructure/src/output_fence.rs#L2) |  |
| `fn eq(&self, other: &OutputFence) -> bool` | impl PartialEq for OutputFence | [源码:2](../../../desktop/infrastructure/src/output_fence.rs#L2) | skin_editor_infrastructure::OutputFence |

## skin_editor_infrastructure::OutputProducer

[源码:264](../../../desktop/infrastructure/src/native_audio.rs#L264) · struct

```rust
pub struct OutputProducer {
    producer: Producer<Frame>,
    shared: Arc<Shared>,
    scene: PcmBuffer,
    audition: PcmBuffer,
    voice_marks: [[u64; 2]; 128],
    rate: u32,
    channels: u16,
    idle_reserve: usize,
    was_idle: bool,
    next_serial: u64,
    traced_voices: [u64; 2],
    owner_wake_ns: u64,
    previous_sleep_ns: u64,
    previous_pump_finish_ns: u64,
    policy: SupplyPolicy,
    period_frames: usize,
    previous_wake: Option<Instant>,
    late_wake: bool,
    diagnostic_recovery: bool,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_infrastructure::native_audio::OutputProducer`

声明引用：`skin_editor_audio::PcmBuffer`, `skin_editor_infrastructure::SupplyPolicy`, `skin_editor_infrastructure::native_audio::Frame`, `skin_editor_infrastructure::native_audio::Shared`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::AudioPlayback::close`, `skin_editor_infrastructure::AudioPlayback::fence_if_changed`, `skin_editor_infrastructure::AudioPlayback::prefill`, `skin_editor_infrastructure::AudioPlayback::refresh`, `skin_editor_infrastructure::AudioPlayback::scene_running`, `skin_editor_infrastructure::AudioPlayback::step`, `skin_editor_infrastructure::AudioPlayback::stop_audition`, `skin_editor_infrastructure::AudioPlayback::trigger`, `skin_editor_infrastructure::AudioPlayback::wait_preparation`, `skin_editor_infrastructure::AudioSupplyClient::spawn`, `skin_editor_infrastructure::NativeOutput (声明)`, `skin_editor_infrastructure::NativeOutput::into_parts`, `skin_editor_infrastructure::audio_supply::SupplyCore::apply_owned`, `skin_editor_infrastructure::audio_supply::SupplyCore::new`, `skin_editor_infrastructure::audio_supply::SupplyOutput (声明)`, `skin_editor_infrastructure::native_audio::OutputSafety::is_for`, `skin_editor_infrastructure::output_boundary (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl OutputProducer` — [源码:399](../../../desktop/infrastructure/src/native_audio.rs#L399)
- `impl OutputProducer` — [源码:501](../../../desktop/infrastructure/src/native_audio.rs#L501)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub(crate) fn arm_diagnostic_recovery(&mut self) -> Result<(), String>` | impl OutputProducer | [源码:400](../../../desktop/infrastructure/src/native_audio.rs#L400) |  |
| `pub(crate) fn cancel_diagnostic_recovery(&mut self)` | impl OutputProducer | [源码:415](../../../desktop/infrastructure/src/native_audio.rs#L415) |  |
| `pub(crate) fn owner_woke(&mut self, now: Instant)` | impl OutputProducer | [源码:418](../../../desktop/infrastructure/src/native_audio.rs#L418) |  |
| `pub(crate) fn owner_sleeping(&mut self)` | impl OutputProducer | [源码:429](../../../desktop/infrastructure/src/native_audio.rs#L429) |  |
| `fn complete_pump( &mut self, origin: Option<Instant>, started: Option<u64>, before: usize, produced: usize, )` | impl OutputProducer | [源码:434](../../../desktop/infrastructure/src/native_audio.rs#L434) |  |
| `pub fn control(&self) -> OutputControl` | impl OutputProducer | [源码:459](../../../desktop/infrastructure/src/native_audio.rs#L459) | skin_editor_infrastructure::OutputControl |
| `pub(crate) fn safety(&self) -> OutputSafety` | impl OutputProducer | [源码:462](../../../desktop/infrastructure/src/native_audio.rs#L462) | skin_editor_infrastructure::native_audio::OutputSafety |
| `pub fn fail_device(&self)` | impl OutputProducer | [源码:466](../../../desktop/infrastructure/src/native_audio.rs#L466) |  |
| `pub fn stats(&self) -> OutputStats` | impl OutputProducer | [源码:469](../../../desktop/infrastructure/src/native_audio.rs#L469) | skin_editor_infrastructure::OutputStats |
| `pub fn configure_period(&mut self, period_frames: usize) -> Result<(), String>` | impl OutputProducer | [源码:504](../../../desktop/infrastructure/src/native_audio.rs#L504) |  |
| `pub fn configure_supply_policy( &mut self, policy: SupplyPolicy, ) -> Result<(), String>` | impl OutputProducer | [源码:525](../../../desktop/infrastructure/src/native_audio.rs#L525) | skin_editor_infrastructure::SupplyPolicy |
| `pub fn idle_reserve_frames(&self) -> usize` | impl OutputProducer | [源码:532](../../../desktop/infrastructure/src/native_audio.rs#L532) |  |
| `pub fn queued_frames(&self) -> usize` | impl OutputProducer | [源码:535](../../../desktop/infrastructure/src/native_audio.rs#L535) |  |
| `pub fn invalidate( &self, scene: bool, audition: bool, ) -> Result<OutputFence, String>` | impl OutputProducer | [源码:538](../../../desktop/infrastructure/src/native_audio.rs#L538) | skin_editor_infrastructure::OutputFence |
| `pub fn acknowledged(&self, fence: OutputFence) -> bool` | impl OutputProducer | [源码:550](../../../desktop/infrastructure/src/native_audio.rs#L550) | skin_editor_infrastructure::OutputFence |
| `pub fn pump(&mut self, runtime: &mut Runtime) -> Result<usize, String>` | impl OutputProducer | [源码:554](../../../desktop/infrastructure/src/native_audio.rs#L554) | skin_editor_audio::Runtime |
| `pub fn prefill(&mut self, runtime: &mut Runtime) -> Result<usize, String>` | impl OutputProducer | [源码:558](../../../desktop/infrastructure/src/native_audio.rs#L558) | skin_editor_audio::Runtime |
| `fn pump_to( &mut self, runtime: &mut Runtime, target: usize, ) -> Result<usize, String>` | impl OutputProducer | [源码:561](../../../desktop/infrastructure/src/native_audio.rs#L561) | skin_editor_audio::Runtime |
| `pub(crate) fn pump_timeline( &mut self, runtime: &mut Runtime, background: Option<&mut BackgroundTrack>, mark: OutputMark, target: usize, ) -> Result<usize, String>` | impl OutputProducer | [源码:564](../../../desktop/infrastructure/src/native_audio.rs#L564) | skin_editor_audio::BackgroundTrack, skin_editor_audio::Runtime, skin_editor_infrastructure::output_fence::OutputMark |

## skin_editor_infrastructure::OutputStats

[源码:75](../../../desktop/infrastructure/src/native_audio.rs#L75) · struct

```rust
pub struct OutputStats {
    pub recovery_passes: u64,
    pub diagnostic_recovery_passes: u64,
    pub diagnostic_refill_passes: u64,
    pub diagnostic_refill_frames: u64,
    pub max_recovery_target: u64,
    pub background_next: u64,
    pub background_gaps: u64,
    pub background_frames: u64,
    pub callbacks: u64,
    pub max_callback_frames: u64,
    pub frames: u64,
    pub empty_frames: u64,
    pub empty_callbacks: u64,
    pub retimed_idle_frames: u64,
    pub max_empty_callback_frames: u64,
    pub stale_lane_frames: u64,
    pub nonzero_samples: u64,
    pub lane_nonzero_frames: [u64; 2],
    pub submitted_voice_ids: [u64; 2],
    pub lane_epochs: [u64; 2],
    pub lane_acknowledged: [u64; 2],
    pub failed: bool,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_infrastructure::native_audio::OutputStats`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::combined_verification::Combined (声明)`, `skin_editor_infrastructure::OutputControl::stats`, `skin_editor_infrastructure::OutputProducer::stats`, `skin_editor_infrastructure::OutputStats::clone`, `skin_editor_infrastructure::OutputStats::default`, `skin_editor_infrastructure::native_audio::shared_stats (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for OutputStats` — [源码:74](../../../desktop/infrastructure/src/native_audio.rs#L74)
- `impl Debug for OutputStats` — [源码:74](../../../desktop/infrastructure/src/native_audio.rs#L74)
- `impl Default for OutputStats` — [源码:74](../../../desktop/infrastructure/src/native_audio.rs#L74)
- `impl Copy for OutputStats` — [源码:74](../../../desktop/infrastructure/src/native_audio.rs#L74)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> OutputStats` | impl Clone for OutputStats | [源码:74](../../../desktop/infrastructure/src/native_audio.rs#L74) | skin_editor_infrastructure::OutputStats |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for OutputStats | [源码:74](../../../desktop/infrastructure/src/native_audio.rs#L74) |  |
| `fn default() -> OutputStats` | impl Default for OutputStats | [源码:74](../../../desktop/infrastructure/src/native_audio.rs#L74) | skin_editor_infrastructure::OutputStats |

## skin_editor_infrastructure::PreparationError

[源码:19](../../../desktop/infrastructure/src/audio_preparation.rs#L19) · enum

```rust
pub enum PreparationError {
    Unsupported,
    Busy,
    Closed,
    ResourceLimit,
    MetadataMismatch,
    Cancelled,
    Stale,
    MissingResource,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_infrastructure::audio_preparation::PreparationError`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::AudioPreparationQueue::finish`, `skin_editor_infrastructure::AudioPreparationQueue::submit`, `skin_editor_infrastructure::BuildAdmission (声明)`, `skin_editor_infrastructure::PreparationError::clone`, `skin_editor_infrastructure::PreparationError::eq`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for PreparationError` — [源码:18](../../../desktop/infrastructure/src/audio_preparation.rs#L18)
- `impl Debug for PreparationError` — [源码:18](../../../desktop/infrastructure/src/audio_preparation.rs#L18)
- `impl PartialEq for PreparationError` — [源码:18](../../../desktop/infrastructure/src/audio_preparation.rs#L18)
- `impl Copy for PreparationError` — [源码:18](../../../desktop/infrastructure/src/audio_preparation.rs#L18)
- `impl Eq for PreparationError` — [源码:18](../../../desktop/infrastructure/src/audio_preparation.rs#L18)
- `impl StructuralPartialEq for PreparationError` — [源码:18](../../../desktop/infrastructure/src/audio_preparation.rs#L18)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> PreparationError` | impl Clone for PreparationError | [源码:18](../../../desktop/infrastructure/src/audio_preparation.rs#L18) | skin_editor_infrastructure::PreparationError |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for PreparationError | [源码:18](../../../desktop/infrastructure/src/audio_preparation.rs#L18) |  |
| `fn eq(&self, other: &PreparationError) -> bool` | impl PartialEq for PreparationError | [源码:18](../../../desktop/infrastructure/src/audio_preparation.rs#L18) | skin_editor_infrastructure::PreparationError |

## skin_editor_infrastructure::PreparationObservation

[源码:92](../../../desktop/infrastructure/src/audio_preparation.rs#L92) · struct

```rust
pub struct PreparationObservation {
    pub request: BuildRequest,
    pub queue_wait: Duration,
    pub decode_and_allocate: Duration,
    pub process: Duration,
    pub result_handoff: Duration,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_infrastructure::audio_preparation::PreparationObservation`

声明引用：`skin_editor_audio::BuildRequest`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::FinishedPreparation (声明)`, `skin_editor_infrastructure::FinishedPreparation::observation`, `skin_editor_infrastructure::PreparationObservation::clone`, `skin_editor_infrastructure::PreparationObservation::eq`, `skin_editor_infrastructure::SupplyStatus (声明)`, `skin_editor_infrastructure::audio_supply::SupplyCore (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for PreparationObservation` — [源码:91](../../../desktop/infrastructure/src/audio_preparation.rs#L91)
- `impl Debug for PreparationObservation` — [源码:91](../../../desktop/infrastructure/src/audio_preparation.rs#L91)
- `impl PartialEq for PreparationObservation` — [源码:91](../../../desktop/infrastructure/src/audio_preparation.rs#L91)
- `impl Copy for PreparationObservation` — [源码:91](../../../desktop/infrastructure/src/audio_preparation.rs#L91)
- `impl StructuralPartialEq for PreparationObservation` — [源码:91](../../../desktop/infrastructure/src/audio_preparation.rs#L91)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> PreparationObservation` | impl Clone for PreparationObservation | [源码:91](../../../desktop/infrastructure/src/audio_preparation.rs#L91) | skin_editor_infrastructure::PreparationObservation |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for PreparationObservation | [源码:91](../../../desktop/infrastructure/src/audio_preparation.rs#L91) |  |
| `fn eq(&self, other: &PreparationObservation) -> bool` | impl PartialEq for PreparationObservation | [源码:91](../../../desktop/infrastructure/src/audio_preparation.rs#L91) | skin_editor_infrastructure::PreparationObservation |

## skin_editor_infrastructure::PreparationUsage

[源码:166](../../../desktop/infrastructure/src/audio_preparation.rs#L166) · struct

```rust
pub struct PreparationUsage {
    pub active: usize,
    pub pending: usize,
    pub cached: usize,
    pub pcm: PcmUsage,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_infrastructure::audio_preparation::PreparationUsage`

声明引用：`skin_editor_audio::PcmUsage`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::AudioPreparationQueue::usage`, `skin_editor_infrastructure::PreparationUsage::clone`, `skin_editor_infrastructure::PreparationUsage::eq`, `skin_editor_infrastructure::SkinAudioController::preparation_usage`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for PreparationUsage` — [源码:165](../../../desktop/infrastructure/src/audio_preparation.rs#L165)
- `impl Debug for PreparationUsage` — [源码:165](../../../desktop/infrastructure/src/audio_preparation.rs#L165)
- `impl PartialEq for PreparationUsage` — [源码:165](../../../desktop/infrastructure/src/audio_preparation.rs#L165)
- `impl Copy for PreparationUsage` — [源码:165](../../../desktop/infrastructure/src/audio_preparation.rs#L165)
- `impl Eq for PreparationUsage` — [源码:165](../../../desktop/infrastructure/src/audio_preparation.rs#L165)
- `impl StructuralPartialEq for PreparationUsage` — [源码:165](../../../desktop/infrastructure/src/audio_preparation.rs#L165)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> PreparationUsage` | impl Clone for PreparationUsage | [源码:165](../../../desktop/infrastructure/src/audio_preparation.rs#L165) | skin_editor_infrastructure::PreparationUsage |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for PreparationUsage | [源码:165](../../../desktop/infrastructure/src/audio_preparation.rs#L165) |  |
| `fn eq(&self, other: &PreparationUsage) -> bool` | impl PartialEq for PreparationUsage | [源码:165](../../../desktop/infrastructure/src/audio_preparation.rs#L165) | skin_editor_infrastructure::PreparationUsage |

## skin_editor_infrastructure::PreparationWorker

[源码:6](../../../desktop/infrastructure/src/audio_worker.rs#L6) · struct

```rust
pub struct PreparationWorker {
    sender: Option<SyncSender<RunningPreparation>>,
    receiver: Receiver<FinishedPreparation>,
    thread: Option<JoinHandle<()>>,
    join_ok: bool,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_infrastructure::audio_worker::PreparationWorker`

声明引用：`skin_editor_infrastructure::FinishedPreparation`, `skin_editor_infrastructure::RunningPreparation`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::AudioPlayback (声明)`, `skin_editor_infrastructure::audio_supply::SupplyCore (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl PreparationWorker` — [源码:12](../../../desktop/infrastructure/src/audio_worker.rs#L12)
- `impl Drop for PreparationWorker` — [源码:65](../../../desktop/infrastructure/src/audio_worker.rs#L65)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn new(decoder: impl AudioDecoder + Send + 'static) -> Result<Self>` | impl PreparationWorker | [源码:13](../../../desktop/infrastructure/src/audio_worker.rs#L13) | skin_editor_infrastructure::AudioDecoder |
| `pub fn try_submit( &self, job: RunningPreparation, ) -> Result<(), Box<RunningPreparation>>` | impl PreparationWorker | [源码:32](../../../desktop/infrastructure/src/audio_worker.rs#L32) | skin_editor_infrastructure::RunningPreparation |
| `pub fn try_result(&self) -> Option<FinishedPreparation>` | impl PreparationWorker | [源码:40](../../../desktop/infrastructure/src/audio_worker.rs#L40) | skin_editor_infrastructure::FinishedPreparation |
| `pub fn wait_result(&self, timeout: Duration) -> Option<FinishedPreparation>` | impl PreparationWorker | [源码:44](../../../desktop/infrastructure/src/audio_worker.rs#L44) | skin_editor_infrastructure::FinishedPreparation |
| `pub fn close(&mut self)` | impl PreparationWorker | [源码:48](../../../desktop/infrastructure/src/audio_worker.rs#L48) |  |
| `pub fn closed(&self) -> bool` | impl PreparationWorker | [源码:61](../../../desktop/infrastructure/src/audio_worker.rs#L61) |  |
| `fn drop(&mut self)` | impl Drop for PreparationWorker | [源码:66](../../../desktop/infrastructure/src/audio_worker.rs#L66) |  |

## skin_editor_infrastructure::RawBlob

[源码:40](../../../desktop/infrastructure/src/blobs.rs#L40) · struct

```rust
pub struct RawBlob(Arc<Entry>);
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_infrastructure::blobs::RawBlob`

声明引用：`skin_editor_infrastructure::blobs::Entry`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::frame_worker::Job (声明)`, `skin_editor_infrastructure::AudioPreparationQueue::submit`, `skin_editor_infrastructure::BlobStore::get`, `skin_editor_infrastructure::BlobStore::insert`, `skin_editor_infrastructure::BlobStore::stage_audio`, `skin_editor_infrastructure::BlobStore::stage_bytes`, `skin_editor_infrastructure::BlobStore::stage_original_file`, `skin_editor_infrastructure::BlobStore::stage_reader`, `skin_editor_infrastructure::BlobStore::stage_wav`, `skin_editor_infrastructure::Images::prepare`, `skin_editor_infrastructure::RawBlob::clone`, `skin_editor_infrastructure::audio_preparation::Pending (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl RawBlob` — [源码:41](../../../desktop/infrastructure/src/blobs.rs#L41)
- `impl Clone for RawBlob` — [源码:39](../../../desktop/infrastructure/src/blobs.rs#L39)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn reference(&self) -> BlobRef` | impl RawBlob | [源码:42](../../../desktop/infrastructure/src/blobs.rs#L42) |  |
| `pub fn bytes(&self) -> &[u8] ⓘ` | impl RawBlob | [源码:45](../../../desktop/infrastructure/src/blobs.rs#L45) |  |
| `pub fn lease(&self) -> Arc<dyn BlobLease>` | impl RawBlob | [源码:48](../../../desktop/infrastructure/src/blobs.rs#L48) |  |
| `pub fn same_allocation(&self, other: &Self) -> bool` | impl RawBlob | [源码:51](../../../desktop/infrastructure/src/blobs.rs#L51) |  |
| `pub fn audio_metadata(&self) -> Option<AudioMetadata>` | impl RawBlob | [源码:54](../../../desktop/infrastructure/src/blobs.rs#L54) |  |
| `pub(crate) fn attest_audio( &self, metadata: AudioMetadata, ) -> Result<(), BlobError>` | impl RawBlob | [源码:57](../../../desktop/infrastructure/src/blobs.rs#L57) | skin_editor_infrastructure::BlobError |
| `fn clone(&self) -> RawBlob` | impl Clone for RawBlob | [源码:39](../../../desktop/infrastructure/src/blobs.rs#L39) | skin_editor_infrastructure::RawBlob |

## skin_editor_infrastructure::RunningPreparation

[源码:77](../../../desktop/infrastructure/src/audio_preparation.rs#L77) · struct

```rust
pub struct RunningPreparation {
    id: u64,
    job: Pending,
    life: Arc<JobLife>,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_infrastructure::audio_preparation::RunningPreparation`

声明引用：`skin_editor_infrastructure::audio_preparation::JobLife`, `skin_editor_infrastructure::audio_preparation::Pending`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::AudioPreparationQueue::start_next`, `skin_editor_infrastructure::PreparationWorker (声明)`, `skin_editor_infrastructure::PreparationWorker::try_submit`, `skin_editor_infrastructure::SkinAudioController::start_preparation`

实现／约束（显式或 derive，无类继承）：

- `impl RunningPreparation` — [源码:107](../../../desktop/infrastructure/src/audio_preparation.rs#L107)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn request(&self) -> BuildRequest` | impl RunningPreparation | [源码:108](../../../desktop/infrastructure/src/audio_preparation.rs#L108) | skin_editor_audio::BuildRequest |
| `pub fn execute(self, decoder: &mut dyn AudioDecoder) -> FinishedPreparation` | impl RunningPreparation | [源码:111](../../../desktop/infrastructure/src/audio_preparation.rs#L111) | skin_editor_infrastructure::AudioDecoder, skin_editor_infrastructure::FinishedPreparation |

## skin_editor_infrastructure::SkinAudioController

[源码:14](../../../desktop/infrastructure/src/skin_audio.rs#L14) · struct

```rust
pub struct SkinAudioController {
    runtime: Runtime,
    preparations: AudioPreparationQueue,
    builds: Vec<BuildRequest>,
    last_origin: Option<Origin>,
    output_invalidation: u64,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_infrastructure::skin_audio::SkinAudioController`

声明引用：`skin_editor_audio::BuildRequest`, `skin_editor_audio::Origin`, `skin_editor_audio::Runtime`, `skin_editor_infrastructure::AudioPreparationQueue`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::AudioPlayback (声明)`, `skin_editor_infrastructure::audio_supply::SupplyCore (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl SkinAudioController` — [源码:21](../../../desktop/infrastructure/src/skin_audio.rs#L21)
- `impl SkinAudioController` — [源码:176](../../../desktop/infrastructure/src/skin_audio.rs#L176)
- `impl SkinAudioPort for SkinAudioController` — [源码:216](../../../desktop/infrastructure/src/skin_audio.rs#L216)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn new(rate: u32, channels: u16) -> Result<Self, RuntimeError>` | impl SkinAudioController | [源码:22](../../../desktop/infrastructure/src/skin_audio.rs#L22) | skin_editor_audio::RuntimeError |
| `pub fn with_budget( rate: u32, channels: u16, budget: PcmBudget, ) -> Result<Self, RuntimeError>` | impl SkinAudioController | [源码:25](../../../desktop/infrastructure/src/skin_audio.rs#L25) | skin_editor_audio::PcmBudget, skin_editor_audio::RuntimeError |
| `pub fn runtime(&mut self) -> &mut Runtime` | impl SkinAudioController | [源码:37](../../../desktop/infrastructure/src/skin_audio.rs#L37) | skin_editor_audio::Runtime |
| `pub fn output_invalidation(&self) -> u64` | impl SkinAudioController | [源码:40](../../../desktop/infrastructure/src/skin_audio.rs#L40) |  |
| `pub fn take_builds(&mut self) -> Vec<BuildRequest>` | impl SkinAudioController | [源码:44](../../../desktop/infrastructure/src/skin_audio.rs#L44) | skin_editor_audio::BuildRequest |
| `pub fn admit_builds( &mut self, session: &ProjectSession, store: &BlobStore, ) -> Result<Vec<BuildAdmission>, SkinAudioError>` | impl SkinAudioController | [源码:48](../../../desktop/infrastructure/src/skin_audio.rs#L48) | skin_editor_infrastructure::BlobStore, skin_editor_infrastructure::BuildAdmission |
| `pub(crate) fn admit_selected( &mut self, store: &BlobStore, ) -> Result<Vec<BuildAdmission>, SkinAudioError>` | impl SkinAudioController | [源码:56](../../../desktop/infrastructure/src/skin_audio.rs#L56) | skin_editor_infrastructure::BlobStore, skin_editor_infrastructure::BuildAdmission |
| `pub(crate) fn request_library( &mut self, plan: SoundPlan, store: &BlobStore, ) -> Result<(), SkinAudioError>` | impl SkinAudioController | [源码:92](../../../desktop/infrastructure/src/skin_audio.rs#L92) | skin_editor_audio::SoundPlan, skin_editor_infrastructure::BlobStore |
| `pub(crate) fn cancel_library(&mut self)` | impl SkinAudioController | [源码:108](../../../desktop/infrastructure/src/skin_audio.rs#L108) |  |
| `pub fn start_preparation(&mut self) -> Option<RunningPreparation>` | impl SkinAudioController | [源码:111](../../../desktop/infrastructure/src/skin_audio.rs#L111) | skin_editor_infrastructure::RunningPreparation |
| `pub fn retry_preparation( &mut self, session: &ProjectSession, component: AudioComponentId, ) -> Result<(), SkinAudioError>` | impl SkinAudioController | [源码:114](../../../desktop/infrastructure/src/skin_audio.rs#L114) |  |
| `pub(crate) fn retry_selected( &mut self, component: AudioComponentId, ) -> Result<(), SkinAudioError>` | impl SkinAudioController | [源码:122](../../../desktop/infrastructure/src/skin_audio.rs#L122) |  |
| `pub fn preparation_usage(&mut self) -> PreparationUsage` | impl SkinAudioController | [源码:133](../../../desktop/infrastructure/src/skin_audio.rs#L133) | skin_editor_infrastructure::PreparationUsage |
| `pub fn finish_preparation( &mut self, session: &ProjectSession, finished: FinishedPreparation, ) -> Result<Completion, SkinAudioError>` | impl SkinAudioController | [源码:136](../../../desktop/infrastructure/src/skin_audio.rs#L136) | skin_editor_audio::Completion, skin_editor_infrastructure::FinishedPreparation |
| `pub(crate) fn finish_selected( &mut self, finished: FinishedPreparation, ) -> Result<Completion, SkinAudioError>` | impl SkinAudioController | [源码:144](../../../desktop/infrastructure/src/skin_audio.rs#L144) | skin_editor_audio::Completion, skin_editor_infrastructure::FinishedPreparation |
| `pub fn close(&mut self)` | impl SkinAudioController | [源码:156](../../../desktop/infrastructure/src/skin_audio.rs#L156) |  |
| `pub fn accept_prepared( &mut self, session: &ProjectSession, request: BuildRequest, result: Result<PreparedSound, PrepareFailure>, ) -> Result<Completion, SkinAudioError>` | impl SkinAudioController | [源码:164](../../../desktop/infrastructure/src/skin_audio.rs#L164) | skin_editor_audio::BuildRequest, skin_editor_audio::Completion, skin_editor_audio::PrepareFailure, skin_editor_audio::PreparedSound |
| `pub(crate) fn select( &mut self, origin: Origin, plans: &[SoundPlan], cancelled_drafts: &[DraftId], ) -> Result<(), SkinAudioError>` | impl SkinAudioController | [源码:177](../../../desktop/infrastructure/src/skin_audio.rs#L177) | skin_editor_audio::Origin, skin_editor_audio::SoundPlan |
| `fn retained_drafts(&self) -> Vec<DraftId>` | impl SkinAudioPort for SkinAudioController | [源码:217](../../../desktop/infrastructure/src/skin_audio.rs#L217) |  |
| `fn synchronize( &mut self, project: &SkinProject, source: SkinAudioSource, cancelled_drafts: &[DraftId], ) -> Result<(), SkinAudioError>` | impl SkinAudioPort for SkinAudioController | [源码:224](../../../desktop/infrastructure/src/skin_audio.rs#L224) |  |

## skin_editor_infrastructure::StreamClockSample

[源码:227](../../../desktop/infrastructure/src/submission_trace.rs#L227) · struct

```rust
pub struct StreamClockSample {
    pub before: Instant,
    pub after: Instant,
    pub stream_nanos: u64,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_infrastructure::submission_trace::StreamClockSample`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::combined_verification::Probe (声明)`, `skin_editor_infrastructure::NativeStream::clock_sample`, `skin_editor_infrastructure::StreamClockSample::clone`

实现／约束（显式或 derive，无类继承）：

- `impl StreamClockSample` — [源码:232](../../../desktop/infrastructure/src/submission_trace.rs#L232)
- `impl Clone for StreamClockSample` — [源码:226](../../../desktop/infrastructure/src/submission_trace.rs#L226)
- `impl Debug for StreamClockSample` — [源码:226](../../../desktop/infrastructure/src/submission_trace.rs#L226)
- `impl Copy for StreamClockSample` — [源码:226](../../../desktop/infrastructure/src/submission_trace.rs#L226)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn callback_bounds_ms(self, nanos: u64, request: Instant) -> [f64; 2]` | impl StreamClockSample | [源码:233](../../../desktop/infrastructure/src/submission_trace.rs#L233) |  |
| `fn clone(&self) -> StreamClockSample` | impl Clone for StreamClockSample | [源码:226](../../../desktop/infrastructure/src/submission_trace.rs#L226) | skin_editor_infrastructure::StreamClockSample |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for StreamClockSample | [源码:226](../../../desktop/infrastructure/src/submission_trace.rs#L226) |  |

## skin_editor_infrastructure::SubmissionTrace

[源码:71](../../../desktop/infrastructure/src/submission_trace.rs#L71) · struct

```rust
pub struct SubmissionTrace {
    pub origin: Instant,
    pub producer: [u64; 9],
    pub callback: [u64; 7],
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_infrastructure::submission_trace::SubmissionTrace`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::OutputControl::submission_trace`, `skin_editor_infrastructure::SubmissionTrace::clone`, `skin_editor_infrastructure::submission_trace::SubmissionCapture::read`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for SubmissionTrace` — [源码:70](../../../desktop/infrastructure/src/submission_trace.rs#L70)
- `impl Debug for SubmissionTrace` — [源码:70](../../../desktop/infrastructure/src/submission_trace.rs#L70)
- `impl Copy for SubmissionTrace` — [源码:70](../../../desktop/infrastructure/src/submission_trace.rs#L70)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> SubmissionTrace` | impl Clone for SubmissionTrace | [源码:70](../../../desktop/infrastructure/src/submission_trace.rs#L70) | skin_editor_infrastructure::SubmissionTrace |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for SubmissionTrace | [源码:70](../../../desktop/infrastructure/src/submission_trace.rs#L70) |  |

## skin_editor_infrastructure::SupplyError

[源码:27](../../../desktop/infrastructure/src/audio_supply.rs#L27) · enum

```rust
pub enum SupplyError {
    Busy,
    Closed,
    Stale,
    Unsupported,
    ResourceLimit,
    InvalidPosition,
    Backend(String),
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_infrastructure::audio_supply::SupplyError`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::editor_audio::EditorAudio (声明)`, `skin_editor_desktop::editor_audio::EditorAudio::receive`, `skin_editor_infrastructure::AudioSupplyClient::audition_asset`, `skin_editor_infrastructure::AudioSupplyClient::barrier`, `skin_editor_infrastructure::AudioSupplyClient::diagnostic_recovery_probe`, `skin_editor_infrastructure::AudioSupplyClient::output`, `skin_editor_infrastructure::AudioSupplyClient::reserve`, `skin_editor_infrastructure::AudioSupplyClient::retry_selected`, `skin_editor_infrastructure::AudioSupplyClient::scene_running`, `skin_editor_infrastructure::AudioSupplyClient::seek_background`, `skin_editor_infrastructure::AudioSupplyClient::send`, `skin_editor_infrastructure::AudioSupplyClient::set_background`, `skin_editor_infrastructure::AudioSupplyClient::spawn`, `skin_editor_infrastructure::AudioSupplyClient::spawn_owned`, `skin_editor_infrastructure::AudioSupplyClient::spawn_separated`, `skin_editor_infrastructure::AudioSupplyClient::spawn_streamed`, `skin_editor_infrastructure::AudioSupplyClient::status`, `skin_editor_infrastructure::AudioSupplyClient::stop_audition`, `skin_editor_infrastructure::AudioSupplyClient::trigger`, `skin_editor_infrastructure::AudioSupplyClient::trigger_batch_selected`, `skin_editor_infrastructure::AudioSupplyClient::trigger_selected`, `skin_editor_infrastructure::AudioSupplyClient::wait_receipt`, `skin_editor_infrastructure::SupplyError::clone`, `skin_editor_infrastructure::SupplyError::eq`, `skin_editor_infrastructure::SupplyReceipt (声明)`, `skin_editor_infrastructure::audio_supply::SupplyCore::activate_library`, `skin_editor_infrastructure::audio_supply::SupplyCore::finish_freeze`, `skin_editor_infrastructure::audio_supply::SupplyCore::new`, `skin_editor_infrastructure::audio_supply::SupplyCore::new_separated`, `skin_editor_infrastructure::audio_supply::SupplyCore::prepare`, `skin_editor_infrastructure::audio_supply::SupplyOutput::pump`, `skin_editor_infrastructure::audio_supply::backend (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for SupplyError` — [源码:26](../../../desktop/infrastructure/src/audio_supply.rs#L26)
- `impl Debug for SupplyError` — [源码:26](../../../desktop/infrastructure/src/audio_supply.rs#L26)
- `impl PartialEq for SupplyError` — [源码:26](../../../desktop/infrastructure/src/audio_supply.rs#L26)
- `impl StructuralPartialEq for SupplyError` — [源码:26](../../../desktop/infrastructure/src/audio_supply.rs#L26)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> SupplyError` | impl Clone for SupplyError | [源码:26](../../../desktop/infrastructure/src/audio_supply.rs#L26) | skin_editor_infrastructure::SupplyError |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for SupplyError | [源码:26](../../../desktop/infrastructure/src/audio_supply.rs#L26) |  |
| `fn eq(&self, other: &SupplyError) -> bool` | impl PartialEq for SupplyError | [源码:26](../../../desktop/infrastructure/src/audio_supply.rs#L26) | skin_editor_infrastructure::SupplyError |

## skin_editor_infrastructure::SupplyOutcome

[源码:44](../../../desktop/infrastructure/src/audio_supply.rs#L44) · enum

```rust
pub enum SupplyOutcome {
    Selected(Vec<BuildAdmission>),
    Triggered(TriggerReceipt),
    BatchTriggered {
        mix_frame: u64,
        results: Vec<Result<TriggerReceipt, RuntimeError>>,
    },
    Frozen {
        next_background_frame: u64,
    },
    Located {
        frame: u64,
        previous_copy_anchor: u64,
    },
    Running,
    BackgroundSet,
    AuditionStopped,
    Observed,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_infrastructure::audio_supply::SupplyOutcome`

声明引用：`skin_editor_audio::RuntimeError`, `skin_editor_audio::TriggerReceipt`, `skin_editor_infrastructure::BuildAdmission`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::editor_audio::EditorAudio::receive`, `skin_editor_desktop::load_verification::outcome (声明)`, `skin_editor_desktop::supply_verification::outcome (声明)`, `skin_editor_desktop::worker_verification::outcome (声明)`, `skin_editor_infrastructure::SupplyReceipt (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Debug for SupplyOutcome` — [源码:43](../../../desktop/infrastructure/src/audio_supply.rs#L43)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for SupplyOutcome | [源码:43](../../../desktop/infrastructure/src/audio_supply.rs#L43) |  |

## skin_editor_infrastructure::SupplyPolicy

[源码:17](../../../desktop/infrastructure/src/native_audio.rs#L17) · enum

```rust
pub enum SupplyPolicy {
    FixedAhead,
    RecoverLateWake,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_infrastructure::native_audio::SupplyPolicy`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::combined_verification::phase::run_with_policy (声明)`, `skin_editor_infrastructure::OutputProducer (声明)`, `skin_editor_infrastructure::OutputProducer::configure_supply_policy`, `skin_editor_infrastructure::SupplyPolicy::clone`, `skin_editor_infrastructure::SupplyPolicy::default`, `skin_editor_infrastructure::SupplyPolicy::eq`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for SupplyPolicy` — [源码:16](../../../desktop/infrastructure/src/native_audio.rs#L16)
- `impl Debug for SupplyPolicy` — [源码:16](../../../desktop/infrastructure/src/native_audio.rs#L16)
- `impl Default for SupplyPolicy` — [源码:16](../../../desktop/infrastructure/src/native_audio.rs#L16)
- `impl PartialEq for SupplyPolicy` — [源码:16](../../../desktop/infrastructure/src/native_audio.rs#L16)
- `impl Copy for SupplyPolicy` — [源码:16](../../../desktop/infrastructure/src/native_audio.rs#L16)
- `impl Eq for SupplyPolicy` — [源码:16](../../../desktop/infrastructure/src/native_audio.rs#L16)
- `impl StructuralPartialEq for SupplyPolicy` — [源码:16](../../../desktop/infrastructure/src/native_audio.rs#L16)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> SupplyPolicy` | impl Clone for SupplyPolicy | [源码:16](../../../desktop/infrastructure/src/native_audio.rs#L16) | skin_editor_infrastructure::SupplyPolicy |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for SupplyPolicy | [源码:16](../../../desktop/infrastructure/src/native_audio.rs#L16) |  |
| `fn default() -> SupplyPolicy` | impl Default for SupplyPolicy | [源码:16](../../../desktop/infrastructure/src/native_audio.rs#L16) | skin_editor_infrastructure::SupplyPolicy |
| `fn eq(&self, other: &SupplyPolicy) -> bool` | impl PartialEq for SupplyPolicy | [源码:16](../../../desktop/infrastructure/src/native_audio.rs#L16) | skin_editor_infrastructure::SupplyPolicy |

## skin_editor_infrastructure::SupplyReceipt

[源码:64](../../../desktop/infrastructure/src/audio_supply.rs#L64) · struct

```rust
pub struct SupplyReceipt {
    pub id: u64,
    pub result: Result<SupplyOutcome, SupplyError>,
    _credit: Credit,
    retained: Vec<DraftId>,
    retained_revision: u64,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_infrastructure::audio_supply::SupplyReceipt`

声明引用：`skin_editor_infrastructure::SupplyError`, `skin_editor_infrastructure::SupplyOutcome`, `skin_editor_infrastructure::audio_supply::Credit`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::worker_verification::receipt (声明)`, `skin_editor_infrastructure::AudioSupplyClient (声明)`, `skin_editor_infrastructure::AudioSupplyClient::accept_receipt`, `skin_editor_infrastructure::AudioSupplyClient::poll`, `skin_editor_infrastructure::AudioSupplyClient::wait_receipt`, `skin_editor_infrastructure::audio_supply::SupplyCore (声明)`, `skin_editor_infrastructure::audio_supply::SupplyCore::apply`, `skin_editor_infrastructure::audio_supply::SupplyCore::apply_output`, `skin_editor_infrastructure::audio_supply::SupplyCore::apply_owned`, `skin_editor_infrastructure::audio_supply::SupplyCore::finish_acknowledgments`, `skin_editor_infrastructure::audio_supply::SupplyCore::finish_freeze`

实现／约束（显式或 derive，无类继承）：

- `impl Debug for SupplyReceipt` — [源码:63](../../../desktop/infrastructure/src/audio_supply.rs#L63)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for SupplyReceipt | [源码:63](../../../desktop/infrastructure/src/audio_supply.rs#L63) |  |

## skin_editor_infrastructure::SupplyStatus

[源码:72](../../../desktop/infrastructure/src/audio_supply.rs#L72) · struct

```rust
pub struct SupplyStatus {
    retained_revision: u64,
    pub origin: Option<Origin>,
    pub generated_background: u64,
    pub copied_background: u64,
    pub background_position: u64,
    pub background_frames: Option<u64>,
    pub background_primed: bool,
    pub frozen: bool,
    pub freezing: bool,
    pub ready_components: usize,
    pub retained_drafts: Vec<DraftId>,
    pub fault: Option<String>,
    pub bindings: Vec<(AudioComponentId, BindingStatus)>,
    pub voices: [usize; 2],
    pub library: Option<(AssetId, BindingStatus)>,
    pub last_preparation: Option<PreparationObservation>,
    pub max_supply_gap_micros: u64,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_infrastructure::audio_supply::SupplyStatus`

声明引用：`skin_editor_audio::BindingStatus`, `skin_editor_audio::Origin`, `skin_editor_infrastructure::PreparationObservation`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::AudioSupplyClient (声明)`, `skin_editor_infrastructure::AudioSupplyClient::status`, `skin_editor_infrastructure::SupplyStatus::clone`, `skin_editor_infrastructure::SupplyStatus::default`, `skin_editor_infrastructure::audio_supply::SupplyCore::publish_progress`, `skin_editor_infrastructure::audio_supply::SupplyCore::status`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for SupplyStatus` — [源码:71](../../../desktop/infrastructure/src/audio_supply.rs#L71)
- `impl Debug for SupplyStatus` — [源码:71](../../../desktop/infrastructure/src/audio_supply.rs#L71)
- `impl Default for SupplyStatus` — [源码:71](../../../desktop/infrastructure/src/audio_supply.rs#L71)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> SupplyStatus` | impl Clone for SupplyStatus | [源码:71](../../../desktop/infrastructure/src/audio_supply.rs#L71) | skin_editor_infrastructure::SupplyStatus |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for SupplyStatus | [源码:71](../../../desktop/infrastructure/src/audio_supply.rs#L71) |  |
| `fn default() -> SupplyStatus` | impl Default for SupplyStatus | [源码:71](../../../desktop/infrastructure/src/audio_supply.rs#L71) | skin_editor_infrastructure::SupplyStatus |

## skin_editor_infrastructure::SupplyTimingSnapshot

[源码:109](../../../desktop/infrastructure/src/native_audio.rs#L109) · struct

```rust
pub struct SupplyTimingSnapshot {
    pub completed_pass: u64,
    pub recent: [Option<[u64; 10]>; 4],
    pub slow_count: u64,
    pub slow: Vec<Option<[u64; 10]>>,
    pub event_count: u64,
    pub events: Vec<Option<[u64; 86]>>,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_infrastructure::native_audio::SupplyTimingSnapshot`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::OutputControl::supply_timing`, `skin_editor_infrastructure::submission_trace::SubmissionCapture::scheduling`

实现／约束（显式或 derive，无类继承）：

- `impl Debug for SupplyTimingSnapshot` — [源码:108](../../../desktop/infrastructure/src/native_audio.rs#L108)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for SupplyTimingSnapshot | [源码:108](../../../desktop/infrastructure/src/native_audio.rs#L108) |  |

## skin_editor_infrastructure::VoiceSubmission

[源码:62](../../../desktop/infrastructure/src/submission_trace.rs#L62) · enum

```rust
pub enum VoiceSubmission {
    Unknown,
    Pending,
    Complete([u64; 7]),
    Superseded,
    Invalidated,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_infrastructure::submission_trace::VoiceSubmission`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::combined_verification::SubmissionBracket::completion`, `skin_editor_infrastructure::OutputControl::voice_submission`, `skin_editor_infrastructure::VoiceSubmission::clone`, `skin_editor_infrastructure::VoiceSubmission::eq`, `skin_editor_infrastructure::submission_trace::SubmissionCapture::observe`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for VoiceSubmission` — [源码:61](../../../desktop/infrastructure/src/submission_trace.rs#L61)
- `impl Debug for VoiceSubmission` — [源码:61](../../../desktop/infrastructure/src/submission_trace.rs#L61)
- `impl PartialEq for VoiceSubmission` — [源码:61](../../../desktop/infrastructure/src/submission_trace.rs#L61)
- `impl Copy for VoiceSubmission` — [源码:61](../../../desktop/infrastructure/src/submission_trace.rs#L61)
- `impl Eq for VoiceSubmission` — [源码:61](../../../desktop/infrastructure/src/submission_trace.rs#L61)
- `impl StructuralPartialEq for VoiceSubmission` — [源码:61](../../../desktop/infrastructure/src/submission_trace.rs#L61)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> VoiceSubmission` | impl Clone for VoiceSubmission | [源码:61](../../../desktop/infrastructure/src/submission_trace.rs#L61) | skin_editor_infrastructure::VoiceSubmission |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for VoiceSubmission | [源码:61](../../../desktop/infrastructure/src/submission_trace.rs#L61) |  |
| `fn eq(&self, other: &VoiceSubmission) -> bool` | impl PartialEq for VoiceSubmission | [源码:61](../../../desktop/infrastructure/src/submission_trace.rs#L61) | skin_editor_infrastructure::VoiceSubmission |

## skin_editor_infrastructure::WavDecoder

[源码:7](../../../desktop/infrastructure/src/wav.rs#L7) · struct

```rust
pub struct WavDecoder;
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_infrastructure::wav::WavDecoder`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::WavDecoder::default`

实现／约束（显式或 derive，无类继承）：

- `impl AudioDecoder for WavDecoder` — [源码:87](../../../desktop/infrastructure/src/wav.rs#L87)
- `impl Default for WavDecoder` — [源码:6](../../../desktop/infrastructure/src/wav.rs#L6)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn probe(&self, bytes: &[u8]) -> Result<AudioMetadata, PrepareFailure>` | impl AudioDecoder for WavDecoder | [源码:88](../../../desktop/infrastructure/src/wav.rs#L88) | skin_editor_audio::PrepareFailure |
| `fn decode_into( &mut self, bytes: &[u8], metadata: AudioMetadata, output: &mut [f32], cancel: &Cancellation, ) -> Result<usize, PrepareFailure>` | impl AudioDecoder for WavDecoder | [源码:91](../../../desktop/infrastructure/src/wav.rs#L91) | skin_editor_audio::PrepareFailure, skin_editor_infrastructure::Cancellation |
| `fn default() -> WavDecoder` | impl Default for WavDecoder | [源码:6](../../../desktop/infrastructure/src/wav.rs#L6) | skin_editor_infrastructure::WavDecoder |

## skin_editor_infrastructure::audio_preparation::Active

[源码:72](../../../desktop/infrastructure/src/audio_preparation.rs#L72) · struct

```rust
struct Active {
    id: u64,
    request: BuildRequest,
    life: Weak<JobLife>,
}
```

声明引用：`skin_editor_audio::BuildRequest`, `skin_editor_infrastructure::audio_preparation::JobLife`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::AudioPreparationQueue (声明)`

## skin_editor_infrastructure::audio_preparation::JobLife

[源码:57](../../../desktop/infrastructure/src/audio_preparation.rs#L57) · struct

```rust
struct JobLife {
    cancelled: AtomicBool,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::Cancellation (声明)`, `skin_editor_infrastructure::FinishedPreparation (声明)`, `skin_editor_infrastructure::RunningPreparation (声明)`, `skin_editor_infrastructure::audio_preparation::Active (声明)`

## skin_editor_infrastructure::audio_preparation::MAX_PCM_CACHE_KEYS

[源码:17](../../../desktop/infrastructure/src/audio_preparation.rs#L17) · constant

```rust
const MAX_PCM_CACHE_KEYS: usize = 256;
```

## skin_editor_infrastructure::audio_preparation::Pending

[源码:66](../../../desktop/infrastructure/src/audio_preparation.rs#L66) · struct

```rust
struct Pending {
    request: BuildRequest,
    raw: RawBlob,
    reservation: PcmReservation,
    admitted: Instant,
}
```

声明引用：`skin_editor_audio::BuildRequest`, `skin_editor_audio::PcmReservation`, `skin_editor_infrastructure::RawBlob`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::AudioPreparationQueue (声明)`, `skin_editor_infrastructure::RunningPreparation (声明)`

## skin_editor_infrastructure::audio_preparation::PreparationResult

[源码:172](../../../desktop/infrastructure/src/audio_preparation.rs#L172) · type

```rust
pub type PreparationResult = (BuildRequest, Result<PreparedSound, PrepareFailure>);
```

声明引用：`skin_editor_audio::BuildRequest`, `skin_editor_audio::PrepareFailure`, `skin_editor_audio::PreparedSound`

## skin_editor_infrastructure::audio_supply::Control

[源码:97](../../../desktop/infrastructure/src/audio_supply.rs#L97) · enum

```rust
enum Control {
    Select(Projection),
    Background(BackgroundTrack),
    Scene {
        running: bool,
        mark: OutputMark,
    },
    Seek {
        frame: u64,
        mark: OutputMark,
    },
    StopAudition(OutputMark),
    Retry(AudioComponentId),
    Library {
        origin: Origin,
        plan: SoundPlan,
        mark: OutputMark,
    },
    Barrier,
    DiagnosticRecovery {
        trigger: Option<(Origin, Channel, AudioComponentId)>,
    },
    TriggerBatch {
        origin: Origin,
        entries: Vec<(Channel, AudioComponentId)>,
        stop_library: Option<OutputMark>,
    },
    Trigger {
        origin: Origin,
        lane: Channel,
        component: AudioComponentId,
        stop_library: Option<OutputMark>,
    },
}
```

声明引用：`skin_editor_audio::BackgroundTrack`, `skin_editor_audio::Channel`, `skin_editor_audio::Origin`, `skin_editor_audio::SoundPlan`, `skin_editor_infrastructure::audio_supply::Projection`, `skin_editor_infrastructure::output_fence::OutputMark`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::AudioSupplyClient::send`, `skin_editor_infrastructure::audio_supply::Request (声明)`

## skin_editor_infrastructure::audio_supply::Credit

[源码:37](../../../desktop/infrastructure/src/audio_supply.rs#L37) · struct

```rust
struct Credit(Arc<AtomicUsize>);
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::AudioSupplyClient::reserve`, `skin_editor_infrastructure::AudioSupplyClient::send`, `skin_editor_infrastructure::SupplyReceipt (声明)`, `skin_editor_infrastructure::audio_supply::Request (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Debug for Credit` — [源码:36](../../../desktop/infrastructure/src/audio_supply.rs#L36)
- `impl Drop for Credit` — [源码:38](../../../desktop/infrastructure/src/audio_supply.rs#L38)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for Credit | [源码:36](../../../desktop/infrastructure/src/audio_supply.rs#L36) |  |
| `fn drop(&mut self)` | impl Drop for Credit | [源码:39](../../../desktop/infrastructure/src/audio_supply.rs#L39) |  |

## skin_editor_infrastructure::audio_supply::Projection

[源码:91](../../../desktop/infrastructure/src/audio_supply.rs#L91) · struct

```rust
struct Projection {
    origin: Origin,
    plans: Vec<SoundPlan>,
    cancelled: Vec<DraftId>,
    mark: OutputMark,
}
```

声明引用：`skin_editor_audio::Origin`, `skin_editor_audio::SoundPlan`, `skin_editor_infrastructure::output_fence::OutputMark`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::audio_supply::Control (声明)`

## skin_editor_infrastructure::audio_supply::Request

[源码:131](../../../desktop/infrastructure/src/audio_supply.rs#L131) · struct

```rust
struct Request {
    id: u64,
    control: Control,
    credit: Credit,
}
```

声明引用：`skin_editor_infrastructure::audio_supply::Control`, `skin_editor_infrastructure::audio_supply::Credit`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::AudioSupplyClient (声明)`, `skin_editor_infrastructure::audio_supply::SupplyCore (声明)`, `skin_editor_infrastructure::audio_supply::SupplyCore::apply`, `skin_editor_infrastructure::audio_supply::SupplyCore::apply_output`, `skin_editor_infrastructure::audio_supply::SupplyCore::apply_owned`

## skin_editor_infrastructure::audio_supply::RetiredSupply

[源码:156](../../../desktop/infrastructure/src/audio_supply.rs#L156) · struct

```rust
struct RetiredSupply {
    core: SupplyCore,
    owner: Box<BackgroundOwner>,
}
```

声明引用：`skin_editor_infrastructure::audio_supply::SupplyCore`, `skin_editor_infrastructure::separated_output::BackgroundOwner`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::AudioSupplyClient (声明)`

## skin_editor_infrastructure::audio_supply::SupplyCore

[源码:748](../../../desktop/infrastructure/src/audio_supply.rs#L748) · struct

```rust
struct SupplyCore {
    controller: SkinAudioController,
    worker: PreparationWorker,
    background: Option<BackgroundTrack>,
    output: SupplyOutputControl,
    mark: OutputMark,
    selected: Option<Origin>,
    frozen: bool,
    freezing: Option<Request>,
    components: Vec<AudioComponentId>,
    library_intent: Option<AssetId>,
    last_preparation: Option<PreparationObservation>,
    max_supply_gap_micros: u64,
    separated_generated: u64,
    separated_position: u64,
    separated_primed: bool,
    background_frames: Option<u64>,
    separated_anchor_ready: bool,
    acknowledgments: Vec<(OutputMark, SupplyReceipt)>,
    external_drafts: Vec<DraftId>,
}
```

声明引用：`skin_editor_audio::BackgroundTrack`, `skin_editor_audio::Origin`, `skin_editor_infrastructure::PreparationObservation`, `skin_editor_infrastructure::PreparationWorker`, `skin_editor_infrastructure::SkinAudioController`, `skin_editor_infrastructure::SupplyReceipt`, `skin_editor_infrastructure::audio_supply::Request`, `skin_editor_infrastructure::output_fence::OutputMark`, `skin_editor_infrastructure::supply_output::SupplyOutputControl`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::AudioSupplyClient::spawn_owned`, `skin_editor_infrastructure::audio_supply::RetiredSupply (声明)`, `skin_editor_infrastructure::audio_supply::SupplyOutput::pump`

实现／约束（显式或 derive，无类继承）：

- `impl SupplyCore` — [源码:769](../../../desktop/infrastructure/src/audio_supply.rs#L769)
- `impl Drop for SupplyCore` — [源码:1403](../../../desktop/infrastructure/src/audio_supply.rs#L1403)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn can_idle(&mut self, separated: bool, voices: [usize; 2]) -> bool` | impl SupplyCore | [源码:770](../../../desktop/infrastructure/src/audio_supply.rs#L770) |  |
| `fn new_separated( background: impl Into<BackgroundInput>, rate: u32, channels: u16, budget: PcmBudget, ) -> Result<(Self, BackgroundOwner, Box<OutputConsumer>), SupplyError>` | impl SupplyCore | [源码:792](../../../desktop/infrastructure/src/audio_supply.rs#L792) | skin_editor_audio::PcmBudget, skin_editor_infrastructure::SupplyError, skin_editor_infrastructure::background_input::BackgroundInput, skin_editor_infrastructure::separated_output::BackgroundOwner, skin_editor_infrastructure::separated_output::OutputConsumer |
| `fn generated_background(&self) -> u64` | impl SupplyCore | [源码:842](../../../desktop/infrastructure/src/audio_supply.rs#L842) |  |
| `fn publish_progress(&self, published: &mut SupplyStatus)` | impl SupplyCore | [源码:851](../../../desktop/infrastructure/src/audio_supply.rs#L851) | skin_editor_infrastructure::SupplyStatus |
| `fn retained_drafts(&mut self) -> Vec<DraftId>` | impl SupplyCore | [源码:858](../../../desktop/infrastructure/src/audio_supply.rs#L858) |  |
| `fn apply_output( &mut self, output: &mut SupplyOutput, request: Request, store: &BlobStore, ) -> Option<SupplyReceipt>` | impl SupplyCore | [源码:869](../../../desktop/infrastructure/src/audio_supply.rs#L869) | skin_editor_infrastructure::BlobStore, skin_editor_infrastructure::SupplyReceipt, skin_editor_infrastructure::audio_supply::Request, skin_editor_infrastructure::audio_supply::SupplyOutput |
| `fn finish_acknowledgments(&mut self, outbound: &mut Vec<SupplyReceipt>)` | impl SupplyCore | [源码:1016](../../../desktop/infrastructure/src/audio_supply.rs#L1016) | skin_editor_infrastructure::SupplyReceipt |
| `fn apply_owned( &mut self, output: &mut OutputProducer, request: Request, store: &BlobStore, ) -> Option<SupplyReceipt>` | impl SupplyCore | [源码:1026](../../../desktop/infrastructure/src/audio_supply.rs#L1026) | skin_editor_infrastructure::BlobStore, skin_editor_infrastructure::OutputProducer, skin_editor_infrastructure::SupplyReceipt, skin_editor_infrastructure::audio_supply::Request |
| `fn new( output: &OutputProducer, rate: u32, channels: u16, budget: PcmBudget, ) -> Result<Self, SupplyError>` | impl SupplyCore | [源码:1069](../../../desktop/infrastructure/src/audio_supply.rs#L1069) | skin_editor_audio::PcmBudget, skin_editor_infrastructure::OutputProducer, skin_editor_infrastructure::SupplyError |
| `fn apply( &mut self, request: Request, store: &BlobStore, ) -> Option<SupplyReceipt>` | impl SupplyCore | [源码:1100](../../../desktop/infrastructure/src/audio_supply.rs#L1100) | skin_editor_infrastructure::BlobStore, skin_editor_infrastructure::SupplyReceipt, skin_editor_infrastructure::audio_supply::Request |
| `fn finish_freeze(&mut self) -> Result<Option<SupplyReceipt>, SupplyError>` | impl SupplyCore | [源码:1279](../../../desktop/infrastructure/src/audio_supply.rs#L1279) | skin_editor_infrastructure::SupplyError, skin_editor_infrastructure::SupplyReceipt |
| `fn prepare(&mut self) -> Result<bool, SupplyError>` | impl SupplyCore | [源码:1323](../../../desktop/infrastructure/src/audio_supply.rs#L1323) | skin_editor_infrastructure::SupplyError |
| `fn activate_library(&mut self) -> Result<(), SupplyError>` | impl SupplyCore | [源码:1338](../../../desktop/infrastructure/src/audio_supply.rs#L1338) | skin_editor_infrastructure::SupplyError |
| `fn status(&mut self) -> SupplyStatus` | impl SupplyCore | [源码:1360](../../../desktop/infrastructure/src/audio_supply.rs#L1360) | skin_editor_infrastructure::SupplyStatus |
| `fn voice_counts(&mut self) -> [usize; 2]` | impl SupplyCore | [源码:1395](../../../desktop/infrastructure/src/audio_supply.rs#L1395) |  |
| `fn drop(&mut self)` | impl Drop for SupplyCore | [源码:1404](../../../desktop/infrastructure/src/audio_supply.rs#L1404) |  |

## skin_editor_infrastructure::audio_supply::SupplyOutput

[源码:160](../../../desktop/infrastructure/src/audio_supply.rs#L160) · enum

```rust
enum SupplyOutput {
    Premixed(Box<OutputProducer>),
    Separated(Box<BackgroundOwner>),
}
```

声明引用：`skin_editor_infrastructure::OutputProducer`, `skin_editor_infrastructure::separated_output::BackgroundOwner`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::AudioSupplyClient::spawn_owned`, `skin_editor_infrastructure::audio_supply::SupplyCore::apply_output`

实现／约束（显式或 derive，无类继承）：

- `impl SupplyOutput` — [源码:164](../../../desktop/infrastructure/src/audio_supply.rs#L164)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn pump( &mut self, core: &mut SupplyCore, ahead: usize, ) -> Result<bool, SupplyError>` | impl SupplyOutput | [源码:165](../../../desktop/infrastructure/src/audio_supply.rs#L165) | skin_editor_infrastructure::SupplyError, skin_editor_infrastructure::audio_supply::SupplyCore |

## skin_editor_infrastructure::audio_supply::backend

[源码:200](../../../desktop/infrastructure/src/audio_supply.rs#L200) · fn

```rust
fn backend(error: impl Debug) -> SupplyError
```

声明引用：`skin_editor_infrastructure::SupplyError`

## skin_editor_infrastructure::audio_supply::origin

[源码:203](../../../desktop/infrastructure/src/audio_supply.rs#L203) · fn

```rust
fn origin(source: SkinAudioSource) -> Origin
```

声明引用：`skin_editor_audio::Origin`

## skin_editor_infrastructure::background_input::BackgroundInput

[源码:3](../../../desktop/infrastructure/src/background_input.rs#L3) · enum

```rust
pub(crate) enum BackgroundInput {
    Memory(BackgroundTrack),
    Buffered(BufferedBackground),
}
```

声明引用：`skin_editor_audio::BackgroundTrack`, `skin_editor_infrastructure::background_stream::BufferedBackground`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::audio_supply::SupplyCore::new_separated`, `skin_editor_infrastructure::separated_output::BackgroundOwner (声明)`, `skin_editor_infrastructure::separated_output::boundary_input (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl BackgroundInput` — [源码:17](../../../desktop/infrastructure/src/background_input.rs#L17)
- `impl From<BackgroundTrack> for BackgroundInput` — [源码:7](../../../desktop/infrastructure/src/background_input.rs#L7)
- `impl From<BufferedBackground> for BackgroundInput` — [源码:12](../../../desktop/infrastructure/src/background_input.rs#L12)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn format(&self) -> (u32, u16)` | impl BackgroundInput | [源码:18](../../../desktop/infrastructure/src/background_input.rs#L18) |  |
| `pub fn frames(&self) -> u64` | impl BackgroundInput | [源码:24](../../../desktop/infrastructure/src/background_input.rs#L24) |  |
| `pub fn position(&self) -> u64` | impl BackgroundInput | [源码:30](../../../desktop/infrastructure/src/background_input.rs#L30) |  |
| `pub fn budget(&self) -> &PcmBudget` | impl BackgroundInput | [源码:36](../../../desktop/infrastructure/src/background_input.rs#L36) | skin_editor_audio::PcmBudget |
| `pub fn restore_position(&mut self, frame: u64) -> Result<(), OutputError>` | impl BackgroundInput | [源码:42](../../../desktop/infrastructure/src/background_input.rs#L42) | skin_editor_infrastructure::separated_output::OutputError |
| `pub fn set_running(&mut self, running: bool)` | impl BackgroundInput | [源码:52](../../../desktop/infrastructure/src/background_input.rs#L52) |  |
| `pub fn next_frame(&mut self) -> Result<Option<(u64, [f32; 2])>, OutputError>` | impl BackgroundInput | [源码:57](../../../desktop/infrastructure/src/background_input.rs#L57) | skin_editor_infrastructure::separated_output::OutputError |
| `fn from(t: BackgroundTrack) -> Self` | impl From<BackgroundTrack> for BackgroundInput | [源码:8](../../../desktop/infrastructure/src/background_input.rs#L8) | skin_editor_audio::BackgroundTrack |
| `fn from(t: BufferedBackground) -> Self` | impl From<BufferedBackground> for BackgroundInput | [源码:13](../../../desktop/infrastructure/src/background_input.rs#L13) | skin_editor_infrastructure::background_stream::BufferedBackground |

## skin_editor_infrastructure::background_stream::BLOCKS

[源码:161](../../../desktop/infrastructure/src/background_stream.rs#L161) · constant

```rust
const BLOCKS: usize = _; // 4usize
```

## skin_editor_infrastructure::background_stream::Block

[源码:162](../../../desktop/infrastructure/src/background_stream.rs#L162) · struct

```rust
struct Block {
    version: u64,
    first: u64,
    frames: usize,
    pcm: PcmBuffer,
}
```

声明引用：`skin_editor_audio::PcmBuffer`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::background_stream::BufferedBackground (声明)`, `skin_editor_infrastructure::background_stream::BufferedBackground::recycle`

## skin_editor_infrastructure::background_stream::BufferedBackground

[源码:168](../../../desktop/infrastructure/src/background_stream.rs#L168) · struct

```rust
pub struct BufferedBackground {
    state: Arc<State>,
    rx: Consumer<Block>,
    free: Producer<Block>,
    current: Option<Block>,
    cursor: usize,
    thread: Option<JoinHandle<()>>,
    budget: PcmBudget,
    observation: StreamObservation,
    rate: u32,
    channels: u16,
    frames: u64,
    next: u64,
    version: u64,
}
```

声明引用：`skin_editor_audio::PcmBudget`, `skin_editor_infrastructure::background_stream::Block`, `skin_editor_infrastructure::background_stream::State`, `skin_editor_infrastructure::background_stream::StreamObservation`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::editor_audio::EditorAudio (声明)`, `skin_editor_desktop::stream_fixture::open (声明)`, `skin_editor_infrastructure::AudioSupplyClient::spawn_streamed`, `skin_editor_infrastructure::background_input::BackgroundInput (声明)`, `skin_editor_infrastructure::background_input::BackgroundInput::from-1`, `skin_editor_infrastructure::background_stream::BufferedBackground::from`

实现／约束（显式或 derive，无类继承）：

- `impl BufferedBackground` — [源码:183](../../../desktop/infrastructure/src/background_stream.rs#L183)
- `impl Drop for BufferedBackground` — [源码:413](../../../desktop/infrastructure/src/background_stream.rs#L413)
- `impl From<BufferedBackground> for BackgroundInput` — [源码:12](../../../desktop/infrastructure/src/background_input.rs#L12)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn open( reader: impl ReadSeek + 'static, budget: &PcmBudget, ) -> Result<Self, StreamError>` | impl BufferedBackground | [源码:186](../../../desktop/infrastructure/src/background_stream.rs#L186) | skin_editor_audio::PcmBudget, skin_editor_infrastructure::background_stream::ReadSeek, skin_editor_infrastructure::background_stream::StreamError |
| `fn spawn(wave: Wave, budget: &PcmBudget) -> Result<Self, StreamError>` | impl BufferedBackground | [源码:189](../../../desktop/infrastructure/src/background_stream.rs#L189) | skin_editor_audio::PcmBudget, skin_editor_infrastructure::background_stream::StreamError, skin_editor_infrastructure::background_stream::Wave |
| `fn spawn_inner(wave: Wave, budget: &PcmBudget) -> Result<Self, StreamError>` | impl BufferedBackground | [源码:197](../../../desktop/infrastructure/src/background_stream.rs#L197) | skin_editor_audio::PcmBudget, skin_editor_infrastructure::background_stream::StreamError, skin_editor_infrastructure::background_stream::Wave |
| `pub fn observation(&self) -> StreamObservation` | impl BufferedBackground | [源码:324](../../../desktop/infrastructure/src/background_stream.rs#L324) | skin_editor_infrastructure::background_stream::StreamObservation |
| `pub fn format(&self) -> (u32, u16)` | impl BufferedBackground | [源码:327](../../../desktop/infrastructure/src/background_stream.rs#L327) |  |
| `pub fn frames(&self) -> u64` | impl BufferedBackground | [源码:330](../../../desktop/infrastructure/src/background_stream.rs#L330) |  |
| `pub fn position(&self) -> u64` | impl BufferedBackground | [源码:333](../../../desktop/infrastructure/src/background_stream.rs#L333) |  |
| `pub(crate) fn budget(&self) -> &PcmBudget` | impl BufferedBackground | [源码:336](../../../desktop/infrastructure/src/background_stream.rs#L336) | skin_editor_audio::PcmBudget |
| `fn recycle(&mut self, block: Block) -> Result<(), StreamError>` | impl BufferedBackground | [源码:339](../../../desktop/infrastructure/src/background_stream.rs#L339) | skin_editor_infrastructure::background_stream::Block, skin_editor_infrastructure::background_stream::StreamError |
| `pub fn restore_position(&mut self, frame: u64) -> Result<(), StreamError>` | impl BufferedBackground | [源码:346](../../../desktop/infrastructure/src/background_stream.rs#L346) | skin_editor_infrastructure::background_stream::StreamError |
| `pub(crate) fn next_frame( &mut self, ) -> Result<Option<(u64, [f32; 2])>, StreamError>` | impl BufferedBackground | [源码:374](../../../desktop/infrastructure/src/background_stream.rs#L374) | skin_editor_infrastructure::background_stream::StreamError |
| `fn drop(&mut self)` | impl Drop for BufferedBackground | [源码:414](../../../desktop/infrastructure/src/background_stream.rs#L414) |  |
| `fn from(t: BufferedBackground) -> Self` | impl From<BufferedBackground> for BackgroundInput | [源码:13](../../../desktop/infrastructure/src/background_input.rs#L13) | skin_editor_infrastructure::background_stream::BufferedBackground |

## skin_editor_infrastructure::background_stream::READ_BYTES

[源码:18](../../../desktop/infrastructure/src/background_stream.rs#L18) · constant

```rust
const READ_BYTES: usize = _; // 8_192usize
```

## skin_editor_infrastructure::background_stream::READ_FRAMES

[源码:17](../../../desktop/infrastructure/src/background_stream.rs#L17) · constant

```rust
const READ_FRAMES: usize = 2048;
```

## skin_editor_infrastructure::background_stream::ReadSeek

[源码:28](../../../desktop/infrastructure/src/background_stream.rs#L28) · trait

```rust
pub trait ReadSeek:
    Read
    + Seek
    + Send { }
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::background_stream::BufferedBackground::open`, `skin_editor_infrastructure::background_stream::Wave (声明)`, `skin_editor_infrastructure::background_stream::Wave::open`

实现／约束（显式或 derive，无类继承）：

- `impl<T: Read + Seek + Send> ReadSeek for T` — [源码:29](../../../desktop/infrastructure/src/background_stream.rs#L29)

## skin_editor_infrastructure::background_stream::SOURCE_RESERVE

[源码:16](../../../desktop/infrastructure/src/background_stream.rs#L16) · constant

```rust
pub const SOURCE_RESERVE: usize = 8192;
```

## skin_editor_infrastructure::background_stream::State

[源码:116](../../../desktop/infrastructure/src/background_stream.rs#L116) · struct

```rust
struct State {
    stamp: AtomicU64,
    position: AtomicU64,
    stop: AtomicBool,
    fault: AtomicBool,
    bytes: AtomicU64,
    discarded: AtomicU64,
    seeks: AtomicU64,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::background_stream::BufferedBackground (声明)`, `skin_editor_infrastructure::background_stream::StreamObservation (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl State` — [源码:125](../../../desktop/infrastructure/src/background_stream.rs#L125)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn requested(&self) -> Option<(u64, u64)>` | impl State | [源码:126](../../../desktop/infrastructure/src/background_stream.rs#L126) |  |

## skin_editor_infrastructure::background_stream::StreamError

[源码:20](../../../desktop/infrastructure/src/background_stream.rs#L20) · enum

```rust
pub enum StreamError {
    InvalidWave,
    Unsupported,
    Io,
    ResourceLimit,
    Closed,
    Protocol,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::background_stream::BufferedBackground::next_frame`, `skin_editor_infrastructure::background_stream::BufferedBackground::open`, `skin_editor_infrastructure::background_stream::BufferedBackground::recycle`, `skin_editor_infrastructure::background_stream::BufferedBackground::restore_position`, `skin_editor_infrastructure::background_stream::BufferedBackground::spawn`, `skin_editor_infrastructure::background_stream::BufferedBackground::spawn_inner`, `skin_editor_infrastructure::background_stream::StreamError::clone`, `skin_editor_infrastructure::background_stream::StreamError::eq`, `skin_editor_infrastructure::background_stream::Wave::open`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for StreamError` — [源码:19](../../../desktop/infrastructure/src/background_stream.rs#L19)
- `impl Debug for StreamError` — [源码:19](../../../desktop/infrastructure/src/background_stream.rs#L19)
- `impl PartialEq for StreamError` — [源码:19](../../../desktop/infrastructure/src/background_stream.rs#L19)
- `impl Copy for StreamError` — [源码:19](../../../desktop/infrastructure/src/background_stream.rs#L19)
- `impl Eq for StreamError` — [源码:19](../../../desktop/infrastructure/src/background_stream.rs#L19)
- `impl StructuralPartialEq for StreamError` — [源码:19](../../../desktop/infrastructure/src/background_stream.rs#L19)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> StreamError` | impl Clone for StreamError | [源码:19](../../../desktop/infrastructure/src/background_stream.rs#L19) | skin_editor_infrastructure::background_stream::StreamError |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for StreamError | [源码:19](../../../desktop/infrastructure/src/background_stream.rs#L19) |  |
| `fn eq(&self, other: &StreamError) -> bool` | impl PartialEq for StreamError | [源码:19](../../../desktop/infrastructure/src/background_stream.rs#L19) | skin_editor_infrastructure::background_stream::StreamError |

## skin_editor_infrastructure::background_stream::StreamObservation

[源码:136](../../../desktop/infrastructure/src/background_stream.rs#L136) · struct

```rust
pub struct StreamObservation {
    state: Arc<State>,
    raw: Arc<ByteBudget>,
}
```

声明引用：`skin_editor_infrastructure::background_stream::State`, `skin_editor_infrastructure::byte_budget::ByteBudget`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::editor_audio::combined::Run (声明)`, `skin_editor_desktop::editor_audio::cycles::Counters (声明)`, `skin_editor_desktop::editor_audio::gui_verification::Run (声明)`, `skin_editor_desktop::editor_audio::stability::Run (声明)`, `skin_editor_desktop::stream_fixture::open (声明)`, `skin_editor_infrastructure::background_stream::BufferedBackground (声明)`, `skin_editor_infrastructure::background_stream::BufferedBackground::observation`, `skin_editor_infrastructure::background_stream::StreamObservation::clone`

实现／约束（显式或 derive，无类继承）：

- `impl StreamObservation` — [源码:149](../../../desktop/infrastructure/src/background_stream.rs#L149)
- `impl Clone for StreamObservation` — [源码:135](../../../desktop/infrastructure/src/background_stream.rs#L135)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn stats(&self) -> StreamStats` | impl StreamObservation | [源码:150](../../../desktop/infrastructure/src/background_stream.rs#L150) | skin_editor_infrastructure::background_stream::StreamStats |
| `fn clone(&self) -> StreamObservation` | impl Clone for StreamObservation | [源码:135](../../../desktop/infrastructure/src/background_stream.rs#L135) | skin_editor_infrastructure::background_stream::StreamObservation |

## skin_editor_infrastructure::background_stream::StreamStats

[源码:141](../../../desktop/infrastructure/src/background_stream.rs#L141) · struct

```rust
pub struct StreamStats {
    pub read_bytes: u64,
    pub discarded_frames: u64,
    pub seeks: u64,
    pub raw_bytes: usize,
    pub raw_peak_bytes: usize,
    pub fault: bool,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::background_stream::StreamObservation::stats`, `skin_editor_infrastructure::background_stream::StreamStats::clone`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for StreamStats` — [源码:140](../../../desktop/infrastructure/src/background_stream.rs#L140)
- `impl Debug for StreamStats` — [源码:140](../../../desktop/infrastructure/src/background_stream.rs#L140)
- `impl Copy for StreamStats` — [源码:140](../../../desktop/infrastructure/src/background_stream.rs#L140)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> StreamStats` | impl Clone for StreamStats | [源码:140](../../../desktop/infrastructure/src/background_stream.rs#L140) | skin_editor_infrastructure::background_stream::StreamStats |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for StreamStats | [源码:140](../../../desktop/infrastructure/src/background_stream.rs#L140) |  |

## skin_editor_infrastructure::background_stream::Wave

[源码:30](../../../desktop/infrastructure/src/background_stream.rs#L30) · struct

```rust
struct Wave {
    reader: Box<dyn ReadSeek>,
    rate: u32,
    channels: u16,
    frames: u64,
    data: u64,
}
```

声明引用：`skin_editor_infrastructure::background_stream::ReadSeek`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::background_stream::BufferedBackground::spawn`, `skin_editor_infrastructure::background_stream::BufferedBackground::spawn_inner`

实现／约束（显式或 derive，无类继承）：

- `impl Wave` — [源码:37](../../../desktop/infrastructure/src/background_stream.rs#L37)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn open(reader: Box<dyn ReadSeek>) -> Result<Self, StreamError>` | impl Wave | [源码:38](../../../desktop/infrastructure/src/background_stream.rs#L38) | skin_editor_infrastructure::background_stream::ReadSeek, skin_editor_infrastructure::background_stream::StreamError |

## skin_editor_infrastructure::background_stream::park_reader

[源码:423](../../../desktop/infrastructure/src/background_stream.rs#L423) · fn

```rust
fn park_reader()
```

## skin_editor_infrastructure::blob_import

[源码:75](../../../desktop/infrastructure/src/lib.rs#L75) · fn

```rust
pub(crate) fn blob_import(error: BlobError) -> ImportError
```

声明引用：`skin_editor_infrastructure::BlobError`

## skin_editor_infrastructure::blobs::Entry

[源码:28](../../../desktop/infrastructure/src/blobs.rs#L28) · struct

```rust
struct Entry {
    reference: BlobRef,
    bytes: Vec<u8>,
    metadata: Mutex<Option<AudioMetadata>>,
    _lease: ByteLease,
}
```

声明引用：`skin_editor_infrastructure::byte_budget::ByteLease`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::BlobStore (声明)`, `skin_editor_infrastructure::RawBlob (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl BlobLease for Entry` — [源码:34](../../../desktop/infrastructure/src/blobs.rs#L34)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn reference(&self) -> BlobRef` | impl BlobLease for Entry | [源码:35](../../../desktop/infrastructure/src/blobs.rs#L35) |  |

## skin_editor_infrastructure::byte_budget::ByteBudget

[源码:6](../../../desktop/infrastructure/src/byte_budget.rs#L6) · struct

```rust
pub(crate) struct ByteBudget {
    limit: usize,
    used: AtomicUsize,
    peak: AtomicUsize,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::BlobStore (声明)`, `skin_editor_infrastructure::ByteObservation (声明)`, `skin_editor_infrastructure::Images (声明)`, `skin_editor_infrastructure::background_stream::StreamObservation (声明)`, `skin_editor_infrastructure::byte_budget::ByteLease (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl ByteBudget` — [源码:27](../../../desktop/infrastructure/src/byte_budget.rs#L27)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn observation(self: &Arc<Self>) -> ByteObservation` | impl ByteBudget | [源码:28](../../../desktop/infrastructure/src/byte_budget.rs#L28) | skin_editor_infrastructure::ByteObservation |
| `pub fn new(limit: usize) -> Arc<Self>` | impl ByteBudget | [源码:31](../../../desktop/infrastructure/src/byte_budget.rs#L31) |  |
| `pub fn reserve(self: &Arc<Self>, bytes: usize) -> Option<ByteLease>` | impl ByteBudget | [源码:38](../../../desktop/infrastructure/src/byte_budget.rs#L38) | skin_editor_infrastructure::byte_budget::ByteLease |
| `pub fn used(&self) -> usize` | impl ByteBudget | [源码:51](../../../desktop/infrastructure/src/byte_budget.rs#L51) |  |
| `pub fn peak(&self) -> usize` | impl ByteBudget | [源码:54](../../../desktop/infrastructure/src/byte_budget.rs#L54) |  |

## skin_editor_infrastructure::byte_budget::ByteLease

[源码:58](../../../desktop/infrastructure/src/byte_budget.rs#L58) · struct

```rust
pub(crate) struct ByteLease {
    budget: Arc<ByteBudget>,
    bytes: usize,
}
```

声明引用：`skin_editor_infrastructure::byte_budget::ByteBudget`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::BlobStore::insert`, `skin_editor_infrastructure::Images::reserve_pixels`, `skin_editor_infrastructure::blobs::Entry (声明)`, `skin_editor_infrastructure::byte_budget::ByteBudget::reserve`

实现／约束（显式或 derive，无类继承）：

- `impl Drop for ByteLease` — [源码:62](../../../desktop/infrastructure/src/byte_budget.rs#L62)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn drop(&mut self)` | impl Drop for ByteLease | [源码:63](../../../desktop/infrastructure/src/byte_budget.rs#L63) |  |

## skin_editor_infrastructure::callback_audit::STATE

[源码:9](../../../desktop/infrastructure/src/callback_audit.rs#L9) · constant

```rust
const STATE: LocalKey<Cell<State>>;
```

声明引用：`skin_editor_infrastructure::callback_audit::State`

## skin_editor_infrastructure::callback_audit::Scope

[源码:10](../../../desktop/infrastructure/src/callback_audit.rs#L10) · struct

```rust
pub(crate) struct Scope {
    previous: State,
    finished: bool,
}
```

声明引用：`skin_editor_infrastructure::callback_audit::State`

实现／约束（显式或 derive，无类继承）：

- `impl Scope` — [源码:26](../../../desktop/infrastructure/src/callback_audit.rs#L26)
- `impl Drop for Scope` — [源码:54](../../../desktop/infrastructure/src/callback_audit.rs#L54)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn enter() -> Self` | impl Scope | [源码:27](../../../desktop/infrastructure/src/callback_audit.rs#L27) |  |
| `pub fn finish(self) -> u64` | impl Scope | [源码:41](../../../desktop/infrastructure/src/callback_audit.rs#L41) |  |
| `fn drop(&mut self)` | impl Drop for Scope | [源码:55](../../../desktop/infrastructure/src/callback_audit.rs#L55) |  |

## skin_editor_infrastructure::callback_audit::State

[源码:5](../../../desktop/infrastructure/src/callback_audit.rs#L5) · struct

```rust
struct State {
    depth: u32,
    releases: u64,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::callback_audit::STATE (声明)`, `skin_editor_infrastructure::callback_audit::Scope (声明)`, `skin_editor_infrastructure::callback_audit::State::clone`, `skin_editor_infrastructure::callback_audit::State::default`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for State` — [源码:4](../../../desktop/infrastructure/src/callback_audit.rs#L4)
- `impl Default for State` — [源码:4](../../../desktop/infrastructure/src/callback_audit.rs#L4)
- `impl Copy for State` — [源码:4](../../../desktop/infrastructure/src/callback_audit.rs#L4)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> State` | impl Clone for State | [源码:4](../../../desktop/infrastructure/src/callback_audit.rs#L4) | skin_editor_infrastructure::callback_audit::State |
| `fn default() -> State` | impl Default for State | [源码:4](../../../desktop/infrastructure/src/callback_audit.rs#L4) | skin_editor_infrastructure::callback_audit::State |

## skin_editor_infrastructure::callback_audit::active

[源码:14](../../../desktop/infrastructure/src/callback_audit.rs#L14) · fn

```rust
pub(crate) fn active() -> bool
```

## skin_editor_infrastructure::callback_audit::released

[源码:17](../../../desktop/infrastructure/src/callback_audit.rs#L17) · fn

```rust
pub(crate) fn released()
```

## skin_editor_infrastructure::native_audio::CallbackObservation

[源码:22](../../../desktop/infrastructure/src/native_audio.rs#L22) · struct

```rust
struct CallbackObservation([AtomicU64; 4]);
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::native_audio::Shared (声明)`

## skin_editor_infrastructure::native_audio::Frame

[源码:25](../../../desktop/infrastructure/src/native_audio.rs#L25) · struct

```rust
struct Frame {
    scene: [f32; 2],
    audition: [f32; 2],
    epochs: [u64; 2],
    background: [f32; 2],
    background_epoch: u64,
    background_position: u64,
    serial: u64,
    activity: u8,
    voices: [u64; 2],
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::OutputConsumer (声明)`, `skin_editor_infrastructure::OutputProducer (声明)`, `skin_editor_infrastructure::native_audio::Frame::clone`, `skin_editor_infrastructure::native_audio::Frame::default`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for Frame` — [源码:24](../../../desktop/infrastructure/src/native_audio.rs#L24)
- `impl Default for Frame` — [源码:24](../../../desktop/infrastructure/src/native_audio.rs#L24)
- `impl Copy for Frame` — [源码:24](../../../desktop/infrastructure/src/native_audio.rs#L24)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> Frame` | impl Clone for Frame | [源码:24](../../../desktop/infrastructure/src/native_audio.rs#L24) | skin_editor_infrastructure::native_audio::Frame |
| `fn default() -> Frame` | impl Default for Frame | [源码:24](../../../desktop/infrastructure/src/native_audio.rs#L24) | skin_editor_infrastructure::native_audio::Frame |

## skin_editor_infrastructure::native_audio::OutputSafety

[源码:294](../../../desktop/infrastructure/src/native_audio.rs#L294) · struct

```rust
pub(crate) struct OutputSafety(Weak<Shared>);
```

声明引用：`skin_editor_infrastructure::native_audio::Shared`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::AudioPlayback (声明)`, `skin_editor_infrastructure::OutputProducer::safety`

实现／约束（显式或 derive，无类继承）：

- `impl OutputSafety` — [源码:295](../../../desktop/infrastructure/src/native_audio.rs#L295)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn is_for(&self, output: &OutputProducer) -> bool` | impl OutputSafety | [源码:296](../../../desktop/infrastructure/src/native_audio.rs#L296) | skin_editor_infrastructure::OutputProducer |
| `pub fn close(&self)` | impl OutputSafety | [源码:299](../../../desktop/infrastructure/src/native_audio.rs#L299) |  |

## skin_editor_infrastructure::native_audio::STARTUP_OBSERVATIONS

[源码:14](../../../desktop/infrastructure/src/native_audio.rs#L14) · constant

```rust
const STARTUP_OBSERVATIONS: usize = 32;
```

## skin_editor_infrastructure::native_audio::Shared

[源码:36](../../../desktop/infrastructure/src/native_audio.rs#L36) · struct

```rust
struct Shared {
    capture: SubmissionCapture,
    monitor_volume: AtomicU64,
    background_epoch: AtomicU64,
    background_ack: AtomicU64,
    background_next: AtomicU64,
    background_gaps: AtomicU64,
    background_frames: AtomicU64,
    epochs: [AtomicU64; 2],
    acknowledged: [AtomicU64; 2],
    callbacks: AtomicU64,
    max_callback_frames: AtomicU64,
    frames: AtomicU64,
    empty: AtomicU64,
    empty_callbacks: AtomicU64,
    max_empty_callback_frames: AtomicU64,
    stale: AtomicU64,
    nonzero: AtomicU64,
    lane_nonzero: [AtomicU64; 2],
    lane_voices: [AtomicU64; 2],
    capture_startup: AtomicBool,
    startup: [CallbackObservation; 32],
    underruns: [UnderrunObservation; 32],
    supply_pass: AtomicU64,
    recovery_passes: AtomicU64,
    diagnostic_recovery_passes: AtomicU64,
    diagnostic_refill_passes: AtomicU64,
    diagnostic_refill_frames: AtomicU64,
    max_recovery_target: AtomicU64,
    idle_cutover: AtomicU64,
    idle_guard: AtomicU64,
    idle_sequence: AtomicU64,
    retimed_idle_frames: AtomicU64,
    failed: AtomicBool,
    closed: AtomicBool,
    _ring_payload: PcmReservation,
}
```

声明引用：`skin_editor_audio::PcmReservation`, `skin_editor_infrastructure::native_audio::CallbackObservation`, `skin_editor_infrastructure::native_audio::UnderrunObservation`, `skin_editor_infrastructure::submission_trace::SubmissionCapture`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::OutputConsumer (声明)`, `skin_editor_infrastructure::OutputControl (声明)`, `skin_editor_infrastructure::OutputProducer (声明)`, `skin_editor_infrastructure::native_audio::OutputSafety (声明)`, `skin_editor_infrastructure::native_audio::shared_stats (声明)`

## skin_editor_infrastructure::native_audio::UnderrunObservation

[源码:23](../../../desktop/infrastructure/src/native_audio.rs#L23) · struct

```rust
struct UnderrunObservation([AtomicU64; 6]);
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::native_audio::Shared (声明)`

## skin_editor_infrastructure::native_audio::shared_stats

[源码:473](../../../desktop/infrastructure/src/native_audio.rs#L473) · fn

```rust
fn shared_stats(s: &Shared) -> OutputStats
```

声明引用：`skin_editor_infrastructure::OutputStats`, `skin_editor_infrastructure::native_audio::Shared`

## skin_editor_infrastructure::output_boundary

[源码:305](../../../desktop/infrastructure/src/native_audio.rs#L305) · fn

```rust
pub fn output_boundary(
    budget: &PcmBudget,
    rate: u32,
    channels: u16,
) -> Result<(OutputProducer, OutputConsumer), String>
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_infrastructure::native_audio::output_boundary`

声明引用：`skin_editor_audio::PcmBudget`, `skin_editor_infrastructure::OutputConsumer`, `skin_editor_infrastructure::OutputProducer`

## skin_editor_infrastructure::output_fence::OutputMark

[源码:13](../../../desktop/infrastructure/src/output_fence.rs#L13) · struct

```rust
pub(crate) struct OutputMark {
    pub skin: OutputFence,
    pub background: u64,
}
```

声明引用：`skin_editor_infrastructure::OutputFence`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::OutputControl::background_acknowledged`, `skin_editor_infrastructure::OutputControl::invalidate_scene`, `skin_editor_infrastructure::OutputControl::invalidate_skin`, `skin_editor_infrastructure::OutputControl::mark`, `skin_editor_infrastructure::OutputProducer::pump_timeline`, `skin_editor_infrastructure::audio_supply::Control (声明)`, `skin_editor_infrastructure::audio_supply::Projection (声明)`, `skin_editor_infrastructure::audio_supply::SupplyCore (声明)`, `skin_editor_infrastructure::output_fence::OutputMark::clone`, `skin_editor_infrastructure::separated_output::BackgroundOwner::apply_mark`, `skin_editor_infrastructure::separated_output::control::Control::acknowledged`, `skin_editor_infrastructure::separated_output::control::Control::background_acknowledged`, `skin_editor_infrastructure::separated_output::control::Control::invalidate`, `skin_editor_infrastructure::separated_output::control::Control::mark`, `skin_editor_infrastructure::separated_output::control::State::acknowledge`, `skin_editor_infrastructure::separated_output::control::State::applied`, `skin_editor_infrastructure::separated_output::control::State::apply`, `skin_editor_infrastructure::separated_output::control::State::desired`, `skin_editor_infrastructure::separated_output::control::State::read`, `skin_editor_infrastructure::supply_output::SupplyOutputControl::acknowledged`, `skin_editor_infrastructure::supply_output::SupplyOutputControl::background_acknowledged`, `skin_editor_infrastructure::supply_output::SupplyOutputControl::invalidate_scene`, `skin_editor_infrastructure::supply_output::SupplyOutputControl::invalidate_skin`, `skin_editor_infrastructure::supply_output::SupplyOutputControl::mark`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for OutputMark` — [源码:12](../../../desktop/infrastructure/src/output_fence.rs#L12)
- `impl Copy for OutputMark` — [源码:12](../../../desktop/infrastructure/src/output_fence.rs#L12)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> OutputMark` | impl Clone for OutputMark | [源码:12](../../../desktop/infrastructure/src/output_fence.rs#L12) | skin_editor_infrastructure::output_fence::OutputMark |

## skin_editor_infrastructure::separated_native::Callback

[源码:145](../../../desktop/infrastructure/src/separated_native.rs#L145) · struct

```rust
struct Callback {
    endpoint: Option<Box<OutputConsumer>>,
    returned: SyncSender<StoppedOutput>,
    first: Producer<FirstCopy>,
    shared: Arc<Shared>,
    channels: usize,
    work_accum: [u64; 6],
    direct_return: bool,
}
```

声明引用：`skin_editor_infrastructure::separated_native::FirstCopy`, `skin_editor_infrastructure::separated_native::Shared`, `skin_editor_infrastructure::separated_output::OutputConsumer`, `skin_editor_infrastructure::separated_output::StoppedOutput`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::separated_native::ControlledCallback (声明)`, `skin_editor_infrastructure::separated_native::executor_gated (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Callback` — [源码:176](../../../desktop/infrastructure/src/separated_native.rs#L176)
- `impl Drop for Callback` — [源码:158](../../../desktop/infrastructure/src/separated_native.rs#L158)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn render(&mut self, output: &mut [f32], stamp: u64)` | impl Callback | [源码:177](../../../desktop/infrastructure/src/separated_native.rs#L177) |  |
| `fn render_inner( &mut self, output: &mut [f32], stamp: u64, capture: bool, parts: &mut [u64; 3], )` | impl Callback | [源码:205](../../../desktop/infrastructure/src/separated_native.rs#L205) |  |
| `fn drop(&mut self)` | impl Drop for Callback | [源码:159](../../../desktop/infrastructure/src/separated_native.rs#L159) |  |

## skin_editor_infrastructure::separated_native::ControlledCallback

[源码:396](../../../desktop/infrastructure/src/separated_native.rs#L396) · struct

```rust
pub struct ControlledCallback(Callback);
```

声明引用：`skin_editor_infrastructure::separated_native::Callback`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::separated_native::NativeExecutor::controlled_return`

实现／约束（显式或 derive，无类继承）：

- `impl ControlledCallback` — [源码:397](../../../desktop/infrastructure/src/separated_native.rs#L397)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn render(&mut self, output: &mut [f32], stamp: u64)` | impl ControlledCallback | [源码:398](../../../desktop/infrastructure/src/separated_native.rs#L398) |  |

## skin_editor_infrastructure::separated_native::EndpointReturnError

[源码:350](../../../desktop/infrastructure/src/separated_native.rs#L350) · enum

```rust
pub enum EndpointReturnError {
    Pending,
    Disconnected,
    AlreadyReceived,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::separated_native::EndpointReturnError::clone`, `skin_editor_infrastructure::separated_native::EndpointReturnError::eq`, `skin_editor_infrastructure::separated_native::NativeExecutor::try_stop`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for EndpointReturnError` — [源码:349](../../../desktop/infrastructure/src/separated_native.rs#L349)
- `impl Debug for EndpointReturnError` — [源码:349](../../../desktop/infrastructure/src/separated_native.rs#L349)
- `impl PartialEq for EndpointReturnError` — [源码:349](../../../desktop/infrastructure/src/separated_native.rs#L349)
- `impl Copy for EndpointReturnError` — [源码:349](../../../desktop/infrastructure/src/separated_native.rs#L349)
- `impl Eq for EndpointReturnError` — [源码:349](../../../desktop/infrastructure/src/separated_native.rs#L349)
- `impl StructuralPartialEq for EndpointReturnError` — [源码:349](../../../desktop/infrastructure/src/separated_native.rs#L349)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> EndpointReturnError` | impl Clone for EndpointReturnError | [源码:349](../../../desktop/infrastructure/src/separated_native.rs#L349) | skin_editor_infrastructure::separated_native::EndpointReturnError |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for EndpointReturnError | [源码:349](../../../desktop/infrastructure/src/separated_native.rs#L349) |  |
| `fn eq(&self, other: &EndpointReturnError) -> bool` | impl PartialEq for EndpointReturnError | [源码:349](../../../desktop/infrastructure/src/separated_native.rs#L349) | skin_editor_infrastructure::separated_native::EndpointReturnError |

## skin_editor_infrastructure::separated_native::FirstCopy

[源码:15](../../../desktop/infrastructure/src/separated_native.rs#L15) · struct

```rust
pub struct FirstCopy {
    pub voice: u64,
    pub channel: Channel,
    pub callback_nanos: u64,
    pub callback_ordinal: u64,
    pub offset_frames: usize,
    pub callback_frames: usize,
}
```

声明引用：`skin_editor_audio::Channel`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::editor_audio::combined::probes::Complete (声明)`, `skin_editor_desktop::editor_audio::combined::probes::Probes::observe`, `skin_editor_desktop::editor_audio::gui_verification::Observations (声明)`, `skin_editor_infrastructure::separated_native::Callback (声明)`, `skin_editor_infrastructure::separated_native::FirstCopy::clone`, `skin_editor_infrastructure::separated_native::FirstObserver (声明)`, `skin_editor_infrastructure::separated_native::FirstObserver::next_first`, `skin_editor_infrastructure::separated_native::NativeExecutor::next_first`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for FirstCopy` — [源码:14](../../../desktop/infrastructure/src/separated_native.rs#L14)
- `impl Debug for FirstCopy` — [源码:14](../../../desktop/infrastructure/src/separated_native.rs#L14)
- `impl Copy for FirstCopy` — [源码:14](../../../desktop/infrastructure/src/separated_native.rs#L14)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> FirstCopy` | impl Clone for FirstCopy | [源码:14](../../../desktop/infrastructure/src/separated_native.rs#L14) | skin_editor_infrastructure::separated_native::FirstCopy |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for FirstCopy | [源码:14](../../../desktop/infrastructure/src/separated_native.rs#L14) |  |

## skin_editor_infrastructure::separated_native::FirstObserver

[源码:121](../../../desktop/infrastructure/src/separated_native.rs#L121) · struct

```rust
pub struct FirstObserver(Consumer<FirstCopy>);
```

声明引用：`skin_editor_infrastructure::separated_native::FirstCopy`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::editor_audio::gui_verification::Observations::new`, `skin_editor_desktop::editor_audio::gui_verification::Observations::with_capacity`, `skin_editor_infrastructure::separated_native::NativeExecutor (声明)`, `skin_editor_infrastructure::separated_native::NativeExecutor::take_first_observer`

实现／约束（显式或 derive，无类继承）：

- `impl FirstObserver` — [源码:122](../../../desktop/infrastructure/src/separated_native.rs#L122)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn next_first(&mut self) -> Option<FirstCopy>` | impl FirstObserver | [源码:123](../../../desktop/infrastructure/src/separated_native.rs#L123) | skin_editor_infrastructure::separated_native::FirstCopy |

## skin_editor_infrastructure::separated_native::NativeExecutor

[源码:288](../../../desktop/infrastructure/src/separated_native.rs#L288) · struct

```rust
pub struct NativeExecutor {
    stream: Option<Stream>,
    returned: Receiver<StoppedOutput>,
    first: Option<FirstObserver>,
    shared: Arc<Shared>,
    build_error: Option<String>,
    period_frames: u32,
    return_received: bool,
}
```

声明引用：`skin_editor_infrastructure::separated_native::FirstObserver`, `skin_editor_infrastructure::separated_native::Shared`, `skin_editor_infrastructure::separated_output::StoppedOutput`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::editor_audio::EditorAudio (声明)`, `skin_editor_desktop::separated_verification::duration (声明)`, `skin_editor_desktop::separated_verification::tick (声明)`, `skin_editor_desktop::separated_verification::wait (声明)`, `skin_editor_desktop::worker_verification::healthy (声明)`, `skin_editor_desktop::worker_verification::ready (声明)`, `skin_editor_infrastructure::separated_native::PreparedDevice::attach`, `skin_editor_infrastructure::separated_native::PreparedDevice::attach_inner`, `skin_editor_infrastructure::separated_native::PreparedDevice::attach_with_held_return`, `skin_editor_infrastructure::separated_native::executor_gated (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl NativeExecutor` — [源码:479](../../../desktop/infrastructure/src/separated_native.rs#L479)
- `impl Drop for NativeExecutor` — [源码:554](../../../desktop/infrastructure/src/separated_native.rs#L554)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn controlled_return( endpoint: Box<OutputConsumer>, ) -> (Self, ControlledCallback, ReturnGate)` | impl NativeExecutor | [源码:480](../../../desktop/infrastructure/src/separated_native.rs#L480) | skin_editor_infrastructure::separated_native::ControlledCallback, skin_editor_infrastructure::separated_native::ReturnGate, skin_editor_infrastructure::separated_output::OutputConsumer |
| `pub fn fault_signal(&self) -> NativeFaultSignal` | impl NativeExecutor | [源码:490](../../../desktop/infrastructure/src/separated_native.rs#L490) | skin_editor_infrastructure::separated_native::NativeFaultSignal |
| `pub fn capture_work_cost(&self)` | impl NativeExecutor | [源码:494](../../../desktop/infrastructure/src/separated_native.rs#L494) |  |
| `pub fn work_cost(&self) -> Option<[u64; 6]>` | impl NativeExecutor | [源码:497](../../../desktop/infrastructure/src/separated_native.rs#L497) |  |
| `pub fn period_frames(&self) -> u32` | impl NativeExecutor | [源码:500](../../../desktop/infrastructure/src/separated_native.rs#L500) |  |
| `pub fn play(&self) -> Result<(), String>` | impl NativeExecutor | [源码:503](../../../desktop/infrastructure/src/separated_native.rs#L503) |  |
| `pub fn stats(&self) -> NativeStats` | impl NativeExecutor | [源码:513](../../../desktop/infrastructure/src/separated_native.rs#L513) | skin_editor_infrastructure::separated_native::NativeStats |
| `pub fn next_first(&mut self) -> Option<FirstCopy>` | impl NativeExecutor | [源码:516](../../../desktop/infrastructure/src/separated_native.rs#L516) | skin_editor_infrastructure::separated_native::FirstCopy |
| `pub fn take_first_observer(&mut self) -> Option<FirstObserver>` | impl NativeExecutor | [源码:519](../../../desktop/infrastructure/src/separated_native.rs#L519) | skin_editor_infrastructure::separated_native::FirstObserver |
| `pub fn set_monitor_volume(&self, volume: u16) -> Result<(), String>` | impl NativeExecutor | [源码:522](../../../desktop/infrastructure/src/separated_native.rs#L522) |  |
| `pub fn try_stop( &mut self, ) -> Result<(StoppedOutput, NativeStats), EndpointReturnError>` | impl NativeExecutor | [源码:531](../../../desktop/infrastructure/src/separated_native.rs#L531) | skin_editor_infrastructure::separated_native::EndpointReturnError, skin_editor_infrastructure::separated_native::NativeStats, skin_editor_infrastructure::separated_output::StoppedOutput |
| `pub fn stop(self) -> Result<(StoppedOutput, NativeStats), String>` | impl NativeExecutor | [源码:549](../../../desktop/infrastructure/src/separated_native.rs#L549) | skin_editor_infrastructure::separated_native::NativeStats, skin_editor_infrastructure::separated_output::StoppedOutput |
| `fn drop(&mut self)` | impl Drop for NativeExecutor | [源码:555](../../../desktop/infrastructure/src/separated_native.rs#L555) |  |

## skin_editor_infrastructure::separated_native::NativeFaultSignal

[源码:88](../../../desktop/infrastructure/src/separated_native.rs#L88) · struct

```rust
pub struct NativeFaultSignal(Arc<Shared>);
```

声明引用：`skin_editor_infrastructure::separated_native::Shared`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::editor_audio::window_recovery::Returned (声明)`, `skin_editor_desktop::editor_audio::window_recovery::Run (声明)`, `skin_editor_desktop::editor_audio::window_recovery::fault::Signals (声明)`, `skin_editor_desktop::editor_audio::window_recovery::fault::Signals::bind`, `skin_editor_desktop::editor_audio::window_recovery::init_failure::Recovered (声明)`, `skin_editor_infrastructure::separated_native::NativeExecutor::fault_signal`, `skin_editor_infrastructure::separated_native::NativeFaultSignal::clone`

实现／约束（显式或 derive，无类继承）：

- `impl NativeFaultSignal` — [源码:89](../../../desktop/infrastructure/src/separated_native.rs#L89)
- `impl Clone for NativeFaultSignal` — [源码:87](../../../desktop/infrastructure/src/separated_native.rs#L87)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn report(&self)` | impl NativeFaultSignal | [源码:90](../../../desktop/infrastructure/src/separated_native.rs#L90) |  |
| `fn clone(&self) -> NativeFaultSignal` | impl Clone for NativeFaultSignal | [源码:87](../../../desktop/infrastructure/src/separated_native.rs#L87) | skin_editor_infrastructure::separated_native::NativeFaultSignal |

## skin_editor_infrastructure::separated_native::NativeStats

[源码:24](../../../desktop/infrastructure/src/separated_native.rs#L24) · struct

```rust
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
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::editor_audio::EditorAudio (声明)`, `skin_editor_desktop::editor_audio::combined::Run (声明)`, `skin_editor_desktop::editor_audio::cycles::Round (声明)`, `skin_editor_desktop::editor_audio::gui_verification::Run (声明)`, `skin_editor_infrastructure::separated_native::NativeExecutor::stats`, `skin_editor_infrastructure::separated_native::NativeExecutor::stop`, `skin_editor_infrastructure::separated_native::NativeExecutor::try_stop`, `skin_editor_infrastructure::separated_native::NativeStats::clone`, `skin_editor_infrastructure::separated_native::NativeStats::default`, `skin_editor_infrastructure::separated_native::Shared::stats`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for NativeStats` — [源码:23](../../../desktop/infrastructure/src/separated_native.rs#L23)
- `impl Debug for NativeStats` — [源码:23](../../../desktop/infrastructure/src/separated_native.rs#L23)
- `impl Default for NativeStats` — [源码:23](../../../desktop/infrastructure/src/separated_native.rs#L23)
- `impl Copy for NativeStats` — [源码:23](../../../desktop/infrastructure/src/separated_native.rs#L23)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> NativeStats` | impl Clone for NativeStats | [源码:23](../../../desktop/infrastructure/src/separated_native.rs#L23) | skin_editor_infrastructure::separated_native::NativeStats |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for NativeStats | [源码:23](../../../desktop/infrastructure/src/separated_native.rs#L23) |  |
| `fn default() -> NativeStats` | impl Default for NativeStats | [源码:23](../../../desktop/infrastructure/src/separated_native.rs#L23) | skin_editor_infrastructure::separated_native::NativeStats |

## skin_editor_infrastructure::separated_native::OBSERVATIONS

[源码:13](../../../desktop/infrastructure/src/separated_native.rs#L13) · constant

```rust
const OBSERVATIONS: usize = 128;
```

## skin_editor_infrastructure::separated_native::PreparedDevice

[源码:284](../../../desktop/infrastructure/src/separated_native.rs#L284) · struct

```rust
pub struct PreparedDevice {
    device: Device,
    config: StreamConfig,
}
```

实现／约束（显式或 derive，无类继承）：

- `impl PreparedDevice` — [源码:402](../../../desktop/infrastructure/src/separated_native.rs#L402)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn format(&self) -> (u32, u16)` | impl PreparedDevice | [源码:403](../../../desktop/infrastructure/src/separated_native.rs#L403) |  |
| `pub fn probe() -> Result<Self, String>` | impl PreparedDevice | [源码:406](../../../desktop/infrastructure/src/separated_native.rs#L406) |  |
| `pub fn attach(self, endpoint: Box<OutputConsumer>) -> NativeExecutor` | impl PreparedDevice | [源码:429](../../../desktop/infrastructure/src/separated_native.rs#L429) | skin_editor_infrastructure::separated_native::NativeExecutor, skin_editor_infrastructure::separated_output::OutputConsumer |
| `pub fn attach_with_held_return( self, endpoint: Box<OutputConsumer>, ) -> (NativeExecutor, ReturnGate)` | impl PreparedDevice | [源码:432](../../../desktop/infrastructure/src/separated_native.rs#L432) | skin_editor_infrastructure::separated_native::NativeExecutor, skin_editor_infrastructure::separated_native::ReturnGate, skin_editor_infrastructure::separated_output::OutputConsumer |
| `fn attach_inner( self, endpoint: Box<OutputConsumer>, held: bool, ) -> (NativeExecutor, Option<ReturnGate>)` | impl PreparedDevice | [源码:439](../../../desktop/infrastructure/src/separated_native.rs#L439) | skin_editor_infrastructure::separated_native::NativeExecutor, skin_editor_infrastructure::separated_native::ReturnGate, skin_editor_infrastructure::separated_output::OutputConsumer |

## skin_editor_infrastructure::separated_native::ReturnGate

[源码:357](../../../desktop/infrastructure/src/separated_native.rs#L357) · struct

```rust
pub struct ReturnGate {
    receiver: Receiver<StoppedOutput>,
    destination: Option<SyncSender<StoppedOutput>>,
    endpoint: Option<StoppedOutput>,
    shared: Arc<Shared>,
}
```

声明引用：`skin_editor_infrastructure::separated_native::Shared`, `skin_editor_infrastructure::separated_output::StoppedOutput`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::editor_audio::EditorAudio (声明)`, `skin_editor_infrastructure::separated_native::NativeExecutor::controlled_return`, `skin_editor_infrastructure::separated_native::PreparedDevice::attach_inner`, `skin_editor_infrastructure::separated_native::PreparedDevice::attach_with_held_return`, `skin_editor_infrastructure::separated_native::executor_gated (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl ReturnGate` — [源码:363](../../../desktop/infrastructure/src/separated_native.rs#L363)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn disconnect(&mut self)` | impl ReturnGate | [源码:364](../../../desktop/infrastructure/src/separated_native.rs#L364) |  |
| `pub fn deliver(&mut self) -> Result<(), String>` | impl ReturnGate | [源码:367](../../../desktop/infrastructure/src/separated_native.rs#L367) |  |

## skin_editor_infrastructure::separated_native::Shared

[源码:66](../../../desktop/infrastructure/src/separated_native.rs#L66) · struct

```rust
struct Shared {
    control: Option<Control>,
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
```

声明引用：`skin_editor_infrastructure::separated_native::WorkCounters`, `skin_editor_infrastructure::separated_output::control::Control`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::separated_native::Callback (声明)`, `skin_editor_infrastructure::separated_native::NativeExecutor (声明)`, `skin_editor_infrastructure::separated_native::NativeFaultSignal (声明)`, `skin_editor_infrastructure::separated_native::ReturnGate (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Shared` — [源码:127](../../../desktop/infrastructure/src/separated_native.rs#L127)
- `impl Default for Shared` — [源码:97](../../../desktop/infrastructure/src/separated_native.rs#L97)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn stats(&self) -> NativeStats` | impl Shared | [源码:128](../../../desktop/infrastructure/src/separated_native.rs#L128) | skin_editor_infrastructure::separated_native::NativeStats |
| `fn default() -> Self` | impl Default for Shared | [源码:98](../../../desktop/infrastructure/src/separated_native.rs#L98) |  |

## skin_editor_infrastructure::separated_native::WorkCounters

[源码:40](../../../desktop/infrastructure/src/separated_native.rs#L40) · struct

```rust
struct WorkCounters {
    stamp: AtomicU64,
    values: [AtomicU64; 6],
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::separated_native::Shared (声明)`, `skin_editor_infrastructure::separated_native::WorkCounters::default`

实现／约束（显式或 derive，无类继承）：

- `impl WorkCounters` — [源码:44](../../../desktop/infrastructure/src/separated_native.rs#L44)
- `impl Default for WorkCounters` — [源码:39](../../../desktop/infrastructure/src/separated_native.rs#L39)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn publish(&self, values: [u64; 6])` | impl WorkCounters | [源码:45](../../../desktop/infrastructure/src/separated_native.rs#L45) |  |
| `fn read(&self) -> Option<[u64; 6]>` | impl WorkCounters | [源码:52](../../../desktop/infrastructure/src/separated_native.rs#L52) |  |
| `fn default() -> WorkCounters` | impl Default for WorkCounters | [源码:39](../../../desktop/infrastructure/src/separated_native.rs#L39) | skin_editor_infrastructure::separated_native::WorkCounters |

## skin_editor_infrastructure::separated_native::executor_gated

[源码:302](../../../desktop/infrastructure/src/separated_native.rs#L302) · fn

```rust
fn executor_gated(
    endpoint: Box<OutputConsumer>,
    held: bool,
) -> (Callback, NativeExecutor, Option<ReturnGate>)
```

声明引用：`skin_editor_infrastructure::separated_native::Callback`, `skin_editor_infrastructure::separated_native::NativeExecutor`, `skin_editor_infrastructure::separated_native::ReturnGate`, `skin_editor_infrastructure::separated_output::OutputConsumer`

## skin_editor_infrastructure::separated_output::BACKGROUND_RESERVE

[源码:18](../../../desktop/infrastructure/src/separated_output.rs#L18) · constant

```rust
pub const BACKGROUND_RESERVE: usize = 2048;
```

## skin_editor_infrastructure::separated_output::BackgroundOwner

[源码:125](../../../desktop/infrastructure/src/separated_output.rs#L125) · struct

```rust
pub struct BackgroundOwner {
    voices: VoiceOwner,
    runtime: VoiceRuntimeKey,
    shared: Arc<Shared>,
    tx: Option<Producer<Frame>>,
    track: Option<BackgroundInput>,
    phase: Phase,
    generation: u64,
    anchor: u64,
    target: Option<u64>,
    last_clock: CopyClock,
}
```

声明引用：`skin_editor_audio::VoiceRuntimeKey`, `skin_editor_infrastructure::background_input::BackgroundInput`, `skin_editor_infrastructure::separated_output::CopyClock`, `skin_editor_infrastructure::separated_output::Frame`, `skin_editor_infrastructure::separated_output::Phase`, `skin_editor_infrastructure::separated_output::Shared`, `skin_editor_infrastructure::voice_handoff::VoiceOwner`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::separated_verification::duration (声明)`, `skin_editor_desktop::separated_verification::tick (声明)`, `skin_editor_desktop::separated_verification::wait (声明)`, `skin_editor_infrastructure::audio_supply::RetiredSupply (声明)`, `skin_editor_infrastructure::audio_supply::SupplyCore::new_separated`, `skin_editor_infrastructure::audio_supply::SupplyOutput (声明)`, `skin_editor_infrastructure::separated_output::boundary (声明)`, `skin_editor_infrastructure::separated_output::boundary_input (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl BackgroundOwner` — [源码:280](../../../desktop/infrastructure/src/separated_output.rs#L280)
- `impl Drop for BackgroundOwner` — [源码:531](../../../desktop/infrastructure/src/separated_output.rs#L531)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub(crate) fn retained_drafts(&self) -> impl Iterator<Item = u64> + '_` | impl BackgroundOwner | [源码:281](../../../desktop/infrastructure/src/separated_output.rs#L281) |  |
| `pub(crate) fn control(&self) -> Control` | impl BackgroundOwner | [源码:284](../../../desktop/infrastructure/src/separated_output.rs#L284) | skin_editor_infrastructure::separated_output::control::Control |
| `pub(crate) fn apply_mark(&self, mark: OutputMark)` | impl BackgroundOwner | [源码:287](../../../desktop/infrastructure/src/separated_output.rs#L287) | skin_editor_infrastructure::output_fence::OutputMark |
| `fn validate(&self, runtime: &Runtime) -> Result<(), OutputError>` | impl BackgroundOwner | [源码:290](../../../desktop/infrastructure/src/separated_output.rs#L290) | skin_editor_audio::Runtime, skin_editor_infrastructure::separated_output::OutputError |
| `pub fn position(&self) -> Position` | impl BackgroundOwner | [源码:307](../../../desktop/infrastructure/src/separated_output.rs#L307) | skin_editor_infrastructure::separated_output::Position |
| `pub fn pump(&mut self, runtime: &mut Runtime) -> Result<PumpReport, OutputError>` | impl BackgroundOwner | [源码:327](../../../desktop/infrastructure/src/separated_output.rs#L327) | skin_editor_audio::Runtime, skin_editor_infrastructure::separated_output::OutputError, skin_editor_infrastructure::separated_output::PumpReport |
| `pub fn refill(&mut self) -> Result<usize, OutputError>` | impl BackgroundOwner | [源码:368](../../../desktop/infrastructure/src/separated_output.rs#L368) | skin_editor_infrastructure::separated_output::OutputError |
| `fn refill_inner( &mut self, published: impl FnMut(), ) -> Result<usize, OutputError>` | impl BackgroundOwner | [源码:371](../../../desktop/infrastructure/src/separated_output.rs#L371) | skin_editor_infrastructure::separated_output::OutputError |
| `fn freeze( &mut self, runtime: &mut Runtime, target: Option<u64>, ) -> Result<(), OutputError>` | impl BackgroundOwner | [源码:404](../../../desktop/infrastructure/src/separated_output.rs#L404) | skin_editor_audio::Runtime, skin_editor_infrastructure::separated_output::OutputError |
| `pub fn pause(&mut self, runtime: &mut Runtime) -> Result<(), OutputError>` | impl BackgroundOwner | [源码:429](../../../desktop/infrastructure/src/separated_output.rs#L429) | skin_editor_audio::Runtime, skin_editor_infrastructure::separated_output::OutputError |
| `pub fn seek( &mut self, runtime: &mut Runtime, frame: u64, ) -> Result<(), OutputError>` | impl BackgroundOwner | [源码:434](../../../desktop/infrastructure/src/separated_output.rs#L434) | skin_editor_audio::Runtime, skin_editor_infrastructure::separated_output::OutputError |
| `pub fn resume(&mut self, runtime: &mut Runtime) -> Result<(), OutputError>` | impl BackgroundOwner | [源码:437](../../../desktop/infrastructure/src/separated_output.rs#L437) | skin_editor_audio::Runtime, skin_editor_infrastructure::separated_output::OutputError |
| `pub(crate) fn is_primed(&self) -> bool` | impl BackgroundOwner | [源码:464](../../../desktop/infrastructure/src/separated_output.rs#L464) |  |
| `pub fn close(&mut self)` | impl BackgroundOwner | [源码:470](../../../desktop/infrastructure/src/separated_output.rs#L470) |  |
| `pub fn reclaim_abandoned( &mut self, runtime: &mut Runtime, ) -> Result<(), OutputError>` | impl BackgroundOwner | [源码:478](../../../desktop/infrastructure/src/separated_output.rs#L478) | skin_editor_audio::Runtime, skin_editor_infrastructure::separated_output::OutputError |
| `pub fn finish( &mut self, runtime: &mut Runtime, stopped: StoppedOutput, ) -> Result<(), StoppedOutput>` | impl BackgroundOwner | [源码:502](../../../desktop/infrastructure/src/separated_output.rs#L502) | skin_editor_audio::Runtime, skin_editor_infrastructure::separated_output::StoppedOutput |
| `fn drop(&mut self)` | impl Drop for BackgroundOwner | [源码:532](../../../desktop/infrastructure/src/separated_output.rs#L532) |  |

## skin_editor_infrastructure::separated_output::CLOSED

[源码:20](../../../desktop/infrastructure/src/separated_output.rs#L20) · constant

```rust
const CLOSED: u64 = _; // 9_223_372_036_854_775_808u64
```

## skin_editor_infrastructure::separated_output::Checkpoint

[源码:544](../../../desktop/infrastructure/src/separated_output.rs#L544) · enum

```rust
enum Checkpoint {
    AfterBackground,
    BeforeCopy,
    AfterCopy,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::separated_output::Checkpoint::clone`, `skin_editor_infrastructure::separated_output::OutputConsumer::render_inner`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for Checkpoint` — [源码:543](../../../desktop/infrastructure/src/separated_output.rs#L543)
- `impl Copy for Checkpoint` — [源码:543](../../../desktop/infrastructure/src/separated_output.rs#L543)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> Checkpoint` | impl Clone for Checkpoint | [源码:543](../../../desktop/infrastructure/src/separated_output.rs#L543) | skin_editor_infrastructure::separated_output::Checkpoint |

## skin_editor_infrastructure::separated_output::Clock

[源码:64](../../../desktop/infrastructure/src/separated_output.rs#L64) · struct

```rust
struct Clock {
    stamp: AtomicU64,
    epoch: AtomicU64,
    running: AtomicU64,
    background: AtomicU64,
    output: AtomicU64,
    missing: AtomicU64,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::separated_output::Shared (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Clock` — [源码:72](../../../desktop/infrastructure/src/separated_output.rs#L72)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn new(position: u64) -> Self` | impl Clock | [源码:73](../../../desktop/infrastructure/src/separated_output.rs#L73) |  |
| `fn read(&self) -> Option<CopyClock>` | impl Clock | [源码:83](../../../desktop/infrastructure/src/separated_output.rs#L83) | skin_editor_infrastructure::separated_output::CopyClock |
| `fn publish(&self, value: CopyClock)` | impl Clock | [源码:100](../../../desktop/infrastructure/src/separated_output.rs#L100) | skin_editor_infrastructure::separated_output::CopyClock |

## skin_editor_infrastructure::separated_output::CopyClock

[源码:56](../../../desktop/infrastructure/src/separated_output.rs#L56) · struct

```rust
pub struct CopyClock {
    pub serial: u64,
    pub epoch: u64,
    pub running: bool,
    pub next_background: u64,
    pub next_output: u64,
    pub missing_total: u64,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::separated_output::BackgroundOwner (声明)`, `skin_editor_infrastructure::separated_output::Clock::publish`, `skin_editor_infrastructure::separated_output::Clock::read`, `skin_editor_infrastructure::separated_output::CopyClock::clone`, `skin_editor_infrastructure::separated_output::CopyClock::default`, `skin_editor_infrastructure::separated_output::CopyClock::eq`, `skin_editor_infrastructure::separated_output::OutputConsumer::copy_clock`, `skin_editor_infrastructure::separated_output::OutputReport (声明)`, `skin_editor_infrastructure::separated_output::Position (声明)`, `skin_editor_infrastructure::separated_output::StoppedOutput::copy_clock`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for CopyClock` — [源码:55](../../../desktop/infrastructure/src/separated_output.rs#L55)
- `impl Debug for CopyClock` — [源码:55](../../../desktop/infrastructure/src/separated_output.rs#L55)
- `impl Default for CopyClock` — [源码:55](../../../desktop/infrastructure/src/separated_output.rs#L55)
- `impl PartialEq for CopyClock` — [源码:55](../../../desktop/infrastructure/src/separated_output.rs#L55)
- `impl Copy for CopyClock` — [源码:55](../../../desktop/infrastructure/src/separated_output.rs#L55)
- `impl Eq for CopyClock` — [源码:55](../../../desktop/infrastructure/src/separated_output.rs#L55)
- `impl StructuralPartialEq for CopyClock` — [源码:55](../../../desktop/infrastructure/src/separated_output.rs#L55)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> CopyClock` | impl Clone for CopyClock | [源码:55](../../../desktop/infrastructure/src/separated_output.rs#L55) | skin_editor_infrastructure::separated_output::CopyClock |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for CopyClock | [源码:55](../../../desktop/infrastructure/src/separated_output.rs#L55) |  |
| `fn default() -> CopyClock` | impl Default for CopyClock | [源码:55](../../../desktop/infrastructure/src/separated_output.rs#L55) | skin_editor_infrastructure::separated_output::CopyClock |
| `fn eq(&self, other: &CopyClock) -> bool` | impl PartialEq for CopyClock | [源码:55](../../../desktop/infrastructure/src/separated_output.rs#L55) | skin_editor_infrastructure::separated_output::CopyClock |

## skin_editor_infrastructure::separated_output::EPOCH_MAX

[源码:21](../../../desktop/infrastructure/src/separated_output.rs#L21) · constant

```rust
const EPOCH_MAX: u64 = _; // 4_611_686_018_427_387_903u64
```

## skin_editor_infrastructure::separated_output::Frame

[源码:120](../../../desktop/infrastructure/src/separated_output.rs#L120) · struct

```rust
struct Frame {
    epoch: u64,
    position: u64,
    samples: [f32; 2],
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::separated_output::BackgroundOwner (声明)`, `skin_editor_infrastructure::separated_output::Frame::clone`, `skin_editor_infrastructure::separated_output::OutputConsumer (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for Frame` — [源码:119](../../../desktop/infrastructure/src/separated_output.rs#L119)
- `impl Copy for Frame` — [源码:119](../../../desktop/infrastructure/src/separated_output.rs#L119)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> Frame` | impl Clone for Frame | [源码:119](../../../desktop/infrastructure/src/separated_output.rs#L119) | skin_editor_infrastructure::separated_output::Frame |

## skin_editor_infrastructure::separated_output::OutputConsumer

[源码:137](../../../desktop/infrastructure/src/separated_output.rs#L137) · struct

```rust
pub struct OutputConsumer {
    voices: Option<Box<VoiceConsumer>>,
    shared: Arc<Shared>,
    rx: Consumer<Frame>,
    scene: PcmBuffer,
    audition: PcmBuffer,
    background: PcmBuffer,
    channels: usize,
    source_frames: u64,
    generation: u64,
    running: bool,
    started_epoch: u64,
    next_background: u64,
    next_output: u64,
    serial: u64,
    missing: u64,
}
```

声明引用：`skin_editor_audio::PcmBuffer`, `skin_editor_infrastructure::separated_output::Frame`, `skin_editor_infrastructure::separated_output::Shared`, `skin_editor_infrastructure::voice_handoff::VoiceConsumer`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::AudioSupplyClient::spawn_separated`, `skin_editor_infrastructure::AudioSupplyClient::spawn_streamed`, `skin_editor_infrastructure::audio_supply::SupplyCore::new_separated`, `skin_editor_infrastructure::separated_native::Callback (声明)`, `skin_editor_infrastructure::separated_native::NativeExecutor::controlled_return`, `skin_editor_infrastructure::separated_native::PreparedDevice::attach`, `skin_editor_infrastructure::separated_native::PreparedDevice::attach_inner`, `skin_editor_infrastructure::separated_native::PreparedDevice::attach_with_held_return`, `skin_editor_infrastructure::separated_native::executor_gated (声明)`, `skin_editor_infrastructure::separated_output::StoppedOutput (声明)`, `skin_editor_infrastructure::separated_output::boundary (声明)`, `skin_editor_infrastructure::separated_output::boundary_input (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl OutputConsumer` — [源码:549](../../../desktop/infrastructure/src/separated_output.rs#L549)
- `impl Drop for OutputConsumer` — [源码:537](../../../desktop/infrastructure/src/separated_output.rs#L537)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub(crate) fn control(&self) -> Control` | impl OutputConsumer | [源码:550](../../../desktop/infrastructure/src/separated_output.rs#L550) | skin_editor_infrastructure::separated_output::control::Control |
| `pub(crate) fn copy_clock(&self) -> Option<CopyClock>` | impl OutputConsumer | [源码:553](../../../desktop/infrastructure/src/separated_output.rs#L553) | skin_editor_infrastructure::separated_output::CopyClock |
| `pub fn format(&self) -> (u32, u16)` | impl OutputConsumer | [源码:556](../../../desktop/infrastructure/src/separated_output.rs#L556) |  |
| `pub fn stop(self: Box<Self>) -> StoppedOutput` | impl OutputConsumer | [源码:559](../../../desktop/infrastructure/src/separated_output.rs#L559) | skin_editor_infrastructure::separated_output::StoppedOutput |
| `fn observe_freeze(&mut self, word: u64)` | impl OutputConsumer | [源码:563](../../../desktop/infrastructure/src/separated_output.rs#L563) |  |
| `pub fn render( &mut self, output: &mut [f32], ) -> Result<OutputReport, OutputError>` | impl OutputConsumer | [源码:584](../../../desktop/infrastructure/src/separated_output.rs#L584) | skin_editor_infrastructure::separated_output::OutputError, skin_editor_infrastructure::separated_output::OutputReport |
| `pub(crate) fn render_profiled( &mut self, output: &mut [f32], ) -> Result<(OutputReport, [u64; 3]), OutputError>` | impl OutputConsumer | [源码:589](../../../desktop/infrastructure/src/separated_output.rs#L589) | skin_editor_infrastructure::separated_output::OutputError, skin_editor_infrastructure::separated_output::OutputReport |
| `fn render_inner( &mut self, output: &mut [f32], checkpoint: impl FnMut(Checkpoint), ) -> Result<OutputReport, OutputError>` | impl OutputConsumer | [源码:610](../../../desktop/infrastructure/src/separated_output.rs#L610) | skin_editor_infrastructure::separated_output::Checkpoint, skin_editor_infrastructure::separated_output::OutputError, skin_editor_infrastructure::separated_output::OutputReport |
| `fn drop(&mut self)` | impl Drop for OutputConsumer | [源码:538](../../../desktop/infrastructure/src/separated_output.rs#L538) |  |

## skin_editor_infrastructure::separated_output::OutputError

[源码:27](../../../desktop/infrastructure/src/separated_output.rs#L27) · enum

```rust
pub enum OutputError {
    Voice(HandoffError),
    Runtime,
    WrongRuntime,
    WrongFormat,
    WrongPool,
    ResourceLimit,
    Closed,
    Busy,
    AwaitingAck,
    NotPrimed,
    InvalidPosition,
    InvalidBlock,
    Protocol,
    BackgroundFault,
}
```

声明引用：`skin_editor_infrastructure::voice_handoff::HandoffError`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::background_input::BackgroundInput::next_frame`, `skin_editor_infrastructure::background_input::BackgroundInput::restore_position`, `skin_editor_infrastructure::separated_output::BackgroundOwner::freeze`, `skin_editor_infrastructure::separated_output::BackgroundOwner::pause`, `skin_editor_infrastructure::separated_output::BackgroundOwner::pump`, `skin_editor_infrastructure::separated_output::BackgroundOwner::reclaim_abandoned`, `skin_editor_infrastructure::separated_output::BackgroundOwner::refill`, `skin_editor_infrastructure::separated_output::BackgroundOwner::refill_inner`, `skin_editor_infrastructure::separated_output::BackgroundOwner::resume`, `skin_editor_infrastructure::separated_output::BackgroundOwner::seek`, `skin_editor_infrastructure::separated_output::BackgroundOwner::validate`, `skin_editor_infrastructure::separated_output::OutputConsumer::render`, `skin_editor_infrastructure::separated_output::OutputConsumer::render_inner`, `skin_editor_infrastructure::separated_output::OutputConsumer::render_profiled`, `skin_editor_infrastructure::separated_output::OutputError::clone`, `skin_editor_infrastructure::separated_output::OutputError::eq`, `skin_editor_infrastructure::separated_output::boundary (声明)`, `skin_editor_infrastructure::separated_output::boundary_input (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for OutputError` — [源码:26](../../../desktop/infrastructure/src/separated_output.rs#L26)
- `impl Debug for OutputError` — [源码:26](../../../desktop/infrastructure/src/separated_output.rs#L26)
- `impl From<HandoffError> for OutputError` — [源码:43](../../../desktop/infrastructure/src/separated_output.rs#L43)
- `impl PartialEq for OutputError` — [源码:26](../../../desktop/infrastructure/src/separated_output.rs#L26)
- `impl Copy for OutputError` — [源码:26](../../../desktop/infrastructure/src/separated_output.rs#L26)
- `impl Eq for OutputError` — [源码:26](../../../desktop/infrastructure/src/separated_output.rs#L26)
- `impl StructuralPartialEq for OutputError` — [源码:26](../../../desktop/infrastructure/src/separated_output.rs#L26)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> OutputError` | impl Clone for OutputError | [源码:26](../../../desktop/infrastructure/src/separated_output.rs#L26) | skin_editor_infrastructure::separated_output::OutputError |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for OutputError | [源码:26](../../../desktop/infrastructure/src/separated_output.rs#L26) |  |
| `fn from(e: HandoffError) -> Self` | impl From<HandoffError> for OutputError | [源码:44](../../../desktop/infrastructure/src/separated_output.rs#L44) | skin_editor_infrastructure::voice_handoff::HandoffError |
| `fn eq(&self, other: &OutputError) -> bool` | impl PartialEq for OutputError | [源码:26](../../../desktop/infrastructure/src/separated_output.rs#L26) | skin_editor_infrastructure::separated_output::OutputError |

## skin_editor_infrastructure::separated_output::OutputReport

[源码:182](../../../desktop/infrastructure/src/separated_output.rs#L182) · struct

```rust
pub struct OutputReport {
    pub voices: HandoffReport,
    pub background_frames: usize,
    pub missing_frames: usize,
    pub clock: CopyClock,
}
```

声明引用：`skin_editor_infrastructure::separated_output::CopyClock`, `skin_editor_infrastructure::voice_handoff::HandoffReport`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::separated_output::OutputConsumer::render`, `skin_editor_infrastructure::separated_output::OutputConsumer::render_inner`, `skin_editor_infrastructure::separated_output::OutputConsumer::render_profiled`, `skin_editor_infrastructure::separated_output::OutputReport::clone`, `skin_editor_infrastructure::separated_output::OutputReport::default`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for OutputReport` — [源码:181](../../../desktop/infrastructure/src/separated_output.rs#L181)
- `impl Debug for OutputReport` — [源码:181](../../../desktop/infrastructure/src/separated_output.rs#L181)
- `impl Default for OutputReport` — [源码:181](../../../desktop/infrastructure/src/separated_output.rs#L181)
- `impl Copy for OutputReport` — [源码:181](../../../desktop/infrastructure/src/separated_output.rs#L181)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> OutputReport` | impl Clone for OutputReport | [源码:181](../../../desktop/infrastructure/src/separated_output.rs#L181) | skin_editor_infrastructure::separated_output::OutputReport |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for OutputReport | [源码:181](../../../desktop/infrastructure/src/separated_output.rs#L181) |  |
| `fn default() -> OutputReport` | impl Default for OutputReport | [源码:181](../../../desktop/infrastructure/src/separated_output.rs#L181) | skin_editor_infrastructure::separated_output::OutputReport |

## skin_editor_infrastructure::separated_output::Phase

[源码:49](../../../desktop/infrastructure/src/separated_output.rs#L49) · enum

```rust
pub enum Phase {
    Frozen,
    Freezing,
    Running,
    Closed,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::separated_output::BackgroundOwner (声明)`, `skin_editor_infrastructure::separated_output::Phase::clone`, `skin_editor_infrastructure::separated_output::Phase::eq`, `skin_editor_infrastructure::separated_output::Position (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for Phase` — [源码:48](../../../desktop/infrastructure/src/separated_output.rs#L48)
- `impl Debug for Phase` — [源码:48](../../../desktop/infrastructure/src/separated_output.rs#L48)
- `impl PartialEq for Phase` — [源码:48](../../../desktop/infrastructure/src/separated_output.rs#L48)
- `impl Copy for Phase` — [源码:48](../../../desktop/infrastructure/src/separated_output.rs#L48)
- `impl Eq for Phase` — [源码:48](../../../desktop/infrastructure/src/separated_output.rs#L48)
- `impl StructuralPartialEq for Phase` — [源码:48](../../../desktop/infrastructure/src/separated_output.rs#L48)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> Phase` | impl Clone for Phase | [源码:48](../../../desktop/infrastructure/src/separated_output.rs#L48) | skin_editor_infrastructure::separated_output::Phase |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for Phase | [源码:48](../../../desktop/infrastructure/src/separated_output.rs#L48) |  |
| `fn eq(&self, other: &Phase) -> bool` | impl PartialEq for Phase | [源码:48](../../../desktop/infrastructure/src/separated_output.rs#L48) | skin_editor_infrastructure::separated_output::Phase |

## skin_editor_infrastructure::separated_output::Position

[源码:168](../../../desktop/infrastructure/src/separated_output.rs#L168) · struct

```rust
pub struct Position {
    pub phase: Phase,
    pub epoch: u64,
    pub timeline_frame: u64,
    pub generated_frame: u64,
    pub copy_clock: CopyClock,
    pub queued: usize,
}
```

声明引用：`skin_editor_infrastructure::separated_output::CopyClock`, `skin_editor_infrastructure::separated_output::Phase`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::separated_output::BackgroundOwner::position`, `skin_editor_infrastructure::separated_output::Position::clone`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for Position` — [源码:167](../../../desktop/infrastructure/src/separated_output.rs#L167)
- `impl Debug for Position` — [源码:167](../../../desktop/infrastructure/src/separated_output.rs#L167)
- `impl Copy for Position` — [源码:167](../../../desktop/infrastructure/src/separated_output.rs#L167)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> Position` | impl Clone for Position | [源码:167](../../../desktop/infrastructure/src/separated_output.rs#L167) | skin_editor_infrastructure::separated_output::Position |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for Position | [源码:167](../../../desktop/infrastructure/src/separated_output.rs#L167) |  |

## skin_editor_infrastructure::separated_output::PumpReport

[源码:177](../../../desktop/infrastructure/src/separated_output.rs#L177) · struct

```rust
pub struct PumpReport {
    pub voices: RuntimePumpReport,
    pub freeze_completed: bool,
}
```

声明引用：`skin_editor_infrastructure::voice_handoff::RuntimePumpReport`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::separated_output::BackgroundOwner::pump`, `skin_editor_infrastructure::separated_output::PumpReport::clone`, `skin_editor_infrastructure::separated_output::PumpReport::default`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for PumpReport` — [源码:176](../../../desktop/infrastructure/src/separated_output.rs#L176)
- `impl Debug for PumpReport` — [源码:176](../../../desktop/infrastructure/src/separated_output.rs#L176)
- `impl Default for PumpReport` — [源码:176](../../../desktop/infrastructure/src/separated_output.rs#L176)
- `impl Copy for PumpReport` — [源码:176](../../../desktop/infrastructure/src/separated_output.rs#L176)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> PumpReport` | impl Clone for PumpReport | [源码:176](../../../desktop/infrastructure/src/separated_output.rs#L176) | skin_editor_infrastructure::separated_output::PumpReport |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for PumpReport | [源码:176](../../../desktop/infrastructure/src/separated_output.rs#L176) |  |
| `fn default() -> PumpReport` | impl Default for PumpReport | [源码:176](../../../desktop/infrastructure/src/separated_output.rs#L176) | skin_editor_infrastructure::separated_output::PumpReport |

## skin_editor_infrastructure::separated_output::RUNNING

[源码:19](../../../desktop/infrastructure/src/separated_output.rs#L19) · constant

```rust
const RUNNING: u64 = _; // 4_611_686_018_427_387_904u64
```

## skin_editor_infrastructure::separated_output::Shared

[源码:110](../../../desktop/infrastructure/src/separated_output.rs#L110) · struct

```rust
struct Shared {
    control: Arc<State>,
    gate: AtomicU64,
    resume_at: AtomicU64,
    resume_epoch: AtomicU64,
    clock: Clock,
    payload: Option<PcmReservation>,
}
```

声明引用：`skin_editor_audio::PcmReservation`, `skin_editor_infrastructure::separated_output::Clock`, `skin_editor_infrastructure::separated_output::control::State`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::separated_output::BackgroundOwner (声明)`, `skin_editor_infrastructure::separated_output::OutputConsumer (声明)`

## skin_editor_infrastructure::separated_output::StoppedOutput

[源码:154](../../../desktop/infrastructure/src/separated_output.rs#L154) · struct

```rust
pub struct StoppedOutput(Box<OutputConsumer>);
```

声明引用：`skin_editor_infrastructure::separated_output::OutputConsumer`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::editor_audio::EditorAudio (声明)`, `skin_editor_infrastructure::AudioSupplyClient::finish_separated`, `skin_editor_infrastructure::separated_native::Callback (声明)`, `skin_editor_infrastructure::separated_native::NativeExecutor (声明)`, `skin_editor_infrastructure::separated_native::NativeExecutor::stop`, `skin_editor_infrastructure::separated_native::NativeExecutor::try_stop`, `skin_editor_infrastructure::separated_native::ReturnGate (声明)`, `skin_editor_infrastructure::separated_output::BackgroundOwner::finish`, `skin_editor_infrastructure::separated_output::OutputConsumer::stop`

实现／约束（显式或 derive，无类继承）：

- `impl StoppedOutput` — [源码:155](../../../desktop/infrastructure/src/separated_output.rs#L155)
- `impl Debug for StoppedOutput` — [源码:162](../../../desktop/infrastructure/src/separated_output.rs#L162)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn copy_clock(&self) -> Option<CopyClock>` | impl StoppedOutput | [源码:158](../../../desktop/infrastructure/src/separated_output.rs#L158) | skin_editor_infrastructure::separated_output::CopyClock |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for StoppedOutput | [源码:163](../../../desktop/infrastructure/src/separated_output.rs#L163) |  |

## skin_editor_infrastructure::separated_output::boundary

[源码:191](../../../desktop/infrastructure/src/separated_output.rs#L191) · fn

```rust
pub fn boundary(
    runtime: &mut Runtime,
    track: BackgroundTrack,
    source: Origin,
) -> Result<(BackgroundOwner, Box<OutputConsumer>), OutputError>
```

声明引用：`skin_editor_audio::BackgroundTrack`, `skin_editor_audio::Origin`, `skin_editor_audio::Runtime`, `skin_editor_infrastructure::separated_output::BackgroundOwner`, `skin_editor_infrastructure::separated_output::OutputConsumer`, `skin_editor_infrastructure::separated_output::OutputError`

## skin_editor_infrastructure::separated_output::boundary_input

[源码:198](../../../desktop/infrastructure/src/separated_output.rs#L198) · fn

```rust
pub(crate) fn boundary_input(
    runtime: &mut Runtime,
    track: BackgroundInput,
    source: Origin,
) -> Result<(BackgroundOwner, Box<OutputConsumer>), OutputError>
```

声明引用：`skin_editor_audio::Origin`, `skin_editor_audio::Runtime`, `skin_editor_infrastructure::background_input::BackgroundInput`, `skin_editor_infrastructure::separated_output::BackgroundOwner`, `skin_editor_infrastructure::separated_output::OutputConsumer`, `skin_editor_infrastructure::separated_output::OutputError`

## skin_editor_infrastructure::separated_output::control::Control

[源码:60](../../../desktop/infrastructure/src/separated_output/control.rs#L60) · struct

```rust
pub(crate) struct Control(pub(super) Arc<State>);
```

声明引用：`skin_editor_infrastructure::separated_output::control::State`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::separated_native::Shared (声明)`, `skin_editor_infrastructure::separated_output::BackgroundOwner::control`, `skin_editor_infrastructure::separated_output::OutputConsumer::control`, `skin_editor_infrastructure::separated_output::control::Control::clone`, `skin_editor_infrastructure::supply_output::SupplyOutputControl (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Control` — [源码:61](../../../desktop/infrastructure/src/separated_output/control.rs#L61)
- `impl Clone for Control` — [源码:59](../../../desktop/infrastructure/src/separated_output/control.rs#L59)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn is_closed(&self) -> bool` | impl Control | [源码:62](../../../desktop/infrastructure/src/separated_output/control.rs#L62) |  |
| `pub fn close(&self)` | impl Control | [源码:65](../../../desktop/infrastructure/src/separated_output/control.rs#L65) |  |
| `pub fn mark(&self) -> OutputMark` | impl Control | [源码:68](../../../desktop/infrastructure/src/separated_output/control.rs#L68) | skin_editor_infrastructure::output_fence::OutputMark |
| `pub fn invalidate( &self, scene: bool, audition: bool, background: bool, ) -> Result<OutputMark, String>` | impl Control | [源码:71](../../../desktop/infrastructure/src/separated_output/control.rs#L71) | skin_editor_infrastructure::output_fence::OutputMark |
| `pub fn acknowledged(&self, mark: OutputMark) -> bool` | impl Control | [源码:92](../../../desktop/infrastructure/src/separated_output/control.rs#L92) | skin_editor_infrastructure::output_fence::OutputMark |
| `pub fn background_acknowledged(&self, mark: OutputMark) -> bool` | impl Control | [源码:99](../../../desktop/infrastructure/src/separated_output/control.rs#L99) | skin_editor_infrastructure::output_fence::OutputMark |
| `pub fn background_next(&self) -> u64` | impl Control | [源码:102](../../../desktop/infrastructure/src/separated_output/control.rs#L102) |  |
| `fn clone(&self) -> Control` | impl Clone for Control | [源码:59](../../../desktop/infrastructure/src/separated_output/control.rs#L59) | skin_editor_infrastructure::separated_output::control::Control |

## skin_editor_infrastructure::separated_output::control::State

[源码:7](../../../desktop/infrastructure/src/separated_output/control.rs#L7) · struct

```rust
pub(super) struct State {
    pub closed: AtomicBool,
    desired: [AtomicU64; 3],
    applied: [AtomicU64; 3],
    ack: [AtomicU64; 3],
    next: AtomicU64,
    running: AtomicBool,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::separated_output::Shared (声明)`, `skin_editor_infrastructure::separated_output::control::Control (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl State` — [源码:15](../../../desktop/infrastructure/src/separated_output/control.rs#L15)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn new(position: u64) -> Arc<Self>` | impl State | [源码:16](../../../desktop/infrastructure/src/separated_output/control.rs#L16) |  |
| `fn read(fields: &[AtomicU64; 3]) -> OutputMark` | impl State | [源码:26](../../../desktop/infrastructure/src/separated_output/control.rs#L26) | skin_editor_infrastructure::output_fence::OutputMark |
| `pub fn desired(&self) -> OutputMark` | impl State | [源码:32](../../../desktop/infrastructure/src/separated_output/control.rs#L32) | skin_editor_infrastructure::output_fence::OutputMark |
| `pub fn applied(&self) -> OutputMark` | impl State | [源码:35](../../../desktop/infrastructure/src/separated_output/control.rs#L35) | skin_editor_infrastructure::output_fence::OutputMark |
| `pub fn apply(&self, mark: OutputMark)` | impl State | [源码:38](../../../desktop/infrastructure/src/separated_output/control.rs#L38) | skin_editor_infrastructure::output_fence::OutputMark |
| `pub fn acknowledge( &self, desired: OutputMark, applied: OutputMark, next: u64, running: bool, )` | impl State | [源码:47](../../../desktop/infrastructure/src/separated_output/control.rs#L47) | skin_editor_infrastructure::output_fence::OutputMark |

## skin_editor_infrastructure::separated_output::epoch

[源码:22](../../../desktop/infrastructure/src/separated_output.rs#L22) · fn

```rust
fn epoch(word: u64) -> u64
```

## skin_editor_infrastructure::submission_trace::Row

[源码:8](../../../desktop/infrastructure/src/submission_trace.rs#L8) · struct

```rust
struct Row<const N: usize> {
    sequence: AtomicU64,
    values: [AtomicU64; N],
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::submission_trace::SubmissionCapture (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl<const N: usize> Row<N>` — [源码:20](../../../desktop/infrastructure/src/submission_trace.rs#L20)
- `impl<const N: usize> Default for Row<N>` — [源码:12](../../../desktop/infrastructure/src/submission_trace.rs#L12)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn write(&self, values: [u64; N])` | impl<const N: usize> Row<N> | [源码:22](../../../desktop/infrastructure/src/submission_trace.rs#L22) |  |
| `fn read(&self) -> Option<[u64; N]>` | impl<const N: usize> Row<N> | [源码:29](../../../desktop/infrastructure/src/submission_trace.rs#L29) |  |
| `fn default() -> Self` | impl<const N: usize> Default for Row<N> | [源码:13](../../../desktop/infrastructure/src/submission_trace.rs#L13) |  |

## skin_editor_infrastructure::submission_trace::SubmissionCapture

[源码:44](../../../desktop/infrastructure/src/submission_trace.rs#L44) · struct

```rust
pub(crate) struct SubmissionCapture {
    origin: OnceLock<Instant>,
    producer: [Row<9>; 2],
    callback: [Row<7>; 2],
    progress: Row<4>,
    stage: AtomicU64,
    stages: [[AtomicU64; 5]; 8],
    underruns: [Row<44>; 32],
    supply: [Row<10>; 4],
    supply_head: AtomicU64,
    slow_supply: [Row<10>; 32],
    slow_supply_count: AtomicU64,
    schedule_events: [Row<86>; 32],
    schedule_event_count: AtomicU64,
}
```

声明引用：`skin_editor_infrastructure::submission_trace::Row`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::native_audio::Shared (声明)`, `skin_editor_infrastructure::submission_trace::SubmissionCapture::default`

实现／约束（显式或 derive，无类继承）：

- `impl SubmissionCapture` — [源码:80](../../../desktop/infrastructure/src/submission_trace.rs#L80)
- `impl Default for SubmissionCapture` — [源码:43](../../../desktop/infrastructure/src/submission_trace.rs#L43)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn begin(&self) -> Result<Instant, String>` | impl SubmissionCapture | [源码:81](../../../desktop/infrastructure/src/submission_trace.rs#L81) |  |
| `pub fn origin(&self) -> Option<Instant>` | impl SubmissionCapture | [源码:88](../../../desktop/infrastructure/src/submission_trace.rs#L88) |  |
| `pub fn produced(&self, lane: usize, row: [u64; 9])` | impl SubmissionCapture | [源码:91](../../../desktop/infrastructure/src/submission_trace.rs#L91) |  |
| `pub fn submitted(&self, lane: usize, row: [u64; 7])` | impl SubmissionCapture | [源码:94](../../../desktop/infrastructure/src/submission_trace.rs#L94) |  |
| `pub fn completed_callback(&self, row: [u64; 4])` | impl SubmissionCapture | [源码:97](../../../desktop/infrastructure/src/submission_trace.rs#L97) |  |
| `pub fn callback_progress(&self) -> Option<[u64; 4]>` | impl SubmissionCapture | [源码:100](../../../desktop/infrastructure/src/submission_trace.rs#L100) |  |
| `pub fn set_stage(&self, stage: u64) -> Result<(), String>` | impl SubmissionCapture | [源码:103](../../../desktop/infrastructure/src/submission_trace.rs#L103) |  |
| `pub fn stage(&self) -> u64` | impl SubmissionCapture | [源码:110](../../../desktop/infrastructure/src/submission_trace.rs#L110) |  |
| `pub fn completed_context(&self, history: [[u64; 11]; 4], empty_index: usize)` | impl SubmissionCapture | [源码:113](../../../desktop/infrastructure/src/submission_trace.rs#L113) |  |
| `pub fn completed_supply(&self, row: [u64; 10])` | impl SubmissionCapture | [源码:150](../../../desktop/infrastructure/src/submission_trace.rs#L150) |  |
| `fn recent_supply(&self, head: u64) -> [Option<[u64; 10]>; 4]` | impl SubmissionCapture | [源码:161](../../../desktop/infrastructure/src/submission_trace.rs#L161) |  |
| `pub fn scheduling(&self) -> SupplyTimingSnapshot` | impl SubmissionCapture | [源码:169](../../../desktop/infrastructure/src/submission_trace.rs#L169) | skin_editor_infrastructure::SupplyTimingSnapshot |
| `pub fn stage_counts(&self) -> [[u64; 5]; 8]` | impl SubmissionCapture | [源码:188](../../../desktop/infrastructure/src/submission_trace.rs#L188) |  |
| `pub fn underrun_contexts(&self, count: usize) -> Vec<Option<[[u64; 11]; 4]>>` | impl SubmissionCapture | [源码:191](../../../desktop/infrastructure/src/submission_trace.rs#L191) |  |
| `pub fn observe(&self, lane: usize, voice: u64, epoch: u64) -> VoiceSubmission` | impl SubmissionCapture | [源码:200](../../../desktop/infrastructure/src/submission_trace.rs#L200) | skin_editor_infrastructure::VoiceSubmission |
| `pub fn read(&self, lane: usize, voice: u64) -> Option<SubmissionTrace>` | impl SubmissionCapture | [源码:214](../../../desktop/infrastructure/src/submission_trace.rs#L214) | skin_editor_infrastructure::SubmissionTrace |
| `fn default() -> SubmissionCapture` | impl Default for SubmissionCapture | [源码:43](../../../desktop/infrastructure/src/submission_trace.rs#L43) | skin_editor_infrastructure::submission_trace::SubmissionCapture |

## skin_editor_infrastructure::supply_output::SupplyOutputControl

[源码:4](../../../desktop/infrastructure/src/supply_output.rs#L4) · enum

```rust
pub(crate) enum SupplyOutputControl {
    Premixed(OutputControl),
    Separated(Control),
}
```

声明引用：`skin_editor_infrastructure::OutputControl`, `skin_editor_infrastructure::separated_output::control::Control`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::AudioSupplyClient (声明)`, `skin_editor_infrastructure::audio_supply::SupplyCore (声明)`, `skin_editor_infrastructure::supply_output::SupplyOutputControl::clone`

实现／约束（显式或 derive，无类继承）：

- `impl SupplyOutputControl` — [源码:8](../../../desktop/infrastructure/src/supply_output.rs#L8)
- `impl Clone for SupplyOutputControl` — [源码:3](../../../desktop/infrastructure/src/supply_output.rs#L3)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn is_separated(&self) -> bool` | impl SupplyOutputControl | [源码:9](../../../desktop/infrastructure/src/supply_output.rs#L9) |  |
| `pub fn premixed(&self) -> Result<OutputControl, String>` | impl SupplyOutputControl | [源码:12](../../../desktop/infrastructure/src/supply_output.rs#L12) | skin_editor_infrastructure::OutputControl |
| `pub fn capture_origin(&self) -> Option<Instant>` | impl SupplyOutputControl | [源码:20](../../../desktop/infrastructure/src/supply_output.rs#L20) |  |
| `pub fn mark(&self) -> OutputMark` | impl SupplyOutputControl | [源码:23](../../../desktop/infrastructure/src/supply_output.rs#L23) | skin_editor_infrastructure::output_fence::OutputMark |
| `pub fn invalidate_skin( &self, scene: bool, audition: bool, ) -> Result<OutputMark, String>` | impl SupplyOutputControl | [源码:29](../../../desktop/infrastructure/src/supply_output.rs#L29) | skin_editor_infrastructure::output_fence::OutputMark |
| `pub fn invalidate_scene(&self) -> Result<OutputMark, String>` | impl SupplyOutputControl | [源码:35](../../../desktop/infrastructure/src/supply_output.rs#L35) | skin_editor_infrastructure::output_fence::OutputMark |
| `pub fn background_acknowledged(&self, mark: OutputMark) -> bool` | impl SupplyOutputControl | [源码:41](../../../desktop/infrastructure/src/supply_output.rs#L41) | skin_editor_infrastructure::output_fence::OutputMark |
| `pub fn acknowledged(&self, mark: OutputMark) -> bool` | impl SupplyOutputControl | [源码:47](../../../desktop/infrastructure/src/supply_output.rs#L47) | skin_editor_infrastructure::output_fence::OutputMark |
| `pub fn background_next(&self) -> u64` | impl SupplyOutputControl | [源码:53](../../../desktop/infrastructure/src/supply_output.rs#L53) |  |
| `pub fn is_closed(&self) -> bool` | impl SupplyOutputControl | [源码:59](../../../desktop/infrastructure/src/supply_output.rs#L59) |  |
| `pub fn close(&self)` | impl SupplyOutputControl | [源码:65](../../../desktop/infrastructure/src/supply_output.rs#L65) |  |
| `fn clone(&self) -> SupplyOutputControl` | impl Clone for SupplyOutputControl | [源码:3](../../../desktop/infrastructure/src/supply_output.rs#L3) | skin_editor_infrastructure::supply_output::SupplyOutputControl |

## skin_editor_infrastructure::voice_handoff::CLOSED

[源码:16](../../../desktop/infrastructure/src/voice_handoff.rs#L16) · constant

```rust
const CLOSED: u64 = _; // 2_305_843_009_213_693_952u64
```

## skin_editor_infrastructure::voice_handoff::Checkpoint

[源码:495](../../../desktop/infrastructure/src/voice_handoff.rs#L495) · enum

```rust
enum Checkpoint {
    AfterDrain,
    BeforeFence,
    AfterRetire,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::voice_handoff::Checkpoint::clone`, `skin_editor_infrastructure::voice_handoff::VoiceConsumer::render_committed`, `skin_editor_infrastructure::voice_handoff::VoiceConsumer::render_inner`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for Checkpoint` — [源码:494](../../../desktop/infrastructure/src/voice_handoff.rs#L494)
- `impl Copy for Checkpoint` — [源码:494](../../../desktop/infrastructure/src/voice_handoff.rs#L494)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> Checkpoint` | impl Clone for Checkpoint | [源码:494](../../../desktop/infrastructure/src/voice_handoff.rs#L494) | skin_editor_infrastructure::voice_handoff::Checkpoint |

## skin_editor_infrastructure::voice_handoff::EPOCH_BITS

[源码:13](../../../desktop/infrastructure/src/voice_handoff.rs#L13) · constant

```rust
const EPOCH_BITS: u32 = 30;
```

## skin_editor_infrastructure::voice_handoff::EPOCH_MAX

[源码:14](../../../desktop/infrastructure/src/voice_handoff.rs#L14) · constant

```rust
const EPOCH_MAX: u64 = _; // 1_073_741_823u64
```

## skin_editor_infrastructure::voice_handoff::HandoffError

[源码:36](../../../desktop/infrastructure/src/voice_handoff.rs#L36) · enum

```rust
pub enum HandoffError {
    InvalidOrigin,
    InvalidFormat,
    WrongPool,
    Stale,
    Frozen,
    Closed,
    Busy,
    Exhausted,
    InvalidBlock,
    RuntimeMode,
    ForeignRuntime,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::separated_output::OutputError (声明)`, `skin_editor_infrastructure::separated_output::OutputError::from`, `skin_editor_infrastructure::voice_handoff::HandoffError::clone`, `skin_editor_infrastructure::voice_handoff::HandoffError::eq`, `skin_editor_infrastructure::voice_handoff::HandoffError::from`, `skin_editor_infrastructure::voice_handoff::VoiceConsumer::render_committed`, `skin_editor_infrastructure::voice_handoff::VoiceConsumer::render_inner`, `skin_editor_infrastructure::voice_handoff::VoiceConsumer::render_split`, `skin_editor_infrastructure::voice_handoff::VoiceConsumer::render_to`, `skin_editor_infrastructure::voice_handoff::VoiceOwner::admit`, `skin_editor_infrastructure::voice_handoff::VoiceOwner::invalidate_source`, `skin_editor_infrastructure::voice_handoff::VoiceOwner::publish`, `skin_editor_infrastructure::voice_handoff::VoiceOwner::pump_runtime`, `skin_editor_infrastructure::voice_handoff::VoiceOwner::set_scene_running`, `skin_editor_infrastructure::voice_handoff::VoiceOwner::stop_audition`, `skin_editor_infrastructure::voice_handoff::VoiceOwner::update`, `skin_editor_infrastructure::voice_handoff::boundary (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for HandoffError` — [源码:35](../../../desktop/infrastructure/src/voice_handoff.rs#L35)
- `impl Debug for HandoffError` — [源码:35](../../../desktop/infrastructure/src/voice_handoff.rs#L35)
- `impl From<HandoffError> for OutputError` — [源码:43](../../../desktop/infrastructure/src/separated_output.rs#L43)
- `impl PartialEq for HandoffError` — [源码:35](../../../desktop/infrastructure/src/voice_handoff.rs#L35)
- `impl Copy for HandoffError` — [源码:35](../../../desktop/infrastructure/src/voice_handoff.rs#L35)
- `impl Eq for HandoffError` — [源码:35](../../../desktop/infrastructure/src/voice_handoff.rs#L35)
- `impl StructuralPartialEq for HandoffError` — [源码:35](../../../desktop/infrastructure/src/voice_handoff.rs#L35)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> HandoffError` | impl Clone for HandoffError | [源码:35](../../../desktop/infrastructure/src/voice_handoff.rs#L35) | skin_editor_infrastructure::voice_handoff::HandoffError |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for HandoffError | [源码:35](../../../desktop/infrastructure/src/voice_handoff.rs#L35) |  |
| `fn from(e: HandoffError) -> Self` | impl From<HandoffError> for OutputError | [源码:44](../../../desktop/infrastructure/src/separated_output.rs#L44) | skin_editor_infrastructure::voice_handoff::HandoffError |
| `fn eq(&self, other: &HandoffError) -> bool` | impl PartialEq for HandoffError | [源码:35](../../../desktop/infrastructure/src/voice_handoff.rs#L35) | skin_editor_infrastructure::voice_handoff::HandoffError |

## skin_editor_infrastructure::voice_handoff::HandoffReport

[源码:114](../../../desktop/infrastructure/src/voice_handoff.rs#L114) · struct

```rust
pub struct HandoffReport {
    pub first: [Option<(VoiceKey, Channel)>; 32],
    pub runtime_first: [Option<(u64, Channel)>; 32],
    pub lane_epochs: [u64; 2],
    pub invalidated_lanes: [bool; 2],
    pub deferred_returns: usize,
}
```

声明引用：`skin_editor_audio::Channel`, `skin_editor_infrastructure::voice_handoff::VoiceKey`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::separated_output::OutputReport (声明)`, `skin_editor_infrastructure::voice_handoff::HandoffReport::clone`, `skin_editor_infrastructure::voice_handoff::HandoffReport::default`, `skin_editor_infrastructure::voice_handoff::VoiceConsumer::render_committed`, `skin_editor_infrastructure::voice_handoff::VoiceConsumer::render_inner`, `skin_editor_infrastructure::voice_handoff::VoiceConsumer::render_split`, `skin_editor_infrastructure::voice_handoff::VoiceConsumer::render_to`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for HandoffReport` — [源码:113](../../../desktop/infrastructure/src/voice_handoff.rs#L113)
- `impl Debug for HandoffReport` — [源码:113](../../../desktop/infrastructure/src/voice_handoff.rs#L113)
- `impl Default for HandoffReport` — [源码:113](../../../desktop/infrastructure/src/voice_handoff.rs#L113)
- `impl Copy for HandoffReport` — [源码:113](../../../desktop/infrastructure/src/voice_handoff.rs#L113)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> HandoffReport` | impl Clone for HandoffReport | [源码:113](../../../desktop/infrastructure/src/voice_handoff.rs#L113) | skin_editor_infrastructure::voice_handoff::HandoffReport |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for HandoffReport | [源码:113](../../../desktop/infrastructure/src/voice_handoff.rs#L113) |  |
| `fn default() -> HandoffReport` | impl Default for HandoffReport | [源码:113](../../../desktop/infrastructure/src/voice_handoff.rs#L113) | skin_editor_infrastructure::voice_handoff::HandoffReport |

## skin_editor_infrastructure::voice_handoff::MAX_HANDOFF_FRAMES

[源码:12](../../../desktop/infrastructure/src/voice_handoff.rs#L12) · constant

```rust
pub const MAX_HANDOFF_FRAMES: usize = 2048;
```

## skin_editor_infrastructure::voice_handoff::Packet

[源码:64](../../../desktop/infrastructure/src/voice_handoff.rs#L64) · struct

```rust
struct Packet {
    key: VoiceKey,
    token: VoiceToken,
    sound: PreparedSound,
    cursor: usize,
    started: bool,
    validity: Option<VoiceValidity>,
    runtime_voice: Option<u64>,
}
```

声明引用：`skin_editor_audio::PreparedSound`, `skin_editor_audio::VoiceValidity`, `skin_editor_infrastructure::voice_handoff::VoiceKey`, `skin_editor_infrastructure::voice_handoff::VoiceToken`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::voice_handoff::VoiceConsumer (声明)`, `skin_editor_infrastructure::voice_handoff::VoiceConsumer::return_packet`, `skin_editor_infrastructure::voice_handoff::VoiceOwner (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Drop for Packet` — [源码:73](../../../desktop/infrastructure/src/voice_handoff.rs#L73)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn drop(&mut self)` | impl Drop for Packet | [源码:74](../../../desktop/infrastructure/src/voice_handoff.rs#L74) |  |

## skin_editor_infrastructure::voice_handoff::Pin

[源码:78](../../../desktop/infrastructure/src/voice_handoff.rs#L78) · struct

```rust
struct Pin {
    key: VoiceKey,
    pcm: PcmBuffer,
    runtime: Option<VoiceHandoff>,
}
```

声明引用：`skin_editor_audio::PcmBuffer`, `skin_editor_audio::VoiceHandoff`, `skin_editor_infrastructure::voice_handoff::VoiceKey`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::voice_handoff::VoiceOwner (声明)`

## skin_editor_infrastructure::voice_handoff::RUNNING

[源码:15](../../../desktop/infrastructure/src/voice_handoff.rs#L15) · constant

```rust
const RUNNING: u64 = _; // 1_152_921_504_606_846_976u64
```

## skin_editor_infrastructure::voice_handoff::RuntimePumpReport

[源码:126](../../../desktop/infrastructure/src/voice_handoff.rs#L126) · struct

```rust
pub struct RuntimePumpReport {
    pub admitted: usize,
    pub retired: usize,
    pub waiting: usize,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::separated_output::PumpReport (声明)`, `skin_editor_infrastructure::voice_handoff::RuntimePumpReport::clone`, `skin_editor_infrastructure::voice_handoff::RuntimePumpReport::default`, `skin_editor_infrastructure::voice_handoff::RuntimePumpReport::eq`, `skin_editor_infrastructure::voice_handoff::VoiceOwner::pump_runtime`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for RuntimePumpReport` — [源码:125](../../../desktop/infrastructure/src/voice_handoff.rs#L125)
- `impl Debug for RuntimePumpReport` — [源码:125](../../../desktop/infrastructure/src/voice_handoff.rs#L125)
- `impl Default for RuntimePumpReport` — [源码:125](../../../desktop/infrastructure/src/voice_handoff.rs#L125)
- `impl PartialEq for RuntimePumpReport` — [源码:125](../../../desktop/infrastructure/src/voice_handoff.rs#L125)
- `impl Copy for RuntimePumpReport` — [源码:125](../../../desktop/infrastructure/src/voice_handoff.rs#L125)
- `impl Eq for RuntimePumpReport` — [源码:125](../../../desktop/infrastructure/src/voice_handoff.rs#L125)
- `impl StructuralPartialEq for RuntimePumpReport` — [源码:125](../../../desktop/infrastructure/src/voice_handoff.rs#L125)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> RuntimePumpReport` | impl Clone for RuntimePumpReport | [源码:125](../../../desktop/infrastructure/src/voice_handoff.rs#L125) | skin_editor_infrastructure::voice_handoff::RuntimePumpReport |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for RuntimePumpReport | [源码:125](../../../desktop/infrastructure/src/voice_handoff.rs#L125) |  |
| `fn default() -> RuntimePumpReport` | impl Default for RuntimePumpReport | [源码:125](../../../desktop/infrastructure/src/voice_handoff.rs#L125) | skin_editor_infrastructure::voice_handoff::RuntimePumpReport |
| `fn eq(&self, other: &RuntimePumpReport) -> bool` | impl PartialEq for RuntimePumpReport | [源码:125](../../../desktop/infrastructure/src/voice_handoff.rs#L125) | skin_editor_infrastructure::voice_handoff::RuntimePumpReport |

## skin_editor_infrastructure::voice_handoff::Shared

[源码:61](../../../desktop/infrastructure/src/voice_handoff.rs#L61) · struct

```rust
struct Shared {
    gate: AtomicU64,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::voice_handoff::VoiceConsumer (声明)`, `skin_editor_infrastructure::voice_handoff::VoiceOwner (声明)`, `skin_editor_infrastructure::voice_handoff::VoiceToken (声明)`

## skin_editor_infrastructure::voice_handoff::StoppedVoiceConsumer

[源码:106](../../../desktop/infrastructure/src/voice_handoff.rs#L106) · struct

```rust
pub struct StoppedVoiceConsumer(Box<VoiceConsumer>);
```

声明引用：`skin_editor_infrastructure::voice_handoff::VoiceConsumer`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::voice_handoff::VoiceConsumer::stop`, `skin_editor_infrastructure::voice_handoff::VoiceOwner::finish`, `skin_editor_infrastructure::voice_handoff::VoiceOwner::finish_runtime`

实现／约束（显式或 derive，无类继承）：

- `impl Debug for StoppedVoiceConsumer` — [源码:107](../../../desktop/infrastructure/src/voice_handoff.rs#L107)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for StoppedVoiceConsumer | [源码:108](../../../desktop/infrastructure/src/voice_handoff.rs#L108) |  |

## skin_editor_infrastructure::voice_handoff::VoiceConsumer

[源码:95](../../../desktop/infrastructure/src/voice_handoff.rs#L95) · struct

```rust
pub struct VoiceConsumer {
    shared: Arc<Shared>,
    pending: Consumer<Packet>,
    returned: Producer<Packet>,
    active: [Option<Packet>; 32],
    deferred: [Option<Packet>; 32],
    channels: usize,
    format: MonitorFormat,
}
```

声明引用：`skin_editor_audio::MonitorFormat`, `skin_editor_infrastructure::voice_handoff::Packet`, `skin_editor_infrastructure::voice_handoff::Shared`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::separated_output::OutputConsumer (声明)`, `skin_editor_infrastructure::voice_handoff::StoppedVoiceConsumer (声明)`, `skin_editor_infrastructure::voice_handoff::boundary (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl VoiceConsumer` — [源码:500](../../../desktop/infrastructure/src/voice_handoff.rs#L500)
- `impl Drop for VoiceConsumer` — [源码:486](../../../desktop/infrastructure/src/voice_handoff.rs#L486)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub(crate) fn format(&self) -> (u32, u16)` | impl VoiceConsumer | [源码:501](../../../desktop/infrastructure/src/voice_handoff.rs#L501) |  |
| `pub fn stop(self: Box<Self>) -> StoppedVoiceConsumer` | impl VoiceConsumer | [源码:504](../../../desktop/infrastructure/src/voice_handoff.rs#L504) | skin_editor_infrastructure::voice_handoff::StoppedVoiceConsumer |
| `fn drain_pending(&mut self)` | impl VoiceConsumer | [源码:508](../../../desktop/infrastructure/src/voice_handoff.rs#L508) |  |
| `fn return_packet(&mut self, packet: Packet)` | impl VoiceConsumer | [源码:518](../../../desktop/infrastructure/src/voice_handoff.rs#L518) | skin_editor_infrastructure::voice_handoff::Packet |
| `fn retire_slot(&mut self, slot: usize)` | impl VoiceConsumer | [源码:525](../../../desktop/infrastructure/src/voice_handoff.rs#L525) |  |
| `fn retire_all(&mut self)` | impl VoiceConsumer | [源码:530](../../../desktop/infrastructure/src/voice_handoff.rs#L530) |  |
| `fn flush_deferred(&mut self)` | impl VoiceConsumer | [源码:535](../../../desktop/infrastructure/src/voice_handoff.rs#L535) |  |
| `pub fn render_split( &mut self, scene: &mut [f32], audition: &mut [f32], ) -> Result<HandoffReport, HandoffError>` | impl VoiceConsumer | [源码:545](../../../desktop/infrastructure/src/voice_handoff.rs#L545) | skin_editor_infrastructure::voice_handoff::HandoffError, skin_editor_infrastructure::voice_handoff::HandoffReport |
| `fn render_inner( &mut self, scene: &mut [f32], audition: &mut [f32], checkpoint: impl FnMut(Checkpoint), ) -> Result<HandoffReport, HandoffError>` | impl VoiceConsumer | [源码:552](../../../desktop/infrastructure/src/voice_handoff.rs#L552) | skin_editor_infrastructure::voice_handoff::Checkpoint, skin_editor_infrastructure::voice_handoff::HandoffError, skin_editor_infrastructure::voice_handoff::HandoffReport |
| `pub(crate) fn render_to( &mut self, scene: &mut [f32], audition: &mut [f32], before_fence: impl FnMut(), copy: impl FnOnce(&[f32], &[f32], &mut HandoffReport), ) -> Result<HandoffReport, HandoffError>` | impl VoiceConsumer | [源码:562](../../../desktop/infrastructure/src/voice_handoff.rs#L562) | skin_editor_infrastructure::voice_handoff::HandoffError, skin_editor_infrastructure::voice_handoff::HandoffReport |
| `fn render_committed( &mut self, scene: &mut [f32], audition: &mut [f32], checkpoint: impl FnMut(Checkpoint), copy: impl FnOnce(&[f32], &[f32], &mut HandoffReport), ) -> Result<HandoffReport, HandoffError>` | impl VoiceConsumer | [源码:580](../../../desktop/infrastructure/src/voice_handoff.rs#L580) | skin_editor_infrastructure::voice_handoff::Checkpoint, skin_editor_infrastructure::voice_handoff::HandoffError, skin_editor_infrastructure::voice_handoff::HandoffReport |
| `fn drop(&mut self)` | impl Drop for VoiceConsumer | [源码:487](../../../desktop/infrastructure/src/voice_handoff.rs#L487) |  |

## skin_editor_infrastructure::voice_handoff::VoiceKey

[源码:56](../../../desktop/infrastructure/src/voice_handoff.rs#L56) · struct

```rust
pub struct VoiceKey {
    slot: usize,
    generation: u64,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::voice_handoff::HandoffReport (声明)`, `skin_editor_infrastructure::voice_handoff::Packet (声明)`, `skin_editor_infrastructure::voice_handoff::Pin (声明)`, `skin_editor_infrastructure::voice_handoff::VoiceKey::clone`, `skin_editor_infrastructure::voice_handoff::VoiceKey::eq`, `skin_editor_infrastructure::voice_handoff::VoiceOwner::admit`, `skin_editor_infrastructure::voice_handoff::VoiceOwner::publish`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for VoiceKey` — [源码:55](../../../desktop/infrastructure/src/voice_handoff.rs#L55)
- `impl Debug for VoiceKey` — [源码:55](../../../desktop/infrastructure/src/voice_handoff.rs#L55)
- `impl PartialEq for VoiceKey` — [源码:55](../../../desktop/infrastructure/src/voice_handoff.rs#L55)
- `impl Copy for VoiceKey` — [源码:55](../../../desktop/infrastructure/src/voice_handoff.rs#L55)
- `impl Eq for VoiceKey` — [源码:55](../../../desktop/infrastructure/src/voice_handoff.rs#L55)
- `impl StructuralPartialEq for VoiceKey` — [源码:55](../../../desktop/infrastructure/src/voice_handoff.rs#L55)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> VoiceKey` | impl Clone for VoiceKey | [源码:55](../../../desktop/infrastructure/src/voice_handoff.rs#L55) | skin_editor_infrastructure::voice_handoff::VoiceKey |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for VoiceKey | [源码:55](../../../desktop/infrastructure/src/voice_handoff.rs#L55) |  |
| `fn eq(&self, other: &VoiceKey) -> bool` | impl PartialEq for VoiceKey | [源码:55](../../../desktop/infrastructure/src/voice_handoff.rs#L55) | skin_editor_infrastructure::voice_handoff::VoiceKey |

## skin_editor_infrastructure::voice_handoff::VoiceOwner

[源码:84](../../../desktop/infrastructure/src/voice_handoff.rs#L84) · struct

```rust
pub struct VoiceOwner {
    shared: Arc<Shared>,
    pending: Option<Producer<Packet>>,
    returned: Consumer<Packet>,
    pins: [Option<Pin>; 32],
    budget: PcmBudget,
    format: (u32, u16),
    source: Origin,
    serial: u64,
    runtime: Option<VoiceRuntimeKey>,
}
```

声明引用：`skin_editor_audio::Origin`, `skin_editor_audio::PcmBudget`, `skin_editor_audio::VoiceRuntimeKey`, `skin_editor_infrastructure::voice_handoff::Packet`, `skin_editor_infrastructure::voice_handoff::Pin`, `skin_editor_infrastructure::voice_handoff::Shared`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::separated_output::BackgroundOwner (声明)`, `skin_editor_infrastructure::voice_handoff::boundary (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl VoiceOwner` — [源码:175](../../../desktop/infrastructure/src/voice_handoff.rs#L175)
- `impl Drop for VoiceOwner` — [源码:480](../../../desktop/infrastructure/src/voice_handoff.rs#L480)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub(crate) fn retained_drafts(&self) -> impl Iterator<Item = u64> + '_` | impl VoiceOwner | [源码:176](../../../desktop/infrastructure/src/voice_handoff.rs#L176) |  |
| `pub fn token(&self, channel: Channel) -> VoiceToken` | impl VoiceOwner | [源码:182](../../../desktop/infrastructure/src/voice_handoff.rs#L182) | skin_editor_audio::Channel, skin_editor_infrastructure::voice_handoff::VoiceToken |
| `pub fn outstanding(&self) -> usize` | impl VoiceOwner | [源码:189](../../../desktop/infrastructure/src/voice_handoff.rs#L189) |  |
| `pub fn admit( &mut self, source: Origin, token: VoiceToken, sound: &PreparedSound, ) -> Result<VoiceKey, HandoffError>` | impl VoiceOwner | [源码:192](../../../desktop/infrastructure/src/voice_handoff.rs#L192) | skin_editor_audio::Origin, skin_editor_audio::PreparedSound, skin_editor_infrastructure::voice_handoff::HandoffError, skin_editor_infrastructure::voice_handoff::VoiceKey, skin_editor_infrastructure::voice_handoff::VoiceToken |
| `fn publish( &mut self, source: Origin, token: VoiceToken, sound: &PreparedSound, runtime: Option<&VoiceHandoff>, ) -> Result<VoiceKey, HandoffError>` | impl VoiceOwner | [源码:203](../../../desktop/infrastructure/src/voice_handoff.rs#L203) | skin_editor_audio::Origin, skin_editor_audio::PreparedSound, skin_editor_audio::VoiceHandoff, skin_editor_infrastructure::voice_handoff::HandoffError, skin_editor_infrastructure::voice_handoff::VoiceKey, skin_editor_infrastructure::voice_handoff::VoiceToken |
| `pub fn pump_runtime( &mut self, runtime: &mut Runtime, ) -> Result<RuntimePumpReport, HandoffError>` | impl VoiceOwner | [源码:269](../../../desktop/infrastructure/src/voice_handoff.rs#L269) | skin_editor_audio::Runtime, skin_editor_infrastructure::voice_handoff::HandoffError, skin_editor_infrastructure::voice_handoff::RuntimePumpReport |
| `fn update( &mut self, lanes: [bool; 2], running: Option<bool>, ) -> Result<(), HandoffError>` | impl VoiceOwner | [源码:327](../../../desktop/infrastructure/src/voice_handoff.rs#L327) | skin_editor_infrastructure::voice_handoff::HandoffError |
| `pub fn set_scene_running(&mut self, running: bool) -> Result<(), HandoffError>` | impl VoiceOwner | [源码:369](../../../desktop/infrastructure/src/voice_handoff.rs#L369) | skin_editor_infrastructure::voice_handoff::HandoffError |
| `pub fn stop_audition(&mut self) -> Result<(), HandoffError>` | impl VoiceOwner | [源码:379](../../../desktop/infrastructure/src/voice_handoff.rs#L379) | skin_editor_infrastructure::voice_handoff::HandoffError |
| `pub fn invalidate_source(&mut self, next: Origin) -> Result<(), HandoffError>` | impl VoiceOwner | [源码:383](../../../desktop/infrastructure/src/voice_handoff.rs#L383) | skin_editor_audio::Origin, skin_editor_infrastructure::voice_handoff::HandoffError |
| `pub fn close(&mut self)` | impl VoiceOwner | [源码:396](../../../desktop/infrastructure/src/voice_handoff.rs#L396) |  |
| `pub fn collect(&mut self) -> usize` | impl VoiceOwner | [源码:399](../../../desktop/infrastructure/src/voice_handoff.rs#L399) |  |
| `fn collect_with(&mut self, finished: impl FnMut(VoiceHandoff)) -> usize` | impl VoiceOwner | [源码:407](../../../desktop/infrastructure/src/voice_handoff.rs#L407) | skin_editor_audio::VoiceHandoff |
| `pub fn finish( &mut self, stopped: StoppedVoiceConsumer, ) -> Result<(), StoppedVoiceConsumer>` | impl VoiceOwner | [源码:443](../../../desktop/infrastructure/src/voice_handoff.rs#L443) | skin_editor_infrastructure::voice_handoff::StoppedVoiceConsumer |
| `pub fn finish_runtime( &mut self, runtime: &mut Runtime, stopped: StoppedVoiceConsumer, ) -> Result<(), StoppedVoiceConsumer>` | impl VoiceOwner | [源码:462](../../../desktop/infrastructure/src/voice_handoff.rs#L462) | skin_editor_audio::Runtime, skin_editor_infrastructure::voice_handoff::StoppedVoiceConsumer |
| `fn drop(&mut self)` | impl Drop for VoiceOwner | [源码:481](../../../desktop/infrastructure/src/voice_handoff.rs#L481) |  |

## skin_editor_infrastructure::voice_handoff::VoiceToken

[源码:50](../../../desktop/infrastructure/src/voice_handoff.rs#L50) · struct

```rust
pub struct VoiceToken {
    boundary: Weak<Shared>,
    epoch: u64,
    channel: Channel,
}
```

声明引用：`skin_editor_audio::Channel`, `skin_editor_infrastructure::voice_handoff::Shared`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::voice_handoff::Packet (声明)`, `skin_editor_infrastructure::voice_handoff::VoiceOwner::admit`, `skin_editor_infrastructure::voice_handoff::VoiceOwner::publish`, `skin_editor_infrastructure::voice_handoff::VoiceOwner::token`, `skin_editor_infrastructure::voice_handoff::VoiceToken::clone`, `skin_editor_infrastructure::voice_handoff::permits (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for VoiceToken` — [源码:49](../../../desktop/infrastructure/src/voice_handoff.rs#L49)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> VoiceToken` | impl Clone for VoiceToken | [源码:49](../../../desktop/infrastructure/src/voice_handoff.rs#L49) | skin_editor_infrastructure::voice_handoff::VoiceToken |

## skin_editor_infrastructure::voice_handoff::boundary

[源码:135](../../../desktop/infrastructure/src/voice_handoff.rs#L135) · fn

```rust
pub fn boundary(
    budget: PcmBudget,
    rate: u32,
    channels: u16,
    source: Origin,
) -> Result<(VoiceOwner, Box<VoiceConsumer>), HandoffError>
```

声明引用：`skin_editor_audio::Origin`, `skin_editor_audio::PcmBudget`, `skin_editor_infrastructure::voice_handoff::HandoffError`, `skin_editor_infrastructure::voice_handoff::VoiceConsumer`, `skin_editor_infrastructure::voice_handoff::VoiceOwner`

## skin_editor_infrastructure::voice_handoff::epoch

[源码:23](../../../desktop/infrastructure/src/voice_handoff.rs#L23) · fn

```rust
fn epoch(word: u64, channel: Channel) -> u64
```

声明引用：`skin_editor_audio::Channel`

## skin_editor_infrastructure::voice_handoff::index

[源码:17](../../../desktop/infrastructure/src/voice_handoff.rs#L17) · fn

```rust
fn index(channel: Channel) -> usize
```

声明引用：`skin_editor_audio::Channel`

## skin_editor_infrastructure::voice_handoff::permits

[源码:26](../../../desktop/infrastructure/src/voice_handoff.rs#L26) · fn

```rust
fn permits(word: u64, token: &VoiceToken) -> bool
```

声明引用：`skin_editor_infrastructure::voice_handoff::VoiceToken`

## skin_editor_infrastructure::voice_handoff::valid_origin

[源码:31](../../../desktop/infrastructure/src/voice_handoff.rs#L31) · fn

```rust
fn valid_origin(origin: Origin) -> bool
```

声明引用：`skin_editor_audio::Origin`

## skin_editor_infrastructure::wav::Wave

[源码:8](../../../desktop/infrastructure/src/wav.rs#L8) · struct

```rust
struct Wave<'a> {
    metadata: AudioMetadata,
    data: &'a [u8],
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::wav::parse (声明)`

## skin_editor_infrastructure::wav::parse

[源码:39](../../../desktop/infrastructure/src/wav.rs#L39) · fn

```rust
fn parse(bytes: &[u8]) -> Result<Wave<'_>, PrepareFailure>
```

声明引用：`skin_editor_audio::PrepareFailure`, `skin_editor_infrastructure::wav::Wave`

## skin_editor_infrastructure::wav::pcm16_format

[源码:18](../../../desktop/infrastructure/src/wav.rs#L18) · fn

```rust
pub(crate) fn pcm16_format(chunk: &[u8]) -> Result<(u32, u16), PrepareFailure>
```

声明引用：`skin_editor_audio::PrepareFailure`

## skin_editor_infrastructure::wav::u16le

[源码:12](../../../desktop/infrastructure/src/wav.rs#L12) · fn

```rust
fn u16le(bytes: &[u8]) -> u16
```

## skin_editor_infrastructure::wav::u32le

[源码:15](../../../desktop/infrastructure/src/wav.rs#L15) · fn

```rust
fn u32le(bytes: &[u8]) -> u32
```

## skin_editor_infrastructure::write_png

[源码:242](../../../desktop/infrastructure/src/lib.rs#L242) · fn

```rust
pub fn write_png(
    path: &Path,
    pixels: &[u8],
    size: PixelSize,
) -> Result<(), String>
```

