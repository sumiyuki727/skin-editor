# P3-0 审阅：身份、格式与增量存储原型

2026-11-02。本轮完成了约定范围的定稿与独立原型，交由用户审阅；**没有接入正式保存／加载功能、自动保存调度或导出，也没有自动暂存／提交／推送 Git**。P2产品Rust源码、manifest／lock、原声音路径和既有验收门槛未改变。

先读[格式v1](PROJECT_FORMAT_V1.md)，再看下方验证与限制。机器证据索引见 [P3_0_PROOF.json](P3_0_PROOF.json)；实验的数据库／媒体／日志留在忽略的 `artifacts/p3-0/run-20261102`，不提交这些大文件或任何私人素材。

## 定稿内容

- 持久项目／历史／节点／对象身份用128位，文件二进制16字节、文本32位hex；每次打开的运行epoch单独生成。
- 保留n≤14、b≤28的文件内短映射；新增k≤26节点短键，外部永久引用hp1固定102字符。深度不增加地址长度，副本的新节点不会因短号相同而混同。当前API6不偷偷改变。
- 采用SQLite事务页容器作为v1路线，成功提交后main自包含；DELETE journal＋EXTRA同步，崩溃journal保留至引擎恢复。单文件不是每次重新打包整份工程。
- 节点／操作／状态／父关系与原始依赖只增不删、不覆盖；标签为独立元数据。保存事实独立追加，系统标签不能通过自定义标签入口伪造。
- 手动位置和最新保护检查点分离，普通重开默认手动位置，主动选择才恢复最新；保存旧内容不删除已存在新分支。完整历史不依赖重放旧Command。
- 字节摘要有明确二进制封装；合法JSON无需复现某语言的排版。固定[黄金向量](../tools/p3_storage/golden_v1.json)供未来Rust实现对照。

这里的“定稿”指本轮可审阅约定；尚未作为用户数据格式发布。`p3-0`实验profile与将来的正式SkinProject profile明确区分，避免原型文件被误识别为已支持的完整工程。

## 实验结构与引用

实验只依赖Python标准库，未引入产品第三方依赖或第二套产品核心。实际产品仍是模块化Rust单体；application唯一写入口保持，UI／AI不访问实验连接。

| 文件／对象 | 方法／职责及引用 |
| --- | --- |
| [identity.py](../tools/p3_storage/identity.py) · NodeReference | identity／new_identity／parse_identity、u64／base36、node_key／short_alias／branch_address及解析；NodeReference.text／parse。仅标准库；ContractError继承ValueError |
| [values.py](../tools/p3_storage/values.py) · Block／BlobInput／Node／TagEdit／Batch／Limits／Receipt | Block.make／sha／envelope，BlobInput.prepare／parts，Node.record／checksum，Batch.fingerprint；只引用identity。独立不可变dataclass，没有继承产品对象 |
| [schema.py](../tools/p3_storage/schema.py) | script生成实验表／索引／触发器；没有项目写用例 |
| [store.py](../tools/p3_storage/store.py) · Store | create／open／close／context、status／captured、block／node／blob_parts／tags／open_position／resolve、commit／verify；内部_put_blob／_put_block／_put_node／_meta／_navigation，引用schema、identity、values；StorageError继承ContractError |
| [fixtures.py](../tools/p3_storage/fixtures.py) | png_bytes／write_media／project_block／make_store／append，生成自制PNG／WAV／不透明字节及实验状态，引用Store和values |
| [metrics.py](../tools/p3_storage/metrics.py) | snapshot／difference，本进程时钟／Windows资源计量；不启动设备，不承担产品预算裁定 |
| [check_p3_storage.py](../tools/check_p3_storage.py) | 默认测试、benchmark、crash_matrix／crash_child，门闩和新输出目录限制；引用上述工具 |
| [test_check_p3_storage.py](../tools/test_check_p3_storage.py) | IdentityTests／StorageTests继承unittest.TestCase；受控错误、锁和字节验证，不依赖GUI／真实资源 |

完整源码身份在证明JSON中记录；10月30日的[旧代码清单](review/README.md)保留原快照，不倒填本轮工具。正式接线时只移植经过测试的契约：application定义捕获／保存／恢复端口，infrastructure实现，desktop组装。不会让产品调用这套Python类，也不会公开通用SQL写接口。

## 确定性与故障验证

两套运行时各通过39项单元测试；Python3.9.6／SQLite3.35.5、Python3.12.14／SQLite3.53.1。包括：

| 范围 | 已验证 |
| --- | --- |
| 身份／地址 | 极值与非法编码、跨打开epoch拒绝、跨项目上下文、文件复制后相同n别名但不同持久节点、固定二进制黄金向量 |
| 历史不变 | 分叉保存／重开、保存旧节点仍保留新分支、同ID改操作拒绝、SQL更新／删除／REPLACE拒绝、删除后对象身份不复用 |
| 标签／检查点 | 标签增改删不改变节点／地址、不新增内容节点；禁止伪造保存标签，元数据不能移动保护头；手动／最新分离与导航直接子关系校验 |
| 资源闭包 | PNG／WAV／未知编码原字节往返和复用、跨分支资源保留、缺依赖拒绝、源文件变化回滚、实际插入首个chunk后取消释放部分资源 |
| 失败／去重 | 节点／元数据超限、受控SQLITE_FULL、读事务阻止COMMIT、失败后新提交可用、捕获冲突、回执丢失／同CommitId重放、同ID换载荷拒绝 |
| 不兼容／损坏 | 未知版本不触碰原文件、未知必要能力只读、移除SQL保护拒绝、资源损坏报错且不删旧数据、不透明扩展保留 |

两套引擎各7个进程强制终止门闩，合计14例：开始、原始字节后、状态块后、节点后、标签后、COMMIT前、真实COMMIT后。写原始字节后已有约15MiB主文件和journal，覆盖页缓存溢出而非仅内存未落盘。前6例恢复旧代次，COMMIT后恢复完整新代次；root字节不变、无半标签／半节点、恢复后journal归还。不是COMMIT系统调用中途断电测试。

两引擎还分别只读打开对方写出的20301节点文件，完整校验通过，文件SHA256前后相同；不是跨OS验证。初轮Python3.12曾发现测试夹具的sqlite连接context并不会负责close，导致临时文件占用；已改为显式closing，两套重测通过。失败日志和早期R1／R2结果均保留，未改成成功。

## 规模与资源成本

每套最终轮次独立顺序运行，Windows10 build19045、本地E盘路径。自制原始字节134,227,478 B（约128.009 MiB），先建10000深链及10000宽分支，再300次单节点增量、300次标签修改，最终20301节点／块，深10300，root直接子10001。20001个既有节点的有序ID＋checksum指纹在增量前后保持；正常打开又校验每个frame。逻辑元数据载荷7,437,848 B，不含所有SQL索引／页空隙。

| 指标 | SQLite3.35.5／Python3.9.6 | SQLite3.53.1／Python3.12.14 |
| --- | --- | --- |
| main文件 | 144.41 MiB | 144.44 MiB |
| 10000深链，40批×250，合计墙钟／CPU | 2.695 s／2.344 s | 2.544 s／2.250 s |
| 10000宽分支，40批×250，合计墙钟／CPU | 3.527 s／3.125 s | 3.368 s／2.828 s |
| 300次内容增量，p95／最大 | 5.568／6.070 ms | 5.675／10.916 ms |
| 上述内容写入量，含journal | 39.59 MiB | 40.31 MiB |
| 300次标签，p95／最大 | 3.923／5.173 ms | 3.999／6.026 ms |
| 上述标签写入量，含journal | 14.82 MiB | 14.80 MiB |
| 3次相同标签改动：增量墙钟／写入 | 12.279 ms／157,372 B | 12.802 ms／157,372 B |
| 3次整文件复制＋同改动＋替换对照 | 496.156 ms／454,444,732 B | 487.336 ms／454,518,460 B |
| 单次全量校验墙钟／CPU | 3.185 s／3.188 s | 2.866 s／2.875 s |
| 本轮进程生命周期私有提交峰值 | 19.21 MiB | 23.28 MiB |
| 生命周期工作集峰值 | 25.43 MiB | 29.80 MiB |
| 10轮打开／关闭，关闭后私有提交范围 | 12.54–13.48 MiB | 16.52–17.05 MiB |
| 10轮打开／校验／关闭，墙钟范围 | 3.135–3.207 s | 2.877–4.006 s |

十轮各次均无写入、完整节点和根不变，无持续单调增长或遗留journal；这只是有限十轮，不证明任意长会话。生命周期峰值包含Python解释器／SQLite／对照复制／验证，不是各阶段峰值，也不是相对产品新增内存。64KiB流式输入和2MiB页缓存有明确边界；不能把它们称作全部进程内存≤4MiB。

I/O来自GetProcessIoCounters，包含页／索引／journal及工具开销，不是物理NAND写入。增量仍有约135–138KiB／内容提交、约51KiB／标签提交的放大，不能把5KiB逻辑修改说成只写5KiB。复制对照只回答“每次完整复制main有多贵”，没有实现或测试ZIP候选，也不证明SQLite胜过所有存储方案。

CPU使用process_time（GetProcessTimes，声明分辨率100ns，实际短段量化可达15.625ms），墙钟perf_counter；报告原始时长，不用短段0ms CPU宣称无成本。没有独立调度参考、UAC、GUI／音频或GPU负载；不构成P2 CPU验收，也不追认存储接线后的零欠载／20ms／资源预算。没有做全程磁盘冷缓存／断电持久性校准。

## 判断、未完成范围与下一步边界

本轮结论：SQLite增量容器在此规模下可避免每次复制原始资源或整棵历史索引；不可变记录、独立标签、已提交检查点恢复及容量／锁错误可以受控验证，适合作为正式存储的下一步路线。没有以删分支／重写节点换取结果。

仍需后续增量完成：

1. 以Rust实现完整SkinProject／历史状态DTO与校验、资源租约和application捕获／恢复；映射持久ID并显式升级API，处理R03长会话回执。原型状态只是fixture，不验收完整嵌套图层、音效配方、配置的文件往返。
2. 选定并锁定Rust SQLite适配依赖，复核许可证、Windows／Linux／macOS构建与实际VFS持久化；原型两引擎通过不等于产品绑定已选或平台已通过。
3. 首次候选文件发布、另存／备份／迁移／未知正式语义、损坏恢复、真实I/O错误和断电场景另测。对恶意数据库还须完善读取预算／结构防护，不能以普通校验宣称安全解析所有不可信文件。
4. 实现有界后台写通道与自动／退出调度的fake时钟验收，验证捕获p95≤2ms及流式暂存候选目标；现在同步原型事务不跑在GUI里。
5. 同一正式构建做保存与图形／声音组合负载，保持零欠载、提交p95≤20ms、内存／GPU及实际回收硬项，播放CPU≤15%仍为优化目标。

全量校验仍需约3秒、反复扫描约329MiB逻辑读取，是正式打开前要处理的可见等待；不能为加速跳过资源闭包验证。可先提供有界后台校验进度并在验证结束前禁止正常覆写；更复杂的增量可信校验需要额外证据，未在本轮自行放宽。

普通编辑器仍无文件保存，自动保护周期、设置／偏好持久化和静态皮肤导出均未实现。本轮到此交给用户审阅，不继续接入这些功能。

## 检查与修改边界

正式核心检查包含398项Python工具测试、Rust格式／Clippy／无窗口测试／边界／私有访问编译拒绝；桌面检查运行无窗口默认与all-features检查，不打开设备。最终结果及工作区保全核对写入[P3_0_PROOF.json](P3_0_PROOF.json)。

现行入口同步了数据兼容、地址、架构、技术栈、路线／P3准入、调度方案、README和文档／审阅索引；AGENTS已有相应长期约束，本轮保持原字节，不塞入实验参数。旧产品源码／锁、历史JSON与归档不变。既有未提交改动保留；Git HEAD／暂存区不变，所有本轮改动仍未提交。
