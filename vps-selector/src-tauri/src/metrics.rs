use crate::models::{ProbeSample, TargetMetrics};

pub fn aggregate_metrics(
    ip: &str,
    city: &str,
    test_period: &str,
    samples: &[ProbeSample],
) -> TargetMetrics {
    let sample_count = samples.len();
    let icmp_sample_count = samples.len();
    let icmp_failures = samples.iter().filter(|sample| !sample.icmp_success).count();
    let icmp_latencies: Vec<f64> = samples
        .iter()
        .filter_map(|sample| sample.icmp_success.then_some(sample.icmp_latency_ms).flatten())
        .collect();
    let tcp_total = samples.iter().map(|sample| sample.tcp_results.len()).sum::<usize>();
    let tcp_success_latencies: Vec<f64> = samples
        .iter()
        .flat_map(|sample| sample.tcp_results.iter())
        .filter_map(|result| result.success.then_some(result.latency_ms).flatten())
        .collect();
    let tcp_success_count = samples
        .iter()
        .flat_map(|sample| sample.tcp_results.iter())
        .filter(|result| result.success)
        .count();

    let connectivity_rate = (!samples.is_empty()).then(|| {
        let connected = samples
            .iter()
            .filter(|sample| {
                sample.icmp_success || sample.tcp_results.iter().any(|result| result.success)
            })
            .count();
        connected as f64 / samples.len() as f64
    });

    let consecutive_failures = samples
        .iter()
        .fold((0_u32, 0_u32), |(max_failures, current), sample| {
            let connected = sample.icmp_success || sample.tcp_results.iter().any(|result| result.success);
            if connected {
                (max_failures, 0)
            } else {
                let next = current + 1;
                (max_failures.max(next), next)
            }
        })
        .0;

    let traceroute_hops = samples.iter().find_map(|sample| sample.traceroute_hops);
    let mut missing_indicators = Vec::new();
    if icmp_sample_count == 0 || icmp_latencies.is_empty() {
        missing_indicators.push("ICMP 缺失".into());
    }
    if tcp_total == 0 {
        missing_indicators.push("TCP 缺失".into());
    }
    if traceroute_hops.is_none() {
        missing_indicators.push("路由追踪缺失".into());
    }

    TargetMetrics {
        ip: ip.into(),
        city: city.into(),
        connectivity_rate,
        icmp_packet_loss_rate: (icmp_sample_count > 0)
            .then_some(icmp_failures as f64 / icmp_sample_count as f64),
        avg_latency_ms: average(&icmp_latencies),
        p95_latency_ms: percentile_95(&icmp_latencies),
        jitter_ms: jitter(&icmp_latencies),
        tcp_success_rate: (tcp_total > 0).then_some(tcp_success_count as f64 / tcp_total as f64),
        tcp_avg_latency_ms: average(&tcp_success_latencies),
        consecutive_failures,
        traceroute_hops,
        sample_count,
        test_period: test_period.into(),
        missing_indicators,
    }
}

fn average(values: &[f64]) -> Option<f64> {
    (!values.is_empty()).then(|| values.iter().sum::<f64>() / values.len() as f64)
}

fn percentile_95(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let index = ((sorted.len() as f64 * 0.95).ceil() as usize).saturating_sub(1);
    sorted.get(index).copied()
}

fn jitter(values: &[f64]) -> Option<f64> {
    let diffs: Vec<f64> = values.windows(2).map(|pair| (pair[1] - pair[0]).abs()).collect();
    average(&diffs)
}
