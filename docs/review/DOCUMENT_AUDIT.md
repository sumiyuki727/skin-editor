# 全部文档审查与归档

2026-10-30。基线共145份Markdown（docs内139份），已逐文件核对位置、状态、用途及链接，重点复核当前契约；不声称逐条重新裁定全部旧研究／测量结论。117份既有JSON证据只做原字节完整性核对，不改写。

2026-11-01后续定约：新增[数据兼容设计](../DATA_COMPATIBILITY.md)，用户已确认首版完整历史与分支持久化；AGENTS及现行需求／架构／路线／地址／标签／设置／准入说明同步这一范围更新。下表保留10月30日审查身份，不把后来要求追认为当时已经实现；旧归档、原证明及代码清单未重写。

## 处理原则

2026-11-02 P3-0补充：新增[格式v1](../PROJECT_FORMAT_V1.md)、[原型审阅结果](../P3_0_REVIEW.md)和工具说明；同步数据兼容／地址、架构／技术栈、路线／阶段入口、调度设计及审阅决定。既有AGENTS已覆盖不可变历史和持久化边界，无需把SQL参数写入长期规则。旧归档／JSON／代码清单保持原字节；此前“尚未开始P3”的带日期段落作为历史背景，当前状态以P3-0审阅包为准。

- 当前入口移除与现状冲突的API4／预览1、后端未验或未来UI描述；保留旧版本变迁的明确历史含义。
- 17份原文按字节归档为`.md.txt`，旧相对链接按原路径解释，清单含长度／SHA256；不自动恢复覆盖新编辑。
- P2测量及P0研究是证据，不因当前不用就删除；主架构不再混入逐轮实验时间线。
- P0源码、fixture、tools、宿主诊断和另一音频路径仍有引用／重现／平台用途，没有被证明为可安全移走的死代码。仅做用途标记，未移动或删减产品源码。
- .p0-tools／target／artifacts本已属于本机工具、缓存与证据，不加入Git或扫描私人媒体；本轮不删除它们。

## 基线逐文档结果

| 文档 | 分类 | 结论／必要变更 |
| --- | --- | --- |
| [AGENTS.md](../../AGENTS.md) | 长期约束 | 保留；本轮未批准新历史／地址政策，不把待讨论方案写成规则 |
| [README.md](../../README.md) | 当前契约／入口 | 已更新事实／版本／状态；失效长日志已归档，业务行为未改 |
| [docs/ARCHITECTURE.md](../../docs/ARCHITECTURE.md) | 当前契约／入口 | 已更新事实／版本／状态；失效长日志已归档，业务行为未改 |
| [docs/AUDIO_EDITING_REVIEW.md](../../docs/AUDIO_EDITING_REVIEW.md) | 当前契约／入口 | 已更新事实／版本／状态；失效长日志已归档，业务行为未改 |
| [docs/COMMANDS.md](../../docs/COMMANDS.md) | 当前契约／入口 | 已更新事实／版本／状态；失效长日志已归档，业务行为未改 |
| [docs/COMPONENT_MODEL.md](../../docs/COMPONENT_MODEL.md) | 当前契约／入口 | 已更新事实／版本／状态；失效长日志已归档，业务行为未改 |
| [docs/CORE_COST_BASELINE.md](../../docs/CORE_COST_BASELINE.md) | P1阶段记录 | 保留原元数据范围及旧计数；当前版本由架构／契约说明 |
| [docs/DRAFTS.md](../../docs/DRAFTS.md) | 当前契约／入口 | 已更新事实／版本／状态；失效长日志已归档，业务行为未改 |
| [docs/EDITOR_INPUT.md](../../docs/EDITOR_INPUT.md) | 当前契约／入口 | 已更新事实／版本／状态；失效长日志已归档，业务行为未改 |
| [docs/HISTORY_ADDRESSING.md](../../docs/HISTORY_ADDRESSING.md) | 当前契约／入口 | 已更新事实／版本／状态；失效长日志已归档，业务行为未改 |
| [docs/HISTORY_PRESENTATION.md](../../docs/HISTORY_PRESENTATION.md) | 现行专题契约 | 保留；与当前实现／边界核对，未发现需本轮改变的规则 |
| [docs/ONLINE_ASSETS.md](../../docs/ONLINE_ASSETS.md) | 未来设计／部分接线 | 保留；未实现部分需按路线验证，不因预留类型判作完成 |
| [docs/P1_PLAN.md](../../docs/P1_PLAN.md) | P1阶段记录 | 保留原元数据范围及旧计数；当前版本由架构／契约说明 |
| [docs/P1_PROGRESS.md](../../docs/P1_PROGRESS.md) | 当前契约／入口 | 已更新事实／版本／状态；失效长日志已归档，业务行为未改 |
| [docs/P2_ACTIVE_EDIT_FAULTS.md](../../docs/P2_ACTIVE_EDIT_FAULTS.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_AUDIO_CORE.md](../../docs/P2_AUDIO_CORE.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_AUDIO_EDIT.md](../../docs/P2_AUDIO_EDIT.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_AUDIO_LIBRARY.md](../../docs/P2_AUDIO_LIBRARY.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_AUDIO_LOAD.md](../../docs/P2_AUDIO_LOAD.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_AUDIO_NATIVE.md](../../docs/P2_AUDIO_NATIVE.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_AUDIO_RESOURCES.md](../../docs/P2_AUDIO_RESOURCES.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_AUDIO_RUNTIME.md](../../docs/P2_AUDIO_RUNTIME.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_AUDIO_STARTUP.md](../../docs/P2_AUDIO_STARTUP.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_AUDIO_SUPPLY.md](../../docs/P2_AUDIO_SUPPLY.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_AUDIO_UI.md](../../docs/P2_AUDIO_UI.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_AUTOMATIC_CLOSEOUT.md](../../docs/P2_AUTOMATIC_CLOSEOUT.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_B05_PRODUCT_ROUTE.md](../../docs/P2_B05_PRODUCT_ROUTE.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_BACKGROUND_COORDINATION.md](../../docs/P2_BACKGROUND_COORDINATION.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_BUDGET_REVIEW.md](../../docs/P2_BUDGET_REVIEW.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_CLEANUP_RECOVERY.md](../../docs/P2_CLEANUP_RECOVERY.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_CLOSEOUT.md](../../docs/P2_CLOSEOUT.md) | 当前契约／入口 | 已更新事实／版本／状态；失效长日志已归档，业务行为未改 |
| [docs/P2_COMBINED.md](../../docs/P2_COMBINED.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_CONTINUOUS_BACKGROUND.md](../../docs/P2_CONTINUOUS_BACKGROUND.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_CONTROLLED_RECOVERY.md](../../docs/P2_CONTROLLED_RECOVERY.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_CPU_ACCOUNTING.md](../../docs/P2_CPU_ACCOUNTING.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_CPU_THREAD_IDENTITY.md](../../docs/P2_CPU_THREAD_IDENTITY.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_CURRENT_COMBINED.md](../../docs/P2_CURRENT_COMBINED.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_ENDPOINT_RETURN.md](../../docs/P2_ENDPOINT_RETURN.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_EVENT_LOOP_UI.md](../../docs/P2_EVENT_LOOP_UI.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_FAULT_REBUILD.md](../../docs/P2_FAULT_REBUILD.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_FINAL_ACCEPTANCE.md](../../docs/P2_FINAL_ACCEPTANCE.md) | 固定验收／反馈 | 保留事实及分构建范围，不重跑或重判旧结论 |
| [docs/P2_FRAMEWORK_BASELINE.md](../../docs/P2_FRAMEWORK_BASELINE.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_FRAME_DISPATCH.md](../../docs/P2_FRAME_DISPATCH.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_FRAME_PREPARATION.md](../../docs/P2_FRAME_PREPARATION.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_GPU_CPU_RECHECK.md](../../docs/P2_GPU_CPU_RECHECK.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_GPU_PATH.md](../../docs/P2_GPU_PATH.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_GUI_BASE_COST.md](../../docs/P2_GUI_BASE_COST.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_GUI_BASE_NATIVE.md](../../docs/P2_GUI_BASE_NATIVE.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_GUI_CANDIDATE.md](../../docs/P2_GUI_CANDIDATE.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_GUI_CPU_ATTRIBUTION.md](../../docs/P2_GUI_CPU_ATTRIBUTION.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_HELD_GESTURE_FIX.md](../../docs/P2_HELD_GESTURE_FIX.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_HISTORY_PAINTER.md](../../docs/P2_HISTORY_PAINTER.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_HISTORY_PANEL.md](../../docs/P2_HISTORY_PANEL.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_INDEPENDENT_CPU.md](../../docs/P2_INDEPENDENT_CPU.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_INIT_RECOVERY.md](../../docs/P2_INIT_RECOVERY.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_INTERACTION_MODE.md](../../docs/P2_INTERACTION_MODE.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_LIVE_AUDITION.md](../../docs/P2_LIVE_AUDITION.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_MAIN_WORK.md](../../docs/P2_MAIN_WORK.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_MANUAL_ACCEPTANCE.md](../../docs/P2_MANUAL_ACCEPTANCE.md) | 固定验收／反馈 | 保留事实及分构建范围，不重跑或重判旧结论 |
| [docs/P2_MANUAL_RESULTS.md](../../docs/P2_MANUAL_RESULTS.md) | 固定验收／反馈 | 保留事实及分构建范围，不重跑或重判旧结论 |
| [docs/P2_MEMORY_USAGE.md](../../docs/P2_MEMORY_USAGE.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_MESH_REPLAY.md](../../docs/P2_MESH_REPLAY.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_PAINTER_CPU.md](../../docs/P2_PAINTER_CPU.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_PRESENT_FAULT.md](../../docs/P2_PRESENT_FAULT.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_PROGRESS.md](../../docs/P2_PROGRESS.md) | 当前契约／入口 | 已更新事实／版本／状态；失效长日志已归档，业务行为未改 |
| [docs/P2_REBUILD_ATTRIBUTION.md](../../docs/P2_REBUILD_ATTRIBUTION.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_REBUILD_POLICY.md](../../docs/P2_REBUILD_POLICY.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_REPAINT_MIN.md](../../docs/P2_REPAINT_MIN.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_REPAINT_ORIGIN.md](../../docs/P2_REPAINT_ORIGIN.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_RESOURCE_ATTRIBUTION.md](../../docs/P2_RESOURCE_ATTRIBUTION.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_RESOURCE_CYCLES.md](../../docs/P2_RESOURCE_CYCLES.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_RUNTIME_VOICE_BRIDGE.md](../../docs/P2_RUNTIME_VOICE_BRIDGE.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_SCHEDULING_CORRELATION.md](../../docs/P2_SCHEDULING_CORRELATION.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_SELECTION_CONTEXT.md](../../docs/P2_SELECTION_CONTEXT.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_SEPARATED_NATIVE.md](../../docs/P2_SEPARATED_NATIVE.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_SEPARATED_SUPPLY_MODEL.md](../../docs/P2_SEPARATED_SUPPLY_MODEL.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_STABLE_WINDOW.md](../../docs/P2_STABLE_WINDOW.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_STAGE_GATE.md](../../docs/P2_STAGE_GATE.md) | 当前契约／入口 | 已更新事实／版本／状态；失效长日志已归档，业务行为未改 |
| [docs/P2_STAGING_RECOVERY.md](../../docs/P2_STAGING_RECOVERY.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_STAGING_REUSE.md](../../docs/P2_STAGING_REUSE.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_SUBMISSION_STAGES.md](../../docs/P2_SUBMISSION_STAGES.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_SUPPLY_RECOVERY.md](../../docs/P2_SUPPLY_RECOVERY.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_TEXT_WORK.md](../../docs/P2_TEXT_WORK.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_THREAD_SAMPLING.md](../../docs/P2_THREAD_SAMPLING.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_TRANSFORM_CONTROLS.md](../../docs/P2_TRANSFORM_CONTROLS.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_UI_CPU.md](../../docs/P2_UI_CPU.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_UI_DISPLAY_REUSE.md](../../docs/P2_UI_DISPLAY_REUSE.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_UI_SEGMENTS.md](../../docs/P2_UI_SEGMENTS.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_UI_WALL.md](../../docs/P2_UI_WALL.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_UNDERRUN_CONTEXT.md](../../docs/P2_UNDERRUN_CONTEXT.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_VOICE_CPU.md](../../docs/P2_VOICE_CPU.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_VOICE_HANDOFF.md](../../docs/P2_VOICE_HANDOFF.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_VOICE_PHASE.md](../../docs/P2_VOICE_PHASE.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_WINDOW_RECOVERY.md](../../docs/P2_WINDOW_RECOVERY.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/P2_WORKER_NATIVE.md](../../docs/P2_WORKER_NATIVE.md) | 历史增量／测量 | 保留原身份；旧“候选／未完成”是当时状态，不是当前路线入口 |
| [docs/PRESENTATION_CONTRACT.md](../../docs/PRESENTATION_CONTRACT.md) | 当前契约／入口 | 已更新事实／版本／状态；失效长日志已归档，业务行为未改 |
| [docs/PREVIEW_SETTINGS.md](../../docs/PREVIEW_SETTINGS.md) | 当前契约／入口 | 已更新事实／版本／状态；失效长日志已归档，业务行为未改 |
| [docs/PREVIEW_TRANSPORT.md](../../docs/PREVIEW_TRANSPORT.md) | 当前契约／入口 | 已更新事实／版本／状态；失效长日志已归档，业务行为未改 |
| [docs/REQUIREMENTS.md](../../docs/REQUIREMENTS.md) | 当前契约／入口 | 已更新事实／版本／状态；失效长日志已归档，业务行为未改 |
| [docs/RESOURCE_BUDGET.md](../../docs/RESOURCE_BUDGET.md) | 现行专题契约 | 保留；与当前实现／边界核对，未发现需本轮改变的规则 |
| [docs/ROADMAP.md](../../docs/ROADMAP.md) | 当前契约／入口 | 已更新事实／版本／状态；失效长日志已归档，业务行为未改 |
| [docs/TECH_STACK.md](../../docs/TECH_STACK.md) | 当前契约／入口 | 已更新事实／版本／状态；失效长日志已归档，业务行为未改 |
| [docs/p0/ALLOCATION_ATTRIBUTION.md](../../docs/p0/ALLOCATION_ATTRIBUTION.md) | 历史研究／实验 | 保留原路径；仍被工具／证据引用，不纳入产品实现 |
| [docs/p0/ASYNC_COMPOSITION.md](../../docs/p0/ASYNC_COMPOSITION.md) | 历史研究／实验 | 保留原路径；仍被工具／证据引用，不纳入产品实现 |
| [docs/p0/ASYNC_LOADING.md](../../docs/p0/ASYNC_LOADING.md) | 历史研究／实验 | 保留原路径；仍被工具／证据引用，不纳入产品实现 |
| [docs/p0/AUDIO_ADAPTATION.md](../../docs/p0/AUDIO_ADAPTATION.md) | 历史研究／实验 | 保留原路径；仍被工具／证据引用，不纳入产品实现 |
| [docs/p0/AUDIO_DEVICE.md](../../docs/p0/AUDIO_DEVICE.md) | 历史研究／实验 | 保留原路径；仍被工具／证据引用，不纳入产品实现 |
| [docs/p0/AUDIO_PCM.md](../../docs/p0/AUDIO_PCM.md) | 历史研究／实验 | 保留原路径；仍被工具／证据引用，不纳入产品实现 |
| [docs/p0/AUDIO_RECOVERY.md](../../docs/p0/AUDIO_RECOVERY.md) | 历史研究／实验 | 保留原路径；仍被工具／证据引用，不纳入产品实现 |
| [docs/p0/COMBINED_SOAK.md](../../docs/p0/COMBINED_SOAK.md) | 历史研究／实验 | 保留原路径；仍被工具／证据引用，不纳入产品实现 |
| [docs/p0/COMPOSITION.md](../../docs/p0/COMPOSITION.md) | 历史研究／实验 | 保留原路径；仍被工具／证据引用，不纳入产品实现 |
| [docs/p0/COMPOSITION_SCALE.md](../../docs/p0/COMPOSITION_SCALE.md) | 历史研究／实验 | 保留原路径；仍被工具／证据引用，不纳入产品实现 |
| [docs/p0/COMPRESSED_AUDIO.md](../../docs/p0/COMPRESSED_AUDIO.md) | 历史研究／实验 | 保留原路径；仍被工具／证据引用，不纳入产品实现 |
| [docs/p0/COMPRESSED_COMBINED_SOAK.md](../../docs/p0/COMPRESSED_COMBINED_SOAK.md) | 历史研究／实验 | 保留原路径；仍被工具／证据引用，不纳入产品实现 |
| [docs/p0/EDIT_COMPOSITION.md](../../docs/p0/EDIT_COMPOSITION.md) | 历史研究／实验 | 保留原路径；仍被工具／证据引用，不纳入产品实现 |
| [docs/p0/EVIDENCE_WORKFLOW.md](../../docs/p0/EVIDENCE_WORKFLOW.md) | 历史研究／实验 | 保留原路径；仍被工具／证据引用，不纳入产品实现 |
| [docs/p0/GPU_UPLOAD.md](../../docs/p0/GPU_UPLOAD.md) | 历史研究／实验 | 保留原路径；仍被工具／证据引用，不纳入产品实现 |
| [docs/p0/INTERACTION.md](../../docs/p0/INTERACTION.md) | 历史研究／实验 | 保留原路径；仍被工具／证据引用，不纳入产品实现 |
| [docs/p0/LOADER_TUNING.md](../../docs/p0/LOADER_TUNING.md) | 历史研究／实验 | 保留原路径；仍被工具／证据引用，不纳入产品实现 |
| [docs/p0/MEDIA_TIMELINE.md](../../docs/p0/MEDIA_TIMELINE.md) | 历史研究／实验 | 保留原路径；仍被工具／证据引用，不纳入产品实现 |
| [docs/p0/MP3_CANCELLATION.md](../../docs/p0/MP3_CANCELLATION.md) | 历史研究／实验 | 保留原路径；仍被工具／证据引用，不纳入产品实现 |
| [docs/p0/P0_EXIT_REVIEW.md](../../docs/p0/P0_EXIT_REVIEW.md) | 历史研究／实验 | 保留原路径；仍被工具／证据引用，不纳入产品实现 |
| [docs/p0/PREVIEW_SCOPE.md](../../docs/p0/PREVIEW_SCOPE.md) | 历史研究／实验 | 保留原路径；仍被工具／证据引用，不纳入产品实现 |
| [docs/p0/REPORT.md](../../docs/p0/REPORT.md) | 历史研究／实验 | 保留原路径；仍被工具／证据引用，不纳入产品实现 |
| [docs/p0/RESOURCE_BUDGET.md](../../docs/p0/RESOURCE_BUDGET.md) | 历史研究／实验 | 保留原路径；仍被工具／证据引用，不纳入产品实现 |
| [docs/p0/RESOURCE_COMPARISON.md](../../docs/p0/RESOURCE_COMPARISON.md) | 历史研究／实验 | 保留原路径；仍被工具／证据引用，不纳入产品实现 |
| [docs/p0/RESOURCE_CYCLES.md](../../docs/p0/RESOURCE_CYCLES.md) | 历史研究／实验 | 保留原路径；仍被工具／证据引用，不纳入产品实现 |
| [docs/p0/STABLE_C03_C08.md](../../docs/p0/STABLE_C03_C08.md) | 历史研究／实验 | 保留原路径；仍被工具／证据引用，不纳入产品实现 |
| [docs/p0/STABLE_C03_C08_RESULTS.md](../../docs/p0/STABLE_C03_C08_RESULTS.md) | 历史研究／实验 | 保留原路径；仍被工具／证据引用，不纳入产品实现 |
| [docs/p0/STABLE_FIRST_BATCH.md](../../docs/p0/STABLE_FIRST_BATCH.md) | 历史研究／实验 | 保留原路径；仍被工具／证据引用，不纳入产品实现 |
| [docs/p0/STABLE_OBSERVATIONS_20261020.md](../../docs/p0/STABLE_OBSERVATIONS_20261020.md) | 历史研究／实验 | 保留原路径；仍被工具／证据引用，不纳入产品实现 |
| [docs/p0/STANDARD_LOAD.md](../../docs/p0/STANDARD_LOAD.md) | 历史研究／实验 | 保留原路径；仍被工具／证据引用，不纳入产品实现 |
| [docs/p0/STREAMING_AUDIO.md](../../docs/p0/STREAMING_AUDIO.md) | 历史研究／实验 | 保留原路径；仍被工具／证据引用，不纳入产品实现 |
| [docs/p0/STREAM_OUTPUT.md](../../docs/p0/STREAM_OUTPUT.md) | 历史研究／实验 | 保留原路径；仍被工具／证据引用，不纳入产品实现 |
| [docs/p0/SWITCH_AUDIO.md](../../docs/p0/SWITCH_AUDIO.md) | 历史研究／实验 | 保留原路径；仍被工具／证据引用，不纳入产品实现 |
| [docs/p0/TRANSPORT.md](../../docs/p0/TRANSPORT.md) | 历史研究／实验 | 保留原路径；仍被工具／证据引用，不纳入产品实现 |
| [docs/p0/UPLOAD_LIFETIME.md](../../docs/p0/UPLOAD_LIFETIME.md) | 历史研究／实验 | 保留原路径；仍被工具／证据引用，不纳入产品实现 |
| [docs/p0/VALIDATION_MATRIX.md](../../docs/p0/VALIDATION_MATRIX.md) | 历史研究／实验 | 保留原路径；仍被工具／证据引用，不纳入产品实现 |
| [docs/p0/VORBIS_TAIL_AND_SEEK.md](../../docs/p0/VORBIS_TAIL_AND_SEEK.md) | 历史研究／实验 | 保留原路径；仍被工具／证据引用，不纳入产品实现 |
| [docs/p0/WINDOWS_DESKTOP.md](../../docs/p0/WINDOWS_DESKTOP.md) | 历史研究／实验 | 保留原路径；仍被工具／证据引用，不纳入产品实现 |
| [docs/research/LAZER_TIMELINE_20261020.md](../../docs/research/LAZER_TIMELINE_20261020.md) | 历史研究／实验 | 保留原路径；仍被工具／证据引用，不纳入产品实现 |
| [experiments/p0-egui/README.md](../../experiments/p0-egui/README.md) | 历史研究／实验 | 保留原路径；仍被工具／证据引用，不纳入产品实现 |
| [experiments/p0-media/README.md](../../experiments/p0-media/README.md) | 历史研究／实验 | 保留原路径；仍被工具／证据引用，不纳入产品实现 |
| [experiments/p0-preview-contract/README.md](../../experiments/p0-preview-contract/README.md) | 历史研究／实验 | 保留原路径；仍被工具／证据引用，不纳入产品实现 |
| [experiments/p0-wasapi/README.md](../../experiments/p0-wasapi/README.md) | 历史研究／实验 | 保留原路径；仍被工具／证据引用，不纳入产品实现 |

## 新增审阅与归档文档

| 文档 | 用途 |
| --- | --- |
| [docs/INDEX.md](../../docs/INDEX.md) | 现行入口／审查意见／归档用途说明 |
| [docs/archive/2026-10-30-pre-review/README.md](../../docs/archive/2026-10-30-pre-review/README.md) | 现行入口／审查意见／归档用途说明 |
| [docs/archive/README.md](../../docs/archive/README.md) | 现行入口／审查意见／归档用途说明 |
| [docs/review/DECISIONS.md](../../docs/review/DECISIONS.md) | 现行入口／审查意见／归档用途说明 |
| [docs/review/DOCUMENT_AUDIT.md](../../docs/review/DOCUMENT_AUDIT.md) | 现行入口／审查意见／归档用途说明 |
| [docs/review/README.md](../../docs/review/README.md) | 现行入口／审查意见／归档用途说明 |
| [docs/review/SYSTEM_MAP.md](../../docs/review/SYSTEM_MAP.md) | 现行入口／审查意见／归档用途说明 |
| [docs/review/VALIDATION.md](../../docs/review/VALIDATION.md) | 本轮覆盖、检查结果及修改边界 |
| [docs/review/catalog/SOURCE_FILES.md](../../docs/review/catalog/SOURCE_FILES.md) | 编译器／源码审查快照，后续变动需重建 |
| [docs/review/catalog/skin_editor_application.md](../../docs/review/catalog/skin_editor_application.md) | 编译器／源码审查快照，后续变动需重建 |
| [docs/review/catalog/skin_editor_audio.md](../../docs/review/catalog/skin_editor_audio.md) | 编译器／源码审查快照，后续变动需重建 |
| [docs/review/catalog/skin_editor_desktop.md](../../docs/review/catalog/skin_editor_desktop.md) | 编译器／源码审查快照，后续变动需重建 |
| [docs/review/catalog/skin_editor_domain.md](../../docs/review/catalog/skin_editor_domain.md) | 编译器／源码审查快照，后续变动需重建 |
| [docs/review/catalog/skin_editor_infrastructure.md](../../docs/review/catalog/skin_editor_infrastructure.md) | 编译器／源码审查快照，后续变动需重建 |
| [docs/review/catalog/skin_editor_preview.md](../../docs/review/catalog/skin_editor_preview.md) | 编译器／源码审查快照，后续变动需重建 |
| [docs/review/catalog/skin_editor_rendering.md](../../docs/review/catalog/skin_editor_rendering.md) | 编译器／源码审查快照，后续变动需重建 |
| [docs/review/catalog/skin_editor_ui.md](../../docs/review/catalog/skin_editor_ui.md) | 编译器／源码审查快照，后续变动需重建 |
| [experiments/README.md](../../experiments/README.md) | 现行入口／审查意见／归档用途说明 |

## 保留并待讨论的内容

短地址、长会话回执、ID分配、素材库历史语义、组件及导出能力仅列[建议](DECISIONS.md)，未改AGENTS或运行行为。许可／默认皮肤、真实端点及其他平台仍待各自验证。

本轮当前源码只改四行说明注释，另增审查清单生成工具及对应测试；正式检查、链接／哈希和来源差异结果随审阅包记录。没有新native／UAC验收、不推进P3、不自动提交Git。

另追加窄范围Git属性，避免历史JSON与归档原文在暂存／检出时因换行转换失去原字节身份；117份既有JSON及17份快照内容保持不变。
