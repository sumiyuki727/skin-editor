# B1-2L 保存捕获的只读借用

保存执行者现在可以把同一份冻结捕获交给验证器逐步读取，同时自己保管写入和完成回执的资格。文件侧在调用期间读取捕获，返回既有的固定大小 `FileSaveComparison`。该结果可以随观察保存供诊断，等级仍为 `InsufficientForSaveReconciliation`。

## 接口与所有权

`SavePreparationWork` 和 `SaveExecution` 均通过 `capture_cursor(SAVE_CAPTURE_CURSOR_VERSION)` 借出游标。版本 1 出借的是对原准备任务的独占引用。游标从同一所有者取得票据、快照、工作计数和下一步遍历结果；版本拒绝保留原位置。结束借用后，执行者可以继续使用自己的协议方法。已发出的节点和克隆快照各自持有捕获租约。[游标实现](../crates/application/src/save_coordination/work.rs)

`FileFactsReader::compare_save_cursor` 以游标、打开选择、原始资源库、已持有的文件观察、原票上下文和取消令牌为输入。它在第一次遍历前核对观察所属读取器、请求序号、完整票据、目标绑定及历史根。它分别保留请求捕获时的预期提交和文件观察时的实际提交，可以查询同一原票在较晚观察中是否出现以及各已比较字段是否吻合。[文件侧实现](../desktop/infrastructure/src/project_storage/file_facts/save.rs)

一次成功查询会用完遍历。有错票、错观察、预取消或准入预算不足时，调用方可以用原任务和正确条件继续。开始遍历后的错误保留实际进度。任务仍由调用方释放，查询返回值只含可复制的事实。原来的 `prepare_save` 继续持有完整 Work，适用于多次独立查询同一候选。[拥有型入口与游标回归](../desktop/infrastructure/src/project_storage/tests/file_facts/profile/cursor.rs)

新查询从 Reader 的 512 KiB 分区预留编码工作区和可能同时存在的固定输入、摘要与结果，捕获本身继续使用其已有预留。临时快照和所有临时预留在查询退出前释放。持有原恢复来源与文件观察时，用例在恰好足额下完成，差一字节时在遍历前拒绝。应用层的固定控制槽、捕获 3、协调 1 和持久文件格式沿用原契约。

## 验收与后续

- 自制工程的真实 Manual 准备任务同时匹配内容和原件。切换读取器、观察号、选择或票据后，请求在第一次 poll 前被拒绝。查询结束后 Ack 仍等待实际 Work 和外部快照；最终释放后仍可保留固定查询结果。
- 自制文件实际从代次 2 追加至代次 3 后，带原票的查询找到了该次提交，核对了请求字段、代次和导航。把原票的 expected 改成代次 3 会被拒绝。打开文件观察期间保留其排他读守卫，释放守卫后核对原文件字节。[两项文件回归](../desktop/infrastructure/src/project_storage/tests/file_facts/profile/cursor.rs)
- application 的受控协议测试通过真实关闭协调生成归属于本次关闭的 Exit 子票，经同一游标逐节点核对冻结树和原件。遍历完成仍须独立发布见证及回执；中途取消由下一次 poll 观察，执行者和节点各自按实际释放结算。编译负例覆盖见证、完成通道、写许可、借用逃逸和所有者重叠使用。[关闭回归](../crates/application/src/save_protocol_tests/close.rs)

真实文件 Exit 仍缺少与已注册目标、物理身份及打开基线绑定的只读资格。下一步应冻结其构造和预算，再确定如何在同一次遍历中保留完整证据，供 CurrentProtection、VerifiedOpening 和条件追加核对。H 的完整元数据与 I／J 的完整比较仍使用原有候选；分叉推导 Redo 的反例继续单独验证。Unknown 的事务阶段与连续性、共同 64 MiB 和 B2 写入均有各自的验收门槛。[B 开工契约](P3_2B_START_CONTRACT.md)
