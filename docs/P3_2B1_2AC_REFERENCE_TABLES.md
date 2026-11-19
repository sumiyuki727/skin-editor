# B1-2AC 块引用表与原件引用表的逐行核对

`block_refs` 和 `blob_refs` 的每一行描述一对 32 字节摘要，分别标识引用方和目标。读取器现在从当前 SQLite 行借用这两列，逐列与缓存核对。**列值复制和期望引用列表汇总所需的新增 Rust 堆申请均为 0 B**，包括遇到错长 BLOB 的路径。Small 档在这一阶段仍保有 B1-2AB 已核算的块表、原文及派生缓冲，具名小计至多 **5,165,384 B**。[逐行核对](../desktop/infrastructure/src/project_storage/read/reference_tables.rs) · [块表容量及原文所有权](P3_2B1_2AB_BLOCK_TABLE_INSERT.md)

## 准入与同时存活的所有者

FileFacts 在读取快照上先做原始大小预检。两张引用表各自至多 65,536 行，单行两列的合计字节数至多 64 B，每表合计至多 4 MiB。进入比较后仍须逐列核对类型和长度，预检合计 64 B 也可能来自 31 B 与 33 B。通用工程读取器另有 2 MiB 的 SQLite 值／记录长度限制。[原始预检](../desktop/infrastructure/src/project_storage/file_facts/preflight.rs) · [连接设置](../desktop/infrastructure/src/project_storage/mod.rs)

两张表的期望引用来自已验证的块。Small 原文合计至多 1 MiB，每个声明的摘要占据互不重叠的至少 66 个原文字节，故两张表**合计**最多核对 ⌊1,048,576 / 66⌋ = **15,887** 个期望项。核对器直接遍历原块表及行内的引用向量。每个期望项请求一行，两表各再请求一次以确认表尾。提前返回只会减少请求，所以 `Rows::next` 最多调用 **15,889 次**。这个界限只计 Rust 侧向游标取行的次数。SQLite 扫描、排序、分页缓存及临时磁盘的工作量另计。[原文与摘要容量](P3_2B1_2Z_FIRST_BLOCK_DECODE.md)

| 所有者 | 生存期与容量 |
| --- | --- |
| 原有块表与派生缓冲 | 核对两表时均由缓存持有。插入时至多新增的 9,264 B 节点已归入 5,165,384 B 小计 |
| SQLite 当前行 | 锁定的 rusqlite 0.41.1 提供行内 BLOB 切片。先确认 BLOB 类型，再按切片长度转为借用的 32 字节数组，错长即返回 |
| Rust 比较与迭代 | 当前的期望哈希、借用及迭代器为固定大小的值，借用在下一次取行前结束。工作量随引用数增加，列副本与全表缓冲申请始终为零 |
| 三个查询的语句和游标 | 块行扫描结束后先释放其游标和语句，再打开块引用查询。该查询完成或失败时释放其所有者，成功才开始原件引用查询。三个扫描至多同时持有一组本阶段的语句和游标 |

原来的 `Row::get::<_, Vec<u8>>` 会先把整个 BLOB 复制为向量，才发现长度不对。当前路径只借用 SQLite 的字节，固定长度比较不会按列长度申请内存。这里的核算限定于 Rust 侧的列副本和应用所有者，SQLite 提供当前行所需的内存仍由引擎管理。[锁定的列取值接口](https://github.com/rusqlite/rusqlite/blob/18ee76dbb976d5ef7e37ad5a8fffae678ca21a78/src/row.rs) · [锁定的 BLOB 转换](https://github.com/rusqlite/rusqlite/blob/18ee76dbb976d5ef7e37ad5a8fffae678ca21a78/src/types/from_sql.rs) · [块扫描移交](../desktop/infrastructure/src/project_storage/read.rs)

## 失败顺序与释放回归

每个期望项先检查取消，再取行，缺行立即返回。随后依次核对 owner 的类型、长度、值，再以相同顺序核对 target。owner 不符时直接报告该表的索引错误。非 BLOB 沿用 `column type`，错长沿用 `fixed bytes`。表尾直接探测是否多出一行，期望项为空时也执行该探测。第一张表报错后立即返回。

- 用原生列转换逐项对照长度为 0、31、32、33 B 及 512 KiB 的 BLOB，以及 TEXT、非法 UTF-8 TEXT、NULL、整数和实数。合法摘要的地址与 SQLite 当前列相同。512 KiB 用例直接验证列读取函数，Small 文件预检会更早拒绝这样的引用行。[列转换与两表首错矩阵](../desktop/infrastructure/src/project_storage/read/reference_tables/tests.rs)
- 两张表分别验证成功、缺行、错长、错类型、值冲突及多余坏行。多个引用方共用同一目标时，改变最后一个目标的第 32 字节仍可检出。交叉测试核对第一表错误先于第二表的准备错误，并覆盖预先取消和空期望表的表尾探测。[游标回归](../desktop/infrastructure/src/project_storage/read/reference_tables/tests.rs)
- 有界临时工程移去最后一条原件引用并恢复原有表约束。连续两次从完整打开入口读取，均报告 `missing blob index`。两次失败后文件摘要及预先独立持有的原件租约均保持，仓储用量回落到原基线。打开失败时本地缓存与本次取得的原件租约随调用退出。语句所有者的生存期依据显式析构与错误返回作用域核对。[完整打开回归](../desktop/infrastructure/src/project_storage/tests/reference_tables.rs)

下一项是块行扫描中早于块原文解码的 SQL 哈希与序号列，需分别核对错误类型、长度、同时占用和释放。此前首次解码的 12,963,344 B 与后续检查点的 59,647,384 B 按各自生存期使用。其余缓存、恢复模型、控制与栈、浮点和错误路径、SQLite 及分配器成本仍待核对。完整共享 64 MiB 预算、生产保存与同一张 Exit 票的关闭资格继续按 B 契约验收。[B 契约第 7 节](P3_2B_START_CONTRACT.md)
