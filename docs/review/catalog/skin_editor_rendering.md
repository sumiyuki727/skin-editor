# skin_editor_rendering：编译器符号清单

快照：2026-10-30。来自含私有项的 rustdoc；显示编译器规范化可见域。含 derive 生成的实现，不把它们视作类继承。来源行可能指向宏定义／展开点。
引用列只列 rustdoc 解析出的项目类型链接，不能充当完整调用图；无链接不表示无引用。所有方法保留源码位置供逐项复核。测试和未启用的 cfg 分支另见源码索引。

## skin_editor_rendering::GpuComposer

[源码:32](../../../desktop/rendering/src/gpu.rs#L32) · struct

```rust
pub struct GpuComposer {
    upload: SourceUpload,
    staging: Mutex<ReusableUpload>,
    submission: Mutex<()>,
    pipeline: ComputePipeline,
    source: Mutex<Option<CachedSource>>,
    uploads: AtomicU64,
    allocations: AtomicU64,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_rendering::gpu::GpuComposer`

声明引用：`skin_editor_rendering::SourceUpload`, `skin_editor_rendering::gpu::CachedSource`, `skin_editor_rendering::upload::ReusableUpload`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::presentation::NativeHost (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl GpuComposer` — [源码:100](../../../desktop/rendering/src/gpu.rs#L100)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn new(device: &Device) -> Self` | impl GpuComposer | [源码:101](../../../desktop/rendering/src/gpu.rs#L101) |  |
| `pub fn with_upload(device: &Device, upload: SourceUpload) -> Self` | impl GpuComposer | [源码:104](../../../desktop/rendering/src/gpu.rs#L104) | skin_editor_rendering::SourceUpload |
| `pub fn staging_usage(&self) -> StagingUsage` | impl GpuComposer | [源码:127](../../../desktop/rendering/src/gpu.rs#L127) | skin_editor_rendering::StagingUsage |
| `pub fn clear_sources(&self)` | impl GpuComposer | [源码:130](../../../desktop/rendering/src/gpu.rs#L130) |  |
| `pub fn source_usage(&self) -> (u64, usize)` | impl GpuComposer | [源码:138](../../../desktop/rendering/src/gpu.rs#L138) |  |
| `pub fn source_allocations(&self) -> u64` | impl GpuComposer | [源码:148](../../../desktop/rendering/src/gpu.rs#L148) |  |
| `fn source_buffer( &self, device: &Device, queue: &Queue, plan: &Plan, images: &Images, ) -> Result<(Buffer, BTreeMap<BlobRef, u32>, GpuPreparation), RenderError>` | impl GpuComposer | [源码:151](../../../desktop/rendering/src/gpu.rs#L151) | skin_editor_rendering::GpuPreparation, skin_editor_rendering::Images, skin_editor_rendering::Plan, skin_editor_rendering::RenderError |
| `pub fn render( &self, device: &Device, queue: &Queue, plan: &Plan, images: &Images, background: bool, wake: impl FnOnce() + Send + 'static, ) -> Result<GpuImage, RenderError>` | impl GpuComposer | [源码:269](../../../desktop/rendering/src/gpu.rs#L269) | skin_editor_rendering::GpuImage, skin_editor_rendering::Images, skin_editor_rendering::Plan, skin_editor_rendering::RenderError |

## skin_editor_rendering::GpuImage

[源码:24](../../../desktop/rendering/src/gpu.rs#L24) · struct

```rust
pub struct GpuImage {
    pub texture: Texture,
    pub ready: Arc<AtomicBool>,
    pub completed: Arc<OnceLock<Instant>>,
    pub size: [u32; 2],
    pub preparation: GpuPreparation,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_rendering::gpu::GpuImage`

声明引用：`skin_editor_rendering::GpuPreparation`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::presentation::NativeTarget (声明)`, `skin_editor_desktop::presentation::Prepared (声明)`, `skin_editor_rendering::GpuComposer::render`

## skin_editor_rendering::GpuPreparation

[源码:10](../../../desktop/rendering/src/gpu.rs#L10) · struct

```rust
pub struct GpuPreparation {
    pub source_lock_ms: f64,
    pub submission_lock_ms: f64,
    pub staging_wait_ms: f64,
    pub staging_alloc_ms: f64,
    pub staging_copy_ms: f64,
    pub write_ms: f64,
    pub flush_ms: f64,
    pub create_ms: f64,
    pub pack_ms: f64,
    pub upload_ms: f64,
    pub encode_submit_ms: f64,
    pub reused: bool,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_rendering::gpu::GpuPreparation`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::presentation::PreparationTrace (声明)`, `skin_editor_rendering::GpuComposer::source_buffer`, `skin_editor_rendering::GpuImage (声明)`, `skin_editor_rendering::GpuPreparation::clone`, `skin_editor_rendering::GpuPreparation::default`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for GpuPreparation` — [源码:9](../../../desktop/rendering/src/gpu.rs#L9)
- `impl Debug for GpuPreparation` — [源码:9](../../../desktop/rendering/src/gpu.rs#L9)
- `impl Default for GpuPreparation` — [源码:9](../../../desktop/rendering/src/gpu.rs#L9)
- `impl Copy for GpuPreparation` — [源码:9](../../../desktop/rendering/src/gpu.rs#L9)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> GpuPreparation` | impl Clone for GpuPreparation | [源码:9](../../../desktop/rendering/src/gpu.rs#L9) | skin_editor_rendering::GpuPreparation |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for GpuPreparation | [源码:9](../../../desktop/rendering/src/gpu.rs#L9) |  |
| `fn default() -> GpuPreparation` | impl Default for GpuPreparation | [源码:9](../../../desktop/rendering/src/gpu.rs#L9) | skin_editor_rendering::GpuPreparation |

## skin_editor_rendering::Images

[源码:24](../../../desktop/rendering/src/lib.rs#L24) · type

```rust
pub type Images = BTreeMap<BlobRef, Arc<Raster>>;
```

声明引用：`skin_editor_rendering::Raster`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::frame_worker::decode (声明)`, `skin_editor_rendering::GpuComposer::render`, `skin_editor_rendering::GpuComposer::source_buffer`, `skin_editor_rendering::compose (声明)`, `skin_editor_rendering::gpu::pack_pixels (声明)`, `skin_editor_rendering::gpu::same_sources (声明)`

## skin_editor_rendering::MAX_DEPTH

[源码:16](../../../desktop/rendering/src/lib.rs#L16) · constant

```rust
pub const MAX_DEPTH: usize = 32;
```

## skin_editor_rendering::MAX_OPS

[源码:18](../../../desktop/rendering/src/lib.rs#L18) · constant

```rust
pub const MAX_OPS: usize = 65_536;
```

## skin_editor_rendering::MAX_PIXELS

[源码:17](../../../desktop/rendering/src/lib.rs#L17) · constant

```rust
pub const MAX_PIXELS: u64 = _; // 1_048_576u64
```

## skin_editor_rendering::MAX_STAGING_BYTES

[源码:9](../../../desktop/rendering/src/upload.rs#L9) · constant

```rust
pub const MAX_STAGING_BYTES: u64 = _; // 83_886_080u64
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_rendering::upload::MAX_STAGING_BYTES`

## skin_editor_rendering::Op

[源码:32](../../../desktop/rendering/src/lib.rs#L32) · enum

```rust
pub enum Op {
    Push(f32),
    Pop,
    Image {
        blob: BlobRef,
        inverse: Affine2,
        opacity: f32,
        clip_inverse: Affine2,
        canvas: PixelSize,
    },
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_rendering::Op::clone`, `skin_editor_rendering::Plan (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for Op` — [源码:31](../../../desktop/rendering/src/lib.rs#L31)
- `impl Debug for Op` — [源码:31](../../../desktop/rendering/src/lib.rs#L31)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> Op` | impl Clone for Op | [源码:31](../../../desktop/rendering/src/lib.rs#L31) | skin_editor_rendering::Op |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for Op | [源码:31](../../../desktop/rendering/src/lib.rs#L31) |  |

## skin_editor_rendering::Plan

[源码:44](../../../desktop/rendering/src/lib.rs#L44) · struct

```rust
pub struct Plan {
    pub(crate) size: PixelSize,
    pub(crate) ops: Vec<Op>,
    pub view: Affine2,
}
```

声明引用：`skin_editor_rendering::Op`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::frame_worker::Job (声明)`, `skin_editor_rendering::GpuComposer::render`, `skin_editor_rendering::GpuComposer::source_buffer`, `skin_editor_rendering::Plan::clone`, `skin_editor_rendering::compose (声明)`, `skin_editor_rendering::gpu::pack_pixels (声明)`, `skin_editor_rendering::gpu::same_sources (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Plan` — [源码:58](../../../desktop/rendering/src/lib.rs#L58)
- `impl Clone for Plan` — [源码:43](../../../desktop/rendering/src/lib.rs#L43)
- `impl Debug for Plan` — [源码:43](../../../desktop/rendering/src/lib.rs#L43)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn build( project: &SkinProject, target: Target, size: PixelSize, ) -> Result<Self, RenderError>` | impl Plan | [源码:59](../../../desktop/rendering/src/lib.rs#L59) | skin_editor_rendering::RenderError, skin_editor_rendering::Target |
| `pub(crate) fn document( &mut self, project: &SkinProject, id: DocumentId, content: Affine2, clip: Affine2, ) -> Result<(), RenderError>` | impl Plan | [源码:123](../../../desktop/rendering/src/lib.rs#L123) | skin_editor_rendering::RenderError |
| `pub fn sources(&self) -> Vec<BlobRef>` | impl Plan | [源码:210](../../../desktop/rendering/src/lib.rs#L210) |  |
| `fn clone(&self) -> Plan` | impl Clone for Plan | [源码:43](../../../desktop/rendering/src/lib.rs#L43) | skin_editor_rendering::Plan |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for Plan | [源码:43](../../../desktop/rendering/src/lib.rs#L43) |  |

## skin_editor_rendering::Raster

[源码:20](../../../desktop/rendering/src/lib.rs#L20) · struct

```rust
pub struct Raster {
    pub size: PixelSize,
    pub pixels: RasterPixels,
}
```

声明引用：`skin_editor_rendering::RasterPixels`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_infrastructure::CachedRaster (声明)`, `skin_editor_infrastructure::Images::load_raster`, `skin_editor_infrastructure::Images::raster`, `skin_editor_rendering::Images (声明)`, `skin_editor_rendering::Raster::clone`, `skin_editor_rendering::gpu::CachedSource (声明)`, `skin_editor_rendering::gpu::SourceIdentity (声明)`, `skin_editor_rendering::gpu::same_sources (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for Raster` — [源码:19](../../../desktop/rendering/src/lib.rs#L19)
- `impl Debug for Raster` — [源码:19](../../../desktop/rendering/src/lib.rs#L19)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> Raster` | impl Clone for Raster | [源码:19](../../../desktop/rendering/src/lib.rs#L19) | skin_editor_rendering::Raster |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for Raster | [源码:19](../../../desktop/rendering/src/lib.rs#L19) |  |

## skin_editor_rendering::RasterPixels

[源码:12](../../../desktop/rendering/src/pixels.rs#L12) · struct

```rust
pub struct RasterPixels(Arc<Allocation>);
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_rendering::pixels::RasterPixels`

声明引用：`skin_editor_rendering::pixels::Allocation`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_rendering::Raster (声明)`, `skin_editor_rendering::RasterPixels::clone`

实现／约束（显式或 derive，无类继承）：

- `impl RasterPixels` — [源码:13](../../../desktop/rendering/src/pixels.rs#L13)
- `impl AsRef<[u8]> for RasterPixels` — [源码:38](../../../desktop/rendering/src/pixels.rs#L38)
- `impl Clone for RasterPixels` — [源码:11](../../../desktop/rendering/src/pixels.rs#L11)
- `impl Debug for RasterPixels` — [源码:43](../../../desktop/rendering/src/pixels.rs#L43)
- `impl Deref for RasterPixels` — [源码:32](../../../desktop/rendering/src/pixels.rs#L32)
- `impl From<Vec<u8>> for RasterPixels` — [源码:24](../../../desktop/rendering/src/pixels.rs#L24)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn with_residency( bytes: Vec<u8>, residency: impl Send + Sync + 'static, ) -> Self` | impl RasterPixels | [源码:14](../../../desktop/rendering/src/pixels.rs#L14) |  |
| `pub fn is_shared(&self) -> bool` | impl RasterPixels | [源码:20](../../../desktop/rendering/src/pixels.rs#L20) |  |
| `fn as_ref(&self) -> &[u8] ⓘ` | impl AsRef<[u8]> for RasterPixels | [源码:39](../../../desktop/rendering/src/pixels.rs#L39) |  |
| `fn clone(&self) -> RasterPixels` | impl Clone for RasterPixels | [源码:11](../../../desktop/rendering/src/pixels.rs#L11) | skin_editor_rendering::RasterPixels |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for RasterPixels | [源码:44](../../../desktop/rendering/src/pixels.rs#L44) |  |
| `fn deref(&self) -> &[u8] ⓘ` | impl Deref for RasterPixels | [源码:34](../../../desktop/rendering/src/pixels.rs#L34) |  |
| `fn from(bytes: Vec<u8>) -> Self` | impl From<Vec<u8>> for RasterPixels | [源码:25](../../../desktop/rendering/src/pixels.rs#L25) |  |

## skin_editor_rendering::RenderError

[源码:50](../../../desktop/rendering/src/lib.rs#L50) · struct

```rust
pub struct RenderError(pub String);
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_rendering::GpuComposer::render`, `skin_editor_rendering::GpuComposer::source_buffer`, `skin_editor_rendering::Plan::build`, `skin_editor_rendering::Plan::document`, `skin_editor_rendering::RenderError::clone`, `skin_editor_rendering::RenderError::eq`, `skin_editor_rendering::compose (声明)`, `skin_editor_rendering::error (声明)`, `skin_editor_rendering::gpu::pack_pixels (声明)`, `skin_editor_rendering::math (声明)`, `skin_editor_rendering::upload::Budget::reserve`, `skin_editor_rendering::upload::ReusableUpload::write`, `skin_editor_rendering::upload::Slot::map`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for RenderError` — [源码:49](../../../desktop/rendering/src/lib.rs#L49)
- `impl Debug for RenderError` — [源码:49](../../../desktop/rendering/src/lib.rs#L49)
- `impl PartialEq for RenderError` — [源码:49](../../../desktop/rendering/src/lib.rs#L49)
- `impl Eq for RenderError` — [源码:49](../../../desktop/rendering/src/lib.rs#L49)
- `impl StructuralPartialEq for RenderError` — [源码:49](../../../desktop/rendering/src/lib.rs#L49)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> RenderError` | impl Clone for RenderError | [源码:49](../../../desktop/rendering/src/lib.rs#L49) | skin_editor_rendering::RenderError |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for RenderError | [源码:49](../../../desktop/rendering/src/lib.rs#L49) |  |
| `fn eq(&self, other: &RenderError) -> bool` | impl PartialEq for RenderError | [源码:49](../../../desktop/rendering/src/lib.rs#L49) | skin_editor_rendering::RenderError |

## skin_editor_rendering::SourceUpload

[源码:11](../../../desktop/rendering/src/upload.rs#L11) · enum

```rust
pub enum SourceUpload {
    QueueWrite,
    ReusableStaging,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_rendering::upload::SourceUpload`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::Editor::with_initial_content`, `skin_editor_desktop::Editor::with_render_state`, `skin_editor_desktop::Editor::with_upload`, `skin_editor_desktop::presentation::NativeHost (声明)`, `skin_editor_desktop::presentation::NativeHost::with_upload`, `skin_editor_desktop::presentation::RebuildSeed (声明)`, `skin_editor_desktop::verification::verify_source_buffer_reuse (声明)`, `skin_editor_rendering::GpuComposer (声明)`, `skin_editor_rendering::GpuComposer::with_upload`, `skin_editor_rendering::SourceUpload::clone`, `skin_editor_rendering::SourceUpload::default`, `skin_editor_rendering::SourceUpload::eq`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for SourceUpload` — [源码:10](../../../desktop/rendering/src/upload.rs#L10)
- `impl Debug for SourceUpload` — [源码:10](../../../desktop/rendering/src/upload.rs#L10)
- `impl Default for SourceUpload` — [源码:10](../../../desktop/rendering/src/upload.rs#L10)
- `impl PartialEq for SourceUpload` — [源码:10](../../../desktop/rendering/src/upload.rs#L10)
- `impl Copy for SourceUpload` — [源码:10](../../../desktop/rendering/src/upload.rs#L10)
- `impl Eq for SourceUpload` — [源码:10](../../../desktop/rendering/src/upload.rs#L10)
- `impl StructuralPartialEq for SourceUpload` — [源码:10](../../../desktop/rendering/src/upload.rs#L10)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> SourceUpload` | impl Clone for SourceUpload | [源码:10](../../../desktop/rendering/src/upload.rs#L10) | skin_editor_rendering::SourceUpload |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for SourceUpload | [源码:10](../../../desktop/rendering/src/upload.rs#L10) |  |
| `fn default() -> SourceUpload` | impl Default for SourceUpload | [源码:10](../../../desktop/rendering/src/upload.rs#L10) | skin_editor_rendering::SourceUpload |
| `fn eq(&self, other: &SourceUpload) -> bool` | impl PartialEq for SourceUpload | [源码:10](../../../desktop/rendering/src/upload.rs#L10) | skin_editor_rendering::SourceUpload |

## skin_editor_rendering::StagingUsage

[源码:17](../../../desktop/rendering/src/upload.rs#L17) · struct

```rust
pub struct StagingUsage {
    pub bytes: u64,
    pub peak_bytes: u64,
    pub allocations: u64,
    pub copies: u64,
    pub remaps: u64,
}
```

同一定义的重导出／别名（不重复计作对象）：`skin_editor_rendering::upload::StagingUsage`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_desktop::presentation::NativeHost::staging_usage`, `skin_editor_rendering::GpuComposer::staging_usage`, `skin_editor_rendering::StagingUsage::clone`, `skin_editor_rendering::StagingUsage::default`, `skin_editor_rendering::upload::Budget::usage`, `skin_editor_rendering::upload::ReusableUpload::usage`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for StagingUsage` — [源码:16](../../../desktop/rendering/src/upload.rs#L16)
- `impl Debug for StagingUsage` — [源码:16](../../../desktop/rendering/src/upload.rs#L16)
- `impl Default for StagingUsage` — [源码:16](../../../desktop/rendering/src/upload.rs#L16)
- `impl Copy for StagingUsage` — [源码:16](../../../desktop/rendering/src/upload.rs#L16)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> StagingUsage` | impl Clone for StagingUsage | [源码:16](../../../desktop/rendering/src/upload.rs#L16) | skin_editor_rendering::StagingUsage |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for StagingUsage | [源码:16](../../../desktop/rendering/src/upload.rs#L16) |  |
| `fn default() -> StagingUsage` | impl Default for StagingUsage | [源码:16](../../../desktop/rendering/src/upload.rs#L16) | skin_editor_rendering::StagingUsage |

## skin_editor_rendering::Target

[源码:26](../../../desktop/rendering/src/lib.rs#L26) · enum

```rust
pub enum Target {
    Document(DocumentId),
    Components,
    Workspace,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_rendering::Plan::build`, `skin_editor_rendering::Target::clone`, `skin_editor_rendering::Target::eq`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for Target` — [源码:25](../../../desktop/rendering/src/lib.rs#L25)
- `impl Debug for Target` — [源码:25](../../../desktop/rendering/src/lib.rs#L25)
- `impl PartialEq for Target` — [源码:25](../../../desktop/rendering/src/lib.rs#L25)
- `impl Copy for Target` — [源码:25](../../../desktop/rendering/src/lib.rs#L25)
- `impl Eq for Target` — [源码:25](../../../desktop/rendering/src/lib.rs#L25)
- `impl StructuralPartialEq for Target` — [源码:25](../../../desktop/rendering/src/lib.rs#L25)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> Target` | impl Clone for Target | [源码:25](../../../desktop/rendering/src/lib.rs#L25) | skin_editor_rendering::Target |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for Target | [源码:25](../../../desktop/rendering/src/lib.rs#L25) |  |
| `fn eq(&self, other: &Target) -> bool` | impl PartialEq for Target | [源码:25](../../../desktop/rendering/src/lib.rs#L25) | skin_editor_rendering::Target |

## skin_editor_rendering::compose

[源码:244](../../../desktop/rendering/src/lib.rs#L244) · fn

```rust
pub fn compose(
    plan: &Plan,
    images: &Images,
    background: bool,
) -> Result<Vec<u8>, RenderError>
```

声明引用：`skin_editor_rendering::Images`, `skin_editor_rendering::Plan`, `skin_editor_rendering::RenderError`

## skin_editor_rendering::error

[源码:51](../../../desktop/rendering/src/lib.rs#L51) · fn

```rust
pub(crate) fn error(text: &str) -> RenderError
```

声明引用：`skin_editor_rendering::RenderError`

## skin_editor_rendering::gamma

[源码:231](../../../desktop/rendering/src/lib.rs#L231) · fn

```rust
pub(crate) fn gamma(c: f32) -> f32
```

## skin_editor_rendering::gpu::CachedSource

[源码:43](../../../desktop/rendering/src/gpu.rs#L43) · struct

```rust
struct CachedSource {
    identity: Vec<(BlobRef, Weak<Raster>)>,
    buffer: Buffer,
    offsets: BTreeMap<BlobRef, u32>,
    bytes: usize,
}
```

声明引用：`skin_editor_rendering::Raster`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_rendering::GpuComposer (声明)`

## skin_editor_rendering::gpu::PixelOffsets

[源码:41](../../../desktop/rendering/src/gpu.rs#L41) · type

```rust
type PixelOffsets = BTreeMap<BlobRef, u32>;
```

## skin_editor_rendering::gpu::SHADER

[源码:435](../../../desktop/rendering/src/gpu.rs#L435) · constant

```rust
const SHADER: &str = r#"
struct Op { info:vec4<u32>, inverse0:vec4<f32>, inverse1:vec4<f32>, clip0:vec4<f32>, clip1:vec4<f32> }
struct Params { size:vec2<u32>, count:u32, background:u32 }
@group(0) @binding(0) var<storage,read> pixels:array<u32>;
@group(0) @binding(1) var<storage,read> ops:array<Op>;
@group(0) @binding(2) var<uniform> params:Params;
@group(0) @binding(3) var output:texture_storage_2d<rgba8unorm,write>;
fn point(a:vec4<f32>,b:vec4<f32>,p:vec2<f32>)->vec2<f32> {return vec2(a.x*p.x+a.z*p.y+b.x,a.y*p.x+a.w*p.y+b.y);}
fn linear(c:vec3<f32>)->vec3<f32> {return select(pow((c+vec3(0.055))/1.055,vec3(2.4)),c/12.92,c<=vec3(0.04045));}
fn gamma(c:vec3<f32>)->vec3<f32> {return select(1.055*pow(c,vec3(1.0/2.4))-vec3(0.055),c*12.92,c<=vec3(0.0031308));}
fn over(back:vec4<f32>,front:vec4<f32>)->vec4<f32> {return front+back*(1.0-front.a);}
@compute @workgroup_size(8,8) fn main(@builtin(global_invocation_id) id:vec3<u32>) {
 if any(id.xy>=params.size) {return;}
 let p=vec2<f32>(id.xy)+vec2(0.5);var color=vec4<f32>(0.0);var stack:array<vec4<f32>,32>;var weights:array<f32,32>;var depth=0u;
 for(var i=0u;i<params.count;i+=1u) {
  let op=ops[i];
  if op.info.x==0u {stack[depth]=color;weights[depth]=op.inverse1.z;depth+=1u;color=vec4(0.0);}
  else if op.info.x==2u {depth-=1u;color=over(stack[depth],color*weights[depth]);}
  else {
   let clip=point(op.clip0,op.clip1,p);let at=point(op.inverse0,op.inverse1,p);
   if all(clip>=vec2(0.0)) && all(clip<op.clip1.zw) && all(at>=vec2(0.0)) && all(at<vec2<f32>(op.info.zw)) {
    let pos=vec2<u32>(floor(at));let rgba=pixels[op.info.y+pos.y*op.info.z+pos.x];
    let value=vec4<f32>(f32(rgba&255u),f32((rgba>>8u)&255u),f32((rgba>>16u)&255u),f32(rgba>>24u))/255.0;
    let alpha=value.a*op.inverse1.z;color=over(color,vec4(linear(value.rgb)*alpha,alpha));
   }
  }
 }
 if params.background!=0u {let grey=select(0.055,0.035,((id.x/16u+id.y/16u)%2u)==0u);color=over(vec4(vec3(grey),1.0),color);}
 var rgb=vec3<f32>(0.0);if color.a>0.0 {rgb=gamma(clamp(color.rgb/color.a,vec3(0.0),vec3(1.0)));}
 textureStore(output,vec2<i32>(id.xy),vec4(rgb,color.a));
}
"#;
```

## skin_editor_rendering::gpu::SourceIdentity

[源码:42](../../../desktop/rendering/src/gpu.rs#L42) · type

```rust
type SourceIdentity = Vec<(BlobRef, Weak<Raster>)>;
```

声明引用：`skin_editor_rendering::Raster`

## skin_editor_rendering::gpu::pack_pixels

[源码:59](../../../desktop/rendering/src/gpu.rs#L59) · fn

```rust
fn pack_pixels(
    plan: &Plan,
    images: &Images,
    limit: usize,
) -> Result<(Vec<u8>, BTreeMap<BlobRef, u32>), RenderError>
```

声明引用：`skin_editor_rendering::Images`, `skin_editor_rendering::Plan`, `skin_editor_rendering::RenderError`

## skin_editor_rendering::gpu::same_sources

[源码:49](../../../desktop/rendering/src/gpu.rs#L49) · fn

```rust
fn same_sources(
    identity: &Vec<(BlobRef, Weak<Raster>)>,
    plan: &Plan,
    images: &Images,
) -> bool
```

声明引用：`skin_editor_rendering::Images`, `skin_editor_rendering::Plan`, `skin_editor_rendering::Raster`

## skin_editor_rendering::inside

[源码:241](../../../desktop/rendering/src/lib.rs#L241) · fn

```rust
pub(crate) fn inside(p: [f64; 2], size: PixelSize) -> bool
```

## skin_editor_rendering::linear

[源码:224](../../../desktop/rendering/src/lib.rs#L224) · fn

```rust
pub(crate) fn linear(c: f32) -> f32
```

## skin_editor_rendering::math

[源码:54](../../../desktop/rendering/src/lib.rs#L54) · fn

```rust
pub(crate) fn math(_: ModelError) -> RenderError
```

声明引用：`skin_editor_rendering::RenderError`

## skin_editor_rendering::over

[源码:238](../../../desktop/rendering/src/lib.rs#L238) · fn

```rust
pub(crate) fn over(back: [f32; 4], front: [f32; 4]) -> [f32; 4]
```

## skin_editor_rendering::pixels::Allocation

[源码:6](../../../desktop/rendering/src/pixels.rs#L6) · struct

```rust
struct Allocation {
    bytes: Arc<[u8]>,
    _residency: Option<Box<dyn Send + Sync>>,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_rendering::RasterPixels (声明)`

## skin_editor_rendering::upload::Allocation

[源码:69](../../../desktop/rendering/src/upload.rs#L69) · struct

```rust
struct Allocation {
    buffer: Buffer,
    mapped: AtomicU8,
    _reservation: Reservation,
}
```

声明引用：`skin_editor_rendering::upload::Reservation`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_rendering::upload::Slot (声明)`

## skin_editor_rendering::upload::Budget

[源码:25](../../../desktop/rendering/src/upload.rs#L25) · struct

```rust
struct Budget {
    bytes: AtomicU64,
    peak: AtomicU64,
    allocations: AtomicU64,
    copies: AtomicU64,
    remaps: AtomicU64,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_rendering::upload::Budget::default`, `skin_editor_rendering::upload::Reservation (声明)`, `skin_editor_rendering::upload::ReusableUpload (声明)`, `skin_editor_rendering::upload::Slot::map`

实现／约束（显式或 derive，无类继承）：

- `impl Budget` — [源码:36](../../../desktop/rendering/src/upload.rs#L36)
- `impl Default for Budget` — [源码:24](../../../desktop/rendering/src/upload.rs#L24)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn reserve(self: &Arc<Self>, bytes: u64) -> Result<Reservation, RenderError>` | impl Budget | [源码:37](../../../desktop/rendering/src/upload.rs#L37) | skin_editor_rendering::RenderError, skin_editor_rendering::upload::Reservation |
| `fn usage(&self) -> StagingUsage` | impl Budget | [源码:54](../../../desktop/rendering/src/upload.rs#L54) | skin_editor_rendering::StagingUsage |
| `fn default() -> Budget` | impl Default for Budget | [源码:24](../../../desktop/rendering/src/upload.rs#L24) | skin_editor_rendering::upload::Budget |

## skin_editor_rendering::upload::Reservation

[源码:32](../../../desktop/rendering/src/upload.rs#L32) · struct

```rust
struct Reservation {
    budget: Arc<Budget>,
    bytes: u64,
}
```

声明引用：`skin_editor_rendering::upload::Budget`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_rendering::upload::Allocation (声明)`, `skin_editor_rendering::upload::Budget::reserve`

实现／约束（显式或 derive，无类继承）：

- `impl Drop for Reservation` — [源码:64](../../../desktop/rendering/src/upload.rs#L64)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn drop(&mut self)` | impl Drop for Reservation | [源码:65](../../../desktop/rendering/src/upload.rs#L65) |  |

## skin_editor_rendering::upload::ReusableUpload

[源码:120](../../../desktop/rendering/src/upload.rs#L120) · struct

```rust
pub(crate) struct ReusableUpload {
    slot: Option<Slot>,
    budget: Arc<Budget>,
}
```

声明引用：`skin_editor_rendering::upload::Budget`, `skin_editor_rendering::upload::Slot`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_rendering::GpuComposer (声明)`, `skin_editor_rendering::upload::ReusableUpload::default`

实现／约束（显式或 derive，无类继承）：

- `impl ReusableUpload` — [源码:130](../../../desktop/rendering/src/upload.rs#L130)
- `impl Default for ReusableUpload` — [源码:119](../../../desktop/rendering/src/upload.rs#L119)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `pub fn usage(&self) -> StagingUsage` | impl ReusableUpload | [源码:131](../../../desktop/rendering/src/upload.rs#L131) | skin_editor_rendering::StagingUsage |
| `pub fn clear(&mut self)` | impl ReusableUpload | [源码:134](../../../desktop/rendering/src/upload.rs#L134) |  |
| `pub fn write( &mut self, device: &Device, queue: &Queue, destination: &Buffer, pixels: &[u8], ) -> Result<UploadTiming, RenderError>` | impl ReusableUpload | [源码:137](../../../desktop/rendering/src/upload.rs#L137) | skin_editor_rendering::RenderError, skin_editor_rendering::upload::UploadTiming |
| `fn default() -> ReusableUpload` | impl Default for ReusableUpload | [源码:119](../../../desktop/rendering/src/upload.rs#L119) | skin_editor_rendering::upload::ReusableUpload |

## skin_editor_rendering::upload::Slot

[源码:79](../../../desktop/rendering/src/upload.rs#L79) · struct

```rust
struct Slot {
    allocation: Arc<Allocation>,
    bytes: u64,
    state: State,
}
```

声明引用：`skin_editor_rendering::upload::Allocation`, `skin_editor_rendering::upload::State`

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_rendering::upload::ReusableUpload (声明)`

实现／约束（显式或 derive，无类继承）：

- `impl Slot` — [源码:84](../../../desktop/rendering/src/upload.rs#L84)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn map(&mut self, device: &Device, budget: &Budget) -> Result<(), RenderError>` | impl Slot | [源码:85](../../../desktop/rendering/src/upload.rs#L85) | skin_editor_rendering::RenderError, skin_editor_rendering::upload::Budget |

## skin_editor_rendering::upload::State

[源码:74](../../../desktop/rendering/src/upload.rs#L74) · enum

```rust
enum State {
    Mapped,
    Submitted(SubmissionIndex),
    Mapping(SubmissionIndex),
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_rendering::upload::Slot (声明)`

## skin_editor_rendering::upload::UploadTiming

[源码:125](../../../desktop/rendering/src/upload.rs#L125) · struct

```rust
pub(crate) struct UploadTiming {
    pub wait_ms: f64,
    pub allocation_ms: f64,
    pub copy_submit_ms: f64,
}
```

反向签名引用（仅已解析的项目类型链接，非完整调用图）：`skin_editor_rendering::upload::ReusableUpload::write`, `skin_editor_rendering::upload::UploadTiming::clone`, `skin_editor_rendering::upload::UploadTiming::default`

实现／约束（显式或 derive，无类继承）：

- `impl Clone for UploadTiming` — [源码:124](../../../desktop/rendering/src/upload.rs#L124)
- `impl Debug for UploadTiming` — [源码:124](../../../desktop/rendering/src/upload.rs#L124)
- `impl Default for UploadTiming` — [源码:124](../../../desktop/rendering/src/upload.rs#L124)
- `impl Copy for UploadTiming` — [源码:124](../../../desktop/rendering/src/upload.rs#L124)

| 方法签名 | 所属实现 | 来源 | 签名类型引用 |
| --- | --- | --- | --- |
| `fn clone(&self) -> UploadTiming` | impl Clone for UploadTiming | [源码:124](../../../desktop/rendering/src/upload.rs#L124) | skin_editor_rendering::upload::UploadTiming |
| `fn fmt(&self, f: &mut Formatter<'_>) -> Result` | impl Debug for UploadTiming | [源码:124](../../../desktop/rendering/src/upload.rs#L124) |  |
| `fn default() -> UploadTiming` | impl Default for UploadTiming | [源码:124](../../../desktop/rendering/src/upload.rs#L124) | skin_editor_rendering::upload::UploadTiming |

