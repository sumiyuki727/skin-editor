# skin_editor_desktop：编译器符号清单

快照：2026-10-30。来自含私有项的 rustdoc；显示编译器规范化可见域。含 derive 生成的实现，不把它们视作类继承。来源行可能指向宏定义／展开点。
引用列只列 rustdoc 解析出的项目类型链接，不能充当完整调用图；无链接不表示无引用。所有方法保留源码位置供逐项复核。测试和未启用的 cfg 分支另见源码索引。

## skin_editor_desktop::Editor

[源码:36](../../../desktop/host/src/main.rs#L36) · struct

```rust
pub(crate) struct Editor {
    pub(crate) session: ProjectSession,
    pub(crate) ui: EditorUi,
    pub(crate) images: Images,
    pub(crate) host: NativeHost,
    pub(crate) preview: PreviewState,
    pub(crate) last: Instant,
    pub(crate) importer: ImportWorker,
    pub(crate) smoke_started: Option<Instant>,
    pub(crate) ui_passes: u64,
    pub(crate) main_work: Option<RefCell<MainWork>>,
    pub(crate) repaint_pass: RepaintPass,
}
```

声明引用：`skin_editor_desktop::import_worker::ImportWorker`, `skin_editor_desktop::main_work::MainWork`, `skin_editor_desktop::presentation::NativeHost`, `skin_editor_desktop::repaint_deadline::RepaintPass`, `skin_editor_infrastructure::Images`, `skin_editor_preview::PreviewState`, `skin_editor_ui::EditorUi`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::combined_verification::Combined (声明)`, `skin_editor_desktop::combined_verification::build_fixture (声明)`, `skin_editor_desktop::combined_verification::fixture (声明)`, `skin_editor_desktop::combined_verification::reopen_fixture (声明)`, `skin_editor_desktop::combined_verification::visual_fixture (声明)`, `skin_editor_desktop::editor_audio::combined::Run (声明)`, `skin_editor_desktop::editor_audio::cycles::Round (声明)`, `skin_editor_desktop::editor_audio::gui_verification::Run (声明)`, `skin_editor_desktop::editor_audio::stability::Run (声明)`, `skin_editor_desktop::editor_audio::window_recovery::Returned (声明)`, `skin_editor_desktop::editor_audio::window_recovery::Run (声明)`, `skin_editor_desktop::editor_audio::window_recovery::Seed::restore`, `skin_editor_desktop::editor_audio::window_recovery::activity::draft (声明)`, `skin_editor_desktop::editor_audio::window_recovery::activity::exercise (声明)`, `skin_editor_desktop::editor_audio::window_recovery::activity::pass (声明)`, `skin_editor_desktop::editor_audio::window_recovery::activity::ready_frame (声明)`, `skin_editor_desktop::editor_audio::window_recovery::init_failure::fixture (声明)`, `skin_editor_desktop::editor_audio::window_recovery::restore::Attempt::commit`, `skin_editor_desktop::editor_audio::window_recovery::restore::Attempt::editor`, `skin_editor_desktop::editor_audio::window_recovery::restore::Attempt::fresh`, `skin_editor_desktop::editor_audio::window_recovery::restore::Cleanup::step`, `skin_editor_desktop::editor_audio::window_recovery::restore::Owner (声明)`, `skin_editor_desktop::editor_audio::window_recovery::sound::install (声明)`, `skin_editor_desktop::editor_audio::window_recovery::sound::reopen (声明)`, `skin_editor_desktop::editor_audio::window_recovery::sound::trigger (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Editor` — [源码:88](../../../desktop/host/src/main.rs#L88)
- `impl Editor` — [源码:266](../../../desktop/host/src/main.rs#L266)
- `impl App for Editor` — [源码:161](../../../desktop/host/src/main.rs#L161)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub(crate) fn new( context: &CreationContext<'_>, smoke: bool, ) -> Result<Self, String>` | impl Editor | [源码:89](../../../desktop/host/src/main.rs#L89) |  |
| `pub(crate) fn with_upload( context: &CreationContext<'_>, smoke: bool, upload: SourceUpload, ) -> Result<Self, String>` | impl Editor | [源码:96](../../../desktop/host/src/main.rs#L96) | skin_editor_rendering::SourceUpload |
| `pub(crate) fn with_render_state( ctx: &Context, state: &RenderState, smoke: bool, upload: SourceUpload, ) -> Result<Self, String>` | impl Editor | [源码:107](../../../desktop/host/src/main.rs#L107) | skin_editor_rendering::SourceUpload |
| `pub(crate) fn with_initial_content( ctx: &Context, state: &RenderState, smoke: bool, upload: SourceUpload, include_sample: bool, ) -> Result<Self, String>` | impl Editor | [源码:115](../../../desktop/host/src/main.rs#L115) | skin_editor_rendering::SourceUpload |
| `pub(crate) fn work_edge(&self, category: u8)` | impl Editor | [源码:155](../../../desktop/host/src/main.rs#L155) |  |
| `pub(crate) fn interrupt_for_fault(&mut self) -> bool` | impl Editor | [源码:267](../../../desktop/host/src/main.rs#L267) |  |
| `pub(crate) fn complete_ui(&mut self, ctx: &Context, output: UiOutput)` | impl Editor | [源码:292](../../../desktop/host/src/main.rs#L292) | skin_editor_ui::UiOutput |
| `fn logic(&mut self, ctx: &Context, _: &mut Frame)` | impl App for Editor | [源码:162](../../../desktop/host/src/main.rs#L162) |  |
| `fn ui(&mut self, root: &mut Ui, _: &mut Frame)` | impl App for Editor | [源码:207](../../../desktop/host/src/main.rs#L207) |  |
| `fn on_exit(&mut self)` | impl App for Editor | [源码:253](../../../desktop/host/src/main.rs#L253) |  |

## skin_editor_desktop::audio_verification::component

[源码:32](../../../desktop/host/src/audio_verification.rs#L32) · fn

```rust
pub(crate) fn component() -> AudioComponentId
```

## skin_editor_desktop::audio_verification::phase

[源码:52](../../../desktop/host/src/audio_verification.rs#L52) · fn

```rust
fn phase(
    audio: &mut AudioPlayback,
    session: &ProjectSession,
    store: &BlobStore,
    native: &mut NativeOutput,
    millis: u64,
) -> Result<(), String>
```

声明引用：`skin_editor_infrastructure::AudioPlayback`, `skin_editor_infrastructure::BlobStore`, `skin_editor_infrastructure::NativeOutput`

## skin_editor_desktop::audio_verification::quiet

[源码:79](../../../desktop/host/src/audio_verification.rs#L79) · fn

```rust
fn quiet(
    audio: &mut AudioPlayback,
    session: &ProjectSession,
    store: &BlobStore,
    native: &mut NativeOutput,
) -> Result<(), String>
```

声明引用：`skin_editor_infrastructure::AudioPlayback`, `skin_editor_infrastructure::BlobStore`, `skin_editor_infrastructure::NativeOutput`

## skin_editor_desktop::audio_verification::run

[源码:101](../../../desktop/host/src/audio_verification.rs#L101) · fn

```rust
pub fn run(destination: &str) -> Result<(), String>
```

## skin_editor_desktop::audio_verification::settle

[源码:67](../../../desktop/host/src/audio_verification.rs#L67) · fn

```rust
fn settle(
    audio: &mut AudioPlayback,
    session: &ProjectSession,
    store: &BlobStore,
    native: &mut NativeOutput,
) -> Result<(), String>
```

声明引用：`skin_editor_infrastructure::AudioPlayback`, `skin_editor_infrastructure::BlobStore`, `skin_editor_infrastructure::NativeOutput`

## skin_editor_desktop::audio_verification::submit

[源码:35](../../../desktop/host/src/audio_verification.rs#L35) · fn

```rust
pub(crate) fn submit(
    session: &mut ProjectSession,
    next: &mut u128,
    commands: Vec<Command>,
) -> Result<(), String>
```

## skin_editor_desktop::audio_verification::tone

[源码:7](../../../desktop/host/src/audio_verification.rs#L7) · fn

```rust
pub(crate) fn tone(rate: u32, channels: u16, hz: f32) -> Vec<u8> ⓘ
```

## skin_editor_desktop::combined_verification::Combined

[源码:482](../../../desktop/host/src/combined_verification.rs#L482) · struct

```rust
struct Combined {
    timing_origin: Instant,
    timing: String,
    cadence: Cadence,
    late_supply: Vec<String>,
    diagnostic_cycle: Option<usize>,
    hold_completion: bool,
    completed: bool,
    control_only: bool,
    cold_control: bool,
    memory_usage: bool,
    reusable_staging: bool,
    editor: Editor,
    path: PathBuf,
    budget: PcmBudget,
    client: Option<AudioSupplyClient>,
    stream: Option<NativeStream>,
    output: Option<OutputControl>,
    final_stats: OutputStats,
    playing_frames: u64,
    observer: Observer,
    next: u128,
    started: Option<Instant>,
    frames_at_start: u64,
    loading: Instant,
    closing: Option<Instant>,
    failed: Option<String>,
    done: bool,
    triggers: usize,
    edits: usize,
    switches: usize,
    probes: usize,
    edit_pending: Option<Instant>,
    edit_ms: Vec<f64>,
    frame_ms: Vec<f64>,
    slow_ui: Vec<[f64; 4]>,
    latencies: [Vec<f64>; 2],
    intervals: [Vec<[f64; 5]>; 2],
    causal: [Vec<[f64; 13]>; 2],
    gpu_snapshots: Vec<String>,
    exact_intervals: [Vec<[f64; 2]>; 2],
    completion_markers: [Vec<[u64; 7]>; 2],
    requested_voices: [Vec<u64>; 2],
    request_phases: [Vec<[f64; 2]>; 2],
    request_refs: [Vec<[u64; 4]>; 2],
    presentation: Option<String>,
    max_probe_poll_gap_ms: f64,
    max_supply_gap_micros: u64,
    underrun_observations: Vec<[u64; 6]>,
    max_targets: usize,
    max_rgba: usize,
    max_voices: usize,
    max_trigger_lateness_ms: f64,
    rate: u32,
    channels: u16,
    period: u32,
    adapter: String,
    cleanup: bool,
    isolation: bool,
    draft_history_cleanup: bool,
    opacities: [f32; 10],
}
```

声明引用：`skin_editor_audio::PcmBudget`, `skin_editor_desktop::Editor`, `skin_editor_desktop::combined_verification::Observer`, `skin_editor_desktop::combined_verification::cadence::Cadence`, `skin_editor_infrastructure::AudioSupplyClient`, `skin_editor_infrastructure::NativeStream`, `skin_editor_infrastructure::OutputControl`, `skin_editor_infrastructure::OutputStats`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::combined_verification::recovery::Recovery (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Combined` — [源码:544](../../../desktop/host/src/combined_verification.rs#L544)
- `impl Combined` — [源码:1241](../../../desktop/host/src/combined_verification.rs#L1241)
- `impl App for Combined` — [源码:1299](../../../desktop/host/src/combined_verification.rs#L1299)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn new( context: &CreationContext<'_>, path: PathBuf, control_only: bool, memory_usage: bool, cold_control: bool, reusable_staging: bool, ) -> Result<Self, String>` | impl Combined | [源码:545](../../../desktop/host/src/combined_verification.rs#L545) |  |
| `fn create( ctx: &Context, state: &RenderState, path: PathBuf, control_only: bool, memory_usage: bool, cold_control: bool, reusable_staging: bool, ) -> Result<Self, String>` | impl Combined | [源码:563](../../../desktop/host/src/combined_verification.rs#L563) |  |
| `fn tick(&mut self) -> Result<(), String>` | impl Combined | [源码:670](../../../desktop/host/src/combined_verification.rs#L670) |  |
| `fn finish(&mut self) -> Result<(), String>` | impl Combined | [源码:957](../../../desktop/host/src/combined_verification.rs#L957) |  |
| `fn report(&mut self, file: &str) -> Result<(), String>` | impl Combined | [源码:1111](../../../desktop/host/src/combined_verification.rs#L1111) |  |
| `fn ns(&self) -> u64` | impl Combined | [源码:1242](../../../desktop/host/src/combined_verification.rs#L1242) |  |
| `fn logic_inner(&mut self, ctx: &Context, frame: &mut Frame)` | impl Combined | [源码:1245](../../../desktop/host/src/combined_verification.rs#L1245) |  |
| `fn logic(&mut self, ctx: &Context, frame: &mut Frame)` | impl App for Combined | [源码:1300](../../../desktop/host/src/combined_verification.rs#L1300) |  |
| `fn ui(&mut self, ui: &mut Ui, frame: &mut Frame)` | impl App for Combined | [源码:1322](../../../desktop/host/src/combined_verification.rs#L1322) |  |
| `fn on_exit(&mut self)` | impl App for Combined | [源码:1370](../../../desktop/host/src/combined_verification.rs#L1370) |  |

## skin_editor_desktop::combined_verification::EDITS

[源码:21](../../../desktop/host/src/combined_verification.rs#L21) · constant

```rust
const EDITS: usize = 300;
```

## skin_editor_desktop::combined_verification::Observer

[源码:151](../../../desktop/host/src/combined_verification.rs#L151) · struct

```rust
struct Observer {
    sender: Option<SyncSender<Probe>>,
    thread: Option<JoinHandle<Result<ProbeObservations, String>>>,
}
```

声明引用：`skin_editor_desktop::combined_verification::Probe`, `skin_editor_desktop::combined_verification::ProbeObservations`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::combined_verification::Combined (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Observer` — [源码:204](../../../desktop/host/src/combined_verification.rs#L204)
- `impl Drop for Observer` — [源码:346](../../../desktop/host/src/combined_verification.rs#L346)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn new(output: OutputControl) -> Self` | impl Observer | [源码:205](../../../desktop/host/src/combined_verification.rs#L205) | skin_editor_infrastructure::OutputControl |
| `fn with_capacity(output: OutputControl, capacity: usize) -> Result<Self, String>` | impl Observer | [源码:208](../../../desktop/host/src/combined_verification.rs#L208) | skin_editor_infrastructure::OutputControl |
| `fn finish(&mut self) -> Result<ProbeObservations, String>` | impl Observer | [源码:337](../../../desktop/host/src/combined_verification.rs#L337) | skin_editor_desktop::combined_verification::ProbeObservations |
| `fn drop(&mut self)` | impl Drop for Observer | [源码:347](../../../desktop/host/src/combined_verification.rs#L347) |  |

## skin_editor_desktop::combined_verification::Probe

[源码:141](../../../desktop/host/src/combined_verification.rs#L141) · struct

```rust
struct Probe {
    callback_reference: Option<[u64; 4]>,
    completed: Option<SyncSender<()>>,
    started: Instant,
    receipt_ms: f64,
    started_in_play_ms: f64,
    voice: u64,
    lane: usize,
    clock: Option<StreamClockSample>,
}
```

声明引用：`skin_editor_infrastructure::StreamClockSample`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::combined_verification::Observer (声明)`

## skin_editor_desktop::combined_verification::ProbeObservations

[源码:155](../../../desktop/host/src/combined_verification.rs#L155) · struct

```rust
struct ProbeObservations {
    latencies: [Vec<f64>; 2],
    intervals: [Vec<[f64; 5]>; 2],
    causal: [Vec<[f64; 13]>; 2],
    exact: [Vec<[f64; 2]>; 2],
    markers: [Vec<[u64; 7]>; 2],
    requested: [Vec<u64>; 2],
    request_phases: [Vec<[f64; 2]>; 2],
    request_refs: [Vec<[u64; 4]>; 2],
    max_poll_gap_ms: f64,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::combined_verification::Observer (声明)`, `skin_editor_desktop::combined_verification::Observer::finish`

## skin_editor_desktop::combined_verification::SECONDS

[源码:19](../../../desktop/host/src/combined_verification.rs#L19) · constant

```rust
const SECONDS: u64 = 300;
```

## skin_editor_desktop::combined_verification::SWITCHES

[源码:22](../../../desktop/host/src/combined_verification.rs#L22) · constant

```rust
const SWITCHES: usize = 20;
```

## skin_editor_desktop::combined_verification::SubmissionBracket

[源码:169](../../../desktop/host/src/combined_verification.rs#L169) · struct

```rust
struct SubmissionBracket {
    lower_ms: f64,
    positive_ms: Option<f64>,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::combined_verification::SubmissionBracket::default`

实现／约束（显式或 derive，无类继承）：

- `impl SubmissionBracket` — [源码:173](../../../desktop/host/src/combined_verification.rs#L173)
- `impl Default for SubmissionBracket` — [源码:168](../../../desktop/host/src/combined_verification.rs#L168)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn completion( &mut self, before: f64, after: f64, value: VoiceSubmission, ) -> Result<Option<[f64; 2]>, String>` | impl SubmissionBracket | [源码:174](../../../desktop/host/src/combined_verification.rs#L174) | skin_editor_infrastructure::VoiceSubmission |
| `fn observe( &mut self, before_ms: f64, after_ms: f64, submitted: bool, ) -> Option<[f64; 2]>` | impl SubmissionBracket | [源码:190](../../../desktop/host/src/combined_verification.rs#L190) |  |
| `fn default() -> SubmissionBracket` | impl Default for SubmissionBracket | [源码:168](../../../desktop/host/src/combined_verification.rs#L168) | skin_editor_desktop::combined_verification::SubmissionBracket |

## skin_editor_desktop::combined_verification::TRIGGERS

[源码:20](../../../desktop/host/src/combined_verification.rs#L20) · constant

```rust
const TRIGGERS: usize = 2400;
```

## skin_editor_desktop::combined_verification::build_fixture

[源码:364](../../../desktop/host/src/combined_verification.rs#L364) · fn

```rust
fn build_fixture(
    editor: &mut Editor,
    path: &Path,
    include_audio: bool,
    create_images: bool,
) -> Result<u128, String>
```

声明引用：`skin_editor_desktop::Editor`

## skin_editor_desktop::combined_verification::cadence::Cadence

[源码:2](../../../desktop/host/src/combined_verification/cadence.rs#L2) · struct

```rust
pub(super) struct Cadence {
    current: [u64; 7],
    pub calls: u64,
    pub late_count: u64,
    pub rows: Vec<[u64; 11]>,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::combined_verification::Combined (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Cadence` — [源码:19](../../../desktop/host/src/combined_verification/cadence.rs#L19)
- `impl Default for Cadence` — [源码:9](../../../desktop/host/src/combined_verification/cadence.rs#L9)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn begin(&mut self, now: u64)` | impl Cadence | [源码:20](../../../desktop/host/src/combined_verification/cadence.rs#L20) |  |
| `pub fn tick_end(&mut self, now: u64)` | impl Cadence | [源码:36](../../../desktop/host/src/combined_verification/cadence.rs#L36) |  |
| `pub fn end(&mut self, now: u64)` | impl Cadence | [源码:42](../../../desktop/host/src/combined_verification/cadence.rs#L42) |  |
| `pub fn repaint(&mut self, now: u64, delay: u64)` | impl Cadence | [源码:48](../../../desktop/host/src/combined_verification/cadence.rs#L48) |  |
| `pub fn ui(&mut self, now: u64, end: bool)` | impl Cadence | [源码:51](../../../desktop/host/src/combined_verification/cadence.rs#L51) |  |
| `fn default() -> Self` | impl Default for Cadence | [源码:10](../../../desktop/host/src/combined_verification/cadence.rs#L10) |  |

## skin_editor_desktop::combined_verification::cleared

[源码:122](../../../desktop/host/src/combined_verification.rs#L122) · fn

```rust
fn cleared(
    client: &mut AudioSupplyClient,
    output: &OutputControl,
) -> Result<bool, String>
```

声明引用：`skin_editor_infrastructure::AudioSupplyClient`, `skin_editor_infrastructure::OutputControl`

## skin_editor_desktop::combined_verification::clock_alignment

[源码:36](../../../desktop/host/src/combined_verification.rs#L36) · fn

```rust
fn clock_alignment(
    output: &OutputControl,
    stream: &NativeStream,
) -> Result<[u64; 3], String>
```

声明引用：`skin_editor_infrastructure::NativeStream`, `skin_editor_infrastructure::OutputControl`

## skin_editor_desktop::combined_verification::cold_run

[源码:1423](../../../desktop/host/src/combined_verification.rs#L1423) · fn

```rust
pub fn cold_run(destination: &str, memory_usage: bool) -> Result<(), String>
```

## skin_editor_desktop::combined_verification::control_run

[源码:1417](../../../desktop/host/src/combined_verification.rs#L1417) · fn

```rust
pub fn control_run(destination: &str) -> Result<(), String>
```

## skin_editor_desktop::combined_verification::document

[源码:61](../../../desktop/host/src/combined_verification.rs#L61) · fn

```rust
fn document(index: usize) -> DocumentId
```

## skin_editor_desktop::combined_verification::error

[源码:58](../../../desktop/host/src/combined_verification.rs#L58) · fn

```rust
fn error(e: impl Debug) -> String
```

## skin_editor_desktop::combined_verification::fixture

[源码:355](../../../desktop/host/src/combined_verification.rs#L355) · fn

```rust
pub(crate) fn fixture(editor: &mut Editor, path: &Path) -> Result<u128, String>
```

声明引用：`skin_editor_desktop::Editor`

## skin_editor_desktop::combined_verification::layer

[源码:67](../../../desktop/host/src/combined_verification.rs#L67) · fn

```rust
fn layer(index: usize) -> LayerId
```

## skin_editor_desktop::combined_verification::memory_control_run

[源码:1420](../../../desktop/host/src/combined_verification.rs#L1420) · fn

```rust
pub fn memory_control_run(destination: &str) -> Result<(), String>
```

## skin_editor_desktop::combined_verification::memory_run

[源码:1414](../../../desktop/host/src/combined_verification.rs#L1414) · fn

```rust
pub fn memory_run(destination: &str) -> Result<(), String>
```

## skin_editor_desktop::combined_verification::optional_rows

[源码:30](../../../desktop/host/src/combined_verification.rs#L30) · fn

```rust
fn optional_rows<const N: usize>(
    rows: impl IntoIterator<Item = Option<[u64; N]>>,
) -> String
```

## skin_editor_desktop::combined_verification::phase::BINS

[源码:4](../../../desktop/host/src/combined_verification/phase.rs#L4) · constant

```rust
const BINS: usize = 20;
```

## skin_editor_desktop::combined_verification::phase::COUNT

[源码:6](../../../desktop/host/src/combined_verification/phase.rs#L6) · constant

```rust
const COUNT: usize = _; // 160usize
```

## skin_editor_desktop::combined_verification::phase::REFERENCE_ATTEMPTS

[源码:7](../../../desktop/host/src/combined_verification/phase.rs#L7) · constant

```rust
const REFERENCE_ATTEMPTS: usize = 32;
```

## skin_editor_desktop::combined_verification::phase::REPEATS

[源码:5](../../../desktop/host/src/combined_verification/phase.rs#L5) · constant

```rust
const REPEATS: usize = 8;
```

## skin_editor_desktop::combined_verification::phase::RecoveryProbeMode

[源码:9](../../../desktop/host/src/combined_verification/phase.rs#L9) · enum

```rust
enum RecoveryProbeMode {
    Natural,
    WithTrigger,
    AfterRefill,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::combined_verification::phase::RecoveryProbeMode::clone`, `skin_editor_desktop::combined_verification::phase::RecoveryProbeMode::eq`, `skin_editor_desktop::combined_verification::phase::run_with_policy (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for RecoveryProbeMode` — [源码:8](../../../desktop/host/src/combined_verification/phase.rs#L8)
- `impl Debug for RecoveryProbeMode` — [源码:8](../../../desktop/host/src/combined_verification/phase.rs#L8)
- `impl PartialEq for RecoveryProbeMode` — [源码:8](../../../desktop/host/src/combined_verification/phase.rs#L8)
- `impl Copy for RecoveryProbeMode` — [源码:8](../../../desktop/host/src/combined_verification/phase.rs#L8)
- `impl Eq for RecoveryProbeMode` — [源码:8](../../../desktop/host/src/combined_verification/phase.rs#L8)
- `impl StructuralPartialEq for RecoveryProbeMode` — [源码:8](../../../desktop/host/src/combined_verification/phase.rs#L8)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> RecoveryProbeMode` | impl Clone for RecoveryProbeMode | [源码:8](../../../desktop/host/src/combined_verification/phase.rs#L8) | skin_editor_desktop::combined_verification::phase::RecoveryProbeMode |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for RecoveryProbeMode | [源码:8](../../../desktop/host/src/combined_verification/phase.rs#L8) |  |
| `fn eq(&self, other: &RecoveryProbeMode) -> bool` | impl PartialEq for RecoveryProbeMode | [源码:8](../../../desktop/host/src/combined_verification/phase.rs#L8) | skin_editor_desktop::combined_verification::phase::RecoveryProbeMode |

## skin_editor_desktop::combined_verification::phase::callback_reference

[源码:29](../../../desktop/host/src/combined_verification/phase.rs#L29) · fn

```rust
fn callback_reference(
    output: &OutputControl,
) -> Result<([u64; 4], usize), String>
```

声明引用：`skin_editor_infrastructure::OutputControl`

## skin_editor_desktop::combined_verification::phase::future_phase

[源码:42](../../../desktop/host/src/combined_verification/phase.rs#L42) · fn

```rust
fn future_phase(
    now: u64,
    entry: u64,
    period: u64,
    bin: usize,
) -> Result<u64, String>
```

## skin_editor_desktop::combined_verification::phase::reference

[源码:15](../../../desktop/host/src/combined_verification/phase.rs#L15) · fn

```rust
fn reference(
    read: impl FnMut() -> Option<[u64; 4]>,
    pause: impl FnMut() -> Result<(), String>,
) -> Result<([u64; 4], usize), String>
```

## skin_editor_desktop::combined_verification::phase::run

[源码:54](../../../desktop/host/src/combined_verification/phase.rs#L54) · fn

```rust
pub fn run(destination: &str) -> Result<(), String>
```

## skin_editor_desktop::combined_verification::phase::run_controlled

[源码:68](../../../desktop/host/src/combined_verification/phase.rs#L68) · fn

```rust
pub fn run_controlled(
    destination: &str,
    after_refill: bool,
) -> Result<(), String>
```

## skin_editor_desktop::combined_verification::phase::run_recovery

[源码:61](../../../desktop/host/src/combined_verification/phase.rs#L61) · fn

```rust
pub fn run_recovery(destination: &str) -> Result<(), String>
```

## skin_editor_desktop::combined_verification::phase::run_with_policy

[源码:79](../../../desktop/host/src/combined_verification/phase.rs#L79) · fn

```rust
fn run_with_policy(
    destination: &str,
    policy: SupplyPolicy,
    mode: RecoveryProbeMode,
) -> Result<(), String>
```

声明引用：`skin_editor_desktop::combined_verification::phase::RecoveryProbeMode`, `skin_editor_infrastructure::SupplyPolicy`

## skin_editor_desktop::combined_verification::play_old

[源码:108](../../../desktop/host/src/combined_verification.rs#L108) · fn

```rust
fn play_old(
    client: &mut AudioSupplyClient,
    output: &OutputControl,
    session: &ProjectSession,
) -> Result<(), String>
```

声明引用：`skin_editor_infrastructure::AudioSupplyClient`, `skin_editor_infrastructure::OutputControl`

## skin_editor_desktop::combined_verification::ready_all

[源码:70](../../../desktop/host/src/combined_verification.rs#L70) · fn

```rust
fn ready_all(
    client: &mut AudioSupplyClient,
    output: &OutputControl,
    session: &ProjectSession,
) -> Result<(), String>
```

声明引用：`skin_editor_infrastructure::AudioSupplyClient`, `skin_editor_infrastructure::OutputControl`

## skin_editor_desktop::combined_verification::recovery::CYCLES

[源码:7](../../../desktop/host/src/combined_verification/recovery.rs#L7) · constant

```rust
const CYCLES: usize = 20;
```

## skin_editor_desktop::combined_verification::recovery::Recovery

[源码:25](../../../desktop/host/src/combined_verification/recovery.rs#L25) · struct

```rust
struct Recovery {
    ctx: Context,
    render: RenderState,
    active: Option<Combined>,
    path: PathBuf,
    cycle: usize,
    stage: Stage,
    deadline: Option<Instant>,
    frames: [u64; 3],
    snapshots: Vec<String>,
    failed: Option<String>,
    done: bool,
}
```

声明引用：`skin_editor_desktop::combined_verification::Combined`, `skin_editor_desktop::combined_verification::recovery::Stage`

实现／约束（显式或 derive，无类继承）：

- `impl Recovery` — [源码:38](../../../desktop/host/src/combined_verification/recovery.rs#L38)
- `impl App for Recovery` — [源码:172](../../../desktop/host/src/combined_verification/recovery.rs#L172)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn open(&mut self) -> Result<(), String>` | impl Recovery | [源码:39](../../../desktop/host/src/combined_verification/recovery.rs#L39) |  |
| `fn begin(&mut self, stage: Stage, duration: Duration, label: &str)` | impl Recovery | [源码:65](../../../desktop/host/src/combined_verification/recovery.rs#L65) | skin_editor_desktop::combined_verification::recovery::Stage |
| `fn snapshot(&mut self, suffix: &str) -> Result<(), String>` | impl Recovery | [源码:70](../../../desktop/host/src/combined_verification/recovery.rs#L70) |  |
| `fn close(&mut self) -> Result<(), String>` | impl Recovery | [源码:80](../../../desktop/host/src/combined_verification/recovery.rs#L80) |  |
| `fn tick(&mut self, ctx: &Context, frame: &mut Frame) -> Result<(), String>` | impl Recovery | [源码:106](../../../desktop/host/src/combined_verification/recovery.rs#L106) |  |
| `fn logic(&mut self, ctx: &Context, frame: &mut Frame)` | impl App for Recovery | [源码:173](../../../desktop/host/src/combined_verification/recovery.rs#L173) |  |
| `fn ui(&mut self, ui: &mut Ui, frame: &mut Frame)` | impl App for Recovery | [源码:183](../../../desktop/host/src/combined_verification/recovery.rs#L183) |  |
| `fn on_exit(&mut self)` | impl App for Recovery | [源码:202](../../../desktop/host/src/combined_verification/recovery.rs#L202) |  |

## skin_editor_desktop::combined_verification::recovery::SETTLE

[源码:8](../../../desktop/host/src/combined_verification/recovery.rs#L8) · constant

```rust
const SETTLE: Duration;
```

## skin_editor_desktop::combined_verification::recovery::STABLE

[源码:9](../../../desktop/host/src/combined_verification/recovery.rs#L9) · constant

```rust
const STABLE: Duration;
```

## skin_editor_desktop::combined_verification::recovery::Stage

[源码:12](../../../desktop/host/src/combined_verification/recovery.rs#L12) · enum

```rust
enum Stage {
    Playing,
    PauseSettle,
    Paused,
    CloseSettle,
    Closed,
    CycleGap,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::combined_verification::recovery::Recovery (声明)`, `skin_editor_desktop::combined_verification::recovery::Recovery::begin`, `skin_editor_desktop::combined_verification::recovery::Stage::clone`, `skin_editor_desktop::combined_verification::recovery::Stage::eq`

实现／约束（显式或 derive，无类继承）：

- `impl Stage` — [源码:20](../../../desktop/host/src/combined_verification/recovery.rs#L20)
- `impl Clone for Stage` — [源码:11](../../../desktop/host/src/combined_verification/recovery.rs#L11)
- `impl Debug for Stage` — [源码:11](../../../desktop/host/src/combined_verification/recovery.rs#L11)
- `impl PartialEq for Stage` — [源码:11](../../../desktop/host/src/combined_verification/recovery.rs#L11)
- `impl Copy for Stage` — [源码:11](../../../desktop/host/src/combined_verification/recovery.rs#L11)
- `impl Eq for Stage` — [源码:11](../../../desktop/host/src/combined_verification/recovery.rs#L11)
- `impl StructuralPartialEq for Stage` — [源码:11](../../../desktop/host/src/combined_verification/recovery.rs#L11)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn stable(self) -> bool` | impl Stage | [源码:21](../../../desktop/host/src/combined_verification/recovery.rs#L21) |  |
| `fn clone(&self) -> Stage` | impl Clone for Stage | [源码:11](../../../desktop/host/src/combined_verification/recovery.rs#L11) | skin_editor_desktop::combined_verification::recovery::Stage |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for Stage | [源码:11](../../../desktop/host/src/combined_verification/recovery.rs#L11) |  |
| `fn eq(&self, other: &Stage) -> bool` | impl PartialEq for Stage | [源码:11](../../../desktop/host/src/combined_verification/recovery.rs#L11) | skin_editor_desktop::combined_verification::recovery::Stage |

## skin_editor_desktop::combined_verification::recovery::run

[源码:216](../../../desktop/host/src/combined_verification/recovery.rs#L216) · fn

```rust
pub fn run(destination: &str) -> Result<(), String>
```

## skin_editor_desktop::combined_verification::reopen_fixture

[源码:361](../../../desktop/host/src/combined_verification.rs#L361) · fn

```rust
pub(crate) fn reopen_fixture(
    editor: &mut Editor,
    path: &Path,
) -> Result<u128, String>
```

声明引用：`skin_editor_desktop::Editor`

## skin_editor_desktop::combined_verification::run

[源码:1411](../../../desktop/host/src/combined_verification.rs#L1411) · fn

```rust
pub fn run(destination: &str) -> Result<(), String>
```

## skin_editor_desktop::combined_verification::run_mode

[源码:1429](../../../desktop/host/src/combined_verification.rs#L1429) · fn

```rust
fn run_mode(
    destination: &str,
    control_only: bool,
    memory_usage: bool,
    cold_control: bool,
    reusable_staging: bool,
) -> Result<(), String>
```

## skin_editor_desktop::combined_verification::scheduling_json

[源码:45](../../../desktop/host/src/combined_verification.rs#L45) · fn

```rust
fn scheduling_json(
    output: &OutputControl,
    alignment: [u64; 3],
    quiescent: bool,
) -> String
```

声明引用：`skin_editor_infrastructure::OutputControl`

## skin_editor_desktop::combined_verification::sound

[源码:64](../../../desktop/host/src/combined_verification.rs#L64) · fn

```rust
fn sound(index: usize) -> AudioComponentId
```

## skin_editor_desktop::combined_verification::staging_run

[源码:1426](../../../desktop/host/src/combined_verification.rs#L1426) · fn

```rust
pub fn staging_run(destination: &str, cold_control: bool) -> Result<(), String>
```

## skin_editor_desktop::combined_verification::visual_fixture

[源码:358](../../../desktop/host/src/combined_verification.rs#L358) · fn

```rust
pub(crate) fn visual_fixture(
    editor: &mut Editor,
    path: &Path,
) -> Result<u128, String>
```

声明引用：`skin_editor_desktop::Editor`

## skin_editor_desktop::edit_verification::Capture

[源码:35](../../../desktop/host/src/edit_verification.rs#L35) · struct

```rust
struct Capture(Option<Origin>);
```

声明引用：`skin_editor_audio::Origin`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::edit_verification::Capture::default`

实现／约束（显式或 derive，无类继承）：

- `impl Default for Capture` — [源码:34](../../../desktop/host/src/edit_verification.rs#L34)
- `impl SkinAudioPort for Capture` — [源码:36](../../../desktop/host/src/edit_verification.rs#L36)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn default() -> Capture` | impl Default for Capture | [源码:34](../../../desktop/host/src/edit_verification.rs#L34) | skin_editor_desktop::edit_verification::Capture |
| `fn synchronize( &mut self, _: &SkinProject, s: SkinAudioSource, _: &[DraftId], ) -> Result<(), SkinAudioError>` | impl SkinAudioPort for Capture | [源码:37](../../../desktop/host/src/edit_verification.rs#L37) |  |

## skin_editor_desktop::edit_verification::draft

[源码:15](../../../desktop/host/src/edit_verification.rs#L15) · fn

```rust
fn draft(
    session: &mut ProjectSession,
    next: &mut u128,
    action: DraftAction,
) -> Result<(), String>
```

## skin_editor_desktop::edit_verification::ds

[源码:27](../../../desktop/host/src/edit_verification.rs#L27) · fn

```rust
fn ds(session: &ProjectSession) -> DraftState
```

## skin_editor_desktop::edit_verification::error

[源码:12](../../../desktop/host/src/edit_verification.rs#L12) · fn

```rust
fn error(e: impl Debug) -> String
```

## skin_editor_desktop::edit_verification::health

[源码:172](../../../desktop/host/src/edit_verification.rs#L172) · fn

```rust
fn health(
    client: &AudioSupplyClient,
    output: &OutputControl,
    name: &str,
) -> Result<[u64; 4], String>
```

声明引用：`skin_editor_infrastructure::AudioSupplyClient`, `skin_editor_infrastructure::OutputControl`

## skin_editor_desktop::edit_verification::prepare

[源码:92](../../../desktop/host/src/edit_verification.rs#L92) · fn

```rust
fn prepare(
    client: &mut AudioSupplyClient,
    output: &OutputControl,
    session: &ProjectSession,
    started: Instant,
) -> Result<[f64; 6], String>
```

声明引用：`skin_editor_infrastructure::AudioSupplyClient`, `skin_editor_infrastructure::OutputControl`

## skin_editor_desktop::edit_verification::ready

[源码:58](../../../desktop/host/src/edit_verification.rs#L58) · fn

```rust
fn ready(
    client: &mut AudioSupplyClient,
    output: &OutputControl,
    expected: Origin,
) -> Result<(), String>
```

声明引用：`skin_editor_audio::Origin`, `skin_editor_infrastructure::AudioSupplyClient`, `skin_editor_infrastructure::OutputControl`

## skin_editor_desktop::edit_verification::restored

[源码:189](../../../desktop/host/src/edit_verification.rs#L189) · fn

```rust
fn restored(before: &ProjectSnapshot, after: &ProjectSnapshot) -> bool
```

## skin_editor_desktop::edit_verification::run

[源码:195](../../../desktop/host/src/edit_verification.rs#L195) · fn

```rust
pub fn run(destination: &str) -> Result<(), String>
```

## skin_editor_desktop::edit_verification::settled_pcm

[源码:163](../../../desktop/host/src/edit_verification.rs#L163) · fn

```rust
fn settled_pcm(
    budget: &PcmBudget,
    native_bytes: usize,
    frames: u64,
) -> Result<(), String>
```

声明引用：`skin_editor_audio::PcmBudget`

## skin_editor_desktop::edit_verification::source

[源码:53](../../../desktop/host/src/edit_verification.rs#L53) · fn

```rust
pub(crate) fn source(session: &ProjectSession) -> Origin
```

声明引用：`skin_editor_audio::Origin`

## skin_editor_desktop::edit_verification::trigger

[源码:140](../../../desktop/host/src/edit_verification.rs#L140) · fn

```rust
fn trigger(
    client: &mut AudioSupplyClient,
    output: &OutputControl,
    session: &ProjectSession,
    started: Instant,
) -> Result<f64, String>
```

声明引用：`skin_editor_infrastructure::AudioSupplyClient`, `skin_editor_infrastructure::OutputControl`

## skin_editor_desktop::editor_audio::EditorAudio

[源码:28](../../../desktop/host/src/editor_audio.rs#L28) · struct

```rust
pub struct EditorAudio {
    store: BlobStore,
    budget: PcmBudget,
    client: Option<AudioSupplyClient>,
    stream: Option<NativeStream>,
    native: Option<NativeExecutor>,
    candidate: bool,
    reference: Option<BackgroundTrack>,
    buffered: Option<BufferedBackground>,
    format: (u32, u16),
    background_frames: u64,
    last_triggered: Option<(u64, TriggerReceipt)>,
    last_trigger_request: Option<u64>,
    last_batch: Option<(u64, Result<Vec<u64>, String>)>,
    last_failed: Option<(u64, SupplyError)>,
    closed_stats: Option<NativeStats>,
    goal: bool,
    monitor_volume: u16,
    transport: Option<u64>,
    error: Option<String>,
    last_audition: Option<(SkinAudioSource, bool)>,
    recovery: Option<Recovery>,
    close_failed: bool,
    stop_pending: bool,
    stranded_endpoint: Option<StoppedOutput>,
    hold_return: bool,
    return_gate: Option<ReturnGate>,
}
```

声明引用：`skin_editor_audio::BackgroundTrack`, `skin_editor_audio::PcmBudget`, `skin_editor_audio::TriggerReceipt`, `skin_editor_desktop::editor_audio::Recovery`, `skin_editor_infrastructure::AudioSupplyClient`, `skin_editor_infrastructure::BlobStore`, `skin_editor_infrastructure::NativeStream`, `skin_editor_infrastructure::SupplyError`, `skin_editor_infrastructure::background_stream::BufferedBackground`, `skin_editor_infrastructure::separated_native::NativeExecutor`, `skin_editor_infrastructure::separated_native::NativeStats`, `skin_editor_infrastructure::separated_native::ReturnGate`, `skin_editor_infrastructure::separated_output::StoppedOutput`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::editor_audio::combined::Run (声明)`, `skin_editor_desktop::editor_audio::cycles::Round (声明)`, `skin_editor_desktop::editor_audio::faults::measure (声明)`, `skin_editor_desktop::editor_audio::faults::selected (声明)`, `skin_editor_desktop::editor_audio::faults::transport (声明)`, `skin_editor_desktop::editor_audio::gui_verification::Run (声明)`, `skin_editor_desktop::editor_audio::gui_verification::Shared (声明)`, `skin_editor_desktop::editor_audio::stability::Run (声明)`, `skin_editor_desktop::editor_audio::window_recovery::Returned (声明)`, `skin_editor_desktop::editor_audio::window_recovery::Run (声明)`, `skin_editor_desktop::editor_audio::window_recovery::activity::exercise (声明)`, `skin_editor_desktop::editor_audio::window_recovery::init_failure::run (声明)`, `skin_editor_desktop::editor_audio::window_recovery::report::window (声明)`, `skin_editor_desktop::editor_audio::window_recovery::restore::Attempt (声明)`, `skin_editor_desktop::editor_audio::window_recovery::restore::Attempt::attach_audio`, `skin_editor_desktop::editor_audio::window_recovery::restore::Cleanup (声明)`, `skin_editor_desktop::editor_audio::window_recovery::restore::Cleanup::step`, `skin_editor_desktop::editor_audio::window_recovery::sound::collect (声明)`, `skin_editor_desktop::editor_audio::window_recovery::sound::install (声明)`, `skin_editor_desktop::editor_audio::window_recovery::sound::reopen (声明)`, `skin_editor_desktop::editor_audio::window_recovery::sound::trigger (声明)`, `skin_editor_desktop::product_policy::Policy::audio`

实现／约束（显式或 derive，无类继承）：

- `impl EditorAudio` — [源码:88](../../../desktop/host/src/editor_audio.rs#L88)
- `impl AudioEditorPort for EditorAudio` — [源码:396](../../../desktop/host/src/editor_audio.rs#L396)
- `impl Drop for EditorAudio` — [源码:370](../../../desktop/host/src/editor_audio.rs#L370)
- `impl SkinAudioPort for EditorAudio` — [源码:375](../../../desktop/host/src/editor_audio.rs#L375)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn receive(&mut self, id: u64, result: &Result<SupplyOutcome, SupplyError>)` | impl EditorAudio | [源码:89](../../../desktop/host/src/editor_audio.rs#L89) | skin_editor_infrastructure::SupplyError, skin_editor_infrastructure::SupplyOutcome |
| `pub fn new(store: BlobStore) -> Self` | impl EditorAudio | [源码:123](../../../desktop/host/src/editor_audio.rs#L123) | skin_editor_infrastructure::BlobStore |
| `pub fn candidate(store: BlobStore) -> Self` | impl EditorAudio | [源码:153](../../../desktop/host/src/editor_audio.rs#L153) | skin_editor_infrastructure::BlobStore |
| `fn take_reference( &mut self, rate: u32, channels: u16, ) -> Result<BackgroundTrack, String>` | impl EditorAudio | [源码:158](../../../desktop/host/src/editor_audio.rs#L158) | skin_editor_audio::BackgroundTrack |
| `fn enable_candidate(&mut self) -> Result<(), String>` | impl EditorAudio | [源码:183](../../../desktop/host/src/editor_audio.rs#L183) |  |
| `fn restore_background(&mut self, format: (u32, u16)) -> Result<(), String>` | impl EditorAudio | [源码:230](../../../desktop/host/src/editor_audio.rs#L230) |  |
| `fn handle_native_fault(&mut self)` | impl EditorAudio | [源码:259](../../../desktop/host/src/editor_audio.rs#L259) |  |
| `fn close(&mut self) -> Result<(), String>` | impl EditorAudio | [源码:285](../../../desktop/host/src/editor_audio.rs#L285) |  |
| `fn transport(&mut self) -> Result<(), AudioEditorError>` | impl EditorAudio | [源码:349](../../../desktop/host/src/editor_audio.rs#L349) |  |
| `fn audition_asset( &mut self, request: AssetAudition, ) -> Result<(), AudioEditorError>` | impl AudioEditorPort for EditorAudio | [源码:397](../../../desktop/host/src/editor_audio.rs#L397) |  |
| `fn request(&mut self, action: AudioEditorAction) -> Result<(), AudioEditorError>` | impl AudioEditorPort for EditorAudio | [源码:414](../../../desktop/host/src/editor_audio.rs#L414) |  |
| `fn poll(&mut self) -> AudioEditorState` | impl AudioEditorPort for EditorAudio | [源码:521](../../../desktop/host/src/editor_audio.rs#L521) |  |
| `fn drop(&mut self)` | impl Drop for EditorAudio | [源码:371](../../../desktop/host/src/editor_audio.rs#L371) |  |
| `fn retained_drafts(&self) -> Vec<DraftId>` | impl SkinAudioPort for EditorAudio | [源码:376](../../../desktop/host/src/editor_audio.rs#L376) |  |
| `fn synchronize( &mut self, p: &SkinProject, s: SkinAudioSource, c: &[DraftId], ) -> Result<(), SkinAudioError>` | impl SkinAudioPort for EditorAudio | [源码:381](../../../desktop/host/src/editor_audio.rs#L381) |  |

## skin_editor_desktop::editor_audio::Recovery

[源码:57](../../../desktop/host/src/editor_audio.rs#L57) · struct

```rust
struct Recovery {
    format: (u32, u16),
    copied_frame: Option<u64>,
    frames: u64,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::editor_audio::EditorAudio (声明)`, `skin_editor_desktop::editor_audio::Recovery::clone`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for Recovery` — [源码:56](../../../desktop/host/src/editor_audio.rs#L56)
- `impl Copy for Recovery` — [源码:56](../../../desktop/host/src/editor_audio.rs#L56)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> Recovery` | impl Clone for Recovery | [源码:56](../../../desktop/host/src/editor_audio.rs#L56) | skin_editor_desktop::editor_audio::Recovery |

## skin_editor_desktop::editor_audio::combined::Run

[源码:43](../../../desktop/host/src/editor_audio/combined.rs#L43) · struct

```rust
struct Run {
    product: bool,
    editor: Editor,
    audio: Rc<RefCell<EditorAudio>>,
    source: StreamObservation,
    observer: Option<Observations>,
    probes: Probes,
    path: PathBuf,
    origin: Instant,
    playing: Option<Instant>,
    clock: StageClock,
    plan: Plan,
    pending_visual: Option<VisualPending>,
    edits_ms: Vec<f64>,
    switches_ms: Vec<f64>,
    opacities: [f32; 10],
    frame_ms: Vec<f64>,
    last_frame: Option<u64>,
    bg_start: u64,
    bg_end: u64,
    bg_previous: u64,
    frames_start: u64,
    playing_frames: u64,
    playing_us: u64,
    anchor: u64,
    quiet: u64,
    baseline_pcm: usize,
    snapshot: Option<ProjectSnapshot>,
    nodes: usize,
    peaks: [usize; 5],
    final_usage: [usize; 5],
    max_voices: usize,
    max_supply_gap: u64,
    stats: NativeStats,
    controls: [bool; 6],
    continuity: Vec<[u64; 3]>,
    adapter: String,
    format: (u32, u16),
    period: u32,
    geometry: Option<[f32; 3]>,
    view_valid: bool,
    physical_inputs: u64,
    completed: bool,
    failure: Option<String>,
}
```

声明引用：`skin_editor_desktop::Editor`, `skin_editor_desktop::editor_audio::EditorAudio`, `skin_editor_desktop::editor_audio::combined::VisualPending`, `skin_editor_desktop::editor_audio::combined::plan::Plan`, `skin_editor_desktop::editor_audio::combined::plan::StageClock`, `skin_editor_desktop::editor_audio::combined::probes::Probes`, `skin_editor_desktop::editor_audio::gui_verification::Observations`, `skin_editor_infrastructure::background_stream::StreamObservation`, `skin_editor_infrastructure::separated_native::NativeStats`

实现／约束（显式或 derive，无类继承）：

- `impl Run` — [源码:2](../../../desktop/host/src/editor_audio\combined/report.rs#L2)
- `impl Run` — [源码:88](../../../desktop/host/src/editor_audio/combined.rs#L88)
- `impl App for Run` — [源码:637](../../../desktop/host/src/editor_audio/combined.rs#L637)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub(super) fn report(&self) -> String` | impl Run | [源码:3](../../../desktop/host/src/editor_audio\combined/report.rs#L3) |  |
| `fn new( c: &CreationContext<'_>, path: PathBuf, product: bool, ) -> Result<Self, String>` | impl Run | [源码:89](../../../desktop/host/src/editor_audio/combined.rs#L89) |  |
| `fn advance(&mut self, stage: u8)` | impl Run | [源码:178](../../../desktop/host/src/editor_audio/combined.rs#L178) |  |
| `fn usage(&self) -> Result<[usize; 5], String>` | impl Run | [源码:181](../../../desktop/host/src/editor_audio/combined.rs#L181) |  |
| `fn action(&mut self, action: AudioEditorAction) -> Result<(), String>` | impl Run | [源码:190](../../../desktop/host/src/editor_audio/combined.rs#L190) |  |
| `fn draft(&mut self, action: DraftAction) -> Result<(), String>` | impl Run | [源码:200](../../../desktop/host/src/editor_audio/combined.rs#L200) |  |
| `fn draft_state(&self) -> Result<DraftState, String>` | impl Run | [源码:206](../../../desktop/host/src/editor_audio/combined.rs#L206) |  |
| `fn ready(&mut self) -> Result<bool, String>` | impl Run | [源码:215](../../../desktop/host/src/editor_audio/combined.rs#L215) |  |
| `fn background(&self) -> u64` | impl Run | [源码:240](../../../desktop/host/src/editor_audio/combined.rs#L240) |  |
| `fn trigger( &mut self, lane: Channel, component: AudioComponentId, measured: bool, ) -> Result<(), String>` | impl Run | [源码:248](../../../desktop/host/src/editor_audio/combined.rs#L248) | skin_editor_audio::Channel |
| `fn collect(&mut self) -> Result<(), String>` | impl Run | [源码:278](../../../desktop/host/src/editor_audio/combined.rs#L278) |  |
| `fn visual(&mut self, edit: bool) -> Result<(), String>` | impl Run | [源码:300](../../../desktop/host/src/editor_audio/combined.rs#L300) |  |
| `fn observe_visual(&mut self) -> Result<(), String>` | impl Run | [源码:312](../../../desktop/host/src/editor_audio/combined.rs#L312) |  |
| `fn event(&mut self, e: Event) -> Result<(), String>` | impl Run | [源码:331](../../../desktop/host/src/editor_audio/combined.rs#L331) | skin_editor_desktop::editor_audio::combined::plan::Event |
| `fn step(&mut self) -> Result<(), String>` | impl Run | [源码:384](../../../desktop/host/src/editor_audio/combined.rs#L384) |  |
| `fn raw_input_hook(&mut self, _: &Context, input: &mut RawInput)` | impl App for Run | [源码:638](../../../desktop/host/src/editor_audio/combined.rs#L638) |  |
| `fn logic(&mut self, ctx: &Context, frame: &mut Frame)` | impl App for Run | [源码:651](../../../desktop/host/src/editor_audio/combined.rs#L651) |  |
| `fn ui(&mut self, root: &mut Ui, frame: &mut Frame)` | impl App for Run | [源码:666](../../../desktop/host/src/editor_audio/combined.rs#L666) |  |
| `fn on_exit(&mut self)` | impl App for Run | [源码:719](../../../desktop/host/src/editor_audio/combined.rs#L719) |  |

## skin_editor_desktop::editor_audio::combined::VisualPending

[源码:37](../../../desktop/host/src/editor_audio/combined.rs#L37) · struct

```rust
struct VisualPending {
    at: Instant,
    draft: Option<DraftState>,
    target: PreviewTarget,
    edit: bool,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::editor_audio::combined::Run (声明)`

## skin_editor_desktop::editor_audio::combined::document

[源码:28](../../../desktop/host/src/editor_audio/combined.rs#L28) · fn

```rust
fn document(n: usize) -> DocumentId
```

## skin_editor_desktop::editor_audio::combined::error

[源码:25](../../../desktop/host/src/editor_audio/combined.rs#L25) · fn

```rust
fn error(e: impl Debug) -> String
```

## skin_editor_desktop::editor_audio::combined::layer

[源码:31](../../../desktop/host/src/editor_audio/combined.rs#L31) · fn

```rust
fn layer(n: usize) -> LayerId
```

## skin_editor_desktop::editor_audio::combined::plan::COUNTS

[源码:3](../../../desktop/host/src/editor_audio\combined/plan.rs#L3) · constant

```rust
pub(super) const COUNTS: [usize; 4];
```

## skin_editor_desktop::editor_audio::combined::plan::Event

[源码:29](../../../desktop/host/src/editor_audio\combined/plan.rs#L29) · enum

```rust
pub(super) enum Event {
    Scene,
    Audition,
    Edit,
    Switch,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::editor_audio::combined::Run::event`, `skin_editor_desktop::editor_audio::combined::plan::Event::clone`, `skin_editor_desktop::editor_audio::combined::plan::Event::eq`, `skin_editor_desktop::editor_audio::combined::plan::Plan::due`, `skin_editor_desktop::editor_audio::combined::plan::Plan::next`, `skin_editor_desktop::editor_audio::combined::plan::Plan::take`

实现／约束（显式或 derive，无类继承）：

- `impl Event` — [源码:35](../../../desktop/host/src/editor_audio\combined/plan.rs#L35)
- `impl Clone for Event` — [源码:28](../../../desktop/host/src/editor_audio\combined/plan.rs#L28)
- `impl Debug for Event` — [源码:28](../../../desktop/host/src/editor_audio\combined/plan.rs#L28)
- `impl PartialEq for Event` — [源码:28](../../../desktop/host/src/editor_audio\combined/plan.rs#L28)
- `impl Copy for Event` — [源码:28](../../../desktop/host/src/editor_audio\combined/plan.rs#L28)
- `impl Eq for Event` — [源码:28](../../../desktop/host/src/editor_audio\combined/plan.rs#L28)
- `impl StructuralPartialEq for Event` — [源码:28](../../../desktop/host/src/editor_audio\combined/plan.rs#L28)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub(super) fn index(self) -> usize` | impl Event | [源码:36](../../../desktop/host/src/editor_audio\combined/plan.rs#L36) |  |
| `fn clone(&self) -> Event` | impl Clone for Event | [源码:28](../../../desktop/host/src/editor_audio\combined/plan.rs#L28) | skin_editor_desktop::editor_audio::combined::plan::Event |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for Event | [源码:28](../../../desktop/host/src/editor_audio\combined/plan.rs#L28) |  |
| `fn eq(&self, other: &Event) -> bool` | impl PartialEq for Event | [源码:28](../../../desktop/host/src/editor_audio\combined/plan.rs#L28) | skin_editor_desktop::editor_audio::combined::plan::Event |

## skin_editor_desktop::editor_audio::combined::plan::Plan

[源码:46](../../../desktop/host/src/editor_audio\combined/plan.rs#L46) · struct

```rust
pub(super) struct Plan {
    pub counts: [usize; 4],
    pub max_late_us: u64,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::editor_audio::combined::Run (声明)`, `skin_editor_desktop::editor_audio::combined::plan::Plan::default`

实现／约束（显式或 derive，无类继承）：

- `impl Plan` — [源码:50](../../../desktop/host/src/editor_audio\combined/plan.rs#L50)
- `impl Default for Plan` — [源码:45](../../../desktop/host/src/editor_audio\combined/plan.rs#L45)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn due(&self, e: Event) -> Option<u64>` | impl Plan | [源码:51](../../../desktop/host/src/editor_audio\combined/plan.rs#L51) | skin_editor_desktop::editor_audio::combined::plan::Event |
| `pub fn next(&self) -> Option<(Event, u64)>` | impl Plan | [源码:62](../../../desktop/host/src/editor_audio\combined/plan.rs#L62) | skin_editor_desktop::editor_audio::combined::plan::Event |
| `pub fn take(&mut self, now_us: u64) -> Result<Option<Event>, &'static str>` | impl Plan | [源码:68](../../../desktop/host/src/editor_audio\combined/plan.rs#L68) | skin_editor_desktop::editor_audio::combined::plan::Event |
| `fn default() -> Plan` | impl Default for Plan | [源码:45](../../../desktop/host/src/editor_audio\combined/plan.rs#L45) | skin_editor_desktop::editor_audio::combined::plan::Plan |

## skin_editor_desktop::editor_audio::combined::plan::SECONDS

[源码:2](../../../desktop/host/src/editor_audio\combined/plan.rs#L2) · constant

```rust
pub(super) const SECONDS: u64 = 300;
```

## skin_editor_desktop::editor_audio::combined::plan::StageClock

[源码:5](../../../desktop/host/src/editor_audio\combined/plan.rs#L5) · struct

```rust
pub(super) struct StageClock {
    pub stage: u8,
    entered: Duration,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::editor_audio::combined::Run (声明)`, `skin_editor_desktop::editor_audio::combined::plan::StageClock::default`

实现／约束（显式或 derive，无类继承）：

- `impl StageClock` — [源码:9](../../../desktop/host/src/editor_audio\combined/plan.rs#L9)
- `impl Default for StageClock` — [源码:4](../../../desktop/host/src/editor_audio\combined/plan.rs#L4)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn enter(&mut self, stage: u8, now: Duration)` | impl StageClock | [源码:10](../../../desktop/host/src/editor_audio\combined/plan.rs#L10) |  |
| `pub fn age(&self, now: Duration) -> Result<Duration, &'static str>` | impl StageClock | [源码:14](../../../desktop/host/src/editor_audio\combined/plan.rs#L14) |  |
| `pub fn check(&self, now: Duration) -> Result<(), &'static str>` | impl StageClock | [源码:17](../../../desktop/host/src/editor_audio\combined/plan.rs#L17) |  |
| `fn default() -> StageClock` | impl Default for StageClock | [源码:4](../../../desktop/host/src/editor_audio\combined/plan.rs#L4) | skin_editor_desktop::editor_audio::combined::plan::StageClock |

## skin_editor_desktop::editor_audio::combined::probes::Complete

[源码:14](../../../desktop/host/src/editor_audio\combined/probes.rs#L14) · struct

```rust
pub(in editor_audio) struct Complete {
    pub id: u64,
    pub lane: Channel,
    pub voice: u64,
    pub upper_ms: f64,
    pub joined_ms: f64,
    pub first: FirstCopy,
    pub measured: bool,
}
```

声明引用：`skin_editor_audio::Channel`, `skin_editor_infrastructure::separated_native::FirstCopy`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::editor_audio::combined::probes::Probes (声明)`

## skin_editor_desktop::editor_audio::combined::probes::LIMIT

[源码:5](../../../desktop/host/src/editor_audio\combined/probes.rs#L5) · constant

```rust
const LIMIT: usize = 2496;
```

## skin_editor_desktop::editor_audio::combined::probes::Pending

[源码:6](../../../desktop/host/src/editor_audio\combined/probes.rs#L6) · struct

```rust
pub(super) struct Pending {
    id: u64,
    lane: Channel,
    at: Instant,
    origin: Origin,
    voice: Option<u64>,
    measured: bool,
}
```

声明引用：`skin_editor_audio::Channel`, `skin_editor_audio::Origin`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::editor_audio::combined::probes::Probes (声明)`

## skin_editor_desktop::editor_audio::combined::probes::Probes

[源码:23](../../../desktop/host/src/editor_audio\combined/probes.rs#L23) · struct

```rust
pub(in editor_audio) struct Probes {
    pending: Vec<Pending>,
    pub complete: Vec<Complete>,
}
```

声明引用：`skin_editor_desktop::editor_audio::combined::probes::Complete`, `skin_editor_desktop::editor_audio::combined::probes::Pending`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::editor_audio::combined::Run (声明)`, `skin_editor_desktop::editor_audio::cycles::Round (声明)`, `skin_editor_desktop::editor_audio::window_recovery::Run (声明)`, `skin_editor_desktop::editor_audio::window_recovery::sound::collect (声明)`, `skin_editor_desktop::editor_audio::window_recovery::sound::rows (声明)`, `skin_editor_desktop::editor_audio::window_recovery::sound::trigger (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Probes` — [源码:35](../../../desktop/host/src/editor_audio\combined/probes.rs#L35)
- `impl Default for Probes` — [源码:27](../../../desktop/host/src/editor_audio\combined/probes.rs#L27)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn begin( &mut self, id: u64, lane: Channel, at: Instant, origin: Origin, measured: bool, ) -> Result<(), String>` | impl Probes | [源码:36](../../../desktop/host/src/editor_audio\combined/probes.rs#L36) | skin_editor_audio::Channel, skin_editor_audio::Origin |
| `pub fn receipt(&mut self, id: u64, r: TriggerReceipt) -> Result<(), String>` | impl Probes | [源码:61](../../../desktop/host/src/editor_audio\combined/probes.rs#L61) | skin_editor_audio::TriggerReceipt |
| `pub fn observe( &mut self, rows: &[(FirstCopy, Instant)], now: Instant, ) -> Result<(), String>` | impl Probes | [源码:73](../../../desktop/host/src/editor_audio\combined/probes.rs#L73) | skin_editor_infrastructure::separated_native::FirstCopy |
| `pub fn empty(&self) -> bool` | impl Probes | [源码:107](../../../desktop/host/src/editor_audio\combined/probes.rs#L107) |  |
| `fn default() -> Self` | impl Default for Probes | [源码:28](../../../desktop/host/src/editor_audio\combined/probes.rs#L28) |  |

## skin_editor_desktop::editor_audio::combined::run

[源码:724](../../../desktop/host/src/editor_audio/combined.rs#L724) · fn

```rust
pub fn run(destination: &str) -> Result<(), String>
```

## skin_editor_desktop::editor_audio::combined::run_policy

[源码:727](../../../desktop/host/src/editor_audio/combined.rs#L727) · fn

```rust
pub fn run_policy(destination: &str, product: bool) -> Result<(), String>
```

## skin_editor_desktop::editor_audio::combined::sound

[源码:34](../../../desktop/host/src/editor_audio/combined.rs#L34) · fn

```rust
fn sound(n: usize) -> AudioComponentId
```

## skin_editor_desktop::editor_audio::cycles::Counters

[源码:32](../../../desktop/host/src/editor_audio/cycles.rs#L32) · struct

```rust
struct Counters {
    pcm: PcmBudget,
    raw: ByteObservation,
    rgba: ByteObservation,
    background: StreamObservation,
}
```

声明引用：`skin_editor_audio::PcmBudget`, `skin_editor_infrastructure::ByteObservation`, `skin_editor_infrastructure::background_stream::StreamObservation`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::editor_audio::cycles::Round (声明)`, `skin_editor_desktop::editor_audio::cycles::Run (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Counters` — [源码:38](../../../desktop/host/src/editor_audio/cycles.rs#L38)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn bytes(&self) -> [usize; 3]` | impl Counters | [源码:39](../../../desktop/host/src/editor_audio/cycles.rs#L39) |  |

## skin_editor_desktop::editor_audio::cycles::Round

[源码:47](../../../desktop/host/src/editor_audio/cycles.rs#L47) · struct

```rust
struct Round {
    editor: Box<Editor>,
    audio: Rc<RefCell<EditorAudio>>,
    observer: Option<Observations>,
    probes: Probes,
    counters: Counters,
    opened: Instant,
    ready_ms: f64,
    session: u128,
    target: usize,
    counts: [usize; 3],
    playing: Option<Instant>,
    bg_start: u64,
    bg_previous: u64,
    bg_end: u64,
    frame_start: u64,
    playing_frames: u64,
    playing_us: u64,
    max_late_us: u64,
    pending_image: Option<DraftState>,
    images_published: usize,
    frame_ms: Vec<f64>,
    last_frame: Option<u64>,
    peaks: [usize; 5],
    max_voices: usize,
    stats: NativeStats,
    workers: [bool; 3],
    pending_at_close: bool,
    voices_at_close: [usize; 2],
    closed_started: Option<Instant>,
    close_ms: f64,
    geometry: Option<[f32; 3]>,
    view_valid: bool,
}
```

声明引用：`skin_editor_desktop::Editor`, `skin_editor_desktop::editor_audio::EditorAudio`, `skin_editor_desktop::editor_audio::combined::probes::Probes`, `skin_editor_desktop::editor_audio::cycles::Counters`, `skin_editor_desktop::editor_audio::gui_verification::Observations`, `skin_editor_infrastructure::separated_native::NativeStats`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::editor_audio::cycles::Run (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Round` — [源码:2](../../../desktop/host/src/editor_audio\cycles/report.rs#L2)
- `impl Round` — [源码:81](../../../desktop/host/src/editor_audio/cycles.rs#L81)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub(super) fn record( &self, index: usize, usage: [usize; 5], post_drop: bool, ) -> String` | impl Round | [源码:3](../../../desktop/host/src/editor_audio\cycles/report.rs#L3) |  |
| `pub(super) fn summary(&self, index: usize) -> String` | impl Round | [源码:84](../../../desktop/host/src/editor_audio\cycles/report.rs#L84) |  |
| `fn new( ctx: &Context, state: &RenderState, path: &Path, index: usize, format: (u32, u16), product: bool, ) -> Result<Self, String>` | impl Round | [源码:82](../../../desktop/host/src/editor_audio/cycles.rs#L82) |  |
| `fn usage(&self) -> [usize; 5]` | impl Round | [源码:202](../../../desktop/host/src/editor_audio/cycles.rs#L202) |  |
| `fn observe(&mut self) -> Result<(), String>` | impl Round | [源码:213](../../../desktop/host/src/editor_audio/cycles.rs#L213) |  |
| `fn action(&mut self, a: AudioEditorAction) -> Result<(), String>` | impl Round | [源码:233](../../../desktop/host/src/editor_audio/cycles.rs#L233) |  |
| `fn ready(&mut self) -> Result<bool, String>` | impl Round | [源码:243](../../../desktop/host/src/editor_audio/cycles.rs#L243) |  |
| `fn draft_state(&self) -> Result<DraftState, String>` | impl Round | [源码:268](../../../desktop/host/src/editor_audio/cycles.rs#L268) |  |
| `fn draft(&mut self, a: DraftAction) -> Result<(), String>` | impl Round | [源码:277](../../../desktop/host/src/editor_audio/cycles.rs#L277) |  |
| `fn image(&mut self, opacity: f32) -> Result<(), String>` | impl Round | [源码:283](../../../desktop/host/src/editor_audio/cycles.rs#L283) |  |
| `fn trigger( &mut self, lane: Channel, component: AudioComponentId, measured: bool, ) -> Result<(), String>` | impl Round | [源码:299](../../../desktop/host/src/editor_audio/cycles.rs#L299) | skin_editor_audio::Channel |
| `fn collect(&mut self) -> Result<(), String>` | impl Round | [源码:329](../../../desktop/host/src/editor_audio/cycles.rs#L329) |  |
| `fn background(&self) -> u64` | impl Round | [源码:350](../../../desktop/host/src/editor_audio/cycles.rs#L350) |  |
| `fn begin_close(&mut self) -> Result<(), String>` | impl Round | [源码:358](../../../desktop/host/src/editor_audio/cycles.rs#L358) |  |

## skin_editor_desktop::editor_audio::cycles::Run

[源码:404](../../../desktop/host/src/editor_audio/cycles.rs#L404) · struct

```rust
struct Run {
    product: bool,
    state: RenderState,
    path: PathBuf,
    fixture: PathBuf,
    mode: Mode,
    format: (u32, u16),
    period: u32,
    adapter: String,
    active: Option<Round>,
    retired: Option<Counters>,
    phase: u8,
    entered: Instant,
    started: Instant,
    index: usize,
    rows: Vec<String>,
    completed: bool,
    failure: Option<String>,
    physical_inputs: u64,
    phase_step: u64,
}
```

声明引用：`skin_editor_desktop::editor_audio::cycles::Counters`, `skin_editor_desktop::editor_audio::cycles::Round`, `skin_editor_desktop::editor_audio::cycles::plan::Mode`

实现／约束（显式或 derive，无类继承）：

- `impl Run` — [源码:97](../../../desktop/host/src/editor_audio\cycles/report.rs#L97)
- `impl Run` — [源码:425](../../../desktop/host/src/editor_audio/cycles.rs#L425)
- `impl App for Run` — [源码:697](../../../desktop/host/src/editor_audio/cycles.rs#L697)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub(super) fn report(&self) -> String` | impl Run | [源码:98](../../../desktop/host/src/editor_audio\cycles/report.rs#L98) |  |
| `fn new( c: &CreationContext<'_>, path: PathBuf, smoke: bool, product: bool, ) -> Result<Self, String>` | impl Run | [源码:426](../../../desktop/host/src/editor_audio/cycles.rs#L426) |  |
| `fn publish_phase(&mut self, name: &str)` | impl Run | [源码:461](../../../desktop/host/src/editor_audio/cycles.rs#L461) |  |
| `fn advance(&mut self, p: u8)` | impl Run | [源码:469](../../../desktop/host/src/editor_audio/cycles.rs#L469) |  |
| `fn step(&mut self, ctx: &Context) -> Result<(), String>` | impl Run | [源码:486](../../../desktop/host/src/editor_audio/cycles.rs#L486) |  |
| `fn delay(&self) -> Duration` | impl Run | [源码:684](../../../desktop/host/src/editor_audio/cycles.rs#L684) |  |
| `fn raw_input_hook(&mut self, _: &Context, input: &mut RawInput)` | impl App for Run | [源码:698](../../../desktop/host/src/editor_audio/cycles.rs#L698) |  |
| `fn logic(&mut self, ctx: &Context, frame: &mut Frame)` | impl App for Run | [源码:709](../../../desktop/host/src/editor_audio/cycles.rs#L709) |  |
| `fn ui(&mut self, root: &mut Ui, frame: &mut Frame)` | impl App for Run | [源码:722](../../../desktop/host/src/editor_audio/cycles.rs#L722) |  |
| `fn on_exit(&mut self)` | impl App for Run | [源码:785](../../../desktop/host/src/editor_audio/cycles.rs#L785) |  |

## skin_editor_desktop::editor_audio::cycles::error

[源码:26](../../../desktop/host/src/editor_audio/cycles.rs#L26) · fn

```rust
fn error(e: impl Debug) -> String
```

## skin_editor_desktop::editor_audio::cycles::plan::Mode

[源码:3](../../../desktop/host/src/editor_audio\cycles/plan.rs#L3) · struct

```rust
pub(super) struct Mode {
    pub smoke: bool,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::editor_audio::cycles::Run (声明)`, `skin_editor_desktop::editor_audio::cycles::plan::Mode::clone`, `skin_editor_desktop::editor_audio::cycles::plan::next (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Mode` — [源码:6](../../../desktop/host/src/editor_audio\cycles/plan.rs#L6)
- `impl Clone for Mode` — [源码:2](../../../desktop/host/src/editor_audio\cycles/plan.rs#L2)
- `impl Copy for Mode` — [源码:2](../../../desktop/host/src/editor_audio\cycles/plan.rs#L2)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn rounds(self) -> usize` | impl Mode | [源码:7](../../../desktop/host/src/editor_audio\cycles/plan.rs#L7) |  |
| `pub fn counts(self) -> [usize; 3]` | impl Mode | [源码:10](../../../desktop/host/src/editor_audio\cycles/plan.rs#L10) |  |
| `pub fn play(self) -> Duration` | impl Mode | [源码:13](../../../desktop/host/src/editor_audio\cycles/plan.rs#L13) |  |
| `pub fn settle(self) -> Duration` | impl Mode | [源码:16](../../../desktop/host/src/editor_audio\cycles/plan.rs#L16) |  |
| `pub fn baseline(self) -> Duration` | impl Mode | [源码:19](../../../desktop/host/src/editor_audio\cycles/plan.rs#L19) |  |
| `pub fn final_settle(self) -> Duration` | impl Mode | [源码:22](../../../desktop/host/src/editor_audio\cycles/plan.rs#L22) |  |
| `pub fn final_observe(self) -> Duration` | impl Mode | [源码:25](../../../desktop/host/src/editor_audio\cycles/plan.rs#L25) |  |
| `fn clone(&self) -> Mode` | impl Clone for Mode | [源码:2](../../../desktop/host/src/editor_audio\cycles/plan.rs#L2) | skin_editor_desktop::editor_audio::cycles::plan::Mode |

## skin_editor_desktop::editor_audio::cycles::plan::next

[源码:29](../../../desktop/host/src/editor_audio\cycles/plan.rs#L29) · fn

```rust
pub(super) fn next(mode: Mode, counts: [usize; 3]) -> Option<(usize, u64)>
```

声明引用：`skin_editor_desktop::editor_audio::cycles::plan::Mode`

## skin_editor_desktop::editor_audio::cycles::plan::released

[源码:44](../../../desktop/host/src/editor_audio\cycles/plan.rs#L44) · fn

```rust
pub(super) fn released(
    usage: [usize; 5],
    workers: [bool; 3],
    endpoint_returns: u64,
    background_bytes: usize,
) -> bool
```

## skin_editor_desktop::editor_audio::cycles::run

[源码:792](../../../desktop/host/src/editor_audio/cycles.rs#L792) · fn

```rust
pub fn run(destination: &str, smoke: bool) -> Result<(), String>
```

## skin_editor_desktop::editor_audio::cycles::run_policy

[源码:795](../../../desktop/host/src/editor_audio/cycles.rs#L795) · fn

```rust
pub fn run_policy(
    destination: &str,
    smoke: bool,
    product: bool,
) -> Result<(), String>
```

## skin_editor_desktop::editor_audio::cycles::sound

[源码:29](../../../desktop/host/src/editor_audio/cycles.rs#L29) · fn

```rust
fn sound(i: usize) -> AudioComponentId
```

## skin_editor_desktop::editor_audio::faults::error

[源码:14](../../../desktop/host/src/editor_audio/faults.rs#L14) · fn

```rust
fn error(e: impl Debug) -> String
```

## skin_editor_desktop::editor_audio::faults::measure

[源码:57](../../../desktop/host/src/editor_audio/faults.rs#L57) · fn

```rust
fn measure(
    a: &mut EditorAudio,
    s: &ProjectSession,
    observer: &Observations,
    count: u64,
) -> Result<String, String>
```

声明引用：`skin_editor_desktop::editor_audio::EditorAudio`, `skin_editor_desktop::editor_audio::gui_verification::Observations`

## skin_editor_desktop::editor_audio::faults::run

[源码:131](../../../desktop/host/src/editor_audio/faults.rs#L131) · fn

```rust
pub fn run(destination: &str) -> Result<(), String>
```

## skin_editor_desktop::editor_audio::faults::selected

[源码:30](../../../desktop/host/src/editor_audio/faults.rs#L30) · fn

```rust
fn selected(a: &mut EditorAudio, s: &ProjectSession) -> Result<(), String>
```

声明引用：`skin_editor_desktop::editor_audio::EditorAudio`

## skin_editor_desktop::editor_audio::faults::transport

[源码:46](../../../desktop/host/src/editor_audio/faults.rs#L46) · fn

```rust
fn transport(a: &mut EditorAudio, running: bool) -> Result<(), String>
```

声明引用：`skin_editor_desktop::editor_audio::EditorAudio`

## skin_editor_desktop::editor_audio::faults::until

[源码:17](../../../desktop/host/src/editor_audio/faults.rs#L17) · fn

```rust
fn until(poll: impl FnMut() -> Result<bool, String>) -> Result<(), String>
```

## skin_editor_desktop::editor_audio::gui_verification::GuiWork

[源码:142](../../../desktop/host/src/editor_audio/gui_verification.rs#L142) · struct

```rust
struct GuiWork {
    logic: [[u64; 3]; 18],
    frame_wall: [[u64; 3]; 18],
    frame: Option<u64>,
    submitted_stage: Option<u8>,
    invalid: u64,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::editor_audio::gui_verification::GuiWork::default`, `skin_editor_desktop::editor_audio::gui_verification::Run (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl GuiWork` — [源码:149](../../../desktop/host/src/editor_audio/gui_verification.rs#L149)
- `impl Default for GuiWork` — [源码:141](../../../desktop/host/src/editor_audio/gui_verification.rs#L141)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn add(row: &mut [u64; 3], ns: u64)` | impl GuiWork | [源码:150](../../../desktop/host/src/editor_audio/gui_verification.rs#L150) |  |
| `fn previous_frame(&mut self, frame: u64, seconds: Option<f32>)` | impl GuiWork | [源码:155](../../../desktop/host/src/editor_audio/gui_verification.rs#L155) |  |
| `fn default() -> GuiWork` | impl Default for GuiWork | [源码:141](../../../desktop/host/src/editor_audio/gui_verification.rs#L141) | skin_editor_desktop::editor_audio::gui_verification::GuiWork |

## skin_editor_desktop::editor_audio::gui_verification::Observations

[源码:56](../../../desktop/host/src/editor_audio/gui_verification.rs#L56) · struct

```rust
pub(super) struct Observations {
    stop: Arc<AtomicBool>,
    pub(super) rows: Arc<Mutex<Vec<(FirstCopy, Instant)>>>,
    pub(super) overflow: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
    join_ok: bool,
}
```

声明引用：`skin_editor_infrastructure::separated_native::FirstCopy`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::editor_audio::combined::Run (声明)`, `skin_editor_desktop::editor_audio::cycles::Round (声明)`, `skin_editor_desktop::editor_audio::faults::measure (声明)`, `skin_editor_desktop::editor_audio::gui_verification::Run (声明)`, `skin_editor_desktop::editor_audio::window_recovery::Run (声明)`, `skin_editor_desktop::editor_audio::window_recovery::activity::exercise (声明)`, `skin_editor_desktop::editor_audio::window_recovery::restore::Attempt (声明)`, `skin_editor_desktop::editor_audio::window_recovery::restore::Cleanup (声明)`, `skin_editor_desktop::editor_audio::window_recovery::restore::Cleanup::step`, `skin_editor_desktop::editor_audio::window_recovery::sound::collect (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Observations` — [源码:63](../../../desktop/host/src/editor_audio/gui_verification.rs#L63)
- `impl Drop for Observations` — [源码:111](../../../desktop/host/src/editor_audio/gui_verification.rs#L111)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn new(reader: FirstObserver) -> Result<Self, String>` | impl Observations | [源码:64](../../../desktop/host/src/editor_audio/gui_verification.rs#L64) | skin_editor_infrastructure::separated_native::FirstObserver |
| `pub(super) fn with_capacity( reader: FirstObserver, capacity: usize, ) -> Result<Self, String>` | impl Observations | [源码:67](../../../desktop/host/src/editor_audio/gui_verification.rs#L67) | skin_editor_infrastructure::separated_native::FirstObserver |
| `pub(super) fn close(&mut self) -> bool` | impl Observations | [源码:103](../../../desktop/host/src/editor_audio/gui_verification.rs#L103) |  |
| `fn drop(&mut self)` | impl Drop for Observations | [源码:112](../../../desktop/host/src/editor_audio/gui_verification.rs#L112) |  |

## skin_editor_desktop::editor_audio::gui_verification::OutputProbe

[源码:24](../../../desktop/host/src/editor_audio/gui_verification.rs#L24) · enum

```rust
enum OutputProbe {
    None,
    Text,
    Mesh,
    GpuPath,
    Upload,
    RepaintOnly,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::editor_audio::gui_verification::OutputProbe::clone`, `skin_editor_desktop::editor_audio::gui_verification::OutputProbe::eq`, `skin_editor_desktop::editor_audio::gui_verification::Run::new`, `skin_editor_desktop::editor_audio::gui_verification::run_control (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for OutputProbe` — [源码:23](../../../desktop/host/src/editor_audio/gui_verification.rs#L23)
- `impl PartialEq for OutputProbe` — [源码:23](../../../desktop/host/src/editor_audio/gui_verification.rs#L23)
- `impl Copy for OutputProbe` — [源码:23](../../../desktop/host/src/editor_audio/gui_verification.rs#L23)
- `impl StructuralPartialEq for OutputProbe` — [源码:23](../../../desktop/host/src/editor_audio/gui_verification.rs#L23)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> OutputProbe` | impl Clone for OutputProbe | [源码:23](../../../desktop/host/src/editor_audio/gui_verification.rs#L23) | skin_editor_desktop::editor_audio::gui_verification::OutputProbe |
| `fn eq(&self, other: &OutputProbe) -> bool` | impl PartialEq for OutputProbe | [源码:23](../../../desktop/host/src/editor_audio/gui_verification.rs#L23) | skin_editor_desktop::editor_audio::gui_verification::OutputProbe |

## skin_editor_desktop::editor_audio::gui_verification::Probe

[源码:136](../../../desktop/host/src/editor_audio/gui_verification.rs#L136) · struct

```rust
struct Probe {
    at: Instant,
    request: u64,
    lane: Channel,
}
```

声明引用：`skin_editor_audio::Channel`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::editor_audio::gui_verification::Run (声明)`

## skin_editor_desktop::editor_audio::gui_verification::Run

[源码:172](../../../desktop/host/src/editor_audio/gui_verification.rs#L172) · struct

```rust
struct Run {
    editor: Editor,
    audio: Rc<RefCell<EditorAudio>>,
    path: PathBuf,
    observer: Option<Observations>,
    started: Instant,
    stage: u8,
    stage_at: Instant,
    pending: Option<Probe>,
    rows: Vec<String>,
    batch: Vec<u64>,
    batch_request: Option<u64>,
    refusal: Option<u64>,
    refused: bool,
    anchor: u64,
    quiet: u64,
    before_hot: u64,
    before_history: u64,
    baseline_pcm: usize,
    snapshot: Option<ProjectSnapshot>,
    nodes: usize,
    stats: NativeStats,
    pcm_peak: usize,
    checks: [bool; 8],
    failure: Option<String>,
    completed: bool,
    lifecycle: Vec<String>,
    visual: Vec<String>,
    ui_cost: [[u64; 3]; 18],
    max_live_targets: usize,
    rate: u32,
    channels: u16,
    period: u32,
    stream: Option<StreamObservation>,
    profile_work: bool,
    work_edges: [[u64; 6]; 2],
    gui_work: Option<GuiWork>,
    repaints: Option<Repaints>,
    adapter: String,
    control: Option<Control>,
    target_format: String,
    text_work: Option<TypedPluginHandle<Census>>,
    mesh_replay: Option<TypedPluginHandle<Replay>>,
    framework_trace: Option<Trace>,
    gpu_path: Option<TypedPluginHandle<Probe>>,
}
```

声明引用：`skin_editor_desktop::Editor`, `skin_editor_desktop::editor_audio::EditorAudio`, `skin_editor_desktop::editor_audio::gui_verification::GuiWork`, `skin_editor_desktop::editor_audio::gui_verification::Observations`, `skin_editor_desktop::editor_audio::gui_verification::Probe`, `skin_editor_desktop::framework_trace::Trace`, `skin_editor_desktop::gpu_path::Probe`, `skin_editor_desktop::gui_baseline::Control`, `skin_editor_desktop::mesh_replay::Replay`, `skin_editor_desktop::repaint_observation::Repaints`, `skin_editor_desktop::text_work::Census`, `skin_editor_infrastructure::background_stream::StreamObservation`, `skin_editor_infrastructure::separated_native::NativeStats`

实现／约束（显式或 derive，无类继承）：

- `impl Run` — [源码:219](../../../desktop/host/src/editor_audio/gui_verification.rs#L219)
- `impl App for Run` — [源码:982](../../../desktop/host/src/editor_audio/gui_verification.rs#L982)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn new( context: &CreationContext<'_>, path: PathBuf, streamed: bool, profile_work: bool, main_work: bool, mode: Option<Mode>, probe: OutputProbe, ) -> Result<Self, String>` | impl Run | [源码:220](../../../desktop/host/src/editor_audio/gui_verification.rs#L220) | skin_editor_desktop::editor_audio::gui_verification::OutputProbe, skin_editor_desktop::gui_baseline::Mode |
| `fn component(&self) -> AudioComponentId` | impl Run | [源码:383](../../../desktop/host/src/editor_audio/gui_verification.rs#L383) |  |
| `fn action(&mut self, a: AudioEditorAction)` | impl Run | [源码:386](../../../desktop/host/src/editor_audio/gui_verification.rs#L386) |  |
| `fn advance(&mut self, stage: u8)` | impl Run | [源码:392](../../../desktop/host/src/editor_audio/gui_verification.rs#L392) |  |
| `fn ready(&self) -> bool` | impl Run | [源码:433](../../../desktop/host/src/editor_audio/gui_verification.rs#L433) |  |
| `fn drain(&mut self) -> Result<(), String>` | impl Run | [源码:441](../../../desktop/host/src/editor_audio/gui_verification.rs#L441) |  |
| `fn step(&mut self) -> Result<(), String>` | impl Run | [源码:462](../../../desktop/host/src/editor_audio/gui_verification.rs#L462) |  |
| `fn report(&self) -> String` | impl Run | [源码:853](../../../desktop/host/src/editor_audio/gui_verification.rs#L853) |  |
| `fn draw_control( &mut self, root: &mut Ui, frame: &mut Frame, ) -> Result<(), String>` | impl Run | [源码:916](../../../desktop/host/src/editor_audio/gui_verification.rs#L916) |  |
| `fn raw_input_hook(&mut self, ctx: &Context, input: &mut RawInput)` | impl App for Run | [源码:983](../../../desktop/host/src/editor_audio/gui_verification.rs#L983) |  |
| `fn logic(&mut self, ctx: &Context, frame: &mut Frame)` | impl App for Run | [源码:994](../../../desktop/host/src/editor_audio/gui_verification.rs#L994) |  |
| `fn ui(&mut self, root: &mut Ui, frame: &mut Frame)` | impl App for Run | [源码:1024](../../../desktop/host/src/editor_audio/gui_verification.rs#L1024) |  |
| `fn clear_color(&self, visuals: &Visuals) -> [f32; 4]` | impl App for Run | [源码:1089](../../../desktop/host/src/editor_audio/gui_verification.rs#L1089) |  |
| `fn on_exit(&mut self)` | impl App for Run | [源码:1099](../../../desktop/host/src/editor_audio/gui_verification.rs#L1099) |  |

## skin_editor_desktop::editor_audio::gui_verification::Shared

[源码:22](../../../desktop/host/src/editor_audio/gui_verification.rs#L22) · struct

```rust
pub(super) struct Shared(pub(super) Rc<RefCell<EditorAudio>>);
```

声明引用：`skin_editor_desktop::editor_audio::EditorAudio`

实现／约束（显式或 derive，无类继承）：

- `impl AudioEditorPort for Shared` — [源码:45](../../../desktop/host/src/editor_audio/gui_verification.rs#L45)
- `impl SkinAudioPort for Shared` — [源码:32](../../../desktop/host/src/editor_audio/gui_verification.rs#L32)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn request(&mut self, a: AudioEditorAction) -> Result<(), AudioEditorError>` | impl AudioEditorPort for Shared | [源码:46](../../../desktop/host/src/editor_audio/gui_verification.rs#L46) |  |
| `fn audition_asset(&mut self, a: AssetAudition) -> Result<(), AudioEditorError>` | impl AudioEditorPort for Shared | [源码:49](../../../desktop/host/src/editor_audio/gui_verification.rs#L49) |  |
| `fn poll(&mut self) -> AudioEditorState` | impl AudioEditorPort for Shared | [源码:52](../../../desktop/host/src/editor_audio/gui_verification.rs#L52) |  |
| `fn retained_drafts(&self) -> Vec<DraftId>` | impl SkinAudioPort for Shared | [源码:33](../../../desktop/host/src/editor_audio/gui_verification.rs#L33) |  |
| `fn synchronize( &mut self, p: &SkinProject, s: SkinAudioSource, c: &[DraftId], ) -> Result<(), SkinAudioError>` | impl SkinAudioPort for Shared | [源码:36](../../../desktop/host/src/editor_audio/gui_verification.rs#L36) |  |

## skin_editor_desktop::editor_audio::gui_verification::baseline

[源码:1156](../../../desktop/host/src/editor_audio/gui_verification.rs#L1156) · fn

```rust
pub fn baseline(destination: &str, mode: Mode) -> Result<(), String>
```

声明引用：`skin_editor_desktop::gui_baseline::Mode`

## skin_editor_desktop::editor_audio::gui_verification::dispatch

[源码:1146](../../../desktop/host/src/editor_audio/gui_verification.rs#L1146) · fn

```rust
pub fn dispatch(destination: &str, main_work: bool) -> Result<(), String>
```

## skin_editor_desktop::editor_audio::gui_verification::gpu_path

[源码:1172](../../../desktop/host/src/editor_audio/gui_verification.rs#L1172) · fn

```rust
pub fn gpu_path(destination: &str, upload: bool) -> Result<(), String>
```

## skin_editor_desktop::editor_audio::gui_verification::main_work

[源码:1132](../../../desktop/host/src/editor_audio/gui_verification.rs#L1132) · fn

```rust
pub fn main_work(destination: &str) -> Result<(), String>
```

## skin_editor_desktop::editor_audio::gui_verification::mesh_replay

[源码:1169](../../../desktop/host/src/editor_audio/gui_verification.rs#L1169) · fn

```rust
pub fn mesh_replay(destination: &str) -> Result<(), String>
```

## skin_editor_desktop::editor_audio::gui_verification::profiled

[源码:1129](../../../desktop/host/src/editor_audio/gui_verification.rs#L1129) · fn

```rust
pub fn profiled(destination: &str) -> Result<(), String>
```

## skin_editor_desktop::editor_audio::gui_verification::repaint_only

[源码:1135](../../../desktop/host/src/editor_audio/gui_verification.rs#L1135) · fn

```rust
pub fn repaint_only(destination: &str) -> Result<(), String>
```

## skin_editor_desktop::editor_audio::gui_verification::run

[源码:1123](../../../desktop/host/src/editor_audio/gui_verification.rs#L1123) · fn

```rust
pub fn run(destination: &str) -> Result<(), String>
```

## skin_editor_desktop::editor_audio::gui_verification::run_control

[源码:1201](../../../desktop/host/src/editor_audio/gui_verification.rs#L1201) · fn

```rust
fn run_control(
    destination: &str,
    streamed: bool,
    profile_work: bool,
    main_work: bool,
    mode: Option<Mode>,
    probe: OutputProbe,
) -> Result<(), String>
```

声明引用：`skin_editor_desktop::editor_audio::gui_verification::OutputProbe`, `skin_editor_desktop::gui_baseline::Mode`

## skin_editor_desktop::editor_audio::gui_verification::run_mode

[源码:1186](../../../desktop/host/src/editor_audio/gui_verification.rs#L1186) · fn

```rust
fn run_mode(
    destination: &str,
    streamed: bool,
    profile_work: bool,
    main_work: bool,
) -> Result<(), String>
```

## skin_editor_desktop::editor_audio::gui_verification::streamed

[源码:1126](../../../desktop/host/src/editor_audio/gui_verification.rs#L1126) · fn

```rust
pub fn streamed(destination: &str) -> Result<(), String>
```

## skin_editor_desktop::editor_audio::gui_verification::text_work

[源码:1166](../../../desktop/host/src/editor_audio/gui_verification.rs#L1166) · fn

```rust
pub fn text_work(destination: &str) -> Result<(), String>
```

## skin_editor_desktop::editor_audio::gui_verification::wav

[源码:116](../../../desktop/host/src/editor_audio/gui_verification.rs#L116) · fn

```rust
fn wav(rate: u32, channels: u16, frames: u32) -> Vec<u8> ⓘ
```

## skin_editor_desktop::editor_audio::preparation

[源码:63](../../../desktop/host/src/editor_audio.rs#L63) · fn

```rust
fn preparation(p: Preparation) -> AudioPreparationState
```

声明引用：`skin_editor_audio::Preparation`

## skin_editor_desktop::editor_audio::source

[源码:70](../../../desktop/host/src/editor_audio.rs#L70) · fn

```rust
fn source(o: Origin) -> SkinAudioSource
```

声明引用：`skin_editor_audio::Origin`

## skin_editor_desktop::editor_audio::stability::Run

[源码:15](../../../desktop/host/src/editor_audio/stability.rs#L15) · struct

```rust
struct Run {
    product: bool,
    editor: Editor,
    audio: Option<Rc<RefCell<EditorAudio>>>,
    source: Option<StreamObservation>,
    path: PathBuf,
    resources_case: bool,
    origin: Instant,
    since: Instant,
    due: Instant,
    stage: u8,
    interval: Option<Interval>,
    before: [usize; 5],
    rows: Vec<String>,
    inputs: u64,
    visible: bool,
    geometry: [f32; 3],
    shape: [usize; 3],
    completed: bool,
    failure: Option<String>,
    adapter: String,
}
```

声明引用：`skin_editor_desktop::Editor`, `skin_editor_desktop::editor_audio::EditorAudio`, `skin_editor_desktop::stable_window::Interval`, `skin_editor_infrastructure::background_stream::StreamObservation`

实现／约束（显式或 derive，无类继承）：

- `impl Run` — [源码:40](../../../desktop/host/src/editor_audio/stability.rs#L40)
- `impl App for Run` — [源码:354](../../../desktop/host/src/editor_audio/stability.rs#L354)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn new( c: &CreationContext<'_>, path: PathBuf, resources_case: bool, product: bool, ) -> Result<Self, String>` | impl Run | [源码:41](../../../desktop/host/src/editor_audio/stability.rs#L41) |  |
| `fn usage(&self) -> Result<[usize; 5], String>` | impl Run | [源码:79](../../../desktop/host/src/editor_audio/stability.rs#L79) |  |
| `fn background(&self) -> u64` | impl Run | [源码:90](../../../desktop/host/src/editor_audio/stability.rs#L90) |  |
| `fn shape(&self) -> [usize; 3]` | impl Run | [源码:98](../../../desktop/host/src/editor_audio/stability.rs#L98) |  |
| `fn begin(&mut self, name: &str, stage: u8, ctx: &Context) -> Result<(), String>` | impl Run | [源码:110](../../../desktop/host/src/editor_audio/stability.rs#L110) |  |
| `fn action(&mut self, action: AudioEditorAction) -> Result<(), String>` | impl Run | [源码:136](../../../desktop/host/src/editor_audio/stability.rs#L136) |  |
| `fn load(&mut self) -> Result<(), String>` | impl Run | [源码:143](../../../desktop/host/src/editor_audio/stability.rs#L143) |  |
| `fn finish_due(&mut self, ctx: &Context) -> Result<(), String>` | impl Run | [源码:221](../../../desktop/host/src/editor_audio/stability.rs#L221) |  |
| `fn after_draw(&mut self, ctx: &Context) -> Result<(), String>` | impl Run | [源码:272](../../../desktop/host/src/editor_audio/stability.rs#L272) |  |
| `fn report(&self) -> String` | impl Run | [源码:327](../../../desktop/host/src/editor_audio/stability.rs#L327) |  |
| `fn raw_input_hook(&mut self, _: &Context, input: &mut RawInput)` | impl App for Run | [源码:355](../../../desktop/host/src/editor_audio/stability.rs#L355) |  |
| `fn logic(&mut self, ctx: &Context, frame: &mut Frame)` | impl App for Run | [源码:361](../../../desktop/host/src/editor_audio/stability.rs#L361) |  |
| `fn ui(&mut self, root: &mut Ui, frame: &mut Frame)` | impl App for Run | [源码:369](../../../desktop/host/src/editor_audio/stability.rs#L369) |  |
| `fn on_exit(&mut self)` | impl App for Run | [源码:402](../../../desktop/host/src/editor_audio/stability.rs#L402) |  |

## skin_editor_desktop::editor_audio::stability::error

[源码:37](../../../desktop/host/src/editor_audio/stability.rs#L37) · fn

```rust
fn error(e: impl Debug) -> String
```

## skin_editor_desktop::editor_audio::stability::physical_input

[源码:409](../../../desktop/host/src/editor_audio/stability.rs#L409) · fn

```rust
fn physical_input(event: &Event) -> bool
```

## skin_editor_desktop::editor_audio::stability::run

[源码:415](../../../desktop/host/src/editor_audio/stability.rs#L415) · fn

```rust
pub fn run(destination: &str, resources_case: bool) -> Result<(), String>
```

## skin_editor_desktop::editor_audio::stability::run_policy

[源码:418](../../../desktop/host/src/editor_audio/stability.rs#L418) · fn

```rust
pub fn run_policy(
    destination: &str,
    resources_case: bool,
    product: bool,
) -> Result<(), String>
```

## skin_editor_desktop::editor_audio::window_recovery::Returned

[源码:76](../../../desktop/host/src/editor_audio/window_recovery.rs#L76) · struct

```rust
struct Returned {
    frame_key: Option<FrameKey>,
    surface_failures: usize,
    editor: Editor,
    audio: Rc<RefCell<EditorAudio>>,
    audio_signal: NativeFaultSignal,
    held: Option<FullOutput>,
    record: String,
    canonical: Vec<u8>,
    failure: Option<String>,
}
```

声明引用：`skin_editor_desktop::Editor`, `skin_editor_desktop::editor_audio::EditorAudio`, `skin_editor_infrastructure::separated_native::NativeFaultSignal`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::editor_audio::window_recovery::Run (声明)`

## skin_editor_desktop::editor_audio::window_recovery::Run

[源码:87](../../../desktop/host/src/editor_audio/window_recovery.rs#L87) · struct

```rust
struct Run {
    plan: Plan,
    fault: Option<Arc<Signals>>,
    late_frame_rejected: bool,
    held_key: Option<FrameKey>,
    previous_key: Option<FrameKey>,
    previous_frame_rejected: bool,
    editor: Option<Editor>,
    audio: Rc<RefCell<EditorAudio>>,
    observer: Option<Observations>,
    probes: Probes,
    path: PathBuf,
    index: usize,
    phase: u8,
    started: Instant,
    playing: Option<Instant>,
    issued: usize,
    pause: u64,
    restore_anchor: Option<u64>,
    screens: Vec<[usize; 4]>,
    expected_screens: Vec<[usize; 2]>,
    shot_tx: SyncSender<Arc<ColorImage>>,
    shot_rx: Receiver<Arc<ColorImage>>,
    software_scale: bool,
    native_before: f32,
    native_after: f32,
    physical_dpi: bool,
    held: Option<FullOutput>,
    canonical: Vec<u8>,
    max_pixel_error: u8,
    notice: Arc<AtomicU8>,
    failure: Option<String>,
    out: Rc<RefCell<Option<Returned>>>,
    old_voice: usize,
    pending: bool,
    baseline: ProjectSnapshot,
    audio_signal: NativeFaultSignal,
}
```

声明引用：`skin_editor_desktop::Editor`, `skin_editor_desktop::editor_audio::EditorAudio`, `skin_editor_desktop::editor_audio::combined::probes::Probes`, `skin_editor_desktop::editor_audio::gui_verification::Observations`, `skin_editor_desktop::editor_audio::window_recovery::Returned`, `skin_editor_desktop::editor_audio::window_recovery::fault::Plan`, `skin_editor_desktop::editor_audio::window_recovery::fault::Signals`, `skin_editor_infrastructure::separated_native::NativeFaultSignal`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::editor_audio::window_recovery::report::window (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Run` — [源码:125](../../../desktop/host/src/editor_audio/window_recovery.rs#L125)
- `impl App for Run` — [源码:444](../../../desktop/host/src/editor_audio/window_recovery.rs#L444)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn shot(&mut self, ctx: &Context)` | impl Run | [源码:126](../../../desktop/host/src/editor_audio/window_recovery.rs#L126) |  |
| `fn screenshots(&mut self) -> Result<(), String>` | impl Run | [源码:137](../../../desktop/host/src/editor_audio/window_recovery.rs#L137) |  |
| `fn step(&mut self, ctx: &Context) -> Result<(), String>` | impl Run | [源码:176](../../../desktop/host/src/editor_audio/window_recovery.rs#L176) |  |
| `fn logic(&mut self, ctx: &Context, frame: &mut Frame)` | impl App for Run | [源码:445](../../../desktop/host/src/editor_audio/window_recovery.rs#L445) |  |
| `fn ui(&mut self, ui: &mut Ui, frame: &mut Frame)` | impl App for Run | [源码:492](../../../desktop/host/src/editor_audio/window_recovery.rs#L492) |  |
| `fn on_exit(&mut self)` | impl App for Run | [源码:562](../../../desktop/host/src/editor_audio/window_recovery.rs#L562) |  |

## skin_editor_desktop::editor_audio::window_recovery::Seed

[源码:31](../../../desktop/host/src/editor_audio/window_recovery.rs#L31) · struct

```rust
struct Seed {
    session: ProjectSession,
    ui: EditorUi,
    images: Images,
    preview: PreviewState,
    gpu: RebuildSeed,
}
```

声明引用：`skin_editor_desktop::presentation::RebuildSeed`, `skin_editor_infrastructure::Images`, `skin_editor_preview::PreviewState`, `skin_editor_ui::EditorUi`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::editor_audio::window_recovery::init_failure::Recovered (声明)`, `skin_editor_desktop::editor_audio::window_recovery::init_failure::run (声明)`, `skin_editor_desktop::editor_audio::window_recovery::init_failure::verify_redo (声明)`, `skin_editor_desktop::editor_audio::window_recovery::report::reject_creation (声明)`, `skin_editor_desktop::editor_audio::window_recovery::restore::Owner (声明)`, `skin_editor_desktop::editor_audio::window_recovery::restore::State::seed`

实现／约束（显式或 derive，无类继承）：

- `impl Seed` — [源码:38](../../../desktop/host/src/editor_audio/window_recovery.rs#L38)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn restore( self, ctx: &Context, state: &RenderState, ) -> Result<Editor, (String, Box<Self>)>` | impl Seed | [源码:39](../../../desktop/host/src/editor_audio/window_recovery.rs#L39) | skin_editor_desktop::Editor |

## skin_editor_desktop::editor_audio::window_recovery::activity::ResultRecord

[源码:6](../../../desktop/host/src/editor_audio\window_recovery/activity.rs#L6) · struct

```rust
pub(super) struct ResultRecord {
    pub record: String,
    pub held: FullOutput,
    pub frame: FrameKey,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::editor_audio::window_recovery::activity::exercise (声明)`

## skin_editor_desktop::editor_audio::window_recovery::activity::button

[源码:38](../../../desktop/host/src/editor_audio\window_recovery/activity.rs#L38) · fn

```rust
fn button(pos: Pos2, pressed: bool) -> Event
```

## skin_editor_desktop::editor_audio::window_recovery::activity::draft

[源码:46](../../../desktop/host/src/editor_audio\window_recovery/activity.rs#L46) · fn

```rust
fn draft(e: &Editor) -> Result<DraftState, String>
```

声明引用：`skin_editor_desktop::Editor`

## skin_editor_desktop::editor_audio::window_recovery::activity::exercise

[源码:53](../../../desktop/host/src/editor_audio\window_recovery/activity.rs#L53) · fn

```rust
pub(super) fn exercise(
    e: &mut Editor,
    ctx: &Context,
    audio: &Rc<RefCell<EditorAudio>>,
    observer: &Observations,
    plan: Plan,
) -> Result<ResultRecord, String>
```

声明引用：`skin_editor_desktop::Editor`, `skin_editor_desktop::editor_audio::EditorAudio`, `skin_editor_desktop::editor_audio::gui_verification::Observations`, `skin_editor_desktop::editor_audio::window_recovery::activity::ResultRecord`, `skin_editor_desktop::editor_audio::window_recovery::fault::Plan`

## skin_editor_desktop::editor_audio::window_recovery::activity::pass

[源码:11](../../../desktop/host/src/editor_audio\window_recovery/activity.rs#L11) · fn

```rust
fn pass(
    e: &mut Editor,
    ctx: &Context,
    events: Vec<Event>,
) -> Result<([u32; 2], u64), String>
```

声明引用：`skin_editor_desktop::Editor`

## skin_editor_desktop::editor_audio::window_recovery::activity::ready_frame

[源码:35](../../../desktop/host/src/editor_audio\window_recovery/activity.rs#L35) · fn

```rust
fn ready_frame(e: &mut Editor, viewport: ([u32; 2], u64)) -> Result<(), String>
```

声明引用：`skin_editor_desktop::Editor`

## skin_editor_desktop::editor_audio::window_recovery::backend::Backend

[源码:19](../../../desktop/host/src/editor_audio\window_recovery/backend.rs#L19) · struct

```rust
pub(super) struct Backend {
    pub instance: Instance,
    pub loss: DeviceLoss,
    pub notice: Arc<AtomicU8>,
    pub renderer_gone: Arc<AtomicBool>,
    pub options: NativeOptions,
    pub adapter_name: Vec<u8>,
}
```

声明引用：`skin_editor_desktop::presentation::DeviceLoss`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::editor_audio::window_recovery::restore::State::open_backend`

实现／约束（显式或 derive，无类继承）：

- `impl Backend` — [源码:27](../../../desktop/host/src/editor_audio\window_recovery/backend.rs#L27)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn new() -> Result<Self, String>` | impl Backend | [源码:28](../../../desktop/host/src/editor_audio\window_recovery/backend.rs#L28) |  |
| `pub fn with_fault(fault: Option<Arc<Signals>>) -> Result<Self, String>` | impl Backend | [源码:31](../../../desktop/host/src/editor_audio\window_recovery/backend.rs#L31) | skin_editor_desktop::editor_audio::window_recovery::fault::Signals |

## skin_editor_desktop::editor_audio::window_recovery::backend::RendererWitness

[源码:13](../../../desktop/host/src/editor_audio\window_recovery/backend.rs#L13) · struct

```rust
pub(super) struct RendererWitness(pub Arc<AtomicBool>);
```

实现／约束（显式或 derive，无类继承）：

- `impl Drop for RendererWitness` — [源码:14](../../../desktop/host/src/editor_audio\window_recovery/backend.rs#L14)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn drop(&mut self)` | impl Drop for RendererWitness | [源码:15](../../../desktop/host/src/editor_audio\window_recovery/backend.rs#L15) |  |

## skin_editor_desktop::editor_audio::window_recovery::backend::destroy

[源码:138](../../../desktop/host/src/editor_audio\window_recovery/backend.rs#L138) · fn

```rust
pub(super) fn destroy(device: &Device, notice: &AtomicU8) -> Result<(), String>
```

## skin_editor_desktop::editor_audio::window_recovery::backend::retired

[源码:153](../../../desktop/host/src/editor_audio\window_recovery/backend.rs#L153) · fn

```rust
pub(super) fn retired(
    renderer_gone: bool,
    surfaces: bool,
    devices: bool,
    targets: usize,
    worker: bool,
    importer: bool,
) -> bool
```

## skin_editor_desktop::editor_audio::window_recovery::backend::surface_ids_only

[源码:167](../../../desktop/host/src/editor_audio\window_recovery/backend.rs#L167) · fn

```rust
pub(super) fn surface_ids_only(
    allocated: usize,
    kept: usize,
    failures: usize,
) -> bool
```

## skin_editor_desktop::editor_audio::window_recovery::error

[源码:28](../../../desktop/host/src/editor_audio/window_recovery.rs#L28) · fn

```rust
fn error(e: impl Debug) -> String
```

## skin_editor_desktop::editor_audio::window_recovery::fault::Inject

[源码:126](../../../desktop/host/src/editor_audio\window_recovery/fault.rs#L126) · struct

```rust
struct Inject {
    signals: Arc<Signals>,
    plan: Plan,
    device: Device,
}
```

声明引用：`skin_editor_desktop::editor_audio::window_recovery::fault::Plan`, `skin_editor_desktop::editor_audio::window_recovery::fault::Signals`

实现／约束（显式或 derive，无类继承）：

- `impl CallbackTrait for Inject` — [源码:131](../../../desktop/host/src/editor_audio\window_recovery/fault.rs#L131)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn prepare( &self, device: &Device, _: &Queue, _: &ScreenDescriptor, _: &mut CommandEncoder, _: &mut CallbackResources, ) -> Vec<CommandBuffer>` | impl CallbackTrait for Inject | [源码:132](../../../desktop/host/src/editor_audio\window_recovery/fault.rs#L132) |  |
| `fn paint( &self, _: PaintCallbackInfo, _: &mut RenderPass<'static>, _: &CallbackResources, )` | impl CallbackTrait for Inject | [源码:146](../../../desktop/host/src/editor_audio\window_recovery/fault.rs#L146) |  |

## skin_editor_desktop::editor_audio::window_recovery::fault::Plan

[源码:9](../../../desktop/host/src/editor_audio\window_recovery/fault.rs#L9) · enum

```rust
pub(super) enum Plan {
    Planned,
    Prepare,
    Paint,
    RebuildFailure,
    InitConsumed,
    InitPrepared,
    InitAudio,
    CleanupHeld,
    EndpointHeld,
    DraftGpu,
    DraftAudio,
    DraftBoth,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::editor_audio::window_recovery::Run (声明)`, `skin_editor_desktop::editor_audio::window_recovery::activity::exercise (声明)`, `skin_editor_desktop::editor_audio::window_recovery::fault::Inject (声明)`, `skin_editor_desktop::editor_audio::window_recovery::fault::Plan::clone`, `skin_editor_desktop::editor_audio::window_recovery::fault::Plan::eq`, `skin_editor_desktop::editor_audio::window_recovery::fault::append (声明)`, `skin_editor_desktop::editor_audio::window_recovery::init_failure::run (声明)`, `skin_editor_desktop::editor_audio::window_recovery::report::run (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Plan` — [源码:23](../../../desktop/host/src/editor_audio\window_recovery/fault.rs#L23)
- `impl Clone for Plan` — [源码:8](../../../desktop/host/src/editor_audio\window_recovery/fault.rs#L8)
- `impl Debug for Plan` — [源码:8](../../../desktop/host/src/editor_audio\window_recovery/fault.rs#L8)
- `impl PartialEq for Plan` — [源码:8](../../../desktop/host/src/editor_audio\window_recovery/fault.rs#L8)
- `impl Copy for Plan` — [源码:8](../../../desktop/host/src/editor_audio\window_recovery/fault.rs#L8)
- `impl Eq for Plan` — [源码:8](../../../desktop/host/src/editor_audio\window_recovery/fault.rs#L8)
- `impl StructuralPartialEq for Plan` — [源码:8](../../../desktop/host/src/editor_audio\window_recovery/fault.rs#L8)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn code(self) -> u8` | impl Plan | [源码:24](../../../desktop/host/src/editor_audio\window_recovery/fault.rs#L24) |  |
| `pub fn active_edit(self) -> bool` | impl Plan | [源码:40](../../../desktop/host/src/editor_audio\window_recovery/fault.rs#L40) |  |
| `fn clone(&self) -> Plan` | impl Clone for Plan | [源码:8](../../../desktop/host/src/editor_audio\window_recovery/fault.rs#L8) | skin_editor_desktop::editor_audio::window_recovery::fault::Plan |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for Plan | [源码:8](../../../desktop/host/src/editor_audio\window_recovery/fault.rs#L8) |  |
| `fn eq(&self, other: &Plan) -> bool` | impl PartialEq for Plan | [源码:8](../../../desktop/host/src/editor_audio\window_recovery/fault.rs#L8) | skin_editor_desktop::editor_audio::window_recovery::fault::Plan |

## skin_editor_desktop::editor_audio::window_recovery::fault::Signals

[源码:45](../../../desktop/host/src/editor_audio\window_recovery/fault.rs#L45) · struct

```rust
pub(super) struct Signals {
    pub fired: AtomicU64,
    pub prepares: AtomicU64,
    pub paints: AtomicU64,
    pub errors: AtomicU64,
    pub validation_errors: AtomicU64,
    pub internal_errors: AtomicU64,
    pub out_of_memory: AtomicU64,
    pub premature_errors: AtomicU64,
    pub surface_status: AtomicU64,
    pub noticed: AtomicU64,
    pub surface_faults: AtomicU64,
    pub device_notices: AtomicU64,
    pub logic_observed: AtomicU64,
    pub audio_closed_before_logic: AtomicU64,
    audio: OnceLock<NativeFaultSignal>,
    context: OnceLock<Context>,
}
```

声明引用：`skin_editor_infrastructure::separated_native::NativeFaultSignal`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::editor_audio::window_recovery::Run (声明)`, `skin_editor_desktop::editor_audio::window_recovery::backend::Backend::with_fault`, `skin_editor_desktop::editor_audio::window_recovery::fault::Inject (声明)`, `skin_editor_desktop::editor_audio::window_recovery::fault::Signals::default`, `skin_editor_desktop::editor_audio::window_recovery::fault::append (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Signals` — [源码:63](../../../desktop/host/src/editor_audio\window_recovery/fault.rs#L63)
- `impl Default for Signals` — [源码:44](../../../desktop/host/src/editor_audio\window_recovery/fault.rs#L44)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn bind( &self, audio: NativeFaultSignal, context: Context, ) -> Result<(), String>` | impl Signals | [源码:64](../../../desktop/host/src/editor_audio\window_recovery/fault.rs#L64) | skin_editor_infrastructure::separated_native::NativeFaultSignal |
| `pub fn notify(&self)` | impl Signals | [源码:72](../../../desktop/host/src/editor_audio\window_recovery/fault.rs#L72) |  |
| `pub fn record_error(&self)` | impl Signals | [源码:81](../../../desktop/host/src/editor_audio\window_recovery/fault.rs#L81) |  |
| `pub fn native_error(&self, error: &Error)` | impl Signals | [源码:87](../../../desktop/host/src/editor_audio\window_recovery/fault.rs#L87) |  |
| `fn fire(&self, device: &Device)` | impl Signals | [源码:96](../../../desktop/host/src/editor_audio\window_recovery/fault.rs#L96) |  |
| `pub fn json(&self) -> String` | impl Signals | [源码:105](../../../desktop/host/src/editor_audio\window_recovery/fault.rs#L105) |  |
| `fn default() -> Signals` | impl Default for Signals | [源码:44](../../../desktop/host/src/editor_audio\window_recovery/fault.rs#L44) | skin_editor_desktop::editor_audio::window_recovery::fault::Signals |

## skin_editor_desktop::editor_audio::window_recovery::fault::append

[源码:158](../../../desktop/host/src/editor_audio\window_recovery/fault.rs#L158) · fn

```rust
pub(super) fn append(ui: &Ui, signals: Arc<Signals>, plan: Plan, device: Device)
```

声明引用：`skin_editor_desktop::editor_audio::window_recovery::fault::Plan`, `skin_editor_desktop::editor_audio::window_recovery::fault::Signals`

## skin_editor_desktop::editor_audio::window_recovery::init_failure::Recovered

[源码:115](../../../desktop/host/src/editor_audio\window_recovery/init_failure.rs#L115) · struct

```rust
pub(super) struct Recovered {
    pub seed: Seed,
    pub record: String,
    pub frame: Option<FrameKey>,
    pub loss: DeviceLoss,
    pub audio: Option<NativeFaultSignal>,
}
```

声明引用：`skin_editor_desktop::editor_audio::window_recovery::Seed`, `skin_editor_desktop::presentation::DeviceLoss`, `skin_editor_infrastructure::separated_native::NativeFaultSignal`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::editor_audio::window_recovery::init_failure::run (声明)`

## skin_editor_desktop::editor_audio::window_recovery::init_failure::Stamp

[源码:10](../../../desktop/host/src/editor_audio\window_recovery/init_failure.rs#L10) · struct

```rust
pub(super) struct Stamp {
    snapshot: ProjectSnapshot,
    entries: Vec<HistorySummary>,
    projects: Vec<HistoricalSnapshot>,
    statistics: HistoryStatistics,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::editor_audio::window_recovery::init_failure::Stamp::clone`, `skin_editor_desktop::editor_audio::window_recovery::init_failure::Stamp::eq`

实现／约束（显式或 derive，无类继承）：

- `impl Stamp` — [源码:16](../../../desktop/host/src/editor_audio\window_recovery/init_failure.rs#L16)
- `impl Clone for Stamp` — [源码:9](../../../desktop/host/src/editor_audio\window_recovery/init_failure.rs#L9)
- `impl Debug for Stamp` — [源码:9](../../../desktop/host/src/editor_audio\window_recovery/init_failure.rs#L9)
- `impl PartialEq for Stamp` — [源码:9](../../../desktop/host/src/editor_audio\window_recovery/init_failure.rs#L9)
- `impl StructuralPartialEq for Stamp` — [源码:9](../../../desktop/host/src/editor_audio\window_recovery/init_failure.rs#L9)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn same_committed(&self, other: &Self) -> bool` | impl Stamp | [源码:17](../../../desktop/host/src/editor_audio\window_recovery/init_failure.rs#L17) |  |
| `pub fn requests(&self) -> usize` | impl Stamp | [源码:23](../../../desktop/host/src/editor_audio\window_recovery/init_failure.rs#L23) |  |
| `pub fn read(session: &ProjectSession) -> Result<Self, String>` | impl Stamp | [源码:26](../../../desktop/host/src/editor_audio\window_recovery/init_failure.rs#L26) |  |
| `fn clone(&self) -> Stamp` | impl Clone for Stamp | [源码:9](../../../desktop/host/src/editor_audio\window_recovery/init_failure.rs#L9) | skin_editor_desktop::editor_audio::window_recovery::init_failure::Stamp |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for Stamp | [源码:9](../../../desktop/host/src/editor_audio\window_recovery/init_failure.rs#L9) |  |
| `fn eq(&self, other: &Stamp) -> bool` | impl PartialEq for Stamp | [源码:9](../../../desktop/host/src/editor_audio\window_recovery/init_failure.rs#L9) | skin_editor_desktop::editor_audio::window_recovery::init_failure::Stamp |

## skin_editor_desktop::editor_audio::window_recovery::init_failure::fixture

[源码:63](../../../desktop/host/src/editor_audio\window_recovery/init_failure.rs#L63) · fn

```rust
pub(super) fn fixture(editor: &mut Editor) -> Result<(), String>
```

声明引用：`skin_editor_desktop::Editor`

## skin_editor_desktop::editor_audio::window_recovery::init_failure::run

[源码:122](../../../desktop/host/src/editor_audio\window_recovery/init_failure.rs#L122) · fn

```rust
pub(super) fn run(
    seed: Seed,
    audio: Rc<RefCell<EditorAudio>>,
    path: &Path,
    plan: Plan,
) -> Result<Recovered, String>
```

声明引用：`skin_editor_desktop::editor_audio::EditorAudio`, `skin_editor_desktop::editor_audio::window_recovery::Seed`, `skin_editor_desktop::editor_audio::window_recovery::fault::Plan`, `skin_editor_desktop::editor_audio::window_recovery::init_failure::Recovered`

## skin_editor_desktop::editor_audio::window_recovery::init_failure::verify_redo

[源码:88](../../../desktop/host/src/editor_audio\window_recovery/init_failure.rs#L88) · fn

```rust
pub(super) fn verify_redo(seed: &mut Seed) -> Result<(), String>
```

声明引用：`skin_editor_desktop::editor_audio::window_recovery::Seed`

## skin_editor_desktop::editor_audio::window_recovery::report::reject_creation

[源码:412](../../../desktop/host/src/editor_audio\window_recovery/report.rs#L412) · fn

```rust
fn reject_creation(seed: &Seed) -> Result<(), String>
```

声明引用：`skin_editor_desktop::editor_audio::window_recovery::Seed`

## skin_editor_desktop::editor_audio::window_recovery::report::run

[源码:42](../../../desktop/host/src/editor_audio\window_recovery/report.rs#L42) · fn

```rust
pub(super) fn run(destination: &str, plan: Plan) -> Result<(), String>
```

声明引用：`skin_editor_desktop::editor_audio::window_recovery::fault::Plan`

## skin_editor_desktop::editor_audio::window_recovery::report::window

[源码:2](../../../desktop/host/src/editor_audio\window_recovery/report.rs#L2) · fn

```rust
pub(super) fn window(r: &Run, a: &EditorAudio, passes: u64) -> String
```

声明引用：`skin_editor_desktop::editor_audio::EditorAudio`, `skin_editor_desktop::editor_audio::window_recovery::Run`

## skin_editor_desktop::editor_audio::window_recovery::restore::Attempt

[源码:272](../../../desktop/host/src/editor_audio\window_recovery/restore.rs#L272) · struct

```rust
pub(super) struct Attempt<'a> {
    loan: Loan<'a, State>,
    notice: Arc<AtomicU8>,
    audio: Option<Rc<RefCell<EditorAudio>>>,
    pub observer: Option<Observations>,
    pub held: Option<FullOutput>,
    pub cleanup_once: bool,
}
```

声明引用：`skin_editor_desktop::editor_audio::EditorAudio`, `skin_editor_desktop::editor_audio::gui_verification::Observations`, `skin_editor_desktop::editor_audio::window_recovery::restore::Loan`, `skin_editor_desktop::editor_audio::window_recovery::restore::State`

实现／约束（显式或 derive，无类继承）：

- `impl<'a> Attempt<'a>` — [源码:280](../../../desktop/host/src/editor_audio\window_recovery/restore.rs#L280)
- `impl Drop for Attempt<'_>` — [源码:358](../../../desktop/host/src/editor_audio\window_recovery/restore.rs#L358)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn begin( slot: &'a mut Slot<State>, ctx: &Context, state: &RenderState, notice: Arc<AtomicU8>, ) -> Result<Self, String>` | impl<'a> Attempt<'a> | [源码:281](../../../desktop/host/src/editor_audio\window_recovery/restore.rs#L281) | skin_editor_desktop::editor_audio::window_recovery::restore::Slot, skin_editor_desktop::editor_audio::window_recovery::restore::State |
| `pub fn fresh( slot: &'a mut Slot<State>, editor: Editor, notice: Arc<AtomicU8>, ) -> Self` | impl<'a> Attempt<'a> | [源码:312](../../../desktop/host/src/editor_audio\window_recovery/restore.rs#L312) | skin_editor_desktop::Editor, skin_editor_desktop::editor_audio::window_recovery::restore::Slot, skin_editor_desktop::editor_audio::window_recovery::restore::State |
| `pub fn editor(&mut self) -> &mut Editor` | impl<'a> Attempt<'a> | [源码:331](../../../desktop/host/src/editor_audio\window_recovery/restore.rs#L331) | skin_editor_desktop::Editor |
| `pub fn attach_audio(&mut self, audio: Rc<RefCell<EditorAudio>>)` | impl<'a> Attempt<'a> | [源码:343](../../../desktop/host/src/editor_audio\window_recovery/restore.rs#L343) | skin_editor_desktop::editor_audio::EditorAudio |
| `pub fn prepared(&mut self)` | impl<'a> Attempt<'a> | [源码:346](../../../desktop/host/src/editor_audio\window_recovery/restore.rs#L346) |  |
| `pub fn commit(self) -> Editor` | impl<'a> Attempt<'a> | [源码:349](../../../desktop/host/src/editor_audio\window_recovery/restore.rs#L349) | skin_editor_desktop::Editor |
| `fn drop(&mut self)` | impl Drop for Attempt<'_> | [源码:359](../../../desktop/host/src/editor_audio\window_recovery/restore.rs#L359) |  |

## skin_editor_desktop::editor_audio::window_recovery::restore::Cleanup

[源码:195](../../../desktop/host/src/editor_audio\window_recovery/restore.rs#L195) · struct

```rust
struct Cleanup {
    notice: Arc<AtomicU8>,
    audio: Option<Rc<RefCell<EditorAudio>>>,
    observer: Option<Observations>,
    held: Option<FullOutput>,
    gate: Gate,
}
```

声明引用：`skin_editor_desktop::editor_audio::EditorAudio`, `skin_editor_desktop::editor_audio::gui_verification::Observations`, `skin_editor_desktop::editor_audio::window_recovery::restore::Gate`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::editor_audio::window_recovery::restore::State (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Cleanup` — [源码:202](../../../desktop/host/src/editor_audio\window_recovery/restore.rs#L202)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn step( editor: &mut Editor, notice: &AtomicU8, audio: &Option<Rc<RefCell<EditorAudio>>>, observer: &mut Option<Observations>, held: &mut Option<FullOutput>, observer_closed: &mut bool, held_blocked: &mut bool, ) -> Result<(), Failure>` | impl Cleanup | [源码:203](../../../desktop/host/src/editor_audio\window_recovery/restore.rs#L203) | skin_editor_desktop::Editor, skin_editor_desktop::editor_audio::EditorAudio, skin_editor_desktop::editor_audio::gui_verification::Observations, skin_editor_desktop::editor_audio::window_recovery::restore::Failure |

## skin_editor_desktop::editor_audio::window_recovery::restore::Failure

[源码:167](../../../desktop/host/src/editor_audio\window_recovery/restore.rs#L167) · enum

```rust
enum Failure {
    Pending(&'static str),
    Terminal(&'static str),
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::editor_audio::window_recovery::restore::Cleanup::step`, `skin_editor_desktop::editor_audio::window_recovery::restore::Failure::clone`, `skin_editor_desktop::editor_audio::window_recovery::restore::Failure::eq`, `skin_editor_desktop::editor_audio::window_recovery::restore::Gate (声明)`, `skin_editor_desktop::editor_audio::window_recovery::restore::Gate::run`

实现／约束（显式或 derive，无类继承）：

- `impl Failure` — [源码:171](../../../desktop/host/src/editor_audio\window_recovery/restore.rs#L171)
- `impl Clone for Failure` — [源码:166](../../../desktop/host/src/editor_audio\window_recovery/restore.rs#L166)
- `impl Debug for Failure` — [源码:166](../../../desktop/host/src/editor_audio\window_recovery/restore.rs#L166)
- `impl PartialEq for Failure` — [源码:166](../../../desktop/host/src/editor_audio\window_recovery/restore.rs#L166)
- `impl Copy for Failure` — [源码:166](../../../desktop/host/src/editor_audio\window_recovery/restore.rs#L166)
- `impl StructuralPartialEq for Failure` — [源码:166](../../../desktop/host/src/editor_audio\window_recovery/restore.rs#L166)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn message(self) -> &'static str` | impl Failure | [源码:172](../../../desktop/host/src/editor_audio\window_recovery/restore.rs#L172) |  |
| `fn clone(&self) -> Failure` | impl Clone for Failure | [源码:166](../../../desktop/host/src/editor_audio\window_recovery/restore.rs#L166) | skin_editor_desktop::editor_audio::window_recovery::restore::Failure |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for Failure | [源码:166](../../../desktop/host/src/editor_audio\window_recovery/restore.rs#L166) |  |
| `fn eq(&self, other: &Failure) -> bool` | impl PartialEq for Failure | [源码:166](../../../desktop/host/src/editor_audio\window_recovery/restore.rs#L166) | skin_editor_desktop::editor_audio::window_recovery::restore::Failure |

## skin_editor_desktop::editor_audio::window_recovery::restore::Gate

[源码:179](../../../desktop/host/src/editor_audio\window_recovery/restore.rs#L179) · struct

```rust
struct Gate(Option<Failure>);
```

声明引用：`skin_editor_desktop::editor_audio::window_recovery::restore::Failure`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::editor_audio::window_recovery::restore::Cleanup (声明)`, `skin_editor_desktop::editor_audio::window_recovery::restore::Gate::default`

实现／约束（显式或 derive，无类继承）：

- `impl Gate` — [源码:180](../../../desktop/host/src/editor_audio\window_recovery/restore.rs#L180)
- `impl Default for Gate` — [源码:178](../../../desktop/host/src/editor_audio\window_recovery/restore.rs#L178)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn terminal(&self) -> bool` | impl Gate | [源码:181](../../../desktop/host/src/editor_audio\window_recovery/restore.rs#L181) |  |
| `fn run( &mut self, work: impl FnOnce() -> Result<(), Failure>, ) -> Result<(), Failure>` | impl Gate | [源码:184](../../../desktop/host/src/editor_audio\window_recovery/restore.rs#L184) | skin_editor_desktop::editor_audio::window_recovery::restore::Failure |
| `fn default() -> Gate` | impl Default for Gate | [源码:178](../../../desktop/host/src/editor_audio\window_recovery/restore.rs#L178) | skin_editor_desktop::editor_audio::window_recovery::restore::Gate |

## skin_editor_desktop::editor_audio::window_recovery::restore::Loan

[源码:26](../../../desktop/host/src/editor_audio\window_recovery/restore.rs#L26) · struct

```rust
pub(super) struct Loan<'a, T> {
    slot: &'a mut Slot<T>,
    value: Option<T>,
}
```

声明引用：`skin_editor_desktop::editor_audio::window_recovery::restore::Slot`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::editor_audio::window_recovery::restore::Attempt (声明)`, `skin_editor_desktop::editor_audio::window_recovery::restore::Slot::take`

实现／约束（显式或 derive，无类继承）：

- `impl<T> Loan<'_, T>` — [源码:30](../../../desktop/host/src/editor_audio\window_recovery/restore.rs#L30)
- `impl<T> Drop for Loan<'_, T>` — [源码:38](../../../desktop/host/src/editor_audio\window_recovery/restore.rs#L38)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn get_mut(&mut self) -> &mut T` | impl<T> Loan<'_, T> | [源码:31](../../../desktop/host/src/editor_audio\window_recovery/restore.rs#L31) |  |
| `pub fn finish(&mut self) -> T` | impl<T> Loan<'_, T> | [源码:34](../../../desktop/host/src/editor_audio\window_recovery/restore.rs#L34) |  |
| `fn drop(&mut self)` | impl<T> Drop for Loan<'_, T> | [源码:39](../../../desktop/host/src/editor_audio\window_recovery/restore.rs#L39) |  |

## skin_editor_desktop::editor_audio::window_recovery::restore::Owner

[源码:45](../../../desktop/host/src/editor_audio\window_recovery/restore.rs#L45) · enum

```rust
pub(super) enum Owner {
    Seed(Box<Seed>),
    Editor(Box<Editor>),
}
```

声明引用：`skin_editor_desktop::Editor`, `skin_editor_desktop::editor_audio::window_recovery::Seed`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::editor_audio::window_recovery::restore::State (声明)`

## skin_editor_desktop::editor_audio::window_recovery::restore::Slot

[源码:5](../../../desktop/host/src/editor_audio\window_recovery/restore.rs#L5) · struct

```rust
pub(super) struct Slot<T> {
    value: Option<T>,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::editor_audio::window_recovery::restore::Attempt::begin`, `skin_editor_desktop::editor_audio::window_recovery::restore::Attempt::fresh`, `skin_editor_desktop::editor_audio::window_recovery::restore::Loan (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl<T> Slot<T>` — [源码:8](../../../desktop/host/src/editor_audio\window_recovery/restore.rs#L8)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn new(value: T) -> Self` | impl<T> Slot<T> | [源码:9](../../../desktop/host/src/editor_audio\window_recovery/restore.rs#L9) |  |
| `pub fn vacant() -> Self` | impl<T> Slot<T> | [源码:12](../../../desktop/host/src/editor_audio\window_recovery/restore.rs#L12) |  |
| `pub fn take(&mut self) -> Option<Loan<'_, T>>` | impl<T> Slot<T> | [源码:15](../../../desktop/host/src/editor_audio\window_recovery/restore.rs#L15) | skin_editor_desktop::editor_audio::window_recovery::restore::Loan |
| `pub fn get(&self) -> Option<&T>` | impl<T> Slot<T> | [源码:22](../../../desktop/host/src/editor_audio\window_recovery/restore.rs#L22) |  |

## skin_editor_desktop::editor_audio::window_recovery::restore::State

[源码:49](../../../desktop/host/src/editor_audio\window_recovery/restore.rs#L49) · struct

```rust
pub(super) struct State {
    pub owner: Option<Owner>,
    pub error: Option<String>,
    pub held_blocked: bool,
    pub prepared: bool,
    pub observer_closed: bool,
    cleanup: Option<Cleanup>,
    framework_retired: bool,
}
```

声明引用：`skin_editor_desktop::editor_audio::window_recovery::restore::Cleanup`, `skin_editor_desktop::editor_audio::window_recovery::restore::Owner`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::editor_audio::window_recovery::restore::Attempt (声明)`, `skin_editor_desktop::editor_audio::window_recovery::restore::Attempt::begin`, `skin_editor_desktop::editor_audio::window_recovery::restore::Attempt::fresh`

实现／约束（显式或 derive，无类继承）：

- `impl State` — [源码:58](../../../desktop/host/src/editor_audio\window_recovery/restore.rs#L58)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn seed(seed: Seed) -> Self` | impl State | [源码:59](../../../desktop/host/src/editor_audio\window_recovery/restore.rs#L59) | skin_editor_desktop::editor_audio::window_recovery::Seed |
| `pub fn snapshot(&self) -> ProjectSnapshot` | impl State | [源码:70](../../../desktop/host/src/editor_audio\window_recovery/restore.rs#L70) |  |
| `pub fn retry_ready(&self) -> bool` | impl State | [源码:77](../../../desktop/host/src/editor_audio\window_recovery/restore.rs#L77) |  |
| `pub fn runtime_retired(&self) -> bool` | impl State | [源码:80](../../../desktop/host/src/editor_audio\window_recovery/restore.rs#L80) |  |
| `pub fn framework_retired(&mut self) -> Result<(), String>` | impl State | [源码:85](../../../desktop/host/src/editor_audio\window_recovery/restore.rs#L85) |  |
| `pub fn open_backend(&self) -> Result<Backend, String>` | impl State | [源码:92](../../../desktop/host/src/editor_audio\window_recovery/restore.rs#L92) | skin_editor_desktop::editor_audio::window_recovery::backend::Backend |
| `pub fn retry_cleanup(&mut self) -> Result<(), String>` | impl State | [源码:100](../../../desktop/host/src/editor_audio\window_recovery/restore.rs#L100) |  |
| `pub(super) fn settle(&mut self, once: bool)` | impl State | [源码:153](../../../desktop/host/src/editor_audio\window_recovery/restore.rs#L153) |  |

## skin_editor_desktop::editor_audio::window_recovery::run

[源码:618](../../../desktop/host/src/editor_audio/window_recovery.rs#L618) · fn

```rust
pub fn run(destination: &str) -> Result<(), String>
```

## skin_editor_desktop::editor_audio::window_recovery::run_fault

[源码:621](../../../desktop/host/src/editor_audio/window_recovery.rs#L621) · fn

```rust
pub fn run_fault(destination: &str, plan: &str) -> Result<(), String>
```

## skin_editor_desktop::editor_audio::window_recovery::size

[源码:440](../../../desktop/host/src/editor_audio/window_recovery.rs#L440) · fn

```rust
fn size(ctx: &Context, want: [f32; 2]) -> bool
```

## skin_editor_desktop::editor_audio::window_recovery::sound::collect

[源码:100](../../../desktop/host/src/editor_audio\window_recovery/sound.rs#L100) · fn

```rust
pub(super) fn collect(
    audio: &Rc<RefCell<EditorAudio>>,
    probes: &mut Probes,
    observer: &Observations,
) -> Result<(), String>
```

声明引用：`skin_editor_desktop::editor_audio::EditorAudio`, `skin_editor_desktop::editor_audio::combined::probes::Probes`, `skin_editor_desktop::editor_audio::gui_verification::Observations`

## skin_editor_desktop::editor_audio::window_recovery::sound::component

[源码:6](../../../desktop/host/src/editor_audio\window_recovery/sound.rs#L6) · fn

```rust
pub(super) fn component() -> AudioComponentId
```

## skin_editor_desktop::editor_audio::window_recovery::sound::install

[源码:9](../../../desktop/host/src/editor_audio\window_recovery/sound.rs#L9) · fn

```rust
pub(super) fn install(
    editor: &mut Editor,
    path: &Path,
    branched_history: bool,
) -> Result<Rc<RefCell<EditorAudio>>, String>
```

声明引用：`skin_editor_desktop::Editor`, `skin_editor_desktop::editor_audio::EditorAudio`

## skin_editor_desktop::editor_audio::window_recovery::sound::reopen

[源码:75](../../../desktop/host/src/editor_audio\window_recovery/sound.rs#L75) · fn

```rust
pub(super) fn reopen(
    editor: &mut Editor,
    audio: &Rc<RefCell<EditorAudio>>,
    path: &Path,
) -> Result<u64, String>
```

声明引用：`skin_editor_desktop::Editor`, `skin_editor_desktop::editor_audio::EditorAudio`

## skin_editor_desktop::editor_audio::window_recovery::sound::rows

[源码:153](../../../desktop/host/src/editor_audio\window_recovery/sound.rs#L153) · fn

```rust
pub(super) fn rows(probes: &Probes) -> String
```

声明引用：`skin_editor_desktop::editor_audio::combined::probes::Probes`

## skin_editor_desktop::editor_audio::window_recovery::sound::trigger

[源码:121](../../../desktop/host/src/editor_audio\window_recovery/sound.rs#L121) · fn

```rust
pub(super) fn trigger(
    editor: &mut Editor,
    audio: &Rc<RefCell<EditorAudio>>,
    probes: &mut Probes,
    lane: Channel,
) -> Result<(), String>
```

声明引用：`skin_editor_audio::Channel`, `skin_editor_desktop::Editor`, `skin_editor_desktop::editor_audio::EditorAudio`, `skin_editor_desktop::editor_audio::combined::probes::Probes`

## skin_editor_desktop::frame_worker::Completion

[源码:27](../../../desktop/host/src/frame_worker.rs#L27) · struct

```rust
pub struct Completion<T> {
    pub job: Job,
    pub started: Instant,
    pub finished: Instant,
    pub result: Result<Option<T>, PreviewError>,
}
```

声明引用：`skin_editor_desktop::frame_worker::Job`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::frame_worker::FrameWorker (声明)`, `skin_editor_desktop::frame_worker::FrameWorker::poll`, `skin_editor_desktop::presentation::NativeHost::accept_prepared`

## skin_editor_desktop::frame_worker::FrameWorker

[源码:35](../../../desktop/host/src/frame_worker.rs#L35) · struct

```rust
pub struct FrameWorker<T> {
    input: Option<SyncSender<Job>>,
    output: Option<Receiver<Completion<T>>>,
    thread: Option<JoinHandle<()>>,
    running: Option<(FrameDescriptor, Arc<AtomicBool>)>,
    received: Option<Completion<T>>,
    join_ok: bool,
}
```

声明引用：`skin_editor_desktop::frame_worker::Completion`, `skin_editor_desktop::frame_worker::Job`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::presentation::NativeHost (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl<T: Send + 'static> FrameWorker<T>` — [源码:43](../../../desktop/host/src/frame_worker.rs#L43)
- `impl<T> Drop for FrameWorker<T>` — [源码:173](../../../desktop/host/src/frame_worker.rs#L173)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn new( prepare: impl Fn(&Job) -> Result<Option<T>, PreviewError> + Send + 'static, wake: impl Fn() + Send + 'static, ) -> Result<Self, PreviewError>` | impl<T: Send + 'static> FrameWorker<T> | [源码:44](../../../desktop/host/src/frame_worker.rs#L44) | skin_editor_desktop::frame_worker::Job |
| `pub fn current(&self) -> Option<FrameDescriptor>` | impl<T: Send + 'static> FrameWorker<T> | [源码:84](../../../desktop/host/src/frame_worker.rs#L84) |  |
| `pub fn cancel(&self)` | impl<T: Send + 'static> FrameWorker<T> | [源码:87](../../../desktop/host/src/frame_worker.rs#L87) |  |
| `pub fn try_close(&mut self) -> bool` | impl<T: Send + 'static> FrameWorker<T> | [源码:95](../../../desktop/host/src/frame_worker.rs#L95) |  |
| `pub fn submit(&mut self, job: Job) -> Result<(), PreviewError>` | impl<T: Send + 'static> FrameWorker<T> | [源码:106](../../../desktop/host/src/frame_worker.rs#L106) | skin_editor_desktop::frame_worker::Job |
| `pub fn poll(&mut self) -> Result<Option<Completion<T>>, PreviewError>` | impl<T: Send + 'static> FrameWorker<T> | [源码:119](../../../desktop/host/src/frame_worker.rs#L119) | skin_editor_desktop::frame_worker::Completion |
| `pub fn wait(&mut self) -> Result<(), PreviewError>` | impl<T: Send + 'static> FrameWorker<T> | [源码:140](../../../desktop/host/src/frame_worker.rs#L140) |  |
| `pub fn close(&mut self)` | impl<T: Send + 'static> FrameWorker<T> | [源码:152](../../../desktop/host/src/frame_worker.rs#L152) |  |
| `pub fn closed(&self) -> bool` | impl<T: Send + 'static> FrameWorker<T> | [源码:164](../../../desktop/host/src/frame_worker.rs#L164) |  |
| `fn drop(&mut self)` | impl<T> Drop for FrameWorker<T> | [源码:174](../../../desktop/host/src/frame_worker.rs#L174) |  |

## skin_editor_desktop::frame_worker::Job

[源码:13](../../../desktop/host/src/frame_worker.rs#L13) · struct

```rust
pub struct Job {
    pub descriptor: FrameDescriptor,
    pub plan: Plan,
    pub queued: Instant,
    pub started_ms: f64,
    pub plan_ms: f64,
    pub raw: Vec<RawBlob>,
    pub cancelled: Arc<AtomicBool>,
}
```

声明引用：`skin_editor_infrastructure::RawBlob`, `skin_editor_rendering::Plan`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::frame_worker::Completion (声明)`, `skin_editor_desktop::frame_worker::FrameWorker (声明)`, `skin_editor_desktop::frame_worker::FrameWorker::new`, `skin_editor_desktop::frame_worker::FrameWorker::submit`, `skin_editor_desktop::frame_worker::decode (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Job` — [源码:22](../../../desktop/host/src/frame_worker.rs#L22)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn is_cancelled(&self) -> bool` | impl Job | [源码:23](../../../desktop/host/src/frame_worker.rs#L23) |  |

## skin_editor_desktop::frame_worker::decode

[源码:186](../../../desktop/host/src/frame_worker.rs#L186) · fn

```rust
pub fn decode(
    job: &Job,
    images: &Images,
) -> Result<Option<(Images, f64)>, PreviewError>
```

声明引用：`skin_editor_desktop::frame_worker::Job`, `skin_editor_infrastructure::Images`, `skin_editor_rendering::Images`

## skin_editor_desktop::framework_trace::Data

[源码:122](../../../desktop/host/src/framework_trace.rs#L122) · struct

```rust
struct Data {
    slots: Vec<Slot>,
    rows: Vec<[u64; 8]>,
    invalid: u64,
    ignored: u64,
    passes: Vec<Pass>,
    sources: Vec<(&'static str, u32)>,
}
```

声明引用：`skin_editor_desktop::framework_trace::Pass`, `skin_editor_desktop::framework_trace::Slot`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::framework_trace::Inner (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Data` — [源码:135](../../../desktop/host/src/framework_trace.rs#L135)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn new() -> Self` | impl Data | [源码:136](../../../desktop/host/src/framework_trace.rs#L136) |  |
| `fn open(&mut self, kind: u8) -> u64` | impl Data | [源码:146](../../../desktop/host/src/framework_trace.rs#L146) |  |
| `fn enter(&mut self, id: u64, at: [u64; 3])` | impl Data | [源码:162](../../../desktop/host/src/framework_trace.rs#L162) |  |
| `fn exit(&mut self, id: u64, at: [u64; 3])` | impl Data | [源码:176](../../../desktop/host/src/framework_trace.rs#L176) |  |
| `fn close(&mut self, id: u64) -> bool` | impl Data | [源码:203](../../../desktop/host/src/framework_trace.rs#L203) |  |

## skin_editor_desktop::framework_trace::Inner

[源码:222](../../../desktop/host/src/framework_trace.rs#L222) · struct

```rust
struct Inner {
    at: Instant,
    stage: AtomicU8,
    data: Mutex<Data>,
}
```

声明引用：`skin_editor_desktop::framework_trace::Data`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::framework_trace::Trace (声明)`

## skin_editor_desktop::framework_trace::LIMIT

[源码:16](../../../desktop/host/src/framework_trace.rs#L16) · constant

```rust
const LIMIT: usize = 32768;
```

## skin_editor_desktop::framework_trace::Pass

[源码:130](../../../desktop/host/src/framework_trace.rs#L130) · struct

```rust
struct Pass {
    row: [u64; 7],
    causes: [usize; 8],
    count: usize,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::framework_trace::Data (声明)`

## skin_editor_desktop::framework_trace::Passes

[源码:294](../../../desktop/host/src/framework_trace.rs#L294) · struct

```rust
pub struct Passes(pub Trace);
```

声明引用：`skin_editor_desktop::framework_trace::Trace`

实现／约束（显式或 derive，无类继承）：

- `impl Plugin for Passes` — [源码:295](../../../desktop/host/src/framework_trace.rs#L295)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn debug_name(&self) -> &'static str` | impl Plugin for Passes | [源码:296](../../../desktop/host/src/framework_trace.rs#L296) |  |
| `fn output_hook(&mut self, ctx: &Context, out: &mut FullOutput)` | impl Plugin for Passes | [源码:299](../../../desktop/host/src/framework_trace.rs#L299) |  |

## skin_editor_desktop::framework_trace::Recorder

[源码:362](../../../desktop/host/src/framework_trace.rs#L362) · struct

```rust
struct Recorder(Trace);
```

声明引用：`skin_editor_desktop::framework_trace::Trace`

实现／约束（显式或 derive，无类继承）：

- `impl Subscriber for Recorder` — [源码:363](../../../desktop/host/src/framework_trace.rs#L363)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn enabled(&self, m: &Metadata<'_>) -> bool` | impl Subscriber for Recorder | [源码:364](../../../desktop/host/src/framework_trace.rs#L364) |  |
| `fn new_span(&self, a: &Attributes<'_>) -> Id` | impl Subscriber for Recorder | [源码:367](../../../desktop/host/src/framework_trace.rs#L367) |  |
| `fn record(&self, _: &Id, _: &Record<'_>)` | impl Subscriber for Recorder | [源码:376](../../../desktop/host/src/framework_trace.rs#L376) |  |
| `fn record_follows_from(&self, _: &Id, _: &Id)` | impl Subscriber for Recorder | [源码:377](../../../desktop/host/src/framework_trace.rs#L377) |  |
| `fn event(&self, _: &Event<'_>)` | impl Subscriber for Recorder | [源码:378](../../../desktop/host/src/framework_trace.rs#L378) |  |
| `fn enter(&self, id: &Id)` | impl Subscriber for Recorder | [源码:379](../../../desktop/host/src/framework_trace.rs#L379) |  |
| `fn exit(&self, id: &Id)` | impl Subscriber for Recorder | [源码:383](../../../desktop/host/src/framework_trace.rs#L383) |  |
| `fn clone_span(&self, id: &Id) -> Id` | impl Subscriber for Recorder | [源码:387](../../../desktop/host/src/framework_trace.rs#L387) |  |
| `fn try_close(&self, id: Id) -> bool` | impl Subscriber for Recorder | [源码:398](../../../desktop/host/src/framework_trace.rs#L398) |  |

## skin_editor_desktop::framework_trace::Scope

[源码:293](../../../desktop/host/src/framework_trace.rs#L293) · struct

```rust
pub struct Scope(Trace, u64);
```

声明引用：`skin_editor_desktop::framework_trace::Trace`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::framework_trace::Trace::handler`, `skin_editor_desktop::framework_trace::Trace::logic`

实现／约束（显式或 derive，无类继承）：

- `impl Drop for Scope` — [源码:345](../../../desktop/host/src/framework_trace.rs#L345)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn drop(&mut self)` | impl Drop for Scope | [源码:346](../../../desktop/host/src/framework_trace.rs#L346) |  |

## skin_editor_desktop::framework_trace::Slot

[源码:117](../../../desktop/host/src/framework_trace.rs#L117) · struct

```rust
struct Slot {
    kind: u8,
    refs: usize,
    entered: Option<[u64; 3]>,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::framework_trace::Data (声明)`

## skin_editor_desktop::framework_trace::Tags

[源码:66](../../../desktop/host/src/framework_trace.rs#L66) · struct

```rust
struct Tags {
    message: Text,
    tag: Text,
}
```

声明引用：`skin_editor_desktop::framework_trace::Text`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::framework_trace::Tags::default`

实现／约束（显式或 derive，无类继承）：

- `impl Default for Tags` — [源码:65](../../../desktop/host/src/framework_trace.rs#L65)
- `impl Visit for Tags` — [源码:70](../../../desktop/host/src/framework_trace.rs#L70)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn default() -> Tags` | impl Default for Tags | [源码:65](../../../desktop/host/src/framework_trace.rs#L65) | skin_editor_desktop::framework_trace::Tags |
| `fn record_debug(&mut self, f: &Field, value: &dyn Debug)` | impl Visit for Tags | [源码:71](../../../desktop/host/src/framework_trace.rs#L71) |  |

## skin_editor_desktop::framework_trace::Text

[源码:36](../../../desktop/host/src/framework_trace.rs#L36) · struct

```rust
struct Text {
    bytes: [u8; 256],
    used: usize,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::framework_trace::Tags (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Text` — [源码:58](../../../desktop/host/src/framework_trace.rs#L58)
- `impl Default for Text` — [源码:40](../../../desktop/host/src/framework_trace.rs#L40)
- `impl Write for Text` — [源码:48](../../../desktop/host/src/framework_trace.rs#L48)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn as_str(&self) -> &str` | impl Text | [源码:59](../../../desktop/host/src/framework_trace.rs#L59) |  |
| `fn default() -> Self` | impl Default for Text | [源码:41](../../../desktop/host/src/framework_trace.rs#L41) |  |
| `fn write_str(&mut self, s: &str) -> Result` | impl Write for Text | [源码:49](../../../desktop/host/src/framework_trace.rs#L49) |  |

## skin_editor_desktop::framework_trace::Trace

[源码:228](../../../desktop/host/src/framework_trace.rs#L228) · struct

```rust
pub struct Trace(Arc<Inner>);
```

声明引用：`skin_editor_desktop::framework_trace::Inner`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::editor_audio::gui_verification::Run (声明)`, `skin_editor_desktop::framework_trace::Passes (声明)`, `skin_editor_desktop::framework_trace::Recorder (声明)`, `skin_editor_desktop::framework_trace::Scope (声明)`, `skin_editor_desktop::framework_trace::Trace::clone`, `skin_editor_desktop::framework_trace::current (声明)`, `skin_editor_desktop::framework_trace::event_loop::Observed (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Trace` — [源码:229](../../../desktop/host/src/framework_trace.rs#L229)
- `impl Trace` — [源码:355](../../../desktop/host/src/framework_trace.rs#L355)
- `impl Clone for Trace` — [源码:227](../../../desktop/host/src/framework_trace.rs#L227)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn new() -> Self` | impl Trace | [源码:230](../../../desktop/host/src/framework_trace.rs#L230) |  |
| `fn stamp(&self) -> [u64; 3]` | impl Trace | [源码:237](../../../desktop/host/src/framework_trace.rs#L237) |  |
| `pub fn stage(&self, v: u8)` | impl Trace | [源码:246](../../../desktop/host/src/framework_trace.rs#L246) |  |
| `pub fn logic(&self) -> Scope` | impl Trace | [源码:249](../../../desktop/host/src/framework_trace.rs#L249) | skin_editor_desktop::framework_trace::Scope |
| `fn report(&self) -> String` | impl Trace | [源码:254](../../../desktop/host/src/framework_trace.rs#L254) |  |
| `fn handler(&self, kind: u8) -> Scope` | impl Trace | [源码:356](../../../desktop/host/src/framework_trace.rs#L356) | skin_editor_desktop::framework_trace::Scope |
| `fn clone(&self) -> Trace` | impl Clone for Trace | [源码:227](../../../desktop/host/src/framework_trace.rs#L227) | skin_editor_desktop::framework_trace::Trace |

## skin_editor_desktop::framework_trace::candidate

[源码:21](../../../desktop/host/src/framework_trace.rs#L21) · fn

```rust
fn candidate(m: &Metadata<'_>) -> bool
```

## skin_editor_desktop::framework_trace::current

[源码:402](../../../desktop/host/src/framework_trace.rs#L402) · fn

```rust
pub fn current() -> Option<Trace>
```

声明引用：`skin_editor_desktop::framework_trace::Trace`

## skin_editor_desktop::framework_trace::event_loop::Observed

[源码:10](../../../desktop/host/src/framework_trace/event_loop.rs#L10) · struct

```rust
struct Observed<T> {
    inner: T,
    trace: Trace,
}
```

声明引用：`skin_editor_desktop::framework_trace::Trace`

实现／约束（显式或 derive，无类继承）：

- `impl<T: ApplicationHandler<UserEvent>> ApplicationHandler<UserEvent> for Observed<T>` — [源码:14](../../../desktop/host/src/framework_trace/event_loop.rs#L14)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn new_events(&mut self, e: &ActiveEventLoop, cause: StartCause)` | impl<T: ApplicationHandler<UserEvent>> ApplicationHandler<UserEvent> for Observed<T> | [源码:17](../../../desktop/host/src/framework_trace/event_loop.rs#L17) |  |
| `fn about_to_wait(&mut self, e: &ActiveEventLoop)` | impl<T: ApplicationHandler<UserEvent>> ApplicationHandler<UserEvent> for Observed<T> | [源码:21](../../../desktop/host/src/framework_trace/event_loop.rs#L21) |  |
| `fn window_event( &mut self, e: &ActiveEventLoop, id: WindowId, event: WindowEvent, )` | impl<T: ApplicationHandler<UserEvent>> ApplicationHandler<UserEvent> for Observed<T> | [源码:25](../../../desktop/host/src/framework_trace/event_loop.rs#L25) |  |
| `fn user_event(&mut self, e: &ActiveEventLoop, event: UserEvent)` | impl<T: ApplicationHandler<UserEvent>> ApplicationHandler<UserEvent> for Observed<T> | [源码:29](../../../desktop/host/src/framework_trace/event_loop.rs#L29) |  |
| `fn device_event( &mut self, e: &ActiveEventLoop, id: DeviceId, event: DeviceEvent, )` | impl<T: ApplicationHandler<UserEvent>> ApplicationHandler<UserEvent> for Observed<T> | [源码:33](../../../desktop/host/src/framework_trace/event_loop.rs#L33) |  |
| `fn resumed(&mut self, e: &ActiveEventLoop)` | impl<T: ApplicationHandler<UserEvent>> ApplicationHandler<UserEvent> for Observed<T> | [源码:37](../../../desktop/host/src/framework_trace/event_loop.rs#L37) |  |
| `fn suspended(&mut self, e: &ActiveEventLoop)` | impl<T: ApplicationHandler<UserEvent>> ApplicationHandler<UserEvent> for Observed<T> | [源码:41](../../../desktop/host/src/framework_trace/event_loop.rs#L41) |  |
| `fn exiting(&mut self, e: &ActiveEventLoop)` | impl<T: ApplicationHandler<UserEvent>> ApplicationHandler<UserEvent> for Observed<T> | [源码:45](../../../desktop/host/src/framework_trace/event_loop.rs#L45) |  |
| `fn memory_warning(&mut self, e: &ActiveEventLoop)` | impl<T: ApplicationHandler<UserEvent>> ApplicationHandler<UserEvent> for Observed<T> | [源码:49](../../../desktop/host/src/framework_trace/event_loop.rs#L49) |  |

## skin_editor_desktop::framework_trace::event_loop::run_native

[源码:54](../../../desktop/host/src/framework_trace/event_loop.rs#L54) · fn

```rust
pub fn run_native(
    name: &str,
    options: NativeOptions,
    creator: AppCreator<'_>,
) -> Result
```

## skin_editor_desktop::framework_trace::kind

[源码:83](../../../desktop/host/src/framework_trace.rs#L83) · fn

```rust
fn kind(name: &str, function: &str, tag: &str) -> u8
```

## skin_editor_desktop::framework_trace::run

[源码:405](../../../desktop/host/src/framework_trace.rs#L405) · fn

```rust
pub fn run(destination: &str, main_work: bool) -> Result<(), String>
```

## skin_editor_desktop::fresh_session

[源码:50](../../../desktop/host/src/main.rs#L50) · fn

```rust
pub(crate) fn fresh_session(images: &Images) -> Result<ProjectSession, String>
```

声明引用：`skin_editor_infrastructure::Images`

## skin_editor_desktop::gpu_observation::apply_memory_hint

[源码:25](../../../desktop/host/src/gpu_observation.rs#L25) · fn

```rust
pub fn apply_memory_hint(
    descriptor: &mut DeviceDescriptor<'_>,
    memory_usage: bool,
)
```

## skin_editor_desktop::gpu_observation::baseline

[源码:83](../../../desktop/host/src/gpu_observation.rs#L83) · fn

```rust
pub fn baseline(
    path: &str,
    large: bool,
    memory_usage: bool,
) -> Result<(), String>
```

## skin_editor_desktop::gpu_observation::configuration

[源码:6](../../../desktop/host/src/gpu_observation.rs#L6) · fn

```rust
pub fn configuration(memory_usage: bool) -> WgpuConfiguration
```

## skin_editor_desktop::gpu_observation::encode

[源码:37](../../../desktop/host/src/gpu_observation.rs#L37) · fn

```rust
fn encode(
    report: Option<AllocatorReport>,
    stage: &str,
    started: Instant,
) -> Result<String, String>
```

## skin_editor_desktop::gpu_observation::snapshot

[源码:33](../../../desktop/host/src/gpu_observation.rs#L33) · fn

```rust
pub fn snapshot(device: &Device, stage: &str) -> Result<String, String>
```

## skin_editor_desktop::gpu_path::FRAMES

[源码:16](../../../desktop/host/src/gpu_path.rs#L16) · constant

```rust
const FRAMES: usize = 512;
```

## skin_editor_desktop::gpu_path::Marker

[源码:68](../../../desktop/host/src/gpu_path.rs#L68) · struct

```rust
struct Marker {
    trace: Arc<Trace>,
    slot: usize,
    first: bool,
}
```

声明引用：`skin_editor_desktop::gpu_path::Trace`

实现／约束（显式或 derive，无类继承）：

- `impl CallbackTrait for Marker` — [源码:73](../../../desktop/host/src/gpu_path.rs#L73)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn prepare( &self, _: &Device, _: &Queue, _: &ScreenDescriptor, _: &mut CommandEncoder, _: &mut CallbackResources, ) -> Vec<CommandBuffer>` | impl CallbackTrait for Marker | [源码:74](../../../desktop/host/src/gpu_path.rs#L74) |  |
| `fn finish_prepare( &self, _: &Device, _: &Queue, encoder: &mut CommandEncoder, _: &mut CallbackResources, ) -> Vec<CommandBuffer>` | impl CallbackTrait for Marker | [源码:87](../../../desktop/host/src/gpu_path.rs#L87) |  |
| `fn paint( &self, _: PaintCallbackInfo, _: &mut RenderPass<'static>, _: &CallbackResources, )` | impl CallbackTrait for Marker | [源码:104](../../../desktop/host/src/gpu_path.rs#L104) |  |

## skin_editor_desktop::gpu_path::Mode

[源码:21](../../../desktop/host/src/gpu_path.rs#L21) · enum

```rust
pub enum Mode {
    Markers,
    Upload,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::gpu_path::Mode::clone`, `skin_editor_desktop::gpu_path::Mode::eq`, `skin_editor_desktop::gpu_path::Probe (声明)`, `skin_editor_desktop::gpu_path::Probe::new`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for Mode` — [源码:20](../../../desktop/host/src/gpu_path.rs#L20)
- `impl PartialEq for Mode` — [源码:20](../../../desktop/host/src/gpu_path.rs#L20)
- `impl Copy for Mode` — [源码:20](../../../desktop/host/src/gpu_path.rs#L20)
- `impl StructuralPartialEq for Mode` — [源码:20](../../../desktop/host/src/gpu_path.rs#L20)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> Mode` | impl Clone for Mode | [源码:20](../../../desktop/host/src/gpu_path.rs#L20) | skin_editor_desktop::gpu_path::Mode |
| `fn eq(&self, other: &Mode) -> bool` | impl PartialEq for Mode | [源码:20](../../../desktop/host/src/gpu_path.rs#L20) | skin_editor_desktop::gpu_path::Mode |

## skin_editor_desktop::gpu_path::NONE

[源码:18](../../../desktop/host/src/gpu_path.rs#L18) · constant

```rust
const NONE: usize = usize::MAX; // 18_446_744_073_709_551_615usize
```

## skin_editor_desktop::gpu_path::ORDINALS

[源码:19](../../../desktop/host/src/gpu_path.rs#L19) · constant

```rust
const ORDINALS: [u64; 4];
```

## skin_editor_desktop::gpu_path::Probe

[源码:152](../../../desktop/host/src/gpu_path.rs#L152) · struct

```rust
pub struct Probe {
    pub stage: u8,
    mode: Mode,
    trace: Arc<Trace>,
    seen: u64,
    last_pass: Option<u64>,
    frames: usize,
    discarded: u64,
    error: Option<String>,
    upload: Option<Upload>,
    released: bool,
}
```

声明引用：`skin_editor_desktop::gpu_path::Mode`, `skin_editor_desktop::gpu_path::Trace`, `skin_editor_desktop::gpu_path::Upload`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::editor_audio::gui_verification::Run (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Probe` — [源码:164](../../../desktop/host/src/gpu_path.rs#L164)
- `impl Plugin for Probe` — [源码:353](../../../desktop/host/src/gpu_path.rs#L353)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn new(mode: Mode, state: Option<&RenderState>) -> Result<Self, String>` | impl Probe | [源码:165](../../../desktop/host/src/gpu_path.rs#L165) | skin_editor_desktop::gpu_path::Mode |
| `pub fn after_tessellation(&self)` | impl Probe | [源码:197](../../../desktop/host/src/gpu_path.rs#L197) |  |
| `fn upload( &mut self, ctx: &Context, output: &FullOutput, pass: u64, ) -> Result<(), String>` | impl Probe | [源码:203](../../../desktop/host/src/gpu_path.rs#L203) |  |
| `pub fn finish(&mut self)` | impl Probe | [源码:289](../../../desktop/host/src/gpu_path.rs#L289) |  |
| `pub fn report(&self) -> String` | impl Probe | [源码:315](../../../desktop/host/src/gpu_path.rs#L315) |  |
| `fn debug_name(&self) -> &'static str` | impl Plugin for Probe | [源码:354](../../../desktop/host/src/gpu_path.rs#L354) |  |
| `fn output_hook(&mut self, ctx: &Context, output: &mut FullOutput)` | impl Plugin for Probe | [源码:357](../../../desktop/host/src/gpu_path.rs#L357) |  |

## skin_editor_desktop::gpu_path::Trace

[源码:25](../../../desktop/host/src/gpu_path.rs#L25) · struct

```rust
struct Trace {
    origin: Instant,
    rows: Vec<[AtomicU64; 8]>,
    done: [AtomicU64; 16],
    active: AtomicUsize,
    invalid: AtomicU64,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::gpu_path::Marker (声明)`, `skin_editor_desktop::gpu_path::Probe (声明)`, `skin_editor_desktop::gpu_path::bracket (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Trace` — [源码:32](../../../desktop/host/src/gpu_path.rs#L32)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn new() -> Self` | impl Trace | [源码:33](../../../desktop/host/src/gpu_path.rs#L33) |  |
| `fn now(&self) -> u64` | impl Trace | [源码:44](../../../desktop/host/src/gpu_path.rs#L44) |  |
| `fn set(&self, slot: usize, field: usize, time: u64)` | impl Trace | [源码:47](../../../desktop/host/src/gpu_path.rs#L47) |  |
| `fn stamp(&self, slot: usize, field: usize)` | impl Trace | [源码:58](../../../desktop/host/src/gpu_path.rs#L58) |  |
| `fn values(&self, count: usize) -> Vec<[u64; 8]>` | impl Trace | [源码:61](../../../desktop/host/src/gpu_path.rs#L61) |  |

## skin_editor_desktop::gpu_path::UPLOADS

[源码:17](../../../desktop/host/src/gpu_path.rs#L17) · constant

```rust
const UPLOADS: usize = 16;
```

## skin_editor_desktop::gpu_path::Upload

[源码:144](../../../desktop/host/src/gpu_path.rs#L144) · struct

```rust
struct Upload {
    renderer: Option<Renderer>,
    device: Device,
    queue: Queue,
    samples: Vec<[u64; 7]>,
    rows: Vec<[u64; 7]>,
    snapshots: Vec<String>,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::gpu_path::Probe (声明)`

## skin_editor_desktop::gpu_path::admit

[源码:129](../../../desktop/host/src/gpu_path.rs#L129) · fn

```rust
fn admit(shapes: &[ClippedShape]) -> bool
```

## skin_editor_desktop::gpu_path::bracket

[源码:113](../../../desktop/host/src/gpu_path.rs#L113) · fn

```rust
fn bracket(output: &mut FullOutput, trace: &Arc<Trace>, slot: usize, rect: Rect)
```

声明引用：`skin_editor_desktop::gpu_path::Trace`

## skin_editor_desktop::graphics_faults::StageObserver

[源码:10](../../../desktop/host/src/graphics_faults.rs#L10) · type

```rust
type StageObserver<'a> = dyn FnMut(&str, Option<&Device>) -> Result<(), String> + 'a;
```

## skin_editor_desktop::graphics_faults::descriptor

[源码:23](../../../desktop/host/src/graphics_faults.rs#L23) · fn

```rust
fn descriptor(memory_usage: bool) -> DeviceDescriptor<'static>
```

## skin_editor_desktop::graphics_faults::error

[源码:11](../../../desktop/host/src/graphics_faults.rs#L11) · fn

```rust
fn error(e: impl Debug) -> String
```

## skin_editor_desktop::graphics_faults::run

[源码:28](../../../desktop/host/src/graphics_faults.rs#L28) · fn

```rust
pub fn run(destination: &str, memory_usage: bool) -> Result<(), String>
```

## skin_editor_desktop::graphics_faults::run_observed

[源码:31](../../../desktop/host/src/graphics_faults.rs#L31) · fn

```rust
pub(crate) fn run_observed(
    destination: &str,
    memory_usage: bool,
    observe: &mut (dyn FnMut(&str, Option<&Device>) -> Result<(), String> + '_),
) -> Result<(), String>
```

## skin_editor_desktop::graphics_faults::wait

[源码:14](../../../desktop/host/src/graphics_faults.rs#L14) · fn

```rust
fn wait(device: &Device) -> Result<(), String>
```

## skin_editor_desktop::gui_baseline::Control

[源码:97](../../../desktop/host/src/gui_baseline.rs#L97) · struct

```rust
pub struct Control {
    pub mode: Mode,
    pub geometry: Option<Geometry>,
    pub changed: bool,
    pub stages: [[u64; 4]; 18],
}
```

声明引用：`skin_editor_desktop::gui_baseline::Geometry`, `skin_editor_desktop::gui_baseline::Mode`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::editor_audio::gui_verification::Run (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Control` — [源码:104](../../../desktop/host/src/gui_baseline.rs#L104)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn new(mode: Mode) -> Self` | impl Control | [源码:105](../../../desktop/host/src/gui_baseline.rs#L105) | skin_editor_desktop::gui_baseline::Mode |
| `pub fn observe(&mut self, geometry: Geometry) -> Result<(), &'static str>` | impl Control | [源码:113](../../../desktop/host/src/gui_baseline.rs#L113) | skin_editor_desktop::gui_baseline::Geometry |
| `pub fn record(&mut self, stage: u8, full: bool, painted: bool)` | impl Control | [源码:121](../../../desktop/host/src/gui_baseline.rs#L121) |  |
| `pub fn report(&self) -> String` | impl Control | [源码:128](../../../desktop/host/src/gui_baseline.rs#L128) |  |

## skin_editor_desktop::gui_baseline::Geometry

[源码:44](../../../desktop/host/src/gui_baseline.rs#L44) · struct

```rust
pub struct Geometry {
    pub window: [f32; 2],
    pub pixels_per_point: f32,
    pub preview: [f32; 4],
    pub size: [u32; 2],
    pub generation: u64,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::gui_baseline::Control (声明)`, `skin_editor_desktop::gui_baseline::Control::observe`, `skin_editor_desktop::gui_baseline::Geometry::clone`, `skin_editor_desktop::gui_baseline::Geometry::eq`

实现／约束（显式或 derive，无类继承）：

- `impl Geometry` — [源码:51](../../../desktop/host/src/gui_baseline.rs#L51)
- `impl Clone for Geometry` — [源码:43](../../../desktop/host/src/gui_baseline.rs#L43)
- `impl Debug for Geometry` — [源码:43](../../../desktop/host/src/gui_baseline.rs#L43)
- `impl PartialEq for Geometry` — [源码:43](../../../desktop/host/src/gui_baseline.rs#L43)
- `impl Copy for Geometry` — [源码:43](../../../desktop/host/src/gui_baseline.rs#L43)
- `impl StructuralPartialEq for Geometry` — [源码:43](../../../desktop/host/src/gui_baseline.rs#L43)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn new( window: Vec2, pixels_per_point: f32, preview: Rect, size: [u32; 2], generation: u64, ) -> Result<Self, &'static str>` | impl Geometry | [源码:52](../../../desktop/host/src/gui_baseline.rs#L52) |  |
| `pub fn rect(self) -> Rect` | impl Geometry | [源码:83](../../../desktop/host/src/gui_baseline.rs#L83) |  |
| `fn report(self) -> String` | impl Geometry | [源码:89](../../../desktop/host/src/gui_baseline.rs#L89) |  |
| `fn clone(&self) -> Geometry` | impl Clone for Geometry | [源码:43](../../../desktop/host/src/gui_baseline.rs#L43) | skin_editor_desktop::gui_baseline::Geometry |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for Geometry | [源码:43](../../../desktop/host/src/gui_baseline.rs#L43) |  |
| `fn eq(&self, other: &Geometry) -> bool` | impl PartialEq for Geometry | [源码:43](../../../desktop/host/src/gui_baseline.rs#L43) | skin_editor_desktop::gui_baseline::Geometry |

## skin_editor_desktop::gui_baseline::Mode

[源码:6](../../../desktop/host/src/gui_baseline.rs#L6) · enum

```rust
pub enum Mode {
    Framework,
    Presentation,
    Editor,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::editor_audio::gui_verification::Run::new`, `skin_editor_desktop::editor_audio::gui_verification::baseline (声明)`, `skin_editor_desktop::editor_audio::gui_verification::run_control (声明)`, `skin_editor_desktop::gui_baseline::Control (声明)`, `skin_editor_desktop::gui_baseline::Control::new`, `skin_editor_desktop::gui_baseline::Mode::clone`, `skin_editor_desktop::gui_baseline::Mode::eq`, `skin_editor_desktop::gui_baseline::viewport (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Mode` — [源码:11](../../../desktop/host/src/gui_baseline.rs#L11)
- `impl Clone for Mode` — [源码:5](../../../desktop/host/src/gui_baseline.rs#L5)
- `impl Debug for Mode` — [源码:5](../../../desktop/host/src/gui_baseline.rs#L5)
- `impl PartialEq for Mode` — [源码:5](../../../desktop/host/src/gui_baseline.rs#L5)
- `impl Copy for Mode` — [源码:5](../../../desktop/host/src/gui_baseline.rs#L5)
- `impl Eq for Mode` — [源码:5](../../../desktop/host/src/gui_baseline.rs#L5)
- `impl StructuralPartialEq for Mode` — [源码:5](../../../desktop/host/src/gui_baseline.rs#L5)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn from_flag(flag: &str) -> Option<Self>` | impl Mode | [源码:12](../../../desktop/host/src/gui_baseline.rs#L12) |  |
| `pub fn name(self) -> &'static str` | impl Mode | [源码:20](../../../desktop/host/src/gui_baseline.rs#L20) |  |
| `pub fn full_ui(self, stage: u8) -> bool` | impl Mode | [源码:27](../../../desktop/host/src/gui_baseline.rs#L27) |  |
| `fn clone(&self) -> Mode` | impl Clone for Mode | [源码:5](../../../desktop/host/src/gui_baseline.rs#L5) | skin_editor_desktop::gui_baseline::Mode |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for Mode | [源码:5](../../../desktop/host/src/gui_baseline.rs#L5) |  |
| `fn eq(&self, other: &Mode) -> bool` | impl PartialEq for Mode | [源码:5](../../../desktop/host/src/gui_baseline.rs#L5) | skin_editor_desktop::gui_baseline::Mode |

## skin_editor_desktop::gui_baseline::viewport

[源码:32](../../../desktop/host/src/gui_baseline.rs#L32) · fn

```rust
pub fn viewport(mode: Option<Mode>) -> ViewportBuilder
```

声明引用：`skin_editor_desktop::gui_baseline::Mode`

## skin_editor_desktop::import

[源码:61](../../../desktop/host/src/main.rs#L61) · fn

```rust
pub(crate) fn import(
    ui: &mut EditorUi,
    session: &mut ProjectSession,
    staged: StagedRaster,
) -> Option<AssetId>
```

声明引用：`skin_editor_ui::EditorUi`

## skin_editor_desktop::import_worker::ImportRequest

[源码:15](../../../desktop/host/src/import_worker.rs#L15) · enum

```rust
pub enum ImportRequest {
    Auto(String),
    Original(String),
    SampleSound,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::import_worker::ImportWorker (声明)`, `skin_editor_desktop::import_worker::ImportWorker::enqueue_request`, `skin_editor_desktop::import_worker::stage (声明)`

## skin_editor_desktop::import_worker::ImportWorker

[源码:9](../../../desktop/host/src/import_worker.rs#L9) · struct

```rust
pub struct ImportWorker {
    input: Option<SyncSender<ImportRequest>>,
    output: Option<Receiver<Result<ImportedAsset, String>>>,
    thread: Option<JoinHandle<()>>,
    join_ok: bool,
}
```

声明引用：`skin_editor_desktop::import_worker::ImportRequest`, `skin_editor_desktop::import_worker::ImportedAsset`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::Editor (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl ImportWorker` — [源码:86](../../../desktop/host/src/import_worker.rs#L86)
- `impl Drop for ImportWorker` — [源码:133](../../../desktop/host/src/import_worker.rs#L133)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn new(images: Images, ctx: Context) -> Result<Self, String>` | impl ImportWorker | [源码:87](../../../desktop/host/src/import_worker.rs#L87) | skin_editor_infrastructure::Images |
| `pub fn enqueue(&self, path: String) -> Result<(), String>` | impl ImportWorker | [源码:109](../../../desktop/host/src/import_worker.rs#L109) |  |
| `pub fn enqueue_request(&self, request: ImportRequest) -> Result<(), String>` | impl ImportWorker | [源码:112](../../../desktop/host/src/import_worker.rs#L112) | skin_editor_desktop::import_worker::ImportRequest |
| `pub fn drain(&self) -> impl Iterator<Item = Result<ImportedAsset, String>> + '_` | impl ImportWorker | [源码:119](../../../desktop/host/src/import_worker.rs#L119) | skin_editor_desktop::import_worker::ImportedAsset |
| `pub fn close(&mut self)` | impl ImportWorker | [源码:122](../../../desktop/host/src/import_worker.rs#L122) |  |
| `pub fn closed(&self) -> bool` | impl ImportWorker | [源码:129](../../../desktop/host/src/import_worker.rs#L129) |  |
| `fn drop(&mut self)` | impl Drop for ImportWorker | [源码:134](../../../desktop/host/src/import_worker.rs#L134) |  |

## skin_editor_desktop::import_worker::ImportedAsset

[源码:20](../../../desktop/host/src/import_worker.rs#L20) · struct

```rust
pub struct ImportedAsset {
    pub blob: BlobRef,
    pub kind: AssetKind,
    pub label: String,
    pub lease: Arc<dyn BlobLease>,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::import_worker::ImportWorker (声明)`, `skin_editor_desktop::import_worker::ImportWorker::drain`, `skin_editor_desktop::import_worker::stage (声明)`

## skin_editor_desktop::import_worker::stage

[源码:26](../../../desktop/host/src/import_worker.rs#L26) · fn

```rust
pub(crate) fn stage(
    images: &mut Images,
    request: ImportRequest,
) -> Result<ImportedAsset, String>
```

声明引用：`skin_editor_desktop::import_worker::ImportRequest`, `skin_editor_desktop::import_worker::ImportedAsset`, `skin_editor_infrastructure::Images`

## skin_editor_desktop::load_verification::consume

[源码:127](../../../desktop/host/src/load_verification.rs#L127) · fn

```rust
pub(crate) fn consume(
    output: &OutputControl,
    rate: u32,
    millis: u64,
) -> Result<(), String>
```

声明引用：`skin_editor_infrastructure::OutputControl`

## skin_editor_desktop::load_verification::control

[源码:40](../../../desktop/host/src/load_verification.rs#L40) · fn

```rust
fn control(client: &mut AudioSupplyClient, running: bool) -> Result<(), String>
```

声明引用：`skin_editor_infrastructure::AudioSupplyClient`

## skin_editor_desktop::load_verification::outcome

[源码:28](../../../desktop/host/src/load_verification.rs#L28) · fn

```rust
pub(crate) fn outcome(
    client: &mut AudioSupplyClient,
    id: u64,
) -> Result<SupplyOutcome, String>
```

声明引用：`skin_editor_infrastructure::AudioSupplyClient`, `skin_editor_infrastructure::SupplyOutcome`

## skin_editor_desktop::load_verification::phase

[源码:11](../../../desktop/host/src/load_verification.rs#L11) · fn

```rust
pub(crate) fn phase(name: &str)
```

## skin_editor_desktop::load_verification::run

[源码:162](../../../desktop/host/src/load_verification.rs#L162) · fn

```rust
pub fn run(destination: &str) -> Result<(), String>
```

## skin_editor_desktop::load_verification::startup

[源码:49](../../../desktop/host/src/load_verification.rs#L49) · fn

```rust
pub fn startup(destination: &str) -> Result<(), String>
```

## skin_editor_desktop::load_verification::stop

[源码:115](../../../desktop/host/src/load_verification.rs#L115) · fn

```rust
pub(crate) fn stop(
    client: &mut AudioSupplyClient,
    output: &OutputControl,
) -> Result<(), String>
```

声明引用：`skin_editor_infrastructure::AudioSupplyClient`, `skin_editor_infrastructure::OutputControl`

## skin_editor_desktop::load_verification::wait

[源码:15](../../../desktop/host/src/load_verification.rs#L15) · fn

```rust
pub(crate) fn wait(
    output: &OutputControl,
    condition: impl FnMut() -> bool,
) -> Result<(), String>
```

声明引用：`skin_editor_infrastructure::OutputControl`

## skin_editor_desktop::load_verification::wav

[源码:136](../../../desktop/host/src/load_verification.rs#L136) · fn

```rust
pub(crate) fn wav(index: u32) -> Vec<u8> ⓘ
```

## skin_editor_desktop::main

[源码:378](../../../desktop/host/src/main.rs#L378) · fn

```rust
pub(crate) fn main() -> Result<(), Box<dyn Error>>
```

## skin_editor_desktop::main_work::DIAGNOSTIC

[源码:8](../../../desktop/host/src/main_work.rs#L8) · constant

```rust
pub const DIAGNOSTIC: u8 = 3;
```

## skin_editor_desktop::main_work::FRAMEWORK

[源码:5](../../../desktop/host/src/main_work.rs#L5) · constant

```rust
pub const FRAMEWORK: u8 = 0;
```

## skin_editor_desktop::main_work::GRAPHICS_POLL

[源码:11](../../../desktop/host/src/main_work.rs#L11) · constant

```rust
pub const GRAPHICS_POLL: u8 = 6;
```

## skin_editor_desktop::main_work::LIMIT

[源码:14](../../../desktop/host/src/main_work.rs#L14) · constant

```rust
const LIMIT: usize = 32768;
```

## skin_editor_desktop::main_work::LOGIC

[源码:6](../../../desktop/host/src/main_work.rs#L6) · constant

```rust
pub const LOGIC: u8 = 1;
```

## skin_editor_desktop::main_work::MainWork

[源码:39](../../../desktop/host/src/main_work.rs#L39) · struct

```rust
pub struct MainWork {
    origin: Instant,
    rows: Vec<[u64; 3]>,
    invalid: u64,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::Editor (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl MainWork` — [源码:45](../../../desktop/host/src/main_work.rs#L45)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn new() -> Self` | impl MainWork | [源码:46](../../../desktop/host/src/main_work.rs#L46) |  |
| `pub fn edge(&mut self, category: u8)` | impl MainWork | [源码:53](../../../desktop/host/src/main_work.rs#L53) |  |
| `fn record(&mut self, row: [u64; 3])` | impl MainWork | [源码:70](../../../desktop/host/src/main_work.rs#L70) |  |
| `pub fn report(&self) -> String` | impl MainWork | [源码:83](../../../desktop/host/src/main_work.rs#L83) |  |

## skin_editor_desktop::main_work::PRESENT_AND_EVENTS

[源码:9](../../../desktop/host/src/main_work.rs#L9) · constant

```rust
pub const PRESENT_AND_EVENTS: u8 = 4;
```

## skin_editor_desktop::main_work::PREVIEW_REQUEST

[源码:12](../../../desktop/host/src/main_work.rs#L12) · constant

```rust
pub const PREVIEW_REQUEST: u8 = 7;
```

## skin_editor_desktop::main_work::UI

[源码:7](../../../desktop/host/src/main_work.rs#L7) · constant

```rust
pub const UI: u8 = 2;
```

## skin_editor_desktop::main_work::UI_DISPATCH

[源码:13](../../../desktop/host/src/main_work.rs#L13) · constant

```rust
pub const UI_DISPATCH: u8 = 8;
```

## skin_editor_desktop::main_work::WIDGETS

[源码:10](../../../desktop/host/src/main_work.rs#L10) · constant

```rust
pub const WIDGETS: u8 = 5;
```

## skin_editor_desktop::main_work::ui_category

[源码:16](../../../desktop/host/src/main_work.rs#L16) · fn

```rust
pub fn ui_category(phase: UiPhase) -> u8
```

声明引用：`skin_editor_ui::UiPhase`

## skin_editor_desktop::mesh_replay::ATLAS_BYTES

[源码:12](../../../desktop/host/src/mesh_replay.rs#L12) · constant

```rust
const ATLAS_BYTES: usize = _; // 4_194_304usize
```

## skin_editor_desktop::mesh_replay::Counts

[源码:26](../../../desktop/host/src/mesh_replay.rs#L26) · struct

```rust
struct Counts {
    nodes: usize,
    texts: usize,
    vertices: usize,
    indices: usize,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::mesh_replay::Counts::default`, `skin_editor_desktop::mesh_replay::admit (声明)`, `skin_editor_desktop::mesh_replay::partition (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Default for Counts` — [源码:25](../../../desktop/host/src/mesh_replay.rs#L25)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn default() -> Counts` | impl Default for Counts | [源码:25](../../../desktop/host/src/mesh_replay.rs#L25) | skin_editor_desktop::mesh_replay::Counts |

## skin_editor_desktop::mesh_replay::INDICES

[源码:10](../../../desktop/host/src/mesh_replay.rs#L10) · constant

```rust
const INDICES: usize = 131072;
```

## skin_editor_desktop::mesh_replay::NODES

[源码:11](../../../desktop/host/src/mesh_replay.rs#L11) · constant

```rust
const NODES: usize = 2048;
```

## skin_editor_desktop::mesh_replay::ORDINALS

[源码:13](../../../desktop/host/src/mesh_replay.rs#L13) · constant

```rust
pub const ORDINALS: [u64; 4];
```

## skin_editor_desktop::mesh_replay::PLAN

[源码:14](../../../desktop/host/src/mesh_replay.rs#L14) · constant

```rust
pub const PLAN: [usize; 7];
```

## skin_editor_desktop::mesh_replay::Packed

[源码:125](../../../desktop/host/src/mesh_replay.rs#L125) · struct

```rust
struct Packed {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
}
```

实现／约束（显式或 derive，无类继承）：

- `impl Packed` — [源码:129](../../../desktop/host/src/mesh_replay.rs#L129)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn new(v: usize, i: usize) -> Self` | impl Packed | [源码:130](../../../desktop/host/src/mesh_replay.rs#L130) |  |
| `fn copy(&mut self, source: &[ClippedPrimitive]) -> Result<(), &'static str>` | impl Packed | [源码:136](../../../desktop/host/src/mesh_replay.rs#L136) |  |

## skin_editor_desktop::mesh_replay::REPEATS

[源码:15](../../../desktop/host/src/mesh_replay.rs#L15) · constant

```rust
const REPEATS: usize = 4;
```

## skin_editor_desktop::mesh_replay::Replay

[源码:154](../../../desktop/host/src/mesh_replay.rs#L154) · struct

```rust
pub struct Replay {
    pub stage: u8,
    seen: u64,
    last_pass: Option<u64>,
    rows: Vec<[u64; 17]>,
    samples: Vec<[u64; 11]>,
    invalid: u64,
    scratch_active: bool,
    error: Option<&'static str>,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::editor_audio::gui_verification::Run (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Replay` — [源码:164](../../../desktop/host/src/mesh_replay.rs#L164)
- `impl Plugin for Replay` — [源码:322](../../../desktop/host/src/mesh_replay.rs#L322)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn new() -> Self` | impl Replay | [源码:165](../../../desktop/host/src/mesh_replay.rs#L165) |  |
| `fn capture( &mut self, ctx: &Context, output: &FullOutput, pass: u64, ) -> Result<(), &'static str>` | impl Replay | [源码:177](../../../desktop/host/src/mesh_replay.rs#L177) |  |
| `pub fn report(&self) -> String` | impl Replay | [源码:308](../../../desktop/host/src/mesh_replay.rs#L308) |  |
| `fn debug_name(&self) -> &'static str` | impl Plugin for Replay | [源码:323](../../../desktop/host/src/mesh_replay.rs#L323) |  |
| `fn output_hook(&mut self, ctx: &Context, output: &mut FullOutput)` | impl Plugin for Replay | [源码:326](../../../desktop/host/src/mesh_replay.rs#L326) |  |

## skin_editor_desktop::mesh_replay::VERTICES

[源码:9](../../../desktop/host/src/mesh_replay.rs#L9) · constant

```rust
const VERTICES: usize = 32768;
```

## skin_editor_desktop::mesh_replay::admit

[源码:32](../../../desktop/host/src/mesh_replay.rs#L32) · fn

```rust
fn admit(
    shape: &Shape,
    depth: usize,
    c: &mut Counts,
) -> Result<(), &'static str>
```

声明引用：`skin_editor_desktop::mesh_replay::Counts`

## skin_editor_desktop::mesh_replay::admit_atlas

[源码:16](../../../desktop/host/src/mesh_replay.rs#L16) · fn

```rust
fn admit_atlas(size: [usize; 2]) -> bool
```

## skin_editor_desktop::mesh_replay::counts

[源码:96](../../../desktop/host/src/mesh_replay.rs#L96) · fn

```rust
fn counts(primitives: &[ClippedPrimitive]) -> Result<[usize; 3], &'static str>
```

## skin_editor_desktop::mesh_replay::equal

[源码:112](../../../desktop/host/src/mesh_replay.rs#L112) · fn

```rust
fn equal(a: &[ClippedPrimitive], b: &[ClippedPrimitive]) -> bool
```

## skin_editor_desktop::mesh_replay::partition

[源码:74](../../../desktop/host/src/mesh_replay.rs#L74) · fn

```rust
fn partition(
    source: &[ClippedShape],
) -> Result<([Vec<ClippedShape>; 4], Counts), &'static str>
```

声明引用：`skin_editor_desktop::mesh_replay::Counts`

## skin_editor_desktop::mesh_replay::select

[源码:62](../../../desktop/host/src/mesh_replay.rs#L62) · fn

```rust
fn select(shape: &Shape, text: bool) -> Option<Shape>
```

## skin_editor_desktop::presentation::DISPLAY

[源码:808](../../../desktop/host/src/presentation.rs#L808) · constant

```rust
const DISPLAY: &str = r#"
@group(0) @binding(0) var source:texture_2d<f32>;
struct Vertex { @builtin(position) position:vec4<f32>, @location(0) uv:vec2<f32> }
@vertex fn vs_main(@builtin(vertex_index) i:u32)->Vertex {
 let p=array<vec2<f32>,6>(vec2(-1.0,1.0),vec2(1.0,1.0),vec2(-1.0,-1.0),vec2(-1.0,-1.0),vec2(1.0,1.0),vec2(1.0,-1.0));
 var out:Vertex;out.position=vec4(p[i],0.0,1.0);out.uv=(p[i]*vec2(1.0,-1.0)+vec2(1.0))/2.0;return out;
}
@fragment fn fs_main(v:Vertex)->@location(0) vec4<f32> {
 let size=textureDimensions(source);let p=clamp(vec2<i32>(v.uv*vec2<f32>(size)),vec2(0),vec2<i32>(size)-vec2(1));var color=textureLoad(source,p,0);
 if TARGET_SRGB {color=vec4(select(pow((color.rgb+vec3(0.055))/1.055,vec3(2.4)),color.rgb/12.92,color.rgb<=vec3(0.04045)),color.a);}
 return color;
}
"#;
```

## skin_editor_desktop::presentation::DeviceLoss

[源码:51](../../../desktop/host/src/presentation.rs#L51) · struct

```rust
pub(crate) struct DeviceLoss(Arc<AtomicBool>);
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::editor_audio::window_recovery::backend::Backend (声明)`, `skin_editor_desktop::editor_audio::window_recovery::init_failure::Recovered (声明)`, `skin_editor_desktop::presentation::DeviceLoss::clone`, `skin_editor_desktop::presentation::DeviceLoss::default`, `skin_editor_desktop::presentation::Draw (声明)`, `skin_editor_desktop::presentation::NativeHost (声明)`, `skin_editor_desktop::presentation::NativeHost::bind_loss`, `skin_editor_desktop::presentation::NativeHost::loss_signal`

实现／约束（显式或 derive，无类继承）：

- `impl DeviceLoss` — [源码:52](../../../desktop/host/src/presentation.rs#L52)
- `impl Clone for DeviceLoss` — [源码:50](../../../desktop/host/src/presentation.rs#L50)
- `impl Default for DeviceLoss` — [源码:50](../../../desktop/host/src/presentation.rs#L50)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn report(&self)` | impl DeviceLoss | [源码:53](../../../desktop/host/src/presentation.rs#L53) |  |
| `fn is_lost(&self) -> bool` | impl DeviceLoss | [源码:56](../../../desktop/host/src/presentation.rs#L56) |  |
| `fn clone(&self) -> DeviceLoss` | impl Clone for DeviceLoss | [源码:50](../../../desktop/host/src/presentation.rs#L50) | skin_editor_desktop::presentation::DeviceLoss |
| `fn default() -> DeviceLoss` | impl Default for DeviceLoss | [源码:50](../../../desktop/host/src/presentation.rs#L50) | skin_editor_desktop::presentation::DeviceLoss |

## skin_editor_desktop::presentation::Draw

[源码:688](../../../desktop/host/src/presentation.rs#L688) · struct

```rust
struct Draw {
    target: Arc<NativeTarget>,
    pipeline: Arc<RenderPipeline>,
    pending: Arc<AtomicBool>,
    ctx: Context,
    loss: DeviceLoss,
}
```

声明引用：`skin_editor_desktop::presentation::DeviceLoss`, `skin_editor_desktop::presentation::NativeTarget`

实现／约束（显式或 derive，无类继承）：

- `impl CallbackTrait for Draw` — [源码:695](../../../desktop/host/src/presentation.rs#L695)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn prepare( &self, _: &Device, _: &Queue, _: &ScreenDescriptor, encoder: &mut CommandEncoder, _: &mut CallbackResources, ) -> Vec<CommandBuffer>` | impl CallbackTrait for Draw | [源码:696](../../../desktop/host/src/presentation.rs#L696) |  |
| `fn paint( &self, _: PaintCallbackInfo, pass: &mut RenderPass<'static>, _: &CallbackResources, )` | impl CallbackTrait for Draw | [源码:720](../../../desktop/host/src/presentation.rs#L720) |  |

## skin_editor_desktop::presentation::Frame

[源码:36](../../../desktop/host/src/presentation.rs#L36) · struct

```rust
struct Frame {
    descriptor: FrameDescriptor,
    view: Affine2,
    target: Arc<NativeTarget>,
    trace: Option<usize>,
    publishable: bool,
}
```

声明引用：`skin_editor_desktop::presentation::NativeTarget`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::presentation::NativeHost (声明)`, `skin_editor_desktop::presentation::NativeHost::retire`

## skin_editor_desktop::presentation::NativeHost

[源码:60](../../../desktop/host/src/presentation.rs#L60) · struct

```rust
pub struct NativeHost {
    pub device: Device,
    pub queue: Queue,
    composer: Arc<GpuComposer>,
    worker: FrameWorker<Prepared>,
    layout: BindGroupLayout,
    pipeline: Arc<RenderPipeline>,
    active: Option<Frame>,
    candidate: Option<Frame>,
    retired: Vec<Weak<NativeTarget>>,
    submitted: Vec<Arc<NativeTarget>>,
    desired: Option<FrameDescriptor>,
    next: u64,
    next_allocation: u64,
    images: Images,
    thumbs: BTreeMap<BlobRef, TextureHandle>,
    ctx: Context,
    pending: Arc<AtomicBool>,
    pub allocations: u64,
    pub publications: u64,
    trace_started: Option<Instant>,
    pub traces: Vec<PreparationTrace>,
    pub trace_overflow: bool,
    shutdown_ok: bool,
    loss: DeviceLoss,
    loss_binding_open: bool,
    lost_cleaned: bool,
    upload: SourceUpload,
}
```

声明引用：`skin_editor_desktop::frame_worker::FrameWorker`, `skin_editor_desktop::presentation::DeviceLoss`, `skin_editor_desktop::presentation::Frame`, `skin_editor_desktop::presentation::NativeTarget`, `skin_editor_desktop::presentation::PreparationTrace`, `skin_editor_desktop::presentation::Prepared`, `skin_editor_infrastructure::Images`, `skin_editor_rendering::GpuComposer`, `skin_editor_rendering::SourceUpload`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::Editor (声明)`, `skin_editor_desktop::presentation::RebuildSeed::build`, `skin_editor_desktop::presentation::RebuildSeed::build_retained`, `skin_editor_desktop::verification::compare (声明)`, `skin_editor_desktop::verification::frame (声明)`, `skin_editor_desktop::verification::readback (声明)`, `skin_editor_desktop::verification::readback_observed (声明)`, `skin_editor_desktop::verification::ui_image (声明)`, `skin_editor_desktop::verification::verify_source_buffer_reuse (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl NativeHost` — [源码:149](../../../desktop/host/src/presentation.rs#L149)
- `impl NativeHost` — [源码:595](../../../desktop/host/src/presentation.rs#L595)
- `impl Presenter for NativeHost` — [源码:734](../../../desktop/host/src/presentation.rs#L734)
- `impl PreviewPort for NativeHost` — [源码:500](../../../desktop/host/src/presentation.rs#L500)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub(crate) fn bind_loss(&mut self, loss: DeviceLoss) -> Result<(), PreviewError>` | impl NativeHost | [源码:152](../../../desktop/host/src/presentation.rs#L152) | skin_editor_desktop::presentation::DeviceLoss |
| `pub fn with_upload( device: Device, queue: Queue, format: TextureFormat, images: Images, ctx: Context, upload: SourceUpload, ) -> Result<Self, PreviewError>` | impl NativeHost | [源码:162](../../../desktop/host/src/presentation.rs#L162) | skin_editor_infrastructure::Images, skin_editor_rendering::SourceUpload |
| `pub fn enable_trace(&mut self)` | impl NativeHost | [源码:287](../../../desktop/host/src/presentation.rs#L287) |  |
| `fn retire(&mut self, frame: Frame)` | impl NativeHost | [源码:294](../../../desktop/host/src/presentation.rs#L294) | skin_editor_desktop::presentation::Frame |
| `pub fn wait_preparation(&mut self) -> Result<(), PreviewError>` | impl NativeHost | [源码:300](../../../desktop/host/src/presentation.rs#L300) |  |
| `pub fn poll(&mut self) -> Result<(), PreviewError>` | impl NativeHost | [源码:303](../../../desktop/host/src/presentation.rs#L303) |  |
| `fn poll_live(&mut self) -> Result<(), PreviewError>` | impl NativeHost | [源码:319](../../../desktop/host/src/presentation.rs#L319) |  |
| `pub fn loss_signal(&self) -> DeviceLoss` | impl NativeHost | [源码:367](../../../desktop/host/src/presentation.rs#L367) | skin_editor_desktop::presentation::DeviceLoss |
| `pub fn is_lost(&self) -> bool` | impl NativeHost | [源码:370](../../../desktop/host/src/presentation.rs#L370) |  |
| `pub fn invalidate_source(&mut self)` | impl NativeHost | [源码:375](../../../desktop/host/src/presentation.rs#L375) |  |
| `fn clean_lost_device(&mut self)` | impl NativeHost | [源码:387](../../../desktop/host/src/presentation.rs#L387) |  |
| `pub fn into_rebuild_seed(self) -> Result<RebuildSeed, Box<Self>>` | impl NativeHost | [源码:416](../../../desktop/host/src/presentation.rs#L416) | skin_editor_desktop::presentation::RebuildSeed |
| `pub fn needs_work(&self) -> bool` | impl NativeHost | [源码:433](../../../desktop/host/src/presentation.rs#L433) |  |
| `pub fn preparation_closed(&self) -> bool` | impl NativeHost | [源码:436](../../../desktop/host/src/presentation.rs#L436) |  |
| `pub fn preparation_pending(&self) -> bool` | impl NativeHost | [源码:439](../../../desktop/host/src/presentation.rs#L439) |  |
| `pub fn shutdown_succeeded(&self) -> bool` | impl NativeHost | [源码:442](../../../desktop/host/src/presentation.rs#L442) |  |
| `pub fn shutdown(&mut self)` | impl NativeHost | [源码:445](../../../desktop/host/src/presentation.rs#L445) |  |
| `pub fn active_texture(&self) -> Option<&Texture>` | impl NativeHost | [源码:473](../../../desktop/host/src/presentation.rs#L473) |  |
| `pub fn source_usage(&self) -> (u64, usize)` | impl NativeHost | [源码:479](../../../desktop/host/src/presentation.rs#L479) |  |
| `pub fn staging_usage(&self) -> StagingUsage` | impl NativeHost | [源码:482](../../../desktop/host/src/presentation.rs#L482) | skin_editor_rendering::StagingUsage |
| `pub fn source_allocations(&self) -> u64` | impl NativeHost | [源码:485](../../../desktop/host/src/presentation.rs#L485) |  |
| `pub fn live_targets(&self) -> usize` | impl NativeHost | [源码:488](../../../desktop/host/src/presentation.rs#L488) |  |
| `fn accept_prepared( &mut self, done: Completion<Prepared>, ) -> Result<(), PreviewError>` | impl NativeHost | [源码:596](../../../desktop/host/src/presentation.rs#L596) | skin_editor_desktop::frame_worker::Completion, skin_editor_desktop::presentation::Prepared |
| `fn current(&self) -> Option<PresentedView>` | impl Presenter for NativeHost | [源码:735](../../../desktop/host/src/presentation.rs#L735) | skin_editor_ui::PresentedView |
| `fn paint(&mut self, ui: &mut Ui, key: FrameKey, rect: Rect)` | impl Presenter for NativeHost | [源码:745](../../../desktop/host/src/presentation.rs#L745) |  |
| `fn thumbnail(&mut self, ui: &mut Ui, blob: BlobRef)` | impl Presenter for NativeHost | [源码:762](../../../desktop/host/src/presentation.rs#L762) |  |
| `fn request( &mut self, project: &SkinProject, source: FrameSource, ) -> Result<FrameDescriptor, PreviewError>` | impl PreviewPort for NativeHost | [源码:501](../../../desktop/host/src/presentation.rs#L501) |  |

## skin_editor_desktop::presentation::NativeTarget

[源码:31](../../../desktop/host/src/presentation.rs#L31) · struct

```rust
struct NativeTarget {
    allocation: u64,
    image: GpuImage,
    group: BindGroup,
}
```

声明引用：`skin_editor_rendering::GpuImage`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::presentation::Draw (声明)`, `skin_editor_desktop::presentation::Frame (声明)`, `skin_editor_desktop::presentation::NativeHost (声明)`

## skin_editor_desktop::presentation::PreparationTrace

[源码:16](../../../desktop/host/src/presentation.rs#L16) · struct

```rust
pub struct PreparationTrace {
    pub source: FrameSource,
    pub started_ms: f64,
    pub plan_ms: f64,
    pub decode_ms: f64,
    pub worker_started_ms: f64,
    pub worker_finished_ms: f64,
    pub queue_ms: f64,
    pub received_ms: f64,
    pub gpu_done_ms: Option<f64>,
    gpu_done: Option<Arc<OnceLock<Instant>>>,
    pub gpu: GpuPreparation,
    pub published_ms: Option<f64>,
}
```

声明引用：`skin_editor_rendering::GpuPreparation`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::presentation::NativeHost (声明)`

## skin_editor_desktop::presentation::Prepared

[源码:43](../../../desktop/host/src/presentation.rs#L43) · struct

```rust
struct Prepared {
    image: GpuImage,
    view: Affine2,
    decode_ms: f64,
}
```

声明引用：`skin_editor_rendering::GpuImage`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::presentation::NativeHost (声明)`, `skin_editor_desktop::presentation::NativeHost::accept_prepared`

## skin_editor_desktop::presentation::RebuildSeed

[源码:90](../../../desktop/host/src/presentation.rs#L90) · struct

```rust
pub(crate) struct RebuildSeed {
    images: Images,
    upload: SourceUpload,
    next: u64,
    next_allocation: u64,
}
```

声明引用：`skin_editor_infrastructure::Images`, `skin_editor_rendering::SourceUpload`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::editor_audio::window_recovery::Seed (声明)`, `skin_editor_desktop::presentation::NativeHost::into_rebuild_seed`

实现／约束（显式或 derive，无类继承）：

- `impl RebuildSeed` — [源码:96](../../../desktop/host/src/presentation.rs#L96)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn build( self, device: Device, queue: Queue, format: TextureFormat, ctx: Context, ) -> Result<NativeHost, PreviewError>` | impl RebuildSeed | [源码:97](../../../desktop/host/src/presentation.rs#L97) | skin_editor_desktop::presentation::NativeHost |
| `pub(crate) fn build_retained( &self, device: Device, queue: Queue, format: TextureFormat, ctx: Context, ) -> Result<NativeHost, PreviewError>` | impl RebuildSeed | [源码:108](../../../desktop/host/src/presentation.rs#L108) | skin_editor_desktop::presentation::NativeHost |

## skin_editor_desktop::presentation::candidate_current

[源码:130](../../../desktop/host/src/presentation.rs#L130) · fn

```rust
pub(crate) fn candidate_current(
    desired: Option<FrameSource>,
    source: FrameSource,
    cancelled: bool,
) -> bool
```

## skin_editor_desktop::presentation::material_equal

[源码:122](../../../desktop/host/src/presentation.rs#L122) · fn

```rust
fn material_equal(a: FrameSource, b: FrameSource) -> bool
```

## skin_editor_desktop::presentation::ready_descriptor

[源码:137](../../../desktop/host/src/presentation.rs#L137) · fn

```rust
fn ready_descriptor(
    desired: Option<FrameDescriptor>,
    source: FrameSource,
    publishable: bool,
) -> Option<FrameDescriptor>
```

## skin_editor_desktop::product_policy::Policy

[源码:6](../../../desktop/host/src/product_policy.rs#L6) · struct

```rust
pub struct Policy {
    pub separated_audio: bool,
    pub memory_usage: bool,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::product_policy::Policy::clone`, `skin_editor_desktop::product_policy::Policy::eq`

实现／约束（显式或 derive，无类继承）：

- `impl Policy` — [源码:10](../../../desktop/host/src/product_policy.rs#L10)
- `impl Clone for Policy` — [源码:5](../../../desktop/host/src/product_policy.rs#L5)
- `impl Debug for Policy` — [源码:5](../../../desktop/host/src/product_policy.rs#L5)
- `impl PartialEq for Policy` — [源码:5](../../../desktop/host/src/product_policy.rs#L5)
- `impl Copy for Policy` — [源码:5](../../../desktop/host/src/product_policy.rs#L5)
- `impl Eq for Policy` — [源码:5](../../../desktop/host/src/product_policy.rs#L5)
- `impl StructuralPartialEq for Policy` — [源码:5](../../../desktop/host/src/product_policy.rs#L5)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `const fn for_windows(windows: bool) -> Self` | impl Policy | [源码:11](../../../desktop/host/src/product_policy.rs#L11) |  |
| `pub fn audio(self, store: BlobStore) -> EditorAudio` | impl Policy | [源码:19](../../../desktop/host/src/product_policy.rs#L19) | skin_editor_desktop::editor_audio::EditorAudio, skin_editor_infrastructure::BlobStore |
| `pub fn graphics(self) -> WgpuConfiguration` | impl Policy | [源码:26](../../../desktop/host/src/product_policy.rs#L26) |  |
| `fn clone(&self) -> Policy` | impl Clone for Policy | [源码:5](../../../desktop/host/src/product_policy.rs#L5) | skin_editor_desktop::product_policy::Policy |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for Policy | [源码:5](../../../desktop/host/src/product_policy.rs#L5) |  |
| `fn eq(&self, other: &Policy) -> bool` | impl PartialEq for Policy | [源码:5](../../../desktop/host/src/product_policy.rs#L5) | skin_editor_desktop::product_policy::Policy |

## skin_editor_desktop::rebuild_stages::ack_matches

[源码:9](../../../desktop/host/src/rebuild_stages.rs#L9) · fn

```rust
fn ack_matches(bytes: &[u8], sequence: u32) -> bool
```

## skin_editor_desktop::rebuild_stages::run

[源码:13](../../../desktop/host/src/rebuild_stages.rs#L13) · fn

```rust
pub fn run(destination: &str, memory_usage: bool) -> Result<(), String>
```

## skin_editor_desktop::repaint_deadline::RepaintCycle

[源码:30](../../../desktop/host/src/repaint_deadline.rs#L30) · struct

```rust
pub struct RepaintCycle {
    started: Instant,
    due: Option<Duration>,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::repaint_deadline::RepaintPass::publish_at`

实现／约束（显式或 derive，无类继承）：

- `impl RepaintCycle` — [源码:34](../../../desktop/host/src/repaint_deadline.rs#L34)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn new() -> Self` | impl RepaintCycle | [源码:35](../../../desktop/host/src/repaint_deadline.rs#L35) |  |
| `pub fn after(&mut self, millis: u64)` | impl RepaintCycle | [源码:41](../../../desktop/host/src/repaint_deadline.rs#L41) |  |
| `fn offer_at(&mut self, elapsed: Duration, delay: Duration)` | impl RepaintCycle | [源码:44](../../../desktop/host/src/repaint_deadline.rs#L44) |  |
| `pub fn publish(self, ctx: &Context, pass: &mut RepaintPass)` | impl RepaintCycle | [源码:52](../../../desktop/host/src/repaint_deadline.rs#L52) | skin_editor_desktop::repaint_deadline::RepaintPass |

## skin_editor_desktop::repaint_deadline::RepaintPass

[源码:6](../../../desktop/host/src/repaint_deadline.rs#L6) · struct

```rust
pub struct RepaintPass {
    pass: Option<u64>,
    published: Option<Instant>,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::Editor (声明)`, `skin_editor_desktop::repaint_deadline::RepaintCycle::publish`, `skin_editor_desktop::repaint_deadline::RepaintPass::default`

实现／约束（显式或 derive，无类继承）：

- `impl RepaintPass` — [源码:10](../../../desktop/host/src/repaint_deadline.rs#L10)
- `impl Default for RepaintPass` — [源码:5](../../../desktop/host/src/repaint_deadline.rs#L5)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn begin(&mut self, ctx: &Context)` | impl RepaintPass | [源码:12](../../../desktop/host/src/repaint_deadline.rs#L12) |  |
| `fn publish_at(&mut self, ctx: &Context, cycle: RepaintCycle, now: Instant)` | impl RepaintPass | [源码:16](../../../desktop/host/src/repaint_deadline.rs#L16) | skin_editor_desktop::repaint_deadline::RepaintCycle |
| `fn default() -> RepaintPass` | impl Default for RepaintPass | [源码:5](../../../desktop/host/src/repaint_deadline.rs#L5) | skin_editor_desktop::repaint_deadline::RepaintPass |

## skin_editor_desktop::repaint_observation::CAUSES_PER_PASS

[源码:13](../../../desktop/host/src/repaint_observation.rs#L13) · constant

```rust
const CAUSES_PER_PASS: usize = 64;
```

## skin_editor_desktop::repaint_observation::PASS_LIMIT

[源码:11](../../../desktop/host/src/repaint_observation.rs#L11) · constant

```rust
const PASS_LIMIT: usize = 4096;
```

## skin_editor_desktop::repaint_observation::Pass

[源码:76](../../../desktop/host/src/repaint_observation.rs#L76) · struct

```rust
struct Pass {
    at: u64,
    pass: u64,
    frame: u64,
    causes: Vec<usize>,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::repaint_observation::Repaints (声明)`

## skin_editor_desktop::repaint_observation::REQUEST_LIMIT

[源码:10](../../../desktop/host/src/repaint_observation.rs#L10) · constant

```rust
const REQUEST_LIMIT: usize = 32768;
```

## skin_editor_desktop::repaint_observation::Repaints

[源码:82](../../../desktop/host/src/repaint_observation.rs#L82) · struct

```rust
pub struct Repaints {
    requests: Arc<Requests>,
    sources: Vec<Source>,
    passes: Vec<Pass>,
    inputs: Vec<[u64; 9]>,
    invalid: u64,
}
```

声明引用：`skin_editor_desktop::repaint_observation::Pass`, `skin_editor_desktop::repaint_observation::Requests`, `skin_editor_desktop::repaint_observation::Source`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::editor_audio::gui_verification::Run (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Repaints` — [源码:101](../../../desktop/host/src/repaint_observation.rs#L101)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn install(ctx: &Context) -> Self` | impl Repaints | [源码:102](../../../desktop/host/src/repaint_observation.rs#L102) |  |
| `fn source(&mut self, file: &str, line: u32) -> Option<usize>` | impl Repaints | [源码:120](../../../desktop/host/src/repaint_observation.rs#L120) |  |
| `pub fn pass(&mut self, ctx: &Context)` | impl Repaints | [源码:136](../../../desktop/host/src/repaint_observation.rs#L136) |  |
| `pub fn input(&mut self, ctx: &Context, input: &RawInput)` | impl Repaints | [源码:163](../../../desktop/host/src/repaint_observation.rs#L163) |  |
| `pub fn report(&self) -> String` | impl Repaints | [源码:185](../../../desktop/host/src/repaint_observation.rs#L185) |  |

## skin_editor_desktop::repaint_observation::Requests

[源码:27](../../../desktop/host/src/repaint_observation.rs#L27) · struct

```rust
struct Requests {
    next: AtomicUsize,
    rows: Box<[Slot]>,
}
```

声明引用：`skin_editor_desktop::repaint_observation::Slot`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::repaint_observation::Repaints (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Requests` — [源码:31](../../../desktop/host/src/repaint_observation.rs#L31)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn new(limit: usize) -> Self` | impl Requests | [源码:32](../../../desktop/host/src/repaint_observation.rs#L32) |  |
| `fn push(&self, at: u64, pass: u64, delay: u64)` | impl Requests | [源码:45](../../../desktop/host/src/repaint_observation.rs#L45) |  |
| `fn snapshot(&self) -> (Vec<[u64; 3]>, usize, usize)` | impl Requests | [源码:54](../../../desktop/host/src/repaint_observation.rs#L54) |  |

## skin_editor_desktop::repaint_observation::SOURCE_LIMIT

[源码:12](../../../desktop/host/src/repaint_observation.rs#L12) · constant

```rust
const SOURCE_LIMIT: usize = 128;
```

## skin_editor_desktop::repaint_observation::Slot

[源码:21](../../../desktop/host/src/repaint_observation.rs#L21) · struct

```rust
struct Slot {
    at: AtomicU64,
    pass: AtomicU64,
    delay: AtomicU64,
    ready: AtomicBool,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::repaint_observation::Requests (声明)`

## skin_editor_desktop::repaint_observation::Source

[源码:72](../../../desktop/host/src/repaint_observation.rs#L72) · struct

```rust
struct Source {
    file: String,
    line: u32,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::repaint_observation::Repaints (声明)`

## skin_editor_desktop::repaint_observation::source_name

[源码:89](../../../desktop/host/src/repaint_observation.rs#L89) · fn

```rust
fn source_name(file: &str) -> String
```

## skin_editor_desktop::repaint_observation::utc

[源码:14](../../../desktop/host/src/repaint_observation.rs#L14) · fn

```rust
fn utc() -> u64
```

## skin_editor_desktop::sample

[源码:78](../../../desktop/host/src/main.rs#L78) · fn

```rust
pub(crate) fn sample(
    ui: &mut EditorUi,
    session: &mut ProjectSession,
    images: &mut Images,
)
```

声明引用：`skin_editor_infrastructure::Images`, `skin_editor_ui::EditorUi`

## skin_editor_desktop::separated_verification::duration

[源码:87](../../../desktop/host/src/separated_verification.rs#L87) · fn

```rust
fn duration(
    owner: &mut BackgroundOwner,
    r: &mut Runtime,
    native: &NativeExecutor,
    span: Duration,
) -> Result<(), String>
```

声明引用：`skin_editor_audio::Runtime`, `skin_editor_infrastructure::separated_native::NativeExecutor`, `skin_editor_infrastructure::separated_output::BackgroundOwner`

## skin_editor_desktop::separated_verification::origin

[源码:12](../../../desktop/host/src/separated_verification.rs#L12) · fn

```rust
fn origin(revision: u64) -> Origin
```

声明引用：`skin_editor_audio::Origin`

## skin_editor_desktop::separated_verification::phase

[源码:96](../../../desktop/host/src/separated_verification.rs#L96) · fn

```rust
fn phase(name: &str)
```

## skin_editor_desktop::separated_verification::run

[源码:100](../../../desktop/host/src/separated_verification.rs#L100) · fn

```rust
pub fn run(destination: &str) -> Result<(), String>
```

## skin_editor_desktop::separated_verification::sound

[源码:21](../../../desktop/host/src/separated_verification.rs#L21) · fn

```rust
fn sound(
    r: &mut Runtime,
    pool: &PcmBudget,
    version: u64,
    frames: u64,
) -> Result<(), String>
```

声明引用：`skin_editor_audio::PcmBudget`, `skin_editor_audio::Runtime`

## skin_editor_desktop::separated_verification::tick

[源码:54](../../../desktop/host/src/separated_verification.rs#L54) · fn

```rust
fn tick(
    owner: &mut BackgroundOwner,
    r: &mut Runtime,
    native: &NativeExecutor,
) -> Result<(), String>
```

声明引用：`skin_editor_audio::Runtime`, `skin_editor_infrastructure::separated_native::NativeExecutor`, `skin_editor_infrastructure::separated_output::BackgroundOwner`

## skin_editor_desktop::separated_verification::trigger

[源码:44](../../../desktop/host/src/separated_verification.rs#L44) · fn

```rust
fn trigger(r: &mut Runtime, lane: Channel) -> Result<u64, String>
```

声明引用：`skin_editor_audio::Channel`, `skin_editor_audio::Runtime`

## skin_editor_desktop::separated_verification::wait

[源码:69](../../../desktop/host/src/separated_verification.rs#L69) · fn

```rust
fn wait(
    owner: &mut BackgroundOwner,
    r: &mut Runtime,
    native: &NativeExecutor,
    ready: impl FnMut(&Runtime, &BackgroundOwner) -> bool,
) -> Result<(), String>
```

声明引用：`skin_editor_audio::Runtime`, `skin_editor_infrastructure::separated_native::NativeExecutor`, `skin_editor_infrastructure::separated_output::BackgroundOwner`

## skin_editor_desktop::stable_window::Interval

[源码:8](../../../desktop/host/src/stable_window.rs#L8) · struct

```rust
pub struct Interval {
    start: Duration,
    draws: u64,
    inputs: u64,
    simulation: u64,
    background: u64,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::editor_audio::stability::Run (声明)`, `skin_editor_desktop::stable_window::Interval::clone`

实现／约束（显式或 derive，无类继承）：

- `impl Interval` — [源码:21](../../../desktop/host/src/stable_window.rs#L21)
- `impl Clone for Interval` — [源码:7](../../../desktop/host/src/stable_window.rs#L7)
- `impl Copy for Interval` — [源码:7](../../../desktop/host/src/stable_window.rs#L7)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn begin( now: Duration, settled_since: Duration, draws: u64, inputs: u64, simulation: u64, background: u64, ) -> Result<Self, &'static str>` | impl Interval | [源码:22](../../../desktop/host/src/stable_window.rs#L22) |  |
| `pub fn finish( self, now: Duration, draws: u64, inputs: u64, simulation: u64, background: u64, ) -> Result<Observation, &'static str>` | impl Interval | [源码:41](../../../desktop/host/src/stable_window.rs#L41) | skin_editor_desktop::stable_window::Observation |
| `fn clone(&self) -> Interval` | impl Clone for Interval | [源码:7](../../../desktop/host/src/stable_window.rs#L7) | skin_editor_desktop::stable_window::Interval |

## skin_editor_desktop::stable_window::MEASURE

[源码:5](../../../desktop/host/src/stable_window.rs#L5) · constant

```rust
pub const MEASURE: Duration;
```

## skin_editor_desktop::stable_window::Observation

[源码:15](../../../desktop/host/src/stable_window.rs#L15) · struct

```rust
pub struct Observation {
    pub elapsed: Duration,
    pub draws: u64,
    pub inputs: u64,
    pub frozen: bool,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::stable_window::Interval::finish`

## skin_editor_desktop::stable_window::SETTLE

[源码:4](../../../desktop/host/src/stable_window.rs#L4) · constant

```rust
pub const SETTLE: Duration;
```

## skin_editor_desktop::stream_fixture::SECONDS

[源码:9](../../../desktop/host/src/stream_fixture.rs#L9) · constant

```rust
pub const SECONDS: u32 = 360;
```

## skin_editor_desktop::stream_fixture::open

[源码:10](../../../desktop/host/src/stream_fixture.rs#L10) · fn

```rust
pub fn open(
    path: &Path,
    rate: u32,
    channels: u16,
    pool: &PcmBudget,
) -> Result<(BufferedBackground, StreamObservation), String>
```

声明引用：`skin_editor_audio::PcmBudget`, `skin_editor_infrastructure::background_stream::BufferedBackground`, `skin_editor_infrastructure::background_stream::StreamObservation`

## skin_editor_desktop::stream_fixture::write

[源码:22](../../../desktop/host/src/stream_fixture.rs#L22) · fn

```rust
pub fn write(path: &Path, rate: u32, channels: u16) -> Result<PathBuf, String>
```

## skin_editor_desktop::supply_verification::draft

[源码:71](../../../desktop/host/src/supply_verification.rs#L71) · fn

```rust
pub(crate) fn draft(
    session: &mut ProjectSession,
    next: &mut u128,
    action: DraftAction,
) -> Result<(), String>
```

## skin_editor_desktop::supply_verification::outcome

[源码:9](../../../desktop/host/src/supply_verification.rs#L9) · fn

```rust
fn outcome(
    client: &mut AudioSupplyClient,
    id: u64,
) -> Result<SupplyOutcome, String>
```

声明引用：`skin_editor_infrastructure::AudioSupplyClient`, `skin_editor_infrastructure::SupplyOutcome`

## skin_editor_desktop::supply_verification::ready

[源码:26](../../../desktop/host/src/supply_verification.rs#L26) · fn

```rust
fn ready(
    client: &mut AudioSupplyClient,
    session: &ProjectSession,
) -> Result<(), String>
```

声明引用：`skin_editor_infrastructure::AudioSupplyClient`

## skin_editor_desktop::supply_verification::run

[源码:87](../../../desktop/host/src/supply_verification.rs#L87) · fn

```rust
pub fn run(destination: &str) -> Result<(), String>
```

## skin_editor_desktop::supply_verification::trigger

[源码:58](../../../desktop/host/src/supply_verification.rs#L58) · fn

```rust
fn trigger(
    client: &mut AudioSupplyClient,
    session: &ProjectSession,
    lane: Channel,
) -> Result<(), String>
```

声明引用：`skin_editor_audio::Channel`, `skin_editor_infrastructure::AudioSupplyClient`

## skin_editor_desktop::text_work::Census

[源码:8](../../../desktop/host/src/text_work.rs#L8) · struct

```rust
pub struct Census {
    pub stage: u8,
    previous: Vec<Weak<Galley>>,
    current: Vec<Weak<Galley>>,
    rows: Vec<[u64; 16]>,
    invalid: u64,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::editor_audio::gui_verification::Run (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Census` — [源码:15](../../../desktop/host/src/text_work.rs#L15)
- `impl Plugin for Census` — [源码:70](../../../desktop/host/src/text_work.rs#L70)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn new() -> Self` | impl Census | [源码:16](../../../desktop/host/src/text_work.rs#L16) |  |
| `fn visit(&mut self, shape: &Shape, row: &mut [u64; 16], depth: usize)` | impl Census | [源码:25](../../../desktop/host/src/text_work.rs#L25) |  |
| `pub fn clear_identities(&mut self)` | impl Census | [源码:57](../../../desktop/host/src/text_work.rs#L57) |  |
| `pub fn report(&self) -> String` | impl Census | [源码:61](../../../desktop/host/src/text_work.rs#L61) |  |
| `fn debug_name(&self) -> &'static str` | impl Plugin for Census | [源码:71](../../../desktop/host/src/text_work.rs#L71) |  |
| `fn output_hook(&mut self, ctx: &Context, output: &mut FullOutput)` | impl Plugin for Census | [源码:74](../../../desktop/host/src/text_work.rs#L74) |  |
| `fn on_exit(&mut self, _: &Context)` | impl Plugin for Census | [源码:115](../../../desktop/host/src/text_work.rs#L115) |  |

## skin_editor_desktop::text_work::PASSES

[源码:7](../../../desktop/host/src/text_work.rs#L7) · constant

```rust
const PASSES: usize = 1024;
```

## skin_editor_desktop::text_work::SHAPES

[源码:6](../../../desktop/host/src/text_work.rs#L6) · constant

```rust
const SHAPES: usize = 512;
```

## skin_editor_desktop::verification::compare

[源码:151](../../../desktop/host/src/verification.rs#L151) · fn

```rust
fn compare(
    host: &NativeHost,
    cpu: &[u8],
    size: [u32; 2],
) -> Result<(Vec<u8>, u8), String>
```

声明引用：`skin_editor_desktop::presentation::NativeHost`

## skin_editor_desktop::verification::cpu_pixels

[源码:122](../../../desktop/host/src/verification.rs#L122) · fn

```rust
pub(crate) fn cpu_pixels(
    session: &ProjectSession,
    ui: &EditorUi,
    images: &Images,
    size: [u32; 2],
) -> Result<Vec<u8>, String>
```

声明引用：`skin_editor_infrastructure::Images`, `skin_editor_ui::EditorUi`

## skin_editor_desktop::verification::frame

[源码:22](../../../desktop/host/src/verification.rs#L22) · fn

```rust
pub(crate) fn frame(
    host: &mut NativeHost,
    ui: &EditorUi,
    session: &ProjectSession,
    size: [u32; 2],
    generation: u64,
) -> Result<(), String>
```

声明引用：`skin_editor_desktop::presentation::NativeHost`, `skin_editor_ui::EditorUi`

## skin_editor_desktop::verification::readback

[源码:54](../../../desktop/host/src/verification.rs#L54) · fn

```rust
pub(crate) fn readback(
    host: &NativeHost,
    texture: &Texture,
    size: [u32; 2],
) -> Result<Vec<u8>, String>
```

声明引用：`skin_editor_desktop::presentation::NativeHost`

## skin_editor_desktop::verification::readback_observed

[源码:61](../../../desktop/host/src/verification.rs#L61) · fn

```rust
pub(crate) fn readback_observed(
    host: &NativeHost,
    texture: &Texture,
    size: [u32; 2],
    observe: &mut dyn FnMut(&str) -> Result<(), String>,
) -> Result<Vec<u8>, String>
```

声明引用：`skin_editor_desktop::presentation::NativeHost`

## skin_editor_desktop::verification::run

[源码:398](../../../desktop/host/src/verification.rs#L398) · fn

```rust
pub fn run(output: &str) -> Result<(), String>
```

## skin_editor_desktop::verification::run_cadence

[源码:411](../../../desktop/host/src/verification.rs#L411) · fn

```rust
fn run_cadence(output: &str, joint: bool) -> Result<(), String>
```

## skin_editor_desktop::verification::run_joint_cadence

[源码:408](../../../desktop/host/src/verification.rs#L408) · fn

```rust
pub fn run_joint_cadence(output: &str) -> Result<(), String>
```

## skin_editor_desktop::verification::run_markers

[源码:630](../../../desktop/host/src/verification.rs#L630) · fn

```rust
pub fn run_markers(output: &str) -> Result<(), String>
```

## skin_editor_desktop::verification::run_memory

[源码:627](../../../desktop/host/src/verification.rs#L627) · fn

```rust
pub fn run_memory(output: &str) -> Result<(), String>
```

## skin_editor_desktop::verification::run_mode

[源码:639](../../../desktop/host/src/verification.rs#L639) · fn

```rust
fn run_mode(
    output: &str,
    audio_ui: bool,
    memory_usage: bool,
    reusable_staging: bool,
    markers: bool,
) -> Result<(), String>
```

## skin_editor_desktop::verification::run_staging

[源码:636](../../../desktop/host/src/verification.rs#L636) · fn

```rust
pub fn run_staging(output: &str, memory_usage: bool) -> Result<(), String>
```

## skin_editor_desktop::verification::run_transform_cadence

[源码:405](../../../desktop/host/src/verification.rs#L405) · fn

```rust
pub fn run_transform_cadence(output: &str) -> Result<(), String>
```

## skin_editor_desktop::verification::run_with_audio

[源码:633](../../../desktop/host/src/verification.rs#L633) · fn

```rust
pub fn run_with_audio(output: &str, audio_ui: bool) -> Result<(), String>
```

## skin_editor_desktop::verification::ui_image

[源码:284](../../../desktop/host/src/verification.rs#L284) · fn

```rust
fn ui_image(
    host: &mut NativeHost,
    ui: &mut EditorUi,
    session: &mut ProjectSession,
    ctx: &Context,
) -> Result<Vec<u8>, String>
```

声明引用：`skin_editor_desktop::presentation::NativeHost`, `skin_editor_ui::EditorUi`

## skin_editor_desktop::verification::verify_source_buffer_reuse

[源码:164](../../../desktop/host/src/verification.rs#L164) · fn

```rust
fn verify_source_buffer_reuse(
    host: &NativeHost,
    session: &ProjectSession,
    ui: &EditorUi,
    images: &Images,
    upload: SourceUpload,
) -> Result<(), String>
```

声明引用：`skin_editor_desktop::presentation::NativeHost`, `skin_editor_infrastructure::Images`, `skin_editor_rendering::SourceUpload`, `skin_editor_ui::EditorUi`

## skin_editor_desktop::verification::wait

[源码:13](../../../desktop/host/src/verification.rs#L13) · fn

```rust
fn wait(device: &Device) -> Result<(), String>
```

## skin_editor_desktop::worker_verification::Probe

[源码:96](../../../desktop/host/src/worker_verification.rs#L96) · struct

```rust
struct Probe {
    request: u64,
    lane: Channel,
    at: Instant,
    voice: Option<u64>,
    receipt_ms: f64,
}
```

声明引用：`skin_editor_audio::Channel`

## skin_editor_desktop::worker_verification::backend

[源码:12](../../../desktop/host/src/worker_verification.rs#L12) · fn

```rust
fn backend(e: impl Debug) -> String
```

## skin_editor_desktop::worker_verification::barrier

[源码:86](../../../desktop/host/src/worker_verification.rs#L86) · fn

```rust
fn barrier(client: &mut AudioSupplyClient) -> Result<(), String>
```

声明引用：`skin_editor_infrastructure::AudioSupplyClient`

## skin_editor_desktop::worker_verification::healthy

[源码:35](../../../desktop/host/src/worker_verification.rs#L35) · fn

```rust
fn healthy(
    client: &AudioSupplyClient,
    native: &NativeExecutor,
) -> Result<(), String>
```

声明引用：`skin_editor_infrastructure::AudioSupplyClient`, `skin_editor_infrastructure::separated_native::NativeExecutor`

## skin_editor_desktop::worker_verification::location_visible

[源码:523](../../../desktop/host/src/worker_verification.rs#L523) · fn

```rust
fn location_visible(
    client: &AudioSupplyClient,
    frame: u64,
) -> Result<(), String>
```

声明引用：`skin_editor_infrastructure::AudioSupplyClient`

## skin_editor_desktop::worker_verification::outcome

[源码:62](../../../desktop/host/src/worker_verification.rs#L62) · fn

```rust
fn outcome(
    client: &mut AudioSupplyClient,
    id: u64,
) -> Result<SupplyOutcome, String>
```

声明引用：`skin_editor_infrastructure::AudioSupplyClient`, `skin_editor_infrastructure::SupplyOutcome`

## skin_editor_desktop::worker_verification::prime

[源码:513](../../../desktop/host/src/worker_verification.rs#L513) · fn

```rust
fn prime(client: &AudioSupplyClient) -> Result<(), String>
```

声明引用：`skin_editor_infrastructure::AudioSupplyClient`

## skin_editor_desktop::worker_verification::ready

[源码:65](../../../desktop/host/src/worker_verification.rs#L65) · fn

```rust
fn ready(
    client: &mut AudioSupplyClient,
    session: &ProjectSession,
    native: &NativeExecutor,
) -> Result<(), String>
```

声明引用：`skin_editor_infrastructure::AudioSupplyClient`, `skin_editor_infrastructure::separated_native::NativeExecutor`

## skin_editor_desktop::worker_verification::receipt

[源码:45](../../../desktop/host/src/worker_verification.rs#L45) · fn

```rust
fn receipt(
    client: &mut AudioSupplyClient,
    id: u64,
) -> Result<SupplyReceipt, String>
```

声明引用：`skin_editor_infrastructure::AudioSupplyClient`, `skin_editor_infrastructure::SupplyReceipt`

## skin_editor_desktop::worker_verification::run

[源码:103](../../../desktop/host/src/worker_verification.rs#L103) · fn

```rust
pub fn run(destination: &str) -> Result<(), String>
```

## skin_editor_desktop::worker_verification::run_mode

[源码:109](../../../desktop/host/src/worker_verification.rs#L109) · fn

```rust
fn run_mode(destination: &str, streamed: bool) -> Result<(), String>
```

## skin_editor_desktop::worker_verification::stop

[源码:91](../../../desktop/host/src/worker_verification.rs#L91) · fn

```rust
fn stop(client: &mut AudioSupplyClient) -> Result<(), String>
```

声明引用：`skin_editor_infrastructure::AudioSupplyClient`

## skin_editor_desktop::worker_verification::streamed

[源码:106](../../../desktop/host/src/worker_verification.rs#L106) · fn

```rust
pub fn streamed(destination: &str) -> Result<(), String>
```

## skin_editor_desktop::worker_verification::wav

[源码:15](../../../desktop/host/src/worker_verification.rs#L15) · fn

```rust
fn wav(rate: u32, channels: u16, frames: u32) -> Vec<u8> ⓘ
```

