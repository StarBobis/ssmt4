pub mod configs;
pub mod utils;
mod commands; // 引入统一的命令模块

// 我们的 run 函数现在主要负责组装（Wiring）
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        // 从各个子模块中注册命令
        .invoke_handler(tauri::generate_handler![
            commands::common::greet,
            commands::settings::save_settings,
            commands::settings::load_settings,
            commands::process::run_resource_executable,
            commands::game_scanner::scan_games
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
