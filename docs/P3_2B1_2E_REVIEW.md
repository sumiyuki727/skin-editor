# P3-2B1-2E 实现与独立审阅

2026-11-17。本片按[冻结契约](P3_2B1_2E_CONTRACT.md)提供历史原件只读集合、域2摘要及真实q对照，保持Experimental／Insufficient／BudgetUnproven；不是完整历史q或保存资格。

## 1. 起点与先行冻结

父38c0c915397f44b4bffe9852cb078305bb3e7eb4与PUBLIC origin一致，[run44068675153](https://github.com/sumiyuki727/skin-editor/actions/runs/44068675153)三项成功。实际／暂存差异为空，复用回退点，不造空提交；883文件字节／mtime基线仅本机，schema.rs既存空差异标记及原字节保留。

两路设计审阅后，在任何源码改动前同时冻结契约与[七组独立模型](P3_2B1_2E_VECTORS.json)：SHA256分别为27f8c52f6db810085e264772a51f77a3cbc9cc866f4f1eace0d09717ee181f9f、1f47f3e22b8526765601a5e98c3e25df413314670b5626abb3e42c6fbaddd694。模型由Python集合／hashlib按字面TLV规范组装，另一审查独立复算全部原字节长度／SHA、entries、mask、字段流及摘要；旧完整域2黄金值保持。

## 2. 实际新增行为

[application接口](../crates/application/src/file_historical_originals.rs)新增独立FileHistoricalOriginals1只读trait与固定纯值。请求保留完整ObservationId、FileFactsRequest和选中Commit；结果保留原请求及ExperimentalValidation，facts含具体FileCommitFacts、proof_version、域2digest、数量／总字节及8项密集Option<BlobRef>明细。前len项Some，余下None；空字节原件仍为Some并带真实空字节SHA。

[查询与比较](../desktop/infrastructure/src/project_storage/file_facts/original_query.rs)只从已保管固定槽按mask筛选历史成员，原件明细严格SHA序，不把未来成员混入旧结果。缺提交为None；比较即使Missing也复用compare_save的完整Ticket、Reader、root及请求／绑定守卫。返回实际q意图／原票expected与独立选中提交、可选originals_equal，结论恒InsufficientForSaveReconciliation；不提供all_equal、Saved或写许可。

实际原字节由同次完整读取逐件校验SHA和长度，真实q则沿用原prepare读取／核对。全局字典只解释本次位号，历史集合来自D逐代验证的actual位图。新增未来原件SHA排在旧原件前面时，旧位号可改变，旧选集的字节流及摘要仍相同。

[D helper](../desktop/infrastructure/src/project_storage/file_facts/historical_originals.rs)继续返回()，在每代actual等于声明后直接填未发布Data固定槽，无第二次图遍历、raw读取、checkpoint解析或恢复模型。当前完整域与历史域共用[original_set](../desktop/infrastructure/src/project_storage/file_facts/proof.rs)：既有FileCoord前缀／domain2，count-u64后逐项SHA32／length-u64，严格SHA序，重复／数量／字节溢出拒绝。末代digest、数量、字节显式核对现有完整summary，不用零hash表示空集合。

API7／捕获3／协调1、旧文件／记录格式及既有端口版本不变。原件域不包含类型／配置／head／导航／features或完整metadata；原件相等不能推导这些领域、提交身份或保存覆盖相等。无PreparedWitness转换、事实安装或生产authority，旧standalone读取／writer能力不变。

## 3. 固定容量与保管

Data新增一次8项字典及128项{mask,digest,bytes}，不逐代重复存8份原件明细。查询在最多128提交中定位，再按8位集合复制固定结果，不读文件或访问Cache。宿主任意保管输出副本的成本不在单Reader池证明内。

新字段随原2×Data计费，静态式额外列入固定新Data构造重叠。D临时WORK_BYTES具名追加2×Encoder、可选BlobRef、SHA／计数及2×OriginalSummary工作，仍断言不超过192KiB；不把新成本塞进旧辅助余量。两类预留与路径、旧观察及真实q共用既定Reader512KiB；validate前取得临时额，实际退出后退还，观察仍持有自己的2Data与Source。

已写入第一代真实摘要后可以取消；新Data全部丢弃，旧页／历史结果／q／原票和源字节保持，Work及观察实际Drop前不退其额度。旧独立prepare、完整prepare_save及聚合inspect的N／N−1继续运行，按新布局计算而不提高预算。

这是新增固定投影及编码工作的证明；旧Cache、累计RestoreNode、codec／JSON／Vec／Map、PNG no_limits、SQLite及VFS临时盘仍欠共同64MiB证明。A控制512KiB独立，不证明无限Reader／宿主聚合或产品成本。

## 4. 回归与独立复核

最终定向存储108项通过，完整保留父D的101项并新增7项；application90项及文档测试通过，新增固定载体检查和一项无保存见证转换的compile-fail。Clippy已通过；正常完整hook、新完整SHA三项CI仍各自核对，不套用父结果；准确来源与日志散列见[结构化证据](P3_2B1_2E_PROOF.json)。

| 组别 | 关键证据 |
| --- | --- |
| 独立域2 | 七组空／零字节／同量异字节／双raw排序／共享去重／位号前移模型，字段流与旧黄金一致；编码器单独拒绝重复、倒序、错数量、总字节溢出和取消 |
| 公开查询 | 真实golden各提交明细密集且SHA正确，完整上下文／版本／Experimental保留，缺ID为None，源字节不变 |
| 真实q后续 | 实际Work→候选→q真实提交→新增SHA0209…位于旧SHA222d…之前；旧历史true、最新false，原expected与observed latest分开保留 |
| 结论分离 | selected400原件等于q而内容域不同，提交身份也不同，仍只Insufficient；不取得保存覆盖 |
| 同量异raw | 从合法Upper文件load_driver→attach_work得到真实Work，与另一合法Old文件经来源守卫对照；均1原件／17字节，摘要分别锁定预冻结case3／case2，结果false |
| 历史语义 | D真实累计正例补逐代空集合、共享／未使用／后删／旧分支／零字节的entries、len、bytes及末代digest；零字节精确核对排序后的空字节SHA／0长度 |
| 来源与取消 | 错完整请求／版本／观察／Reader／票据拒绝，Missing仍守卫；第一代摘要完成后取消不发布，旧页／结果／q与实际Drop／Ack保留 |
| 新容量 | 新Data峰值N／N−1、长期持有量及查询不增管理计数；父D早迟拒绝、完整prepare峰值、图边界及未来损坏整文件拒绝保持 |

首轮编译仅发现新测试把HistorySessionId强类型当整数修改，没有执行测试；改为合法另一身份后通过。另将同量反例的真实Work来源写明并锁定双方独立摘要，独立审阅建议的零字节精确明细断言已补。失败及后续日志全部本机保留。两路实现／向量审阅无剩余阻断，审查者未编辑或构建，重型工作主线串行。

## 5. 剩余退出门槛与备份

本片仅补历史原字节集合域。完整历史q多域关联、features真实来源／完整metadata、CurrentManual／CurrentProtection／VerifiedOpening及NoChange覆盖、Unknown阶段／目标连续性、注册物理绑定与共同预算仍欠，完整B1／PB16未签收。不进入B2、生产保存或恢复部署／迁移／清理；旧first-before失败及两次CI取消根因继续Unknown，不以近期CI成功追认修复。

十一源码与十二文档按精确路径、正常完整hook／PUBLIC推送及新SHA三项CI收尾。全部旧测试、资源预算、25分钟CI和30秒门控保持。旧证据／冻结程序／PRIVATE与schema.rs原字节保留；新增原始忽略日志仅本机，不擅自上传或自动扩充固定恢复快照。

仅自制fixture／显式测试根，不触真实用户工程、素材、恢复材料或个人数据。用户离开、返回未知，UAC／截图／现场测试等待，UI与皮肤Skill暂停；K1、Reference10000／Tiny4、产品NotEvaluated／stable Unknown不变。
