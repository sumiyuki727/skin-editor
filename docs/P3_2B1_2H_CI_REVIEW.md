# P3-2B1-2H：远端取消与证据收尾

2026-11-18（Asia/Tokyo）。[H实现与平台辅助函数收尾](P3_2B1_2H_REVIEW.md)经两次正常完整hook保存为57738b9fb323e3bc1e67c8dd910f78ff24ba1661及2f4a5257235943ef87ec7f137bd3bcc5af43ac0c，并正常推送PUBLIC origin。最终2f4a525的915个受控文件已逐路径／模式／类型／blob SHA与远端核对一致。原schema.rs空差异标记及原字节／mtime保持；原始新日志仅本机，PRIVATE恢复快照未扩充。

## 1. 远端实际结果

[run44167366330](https://github.com/sumiyuki727/skin-editor/actions/runs/44167366330)是该SHA的首次push运行。Ubuntu核心与Windows核心成功；desktop-windows为cancelled，不能签为三项CI成功，也不能以本机完整hook替代远端结果。旧运行不重跑、不追认为通过。

桌面作业20:15:30Z开始、20:40:34Z结束。日志内默认及all-features的基础设施测试均310项全过，分别340.05s／353.55s；本轮10个新用例各通过两次，既有用例仍保留。其后的集成及文档测试均返回成功；20:39:57Z起开始最后的默认宿主build，仍在编译依赖时，于20:40:30Z出现KeyboardInterrupt及operation canceled，未到整个desktop脚本的最终PASS。

作业时长约25分04秒，与既有25分钟配置接近，是可观察事实，不足以确认取消的完整根因。日志未出现测试断言失败，仍不能推出后续构建必然成功。父提交96b3a98的300项测试两配置耗时141.56s／152.13s；工作量与运行环境不完全相同，不能把差值直接归为H实现缺陷、特定测试或共同取消原因。旧first-before及各次取消也继续Unknown。

## 2. 本次仅补文档

只记录取消回执、已完成覆盖及未完成构建，并同步八份入口；不修改产品代码、测试夹具／断言／集合、编译步骤、依赖、预算、25分钟CI或30秒进程门控。不声称已修复取消原因，不以无依据的构建排序、并发或缓存调整追认旧运行。

这些审阅文档按正常完整hook和精确路径备份，新SHA按仓库原流程首次CI；这是新证据备份的验证，不是同SHA重试，也不保证新运行会成功。新结果仅适用于其精确SHA，取消运行和原始日志继续保留。若后续仍未成功，必须明确记录验收未收口，不能以文档更新或偶然绿色宣称根因已修复。

## 3. 能力及验收边界

H已补真实只读恢复来源，经Session／Snapshot／真实Work传入q，并计算捕获已有metadata的完整before域3。F仍NotCompared，fresh-append／would_commit、NoChange／Unknown／物理连续性和生产保存资格均未升级；完整B1／PB16、共同64MiB及旧full／PNG／SQLite／VFS／宿主聚合仍欠。Reader512KiB与A控制512KiB独立、Reference10000／Tiny4、K1和产品NotEvaluated／stable Unknown保持。

仅自制fixture及显式测试根；用户归期未知，UAC／截图／现场步骤及UI／设备／皮肤Skill暂停。详见[结构化取消证据](P3_2B1_2H_CI_PROOF.json)。新增PUBLIC备份覆盖受控源码／文档；新原始日志仅本机，不保证项目目录丢失后能恢复这些新增原件。
