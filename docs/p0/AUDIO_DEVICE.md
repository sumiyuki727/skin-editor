# 默认音频输出流的暂停与恢复

2026-10-19，独立 Windows x64 WASAPI 静音输出诊断通过三次新建音频流的测试。原始样本、脚本身份和结果见 [windows-audio-device.json](windows-audio-device.json)。它推进了设备输出能力验证，但不是已选定的跨平台音频后端，也没有与编辑器或实际游玩连接。

## 方法与结果

通过默认 multimedia render endpoint 打开 shared-mode 流，读取设备混音格式，按设备允许的空闲缓冲提交静音。请求 100 ms 缓冲；播放约 450 ms，Stop 后观察约 350 ms，再 Start 并观察约 450 ms。三次均停止、清空本流并释放接口，不更改默认设备、音量或系统设置，不使用独占模式。

没有录音、麦克风、loopback 或私人音频输入。静音样本不验证扬声器是否可听，也没有确认默认端点是物理设备还是虚拟端点。

| 指标 | 第一次 | 第二次 | 第三次 |
| --- | ---: | ---: | ---: |
| 停止前流时钟前进 | 457.021 ms | 452.354 ms | 453.771 ms |
| 暂停段流位置跨度 | 0 ms | 0 ms | 0 ms |
| 恢复首次位置跳变量 | 0 ms | 0 ms | 0 ms |
| 恢复观察段前进 | 451.167 ms | 441.354 ms | 445.021 ms |

输出格式报告为 48 kHz、双声道、32 bit、8 字节帧；实际缓冲 4800 帧，设备时钟频率为 384000 单位／秒。时间由 `position / frequency` 得到，不假定设备位置单位就是采样帧。使用高分辨率 `perf_counter` 记录 host 时间，保留位置及关联 QPC；S_FALSE 等非精确位置读数不能通过判定。

预先设置的检查要求：暂停观察至少 300 ms、跨度 ≤2 ms；播放及恢复各前进至少 200 ms；恢复首次跳变量不超过实际 host 间隔＋50 ms，整个恢复段不补跑暂停时间。三次全部通过。设备报告的 GetStreamLatency 为 0，只按原值保存，**没有测得零声学延迟或零输入到声音延迟**。

## 依据与工程边界

微软说明 [Stop 会冻结流时钟，Start 从该位置恢复](https://learn.microsoft.com/en-us/windows/win32/api/audioclient/nf-audioclient-iaudioclient-stop)；[GetPosition 的单位须结合 GetFrequency 使用](https://learn.microsoft.com/en-us/windows/win32/api/audioclient/nf-audioclient-iaudioclock-getposition)。[渲染流接口](https://learn.microsoft.com/en-us/windows/win32/coreaudio/rendering-a-stream)提供缓冲提交能力。这些是平台 API 契约，不是 osu!stable 兼容规则。

工具位于 `tools/p0_audio_device.py`，仅使用 Python 标准库 ctypes 和公开 Windows COM ABI，没有新增包、后台服务或 Rust unsafe，也不引入产品的跨语言核心。平台调用局限在诊断工具中，今后产品仍须通过独立音频端口接入选定后端。默认三次短测试使用轮询，不代表合适的游戏调度或 CPU 预算。

无窗口测试覆盖公开 ABI 的字段宽度／布局，以及暂停漂移、恢复补跑、缺失、非有限和不精确样本不能通过。调用失败保留 HRESULT 与失败状态，不能当作设备缺失或成功。

```text
python tools/p0_audio_device.py
```

入口仅在 64 位 Windows 执行真实设备调用，另建结果目录并保存当次脚本；导入分析器和运行默认工具单元测试不会打开音频。尚未测试可听声音、低延迟事件调度、欠载、设备切换、音频与模拟时钟／输入联动、macOS／Linux 或最小离线谱面流程。

后续进展：[TRANSPORT.md](TRANSPORT.md)记录了可选 Rust 适配器与 GPU 编辑／模拟／输入的同进程三轮验证。本文保留独立 Python 设备诊断的原始范围和测量口径；后续结果不补写到本批证据。
