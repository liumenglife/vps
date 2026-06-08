# 历史任务归档 (Task History)

## 1. 归档规则
- 这里保存所有已经从 current.md 迁移出来的任务批次。
- 每个批次都要保留原始状态，至少包含 `[✓]`、`[•]` 和 `[ ]` 标记。
- 不要把历史批次继续留在 current.md。

## 2. 历史批次

### 2026-06-07 批次：规划锚点初始化
- [✓] 初始化 `docs/planning/current.md`、`docs/planning/history.md`、`docs/planning/decisions.md`。
- [✓] 读取 `AGENTS.md` 与项目规范。
- [✓] 确认后续开发以 planning 文件作为主线真相。

### 2026-06-07 批次：Task 1 Tauri 脚手架与最小运行
- [✓] 创建 `vps-selector/` Tauri v2 vanilla TypeScript 脚手架。
- [✓] 安装 npm 依赖并生成 lockfile。
- [✓] 按计划配置 Rust 依赖：`tauri`、`serde`、`serde_json`、`toml`、`tokio`、`thiserror`、`chrono`。
- [✓] 修复脚手架安全与语言规范：启用最小 CSP、关闭全局 Tauri API、移除未使用 opener 插件、将 README 改为简体中文。
- [✓] 规格审查通过。
- [✓] 代码质量审查通过。
- [✓] QA 验证通过：`cargo check --manifest-path src-tauri/Cargo.toml`、`npm run build`。
- [✓] 已提交：`5c77c9b feat: 创建 VPS 选择工具脚手架`。

### 2026-06-07 批次：Task 2 中文 TOML 配置解析与校验
- [✓] 创建 `error.rs` 并定义中文 `AppError`。
- [✓] 创建 `config.rs`，实现中文 TOML 结构体、解析和校验。
- [✓] 创建配置测试与 `valid-config.toml`、`invalid-weight.toml` fixtures。
- [✓] 规格审查通过。
- [✓] 代码质量审查通过。
- [✓] QA 验证通过：`cargo test --manifest-path src-tauri/Cargo.toml --test config_test`、`cargo check --manifest-path src-tauri/Cargo.toml`。
- [✓] 已提交：`88d3919 feat: 实现中文配置解析`。

### 2026-06-07 批次：Task 3 核心模型与探测解析
- [✓] 创建 `models.rs`，定义探测样本、TCP 结果、目标指标和目标评分模型。
- [✓] 创建 `probe.rs`，实现 ping 延迟、丢包率、traceroute 跳数解析、TCP 探测和 macOS 命令探测函数。
- [✓] 创建 `probe_parse_test.rs`，覆盖 macOS ping、丢包率、traceroute 解析。
- [✓] 修复命令失败错误信息中文化。
- [✓] 规格审查通过。
- [✓] 代码质量审查通过。
- [✓] QA 验证通过：`cargo test --manifest-path src-tauri/Cargo.toml --test probe_parse_test`、`cargo check --manifest-path src-tauri/Cargo.toml`。
- [✓] 已提交：`e4559c2 feat: 实现探测解析模型`。

### 2026-06-07 批次：Task 4 指标聚合与评分引擎
- [✓] 创建 `metrics.rs`，实现可连接率、丢包率、延迟、P95、抖动、TCP、连续失败和缺失指标聚合。
- [✓] 创建 `scoring.rs`，实现稳定性、分时段、性能、总分、可信度和原因输出。
- [✓] 创建 `scoring_test.rs`，覆盖评分高低、低可信度、聚合关键路径和分时段原因。
- [✓] 修复缺失指标中文说明和分时段样本不完整判定。
- [✓] 规格审查通过。
- [✓] 代码质量审查通过。
- [✓] QA 验证通过：`cargo test --manifest-path src-tauri/Cargo.toml --test scoring_test`、`cargo check --manifest-path src-tauri/Cargo.toml`。
- [✓] 已提交：`b3dc6ef feat: 实现指标聚合评分`。

### 2026-06-07 批次：Task 5 Markdown 报告生成
- [✓] 创建 `report.rs`，实现中文 Markdown 报告生成。
- [✓] 创建 `report_test.rs`，覆盖报告标题、权重、候选 IP、城市和推荐结论。
- [✓] 修复排名和推荐结论排序，确保按总分降序选择最高分目标。
- [✓] 规格审查通过。
- [✓] 代码质量审查通过。
- [✓] QA 验证通过：`cargo test --manifest-path src-tauri/Cargo.toml --test report_test`、`cargo check --manifest-path src-tauri/Cargo.toml`。
- [✓] 已提交：`b751ce8 feat: 生成中文测试报告`。

### 2026-06-07 批次：Task 6 探测调度与一次立即测试
- [✓] 创建 `scheduler.rs`，实现立即探测、时段判断、并发限制、采样、聚合、评分和报告生成。
- [✓] 创建 `scheduler_test.rs`，覆盖白天、晚上、其他时段和运行中跳过规则。
- [✓] 修复单目标失败降级、短时采样边界和 interval sleep 超时风险。
- [✓] 规格审查通过。
- [✓] 代码质量审查通过。
- [✓] QA 验证通过：`cargo test --manifest-path src-tauri/Cargo.toml`、`cargo check --manifest-path src-tauri/Cargo.toml`。
- [✓] 已提交：`0078bb0 feat: 实现探测调度`。

### 2026-06-07 批次：Task 7 Tauri 后端命令
- [✓] 创建 `commands.rs`，实现配置校验和启动探测命令。
- [✓] 注册 `validate_config_text` 和 `start_probe` Tauri 命令。
- [✓] 创建 `commands_test.rs`，覆盖配置校验成功与中文错误。
- [✓] 修复前端继续调用已移除 `greet` 命令的问题。
- [✓] 规格审查通过。
- [✓] 代码质量审查通过。
- [✓] QA 验证通过：`cargo test --manifest-path src-tauri/Cargo.toml --test commands_test`、`cargo check --manifest-path src-tauri/Cargo.toml`、`npm run build`。
- [✓] 已提交：`2777614 feat: 注册 Tauri 探测命令`。

### 2026-06-07 批次：Task 8 前端 GUI 页面
- [✓] 创建 Tauri API 封装、配置页、测试页、结果页、报告页和应用状态路由。
- [✓] 实现中文 TOML 编辑、配置校验、立即测试、排行榜、推荐结论、报告预览和 `.md` 导出。
- [✓] 修复结果页与后端 Markdown 排名表、推荐结论格式不匹配的问题。
- [✓] 规格审查通过。
- [✓] 代码质量审查通过。
- [✓] QA 验证通过：`npm run build`、`cargo check --manifest-path src-tauri/Cargo.toml`。
- [✓] 已提交：`d7cca0d feat: 实现中文前端界面`。

### 2026-06-07 批次：Task 9 端到端验收与修复
- [✓] Rust 测试通过：`cargo test --manifest-path src-tauri/Cargo.toml`。
- [✓] Rust 编译通过：`cargo check --manifest-path src-tauri/Cargo.toml`。
- [✓] 前端构建通过：`npm run build`。
- [✓] 安装并复核真实 `cargo-tauri` Cargo 子命令：`cargo tauri --version` 输出 `tauri-cli 2.11.2`。
- [✓] `cargo tauri dev` 可进入启动流程，用户终端已成功打开 GUI。
- [✓] 新增并通过业务 E2E：中文 TOML 三地真实候选 IP → 探测调度 → metrics/scores → Markdown 报告。
- [✓] 修复真实公网 E2E 卡死风险：`traceroute` 增加等待与最大跳数边界，并有回归测试覆盖。
- [✓] 完成 Playwright CLI 本机 Chrome GUI 验证：页面可达、三地真实候选 IP 可见、核心控件可见、控制台无错误。
- [✓] 创建并更新验证报告：`docs/reports/verification/101-vps-route-selection-mvp-verification.md`。

### 2026-06-08 批次：102 稳定性评分重构
- [✓] 删除稳定性评分中的基础可连接性权重。
- [✓] 新稳定性评分使用连续失败、丢包率、延迟抖动三项。
- [✓] 连续失败、丢包率、延迟抖动支持负向激励，最终展示分限制在 `0..100`。
- [✓] 新增 ICMP 成功样本数，成功样本为 `0` 或 `1` 时稳定性重罚。
- [✓] 报告显示新稳定性权重，业务 E2E 覆盖真实三地 IP 链路。

### 2026-06-08 批次：PR 前收尾验证
- [✓] 新鲜验证通过：`cargo test --manifest-path src-tauri/Cargo.toml`。
- [✓] 新鲜验证通过：`cargo check --manifest-path src-tauri/Cargo.toml`。
- [✓] 新鲜验证通过：`npm run build`。
- [✓] 新鲜验证通过：`npm run e2e`，Playwright 8 项通过。
- [✓] 当前功能分支准备推送并创建面向 `main` 的 PR。
