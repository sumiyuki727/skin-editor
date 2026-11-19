# B1-2AA 块的 SQL 派生字段与项目 ID

读取器在首次解码项目块时取出 `value.id`，随后直接对照当前 SQLite 行的种类、版本和项目 ID。在固定 FileFacts Small 档及已审计的 64 位布局下，从首次解析返回到派生列比较结束，块表、原文和派生缓冲同时存活的申请布局小计至多 **5,165,384 B（约 4.93 MiB）**。[块解析](../desktop/infrastructure/src/project_storage/blocks.rs) · [逐列核对](../desktop/infrastructure/src/project_storage/read/block_columns.rs) · [布局断言](../desktop/infrastructure/src/project_storage/file_facts.rs)

## 校验顺序与所有权

块哈希、完整 JSON、外壳字段及未知外壳字段按原顺序校验。外壳通过后，只对 `skin-editor.project` 移出内容对象的直接 `id` 字段。其余内容字段仍按原有用途留待后续验证。取得的字符串或缺失／类型错误随解析结果临时交给调用者。块表中的行仍只持有原文、种类与两份摘要列表。[取字段的移动语义](../desktop/infrastructure/src/project_storage/values.rs)

| 路径 | 首个可见错误的顺序 |
| --- | --- |
| 读块 | 外壳 → SQL 种类 → SQL 版本 → 解析出的 ID → SQL 项目 ID → 块与原件依赖 |
| 添加块 | 相同哈希的字节核对及复用 → 新块序号与外壳 → 块数及原文额度 → 块依赖 → 原件依赖 → 解析出的 ID → SQL 插入 |

SQL 种类不符就立即返回。种类与版本通过后才取出延迟的 ID 结果，随后读取可空的 SQL 项目 ID。文本从当前行借用，在该行有效期间验证整个 UTF-8 并比较，保留内嵌 NUL、大小写及空字符串；项目 ID 列允许 NULL。类型错误仍归为 `column type`，非法 UTF-8 沿用锁定版 rusqlite 的 `sqlite` 类别。非项目块要求该列为 NULL。成功和失败都会结束对该行的借用，预期 ID 在读块核对函数返回时释放。[读块与依赖](../desktop/infrastructure/src/project_storage/read.rs) · [存储错误分类](../desktop/infrastructure/src/project_storage/mod.rs) · [锁定的 rusqlite 行接口](https://github.com/rusqlite/rusqlite/blob/18ee76dbb976d5ef7e37ad5a8fffae678ca21a78/src/row.rs)

## 分配前的界限

| 此阶段计入的所有者 | 申请字节上界 |
| --- | ---: |
| 块表节点及表句柄，沿用 1,639 × 1,872 B + 24 B | 3,068,232 |
| 先前行的原文字节与当前 Rust BLOB | 1,048,576 |
| 先前行的派生缓冲，当前种类、两份二进制摘要及临时项目 ID | 1,048,576 |
| **小计** | **5,165,384** |

取 BLOB 前，读取器检查当前块的 256 KiB 上限与全部原文的 1 MiB 剩余额度。Small 的已完成字符串容量等于 UTF-8 字节数，摘要向量也只保留实际项数。当前种类、每个摘要及直接 ID 在原文中占据互不重叠的片段。JSON 文本解码后的字节数最多等于该片段长度，32 B 摘要也能由各自的 64 个十六进制字符覆盖。因此全部这些派生分配都能计入第二份 1 MiB。临时 ID 直接移出解析树；比较 SQL 文本无需按列长度再申请 Rust 缓冲。SQL 文本本身可以长于对应 JSON 字段。[前序块表推导](P3_2B1_2Y_INDEX_ROOT_BLOCK_CACHE.md) · [紧凑解析器](../desktop/infrastructure/src/project_format/json.rs)

首次解码及摘要转换分别使用 B1-2Z 的 12,963,344 B、11,779,760 B 小计。进入本表阶段时，当前解析树和解码缓冲都已退出。后续检查点读取另用 B1-2Y 的 59,647,384 B 小计。三个阶段按存活期分别核算。64 MiB 共享预算仍待核对块表插入、两张引用表、先读取的 SQL 哈希与序号列、其他缓存及恢复模型、浮点和错误路径、栈、SQLite 自身空间及分配器开销。宽档仍按原有限额执行。[首次块解析](P3_2B1_2Z_FIRST_BLOCK_DECODE.md) · [共同准入及关闭资格](P3_2B_START_CONTRACT.md)

## 可复核的边界

- 在内存 SQLite 上，将新的借用取值与原来的原生取值逐列比较。覆盖长文本、空串、非法 UTF-8 尾字节、NULL、BLOB、整数和实数。成功的结果直接指向 SQLite 提供的文本字节。[列转换与首错回归](../desktop/infrastructure/src/project_storage/read/block_columns/tests.rs)
- 恰好 256 KiB 的项目块验证移出的 Small ID 内容与实际容量。含无效 ID 和未知外壳字段的块在窄、宽两档均先报告外壳错误。添加块的失败用例逐次核对实际表行数、缓存行数和字节账本，成功用例从 SQL 读回原 ID 并复用相同块。[容量与添加块回归](../desktop/infrastructure/src/project_storage/tests/block_columns.rs)
- 自制临时工程在健康行之后同时放入项目 ID 冲突和缺失依赖。两次加载均在依赖检查前返回 ID 冲突，每次保留独立的原件租约且文件摘要相同。测试直接观察原件仓储用量；临时字符串、解析对象及向量的释放依据 Rust 的所有权与返回作用域。[整条加载失败路径](../desktop/infrastructure/src/project_storage/tests/block_columns.rs)

下一项先核对块表插入时分裂节点的同时占用与失败释放，再处理 `block_refs`、`blob_refs` 的逐项比对。生产保存与同一张 Exit 票的关闭资格继续按 B 契约验收。
