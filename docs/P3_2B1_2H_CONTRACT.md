# P3-2B1-2H：真实恢复来源与捕获已有元数据

2026-11-18（Asia/Tokyo）。父提交96b3a98086f24b3e0d59b21eb846d911179268a7已推送PUBLIC origin，首次run44136548111三项成功。本轮实际／暂存差异为空，复用回退点；906文件原字节／mtime基线仅本机，schema.rs原标记保留。主线唯一写入及串行构建，两路独立只读审阅先确认本契约及独立向量，再改源码。

## 1. 最小闭合范围

只保管ExistingVerified文件在完整只读恢复时实际读得的不可变容器features来源，沿detached RestoreCandidate→新Session→CaptureSnapshot资源保管→真实SavePreparationWork→不透明q传递。计算捕获已有ArchiveMetadata加该来源全部features的FileCoord-Proof-v1域3（before），与G的选定历史metadata比较。未知optional、required标志及UTF-8原字节全部参与，不以writer四项默认值充当来源；optional版本允许空，名字允许旧格式接受的NUL，不增加归一化。

不决定首次保存profile、能力编辑／升降级、迁移、另存、跨Reader转移或新回执后继承。fresh-append完整metadata、旧would_commit子集及F.metadata均不升级，F继续NotCompared；相等仍只得Experimental／InsufficientForSaveReconciliation，不安装事实、不产PreparedWitness或NoChange／Unknown、不进入实际保存／B2。

## 2. 联合只读恢复与来源

新增独立FileProfile1 opt-in入口；旧inspect与standalone load行为不变、无来源。新入口复用同次只读scratch的完整full、G、D严格原件验证、逻辑绑定、源摘要／身份复核及cleanup。它不调用会在末尾打开READ_WRITE的旧load。使用调用方BlobStore，Small的8原件／256KiB等上限保持，真实candidate pins保管原件，不复制无计量的资源库。

所有步骤成功后，另预留并复制已验证Cache.features到全新受管载荷；旧Vec、Cache／JSON／恢复模型仍属旧full未证明成本，不改名冒充已计费。结束原文件guard后才联合发布candidate与origin，避免Session永久锁住将来写入；origin只证明那次实际读取，未证明未来物理连续性。

origin固定记录版本1、完整原FileFactsRequest及ObservationId、持久root、opening selected head、实际latest features_root及原文件SHA256。opening Manual A与最新保护C分开：RestoreHeader.metadata、features来源均为C，不把A的features当当前来源。原read sequence与新q sequence分别保留，新Session epoch必须等于origin请求预定的session。

## 3. 核心端口与隔离

application定义独立版本1的Any+Send+Sync只读FileProfileSource窄端口；不导入infrastructure、路径、文件句柄或SQLite。detached candidate只允许一次附着，核对版本／project／history／root／opening head／latest expected及逻辑容量；激活核对新session。活动Session无setter，getter只借用、不暴露可克隆Arc。私有共享附件保留附着时origin和逻辑成本。

infrastructure只接受私有实际provider类型：通过trait向Any安全上转并downcast_ref，禁止可伪造as_any。外部假provider即使POD相同也不能取得比较资格；伪造公共origin／summary不转换为真实q。真实来源不暴露所有权，仍不可替换。

prepare_save在真实Work的Snapshot读取来源；同Reader、完整原target binding（含expected）、project/history/root和session匹配才计算before。无来源、非实际类型或不相容来源明确Unavailable；旧入口保持可用且Unknown。A确认真实新回执使expected C→D时，原来源C不改写，新q须Unavailable，不能默默继承；旧q继续只对应原Ticket。

新增独立版本1比较接口，仅接受不透明FileSaveCandidate。先复用完整Ticket／CaptureSource／binding／观察／Reader／root守卫，再核对完整FileFactsRequest并取G选定commit；Missing与Unavailable同样必须过守卫。固定结果分别保管captured来源／before摘要与selected G事实，metadata_equal仅双方存在时Some(bool)。观察中最新features不得替换q的来源。所有既有API7／捕获3／协调1、持久文件／记录／证明及旧公开结构保持。

## 4. 预算与实际所有权

来源最多256项；每项name≤256B且非空、version≤64B（可空），原文本结构上界81920B。固定256×8B索引保存u32起点、u16名字长、u8版本长和bool required；一个全新Vec精确预留原文本容量并核对实际capacity。Box载荷、Arc包装／计数、固定构造重叠和全部原文本在分配前合并预留，字段顺序保证实际Box／Vec先释放再退额。共享附件只计一次，Session／Snapshot／Node／Work最后实际持有者释放后退额；取消或Ack本身不退还仍持有的来源。

Reader预算512KiB与A控制512KiB独立，不变更旧2Data／D+G计费或旧回归。具名来源固定保守费用为4×size_of(Payload)+2×size_of(Provider)+Arc两AtomicUsize计数与对齐余量，再加实际text capacity；附着及捕获逻辑预算另计共享来源与句柄。实际合法最大文件为四固定required（62B）＋252个256/64B optional，即80702B文本；81920／81921仅作内部容器边界，不能冒充完整文件准入。Session仍持来源只保持来源租约；q／Work及外部capture消费者实际释放后，保存Ack可Released，不等待Session销毁。

保留旧无来源两观察＋q，及典型4／5项来源两观察＋q聚合回归；最大真实来源＋q＋一观察须在512KiB内。最大来源再申请第二观察可明确Limit(Control)，不承诺所有最大载荷同时驻留；失败保留旧source／q／view。已有管理计数基线root16384、单观察retained136352／peak347344、双观察retained256320／peak467312，仅是现有管理准入计数，不是分配或性能测量。

共同64MiB、旧Cache／RestoreNode／codec／JSON／Vec／Map、PNG no_limits、SQLite／VFS、ABI栈／RSS及无限宿主聚合仍未证明。

## 5. 冻结回归与交付

实现前生成独立Python结构模型／完整TLV／SHA：保留G原黄金，加相同计数下optional name、version、required变化及UTF8／NUL／空版本向量，before与最新C精确匹配、与旧A不同。

真实fixture覆盖Manual A／Exit C来源分离、标签／二进制扩展、同内容／原件／导航／legacy但features不同、旧q对后来观察不换源、expected前进Unavailable、旧入口None、假provider、版本／session／target／Reader／观察／Missing隔离。覆盖源复制及before编码完成后取消、cleanup失败、激活失败、Session／Reader先释放而Snapshot／Node仍存活、实际Drop／Ack及预算N／N−1；保留所有旧测试和黄金。

正常完整hook、精确路径PUBLIC推送、新完整SHA首次三项CI及远端受控树核对。25分钟CI及30秒门控不变，旧first-before及各次取消根因Unknown，不追认。仅自制有界fixture及显式测试根，不碰真实用户内容；用户归期未知，UAC／截图／现场测试等待，UI／Skill暂停。新增忽略原始日志仅本机，不扩充旧私有快照。K1／Reference10000／Tiny4、产品NotEvaluated／stable Unknown保持。
