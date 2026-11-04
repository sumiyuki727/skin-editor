# 独立 stable 兼容内核：新对话交接

2026-11-03。这是架构与实施计划的交接，**没有实现内核功能**。新对话先读当前AGENTS／Git和用户本轮任务；本文件不授权自动开工、访问私人谱面、运行窗口或提升权限。

## 1. 当前任务与停靠点

用户希望独立实现一个适合皮肤编辑／离线测试的stable兼容内核。首批限定 **osu!standard／No Mod／圆圈子集**，先明确时间、输入、生命周期、编辑器接口和验收。当前主文档为[架构与分阶段计划](STABLE_KERNEL_PLAN.md)，K1–K5和KT01–KT24全部未实施。K0只有设计交付，规则证据准备仍未完成。

“独立”是纯Rust、可无窗口测试的模块；产品保持单进程模块化单体。不能裁剪、复制、翻译移植无授权旧stable源码，不能因为lazer有实现就当stable规则已验证。

推荐首个实现增量为K1，但只有用户明确要求实施后才执行。它先用SyntheticTest预编译圆圈和规则，验证确定性时间／输入／生命周期／冻结；不以此宣称stable兼容。不要自动从该交接转去接P3保存UI或开发素材skill。

## 2. Git、代码基线和并行修改

- 工作区为 E:\skin-editor，分支codex/p1-bootstrap；远端[sumiyuki727/skin-editor](https://github.com/sumiyuki727/skin-editor)须再次核对PRIVATE。
- 当前恢复代码基线为2719d0f；先前交接文档保存于6e26e9d。本轮开始的动画设计回退点为d3f4b2b34b2c088932acf1641163c72915163e41，经过正常仓库hook并同步私有origin；它没有增加动画功能。
- 本轮独立内核方案及关联入口按交付时实际状态保留待审阅，不把文档当已完成代码或已通过KT。
- 最新默认流程：先看status／log／实际暂存差异，将上一轮来源明确、可审查的工作通过检查及hook提交成回退点，默认同步核对过的私有origin；用户明确说不提交／推送时除外。新增量轮末不自动一起提交。
- 保留其他对话工作，不并发Git写操作，不替其他对话暂存，不盲目全目录add、不跳过hook／强推／amend／重写历史。
- schema.rs可能显示M而实际diff为空，已有核对属换行规范化现象；先检查内容，不为消标记修改DDL、清理源码或改旧证明。

如果在同checkout继续，先确认是否有编辑器存储／素材skill对话在工作；按实际文件责任错开写入。需要隔离时遵循当前worktree工具和用户授权，不擅自搬移工程。

## 3. 最短阅读路径

| 顺序 | 内容 |
| --- | --- |
| 1 | [AGENTS](../AGENTS.md)、[Git规则](GIT_WORKFLOW.md)、[README](../README.md) |
| 2 | [内核方案](STABLE_KERNEL_PLAN.md)：范围、未来依赖、时间／输入／代次、K切片与KT矩阵 |
| 3 | [当前架构](ARCHITECTURE.md)、[技术栈](TECH_STACK.md)、[路线](ROADMAP.md) |
| 4 | [传输／定位](PREVIEW_TRANSPORT.md)、[输入](EDITOR_INPUT.md)、[设置／外部来源](PREVIEW_SETTINGS.md)、[GPU交接](PRESENTATION_CONTRACT.md) |
| 5 | [预算v2](RESOURCE_BUDGET.md)、[P2最终证据范围](P2_FINAL_ACCEPTANCE.md)、[首轮预览边界](p0/PREVIEW_SCOPE.md) |
| 6 | 需要编辑器全局背景再读[编辑器交接](HANDOFF_EDITOR_CONTINUATION.md)、[数据兼容](DATA_COMPATIBILITY.md)及P3审阅 |
| 7 | 涉及动画才读[原始动画／元素重采样](ANIMATION_ASSETS.md)，涉及素材skill才读[独立skill交接](HANDOFF_SKIN_ASSET_SKILL.md) |

不用重新读整段旧聊天或重跑P2全部CPU归因。docs/review是2026-10-30代码快照，各P3增量另有审阅；旧证明／归档不覆盖当前新功能。

## 4. 现有能力与具体代码入口

| 入口 | 真实状态／可复用范围 |
| --- | --- |
| [desktop/preview/src/lib.rs](../desktop/preview/src/lib.rs) | 占位PreviewState／Scene、冻结及输入计数；没有谱面、对象、判定或成绩 |
| [application/preview_port.rs](../crates/application/src/preview_port.rs) | 现有PreviewPort、PreviewContext／FrameSource／FrameKey和静态PNG准备值；不是已存在的游戏运行端口 |
| [desktop/ui/src/shortcuts.rs](../desktop/ui/src/shortcuts.rs) | 产品快捷键／修饰键统一入口；文本／IME／工具隔离继续复用 |
| [desktop/infrastructure/src/audio_supply.rs](../desktop/infrastructure/src/audio_supply.rs) | 分离背景／声音、有界控制／回执、触发／暂停／定位入口；尚不能据此宣称精确游戏排程／设备消费时钟 |
| [background_stream.rs](../desktop/infrastructure/src/background_stream.rs) | 有界背景读取／缓存；普通GUI尚未接真实歌曲 |
| [desktop/host/src/main.rs](../desktop/host/src/main.rs) | Editor与模块组装／原生资源生命周期；不能从中绕过application写项目 |
| [当前架构](ARCHITECTURE.md)及[审阅清单](review/README.md) | 图层共享合成、音频候选／资源与历史；按真实源码查具体方法，不编造新的现成API |

当前只有domain／application两个根核心包；桌面六包。gameplay／compatibility尚未创建，crate名／路径是方案建议。没有新增规则集插件、跨进程服务或第二游戏窗口框架。

产品仅支持有界PCM16 WAV，44.1／48kHz单／双声道；压缩音乐只在P0实验，不是产品能力。原生证据限已声明Windows设备；Linux实测暂缓、macOS待设备，真实USB／第二输出端点变化仍硬件后补。普通图形故障冻结诊断是已接受限制，完整窗口恢复只是显式验证入口。

## 5. 不得丢失的设计约定

**项目与运行分离。** application仍是SkinProject唯一活动写入口。GameRun可以拥有可变玩法状态，但不能持有项目写权限；载入、播放、输入、定位和成绩不进入项目历史，皮肤编辑仍经Command／草稿／事务。首版完整工程历史只增不删不可覆盖，所有分支原字节仍保留。

**依赖方向。** 未来gameplay无产品依赖，compatibility→gameplay，preview→gameplay，infrastructure→compatibility／gameplay。host注入application定义的控制／观察端口；application不导入preview、gameplay、egui／wgpu或设备。渲染器／音频不实现判定，UI不持有运行器可变引用。新包必须同步更新两套检查及负向canary，不放宽边界隐藏循环。

**时间。** 内核只消费有符号i64微秒的绝对MapTime，允许负预备时间，0为音频起点；主机单调时、源帧、输出帧、生成／复制／消费位置分开。稳定锚点＋有理／余数换算避免逐块漂移；原客户端判定量化单独由profile验证。现有u64 simulation_micros及100ms增量截断不可直接升级成权威音乐时钟。GUI只取样，不推进歌曲。

**输入。** 按下时固定坐标和捕获时间，按(MapTime, sequence)排序；原生接纳与输入封口须可线性化，不能越过未交接输入推进。重复、迟到、队列满、失焦和按住态分别测试；文本／IME、编辑、冻结及定位阻止手动游戏输入。自动驱动走同一判定，与Auto Mod分离且只能一个驱动生效，恢复不补跑。

**对象／事件。** 对象原始序号及谱面修订标识稳定，Future→VisibleUnjudged→Resolved→Retired，判定与显隐分开。事件有唯一键和有界消费，GUI多画一次不多发声音。未知的notelock／堆叠／目标选择不能以最近圆圈替代；超出受测准入条件整谱拒绝执行，不删掉其他对象冒充兼容。

**冻结／定位／统计。** 冻结固定时刻，编辑可刷新、显式试听可继续，恢复重锚且不补输入／旧声音。定位用新TransportEpoch协调音频、对象与动画，候选一起发布，默认冻结；检查跳过段不生成假Miss或音效。累计score／ACC／combo不可靠时Unavailable，区段／自动／样例来源可见；最终跳转HUD默认方案仍待讨论。

**皮肤交接。** 皮肤热替换、草稿取消、Undo／Redo／Checkout不得重置歌曲／对象／已判定状态。图片、独立音效与背景各自门控，核对自己依赖的完整来源；同源显式试听增益例外继续沿用。一个圆圈的多个部件可复用同一文档，但图层变换不改变命中点／半径。GUI区域、变换框、玩法判定区不同。

**外部资源。** 用户选定stable／Songs只读来源属于本机设置，先校验根／真实路径／重解析边界／限额；谱面／歌曲不进AssetLibrary、.oskproj、Git或皮肤导出。默认素材发布须自制／获授权；旧stable代码／附带资源不得进入产品、实验依赖或fixture。

**格式／动画。** 运行器不要求改工程schema；持久设置或必要项目语义确需演进再单独定版本／迁移。历史短地址继续与永久身份分离，不借用为对象ID。动画来源保留原始时长、按元素规则输出的AN设计不变，程序化接近圈动画不等于已实现动图导入。

## 6. 下一增量、检查和证据

用户批准实施K1时，先补足K0的首批准入／规则证据表，至少区分SyntheticTest、官方文档、当前客户端已验证、未知。可先做不依赖具体stable数值的小模型；必需未知项不能包装成兼容profile。公开wiki给出线索，窗口取整／时序、几何边缘与顺序仍须按范围核对。

建议K1每次只处理一个可审查增量：

1. 定义强类型时间／坐标／身份、不可变圆圈输入、显式SyntheticTest规则和有界输出；增加gameplay与检查规则。
2. 先验证KT05／KT06：绝对时间分段不变、生命周期及事件只发生一次。
3. 接带时间戳的两路输入／自动驱动，覆盖KT08／KT09的重复、位置、顺序／封口与超限。
4. 补冻结／恢复／检查定位、旧代次及资源所有权，覆盖KT10／KT11，记录10000圆圈成本基线。
5. K1审阅后再加K2纯解析器／profile，然后K3编辑器／WAV／材料接线，K4手动圆圈对照，K5原生稳定性；不跨阶段宣称完成。

以上是建议，不是本轮已有实现。真实谱面解析前确认支持的版本、字段／未知规则和限额；原生接线前明确音乐时钟／排程能力，不把现有trigger当万能定时器。不先搭通用玩法框架或全套UI。K3另有三项接线风险：锁定eframe是否能在布局前捕获／封口原生输入；背景复制锚点与真实消费位置的差异；静态元素缓存到多圆圈场景绘制的增量。未解决前只能明确较窄能力，不能把帧时间当硬件时间、即时trigger当精确排程或现有合成器当完整场景渲染器。

现有检查入口：

```text
python -B tools/check_repository.py
python -B tools/check_core.py
python -B tools/check_desktop.py
```

Rust1.95.0，工具链路径与本机缓存按README核对；不得静默下载／改全局工具链。新增包同步两个Cargo工作区的实际依赖检查、源码规则和禁止边的负向回归。默认检查无窗口、无GPU／声卡、无网络／UAC；原生／人工测试另经明确任务执行。

继承零欠载、已就绪短音完整请求到原生复制观察p95≤20ms、进程内存／GPU及实际释放门槛；播放单核CPU≤15%为优化目标，稳定空闲硬项不豁免。20ms不是设备消费、声学或判定误差指标；到期→接纳的宿主延迟另外报告。新内核先fake／门闩再短原生，最后当前产品入口两轮300秒／20次循环；不能套用P2旧证据直接判新路径通过。

所有新KT均待实施；本轮只做文档链接／范围／来源与保留检查，前置回退hook验证既有代码，不证明内核已存在。不重写旧_PROOF.json或清理历史失败来制造通过。

## 7. 可粘贴的新对话开场

```text
请在 E:\skin-editor 接手独立stable兼容内核规划。先读AGENTS.md、docs/HANDOFF_STABLE_KERNEL.md、docs/STABLE_KERNEL_PLAN.md及当前Git状态／HEAD／暂存差异，按最新规则保护上一轮已审查工作，核对私有origin并保留其他对话修改，不并发Git写入。

首批仅osu!standard／No Mod／纯圆圈受测子集。独立内核仍在同一模块化单体内；gameplay／compatibility尚未实现。application保持唯一项目写入口，运行控制与tick不进入项目Command／历史。先复核时间权威／输入封口／生命周期／来源代次和K0准入表；未知stable数值不得硬编码成已验证规则，不复制旧stable源码，也不把lazer当stable证据。

本轮先与我确认方案审阅及首个实现任务；不要仅凭交接自动新增功能、接P3保存／打开UI、访问私人谱面或启动窗口／设备。若我明确要求实施K1，先做无窗口SyntheticTest小模型和KT05／06／08／10等确定性回归，再按实际增量补检查，不跨阶段宣称兼容或完整standard。

保持完整历史不删不覆盖、图片与音效独立、皮肤热替换不重置音乐／对象、冻结恢复不补跑及旧代次清理。零欠载、提交p95≤20ms、内存／GPU／实际释放门槛保持，播放CPU≤15%为优化目标。新原生路径和兼容性须另验，当前仅有文档设计。
```
