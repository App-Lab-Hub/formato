use crate::{AppState, db};
use tauri::Manager;

use std::fs;
use crate::paths::temp_dir;
use std::time::{SystemTime, UNIX_EPOCH};


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

// src/commands.rs или где у тебя команды

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
        "format_type": format.format_type, // 👈 ДОБАВЛЯЕМ
    }))
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
                "format_type": f.format_type, // 👈 ДОБАВЛЯЕМ
            })
        })
        .collect())
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
    let dir = temp_dir();
    
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    
    let file_name = format!("{}_{}.{}", name, timestamp, extension);
    let file_path = dir.join(file_name);
    
    fs::write(&file_path, content).map_err(|e| e.to_string())?;
    
    Ok(file_path.to_string_lossy().to_string())
}





// src/utils/mod.rs

use serde::{Deserialize, Serialize};

// ============================================================
// ТИПЫ
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentType {
    Text,
    Image,
    Audio,
    Video,
    Document,
}

impl From<String> for ContentType {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "text" => ContentType::Text,
            "image" => ContentType::Image,
            "audio" => ContentType::Audio,
            "video" => ContentType::Video,
            "document" => ContentType::Document,
            _ => ContentType::Text,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct AvailabilityResponse {
    pub text: String,
    pub image: String,
    pub audio: String,
    pub video: String,
    pub document: String, // 👈 ДОБАВЛЯЕМ
}

// ============================================================
// ЛОГИКА ДОСТУПНОСТИ
// ============================================================

pub fn get_availability_from_type(from_type: &str) -> AvailabilityResponse {
    let from: ContentType = from_type.to_string().into();
    
    match from {
        ContentType::Text => AvailabilityResponse {
            text: "available".to_string(), //ready
            image: "not_available".to_string(), //ready(y)
            audio: "available".to_string(), //ready(y)
            video: "not_available".to_string(), //ready(y)
            document: "available".to_string(), 
        },
        ContentType::Image => AvailabilityResponse {
            text: "available".to_string(), //ready
            image: "available".to_string(),//ready
            audio: "not_available".to_string(), //ready
            video: "not_available".to_string(), //ready
            document: "available".to_string(),
        },
        ContentType::Audio => AvailabilityResponse {
            text: "available".to_string(), //ready
            image: "not_available".to_string(), //ready
            audio: "available".to_string(),  //ready
            video: "not_available".to_string(),  //ready
            document: "available".to_string(),
        },
        ContentType::Video => AvailabilityResponse {
            text: "available".to_string(),
            image: "not_available".to_string(), //ready
            audio: "available".to_string(), //ready
            video: "available".to_string(), //ready
            document: "not_available".to_string(),
        },
        ContentType::Document => AvailabilityResponse {
            text: "available".to_string(),      //ready
            image: "not_available".to_string(), //ready
            audio: "available".to_string(), //ready
            video: "not_available".to_string(), //ready
            document: "available".to_string(),  //ready
        },
    }
}

// ============================================================
// TAURI COMMAND
// ============================================================

#[tauri::command]
pub fn get_availability(from_type: String) -> AvailabilityResponse {
    get_availability_from_type(&from_type)
}





use ffmpeg_sidecar::download::auto_download;

pub fn init_ffmpeg() -> Result<(), String> {
    // auto_download сам проверит, есть ли FFmpeg
    // Если есть - ничего не сделает
    // Если нет - скачает
    auto_download()
        .map_err(|e| format!("Failed to download FFmpeg: {}", e))
}