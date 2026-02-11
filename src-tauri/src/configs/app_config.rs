use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use crate::utils::file_manager;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")] // Match JS conventions
#[serde(default)]
pub struct AppConfig {
    pub bg_type: String,
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
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            bg_type: "image".to_string(),
            bg_image: "/background.png".to_string(), // Keep defaults similar to frontend
            bg_video: "/background.webm".to_string(),
            sidebar_opacity: 0.3,
            sidebar_blur: 20.0,
            content_opacity: 0.2,
            content_blur: 3.0,
            cache_dir: "".to_string(),
            current_config_name: "Default".to_string(),
            window_width: 1000.0,
            window_height: 600.0,
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
        if let Some(path) = Self::get_config_path() {
            println!("Loading settings from: {:?}", path);
            if path.exists() {
                let content = fs::read_to_string(&path)
                    .map_err(|e| format!("Failed to read settings file: {}", e))?;
                
                println!("Settings content: {}", content);
                let config = serde_json::from_str::<Self>(&content)
                    .map_err(|e| format!("Failed to parse settings.json: {}", e))?;
                    
                println!("Loaded config successfully: {:?}", config);
                return Ok(config);
            } else {
                println!("Settings file does not exist, using defaults");
                Ok(Self::default())
            }
        } else {
            Ok(Self::default())
        }
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

