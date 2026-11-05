# K1-6A：固定四槽来源、容量与只读完整截面

2026-11-06，按用户明确任务实现[K1-6规格](K1_6_SCALE_SPEC.md)的A步。先将规格审阅6文件经正常完整hook保存为`f1c8dbfdd086a84eafdb0f6ed0a2c9a1f7734389`，推送本轮核对归属及PRIVATE可见性的origin、远端SHA一致，再开始实现。备份验证的是API5／116项旧基线；下述新增实现另经正式核心检查。本轮新增量未提交，原schema.rs空差异标记保留。

仍为纯Rust、无窗口、最多4个不重叠接受区间的预编译SyntheticTest圆圈，profile1及判定／输入／冻结规则不变。观察与借用接口改变，gameplay API升为6；没有放大Reference对象数、运行成本工具、接GUI／原生设备或声称stable兼容。未访问旧源码、真实客户端或私人媒体；没有新增包、依赖、feature、线程或修改manifest／锁文件、application API及工程schema。

入口：[运行](../crates/gameplay/src/run.rs)、[帧读取](../crates/gameplay/src/run/frame.rs)、[发布](../crates/gameplay/src/run/engine.rs)、[协调来源](../crates/gameplay/src/coordinator.rs)、[任务](../crates/gameplay/src/coordinator/task.rs)、[新接口回归](../crates/gameplay/tests/frames.rs)。

## 固定容量与来源所有权

SyntheticMap仍是最多4槽的受验证编译值，MapPreparation逐项校验并构造它；便捷构造仍只接受至多4项。A步没有引入变长容器、归并索引或万对象入口。Limits增加object_capacity／with_object_capacity，默认Tiny和Reference均为4，只允许配置1至4；空地图合法。独立运行和协调创建／载入都检查自己的对象限制，较大的准备池不覆盖较小运行限制。原有输入、事件、回放及工作量默认值保持。

新建的受管来源由两份固定payload组成：Rc内的SyntheticMap和Rc内的CompiledSource管理值；后者包含地图句柄与来源额度记录。运行只克隆地图句柄，托管运行同时持有来源管理句柄；检查候选及保留任务共享同一份地图和来源额度，不再在每个运行内复制规范化地图数组。每份新来源的两个payload各计一次，并非宣称只发生一次分配；相同MapRevision令牌的独立分配仍分别计费。

ManagedRun先释放运行，再释放来源管理值，最后归还运行额度；CompiledSource先释放地图句柄，再释放来源额度。任务的运行／来源先于其相关预留销毁，取消只撤销资格；Ready、失败、已激活但仍保留的任务工作区也不提前退额。公开入口不交出受管地图所有权或可变活动运行引用，不能绕过这些释放关系。

来源共享／最后释放回归用Rc指针相等及Weak观察核对真实payload的强所有者结束，再与预留账本交叉核对；Weak自身可能保留Rc控制头，不把这个观察等同分配器／OS全部归零。调用者原始slice、编译值及页输出是调用者存储，单独说明作用域，不当作协调池内已登记来源。

## 容量公式与适用范围

以下均是编译目标的类型布局及已知临时存储预留，不是耗时／堆采样或进程成本测量。size表示相应Rust类型的size_of，不是持久化格式。

| 项目 | API6计费 |
| --- | --- |
| 来源S | source_map_bytes=size(SyntheticMap)，source_owner_bytes=size(CompiledSource)，source_bytes为二者之和；保留空图也计完整四槽容量 |
| 运行R | static_storage_bytes=size(SyntheticRun)＋64项输入接纳临时数组＋四槽资格临时值＋InspectionCandidate移动临时值＋两输入自动脉冲临时数组＋一份借用视图／页回执已知临时值 |
| 任务T | size(PreparationTask)＋运行装箱初始化值＋地图与来源管理初始化值；作为最大工作区预留至任务实际释放，不用当前阶段长度折算 |
| 协调基础 | pool_bytes＋coordinator_bytes；按实际构建布局核算，测试构建的局部失败点字段不假定与发布构建相同 |
| 独立fixture | standalone_storage_bytes=R＋2×size(SyntheticMap)，覆盖独立地图payload及已知初始化值；不借用任何协调池额度 |

协调池至少容纳基础、一个T、一个R及一个S；替换时当前运行、候选和保留旧任务合计核对同一组池。单运行storage_bytes限制包含于运行池合计，不额外授予一池。协调运行通过独立来源池覆盖S；独立构造则在配置／来源／对象校验后、分配前检查standalone_storage_bytes。因此仅够R的配置可供协调运行使用，独立构造会明确StorageLimit，不能悄悄漏掉其额外来源。

两个池仍为Tiny各64KiB／Reference各8MiB，全部加法准入及原有差1字节／两池溢出拒绝回归保持。ResourceUsage仍表示预留，不是活堆读数；不能将它与来源payload再次相加冒充内存。Rc控制头、分配器内部、编译器其他副本及完整调用栈继续单列未知，未据此宣布全部原生内存或进程预算通过。

本步使用标准库Rc／Box，真实全局内存耗尽并不保证可恢复为Result。单测在地图、来源管理值及运行这三个分配前边界注入局部拒绝，返回结构化AllocationUnavailable，验证失败不替换旧运行／帧、不留下有效候选、持有任务不早退额及实际Drop后可重试。失败点只在cfg(test)中启用；这不是实际耗尽机器内存的实验，也不夸大为任意OOM恢复能力。

## API6只读观察与分页

| 接口／类型 | 行为 |
| --- | --- |
| observation → Observation | 固定大小实时摘要；committed_counts是当前已提交前缀，ready_frame只含上一完整FrameHeader，二者来源／时间语义分开；不返回对象／账本数组 |
| committed_judgement(source,index) | 完整来源校验后的单项实时已提交判定，超出有效记录返回None；Pending的已提交Hit／Miss仍可观察 |
| ready_frame(source) → Result<Option<ReadyFrame借用>> | 首次未评估时为None；有完整截面时借用其运行，期间不能可变推进或销毁运行，不生成拥有式全帧快照 |
| FrameHeader／FrameKey | 固定RunSource、T、Evidence／profile、统计来源、T处计数、总对象数和从1开始的非回绕发布号；原记录索引与ObjectId分别表示 |
| read_frame_page(cursor,limit,output) | 核对来源→完整帧键→页长／位置／输出容量；无完整截面明确Ok(None)，旧来源不会被误报为空 |
| FramePage／FrameReadError | written、next及end明确；旧来源／旧帧、无效页长／位置、输出不足结构化区分，不修改运行或失败输出 |

页长为1至64；起始索引N是成功末页，超过N拒绝，计算count=min(limit,N-index)并检查算术。只要求输出至少容纳count项，末页count=0可用空缓冲；页长0仍拒绝。成功只写实际前缀，错误不改缓冲，尾部保持；不扫描或清空调用者的其余大缓冲。重复游标读取无副作用，确认事件、取消推进、仅改变实时Pending状态及同源同T无变化完成不更换旧完整帧键。驱动切换／恢复即使T不变也换完整来源；新截面、检查及替换载入分别拒绝旧页。

完整对象数据按该帧T及只读来源派生，只接纳判定时刻≤T的单次记录，按T计算可见／消退／退休，检查跳过标记在该运行内固定。当前活动头／实时新判定不混入旧帧。帧计数在相同完成边界缓存；Pending／Failed仍保留上一完整帧，同时实时摘要保持Pending／Failed，不能用旧帧假报恢复。

普通发布不再重建两份全谱对象帧：非空图用固定头的准备／提交两步，空图可一步完成，保持WorkBudget=1可继续；同刻真实工作收齐后才提交头。发布号耗尽明确Failed，保留已提交前缀、事件及旧完整帧，不回绕／复用，也不阻塞取消／释放。检查仍逐对象准备跳过标记及头，完成后新TransportEpoch发布Frozen；没有补过去Miss、声音或累计成绩。

## 回归、检查与差异

原有五组公开黄金用例改用tests/common中的测试侧四对象语义快照，逐项调用新只读接口；时间、身份、错误、账本、事件和控制顺序的期望不变。已做规范化文本核对，旧五组除读取表达式／快照返回类型和格式外没有内容变化。快照只用于测试，生产Observation没有这些数组；不同推进分段造成的发布号属于帧身份，不混入旧业务结果的等价比较，新帧身份由专用回归验证。

新增10项公开接口回归、5项单测及1项借用生命周期编译拒绝，gameplay共16项单测、20项驱动、10项帧、16项冻结、18项检查、26项协调、23项合成及3项编译拒绝，共132项。覆盖实时前缀／旧完整帧、Pending取消后发布、Failed只读、来源各分量、分页边界／原子输出、同T换代、检查／替换载入隔离、较小对象容量、来源共享／真实最后释放、分配前拒绝和发布号边界。

正式python -B tools/check_core.py通过424项Python回归、格式、Clippy、三个核心包无窗口测试及release构建。新增一项嵌套frame模块的禁止时钟／项目依赖canary，规则未放宽；未新增包／依赖／feature。python -B tools/check_desktop.py --architecture-only通过；本轮修改不接桌面运行，备份完整hook和新增模型验证分别记录。本机忽略日志目录为artifacts/k1-6a/。

首轮新增测试将最后发布号放在需要两个截面的目标上，得到CounterExhausted；已把夹具定位到单个出现截面并复测，未修改耗尽规则。整理换行时曾误清空本轮14个已改Rust文件，已从本轮备份精确恢复并重放改动，验证全部非空、旧黄金内容及最终正式检查通过；未改写Git历史、旧证明或其他已有文件，相关失败／修复日志保留。

本步只完成A的四槽表示／来源／容量及只读接口，不宣称完整K1／KS／KT23。变长存储上界、分步归并／初始化、完整原始工作计量、10000轨迹及实际成本仍属B–D；本轮没有运行计量工具。下一步先审阅本实现与K1-6规格的一致性，再按用户明确任务决定后续增量；GUI／原生设备、K0-B及K2–K5均未开放。

收尾检查通过10份Markdown的本地路径／锚点、围栏／尾空白及git diff --check，783个仓库候选文件扫描通过。实际增量28文件，暂存区为空；两个manifest／锁文件及原schema.rs规范化字节保持HEAD，旧证明未改。本轮源码／测试／文档尚未提交，不将回退提交的远端状态套用于新增量。
