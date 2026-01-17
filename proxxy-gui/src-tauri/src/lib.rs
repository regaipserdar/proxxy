use std::fs;
use std::io::Write;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// Opens response content in the default browser by writing to a temp file
#[tauri::command]
fn open_response_in_browser(content: String, content_type: String) -> Result<String, String> {
    // Determine file extension based on content type
    let extension = if content_type.contains("html") {
        "html"
    } else if content_type.contains("json") {
        "json"
    } else if content_type.contains("xml") {
        "xml"
    } else {
        "txt"
    };
    
    // Create temp file path
    let temp_dir = std::env::temp_dir();
    let file_name = format!("proxxy_response_{}.{}", 
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis(),
        extension
    );
    let file_path = temp_dir.join(&file_name);
    
    // Write content to file
    let mut file = fs::File::create(&file_path)
        .map_err(|e| format!("Failed to create temp file: {}", e))?;
    file.write_all(content.as_bytes())
        .map_err(|e| format!("Failed to write to temp file: {}", e))?;
    
    // Open with default application
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&file_path)
            .spawn()
            .map_err(|e| format!("Failed to open file: {}", e))?;
    }
    
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", file_path.to_str().unwrap_or_default()])
            .spawn()
            .map_err(|e| format!("Failed to open file: {}", e))?;
    }
    
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&file_path)
            .spawn()
            .map_err(|e| format!("Failed to open file: {}", e))?;
    }
    
    Ok(file_path.to_string_lossy().to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![greet, open_response_in_browser])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

