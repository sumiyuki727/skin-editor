# P3-2B1-2G：文件侧历史metadata实现审阅

2026-11-18（Asia/Tokyo）。轮前bc0b935bb6700dc0ae04fea029235300cb369f55与PUBLIC origin一致，run44104073562三项首次成功；实际／暂存差异为空，复用HEAD，不建空提交。897文件原字节／mtime基线仅本机，schema.rs既存空差异标记保留。

## 1. 补齐的欠项与仍缺来源

本片补齐旧读取器支持范围内的文件侧逐代post-commit metadata域3。此前只有最新完整域3和ArchiveMetadata-only意图，不能对旧提交套用最新tags／features。调查确认文件checkpoint保三metadata根，原full逐代已有真实列表及扩展字节验证，因此可在该窗口直接汇总。

真实q仍缺features：ArchiveMetadata、RestoreHeader和CaptureManifest没有该字段；writer四项默认能力是新文件／读取规则，不是捕获来源。本片没有补造q.features，F.metadata继续NotCompared，不提供q metadata_equal、NoChange或保存资格。完整历史q仍未完成。

[契约](P3_2B1_2G_CONTRACT.md)与[八组独立模型](P3_2B1_2G_VECTORS.json)经两路只读审阅后，于2026-11-17T15:39:16Z冻结。契约SHA256为3b38ec0249079b4f274f9ddc69c4f72d520c29c5088056aa2dde0356196c1fd0；向量为a9c83a795b9f56c37a495dc5314d8cb00a639170edbffd33fc840a720841bbc1。字段／TLV／SHA、旧401黄金及固定身份来源已独立复算，未从新实现生成期望。

## 2. 来源、编码与发布

[内部读取入口](../desktop/infrastructure/src/project_storage/read.rs)新增可选窄observer，旧full_probe默认None，不实例化固定collector或新增Small限制。FileFacts在本代三列表完成旧validate_past_metadata、receipts／Manual marks更新后同步借用。tags用本代原值；extensions按本代声明名字、已经验证的SHA／长度取得真实全局原字节；features只取本代列表。未知optional保留；旧格式不支持已从全局消失或同名换字节的历史扩展，照旧拒绝。

[固定collector](../desktop/infrastructure/src/project_storage/file_facts/historical_metadata.rs)仅排序u16索引，不克隆metadata或列表，不增加JSON／Vec／Map、图／原件遍历或checkpoint解析。node用无分配数值解析，扩展在规范全局表二分借用。tags按数值node/name、扩展按name、features按name/version；marks保持代次顺序。[共用编码helper](../desktop/infrastructure/src/project_storage/file_facts/proof.rs)精确沿用域3前缀／TLV，旧黄金不变；末代摘要、五计数与扩展字节显式核对当前完整结果。

observer只写未发布槽；后续任意full、D时序、源复核或清理错误都不发布。公开FileHistoricalMetadata1保完整观察／请求／selected提交、ExperimentalValidation及可选facts，缺失不冒称NotCommitted。facts给post-commit摘要、完整FileCommitFacts、features_root、五计数和扩展字节；checkpoint.navigation绑定三根。API7／捕获3／协调1、旧端口和文件／记录／证明格式不变，无见证转换或项目写入。

## 3. 容量与取消

每点静态不超过80字节，128槽；计数u16、扩展总字节u32均checked并受既有Small边界约束。历史tags可到8192，不能套用当前SQL tags的4096上限。固定u16[8192]工作按两份构造重叠及具名局部／Encoder／Point预留，不超过40KiB；新数组构造重叠纳入2Data。

D＋G工作在validate帧前合预留，实际退出后退额。既有Reader512KiB不变，合计两观察（一个旧＋当前）加真实q的准确N／N−1已验，全部旧聚合及独立prepare边界保留；查询不增加管理计数。Reader→观察→q实际Drop／Ack测试确认最终所有者前不退额。首个真实metadata摘要写入后取消不发布，旧metadata／页／F关联继续有效。

这是新固定投影和工作的证明。共同64MiB、Cache／RestoreNode／codec／JSON／Vec／Map、PNG no_limits、SQLite／VFS临时盘、无限Reader／宿主输出副本聚合仍欠证；A控制512KiB独立，不宣称ABI全部栈峰值或产品成本。

## 4. 回归与独立审阅

最终定向存储124项完整保留父F的116项并新增8项；application92项及全部集成／文档测试通过。八模型包括400／401／真实q2001／future3001、900→2两Manual变更metadata、旧列表重排等值及误用最新列表的反事实不等；非递增CommitId不重排marks，UTF8／CRLF／NUL标签、非UTF8及空扩展、后增optional均保留。

自制变化文件先经旧standalone load成功，再核对历史摘要和独立保存的features根；坏例包括未知required、缺必要项、旧／未来不在global的feature、required冲突、扩展SHA／长度错、重复tag。每例先锁旧StorageError真因，再锁新端口Unsupported／ValidationFailed，避免I/O／Busy／预算假阳性；guards／拓扑／计数全部恢复，不修改真实用户文件。

首轮只因新测试sum类型推断失败而未执行测试；显式u64后通过，随后加强负例并再次全过。Clippy发现测试路径多余借用，直接修正、未抑制lint；最终Clippy及124／92回归通过。两路实现／向量／回归审阅无剩余阻断，审查者未编辑或构建；重型工作主线串行。精确日志散列与测试集合见[结构化证据](P3_2B1_2G_PROOF.json)，正常完整hook与新SHA三项CI分别核对，不套用父结果。

## 5. 剩余退出与备份边界

真实q的features传递／完整metadata资格仍缺，后续须另行明确版本化来源链和产品语义，不能偷渡writer默认值。CurrentManual／CurrentProtection／VerifiedOpening及NoChange、Unknown阶段／目标连续性、注册物理绑定、共同预算和生产authority继续欠证；完整B1／PB16未签收，不跨B2或恢复部署／迁移／清理。

保持Experimental／Insufficient／BudgetUnproven；K1／Reference10000／Tiny4、产品NotEvaluated／stable Unknown不变。旧first-before与两次取消根因仍Unknown，近期成功不代表修复；25分钟CI／30秒门控不变。十一源码＋十二文档正常完整hook／精确路径PUBLIC推送／新SHA核验；旧证明／冻结程序／PRIVATE和schema.rs原字节／mtime保留。新增原始忽略日志仅本机，不自动扩张旧私有快照；仅自制fixture和显式测试根，用户返回未知，现场UAC／截图／测试等待，UI／Skill暂停。
