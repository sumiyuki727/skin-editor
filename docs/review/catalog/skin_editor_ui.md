# skin_editor_ui：编译器符号清单

快照：2026-10-30。来自含私有项的 rustdoc；显示编译器规范化可见域。含 derive 生成的实现，不把它们视作类继承。来源行可能指向宏定义／展开点。
引用列只列 rustdoc 解析出的项目类型链接，不能充当完整调用图；无链接不表示无引用。所有方法保留源码位置供逐项复核。测试和未启用的 cfg 分支另见源码索引。

## skin_editor_ui::DEFAULT_SHORTCUTS

[源码:41](../../../desktop/ui/src/shortcuts.rs#L41) · constant

```rust
pub const DEFAULT_SHORTCUTS: &[ShortcutBinding];
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_ui::shortcuts::DEFAULT_SHORTCUTS`

声明引用：`skin_editor_ui::ShortcutBinding`

## skin_editor_ui::EditorAction

[源码:6](../../../desktop/ui/src/shortcuts.rs#L6) · enum

```rust
pub enum EditorAction {
    Cancel,
    DeleteSelection,
    Undo,
    Redo,
    GamePrimary,
    GameSecondary,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_ui::shortcuts::EditorAction`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_ui::EditorAction::clone`, `skin_editor_ui::EditorAction::eq`, `skin_editor_ui::EditorUi::perform_action`, `skin_editor_ui::ShortcutBinding (声明)`, `skin_editor_ui::shortcuts::FrameInput::actions`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for EditorAction` — [源码:5](../../../desktop/ui/src/shortcuts.rs#L5)
- `impl Debug for EditorAction` — [源码:5](../../../desktop/ui/src/shortcuts.rs#L5)
- `impl PartialEq for EditorAction` — [源码:5](../../../desktop/ui/src/shortcuts.rs#L5)
- `impl Copy for EditorAction` — [源码:5](../../../desktop/ui/src/shortcuts.rs#L5)
- `impl Eq for EditorAction` — [源码:5](../../../desktop/ui/src/shortcuts.rs#L5)
- `impl StructuralPartialEq for EditorAction` — [源码:5](../../../desktop/ui/src/shortcuts.rs#L5)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> EditorAction` | impl Clone for EditorAction | [源码:5](../../../desktop/ui/src/shortcuts.rs#L5) | skin_editor_ui::EditorAction |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for EditorAction | [源码:5](../../../desktop/ui/src/shortcuts.rs#L5) |  |
| `fn eq(&self, other: &EditorAction) -> bool` | impl PartialEq for EditorAction | [源码:5](../../../desktop/ui/src/shortcuts.rs#L5) | skin_editor_ui::EditorAction |

## skin_editor_ui::EditorUi

[源码:45](../../../desktop/ui/src/lib.rs#L45) · struct

```rust
pub struct EditorUi {
    pub(crate) audio_port: Box<dyn AudioEditorPort>,
    pub audio_state: AudioEditorState,
    pub(crate) audio_sync_pending: bool,
    pub(crate) audio_sync_error: Option<String>,
    pub(crate) audio_window: bool,
    pub(crate) audio_labels: CounterLabels,
    pub audio_asset: Option<AssetId>,
    pub pending_imports: usize,
    pub audio_component: Option<AudioComponentId>,
    pub(crate) audio_path: String,
    pub(crate) audio_original: bool,
    pub(crate) audio_slot: String,
    pub target: Option<PreviewTarget>,
    pub(crate) selected: Vec<Selection>,
    pub(crate) selection_frame: Option<Frame>,
    pub paused: bool,
    pub interaction: bool,
    pub(crate) previous_interaction: bool,
    pub(crate) shortcuts: Router,
    pub scene: u8,
    pub auto_play: bool,
    pub fullscreen: bool,
    pub message: String,
    pub username: String,
    pub avatar: String,
    pub default_skin: String,
    pub stable: String,
    pub songs: String,
    pub simulation_micros: u64,
    pub accepted_inputs: u64,
    pub(crate) next_object: u128,
    pub(crate) next_request: u128,
    pub(crate) gesture: Option<EditGesture>,
    pub(crate) parameter_draft: bool,
    pub(crate) owned_draft: Option<(HistorySessionId, DraftId)>,
    pub(crate) blocked_parameter: bool,
    pub(crate) viewport_size: [u32; 2],
    pub(crate) viewport_generation: u64,
    pub(crate) path: String,
    pub(crate) settings: bool,
    pub canvas_rect: Option<Rect>,
}
```

声明引用：`skin_editor_ui::Selection`, `skin_editor_ui::audio::CounterLabels`, `skin_editor_ui::selection::EditGesture`, `skin_editor_ui::selection::Frame`, `skin_editor_ui::shortcuts::Router`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::Editor (声明)`, `skin_editor_desktop::editor_audio::window_recovery::Seed (声明)`, `skin_editor_desktop::import (声明)`, `skin_editor_desktop::sample (声明)`, `skin_editor_desktop::verification::cpu_pixels (声明)`, `skin_editor_desktop::verification::frame (声明)`, `skin_editor_desktop::verification::ui_image (声明)`, `skin_editor_desktop::verification::verify_source_buffer_reuse (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl EditorUi` — [源码:50](../../../desktop/ui/src/audio.rs#L50)
- `impl EditorUi` — [源码:5](../../../desktop/ui/src/history.rs#L5)
- `impl EditorUi` — [源码:237](../../../desktop/ui/src/selection.rs#L237)
- `impl EditorUi` — [源码:246](../../../desktop/ui/src/shortcuts.rs#L246)
- `impl EditorUi` — [源码:32](../../../desktop/ui/src/observation.rs#L32)
- `impl EditorUi` — [源码:135](../../../desktop/ui/src/lib.rs#L135)
- `impl Default for EditorUi` — [源码:88](../../../desktop/ui/src/lib.rs#L88)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn scene_is_frozen(&self) -> bool` | impl EditorUi | [源码:51](../../../desktop/ui/src/audio.rs#L51) |  |
| `pub fn open_audio_editor(&mut self)` | impl EditorUi | [源码:55](../../../desktop/ui/src/audio.rs#L55) |  |
| `pub fn set_audio_port(&mut self, port: Box<dyn AudioEditorPort>)` | impl EditorUi | [源码:58](../../../desktop/ui/src/audio.rs#L58) |  |
| `pub(crate) fn synchronize_audio(&mut self, session: &ProjectSession)` | impl EditorUi | [源码:62](../../../desktop/ui/src/audio.rs#L62) |  |
| `pub fn refresh_audio(&mut self, session: &ProjectSession)` | impl EditorUi | [源码:69](../../../desktop/ui/src/audio.rs#L69) |  |
| `pub fn audio_needs_poll(&self) -> bool` | impl EditorUi | [源码:83](../../../desktop/ui/src/audio.rs#L83) |  |
| `pub fn audio_command( &mut self, session: &ProjectSession, action: AudioEditorAction, )` | impl EditorUi | [源码:100](../../../desktop/ui/src/audio.rs#L100) |  |
| `pub fn create_audio_component( &mut self, session: &mut ProjectSession, asset: AssetId, slot: String, ) -> Option<AudioComponentId>` | impl EditorUi | [源码:119](../../../desktop/ui/src/audio.rs#L119) |  |
| `fn source_label(source: SkinAudioSource) -> String` | impl EditorUi | [源码:141](../../../desktop/ui/src/audio.rs#L141) |  |
| `pub(crate) fn audio_panel( &mut self, ctx: &Context, session: &mut ProjectSession, output: &mut UiOutput, observer: &mut Option<&mut dyn FnMut(UiPhase)>, )` | impl EditorUi | [源码:154](../../../desktop/ui/src/audio.rs#L154) | skin_editor_ui::UiOutput, skin_editor_ui::UiPhase |
| `pub(crate) fn history_panel( &mut self, ui: &mut Ui, session: &mut ProjectSession, )` | impl EditorUi | [源码:6](../../../desktop/ui/src/history.rs#L6) |  |
| `pub fn selection(&self) -> Option<Selection>` | impl EditorUi | [源码:238](../../../desktop/ui/src/selection.rs#L238) | skin_editor_ui::Selection |
| `pub fn selections(&self) -> &[Selection]` | impl EditorUi | [源码:241](../../../desktop/ui/src/selection.rs#L241) | skin_editor_ui::Selection |
| `pub fn toggle_selection( &mut self, session: &mut ProjectSession, item: Selection, )` | impl EditorUi | [源码:245](../../../desktop/ui/src/selection.rs#L245) | skin_editor_ui::Selection |
| `pub fn select(&mut self, session: &mut ProjectSession, item: Option<Selection>)` | impl EditorUi | [源码:248](../../../desktop/ui/src/selection.rs#L248) | skin_editor_ui::Selection |
| `pub(crate) fn pick( &mut self, session: &mut ProjectSession, item: Selection, additive: bool, )` | impl EditorUi | [源码:257](../../../desktop/ui/src/selection.rs#L257) | skin_editor_ui::Selection |
| `fn selection_stamp( &self, session: &ProjectSession, ) -> (HistorySessionId, ExpectedState, Option<DraftState>, Option<PreviewTarget>)` | impl EditorUi | [源码:291](../../../desktop/ui/src/selection.rs#L291) |  |
| `pub(crate) fn selection_geometry( &mut self, session: &ProjectSession, project: &SkinProject, ) -> Option<Geometry>` | impl EditorUi | [源码:300](../../../desktop/ui/src/selection.rs#L300) | skin_editor_ui::selection::Geometry |
| `pub(crate) fn remember_frame( &mut self, session: &ProjectSession, geometry: Geometry, )` | impl EditorUi | [源码:319](../../../desktop/ui/src/selection.rs#L319) | skin_editor_ui::selection::Geometry |
| `pub(crate) fn sync_interaction_mode(&mut self, session: &mut ProjectSession)` | impl EditorUi | [源码:247](../../../desktop/ui/src/shortcuts.rs#L247) |  |
| `pub(crate) fn perform_action( &mut self, session: &mut ProjectSession, action: EditorAction, ) -> u32` | impl EditorUi | [源码:254](../../../desktop/ui/src/shortcuts.rs#L254) | skin_editor_ui::EditorAction |
| `pub(crate) fn observed_project( session: &ProjectSession, observer: &mut Option<&mut dyn FnMut(UiPhase)>, resume: UiPhase, ) -> SkinProject` | impl EditorUi | [源码:33](../../../desktop/ui/src/observation.rs#L33) | skin_editor_ui::UiPhase |
| `pub fn object_id(&mut self) -> u128` | impl EditorUi | [源码:136](../../../desktop/ui/src/lib.rs#L136) |  |
| `pub(crate) fn request_id(&mut self) -> RequestId` | impl EditorUi | [源码:141](../../../desktop/ui/src/lib.rs#L141) |  |
| `pub fn submit( &mut self, session: &mut ProjectSession, commands: Vec<Command>, ) -> bool` | impl EditorUi | [源码:146](../../../desktop/ui/src/lib.rs#L146) |  |
| `pub fn draft( &mut self, session: &mut ProjectSession, action: DraftAction, ) -> bool` | impl EditorUi | [源码:166](../../../desktop/ui/src/lib.rs#L166) |  |
| `pub(crate) fn draft_state(session: &ProjectSession) -> Option<DraftState>` | impl EditorUi | [源码:191](../../../desktop/ui/src/lib.rs#L191) |  |
| `pub fn cancel(&mut self, session: &mut ProjectSession)` | impl EditorUi | [源码:198](../../../desktop/ui/src/lib.rs#L198) |  |
| `pub(crate) fn cancel_owned(&mut self, session: &mut ProjectSession)` | impl EditorUi | [源码:211](../../../desktop/ui/src/lib.rs#L211) |  |
| `pub fn interrupt_for_fault(&mut self, session: &mut ProjectSession) -> bool` | impl EditorUi | [源码:227](../../../desktop/ui/src/lib.rs#L227) |  |
| `pub fn commit(&mut self, session: &mut ProjectSession)` | impl EditorUi | [源码:240](../../../desktop/ui/src/lib.rs#L240) |  |
| `pub(crate) fn parameter( &mut self, session: &mut ProjectSession, response: Response, commands: Vec<Command>, )` | impl EditorUi | [源码:259](../../../desktop/ui/src/lib.rs#L259) |  |
| `pub fn navigation( &mut self, session: &mut ProjectSession, action: HistoryAction, )` | impl EditorUi | [源码:292](../../../desktop/ui/src/lib.rs#L292) |  |
| `pub fn project(session: &ProjectSession) -> SkinProject` | impl EditorUi | [源码:311](../../../desktop/ui/src/lib.rs#L311) |  |
| `pub fn reconcile(&mut self, session: &ProjectSession)` | impl EditorUi | [源码:319](../../../desktop/ui/src/lib.rs#L319) |  |
| `pub(crate) fn reconcile_project(&mut self, project: &SkinProject)` | impl EditorUi | [源码:323](../../../desktop/ui/src/lib.rs#L323) |  |
| `pub fn add_document(&mut self, session: &mut ProjectSession) -> DocumentId` | impl EditorUi | [源码:345](../../../desktop/ui/src/lib.rs#L345) |  |
| `pub fn add_asset_layer(&mut self, session: &mut ProjectSession, asset: AssetId)` | impl EditorUi | [源码:362](../../../desktop/ui/src/lib.rs#L362) |  |
| `pub(crate) fn transform( project: &SkinProject, selection: Selection, ) -> Option<Affine2>` | impl EditorUi | [源码:426](../../../desktop/ui/src/lib.rs#L426) | skin_editor_ui::Selection |
| `pub(crate) fn transform_command( selection: Selection, transform: Affine2, ) -> Command` | impl EditorUi | [源码:432](../../../desktop/ui/src/lib.rs#L432) | skin_editor_ui::Selection |
| `pub fn show( &mut self, root: &mut Ui, session: &mut ProjectSession, presenter: &mut dyn Presenter, ) -> UiOutput` | impl EditorUi | [源码:445](../../../desktop/ui/src/lib.rs#L445) | skin_editor_ui::Presenter, skin_editor_ui::UiOutput |
| `pub fn show_observed( &mut self, root: &mut Ui, session: &mut ProjectSession, presenter: &mut dyn Presenter, observer: Option<&mut dyn FnMut(UiPhase)>, ) -> UiOutput` | impl EditorUi | [源码:455](../../../desktop/ui/src/lib.rs#L455) | skin_editor_ui::Presenter, skin_editor_ui::UiOutput, skin_editor_ui::UiPhase |
| `fn default() -> Self` | impl Default for EditorUi | [源码:89](../../../desktop/ui/src/lib.rs#L89) |  |

## skin_editor_ui::MODIFIER_HINTS

[源码:99](../../../desktop/ui/src/shortcuts.rs#L99) · constant

```rust
pub const MODIFIER_HINTS: &[(&str, &str)];
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_ui::shortcuts::MODIFIER_HINTS`

## skin_editor_ui::PresentedView

[源码:26](../../../desktop/ui/src/lib.rs#L26) · struct

```rust
pub struct PresentedView {
    pub frame: FrameDescriptor,
    pub view: Affine2,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::presentation::NativeHost::current`, `skin_editor_ui::PresentedView::clone`, `skin_editor_ui::Presenter (声明)`, `skin_editor_ui::Presenter::current`, `skin_editor_ui::picking::layer_at (声明)`, `skin_editor_ui::selection::EditGesture::new`, `skin_editor_ui::selection::Geometry::controls`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for PresentedView` — [源码:25](../../../desktop/ui/src/lib.rs#L25)
- `impl Copy for PresentedView` — [源码:25](../../../desktop/ui/src/lib.rs#L25)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> PresentedView` | impl Clone for PresentedView | [源码:25](../../../desktop/ui/src/lib.rs#L25) | skin_editor_ui::PresentedView |

## skin_editor_ui::Presenter

[源码:31](../../../desktop/ui/src/lib.rs#L31) · trait

```rust
pub trait Presenter {
    // Required methods
    fn current(&self) -> Option<PresentedView>;
    fn paint(&mut self, ui: &mut Ui, key: FrameKey, rect: Rect);
    fn thumbnail(&mut self, ui: &mut Ui, blob: BlobRef);
}
```

声明引用：`skin_editor_ui::PresentedView`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_ui::EditorUi::show`, `skin_editor_ui::EditorUi::show_observed`

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn current(&self) -> Option<PresentedView>` |  | [源码:32](../../../desktop/ui/src/lib.rs#L32) | skin_editor_ui::PresentedView |
| `fn paint(&mut self, ui: &mut Ui, key: FrameKey, rect: Rect)` |  | [源码:33](../../../desktop/ui/src/lib.rs#L33) |  |
| `fn thumbnail(&mut self, ui: &mut Ui, blob: BlobRef)` |  | [源码:34](../../../desktop/ui/src/lib.rs#L34) |  |

## skin_editor_ui::Selection

[源码:21](../../../desktop/ui/src/lib.rs#L21) · enum

```rust
pub enum Selection {
    Layer(DocumentId, LayerId),
    Component(ComponentId),
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_ui::EditorUi (声明)`, `skin_editor_ui::EditorUi::pick`, `skin_editor_ui::EditorUi::select`, `skin_editor_ui::EditorUi::selection`, `skin_editor_ui::EditorUi::selections`, `skin_editor_ui::EditorUi::toggle_selection`, `skin_editor_ui::EditorUi::transform`, `skin_editor_ui::EditorUi::transform_command`, `skin_editor_ui::Selection::clone`, `skin_editor_ui::Selection::cmp`, `skin_editor_ui::Selection::eq`, `skin_editor_ui::Selection::partial_cmp`, `skin_editor_ui::gestures::geometry (声明)`, `skin_editor_ui::picking::document (声明)`, `skin_editor_ui::picking::layer_at (声明)`, `skin_editor_ui::selection::EditGesture::new`, `skin_editor_ui::selection::EditGesture::values`, `skin_editor_ui::selection::Geometry::joint`, `skin_editor_ui::selection::Geometry::single`, `skin_editor_ui::selection::Member (声明)`, `skin_editor_ui::selection::effective (声明)`, `skin_editor_ui::selection::parent (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for Selection` — [源码:20](../../../desktop/ui/src/lib.rs#L20)
- `impl Debug for Selection` — [源码:20](../../../desktop/ui/src/lib.rs#L20)
- `impl Ord for Selection` — [源码:20](../../../desktop/ui/src/lib.rs#L20)
- `impl PartialEq for Selection` — [源码:20](../../../desktop/ui/src/lib.rs#L20)
- `impl PartialOrd for Selection` — [源码:20](../../../desktop/ui/src/lib.rs#L20)
- `impl Copy for Selection` — [源码:20](../../../desktop/ui/src/lib.rs#L20)
- `impl Eq for Selection` — [源码:20](../../../desktop/ui/src/lib.rs#L20)
- `impl StructuralPartialEq for Selection` — [源码:20](../../../desktop/ui/src/lib.rs#L20)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> Selection` | impl Clone for Selection | [源码:20](../../../desktop/ui/src/lib.rs#L20) | skin_editor_ui::Selection |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for Selection | [源码:20](../../../desktop/ui/src/lib.rs#L20) |  |
| `fn cmp(&self, other: &Selection) -> Ordering` | impl Ord for Selection | [源码:20](../../../desktop/ui/src/lib.rs#L20) | skin_editor_ui::Selection |
| `fn eq(&self, other: &Selection) -> bool` | impl PartialEq for Selection | [源码:20](../../../desktop/ui/src/lib.rs#L20) | skin_editor_ui::Selection |
| `fn partial_cmp(&self, other: &Selection) -> Option<Ordering>` | impl PartialOrd for Selection | [源码:20](../../../desktop/ui/src/lib.rs#L20) | skin_editor_ui::Selection |

## skin_editor_ui::ShortcutBinding

[源码:27](../../../desktop/ui/src/shortcuts.rs#L27) · struct

```rust
pub struct ShortcutBinding {
    pub action: EditorAction,
    pub scope: ShortcutScope,
    pub display: &'static str,
    pub description: &'static str,
    key: Key,
    chord: Chord,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_ui::shortcuts::ShortcutBinding`

声明引用：`skin_editor_ui::EditorAction`, `skin_editor_ui::ShortcutScope`, `skin_editor_ui::shortcuts::Chord`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_ui::DEFAULT_SHORTCUTS (声明)`, `skin_editor_ui::ShortcutBinding::clone`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for ShortcutBinding` — [源码:26](../../../desktop/ui/src/shortcuts.rs#L26)
- `impl Debug for ShortcutBinding` — [源码:26](../../../desktop/ui/src/shortcuts.rs#L26)
- `impl Copy for ShortcutBinding` — [源码:26](../../../desktop/ui/src/shortcuts.rs#L26)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> ShortcutBinding` | impl Clone for ShortcutBinding | [源码:26](../../../desktop/ui/src/shortcuts.rs#L26) | skin_editor_ui::ShortcutBinding |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for ShortcutBinding | [源码:26](../../../desktop/ui/src/shortcuts.rs#L26) |  |

## skin_editor_ui::ShortcutScope

[源码:15](../../../desktop/ui/src/shortcuts.rs#L15) · enum

```rust
pub enum ShortcutScope {
    Editing,
    GameCanvas,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_ui::shortcuts::ShortcutScope`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_ui::ShortcutBinding (声明)`, `skin_editor_ui::ShortcutScope::clone`, `skin_editor_ui::ShortcutScope::eq`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for ShortcutScope` — [源码:14](../../../desktop/ui/src/shortcuts.rs#L14)
- `impl Debug for ShortcutScope` — [源码:14](../../../desktop/ui/src/shortcuts.rs#L14)
- `impl PartialEq for ShortcutScope` — [源码:14](../../../desktop/ui/src/shortcuts.rs#L14)
- `impl Copy for ShortcutScope` — [源码:14](../../../desktop/ui/src/shortcuts.rs#L14)
- `impl Eq for ShortcutScope` — [源码:14](../../../desktop/ui/src/shortcuts.rs#L14)
- `impl StructuralPartialEq for ShortcutScope` — [源码:14](../../../desktop/ui/src/shortcuts.rs#L14)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> ShortcutScope` | impl Clone for ShortcutScope | [源码:14](../../../desktop/ui/src/shortcuts.rs#L14) | skin_editor_ui::ShortcutScope |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for ShortcutScope | [源码:14](../../../desktop/ui/src/shortcuts.rs#L14) |  |
| `fn eq(&self, other: &ShortcutScope) -> bool` | impl PartialEq for ShortcutScope | [源码:14](../../../desktop/ui/src/shortcuts.rs#L14) | skin_editor_ui::ShortcutScope |

## skin_editor_ui::UiOutput

[源码:37](../../../desktop/ui/src/lib.rs#L37) · struct

```rust
pub struct UiOutput {
    pub imports: Vec<String>,
    pub sample: bool,
    pub viewport: Option<([u32; 2], u64)>,
    pub game_inputs: u32,
    pub audio_imports: Vec<(String, bool)>,
    pub sample_sound: bool,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::Editor::complete_ui`, `skin_editor_ui::EditorUi::audio_panel`, `skin_editor_ui::EditorUi::show`, `skin_editor_ui::EditorUi::show_observed`, `skin_editor_ui::UiOutput::default`

实现／约束（显式或 derive，无类继承）：

- `impl Default for UiOutput` — [源码:36](../../../desktop/ui/src/lib.rs#L36)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn default() -> UiOutput` | impl Default for UiOutput | [源码:36](../../../desktop/ui/src/lib.rs#L36) | skin_editor_ui::UiOutput |

## skin_editor_ui::UiPhase

[源码:6](../../../desktop/ui/src/observation.rs#L6) · enum

```rust
pub enum UiPhase {
    Prelude,
    SceneControls,
    LayersAssets,
    PropertiesHistory,
    Canvas,
    AudioPanel,
    SettingsDiagnostics,
    ProjectSnapshot,
    SceneWidgets,
    SceneReadout,
    AudioDevice,
    AudioStatus,
    AudioLibrary,
    AudioComponents,
    AudioProperties,
    AudioReadiness,
    AudioActions,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_ui::observation::UiPhase`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::main_work::ui_category (声明)`, `skin_editor_ui::EditorUi::audio_panel`, `skin_editor_ui::EditorUi::observed_project`, `skin_editor_ui::EditorUi::show_observed`, `skin_editor_ui::UiPhase::clone`, `skin_editor_ui::UiPhase::eq`, `skin_editor_ui::observation::phase (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for UiPhase` — [源码:5](../../../desktop/ui/src/observation.rs#L5)
- `impl Debug for UiPhase` — [源码:5](../../../desktop/ui/src/observation.rs#L5)
- `impl PartialEq for UiPhase` — [源码:5](../../../desktop/ui/src/observation.rs#L5)
- `impl Copy for UiPhase` — [源码:5](../../../desktop/ui/src/observation.rs#L5)
- `impl Eq for UiPhase` — [源码:5](../../../desktop/ui/src/observation.rs#L5)
- `impl StructuralPartialEq for UiPhase` — [源码:5](../../../desktop/ui/src/observation.rs#L5)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> UiPhase` | impl Clone for UiPhase | [源码:5](../../../desktop/ui/src/observation.rs#L5) | skin_editor_ui::UiPhase |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for UiPhase | [源码:5](../../../desktop/ui/src/observation.rs#L5) |  |
| `fn eq(&self, other: &UiPhase) -> bool` | impl PartialEq for UiPhase | [源码:5](../../../desktop/ui/src/observation.rs#L5) | skin_editor_ui::UiPhase |

## skin_editor_ui::audio::CounterLabels

[源码:5](../../../desktop/ui/src/audio.rs#L5) · struct

```rust
pub(crate) struct CounterLabels {
    previous: Option<(AudioOutputObservation, bool, bool)>,
    at: f64,
    values: [String; 2],
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_ui::EditorUi (声明)`, `skin_editor_ui::audio::CounterLabels::default`

实现／约束（显式或 derive，无类继承）：

- `impl CounterLabels` — [源码:10](../../../desktop/ui/src/audio.rs#L10)
- `impl Default for CounterLabels` — [源码:4](../../../desktop/ui/src/audio.rs#L4)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn update( &mut self, now: f64, output: &AudioOutputObservation, frozen: bool, freezing: bool, ) -> bool` | impl CounterLabels | [源码:11](../../../desktop/ui/src/audio.rs#L11) |  |
| `fn default() -> CounterLabels` | impl Default for CounterLabels | [源码:4](../../../desktop/ui/src/audio.rs#L4) | skin_editor_ui::audio::CounterLabels |

## skin_editor_ui::gestures::ControlBox

[源码:206](../../../desktop/ui/src/gestures.rs#L206) · struct

```rust
pub(crate) struct ControlBox {
    pub corners: [Pos2; 4],
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_ui::gestures::ControlBox::clone`, `skin_editor_ui::gestures::hit (声明)`, `skin_editor_ui::gestures::rotation (声明)`, `skin_editor_ui::selection::Geometry::controls`

实现／约束（显式或 derive，无类继承）：

- `impl ControlBox` — [源码:222](../../../desktop/ui/src/gestures.rs#L222)
- `impl Clone for ControlBox` — [源码:205](../../../desktop/ui/src/gestures.rs#L205)
- `impl Debug for ControlBox` — [源码:205](../../../desktop/ui/src/gestures.rs#L205)
- `impl From<Rect> for ControlBox` — [源码:210](../../../desktop/ui/src/gestures.rs#L210)
- `impl Copy for ControlBox` — [源码:205](../../../desktop/ui/src/gestures.rs#L205)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn contains(self, p: Pos2) -> bool` | impl ControlBox | [源码:223](../../../desktop/ui/src/gestures.rs#L223) |  |
| `pub fn center(self) -> Pos2` | impl ControlBox | [源码:235](../../../desktop/ui/src/gestures.rs#L235) |  |
| `pub fn top(self) -> Pos2` | impl ControlBox | [源码:238](../../../desktop/ui/src/gestures.rs#L238) |  |
| `fn clone(&self) -> ControlBox` | impl Clone for ControlBox | [源码:205](../../../desktop/ui/src/gestures.rs#L205) | skin_editor_ui::gestures::ControlBox |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for ControlBox | [源码:205](../../../desktop/ui/src/gestures.rs#L205) |  |
| `fn from(r: Rect) -> Self` | impl From<Rect> for ControlBox | [源码:211](../../../desktop/ui/src/gestures.rs#L211) |  |

## skin_editor_ui::gestures::Gesture

[源码:11](../../../desktop/ui/src/gestures.rs#L11) · struct

```rust
pub(crate) struct Gesture {
    base: Affine2,
    mapping: Affine2,
    inverse: Affine2,
    ratio: [f64; 2],
    press: [f64; 2],
    handle: Handle,
}
```

声明引用：`skin_editor_ui::gestures::Handle`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_ui::selection::EditGesture (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Gesture` — [源码:19](../../../desktop/ui/src/gestures.rs#L19)
- `impl Debug for Gesture` — [源码:10](../../../desktop/ui/src/gestures.rs#L10)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn new( base: Affine2, mapping: Affine2, ratio: [f64; 2], press: [f64; 2], handle: Handle, ) -> Option<Self>` | impl Gesture | [源码:20](../../../desktop/ui/src/gestures.rs#L20) | skin_editor_ui::gestures::Handle |
| `pub fn transform_at( &self, pointer: Pos2, canvas_origin: Pos2, preserve: bool, ) -> Option<Affine2>` | impl Gesture | [源码:50](../../../desktop/ui/src/gestures.rs#L50) |  |
| `fn pointer_delta(&self, pointer: Pos2, canvas_origin: Pos2) -> Vec2` | impl Gesture | [源码:60](../../../desktop/ui/src/gestures.rs#L60) |  |
| `pub fn frame_delta_at( &self, pointer: Pos2, origin: Pos2, preserve: bool, ) -> Option<Affine2>` | impl Gesture | [源码:67](../../../desktop/ui/src/gestures.rs#L67) |  |
| `fn transform_with_aspect(&self, delta: Vec2, preserve: bool) -> Option<Affine2>` | impl Gesture | [源码:74](../../../desktop/ui/src/gestures.rs#L74) |  |
| `fn frame_delta(&self, delta: Vec2, preserve: bool) -> Option<Affine2>` | impl Gesture | [源码:87](../../../desktop/ui/src/gestures.rs#L87) |  |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for Gesture | [源码:10](../../../desktop/ui/src/gestures.rs#L10) |  |

## skin_editor_ui::gestures::Handle

[源码:5](../../../desktop/ui/src/gestures.rs#L5) · enum

```rust
pub(crate) enum Handle {
    Move,
    Scale([f64; 2]),
    Rotate([f64; 2]),
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_ui::gestures::Gesture (声明)`, `skin_editor_ui::gestures::Gesture::new`, `skin_editor_ui::gestures::hit (声明)`, `skin_editor_ui::selection::EditGesture::new`

实现／约束（显式或 derive，无类继承）：

- `impl Debug for Handle` — [源码:4](../../../desktop/ui/src/gestures.rs#L4)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for Handle | [源码:4](../../../desktop/ui/src/gestures.rs#L4) |  |

## skin_editor_ui::gestures::geometry

[源码:151](../../../desktop/ui/src/gestures.rs#L151) · fn

```rust
pub(crate) fn geometry(
    project: &SkinProject,
    selection: Selection,
) -> Option<(Bounds, Affine2)>
```

声明引用：`skin_editor_ui::Selection`

## skin_editor_ui::gestures::hit

[源码:242](../../../desktop/ui/src/gestures.rs#L242) · fn

```rust
pub(crate) fn hit(
    bounds: Option<ControlBox>,
    canvas: Rect,
    press: Pos2,
    to_frame: impl Fn(Pos2) -> [f64; 2],
) -> Handle
```

声明引用：`skin_editor_ui::gestures::ControlBox`, `skin_editor_ui::gestures::Handle`

## skin_editor_ui::gestures::rotation

[源码:261](../../../desktop/ui/src/gestures.rs#L261) · fn

```rust
pub(crate) fn rotation(bounds: impl Into<ControlBox>, canvas: Rect) -> Pos2
```

声明引用：`skin_editor_ui::gestures::ControlBox`

## skin_editor_ui::observation::phase

[源码:26](../../../desktop/ui/src/observation.rs#L26) · fn

```rust
pub(crate) fn phase(
    observer: &mut Option<&mut dyn FnMut(UiPhase)>,
    value: UiPhase,
)
```

声明引用：`skin_editor_ui::UiPhase`

## skin_editor_ui::picking::document

[源码:7](../../../desktop/ui/src/picking.rs#L7) · fn

```rust
fn document(
    project: &SkinProject,
    id: DocumentId,
    content: Affine2,
    clip: Affine2,
    point: [f64; 2],
) -> Option<Selection>
```

声明引用：`skin_editor_ui::Selection`

## skin_editor_ui::picking::inside

[源码:4](../../../desktop/ui/src/picking.rs#L4) · fn

```rust
fn inside(p: [f64; 2], size: PixelSize) -> bool
```

## skin_editor_ui::picking::layer_at

[源码:48](../../../desktop/ui/src/picking.rs#L48) · fn

```rust
pub(crate) fn layer_at(
    project: &SkinProject,
    view: PresentedView,
    canvas: Rect,
    pointer: Pos2,
) -> Option<Selection>
```

声明引用：`skin_editor_ui::PresentedView`, `skin_editor_ui::Selection`

## skin_editor_ui::selection::EditGesture

[源码:168](../../../desktop/ui/src/selection.rs#L168) · struct

```rust
pub(crate) struct EditGesture {
    control: Gesture,
    members: Vec<Member>,
    pub before: Geometry,
    pub after: Geometry,
}
```

声明引用：`skin_editor_ui::gestures::Gesture`, `skin_editor_ui::selection::Geometry`, `skin_editor_ui::selection::Member`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_ui::EditorUi (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl EditGesture` — [源码:174](../../../desktop/ui/src/selection.rs#L174)
- `impl Debug for EditGesture` — [源码:167](../../../desktop/ui/src/selection.rs#L167)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn new( project: &SkinProject, items: &[Selection], geometry: Geometry, view: PresentedView, ratio: [f64; 2], press: [f64; 2], handle: Handle, ) -> Option<Self>` | impl EditGesture | [源码:175](../../../desktop/ui/src/selection.rs#L175) | skin_editor_ui::PresentedView, skin_editor_ui::Selection, skin_editor_ui::gestures::Handle, skin_editor_ui::selection::Geometry |
| `pub fn values( &self, pointer: Pos2, origin: Pos2, preserve: bool, ) -> Option<(Vec<(Selection, Affine2)>, Geometry)>` | impl EditGesture | [源码:209](../../../desktop/ui/src/selection.rs#L209) | skin_editor_ui::Selection, skin_editor_ui::selection::Geometry |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for EditGesture | [源码:167](../../../desktop/ui/src/selection.rs#L167) |  |

## skin_editor_ui::selection::Frame

[源码:98](../../../desktop/ui/src/selection.rs#L98) · struct

```rust
pub(crate) struct Frame {
    pub stamp: (HistorySessionId, ExpectedState, Option<DraftState>, Option<PreviewTarget>),
    pub geometry: Geometry,
}
```

声明引用：`skin_editor_ui::selection::Geometry`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_ui::EditorUi (声明)`, `skin_editor_ui::selection::Frame::clone`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for Frame` — [源码:97](../../../desktop/ui/src/selection.rs#L97)
- `impl Debug for Frame` — [源码:97](../../../desktop/ui/src/selection.rs#L97)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> Frame` | impl Clone for Frame | [源码:97](../../../desktop/ui/src/selection.rs#L97) | skin_editor_ui::selection::Frame |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for Frame | [源码:97](../../../desktop/ui/src/selection.rs#L97) |  |

## skin_editor_ui::selection::Geometry

[源码:13](../../../desktop/ui/src/selection.rs#L13) · struct

```rust
pub(crate) struct Geometry {
    pub low: [f64; 2],
    pub high: [f64; 2],
    pub world: Affine2,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_ui::EditorUi::remember_frame`, `skin_editor_ui::EditorUi::selection_geometry`, `skin_editor_ui::selection::EditGesture (声明)`, `skin_editor_ui::selection::EditGesture::new`, `skin_editor_ui::selection::EditGesture::values`, `skin_editor_ui::selection::Frame (声明)`, `skin_editor_ui::selection::Geometry::clone`, `skin_editor_ui::selection::Geometry::eq`

实现／约束（显式或 derive，无类继承）：

- `impl Geometry` — [源码:18](../../../desktop/ui/src/selection.rs#L18)
- `impl Clone for Geometry` — [源码:12](../../../desktop/ui/src/selection.rs#L12)
- `impl Debug for Geometry` — [源码:12](../../../desktop/ui/src/selection.rs#L12)
- `impl PartialEq for Geometry` — [源码:12](../../../desktop/ui/src/selection.rs#L12)
- `impl Copy for Geometry` — [源码:12](../../../desktop/ui/src/selection.rs#L12)
- `impl StructuralPartialEq for Geometry` — [源码:12](../../../desktop/ui/src/selection.rs#L12)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn single( project: &SkinProject, item: Selection, target: PreviewTarget, ) -> Option<Self>` | impl Geometry | [源码:19](../../../desktop/ui/src/selection.rs#L19) | skin_editor_ui::Selection |
| `fn points(self) -> [[f64; 2]; 4]` | impl Geometry | [源码:36](../../../desktop/ui/src/selection.rs#L36) |  |
| `pub fn controls(self, view: PresentedView, canvas: Rect) -> Option<ControlBox>` | impl Geometry | [源码:44](../../../desktop/ui/src/selection.rs#L44) | skin_editor_ui::PresentedView, skin_editor_ui::gestures::ControlBox |
| `pub fn joint( project: &SkinProject, items: &[Selection], target: PreviewTarget, ) -> Option<Self>` | impl Geometry | [源码:60](../../../desktop/ui/src/selection.rs#L60) | skin_editor_ui::Selection |
| `fn clone(&self) -> Geometry` | impl Clone for Geometry | [源码:12](../../../desktop/ui/src/selection.rs#L12) | skin_editor_ui::selection::Geometry |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for Geometry | [源码:12](../../../desktop/ui/src/selection.rs#L12) |  |
| `fn eq(&self, other: &Geometry) -> bool` | impl PartialEq for Geometry | [源码:12](../../../desktop/ui/src/selection.rs#L12) | skin_editor_ui::selection::Geometry |

## skin_editor_ui::selection::MAX_SELECTED

[源码:4](../../../desktop/ui/src/selection.rs#L4) · constant

```rust
pub const MAX_SELECTED: usize = 256;
```

## skin_editor_ui::selection::Member

[源码:161](../../../desktop/ui/src/selection.rs#L161) · struct

```rust
struct Member {
    item: Selection,
    base: Affine2,
    parent: Affine2,
    inverse: Affine2,
}
```

声明引用：`skin_editor_ui::Selection`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_ui::selection::EditGesture (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Debug for Member` — [源码:160](../../../desktop/ui/src/selection.rs#L160)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for Member | [源码:160](../../../desktop/ui/src/selection.rs#L160) |  |

## skin_editor_ui::selection::Stamp

[源码:5](../../../desktop/ui/src/selection.rs#L5) · type

```rust
pub(crate) type Stamp = (HistorySessionId, ExpectedState, Option<DraftState>, Option<PreviewTarget>);
```

## skin_editor_ui::selection::effective

[源码:105](../../../desktop/ui/src/selection.rs#L105) · fn

```rust
pub(crate) fn effective(
    project: &SkinProject,
    items: &[Selection],
) -> Vec<Selection>
```

声明引用：`skin_editor_ui::Selection`

## skin_editor_ui::selection::parent

[源码:137](../../../desktop/ui/src/selection.rs#L137) · fn

```rust
fn parent(
    project: &SkinProject,
    item: Selection,
    target: PreviewTarget,
) -> Option<Affine2>
```

声明引用：`skin_editor_ui::Selection`

## skin_editor_ui::shortcuts::Chord

[源码:20](../../../desktop/ui/src/shortcuts.rs#L20) · enum

```rust
enum Chord {
    Any,
    Plain,
    Primary,
    PrimaryShift,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_ui::ShortcutBinding (声明)`, `skin_editor_ui::shortcuts::Chord::clone`, `skin_editor_ui::shortcuts::Chord::eq`

实现／约束（显式或 derive，无类继承）：

- `impl Chord` — [源码:113](../../../desktop/ui/src/shortcuts.rs#L113)
- `impl Clone for Chord` — [源码:19](../../../desktop/ui/src/shortcuts.rs#L19)
- `impl Debug for Chord` — [源码:19](../../../desktop/ui/src/shortcuts.rs#L19)
- `impl PartialEq for Chord` — [源码:19](../../../desktop/ui/src/shortcuts.rs#L19)
- `impl Copy for Chord` — [源码:19](../../../desktop/ui/src/shortcuts.rs#L19)
- `impl Eq for Chord` — [源码:19](../../../desktop/ui/src/shortcuts.rs#L19)
- `impl StructuralPartialEq for Chord` — [源码:19](../../../desktop/ui/src/shortcuts.rs#L19)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn matches(self, m: Modifiers) -> bool` | impl Chord | [源码:114](../../../desktop/ui/src/shortcuts.rs#L114) |  |
| `fn clone(&self) -> Chord` | impl Clone for Chord | [源码:19](../../../desktop/ui/src/shortcuts.rs#L19) | skin_editor_ui::shortcuts::Chord |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for Chord | [源码:19](../../../desktop/ui/src/shortcuts.rs#L19) |  |
| `fn eq(&self, other: &Chord) -> bool` | impl PartialEq for Chord | [源码:19](../../../desktop/ui/src/shortcuts.rs#L19) | skin_editor_ui::shortcuts::Chord |

## skin_editor_ui::shortcuts::FrameInput

[源码:136](../../../desktop/ui/src/shortcuts.rs#L136) · struct

```rust
pub(crate) struct FrameInput {
    bindings: Vec<usize>,
    blocked: bool,
    text_owned: bool,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_ui::shortcuts::Router::begin`

实现／约束（显式或 derive，无类继承）：

- `impl FrameInput` — [源码:193](../../../desktop/ui/src/shortcuts.rs#L193)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn early_cancel( &mut self, ctx: &Context, editing: bool, owned_draft: bool, ) -> bool` | impl FrameInput | [源码:196](../../../desktop/ui/src/shortcuts.rs#L196) |  |
| `fn consume(&self, ctx: &Context, n: usize)` | impl FrameInput | [源码:212](../../../desktop/ui/src/shortcuts.rs#L212) |  |
| `pub fn actions( &self, ctx: &Context, interaction: bool, canvas_focused: bool, ) -> Vec<EditorAction>` | impl FrameInput | [源码:217](../../../desktop/ui/src/shortcuts.rs#L217) | skin_editor_ui::EditorAction |

## skin_editor_ui::shortcuts::Router

[源码:131](../../../desktop/ui/src/shortcuts.rs#L131) · struct

```rust
pub(crate) struct Router {
    frame: Option<u64>,
    composing: bool,
    text_owned: bool,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_ui::EditorUi (声明)`, `skin_editor_ui::shortcuts::Router::default`

实现／约束（显式或 derive，无类继承）：

- `impl Router` — [源码:141](../../../desktop/ui/src/shortcuts.rs#L141)
- `impl Default for Router` — [源码:130](../../../desktop/ui/src/shortcuts.rs#L130)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn begin(&mut self, ctx: &Context) -> FrameInput` | impl Router | [源码:142](../../../desktop/ui/src/shortcuts.rs#L142) | skin_editor_ui::shortcuts::FrameInput |
| `pub fn finish(&mut self, ctx: &Context)` | impl Router | [源码:189](../../../desktop/ui/src/shortcuts.rs#L189) |  |
| `fn default() -> Router` | impl Default for Router | [源码:130](../../../desktop/ui/src/shortcuts.rs#L130) | skin_editor_ui::shortcuts::Router |

## skin_editor_ui::shortcuts::SELECTION_HINT

[源码:104](../../../desktop/ui/src/shortcuts.rs#L104) · constant

```rust
pub const SELECTION_HINT: &str = "Ctrl／Command 点击图层或组件可多选；拖动中 Esc 取消本次，空闲时 Esc 解除选择。";
```

## skin_editor_ui::shortcuts::TRANSFORM_HINT

[源码:103](../../../desktop/ui/src/shortcuts.rs#L103) · constant

```rust
pub const TRANSFORM_HINT: &str = "画布平移；角点缩放（Shift 等比）；圆柄旋转；Esc 取消。";
```

## skin_editor_ui::shortcuts::additive

[源码:107](../../../desktop/ui/src/shortcuts.rs#L107) · fn

```rust
pub(crate) fn additive(m: Modifiers) -> bool
```

## skin_editor_ui::shortcuts::proportional

[源码:110](../../../desktop/ui/src/shortcuts.rs#L110) · fn

```rust
pub(crate) fn proportional(m: Modifiers) -> bool
```

