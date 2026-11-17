# B1-2J 原文件基线上的 NoChange 只读证据

本片从已备份的 35d7b0c 接续。查询回答冻结的保存请求 q 在已验证源工程文件上满足哪些覆盖条件。三项条件可独立成立，查询保留 Manual、保护和最新三个文件事实。适配器不给协调器投递完成消息。

| 判据 | 请求及导航要求 |
| --- | --- |
| CurrentManual | Manual 请求，有同头的真实手动回执，q 的 Redo 等于该手动检查点原始 Redo。 |
| CurrentProtection | Exit 请求，有同头的真实保护回执，q 的 Redo 等于该保护检查点原始 Redo。 |
| VerifiedOpening | q 来自实际恢复会话，原选择、打开头和含推导 Redo 的打开导航一致，编辑和导航修订均仍为零。Manual 意图另需已有同头 Manual，Exit 意图需已有保护。 |

三项共享完整树及 admissions、原始资源集合及字节数、H 捕获的已有元数据，以及最新 alias／branch 计数的核对。原始检查点 Redo 按该检查点的计数重算 q 的导航摘要，全树当前计数另按最新检查点验证。真实导航往返会失去打开等价，当前检查点的语义等价仍可单独查询。

入口沿用 I 的原基线守卫，绑定真实 q、完整 Ticket、FileFactsRequest 和观察，核对原 expected、真实恢复来源、源工程文件 SHA256 及 features 根，并在全部已验证提交中排除票上 ID 冲突。无合格基线时无结果；错误的调用上下文保留结构化错误。在可信基线上，不适用的保存意图与已核对但不匹配分开表达。

既有只读观察可在提交额度已满时核对。查询按 Reader 额度临时预留固定工作区；q 和观察均无需常驻新索引。固定返回值由调用方保管。实际 writer 可能返回旧保护回执，也可能因推导 Redo 而新增提交；两种情况均须通过独立文件观察验证。I 的条件追加摘要继续独立查询。

## 验收

以普通 A／C、分叉推导 Redo、真实导航往返、新增分支再回旧头、缺 Manual、原提交 ID 碰撞、错源与耗尽额度为边界。真实 Manual Work 经原恢复来源取得；真实 Exit Work 仍依赖生产协议资格，文件侧保护导航与纯意图判据分别验证。结果限于 Experimental／Insufficient，生产事实安装、Unknown 阶段判定、物理连续性与共同 64 MiB 仍需后续证据。

## 实现与验证结果

- 普通 A／C 的默认 Manual 打开同时匹配 CurrentManual 和 VerifiedOpening。独立写入器返回已有保护回执 C，文件字节保持原样。分叉样例的打开 Redo 含加载时推导的后继，只有 VerifiedOpening 匹配。独立写入器在自制副本上实际新增一代，重读后核对了该票的请求、代次及导航。[真实文件回归](../desktop/infrastructure/src/project_storage/tests/file_facts/profile/coverage.rs)
- 用户离开再回 A 后，当前 Manual 导航可重新匹配，打开等价失效。新增未使用素材再回 A 则保留新历史和新原件，完整覆盖不匹配。另用同头、同导航、修订均为零的工程验证原恢复选择不可串用。q 后发生的编辑继续留在当前会话。
- 九行固定判据中，五行通过真实 Manual Work 比对，另四行独立检验已脱离运行时的意图分类。四项共有条件分别置为不匹配的反例同属纯判据测试。未向基础设施签发 Exit 协调资格。[固定判据](P3_2B1_2J_CASES.json)
- 真实 128 提交文件拒绝继续预测追加，仍可查询 Manual 覆盖。检查已有 source／q／view 时查询工作额度恰好足额、差一字节、末段取消后的退额与 Ack。读取旧基线、来源不符和已占用 ID 均沿用 I 的守卫。[只读实现](../desktop/infrastructure/src/project_storage/file_facts/profile/coverage.rs)

受控源码及文档按新提交备份。新增原始日志仅本机保管，旧冻结证据、私有快照及未知的旧取消原因继续保留。K1、产品性能和 stable 结论沿用原审阅。
