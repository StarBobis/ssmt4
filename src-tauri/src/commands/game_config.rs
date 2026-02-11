use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct BasicSettings {
    pub game_preset: String,
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
    // Helper from game_scanner (but private there), so we duplicate or move logic. 
    // Ideally we share "find_games_dir" logic.
    // For now, let's look at how "scan_games" finds it. 
    // It's better to expose "find_games_dir" from game_scanner or move it to common utils.
    // Or just re-implement simple check for now since I can't easily refactor shared code without multiple edits.
    
    // We will assume the standard location logic:
    // 1. resource_dir/Games
    // 2. adjacent Games folder
    
    let mut games_dir = PathBuf::from("Games"); // Default fallback
    
    if let Ok(resource_dir) = app.path().resource_dir() {
        let p = resource_dir.join("Games");
        if p.exists() {
            games_dir = p;
        }
    }
    
    // Check adjacent to exe if not found in resources (dev mode fallback mostly)
    if !games_dir.exists() {
         if let Ok(mut exec_dir) = std::env::current_exe() {
            exec_dir.pop(); 
            let p1 = exec_dir.join("resources").join("Games");
            if p1.exists() { games_dir = p1; }
            else if exec_dir.join("Games").exists() { games_dir = exec_dir.join("Games"); }
        }
    }
    
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
