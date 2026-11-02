# P3-1A 审阅：实际项目状态编解码

后续增量：[P3-1C存储／完整恢复](P3_1_STORAGE_REVIEW.md)已在此codec之上加入Rust文件适配和恢复候选；以下未有捕获／存储的段落按P3-1A历史范围阅读，旧证明保留。

后续增量：[P3-1B](P3_1_CAPTURE_REVIEW.md)现已加入真实完整历史捕获、永久身份映射和原字节租约；下文保留P3-1A实现时的检查与边界，旧证明不改写。正式保存／打开仍未接入。

2026-11-02。已完成 P3-1 的第一个有界增量：为真实 SkinProject 实现独立、带版本、可共享的状态块编码和验证解码。**尚未接入项目保存／打开、完整历史捕获、SQLite Rust 绑定、自动保护或皮肤导出。** 普通桌面仍是内存编辑器。

本轮开始已按最新规则检查并通过正常 Git hook 保存上一轮 25 个路径的变更，回退提交为 `d65a47f`。随后本页列出的新代码／文档保持未提交，供审阅及下轮开始时保存；没有推送。

## 已完成的范围

- 显式编码当前模型全部字段：已用／未用／不透明 Asset，文档与嵌套图层，跨元素持久组件，独立音效绑定／增益／隐藏，原始配置记录及解释区间，输出映射。
- schema 1 与内存模型 2／application API 6 分开；128 位对象身份不经 JSON 浮点，完整 u64 计数使用字符串。既有运行时历史 ID、地址和去重协议保持原样。
- 小块按内容摘要共享，序列分页，嵌套关系以 ID 表示；单层变更复用无关块，保留旧 bundle。不是重放 Command，也不序列化私有 Patch 或 Rust 布局。
- 解码严格核对字节摘要、字段／版本、直接引用、完整状态闭包和领域不变量。返回脱离会话的值，不增加活动项目替换入口；原始媒体存在性及租约仍待后续加载用例完成。
- 有界 JSON、输出和累计展开准入；拒绝重复键、数字损失、迟余数据及无法精确表示的透明度。共享小输入超限时拒绝，不借共享绕过展开成本。

文件字段、默认限额及迁移边界见[状态 schema](PROJECT_STATE_SCHEMA.md)。本 schema 尚未作为发布格式，不能把 P3-0 的 `state.p3-fixture:2` 实验能力用于接纳它；正式 profile 和必要能力需在完整存储接线时注册。

## 对象、方法与依赖

以下是对[10月30日全量清单](review/README.md)的新增清单，不覆盖旧快照。Rust 没有类继承；这里新增的是组合、trait 实现和函数调用。

| 源码／对象 | 方法与引用／所有权 |
| --- | --- |
| [project_format/mod.rs](../desktop/infrastructure/src/project_format/mod.rs) · StateHash | from_bytes／bytes／hex；32 字节摘要值，与永久节点 ID 分离 |
| 同文件 · StateBlock／StateBundle | 组合原始字节、摘要和根；公开字段为未受信任输入，不意味着已验证 |
| 同文件 · FormatLimits／CodecError | default／validate（后者私有）；From<ModelError>；明确超限、格式和领域错误 |
| 同文件 · encode_project／decode_project／required_originals | 只读 SkinProject／bundle；前两者分别委托 write／read；后者列出原始 BlobRef，不申请租约或读取文件 |
| 同文件 · Writer／LimitedOutput（私有） | block／sequence、有界 std::io::Write 的 write／flush；持有块列表、去重索引及字节计数，使用 sha2／serde_json |
| [write.rs](../desktop/infrastructure/src/project_format/write.rs) | encode／preflight；只查询 application::values 暴露的领域只读值，按模型种类显式映射 |
| [read.rs](../desktop/infrastructure/src/project_format/read.rs) · Fields／Record（私有） | 字段提取、finish、真实引用核对；不接受未消费的未知语义字段 |
| 同文件 · Reader（私有） | record／sequence／refs／ids／asset／layer／document／components／sound／configuration／output；持有输入借用索引、已访问集合和累计准入；最后 SkinProject::from_definition 验证 |
| [json.rs](../desktop/infrastructure/src/project_format/json.rs) · Seed（私有） | parse、DeserializeSeed::deserialize 和 Visitor 的基础类型／序列／映射回调；有界结构、拒绝重复键，使用 serde／serde_json，不拥有项目 |
| [application/lib.rs](../crates/application/src/lib.rs) · values | 仅增加 ComponentTree／ProjectDefinition／ProjectId／SkinConfiguration 重导出；未增加 ProjectSession 写接口或文件依赖 |

静态方向仍为 infrastructure → application → domain。新增直接依赖仅在 desktop/infrastructure：serde `=1.0.229`（std）、serde_json `=1.0.154`（std、float_roundtrip），均禁用默认 feature。前者用于自定义有界反序列化 Visitor；后者用于 JSON 字节解析／输出和浮点往返；domain 未增加 derive 或第三方库。用途依据[Serde Visitor](https://serde.rs/impl-deserialize.html)及[serde_json 文档](https://docs.rs/serde_json/latest/serde_json/)核对。

平台范围为纯 Rust 元数据处理；本机 Windows 离线编译／检查通过，Linux／macOS 未实测。锁文件新增 serde_json、itoa、zmij 三项包记录；serde 已在桌面图中。代价为 JSON 暂存／索引分配、摘要计算与新的依赖维护面，没有新增线程、设备、网络或 SQLite 运行时；不宣称零成本。精确版本、feature 与禁止逆向依赖的负向测试已加入边界检查。根工作区仍无第三方依赖。

## 验证

`tools/check_core.py` 通过（包括 417 项 Python 工具测试、格式、Clippy、核心测试、实际依赖／私有访问检查）；`tools/check_desktop.py` 默认／全部 feature 的无窗口检查通过。新增边界测试曾暴露 default-features 约束缺口，已修复实际规则，初次失败日志保留。最后格式边界微调后另外复查当前 infrastructure 的格式、Clippy 和 14 项状态集成测试，原始日志索引及源码摘要见[机器证据](P3_1_STATE_PROOF.json)。没有运行 GPU、声卡或 UAC，不重判 P2 原生验收。

| 测试组 | 已覆盖行为 |
| --- | --- |
| 全字段往返 | 完整 128 位 ID／u64 计数，未用／不透明资源，独立图片／声音，隐藏、嵌套组和原始配置；模型相等 |
| 共享与旧状态 | 单层改动共享无关块，旧 bundle 不变；3000 根分页、2000 深组树；不代表整历史捕获成本已验收 |
| 格式与完整性 | 缺块、坏摘要、重复／游离块，未知版本／语义／字段，伪造引用、非法领域关系／配置区间 |
| 数字与解析 | 重复／转义重复键、超大整数及正负 2^53 边界、坏 UTF-8、尾随 JSON，f64 极值／负零及透明度精度 |
| 准入与隔离 | 字节／结构／逻辑项／累计展开超限不产生半个结果；小共享输入不能无限展开；解码不改变活动会话和历史 |
| 独立向量 | 自制 Python 标准库生成的空项目固定 JSON 字节，Rust 读取；不在测试中调用产品编码器重建预期值 |

固定样例为 [project-state-v1-empty.json](../desktop/infrastructure/tests/fixtures/project-state-v1-empty.json)，9 个块、项目 ID 42，根摘要 `e12103e91d4b26099887d9a4ed2c17304ef9c1af15f93606dfeceb598eaa891f`。只含自制元数据，没有私人素材、P0 实现或旧 stable 代码。

## 后续增量与审查点

| 增量 | 必须完成才可宣称的能力 |
| --- | --- |
| P3-1B：application 捕获／身份 | 完整分支、永久节点／历史身份、对象准入表、导航／标签／检查点和全部可达原字节租约；固定截面且允许后续编辑，保存回执不能把新编辑标成已保存 |
| P3-1B：成本与恢复候选 | 避免逐节点从根 materialize 导致深历史二次方成本；验证共享捕获、取消、超限、恢复校验及实际内存。不能直接循环调用本 codec 就声称高效全历史保存 |
| P3-1C：存储适配 | Rust SQLite／打包及平台路线、正式 profile／能力、增量事务、首次发布、文件故障保护、完整资源校验与候选恢复；保持原始节点／迁移表示只追加 |
| 后续保存闭环 | 再接手动保存／普通打开与主动最新、退出补存、有界自动保护和设置；有文件往返及失败证据后才开放正式 UI |

application 当前没有历史捕获／反序列化激活接口，长期回执容量问题也未顺带改变。当前 codec 限额不是整进程预算；没有测本轮冷加载速度、全历史内存／I/O 或保存期间图形／音频负载，不声称资源预算通过。状态共享已验证，磁盘增量共享和全部历史实际恢复仍未实现。

旧 P3-0／P2 证明、黄金向量、归档及冻结二进制继续保留，不改写历史测量。本轮也未改变声音供给、GPU 策略、CPU 目标、端点硬件后补范围或 stable 兼容规则。
