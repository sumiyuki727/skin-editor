# B1-2AF 原件描述符与分块读取前的准入

打开工程时，读取器从当前 SQLite 行借用 `blobs.sha`。列为 BLOB 且宽度恰好为 32 B 时才复制摘要，随后读取 `byte_len`。有效及无效描述符都省去了按摘要列长度申请的 Rust 字节向量。该改动位于共享的快照读取器，独立工程打开和 FileFacts 使用同一个字段转换。[描述符读取](../desktop/infrastructure/src/project_storage/read.rs)

## 检查顺序

| 次序 | 通过条件与失败位置 |
| --- | --- |
| 1 | 先检查原件表行数。成功取到一行后显式检查取消，再读取这一行的列。 |
| 2 | SHA 必须是 BLOB，错类型报 `column type`，错长度报 `fixed bytes`。然后读取 SQL 整数长度，错类型报 `column type`，负值报 `blob length`。 |
| 3 | 将长度加到此前各行合计，溢出或超过额度报 `original bytes`。恰好用完额度可以继续。 |
| 4 | 仅统计相同 SHA 的分块行，要求个数等于长度除以 65,536 向上取整，否则报 `chunk count`。零长度要求零分块。到此尚未读取本件的 `part` 或 `bytes`，也未为本件取得原始字节租约。 |
| 5 | 通过上述检查才调用 `stage_reader`，按其单件和总池准入处理内容。取得的 `RawBlob` 还须与声明的 SHA 及长度一致，随后才进入本次打开的租约列表和缓存原件集合，最后执行 `ReadOriginal` 探针。 |

Small 的原始大小预检额外限定最多 8 个原件、原件合计 256 KiB、单件最多 min(原件总额度, 65,536) B，以及最多 8 个分块。独立 `SqliteProjectFile::load` 直接使用传入的 `FileLimits`，并未执行这项 SQL 汇总预检。在这个入口，65,537 B 的描述符须有两块，后续读取还会验证分块位置、内容和摘要。[Small 限额](../desktop/infrastructure/src/project_storage/file_facts.rs) · [原始大小预检](../desktop/infrastructure/src/project_storage/file_facts/preflight.rs) · [按块读取与校验](../desktop/infrastructure/src/project_storage/read.rs)

## 容量与所有权

Small 一次扫描最多从 8 行复制合计 **256 B 的有效摘要**，逐行消费。错长 BLOB 保留在当前 SQLite 行。锁定的 64 位布局下，`BlobRef` 占 40 B，连同新累计值返回时共占 **48 B**。256 B 是整次扫描处理的摘要量，48 B 是单个返回值的布局。摘要列转换的输入等长 Rust 堆副本为 **0 B**。[布局及边界回归](../desktop/infrastructure/src/project_storage/tests/original_descriptor.rs)

分块计数查询仍由 SQLite 执行。通过描述符检查后，`BlobReader` 持有当前块，`stage_reader` 先用 16 KiB 哈希工作区处理内容，命中共享原件时复用租约。新原件先预留字节再分配并复核。已处理的原件由 `raw_pins` 持有，直至恢复构建完成或本次打开退出。失败时，已取出但尚未登记的 `RawBlob` 与此前的本次打开租约按各自作用域释放。块缓冲、新原件、共享所有者、租约向量、原件集合及 SQLite 成本仍须分别核算。[字节池与租约](../desktop/infrastructure/src/blobs.rs) · [打开与移交](../desktop/infrastructure/src/project_storage/read.rs)

## 回归与交接

- 在没有分块表的内存连接中，组合摘要类型／宽度、整数类型／符号、最大 SQL 整数、额度不足及累计溢出。每组都在查询分块前报告相应首错，包括 512 KiB 的错长摘要。
- 分块计数夹具仅提供 SHA 列。覆盖零长度、65,535 至 65,537 B、131,072 与 131,073 B，以及额外分块、缺块、其他原件的分块和恰好用完额度。描述符检查无需分块载荷。
- 采用正式 schema 的第一个自制工程含 32 个汉字的 TEXT 主键，满足 SQLite 字符长度约束。先收紧原件数，再放开行数，完整打开依次报告 `table rows`、`column type`，尚未取得任何工程原件。
- 第二个自制工程按摘要顺序先装入合法原件，再遇到一份声明为 128 B 的原件。逐一解除限制和补齐分块后，首错依次为 `original bytes`、`chunk count`、`original admission`、`original digest`。最后一次允许分配 128 B 并观察到对应峰值。各条件连续重开两次，失败后原始字节占用与条目数均回到基线，独立原件的分配身份和工程文件摘要保持一致。登记后取消继续由原有 `ReadOriginal` 用例覆盖。[新增回归](../desktop/infrastructure/src/project_storage/tests/original_descriptor.rs) · [登记后取消](../desktop/infrastructure/src/project_storage/tests.rs)

下一项建议核对 `BlobReader` 换块时新旧缓冲的重叠、实际块长、末块及读取失败释放，覆盖独立入口的多块原件。B1-2AE 的 176 B 是 header／control 的跨阶段字段布局，首次解码的 12,963,520 B 与后续检查点的 59,647,560 B 分属不同阶段。完整共享 64 MiB 的准入、生产保存及同一张 Exit 票的关闭资格仍待验收。[上一阶段容量证明](P3_2B1_2AE_SNAPSHOT_COLUMNS.md) · [B 开工契约](P3_2B_START_CONTRACT.md)
