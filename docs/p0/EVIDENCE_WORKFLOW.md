# 文件、日志与 F12 证据流程

日期：2026-10-20。已按用户要求取消系统窗口捕获依赖。真实客户端使用 F12 本地图片或用户明确提供的截图；自有原型通过自身渲染器回读输出图片和结构化状态。

## 文件与日志检查

```text
python tools/p0_evidence.py inspect-client --client-dir "D:\osu!" --output artifacts/p0/client-20261018.json
```

此命令只读目标目录，记录 osu!.exe 哈希、允许的发布流/版本字段，以及 runtime/session/update_success 三类日志的受限采样元数据。不会读认证日志或个人配置，不输出日志原文。日志中找到的版号是未核实线索，不能自动认定为当前客户端发布版。

命令拒绝覆盖已有输出；重复执行时使用新文件名。早期文件检查结果为 `_ReleaseStream=Stable40`；session.log 含历史版号线索 `20260101.1`，其修改时间早于当时的运行日志，不能认定为实际发布版本。

以上为早期文件检查结果。2026-10-20 的[指定图片审查](STABLE_OBSERVATIONS_20261020.md)已直接确认 **b20260924.1／Stable**，并补齐 C01 的 Baseline／无 Mods／1920×1080 全屏／100% 缩放设置。保留旧报告原样，新的图片、设置确认和审查单独追加；C00 身份与 C01 基线观察不等于全部兼容行为通过。

## 自制输入素材

```text
python tools/p0_evidence.py make-fixture --output artifacts/p0/circle-fixture-v1
```

输出包括：

- `skin/`：skin.ini、光标、单击对象、overlay、approachcircle 和 0–9 数字测试图片。
- `beatmap/`：6 个单击对象的自制 `.osu` 测试谱面和 10 秒静音 WAV。静音用于可重复载入，不用于验证真实音频同步或 hitsound。
- `manifest.json`：每个输入文件的路径与 SHA-256，以及“未验证客户端执行”的标记。

图片、数字图案、谱面和静音音频均由本项目独立生成，无旧源码、默认皮肤、字体或第三方歌曲。尺寸、位置与时序是测试选择，不是未经验证的产品兼容规则。命令只在 `artifacts/p0` 内生成新目录，不向游戏目录安装文件、不修改用户设置，也不把静态文件生成成功视为客户端案例通过。

格式研究入口：[skin.ini](https://osu.ppy.sh/wiki/en/Skinning/skin.ini)、[osu! 元素](https://osu.ppy.sh/wiki/en/Skinning/osu%21)、[.osu 格式](https://osu.ppy.sh/wiki/en/Client/File_formats/osu_%28file_format%29)。后续加载方式与测试皮肤安装位置随具体案例提供；不覆盖已有资源。

首批实际对照现另有[操作卡](STABLE_FIRST_BATCH.md)与 `tools/p0_stable_batch.py`：三种 overlay 变量的 `.osk`、同一八圆圈／26 秒静音 `.osz`、刷新 A／B 图及逐文件哈希。与上面的基础六圆圈 fixture 不混用；每张图片必须登记实际采用的 manifest。C00、C01 基线和 [C03／C08 首轮结果](STABLE_C03_C08_RESULTS.md)已取得；图片登记、人工审查、用户默认素材来源确认分别保留，工具不会安装或更改客户端设置。生成时清单中的执行数 0 是历史状态，不原地改写该输入包。

## 获取并登记本地图片

原版默认 `F12` 将截图保存在 Screenshots 目录；`Shift+F12` 带上传行为，本流程只使用本地 F12。[官方快捷键](https://osu.ppy.sh/wiki/en/Client/Keyboard_shortcuts)、[文件目录说明](https://osu.ppy.sh/wiki/en/Client/Program_files)。

当某个案例需要图像证据时，再明确该次场景、测试素材、客户端版本及操作步骤，由用户提供对应 F12 文件。不会扫描或读取历史私人截图作为新案例证据，也不会盲发按键导航。无需为了其他可自动执行的工作提前提供图片。

登记示例（替换图片路径和版号；以下仅示意，没有读取该图片）：

```text
python tools/p0_evidence.py register-screenshot --image "D:\osu!\Screenshots\指定文件.png" --client-report artifacts/p0/client-20261018.json --case C01 --release "实际版号" --fixture artifacts/p0/circle-fixture-v1/manifest.json --output artifacts/p0/C01-first
```

登记工具只读取明确指定的 PNG/JPEG，保留原始字节、尺寸、哈希、声明版号、客户端哈希与 fixture 清单哈希。图像检查限于格式头元数据，不能证明图片可完整解码或画面正确；仍须读取图片并人工/自动比对。PNG 用于需要精确像素的后续案例；JPEG 可作布局和状态线索，但必须记录有损压缩，不能直接用于严格边缘/色值相等判断。

版号、fixture 或场景信息缺失时保留未完成状态。`C00` 用于客户端身份信息，其余案例编号对应验证矩阵。登记不会增加已执行/通过案例计数，`compatibility_passed` 始终为空，直到另行完成图像审查与案例判定。

## 不依赖窗口的自有原型验证

独立契约探针验证时间和输入；原生候选直接从自身 GPU 渲染目标回读 PNG。轻量入口隐藏测试窗口，标准 PNG 负载使用自有可见测试窗口和目标 60 Hz 的脚本，记录真实编辑间隔、CPU、进程/GPU 内存及 60 秒静止观察。两者都不调用系统窗口捕获，也不读取其他客户端画面。详见 [实验说明](../../experiments/p0-egui/README.md)。

这不能证明当前 stable 的显示行为，也不代替物理输入、输入法、真实音频和跨平台人工体验检查。两类证据独立保留。

## 检查入口

```text
python -m unittest discover -s tools/tests -v
python tools/check_p0.py
python tools/check_p0_gui.py --run-native
python tools/check_p0_gui.py --run-standard
python tools/check_p0_gui.py --run-comparison
```

前两项分别为文件/日志/fixture/图片登记测试，以及原有 10 项无依赖契约测试。最后一项需要已准备的 Rust 1.95、锁定的原型依赖和本机图形设备；结果是否通过以具体运行记录为准。正式应用的保存、导出、兼容与游玩测试仍待后续实现。
