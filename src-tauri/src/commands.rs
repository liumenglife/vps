use crate::config::parse_config;
use crate::scheduler::{run_probe_once_with_logger, ProbeLogSink};
use tauri::{Emitter, Window};

#[tauri::command]
pub fn validate_config_text(content: String) -> Result<String, String> {
    let config = parse_config(&content).map_err(|error| error.to_string())?;

    Ok(format!("配置校验通过：{} 个候选 IP", config.targets.len()))
}

#[tauri::command]
pub async fn start_probe(window: Window, content: String) -> Result<String, String> {
    let config = parse_config(&content).map_err(|error| error.to_string())?;
    let logger: ProbeLogSink = std::sync::Arc::new(move |message| {
        let _ = window.emit("probe-log", message);
    });
    let (_, _, report) = run_probe_once_with_logger(&config, Some(logger))
        .await
        .map_err(|error| error.to_string())?;

    Ok(report)
}
