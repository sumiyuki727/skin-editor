# K1-6D：独立成本工具与首轮基线

2026-11-07。轮前HEAD为已通过完整hook并推送私有origin的`13fb4667840b6371d4830d92d308ad436fe5eb49`。本轮无新的可审查差异，复用该回退点，不创建空提交；原schema.rs空差异标记保留。随后按[K1-6规格](K1_6_SCALE_SPEC.md)实现D工具并执行测量。本次新增量尚未提交，不能用HEAD或先前CI代表本次构建。

## 实现与边界

新增独立工作区[tools/gameplay-cost](../tools/gameplay-cost/Cargo.toml)，仅依赖gameplay和标准库，自有锁文件，forbid unsafe。时钟、stdin/stdout及样本存储在工具中；根工作区仍三个纯核心包，无第三方依赖，没有改gameplay运行逻辑、API7／profile1、Reference10000／Tiny4或默认池。没有接GUI、原生输入／音频／GPU、真实客户端或私人媒体。

[check_gameplay_cost.py](../tools/check_gameplay_cost.py)负责图／源码检查、格式、Clippy、工具测试与release构建；正式[check_core.py](../tools/check_core.py)已纳入它的--check。独立工具不能反向进入产品，额外依赖／feature／目标／嵌套模块有负向回归。Root／desktop边界规则没有放宽。Rust工具只接收固定轨迹／预算／对象数参数，不读文件、网络或平台API；Python观察器只打开自己启动的子进程句柄。新依赖仅为既有gameplay本地路径，没有下载或安装新工具。

显式测量入口：python -B tools/check_gameplay_cost.py --measure --output artifacts/新目录。默认每个组合两个独立进程，共G0／G1／A1 × 工作量1／256 × 2=12进程；--check不启动实际成本轮。进程观测v1仅Windows实现，其他平台标Unknown，不冒充已测。固定配置不允许Cargo profile／target等环境覆盖；不改变全局配置。

### 冻结负载与输出v1

每进程10000圆圈，ID=i、T=i×1000000µs、中心(0,0)，S=-200001，终点9999300000；固定输入及真实Automatic各按32对象／64边沿分批。Rust逐事件核对来源、对象、时刻、种类及序号，逐页核对完整帧和实时账本：G0为10000 Miss／10000事件，G1和A1为10000 Hit／20000事件，均全退休、无额外事件。检查激活Q=5000000000，明确5000 InspectionSkipped、无伪判定／声音。

阶段依次为ready、calibrated、prepared、played、queried、inspected、cancelled、task_dropped、released，最后必须有end。查询含256摘要、157页、10000单项及一个4096次64项页批次；取消探针固定单步运行初始化并重建5000项后撤销资格，保留任务到下一阶段再真实Drop。另测一批256次“申请／取消／Drop未开始的检查候选”，它包含三项管理工作，不能除以256后称纯cancel耗时。最后分别计量owner及调用者fixture释放。

输出schema_version=1独立于gameplay API／profile及项目格式。identity.json冻结全部工具／gameplay相关源码及锁文件SHA、编译器、二进制SHA、目标／指针宽度、系统／CPU、调度与容量；report.json保留阶段结果、原始调用数、资源及工作计数、统计和置信状态。每进程另存原始child.jsonl、进程samples.csv及stderr.log；新轮须新目录，失败不覆盖，不自动换口径重试。

每类调用按N和有限推进上界预留样本，raw_ns完整保留；容量耗尽在下一操作执行前拒绝，旧样本保留并标Invalid，不丢尾部再报百分位。最近秩ceil(p×n)由Python从原始样本独立重算；零样本无百分位，数字类型、版本、阶段、工作数、容量、完整结束及资源归还均校验。原始时钟读数、阶段总时长及API调用样本是不同口径。

本构建的原始计时样本缓冲请求9,792,160字节，启动ready前用非零值写入并经black_box触碰，再清空逻辑长度。固定描述符、stdout缓冲、统计排序临时副本及调用者fixture另属诊断开销；排序／输出在阶段动作计时之外，其emit耗时与gate等待单列。子进程CPU／内存包含这些诊断成本；父观察器CPU单独报告且不混入子进程，也不从结果中扣减误差或开销。

### 计量有效性

墙钟使用Rust Instant；每进程先观察20000次时钟差及20000个空调用，再用32／2048对象各4次实际准备＋推进交叉校验，另连续运行至少2秒的256对象代表性准备／推进。固定校验要求参考负载确实执行、计数正确、参考时长超过观测分辨率100倍，CPU参考原始user＋kernel相对动作墙钟在0.75–1.25内；这是工具校准判据，不是产品性能目标。

每轮实际观测分辨率和空调用p95代理均为100ns，短调用p50不足代理20倍标Unknown；阶段动作不足代理100倍也标Unknown。raw百分位仍保留但不当成已分辨延迟。4096页批次只报告该批次时间，不冒充单次页调用p95。CPU的FILETIME单位是100ns，单位不等于实际计量分辨率；本机观测正增量为15,625,000ns，短于有效窗口的CPU百分比不报0。原始user／kernel差值保持。[GetProcessTimes定义](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-getprocesstimes)

CPU区间包含协议打印和门闩等待；连续忙参考不能验证周期性真实播放的全部采样误差。目标进程采样间隔1ms只是请求，实际最大采样空档114.0014ms（含校验／门闩时段），不保证捕获瞬态峰值。工作集与PrivateUsage分别记录，OS峰值是进程累计高水位，不能当成各阶段独立峰值或GPU占用；PrivateUsage与PeakPagefileUsage的字段含义按Windows文档记录。[内存计数定义](https://learn.microsoft.com/en-us/windows/win32/api/psapi/ns-psapi-process_memory_counters_ex)

## 实测：measurement-02

Windows 10 build19045／x86_64，AMD64 Family23 Model96 Stepping1；Rust1.95.0 commit59807616e1fa2540724bfbac14d7976d7e4a3860，release／offline／locked，无feature。只表示本机合成诊断基线，不外推Linux、macOS、其他CPU、产品N／S或stable。

| 冻结身份 | SHA-256 |
| --- | --- |
| release二进制 | e15b87f782e5d421bb2e46fea60dd552e17b4852a0af588d02c0665adec3d06d |
| report.json | 58b7538aeaacbc46e47ce991b0e3bf425aa23b2711c2e28d7fac8031bd416c55 |
| 排序源码摘要映射的紧凑JSON | 8a2a6573da61c5434278398aec675f38035a3f9d57c8f38e81949f9b7e8c4309 |

原始证据位于本机忽略目录artifacts/k1-6d/measurement-02/，共12进程／108阶段。协议、冻结身份、黄金结果、校准及资源回执均Valid；此Valid只指这些校验，不代表所有短调用／CPU窗口已分辨或产品预算通过。

下表为两个独立进程的阶段动作总时长范围，单位ms，包含驱动／计时／校验等诊断工作；不是单次调用百分位，也不是实时播放CPU。

| 轨迹／预算 | 创建准备 | 整曲推进 | 查询（含4096页批次） | 检查重建 | 取消探针（含半程准备） |
| --- | --- | --- | --- | --- | --- |
| G0／1 | 47.855–49.566 | 18.956–20.542 | 7.297–7.739 | 4.508–5.045 | 4.015–4.478 |
| G0／256 | 9.154–9.938 | 6.875–7.053 | 6.749–7.950 | 1.182–1.684 | 3.198–3.259 |
| G1／1 | 47.350–48.844 | 41.628–42.573 | 7.569–9.889 | 5.141–5.212 | 3.779–4.729 |
| G1／256 | 9.438–10.343 | 15.856–16.267 | 6.855–7.780 | 1.887–1.949 | 3.312–3.719 |
| A1／1 | 48.292–48.677 | 44.241–44.512 | 6.834–8.116 | 4.498–5.090 | 3.514–4.675 |
| A1／256 | 8.680–10.460 | 15.925–16.475 | 8.249–8.406 | 1.652–2.254 | 3.208–3.246 |

预算1／256的创建准备调用分别340032／1329次；G0推进90156／469次，G1及A1推进180000／939次，A1自动准备20002／314次。不能只比较一次调用或总毫秒而省略工作量与诊断调用数量。取消探针始终单步到同一半程位置，不把它当作预算256的整段准备速度。

纯cancel的原始单样本100–800ns全部Unknown。实际已准备取消任务的Drop原始46.2–96.2µs，owner Drop原始61.7–166.5µs；每阶段该操作n=1，两个进程分别保留，不以它们伪造充分统计样本。256次管理批次24.1–75µs；4096页批次5.441–8.462ms，批次单位保持。合计209个有样本调用序列仍Unknown；108阶段中96个CPU窗口Unknown，只有12个连续参考满足本轮CPU解释条件。不能据短窗口零读数宣布核心无CPU成本。

观测子进程工作集／私有提交采样峰值各16.953／14.727MiB，OS累计峰值各17.117／14.789MiB；包含约9.34MiB原始样本存储及工具开销。释放边界仍有10.50–11.12MiB私有提交，不据此判定核心泄漏或已返还OS；工具缓冲仍活着，分配器保留单列。最后owner Drop后managed／work字段为null而非伪造零账本。

已知受管峰值仍是来源预留／payload640168字节、运行预留1316408字节、运行payload739632字节；两池各8MiB未改，payload已含于预留，不能相加。取消后保持任务额度及payload，任务真实Drop后恢复活动检查运行基线；最后共享所有者的销毁另外由既有Weak／释放回归支持。这些数值与进程计数是不同证据。

## 失败保留、正式检查与停靠点

measurement-01保留Invalid：取消校验器错误要求初始化次数为N，漏算私有检查重建的N/2写入。既有B契约是N+N/2，实际15000次正确；新增回归修正校验，未改gameplay或回写原轮状态。随后还将工具样本缓冲改为非零触碰，冻结新源码／二进制，在measurement-02重测全部12进程；没有合并两构建成绩或追认01通过。

早期工具Clippy提示及Windows临时目录短路径夹具问题已修正，失败日志保留。正式tools/check_core.py现通过443项Python（新增18项）、169项gameplay及其他原核心测试、成本工具6项Rust测试、格式／Clippy／release和全部依赖／源码边界。桌面architecture-only通过。日志见artifacts/k1-6d/core-verified.log、desktop-architecture.log、tool-check-first.log及各measurement日志；新工具回归覆盖格式／缺项／零样本／容量耗尽／校准失败／进度退出／取消计数及边界canary。

最终核对共20个实际变更／新增文件、9份Markdown，git diff --check、805文件仓库候选检查及本轮Markdown本地链接／锚点／代码围栏／行尾检查通过。源码摘要与measurement-02冻结映射一致，12份原始协议日志哈希及report.json哈希复核一致。gameplay运行源码、根／desktop清单及锁文件、schema.rs规范化内容与HEAD一致；暂存区为空，构建及原始诊断保持忽略，本轮增量未提交。

D初版工具及这组本机基线完成，下一步先检查备份再独立审阅D，重点核对协议Valid与指标Unknown的区分、校准范围、诊断扰动、峰值观察和最后释放；之后再核对K1全部适用退出条件。没有整体验收K1／KT23，没有产品预算或stable兼容通过声明；K0-B、K2–K5及GUI／原生设备仍是后续独立范围。
