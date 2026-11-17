# P3-2B1：文件侧只读事实接口与证明规范

2026-11-17，B1-1开工附录。用户已确认可调整的U1系统持久恢复目录、U2初始4GiB／32条目及U3同工程连续命名，详见[总契约](P3_2B_START_CONTRACT.md#9-已确认的可调整默认政策)。本片只读取明确指定测试根内的自制健康ExistingVerified工程，不部署恢复根，不保存／发布工程，不签发MayWrite，不替换活动会话。

## 1. 切片及资格

B1细分为B1-1文件侧完整验证与只读事实、B1-2捕获q与文件证据对照／四类观察和共同预算资格。独立FileFactsPort1是新增只读端口，现有API7／捕获3／协调1／旧存储1的载荷及行为不变；API8／协调2保留给实际协调接线。这是实施顺序细化，未实现拟议的新版本。

B1-1的产物是**内部实验性文件观察**，不是可安装的生产保存authority，也不是B1整体签收。FileFactsProof与PreparedWitness不同类型，无From／裸值转换。观察对象只提供摘要与固定提交分页，不返回SqliteProjectFile、RestoreCandidate、资源池、可变项目或完成发送者。FileFactsPort只有只读方法，不存在写许可或恢复入口。B1-2仍须核对原q、完整Ticket和同一次目标观察；仅有文件hash不能确认任意q、NoChange或解除Unknown。

## 2. 版本化值和接口

application定义以下纯值及只读trait，infrastructure实现；路径仅传给infra显式实验入口，不进入公共请求／摘要。

| 值／方法 | 冻结语义 |
| --- | --- |
| FILE_FACTS_PORT_VERSION=1 | 请求和分页版本独立于应用API |
| FILE_COORD_PROOF_VERSION=1 | 证明字节编码；独立于持久文件request_hash |
| FileFactsKey | 非零运行会话、非零调用序号；与目标绑定完整保存在观察上下文，不持久化 |
| FileObservationId | infra新Reader签发的非零随机instance及checked递增观察sequence；生成失败／零值拒绝，不归一化；摘要和分页额外绑定此身份，同调用者key复用也不接受旧页。随机命名空间不声称数学上消除任意碰撞，不使用全局可变状态 |
| FileFactsRequest | version、key、完整SaveTargetBinding、Manual／Latest；只接受ExistingVerified及完整expected提交／代次和history范围 |
| FileFactsSummary | 原请求关联、最新／Manual／保护、实际所选头、是否存在Manual、节点／准入／原字节／提交计数、五域文件proof；不声称所选状态已激活 |
| FileCommitFacts | 完整四字段事实＋previous、旧request_hash、navigation root、prefix、node_count／next_alias／next_branch；旧hash只作该具体提交映射 |
| FileFactsPageRequest | version、ObservationId、完整key／binding、offset、limit，limit为1..16；页固定于同一观察对象，旧上下文拒绝 |
| FileFactsPage | 固定16槽＋length／next，空页与版本／上下文／范围错误不同；不分配Vec或截断冒充完整结果 |
| FileFactsPort | summary和commits只读查询；无原件路径、ProjectSession或写能力 |

返回值不自动解决Unknown：找到CommitId也须由B1-2比较完整持久请求；None仍是“未观察到该ID”，不证明未提交。Manual缺位的普通Manual观察返回NeedsExplicitRecovery，Latest可完整读取但不制造Manual。绑定expected不匹配返回TargetConflict；对已确认事实的后续冲突另锁绑定，不能回退事实。

公共签名冻结为：

```rust
pub trait FileFactsPort {
    fn summary(&self) -> FileFactsSummary;
    fn commits(&self, request: FileFactsPageRequest) -> Result<FileFactsPage, FileFactsError>;
}
// infrastructure显式实验上下文，路径不进入application值：
impl FileFactsReader {
    pub fn open(scratch_root: &Path, limits: FileFactsLimits) -> Result<Self, FileFactsError>;
    pub fn inspect(&mut self, source: &Path, request: FileFactsRequest,
        cancel: CaptureCancellation) -> Result<InspectedFileFacts, FileFactsError>;
    pub fn pending_cleanup(&self) -> Option<u128>;
    pub fn retry_cleanup(&mut self) -> Result<(), FileFactsError>;
}
```

FileFactsError为有限枚举：UnsupportedVersion、InvalidRequest、InvalidLimits、Unsupported、Busy、Cancelled、IdentityUnavailable、TargetConflict、NeedsExplicitRecovery、ValidationFailed、Io、Limit(有限资源枚举)、InvalidPage、StalePage、Cleanup{token:u128}。不得带任意String／路径／堆栈。FileFactsSummary明确携带ExperimentalValidation等级；它及纯proof摘要可被值复制，但只有私有构造的InspectedFileFacts持有稳定源所有权，摘要副本不能声称目标仍冻结。端口实现者是受信适配器，trait不是抗恶意实现的安全边界；B1-1不注册可持久复用的物理目标绑定。

FileFactsReader持有一个固定CleanupPending槽及scratch根资格。清理失败时，该槽继续持有自有残留身份和磁盘保留，返回token不退额；新的inspect被拒绝，只有显式retry_cleanup实际成功才清槽。Reader退出可再尝试一次清理；若仍失败，保留残留，不伪称释放磁盘。新的Reader只接纳空的专用scratch根，非空则拒绝，不把未登记残留当免费空间，不自动清理它；这不是B3的跨进程恢复账本。B1-1的disk预算只在该实验上下文内有意义，不能当作产品恢复总配额已实现。

## 3. 稳定只读策略

受支持范围限定Windows本机绝对磁盘路径；拒绝网络路径、特殊设备、相对／父级穿越、重解析组件，限制为1024个UTF-16代码单元／最多16个普通组件。源及scratch根均由调用者显式指定，不探测真实用户工程或默认恢复目录。按根到叶顺序以GENERIC_READ打开父目录，FILE_FLAG_BACKUP_SEMANTICS(0x02000000)与FILE_FLAG_OPEN_REPARSE_POINT(0x00200000)，share_mode(READ|WRITE=3)而不共享删除；从已打开句柄再核对目录类型及非reparse属性。scratch目录同样共享读取以允许自身枚举，空根内create_new一个自有file-facts-reader.lock并以share_mode(0)保管；同根第二Reader拒绝，遗留未知标记不自动删除。源主文件GENERIC_READ／share_mode(0)／OPEN_REPARSE_POINT，句柄属性核实为普通非reparse文件。守卫持续保留，Busy不等待／不强夺；目录rename／替换行为由Windows回归证明，不仅凭flags宣称能阻止任意恶意重解析修改。其他平台返回Unsupported，不能用普通复制假装安全冻结。

全部journal／wal／shm均须不存在，零长也拒绝。主文件健康头、物理身份、长度和完整SHA先验证，在源冻结期间向显式scratch根中唯一create_new的自有文件物理复制；核对复制完整SHA，不能使用硬链接。scratch写句柄允许独立SQLite只读读取并阻挡其他写／删除；SQLite仅以READ_ONLY接触这个自有副本，无RW重开、回滚恢复或正式发布。

复用完整read验证器：schema／链／全部分支原记录及状态、所有原字节、索引／admissions／地址和领域不变量一并校验。使用独立临时BlobStore和内部RestoreCandidate，避免音频metadata证明影响活动资源池；结果提取后全部释放，不向外返回激活候选。再次核对源和副本摘要、sidecar及物理身份，关闭SQLite后清理自己的scratch；清理失败返回有界token及Cleanup错误，不能假成功，也不删除其他文件。最终只读观察持续持有源／父目录守卫，直到对象Drop；这不是允许释放守卫后继续使用的永久目标绑定。

Scratch创建成功后立即归Reader固定槽保管，再取得并持续持有实际文件身份句柄；身份取得失败保持CleanupPending，不能退额或仅凭名字删除。重试／Reader退出清理前核对同一物理文件，路径被替换或原件移走均保留诊断、额度与文件，不删除替换物。身份比较使用现有same-file能力，验证范围限本轮受测本机文件行为；其他VFS／ReFS身份语义、恶意管理员篡改与持久恢复账本未取得保证。

这是受控本机路径与非恶意同用户环境的源保全协议，不是阻止管理员／恶意进程篡改的安全沙箱；副本目录须是调用者控制的专用scratch根。任何无法验证的身份／路径条件拒绝，不能放宽为成功。SQLite读取可能涉及热日志的依据见[官方锁与热日志说明](https://www.sqlite.org/lockingv3.html)；本片拒绝源sidecar，不调用原件SQLite连接。

## 4. FileCoord-Proof-v1规范

每域SHA-256输入为UTF-8/ASCII字节`skin-editor/FileCoord-Proof/v1`后接零字节，再接域编号u16大端；之后为按本节固定顺序的TLV字段：tag:u16大端、payload长度:u64大端、原始payload。tag为该域从1开始递增的字段序号，按固定遍历顺序分配，溢出拒绝；整数payload使用固定u64／u128大端；可选身份为单字节0或1＋固定身份。枚举使用规范数值，不使用Debug／Rust内存。每个序列先写count:u64，再按列明顺序编码item；重复主键由完整验证器拒绝。字段名只是规范标签，不能将同一旧hash复制到五域。

| 域编号 | 输入字段及顺序 |
| --- | --- |
| 1 content | 工程／history／lineage／address_epoch／root；按alias顺序的全部节点，逐项id、parent、alias、branch／index、depth、原记录版本／state／action／checksum、当前canonical根；按ObjectId种类／id排序的admissions及first_node |
| 2 originals | 按BlobRef排序的全部资源：SHA-256及实际byte_len；实际字节已由完整读取／SHA核实，类型／资源元数据由content状态根绑定；不能把未读引用当原字节证明 |
| 3 metadata | 最新／Manual／保护完整事实，按代次顺序的Manual marks；按node/name排序的tags，按name排序的extensions原字节；受验证features按name／version／required三个TLV字段编码；不包含运行epoch／路径 |
| 4 request | 按文件代次顺序的全部提交：commit、generation、origin、head、previous、旧request_hash、navigation root、prefix、node_count、next_alias、next_branch、按parent排序的原Redo。它证明这些历史持久请求，不能代替尚未写入的q |
| 5 baseline | content／originals／metadata／request四域hash、Manual／Latest选择、所选头与加载推导后的Redo（按parent排序）；默认Manual A与较新保护C分别保留 |

精确字段流如下，每个逗号项是一项TLV、递增一个tag；括号为遍历控制，不编码额外容器标记。`u64/u128`为定长大端，`bytes/text`分别为原字节／UTF-8（不规范化换行或Unicode），`hash`为32字节。`option-id`在同一个payload内编码0或1＋16字节身份；计数均为u64。SaveOrigin为Manual=1、Automatic=2、Exit=3；selection为Manual=1、Latest=2。

```text
receipt = commit:u128, generation:u64, origin:u64, head:u128
optional-receipt = present:u64 (0结束；1后接receipt)
content = project:u128, history:u128, lineage:u128, epoch:u128, root:u128,
  nodes.count (alias升序：id:u128,parent:option-id,alias:u64,branch:u64,
    ordinal:u64,depth:u64,record_version:u64,state:hash,action:bytes,
    checksum:hash,canonical:hash),
  admissions.count (kind按ASCII字节/id升序：kind:text,id:u128,first:u128)
originals = count (sha字节升序：sha:hash,bytes:u64)
metadata = optional-receipt(latest),optional-receipt(manual),optional-receipt(protected),
  marks.count (generation顺序：commit:u128,node:u128),
  tags.count (node/name升序：node:u128,name:text,value:text),
  extensions.count (name升序：name:text,bytes:bytes),
  features.count (name/version升序：name:text,version:text,required:u64(0或1))
request = commits.count (generation升序：receipt,previous:option-id,
  legacy_request:hash,navigation:hash,prefix:hash,nodes:u64,next_alias:u64,
  next_branch:u64,redo.count (parent升序：parent:u128,child:u128))
baseline = content:hash,originals:hash,metadata:hash,request:hash,
  selection:u64,selected_head:u128,derived_redo.count (parent升序：parent:u128,child:u128)
```

features不序列化任意JSON，而按上表固定三字段编码；种类字符串与既有受验证ObjectId种类对应。TLV封套向量及完整小文件字段向量分别登记在[独立测试向量](P3_2B1_PROOF_VECTORS.json)，不能用前者替代后者。B1-2须定义q的同域规范化：origin=None的新节点先形成预期首写记录，排除changed_objects等运行差异；不得直接比较未规范化的None／Some或向文件写入运行会话。

## 5. 预算设计和当前资格边界

读取单项上限参考FileCoord-Small-v1：32MiB文件、8192块、64节点／256admissions／8原始资源／总256KiB单份64KiB、128提交、metadata1MiB、restore逻辑16MiB、累计状态闭包8MiB、受限codec。这不是Small完整编辑／准备共同准入的实现。自有主副本磁盘额度独立保留，复制scratch固定64KiB；结果使用固定128提交槽和固定16分页槽。新增持有者、父目录固定句柄槽及受限路径capacity、结果静态布局分别核对，取消／Drop及cleanup按真实所有权处理。Reader锁文件是本上下文自有控制标记，不是已部署的恢复Entry。

FileFactsLimits只允许收紧上述上限，零值和超上限在文件工作前拒绝；scratch磁盘最多32MiB，提交数最多128，结果／路径包装另用同一Reader合计最多512KiB的独立受管额度，涵盖该Reader所有仍存活观察对象，不为每次inspect重置，不从A控制分区借用。新包装按静态结果布局及目录／路径容器capacity计费，最后所有者退出才退；已有完整验证器分配的欠项继续按下一段公开，不能把“新包装受管”写成验证器整体受管。B1-1不提供无限Reader的聚合产品预算，宿主并发准入仍待后片。调用者给定的opaque binding.id／target／session只是本次观察关联，B1-1只能实际核对文件project／history／expected以及受守卫保护的物理源；产品注册绑定和原q仍属B1-2。

**现有full验证器的Cache、JSON、全部恢复模型、codec临时及SQLite分配仍缺64MiB受管聚合证明。** 文件长度／Restore逻辑量不能冒充这一证明；复用configure的temp_store=FILE也可能产生SQLite临时磁盘，占用尚未合入自有主副本32MiB账本，不能宣称全部验证磁盘被该数字涵盖。本片只记录逻辑边界、新包装载荷和资源释放，不签发“共同profile生产合格”资格，不登记PB16或完整B1通过；生产保存／只读协调接线保持不可用。B1-2取得逐分配预留／capacity、SQLite工作区及共同manifest，或另审更窄的可证明实现后才可启用生产资格。系统页缓存和实际RSS另列，不测成本、不升级产品性能。

## 6. 设计审阅与最小回归

在实现前独立复核端口能力、五域映射、稳定源／副本及预算未证明部分；后续真实回归只使用自制显式测试根。最低覆盖：版本／完整绑定拒绝；Manual缺位与Latest；全分支／原字节／推导Redo；非当前分支损坏；提交链及原字节损坏；空／非空sidecar、锁冲突、路径重解析；取消与限额差一单位；源与sidecar原字节不变、对象持有时仍锁定、Drop后可重新打开；固定分页／旧上下文拒绝；独立五域向量和同数量变异；自有scratch清理及独立资源释放。缺证据仍为错误／Unknown，不能缩减验证换取通过。

本附录描述B1-1内部增量，不能据此签收B1-2、B2写入、恢复发现／命名交接、产品保存／关闭／重开或stable兼容。准确实现、测试与剩余门槛在后续本轮审阅记录中分别登记，旧CI不能充新验证。

设计门槛复核：主线补齐完整字段流与独立完整文件向量、ObservationId、固定CleanupPending及路径flags后，两路只读审阅同意开始限定的B1-1。实现阶段按真实OS回归将目录独占修为自有RootLock，补清理实际文件身份与原字节收紧边界，并完成第二轮独立代码／测试复核。具体源码、测试和未覆盖范围见[本轮实现审阅](P3_2B1_REVIEW.md)；本段不提升共同预算或生产资格。
