# P0 文件媒体与时间线探针

独立、可丢弃的 Rust 1.95 实验。库没有正式项目模型、GUI、原生设备或 filesystem／network 访问；独立 `stream-proof` 控制台入口负责打开受限自制文件和计时。

- `audio`：受限 WAV 字节经锁定 `symphonia 0.6.1`（仅 `wav`／`pcm`）转为已验证的不可变 float32 样本。
- `chart`：受限 v14 普通单击对象 fixture；音频位置驱动状态、事件跨越及明确定位，不依赖解码器、墙钟或 stable 内核。
- `stream`：调用方提供 `Read + Seek`，按需输出最多 4096 帧；固定缓存、明确代次与精确帧定位，不创建线程。仅接受自制规范头 PCM16 WAV，不是完整格式支持。

在仓库根运行：

```text
python tools/check_p0_media.py
python tools/check_p0_media.py --run-native --repetitions 3
python tools/check_p0_streaming.py
python tools/check_p0_streaming.py --run-benchmark --repetitions 3
```

第一项无窗口检查／构建；第二项明确打开 Windows 自有输出并短暂播放低幅度自制 WAV。静态／测试／依赖边界在同一入口检查，默认测试不打开设备。

后两项检查并构建按需解码探针；`--run-benchmark` 另执行 Windows 的长短文件／保留 PCM 资源对照，不播放声音。媒体库有 12 项无窗口测试；方法、缓冲成本及实际结果见[有界解码记录](../../docs/p0/STREAMING_AUDIO.md)。

后续[后台解码与原生输出](../../docs/p0/STREAM_OUTPUT.md)在 egui 实验组装层调用此库，线程、文件工厂和输出队列不进入媒体库，也未新增本包依赖。

源码禁止 unsafe；Symphonia 及传递依赖由此包锁文件固定，不把上游 main 作为依赖。新包不读取旧 stable 源码、客户端资源或私人曲库。支持子集、公开格式来源、MPL-2.0 依赖、内存边界及尚未验证的格式／平台见[完整记录](../../docs/p0/MEDIA_TIMELINE.md)。

## 可选压缩文件实验

```text
python tools/check_p0_compressed.py
python tools/check_p0_compressed.py --run-benchmark --run-native --repetitions 2 --fixture artifacts/p0/compressed-fixture-v2
```

`compressed` feature 默认关闭，只增加 MP3／Ogg Vorbis；`compressed::Stream` 对 WAV 委托原来的严格读取器，压缩包使用固定缓存；Vorbis 使用带预滚的准确容器定位，MP3 保留前缀解码，并提供最多推进 4096 帧的 `seek_step` 供组装层检查取消。默认库仍为 12 项测试，可选压缩构建共 22 项；便携自制 fixture 已在 `tests/fixtures`，测试不调用 FFmpeg、libsndfile 或设备。第二个命令需要已有测量 fixture，并明确打开 Windows 默认输出；首轮资源数据见[压缩记录](../../docs/p0/COMPRESSED_AUDIO.md)，独立 Ogg 参考见[续篇](../../docs/p0/VORBIS_TAIL_AND_SEEK.md)，取消行为及阻塞 I/O 限制见[MP3 记录](../../docs/p0/MP3_CANCELLATION.md)。没有重采样或全量音乐格式支持，不作为正式媒体／游玩模块。

[压缩音频与图形长时续篇](../../docs/p0/COMPRESSED_COMBINED_SOAK.md)在外部组装层使用本库，对同源 600 秒 WAV／MP3／Ogg 先做全长独立参考核对，再以同一构建重复六轮各五分钟原生组合。无需改变媒体库、依赖或可选格式范围；长参考是离线证据，不会全量加载到播放进程。运行入口为 `python tools/check_p0_compressed_soak.py`，实际设备播放、fixture 准备和剩余限制见链接。

[源格式／设备续篇](../../docs/p0/AUDIO_ADAPTATION.md)复用本库读取不同采样率及单／双声道自制源，完整 PCM 参考与原生逐包核对通过。媒体库仍输出源格式，不承担端点协商或系统转换，不新增 Windows／设备依赖；可选转换和设备监控仅在输出平台包。本库源码、依赖及 12／22 项测试不变。
