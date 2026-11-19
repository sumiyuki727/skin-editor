# B1-2AD 块行的 SQL 哈希与序号

块读取器现在直接从当前 SQLite 行提取 32 字节的 `sha` 和 8 字节的 `sequence`。长度匹配后，哈希复制进 `StateHash`，序号按大端字节序读成 `u64`。两列转换所需的 Rust 堆缓冲为 **0 B**，错误列的长度也不会触发相应大小的 Rust 分配。两张引用表复用同一个定长 BLOB 读取函数。[读取实现](../desktop/infrastructure/src/project_storage/read.rs) · [既有引用表核对](P3_2B1_2AC_REFERENCE_TABLES.md)

## 分配前边界与报错位置

| 次序 | 读取器在当前行执行的检查 |
| --- | --- |
| 1 | 表总行数通过后按序号取行。成功取到一行才显式检查取消，然后读取 `length(bytes)`，核对块上限和剩余元数据字节额度 |
| 2 | 先取哈希，再取序号。每列先确认 SQLite 类型为 BLOB，再检查定长。错类型沿用 `column type`，错长沿用 `fixed bytes` |
| 3 | 将序号与已装入块数加一比较，有缺口则返回 `block sequence` |
| 4 | 取出块原文，按已有路径校验内容哈希、JSON、派生字段及依赖，随后入表计费 |

锁定的 rusqlite 0.41.1 对 `Vec<u8>` 先复制整个 BLOB。使用定长借用后，SQLite 行只提供切片，长度正确时才读取固定值。借用在本行内结束，可以将哈希保留到后续校验和入表。无效输入在第 2 或第 3 步退出时，当前块的 Rust 原文向量尚未创建，先前已加载的块仍由当前打开操作的缓存持有。[锁定的列取值](https://github.com/rusqlite/rusqlite/blob/18ee76dbb976d5ef7e37ad5a8fffae678ca21a78/src/types/from_sql.rs) · [块内容校验](../desktop/infrastructure/src/project_storage/blocks.rs)

Small 的块表上限为 8,192 行。两列合计最多请求 **16,384 次列取值**。若所有行均合法，最多读出 8,192 × (32 + 8) = **327,680 B** 的定长数据，逐行消费。当前逻辑值为一个哈希和一个序号，序号只参与当前行的连续性检查。缓存仍可使用 B1-2AB 的 **5,165,384 B** 保守小计。两列的借用与取值无需加上另一份按输入长度申请的堆缓冲。栈及编译器临时值、SQLite 页和当前行、BLOB 排序、事务与其他缓存所有者分别核算。[块表分配及计费](P3_2B1_2AB_BLOCK_TABLE_INSERT.md)

FileFacts 的原始大小预检对 blocks 汇总的是块原文、种类和可空的项目 ID。正式表另有哈希和序号的长度约束，并在完整打开时接受 SQLite 完整性检查。该表允许存储类型与列声明不同，SQLite 的 `length` 对 TEXT 和 BLOB 采用不同单位。夹具中的 32 个“雪”字能满足哈希长度约束，仍须由读取器以错类型拒绝。[预检公式](../desktop/infrastructure/src/project_storage/file_facts/preflight.rs) · [表结构及完整性检查](../desktop/infrastructure/src/project_storage/schema.rs) · [SQLite 长度函数](https://www.sqlite.org/lang_corefunc.html#length)

## 回归与释放

- 将 8 字节借用结果与旧转换逐列对照，覆盖 0、7、8、9 B、512 KiB、TEXT、非法 UTF-8 TEXT、NULL 和数值。正确长度的结果指向 SQLite 当前行，非对称字节序列和 `u64::MAX` 均得到预期数值。原有 32 字节差分回归继续覆盖哈希宽度。长 BLOB 用来检查列读取函数的拒绝路径。[序号取值](../desktop/infrastructure/src/project_storage/read/block_columns/tests.rs) · [32 字节取值](../desktop/infrastructure/src/project_storage/read/reference_tables/tests.rs)
- 对已有缓存前缀注入一行，同时放入不同阶段的错误。测试核对长度与余额先于哈希、哈希类型和长度先于序号、序号先于原文取值，以及内容哈希先于 JSON。每次拒绝后核对旧行的缓冲身份、块数和字节账本。[首错矩阵](../desktop/infrastructure/src/project_storage/tests/block_row_header.rs)
- 在采用正式 schema 的两份自制临时工程末尾各追加一行，序号均为合法八字节的最大值。使用 TEXT 哈希的工程先报 `column type`。换成 32 字节 BLOB 后，读取器继续检查序号并返回 `block sequence`。每份工程连续两次从完整打开入口失败，读取前后文件摘要相同，独立原件租约保留，本次打开取得的原件占用回到基线。该账本直接观察原件租约，缓存和当前行的释放按 Rust 所有权与返回作用域核对。[完整打开回归](../desktop/infrastructure/src/project_storage/tests/block_row_header.rs)

下一项优先核对 `read_snapshot` 在块扫描前读取并持有的 `header`、`control` 字段，尤其是要留到末尾才校验的提交标识及代次。它们会与块表或后续恢复对象同时存活。首次解码与后续检查点的小计仍按各自生存期使用。其他恢复模型、栈、错误路径、SQLite 与分配器成本，以及完整共享 64 MiB、生产保存及同一张 Exit 票的关闭资格仍待验收。[B 契约第 7 节](P3_2B_START_CONTRACT.md)
