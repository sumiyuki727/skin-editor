# osu!stable 皮肤编辑器（暂定名）

面向 Windows／Linux 的模块化单体桌面编辑器，macOS 待设备验证。当前有可运行的**内存编辑器**：PNG 素材库、嵌套图层组、临时跨元素共同变换、分叉历史，以及独立音频素材／音效编辑和 WAV 试听。真实 stable 场景／游玩、项目保存／打开、实际皮肤导出和外部 AI 传输尚未实现；关闭窗口会丢失当前内存项目。

## 当前能力

| 能力 | 已实现与限制 |
| --- | --- |
| 项目核心 | Rust domain／application，原子 Command、冲突／去重、撤销／重做／检出／分叉、有界短地址、一次提交的可取消草稿；核心无第三方依赖 |
| 图像编辑 | PNG 入库／缩略图、普通层与嵌套组、变换／透明度、跨元素临时联合选择、CPU／GPU 共享合成计划；当前有明确资源和画布能力上限 |
| 桌面交互 | 画布拾取、Shift比例约束、旋转控制框、集中快捷键及文本／游戏输入隔离；历史面板为有界祖先／直接分支，不是完整树视图 |
| 音频 | WAV PCM16 44.1／48 kHz 单／双声道准备、独立组件绑定／增益／隐藏、原始素材与组件试听、监听音量；其他编码可只保留原字节。设备需显式启用，普通 GUI 未载入歌曲背景 |
| 预览 | 两个占位场景、冻结下编辑刷新及全屏入口；没有完整 stable 内核。正常 Windows 使用分离声音和 MemoryUsage，图形故障冻结／诊断，不自动重开窗口 |
| 合成玩法核心 | 独立纯Rust `gameplay` API7／profile1：Tiny为4对象，Reference支持分步准备10000个预编译圆圈；G0无输入、G1固定输入和A1实际Automatic规模轨迹及有限冻结／恢复／检查交错已有回归。来源共享、有界变长存储、分步校验／初始化及工作计数，实时账本与只读完整帧／分页分离；三驱动互斥、重新武装及旧来源隔离。仅SyntheticTest，成本补证未收口，不接桌面预览、不宣称stable兼容 |
| 文件与接口 | 配置保留原始记录，未实现 skin.ini 解析 UI、项目保存／打开 UI、导出、谱面读取、在线素材库或外部 AI 服务 |
| 存储基础 | 状态块编解码、完整历史捕获与永久身份；Rust SQLite 增量适配、完整文件／原字节验证和恢复候选；Windows显式副本热日志恢复及进程中断测试。尚未接入正式保存／打开 UI、自动保护、另存或迁移 |

模型版本2、application API6、预览端口2、音效编辑端口3是不同的进程内契约。Rust 文件适配使用独立 profile，尚无正式项目保存入口；P3-0实验格式不是它的输入。平台结果、硬件后补项和性能限制见[验收记录](docs/P2_FINAL_ACCEPTANCE.md)，不能将本机结果视为 Linux／macOS 通过。

历史捕获端口单独使用版本2，存储端口版本1，不改变现有命令协议；对象、限制和文件恢复验证见[P3-1C审阅](docs/P3_1_STORAGE_REVIEW.md)。测试包含在正式检查入口中，不需要设备或窗口。

Windows的[恢复补充](docs/P3_1_RECOVERY_REVIEW.md)保留原件并恢复到新文件，需关闭源文件占用；未知日志、其他平台冻结和物理断电仍有明确限制，没有正式恢复UI。

## 安装依赖

需要 Python 3.9+、Rust 1.95.0（rustfmt／clippy）及本机链接工具。Windows 使用 MSVC x64 和 Windows SDK；Linux 音频还需要 ALSA 开发库／pkg-config，Linux／macOS 未实机验证。根工作区有domain／application／gameplay三个纯核心包；桌面依赖由 `desktop/Cargo.lock` 单独锁定。

首次获取桌面公开依赖的已验证 Windows 命令：

```text
cargo fetch --manifest-path desktop/Cargo.toml --locked
```

检查脚本离线运行，其 Cargo metadata 会检查完整锁文件，因此首次预取不限制 target。脚本不自动安装工具、下载依赖或改变全局 Rust。已有独立工具链可传 `--toolchain-dir <bin目录>`，或设置本机 `SKIN_EDITOR_RUST_TOOLCHAIN_DIR`。`.p0-tools` 是本机忽略缓存，不是产品依赖。SQLite bundled 绑定另需本机 C 编译器；Windows 使用已有 MSVC。

## 启动与测试

在仓库根执行：

```text
python tools/check_core.py
python tools/check_desktop.py
python tools/run_desktop.py
```

核心检查包括格式、Clippy、无窗口测试、实际 Cargo 图与源码边界；桌面检查包含默认／全部 feature，不默认打开窗口、GPU或声卡。`--architecture-only` 只检查边界。远程 Windows／Linux CI 已配置但未在本机替代实测。

正式核心检查包含独立的 SyntheticTest 成本工具，可单独运行 `python -B tools/check_gameplay_cost.py --check`。当前新构建输出schema3／CostClosure-PC128-v1，读取器为CostClosure-PC128-reader-v1；--check验证六种Reference组合的真实Rust补充输出，PC128贯通由检查入口明确执行一次，普通cargo test里的该测试标记为ignored，不能单独代替完整检查。`python -B tools/check_gameplay_cost.py --verify <schema3证据目录>`只读复核新计划或旧CostClosure-Query-v1／CostClosure-v1，不构建、执行冻结程序或采样；旧读取器规则独立保留。

Windows显式入口 `--measure --output artifacts/gameplay-cost-new` 要求全新目录，固定三轨迹／两预算／各两个进程并执行冻结副本。PC128正式测量尚未运行；既有完整PC32轮已实际测量及只读复核，协议／保管Valid、coverage NeedsEvidence、产品NotEvaluated，详见[结果与限制](docs/K1_EXIT_REVIEW.md#2026-11-09完整成本新轮与退出决定)。

公开远端、检查后提交／同步和缓存整理规则见[Git 管理](docs/GIT_WORKFLOW.md)。

启动后拖入 PNG，或填写路径导入，再“加入当前元素”。编辑工具下选择图片，拖动／角点缩放／圆柄旋转；Shift锁定比例，拖动中Esc取消，松手一次提交。多选形成临时共同框；切交互工具清空视觉选择。音效编辑窗口支持 WAV 路径导入、组件绑定及显式试听；同源增益可作用于当前试听，Esc恢复本次增益。

显式自有渲染回读与低音量原生音频诊断（输出目录须未存在）：

```text
python tools/check_desktop.py --run-native --output artifacts/p2/native-new-run
python tools/check_desktop.py --run-audio --output artifacts/p2/audio-new-run
```

真实设备诊断、长时压力及 UAC 调度采集不会由默认检查触发；各入口及限制见[文档索引](docs/INDEX.md)。P0 重现入口保留在[历史实验索引](experiments/README.md)，不属于产品启动方式。

## 审阅与版本管理

长期约定见[数据兼容设计](docs/DATA_COMPATIBILITY.md)。实际 Rust 状态 schema 1 的入口、检查与限制见[P3-1A审阅](docs/P3_1_STATE_REVIEW.md)，其无窗口测试包含在桌面检查中；它不是可打开的工程文件。P3-0[格式与加载审阅](docs/P3_0_LOAD_REVIEW.md)是独立工具实验。可用 `python -B tools/check_p3_storage.py` 运行该实验存储测试；依赖Python标准库sqlite3，不需要窗口／声卡或额外Cargo依赖。

[架构](docs/ARCHITECTURE.md)解释当前模块／依赖和数据流；[审阅包](docs/review/README.md)列出对象、方法、trait／组合关系、引用、待讨论决定及完整源码清单。[文档索引](docs/INDEX.md)区分现行契约、未来设计与历史证据。

```text
python tools/check_repository.py
python tools/check_repository.py --staged
```

仓库检查扫描常见敏感信息及生成／私人路径，不是资源授权证明。Git hook复用正式检查；每轮项目工作先按[长期规则](AGENTS.md)检查并提交已有可审查变更作为回退点，并同步到已确认归属及用户指定可见性的origin（当前PUBLIC），用户明确要求不提交／推送时除外。本轮新增工作检查后保留供审阅，下一轮再按规则保存；用户设置、原版皮肤／头像／谱面／音乐、密钥和构建诊断不得进入版本库。

旧 stable 源码仅作外部只读研究，不进入产品或实验。预览只使用自制或获准输入；对照通过明确的 F12 本地图片及受限文件／日志完成，不依赖系统窗口捕获。发布资源和项目自身许可证仍须在分发前明确。
