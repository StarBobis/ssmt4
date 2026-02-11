use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};
use crate::utils::file_manager::get_global_games_dir;
use crate::configs::app_config::AppConfig;
use serde::{Deserialize, Serialize};
use std::io::{Write, Cursor};

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


#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub version: String,
    pub description: String,
    pub download_url: String,
}

#[tauri::command]
pub async fn get_3dmigoto_latest_release(game_preset: String) -> Result<UpdateInfo, String> {
    let repo = match game_preset.as_str() {
        "GIMI" => "SilentNightSound/GIMI-Package",
        "HIMI" => "leotorrez/HIMI-Package",
        "SRMI" => "SpectrumQT/SRMI-Package",
        "ZZMI" => "leotorrez/ZZMI-Package",
        "WWMI" => "SpectrumQT/WWMI-Package",
        "EFMI" => "SpectrumQT/EFMI-Package",
        "AEMI" => "StarBobis/MinBase-Package",
        _ => return Err("Unsupported game preset for package update".to_string()),
    };

    let app_config = AppConfig::load().map_err(|e| e.to_string())?;
    let client = reqwest::Client::new();
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(reqwest::header::USER_AGENT, reqwest::header::HeaderValue::from_static("ssmt4-app"));
    
    if !app_config.github_token.is_empty() {
        let auth_value = format!("Bearer {}", app_config.github_token);
        let mut auth_header = reqwest::header::HeaderValue::from_str(&auth_value).map_err(|e| format!("Invalid token: {}", e))?;
        auth_header.set_sensitive(true);
        headers.insert(reqwest::header::AUTHORIZATION, auth_header);
    }

    let url = format!("https://api.github.com/repos/{}/releases/latest", repo);
    
    let resp = client.get(&url)
        .headers(headers)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;
        
    if !resp.status().is_success() {
        return Err(format!("GitHub API Error: {} - {}", resp.status(), resp.text().await.unwrap_or_default()));
    }

    let json: serde_json::Value = resp.json().await.map_err(|e| format!("Failed to parse JSON: {}", e))?;
    
    let tag_name = json.get("tag_name").and_then(|v| v.as_str()).ok_or("No tag_name found")?.to_string();
    let body = json.get("body").and_then(|v| v.as_str()).unwrap_or("No description").to_string();
    
    // Find asset
    let assets = json.get("assets").and_then(|v| v.as_array()).ok_or("No assets found")?;
    if assets.is_empty() {
        return Err("Release has no assets".to_string());
    }
    
    // Prefer zip
    let asset = assets.iter().find(|a| {
        a.get("name").and_then(|n| n.as_str()).map(|n| n.ends_with(".zip")).unwrap_or(false)
    }).or(assets.get(0)).ok_or("No suitable asset found")?;
    
    let download_url = asset.get("browser_download_url").and_then(|v| v.as_str()).ok_or("No download URL found")?.to_string();
    
    Ok(UpdateInfo {
        version: tag_name,
        description: body,
        download_url,
    })
}

#[tauri::command]
pub async fn install_3dmigoto_update(app: AppHandle, game_name: String, download_url: String) -> Result<(), String> {
    println!("[Update] Starting update for game: {}", game_name);
    
    // 1. Determine Target Directory
    let game_config = load_game_config(app.clone(), game_name.clone())?;
    
    let target_dir: PathBuf = if let Some(dir) = game_config.three_d_migoto.get("installDir").and_then(|v| v.as_str()).filter(|s| !s.is_empty()) {
        PathBuf::from(dir)
    } else {
        println!("[Update] installDir not set in config, trying cache fallback...");
        // Fallback to cache dir
        let app_config = AppConfig::load().map_err(|e| e.to_string())?;
        if !app_config.cache_dir.is_empty() {
             PathBuf::from(&app_config.cache_dir).join("3Dmigoto").join(&game_name)
        } else {
            return Err("Cannot update: 3Dmigoto installation directory is not set, and no Cache Directory configured.".to_string());
        }
    };

    println!("[Update] Target directory detected: {:?}", target_dir);

    // Create target dir if not exists (especially for cache fallback)
    if !target_dir.exists() {
        println!("[Update] Creating target directory: {:?}", target_dir);
        fs::create_dir_all(&target_dir).map_err(|e| format!("Failed to create target directory: {}", e))?;
    }
    
    println!("[Update] Downloading update from: {}", download_url);
    
    let client = reqwest::Client::new();
    let download_resp = client.get(&download_url).send().await.map_err(|e| format!("Download failed: {}", e))?;
    let bytes = download_resp.bytes().await.map_err(|e| format!("Failed to get bytes: {}", e))?;
    println!("[Update] Download complete. Size: {} bytes", bytes.len());
    
    // Unzip
    let reader = Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(reader).map_err(|e| format!("Failed to read zip: {}", e))?;
    
    println!("[Update] Extracting {} files...", archive.len());
    
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| format!("Zip error: {}", e))?;
        
        // Handle "Check for single top-level folder" logic if needed? 
        // For now, let's just strip known garbage if any, or just extract flat if needed. 
        // Actually standard packages usually are:
        // GIMI-Package/
        //   d3dx.ini
        //   ...
        // OR just flat.
        
        let outpath = match file.enclosed_name() {
             Some(path) => target_dir.join(path),
             None => continue,
        };

        println!("[Update] Extracting file: {:?}", outpath);

        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath).map_err(|e| format!("Failed to create dir: {}", e))?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(&p).map_err(|e| format!("Failed to create dir: {}", e))?;
                }
            }
            let mut outfile = fs::File::create(&outpath).map_err(|e| format!("Failed to create file: {}", e))?;
            std::io::copy(&mut file, &mut outfile).map_err(|e| format!("Failed to copy content: {}", e))?;
        }
    }
    
    println!("[Update] Extraction complete.");

    // Copy Resources
    println!("[Update] Copying essential resources...");
    let resource_dir = app.path().resource_dir().map_err(|e| format!("Failed to resolve resource dir: {}", e))?;
    println!("[Update] Resource Dir resolved to: {:?}", resource_dir);

    let files_to_copy = ["d3d11.dll", "d3dcompiler_47.dll", "Run.exe"];

    for filename in files_to_copy {
        let src_path = resource_dir.join(filename);
        let dest_path = target_dir.join(filename);
        
        println!("[Update] Copying {} -> {:?}", filename, dest_path);
        
        // Helper to check existence and copy
        let mut source_to_use = src_path.clone();
        if !source_to_use.exists() {
             // Fallback for dev environment
             let dev_path = PathBuf::from("resources").join(filename); // Try relative to cwd first? or src-tauri/resources
             let dev_path_2 = PathBuf::from("src-tauri/resources").join(filename);
             
             if dev_path.exists() {
                 source_to_use = dev_path;
             } else if dev_path_2.exists() {
                 source_to_use = dev_path_2;
             } else {
                 return Err(format!("Resource file '{}' not found. Checked: {:?}, {:?}, {:?}", filename, src_path, dev_path, dev_path_2));
             }
        }

        fs::copy(&source_to_use, &dest_path)
            .map_err(|e| format!("Failed to copy {}: {}.\nTip: Is the game or 3Dmigoto running? Please close it.", filename, e))?;
    }
    
    println!("[Update] All steps completed successfully.");

    Ok(())
}