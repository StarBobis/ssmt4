use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
pub fn get_resource_path(app: AppHandle, filename: String) -> Result<String, String> {
    let resource_dir = app.path().resource_dir().map_err(|e| e.to_string())?;
    let path = resource_dir.join(&filename);
    
    if path.exists() {
        return Ok(path.to_string_lossy().to_string());
    }

    // Fallback for dev environment (try looking in local resources folder)
    let dev_path = PathBuf::from("resources").join(&filename);
    if dev_path.exists() {
        // Returns absolute path
        if let Ok(abs_path) = fs::canonicalize(&dev_path) {
            return Ok(abs_path.to_string_lossy().to_string());
        }
    }
    
    // Fallback 2: src-tauri/resources
    let dev_path2 = PathBuf::from("src-tauri/resources").join(&filename);
    if dev_path2.exists() {
         if let Ok(abs_path) = fs::canonicalize(&dev_path2) {
            return Ok(abs_path.to_string_lossy().to_string());
        }
    }

    Err(format!("Resource '{}' not found in {:?}", filename, resource_dir))
}

#[tauri::command]
pub fn ensure_directory(path: String) -> Result<(), String> {
    let p = PathBuf::from(&path);
    if !p.exists() {
        fs::create_dir_all(&p).map_err(|e| format!("Failed to create directory: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
pub fn open_in_explorer(path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(path)
            .spawn()
            .map_err(|e| format!("Failed to open explorer: {}", e))?;
    }
    #[cfg(not(target_os = "windows"))]
    {
        // Simple fallback for linux/mac if needed later
        // or explicitly fail
    }
    Ok(())
}
