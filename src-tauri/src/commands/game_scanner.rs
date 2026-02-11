use std::fs;
use tauri::{AppHandle, Manager};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameInfo {
    pub name: String,
    pub icon_path: String,
    pub bg_path: String,
    pub bg_video_path: Option<String>,
    pub show_sidebar: bool,
}

#[derive(Serialize, Deserialize)]
struct GameIconSetting {
    #[serde(rename = "GameName")]
    game_name: String,
    #[serde(rename = "Show")]
    show: bool,
}

#[derive(Serialize, Deserialize)]
struct GameIconConfig {
    #[serde(rename = "GameIconSettingList")]
    list: Vec<GameIconSetting>,
}

// Function to find games directory (shared logic)
fn find_games_dir(app: &AppHandle) -> PathBuf {
    // 1. 尝试标准的资源目录 (resources/Games)
    if let Ok(resource_dir) = app.path().resource_dir() {
        let p = resource_dir.join("Games");
        if p.exists() {
            return p;
        }
    }

    // 2. 尝试从 executable 旁边的 resources/Games 或直接 Games
    if let Ok(mut exec_dir) = std::env::current_exe() {
        exec_dir.pop(); // remove executable name
        
        let p1 = exec_dir.join("resources").join("Games");
        if p1.exists() {
            return p1;
        }

        let p2 = exec_dir.join("Games");
        if p2.exists() {
            return p2;
        }
    }

    // 3. 开发环境/默认 fallback
    PathBuf::from("Games")
}

// Normalize paths: strip Windows extended prefix and force forward slashes so convertFileSrc gets a POSIX-ish path
fn normalize_path(p: &Path) -> String {
    let mut s = p.to_string_lossy().to_string();
    #[cfg(windows)]
    {
        if s.starts_with("\\\\?\\") {
            s = s.trim_start_matches("\\\\?\\").to_string();
        }
        s = s.replace('\\', "/");
    }
    s
}

#[tauri::command]
pub fn scan_games(app: AppHandle) -> Result<Vec<GameInfo>, String> {
    let games_dir = find_games_dir(&app);
    println!("Scanning games in: {}", normalize_path(&games_dir));

    // Load GameIconConfig.json if it exists
    let mut sidebar_config: HashMap<String, bool> = HashMap::new();
    let config_path = games_dir.join("GameIconConfig.json");
    if config_path.exists() {
        if let Ok(content) = fs::read_to_string(&config_path) {
            if let Ok(parsed) = serde_json::from_str::<GameIconConfig>(&content) {
                for item in parsed.list {
                    sidebar_config.insert(item.game_name, item.show);
                }
            } else {
                eprintln!("Failed to parse GameIconConfig.json");
            }
        }
    }

    let mut games = Vec::new();
    let entries = fs::read_dir(&games_dir)
        .map_err(|e| format!("Failed to read games directory: {}", e))?;

    for entry in entries {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    
                    // 构建图片路径，这里直接使用 path.join
                    let icon_path = path.join("Icon.png");
                    let bg_path = path.join("Background.png");
                    
                    // Check for video
                    let mut video_path = path.join("Background.mp4");
                    if !video_path.exists() {
                        video_path = path.join("Background.webm");
                    }
                    let video_str = if video_path.exists() {
                        Some(normalize_path(&video_path))
                    } else {
                        None
                    };

                    let icon_str = normalize_path(&icon_path);
                    let bg_str = normalize_path(&bg_path);

                    // 不论文件是否存在都加入列表，让前端处理加载失败
                    // 但可以在这里打印日志方便调试
                    if !icon_path.exists() {
                        println!("Warning: Icon missing for {}: {:?}", name, icon_path);
                    }
                    if !bg_path.exists() {
                        println!("Warning: Background missing for {}: {:?}", name, bg_path);
                    }

                    // Determine show_sidebar status
                    let show_sidebar = *sidebar_config.get(name).unwrap_or(&false);

                    games.push(GameInfo {
                        name: name.to_string(),
                        icon_path: icon_str,
                        bg_path: bg_str,
                        bg_video_path: video_str,
                        show_sidebar,
                    });
                }
            }
        }
    }

    Ok(games)
}

#[tauri::command]
pub fn set_game_visibility(app: AppHandle, game_name: String, visible: bool) -> Result<(), String> {
    let games_dir = find_games_dir(&app);
    let config_path = games_dir.join("GameIconConfig.json");

    // Read existing config or create new
    let mut config = if config_path.exists() {
        let content = fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;
        serde_json::from_str::<GameIconConfig>(&content)
            .map_err(|e| format!("Failed to parse config file: {}", e))?
    } else {
        GameIconConfig { list: Vec::new() }
    };

    // Update or Add entry
    if let Some(entry) = config.list.iter_mut().find(|x| x.game_name == game_name) {
        entry.show = visible;
    } else {
        config.list.push(GameIconSetting {
            game_name,
            show: visible,
        });
    }

    // Write back
    let new_content = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    fs::write(&config_path, new_content)
        .map_err(|e| format!("Failed to write config file: {}", e))?;

    Ok(())
}

