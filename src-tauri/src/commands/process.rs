use tauri::{AppHandle, Manager, Runtime};
use std::process::Command;
use std::os::windows::process::CommandExt; // 仅限 Windows 平台，用于隐藏控制台窗口

const CREATE_NO_WINDOW: u32 = 0x08000000;

#[tauri::command]
pub fn run_resource_executable<R: Runtime>(
    app: AppHandle<R>,
    filename: String, // 例如 "ffmpeg.exe", 不带路径
    args: Vec<String>
) -> Result<String, String> {
    // 1. 获取资源目录的绝对路径
    let resource_dir = app.path().resource_dir()
        .map_err(|e| format!("无法获取资源目录: {}", e))?;

    // 2. 拼接 exe 的完整路径
    // 注意：假设你的 tauri.conf.json 配置是 "resources": ["resources/*"]
    // 那么构建后的结构通常是 resource_dir/resources/filename
    // 如果运行时找不到，请检查实际打包后的结构
    let exe_path = resource_dir.join("resources").join(&filename);

    if !exe_path.exists() {
        return Err(format!("找不到可执行文件: {:?}", exe_path));
    }

    // 3. 执行命令
    let output = Command::new(&exe_path)
        .args(args)
        .creation_flags(CREATE_NO_WINDOW) // 默默执行，不跳出黑框
        .output()
        .map_err(|e| format!("执行失败: {}", e))?;

    // 4. 处理输出
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(stdout)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(stderr)
    }
}
