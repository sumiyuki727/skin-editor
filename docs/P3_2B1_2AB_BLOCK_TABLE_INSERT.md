# B1-2AB 块表插入、节点分裂与失败释放

固定 FileFacts Small 档的一次插入，最多新增 **9,264 B** 的 B-tree 节点申请。新增节点全部归插入后的块表所有，已经计入 **5,165,384 B（约 4.93 MiB）** 的块表、原文和派生缓冲小计。本轮把这项原先针对常驻表的上界延伸到分裂进行中的时刻。[编译期断言](../desktop/infrastructure/src/project_storage/file_facts.rs) · [原有常驻表布局](P3_2B1_2Y_INDEX_ROOT_BLOCK_CACHE.md)

## 插入前已经确定的容量

正式读取器在同一读取事务中校验表结构，从空的块表开始。取第一行之前先查总行数，Small 最多 8,192 行。每行先检查取消、块长度和剩余原文额度，再读取哈希列并核对序号。随后才取原文，校验内容哈希、外壳、SQL 派生字段及依赖。插入时项目 ID、解析树与解码缓冲均已析构。当前行和先前行的原文合计至多 1 MiB，种类和摘要列表合计至多另 1 MiB。当前行的堆缓冲随行移动。[读取路径](../desktop/infrastructure/src/project_storage/read.rs) · [派生字段的存活期](P3_2B1_2AA_BLOCK_DERIVED_COLUMNS.md)

证明采用锁定的 Rust 1.95.0（提交 `59807616e`）、64 位 Windows MSVC、默认全局分配器及 `BTreeMap<StateHash, Row>`。键 32 B，行 128 B。叶节点含 11 组键值槽和父链接等字段，布局 1,776 B。内部节点含 12 个额外子链接，布局 1,872 B。工具链或布局更换时需复核。[节点布局及分裂规则](https://github.com/rust-lang/rust/blob/59807616e1fa2540724bfbac14d7976d7e4a3860/library/alloc/src/collections/btree/node.rs)

| 插入情形 | 新节点归属 |
| --- | --- |
| 空表首次插入 | 分配一个叶根 |
| 非满叶 | 复用原节点 |
| 满叶或满内部节点 | 旧节点原地保留，每分裂一级增加一个右兄弟，中间键值向上移动 |
| 分裂传播到根 | 在保留旧根和右兄弟的同时增加一个内部根 |
| 已占用的键 | 在插入前报 `duplicate block`，旧行和账本原样保留 |

按叶高 0 计算，高度 h 的完成树至少有 2 × 6ʰ − 1 个键。高度 5 至少需要 15,551 个键，所以 Small 的完成树高度最多为 4。一次插入最多分配一个叶节点，新内部节点各处于不同的高度 1 至 4，包含可能的新根。单次新增布局至多为 1,776 + 4 × 1,872 = **9,264 B**。[空位插入与升根](https://github.com/rust-lang/rust/blob/59807616e1fa2540724bfbac14d7976d7e4a3860/library/alloc/src/collections/btree/map/entry.rs) · [沿父路径传递分裂](https://github.com/rust-lang/rust/blob/59807616e1fa2540724bfbac14d7976d7e4a3860/library/alloc/src/collections/btree/node.rs)

完成树的根至少有一个键，其他节点至少有五个键，故最多有 1 + ⌊(8,192 − 1) / 5⌋ = 1,639 个节点。分裂中尚待插入的一侧可能暂时不足五项，新根也可能暂时为空。由于插入只增加节点，一次成功插入的每个中间时刻，旧节点和新增节点都是最终树的一部分，整个插入过程都能用最终节点数计费。由此得到 **1,639 × 1,872 + 24 + 2 × 1,048,576 = 5,165,384 B**，其中 24 B 是表句柄。行内向量和字符串的移动不用复制其内容。

## 首错、释放与回归

读取器在原有依赖检查之后查询键的空位。只有成功插入才累计当前行的原文字节。正式表的主键和不可变记录触发器已经阻止重复摘要。内部回归向已有键再次写入，单独验证这一防线。旧实现会先替换原行并计费再返回错误，新回归在旧实现上复现了原字节缓冲被替换。[正式表结构](../desktop/infrastructure/src/project_storage/schema.rs) · [块表回归](../desktop/infrastructure/src/project_storage/tests/block_table.rs)

普通错误或取消在入表前发生时，当前行随返回退出。已插入的前缀仍由调用方的缓存持有。整次打开失败时，`read_snapshot` 的局部缓存与本次打开取得的原件租约析构。映射按所有权逐项释放行和节点。全局分配器耗尽可能终止进程，标准库插入接口没有可返回的分配错误。该情形不在可恢复错误路径的验收范围内。[缓存的所有者](../desktop/infrastructure/src/project_storage/read.rs) · [标准库映射析构](https://github.com/rust-lang/rust/blob/59807616e1fa2540724bfbac14d7976d7e4a3860/library/alloc/src/collections/btree/map.rs) · [逐节点释放](https://github.com/rust-lang/rust/blob/59807616e1fa2540724bfbac14d7976d7e4a3860/library/alloc/src/collections/btree/navigate.rs)

- 8,192 个有界自制块分别按摘要升序和降序装入。每次核对全部键、原文及字节账本，再多加一行，核对总行数在第一行入表前被拒绝。8,192 项超过高度 2 满树的 1,727 项，覆盖多层分裂。
- 已占用键用例核对旧缓冲身份、行数及账本，并在同一键上验证预先取消、坏派生字段、缺失依赖各自的先后位置。
- 在有原件的临时 SQLite 工程中追加 160 个有效块，再追加坏 JSON。两次从完整打开入口失败后，文件摘要和独立租约均得以保留。原件账本直接观察租约释放。B-tree 节点数与释放依据锁定的源码和所有权推导。[三项回归](../desktop/infrastructure/src/project_storage/tests/block_table.rs)

## 尚待核对

下一项是 `block_refs` 与 `blob_refs` 的逐项读取及失败边界。早于本阶段的首次解码小计为 12,963,344 B，后续检查点小计为 59,647,384 B，各项按相应存活期使用。SQL 哈希与序号列、其他缓存与恢复模型、控制和栈、浮点与错误路径、SQLite 内存和临时磁盘、分配器额外开销仍需核对。完整共享 64 MiB、生产保存和同一张 Exit 票的关闭资格继续按 B 契约验收。[首次解码](P3_2B1_2Z_FIRST_BLOCK_DECODE.md) · [B 契约第 7 节](P3_2B_START_CONTRACT.md)
