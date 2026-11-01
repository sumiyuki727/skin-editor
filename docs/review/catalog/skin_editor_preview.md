# skin_editor_preview：编译器符号清单

快照：2026-10-30。来自含私有项的 rustdoc；显示编译器规范化可见域。含 derive 生成的实现，不把它们视作类继承。来源行可能指向宏定义／展开点。
引用列只列 rustdoc 解析出的项目类型链接，不能充当完整调用图；无链接不表示无引用。所有方法保留源码位置供逐项复核。测试和未启用的 cfg 分支另见源码索引。

## skin_editor_preview::PreviewState

[源码:9](../../../desktop/preview/src/lib.rs#L9) · struct

```rust
pub struct PreviewState {
    pub paused: bool,
    pub scene: Scene,
    pub auto_play: bool,
    pub fullscreen: bool,
    pub(crate) micros: u64,
    pub(crate) accepted_inputs: u64,
}
```

声明引用：`skin_editor_preview::Scene`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::Editor (声明)`, `skin_editor_desktop::editor_audio::window_recovery::Seed (声明)`, `skin_editor_preview::PreviewState::clone`

实现／约束（显式或 derive，无类继承）：

- `impl PreviewState` — [源码:29](../../../desktop/preview/src/lib.rs#L29)
- `impl Clone for PreviewState` — [源码:8](../../../desktop/preview/src/lib.rs#L8)
- `impl Debug for PreviewState` — [源码:8](../../../desktop/preview/src/lib.rs#L8)
- `impl Default for PreviewState` — [源码:17](../../../desktop/preview/src/lib.rs#L17)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn advance(&mut self, elapsed_micros: u64)` | impl PreviewState | [源码:30](../../../desktop/preview/src/lib.rs#L30) |  |
| `pub fn input(&mut self, interaction_tool: bool, canvas_focused: bool) -> bool` | impl PreviewState | [源码:35](../../../desktop/preview/src/lib.rs#L35) |  |
| `pub fn micros(&self) -> u64` | impl PreviewState | [源码:43](../../../desktop/preview/src/lib.rs#L43) |  |
| `pub fn accepted_inputs(&self) -> u64` | impl PreviewState | [源码:46](../../../desktop/preview/src/lib.rs#L46) |  |
| `fn clone(&self) -> PreviewState` | impl Clone for PreviewState | [源码:8](../../../desktop/preview/src/lib.rs#L8) | skin_editor_preview::PreviewState |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for PreviewState | [源码:8](../../../desktop/preview/src/lib.rs#L8) |  |
| `fn default() -> Self` | impl Default for PreviewState | [源码:18](../../../desktop/preview/src/lib.rs#L18) |  |

## skin_editor_preview::Scene

[源码:4](../../../desktop/preview/src/lib.rs#L4) · enum

```rust
pub enum Scene {
    Canvas,
    Playground,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_preview::PreviewState (声明)`, `skin_editor_preview::Scene::clone`, `skin_editor_preview::Scene::eq`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for Scene` — [源码:3](../../../desktop/preview/src/lib.rs#L3)
- `impl Debug for Scene` — [源码:3](../../../desktop/preview/src/lib.rs#L3)
- `impl PartialEq for Scene` — [源码:3](../../../desktop/preview/src/lib.rs#L3)
- `impl Copy for Scene` — [源码:3](../../../desktop/preview/src/lib.rs#L3)
- `impl Eq for Scene` — [源码:3](../../../desktop/preview/src/lib.rs#L3)
- `impl StructuralPartialEq for Scene` — [源码:3](../../../desktop/preview/src/lib.rs#L3)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> Scene` | impl Clone for Scene | [源码:3](../../../desktop/preview/src/lib.rs#L3) | skin_editor_preview::Scene |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for Scene | [源码:3](../../../desktop/preview/src/lib.rs#L3) |  |
| `fn eq(&self, other: &Scene) -> bool` | impl PartialEq for Scene | [源码:3](../../../desktop/preview/src/lib.rs#L3) | skin_editor_preview::Scene |

