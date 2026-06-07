use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeSample {
    pub ip: String,
    pub city: String,
    pub timestamp: DateTime<Local>,
    pub icmp_latency_ms: Option<f64>,
    pub icmp_success: bool,
    pub tcp_results: Vec<TcpProbeResult>,
    pub traceroute_hops: Option<u32>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcpProbeResult {
    pub port: u16,
    pub success: bool,
    pub latency_ms: Option<f64>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetMetrics {
    pub ip: String,
    pub city: String,
    pub connectivity_rate: Option<f64>,
    pub icmp_packet_loss_rate: Option<f64>,
    pub avg_latency_ms: Option<f64>,
    pub p95_latency_ms: Option<f64>,
    pub jitter_ms: Option<f64>,
    pub tcp_success_rate: Option<f64>,
    pub tcp_avg_latency_ms: Option<f64>,
    pub consecutive_failures: u32,
    pub traceroute_hops: Option<u32>,
    pub sample_count: usize,
    pub test_period: String,
    pub missing_indicators: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetScore {
    pub ip: String,
    pub city: String,
    pub total_score: f64,
    pub stability_score: f64,
    pub time_period_score: f64,
    pub performance_score: f64,
    pub confidence: String,
    pub reasons: Vec<String>,
}
