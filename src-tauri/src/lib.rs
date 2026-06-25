// src-tauri/src/lib.rs

mod convert;
mod html_convert;
mod macros;
mod db;
mod paths;

use sea_orm::DatabaseConnection;
use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::Manager;

// ============================================================
// STATE
// ============================================================

/// Глобальное состояние приложения
#[derive(Default)]  // ← автоматически реализует Default
pub struct AppState {
    pub db: Arc<Mutex<Option<DatabaseConnection>>>,
}

// ============================================================
// COMMANDS
// ============================================================

/// Показывает главное окно (вызывается после загрузки фронтенда)
#[tauri::command]
fn app_ready(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {                         
        let _ = window.set_background_color(Some(tauri::utils::config::Color(6, 6, 8, 255)));
        let _ = window.set_theme(Some(tauri::Theme::Dark));
        let _ = window.show();
    }
}

/// Устанавливает цвет фона окна
#[tauri::command]
fn set_window_background(app: tauri::AppHandle, r: u8, g: u8, b: u8, a: u8) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.set_background_color(Some(tauri::utils::config::Color(r, g, b, a)));
    }
}

/// Проверяет состояние БД
#[tauri::command]
async fn get_db_status(state: tauri::State<'_, AppState>) -> Result<String, String> {
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
async fn get_formats(state: tauri::State<'_, AppState>) -> Result<Vec<serde_json::Value>, String> {
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
                "description": f.description,
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
async fn get_format_by_id(
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
        "description": format.description,
        "icon": format.icon,
        "color": format.color,
        "glow": format.glow,
        "text_color": format.text_color,
        "border_hover": format.border_hover,
    }))
}










// ============================================================
// RUN
// ============================================================

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // Plugins
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        
        // State
        .manage(AppState::default())
        
        // Commands
        .invoke_handler(tauri::generate_handler![
            // Convert
            convert::convert_file,
            convert::read_file_content,
            convert::open_file,
            
            // Window
            app_ready,
            set_window_background,
            
            // Database
            get_db_status,
            get_formats,
            get_format_by_id
        ])
        
        // Setup
        .setup(|app| {
            // ✅ Клонируем app_handle ДО spawn
            let app_handle = app.handle().clone();
            
            // Инициализируем БД в фоне при старте
            tauri::async_runtime::spawn(async move {
                match db::db_init().await {
                    Ok(db) => {
                        let state = app_handle.state::<AppState>();
                        let mut db_guard = state.db.lock().await;
                        *db_guard = Some(db);
                        println!("✅ Database initialized successfully");
                    }
                    Err(e) => {
                        eprintln!("❌ Database initialization failed: {}", e);
                    }
                }
            });
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}