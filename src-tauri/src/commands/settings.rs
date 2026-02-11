use crate::configs::app_config::AppConfig;

#[tauri::command]
pub fn load_settings() -> Result<AppConfig, String> {
    AppConfig::load()
}

#[tauri::command]
pub fn save_settings(config: AppConfig) -> Result<(), String> {
    config.save()
}

