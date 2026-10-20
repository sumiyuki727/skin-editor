# P0 egui 原生候选实验

独立、可丢弃的桌面框架实验，不是产品工程或 stable 模拟器。仅依赖自有契约探针及公开 Rust 库，不读取旧 stable 源码、游戏资源或用户项目。

## 验证内容

原有轻量入口包含中央纹理画布、场景/工具控制、占位素材/图层列表与视图参数。另有 `standard-load` 入口，从自制数据集读取真实 PNG、文档元数据与嵌套组参数，按需解码并通过 GPU 绘制。均没有产品素材导入、项目编辑、历史树或游玩。

自动脚本运行隐藏窗口、绘制 120 帧后冻结模拟、回读自身渲染目标、修改材质、再次回读并比较画布像素及模拟时间。输出两张 PNG 和结构化结果，随后退出；不调用系统窗口捕获，不向其他应用发送输入，也不读取 stable 的画面。

此负载是 50 个共享一张 32×32 纹理的矩形。它能验证原生 GPU 路径及冻结下刷新，**不等于标准项目负载**，不能据其 CPU/内存样本宣布性能预算达标或技术栈胜出。隐藏窗口的自动脚本也不能替代物理输入、输入法、拖放、高 DPI 与真实声音测试。

### 标准 PNG 负载入口

`standard-load` 使用独立生成的 100 张 1024×1024 RGBA PNG、100 张 64×64 缩略图、10 个文档与 500 图层；当前文档 50 层共享 10 个原图，列表显示 12 张缩略图。目录包含中文/日文字符。原图只读取尺寸，首次仅解码当前文档引用的 10 张，之后按需加载；CPU 与 GPU 纹理缓存分别上限 128 MiB。压缩文件、CPU 预乘像素、GPU 纹理和回读文件分开，原始 PNG 不被修改。

自动阶段依次为空界面、首次加载/预热、目标 60 Hz 的 300 次脚本变换、冻结后替换 PNG 的两次 GPU 回读、静止 60 秒、3 个即时文档请求及 CPU 缓存重新上传、20 轮遍历 10 文档及清空缓存、关闭后 2 秒收尾和 60 秒静止观察。标准测量使用可见的自有原生窗口、AutoNoVsync 和脚本时限，避免框架对隐藏逻辑额外休眠干扰；不捕获系统窗口。暂停无变化时不循环申请绘制。每轮记录缓存与进程样本，实际编辑间隔、UI 帧与布局 pass 分开，不能用目标频率冒充实际频率。

此入口读取只读 fixture，修改的是实验视图参数，没有建立或绕过产品 SkinProject/Command。默认路径保留同步加载；`--async-load` 改用后台元数据任务、单线程 PNG 解码、代次失效与每帧两张/8 MiB 上传限额。只读 fixture 不因此获得项目写入或事务入口；运行中的解码与文件 I/O 没有强制中断保证。两层嵌套组仅传递平移、正的等比缩放与 opacity；按绘制顺序 source-over，不实现隔离组、蒙版、特效或导出。PNG 转为 egui 的预乘 sRGBA，LINEAR 采样、逻辑点定位；这不是最终导出色彩/像素契约。

内存脚本以 100 ms 目标间隔读取自有进程的 Windows 工作集、私有字节、CPU 时间，以及 PDH 中按 PID/适配器统计的 GPU Dedicated/Shared Usage；不启动 PowerShell 逐帧采样，不调用窗口 API。GPU 缓存字节只计纹理 payload，不能等同于 GPU 驻留。eframe 的 CPU 帧耗时排除 vsync；编辑到回读延迟包括队列、GPU、复制和回调，不能代表物理输入到显示延迟。独立 GPU 执行时间戳尚未接入。

冻结替换使用单独的视图覆盖纹理（4 MiB），不改变原始 Asset 与其缓存身份；缩略图 payload 为 192 KiB，均与 128 MiB 原图缓存分开。加载是本地文件/解码/上传排队耗时，不包括等待 GPU 完成。逐次变换回读检查动画标记之外的图像区域确实变化，冻结前后另检查完整画布变化而时钟不动。

上传直接传递共享的 `Arc<ColorImage>`，不再复制完整像素平面；已有无窗口用例检查共享身份。空列表显示静态占位，避免内容已清空时继续为滚动条淡出请求高频绘制。修改前的回归用例观察到额外动画帧，修改后通过；不全局关闭编辑器动画。

阶段记录另含重绘请求来源、100 ms 帧数分布、已登记原图的存活引用、自有纹理在 renderer 中的存活数，以及 wgpu 分配器的 allocated/reserved（后端可不支持）。阶段事件发生在当前帧提交前，释放结论使用收尾后的 Closed/Done。分配器 reserved 与 OS Dedicated/Shared 是不同口径；自有引用为零也不证明整个驱动不存在泄漏。

## 依赖和维护成本

| 依赖 | 用途与边界 |
| --- | --- |
| eframe/egui `=0.37.0` | 原生 UI、窗口和 wgpu 渲染；启用默认字体、系统字体、系统剪贴板及桌面后端。关闭持久化等非必要功能，不实现网络服务；传递依赖以 Cargo.lock 为准 |
| image `=0.25.10` | PNG 元数据、受限解码和自有回读编码；仅启用 PNG 功能 |
| serde_json | 输出结构化验证记录，版本锁定于 Cargo.lock |
| p0-preview-contract | 本仓库独立时钟/刷新探针；保持零第三方依赖，不因此变成产品核心 |

eframe 此版本要求 Rust 1.95。本机为实验准备了项目私有工具链及 Cargo 缓存，位于 `.p0-tools`，不进入版本管理、不作为产品分发内容。原系统 Rust 1.80 仍可运行原有无依赖探针。首次编译与工具链缓存占用属于开发成本，运行时 CPU/内存另行记录。

已锁定的框架选择仅用于 P0 测量，升级时必须重跑冻结、回读与资源检查；不把上游 main 当作固定依赖。公开来源：[eframe/egui 0.37.0](https://github.com/emilk/egui/releases/tag/0.37.0)。

## 检查与运行

### 准备实验依赖

Windows 下需要已有 Rustup、MSVC x64 及 Windows SDK。在仓库根的 PowerShell 中，已验证的项目局部准备流程为：

```powershell
$env:RUSTUP_HOME = Join-Path (Get-Location) '.p0-tools/rustup'
$env:CARGO_HOME = Join-Path (Get-Location) '.p0-tools/cargo'
rustup toolchain install 1.95.0 --profile minimal --no-self-update --component clippy --component rustfmt
& '.\.p0-tools\rustup\toolchains\1.95.0-x86_64-pc-windows-msvc\bin\cargo.exe' fetch --locked --manifest-path experiments/p0-egui/Cargo.toml
```

准备阶段需要访问官方工具链和 crates.io；后续检查使用 `--offline --locked`。这些环境变量只作用于当前 shell 及其子进程；不修改系统工具链目录或全局 PATH。其他平台的工具链准备流程尚未实机验证。

### 自动验证

在当前已准备依赖的 Windows x64 环境，仓库根执行：

```text
python tools/check_p0_gui.py --run-native
```

它检查依赖边界、格式、Clippy，以两个并行编译任务构建 release，然后执行内部渲染验证。每次使用新的 `artifacts/p0/egui-*` 目录；已有结果不覆盖。输出包括 `result.json`、`render-0.png`、`render-1.png` 与有限的 `process-samples.json`。是否成功以结果中的 status 为准；构建成功不等于本机 GPU 验证通过。

仅构建检查可省略 `--run-native`。平台原生链接器通过现有工具发现逻辑配置；macOS/Linux 尚待实机运行，不能以依赖声明代替验证。

标准负载及 Windows 进程/GPU 采样入口：

```text
python tools/check_p0_gui.py --run-standard
```

脚本同时检查格式、Clippy、无窗口加载/缓存测试及 release 构建。每次生成新的 `artifacts/p0/standard-fixture-*`（含逐文件 SHA-256 清单）和 `standard-*`（结果、逐阶段日志、两张冻结 PNG 与完整进程样本）；已有证据不覆盖。核心行为成功与性能预算逐项分开，未达预算不修改目标。完整结果与验证状态见 [P0 报告](../../docs/p0/REPORT.md)。此采样实现只适用于 Windows，其他平台必须补充实际采样适配和实机验证。

回读与分配策略的完整对照：

```text
python tools/check_p0_gui.py --run-comparison
```

共用同一数据集、同一 release 可执行文件，依次运行 Performance/无回读、MemoryUsage/无回读、MemoryUsage/逐次回读。两组无回读不调用截图接口，也不做冻结画面的像素判定；对应结果为 null。第三组保留 300 次编辑回读及冻结 PNG 证据。每组都执行 60 秒暂停和 60 秒关闭后观察，不并行运行 GPU 测量。

`--run-standard` 仍使用原来的 Performance 策略，不静默替换旧测量口径。MemoryUsage 是 wgpu 的公开分配提示，有些后端可能忽略；当前实验只比较本机表现，不设手工堆块大小、不修改上游库、不重建 GPU 设备来降低统计值。

历史实验结果 schema v2：区分是否请求回读、未测像素/延迟、内存提示、原图/纹理引用和 allocated/reserved。旧 v1 证据保留为历史记录，不伪造缺失字段或改写旧成绩。`standard-load` 的 `--diagnostic` 为缩短验证，仅 30 次编辑、一轮切换和较短观察，不用于正式预算；该选项不由完整对照入口启用。

加载路径比较入口：

```text
python tools/check_p0_gui.py --run-loading-comparison
```

三组共用同一 exe 和数据集，均为 MemoryUsage：同步无回读、后台无回读、后台逐次回读。与上面的分配策略比较是不同批次，不能混用旧成绩作为因果对照。背景与限制见[后台加载报告](../../docs/p0/ASYNC_LOADING.md)。元数据读取任务和 PNG 解码分阶段运行；最多一个 PNG 在途请求和一个结果槽，每帧限额同时覆盖缓存命中、缩略图与冻结替换，切换仍保留旧文档直到目标纹理全部提交。

新结果为 schema v3，旧 `document_load_cpu_ms` 在后台组为 null，以 `document_load_wall_ms` 记录排队至纹理提交耗时。`timing-samples.json` 另保存原始耗时、间隔和每帧上传记录，最大耗时与 p95 分开。历史结果不改写；没有产品项目格式需要迁移。后台任务仅生成派生数据，正式可变资源仍须使用包含内容修订的键。

加载 CPU 与私有提交定位入口：

```text
python tools/check_p0_gui.py --run-loader-tuning
```

同一数据集顺序执行立即通知无回读、合并通知无回读、合并通知逐次回读三组完整验证，以及上传并绘制、仅上传、不上传原图三组诊断。`--paced-loading` 只用于后台路径，目标 60 Hz 合并通知、最多两个在途请求；结果与 UI 交接 payload 上界为 12 MiB，每帧仍限两张／8 MiB。取消不强制中断已执行的解码，不增加工作线程或依赖。

`--profile-memory` 保留 20 轮，仅缩短编辑和静止观察，额外输出 `memory-trace.jsonl` 与 `virtual-memory.json`；这部分开销不混入常规 CPU 对照。`--no-original-draw` 与 `--no-original-upload` 仅在内存诊断且关闭回读时允许，不能当作正常功能或预算通过。进程记录增加 OS 生命周期提交峰值、完整循环的样本 CPU 工作量和墙钟，结果 schema v4／进程记录 v3；旧证据不覆盖。详见[方法与结果](../../docs/p0/LOADER_TUNING.md)。

GPU 上传归因和缓存验证入口（需要先生成或复用只读标准 fixture）：

```text
python tools/check_p0_gpu_memory.py --fixture artifacts/p0/gpu-fixture --diagnostics-only
python tools/check_p0_gpu_memory.py --fixture artifacts/p0/gpu-fixture --compact
```

结果 schema v5 增加纹理创建／更新次数、缓存真实预算、诊断堆统计、GPU 等待和独立纹理回读；历史 v1–v4 文件保持原样，不涉及产品项目格式迁移。`--reuse-textures` 只对同尺寸 1024² 原图使用全尺寸 partial delta，避免 full delta 重建后端纹理；逻辑 ID 与物理槽独立，旧／新视图同时固定。每轮故意清空逻辑缓存是资源压力脚本，未验收切换过程的完整视觉连续性；仅提交完成的文档做像素比较。

`--small-texture-cache` 使用 80 MiB GPU 原图预算，满足本 fixture 当前＋下一文档的 20 张纹理，CPU 缓存仍为 128 MiB。空闲槽计入预算，最终关闭会释放整个池，不能把 retained pool 当作已释放。正式产品有变尺寸素材，不能直接继承这个容量或无条件复用。

`heap-profile` feature 是可选 `stats_alloc = 0.1.10`（MIT，无传递依赖，基于 Rust System 分配器）的测量构建，适用于原生 Rust 目标；本轮只验证 Windows。逐次原子计数有 CPU 成本，统计 snapshot 也非原子，不纳入默认构建的性能成绩。它报告请求字节，不计分配器已空闲但保留的页、原生库、驱动和 GPU 专用内存。工具分别保存默认／诊断 exe，始终记录实际哈希。`--drain-uploads` 仅允许诊断，逐轮等待已提交 GPU 工作完成，最长 2 秒；这不是常规编辑路径的优化。详见[证据与限制](../../docs/p0/GPU_UPLOAD.md)。

`python tools/p0_gpu_repeats.py --fixture artifacts/p0/gpu-fixture` 固定 exe／PDB、Rust 源码和测量脚本后，交替执行三种纹理策略各三次完整负载。进程记录 v4 增加启动阶段的模块元数据和允许记录的 Vulkan 子进程环境字段；失败的模块枚举不能当作干净图形环境，旧记录不补造字段。独立 Windows x64 分配诊断 `python tools/p0_vulkan_memory.py --repetitions 3` 不依赖本实验窗口或 wgpu，不作为性能结果；`--settle-seconds 3` 增加资源释放后观察。详见[重复与分配寿命](../../docs/p0/ALLOCATION_ATTRIBUTION.md)。

如需查看可交互占位窗口，可在本机 release 构建后执行（需要新的输出路径）：

```text
experiments/p0-egui/target/release/p0-egui.exe --interactive artifacts/p0/egui-manual
```

交互模式尚未人工验收，后续通过应用自身导出图像或用户提供图片记录，不依赖系统窗口捕获。它不向外部 stable 注入或导出任何修改。

## 资源寿命与合成验证

```text
python tools/check_p0_lifetime.py --fixture artifacts/p0/gpu-fixture
python tools/check_p0_composition.py
python tools/check_p0_composition.py --run-native
```

`--bounded-uploads` 是明确选择的实验方案，将未回执工作限制在一个上传帧内；回调复用 60 Hz 合并通知，既不阻塞 UI，也不等同于驱动已经回收虚拟页。`--lifetime-trace` 要求 `--profile-memory`，单独记录排队、回执、清除与关闭后的观察，额外刷新成本不混入完整性能。结果 schema v6，旧记录保留，详见[寿命记录](../../docs/p0/UPLOAD_LIFETIME.md)。

`composition-proof` 是无窗口 Vulkan 对照，独立于 standard-load 的绘制路径。纯 CPU 模块编译只读计划，CPU／WGSL 消费同一计划并进行 PNG 像素比较；默认测试不初始化 GPU。直接引用锁定依赖树中已有的 `pollster 1.0.1`（Apache-2.0／MIT）以等待原生 GPU 初始化，不新增第三方包版本。当前共 70 项无窗口测试，包含设备代次、恢复锚点及桌面边界回归。语义与边界见[合成记录](../../docs/p0/COMPOSITION.md)、[联动记录](../../docs/p0/TRANSPORT.md)和[后台合成交接](../../docs/p0/ASYNC_COMPOSITION.md)。

```text
python tools/check_p0_composition.py --fixture artifacts/p0/gpu-fixture
python tools/check_p0_interaction.py --run-native
experiments/p0-egui/target/release/interaction-proof.exe --interactive
```

标准合成入口复用已生成的 100 原图／10 文档 fixture，先校验所有文件哈希，再顺序重复两次 1024²、每文档 50 层的隔离组合成。GPU 源／操作／输出缓冲常驻复用，300 次温热派发重复静态计划；时间戳只覆盖 compute pass。详见[标准规模记录](../../docs/p0/COMPOSITION_SCALE.md)，不代表完整应用预算。

交互入口用同一控件／命令会话路径验证暂停下草稿、Esc 取消及一次提交，并回读自有渲染器；所有输入仍为合成 egui 事件。交互模式可以人工拖动矩形、取消和选择旧节点后分叉，但物理鼠标／键盘、IME、焦点及 DPI／全屏尚未人工验收。详见[交互记录](../../docs/p0/INTERACTION.md)。两个实验尚未连接为正式编辑器，不读取旧源码或客户端资源。

## 连续编辑与直接 GPU 呈现

```text
python tools/check_p0_edit_composition.py --fixture artifacts/p0/gpu-fixture
experiments/p0-egui/target/release/interaction-proof.exe --interactive --fixture artifacts/p0/gpu-fixture
```

新入口共享纯 CPU 计划与 GPU 执行器，按同一命令会话的位置投影更新标准文档首个图层；普通路径每次只上传 4480 字节操作，GPU 合成及显示适配后直接注册纹理供 egui 使用。自动入口顺序运行普通／逐次回读各两次，每次 300 个真实不同的草稿投影；原图上传一次，四个关键状态做 CPU／GPU 和显示像素对照。详见[连续编辑记录](../../docs/p0/EDIT_COMPOSITION.md)。

该实验使用 Windows Vulkan、MemoryUsage 和 AutoNoVsync，初始 PNG 仍同步载入；没有正式项目、稳定的外部接口或游玩逻辑。独立音频工具 `python tools/p0_audio_device.py` 只验证 WASAPI 默认输出的静音流时钟，未接入此 UI，见[音频记录](../../docs/p0/AUDIO_DEVICE.md)。没有新增第三方依赖或 Rust unsafe。

## 可选设备时钟联动

```text
python tools/check_p0_gui.py --device-audio
python tools/check_p0_transport.py --fixture artifacts/p0/gpu-fixture --repetitions 3
```

新增 `transport-proof` 将真实静音设备流、纯 Transport 状态机、同一控件／命令会话与 GPU 合成连接；默认 feature 关闭。第一项无窗口检查／构建，另运行适配器的 11 项换算／PCM／故障／寿命测试；第二项显式打开 Windows 默认输出和自有窗口，顺序执行三次。所有指针操作隔离在可选 [p0-wasapi](../p0-wasapi/README.md)，本包继续禁止 unsafe，静音路径不引入解码依赖。三轮暂停／恢复、输入丢弃、取消／单次提交及像素验证通过，不代表可听延迟、物理输入或实际游玩。初始载入仍同步；后台接入的像素／组语义差异已有回归，详见[联动证据及下一步](../../docs/p0/TRANSPORT.md)。

## 后台文档与候选呈现

```text
python tools/check_p0_switch.py --fixture artifacts/p0/gpu-fixture
python tools/check_p0_switch.py --fixture artifacts/p0/gpu-fixture --smoke
```

第一个入口在 Windows 顺序执行像素诊断及两次无回读完整观察；第二个只做诊断。新 `switch-proof` 使用原始 RGBA 后台文档 worker、共享计划和有界源写入，保留组结构；GPU 首帧完成后交接，过期候选、取消和缺失 fixture 不替换当前视图。每文档的手势状态独立且初始暂停，20 次切换后观察静止与关闭，未接入真实音频流或正式项目写入。无新依赖、服务或 unsafe。Linux 实测暂缓，macOS 待有设备；详见[方法、结果与资源边界](../../docs/p0/ASYNC_COMPOSITION.md)。

## 可选静音流重建

```text
python tools/check_p0_recovery.py --fixture artifacts/p0/gpu-fixture --repetitions 3
```

`recovery-proof` 通过纯值 `audio_output::Output` 端口使用 Windows 组装适配器。专用线程实际释放／重开两次自有静音流；故障先发布，清理与开流不在 UI 线程执行。新流代次保持暂停，逻辑位置以故障前冻结点加设备新位置差推进；旧流回执与 held／repeat 不重放。故障中 30 次拖动只提交一个节点，四个 CPU／GPU／显示检查验证重建不改编辑像素。Linux／macOS 后端未实现、未测试；物理拔出、PCM 重定位、声学延迟及原生调用阻塞退出上界未验证。详见[完整记录](../../docs/p0/AUDIO_RECOVERY.md)。

## Pipeline 预热与共享音频文档切换

```text
python tools/check_p0_switch.py --fixture artifacts/p0/gpu-fixture --profile
python tools/check_p0_switch.py --fixture artifacts/p0/gpu-fixture --profile --warmup
python tools/check_p0_switch.py --fixture artifacts/p0/gpu-fixture --warmup
python tools/check_p0_switch_audio.py --fixture artifacts/p0/gpu-fixture --repetitions 3
```

`--profile` 记录每帧候选准备、上传、首帧提交、GPU poll、UI 与上一帧 eframe CPU，用唯一连续帧号关联；只做短诊断。`--warmup` 显式选择一次性后台 pipeline 创建与设备会话内复用，默认不开启；第三项对它运行像素与完整静止／关闭观察。预热和首文档时间保持记录，不当作工作量消失。详见[长帧归因](../../docs/p0/ASYNC_COMPOSITION.md#加载长帧归因与可选后台预热)。

最后一项启用既有可选 `device-audio`，运行 `switch-audio-proof`：一个真实静音流与一个应用级 Transport，文档视图只借用时钟；请求代次、加载失败、编辑历史与音频控制独立。默认构建／无窗口测试不打开设备。三次通过暂停编辑、拖动中切换、恢复、播放中切换与加载中关闭；所有输入仍为合成事件，像素诊断仅在真实暂停下执行。详见[共享音频验证](../../docs/p0/SWITCH_AUDIO.md)。没有新增第三方包版本、unsafe 或产品后台服务。

## 生成 PCM 与内容定位

```text
python tools/check_p0_pcm.py --fixture artifacts/p0/gpu-fixture --repetitions 3
```

此可选入口实际提交低幅度自制 float32 提示音，验证普通暂停恢复、暂停编辑、前后定位及带内容起点的受控重建。源位置／提交前沿和设备播放头分开，暂停不提交 PCM，新流等待显式播放；旧代次和 held／repeat 不重放。纯端口仍为三个方法，恢复和定位共用显式 `{epoch, position_us}` 替换请求。原生写入只在既有可选平台包，主包继续禁止 unsafe；生成音调不引入解码依赖。当前默认 70、平台包 11 项无窗口测试，媒体 feature 另有 1 项文件读取测试；详见[PCM 结果与限制](../../docs/p0/AUDIO_PCM.md)。用户确认本机一次可听且暂停恢复正常，不代表声学延迟、物理输入、谱面或 stable 判定验收。

## Windows 桌面人工验收

```text
python tools/check_p0_desktop.py
python tools/check_p0_desktop.py --interactive
```

先执行默认检查和自有渲染目标回读；第二项随后打开有中文步骤的人工窗口，使用合成时钟、不打开音频设备。只接受本轮自制 PNG 拖放，输入文本不写入证据。原生键鼠／IME／全屏／DPI 事件与人工勾选分开保存；缺少的输入法或跨 DPI 环境保持未验收。A/B 为普通左侧／底部面板，不是最终产品布局。详细步骤及记录边界见 [Windows 桌面验收](../../docs/p0/WINDOWS_DESKTOP.md)。

## 可选短文件与对象时间线

```text
python tools/check_p0_media.py
python tools/check_p0_media.py --run-native --repetitions 3
```

默认关闭的 `media-proof` 引用独立 [p0-media](../p0-media/README.md)，使用 `symphonia 0.6.1` 的 WAV／PCM 功能；原生输出包不依赖解码器。`media-proof` 控制台入口读取自制短 WAV 和受限 v14 fixture，以同一 Transport 验证设备内容位置、暂停和双向定位，不打开图形窗口或实现判定。新增包、feature、资源成本及格式子集见[完整记录](../../docs/p0/MEDIA_TIMELINE.md)。默认 GUI 不编译解码库，新增传递包由锁文件固定。

## 可选后台解码与原生流式输出

```text
python tools/check_p0_stream_output.py
python tools/check_p0_stream_output.py --run-native --repetitions 3
```

复用默认关闭的 `media-proof`，`streaming_decode` 负责单后台线程和不可变源工厂；原生包只接收固定池中的 PCM 值。第一项无窗口检查并构建；第二项顺序播放低幅度自制 WAV，验证连续设备位置、暂停／定位的旧块丢弃、受控欠载与显式恢复。无新依赖或 API feature，不打开图形窗口。默认 GUI 70 项测试，媒体 feature 额外包含 1 项文件读取及 3 项 worker 测试；可选适配器当前 20 项。资源边界和原生结果见[流式输出记录](../../docs/p0/STREAM_OUTPUT.md)。

## 可选压缩音频输出

```text
python tools/check_p0_gui.py --compressed
python tools/check_p0_compressed.py
python tools/check_p0_compressed.py --run-benchmark --run-native --repetitions 2 --fixture artifacts/p0/compressed-fixture-v2
```

前两项只检查／构建；第三项显式顺序执行无窗口读取和低幅度原生播放。`compressed-proof` 组合 `media-proof` 和媒体库的压缩 feature，worker 仍复用同一单线程／五块池，输出包不依赖解码器。默认和原有 WAV feature 不增加解码格式，主包仍禁止 unsafe。当前 WAV GUI 74 项、压缩 GUI 77 项、适配器 23 项、独立压缩媒体 22 项测试；数据和依赖见[首轮记录](../../docs/p0/COMPRESSED_AUDIO.md)，Vorbis 准确定位及私有独立参考见[续篇](../../docs/p0/VORBIS_TAIL_AND_SEEK.md)，分步定位及暂停／替换／关闭取消见[MP3 记录](../../docs/p0/MP3_CANCELLATION.md)。这里只测音频，不代表图形组合、低延迟游玩或 stable 内核。

## 压缩音频与图形长时组合

```text
python tools/check_p0_compressed_soak.py
python tools/check_p0_compressed_soak.py --skip-checks --prepare-audio artifacts/p0/compressed-soak-audio-v1
python tools/check_p0_compressed_soak.py --skip-checks --run-native --repetitions 2 --fixture artifacts/p0/standard-fixture-9f325f13 --audio-fixture artifacts/p0/compressed-soak-audio-v1
```

第一项检查／构建，后两项需已有离线编码器和已验证 PNG；音频准备目录须未存在，已有则直接复用。`combined-soak` 用同一压缩构建顺序执行 WAV／MP3／Ogg，各重复两次，每轮至少 300 秒；原 WAV 工具保留 `media-proof` 构建，JSON 显式记录 feature 避免错配。六轮及 54 项既有预算检查通过，队列、解码 worker 与合成语义不变，无新增包／平台 API／线程。完整参考只由外部证据工具按需读取，不进入原生进程。长帧、CPU／内存口径及未定上限见[组合续篇](../../docs/p0/COMPRESSED_COMBINED_SOAK.md)；仍非正式编辑器、游玩后端或跨平台验收。

## 源格式与默认音频设备变化

```text
python tools/check_p0_audio_adaptation.py
python tools/check_p0_audio_adaptation.py --skip-checks --prepare artifacts/p0/audio-adaptation-fixture-v1
python tools/check_p0_audio_adaptation.py --skip-checks --run-native --fixture artifacts/p0/audio-adaptation-fixture-v1 --repetitions 2
```

准备目录须未存在；使用已有私有离线编码器和本轮自制音频。默认关闭的 `format-device-proof` 要求 `compressed-proof`，无 GUI／图形负载；Windows 组装层显式选择严格匹配或可选共享引擎转换，媒体库、Transport 和 Output 三方法不变。十六轮源格式／策略及一次真实模拟耳机失效后的暂停重建通过，逐包旧代次拒绝和块池归还已核对。人工轮 `--manual` 按阶段标记等待接回确认及显式播放，不修改系统设置；听感、系统引擎成本、转换后采样和其他硬件仍未验收，详见[格式与设备记录](../../docs/p0/AUDIO_ADAPTATION.md)。
