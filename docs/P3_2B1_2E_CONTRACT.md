# P3-2B1-2E：历史原件摘要与真实q对照契约

2026-11-17。父提交38c0c915397f44b4bffe9852cb078305bb3e7eb4与PUBLIC origin一致，[run44068675153](https://github.com/sumiyuki727/skin-editor/actions/runs/44068675153)三项CI成功。实际／暂存差异为空，复用回退点，883文件字节／mtime基线仅本机；schema.rs既存标记及原字节、旧证据与PRIVATE材料保留。

## 1. 最小范围与前置

在[B1-2D严格时序校验](P3_2B1_2D_REVIEW.md)的每代状态原件闭包等于累计声明之后，生成固定的历史原件集合与FileCoord域2摘要，并提供独立只读查询／真实q比较。保留原record.state起步、全部历史节点、共享／未使用／后删／旧分支原件及全文件验证；不再次遍历图、解析checkpoint、读取原字节或构造恢复模型。

所有原字节实际长度和SHA由同次既有full逐件读取、校验及codec／资源关联保证。每代集合来自严格校验的actual位图；全局最多8项BlobRef只作已验证SHA／长度字典，不能当作每代集合。后续新SHA插到旧SHA前面导致位号改变时，仍须按当次字典和实际位图选成员，不能取字典的前count项。

只补原字节集合域，不含资源类型／配置等节点语义、features或完整metadata；原件相等不能推导内容、head、导航、提交身份或保存覆盖相等。完整历史q、NoChange／Unknown、物理绑定和共同预算继续欠证。保持Experimental／Insufficient／BudgetUnproven，不安装保存事实、不签生产authority，不进入B2、产品保存或恢复部署／迁移／清理。

## 2. 集合与域2字节

规范精确复用FileCoord-Proof-v1域2：既有前缀／版本及domain=2，第一字段为唯一原件count的u64大端；之后按SHA字节严格升序，每个成员依次写32字节SHA与u64大端byte_len，沿用现有字段tag／长度封套。共享只计一次。计数、实际迭代数和顺序核对；总字节checked_add，非法重复／乱序／溢出拒绝。

空集合使用正式空域摘要，不是零hash；零字节原件仍是一个成员，count非零且带真实空字节SHA。不将mask、CommitId、ObservationId或运行Ticket放进域2；这些身份属于外层上下文。相同原件集合在不同提交可有相同摘要。完整末代的digest、count、bytes须与现有summary.proof.originals及完整原件计数／字节显式一致，旧完整黄金值不变。

当前完整域与历史域共用一个小的域2编码helper，实际q继续沿用已验证捕获的独立域2来源。编码不读取raw、不修改旧文件或格式。真实q原件已由prepare读取／核对，不能把捕获清单长度或来源指针冒充实际原字节摘要。

## 3. 独立接口与来源

application新增FileHistoricalOriginals1固定纯值和只读trait，不改FileHistoricalContent1等既有接口。请求保version、完整ObservationId、完整FileFactsRequest（含observed expected／selection）和selected CommitId。结果外层保版本、完整原请求及ExperimentalValidation，facts为Option；缺ID为None，不是NotCommitted。

facts保FileCommitFacts、FileCoord proof_version、域2digest、len和checked total bytes，并提供最多8项密集entries:[Option<BlobRef>;8]。entries仅包含所选历史集合，严格SHA序，前len项为Some，其余None；零字节仍Some。每项SHA与长度来自同次完整验证的字典，不能把未来成员带入旧结果。此公开固定上限由application定义；不是可变容器或资源所有权。

infra比较方法消费不透明FileSaveCandidate和完整原SaveTicket，复用既有compare_save的全部Ticket／Reader／root／请求绑定守卫。结果保实际q意图／原Ticket／expected、完整历史查询结果和originals_equal:Option<bool>，结论恒InsufficientForSaveReconciliation。比较核对域2digest、count及总字节；不返回all_equal、Saved或写许可。

只有原q.expected与本次已验证latest可不同，观察请求本身仍准确匹配latest；不能改票据迁就观察。selectedCommit可以不同于ticket.commit，两者同时保留。缺ID时facts与比较值都None，仍先执行真实q来源守卫；错误Ticket不能借Missing通过。旧观察在自己的作用域内仍有效，旧请求不能用于新观察或另一Reader。

API7／捕获3／协调1、存储／记录及现有端口／投影版本均不变，新增接口独立版本化。固定结果无PreparedWitness／保存事实转换，不暴露Cache、可变项目、原字节存储或运行时句柄。

## 4. 固定槽、重叠与取消

Data只新增一次8项字典及128项固定{mask,digest,bytes}槽。D helper直接填未发布Data的私有槽，继续返回()；每代先通过actual==declared，再编码并写本代，不返回另一大数组。不新增第二遍图／raw读取、JSON／Vec／Map或checkpoint解析。查询只在最多128提交中定位，按8位集合复制固定输出，不做I/O或验证工作。

新增Data字段计入原2×Data，静态式列入字典／固定槽构造重叠。D临时WORK_BYTES明确追加具名2×Encoder、可选BlobRef、摘要／计数及固定Summary辅助工作，不能吞进旧128×usize余量。两类额度继续与路径、旧观察及真实候选共享单Reader512KiB；临时额在构造前预留，实际工作退出后退还，固定结果随观察实际保管。独立prepare与inspect的N／N−1分阶段验收继续保留，不提高预算或删除失败分支。

新增取消探针只在至少一代真实原件摘要已写入私有槽后触发；沿用图节点／边、声明循环的取消检查，编码也检查令牌。任何后续代次失败、取消或末代一致性失败，整个新Data丢弃，不发布半个观察。旧页、旧历史结果、实际q／Work／原Ticket和源字节保持，实际所有者退出前不退其额度。

这只证明新固定载荷／编码工作。旧Cache、RestoreNode、codec／JSON／Vec／Map、PNG no_limits、SQLite与临时盘的共同64MiB仍未证明；A控制512KiB独立，不证明无限Reader／宿主／输出副本聚合或产品性能。

## 5. 独立验收

| 组别 | 必须覆盖 |
| --- | --- |
| 独立字节流 | Python结构化字段／hashlib生成空集合、零字节、双raw SHA排序、共享去重及位号前移的历史选集域2向量；完整末代维持既有黄金值 |
| 真实q后续 | 真实Work→候选→真实q提交→后续新增SHA排在旧raw之前的新原件；旧提交集合／摘要仍等q，最新不同；原票expected与observed latest分开保留 |
| 同量异字节 | 合法真实文件／真实Work经来源守卫通过，原件数量及总字节相同但实际raw不同，比较必须false |
| 历史语义 | 未使用／共享／后来删除／旧分支及零字节的真实文件，逐代entries／len／bytes／digest准确；原件相等不取得完整内容或保存资格 |
| 隔离／缺失 | version、完整file请求、Observation、Reader、Ticket及选择错配拒绝；Missing仍守卫，旧观察自身可查，原查询／选中提交／原票均保留 |
| 实际取消 | 第一代摘要完成后取消无半成品，旧页／历史结果／q仍有效；源字节、临时退额、Work／Ack／实际Drop保持 |
| 新布局容量 | N／N−1验证新Data与编码临时峰值／旧候选观察聚合；查询不增管理计数，输出副本宿主成本不冒充受测；保留D早迟拒绝及未来损坏整文件拒绝 |

两路独立只读审阅后冻结设计和测试模型再实施；主线唯一写入与重型构建串行。按精确路径、正常完整hook／PUBLIC推送及新完整SHA三项CI收尾，不套用父结果。

旧first-before失败及两次CI取消根因仍未知，不以近期CI成功追认修复。全部旧测试、资源预算、25分钟CI和30秒门控保持。用户离开、返回未知，UAC／截图／现场测试等待；UI和皮肤Skill暂停，仅自制fixture／显式测试根，不接触真实用户工程、素材、恢复材料或个人数据。旧证据／冻结程序／PRIVATE保留，新增原始日志仅本机，不擅自上传；K1、Reference10000／Tiny4、产品NotEvaluated／stable Unknown不变。
