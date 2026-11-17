# P3-2B1-1：健康既有文件的实验性只读事实

2026-11-17。本增量完成FileFactsPort1纯值、FileCoord-Proof-v1字段／独立向量和Windows健康文件的内部只读验证。**只按B1-1实验增量交付，完整B1、PB16及生产共同profile资格仍未签收**；q与文件对照、Unknown四类协调安装留B1-2。没有B2写端口、真实产品保存、恢复根部署、UI／设备或皮肤Skill接线。

用户已确认[可调整默认政策](P3_2B_START_CONTRACT.md#9-已确认的可调整默认政策)：系统用户持久恢复子目录可迁移、初始4GiB／32条目可调整且满额明显警告／拒绝新增、同工程完整历史连续命名并保留旧恢复副本。首次命名由用户选位，覆盖与清理协议另约；本轮没有覆盖现有文件，也没有把政策调整解释为Codex权限或UAC变更。产品设置入口与目录迁移并未由本片实现。

## 1. 回退点、版本和实际入口

轮初HEAD／PUBLIC origin／codex/p1-bootstrap为00492b1972ba1d02b1c986e6f222fdd4c325f0cf，[同SHA三项CI](https://github.com/sumiyuki727/skin-editor/actions/runs/43967388402)成功；暂存和实际内容差异为空，只保留schema.rs既存标记，复用回退点而不造空提交。旧CI不验证本轮新代码。

[application纯值与窄端口](../crates/application/src/file_facts.rs)定义版本化请求、观察身份、固定16槽页／128提交上限、有限错误和ExperimentalValidation摘要等级。[infra入口](../desktop/infrastructure/src/project_storage/file_facts.rs)为FileFactsReader::open、inspect、pending_cleanup、retry_cleanup及usage；返回对象只实现FileFactsPort，不返回RestoreCandidate、可写SqliteProjectFile、ProjectSession、资源池或保存authority。

FileFactsPort1／FileCoord-Proof1独立版本化；现有API7／捕获3／协调1／旧存储1不变。API8／协调2保留给实际接线，未被提前声称实现。原文件profile、DDL、request_hash、全部持久记录版本及依赖不变；application仍只依赖domain，未引入第三方核心依赖或原生句柄。

## 2. 设计门槛和完整证据

[开工附录](P3_2B1_FILE_FACTS.md)先冻结了接口、规范编码、Windows路径／句柄策略与预算欠项，再由两路独立审阅同意开始内部B1-1。初轮封套向量被审阅指出不足，已明确分为framing、完整合法小文件及单独metadata空值／扩展编码三类；不把两项任意payload当成完整五域证明。

[向量](P3_2B1_PROOF_VECTORS.json)来自自制4节点、Manual400／Exit401、17字节真实原始资源和含CRLF／非ASCII配置的健康文件。测试准备使用既有writer，独立Python sqlite3／hashlib按规范逐字段组装输入与摘要；新proof实现不产生自己的预期值，运行测试不调用生成器。五个完整文件域分别有54／3／33／25／7个TLV字段；五个framing及一个metadata变体也分别核对。额外扩展／None变体只证明序列化，不冒充另一个已验证文件。

[proof实现](../desktop/infrastructure/src/project_storage/file_facts/proof.rs)绑定全体节点原记录与当前canonical、admissions、经实际字节校验的全部资源、完整metadata、各历史提交的旧request_hash／导航／链以及打开基线和推导Redo。runtime会话／绑定／路径不进入持久证明。FileFactsProof是纯摘要值，不授信；新增compile-fail拒绝转换成PreparedWitness。B1-2还须规范化原q的新节点首写记录，不能直接把五个file hash填成写见证或解除Unknown。

## 3. 源保全、结果寿命和清理

[snapshot](../desktop/infrastructure/src/project_storage/file_facts/snapshot.rs)只接受明确的Windows本机绝对路径，限制长度／组件，拒绝网络、父级穿越及重解析。父目录以OPEN_REPARSE_POINT／BACKUP_SEMANTICS打开并从句柄核属性，保留不共享删除的守卫；主源只读share_mode(0)，所有sidecar（含空文件）拒绝。未验证平台返回Unsupported，不把普通复制当安全冻结。

scratch目录用可枚举的目录守卫与自有独占RootLock，避免目录share_mode(0)阻挡自身枚举。仅在显式空专用根create_new物理副本；SQLite只以READ_ONLY接触副本。内部完整验证仍检查schema／链、全部分支与状态、原字节、admissions／地址／元数据；私有BlobStore、恢复候选及缓存释放后，仅返回固定结果和仍冻结的源守卫。摘要副本不保有守卫，不能冒充永久有效的注册绑定。

每次观察有Reader随机非零instance及checked递增sequence，完整分页上下文另核对ObservationId、key和完整SaveTargetBinding。相同调用者请求复用也不能接受旧观察页；sequence最大值一次可用，随后拒绝，不回绕。同Reader的全部仍存活观察共用512KiB新包装额度，Reader先退出也不提前解除观察持有的源锁。

Scratch创建成功即进入Reader固定残留槽，实际文件身份随后在复制前保管。清理失败不退自有主副本保留、不接纳新inspect；重试和Reader退出前比较实际文件身份，同名替换文件不会被删除。未取得身份时也保留CleanupPending。Reader退出仍清不掉时保留文件；新Reader拒绝非空／未知根，不默默重建账本或自动清理孤儿。此处不是B3持久恢复管理器。

## 4. 实际回归与审查发现

[Windows专项](../desktop/infrastructure/src/project_storage/tests/file_facts.rs)18项定向回归通过；[结构化审阅证据](P3_2B1_REVIEW_PROOF.json)列出准确测试名、源码散列与日志摘要。application另有固定载荷边界测试与不可转写见证的compile-fail，随正常核心检查执行。非Windows适配器的Unsupported分支有源码／测试定义，本轮没有Linux桌面执行证据；Linux核心CI不能冒充该适配器实测。

| 证据组 | 实际观察／限制 |
| --- | --- |
| 独立规范向量 | 五域封套、完整文件五域及实际metadata None／extension编码对照；同拼接字节／不同字段边界不能相互替换 |
| 健康完整读 | 4节点／真实原字节、Manual A与较新保护B；重新核对完整源字节，结果无写资格，scratch只留本上下文锁标记 |
| 分叉与导航 | 真正5节点分支，Manual403、Exit404；期望推导Redo明确为1006→1007，保持活动Session不变 |
| 失败封闭 | 非选中分支校验和、原字节和提交请求摘要损坏均拒绝；Manual缺位只能显式Latest，不能伪造Manual |
| 路径与冻结 | 空／非空journal/wal/shm、写／删除／父目录rename、真实自有junction、长度／组件／相对／网络路径边界；只读观察退出后才解锁 |
| 上下文与容量 | 版本／零序号／expected拒绝，旧观察／完整绑定不匹配页拒绝；实际文件、自有副本、新包装及原字节17／16边界；双观察聚合N／N−1；没有宣称32MiB满额或所有最大值组合通过 |
| 取消与清理 | 预取消、复制后及完整验证内部取消；清理失败保留、实际重试退额、替换路径保护；测试没有冒充复制中途取消的执行证据 |

独立审阅发现并修复：①目录独占导致自身枚举Io；②总原字节额度收紧时固定64KiB单份限额误拒绝；③失败后只按scratch路径删除可能误删替换文件。另补齐观察身份／完整分页绑定、失败取得身份的残留所有权与Drop顺序。初轮失败、夹具把NoChange错当新提交的失败及Clippy失败日志均保留。首次完整hook在核心／canary通过后，发现测试的编译期文件包含违反桌面源码边界，故阻止提交；改为测试运行时读取仓库内自制JSON夹具，保持检查规则不变并重跑。修订后18项重新通过，没有覆盖失败记录。两路最终只读复核均未发现剩余B1-1实验范围阻断，构建由主线独占，没有重型检查叠加。

## 5. 仍未取得的资格

本片只完成健康文件的内部事实观察。尚未完成原q与完整目标注册绑定对照、NoChange自动确认、四类Unknown在应用协调中的安装、生产authority及B1整体退出。调用者opaque target／binding／session只是本次观察关联，不能当作已验证产品注册关系。

新包装载荷与自有主副本账本不涵盖旧full验证器的Cache／JSON／所有恢复状态／codec临时及SQLite聚合分配；64MiB受管总量仍未证明。temp_store=FILE可能产生的SQLite临时磁盘也未合入自有主副本32MiB计量。Small完整编辑／准备共同准入、其他VFS身份语义、实际RSS／性能及断电保证均未取得；保留ExperimentalValidation，不签生产共同profile或PB16。一次只读文件系统类型查询因权限拒绝而未取得信息，未为此提权或请求UAC，不影响本轮明确自制路径行为回归，也不据此声称更广平台保证。

U1目录部署／迁移、U2配额／明显警告的产品入口、U3用户选位／覆盖安全协议、恢复发现与转交分别待B3／B4及相应宿主／UI切片。没有对用户真实工程或恢复材料运行本入口。K1、Reference10000／Tiny4、产品性能NotEvaluated、stable兼容Unknown保持，不作正式成本测量。

## 6. 提交与保管

源码、向量、政策和同步文档经明确路径审查、正常完整hook提交并推送既定PUBLIC origin；最终接受以包含本报告的提交及同完整SHA三项CI成功为门槛，不套用00492b1旧结果。旧测量／冻结程序／PRIVATE恢复附件和密钥保管不改；schema.rs原字节与既存空差异标记保持，其SHA256为713f3850740db024c4b861327de560f4eb81f5820980d1f89672dcdcfd499125。

本轮844文件基线、设计fixture原件／临时producer、原始测试与失败日志、暂存复核及回执位于本机忽略目录artifacts/p3-2b1-20261117。设计producer仅用于自制fixture，临时example已移出源码树，不纳入产品或提交。Git恢复包含提交源码／小型独立向量，新增忽略日志不自动属于[3435405私有历史快照](PROJECT_RECOVERY.md)；不把源码推送说成全部本机artifacts已异地备份。
