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

#[tauri::command]
async fn repaint_window(window: tauri::Window) -> Result<(), String> {
  
    // Принудительно изменяем размер окна на 1px и возвращаем обратно
    // Это вызывает перерисовку в WebKitGTK
    let current_size = window.inner_size().map_err(|e| e.to_string())?;

    // Изменяем размер на +1px
    let _ = window.set_size(
        tauri::LogicalSize::new(
            current_size.width as f64 + 1.0,
            current_size.height as f64
        )
    );

    // Возвращаем обратно через небольшой промежуток времени
    std::thread::sleep(std::time::Duration::from_millis(10));

    let _ = window.set_size(
        tauri::LogicalSize::new(
            current_size.width as f64,
            current_size.height as f64
        )
    );

    println!("✅ Window repaint triggered from command");
    Ok(())
    

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
            utils::get_file_size,
            db::reset_database,
            
            utils::create_temp_file,
            // settings
            settings::get_settings,
            settings::save_settings,
            // archive
            archive::archive_file,
            archive::archive_multiple_files,
            // files
            files::get_files,
            files::delete_file,
            repaint_window

        ])
        
        // Setup
        .setup(|app| {
            let app_handle = app.handle().clone();
            
            tauri::async_runtime::block_on(async {
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