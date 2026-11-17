# P3-2B1-2F：历史q多域关联审阅

2026-11-17。回退点5a340284f0275e07564fc0daa7a3e2a89e611c7e与PUBLIC origin一致，run44085058173三项成功。无实际／暂存变更，复用HEAD；890受控文件原字节／mtime基线仅本机，schema.rs既存空差异标记保留。

## 1. 冻结依据与选择

[契约](P3_2B1_2F_CONTRACT.md)及[固定语义矩阵](P3_2B1_2F_MATRIX.json)先经两路独立只读审阅，再于2026-11-17T13:54:08Z冻结并开始源码变更。契约SHA256为503e48f6f86115dec53e64a04729667a55b11b707479f468cc5f4ce4ddc02648；矩阵为c4ff956ff8b4322e7a1cc761fac860d5d19339fbd83db616a2067e0df17b98a0。

本片优先关联已存在的C／E域，解决调用者自行混拼不同观察／提交证据的风险；不猜测features，也不增加历史metadata解析。审阅修正了“legacy不含commit身份”的措辞：其legacy_request前像含原Ticket commit，独立relation仍不可省略。补齐previous／origin固定预期及generation缺失／溢出的纯算术边界。

## 2. 实际能力与来源

[application纯值](../crates/application/src/file_historical_save.rs)新增独立版本1 Context／比较结果。Context完整保观察、FileFactsRequest、selected CommitId和原SaveTicket；返回实际q完整意图、observed_expected及原expected是否相同、ExperimentalValidation。没有保存端口、事实安装或PreparedWitness转换。

[infra关联](../desktop/infrastructure/src/project_storage/file_facts/historical_save.rs)必须消费实际不透明FileSaveCandidate，先经原compare_save完整票据／Reader／root／绑定守卫，再从同观察读取C／E固定事实。两份FileCommitFacts及proof_version须一致，不一致fail closed。Missing返回selected=None，也不能绕过来源守卫；不解释成NotCommitted。

Some单独区分TicketCommit／OtherCommit。各项内容／原件／head／导航、previous／origin／generation／legacy请求字段及is_latest均取选中提交自己的事实；不误取compare_save().commit的原Ticket提交。旧match_commit与本片共用抽出的原legacy_fields_equal，原规则不变。generation保expected_generation与可选相等，None和false不同。

metadata始终NotCompared，不带当前ArchiveMetadata-only预测去证明历史；结论恒Insufficient。提交ID相同、各域相等甚至请求字段相等都不授予NoChange、覆盖或Unknown解除。原expected与后续观察latest分别保留，不改原票。

## 3. 固定成本与回归边界

没有新增Data／Cache槽、受管分配、编码、图／原字节遍历、checkpoint解析或文件I/O。Context／结果各静态不超过4096字节；具名固定局部副本总式不超过16KiB，明确不是语言ABI全部栈峰值或无限宿主聚合证明。至多128提交定位与8原件复制，既有Reader512KiB不变。

实际prepare＋inspect聚合P／P−1及关联前后管理计数／峰值验证保持旧准入；旧独立prepare、完整prepare_save、inspect N／N−1及所有旧测试继续执行。取消第一代真实原件摘要后，以及后续分支未来raw损坏时，都不发布新观察，旧关联／旧页及真实q保持；Reader、观察和q逐个实际Drop，最后所有者前不退额度，Ack先等待资源再Released。

共同64MiB、Cache／RestoreNode／codec／JSON／Vec／Map、PNG no_limits、SQLite／VFS临时盘仍欠证；A控制512KiB独立，单Reader不证明宿主输出副本或无限Reader总成本。

## 4. 验证记录

十行固定矩阵覆盖Missing、旧400／401、真实q提交、后续分支／unused原件、原SHA位号前移、同ID错head／导航／请求及同量异raw；四行纯算术边界不冒称真实q准入。既有B／C／E与完整证明黄金字节不变。具体测试结果、审阅及日志散列见[结构化证据](P3_2B1_2F_PROOF.json)；正常完整hook和新SHA三项CI分别核对，不套用父结果。

## 5. 剩余门槛与保管

只完成已有历史域的同观察关联，完整历史q仍欠features真实来源／完整metadata、NoChange的CurrentManual／CurrentProtection／VerifiedOpening覆盖、Unknown阶段／目标连续性、注册物理绑定及共同预算。完整B1／PB16未签收，不跨B2／生产保存／恢复部署、迁移或清理。

旧first-before失败和两次CI取消根因仍Unknown，近期成功不代表修复；旧证据、冻结程序、PRIVATE及schema.rs原字节／mtime保留。源码／文档精确路径按正常完整hook、PUBLIC推送及新SHA三项CI收尾；新增原始忽略日志仅本机，不自动扩充旧私有恢复快照。仅自制fixture与显式测试根；用户返回未知，UAC／截图／现场测试等待，UI／Skill暂停。K1／Reference10000／Tiny4、产品NotEvaluated／stable Unknown保持。
