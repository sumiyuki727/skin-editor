# P3-2B1-2A：冻结捕获投影与只读资源预检

2026-11-17。由已推送77ec779及三项成功CI接续。用户逐轮接续授权下，先冻结本设计并独立审阅，再实施适当只读增量。B1-1不重做；完整B1／PB16、B2写入、产品保存、恢复部署／迁移／覆盖／清理、GUI／设备和皮肤Skill均不由本片签收。

## 1. 本片交付与不变的版本

消费真实HistoryCapture，产生完整内容／原字节的规范化投影；与仍持有源守卫的InspectedFileFacts作实验比较。另在旧完整验证前增加版本化原始量预检，输出固定清单。本片结果始终ExperimentalValidation／Insufficient，不能转换为PreparedWitness、生产authority或写许可，不安装保存事实、不自动重试写、不解除Unknown。

API7／捕获3／协调1／旧存储1及FileFactsPort1／FileCoord-Proof-v1保持。新增FileCaptureProjection1和FileReadPreflight1独立版本化，不修改已发布文件、旧request_hash或五域规范。projection只复用FileCoord-Proof-v1的content／originals两域编码，不填造其余三个域，不假装完整FileFactsProof。

## 2. 窄接口和来源

application只增加固定纯值。infrastructure的Reader提供prepare_capture(request:FileFactsRequest, capture:HistoryCapture, originals:&BlobStore, cancel:CaptureCancellation)，返回不可裸值构造的FileCaptureCandidate。候选只提供summary，不返回活动Session、可变项目或写端口；内部持有最初CaptureSnapshot，维持原字节与捕获槽的真实寿命。

prepare在任何遍历前核对既有请求版本、完整ExistingVerified绑定、expected、非零key，以及Snapshot的project／history／lineage／address_epoch和实际运行session。节点／admissions／原字节数额只允许收紧既有Small实验上限。捕获一次消费；完成后不重新从Session抓取q+1，不伪造CaptureSource。

InspectedFileFacts::compare_capture(candidate, context)要求完整请求相等，context包含candidate实际CaptureSource及本观察ObservationId，核对version与完整source（含navigation_revision）及同Reader命名空间，拒绝旧观察、旧session、旧绑定／代次或改写expected。不借把原Ticket.expected改成文件latest实现已提交匹配。本片处理的是同观察的捕获投影，不声称已登记产品物理目标或完整SaveTicket。

比较分别返回content_equal、originals_equal、selected_head_equal及实际q来源／ObservationId。完整头与导航不是同一个字段；本片不把selected_head_equal叫导航相等。返回结论只有InsufficientForSaveReconciliation，纯值可复制不授信；比较对象本身仍持有源守卫，摘要副本不能延长守卫。

## 3. 首写规范化

最初Snapshot先于poll保管。HistoryCapture按DFS发射，固定64槽以持久NodeId定位每个canonical root，拒绝重复／缺失；Complete且Snapshot.is_complete后按manifest.nodes稳定追加／alias顺序生成content。发射顺序不能代替追加顺序，不能累计全部SkinProject。

每个状态使用既有独立codec求当前canonical root，随即释放该状态和bundle。origin=None按当前自制文件profile的纯值operation_bytes和node_checksum形成record_version2首写投影，state为该canonical root；它不是已落盘事实。origin=Some先以operation(raw)核对CapturedOperation，并以原record.state复算node_checksum；只支持当前record_version2，冲突或未知版本拒绝。之后保留原record_version／state／action／checksum，当前canonical另列，不重编码原action、不丢额外描述或原始空白，不强求原state等于当前canonical。changed_objects、运行session／revision不进入持久content，运行关联仍保留在外层。

admissions按既有ASCII kind／id顺序编码，原字节按SHA顺序编码；实际从只读BlobStore取字节核对SHA／长度，不只相信引用。只读来源缺失或取消拒绝全部结果，不产生部分摘要。编码尺寸与状态／字节计数属于原始工作量，不当作allocator或RSS测量。

原捕获取消令牌由core在消费期检查；prepare独立任务令牌覆盖投影、原字节校验及最终发布。Complete后的原token不被承诺能撤销候选；已发布候选保持不可变，外层取消不会回收其他实际消费者仍持有的Snapshot。部分消费或已经Complete后才交给prepare的producer不能构成完整候选。

## 4. NoChange与Unknown的证据边界

| 情形 | 本片结论与后续门槛 |
| --- | --- |
| 两域相等 | 只证明本q完整内容与原字节投影相等；仍Insufficient，不确认保存 |
| Manual A／保护C，完整树相同 | 独立报告所选头比较；导航、意图、保存位置不从两域推导 |
| 找到CommitId | 仍需原expected／origin／head／prefix／counter／导航及完整持久请求规范化匹配，单一ID不够 |
| 原请求NoChange | 还需完整metadata、意图及CurrentManual／CurrentProtection／VerifiedOpening覆盖证明；捕获metadata是既有事实，不能当提交后metadata，且Manifest没有文件features |
| ID缺位或文件追加更晚历史 | 只报两域差异／证据不足；缺少任务阶段与目标连续性不能证明NotCommitted |
| 来源冲突、读失败、取消或预算欠证 | 拒绝或Insufficient；不更改已确认事实、不自动新写、不释放原Unknown |

历史请求expected通常是提交前一代，B1-1观察expected严格等于文件latest。后续必须为历史提交匹配另定约，不能在本片放宽全绑定。新节点首写规范化只是两域投影，不宣称原请求五域已经定义。

## 5. 新增资源预检

FileReadPreflight1先只读确认PRAGMA encoding为UTF-8，其他编码明确Unsupported，不改文件。随后以SQLite标量聚合读取sqlite_schema全部目录行（含sqlite_%）及type／name／tbl_name／sql的UTF-8字节合计和最大行载荷，拒绝超过128行、128KiB合计、64KiB单行；不先复制大目录字符串到Rust。随后仍执行原schema::verify，再对已核实固定表作count／sum／max原始量查询，之后仍执行原full验证（含其事务内第二次schema核实）。预检不替代schema／全分支／原字节／索引／领域校验。

固定清单包含catalog及按下方顺序排列的16表，每项登记rows、bytes、largest。它是原始量，不伪装实际内存；blobs的bytes特指声明原字节量，实际内容由后续完整验证读取。设B(x)=length(CAST(x AS BLOB))，只对合法nullable字段以0计，空表sum／max为0；每行表达式必须非NULL、INTEGER且非负，结果严格读INTEGER，溢出／REAL／负值／异常NULL／SQLite错误拒绝，不作浮点截断。SQL为编译期静态常量，不接收任意表／表达式。未知schema不能先运行任意表聚合。

| 固定次序／表 | 每行bytes表达式 |
| --- | --- |
| catalog（独立项） | B(type)＋B(name)＋B(tbl_name)＋COALESCE(B(sql),0) |
| 0 header | B(family)＋B(profile) |
| 1 features | B(name)＋B(version) |
| 2 blobs | byte_len（声明值，非实际载荷） |
| 3 blob_chunks | B(bytes) |
| 4 blocks | B(bytes)＋B(kind)＋COALESCE(B(project_id),0) |
| 5 block_refs | B(owner)＋B(target) |
| 6 blob_refs | B(owner)＋B(target) |
| 7 nodes | B(action)；其余固定字段仍由后续完整验证核实 |
| 8 addresses | B(node_id)＋B(alias)＋B(branch)＋B(ordinal)＋B(checksum) |
| 9 state_views | 0（仅接受0行） |
| 10 commits | B(origin)；其余固定字段仍由后续完整验证核实 |
| 11 manual_marks | B(commit_id)＋B(node_id) |
| 12 object_ids | B(kind) |
| 13 tags | B(name)＋B(value) |
| 14 control | B(generation)＋COALESCE(B(save_id),0)＋COALESCE(B(manual_id),0)＋COALESCE(B(protected_id),0)＋B(address_epoch) |
| 15 extensions | B(name)＋B(bytes) |

| 对象 | 预检口径／上限 |
| --- | --- |
| blocks | 行数8192；bytes／kind／project_id原始载荷合计metadata上限，单行codec.block_bytes＋1024 |
| nodes | 行数64；action合计metadata上限，单份action不超过64KiB |
| blobs／blob_chunks | 引用声明总长度／实际chunk字节分别不超过256KiB，单份／chunk不超过64KiB；资源数8，chunk行数不超过8 |
| block_refs／blob_refs | 分别最多65536行、每行两个32字节引用；这是保守索引上限，不是展开todo峰值证明 |
| commits／manual_marks | 分别最多128行；origin字节合计metadata上限，单份不超过32；marks为固定身份字段 |
| addresses／object_ids | 64／256行；kind原始UTF-8载荷合计metadata上限，单份不超过32 |
| tags／extensions／features | 最多4096／1024／256行；各表原始载荷合计metadata上限；tag单行4352、feature单行4096、extension单行metadata上限 |
| header／control／state_views | 1／1／0行；header family＋profile最多1024字节；固定字段仍由完整schema及验证核实 |

表中64／256／8／128等为封顶，实际同时服从Reader已收紧的nodes／admissions／originals／commits／metadata／original_bytes。tags行上限随nodes×64收紧；声明单份原始字节不超过min(64KiB,original_bytes)，总chunk行数封顶originals。addresses／marks／control固定载荷分别最多72／32／72字节每行及对应行数乘积。新增预检门槛为独立实验profile的保守收窄，不表示旧格式被迁移；预检通过仍可能被旧full更严或语义校验拒绝。

Reader固定控制池仍512KiB。预检清单成为固定Data的一部分，按真实size_of与原来结果一起保留；候选固定载荷／64槽工作表与存活文件观察共享同Reader控制池，预留后工作、真实Drop后退额，不为每次prepare重建池。候选持有的CaptureSnapshot属于既有application捕获所有者；其旧动态分配及codec／operation JSON等欠项继续列在下节，不把它们算成已经覆盖。

## 6. 聚合预算台账与保守准入

| 阶段／同活所有者 | 已可核对部分 | 仍未证明部分 |
| --- | --- | --- |
| 打开／schema预检 | 源／scratch守卫、固定清单与标量结果；目录量在Rust字符串复制前拒绝 | SQLite打开、schema解析、VM及期望内存连接已可能分配；不是整体分配前准入 |
| Cache与索引 | 新原始字节／行数清单、既有原始字节池预留 | BTreeMap节点、Arc控制块、Vec capacity／扩容重叠、所有JSON字符串／Map |
| full恢复 | 原始块尺寸、8MiB闭包累计和16MiB逻辑限额分别记录 | Cache＋全部已完成RestoreNode＋当前bundle＋模型＋重编码同时存活，逻辑检查晚于构造；finish另有children／ranges／stack／sets |
| metadata／proof | 固定返回槽；hash主体流式 | read_paged完整Vec、checkpoint再解析、redo复制排序、旧cost漏项与原始索引 |
| q投影 | 同控制池固定64槽／候选与真实Snapshot寿命，逐状态释放 | Capture旧模型／历史和operation JSON／codec临时分配仍未聚合预留 |
| 媒体probe | 现有原始字节池；WAV借用切片 | PNG第三方构造分配另列，不能省略验证 |
| SQLite／临时磁盘 | 自有主副本额度与实际清理保持 | cache_size不是总内存上限；temp_store=FILE的排序／物化可在VFS系统临时目录，不在主副本32MiB账本内；不能改MEMORY冒充解决 |

完整64MiB受管聚合、SQLite私有工作区及临时磁盘仍BudgetUnproven，所有生产共同profile准入继续不可用；即使两域一致也不签PB16。SQLite全进程heap limit不能作为单Reader私有额度。后续须逐分配预留／capacity、明确容器上界、PNG和SQLite／VFS方案及共同编辑／准备manifest，或另审更窄的可证明实现；不静默抬高预算、不删验证、不测正式成本、不升级性能结论。

SQLite依据分别见[cache_size](https://www.sqlite.org/pragma.html#pragma_cache_size)、[临时文件与位置](https://www.sqlite.org/tempfiles.html)及[全进程heap limit](https://www.sqlite.org/c3ref/hard_heap_limit64.html)。这解释了为何页缓存建议值与主副本额度不足以推导本Reader的聚合保证，未据此宣称已经观察到实际临时盘峰值。

## 7. 最小验收及审阅门槛

实现前独立审阅上述接口／规范及预算边界。最低回归：DFS与alias不同、None首写与同Session重复捕获、Some额外原字节保留、q后编辑隔离、Manual／Latest头差异、完整来源／绑定／观察拒绝、相同数量不同状态／资源／admission、真实原字节缺失、取消／容量差一单位、候选及外部Snapshot真实延迟释放、较晚文件仅Insufficient；目录／字段超额在Rust大复制前拒绝，UTF-8按字节，清单固定且不把原始量叫内存证明。原B1-1全部回归与旧核心／存储测试保持。

两路实现前审阅均同意限定实验方向；其要求的UTF-8前置、17项固定清单／空值与整数口径、实际收紧上限、Some语义／checksum核对和双令牌覆盖已合入正文。随后才进入功能实现，完整64MiB与SQLite资格仍未取得。

实现、独立复核、原始失败／成功记录及精确提交／CI另由本轮审阅记录登记。所有fixture为自制显式测试根，不读真实用户工程或恢复目录；旧证明、schema.rs原字节、PRIVATE材料保持。原始忽略日志仍仅本机，不自动扩展旧私有恢复快照。
