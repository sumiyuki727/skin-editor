# P3-2A A3：关闭门控与最终保护

2026-11-15。用户在本对话直接确认开始A3，并授权主实现与独立审查并行、接续已定约范围。本增量按[开工契约修订2](P3_2A_START_CONTRACT.md)实现纯application关闭意图、既有草稿收尾、最终捕获及故障回归。轮前复用已提交／推送的A2基线894328696d35877823f61db1aa9c8d30edb07ed5，[同提交三项CI](https://github.com/sumiyuki727/skin-editor/actions/runs/43627066529)成功；无可提交内容差异，不造空提交。该CI不是A3新代码的验证。

API7／捕获3／协调端口1、存储端口1及全部持久格式不变。无新依赖，application仍只依赖domain；无真实文件端口、GUI／原生设备、皮肤制作Skill或K1修改。Reference10000／Tiny4、产品性能NotEvaluated和stable兼容Unknown保持，不运行正式成本测量。

## 1. 实际入口与资格

[关闭协调](../crates/application/src/save_coordination/close.rs)提供request_close、choose_close、finish_close_draft、advance_close、retry_close、cancel_close、acknowledge_close及只读close_status。CloseRequest先校验版本／会话，再识别同key同载荷重复；已接纳后来源变化不重捕。同key改载荷Conflict，终结Ack后旧key Stale，末值不回绕。选择Manual／Exit固定，变更须取消原意图。ClosePurpose只是退出／新建／打开／重开意图，不执行宿主动作。

实际submit、navigate及普通Draft Begin／Replace／Commit在成功缓存之前检查BlockedByClose；只读validate不预留提交资格。阻断不新增普通回执。公开capture_history在合法封套后报告独立Closing，不混同SaveOutstanding／草稿Busy／InFlight。精确既有SaveRequest重复仍返回原Ticket，不获得第二任务。

CloseFinishPermit只由application针对接纳时实际草稿签发，绑定同CloseTicket、基础状态及DraftState；外部不能构造。收尾先核对实际草稿，再复用原子草稿入口。成功Commit最多新增一个节点，NoChange／Cancel结束草稿但不造节点；失败保留草稿、Permit及正确Cancel保留位。公共合法Cancel委托同一路径，在AwaitChoice也可取消；仍使用合法普通RequestId。旧成功／取消回执、旧ClosePermit、旧运行会话不能结束另一草稿。

最终保存只由私有FinalCapturePermit通过同一完整捕获实现豁免所属关闭门控；未跳过版本、会话、ExpectedState、限额、取消、单捕获CAS、资源／拓扑／永久身份检查。不提供公开bypass、可变项目引用，不临时移除协调器或关闭门控。

## 2. 事实、等待、取消和重试

SettleExisting先消费既有保存事实，经同一精确Ack入口结清，再等待worker、sender、查询及最后Snapshot／CapturedNode实际释放。Unknown保持原SaveSlot，不能盲重试或用Ack丢弃。旧独立保存q不能直接满足新Close；旧任务确实释放后，在持续门控下捕获包含全部草稿收尾的最终q。

最终子保存的完整Ticket保留parent_close。CancelClose立即解除自己的写门控，仅取消自有子保存；独立Manual、已提交编辑及已确认文件事实保持。CloseSlot是固定POD，Cancelled可终结Ack；其子SaveSlot仍按原身份、Unknown、Ack及实际消费者独立保管。旧关闭子保存未释放时新Close返回Busy(SaveOutstanding)，不消耗新CloseKey；释放后同key可重试接纳，但仍需新Close自己的最终捕获。旧完成不能推进后来关闭。

确定失败子保存不会被重复advance自动重试。retry_close须核对当前Close及失败子Ticket，经Ack／实际释放并再次核对最新事实后重新准入。Ack的第二次事实消费若收到迟到成功，不再当失败发起新保存。新子准入失败且没有Ticket时保留原失败关联，允许同Close／失败Ticket再次明确重试；不回滚已完成Ack，也不伪造未消耗的永久身份。Unknown不能写重试，有限保留身份不保证无限重试。

Exit确认不推进Manual。NoChange的CurrentProtection须有同目标完整证明，且protected.head匹配最终捕获头；头不同的Manual A／保护C／默认A只可按精确VerifiedOpening基线处理。真实导航／新分支不能复用旧打开等价。已确认后出现矛盾证据保留真实事实、锁定绑定并撤销关闭满足；尚未确认的无效覆盖证明保持Unknown。诊断随新阶段更新，不保留已过时的确定失败原因。

ProtectionSatisfied及EmptyCreationBaselineSatisfied始终非终结，保持写门控，不能acknowledge_close，不能关闭窗口或替换项目。只有显式Cancel进入可Ack终结；实际Quiescing／激活权限仍由C／D另行提供。保护满足也不声称所有消费者已释放。

## 3. 无目标、空创建和容量

显式install_close_control复用同一个Coordinator和512KiB预算，配置为无目标，不返回可执行工作端口、不制造binding或文件事实。已有目标安装方式不变，仍只安装一次；无目标选择／重绑定功能。需要保存但没有目标时Manual诊断NeedsTarget，Exit诊断RecoveryUnavailable并保持工程打开，不选择临时目录替代真实恢复区。PreparationOnly也不能冒充保护成功。

从未有效编辑的新空根、无持久元数据且选择Exit，可得到EmptyCreationBaselineSatisfied；仅未保存捕获分配永久映射不算编辑。编辑后Undo根、真正恢复的空根均不适用。首次明确Manual即使空根也需要真实合格提交；此处测试端口的确认不是生产文件成功。

普通保存继续保留u64::MAX供一次最终保护。最终保存可实际消耗末值，之后记录耗尽；Close末值同样只用一次。取消／Ack／已有完成及查询不消耗新普通或控制身份。耗尽拒绝新修改，精确旧普通回执仍可只读重放，正确Draft Cancel仍可达；活跃Close门控始终优先于旧回执。没有清回执、剪历史、重开伪造资格或计数回绕。

[静态布局证明](../crates/application/src/save_coordination/budget.rs)保持8×64KiB分区、普通／保留区各256KiB，总上限512KiB。Windows x64、固定Rust1.95.0的实际布局如下；数字是受管布局与同时载体的静态上界，不是allocator／RSS／实际成本测量。

| 分区 | 静态上界字节 |
| --- | ---: |
| 常驻／绑定 | 1904 |
| 保存槽 | 4360 |
| 工作／完成信箱 | 9392 |
| 事实摘要 | 7840 |
| 关闭控制 | 6080 |
| Unknown查询 | 6288 |
| 失败／确认／归还 | 5123 |
| 最终保护交接 | 2096 |
| 合计 | 43083 |

普通区23496、保留区19587；Coordinator4352、共享控制块2728字节。Ticket400、Witness560、保存状态1488、关闭状态688、诊断80、工作1296、完成1120、查询864、sender416字节。新增CloseRequest／Admission／Status／ActiveStatus／Permit及有界错误的生产、消费副本已计入；关闭不新增动态队列／控制分配点。候选独立1MiB逻辑限额及实际capacity证明、捕获逻辑限额和历史固定原字节仍分别计量，不借512KiB数字证明整个Session堆成本。

## 4. 回归与独立审查

[23项关闭回归](../crates/application/src/save_protocol_tests/close.rs)复用真实Session／Command／草稿／RestoreBuilder／捕获及A2独立黄金状态比较；另有[3项控制末值／预算回归](../crates/application/src/save_coordination/close_bounds.rs)。无文件、时钟、线程、窗口、设备或网络测试输入。测试端口只在cfg(test)签发authority，生产仍无合格适配器签发器。

| 适用PA | 实际测试名／覆盖 |
| --- | --- |
| PA17／28／29／33 | close_gate_precedes_cached_command_navigation_and_draft_success；exact_close_and_save_repeats_do_not_recapture_or_change_choice；an_external_capture_last_owner_holds_final_capture_without_bypassing_closing |
| PA18／24／27／34 | old_draft_commit_and_cancel_receipts_cannot_consume_a_new_close_permit；failed_close_commit_keeps_permit_and_the_last_correct_cancel_receipt；no_change_close_commit_ends_only_the_existing_draft_without_a_node；close_tickets_and_permits_never_cross_runtime_sessions；the_same_draft_needs_a_new_permit_after_cancelled_close_is_replaced |
| PA19／20／25／35 | existing_save_is_settled_and_released_before_the_final_draft_capture；cancelling_close_does_not_cancel_an_independent_manual_save；cancelled_close_keeps_unknown_child_and_late_fact_separate_from_new_close；cancelled_child_keeps_save_slot_until_the_last_captured_node_is_dropped |
| PA13–16／32／34 | failed_child_has_no_automatic_retry_and_explicit_retry_waits_for_actual_release；closing_query_survives_close_cancel_and_late_main_fact_waits_for_query_drop；a_late_contradictory_query_retains_real_fact_but_revokes_close_satisfaction |
| PA11／21／23 | unchanged_opening_exit_confirms_final_q_without_replacing_manual_a_or_protected_c；exact_current_exit_protection_no_change_preserves_the_older_manual_point；another_protected_head_cannot_claim_current_exit_coverage_for_a_new_head |
| PA12／22／23／30 | untouched_empty_exit_needs_no_fake_fact_but_first_manual_really_saves；editing_then_undo_to_empty_still_protects_the_complete_tree；missing_target_keeps_completed_history_open_without_a_recovery_path；an_unsaved_empty_capture_reserves_identity_without_creating_work_to_protect；a_really_restored_empty_root_uses_final_no_change_instead_of_creation_baseline |
| PA24–26／33／38 | final_save_and_close_maximum_are_usable_once_without_wrap_or_lost_cleanup；last_close_identity_keeps_real_draft_cancel_and_final_capture_available；close_only_installation_reserves_the_same_profile_without_a_work_port |

独立生产代码审查发现并修复5项：CurrentProtection缺保护头核对、Close末值释放后旧请求分类、重试准入失败丢失旧关联、Ack二次消费期间迟到确认的再校验，以及耗尽错误遮挡已存在普通回执。相关固定输入／边界回归通过；Ack内部的并发到达窗口另经只读代码复核，没有用线程时序猜测代替协议检查。审查者未发现剩余阻断性生产代码问题。实现与测试分别由不同执行者编写，构建仅由主线统一运行。

26项新增关闭测试、原application公开／内部回归及doctest通过，新增Permit不可伪造compile-fail通过；application Clippy通过。初轮失败日志保留：夹具误丢port、旧子任务准入策略／打开基线假设不符和Unknown断言过强，修订后重新运行；不把这些失败记录覆盖为成功。最后3项新增边界已纳入冻结后的26项运行。最终PA盘点又在原测试内补齐三组直接组合断言：真实草稿提交q+1后原CloseRequest精确重放；导航／新分支回到A后旧VerifiedOpening不能满足新Exit；128普通回执全满时首次Unknown核对仍复用原预留对象、取消关闭后独立运行并等待实际Drop。补齐后26项再次全通过，未新增功能。完整核心／桌面与仓库扫描仍通过正常提交hook执行，准确验证身份以包含本报告的提交及同SHA三项GitHub任务为准。

## 5. 交付与剩余范围

A3完成的是应用层的关闭门控和保护事实条件。独立PA盘点未发现A层仍需新增功能的实现硬项，A整体签收仍应另行汇总PA01–PA38的测试、准确提交与平台执行证据，不能仅以26项计数代替。A1／A2的既有适用证据继续保留；真实文件proof、恢复区位置与总配额、发布／发现中断及共同容量属B，宿主新会话／Quiescing／激活属C，文本／IME和产品入口属D。无窗口替身成功不能签收这些欠项或启用产品保存按钮。可视化拖拽／缩放V0–V3及皮肤制作Skill仍未在本增量开发。

本轮源码／回归及同步文档经正常hook提交并推送既定PUBLIC origin；准确源码身份是包含本报告的Git提交，父提交为8943286。收尾按完整SHA核对远端与GitHub任务；不套用轮前CI。工作区基线、原始检查日志、暂存审查和远端回执位于本机忽略目录artifacts/p3-2a-a3-20261115。保留schema.rs既存空差异标记及SHA256 713f3850740db024c4b861327de560f4eb81f5820980d1f89672dcdcfd499125。

旧测量原件、冻结程序及PRIVATE恢复附件保持，不读取／上传密钥。[3435405固定恢复快照](PROJECT_RECOVERY.md)不自动包含本轮本机新日志。源码已推送不等于新增忽略文件已异地备份；项目源码可由本轮Git恢复，历史材料仍按既有PRIVATE恢复说明保管。
