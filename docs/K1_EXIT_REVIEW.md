# K1整体退出审阅：纯核心证据与成本补证

2026-11-09查询增量一致性审阅：轮前实现已通过正常完整hook备份推送`019a296f3198446aa1d2ee267dc9fda4f3378e0d`，私有origin及远端SHA一致。本轮修复冻结清单／原始证据／最终报告的发布截面、索引复核、输入代次校验和失败收尾边界，并补真实Rust查询输出到Python的无计时贯通回归，详见[审阅记录](K1_COST_CLOSURE_SPEC.md#2026-11-09查询增量一致性审阅)。计划仍为schema3／CostClosure-Query-v1，读取器修订为reader-v2；Reference10000／Tiny4、API7／profile1不变。PC三批次和正式成本测量未开展，coverage NeedsEvidence、产品NotEvaluated、KT23／K1及stable兼容结论保持。本轮修订未提交；后续先备份，再依明确任务推进。下方历史记录保留各自时点。

2026-11-09查询最小增量：轮前8份规格审阅文档已通过正常完整hook备份推送`d8f2ad99e8bb6035470619c6136701388540d180`，私有origin及远端SHA一致。本轮实现schema3／`CostClosure-Query-v1`的受测副本冻结、限根只读复核、摘要／账本／分页各16固定批次及真实Tiny canary；细节和验证见[实现记录](K1_COST_CLOSURE_SPEC.md#2026-11-09查询增量实现记录)。核心API7／profile1、Reference10000／Tiny4保持。PC三批次及正式新成本测量未开展，coverage固定NeedsEvidence、产品NotEvaluated，KT23／K1和stable兼容结论不升级。新增量未提交；下一步先备份并审阅本轮实现，下方旧记录保留各自时点。

2026-11-09成本收口规格一致性审阅：上一轮8份文档已通过正常完整hook备份推送`bc6075737be8893735f896a5ef57aa5eec2f0e8a`，PRIVATE origin及远端SHA一致。[本轮文档修正](K1_COST_CLOSURE_SPEC.md#2026-11-09一致性审阅记录)补齐真实Tiny 4／2／4容量与专用调度、24类操作的阶段映射、补证建立的采样容量、逐样本回执、失败清理及冻结证据只读复核。32份准备样本的64MiB静态公式保持，1MiB容器准入和实际分辨能力待实现验证。本轮仅审阅／文档修正，无schema3实现、无新成本轮；Reference10000／Tiny4、API7／profile1和现有schema2保持，KT23／K1、产品性能及stable兼容结论均不升级。修订未提交；下一步先备份，再按用户明确任务推进，不自动进入功能开发。下方旧记录保留原时点。

2026-11-09后续规格更新：本轮前已将下述退出审阅9文件通过正常完整hook保存并推送`921547f8462d93e4cfef7531c11b756f1932406b`，私有origin及远端SHA一致。[成本收口开工规格](K1_COST_CLOSURE_SPEC.md)已拟定待一致性审阅，明确6批次、32独立已准备样本的单池／聚合准入、Unknown和二进制保管；现有工具仍schema2，拟议schema3未实现。本轮不采新成本或改核心／工具源码，所列旧报告及散列保持；K1仍暂不签收，不能把写完规格当作补证完成。下文为2026-11-08审阅与复验记录。

2026-11-08。轮前将K1-6D一致性修订15文件通过正常完整hook保存为`2c387f2f57641e893d8dd12c3c363a75d4799995`，核对origin归属及PRIVATE可见性后正常推送，远端完整SHA一致。hook通过453项Python、169项gameplay、6项工具Rust及其他原核心测试、格式／Clippy／release／依赖边界，以及桌面默认／全feature测试和构建；未启动窗口或原生设备。原schema.rs空差异标记保留且未暂存。

**结论：K1暂不整体签收。** 已声明的SyntheticTest功能及有界受管资源已有适用纯核心通过证据，未发现本轮需要修正的运行逻辑偏差；KT23成本侧还缺“未分辨短调用的固定批次补证与对应关系”。这项缺口来自既有[K1-6成本契约](K1_6_SCALE_SPEC.md#成本记录和结论分层)，不是新增产品性能线。不能因为协议Valid就签收全部成本，也不要求将所有Unknown消除后才承认既有功能。

本轮只审阅、复验和补文档，不改gameplay／工具源码、API7／profile1、Reference10000／Tiny4、64槽输入／事件／确认窗口、256工作量；Reference两池各8MiB、Tiny两池各64KiB保持。不推进K2／GUI／原生设备，不验证stable兼容或产品N／S性能。

## 审阅口径

[K计划K1退出条件](STABLE_KERNEL_PLAN.md#12-分阶段执行与准入)仅要求KT01、03–11、14、21、23的**适用纯核心**用例及C1／C2／C3纯事件部分；总矩阵的客户端、宿主、编辑器和产品负载条款不能整体套用到K1。下表“已有通过证据”限于明确列出的切片，不将整项KT或全部平台标为通过。

本机证据为Windows x86_64／Rust1.95.0。纯核心的依赖图不引入平台运行时；这不等于本轮已在Linux／macOS执行全部用例。远端CI与本机检查分开，不把未查询的CI结论补作跨平台证明。RunId不复用仍是调用者约束，包含脱离借用的帧键／游标；不新增无界身份注册表。

## K1所属退出条款

| 条款 | 本轮核对的适用证据 | 结论与边界 |
| --- | --- | --- |
| KT01 | [正式边界检查](../tools/check_core_architecture.py)、[禁止边／嵌套模块canary](../tools/test_check_core_architecture.py)、[gameplay入口及私有状态编译拒绝](../crates/gameplay/src/lib.rs)；独立成本工具只能tool→gameplay | 本机纯核心通过；核心不读时钟／I/O／设备，不依赖项目写入口。工具的stdio／时钟及父进程观察不进入核心 |
| KT03 | [身份保留与错源](../crates/gameplay/tests/synthetic.rs)、[每个来源分量／旧页隔离](../crates/gameplay/tests/frames.rs)、[相同外部ID的外国结果拒绝](../crates/gameplay/tests/preparation.rs) | 适用身份隔离通过；同点同刻身份可表达，但重叠／有歧义布局按profile拒绝，不声称已实现notelock或堆叠 |
| KT04 | synthetic的`signed_time_extremes_empty_maps_and_storage_limits_are_explicit`、`radius_uses_both_axes_relative_to_center_and_valid_time_endpoints_survive`；[驱动端点](../crates/gameplay/tests/drivers.rs)及[规模检查端点](../crates/gameplay/tests/scale_controls.rs) | 有符号MapTime、端点检查及无Q-1哨兵通过；offset／采样率余数／生成复制消费位置和真实时钟漂移属宿主阶段 |
| KT05 | synthetic的`absolute_time_segmentation_budget_and_gui_sampling_do_not_change_results`明确30／60／144／240Hz、规则／不规则分段及1／256工作量；[冻结分段](../crates/gameplay/tests/freeze.rs)、[原子前缀与旧完整帧](../crates/gameplay/tests/frames.rs)、万对象长目标推进 | 同一接纳集合及相同控制顺序的适用确定性通过；不是实时调度或不同接纳集合成绩相等的证明 |
| KT06 | synthetic的生命周期／闭窗后一微秒Miss／同刻输入顺序／事件背压；frames的同刻Hit／Miss、Pending／Failed旧截面；[事件号耗尽拒绝](../crates/gameplay/src/run.rs) | 已提交状态、账本及两事件Hit的原子性、只判一次、稳定重读／确认通过 |
| KT07 | synthetic的时间／空间边界及双轴半径；[分步校验及多错分类](../crates/gameplay/tests/scale.rs)、[非整齐归并](../crates/gameplay/tests/scale_consistency.rs) | SyntheticTest-v1的已知边界／拒绝通过；固定微秒／整数坐标不充当stable量化，未知规则未进入兼容profile |
| KT08 | synthetic的物理槽聚合、重复／冲突／整批接纳、C1含等号封口；freeze的定时重新武装；[四槽与旧释放规模隔离](../crates/gameplay/tests/scale_controls.rs) | 合成输入协议通过；文本／IME、系统捕获完整性、失焦及UI工具路由留到原生／编辑器接线 |
| KT09 | [三驱动互斥、共同判定与恢复](../crates/gameplay/tests/drivers.rs)、[真实Automatic万对象轨迹](../crates/gameplay/tests/scale_playback.rs)、[真实脉冲序号](../crates/gameplay/src/run/driver.rs) | 纯驱动通过；没有产品非全屏默认、开关UI或Auto Mod接线 |
| KT10 | freeze／drivers的同刻控制顺序、部分推进／背压冻结、当前来源重新武装及恢复不补旧输入；scale_controls六个对象检查点 | 玩法时间／输入／语义事件适用部分通过；音频回执、材料热改及独立试听的场景接线不由K1证明 |
| KT11 | [检查候选](../crates/gameplay/tests/inspection.rs)、[创建／载入／检查统一候选](../crates/gameplay/tests/preparation.rs)、[规模检查与旧任务](../crates/gameplay/tests/scale_candidates.rs) | 分步准备、关闭Q、Skipped／Unavailable、无伪判定／声音、失败／取消不发布、旧继续拒绝及原统计来源通过 |
| KT14 | [容量／原始工作计数](../crates/gameplay/tests/scale.rs)、[双池差1字节／溢出](../crates/gameplay/tests/preparation.rs)、[真实共享所有者及分配拒绝](../crates/gameplay/src/coordinator/tests.rs)、[取消后协调器关闭](../crates/gameplay/tests/scale_candidates.rs) | 受管共享、预留、payload及最后所有者寿命的适用部分通过；C3、缓存、历史原字节及分配器／OS占用不以此替代 |
| KT21 | [FrameCounts／StatisticsOrigin／实时单项与完整帧](../crates/gameplay/src/run/frame.rs)、inspection的Skipped不伪造Miss、scale_controls的Q及恢复后完整账本 | 从头／检查统计和未计算／旧完整帧可辨；没有score／ACC／combo样例冒充真实结果，HUD与区段产品呈现未接入 |
| KT23 | [G0](../crates/gameplay/tests/scale.rs)、[G1／A1](../crates/gameplay/tests/scale_playback.rs)、scale_controls／scale_candidates、[B容量依据](K1_6B_STORAGE_PROOF.md)、[D v2原始成本记录](K1_6D_COST_REVIEW.md#2026-11-08一致性审阅记录) | 黄金轨迹、有界工作／所有权通过；已建立有范围限制的有效阶段基线，短调用的固定批次补证仍待收口，不签收完整KT23／K1 |

另外核对KT13／20／24的交叉约束：当前纯模型拒绝旧来源／候选、故障不复活输入、取消不依赖额外容量；没有项目依赖、Command或schema变化。这不是K3的皮肤／设备代次和故障接线验收。

169项gameplay由24单测、20驱动、12帧、16冻结、18检查、26候选协调、8原规模、3规模候选、7规模一致性、7规模控制、2规模轨迹、23合成及3编译拒绝组成。按实际测试定义及本轮正常hook日志复核，不把不同历史版本的测试数相加。四对象返回点矩阵和万对象固定选点有各自范围，不声称穷举所有万对象调度。

## K1-6矩阵交叉核对

| 条目 | 适用证据与结果 |
| --- | --- |
| KS01 | scale覆盖0／1／4／5／31／32／9999／10000，Tiny及便捷入口第5项、Reference第10001项、任意ID、原记录错误顺序及受控拒绝；各入口范围见scale_consistency、frames、preparation，已有通过记录 |
| KS02 | 准备每步读取／复制／索引初始化／比较／搬移及运行初始化；1／256、Ready前拒绝激活、无零工作Pending；容量不靠事后capacity追加计费，已有通过记录 |
| KS03 | G0各10000 Miss／10000事件，G1／A1各10000 Hit／20000事件；完整来源及A1释放事实／脉冲序号单独核对，末态全退休，已有通过记录 |
| KS04 | 常量摘要、实时单项、最多64项页；源码只迭代请求页，计数回归核对复制数；Pending／Failed旧完整帧与旧页、同T换源／发布号、无帧／空图／错误输出及耗尽，已有通过记录 |
| KS05 | 自动计划未接纳／已接纳／Hit已提交、输入／事件背压及差一槽、冻结恢复不补发、四槽旧释放拒绝，已有通过记录 |
| KS06 | Q=-1／0／5000000000／9999100001／i64最小值；检查Q关闭，Manual可信释放后Q+1与Automatic仅从目标>Q继续的不同预期完整核对，已有通过记录 |
| KS07 | 三类候选各准备阶段、取消／失败／Ready／已激活仍持有；旧键／新键配旧结果／外国池／旧任务Drop隔离，已有通过记录 |
| KS08 | 容量／乘加溢出／双池原子预留、共享一次与独立分配分别计费、取消保持／实际Drop释放、Weak最后所有者与payload交叉核对；受管峰值已有记录，不能改名为进程阶段真实峰值 |
| KS09 | 取消保留已提交前缀、旧继续不污染计数、故障与代次／事件／进度耗尽的停止路径、无项目写入口，已有通过记录 |

这些功能／受管资源记录不自动满足D的全部计量契约，尤其不能用工作计数替代短调用批次或进程读数。

## Unknown的实际影响及待办

v2-01报告仍为原协议Valid，report SHA-256为`297b3900e252683908cd28b7fb895cd8964c470036af6a409dcf91453ff6a17c`，源码摘要映射仍为`70c7a5452eab7011afd6f740fb263c64cff3eacc7e17999e0c5aec11cf37aaa0`。原始数据不回写，不把整体审阅缺口改称该轮协议Invalid。

| v2-01的Unknown | 对K1退出的影响 |
| --- | --- |
| 96／108个CPU窗口及108个阶段真实峰值 | 保留原始user／kernel、边界实读、采样最大值和OS累计高水位。阶段峰值不可由有限采样证明，CPU短窗不可填0；K1-6允许未覆盖项单列，没有据此新增产品性能门槛 |
| 12个empty序列、10个fixture_drop序列 | 前者是空调用参考，后者是调用者数据释放；与核心操作补证区分，不能要求把空参考强行测成正值来“通过” |
| 7个page序列 | 已有4096页固定批次，可解释对应批次时长；单次页调用Unknown仍保留，不除以4096伪造其p95 |
| 12个summary＋12个ledger序列 | 无对应固定只读批次；整段queried还含其他调用／驱动校验，不能冒充单独摘要或账本批次。这是明确缺口 |
| 其余156个核心调用序列 | 含创建／准备／激活、检查准备、seal／advance／ack／automatic、冻结家族、取消及任务Drop。需按真实操作／家族及阶段建立批次对应，不把混合轨迹、不同阶段的同名桶或纯管理批次当成任一API的单次百分位 |

上述共209个序列、18种操作名；其中包括诊断／调用者字段及已有页批次，不等于还要实现209个基准。特别是256次request／cancel／Drop批次针对未开始的任务，并非5000项检查写入后取消的单独成本；cancelled阶段又含半程准备。两者都不能替代缺少的那一项记录。

**下一最小工作组：成本收口补证。** 2026-11-09[有界批次规格](K1_COST_CLOSURE_SPEC.md)草案已备份bc60757，并完成本轮一致性审阅文档修正；下一步先备份修订，随后仅按明确任务实现、回归和另轮测量。部分实施不能冒充完整CostClosure-v1通过；不增加模型功能或改变Reference／Tiny。下列审阅最低要求仍保留：

1. 将当前未分辨核心操作映射到只读、分步准备／推进、控制／生命周期等实际批次，保留所处阶段和前置状态。freeze桶内请求／收尾／选驱动／恢复的含义不同，不能从混合桶反推任一API成本。
2. 固定每批操作数、对象／事件负载、有限重复数、1／256工作量和黄金结果；状态准备放在哪里、是否计入批次必须明示。真实取消与实际Drop继续分开，不用旧请求重放或无效调用替代成功路径。
3. 批次总时长、每批样本数和最近秩统计按批次单位报告；样本／资源在预算内，计量参考、诊断扰动及不足分辨时的Unknown仍保留。任何无法满足既有契约的部分先明确修订范围和依据，不悄悄豁免后回填旧结果通过。
4. 保存本轮实际受测的独立二进制副本及来源／二进制／原始报告散列，检查可覆盖target输出但不能覆盖已冻结证据。新旧构建与输出版本分别解释；独立审阅后再回到本表签收KT23／K1。

这不是产品预算达标的验收单，也不要求此阶段补原生设备、真实播放CPU、声学延迟或stable行为证据。K0-B的规则支持矩阵／非空兼容profile、K2格式、K3宿主／材料／声音、K4客户端对照、K5产品N／S与长时生命周期仍按原门槛执行。

## 本轮复验与冻结证据

轮前hook重建release后，target中的exe SHA为`3400431b58fdaa051ddf18570cf28d3c410283ff429b245d75d6589e9590f8b7`，与v2-01受测哈希不同，源码映射却一致；哈希变化的具体构建原因未确定。旧轮只证明其当时冻结构建，不能拿后来重建的同路径exe冒充旧轮产物。它的原始报告及状态不改写。

本轮在独立目录`artifacts/k1-exit-review/measurement-v2-frozen/`通过现有入口重新执行G0／G1／A1 × 1／256 × 两进程的12进程诊断。没有修改工具、运行时或重选负载。完成后立即核对实际exe与新报告散列，独占新建`frozen-probe.exe`（347136字节）并再次校验，另存`freeze-receipt.json`。冻结副本与原始诊断均在本机忽略目录，不进入Git；不依赖未来仍可覆盖的target路径来保管证据。

| 本轮冻结项 | SHA-256 |
| --- | --- |
| report.json | 704294e75c05058dcd5ad864b2561f4df383e01774459a4020bec8eee4544df0 |
| 受测exe及独立冻结副本 | 882707c465e89ccd46b48b83578cc8366e1dee8a1a9fceb7114435cc2d8d9e37 |
| 排序源码摘要映射的紧凑JSON | 70c7a5452eab7011afd6f740fb263c64cff3eacc7e17999e0c5aec11cf37aaa0 |

所有12进程／108阶段及end重新校验通过；逐JSONL散列、CSV记录数、采样最大值和跨阶段空档与新报告一致；取消持有／任务真实Drop／最终无观察者的生命周期校验通过。新轮206个短调用序列（18种操作名）、96个CPU窗口及108个阶段真实峰值Unknown，最大观察空档151.4401ms；其余12个CPU窗口仍仅为连续参考。协议Valid、产品NotEvaluated。本轮数字与上文v2-01的209序列属于不同构建／轮次，不合并或据其差异宣布优化。

此次复验解决本轮受测二进制的保管问题，没有新增缺失批次，故不改变KT23待补证结论。通用测量入口以后如何自动保存冻结副本，仍随成本收口规格处理；本轮没有悄悄改工具。正式源检查由轮前完整hook及诊断入口再次检查提供，文档增量另作路径／锚点及仓库检查；本轮新文档尚未提交。

最终新增／修订9份Markdown，git diff --check、806文件仓库候选检查、本轮文档本地路径／锚点／围栏／行尾检查通过。当前核心及工具源码、根／desktop清单和锁文件与回退HEAD一致，schema.rs规范化内容不变；暂存区为空，原空差异标记保留。新冻结二进制／报告／源码身份复核一致，旧v2-01报告散列未变。没有新增自动测试或另跑未涉及的功能测试，正常完整hook与现有测量入口的检查结果分别保留在本机`artifacts/k1-exit-review/`。
