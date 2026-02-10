pub mod configs;
pub mod utils;
mod commands; // 引入统一的命令模块

// 我们的 run 函数现在主要负责组装（Wiring）
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        // 从各个子模块中注册命令
        .invoke_handler(tauri::generate_handler![
            commands::common::greet,
            commands::settings::save,
            commands::settings::load
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
