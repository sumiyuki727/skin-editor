# 文档索引

2026-11-19 B1-2AA 记录首次解码后如何借用 SQL 列并核对项目 ID，附该阶段容量与失败释放回归。[SQL 派生字段与项目 ID](P3_2B1_2AA_BLOCK_DERIVED_COLUMNS.md)

2026-11-19 B1-2Z 记录首次加载块时，已缓存块、当前原文、解析树和两份摘要输出的分配与释放。[首次块解析分阶段证明](P3_2B1_2Z_FIRST_BLOCK_DECODE.md)

2026-11-19 B1-2Y 核对了 Small 元数据索引根、页目录和读取时已加载的块表，并记录坏目录的错误顺序。[索引根与常驻块表证明](P3_2B1_2Y_INDEX_ROOT_BLOCK_CACHE.md)

2026-11-19 B1-2X 记录 Small 检查点元数据页如何释放嵌套容器、保留首错和摘要，以及各页实际同时持有的请求字节。[逐页释放与 52 MiB 分项证明](P3_2B1_2X_METADATA_ROW_FIELDS.md)

2026-11-19 B1-2W 核对 Small 元数据 Map 的插入、分裂、析构和重复页共享，并给出不同页的拒绝路径下界。[Map 节点与页所有权](P3_2B1_2W_JSON_MAP_NODES.md)

2026-11-19 B1-2V 记录 Small 解析中对象键的借用与移交、转义和长浮点共用的字节缓冲，以及两种转换的合并上界。[文本与解码缓冲](P3_2B1_2V_JSON_TEXT_SCRATCH.md)

2026-11-19 B1-2U 记录未闭合数组、已完成子树和单次扩缩容的并存界限，以及与已完成页的合并方法。[解析中数组的容量](P3_2B1_2U_JSON_ARRAY_GROWTH.md)

2026-11-19 B1-2T 给出 Small 已完成历史元数据页的数组和文本容量公式，包含嵌套内容、对象键与重复页。[JSON 页容量证明](P3_2B1_2T_METADATA_JSON_CAPACITY.md)

2026-11-19 B1-2S 记录单代三类历史元数据的页所有权、实际槽容量公式、宽档读取及首错回归。[分页读取与容量](P3_2B1_2S_METADATA_PAGES.md)

2026-11-19 [B1-2R 历史元数据页的持有期与末代比对](P3_2B1_2R_METADATA_BUDGET.md)记录跨代释放、逐项比较、Small 去重工作值与单代 DOM 的剩余成本。

2026-11-19 [B1-2Q 完整恢复的解码输入与同时存活的分配](P3_2B1_2Q_DECODE_BUDGET.md)记录每节点借用、固定索引及 Reader 工作租约的适用阶段，并列出 SQLite、媒体、历史页和恢复模型的证据缺口。

2026-11-18 [B1-2P 初始恢复会话与原打开文件的只读核对](P3_2B1_2P_SESSION_PREFLIGHT.md)记录安装前配对、累计解码额度和生产关闭剩余门槛。

2026-11-18 [B1-2O 登记期间的单次捕获覆盖核对](P3_2B1_2O_REGISTERED_COVERAGE.md)记录原父票、完整覆盖、完成后取消及两端各自的验收范围。

2026-11-18 [B1-2N 原打开文件的只读登记](P3_2B1_2N_REGISTERED_OPENING.md)记录注册者身份、条件重绑、持续持有的物理句柄及释放边界。

2026-11-18 [B1-2M 原打开目标的只读观察](P3_2B1_2M_OPENING_TARGET.md)记录名称、瞬时文件标识与打开基线的独立比较及真实 Exit 的剩余条件。

2026-11-18 [B1-2L 保存捕获只读游标](P3_2B1_2L_CAPTURE_CURSOR.md)说明原票与文件观察的比对、遍历位置、取消及释放行为。

2026-11-18 [B1-2J 的只读覆盖对照](P3_2B1_2J_NO_CHANGE.md)记录三类判据、真实 Manual 请求，以及默认打开和实际写入分歧的回归。

2026-11-18 [B1-2I 普通追加元数据](P3_2B1_2I_FRESH_APPEND.md)记录原基线查重、只读条件摘要及独立 writer 对照。

2026-11-18 H远端收尾：[取消证据审阅](P3_2B1_2H_CI_REVIEW.md)记录2f4a525/run44167366330首次两核心成功／桌面cancelled；桌面两配置各310项、后续集成及文档通过，最后默认宿主build未完成，不能签三项CI成功。取消完整根因及旧各次原因仍Unknown；本次只补文档，不改源码／断言／构建步骤／预算／25分钟或30秒时限，不重跑旧SHA或宣称修复。新文档备份按正常完整hook和新SHA首次CI另验，旧证据／PRIVATE／schema.rs保持，原始新日志仅本机；H before能力及完整B1／PB16未退出、产品／stable结论不变。以下保留历史时点。

2026-11-18 B1-2H：[真实恢复来源与捕获已有metadata](P3_2B1_2H_REVIEW.md)已按[冻结契约](P3_2B1_2H_CONTRACT.md)及[独立向量](P3_2B1_2H_VECTORS.json)接通只读恢复→Session→Snapshot→真实Work/q来源链和before完整域3比较；无源／错绑／expected前进不补默认值，Missing与Unavailable先过完整守卫。旧F仍NotCompared，fresh-append／would_commit及保存资格不升级。Reader与A各512KiB独立，完整B1／PB16、NoChange／Unknown／物理连续性／共同64MiB仍欠；保持Experimental／Insufficient／BudgetUnproven及产品NotEvaluated／stable Unknown。旧证据、PRIVATE与schema.rs保留，原始新日志仅本机；UI／设备／Skill及现场步骤暂停。以下记录保留各自时点。

2026-11-18 G夹具事务修订：[CI收尾审阅](P3_2B1_2G_CI_REVIEW.md)记录075b204/run44125234390的两核心成功／桌面cancelled，不能签三项成功；默认和all-features各300项已过，取消根因仍Unknown。仅合并自制metadata输入构造为一次事务，保留所有旧／新用例、黄金、同步等级、预算与时限，另以正常完整hook及新SHA CI核验。G文件侧能力及q features缺口／F NotCompared不变，完整B1／PB16和NoChange／Unknown／物理绑定／共同预算仍欠；保持Experimental／Insufficient／BudgetUnproven，K1、产品NotEvaluated／stable Unknown和UI／设备／Skill限制不变。旧证明／PRIVATE／schema.rs保留，新原始日志仅本机。

2026-11-18 B1-2G：[文件侧历史metadata](P3_2B1_2G_REVIEW.md)本轮入口为[契约](P3_2B1_2G_CONTRACT.md)、[八组独立域3模型](P3_2B1_2G_VECTORS.json)、[审阅](P3_2B1_2G_REVIEW.md)、[结构化证据](P3_2B1_2G_PROOF.json)。文件侧材料补齐不等于真实q或保存资格补齐。 保持Experimental／Insufficient／BudgetUnproven；K1／Reference10000／Tiny4、产品NotEvaluated／stable Unknown及UI／设备／Skill限制不变。

2026-11-17 B1-2F：[历史q多域关联](P3_2B1_2F_REVIEW.md)本轮入口为[契约](P3_2B1_2F_CONTRACT.md)、[固定语义矩阵](P3_2B1_2F_MATRIX.json)、[审阅](P3_2B1_2F_REVIEW.md)及[结构化证据](P3_2B1_2F_PROOF.json)。已比历史域只读关联，不授予保存资格。 保持Experimental／Insufficient／BudgetUnproven；K1／Reference10000／Tiny4、产品NotEvaluated／stable Unknown及UI／设备／Skill限制不变。

2026-11-17最新增量：[B1-2E契约](P3_2B1_2E_CONTRACT.md)、[实现审阅](P3_2B1_2E_REVIEW.md)、[七组独立域2模型](P3_2B1_2E_VECTORS.json)和[结构化证据](P3_2B1_2E_PROOF.json)记录历史原件集合／摘要与真实q对照，仍仅Experimental／Insufficient／BudgetUnproven，不增加保存资格。完整B1／PB16及共同预算仍欠。

2026-11-17最新增量：[B1-2D契约](P3_2B1_2D_CONTRACT.md)、[实现审阅](P3_2B1_2D_REVIEW.md)、[四组独立模型](P3_2B1_2D_VECTORS.json)及[结构化证据](P3_2B1_2D_PROOF.json)记录实验观察的历史原件时序严格准入，不新增公开摘要或保存资格。完整B1／PB16及共同预算继续欠证，保持Experimental／Insufficient／BudgetUnproven。

2026-11-17最新增量：[B1-2C契约](P3_2B1_2C_CONTRACT.md)、[实现审阅](P3_2B1_2C_REVIEW.md)、[三组独立向量](P3_2B1_2C_VECTORS.json)及[结构化证据](P3_2B1_2C_PROOF.json)记录历史节点／admissions单域只读查询与真实q比较，不含历史原件或保存资格。完整B1／PB16及共同预算仍欠证，保持Experimental／Insufficient／BudgetUnproven。

2026-11-17最新增量：[B1-2B契约](P3_2B1_2B_CONTRACT.md)、[实现审阅](P3_2B1_2B_REVIEW.md)、[独立向量](P3_2B1_2B_VECTORS.json)和[结构化证据](P3_2B1_2B_PROOF.json)记录真实Work／原Ticket与只读请求、导航和ArchiveMetadata-only对照；保持Experimental／Insufficient／BudgetUnproven。完整B1／PB16仍欠证，不进入B2或产品保存。

当前总体进度、路线建议和检查结果先读[2026-11-09全面检查](PROJECT_REVIEW_20261109.md)，再按[架构](ARCHITECTURE.md)、[技术栈](TECH_STACK.md)和[README](../README.md)查对应实现。[代码与决策包](review/README.md)是2026-10-30审查快照，后续已有P3状态／捕获／文件恢复及K1增量，不能将原清单当当前全量代码。独立存储基础仍不等于普通保存闭环，历史测量也不覆盖当前版本的新路径。

## 当前阅读入口

- [B1-2A冻结捕获两域比较与资源预检](P3_2B1_2A_REVIEW.md)／[开工契约及预算台账](P3_2B1_2A_READONLY_CONTRACT.md)／[独立投影向量](P3_2B1_2A_CAPTURE_VECTORS.json)（新增24项＋原18项；仍Experimental／Insufficient，完整B1／PB16与生产保存未签收）

- [B1-1健康既有文件的实验性只读事实](P3_2B1_REVIEW.md)／[接口与proof规范](P3_2B1_FILE_FACTS.md)／[独立向量](P3_2B1_PROOF_VECTORS.json)（18项Windows专项；政策可调默认已确认，完整B1／生产共同预算／q对照仍欠项，无写入／UI）

- [P3-2B真实存储衔接开工契约](P3_2B_START_CONTRACT.md)（2026-11-17审阅修订2，BR01–BR09已合入；政策已确认，当前B1-1实验见上；32组整体验收仍未签收，真实写入／恢复部署后置）

- [P3-2A整体退出审阅](P3_2A_EXIT_REVIEW.md)／[38组证据映射](P3_2A_EXIT_PROOF.json)（82项专项；A限SaveCoord-Fixture-v1应用层签收，e1934b8同SHA三项CI已成功；B契约见上，真实保存／重开仍待B–D）

- [P3-2A A3关闭门控与最终保护](P3_2A_A3_REVIEW.md)（26项新增回归、独立审查与固定槽证明；保护满足不授权真实关闭，B–D仍欠项）

- [P3-2A A2事实、见证与实际释放](P3_2A_A2_REVIEW.md)（28项专项、候选capacity和独立核对；仅无窗口测试profile，生产资格／文件端口未接；A2历史报告；当前整体退出见上，B–D待续）

- [skin.ini可视化布局编辑：拖拽与缩放](SKIN_INI_VISUAL_EDITING.md)（2026-11-14一致性修订2、10组发现；不限mania，V0–V3／VL01–VL22，全部待实施；主线仍先P3保存闭环，V0真实参数表及具体开工限额仍待冻结）
- [P3-2A A1实现与静态容量证明](P3_2A_A1_REVIEW.md)（API7／捕获3／协调端口1、八固定控制分区、单槽真实捕获准备及回归；A1历史报告；当前整体退出见上，仍无真实文件保存／关闭）
- [P3-2A无窗口保存协调开工契约](P3_2A_START_CONTRACT.md)（一致性修订2；PA01–PA38适用A部分已有退出证据，修订2要求保持；真实适配／宿主／GUI仍欠B–D）
- [P3恢复基础复核与保存／关闭／重开闭环规格](P3_SAVE_LOOP_SPEC.md)（一致性修订2、8组发现、四增量／20项分层验收；A已审阅开工契约另见上，B–D仍欠项，尚未接产品保存／UI）
- [2026-11-09项目全面检查](PROJECT_REVIEW_20261109.md)
- [K1独立退出审阅与签收结论](K1_EXIT_REVIEW.md#2026-11-10独立退出审阅与签收结论)（正式结果已备份b6bc96d、三项CI成功；K1适用SyntheticTest纯核心范围可签收，合法Unknown保留，产品NotEvaluated／stable Unknown）
- [统一时基PC128新轮与只读复核](K1_EXIT_REVIEW.md#2026-11-10统一时基正式成本轮与只读复核)（完整12例Valid／ReadyForReview及全部原始证据保持，退出决定独立记录于上项）
- [父端统一时基修正与审阅](K1_COST_CLOSURE_SPEC.md#2026-11-10父端统一时基修正与审阅)（修正已保存3310e49并通过CI；寿命与CSV统一perf_counter，旧失败不追认，新正式证据见上）
- [旧PC128正式成本轮与K1独立退出审阅](K1_EXIT_REVIEW.md#2026-11-10pc128正式成本轮与退出审阅)（8043028旧轮因父端时基跨度矛盾Invalid／NeedsEvidence；失败原件保留，与后续成功轮分开解释）
- [完整CostClosure-v1正式成本与退出决定](K1_EXIT_REVIEW.md#2026-11-09完整成本新轮与退出决定)（结果已保存814de56；12进程协议及保管Valid，9个纯取消序列不足分辨，NeedsEvidence，K1仍暂不签收）
- [K1整体退出审阅／成本补证](K1_EXIT_REVIEW.md)（含历史缺口、历次失败与最新独立退出决定；不将历史NeedsEvidence或Invalid追认为通过）
- [K1-6D工具一致性审阅及v2重测](K1_6D_COST_REVIEW.md#2026-11-08一致性审阅记录)（初版4994d2e、一致性修订2c387f2已备份；453项Python／169项gameplay／6项工具Rust通过；历史v2-01协议Valid／指标Unknown／产品NotEvaluated，整体退出结论及新轮证据见上）
- [K1-6C规模回归及一致性审阅](K1_6C_SCALE_REVIEW.md#2026-11-07一致性审阅记录)（C初版备份09075738、一致性修订备份13fb466；169项gameplay通过，API7／profile1、Reference10000／Tiny4保持；后续D工具及基线见上）
- [K1-6B变长存储／容量证明／分步工作及一致性审阅](K1_6B_STORAGE_PROOF.md#2026-11-06一致性审阅记录)（API7／profile1，Tiny4／Reference10000；B阶段156项回归与容量证明，后续C规模复核及D工具基线见上）
- [独立stable兼容内核架构／K计划](STABLE_KERNEL_PLAN.md) · [专用新对话交接](HANDOFF_STABLE_KERNEL.md) · [K1-1合成圆圈模型](K1_1_SYNTHETIC_REVIEW.md) · [K1-2冻结／恢复／重新武装](K1_2_FREEZE_REVIEW.md) · [K1-3同谱面检查定位](K1_3_INSPECTION_REVIEW.md) · [K1-4统一候选准备／激活](K1_4_PREPARATION_REVIEW.md) · [K1-5自动输入／驱动互斥](K1_5_DRIVER_REVIEW.md) · [K1-6A固定四槽／只读截面及一致性审阅](K1_6A_STORAGE_REVIEW.md)（API6、4对象无窗口，137项gameplay通过，K1部分完成；未接GUI／设备，不宣称stable兼容）
- [K1-6规模／资源成本规格及文档审阅](K1_6_SCALE_SPEC.md)（A–D、PC128及父端时基修正已有实现／审阅／测量；适用K1退出审阅通过，产品及兼容性结论保持）
- [编辑器新对话交接](HANDOFF_EDITOR_CONTINUATION.md) · [素材制作／未来编辑器操作skill交接](HANDOFF_SKIN_ASSET_SKILL.md)（2026-11-03快照，含可粘贴开场提示）
- [动画来源／原始帧时长／stable重采样](ANIMATION_ASSETS.md)（设计已记录；模型、版本和功能未改；AN01–AN16全部待实施）
- [P3-1C 热日志／进程中断恢复补充](P3_1_RECOVERY_REVIEW.md) · [独立证据](P3_1_RECOVERY_PROOF.json)
- [P3-1C Rust存储／完整恢复候选](P3_1_STORAGE_REVIEW.md) · [本轮证据](P3_1_STORAGE_PROOF.json)
- [GitHub 与空间整理规则](GIT_WORKFLOW.md)
- [Git与加密证据附件恢复](PROJECT_RECOVERY.md)（2026-11-11用户要求结束时全部提交推送并备份原始材料；恢复范围、独立密钥和远端下载／解密核验，完成状态见发布回执）

- [P3-1B完整捕获／身份／租约](P3_1_CAPTURE_REVIEW.md) · [本轮证据](P3_1_CAPTURE_PROOF.json)

- [P3-1A实际状态codec](P3_1_STATE_REVIEW.md) · [schema 1契约](PROJECT_STATE_SCHEMA.md) · [检查证据](P3_1_STATE_PROOF.json)
- [P3-0最新审阅：地址、Schema与加载](P3_0_LOAD_REVIEW.md) · [本轮证明](P3_0_LOAD_PROOF.json)
- [P3-0审阅包](P3_0_REVIEW.md) · [身份／文件格式v1](PROJECT_FORMAT_V1.md) · [原型证据](P3_0_PROOF.json)
- [系统对象／方法与引用](review/SYSTEM_MAP.md) · [完整清单](review/README.md#全量代码清单) · [待讨论决定](review/DECISIONS.md)
- [用户数据与长期兼容](DATA_COMPATIBILITY.md)：首版完整历史已确认；P3-0身份／容器已定稿供审阅，偏好与正式迁移尚未实现
- [自动保护周期与触发](AUTOSAVE_POLICY.md)：停顿／最久等待／限流、队列与失败处理；历史不可删改、标签独立可变，参数仍待实施验证
- [命令](COMMANDS.md) · [草稿](DRAFTS.md) · [地址](HISTORY_ADDRESSING.md) · [标签／导航](HISTORY_PRESENTATION.md)
- [呈现交接](PRESENTATION_CONTRACT.md) · [输入](EDITOR_INPUT.md) · [组件](COMPONENT_MODEL.md) · [音频设计](AUDIO_EDITING_REVIEW.md)
- [需求](REQUIREMENTS.md) · [路线](ROADMAP.md) · [资源预算](RESOURCE_BUDGET.md) · [后续设置](PREVIEW_SETTINGS.md) · [时间轴](PREVIEW_TRANSPORT.md) · [在线素材](ONLINE_ASSETS.md)
- [逐文档审查](review/DOCUMENT_AUDIT.md) · [17份审阅前原文](archive/2026-10-30-pre-review/README.md) · [历史实验](../experiments/README.md)

## 历史证据与专题（逐项保留）

以下按用途区分，不将固定时点的候选／待验状态当作当前事实。配对JSON保留原路径和字节；不是当前项目文件schema。

| 文档 | 阅读身份 |
| --- | --- |
| [ARCHITECTURE.md](ARCHITECTURE.md) | 当前契约／入口 |
| [STABLE_KERNEL_PLAN.md](STABLE_KERNEL_PLAN.md) | 独立兼容内核设计、时间／输入／生命周期、模块交接与K／KT待验；不是已实现玩法 |
| [HANDOFF_STABLE_KERNEL.md](HANDOFF_STABLE_KERNEL.md) | 新对话接手内核规划的范围、代码入口、风险、检查及可粘贴任务 |
| [AUDIO_EDITING_REVIEW.md](AUDIO_EDITING_REVIEW.md) | 当前契约／入口 |
| [ANIMATION_ASSETS.md](ANIMATION_ASSETS.md) | 动画导入、精确原时序、元素输出计划及后续AN验收；尚未实现，不能将静态PNG／分页sequence视作动画 |
| [SKIN_INI_VISUAL_EDITING.md](SKIN_INI_VISUAL_EDITING.md) | 所有适用skin.ini项的可视化编辑需求，当前仅拖拽／缩放的映射、草稿、来源与文件闭环规格；尚未实现 |
| [COMMANDS.md](COMMANDS.md) | 当前契约／入口 |
| [COMPONENT_MODEL.md](COMPONENT_MODEL.md) | 当前契约／入口 |
| [CORE_COST_BASELINE.md](CORE_COST_BASELINE.md) | P1阶段记录 |
| [DRAFTS.md](DRAFTS.md) | 当前契约／入口 |
| [DATA_COMPATIBILITY.md](DATA_COMPATIBILITY.md) | 2026-11-01数据契约设计；明确已确认要求与未冻结方案，不表示已实现 |
| [AUTOSAVE_POLICY.md](AUTOSAVE_POLICY.md) | 2026-11-02调度设计；补充历史不可删改与标签例外，参数和AP计划仍待实现验证，不是已启用周期 |
| [EDITOR_INPUT.md](EDITOR_INPUT.md) | 当前契约／入口 |
| [HISTORY_ADDRESSING.md](HISTORY_ADDRESSING.md) | 当前契约／入口 |
| [HISTORY_PRESENTATION.md](HISTORY_PRESENTATION.md) | 现行专题契约 |
| [ONLINE_ASSETS.md](ONLINE_ASSETS.md) | 未来设计／部分接线 |
| [PROJECT_STATE_SCHEMA.md](PROJECT_STATE_SCHEMA.md) | P3-1A已实现的内容状态schema；不是完整文件或历史格式 |
| [P3_1_STORAGE_REVIEW.md](P3_1_STORAGE_REVIEW.md) | Rust增量文件、完整历史验证与脱离活动会话恢复；未接正式保存／打开UI |
| [P3_1_RECOVERY_REVIEW.md](P3_1_RECOVERY_REVIEW.md) | Windows原件保留／副本热日志恢复及真实进程中断验证；其他平台／物理断电／正式UI仍未测或未接 |
| [GIT_WORKFLOW.md](GIT_WORKFLOW.md) | 私有远端、检查／提交／同步、附件和缓存整理规则 |
| [P3_1_STATE_REVIEW.md](P3_1_STATE_REVIEW.md) | P3-1A实现、对象／方法与后续范围 |
| [P3_1_CAPTURE_REVIEW.md](P3_1_CAPTURE_REVIEW.md) | P3-1B完整历史捕获、永久身份、准入／取消与实际原字节释放；尚无正式文件恢复 |
| [P1_PLAN.md](P1_PLAN.md) | P1阶段记录 |
| [P1_PROGRESS.md](P1_PROGRESS.md) | 当前契约／入口 |
| [P2_ACTIVE_EDIT_FAULTS.md](P2_ACTIVE_EDIT_FAULTS.md) | 历史增量／测量 |
| [P2_AUDIO_CORE.md](P2_AUDIO_CORE.md) | 历史增量／测量 |
| [P2_AUDIO_EDIT.md](P2_AUDIO_EDIT.md) | 历史增量／测量 |
| [P2_AUDIO_LIBRARY.md](P2_AUDIO_LIBRARY.md) | 历史增量／测量 |
| [P2_AUDIO_LOAD.md](P2_AUDIO_LOAD.md) | 历史增量／测量 |
| [P2_AUDIO_NATIVE.md](P2_AUDIO_NATIVE.md) | 历史增量／测量 |
| [P2_AUDIO_RESOURCES.md](P2_AUDIO_RESOURCES.md) | 历史增量／测量 |
| [P2_AUDIO_RUNTIME.md](P2_AUDIO_RUNTIME.md) | 历史增量／测量 |
| [P2_AUDIO_STARTUP.md](P2_AUDIO_STARTUP.md) | 历史增量／测量 |
| [P2_AUDIO_SUPPLY.md](P2_AUDIO_SUPPLY.md) | 历史增量／测量 |
| [P2_AUDIO_UI.md](P2_AUDIO_UI.md) | 历史增量／测量 |
| [P2_AUTOMATIC_CLOSEOUT.md](P2_AUTOMATIC_CLOSEOUT.md) | 历史增量／测量 |
| [P2_B05_PRODUCT_ROUTE.md](P2_B05_PRODUCT_ROUTE.md) | 历史增量／测量 |
| [P2_BACKGROUND_COORDINATION.md](P2_BACKGROUND_COORDINATION.md) | 历史增量／测量 |
| [P2_BUDGET_REVIEW.md](P2_BUDGET_REVIEW.md) | 历史增量／测量 |
| [P2_CLEANUP_RECOVERY.md](P2_CLEANUP_RECOVERY.md) | 历史增量／测量 |
| [P2_CLOSEOUT.md](P2_CLOSEOUT.md) | 当前契约／入口 |
| [P2_COMBINED.md](P2_COMBINED.md) | 历史增量／测量 |
| [P2_CONTINUOUS_BACKGROUND.md](P2_CONTINUOUS_BACKGROUND.md) | 历史增量／测量 |
| [P2_CONTROLLED_RECOVERY.md](P2_CONTROLLED_RECOVERY.md) | 历史增量／测量 |
| [P2_CPU_ACCOUNTING.md](P2_CPU_ACCOUNTING.md) | 历史增量／测量 |
| [P2_CPU_THREAD_IDENTITY.md](P2_CPU_THREAD_IDENTITY.md) | 历史增量／测量 |
| [P2_CURRENT_COMBINED.md](P2_CURRENT_COMBINED.md) | 历史增量／测量 |
| [P2_ENDPOINT_RETURN.md](P2_ENDPOINT_RETURN.md) | 历史增量／测量 |
| [P2_EVENT_LOOP_UI.md](P2_EVENT_LOOP_UI.md) | 历史增量／测量 |
| [P2_FAULT_REBUILD.md](P2_FAULT_REBUILD.md) | 历史增量／测量 |
| [P2_FINAL_ACCEPTANCE.md](P2_FINAL_ACCEPTANCE.md) | 固定验收／反馈 |
| [P2_FRAMEWORK_BASELINE.md](P2_FRAMEWORK_BASELINE.md) | 历史增量／测量 |
| [P2_FRAME_DISPATCH.md](P2_FRAME_DISPATCH.md) | 历史增量／测量 |
| [P2_FRAME_PREPARATION.md](P2_FRAME_PREPARATION.md) | 历史增量／测量 |
| [P2_GPU_CPU_RECHECK.md](P2_GPU_CPU_RECHECK.md) | 历史增量／测量 |
| [P2_GPU_PATH.md](P2_GPU_PATH.md) | 历史增量／测量 |
| [P2_GUI_BASE_COST.md](P2_GUI_BASE_COST.md) | 历史增量／测量 |
| [P2_GUI_BASE_NATIVE.md](P2_GUI_BASE_NATIVE.md) | 历史增量／测量 |
| [P2_GUI_CANDIDATE.md](P2_GUI_CANDIDATE.md) | 历史增量／测量 |
| [P2_GUI_CPU_ATTRIBUTION.md](P2_GUI_CPU_ATTRIBUTION.md) | 历史增量／测量 |
| [P2_HELD_GESTURE_FIX.md](P2_HELD_GESTURE_FIX.md) | 历史增量／测量 |
| [P2_HISTORY_PAINTER.md](P2_HISTORY_PAINTER.md) | 历史增量／测量 |
| [P2_HISTORY_PANEL.md](P2_HISTORY_PANEL.md) | 历史增量／测量 |
| [P2_INDEPENDENT_CPU.md](P2_INDEPENDENT_CPU.md) | 历史增量／测量 |
| [P2_INIT_RECOVERY.md](P2_INIT_RECOVERY.md) | 历史增量／测量 |
| [P2_INTERACTION_MODE.md](P2_INTERACTION_MODE.md) | 历史增量／测量 |
| [P2_LIVE_AUDITION.md](P2_LIVE_AUDITION.md) | 历史增量／测量 |
| [P2_MAIN_WORK.md](P2_MAIN_WORK.md) | 历史增量／测量 |
| [P2_MANUAL_ACCEPTANCE.md](P2_MANUAL_ACCEPTANCE.md) | 固定验收／反馈 |
| [P2_MANUAL_RESULTS.md](P2_MANUAL_RESULTS.md) | 固定验收／反馈 |
| [P2_MEMORY_USAGE.md](P2_MEMORY_USAGE.md) | 历史增量／测量 |
| [P2_MESH_REPLAY.md](P2_MESH_REPLAY.md) | 历史增量／测量 |
| [P2_PAINTER_CPU.md](P2_PAINTER_CPU.md) | 历史增量／测量 |
| [P2_PRESENT_FAULT.md](P2_PRESENT_FAULT.md) | 历史增量／测量 |
| [P2_PROGRESS.md](P2_PROGRESS.md) | 当前契约／入口 |
| [P2_REBUILD_ATTRIBUTION.md](P2_REBUILD_ATTRIBUTION.md) | 历史增量／测量 |
| [P2_REBUILD_POLICY.md](P2_REBUILD_POLICY.md) | 历史增量／测量 |
| [P2_REPAINT_MIN.md](P2_REPAINT_MIN.md) | 历史增量／测量 |
| [P2_REPAINT_ORIGIN.md](P2_REPAINT_ORIGIN.md) | 历史增量／测量 |
| [P2_RESOURCE_ATTRIBUTION.md](P2_RESOURCE_ATTRIBUTION.md) | 历史增量／测量 |
| [P2_RESOURCE_CYCLES.md](P2_RESOURCE_CYCLES.md) | 历史增量／测量 |
| [P2_RUNTIME_VOICE_BRIDGE.md](P2_RUNTIME_VOICE_BRIDGE.md) | 历史增量／测量 |
| [P2_SCHEDULING_CORRELATION.md](P2_SCHEDULING_CORRELATION.md) | 历史增量／测量 |
| [P2_SELECTION_CONTEXT.md](P2_SELECTION_CONTEXT.md) | 历史增量／测量 |
| [P2_SEPARATED_NATIVE.md](P2_SEPARATED_NATIVE.md) | 历史增量／测量 |
| [P2_SEPARATED_SUPPLY_MODEL.md](P2_SEPARATED_SUPPLY_MODEL.md) | 历史增量／测量 |
| [P2_STABLE_WINDOW.md](P2_STABLE_WINDOW.md) | 历史增量／测量 |
| [P2_STAGE_GATE.md](P2_STAGE_GATE.md) | 当前契约／入口 |
| [P2_STAGING_RECOVERY.md](P2_STAGING_RECOVERY.md) | 历史增量／测量 |
| [P2_STAGING_REUSE.md](P2_STAGING_REUSE.md) | 历史增量／测量 |
| [P2_SUBMISSION_STAGES.md](P2_SUBMISSION_STAGES.md) | 历史增量／测量 |
| [P2_SUPPLY_RECOVERY.md](P2_SUPPLY_RECOVERY.md) | 历史增量／测量 |
| [P2_TEXT_WORK.md](P2_TEXT_WORK.md) | 历史增量／测量 |
| [P2_THREAD_SAMPLING.md](P2_THREAD_SAMPLING.md) | 历史增量／测量 |
| [P2_TRANSFORM_CONTROLS.md](P2_TRANSFORM_CONTROLS.md) | 历史增量／测量 |
| [P2_UI_CPU.md](P2_UI_CPU.md) | 历史增量／测量 |
| [P2_UI_DISPLAY_REUSE.md](P2_UI_DISPLAY_REUSE.md) | 历史增量／测量 |
| [P2_UI_SEGMENTS.md](P2_UI_SEGMENTS.md) | 历史增量／测量 |
| [P2_UI_WALL.md](P2_UI_WALL.md) | 历史增量／测量 |
| [P2_UNDERRUN_CONTEXT.md](P2_UNDERRUN_CONTEXT.md) | 历史增量／测量 |
| [P2_VOICE_CPU.md](P2_VOICE_CPU.md) | 历史增量／测量 |
| [P2_VOICE_HANDOFF.md](P2_VOICE_HANDOFF.md) | 历史增量／测量 |
| [P2_VOICE_PHASE.md](P2_VOICE_PHASE.md) | 历史增量／测量 |
| [P2_WINDOW_RECOVERY.md](P2_WINDOW_RECOVERY.md) | 历史增量／测量 |
| [P2_WORKER_NATIVE.md](P2_WORKER_NATIVE.md) | 历史增量／测量 |
| [PRESENTATION_CONTRACT.md](PRESENTATION_CONTRACT.md) | 当前契约／入口 |
| [PREVIEW_SETTINGS.md](PREVIEW_SETTINGS.md) | 当前契约／入口 |
| [PREVIEW_TRANSPORT.md](PREVIEW_TRANSPORT.md) | 当前契约／入口 |
| [REQUIREMENTS.md](REQUIREMENTS.md) | 当前契约／入口 |
| [RESOURCE_BUDGET.md](RESOURCE_BUDGET.md) | 现行专题契约 |
| [ROADMAP.md](ROADMAP.md) | 当前契约／入口 |
| [TECH_STACK.md](TECH_STACK.md) | 当前契约／入口 |
| [p0/ALLOCATION_ATTRIBUTION.md](p0/ALLOCATION_ATTRIBUTION.md) | 历史研究／实验 |
| [p0/ASYNC_COMPOSITION.md](p0/ASYNC_COMPOSITION.md) | 历史研究／实验 |
| [p0/ASYNC_LOADING.md](p0/ASYNC_LOADING.md) | 历史研究／实验 |
| [p0/AUDIO_ADAPTATION.md](p0/AUDIO_ADAPTATION.md) | 历史研究／实验 |
| [p0/AUDIO_DEVICE.md](p0/AUDIO_DEVICE.md) | 历史研究／实验 |
| [p0/AUDIO_PCM.md](p0/AUDIO_PCM.md) | 历史研究／实验 |
| [p0/AUDIO_RECOVERY.md](p0/AUDIO_RECOVERY.md) | 历史研究／实验 |
| [p0/COMBINED_SOAK.md](p0/COMBINED_SOAK.md) | 历史研究／实验 |
| [p0/COMPOSITION.md](p0/COMPOSITION.md) | 历史研究／实验 |
| [p0/COMPOSITION_SCALE.md](p0/COMPOSITION_SCALE.md) | 历史研究／实验 |
| [p0/COMPRESSED_AUDIO.md](p0/COMPRESSED_AUDIO.md) | 历史研究／实验 |
| [p0/COMPRESSED_COMBINED_SOAK.md](p0/COMPRESSED_COMBINED_SOAK.md) | 历史研究／实验 |
| [p0/EDIT_COMPOSITION.md](p0/EDIT_COMPOSITION.md) | 历史研究／实验 |
| [p0/EVIDENCE_WORKFLOW.md](p0/EVIDENCE_WORKFLOW.md) | 历史研究／实验 |
| [p0/GPU_UPLOAD.md](p0/GPU_UPLOAD.md) | 历史研究／实验 |
| [p0/INTERACTION.md](p0/INTERACTION.md) | 历史研究／实验 |
| [p0/LOADER_TUNING.md](p0/LOADER_TUNING.md) | 历史研究／实验 |
| [p0/MEDIA_TIMELINE.md](p0/MEDIA_TIMELINE.md) | 历史研究／实验 |
| [p0/MP3_CANCELLATION.md](p0/MP3_CANCELLATION.md) | 历史研究／实验 |
| [p0/P0_EXIT_REVIEW.md](p0/P0_EXIT_REVIEW.md) | 历史研究／实验 |
| [p0/PREVIEW_SCOPE.md](p0/PREVIEW_SCOPE.md) | 历史研究／实验 |
| [p0/REPORT.md](p0/REPORT.md) | 历史研究／实验 |
| [p0/RESOURCE_BUDGET.md](p0/RESOURCE_BUDGET.md) | 历史研究／实验 |
| [p0/RESOURCE_COMPARISON.md](p0/RESOURCE_COMPARISON.md) | 历史研究／实验 |
| [p0/RESOURCE_CYCLES.md](p0/RESOURCE_CYCLES.md) | 历史研究／实验 |
| [p0/STABLE_C03_C08.md](p0/STABLE_C03_C08.md) | 历史研究／实验 |
| [p0/STABLE_C03_C08_RESULTS.md](p0/STABLE_C03_C08_RESULTS.md) | 历史研究／实验 |
| [p0/STABLE_FIRST_BATCH.md](p0/STABLE_FIRST_BATCH.md) | 历史研究／实验 |
| [p0/STABLE_OBSERVATIONS_20261020.md](p0/STABLE_OBSERVATIONS_20261020.md) | 历史研究／实验 |
| [p0/STANDARD_LOAD.md](p0/STANDARD_LOAD.md) | 历史研究／实验 |
| [p0/STREAMING_AUDIO.md](p0/STREAMING_AUDIO.md) | 历史研究／实验 |
| [p0/STREAM_OUTPUT.md](p0/STREAM_OUTPUT.md) | 历史研究／实验 |
| [p0/SWITCH_AUDIO.md](p0/SWITCH_AUDIO.md) | 历史研究／实验 |
| [p0/TRANSPORT.md](p0/TRANSPORT.md) | 历史研究／实验 |
| [p0/UPLOAD_LIFETIME.md](p0/UPLOAD_LIFETIME.md) | 历史研究／实验 |
| [p0/VALIDATION_MATRIX.md](p0/VALIDATION_MATRIX.md) | 历史研究／实验 |
| [p0/VORBIS_TAIL_AND_SEEK.md](p0/VORBIS_TAIL_AND_SEEK.md) | 历史研究／实验 |
| [p0/WINDOWS_DESKTOP.md](p0/WINDOWS_DESKTOP.md) | 历史研究／实验 |
| [research/LAZER_TIMELINE_20261020.md](research/LAZER_TIMELINE_20261020.md) | 历史研究／实验 |

## 原始证明JSON

基线117份JSON（包括P0／P1／P2）逐字节保留；按文件原范围解读，不重判失败、不跨二进制合并成绩。审查生成的CODE_INDEX／SUMMARY及归档MANIFEST具有独立用途，不是产品API或项目格式。
