# K1-5：SyntheticTest 自动输入与驱动互斥

2026-11-05，按用户明确任务推进四对象最小增量。先将K1-4一致性测试／审阅7文件通过正常完整hook保存为`de3cf8bed7dff80fe05c0751d3c0b3d43b96fbcf`，推送本轮核对为PRIVATE的origin并核对远端SHA一致，再开始新工作。原有schema.rs空差异标记保留；本轮新增量未提交。

仍为纯Rust、最多4个预编译SyntheticTest圆圈，profile1不变。新增驱动类型、准备报告、观察字段及控制方法，gameplay API升为5；没有真实格式、GUI、原生输入／音频、后台线程、Mod实现或stable兼容声明。未访问旧源码、真实客户端或私人素材。项目模型、application接口、持久化格式、依赖及两个锁文件不变。

实现入口：[驱动／自动准备](../crates/gameplay/src/run/driver.rs)、[共同输入接纳](../crates/gameplay/src/run/admission.rs)、[冻结／换代](../crates/gameplay/src/run/control.rs)、[协调控制](../crates/gameplay/src/coordinator/control.rs)及[公开回归](../crates/gameplay/tests/drivers.rs)。

## 接口与有效驱动

| 入口 | 实际契约 |
| --- | --- |
| InputDriver、driver()／Observation.driver | Manual、Automatic、Regression三者仅一个有效；独立于Auto Mod及项目内容。现有公开构造／创建／替换载入默认Manual，未接入产品的“嵌入预览默认自动”设置 |
| select_driver(source, driver) | 要求非故障、已完成Frozen且没有检查／协调候选。实际切换检查递增InputEpoch、清理旧输入资格并保持Frozen；同驱动为无修改返回。TransportEpoch、时间、账本、事件身份不变；随后显式resume仍按既有规则再次递增InputEpoch |
| submit_driver_inputs、observe_driver_release | 手动／回归输入先核对完整RunSource，再核对所选驱动，最后进入既有完整载荷／重试／门控／时间／容量校验。其他驱动返回DriverMismatch；外部冒用Automatic返回AutomaticInputReserved，均不使当前运行故障 |
| submit_inputs、observe_release | 保留原签名，明确作为Manual便捷入口；Regression必须使用带驱动的方法，不能借便捷入口混入输入 |
| prepare_automatic_inputs(source, through, budget) | 仅Automatic且Running时生成／接纳有界输入，返回AutomaticReport。Complete只说明输入已收齐该前缀，不代表玩法已推进、对象已判定或Ready帧更新 |
| AutomaticReport::Pending | 区分WorkBudget及InputBackpressure；携带完整来源、目标、已可封口的ready_through、当次新接纳的游戏输入数量与步骤数。按同源重调继续，进度保存在运行内部，没有外部异步任务或可激活的迟到结果 |

SyntheticCoordinator具名委托上述控制，current仍只读。PreparationKey新增拟激活driver并参与完整键核对；同谱面检查从活动运行捕获驱动，通过私有完整构造传入候选，显式激活后仍是所选驱动的Frozen。创建／替换载入保留Manual默认；新RunId／旧来源拒绝规则不变。候选存在时切换返回Busy，不能暗中改变候选基准。

## 自动轨迹、工作量与C1

自动控制只使用PrimaryOne虚拟槽；对每个合格对象，在目标T按下该对象中心、T+1µs释放，构成原子两边沿脉冲。它生成普通InputEvent并经同一个有界批次接纳，再由既有推进／判定路径产生Judgement和SoundIntent；不写Hit／分数或跳过事件背压。自动输入可能在请求through=T时一并交接T+1释放；这是固定1µs的已声明前瞻，不改变封口包含边界的规则。

每次基本步骤至多检查一个对象、提交一次已准备的两输入原子接纳，或提供一次当前来源的虚拟槽释放事实；脉冲规划和接纳分成两个步骤。搜索／跳过已关闭目标及已判定对象也计费，WorkBudget=1可继续。事件推进仍使用原工作量，不把自动准备与玩法推进的Complete混用。这里只覆盖4对象和固定长度输入事务，不宣称10000对象成本已验收。

自动脉冲要求输入总容量至少2；容量1仍可用于手动／回归，选择Automatic时明确AutomaticCapacity且不改变来源。Tiny仍为4输入／2事件／工作量1，未放大默认容量。可用输入槽不足2时保留整个计划并返回InputBackpressure，不故障、不接纳半个脉冲；调用者先封口、消费报告给出的前缀，并按既有回执释放事件容量，再继续自动准备。不能丢输入或清空事件后假报成功。

Running状态的Automatic封口不得超出已收齐前缀，提前封口返回非故障AutomaticInputsPending。请求冻结先关闭自动生成并撤销未提交计划，因而可明确关闭选定F的接纳集合，再按原封口／有界收尾规则形成Frozen；不会为了补自动输入而越过关闭入口。自动准备不读取时钟，不由观察／取帧调用触发，也不自动推进或确认事件。

## 冻结、重新武装与旧输入

- 冻结前已接纳且时间≤F的输入保留结算资格；F之后的旧队列按原逻辑取消。自动索引只有在整个脉冲接纳成功后前进，冻结／恢复及切走再切回驱动均不倒退它，已经接纳后被取消的未来脉冲不能换个InputEpoch重新补发。
- 未接纳的计划在冻结时撤销。恢复后只有T仍严格晚于关闭截面时，才能按新来源重新规划这个未来对象；T≤关闭截面的按下不再生成。已检查定位使用新TransportEpoch及新自动进度，同样只处理Q之后的目标；不把Q处未判定对象直接当Hit。
- 恢复／实际切换驱动清理四槽资格。手动／回归必须提供新来源可信释放；Automatic的受控虚拟槽先通过共同ReleaseObservation路径提供当前来源释放，再生成新脉冲。该控制事实消耗共享输入序号，`admitted`仅计游戏边沿，不把0个游戏输入误称为没有控制工作。
- 本切片显式选择／恢复Automatic后，首个释放事实通常为新InputEpoch的序号1，首脉冲为2／3；这不是KT23未来Reference轨迹的1／2序号验收，也不改写其黄金负载定义。旧输入／旧释放先按完整来源拒绝，不能替新驱动重新武装。
- 序号／代次递增检查溢出，不回绕；脉冲两序号一起验证，失败不接纳半份输入。关闭／冻结仍不需新的普通请求、输入或事件槽。原声音意图门控及事件身份继续保留，恢复不重播旧SoundIntent。

## 所有权、验证与剩余范围

自动进度、一个可选两输入计划及驱动枚举固定存于SyntheticRun；无新堆分配／线程／设备，协调器继续使用既有Box／Rc及两池。新增运行值、候选键和池管理存储按实际类型大小计费，static_storage_bytes另计已知两输入临时数组；所有者释放与共享来源计费规则不变。Rc头、分配器、完整栈及进程资源仍未测量，不把payload预留当成完整内存验收。

新增15项公开回归覆盖三驱动共同Hit与互斥、自动专用入口、C1提前封口、准备不发布帧、四对象不同中心、Tiny背压与足额预算等价、较小目标继续、计划未接纳／已接纳的冻结交错、恢复不补发、驱动切换重新武装／旧释放拒绝、切走再切回不重标旧脉冲、Q前／等于／后检查、完整来源各字段、最小容量／代次耗尽、空图及有符号端点、协调检查继承驱动与载入回到Manual。另新增1项单测覆盖脉冲序号耗尽无半份接纳且仍可冻结，并扩展既有候选键单测拒绝错误driver。

`python -B tools/check_core.py`通过423项Python检查回归、格式、Clippy、三个根包无窗口测试及release构建；gameplay为10项单测、15项驱动、16项冻结、18项检查、26项协调、23项原合成及2项编译拒绝，共110项。`python -B tools/check_desktop.py --architecture-only`通过；备份完整hook与本轮检查日志分开保存在本机忽略目录`artifacts/k1-5-driver/`。没有新增包、依赖边或feature，没有放宽检查；新增内部模块继续由通用源码边界及既有负向canary覆盖。

下一步先审阅K1-5与K0-A的一致性，再按用户明确任务推进K1剩余规模／实际资源成本。产品自动默认值、失焦／工具／平台捕获门控及宿主调度仍属后续接线；10000对象、真实CPU／进程内存、K0-B及K2–K5未完成，不表示完整K1／KT或stable兼容通过。
