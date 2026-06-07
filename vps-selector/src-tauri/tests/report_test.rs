use vps_selector::config::parse_config;
use vps_selector::models::{TargetMetrics, TargetScore};
use vps_selector::report::generate_markdown_report;

#[test]
fn generates_chinese_markdown_report_with_weights_and_recommendation() {
    let config = parse_config(include_str!("fixtures/valid-config.toml")).unwrap();
    let scores = vec![TargetScore {
        ip: "203.0.113.10".into(),
        city: "东京".into(),
        total_score: 92.5,
        stability_score: 95.0,
        time_period_score: 90.0,
        performance_score: 88.0,
        confidence: "高".into(),
        reasons: vec!["可连接性优秀".into()],
    }];
    let metrics = vec![TargetMetrics {
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
    }];

    let report = generate_markdown_report(&config, &scores, &metrics);

    assert!(report.contains("VPS 线路测试报告"));
    assert!(report.contains("评分权重"));
    assert!(report.contains("稳定性"));
    assert!(report.contains("可连接性"));
    assert!(report.contains("203.0.113.10"));
    assert!(report.contains("东京"));
    assert!(report.contains("推荐结论"));
}

#[test]
fn sorts_scores_by_total_score_for_ranking_and_recommendation() {
    let config = parse_config(include_str!("fixtures/valid-config.toml")).unwrap();
    let scores = vec![
        TargetScore {
            ip: "203.0.113.20".into(),
            city: "大阪".into(),
            total_score: 70.0,
            stability_score: 70.0,
            time_period_score: 70.0,
            performance_score: 70.0,
            confidence: "中".into(),
            reasons: vec!["表现一般".into()],
        },
        TargetScore {
            ip: "203.0.113.10".into(),
            city: "东京".into(),
            total_score: 92.5,
            stability_score: 95.0,
            time_period_score: 90.0,
            performance_score: 88.0,
            confidence: "高".into(),
            reasons: vec!["可连接性优秀".into()],
        },
    ];
    let metrics = vec![];

    let report = generate_markdown_report(&config, &scores, &metrics);

    assert!(report.contains("推荐选择 203.0.113.10（东京），总分 92.50"));
    let high_score_row = report.find("| 1 | 203.0.113.10 | 东京 | 92.50 |").unwrap();
    let low_score_row = report.find("| 2 | 203.0.113.20 | 大阪 | 70.00 |").unwrap();
    assert!(high_score_row < low_score_row);
}
