# Stability Scoring Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 重构稳定性评分，删除基础可连接性权重，改为连续失败、丢包率、延迟抖动三项，并加入负向激励。

**Architecture:** 保持现有 Rust 后端评分管线，最小修改配置模型、指标聚合、评分函数和报告生成。新增 ICMP 成功样本数字段，稳定性子项内部允许 `-100..100`，最终展示分限制为 `0..100`。

**Tech Stack:** Rust、Tauri v2、TOML 配置、现有 `cargo test` 集成测试。

---

## 文件结构

- Modify: `vps-selector/src-tauri/src/config.rs`，更新 `StabilityWeights` 字段和校验。
- Modify: `vps-selector/src-tauri/src/models.rs`，给 `TargetMetrics` 增加 `icmp_success_count`。
- Modify: `vps-selector/src-tauri/src/metrics.rs`，聚合 ICMP 成功样本数。
- Modify: `vps-selector/src-tauri/src/scoring.rs`，实现分档稳定性评分和负向激励。
- Modify: `vps-selector/src-tauri/src/report.rs`，更新稳定性权重展示和推荐理由。
- Modify: `vps-selector/src-tauri/tests/fixtures/valid-config.toml`，更新稳定性权重。
- Modify: `vps-selector/src-tauri/tests/fixtures/invalid-weight.toml`，更新字段并保持权重错误。
- Modify: `vps-selector/src/app.ts`，更新前端默认中文 TOML。
- Modify tests: `config_test.rs`、`scoring_test.rs`、`report_test.rs`、`business_e2e_test.rs`。

## Task 1: 配置模型改为三项稳定性权重

**Files:**
- Modify: `vps-selector/src-tauri/src/config.rs`
- Modify: `vps-selector/src-tauri/tests/fixtures/valid-config.toml`
- Modify: `vps-selector/src-tauri/tests/fixtures/invalid-weight.toml`
- Modify: `vps-selector/src-tauri/tests/config_test.rs`
- Modify: `vps-selector/src/app.ts`

- [ ] **Step 1: 写失败测试**

在 `vps-selector/src-tauri/tests/config_test.rs` 的 `parses_chinese_toml_config` 中改断言：

```rust
assert_eq!(config.stability_weights.consecutive_failure, 0.45);
assert_eq!(config.stability_weights.packet_loss, 0.40);
assert_eq!(config.stability_weights.jitter, 0.15);
```

删除对 `connectivity` 的断言。

- [ ] **Step 2: 运行测试确认失败**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml --test config_test
```

Expected: FAIL，原因是 `StabilityWeights` 没有 `jitter` 字段，或 fixture 仍包含旧字段。

- [ ] **Step 3: 更新配置结构**

在 `vps-selector/src-tauri/src/config.rs` 中把 `StabilityWeights` 改为：

```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StabilityWeights {
    #[serde(rename = "丢包率")]
    pub packet_loss: f64,
    #[serde(rename = "连续失败")]
    pub consecutive_failure: f64,
    #[serde(rename = "延迟抖动")]
    pub jitter: f64,
}
```

更新 `validate_config` 中稳定性权重校验：

```rust
validate_weight_sum(
    "稳定性权重",
    &[
        config.stability_weights.packet_loss,
        config.stability_weights.consecutive_failure,
        config.stability_weights.jitter,
    ],
)?;
```

- [ ] **Step 4: 更新 TOML fixture**

在 `valid-config.toml` 中替换：

```toml
["稳定性权重"]
"连续失败" = 0.45
"丢包率" = 0.40
"延迟抖动" = 0.15
```

在 `invalid-weight.toml` 中保持错误总和，例如：

```toml
["稳定性权重"]
"连续失败" = 0.45
"丢包率" = 0.40
"延迟抖动" = 0.20
```

- [ ] **Step 5: 更新前端默认配置**

在 `vps-selector/src/app.ts` 的 `sampleConfig` 中同步替换稳定性权重为：

```toml
["稳定性权重"]
"连续失败" = 0.45
"丢包率" = 0.40
"延迟抖动" = 0.15
```

- [ ] **Step 6: 验证通过**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml --test config_test
npm run build
```

Expected: PASS。

- [ ] **Step 7: Code review + QA + Commit**

按项目规范执行：实现完成后派 code review 子代理；code review 通过后派 QA 子代理；QA 通过后提交。

```bash
git add vps-selector/src-tauri/src/config.rs vps-selector/src-tauri/tests/config_test.rs vps-selector/src-tauri/tests/fixtures/valid-config.toml vps-selector/src-tauri/tests/fixtures/invalid-weight.toml vps-selector/src/app.ts
git -c user.name="liumenglife" -c user.email="liumenglife@163.com" commit -m "feat: 更新稳定性权重配置"
```

## Task 2: 指标聚合增加 ICMP 成功样本数

**Files:**
- Modify: `vps-selector/src-tauri/src/models.rs`
- Modify: `vps-selector/src-tauri/src/metrics.rs`
- Modify: `vps-selector/src-tauri/tests/scoring_test.rs`
- Modify all `TargetMetrics` literals in tests.

- [ ] **Step 1: 写失败测试**

在 `vps-selector/src-tauri/tests/scoring_test.rs` 的 `aggregates_probe_samples_into_target_metrics` 中新增：

```rust
assert_eq!(metrics.icmp_success_count, 2);
```

- [ ] **Step 2: 运行测试确认失败**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml --test scoring_test
```

Expected: FAIL，`TargetMetrics` 没有 `icmp_success_count` 字段。

- [ ] **Step 3: 更新模型**

在 `TargetMetrics` 中新增字段：

```rust
pub icmp_success_count: usize,
```

- [ ] **Step 4: 更新聚合**

在 `metrics.rs` 中新增：

```rust
let icmp_success_count = samples.iter().filter(|sample| sample.icmp_success).count();
```

并在 `TargetMetrics` 初始化中加入：

```rust
icmp_success_count,
```

- [ ] **Step 5: 更新测试对象字面量**

所有测试里的 `TargetMetrics { ... }` 必须补充：

```rust
icmp_success_count: 10,
```

对低样本测试使用对应数量，例如 `0`、`1`。

- [ ] **Step 6: 验证通过**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml --test scoring_test
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected: PASS。

- [ ] **Step 7: Code review + QA + Commit**

按项目规范完成 code review、QA 后提交：

```bash
git add vps-selector/src-tauri/src/models.rs vps-selector/src-tauri/src/metrics.rs vps-selector/src-tauri/tests
git -c user.name="liumenglife" -c user.email="liumenglife@163.com" commit -m "feat: 记录 ICMP 成功样本数"
```

## Task 3: 实现负向激励稳定性评分

**Files:**
- Modify: `vps-selector/src-tauri/src/scoring.rs`
- Modify: `vps-selector/src-tauri/tests/scoring_test.rs`

- [ ] **Step 1: 写连续失败分档失败测试**

在 `scoring_test.rs` 新增测试：

```rust
#[test]
fn consecutive_failures_use_penalty_buckets() {
    let config = parse_config(include_str!("fixtures/valid-config.toml")).unwrap();
    let cases = [(0, 100.0), (1, 80.0), (2, 50.0), (3, 0.0), (4, -50.0), (5, -100.0)];

    for (failures, expected) in cases {
        let metrics = stable_metrics_with_overrides(failures, Some(0.0), Some(2.0), 10, 10);
        let score = score_target(&config, &metrics);
        assert!(score.reasons.iter().any(|reason| reason.contains("连续失败")) || failures == 0);
        let expected_component = expected * 0.45;
        assert!(score.stability_breakdown_debug().contains(&format!("连续失败={expected_component:.2}")));
    }
}
```

如果当前没有 `stability_breakdown_debug`，不要实现这个测试形式；改为新增公开纯函数测试：

```rust
use vps_selector::scoring::score_consecutive_failures;

assert_eq!(score_consecutive_failures(0), 100.0);
assert_eq!(score_consecutive_failures(1), 80.0);
assert_eq!(score_consecutive_failures(2), 50.0);
assert_eq!(score_consecutive_failures(3), 0.0);
assert_eq!(score_consecutive_failures(4), -50.0);
assert_eq!(score_consecutive_failures(5), -100.0);
```

- [ ] **Step 2: 写丢包率分档失败测试**

新增：

```rust
use vps_selector::scoring::score_packet_loss;

#[test]
fn packet_loss_uses_penalty_buckets() {
    assert_eq!(score_packet_loss(Some(0.0)), 100.0);
    assert_eq!(score_packet_loss(Some(0.01)), 90.0);
    assert_eq!(score_packet_loss(Some(0.03)), 75.0);
    assert_eq!(score_packet_loss(Some(0.05)), 50.0);
    assert_eq!(score_packet_loss(Some(0.10)), 0.0);
    assert_eq!(score_packet_loss(Some(0.20)), -50.0);
    assert_eq!(score_packet_loss(Some(0.21)), -100.0);
    assert_eq!(score_packet_loss(None), -100.0);
}
```

- [ ] **Step 3: 写抖动和 ICMP 有效性失败测试**

新增：

```rust
use vps_selector::scoring::score_jitter_stability;

#[test]
fn jitter_stability_penalizes_too_few_successful_ping_samples() {
    assert_eq!(score_jitter_stability(None, 0, 300), -100.0);
    assert_eq!(score_jitter_stability(None, 1, 300), -80.0);
    assert_eq!(score_jitter_stability(Some(2.0), 10, 300), -50.0);
}

#[test]
fn jitter_stability_uses_jitter_buckets_after_enough_successful_samples() {
    assert_eq!(score_jitter_stability(Some(5.0), 100, 300), 100.0);
    assert_eq!(score_jitter_stability(Some(15.0), 100, 300), 80.0);
    assert_eq!(score_jitter_stability(Some(30.0), 100, 300), 50.0);
    assert_eq!(score_jitter_stability(Some(60.0), 100, 300), 0.0);
    assert_eq!(score_jitter_stability(Some(100.0), 100, 300), -50.0);
    assert_eq!(score_jitter_stability(Some(101.0), 100, 300), -100.0);
}
```

- [ ] **Step 4: 运行测试确认失败**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml --test scoring_test
```

Expected: FAIL，新增评分函数不存在。

- [ ] **Step 5: 实现公开纯函数**

在 `scoring.rs` 新增：

```rust
pub fn score_consecutive_failures(value: u32) -> f64 {
    match value {
        0 => 100.0,
        1 => 80.0,
        2 => 50.0,
        3 => 0.0,
        4 => -50.0,
        _ => -100.0,
    }
}

pub fn score_packet_loss(value: Option<f64>) -> f64 {
    let Some(loss) = value else { return -100.0; };
    if loss == 0.0 { 100.0 }
    else if loss <= 0.01 { 90.0 }
    else if loss <= 0.03 { 75.0 }
    else if loss <= 0.05 { 50.0 }
    else if loss <= 0.10 { 0.0 }
    else if loss <= 0.20 { -50.0 }
    else { -100.0 }
}

pub fn score_jitter_stability(jitter_ms: Option<f64>, icmp_success_count: usize, sample_count: usize) -> f64 {
    if icmp_success_count == 0 { return -100.0; }
    if icmp_success_count == 1 { return -80.0; }
    if sample_count > 0 && (icmp_success_count as f64 / sample_count as f64) < 0.20 { return -50.0; }

    let Some(jitter) = jitter_ms else { return -100.0; };
    if jitter <= 5.0 { 100.0 }
    else if jitter <= 15.0 { 80.0 }
    else if jitter <= 30.0 { 50.0 }
    else if jitter <= 60.0 { 0.0 }
    else if jitter <= 100.0 { -50.0 }
    else { -100.0 }
}
```

- [ ] **Step 6: 接入 score_target**

替换稳定性计算：

```rust
let consecutive_failure_score = score_consecutive_failures(metrics.consecutive_failures);
let packet_loss_score = score_packet_loss(metrics.icmp_packet_loss_rate);
let jitter_stability_score = score_jitter_stability(
    metrics.jitter_ms,
    metrics.icmp_success_count,
    metrics.sample_count,
);
let raw_stability_score = consecutive_failure_score * config.stability_weights.consecutive_failure
    + packet_loss_score * config.stability_weights.packet_loss
    + jitter_stability_score * config.stability_weights.jitter;
let stability_score = raw_stability_score.clamp(0.0, 100.0);
```

- [ ] **Step 7: 增加推荐理由**

在 `score_target` 中按异常添加原因：

```rust
if metrics.consecutive_failures >= 4 {
    reasons.push(format!("连续失败 {} 次，稳定性重罚", metrics.consecutive_failures));
}
if let Some(loss) = metrics.icmp_packet_loss_rate {
    if loss > 0.10 {
        reasons.push(format!("丢包率 {:.2}%，线路质量严重不稳", loss * 100.0));
    }
}
if metrics.icmp_success_count == 0 {
    reasons.push("ICMP 成功样本为 0，稳定性重罚".into());
} else if metrics.icmp_success_count == 1 {
    reasons.push("ICMP 成功样本仅 1 个，稳定性重罚".into());
}
if let Some(jitter) = metrics.jitter_ms {
    if jitter > 60.0 {
        reasons.push(format!("抖动 {:.2} ms，延迟波动严重", jitter));
    }
}
```

- [ ] **Step 8: 验证通过**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml --test scoring_test
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected: PASS。

- [ ] **Step 9: Code review + QA + Commit**

按项目规范完成 code review、QA 后提交：

```bash
git add vps-selector/src-tauri/src/scoring.rs vps-selector/src-tauri/tests/scoring_test.rs
git -c user.name="liumenglife" -c user.email="liumenglife@163.com" commit -m "feat: 重构稳定性负向评分"
```

## Task 4: 报告与 E2E 更新

**Files:**
- Modify: `vps-selector/src-tauri/src/report.rs`
- Modify: `vps-selector/src-tauri/tests/report_test.rs`
- Modify: `vps-selector/src-tauri/tests/business_e2e_test.rs`

- [ ] **Step 1: 写失败测试**

在 `report_test.rs` 中断言报告不再显示可连接性稳定性权重，并显示延迟抖动：

```rust
assert!(!report.contains("- 可连接性："));
assert!(report.contains("- 连续失败：0.45"));
assert!(report.contains("- 丢包率：0.40"));
assert!(report.contains("- 延迟抖动：0.15"));
```

- [ ] **Step 2: 运行测试确认失败**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml --test report_test
```

Expected: FAIL，报告仍显示旧字段或不显示延迟抖动。

- [ ] **Step 3: 更新报告稳定性权重输出**

在 `report.rs` 中替换稳定性权重段：

```rust
report.push_str("### 稳定性权重\n\n");
report.push_str(&format!("- 连续失败：{:.2}\n", config.stability_weights.consecutive_failure));
report.push_str(&format!("- 丢包率：{:.2}\n", config.stability_weights.packet_loss));
report.push_str(&format!("- 延迟抖动：{:.2}\n\n", config.stability_weights.jitter));
```

- [ ] **Step 4: 更新业务 E2E 断言**

在 `business_e2e_test.rs` 中增加：

```rust
assert!(report.contains("延迟抖动"));
assert!(!report.contains("可连接性：0.60"));
```

- [ ] **Step 5: 验证通过**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml --test report_test
cargo test --manifest-path src-tauri/Cargo.toml --test business_e2e_test -- --nocapture
```

Expected: PASS。

- [ ] **Step 6: Code review + QA + Commit**

按项目规范完成 code review、QA 后提交：

```bash
git add vps-selector/src-tauri/src/report.rs vps-selector/src-tauri/tests/report_test.rs vps-selector/src-tauri/tests/business_e2e_test.rs
git -c user.name="liumenglife" -c user.email="liumenglife@163.com" commit -m "feat: 更新稳定性报告展示"
```

## Task 5: 总体验证与文档更新

**Files:**
- Modify: `docs/reports/verification/101-vps-route-selection-mvp-verification.md`
- Modify: `docs/planning/current.md`
- Modify: `docs/planning/history.md`

- [ ] **Step 1: 运行总体验证**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
npm run build
npm run e2e
```

Expected: 全部 PASS。

- [ ] **Step 2: 更新验证报告**

在验证报告中新增：

```markdown
## 稳定性评分重构

- 删除基础可连接性稳定性权重。
- 稳定性由连续失败、丢包率、延迟抖动组成。
- 连续失败、丢包率、延迟抖动支持负向激励。
- 最终展示分限制在 0..100。
```

- [ ] **Step 3: 更新 planning 真相**

在 `current.md` 和 `history.md` 记录 102 稳定性评分重构完成。

- [ ] **Step 4: Commit**

```bash
git add docs/reports/verification/101-vps-route-selection-mvp-verification.md docs/planning/current.md docs/planning/history.md
git -c user.name="liumenglife" -c user.email="liumenglife@163.com" commit -m "docs: 记录稳定性评分重构验证"
```

## Self-Review

- Spec coverage: 覆盖删除可连接性、连续失败、丢包率、抖动、负分、配置、报告、测试。
- Placeholder scan: 无 `TBD`、`TODO`、`implement later`。
- Type consistency: 新字段统一为 `icmp_success_count`，新配置字段统一为 `jitter`，中文 TOML 字段统一为 `延迟抖动`。
