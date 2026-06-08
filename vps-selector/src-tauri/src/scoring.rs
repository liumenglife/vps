use crate::config::AppConfig;
use crate::models::{TargetMetrics, TargetScore};

pub fn score_target(config: &AppConfig, metrics: &TargetMetrics) -> TargetScore {
    let reasons = metrics.missing_indicators.clone();

    let connectivity = metrics.connectivity_rate.unwrap_or(0.0);
    let packet_loss = metrics
        .icmp_packet_loss_rate
        .map(|loss| 1.0 - loss)
        .unwrap_or(0.0);
    let consecutive_failure = 1.0 - (metrics.consecutive_failures as f64 / 10.0).min(1.0);
    let stability_score = 100.0
        * (consecutive_failure * config.stability_weights.consecutive_failure
            + packet_loss * config.stability_weights.packet_loss
            + latency_score(metrics.jitter_ms, 100.0) * config.stability_weights.jitter);

    let period_score = (connectivity + metrics.tcp_success_rate.unwrap_or(0.0)) / 2.0;
    let time_period_score = match metrics.test_period.as_str() {
        "白天" => 100.0 * period_score * config.time_period_weights.day,
        "晚上" => 100.0 * period_score * config.time_period_weights.night,
        _ => 100.0 * period_score,
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
