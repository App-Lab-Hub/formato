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
    
    #[serde(default = "default_synthesis_model")]
    pub synthesis_model: HashMap<String, String>,
    #[serde(default = "default_recognition_model")]
    pub recognition_model: String, 
}

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








#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // ============================================================
    // КОНСТАНТЫ ДЛЯ ТЕСТОВ
    // ============================================================

    const VALID_THEMES: [&str; 3] = ["dark", "light", "system"];
    const VALID_LANGUAGES: [&str; 2] = ["ru", "en"];
    const VALID_ARCHIVE_FORMATS: [&str; 3] = ["zip", "tar.gz", "tar.xz"];

    // ============================================================
    // ВСПОМОГАТЕЛЬНЫЕ ФУНКЦИИ
    // ============================================================

    fn is_valid_theme(theme: &str) -> bool {
        VALID_THEMES.contains(&theme)
    }

    fn is_valid_language(lang: &str) -> bool {
        VALID_LANGUAGES.contains(&lang)
    }

    // ============================================================
    // ТЕСТЫ: ВАЛИДНЫЕ ЗНАЧЕНИЯ
    // ============================================================

    #[test]
    fn test_valid_themes() {
        for theme in VALID_THEMES {
            assert!(is_valid_theme(theme));
            println!("✅ Valid theme: {}", theme);
        }
        assert_eq!(VALID_THEMES.len(), 3);
    }

    #[test]
    fn test_valid_languages() {
        for lang in VALID_LANGUAGES {
            assert!(is_valid_language(lang));
            println!("✅ Valid language: {}", lang);
        }
        assert_eq!(VALID_LANGUAGES.len(), 2);
    }

    #[test]
    fn test_invalid_theme() {
        assert!(!is_valid_theme("invalid_theme"));
        assert!(!is_valid_theme(""));
        assert!(!is_valid_theme("DARK"));
        println!("✅ Invalid themes correctly rejected");
    }

    #[test]
    fn test_invalid_language() {
        assert!(!is_valid_language("fr"));
        assert!(!is_valid_language("de"));
        assert!(!is_valid_language(""));
        assert!(!is_valid_language("RU"));
        println!("✅ Invalid languages correctly rejected");
    }

    // ============================================================
    // ТЕСТЫ: ЗНАЧЕНИЯ ПО УМОЛЧАНИЮ
    // ============================================================

    #[test]
    fn test_default_settings() {
        let settings = AppSettings::default();
        
        assert_eq!(settings.theme, "system");
        assert!(is_valid_theme(&settings.theme));
        
        assert_eq!(settings.language, "ru");
        assert!(is_valid_language(&settings.language));
        
        assert!(!settings.auto_preview);
        assert_eq!(settings.max_preview_size, 1.0);
        assert!(settings.show_extensions);
        assert!(settings.enable_cache);
        assert!(!settings.enable_archive);
        assert_eq!(settings.archive_format, "zip");
        assert_eq!(settings.recognition_model, "ggml-tiny-q5_1.bin");
        
        // Проверяем модели синтеза
        assert_eq!(settings.synthesis_model.len(), 2);
        assert!(settings.synthesis_model.contains_key("ru"));
        assert!(settings.synthesis_model.contains_key("en"));
        assert_eq!(settings.synthesis_model.get("ru"), Some(&"ru_RU-dmitri-medium".to_string()));
        assert_eq!(settings.synthesis_model.get("en"), Some(&"en_US-lessac-medium".to_string()));
        
        println!("✅ Default settings: theme={}, language={}", settings.theme, settings.language);
    }

    #[test]
    fn test_default_functions() {
        assert_eq!(default_theme(), "system");
        assert_eq!(default_language(), "ru");
        assert_eq!(default_archive_format(), "zip");
        assert!(default_true());
        assert_eq!(default_max_preview_size(), 1.0);
        assert_eq!(default_recognition_model(), "ggml-tiny-q5_1.bin");
        
        let synthesis = default_synthesis_model();
        assert_eq!(synthesis.len(), 2);
        assert_eq!(synthesis.get("ru"), Some(&"ru_RU-dmitri-medium".to_string()));
        assert_eq!(synthesis.get("en"), Some(&"en_US-lessac-medium".to_string()));
        
        println!("✅ All default functions work correctly");
    }

    // ============================================================
    // ТЕСТЫ: СЕРИАЛИЗАЦИЯ/ДЕСЕРИАЛИЗАЦИЯ
    // ============================================================

    #[test]
    fn test_serialize_settings() {
        let settings = AppSettings::default();
        let yaml = serde_yaml::to_string(&settings).unwrap();
        
        assert!(yaml.contains("theme: system"));
        assert!(yaml.contains("language: ru"));
        assert!(yaml.contains("auto_preview: false"));
        assert!(yaml.contains("enable_cache: true"));
        assert!(yaml.contains("archive_format: zip"));
        assert!(yaml.contains("recognition_model: ggml-tiny-q5_1.bin"));
        assert!(yaml.contains("synthesis_model:"));
        assert!(yaml.contains("ru: ru_RU-dmitri-medium"));
        assert!(yaml.contains("en: en_US-lessac-medium"));
        
        println!("✅ Serialization successful");
    }

    #[test]
    fn test_serialize_settings_with_dark_theme() {
        // 🔥 Исправлено: используем ..Default::default()
        let settings = AppSettings {
            theme: "dark".to_string(),
            language: "en".to_string(),
            ..AppSettings::default()
        };
        
        let yaml = serde_yaml::to_string(&settings).unwrap();
        
        assert!(yaml.contains("theme: dark"));
        assert!(yaml.contains("language: en"));
        assert!(!yaml.contains("theme: system"));
        assert!(!yaml.contains("language: ru"));
        
        println!("✅ Serialization with dark theme successful");
    }

    #[test]
    fn test_serialize_settings_with_light_theme() {
        // 🔥 Исправлено: используем ..Default::default()
        let settings = AppSettings {
            theme: "light".to_string(),
            ..AppSettings::default()
        };
        
        let yaml = serde_yaml::to_string(&settings).unwrap();
        assert!(yaml.contains("theme: light"));
        assert!(!yaml.contains("theme: system"));
        
        println!("✅ Serialization with light theme successful");
    }

    #[test]
    fn test_deserialize_settings() {
        let yaml = r#"
theme: dark
language: en
auto_preview: true
max_preview_size: 2.5
show_extensions: false
enable_cache: false
enable_archive: true
archive_format: tar.gz
recognition_model: ggml-base-q5_1.bin
synthesis_model:
  ru: ru_RU-irina-medium
  en: en_US-amy-medium
"#;
        
        let settings: AppSettings = serde_yaml::from_str(yaml).unwrap();
        
        assert_eq!(settings.theme, "dark");
        assert_eq!(settings.language, "en");
        assert!(settings.auto_preview);
        assert_eq!(settings.max_preview_size, 2.5);
        assert!(!settings.show_extensions);
        assert!(!settings.enable_cache);
        assert!(settings.enable_archive);
        assert_eq!(settings.archive_format, "tar.gz");
        assert_eq!(settings.recognition_model, "ggml-base-q5_1.bin");
        assert_eq!(settings.synthesis_model.get("ru"), Some(&"ru_RU-irina-medium".to_string()));
        assert_eq!(settings.synthesis_model.get("en"), Some(&"en_US-amy-medium".to_string()));
        
        println!("✅ Deserialization successful");
    }

    #[test]
    fn test_deserialize_all_themes() {
        for theme in VALID_THEMES {
            let yaml = format!(r#"
theme: {}
language: ru
"#, theme);
            
            let settings: AppSettings = serde_yaml::from_str(&yaml).unwrap();
            assert_eq!(settings.theme, theme);
            println!("✅ Deserialized theme: {}", theme);
        }
    }

    #[test]
    fn test_deserialize_all_languages() {
        for lang in VALID_LANGUAGES {
            let yaml = format!(r#"
theme: system
language: {}
"#, lang);
            
            let settings: AppSettings = serde_yaml::from_str(&yaml).unwrap();
            assert_eq!(settings.language, lang);
            println!("✅ Deserialized language: {}", lang);
        }
    }

    #[test]
    fn test_deserialize_partial_settings() {
        let yaml = r#"
theme: dark
language: en
"#;
        
        let settings: AppSettings = serde_yaml::from_str(yaml).unwrap();
        
        assert_eq!(settings.theme, "dark");
        assert_eq!(settings.language, "en");
        
        // Проверяем, что остальные поля заполнены значениями по умолчанию
        assert!(!settings.auto_preview);
        assert_eq!(settings.max_preview_size, 1.0);
        assert!(settings.show_extensions);
        assert!(settings.enable_cache);
        assert!(!settings.enable_archive);
        assert_eq!(settings.archive_format, "zip");
        assert_eq!(settings.recognition_model, "ggml-tiny-q5_1.bin");
        assert!(settings.synthesis_model.contains_key("ru"));
        assert!(settings.synthesis_model.contains_key("en"));
        
        println!("✅ Partial deserialization successful");
    }

    // ============================================================
    // ТЕСТЫ: GET_SETTINGS
    // ============================================================

    #[tokio::test]
    async fn test_get_settings_default_when_no_file() {
        let settings = get_settings().await;
        
        assert_eq!(settings.theme, "system");
        assert_eq!(settings.language, "ru");
        assert!(settings.synthesis_model.contains_key("ru"));
        assert!(settings.synthesis_model.contains_key("en"));
        assert_eq!(settings.recognition_model, "ggml-tiny-q5_1.bin");
        
        println!("✅ get_settings returned defaults");
    }

    // ============================================================
    // ТЕСТЫ: SAVE_SETTINGS
    // ============================================================

    #[tokio::test]
    async fn test_save_settings_serialization() {
        let settings = AppSettings {
            theme: "dark".to_string(),
            language: "en".to_string(),
            auto_preview: true,
            max_preview_size: 2.5,
            show_extensions: false,
            enable_cache: false,
            enable_archive: true,
            archive_format: "tar.gz".to_string(),
            synthesis_model: {
                let mut map = HashMap::new();
                map.insert("ru".to_string(), "ru_RU-irina-medium".to_string());
                map.insert("en".to_string(), "en_US-amy-medium".to_string());
                map
            },
            recognition_model: "ggml-base-q5_1.bin".to_string(),
        };
        
        let yaml = serde_yaml::to_string(&settings).unwrap();
        
        assert!(yaml.contains("theme: dark"));
        assert!(yaml.contains("language: en"));
        assert!(yaml.contains("auto_preview: true"));
        assert!(yaml.contains("recognition_model: ggml-base-q5_1.bin"));
        assert!(yaml.contains("ru: ru_RU-irina-medium"));
        assert!(yaml.contains("en: en_US-amy-medium"));
        
        println!("✅ Settings serialization for save successful");
    }

    // ============================================================
    // ТЕСТЫ: ИНТЕГРАЦИОННЫЕ
    // ============================================================

    #[test]
    fn test_settings_roundtrip() {
        let original = AppSettings {
            theme: "dark".to_string(),
            language: "en".to_string(),
            auto_preview: true,
            max_preview_size: 2.5,
            show_extensions: false,
            enable_cache: false,
            enable_archive: true,
            archive_format: "tar.gz".to_string(),
            synthesis_model: {
                let mut map = HashMap::new();
                map.insert("ru".to_string(), "ru_RU-irina-medium".to_string());
                map.insert("en".to_string(), "en_US-amy-medium".to_string());
                map
            },
            recognition_model: "ggml-base-q5_1.bin".to_string(),
        };
        
        let yaml = serde_yaml::to_string(&original).unwrap();
        let deserialized: AppSettings = serde_yaml::from_str(&yaml).unwrap();
        
        assert_eq!(deserialized.theme, original.theme);
        assert_eq!(deserialized.language, original.language);
        assert_eq!(deserialized.auto_preview, original.auto_preview);
        assert_eq!(deserialized.max_preview_size, original.max_preview_size);
        assert_eq!(deserialized.show_extensions, original.show_extensions);
        assert_eq!(deserialized.enable_cache, original.enable_cache);
        assert_eq!(deserialized.enable_archive, original.enable_archive);
        assert_eq!(deserialized.archive_format, original.archive_format);
        assert_eq!(deserialized.recognition_model, original.recognition_model);
        assert_eq!(deserialized.synthesis_model, original.synthesis_model);
        
        println!("✅ Roundtrip successful");
    }

    #[test]
    fn test_all_theme_language_combinations() {
        for theme in VALID_THEMES {
            for lang in VALID_LANGUAGES {
                let settings = AppSettings {
                    theme: theme.to_string(),
                    language: lang.to_string(),
                    ..AppSettings::default()
                };
                
                assert_eq!(settings.theme, theme);
                assert_eq!(settings.language, lang);
                
                let yaml = serde_yaml::to_string(&settings).unwrap();
                assert!(yaml.contains(&format!("theme: {}", theme)));
                assert!(yaml.contains(&format!("language: {}", lang)));
                
                println!("✅ Theme '{}' + Language '{}' works", theme, lang);
            }
        }
    }

    // ============================================================
    // ТЕСТЫ: EDGE CASES
    // ============================================================

   #[test]
    fn test_empty_synthesis_model() {
        // 🔥 Исправлено: используем ..Default::default()
        let settings = AppSettings {
            synthesis_model: HashMap::new(),
            ..AppSettings::default()
        };
        
        assert!(settings.synthesis_model.is_empty());
        assert!(!settings.synthesis_model.contains_key("ru"));
        assert!(!settings.synthesis_model.contains_key("en"));
        
        let yaml = serde_yaml::to_string(&settings).unwrap();
        assert!(yaml.contains("synthesis_model: {}"));
        
        println!("✅ Empty synthesis model works");
    }

    #[test]
    fn test_custom_recognition_model() {
        // 🔥 Исправлено: используем ..Default::default()
        let settings = AppSettings {
            recognition_model: "ggml-large-v3-turbo-q5_0.bin".to_string(),
            ..AppSettings::default()
        };
        
        assert_eq!(settings.recognition_model, "ggml-large-v3-turbo-q5_0.bin");
        
        let yaml = serde_yaml::to_string(&settings).unwrap();
        assert!(yaml.contains("recognition_model: ggml-large-v3-turbo-q5_0.bin"));
        
        println!("✅ Custom recognition model works");
    }

    #[test]
    fn test_custom_synthesis_model() {
        let mut settings = AppSettings::default();
        let mut custom_map = HashMap::new();
        custom_map.insert("ru".to_string(), "ru_RU-irina-medium".to_string());
        custom_map.insert("en".to_string(), "en_US-amy-medium".to_string());
        settings.synthesis_model = custom_map;
        
        assert_eq!(settings.synthesis_model.get("ru"), Some(&"ru_RU-irina-medium".to_string()));
        assert_eq!(settings.synthesis_model.get("en"), Some(&"en_US-amy-medium".to_string()));
        
        let yaml = serde_yaml::to_string(&settings).unwrap();
        assert!(yaml.contains("ru: ru_RU-irina-medium"));
        assert!(yaml.contains("en: en_US-amy-medium"));
        
        println!("✅ Custom synthesis model works");
    }

    #[test]
    fn test_archive_format_values() {
        for format in VALID_ARCHIVE_FORMATS {
            let settings = AppSettings {
                archive_format: format.to_string(),
                ..AppSettings::default()
            };
            
            assert_eq!(settings.archive_format, format);
            let yaml = serde_yaml::to_string(&settings).unwrap();
            assert!(yaml.contains(&format!("archive_format: {}", format)));
            
            println!("✅ Archive format: {}", format);
        }
    }
}