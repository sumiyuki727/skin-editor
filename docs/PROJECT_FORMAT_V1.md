# 工程文件首版设计：P3-0审阅修订2

2026-11-02，审阅修订2。工程扩展名采用用户指定的`.oskproj`；修正显示地址与节点摘要的耦合，增加跨Schema状态表示规则。当前实验为容器2／历史2，原稿[按字节归档](archive/p3-0-r1/README.md)，旧证明不改写。最新结果见[加载与格式修订审阅](P3_0_LOAD_REVIEW.md)。这是正式文件v1的设计阶段，尚无发布格式或产品保存接线。后续P3-1A已实现真实内容状态的[Rust schema 1](PROJECT_STATE_SCHEMA.md)；Rust存储适配、完整历史捕获和application保存用例仍待实施，不将实验文件当作完整SkinProject文件。

## 1. 版本与识别

工程扩展名 `.oskproj`，识别以文件内容为准，不做系统关联。容器采用 SQLite 3 文件，4096 字节页，`application_id=0x534b4550`（SKEP），`user_version=2`。这里的 2 是本产品容器版本，不是 SQLite 内部格式版本。头记录包括：

| 字段 | 本轮实验约定 |
| --- | --- |
| family | `skin-editor-project` |
| profile | 当前实验为 `p3-0-r2`；未来正式 profile 必须另行命名和注册 |
| container_version / project_schema / history_schema | 当前分别为2／1／2；另外每节点record_version=2，每状态块也自带schema；均不等同于应用版本或API |
| project_id / history_id / lineage_id / root_id | 各为非零 16 字节身份 |
| features | 名称、版本、是否必要；原型必要能力为 `storage.core:2`、`state.p3-fixture:2` |

未知容器版本／结构直接拒绝写入；已知结构但未知必要能力只提供受限只读诊断，不能称为完整语义验证通过。可选不透明扩展按原始字节保留，不能因为当前不能解释就删除。已知能力中未知状态种类也不能冒充可编辑。未知结构的自动迁移不在本原型中。

正式 profile 必须有完整 domain 模型 DTO 和验证，包含项目素材库（包括暂未使用的资源）、文档／嵌套图层、持久视觉组件、独立音效组件、输出映射和保留原记录的配置。图片与音效不合并为一套组件。本机路径、设备、监听音量、临时联合选择、草稿、解码缓存和 GPU 句柄不进入这些 DTO。当前实验 `fixture.project` 只含 project_id、测试步骤及依赖引用，不支持上述真实模型。P3-1A的真实状态codec独立于该实验能力；正式profile／能力、历史元数据和实际媒体保存仍须另行接线。

## 2. 身份与有界地址

持久 ProjectId、HistoryId、NodeId、对象 ID 和 CommitId 均使用非零 128 位值。新值由宿主安全随机源提供，写入前核对冲突；重复且内容不同必须拒绝，不将碰撞概率当作省略检查的理由。domain 不获取随机数。文件中用 16 字节 BLOB；文本用恰好 32 位小写十六进制，不带连字符，不经 JSON 浮点数。

| 寻址 | 编码／上限 | 作用域 |
| --- | --- | --- |
| 永久引用 | `hp1:<project32>:<history32>:<node32>`，固定 102 ASCII 字符 | 可复制到外部记录；仍需核对实际打开文件与节点存在性 |
| 节点短键 | `k`＋128 位值的小写 base36，最多 26 字符 | 已提供项目／历史上下文 |
| UI 短别名 | `n`＋非零 u64 的 base36，最多 14 字符 | 文件内显示映射；不是跨副本全局身份 |
| 分支坐标 | `b<branch>:<ordinal>`，非零 u64 分支及可为零的 u64 序号，最多 28 字符 | 文件内辅助映射；不拼接祖先路径 |

拒绝大写、前导零、溢出、空字段和零身份。标签、文件名、用户名与深度均不参与地址。永久节点只绑定固定身份、父身份、原始状态引用和操作；别名／坐标在独立addresses表，深度和子索引为派生数据。不得每节点存完整祖先数组。当前history_schema=2的树只允许一个父节点；未来合并DAG须新增版本／必要能力及对应读取规则。

每次打开创建新的运行 epoch，不写进文件。另有持久address_epoch标识当前显示映射；普通追加节点不改变它，只有显式原子重分配全部显示地址时才更换。n／b请求必须核对这个映射代次，不能让旧n1在重分配后静默指向另一节点。k／hp1直接使用永久身份，不依赖显示映射；标签、检出、排序或普通编辑不触发重分配。查询短地址／执行写入都要带打开上下文；恢复同一个持久节点不能使旧 AI 请求、草稿、音频事件或 GPU 结果重新有效。当前产品 API6 的 u64 NodeId 和 HistorySessionId **保持原样**，将来以显式 API 升级和类型映射采用本约定，不能把旧 u64 直接解释成新持久主键。

复制文件保留已有项目／历史／节点身份；副本继续编辑时生成新的随机 NodeId，因此两边即使都出现 `n42` 也不是同一新节点。跨副本引用用 hp1／k，不使用裸 n／b。lineage 标识保存谱系，普通文件复制会复制它，不能独自识别物理副本；实际目标还需文件身份和捕获的上次 CommitId。独立创建项目／历史才分配新 ProjectId／HistoryId。另存为、合并与去重不是隐式功能；覆盖既有目标须先证明不缩减其历史、同 ID 内容一致，否则拒绝。原型没有自动合并／另存覆盖入口。

对象身份准入表记录种类、ID、首次创建节点；同一事务内创建后删除的已用 ID 也不能重新分配。仅扫描当前头不足以恢复该规则。正式加载必须恢复此表和完整历史。

## 3. 字节、状态块与历史记录

原始资源按 SHA-256 和长度索引，分为至多 65536 字节的顺序块；哈希针对完整原字节，不针对路径、解码结果或图片像素。同摘要复用仍核对长度／字节，冲突拒绝。未支持的音频和未知附件也完整保留；PCM、RGBA 和导出文件另管。原型测试素材全为自制，不访问用户皮肤、谱面或音乐。

不可变状态块保存原始 UTF-8 JSON 字节及其 SHA-256。信封恰含 `kind`、`schema`、`blocks`、`blobs`、`value`；普通块schema为1；fixture.project另支持schema2，引用是排序去重的摘要数组，每项为64字符小写十六进制（32字节摘要）。所有依赖先入库再写块，拓扑序号校验状态 DAG，无递归恢复。完整项目根可以引用共享的小块；不是每个节点全量复制项目，也不是要求重放旧 Command 或私有 Patch。

JSON 拒绝重复键、无效 UTF-8／孤立代理字符、非有限数字；绝对值不小于 2^53 的整数用明确文本表示。合法空白、键顺序和浮点数词法形式不被强制改写；校验哈希针对**实际存储字节**。确定性写出是编码器的选择，不要求 Rust 重现 Python 的 JSON 排版。未知内容保留原字节。原型限额见第 7 节，不能当作 stable 格式规则。

不可变节点包括record_version、ID、父ID、原始状态根摘要和操作描述原始JSON。n别名、分支坐标与标签均不参与历史摘要；地址绑定有自己的校验，受完整性检查与映射代次保护。工具里的Node是组装的查询／捕获条目，虽然同时返回地址字段，node.record()只编码永久历史部分。操作描述供审查与 UI 使用，不是加载时要执行的命令。内容校验采用以下二进制 frame；所有整数为大端，`len64` 是字节长度的 u64，所有原始字节直接拼接：

```text
optional(x) = 00                         if absent
              01 || x                   if present
sized(bytes) = len64 || bytes
node_frame = ASCII("skin-editor.node.v2") || 00
             || node_id16 || optional(parent_id16) || state_sha32
             || sized(action_raw_utf8)
node_checksum = SHA256(node_frame)
```

深度不参与 frame，由父链验证；标签、保存位置及当前头也不参与 frame。同 ID 再写相同记录仅可幂等复用；任何不同父关系、原始操作或原始状态都拒绝。显示映射调整不改此frame或checksum。新历史只追加，不更新／删除旧节点或它们依赖的原始字节。这里是**逻辑历史追加**，不是承诺底层 B-tree 页永不重写或提供 WORM 防篡改。

## 4. 表与可变边界

| 表 | 内容／更新策略 |
| --- | --- |
| header、features | 格式身份与能力；同版本内不可替换已存在的记录 |
| blobs、blob_chunks | 原始字节；只追加，内容寻址复用 |
| blocks、block_refs、blob_refs | 状态原字节与依赖；只追加；kind／schema／project_id派生列须与原字节核对，索引随新增块局部建立 |
| nodes | 不可变节点及record_version；父／ID索引，不存祖先路径 |
| addresses | 每节点一个当前n／b绑定及独立摘要；可在新address_epoch下完整原子替换，不改节点或其摘要 |
| state_views | 原始状态摘要、目标schema、目标摘要、带版本converter及证明摘要；只追加，主键含origin／target_schema／converter |
| object_ids | 已使用对象身份，包括当前已删除对象；只追加 |
| commits | 逐次文件提交的身份、前次提交、代次、来源、可选头／导航和请求摘要；只追加 |
| manual_marks | 真实手动提交所捕获节点的保存事实；只追加 |
| tags | 用户标签的当前值，允许增／改／删；不新增内容历史节点 |
| control | 当前文件代次、最近文件提交、手动／保护检查点、address_epoch及资源计数；事务更新 |
| extensions | 可选扩展原字节；原型只追加，修改扩展的版本协议另立 |

不可变表拒绝 UPDATE、DELETE 和重复主键 INSERT（包括 REPLACE）；同值复用在插入前明确核对。SQL 约束和触发器防编程错误，外部文件编辑者仍能破坏它们；它们不是加密认证、权限隔离或防恶意篡改承诺。加载需检查结构、引用和字节校验，发现损坏不能删节点来“修好”。

用户标签用 `user:` 命名空间，独立编辑／持久化；保存标签不能通过这个入口伪造。系统标签由事实求得：根／当前／末端／分叉由树和活动头决定，保存／曾保存由 manual_marks 与最近手动提交决定。检出旧节点不回滚标签。原型证明标签与节点分离，但没有正式标签 UI／application 用例。

## 5. 增量事务与检查点

写入先校验 epoch、捕获的 previous CommitId、容量与引用，`BEGIN IMMEDIATE` 后在同一事务写新资源、新状态、新节点、标签和局部索引，最后追加提交记录并更新 control。失败全部回滚；回执丢失后查询 CommitId 判定是否真实落盘，不用盲目重试覆盖。单一写入者；冲突／忙碌显式返回，不睡眠等锁。保存旧内容也必须保留目标中所有较新分支；原型没有删除接口，实际捕获全会话历史仍须正式 application 证明。

提交请求摘要是 SHA-256，frame 如下；epoch 仅供运行时拒绝旧请求，不进入持久摘要。集合按请求顺序，不排序。计数 count32 为大端 u32，文本长度均为 sized 的 u64 字节长度。空集合也写 count32=0。

```text
ASCII("skin-editor.batch.v2") || 00 || commit_id16
|| optional(previous_commit16) || origin_byte
|| optional(head16) || optional(navigation_sha32)
|| address_epoch16 || optional(new_address_epoch16)
|| count32(nodes) || each(node_checksum32 || address_checksum32)
|| count32(blocks) || each(block_sha32)
|| count32(blobs) || each(blob_sha32 || byte_len_u64)
|| count32(tags) || each(node16 || sized(key_utf8) || optional(sized(value_utf8)))
|| count32(object_ids) || each(sized(kind_utf8) || id16 || first_node16)
|| count32(extensions) || each(sized(name_utf8) || byte_len_u64 || raw_sha32)
|| count32(remap) || each(address_checksum32)
|| count32(views) || each(state_view_checksum32)
origin_byte: manual=01, automatic=02, exit=03, metadata=04
```

人工标签元数据提交不能移动检查点。相同 CommitId／摘要返回既有回执；同 ID 不同请求拒绝。无有效变化不增加文件代次。原型 CommitId 去重不替代 API6 的 RequestId 长会话策略，R03 仍待单独处理。

三个指针各有用途：最近文件提交包括标签修改；最近手动检查点决定普通重开状态；最新保护检查点决定主动恢复状态。每个检查点指向头和不可变导航块，后者保存 Undo 记忆的直接子关系并验证。默认打开／无变化关闭不重写较新保护头。自动／退出保护不推进手动位置；无手动位置的新项目只能用保护位置，不伪造保存标记。原型创建 fixture 使用真实实验手动事务建立根；正式未命名项目生命周期尚未接线。

成功后的 main 文件含全部已提交内容。事务中使用 DELETE rollback journal，`synchronous=EXTRA`；中断后可能留下 hot journal，它是恢复所需资料，不能清理临时文件时删掉。未知文件先拒绝自动恢复；已识别文件在明确恢复模式下由 SQLite 回滚。WAL暂不选为当前实验默认，因为已提交内容可能仍在 WAL 中，需要额外处理单文件交付边界。[SQLite 原子提交](https://www.sqlite.org/atomiccommit.html)、[同步等级](https://www.sqlite.org/pragma.html#pragma_synchronous)、[WAL 文件](https://www.sqlite.org/wal.html)说明引擎条件；不等于本项目已经验证断电持久性。

## 6. 恢复、迁移与不得破坏历史

正常打开先检查识别／版本／能力，在同一读取事务内校验结构、完整integrity_check、FK、唯一根、父深度、状态 DAG、引用索引、原始资源流式摘要、节点 frame、对象准入、保存事实、提交链与计数。任何必需资源缺失都诊断，不悄悄降级成缺图或删掉分支。原型普通打开执行全量检查，成功之前不发布可用结果。流式实现减少重复查询／JSON解析，不用只验证当前分支或跳过原字节换取速度；支持阶段进度和取消，SQLite长语句使用progress handler，取消／失败归还读取事务和连接。可信的逐字节副本对照实验仅跳过重复语义扫描，不是产品加载旁路。

受控进程终止分别覆盖开始、写入原始资源后、块后、节点后、标签后、COMMIT 前和已 COMMIT 后。恢复只能得到完整旧代次或完整新代次；原节点不能改变。该实验含缓存溢出导致主文件已写入的中断，但没有覆盖 COMMIT 系统调用内部断电、文件系统／驱动故障、SQLite VFS I/O 故障或真实磁盘损坏。物理磁盘限额由 `max_page_count` 受控模拟，不把它当作真实坏盘。

迁移必须创建另一个受验证候选，保留源文件和全部旧节点／操作／依赖。身份和逻辑内容不得重写成新根；升级表示时要保留可校验的原记录／版本，并验证等价性，再按平台协议发布。原型不实现迁移、自动备份、首次文件候选发布、另存为、修复或整理；尤其不能把它的 O_EXCL 创建直接当作正式首次保存的崩溃协议。未来这些路径也不得缩减既有目标历史。文件备份复制不应在事务中只复制 main 而丢掉 journal。

### 6.1 跨Schema迁移不重绑定历史节点

节点的state永远指向创建时的原始状态块；该块信封的kind／schema及原字节由摘要固定。迁移时保留原块及原始资源，追加新的表示和StateView，不更新nodes.state、父关系、原始操作、record_version或checksum。加载按目标读取能力和明确的converter版本选择表示，不按“最新一条”猜测。

StateView的键为(origin_sha, target_schema, converter_id_with_version)，值为target_sha及绑定校验；新的转换器版本可追加另一表示，不能覆盖旧转换事实。转换器有固定语义与回归样例；必须验证源／目标、资源依赖和语义等价。仅有哈希相等／外键成立不能证明转换正确。失败或无法保持含义时拒绝迁移并保留原件，不删除未知内容。

相同键已存在时优先复用已验证目标，不要求未来编码器产生与Python完全相同的JSON排版。相同含义但不同字节的重序列化也不能覆盖旧映射；可以验证并复用旧目标，不能重新绑定过去节点或偷偷替换转换事实。

```text
state_view_frame = ASCII("skin-editor.state-view.v1") || 00
                   || origin_sha32 || target_sha32 || target_schema_u64
                   || sized(converter_id_utf8)
address_frame = ASCII("skin-editor.address.v1") || 00
                || node_id16 || alias_u64 || branch_u64 || ordinal_u64
各自checksum = SHA256(frame)
```

实验只实现fixture.project的schema1(step)→schema2(edit_index)，保留未知JSON成员和全部资源引用；不是正式SkinProject迁移器。读取迁移表示后再编辑会追加新节点，其原始状态可以是schema2；旧父节点仍引用schema1。操作描述只展示，不重放旧Command。以后改变历史记录结构时保留旧record_version的读取与校验规则，新节点可使用新记录版本；容器／history_schema声明允许的组合和必要能力，不能给旧记录重新编号或重算摘要。

当前未发布实验容器1由旧提交fbea6b3及归档稿保留，新原型明确拒绝直接读写它，不做破坏性就地转换。正式格式发布后，增加版本号必须同时提供已发布版本的读取／无损迁移与fixture；版本号负责选择规则，不能代替兼容实现。暂不能理解的必要版本只读或拒绝，原件不动。

## 7. 资源边界与后续接线

P3-1B已提供[真实完整历史捕获](P3_1_CAPTURE_REVIEW.md)：永久ID与局部地址分离、完整分支／准入／导航截面和独立原字节租约。它没有生成或读取本容器的正式节点记录，也没有恢复活动会话；捕获版本1与容器2／历史2／状态schema分别解释，不能直接把运行source或私有Patch写入文件。

原型默认：原始字节总量 512 MiB／单个 128 MiB，节点 65536，块 250000，逻辑元数据 128 MiB，文件 1 GiB；每批最多 1024 节点／4096 块／4 MiB 元数据；JSON 块 1 MiB／操作 64 KiB／深度 32；每节点 64 用户标签、值 4096 UTF-8 字节；SQLite 页缓存 2 MiB、mmap 关闭、临时内容落文件。分配／写入前检查，额度不足拒绝新增，不能裁剪旧历史。这里只是原型准入，不是最终产品容量或格式数字的永久上限。

逻辑元数据计数包括状态／节点 frame、提交、标签、准入及扩展载荷，不包含所有 SQL 索引页、页空隙、引擎和解释器内存。另测 main 文件、journal I/O、进程私有提交／峰值；不能只报逻辑计数冒充物理资源成本。校验按资源块流式读取，不把 128 MiB 原图整体读进 Python 内存。正式 application 的捕获共享、后台任务队列、GUI／声音组合负载仍未验证。

黄金向量位于 [golden_v2.json](../tools/p3_storage/golden_v2.json)；旧golden_v1保留原字节，不改其历史含义，明确给出 node frame 和摘要，可供未来 Rust 独立实现对照。精确实验 SQL 见 [schema.py](../tools/p3_storage/schema.py)，属于 `p3-0-r2` profile；不要求未来正式 profile 把 Python 内部类或私有数据库连接暴露给产品。正式接入仍为 application 定义窄捕获／保存／恢复端口，infrastructure 实现，desktop 注入；加载通过整体验证后发布新会话，UI／AI 不直接写数据库。
