use crate::{AppState, db};
use tauri::Manager;

use std::fs;
use std::path::PathBuf;
use std::env::temp_dir;

/// Показывает главное окно (вызывается после загрузки фронтенда)
#[tauri::command]
pub fn app_ready(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {                         
        let _ = window.set_background_color(Some(tauri::utils::config::Color(6, 6, 8, 255)));
        // let _ = window.set_theme(Some(tauri::Theme::Dark));
        let _ = window.show();
    }
}

/// Устанавливает цвет фона окна
#[tauri::command]
pub fn set_window_background(app: tauri::AppHandle, r: u8, g: u8, b: u8, a: u8) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.set_background_color(Some(tauri::utils::config::Color(r, g, b, a)));
    }
}

/// Проверяет состояние БД
#[tauri::command]
pub async fn get_db_status(state: tauri::State<'_, AppState>) -> Result<String, String> {
    let db_guard = state.db.lock().await;
    match db_guard.as_ref() {
        Some(db) => {
            db.ping().await.map_err(|e| e.to_string())?;
            Ok("✅ Database is ready".to_string())
        }
        None => Err("❌ Database not initialized".to_string()),
    }
}

#[tauri::command]
pub async fn get_formats(state: tauri::State<'_, AppState>) -> Result<Vec<serde_json::Value>, String> {
    let db_guard = state.db.lock().await;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;
    
    let formats = db::get_all_formats(db)
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(formats
        .into_iter()
        .map(|f| {
            serde_json::json!({
                "format_id": f.format_id,
                "name": f.name,
                "extensions": f.extensions,
                "icon": f.icon,
                "color": f.color,
                "glow": f.glow,
                "text_color": f.text_color,
                "border_hover": f.border_hover,
            })
        })
        .collect())
}

#[tauri::command]
pub async fn get_format_by_id(
    state: tauri::State<'_, AppState>,
    format_id: String,
) -> Result<serde_json::Value, String> {
    let db_guard = state.db.lock().await;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;
    
    let format = db::get_format_by_id(db, &format_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Format not found")?;
    
    Ok(serde_json::json!({
        "format_id": format.format_id,
        "name": format.name,
        "extensions": format.extensions,
        "icon": format.icon,
        "color": format.color,
        "glow": format.glow,
        "text_color": format.text_color,
        "border_hover": format.border_hover,
    }))
}


#[tauri::command]
pub async fn get_file_size(path: String) -> Result<u64, String> {
    tokio::fs::metadata(&path)
        .await
        .map(|m| m.len())
        .map_err(|e| format!("Cannot get file size: {e}"))
}



#[tauri::command]
pub fn create_temp_file(content: String, extension: String, name: String) -> Result<String, String> {

    
    let temp_dir = temp_dir().join("formato_temp");
    if !temp_dir.exists() {
        fs::create_dir_all(&temp_dir).map_err(|e| e.to_string())?;
    }
    
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    
    let file_name = format!("{}_{}.{}", name, timestamp, extension);
    let file_path = temp_dir.join(file_name);
    
    fs::write(&file_path, content).map_err(|e| e.to_string())?;
    
    Ok(file_path.to_string_lossy().to_string())
}