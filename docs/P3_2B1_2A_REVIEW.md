# P3-2B1-2A：捕获两域对照、原始量预检与预算复核

2026-11-17。本片完成真实冻结捕获的规范化内容／原字节投影、同Reader完整上下文比较，以及不削减旧验证的版本化资源预检。**结果仍ExperimentalValidation／Insufficient，完整B1／PB16未签收**。没有B2写入、保存事实安装、自动解除Unknown、产品保存或恢复根部署／迁移／覆盖／清理，也没有GUI／设备或皮肤Skill。

## 1. 回退与开工审阅

轮初HEAD与既定PUBLIC origin／codex/p1-bootstrap均为77ec779676555b40c4323811806d555af38a09ef，[三项CI 43987896203](https://github.com/sumiyuki727/skin-editor/actions/runs/43987896203)成功。没有实际内容／暂存差异，保留schema.rs原有标记，复用回退点不创建空提交。853文件原字节与mtime基线在本机忽略目录artifacts/p3-2b2-20261117；旧测量／冻结程序及PRIVATE恢复材料不改。

按[开工契约](P3_2B1_2A_READONLY_CONTRACT.md)先进行两路只读设计审阅，再开始代码。合入UTF-8前置、固定17项清单／INTEGER空值口径、实际收紧限额、Some原记录语义与checksum以及双取消令牌范围后才开工。文件名称最终明确为B1-2A，避免与后续B2写片混用。默认U1–U3政策继续保持，设置入口及迁移协议未由本片实现。

## 2. 真实实现与证据边界

[application固定纯值](../crates/application/src/file_capture.rs)新增FileCaptureProjection1、FileReadPreflight1、比较上下文及唯一结论InsufficientForSaveReconciliation。API7／捕获3／协调1／旧存储1、FileFactsPort1／FileCoord-Proof-v1及文件格式不变，没有新增依赖。投影只包含content／originals两域，compile-fail拒绝将其转换为PreparedWitness。

[捕获实现](../desktop/infrastructure/src/project_storage/file_facts/capture.rs)消费真实HistoryCapture并持有最初Snapshot。固定64槽按持久节点身份收集canonical root，完整消费后按manifest追加顺序编码；每个状态与bundle及时释放，不累计所有SkinProject。None按旧自制profile纯值编码形成首写投影；Some解析原action与operation比对，并用原state复算checksum，保留原始空白／扩展字段及不同于当前canonical的原state。实际原字节由只读BlobStore核对长度和SHA。

候选及工作槽与存活文件观察共享Reader的512KiB固定控制池。预留后工作、最后真实所有者退出才退额；Snapshot仍遵循application既有捕获槽和原字节租约，其旧动态成本不冒充已被此池覆盖。Reader先退出时，候选和观察继续保管各自所有权。独立prepare任务令牌覆盖消费／hash／原字节／最终发布；原捕获令牌只按core消费期规则处理，不承诺Complete后撤销已发布候选。

compare_capture核对完整request、CaptureSource、ObservationId、Reader命名空间及root。分别报告两域和所选头相等，不把头叫导航、不把相等叫已保存。q后编辑保留q来源，接口不判断活动Session是否仍停在q。不同内容、较晚文件或原ID缺位均不能推出NotCommitted；原expected不会为匹配文件latest而重写。

## 3. 预检与仍未知的预算

[预检实现](../desktop/infrastructure/src/project_storage/file_facts/preflight.rs)只读确认UTF-8，先按实际字节聚合整个sqlite_schema（含内部项），在Rust大目录复制前拒绝超限；随后原schema完整检查不删，再用编译期静态SQL检查16表的数量、原始量和最大值，严格INTEGER／非负／NULL／溢出处理。BLOB声明列也统一CAST为BLOB后计字节，避免非STRICT SQLite把TEXT按字符或NUL截断。固定清单随原观察Data计费／保管，提供preflight()只读查询；它不是完整文件验证资格。

后续旧full、事务内再次schema检查、完整历史／原字节／索引／领域验证均保留。[预算台账](P3_2B1_2A_READONLY_CONTRACT.md#6-聚合预算台账与保守准入)登记实际重叠：Cache＋已完成恢复模型＋当前bundle／decode／encode，JSON／Map／Vec capacity，checkpoint／redo再次解析，原字节池索引、PNG，以及SQLite连接／期望schema内存连接和系统临时盘。原始载荷与逻辑倍率不冒充实际分配前证明。

完整64MiB、SQLite总工作区和临时磁盘仍BudgetUnproven。页缓存建议值与自有主副本32MiB不足以推导聚合保证；本片不改全进程heap限制或把temp_store切为MEMORY冒充解决，不用预检取代完整校验，也不做正式成本测量。无生产资格签发器，因此两域相等不能解除这些门槛。

## 4. 回归与独立复核

Windows定向42项通过：原B1-1的18项保持，新增15项捕获与9项预检。[结构化证据](P3_2B1_2A_PROOF.json)登记准确名称、源码／向量和原始日志散列；最终接受还须本轮正常完整hook及包含本报告的新提交同SHA三项CI成功，77ec779旧CI只作轮前回退证据。

| 组别 | 实际覆盖 |
| --- | --- |
| 独立向量 | [两份18字段投影向量](P3_2B1_2A_CAPTURE_VECTORS.json)由独立Python字节组装／hashlib产生；None首写及Some原字节／不同原state。Some变体只证明投影，不声称合法文件迁移。两路复核之一另行重算全部checksum／字段流／SHA |
| 真实捕获 | 同Session两次None、实际DFS与alias不同、Some语义／checksum冲突、q后编辑、同数量异状态／异原字节、较晚完整文件；均只输出两域证据和Insufficient |
| 上下文 | 来源／导航修订／版本、完整请求、selection、旧观察与Reader命名空间隔离，错误expected原样保留并拒绝对正确观察匹配 |
| 取消／所有权 | 原令牌消费期与外层States／Originals／发布前取消；部分／Complete入口拒绝；外部Snapshot及Node最后持有者；Reader先Drop；观察＋一个／两个候选的N／N−1聚合准入 |
| 预检 | 空表不冒充健康文件、UTF-8中文／NUL及BLOB列实际TEXT、UTF-16明确Unsupported、大目录／大字段、声明量N／N＋1／INTEGER溢出／REAL拒绝、收紧行数 |
| SQLite／清理 | 早期／后续进度位置中断归Cancelled；阈值未独立证明具体schema阶段，不能扩大归因。真实大目录无效文件拒绝后保持源字节，正常清理及失败保留／实际重试 |

初轮28通过／2失败揭示Complete入口分类及DFS夹具不足，修正后39项通过，再补三项得42。另一次测试编译失败为progress_handler所需可变连接，修复后通过；失败日志保持。独立代码复核另发现并修复BLOB字节口径和取消错误分类。最终两路只读复核未发现本片范围剩余阻断；构建由主线独占，未叠加重型检查。

## 5. B1剩余退出条件与保管

后续仍需：原SaveTicket／q和历史提交expected关系；完整请求／导航／metadata／features对照；CurrentManual／CurrentProtection／VerifiedOpening和NoChange覆盖证明；四类Unknown所需阶段／目标连续性及注册绑定；旧Rust动态分配、PNG、SQLite与临时磁盘的共同预算／准入证明。全部满足前保持实验资格，不能跳到B2或生产保存。K1、Reference10000／Tiny4、产品性能NotEvaluated与stable Unknown不变。

源码、独立小向量、契约、审阅与同步文档按精确路径审查，经正常hook提交推送既定PUBLIC origin；最终以包含本报告的新完整SHA及三项成功CI为准。schema.rs原字节SHA256保持713f3850740db024c4b861327de560f4eb81f5820980d1f89672dcdcfd499125；旧证明和PRIVATE材料不改。新增原始日志只在本机忽略目录，不擅自上传，也不自动纳入3435405私有历史快照。无UAC或系统安全设置变更。
