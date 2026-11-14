# P3-2A A1：版本、固定控制槽与真实捕获关联

2026-11-15后续保管：本报告所述A1已正常hook提交并推送abd879dc6dd03d6d1ea46ce722ea0592d436b7f9，[同提交三项CI](https://github.com/sumiyuki727/skin-editor/actions/runs/43615335146)成功。用户随后明确授权A2，其当前实现／容量变化另见[A2报告](P3_2A_A2_REVIEW.md)；下方A1数值及“未提交／待A2”保留当时事实，不作为当前进度。

2026-11-14。按[开工契约修订2](P3_2A_START_CONTRACT.md)实施A1；轮前10份契约及同步文档已通过正常完整hook，备份并推送为1661c84e1a342f90d70bd84212cfa31d0b8a8d51，[同提交三项GitHub检查](https://github.com/sumiyuki727/skin-editor/actions/runs/43483039794)全部成功。本文和本轮源码是该提交之后的工作区增量，不将这次远端CI套用于新增源码。

A1实现的是无窗口、仅准备的保存协调能力：application API7、捕获3、协调端口1及SaveCoord-Control-v1。存储端口1、模型2及全部持久记录版本保持。无真实文件、GUI或原生设备接线；K1、Reference10000／Tiny4、产品性能NotEvaluated与stable兼容Unknown保持。本轮不进行正式成本测量。

## 1. 实现与不能混同的结果

| 内容 | 实际入口／所有者 | 本轮结论 |
| --- | --- | --- |
| 显式安装 | [ProjectSession保存协调](../crates/application/src/save_coordination.rs)的install_save_preparation | 默认Unavailable；只有显式安装后具备PreparationOnly。没有默认成功替身、目标重绑定或文件写权限 |
| 值与版本 | [值契约](../crates/application/src/save_coordination/values.rs) | SaveRequest携带API／捕获／协调版本；Ticket绑定完整运行来源、目标、持久身份、捕获、覆盖、提交候选、expected链及origin。非法零身份、混合expected字段、未知版本拒绝；修订0合法 |
| 单槽准入 | request_save | 接纳时冻结真实HistoryCapture；保存序号只在成功接纳后消费。先识别精确在途重复，q+1不影响Existing；改载荷Conflict，另一请求Busy，释放后的旧key Stale；无无限去重表 |
| 真实捕获入口 | [capture_history](../crates/application/src/capture.rs) | 捕获3在真实入口返回SaveOutstanding；与Draft Busy及最后消费者InFlight区分。所属保存在槽尚空时经同一入口捕获，不提供外部bypass |
| 脱离会话的工作 | [SavePreparationWork](../crates/application/src/save_coordination/work.rs) | 不持有活动ProjectSession；只暴露真实分步poll、固定Ticket、统计及提前保留的Snapshot。不能取得内部HistoryCapture交给现有文件写入器 |
| 终结结果 | CompleteNotSaved／StoppedNotSaved | 只说明遍历完成或确定停止。没有ArchiveReceipt、保存标签、真实覆盖事实、NoChange、Unknown处理或关闭许可 |
| 取消与确认 | cancel_save／acknowledge_save | 不申请普通RequestId或新控制分配。结果确定、Ack和真实释放分别核对；任一缺少仍保留槽。错误／旧资格不释放当前任务 |
| 恢复来源 | [RestoreCandidate激活](../crates/application/src/restore.rs) | 私有restored_history保留来源，拒绝把无last_commit的恢复历史当首次发布；仅未保存捕获形成的永久映射不等于已发布 |

Automatic明确UnsupportedOrigin；没有Close归属的Exit返回NeedsClose。CloseTicket、CloseFinishPermit、ReconcileTicket及PreparedWitness只落实有界载荷／保留位置，A1不发放收尾许可、不启动核对、不发布内容见证。不能因这些类型存在而宣称A2／A3已完成。

ExistingVerified在A1只核对注入上下文与真实会话的持久身份和已恢复元数据；不验证磁盘路径、文件身份或介质。NewUnpublished允许同一未发布历史的后续准备，但不执行发布状态转换。文件代次为u64最大值仍允许纯准备，绝不取得新增写权限；VerifyOnly／RequiresWriteCapacity的语义在A2落实。

CommitId仍取自HistoryIdentitySource，拒绝非法值及已知持久身份、节点、保存记录和上一准备候选的碰撞；调用方必须履行既有“新鲜候选”契约。A1不构造无限候选去重表，也不证明全宿主随机身份新鲜性；完整文件请求摘要／ID复用检查在B、运行会话准入在C。

若真实capture_history已成功、但随后CommitId候选被拒绝，既有捕获语义允许保留已经建立的永久身份映射；它不是项目内容修改、历史节点或保存事实。接纳key、普通命令回执和内容状态仍不变，捕获及控制所有权实际释放后可用同一未消费key再试，不能把候选号消耗解释成成功保存。

## 2. 512KiB静态容量证明

证明实现为[budget.rs](../crates/application/src/save_coordination/budget.rs)。八个固定lane分别对齐到16字节，编译期断言owner尺寸等于八lane之和；每lane最多64KiB，普通／保留各256KiB。安装前必须一次预留完整512KiB profile，差一字节即拒绝，不能因为当前布局较小便借用保留分区。A1不按占用缩减这份预留，也不在Cancel／Ack时虚报仍存活资源已归还。

下表来自Windows x64、固定Rust1.95.0的size_of／align_of算术，不是成本测量。数值包含类型padding，显式生产／信箱／消费副本按三份保守计，唯一共享分配只计一次。其他受支持目标须各自通过同一编译期断言及回归。

| 分区 | 静态上界（字节） | 包含内容 |
| --- | ---: | --- |
| 普通常驻／绑定 | 1072 | Resident、owner指针、PortState共享分配、端口句柄、安装配置及证明值的临时副本 |
| 普通保存槽 | 2280 | SaveSlot、TaskControl共享分配、保存取消块、请求／Ticket交接副本 |
| 普通信箱 | 4448 | 持有的HistoryCapture头及三份工作／完成载荷；整个上界低于32KiB，因此各自32KiB容器亦满足 |
| 普通事实／摘要 | 3440 | 独立见证位置、候选句柄、三份状态查询值及见证副本 |
| 保留关闭 | 592 | 固定CloseTicket／Permit位置、关闭序号及载荷副本；尚未执行关闭 |
| 保留核对 | 1656 | 单查询位置／attempt、独立首个核对取消块及资格副本；尚未执行核对 |
| 保留失败／Cancel／Ack／释放 | 1955 | 有限终结、确认和完成值副本；没有动态错误队列 |
| 保留最后保护交接 | 2096 | 固定最终请求／Ticket／Close位置与副本；A1只保留资格，不启动最终保护 |
| 合计 | 17539 | 普通11240、保留6299；各自不越界，合计小于524288 |

owner实际布局3472字节，新增唯一共享控制布局合计208字节；二者均已包含在分区上界中，不能再次相加充当额外余量。载荷上界：Save／Reconcile Ticket最大400、PreparedWitness560、状态576、诊断80、工作封套864、完成封套480字节，分别低于1KiB／4KiB／128B／16KiB契约上限。没有分配512KiB填充缓冲来伪造计量；512KiB是不可互借的准入预留，17539是当前受管布局的保守证明值。

共享块按固定工具链的[标准库原始布局](https://github.com/rust-lang/rust/blob/59807616e1fa2540724bfbac14d7976d7e4a3860/library/alloc/src/sync.rs)计算：两枚原子usize计数、payload及对齐padding；不读取私有内存、不复制标准库实现，不视为稳定ABI。工具链或字段／分配点变动必须重新核对。此证明不包括系统分配器元数据、碎片、编译器栈溢出保存区或RSS，不能称为实际进程RAM。

### 所有权、分配点与保管

固定控制分配仅为一份Coordinator box、一份持久PortState、一份在途TaskControl及两份独立取消块。两个取消块在接纳阶段创建，旧取消token不能复位／再利用；首个查询对象的存储与资格有固定保留位，但查询状态机尚未实施。普通保存最多用到u64::MAX−1，最后值保留；Cancel、Ack和已有任务终结不再消耗身份。关闭序号／后续attempt的完整耗尽行为仍是A2／A3门槛。

[源码边界检查](../tools/check_core_architecture.py)禁止协调模块的变长控制容器，并限制新增分配点；[负向回归](../tools/test_check_core_architecture.py)覆盖嵌套模块、容器及未计费分配。它是审阅门禁，不是通用Rust内存安全沙箱，不能代替类型尺寸、实际所有权和回归。

Snapshot在首次poll前由协调器保留；worker另持独立引用。终结后协调器在控制动作中释放不再需要的重型捕获；只读状态查询本身不做全树工作或副作用。worker的释放守卫是最后一个字段，其通知晚于捕获、Snapshot和TaskControl句柄Drop；独立CapturedNode／Snapshot继续维持既有capture_in_flight。只有Ack、工作真实退出和最后捕获消费者退出全部满足才能重新接纳。即使捕获槽已空，未Ack的保存槽仍返回SaveOutstanding。

PreparedWitness保留槽属于协调所有者，独立于worker／完成消息；A1不构造或发布见证。事实候选目前只有固定句柄位置，没有新增ArchiveMetadata分配。**在A2实际交付任何MayWrite之前，仍须准备完整见证及最多1MiB逻辑候选，并另证实际分配capacity／所有权；本轮不能替代这项门槛。**A1不交付MayWrite，因此不能发生“已写入却没有候选保管”的路径。

捕获模型／manifest／原字节租约的动态存储使用既有逻辑限额，不能塞入上述17539或512KiB后称为实际RAM。SaveCoord-Fixture-v1采用64节点／256对象准入／8原始资源、原字节总256KiB、元数据1MiB、单状态256KiB及累计逻辑状态16MiB；测试Session为64节点／128普通回执／16命令，自制资源单份不超过64KiB。黄金树实际5节点／4对象准入／3原始资源，另有q之后的第6节点和共享原字节的新准入；不把各上限当作可同时兑现的笛卡尔积。64节点满额与63限额对同一完整历史的拒绝有单独回归，不外推产品容量。

捕获消费者释放不回收历史仍持有的原字节。黄金回归分别检查消费结束后资源仍存在、会话最终释放后原始分配只销毁一次。完整Session成本、真实codec／文件／恢复展开、旧新会话共存及宿主分配器证明继续由B／C承担。

## 3. 回归对应关系

[公开入口回归](../crates/application/tests/save_coordination.rs)使用真实ProjectSession、Command、草稿、历史导航及HistoryCapture；测试资源池只管理自制内存字节，工作由测试驱动明确取走／poll／Drop，无sleep、网络、时钟、窗口或设备。另有[内部边界单测](../crates/application/src/save_coordination.rs)仅用于接近u64最大值及独立取消块的保留证明。

| 适用契约 | 实际测试名／观察 | 本轮边界 |
| --- | --- | --- |
| PA01／02／29 | versions_unavailable_and_unsupported_origins_do_not_capture_or_consume_commands；zero_revision_is_legal_but_wrong_source_and_target_are_not | API7／捕获3／协调1、非法版本／零身份、合法零修订；Closing部分尚未实施 |
| PA03／04／28／37 | real_frozen_golden_tree_keeps_branches_admissions_originals_and_raw_configuration_after_q | 全分支、瞬时创建后删除的准入、未引用素材／不透明音频、未知配置原文／CRLF、原字节范围；q后编辑／导航／草稿不受准备结果影响。真实保存确认仍欠项 |
| PA07／29 | active_draft_and_external_capture_are_busy_without_identity_or_receipt_cost；worker_drop_before_ack_releases_capture_but_keeps_save_outstanding | Draft、独立捕获、未Ack保存分别阻挡；不改变普通回执 |
| PA10／27／33 | wrong_port_ticket_and_session_do_not_change_the_active_task；two_runtime_sessions_with_the_same_project_and_head_reject_each_others_controls；repeats_match_original_payload_before_live_revision_draft_or_port_liveness | 逐字段错资格、不同端口及两个真实会话隔离、q+1／草稿／端口失效后的精确重复、Conflict／Busy／Stale；不验证文件事实或全宿主新会话 |
| PA15／16／35 | pending_cancel_is_not_saved_and_does_not_require_a_worker_or_new_request_id；working_cancel_abandon_and_invalid_poll_are_finite_not_saved_results；ack_waits_for_worker_snapshot_and_node_owners_then_old_keys_cannot_release_new_slot | 仅准备阶段取消／丢工作／失败；Ack与Drop两种顺序、独立消费者；可能已提交的Unknown仍欠项 |
| PA24／25 | ordinary_receipt_exhaustion_does_not_disable_cancel_ack_or_change_old_history；invalid_profiles_and_binding_pairs_reject_before_installation | 普通回执满额不锁死控制清理；512KiB差一字节拒绝、expected配对与fixture上限；不替代关闭草稿收尾或候选预算验收 |
| PA25／35 | static_control_layout_accounts_for_all_lanes_and_bounds_public_carriers；capture_limit_failure_is_atomic_and_does_not_consume_the_admission_key；late_state_limit_failure_is_terminal_but_waits_for_actual_worker_drop；fixture_full_node_capacity_and_one_less_limit_have_atomic_distinct_outcomes | 静态计费、准入前失败、分步状态超限、64／63节点边界及实际Drop |
| PA26／38 | final_save_identity_is_reserved_without_wrap_and_cleanup_needs_no_new_identity；first_query_token_is_preallocated_and_independent_not_a_reset_cancel_token | 保存序号最后资格／独立首核对token；关闭、追加attempt及实际Unknown核对待A2／A3 |
| PA30 | an_unsaved_permanent_map_is_not_mistaken_for_a_published_file；restored_history_without_commit_metadata_cannot_be_published_as_new；existing_binding_requires_full_restored_scope_and_exact_expected_chain | 未保存映射／恢复来源／既有链分别验证；首次真实发布转换待A2／B |
| PA02／26 | failed_commit_candidate_releases_capture_and_allows_the_same_unconsumed_key | 非法／已知冲突候选不占槽、不消费接纳key、不改历史；不证明无限跨会话去重 |
| PA16／28 | dropping_session_cancels_detached_work_without_access_to_live_project | 所有者退出使独立工作取消；旧Snapshot仍可读，不借工作保留活动项目写入权 |

PA05／06／08／09／11–14／17–23／31／32／34／36的完整语义，以及上表每组的事实确认、核对和关闭部分继续未实施。不能将A1的局部通过改写成38组全通过或P3-2A签收。

## 4. 检查、保全与交接

本轮在Windows x64／固定Rust1.95.0完成以下检查，均成功退出：

| 检查 | 结果与证据 |
| --- | --- |
| tools/check_core.py | 完整格式、Clippy、无窗口测试、514项Python检查（新增2项控制存储负向回归）、release及工具无计时贯通通过；final-core.log |
| 最终A1源码定向复验 | 全核心格式／源码边界及application全部target的Clippy通过；公开回归22项全部通过，含最后补充的两个真实运行会话隔离；final-focused.log及final-source-check.json |
| 内部边界／编译隔离 | 2项内部边界单测、1项新增compile-fail通过，记录于完整核心日志；不以过滤后0项运行冒充通过 |
| tools/check_desktop.py | 默认与全feature的格式／Clippy／测试及最终宿主构建通过；final-desktop.log，未打开窗口或GPU |
| 文档与工作区 | 11份文档、482个本地链接／59个锚点、PA01–PA38／AR01–AR10及预算算术通过；829文件候选扫描、git diff --check通过；final-result-check.json |

完整核心脚本已覆盖当时的21项公开A1回归；最后新增跨会话用例后，再对最终源码执行上述格式／源码边界／Clippy和全部22项公开回归。运行时代码与工具规则没有随该用例再修改，最终散列核对一致。验证摘要及三个日志散列记录于verification-completed.json；这些证据都位于本机忽略目录artifacts/p3-2a-a1-20261114。仓库强制的既有无计时回归没有产生新的K1正式成本样本或产品性能结论。

最终新增／修订共22个文件（16个既有、6个新增），暂存区为空；其余807个跟踪文件的原字节及mtime与轮初基线一致。schema.rs散列仍为713f3850740db024c4b861327de560f4eb81f5820980d1f89672dcdcfd499125，其既存空差异标记未被暂存或修正。

本轮源码／文档未提交；下一次经用户明确恢复工作时按仓库规则备份，不自动进入A2。旧测量原件、冻结程序、恢复附件及schema.rs既存字节／空差异标记保留，未读取密钥或更改PUBLIC主仓库／PRIVATE恢复材料安排。3435405固定恢复快照不会自动包含本轮新文件。

A2仍须实现完整脏事实投影、PreparedWitness实际发布／验证、确认候选、NoChange／Unknown与事实安装；A3实施关闭门控、现有草稿专属收尾及其余PA。B–D、真实文件闭环、可视化配置V0–V3的顺序不变。本轮完成A1后停止，等待用户明确指令；不提前开始A2或扩大结论。
