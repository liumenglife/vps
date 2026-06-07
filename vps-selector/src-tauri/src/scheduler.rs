use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::Local;
use tokio::sync::Semaphore;

use crate::config::{AppConfig, Target};
use crate::error::AppError;
use crate::metrics::aggregate_metrics;
use crate::models::{ProbeSample, TargetMetrics, TargetScore};
use crate::probe::{run_ping_once, run_traceroute_once, tcp_connect};
use crate::report::generate_markdown_report;
use crate::scoring::score_target;

pub async fn run_probe_once(
    config: &AppConfig,
) -> Result<(Vec<TargetMetrics>, Vec<TargetScore>, String), AppError> {
    let now = Local::now();
    let test_period = determine_period(
        &now.format("%H:%M").to_string(),
        &config.probe.day_period,
        &config.probe.night_period,
    );
    let semaphore = Arc::new(Semaphore::new(config.probe.concurrency.max(1)));
    let mut tasks = Vec::with_capacity(config.targets.len());

    for target in config.targets.clone() {
        let fallback_target = target.clone();
        let permit = semaphore.clone().acquire_owned().await.map_err(|error| {
            AppError::Probe(format!("并发控制初始化失败：{error}"))
        })?;
        let default_ports = config.ports.default_ports.clone();
        let probe_settings = config.probe.clone();
        let target_period = test_period.clone();

        tasks.push((
            fallback_target,
            target_period.clone(),
            tokio::spawn(async move {
                let _permit = permit;
                probe_target(&target, &default_ports, &probe_settings, &target_period)
                    .await
                    .unwrap_or_else(|error| {
                        let sample = failed_probe_sample(&target, format!("目标探测失败：{error}"));
                        aggregate_metrics(&target.ip, &target.city, &target_period, &[sample])
                    })
            }),
        ));
    }

    let mut metrics = Vec::with_capacity(tasks.len());
    for (target, target_period, task) in tasks {
        let metric = task.await.unwrap_or_else(|error| {
            let sample = failed_probe_sample(&target, format!("探测任务执行失败：{error}"));
            aggregate_metrics(&target.ip, &target.city, &target_period, &[sample])
        });
        metrics.push(metric);
    }

    let mut scores = metrics
        .iter()
        .map(|metric| score_target(config, metric))
        .collect::<Vec<_>>();
    scores.sort_by(|a, b| b.total_score.total_cmp(&a.total_score));
    let report = generate_markdown_report(config, &scores, &metrics);

    Ok((metrics, scores, report))
}

pub fn determine_period(now_hhmm: &str, day_period: &str, night_period: &str) -> String {
    let now = match parse_hhmm(now_hhmm) {
        Some(value) => value,
        None => return "其他".into(),
    };

    if period_contains(now, day_period) {
        "白天".into()
    } else if period_contains(now, night_period) {
        "晚上".into()
    } else {
        "其他".into()
    }
}

pub fn should_skip_scheduled_run(is_running: bool) -> bool {
    is_running
}

async fn probe_target(
    target: &Target,
    default_ports: &[u16],
    settings: &crate::config::ProbeSettings,
    test_period: &str,
) -> Result<TargetMetrics, AppError> {
    let ports = target.ports.as_deref().unwrap_or(default_ports).to_vec();
    let duration = Duration::from_secs(settings.default_duration_minutes * 60);
    let interval = Duration::from_millis(settings.icmp_interval_ms);
    let started_at = Instant::now();
    let mut samples = Vec::new();
    let mut traceroute_result = Some(run_traceroute(&target.ip).await);
    let sample_count = calculate_sample_count(duration, interval);

    for sample_index in 0..sample_count {
        let (traceroute_hops, traceroute_errors) = traceroute_result
            .take()
            .map(|result| result.unwrap_or_else(|error| (None, vec![format!("路由追踪失败：{error}")])))
            .unwrap_or_default();
        let sample = match probe_sample(
            target,
            &ports,
            settings.tcp_timeout_ms,
            traceroute_hops,
            traceroute_errors,
        )
        .await
        {
            Ok(sample) => sample,
            Err(error) => failed_probe_sample(target, format!("目标探测失败：{error}")),
        };
        samples.push(sample);

        if sample_index + 1 >= sample_count {
            break;
        }

        let remaining = duration.saturating_sub(started_at.elapsed());
        let Some(sleep_duration) = next_sleep_duration(remaining, interval) else {
            break;
        };
        tokio::time::sleep(sleep_duration).await;
    }

    Ok(aggregate_metrics(&target.ip, &target.city, test_period, &samples))
}

fn calculate_sample_count(duration: Duration, interval: Duration) -> u64 {
    if duration.is_zero() || interval.is_zero() {
        return 1;
    }

    let intervals = duration.as_nanos().div_ceil(interval.as_nanos());
    intervals.max(1).min(u64::MAX as u128) as u64
}

fn next_sleep_duration(remaining: Duration, interval: Duration) -> Option<Duration> {
    if remaining.is_zero() || interval.is_zero() {
        return None;
    }

    Some(remaining.min(interval))
}

fn failed_probe_sample(target: &Target, error: String) -> ProbeSample {
    ProbeSample {
        ip: target.ip.clone(),
        city: target.city.clone(),
        timestamp: Local::now(),
        icmp_latency_ms: None,
        icmp_success: false,
        tcp_results: Vec::new(),
        traceroute_hops: None,
        errors: vec![error],
    }
}

async fn probe_sample(
    target: &Target,
    ports: &[u16],
    tcp_timeout_ms: u64,
    traceroute_hops: Option<u32>,
    mut errors: Vec<String>,
) -> Result<ProbeSample, AppError> {
    let ip = target.ip.clone();
    let ping_timeout_ms = tcp_timeout_ms;
    let ping_result = tokio::task::spawn_blocking(move || {
        run_ping_once(&ip, ping_timeout_ms)
    })
    .await
    .map_err(|error| AppError::Probe(format!("ICMP 探测任务执行失败：{error}")))?;
    let (icmp_latency_ms, icmp_success, ping_errors) = match ping_result {
        Ok(result) => result,
        Err(error) => (None, false, vec![format!("ICMP 探测失败：{error}")]),
    };
    errors.extend(ping_errors);

    let mut tcp_results = Vec::with_capacity(ports.len());
    for port in ports.iter().copied() {
        let ip = target.ip.clone();
        let result = tokio::task::spawn_blocking(move || tcp_connect(&ip, port, tcp_timeout_ms))
            .await
            .map_err(|error| AppError::Probe(format!("TCP 探测任务执行失败：{error}")))?;
        if let Some(error) = &result.error {
            errors.push(format!("TCP {port} 失败：{error}"));
        }
        tcp_results.push(result);
    }

    Ok(ProbeSample {
        ip: target.ip.clone(),
        city: target.city.clone(),
        timestamp: Local::now(),
        icmp_latency_ms,
        icmp_success,
        tcp_results,
        traceroute_hops,
        errors,
    })
}

async fn run_traceroute(ip: &str) -> Result<(Option<u32>, Vec<String>), AppError> {
    let ip = ip.to_string();
    tokio::task::spawn_blocking(move || run_traceroute_once(&ip))
        .await
        .map_err(|error| AppError::Probe(format!("路由追踪任务执行失败：{error}")))?
}

fn period_contains(now: u32, period: &str) -> bool {
    let Some((start, end)) = parse_period(period) else {
        return false;
    };

    if start <= end {
        now >= start && now < end
    } else {
        now >= start || now < end
    }
}

fn parse_period(period: &str) -> Option<(u32, u32)> {
    let (start, end) = period.split_once('-')?;
    Some((parse_hhmm(start.trim())?, parse_hhmm(end.trim())?))
}

fn parse_hhmm(value: &str) -> Option<u32> {
    let (hour, minute) = value.split_once(':')?;
    let hour = hour.parse::<u32>().ok()?;
    let minute = minute.parse::<u32>().ok()?;
    (hour < 24 && minute < 60).then_some(hour * 60 + minute)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculates_sample_count_for_short_test_config() {
        assert_eq!(calculate_sample_count(Duration::from_millis(250), Duration::from_millis(100)), 3);
    }

    #[test]
    fn calculates_at_least_one_sample_for_zero_duration() {
        assert_eq!(calculate_sample_count(Duration::ZERO, Duration::from_millis(100)), 1);
    }

    #[test]
    fn next_sleep_duration_is_capped_by_remaining_time() {
        assert_eq!(
            next_sleep_duration(Duration::from_millis(40), Duration::from_millis(100)),
            Some(Duration::from_millis(40))
        );
    }

    #[test]
    fn next_sleep_duration_exits_when_time_has_expired() {
        assert_eq!(next_sleep_duration(Duration::ZERO, Duration::from_millis(100)), None);
    }

    #[test]
    fn target_failure_is_downgraded_to_error_sample_with_missing_indicators() {
        let target = Target {
            ip: "203.0.113.10".into(),
            city: "测试城市".into(),
            ports: None,
        };

        let sample = failed_probe_sample(&target, "目标探测失败：ping 命令不可用".into());
        let metrics = aggregate_metrics(&target.ip, &target.city, "白天", &[sample.clone()]);

        assert!(!sample.icmp_success);
        assert_eq!(sample.errors, vec!["目标探测失败：ping 命令不可用".to_string()]);
        assert!(metrics.missing_indicators.contains(&"ICMP 缺失".to_string()));
        assert!(metrics.missing_indicators.contains(&"TCP 缺失".to_string()));
        assert!(metrics.missing_indicators.contains(&"路由追踪缺失".to_string()));
    }
}
