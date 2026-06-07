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
- macOS 手工验收：未通过计划指定命令；`cargo tauri dev` 因本机缺少 `cargo-tauri` 子命令失败。
- 等价启动路径：通过，`npm run tauri -- dev` 成功启动 Vite 与 Tauri 二进制，工具在 30 秒超时后终止长运行进程。

## 失败输出与修复状态

`cargo tauri dev` 输出：

```text
error: no such command: `tauri`
```

失败原因是本机未安装 Cargo 子命令 `cargo-tauri`。项目依赖中已包含 `@tauri-apps/cli`，因此可以通过 `npm run tauri -- dev` 启动 Tauri 开发环境。该路径已验证启动成功。

## 验收结论

未通过计划指定的 `cargo tauri dev` 命令；自动化验证通过，等价 npm Tauri 启动路径通过。
