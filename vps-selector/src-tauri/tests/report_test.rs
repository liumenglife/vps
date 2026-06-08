use vps_selector::config::parse_config;
use vps_selector::models::{TargetMetrics, TargetScore};
use vps_selector::report::{build_comprehensive_ranking, generate_markdown_report};

#[test]
fn generates_chinese_markdown_report_with_weights_and_recommendation() {
    let config = parse_config(include_str!("fixtures/valid-config.toml")).unwrap();
    let scores = vec![TargetScore {
        ip: "192.3.81.8".into(),
        city: "纽约".into(),
        total_score: 92.5,
        stability_score: 95.0,
        time_period_score: 90.0,
        performance_score: 88.0,
        confidence: "高".into(),
        reasons: vec!["可连接性优秀".into()],
    }];
    let metrics = vec![TargetMetrics {
        ip: "192.3.81.8".into(),
        city: "纽约".into(),
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

    let report = generate_markdown_report(&config, &scores, &metrics, "白天单次综合排名");

    assert!(report.contains("VPS 线路测试报告"));
    assert!(report.contains("评分权重"));
    assert!(report.contains("稳定性"));
    assert!(report.contains("可连接性"));
    assert!(report.contains("192.3.81.8"));
    assert!(report.contains("纽约"));
    assert!(report.contains("推荐结论"));
}

#[test]
fn sorts_scores_by_total_score_for_ranking_and_recommendation() {
    let config = parse_config(include_str!("fixtures/valid-config.toml")).unwrap();
    let scores = vec![
        TargetScore {
            ip: "107.174.51.158".into(),
            city: "洛杉矶 DCO3".into(),
            total_score: 70.0,
            stability_score: 70.0,
            time_period_score: 70.0,
            performance_score: 70.0,
            confidence: "中".into(),
            reasons: vec!["表现一般".into()],
        },
        TargetScore {
            ip: "192.3.81.8".into(),
            city: "纽约".into(),
            total_score: 92.5,
            stability_score: 95.0,
            time_period_score: 90.0,
            performance_score: 88.0,
            confidence: "高".into(),
            reasons: vec!["可连接性优秀".into()],
        },
    ];
    let metrics = vec![];

    let report = generate_markdown_report(&config, &scores, &metrics, "白天单次综合排名");

    assert!(report.contains("按白天单次综合排名推荐选择 192.3.81.8（纽约），总分 92.50"));
    let high_score_row = report.find("| 1 | 192.3.81.8 | 纽约 | 92.50 |").unwrap();
    let low_score_row = report
        .find("| 2 | 107.174.51.158 | 洛杉矶 DCO3 | 70.00 |")
        .unwrap();
    assert!(high_score_row < low_score_row);
}

#[test]
fn report_shows_day_single_comprehensive_ranking_basis_and_beijing_time() {
    let config = parse_config(include_str!("fixtures/valid-config.toml")).unwrap();
    let metrics = vec![target_metrics("192.3.81.8", "纽约", "白天", 1.0, 10.0)];
    let ranking = build_comprehensive_ranking(&config, metrics, None);

    let report = generate_markdown_report(
        &config,
        &ranking.scores,
        &ranking.metrics,
        &ranking.ranking_basis,
    );

    assert_eq!(ranking.ranking_basis, "白天单次综合排名");
    assert!(report.contains("## 白天单次综合排名"));
    assert!(report.contains("- 当前北京时间："));
    assert!(report.contains("- 本次测试时段：白天"));
    assert!(report.contains("按白天单次综合排名推荐选择 192.3.81.8（纽约）"));
    assert!(!report.contains("伪综合排名"));
}

#[test]
fn comprehensive_ranking_cross_validates_when_history_has_complement_period() {
    let config = parse_config(include_str!("fixtures/valid-config.toml")).unwrap();
    let current = vec![target_metrics("192.3.81.8", "纽约", "白天", 0.8, 30.0)];
    let history = vec![target_metrics("192.3.81.8", "纽约", "晚上", 1.0, 20.0)];

    let ranking = build_comprehensive_ranking(&config, current, Some(history));

    assert_eq!(ranking.ranking_basis, "白天与晚上交叉验证综合排名");
    assert_eq!(ranking.metrics[0].test_period, "白天与晚上");
    assert_eq!(ranking.metrics[0].sample_count, 20);
}

#[test]
fn comprehensive_ranking_cross_validates_day_and_night_in_current_metrics_without_history() {
    let config = parse_config(include_str!("fixtures/valid-config.toml")).unwrap();
    let current = vec![
        target_metrics("192.3.81.8", "纽约", "白天", 0.8, 30.0),
        target_metrics("192.3.81.8", "纽约", "晚上", 1.0, 20.0),
    ];

    let ranking = build_comprehensive_ranking(&config, current, None);

    assert_eq!(ranking.ranking_basis, "白天与晚上交叉验证综合排名");
    assert_eq!(ranking.metrics.len(), 1);
    assert_eq!(ranking.scores.len(), 1);
    assert_eq!(ranking.metrics[0].ip, "192.3.81.8");
    assert_eq!(ranking.metrics[0].test_period, "白天与晚上");
    assert_eq!(ranking.metrics[0].sample_count, 20);
}

#[test]
fn comprehensive_ranking_excludes_targets_without_complement_period_from_cross_validation() {
    let config = parse_config(include_str!("fixtures/valid-config.toml")).unwrap();
    let current = vec![
        target_metrics("192.3.81.8", "纽约", "白天", 0.8, 30.0),
        target_metrics("192.3.81.8", "纽约", "晚上", 1.0, 20.0),
        target_metrics("107.174.51.158", "洛杉矶 DCO3", "白天", 1.0, 10.0),
    ];

    let ranking = build_comprehensive_ranking(&config, current, None);

    assert_eq!(ranking.ranking_basis, "白天与晚上交叉验证综合排名");
    assert_eq!(ranking.metrics.len(), 1);
    assert_eq!(ranking.scores.len(), 1);
    assert_eq!(ranking.metrics[0].ip, "192.3.81.8");
    assert!(ranking
        .metrics
        .iter()
        .all(|metric| metric.test_period == "白天与晚上"));
    assert!(!ranking
        .scores
        .iter()
        .any(|score| score.ip == "107.174.51.158"));
}

fn target_metrics(
    ip: &str,
    city: &str,
    test_period: &str,
    connectivity_rate: f64,
    avg_latency_ms: f64,
) -> TargetMetrics {
    TargetMetrics {
        ip: ip.into(),
        city: city.into(),
        connectivity_rate: Some(connectivity_rate),
        icmp_packet_loss_rate: Some(1.0 - connectivity_rate),
        avg_latency_ms: Some(avg_latency_ms),
        p95_latency_ms: Some(avg_latency_ms + 10.0),
        jitter_ms: Some(2.0),
        tcp_success_rate: Some(connectivity_rate),
        tcp_avg_latency_ms: Some(avg_latency_ms / 2.0),
        consecutive_failures: 0,
        traceroute_hops: Some(8),
        sample_count: 10,
        test_period: test_period.into(),
        missing_indicators: vec![],
    }
}
