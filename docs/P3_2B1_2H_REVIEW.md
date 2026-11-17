# P3-2B1-2H：真实来源链与捕获已有metadata审阅

2026-11-18（Asia/Tokyo）。轮前HEAD／PUBLIC origin为96b3a98086f24b3e0d59b21eb846d911179268a7，run44136548111首次三项成功；实际／暂存差异为空，复用回退点。906个受控文件原字节／mtime基线仅本机，schema.rs既存空差异标记保持。

## 1. 闭合的窄门槛

[冻结契约](P3_2B1_2H_CONTRACT.md)及[八个独立向量](P3_2B1_2H_VECTORS.json)在Rust修改前由两路审阅复算字段、TLV及SHA。新增独立FileProfile1入口只在同次完整只读full／G／D、源摘要／身份复核及cleanup成功后，共同发布恢复candidate与实际来源。源guard在返回前关闭，来源只证明那次读取；不调用旧load末尾的READ_WRITE，不授予物理连续性或保存资格。

不可变来源经detached candidate、新Session、共享Snapshot资源及真实Work传到不透明q。核心只定义Any+Send+Sync端口，getter借用；infrastructure识别私有真实provider类型，公共origin／summary和假provider不能取得比较资格。Manual A打开时仍保管最新Exit C的features和ArchiveMetadata，不混用打开点的旧features，也不拿writer默认四项补源。

before完整域3编码捕获已有receipts、按代次marks、规范排序tags、扩展原字节及全部真实features。新只读比较先执行旧完整q／Ticket／来源／binding／Reader／root守卫，然后用G查询选定commit；来源Unavailable、提交Missing和合法不等分别表达，均不绕过守卫。旧q对后续文件保持旧源。F.metadata仍NotCompared，fresh-append及旧would_commit不升级，相等仍InsufficientForSaveReconciliation。

## 2. 容量与审阅修订

全新紧凑载荷先合并预留固定索引、原文本实际capacity、Box／Arc及具名构造重叠；旧Cache.features只作临时复制来源，其旧full成本仍未证明。Box／Vec先Drop再退额，Session／Snapshot／Node共享一次租约；来源仍由Session持有时，已释放Work／capture消费者的Ack仍可Released。

两路实现审阅发现附着漏计附件自身逻辑成本，已让Restore及Capture共用checked的source＋handle计算，补有效恢复及捕获N／N−1。另让旧intent与新before显式共用一张工作表，并在独立词法块结束后才退临时额度。Clippy首轮拒绝对无Drop的固定Data显式drop；改为独立作用域，不加allow或减弱检查，原日志保留本机。

Reader512KiB、独立A控制512KiB和旧2Data／D+G费用不变。典型5项来源＋Session＋两个观察＋真实q通过；合法最大80702B文本来源＋Session＋q＋一个观察通过，再申请第二观察明确Limit并保留旧值。81920／81921仅为纯容器边界，不能称为合法最大文件。精确N／N−1覆盖新provider容器和核心附着／捕获；真实聚合覆盖既定512KiB准入及第二观察拒绝，未声称测得整个聚合组合的精确N−1。

## 3. 验证与证据界限

定向存储134项精确保留原124项并新增10项（2个内部容器／编码探针、8个实际文件组）；application94项精确保留原92项，全部现有integration及16项文档测试（1普通／15compile-fail）通过。八向量包括UTF-8、NUL／空版本和错误默认四项；same_count_required仅为编码反事实，真实完整读取明确Unsupported且不发布profile。

实际文件覆盖最新Exit与旧Manual、同数量optional name／version变化而旧content／originals／navigation／legacy相同、目标／绑定代次错配的实际Work、假provider／旧入口无源、外Reader、完整上下文／Missing隔离、源及before完成后取消、cleanup和激活失败、旧q面对后续独立writer文件、Reader／Session先Drop后Node延迟释放。

expected前进证据分两部分：核心synthetic A真实确认使expected更新，同时下个Work仍保旧origin；infrastructure对真实provider的expected错配探针不给before。它们不冒充B生产保存端到端。详细测试集合、归一化源SHA及本机日志摘要见[结构化证据](P3_2B1_2H_PROOF.json)。两路最终只读审阅无剩余实现阻断；正常完整hook、新SHA首次CI及远端受控树须单独核验，不能复用父提交绿灯。

## 4. 尚未关闭的门槛

完整B1／PB16、fresh-append metadata、三类NoChange覆盖、Unknown阶段与目标连续性、注册／物理绑定、共同64MiB、旧full／PNG no_limits／SQLite／VFS及宿主聚合仍欠。不得把before相等解释为已写入或可保存。本片不进入B2、首次保存／迁移策略、GUI／设备或皮肤制作Skill。

旧first-before、旧各次取消原因仍Unknown；旧证据／冻结程序／PRIVATE备份及schema.rs原字节／mtime保留，25分钟CI与30秒进程门控不变。仅自制fixture和显式测试根，用户归期未知，现场UAC／截图等待。新增原始忽略日志仅本机，不扩充旧私有恢复快照；PUBLIC备份覆盖受控源码和文档。K1／Reference10000／Tiny4、产品NotEvaluated及stable Unknown保持。

## 5. 平台辅助函数收尾

前述H实现及结构化证据对应57738b9fb323e3bc1e67c8dd910f78ff24ba1661：正常完整hook已通过516项Python、核心／协议贯通，以及桌面默认和all-features各310项基础设施测试。原PROOF、冻结契约／向量及其日志摘要保留；尚未以该中间SHA发布或套用远端CI。

补充静态审阅发现source_bytes测试helper只在Windows有调用，其自身cfg(test)会在非Windows测试目标留下未用私有函数的警告风险。唯一源码修订是改为cfg(all(test, windows))，与唯一调用方一致；生产代码、全部测试函数／断言、预算、接口及语义不变。未运行Linux桌面构建，不能称为已实测Linux失败或宣称整个平台通过。此修订后profile.rs归一化LF SHA256为f2772c456661cf57db62e8e103076c2b1470edee984b3fec9db622ab66c4aa13；原H PROOF内该源散列仍表示中间提交，其余15个源及全部旧证据原字节保持。

这一行修订与本追加说明构成独立小提交，须另经正常完整hook、最终SHA首次三项CI及915个远端受控文件逐项核对；保留两个提交，不amend。最终PUBLIC备份覆盖源码／文档，新增原始日志仍仅本机；保存资格及其余未关闭门槛不变。
