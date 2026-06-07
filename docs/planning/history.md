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
