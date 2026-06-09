use chrono::{TimeZone, Utc};
use vps_selector::config::parse_config;
use vps_selector::scheduler::{
    determine_beijing_period, determine_period, history_cache_path, should_skip_scheduled_run,
};

#[test]
fn returns_day_for_time_inside_day_period() {
    assert_eq!(
        determine_period("09:00", "08:00-18:00", "18:00-23:30"),
        "白天"
    );
}

#[test]
fn returns_night_for_time_inside_night_period() {
    assert_eq!(
        determine_period("20:00", "08:00-18:00", "18:00-23:30"),
        "晚上"
    );
}

#[test]
fn determines_period_by_fixed_beijing_time_not_local_timezone() {
    let utc_time = Utc.with_ymd_and_hms(2026, 6, 8, 10, 30, 0).unwrap();

    assert_eq!(
        determine_beijing_period(utc_time, "08:00-18:00", "18:00-23:30"),
        "晚上"
    );
}

#[test]
fn returns_other_for_time_outside_day_and_night_periods() {
    assert_eq!(
        determine_period("07:30", "08:00-18:00", "18:00-23:30"),
        "其他"
    );
}

#[test]
fn skips_scheduled_run_when_probe_is_already_running() {
    assert!(should_skip_scheduled_run(true));
    assert!(!should_skip_scheduled_run(false));
}

#[test]
fn history_cache_path_is_isolated_by_candidate_ip_set() {
    let mut first = parse_config(include_str!("fixtures/valid-config.toml")).unwrap();
    let mut second = first.clone();
    second.targets[0].ip = "203.0.113.10".into();

    let first_path = history_cache_path(&first);
    let reordered_first_path = {
        first.targets.reverse();
        history_cache_path(&first)
    };
    let second_path = history_cache_path(&second);

    assert_ne!(first_path, second_path);
    assert_eq!(first_path, reordered_first_path);
    assert_ne!(
        first_path.file_name().and_then(|name| name.to_str()),
        Some("vps-selector-history.json")
    );
}

#[test]
fn history_cache_path_is_isolated_by_default_ports_for_targets_without_explicit_ports() {
    let mut first = parse_config(include_str!("fixtures/valid-config.toml")).unwrap();
    for target in &mut first.targets {
        target.ports = None;
    }
    let mut second = first.clone();
    second.ports.default_ports = vec![443, 8443];

    assert_ne!(history_cache_path(&first), history_cache_path(&second));
}

#[test]
fn history_cache_path_is_isolated_by_reporting_and_scoring_config() {
    let first = parse_config(include_str!("fixtures/valid-config.toml")).unwrap();
    let mut changed_city = first.clone();
    changed_city.targets[0].city = "东京".into();
    let mut changed_period = first.clone();
    changed_period.probe.day_period = "07:00-17:00".into();
    let mut changed_weights = first.clone();
    changed_weights.weights.stability = 0.50;
    changed_weights.weights.performance = 0.30;

    assert_ne!(
        history_cache_path(&first),
        history_cache_path(&changed_city)
    );
    assert_ne!(
        history_cache_path(&first),
        history_cache_path(&changed_period)
    );
    assert_ne!(
        history_cache_path(&first),
        history_cache_path(&changed_weights)
    );
}

#[test]
fn history_cache_path_is_isolated_by_other_time_period_weight() {
    let first = parse_config(include_str!("fixtures/valid-config.toml")).unwrap();
    let mut changed_other = first.clone();
    changed_other.time_period_weights.other = first.time_period_weights.other + 0.01;

    assert_ne!(
        history_cache_path(&first),
        history_cache_path(&changed_other)
    );
}
