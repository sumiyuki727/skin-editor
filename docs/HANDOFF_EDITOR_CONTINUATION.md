# 编辑器开发交接：从 P3-1C 继续

2026-11-18 B1-2L 新增版本 1 的只读游标。准备任务和协调执行者均能借出原票对应的捕获，文件读取器可消费一次并返回固定比对结果。下一步应先设计目标注册、物理身份及打开基线的只读资格，再接真实 Exit；同一捕获不能在这次比对后重新遍历以计算完整覆盖。[交接细节](P3_2B1_2L_CAPTURE_CURSOR.md)

## B1-2K 关闭资格

自制工程从手动保存点 A 打开，文件另有保护点 C。实际 Manual 捕获匹配 CurrentManual 和 VerifiedOpening。同一会话另行请求 Exit 关闭时，协调器先确认旧准备任务，再依次等待候选持有的工作和独立快照释放。最后一个所有者退出后，关闭返回 `Failed(Unavailable)`，未生成子保存或消耗下一保存身份。重复推进维持失败，普通编辑仍受门控。显式取消并确认后，下一次 Manual 获准，同一编辑请求也能成功。前后捕获的持久元数据和原文件字节相同。[关闭回归](../desktop/infrastructure/src/project_storage/tests/file_facts/profile/close.rs)

当前公开接口有两处待接通。关闭协调器创建 Exit 子任务前需要已安装的协议资格；现有合成签发器仅供 application 内部测试。即使有协议资格，执行端交付的是封装了捕获的 `SaveExecution`，文件读取器接收的是 `SavePreparationWork`。下一步应先冻结目标、打开基线和预算绑定的只读资格及执行端如何借用该捕获，再以关闭创建的原票核对 CurrentProtection、VerifiedOpening 和条件追加。共同 64 MiB、物理目标连续性和 Unknown 的事务阶段仍待验证。[B 开工契约](P3_2B_START_CONTRACT.md)

J 的分叉样例继续单独核对加载时推导的 Redo。只读判据可以匹配，而既有 writer 在自制副本上实际追加一代。生产保存事实仍须由获准的任务确认。本轮源码及文档随 Git 备份，新测试原始日志仅在本机保管。[J 回归记录](P3_2B1_2J_NO_CHANGE.md)

2026-11-18 B1-2J 验证了原基线的整树覆盖、历史检查点原始 Redo 与默认打开时推导 Redo 的区别。继续 B1 时应保留实际 writer 可能给出不同结果的回归，以及真实 Exit 协调和 Unknown 未闭合的边界。[交接记录](P3_2B1_2J_NO_CHANGE.md)

2026-11-18 B1-2I 接续 H 的真实来源，新增原基线全提交查重及普通追加条件摘要。实际 Manual Work 与旧 writer 的后验历史域 3 已分别核对。完整 B1／PB16 仍待收口，K1、产品性能与 stable 结论沿用原审阅。[交接细节](P3_2B1_2I_FRESH_APPEND.md)

2026-11-18 H远端收尾：[取消证据审阅](P3_2B1_2H_CI_REVIEW.md)记录2f4a525/run44167366330首次两核心成功／桌面cancelled；桌面两配置各310项、后续集成及文档通过，最后默认宿主build未完成，不能签三项CI成功。取消完整根因及旧各次原因仍Unknown；本次只补文档，不改源码／断言／构建步骤／预算／25分钟或30秒时限，不重跑旧SHA或宣称修复。新文档备份按正常完整hook和新SHA首次CI另验，旧证据／PRIVATE／schema.rs保持，原始新日志仅本机；H before能力及完整B1／PB16未退出、产品／stable结论不变。以下保留历史时点。

2026-11-18 B1-2H：[真实恢复来源与捕获已有metadata](P3_2B1_2H_REVIEW.md)已按[冻结契约](P3_2B1_2H_CONTRACT.md)及[独立向量](P3_2B1_2H_VECTORS.json)接通只读恢复→Session→Snapshot→真实Work/q来源链和before完整域3比较；无源／错绑／expected前进不补默认值，Missing与Unavailable先过完整守卫。旧F仍NotCompared，fresh-append／would_commit及保存资格不升级。Reader与A各512KiB独立，完整B1／PB16、NoChange／Unknown／物理连续性／共同64MiB仍欠；保持Experimental／Insufficient／BudgetUnproven及产品NotEvaluated／stable Unknown。旧证据、PRIVATE与schema.rs保留，原始新日志仅本机；UI／设备／Skill及现场步骤暂停。以下记录保留各自时点。

2026-11-18 G夹具事务修订：[CI收尾审阅](P3_2B1_2G_CI_REVIEW.md)记录075b204/run44125234390的两核心成功／桌面cancelled，不能签三项成功；默认和all-features各300项已过，取消根因仍Unknown。仅合并自制metadata输入构造为一次事务，保留所有旧／新用例、黄金、同步等级、预算与时限，另以正常完整hook及新SHA CI核验。G文件侧能力及q features缺口／F NotCompared不变，完整B1／PB16和NoChange／Unknown／物理绑定／共同预算仍欠；保持Experimental／Insufficient／BudgetUnproven，K1、产品NotEvaluated／stable Unknown和UI／设备／Skill限制不变。旧证明／PRIVATE／schema.rs保留，新原始日志仅本机。

2026-11-18 B1-2G：[文件侧历史metadata](P3_2B1_2G_REVIEW.md)已补文件侧历史post-commit域3、真实features根和五计数／扩展字节；八独立模型、124存储／92app及两观察+q512KiB边界已验。q的ArchiveMetadata／RestoreHeader／CaptureManifest无features，F仍NotCompared，完整历史q未完成。 保持Experimental／Insufficient／BudgetUnproven；K1／Reference10000／Tiny4、产品NotEvaluated／stable Unknown及UI／设备／Skill限制不变。

2026-11-17 B1-2F：[历史q多域关联](P3_2B1_2F_REVIEW.md)已将历史内容／原件／导航及选中提交请求证据统一关联到同一完整观察和实际q，保原票／expected。十行固定矩阵与四算术边界经独立审阅，不用compare_save原Ticket提交冒充selected证据。features／完整metadata及覆盖／Unknown／物理绑定／共同预算仍欠，不签完整B1。 保持Experimental／Insufficient／BudgetUnproven；K1／Reference10000／Tiny4、产品NotEvaluated／stable Unknown及UI／设备／Skill限制不变。

2026-11-17最新停靠点：[B1-2E历史原件摘要](P3_2B1_2E_REVIEW.md)完成独立只读集合／域2摘要与真实q原件对照；明确SHA前移位号变化、空集合／零字节、同量异字节和完整来源隔离，结论恒Insufficient。定向108存储／90app及Clippy通过，仍须正常完整hook／新SHA三项CI。完整历史q多域关联、features／metadata、NoChange／Unknown／物理绑定与共同预算仍欠，不签完整B1／PB16、不进入B2；旧38c0c915作回退点，schema.rs／旧证据／PRIVATE保持，新日志仅本机。UI／Skill及现场步骤仍暂停，K1结论不变。

2026-11-17最新停靠点：[B1-2D严格时序准入](P3_2B1_2D_REVIEW.md)已补实验观察每代状态原件闭包与声明精确相等，入队即去重、原state根、共享／未使用／后删及旧分支均覆盖。定向101项与Clippy通过，仍须本轮正常完整hook／新SHA三项CI。无历史原件公开摘要，完整历史q、features／metadata、NoChange／Unknown／物理绑定与共同预算继续欠证；完整B1／PB16不签收，不进入B2。API7／捕获3／协调1及K1结论保持，旧2e869da6为回退点；schema.rs／旧证据／PRIVATE不动，新日志仅本机，UI／Skill和现场步骤仍暂停。

2026-11-17最新停靠点：[B1-2C历史节点内容域](P3_2B1_2C_REVIEW.md)已实现固定提交前缀的节点／admissions域1查询及真实q的内容／头／导航比较，外层保留完整观察／文件请求／选中提交和原Ticket。新增8组回归、存储定向92项通过，仍须本轮完整hook和新SHA三项CI。明确不含历史originals，旧full的逐代原件声明≠状态闭包缺证已登记；features／完整metadata、NoChange覆盖、Unknown连续性与共同预算仍欠，B1／PB16不签收，不进入B2／产品保存。原3c52757及三CI作轮前回退点；本片源码／文档另收口，旧证据／PRIVATE／schema.rs原字节保留，新日志仅本机。用户离开，现场UAC／截图／交互等待，UI／Skill暂停；其他已授权无窗口工作可接续。

2026-11-17最新停靠点：[B1-2B真实Work与只读意图对照](P3_2B1_2B_REVIEW.md)已实现原SaveTicket／expected保留、legacy与独立generation、q／打开／latest导航及ArchiveMetadata-only前／假设提交后投影；新增Work回归12项与门控诊断2项，存储定向84项通过。结果仍ExperimentalValidation／Insufficient／BudgetUnproven，B1／PB16未签收。后续继续已授权B1分片补features／历史完整q／NoChange／Unknown连续性与完整预算，不进入B2写入、产品保存或恢复部署。API7／捕获3／协调1、K1／Reference10000／Tiny4及性能／stable结论保持。轮前1a35784最终三项CI通过但首desktop尝试失败根因未明；新诊断不放宽30秒或精确READY。本片另经正常hook、新提交推送及同SHA三项CI；旧证据／PRIVATE材料／schema.rs原字节保留，新增日志仅本机，不自动纳入旧恢复快照。

2026-11-17 B1-2A：[冻结捕获投影与原始量预检](P3_2B1_2A_REVIEW.md)完成真实q的content／originals两域规范化、同观察完整来源比较、UTF-8及固定17项原始量预检。原18＋新增24专项通过，结果始终ExperimentalValidation／Insufficient；原Ticket、metadata／导航／完整Unknown与64MiB／SQLite／临时盘仍欠证，完整B1／PB16未签收。下一步继续B1证据收口，不进入B2写入或产品保存。API7／捕获3／协调1及旧格式保持，U1–U3可调默认和K1／Reference10000／Tiny4／产品NotEvaluated／stable Unknown不变。以下日期段保留历史时点。

2026-11-17当前停靠点：[B1-1内部实验增量](P3_2B1_REVIEW.md)完成FileFactsPort1、独立FileCoord-Proof-v1规范向量和Windows健康既有文件只读验证；18项专项、两路独立审查及新提交正常hook／同SHA CI分别记录。结果不授保存／激活资格，q对照、完整64MiB聚合与SQLite临时磁盘仍待B1-2，完整B1／PB16未签收。U1系统持久恢复子目录可调整迁移、U2初始4GiB／32条可调整满额拒绝并警告且不清理、U3同身份完整历史首次命名用户选位已获确认；覆盖／迁移／清理协议后续另约，B1不部署目录／设置UI。API7／捕获3／协调1及旧文件版本保持。轮前00492b1三CI通过，新源码以本报告所在提交的新CI为准；schema.rs、旧证明及PRIVATE材料保全。后续先补B1-2证据，不提前B2写入、GUI／设备／Skill或升级K1／产品／stable结论。以下“U1–U3待决定”等是此前时点记录，以总契约第9节及本段为准。

2026-11-17恢复停靠点：[P3-2B审阅修订2](P3_2B_START_CONTRACT.md)已将上轮初审9组待办合入实际条款与PB验收，两路针对性独立复核通过；本轮另按链接／一致性、正常hook及新提交同SHA CI收口，准确提交与检查结果以Git回执为准。当前工作范围仅8份文档，B实现仍未开始。U1持久恢复根、U2总配额／条目数、U3首次命名身份尚待用户决定；可准备不依赖这些选择的接口资料，不能越过政策与B1开工门槛。推荐系统持久数据目录、4GiB／32条目及连续命名都不是已批准配置。schema.rs、旧证明和PRIVATE材料保持；源码Git与新本机日志保管仍分开。下方2026-11-16停止／草案段保留当时事实，以本段及最新Git回执为准。

2026-11-16收尾停止：用户要求因额度耗尽结束当前工作。P3-2B草案及7份同步文档保留本地未提交，暂存区为空；两路独立初审已完成，待合入修订列在[B契约第10节](P3_2B_START_CONTRACT.md#10-审阅开工与保管门槛)。本轮尚未运行完整hook、创建新提交、推送或新CI；最近已推送并三项CI成功的回退点仍为e1934b881b81832051576909a709722747782eb6。需用户明确恢复后，先核对状态、合入待修订项，再继续文档检查／正常提交流程；不自动开始B实现或下一任务。schema.rs原字节／既存标记和私人恢复材料保留。

2026-11-16最新停靠点：[P3-2B开工契约草案](P3_2B_START_CONTRACT.md)已从A退出及现有SQLite／恢复实现细化B1→B2a／B2b→B3→B4，PB01–PB32全部待实现。B必须先完整准备／保管见证再写，新增真正只读事实入口，按实际提交阶段处理Unknown，实际连接退出先于A执行守卫；恢复文件提交与可发现性分离。U1恢复根、U2总额／槽数、U3首次命名身份待用户决定，推荐方案不是已批准配置；随后冻结B1接口／proof向量和容量资料，再另行进入功能开发。本轮只文档，无真实文件端口／GUI／设备／皮肤Skill。轮前e1934b8及[三项CI](https://github.com/sumiyuki727/skin-editor/actions/runs/43734186283)成功，A限定范围已签收，新契约按自身提交及同SHA CI保管。K1／Reference10000／Tiny4、产品NotEvaluated／stable Unknown保持；旧证据与PRIVATE材料不改，新本机日志不属于旧固定快照。下方段落保留历史时点。

2026-11-16最新停靠点：[P3-2A整体退出审阅](P3_2A_EXIT_REVIEW.md)已逐项对齐PA01–PA38，[证明](P3_2A_EXIT_PROOF.json)记录原78项与4项新补证、源码散列及准确执行身份。应用层SaveCoord-Fixture-v1范围支持签收，最终核对本报告所在提交正常hook、已推送完整SHA及三项CI全部成功；不套用轮前0ae8003的[成功CI](https://github.com/sumiyuki727/skin-editor/actions/runs/43725492883)为新测试。独立审阅未发现生产阻断项，本轮不改生产实现。下一步先定约B：真实证明／只读核对、持久恢复区与配额、发布／发现中断、转正身份关系和共同容量；不直接接文件、GUI或设备。A的保护满足不授权关窗／激活。K1／Reference10000／Tiny4及产品NotEvaluated／stable Unknown保持。PUBLIC源码／PRIVATE历史恢复材料安排不变；本机新日志不自动属于3435405快照。下方日期段和早期章节保留历史时点，以本段与最新Git记录为准。

2026-11-15最新停靠点：[P3-2A A3实现与独立审查](P3_2A_A3_REVIEW.md)已完成纯用例关闭门控、既有草稿Permit收尾、旧任务结清／实际释放、最终q、明确重试和取消归属；26项新增关闭回归通过。保护满足不是关窗／新建／打开／重开许可，生产authority和真实文件端口仍未接。API7／捕获3／协调1及512KiB预算保持，K1／产品／stable结论不变。轮前复用已推送8943286及[同SHA三项CI](https://github.com/sumiyuki727/skin-editor/actions/runs/43627066529)，本增量另按正常hook和同SHA CI收口。后续先按A1–A3证据核对纯用例退出边界；B真实证明／恢复区／配额等须先定约，不自动接文件或GUI。用户已授权接续A3及独立审查，无需重问A3授权。下方日期段保留历史时点。

2026-11-15最新停靠点：[P3-2A A2实现与证明](P3_2A_A2_REVIEW.md)已完成事实投影、PreparedWitness／确认候选、NoChange／Unknown、独立核对及实际释放；28项专项、原A1回归和application既有测试保持通过。生产authority没有签发器，fake只在cfg(test)，不能宣称已接真实保存。API7／捕获3／协调1及文件格式不变，A3／B–D仍欠项。轮前A1提交abd879dc6dd03d6d1ea46ce722ea0592d436b7f9及[同SHA三项CI](https://github.com/sumiyuki727/skin-editor/actions/runs/43615335146)通过；本轮收尾核对包含A2报告的Git提交及其CI，不借用A1结果。旧证据、PRIVATE恢复材料和schema.rs原字节保持，3435405固定快照不自动延伸。后续先审阅A2，不自动进入A3；下方日期段保留历史时点。

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
接着读 docs/P3_2A_START_CONTRACT.md、docs/P3_SAVE_LOOP_SPEC.md、docs/P3_1_RECOVERY_REVIEW.md、docs/DATA_COMPATIBILITY.md 及相关源码。P3-2A修订2的A1已实现，另读docs/P3_2A_A1_REVIEW.md；实际API7／捕获3／协调端口1、单槽真实准备及512KiB静态证明，PA仅适用部分通过。A2见证／候选、事实／Unknown已按用户直接确认实施，另读docs/P3_2A_A2_REVIEW.md；生产authority未签发，A3关闭及B前真实证明／共同profile仍欠。后续先审阅A2，不自动推进A3。
普通保存／关闭／重开闭环尚未实现；草稿捕获、手动排队、验证复制及容量相容不能假设已有。A2已完成本轮获授权范围，后续先审阅，不自动开始A3、UI／设备或其他任务；皮肤skill暂停。旧证据与schema.rs既存空差异标记保留，不写回历史证明。
```
