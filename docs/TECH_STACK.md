# 技术栈：当前基线与未验证范围

2026-10-30，以下来自实际manifest／锁文件和当前P2证据，区别于已[归档的历次选型记录](archive/2026-10-30-pre-review/README.md)。不重新排名未做同负载对照的框架，不把平台声明视为本项目实测。

| 部分 | 当前实际选择 | 范围／限制 |
| --- | --- | --- |
| 核心 | Rust 1.95.0／2024 edition，domain＋application＋gameplay | 无第三方依赖、禁止unsafe；gameplay仅4对象SyntheticTest、未接产品，本机Windows无窗口检查通过，Linux实测延期 |
| UI／宿主 | egui／eframe =0.37.0 | 可折叠普通面板及独立音效窗口，无复杂停靠；实际布局不是最终产品设计 |
| GPU | wgpu =30.0.1；Windows普通配置MemoryUsage | 本机Vulkan验证；DX12／UMA及其他设备不继承数据 |
| 图像 | image =0.25.10，仅PNG feature | 资源准备与CPU/GPU共享合成语义；不宣称完整stable图片格式支持 |
| 原生音频 | CPAL =0.18.2、rtrb =0.4.0 | Windows普通组装使用分离背景／声音路径；其他平台未验，真实输出端点变化硬件后补 |
| 解码 | 产品内有界PCM16 WAV解析器 | 44.1／48 kHz、单／双声道监听；Symphonia仅在P0独立实验，未加入产品 |
| 其他 | sha2 =0.10.9、pollster =1.0.1、getrandom =0.4.3 | 实际摘要、初始化等待、宿主会话身份及infrastructure捕获身份源；不将随机／设备能力引入domain或application |
| 状态codec（2026-11-02） | serde =1.0.229（std）、serde_json =1.0.154（std／float_roundtrip） | 仅infrastructure；默认feature关闭，有界Visitor及显式schema，不序列化领域布局；纯Rust，Windows离线检查通过，其他平台未实测 |
| Rust文件存储（2026-11-02） | rusqlite =0.41.1（bundled／hooks／limits）、same-file =1.0.6 | 仅infrastructure，默认feature关闭；SQLite C源码随依赖编译，需已有C编译器，文件身份通过平台句柄核对；不启用动态扩展或新后台线程 |
| 可选诊断 | profiling =1.0.18、tracing =0.1.44、winit =0.30.13 | host的framework-trace默认关闭；有界观察成本不得从产品CPU估算扣除 |

当前九个本地包及依赖见[当前架构](ARCHITECTURE.md)；[架构审阅](review/README.md)是原八包快照，新增gameplay见[K1-1审阅](K1_1_SYNTHETIC_REVIEW.md)。根锁文件新增无依赖的gameplay记录，桌面锁文件未改；没有新增第三方库、线程、设备、插件框架或服务。Windows ETW的C采集器是外部诊断工具，不是产品运行依赖。

## 平台和资源结论

当前原生证据范围为Windows 10 build19045／Ryzen 7 4800H／RTX2060、NVIDIA560.94、Vulkan参考环境，不是最低配置或全平台保证。Linux实测暂缓，macOS待设备；Linux音频依赖系统ALSA开发库／pkg-config。CI已配置，不等于远程已执行或Linux已通过。

资源门槛仍按[预算v2](RESOURCE_BUDGET.md)：稳定CPU≤2%、零欠载、提交p95≤20 ms、内存／GPU和实际回收保持硬项；播放CPU≤15%只是未达成的优化目标。P2当前产品配置与两轮300秒／有限20轮的结果见[最终验收](P2_FINAL_ACCEPTANCE.md)，不追认旧失败或跨构建数据。

128 MiB原始字节、80 MiB RGBA、32 MiB PCM、工作区／预览大小等均为目前实现／资源策略，不是未经验证的stable规则。P3前要决定实际项目规模／超限素材的保留与导出策略，见[待讨论决定](review/DECISIONS.md)。

## 后续重审条件

2026-11-03的[恢复增量](P3_1_RECOVERY_REVIEW.md)没有增加依赖或线程：Windows使用std文件只读独占共享句柄，已有sha2计算流式摘要，SQLite仅在物理副本回滚，same-file核对身份／源sidecar目标别名。其他平台缺少验证后的冻结原语时明确Unsupported。复制需要额外完整文件／日志磁盘空间及有界验证成本，不能把逻辑配额／租约释放测试当作峰值预算或断电证明。

P3-1C在桌面锁文件新增rusqlite、libsqlite3-sys、fallible-iterator、fallible-streaming-iterator、vcpkg五个包，same-file原已间接锁定；没有顺带升级其他包。SQLite／连接、受限JSON、摘要及完整模型恢复均有内存／CPU成本，见[审阅](P3_1_STORAGE_REVIEW.md)，逻辑配额不替代进程实测。Bundled避免依赖用户安装不同SQLite版本，但增加C编译、上游安全更新及分发许可审查责任；最终打包前还须验证目标平台VFS、硬链接／断电恢复和第三方声明。核心继续零第三方依赖。

P3-1B为infrastructure增加已锁定getrandom的直接边，实现application定义的永久身份候选源；没有新增包或版本。完整捕获与原字节租约见[审阅](P3_1_CAPTURE_REVIEW.md)，OS熵延迟与GUI冻结阶段尚未实测；本轮不以逻辑准入替代整进程内存测量。

P3-0审阅修订2采用同一SQLite容器路线，增加独立地址绑定、Schema表示、完整索引检查及流式读取；两引擎同文件对照见[加载审阅](P3_0_LOAD_REVIEW.md)。该工具增量未新增Cargo依赖，结果仍限Windows本机原型。

P3-1A实际项目状态codec新增上表serde／serde_json直接依赖；锁文件增加serde_json、itoa、zmij包记录，不增加SQLite绑定、线程或设备。JSON对象／索引分配与摘要计算有成本，限额不等于整进程预算；接口、维护影响及回归见[审阅](P3_1_STATE_REVIEW.md)。根核心仍无第三方依赖，产品声音／GPU策略不变。

P3-0使用Python标准库sqlite3作独立实验，未向产品加入Python运行时、SQLite绑定或第三方Rust依赖。v1候选为SQLite DELETE journal／EXTRA同步；Windows两套SQLite运行时的数据、资源成本和未测范围见[P3-0审阅](P3_0_REVIEW.md)。Rust绑定、打包版本、许可证、平台VFS与首次文件发布协议在正式接入前另行确定；根核心继续零第三方依赖。

- P3引入项目格式、压缩或编码器前，说明必要能力、平台、资源成本及迁移，不复用P0依赖作为默认批准。
- P4／P5需要真实谱面、歌曲和精细时间线时，重新验证codec、寻址、源格式转换、设备变化及声学范围；当前PCM复制锚点不是设备消费或声学位置。
- 发布前决定项目自身许可、检查实际产物和第三方／字体／默认素材清单；本次不提供分发许可结论。stable原版默认素材和用户媒体不能打包。
- 出现明确框架能力／平台阻断或同负载证据时再比较替代方案；没有证据不以另一套技术栈消耗替代维护。
