use chrono::{Duration, TimeZone, Utc};
use vps_selector::history::{comparison_history_metrics, retain_history_records, HistoryRecord};
use vps_selector::models::TargetMetrics;

#[test]
fn comparison_history_uses_rolling_24_hours_only() {
    let now = Utc.with_ymd_and_hms(2026, 6, 8, 12, 0, 0).unwrap();
    let recent = HistoryRecord {
        timestamp: now - Duration::hours(23),
        metrics: vec![metric("晚上")],
    };
    let old = HistoryRecord {
        timestamp: now - Duration::hours(25),
        metrics: vec![metric("晚上")],
    };

    let metrics = comparison_history_metrics(&[recent, old], now);

    assert_eq!(metrics.len(), 1);
    assert_eq!(metrics[0].test_period, "晚上");
}

#[test]
fn history_records_are_retained_for_48_hours() {
    let now = Utc.with_ymd_and_hms(2026, 6, 8, 12, 0, 0).unwrap();
    let retained = HistoryRecord {
        timestamp: now - Duration::hours(48),
        metrics: vec![metric("白天")],
    };
    let expired = HistoryRecord {
        timestamp: now - Duration::hours(49),
        metrics: vec![metric("晚上")],
    };

    let records = retain_history_records(vec![retained, expired], now);

    assert_eq!(records.len(), 1);
    assert_eq!(records[0].metrics[0].test_period, "白天");
}

fn metric(test_period: &str) -> TargetMetrics {
    TargetMetrics {
        ip: "192.3.81.8".into(),
        city: "纽约".into(),
        connectivity_rate: Some(1.0),
        icmp_packet_loss_rate: Some(0.0),
        avg_latency_ms: Some(20.0),
        p95_latency_ms: Some(30.0),
        jitter_ms: Some(2.0),
        tcp_success_rate: Some(1.0),
        tcp_avg_latency_ms: Some(10.0),
        consecutive_failures: 0,
        traceroute_hops: Some(8),
        sample_count: 10,
        test_period: test_period.into(),
        missing_indicators: vec![],
    }
}
