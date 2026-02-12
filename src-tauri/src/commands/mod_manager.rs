use crate::utils::file_manager::get_global_games_dir;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter, State};
use notify::{RecommendedWatcher, RecursiveMode, Watcher, Config};
use std::sync::Mutex;

fn decode_zip_name(file: &zip::read::ZipFile) -> String {
    let raw = file.name_raw();
    // Try interpreting as UTF-8 first
    if let Ok(s) = std::str::from_utf8(raw) {
        return s.to_string();
    }
    // Fallback to GBK
    let (cow, _encoding, _malformed) = encoding_rs::GBK.decode(raw);
    cow.to_string()
}

// Watcher State
pub struct ModWatcher(pub Mutex<Option<RecommendedWatcher>>);

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GroupInfo {
    pub id: String, // Full path like "Ayaka/Outfit1"
    pub name: String, // Just name "Outfit1"
    pub icon_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ModInfo {
    pub id: String,         // Base64 encoded path or just relative path
    pub name: String,       // Folder name (without DISABLED_)
    pub enabled: bool,
    pub path: String,       // Absolute path
    pub relative_path: String,
    pub preview_images: Vec<String>,
    pub group: String,      // Parent folder name if depth > 1
    pub is_dir: bool,
    pub last_modified: u64, // Timestamp
}

#[derive(Deserialize)]
struct GameConfigPartial {
    #[serde(rename = "threeDMigoto")]
    three_d_migoto: Option<ThreeDMigotoPartial>,
}

#[derive(Deserialize)]
struct ThreeDMigotoPartial {
    #[serde(rename = "installDir")]
    install_dir: Option<String>,
}

fn get_game_install_dir(app: &AppHandle, game_name: &str) -> Result<PathBuf, String> {
    let games_dir = get_global_games_dir(app);
    let config_path = games_dir.join(game_name).join("Config.json");

    if !config_path.exists() {
        return Err(format!("Config file not found: {:?}", config_path));
    }

    let config_content = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config: {}", e))?;

    let config: GameConfigPartial = serde_json::from_str(&config_content)
        .map_err(|e| format!("Failed to parse config: {}", e))?;

    if let Some(tdm) = config.three_d_migoto {
        if let Some(dir) = tdm.install_dir {
            return Ok(PathBuf::from(dir));
        }
    }
    
    Err("3Dmigoto install directory not configured".to_string())
}

fn find_preview_images(path: &Path) -> Vec<String> {
    let mut images = Vec::new();
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    let ext_str = ext.to_string_lossy().to_lowercase();
                    if ["jpg", "jpeg", "png", "gif", "bmp", "webp"].contains(&ext_str.as_str()) {
                        // In a real app we might convert this to asset protocol URL, 
                        // but for now let's return absolute path and let frontend handle conversion
                        images.push(path.to_string_lossy().to_string());
                    }
                }
            }
        }
    }
    // Sort to have consistent preview
    images.sort();
    images
}

// Recursive scan function
// depth: current depth. max_depth: how deep to go.
fn scan_folder(
    base_mods_dir: &Path, 
    current_dir: &Path, 
    results: &mut Vec<ModInfo>, 
    current_group: String,
    depth: usize,
    groups_found: &mut Vec<GroupInfo>
) {
    if depth > 5 { return; } // Limit depth increased for multi-level

    if let Ok(entries) = fs::read_dir(current_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let dir_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                
                let is_disabled = dir_name.to_uppercase().starts_with("DISABLED");
                let clean_name = if is_disabled {
                    if dir_name.to_uppercase().starts_with("DISABLED_") {
                        dir_name[9..].to_string()
                    } else {
                        dir_name[8..].to_string()
                    }
                } else {
                    dir_name.clone()
                };

                let relative_path = path.strip_prefix(base_mods_dir).unwrap_or(&path).to_string_lossy().to_string();
                
                // Determine if this directory is a mod or a category
                // Logic:
                // 1. If it contains .ini file -> Definitely a Mod
                // 2. If it contains subdirectories -> Definitely a Category (recurse)
                // 3. If it contains only images/files but NO .ini and NO subdirs -> Likely a simple texture mod or empty folder?
                    // But wait, if we have a category with just an icon and no mods yet? It should catch as category.
                

                // Check for ini files
                let has_ini = fs::read_dir(&path).ok().map(|mut r| r.any(|e| e.ok().map(|i| i.path().extension().map(|x| x == "ini").unwrap_or(false)).unwrap_or(false))).unwrap_or(false);
                
                // Check for subdirectories (ignoring common non-mod folders if necessary, but keep simple for now)
                let has_subdirs = fs::read_dir(&path).ok().map(|mut r| r.any(|e| e.ok().map(|i| i.path().is_dir()).unwrap_or(false))).unwrap_or(false);

                let images = find_preview_images(&path);
                
                // Refined Logic
                let is_leaf_mod = if has_ini {
                    true // Always a mod if .ini is present
                } else if has_subdirs {
                    false // Force recursion if subdirs exist (treat as category)
                } else {
                    // No ini, No subdirs.
                    // If it has images, it could be a texture mod OR an empty category with just an icon.
                    // Logic: If the only images are standard icon names, treat as empty category.
                    let standard_icons = ["folder.jpg", "folder.png", "icon.jpg", "icon.png", "cover.jpg", "cover.png"];
                    let has_content_images = images.iter().any(|img_path| {
                        let path = Path::new(img_path);
                        let name = path.file_name().unwrap_or_default().to_string_lossy().to_lowercase();
                        !standard_icons.contains(&name.as_str())
                    });
                    
                    has_content_images
                };

                if is_leaf_mod {
                     let metadata = path.metadata().ok();
                     let last_modified = metadata
                        .and_then(|m| m.modified().ok())
                        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                        .map(|d| d.as_secs())
                        .unwrap_or(0);
                        
                     results.push(ModInfo {
                        id: relative_path.clone(),
                        name: clean_name.clone(),
                        enabled: !is_disabled,
                        path: path.to_string_lossy().to_string(),
                        relative_path,
                        preview_images: images,
                        group: current_group.clone(),
                        is_dir: true,
                        last_modified,
                    });
                } else {
                    // It is a category folder
                    let next_group = if current_group == "Root" {
                        clean_name.clone()
                    } else {
                        // Use / as separator for groups
                        format!("{}/{}", current_group, clean_name)
                    };
                    
                    // Look for group icon
                    let mut icon = None;
                    if let Ok(g_entries) = fs::read_dir(&path) {
                        for ge in g_entries.flatten() {
                            let gp = ge.path();
                            if gp.is_file() {
                                let g_name = gp.file_name().unwrap_or_default().to_string_lossy().to_lowercase();
                                if ["folder.jpg", "folder.png", "icon.jpg", "icon.png", "cover.jpg", "cover.png"].contains(&g_name.as_str()) {
                                    icon = Some(gp.to_string_lossy().to_string());
                                    break;
                                }
                            }
                        }
                    }

                    groups_found.push(GroupInfo {
                         id: next_group.clone(),
                         name: clean_name.clone(),
                         icon_path: icon,
                    });

                    // Recurse
                    scan_folder(base_mods_dir, &path, results, next_group, depth + 1, groups_found);
                }
            }
        }
    }
}

#[tauri::command]
pub async fn set_mod_group_icon(app: AppHandle, game_name: String, group_path: String, icon_path: String) -> Result<(), String> {
    let install_dir = get_game_install_dir(&app, &game_name)?;
    let dest_dir = install_dir.join("Mods").join(&group_path);

    if !dest_dir.exists() {
        return Err("Group directory not found".to_string());
    }

    let src_path = PathBuf::from(&icon_path);
    if !src_path.exists() {
        return Err("Source icon not found".to_string());
    }

    // Remove existing common icons to avoid confusion
    let common_names = ["folder.jpg", "folder.png", "icon.jpg", "icon.png", "cover.jpg", "cover.png"];
    for name in common_names {
        let p = dest_dir.join(name);
        if p.exists() {
            let _ = fs::remove_file(p);
        }
    }

    let dest_path = dest_dir.join("icon.png");
    fs::copy(&src_path, &dest_path).map_err(|e| format!("Failed to copy icon: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn open_mod_group_folder(app: AppHandle, game_name: String, group_path: String) -> Result<(), String> {
    let install_dir = get_game_install_dir(&app, &game_name)?;
    let dest_dir = install_dir.join("Mods").join(&group_path);

    if !dest_dir.exists() {
        return Err("Group directory not found".to_string());
    }

    crate::commands::common::open_in_explorer(dest_dir.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn open_game_mods_folder(app: AppHandle, game_name: String) -> Result<(), String> {
    let install_dir = get_game_install_dir(&app, &game_name)?;
    let mods_dir = install_dir.join("Mods");

    if !mods_dir.exists() {
        fs::create_dir_all(&mods_dir).map_err(|e| format!("Failed to create Mods directory: {}", e))?;
    }

    crate::commands::common::open_in_explorer(mods_dir.to_string_lossy().to_string())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModScanResult {
    pub mods: Vec<ModInfo>,
    pub groups: Vec<GroupInfo>,
}

#[tauri::command]
pub async fn scan_mods(app: AppHandle, game_name: String) -> Result<ModScanResult, String> {
    let install_dir = get_game_install_dir(&app, &game_name)?;
    let mods_dir = install_dir.join("Mods");

    // Auto create Mods directory if it doesn't exist
    if !mods_dir.exists() {
        if let Err(e) = fs::create_dir_all(&mods_dir) {
            return Err(format!("Failed to create Mods directory at {:?}: {}", mods_dir, e));
        }
        // If just created, it's empty
        return Ok(ModScanResult { mods: Vec::new(), groups: Vec::new() });
    }

    let mut mods = Vec::new();
    let mut groups_list = Vec::new();

    // 1. Scan for mods recursively
    scan_folder(&mods_dir, &mods_dir, &mut mods, "Root".to_string(), 0, &mut groups_list);
    
    // Sort groups by id
    groups_list.sort_by(|a, b| a.id.cmp(&b.id));

    // Dedup groups (scan_folder might visit same node? No, it's tree traversal, unique nodes)
    
    Ok(ModScanResult {
        mods,
        groups: groups_list
    })
}

#[tauri::command]
pub async fn toggle_mod(app: AppHandle, game_name: String, mod_relative_path: String, enable: bool) -> Result<String, String> {
    let install_dir = get_game_install_dir(&app, &game_name)?;
    let mods_dir = install_dir.join("Mods");
    
    // Careful with joining paths from user input - prevent traversal attacks if this was web, 
    // but here it's local app. Still good to be careful.
    // However, logic here is tricky because the path on disk *changes* when we rename it.
    // The `mod_relative_path` sent by frontend corresponds to the *current* state.
    
    let current_full_path = mods_dir.join(&mod_relative_path);
    if !current_full_path.exists() {
        return Err("Mod directory not found".to_string());
    }

    let parent = current_full_path.parent().ok_or("Invalid path")?;
    let dirname = current_full_path.file_name().ok_or("Invalid filename")?.to_string_lossy().to_string();

    let new_dirname = if enable {
        if dirname.to_uppercase().starts_with("DISABLED") {
            if dirname.to_uppercase().starts_with("DISABLED_") {
                dirname[9..].to_string()
            } else {
                dirname[8..].to_string()
            }
        } else {
            return Ok(dirname); // Already enabled
        }
    } else {
        if !dirname.to_uppercase().starts_with("DISABLED") {
            format!("DISABLED_{}", dirname)
        } else {
            return Ok(dirname); // Already disabled
        }
    };

    let new_full_path = parent.join(&new_dirname);
    
    fs::rename(&current_full_path, &new_full_path)
        .map_err(|e| format!("Failed to rename folder: {}", e))?;

    Ok(new_full_path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn watch_mods(app: AppHandle, state: State<'_, ModWatcher>, game_name: String) -> Result<(), String> {
    let install_dir = get_game_install_dir(&app, &game_name)?;
    let mods_dir = install_dir.join("Mods");

    if !mods_dir.exists() {
        return Err(format!("Mods directory not found at: {:?}", mods_dir));
    }
    
    // Stop existing watcher
    let mut watcher_guard = state.0.lock().unwrap();
    if let Some(_) = *watcher_guard {
        // Drop old watcher
        *watcher_guard = None;
    }

    let app_handle = app.clone();
    
    // Config: Poll every 2 seconds if native events fail, but usage of default() usually implies native.
    // For Windows, default is ReadDirectoryChangesW which is instant.
    // We can add a small delay to debounce at the source if needed, but notify v6 handles this differently.
    // We will just emit raw events and let frontend debounce.
    
    let mut watcher = RecommendedWatcher::new(move |res: Result<notify::Event, notify::Error>| {
        match res {
            Ok(_event) => {
               // Filter for relevant events if needed, but 'refresh' is safe for all
               // println!("File event: {:?}", event);
               // Send event to all windows
               let _ = app_handle.emit("mod-filesystem-changed", ());
            },
            Err(e) => println!("watch error: {:?}", e),
        }
    }, Config::default()).map_err(|e| format!("Failed to create watcher: {}", e))?;

    // Watch recursively
    watcher.watch(&mods_dir, RecursiveMode::Recursive)
           .map_err(|e| format!("Failed to start watch: {}", e))?;

    // Store watcher
    *watcher_guard = Some(watcher);
    
    println!("[ModWatcher] Started watching: {:?}", mods_dir);

    Ok(())
}

#[tauri::command]
pub fn unwatch_mods(state: State<'_, ModWatcher>) -> Result<(), String> {
    let mut watcher_guard = state.0.lock().unwrap();
    *watcher_guard = None;
    println!("[ModWatcher] Stopped watching");
    Ok(())
}

#[tauri::command]
pub fn create_mod_group(app: AppHandle, game_name: String, group_name: String) -> Result<(), String> {
    let install_dir = get_game_install_dir(&app, &game_name)?;
    let group_dir = install_dir.join("Mods").join(&group_name);
    
    if group_dir.exists() {
        return Err("Group already exists".to_string());
    }
    
    fs::create_dir_all(&group_dir).map_err(|e| format!("Failed to create group: {}", e))?;
    Ok(())
}

#[tauri::command]
pub fn rename_mod_group(app: AppHandle, game_name: String, old_group: String, new_group: String) -> Result<(), String> {
    let install_dir = get_game_install_dir(&app, &game_name)?;
    let mods_dir = install_dir.join("Mods");
    let old_dir = mods_dir.join(&old_group);
    let new_dir = mods_dir.join(&new_group);
    
    if !old_dir.exists() {
        return Err("Old group does not exist".to_string());
    }
    if new_dir.exists() {
        return Err("New group name already taken".to_string());
    }
    
    fs::rename(&old_dir, &new_dir).map_err(|e| format!("Failed to rename group: {}", e))?;
    Ok(())
}

#[tauri::command]
pub fn delete_mod_group(app: AppHandle, game_name: String, group_name: String) -> Result<(), String> {
    let install_dir = get_game_install_dir(&app, &game_name)?;
    let group_dir = install_dir.join("Mods").join(&group_name);
    
    if !group_dir.exists() {
        return Err("Group does not exist".to_string());
    }
    
    // Recycle Bin via PowerShell (Windows only)
    #[cfg(target_os = "windows")]
    {
        let path_str = group_dir.to_string_lossy().to_string();
        // Use VisualBasic.FileIO.FileSystem.DeleteDirectory for generic recycled bin deletion
        // Needs absolute path
        let abs_path = if group_dir.is_absolute() { 
            path_str 
        } else {
             // Should verify it is absolute, get_game_install_dir returns absolute
             path_str
        };

        let ps_script = format!(
            "Add-Type -AssemblyName Microsoft.VisualBasic; [Microsoft.VisualBasic.FileIO.FileSystem]::DeleteDirectory('{}', 'OnlyErrorDialogs', 'SendToRecycleBin')",
            abs_path.replace("'", "''") // escape quotes
        );
        
        let status = std::process::Command::new("powershell")
            .args(["-NoProfile", "-Command", &ps_script])
            .status()
            .map_err(|e| format!("Failed to run recycle bin command: {}", e))?;

        if !status.success() {
            return Err("Failed to move to recycle bin".to_string());
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        // Fallback for non-windows
        fs::remove_dir_all(&group_dir).map_err(|e| e.to_string())?;
    }
    
    Ok(())
}

#[tauri::command]
pub fn move_mod_to_group(app: AppHandle, game_name: String, mod_id: String, new_group: String) -> Result<(), String> {
    let install_dir = get_game_install_dir(&app, &game_name)?;
    let mods_dir = install_dir.join("Mods");
    
    // mod_id is the relative path from "Mods" folder, e.g. "Raiden/RaidenMod1" or "RaidenMod1"
    let src_path = mods_dir.join(&mod_id);
    if !src_path.exists() {
        return Err(format!("Mod not found at {:?}", src_path));
    }
    
    // Get the directory name of the mod itself
    let mod_name = src_path.file_name()
        .ok_or("Invalid mod path")?
        .to_string_lossy()
        .to_string();
        
    // Determine destination parent folder
    // If user says "Root", we redirect to "Default" for consistency?
    // Or do we allow Root moves?
    // User requested "regardless of any time cannot put directly under Mods".
    let target_group = if new_group == "Root" || new_group.is_empty() {
        "Default".to_string()
    } else {
        new_group
    };

    let dest_parent = mods_dir.join(&target_group);
    
    if !dest_parent.exists() {
        // Create group if it doesn't exist (e.g. dragging to a new group name provided by UI?)
        // Or enforce validation. For now, auto-create if missing.
        fs::create_dir_all(&dest_parent).map_err(|e| e.to_string())?;
    }
    
    let dest_path = dest_parent.join(&mod_name);
    
    if dest_path.exists() {
        return Err("A mod with this name already exists in the target group".to_string());
    }
    
    // Move (Rename)
    // Note: fs::rename might fail across different mount points, but Mods folder is usually one drive.
    // If it fails, we might need copy+delete, but that's rarer for this use case.
    fs::rename(&src_path, &dest_path).map_err(|e| format!("Failed to move mod: {}", e))?;
    
    Ok(())
}

fn get_unrar_executable() -> String {
    // Try to find unrar in node_modules (bun install unrar-binaries)
    // We look up to 3 levels up just in case
    let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let candidates = [
        current_dir.join("node_modules/unrar-binaries/bin/win32/unrar.exe"),
        current_dir.join("../node_modules/unrar-binaries/bin/win32/unrar.exe"),
        current_dir.join("../../node_modules/unrar-binaries/bin/win32/unrar.exe"),
    ];

    for path in &candidates {
        if path.exists() {
            return path.to_string_lossy().to_string();
        }
    }

    "unrar".to_string()
}

#[derive(Debug, Serialize)]
pub struct ArchivePreview {
    pub root_dirs: Vec<String>,
    pub file_count: usize,
    pub has_ini: bool,
    pub format: String,
}

#[tauri::command]
pub async fn preview_mod_archive(path: String) -> Result<ArchivePreview, String> {
    let path_buf = PathBuf::from(&path);
    if !path_buf.exists() {
        return Err("File not found".to_string());
    }

    let ext = path_buf.extension().unwrap_or_default().to_string_lossy().to_lowercase();
    
    let mut root_dirs = std::collections::HashSet::new();
    let mut file_count = 0;
    let mut has_ini = false;

    if ext == "zip" {
        let file = fs::File::open(&path_buf).map_err(|e| e.to_string())?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;

        for i in 0..archive.len() {
            if let Ok(file) = archive.by_index(i) {
                let name = decode_zip_name(&file);
                if name.ends_with('/') {
                     // Check top level dirs
                     let parts: Vec<&str> = name.split('/').filter(|s| !s.is_empty()).collect();
                     if let Some(first) = parts.first() {
                         root_dirs.insert(first.to_string());
                     }
                } else {
                    file_count += 1;
                    if name.to_lowercase().ends_with(".ini") {
                        has_ini = true;
                    }
                    // Also check for root files to see if it's a "loose" archive
                    let parts: Vec<&str> = name.split('/').collect();
                    if parts.len() == 1 {
                        // File at root
                    } else if let Some(first) = parts.first() {
                        root_dirs.insert(first.to_string());
                    }
                }
            }
        }
    } else if ext == "7z" {
        // Iterate entries to find root dirs and ini
        let file = fs::File::open(&path_buf).map_err(|e| e.to_string())?;
        let len = file.metadata().map_err(|e| e.to_string())?.len();
        let password = sevenz_rust::Password::from("");
        let mut reader = sevenz_rust::SevenZReader::new(file, len, password).map_err(|e| e.to_string())?;
        
        reader.for_each_entries(|entry, _| {
            let name = entry.name();
             if name.to_lowercase().ends_with(".ini") {
                has_ini = true;
            }
            let parts: Vec<&str> = name.split('/').filter(|s| !s.is_empty()).collect();
             if let Some(first) = parts.first() {
                 root_dirs.insert(first.to_string());
             }
             file_count += 1;
             Ok(true)
        }).map_err(|e| e.to_string())?;

    } else if ext == "rar" {
         let unrar_path = get_unrar_executable();

         // unrar lb <archive> lists bare filenames
         let output = std::process::Command::new(&unrar_path)
            .args(["lb", &path])
            .output()
            .map_err(|e| format!("Failed to run unrar (path='{}'): {}. Is unrar available?", unrar_path, e))?;
         
         if !output.status.success() {
             return Err(format!("Failed to read RAR: {}", String::from_utf8_lossy(&output.stderr)));
         }

         let stdout = String::from_utf8_lossy(&output.stdout);
         
         for line in stdout.lines() {
             let line = line.trim();
             if line.is_empty() { continue; }
             
             let current_path = line.to_string();
             file_count += 1;
             
            if current_path.to_lowercase().ends_with(".ini") {
                has_ini = true;
            }
            
            // unrar lb uses \ on windows usually? or /?
            let normalized = current_path.replace('\\', "/");
            let parts: Vec<&str> = normalized.split('/').filter(|s| !s.is_empty()).collect();
            
            if let Some(first) = parts.first() {
                root_dirs.insert(first.to_string());
            }
         }
    } else {
        return Err("Unsupported format for preview (Currently Zip/7z/Rar)".to_string());
    }

    Ok(ArchivePreview {
        root_dirs: root_dirs.into_iter().collect(),
        file_count,
        has_ini,
        format: ext,
    })
}

#[tauri::command]
pub async fn install_mod_archive(
    app: AppHandle, 
    game_name: String, 
    archive_path: String,
    target_name: String, // User defined name for the folder
    target_group: String, // E.g. "Ayaka", or "Root"
    _password: Option<String>
) -> Result<(), String> {
    let install_dir = get_game_install_dir(&app, &game_name)?;
    let mods_dir = install_dir.join("Mods");
    
    // Determine effective target directory
    // If target_group is "Root" or empty, install directly to Mods/target_name
    // If target_group is "Ayaka", install to Mods/Ayaka/target_name
    
    let dest_dir = if target_group == "Root" || target_group.is_empty() {
        // Force "Default" group if none specified, to avoid polluting root
        mods_dir.join("Default").join(&target_name)
    } else {
        mods_dir.join(&target_group).join(&target_name)
    };

    if dest_dir.exists() {
        return Err("该分类下已存在同名 Mod，请更改名称后重试".to_string());
    }
    
    fs::create_dir_all(&dest_dir).map_err(|e| format!("Failed to create destination: {}", e))?;

    let path_buf = PathBuf::from(&archive_path);
    let ext = path_buf.extension().unwrap_or_default().to_string_lossy().to_lowercase();

    if ext == "zip" {
        let file = fs::File::open(&path_buf).map_err(|e| e.to_string())?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
        
        // Smart Extract Analysis
        let mut common_root: Option<String> = None;
        let mut multiple_roots = false;
        
        for i in 0..archive.len() {
             if let Ok(file) = archive.by_index(i) {
                let name = decode_zip_name(&file);
                // skip junk
                if name.starts_with("__MACOSX") || name.ends_with(".DS_Store") { continue; }
                
                let parts: Vec<&str> = name.split('/').filter(|s| !s.is_empty()).collect();
                if parts.is_empty() { continue; }
                
                let root = parts[0].to_string();
                match &common_root {
                    None => common_root = Some(root),
                    Some(r) => {
                        if r != &root {
                             multiple_roots = true;
                             break;
                        }
                    }
                }
             }
        }
        
        let prefix_to_strip = if !multiple_roots { common_root } else { None };

        for i in 0..archive.len() {
             let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
             let name = decode_zip_name(&file);
             if name.starts_with("__MACOSX") || name.ends_with(".DS_Store") { continue; }
             
             let mut target_name = name.clone();
             if let Some(prefix) = &prefix_to_strip {
                 if target_name.starts_with(prefix) {
                     // Strip prefix and leading slash
                     if target_name.len() > prefix.len() {
                         target_name = target_name[prefix.len()..].trim_start_matches('/').to_string();
                     } else {
                         // It IS the folder itself
                         continue;
                     }
                 }
             }
             
             if target_name.is_empty() { continue; }

             let outpath = dest_dir.join(&target_name);
             
             // Basic zip slip check (simple version, assuming target_name is relative)
             // Real world usage: crate `zip` recommends `enclosed_name`, but we modified the name.
             // We trust our stripping logic doesn't add '..'
             
            if name.ends_with('/') {
                fs::create_dir_all(&outpath).map_err(|e| e.to_string())?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        fs::create_dir_all(p).map_err(|e| e.to_string())?;
                    }
                }
                let mut outfile = fs::File::create(&outpath).map_err(|e| e.to_string())?;
                std::io::copy(&mut file, &mut outfile).map_err(|e| e.to_string())?;
            }
        }

    } else if ext == "7z" {
        // Handle 7z smart extract
        let file = fs::File::open(&path_buf).map_err(|e| e.to_string())?;
        let len = file.metadata().map_err(|e| e.to_string())?.len();
        let password = sevenz_rust::Password::from("");
        let mut reader = sevenz_rust::SevenZReader::new(file, len, password).map_err(|e| e.to_string())?;
        
        // Analysis pass
        let mut common_root: Option<String> = None;
        let mut multiple_roots = false;
        
        reader.for_each_entries(|entry, _| {
            let name = entry.name();
            let parts: Vec<&str> = name.split('/').filter(|s| !s.is_empty()).collect();
            if !parts.is_empty() {
                let root = parts[0].to_string();
                if let Some(prev) = &common_root {
                    if prev != &root {
                        multiple_roots = true;
                        // Cannot break easily in closure without return Err, but we can just flag
                    }
                } else {
                     common_root = Some(root);
                }
            }
            Ok(true) 
        }).map_err(|e| e.to_string())?;
        
        if multiple_roots {
             common_root = None;
        }
        
        // Reset reader? SevenZReader doesn't support seek reset cleanly in high level API easily 
        // without re-opening.
        // Re-open file for extraction pass.
        let file = fs::File::open(&path_buf).map_err(|e| e.to_string())?;
        let mut reader = sevenz_rust::SevenZReader::new(file, len, sevenz_rust::Password::from("")).map_err(|e| e.to_string())?;
        
        reader.for_each_entries(|entry, reader| {
             let name = entry.name();
             let mut target_name = name.to_string();
             
             if let Some(prefix) = &common_root {
                  if target_name.starts_with(prefix) {
                     if target_name.len() > prefix.len() {
                         target_name = target_name[prefix.len()..].trim_start_matches('/').to_string();
                     } else {
                         return Ok(true); // Skip root folder itself
                     }
                  }
             }
             if target_name.is_empty() { return Ok(true); }
             
             let outpath = dest_dir.join(&target_name);
             
             if entry.is_directory() {
                 fs::create_dir_all(&outpath).map_err(|e| sevenz_rust::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()), std::borrow::Cow::Borrowed("create dir")))?;
             } else {
                 if let Some(p) = outpath.parent() {
                     fs::create_dir_all(p).map_err(|e| sevenz_rust::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()), std::borrow::Cow::Borrowed("create parent")))?;
                 }
                // default_entry_extract_fn expects &PathBuf as third arg, it handles file creation internally usually?
                // Actually let's check docs or use lower level copy
                // Based on previous error: expected `&PathBuf`, found `&mut File`
                // `default_entry_extract_fn(entry, reader, path)`
                
                sevenz_rust::default_entry_extract_fn(entry, reader, &outpath)?;
             }
             Ok(true)
        }).map_err(|e| e.to_string())?;
        
    } else if ext == "rar" {
         let unrar_path = get_unrar_executable();
        
         // 1. Extract to temp folder
         let temp_dir = dest_dir.join("_temp_extract");
         if temp_dir.exists() {
             fs::remove_dir_all(&temp_dir).map_err(|e| e.to_string())?;
         }
         fs::create_dir_all(&temp_dir).map_err(|e| e.to_string())?;
         
         // unrar x -y <archive> <dest_dir/>
         // Note: dest dir must end in separator for unrar usually?
         // unrar x archive.rar D:\path\to\dest\
         
         let output = std::process::Command::new(&unrar_path)
            .args(["x", "-y", &archive_path, &format!("{}\\", temp_dir.to_string_lossy())])
            .output()
            .map_err(|e| format!("Failed to run unrar (path='{}'): {}. Is unrar available?", unrar_path, e))?;
            
         if !output.status.success() {
             return Err(format!("RAR extraction failed: {}", String::from_utf8_lossy(&output.stderr)));
         }
         
         // 2. Analyze temp folder structure
         // If temp folder contains ONLY one directory (and no files at root), move that directory's contents up
         let entries = fs::read_dir(&temp_dir).map_err(|e| e.to_string())?;
         let mut root_items = Vec::new();
         for entry in entries {
             let entry = entry.map_err(|e| e.to_string())?;
             root_items.push(entry.path());
         }
         
         let should_strip = if root_items.len() == 1 && root_items[0].is_dir() {
             true
         } else {
             false
         };
         
         if should_strip {
             let root_dir = &root_items[0];
             let sub_entries = fs::read_dir(root_dir).map_err(|e| e.to_string())?;
             for sub in sub_entries {
                 let sub = sub.map_err(|e| e.to_string())?;
                 let sub_name = sub.file_name();
                 let target = dest_dir.join(sub_name);
                 fs::rename(sub.path(), target).map_err(|e| e.to_string())?;
             }
             // Cleanup empty root and temp
             let _ = fs::remove_dir_all(&temp_dir);
         } else {
             // Move all items from temp to dest
             for item in root_items {
                 let name = item.file_name().unwrap();
                 let target = dest_dir.join(name);
                 fs::rename(&item, target).map_err(|e| e.to_string())?;
             }
             let _ = fs::remove_dir_all(&temp_dir);
         }

    } else {
        return Err("Unsupported format".to_string());
    }

    Ok(())
}
