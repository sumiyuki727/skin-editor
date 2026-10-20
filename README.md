# osu!stable 皮肤编辑器（暂定名）

面向 Windows 和 Linux，macOS 待具备设备后验证的桌面皮肤编辑器。当前先推进 Windows，Linux 实测暂缓并保留适配接口。目标是在中央游戏场景中编辑可分层的皮肤素材，通过操作历史树保留不同方案，并为 AI 提供与用户操作一致的结构化控制接口。

**P0 范围与技术决策已形成基线：核心采用 Rust，桌面以 egui／eframe＋wgpu 为集成基线。当前只有设计文档及独立实验，尚无正式应用，P1 尚未开始。**

## 当前状态

| 内容 | 状态 |
| --- | --- |
| 产品目标、需求边界与验收场景 | 已形成文档基线，待开发验证 |
| 模块边界、命令与事务、历史树、预览刷新模型 | 已形成设计，尚未实现 |
| 旧 stable 源码研究 | 已建立只读研究摘要和文件身份索引，未接入或复制源码 |
| 预览时钟、刷新与输入隔离契约 | 独立探针的 10 项测试通过；真实后端未实现 |
| 文件证据和负载数据集工具 | 111 项工具测试通过；包括图形／桌面／音频计量、组合负载、格式与设备变化、独立参考及首批 stable 输入包生成 |
| 独立文件音频实验 | WAV／MP3／Ogg 图形长时组合、Vorbis 定位及 MP3 取消已有证据；Windows 可选共享引擎转换与一次真实耳机失效后的暂停重建已验证。MP3 总定位仍为前缀解码，尚无正式游玩 |
| 原生 GUI / GPU 候选 | 有真实 PNG 负载、1024² 合成、连续编辑及后台文档与共享静音时钟联动原型；独立实验，不是产品工程 |
| 技术栈及资源预算 | 核心与桌面集成基线已选定；已有 Windows 实验测量，新工程预算及其他平台仍待验证 |
| 素材库、图层组、变换和配置编辑 | 未实现 |
| 撤销、重做和历史分叉 | 未实现 |
| 项目保存、皮肤导入与导出 | 未实现 |
| 场景交互、暂停刷新、全屏与实际游玩测试 | 未实现 |
| AI 控制接口、历史寻址与预览截图 | 未实现 |
| 桌面发布包、CI 与跨平台测试 | 未实现 |

内置 stable 兼容预览、真实 stable 客户端验证和实际游玩支持是不同能力。当前主线为精简的独立兼容实现；非开源旧源码仅作研究，不接入产品或实验。对照采用文件、日志与 F12 本地图片，不依赖系统窗口捕获；已有 [b20260924.1／Stable 身份及基线](docs/p0/STABLE_OBSERVATIONS_20261020.md)，并完成 [overlay 存在方式、菜单及普通暂停刷新三个定性子项](docs/p0/STABLE_C03_C08_RESULTS.md)，完整兼容范围尚未验收。详见 [P0 报告](docs/p0/REPORT.md)及[预览边界](docs/REQUIREMENTS.md#预览与实际游玩)。

## 仓库检查

仓库检查使用 Python 3.9+ 标准库及 Git；扫描候选文件或实际暂存字节，报告疑似问题的位置，不输出疑似密钥值：

```text
python tools/check_repository.py
python tools/check_repository.py --staged
python -m unittest discover -s tools -p test_check_repository.py
```

这些检查覆盖常见密钥与私有／生成路径，不是完整安全审计或资源授权证明。`local-data/` 用于本地外部皮肤、头像、运行设置等，构建输出与缓存也被忽略；原版默认素材不纳入 Git 或发布包。仓库内的提交 hook 在暂存检查失败时阻止提交；完整增量按 [AGENTS.md](AGENTS.md#git-与自动保存规则)创建本地 Commit，不自动推送。现有 P0 源码、工具及证据按原字节保存，避免换行转换破坏历史哈希；新工程文本使用 LF。

## 安装依赖

阅读和编辑文档无需应用运行时；仅需文本编辑器，使用版本管理时需 Git。

可选执行 P0 探针时，需要 Python 3.9+、Rust 1.80+（Cargo、rustfmt、clippy）和原生链接工具。Windows 需要已有的 MSVC x64 工具及 Windows SDK；检查脚本会自动发现已有安装，仅对子进程配置链接环境。探针使用标准库，没有第三方 Rust/Python 依赖，无需下载依赖。

文件/日志与 fixture 工具仅需 Python 3.9+ 标准库。原生 GUI 候选另需 Rust 1.95 和已锁定的公开依赖；Windows 的已验证准备流程见 [原生实验说明](experiments/p0-egui/README.md#准备实验依赖)。本机工具链与依赖缓存在 `.p0-tools`，不修改系统已有 Rust，也不纳入产品分发。

核心工具链基线为 Rust 1.95.0／2024 edition；正式工程尚未建立，当前没有产品依赖安装入口或锁文件。工程落地时同步提供实际验证过的安装、启动和测试命令；现有实验工具链及命令仅用于 P0。决策与未完成验收见 [P0 收口评审](docs/p0/P0_EXIT_REVIEW.md)。

## 启动应用

当前没有正式桌面应用。轻量原生实验可自动验证自己的 GPU 渲染并输出 PNG 后退出：

```text
python tools/check_p0_gui.py --run-native
```

它没有真实素材库、图层文档或游玩功能。首个正式桌面工程落地时再提供产品启动及打包命令。

标准 PNG 负载测量使用自有原生测试窗口，不读取 stable 画面：

```text
python tools/check_p0_gui.py --run-standard
```

此入口生成 100 张 1024×1024 原图、缩略图与 10 文档/500 图层清单，按需解码、绘制并记录加载、300 次编辑回读、60 秒暂停及 20 轮资源切换。当前进程/GPU 采样实现仅适用于 Windows；结果和限制见 [P0 报告](docs/p0/REPORT.md)。

比较回读开销与 wgpu 分配策略时执行：

```text
python tools/check_p0_gui.py --run-comparison
```

它在同一数据集上顺序运行三个独立进程：Performance 无回读、MemoryUsage 无回读、MemoryUsage 逐次回读。每组都有 60 秒暂停及关闭资源后的 60 秒观察；无回读组不声称验证了画面或回读延迟。输出保留独立身份及原始样本，实验不修改 stable。

同步/后台加载与上传限额的同素材对照：

```text
python tools/check_p0_gui.py --run-loading-comparison
```

三组均使用 MemoryUsage，分别为同步无回读、后台无回读、后台逐次回读。后台入口包含元数据读取、单线程 PNG 解码、过期结果丢弃和每帧上传限额；它仍是只读 fixture 实验。结果与限制见[后台加载记录](docs/p0/ASYNC_LOADING.md)。

后台加载 CPU 调优与内存归因对照：

```text
python tools/check_p0_gui.py --run-loader-tuning
```

包含三组完整运行和三组短观察诊断，分别比较立即刷新、合并通知及原图上传／绘制路径。仅诊断组扫描自有进程的页面元数据，不读取页面内容；结果和限制见[加载调优记录](docs/p0/LOADER_TUNING.md)。

进一步定位 GPU 上传成本和验证纹理缓存方案：

```text
python tools/p0_workload.py artifacts/p0/gpu-fixture
python tools/check_p0_gpu_memory.py --fixture artifacts/p0/gpu-fixture --diagnostics-only
python tools/check_p0_gpu_memory.py --fixture artifacts/p0/gpu-fixture --compact
```

输出路径须未存在。首组比较新建／复用、仅解码、Vulkan／DX12、GPU 等待及画布回读；第二组补充 80 MiB GPU 缓存和完整性能运行。诊断构建可选启用 `stats_alloc`（MIT，`heap-profile` feature），统计 Rust 请求分配的存活量；默认构建不启用。它不计原生库、驱动或堆保留，不能代替进程提交计量。详见[GPU 上传记录](docs/p0/GPU_UPLOAD.md)。这两组只在 Windows 验证，顺序使用自有原生测试窗口。

已有标准 fixture 的三次重复对照和独立分配诊断：

```text
python tools/p0_gpu_repeats.py --fixture artifacts/p0/gpu-fixture
python tools/p0_vulkan_memory.py --repetitions 3
python tools/p0_vulkan_memory.py --settle-seconds 3
```

重复入口固定二进制与源码，顺序运行九个完整原生进程，比较新建 128、复用 128 和复用 80 MiB 纹理池。分配入口需要 64 位 Windows Python 及已有 Vulkan loader／驱动，没有窗口、绘制或 PNG，也不需新增依赖；分别记录资源申请、页面元数据与释放后观察。隐式层开关仅作用于对应子进程。结果和未归因部分见[分配与重复验证](docs/p0/ALLOCATION_ATTRIBUTION.md)，诊断值不作为应用性能成绩。

完整流程的资源寿命、有界上传与画布对照：

```text
python tools/check_p0_lifetime.py --fixture artifacts/p0/gpu-fixture
```

常规性能、额外时间线与回读分别运行，回执不等于驱动提交页已经回收。详见[资源寿命记录](docs/p0/UPLOAD_LIFETIME.md)。该入口只在 Windows 验证，使用自建原生测试进程。

小型图层合成验证：

```text
python tools/check_p0_composition.py
python tools/check_p0_composition.py --run-native
```

第一项运行无窗口检查与构建；第二项额外执行无窗口 Vulkan 对照。包含不同尺寸 PNG、仿射变换、隔离组透明度与 CPU／GPU 像素比较，不提供实际编辑 UI。直接引用已有锁文件中的 `pollster 1.0.1`，用于 GPU 初始化等待；无新增第三方包版本。语义、平台和规模限制见[合成记录](docs/p0/COMPOSITION.md)。

标准规模合成与可取消拖动验证：

```text
python tools/check_p0_composition.py --fixture artifacts/p0/gpu-fixture
python tools/check_p0_interaction.py --run-native
```

第一项复用已生成的标准 fixture，顺序测量两次 10 文档／500 层合成，记录 GPU 时间戳、回读及进程资源；每文档 50 层、10 张原图，温热派发重复静态计划。第二项通过合成 egui 输入验证草稿、Esc 取消和一次提交，并回读自有渲染器。详见[标准合成](docs/p0/COMPOSITION_SCALE.md)与[拖动记录](docs/p0/INTERACTION.md)。物理鼠标／键盘、IME、可听低延迟音频及完整应用预算仍未验收。

连续编辑、GPU 直接呈现与 Windows 输出流检查：

```text
python tools/check_p0_edit_composition.py --fixture artifacts/p0/gpu-fixture
python tools/p0_audio_device.py
```

前者在标准 fixture 的单个 50 层文档中执行 300 次连续变换，普通呈现和逐次回读各重复两次；普通路径只上传操作参数，不回读和重传整张像素。后者以静音 WASAPI shared 流检查默认输出的时钟、暂停与恢复，仅适用于 64 位 Windows。详见[连续编辑](docs/p0/EDIT_COMPOSITION.md)与[输出流记录](docs/p0/AUDIO_DEVICE.md)，没有可听声音或实际游玩验收。

同进程设备时钟、模拟／输入和暂停编辑的联动验证：

```text
python tools/check_p0_gui.py --device-audio
python tools/check_p0_transport.py --fixture artifacts/p0/gpu-fixture --repetitions 3
```

第一项仅检查与构建，不打开设备；第二项显式使用默认 WASAPI 静音输出与自有 Vulkan 窗口，三次验证冻结时间下的 300 次编辑、取消／单次提交、丢弃暂停输入和恢复无补跑。可选适配器只在 Windows 实测，默认关闭；主实验仍禁止 unsafe，原生调用隔离于独立包。详见[联动记录](docs/p0/TRANSPORT.md)及[适配器依赖与成本](experiments/p0-wasapi/README.md)。初次 PNG 加载仍同步，未接入后台文档切换。

后台原图、隔离组合成和文档切换验证：

```text
python tools/check_p0_switch.py --fixture artifacts/p0/gpu-fixture
```

Windows 入口先做无窗口检查，再顺序运行像素诊断和两次完整观察；另可用 `--smoke` 只做像素诊断。工作线程保留原始 RGBA 和嵌套组，候选按每帧两张／8 MiB 原图写入限额推进，首帧 GPU 完成后才替换旧视图。脚本覆盖过期候选、取消、缺失 fixture、20 次切换和静止／关闭观察，详见[后台合成交接](docs/p0/ASYNC_COMPOSITION.md)。此入口使用占位暂停状态，真实音频组合使用下面的新入口；Linux 和 macOS 未实测。

加载帧归因、可选 pipeline 预热与真实音频切换：

```text
python tools/check_p0_switch.py --fixture artifacts/p0/gpu-fixture --profile
python tools/check_p0_switch.py --fixture artifacts/p0/gpu-fixture --profile --warmup
python tools/check_p0_switch.py --fixture artifacts/p0/gpu-fixture --warmup
python tools/check_p0_switch_audio.py --fixture artifacts/p0/gpu-fixture --repetitions 3
```

前两项各执行两次短帧诊断，第三项检查预热候选的像素与完整观察。预热在一次性 GPU worker 中创建并复用程序，默认关闭，不增加依赖；计入预热及首文档时间，详见[长帧归因](docs/p0/ASYNC_COMPOSITION.md#加载长帧归因与可选后台预热)。最后一项显式打开 Windows 默认静音输出，验证暂停编辑、过期请求、加载失败、恢复、播放中切换及加载中关闭；文档视图借用同一时钟，详见[共享音频与切换](docs/p0/SWITCH_AUDIO.md)。该入口不测试设备重建，没有可听声音、物理输入或实际游玩验收。

自有静音流的受控关闭与重开：

```text
python tools/check_p0_recovery.py --fixture artifacts/p0/gpu-fixture --repetitions 3
```

此 Windows 可选入口验证故障状态下编辑、两次实际流重建、旧代次拒绝、暂停就绪和显式播放，不改系统音频设置。纯音频端口预留其他平台适配，Linux 实测暂缓。物理拔出、默认设备切换和真实 PCM 恢复未验证，详见[流重建记录](docs/p0/AUDIO_RECOVERY.md)。

生成 PCM 的播放、暂停和定位诊断：

```text
python tools/check_p0_pcm.py --fixture artifacts/p0/gpu-fixture --repetitions 3
```

该可选 Windows 入口会短暂播放低幅度自制音调，验证暂停编辑、向前／向后定位及从最后确认内容位置重建。普通入口继续静音；新入口只接受单／双声道 float32 混音格式。本机另有一次用户听感确认，未测声学延迟，详见[PCM 记录](docs/p0/AUDIO_PCM.md)。不录音、不改系统设置，Linux 实测继续暂缓。

Windows 桌面验收入口：

```text
python tools/check_p0_desktop.py
python tools/check_p0_desktop.py --interactive
```

第一项运行默认检查并回读自有原生窗口；第二项随后打开包含键鼠、中文／日文 IME、受限 PNG 拖放、焦点、全屏、缩放和两种面板的人工步骤。使用合成时钟、无音频；只保存计数与确认，不保存输入文本。自动画面检查不代表人工验收，详见[操作步骤与边界](docs/p0/WINDOWS_DESKTOP.md)。

自制 WAV 与最小对象时间线：

```text
python tools/check_p0_media.py
python tools/check_p0_media.py --run-native --repetitions 3
```

第一项检查独立媒体包及默认关闭的 `media-proof` 构建；第二项额外顺序播放低幅度自制 WAV，以设备消费位置验证暂停、双向定位和受限 v14 单击对象时间线。仅 WAV／PCM、短文件及同采样率／声道输出，无窗口、判定或正式游玩；新增依赖、格式子集与内存边界见[文件与时间线记录](docs/p0/MEDIA_TIMELINE.md)。

按需长 WAV 解码与进程资源对照：

```text
python tools/check_p0_streaming.py
python tools/check_p0_streaming.py --run-benchmark --repetitions 3
```

第一项检查并构建独立控制台探针；第二项在 Windows 顺序比较 3 秒／180 秒文件的按需解码与保留整段 PCM，记录进程内存、读取／定位耗时和独立哈希。仅自制规范头 PCM16 WAV，不打开音频设备或 GUI，尚未接入流式播放；详见[有界解码记录](docs/p0/STREAMING_AUDIO.md)。

后台解码与原生流式输出：

```text
python tools/check_p0_stream_output.py
python tools/check_p0_stream_output.py --run-native --repetitions 3
```

第一项检查并构建默认关闭的媒体输出入口；第二项在 Windows 播放低幅度自制 WAV，逐轮验证十秒连续播放、暂停／定位时旧块丢弃、主动阻塞解码引发的欠载、显式恢复及块池回收。不打开 GUI 或录音，格式仍为同设备采样率／声道的规范头 PCM16；详见[流式输出记录](docs/p0/STREAM_OUTPUT.md)。

流式音频与标准 PNG／GPU 组合长时观察：

```text
python tools/check_p0_combined_soak.py
python tools/check_p0_combined_soak.py --run-native --fixture artifacts/p0/standard-fixture-9f325f13 --repetitions 2
```

第二项需要本机已生成的标准 PNG fixture，逐轮播放 300 秒并切换／编辑文档，随后观察暂停和关闭资源后的 CPU、工作集、私有提交及 GPU 内存。使用自有隐藏渲染窗口，不捕获系统窗口；会播放低幅度自制 PCM。短轮可加 `--smoke --repetitions 1`，不评定预算。详见[组合长时记录](docs/p0/COMBINED_SOAK.md)。

反复打开／关闭文档与音频资源：

```text
python tools/check_p0_resource_cycles.py
python tools/check_p0_resource_cycles.py --run-native --fixture artifacts/p0/standard-fixture-9f325f13 --repetitions 2
```

第二项在同一图形会话中逐轮重建并关闭文档、后台解码和原生音频流，检查块池回收、暂停位置及关闭后的内存趋势，最后观察稳定空闲。需要已生成的标准 fixture；短轮可加 `--smoke --repetitions 1`，不评定预算。它没有正式项目保存／加载，详见[资源循环记录](docs/p0/RESOURCE_CYCLES.md)。

可选压缩音频检查、资源测量及原生播放：

```text
python tools/check_p0_compressed.py
python tools/check_p0_compressed.py --run-benchmark --run-native --repetitions 2 --fixture artifacts/p0/compressed-fixture-v2
```

第一项不打开设备。第二项使用本机已生成的自制 MP3／Ogg 与 WAV，顺序测量并播放低幅度音调；无 GUI 或录音。缺少 fixture 时的私有编码工具准备、生成方式、依赖及已知格式限制见[压缩音频记录](docs/p0/COMPRESSED_AUDIO.md)。默认 GUI／WAV feature 不启用压缩格式，FFmpeg 不作为运行依赖。

MP3 分步定位、暂停／替换／关闭期间的取消回归及作用范围见[取消交接记录](docs/p0/MP3_CANCELLATION.md)。离线探针同时记录定位步数及最长一步耗时；该诊断不代表阻塞 I/O 或取消的实时期限。

同源 WAV／MP3／Ogg 与标准图形的长时组合：

```text
python tools/check_p0_compressed_soak.py
python tools/check_p0_compressed_soak.py --skip-checks --prepare-audio artifacts/p0/compressed-soak-audio-v1
python tools/check_p0_compressed_soak.py --skip-checks --run-native --repetitions 2 --fixture artifacts/p0/standard-fixture-9f325f13 --audio-fixture artifacts/p0/compressed-soak-audio-v1
```

第一项仅检查并构建压缩 feature；准备步骤需要已有私有离线编码工具，输出须未存在。已有目录直接复用。最后一项在 Windows 使用同一二进制顺序执行六轮，每轮连续消费至少 300 秒并编辑／切换图形文档，随后检查暂停与关闭空闲。需要已生成的标准 PNG fixture，会播放低幅度自制音调；短轮可加 `--smoke --repetitions 1`，不评定预算。完整参考占 GiB 级磁盘，但不加载到被测原生进程；资源口径、长帧及实际通过范围见[组合记录](docs/p0/COMPRESSED_COMBINED_SOAK.md)。

源格式不匹配与音频设备变化：

```text
python tools/check_p0_audio_adaptation.py
python tools/check_p0_audio_adaptation.py --skip-checks --prepare artifacts/p0/audio-adaptation-fixture-v1
python tools/check_p0_audio_adaptation.py --skip-checks --run-native --fixture artifacts/p0/audio-adaptation-fixture-v1 --repetitions 2
```

第一项只检查／构建；准备需要已有私有离线编码器，目录须未存在。原生入口在 Windows 对照严格匹配与可选共享引擎转换，覆盖不同源采样率／声道、暂停旧块拒绝、定位、受控失效及暂停重建；使用自制低幅度音频。`--smoke` 仅做单个短轮，`--manual` 另按阶段标记做人工设备观察。已记录一次真实模拟耳机失效与恢复，但听感、转换后采样和系统音频服务总成本未确认；步骤及范围见[格式与设备记录](docs/p0/AUDIO_ADAPTATION.md)。不改系统设备、格式或音量设置。

独立 Ogg 首尾与分页控制审计：

```text
python tools/p0_vorbis_tail.py --fixture artifacts/p0/compressed-fixture-v2 --output artifacts/p0/vorbis-tail-audit-v1
```

输出须未存在，重复时更换目录。先运行压缩检查以构建对应探针；Windows x64 私有 libsndfile 工具的准备、24 个边界／分页对照及准确定位限制见[续篇](docs/p0/VORBIS_TAIL_AND_SEEK.md)。审计不播放音频，不作性能测量，也不需要客户端。便携核心测试无需 FFmpeg、libsndfile 或 Python 第三方包。

首批 stable 输入包准备：

```text
python tools/p0_stable_batch.py --output artifacts/p0/stable-first-batch-v1 --client-dir "D:\osu!"
```

输出须未存在；已生成时直接使用现有包，重复生成应更换目录。只在仓库生成自制皮肤／谱面包和操作卡，客户端目录可省略、提供时只作受限读取。安装与 F12 对照由用户按[首批步骤](docs/p0/STABLE_FIRST_BATCH.md)执行；当前已有身份、基线和 C03／C08 首轮定性结果，详见对应结果记录。

## 运行测试

在仓库根执行：

```text
python tools/check_p0.py
python -m unittest discover -s tools/tests -v
```

第一项检查实验隔离、Rust 格式、Clippy 和 10 个契约测试，并执行控制台场景，详见 [探针说明](experiments/p0-preview-contract/README.md)；第二项运行 111 个文件／图形／音频／桌面证据工具测试，不打开音频。原生默认入口另有 70 项无窗口测试；音频 feature 检查适配器 25 项；WAV 媒体 feature 的 GUI 共 74 项，压缩 feature 再增加 3 项取消竞争回归，共 77 项。独立媒体包默认 12 项，可选压缩 feature 为 22 项。Windows 本轮合计 245 项，不重复计数。检查包含依赖边界、格式、Clippy、release 构建与输出保护。原生 GPU 及音频设备验证另行显式执行，Linux 实测暂缓，macOS 待有设备再验收。

目前没有正式应用、正式音频后端或已完成的当前 stable 兼容测试。静音流已与实验模拟／输入和合成联动，尚无实际游玩；标准 PNG 负载也未包含正式命令/历史或完整产品能力，不能代表最终产品预算。后续新增基础行为仍须补充对应测试。

文档变更可用 `git diff --check` 检查补丁空白问题；它不是应用测试，也不检查尚未跟踪的新文件内容。

## 文档导航

- [AGENTS.md](AGENTS.md)：长期项目约束、边界与修改规范。
- [需求与验收](docs/REQUIREMENTS.md)：产品目标、用例、阶段范围、待确认事项。
- [架构](docs/ARCHITECTURE.md)：模块职责、依赖方向、核心模型、数据流和当前决策。
- [开发路线](docs/ROADMAP.md)：阶段产物、进入条件、验收标准与测试计划。
- [技术栈评估](docs/TECH_STACK.md)：候选方案、验证方法、性能预算与选型条件。
- [P0 研究报告](docs/p0/REPORT.md)：旧源码观察、独立实验、环境和未完成验证。
- [当前 stable 验证矩阵](docs/p0/VALIDATION_MATRIX.md)：公开文档、旧观察、当前实测和自有回归的独立记录。
- [F12 与文件证据流程](docs/p0/EVIDENCE_WORKFLOW.md)：自制输入包、只读检查和明确图片的登记方式。
- [精简预览器范围](docs/p0/PREVIEW_SCOPE.md)：保留、简化和省去的原版功能。

推荐先阅读需求与架构，再依据路线图推进。后续实现需要同时更新功能状态和实际运行说明；本仓库不会自动提交或推送变更。
