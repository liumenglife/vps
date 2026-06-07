# VPS 线路选择工具 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 构建 macOS 优先的 VPS 线路选择 Tauri GUI MVP，支持中文 TOML 配置、ICMP/TCP/traceroute 探测、可配置权重评分、中文 GUI 展示和 Markdown 报告导出。

**Architecture:** 前端使用 Tauri Web UI 展示配置、测试进度、排行榜、详情和报告；Rust 后端负责中文 TOML 解析、配置校验、探测调度、指标聚合、评分和 Markdown 生成。TCP 使用 Rust 原生 `TcpStream` 探测，ICMP 与 traceroute 在 MVP 中调用 macOS `ping`、`traceroute` 系统命令并解析输出。

**Tech Stack:** Rust、Tauri v2、TypeScript、TOML、Serde、Tokio、macOS `ping`、macOS `traceroute`。

---

## 文件结构

实施后项目应形成以下核心结构：

```text
vps-selector/
├── package.json
├── index.html
├── src/
│   ├── main.ts
│   ├── styles.css
│   ├── app.ts
│   ├── pages/
│   │   ├── config-page.ts
│   │   ├── test-page.ts
│   │   ├── result-page.ts
│   │   └── report-page.ts
│   └── utils/
│       └── tauri-api.ts
└── src-tauri/
    ├── Cargo.toml
    ├── tauri.conf.json
    ├── src/
    │   ├── lib.rs
    │   ├── error.rs
    │   ├── config.rs
    │   ├── models.rs
    │   ├── probe.rs
    │   ├── metrics.rs
    │   ├── scoring.rs
    │   ├── report.rs
    │   ├── scheduler.rs
    │   └── commands.rs
    └── tests/
        ├── config_test.rs
        ├── probe_parse_test.rs
        ├── scoring_test.rs
        └── report_test.rs
```

文件职责：

- `config.rs`：中文 TOML 结构体、`serde(rename = "...")` 映射、配置校验。
- `models.rs`：`Target`、`ProbeSample`、`TargetMetrics`、`TargetScore`、`TestReport` 等共享模型。
- `probe.rs`：TCP 探测、macOS `ping` 输出解析、macOS `traceroute` 输出解析。
- `metrics.rs`：从采样结果聚合可连接率、丢包率、延迟、P95、抖动、TCP 成功率、连续失败、路由跳数。
- `scoring.rs`：按 TOML 权重计算稳定性分、分时段分、性能分、总分、扣分原因和可信度。
- `report.rs`：生成中文 Markdown 报告。
- `scheduler.rs`：一次立即测试的执行流程和 GUI 打开时定时触发逻辑。
- `commands.rs`：Tauri 命令，给前端提供配置校验、开始测试、获取报告等接口。
- `src/pages/*`：中文 GUI 页面。

---

### Task 1: Tauri 脚手架与最小运行

**Files:**
- Create: `vps-selector/`
- Create: `vps-selector/src-tauri/src/lib.rs`
- Create: `vps-selector/src/app.ts`
- Modify: `vps-selector/src-tauri/Cargo.toml`

- [ ] **Step 1: 创建项目**

Run:

```bash
cargo create-tauri-app vps-selector --template vanilla-ts --manager npm
```

Expected: 生成 `vps-selector/`，包含 `src/` 和 `src-tauri/`。

- [ ] **Step 2: 安装依赖**

Run:

```bash
npm install
```

Workdir: `vps-selector`

Expected: `node_modules/` 创建完成，命令退出码为 0。

- [ ] **Step 3: 配置 Rust 依赖**

Modify `vps-selector/src-tauri/Cargo.toml`：

```toml
[dependencies]
tauri = { version = "2", features = [] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"
tokio = { version = "1", features = ["full"] }
thiserror = "1"
chrono = { version = "0.4", features = ["serde"] }

[build-dependencies]
tauri-build = { version = "2", features = [] }
```

- [ ] **Step 4: 验证脚手架**

Run:

```bash
cargo check --manifest-path src-tauri/Cargo.toml
npm run build
```

Expected: 两个命令都成功。

---

### Task 2: 中文 TOML 配置解析与校验

**Files:**
- Create: `vps-selector/src-tauri/src/error.rs`
- Create: `vps-selector/src-tauri/src/config.rs`
- Create: `vps-selector/src-tauri/tests/config_test.rs`

- [ ] **Step 1: 定义错误类型**

Create `error.rs`：

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("配置错误：{0}")]
    Config(String),
    #[error("探测错误：{0}")]
    Probe(String),
    #[error("报告错误：{0}")]
    Report(String),
    #[error("IO 错误：{0}")]
    Io(#[from] std::io::Error),
}
```

- [ ] **Step 2: 定义中文配置结构体**

Create `config.rs`，必须包含以下类型：

```rust
use serde::{Deserialize, Serialize};
use crate::error::AppError;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    #[serde(rename = "探测设置")]
    pub probe: ProbeSettings,
    #[serde(rename = "端口设置")]
    pub ports: PortSettings,
    #[serde(rename = "总权重")]
    pub weights: MainWeights,
    #[serde(rename = "稳定性权重")]
    pub stability_weights: StabilityWeights,
    #[serde(rename = "分时段权重")]
    pub time_period_weights: TimePeriodWeights,
    #[serde(rename = "性能权重")]
    pub performance_weights: PerformanceWeights,
    #[serde(rename = "候选IP")]
    pub targets: Vec<Target>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProbeSettings {
    #[serde(rename = "默认测试分钟数")]
    pub default_duration_minutes: u64,
    #[serde(rename = "ICMP间隔毫秒")]
    pub icmp_interval_ms: u64,
    #[serde(rename = "TCP超时毫秒")]
    pub tcp_timeout_ms: u64,
    #[serde(rename = "并发数")]
    pub concurrency: usize,
    #[serde(rename = "白天时段")]
    pub day_period: String,
    #[serde(rename = "晚上时段")]
    pub night_period: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PortSettings {
    #[serde(rename = "默认端口")]
    pub default_ports: Vec<u16>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MainWeights {
    #[serde(rename = "稳定性")]
    pub stability: f64,
    #[serde(rename = "分时段表现")]
    pub time_period: f64,
    #[serde(rename = "性能")]
    pub performance: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StabilityWeights {
    #[serde(rename = "可连接性")]
    pub connectivity: f64,
    #[serde(rename = "丢包率")]
    pub packet_loss: f64,
    #[serde(rename = "连续失败")]
    pub consecutive_failure: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TimePeriodWeights {
    #[serde(rename = "白天")]
    pub day: f64,
    #[serde(rename = "晚上")]
    pub night: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PerformanceWeights {
    #[serde(rename = "平均延迟")]
    pub avg_latency: f64,
    #[serde(rename = "P95延迟")]
    pub p95_latency: f64,
    #[serde(rename = "抖动")]
    pub jitter: f64,
    #[serde(rename = "TCP连接耗时")]
    pub tcp_connect_latency: f64,
    #[serde(rename = "路由跳数")]
    pub traceroute_hops: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Target {
    #[serde(rename = "IP")]
    pub ip: String,
    #[serde(rename = "城市")]
    pub city: String,
    #[serde(rename = "端口")]
    pub ports: Option<Vec<u16>>,
}

pub fn parse_config(input: &str) -> Result<AppConfig, AppError> {
    let config: AppConfig = toml::from_str(input)
        .map_err(|e| AppError::Config(format!("TOML 解析失败：{e}")))?;
    validate_config(&config)?;
    Ok(config)
}

pub fn validate_config(config: &AppConfig) -> Result<(), AppError> {
    validate_weight_sum("总权重", &[config.weights.stability, config.weights.time_period, config.weights.performance])?;
    validate_weight_sum("稳定性权重", &[config.stability_weights.connectivity, config.stability_weights.packet_loss, config.stability_weights.consecutive_failure])?;
    validate_weight_sum("分时段权重", &[config.time_period_weights.day, config.time_period_weights.night])?;
    validate_weight_sum("性能权重", &[config.performance_weights.avg_latency, config.performance_weights.p95_latency, config.performance_weights.jitter, config.performance_weights.tcp_connect_latency, config.performance_weights.traceroute_hops])?;

    if config.probe.default_duration_minutes == 0 || config.probe.icmp_interval_ms == 0 || config.probe.tcp_timeout_ms == 0 || config.probe.concurrency == 0 {
        return Err(AppError::Config("探测设置中的数值必须大于 0".into()));
    }

    if config.ports.default_ports.is_empty() {
        return Err(AppError::Config("默认端口不能为空".into()));
    }

    for target in &config.targets {
        target.ip.parse::<std::net::IpAddr>().map_err(|_| AppError::Config(format!("IP 格式错误：{}", target.ip)))?;
        if target.city.trim().is_empty() {
            return Err(AppError::Config(format!("城市不能为空：{}", target.ip)));
        }
    }
    Ok(())
}

fn validate_weight_sum(name: &str, values: &[f64]) -> Result<(), AppError> {
    if values.iter().any(|v| *v < 0.0) {
        return Err(AppError::Config(format!("{name} 不能包含负数")));
    }
    let sum: f64 = values.iter().sum();
    if (sum - 1.0).abs() > 0.0001 {
        return Err(AppError::Config(format!("{name} 之和必须为 1.0，当前为 {sum:.4}")));
    }
    Ok(())
}
```

- [ ] **Step 3: 编写配置测试**

Create `tests/config_test.rs`，至少覆盖：

```rust
use vps_selector::config::parse_config;

#[test]
fn parses_chinese_toml_config() {
    let input = include_str!("fixtures/valid-config.toml");
    let config = parse_config(input).unwrap();
    assert_eq!(config.targets[0].ip, "203.0.113.10");
    assert_eq!(config.targets[0].city, "东京");
    assert!((config.weights.stability - 0.5).abs() < 0.0001);
}

#[test]
fn rejects_invalid_weight_sum() {
    let input = include_str!("fixtures/invalid-weight.toml");
    let error = parse_config(input).unwrap_err().to_string();
    assert!(error.contains("总权重"));
}
```

Create fixture files under `src-tauri/tests/fixtures/` using the TOML example from `docs/superpowers/specs/101-vps-route-selection-design.md`.

- [ ] **Step 4: 验证配置模块**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml --test config_test
```

Expected: 配置解析和权重校验测试通过。

---

### Task 3: 核心模型与探测解析

**Files:**
- Create: `vps-selector/src-tauri/src/models.rs`
- Create: `vps-selector/src-tauri/src/probe.rs`
- Create: `vps-selector/src-tauri/tests/probe_parse_test.rs`

- [ ] **Step 1: 定义共享模型**

`models.rs` 必须定义：

```rust
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeSample {
    pub ip: String,
    pub city: String,
    pub timestamp: DateTime<Local>,
    pub icmp_latency_ms: Option<f64>,
    pub icmp_success: bool,
    pub tcp_results: Vec<TcpProbeResult>,
    pub traceroute_hops: Option<u32>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcpProbeResult {
    pub port: u16,
    pub success: bool,
    pub latency_ms: Option<f64>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetMetrics {
    pub ip: String,
    pub city: String,
    pub connectivity_rate: Option<f64>,
    pub icmp_packet_loss_rate: Option<f64>,
    pub avg_latency_ms: Option<f64>,
    pub p95_latency_ms: Option<f64>,
    pub jitter_ms: Option<f64>,
    pub tcp_success_rate: Option<f64>,
    pub tcp_avg_latency_ms: Option<f64>,
    pub consecutive_failures: u32,
    pub traceroute_hops: Option<u32>,
    pub sample_count: usize,
    pub test_period: String,
    pub missing_indicators: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetScore {
    pub ip: String,
    pub city: String,
    pub total_score: f64,
    pub stability_score: f64,
    pub time_period_score: f64,
    pub performance_score: f64,
    pub confidence: String,
    pub reasons: Vec<String>,
}
```

- [ ] **Step 2: 实现探测函数与解析函数**

`probe.rs` 必须提供：

```rust
pub fn parse_ping_avg_latency(output: &str) -> Option<f64>;
pub fn parse_ping_packet_loss(output: &str) -> Option<f64>;
pub fn parse_traceroute_hops(output: &str) -> Option<u32>;
pub fn tcp_connect(ip: &str, port: u16, timeout_ms: u64) -> crate::models::TcpProbeResult;
pub fn run_ping_once(ip: &str, timeout_ms: u64) -> Result<(Option<f64>, bool, Vec<String>), crate::error::AppError>;
pub fn run_traceroute_once(ip: &str) -> Result<(Option<u32>, Vec<String>), crate::error::AppError>;
```

解析要求：

- macOS ping 样例 `round-trip min/avg/max/stddev = 10.234/12.345/15.678/2.345 ms` 解析出 `12.345`。
- `0.0% packet loss` 解析为 `0.0`。
- `100.0% packet loss` 解析为 `1.0`。
- traceroute 统计非标题跳数行，返回跳数。

- [ ] **Step 3: 编写解析测试**

`probe_parse_test.rs` 至少包含：

```rust
use vps_selector::probe::{parse_ping_avg_latency, parse_ping_packet_loss, parse_traceroute_hops};

#[test]
fn parses_macos_ping_latency() {
    let output = "round-trip min/avg/max/stddev = 10.234/12.345/15.678/2.345 ms";
    assert_eq!(parse_ping_avg_latency(output), Some(12.345));
}

#[test]
fn parses_packet_loss() {
    assert_eq!(parse_ping_packet_loss("5 packets transmitted, 5 packets received, 0.0% packet loss"), Some(0.0));
    assert_eq!(parse_ping_packet_loss("5 packets transmitted, 0 packets received, 100.0% packet loss"), Some(1.0));
}

#[test]
fn parses_traceroute_hops() {
    let output = "traceroute to 203.0.113.10\n 1  192.168.1.1  1.2 ms\n 2  10.0.0.1  3.4 ms\n 3  203.0.113.10  20.1 ms";
    assert_eq!(parse_traceroute_hops(output), Some(3));
}
```

- [ ] **Step 4: 验证探测解析**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml --test probe_parse_test
```

Expected: 解析测试通过。

---

### Task 4: 指标聚合与评分引擎

**Files:**
- Create: `vps-selector/src-tauri/src/metrics.rs`
- Create: `vps-selector/src-tauri/src/scoring.rs`
- Create: `vps-selector/src-tauri/tests/scoring_test.rs`

- [ ] **Step 1: 实现指标聚合**

`metrics.rs` 必须提供：

```rust
pub fn aggregate_metrics(
    ip: &str,
    city: &str,
    test_period: &str,
    samples: &[crate::models::ProbeSample],
) -> crate::models::TargetMetrics;
```

聚合规则：

- `connectivity_rate`：ICMP 成功或任一 TCP 成功视为本轮可连接。
- `icmp_packet_loss_rate`：ICMP 失败次数 / ICMP 样本数。
- `avg_latency_ms`：ICMP 成功样本平均值。
- `p95_latency_ms`：ICMP 成功样本排序后第 95 百分位。
- `jitter_ms`：相邻 ICMP 延迟差值绝对值的平均值。
- `tcp_success_rate`：TCP 成功次数 / TCP 总次数。
- `tcp_avg_latency_ms`：TCP 成功连接耗时平均值。
- `consecutive_failures`：最长连续不可连接轮数。
- `missing_indicators`：缺少 ICMP、TCP、traceroute 时写入中文说明。

- [ ] **Step 2: 实现评分引擎**

`scoring.rs` 必须提供：

```rust
pub fn score_target(
    config: &crate::config::AppConfig,
    metrics: &crate::models::TargetMetrics,
) -> crate::models::TargetScore;
```

评分规则：

- 总分公式必须来自规格：稳定性、分时段表现、性能。
- 所有权重必须来自 TOML 配置。
- 稳定性包含可连接性、低丢包率、连续失败。
- 分时段表现 MVP 使用当前时段样本评分，并在缺少另一时段样本时加入原因“分时段样本不完整”。
- 性能包含平均延迟、P95 延迟、抖动、TCP 连接耗时、路由跳数。
- 缺失辅助指标允许降级；ICMP 与 TCP 均不可用时可信度为“不可推荐”。

- [ ] **Step 3: 编写评分测试**

`scoring_test.rs` 至少包含：

```rust
use vps_selector::config::parse_config;
use vps_selector::models::{TargetMetrics, TargetScore};
use vps_selector::scoring::score_target;

#[test]
fn good_target_scores_higher_than_bad_target() {
    let config = parse_config(include_str!("fixtures/valid-config.toml")).unwrap();
    let good = TargetMetrics { ip: "203.0.113.10".into(), city: "东京".into(), connectivity_rate: Some(1.0), icmp_packet_loss_rate: Some(0.0), avg_latency_ms: Some(20.0), p95_latency_ms: Some(35.0), jitter_ms: Some(2.0), tcp_success_rate: Some(1.0), tcp_avg_latency_ms: Some(8.0), consecutive_failures: 0, traceroute_hops: Some(8), sample_count: 10, test_period: "白天".into(), missing_indicators: vec![] };
    let bad = TargetMetrics { ip: "203.0.113.20".into(), city: "大阪".into(), connectivity_rate: Some(0.4), icmp_packet_loss_rate: Some(0.5), avg_latency_ms: Some(280.0), p95_latency_ms: Some(600.0), jitter_ms: Some(80.0), tcp_success_rate: Some(0.2), tcp_avg_latency_ms: Some(180.0), consecutive_failures: 6, traceroute_hops: None, sample_count: 10, test_period: "白天".into(), missing_indicators: vec!["traceroute 缺失".into()] };
    assert!(score_target(&config, &good).total_score > score_target(&config, &bad).total_score);
}

#[test]
fn missing_icmp_but_tcp_available_is_low_confidence() {
    let config = parse_config(include_str!("fixtures/valid-config.toml")).unwrap();
    let metrics = TargetMetrics { ip: "203.0.113.30".into(), city: "首尔".into(), connectivity_rate: Some(1.0), icmp_packet_loss_rate: None, avg_latency_ms: None, p95_latency_ms: None, jitter_ms: None, tcp_success_rate: Some(1.0), tcp_avg_latency_ms: Some(20.0), consecutive_failures: 0, traceroute_hops: Some(10), sample_count: 10, test_period: "白天".into(), missing_indicators: vec!["ICMP 缺失".into()] };
    let score = score_target(&config, &metrics);
    assert_eq!(score.confidence, "低");
}
```

- [ ] **Step 4: 验证评分模块**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml --test scoring_test
```

Expected: 评分测试通过。

---

### Task 5: Markdown 报告生成

**Files:**
- Create: `vps-selector/src-tauri/src/report.rs`
- Create: `vps-selector/src-tauri/tests/report_test.rs`

- [ ] **Step 1: 实现报告生成函数**

`report.rs` 必须提供：

```rust
pub fn generate_markdown_report(
    config: &crate::config::AppConfig,
    scores: &[crate::models::TargetScore],
    metrics: &[crate::models::TargetMetrics],
) -> String;
```

报告必须包含中文章节：

- `# VPS 线路测试报告`
- `## 测试配置`
- `## 评分权重`
- `## 候选 IP`
- `## 排名`
- `## IP 详情`
- `## 缺失数据说明`
- `## 推荐结论`

报告必须展示所有权重，包括总权重、稳定性权重、分时段权重、性能权重。

- [ ] **Step 2: 编写报告测试**

`report_test.rs` 至少断言报告包含：

```rust
assert!(report.contains("VPS 线路测试报告"));
assert!(report.contains("评分权重"));
assert!(report.contains("稳定性"));
assert!(report.contains("可连接性"));
assert!(report.contains("203.0.113.10"));
assert!(report.contains("东京"));
assert!(report.contains("推荐结论"));
```

- [ ] **Step 3: 验证报告模块**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml --test report_test
```

Expected: 报告测试通过。

---

### Task 6: 探测调度与一次立即测试

**Files:**
- Create: `vps-selector/src-tauri/src/scheduler.rs`
- Modify: `vps-selector/src-tauri/src/probe.rs`

- [ ] **Step 1: 实现一次目标探测**

`scheduler.rs` 必须提供：

```rust
pub async fn run_probe_once(
    config: &crate::config::AppConfig,
) -> Result<(Vec<crate::models::TargetMetrics>, Vec<crate::models::TargetScore>, String), crate::error::AppError>;
```

函数职责：

- 判断当前时段为白天、晚上或其他。
- 对每个 `Target` 使用其自定义端口，未配置时使用默认端口。
- 按 `concurrency` 限制并发。
- 在 `default_duration_minutes` 内按 `icmp_interval_ms` 采样。
- 每轮采样执行 ICMP、TCP；traceroute 每个目标只执行一次。
- 聚合 metrics，计算 score，生成 Markdown report。

- [ ] **Step 2: 支持 GUI 打开时定时触发**

`scheduler.rs` 必须提供：

```rust
pub fn determine_period(now_hhmm: &str, day_period: &str, night_period: &str) -> String;
pub fn should_skip_scheduled_run(is_running: bool) -> bool;
```

测试要求：

- `09:00` 在 `08:00-18:00` 中返回 `白天`。
- `20:00` 在 `18:00-23:30` 中返回 `晚上`。
- 既非白天也非晚上返回 `其他`。
- 已有测试运行时返回跳过。

- [ ] **Step 3: 手动验证真实探测**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected: 所有自动测试通过。真实网络探测不应作为强制 CI 测试，只在手工验收执行。

---

### Task 7: Tauri 后端命令

**Files:**
- Create: `vps-selector/src-tauri/src/commands.rs`
- Modify: `vps-selector/src-tauri/src/lib.rs`

- [ ] **Step 1: 注册模块**

`lib.rs` 必须包含：

```rust
pub mod commands;
pub mod config;
pub mod error;
pub mod metrics;
pub mod models;
pub mod probe;
pub mod report;
pub mod scheduler;
pub mod scoring;
```

- [ ] **Step 2: 实现 Tauri 命令**

`commands.rs` 必须提供：

```rust
#[tauri::command]
pub fn validate_config_text(content: String) -> Result<String, String>;

#[tauri::command]
pub async fn start_probe(content: String) -> Result<String, String>;
```

行为要求：

- `validate_config_text` 成功返回 `配置校验通过：N 个候选 IP`。
- 配置错误返回中文错误。
- `start_probe` 成功返回 Markdown 报告字符串。
- 探测错误返回中文错误。

- [ ] **Step 3: 注册命令到 Tauri Builder**

`lib.rs` 的 `run()` 必须注册：

```rust
.invoke_handler(tauri::generate_handler![
    commands::validate_config_text,
    commands::start_probe,
])
```

- [ ] **Step 4: 验证后端编译**

Run:

```bash
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected: 编译通过。

---

### Task 8: 前端 GUI 页面

**Files:**
- Create: `vps-selector/src/app.ts`
- Create: `vps-selector/src/utils/tauri-api.ts`
- Create: `vps-selector/src/pages/config-page.ts`
- Create: `vps-selector/src/pages/test-page.ts`
- Create: `vps-selector/src/pages/result-page.ts`
- Create: `vps-selector/src/pages/report-page.ts`
- Modify: `vps-selector/src/main.ts`
- Modify: `vps-selector/src/styles.css`

- [ ] **Step 1: 封装 Tauri API**

`tauri-api.ts` 必须提供：

```ts
import { invoke } from '@tauri-apps/api/core';

export function validateConfigText(content: string): Promise<string> {
  return invoke<string>('validate_config_text', { content });
}

export function startProbe(content: string): Promise<string> {
  return invoke<string>('start_probe', { content });
}
```

- [ ] **Step 2: 配置页面**

`config-page.ts` 必须实现：

- 中文 TOML 文本编辑区。
- “校验配置”按钮。
- “立即开始测试”按钮。
- 校验成功/失败中文提示。

- [ ] **Step 3: 测试页面**

`test-page.ts` 必须实现：

- 显示“测试中”。
- 调用 `startProbe(content)`。
- 成功后保存 Markdown 报告到前端状态并进入结果页面。
- 失败时展示中文错误。

- [ ] **Step 4: 结果页面**

`result-page.ts` 必须实现：

- 从 Markdown 的排名表中展示排行榜。
- 展示推荐购买 IP 和城市。
- 提供“查看报告”“重新测试”“返回配置”按钮。

- [ ] **Step 5: 报告页面**

`report-page.ts` 必须实现：

- 预览 Markdown 报告。
- 导出 `.md` 文件。
- 不提供 CSV。

- [ ] **Step 6: 验证前端构建**

Run:

```bash
npm run build
```

Expected: 构建通过。

---

### Task 9: 端到端验收与修复

**Files:**
- Modify as needed: `vps-selector/src-tauri/src/*.rs`
- Modify as needed: `vps-selector/src/**/*.ts`
- Create: `docs/reports/verification/101-vps-route-selection-mvp-verification.md`

- [ ] **Step 1: Rust 验证**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected: 所有测试通过，编译通过。

- [ ] **Step 2: 前端验证**

Run:

```bash
npm run build
```

Expected: 构建通过。

- [ ] **Step 3: macOS 手工验收**

Run:

```bash
cargo tauri dev
```

验收项目：

- GUI 启动成功。
- 中文 TOML 校验成功。
- 权重总和错误时出现中文错误提示。
- 使用 2-3 个真实商家测试 IP 可以完成立即测试。
- 结果页显示 IP、城市、排名、总分、推荐理由。
- 报告页展示所有权重。
- Markdown 导出文件包含 IP、城市、权重、指标、可信度、推荐结论。
- 禁 ping 或 traceroute 失败时程序不崩溃，并显示缺失指标说明。

- [ ] **Step 4: 写验证报告**

Create `docs/reports/verification/101-vps-route-selection-mvp-verification.md`，内容必须包含：

```markdown
# VPS 线路选择工具 MVP 验证报告

## 验证命令

- `cargo test --manifest-path src-tauri/Cargo.toml`
- `cargo check --manifest-path src-tauri/Cargo.toml`
- `npm run build`
- `cargo tauri dev`

## 验证结果

- Rust 测试：通过
- Rust 编译：通过
- 前端构建：通过
- macOS 手工验收：通过

## 验收结论

通过。
```

如果任一项失败，报告必须写“未通过”并列出失败输出和修复状态。

---

## 实施顺序

1. Task 1：脚手架。
2. Task 2：配置解析。
3. Task 3：模型与探测解析。
4. Task 4：指标聚合与评分。
5. Task 5：Markdown 报告。
6. Task 6：探测调度。
7. Task 7：Tauri 命令。
8. Task 8：GUI 页面。
9. Task 9：端到端验收。

## 批次边界

本计划只交付 MVP：立即测试、GUI 打开时可扩展定时、中文 TOML、中文 GUI、中文 Markdown、macOS 优先、ICMP/TCP/traceroute 组合探测。不交付后台常驻、HTTP 健康检查、CSV、自动购买、Windows/Linux 兼容。
