# P0 可选 Windows 输出适配器

只用于验证真实输出流的设备时钟及 PCM 内容能否驱动编辑器的模拟／输入门控，不是正式音频后端。静音证据见 [TRANSPORT.md](../../docs/p0/TRANSPORT.md)，显式非零输出见 [PCM 记录](../../docs/p0/AUDIO_PCM.md)。

从仓库根执行已验证入口：

```text
python tools/check_p0_gui.py --device-audio
python tools/check_p0_transport.py --fixture artifacts/p0/gpu-fixture --repetitions 3
```

需要 Rust 1.95、MSVC／Windows SDK 和已缓存的锁定依赖；准备方式见[原生实验](../p0-egui/README.md#准备实验依赖)。第一项检查本包及 UI 实验，不打开设备；第二项要求已有 fixture，显式使用默认 shared 静音流和 Vulkan 窗口。默认 UI feature 关闭，无窗口测试验证换算、PCM、格式、故障与生命周期，未覆盖所有原生失败路径。仅 Windows x64 实测，非 Windows stub 不构成平台支持。

本包仅直接依赖 `windows = 0.62.2`（MIT／Apache-2.0），复用旧锁定树的包版本。Foundation、Audio、COM、StructuredStorage 和 Variant feature 提供接口签名／激活所需类型；没有 UI、图像、网络、文件或项目依赖，不访问旧源码或客户端。

安全公共入口只收发值：递增暂停请求、带内容位置的流替换、只读快照和结构化错误。所有 unsafe 限于 `src/platform.rs`：专用 MTA 线程内创建／使用／释放 COM 接口，混音格式通过 CoTaskMemFree 释放；GetBuffer 与 ReleaseBuffer 成对，默认 SILENT 路径不解引用样本指针。显式 PCM 路径先验证 float32 身份／布局和至多 1 MiB 包长，再完整初始化原生字节切片，Release 成功后才推进游标；读取流位置的输出指针为栈变量。接口先释放再结束 apartment，Drop 尝试 Stop／Reset；静音、生成 PCM 和短 Clip 的普通暂停不 Reset，流式路径另见下文。主实验仍 `unsafe_code = "forbid"`。边界检查拒绝反向依赖、额外 API feature、外部 include 和平台文件之外的 unsafe，不代替人工审查或内存安全证明。

成本为一个输出线程、8 项控制队列和一份最新快照，播放或等待预备时每 5 ms／暂停或故障每 100 ms 等待控制。无变化的暂停快照不触发 GUI 刷新。设备失败先发布 Faulted，再在同线程释放旧流，等待显式新代次重建；不开自动重试、不回退墙钟。新流从暂停／请求 0 开始，拒绝旧代次控制；`request` 仅为旧实验保留初始代次入口，新适配使用 `request_in`。旧流释放完才开新流，不并行占用多个流。`invalidate` 只供受控关闭自有流的诊断，不能冒充驱动 HRESULT 或物理拔出。

二十五项无窗口测试覆盖时钟单位／HRESULT、PCM 包、源／转换格式、欠载、失败重试、跨代次拒绝、带位置重开、暂停／播放中设备变化及阻塞清理前通知，另覆盖文件样本、固定块池、背压、准备令牌／迟到结果、EOF 和待启动播放取消。fake stream 不打开真实设备；原生诊断另行执行。`spawn_pcm` 显式输出自制六秒音调，`relocate` 明确指定内容位置；旧底层 `rebuild` 仅为起点 0 的便利入口，预览端口不暴露省略内容位置的恢复。不包含解码器、自写重采样器或游戏混音器；可选系统引擎转换见下文。没有麦克风、loopback、音量／默认设备修改、独占访问或后台服务。100 ms 缓冲及轮询不作为低延迟方案，最终 join 的原生阻塞上界仍未知；不代表 Linux／macOS 后端。

`spawn_clip` 接受已验证的不可变 float32 样本，增益不超过 0.04、单／双声道、最长 6 秒；采样率／声道必须与设备完全匹配。文件读取和解码在独立媒体实验完成，本包仍只依赖既有 Windows bindings，无新增平台 API 或线程。流重建复用同一素材并按显式内容起点定位；证据和内存限制见[文件与时间线](../../docs/p0/MEDIA_TIMELINE.md)。

`buffered::channel` 分开生产端与单接收端；`spawn_buffered` 接收后者，消费固定五块池中的值，不获得文件或解码器。流式暂停 Stop／Reset 清掉原生尾部，以逻辑时钟偏移保持确认位置，并作废排队／在途旧代次；新块预备不自动播放。`Snapshot.buffered` 记录采样当刻的队列／提交统计。解码线程属于外部实验组装，输出线程只准备已验证值并在原生接受后提交消费；中途不足一包且端点无数据会明确故障。三轮真实设备连续段、受控欠载与恢复见[流式输出记录](../../docs/p0/STREAM_OUTPUT.md)。

`spawn_buffered_converting` 显式启用 shared 引擎转换及默认 `eMultimedia` 端点监控，源 PCM 与队列配置不变；`Format.rate/channels` 是客户端格式，`mix_rate/mix_channels` 单列真实混音格式。已锁定绑定内的 `AUTOCONVERTPCM | SRC_DEFAULT_QUALITY` 完成转换，未增加依赖／绑定 feature／线程。原生缓冲超过队列容量会拒绝初始化。约 250 ms 检查默认端点身份与 client 健康，暂停线程等待会拉长至约 300 ms，无硬期限；opaque ID 在同一 MTA 线程复制、比较、释放，不落盘。失效后发布故障并退休旧流，重建重新查询默认混音格式、保持暂停，不自动恢复。十六轮格式矩阵及一次真实模拟耳机失效后恢复通过，资源、听感与未测范围见[完整记录](../../docs/p0/AUDIO_ADAPTATION.md)。这不是产品设备通知框架、全格式转换或系统引擎资源总预算。
