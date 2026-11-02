# P3-0 独立单文件存储实验

这是 Python 标准库 sqlite3 的有界实验，不是产品核心或正式保存实现；不被任何 Rust 包导入，不增加 Cargo 依赖，不读用户资源，不打开窗口／GPU／音频／网络。Python 3.9+；已在 Windows 的 SQLite 3.35.5 与 3.53.1 执行。Linux／macOS 未实测。

在仓库根运行：

```text
python -B tools/check_p3_storage.py
python -B tools/check_p3_storage.py --bench artifacts/p3-0/my-new-benchmark
python -B tools/check_p3_storage.py --crashes artifacts/p3-0/my-new-crash-test
python -B tools/measure_p3_load.py --input artifacts/p3-0/my-new-benchmark/scale.oskproj --output artifacts/p3-0/my-new-load
```

输出目录必须是 artifacts/p3-0 下尚未存在的子目录；不会覆盖旧结果。默认只跑自制临时 fixture 的无窗口确定性单元测试；核心正式检查也会发现 `test_check_p3_storage.py`。bench 会写约 128 MiB 自制数据、两份约 144 MiB 数据库及复制对照，并全量开关十轮；不是 GUI／声音性能验收。crashes 只终止自身创建且已到达确定性门闩的七个子进程，不使用 sleep 猜测竞争。等待门闩上限 30 秒。

当前为容器2／历史2的p3-0-r2 profile，新输出使用.oskproj。旧实验容器1明确拒绝直接读写，来源及结果仍在Git提交fbea6b3和旧证据中，不做就地覆盖。

文件职责：identity.py定义身份／地址；values.py定义记录、独立地址绑定、StateView和批次；schema.py定义SQL；store.py执行事务及全量验证；fixtures.py生成自制输入；metrics.py采集资源。golden_v1保留旧字节，golden_v2校验当前frame，测试不会重新生成它们。Node是永久记录和地址绑定组合的查询条目，node.record()不含显示地址。没有真实ProjectSession，Store.captured只是实验捕获值。

measure_p3_load在同一份已验证fixture上交替进行五轮reference／streaming完整打开，另测SQL语句数和cProfile；诊断开销不混入延迟。两者均执行integrity_check、FK、所有块／节点／地址／资源／标签／检查点和已登记迁移表示验证；不跳过旧分支或大资源。SQLite长语句和逐阶段循环可取消，验证持有同一读取事务。不是清空OS缓存后的冷盘测量，也不接GUI／音频。

SQL 连接和故障 hook 属于实验内部接口，不是给 UI／AI 的写入端口。schema 中 fixture.project 没有完整 SkinProject codec；实验成功不表示用户项目已经可以保存／恢复。`Store.open` 默认全面验证，`verify=False` 只用于工具中逐字节复制的可信 fixture 对照，不得复制为正式加载旁路。

约定和成本／限制分别见 [格式](../../docs/PROJECT_FORMAT_V1.md)、[审阅包](../../docs/P3_0_REVIEW.md)。成功关闭后是完整 main 文件；崩溃遗留 journal 必须保留至 SQLite 恢复，不能手动清理。没有迁移／备份／正式首次发布协议、物理断电保证或正式保存 UI。
