# P2 后代码与架构审阅包

2026-10-30。目标是让下一步实施前能按代码核对职责、对象、方法、引用和约束。当前应用仍是P2内存编辑器；本轮不实现P3、不改历史／地址行为、不自动提交Git。

2026-11-01需求补充：[用户数据与长期兼容](../DATA_COMPATIBILITY.md)优先审阅。首版完整历史／分支持久化已获确认，取代旧会话历史取舍；该要求和具体存储／身份建议分开，代码清单仍为10月30日的实现快照。

建议从以下三份开始共同审阅：

2026-11-02后续：本轮已按授权完成[P3-0定约与独立原型](../P3_0_REVIEW.md)，新增源码／方法与证据在该审阅包列出；下方353份清单仍保留10月30日快照，产品Rust源码未因原型改变，不将新工具倒填为旧快照内容。

同日P3-1A增量：[真实项目状态codec审阅](../P3_1_STATE_REVIEW.md)补列新增Rust对象、方法、trait与依赖；产品infrastructure及application只读重导出已改变。下方全量目录仍为原快照，不能当作包含本轮新增源码的最新全量清单。

1. [系统图与关键对象](SYSTEM_MAP.md)：实际八包、主要持有关系、trait实现、线程／资源所有权和主调用链。
2. [待讨论决定](DECISIONS.md)：短地址／重开、素材导入历史、回执寿命、身份分配、组件和导出等；建议尚未采纳。
3. [全部文档审查与归档](DOCUMENT_AUDIT.md)：逐文档分类、变更及保留理由。

[检查记录与修改边界](VALIDATION.md)说明本轮覆盖范围、实际检查和仍不能据此下结论的部分。

## 全量代码清单

P3-1C新增 application 存储端口、恢复DTO／工厂，以及 infrastructure SQLite／文件／原始资源完整验证的方法与组合关系，见[存储审阅](../P3_1_STORAGE_REVIEW.md)。此增量单列，不重生成旧源码清单或改写旧证明。私有远端与本地空间整理见[Git管理](../GIT_WORKFLOW.md)。

P3-1B新增的捕获／永久身份、共享补丁和租约对象及方法，见[完整历史捕获审阅](../P3_1_CAPTURE_REVIEW.md)；不倒填旧清单或改写旧证明。

清单来自Rust 1.95私有项rustdoc、实际Cargo metadata及源码索引，所有项带源码位置。重导出别名合并，宏／derive生成内容注明来源限制。**Rust没有类继承**：清单列trait／supertrait实现、组合字段和签名引用；Python类另外列真实基类。

当前快照包含771个唯一Rust符号项、1781个关联方法条目，以及353份源码／测试／工具文件。方法表同时列出所属实现，避免将同名trait方法或重导出展示误当成不同业务接口。

| 产品部分 | 完整字段、方法、实现与签名引用 |
| --- | --- |
| domain | [skin_editor_domain](catalog/skin_editor_domain.md) |
| application | [skin_editor_application](catalog/skin_editor_application.md) |
| rendering | [skin_editor_rendering](catalog/skin_editor_rendering.md) |
| audio | [skin_editor_audio](catalog/skin_editor_audio.md) |
| preview | [skin_editor_preview](catalog/skin_editor_preview.md) |
| infrastructure | [skin_editor_infrastructure](catalog/skin_editor_infrastructure.md) |
| ui | [skin_editor_ui](catalog/skin_editor_ui.md) |
| desktop/host | [skin_editor_desktop](catalog/skin_editor_desktop.md) |

[全部源码／测试／Python／C／WGSL／P0索引](catalog/SOURCE_FILES.md)覆盖版本库候选源码；[CODE_INDEX.json](catalog/CODE_INDEX.json)可机器读取，[统计](catalog/SUMMARY.json)给出覆盖数量。外部依赖和构建缓存不复制为自有对象。

## 置信边界

- Cargo图、编译器解析的声明／字段／方法／trait来源可直接追溯；只列自有来源，不把第三方blanket方法重复算作自写接口。
- 签名正／反向引用仅覆盖rustdoc实际解析出的项目类型链接；没有链接不表示没有引用。主调用链经关键入口核对；**不是全程序精确调用图**，不推断动态派发的全部调用者。
- rustdoc使用all-features但不编译cfg(test)；测试、未启用平台分支及P0的Rust条目是屏蔽注释／字符串后的词法位置。Python来自AST，调用拼写不等于运行时解析；C／WGSL只给有限声明索引。
- 本轮是结构／接口／边界审查和文档整理，不是对全部并发交错、算法、设备、许可证或安全属性的逐行证明。已知问题和P2原证据分别保留，不能凭图无环宣称永远无缺陷。

## 复现与维护

使用已有工具链离线生成两个工作区的`cargo doc --workspace --all-features --no-deps --document-private-items`及`cargo metadata --all-features --format-version 1`，再执行：

```text
python tools/review_inventory.py --metadata <核心metadata.json> --metadata <桌面metadata.json>
python -m unittest discover -s tools -p test_check_review_inventory.py
```

默认输出为`docs/review/catalog`，不访问窗口、声卡、网络或用户媒体。快照带源文件哈希，后续代码改变后应重新生成再使用，不把旧行号当成当前事实。项目正式检查仍使用check_core.py／check_desktop.py。

当前架构主文档已整理为[实现与规划分开](../ARCHITECTURE.md)；17份旧文档的原字节备份见[归档清单](../archive/2026-10-30-pre-review/README.md)。所有117份既有证明JSON和P0路径保持，没有删除被测试／证据引用的源代码。
