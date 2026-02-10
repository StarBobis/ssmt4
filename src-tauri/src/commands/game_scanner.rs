use std::fs;
use tauri::{AppHandle, Manager};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameInfo {
    pub name: String,
    pub icon_path: String,
    pub bg_path: String,
    pub bg_video_path: Option<String>,
}

#[derive(Deserialize)]
struct GameIconSetting {
    #[serde(rename = "GameName")]
    game_name: String,
    #[serde(rename = "Show")]
    show: bool,
}

#[derive(Deserialize)]
struct GameIconConfig {
    #[serde(rename = "GameIconSettingList")]
    list: Vec<GameIconSetting>,
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
    // 尝试多个可能的路径来定位 Games 目录
    let mut games_dir: Option<PathBuf> = None;

    // 1. 尝试标准的资源目录 (resources/Games)
    if let Ok(resource_dir) = app.path().resource_dir() {
        let p = resource_dir.join("resources").join("Games");
        if p.exists() {
            games_dir = Some(p);
        } else {
            // 2. 尝试资源目录的根 (Games)
            let p2 = resource_dir.join("Games");
            if p2.exists() {
                games_dir = Some(p2);
            }
        }
    }

    // 3. 开发环境回退：尝试从当前工作目录查找 (D:\Dev\ssmt4\src-tauri\resources\Games)
    if games_dir.is_none() {
        // 假设 CWD 是项目根目录
        let dev_p = Path::new("src-tauri/resources/Games");
        if let Ok(abs_dev) = std::fs::canonicalize(dev_p) {
             if abs_dev.exists() { games_dir = Some(abs_dev); }
        }
        
         // 假设 CWD 是 src-tauri
        let dev_p2 = Path::new("resources/Games");
        if let Ok(abs_dev) = std::fs::canonicalize(dev_p2) {
             if abs_dev.exists() { games_dir = Some(abs_dev); }
        }
    }

    let games_dir = games_dir.ok_or_else(|| "Games directory not found".to_string())?;

    println!("Scanning games in: {}", normalize_path(&games_dir));

    // Previous GameIconConfig.json logic removed as per request to show all games.

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

                    games.push(GameInfo {
                        name: name.to_string(),
                        icon_path: icon_str,
                        bg_path: bg_str,
                        bg_video_path: video_str,
                    });
                }
            }
        }
    }

    Ok(games)
}
