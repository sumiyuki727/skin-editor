# 首批 stable 对照操作卡

日期：2026-10-20。**C00 身份、C01 基线和 C03／C08 首轮三个定性子项已记录**，见[基线观察](STABLE_OBSERVATIONS_20261020.md)与[本轮结果](STABLE_C03_C08_RESULTS.md)。工具没有自动安装、改客户端设置或启动客户端。完整兼容范围、音频、判定和游玩规则另批验证。

生成方式（输出必须尚未存在）：

```text
python tools/p0_stable_batch.py --output artifacts/p0/stable-first-batch-v1 --client-dir "D:\osu!"
```

`--client-dir` 可省略；提供时只执行受限元数据检查，不读取认证信息或旧截图。生成包时检查到 `_ReleaseStream=Stable40`，可执行文件 SHA-256 `3adc2d001aacced5f92ba8ae24121980d5e23f5653f37d14cf4765bca1abd9f9`；当时尚未取得实际发布编号，旧日志／文件版本不能替代。

上述是生成包时的检查记录；2026-10-20 已通过指定图片补齐 **b20260924.1／Stable**，exe 哈希未变。用户确认 Baseline、Mods 关闭、1920×1080 全屏和 Windows 缩放 100%。下面 C00／C01 步骤保留供复测，不需要重新提供版本图片；完整行为验收仍未通过。

**[C03／C08 操作卡](STABLE_C03_C08.md)保留供复测**：七张核心图片已审查，不需要重复；独立活动游玩请求尚未操作。基线有粉红 overlay、全透明不显示该层、缺失显示默认素材；菜单刷新变绿，普通暂停刷新仍旧、直接返回选歌后变绿。用户已确认恢复 A 并刷新，完整结论和范围以结果记录为准。

本机现有[完整操作步骤](../../artifacts/p0/stable-first-batch-v1/STEPS.md)、[输入清单](../../artifacts/p0/stable-first-batch-v1/manifest.json)、[观察模板](../../artifacts/p0/stable-first-batch-v1/observation-template.json)只在本机 artifacts 中保留。可用同一工具在新目录重建。所有图案、数字、谱面和静音由项目独立生成，无客户端／旧源码资源。

## C00／C01 原操作步骤（已有基线证据，供复测）

1. 手动运行 `D:\osu!\osu!.exe`，确认进入 stable；主菜单打开设置，将版本和更新通道保持可见，按 **F12**，同时抄下完整发布编号。如果界面未显示完整版本，记录实际可见文字，不推断。
2. 将[包目录](../../artifacts/p0/stable-first-batch-v1/packages)中的 `P0 First Batch Baseline.osk` 与 `P0 First Batch.osz` 分别拖到这个 stable 窗口；已安装时无需重复导入。在设置选择 `P0 First Batch Baseline`，关闭 Mods；记录原设置以便恢复。导入用压缩包可能不再留在原目录；源图、谱面及清单另存，需重新准备时使用上方生成命令的新输出目录。
3. 搜索 `P0 First Batch`，选择 `Static Circles` 并开始。第一个圆圈约在两秒处；出现后、尚未消失时按 **F12**，不需要击打。后续每三秒一个圆圈，错过可重试；记录对象序号及是否在游戏内暂停。
4. 提供这两张**新图片的完整路径**或直接发送图片，并附分辨率、窗口／全屏、Windows 显示缩放、背景暗化设置。默认本地目录为 `D:\osu!\Screenshots`，若已自定义则使用实际路径。只登记指定文件，不扫描历史截图。

F12 保存本地；**不要按 Shift+F12**，该组合会上传。快捷键和刷新时机依据[官方快捷键说明](https://osu.ppy.sh/wiki/en/Client/Keyboard_shortcuts)，查阅于 2026-10-19；页面说明不是当前具体客户端已经通过的证据。安装时拖到已确认的 stable 窗口，避免系统文件关联误开 lazer。

## 后续准备好的变量

| 案例 | 独立输入 | 待记录的观察 |
| --- | --- | --- |
| C00 | 同一可执行文件与设置界面 | 实际发布版本／通道、截图及元数据身份 |
| C01 | 64×64 非对称标记、固定八个圆圈、26 秒静音 | 显示几何、角标、中心及 overlay；不预设显示比例 |
| C03 | 基线／全透明 overlay／缺失 overlay 三个皮肤 | 同一对象与设置下的差异；不能将“看起来一样”直接判定为相同回退规则 |
| C08 | 本包 Baseline 的两张 A 图、`refresh-B` 两张 B 图 | 菜单刷新、游玩／普通暂停下何时变化，分别记录 |

C08 只手动替换本包 Baseline 中 `cursor.png`、`hitcircleoverlay.png`，不覆盖其他皮肤。菜单按 `Ctrl+Alt+Shift+S` 后记录 F12；官方说明游玩／结算中的刷新可能延后至选歌，不能将普通暂停等同于编辑器冻结模拟。完成后从本包 `skins/baseline` 恢复 A 的两个文件，再选择原皮肤和原设置；详细步骤均在操作卡。

清单记录每个输入的哈希。观察登记与审查结论分开，执行后仍需版本／配置／fixture／指定图片形成完整证据链。该包未验证音频同步、缺失资源的所有规则或 stable 内核；生成、打包和图片登记测试不能增加兼容案例通过数。格式参考：[.osz](https://osu.ppy.sh/wiki/en/Client/File_formats/osz_(file_format))、[skin.ini](https://osu.ppy.sh/wiki/en/Skinning/skin.ini)、[osu! 皮肤元素](https://osu.ppy.sh/wiki/en/Skinning/osu%21)。
