# VPS 线路选择工具 MVP 验证报告

## 验证命令

- `cargo test --manifest-path src-tauri/Cargo.toml`
- `cargo check --manifest-path src-tauri/Cargo.toml`
- `npm run build`
- `cargo tauri dev`
- `npm run tauri -- dev`
- `cargo test --manifest-path src-tauri/Cargo.toml --test business_e2e_test -- --nocapture`
- `npm run e2e`

## 验证结果

- Rust 测试：通过，`cargo test --manifest-path src-tauri/Cargo.toml` 全部测试通过。
- Rust 编译：通过，`cargo check --manifest-path src-tauri/Cargo.toml` 退出码为 0。
- 前端构建：通过，`npm run build` 完成 `tsc && vite build`。
- macOS 手工验收：通过，`cargo-tauri` 已安装为真实二进制，`cargo tauri --version` 输出 `tauri-cli 2.11.2`。
- GUI 启动：通过，用户在终端执行 `cargo tauri dev` 后成功打开 `vps-selector` GUI。
- 主会话复核：通过，`cargo tauri dev` 已进入 Tauri 启动流程；因用户终端已有 Vite 服务占用 `127.0.0.1:1420`，复跑时报告端口占用。
- 等价启动路径：通过，`npm run tauri -- dev` 可启动 Vite 与 Tauri 二进制。
- 业务 E2E：通过，`business_e2e_test` 从中文 TOML fixture 解析三地真实候选 IP，调用 `run_probe_once`，生成 metrics、scores 和 Markdown 报告。
- GUI 页面验证：通过，Playwright CLI 使用本机 Chrome 访问 `http://127.0.0.1:1420/`，覆盖配置页、测试中细节页、报告导出反馈和普通浏览器失败反馈，控制台无错误。
- 探测边界：通过，`traceroute` 已限制等待与最大跳数，真实公网 IP E2E 不再被系统命令长时间阻塞。
- 排名算法：通过，时段统一按北京时间 UTC+8 判定；单时段报告为“白天单次综合排名”或“晚上单次综合排名”；同一对比周期具备白天与晚上样本时输出“白天与晚上交叉验证综合排名”。
- 历史缓存：通过，使用 24 小时滚动对比周期、48 小时保留，并按候选集合与评分配置隔离缓存文件。
- Markdown 导出：通过，已接入 Tauri 原生保存对话框和文件写入权限；普通浏览器环境下给出中文失败反馈。
- 实时探测日志：通过，后端真实探测流程向前端发送 `probe-log` 事件，测试页展示 traceroute、ICMP、TCP 和目标完成日志；静态“阶段说明”已移除。
- 稳定性评分重构：通过，删除基础可连接性稳定性权重，稳定性改为连续失败、丢包率、延迟抖动三项，并支持负向激励。

## 业务 E2E 覆盖

新增测试：`vps-selector/src-tauri/tests/business_e2e_test.rs`。

覆盖链路：

1. 读取 `tests/fixtures/valid-config.toml`。
2. 使用 `parse_config` 解析中文 TOML 配置。
3. 将运行态探测参数压缩为快速真实探测：`default_duration_minutes = 0`、`icmp_interval_ms = 1`、`tcp_timeout_ms = 300`、`concurrency = 3`、默认端口 `[22, 443]`。
4. 调用 `scheduler::run_probe_once(&config).await`。
5. 断言 `metrics` 非空、`scores` 非空。
6. 断言 Markdown 报告包含 `VPS 线路测试报告`、`排名`、`IP 详情`、`推荐结论`、单时段或交叉验证综合排名标题、`192.3.81.8`、`纽约`、`107.174.51.158`、`洛杉矶 DCO3`、`198.23.228.15`、`伊利诺伊州芝加哥`。

验证输出：

```text
running 1 test
test runs_business_e2e_from_chinese_toml_to_markdown_report ... ok
test result: ok. 1 passed; 0 failed; finished in 24.24s
```

该测试证明配置解析、真实公网候选 IP 探测调度、指标聚合、评分和 Markdown 报告生成链路可以跑通。为保持稳定和快速，它不执行 10 分钟长时探测质量评估。

## GUI 验证证据

- 测试工具：Playwright CLI，本机 Chrome channel。
- 测试文件：`vps-selector/tests/e2e/gui-flow.spec.ts`。
- 覆盖内容：配置页加载、三地真实候选 IP 展示、`校验配置` 按钮、`立即开始测试` 按钮、测试中页北京时间与配置细节、实时探测日志、其他时段展示、配置解析失败提示、导出报告中文反馈、普通浏览器环境下中文失败反馈。
- 控制台：无错误。

验证输出：

```text
Running 8 tests using 1 worker
8 passed (20.0s)
```

## 排名与缓存规则

- 白天/晚上统一按北京时间 UTC+8 判定，不受本机时区影响。
- 仅覆盖白天时，报告输出“白天单次综合排名”。
- 仅覆盖晚上时，报告输出“晚上单次综合排名”。
- 同一对比周期内同时具备白天与晚上样本时，报告输出“白天与晚上交叉验证综合排名”。
- 历史缓存使用 24 小时滚动对比周期，保留 48 小时。
- 缓存文件按候选 IP、城市、有效端口、时段配置和评分权重隔离，避免不同配置互相污染。

## 导出与测试中细节

- 导出 `.md` 使用 Tauri 原生 `dialog.save` 与 `fs.writeTextFile`。
- 导出成功、取消、失败和空报告均有中文反馈。
- 测试中页面展示北京时间、判定时段、候选 IP 数、默认端口、预计测试分钟数、ICMP 间隔、TCP 超时、并发数、候选 IP 列表和阶段说明。

## 实时探测日志

- 后端探测流程通过 `probe-log` 事件输出运行态日志。
- 日志覆盖目标开始、traceroute 开始/完成/失败、ICMP 样本、TCP 端口结果和目标完成摘要。
- 前端等待事件监听注册完成后再启动探测，避免漏掉首批日志。
- 页面离开或重渲染时清理事件监听，避免重复日志。
- 普通浏览器环境显示 `等待 Tauri 探测日志通道`，不阻塞自动化测试。

## 稳定性评分重构

- 删除基础可连接性稳定性权重。
- 稳定性由连续失败、丢包率、延迟抖动组成。
- 连续失败、丢包率、延迟抖动支持负向激励，内部子项允许 `-100..100`。
- 最终展示稳定性分限制在 `0..100`。
- ICMP 成功样本数为 `0` 或 `1` 时直接重罚，不再把这类极差信号当作普通抖动缺失。
- 报告仍保留可连接率作为辅助观察指标，但它不参与稳定性评分。

## 真实候选 IP

- 纽约：`192.3.81.8`
- 洛杉矶 DCO3：`107.174.51.158`
- 伊利诺伊州芝加哥：`198.23.228.15`

## Traceroute 卡死修复

- 根因：真实公网 IP E2E 调用 `traceroute` 时没有等待和最大跳数边界，系统命令可能长时间阻塞。
- 修复：`traceroute` 调用增加 `-w 1 -m 8`，将每跳等待和最大跳数限定在可控范围。
- 回归测试：`probe::tests::traceroute_args_bound_wait_time_and_hop_count`。

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

最终验证命令均通过：

- `cargo test --manifest-path src-tauri/Cargo.toml`
- `cargo check --manifest-path src-tauri/Cargo.toml`
- `npm run build`
- `npm run e2e`
