# B1-2I 已验证旧文件的普通追加元数据

2026-11-18。回退点为 ff290b79fdaddf6d33d01a460d311a280372341c。H 保管的是捕获已有的元数据，本片回答一个更窄的问题：在原文件的确切基线上，如果原票按现有 writer 的普通追加分支执行，新检查点应包含什么？[H 审阅](P3_2B1_2H_REVIEW.md) · [原写入器](../desktop/infrastructure/src/project_storage/write.rs)

### 来源与准入

FileFactsReader 以同一 Reader 的实际 q 及完整只读观察查询。沿用 H 的版本、完整 Ticket、FileFactsRequest、观察、会话、root 和私有来源守卫；选定 commit 必须是票上的原 expected。最新 commit 及 generation、逻辑绑定、原件 SHA256、features_root 和完整 before 域 3 都应吻合。原恢复与 q 的请求序号可以不同。以基线已验证的全部提交查重，涵盖早期 Automatic／Exit。需要能容纳一代的 Reader 提交额度，以及可加一的 generation。原件字节相同的副本只能提供逻辑证据，物理绑定仍待验证。

### 投影与结果

只在显式查询时使用真实 q 的快照和 H 私有来源。以原 Ticket 生成虚拟回执，last／protected 指向该回执；Manual 还推进 manual 并在已有 marks 末尾追加一次，旧顺序保持。复用现有格式的 FileCoord-Proof-v1 域 3、真实 features、标签和扩展原字节。读取器在共享 512 KiB 额度中临时预留排序与编码空间，查询结束退额。q、H before 和已有页不增加常驻字段。

返回固定、独立版本的条件摘要、来源、原始上下文和假设回执，结论恒为 Experimental／Insufficient。调用方保管返回值，其长期副本另计。无真实来源、基线不符或 ID 已占用时没有投影；错版本／错查询上下文、超容量和取消返回对应错误。既有 G 接口可按后续独立观察及票上 commit 读取实际历史域 3；生产流程没有从两个公开摘要安装回执的入口。

### 验收界限

用独立结构模型冻结黄金值，覆盖 Manual 推进及 mark 顺序、Exit／Automatic 保持旧 manual、同数异值和原始二进制。实际文件以同头、同导航、同历史的独立 writer 作为 oracle，验证实际 Committed、另一次真实 NoChange、已占用的历史非 Manual ID、后续历史观察、来源不符、容量和取消。新增查询沿用现有 Work 准入：Automatic 仍不接受，Exit 的协调路径仍由关闭协议管理。

完整 B1、NoChange／Unknown 分类、物理连续性、共同 64 MiB 和 B2 写入保持原审阅状态。此契约只使用普通旧文件追加规则；首次保存、另存、features 编辑／迁移及回执后的来源继承留待各自规格。

### 实现与证据

- [只读查询](../desktop/infrastructure/src/project_storage/file_facts/profile/append.rs)在 Reader、q、基线观察同时存活时工作。结果保留 H 来源和 before 摘要，另列假设回执及完整域 3。未来追加是否满足其他写入条件仍待 writer 核对。
- [八份独立黄金](P3_2B1_2I_VECTORS.json)引用 H 的具名结构模型，保留原标签、扩展和 features。将指定回执应用于该结构后按 FileCoord-Proof-v1 编码，字段数和 SHA256 经独立复算。
- [真实文件回归](../desktop/infrastructure/src/project_storage/tests/file_facts/profile/append.rs)使用自制工程。实际 Manual Work 和独立 writer 都从 A 定位至 C，提交后又追加一代，再以 G 选回目标提交。另验真实 NoChange、Replayed、早期 Exit ID 碰撞，以及完整元数据相同而原件 SHA 不同的复本。
- Exit／Automatic 的证据覆盖结构模型及旧 writer 的提交分支。通过应用新入口取得的实际 Work 本片仅验证 Manual。关闭子任务须经完整协调协议，当前基础设施没有可用的生产 authority。
- 查询控制额度按已有 source／q／view 仍存活的情况验证恰好足额和差一字节，另验 Reader 提交上限、自制 80702 字节 features 的单观察查询，以及取消后释放临时额度。这些是管理额度证据。宿主实际内存、无限份返回值及整条保存链的共同预算仍未计量。

本片仅同步受控源码、黄金及文档。新的原始日志仅保存在本机。旧取消原因与旧 first-before 均为 Unknown，既有证明、私有恢复快照及 K1 结论沿用原记录。
