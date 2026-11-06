# 独立 stable 兼容内核：新对话交接

**2026-11-06最新B审阅停靠点：** B初版32文件已通过正常完整hook备份推送为`747222c8c69e39ff39a84e933a73883bcfedb80c`，PRIVATE origin及远端SHA一致。[本次一致性审阅](K1_6B_STORAGE_PROOF.md#2026-11-06一致性审阅记录)修复激活计数回执丢失、无效起点多记分配及旧控制污染摘要计数，新增7项公开回归。正式核心通过425项Python及156项gameplay，桌面架构检查通过。固定Rust commit容量依据、Reference10000／Tiny4、API7／profile1及布局保持；G0、取消后的延迟释放、Pending／Failed旧完整帧和旧页隔离复核通过。本次修订未提交，C完整输入／实际自动规模轨迹及D实际成本仍未完成，不接GUI／原生设备、不宣称stable兼容。下方A及早期记录保留各自时点。

**2026-11-06最新停靠点：** K1-6A初版28文件已通过正常完整hook备份推送为`c447a2a`，远端SHA一致。[本次一致性审阅](K1_6A_STORAGE_REVIEW.md#2026-11-06一致性审阅记录)补5项来源寿命、同刻迁移、Pending／Failed旧完整帧／页及首次失败无帧回归，共137项gameplay通过；未发现需修改运行逻辑的偏差。API6／profile1、四对象及默认池保持，正式核心及桌面架构检查通过。本次测试／说明修订未提交，后续先备份再按明确任务推进；B–D、10000对象及实际成本仍待做，不接GUI／设备，不宣称stable兼容或完整K1／KT。下方早期记录按当时时点保留。

2026-11-03。这是架构与实施计划的交接，**没有实现内核功能**。新对话先读当前AGENTS／Git和用户本轮任务；本文件不授权自动开工、访问私人谱面、运行窗口或提升权限。

2026-11-04补充三项执行契约及K0-A／K0-B准入要求，详见[方案](STABLE_KERNEL_PLAN.md)；同日后续整理[K0-A开工规格](STABLE_KERNEL_K0_A.md)。本轮仅文档修改，未新增功能、fixture或KT通过记录，K0清单仍待逐项审阅／补齐。

本轮已完成K0-A文档自审并补齐列出的初始／检查截面、批次载荷／序号、重新武装、分步准备预算、错误分类及KT23固定负载问题。只表示文档修订完成，不记录用户确认或K0-A／KT通过；K0-B和K1实现仍是独立后续工作。

## 1. 当前任务与停靠点

用户希望独立实现一个适合皮肤编辑／离线测试的stable兼容内核。首批限定 **osu!standard／No Mod／圆圈子集**。当前主文档为[架构与分阶段计划](STABLE_KERNEL_PLAN.md)，K1-1合成模型、K1-2纯冻结、K1-3受限检查、K1-4统一候选及K1-5纯驱动增量已实现；K1剩余项、K2–K5和完整KT仍待完成。K0-B是可执行兼容profile准入，规则证据准备仍未完成，不因合成模型通过而开放。

“独立”是纯Rust、可无窗口测试的模块；产品保持单进程模块化单体。不能裁剪、复制、翻译移植无授权旧stable源码，不能因为lazer有实现就当stable规则已验证。

已实现SyntheticTest预编译圆圈、确定性时间／输入／生命周期、有界取消、冻结／恢复／重新武装、同谱面检查、统一候选／托管预算及自动输入／三驱动互斥；K1-1至K1-6B及各自适用复核完成，本次B计数修订待下轮检查备份。纯驱动不代表产品自动预览、音乐／设备、平台捕获／失焦／工具门控接通。MAX_CIRCLES仍是便捷入口4对象边界，Reference显式上限MAX_REFERENCE_CIRCLES=10000；不自动开始C／D或转去P3保存UI、素材skill或原生设备。

## 2. Git、代码基线和并行修改

- 工作区为 E:\skin-editor，分支codex/p1-bootstrap；远端[sumiyuki727/skin-editor](https://github.com/sumiyuki727/skin-editor)须再次核对PRIVATE。
- 历史恢复代码基线为2719d0f；先前交接文档保存于6e26e9d。动画设计回退点为d3f4b2b34b2c088932acf1641163c72915163e41，经过正常仓库hook并同步私有origin；它没有增加动画功能。
- 独立内核方案初版及关联交接已保存于881b6613625e4821036bd7fd2cb8a8cd710f96e5，完整本地hook通过并推送私有origin；后续契约文档回退点见下一条。该初版文档提交没有增加内核功能，不证明KT或规则已验证。
- C1／C2／C3及K0准入文档已保存于ff1a3081e2a5123a695bbbb1fc563c772ff94915，正常完整hook通过并同步私有origin；它只包含两份文档。
- K0-A规格初版及入口修订已保存于2be447c4fde0af92064598bddb68c6ed6ec93f49，本轮先通过正常完整hook、推送私有origin并核对远端SHA，范围为三份文档。之后的本轮自审修订按实际Git状态交付，不冒充已推送或已通过KT。
- K0-A自审修订三份文档已在首版实现前保存于c295816e2e0212383854b42d1f72386d6d2b1e99，经正常完整hook并推送私有origin、核对远端SHA。
- K1-1首版24文件已在2026-11-05保存为a609d743e97a6e951b7ee0cf1221e68e525f5def，正常完整hook通过并推送已核对的PRIVATE origin、远端SHA一致。
- K1-1一致性修订7文件已保存为88ed5df123a8d94b2d81ca1b72a706e2f4055513，同样通过正常完整hook并推送私有origin、远端SHA一致。
- K1-2首版17文件已保存为632b5d2317944a88144014691d5df18d9eeaf9d5，通过正常完整hook、推送已核对的PRIVATE origin并核对远端SHA。
- K1-2一致性修订7文件已保存为78b0236e0f834d6618752f5094cf7d5d0b17e129，正常完整hook通过并推送私有origin、远端SHA一致。
- K1-3初版15文件已保存为44687bf2db826d6f5c8aacf0899b3554e4f59244，正常完整hook通过并推送私有origin、远端SHA一致；首次链接失败及后续复核／完整重跑分别保留，详见该增量审阅。
- K1-3一致性审阅的6个测试／文档文件已保存为5c5e1ed1348c02e2895760a70b724de7f5daef5c，正常完整hook通过并推送PRIVATE origin、远端SHA一致。
- K1-4初版18文件已保存为4d4ee9bd3e11a9baaf6aef660c713b5e6b914898，正常完整hook通过并推送PRIVATE origin、远端SHA一致。
- K1-4一致性测试／审阅7文件已保存为de3cf8bed7dff80fe05c0751d3c0b3d43b96fbcf，正常完整hook通过并推送PRIVATE origin、远端SHA一致。
- K1-5初版21文件已保存为0a52f858c7de4824f5ab10e433ae067efdd00e4a，本轮正常完整hook通过并推送已核对PRIVATE的origin、远端SHA一致；随后的一致性测试及审阅文档另为本轮未提交修订。
- K1-5一致性测试及审阅7文件已保存为62a4e4d7bcd9647dc1a43c0ec50e7bcf59146349，本轮正常完整hook通过并推送PRIVATE origin、远端SHA一致；116项gameplay为该备份基线。本轮K1-6规格另作未提交文档，不修改生产源码或测试。
- K1-6开工规格初稿及入口7文件已保存为ffe2361a6a89bcc4c34f043c99b3974a217f35a2，本轮正常完整hook通过并推送PRIVATE origin、远端SHA一致。随后只修订规格／方案／交接等文档，未开发；旧“本轮未提交”按其当时状态保留。
- K1-6规格审阅6文件已保存为f1c8dbfdd086a84eafdb0f6ed0a2c9a1f7734389，本轮正常完整hook通过并推送PRIVATE origin、远端SHA一致；之后K1-6A代码／测试／文档另作本轮未提交增量，旧“本轮未提交”保留当时含义。
- K1-6A初版28文件已保存为c447a2a37380ee862699621917a8d366dd2a0d54，正常完整hook包含132项gameplay及桌面默认／全feature检查，推送PRIVATE origin并核对远端SHA；之后本次一致性回归／说明修订未提交，137项新状态另经正式核心检查，不沿用备份或远端CI结论。
- K1-6A一致性修订11文件已保存为b3c5f088932666c97ae8576b468b0c7b177bc7d5，正常完整hook包含137项gameplay及桌面默认／全feature，推送PRIVATE origin并核对远端SHA；本轮B的149项新增状态另经正式核心检查，源码／测试／文档未提交，不能沿用备份或远端CI结论。
- K1-6B初版32文件已保存为747222c8c69e39ff39a84e933a73883bcfedb80c，正常完整hook包含149项gameplay及桌面默认／全feature，推送PRIVATE origin并核对远端SHA。本次B一致性审阅修复三项计数边界，补7项回归，共156项gameplay另经正式核心检查，源码／测试／说明未提交；旧条目的“本轮未提交”保留其原时点。
- 最新默认流程：先看status／log／实际暂存差异，将上一轮来源明确、可审查的工作通过检查及hook提交成回退点，默认同步核对过的私有origin；用户明确说不提交／推送时除外。新增量轮末不自动一起提交。
- 保留其他对话工作，不并发Git写操作，不替其他对话暂存，不盲目全目录add、不跳过hook／强推／amend／重写历史。
- schema.rs可能显示M而实际diff为空，已有核对属换行规范化现象；先检查内容，不为消标记修改DDL、清理源码或改旧证明。

如果在同checkout继续，先确认是否有编辑器存储／素材skill对话在工作；按实际文件责任错开写入。需要隔离时遵循当前worktree工具和用户授权，不擅自搬移工程。

## 3. 最短阅读路径

| 顺序 | 内容 |
| --- | --- |
| 1 | [AGENTS](../AGENTS.md)、[Git规则](GIT_WORKFLOW.md)、[README](../README.md) |
| 2 | [内核方案](STABLE_KERNEL_PLAN.md)：C1／C2／C3、K0总表与KT矩阵；再读[K0-A规格](STABLE_KERNEL_K0_A.md)、[K1-2冻结接口](K1_2_FREEZE_REVIEW.md)、[K1-3检查](K1_3_INSPECTION_REVIEW.md)、[K1-4候选协调／计费](K1_4_PREPARATION_REVIEW.md)、[K1-5驱动／自动准备](K1_5_DRIVER_REVIEW.md)及[K1-6开工规格](K1_6_SCALE_SPEC.md)，区分完整计划与已实现部分 |
| 3 | [当前架构](ARCHITECTURE.md)、[技术栈](TECH_STACK.md)、[路线](ROADMAP.md) |
| 4 | [传输／定位](PREVIEW_TRANSPORT.md)、[输入](EDITOR_INPUT.md)、[设置／外部来源](PREVIEW_SETTINGS.md)、[GPU交接](PRESENTATION_CONTRACT.md) |
| 5 | [预算v2](RESOURCE_BUDGET.md)、[P2最终证据范围](P2_FINAL_ACCEPTANCE.md)、[首轮预览边界](p0/PREVIEW_SCOPE.md) |
| 6 | 需要编辑器全局背景再读[编辑器交接](HANDOFF_EDITOR_CONTINUATION.md)、[数据兼容](DATA_COMPATIBILITY.md)及P3审阅 |
| 7 | 涉及动画才读[原始动画／元素重采样](ANIMATION_ASSETS.md)，涉及素材skill才读[独立skill交接](HANDOFF_SKIN_ASSET_SKILL.md) |

不用重新读整段旧聊天或重跑P2全部CPU归因。docs/review是2026-10-30代码快照，各P3增量另有审阅；旧证明／归档不覆盖当前新功能。

## 4. 现有能力与具体代码入口

| 入口 | 真实状态／可复用范围 |
| --- | --- |
| [crates/gameplay/src/lib.rs](../crates/gameplay/src/lib.rs)、[分步准备](../crates/gameplay/src/map/preparation.rs)、[初始化](../crates/gameplay/src/run/initialization.rs)、[存储](../crates/gameplay/src/storage.rs)、[计数](../crates/gameplay/src/work.rs)及[K1-6B证明](K1_6B_STORAGE_PROOF.md) | 独立SyntheticTest API7／profile1，Tiny4／Reference10000；变长容量、分步工作及G0已测，来源共享和只读帧／分页保持；无桌面或stable规则 |
| [desktop/preview/src/lib.rs](../desktop/preview/src/lib.rs) | 占位PreviewState／Scene、冻结及输入计数；没有谱面、对象、判定或成绩 |
| [application/preview_port.rs](../crates/application/src/preview_port.rs) | 现有PreviewPort、PreviewContext／FrameSource／FrameKey和静态PNG准备值；不是已存在的游戏运行端口 |
| [desktop/ui/src/shortcuts.rs](../desktop/ui/src/shortcuts.rs) | 产品快捷键／修饰键统一入口；文本／IME／工具隔离继续复用 |
| [desktop/infrastructure/src/audio_supply.rs](../desktop/infrastructure/src/audio_supply.rs) | 分离背景／声音、有界控制／回执、触发／暂停／定位入口；尚不能据此宣称精确游戏排程／设备消费时钟 |
| [background_stream.rs](../desktop/infrastructure/src/background_stream.rs) | 有界背景读取／缓存；普通GUI尚未接真实歌曲 |
| [desktop/host/src/main.rs](../desktop/host/src/main.rs) | Editor与模块组装／原生资源生命周期；不能从中绕过application写项目 |
| [当前架构](ARCHITECTURE.md)及[审阅清单](review/README.md) | 图层共享合成、音频候选／资源与历史；按真实源码查具体方法，不编造新的现成API |

当前domain／application／gameplay三个根核心包，桌面六包；gameplay无产品依赖且未接入桌面，compatibility尚未创建。没有新增规则集插件、跨进程服务或第二游戏窗口框架。2026-10-30审阅包保留原八包快照，不是新gameplay清单。

产品仅支持有界PCM16 WAV，44.1／48kHz单／双声道；压缩音乐只在P0实验，不是产品能力。原生证据限已声明Windows设备；Linux实测暂缓、macOS待设备，真实USB／第二输出端点变化仍硬件后补。普通图形故障冻结诊断是已接受限制，完整窗口恢复只是显式验证入口。

## 5. 不得丢失的设计约定

**项目与运行分离。** application仍是SkinProject唯一活动写入口。GameRun可以拥有可变玩法状态，但不能持有项目写权限；载入、播放、输入、定位和成绩不进入项目历史，皮肤编辑仍经Command／草稿／事务。首版完整工程历史只增不删不可覆盖，所有分支原字节仍保留。

**依赖方向。** 未来gameplay无产品依赖，compatibility→gameplay，preview→gameplay，infrastructure→compatibility／gameplay。host注入application定义的控制／观察端口；application不导入preview、gameplay、egui／wgpu或设备。渲染器／音频不实现判定，UI不持有运行器可变引用。新包必须同步更新两套检查及负向canary，不放宽边界隐藏循环。

**时间。** 内核只消费有符号i64微秒的绝对MapTime，允许负预备时间，0为音频起点；主机单调时、源帧、输出帧、生成／复制／消费位置分开。稳定锚点＋有理／余数换算避免逐块漂移；原客户端判定量化单独由profile验证。现有u64 simulation_micros及100ms增量截断不可直接升级成权威音乐时钟。GUI只取样，不推进歌曲。

**输入／C1。** [输入封口](STABLE_KERNEL_PLAN.md#c1-输入封口)包含T，当前有效来源的新输入若时间≤T则拒绝并进入冻结／故障清理、统计不完整；旧来源先拒绝，不能误使新运行失效。封口前已接纳但待消费的T输入仍执行，正常冻结保留其内部结算资格。整批基于接纳前状态核对完整载荷／序号后原子接纳，不仅凭指纹；批内去重、过期序号及窗口有明确结果。入口与消费按住态分开，恢复以当前来源可信释放重新武装，旧释放不能解除隔离。文本／IME、编辑、失焦等门控保持；自动驱动与Auto Mod分离，恢复不补跑。

**对象／事件。** 对象原始序号及谱面修订标识稳定，Future→VisibleUnjudged→Resolved→Retired，判定与显隐分开。事件有唯一键和有界消费，GUI多画一次不多发声音。未知的notelock／堆叠／目标选择不能以最近圆圈替代；超出受测准入条件整谱拒绝执行，不删掉其他对象冒充兼容。

**有界推进／C2。** [推进和事件发布](STABLE_KERNEL_PLAN.md#c2-有界推进和事件发布)分开请求目标、已完成时间截面与同刻继续位置；Complete须完成目标全部前缀，Pending不能冒充已到达。一次迁移的对象状态、账本和全部语义事件在预留容量／验证计算后一起提交；不足时无半次迁移。同刻处理一半不发布Ready投影，继续不重复执行，取消不回滚已提交玩法前缀；未激活定位候选可丢弃。拒绝连最大单次迁移都放不下的配置，不用无限Pending、丢事件或全段复制／回滚解决背压。

**K0-A截面与准备。** 首次评估开始边界S之前无Ready，推进不得早于S；检查定位以Q为新的关闭截面，不重放输入或补Miss，已过窗口结果Unavailable／Inspection，账本只计后续事件。准备／校验／检查重建也按Reference=256、Tiny=1的独立工作量分步，构造／定位共用一个有效候选位。核对操作类别、基准来源和请求代次，不能只看下一TransportEpoch；过期在途分配仍计费。输入丢失、调用错误、旧请求及临时事件背压按规格错误表分别处理。

**音效交接／C3。** [提交和回执](STABLE_KERNEL_PLAN.md#c3-音效事件提交和回执)独立于事件生成：玩法事件键为运行／传输代次＋事件序号，多声音另带稳定子序号；宿主有界关联固定声音来源、目标时间、供给请求ID及状态。trigger_selected每次生成新请求，不能充当游戏事件幂等接口。已提交等回执时重读事件不重发，超时不证明未执行；部分接纳不重发整组，未知结果经门控／清理后明确取消。按连续前缀确认消费，旧回执不复活声音，记录退休不等于PCM释放。K1仅验证纯事件身份／有界确认，K3先fake验证宿主交接，再接原生复制与清理，不把接纳当声学播放。

**冻结／定位／统计。** 冻结固定时刻，编辑可刷新、显式试听可继续，恢复重锚且不补输入／旧声音。定位用新TransportEpoch协调音频、对象与动画，候选一起发布，默认冻结；检查跳过段不生成假Miss或音效。累计score／ACC／combo不可靠时Unavailable，区段／自动／样例来源可见；最终跳转HUD默认方案仍待讨论。

**皮肤交接。** 皮肤热替换、草稿取消、Undo／Redo／Checkout不得重置歌曲／对象／已判定状态。图片、独立音效与背景各自门控，核对自己依赖的完整来源；同源显式试听增益例外继续沿用。一个圆圈的多个部件可复用同一文档，但图层变换不改变命中点／半径。GUI区域、变换框、玩法判定区不同。

**外部资源。** 用户选定stable／Songs只读来源属于本机设置，先校验根／真实路径／重解析边界／限额；谱面／歌曲不进AssetLibrary、.oskproj、Git或皮肤导出。默认素材发布须自制／获授权；旧stable代码／附带资源不得进入产品、实验依赖或fixture。

**格式／动画。** 运行器不要求改工程schema；持久设置或必要项目语义确需演进再单独定版本／迁移。历史短地址继续与永久身份分离，不借用为对象ID。动画来源保留原始时长、按元素规则输出的AN设计不变，程序化接近圈动画不等于已实现动图导入。

## 6. 下一增量、检查和证据

当前B步及适用审阅已完成：容量证明对应本机rustc1.95.0 commit59807616e1fa2540724bfbac14d7976d7e4a3860／Windows MSVC，Reference10000、Tiny4；分步归并／状态初始化、原始工作与当前payload账本、G0预算1／256及大图检查保持。本次修复任务激活回执、无效起点分配计数及旧控制摘要污染，补7项回归。正式check_core通过425项Python、格式、Clippy、三个核心包无窗口测试及release，gameplay156项；桌面architecture-only通过，日志为artifacts/k1-6b-consistency/，先红后绿日志单独保留。API7的SyntheticMap非Copy／Clone；大图走MapPreparation::with_limits、RunPreparation或协调器，costs_for_objects(N)按容量预留，旧costs／static_storage_bytes只表示四槽下界。B初版已备份推送747222c8，本次修订留待下轮检查备份，不自动开始C／D。下列A与路线条目保留原时点，当前状态以本段及B记录为准。

规格审阅已备份为f1c8dbf，正常完整hook验证旧API5／116项及桌面默认／全feature无窗口基线；随后实现[K1-6A](K1_6A_STORAGE_REVIEW.md)，API6／profile1、4对象。新增实现经正式check_core通过424项Python检查回归、格式、Clippy、三个核心包测试及release构建，gameplay共132项；桌面architecture-only通过。没有成本测量／窗口／设备接线，日志在本机忽略目录artifacts/k1-6a/。过程中一次夹具目标错误及换行整理误清空本轮源码均已修复，恢复／黄金内容核对和最终正式检查通过，历史日志保留。

K1-6A已完成本次一致性复核，补5项回归至137项并通过正式check_core，含424项Python回归、格式／Clippy、三个核心包无窗口测试及release；桌面architecture-only通过。核对共享来源真实最后所有者、候选独立账本、同刻Hit／Miss背压与故障下的旧完整帧／旧页，以及首次发布失败。没有改运行算法或黄金预期，API6／profile1及四槽计费保持。新增日志在artifacts/k1-6a-consistency/，本次修订未提交，后续先备份。帧键依赖既有RunId在旧消息仍可能到达时不复用的组装者契约，现明确包括脱离借用的帧键／游标；静态额度不等于进程内存，受控拒绝不等于任意OOM恢复。B的变长布局／分步索引及完整原始工作计量、C的10000轨迹、D的外部成本基线分别待实施，不自动推进。

读取须同时保留实时已提交单项与T处完整帧；Pending不丢既有完整帧，也不将新判定／实时计数混入旧T。只读分页核对来源／发布键及有界输出，确认事件不换键，来源换代／新截面拒旧页。工作量除steps外记录读取／初始化／索引比较搬移和管理计数；准入预留、实际请求容量及存活所有者分列，不能相加冒充内存。相同修订令牌的独立分配不去重，共享来源一次、独立账本各计，取消资格后仍持有则不退额。成本输出另有版本及Unknown／Invalid状态，不凭低读数或缺失样本通过。

KT23已固定10000圆圈生成式：ObjectId=i、目标i×1000000µs、S01相对偏移、从-200001推进至9999300000。无输入轨迹预期10000次Miss／10000个Judgement；固定输入黄金脚本每批最多32对象、64条按下／释放，序号2i+1／2i+2，预期10000次Hit及20000个Judgement／SoundIntent事件。K1-6另列实际Automatic轨迹：经S冻结／切换／恢复后，序号1是释放事实，脉冲为2i+2／2i+3；分别核对来源后比较相同语义，不改原黄金输入。SoundIntent不触发声卡；规模、实际成本及完整KT仍待验，不是生产上限或CPU／内存通过证据。

| 门槛 | 必须准备 | 当前边界 |
| --- | --- | --- |
| K0-A → K1 | 固定SyntheticTest范围／值模型，C1／C2及C3纯事件约定，有限配置与拒绝结果，S01正例和S02–S04边界／交错／小预算的完整预期及KT映射 | 只允许合成预编译圆圈；没有.osu／设备／GUI依赖，不声称stable兼容 |
| K0-B → 可执行兼容profile | v14／Mode0逐字段支持矩阵，必需规则逐项来源／版本／数值边界／未知及fixture，R01非空自制兼容候选和R02明确拒绝样例 | 可以独立推进解析／拒绝诊断；必需未知的profile仍Blocked，不以过滤对象或合成规则放行 |

S01的工程数值和Hit／Miss预期以主方案定义为准，不是stable窗口／半径／分值；S01及S02–S04适用纯核心部分已在K1-1至K1-5内联自制数据中验证，含冻结／恢复／重新武装、同谱面检查、统一候选与托管预留／所有权释放及四对象自动输入／驱动互斥；规模及实际成本仍待补。R01必须另有必要规则依据，R02包含不支持对象／未知语义／歧义布局的整谱拒绝；同点同刻身份可表达，不表示已验证该布局。没有新增真实谱面或外部资源fixture。

证据至少区分SyntheticTest、公开文档依据、当前客户端已验证和Unknown；逐项覆盖格式／属性默认、AR／CS／OD、边界／顺序／堆叠、皮肤呈现及声音选择。客户端条目记录真实build／设置／输入与结果，不能从旧源码、lazer或合成通过自动升级；静态F12仅在已记录范围内验证外观，不证明时序。K0-A不要求把所有stable未知项解决后才做工程模型；K0-B不能用S01补齐必需未知或宣称空的兼容范围已可执行。

建议K1每次只处理一个可审查增量：

1. K0-A满足且用户明确要求后，定义强类型时间／坐标／身份、不可变圆圈输入、显式SyntheticTest规则和有限配置；增加gameplay与检查规则。
2. 先验证KT05／KT06、KT14适用纯核心项及C2：完成截面／继续位置分离，迁移预留不足无半次状态，绝对时间分段不变，事件稳定重读和有界确认。
3. 接带时间戳的两路输入／自动驱动，覆盖KT08／KT09及C1的封口前／等于／后、已接纳待消费、重试／冲突、顺序／门控及超限。
4. 补冻结／恢复／检查定位、旧代次及资源所有权，覆盖KT10／KT11，记录10000圆圈成本基线。
5. K1审阅后再加K2纯解析器／profile，可执行兼容范围须满足K0-B；K3先fake验证C3提交／部分接纳／回执去重、迟到／未知／换代及停止路径，再做编辑器／WAV／材料接线，之后K4手动圆圈对照和K5原生稳定性；不跨阶段宣称完成。

以上是分步路线：K1-1已实现第1项和第2／3项的适用部分，K1-2已补第4项的纯冻结／恢复／输入换代，K1-3已补同谱面检查，K1-4已补统一候选／托管预算，K1-5已补四对象自动输入／驱动互斥；10000对象及实际资源成本仍未实施，具体以增量审阅为准。真实谱面解析前确认支持的版本、字段／未知规则和限额；原生接线前明确音乐时钟／排程能力，不把现有trigger当万能定时器。不先搭通用玩法框架或全套UI。K3另有三项接线风险：锁定eframe是否能在布局前捕获／封口原生输入；背景复制锚点与真实消费位置的差异；静态元素缓存到多圆圈场景绘制的增量。未解决前只能明确较窄能力，不能把帧时间当硬件时间、即时trigger当精确排程或现有合成器当完整场景渲染器。

现有检查入口：

```text
python -B tools/check_repository.py
python -B tools/check_core.py
python -B tools/check_desktop.py
```

Rust1.95.0，工具链路径与本机缓存按README核对；不得静默下载／改全局工具链。新增包同步两个Cargo工作区的实际依赖检查、源码规则和禁止边的负向回归。默认检查无窗口、无GPU／声卡、无网络／UAC；原生／人工测试另经明确任务执行。

继承零欠载、已就绪短音完整请求到原生复制观察p95≤20ms、进程内存／GPU及实际释放门槛；播放单核CPU≤15%为优化目标，稳定空闲硬项不豁免。20ms不是设备消费、声学或判定误差指标；到期→接纳的宿主延迟另外报告。新内核先fake／门闩再短原生，最后当前产品入口两轮300秒／20次循环；不能套用P2旧证据直接判新路径通过。

K1-1至K1-5已运行适用纯核心回归，完整KT仍待验收，具体检查与未测范围见增量审阅。前置回退hook验证的是备份时既有代码，不证明之后新增模型；新增模型另经check_core验证。不重写旧_PROOF.json或清理历史失败来制造通过。

## 7. 可粘贴的新对话开场

```text
请在 E:\skin-editor 接手独立stable兼容内核规划。先读AGENTS.md、docs/HANDOFF_STABLE_KERNEL.md、docs/STABLE_KERNEL_PLAN.md及当前Git状态／HEAD／暂存差异，按最新规则保护上一轮已审查工作，核对私有origin并保留其他对话修改，不并发Git写入。

首批仅osu!standard／No Mod／纯圆圈受测子集。独立内核仍在同一模块化单体内；gameplay已有API7／profile1，Tiny4／Reference10000，来源共享、只读完整帧／分页及分步校验／初始化／原始工作计数，G0无输入规模已测；compatibility未实现。application保持唯一项目写入口，运行控制与tick不进入项目Command／历史。先读docs/K1_6B_STORAGE_PROOF.md的固定commit容量证明、API7接口／计费及一致性审阅后156项回归，再读A记录和K1-6规格、K0-A、C1／C2／C3及先前控制／所有权契约。未知stable数值不得硬编码成已验证规则，不复制旧stable源码，也不把lazer当stable证据。

按我本轮明确任务执行；交接本身不授权新增功能、接P3保存／打开UI、访问私人谱面或启动窗口／设备。2026-11-06最新停靠点是B初版已备份推送747222c8，随后完成B适用一致性审阅，修复三项计数边界并补7项回归；425项Python／156项gameplay和正式核心检查通过，未测实际成本。本次修订未提交，下一轮先检查备份，不自动开始C／D。固定Rust commit请求布局、两池预留与实际payload、计数、归并错误位置及分步初始化已复核；Pending／Failed旧完整帧、旧页隔离和万对象取消后延迟释放有新增回归。RunId在旧输入／事件／任务／帧键／游标仍可能到达期间不复用，控制头／分配器／完整栈未知，受控拒绝不是任意OOM恢复。C完整固定输入／实际自动规模轨迹及控制交错、D实际成本仍待做。取消仅撤销资格，持有任务继续计费；Prepared不是已评估起点，检查激活才ReadyFrozen并继承驱动。冻结／恢复不补旧时间／输入／声音，原生冻结锚点／Voice／捕获仍未接线；K0-B、K3和兼容性分别验收。

保持完整历史不删不覆盖、图片与音效独立、皮肤热替换不重置音乐／对象、冻结恢复不补跑及旧代次清理。零欠载、提交p95≤20ms、内存／GPU／实际释放门槛保持，播放CPU≤15%为优化目标。新原生路径和兼容性须另验，合成小模型测试不能替代这些证据。
```
