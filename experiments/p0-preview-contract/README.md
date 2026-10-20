# P0 预览契约探针

独立编写的可丢弃实验，仅使用 Rust 标准库。它不是编辑器、stable 内核或 GUI 候选原型，不读取旧版源码、真实客户端或用户素材，不实现任何 stable 兼容性规则。选择 Rust 是为了利用本机可用工具链验证架构契约，不代表产品技术栈定案。

验证范围：逻辑时钟与模拟音频游标冻结、暂停后替换合成输入仍改变像素、输入隔离、恢复不补跑时间、场景控制、按需刷新、过期帧拒收及修订冲突。RGB 图案的尺寸和位置完全为测试自定义。

音频游标只是逻辑数值，输入是模拟按钮调用。没有真实音频设备、键盘事件、GPU、窗口、skin.ini 或谱面。10 个集成测试通过不能被描述为游戏兼容或性能达标。

## 运行

需要 Rust 1.80+ 的 rustc、Cargo、rustfmt、clippy，以及本机目标的链接工具。无第三方 Rust 依赖，无需网络；Cargo.lock 已保留。

在仓库根执行完整检查：

```text
python tools/check_p0.py
```

脚本使用 Python 3.9+ 标准库；Windows 下自动发现已有的 MSVC x64 工具及 Windows SDK，仅给子进程设置 LIB 和链接器路径，不安装或修改系统工具链。已在本机 Windows x64 执行；macOS/Linux 尚未执行。

在已经配置好原生链接器的开发环境中也可直接运行：

```text
cargo fmt --manifest-path experiments/p0-preview-contract/Cargo.toml -- --check
cargo clippy --manifest-path experiments/p0-preview-contract/Cargo.toml --offline --locked --all-targets -- -D warnings
cargo test --manifest-path experiments/p0-preview-contract/Cargo.toml --offline --locked
cargo run --manifest-path experiments/p0-preview-contract/Cargo.toml --offline --locked --quiet
```

完整检查还会验证实验是单独工作区、依赖为空、不通过 include/path 引入外部源码或资源。这个范围很小的隔离检查不替代未来正式工程的模块依赖规则、版权审查或完整来源检查。

控制台场景输出 JSON：在 123 ms 暂停，替换材质并验证像素改变，60 秒后恢复再推进 10 ms，模拟时间应为 133 ms。没有输出 FPS、内存排名或技术栈选型结论。
