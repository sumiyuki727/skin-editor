# lazer 时间轴与状态恢复参考

2026-10-20。来源为用户指定的外部只读目录 `D:\osu`，Git HEAD `d4993624374c2984b0df8203acbfbc6267ab5e6e`；11 个选定文件在该提交上无本地修改，大小与 SHA-256 见 [身份记录](lazer-timeline-20261020.json)。这是 Git 快照身份，不是已确认的发布编号或上游最新状态。

根 `LICENCE` 声明 MIT 条件；本轮只记录事实摘要及设计问题，没有复制／移植代码、资源或二进制，没有构建、执行或修改该目录。该目录不成为产品的构建／测试／运行依赖，也不扩大旧 stable 源码的使用边界；本记录不评价附带素材的分发条件。

## 观察与本项目影响

| 位置（仅适用于记录的文件哈希） | 观察摘要 | 可参考的边界 |
| --- | --- | --- |
| `osu.Game/Screens/Edit/EditorClock.cs:73–106` | 编辑器可按当前定时点与拍细分吸附，并提供前后步进 | 本地 lazer 编辑器已有精细节拍导航；不能笼统认为 lazer 完全没有精细定位。我们的预留还包含小节定位、细微偏移和玩法预览联动 |
| `osu.Game/Screens/Play/ReplayPlayer.cs:167–227` | 回放支持前后 seek、暂停及按回放帧步进；逐帧入口先停止时钟 | 回放帧、音乐节拍、音频采样帧是不同概念，不能用回放逐帧代替小节线定位 |
| `osu.Game/Screens/Play/HUD/SongProgressBar.cs:80–94` | 进度条计算时间目标并替换待执行 seek | UI 只请求位置，取消过期请求；它不自行恢复游戏对象或统计 |
| `osu.Game/Screens/Play/Player.cs:399–410,734–760` | 新判定和撤回判定分别进入统计处理；自定义起点的快速定位明确不保证中间判定正确恢复 | “改时钟”与“恢复判定／统计”必须是可区分能力，不能因目标画面出现就称成绩正确 |
| `osu.Game/Rulesets/Scoring/ScoreProcessor.cs:515–551`、`Rulesets/UI/RulesetInputManager.cs:93–95` | 回放统计事件可恢复含 combo、计数及统计内部量的快照，再更新分数 | 重建需要足够的统计状态及明确输入来源；不是仅设置一个 score 或 accuracy 字段。尚未验证所有回放版本／自动输入情况下的完整正确性 |
| `osu.Game/Rulesets/UI/FrameStabilityContainer.cs:145–247` | 时间推进、回放消费和追赶状态分别处理 | 可参考状态分离；不移植其算法或假设本项目已有确定性重放 |
| `osu.Game/Beatmaps/ControlPoints/TimingControlPoint.cs:15–22,42–83`、`Rulesets/Objects/BarLineGenerator.cs:25–89` | 定时、小节与拍号存在分段语义，生成器还明确记有与 stable 的局部差异 | 小节线／拍号不能由全曲固定 4/4 或单一 BPM 推导；lazer 结果不能直接当 stable 证据 |
| `osu.Game.Rulesets.Mania/Edit/ManiaBeatSnapGrid.cs:15–20` | mania 节拍网格与轨道的显示容器关联 | 时间网格与玩法空间投影分开建模；不能从这个入口推断 stable 判定线位置或滚速规则 |

## 本轮采用与未采用

采用“目标解析、预览状态恢复、统计来源、UI 请求分开”的设计方向，见 [时间轴预留](../PREVIEW_TRANSPORT.md)。不在 P1 引入 C# 核心、lazer 包、回放算法或现有评分公式。统计显示策略仍待用户讨论，stable 的定时／判定／滚速表现继续走单独的兼容证据链。

需要后续回答的问题包括：自动输入能否确定性重建长物件状态、可靠检查点应保存什么、精细跳转延迟和取消成本，以及当前玩法的目标时间怎样投影到判定线。上述源码观察不代替这些实测。
