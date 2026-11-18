# B1-2S 单代历史元数据的分页读取

读取一次提交时，标签、扩展声明和特性现在各自保管解析器交出的原始页缓冲。结构检查、逐行校验和域 3 摘要共用这些行。原先把每页的 `Value` 移入整表 `Vec` 的步骤已经移除。节点、准入、原件和 redo 继续沿用原有的汇总接口。[页所有者](../desktop/infrastructure/src/project_storage/blocks/metadata_pages.rs) · [共用分页校验](../desktop/infrastructure/src/project_storage/blocks.rs)

## 所有权与同时存活的容量

| 对象 | 准入、容量与释放点 |
| --- | --- |
| Small 三份页目录 | 每类上限 8192 行，格式页长 256。每类固定 32 个 `Vec<Value>` 句柄，空槽不分配行缓冲。三个目录同时存活；按每个目录两份布局（构造、移交）、一份在途 Vec 句柄及固定游标留量计算，编译期上界 **6 KiB**。它与 R 的 **34 KiB** 去重工作可同时存活，二者之和在后续原件图遍历的工作份额内，域 3 collector 另占原有份额。Reader 控制池仍为 512 KiB。[分区断言](../desktop/infrastructure/src/project_storage/file_facts.rs) |
| 一代内已接收的行 | 每页的指针、长度、capacity 一起移交。对三个目录中的全部页，行缓冲的槽成本为 `Σ(page.capacity() × size_of::<Value>())`，另计行内持有的分配。当前正在解析的新页与此前已接收的页可同时存活；移交同一个缓冲不再产生第二笔整表行槽。三个角色合计最多 24576 个**有效行**。自有 JSON 解析器逐项 push，页长在解析后核对，坏页也可能超过 256 项；尚无解析阶段及实际堆峰值的字节上界。[JSON 数组解析](../desktop/infrastructure/src/project_format/json.rs) |
| 一般档 | 合法索引可超过 32 页。额外目录按需分配页句柄，额外成本为 `overflow.capacity() × size_of::<Vec<Value>>()`；原项数、字节和格式限制继续生效。 |
| 代末／失败 | 三份目录保管到该代的观察者返回，离开循环体即释放。坏末页、语义拒绝及取消均沿所有者释放。仅末代比较结果跨出循环；历史摘要另存固定结果。[恢复循环](../desktop/infrastructure/src/project_storage/read.rs) |

索引中的页哈希保留出现顺序和次数，引用表按有序去重集合核对。同一哈希出现多次会按次数解析和持有。每页通过种类、角色、字段、行数及引用检查后才移交。三个角色均完成结构校验后才开始业务校验。因此早期标签错误与后续特性末页损坏并存时，仍先报告页结构错误。末代逐项比较与历史排序沿用原始行号和既有编码。

## 回归与下一项证明

无窗口回归核对 0、1、255、256、257、8192、8193 行的索引与遍历顺序，并以有额外 capacity 的输入检查移交前后的地址和容量。自制索引以同一内容页连续引用 32 次，覆盖 Small 超限、一般档第 33 页、缺页和错误引用。两代自制工程同时置入坏标签和后续坏特性页，核对两个读取入口的拒绝、原文件字节与额度回收。原有历史乱序、末代 Exit、跨页重复、八个独立域 3 向量和旧观察／候选释放继续覆盖真实调用路径。[页与宽档回归](../desktop/infrastructure/src/project_storage/tests.rs) · [完整读取回归](../desktop/infrastructure/src/project_storage/tests/file_facts/historical_metadata.rs)

下一项可沿 JSON 解析器核算索引根、在途页及已接收行的实际槽容量、字符串和 Map 分配。整路径 64 MiB 准入还需覆盖块缓存、恢复模型与编解码，并分别证明 SQLite 工作内存及临时磁盘。生产只读资格及同一张真实 Exit 子票的端到端验收仍待完成。[B 开工契约第 7 节](P3_2B_START_CONTRACT.md)
