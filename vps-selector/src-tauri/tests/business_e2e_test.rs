use vps_selector::config::parse_config;
use vps_selector::scheduler::run_probe_once;

#[tokio::test]
async fn runs_business_e2e_from_chinese_toml_to_markdown_report() {
    let input = include_str!("fixtures/valid-config.toml");
    let mut config = parse_config(input).expect("中文 TOML 配置应可解析");

    config.probe.default_duration_minutes = 0;
    config.probe.icmp_interval_ms = 1;
    config.probe.tcp_timeout_ms = 300;
    config.probe.concurrency = 3;
    config.ports.default_ports = vec![22, 443];

    let (metrics, scores, report) = run_probe_once(&config)
        .await
        .expect("业务 E2E 探测链路应完成");

    assert!(!metrics.is_empty());
    assert!(!scores.is_empty());
    assert!(report.contains("VPS 线路测试报告"));
    assert!(
        report.contains("白天单次综合排名")
            || report.contains("晚上单次综合排名")
            || report.contains("白天与晚上交叉验证综合排名")
    );
    assert!(report.contains("当前北京时间"));
    assert!(report.contains("本次测试时段"));
    assert!(!report.contains("伪综合排名"));
    assert!(report.contains("IP 详情"));
    assert!(report.contains("推荐结论"));
    assert!(report.contains("192.3.81.8"));
    assert!(report.contains("纽约"));
    assert!(report.contains("107.174.51.158"));
    assert!(report.contains("洛杉矶 DCO3"));
    assert!(report.contains("198.23.228.15"));
    assert!(report.contains("伊利诺伊州芝加哥"));
}
