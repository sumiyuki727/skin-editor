# P3-2A 整体退出审阅

2026-11-16。结论：PA01–PA38的纯application要求已有相应实现与证据，**A在SaveCoord-Fixture-v1无窗口测试端口范围可以签收**。签收以包含本报告的提交完成正常hook、推送后同SHA三项CI成功为最终门槛；不能用轮前CI代替本轮4项新回归。路线继续A→B→C→D，无需改换保存基础；下一步先冻结B开工契约，本轮不接真实文件／GUI／设备。

轮前实际差异和暂存区为空，复用已推送的0ae80031e203272b5683695ccb8018546626e220，不创建空提交。该基线的[Actions 43725492883](https://github.com/sumiyuki727/skin-editor/actions/runs/43725492883)三项均成功，已逐名核对78项A专项测试在两份核心CI日志中各出现一次通过。独立审查复核了源码、契约和该完整SHA；本轮补4项直接组合回归，全部通过，没有修改生产实现、依赖或持久格式。

## 1. 证据身份与复核方法

[结构化证明](P3_2A_EXIT_PROOF.json)保存38行映射、82个真实测试名／路径、来源散列、基线run／job身份、解码CI日志规范化散列及补证执行回执。它是审阅清单，不是生产保存proof，也不引入新运行协议。未写循环引用的“自身提交SHA”：基线身份固定为0ae8003，新增测试／本报告以包含它们的Git提交及同SHA CI为准。本轮新测试不在旧CI中，证明明确区分。

| 代号 | 实际测试文件及数量 | 运行身份 |
| --- | --- | --- |
| A1 | [公开保存准入回归](../crates/application/tests/save_coordination.rs)，22项 | integration target save_coordination，裸函数名 |
| A1B | [固定槽内部边界](../crates/application/src/save_coordination.rs)，2项 | save_coordination::tests::* |
| A2 | [事实／核对回归](../crates/application/src/save_protocol_tests.rs)，28项 | save_protocol_tests::*，不包含子模块 |
| A3 | [关闭回归](../crates/application/src/save_protocol_tests/close.rs)，23项 | save_protocol_tests::close::* |
| A3B | [关闭末值／容量](../crates/application/src/save_coordination/close_bounds.rs)，3项 | save_coordination::close_bounds::* |
| EX | [本轮退出补证](../crates/application/src/save_protocol_tests/exit.rs)，4项 | save_protocol_tests::exit::* |

表内A1-01等稳定审阅编号指向证明中的完整函数名，按基线源码顺序分配；不是产品接口。82项计数不含helper、一般Command／capture回归和compile-fail doctest。既有[Command原子性](../crates/application/tests/commands.rs)、[捕获租约](../crates/application/tests/capture.rs)及外部不可伪造Permit的doctest继续由完整核心检查运行。

生产路径仍为[准入／固定槽](../crates/application/src/save_coordination.rs)、[事实／核对](../crates/application/src/save_coordination/protocol.rs)、[独立候选](../crates/application/src/save_coordination/candidate.rs)、[工作所有权](../crates/application/src/save_coordination/protocol_work.rs)、[关闭协调](../crates/application/src/save_coordination/close.rs)及真实submit／navigate／draft／capture入口。application只依赖domain，不导入文件、预览运行时或原生句柄。cfg(test)之外没有合格SaveProtocolAuthority签发器；PreparedWitness的测试承诺不能充当真实文件摘要或已落盘证明。

## 2. PA01–PA38逐项结论

每行只签收[开工契约修订2](P3_2A_START_CONTRACT.md)定义的A范围。测试断言与只读源码依据共同支持结论，未执行的真实文件／宿主部分继续欠项；不能按测试数量或某个成功状态外推SL整项通过。

| 要求 | 测试编号 | 已证明行为 | 边界／仍未证明 |
| --- | --- | --- | --- |
| PA01 | A1-01、A2-07 | 版本、无端口、Automatic及无关闭归属Exit在捕获前拒绝；不消费普通请求或永久身份 | 文件格式版本另按源码未变核对，测试不代表真实文件签发资格 |
| PA02 | A1-03、A1-22、A3-15 | 会话、修订、导航、绑定及无目标明确拒绝；合法零修订和q后延迟worker读取原q | 目标是注入值，未验证OS路径 |
| PA03 | A1-22、A2-20、A2-21 | 5节点／4准入／3原字节黄金树逐项核对分支、状态、父关系、原配置及资源字节；等数量变异拒绝 | 自制小型黄金负载，不外推产品深宽树或codec |
| PA04 | A2-20、A2-06 | q后新增E、资源、导航及草稿，q确认只覆盖旧范围，活动头／新工作保持 | 不宣称q实际写入文件 |
| PA05 | A2-06、A3-12 | 新增分支后回A仍待保护，导航或新分支使旧打开等价失效 | 相同head／导出不是全树已保存证明 |
| PA06 | EX-01、A2-08 | 真实确认后失败事务、空／净零命令保持current／covered、历史和快照；事实安装不自致脏 | 预览分离由无持久写入口及模块边界支持，未运行GUI／preview场景 |
| PA07 | A1-11、A1-04、A1-06 | 草稿、第二保存、外部捕获和最后消费者分别Busy，原截面不被替换 | 保留消费者阻挡新工作是预期行为 |
| PA08 | A2-08、A2-16、A3-18 | Manual／Exit事实分离、标记只追加一次，旧快照不变；Exit保留旧Manual | 标记由测试合格端口事实驱动，未接生产文件 |
| PA09 | A2-16、A1-16、EX-02 | 重放／重复完成不重复标记；已知CommitId拒绝；有效旧代次2及旧fact不能确认新候选3 | 文件内请求摘要与ID复用核对仍属B |
| PA10 | A1-09、A2-17、A2-18、A2-19 | 列明的资格／承诺变异、同head同数量不同内容、错误代次或上下文均不能安装错误事实 | 测试覆盖16类资格变异及5个承诺域；公共完整值等式由源码审查补证，不声称逐字段穷举 |
| PA11 | A2-01、A2-02、A3-12 | Manual A／保护C／默认A及推导Redo完整基线NoChange保留A／C；真实导航拒绝旧等价 | 打开基线由受验证恢复候选构造；真实读盘属B |
| PA12 | A2-03、A2-23、A3-13 | Latest无Manual及首次显式Manual空根须有新Manual事实，不能借全树覆盖或空创建跳过 | 从未编辑空根的Exit例外不扩大到Manual |
| PA13 | A2-09、A2-11、A2-13、A2-14、A2-15、A2-28 | 原提交、NoChange、VerifiedNotCommitted、证据不足四类观察分开；缺证据／读失败／冲突不盲写，事实与绑定冲突并存 | 受控观察矩阵，不是真实介质故障注入 |
| PA14 | A2-11、A2-13、A2-10 | 查询取消／失败不抹Unknown，旧attempt控制过期，原槽阻挡新写，可再次核对 | 先等旧查询实际释放才接纳新attempt，不允许仍持有旧owner而复用槽 |
| PA15 | A1-05、A1-08、A2-12、A2-09、A2-14 | 交付前、写许可前、可能写入后的取消分别处理；取消不抹已提交事实 | 取消本身不证明未提交 |
| PA16 | A1-06、A1-07、A1-20、A2-08、A2-10、A2-25 | Ack与Drop两种顺序及worker／sender／Snapshot／CapturedNode／query分别保管；最后退出才退槽 | capture_in_flight清除不能替代SaveSlot清理 |
| PA17 | A3-01、A3-17 | submit／navigate／草稿实际写入口在缓存前门控，新捕获Closing；只读／取消／协调可达 | 应用层门控不等于GUI输入路由 |
| PA18 | A3-03、A3-04、A3-05、A3-16、A3-21 | 同Close与真实草稿Permit才可收尾，失败原子保留，NoChange不造节点，旧回执／Permit拒绝 | 外部不能构造Permit另由compile-fail回归保证 |
| PA19 | A3-07、A3-08、A3-09、A3-20 | 取消仅作用自有子保存，独立Manual和事实保留；Unknown／旧完成／旧Ack不推进后来Close | 已确认矛盾保留事实并撤销满足，不伪造未提交 |
| PA20 | A3-06、A3-17 | 旧保存事实安装、Ack、实际释放之后才捕获最终q；q含草稿收尾，不依赖UI刷新 | 通过私有所属许可复用完整捕获，没有公开bypass |
| PA21 | A3-12、A3-18、A3-19 | Exit不推进Manual；VerifiedOpening按完整打开等价保留A／C，新分支／导航使其失效；CurrentProtection另核保护头 | 不同保护头的错误CurrentProtection保持Unknown |
| PA22 | A3-13、A3-14、A3-15、A3-22、A3-23 | 未编辑新空根、编辑Undo空根、恢复空根和仅预留持久身份分别处理；无目标保持打开 | 不选择临时恢复路径，真实恢复区属B |
| PA23 | A3-06、A3-12、A3-13、A3-18 | ProtectionSatisfied与空创建满足仍保持门控、不能终结Ack；显式取消恢复普通编辑 | 没有OS关闭、新建／打开／重开激活许可 |
| PA24 | A1-17、A2-24、A3-04、A3-11 | 128回执或节点上限拒绝新增，不剪历史；合法草稿Cancel、确认、首次查询和释放仍可达 | 宿主RequestId预留与设备清理仍属C |
| PA25 | A1-02、A1-12、A1-18、A1-19、A1-21、A2-22、A2-27、A3B-03、EX-03 | 控制512KiB−1、64／63节点、候选逻辑／capacity exact与−1、真实原字节N／N−1及实际Drop分别核对 | 原字节使用缩小配额N，未测试256KiB满额；限额不是同时可兑现的笛卡尔积，也不是RSS |
| PA26 | A1B-01、A3B-01、A3B-02、A2-11、A2-04 | 最终Save／Close最大值可用一次，普通准入保留最终资格；attempt／generation耗尽不回绕，清理不用新控制ID | 有限预留不保证无限失败重试 |
| PA27 | A1-10、A2-26、A3-16、EX-04 | 同ProjectId／head两会话的保存、查询、关闭／Permit隔离；交叉实际见证仍Unknown，正确下一attempt各自确认 | 全局会话新鲜性及导入／音频／GPU旧结果仍属C |
| PA28 | A2-24、A1-22、A2-20 | 重复只读状态不推进工作／消费观察／复制全树或生成历史；旧q及跨分支原字节保持 | 有界载荷另据静态证明，不等于测得整个Session堆成本 |
| PA29 | A1-01、A1-02、A1-03、A1-11、A3-01、A3-17 | 捕获3区分Closing／SaveOutstanding／Busy／InFlight，旧版本拒绝；零修订合法，混合expected拒绝 | 私有最终许可沿用原捕获校验；不可伪造另有compile-fail |
| PA30 | A1-13、A1-14、A1-15、A2-08 | 未保存永久映射不冒充发布；新绑定确认后转ExistingVerified且原Ticket仍Ack；恢复来源不可首次发布 | A核对绑定／token资格值；真实发布、发现及恢复转正身份仍属B |
| PA31 | A2-17、A2-18、A2-21、A2-23、A2-09 | 预先Snapshot和独立Witness保管；实际黄金内容先核对，缺见证不写、丢工作可核对，等计数不同内容拒绝 | 测试profile承诺不是生产文件摘要／加密证明 |
| PA32 | A2-09、A2-11、A2-10、A3-11 | 保存／关闭取消后独立query token仍可核对，迟到主完成可确认且不倒退，query实际Drop才退槽 | 未复位原取消token |
| PA33 | A1-04、A3-02、A3-06 | q+1及真实草稿收尾后精确Save／Close重放返回原Ticket、不新增身份；改载荷Conflict，释放后Stale | 重复在当前source重新校验之前识别 |
| PA34 | A3-04、A3-03、A3-10、A3-21、A3-13 | 失败草稿保留Permit和Cancel，旧取消不消费新Permit；advance不盲重试，显式重试等释放、再校验，失败准入保留关联 | Ack第二次消费可能收到迟到确认的再校验另经源码审查；未用线程时序猜竞争 |
| PA35 | A1-21、A1-07、A2-12、A2-20 | 静态容量计同时副本／见证／查询，取消／Ack不提前退候选；AwaitAck阻挡，捕获释放不退历史原字节 | 43083字节是Windows x64受管布局上界，不是allocator或RSS峰值 |
| PA36 | A2-04、A2-05、A2-22、A2-27、A2-23 | VerifyOnly在候选不足或代次耗尽时仅可核对既有NoChange；需写与读失败区分且无写权限，预算不互借 | 新目标／首次Manual不能借VerifyOnly成功 |
| PA37 | A2-06、A3-06 | q确认保留新草稿PendingDraft及基础，取消后新分支仍待保护；最终q在收尾后捕获 | 尚未进入application草稿的文本／IME属D |
| PA38 | A1B-02、A2-09、A2-11、A3-11、A3B-01、A3B-02 | 首次query同一预留对象及独立token在回执满额仍可用；追加attempt耗尽保留Unknown，最终身份与清理可达 | 不解释为无限重试或无限容量 |

## 3. 补证与独立审查

本轮识别的是四处直接组合证据不足，不是新功能需求。EX-01先完成真实协调确认和释放，再投递后一步失败的原子事务、空请求及净零请求；原facts、source、快照和节点不变，只有普通回执增加。EX-02从代次2恢复，候选应为3，分别投递合法非零旧代次及旧fact，仍Unknown且旧Manual／metadata不变。EX-03将实际黄金原字节总量设为N和N−1，验证准入原子性、key不误耗、独立capture可重用及真实最后所有者释放。EX-04让相同ProjectId／head的两运行会话同时Unknown，交叉控制／端口／实际Witness结果均不能串写；各自正确第二attempt可确认，但仍等query Drop。

实现者、退出测试补证者及只读审查者分开；主线统一构建。独立审查重点复核保存／关闭旧结果、FullTicket与候选来源、NoChange打开等价、Unknown清理、末值和512KiB边界。A3修复的五项问题和旧失败日志保留于[A3历史审阅](P3_2A_A3_REVIEW.md)，本轮没有再发现生产代码阻断项。新增四项以真实Session／Command／RestoreBuilder／HistoryCapture及已有受控端口执行，不造文件成功、不依赖sleep、网络或私人资源。

API7／捕获3／协调端口1／存储端口1及持久格式不变。[静态容量证明](../crates/application/src/save_coordination/budget.rs)仍为八个64KiB分区，普通／保留各256KiB，合计上限512KiB；Windows x64、Rust1.95.0的受管布局上界43083字节。候选1MiB逻辑额度及独立实际capacity、捕获逻辑额度和历史原字节分别计量。原字节N／N−1是缩小配额边界，不能写成256KiB满额或各上限笛卡尔积通过；布局数值不证明整个Session、allocator、RSS或产品性能。

## 4. A签收不包含的范围

| 后续 | 开工前须明确／验收的内容 |
| --- | --- |
| B真实存储 | 生产authority签发、完整文件内容与链／来源proof、只读四类核对、路径与绑定／首次发布／发现中断协议；持久恢复区位置、总配额、恢复转正式目标的身份关系；捕获／codec／写入／恢复共同容量及旧格式路线 |
| C完整交接 | 每次打开全局新运行会话、全部已消费ID准入；Manual／Latest候选激活、Quiescing与实际资源归还；旧导入／音频／GPU结果隔离、宿主请求号／取消预留。ProtectionSatisfied不能代替这些条件 |
| D产品入口 | 文本／IME／手势最后意图、快捷键、正式保存／打开及受保护关窗；普通Windows入口端到端保存→关闭→重开→继续编辑，保留全部历史和资源 |
| 后续独立项 | AP周期自动保护、通用SaveAs、迁移／标签／设置／导出，可视化拖拽／缩放V0–V3及皮肤制作Skill；均未由本轮实施 |

[总规格SL矩阵](P3_SAVE_LOOP_SPEC.md)仍按层验收：SL01–09、11、15–17、19仅其中A用例部分获得上述证据；SL13的空根／缺目标纯A用例亦有证据。SL10完整激活、SL12文件损坏、SL13真实恢复发现／转正、SL14恢复故障、SL18共同容量、SL20普通入口及其他真实路径未签收。普通编辑器仍无正式保存／重开闭环。K1保持已签收SyntheticTest适用范围，Reference10000／Tiny4不变；产品性能NotEvaluated、stable兼容Unknown，不重测、不扩大结论。

## 5. 检查、提交与保管

补证定向运行4项通过。完整正常提交hook继续运行仓库扫描、tools/check_core.py和tools/check_desktop.py；无时钟工具贯通是现有检查，不是正式成本测量，也不构成新的K1性能结论。最终接收须核对实际提交路径与散列、远端完整SHA及三个CI任务的同SHA success，失败时不能提前签收；检查回执存本机忽略目录artifacts/p3-2a-exit-20261116。证明内已执行的基线／定向结果与最终提交门槛分别记录，不套用轮前检查。

只提交本轮明确审查的测试、证明及同步文档。既有schema.rs空差异标记、原字节与mtime保持；其SHA256仍为713f3850740db024c4b861327de560f4eb81f5820980d1f89672dcdcfd499125。旧A1／A2／A3报告、成本原件、冻结程序和PRIVATE附件均不改。PUBLIC源码仓库与PRIVATE恢复材料安排不变，密钥不读取／上传。[3435405固定恢复快照](PROJECT_RECOVERY.md)不自动含本轮新增日志；本轮源码和公开审阅证明随Git保管，不能把推送源码描述为忽略目录已异地备份。
