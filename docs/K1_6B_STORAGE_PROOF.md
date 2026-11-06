# K1-6B：分配请求容量证明与实施记录

**2026-11-07后续入口：** 本页B一致性修订已通过正常完整hook备份推送为`bf3fff51cac5e7a9c1fa618efda3e07c11e86cac`，远端SHA一致；随后完成[K1-6C规模回归增量](K1_6C_SCALE_REVIEW.md)，当前共167项gameplay通过，运行时及本页容量布局保持。C新增量未提交、待独立审阅，D实际成本未开始。下方B记录保留原时点。

**2026-11-06一致性审阅更新：** B初版32文件已通过正常完整hook备份推送为`747222c8c69e39ff39a84e933a73883bcfedb80c`，PRIVATE origin及远端SHA一致。本次修复三处原始工作计数边界，新增7项公开回归，正式核心通过425项Python及156项gameplay；Reference10000／Tiny4、API7／profile1和存储布局保持。详见[一致性审阅记录](#2026-11-06一致性审阅记录)。本次修订未提交，C／D仍未开始，下方初版记录保留其当时时点。

2026-11-06。先将K1-6A一致性测试／审阅11文件通过正常完整hook备份推送为b3c5f088932666c97ae8576b468b0c7b177bc7d5，PRIVATE origin和远端SHA均已核对。备份基线137项gameplay，本轮B新增实现另行验证。来源仍为SyntheticTest profile1，不接GUI／设备，不测实际成本或宣称stable兼容。

## 分配前依据

通用Vec API不保证最小capacity，不能只凭方法名准入。本机rustc为1.95.0，commit为`59807616e1fa2540724bfbac14d7976d7e4a3860`，host为x86_64-pc-windows-msvc。对该commit的标准Global实现，Vec把try_reserve_exact交给RawVec；空向量len=0时，grow_exact检查相加得到请求capacity，finish_grow按元素布局申请未初始化存储，set_ptr_and_cap保存该capacity。Global返回的可用切片长度是请求字节。因此对非零大小T，单次从空Vec申请C项的暴露容量为C，请求布局为Layout::array(T,C)。这是对固定源码实现的推导，非通用API保证。[Vec入口](https://github.com/rust-lang/rust/blob/59807616e1fa2540724bfbac14d7976d7e4a3860/library/alloc/src/vec/mod.rs)、[RawVec分配路径](https://github.com/rust-lang/rust/blob/59807616e1fa2540724bfbac14d7976d7e4a3860/library/alloc/src/raw_vec/mod.rs)、[Global实现](https://github.com/rust-lang/rust/blob/59807616e1fa2540724bfbac14d7976d7e4a3860/library/alloc/src/alloc.rs)。

仅storage模块封装该路径：先验证检查算术及完整预算，再一次预留；不增长、收缩、转boxed slice、Clone、resize、collect或批量填充。分配后capacity核对只是防漂移canary，不用于追加收费。Formal检查固定rustc1.95.0；工具链变化须重审。类型布局在构建目标上用size_of／align_of／Layout计算，不序列化内存布局。分配器内部／OS占用、Rc控制头及完整调用栈仍未知。

容量C=max(4,N)，N最多10000。保留四槽最小请求，避免把空／小图按len低收费；大图按完整请求容量计费。来源圆圈、两个ID索引、运行状态及检查候选各为独立有界缓冲。索引双缓冲预留在运行池，最终圆圈在来源池；运行和检查共享最终来源，各持自己的可变状态。每个缓冲从空开始逐项写入，没有随N清零或整值复制。

初次5项容量回归通过后才将Reference默认上限从4开放至10000，随后G0及完整回归再次核对最终构建布局。Tiny仍为4，并保留其4对象配置上界；自定义Limits可显式设置不超过10000的容量及独立字节上限。没有上调两个池、输入／事件64槽或工作量256的Reference默认值。

## 布局、所有权与计费

令C=max(4,N)，size及align均是构建目标上相应类型的size_of／align_of；Layout::array检查乘法及isize范围，预算相加也检查溢出。以下是分配请求清单，不是分配器物理块或进程内存报告。本机验证范围为Windows x86_64-pc-windows-msvc／Rust1.95.0；同版本源码推导不代替Linux或其他目标的实际检查。

| 分配 | 元素／容量／请求布局 | 所属与释放 |
| --- | --- | --- |
| 最终圆圈 | Circle，C项，C×size(Circle)，对齐align(Circle)；先申请未初始化容量，逐项读取并写入 | 来源池，随最后来源所有者释放；MapPreparation中的同一缓冲直接移交Rc地图，不重分配／复制 |
| 两份ID索引 | IdKey=(ObjectId,原记录位置)，各C项，各C×size(IdKey)，对齐align(IdKey) | 运行池任务工作区；准备完成实际销毁两缓冲，工作区预留仍保留至任务Drop，取消／失败持有时不早退额 |
| 当前对象状态 | ObjectState=(单次判定,检查跳过标记)，C项，C×size(ObjectState)，对齐align(ObjectState) | 每运行独立；Box运行先分配，随后每步初始化一项有效记录，闲置capacity不伪报为已初始化对象 |
| 私有检查状态 | 同上C项；请求仅建立描述符，首次step单列分配，之后每步检查并初始化一项 | 单运行预留另一整份状态容量，激活只移动描述符并释放旧状态，不对N条fill／复制 |
| Rc地图／来源管理／Box运行／Rc池 | 固定类型payload大小及自身对齐；Rc／Box包装只移动固定描述符 | 分别为来源／运行池；每份实际payload有独立计数守卫，Rc克隆不重复计费。Rc控制头仍Unknown |

PreparationCosts::costs_for_objects(N)给出：source_map_bytes=size(SyntheticMap)+圆圈容量字节，source_owner_bytes=size(CompiledSource)，S为二者之和；state_bytes是一份状态缓冲，index_bytes是两份索引的合计。R=原四槽static_storage_bytes+2×(C-4)×size(ObjectState)，包含当前及私有检查容量；T=size(PreparationTask)+size(SyntheticRun)+size(SyntheticMap)+size(CompiledSource)+index_bytes，覆盖任务描述符及固定初始化／移动工作区。协调基础B=pool_bytes+coordinator_bytes。N≤4仍收四槽容量，旧costs()／static_storage_bytes()只代表四槽下界，不可作为万对象成本。

创建／替换准入在扫描、请求号变化或分配之前核对来源池S与运行池B+所有活动／保留任务的T／R总和，检查自己的run storage_bytes≥R。检查复用来源S，只新增T+R，不能为共享句柄再收费。单运行限制包含在池合计中。独立MapPreparation::with_limits按自身描述符、圆圈和两索引总量核对调用者限制；RunPreparation另外核对R、来源payload及已知Map／运行移动值，不能借用协调池额度。

最终release构建的静态布局交叉核对如下。只查询类型大小／对齐及costs_for_objects，没有创建地图／运行、计时、采样堆或读取进程内存；此表不是K1-6D成本测量。Circle为64字节／对齐8，SyntheticMap为112／16，SyntheticRun为9664／16；工作区索引每项16字节，状态每项24字节。

| N／容量 | 来源S | 运行R | 任务T（含双索引） | 协调器／池payload |
| --- | --- | --- | --- | --- |
| 0–4／4 | 424 | 12648 | 11128（索引128） | 192／304 |
| 10000／10000 | 640168 | 492456 | 331000（索引320000） | 192／304 |

N=10000创建的预留为来源640168、运行823952字节；保留当前运行并准备同谱检查的预留为来源640168、运行1316408，均在各8MiB默认池内。测试构建含局部受控拒绝字段，布局以自身costs计算，不把release字节强套给它。跨目标／工具链须重新核对，不能套用本表。静态核对源／日志留在忽略目录artifacts/k1-6b/static-layout.*，不作为正式成本工具或发布入口。

ResourceUsage仍是准入预留。新增AllocationUsage分别报告来源／运行池当前活着的已请求堆payload字节及分配数，已包含在预留内，不能与预留相加；调用者的原始slice、页缓冲和内联协调／任务值单列。池自身在池仍存活期间计一个基础payload；原始容量、累计分配尝试及字节、当前payload寿命是不同记录。Buffer先销毁数据再销毁payload守卫；运行／来源及其守卫先于额度记录释放。失败任务、取消任务和已激活但仍保留的工作区各按真实所有权计费。

## 分步工作与API7

SyntheticMap不再Copy／Clone。MapPreparation::new／SyntheticMap::try_new及SyntheticRun::new仍只提供至多四对象的有界便捷路径；大图使用MapPreparation::with_limits、RunPreparation的step／finish，或统一协调器。公开接口变为gameplay API7；profile1的时间、半径、判定及输入语义不变，application及工程schema没有变更。

校验依次执行3个分配管理步骤、逐条规范化记录及ID键写入、逐项第二索引初始化、自底向上分步归并和相邻重复检查。每次归并比较和搬移分别消耗一步，只排序ID索引，不重排地图。错误按最早原记录位置以及DuplicateObject→UnsortedMap→UnsupportedOverlap优先级选择；记录相同位置以重复优先，后面重复不覆盖先前乱序。诊断只保留位置／种类和固定计数，Debug不展开N条内容。

运行初始化先分配一份状态容量，再逐条写入；检查重建同理。检查游标新增独立非回绕进度，分配完成但checked仍为0时也会使旧继续值失效；耗尽仍可取消，分配拒绝撤销私有检查且保留旧完整帧。普通发布仍只提交固定头，查询、初始化结束和激活都不暗扫全图；固定队列最多64项的管理及分配／释放本身不具硬实时保证。

WorkCounts为每任务／运行局部的固定计数，分别记录原始记录读取／写入、索引初始化、ID比较／搬移、状态初始化、搜索／检查／自动对象检查、发布、摘要／帧对象／实时账本读取、管理步骤及分配尝试／请求字节。汇总不扫描对象。诊断溢出保持溢出字段旧值并显式valid=false，不回绕；普通请求及继续进度仍按各自结构化耗尽规则拒绝。只读查询只更新局部诊断，不推进玩法或改帧键；调用错误不计成功读取。同T重复读取没有对象全扫描或新分配。

对N条记录及L=ceil(log2(max(N,1)))，ID比较≤N×(L+1)、搬移≤2N×(L+1)；原始读取／写入各N、索引初始化2N，单次相应步骤各≤1。创建运行状态初始化N；检查私有运行先初始化N，再逐项重建N，检查数N。固定64槽输入／事件数组初始化及移动属于明列的固定管理工作，未伪造为对象初始化为0的完整进程成本。allocator内部、完整调用栈、编译器其他临时副本仍未知，所有计数均不等于耗时或产品预算结论。

## 回归与当前退出范围

先前137项小模型业务预期保持；仅将依赖具体第k次Pending的准备／检查夹具改为有界继续，并按新的归并及分配步骤核对原始计数，重建失去Copy后的测试输入值。新增8项公开规模回归及4项单测，gameplay共149项：

- 容量覆盖0／1／4／5／31／32／9999／10000，分配前拒10001及Tiny第5项；逐一核对暴露capacity、请求字节、单步读取／比较／搬移／初始化及有限调用上界。
- 错误优先级用独立小输入顺序校验器交叉核对，含稀疏／逆序ID、u32最大值、非相邻重复和多错位置；不改变原记录顺序。
- 受管两池恰够／差1字节、单运行不足、原始超限原子拒绝；大图归并取消后保留真实缓冲，旧继续不能影响替代候选，Drop才释放。
- G0按默认Reference及预算1／256各完成10000次Miss、10000个连续Judgement事件，无SoundIntent，末态10000对象退休；逐页和单项账本流式核对，不保留整曲事件列表。摘要／单项／64项页的重复读取按实际返回数量计数，没有隐式全图扫描。
- 两档预算重建Q=5000000000的10000对象检查：5000项InspectionSkipped，目标对象5000仍未判定；旧完整帧在准备中保持，新来源激活拒旧页，无假Miss／声音。源码共享而账本独立。
- 受控拒绝覆盖圆圈、两个索引、Rc地图、Rc来源管理、Box运行、运行状态及检查状态8个边界；真实payload与预留交叉核对。另有检查进度耗尽可取消、布局溢出／零大小拒绝、诊断计数失效回归。标准Rc／Box任意真实OOM仍可能终止进程，不宣称通用恢复。

正式python -B tools/check_core.py通过425项Python回归、格式、Clippy、三个核心包无窗口测试及release构建；gameplay为23项单测、20驱动、12帧、16冻结、18检查、26协调、8规模、23合成及3编译拒绝，合计149项。python -B tools/check_desktop.py --architecture-only通过。新增存储源码边界及负向canary约束其他生产模块不能自行引入对象变长分配；原有禁止依赖／时钟／I/O／unsafe规则未放宽，无新包、依赖、feature、锁文件或工程schema变更。

旧失败日志保留在本机忽略目录artifacts/k1-6b/：包括旧夹具对步数的假设、首个存储规则发现旧unit fixture的Vec、Clippy测试排列／循环提示及相应修复。旧unit夹具改用同内容的三元素数组，不豁免它来绕过规则；未修改旧证明或追认失败通过。完整hook只验证备份b3c5f08时的137项基线，不能代替本轮149项验证，也不把远端CI结果套给未推送源码。

收尾补一项先红后绿回归：五对象的已编译大图传入四对象便捷运行构造，且起点也无效时，原实现先读取首记录并报InvalidStart。现将该入口自身容量与运行配置上限一起在首记录／字节分配前检查，明确ObjectLimit；大图仍须RunPreparation。来源／配置的既有错误优先级保持，回归、原始失败及最终正式检查分别留存。

B只证明已声明的变长布局／分步工作及G0无输入轨迹。C的完整固定输入／实际Automatic规模轨迹与规模控制交错、D的耗时／CPU／内存计量仍待后续任务；不以G0替代它们，不标记完整KS／KT23／K1或stable兼容通过。下一步先独立审阅B一致性，暂不自动推进C／D或GUI／原生设备。

收尾检查：10份Markdown的本地链接／锚点、围栏及尾空白检查，git diff --check与789个仓库候选文件扫描均通过。实际新增／修改32文件，暂存区为空；原schema.rs空差异状态标记保留，两个manifest／锁文件及旧证明均未改。本轮B新增源码／测试／文档尚未提交，不将b3c5f08的远端状态套给这些新增量。

## 2026-11-06一致性审阅记录

先核对Git状态、最近提交、空暂存差异及上一轮32个候选文件，再通过正常完整hook创建回退Commit `747222c8c69e39ff39a84e933a73883bcfedb80c`。hook包含149项gameplay及桌面默认／全feature检查。已核对仓库归属、PRIVATE可见性、推送差异与敏感信息，普通推送后远端分支SHA一致。本次源码／测试／说明另为未提交修订，不将备份hook或远端状态套给它们。

### 发现与修复

本次先加入三项公开回归，三项均在原实现上失败，分别修复后通过：

| 边界 | 原偏差 | 修正后的契约 |
| --- | --- | --- |
| 创建／载入／检查激活 | 任务移交运行后，work_counts不再包含运行中的初始化／重建计数，累计回执倒退 | 移交时将运行计数合入任务固定回执；激活前后任务计数相同，后续运行读取不再增加旧任务回执。该回执含移交前运行工作，不能再与同一时段运行计数相加 |
| 无效起点 | 尚未通过运行构造校验，就将Box运行记为一次已尝试分配 | 仅在校验通过、即将进入受控分配边界时记录尝试；边界拒绝仍算尝试，未到边界则不算。独立运行与受管任务共用这一位置，不改请求布局／预留 |
| 无效／旧控制请求 | 协调器为同步内部状态调用公共observation，拒绝请求也使摘要计数增加 | 内部只读固定状态／对象数量，不走公共摘要计数；公共读取仍按成功读取计数，旧游标或拒绝控制不污染当前运行工作回执 |

只抽出私有固定元数据读取及调整计数移交／登记位置，没有新增字段、容器、依赖、公共接口版本或玩法规则；S／R／T及已记录类型布局不变。原始失败保留在本机忽略目录artifacts/k1-6b-consistency/counters-red.log，修复后单独回归为counters-green.log，扩展7项回归为extended-regressions.log。没有删掉失败日志或调整旧黄金业务预期来追认通过。

### 容量、分步工作与所有权复核

逐项核对固定commit的Vec／RawVec／Global实现、storage唯一变长分配入口、检查算术、C=max(4,N)、先预留后申请、无隐式增长、描述符移交和销毁顺序。分配后capacity核对仍只是防漂移canary。该证明只覆盖声明工具链及本机目标的请求布局；不推出物理分配块、Rc控制头、完整栈或任意OOM恢复。没有实际成本测量，也没有修改正式检查器去放宽规则。

新增[scale_consistency.rs](../crates/gameplay/tests/scale_consistency.rs)共7项公开回归，覆盖：

- 空图／Tiny4／Reference10000在创建、载入及检查激活前后的冻结计数回执；无效起点实际未到Box边界时的分配计数；旧游标、错误来源、倒退目标及未知事件确认等拒绝控制的诊断不变。
- 10000对象检查在请求后、初始化中、重建中、Ready四个位置取消。资格立即失效，但保留任务仍持有来源、运行payload及预留；其后恢复、冻结、替换运行，最后Drop旧任务才释放最后旧来源。真实payload与预留分别核对，不以撤销请求冒充资源释放。
- 原完整T=100000，推进至100001只提交第一个Miss时，实时账本为1而旧完整帧仍为0；取消推进及随后LateInput失败都保留旧完整截面。页返回旧时刻事实，错误页不改输出、分配或成功读取计数，末页位置N成功。恢复／替换后即使修订及其他代次数值相同，也因完整来源不同拒绝旧页。
- 5／31／32／33／65／10000记录、预算1／256下的非整齐归并及多个重复组，仍按最早原记录位置与既有错误优先级报告，不能按ID排序后的先后选错。
- 独立万对象来源准备的字节恰够／差1、独立运行限制恰够／差1、错误预算无工作及无容量分配；状态分配单列一步，随后每次只初始化一项，完成不生成伪完整帧。独立作用域不借用协调池额度。

既有8项规模回归全部重跑，包括Reference10000／Tiny第5项拒绝、两池恰够／差1、受控拒绝与取消、G0两档预算各10000个Miss／连续事件／无SoundIntent、末态全退休及万对象检查。149项原有gameplay回归保持，加入7项后共156项；固定输入／实际Automatic完整规模与其全部控制交错仍留给C，本次局部所有权／旧页回归不替代C。

### 检查与停靠点

python -B tools/check_core.py通过425项Python回归、fmt、Clippy、三个核心包无窗口测试和release构建；gameplay为23单测、20驱动、12帧、16冻结、18检查、26协调、8规模、7本次一致性、23合成及3编译拒绝，共156项。python -B tools/check_desktop.py --architecture-only通过。日志分别为artifacts/k1-6b-consistency/core-final.log及desktop-architecture-final.log；备份完整hook另存backup-hook.log。

本次B适用范围的一致性审阅及已发现问题修复完成，保持API7／profile1、Reference10000／Tiny4、64槽队列和原预算。不自动开始C／D、GUI／原生设备或stable规则，不宣称完整K1／KS／KT或stable兼容通过。本次修订留作下一轮检查备份。

收尾：7份Markdown的本地链接／锚点、围栏及尾空白检查通过，git diff --check与790个仓库候选文件扫描通过。实际改动13文件（5源码、1新增回归文件、7文档），暂存区为空。根／桌面manifest与锁文件未变；保留schema.rs原有空差异状态，规范化内容与HEAD一致。未修改既有证明JSON、引入外部客户端／素材依赖或将本机忽略日志纳入候选。
