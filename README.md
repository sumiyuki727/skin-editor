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
| 文件与接口 | 配置保留原始记录，未实现 skin.ini 解析 UI、项目文件、导出、谱面读取、在线素材库或外部 AI 服务 |

模型版本2、application API6、预览端口2、音效编辑端口3是不同的进程内契约，尚无正式项目文件读写；P3-0独立实验格式不属于产品支持格式。平台结果、硬件后补项和性能限制见[验收记录](docs/P2_FINAL_ACCEPTANCE.md)，不能将本机结果视为 Linux／macOS 通过。

## 安装依赖

需要 Python 3.9+、Rust 1.95.0（rustfmt／clippy）及本机链接工具。Windows 使用 MSVC x64 和 Windows SDK；Linux 音频还需要 ALSA 开发库／pkg-config，Linux／macOS 未实机验证。根工作区只有两个纯核心包；桌面依赖由 `desktop/Cargo.lock` 单独锁定。

首次获取桌面公开依赖的已验证 Windows 命令：

```text
cargo fetch --manifest-path desktop/Cargo.toml --locked --target x86_64-pc-windows-msvc
```

检查脚本离线运行，不自动安装工具、下载依赖或改变全局 Rust。已有独立工具链可传 `--toolchain-dir <bin目录>`，或设置本机 `SKIN_EDITOR_RUST_TOOLCHAIN_DIR`。`.p0-tools` 是本机忽略缓存，不是产品依赖。

## 启动与测试

在仓库根执行：

```text
python tools/check_core.py
python tools/check_desktop.py
python tools/run_desktop.py
```

核心检查包括格式、Clippy、无窗口测试、实际 Cargo 图与源码边界；桌面检查包含默认／全部 feature，不默认打开窗口、GPU或声卡。`--architecture-only` 只检查边界。远程 Windows／Linux CI 已配置但未在本机替代实测。

启动后拖入 PNG，或填写路径导入，再“加入当前元素”。编辑工具下选择图片，拖动／角点缩放／圆柄旋转；Shift锁定比例，拖动中Esc取消，松手一次提交。多选形成临时共同框；切交互工具清空视觉选择。音效编辑窗口支持 WAV 路径导入、组件绑定及显式试听；同源增益可作用于当前试听，Esc恢复本次增益。

显式自有渲染回读与低音量原生音频诊断（输出目录须未存在）：

```text
python tools/check_desktop.py --run-native --output artifacts/p2/native-new-run
python tools/check_desktop.py --run-audio --output artifacts/p2/audio-new-run
```

真实设备诊断、长时压力及 UAC 调度采集不会由默认检查触发；各入口及限制见[文档索引](docs/INDEX.md)。P0 重现入口保留在[历史实验索引](experiments/README.md)，不属于产品启动方式。

## 审阅与版本管理

长期约定见[数据兼容设计](docs/DATA_COMPATIBILITY.md)。P3-0[格式与加载审阅](docs/P3_0_LOAD_REVIEW.md)仅供审阅，未接入产品。可用 `python -B tools/check_p3_storage.py` 运行独立无窗口存储测试；依赖Python标准库sqlite3，不需要窗口／声卡或额外Cargo依赖。

[架构](docs/ARCHITECTURE.md)解释当前模块／依赖和数据流；[审阅包](docs/review/README.md)列出对象、方法、trait／组合关系、引用、待讨论决定及完整源码清单。[文档索引](docs/INDEX.md)区分现行契约、未来设计与历史证据。

```text
python tools/check_repository.py
python tools/check_repository.py --staged
```

仓库检查扫描常见敏感信息及生成／私人路径，不是资源授权证明。Git hook复用正式检查；每轮项目工作先按[长期规则](AGENTS.md)检查并提交已有可审查变更作为本地回退点，用户明确要求不提交时除外；不自动推送。用户设置、原版皮肤／头像／谱面／音乐、密钥和构建诊断不得进入版本库，详见[长期约束](AGENTS.md)。

旧 stable 源码仅作外部只读研究，不进入产品或实验。预览只使用自制或获准输入；对照通过明确的 F12 本地图片及受限文件／日志完成，不依赖系统窗口捕获。发布资源和项目自身许可证仍须在分发前明确。
