use std::path::PathBuf;

const APP_NAME: &str = "formato";

// Постоянные данные (БД, конфиги)
pub fn app_root() -> PathBuf {
    let data_dir = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    let root = data_dir.join(APP_NAME);
    
    if !root.exists() {
        std::fs::create_dir_all(&root).expect("Failed to create app directory");
    }
    
    root
}

pub fn db_path() -> PathBuf {
    app_root().join("converter.db")
}

pub fn converted_dir() -> PathBuf {
    let dir = app_root().join("converted");
    if !dir.exists() {
        std::fs::create_dir_all(&dir).expect("Failed to create converted directory");
    }
    dir
}

pub fn config_dir() -> PathBuf {
    let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    let dir = config_dir.join(APP_NAME);
    if !dir.exists() {
        std::fs::create_dir_all(&dir).expect("Failed to create config directory");
    }
    dir
}

// Временные файлы — используем cache_dir
pub fn temp_dir() -> PathBuf {
    let cache_dir = dirs::cache_dir().unwrap_or_else(|| PathBuf::from("."));
    let dir = cache_dir.join(APP_NAME);
    if !dir.exists() {
        std::fs::create_dir_all(&dir).expect("Failed to create temp directory");
    }
    dir
}

// Папка для всех моделей
pub fn models_dir() -> PathBuf {
    let dir = app_root().join("models");
    if !dir.exists() {
        std::fs::create_dir_all(&dir).expect("Failed to create models directory");
    }
    dir
}

// Модели Whisper
pub fn whisper_models_dir() -> PathBuf {
    let dir = models_dir().join("whisper");
    if !dir.exists() {
        std::fs::create_dir_all(&dir).expect("Failed to create whisper models directory");
    }
    dir
}

// Модели Piper
pub fn piper_models_dir() -> PathBuf {
    let dir = models_dir().join("piper");
    if !dir.exists() {
        std::fs::create_dir_all(&dir).expect("Failed to create piper models directory");
    }
    dir
}