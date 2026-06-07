use crate::config::parse_config;
use crate::scheduler::run_probe_once;

#[tauri::command]
pub fn validate_config_text(content: String) -> Result<String, String> {
    let config = parse_config(&content).map_err(|error| error.to_string())?;

    Ok(format!("配置校验通过：{} 个候选 IP", config.targets.len()))
}

#[tauri::command]
pub async fn start_probe(content: String) -> Result<String, String> {
    let config = parse_config(&content).map_err(|error| error.to_string())?;
    let (_, _, report) = run_probe_once(&config)
        .await
        .map_err(|error| error.to_string())?;

    Ok(report)
}
