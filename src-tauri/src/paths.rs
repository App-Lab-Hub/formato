// src-tauri/src/paths.rs

use std::path::PathBuf;
// use dirs;

const APP_NAME: &str = "formato";

/// Возвращает корневую директорию приложения: ~/.local/share/formato/
fn app_root() -> PathBuf {
    let data_dir = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    let root = data_dir.join(APP_NAME);
    
    if !root.exists() {
        std::fs::create_dir_all(&root).expect("Failed to create app directory");
    }
    
    root
}

/// Возвращает путь к БД: ~/.local/share/formato/converter.db
pub fn db_path() -> PathBuf {
    app_root().join("converter.db")
}

/// Возвращает путь к директории с конвертированными файлами
pub fn converted_dir() -> PathBuf {
    let dir = app_root().join("converted");
    if !dir.exists() {
        std::fs::create_dir_all(&dir).expect("Failed to create converted directory");
    }
    dir
}