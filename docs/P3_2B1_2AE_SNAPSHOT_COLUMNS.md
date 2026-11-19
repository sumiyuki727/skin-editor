# B1-2AE 打开期间保留的 header 与 control 字段

打开工程时，读取器在 SQLite 行内检查 `family`、`profile` 的 UTF-8 并比较内容。身份和代次只复制正确宽度的 16 或 8 字节。对于 SQL 类型正确、宽度错误的 BLOB，读取器只保留一个表示错长的状态，在原校验位置返回 `fixed bytes`。因此，两行的列值需要的**随输入长度增长的 Rust 堆缓冲为 0 B**。这条性质同时适用于 FileFacts Small 和使用同一读取函数的独立工程打开入口。[行取值](../desktop/infrastructure/src/project_storage/read/snapshot_columns.rs) · [打开流程](../desktop/infrastructure/src/project_storage/read.rs)

## 分阶段容量与释放

| 阶段 | 已核对的所有者及生存期 |
| --- | --- |
| header 行 | 借用两个文本列，保存比较结果。四个待校验身份组成的 `HeaderIds` 为 68 B。header 查询结束便可释放语句和行，随后才读取 control。 |
| control 行与身份校验 | 身份缓冲继续存活。读取 control 后，独立的 `address_epoch` 为 17 B，延迟核对的 `Control` 为 96 B。转换成功后，四个身份缓冲和原始 epoch 用毕，业务身份进入缓存。两次查询都在开始后续扫描前结束。 |
| 原件、块、历史和索引验证 | `Cache.header` 的五个身份共 80 B，`Control` 为 96 B。两者同时存活的字段布局合计 **176 B**。控制值含代次、三个提交标识及四个计数器，最多只有 manual 标识可为空。 |
| 完成或中途失败 | 校验最新回执、手动标记及闭包后比较 control。最后一个用途是与重新核算的元数据开销比较。成功返回后，header 随缓存保留，control 用毕。失败时两行均无列值堆所有权需要退还，原件租约由相应打开作用域释放。 |

尺寸按锁定工具链的 64 位布局取值，回归检查具体大小和析构需求。176 B 可分别计入此前的 Small 首次哈希／解码分项（12,963,344 + 176 = **12,963,520 B**）和后续检查点页分项（59,647,384 + 176 = **59,647,560 B**）。两个分项对应不同阶段。构造与返回临时值、完整栈帧、错误对象、SQLite 语句／当前行、其他缓存与恢复模型、分配器成本仍须分别核算。[布局回归](../desktop/infrastructure/src/project_storage/read/snapshot_columns/tests.rs) · [首次块解析](P3_2B1_2Z_FIRST_BLOCK_DECODE.md) · [检查点页与块表](P3_2B1_2Y_INDEX_ROOT_BLOCK_CACHE.md)

Small 原始大小预检分别限定两个 header 文本合计 1,024 B、control 五个二进制字段合计 72 B。正式 schema 还对四个 header 身份、代次及地址代次施加长度约束。读取时仍独立核对 SQL 存储类型、UTF-8 和字段宽度。读取超长夹具时，SQLite 当前行的空间和文本验证工作仍存在。[预检](../desktop/infrastructure/src/project_storage/file_facts/preflight.rs) · [表约束](../desktop/infrastructure/src/project_storage/schema.rs)

## 首错与失败保护

1. 先按列顺序获取 header 九列，再判断格式／版本。即使 family 不匹配，后列的 SQL 类型错误或非法 UTF-8 仍优先报告。格式不受支持时，返回位置仍在 control 查询之前。
2. 先读完 control 九列的 SQL 类型，再依次校验 history、lineage、address epoch、project、root 的宽度和非零身份。`save_id` 与 `protected_id` 必须为 BLOB。缺少手动标记的 SQL NULL 与非空错长 BLOB 有独立表示。
3. 原件、块、历史、闭包、索引及手动标记通过后，才按 generation、save、manual、protected、节点数、块数和原件字节数核对。前项不匹配时立即报 `control`，后项的错误长度或零身份只在执行到该项时报告。元数据计数仍在重新计算缓存开销后核对。[调用位置](../desktop/infrastructure/src/project_storage/read.rs)

回归覆盖空值、相邻宽度与 512 KiB BLOB，对照原有字节转换，验证 SQLite 连接关闭后仍能使用捕获字段。正式 schema 的自制工程依次累积错误，先只破坏代次，再追加序号有缺口的块，最后把 save 设为 NULL。三个阶段分别报告 `control`、`block sequence`、`column type`。每个阶段从完整入口连续打开两次，均核对文件摘要和独立租约。前两个阶段已取得的原件占用回到基线，第三个阶段在取得原件前退出。[列级回归](../desktop/infrastructure/src/project_storage/read/snapshot_columns/tests.rs) · [完整打开与租约回归](../desktop/infrastructure/src/project_storage/tests/snapshot_columns.rs)

下一项建议沿 `blobs(sha, byte_len)` 原件描述符追踪到分块读取和租约登记，先核对描述符取值及长度／总量／块数拒绝时的释放。完整共享 64 MiB 的分配前准入、生产保存以及同一张 Exit 票的关闭资格仍待验收。[B 开工契约](P3_2B_START_CONTRACT.md)
