# P3-1B 审阅：完整历史捕获、永久身份与原始资源租约

2026-11-02。本轮已实现 application 管理的固定截面捕获：保留全部历史节点／分支，流式产生各节点的完整内容状态，并独立固定其原始资源。**尚未接入正式保存／打开、Rust SQLite 存储器、自动保护、标签编辑或皮肤导出。** 普通桌面仍是内存编辑器。前一增量的[状态 codec](P3_1_STATE_REVIEW.md)保持可用。

本轮开始先正常运行 Git hook 保存 P3-1A，回退 Commit 为 `6972c55`；之后的本轮修改不在轮末自动提交，不推送。原 P3-0／P2 证明、黄金向量、归档和冻结程序保留原字节，新证据另见 [P3_1_CAPTURE_PROOF.json](P3_1_CAPTURE_PROOF.json)。

## 捕获了什么

| 部分 | 当前实现 |
| --- | --- |
| 历史内容 | 根、全部分支／节点、直接父关系、各节点完整 SkinProject 状态；不是仅取当前头祖先，也不合并旧节点 |
| 永久身份 | 独立的 PersistentHistoryId／PersistentNodeId／LineageId／AddressEpoch，均非零 128 位；与运行会话及局部 u64 NodeId 分离 |
| 地址 | 捕获局部别名／分支坐标及其映射代次；k 短键最多 26 字符，hp1 永久引用固定 102 字符；不复制完整祖先路径 |
| 操作描述 | 显式版本 1 的不可执行摘要：Root／Command／Draft 来源、稳定步骤名称序列及改变对象；最终参数／内容在节点状态，不序列化旧 Command 或私有 Patch |
| 身份准入 | 全部已消费对象 ID 及首次提交节点，包含已删除对象、有效事务中创建后又删除的对象；不从当前头重新推测 |
| 导航 | 捕获头、普通 Redo 的分叉偏好、局部下一个别名／分支计数；原 UI／命令地址协议仍为 API6 |
| 原始资源 | 全部历史状态所需的去重 BlobRef 和独立租约，包括非当前分支、未使用／不支持解码的素材；不固定 PCM／RGBA／Voice |
| 运行关联 | 源 HistorySessionId、ExpectedState 和导航修订供回执核对；这是运行时关联，不是文件中应恢复的打开 epoch 或请求回执 |

当前从未有真实文件保存，因此没有保存检查点／保存事实可捕获，也不伪造“已保存”标签。根／当前／分叉／末端可从完整拓扑及头求得；自定义标签、真实保存标记及持久元数据修改用例仍待后续。原有回执上限策略不变，不把回执／临时 UI 状态装入历史文件。

操作摘要用于说明一次原子提交的步骤，不保存中间草稿／每次鼠标移动或用于重放的命令参数。以后存储须固定节点首写的原始操作 JSON 和状态摘要，显示文案、语言或新编码器不能改写已有节点；未知旧描述按格式规则保留，不能靠执行旧命令恢复。当前尚未生成文件节点 checksum，原始记录绑定由 P3-1C 存储适配完成。

## 捕获与后台数据流

```text
ProjectSession.capture_history(request, identity_source)
  → 校验来源／草稿 Busy／准入，冻结拓扑及共享不可变补丁
  → 捕获身份、准入、导航，克隆已验证原始字节租约
  → HistoryCapture（不持有 ProjectSession／ResourceStore，可移动到 worker）
  → poll(有界边数) → CapturedNode（完整只读项目值）／Pending
  → 全部节点完成，CaptureSnapshot.is_complete() 才变为 true
  → 后续存储器仍须编码、校验闭包并真实提交；本轮没有这些写入
```

capture_history 须核对版本、会话和基础修订；活动草稿返回 Busy。捕获不提交命令、不移动头、不清理旧分支、不设置保存标签。成功接纳后，UI 仍可经原入口编辑、导航或开启新草稿，已冻结截面不随之改变。

内部 Patch 改为 Arc 共享不可变载荷；子索引复制为固定拓扑。遍历使用显式栈：从根进入子节点应用一次前向补丁，退出时应用一次反向补丁，**每条边最多各走一次**。完整状态验证及编码仍按各状态内容付费，不承诺整体 O(节点数) 字节成本；避免的是每个节点重新从根回放的二次方祖先遍历。Patch 仍私有，适配器只得到验证后的 SkinProject。

poll 每次最多接纳 1–1024 次边转换；深分支回退可返回 Pending，便于协作式调度。它不是实时音频回调，也没有启动新的线程。每个输出节点按其永久 ID／固定 metadata 索引关联，DFS 输出顺序可与追加顺序不同，父节点仍先于子节点；manifest 保留追加顺序。

捕获开始需要遍历全部历史元数据并建立有界索引，当前仍由调用它的所有者执行；本轮未接 GUI 后台调度，也未实测此冻结步骤的 GUI 延迟。不能把有界 poll 当作整个开始步骤恒定耗时。

## 身份分配与兼容边界

application 定义 HistoryIdentitySource，infrastructure 的 RandomHistoryIdentities 通过 OS 熵提供候选；核心不获取随机数、时间或平台资源。application 拒绝零值、碰撞和来源失败，不靠碰撞概率省略检查。首次成功接纳捕获时分配历史／谱系／映射代次及各节点永久身份；后续只为新增节点分配，既有节点不改号，普通追加不改变 AddressEpoch。

身份候选在临时映射中准备，失败／开始前取消不发布部分映射。成功接纳后再取消遍历，已经分配的永久身份仍保留，不能因为一次取消给同一节点重新编号；这不是项目内容修改或历史节点。资源、节点及状态不因身份分配失败而改变。

对象 ID 的首次准入由发布时记录，不借用运行 RequestId。局部 n／b 与永久 hp1／k 分开，捕获只携带当前映射，没有加入显示地址重分配入口。新类型／捕获端口独立使用 HISTORY_CAPTURE_VERSION=1，模型2／application API6不偷偷升级。

**目前的稳定性验证覆盖同一会话内反复捕获、分叉、取消及后续编辑。** 跨关闭重开的身份恢复、文件复制后的分歧、旧 epoch 拒绝等仍须 P3-1C 的完整记录／资源校验与受验证恢复用例；本轮没有让 ProjectSession 接受任意外部 manifest 或整项目替换。

## 准入、取消与实际释放

| CaptureLimits | 默认值 |
| --- | --- |
| 节点数 | 65536 |
| 已消费对象 ID | 262144 |
| 去重原始资源数／字节 | 4096／128 MiB |
| 保留元数据逻辑成本 | 32 MiB |
| 单节点完整状态逻辑成本 | 8 MiB |
| 累计产生的状态逻辑成本 | 512 MiB |

每会话当前只允许一份在途捕获。开始前按节点／身份／原字节及保守元数据成本预检，再复制冻结结构与租约；原字节从已有历史 pins 克隆所有权，不重新读文件、解码或分配媒体副本。单状态和累计展开上限在生成完整输出前检查。成本含固定对象、文字、树和共享补丁保留的保守估计；共享树可能重复计费。这是逻辑准入，**不是 allocator／进程 RSS 或私有提交的实测上限**。原字节实际分配仍由 BlobStore 的共享 ByteBudget 管理。

直接 cancel 幂等且不占回执／任务额度；外部取消令牌由下一次 poll／释放确认。取消／终止错误丢弃工作状态、拓扑和补丁引用，已经交给消费者的 CapturedNode／CaptureSnapshot 继续保有自己的租约。只有最后一个所有者释放后，实际字节额度和在途捕获槽才归还；仅设置取消标志不等于已经释放。捕获完成也不强制撤销消费者所有权。

取消或部分输出的 snapshot 不会变成 complete。complete 仅证明遍历完整，不代表消费者已保存全部状态或文件已提交；后续存储仍须核对所有节点／资源闭包、取消状态和真实回执，不能拿这一标志直接打保存标签。

超限、身份失败、原始租约缺失／不匹配、拓扑损坏及状态校验失败均明确返回错误，不删节点修复，不回收旧历史资源，也不发布半个活动项目。所有 captures 都不修改项目；永久身份准入是唯一单独持有的元数据变化。

## 新对象、方法与依赖

本表补充[原代码快照](review/README.md)；没有类继承，只有组合与 trait 实现。

| 源码／对象 | 方法与关系 |
| --- | --- |
| [capture.rs](../crates/application/src/capture.rs) · ProjectSession | capture_history、私有 capture_identities；唯一所有者冻结当前历史，不开放写入／加载旁路 |
| 同文件 · HistoryCapture | snapshot／statistics／poll／cancel；私有 poll_once／emit／abort；拥有固定遍历计划、工作定义和取消状态 |
| 同文件 · CaptureSnapshot／Resources／Permit | manifest／is_complete；共享只读 metadata、原字节 pins 和捕获准入租约；Permit::drop 最后归还槽 |
| [capture_contract.rs](../crates/application/src/capture_contract.rs) | CaptureRequest／Source／Limits／Manifest、CapturedNodeInfo／Admission／Operation、CaptureError／Statistics／Poll；CapturedNode.project／info／snapshot，CaptureCancellation.cancel／is_cancelled |
| [capture_identity.rs](../crates/application/src/capture_identity.rs) | 四种非零128位ID的new／get；PersistentNodeId.short_key／parse_short_key、PersistentNodeReference.text／parse；HistoryIdentitySource.next_identity，私有PersistenceMap |
| [capture_operation.rs](../crates/application/src/capture_operation.rs) | 显式 Command→稳定步骤名映射，Root／Command／Draft来源；不序列化或重放Command |
| [capture_cost.rs](../crates/application/src/capture_cost.rs) | Cost及asset／document／sound／components／configuration／outputs计量，project／definition成本；无平台计时／内存读取 |
| [history.rs](../crates/application/src/history.rs) | Node组合Arc补丁、操作摘要和首次准入；Patch.originals／capture_cost及私有应用；append_record仍仅在成功发布后调用 |
| [project_identity.rs](../desktop/infrastructure/src/project_identity.rs) | RandomHistoryIdentities实现HistoryIdentitySource，使用getrandom `=0.4.3`，每次提供16字节候选 |

项目内依赖方向不变。getrandom 已是桌面锁文件／宿主依赖，本轮仅给 infrastructure 增加同版本直接边和受审查规则，没有新增包版本、根核心依赖、线程或设备。平台范围是该库的 OS 熵后端；本机 Windows 编译检查，Linux／macOS 未实测；确定性测试注入受控身份源，不以随机调用验证碰撞行为。首次捕获需要节点数＋3次候选调用，暖捕获只请求新增节点；OS 熵路线的实际延迟尚未测量。

## 验证及后续

核心检查及桌面默认／全部 feature 无窗口检查通过，格式、Clippy 和边界负向回归随改动执行；初次测试代码的 Clippy 失败保留并已修正。最终源码与日志摘要见[证明索引](P3_1_CAPTURE_PROOF.json)。不使用真实设备、窗口捕获、sleep、UAC 或网络夹具。

- 固定截面：全部分支／节点状态逐项比对，捕获期间继续编辑／检出不改变旧截面；Redo偏好、旧身份及原历史保留。
- 有界与取消：开始／遍历阶段超限、身份源中途取消、零值／碰撞／失败、错误来源／修订、活动草稿、回执耗尽、外部取消和旧令牌隔离。
- 完整性：瞬时创建删除的对象准入、草稿最终操作、重复拓扑拒绝、缺失／变化租约拒绝；未完成不标complete。
- 深历史：10001节点，10000次前向＋10000次反向边转换；bounded poll／k键往返及固定102字符hp1；共享Patch的Weak引用在取消／失败后实际失效。
- 真实资源：非当前分支的两个原始资源、同一素材的多条引用和不支持解码字节；所有捕获状态经P3-1A codec往返。媒体实际占用／峰值不因捕获增加；会话消失后消费者仍保住原字节，最后释放后预算／条目归零。另做20轮取消，当前会话资源保持且终结归零。

P3-1B本轮完成的是捕获与身份／租约基础，不是磁盘恢复。P3-1C仍须实现正式profile／能力、状态／节点原始记录绑定、全历史文件验证和身份／准入／导航恢复候选、Rust存储适配、首次发布及失败保护，再讨论正式保存／打开和自动保护。标签用例、偏好存储、长期回执容量和实际导出仍未完成。

本轮未复测完整捕获的GUI停顿、OS随机源、磁盘I/O、私有提交峰值或与图形／音频的组合性能，不宣称这些预算通过；没有改变原声音供给、GPU策略或P2门槛，也没有引入新的stable兼容规则。
