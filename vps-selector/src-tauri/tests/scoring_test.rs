use vps_selector::config::parse_config;
use vps_selector::metrics::aggregate_metrics;
use vps_selector::models::{ProbeSample, TargetMetrics, TcpProbeResult};
use vps_selector::scoring::score_target;

#[test]
fn aggregates_probe_samples_into_target_metrics() {
    let samples = vec![
        sample(true, Some(10.0), vec![tcp(true, Some(8.0))], Some(8)),
        sample(false, None, vec![tcp(false, None)], None),
        sample(true, Some(30.0), vec![tcp(false, None)], Some(9)),
    ];

    let metrics = aggregate_metrics("203.0.113.10", "东京", "白天", &samples);

    assert_eq!(metrics.connectivity_rate, Some(2.0 / 3.0));
    assert_eq!(metrics.icmp_packet_loss_rate, Some(1.0 / 3.0));
    assert_eq!(metrics.avg_latency_ms, Some(20.0));
    assert_eq!(metrics.p95_latency_ms, Some(30.0));
    assert_eq!(metrics.jitter_ms, Some(20.0));
    assert_eq!(metrics.tcp_success_rate, Some(1.0 / 3.0));
    assert_eq!(metrics.tcp_avg_latency_ms, Some(8.0));
    assert_eq!(metrics.consecutive_failures, 1);
    assert_eq!(metrics.traceroute_hops, Some(8));
    assert!(metrics.missing_indicators.is_empty());
}

#[test]
fn good_target_scores_higher_than_bad_target() {
    let config = parse_config(include_str!("fixtures/valid-config.toml")).unwrap();
    let good = TargetMetrics {
        ip: "203.0.113.10".into(),
        city: "东京".into(),
        connectivity_rate: Some(1.0),
        icmp_packet_loss_rate: Some(0.0),
        avg_latency_ms: Some(20.0),
        p95_latency_ms: Some(35.0),
        jitter_ms: Some(2.0),
        tcp_success_rate: Some(1.0),
        tcp_avg_latency_ms: Some(8.0),
        consecutive_failures: 0,
        traceroute_hops: Some(8),
        sample_count: 10,
        test_period: "白天".into(),
        missing_indicators: vec![],
    };
    let bad = TargetMetrics {
        ip: "203.0.113.20".into(),
        city: "大阪".into(),
        connectivity_rate: Some(0.4),
        icmp_packet_loss_rate: Some(0.5),
        avg_latency_ms: Some(280.0),
        p95_latency_ms: Some(600.0),
        jitter_ms: Some(80.0),
        tcp_success_rate: Some(0.2),
        tcp_avg_latency_ms: Some(180.0),
        consecutive_failures: 6,
        traceroute_hops: None,
        sample_count: 10,
        test_period: "白天".into(),
        missing_indicators: vec!["路由追踪缺失".into()],
    };

    assert!(score_target(&config, &good).total_score > score_target(&config, &bad).total_score);
}

#[test]
fn missing_icmp_but_tcp_available_is_low_confidence() {
    let config = parse_config(include_str!("fixtures/valid-config.toml")).unwrap();
    let metrics = TargetMetrics {
        ip: "203.0.113.30".into(),
        city: "首尔".into(),
        connectivity_rate: Some(1.0),
        icmp_packet_loss_rate: None,
        avg_latency_ms: None,
        p95_latency_ms: None,
        jitter_ms: None,
        tcp_success_rate: Some(1.0),
        tcp_avg_latency_ms: Some(20.0),
        consecutive_failures: 0,
        traceroute_hops: Some(10),
        sample_count: 10,
        test_period: "白天".into(),
        missing_indicators: vec!["ICMP 缺失".into()],
    };

    let score = score_target(&config, &metrics);

    assert_eq!(score.confidence, "低");
}

#[test]
fn day_and_night_test_periods_add_incomplete_period_sample_reason() {
    let config = parse_config(include_str!("fixtures/valid-config.toml")).unwrap();

    for test_period in ["白天", "晚上"] {
        let metrics = TargetMetrics {
            ip: "203.0.113.45".into(),
            city: "台北".into(),
            connectivity_rate: Some(1.0),
            icmp_packet_loss_rate: Some(0.0),
            avg_latency_ms: Some(25.0),
            p95_latency_ms: Some(40.0),
            jitter_ms: Some(3.0),
            tcp_success_rate: Some(1.0),
            tcp_avg_latency_ms: Some(10.0),
            consecutive_failures: 0,
            traceroute_hops: Some(9),
            sample_count: 10,
            test_period: test_period.into(),
            missing_indicators: vec![],
        };

        let score = score_target(&config, &metrics);

        assert!(
            score.reasons.contains(&"分时段样本不完整".into()),
            "test_period {test_period} should add incomplete period sample reason"
        );
    }
}

#[test]
fn other_test_period_does_not_add_incomplete_period_sample_reason() {
    let config = parse_config(include_str!("fixtures/valid-config.toml")).unwrap();
    let metrics = TargetMetrics {
        ip: "203.0.113.40".into(),
        city: "新加坡".into(),
        connectivity_rate: Some(1.0),
        icmp_packet_loss_rate: Some(0.0),
        avg_latency_ms: Some(25.0),
        p95_latency_ms: Some(40.0),
        jitter_ms: Some(3.0),
        tcp_success_rate: Some(1.0),
        tcp_avg_latency_ms: Some(10.0),
        consecutive_failures: 0,
        traceroute_hops: Some(9),
        sample_count: 10,
        test_period: "其他".into(),
        missing_indicators: vec![],
    };

    let score = score_target(&config, &metrics);

    assert!(!score.reasons.contains(&"分时段样本不完整".into()));
}

#[test]
fn full_day_test_period_does_not_add_incomplete_period_sample_reason() {
    let config = parse_config(include_str!("fixtures/valid-config.toml")).unwrap();
    let metrics = TargetMetrics {
        ip: "203.0.113.50".into(),
        city: "香港".into(),
        connectivity_rate: Some(1.0),
        icmp_packet_loss_rate: Some(0.0),
        avg_latency_ms: Some(18.0),
        p95_latency_ms: Some(32.0),
        jitter_ms: Some(2.0),
        tcp_success_rate: Some(1.0),
        tcp_avg_latency_ms: Some(9.0),
        consecutive_failures: 0,
        traceroute_hops: Some(7),
        sample_count: 10,
        test_period: "全天".into(),
        missing_indicators: vec![],
    };

    let score = score_target(&config, &metrics);

    assert!(!score.reasons.contains(&"分时段样本不完整".into()));
}

fn sample(
    icmp_success: bool,
    icmp_latency_ms: Option<f64>,
    tcp_results: Vec<TcpProbeResult>,
    traceroute_hops: Option<u32>,
) -> ProbeSample {
    ProbeSample {
        ip: "203.0.113.10".into(),
        city: "东京".into(),
        timestamp: chrono::Local::now(),
        icmp_latency_ms,
        icmp_success,
        tcp_results,
        traceroute_hops,
        errors: vec![],
    }
}

fn tcp(success: bool, latency_ms: Option<f64>) -> TcpProbeResult {
    TcpProbeResult {
        port: 443,
        success,
        latency_ms,
        error: None,
    }
}
