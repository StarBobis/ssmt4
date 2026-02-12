use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use crate::commands::game_scanner::BGType;
use crate::utils::file_manager;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")] // Match JS conventions
#[serde(default)]
pub struct AppConfig {
    pub bg_type: BGType,
    pub bg_image: String,
    pub bg_video: String,
    pub sidebar_opacity: f64,
    pub sidebar_blur: f64,
    pub content_opacity: f64,
    pub content_blur: f64,
    pub cache_dir: String,
    pub current_config_name: String,
    pub window_width: f64,
    pub window_height: f64,
    #[serde(default)]
    pub github_token: String,
    
    // Page Visibility Settings
    #[serde(default)]
    pub show_mods: bool,
    #[serde(default)]
    pub show_workbench: bool,
    #[serde(default)]
    pub show_stickers: bool,
    #[serde(default)]
    pub show_websites: bool,
    #[serde(default)]
    pub show_documents: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            bg_type: BGType::Image,
            bg_image: "".to_string(), 
            bg_video: "".to_string(),
            sidebar_opacity: 0.3,
            sidebar_blur: 20.0,
            content_opacity: 0.0,
            content_blur: 0.0,
            cache_dir: "".to_string(),
            current_config_name: "Default".to_string(),
            window_width: 1000.0,
            window_height: 600.0,
            github_token: "".to_string(),            show_mods: true,            show_workbench: false,
            show_stickers: false,
            show_websites: false,
            show_documents: false,
        }
    }
}

impl AppConfig {
    pub fn new() -> Self {
        Self::default()
    }

    fn get_config_path() -> Option<PathBuf> {
        if let Some(mut dir) = file_manager::get_app_config_dir() {
            // Ensure directory exists
            if let Err(e) = file_manager::check_and_create_dir(&dir) {
                eprintln!("Failed to create config directory: {}", e);
                return None;
            }
            dir.push("settings.json");
            Some(dir)
        } else {
            None
        }
    }

    pub fn load() -> Result<Self, String> {
        let mut config = if let Some(path) = Self::get_config_path() {
            println!("Loading settings from: {:?}", path);
            if path.exists() {
                let content = fs::read_to_string(&path)
                    .map_err(|e| format!("Failed to read settings file: {}", e))?;
                
                println!("Settings content: {}", content);
                match serde_json::from_str::<Self>(&content) {
                    Ok(c) => c,
                    Err(e) => {
                         println!("Failed to parse settings, using defaults: {}", e);
                         Self::default()
                    }
                }
            } else {
                println!("Settings file does not exist, using defaults");
                Self::default()
            }
        } else {
            Self::default()
        };

        // Auto-initialize cache_dir if empty
        if config.cache_dir.is_empty() {
            if let Ok(local_data) = std::env::var("LOCALAPPDATA") {
                 let default_cache = PathBuf::from(local_data).join("SSMT4CachedFolder");
                 if !default_cache.exists() {
                     let _ = fs::create_dir_all(&default_cache);
                 }
                 config.cache_dir = default_cache.to_string_lossy().to_string();
                 // Save the initialized value immediately
                 let _ = config.save();
            }
        }
        
        Ok(config)
    }

    pub fn save(&self) -> Result<(), String> {
        if let Some(path) = Self::get_config_path() {
            println!("Saving settings to: {:?}", path);
            let content = serde_json::to_string_pretty(self)
                .map_err(|e| format!("Serialization error: {}", e))?;
            
            fs::write(path, content)
                .map_err(|e| format!("File write error: {}", e))?;
            Ok(())
        } else {
            Err("Could not determine config path".to_string())
        }
    }
}

