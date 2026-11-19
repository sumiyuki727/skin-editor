# B1-2W 元数据对象节点与重复页所有权

Small 读取标签、扩展或特性索引时，同一索引中重复出现的页哈希现在共用首次验证的页。**不同页仍可在行内容被拒绝前同时占用超过 64 MiB 的 Map 节点空间。** B1 的共同预算验收仍然开放，生产保存和同一张 Exit 票的关闭门控仍待后续验收。

## 同一页如何复用

读取器先核对索引的角色、项数、页数和引用集合，再按原顺序访问页。首次出现时检查块类型、解码结果、角色、完整字段、该位置所需的行数和页引用。后续出现从同一只读缓存、同一角色和同一索引内引用首份所有者，并重新核对当前位置所需行数。足 256 行的页被用作短尾时仍返回原有的 `page count` 错误。逐行校验、索引访问、迭代和域 3 摘要均按出现次数读取，业务重复项仍可被拒绝。[分页读取](../desktop/infrastructure/src/project_storage/blocks.rs) · [行所有者与视图](../desktop/infrastructure/src/project_storage/blocks/metadata_pages.rs)

每个 Small 目录增加 32 个单字节位置。首次出现的查找复用已解码的有序哈希表，在 32 页内至多作 496 次哈希比较。三个角色的目录连同移交及游标留量在已审计的 64 位布局下合计 5,528 B，仍在 6 KiB 控制预算内。一个目录退出会释放其中所有不同页，位置索引没有独立的堆所有权。较宽档案继续逐页解析。[控制分区断言](../desktop/infrastructure/src/project_storage/file_facts.rs)

## 单页 Map 的请求字节

证明针对锁定的 Rust 1.95.0（`59807616`）、64 位目标以及 serde_json 1.0.154 的默认 BTreeMap 后端。该布局中指针、String、Value 分别占 8、24、32 B。节点有父指针、两个 16 位字段以及各 11 个键槽和值槽。对齐后，一次叶节点分配申请 **632 B**。内部节点另含 12 个子节点指针，申请 **728 B**。工具链、布局或容器特性变更时需重新验证。[依赖锁](../desktop/Cargo.lock) · [固定版本节点布局与分裂](https://github.com/rust-lang/rust/blob/59807616e1fa2540724bfbac14d7976d7e4a3860/library/alloc/src/collections/btree/node.rs) · [serde_json Map 后端](https://github.com/serde-rs/json/blob/20572a6d066958542d79cdd6cf1a7cd9f0c8e1f8/src/map.rs)

空对象起初没有节点。首次成功插入创建根叶。节点有空间时复用原分配，满节点分裂时增加右兄弟，分裂传播到根时再增加新根。键、值及子树的所有权随分裂移动。一次插入所分配的节点都留在插入后的树内，分裂中无需重复计算另一份完整子树。每个完成的节点至少含一个成员，每个已插入成员在解析时已消费一个非根 JSON 值。正在等待插入父对象的子树也来自同一份节点配额。故整页各对象合计，包括一次插入的瞬间，保守界为 **(8192 − 1) × 728 = 5,963,048 B**。[解析与插入顺序](../desktop/infrastructure/src/project_format/json.rs) · [BTreeMap 的插入入口](https://github.com/rust-lang/rust/blob/59807616e1fa2540724bfbac14d7976d7e4a3860/library/alloc/src/collections/btree/map/entry.rs)

重复键在解析对应的值前失败。子值的语法、深度或项数失败时，父对象尚未执行该次插入。解析错误、坏尾页或后置校验失败使各自已拥有的树沿析构路径释放。析构遍历逐步退还叶和内部节点的布局，直到最后一个实际所有者退出。此处只计节点向分配器申请的布局字节。系统分配器的驻留开销、错误对象和分配失败行为不在这个数内；普通堆耗尽也不能当作协调端口已返回可恢复的容量错误。[映射的析构](https://github.com/rust-lang/rust/blob/59807616e1fa2540724bfbac14d7976d7e4a3860/library/alloc/src/collections/btree/map.rs) · [逐节点回收](https://github.com/rust-lang/rust/blob/59807616e1fa2540724bfbac14d7976d7e4a3860/library/alloc/src/collections/btree/navigate.rs)

## 不同页的拒绝路径

自制回归的足页含 256 行，每行有 15 个单成员对象。完整页有 7,944 个 JSON 值、深度 5、编码长 27,495 B。改变一位数字在行中的位置即可得到 32 个相异的同长度页，页原文共 879,840 B。它们在页级检查阶段有效，行的标签业务形状会在三个角色的页全部读完后才接受检查。[行校验时序](../desktop/infrastructure/src/project_storage/read.rs)

只计这些行里的叶节点，32 个不同页可同时持有 **32 × 256 × 15 × 632 = 77,660,160 B，即 74.0625 MiB**。这个下界已经超过共同预算，无需相加多个无法同时发生的上界。它是页读取路径的静态反例。回归只构造两个不同页，交错引用到 32 个位置，验证每个位置的身份、值和顺序，并从单页真实对象数核对推导。没有测量进程峰值，也没有构造完整用户工程。[分页回归](../desktop/infrastructure/src/project_storage/tests.rs) · [分裂顺序与失败分类回归](../desktop/infrastructure/src/project_format/json/compact_page_tests.rs)

下一项宜缩短不同页通用 DOM 的保管时间，保留“各角色先完成结构检查，再按既定顺序核对行与摘要”的首错语义。此前 49 MiB 继续作为数组、文本和解码字节缓冲的局部保守界。索引根、浮点工作区、首次通用块解析、缓存、恢复模型、SQLite 内存及临时磁盘仍需独立核算。[在途数组与文本](P3_2B1_2V_JSON_TEXT_SCRATCH.md) · [共同预算契约](P3_2B_START_CONTRACT.md)
