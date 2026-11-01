# osu!stable 皮肤编辑器（暂定名）

面向 Windows 和 Linux，macOS 待具备设备后验证的桌面皮肤编辑器。当前先推进 Windows，Linux 实测暂缓并保留适配接口。目标是在中央游戏场景中编辑可分层的皮肤素材，通过操作历史树保留不同方案，并为 AI 提供与用户操作一致的结构化控制接口。

**已有独立 Rust 核心、统一命令、分叉历史及可取消草稿。P2 内存桌面编辑器已接入 PNG 素材库、图层／组件变换和占位 GPU 预览；尚无项目保存、导出或真实游玩。**

## 当前状态

| 内容 | 状态 |
| --- | --- |
| 产品目标、需求边界与验收场景 | 已形成文档基线，待开发验证 |
| 模块边界、命令与事务、历史树、预览刷新模型 | 单向依赖、原子 Command、冲突／去重和分叉历史控制，已接入只读占位预览 |
| 正式核心工程 | Rust 1.95／2024 edition，模型 v2、application v6／预览 v1 值契约；无第三方核心依赖 |
| 旧 stable 源码研究 | 已建立只读研究摘要和文件身份索引，未接入或复制源码 |
| 预览时钟、刷新与输入隔离契约 | 独立探针的 10 项测试通过；真实后端未实现 |
| 文件证据和负载数据集工具 | 111 项工具测试通过；包括图形／桌面／音频计量、组合负载、格式与设备变化、独立参考及首批 stable 输入包生成 |
| 独立文件音频实验 | WAV／MP3／Ogg 图形长时组合、Vorbis 定位及 MP3 取消已有证据；Windows 可选共享引擎转换与一次真实耳机失效后的暂停重建已验证。MP3 总定位仍为前缀解码，尚无正式游玩 |
| 原生 GUI / GPU 候选 | 有真实 PNG 负载、1024² 合成、连续编辑及后台文档与共享静音时钟联动原型；独立实验，不是产品工程 |
| 技术栈及资源预算 | 核心与桌面集成基线已选定；已有 Windows 实验测量，新工程预算及其他平台仍待验证 |
| 素材库、嵌套图层组／跨元素组件、变换和配置 | PNG 导入与缩略图、图层／组件 UI 和 CPU／GPU 合成已接入；配置保留原始字节，尚无解析器或配置 UI |
| 撤销、重做和历史分叉 | 已有核心、会话内短地址、分页及简单历史面板；API v6普通Redo记住Undo前的直接分支；10000层／10000同级分支检查通过；六类系统标签UI与文件保存回执尚未接入，无历史持久化 |
| 可取消草稿 | 已接入画布／参数连续拖动及 Esc、失焦和视口变化取消 |
| 项目保存、皮肤导入与导出 | 未实现 |
| 音频素材库、音效编辑与实时替换 | 已接WAV／原始保留导入、独立音效绑定、组件／原始素材显式试听和监听总音量；组件试听可实时响应增益，Esc恢复本次增益，重绑定撤销旧试听；共享预算及独立供给。实际歌曲流及文件导出未实现 |
| 场景交互、暂停刷新、全屏与实际游玩测试 | 占位场景、暂停刷新和全屏入口已接入；真实 stable 玩法／音频未实现 |
| 宿主重绘期限合并 | 已接入同pass最早绝对期限、命令后新工作及跨pass重置；原生消息减少，CPU收益尚未证实 |
| AI 控制接口、历史寻址与预览截图 | 可共用的 Rust 值接口、历史寻址和自有 GPU 诊断回读；AI 传输未实现 |
| 桌面发布包、CI 与跨平台测试 | Windows 核心检查通过；Windows／Linux CI 配置已建立但未远程运行，Linux／macOS 实测及发布包未完成 |

内置 stable 兼容预览、真实 stable 客户端验证和实际游玩支持是不同能力。当前主线为精简的独立兼容实现；非开源旧源码仅作研究，不接入产品或实验。对照采用文件、日志与 F12 本地图片，不依赖系统窗口捕获；已有 [b20260924.1／Stable 身份及基线](docs/p0/STABLE_OBSERVATIONS_20261020.md)，并完成 [overlay 存在方式、菜单及普通暂停刷新三个定性子项](docs/p0/STABLE_C03_C08_RESULTS.md)，完整兼容范围尚未验收。详见 [P0 报告](docs/p0/REPORT.md)及[预览边界](docs/REQUIREMENTS.md#预览与实际游玩)。

## 仓库检查

仓库检查使用 Python 3.9+ 标准库及 Git；扫描候选文件或实际暂存字节，报告疑似问题的位置，不输出疑似密钥值：

```text
python tools/check_repository.py
python tools/check_repository.py --staged
python -m unittest discover -s tools -p test_check_repository.py
```

这些检查覆盖常见密钥与私有／生成路径，不是完整安全审计或资源授权证明。`local-data/` 用于本地外部皮肤、头像、运行设置等，构建输出与缓存也被忽略；原版默认素材不纳入 Git 或发布包。提交 hook 运行暂存扫描、核心与桌面检查；按 [AGENTS.md](AGENTS.md#git-与自动保存规则)保留未提交变更，不自动提交或推送。现有 P0 源码、工具及证据按原字节保存；新工程文本使用 LF。

## 安装依赖

阅读和编辑文档无需应用运行时；仅需文本编辑器，使用版本管理时需 Git。

可选执行 P0 探针时，需要 Python 3.9+、Rust 1.80+（Cargo、rustfmt、clippy）和原生链接工具。Windows 需要已有的 MSVC x64 工具及 Windows SDK；检查脚本会自动发现已有安装，仅对子进程配置链接环境。探针使用标准库，没有第三方 Rust/Python 依赖，无需下载依赖。

Rust 1.80 是旧契约探针自身的下限；现在从仓库根运行时，rustup 会读取根工具链锁并选择 1.95.0。运行 P0 命令也应先准备好该工具链，不能把当前系统默认 1.80.1 当作已满足整个仓库的要求。

文件/日志与 fixture 工具仅需 Python 3.9+ 标准库。原生 GUI 候选另需 Rust 1.95 和已锁定的公开依赖；Windows 的已验证准备流程见 [原生实验说明](experiments/p0-egui/README.md#准备实验依赖)。本机工具链与依赖缓存在 `.p0-tools`，不修改系统已有 Rust，也不纳入产品分发。

正式工程需要 Rust 1.95.0（含 rustfmt、clippy）、Python 3.9+ 及本机链接工具；Windows 需要 MSVC x64 和 Windows SDK。根目录锁定两个无第三方依赖的核心包；桌面图形依赖单独锁定在 `desktop/Cargo.lock`。检查脚本不会安装工具或修改系统默认版本。现有 P0 实验命令与产品工程分开。

## 正式核心检查

已通过 rustup 注册 1.95.0 时，在仓库根目录执行：

```text
python tools/check_core.py
```

它离线运行 Cargo 依赖／源码边界、仓库与架构工具测试、格式、Clippy、核心测试（含私有访问编译拒绝）和 release 库构建。只有两个 library 包，无应用启动命令。

已有独立 1.95.0 工具链时，可用 `--toolchain-dir` 指定其 `bin` 目录，或设置本机环境变量 `SKIN_EDITOR_RUST_TOOLCHAIN_DIR`；本机已按此方式完整验证。脚本不依赖 `.p0-tools` 或实验包的路径，缺少工具时明确失败，不自动下载。提交 hook 使用同一检查入口，并继承该环境变量。只检查架构可加 `--architecture-only`。

本机 Windows 核心及桌面默认检查已通过，覆盖格式、Clippy、无窗口行为／文档测试和依赖边界；最新分项与显式原生结果见[P2记录](docs/P2_PROGRESS.md)及[音效UI记录](docs/P2_AUDIO_UI.md)。远程 CI 尚未执行，Linux 构建／实测未通过本机验证。CI 固定 [checkout 提交](https://github.com/actions/checkout/commit/9c091bb21b7c1c1d1991bb908d89e4e9dddfe3e0)，只读权限且不保留凭据，不自动发布或推送。

基础使用见 [命令契约](docs/COMMANDS.md)和 application 的可运行文档示例。创建会话需要项目 ID 与独立历史会话 ID；默认会话没有资源存储，登记原始资源需要组装 ResourceStore。该核心示例使用内存适配，不读取实际 PNG、谱面或用户目录；桌面已有真实 PNG／WAV 导入，见下文。

Windows 可选运行独立核心成本诊断（相同依赖，输出目录必须尚不存在）：

```text
python tools/check_core_cost.py --output artifacts/p1/core-cost-new-run
```

它离线检查并构建独立 release 工具，默认三种元数据负载各重复两次，记录 CPU／工作集／私有提交及 10000 次事务、跨分支恢复和草稿取消。详情与测量边界见[基线](docs/CORE_COST_BASELINE.md)；不打开窗口或设备，不代表实际图片／GPU 的整体预算。

## 启动应用

现有内存桌面编辑器。首次准备公开依赖需要网络，Windows 已验证的获取命令为：

```text
cargo fetch --manifest-path desktop/Cargo.toml --locked --target x86_64-pc-windows-msvc
```

随后离线检查与启动：

```text
python tools/check_core.py
python tools/check_desktop.py
python tools/run_desktop.py
```

启动时展示自制圆环。展开左侧“素材库”，投放 PNG 或填写文件路径导入，再点“加入当前元素”；选中图层／组后画布拖动平移、拖角点缩放、拖圆形柄旋转，右侧参数也可调整变换和透明度。拖动中 Esc 取消，松开一次提交；历史按钮可以回到旧节点继续分叉。**当前只保存于内存，关闭后丢失；没有保存、导出、真实谱面或游玩。** 嵌套图层组与跨元素组件组可整体变换，连续手势仍由 application 草稿管理。

桌面自有 Vulkan／egui 像素回读（输出目录须未存在）：

```text
python tools/check_desktop.py --run-native --output artifacts/p2/native-proof
```

正常呈现不做 CPU 回读。功能边界和人工验收见 [P2 记录](docs/P2_PROGRESS.md)。Linux／macOS 未测；当前 debug 构建不作为性能预算成绩。启动和检查均支持 `--toolchain-dir` 或 `SKIN_EDITOR_RUST_TOOLCHAIN_DIR` 指定现有 Rust 1.95.0 bin 目录。

点击顶部“音效编辑”可打开独立窗口。添加自制WAV示例或输入WAV路径导入，显式启用音频设备后，可直接“试听原始素材”；也可填写唯一用途键创建音效组件，再“显式试听音效”。增益拖动松开一次提交，Esc取消；冻结场景时仍可试听。监听总音量不影响项目／导出，1000为原音量、0为静音。其他编码可显式选择原始保留，不支持解码时显示错误并保留文件字节。详见[素材试听与操作](docs/P2_AUDIO_LIBRARY.md)。

显式短WAV原生诊断会播放低音量自制信号，不打开窗口或读取私人媒体。输出目录须未存在；默认检查／正常启动不打开音频设备：

```text
python tools/check_desktop.py --run-audio --output artifacts/p2/native-audio-proof
```

支持RIFF/WAVE PCM16、44.1／48 kHz、单／双声道，监听转换与原始文件分离；本机输出使用默认设备。详见[音频接入记录](docs/P2_AUDIO_NATIVE.md)。桌面新增CPAL（Apache-2.0）及rtrb（MIT／Apache-2.0），仅适配层使用；Linux需系统ALSA开发库／pkg-config，Linux和macOS均未实测。

独立供给／背景参考诊断同样会发出低音量自制信号，在控制端停顿期间持续供给，并检查热替换、暂停／试听和清理。背景是共享预算内的有限预解码参考，未接实际歌曲或游戏时钟：

```text
python tools/check_desktop.py --run-audio-supply --output artifacts/p2/audio-supply-proof
```

验证范围见[独立供给记录](docs/P2_AUDIO_SUPPLY.md)；回调提交位置不表示扬声器已经播放到该处。

Windows已就绪触发延迟／32 Voice原生小负载检查，使用release二进制执行两轮，只采样自己启动的音频进程CPU／内存。会播放低音量自制信号，不打开GUI／GPU：

```text
python tools/check_desktop.py --run-audio-load --output artifacts/p2/audio-load-proof
```

定义和限制见[原生提交与32 Voice记录](docs/P2_AUDIO_LOAD.md)；不代表完整编辑器预算或声学延迟。

Windows的300次增益热修改与短WAV暖准备计量，同样显式运行两轮release原生音频，输出目录须尚不存在：

```text
python tools/check_desktop.py --run-audio-edit --output artifacts/p2/audio-edit-proof
```

延迟、资源释放与CPU／内存结果见[热修改记录](docs/P2_AUDIO_EDIT.md)。该轮发现的初始静音段192帧缺口已完成[受控复现与当前Windows设备修复验证](docs/P2_AUDIO_STARTUP.md)，失败数据仍保留；完整A2仍有未验收范围。

显式重复静音输出启动与释放，两轮各20次，不打开窗口或GPU：

```text
python tools/check_desktop.py --run-audio-startup --output artifacts/p2/audio-startup-proof
```

Windows标准静态画布＋原生声音组合，两轮各300秒，显式打开自有窗口并播放低幅度自制音频；新目录内生成100个PNG测试输入，不读取玩家媒体：

```text
python tools/check_desktop.py --run-combined --output artifacts/p2/combined-proof
```

首轮组合成绩见[组合记录](docs/P2_COMBINED.md)。后续已接单候选后台准备和原图buffer容量复用，见[候选准备记录](docs/P2_FRAME_PREPARATION.md)；逐Voice提交观察区间、GPU分配器／进程计量和显式短轮控制见[归因记录](docs/P2_RESOURCE_ATTRIBUTION.md)。稳定空闲、完整资源循环及设备等门槛仍开放，不代表完整P2或真实玩法通过。

显式 MemoryUsage 标准负载和跨配置逐字节像素对照的入口见[验证记录](docs/P2_MEMORY_USAGE.md)。普通 Windows 入口已接入 MemoryUsage 与分离声音工厂，验证状态见[自动收尾](docs/P2_AUTOMATIC_CLOSEOUT.md)；其他平台仍待实测。

同一release构建的14秒冷准备交错控制（Performance／MemoryUsage／MemoryUsage／Performance，顺序打开自有窗口和原生输出；用于归因）：

```text
python tools/measure_combined.py --binary desktop/target/release/skin-editor-desktop.exe --output artifacts/p2/new-cold-controls --cold-controls
```

新增的有界提交／worker／上传分段及两轮完整组合结果见[分段验证](docs/P2_SUBMISSION_STAGES.md)。保持原声音供给和20 ms上界p95门槛；短控制不替代标准负载，历史诊断不用于追认后续默认配置。

显式[有界暂存候选](docs/P2_STAGING_REUSE.md)支持 `measure_combined.py --staging-controls` 交错对照，以及在 `--memory-usage` 或 `--pixel-controls` 下附加 `--reusable-staging`。均须同时传入已构建的 `--binary` 和新的 `--output` 目录。候选已通过两轮标准组合、预算／像素／逻辑清理，普通应用上传默认保持原路径。

后续[寿命验证](docs/P2_STAGING_RECOVERY.md)入口为 `python tools/measure_staging_recovery.py --binary desktop/target/release/skin-editor-desktop.exe --output artifacts/p2/new-staging-recovery`，显式打开自有窗口／原生音频，每轮约11分钟、共两轮，包含标准300秒、稳定空闲及20次资源重建。360项自动检查及资源／预算／像素子项通过；第二轮场景提交p95上界20.262 ms超过原目标，整体仍未通过。普通完整UI空闲及小幅驻留保留仍待验证／归因，默认与声音供给未改。

[精确Voice／相位诊断](docs/P2_VOICE_PHASE.md)入口为 `python tools/measure_voice_phase.py --binary desktop/target/release/skin-editor-desktop.exe --output artifacts/p2/new-voice-phase`，显式使用当前默认音频输出播放低音量自制信号，不打开GUI／GPU。两轮各160次／通道扫描，使用新输出目录；固定容量记录完整Voice身份、完成区间与实到相位。372项默认检查通过；标准组合预算／像素／清理通过，但一次20 ms提交夹界及独立扫描416帧欠载仍未解决。原生延迟和资源数字只适用于当前环境，迁移须单独复验；Linux／macOS未测。

后续[欠载现场诊断](docs/P2_UNDERRUN_CONTEXT.md)沿用该入口，报告schema 2增加32次有界上下文及关闭输出流后的阶段总计。378项默认检查通过；真实扫描捕获一次1056帧请求耗尽832帧队列的224帧欠载及同回调超过110 ms的Voice提交。旧416帧路径仍未知，供给策略与20 ms门槛保持，不能据此宣布完整P2通过。

最新[供给／回调／宿主关联](docs/P2_SCHEDULING_CORRELATION.md)沿用相位与组合入口，新增有界批次完成及宿主调用记录，387项默认检查通过。新扫描将长等待定位到供给休眠前至再次运行；标准首轮300秒预算／20 ms／像素／清理通过但缺96帧，第二轮GPU初始化失败，另列补测宿主迟到失败。原102.9／420.5 ms事件不能事后定因，20 ms夹界及系统／框架内部等待仍开放；声音策略、门槛和默认配置不变，P2-A3整体未通过。

后续[迟到供给恢复候选](docs/P2_SUPPLY_RECOVERY.md)通过401项默认检查，新增`measure_supply_recovery.py`固定四进程音频对照入口。具体192／288帧交错已确定性修复；小范围原生正常路径零欠载、提交p95≤20 ms及CPU／内存通过，但两轮候选均无自然恢复，恢复分支原生覆盖仍待验收。普通默认保持原策略，完整组合及P2-A3未关闭；首次诊断读取失败和修正后结果分别保留。

最新[受控恢复验证](docs/P2_CONTROLLED_RECOVERY.md)提供`measure_controlled_recovery.py`，在实际供给所有者及原生输出中分别执行恢复与触发同轮／补充后触发。409项默认检查通过，两种模式各320次真实PCM提交；后者提交p95约29.5–29.7 ms超过原门槛，候选不晋升默认，P2-A3仍未通过。另保留旧策略的新416帧回调突发现场；原生小轮不代表完整图形预算或其他设备通过。

显式检查实际音效界面、自有GPU回读和冻结下的原生试听（输出低音量自制音频）：

```text
python tools/check_desktop.py --run-audio-ui --output artifacts/p2/audio-ui-proof
```

只做三秒原生窗口启动／关闭检查可运行 `python tools/run_desktop.py --smoke-window`，不作人工输入验收。

显式试用分离音频候选可运行 `python tools/run_desktop.py --candidate-audio`（同样可带 `--toolchain-dir`）。在“音效编辑”中启用设备后，可查看准备／错误、监听音量、回调／欠载与背景状态。当前普通候选尚未载入背景歌曲，可试听音效；该入口不改变默认输出，也不代表 P2-A3 已验收。验证范围见[候选 GUI 记录](docs/P2_GUI_CANDIDATE.md)。有界持续背景目前仅接入自制 WAV 诊断，实际歌曲选择尚未接入；32 Voice 的 CPU 仍有超限，包括后续普通观察轮次，见[CPU 记录](docs/P2_VOICE_CPU.md)及[线程采样对照](docs/P2_THREAD_SAMPLING.md)。

Windows 线程观察成本诊断可使用已有 release 二进制运行 `python tools/measure_thread_sampling.py --output artifacts/p2/thread-sampling-new`（输出目录须尚不存在）。它按固定顺序打开12个自有GUI进程，播放低音量自制信号，比较普通／逐次枚举／有界句柄复用；不做系统窗口捕获。预算失败返回非零并保留全部录制，不能用它替代两轮300秒标准组合验收。

当前性能状态按[预算v2](docs/RESOURCE_BUDGET.md)分别记录CPU优化目标与硬性检查；[默认发布短测](docs/P2_BUDGET_REVIEW.md)零欠载、20 ms及内存／音频清理通过，单核15%目标未达，P2整体验收仍开放。旧脚本的`passed`保留原判据。

当前分离声音候选的[完整桌面稳定段](docs/P2_STABLE_WINDOW.md)、[标准图形／持续背景两轮300秒](docs/P2_CURRENT_COMBINED.md)及[20次完整资源循环](docs/P2_RESOURCE_CYCLES.md)已按各自Windows构建范围通过。循环验证零欠载、提交p95≤20 ms、任务join及最后资源租约归还；私有提交关闭基线未持续增长，仍有未归因的进程保留。CPU≤15%优化目标未验收，内存／GPU余量有限；普通声音默认尚未晋升，故障和人工交互等P2验收仍开放，P3暂缓。

[C4窗口故障与初始化失败验证](docs/P2_INIT_RECOVERY.md)提供显式`tools/measure_present_fault.py --initialization`三用例，要求普通权限冻结release、构建回执和新artifact目录。实际消费交接并构建Editor后，在不同初始化位置失败，验证原会话／分叉历史保留、资源退休后重试、声音暂停和背景位置恢复；零欠载、提交p95最高13.4872 ms，内存／GPU通过。普通启动尚无自动恢复，真实驱动／清理失败、物理音频与C5等仍开放，默认策略未晋升。

[受阻清理与恢复验证](docs/P2_CLEANUP_RECOVERY.md)增加显式`tools/measure_present_fault.py --cleanup`，使用同样的冻结release／回执／新artifact目录参数。真实旧绘制仍持有GPU目标时，保留项目、历史和清理上下文，拒绝新后端；实际释放并退休旧Instance后才允许恢复。它不是普通启动的自动恢复，也不证明真实驱动或音频端点清理错误可恢复。

[音频端点归还验证](docs/P2_ENDPOINT_RETURN.md)提供`tools/measure_present_fault.py --endpoint-return`，沿用冻结release／构建回执／新artifact目录参数。实际CPAL流停止后由单槽门闩保留返回端点，验证待归还时保留项目与资源、拒绝新工作，实际finish及旧框架退休后恢复。通道断开与错误端点另有无设备回归；普通启动默认、真实设备故障及永久失败恢复范围不因此扩展。

[活动编辑故障验证](docs/P2_ACTIVE_EDIT_FAULTS.md)提供`tools/measure_present_fault.py --active-edit`，沿用冻结release／构建回执／新artifact目录参数。真实egui拖动与视觉／音频草稿遇到图形、音频或组合失效时，验证取消、原松开拒绝、历史保留与资源清理；另有数值响应、外部草稿及迟到任务无窗口回归。普通启动默认、物理输入与真实设备异常范围不因此扩展。

本机P2-A3周期负载的CPU计量已有[可靠性限制](docs/P2_CPU_ACCOUNTING.md)：相关脚本的原数值`passed`不能单独证明CPU≤15%。可运行 `python tools/measure_cpu_accounting.py --output artifacts/p2/cpu-accounting-new` 做固定两进程参考校验（输出目录须不存在；此诊断的高分辨率定时器需Windows 10 1803及以上）；无GUI／音频，包含两段各约2.1秒的一核连续对照。它不更改系统时钟或产品策略，完成参考采集不等于产品性能通过。

新增[独立调度计量](docs/P2_INDEPENDENT_CPU.md)：Windows上使用现有x64 MSVC／SDK和性能跟踪权限，显式运行 `python tools/measure_independent_cpu.py --run-product --output artifacts/p2/independent-cpu-new`。先校验正／反序参考，全部通过后才运行两轮原生GUI／音频短测；输出目录须不存在。`--build-only`只编译诊断，不启动ETW／负载。无系统ETL或窗口捕获；权限缺失明确失败，不自动提升或修改系统权限。当前参考已通过，但产品CPU超限，P2-A3仍未验收；产品日常启动不需要该权限或C诊断。

同次[调度／线程身份诊断](docs/P2_CPU_THREAD_IDENTITY.md)入口为 `python tools/measure_cpu_threads.py --elevated-only --output artifacts/p2/cpu-thread-new`，需要上述Windows工具和已提升的性能跟踪权限，输出目录须不存在。先复验参考，再固定运行基础／阶段身份／阶段身份／基础四轮GUI音频；不做窗口捕获。默认混合计划中的关联普通令牌路线在本机被系统拒绝，尚不能作为普通权限验收入口；提升诊断仍显示CPU超限。

新的[普通权限／main边界诊断](docs/P2_MAIN_WORK.md)入口为 `python tools/measure_main_work.py --output artifacts/p2/main-work-new`，须在普通Windows桌面运行，使用现有x64 MSVC／SDK、已有release二进制及不存在的输出目录。它通过UAC只提升自有ETW采集器，由普通runner创建限时参考和四轮GUI音频进程，对照边界记录关／开／开／关；不做窗口捕获。参考负载漏周期会阻止后续准入，完整计量在会话停止排空后核对；失败返回非零并保留记录，诊断值不能当作CPU验收通过。该入口已运行，但本轮参考未全部通过，P2-A3仍未验收。日常应用不需要提升权限或C诊断。

进一步运行[严格参考／重绘来源诊断](docs/P2_REPAINT_ORIGIN.md)可使用 `python tools/measure_repaint.py --output artifacts/p2/repaint-new`，条件同上，runner须未提升。它分别为参考和产品采集器触发UAC；参考会话完整停止排空且两轮严格通过后，才创建四个产品进程。可选观察保存重绘延迟、调用位置和输入种类计数，不记录输入值或做窗口捕获。当前严格参考已通过，CPU仍超限，203次／秒尚未复现；失败返回非零，全部记录保留，P2-A3未验收。

该入口现支持[宿主分段格式3](docs/P2_UI_WALL.md)，进一步区分面板与四处主要快照获取，读取工具兼容格式1／2。观察记录仅在显式诊断中启用；没有修改正常声音策略或刷新间隔，CPU预算仍未通过。

恢复UAC后已用该入口完成[同构建面板CPU验证](docs/P2_UI_CPU.md)，参考校验有效，CPU仍超限。无需为这次验证重建产品或引入新依赖；P2-A3保持未验收。

框架／呈现／完整UI对照可运行 `python tools/measure_framework_baseline.py --output artifacts/p2/framework-new`。需要已有当前release、普通Windows runner及上述MSVC／SDK；两个采集器分别请求UAC，严格参考通过后固定运行六轮。诊断窗口不激活并启用鼠标穿透，三种模式保留相同声音脚本；普通编辑器窗口不受影响。结果用于归因，不能操作这些窗口或用最小模式代替产品验收；完整UI门槛失败时非零退出，全部记录保留。范围与已选后续候选见[基线记录](docs/P2_FRAMEWORK_BASELINE.md)。

不方便授权UAC时，可使用 `python tools/measure_ui_wall.py --output artifacts/p2/ui-wall-new`。它要求已有当前release构建和普通Windows令牌，固定运行两轮自制信号GUI诊断，不启动采集器或请求提升权限。成功退出只代表墙钟记录、原生提交／生命周期检查完成，CPU预算明确保持未验证；输出目录须不存在。详见[普通权限记录](docs/P2_UI_WALL.md)。

细分控件并统计文字复用可运行 `python tools/measure_text_work.py --output artifacts/p2/text-work-new`，同样要求已有当前release、普通Windows runner和全新输出目录。固定两轮只读输出统计保留原声音脚本，不请求UAC；成功退出只表示墙钟／文字统计和非CPU检查完成，CPU预算明确未知。边界读取支持格式1–4；字体失效与交互范围见[文字记录](docs/P2_TEXT_WORK.md)，当前未采用外置跨帧网格缓存。

同一新鲜输出的文字／其他网格回放入口为 `python tools/measure_mesh_replay.py --output artifacts/p2/mesh-replay-new`，运行条件同上，不请求UAC。固定两轮分别记录纯／框架tessellation、形状克隆和CPU复制，原输出继续正常绘制；成功只表示精确核对及非CPU检查完成，不代表GPU上传或CPU预算通过。范围和已作选择见[网格回放记录](docs/P2_MESH_REPLAY.md)。

真实帧与独立上传诊断入口为 `python tools/measure_gpu_path.py --output artifacts/p2/gpu-path-new`，同样需已有发布构建、实际Windows桌面和普通权限，不请求UAC。固定两轮帧标记、两轮独立缓冲上传；包含原生声音，普通产品默认不启用。成功仅表示有界诊断、声音／内存／清理检查完成，不代表正常present、GPU执行时长或CPU预算通过；边界见[真实帧／上传记录](docs/P2_GPU_PATH.md)。

官方框架区段诊断使用 `framework-trace` feature，普通构建默认关闭。可用 `cargo build --manifest-path desktop/Cargo.toml --offline --locked --release -p skin-editor-desktop --features framework-trace` 构建；将该二进制固定到仓库内新的 `artifacts` 子目录，再重建默认发布构建。工具入口为 `python tools/measure_frame_dispatch.py --output artifacts/p2/frame-dispatch-new --trace-binary <已固定的诊断二进制路径>`，要求实际Windows桌面、普通runner及两次采集器UAC；严格参考通过后才运行固定四轮。它包含原生声音，记录真实提交／呈现、输入种类和多次布局，不保存输入内容；当前CPU仍未通过，范围及旧辅助过滤限制见[区段记录](docs/P2_FRAME_DISPATCH.md)。默认桌面检查也会检查诊断feature，运行时普通产品不会自动安装观察器。

同一工具添加 `--no-uac` 可执行固定两轮普通权限墙钟／声音／内存诊断，不构建或启动采集器、不请求UAC；需要上述固定feature构建，当前使用诊断格式3并兼容读取格式1／2。它不提供独立CPU验收，原格式2实际范围见[无UAC记录](docs/P2_HISTORY_PAINTER.md)。

完整格式2／3的原固定关／开／开／关采集结束后，可运行 `python tools/measure_painter_cpu.py --capture <仓库内artifacts采集目录> --output <artifacts内新JSON路径>` 离线分析同窗互斥CPU；不启动程序或请求UAC，不改变原判据。已验证范围与CPU失败见[复查记录](docs/P2_PAINTER_CPU.md)。

格式3增加官方事件循环回调；可用 `python tools/measure_event_loop_ui.py --output <新的artifacts目录> --baseline <固定A程序> --candidate <固定B程序>` 执行带严格参考的固定A／B／B／A，两份程序均需已固定的framework-trace构建和同名`skin-editor-desktop.exe`，仅采集器两次请求UAC。它不自动采用候选；当前文字块候选已撤回、CPU仍未通过，见[对照记录](docs/P2_EVENT_LOOP_UI.md)。

[呈现基础成本对照](docs/P2_GUI_BASE_COST.md)工具为 `python tools/measure_gui_base.py --baseline artifacts/p2/保存的基线目录/skin-editor-desktop.exe --output artifacts/p2/gui-base-new`。它使用已保存的自有基线与当前release，固定运行A/B/B/A，四轮观察配置相同，严格参考通过后才允许产品准入。基线文件须位于artifacts内，输出目录须不存在，Windows工具和普通runner要求同上。最新[当前构建原生对照](docs/P2_GUI_BASE_NATIVE.md)经授权中断后续跑完成；CPU仍超限且未显示改善，零欠载、20 ms及资源清理通过，取消与中断记录保留。

该工具当前输出格式2，额外拒绝设备／目标／刷新密度／输入／焦点条件不一致或缺少分段观察的比较，缺少CPU／欠载／20 ms谓词不能通过。[展示文本候选](docs/P2_UI_DISPLAY_REUSE.md)的普通窗口四轮已完成，候选CPU仍超限且首轮基线分段时钟失败；缓存已撤回，原记录保留，尚无CPU收益或P2-A3验收结论。

进一步区分GUI／框架墙钟、线程周期和启动模块，可运行 `python tools/measure_gui_cpu.py --output artifacts/p2/gui-cpu-new`。加 `--backend-control` 使用另一固定四轮计划，对照默认初始化与保留Vulkan／DX12、排除GL；只临时影响自有子进程，实际适配器身份另行记录。它播放低音量自制信号，仍保留原CPU／延迟／资源门槛；[当前结果](docs/P2_GUI_CPU_ATTRIBUTION.md)未关闭旧CPU失败或晋升默认。

以下为保留的独立 P0 实验入口。轻量原生实验可自动验证自己的 GPU 渲染并输出 PNG 后退出：

```text
python tools/check_p0_gui.py --run-native
```

这个 P0 实验没有真实素材库、图层文档或游玩功能，与上面的产品桌面工作区独立。产品打包未实现。

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

目前有占位桌面编辑器，没有正式音频后端或完整的当前 stable 兼容测试。P0 静音流已与实验模拟／输入和合成联动，尚无实际游玩；旧负载也未包含正式命令／历史或完整产品能力，不能代表最终产品预算。

文档变更可用 `git diff --check` 检查补丁空白问题；它不是应用测试，也不检查尚未跟踪的新文件内容。

## 文档导航

- [AGENTS.md](AGENTS.md)：长期项目约束、边界与修改规范。
- [需求与验收](docs/REQUIREMENTS.md)：产品目标、用例、阶段范围、待确认事项。
- [架构](docs/ARCHITECTURE.md)：模块职责、依赖方向、核心模型、数据流和当前决策。
- [开发路线](docs/ROADMAP.md)：阶段产物、进入条件、验收标准与测试计划。
- [P2 桌面记录](docs/P2_PROGRESS.md)：实际功能、依赖、资源上限与待验收部分。
- [技术栈评估](docs/TECH_STACK.md)：候选方案、验证方法、性能预算与选型条件。
- [P0 研究报告](docs/p0/REPORT.md)：旧源码观察、独立实验、环境和未完成验证。
- [当前 stable 验证矩阵](docs/p0/VALIDATION_MATRIX.md)：公开文档、旧观察、当前实测和自有回归的独立记录。
- [F12 与文件证据流程](docs/p0/EVIDENCE_WORKFLOW.md)：自制输入包、只读检查和明确图片的登记方式。
- [精简预览器范围](docs/p0/PREVIEW_SCOPE.md)：保留、简化和省去的原版功能。

推荐先阅读需求与架构，再依据路线图推进。后续实现同步更新功能状态和实际运行说明；按 AGENTS.md 检查，不自动提交或推送。
