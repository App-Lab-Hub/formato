// src-tauri/src/settings.rs
use serde::{Deserialize, Serialize};
use crate::paths::config_dir;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppSettings {
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default)]
    pub auto_preview: bool,
   #[serde(default = "default_max_preview_size")]  // 👈 меняем
    pub max_preview_size: f64,  // 👈 меняем на f64
    #[serde(default = "default_after_convert")]
    pub after_convert: String,
    #[serde(default = "default_true")]
    pub show_extensions: bool,
    #[serde(default = "default_true")]
    pub enable_cache: bool,
    #[serde(default)]
    pub enable_archive: bool,
    #[serde(default = "default_archive_format")]
    pub archive_format: String,
}

fn default_theme() -> String { "system".into() }
fn default_language() -> String { "ru".into() }
fn default_after_convert() -> String { "stay".into() }
fn default_archive_format() -> String { "zip".into() }
fn default_true() -> bool { true }
fn default_max_preview_size() -> f64 { 1.0 }  // 👈 добавляем

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            language: default_language(),
            auto_preview: false,
            max_preview_size: 1.0,  // 👈 1.0 вместо 0
            after_convert: default_after_convert(),
            show_extensions: true,
            enable_cache: true,
            enable_archive: false,
            archive_format: default_archive_format(),
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