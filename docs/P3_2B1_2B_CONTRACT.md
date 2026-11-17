# P3-2B1-2B：真实保存Work与只读意图对照

2026-11-17。基线1a3578484fb2fcaab315701bb02bfc731f62ff4d及run44006333067最新三项CI成功；旧桌面attempt1失败保留，根因未确认。无实际工作区差异，复用回退点不造空提交。本片在B1-2A后推进真实SaveTicket、原expected、请求、导航与ArchiveMetadata对照，仍不签收完整B1／PB16，不进入B2、产品保存、恢复部署或GUI／设备／Skill。

## 1. 真实Work与所有权

新增infra的prepare_save(work:SavePreparationWork, selection, originals, cancel)，不接SaveExecution、不取得authority、不偷取private capture。私有泛型CaptureStream统一旧HistoryCapture与真实Work的snapshot／statistics／poll；旧prepare_capture行为保持。FileSaveCandidate保管旧捕获候选、实际Ticket／只读意图、完整Work和固定控制租约；内层捕获候选的Snapshot及其自身租约先结束，Work及其WorkRelease后退出，覆盖Work包装的额外租约最后退额。外部Snapshot／Node仍按core捕获槽独立保管。

Work.poll(Complete)只产生CompleteNotSaved。候选保管期不能报告全部消费者已释放，Ack不自动回收Work；接口不发送保存完成、不自动Ack、不安装文件事实。实际Ticket来自不可裸值构造的Work，比较时核对完整Ticket，不接受同key下改写的副本。

prepare以实际Ticket构建原请求关联，重核Ticket／capture／coord版本、实际source／head／history、ExistingVerified及完整expected；API7由真实core request准入保证，Work／Ticket不另造API字段。Snapshot既有last_commit必须与原expected一致，不用文件当前值覆盖q。仅支持Manual／Exit等现有core实际准入来源，不增加新保存来源。API7／捕获3／协调1、FileFactsPort1／FileCaptureProjection1及文件格式不变；新FileSaveIntent1独立版本化，无向PreparedWitness／写许可的转换。

## 2. 原expected与观察latest

原Ticket.target.expected永远保留。观察仍用FileFactsRequest准确声明本次已完整核实的latest；inspect并未因此获得任意发现未知最新代次的能力。新比较只允许这两个expected不同，绑定version／key／generation／target／project／phase／history、运行key及selection仍逐项核对，并绑定Reader实例、ObservationId、root及完整Ticket。

观察latest等于原expected时可报告baseline_expected_equal；找到原Ticket.commit时，可报告该具体提交的字段和导航对照；ID缺位只返回None，不叫NotCommitted。较晚文件可保留具体原提交证据，但不把当前完整内容／metadata冒充那个历史截面。

legacy请求按原格式完整映射commit、expected.commit、origin、project／history／lineage／epoch／root、捕获head、规范历史prefix、node_count、next_alias／next_branch和排序Redo。prefix按manifest追加顺序、原记录checksum、地址hash及按首入节点分组且kind／id排序的admissions生成。计数器零仍表示耗尽。旧legacy hash没有expected.generation，必须单列原generation，并校验查到事实的previous及checked_add(1)后的generation，不能靠hash相等补造这项证据。

## 3. 独立意图投影

使用独立前缀skin-editor/SaveIntent-Proof/v1后接NUL，域u16大端，后续复用既有TLV(tag:u16be,len:u64be,payload)字节封套；不修改FileCoord-Proof-v1五域。

| 域 | 精确字段顺序 |
| --- | --- |
| 1 navigation | head:u128,next_alias:u64,next_branch:u64,redo.count:u64，按parent持久ID排序的parent:u128／child:u128；重复parent拒绝 |
| 2 archive | optional-receipt(latest),optional-receipt(manual),optional-receipt(protected),marks.count及原顺序commit／node，tags.count及node／name排序的node／name原UTF-8／value原UTF-8，extensions.count及name排序的name／原bytes |

optional-receipt与receipt字段编码沿用已定的present:u64及commit:u128,generation:u64,origin:u64,head:u128；origin为Manual1／Automatic2／Exit3。整数固定大端，文本不规范化Unicode或换行。排序只操作固定索引工作表，不改原Vec或原字节。Manual marks保留已验证的原顺序，不能按随机CommitId排序。

导航明确分q、此次打开的派生Redo及latest checkpoint原Redo，另为每提交保留固定导航摘要。完整checkpoint块hash不是导航子集hash。Archive摘要不含features，必须叫ArchiveMetadata-only，不能给旧第三域传空features冒充完整metadata。

would_commit仅是假设发生新提交的纯投影：原expected.generation checked_add1，last/protected为该票，Manual时覆盖manual并虚拟追加mark，Exit／Automatic保持manual；其他既有marks／tags／extensions借用流式编码，不clone整份metadata或构造JSON。代次耗尽返回无预测，不回绕；NoChange／重放不能自动采用预测。比较捕获前metadata只在观察为原baseline时有意义；预测提交后metadata只在原commit就是latest时比较，较晚文件返回None。

## 4. 结果与缺证

FileSaveComparison分别返回两域相等、baseline expected关系、打开／latest导航相等、捕获前Archive相等，以及可选原提交的legacy字段／独立generation／导航／预测Archive比较。结论只有InsufficientForSaveReconciliation。找到ID、全部布尔相等或CI通过均不授予保存资格、不解除Unknown；没有原ID也不证明未提交。

仍欠features／完整metadata、历史q完整截面、NoChange三种覆盖证明、第三类Unknown的阶段／目标连续性、产品物理绑定与完整工作预算。旧已确认事实不回退，不自动重写或换ID，不改写原Ticket以调用旧完整request比较。

## 5. 控制预算与后续证明

Reader512KiB与A控制512KiB是独立分区。所有观察、旧捕获候选、新Work候选的固定载荷及当前工作槽共享同Reader池，按size_of和生产／返回瞬时重叠预留，真实Drop退额。Work的TaskControl、共享Snapshot／资源及旧codec分配仍归已有所有者，不能把共享底层重复算成已证明的新池。

新的hash借用原数据，固定64节点根表及最多4096项索引工作表（Redo最多64、tags4096、extensions1024），不创建新可变长metadata／Redo副本。文件侧每提交只增加固定32字节导航摘要，不保留另一套Checkpoint Vec；Data更新按真实静态布局计费。文件context helper退出后才构造Data，双份Data预留以静态断言覆盖索引／hash、返回值、FileFactsSummary和FileReadPreflight。旧捕获入口新增prefix工作另预留2个Sha256与6个digest的保守固定上界。临时hash／索引与结果生产不能落到账外，必须用静态上界及N／N−1测试核对。

64MiB完整Rust／codec、PNG、SQLite全部连接和临时盘继续BudgetUnproven。PNG后续须研究锁定image／png的文本／ICC／解压分配与with_limits覆盖，不能单凭64KiB原件或一个max_alloc签收。SQLite仍须私有分配／VFS临时文件方案；不改全进程heap、不扩大上限、不减旧验证。本片不做正式成本测量或性能／stable升级。

## 6. 上轮CI门控失败

保留attempt1原工具解码日志、246过／1失败及同SHA重试全通过的证据。现有start把30秒接收超时、EOF／读错、提前退出统一成未到READY，故不能确认根因。新代码与旧first-before链没有已发现的直接调用或全局状态接缝；不扩大追查为全系统日志／用户数据调查。

仅增强该测试辅助器的有界失败诊断：保留接收结果、停止前try_wait状态、有限末行／结束类别。30秒时限、同nonce精确READY比较、成功后的排水、实际停止／join及原恢复断言保持。新增保留后缀最多256字节；旧BufRead.lines的单行临时String未改，不在本片容量证明内。新诊断只解释失败，不重试、忽略或放行失败，不声称原故障已修复；不得为诊断引入无界保留。

## 7. 验收

实现前独立审阅；至少覆盖真实restore→A1 Work→候选、完整Ticket与原expected、CompleteNotSaved及Ack／真实释放、外部Snapshot／Node、失败／取消与同池N／N−1；baseline／真实后续提交／较晚文件、generation单列、分叉prefix／独立legacy向量、打开派生Redo与latest原Redo、前metadata／Manual预测／Exit保持manual、原数据不变及代次溢出。门控诊断验证EOF／错误／超时分类和保留上限，不用睡满30秒替代逻辑断言。全部旧42专项和正常核心／桌面检查保持。

最终按实际源码、独立审查、原始失败／成功、正常完整hook、精确提交推送及新完整SHA CI记录收口。仅自制测试根与fixture；schema.rs标记、旧证明、PRIVATE材料保持。新增忽略日志仅在本机，不擅自上传。
