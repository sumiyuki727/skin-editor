# 项目状态块：Rust schema 1

2026-11-02，P3-1A。本文记录已实现的 `infrastructure::project_format`，供后续正式工程文件接线使用；它编码一个完整 SkinProject **内容状态**，不包含历史树、标签、保存检查点、原始媒体字节或运行状态。外层 `.oskproj` 容器／永久历史约定继续见[文件格式](PROJECT_FORMAT_V1.md)。当前没有正式可保存／打开的产品文件 profile。

## 版本与边界

| 标识 | 当前值／用途 |
| --- | --- |
| 状态块 schema | `PROJECT_STATE_SCHEMA = 1`；本页所有块均检查它 |
| 根语义 | `editor-affine-audio-v1`；明确图像变换与独立音效的解释 |
| 内存模型适配 | 显式适配 MODEL_VERSION 2；后续模型改变须审阅适配器，不能借默认值悄悄沿用旧语义 |
| application API | 仍为 6；本轮不改变命令、u64 历史节点或会话协议 |
| 实验容器 | Python `p3-0-r2` 仍只支持自己的 fixture 状态；不能读本页块，也不因本轮实现自动成为产品存储器 |

版本分别选择容器、历史记录和状态读取规则，不按应用版本猜测。将来增加 schema 时须保留已发布旧格式的读取／无损迁移与固定样例。已有历史节点始终引用原始状态字节及摘要；另一编码器即使写出相同含义，也不得覆盖旧节点或迁移映射。新表示经验证后以 StateView 追加，见[迁移规则](PROJECT_FORMAT_V1.md#61-跨schema迁移不重绑定历史节点)。

## 入口与所有权

```rust
encode_project(&SkinProject, FormatLimits) -> Result<StateBundle, CodecError>
decode_project(&StateBundle, FormatLimits) -> Result<SkinProject, CodecError>
required_originals(&SkinProject) -> Vec<BlobRef>
```

StateBundle 具有 root 摘要及去重的 StateBlock 列表；StateBlock 是摘要和未受信任的字节。decode 先验证，再返回脱离活动会话的不可变 SkinProject；不持有 ProjectSession，不替换项目、不创建历史或租约。application 仅重导出构造此类验证值所需的领域类型，唯一活动写入口没有增加旁路。

required_originals 返回全部已登记资源的去重 BlobRef，包括暂未使用及不支持解码的素材。**它不读取、校验或固定原始字节。** 后续加载仍须验证媒体长度／摘要与实际存在性，持有资源租约并恢复完整历史，才能经 application 激活。当前元数据往返不是完整工程保存成功的证明。

## 信封、身份与数字

每个块是 UTF-8 JSON 对象，恰有以下五个字段：

```json
{"kind":"skin-editor.project","schema":1,"blocks":[],"blobs":[],"value":{}}
```

上例只展示信封，不是有效项目根。blocks 是该块直接引用的状态摘要集合；blobs 是该块直接引用的原始资源摘要集合。两者均严格排序、去重，摘要为 64 位小写十六进制；解码按 value 的真实语义逐项核对，不只相信声明。只有 asset 块直接声明原字节摘要，父块通过子块形成闭包。

- 对象 ID 为非零 128 位，写成恰好 32 位小写 hex 字符串，不与状态摘要或显示地址混用。
- u64 长度、音频帧数、序列长度和配置字节偏移写为规范十进制字符串，无符号、无前导零（零为 `"0"`）。支持完整 u64 范围；转换到本机 usize 超限时拒绝。
- 小范围宽高、采样率和声道为 JSON 整数。整数词法绝对值达到 2^53 时拒绝，避免解析器溢出后转浮点损失身份或计数。浮点字段可用带小数／指数的合法有限数表示。
- 变换六系数和音效增益使用 f64 往返；透明度使用 f32，解码要求 f64 解析值可精确表示为对应 f32，不能静默量化。非有限数拒绝。
- 拒绝重复键（包括转义后相同的键）、无效 UTF-8、孤立代理及尾随 JSON。合法空白、键顺序和等价浮点词法不要求统一排版。

SHA-256 对**实际存储字节**计算。编码器当前确定性输出，但不规定跨语言 JSON 规范化，也不把 StateHash 当作永久历史身份。decode 不修改输入 bundle。

## 块种类及字段

以下省略共同信封，列出 value 的全部字段。未知字段、种类、schema、枚举 tag 或根语义均明确拒绝；不返回可编辑候选，不静默丢弃。外层容器若支持可选不透明扩展，应另行原字节保留，不能让该解码器忽略未知必要语义。

| kind（统一前缀 `skin-editor.`） | value 字段／语义 |
| --- | --- |
| project | id、semantics、assets、documents、components、sounds、configuration、outputs；后六项为对应块摘要 |
| asset | id、name、blob `{sha256, bytes}`、media；media 为 image `{tag, size}`、audio `{tag, rate, channels, frames}` 或 opaque `{tag}` |
| document | id、size `[width,height]`、visible、roots（layer-ids 序列）、layers（图层块序列） |
| layer | id、local（六系数）、opacity、visible、node；image 为 `{tag, asset}`，group 为 `{tag, children}`，children 指向 layer-ids 序列 |
| visual-tree | roots（component-ids 序列）、nodes（组件块序列） |
| component | id、local、node；group 为 `{tag, children}`，element 为 `{tag, document, mapping}`，mapping 是文档到组件的六系数映射 |
| sound | id、slot、asset、gain、visible；不继承图像变换／透明度／可见性，不参与图像包围框 |
| config-record | raw（原记录字节的小写 hex）、interpretation；opaque 为 `{tag}`，section 为 `{tag, name}`，entry 为 `{tag, key, value}`；后者均为 `[start,end]` 字节范围字符串 |
| output | id、source `{tag, id}`（image 文档或 audio 音效）、key、path；路径仍由领域 OutputPath 验证，不是本机绝对资源路径 |
| sequence | role、len、pages（按逻辑顺序的页摘要） |
| page | role、items（按逻辑顺序的字符串，最多 256 项） |

local／mapping 数组遵循现行 Affine2 的 `[a,b,c,d,tx,ty]`：`x'=a*x+c*y+tx`、`y'=b*x+d*y+ty`。这只是编辑器状态语义，不表示已验证的 stable 坐标规则。

序列 role 为 assets、documents、layers、components、sounds、configuration、outputs 时，items 是块摘要；layer-ids／component-ids 时为对象 ID。序列与页须匹配用途、长度和页数，最后一页允许不足 256，空序列没有页。页内容顺序不能按哈希排序；相同配置记录和页允许重复引用，其出现次数和顺序保留。信封的引用集合去重不改变逻辑序列。

根节点和子节点引用使用扁平 ID 序列，不嵌套整棵树。领域构造器最终校验唯一身份、树关系、资源引用、音效槽、输出冲突等完整不变量。2000 层嵌套不会变成 2000 层 JSON；本轮仍不表示渲染器已支持同深度绘制。

## 共享、原字节与不可变状态

每个 Asset、图层、组件、配置记录和输出独立成块，索引按 256 项分页。修改一个图层只改变对应块及相关索引／根；无关素材、音效和文档块可复用。组引用子 ID 而非子内容摘要，避免修改深处子层时重写全部祖先组内容。当前仍会遍历并编码输入项目，本轮没有实现仅访问变化对象的历史捕获。

编码按摘要去重且核对同摘要字节相等；相同的旧 bundle 不被修改。解码接受任意块列表顺序，检查每块摘要、引用和领域语义，只接受**一个状态的完整闭包**；缺失、重复或不属于该闭包的块均拒绝。后续容器包含多个历史根和共享块时，应由存储器提取指定根闭包，不能把全文件块列表直接传入该入口。

配置保留原字节，包括未知／无效 UTF-8 记录、重复项及混合换行；已解释区间同时保留，不重建整份 skin.ini。音频 Asset 不因缺少解码器而丢失；状态记录保存原字节引用而非 PCM。媒体原字节在外层容器另存，派生图像／PCM／GPU／Voice 不进入状态。

## 准入与失败

| FormatLimits | 默认值 | 实际计量 |
| --- | --- | --- |
| block_bytes | 1 MiB | 单块实际 JSON 字节 |
| total_bytes | 16 MiB | 一个 bundle 的唯一块字节总量，不含原媒体 |
| decoded_bytes | 32 MiB | 实际重复解析的块字节累计；共享记录每次展开都计入，编码端也预检对应展开成本 |
| blocks | 131072 | 唯一块数 |
| items | 262144 | 解码时所有序列的逻辑项累计；编码预检对象、根与子引用 |
| json_values | 65536 | 每块 JSON 值数量，计入容器和值，不计键为额外值 |
| json_depth | 32 | 每块 JSON 嵌套，根深度 0；配置只能为 1–64 |

以上是可配置的 codec 准入策略，**不是 stable 规则、正式工程规模承诺或整进程内存租约**。原始媒体不占这些字节计数；BTree 索引、JSON 对象、领域重建和调用者输入仍有额外内存成本，后续组合负载须单列。小体积共享输入也可能展开成大模型，因此不能只限制去重后的文件大小。

编码前还按对象和文本保守预检（预留最多六倍 JSON 转义空间）；可能提前拒绝领域允许但超过当前策略的状态。超限明确报错，不截断、不删历史、不留半个候选。JSON 输出由有界 Writer 接纳，读取按单块处理，不缓存全部解析树；当前 API 仍由调用者提供内存中的完整 bundle，尚非磁盘流式读取器。

失败包括 ResourceLimit、InvalidJson／Field、UnsupportedSchema／Kind／Field、摘要／引用／闭包错误和领域 ModelError。失败只返回结构化错误，源项目和输入 bundle 保留；调用者后续仍须保留原文件，不能把失败当作空项目加载。

实现及固定黄金样例见[本轮审阅](P3_1_STATE_REVIEW.md)。后续读取性能、全历史捕获和实际资源租约须独立验证，不从本轮无窗口状态往返推断正式保存已完成。
