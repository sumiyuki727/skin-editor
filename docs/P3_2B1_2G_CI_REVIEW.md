# P3-2B1-2G：夹具构造事务与CI收尾

2026-11-18（Asia/Tokyo）。[G实现](P3_2B1_2G_REVIEW.md)已通过本地正常完整hook并公开推送075b2044549a7fc4a40685b6ef90c751be8cf1bb。[首次CI44125234390](https://github.com/sumiyuki727/skin-editor/actions/runs/44125234390)两项核心成功，桌面为cancelled，不能签为三项成功。轮前实际／暂存差异为空，复用该已推送回退点，明确其CI未收口；904文件原字节／mtime基线仅本机，不建空提交、不重跑或追认旧SHA。

## 1. 实际观察与未确定事项

桌面原始日志显示默认与all-features的基础设施测试各300项全过，分别403.64s／400.50s；随后集成检查未全部完成，出现operation canceled，作业约25分钟后结束。两次新metadata负例循环从“running over60s”警告到完成又经过约220.5s／212.5s，故各至少约280.5s／272.5s；这是CI日志中可观察的耗时区间，不是正式性能测量，也不足以确认取消根因。

现有25分钟作业配置未变。连接器不支持所尝试的check-run annotations端点，不能据此补造取消原因；本次原因仍Unknown。旧first-before及较早两次取消也继续Unknown，不能归为同因或宣称修复。

检查自制metadata_fixture发现：它在专用临时目录内重建测试输入，每次DROP／INSERT／UPDATE／块索引与拓扑操作分别自动提交。可减少这些构造阶段提交，而不改变最终受测文件的语义或验证强度；不预先保证新CI必在时限内结束。

## 2. 唯一源码修订

仅修改[测试夹具](../desktop/infrastructure/src/project_storage/tests/file_facts/historical_metadata.rs)：FK OFF后BEGIN，保留全部原构造动作，恢复全部guards后COMMIT，再FK ON并执行原schema::verify。显式断言BEGIN后／COMMIT前不是autocommit，提交后恢复autocommit，foreign_keys读回1。

publish_new／write_capture在此构造前已完成；blocks::paged／make／add不另开事务。只合并自制输入构造的提交，不改产品存储／read observer／接口／schema／writer、journal模式、同步等级、预算、25分钟CI、30秒门控或任何黄金与预期。事务可能改变SQLite物理头／页布局，不要求不同构造版本整库原始SHA相等；完成构造后的源字节不变检查仍保留。

健康文件仍先经旧standalone load及冻结黄金核对；八坏例仍独立目录，精确断言旧StorageError及新Unsupported／ValidationFailed；所有源字节、额度／scratch回收、旧结果与实际Drop／Ack断言保持。其余十个G源码、原G契约／向量／证明均按基线保留。

## 3. 复核与交付门槛

两路独立只读审阅先确认方案，再核对实际diff和全部用例未删改。修订后定向124项测试集合与075完全相同，application92项及15项文档测试通过；Clippy单独核对。精确散列和旧取消回执见[结构化证据](P3_2B1_2G_CI_PROOF.json)。正常完整hook、精确路径PUBLIC推送、新完整SHA首次三项CI及远端文件树逐项核对，不能套用075的核心成功或本机结果。

本修订不改变G的能力：文件侧历史metadata有来源，q仍缺features，F.metadata为NotCompared。Experimental／Insufficient／BudgetUnproven、完整B1／PB16及NoChange／Unknown／物理绑定／共同64MiB欠项保持，不跨B2或产品保存。K1／Reference10000／Tiny4、产品NotEvaluated／stable Unknown不变。

唯一源码和十份文档按完整流程收尾；旧证明／失败日志／冻结程序／PRIVATE及schema.rs原字节／mtime保留，新增原始忽略日志仅本机，不自动扩张旧私有快照。仅自制fixture及显式测试根，用户返回未知，现场UAC／截图／测试等待，UI／Skill暂停。
