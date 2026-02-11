use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

pub fn get_global_games_dir(app: &AppHandle) -> PathBuf {
    // 1. 获取 LocalAppData 路径: C:\Users\xxx\AppData\Local
    let local_data = app.path().local_data_dir().unwrap_or(PathBuf::from("."));
    
    // 2. 构造目标路径: ...\AppData\Local\SSMT4GlobalConfigs\Games
    let global_config_dir = local_data.join("SSMT4GlobalConfigs");
    let target_games_dir = global_config_dir.join("Games");
    
    // 3. 如果目录不存在，执行初始化拷贝逻辑
    if !target_games_dir.exists() {
        // 先尝试创建父目录
        if !global_config_dir.exists() {
            let _ = fs::create_dir_all(&global_config_dir);
        }
        
        // 查找源头（我们打包自带的 resources/Games）
        if let Some(source_games) = find_source_games_dir(app) {
             println!("Initializing Global Games Dir from: {:?} to {:?}", source_games, target_games_dir);
             let _ = copy_dir_recursive(&source_games, &target_games_dir);
        } else {
             // 没找到源，依然尝试创建空目录，防止报错
             let _ = fs::create_dir_all(&target_games_dir);
        }
    }

    target_games_dir
}

// 辅助：查找打包自带的资源目录
fn find_source_games_dir(app: &AppHandle) -> Option<PathBuf> {
    if let Ok(resource_dir) = app.path().resource_dir() {
        let p = resource_dir.join("Games");
        if p.exists() { return Some(p); }
    }
    
    if let Ok(mut exec_dir) = std::env::current_exe() {
        exec_dir.pop(); 
        
        let p1 = exec_dir.join("resources").join("Games");
        if p1.exists() { return Some(p1); }

        let p2 = exec_dir.join("Games");
        if p2.exists() { return Some(p2); }
    }
    
    // Dev environment fallback
    if Path::new("Games").exists() {
        return Some(PathBuf::from("Games"));
    }
    
    None
}

// 递归拷贝目录，如果目标文件存在则跳过
fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if ty.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            // 如果文件已存在，则跳过（要求不要替换）
            if !dst_path.exists() {
                fs::copy(&src_path, &dst_path)?;
            }
        }
    }
    Ok(())
}

pub fn check_and_create_dir(path: &Path) -> std::io::Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

pub fn get_app_config_dir() -> Option<PathBuf> {
    // Check for LOCALAPPDATA environment variable (Windows)
    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        let mut path = PathBuf::from(local_app_data);
        path.push("SSMT4Configs");
        return Some(path);
    }
    
    // Simple fallback for Linux/Mac if needed, though user is on Windows
    if let Ok(home) = std::env::var("HOME") {
        let mut path = PathBuf::from(home);
        path.push(".config");
        path.push("SSMT4Configs");
        return Some(path);
    }

    None
}
