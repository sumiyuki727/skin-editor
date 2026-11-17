# P3-2B1-2F：同一观察的历史q多域关联契约

2026-11-17。轮前PUBLIC origin与HEAD均为5a340284f0275e07564fc0daa7a3e2a89e611c7e，run44085058173三项成功。实际／暂存差异为空，复用回退点，不创建空提交；890文件原字节／mtime基线仅本机。schema.rs原标记、旧证据及PRIVATE材料保留。

## 1. 最小范围与前置

在C历史内容、D逐代原件严格准入、E历史原件投影之后，为实际不透明FileSaveCandidate提供一个同观察、同完整请求、同选中提交的固定只读关联。此前调用者须自行拼接两种查询和最新文件比较；本片由受信适配器保证选中提交一致，并独立保留提交身份与各域结果，不提供all_equal。

选择本片优于立即扩展metadata：现有C／E固定数据和真实q已齐备，无需猜测features来源、增加历史metadata解码或扩张Cache。只补已有域的关联，不宣称完整历史q已覆盖。features／完整metadata没有真实q完整来源，明确NotCompared；不能用writer默认features填补。

保持ExperimentalValidation／InsufficientForSaveReconciliation／BudgetUnproven。不是NoChange、CurrentManual／CurrentProtection／VerifiedOpening覆盖、Unknown解除、注册物理绑定或生产authority；不进入B2、产品保存或恢复部署／迁移／清理。

## 2. 接口与来源

application新增独立FileHistoricalSave1纯值；版本1不改API7／捕获3／协调1、既有端口版本、文件／记录及证明字节。Context保新版本、完整ObservationId、完整FileFactsRequest、selected CommitId和原SaveTicket。结果保Context、实际q的完整FileSaveIntentSummary、assurance、本次observed_expected及baseline_expected_equal。

比较只由infra方法接收真实FileSaveCandidate，不接受调用方自造摘要替代所有者。复用compare_save完整原票／Reader实例／root／请求绑定守卫；只有q原expected与观察latest可以不同。完整观察请求必须等于该观察，不改原票迁就latest。缺失提交也先完成所有来源守卫。

selected为Option。None表示本次完整验证中没有该ID，不是NotCommitted；Some同时保C内容facts和E原件facts，二者完整FileCommitFacts及proof_version须一致，否则ValidationFailed。外层一份完整来源禁止混合观察／请求／选中提交，结果无I/O、无保存事实安装、无可变引用／资源所有权，也无PreparedWitness转换。

## 3. 关系与分域结论

Some的relation仅按selected commit与原ticket.commit分TicketCommit／OtherCommit，不能因摘要相同改判。分别给出content_equal、originals_equal（digest／count／bytes）、head_equal及SaveIntent导航子集navigation_equal；导航子集不等于checkpoint.navigation块hash。

请求关联另给previous_equal、origin_equal、expected_generation:Option<u64>、generation_equal:Option<bool>和legacy_fields_equal。previous比较原expected.commit，expected_generation取原expected.generation.checked_add(1)，不可算／溢出时expected_generation与generation_equal均None，区别于已算出的不等；legacy_fields_equal精确复用既有match_commit的previous／origin／head／legacy_request／prefix／节点数／地址计数规则。它不代替独立commit身份判断，不单独比较FileCoord内容／原件域，也不证明完整绑定；其中legacy_request前像仍含原ticket.commit，不能重写成selected ID。is_latest只比较该观察latest身份。新结果不含would_commit_archive_equal，避免将最新ArchiveMetadata-only投影误称历史完整metadata。

metadata明确FileHistoricalSaveMetadata::NotCompared，结论恒Insufficient。即使TicketCommit、内容／原件／导航／请求字段均相等，仍保留此边界；OtherCommit即使所有已比域相等也不转TicketCommit。SameCommit但head／请求或原件不符须如实返回各域false，不能抹为缺失或取得成功回执。

## 4. 容量、工作与保管

不增加Data／Cache槽、临时验证工作、JSON／Vec／Map、图／原件遍历、checkpoint解析、原字节读取或SHA编码。调用固定查询及一次已有真实q守卫；每个查询最多扫描128提交，原件最多8项。结果与局部均为固定纯值，Context／比较各不超过4096字节，静态断言列出固定工作构成；不宣称语言ABI的全部栈帧峰值或任意宿主复制已受池计量。

Reader512KiB管理计数／峰值及所有者布局不变。测试在已完成prepare＋inspect后的实际峰值P处重开限制为P的Reader，关联查询成功且不增加管理计数；P−1保持原准入失败，不在查询中另借额度或提前退旧q／Work／观察。Reader→观察→q实际Drop／Ack仍按既有规则，取消／后续损坏失败不发布新观察，旧关联／页继续有效。

本片没有新的受管分配，不需要修改共同预算作为前置；共同64MiB仍欠Cache／RestoreNode／codec／JSON／Vec／Map、PNG no_limits、SQLite／VFS临时盘及无限Reader／宿主／输出副本聚合证明。A控制512KiB独立。固定值大小不是生产总资源合格证。

## 5. 冻结验收

先由两路独立只读审阅确认本契约及固定期望矩阵，再冻结散列、开始源码变更。矩阵从自制fixture的已知提交／原件／head／导航和真实q建立，不从新比较实现生成；本片不新增编码，继续执行C／E和旧完整证明黄金向量。

| 组别 | 必须覆盖 |
| --- | --- |
| 完整来源 | 新接口／file版本、观察两部分、完整请求所有字段、原票字段错配拒绝；Missing也拒绝错票及另一Reader，旧观察仅在自己作用域有效 |
| 历史与实际q | 实际Work→q→真实提交→后续分支／未使用原件；原q选中提交多域true且非latest，保原expected和本次latest；最新多域变化如实报告 |
| 关系独立 | 旧401与q具有相同内容／原件／head／导航但OtherCommit且请求不同；400原件相同而内容不同；数值ID相同但真实qhead不同仍TicketCommit且head／导航／请求false |
| 原字节反例 | 合法实际Work与合法文件的原件数量／字节相同但raw不同，originals_equal为false，不因同计数／同ID认定相同 |
| 缺失／未比 | Missing为None不等于未提交；metadata始终NotCompared，全部结果始终Insufficient且application保存状态仍CompleteNotSaved |
| 保管与失败 | 完成一代实际原件摘要后取消，以及未来已损坏文件的完整验证失败，旧关联／历史页不变；源字节／q原票、实际Drop／Ack保持 |
| 容量 | 固定大小断言、P／P−1、查询无管理增长及旧独立prepare／inspect N／N−1全部保留；不提高预算／时限 |

主线唯一写入，重型构建串行，审查者不编辑／构建。正常完整hook、精确路径PUBLIC推送和新完整SHA三项CI。旧first-before失败及两次CI取消根因仍Unknown，25分钟CI／30秒门控保持。仅自制fixture与显式测试根，不触真实用户内容。用户返回未知，UAC／截图／现场测试等待，UI／皮肤Skill暂停；新增忽略原始日志仅本机，旧PRIVATE快照不自动扩张。K1／Reference10000／Tiny4、产品NotEvaluated／stable Unknown保持。
