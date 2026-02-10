use crate::configs::app_config::AppConfig;

#[tauri::command(rename = "loadSettings")]
pub fn load() -> AppConfig {
    AppConfig::load()
}

#[tauri::command(rename = "saveSettings")]
pub fn save(config: AppConfig) -> Result<(), String> {
    config.save()
}

