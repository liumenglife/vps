use crate::config::AppConfig;
use crate::models::{TargetMetrics, TargetScore};

pub fn score_target(config: &AppConfig, metrics: &TargetMetrics) -> TargetScore {
    let mut reasons = metrics.missing_indicators.clone();

    let connectivity = metrics.connectivity_rate.unwrap_or(0.0);
    let stability_score = (score_consecutive_failures(metrics.consecutive_failures)
        * config.stability_weights.consecutive_failure
        + score_packet_loss(metrics.icmp_packet_loss_rate) * config.stability_weights.packet_loss
        + score_jitter_stability(
            metrics.jitter_ms,
            metrics.icmp_success_count,
            metrics.sample_count,
        ) * config.stability_weights.jitter)
        .clamp(0.0, 100.0);

    add_stability_reasons(metrics, &mut reasons);

    let period_score = (connectivity + metrics.tcp_success_rate.unwrap_or(0.0)) / 2.0;
    let time_period_score = match metrics.test_period.as_str() {
        "白天" => 100.0 * period_score * config.time_period_weights.day,
        "晚上" => 100.0 * period_score * config.time_period_weights.night,
        _ => 100.0 * period_score * config.time_period_weights.other,
    };
    let performance_score = 100.0
        * (latency_score(metrics.avg_latency_ms, 300.0) * config.performance_weights.avg_latency
            + latency_score(metrics.p95_latency_ms, 600.0)
                * config.performance_weights.p95_latency
            + latency_score(metrics.jitter_ms, 100.0) * config.performance_weights.jitter
            + latency_score(metrics.tcp_avg_latency_ms, 200.0)
                * config.performance_weights.tcp_connect_latency
            + hops_score(metrics.traceroute_hops) * config.performance_weights.traceroute_hops);

    let total_score = config.weights.stability * stability_score
        + config.weights.time_period * time_period_score
        + config.weights.performance * performance_score;

    TargetScore {
        ip: metrics.ip.clone(),
        city: metrics.city.clone(),
        total_score,
        stability_score,
        time_period_score,
        performance_score,
        confidence: confidence(metrics),
        reasons,
    }
}

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
    let Some(value) = value else {
        return -100.0;
    };

    if value == 0.0 {
        100.0
    } else if value <= 0.01 {
        90.0
    } else if value <= 0.03 {
        75.0
    } else if value <= 0.05 {
        50.0
    } else if value <= 0.10 {
        0.0
    } else if value <= 0.20 {
        -50.0
    } else {
        -100.0
    }
}

pub fn score_jitter_stability(
    jitter_ms: Option<f64>,
    icmp_success_count: usize,
    sample_count: usize,
) -> f64 {
    if icmp_success_count == 0 {
        return -100.0;
    }
    if icmp_success_count == 1 {
        return -80.0;
    }
    if sample_count > 0 && (icmp_success_count as f64 / sample_count as f64) < 0.2 {
        return -50.0;
    }

    let Some(jitter_ms) = jitter_ms else {
        return -100.0;
    };

    if jitter_ms <= 5.0 {
        100.0
    } else if jitter_ms <= 15.0 {
        80.0
    } else if jitter_ms <= 30.0 {
        50.0
    } else if jitter_ms <= 60.0 {
        0.0
    } else if jitter_ms <= 100.0 {
        -50.0
    } else {
        -100.0
    }
}

fn add_stability_reasons(metrics: &TargetMetrics, reasons: &mut Vec<String>) {
    if metrics.consecutive_failures >= 4 {
        reasons.push(format!(
            "连续失败 {} 次，稳定性重罚",
            metrics.consecutive_failures
        ));
    }

    if let Some(packet_loss) = metrics.icmp_packet_loss_rate {
        if packet_loss > 0.10 {
            reasons.push(format!(
                "丢包率 {:.2}%，线路质量严重不稳",
                packet_loss * 100.0
            ));
        }
    }

    if metrics.icmp_success_count == 0 {
        reasons.push("ICMP 成功样本为 0，稳定性重罚".into());
    } else if metrics.icmp_success_count == 1 {
        reasons.push("ICMP 成功样本仅 1 个，稳定性重罚".into());
    }

    if let Some(jitter_ms) = metrics.jitter_ms {
        if jitter_ms > 60.0 {
            reasons.push(format!("抖动 {:.2} ms，延迟波动严重", jitter_ms));
        }
    }
}

fn latency_score(value: Option<f64>, worst: f64) -> f64 {
    value
        .map(|value| 1.0 - (value / worst).min(1.0))
        .unwrap_or(0.0)
}

fn hops_score(value: Option<u32>) -> f64 {
    value
        .map(|value| 1.0 - (value as f64 / 30.0).min(1.0))
        .unwrap_or(0.0)
}

fn confidence(metrics: &TargetMetrics) -> String {
    if metrics.icmp_packet_loss_rate.is_none() && metrics.tcp_success_rate.is_none() {
        "不可推荐".into()
    } else if metrics.icmp_packet_loss_rate.is_none()
        || metrics.tcp_success_rate.is_none()
        || !metrics.missing_indicators.is_empty()
    {
        "低".into()
    } else {
        "高".into()
    }
}
