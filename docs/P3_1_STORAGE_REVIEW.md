# P3-1C 审阅：Rust 增量文件与完整恢复候选

2026-11-02。已加入 infrastructure 的 Rust SQLite 适配及 application 管理的脱离活动会话恢复。**普通桌面仍是内存编辑器，没有正式保存／打开 UI、自动保护、退出补存或皮肤导出。** 本增量供审阅，不把独立文件测试当作这些产品流程已完成。

2026-11-03补充：[热日志与进程中断恢复](P3_1_RECOVERY_REVIEW.md)新增Windows原件冻结／副本回滚／完整验证及真实进程终止测试。下文“尚未验证进程强杀”是本篇初轮范围；物理断电、其他平台和正式恢复UI仍未完成，旧证明保持原样。

本轮开始的 P3-1B 回退点为 `8cb882e1389f955f95b6059431fd0c7025adad9e`，现已上传私有 GitHub。Git 管理和空间清理见 [GIT_WORKFLOW.md](GIT_WORKFLOW.md)。原 P2／P3 证明、黄金向量及冻结程序没有改写；新检查日志和源码摘要另见 [P3_1_STORAGE_PROOF.json](P3_1_STORAGE_PROOF.json)。

## 对象、方法、依赖和所有权

Rust 无类继承。下表是组合关系及 trait 实现；文件／SQL／随机数仍不进入核心包。

| 对象／路径 | 方法或字段 | 调用与持有关系 |
| --- | --- | --- |
| application::ProjectArchive · [archive.rs](../crates/application/src/archive.rs) | write_capture、restore_candidate；关联 Error | application 定义存储端口，infrastructure 实现，不依赖运行时适配器 |
| ArchiveWriteRequest／Outcome／Receipt | 显式端口版本、CommitId、期望前次提交、Manual／Automatic／Exit、取消；Committed／Replayed／NoChange | NoChange 不产生文件提交或保存事实；真实回执将来由保存用例关联捕获截面 |
| RestoreHeader／RestoreNode | 永久身份、完整拓扑、地址计数、对象首次准入、导航、原始记录及完整项目值 | 独立 DTO；不序列化运行 Request、Draft、Patch 或 UI |
| RestoreBuilder · [restore.rs](../crates/application/src/restore.rs) | new、push、finish | 验证数量／身份／父先于子／深度／导航／资源闭包；有界、迭代校验，不修改活动项目 |
| RestoreCandidate | header、nodes、into_session | 持有完整不可变状态和原字节租约；通过 application 工厂建立新会话，不提供现有会话替换 setter |
| OriginalNodeRecord／ArchiveMetadata | 首写版本、state 摘要、原 action 字节、节点 checksum；真实提交、标记、用户标签和扩展 | 节点记录随历史保留；标签及导航与节点内容分离 |
| infrastructure::SqliteProjectFile · [project_storage](../desktop/infrastructure/src/project_storage/mod.rs) | publish_new、load、write_capture、restore_candidate、receipt、committed、sqlite_version | 实现 ProjectArchive，拥有文件句柄／SQL 连接、限定元数据缓存与 BlobStore；不拥有 ProjectSession |
| FileLimits／StorageError | 文件／元数据／原字节／状态展开限额；Conflict、Busy、ExternalChange、OutcomeUnknown 等 | 结构化错误，不向 UI 输出 SQL 或私人路径；不自动重试不确定提交 |
| 私有 schema／blocks／read／write／values | DDL、分块／分页、完整读事务、增量事务、稳定二进制摘要 framing | 只有本模块可见；没有导出通用 SQL 执行器或故障注入 UI |

数据流为 `ProjectSession.capture_history → HistoryCapture → SqliteProjectFile`。加载为 `只读 SQL 事务 → 全文件验证 → RestoreBuilder → RestoreCandidate → 新 ProjectSession`。新会话由宿主提供全新运行 epoch，旧请求／回执／草稿／音频／GPU 结果不恢复。激活重建私有前后补丁，不执行旧 Command；此工厂不供 UI／AI 随意替换活动项目。

候选及会话分别固定已经验证的原始字节；派生 RGBA、PCM、Voice 不随历史持久化。无论选手动还是最新位置，**完整树、全部分支、未使用及不透明素材**均须通过验证并保留。恢复／失败／取消释放的是实际租约，不是只清空队列。

## 当前文件约定

| 层 | 版本／标识 |
| --- | --- |
| 进程内模型／命令 | 模型2、application API6，未改变 |
| 捕获／存储端口 | HISTORY_CAPTURE_VERSION=2；ARCHIVE_PORT_VERSION=1 |
| 文件识别 | `.oskproj`，SQLite application_id=0x534b4550，user_version=2 |
| 文件头 | family=skin-editor-project；**profile=skin-editor-rust-v1** |
| 容器／项目／历史／节点 | container2、project_schema1、history_schema2、record_version2 |
| 状态块 | [PROJECT_STATE_SCHEMA.md](PROJECT_STATE_SCHEMA.md) schema1 |
| 必要能力 | storage.core:2、state.editor:1、history.summary:1、history.delta-index:1 |

P3-0 的 p3-0-r2／fixture 状态不是本 profile 的输入。当前尚无发布格式；本 profile 的精确 DDL 与能力共同选择读取规则，不能仅凭容器版本相同就读取别的布局。后续若改变本 profile 的 DDL（含触发器），须保留其旧读取规则或另给有测迁移，不能改常量后让既有文件无路可读。

捕获端口从1升2，是因为步骤描述改为自有字符串，并新增首写记录和文件元数据；未知步骤名称是不可执行描述，不能为载入它泄露静态字符串或回放旧代码。首写 action JSON 的额外描述字段保留原字节。永久节点摘要仍采用已定稿 node.v2 framing，独立地址摘要采用 address.v1；显示地址不进入节点内容摘要。原始引用和旧摘要不因当前编码器排版或迁移计划而改变。

## 增量与完整性

原始媒体按 SHA-256／长度去重并以最多64 KiB块保存；同一 Asset 或多历史状态引用不重复存字节。状态为不可变内容块，块引用先于引用者入库，新增内容复用旧块。节点、操作、父关系、状态、准入和原字节只追加；数据库触发器拒绝对不可变表 UPDATE／DELETE／REPLACE，读取仍重新核对，不能把触发器当作不可篡改的密码学证明。

每次文件提交只追加新增节点、对象首次准入和新增原字节的分页索引；不重写整个历史树索引。分页每页最多256条，以 history-index／history-page 块表达；滚动历史前缀绑定节点摘要、独立地址摘要及首次准入。Redo 是该检查点的完整导航快照；未变标签、扩展及能力索引根复用。常规事务不重写所有媒体，但捕获和一致性比对仍有全树 CPU 成本，不宣称每次保存为恒定时间。

一次完整读事务验证 SQLite 结构／外键、精确 schema、全部块和引用索引、全部节点与地址、所有提交／前缀与元数据索引、全历史状态、原字节哈希／长度／块连续性、领域不变量、首次对象准入和资源闭包。非当前分支损坏也拒绝恢复；不只检查当前头。媒体分块流式校验，原始存储在分配前按共享预算准入；PCM 不在加载中解码。PNG 尺寸及类型化 WAV 元数据还须与声明一致，不透明资源按字节保留。

默认限额：文件1 GiB、缓存元数据32 MiB、原字节128 MiB／4096项、块131072项、历史65536节点／262144对象ID、累计状态闭包64 MiB。RestoreBuilder 另计模型逻辑展开及恢复所需结构；这些是拒绝超限的逻辑准入，不是整进程实测内存承诺。SQLite 页4096、缓存约2 MiB、mmap关闭、临时数据可落盘；原字节流式读取也仍需要受限 scratch。

## 手动位置、恢复位置与导航

Manual 文件提交推进手动与保护位置，Automatic／Exit 只推进保护位置。普通恢复选最近手动头和当时导航，主动 Latest 选最新保护头；两者都恢复整棵树。默认打开后无新工作关闭返回 NoChange，保留较新的保护状态，不能把回到旧手动位置误当作新保护提交。

旧手动检查点当时可能只有一个后继，运行时因此没有保存显式分叉偏好；完整树后来新增分支时，加载器只对这种“当时唯一后继、现在分叉”的父节点推导当时 Redo 路径。已有明确偏好优先，当时就歧义的分叉保持歧义；不擅自选最新分支，也不为所有线性节点持久化路径数组。

对象首次准入包括事务中创建又删除的 ID；重开不得重用这些 ID。运行请求容量、当前 UI 选择和历史显示折叠不写入文件。标签与旧保存标记可以恢复，但标签编辑及 GUI 保存状态确认仍未接线。

当前 API6 恢复工厂用文件局部 alias 建立运行 NodeId，要求根 alias=1、按追加顺序递增；永久节点身份仍另存，且不进入这种局部编号假设。**地址重分配与分支合并尚未实现，也没有对外入口。** 将来启用时须明确拆分显示映射与运行ID的转换、能力／版本门控及旧地址失效测试，不能仅改地址表后声称当前工厂已支持任意重排。当前短键／永久引用长度上限不因深度增加。

## 失败保护与尚未完成范围

现有文件使用 SQLite DELETE journal／synchronous=EXTRA 原子事务；取消在遍历、分块及SQLite进度回调处检查，关闭取消回调后回滚，避免取消同时阻断清理。失败使写适配器失效，必须重新打开校验，不能使用未回滚的缓存继续写。锁冲突不等待／自动重试；外部连接改动、路径文件替换及元数据变化拒绝旧写入。提交后丢失回执返回 OutcomeUnknown(CommitId)，重新读取再用 committed 查询事实，不凭错误猜测是否已提交。

首次文件在同目录唯一临时名内写入、完整验证、关闭并 sync_all，然后通过不覆盖目标的 hard_link 发布；不支持硬链接的文件系统明确失败，不偷偷降级为覆盖复制。临时别名失败清理与已发布目标分开；发布后失败同样返回不确定提交。**尚未证明目录项断电持久性、平台VFS／网络盘、进程强杀／断电恢复。** 读取遇到非空 journal／wal／shm 显式拒绝并保留现场，未实现正式热日志恢复向导。

未实现：已有历史的验证复制／另存；state_views 的实际 Schema 迁移器；用户标签修改事务；异步保存调度与 GUI 回执；退出补存／未命名恢复／偏好存储；实际皮肤文件导出。含未知必要能力／不支持版本或迁移表示时拒绝，不覆盖源文件。未知可选内容的正式跨版本演进仍须专门 fixture，不能据当前结构推断未来兼容已经完成。

收尾回归发现并修复：尚未支持的标签／扩展修改曾可能随普通内容保存被忽略。现在在事务开始前明确返回 Unsupported(metadata editing)，同时保留原文件和内存中的未保存内容；不以“不支持该编辑”为理由默默丢弃它。先运行旧实现确认测试失败，再加入拒绝逻辑并复测；这不是已开放标签编辑功能。

## 验证与下一步

专项测试包括完整分支与原字节往返、默认手动／主动最新、旧分支保留、Redo路径、旧运行请求拒绝、瞬时对象ID不可复用、事务故障／取消、首次发布不覆盖、损坏原字节／状态／操作／地址／准入／控制字段拒绝、锁冲突、外部修改、预算和实际租约释放。确定性故障点不依赖设备、窗口或 sleep。

10000层、10001节点的真实 SQLite 文件已完成深历史往返及根检出：初轮本机调试构建写入并完整验证36.500秒，完整读17.131秒；发布构建初测7.907／2.138秒。补上元数据拒绝后，当前发布构建重新测得写入并验证7.925秒、完整读至恢复候选2.257秒，文件同为45,039,616字节；旧轮次保留，不选较小值追认当前构建。用例为10000次小配置修改，未加载代表性大量媒体；读取紧接刚写入的文件，未清系统页缓存，不是冷启动。读取计时不含候选激活和GUI切换，写入计时则包含再次完整读取验证，两列不能作为两种加载算法对照。仍需代表性媒体／分支规模、激活及整进程峰值验证，不宣称正式打开速度或产品资源预算通过。

正常hook完成核心418项Python工具检查、格式／Clippy／无窗口核心测试，以及桌面默认／全部feature检查；当时infrastructure两套feature各188项单元通过。新增核心恢复集成3项、存储集成6项，私有存储／元数据测试原为12项；收尾增加第13项元数据拒绝回归后，受影响infrastructure发布构建的格式、Clippy和全部测试通过（189项单元及原有各集成测试），并重测当前深历史二进制。真实PNG／WAV、未选中分支及20次含原字节的恢复／释放均包含其中。运行绑定SQLite 3.54.1。没有运行GUI、GPU、原生播放或UAC，不重判P2预算。

完整核心／桌面检查的最终状态及日志摘要以本轮机器证据为准。尚未启动正式保存 UI；审阅应优先核对永久身份与旧记录复用、完整树验证、默认手动与保护位置、失败后重开，以及未测断电／平台恢复的边界。
