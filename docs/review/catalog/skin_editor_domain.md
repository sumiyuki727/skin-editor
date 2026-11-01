# skin_editor_domain：编译器符号清单

快照：2026-10-30。来自含私有项的 rustdoc；显示编译器规范化可见域。含 derive 生成的实现，不把它们视作类继承。来源行可能指向宏定义／展开点。
引用列只列 rustdoc 解析出的项目类型链接，不能充当完整调用图；无链接不表示无引用。所有方法保留源码位置供逐项复核。测试和未启用的 cfg 分支另见源码索引。

## skin_editor_domain::Affine2

[源码:51](../../../crates/domain/src/geometry.rs#L51) · struct

```rust
pub struct Affine2([f64; 6]);
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::values::Affine2`, `skin_editor_domain::geometry::Affine2`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::Command (声明)`, `skin_editor_domain::Affine2::clone`, `skin_editor_domain::Affine2::compose`, `skin_editor_domain::Affine2::eq`, `skin_editor_domain::Affine2::inverse`, `skin_editor_domain::Affine2::new`, `skin_editor_domain::Affine2::rebased`, `skin_editor_domain::Bounds::transformed`, `skin_editor_domain::ComponentKind (声明)`, `skin_editor_domain::ComponentNode (声明)`, `skin_editor_domain::ComponentTree::document_content_transform`, `skin_editor_domain::ComponentTree::world_transform`, `skin_editor_domain::LayerNode (声明)`, `skin_editor_domain::LayerTree::world_transform`, `skin_editor_domain::SkinProject::document_content_transform`, `skin_editor_domain::SkinProject::document_workspace_transform`, `skin_editor_domain::components::ComponentStorage (声明)`, `skin_editor_domain::layers::LayerStorage (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Affine2` — [源码:53](../../../crates/domain/src/geometry.rs#L53)
- `impl Clone for Affine2` — [源码:50](../../../crates/domain/src/geometry.rs#L50)
- `impl Debug for Affine2` — [源码:50](../../../crates/domain/src/geometry.rs#L50)
- `impl PartialEq for Affine2` — [源码:50](../../../crates/domain/src/geometry.rs#L50)
- `impl Copy for Affine2` — [源码:50](../../../crates/domain/src/geometry.rs#L50)
- `impl StructuralPartialEq for Affine2` — [源码:50](../../../crates/domain/src/geometry.rs#L50)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn new(coefficients: [f64; 6]) -> Result<Self, ModelError>` | impl Affine2 | [源码:56](../../../crates/domain/src/geometry.rs#L56) | skin_editor_domain::Affine2, skin_editor_domain::ModelError |
| `pub const fn coefficients(self) -> [f64; 6]` | impl Affine2 | [源码:63](../../../crates/domain/src/geometry.rs#L63) |  |
| `pub fn compose(self, local: Self) -> Result<Self, ModelError>` | impl Affine2 | [源码:68](../../../crates/domain/src/geometry.rs#L68) | skin_editor_domain::Affine2, skin_editor_domain::ModelError |
| `pub fn inverse(self) -> Result<Self, ModelError>` | impl Affine2 | [源码:82](../../../crates/domain/src/geometry.rs#L82) | skin_editor_domain::Affine2, skin_editor_domain::ModelError |
| `pub fn rebased(world: Self, new_parent_world: Self) -> Result<Self, ModelError>` | impl Affine2 | [源码:100](../../../crates/domain/src/geometry.rs#L100) | skin_editor_domain::Affine2, skin_editor_domain::ModelError |
| `pub fn point(self, x: f64, y: f64) -> Result<[f64; 2], ModelError>` | impl Affine2 | [源码:104](../../../crates/domain/src/geometry.rs#L104) | skin_editor_domain::ModelError |
| `fn clone(&self) -> Affine2` | impl Clone for Affine2 | [源码:50](../../../crates/domain/src/geometry.rs#L50) | skin_editor_domain::Affine2 |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for Affine2 | [源码:50](../../../crates/domain/src/geometry.rs#L50) |  |
| `fn eq(&self, other: &Affine2) -> bool` | impl PartialEq for Affine2 | [源码:50](../../../crates/domain/src/geometry.rs#L50) | skin_editor_domain::Affine2 |

## skin_editor_domain::Asset

[源码:36](../../../crates/domain/src/assets.rs#L36) · struct

```rust
pub struct Asset {
    id: AssetId,
    name: String,
    blob: BlobRef,
    kind: AssetKind,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::values::Asset`, `skin_editor_domain::assets::Asset`

声明引用：`skin_editor_domain::AssetId`, `skin_editor_domain::AssetKind`, `skin_editor_domain::BlobRef`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::Command (声明)`, `skin_editor_application::history::Patch (声明)`, `skin_editor_domain::Asset::clone`, `skin_editor_domain::Asset::eq`, `skin_editor_domain::Asset::new`, `skin_editor_domain::AssetLibrary (声明)`, `skin_editor_domain::AssetLibrary::get`, `skin_editor_domain::AssetLibrary::iter`, `skin_editor_domain::AssetLibrary::new`, `skin_editor_domain::ProjectDefinition (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Asset` — [源码:43](../../../crates/domain/src/assets.rs#L43)
- `impl Clone for Asset` — [源码:35](../../../crates/domain/src/assets.rs#L35)
- `impl Debug for Asset` — [源码:35](../../../crates/domain/src/assets.rs#L35)
- `impl PartialEq for Asset` — [源码:35](../../../crates/domain/src/assets.rs#L35)
- `impl Eq for Asset` — [源码:35](../../../crates/domain/src/assets.rs#L35)
- `impl StructuralPartialEq for Asset` — [源码:35](../../../crates/domain/src/assets.rs#L35)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn new( id: AssetId, name: String, blob: BlobRef, kind: AssetKind, ) -> Result<Self, ModelError>` | impl Asset | [源码:44](../../../crates/domain/src/assets.rs#L44) | skin_editor_domain::Asset, skin_editor_domain::AssetId, skin_editor_domain::AssetKind, skin_editor_domain::BlobRef, skin_editor_domain::ModelError |
| `pub const fn id(&self) -> AssetId` | impl Asset | [源码:61](../../../crates/domain/src/assets.rs#L61) | skin_editor_domain::AssetId |
| `pub fn name(&self) -> &str` | impl Asset | [源码:65](../../../crates/domain/src/assets.rs#L65) |  |
| `pub const fn blob(&self) -> BlobRef` | impl Asset | [源码:69](../../../crates/domain/src/assets.rs#L69) | skin_editor_domain::BlobRef |
| `pub fn kind(&self) -> &AssetKind` | impl Asset | [源码:73](../../../crates/domain/src/assets.rs#L73) | skin_editor_domain::AssetKind |
| `fn clone(&self) -> Asset` | impl Clone for Asset | [源码:35](../../../crates/domain/src/assets.rs#L35) | skin_editor_domain::Asset |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for Asset | [源码:35](../../../crates/domain/src/assets.rs#L35) |  |
| `fn eq(&self, other: &Asset) -> bool` | impl PartialEq for Asset | [源码:35](../../../crates/domain/src/assets.rs#L35) | skin_editor_domain::Asset |

## skin_editor_domain::AssetId

[源码:23](../../../crates/domain/src/identity.rs#L23) · struct

```rust
pub struct AssetId(NonZeroU128);
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::values::AssetId`, `skin_editor_domain::identity::AssetId`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::AssetAudition (声明)`, `skin_editor_application::AssetAuditionState (声明)`, `skin_editor_application::AudioEditorAction (声明)`, `skin_editor_application::Command (声明)`, `skin_editor_application::CommandFailure (声明)`, `skin_editor_application::history::Patch (声明)`, `skin_editor_domain::Asset (声明)`, `skin_editor_domain::Asset::id`, `skin_editor_domain::Asset::new`, `skin_editor_domain::AssetId::clone`, `skin_editor_domain::AssetId::cmp`, `skin_editor_domain::AssetId::eq`, `skin_editor_domain::AssetId::new`, `skin_editor_domain::AssetId::partial_cmp`, `skin_editor_domain::AssetLibrary (声明)`, `skin_editor_domain::AssetLibrary::get`, `skin_editor_domain::AudioComponent (声明)`, `skin_editor_domain::AudioComponent::asset`, `skin_editor_domain::AudioComponent::new`, `skin_editor_domain::LayerKind (声明)`, `skin_editor_domain::ObjectId (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl AssetId` — [源码:23](../../../crates/domain/src/identity.rs#L23)
- `impl Clone for AssetId` — [源码:23](../../../crates/domain/src/identity.rs#L23)
- `impl Debug for AssetId` — [源码:23](../../../crates/domain/src/identity.rs#L23)
- `impl Hash for AssetId` — [源码:23](../../../crates/domain/src/identity.rs#L23)
- `impl Ord for AssetId` — [源码:23](../../../crates/domain/src/identity.rs#L23)
- `impl PartialEq for AssetId` — [源码:23](../../../crates/domain/src/identity.rs#L23)
- `impl PartialOrd for AssetId` — [源码:23](../../../crates/domain/src/identity.rs#L23)
- `impl Copy for AssetId` — [源码:23](../../../crates/domain/src/identity.rs#L23)
- `impl Eq for AssetId` — [源码:23](../../../crates/domain/src/identity.rs#L23)
- `impl StructuralPartialEq for AssetId` — [源码:23](../../../crates/domain/src/identity.rs#L23)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub const fn new(value: u128) -> Option<Self>` | impl AssetId | [源码:23](../../../crates/domain/src/identity.rs#L23) | skin_editor_domain::AssetId |
| `pub const fn get(self) -> u128` | impl AssetId | [源码:23](../../../crates/domain/src/identity.rs#L23) |  |
| `fn clone(&self) -> AssetId` | impl Clone for AssetId | [源码:23](../../../crates/domain/src/identity.rs#L23) | skin_editor_domain::AssetId |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for AssetId | [源码:23](../../../crates/domain/src/identity.rs#L23) |  |
| `fn hash<__H: Hasher>(&self, state: &mut __H)` | impl Hash for AssetId | [源码:23](../../../crates/domain/src/identity.rs#L23) |  |
| `fn cmp(&self, other: &AssetId) -> Ordering` | impl Ord for AssetId | [源码:23](../../../crates/domain/src/identity.rs#L23) | skin_editor_domain::AssetId |
| `fn eq(&self, other: &AssetId) -> bool` | impl PartialEq for AssetId | [源码:23](../../../crates/domain/src/identity.rs#L23) | skin_editor_domain::AssetId |
| `fn partial_cmp(&self, other: &AssetId) -> Option<Ordering>` | impl PartialOrd for AssetId | [源码:23](../../../crates/domain/src/identity.rs#L23) | skin_editor_domain::AssetId |

## skin_editor_domain::AssetKind

[源码:27](../../../crates/domain/src/assets.rs#L27) · enum

```rust
pub enum AssetKind {
    RasterImage(PixelSize),
    Audio(AudioMetadata),
    Opaque,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::values::AssetKind`, `skin_editor_domain::assets::AssetKind`

声明引用：`skin_editor_domain::AudioMetadata`, `skin_editor_domain::PixelSize`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_domain::Asset (声明)`, `skin_editor_domain::Asset::kind`, `skin_editor_domain::Asset::new`, `skin_editor_domain::AssetKind::clone`, `skin_editor_domain::AssetKind::eq`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for AssetKind` — [源码:26](../../../crates/domain/src/assets.rs#L26)
- `impl Debug for AssetKind` — [源码:26](../../../crates/domain/src/assets.rs#L26)
- `impl PartialEq for AssetKind` — [源码:26](../../../crates/domain/src/assets.rs#L26)
- `impl Eq for AssetKind` — [源码:26](../../../crates/domain/src/assets.rs#L26)
- `impl StructuralPartialEq for AssetKind` — [源码:26](../../../crates/domain/src/assets.rs#L26)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> AssetKind` | impl Clone for AssetKind | [源码:26](../../../crates/domain/src/assets.rs#L26) | skin_editor_domain::AssetKind |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for AssetKind | [源码:26](../../../crates/domain/src/assets.rs#L26) |  |
| `fn eq(&self, other: &AssetKind) -> bool` | impl PartialEq for AssetKind | [源码:26](../../../crates/domain/src/assets.rs#L26) | skin_editor_domain::AssetKind |

## skin_editor_domain::AssetLibrary

[源码:79](../../../crates/domain/src/assets.rs#L79) · struct

```rust
pub struct AssetLibrary {
    assets: BTreeMap<AssetId, Asset>,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_domain::assets::AssetLibrary`

声明引用：`skin_editor_domain::Asset`, `skin_editor_domain::AssetId`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_domain::AssetLibrary::clone`, `skin_editor_domain::AssetLibrary::default`, `skin_editor_domain::AssetLibrary::eq`, `skin_editor_domain::SkinProject (声明)`, `skin_editor_domain::SkinProject::assets`

实现／约束（显式或 derive，无类继承）：

- `impl AssetLibrary` — [源码:83](../../../crates/domain/src/assets.rs#L83)
- `impl Clone for AssetLibrary` — [源码:78](../../../crates/domain/src/assets.rs#L78)
- `impl Debug for AssetLibrary` — [源码:78](../../../crates/domain/src/assets.rs#L78)
- `impl Default for AssetLibrary` — [源码:78](../../../crates/domain/src/assets.rs#L78)
- `impl PartialEq for AssetLibrary` — [源码:78](../../../crates/domain/src/assets.rs#L78)
- `impl Eq for AssetLibrary` — [源码:78](../../../crates/domain/src/assets.rs#L78)
- `impl StructuralPartialEq for AssetLibrary` — [源码:78](../../../crates/domain/src/assets.rs#L78)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn new(assets: Vec<Asset>) -> Result<Self, ModelError>` | impl AssetLibrary | [源码:84](../../../crates/domain/src/assets.rs#L84) | skin_editor_domain::Asset, skin_editor_domain::ModelError |
| `pub fn get(&self, id: AssetId) -> Option<&Asset>` | impl AssetLibrary | [源码:118](../../../crates/domain/src/assets.rs#L118) | skin_editor_domain::Asset, skin_editor_domain::AssetId |
| `pub fn iter(&self) -> impl ExactSizeIterator<Item = &Asset>` | impl AssetLibrary | [源码:122](../../../crates/domain/src/assets.rs#L122) | skin_editor_domain::Asset |
| `fn clone(&self) -> AssetLibrary` | impl Clone for AssetLibrary | [源码:78](../../../crates/domain/src/assets.rs#L78) | skin_editor_domain::AssetLibrary |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for AssetLibrary | [源码:78](../../../crates/domain/src/assets.rs#L78) |  |
| `fn default() -> AssetLibrary` | impl Default for AssetLibrary | [源码:78](../../../crates/domain/src/assets.rs#L78) | skin_editor_domain::AssetLibrary |
| `fn eq(&self, other: &AssetLibrary) -> bool` | impl PartialEq for AssetLibrary | [源码:78](../../../crates/domain/src/assets.rs#L78) | skin_editor_domain::AssetLibrary |

## skin_editor_domain::AudioComponent

[源码:60](../../../crates/domain/src/audio.rs#L60) · struct

```rust
pub struct AudioComponent {
    id: AudioComponentId,
    slot: SoundSlotKey,
    asset: AssetId,
    gain: Gain,
    visible: bool,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::values::AudioComponent`, `skin_editor_domain::audio::AudioComponent`

声明引用：`skin_editor_domain::AssetId`, `skin_editor_domain::AudioComponentId`, `skin_editor_domain::Gain`, `skin_editor_domain::SoundSlotKey`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::Command (声明)`, `skin_editor_application::history::Patch (声明)`, `skin_editor_domain::AudioComponent::clone`, `skin_editor_domain::AudioComponent::eq`, `skin_editor_domain::AudioComponent::new`, `skin_editor_domain::AudioComponent::with_properties`, `skin_editor_domain::ProjectDefinition (声明)`, `skin_editor_domain::SkinProject (声明)`, `skin_editor_domain::SkinProject::audio_component`, `skin_editor_domain::SkinProject::audio_components`

实现／约束（显式或 derive，无类继承）：

- `impl AudioComponent` — [源码:67](../../../crates/domain/src/audio.rs#L67)
- `impl Clone for AudioComponent` — [源码:59](../../../crates/domain/src/audio.rs#L59)
- `impl Debug for AudioComponent` — [源码:59](../../../crates/domain/src/audio.rs#L59)
- `impl PartialEq for AudioComponent` — [源码:59](../../../crates/domain/src/audio.rs#L59)
- `impl StructuralPartialEq for AudioComponent` — [源码:59](../../../crates/domain/src/audio.rs#L59)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn new(id: AudioComponentId, slot: SoundSlotKey, asset: AssetId) -> Self` | impl AudioComponent | [源码:68](../../../crates/domain/src/audio.rs#L68) | skin_editor_domain::AssetId, skin_editor_domain::AudioComponent, skin_editor_domain::AudioComponentId, skin_editor_domain::SoundSlotKey |
| `pub fn with_properties(self, gain: Gain, visible: bool) -> Self` | impl AudioComponent | [源码:78](../../../crates/domain/src/audio.rs#L78) | skin_editor_domain::AudioComponent, skin_editor_domain::Gain |
| `pub const fn id(&self) -> AudioComponentId` | impl AudioComponent | [源码:83](../../../crates/domain/src/audio.rs#L83) | skin_editor_domain::AudioComponentId |
| `pub fn slot(&self) -> &SoundSlotKey` | impl AudioComponent | [源码:86](../../../crates/domain/src/audio.rs#L86) | skin_editor_domain::SoundSlotKey |
| `pub const fn asset(&self) -> AssetId` | impl AudioComponent | [源码:89](../../../crates/domain/src/audio.rs#L89) | skin_editor_domain::AssetId |
| `pub const fn gain(&self) -> Gain` | impl AudioComponent | [源码:92](../../../crates/domain/src/audio.rs#L92) | skin_editor_domain::Gain |
| `pub const fn visible(&self) -> bool` | impl AudioComponent | [源码:95](../../../crates/domain/src/audio.rs#L95) |  |
| `fn clone(&self) -> AudioComponent` | impl Clone for AudioComponent | [源码:59](../../../crates/domain/src/audio.rs#L59) | skin_editor_domain::AudioComponent |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for AudioComponent | [源码:59](../../../crates/domain/src/audio.rs#L59) |  |
| `fn eq(&self, other: &AudioComponent) -> bool` | impl PartialEq for AudioComponent | [源码:59](../../../crates/domain/src/audio.rs#L59) | skin_editor_domain::AudioComponent |

## skin_editor_domain::AudioComponentId

[源码:28](../../../crates/domain/src/identity.rs#L28) · struct

```rust
pub struct AudioComponentId(NonZeroU128);
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::values::AudioComponentId`, `skin_editor_domain::identity::AudioComponentId`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::AudioComponentState (声明)`, `skin_editor_application::AudioEditorAction (声明)`, `skin_editor_application::Command (声明)`, `skin_editor_application::CommandFailure (声明)`, `skin_editor_application::history::Patch (声明)`, `skin_editor_domain::AudioComponent (声明)`, `skin_editor_domain::AudioComponent::id`, `skin_editor_domain::AudioComponent::new`, `skin_editor_domain::AudioComponentId::clone`, `skin_editor_domain::AudioComponentId::cmp`, `skin_editor_domain::AudioComponentId::eq`, `skin_editor_domain::AudioComponentId::new`, `skin_editor_domain::AudioComponentId::partial_cmp`, `skin_editor_domain::ObjectId (声明)`, `skin_editor_domain::OutputMapping::for_audio`, `skin_editor_domain::OutputSource (声明)`, `skin_editor_domain::SkinProject (声明)`, `skin_editor_domain::SkinProject::audio_component`

实现／约束（显式或 derive，无类继承）：

- `impl AudioComponentId` — [源码:28](../../../crates/domain/src/identity.rs#L28)
- `impl Clone for AudioComponentId` — [源码:28](../../../crates/domain/src/identity.rs#L28)
- `impl Debug for AudioComponentId` — [源码:28](../../../crates/domain/src/identity.rs#L28)
- `impl Hash for AudioComponentId` — [源码:28](../../../crates/domain/src/identity.rs#L28)
- `impl Ord for AudioComponentId` — [源码:28](../../../crates/domain/src/identity.rs#L28)
- `impl PartialEq for AudioComponentId` — [源码:28](../../../crates/domain/src/identity.rs#L28)
- `impl PartialOrd for AudioComponentId` — [源码:28](../../../crates/domain/src/identity.rs#L28)
- `impl Copy for AudioComponentId` — [源码:28](../../../crates/domain/src/identity.rs#L28)
- `impl Eq for AudioComponentId` — [源码:28](../../../crates/domain/src/identity.rs#L28)
- `impl StructuralPartialEq for AudioComponentId` — [源码:28](../../../crates/domain/src/identity.rs#L28)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub const fn new(value: u128) -> Option<Self>` | impl AudioComponentId | [源码:28](../../../crates/domain/src/identity.rs#L28) | skin_editor_domain::AudioComponentId |
| `pub const fn get(self) -> u128` | impl AudioComponentId | [源码:28](../../../crates/domain/src/identity.rs#L28) |  |
| `fn clone(&self) -> AudioComponentId` | impl Clone for AudioComponentId | [源码:28](../../../crates/domain/src/identity.rs#L28) | skin_editor_domain::AudioComponentId |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for AudioComponentId | [源码:28](../../../crates/domain/src/identity.rs#L28) |  |
| `fn hash<__H: Hasher>(&self, state: &mut __H)` | impl Hash for AudioComponentId | [源码:28](../../../crates/domain/src/identity.rs#L28) |  |
| `fn cmp(&self, other: &AudioComponentId) -> Ordering` | impl Ord for AudioComponentId | [源码:28](../../../crates/domain/src/identity.rs#L28) | skin_editor_domain::AudioComponentId |
| `fn eq(&self, other: &AudioComponentId) -> bool` | impl PartialEq for AudioComponentId | [源码:28](../../../crates/domain/src/identity.rs#L28) | skin_editor_domain::AudioComponentId |
| `fn partial_cmp(&self, other: &AudioComponentId) -> Option<Ordering>` | impl PartialOrd for AudioComponentId | [源码:28](../../../crates/domain/src/identity.rs#L28) | skin_editor_domain::AudioComponentId |

## skin_editor_domain::AudioMetadata

[源码:5](../../../crates/domain/src/audio.rs#L5) · struct

```rust
pub struct AudioMetadata {
    sample_rate: u32,
    channels: u16,
    frames: u64,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::values::AudioMetadata`, `skin_editor_domain::audio::AudioMetadata`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::AssetAudition (声明)`, `skin_editor_application::ResourceStore (声明)`, `skin_editor_application::ResourceStore::audio_metadata`, `skin_editor_application::resources::NoResources::audio_metadata`, `skin_editor_domain::AssetKind (声明)`, `skin_editor_domain::AudioMetadata::clone`, `skin_editor_domain::AudioMetadata::eq`, `skin_editor_domain::AudioMetadata::new`

实现／约束（显式或 derive，无类继承）：

- `impl AudioMetadata` — [源码:10](../../../crates/domain/src/audio.rs#L10)
- `impl Clone for AudioMetadata` — [源码:4](../../../crates/domain/src/audio.rs#L4)
- `impl Debug for AudioMetadata` — [源码:4](../../../crates/domain/src/audio.rs#L4)
- `impl PartialEq for AudioMetadata` — [源码:4](../../../crates/domain/src/audio.rs#L4)
- `impl Copy for AudioMetadata` — [源码:4](../../../crates/domain/src/audio.rs#L4)
- `impl Eq for AudioMetadata` — [源码:4](../../../crates/domain/src/audio.rs#L4)
- `impl StructuralPartialEq for AudioMetadata` — [源码:4](../../../crates/domain/src/audio.rs#L4)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn new( sample_rate: u32, channels: u16, frames: u64, ) -> Result<Self, ModelError>` | impl AudioMetadata | [源码:11](../../../crates/domain/src/audio.rs#L11) | skin_editor_domain::AudioMetadata, skin_editor_domain::ModelError |
| `pub const fn sample_rate(self) -> u32` | impl AudioMetadata | [源码:21](../../../crates/domain/src/audio.rs#L21) |  |
| `pub const fn channels(self) -> u16` | impl AudioMetadata | [源码:24](../../../crates/domain/src/audio.rs#L24) |  |
| `pub const fn frames(self) -> u64` | impl AudioMetadata | [源码:27](../../../crates/domain/src/audio.rs#L27) |  |
| `fn clone(&self) -> AudioMetadata` | impl Clone for AudioMetadata | [源码:4](../../../crates/domain/src/audio.rs#L4) | skin_editor_domain::AudioMetadata |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for AudioMetadata | [源码:4](../../../crates/domain/src/audio.rs#L4) |  |
| `fn eq(&self, other: &AudioMetadata) -> bool` | impl PartialEq for AudioMetadata | [源码:4](../../../crates/domain/src/audio.rs#L4) | skin_editor_domain::AudioMetadata |

## skin_editor_domain::BlobRef

[源码:7](../../../crates/domain/src/assets.rs#L7) · struct

```rust
pub struct BlobRef {
    sha256: [u8; 32],
    byte_len: u64,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::values::BlobRef`, `skin_editor_domain::assets::BlobRef`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::AssetAudition (声明)`, `skin_editor_application::BlobLease (声明)`, `skin_editor_application::BlobLease::reference`, `skin_editor_application::CommandError (声明)`, `skin_editor_application::HistoryError (声明)`, `skin_editor_application::ProjectSession (声明)`, `skin_editor_application::ProjectSession::pin_candidate`, `skin_editor_application::ProjectSession::publish`, `skin_editor_application::ResourceStore (声明)`, `skin_editor_application::ResourceStore::audio_metadata`, `skin_editor_application::ResourceStore::pin`, `skin_editor_application::StagedRaster (声明)`, `skin_editor_application::draft::Draft (声明)`, `skin_editor_application::resources::NoResources::audio_metadata`, `skin_editor_application::resources::NoResources::pin`, `skin_editor_domain::Asset (声明)`, `skin_editor_domain::Asset::blob`, `skin_editor_domain::Asset::new`, `skin_editor_domain::BlobRef::clone`, `skin_editor_domain::BlobRef::cmp`, `skin_editor_domain::BlobRef::eq`, `skin_editor_domain::BlobRef::new`, `skin_editor_domain::BlobRef::partial_cmp`

实现／约束（显式或 derive，无类继承）：

- `impl BlobRef` — [源码:12](../../../crates/domain/src/assets.rs#L12)
- `impl Clone for BlobRef` — [源码:6](../../../crates/domain/src/assets.rs#L6)
- `impl Debug for BlobRef` — [源码:6](../../../crates/domain/src/assets.rs#L6)
- `impl Hash for BlobRef` — [源码:6](../../../crates/domain/src/assets.rs#L6)
- `impl Ord for BlobRef` — [源码:6](../../../crates/domain/src/assets.rs#L6)
- `impl PartialEq for BlobRef` — [源码:6](../../../crates/domain/src/assets.rs#L6)
- `impl PartialOrd for BlobRef` — [源码:6](../../../crates/domain/src/assets.rs#L6)
- `impl Copy for BlobRef` — [源码:6](../../../crates/domain/src/assets.rs#L6)
- `impl Eq for BlobRef` — [源码:6](../../../crates/domain/src/assets.rs#L6)
- `impl StructuralPartialEq for BlobRef` — [源码:6](../../../crates/domain/src/assets.rs#L6)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub const fn new(sha256: [u8; 32], byte_len: u64) -> Self` | impl BlobRef | [源码:13](../../../crates/domain/src/assets.rs#L13) | skin_editor_domain::BlobRef |
| `pub const fn sha256(self) -> [u8; 32]` | impl BlobRef | [源码:17](../../../crates/domain/src/assets.rs#L17) |  |
| `pub const fn byte_len(self) -> u64` | impl BlobRef | [源码:21](../../../crates/domain/src/assets.rs#L21) |  |
| `fn clone(&self) -> BlobRef` | impl Clone for BlobRef | [源码:6](../../../crates/domain/src/assets.rs#L6) | skin_editor_domain::BlobRef |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for BlobRef | [源码:6](../../../crates/domain/src/assets.rs#L6) |  |
| `fn hash<__H: Hasher>(&self, state: &mut __H)` | impl Hash for BlobRef | [源码:6](../../../crates/domain/src/assets.rs#L6) |  |
| `fn cmp(&self, other: &BlobRef) -> Ordering` | impl Ord for BlobRef | [源码:6](../../../crates/domain/src/assets.rs#L6) | skin_editor_domain::BlobRef |
| `fn eq(&self, other: &BlobRef) -> bool` | impl PartialEq for BlobRef | [源码:6](../../../crates/domain/src/assets.rs#L6) | skin_editor_domain::BlobRef |
| `fn partial_cmp(&self, other: &BlobRef) -> Option<Ordering>` | impl PartialOrd for BlobRef | [源码:6](../../../crates/domain/src/assets.rs#L6) | skin_editor_domain::BlobRef |

## skin_editor_domain::Bounds

[源码:116](../../../crates/domain/src/geometry.rs#L116) · struct

```rust
pub struct Bounds {
    min: [f64; 2],
    max: [f64; 2],
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::values::Bounds`, `skin_editor_domain::geometry::Bounds`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_domain::Bounds::clone`, `skin_editor_domain::Bounds::eq`, `skin_editor_domain::Bounds::transformed`, `skin_editor_domain::Bounds::union`, `skin_editor_domain::SkinProject::component_bounds`, `skin_editor_domain::SkinProject::layer_bounds`

实现／约束（显式或 derive，无类继承）：

- `impl Bounds` — [源码:121](../../../crates/domain/src/geometry.rs#L121)
- `impl Clone for Bounds` — [源码:115](../../../crates/domain/src/geometry.rs#L115)
- `impl Debug for Bounds` — [源码:115](../../../crates/domain/src/geometry.rs#L115)
- `impl PartialEq for Bounds` — [源码:115](../../../crates/domain/src/geometry.rs#L115)
- `impl Copy for Bounds` — [源码:115](../../../crates/domain/src/geometry.rs#L115)
- `impl StructuralPartialEq for Bounds` — [源码:115](../../../crates/domain/src/geometry.rs#L115)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn transformed( size: PixelSize, transform: Affine2, ) -> Result<Self, ModelError>` | impl Bounds | [源码:122](../../../crates/domain/src/geometry.rs#L122) | skin_editor_domain::Affine2, skin_editor_domain::Bounds, skin_editor_domain::ModelError, skin_editor_domain::PixelSize |
| `pub fn union(self, other: Self) -> Self` | impl Bounds | [源码:144](../../../crates/domain/src/geometry.rs#L144) | skin_editor_domain::Bounds |
| `pub const fn min(self) -> [f64; 2]` | impl Bounds | [源码:151](../../../crates/domain/src/geometry.rs#L151) |  |
| `pub const fn max(self) -> [f64; 2]` | impl Bounds | [源码:155](../../../crates/domain/src/geometry.rs#L155) |  |
| `fn clone(&self) -> Bounds` | impl Clone for Bounds | [源码:115](../../../crates/domain/src/geometry.rs#L115) | skin_editor_domain::Bounds |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for Bounds | [源码:115](../../../crates/domain/src/geometry.rs#L115) |  |
| `fn eq(&self, other: &Bounds) -> bool` | impl PartialEq for Bounds | [源码:115](../../../crates/domain/src/geometry.rs#L115) | skin_editor_domain::Bounds |

## skin_editor_domain::ComponentId

[源码:26](../../../crates/domain/src/identity.rs#L26) · struct

```rust
pub struct ComponentId(NonZeroU128);
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::values::ComponentId`, `skin_editor_domain::identity::ComponentId`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::Command (声明)`, `skin_editor_application::execute::ComponentNodes (声明)`, `skin_editor_application::execute::component_children (声明)`, `skin_editor_application::execute::component_node (声明)`, `skin_editor_application::execute::components (声明)`, `skin_editor_domain::ComponentId::clone`, `skin_editor_domain::ComponentId::cmp`, `skin_editor_domain::ComponentId::eq`, `skin_editor_domain::ComponentId::new`, `skin_editor_domain::ComponentId::partial_cmp`, `skin_editor_domain::ComponentKind (声明)`, `skin_editor_domain::ComponentNode (声明)`, `skin_editor_domain::ComponentNode::children`, `skin_editor_domain::ComponentTree::member`, `skin_editor_domain::ComponentTree::new`, `skin_editor_domain::ComponentTree::node`, `skin_editor_domain::ComponentTree::parent`, `skin_editor_domain::ComponentTree::roots`, `skin_editor_domain::ComponentTree::world_transform`, `skin_editor_domain::ObjectId (声明)`, `skin_editor_domain::SkinProject::component_bounds`, `skin_editor_domain::components::ComponentStorage (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl ComponentId` — [源码:26](../../../crates/domain/src/identity.rs#L26)
- `impl Clone for ComponentId` — [源码:26](../../../crates/domain/src/identity.rs#L26)
- `impl Debug for ComponentId` — [源码:26](../../../crates/domain/src/identity.rs#L26)
- `impl Hash for ComponentId` — [源码:26](../../../crates/domain/src/identity.rs#L26)
- `impl Ord for ComponentId` — [源码:26](../../../crates/domain/src/identity.rs#L26)
- `impl PartialEq for ComponentId` — [源码:26](../../../crates/domain/src/identity.rs#L26)
- `impl PartialOrd for ComponentId` — [源码:26](../../../crates/domain/src/identity.rs#L26)
- `impl Copy for ComponentId` — [源码:26](../../../crates/domain/src/identity.rs#L26)
- `impl Eq for ComponentId` — [源码:26](../../../crates/domain/src/identity.rs#L26)
- `impl StructuralPartialEq for ComponentId` — [源码:26](../../../crates/domain/src/identity.rs#L26)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub const fn new(value: u128) -> Option<Self>` | impl ComponentId | [源码:26](../../../crates/domain/src/identity.rs#L26) | skin_editor_domain::ComponentId |
| `pub const fn get(self) -> u128` | impl ComponentId | [源码:26](../../../crates/domain/src/identity.rs#L26) |  |
| `fn clone(&self) -> ComponentId` | impl Clone for ComponentId | [源码:26](../../../crates/domain/src/identity.rs#L26) | skin_editor_domain::ComponentId |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for ComponentId | [源码:26](../../../crates/domain/src/identity.rs#L26) |  |
| `fn hash<__H: Hasher>(&self, state: &mut __H)` | impl Hash for ComponentId | [源码:26](../../../crates/domain/src/identity.rs#L26) |  |
| `fn cmp(&self, other: &ComponentId) -> Ordering` | impl Ord for ComponentId | [源码:26](../../../crates/domain/src/identity.rs#L26) | skin_editor_domain::ComponentId |
| `fn eq(&self, other: &ComponentId) -> bool` | impl PartialEq for ComponentId | [源码:26](../../../crates/domain/src/identity.rs#L26) | skin_editor_domain::ComponentId |
| `fn partial_cmp(&self, other: &ComponentId) -> Option<Ordering>` | impl PartialOrd for ComponentId | [源码:26](../../../crates/domain/src/identity.rs#L26) | skin_editor_domain::ComponentId |

## skin_editor_domain::ComponentKind

[源码:8](../../../crates/domain/src/components.rs#L8) · enum

```rust
pub enum ComponentKind {
    Group {
        children: Vec<ComponentId>,
    },
    Element {
        document: DocumentId,
        document_to_component: Affine2,
    },
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::values::ComponentKind`, `skin_editor_domain::components::ComponentKind`

声明引用：`skin_editor_domain::Affine2`, `skin_editor_domain::ComponentId`, `skin_editor_domain::DocumentId`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_domain::ComponentKind::clone`, `skin_editor_domain::ComponentKind::eq`, `skin_editor_domain::ComponentNode (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for ComponentKind` — [源码:7](../../../crates/domain/src/components.rs#L7)
- `impl Debug for ComponentKind` — [源码:7](../../../crates/domain/src/components.rs#L7)
- `impl PartialEq for ComponentKind` — [源码:7](../../../crates/domain/src/components.rs#L7)
- `impl StructuralPartialEq for ComponentKind` — [源码:7](../../../crates/domain/src/components.rs#L7)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> ComponentKind` | impl Clone for ComponentKind | [源码:7](../../../crates/domain/src/components.rs#L7) | skin_editor_domain::ComponentKind |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for ComponentKind | [源码:7](../../../crates/domain/src/components.rs#L7) |  |
| `fn eq(&self, other: &ComponentKind) -> bool` | impl PartialEq for ComponentKind | [源码:7](../../../crates/domain/src/components.rs#L7) | skin_editor_domain::ComponentKind |

## skin_editor_domain::ComponentNode

[源码:21](../../../crates/domain/src/components.rs#L21) · struct

```rust
pub struct ComponentNode {
    pub id: ComponentId,
    pub local: Affine2,
    pub kind: ComponentKind,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::values::ComponentNode`, `skin_editor_domain::components::ComponentNode`

声明引用：`skin_editor_domain::Affine2`, `skin_editor_domain::ComponentId`, `skin_editor_domain::ComponentKind`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::Command (声明)`, `skin_editor_application::execute::ComponentNodes (声明)`, `skin_editor_application::execute::component_children (声明)`, `skin_editor_application::execute::component_node (声明)`, `skin_editor_application::execute::components (声明)`, `skin_editor_domain::ComponentNode::clone`, `skin_editor_domain::ComponentNode::eq`, `skin_editor_domain::ComponentTree::iter`, `skin_editor_domain::ComponentTree::new`, `skin_editor_domain::ComponentTree::node`, `skin_editor_domain::components::ComponentStorage (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl ComponentNode` — [源码:27](../../../crates/domain/src/components.rs#L27)
- `impl Clone for ComponentNode` — [源码:20](../../../crates/domain/src/components.rs#L20)
- `impl Debug for ComponentNode` — [源码:20](../../../crates/domain/src/components.rs#L20)
- `impl PartialEq for ComponentNode` — [源码:20](../../../crates/domain/src/components.rs#L20)
- `impl StructuralPartialEq for ComponentNode` — [源码:20](../../../crates/domain/src/components.rs#L20)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn children(&self) -> &[ComponentId]` | impl ComponentNode | [源码:28](../../../crates/domain/src/components.rs#L28) | skin_editor_domain::ComponentId |
| `fn clone(&self) -> ComponentNode` | impl Clone for ComponentNode | [源码:20](../../../crates/domain/src/components.rs#L20) | skin_editor_domain::ComponentNode |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for ComponentNode | [源码:20](../../../crates/domain/src/components.rs#L20) |  |
| `fn eq(&self, other: &ComponentNode) -> bool` | impl PartialEq for ComponentNode | [源码:20](../../../crates/domain/src/components.rs#L20) | skin_editor_domain::ComponentNode |

## skin_editor_domain::ComponentTree

[源码:37](../../../crates/domain/src/components.rs#L37) · struct

```rust
pub struct ComponentTree {
    data: Arc<ComponentStorage>,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_domain::components::ComponentTree`

声明引用：`skin_editor_domain::components::ComponentStorage`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::execute::components (声明)`, `skin_editor_application::history::Patch (声明)`, `skin_editor_domain::ComponentTree::clone`, `skin_editor_domain::ComponentTree::default`, `skin_editor_domain::ComponentTree::eq`, `skin_editor_domain::ProjectDefinition (声明)`, `skin_editor_domain::SkinProject (声明)`, `skin_editor_domain::SkinProject::components`

实现／约束（显式或 derive，无类继承）：

- `impl ComponentTree` — [源码:51](../../../crates/domain/src/components.rs#L51)
- `impl Clone for ComponentTree` — [源码:36](../../../crates/domain/src/components.rs#L36)
- `impl Debug for ComponentTree` — [源码:36](../../../crates/domain/src/components.rs#L36)
- `impl Default for ComponentTree` — [源码:36](../../../crates/domain/src/components.rs#L36)
- `impl PartialEq for ComponentTree` — [源码:36](../../../crates/domain/src/components.rs#L36)
- `impl StructuralPartialEq for ComponentTree` — [源码:36](../../../crates/domain/src/components.rs#L36)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn new( roots: Vec<ComponentId>, definitions: Vec<ComponentNode>, ) -> Result<Self, ModelError>` | impl ComponentTree | [源码:52](../../../crates/domain/src/components.rs#L52) | skin_editor_domain::ComponentId, skin_editor_domain::ComponentNode, skin_editor_domain::ModelError |
| `pub fn roots(&self) -> &[ComponentId]` | impl ComponentTree | [源码:108](../../../crates/domain/src/components.rs#L108) | skin_editor_domain::ComponentId |
| `pub fn node(&self, id: ComponentId) -> Option<&ComponentNode>` | impl ComponentTree | [源码:112](../../../crates/domain/src/components.rs#L112) | skin_editor_domain::ComponentId, skin_editor_domain::ComponentNode |
| `pub fn parent(&self, id: ComponentId) -> Option<Option<ComponentId>>` | impl ComponentTree | [源码:116](../../../crates/domain/src/components.rs#L116) | skin_editor_domain::ComponentId |
| `pub fn world_transform(&self, id: ComponentId) -> Option<Affine2>` | impl ComponentTree | [源码:120](../../../crates/domain/src/components.rs#L120) | skin_editor_domain::Affine2, skin_editor_domain::ComponentId |
| `pub fn member(&self, document: DocumentId) -> Option<ComponentId>` | impl ComponentTree | [源码:124](../../../crates/domain/src/components.rs#L124) | skin_editor_domain::ComponentId, skin_editor_domain::DocumentId |
| `pub fn document_content_transform( &self, document: DocumentId, ) -> Option<Affine2>` | impl ComponentTree | [源码:128](../../../crates/domain/src/components.rs#L128) | skin_editor_domain::Affine2, skin_editor_domain::DocumentId |
| `pub fn iter(&self) -> impl ExactSizeIterator<Item = &ComponentNode>` | impl ComponentTree | [源码:132](../../../crates/domain/src/components.rs#L132) | skin_editor_domain::ComponentNode |
| `fn clone(&self) -> ComponentTree` | impl Clone for ComponentTree | [源码:36](../../../crates/domain/src/components.rs#L36) | skin_editor_domain::ComponentTree |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for ComponentTree | [源码:36](../../../crates/domain/src/components.rs#L36) |  |
| `fn default() -> ComponentTree` | impl Default for ComponentTree | [源码:36](../../../crates/domain/src/components.rs#L36) | skin_editor_domain::ComponentTree |
| `fn eq(&self, other: &ComponentTree) -> bool` | impl PartialEq for ComponentTree | [源码:36](../../../crates/domain/src/components.rs#L36) | skin_editor_domain::ComponentTree |

## skin_editor_domain::ConfigInterpretation

[源码:7](../../../crates/domain/src/configuration.rs#L7) · enum

```rust
pub enum ConfigInterpretation {
    Opaque,
    Section {
        name: Range<usize>,
    },
    Entry {
        key: Range<usize>,
        value: Range<usize>,
    },
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::values::ConfigInterpretation`, `skin_editor_domain::configuration::ConfigInterpretation`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_domain::ConfigInterpretation::clone`, `skin_editor_domain::ConfigInterpretation::eq`, `skin_editor_domain::ConfigRecord (声明)`, `skin_editor_domain::ConfigRecord::interpretation`, `skin_editor_domain::ConfigRecord::new`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for ConfigInterpretation` — [源码:6](../../../crates/domain/src/configuration.rs#L6)
- `impl Debug for ConfigInterpretation` — [源码:6](../../../crates/domain/src/configuration.rs#L6)
- `impl PartialEq for ConfigInterpretation` — [源码:6](../../../crates/domain/src/configuration.rs#L6)
- `impl Eq for ConfigInterpretation` — [源码:6](../../../crates/domain/src/configuration.rs#L6)
- `impl StructuralPartialEq for ConfigInterpretation` — [源码:6](../../../crates/domain/src/configuration.rs#L6)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> ConfigInterpretation` | impl Clone for ConfigInterpretation | [源码:6](../../../crates/domain/src/configuration.rs#L6) | skin_editor_domain::ConfigInterpretation |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for ConfigInterpretation | [源码:6](../../../crates/domain/src/configuration.rs#L6) |  |
| `fn eq(&self, other: &ConfigInterpretation) -> bool` | impl PartialEq for ConfigInterpretation | [源码:6](../../../crates/domain/src/configuration.rs#L6) | skin_editor_domain::ConfigInterpretation |

## skin_editor_domain::ConfigRecord

[源码:19](../../../crates/domain/src/configuration.rs#L19) · struct

```rust
pub struct ConfigRecord {
    raw: Vec<u8>,
    interpretation: ConfigInterpretation,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::values::ConfigRecord`, `skin_editor_domain::configuration::ConfigRecord`

声明引用：`skin_editor_domain::ConfigInterpretation`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::Command (声明)`, `skin_editor_domain::ConfigRecord::clone`, `skin_editor_domain::ConfigRecord::eq`, `skin_editor_domain::ConfigRecord::new`, `skin_editor_domain::SkinConfiguration (声明)`, `skin_editor_domain::SkinConfiguration::new`, `skin_editor_domain::SkinConfiguration::records`

实现／约束（显式或 derive，无类继承）：

- `impl ConfigRecord` — [源码:24](../../../crates/domain/src/configuration.rs#L24)
- `impl Clone for ConfigRecord` — [源码:18](../../../crates/domain/src/configuration.rs#L18)
- `impl Debug for ConfigRecord` — [源码:18](../../../crates/domain/src/configuration.rs#L18)
- `impl PartialEq for ConfigRecord` — [源码:18](../../../crates/domain/src/configuration.rs#L18)
- `impl Eq for ConfigRecord` — [源码:18](../../../crates/domain/src/configuration.rs#L18)
- `impl StructuralPartialEq for ConfigRecord` — [源码:18](../../../crates/domain/src/configuration.rs#L18)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn new( raw: Vec<u8>, interpretation: ConfigInterpretation, ) -> Result<Self, ModelError>` | impl ConfigRecord | [源码:26](../../../crates/domain/src/configuration.rs#L26) | skin_editor_domain::ConfigInterpretation, skin_editor_domain::ConfigRecord, skin_editor_domain::ModelError |
| `pub fn raw(&self) -> &[u8] ⓘ` | impl ConfigRecord | [源码:48](../../../crates/domain/src/configuration.rs#L48) |  |
| `pub fn interpretation(&self) -> &ConfigInterpretation` | impl ConfigRecord | [源码:52](../../../crates/domain/src/configuration.rs#L52) | skin_editor_domain::ConfigInterpretation |
| `fn clone(&self) -> ConfigRecord` | impl Clone for ConfigRecord | [源码:18](../../../crates/domain/src/configuration.rs#L18) | skin_editor_domain::ConfigRecord |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for ConfigRecord | [源码:18](../../../crates/domain/src/configuration.rs#L18) |  |
| `fn eq(&self, other: &ConfigRecord) -> bool` | impl PartialEq for ConfigRecord | [源码:18](../../../crates/domain/src/configuration.rs#L18) | skin_editor_domain::ConfigRecord |

## skin_editor_domain::DocumentId

[源码:24](../../../crates/domain/src/identity.rs#L24) · struct

```rust
pub struct DocumentId(NonZeroU128);
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::values::DocumentId`, `skin_editor_domain::identity::DocumentId`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::Command (声明)`, `skin_editor_application::CommandFailure (声明)`, `skin_editor_application::PreviewTarget (声明)`, `skin_editor_application::execute::doc_index (声明)`, `skin_editor_application::execute::layers (声明)`, `skin_editor_application::history::Patch (声明)`, `skin_editor_domain::ComponentKind (声明)`, `skin_editor_domain::ComponentTree::document_content_transform`, `skin_editor_domain::ComponentTree::member`, `skin_editor_domain::DocumentId::clone`, `skin_editor_domain::DocumentId::cmp`, `skin_editor_domain::DocumentId::eq`, `skin_editor_domain::DocumentId::new`, `skin_editor_domain::DocumentId::partial_cmp`, `skin_editor_domain::ElementDocument (声明)`, `skin_editor_domain::ElementDocument::id`, `skin_editor_domain::ElementDocument::new`, `skin_editor_domain::ModelError (声明)`, `skin_editor_domain::ObjectId (声明)`, `skin_editor_domain::OutputMapping::document`, `skin_editor_domain::OutputMapping::new`, `skin_editor_domain::OutputSource (声明)`, `skin_editor_domain::SkinProject (声明)`, `skin_editor_domain::SkinProject::document`, `skin_editor_domain::SkinProject::document_content_transform`, `skin_editor_domain::SkinProject::document_workspace_transform`, `skin_editor_domain::SkinProject::layer_bounds`, `skin_editor_domain::components::ComponentStorage (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl DocumentId` — [源码:24](../../../crates/domain/src/identity.rs#L24)
- `impl Clone for DocumentId` — [源码:24](../../../crates/domain/src/identity.rs#L24)
- `impl Debug for DocumentId` — [源码:24](../../../crates/domain/src/identity.rs#L24)
- `impl Hash for DocumentId` — [源码:24](../../../crates/domain/src/identity.rs#L24)
- `impl Ord for DocumentId` — [源码:24](../../../crates/domain/src/identity.rs#L24)
- `impl PartialEq for DocumentId` — [源码:24](../../../crates/domain/src/identity.rs#L24)
- `impl PartialOrd for DocumentId` — [源码:24](../../../crates/domain/src/identity.rs#L24)
- `impl Copy for DocumentId` — [源码:24](../../../crates/domain/src/identity.rs#L24)
- `impl Eq for DocumentId` — [源码:24](../../../crates/domain/src/identity.rs#L24)
- `impl StructuralPartialEq for DocumentId` — [源码:24](../../../crates/domain/src/identity.rs#L24)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub const fn new(value: u128) -> Option<Self>` | impl DocumentId | [源码:24](../../../crates/domain/src/identity.rs#L24) | skin_editor_domain::DocumentId |
| `pub const fn get(self) -> u128` | impl DocumentId | [源码:24](../../../crates/domain/src/identity.rs#L24) |  |
| `fn clone(&self) -> DocumentId` | impl Clone for DocumentId | [源码:24](../../../crates/domain/src/identity.rs#L24) | skin_editor_domain::DocumentId |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for DocumentId | [源码:24](../../../crates/domain/src/identity.rs#L24) |  |
| `fn hash<__H: Hasher>(&self, state: &mut __H)` | impl Hash for DocumentId | [源码:24](../../../crates/domain/src/identity.rs#L24) |  |
| `fn cmp(&self, other: &DocumentId) -> Ordering` | impl Ord for DocumentId | [源码:24](../../../crates/domain/src/identity.rs#L24) | skin_editor_domain::DocumentId |
| `fn eq(&self, other: &DocumentId) -> bool` | impl PartialEq for DocumentId | [源码:24](../../../crates/domain/src/identity.rs#L24) | skin_editor_domain::DocumentId |
| `fn partial_cmp(&self, other: &DocumentId) -> Option<Ordering>` | impl PartialOrd for DocumentId | [源码:24](../../../crates/domain/src/identity.rs#L24) | skin_editor_domain::DocumentId |

## skin_editor_domain::ElementDocument

[源码:114](../../../crates/domain/src/layers.rs#L114) · struct

```rust
pub struct ElementDocument {
    id: DocumentId,
    canvas: PixelSize,
    layers: LayerTree,
    visible: bool,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::values::ElementDocument`, `skin_editor_domain::layers::ElementDocument`

声明引用：`skin_editor_domain::DocumentId`, `skin_editor_domain::LayerTree`, `skin_editor_domain::PixelSize`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::Command (声明)`, `skin_editor_application::history::Patch (声明)`, `skin_editor_domain::ElementDocument::clone`, `skin_editor_domain::ElementDocument::eq`, `skin_editor_domain::ElementDocument::new`, `skin_editor_domain::ElementDocument::with_visibility`, `skin_editor_domain::ProjectDefinition (声明)`, `skin_editor_domain::SkinProject (声明)`, `skin_editor_domain::SkinProject::document`, `skin_editor_domain::SkinProject::documents`

实现／约束（显式或 derive，无类继承）：

- `impl ElementDocument` — [源码:121](../../../crates/domain/src/layers.rs#L121)
- `impl Clone for ElementDocument` — [源码:113](../../../crates/domain/src/layers.rs#L113)
- `impl Debug for ElementDocument` — [源码:113](../../../crates/domain/src/layers.rs#L113)
- `impl PartialEq for ElementDocument` — [源码:113](../../../crates/domain/src/layers.rs#L113)
- `impl StructuralPartialEq for ElementDocument` — [源码:113](../../../crates/domain/src/layers.rs#L113)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub const fn new(id: DocumentId, canvas: PixelSize, layers: LayerTree) -> Self` | impl ElementDocument | [源码:122](../../../crates/domain/src/layers.rs#L122) | skin_editor_domain::DocumentId, skin_editor_domain::ElementDocument, skin_editor_domain::LayerTree, skin_editor_domain::PixelSize |
| `pub fn with_visibility(self, visible: bool) -> Self` | impl ElementDocument | [源码:130](../../../crates/domain/src/layers.rs#L130) | skin_editor_domain::ElementDocument |
| `pub const fn visible(&self) -> bool` | impl ElementDocument | [源码:134](../../../crates/domain/src/layers.rs#L134) |  |
| `pub const fn id(&self) -> DocumentId` | impl ElementDocument | [源码:138](../../../crates/domain/src/layers.rs#L138) | skin_editor_domain::DocumentId |
| `pub const fn canvas(&self) -> PixelSize` | impl ElementDocument | [源码:142](../../../crates/domain/src/layers.rs#L142) | skin_editor_domain::PixelSize |
| `pub fn layers(&self) -> &LayerTree` | impl ElementDocument | [源码:146](../../../crates/domain/src/layers.rs#L146) | skin_editor_domain::LayerTree |
| `fn clone(&self) -> ElementDocument` | impl Clone for ElementDocument | [源码:113](../../../crates/domain/src/layers.rs#L113) | skin_editor_domain::ElementDocument |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for ElementDocument | [源码:113](../../../crates/domain/src/layers.rs#L113) |  |
| `fn eq(&self, other: &ElementDocument) -> bool` | impl PartialEq for ElementDocument | [源码:113](../../../crates/domain/src/layers.rs#L113) | skin_editor_domain::ElementDocument |

## skin_editor_domain::Gain

[源码:32](../../../crates/domain/src/audio.rs#L32) · struct

```rust
pub struct Gain(f64);
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::values::Gain`, `skin_editor_domain::audio::Gain`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::Command (声明)`, `skin_editor_domain::AudioComponent (声明)`, `skin_editor_domain::AudioComponent::gain`, `skin_editor_domain::AudioComponent::with_properties`, `skin_editor_domain::Gain::clone`, `skin_editor_domain::Gain::eq`, `skin_editor_domain::Gain::new`

实现／约束（显式或 derive，无类继承）：

- `impl Gain` — [源码:33](../../../crates/domain/src/audio.rs#L33)
- `impl Clone for Gain` — [源码:31](../../../crates/domain/src/audio.rs#L31)
- `impl Debug for Gain` — [源码:31](../../../crates/domain/src/audio.rs#L31)
- `impl PartialEq for Gain` — [源码:31](../../../crates/domain/src/audio.rs#L31)
- `impl Copy for Gain` — [源码:31](../../../crates/domain/src/audio.rs#L31)
- `impl StructuralPartialEq for Gain` — [源码:31](../../../crates/domain/src/audio.rs#L31)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn new(value: f64) -> Result<Self, ModelError>` | impl Gain | [源码:35](../../../crates/domain/src/audio.rs#L35) | skin_editor_domain::Gain, skin_editor_domain::ModelError |
| `pub const fn get(self) -> f64` | impl Gain | [源码:41](../../../crates/domain/src/audio.rs#L41) |  |
| `fn clone(&self) -> Gain` | impl Clone for Gain | [源码:31](../../../crates/domain/src/audio.rs#L31) | skin_editor_domain::Gain |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for Gain | [源码:31](../../../crates/domain/src/audio.rs#L31) |  |
| `fn eq(&self, other: &Gain) -> bool` | impl PartialEq for Gain | [源码:31](../../../crates/domain/src/audio.rs#L31) | skin_editor_domain::Gain |

## skin_editor_domain::LayerId

[源码:25](../../../crates/domain/src/identity.rs#L25) · struct

```rust
pub struct LayerId(NonZeroU128);
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::values::LayerId`, `skin_editor_domain::identity::LayerId`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::Command (声明)`, `skin_editor_application::CommandFailure (声明)`, `skin_editor_application::execute::LayerNodes (声明)`, `skin_editor_application::execute::layer_children (声明)`, `skin_editor_application::execute::layer_node (声明)`, `skin_editor_application::execute::layers (声明)`, `skin_editor_domain::LayerId::clone`, `skin_editor_domain::LayerId::cmp`, `skin_editor_domain::LayerId::eq`, `skin_editor_domain::LayerId::new`, `skin_editor_domain::LayerId::partial_cmp`, `skin_editor_domain::LayerKind (声明)`, `skin_editor_domain::LayerNode (声明)`, `skin_editor_domain::LayerNode::children`, `skin_editor_domain::LayerTree::new`, `skin_editor_domain::LayerTree::node`, `skin_editor_domain::LayerTree::parent`, `skin_editor_domain::LayerTree::roots`, `skin_editor_domain::LayerTree::world_transform`, `skin_editor_domain::ObjectId (声明)`, `skin_editor_domain::SkinProject::layer_bounds`, `skin_editor_domain::layers::LayerStorage (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl LayerId` — [源码:25](../../../crates/domain/src/identity.rs#L25)
- `impl Clone for LayerId` — [源码:25](../../../crates/domain/src/identity.rs#L25)
- `impl Debug for LayerId` — [源码:25](../../../crates/domain/src/identity.rs#L25)
- `impl Hash for LayerId` — [源码:25](../../../crates/domain/src/identity.rs#L25)
- `impl Ord for LayerId` — [源码:25](../../../crates/domain/src/identity.rs#L25)
- `impl PartialEq for LayerId` — [源码:25](../../../crates/domain/src/identity.rs#L25)
- `impl PartialOrd for LayerId` — [源码:25](../../../crates/domain/src/identity.rs#L25)
- `impl Copy for LayerId` — [源码:25](../../../crates/domain/src/identity.rs#L25)
- `impl Eq for LayerId` — [源码:25](../../../crates/domain/src/identity.rs#L25)
- `impl StructuralPartialEq for LayerId` — [源码:25](../../../crates/domain/src/identity.rs#L25)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub const fn new(value: u128) -> Option<Self>` | impl LayerId | [源码:25](../../../crates/domain/src/identity.rs#L25) | skin_editor_domain::LayerId |
| `pub const fn get(self) -> u128` | impl LayerId | [源码:25](../../../crates/domain/src/identity.rs#L25) |  |
| `fn clone(&self) -> LayerId` | impl Clone for LayerId | [源码:25](../../../crates/domain/src/identity.rs#L25) | skin_editor_domain::LayerId |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for LayerId | [源码:25](../../../crates/domain/src/identity.rs#L25) |  |
| `fn hash<__H: Hasher>(&self, state: &mut __H)` | impl Hash for LayerId | [源码:25](../../../crates/domain/src/identity.rs#L25) |  |
| `fn cmp(&self, other: &LayerId) -> Ordering` | impl Ord for LayerId | [源码:25](../../../crates/domain/src/identity.rs#L25) | skin_editor_domain::LayerId |
| `fn eq(&self, other: &LayerId) -> bool` | impl PartialEq for LayerId | [源码:25](../../../crates/domain/src/identity.rs#L25) | skin_editor_domain::LayerId |
| `fn partial_cmp(&self, other: &LayerId) -> Option<Ordering>` | impl PartialOrd for LayerId | [源码:25](../../../crates/domain/src/identity.rs#L25) | skin_editor_domain::LayerId |

## skin_editor_domain::LayerKind

[源码:8](../../../crates/domain/src/layers.rs#L8) · enum

```rust
pub enum LayerKind {
    Image {
        asset: AssetId,
    },
    Group {
        children: Vec<LayerId>,
    },
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::values::LayerKind`, `skin_editor_domain::layers::LayerKind`

声明引用：`skin_editor_domain::AssetId`, `skin_editor_domain::LayerId`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_domain::LayerKind::clone`, `skin_editor_domain::LayerKind::eq`, `skin_editor_domain::LayerNode (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for LayerKind` — [源码:7](../../../crates/domain/src/layers.rs#L7)
- `impl Debug for LayerKind` — [源码:7](../../../crates/domain/src/layers.rs#L7)
- `impl PartialEq for LayerKind` — [源码:7](../../../crates/domain/src/layers.rs#L7)
- `impl StructuralPartialEq for LayerKind` — [源码:7](../../../crates/domain/src/layers.rs#L7)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> LayerKind` | impl Clone for LayerKind | [源码:7](../../../crates/domain/src/layers.rs#L7) | skin_editor_domain::LayerKind |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for LayerKind | [源码:7](../../../crates/domain/src/layers.rs#L7) |  |
| `fn eq(&self, other: &LayerKind) -> bool` | impl PartialEq for LayerKind | [源码:7](../../../crates/domain/src/layers.rs#L7) | skin_editor_domain::LayerKind |

## skin_editor_domain::LayerNode

[源码:20](../../../crates/domain/src/layers.rs#L20) · struct

```rust
pub struct LayerNode {
    pub id: LayerId,
    pub local: Affine2,
    pub opacity: Opacity,
    pub visible: bool,
    pub kind: LayerKind,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::values::LayerNode`, `skin_editor_domain::layers::LayerNode`

声明引用：`skin_editor_domain::Affine2`, `skin_editor_domain::LayerId`, `skin_editor_domain::LayerKind`, `skin_editor_domain::Opacity`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::Command (声明)`, `skin_editor_application::execute::LayerNodes (声明)`, `skin_editor_application::execute::layer_children (声明)`, `skin_editor_application::execute::layer_node (声明)`, `skin_editor_application::execute::layers (声明)`, `skin_editor_domain::LayerNode::clone`, `skin_editor_domain::LayerNode::eq`, `skin_editor_domain::LayerTree::iter`, `skin_editor_domain::LayerTree::new`, `skin_editor_domain::LayerTree::node`, `skin_editor_domain::layers::LayerStorage (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl LayerNode` — [源码:28](../../../crates/domain/src/layers.rs#L28)
- `impl Clone for LayerNode` — [源码:19](../../../crates/domain/src/layers.rs#L19)
- `impl Debug for LayerNode` — [源码:19](../../../crates/domain/src/layers.rs#L19)
- `impl PartialEq for LayerNode` — [源码:19](../../../crates/domain/src/layers.rs#L19)
- `impl StructuralPartialEq for LayerNode` — [源码:19](../../../crates/domain/src/layers.rs#L19)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn children(&self) -> &[LayerId]` | impl LayerNode | [源码:29](../../../crates/domain/src/layers.rs#L29) | skin_editor_domain::LayerId |
| `fn clone(&self) -> LayerNode` | impl Clone for LayerNode | [源码:19](../../../crates/domain/src/layers.rs#L19) | skin_editor_domain::LayerNode |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for LayerNode | [源码:19](../../../crates/domain/src/layers.rs#L19) |  |
| `fn eq(&self, other: &LayerNode) -> bool` | impl PartialEq for LayerNode | [源码:19](../../../crates/domain/src/layers.rs#L19) | skin_editor_domain::LayerNode |

## skin_editor_domain::LayerTree

[源码:44](../../../crates/domain/src/layers.rs#L44) · struct

```rust
pub struct LayerTree {
    data: Arc<LayerStorage>,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::values::LayerTree`, `skin_editor_domain::layers::LayerTree`

声明引用：`skin_editor_domain::layers::LayerStorage`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::execute::layers (声明)`, `skin_editor_domain::ElementDocument (声明)`, `skin_editor_domain::ElementDocument::layers`, `skin_editor_domain::ElementDocument::new`, `skin_editor_domain::LayerTree::clone`, `skin_editor_domain::LayerTree::default`, `skin_editor_domain::LayerTree::eq`, `skin_editor_domain::LayerTree::new`

实现／约束（显式或 derive，无类继承）：

- `impl LayerTree` — [源码:58](../../../crates/domain/src/layers.rs#L58)
- `impl Clone for LayerTree` — [源码:43](../../../crates/domain/src/layers.rs#L43)
- `impl Debug for LayerTree` — [源码:43](../../../crates/domain/src/layers.rs#L43)
- `impl Default for LayerTree` — [源码:43](../../../crates/domain/src/layers.rs#L43)
- `impl PartialEq for LayerTree` — [源码:43](../../../crates/domain/src/layers.rs#L43)
- `impl StructuralPartialEq for LayerTree` — [源码:43](../../../crates/domain/src/layers.rs#L43)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn new( roots: Vec<LayerId>, definitions: Vec<LayerNode>, ) -> Result<Self, ModelError>` | impl LayerTree | [源码:60](../../../crates/domain/src/layers.rs#L60) | skin_editor_domain::LayerId, skin_editor_domain::LayerNode, skin_editor_domain::LayerTree, skin_editor_domain::ModelError |
| `pub fn roots(&self) -> &[LayerId]` | impl LayerTree | [源码:88](../../../crates/domain/src/layers.rs#L88) | skin_editor_domain::LayerId |
| `pub fn node(&self, id: LayerId) -> Option<&LayerNode>` | impl LayerTree | [源码:92](../../../crates/domain/src/layers.rs#L92) | skin_editor_domain::LayerId, skin_editor_domain::LayerNode |
| `pub fn parent(&self, id: LayerId) -> Option<Option<LayerId>>` | impl LayerTree | [源码:96](../../../crates/domain/src/layers.rs#L96) | skin_editor_domain::LayerId |
| `pub fn world_transform(&self, id: LayerId) -> Option<Affine2>` | impl LayerTree | [源码:100](../../../crates/domain/src/layers.rs#L100) | skin_editor_domain::Affine2, skin_editor_domain::LayerId |
| `pub fn iter(&self) -> impl ExactSizeIterator<Item = &LayerNode>` | impl LayerTree | [源码:104](../../../crates/domain/src/layers.rs#L104) | skin_editor_domain::LayerNode |
| `fn clone(&self) -> LayerTree` | impl Clone for LayerTree | [源码:43](../../../crates/domain/src/layers.rs#L43) | skin_editor_domain::LayerTree |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for LayerTree | [源码:43](../../../crates/domain/src/layers.rs#L43) |  |
| `fn default() -> LayerTree` | impl Default for LayerTree | [源码:43](../../../crates/domain/src/layers.rs#L43) | skin_editor_domain::LayerTree |
| `fn eq(&self, other: &LayerTree) -> bool` | impl PartialEq for LayerTree | [源码:43](../../../crates/domain/src/layers.rs#L43) | skin_editor_domain::LayerTree |

## skin_editor_domain::MODEL_VERSION

[源码:10](../../../crates/domain/src/project.rs#L10) · constant

```rust
pub const MODEL_VERSION: u16 = 2;
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_domain::project::MODEL_VERSION`

## skin_editor_domain::ModelError

[源码:45](../../../crates/domain/src/lib.rs#L45) · enum

```rust
pub enum ModelError {
    InvalidValue(&'static str),
    DuplicateId(ObjectId),
    MissingReference(ObjectId),
    MultipleParents(ObjectId),
    Unreachable(ObjectId),
    Cycle,
    DuplicateMembership(DocumentId),
    OutputConflict(String),
    InvalidOutputPath(String),
    UnsupportedVersion(u16),
    NotInvertible,
    NumericOverflow,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::values::ModelError`

声明引用：`skin_editor_domain::DocumentId`, `skin_editor_domain::ObjectId`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::CommandFailure (声明)`, `skin_editor_application::CommandFailure::from`, `skin_editor_application::HistoryQueryError (声明)`, `skin_editor_domain::Affine2::compose`, `skin_editor_domain::Affine2::inverse`, `skin_editor_domain::Affine2::new`, `skin_editor_domain::Affine2::point`, `skin_editor_domain::Affine2::rebased`, `skin_editor_domain::Asset::new`, `skin_editor_domain::AssetLibrary::new`, `skin_editor_domain::AudioMetadata::new`, `skin_editor_domain::Bounds::transformed`, `skin_editor_domain::ComponentTree::new`, `skin_editor_domain::ConfigRecord::new`, `skin_editor_domain::Gain::new`, `skin_editor_domain::LayerTree::new`, `skin_editor_domain::ModelError::clone`, `skin_editor_domain::ModelError::eq`, `skin_editor_domain::ModelError::from`, `skin_editor_domain::Opacity::new`, `skin_editor_domain::OutputMapping::for_audio`, `skin_editor_domain::OutputMapping::new`, `skin_editor_domain::OutputPath::new`, `skin_editor_domain::PixelSize::new`, `skin_editor_domain::ProjectDefinition::upgrade_v1`, `skin_editor_domain::SkinProject::component_bounds`, `skin_editor_domain::SkinProject::document_content_transform`, `skin_editor_domain::SkinProject::document_workspace_transform`, `skin_editor_domain::SkinProject::from_definition`, `skin_editor_domain::SkinProject::layer_bounds`, `skin_editor_domain::SoundSlotKey::new`, `skin_editor_domain::forest::index_forest (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for ModelError` — [源码:44](../../../crates/domain/src/lib.rs#L44)
- `impl Debug for ModelError` — [源码:44](../../../crates/domain/src/lib.rs#L44)
- `impl PartialEq for ModelError` — [源码:44](../../../crates/domain/src/lib.rs#L44)
- `impl Eq for ModelError` — [源码:44](../../../crates/domain/src/lib.rs#L44)
- `impl StructuralPartialEq for ModelError` — [源码:44](../../../crates/domain/src/lib.rs#L44)
- `impl From<ModelError> for CommandFailure` — [源码:227](../../../crates/application/src/contract.rs#L227)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> ModelError` | impl Clone for ModelError | [源码:44](../../../crates/domain/src/lib.rs#L44) | skin_editor_domain::ModelError |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for ModelError | [源码:44](../../../crates/domain/src/lib.rs#L44) |  |
| `fn eq(&self, other: &ModelError) -> bool` | impl PartialEq for ModelError | [源码:44](../../../crates/domain/src/lib.rs#L44) | skin_editor_domain::ModelError |
| `fn from(value: ModelError) -> Self` | impl From<ModelError> for CommandFailure | [源码:228](../../../crates/application/src/contract.rs#L228) | skin_editor_domain::ModelError |

## skin_editor_domain::ObjectId

[源码:35](../../../crates/domain/src/lib.rs#L35) · enum

```rust
pub enum ObjectId {
    Asset(AssetId),
    Document(DocumentId),
    Layer(LayerId),
    Component(ComponentId),
    Output(OutputId),
    AudioComponent(AudioComponentId),
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::values::ObjectId`

声明引用：`skin_editor_domain::AssetId`, `skin_editor_domain::AudioComponentId`, `skin_editor_domain::ComponentId`, `skin_editor_domain::DocumentId`, `skin_editor_domain::LayerId`, `skin_editor_domain::OutputId`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::CommandFailure (声明)`, `skin_editor_application::CommandResult (声明)`, `skin_editor_application::HistoryEntry (声明)`, `skin_editor_application::HistoryResult (声明)`, `skin_editor_application::ProjectSession (声明)`, `skin_editor_application::ProjectSession::prepare`, `skin_editor_application::ProjectSession::publish`, `skin_editor_application::draft::Draft (声明)`, `skin_editor_application::execute::apply (声明)`, `skin_editor_application::execute::reserve (声明)`, `skin_editor_application::history::Patch (声明)`, `skin_editor_domain::ModelError (声明)`, `skin_editor_domain::ObjectId::clone`, `skin_editor_domain::ObjectId::cmp`, `skin_editor_domain::ObjectId::eq`, `skin_editor_domain::ObjectId::partial_cmp`, `skin_editor_domain::OutputSource::object_id`, `skin_editor_domain::forest::index_forest (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for ObjectId` — [源码:34](../../../crates/domain/src/lib.rs#L34)
- `impl Debug for ObjectId` — [源码:34](../../../crates/domain/src/lib.rs#L34)
- `impl Hash for ObjectId` — [源码:34](../../../crates/domain/src/lib.rs#L34)
- `impl Ord for ObjectId` — [源码:34](../../../crates/domain/src/lib.rs#L34)
- `impl PartialEq for ObjectId` — [源码:34](../../../crates/domain/src/lib.rs#L34)
- `impl PartialOrd for ObjectId` — [源码:34](../../../crates/domain/src/lib.rs#L34)
- `impl Copy for ObjectId` — [源码:34](../../../crates/domain/src/lib.rs#L34)
- `impl Eq for ObjectId` — [源码:34](../../../crates/domain/src/lib.rs#L34)
- `impl StructuralPartialEq for ObjectId` — [源码:34](../../../crates/domain/src/lib.rs#L34)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> ObjectId` | impl Clone for ObjectId | [源码:34](../../../crates/domain/src/lib.rs#L34) | skin_editor_domain::ObjectId |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for ObjectId | [源码:34](../../../crates/domain/src/lib.rs#L34) |  |
| `fn hash<__H: Hasher>(&self, state: &mut __H)` | impl Hash for ObjectId | [源码:34](../../../crates/domain/src/lib.rs#L34) |  |
| `fn cmp(&self, other: &ObjectId) -> Ordering` | impl Ord for ObjectId | [源码:34](../../../crates/domain/src/lib.rs#L34) | skin_editor_domain::ObjectId |
| `fn eq(&self, other: &ObjectId) -> bool` | impl PartialEq for ObjectId | [源码:34](../../../crates/domain/src/lib.rs#L34) | skin_editor_domain::ObjectId |
| `fn partial_cmp(&self, other: &ObjectId) -> Option<Ordering>` | impl PartialOrd for ObjectId | [源码:34](../../../crates/domain/src/lib.rs#L34) | skin_editor_domain::ObjectId |

## skin_editor_domain::Opacity

[源码:32](../../../crates/domain/src/geometry.rs#L32) · struct

```rust
pub struct Opacity(f32);
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::values::Opacity`, `skin_editor_domain::geometry::Opacity`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::Command (声明)`, `skin_editor_domain::LayerNode (声明)`, `skin_editor_domain::Opacity::clone`, `skin_editor_domain::Opacity::eq`, `skin_editor_domain::Opacity::new`

实现／约束（显式或 derive，无类继承）：

- `impl Opacity` — [源码:34](../../../crates/domain/src/geometry.rs#L34)
- `impl Clone for Opacity` — [源码:31](../../../crates/domain/src/geometry.rs#L31)
- `impl Debug for Opacity` — [源码:31](../../../crates/domain/src/geometry.rs#L31)
- `impl PartialEq for Opacity` — [源码:31](../../../crates/domain/src/geometry.rs#L31)
- `impl Copy for Opacity` — [源码:31](../../../crates/domain/src/geometry.rs#L31)
- `impl StructuralPartialEq for Opacity` — [源码:31](../../../crates/domain/src/geometry.rs#L31)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn new(value: f32) -> Result<Self, ModelError>` | impl Opacity | [源码:37](../../../crates/domain/src/geometry.rs#L37) | skin_editor_domain::ModelError, skin_editor_domain::Opacity |
| `pub const fn get(self) -> f32` | impl Opacity | [源码:44](../../../crates/domain/src/geometry.rs#L44) |  |
| `fn clone(&self) -> Opacity` | impl Clone for Opacity | [源码:31](../../../crates/domain/src/geometry.rs#L31) | skin_editor_domain::Opacity |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for Opacity | [源码:31](../../../crates/domain/src/geometry.rs#L31) |  |
| `fn eq(&self, other: &Opacity) -> bool` | impl PartialEq for Opacity | [源码:31](../../../crates/domain/src/geometry.rs#L31) | skin_editor_domain::Opacity |

## skin_editor_domain::OutputId

[源码:27](../../../crates/domain/src/identity.rs#L27) · struct

```rust
pub struct OutputId(NonZeroU128);
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::values::OutputId`, `skin_editor_domain::identity::OutputId`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::Command (声明)`, `skin_editor_domain::ObjectId (声明)`, `skin_editor_domain::OutputId::clone`, `skin_editor_domain::OutputId::cmp`, `skin_editor_domain::OutputId::eq`, `skin_editor_domain::OutputId::new`, `skin_editor_domain::OutputId::partial_cmp`, `skin_editor_domain::OutputMapping (声明)`, `skin_editor_domain::OutputMapping::for_audio`, `skin_editor_domain::OutputMapping::id`, `skin_editor_domain::OutputMapping::new`

实现／约束（显式或 derive，无类继承）：

- `impl OutputId` — [源码:27](../../../crates/domain/src/identity.rs#L27)
- `impl Clone for OutputId` — [源码:27](../../../crates/domain/src/identity.rs#L27)
- `impl Debug for OutputId` — [源码:27](../../../crates/domain/src/identity.rs#L27)
- `impl Hash for OutputId` — [源码:27](../../../crates/domain/src/identity.rs#L27)
- `impl Ord for OutputId` — [源码:27](../../../crates/domain/src/identity.rs#L27)
- `impl PartialEq for OutputId` — [源码:27](../../../crates/domain/src/identity.rs#L27)
- `impl PartialOrd for OutputId` — [源码:27](../../../crates/domain/src/identity.rs#L27)
- `impl Copy for OutputId` — [源码:27](../../../crates/domain/src/identity.rs#L27)
- `impl Eq for OutputId` — [源码:27](../../../crates/domain/src/identity.rs#L27)
- `impl StructuralPartialEq for OutputId` — [源码:27](../../../crates/domain/src/identity.rs#L27)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub const fn new(value: u128) -> Option<Self>` | impl OutputId | [源码:27](../../../crates/domain/src/identity.rs#L27) | skin_editor_domain::OutputId |
| `pub const fn get(self) -> u128` | impl OutputId | [源码:27](../../../crates/domain/src/identity.rs#L27) |  |
| `fn clone(&self) -> OutputId` | impl Clone for OutputId | [源码:27](../../../crates/domain/src/identity.rs#L27) | skin_editor_domain::OutputId |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for OutputId | [源码:27](../../../crates/domain/src/identity.rs#L27) |  |
| `fn hash<__H: Hasher>(&self, state: &mut __H)` | impl Hash for OutputId | [源码:27](../../../crates/domain/src/identity.rs#L27) |  |
| `fn cmp(&self, other: &OutputId) -> Ordering` | impl Ord for OutputId | [源码:27](../../../crates/domain/src/identity.rs#L27) | skin_editor_domain::OutputId |
| `fn eq(&self, other: &OutputId) -> bool` | impl PartialEq for OutputId | [源码:27](../../../crates/domain/src/identity.rs#L27) | skin_editor_domain::OutputId |
| `fn partial_cmp(&self, other: &OutputId) -> Option<Ordering>` | impl PartialOrd for OutputId | [源码:27](../../../crates/domain/src/identity.rs#L27) | skin_editor_domain::OutputId |

## skin_editor_domain::OutputMapping

[源码:54](../../../crates/domain/src/output.rs#L54) · struct

```rust
pub struct OutputMapping {
    id: OutputId,
    source: OutputSource,
    element_key: String,
    path: OutputPath,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::values::OutputMapping`, `skin_editor_domain::output::OutputMapping`

声明引用：`skin_editor_domain::OutputId`, `skin_editor_domain::OutputPath`, `skin_editor_domain::OutputSource`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::Command (声明)`, `skin_editor_application::history::Patch (声明)`, `skin_editor_domain::OutputMapping::clone`, `skin_editor_domain::OutputMapping::eq`, `skin_editor_domain::OutputMapping::for_audio`, `skin_editor_domain::OutputMapping::new`, `skin_editor_domain::ProjectDefinition (声明)`, `skin_editor_domain::SkinProject (声明)`, `skin_editor_domain::SkinProject::outputs`

实现／约束（显式或 derive，无类继承）：

- `impl OutputMapping` — [源码:61](../../../crates/domain/src/output.rs#L61)
- `impl Clone for OutputMapping` — [源码:53](../../../crates/domain/src/output.rs#L53)
- `impl Debug for OutputMapping` — [源码:53](../../../crates/domain/src/output.rs#L53)
- `impl PartialEq for OutputMapping` — [源码:53](../../../crates/domain/src/output.rs#L53)
- `impl Eq for OutputMapping` — [源码:53](../../../crates/domain/src/output.rs#L53)
- `impl StructuralPartialEq for OutputMapping` — [源码:53](../../../crates/domain/src/output.rs#L53)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn new( id: OutputId, document: DocumentId, element_key: String, path: OutputPath, ) -> Result<Self, ModelError>` | impl OutputMapping | [源码:62](../../../crates/domain/src/output.rs#L62) | skin_editor_domain::DocumentId, skin_editor_domain::ModelError, skin_editor_domain::OutputId, skin_editor_domain::OutputMapping, skin_editor_domain::OutputPath |
| `pub const fn id(&self) -> OutputId` | impl OutputMapping | [源码:79](../../../crates/domain/src/output.rs#L79) | skin_editor_domain::OutputId |
| `pub const fn document(&self) -> Option<DocumentId>` | impl OutputMapping | [源码:83](../../../crates/domain/src/output.rs#L83) | skin_editor_domain::DocumentId |
| `pub fn for_audio( id: OutputId, component: AudioComponentId, element_key: String, path: OutputPath, ) -> Result<Self, ModelError>` | impl OutputMapping | [源码:89](../../../crates/domain/src/output.rs#L89) | skin_editor_domain::AudioComponentId, skin_editor_domain::ModelError, skin_editor_domain::OutputId, skin_editor_domain::OutputMapping, skin_editor_domain::OutputPath |
| `pub const fn source(&self) -> OutputSource` | impl OutputMapping | [源码:105](../../../crates/domain/src/output.rs#L105) | skin_editor_domain::OutputSource |
| `pub fn element_key(&self) -> &str` | impl OutputMapping | [源码:109](../../../crates/domain/src/output.rs#L109) |  |
| `pub fn path(&self) -> &OutputPath` | impl OutputMapping | [源码:113](../../../crates/domain/src/output.rs#L113) | skin_editor_domain::OutputPath |
| `fn clone(&self) -> OutputMapping` | impl Clone for OutputMapping | [源码:53](../../../crates/domain/src/output.rs#L53) | skin_editor_domain::OutputMapping |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for OutputMapping | [源码:53](../../../crates/domain/src/output.rs#L53) |  |
| `fn eq(&self, other: &OutputMapping) -> bool` | impl PartialEq for OutputMapping | [源码:53](../../../crates/domain/src/output.rs#L53) | skin_editor_domain::OutputMapping |

## skin_editor_domain::OutputPath

[源码:20](../../../crates/domain/src/output.rs#L20) · struct

```rust
pub struct OutputPath(String);
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::values::OutputPath`, `skin_editor_domain::output::OutputPath`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_domain::OutputMapping (声明)`, `skin_editor_domain::OutputMapping::for_audio`, `skin_editor_domain::OutputMapping::new`, `skin_editor_domain::OutputMapping::path`, `skin_editor_domain::OutputPath::clone`, `skin_editor_domain::OutputPath::eq`, `skin_editor_domain::OutputPath::new`

实现／约束（显式或 derive，无类继承）：

- `impl OutputPath` — [源码:22](../../../crates/domain/src/output.rs#L22)
- `impl Clone for OutputPath` — [源码:19](../../../crates/domain/src/output.rs#L19)
- `impl Debug for OutputPath` — [源码:19](../../../crates/domain/src/output.rs#L19)
- `impl PartialEq for OutputPath` — [源码:19](../../../crates/domain/src/output.rs#L19)
- `impl Eq for OutputPath` — [源码:19](../../../crates/domain/src/output.rs#L19)
- `impl StructuralPartialEq for OutputPath` — [源码:19](../../../crates/domain/src/output.rs#L19)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn new(path: String) -> Result<Self, ModelError>` | impl OutputPath | [源码:23](../../../crates/domain/src/output.rs#L23) | skin_editor_domain::ModelError, skin_editor_domain::OutputPath |
| `pub fn as_str(&self) -> &str` | impl OutputPath | [源码:44](../../../crates/domain/src/output.rs#L44) |  |
| `pub(crate) fn collision_key(&self) -> String` | impl OutputPath | [源码:48](../../../crates/domain/src/output.rs#L48) |  |
| `fn clone(&self) -> OutputPath` | impl Clone for OutputPath | [源码:19](../../../crates/domain/src/output.rs#L19) | skin_editor_domain::OutputPath |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for OutputPath | [源码:19](../../../crates/domain/src/output.rs#L19) |  |
| `fn eq(&self, other: &OutputPath) -> bool` | impl PartialEq for OutputPath | [源码:19](../../../crates/domain/src/output.rs#L19) | skin_editor_domain::OutputPath |

## skin_editor_domain::OutputSource

[源码:4](../../../crates/domain/src/output.rs#L4) · enum

```rust
pub enum OutputSource {
    Image(DocumentId),
    Audio(AudioComponentId),
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::values::OutputSource`, `skin_editor_domain::output::OutputSource`

声明引用：`skin_editor_domain::AudioComponentId`, `skin_editor_domain::DocumentId`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_domain::OutputMapping (声明)`, `skin_editor_domain::OutputMapping::source`, `skin_editor_domain::OutputSource::clone`, `skin_editor_domain::OutputSource::eq`

实现／约束（显式或 derive，无类继承）：

- `impl OutputSource` — [源码:8](../../../crates/domain/src/output.rs#L8)
- `impl Clone for OutputSource` — [源码:3](../../../crates/domain/src/output.rs#L3)
- `impl Debug for OutputSource` — [源码:3](../../../crates/domain/src/output.rs#L3)
- `impl PartialEq for OutputSource` — [源码:3](../../../crates/domain/src/output.rs#L3)
- `impl Copy for OutputSource` — [源码:3](../../../crates/domain/src/output.rs#L3)
- `impl Eq for OutputSource` — [源码:3](../../../crates/domain/src/output.rs#L3)
- `impl StructuralPartialEq for OutputSource` — [源码:3](../../../crates/domain/src/output.rs#L3)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub const fn object_id(self) -> ObjectId` | impl OutputSource | [源码:9](../../../crates/domain/src/output.rs#L9) | skin_editor_domain::ObjectId |
| `fn clone(&self) -> OutputSource` | impl Clone for OutputSource | [源码:3](../../../crates/domain/src/output.rs#L3) | skin_editor_domain::OutputSource |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for OutputSource | [源码:3](../../../crates/domain/src/output.rs#L3) |  |
| `fn eq(&self, other: &OutputSource) -> bool` | impl PartialEq for OutputSource | [源码:3](../../../crates/domain/src/output.rs#L3) | skin_editor_domain::OutputSource |

## skin_editor_domain::PixelSize

[源码:5](../../../crates/domain/src/geometry.rs#L5) · struct

```rust
pub struct PixelSize {
    width: u32,
    height: u32,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::values::PixelSize`, `skin_editor_domain::geometry::PixelSize`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::Command (声明)`, `skin_editor_application::StagedRaster (声明)`, `skin_editor_domain::AssetKind (声明)`, `skin_editor_domain::Bounds::transformed`, `skin_editor_domain::ElementDocument (声明)`, `skin_editor_domain::ElementDocument::canvas`, `skin_editor_domain::ElementDocument::new`, `skin_editor_domain::PixelSize::clone`, `skin_editor_domain::PixelSize::eq`, `skin_editor_domain::PixelSize::new`

实现／约束（显式或 derive，无类继承）：

- `impl PixelSize` — [源码:10](../../../crates/domain/src/geometry.rs#L10)
- `impl Clone for PixelSize` — [源码:4](../../../crates/domain/src/geometry.rs#L4)
- `impl Debug for PixelSize` — [源码:4](../../../crates/domain/src/geometry.rs#L4)
- `impl PartialEq for PixelSize` — [源码:4](../../../crates/domain/src/geometry.rs#L4)
- `impl Copy for PixelSize` — [源码:4](../../../crates/domain/src/geometry.rs#L4)
- `impl Eq for PixelSize` — [源码:4](../../../crates/domain/src/geometry.rs#L4)
- `impl StructuralPartialEq for PixelSize` — [源码:4](../../../crates/domain/src/geometry.rs#L4)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn new(width: u32, height: u32) -> Result<Self, ModelError>` | impl PixelSize | [源码:11](../../../crates/domain/src/geometry.rs#L11) | skin_editor_domain::ModelError, skin_editor_domain::PixelSize |
| `pub const fn width(self) -> u32` | impl PixelSize | [源码:22](../../../crates/domain/src/geometry.rs#L22) |  |
| `pub const fn height(self) -> u32` | impl PixelSize | [源码:26](../../../crates/domain/src/geometry.rs#L26) |  |
| `fn clone(&self) -> PixelSize` | impl Clone for PixelSize | [源码:4](../../../crates/domain/src/geometry.rs#L4) | skin_editor_domain::PixelSize |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for PixelSize | [源码:4](../../../crates/domain/src/geometry.rs#L4) |  |
| `fn eq(&self, other: &PixelSize) -> bool` | impl PartialEq for PixelSize | [源码:4](../../../crates/domain/src/geometry.rs#L4) | skin_editor_domain::PixelSize |

## skin_editor_domain::ProjectDefinition

[源码:14](../../../crates/domain/src/project.rs#L14) · struct

```rust
pub struct ProjectDefinition {
    pub model_version: u16,
    pub id: ProjectId,
    pub assets: Vec<Asset>,
    pub documents: Vec<ElementDocument>,
    pub audio_components: Vec<AudioComponent>,
    pub components: ComponentTree,
    pub configuration: SkinConfiguration,
    pub outputs: Vec<OutputMapping>,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_domain::project::ProjectDefinition`

声明引用：`skin_editor_domain::Asset`, `skin_editor_domain::AudioComponent`, `skin_editor_domain::ComponentTree`, `skin_editor_domain::ElementDocument`, `skin_editor_domain::OutputMapping`, `skin_editor_domain::ProjectId`, `skin_editor_domain::SkinConfiguration`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::execute::components (声明)`, `skin_editor_application::execute::doc_index (声明)`, `skin_editor_application::execute::layers (声明)`, `skin_editor_application::history::Patch::apply_definition`, `skin_editor_application::history::definition (声明)`, `skin_editor_domain::ProjectDefinition::clone`, `skin_editor_domain::ProjectDefinition::eq`, `skin_editor_domain::SkinProject::from_definition`

实现／约束（显式或 derive，无类继承）：

- `impl ProjectDefinition` — [源码:25](../../../crates/domain/src/project.rs#L25)
- `impl Clone for ProjectDefinition` — [源码:13](../../../crates/domain/src/project.rs#L13)
- `impl Debug for ProjectDefinition` — [源码:13](../../../crates/domain/src/project.rs#L13)
- `impl PartialEq for ProjectDefinition` — [源码:13](../../../crates/domain/src/project.rs#L13)
- `impl StructuralPartialEq for ProjectDefinition` — [源码:13](../../../crates/domain/src/project.rs#L13)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn upgrade_v1(self) -> Result<Self, ModelError>` | impl ProjectDefinition | [源码:28](../../../crates/domain/src/project.rs#L28) | skin_editor_domain::ModelError |
| `pub fn empty(id: ProjectId) -> Self` | impl ProjectDefinition | [源码:49](../../../crates/domain/src/project.rs#L49) | skin_editor_domain::ProjectId |
| `fn clone(&self) -> ProjectDefinition` | impl Clone for ProjectDefinition | [源码:13](../../../crates/domain/src/project.rs#L13) | skin_editor_domain::ProjectDefinition |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for ProjectDefinition | [源码:13](../../../crates/domain/src/project.rs#L13) |  |
| `fn eq(&self, other: &ProjectDefinition) -> bool` | impl PartialEq for ProjectDefinition | [源码:13](../../../crates/domain/src/project.rs#L13) | skin_editor_domain::ProjectDefinition |

## skin_editor_domain::ProjectId

[源码:69](../../../crates/domain/src/lib.rs#L69) · struct

```rust
pub struct ProjectId(pub(crate) NonZeroU128);
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_domain::ProjectDefinition (声明)`, `skin_editor_domain::ProjectDefinition::empty`, `skin_editor_domain::ProjectId::clone`, `skin_editor_domain::ProjectId::cmp`, `skin_editor_domain::ProjectId::eq`, `skin_editor_domain::ProjectId::partial_cmp`, `skin_editor_domain::SkinProject (声明)`, `skin_editor_domain::SkinProject::empty`, `skin_editor_domain::SkinProject::id`

实现／约束（显式或 derive，无类继承）：

- `impl ProjectId` — [源码:71](../../../crates/domain/src/lib.rs#L71)
- `impl Clone for ProjectId` — [源码:68](../../../crates/domain/src/lib.rs#L68)
- `impl Debug for ProjectId` — [源码:68](../../../crates/domain/src/lib.rs#L68)
- `impl Hash for ProjectId` — [源码:68](../../../crates/domain/src/lib.rs#L68)
- `impl Ord for ProjectId` — [源码:68](../../../crates/domain/src/lib.rs#L68)
- `impl PartialEq for ProjectId` — [源码:68](../../../crates/domain/src/lib.rs#L68)
- `impl PartialOrd for ProjectId` — [源码:68](../../../crates/domain/src/lib.rs#L68)
- `impl Copy for ProjectId` — [源码:68](../../../crates/domain/src/lib.rs#L68)
- `impl Eq for ProjectId` — [源码:68](../../../crates/domain/src/lib.rs#L68)
- `impl StructuralPartialEq for ProjectId` — [源码:68](../../../crates/domain/src/lib.rs#L68)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub const fn new(value: u128) -> Option<Self>` | impl ProjectId | [源码:73](../../../crates/domain/src/lib.rs#L73) |  |
| `pub const fn get(self) -> u128` | impl ProjectId | [源码:81](../../../crates/domain/src/lib.rs#L81) |  |
| `fn clone(&self) -> ProjectId` | impl Clone for ProjectId | [源码:68](../../../crates/domain/src/lib.rs#L68) | skin_editor_domain::ProjectId |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for ProjectId | [源码:68](../../../crates/domain/src/lib.rs#L68) |  |
| `fn hash<__H: Hasher>(&self, state: &mut __H)` | impl Hash for ProjectId | [源码:68](../../../crates/domain/src/lib.rs#L68) |  |
| `fn cmp(&self, other: &ProjectId) -> Ordering` | impl Ord for ProjectId | [源码:68](../../../crates/domain/src/lib.rs#L68) | skin_editor_domain::ProjectId |
| `fn eq(&self, other: &ProjectId) -> bool` | impl PartialEq for ProjectId | [源码:68](../../../crates/domain/src/lib.rs#L68) | skin_editor_domain::ProjectId |
| `fn partial_cmp(&self, other: &ProjectId) -> Option<Ordering>` | impl PartialOrd for ProjectId | [源码:68](../../../crates/domain/src/lib.rs#L68) | skin_editor_domain::ProjectId |

## skin_editor_domain::SkinConfiguration

[源码:60](../../../crates/domain/src/configuration.rs#L60) · struct

```rust
pub struct SkinConfiguration {
    records: Vec<ConfigRecord>,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_domain::configuration::SkinConfiguration`

声明引用：`skin_editor_domain::ConfigRecord`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::history::Patch (声明)`, `skin_editor_domain::ProjectDefinition (声明)`, `skin_editor_domain::SkinConfiguration::clone`, `skin_editor_domain::SkinConfiguration::default`, `skin_editor_domain::SkinConfiguration::eq`, `skin_editor_domain::SkinProject (声明)`, `skin_editor_domain::SkinProject::configuration`

实现／约束（显式或 derive，无类继承）：

- `impl SkinConfiguration` — [源码:64](../../../crates/domain/src/configuration.rs#L64)
- `impl Clone for SkinConfiguration` — [源码:59](../../../crates/domain/src/configuration.rs#L59)
- `impl Debug for SkinConfiguration` — [源码:59](../../../crates/domain/src/configuration.rs#L59)
- `impl Default for SkinConfiguration` — [源码:59](../../../crates/domain/src/configuration.rs#L59)
- `impl PartialEq for SkinConfiguration` — [源码:59](../../../crates/domain/src/configuration.rs#L59)
- `impl Eq for SkinConfiguration` — [源码:59](../../../crates/domain/src/configuration.rs#L59)
- `impl StructuralPartialEq for SkinConfiguration` — [源码:59](../../../crates/domain/src/configuration.rs#L59)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub const fn new(records: Vec<ConfigRecord>) -> Self` | impl SkinConfiguration | [源码:65](../../../crates/domain/src/configuration.rs#L65) | skin_editor_domain::ConfigRecord |
| `pub fn records(&self) -> &[ConfigRecord]` | impl SkinConfiguration | [源码:69](../../../crates/domain/src/configuration.rs#L69) | skin_editor_domain::ConfigRecord |
| `pub fn to_bytes(&self) -> Vec<u8> ⓘ` | impl SkinConfiguration | [源码:73](../../../crates/domain/src/configuration.rs#L73) |  |
| `fn clone(&self) -> SkinConfiguration` | impl Clone for SkinConfiguration | [源码:59](../../../crates/domain/src/configuration.rs#L59) | skin_editor_domain::SkinConfiguration |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for SkinConfiguration | [源码:59](../../../crates/domain/src/configuration.rs#L59) |  |
| `fn default() -> SkinConfiguration` | impl Default for SkinConfiguration | [源码:59](../../../crates/domain/src/configuration.rs#L59) | skin_editor_domain::SkinConfiguration |
| `fn eq(&self, other: &SkinConfiguration) -> bool` | impl PartialEq for SkinConfiguration | [源码:59](../../../crates/domain/src/configuration.rs#L59) | skin_editor_domain::SkinConfiguration |

## skin_editor_domain::SkinProject

[源码:71](../../../crates/domain/src/project.rs#L71) · struct

```rust
pub struct SkinProject {
    id: ProjectId,
    assets: AssetLibrary,
    documents: BTreeMap<DocumentId, ElementDocument>,
    audio_components: BTreeMap<AudioComponentId, AudioComponent>,
    components: ComponentTree,
    configuration: SkinConfiguration,
    outputs: Vec<OutputMapping>,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::values::SkinProject`, `skin_editor_domain::project::SkinProject`

声明引用：`skin_editor_domain::AssetLibrary`, `skin_editor_domain::AudioComponent`, `skin_editor_domain::AudioComponentId`, `skin_editor_domain::ComponentTree`, `skin_editor_domain::DocumentId`, `skin_editor_domain::ElementDocument`, `skin_editor_domain::OutputMapping`, `skin_editor_domain::ProjectId`, `skin_editor_domain::SkinConfiguration`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_application::DisabledAudio::synchronize`, `skin_editor_application::DraftSnapshot (声明)`, `skin_editor_application::HistoricalSnapshot (声明)`, `skin_editor_application::PreviewPort (声明)`, `skin_editor_application::PreviewPort::request`, `skin_editor_application::ProjectSession (声明)`, `skin_editor_application::ProjectSession::pin_candidate`, `skin_editor_application::ProjectSession::prepare`, `skin_editor_application::ProjectSession::publish`, `skin_editor_application::ProjectSnapshot (声明)`, `skin_editor_application::SkinAudioPort (声明)`, `skin_editor_application::SkinAudioPort::synchronize`, `skin_editor_application::draft::Draft (声明)`, `skin_editor_application::execute::apply (声明)`, `skin_editor_application::history::History (声明)`, `skin_editor_application::history::History::materialize`, `skin_editor_application::history::History::new`, `skin_editor_application::history::History::transition`, `skin_editor_application::history::Patch::between`, `skin_editor_application::history::definition (声明)`, `skin_editor_domain::SkinProject::clone`, `skin_editor_domain::SkinProject::empty`, `skin_editor_domain::SkinProject::eq`, `skin_editor_domain::SkinProject::from_definition`

实现／约束（显式或 derive，无类继承）：

- `impl SkinProject` — [源码:81](../../../crates/domain/src/project.rs#L81)
- `impl Clone for SkinProject` — [源码:70](../../../crates/domain/src/project.rs#L70)
- `impl Debug for SkinProject` — [源码:70](../../../crates/domain/src/project.rs#L70)
- `impl PartialEq for SkinProject` — [源码:70](../../../crates/domain/src/project.rs#L70)
- `impl StructuralPartialEq for SkinProject` — [源码:70](../../../crates/domain/src/project.rs#L70)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn empty(id: ProjectId) -> Self` | impl SkinProject | [源码:83](../../../crates/domain/src/project.rs#L83) | skin_editor_domain::ProjectId, skin_editor_domain::SkinProject |
| `pub fn from_definition( definition: ProjectDefinition, ) -> Result<Self, ModelError>` | impl SkinProject | [源码:95](../../../crates/domain/src/project.rs#L95) | skin_editor_domain::ModelError, skin_editor_domain::ProjectDefinition, skin_editor_domain::SkinProject |
| `pub const fn id(&self) -> ProjectId` | impl SkinProject | [源码:225](../../../crates/domain/src/project.rs#L225) | skin_editor_domain::ProjectId |
| `pub const fn model_version(&self) -> u16` | impl SkinProject | [源码:229](../../../crates/domain/src/project.rs#L229) |  |
| `pub fn assets(&self) -> &AssetLibrary` | impl SkinProject | [源码:233](../../../crates/domain/src/project.rs#L233) | skin_editor_domain::AssetLibrary |
| `pub fn document(&self, id: DocumentId) -> Option<&ElementDocument>` | impl SkinProject | [源码:237](../../../crates/domain/src/project.rs#L237) | skin_editor_domain::DocumentId, skin_editor_domain::ElementDocument |
| `pub fn documents(&self) -> impl ExactSizeIterator<Item = &ElementDocument>` | impl SkinProject | [源码:241](../../../crates/domain/src/project.rs#L241) | skin_editor_domain::ElementDocument |
| `pub fn components(&self) -> &ComponentTree` | impl SkinProject | [源码:245](../../../crates/domain/src/project.rs#L245) | skin_editor_domain::ComponentTree |
| `pub fn audio_component(&self, id: AudioComponentId) -> Option<&AudioComponent>` | impl SkinProject | [源码:249](../../../crates/domain/src/project.rs#L249) | skin_editor_domain::AudioComponent, skin_editor_domain::AudioComponentId |
| `pub fn audio_components(&self) -> impl ExactSizeIterator<Item = &AudioComponent>` | impl SkinProject | [源码:253](../../../crates/domain/src/project.rs#L253) | skin_editor_domain::AudioComponent |
| `pub fn configuration(&self) -> &SkinConfiguration` | impl SkinProject | [源码:257](../../../crates/domain/src/project.rs#L257) | skin_editor_domain::SkinConfiguration |
| `pub fn outputs(&self) -> &[OutputMapping]` | impl SkinProject | [源码:261](../../../crates/domain/src/project.rs#L261) | skin_editor_domain::OutputMapping |
| `pub fn document_content_transform( &self, id: DocumentId, ) -> Result<Affine2, ModelError>` | impl SkinProject | [源码:266](../../../crates/domain/src/project.rs#L266) | skin_editor_domain::Affine2, skin_editor_domain::DocumentId, skin_editor_domain::ModelError |
| `pub fn document_workspace_transform( &self, id: DocumentId, ) -> Result<Affine2, ModelError>` | impl SkinProject | [源码:278](../../../crates/domain/src/project.rs#L278) | skin_editor_domain::Affine2, skin_editor_domain::DocumentId, skin_editor_domain::ModelError |
| `pub fn layer_bounds( &self, document: DocumentId, layer: LayerId, ) -> Result<Option<Bounds>, ModelError>` | impl SkinProject | [源码:305](../../../crates/domain/src/project.rs#L305) | skin_editor_domain::Bounds, skin_editor_domain::DocumentId, skin_editor_domain::LayerId, skin_editor_domain::ModelError |
| `pub fn component_bounds( &self, component: ComponentId, ) -> Result<Option<Bounds>, ModelError>` | impl SkinProject | [源码:343](../../../crates/domain/src/project.rs#L343) | skin_editor_domain::Bounds, skin_editor_domain::ComponentId, skin_editor_domain::ModelError |
| `fn clone(&self) -> SkinProject` | impl Clone for SkinProject | [源码:70](../../../crates/domain/src/project.rs#L70) | skin_editor_domain::SkinProject |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for SkinProject | [源码:70](../../../crates/domain/src/project.rs#L70) |  |
| `fn eq(&self, other: &SkinProject) -> bool` | impl PartialEq for SkinProject | [源码:70](../../../crates/domain/src/project.rs#L70) | skin_editor_domain::SkinProject |

## skin_editor_domain::SoundSlotKey

[源码:46](../../../crates/domain/src/audio.rs#L46) · struct

```rust
pub struct SoundSlotKey(String);
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_application::values::SoundSlotKey`, `skin_editor_domain::audio::SoundSlotKey`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_domain::AudioComponent (声明)`, `skin_editor_domain::AudioComponent::new`, `skin_editor_domain::AudioComponent::slot`, `skin_editor_domain::SoundSlotKey::clone`, `skin_editor_domain::SoundSlotKey::cmp`, `skin_editor_domain::SoundSlotKey::eq`, `skin_editor_domain::SoundSlotKey::new`, `skin_editor_domain::SoundSlotKey::partial_cmp`

实现／约束（显式或 derive，无类继承）：

- `impl SoundSlotKey` — [源码:47](../../../crates/domain/src/audio.rs#L47)
- `impl Clone for SoundSlotKey` — [源码:45](../../../crates/domain/src/audio.rs#L45)
- `impl Debug for SoundSlotKey` — [源码:45](../../../crates/domain/src/audio.rs#L45)
- `impl Ord for SoundSlotKey` — [源码:45](../../../crates/domain/src/audio.rs#L45)
- `impl PartialEq for SoundSlotKey` — [源码:45](../../../crates/domain/src/audio.rs#L45)
- `impl PartialOrd for SoundSlotKey` — [源码:45](../../../crates/domain/src/audio.rs#L45)
- `impl Eq for SoundSlotKey` — [源码:45](../../../crates/domain/src/audio.rs#L45)
- `impl StructuralPartialEq for SoundSlotKey` — [源码:45](../../../crates/domain/src/audio.rs#L45)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn new(value: String) -> Result<Self, ModelError>` | impl SoundSlotKey | [源码:49](../../../crates/domain/src/audio.rs#L49) | skin_editor_domain::ModelError, skin_editor_domain::SoundSlotKey |
| `pub fn as_str(&self) -> &str` | impl SoundSlotKey | [源码:55](../../../crates/domain/src/audio.rs#L55) |  |
| `fn clone(&self) -> SoundSlotKey` | impl Clone for SoundSlotKey | [源码:45](../../../crates/domain/src/audio.rs#L45) | skin_editor_domain::SoundSlotKey |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for SoundSlotKey | [源码:45](../../../crates/domain/src/audio.rs#L45) |  |
| `fn cmp(&self, other: &SoundSlotKey) -> Ordering` | impl Ord for SoundSlotKey | [源码:45](../../../crates/domain/src/audio.rs#L45) | skin_editor_domain::SoundSlotKey |
| `fn eq(&self, other: &SoundSlotKey) -> bool` | impl PartialEq for SoundSlotKey | [源码:45](../../../crates/domain/src/audio.rs#L45) | skin_editor_domain::SoundSlotKey |
| `fn partial_cmp(&self, other: &SoundSlotKey) -> Option<Ordering>` | impl PartialOrd for SoundSlotKey | [源码:45](../../../crates/domain/src/audio.rs#L45) | skin_editor_domain::SoundSlotKey |

## skin_editor_domain::components::ComponentStorage

[源码:42](../../../crates/domain/src/components.rs#L42) · struct

```rust
struct ComponentStorage {
    roots: Vec<ComponentId>,
    nodes: BTreeMap<ComponentId, ComponentNode>,
    index: ForestIndex<ComponentId>,
    worlds: BTreeMap<ComponentId, Affine2>,
    members: BTreeMap<DocumentId, ComponentId>,
    content_transforms: BTreeMap<DocumentId, Affine2>,
}
```

声明引用：`skin_editor_domain::Affine2`, `skin_editor_domain::ComponentId`, `skin_editor_domain::ComponentNode`, `skin_editor_domain::DocumentId`, `skin_editor_domain::forest::ForestIndex`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_domain::ComponentTree (声明)`, `skin_editor_domain::components::ComponentStorage::default`, `skin_editor_domain::components::ComponentStorage::eq`

实现／约束（显式或 derive，无类继承）：

- `impl Debug for ComponentStorage` — [源码:41](../../../crates/domain/src/components.rs#L41)
- `impl Default for ComponentStorage` — [源码:41](../../../crates/domain/src/components.rs#L41)
- `impl PartialEq for ComponentStorage` — [源码:41](../../../crates/domain/src/components.rs#L41)
- `impl StructuralPartialEq for ComponentStorage` — [源码:41](../../../crates/domain/src/components.rs#L41)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for ComponentStorage | [源码:41](../../../crates/domain/src/components.rs#L41) |  |
| `fn default() -> ComponentStorage` | impl Default for ComponentStorage | [源码:41](../../../crates/domain/src/components.rs#L41) | skin_editor_domain::components::ComponentStorage |
| `fn eq(&self, other: &ComponentStorage) -> bool` | impl PartialEq for ComponentStorage | [源码:41](../../../crates/domain/src/components.rs#L41) | skin_editor_domain::components::ComponentStorage |

## skin_editor_domain::forest::ForestIndex

[源码:7](../../../crates/domain/src/forest.rs#L7) · struct

```rust
pub(crate) struct ForestIndex<I> {
    pub parents: BTreeMap<I, Option<I>>,
    pub preorder: Vec<I>,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_domain::components::ComponentStorage (声明)`, `skin_editor_domain::forest::ForestIndex::clone`, `skin_editor_domain::forest::ForestIndex::eq`, `skin_editor_domain::forest::index_forest (声明)`, `skin_editor_domain::layers::LayerStorage (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl<I: Clone> Clone for ForestIndex<I>` — [源码:6](../../../crates/domain/src/forest.rs#L6)
- `impl<I: Debug> Debug for ForestIndex<I>` — [源码:6](../../../crates/domain/src/forest.rs#L6)
- `impl<I> Default for ForestIndex<I>` — [源码:12](../../../crates/domain/src/forest.rs#L12)
- `impl<I: PartialEq> PartialEq for ForestIndex<I>` — [源码:6](../../../crates/domain/src/forest.rs#L6)
- `impl<I> StructuralPartialEq for ForestIndex<I>` — [源码:6](../../../crates/domain/src/forest.rs#L6)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> ForestIndex<I>` | impl<I: Clone> Clone for ForestIndex<I> | [源码:6](../../../crates/domain/src/forest.rs#L6) | skin_editor_domain::forest::ForestIndex |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl<I: Debug> Debug for ForestIndex<I> | [源码:6](../../../crates/domain/src/forest.rs#L6) |  |
| `fn default() -> Self` | impl<I> Default for ForestIndex<I> | [源码:13](../../../crates/domain/src/forest.rs#L13) |  |
| `fn eq(&self, other: &ForestIndex<I>) -> bool` | impl<I: PartialEq> PartialEq for ForestIndex<I> | [源码:6](../../../crates/domain/src/forest.rs#L6) | skin_editor_domain::forest::ForestIndex |

## skin_editor_domain::forest::index_forest

[源码:21](../../../crates/domain/src/forest.rs#L21) · fn

```rust
pub(crate) fn index_forest<I: Copy + Ord>(
    roots: &[I],
    edges: &BTreeMap<I, Vec<I>>,
    object: fn(I) -> ObjectId,
) -> Result<ForestIndex<I>, ModelError>
```

声明引用：`skin_editor_domain::ModelError`, `skin_editor_domain::ObjectId`, `skin_editor_domain::forest::ForestIndex`

## skin_editor_domain::identity::identity

[源码:3](../../../crates/domain/src/identity.rs#L3) · macro

```rust
macro_rules! identity {
    ($name:ident) => { ... };
}
```

## skin_editor_domain::layers::LayerStorage

[源码:51](../../../crates/domain/src/layers.rs#L51) · struct

```rust
struct LayerStorage {
    roots: Vec<LayerId>,
    nodes: BTreeMap<LayerId, LayerNode>,
    index: ForestIndex<LayerId>,
    worlds: BTreeMap<LayerId, Affine2>,
}
```

声明引用：`skin_editor_domain::Affine2`, `skin_editor_domain::LayerId`, `skin_editor_domain::LayerNode`, `skin_editor_domain::forest::ForestIndex`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_domain::LayerTree (声明)`, `skin_editor_domain::layers::LayerStorage::default`, `skin_editor_domain::layers::LayerStorage::eq`

实现／约束（显式或 derive，无类继承）：

- `impl Debug for LayerStorage` — [源码:50](../../../crates/domain/src/layers.rs#L50)
- `impl Default for LayerStorage` — [源码:50](../../../crates/domain/src/layers.rs#L50)
- `impl PartialEq for LayerStorage` — [源码:50](../../../crates/domain/src/layers.rs#L50)
- `impl StructuralPartialEq for LayerStorage` — [源码:50](../../../crates/domain/src/layers.rs#L50)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for LayerStorage | [源码:50](../../../crates/domain/src/layers.rs#L50) |  |
| `fn default() -> LayerStorage` | impl Default for LayerStorage | [源码:50](../../../crates/domain/src/layers.rs#L50) | skin_editor_domain::layers::LayerStorage |
| `fn eq(&self, other: &LayerStorage) -> bool` | impl PartialEq for LayerStorage | [源码:50](../../../crates/domain/src/layers.rs#L50) | skin_editor_domain::layers::LayerStorage |

