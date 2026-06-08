pub mod commands;
pub mod config;
pub mod error;
pub mod history;
pub mod metrics;
pub mod models;
pub mod probe;
pub mod report;
pub mod scheduler;
pub mod scoring;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            commands::validate_config_text,
            commands::start_probe,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
