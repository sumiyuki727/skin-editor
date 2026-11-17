# P3-2B1-2C：历史节点内容域开工契约

2026-11-17。轮前回退点3c527578ab4dc5d03a69fae181d54d41d311d1b0已推送PUBLIC origin，run44034697724三项CI成功。暂存及实际差异为空，复用HEAD，不造空提交；既存schema.rs标记／原字节、旧证据与PRIVATE材料保留。870文件原字节／mtime基线另存本机忽略目录。

## 1. 选定范围

在[B1-2B剩余条件](P3_2B1_2B_REVIEW.md#6-剩余退出条件与保管)中，本片只补历史节点／admissions的FileCoord内容域。文件已完整验证、Cache仍存活时按每代节点前缀流式计算固定摘要，Cache退出后提供只读查询及真实q比较。它不是完整历史q截面，不生成历史原件／metadata／features完整资格，不安装保存事实或解除Unknown，不进入B2、生产保存、恢复部署或清理。保持Experimental／Insufficient／BudgetUnproven。

未选features是因为真实捕获来源及未知可选feature的保留语义尚未定约，不能拿write::features默认清单冒充q来源。历史原件也另有前置缺口：旧full逐代检查原件delta属于全局集合且不重复，并在最后核对总并集，却未逐代证明声明集合等于当代全部历史节点的原件闭包。本片不返回历史originals摘要／相等字段，不能借旧full成功补造这项证据。

后续历史原件专片至少须从原record.state的已验证块图计算原件闭包（canonical根未必存在于块表），包含未使用及后来删除的素材、共享只计一次，并逐代与原件声明集合核对；固定遍历须入队去重且预留工作容量。本片不实施该图遍历或新增旧文件修复。

## 2. 前缀与字节语义

旧read.full已按每代nodes delta逐项确认cache.order[seen_nodes.len()]，admission只能随其首次节点登记，并验证head／Redo位于该代、prefix／计数器／legacy请求。本片据此使用order[..checkpoint.node_count]；不能按当前头祖先、DFS顺序或持久ID大小截取，也不能将节点数量当alias上限。

内容精确复用FileCoord-Proof-v1域1：project／history／lineage／address_epoch／root五个u128；前缀node_count:u64；各节点的id、optional parent、alias／branch／ordinal／depth／record_version、原state／action／checksum及已缓存canonical；最后是首入节点属于该前缀的admissions数量与按既定kind／id顺序的kind／id／first_node。保留原字节与canonical的区别，不重编码／恢复项目，不改旧五域封套或向量。

前缀成员直接按order[..node_count]成员关系判断，支持合法计数器耗尽的0值，不用next_alias、持久ID大小或alias数值猜成员。共享一个content helper给当前完整域和历史域，补取消检查，完整前缀的字节输出必须保持原黄金值。历史内容域本来不含head、origin、计数器、Redo、metadata或原件完整性；这些字段不得因content相等而被推断。

## 3. 独立接口与来源

新增由application定义的独立FileHistoricalContent1只读端口及纯值：查询携带version、完整ObservationId、完整FileFactsRequest（含observed expected及selection）和selected CommitId。查询只接受与此观察完全相等的上下文；返回的固定外层结果保留版本及完整原查询，内部为可选FileHistoricalContentFacts，脱离方法调用后的副本也可核对来源。缺ID为None，不是NotCommitted。facts保留该具体FileCommitFacts、节点／admissions数量、域1摘要及独立的intent navigation摘要，不把完整checkpoint块hash叫导航子集。

新比较携带上述查询及完整原SaveTicket，消费不透明FileSaveCandidate。复用既有compare_save来源守卫核对完整Ticket、Reader实例、root及绑定／请求范围；只有原q.expected与已验证观察latest可不同。原Ticket与expected原样保留，不能改Ticket来调用旧接口。查询上下文本身仍须准确匹配观察latest，不接受旧页／旧观察／错误selection／其它绑定或Reader。

比较结果保留完整外层查询结果和实际q意图／原Ticket，分别返回可选content_equal、head_equal、navigation_equal及选中facts；缺ID时facts与三个比较值一致为None，不返回半个比较。selected CommitId允许不同于q.ticket.commit，二者同时保留，不能静默改选原票commit。不提供all_equal／Saved／NotCommitted或历史originals相等。不同提交可有相同内容，head／navigation仍可不同；全真也只有InsufficientForSaveReconciliation。选中旧Manual／Protection不代表CurrentManual／CurrentProtection覆盖合格。本片不连接生产authority、PreparedWitness或A2事实安装。

API7／捕获3／协调1、存储1、FileFactsPort1、FileCoord1、FileCaptureProjection1及FileSaveIntent1不变；FileHistoricalContent1独立版本化，既有结果结构不变。公开值不能向写许可／保存见证转换，不暴露Cache、可变项目或运行时句柄。

## 4. 新增容量与工作

每观察最多128提交，新Data只追加固定的128个内容digest和128个admissions计数，复用既有FileCommitFacts与导航数组。历史helper串行借用同一Cache，不新增Vec／Map／JSON／Redo副本，不再次read_paged、checkpoint解析、图遍历、codec encode/decode或RestoreCandidate。

新增固定hash／标量工作与返回数组按真实size_of列式纳入现有2×Data预留，静态断言覆盖旧intent结果、summary／preflight与新helper结果生产重叠；helper退出后才构造Data。观察、候选及当前工作仍共享同一Reader512KiB，N／N−1测试须验证新增静态布局后的聚合准入、取消及实际所有者退出。A控制512KiB是另一池。

按提交、节点及admission有界遍历并检查取消；查询只扫描最多128固定槽、复制有界纯值，不执行文件或验证工作。输出类型限定固定容量；宿主无限保管查询副本不在单Reader池证明内。旧Cache／完整恢复／codec／PNG／SQLite／临时盘仍BudgetUnproven，不因本片固定载荷而宣称64MiB／PB16或无限Reader宿主聚合通过。

## 5. 验收边界

| 组别 | 必须验证 |
| --- | --- |
| 独立黄金 | 从既有自制黄金模型独立生成三节点及四节点域1字段流／SHA；完整域维持原值，记录原state与canonical |
| 真实q | restore→真实Work→候选；真实q提交后追加分支／新素材，选中旧提交内容域相等、当前完整域不同；原Ticket／expected不改 |
| 语义分离 | 同节点前缀的不同提交可content相等而head／导航／来源不同；NoChange旧Manual点不被内容相等提升为保存资格 |
| 完整验证 | 后续分支／原件损坏必须使整个新观察失败，不能仅校验目标旧前缀；不返回部分历史facts |
| 隔离 | 缺提交None；版本／旧观察／完整file请求／错误selection／target／Ticket及不同Reader拒绝，保留旧有效观察／页 |
| 取消与容量 | 历史批次至少完成一个摘要后取消不发布结果，节点／admission循环仍逐步检查令牌；新布局下N／N−1及存活候选／观察聚合，实际Drop前不退额；已有84项存储及全部正常核心／桌面检查保持 |

独立只读审阅后冻结契约及测试边界再实施。主线唯一写入及重型构建串行；另一只读审查可梳理完整预算欠项。收尾精确路径、实际暂存审查、正常完整hook、PUBLIC origin推送及新完整SHA三项CI；不借轮前成功作本片证据。

旧first-before根因及两次CI取消根因仍未确认，诊断改善或后续CI成功不能改写旧结论。全部测试、资源预算、30秒门控、25分钟CI及检查规则保持。用户离开，UAC／截图／现场交互等待；UI与皮肤Skill暂停。仅自制fixture和明确测试根，不触真实用户工程／素材／恢复材料／个人数据；新增原始忽略日志仅本机，不擅自上传。
