# 编辑器开发交接：从 P3-1C 继续

2026-11-14最新停靠点：[P3-2A A1实现与证明](P3_2A_A1_REVIEW.md)已落地API7／捕获3／协调端口1、八个固定控制分区、512KiB静态证明、单槽准入与真实捕获关联；仅PreparationOnly，不能保存文件或关闭项目。A2／A3及PA未覆盖部分继续欠项。本轮完成A1后停止，等待用户明确指令，不自动进入下一任务。轮前修订2及9份同步文档已正常hook提交推送1661c84e1a342f90d70bd84212cfa31d0b8a8d51，[三项CI](https://github.com/sumiyuki727/skin-editor/actions/runs/43483039794)成功；A1新源码／文档未提交，其验证另见实现记录。PUBLIC源码／PRIVATE恢复材料、旧证据及schema.rs既存空差异标记保持；3435405固定快照不自动包含本轮。下方日期段为历史记录。

2026-11-14最新停靠点：[P3-2A契约修订2](P3_2A_START_CONTRACT.md#11-2026-11-14独立一致性审阅)已独立审阅并补齐10组边界，PA01–PA38全部待实施。关键为捕获3独立门控、首次发布来源、提交前PreparedWitness／候选保管、独立查询取消、关闭失败收尾／显式重试及512KiB全部所有者核算。后续按具体任务从A1版本化值／固定槽／静态容量证明起，不自动开发。当前仍API6／捕获2／存储1，全部文件格式不变。上轮10文档已正常hook提交推送21f8fc935bbbaa6c25ecb4104925fd6cf0d3df21，[三项CI](https://github.com/sumiyuki727/skin-editor/actions/runs/43473977545)成功；本轮新修订尚未提交。schema.rs既存空差异标记及原字节保留；PUBLIC主仓库／PRIVATE恢复材料不变，3435405固定快照不自动包含新轮次。下方日期段为历史记录。

2026-11-14当前停靠点：[skin.ini拖拽／缩放规格修订2](SKIN_INI_VISUAL_EDITING.md#9-2026-11-14一致性审阅与剩余准入)已完成文档审阅，10组发现、VL01–VL22均待实现／验收。重点约束是原文字节／共享参数整组写回、application最后意图提交资格、关闭专属收尾、同源完整呈现及完整保存重开；现有草稿API不自动具备新守卫。后续主线仍先细化P3-2A开工契约，专题V0覆盖表及具体参数证据仍欠项，不自动开发。轮前HEAD／远端为3435405、六项CI成功；无新增可提交内容，未造空提交。上一轮完整备份实际为公开源码入口＋[私有恢复套件](https://github.com/sumiyuki727/skin-editor-recovery/releases/tag/backup-20261111-visual-config)，本轮只读核对小型回执与附件元数据，未重做全量恢复或成本测量。当前新文档修订未提交、不属于旧固定快照；下一轮按规则先备份。下方带日期内容保留当时事实。

2026-11-11新增需求：[skin.ini可视化编辑](SKIN_INI_VISUAL_EDITING.md)不限mania，所有适合可视化调节的配置都需对应能力；当前只定约拖拽／缩放，R31及VL01–VL14已登记、尚未开发。现有图片／组件变换和通用配置记录不是配置布局编辑。后续先审阅专题，再按V0–V3逐项证据与闭环实施；当前P3-2A开工契约／保存闭环顺序保持，不重测或扩展K1。本轮用户另要求结束前将新文档也检查提交推送，并核对远端可恢复范围；轮前回退点bfd7b38及最终同步状态见[Git记录](GIT_WORKFLOW.md)。下方旧快照不作为最新备份状态。

2026-11-10当前工作：上一轮11份P3规格文档已通过正常完整hook备份并公开推送10c8d14f301df1cae2f0cd12b4b42b8df87a6bd9，[GitHub三项检查](https://github.com/sumiyuki727/skin-editor/actions/runs/42735776243)全部成功。本轮完成[P3-2规格一致性审阅](P3_SAVE_LOOP_SPEC.md#11-2026-11-10一致性审阅与剩余准入)，修正NoChange／未知提交、取消归属、无Manual激活、捕获释放及容量相容等8组问题，20项验收按A–D分层；仅文档修订、未开发。下一步先备份，再细化P3-2A无窗口开工契约，不自动实施。B前的恢复区／配额及转正身份关系仍待定约。K1保持SyntheticTest纯核心已签收范围、Reference10000／Tiny4、产品NotEvaluated／stable Unknown，未重测或扩展；皮肤skill不推进。下方带日期内容保留各自时点。 本页早期P3-1C记录是2026-11-03快照。

交接日期：2026-11-03。供没有旧聊天上下文的新对话使用。本文是有日期的交接快照；执行时先核对最新代码、Git、AGENTS.md 和用户本轮指令，不把建议当作已实现功能或新任务授权。

2026-11-11恢复入口补充：[项目恢复说明](PROJECT_RECOVERY.md)规定源码Git／bundle与历史artifacts加密分卷的组合恢复。密钥不在项目和远端附件中；后续接手必须核对本轮发布最终回执，不把较早“原始诊断仅本机保管”的日期记录理解为本轮结束后的完整备份状态。后续新材料不自动属于这次快照。

## 1. 先建立工作基线

- 工作区：`E:\skin-editor`；当前分支 `codex/p1-bootstrap`。
- 私有远端：[sumiyuki727/skin-editor](https://github.com/sumiyuki727/skin-editor)。本轮再次核对为 PRIVATE。
- 最新已保存且远端SHA核对一致的动画设计回退点：`d3f4b2b34b2c088932acf1641163c72915163e41`；交接初版为`6e26e9dd02c5011cb1c1c0e0c778b8c5e32e79a5`，恢复代码基线仍为`2719d0fbd3a57c024f501416085855f7979b8032`。
- 上一存储基线：`bd3b6262d2a39830d3c94c4c81227cb5725a1700`；Git 管理基线：`1778c378e9dd4662a4a1e2cd39473f905b4330a6`。
- `2719d0f`正常运行仓库hook，包含18个已审查文件；两份交接初版及索引保存于`6e26e9d`，动画设计经正常完整hook后保存于`d3f4b2b`。这些文档提交不增加动画／玩法功能；本轮独立内核设计按交付时状态保留待审阅。

先读根 [AGENTS.md](../AGENTS.md) 和 [Git 流程](GIT_WORKFLOW.md)，再检查状态、最近提交、实际暂存差异及远端。最新默认规则是：每轮先把已有、来源明确且可审查的上一轮工作通过检查保存成回退 Commit，并同步到核对过的私有 origin；用户明确要求不提交／推送时除外。轮末新工作不自动一起提交。不要回退到聊天早期的“不自动推送”旧规则。

如果另一个对话正在同一 checkout 工作，不替它暂存或提交修改，不并发运行 Git 写操作。优先错开写入；需要隔离工作区时按用户授权和当前 worktree 工具规则处理，不擅自新建或搬移。素材 skill 对话的责任见[另一份交接](HANDOFF_SKIN_ASSET_SKILL.md)。

已知工作区现象：`schema.rs` 可能在 `git status` 显示 M，但实际 `git diff` 为空。已有核对表明其规范化内容与 HEAD 相同，涉及 CRLF／LF 和工作区状态，不改变 Rust 编译后的 DDL。先核对实际 diff／blob，不为消除标记改 schema 或改写旧证明；不要盲目全目录暂存。

## 2. 按需阅读，避免重新加载全部旧记录

| 顺序／用途 | 入口 |
| --- | --- |
| 当前产品能力、依赖与运行 | [README](../README.md)、[架构](ARCHITECTURE.md)、[技术栈](TECH_STACK.md) |
| 当前停靠点 | [P3-1C恢复审阅](P3_1_RECOVERY_REVIEW.md)、[恢复证明](P3_1_RECOVERY_PROOF.json) |
| 完整存储／恢复对象与限制 | [存储审阅](P3_1_STORAGE_REVIEW.md)、[存储证明](P3_1_STORAGE_PROOF.json) |
| 数据兼容与文件版本 | [数据约定](DATA_COMPATIBILITY.md)、[格式设计](PROJECT_FORMAT_V1.md)、[内容schema](PROJECT_STATE_SCHEMA.md) |
| 新增动画设计，尚未实现 | [动画来源／原时长／元素重采样及AN01–AN16](ANIMATION_ASSETS.md) |
| 独立兼容内核设计，尚未实现 | [standard／No Mod／圆圈架构及K／KT计划](STABLE_KERNEL_PLAN.md)、[内核专用交接](HANDOFF_STABLE_KERNEL.md) |
| 捕获与保存调度 | [捕获审阅](P3_1_CAPTURE_REVIEW.md)、[自动保护设计](AUTOSAVE_POLICY.md) |
| 后续顺序与待定事项 | [路线图](ROADMAP.md)、[决策清单](review/DECISIONS.md)、[文档索引](INDEX.md) |
| 对象／方法／trait／组合关系 | [代码审阅包](review/README.md)、[系统图](review/SYSTEM_MAP.md)及各P3增量审阅 |
| P2已验范围与后补 | [最终验收](P2_FINAL_ACCEPTANCE.md)、[资源预算v2](RESOURCE_BUDGET.md) |

代码审阅包中的全量生成清单是2026-10-30快照；后续P3对象在各增量审阅中补列，不能把旧清单当作当前完整源码目录。旧文档中“P3尚未开始”“无文件适配”等按其日期阅读；当前已有独立存储基础，但普通GUI仍无保存入口。

## 3. 产品目标与当前实际能力

目标是跨平台 osu!stable 皮肤编辑器，支持素材准备、分层编辑、必要游戏场景预览、实际游玩测试、完整历史及导出。Windows 是当前原生实测范围，Linux 实测暂缓、macOS 待设备。不能把本机性能或CI核心检查推广为其他平台的桌面／设备验收。

已实现：内存编辑器、PNG素材库、嵌套图层组、临时跨元素共同变换、统一Command／原子事务、可取消草稿、分叉历史及有界导航；独立音频素材／音效组件和WAV试听；占位预览、冻结／输入隔离；真实内容codec、完整历史捕获、Rust SQLite增量文件和恢复候选。

未实现：普通GUI正式保存／打开／崩溃恢复、自动保护／退出补存、正式标签编辑、SaveAs／Schema迁移、偏好持久化、实际皮肤文件导出、完整stable内核／真实玩法、普通GUI歌曲背景、stable／Songs受限读取、在线素材库、外部AI传输。**当前直接关闭普通编辑器仍会丢失内存工程；不能把存储测试入口宣传成产品保存功能。**

同日动画补充只确定设计：当前还没有动画资产、逐帧时间、动图解码、编号PNG序列关联或动画导出。静态PNG能显示、原始字节可保管或schema中的分页sequence，均不能算动画支持。

本次用户新明确：接入Codex的重要目的，是利用GPT生图制作皮肤素材；另一对话将先制作素材skill并预留编辑器操作skill。它不意味着本轮已经给编辑器添加生图后端或MCP服务，也不自动改变P3／P6的实施顺序。

## 4. 模块与唯一写入口

采用同一Rust进程内的模块化单体。根工作区为无第三方依赖的domain／application；桌面工作区另外六包、独立锁文件。

| 模块 | 核心职责／直接项目依赖 |
| --- | --- |
| `crates/domain` | 不可变项目值、图层／组件树、独立音效、配置与不变量；无项目外依赖 |
| `crates/application` | `ProjectSession`唯一活动项目写入口；Command、事务、草稿、历史、捕获、恢复工厂、控制端口；只依赖domain |
| `desktop/rendering` | 共用合成计划、CPU／GPU合成及派生资源；依赖domain |
| `desktop/audio` | 共享PCM、声音实例、混音与来源门控；依赖domain |
| `desktop/preview` | 当前占位场景／模拟状态；不写项目、不负责保存 |
| `desktop/infrastructure` | 原字节／图像／解码／设备、状态codec、SQLite与恢复；依赖application、audio、rendering |
| `desktop/ui` | 面板、选择／手势、统一输入路由及窄呈现端口；依赖application |
| `desktop/host` | 组装、原生呈现／生命周期、线程与显式诊断入口；注入各适配器 |

UI与未来AI都提交application值请求；不能互相调用内部实现，不能获得可变项目／渲染器引用。当前没有独立的compatibility／automation产品包。不要为预留AI引入通用事件总线、全局服务容器、微服务或插件引擎。

关键代码入口：

- [application入口](../crates/application/src/lib.rs)：`project_info`、`snapshot`、`validate`、`submit`；[contract](../crates/application/src/contract.rs)：`CommandRequest`／`ExpectedState`／结构化结果与错误。
- [草稿](../crates/application/src/draft.rs)、[历史导航](../crates/application/src/history_control.rs)、[查询](../crates/application/src/history_queries.rs)：只用公开重导出，不调用私有execute。
- [捕获](../crates/application/src/capture.rs)、[存储端口／DTO](../crates/application/src/archive.rs)、[恢复工厂](../crates/application/src/restore.rs)。
- [状态codec](../desktop/infrastructure/src/project_format/mod.rs)、[文件适配](../desktop/infrastructure/src/project_storage/mod.rs)、[恢复入口](../desktop/infrastructure/src/project_storage/recovery/mod.rs)。

原始Asset、图层实例、导出结果分别建模；共享字节不等于共享变换。图片组件和音效独立。预览／选择／监听音量／用户资料／设备／本机路径不属于项目内容。

## 5. 不可丢失的数据与历史约定

1. 首版文件必须含完整已捕获历史、全部分支、原始资源及未使用的项目素材，重开恢复身份和导航；不能只存当前头或创建新根替代旧树。
2. 已提交节点、父关系、原始操作和内容状态不可删除、覆盖、裁剪或合并掉。删除当前对象是追加新节点。容量不足拒绝新增，不回收历史仍可达资源。
3. 标签独立可变，不改节点身份／内容摘要；保存类标签依真实提交。六类标签为保存、曾保存标记、分支、当前、末端、起始。普通Redo记住Undo离开的直接子分支。
4. 手动检查点和最新保护位置分离；正常打开回到最近手动位置，用户可选最新保护。选择“不保存”退出仍须保护最后完成的编辑，但不推进手动位置。**该关闭调度尚未接入产品。**
5. 自动保护的2秒停顿／30秒最久等待／5秒间隔／100事务等是待验证建议，未启用，不是“最多丢30秒”的保证。正式实现须测试调度、提交范围和失败门控。
6. 当前头等于保存头，不代表新建后撤回的其他分支也已保存。完整历史和持久元数据都参与未保存判定。
7. 显示别名与永久身份分离；n≤14、b≤28字符，不存完整祖先地址。永久128位节点身份、k／hp1引用及address epoch按格式契约处理。API7的运行u64 NodeId仍存在，不把它解释成永久主键。
8. 地址重分配／分支合并尚未实现；当前恢复工厂要求根alias=1及追加编号。未来修改地址须显式契约／能力与失效测试，不能只改地址表。
9. Schema迁移追加表示并保留原始状态／操作与摘要，不重新编码覆盖旧节点；加载不重放旧Command。新打开epoch不得复活旧请求、草稿、Voice或GPU结果。
10. 历史固定原始字节，不固定全部RGBA／PCM。共享分配只计一次，任务／租约实际释放才退还额度；取消队列不等于内存已回收。

隐藏子图层／组只去掉贡献；整个元素／音效隐藏需要有效透明／静音输出，不能用缺失文件或零字节表示。未编辑的未知音频保留原字节。正式输出尚未实现，不能把当前试听格式限制当作导出时丢文件的理由。

## 6. P3-1C 的实际停靠点

版本分开记录，执行时再读代码／证明核对：

| 契约 | 当前值 |
| --- | --- |
| application API／模型 | 6／2 |
| 历史捕获／存储端口 | 2／1 |
| 文件profile | `skin-editor-rust-v1`，区别于P3-0 fixture |
| container／project schema／history schema | 2／1／2 |
| node record／内容state schema／恢复报告 | 2／1／1 |

工程扩展名为`.oskproj`，与皮肤导出`.osk`分开。SQLite bundled绑定为rusqlite 0.41.1，已测运行SQLite 3.54.1；不因应用版本升级自动更换数据含义。

文件路径：`capture_history → 完整捕获／原字节租约 → 编码／增量事务 → 真实回执`；恢复：`同一读事务内完整验证 → RestoreBuilder → RestoreCandidate → 新ProjectSession`。候选不覆盖现有活动会话。

新增 `SqliteProjectFile::recover_copy`：Windows只读独占冻结关闭来源，流式复制主文件与日志，SQLite只在物理副本回滚，验证全部分支／资源后不覆盖发布新目标。源文件、日志及旧节点身份／摘要保留。活跃来源Busy，WAL／super-journal／未知或损坏日志、非Windows冻结未实现时明确拒绝；不剪枝、不偷偷删除源日志。

重要结果区分：

- 提交前中断恢复此前完整文件代次；提交后回执丢失找回新代次。尚未落盘的内存编辑不在文件恢复保证内。
- 发布前失败／取消通常不留下目标；发布后返回`OutcomeUnknown`时目标可能已完整可用，须重开核对，不能盲目覆盖。
- `RecoveryCleanup { token, cause }`保留诊断残留；恢复进程强杀后的孤儿不自动删除或提升。资源句柄／租约仍须归还。
- `report.restored`是最新文件回执；Manual实际活动头看`candidate.header().head`，两者不能混为当前手动位置。
- 普通load遇非空sidecar返回`JournalPresent`；已有适配器guard先检查sidecar，避免SQL读取偷偷触发原位恢复。
- 恢复副本保留身份，不等于已完成SaveAs或身份分叉；自定义标签／扩展编辑当前明确Unsupported，不得静默丢弃请求。

已通过：16项恢复专项（含1个子进程入口）、真实提交前后／首次发布前后／恢复自身三个阶段进程终止、八阶段失败与取消、未选中分支损坏拒绝、20轮资源复用／释放、完整历史及10000层文件往返。`2719d0f`的正常hook再次通过418项Python工具测试、核心检查及桌面默认／全部feature；infrastructure两套各205项单元、存储集成各6项。

未证明：物理断电／扇区撕裂／目录项持久性、其他平台冻结及VFS、网络盘、真实磁盘满／只读介质、代表性大媒体恢复耗时与整进程峰值。默认文件／元数据／原字节等逻辑限额不等于产品CPU／内存预算通过。

### 动画演进的新增约定（未实现）

按[专题](ANIMATION_ASSETS.md)及[数据第2.1节](DATA_COMPATIBILITY.md)继续：

- 导入时识别单文件动画或显式图片序列，保留原字节／全部输入帧、原始逐帧时基／时长及来源；时长不只存平均FPS。PNG／AI帧图无原生时间时需明确创作设定。
- 原始动画、可编辑播放配方／图层实例、派生输出计划分开；帧不自动成为显隐图层，帧文件族仍属于一个元素输出。RGBA／GPU按需有界缓存，全部历史保管原件。
- 导出时按有证据的stable元素规则采样／合成／编码，兼容预览复用同一计划，原生素材预览另有标识。AnimationFramerate不是总帧数限制；全局／固定／BPM／事件规则和-1的一秒模式不能混为一谈。
- 默认尽量保时长／速度，通过持帧／取帧转换并报告量化／丢帧；严格要求无法满足则拒绝，不自动补帧、改全局配置或采用所有FPS最小公倍数。
- 影响输出的配方／绑定／配置／持久策略进入Command与历史；播放游标、预览模式和缓存不进入。取消、冻结、热替换、迟到帧及资源回收分别测试。
- 当前schema1／模型2不变，运行时API7；尚未扩展动画语义。实现前确定新增类型／内容语义／必要能力，扩展捕获与资源闭包，迁移追加表示并保留旧节点摘要；旧静态素材仍静态，未知动画不得变成首帧保存。
- D22及AN01–AN16全部待实施，F12静态图不能单独证明FPS。当前P3静态子集可继续，但动画导入／保存／输出的能力声明必须等待自身验收。

## 7. 继续工作的建议顺序

2026-11-10当前顺序：K1适用纯核心退出审阅已通过，用户已明确先补P3保存闭环。[P3保存／关闭／重开闭环规格](P3_SAVE_LOOP_SPEC.md)记录恢复基础复核、8项现状／缺口、四个实施增量和20项待实现回归；规格一致性修订已完成，下一步先备份并细化P3-2A开工契约，实施仍需具体任务。gameplay已有SyntheticTest实现，compatibility和K0-B／K2–K5仍另行推进；不把纯核心结果当作真实游玩或stable兼容。

本次已对照当前源码／实际回归复核P3-1C恢复边界。完整历史、非当前资源、持久身份和Windows副本恢复基础可复用；保存事实关联、关闭门控、UI身份重绑、旧导入隔离、未命名恢复转正式目标及容量相容证明仍缺。当前不启动新成本轮，其他平台原生实测继续单列，不阻塞独立Windows规格工作。

接正式保存前按新规格依次完成：无窗口保存协调和真实回执→真实文件／未命名恢复及验证复制→无窗口关闭／激活交接→普通宿主／UI。活动草稿Busy及单捕获槽与自动保护AP04／AP05的能力差异已明确；周期参数、标签事务、通用SaveAs／迁移、设置与导出分别验收。本轮不授权自动开发或接线。

动画数据的语义／拒绝边界已补入文件演进设计；开启动画前先补模型／codec／原时序与完整历史保存，再按元素编译输出。不要因专题存在就增加伪动画字段、解码依赖或完整创作时间轴，也不要把静态保存当作动画验收通过；实际实现仍以用户后续任务为准。

新对话不应重新做P2全部CPU归因：≤15%是播放优化目标，不再单独阻塞；零欠载、已就绪短音提交p95≤20ms、内存／GPU及回收门槛继续有效，空闲B05另有硬门槛。P2完成范围限已约定Windows负载，真实USB／第二输出端点变化后补；3.5mm模拟耳机和软件启停不替代该项。

## 8. 检查、证据与操作边界

使用README中的已验证入口：

```text
python -B tools/check_repository.py
python -B tools/check_core.py
python -B tools/check_desktop.py
```

Rust固定1.95.0。工具链不在PATH时按README使用`--toolchain-dir`或`SKIN_EDITOR_RUST_TOOLCHAIN_DIR`；本机已有忽略的`.p0-tools`缓存，不静默下载工具或修改全局版本。Windows需要已有MSVC／SDK；桌面默认检查离线、不开窗口／GPU／声卡／UAC。

正常hook包含完整核心与桌面检查，切换配置／重建缓存可能较久。可按既有方式只在当前进程设`CARGO_INCREMENTAL=0`限制缓存增长，不改产品策略或门槛，不跳过hook。仅文档修改先做链接／状态／仓库检查；按提交规则需要的hook仍正常运行。

恢复基线2719的本机hook日志在忽略的`artifacts/handoff-20261103/checkpoint-commit.log`，上一恢复日志在`artifacts/p3-1/20261103-recovery/`。本次发现并行会话已生成同内容交接提交6e26e9d，核对三文件及父提交后复用；本会话针对旧HEAD的重复提交检查已取消，记录在`artifacts/animation-design-20261103/checkpoint-adopted.json`，未把它记作完整通过。跨机器不保证本机日志存在；Git中的旧证明记录其证据时间点，不追改历史字段。

保留全部旧证明、黄金向量、归档和冻结二进制。不要因磁盘紧张删除证据或源素材；缓存清理须核对确切路径、无活动构建／受跟踪文件／重解析点，上传不会自动释放本地空间。不要批量读取私人截图、路径、凭据或谱面；外部stable证据使用指定F12截图，自有渲染用直接回读。

旧stable源码仅外部只读研究，不能复制、翻译移植或作为构建依赖。原版默认素材、用户皮肤／头像／歌曲不进Git或发行包；正式默认皮肤须自制或有分发授权。不得用lazer或占位场景代替stable兼容性证据。

## 9. 可粘贴的新对话开场

```text
项目位于 E:\skin-editor。先读根 AGENTS.md，核对 Git 状态、最近提交及暂存差异，按既定规则检查并备份已有可审查文档、推送 PUBLIC origin 并确认 GitHub 检查。
K1已在SyntheticTest纯核心范围通过独立退出审阅，Reference10000／Tiny4保持，产品性能NotEvaluated、stable兼容Unknown，不重测或扩展K1。
接着读 docs/P3_2A_START_CONTRACT.md、docs/P3_SAVE_LOOP_SPEC.md、docs/P3_1_RECOVERY_REVIEW.md、docs/DATA_COMPATIBILITY.md 及相关源码。P3-2A修订2的A1已实现，另读docs/P3_2A_A1_REVIEW.md；实际API7／捕获3／协调端口1、单槽真实准备及512KiB静态证明，PA仅适用部分通过。A2见证／候选、事实／Unknown及A3关闭仍未实施；B前真实证明／共同profile等未决项保留。本轮A1结束后停止，后续须用户明确恢复，不自行推进A2。
普通保存／关闭／重开闭环尚未实现；草稿捕获、手动排队、验证复制及容量相容不能假设已有。A1已完成本轮获授权范围，后续除非用户明确恢复，不开始A2、UI／设备或其他任务；皮肤skill暂停。旧证据与schema.rs既存空差异标记保留，不写回历史证明。
```
