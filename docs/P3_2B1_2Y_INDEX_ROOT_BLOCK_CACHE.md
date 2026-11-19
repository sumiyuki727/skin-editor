# B1-2Y 页索引根与常驻块表

在固定 FileFacts Small 档、锁定工具链的 64 位布局下，读取检查点标签、扩展和特性时，**已保留的行、一个在途索引根或页、页目录及已加载块表的具名分配合计至多 59,647,384 B，约 56.88 MiB**。这一界限覆盖坏索引、坏页和后置行校验的退出路径。首次加载块表另有解析和所有权移交成本，共同 64 MiB 的分配前准入仍待验证。[编译期断言](../desktop/infrastructure/src/project_storage/file_facts.rs)

## 索引根的生命周期和首错

Small 索引根和历史页现在共用紧凑 JSON 解码。解析根时，已完成的元数据行继续存活。进入该索引的第一个页之前，根 DOM 已析构。故在 B1-2X 的分项中，当前完整页的预算也能容纳当前完整根，包含失败时已建立的数组、文本、Map 和解码字节缓冲。三类元数据索引按顺序读取，每个索引的目录用完即释放。[已完成行和在途 DOM 的证明](P3_2B1_2X_METADATA_ROW_FIELDS.md) · [目录与页读取](../desktop/infrastructure/src/project_storage/blocks/metadata_pages.rs)

根通过 JSON 校验后，先检查角色和声明行数。仅当原始页数与声明一致时，为有序摘要目录一次申请所需容量。页数不符时仍依次检查**全部**摘要的类型、十六进制文本和字节长度，再校验剩余字段，最后才报告 `index refs`。页数正确时，复制一份目录排序、去重，与块声明的引用逐项比较。原目录保留每个位置和重复哈希。Small 最多 32 页，两个摘要缓冲各至多 1,024 B，就地排序与去重后释放副本；随后读页时仅有原目录。[页索引实现](../desktop/infrastructure/src/project_storage/blocks.rs)

## 成功加载后实际保留的块表

以下仅计算读取侧 `Cache.blocks`。SQL 装载逐块核对原文长度与剩余额度，所有原文合计至多 1 MiB。锁定的 rusqlite 0.41.1 从 BLOB 切片创建字节向量，Rust 1.95.0 的字节切片复本按长度申请；随后原向量移入块行。每行保留的种类字符串和两份二进制摘要列表交给常驻表时均为精确容量。每个 32 字节摘要对应独立的 64 个十六进制字符，种类字符串对应另一段输入，解码后的 UTF-8 字节数不超过该段原文。因而在同一份原文额度内可以覆盖所有派生字符串和摘要缓冲。[装载与额度核对](../desktop/infrastructure/src/project_storage/read.rs) · [锁定的 BLOB 读取](https://github.com/rusqlite/rusqlite/blob/18ee76dbb976d5ef7e37ad5a8fffae678ca21a78/src/types/from_sql.rs) · [固定 Rust 切片复制](https://github.com/rust-lang/rust/blob/59807616e1fa2540724bfbac14d7976d7e4a3860/library/alloc/src/slice.rs)

块表的键占 32 B，行占 128 B，行布局已含原文、种类和两份摘要向量的句柄。采用固定 Rust B-tree 布局，叶节点 1,776 B，内部节点 1,872 B。成功装载最多 8,192 个不同块，非空树根至少一个成员、其余节点至少五个成员，节点数至多 1 + ⌊(8192 − 1) / 5⌋ = 1,639。表句柄另计一次。块表在后续检查点读取期间保持所有权；只有整份缓存释放时才收回其存储。[缓存结构](../desktop/infrastructure/src/project_storage/mod.rs) · [固定 Rust 的节点布局与分裂](https://github.com/rust-lang/rust/blob/59807616e1fa2540724bfbac14d7976d7e4a3860/library/alloc/src/collections/btree/node.rs)

| 同时计入的所有者 | 字节上界 |
| --- | ---: |
| 已完成行与一个在途根或页的 Map、数组、文本及解码字节缓冲 | 54,479,952 |
| 有序页目录及引用比较副本 | 2,048 |
| 常驻块原文及派生缓冲 | 2,097,152 |
| 块表全部节点及表句柄 | 3,068,208 + 24 |
| **合计** | **59,647,384** |

此表按申请布局计量。Small 目录容量来自 8192 行、每页 256 行；1 MiB 原文额度来自 FileFacts 配置。较宽的独立档案继续使用其自身限额，不能套用本表。[Small 准入及尺寸检查](../desktop/infrastructure/src/project_storage/file_facts.rs)

## 回归与后续

自制用例覆盖页数超额时的晚发坏摘要、未知字段与引用错误的先后次序，重复且乱序的逻辑目录，以及伪造的缺项、多项、重复或逆序引用声明。临时 SQLite 工程重开后逐行核对原文和派生容量。单独的块样例核对转义种类文本、17 个块摘要、5 个原件摘要及错误优先。既有空索引、最多 32 页、宽档 33 页、域 3 摘要和取消释放回归继续使用正式读取入口。[本轮回归](../desktop/infrastructure/src/project_storage/tests/metadata_pages_index.rs) · [分页与宽档回归](../desktop/infrastructure/src/project_storage/tests.rs)

下一项应跟踪首次通用块解析期间，已装入的块、当前 SQL BLOB、解析 DOM、摘要转换与收缩的同时占用和失败释放。节点及准入索引等其他缓存字段、恢复模型、浮点转换工作区、错误对象、控制与栈空间、SQLite 内存／临时磁盘和分配器开销仍须按其所有者核对。生产保存及同一张 Exit 票的关闭资格仍待验收。[B 开工契约第 7 节](P3_2B_START_CONTRACT.md)
