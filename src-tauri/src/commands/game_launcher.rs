use crate::utils::file_manager::get_global_games_dir;
use crate::utils::ini_manager::IniManager;
use serde::Deserialize;
use std::path::PathBuf;
use std::process::Command;
use tauri::AppHandle;
use sysinfo::{System, ProcessRefreshKind, UpdateKind};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct GameConfigFull {
    three_d_migoto: Option<ThreeDMigotoConfig>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ThreeDMigotoConfig {
    install_dir: Option<String>,
    target_exe_path: Option<String>,
    launcher_exe_path: Option<String>,
    launch_args: Option<String>,
    use_shell: Option<bool>,
    show_error_popup: Option<bool>,
    auto_set_analyse_options: Option<bool>,
    delay: Option<i32>,
    extra_dll: Option<String>,
    auto_exit_seconds: Option<i32>,
    use_upx: Option<bool>,
}

fn resolve_migoto_path(app: &AppHandle, game_name: &str, config: &ThreeDMigotoConfig) -> Result<PathBuf, String> {
    if let Some(ref p) = config.install_dir {
        if !p.trim().is_empty() {
             return Ok(PathBuf::from(p));
        }
    }
    
    use crate::configs::app_config::AppConfig;
    let app_config = AppConfig::load().map_err(|e| format!("Failed to load app config: {}", e))?;
    let cache_dir = PathBuf::from(&app_config.cache_dir);
    if app_config.cache_dir.is_empty() {
        return Err("3Dmigoto Path not set and Cache Dir not set.".into());
    }
    Ok(cache_dir.join("3Dmigoto").join(game_name))
}

#[tauri::command]
pub async fn check_3dmigoto_integrity(app: AppHandle, game_name: String) -> Result<bool, String> {
    let games_dir = get_global_games_dir(&app);
    let game_dir = games_dir.join(&game_name);
    let config_path = game_dir.join("Config.json");

    if !config_path.exists() {
        return Err(format!("Config file not found: {:?}", config_path));
    }

    let config_content = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config: {}", e))?;
    
    let config_full: GameConfigFull = serde_json::from_str(&config_content)
        .map_err(|e| format!("Failed to parse config: {}", e))?;

    let migoto_config = config_full.three_d_migoto.unwrap_or(ThreeDMigotoConfig {
        install_dir: None,
        target_exe_path: None,
        launcher_exe_path: None,
        launch_args: None,
        use_shell: Some(false),
        show_error_popup: Some(false),
        auto_set_analyse_options: Some(false),
        delay: None,
        extra_dll: None,
        auto_exit_seconds: None,
        use_upx: None,
    });

    let migoto_path = resolve_migoto_path(&app, &game_name, &migoto_config)?;
    
    let d3d11 = migoto_path.join("d3d11.dll");
    let d3dx = migoto_path.join("d3dx.ini");

    Ok(d3d11.exists() && d3dx.exists())
}

#[tauri::command]
pub async fn start_game(app: AppHandle, game_name: String) -> Result<(), String> {
    let games_dir = get_global_games_dir(&app);
    let game_dir = games_dir.join(&game_name);
    let config_path = game_dir.join("Config.json");

    if !config_path.exists() {
        return Err(format!("Config file not found: {:?}", config_path));
    }

    let config_content = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config: {}", e))?;
    
    // Log the config content to verify structure if needed (optional)
    // println!("[GameLauncher] Config Content: {}", config_content);

    let config_full: GameConfigFull = serde_json::from_str(&config_content)
        .map_err(|e| format!("Failed to parse config: {}", e))?;

    let migoto_config = config_full.three_d_migoto.unwrap_or(ThreeDMigotoConfig {
        install_dir: None,
        target_exe_path: None,
        launcher_exe_path: None,
        launch_args: None,
        use_shell: Some(false),
        show_error_popup: Some(false),
        auto_set_analyse_options: Some(false),
        delay: None,
        extra_dll: None,
        auto_exit_seconds: None,
        use_upx: None,
    });

    let migoto_path = resolve_migoto_path(&app, &game_name, &migoto_config)?;
    
    // Check if target_exe_path is set and exists
    let target_exe = migoto_config.target_exe_path.as_deref().unwrap_or("");
    if target_exe.trim().is_empty() {
        return Err("游戏进程路径(Target Exe)未配置，请在设置中指定游戏可执行文件。".to_string());
    }

    // Verify if the executable actually exists
    let target_path = PathBuf::from(target_exe);
    if !target_path.exists() {
        return Err(format!("配置的游戏进程文件不存在: {}\n请检查路径是否正确。", target_exe));
    }

    // Copy essential boot files
    crate::utils::file_manager::copy_boot_files(&app, &migoto_path);
    
    // UPX Packing (after copying files)
    if migoto_config.use_upx.unwrap_or(false) {
        println!("[GameLauncher] UPX packing enabled. Packing d3d11.dll...");
        let d3d11_path = migoto_path.join("d3d11.dll");
        if d3d11_path.exists() {
             use tauri::Manager;
             let resource_dir = app.path().resource_dir().map_err(|e| format!("Failed to get resource dir for UPX: {}", e))?;
             let upx_exe = resource_dir.join("upx.exe");
             
             // Check if upx.exe exists
             let mut upx_to_use = upx_exe.clone();
             if !upx_to_use.exists() {
                 let dev_upx = PathBuf::from("resources").join("upx.exe");
                 if dev_upx.exists() {
                     upx_to_use = dev_upx; 
                 } else {
                     println!("[GameLauncher] Warning: upx.exe not found. Skipping packing.");
                 }
             }

             if upx_to_use.exists() {
                  println!("[GameLauncher] Running UPX: {:?} {:?}", upx_to_use, d3d11_path);
                  
                  // Execute UPX: upx.exe d3d11.dll
                  let output = Command::new(&upx_to_use)
                    .arg(&d3d11_path)
                    .output()
                    .map_err(|e| format!("Failed to execute UPX process: {}", e))?;
                 
                 let stdout = String::from_utf8_lossy(&output.stdout);
                 let stderr = String::from_utf8_lossy(&output.stderr);
                 
                 println!("[GameLauncher] UPX stdout: {}", stdout);
                 if !stderr.is_empty() {
                     println!("[GameLauncher] UPX stderr: {}", stderr);
                 }

                 if !output.status.success() {
                     println!("[GameLauncher] UPX failed with exit code: {:?}", output.status.code());
                 } else {
                     println!("[GameLauncher] UPX packing successful.");
                 }
             }
        }
    }

    let d3dx_path = migoto_path.join("d3dx.ini");
    if !d3dx_path.exists() {
        return Err(format!("d3dx.ini not found at {:?}", d3dx_path));
    }

    // Load INI
    let mut ini = IniManager::load(&d3dx_path)?;

    // 1. [Loader] target
    if let Some(target) = &migoto_config.target_exe_path {
        if !target.is_empty() {
             ini.set("Loader", "target", target);
        }
    }
    
    let run_shell = migoto_config.use_shell.unwrap_or(false);

    // 2. [Loader] launch & launch_args
    if run_shell {
        // If run_shell is TRUE, users request: "set launch and launch_args to empty (remove them)"
        ini.remove_key("Loader", "launch");
        ini.remove_key("Loader", "launch_args");
    } else {
        // If run_shell is FALSE, set them to valid paths
        if let Some(launch) = &migoto_config.launcher_exe_path {
            if !launch.is_empty() {
                ini.set("Loader", "launch", launch);
            } else {
                ini.remove_key("Loader", "launch");
            }
        } else {
             ini.remove_key("Loader", "launch");
        }
        
        if let Some(args) = &migoto_config.launch_args {
            if !args.is_empty() {
                ini.set("Loader", "launch_args", args);
            } else {
                ini.remove_key("Loader", "launch_args");
            }
        } else {
             ini.remove_key("Loader", "launch_args");
        }
    }

    // 3. [Logging] show_warnings
    let show_warnings = migoto_config.show_error_popup.unwrap_or(false); 
    ini.set("Logging", "show_warnings", if show_warnings { "1" } else { "0" });

    // 4. [Hunting] analyse_options
    if let Some(auto_set) = migoto_config.auto_set_analyse_options {
        if auto_set { 
             ini.set("Hunting", "analyse_options", "deferred_ctx_immediate dump_rt dump_cb dump_vb dump_ib buf txt dds dump_tex dds");
        }
    }

    // 5. [System] dll_initialization_delay - Maps to "delay" in JSON config
    if let Some(delay) = migoto_config.delay {
         ini.set("System", "dll_initialization_delay", &delay.to_string());
    }

    // 6. [Hunting] hunting = 2
    ini.set("Hunting", "hunting", "2");

    // 7. [Hunting] marking_actions
    ini.set("Hunting", "marking_actions", "clipboard asm hlsl");

    // 8. [Loader] delay - Maps to "autoExitSeconds" in JSON config
    if let Some(seconds) = migoto_config.auto_exit_seconds {
        ini.set("Loader", "delay", &seconds.to_string());
    }

    // 9. [Loader] inject_dll
    if let Some(dll) = &migoto_config.extra_dll {
        if !dll.is_empty() {
            ini.set("Loader", "inject_dll", dll);
        } else {
             ini.remove_key("Loader", "inject_dll");
        }
    } else {
        ini.remove_key("Loader", "inject_dll");
    }

    // Save changes
    ini.save()?;

    // 10. Always Launch Run.exe first
    // This is the 3DMigoto Loader that handles injection.
    let run_exe_name = "Run.exe";
    let run_exe = migoto_path.join(run_exe_name);
    if !run_exe.exists() {
        return Err(format!("{} not found in 3DMigoto directory: {:?}", run_exe_name, migoto_path));
    }
    
    // Launch Run.exe using PowerShell (fixes UAC focus issues)
    let run_script = format!(
        "Start-Process -FilePath '{}' -WorkingDirectory '{}'", 
        run_exe_name, 
        migoto_path.to_string_lossy()
    );
    
    Command::new("powershell")
       .args(&["-NoProfile", "-Command", &run_script])
       .spawn()
       .map_err(|e| format!("Failed to launch {}: {}", run_exe_name, e))?;

    // 11. If Shell Mode is enabled, we manually launch the game target
    // Run.exe won't do it because we removed the [Loader] launch key in INI.
    if run_shell {
        // Wait for Run.exe to initialize/UAC confirmation
        // A short delay ensures Run.exe is watching for the process before we start it.
        println!("[GameLauncher] Waiting for Run.exe to start...");

        let mut sys = System::new();
        let start_time = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(30); // 30 seconds timeout
        let mut found = false;

        // Path to look for
        let run_exe_path_str = run_exe.to_string_lossy().to_lowercase();

        while start_time.elapsed() < timeout {
             // In sysinfo 0.30, refresh_processes_specific is used for detailed updates
             // or refresh_processes for basic list.
             // We use refresh_processes_specific to get exe path if possible
             
             let refresh_kind = ProcessRefreshKind::new().with_exe(UpdateKind::Always);
             sys.refresh_processes_specifics(refresh_kind);

             for (_pid, process) in sys.processes() {
                 let proc_name = std::path::Path::new(process.name()).to_string_lossy().to_lowercase();
                 if proc_name == "run.exe" {
                     if let Some(exe_path) = process.exe() {
                         if exe_path.to_string_lossy().to_lowercase() == run_exe_path_str {
                             found = true;
                             break;
                         }
                     } else {
                         // Fallback: If we can't read the exe path (likely due to UAC),
                         // but the name matches, we assume it's our process.
                         found = true;
                         break;
                     }
                 }
            }
            
            if found {
                 println!("[GameLauncher] Run.exe detected!");
                 break;
            }

            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }

        if !found {
             println!("[GameLauncher] Timed out waiting for Run.exe. Launching game anyway...");
        } else {
             // Add a small buffer after detection
             tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }

        // Run Target Executable via Shell
        let exe_path = migoto_config.launcher_exe_path
             .or(migoto_config.target_exe_path)
             .ok_or("No executable path found for shell launch")?;

        if exe_path.is_empty() {
             return Err("Launch path is empty".into());
        }
        let args = migoto_config.launch_args.unwrap_or_default();
        
        let path_obj = PathBuf::from(&exe_path);
        let default_work_dir = PathBuf::from(".");
        let work_dir = path_obj.parent().unwrap_or(&default_work_dir);

        // Use Powershell Start-Process
        let mut ps_script = format!(
            "Start-Process -FilePath '{}' -WorkingDirectory '{}'", 
            exe_path, 
            work_dir.to_string_lossy()
        );
        
        if !args.is_empty() {
             ps_script.push_str(&format!(" -ArgumentList '{}'", args));
        }
        
        println!("[GameLauncher] Launching target via shell: {}", exe_path);
        Command::new("powershell")
           .args(&["-NoProfile", "-Command", &ps_script])
           .spawn()
           .map_err(|e| format!("Failed to launch shell command: {}", e))?;
    }

    Ok(())
}

#[tauri::command]
pub fn toggle_symlink(app: AppHandle, game_name: String, enable: bool) -> Result<(), String> {
    let games_dir = get_global_games_dir(&app);
    let game_dir = games_dir.join(&game_name);
    let config_path = game_dir.join("Config.json");

    if !config_path.exists() {
         return Err(format!("Config file not found: {:?}", config_path));
    }

    let config_content = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config: {}", e))?;
    let config_full: GameConfigFull = serde_json::from_str(&config_content)
        .map_err(|e| format!("Failed to parse config: {}", e))?;

    let migoto_config = config_full.three_d_migoto.ok_or("No 3DMigoto config found")?;
    
    // Resolve install_dir similar to start_game
    let install_dir = if let Some(ref p) = migoto_config.install_dir {
        if p.trim().is_empty() {
             use crate::configs::app_config::AppConfig;
             let app_config = AppConfig::load().map_err(|e| format!("Failed to load app config: {}", e))?;
             let cache_dir = PathBuf::from(&app_config.cache_dir);
             if app_config.cache_dir.is_empty() {
                 return Err("3Dmigoto Path not set and Cache Dir not set.".into());
             }
             cache_dir.join("3Dmigoto").join(&game_name)
        } else {
             PathBuf::from(p)
        }
    } else {
         use crate::configs::app_config::AppConfig;
         let app_config = AppConfig::load().map_err(|e| format!("Failed to load app config: {}", e))?;
         let cache_dir = PathBuf::from(&app_config.cache_dir);
         if app_config.cache_dir.is_empty() {
             return Err("3Dmigoto Path not set and Cache Dir not set.".into());
         }
         cache_dir.join("3Dmigoto").join(&game_name)
    };

    let ini_path = install_dir.join("d3dx.ini");
    if !ini_path.exists() {
        return Err(format!("d3dx.ini not found at {:?}", ini_path));
    }

    let mut ini = IniManager::load(&ini_path)?;
    
    // Value definition
    // Enable: "deferred_ctx_immediate dump_rt dump_cb dump_vb dump_ib buf txt dds dump_tex dds symlink"
    // Disable: "deferred_ctx_immediate dump_rt dump_cb dump_vb dump_ib buf txt dds dump_tex dds"
    
    let val_enable = "deferred_ctx_immediate dump_rt dump_cb dump_vb dump_ib buf txt dds dump_tex dds symlink";
    let val_disable = "deferred_ctx_immediate dump_rt dump_cb dump_vb dump_ib buf txt dds dump_tex dds";

    let target_value = if enable { val_enable } else { val_disable };

    ini.set("hunting", "analyse_options", target_value);
    
    ini.save()?;

    Ok(())
}
