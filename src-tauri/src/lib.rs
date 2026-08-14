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
mod models;

use sea_orm::DatabaseConnection;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Глобальное состояние приложения
#[derive(Default)] 
pub struct AppState {
    pub db: Arc<Mutex<Option<DatabaseConnection>>>,
    pub system_theme: Mutex<String>, 
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

    // Инициализация FFmpeg в отдельном потоке (не блокируем запуск)
    std::thread::spawn(|| {
        match utils::init_ffmpeg() {
            Ok(_) => println!("✅ [Rust] FFmpeg initialized successfully!"),
            Err(e) => eprintln!("⚠️ [Rust] FFmpeg init failed: {}", e),
        }
    });

    tauri::Builder::default()
        // Plugins
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        
        // State
        .manage(AppState {
            db: Arc::new(Mutex::new(db_conn)),
            system_theme: Mutex::new("dark".to_string()),
        })
        
        // Commands
        .invoke_handler(tauri::generate_handler![
            // Convert
            convert::convert_file,
            convert::read_file_content,
            convert::read_file_bytes,
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
            // models
            models::get_models_status,
            models::download_synthesis_model,
            models::download_recognition_model,
            
        ])
        
        // Setup
        .setup(|_app| {
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
