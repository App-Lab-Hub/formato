// src-tauri/src/lib.rs

mod convert;
mod html_convert;
mod macros;
mod db;
mod paths;
mod utils;
mod settings;
mod archive;
mod files;

use sea_orm::DatabaseConnection;
use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::Manager;

/// Глобальное состояние приложения
#[derive(Default)] 
pub struct AppState {
    pub db: Arc<Mutex<Option<DatabaseConnection>>>,
    pub system_theme: Mutex<String>, // 'dark' или 'light'
}




#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let db_conn = tauri::async_runtime::block_on(async {
        match db::db_init().await {
            Ok(db) => {
                println!("✅ [Rust] Database fully initialized and ready.");
                Some(db)
            }
            Err(e) => {
                eprintln!("❌ [Rust] Critical DB init error: {}", e);
                None
            }
        }
    });

    tauri::Builder::default()
        // Plugins
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        
        // State
        .manage(AppState{
            db:Arc::new(Mutex::new(db_conn)),
            ..Default::default()
        })
        
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
            utils::get_file_size,
            db::reset_database,
            
            utils::create_temp_file,
            utils::get_availability,
            // settings
            settings::get_settings,
            settings::save_settings,
            // archive
            archive::archive_file,
            archive::archive_multiple_files,
            // files
            files::get_files,
            files::delete_file,

        ])
        
        // Setup
        .setup(|_app| {
            // let app_handle = app.handle().clone();
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}