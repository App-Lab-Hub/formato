// src-tauri/src/lib.rs

mod convert;
mod html_convert;
mod macros;
mod db;
mod paths;
mod utils;

use sea_orm::DatabaseConnection;
use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::Manager;

/// Глобальное состояние приложения
#[derive(Default)] 
pub struct AppState {
    pub db: Arc<Mutex<Option<DatabaseConnection>>>,
}

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
            convert::hash_file,
            
            // Window
            utils::app_ready,
            utils::set_window_background,
            
            // Database
            utils::get_db_status,
            utils::get_formats,
            utils::get_format_by_id,
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