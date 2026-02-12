use crate::configs::app_config::AppConfig;
use tauri::State;
use std::sync::Mutex;

#[tauri::command]
pub fn load_settings(state: State<Mutex<AppConfig>>) -> Result<AppConfig, String> {
    // 直接从内存中获取，不再读取磁盘
    Ok(state.lock().unwrap().clone())
}

#[tauri::command]
pub fn save_settings(state: State<Mutex<AppConfig>>, config: AppConfig) -> Result<(), String> {
    // 1. 更新内存中的状态
    // 我们需要把新配置写入到 State 里
    let mut state_config = state.lock().unwrap();
    *state_config = config;

    // 2. 持久化到磁盘
    state_config.save()
}

