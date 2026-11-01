# P2-A1：独立音效模型与输出计划

2026-10-21，Windows。用户审查的两项修订及后续澄清已纳入[方案v2](AUDIO_EDITING_REVIEW.md)、需求／架构／路线和AGENTS.md。本轮只关闭A1的模型、命令／历史、纯处理及隐藏像素部分，**完整A1和实时音频仍未验收**。

后续已完成[候选激活、双通道与旧声音清理](P2_AUDIO_RUNTIME.md)，以及[通用资源、共享PCM预算和准备准入](P2_AUDIO_RESOURCES.md)的确定性增量。以下数字保留前一轮模型／图形检查身份；最新结果见后续记录，真实codec／原生输出及文件导出仍待验收。

## 本轮实现

| 内容 | 已实现行为 |
| --- | --- |
| 独立音效组件 | AudioComponentId、项目内唯一且≤128字节的SoundSlotKey、一个Audio／Opaque Asset引用、增益0–1及可见性；素材允许复用，不存在图片—声音关系表 |
| 活动项目写入 | 创建／删除音效组件、替换素材、修改增益／可见性、整个图片文档可见性均经Command；图片与声音可组合事务，失败一起回滚 |
| 引用与元数据 | 拒绝删除被音效引用的素材、被输出引用的音效；新typed Audio通过ResourceStore只读已验证索引核对，已固定Opaque字节也不能绕过检查 |
| 历史与草稿 | 音效差异补丁、撤销／重做／检出／分叉、跨分支原始租约；300次增益草稿一次提交，取消／失败／无效操作不增加节点；Busy／冲突／请求去重沿用统一入口 |
| 纯声音计划 | 新audio库只依赖domain，无第三方依赖、I/O、线程或设备；SoundPlan v1共用增益处理，先检查样本／容量，不修改源PCM |
| 未编辑音频 | 可见且增益1选择CopyOriginal；即使Opaque无法解码，返回相同字节切片，不解码或重编码。调用方须已核验摘要和长度；实际磁盘复制未实现 |
| 已编辑／隐藏音频 | 增益修改请求PCM处理，缺少可处理元数据明确报错；隐藏选择EncodeSilence，不返回原文件／遗漏。真实静音容器、扩展名与时长策略待验证，不能写零字节代替 |
| 图片隐藏 | 子图层／文档内组只移除该子树；整个ElementDocument隐藏保留画布／输出映射并合成透明像素，CPU及GPU空计划不需要原图像素 |
| 输出边界 | OutputSource::Image／Audio共用大小写及文件目录冲突检查；未知音频用途不能成为后续导入／导出漏文件的理由 |

本轮不新增声音节点到视觉树，视觉组整体变换保持原有语义。跨元素视觉组的组级隐藏尚未实现；后续须逐成员保留透明图片输出，不影响声音。

## 版本与维护边界

- MODEL_VERSION 1→2，application API 4→5；旧请求明确拒绝，FrameDescriptor v1和有界历史地址不变。
- `ProjectDefinition::upgrade_v1`只转换分离值，保留Opaque、未知配置、图片输出身份／路径和文档；新集合为空。未知版本或v1中夹带v2内容拒绝。它不是磁盘迁移或活动项目替换接口。
- 根工作区仍只有domain／application两个无第三方包。桌面增加audio后共六个成员；当前宿主未调用audio库，未引入编解码／设备后端。
- 依赖检查明确拒绝UI／application→audio、audio→infrastructure及纯audio中的文件／线程等能力。通用原始Blob存储与RGBA／PCM缓存分离仍待A1／A2接入。

## 已执行检查

运行环境、源码／锁／日志摘要与GPU报告见[P2_AUDIO_CORE_PROOF.json](P2_AUDIO_CORE_PROOF.json)。旧[P2_NATIVE_PROOF.json](P2_NATIVE_PROOF.json)保留原身份。

| 检查 | 结果与范围 |
| --- | --- |
| `python tools/check_core.py` | 32项Python、114项Rust行为、6项编译拒绝、1项可运行文档，共153项；格式、Clippy、release核心库及依赖边界通过 |
| `python tools/check_desktop.py --run-native --output artifacts/p2/audio-core-native` | 24项桌面行为：audio5、host2、infrastructure2、preview1、rendering7、UI7；格式、Clippy全目标、debug构建及依赖边界通过 |
| 自有Vulkan回读 | RTX2060；隐藏整个图片元素的CPU／GPU结果均为全透明，其他合成最大通道误差1/255（容差2）；暂停草稿、取消精确恢复、分支及过期视口回归通过 |

默认无窗口检查合计177项，比旧P2-1新增23项。它们没有验证真实音频解码、静音文件编码、声音播放或声学延迟；纯数组处理不是PCM设备输出证据。性能预算、人工UI操作、其他平台支持和远程CI也不能由测试数量替代。

SA02／03／08／09的项目侧、SA04的增益处理、SA17的输出冲突及内存升级、SA19的字节选择、SA20的图片像素／音频计划已有回归。SA01实际导入、SA06／07／10–16运行时、SA17／19／20真实文件及SA18 stable行为仍开放；没有将整个SA编号标为通过。

## 下一增量

1. 候选激活／迟到拒绝、双通道、草稿取消／历史清理及固定Voice生命周期已补确定性测试，见[运行记录](P2_AUDIO_RUNTIME.md)；通用原始存储、共享PCM预算与有界工作单元准入见[资源记录](P2_AUDIO_RESOURCES.md)。
2. A2接入真实WAV探测／解码、单worker执行及原生交接，评估一个可维护输出后端，验证最低格式矩阵、热替换、双通道及实际短音软件提交；再提供音效编辑UI。
3. A3验证长时声音＋图形预算／生命周期；P3补固定节点文件保存／导出、未经编辑音频原字节哈希、有效静音／透明占位及失败保护。stable文件规则按证据逐批实现。

本轮保留此前P2未提交变更；没有暂存、提交或推送Git，没有修改旧stable源码、用户客户端或系统音频设置。
