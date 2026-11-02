# P3-1C 补充审阅：热日志与进程中断恢复

2026-11-03。在上一轮 Rust 完整文件验证之上，增加**保留原件、恢复到新文件**的独立入口和受控进程中断测试。普通编辑器仍未接入正式保存／打开 UI、自动保护或退出补存；本轮不把文件恢复测试当作这些流程已完成。

本轮开始通过正常 hook 提交并同步到私有 origin 的回退点为 `bd3b6262d2a39830d3c94c4c81227cb5725a1700`。本轮恢复增量保留未提交供审阅；旧 [P3-1C 证明](P3_1_STORAGE_PROOF.json)、P2 证据、冻结程序和黄金向量不改写。新证据单列为 [P3_1_RECOVERY_PROOF.json](P3_1_RECOVERY_PROOF.json)。

## 恢复承诺与边界

恢复保留最后完整提交的文件历史：根、全部分支、永久身份、节点原始记录及摘要、原始资源、检查点和导航。不会用当前头替代整棵树，也不会删掉损坏分支以换取“恢复成功”。任何未选中分支校验失败，整个候选失败并保留原件。

文件事务提交前的进程中断，恢复到此前已完整提交的版本；提交后但回执返回前中断，找回已经提交的新版本。这两种情况都不删除已提交历史。**尚未提交到文件的内存编辑不在本轮保证内**；正式退出补存和自动保护尚未接线，不能把数据库事务回滚等同于主动丢弃项目历史。

SQLite 读取时可能自动回滚热日志，故不能先连接原件再声称“只读检查没有修改”。本实现对原件仅取得只读文件句柄、摘要和物理字节副本；SQLite 只接触独立暂存副本，回滚后再运行既有的全文件验证。依据为 [SQLite 锁与热日志恢复](https://www.sqlite.org/lockingv3.html)及[回滚日志格式](https://www.sqlite.org/fileformat2.html#the_rollback_journal)。

目前源文件冻结仅验证 **Windows 本地文件**：使用 [`OpenOptionsExt::share_mode(0)`](https://doc.rust-lang.org/std/os/windows/fs/trait.OpenOptionsExt.html)拒绝其他读写／删除共享，直到恢复结束。已有读者或写者返回 Busy，不猜测它是否已经退出。Linux／macOS 尚无等价冻结实现，明确返回 Unsupported；不能用普通复制或与 SQLite 锁不一致的 flock 冒充安全快照。

## 对象、方法和引用关系

新增实现均在 `desktop/infrastructure/src/project_storage/recovery/`。没有新增 Cargo 依赖、线程、项目写入口或 trait 继承；application、domain、UI、声音和 GPU 路线不变。

| 对象／方法 | 责任与持有关系 |
| --- | --- |
| `SqliteProjectFile::recover_copy(source, target, originals, limits, selection, cancel)` | 显式同步恢复入口；返回 `Result<RecoveredArchive, StorageError>`，不覆盖活动 `ProjectSession` |
| `RecoveredArchive` | 组合已发布新文件的 `SqliteProjectFile`、独立 `RestoreCandidate` 和 `RecoveryReport` |
| `RecoveryReport` | 诊断版本、恢复方式、源主文件／日志的长度及 SHA-256、日志记录数、最新已验证文件回执 |
| `RecoveryDigest` | 文件字节身份；不是 `AssetId`、节点身份或内容状态 hash |
| `RecoveryMode` | `VerifiedCopy` 为无需回滚的完整复制；`JournalRollback` 为副本经过回滚 |
| `source::Frozen`／`Part` | 私有只读独占句柄及源摘要；持有到验证、发布及清理结束；`open`、`copy_into`、`verify` 不连接 SQLite |
| `source::Scratch` | 同目录唯一暂存文件及自己创建的日志；`create`、`copy`、`cleanup` 明确所有权，不删除原件或其他孤儿文件 |
| `journal::validate` | 有界日志预检；SQLite 仍是实际回滚引擎，不在项目内另写一个数据库恢复器 |
| `Work::attempt` | 私有编排：复制 → 副本回滚 → `read::full_probe` → 复核原件 → 发布新文件 |
| `Probe` | 私有测试故障／取消注入；进程门闩和清理失败注入仅在 `cfg(test)` 编译 |

`RECOVERY_VERSION=1` 只版本化恢复报告，不改变文件 profile 或项目契约：仍为 `skin-editor-rust-v1`、container 2、project schema 1、history schema 2、node record 2、state schema 1；捕获端口 2、存储端口 1、application API 6、模型 2 不变。

`selection` 继续区分 Manual 与 Latest。恢复结果的 `report.restored` 表示文件最新已提交回执；实际激活头以 `candidate.header().head` 为准。选择手动位置也保留较新的保护检查点和完整树，不添加新文件提交或推进保存标签。恢复副本保留原持久身份，**不等于已经完成“另存为”或身份分叉流程**。

## 数据流与发布

1. 检查取消、新目标及其 sidecar 不存在，拒绝目标落入源文件的 `-journal`／`-wal`／`-shm` 名称空间；通过文件身份识别大小写及已有别名。
2. 独占打开普通源主文件及可选日志，拒绝源符号链接／重解析文件和 WAL／SHM。检查已知 SQLite 页大小及 rollback 模式、应用标识、长度限额，分块计算摘要。
3. 按已知日志 profile 预检：4096 字节页、sector／记录边界、页号、重复页、稀疏校验和、padding 和已知未发布尾 header。super-journal、未知／截断／损坏尾部明确拒绝，不能让副本跟随日志访问或删除外部 super-journal。
4. 在目标同目录使用唯一名和 `create_new` 建立暂存，物理复制主文件和日志；不把原件硬链接给 SQLite 回滚。64 KiB 块间检查取消，复制同时核对源摘要。
5. 仅在暂存副本上打开 SQLite，触发必要回滚，然后复用完整文件／原始资源／全历史验证，得到脱离活动会话的候选。
6. 关闭副本连接、`sync_all`，在源句柄仍持有时复核摘要和 sidecar，核对暂存文件身份。使用不覆盖目标的硬链接发布，随后打开新目标并回收暂存别名。

普通 `load` 仍对非空日志返回 `JournalPresent`，并保留现场。现有适配器的回执／写入 guard 也先检查 sidecar，再执行可能触发 SQLite 恢复的 SQL。显式恢复要求源适配器关闭；没有提供原位恢复按钮或自动删除日志的捷径。

## 失败、取消与资源

| 情况 | 结果 |
| --- | --- |
| Busy、格式／校验失败、超限、发布前取消或故障 | 不发布目标；源主文件和日志逐字节不变；正常清理自己拥有的暂存和资源租约 |
| 发布后故障／取消 | 返回 `OutcomeUnknown(commit)`；新目标可能已经完整可用，须完整重开确认，不能自动重写或删除它 |
| 暂存清理失败 | 返回 `RecoveryCleanup { token, cause }`；诊断 token 对应唯一暂存名，保留残留供检查，释放原件句柄和原始资源租约；发布后的 cause 为不确定结果 |
| 恢复进程自身被强杀 | 可能遗留暂存／日志或已发布目标；原件仍完整，后续从原件恢复到另一新路径，不自动扫描、提升或删除孤儿文件 |
| 已有目标或 sidecar | `TargetExists`；不会覆盖已有文件，也不会借目标路径改写源 sidecar |

流式复制／摘要使用固定 64 KiB 缓冲；journal 记录索引按 `min(file_bytes / 4096, metadata_bytes / 32)` 限制，日志输入长度不超过 `2 * file_bytes + 1 MiB`。原始字节仍通过共享 BlobStore 准入／租约固定，同一资源在多个候选中只分配一次。候选／新会话持有期间不提前归还，最后持有者释放后才归还预算。

原有文件限额继续生效。恢复需要源文件之外的一份完整磁盘副本及日志空间，完整验证还会产生有界模型／元数据和 SQLite 缓存；逻辑准入不是整进程峰值承诺。本轮未测代表性大媒体恢复耗时、峰值内存或磁盘不足的真实 OS 路径，不把小样例的租约归零当作产品资源预算通过。

## 确定性与真实进程测试

新增测试在 `desktop/infrastructure/src/project_storage/tests/recovery_process.rs`，仅 Windows 编译。父进程启动自己当前的测试程序，限定自制临时目录；子进程在精确阶段经管道发送带随机 nonce 的就绪回执，然后等待放行。父进程只终止自己创建的子进程并等待退出。30 秒是门闩失败上限，**不通过 sleep 猜测竞争时机**。

事务提交前门闩在测试内调用 [SQLite cache flush](https://www.sqlite.org/c3ref/db_cacheflush.html)，制造真实 dirty-page／热日志现场；这不改变生产供给、同步或存储策略。提交后门闩用于覆盖提交成功而调用方未收到回执的情况。

覆盖项：

- 真实提交前／提交后中断：此前五节点全部保留；提交后新增第六节点和新原字节可恢复，旧节点身份与摘要不变。
- 首次发布前／后中断：目标未发布时保留完整暂存；发布后目标可完整验证，不覆盖现存路径。
- 恢复复制后、完整验证后、发布后被终止：源主文件／热日志不变，重试成功，孤儿文件不被自动修改。
- 非当前分支独有的原始资源仍存在；Redo／Checkout 保留；Manual 与 Latest 的头不同但均保留完整树。
- 活跃读写者 Busy，拒绝后可继续完成原事务；已打开适配器不对已知热日志偷偷执行 SQL 回滚。
- 八个恢复阶段各注入失败和取消，检查目标／暂存、原件字节、实际原字节配额以及独占文件句柄已释放。
- 损坏日志、super-journal、未选中分支损坏、已有目标／源 sidecar 目标、原字节准入不足、文件超限和预取消均拒绝。
- 发布后取消／清理失败明确报告；目标仍能完整打开，资源不泄漏。
- 二十轮恢复与释放共享相同原始分配，保留一个锚点时不提前释放，最后锚点释放后归零。

本轮曾由回归测试发现：把源文件 `-journal` 作为新目标时，普通“目标不存在”检查不足以保护源 sidecar。失败日志保留；已增加源名称空间／文件身份检查，并覆盖大小写及 Windows 尾点形式。只影响自制临时样例，没有操作用户工程。

## 检查结果与后续范围

本轮16项专项通过（含1个子进程入口）；`tools/check_core.py`通过418项Python工具测试、核心格式／Clippy／测试和边界检查；`tools/check_desktop.py`默认及全部feature均通过格式／Clippy／测试与最终普通构建，两套各205项infrastructure单元及6项存储集成通过，包含10000层完整历史往返。仓库候选747份检查通过，10份变更文档的304个内部相对链接可达。源码、日志及保留性摘要见配对证明。

默认检查未打开窗口、GPU、声卡或UAC；没有重新解释P2的CPU／延迟／内存验收。本机原始日志存于忽略的artifacts，证明保留其路径与摘要；上一轮的正常hook／远端检查不用于替代本轮未提交代码的本机结果。

本轮证据是本机进程中断，不是物理断电／扇区撕裂证明。目录项持久性、其他平台 VFS、网络盘、只读介质和真实磁盘满尚未验证；未知日志宁可保留并拒绝，也不忽略损坏记录。没有实现孤儿自动清理器、恢复 UI、自动保存调度、正式标签修改、Schema 迁移、SaveAs、偏好存储或皮肤导出。

审阅建议先核对：原件从不交给恢复连接；全部历史和非当前资源必须验证；发布前后取消的不同结果；原字节租约和文件句柄真实释放；窗口／调度接线仍未开放。
