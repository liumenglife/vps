use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::{DateTime, FixedOffset, Local, Utc};
use tokio::sync::Semaphore;

use crate::config::{AppConfig, Target};
use crate::error::AppError;
use crate::history::{
    append_history_record, comparison_history_metrics, load_history, save_history,
};
use crate::metrics::aggregate_metrics;
use crate::models::{ComprehensiveRanking, ProbeSample, TargetMetrics, TargetScore};
use crate::probe::{run_ping_once, run_traceroute_once, tcp_connect};
use crate::report::{build_comprehensive_ranking, generate_markdown_report};

pub type ProbeLogSink = Arc<dyn Fn(String) + Send + Sync + 'static>;

pub async fn run_probe_once(
    config: &AppConfig,
) -> Result<(Vec<TargetMetrics>, Vec<TargetScore>, String), AppError> {
    run_probe_once_with_logger(config, None).await
}

pub async fn run_probe_once_with_logger(
    config: &AppConfig,
    logger: Option<ProbeLogSink>,
) -> Result<(Vec<TargetMetrics>, Vec<TargetScore>, String), AppError> {
    let now = Utc::now();
    let test_period =
        determine_beijing_period(now, &config.probe.day_period, &config.probe.night_period);
    let semaphore = Arc::new(Semaphore::new(config.probe.concurrency.max(1)));
    let mut tasks = Vec::with_capacity(config.targets.len());

    for target in config.targets.clone() {
        let fallback_target = target.clone();
        let permit = semaphore
            .clone()
            .acquire_owned()
            .await
            .map_err(|error| AppError::Probe(format!("并发控制初始化失败：{error}")))?;
        let default_ports = config.ports.default_ports.clone();
        let probe_settings = config.probe.clone();
        let target_period = test_period.clone();
        let target_logger = logger.clone();

        tasks.push((
            fallback_target,
            target_period.clone(),
            tokio::spawn(async move {
                let _permit = permit;
                probe_target(
                    &target,
                    &default_ports,
                    &probe_settings,
                    &target_period,
                    target_logger.as_deref(),
                )
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

    let current_metrics = metrics;
    let history_path = history_cache_path(config);
    let history_records = load_history(&history_path);
    let comparison_metrics = comparison_history_metrics(&history_records, now);
    let ComprehensiveRanking {
        metrics,
        scores,
        ranking_basis,
    } = build_comprehensive_ranking(config, current_metrics.clone(), Some(comparison_metrics));
    let updated_history = append_history_record(history_records, current_metrics, now);
    save_history(&history_path, &updated_history);
    let report = generate_markdown_report(config, &scores, &metrics, &ranking_basis);

    Ok((metrics, scores, report))
}

pub fn determine_beijing_period(
    utc_now: DateTime<Utc>,
    day_period: &str,
    night_period: &str,
) -> String {
    let beijing = utc_now.with_timezone(&FixedOffset::east_opt(8 * 3600).unwrap());
    determine_period(
        &beijing.format("%H:%M").to_string(),
        day_period,
        night_period,
    )
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

pub fn history_cache_path(config: &AppConfig) -> PathBuf {
    let mut candidates = config
        .targets
        .iter()
        .map(|target| {
            let mut ports = target
                .ports
                .clone()
                .unwrap_or_else(|| config.ports.default_ports.clone());
            ports.sort_unstable();
            (target.ip.clone(), target.city.clone(), ports)
        })
        .collect::<Vec<_>>();
    candidates.sort();

    let mut key = 0xcbf29ce484222325_u64;
    key = hash_str(key, &config.probe.day_period);
    key = hash_str(key, &config.probe.night_period);
    key = hash_f64(key, config.weights.stability);
    key = hash_f64(key, config.weights.time_period);
    key = hash_f64(key, config.weights.performance);
    key = hash_f64(key, config.stability_weights.consecutive_failure);
    key = hash_f64(key, config.stability_weights.packet_loss);
    key = hash_f64(key, config.stability_weights.jitter);
    key = hash_f64(key, config.time_period_weights.day);
    key = hash_f64(key, config.time_period_weights.night);
    key = hash_f64(key, config.time_period_weights.other);
    key = hash_f64(key, config.performance_weights.avg_latency);
    key = hash_f64(key, config.performance_weights.p95_latency);
    key = hash_f64(key, config.performance_weights.jitter);
    key = hash_f64(key, config.performance_weights.tcp_connect_latency);
    key = hash_f64(key, config.performance_weights.traceroute_hops);

    let key = candidates
        .iter()
        .fold(key, |hash, candidate| hash_candidate(hash, candidate));

    std::env::temp_dir().join(format!("vps-selector-history-{key:016x}.json"))
}

fn hash_candidate(mut hash: u64, candidate: &(String, String, Vec<u16>)) -> u64 {
    hash = hash_str(hash, &candidate.0);
    hash = hash_str(hash, &candidate.1);
    for port in &candidate.2 {
        for byte in port.to_be_bytes() {
            hash = fnv1a_update(hash, byte);
        }
        hash = fnv1a_update(hash, 0);
    }
    hash
}

fn hash_str(mut hash: u64, value: &str) -> u64 {
    for byte in value.as_bytes() {
        hash = fnv1a_update(hash, *byte);
    }
    fnv1a_update(hash, 0)
}

fn hash_f64(mut hash: u64, value: f64) -> u64 {
    for byte in value.to_be_bytes() {
        hash = fnv1a_update(hash, byte);
    }
    hash
}

fn fnv1a_update(hash: u64, byte: u8) -> u64 {
    (hash ^ byte as u64).wrapping_mul(0x100000001b3)
}

async fn probe_target(
    target: &Target,
    default_ports: &[u16],
    settings: &crate::config::ProbeSettings,
    test_period: &str,
    logger: Option<&(dyn Fn(String) + Send + Sync + 'static)>,
) -> Result<TargetMetrics, AppError> {
    let ports = target.ports.as_deref().unwrap_or(default_ports).to_vec();
    let duration = Duration::from_secs(settings.default_duration_minutes * 60);
    let interval = Duration::from_millis(settings.icmp_interval_ms);
    let started_at = Instant::now();
    let mut samples = Vec::new();
    emit_probe_log(logger, format_probe_start_log(&target.ip, &target.city));
    emit_probe_log(logger, format_traceroute_start_log(&target.ip));
    let mut traceroute_result = Some(run_traceroute(&target.ip).await);
    let sample_count = calculate_sample_count(duration, interval);

    for sample_index in 0..sample_count {
        let sample_number = sample_index + 1;
        let (traceroute_hops, traceroute_errors) = traceroute_result
            .take()
            .map(|result| {
                result.unwrap_or_else(|error| (None, vec![format!("路由追踪失败：{error}")]))
            })
            .unwrap_or_default();
        if sample_index == 0 {
            emit_probe_log(
                logger,
                format_traceroute_result_log(&target.ip, traceroute_hops, &traceroute_errors),
            );
        }
        let sample = match probe_sample(
            target,
            &ports,
            settings.tcp_timeout_ms,
            traceroute_hops,
            traceroute_errors,
            sample_number,
            logger,
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

    let metrics = aggregate_metrics(&target.ip, &target.city, test_period, &samples);
    emit_probe_log(
        logger,
        format_target_complete_log(
            &target.ip,
            metrics.sample_count,
            metrics.connectivity_rate,
            metrics.tcp_success_rate,
        ),
    );

    Ok(metrics)
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
    sample_number: u64,
    logger: Option<&(dyn Fn(String) + Send + Sync + 'static)>,
) -> Result<ProbeSample, AppError> {
    let ip = target.ip.clone();
    let ping_timeout_ms = tcp_timeout_ms;
    let ping_result = tokio::task::spawn_blocking(move || run_ping_once(&ip, ping_timeout_ms))
        .await
        .map_err(|error| AppError::Probe(format!("ICMP 探测任务执行失败：{error}")))?;
    let (icmp_latency_ms, icmp_success, ping_errors) = match ping_result {
        Ok(result) => result,
        Err(error) => (None, false, vec![format!("ICMP 探测失败：{error}")]),
    };
    errors.extend(ping_errors);
    emit_probe_log(
        logger,
        format_icmp_sample_log(
            &target.ip,
            sample_number,
            icmp_success,
            icmp_latency_ms,
            &errors,
        ),
    );

    let mut tcp_results = Vec::with_capacity(ports.len());
    for port in ports.iter().copied() {
        let ip = target.ip.clone();
        let result = tokio::task::spawn_blocking(move || tcp_connect(&ip, port, tcp_timeout_ms))
            .await
            .map_err(|error| AppError::Probe(format!("TCP 探测任务执行失败：{error}")))?;
        if let Some(error) = &result.error {
            errors.push(format!("TCP {port} 失败：{error}"));
        }
        emit_probe_log(logger, format_tcp_result_log(&target.ip, &result));
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

fn emit_probe_log(logger: Option<&(dyn Fn(String) + Send + Sync + 'static)>, message: String) {
    if let Some(logger) = logger {
        logger(message);
    }
}

pub fn format_probe_start_log(ip: &str, city: &str) -> String {
    format!("[probe] start ip={ip} city={city}")
}

pub fn format_traceroute_start_log(ip: &str) -> String {
    format!("[traceroute] start ip={ip}")
}

pub fn format_traceroute_result_log(ip: &str, hops: Option<u32>, errors: &[String]) -> String {
    match hops {
        Some(hops) => format!("[traceroute] done ip={ip} hops={hops}"),
        None => format!("[traceroute] failed ip={ip} error={}", first_error(errors)),
    }
}

pub fn format_icmp_sample_log(
    ip: &str,
    sample_number: u64,
    success: bool,
    latency_ms: Option<f64>,
    errors: &[String],
) -> String {
    if success {
        format!(
            "[ping] sample={sample_number} ip={ip} success latency={:.2}ms loss=0%",
            latency_ms.unwrap_or_default()
        )
    } else {
        format!(
            "[ping] sample={sample_number} ip={ip} failed loss=100% error={}",
            first_error(errors)
        )
    }
}

pub fn format_tcp_result_log(ip: &str, result: &crate::models::TcpProbeResult) -> String {
    if result.success {
        format!(
            "[tcp] ip={ip} port={} success time={:.2}ms",
            result.port,
            result.latency_ms.unwrap_or_default()
        )
    } else {
        format!(
            "[tcp] ip={ip} port={} failed error={}",
            result.port,
            result.error.as_deref().unwrap_or("unknown error")
        )
    }
}

pub fn format_target_complete_log(
    ip: &str,
    sample_count: usize,
    connectivity_rate: Option<f64>,
    tcp_success_rate: Option<f64>,
) -> String {
    format!(
        "[probe] done ip={ip} samples={sample_count} connectivity={} tcp_success={}",
        format_percent(connectivity_rate),
        format_percent(tcp_success_rate)
    )
}

fn format_percent(value: Option<f64>) -> String {
    value
        .map(|value| format!("{:.2}%", value * 100.0))
        .unwrap_or_else(|| "n/a".into())
}

fn first_error(errors: &[String]) -> &str {
    errors
        .first()
        .map(String::as_str)
        .unwrap_or("unknown error")
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
    use crate::models::TcpProbeResult;

    #[test]
    fn calculates_sample_count_for_short_test_config() {
        assert_eq!(
            calculate_sample_count(Duration::from_millis(250), Duration::from_millis(100)),
            3
        );
    }

    #[test]
    fn calculates_at_least_one_sample_for_zero_duration() {
        assert_eq!(
            calculate_sample_count(Duration::ZERO, Duration::from_millis(100)),
            1
        );
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
        assert_eq!(
            next_sleep_duration(Duration::ZERO, Duration::from_millis(100)),
            None
        );
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
        assert_eq!(
            sample.errors,
            vec!["目标探测失败：ping 命令不可用".to_string()]
        );
        assert!(metrics
            .missing_indicators
            .contains(&"ICMP 缺失".to_string()));
        assert!(metrics.missing_indicators.contains(&"TCP 缺失".to_string()));
        assert!(metrics
            .missing_indicators
            .contains(&"路由追踪缺失".to_string()));
    }

    #[test]
    fn formats_probe_log_lines_like_command_output() {
        assert_eq!(
            format_probe_start_log("192.0.2.10", "东京"),
            "[probe] start ip=192.0.2.10 city=东京"
        );
        assert_eq!(
            format_traceroute_start_log("192.0.2.10"),
            "[traceroute] start ip=192.0.2.10"
        );
        assert_eq!(
            format_traceroute_result_log("192.0.2.10", Some(7), &[]),
            "[traceroute] done ip=192.0.2.10 hops=7"
        );
        assert_eq!(
            format_traceroute_result_log("192.0.2.10", None, &["timeout".to_string()]),
            "[traceroute] failed ip=192.0.2.10 error=timeout"
        );
    }

    #[test]
    fn formats_probe_log_lines_for_icmp_tcp_and_completion() {
        assert_eq!(
            format_icmp_sample_log("192.0.2.10", 2, true, Some(18.42), &[]),
            "[ping] sample=2 ip=192.0.2.10 success latency=18.42ms loss=0%"
        );
        assert_eq!(
            format_icmp_sample_log(
                "192.0.2.10",
                3,
                false,
                None,
                &["request timeout".to_string()]
            ),
            "[ping] sample=3 ip=192.0.2.10 failed loss=100% error=request timeout"
        );
        assert_eq!(
            format_tcp_result_log(
                "192.0.2.10",
                &TcpProbeResult {
                    port: 443,
                    success: true,
                    latency_ms: Some(31.5),
                    error: None,
                },
            ),
            "[tcp] ip=192.0.2.10 port=443 success time=31.50ms"
        );
        assert_eq!(
            format_target_complete_log("192.0.2.10", 5, Some(0.8), Some(0.67)),
            "[probe] done ip=192.0.2.10 samples=5 connectivity=80.00% tcp_success=67.00%"
        );
    }
}
