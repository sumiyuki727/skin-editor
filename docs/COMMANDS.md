# 命令、事务与历史控制契约

2026-10-21，P1-2／3 Windows 无窗口检查通过，application API v3。当前是结构化 Rust 值接口，不是已发布的 JSON／AI 服务或 GUI。源码入口为 `crates/application/src/lib.rs`，值协议为 `contract.rs`，候选执行与历史实现均为私有模块。

## 唯一内容写入口

`ProjectSession::submit(CommandRequest)` 是唯一公开活动项目内容写入口。请求包含 `api_version`、`session_id`、`request_id`、`expected { head, revision }` 和有序 Command 列表。UI／AI／导入适配器共用接口；application 的 `values` 公开构造请求所需的纯值，适配器不必直接依赖 domain。没有整体项目替换、任意字段路径、脚本、回调或可变项目引用。

| 命令范围 | 当前能力及约束 |
| --- | --- |
| 资源 | RegisterAsset、RemoveAsset、RenameAsset；原始 BlobRef 不就地改写，删除仍被图层引用的资源返回 AssetInUse |
| 文档 | CreateDocument、RemoveDocument、ResizeDocument；仍有组件成员／输出关联时拒绝删除，调用方显式安排解除引用的顺序 |
| 图层／组 | AddLayer、RemoveLayer、MoveLayer、SetLayerTransform、SetLayerProperties、SetLayerAsset；新组先为空，删除组含后代，移动不跨文档 |
| 组件 | AddComponent、RemoveComponent、MoveComponent、SetComponentTransform、SetComponentMapping；删除组件树只解除组织关系，不删除文档或输出 |
| 输出 | SetOutputMapping、RemoveOutputMapping；新增输出 ID 只准首次使用，已有映射可修改，路径冲突仍由模型拒绝 |
| 配置 | InsertConfigRecord、SetConfigRecord、RemoveConfigRecord；有序保留未知原字节，不冒充 skin.ini 语义解析器 |

列表位置从零开始；移动目标索引以移出原位置后的目标列表为准。跨父组移动通过仿射重定位保持几何，拒绝循环／不可逆目标；同父组排序不重新求逆，避免无意义的浮点漂移。空操作和修改后回到原内容的有效批次返回 NoChange，不增加修订或节点。

新对象 ID 由请求提供类型化候选值，application 负责准入及会话内不复用；成功事务涉及的创建身份会保留，即使该事务中稍后移除该对象。失败／NoChange 的候选身份不发布或保留。后续可按真实适配器需求补充 ID 分配入口，不引入全局计数器或随机／时钟依赖。P1-2 尚无一般的跨文档复制／迁移用例。

## 原子提交与预校验

1. 核对接口版本、历史会话和命令数量；查已执行请求回执。
2. 对新请求核对期望节点与修订、回执容量，在隔离副本中按序执行。每个基础命令后检查领域不变量；不能依靠后面的命令修复非法中间步骤。错误含失败命令索引、对象身份或领域诊断。
3. 比较最终内容。NoChange 只记录请求回执，不产生内容修订、历史节点或资源固定操作。
4. 对有效变更先检查节点／修订溢出和历史容量，构造可逆元数据补丁，取得新增原始资源的租约。任何失败都丢弃候选、候选身份及临时租约。
5. 全部可失败步骤完成后，在同一 owner 内一起发布项目、历史节点、修订和资源引用；返回一次 Committed 结果。`&mut self` 将单个会话的提交串行化；未建立事件总线或跨进程事务。

`validate(&CommandRequest)` 是只读元数据预检，不保留身份／回执、不获取资源或改历史。返回 `would_change`、`cached` 和 `resource_check_pending`；资源待检查时不保证可以提交。提交始终重新核对修订、模型和资源。当前提交是同步原子调用；交互草稿及取消会话在 P1-4 实现，不能将未提交候选显示为历史节点。

## 去重与范围

同一历史会话内，相同 RequestId 和相同已解析请求值返回原始成功／NoChange／拒绝结果；不会再次执行或获取资源。去重发生在期望修订检查之前，因此已提交请求在后续编辑后重试仍返回原回执。返回值的 state 是原执行状态，当前状态应另查 project_info／snapshot。

通过信封校验后，同一 ID 携带不同请求（包括不同期望修订）返回 RequestIdReused，不覆盖原回执。冲突、命令失败和资源失败的回执也保留；修正后或外部资源恢复后应使用新 RequestId。错误会话、不支持版本、过多命令和回执已满的拒绝不登记新回执，避免非法／超限输入挤占回执表。

SessionLimits 默认每请求 1024 命令、65536 回执及 65536 历史节点，可在创建会话时显式配置。到限返回 ResourceLimit，仍能重取既有回执；不自动驱逐请求、裁剪历史或复用 ID。NodeId／revision／分支计数等溢出在发布前拒绝。这些是元数据数量保护，不代表整体 CPU／内存预算通过；请求字节、补丁粒度和长期成本在 P1-4 继续验证。

## 资源与历史

ResourceStore 是 application 的只读生命周期端口，BlobLease 固定已暂存并验证的不可变内容，租约不强迫原图常驻 RAM。导入适配器先在命令外读取、解码、核对摘要／长度和暂存；pin 只操作生命周期索引，不进行阻塞字节 I/O、哈希、解码或联网。默认会话没有资源存储，引用未接入资源的登记会明确失败。测试使用内存存储／租约，不是正式文件存储或真实 PNG 校验。

同内容资源共享一份租约；已成功发布内容仍可能被任意旧节点引用，当前删除不释放它。本阶段无历史裁剪，租约保留到会话结束。资源获取途中失败会释放本轮已取得的临时租约，不改变原租约或项目。端口返回错误 BlobRef 时同样拒绝。

首次内容写入已经同步建立根节点及追加历史：NodeId 为会话内非零 u64，外部另带 HistorySessionId。补丁只保存改变的资源／文档；组件树、配置和输出在该类别变化时保存前后值，不复制原图或每个节点的全部项目。changed_objects 是保守的失效集合，不宣称是逐字段精确差异。补丁正反向恢复已测试；history_snapshot 迭代恢复只读旧状态，HistoricalSnapshot 明确节点身份，不冒用当前修订。

P1-3 已实现 `navigate(HistoryRequest)`，仅含 Undo、Redo { child }、Checkout { node }。请求与 submit 共用 API／会话、RequestId、期望 head＋revision、回执容量及重试规则；跨两类请求复用 ID 拒绝。切头增加 revision，不追加内容节点；同头、根撤销、叶重做返回 NoChange。多个后继必须明确选择直接子节点；任意已存在节点使用 Checkout。

恢复沿最近公共祖先在私有候选内迭代应用补丁并校验模型／已固定资源，全部成功才切头。历史控制不接受外部项目值、任意补丁或资源替换，因此不是绕过历史的写入口。新内容仍只能 submit；在旧节点提交会追加分支，旧后代、准入身份和资源租约不删。新的失效集合根据切换前后内容生成，节点相异但内容相同时允许为空。

两类规范短地址、会话范围、分页和 10000 层／10000 同级分支测试见[历史契约](HISTORY_ADDRESSING.md)。P1-4 继续验证文档粒度补丁、请求回执和资源租约的实际成本并接入草稿。历史仍仅保存在会话中。

## 接口版本变更

P1-2 将 API_VERSION 从 1 升为 2：查询新增历史会话／活动状态，创建会话改为 `ProjectSession::new(project_id, history_session_id)`。宿主负责提供不复用的非零历史会话身份，不能用项目 ID 自动充当跨打开会话身份。

P1-3 将 API_VERSION 从 2 升为 3：新增历史控制、分页、地址与错误值，共享命令／导航请求命名空间。源调用方使用当前常量重新构造请求，保留两个创建参数；v1／v2 请求明确拒绝，不隐式升级或执行。现有 Rust 示例与测试已同步。MODEL_VERSION 仍为 1，尚无项目文件或持久化历史，不存在本轮磁盘迁移；未来传输绑定须明确 u128／u64 身份编码，不能经浮点数损失精度。
