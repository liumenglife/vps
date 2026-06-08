use serde::{Deserialize, Serialize};

use crate::error::AppError;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    #[serde(rename = "探测设置")]
    pub probe: ProbeSettings,
    #[serde(rename = "端口设置")]
    pub ports: PortSettings,
    #[serde(rename = "总权重")]
    pub weights: MainWeights,
    #[serde(rename = "稳定性权重")]
    pub stability_weights: StabilityWeights,
    #[serde(rename = "分时段权重")]
    pub time_period_weights: TimePeriodWeights,
    #[serde(rename = "性能权重")]
    pub performance_weights: PerformanceWeights,
    #[serde(rename = "候选IP")]
    pub targets: Vec<Target>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProbeSettings {
    #[serde(rename = "默认测试分钟数")]
    pub default_duration_minutes: u64,
    #[serde(rename = "ICMP间隔毫秒")]
    pub icmp_interval_ms: u64,
    #[serde(rename = "TCP超时毫秒")]
    pub tcp_timeout_ms: u64,
    #[serde(rename = "并发数")]
    pub concurrency: usize,
    #[serde(rename = "白天时段")]
    pub day_period: String,
    #[serde(rename = "晚上时段")]
    pub night_period: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PortSettings {
    #[serde(rename = "默认端口")]
    pub default_ports: Vec<u16>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MainWeights {
    #[serde(rename = "稳定性")]
    pub stability: f64,
    #[serde(rename = "分时段表现")]
    pub time_period: f64,
    #[serde(rename = "性能")]
    pub performance: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StabilityWeights {
    #[serde(rename = "连续失败")]
    pub consecutive_failure: f64,
    #[serde(rename = "丢包率")]
    pub packet_loss: f64,
    #[serde(rename = "延迟抖动")]
    pub jitter: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TimePeriodWeights {
    #[serde(rename = "白天")]
    pub day: f64,
    #[serde(rename = "晚上")]
    pub night: f64,
    #[serde(rename = "其他")]
    pub other: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PerformanceWeights {
    #[serde(rename = "平均延迟")]
    pub avg_latency: f64,
    #[serde(rename = "P95延迟")]
    pub p95_latency: f64,
    #[serde(rename = "抖动")]
    pub jitter: f64,
    #[serde(rename = "TCP连接耗时")]
    pub tcp_connect_latency: f64,
    #[serde(rename = "路由跳数")]
    pub traceroute_hops: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Target {
    #[serde(rename = "IP")]
    pub ip: String,
    #[serde(rename = "城市")]
    pub city: String,
    #[serde(rename = "端口")]
    pub ports: Option<Vec<u16>>,
}

pub fn parse_config(input: &str) -> Result<AppConfig, AppError> {
    let config: AppConfig =
        toml::from_str(input).map_err(|e| AppError::Config(format!("TOML 解析失败：{e}")))?;
    validate_config(&config)?;
    Ok(config)
}

pub fn validate_config(config: &AppConfig) -> Result<(), AppError> {
    validate_weight_sum(
        "总权重",
        &[
            config.weights.stability,
            config.weights.time_period,
            config.weights.performance,
        ],
    )?;
    validate_weight_sum(
        "稳定性权重",
        &[
            config.stability_weights.consecutive_failure,
            config.stability_weights.packet_loss,
            config.stability_weights.jitter,
        ],
    )?;
    validate_weight_sum(
        "分时段权重",
        &[
            config.time_period_weights.day,
            config.time_period_weights.night,
            config.time_period_weights.other,
        ],
    )?;
    validate_weight_sum(
        "性能权重",
        &[
            config.performance_weights.avg_latency,
            config.performance_weights.p95_latency,
            config.performance_weights.jitter,
            config.performance_weights.tcp_connect_latency,
            config.performance_weights.traceroute_hops,
        ],
    )?;

    if config.probe.default_duration_minutes == 0
        || config.probe.icmp_interval_ms == 0
        || config.probe.tcp_timeout_ms == 0
        || config.probe.concurrency == 0
    {
        return Err(AppError::Config("探测设置中的数值必须大于 0".into()));
    }

    validate_ports("默认端口", &config.ports.default_ports)?;

    for target in &config.targets {
        target
            .ip
            .parse::<std::net::IpAddr>()
            .map_err(|_| AppError::Config(format!("IP 格式错误：{}", target.ip)))?;
        if target.city.trim().is_empty() {
            return Err(AppError::Config(format!("城市不能为空：{}", target.ip)));
        }
        if let Some(ports) = &target.ports {
            validate_ports("候选 IP 端口", ports)?;
        }
    }

    Ok(())
}

fn validate_weight_sum(name: &str, values: &[f64]) -> Result<(), AppError> {
    if values.iter().any(|value| *value < 0.0) {
        return Err(AppError::Config(format!("{name} 不能包含负数")));
    }

    let sum: f64 = values.iter().sum();
    if (sum - 1.0).abs() > 0.0001 {
        return Err(AppError::Config(format!(
            "{name} 之和必须为 1.0，当前为 {sum:.4}"
        )));
    }

    Ok(())
}

fn validate_ports(name: &str, ports: &[u16]) -> Result<(), AppError> {
    if ports.is_empty() {
        return Err(AppError::Config(format!("{name}不能为空")));
    }
    if ports.contains(&0) {
        return Err(AppError::Config(format!("{name}不能包含 0")));
    }

    Ok(())
}
