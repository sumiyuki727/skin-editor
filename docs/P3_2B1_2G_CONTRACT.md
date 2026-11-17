# P3-2B1-2G：文件侧历史metadata来源与摘要契约

2026-11-18（Asia/Tokyo）。父提交bc0b935bb6700dc0ae04fea029235300cb369f55与PUBLIC origin一致，run44104073562三项首次运行成功。实际／暂存差异为空，不建空提交；897文件原字节／mtime基线仅本机，schema.rs原标记、旧证据及PRIVATE材料保留。

## 1. 实际缺口及本片边界

features实际位于文件SQL全局表及各checkpoint的features根；旧read逐代验证必要项和全局成员关系。write::features仅是四项默认必要能力及读取白名单。ArchiveMetadata、RestoreHeader与CaptureManifest没有features字段，当前真实q没有完整来源链；不能从q.expected、同ID、当前观察或writer默认值补造q.features。本片不改捕获／持久格式或产品语义，F.metadata继续NotCompared，不计算q metadata_equal。

文件侧已有足够证据：各代tags／extensions／features根，以及按代次重建的receipts／Manual marks。新增FileHistoricalMetadata1只读查询，补齐完整旧读取器所支持的文件历史域3；这不是完整历史q。保持Experimental／Insufficient／BudgetUnproven，不安装事实或生成见证，不跨B2／生产保存／恢复部署、迁移或清理。

## 2. 借用已验证材料

read保留原入口，默认无observer，不新增Small／128点限制或固定工作分配。FileFacts专用入口接私有可选窄observer；在本代三列表经过validate_past_metadata且last/manual/protected/manual_marks更新后、push checkpoint前同步借用列表。只生成未发布固定槽，任何后续full、D严格原件时序、源复核或清理失败都丢弃。

tags使用本代原值；extensions只按本代声明名字、SHA和长度，从同次已验证全局表借原字节；features只用本代已验证列表。meta只提供本代receipts／marks及扩展原字节，不直接用最终meta.tags或全局features代替旧列表。旧格式只支持历史扩展仍能在当前全局表中按同名／SHA／长度取得的字节；缺失、删除或同名改字节继续沿旧full拒绝，不宣称通用历史扩展迁移支持。

按数值node/name、extension name、feature name/version规范排序；Manual marks按代次保留，不按CommitId排序。不因历史列表原顺序不同而拒绝，最新列表仍须满足旧读取器与SQL索引的精确校验。未知required／缺必要项／历史feature不在全局及扩展绑定错误照旧整文件拒绝；未知optional必须原样影响摘要。

## 3. 字节、接口与来源

精确复用FileCoord-Proof-v1域3：last/manual/protected、marks、tags、扩展名字及真实bytes、features三字段；前缀／TLV和旧黄金不变。提取小的头及逐字段编码helper供当前完整域3和observer共用，不新增编码域或版本。末代摘要必须等于现有summary.proof.metadata，数量也核对完整已验证metadata。

独立版本1请求保完整ObservationId、完整FileFactsRequest和selected CommitId。结果保版本、原请求、ExperimentalValidation，facts为Option；Missing为None，不是NotCommitted。facts保proof_version、完整FileCommitFacts、post-commit域3摘要、features_root以及tags／extensions／features／required_features／manual_marks计数和extension_bytes。checkpoint.navigation自身绑定全部三根，features_root额外明确实际能力来源；不返回只有计数而无摘要的伪完整证据。

查询只消费观察内固定数据，不做I/O、解析、哈希或项目写入；version、完整请求及观察错配拒绝，旧观察在自己的上下文内仍可查询。没有q比较端口、可变项目／资源引用或PreparedWitness转换。API7／捕获3／协调1及既有接口、文件／记录／证明版本保持。

## 4. 固定容量与实际释放

新增128个私有Point，每项不超过80字节：digest32、features_root32、五个u16计数和u32扩展总字节。转换checked，受既有Small tags≤8192、extensions≤1024、features≤256、marks≤128、metadata≤1MiB约束，不缩窄旧支持范围。工作索引为u16[8192]，具名计入两份构造重叠、Encoder及Point／局部值，总式不超过40KiB；不能误用4096 tags上限。

新Point数组及构造重叠纳入2Data静态式，observer与D工作在validate帧前合并预留，实际validate退出后才退额。既定Reader512KiB内保留全部旧单／多观察、捕获及真实q聚合N／N−1回归；新增存活旧观察＋真实q＋下一观察（合计两观察）的准确容量与失败保管回归，不提高预算，不删原测试。若容量条件不成立，先调整固定布局／生命周期，不能抬限。固定输出不增长Reader管理计数；宿主无限副本及ABI全部栈峰值不属于此证明。

共同64MiB仍欠Cache／RestoreNode／codec／JSON／Vec／Map、PNG no_limits、SQLite／VFS临时盘及无限Reader／宿主聚合。A控制512KiB独立，本片不关闭这些欠项。

## 5. 冻结验收

两路独立只读审阅后冻结本契约和Python独立结构模型／完整TLV／SHA，再改源码。至少覆盖：400／401／真实q2001／future3001四历史截面；两Manual提交900→2的变更metadata（UTF8／CRLF／NUL tags、非UTF8及空扩展、后增optional、旧列表乱序），旧值不能被最新替代；marks按代次。

真实文件负例覆盖未知required、缺必要项、历史feature不在global、旧extension SHA／长度错误，以及未来未选中metadata损坏整文件拒绝。边界覆盖8192索引接受／8193拒绝（局部纯值不冒充完整合法文件）、全部版本／请求／观察隔离、首个真实metadata摘要完成后取消、旧页／F关联／q及Reader→观察→q实际Drop／Ack、新布局／工作与多观察N／N−1。既有域1／2／3／完整黄金和全部旧测试保留，F metadata始终NotCompared。

主线唯一写入／重型构建串行；正常完整hook、精确路径PUBLIC推送及新完整SHA三项CI。旧first-before与两次取消根因仍Unknown，25分钟CI及30秒门控不变。仅自制fixture与显式测试根，不碰真实用户内容；用户返回未知，现场UAC／截图／测试等待，UI与Skill暂停。新增忽略原始日志仅本机，不扩充旧私有快照；K1／Reference10000／Tiny4、产品NotEvaluated／stable Unknown保持。
