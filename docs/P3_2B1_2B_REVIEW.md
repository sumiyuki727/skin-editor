# P3-2B1-2B 实现与独立审阅

2026-11-17。本片按[B1-2B契约](P3_2B1_2B_CONTRACT.md)将真实SavePreparationWork与只读文件事实关联，补齐原SaveTicket／expected、legacy请求、导航及ArchiveMetadata-only证据；仍为ExperimentalValidation／InsufficientForSaveReconciliation／BudgetUnproven。完整B1／PB16未签收，不进入B2写入、产品保存或恢复部署。

## 1. 真实Work与原expected

[入口](../desktop/infrastructure/src/project_storage/file_facts/save.rs)消费并保管真实Work，仅通过公开snapshot／statistics／poll读取。私有泛型流复用既有捕获投影，不提取private capture、不取得SaveExecution或authority。API7由真实core请求准入保证；捕获3／协调1仍逐项核对。新FileSaveIntent1纯值不向PreparedWitness或写许可转换，文件格式与既有端口版本不变。

候选保留完整Ticket及原target.expected。观察请求准确声明该次文件latest，比较只允许expected不同，其他绑定、会话／请求、selection、Reader实例、ObservationId及root仍须一致。完整Ticket比较包含source／capture／coverage，不因hash相同跳过来源。该入口尚不能自行发现任意未知最新代次，不能把观察latest写回原Ticket。

原ID缺位返回None，不是NotCommitted。找到该提交时分别给出legacy字段、previous及独立generation、该提交导航、是否latest，以及可适用的预测Archive相等；较晚文件不把当前完整内容或metadata冒充历史q。所有布尔相等也只得到Insufficient，不安装保存事实、不解除Unknown。

## 2. 请求与独立投影

规范prefix在原根表仍存活时按manifest顺序增量生成，覆盖原记录checksum、地址及首入对象。legacy字段复现既有archive-request格式；该旧格式缺expected.generation，故必须另外校验checked_add后的generation。既有FileCoord五域未变；[新独立向量](P3_2B1_2B_VECTORS.json)按Python结构化字节流与hashlib生成，并由另一只读审查重算。

SaveIntent独立命名空间分navigation与ArchiveMetadata-only。导航区分q、打开时派生Redo及latest原始Redo，另存每提交固定导航摘要；完整checkpoint hash与导航摘要不混用。Archive流式编码完整已验证receipts／原序marks／按键排序tags和extensions，保留原Unicode、NUL、换行和扩展字节；明确不含features。

would_commit只是假设发生新提交的纯投影，不承诺一定提交。真实线性Manual旧点再次保存返回NoChange，测试确认文件字节不变；不能套用新提交预测。Manual／Exit／Automatic各自元数据规则与代次溢出均核对；预测只在目标commit就是latest时比较，较晚文件返回None。

## 3. 所有权与固定预算

成功候选先退出内层捕获Snapshot及其自身租约，再退出Work，覆盖Work包装的额外租约最后退额。独立审查发现原错误路径的局部租约可能早于函数参数Work退出，已改为预算准入后立即建立WorkOwner，统一成功和所有提前返回路径；字段顺序是Rust规定的析构次序，不增加全局状态或测试专用生产布局。

顺序用例解构实际WorkOwner，先释放真实Work并观察work_released，而额外额度仍保留，再释放租约。该用例验证真实资源职责；自动错误路径的中间时刻顺序依据是字段声明顺序，并非用例直接截获。实际临时额度不足、部分消费、取消及正常候选Drop另有回归。Ack不回收仍存活Work／外部Snapshot／Node；Reader先Drop也不提前退还候选额度。

Reader512KiB与A控制512KiB分开核算。新投影复用固定4096项索引工作表、原64节点根表及128提交摘要，不克隆metadata／Redo Vec或构造JSON。文件侧Data双份预留由静态断言覆盖工作相位的索引／hash、返回摘要／导航数组、FileFactsSummary和FileReadPreflight；helper退出后才构造Data。旧prepare_capture也为新增prefix计算额外预留2个Sha256和6个digest的保守固定上界。N／N−1测试核对聚合准入，但不以同源公式测试替代列式静态核算。

原Rust／codec／PNG、SQLite全部连接及临时盘仍无完整64MiB聚合证明。锁定image0.25.10的[PngDecoder源码](https://docs.rs/image/0.25.10/src/image/codecs/png.rs.html)仍需后续逐项研究构造、文本／ICC及解压分配与limits适用阶段，本片不改解码器或全进程heap，也不据小fixture推导产品容量／性能达标。

## 4. 回归与发现

Windows存储定向84项通过：原70项保留，新增12项真实Work／意图专项和2项门控诊断；其中旧FileFacts42项保持。application另增1项固定纯值／版本测试。准确名称、源码与向量散列及原始日志见[结构化证据](P3_2B1_2B_PROOF.json)；最终正常完整hook和新提交同SHA三项CI另行核验，不能套用父提交结果。

| 组别 | 实际覆盖 |
| --- | --- |
| 实际来源 | restore→真实A1 Work、完整Ticket、原expected、API／捕获／协调版本、不修改活动项目或保存事实 |
| 完整请求 | 独立prefix／legacy向量、真实后续提交与较晚文件、独立generation反例、Some恢复记录＋None新编辑与独立writer交叉验证 |
| 导航／metadata | 线性空Redo与NoChange、分叉派生Redo、latest原Redo、Manual预测、Exit／Automatic保持manual、原序marks、乱序标签／扩展的原字节、溢出与固定索引长度门控 |
| 取消／释放 | core预取消、部分消费拒绝、外部预取消、Complete后States／发布前外层取消保持CompleteNotSaved、Ack与外部Snapshot／Node、Reader先Drop、Work／包装租约顺序、临时额度及聚合N／N−1 |
| 隔离 | 完整Ticket字段变化、观察／版本、错误target、旧原expected与新观察latest的分离，不把原ID缺位当未提交 |

初轮72项通过；补真实Work后出现1处测试编译类型错误，修正后78过／2失败。独立分析确认两失败为夹具错误：线性重开不增加显式Redo，旧Manual点重存应为NoChange。独立向量改为空Redo，真实追加测试改为实际不同手动位置，并补新编辑混合记录，未按Rust输出反填向量。随后80、82及最终84项依次通过。全部失败／中间成功日志保留，不隐藏或覆盖。

Clippy首轮指出一个未使用测试变量，移除后通过。正常完整hook另通过516项工具回归、application88项及桌面默认／全feature基础设施各261项，得到首个回退提交0cf6ad7d681c3e73085d828f815933bbd4df4651并已推送。最终只读平台核对又发现两个仅供Windows专项调用的测试辅助函数缺同样cfg条件，已精准补齐，避免非Windows测试构建留下dead_code定义；未实际运行非Windows基础设施构建，不能扩大为该平台已验证。此三路径修订另经新正常完整hook和后续新SHA CI，不改写首个提交或沿用其CI作为最终结果。

## 5. 上轮CI异常与本轮诊断

父提交1a3578484fb2fcaab315701bb02bfc731f62ff4d的[run44006333067](https://github.com/sumiyuki727/skin-editor/actions/runs/44006333067)首个desktop尝试246过／1失败，失败是旧first-before进程门控未到READY；新24专项当时均过。同SHA仅一次桌面作业重跑后247项及全feature均过，三项CI最终成功。失败与重跑原日志保留，不称首轮成功。

首轮整套耗时258.53秒与本机不同，不足以确认负载根因。只读调用链核对未发现本片与旧first-before链的直接调用或全局状态接缝，根因仍Unknown。本轮仅改善测试辅助器：区分recv_timeout结果、EOF／读取错误及停止前进程状态，新增保留后缀最多256字节；旧BufRead.lines单行临时String未改，不在本轮有界证明内。

30秒时限、同nonce精确READY条件、成功后排水、停止／join和原恢复断言均保持。两项诊断用例验证上限及分类，不靠睡满30秒、不放宽时限、不自动重试、不忽略失败，也不声称已修复旧CI根因。最终新SHA如再失败须保留新证据再分析，不能反复重跑掩盖。

## 6. 剩余退出条件与保管

完整B1仍欠features／完整metadata、历史q完整内容截面、CurrentManual／CurrentProtection／VerifiedOpening与NoChange覆盖证明、Unknown阶段／目标连续性、注册物理目标及完整预算。后续只在已授权B1范围继续分片收口；B2写资格和生产保存另有准入。K1、Reference10000／Tiny4、产品NotEvaluated与stable Unknown保持，不重测或扩展。

轮前复用已推送且最终三项CI通过的1a35784，无实际差异而未造空提交。本轮累计25条源码／构建配置／文档精确路径仍须经正常完整hook、PUBLIC origin推送和新完整SHA CI；以包含本报告的提交为准。schema.rs原字节SHA256保持713f3850740db024c4b861327de560f4eb81f5820980d1f89672dcdcfd499125，未改路径原字节／mtime逐项核对；旧证明及PRIVATE恢复材料不改。新增原始日志仅本机忽略目录，不擅自上传或自动延伸固定恢复快照。未调用UAC或更改系统安全设置。

## 7. CI容量补救与未完成记录

[0cf6ad7运行44020691259](https://github.com/sumiyuki727/skin-editor/actions/runs/44020691259)和[4844480运行44024475913](https://github.com/sumiyuki727/skin-editor/actions/runs/44024475913)均为两平台核心成功、桌面cancelled。两次桌面作业在既定25分钟边界终止，全feature基础设施尚无完整汇总，后续构建也未完成；日志已输出部分没有断言失败，不能记为三项通过。两次均未重跑，原工具解码结果及可读派生日志各自保留。旧first-before失败与本次取消的根因都未因此确认或解决。

首次日志中编译／检查累计约9分钟，默认完整测试及文档测试约8分19秒，全feature测试至取消约6分35秒仍未完成；这些是CI日志分段，不是产品成本计量，也不能归因到某个磁盘或单一构建项。两次旧日志都没有记录有效TEMP／TMP，不能倒推出临时盘是原因。

经独立只读审阅，采用两项明确配置：[桌面manifest](../desktop/Cargo.toml)仅将dev/test第三方依赖debug设为0；六个桌面成员保持默认完整调试信息，两个非成员本地包domain／application显式debug=2。Cargo的星号会覆盖所有非当前workspace成员，因此这两个例外必要；代价是第三方代码的变量／类型调试信息减少。[Cargo规则](https://doc.rust-lang.org/cargo/reference/profiles.html#overrides)

本机编译器artifact核对dev/test各183个含版本package_id（178种包名）的实际debug级别、opt_level=0、debug_assertions及overflow_checks=true。已锁libsqlite3-sys0.39.1／cc1.8.0的Windows/MSVC链没有Cargo DEBUG到NDEBUG、SQLITE_DEBUG或优化级别的映射，MSVC调试分支仅增减符号参数；wgpu／naga／cpal相关入口未读取DEBUG。cc的GCC／Clang分支可能随DEBUG改变栈帧保留，因此不宣称各平台机器码完全相同，不把本次核对扩大为非Windows基础设施构建已通过。

[桌面CI检查step](../.github/workflows/core.yml)显式设TEMP／TMP为runner.temp，统一该步骤的临时目录；影响继承环境的Cargo、构建脚本、测试及其子进程，也可能影响SQLite Windows临时文件位置，不能叫仅改变测试根。本机hook和系统环境不改，不新增对runner.temp的整体清理；既有测试及恢复子进程仍以各自std::env::temp_dir核对canonical parent。[GitHub上下文](https://docs.github.com/en/actions/reference/workflows-and-actions/contexts#runner-context)、[SQLite临时文件规则](https://sqlite.org/tempfiles.html#temporary_file_storage_locations)

优化级别、断言、溢出检查、panic策略、并发2、默认／全feature完整检查、测试与资源预算、25分钟作业及30秒门控均保持。根核心及独立成本工具workspace、全部release配置和冻结证据不改。编译器artifact投影核对实际debug级别与断言／溢出设置，再运行正常完整hook；新配置是否足以让CI完整结束须由后续新SHA运行确认，不承诺时限内完成，不追认前两次取消，不升级B1、产品性能或stable结论。
