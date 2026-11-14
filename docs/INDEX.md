# 文档索引

当前总体进度、路线建议和检查结果先读[2026-11-09全面检查](PROJECT_REVIEW_20261109.md)，再按[架构](ARCHITECTURE.md)、[技术栈](TECH_STACK.md)和[README](../README.md)查对应实现。[代码与决策包](review/README.md)是2026-10-30审查快照，后续已有P3状态／捕获／文件恢复及K1增量，不能将原清单当当前全量代码。独立存储基础仍不等于普通保存闭环，历史测量也不覆盖当前版本的新路径。

## 当前阅读入口

- [P3-2A A2事实、见证与实际释放](P3_2A_A2_REVIEW.md)（28项专项、候选capacity和独立核对；仅无窗口测试profile，生产资格／文件端口未接；A3关闭及B–D待续）

- [skin.ini可视化布局编辑：拖拽与缩放](SKIN_INI_VISUAL_EDITING.md)（2026-11-14一致性修订2、10组发现；不限mania，V0–V3／VL01–VL22，全部待实施；主线仍先P3保存闭环，V0真实参数表及具体开工限额仍待冻结）
- [P3-2A A1实现与静态容量证明](P3_2A_A1_REVIEW.md)（API7／捕获3／协调端口1、八固定控制分区、单槽真实捕获准备及回归；A1历史报告；当前A2另见上，仍无真实文件保存／关闭）
- [P3-2A无窗口保存协调开工契约](P3_2A_START_CONTRACT.md)（一致性修订2；A1／A2适用纯用例部分已实施，PA01–PA38的关闭及真实适配语义仍欠项，不能签收完整A）
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
