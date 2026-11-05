# K1-3：SyntheticTest 同谱面检查定位

**2026-11-05一致性审阅更新：** 初版15文件已通过正常完整hook保存并推送为`44687bf2db826d6f5c8aacf0899b3554e4f59244`，远端SHA一致。本轮补5项公开边界回归，gameplay共64项测试通过，未发现本切片需修订的实现偏差；运行源码、API3／profile1及四对象上限不变。具体检查与剩余范围见[本轮审阅](#2026-11-05检查定位一致性审阅)，下方初版结果按当时状态保留。

2026-11-05，承接用户“继续接下来的任务”，实现K1的下一个最小增量。上一轮K1-2一致性修订已通过正常完整hook保存并推送为`78b0236e0f834d6618752f5094cf7d5d0b17e129`；本轮先核对status／log／暂存差异及远端SHA，无新增内容差异，不创建空提交。原有schema.rs空差异标记保留。本轮新增量检查后未提交。

仍只支持纯Rust、无窗口、最多4个预编译SyntheticTest圆圈；规则profile保持1，公开运行接口／观察值升为gameplay API3。项目模型、application API、持久化格式、依赖及两个锁文件不变。不读取旧stable源码、真实客户端、路径、谱面或媒体，不接GUI／设备，也不宣称stable兼容。

## 本次范围与前置条件

本次只允许**已完成冻结的运行，在同一不可变SyntheticMap上作绝对时间检查**。调用者先用K1-2完成Freezing→Frozen，再请求检查；Running／Prepared／Freezing返回NotFrozen，故障运行返回Faulted。没有自动冻结、重新载入、换谱面或异步工作器。统一构造／载入／检查候选协调及完整Reference池仍待后续实现。

目标Q可在旧位置前后，也可为i64的两个端点。直接按Q与已验证圆圈端点比较，不用Q-1哨兵、不重放历史输入。Q处仍在接受区间的对象未判定；Q严格晚于close的对象标UnavailableReason::InspectionSkipped，其judgement保持None，投影直接Retired，不保留检查消退视觉。正常从头运行的生命周期／判定不变。

## 实际接口

| 接口／值 | 行为 |
| --- | --- |
| request_inspection(source, Q) → InspectionCursor | 核对完整当前来源、Frozen、唯一候选位、TransportEpoch／InputEpoch及独立请求计数的递增能力；成功仅创建候选，不替换当前截面 |
| step_inspection(cursor, budget) → InspectionReport | 使用运行配置的独立准备工作上限；每单位至多重建一个对象及其投影，返回Pending或Ready。空图也须一次评估才Ready |
| activate_inspection(key) → RunSource | 未完成返回InspectionIncomplete；完整候选才一起发布新来源、关闭Q截面、完整帧与Inspection统计来源，保持Frozen |
| cancel_inspection(key) | 取消Pending或Ready候选，不改变旧运行的账本、帧、时间、输入／事件。无需新代次、请求ID或队列空槽 |
| InspectionKey／InspectionCursor | 私有字段的检查专用类型，不能与MapPreparation、AdvanceCursor／RequestKey混用；绑定完整基准RunSource、Q、独立请求代次及逐步进度 |
| Observation.inspection | 暴露候选身份、已检查数／总数及准备完成事实；准备期间ready_frame仍是旧来源的Frozen截面，不冒充已到Q |
| StatisticsOrigin | Observation及ReadyFrame均区分FromStart{start}和Inspection{at}；后者的账本只记Q之后真实处理的合成事件，不计算或伪造累计score／ACC／combo |
| CircleFrame.unavailable | 检查跳过的显式原因；None判定不再被误读为正常等待。后续推进仍保留该标记，不补Miss／声音 |

一运行最多一个有效检查候选，新请求遇占用返回Busy，调用者先取消旧候选再提交新请求。候选存在时resume返回Busy，避免改变基准输入来源；旧基准发生故障时立即使候选失效。旧来源先返回StaleSource，同来源旧请求／游标／目标返回StaleInspection，拒绝不改当前候选。超预算调用不推进进度，原游标仍可使用。

取消A后再准备B，即便两者目标或拟用的下一TransportEpoch相同，也以独立请求代次区分。旧A在B完成前后均不可激活或取消B。达到计数上限不回绕；不能分配候选时旧Frozen状态保留，最后一个有效请求仍可无额外额度取消。

## 激活、恢复与事件

成功激活保留RunId／MapRevision，TransportEpoch和InputEpoch均检查递增；旧输入、推进游标、事件确认及声音意图不因新事件序号相同而有效。新事件序号从1开始，旧账本／事件不复制到新传输代次。这里仅清理纯值队列，不表示PCM／设备缓冲已释放。

激活时start、completed_through、last_committed_time和sealed_through共同为Q；输入清空、物理槽未知、无新判定／声音。恢复沿用K1-2再次换InputEpoch及逐槽重新武装，Q及更早输入仍不能倒插；重新武装后Q+1的有效新按下可以正常命中。没有自动输入生成器，也没有补发Q输入的路径。

## 存储及工作量

候选是运行器内唯一的固定内联记录，直接使用既有不可变map，不分配第二份运行／谱面，也没有晚释放的堆所有者。static_storage_bytes()计入保留候选及移动时的固定草稿空间，构造前按同一storage_bytes限额准入；取消不将仍随运行保留的内联空间虚假归还。Tiny的原64KiB静态存储断言通过。

准备的对象扫描／投影按预算1或256分步，最多4对象；激活只切换已准备帧、游标及固定队列簿记，不重扫map。这里的固定数组初始化／清理有明确4对象／64输入／64事件上限，不作为可扩展Reference实现。没有测量全部调用栈、分配器开销、进程工作集、CPU、取消实际耗时或10000对象成本，不宣称资源性能验收通过。

## 回归与检查结果

- 新增13项公开检查定位回归：S04窗口内／后、可见／接受／退休端点、前后及负时间／i64端点、完整截面发布、未完成激活、取消／替换／同目标旧请求、门控／错误无变更、重新武装与Q+1命中、关闭Q、跳过对象不补Miss、旧输入／事件／声音／推进作用域、空图、旧基准故障及大小预算结果一致。
- 新增3项单测：代次耗尽保留旧运行、最后请求仍可取消、完整基准来源／目标校验。首轮公开回归实际发现统计来源随双帧缓冲回退及基准故障未撤销候选两处遗漏，修复后通过。
- `python -B tools/check_core.py`通过：423项Python检查回归，三个根包格式、Clippy（拒绝warnings）、无窗口测试及release构建。gameplay为6项单测、16项冻结回归、13项检查定位回归、23项原有合成回归及1项编译拒绝，共59项。
- `python -B tools/check_desktop.py --architecture-only`通过；桌面代码／依赖未改，未重复桌面编译或运行原生路径。
- 收尾通过768个仓库候选文件扫描、9份更新Markdown的本地链接目标／围栏／空白检查及`git diff --check`；两个manifest／锁文件与schema.rs规范化字节保持HEAD，旧证明JSON未改。本轮实际新增量15文件，暂存区为空，原有schema.rs空差异标记保留。

本次仅覆盖K0-A／S04与KT11、KT21等适用的纯核心检查条款；统一准备候选、驱动互斥／自动驱动、10000对象及实际资源计量仍未完成。不将完整K1／KT11／KT21／KT23标为通过，K0-B与K2–K5继续分别验收。下一步先审阅本增量与K0-A一致性，再选择下一个小增量。

## 2026-11-05检查定位一致性审阅

本轮按用户“继续推进”，先备份初版，再审阅K0-A的初始／检查截面、S04、准备预算／取消和错误分类，结合运行推进、输入及冻结控制检查交错。新增测试验证已有公开行为，功能范围和黄金规则保持；统一构造／载入候选协调、Reference池和实际资源成本仍待实施。另修正路线正文残留的“K1尚待开始／全部KT待实施”表述，与已有切片和适用条款回归状态对齐。

| 审阅项 | 结论／补充回归 |
| --- | --- |
| 完整冻结准入 | Prepared及尚无Ready帧的Freezing不能创建检查候选；同源故障运行返回Faulted，不修改其既有前缀。新增公开回归覆盖这些门槛 |
| 接受窗口末端 | Q=close仍VisibleUnjudged、无判定／事件／Unavailable；恢复至close+1后才自然Miss，继续退休不重复事件。与初版Q>close的InspectionSkipped区分 |
| 候选期间确认事件 | 旧Judgement确认可正常退休；随后取消Pending候选不回滚该确认、账本或旧帧，恢复也不能重新交接保留的旧SoundIntent |
| 可见期交叠 | 使用目标0／200001、接受区间相邻而可见／消退期交叠的双圆圈，检查至100001跳过前者，100002新按下只命中后者；继续至全部退休不补旧Miss／声音 |
| 真实命中后再次定位 | 首次Inspection恢复产生Hit后，满事件队列下冻结、准备／取消及同目标重新准备均保持正确旧状态；新激活清除旧账本／事件，重复冻结幂等，后续新Hit保持最新Inspection来源，旧同序号确认无效 |
| 来源、进度和预算 | 初版完整来源／目标／独立请求代次、旧继续值、1／256工作量、i64端点、取消及计数耗尽回归继续通过；固定内联空间随运行保留，不将取消当作动态内存退还 |
| 产品范围 | 仍只支持已完成冻结的同谱面检查；没有自动冻结、换谱面、异步结果、自动驱动、GUI或原生音频／输入接线，不标完整KT或stable兼容通过 |

本轮没有发现已声明范围的新实现缺陷，补齐上述5项公开回归后无需改动运行源码或签名。`python -B tools/check_core.py`通过：423项Python检查回归、三个根包格式／Clippy／无窗口测试／release构建；gameplay为6项单测、16项冻结、18项检查定位、23项原合成回归及1项编译拒绝，共64项。`python -B tools/check_desktop.py --architecture-only`通过；本轮新增量不重复桌面编译，前置备份完整hook另行通过。

前置备份首次完整hook在全部feature的audio_a2测试链接时返回MSVC退出码1，未提供具体链接诊断，未创建Commit。随后单独重建／运行该测试16项通过，再正常重跑完整hook，默认／全部feature及最终桌面构建均通过，才提交并推送44687bf。确切失败原因未确认；失败与复核日志保留在本机忽略的artifacts/k1-3-audit。检查及hook保持原样，首次失败记录保留。

收尾通过768个仓库候选文件扫描、5份更新Markdown的本地链接／新审阅锚点／围栏／空白检查及git diff --check；运行源码、两个manifest／锁文件及schema.rs规范化字节与HEAD相同，旧证明JSON未改。本轮实际修改6文件、暂存区为空，schema.rs原有空差异标记继续保留。

后续仍按用户任务选择K1的最小增量：统一构造／载入／检查候选协调及资源所有权、驱动互斥／自动输入、10000对象与实际成本验收；K0-B规则证据和K2–K5分别推进。本轮测试及审阅文档未提交，不冒充已包含在44687bf中。
