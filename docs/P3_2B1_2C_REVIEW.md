# P3-2B1-2C 实现与独立审阅

2026-11-17。本片按[已冻结契约](P3_2B1_2C_CONTRACT.md)实现历史节点／admissions内容域的只读查询与真实q比较。保持Experimental／Insufficient／BudgetUnproven；不是完整历史q、NoChange覆盖或保存资格，完整B1／PB16仍未签收。

## 1. 切片选择与前置审阅

轮前HEAD／PUBLIC origin为3c527578ab4dc5d03a69fae181d54d41d311d1b0，[run44034697724](https://github.com/sumiyuki727/skin-editor/actions/runs/44034697724)三项成功。实际／暂存差异为空，复用回退点而不造空提交；870文件原字节／mtime基线另存本机忽略目录。根AGENTS适用，无新增本地skill要求。

两路独立只读审阅分别梳理来源／测试和完整预算。选择本片是因为旧full已证明每代节点为cache.order的追加前缀、admission只随首次节点登记，而features真实来源和历史原件闭包仍各有缺证。实现前冻结契约SHA256为2a85ea01d8a3ec4c20e09b3c508590da9e7d2ed7b4b321cef8c93871b3c165ad；输出来源／独立selectedCommit语义及容量重叠要求均已合入。

## 2. 实际新增行为

[application](../crates/application/src/file_historical_content.rs)定义独立FileHistoricalContent1只读端口、完整查询、固定外层结果及q比较值；API7／捕获3／协调1、存储1和既有端口／投影版本均不变。外层结果保留完整ObservationId、FileFactsRequest和选中CommitId，比较另保留实际q意图／原Ticket。缺ID时facts和三个比较值均为None，不能叫NotCommitted。

[基础设施](../desktop/infrastructure/src/project_storage/file_facts/historical_content.rs)在完整文件已验证且Cache仍活着时计算每代固定摘要，之后释放Cache，查询只扫描最多128固定槽。完整当前域与历史前缀共用[域1编码器](../desktop/infrastructure/src/project_storage/file_facts/proof.rs)，直接按order[..node_count]成员关系筛admission，不用持久ID、alias数值、当前头祖先或耗尽后的计数器猜前缀。

域1包含文件固定身份、全部前缀分支节点的原record／canonical及admissions，保留原state／action／checksum与重新计算canonical的区别。历史helper不新增JSON、Vec、Map、checkpoint解析、图遍历、codec encode/decode或RestoreCandidate；既有全文件验证未减少。当前完整前缀保持既有FileCoord黄金值。

q比较复用既有compare_save的完整Ticket、Reader、root及请求／绑定守卫；原expected不改，观察请求仍准确声明本次latest。分别返回content_equal、head_equal、navigation_equal。selectedCommit可不同于q.ticket.commit，两者都保留；即使三个值全真，选中提交／来源仍可不同，结论始终InsufficientForSaveReconciliation。不安装A2事实，不签发PreparedWitness或生产authority。

## 3. 明确未补齐的历史原件证据

新结果没有历史originals摘要或相等字段。旧full逐代检查原件delta属于全局集合且不重复，最终检查总并集，却未逐代证明声明集合等于当代全部节点状态的原件闭包。因此历史内容摘要不能证明该代原件完整，更不能借当前全局原件域冒充历史集合。

后续至少须从原record.state块图累计原件闭包，包含未使用／后来删除的素材、共享只计一次，再逐代对照声明集合。canonical根是重新编码所得，未必在原块表；遍历还须在入队时去重、预留固定工作容量，并覆盖原件提前／延后登记错位。本片仅登记这项欠证，不实施图遍历、旧文件修复或迁移。

## 4. 固定容量与实际释放

Data只增加128个32字节digest及128个usize计数，复用既有FileCommitFacts和导航数组。2×Data静态式保守计入旧intent工作／返回、新历史数组双份返回、summary／preflight及hash／标量工作；helper退出后才构造Data，未将同一额度重复用于实际共存载荷。

新增固定载荷仍与观察、候选和当前工作共享单Reader512KiB。Data／Source先于总租约释放，失败路径先丢弃结果再退额。新布局的N／N−1和已有候选＋观察聚合回归保持；查询管理计数不变是账本证据，无新增变长分配的判断来自源码中仅固定Copy值／有界迭代、没有Vec／JSON／I/O的审阅，不能据此宣称全路径无分配或64MiB通过。宿主无限复制查询值也不在单Reader池证明内。

历史批次在完成至少一个真实摘要后可取消，节点及admission循环也检查令牌。取消不发布部分观察，保留旧有效观察和页；源守卫及存活所有者继续保管实际资源，Reader先Drop也不能提前退额。

## 5. 回归、发现与独立复核

本机Windows存储定向92项通过：原84项保留，新增8组历史内容回归。Clippy通过。application新增固定纯值测试及两项compile-fail，正常完整hook另行执行后才能确认；以包含本报告的新完整SHA三项CI为最终检查，不套用轮前CI。[结构化证据](P3_2B1_2C_PROOF.json)记录准确源码、向量及原始日志散列。

| 组别 | 实际覆盖 |
| --- | --- |
| 独立字节流 | [三组向量](P3_2B1_2C_VECTORS.json)由Python结构化字段／hashlib产生，另一只读审查重建43／54／18字段TLV及SHA；完整前缀维持旧值 |
| 原记录区别 | Some原state不同于canonical，节点共享编码器保留两者；该合成样例仅证明序列化，不是合法旧文件迁移或原件资格 |
| 真实q与后续 | restore→真实Work→候选；真实q提交后新增分支及未使用素材，旧q内容域相等、当前完整域不同；原Ticket／expected原样保留 |
| 语义分离 | 内容相等但头／导航不同；内容／头／导航全真而提交／来源不同，仍Insufficient；缺ID保留完整上下文且所有可选比较为空 |
| 完整验证 | 仅损坏未来分支或未来原件也必须使整个新观察失败，不仅检查目标旧前缀 |
| 隔离／所有者 | 版本、完整file请求、selection、Ticket、旧观察及不同Reader拒绝；旧观察／页保留，源Busy不放宽，Reader先退出仍等实际所有者 |
| 取消／容量 | 历史摘要实际完成后取消、无部分结果、源字节保护、N／N−1、查询管理计数不增长及既有候选聚合 |

首轮89过／3失败均来自新测试错误使用活跃源独占守卫：在观察仍存活时再次直接读取或重开同一源。修订为预先创建独立自制副本，并明确断言同源重开Busy；校验源字节时先释放相应守卫，没有放宽分享模式或检查。第二轮92项通过，失败日志保留。随后只精确化一项测试名称为queries_keep_managed_usage，不改行为；最终完整hook须运行该新名称。

两路实现审阅均未发现新增阻断；一条明确建议是不要仅凭usage不变声称无分配，已在名称和上述证据边界中落实。重型构建由主线串行执行，审查者未编辑或构建。

## 6. 剩余退出条件与保管

仍欠历史原件时序闭包／完整历史q、features真实来源／完整metadata、CurrentManual／CurrentProtection／VerifiedOpening与NoChange覆盖、Unknown阶段／目标连续性、注册物理绑定及完整预算。Cache＋累计恢复模型＋当前codec闭包、JSON／Vec／Map分配、PNG no_limits路径、SQLite连接／VM／页缓存及VFS临时盘仍未获共同64MiB分配前证明。下一预算专片可先验证一个codec闭包的实际capacity、共享与取消释放；这是建议，不是已实现。

Reader512KiB与A控制512KiB独立，不证明无限Reader宿主聚合；CI的debug／TEMP配置与这些预算资格无关。旧first-before根因及两次取消根因仍未确认，后续CI成功不改写旧结论。全部旧测试、预算、25分钟CI、30秒门控和检查规则保持。

本轮九条源码及十二条文档精确路径仍须通过正常完整hook、PUBLIC origin推送及新SHA三项CI。schema.rs既存空差异标记／原字节、旧证明、冻结材料及PRIVATE恢复附件保持；新增原始日志仅本机忽略目录，不擅自上传或自动纳入固定历史恢复快照。用户离开，UAC／截图／现场交互等待；本片无需现场操作。UI与皮肤Skill暂停，不触真实用户工程／素材／恢复材料或个人数据。K1、Reference10000／Tiny4、产品NotEvaluated及stable Unknown不变。
