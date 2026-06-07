use vps_selector::scheduler::{determine_period, should_skip_scheduled_run};

#[test]
fn returns_day_for_time_inside_day_period() {
    assert_eq!(determine_period("09:00", "08:00-18:00", "18:00-23:30"), "白天");
}

#[test]
fn returns_night_for_time_inside_night_period() {
    assert_eq!(determine_period("20:00", "08:00-18:00", "18:00-23:30"), "晚上");
}

#[test]
fn returns_other_for_time_outside_day_and_night_periods() {
    assert_eq!(determine_period("07:30", "08:00-18:00", "18:00-23:30"), "其他");
}

#[test]
fn skips_scheduled_run_when_probe_is_already_running() {
    assert!(should_skip_scheduled_run(true));
    assert!(!should_skip_scheduled_run(false));
}
