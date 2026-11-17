# P3-2B1-2D 实现与独立审阅

2026-11-17。本片按[冻结契约](P3_2B1_2D_CONTRACT.md)加强实验FileFactsReader的逐代原件时序准入；保持Experimental／Insufficient／BudgetUnproven，不新增保存资格或公开历史原件查询。

## 1. 起点与切片选择

父提交2e869da6c3b71120022eb14e3751d6c1611d89e1已推送PUBLIC origin，[run44053396988](https://github.com/sumiyuki727/skin-editor/actions/runs/44053396988)三项成功。轮前实际／暂存差异为空，复用回退点不造空提交，877文件字节／mtime基线保存在本机忽略目录。schema.rs既存空差异标记及原字节保留。

两路独立设计审阅确认可复用已完整验证的块图，不必先重写codec。契约SHA256为d45afbb35debf54855634d442e3c0fc6504fd3da199540428635c24d30e4b52d，两路审阅及合成图上限措辞澄清均在源码实施前完成。最小范围选择严格校验，不附加新公开摘要／端口；完整历史q及资格继续另行收口。

## 2. 实际新增行为

[严格helper](../desktop/infrastructure/src/project_storage/file_facts/historical_originals.rs)只借用完整验证后的Row.refs／Row.blobs。按各代node_count遍历全部节点前缀，从原record.state起步，跨状态累计已入队位图与原件集合；共享、未使用、后来删除及旧分支素材均纳入。全局8项BlobRef只作为SHA到已核对长度的字典，历史集合独立累计。

声明侧只读originals索引到页的声明；原件索引不能带blobs，页不能带状态refs。每代累计声明须与真实累计闭包精确相等。完整文件原始字节、SHA／长度、节点／记录、codec字段和refs／blobs一致性的前置验证均保留，不只检查选中的历史截面。

[读取接线](../desktop/infrastructure/src/project_storage/file_facts.rs)复用原有逐代checkpoint解析，同时取得legacy_request和原件索引根；[read访问器](../desktop/infrastructure/src/project_storage/read.rs)仅暴露已解析的根。无新增JSON解析、read_paged、StateBundle复制、decode／encode、文件读取、Cache字段或writer修改。任何代次错配或取消拒绝整个新观察，不发布部分facts。

API7／捕获3／协调1、既有查询与文件／记录版本均不变。旧standalone读取仍没有本片严格时序资格，旧writer能力不变；不修复或改写旧文件。B1-2C公开查询仍只有节点／admissions内容域，不含历史originals摘要或比较。真实q、Ticket／expected、selectedCommit及恒Insufficient结论保持，不安装A2事实或签发PreparedWitness。

## 3. 新固定容量与真实释放

Work包含8192项借用Row指针索引、8192项u16队列、1024字节位图、8项BlobRef字典及固定标量。索引按已有块hash顺序填充并二分，入队时立即标记，每块跨全部状态最多入队一次；重新处理已见根不增队列。上限、根数量、前缀单调及索引转换在使用前核对；节点、边、页和原件循环检查取消。

新增WORK_BYTES静态式为2×Work＋2×128项根数组＋2×8项BlobRef字典＋128×usize辅助余量，并断言不超过192KiB。双份覆盖数组构造／目的载荷重叠；生产helper只返回()。inspect在调用validate前独立申请临时租约，与原2Data、路径和存活候选／观察共用既定Reader512KiB；validate及其所有工作实际退出后才退还临时额，观察继续持有原有额度。

观察成功后的长期使用量与新瞬时峰值分别验证；不能把临时工作永久附在观察，或用调用取消代替实际释放。失败、源复核及scratch清理仍走原路径；Reader先退出时，旧观察／候选继续保管自身实际资源。

这只是新固定工作区的容量证明，不是完整64MiB证明。旧Cache、累计RestoreNode、codec／JSON／Vec／Map、PNG、SQLite内存与VFS临时盘仍欠共同分配前证明。A控制512KiB与Reader池分离，不证明无限Reader／宿主聚合或运行性能。

## 4. 回归与发现

定向范围现在包含原B1-2C的92项、既有内部metadata回归及本轮8组，共101项。Clippy已通过；最终精确源码的定向日志、正常完整hook及新SHA三项CI仍各自核对，详见[结构化证据](P3_2B1_2D_PROOF.json)。[四组独立模型](P3_2B1_2D_VECTORS.json)由Python集合／hashlib产生，另一审查独立复算全部原始长度／SHA、逐代集合、位图、数量、字节及接纳结果。

| 组别 | 关键证据 |
| --- | --- |
| 独立累计模型 | 共享／未使用／后删、零字节成员、提前／迟到声明；从原state根开始，canonical缺席；合成图不冒充合法迁移文件 |
| 密集共享图 | 130块密集DAG只入队130次，重复根不再入队，能识别出队才去重造成的重复压力 |
| 固定边界 | 合成图8192可接纳、8193拒绝；8原件含零字节可接纳、9拒绝，缺根／字典及根数组错配拒绝；不称真实文件满额证明 |
| 真实时序缺口 | 自制两提交分别交换早／迟原件索引根，仅重建checkpoint及相关哈希、恢复拓扑序与全部guards；旧full先成功，新观察再拒绝，源字节及额度保持 |
| 真实累计语义 | 未使用／共享原字节、零字节、后来删除及旧分支独有素材仍完整验证，B1-2C旧查询保持有效 |
| 实际取消 | 图边／原件引用已处理后取消，无部分观察；旧页、源字节、真实Work、Reader→观察→候选Drop及Ack保持 |
| 捕获准备边界 | 独立第一／第二候选prepare的完整N／N−1；与保管候选后的观察瞬时峰值分开测试 |
| 保存准备边界 | 完整extra＋context_work＋capture持有／工作峰值N／N−1；真实Ticket、Ack等待资源、Work实际释放及最终Ack |

新增临时工作使旧低额场景先在inspect拒绝，因此原聚合用例明确改为先保管候选，再测观察N／N−1。长期持有、旧候选、页和实际释放断言保留；另补独立prepare边界，没有仅放大预算或移除失败分支。独立实现审阅发现保存完整峰值的覆盖尚需单独补回，已新增该回归并经复核通过。

首轮只在新测试迭代器签名处编译失败，没有执行测试；修正后定向通过。随后补保存峰值和观察长期额度断言，分别复核；全部原始日志保留，不把首轮编译失败计为运行通过。两路实现／向量审阅均无剩余阻断，审查者未编辑或构建，重型工作仅主线串行。

## 5. 剩余退出条件与保管

本片补齐实验观察的逐代原件声明与状态闭包核对，不提供历史原件摘要／真实q的完整关联证据。完整历史q、features真实来源／完整metadata、CurrentManual／CurrentProtection／VerifiedOpening及NoChange覆盖、Unknown阶段／目标连续性、注册物理绑定和共同预算仍欠，完整B1／PB16未签收，不进入B2／生产保存／恢复部署或清理。

旧first-before失败及两次CI取消根因继续Unknown；后续成功、build配置或诊断改善均不改写旧结果。全部旧测试、预算、25分钟CI和30秒门控保持。K1、Reference10000／Tiny4、产品NotEvaluated及stable Unknown不变。

本片九源码与十二文档按精确路径、正常完整hook、PUBLIC origin及新SHA三项CI收尾。旧证据／冻结程序／PRIVATE材料与schema.rs原字节保持；新增原始日志仅本机忽略目录，不擅自上传或自动扩充固定恢复快照。仅自制fixture与显式测试根，不触真实用户工程／素材／恢复材料／个人数据。用户离开、返回时间未知，UAC／截图／现场测试等待；UI和皮肤Skill暂停，不接GUI／原生设备。
