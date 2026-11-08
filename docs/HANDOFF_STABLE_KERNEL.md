# 独立 stable 兼容内核：新对话交接

2026-11-09当前状态以[项目全面检查](PROJECT_REVIEW_20261109.md)为统一入口：Query-v1实现及一致性修订已完成；轮前CI测试修正已通过完整hook备份推送`6ac5fda`。K1仍缺PC三批次／完整新轮及独立退出决定，P3普通保存闭环未接入。建议有限收口K1后优先审阅P3接线，尚未授权开发。本轮只修正文档；下方带日期的增量记录保留各自时点，其“本轮／下一步／未提交”不覆盖当前快照。

2026-11-09 CI故障确认：[019a296的Actions 42523539838](https://github.com/sumiyuki727/skin-editor/actions/runs/42523539838)中Linux核心和Windows桌面通过，Windows核心在Python 3.12.10的test_reparse_points_are_refused_without_following_target失败。硬链接测试以tuple重建stat_result，丢失Windows专用st_file_attributes；新pathlib的lstat委托stat后取得None，触发None与0x400位运算异常。本机Python 3.9.6用同一委托路径成功复现。修正仅用转发替身覆盖st_nlink，保留全部原生属性，并显式覆盖委托路径；生产安全检查、workflow及产品行为均不变。上一轮17文件修订已通过正常完整hook备份推送856cabd5ae34af686bc10f933d6d8aeec63590ea，远端SHA一致；该备份尚不包含本次CI修正，不能声明远端已恢复。本次测试修正与此记录未提交，修正后62项成本工具Python回归及仓库候选／git diff --check通过；直接Python 3.12复验尚缺，日志见本机artifacts/github-query-ci-review/，后续需提交后由远端重新验证。

2026-11-09查询增量一致性审阅：轮前实现已通过正常完整hook备份推送`019a296f3198446aa1d2ee267dc9fda4f3378e0d`，私有origin及远端SHA一致。本轮修复冻结清单／原始证据／最终报告的发布截面、索引复核、输入代次校验和失败收尾边界，并补真实Rust查询输出到Python的无计时贯通回归，详见[审阅记录](K1_COST_CLOSURE_SPEC.md#2026-11-09查询增量一致性审阅)。计划仍为schema3／CostClosure-Query-v1，读取器修订为reader-v2；Reference10000／Tiny4、API7／profile1不变。PC三批次和正式成本测量未开展，coverage NeedsEvidence、产品NotEvaluated、KT23／K1及stable兼容结论保持。本轮修订未提交；后续先备份，再依明确任务推进。下方历史记录保留各自时点。

2026-11-09查询最小增量：轮前8份规格审阅文档已通过正常完整hook备份推送`d8f2ad99e8bb6035470619c6136701388540d180`，私有origin及远端SHA一致。本轮实现schema3／`CostClosure-Query-v1`的受测副本冻结、限根只读复核、摘要／账本／分页各16固定批次及真实Tiny canary；细节和验证见[实现记录](K1_COST_CLOSURE_SPEC.md#2026-11-09查询增量实现记录)。核心API7／profile1、Reference10000／Tiny4保持。PC三批次及正式新成本测量未开展，coverage固定NeedsEvidence、产品NotEvaluated，KT23／K1和stable兼容结论不升级。新增量未提交；下一步先备份并审阅本轮实现，下方旧记录保留各自时点。

2026-11-09成本收口规格一致性审阅：上一轮8份文档已通过正常完整hook备份推送`bc6075737be8893735f896a5ef57aa5eec2f0e8a`，PRIVATE origin及远端SHA一致。[本轮文档修正](K1_COST_CLOSURE_SPEC.md#2026-11-09一致性审阅记录)补齐真实Tiny 4／2／4容量与专用调度、24类操作的阶段映射、补证建立的采样容量、逐样本回执、失败清理及冻结证据只读复核。32份准备样本的64MiB静态公式保持，1MiB容器准入和实际分辨能力待实现验证。本轮仅审阅／文档修正，无schema3实现、无新成本轮；Reference10000／Tiny4、API7／profile1和现有schema2保持，KT23／K1、产品性能及stable兼容结论均不升级。修订未提交；下一步先备份，再按用户明确任务推进，不自动进入功能开发。下方旧记录保留原时点。

2026-11-09成本收口规格：K1整体退出审阅9文件已通过正常完整hook备份推送`921547f8462d93e4cfef7531c11b756f1932406b`，私有origin及远端SHA一致。[成本补证开工规格](K1_COST_CLOSURE_SPEC.md)已细化6类固定批次、32份实际准备后取消样本的单池／64MiB聚合准入、Unknown边界和受测二进制独立冻结。schema3／CostClosure-v1仍是待一致性审阅的拟实施契约，现有工具仍schema2；本轮只改文档，不开发／重测。Reference10000／Tiny4、API7／profile1和产品预算保持，KT23／K1仍未签收。本轮规格未提交，下一步先备份并审阅规格，再按明确任务实施最小增量；不接GUI／原生设备，不宣称产品性能或stable兼容达标。下方旧记录保留原时点。

2026-11-08最新K1整体退出审阅：K1-6D v2修订已通过正常完整hook备份推送`2c387f2f57641e893d8dd12c3c363a75d4799995`，私有origin及远端SHA一致。[整体退出核对](K1_EXIT_REVIEW.md)已完成：已声明功能与有界受管资源的适用纯核心项已有通过证据；KT23仍缺未分辨短调用的固定批次补证与对应关系，K1暂不整体签收。独立新轮12进程协议Valid，受测二进制已单独冻结；Unknown和产品NotEvaluated保持，不与旧轮混合。本轮只改文档、复验证据，API7／profile1、Reference10000／Tiny4及运行逻辑保持；文档未提交。下一步先备份本轮审阅，再细化成本收口补证规格，不自动开始K2／GUI／原生设备或声明性能／stable兼容达标。下方旧记录保留各自时点。

2026-11-08最新D一致性审阅：初版20文件已通过正常完整hook备份推送`4994d2e73989b76ae727cb83dbc08ec943502e6e`，私有origin及远端SHA核对一致。[本轮修订](K1_6D_COST_REVIEW.md#2026-11-08一致性审阅记录)补10项Python回归，修复构建冻结、结束／阶段协议、异常记录保留、计量及资源边界，成本输出升为v2；453项Python／169项gameplay／6项工具Rust、正式核心和桌面架构检查通过。v2新测12进程协议Valid，209个调用序列、96个CPU窗口和全部108个阶段真实峰值仍Unknown，产品NotEvaluated。Reference10000／Tiny4、API7／profile1及运行逻辑保持。本轮修订未提交，下一步先备份，再核对K1整体退出条件及Unknown的影响；不自动进入K2／GUI／原生设备，不宣称性能或stable兼容达标。下方旧记录保留各自时点。

**2026-11-07最新D停靠点：** C一致性修订已备份推送为`13fb4667840b6371d4830d92d308ad436fe5eb49`，本轮无新可审查差异而复用该回退点。[K1-6D](K1_6D_COST_REVIEW.md)已新增独立标准库成本工具与Windows自有子进程采集，完成12进程首组基线；协议／黄金／资源校验Valid，短调用与多数CPU窗口Unknown，产品预算NotEvaluated。443项Python／169项gameplay／6项工具Rust、正式核心和桌面架构检查通过。运行时、API7／profile1、Reference10000／Tiny4及预算保持。本轮增量未提交，下一步先检查备份再独立审阅D，随后核对K1整体退出条件；不自动接GUI／原生设备或声明stable兼容。下方旧记录保留各自时点。

**2026-11-07最新C审阅停靠点：** C初版13文件已通过正常完整hook备份推送`0907573862759290e4b82175345a1b233ed7919e`，PRIVATE origin及远端SHA一致。[一致性审阅](K1_6C_SCALE_REVIEW.md#2026-11-07一致性审阅记录)修正私有检查第0项与仍留队声音的测试条件，补2项规模回归及黄金夹具／故障隔离断言；正式核心425项Python／169项gameplay、桌面架构检查通过。运行时、API7／profile1、Reference10000／Tiny4及预算保持。本次修订未提交，留待下轮检查备份，不自动开始D成本、GUI／原生设备或stable兼容。下方旧记录保留原时点。

**2026-11-06最新B审阅停靠点：** B初版32文件已通过正常完整hook备份推送为`747222c8c69e39ff39a84e933a73883bcfedb80c`，PRIVATE origin及远端SHA一致。[本次一致性审阅](K1_6B_STORAGE_PROOF.md#2026-11-06一致性审阅记录)修复激活计数回执丢失、无效起点多记分配及旧控制污染摘要计数，新增7项公开回归。正式核心通过425项Python及156项gameplay，桌面架构检查通过。固定Rust commit容量依据、Reference10000／Tiny4、API7／profile1及布局保持；G0、取消后的延迟释放、Pending／Failed旧完整帧和旧页隔离复核通过。本次修订未提交，C完整输入／实际自动规模轨迹及D实际成本仍未完成，不接GUI／原生设备、不宣称stable兼容。下方A及早期记录保留各自时点。

**2026-11-06最新停靠点：** K1-6A初版28文件已通过正常完整hook备份推送为`c447a2a`，远端SHA一致。[本次一致性审阅](K1_6A_STORAGE_REVIEW.md#2026-11-06一致性审阅记录)补5项来源寿命、同刻迁移、Pending／Failed旧完整帧／页及首次失败无帧回归，共137项gameplay通过；未发现需修改运行逻辑的偏差。API6／profile1、四对象及默认池保持，正式核心及桌面架构检查通过。本次测试／说明修订未提交，后续先备份再按明确任务推进；B–D、10000对象及实际成本仍待做，不接GUI／设备，不宣称stable兼容或完整K1／KT。下方早期记录按当时时点保留。

2026-11-03。这是架构与实施计划的交接，**没有实现内核功能**。新对话先读当前AGENTS／Git和用户本轮任务；本文件不授权自动开工、访问私人谱面、运行窗口或提升权限。

2026-11-04补充三项执行契约及K0-A／K0-B准入要求，详见[方案](STABLE_KERNEL_PLAN.md)；同日后续整理[K0-A开工规格](STABLE_KERNEL_K0_A.md)。本轮仅文档修改，未新增功能、fixture或KT通过记录，K0清单仍待逐项审阅／补齐。

本轮已完成K0-A文档自审并补齐列出的初始／检查截面、批次载荷／序号、重新武装、分步准备预算、错误分类及KT23固定负载问题。只表示文档修订完成，不记录用户确认或K0-A／KT通过；K0-B和K1实现仍是独立后续工作。

## 1. 当前任务与停靠点

用户希望独立实现一个适合皮肤编辑／离线测试的stable兼容内核。首批限定 **osu!standard／No Mod／圆圈子集**。主文档为[架构与分阶段计划](STABLE_KERNEL_PLAN.md)，本轮综合结论见[全面检查](PROJECT_REVIEW_20261109.md)。K1-1至K1-5、K1-6 A–D及Query-v1已有适用实现／复核，剩余成本补证未闭合；K2–K5与完整KT仍待完成。K0-B是可执行兼容profile准入，规则证据仍未完成，不因合成模型通过而开放。

“独立”是纯Rust、可无窗口测试的模块；产品保持单进程模块化单体。不能裁剪、复制、翻译移植无授权旧stable源码，不能因为lazer有实现就当stable规则已验证。

已实现SyntheticTest圆圈生命周期、输入、冻结／恢复／重新武装、检查、候选／预算及三驱动互斥；API7／profile1、Tiny4／Reference10000，G0／G1／A1及有限控制交错已有回归。成本工具schema3／CostClosure-Query-v1、reader-v2已实现并审阅冻结保管、只读复核与三类查询批次。PC三批次／完整CostClosure-v1未实现，未运行正式新轮；KT23／K1、产品性能及stable兼容结论不升级。本轮只审阅项目和同步当前文档。

## 2. Git、代码基线和并行修改

- 本轮正式核心及桌面默认／全feature无窗口检查、GitHub复验和文档检查结果统一见[全面检查](PROJECT_REVIEW_20261109.md#本轮验证与证据身份)；旧冻结副本／报告散列不变。
- 本轮回退点6ac5fda1efc342ae72001f60058c4205def4c5b2已通过正常完整hook并同步PRIVATE origin，包含上一轮CI测试替身修正和两份说明；Query一致性修订此前已保存856cabd。本轮全面检查文档为新未提交增量，既有schema.rs规范化空差异标记保留。
- 工作区为 E:\skin-editor，分支codex/p1-bootstrap；远端[sumiyuki727/skin-editor](https://github.com/sumiyuki727/skin-editor)须再次核对PRIVATE。
- 历史恢复代码基线为2719d0f；先前交接文档保存于6e26e9d。动画设计回退点为d3f4b2b34b2c088932acf1641163c72915163e41，经过正常仓库hook并同步私有origin；它没有增加动画功能。
- 独立内核方案初版及关联交接已保存于881b6613625e4821036bd7fd2cb8a8cd710f96e5，完整本地hook通过并推送私有origin；后续契约文档回退点见下一条。该初版文档提交没有增加内核功能，不证明KT或规则已验证。
- C1／C2／C3及K0准入文档已保存于ff1a3081e2a5123a695bbbb1fc563c772ff94915，正常完整hook通过并同步私有origin；它只包含两份文档。
- K0-A规格初版及入口修订已保存于2be447c4fde0af92064598bddb68c6ed6ec93f49，本轮先通过正常完整hook、推送私有origin并核对远端SHA，范围为三份文档。之后的本轮自审修订按实际Git状态交付，不冒充已推送或已通过KT。
- K0-A自审修订三份文档已在首版实现前保存于c295816e2e0212383854b42d1f72386d6d2b1e99，经正常完整hook并推送私有origin、核对远端SHA。
- K1-1首版24文件已在2026-11-05保存为a609d743e97a6e951b7ee0cf1221e68e525f5def，正常完整hook通过并推送已核对的PRIVATE origin、远端SHA一致。
- K1-1一致性修订7文件已保存为88ed5df123a8d94b2d81ca1b72a706e2f4055513，同样通过正常完整hook并推送私有origin、远端SHA一致。
- K1-2首版17文件已保存为632b5d2317944a88144014691d5df18d9eeaf9d5，通过正常完整hook、推送已核对的PRIVATE origin并核对远端SHA。
- K1-2一致性修订7文件已保存为78b0236e0f834d6618752f5094cf7d5d0b17e129，正常完整hook通过并推送私有origin、远端SHA一致。
- K1-3初版15文件已保存为44687bf2db826d6f5c8aacf0899b3554e4f59244，正常完整hook通过并推送私有origin、远端SHA一致；首次链接失败及后续复核／完整重跑分别保留，详见该增量审阅。
- K1-3一致性审阅的6个测试／文档文件已保存为5c5e1ed1348c02e2895760a70b724de7f5daef5c，正常完整hook通过并推送PRIVATE origin、远端SHA一致。
- K1-4初版18文件已保存为4d4ee9bd3e11a9baaf6aef660c713b5e6b914898，正常完整hook通过并推送PRIVATE origin、远端SHA一致。
- K1-4一致性测试／审阅7文件已保存为de3cf8bed7dff80fe05c0751d3c0b3d43b96fbcf，正常完整hook通过并推送PRIVATE origin、远端SHA一致。
- K1-5初版21文件已保存为0a52f858c7de4824f5ab10e433ae067efdd00e4a，本轮正常完整hook通过并推送已核对PRIVATE的origin、远端SHA一致；随后的一致性测试及审阅文档另为本轮未提交修订。
- K1-5一致性测试及审阅7文件已保存为62a4e4d7bcd9647dc1a43c0ec50e7bcf59146349，本轮正常完整hook通过并推送PRIVATE origin、远端SHA一致；116项gameplay为该备份基线。本轮K1-6规格另作未提交文档，不修改生产源码或测试。
- K1-6开工规格初稿及入口7文件已保存为ffe2361a6a89bcc4c34f043c99b3974a217f35a2，本轮正常完整hook通过并推送PRIVATE origin、远端SHA一致。随后只修订规格／方案／交接等文档，未开发；旧“本轮未提交”按其当时状态保留。
- K1-6规格审阅6文件已保存为f1c8dbfdd086a84eafdb0f6ed0a2c9a1f7734389，本轮正常完整hook通过并推送PRIVATE origin、远端SHA一致；之后K1-6A代码／测试／文档另作本轮未提交增量，旧“本轮未提交”保留当时含义。
- K1-6A初版28文件已保存为c447a2a37380ee862699621917a8d366dd2a0d54，正常完整hook包含132项gameplay及桌面默认／全feature检查，推送PRIVATE origin并核对远端SHA；之后本次一致性回归／说明修订未提交，137项新状态另经正式核心检查，不沿用备份或远端CI结论。
- K1-6A一致性修订11文件已保存为b3c5f088932666c97ae8576b468b0c7b177bc7d5，正常完整hook包含137项gameplay及桌面默认／全feature，推送PRIVATE origin并核对远端SHA；本轮B的149项新增状态另经正式核心检查，源码／测试／文档未提交，不能沿用备份或远端CI结论。
- K1-6B初版32文件已保存为747222c8c69e39ff39a84e933a73883bcfedb80c，正常完整hook包含149项gameplay及桌面默认／全feature，推送PRIVATE origin并核对远端SHA。本次B一致性审阅修复三项计数边界，补7项回归，共156项gameplay另经正式核心检查，源码／测试／说明未提交；旧条目的“本轮未提交”保留其原时点。
- 最新默认流程：先看status／log／实际暂存差异，将上一轮来源明确、可审查的工作通过检查及hook提交成回退点，默认同步核对过的私有origin；用户明确说不提交／推送时除外。新增量轮末不自动一起提交。
- 2026-11-09成本收口规格草案8文档已正常备份为bc6075737be8893735f896a5ef57aa5eec2f0e8a，完整hook及PRIVATE origin远端SHA核对通过。本轮仅完成一致性审阅文档修正，尚未提交；没有schema3实现或新成本轮。
- 2026-11-09 K1整体退出审阅9文件已保存为921547f8462d93e4cfef7531c11b756f1932406b，正常完整hook含453项Python／169项gameplay／6项工具Rust及桌面默认／全feature检查／构建，正常推送PRIVATE origin并核对远端SHA。随后本轮只补成本收口规格和交接，未开始功能开发或成本重测；文档新增量未提交，旧本机冻结证据继续保留，不沿用远端CI为草案背书。
- 2026-11-08 D v2一致性修订15文件已保存为2c387f2f57641e893d8dd12c3c363a75d4799995，正常完整hook含453项Python／169项gameplay／6项工具Rust及桌面默认／全feature检查／构建，推送PRIVATE origin并核对远端SHA。本轮K1整体退出审阅及新原始诊断只追加文档／本机证据，文档尚未提交；不沿用远端CI为未上传文档或新测量背书。
- 2026-11-08 K1-6D初版20文件已保存为4994d2e73989b76ae727cb83dbc08ec943502e6e，正常完整hook含443项Python／169项gameplay／6项工具Rust、桌面默认／全feature检查与构建，推送PRIVATE origin并核对远端SHA。随后本轮D一致性修订为未提交工作，成本协议v2、453项Python及新12进程证据单列，不沿用备份或远端CI结论。
- C一致性修订11文件已保存为13fb4667840b6371d4830d92d308ad436fe5eb49，正常完整hook含169项gameplay及桌面默认／全feature，前轮已推送PRIVATE origin并核对远端SHA。本轮启动无新可审查差异、复用HEAD未造空提交；随后D工具／基线说明为本轮未提交新增量，测量源码身份以D报告的文件摘要为准，不冒用HEAD。
- 2026-11-07 K1-6C初版13文件已保存为0907573862759290e4b82175345a1b233ed7919e，正常完整hook包含167项gameplay及桌面默认／全feature，已推送PRIVATE origin并核对远端SHA。随后本次一致性审阅修订测试条件并补2项规模回归，共169项gameplay另经正式核心检查；本次修订未提交，不沿用备份或远端CI结论。
- 2026-11-07 K1-6B一致性修订13文件已保存为bf3fff51cac5e7a9c1fa618efda3e07c11e86cac，正常完整hook包含156项gameplay及桌面默认／全feature，已推送PRIVATE origin并核对远端SHA。随后本次K1-6C新增11项回归，共167项gameplay另经正式核心检查；本次测试／说明增量未提交，不沿用备份或远端CI结论。
- 保留其他对话工作，不并发Git写操作，不替其他对话暂存，不盲目全目录add、不跳过hook／强推／amend／重写历史。
- schema.rs可能显示M而实际diff为空，已有核对属换行规范化现象；先检查内容，不为消标记修改DDL、清理源码或改旧证明。

如果在同checkout继续，先确认是否有编辑器存储／素材skill对话在工作；按实际文件责任错开写入。需要隔离时遵循当前worktree工具和用户授权，不擅自搬移工程。

## 3. 最短阅读路径

| 顺序 | 内容 |
| --- | --- |
| 1 | [AGENTS](../AGENTS.md)、[Git规则](GIT_WORKFLOW.md)、[README](../README.md) |
| 2 | [内核方案](STABLE_KERNEL_PLAN.md)：C1／C2／C3、K0总表与KT矩阵；再读[K0-A规格](STABLE_KERNEL_K0_A.md)、[K1-2冻结接口](K1_2_FREEZE_REVIEW.md)、[K1-3检查](K1_3_INSPECTION_REVIEW.md)、[K1-4候选协调／计费](K1_4_PREPARATION_REVIEW.md)、[K1-5驱动／自动准备](K1_5_DRIVER_REVIEW.md)及[K1-6开工规格](K1_6_SCALE_SPEC.md)，区分完整计划与已实现部分 |
| 3 | [当前架构](ARCHITECTURE.md)、[技术栈](TECH_STACK.md)、[路线](ROADMAP.md) |
| 4 | [传输／定位](PREVIEW_TRANSPORT.md)、[输入](EDITOR_INPUT.md)、[设置／外部来源](PREVIEW_SETTINGS.md)、[GPU交接](PRESENTATION_CONTRACT.md) |
| 5 | [预算v2](RESOURCE_BUDGET.md)、[P2最终证据范围](P2_FINAL_ACCEPTANCE.md)、[首轮预览边界](p0/PREVIEW_SCOPE.md) |
| 6 | 需要编辑器全局背景再读[编辑器交接](HANDOFF_EDITOR_CONTINUATION.md)、[数据兼容](DATA_COMPATIBILITY.md)及P3审阅 |
| 7 | 涉及动画才读[原始动画／元素重采样](ANIMATION_ASSETS.md)，涉及素材skill才读[独立skill交接](HANDOFF_SKIN_ASSET_SKILL.md) |

不用重新读整段旧聊天或重跑P2全部CPU归因。docs/review是2026-10-30代码快照，各P3增量另有审阅；旧证明／归档不覆盖当前新功能。

## 4. 现有能力与具体代码入口

| 入口 | 真实状态／可复用范围 |
| --- | --- |
| [crates/gameplay/src/lib.rs](../crates/gameplay/src/lib.rs)、[分步准备](../crates/gameplay/src/map/preparation.rs)、[初始化](../crates/gameplay/src/run/initialization.rs)、[存储](../crates/gameplay/src/storage.rs)、[计数](../crates/gameplay/src/work.rs)及[K1-6B证明](K1_6B_STORAGE_PROOF.md) | 独立SyntheticTest API7／profile1，Tiny4／Reference10000；变长容量、分步工作及G0已测，来源共享和只读帧／分页保持；无桌面或stable规则 |
| [desktop/preview/src/lib.rs](../desktop/preview/src/lib.rs) | 占位PreviewState／Scene、冻结及输入计数；没有谱面、对象、判定或成绩 |
| [application/preview_port.rs](../crates/application/src/preview_port.rs) | 现有PreviewPort、PreviewContext／FrameSource／FrameKey和静态PNG准备值；不是已存在的游戏运行端口 |
| [desktop/ui/src/shortcuts.rs](../desktop/ui/src/shortcuts.rs) | 产品快捷键／修饰键统一入口；文本／IME／工具隔离继续复用 |
| [desktop/infrastructure/src/audio_supply.rs](../desktop/infrastructure/src/audio_supply.rs) | 分离背景／声音、有界控制／回执、触发／暂停／定位入口；尚不能据此宣称精确游戏排程／设备消费时钟 |
| [background_stream.rs](../desktop/infrastructure/src/background_stream.rs) | 有界背景读取／缓存；普通GUI尚未接真实歌曲 |
| [desktop/host/src/main.rs](../desktop/host/src/main.rs) | Editor与模块组装／原生资源生命周期；不能从中绕过application写项目 |
| [当前架构](ARCHITECTURE.md)及[审阅清单](review/README.md) | 图层共享合成、音频候选／资源与历史；按真实源码查具体方法，不编造新的现成API |

当前domain／application／gameplay三个根核心包，桌面六包；gameplay无产品依赖且未接入桌面，compatibility尚未创建。没有新增规则集插件、跨进程服务或第二游戏窗口框架。2026-10-30审阅包保留原八包快照，不是新gameplay清单。

产品仅支持有界PCM16 WAV，44.1／48kHz单／双声道；压缩音乐只在P0实验，不是产品能力。原生证据限已声明Windows设备；Linux实测暂缓、macOS待设备，真实USB／第二输出端点变化仍硬件后补。普通图形故障冻结诊断是已接受限制，完整窗口恢复只是显式验证入口。

## 5. 不得丢失的设计约定

**项目与运行分离。** application仍是SkinProject唯一活动写入口。GameRun可以拥有可变玩法状态，但不能持有项目写权限；载入、播放、输入、定位和成绩不进入项目历史，皮肤编辑仍经Command／草稿／事务。首版完整工程历史只增不删不可覆盖，所有分支原字节仍保留。

**依赖方向。** 未来gameplay无产品依赖，compatibility→gameplay，preview→gameplay，infrastructure→compatibility／gameplay。host注入application定义的控制／观察端口；application不导入preview、gameplay、egui／wgpu或设备。渲染器／音频不实现判定，UI不持有运行器可变引用。新包必须同步更新两套检查及负向canary，不放宽边界隐藏循环。

**时间。** 内核只消费有符号i64微秒的绝对MapTime，允许负预备时间，0为音频起点；主机单调时、源帧、输出帧、生成／复制／消费位置分开。稳定锚点＋有理／余数换算避免逐块漂移；原客户端判定量化单独由profile验证。现有u64 simulation_micros及100ms增量截断不可直接升级成权威音乐时钟。GUI只取样，不推进歌曲。

**输入／C1。** [输入封口](STABLE_KERNEL_PLAN.md#c1-输入封口)包含T，当前有效来源的新输入若时间≤T则拒绝并进入冻结／故障清理、统计不完整；旧来源先拒绝，不能误使新运行失效。封口前已接纳但待消费的T输入仍执行，正常冻结保留其内部结算资格。整批基于接纳前状态核对完整载荷／序号后原子接纳，不仅凭指纹；批内去重、过期序号及窗口有明确结果。入口与消费按住态分开，恢复以当前来源可信释放重新武装，旧释放不能解除隔离。文本／IME、编辑、失焦等门控保持；自动驱动与Auto Mod分离，恢复不补跑。

**对象／事件。** 对象原始序号及谱面修订标识稳定，Future→VisibleUnjudged→Resolved→Retired，判定与显隐分开。事件有唯一键和有界消费，GUI多画一次不多发声音。未知的notelock／堆叠／目标选择不能以最近圆圈替代；超出受测准入条件整谱拒绝执行，不删掉其他对象冒充兼容。

**有界推进／C2。** [推进和事件发布](STABLE_KERNEL_PLAN.md#c2-有界推进和事件发布)分开请求目标、已完成时间截面与同刻继续位置；Complete须完成目标全部前缀，Pending不能冒充已到达。一次迁移的对象状态、账本和全部语义事件在预留容量／验证计算后一起提交；不足时无半次迁移。同刻处理一半不发布Ready投影，继续不重复执行，取消不回滚已提交玩法前缀；未激活定位候选可丢弃。拒绝连最大单次迁移都放不下的配置，不用无限Pending、丢事件或全段复制／回滚解决背压。

**K0-A截面与准备。** 首次评估开始边界S之前无Ready，推进不得早于S；检查定位以Q为新的关闭截面，不重放输入或补Miss，已过窗口结果Unavailable／Inspection，账本只计后续事件。准备／校验／检查重建也按Reference=256、Tiny=1的独立工作量分步，构造／定位共用一个有效候选位。核对操作类别、基准来源和请求代次，不能只看下一TransportEpoch；过期在途分配仍计费。输入丢失、调用错误、旧请求及临时事件背压按规格错误表分别处理。

**音效交接／C3。** [提交和回执](STABLE_KERNEL_PLAN.md#c3-音效事件提交和回执)独立于事件生成：玩法事件键为运行／传输代次＋事件序号，多声音另带稳定子序号；宿主有界关联固定声音来源、目标时间、供给请求ID及状态。trigger_selected每次生成新请求，不能充当游戏事件幂等接口。已提交等回执时重读事件不重发，超时不证明未执行；部分接纳不重发整组，未知结果经门控／清理后明确取消。按连续前缀确认消费，旧回执不复活声音，记录退休不等于PCM释放。K1仅验证纯事件身份／有界确认，K3先fake验证宿主交接，再接原生复制与清理，不把接纳当声学播放。

**冻结／定位／统计。** 冻结固定时刻，编辑可刷新、显式试听可继续，恢复重锚且不补输入／旧声音。定位用新TransportEpoch协调音频、对象与动画，候选一起发布，默认冻结；检查跳过段不生成假Miss或音效。累计score／ACC／combo不可靠时Unavailable，区段／自动／样例来源可见；最终跳转HUD默认方案仍待讨论。

**皮肤交接。** 皮肤热替换、草稿取消、Undo／Redo／Checkout不得重置歌曲／对象／已判定状态。图片、独立音效与背景各自门控，核对自己依赖的完整来源；同源显式试听增益例外继续沿用。一个圆圈的多个部件可复用同一文档，但图层变换不改变命中点／半径。GUI区域、变换框、玩法判定区不同。

**外部资源。** 用户选定stable／Songs只读来源属于本机设置，先校验根／真实路径／重解析边界／限额；谱面／歌曲不进AssetLibrary、.oskproj、Git或皮肤导出。默认素材发布须自制／获授权；旧stable代码／附带资源不得进入产品、实验依赖或fixture。

**格式／动画。** 运行器不要求改工程schema；持久设置或必要项目语义确需演进再单独定版本／迁移。历史短地址继续与永久身份分离，不借用为对象ID。动画来源保留原始时长、按元素规则输出的AN设计不变，程序化接近圈动画不等于已实现动图导入。

## 6. 下一增量、检查和证据

当前停靠点是[项目全面检查](PROJECT_REVIEW_20261109.md)，Query-v1实现与一致性审阅已完成，分别保存于019a296／856cabd；本轮轮前备份为6ac5fda。现有计划是独立16记录的QueryOnly协议：baseline9、查询建立／3批次／释放5、summary和end；三类各16样本、外层参考各16，共96个新时间槽。查询组无单调用计时，1MiB实际容器准入，复用64项页缓冲；每样本核对完整结果、FrameKey、WorkCounts差值和资源不变。Reference10000两预算／三轨迹及真实Tiny4三轨迹已有无窗口功能回归，正式成本尚未运行。

冻结入口复制Cargo真实产物到新目录frozen/probe.exe，关闭并核对后独占发布freeze.json；正式测量时逐子进程核对源码／副本，全部收尾后依序非覆盖发布report.json与evidence-index.json。只读--verify核对角色、路径、字节数、SHA及逐条重算结果，不构建、执行、采样或读取当前源码。Python临时假证据只作回归。剩余为PC三批次／完整读取器与回归审阅、正式完整12进程新轮、独立退出决定；Query-only正式测量不是必须另跑的前置，旧Query结果不得拼接。当前coverage保持NeedsEvidence，既定合法Unknown保留；实现／审阅事实见[Query修订记录](K1_COST_CLOSURE_SPEC.md#2026-11-09查询增量一致性审阅)。

规格审阅已备份为f1c8dbf，正常完整hook验证旧API5／116项及桌面默认／全feature无窗口基线；随后实现[K1-6A](K1_6A_STORAGE_REVIEW.md)，API6／profile1、4对象。新增实现经正式check_core通过424项Python检查回归、格式、Clippy、三个核心包测试及release构建，gameplay共132项；桌面architecture-only通过。没有成本测量／窗口／设备接线，日志在本机忽略目录artifacts/k1-6a/。过程中一次夹具目标错误及换行整理误清空本轮源码均已修复，恢复／黄金内容核对和最终正式检查通过，历史日志保留。

K1-6A已完成本次一致性复核，补5项回归至137项并通过正式check_core，含424项Python回归、格式／Clippy、三个核心包无窗口测试及release；桌面architecture-only通过。核对共享来源真实最后所有者、候选独立账本、同刻Hit／Miss背压与故障下的旧完整帧／旧页，以及首次发布失败。没有改运行算法或黄金预期，API6／profile1及四槽计费保持。新增日志在artifacts/k1-6a-consistency/，本次修订未提交，后续先备份。帧键依赖既有RunId在旧消息仍可能到达时不复用的组装者契约，现明确包括脱离借用的帧键／游标；静态额度不等于进程内存，受控拒绝不等于任意OOM恢复。B的变长布局／分步索引及完整原始工作计量、C的10000轨迹、D的外部成本基线分别待实施，不自动推进。

读取须同时保留实时已提交单项与T处完整帧；Pending不丢既有完整帧，也不将新判定／实时计数混入旧T。只读分页核对来源／发布键及有界输出，确认事件不换键，来源换代／新截面拒旧页。工作量除steps外记录读取／初始化／索引比较搬移和管理计数；准入预留、实际请求容量及存活所有者分列，不能相加冒充内存。相同修订令牌的独立分配不去重，共享来源一次、独立账本各计，取消资格后仍持有则不退额。成本输出另有版本及Unknown／Invalid状态，不凭低读数或缺失样本通过。

KT23固定10000圆圈生成式：ObjectId=i、目标i×1000000µs、S01偏移，从-200001至9999300000。G0预期10000 Miss／10000 Judgement，G1固定脚本每批最多32对象／64边沿，序号2i+1／2i+2，预期10000 Hit／20000事件；A1经S冻结／切换／恢复，序号1为释放事实，脉冲2i+2／2i+3。三轨迹及来源／最终账本／页／控制交错已有通过证据，SoundIntent不触发设备。成本阶段已有有效基线，短调用批次补证尚缺，不把规模或协议Valid当成产品预算或完整KT23通过。

| 门槛 | 必须准备 | 当前边界 |
| --- | --- | --- |
| K0-A → K1 | 固定SyntheticTest范围／值模型，C1／C2及C3纯事件约定，有限配置与拒绝结果，S01正例和S02–S04边界／交错／小预算的完整预期及KT映射 | 只允许合成预编译圆圈；没有.osu／设备／GUI依赖，不声称stable兼容 |
| K0-B → 可执行兼容profile | v14／Mode0逐字段支持矩阵，必需规则逐项来源／版本／数值边界／未知及fixture，R01非空自制兼容候选和R02明确拒绝样例 | 可以独立推进解析／拒绝诊断；必需未知的profile仍Blocked，不以过滤对象或合成规则放行 |

S01工程数值和Hit／Miss预期以主方案为准，不是stable窗口／半径／分值。S01–S04适用纯核心、Reference10000／Tiny4容量、G0／G1／A1及有限控制交错已有通过证据；KT23成本批次待补。R01另须必要规则依据，R02整谱拒绝不支持对象／未知语义／歧义布局；同点同刻身份可表达不代表已验证布局，没有新增真实谱面或外部资源fixture。

证据至少区分SyntheticTest、公开文档依据、当前客户端已验证和Unknown；逐项覆盖格式／属性默认、AR／CS／OD、边界／顺序／堆叠、皮肤呈现及声音选择。客户端条目记录真实build／设置／输入与结果，不能从旧源码、lazer或合成通过自动升级；静态F12仅在已记录范围内验证外观，不证明时序。K0-A不要求把所有stable未知项解决后才做工程模型；K0-B不能用S01补齐必需未知或宣称空的兼容范围已可执行。

建议K1每次只处理一个可审查增量：

1. K0-A满足且用户明确要求后，定义强类型时间／坐标／身份、不可变圆圈输入、显式SyntheticTest规则和有限配置；增加gameplay与检查规则。
2. 先验证KT05／KT06、KT14适用纯核心项及C2：完成截面／继续位置分离，迁移预留不足无半次状态，绝对时间分段不变，事件稳定重读和有界确认。
3. 接带时间戳的两路输入／自动驱动，覆盖KT08／KT09及C1的封口前／等于／后、已接纳待消费、重试／冲突、顺序／门控及超限。
4. 补冻结／恢复／检查定位、旧代次及资源所有权，覆盖KT10／KT11，记录10000圆圈成本基线。
5. K1审阅后再加K2纯解析器／profile，可执行兼容范围须满足K0-B；K3先fake验证C3提交／部分接纳／回执去重、迟到／未知／换代及停止路径，再做编辑器／WAV／材料接线，之后K4手动圆圈对照和K5原生稳定性；不跨阶段宣称完成。

以上是分步路线：K1-1至K1-5已经实现适用四对象功能，K1-6A／B放大存储及有界工作，C补万对象轨迹与控制，D建立独立成本基线并完成工具一致性修订。整体退出审阅暂未签收K1，只留下成本批次补证工作组；收口规格和Query部分均已审阅，后续按明确任务完成PC／完整计划、正式新轮与独立退出决定。真实谱面解析仍须K0-B，原生接线仍须C3提交／回执与时钟能力。K3风险保持：eframe布局前输入捕获／封口、背景复制锚点与真实消费位置差异、静态元素到多圆圈绘制；不把GUI帧时间当硬件时间或即时trigger当精确排程。

现有检查入口：

```text
python -B tools/check_repository.py
python -B tools/check_core.py
python -B tools/check_desktop.py
```

Rust1.95.0，工具链路径与本机缓存按README核对；不得静默下载／改全局工具链。新增包同步两个Cargo工作区的实际依赖检查、源码规则和禁止边的负向回归。默认检查无窗口、无GPU／声卡、无网络／UAC；原生／人工测试另经明确任务执行。

继承零欠载、已就绪短音完整请求到原生复制观察p95≤20ms、进程内存／GPU及实际释放门槛；播放单核CPU≤15%为优化目标，稳定空闲硬项不豁免。20ms不是设备消费、声学或判定误差指标；到期→接纳的宿主延迟另外报告。新内核先fake／门闩再短原生，最后当前产品入口两轮300秒／20次循环；不能套用P2旧证据直接判新路径通过。

K1-1至K1-5已运行适用纯核心回归，完整KT仍待验收，具体检查与未测范围见增量审阅。前置回退hook验证的是备份时既有代码，不证明之后新增模型；新增模型另经check_core验证。不重写旧_PROOF.json或清理历史失败来制造通过。

## 7. 可粘贴的新对话开场

```text
请在 E:\skin-editor 接手独立stable兼容内核规划。先读AGENTS.md、docs/HANDOFF_STABLE_KERNEL.md、docs/STABLE_KERNEL_PLAN.md及当前Git状态／HEAD／暂存差异，按最新规则保护上一轮已审查工作，核对私有origin并保留其他对话修改，不并发Git写入。

首批仅osu!standard／No Mod／纯圆圈受测子集。gameplay API7／profile1、Tiny4／Reference10000，G0／G1／A1及有限交错已测；成本工具独立std＋gameplay，时钟／I/O不进入核心。先读docs/K1_COST_CLOSURE_SPEC.md的6批次、32样本容量、候选键公开接口限制、Unknown及冻结保管，再读K1_EXIT_REVIEW.md和D／C／B／A、K1-6、K0-A及C1／C2／C3。当前成本工具为schema3／CostClosure-Query-v1；完整CostClosure-v1仍待PC三批次，历史v1／v2保留原样；compatibility未实现，不用合成数值或旧源码补未知stable规则。

按我本轮明确任务执行；交接不授权后续开发、私人谱面、P3保存UI或窗口／设备。2026-11-09已完成项目全面检查，先读docs/PROJECT_REVIEW_20261109.md；轮前CI修正已备份6ac5fda，本轮审阅文档未提交。下一轮照既定Git流程备份，再执行明确任务。Query-v1实现与一致性修订均已完成，不重复首次规格／实现审阅；保持Reference10000／Tiny4、API7／profile1。PC三批次／完整CostClosure-v1与正式新成本测量未开展，建议有限收口K1后优先审阅P3保存闭环；只是建议，不自动开工。coverage仍NeedsEvidence，旧证据保留，不签收KT23／K1、产品性能或stable兼容。

保持完整历史不删不覆盖、图片与音效独立、皮肤热替换不重置音乐／对象、冻结恢复不补跑及旧代次清理。零欠载、提交p95≤20ms、内存／GPU／实际释放门槛保持，播放CPU≤15%为优化目标。新原生路径和兼容性须另验，合成小模型测试不能替代这些证据。
```
