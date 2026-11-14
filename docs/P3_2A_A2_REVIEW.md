# P3-2A A2：保存事实、见证与实际释放

2026-11-15。用户在本对话明确确认开始A2，按[开工契约修订2](P3_2A_START_CONTRACT.md)实施事实投影、PreparedWitness／候选保管、NoChange／Unknown、确认和实际释放。轮前[A1](P3_2A_A1_REVIEW.md)已正常hook提交并推送abd879dc6dd03d6d1ea46ce722ea0592d436b7f9，[同提交三项GitHub检查](https://github.com/sumiyuki727/skin-editor/actions/runs/43615335146)全部成功。该备份CI不是A2新代码的验证。

本增量仍是无窗口应用用例。API7／捕获3／协调端口1、存储端口1、模型2及全部持久记录版本保持；无新依赖。A3关闭、真实文件／恢复区／发布、GUI和设备均未实施。K1及Reference10000／Tiny4不变，产品性能NotEvaluated、stable兼容Unknown，不运行正式成本测量。

## 1. 实际入口与信任边界

[协调器](../crates/application/src/save_coordination/protocol.rs)增加显式install_save_protocol、take_save_execution、advance_save_coordination、save_protocol_status及同槽核对入口；保留A1的PreparationOnly接口和行为。默认Unavailable，无默认成功替身。SaveProtocolAuthority是不提供生产构造器的不透明组装资格，当前只有cfg(test)的受控测试签发器。因此生产构建虽然包含值和协调实现，**尚不能启用真实保存端口或产品保存按钮**；B必须另行冻结证明profile、目标校验和真实适配器准入。

工作不持有活动ProjectSession。独立CompletionSender只发布到原任务一次性单元，不能通过单独构造ArchiveReceipt或布尔值给项目标记已保存。advance只消费有资格的有界观察；只读status不推进捕获、消费结果、分配候选或遍历历史。外部复制的值不是控制资格。

[事实投影](../crates/application/src/save_coordination/protocol_values.rs)分别暴露最新、Manual、保护、已确认截面、当前完整范围、PendingDraft和绑定冲突。CoverageStamp是不可变历史／对象准入前缀及精确运行来源的本机关联，**不是内容摘要或文件证明**。q确认不修改当前头、修订、普通请求回执或已提交节点；q之后的新分支／资源／导航继续待保护，新草稿仍PendingDraft。未来持久标签写入口须同步推进持久元数据来源，不得借现有stamp漏计修改。

已验证打开基线单独保管，须匹配真实RestoreBuilder恢复来源、初始运行来源和元数据；当前测试资格不等于真实文件验证。Manual A／保护C、默认打开A及推导Redo可以保留A／C；真实导航后不能复用原打开等价。合法无Manual的Latest恢复及新空根，第一次Manual必须形成新提交。仅树已在文件里不足以满足Manual意图。

## 2. 准备、确认和Unknown

[执行载体](../crates/application/src/save_coordination/protocol_work.rs)在真实捕获Complete之后发布不可变PreparedWitness；核对版本、profile、完整Ticket，重复相同证明幂等，改变任一承诺拒绝。见证独立于工作／完成消息存活，覆盖完整内容、原字节、元数据／导航、请求及基线。发布本身不是保存事实。

测试fake流式读取实际捕获状态，并与独立保留的黄金状态、父关系、全部对象准入、自制资源池的实际原字节、配置及持久元数据逐项比较。测试profile的有界非加密承诺补充这些比较，使用Debug拼写，不是持久codec、文件摘要算法或抗篡改证明。相同head／计数／Ticket但内容不同的证明不能互换；真实算法仍由B冻结。

最多一个[确认候选](../crates/application/src/save_coordination/candidate.rs)在工作交付前完整准备，保留旧标签、扩展原字节和Manual标记，预先追加本次潜在事实。begin_write还要求已有见证、未取消、无完成及候选可用；返回借用执行对象的许可，不能带出执行对象寿命。确认仅替换预备Arc，不分配元数据、不执行Command。首次确认把原绑定原地转为ExistingVerified，保留绑定id／generation及原Ticket的Ack资格。

候选逻辑／实际capacity不足或文件generation耗尽时为VerifyOnly，无写权限。它仍可验证已有NoChange；确需新增则RequiresWriteCapacity，读取失败保持ReadFailed／Unknown。NoChange保留已有Manual／保护／metadata，不追加本次CommitId、代次或标记。已确认提交的重放不重复标记，发现后续冲突保留已确认事实并锁定绑定。

可能写入后没有合格完成，进入Reconciling；Unknown不是可Ack抛弃的失败。查询等待原写执行／scratch实际退出，刻意保留的Snapshot／证明／候选不阻塞同槽核对。首次QueryControl及独立取消token在接纳时预分配；后续attempt只在旧查询真实退出后重新分配，checked序号不回绕。保存取消不取消查询，查询取消／读取失败／目标缺失／仅None不清除Unknown。

四类核对观察均有回归：找到原提交；独立证明NoChange；同目标完整验证未提交且未覆盖；证据不足继续Unknown。主完成迟到可先确认，查询仍须退出；查询取消或冲突不能抹去已确认事实。Ack、完成消息、工作退出和最后捕获消费者释放分别核对。各释放守卫是最后字段，只持常驻端口及身份，不持它正在宣告释放的任务／查询分配。

取消、Ack、已接纳任务确认和首次核对不申请普通RequestId。取消前已Complete但尚无写许可的执行退出，正确分类为Cancelled；本轮先用失败回归复现了误分类Abandoned，再修复。已确认后迟到的合格“确定未提交／未覆盖”观察构成矛盾，保留事实并锁定绑定；本轮也先复现漏锁绑定再修复，不把它与单纯取消等同。候选转入活动metadata后仍占真实内存，installed_metadata保留成本，不把任务Ack说成已释放该分配。

## 3. 容量证明与边界

[静态证明](../crates/application/src/save_coordination/budget.rs)仍要求完整512KiB准入，普通／保留各256KiB、八lane各64KiB。以下为Windows x64／固定Rust1.95.0的布局算术，包含padding、生产／信箱／消费副本；不是计时、RSS或分配器峰值测量。

| 分区 | 受管静态上界（字节） |
| --- | ---: |
| 常驻／绑定 | 1872 |
| 保存槽 | 4360 |
| 工作／完成交接 | 9392 |
| 事实／终结及候选临时头 | 7840 |
| 关闭预留 | 592 |
| 同槽核对 | 6288 |
| 失败／Ack／归还 | 5123 |
| 最终保护预留 | 2096 |
| 合计 | 37563 |

owner为3744字节，唯一共享控制分配共2728字节；Ticket400、Witness560、状态1488、诊断80、工作封套1296、完成封套1120、查询封套864、CompletionSender416字节，均受编译期上限断言。最多一个任务、一个预留／活动查询、三个不同取消块（工作、原首次token、后来attempt token）；旧查询未释放不能分配下一查询。普通静态合计23464、保留14099字节，不降低512KiB保留要求。

候选另列逻辑上限1MiB，受管分配上界为1MiB＋Arc<ArchiveMetadata>控制块布局。新marks／tags／extensions向量以及每个字符串／原始bytes均先计算完整所需容量，再用空容器try_reserve_exact一次预留，填充不增长，逐个对账capacity；共享旧metadata不借给新分配。各元素大小小于其64字节逻辑系数有编译期断言。固定头的临时副本计入控制lane。

依据为固定Rust源码[RawVec精确增长](https://github.com/rust-lang/rust/blob/59807616e1fa2540724bfbac14d7976d7e4a3860/library/alloc/src/raw_vec/mod.rs)和[Arc布局](https://github.com/rust-lang/rust/blob/59807616e1fa2540724bfbac14d7976d7e4a3860/library/alloc/src/sync.rs)，不复制其实现，不声称私有布局稳定ABI；换工具链须重核。系统分配器开销、取整、碎片、完整栈及整个Session成本不在证明内。可失败buffer预留返回有限诊断；标准库Arc的进程OOM中止不在可恢复错误保证内。

捕获仍用SaveCoord-Fixture-v1的独立逻辑预算。候选恰为容量／差一字节的两种限制分别回归；1MiB候选上限不能挤占捕获manifest的1MiB额度，各上限不保证同时可兑现。原字节由历史、捕获租约及池共享；捕获最后释放不回收历史仍固定的字节。测试没有放大产品容量或更改Reference／Tiny。

[源码门禁](../tools/check_core_architecture.py)仅放行candidate.rs的受审精确预留点、protocol.rs首个／后续查询分配点；其他控制模块继续禁止可变容器。新增两项负向回归拒绝仿冒子路径、额外容器／增长及未核算查询分配，未放宽模块依赖。

## 4. 本轮回归与未覆盖部分

[A2回归](../crates/application/src/save_protocol_tests.rs)使用真实Command、草稿、导航、RestoreBuilder和捕获；无文件、时钟、线程、窗口、设备、网络或个人目录。以下是适用纯用例覆盖，不能解释为整组PA或SL产品验收。

| 适用PA | 实际测试名（可在源码检索）／范围 |
| --- | --- |
| PA01／29／31 | protocol_is_unavailable_without_qualified_authority_and_no_default_success；versions_missing_witness_and_new_empty_root_cannot_manufacture_no_change；版本、资格、缺见证、新空根 |
| PA03–06／28／37 | full_resource_golden_confirms_only_q_and_capture_release_does_not_free_history_originals；q_confirmation_keeps_new_branch_navigation_and_new_draft_pending；完整5节点／4准入／3原字节黄金树，q后第6节点及新资源／导航／草稿 |
| PA06／08／09／30 | first_manual_commit_installs_prepared_metadata_without_content_or_receipt_changes；replay_and_late_main_completion_append_the_manual_mark_only_once；首次绑定转换、Arc安装、重放及原Ticket |
| PA10／27／31 | prepared_proof_is_immutable_and_all_commitment_fields_are_bound；same_head_counts_and_ticket_cannot_substitute_another_frozen_content_proof；golden_verifier_rejects_equal_count_mutations_before_publishing_proof；wrong_generation_or_request_context_cannot_install_a_receipt；foreign_port_and_old_runtime_ticket_cannot_take_current_execution |
| PA11／12／36 | default_manual_opening_no_change_preserves_newer_protection_and_derived_redo；real_navigation_cannot_reuse_the_unchanged_opening_equivalence；first_manual_after_latest_without_manual_requires_a_real_new_manual_fact；existing_no_change_survives_candidate_limits_and_generation_exhaustion_without_write_permission；verify_only_requires_write_and_read_failure_have_distinct_outcomes |
| PA13–16／32 | writer_loss_after_permission_is_unknown_and_query_waits_for_actual_writer_exit；query_no_change_keeps_manual_a_and_protected_c_and_drops_unused_candidate；query_read_failures_missing_target_and_absence_without_proof_never_settle_unknown；query_found_original_commit_and_later_binding_conflict_are_both_retained；contradictory_late_query_cannot_erase_confirmed_fact_or_unlock_binding |
| PA15／16／24／35 | cancel_before_permission_keeps_candidate_until_actual_owners_exit；late_main_commit_confirms_during_query_but_query_drop_still_gates_reuse；undispatched_cancel_drop_before_ack_and_session_drop_keep_real_ownership_rules；plain_queries_and_exhausted_ordinary_receipts_do_not_block_confirmation_or_cleanup |
| PA25／35／36／38 | candidate_logical_and_managed_allocation_caps_are_independent_exact_boundaries；candidate_ceiling_does_not_borrow_the_separate_capture_manifest_budget；query_cancellation_retry_and_attempt_exhaustion_preserve_unknown_and_old_context；首次预留对象复用、独立取消、attempt最大值与资源释放 |

28项A2专项通过；A1公开22项及既有application全套回归／doctest保持通过，新增不透明authority和借用写许可共2项compile-fail。application Clippy及源码门禁18项通过。一次直接运行Python门禁时误选全局rustup，因受限临时目录失败；改为仓库既有固定工具链后18项通过，未下载工具或更改全局设置。完整仓库提交仍必须通过正常hook中的check_repository、tools/check_core.py和tools/check_desktop.py，远端结果以包含本文的同SHA三项GitHub任务为准，不套用A1结果。

A3的PA17–23／34、关闭归属Exit／最终保护／草稿收尾，以及各组未实现关闭部分继续欠项。B的真实内容／文件证明、恢复区、发布／发现中断及共同容量，C的宿主全局新会话与完整激活／资源交接，D的文本／IME及产品入口均未签收。后续先审阅A2，再按独立授权进入A3；本增量不是完整P3-2A退出或产品保存闭环。

## 5. 保管

本轮源码、回归与同步文档使用正常提交／公开origin备份；准确源码身份为包含本报告的Git提交及其父abd879d。定向检查、最终hook、原字节／mtime核对和远端回执留在本机忽略目录artifacts/p3-2a-a2-20261115；旧失败记录不覆盖。首轮hook在收尾复核发现矛盾观察漏锁绑定后主动停止，未生成提交；日志保留为a2-hook-interrupted.log，修复后须重新完整执行正常hook。schema.rs既存空差异标记及散列713f3850740db024c4b861327de560f4eb81f5820980d1f89672dcdcfd499125保留。

旧测量原件、冻结程序及PRIVATE恢复附件保持，不读取或上传密钥。[3435405固定恢复快照](PROJECT_RECOVERY.md)不自动包含本轮本机日志；源码恢复依赖本轮正常Git推送，不把“源码已推送”描述为“新增忽略文件已做异地备份”。
