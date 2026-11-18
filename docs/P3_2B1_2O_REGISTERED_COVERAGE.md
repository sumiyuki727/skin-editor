# B1-2O 登记期间的单次捕获覆盖核对

保存协调执行者可以按关闭请求的身份借出真实 Exit 子票的原游标。文件读取器现在可以在 B1-2N 登记守卫持有期间消费一个真实保存游标，核对冻结截面与原打开文件的完整逻辑覆盖。两端分别通过测试。生产恢复会话仍需取得关闭协议资格，才能把同一张真实 Exit 子票送到文件读取器。

## 关闭子票与原打开证据

`SaveExecution::exit_capture_cursor` 核对游标版本、实际保存意图及调用方期望的父关闭键。错父票或同一父票下的 Manual 子票都会在第一次遍历前被拒绝。该入口借用原执行者，保留任务、写许可和回执的所有权。[执行者](../crates/application/src/save_coordination/protocol_work.rs) · [关闭协调回归](../crates/application/src/save_protocol_tests/close.rs)

`compare_registered_coverage` 版本 1 的请求携带当前登记戳和完整保存票。查询通过登记槽取得从原打开一直持有的文件观察，再核对 Reader、真实游标、私有恢复来源及原预期提交。覆盖结果分别保留原打开请求及保存票。测试中的打开序号为 9、保存序号为 1。旧 J 查询继续要求其独立观察与保存请求序号相符。[登记覆盖入口](../desktop/infrastructure/src/project_storage/file_facts/profile/registered_coverage.rs)

原预期提交、代次、文件字节摘要、features 根及历史提交 ID 冲突核对沿用原基线规则。没有合格基线时返回空证据；旧提交 ID 碰撞在消费游标前检出。合格时仅遍历一次冻结树，核对节点及 admissions、全部原件及其字节、捕获元数据和分配计数。Manual／保护检查点的原始 Redo 各用对应检查点的计数比较，打开等价另核对加载时推导的 Redo 和修订。结果保留全部字段，以 `ExperimentalValidation` 和 `InsufficientForSaveReconciliation` 返回。[共用覆盖计算](../desktop/infrastructure/src/project_storage/file_facts/profile/coverage.rs)

## 预算、取消与释放

登记槽及来源沿用既有租约。查询从同一 Reader 的 512 KiB 控制池预留固定工作值和共享编码空间，流式核对在第一次 poll 前预留临时快照和遍历工作表。预算少一字节时原游标可重用，恰好足额时完成。已有 128 次文件提交仍可查询。部分消费和已完成的游标会被拒绝。

查询在遍历结束和覆盖返回前均检查原任务与调用方的取消状态。捕获即使已经 Complete，关闭协调仍可取消原任务。末段取消保留真实进度且不发布覆盖结果。返回值只含固定大小事实，查询退出即释放临时预留。原 Work 和独立快照分别释放后，保存协调才确认资源已归还；文件锁继续由登记槽保管。

## 本轮证据及下一门槛

| 路径 | 已覆盖的行为 |
| --- | --- |
| 文件侧真实 Manual 准备任务 | 普通打开、分叉推导 Redo、导航往返及新增未引用原件后返回同头；独立核对项目在冻结后继续编辑、工作计数、登记释放及原文件字节。 |
| 文件侧拒绝和边界 | 错父票、错 Reader、旧登记、旧 Work 配新登记、部分遍历、重复遍历、已占用历史 ID、容量边界和末段取消。 |
| 应用侧真实 Exit 子票 | 在 application 私有测试协议下由关闭协调器实际生成子票。核对正确父票、拒绝错父票及 Manual 子票；遍历完成后仍可观察真正的关闭取消，资源确认等待实际释放。 |

文件侧回归使用自制工程和真实登记。应用层关闭回归没有向 infrastructure 暴露测试资格。生产端仍须按已审阅的 B 契约将受信文件验证与活动会话的关闭执行资格接通，然后才能验证同一真实 Exit 子票穿过整条链路。条件追加、转入写入及最终关闭确认各有后续验收。[文件侧回归](../desktop/infrastructure/src/project_storage/tests/file_facts/profile/registered_coverage.rs) · [B 总契约](P3_2B_START_CONTRACT.md)
