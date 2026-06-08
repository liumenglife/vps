use std::collections::HashSet;

use chrono::{FixedOffset, Utc};

use crate::config::AppConfig;
use crate::models::{ComprehensiveRanking, TargetMetrics, TargetScore};
use crate::scoring::score_target;

pub fn build_comprehensive_ranking(
    config: &AppConfig,
    current_metrics: Vec<TargetMetrics>,
    history_metrics: Option<Vec<TargetMetrics>>,
) -> ComprehensiveRanking {
    let has_day = current_metrics
        .iter()
        .any(|metric| metric.test_period == "白天");
    let has_night = current_metrics
        .iter()
        .any(|metric| metric.test_period == "晚上");
    let history = history_metrics.unwrap_or_default();
    let cross_period_metrics = merge_cross_period_metrics(&current_metrics, &history);
    let has_cross_period_metrics = !cross_period_metrics.is_empty();

    let metrics = if has_cross_period_metrics {
        cross_period_metrics
    } else {
        current_metrics
    };
    let mut scores = metrics
        .iter()
        .map(|metric| score_target(config, metric))
        .collect::<Vec<_>>();
    scores.sort_by(|a, b| b.total_score.total_cmp(&a.total_score));
    let ranking_basis = if has_cross_period_metrics {
        "白天与晚上交叉验证综合排名"
    } else if has_day {
        "白天单次综合排名"
    } else if has_night {
        "晚上单次综合排名"
    } else {
        "单次综合排名"
    }
    .to_string();

    ComprehensiveRanking {
        metrics,
        scores,
        ranking_basis,
    }
}

pub fn generate_markdown_report(
    config: &AppConfig,
    scores: &[TargetScore],
    metrics: &[TargetMetrics],
    ranking_basis: &str,
) -> String {
    let mut report = String::new();
    let mut ranked_scores = scores.iter().collect::<Vec<_>>();
    ranked_scores.sort_by(|a, b| b.total_score.total_cmp(&a.total_score));

    report.push_str("# VPS 线路测试报告\n\n");
    let beijing_now = Utc::now().with_timezone(&FixedOffset::east_opt(8 * 3600).unwrap());
    let test_period = metrics
        .first()
        .map(|metric| metric.test_period.as_str())
        .unwrap_or("未知");

    report.push_str("## 测试时间\n\n");
    report.push_str(&format!(
        "- 当前北京时间：{}\n",
        beijing_now.format("%Y-%m-%d %H:%M:%S UTC+08:00")
    ));
    report.push_str(&format!("- 本次测试时段：{}\n\n", test_period));

    report.push_str("## 测试配置\n\n");
    report.push_str(&format!(
        "- 默认测试分钟数：{}\n",
        config.probe.default_duration_minutes
    ));
    report.push_str(&format!(
        "- ICMP 间隔毫秒：{}\n",
        config.probe.icmp_interval_ms
    ));
    report.push_str(&format!(
        "- TCP 超时毫秒：{}\n",
        config.probe.tcp_timeout_ms
    ));
    report.push_str(&format!("- 并发数：{}\n", config.probe.concurrency));
    report.push_str(&format!("- 白天时段：{}\n", config.probe.day_period));
    report.push_str(&format!("- 晚上时段：{}\n", config.probe.night_period));
    report.push_str(&format!(
        "- 默认端口：{}\n\n",
        join_values(&config.ports.default_ports)
    ));

    report.push_str("## 评分权重\n\n");
    report.push_str("### 总权重\n\n");
    report.push_str(&format!("- 稳定性：{:.2}\n", config.weights.stability));
    report.push_str(&format!(
        "- 分时段表现：{:.2}\n",
        config.weights.time_period
    ));
    report.push_str(&format!("- 性能：{:.2}\n\n", config.weights.performance));
    report.push_str("### 稳定性权重\n\n");
    report.push_str(&format!(
        "- 连续失败：{:.2}\n",
        config.stability_weights.consecutive_failure
    ));
    report.push_str(&format!(
        "- 丢包率：{:.2}\n",
        config.stability_weights.packet_loss
    ));
    report.push_str(&format!(
        "- 延迟抖动：{:.2}\n\n",
        config.stability_weights.jitter
    ));
    report.push_str("### 分时段权重\n\n");
    report.push_str(&format!("- 白天：{:.2}\n", config.time_period_weights.day));
    report.push_str(&format!(
        "- 晚上：{:.2}\n\n",
        config.time_period_weights.night
    ));
    report.push_str("### 性能权重\n\n");
    report.push_str(&format!(
        "- 平均延迟：{:.2}\n",
        config.performance_weights.avg_latency
    ));
    report.push_str(&format!(
        "- P95 延迟：{:.2}\n",
        config.performance_weights.p95_latency
    ));
    report.push_str(&format!(
        "- 抖动：{:.2}\n",
        config.performance_weights.jitter
    ));
    report.push_str(&format!(
        "- TCP 连接耗时：{:.2}\n",
        config.performance_weights.tcp_connect_latency
    ));
    report.push_str(&format!(
        "- 路由跳数：{:.2}\n\n",
        config.performance_weights.traceroute_hops
    ));

    report.push_str("## 候选 IP\n\n");
    for target in &config.targets {
        let ports = target.ports.as_ref().unwrap_or(&config.ports.default_ports);
        report.push_str(&format!(
            "- {}（{}）：端口 {}\n",
            target.ip,
            target.city,
            join_values(ports)
        ));
    }
    report.push('\n');

    report.push_str(&format!("## {ranking_basis}\n\n"));
    report.push_str("| 排名 | IP | 城市 | 总分 | 稳定性 | 分时段 | 性能 | 可信度 |\n");
    report.push_str("| --- | --- | --- | ---: | ---: | ---: | ---: | --- |\n");
    for (index, score) in ranked_scores.iter().enumerate() {
        report.push_str(&format!(
            "| {} | {} | {} | {:.2} | {:.2} | {:.2} | {:.2} | {} |\n",
            index + 1,
            score.ip,
            score.city,
            score.total_score,
            score.stability_score,
            score.time_period_score,
            score.performance_score,
            score.confidence
        ));
    }
    report.push('\n');

    report.push_str("## IP 详情\n\n");
    for metric in metrics {
        report.push_str(&format!("### {}（{}）\n\n", metric.ip, metric.city));
        report.push_str(&format!("- 样本数：{}\n", metric.sample_count));
        report.push_str(&format!("- 测试时段：{}\n", metric.test_period));
        report.push_str(&format!(
            "- 可连接率：{}\n",
            format_percent(metric.connectivity_rate)
        ));
        report.push_str(&format!(
            "- ICMP 丢包率：{}\n",
            format_percent(metric.icmp_packet_loss_rate)
        ));
        report.push_str(&format!(
            "- 平均延迟：{}\n",
            format_ms(metric.avg_latency_ms)
        ));
        report.push_str(&format!(
            "- P95 延迟：{}\n",
            format_ms(metric.p95_latency_ms)
        ));
        report.push_str(&format!("- 抖动：{}\n", format_ms(metric.jitter_ms)));
        report.push_str(&format!(
            "- TCP 成功率：{}\n",
            format_percent(metric.tcp_success_rate)
        ));
        report.push_str(&format!(
            "- TCP 平均连接耗时：{}\n",
            format_ms(metric.tcp_avg_latency_ms)
        ));
        report.push_str(&format!("- 连续失败：{}\n", metric.consecutive_failures));
        report.push_str(&format!(
            "- 路由跳数：{}\n\n",
            format_optional(metric.traceroute_hops)
        ));
    }

    report.push_str("## 缺失数据说明\n\n");
    let mut has_missing = false;
    for metric in metrics {
        if metric.missing_indicators.is_empty() {
            continue;
        }
        has_missing = true;
        report.push_str(&format!(
            "- {}（{}）：{}\n",
            metric.ip,
            metric.city,
            metric.missing_indicators.join("、")
        ));
    }
    if !has_missing {
        report.push_str("- 无缺失数据。\n");
    }
    report.push('\n');

    report.push_str("## 推荐结论\n\n");
    if let Some(best) = ranked_scores.first() {
        let reasons = if best.reasons.is_empty() {
            "无额外说明".to_string()
        } else {
            best.reasons.join("、")
        };
        report.push_str(&format!(
            "按{ranking_basis}推荐选择 {}（{}），总分 {:.2}，可信度：{}。推荐理由：{}。\n",
            best.ip, best.city, best.total_score, best.confidence, reasons
        ));
    } else {
        report.push_str("暂无可推荐 IP。\n");
    }

    report
}

fn merge_cross_period_metrics(
    current: &[TargetMetrics],
    history: &[TargetMetrics],
) -> Vec<TargetMetrics> {
    let mut merged_keys = HashSet::new();
    let mut merged_metrics = Vec::new();

    for metric in current {
        let key = (metric.ip.clone(), metric.city.clone());
        if merged_keys.contains(&key) {
            continue;
        }

        let complement = current.iter().chain(history.iter()).find(|candidate| {
            candidate.ip == metric.ip
                && candidate.city == metric.city
                && is_day_night_pair(&metric.test_period, &candidate.test_period)
        });

        if let Some(other) = complement {
            merged_metrics.push(merge_metric_pair(metric, other));
            merged_keys.insert(key);
        }
    }

    merged_metrics
}

fn is_day_night_pair(left: &str, right: &str) -> bool {
    matches!((left, right), ("白天", "晚上") | ("晚上", "白天"))
}

fn merge_metric_pair(left: &TargetMetrics, right: &TargetMetrics) -> TargetMetrics {
    let total_samples = left.sample_count + right.sample_count;
    TargetMetrics {
        ip: left.ip.clone(),
        city: left.city.clone(),
        connectivity_rate: weighted_average(
            left.connectivity_rate,
            left.sample_count,
            right.connectivity_rate,
            right.sample_count,
        ),
        icmp_packet_loss_rate: weighted_average(
            left.icmp_packet_loss_rate,
            left.sample_count,
            right.icmp_packet_loss_rate,
            right.sample_count,
        ),
        avg_latency_ms: weighted_average(
            left.avg_latency_ms,
            left.sample_count,
            right.avg_latency_ms,
            right.sample_count,
        ),
        p95_latency_ms: max_optional(left.p95_latency_ms, right.p95_latency_ms),
        jitter_ms: weighted_average(
            left.jitter_ms,
            left.sample_count,
            right.jitter_ms,
            right.sample_count,
        ),
        tcp_success_rate: weighted_average(
            left.tcp_success_rate,
            left.sample_count,
            right.tcp_success_rate,
            right.sample_count,
        ),
        tcp_avg_latency_ms: weighted_average(
            left.tcp_avg_latency_ms,
            left.sample_count,
            right.tcp_avg_latency_ms,
            right.sample_count,
        ),
        consecutive_failures: left.consecutive_failures.max(right.consecutive_failures),
        traceroute_hops: left.traceroute_hops.or(right.traceroute_hops),
        sample_count: total_samples,
        test_period: "白天与晚上".into(),
        missing_indicators: merge_missing_indicators(
            &left.missing_indicators,
            &right.missing_indicators,
        ),
    }
}

fn weighted_average(
    left: Option<f64>,
    left_count: usize,
    right: Option<f64>,
    right_count: usize,
) -> Option<f64> {
    match (left, right) {
        (Some(left), Some(right)) => Some(
            (left * left_count as f64 + right * right_count as f64)
                / (left_count + right_count).max(1) as f64,
        ),
        (Some(value), None) | (None, Some(value)) => Some(value),
        (None, None) => None,
    }
}

fn max_optional(left: Option<f64>, right: Option<f64>) -> Option<f64> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left.max(right)),
        (Some(value), None) | (None, Some(value)) => Some(value),
        (None, None) => None,
    }
}

fn merge_missing_indicators(left: &[String], right: &[String]) -> Vec<String> {
    let mut indicators = left.to_vec();
    for indicator in right {
        if !indicators.contains(indicator) {
            indicators.push(indicator.clone());
        }
    }
    indicators
}

fn join_values<T: ToString>(values: &[T]) -> String {
    values
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("、")
}

fn format_percent(value: Option<f64>) -> String {
    value
        .map(|v| format!("{:.2}%", v * 100.0))
        .unwrap_or_else(|| "缺失".into())
}

fn format_ms(value: Option<f64>) -> String {
    value
        .map(|v| format!("{v:.2} ms"))
        .unwrap_or_else(|| "缺失".into())
}

fn format_optional<T: ToString>(value: Option<T>) -> String {
    value
        .map(|v| v.to_string())
        .unwrap_or_else(|| "缺失".into())
}
