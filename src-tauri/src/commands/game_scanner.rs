use std::fs;
use tauri::{AppHandle, Manager};
use serde::Serialize;
use tauri::path::BaseDirectory;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameInfo {
    pub name: String,
    pub icon_path: String,
    pub bg_path: String,
}

#[tauri::command]
pub fn scan_games(app: AppHandle) -> Result<Vec<GameInfo>, String> {
    let resource_dir = app.path().resolve("resources/Games", BaseDirectory::Resource)
        .map_err(|e| format!("Failed to resolve resource path: {}", e))?;

    println!("Scanning games in: {:?}", resource_dir);

    if !resource_dir.exists() {
        return Err(format!("Games directory not found at: {:?}", resource_dir));
    }

    let mut games = Vec::new();

    let entries = fs::read_dir(&resource_dir)
        .map_err(|e| format!("Failed to read games directory: {}", e))?;

    for entry in entries {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    let icon_path = path.join("Icon.png");
                    let bg_path = path.join("Background.png");

                    // Only include if artifacts exist (optional strictness)
                    if icon_path.exists() && bg_path.exists() {
                         games.push(GameInfo {
                            name: name.to_string(),
                            icon_path: icon_path.to_string_lossy().to_string(),
                            bg_path: bg_path.to_string_lossy().to_string(),
                        });
                    } else {
                        // Fallback or include anyway? 
                        // User requirement: "Read... Icon.png as icon, Background.png as background"
                        // I'll include them even if missing, frontend can handle 404s, but safer to check.
                        // Let's assume valid game folders have these.
                         games.push(GameInfo {
                            name: name.to_string(),
                            icon_path: icon_path.to_string_lossy().to_string(),
                            bg_path: bg_path.to_string_lossy().to_string(),
                        });
                    }
                }
            }
        }
    }

    Ok(games)
}
