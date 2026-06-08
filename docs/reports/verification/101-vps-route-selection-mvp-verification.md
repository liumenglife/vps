# VPS 线路选择工具 MVP 验证报告

## 验证命令

- `cargo test --manifest-path src-tauri/Cargo.toml`
- `cargo check --manifest-path src-tauri/Cargo.toml`
- `npm run build`
- `cargo tauri dev`
- `npm run tauri -- dev`

## 验证结果

- Rust 测试：通过，`cargo test --manifest-path src-tauri/Cargo.toml` 全部测试通过。
- Rust 编译：通过，`cargo check --manifest-path src-tauri/Cargo.toml` 退出码为 0。
- 前端构建：通过，`npm run build` 完成 `tsc && vite build`。
- macOS 手工验收：通过，`cargo-tauri` 已安装为真实二进制，`cargo tauri --version` 输出 `tauri-cli 2.11.2`。
- GUI 启动：通过，用户在终端执行 `cargo tauri dev` 后成功打开 `vps-selector` GUI。
- 主会话复核：通过，`cargo tauri dev` 已进入 Tauri 启动流程；因用户终端已有 Vite 服务占用 `127.0.0.1:1420`，复跑时报告端口占用。
- 等价启动路径：通过，`npm run tauri -- dev` 可启动 Vite 与 Tauri 二进制。

## 修复状态

此前 `cargo tauri dev` 失败输出：

```text
error: no such command: `tauri`
```

失败原因是本机未安装 Cargo 子命令 `cargo-tauri`。用户已执行：

```bash
rm -f ~/.cargo/bin/cargo-tauri
cargo install tauri-cli --version "^2.0.0" --locked --force
cargo tauri --version
```

修复后 `cargo tauri --version` 输出：

```text
tauri-cli 2.11.2
```

主会话复核 `~/.cargo/bin/cargo-tauri` 为 Mach-O 可执行文件，不再是临时包装脚本。

## 验收结论

通过。
