pub mod configs;
pub mod utils;
mod commands; // 引入统一的命令模块

use crate::configs::app_config::AppConfig;
use tauri::{Manager, WindowEvent};

// 我们的 run 函数现在主要负责组装（Wiring）
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            let _ = app.get_webview_window("main").expect("no main window").set_focus();
        }))
        .setup(|app| {
            let main_window = app.get_webview_window("main").unwrap();
            if let Ok(config) = AppConfig::load() {
                let _ = main_window.set_size(tauri::Size::Logical(tauri::LogicalSize {
                    width: config.window_width,
                    height: config.window_height,
                }));
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { .. } = event {
                if let Ok(size) = window.inner_size() {
                    let scale_factor = window.scale_factor().unwrap_or(1.0);
                    let logical_size = size.to_logical::<f64>(scale_factor);
                    
                    if let Ok(mut config) = AppConfig::load() {
                        config.window_width = logical_size.width;
                        config.window_height = logical_size.height;
                        let _ = config.save();
                    }
                }
            }
        })
        // 从各个子模块中注册命令
        .invoke_handler(tauri::generate_handler![
            commands::common::greet,
            commands::common::ensure_directory,
            commands::common::open_in_explorer,
            commands::settings::save_settings,
            commands::settings::load_settings,
            commands::process::run_resource_executable,
            commands::game_scanner::scan_games,
            commands::game_scanner::set_game_visibility,
            commands::game_config::load_game_config,
            commands::game_config::save_game_config,
            commands::game_config::create_new_config,
            commands::game_config::delete_game_config_folder,
            commands::game_config::set_game_background,
            commands::game_config::set_game_icon,
            commands::game_config::update_game_background
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
