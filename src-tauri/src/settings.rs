// src-tauri/src/settings.rs
use serde::{Deserialize, Serialize};
use crate::paths::config_dir;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppSettings {
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default)]
    pub auto_preview: bool,
    #[serde(default = "default_max_preview_size")]
    pub max_preview_size: f64,
    #[serde(default = "default_true")]
    pub show_extensions: bool,
    #[serde(default = "default_true")]
    pub enable_cache: bool,
    #[serde(default)]
    pub enable_archive: bool,
    #[serde(default = "default_archive_format")]
    pub archive_format: String,
    
    // 🆕 Модели для синтеза и распознавания речи
    #[serde(default = "default_synthesis_model")]
    pub synthesis_model: HashMap<String, String>, // { "ru": "ru_RU-dmitri-medium", "en": "en_US-lessac-medium" }
    #[serde(default = "default_recognition_model")]
    pub recognition_model: String, // Модель для распознавания речи
}

// 🆕 Модели для синтеза речи (словарь)
fn default_synthesis_model() -> HashMap<String, String> {
    let mut map = HashMap::new();
    map.insert("ru".to_string(), "ru_RU-dmitri-medium".to_string());
    map.insert("en".to_string(), "en_US-lessac-medium".to_string());
    map
}



fn default_theme() -> String { "system".into() }
fn default_language() -> String { "ru".into() }
fn default_archive_format() -> String { "zip".into() }
fn default_true() -> bool { true }
fn default_max_preview_size() -> f64 { 1.0 }

fn default_recognition_model() -> String { 
    "ggml-tiny-q5_1.bin".into() 
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            language: default_language(),
            auto_preview: false,
            max_preview_size: 1.0,
            show_extensions: true,
            enable_cache: true,
            enable_archive: false,
            archive_format: default_archive_format(),
            synthesis_model: default_synthesis_model(),
            recognition_model: default_recognition_model(),
        }
    }
}

fn settings_path() -> std::path::PathBuf {
    config_dir().join("settings.yaml")
}

#[tauri::command]
pub async fn get_settings() -> AppSettings {
    tokio::fs::read_to_string(settings_path())
        .await
        .ok()
        .and_then(|s| serde_yaml::from_str(&s).ok())
        .unwrap_or_default()
}

#[tauri::command]
pub async fn save_settings(settings: AppSettings) -> Result<(), String> {
    let yaml = serde_yaml::to_string(&settings).map_err(|e| e.to_string())?;
    tokio::fs::write(settings_path(), yaml)
        .await
        .map_err(|e| e.to_string())
}