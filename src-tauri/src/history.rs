use std::fs;
use std::path::Path;

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::models::TargetMetrics;

const COMPARISON_WINDOW_HOURS: i64 = 24;
const RETENTION_HOURS: i64 = 48;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryRecord {
    pub timestamp: DateTime<Utc>,
    pub metrics: Vec<TargetMetrics>,
}

pub fn comparison_history_metrics(
    records: &[HistoryRecord],
    now: DateTime<Utc>,
) -> Vec<TargetMetrics> {
    records
        .iter()
        .filter(|record| record.timestamp >= now - Duration::hours(COMPARISON_WINDOW_HOURS))
        .flat_map(|record| record.metrics.clone())
        .collect()
}

pub fn retain_history_records(
    records: Vec<HistoryRecord>,
    now: DateTime<Utc>,
) -> Vec<HistoryRecord> {
    records
        .into_iter()
        .filter(|record| record.timestamp >= now - Duration::hours(RETENTION_HOURS))
        .collect()
}

pub fn load_history(path: &Path) -> Vec<HistoryRecord> {
    let Ok(content) = fs::read_to_string(path) else {
        return Vec::new();
    };

    serde_json::from_str(&content).unwrap_or_default()
}

pub fn save_history(path: &Path, records: &[HistoryRecord]) {
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(content) = serde_json::to_string_pretty(records) {
        let _ = fs::write(path, content);
    }
}

pub fn append_history_record(
    records: Vec<HistoryRecord>,
    metrics: Vec<TargetMetrics>,
    now: DateTime<Utc>,
) -> Vec<HistoryRecord> {
    let mut records = retain_history_records(records, now);
    records.push(HistoryRecord {
        timestamp: now,
        metrics,
    });
    records
}
