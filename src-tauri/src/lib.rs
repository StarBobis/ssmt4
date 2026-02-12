pub mod configs;
pub mod utils;
mod commands; // 引入统一的命令模块

use crate::configs::app_config::AppConfig;
use std::sync::Mutex;
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
            
            // 1. 加载配置到内存 (如果失败则使用默认值)
            let config = AppConfig::load().unwrap_or_default();
            
            // 2. 根据配置初始化窗口大小
            let _ = main_window.set_size(tauri::Size::Logical(tauri::LogicalSize {
                width: config.window_width,
                height: config.window_height,
            }));

            // 3. 将配置包装在 Mutex 中，并托管给 Tauri 全局状态
            app.manage(Mutex::new(config));
            
            // 4. 初始化 ModWatcher 状态
            app.manage(commands::mod_manager::ModWatcher(Mutex::new(None)));

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { .. } = event {
                if let Ok(size) = window.inner_size() {
                    let scale_factor = window.scale_factor().unwrap_or(1.0);
                    let logical_size = size.to_logical::<f64>(scale_factor);
                    
                    // 从 State 中获取配置，而不是重新从磁盘加载
                    let state = window.state::<Mutex<AppConfig>>();
                    // 直接 unwrap，如果锁损坏(poisoned)，程序 Panic 是合理的安全行为
                    let mut config = state.lock().unwrap();
                    config.window_width = logical_size.width;
                    config.window_height = logical_size.height;
                    let _ = config.save();
                }
            }
        })
        // 从各个子模块中注册命令
        .invoke_handler(tauri::generate_handler![
            commands::common::greet,
            commands::common::get_resource_path,
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
            commands::game_config::update_game_background,
            commands::game_config::get_3dmigoto_latest_release,
            commands::game_config::install_3dmigoto_update,
            commands::game_launcher::start_game,
            commands::game_launcher::check_3dmigoto_integrity,
            commands::mod_manager::watch_mods,
            commands::mod_manager::unwatch_mods,
            commands::game_launcher::toggle_symlink,
            commands::mod_manager::scan_mods,
            commands::mod_manager::toggle_mod,
            commands::mod_manager::open_game_mods_folder,
            commands::mod_manager::preview_mod_archive,
            commands::mod_manager::install_mod_archive,
            commands::mod_manager::create_mod_group,
            commands::mod_manager::set_mod_group_icon,
            commands::mod_manager::open_mod_group_folder,
            commands::mod_manager::rename_mod_group,
            commands::mod_manager::move_mod_to_group,
            commands::mod_manager::delete_mod_group
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
