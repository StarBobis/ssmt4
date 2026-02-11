use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};
use crate::utils::file_manager::get_global_games_dir;
use serde::{Deserialize, Serialize}; // Added missing import
use std::io::Write; // check imports

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BasicSettings {
    pub game_preset: String,
    #[serde(default = "default_bg_type")]
    pub background_type: String, 
}

fn default_bg_type() -> String {
    "image".to_string()
}

impl Default for BasicSettings {
    fn default() -> Self {
        Self {
            game_preset: "Default".to_string(),
            background_type: "image".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct GameConfig {
    #[serde(default)]
    pub basic: BasicSettings,
    #[serde(default)]
    pub three_d_migoto: serde_json::Value,
    #[serde(default)]
    pub other: serde_json::Value,
}

fn get_game_config_path(app: &AppHandle, game_name: &str) -> PathBuf {
    let games_dir = get_global_games_dir(app);
    games_dir.join(game_name).join("Config.json")
}

#[tauri::command]
pub fn load_game_config(app: AppHandle, game_name: String) -> Result<GameConfig, String> {
    let config_path = get_game_config_path(&app, &game_name);
    
    if config_path.exists() {
        let content = fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config: {}", e))?;
        
        let config: GameConfig = serde_json::from_str(&content)
             .map_err(|e| format!("Failed to parse config: {}", e))?;
             
        Ok(config)
    } else {
        // Return default if not exists
        Ok(GameConfig::default())
    }
}

#[tauri::command]
pub fn save_game_config(app: AppHandle, game_name: String, config: GameConfig) -> Result<(), String> {
    let config_path = get_game_config_path(&app, &game_name);
    
    // Ensure parent dir exists (it should, since game exists, but just in case)
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }
    
    let content = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
        
    fs::write(config_path, content)
        .map_err(|e| format!("Failed to write config: {}", e))?;
        
    Ok(())
}

#[tauri::command]
pub fn create_new_config(app: AppHandle, new_name: String, config: GameConfig) -> Result<(), String> {
    // Reuse specific path logic or just construct it
    // We assume "Games/<new_name>/Config.json"
    
    // Find Games dir root
    let games_root = get_game_config_path(&app, "Dummy").parent().unwrap().parent().unwrap().to_path_buf();
    
    let new_dir = games_root.join(&new_name);
    if !new_dir.exists() {
        fs::create_dir_all(&new_dir)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }
    
    let config_path = new_dir.join("Config.json");
    let content = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
        
    fs::write(config_path, content)
        .map_err(|e| format!("Failed to write config: {}", e))?;

    Ok(())
}

#[tauri::command]
pub fn delete_game_config_folder(app: AppHandle, game_name: String) -> Result<(), String> {
    let config_path = get_game_config_path(&app, &game_name);
    let game_dir = config_path.parent().ok_or("Invalid path")?;
    
    if game_dir.exists() {
        fs::remove_dir_all(game_dir)
             .map_err(|e| format!("Failed to delete directory: {}", e))?;
    }
    
    Ok(())
}

#[tauri::command]
pub fn set_game_background(app: AppHandle, game_name: String, file_path: String, bg_type: String) -> Result<(), String> {
    let config_path = get_game_config_path(&app, &game_name);
    let game_dir = config_path.parent().ok_or("Invalid game path")?;
    
    let source_path = PathBuf::from(&file_path);
    if !source_path.exists() {
        return Err(format!("Source file does not exist: {}", file_path));
    }
    
    let extension = source_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
        
    let target_name = format!("Background.{}", extension);
    let target_path = game_dir.join(&target_name);

    if bg_type == "image" {
        let candidates = ["Background.png", "Background.webp", "Background.jpg", "Background.jpeg"]; 
        for c in candidates {
            let p = game_dir.join(c);
            if p.exists() { let _ = fs::remove_file(p); }
        }
    } else if bg_type == "video" {
        let candidates = ["Background.mp4", "Background.webm", "Background.mkv"]; 
        for c in candidates {
            let p = game_dir.join(c);
            if p.exists() { let _ = fs::remove_file(p); }
        }
    } else {
        return Err(format!("Unknown background type: {}", bg_type));
    }

    fs::copy(&source_path, &target_path)
        .map_err(|e| format!("Failed to copy file from {} to {:?}: {}", file_path, target_path, e))?;
        
    // Update Config.json with the new type
    let mut config = load_game_config(app.clone(), game_name.clone()).unwrap_or(GameConfig::default());
    config.basic.background_type = bg_type;
    save_game_config(app.clone(), game_name, config).map_err(|e| format!("Failed to update config: {}", e))?;

    Ok(())
}

#[tauri::command]
pub fn set_game_icon(app: AppHandle, game_name: String, file_path: String) -> Result<(), String> {
    let config_path = get_game_config_path(&app, &game_name);
    let game_dir = config_path.parent().ok_or("Invalid game path")?;
    
    let source_path = PathBuf::from(&file_path);
    if !source_path.exists() { return Err(format!("File not found: {}", file_path)); }
    
    let target_path = game_dir.join("Icon.png");
    
    if target_path.exists() {
        let _ = fs::remove_file(&target_path);
    }
    
    fs::copy(&source_path, &target_path)
        .map_err(|e| format!("Failed to copy icon: {}", e))?;
        
    Ok(())
}
#[tauri::command]
pub async fn update_game_background(app: AppHandle, game_name: String, game_preset: String, bg_type: String) -> Result<(), String> {
    let game_id = match game_preset.as_str() {
        "GIMI" => "1Z8W5NHUQb",
        "HIMI" => "osvnlOc0S8",
        "SRMI" => "64kMb5iAWu",
        "ZZMI" => "x6znKlJ0xK",
        _ => return Err("Unsupported game preset for auto-update".to_string()),
    };

    let url = format!("https://hyp-api.mihoyo.com/hyp/hyp-connect/api/getAllGameBasicInfo?launcher_id=jGHBHlcOq1&language=zh-cn&game_id={}", game_id);
    
    let resp = reqwest::get(&url).await.map_err(|e| format!("Request failed: {}", e))?;
    let json: serde_json::Value = resp.json().await.map_err(|e| format!("Failed to parse JSON: {}", e))?;
    
    // Parse
    let data = json.get("data").ok_or("Missing data field")?;
    let list = data.get("game_info_list").and_then(|v| v.as_array()).ok_or("Missing game_info_list")?;
    let first = list.get(0).ok_or("Empty game info list")?;
    let backgrounds = first.get("backgrounds").and_then(|v| v.as_array()).ok_or("Missing backgrounds")?;
    let first_bg = backgrounds.get(0).ok_or("Empty backgrounds list")?;
    
    let target_url = if bg_type == "video" {
         first_bg.get("video")
            .and_then(|v| v.get("url"))
            .and_then(|v| v.as_str())
            .ok_or("No video URL found")?
    } else {
         first_bg.get("background")
            .and_then(|v| v.get("url"))
            .and_then(|v| v.as_str())
            .ok_or("No image URL found")?
    };
    
    println!("Downloading background from: {}", target_url);
    
    // Download
    let download_resp = reqwest::get(target_url).await.map_err(|e| format!("Download failed: {}", e))?;
    let bytes = download_resp.bytes().await.map_err(|e| format!("Failed to get bytes: {}", e))?;
    
    // Save
    let config_path = get_game_config_path(&app, &game_name);
    let game_dir = config_path.parent().ok_or("Invalid path")?;
    
    // Determine filename extension from URL or just use strict defaults
    // URL might not have extension if signed? usually it does.
    let ext = if bg_type == "video" { "mp4" } else { "png" }; // Default fallback
    // Try to get from url
    let url_path = std::path::Path::new(target_url);
    let url_ext = url_path.extension().and_then(|s| s.to_str()).unwrap_or(ext);
    
    let filename = format!("Background.{}", url_ext);
    let target_path = game_dir.join(&filename);
    
    // Clean old
    if bg_type == "image" {
        let candidates = ["Background.png", "Background.webp", "Background.jpg", "Background.jpeg"]; 
        for c in candidates {
            let p = game_dir.join(c);
            if p.exists() { let _ = fs::remove_file(p); }
        }
    } else {
        // video
        let candidates = ["Background.mp4", "Background.webm", "Background.mkv"]; 
        for c in candidates {
            let p = game_dir.join(c);
            if p.exists() { let _ = fs::remove_file(p); }
        }
    }
    
    fs::write(&target_path, bytes).map_err(|e| format!("Failed to write file: {}", e))?;
    
    // Update config
    let mut config = load_game_config(app.clone(), game_name.clone()).unwrap_or(GameConfig::default());
    config.basic.background_type = bg_type.clone();
    save_game_config(app.clone(), game_name, config).map_err(|e| format!("Failed to save config: {}", e))?;
    
    Ok(())
}