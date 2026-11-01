# 文档索引

当前实现以[架构](ARCHITECTURE.md)、[技术栈](TECH_STACK.md)、[README](../README.md)为准；代码基线见[代码与决策包](review/README.md)；本轮已按授权完成[P3-0定约与独立原型](P3_0_REVIEW.md)，交由用户审阅，未接入正式保存。历史测量中的旧待办不覆盖当前[阶段结论](P2_FINAL_ACCEPTANCE.md)。

## 当前阅读入口

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
| [AUDIO_EDITING_REVIEW.md](AUDIO_EDITING_REVIEW.md) | 当前契约／入口 |
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
