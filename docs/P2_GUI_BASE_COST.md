# P2-A3：正常刷新下的呈现基础成本

后续[当前构建原生对照](P2_GUI_BASE_NATIVE.md)已在授权中断后续跑完成。候选32 Voice仍为19.6536%／19.9927%单核，未低于基线；零欠载、提交延迟及资源清理通过，P2-A3仍未验收。以下首次UAC取消记录和原证明保持原样。

2026-10-25。宿主呈现优化及原生像素／租约检查已完成，当前构建已冻结。**性能对照第一次在UAC阶段被Windows以1223取消，尚未启动参考或GUI；CPU、欠载与20 ms复测没有新成绩，P2-A3未验收。** 前轮有效CPU17.8077%–18.4510%及未复现的203次／秒记录见 [重绘来源](P2_REPAINT_ORIGIN.md)，不追认通过。39份旧证明保留，原声音供给和门槛不变，不自动提交Git。

本轮结构化记录见 [P2_GUI_BASE_COST_PROOF.json](P2_GUI_BASE_COST_PROOF.json)；未执行的性能字段保持未知，不记为0或通过。

## 实现

`NativeHost` 的Frame描述为宿主独有值；UI仍只取得不透明FrameKey及描述副本。GPU绘制和完成回执改为持有不可变 `Arc<NativeTarget>`，不再持有包含场景／时间描述的Frame。静态素材的时间／场景别名沿用原 `material_equal` 范围，只更新活动描述，不再每帧创建Arc<Frame>、登记Weak<Frame>或退休同一纹理。没有修改SkinProject、历史、场景模拟或stable规则。

退休列表现在追踪真实纹理Target的Weak，未完成合成的Target另有提交租约。旧CPU绘制和未完成GPU读取仍持有原Target，描述更新不能改变其像素或提前退还预算；活动、候选、退休及提交中的Target按分配身份计量。

绘制完成回执直接登记到egui传入的主CommandEncoder，`prepare` 返回空的附加缓冲列表，移除原来每帧为回执单独创建的空编码器／command buffer。依据为锁定 [wgpu 30.0.1 CommandEncoder](https://docs.rs/wgpu/30.0.1/wgpu/struct.CommandEncoder.html#method.on_submitted_work_done) 的延迟完成回执接口，以及本机egui-wgpu 0.37.0的CallbackTrait编码契约。回执仍等待实际绘制提交完成，不以UI刷新或TextureId代替所有权。

改动限定于宿主呈现和诊断工具；UI库、声音供给、混音、后台时间线、刷新间隔、着色器、合成、采样、MemoryUsage配置及依赖不变。突增来源观察保留，未增加窗口捕获或项目写入口。

## 检查与像素

681项默认检查通过（218 Python＋130核心Rust＋333桌面Rust），含格式、Clippy和边界。新增四项Python回归覆盖A/B/B/A完整性、候选失败、额外／重排结果及自有基线的入口线程身份；既有工具默认路径保持兼容。

原生 `--verify-gpu-memory` 先后验证保存的基线与当前候选：

- 七组自制素材合成结果逐字节一致，CPU／GPU最大通道差均为1（门槛2）。
- 整张1180×800编辑器回读PNG也逐字节一致；没有系统窗口捕获。
- 新增64次时间／场景别名检查：旧描述副本不变，Target及源上传数不增加。
- UI编码不再返回附加command buffer；准备之后、提交之前切换素材并丢弃CPU绘制数据，旧Target租约仍保持，完成后退休资源消失。
- 暂停下草稿刷新、取消精确恢复、历史分支恢复、旧viewport拒绝、源复用与关闭清理通过；关闭后仍持有的CPU绘制可追踪，释放后Target／源缓存／暂存预算归零。

两组均为RTX2060，合成分配8、发布8、源上传1；没有新增GPU dedicated／shared预算或300秒成绩。候选附加结构化检查保存为 `presentation_checks.json`。

像素轨迹SHA-256：`bee8433fe9c6c6530704fbd6f5ca794e4b900ca16ea5371a14c3f922bceb2270`；编辑器PNG：`d96c84434c26d7083c799cabc0e4770196f63ad6bd491c2eff0d97291e6db04e`。基线release为 `da2621e20748c99c9436bfc581560d766e2b73248c965d84e860fe9790e86405`；候选两次release构建一致，为 `a3f0d09b2d30e203ed6ba051a5a9552e02028112007ec6a47493c1d98a69f0ee`。源码、二进制与原始记录保存在 `artifacts/p2/gui-base-cost/`。

## 性能计划与当前限制

`tools/measure_gui_base.py` 固定运行基线／候选／候选／基线，四轮均启用相同的main边界和重绘来源观察，保持相同可执行文件名。普通runner拥有测试进程；严格参考独立停止、排空、全部通过之后才允许产品准入。UAC仅用于两个采集器。每次准入前后核对冻结二进制与源码，不在测量后追加测试或重建以混淆构建身份。

基线检查原样保留；候选的全部稳定阶段须同时满足单核CPU≤15%、零欠载、提交p95≤20 ms及资源／来源检查，不能挑选候选窗口或扣除诊断成本。固定四轮用于小范围对照，不能替代两轮300秒标准组合。

当前1223取消发生在参考采集器授权之前，没有新的CPU改进比例、32 Voice欠载或提交延迟数据。性能效果仍待原生对照，不以去掉两处分配推导已达到预算；此前CPU失败保持开放。
