// src-tauri/src/models.rs
use crate::paths;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelStatus {
    pub exists: bool,
    pub path: Option<String>,
    pub size: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelsStatus {
    pub synthesis: std::collections::HashMap<String, ModelStatus>,
    pub recognition: std::collections::HashMap<String, ModelStatus>,
    pub has_any_synthesis: bool,
    pub has_any_recognition: bool,
}

fn get_model_path(model_type: &str, model_name: &str) -> PathBuf {
    let models_dir = match model_type {
        "piper" => paths::piper_models_dir(),
        "whisper" => paths::whisper_models_dir(),
        _ => paths::models_dir().join(model_type),
    };
    models_dir.join(model_name)
}

fn check_model_exists(model_path: &PathBuf) -> ModelStatus {
    if model_path.exists() {
        let size = std::fs::metadata(model_path).ok().map(|m| m.len());
        ModelStatus {
            exists: true,
            path: Some(model_path.to_string_lossy().to_string()),
            size,
        }
    } else {
        ModelStatus {
            exists: false,
            path: None,
            size: None,
        }
    }
}

#[tauri::command]
pub async fn get_models_status() -> ModelsStatus {
    // Синтез модели (Piper) - с .onnx для проверки файлов
    let synthesis_models = vec![
        ("ru_RU-dmitri-medium", "ru_RU-dmitri-medium.onnx"),
        ("ru_RU-irina-medium", "ru_RU-irina-medium.onnx"),
        ("en_US-lessac-medium", "en_US-lessac-medium.onnx"),
        ("en_US-amy-medium", "en_US-amy-medium.onnx"),
    ];

    let mut synthesis_map = std::collections::HashMap::new();
    let mut has_any_synthesis = false;

    for (model_key, model_file) in &synthesis_models {
        let path = get_model_path("piper", model_file);
        let status = check_model_exists(&path);
        if status.exists {
            has_any_synthesis = true;
        }
        synthesis_map.insert(model_key.to_string(), status);
    }

    // Распознавание модели (Whisper)
    let recognition_models = vec![
        "ggml-tiny-q5_1.bin",
        "ggml-base-q5_1.bin",
        "ggml-small-q5_1.bin",
        "ggml-medium-q5_0.bin",
        "ggml-large-v3-turbo-q5_0.bin",
    ];

    let mut recognition_map = std::collections::HashMap::new();
    let mut has_any_recognition = false;

    for model in &recognition_models {
        let path = get_model_path("whisper", model);
        let status = check_model_exists(&path);
        if status.exists {
            has_any_recognition = true;
        }
        recognition_map.insert(model.to_string(), status);
    }

    ModelsStatus {
        synthesis: synthesis_map,
        recognition: recognition_map,
        has_any_synthesis,
        has_any_recognition,
    }
}

#[tauri::command]
pub async fn download_synthesis_model(model_name: String) -> Result<(), String> {
    let model_dir = paths::piper_models_dir();

    // ✅ Добавляем .onnx к имени файла
    let model_path = model_dir.join(format!("{}.onnx", model_name));
    if model_path.exists() {
        return Ok(());
    }

    let base_url = "https://huggingface.co/rhasspy/piper-voices/resolve/main";

    let (lang, voice) = if model_name.starts_with("ru_RU") {
        ("ru", "ru_RU")
    } else {
        ("en", "en_US")
    };

    let voice_name = if model_name.contains("dmitri") {
        "dmitri"
    } else if model_name.contains("irina") {
        "irina"
    } else if model_name.contains("lessac") {
        "lessac"
    } else if model_name.contains("amy") {
        "amy"
    } else {
        return Err("Unknown voice".to_string());
    };

    let onnx_url = format!(
        "{}/{}/{}/{}/medium/{}.onnx",
        base_url, lang, voice, voice_name, model_name
    );
    let config_url = format!(
        "{}/{}/{}/{}/medium/{}.onnx.json",
        base_url, lang, voice, voice_name, model_name
    );

    let model_path_clone = model_path.clone();
    let config_path = model_dir.join(format!("{}.onnx.json", model_name));
    let config_path_clone = config_path.clone();

    let result = tokio::task::spawn_blocking(move || {
        download_file(&onnx_url, &model_path_clone)?;
        download_file(&config_url, &config_path_clone)?;
        Ok::<(), String>(())
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?;

    result
}

#[tauri::command]
pub async fn download_recognition_model(model_name: String) -> Result<(), String> {
    let model_dir = paths::whisper_models_dir();

    let model_path = model_dir.join(&model_name);
    if model_path.exists() {
        return Ok(());
    }

    // Исправляем URL для turbo модели
    let url = if model_name == "ggml-large-v3-turbo-q5_0.bin" {
        "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo-q5_0.bin"
            .to_string()
    } else {
        format!(
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/{}",
            model_name
        )
    };

    let model_path_clone = model_path.clone();

    let result = tokio::task::spawn_blocking(move || download_file(&url, &model_path_clone))
        .await
        .map_err(|e| format!("Task failed: {}", e))?;

    result
}

fn download_file(url: &str, output_path: &PathBuf) -> Result<(), String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(600))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let response = client
        .get(url)
        .send()
        .map_err(|e| format!("Download request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Server returned error: {}", response.status()));
    }

    let bytes = response
        .bytes()
        .map_err(|e| format!("Failed to read download bytes: {}", e))?;

    let mut file =
        std::fs::File::create(output_path).map_err(|e| format!("Failed to create file: {}", e))?;

    std::io::Write::write_all(&mut file, &bytes)
        .map_err(|e| format!("Failed to write data: {}", e))?;

    Ok(())
}

// src-tauri/src/models.rs

// ... весь существующий код ...

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::fs;
    use std::path::{Path, PathBuf};
    use tempfile::tempdir;

    // ============================================================
    // ВСПОМОГАТЕЛЬНЫЕ ФУНКЦИИ
    // ============================================================

    /// Создает временную директорию для тестов
    fn setup_test_dir() -> tempfile::TempDir {
        tempdir().unwrap()
    }

    /// Создает фейковый файл модели
    fn create_fake_model(dir: &Path, name: &str, size: u64) -> PathBuf {
        let path = dir.join(name);
        let data = vec![0u8; size as usize];
        fs::write(&path, data).unwrap();
        path
    }

    /// Проверяет, что модель существует
    fn assert_model_exists(path: &Path) {
        assert!(path.exists(), "Model not found: {:?}", path);
        let size = fs::metadata(path).unwrap().len();
        assert!(size > 0, "Model is empty: {:?}", path);
        println!(
            "✅ Model exists: {:?} ({} bytes)",
            path.file_name().unwrap(),
            size
        );
    }

    /// Проверяет, что модель не существует
    fn assert_model_not_exists(path: &Path) {
        assert!(!path.exists(), "Model should not exist: {:?}", path);
        println!("✅ Model does not exist: {:?}", path.file_name().unwrap());
    }

    // ============================================================
    // МОДУЛЬ 1: БАЗОВЫЕ ТЕСТЫ (check_model_exists, get_model_path, структуры)
    // ============================================================

    mod basic {
        use super::*;

        // ============================================================
        // ТЕСТЫ: check_model_exists
        // ============================================================

        #[test]
        fn test_check_model_exists_found() {
            let temp_dir = setup_test_dir();
            let model_path = create_fake_model(temp_dir.path(), "model.onnx", 1024);

            let status = check_model_exists(&model_path);
            assert!(status.exists);
            assert_eq!(status.path, Some(model_path.to_string_lossy().to_string()));
            assert_eq!(status.size, Some(1024));
            println!(
                "✅ Model found: {:?} ({} bytes)",
                status.path,
                status.size.unwrap()
            );
        }

        #[test]
        fn test_check_model_exists_not_found() {
            let temp_dir = setup_test_dir();
            let model_path = temp_dir.path().join("nonexistent.onnx");

            let status = check_model_exists(&model_path);
            assert!(!status.exists);
            assert!(status.path.is_none());
            assert!(status.size.is_none());
            println!("✅ Model not found as expected");
        }

        // ============================================================
        // ТЕСТЫ: get_model_path
        // ============================================================

        #[test]
        fn test_get_model_path_piper() {
            let path = get_model_path("piper", "test_model.onnx");
            let path_str = path.to_string_lossy().to_string();
            assert!(path_str.contains("piper"));
            assert!(path_str.ends_with("test_model.onnx"));
            println!("✅ Piper model path: {}", path_str);
        }

        #[test]
        fn test_get_model_path_whisper() {
            let path = get_model_path("whisper", "ggml-tiny.bin");
            let path_str = path.to_string_lossy().to_string();
            assert!(path_str.contains("whisper"));
            assert!(path_str.ends_with("ggml-tiny.bin"));
            println!("✅ Whisper model path: {}", path_str);
        }

        #[test]
        fn test_get_model_path_unknown() {
            let path = get_model_path("unknown", "model.bin");
            let path_str = path.to_string_lossy().to_string();
            assert!(path_str.contains("unknown"));
            assert!(path_str.ends_with("model.bin"));
            println!("✅ Unknown model path: {}", path_str);
        }

        // ============================================================
        // ТЕСТЫ: get_models_status
        // ============================================================

        #[tokio::test]
        async fn test_get_models_status_structure() {
            let status = get_models_status().await;

            assert!(!status.synthesis.is_empty());
            assert!(!status.recognition.is_empty());

            let expected_synthesis = vec![
                "ru_RU-dmitri-medium",
                "ru_RU-irina-medium",
                "en_US-lessac-medium",
                "en_US-amy-medium",
            ];
            for model in expected_synthesis {
                assert!(status.synthesis.contains_key(model));
            }

            let expected_recognition = vec![
                "ggml-tiny-q5_1.bin",
                "ggml-base-q5_1.bin",
                "ggml-small-q5_1.bin",
                "ggml-medium-q5_0.bin",
                "ggml-large-v3-turbo-q5_0.bin",
            ];
            for model in expected_recognition {
                assert!(status.recognition.contains_key(model));
            }

            println!("✅ Models status structure OK");
        }

        // ============================================================
        // ТЕСТЫ: Структуры моделей
        // ============================================================

        #[test]
        fn test_model_status_serialization() {
            let status = ModelStatus {
                exists: true,
                path: Some("/path/to/model.bin".to_string()),
                size: Some(1024),
            };

            let json = serde_json::to_string(&status).unwrap();
            let deserialized: ModelStatus = serde_json::from_str(&json).unwrap();

            assert_eq!(deserialized.exists, status.exists);
            assert_eq!(deserialized.path, status.path);
            assert_eq!(deserialized.size, status.size);
            println!("✅ ModelStatus serialization: {}", json);
        }

        #[test]
        fn test_models_status_serialization() {
            let mut synthesis = HashMap::new();
            synthesis.insert(
                "model1".to_string(),
                ModelStatus {
                    exists: true,
                    path: Some("/path/model1.bin".to_string()),
                    size: Some(512),
                },
            );

            let status = ModelsStatus {
                synthesis,
                recognition: HashMap::new(),
                has_any_synthesis: true,
                has_any_recognition: false,
            };

            let json = serde_json::to_string(&status).unwrap();
            let deserialized: ModelsStatus = serde_json::from_str(&json).unwrap();

            assert_eq!(deserialized.has_any_synthesis, status.has_any_synthesis);
            assert_eq!(deserialized.has_any_recognition, status.has_any_recognition);
            assert!(deserialized.synthesis.contains_key("model1"));
            println!("✅ ModelsStatus serialization: {}", json);
        }

        // ============================================================
        // ТЕСТЫ: Список моделей
        // ============================================================

        #[test]
        fn test_synthesis_models_list() {
            let expected_models = vec![
                "ru_RU-dmitri-medium",
                "ru_RU-irina-medium",
                "en_US-lessac-medium",
                "en_US-amy-medium",
            ];

            assert_eq!(expected_models.len(), 4);

            for model in &expected_models {
                let file_name = format!("{}.onnx", model);
                assert!(file_name.ends_with(".onnx"));
            }
            println!("✅ Synthesis models list: {:?}", expected_models);
        }

        #[test]
        fn test_recognition_models_list() {
            let expected_models = vec![
                "ggml-tiny-q5_1.bin",
                "ggml-base-q5_1.bin",
                "ggml-small-q5_1.bin",
                "ggml-medium-q5_0.bin",
                "ggml-large-v3-turbo-q5_0.bin",
            ];

            assert_eq!(expected_models.len(), 5);

            for model in &expected_models {
                assert!(model.ends_with(".bin"));
            }
            println!("✅ Recognition models list: {:?}", expected_models);
        }
    }

    // ============================================================
    // МОДУЛЬ 2: ТЕСТЫ СКАЧИВАНИЯ (download)
    // ============================================================

    mod download {
        use super::*;

        // ============================================================
        // ТЕСТЫ: download_synthesis_model
        // ============================================================

        #[tokio::test]
        async fn test_unknown_voice() {
            let result = download_synthesis_model("unknown_voice".to_string()).await;
            assert!(result.is_err());
            let err = result.err().unwrap();
            assert!(err.contains("Unknown voice"));
            println!("❌ Expected error: {}", err);
        }

        #[tokio::test]
        #[ignore = "Requires network and writes to disk - run manually"]
        async fn test_download_ru_dmitri() {
            let result = download_synthesis_model("ru_RU-dmitri-medium".to_string()).await;
            assert!(result.is_ok());
            println!("✅ Downloaded ru_RU-dmitri-medium model");

            let model_dir = paths::piper_models_dir();
            let model_path = model_dir.join("ru_RU-dmitri-medium.onnx");
            let config_path = model_dir.join("ru_RU-dmitri-medium.onnx.json");

            assert_model_exists(&model_path);
            assert_model_exists(&config_path);
        }

        #[tokio::test]
        #[ignore = "Requires network and writes to disk - run manually"]
        async fn test_download_ru_irina() {
            let result = download_synthesis_model("ru_RU-irina-medium".to_string()).await;
            assert!(result.is_ok());
            println!("✅ Downloaded ru_RU-irina-medium model");

            let model_dir = paths::piper_models_dir();
            let model_path = model_dir.join("ru_RU-irina-medium.onnx");
            let config_path = model_dir.join("ru_RU-irina-medium.onnx.json");

            assert_model_exists(&model_path);
            assert_model_exists(&config_path);
        }

        #[tokio::test]
        #[ignore = "Requires network and writes to disk - run manually"]
        async fn test_download_en_lessac() {
            let result = download_synthesis_model("en_US-lessac-medium".to_string()).await;
            assert!(result.is_ok());
            println!("✅ Downloaded en_US-lessac-medium model");

            let model_dir = paths::piper_models_dir();
            let model_path = model_dir.join("en_US-lessac-medium.onnx");
            let config_path = model_dir.join("en_US-lessac-medium.onnx.json");

            assert_model_exists(&model_path);
            assert_model_exists(&config_path);
        }

        #[tokio::test]
        #[ignore = "Requires network and writes to disk - run manually"]
        async fn test_download_en_amy() {
            let result = download_synthesis_model("en_US-amy-medium".to_string()).await;
            assert!(result.is_ok());
            println!("✅ Downloaded en_US-amy-medium model");

            let model_dir = paths::piper_models_dir();
            let model_path = model_dir.join("en_US-amy-medium.onnx");
            let config_path = model_dir.join("en_US-amy-medium.onnx.json");

            assert_model_exists(&model_path);
            assert_model_exists(&config_path);
        }

        #[tokio::test]
        async fn test_synthesis_already_exists() {
            let result = download_synthesis_model("ru_RU-dmitri-medium".to_string()).await;
            let _ = result;
        }

        // ============================================================
        // ТЕСТЫ: download_recognition_model
        // ============================================================

        #[tokio::test]
        #[ignore = "Requires network and writes to disk - run manually"]
        async fn test_download_tiny() {
            let result = download_recognition_model("ggml-tiny-q5_1.bin".to_string()).await;
            assert!(result.is_ok());
            println!("✅ Downloaded ggml-tiny-q5_1.bin model");

            let model_dir = paths::whisper_models_dir();
            let model_path = model_dir.join("ggml-tiny-q5_1.bin");
            assert_model_exists(&model_path);
        }

        #[tokio::test]
        #[ignore = "Requires network and writes to disk - run manually"]
        async fn test_download_base() {
            let result = download_recognition_model("ggml-base-q5_1.bin".to_string()).await;
            assert!(result.is_ok());
            println!("✅ Downloaded ggml-base-q5_1.bin model");

            let model_dir = paths::whisper_models_dir();
            let model_path = model_dir.join("ggml-base-q5_1.bin");
            assert_model_exists(&model_path);
        }

        #[tokio::test]
        #[ignore = "Requires network and writes to disk - run manually"]
        async fn test_download_small() {
            let result = download_recognition_model("ggml-small-q5_1.bin".to_string()).await;
            assert!(result.is_ok());
            println!("✅ Downloaded ggml-small-q5_1.bin model");

            let model_dir = paths::whisper_models_dir();
            let model_path = model_dir.join("ggml-small-q5_1.bin");
            assert_model_exists(&model_path);
        }

        #[tokio::test]
        #[ignore = "Requires network and writes to disk - run manually"]
        async fn test_download_medium() {
            let result = download_recognition_model("ggml-medium-q5_0.bin".to_string()).await;
            assert!(result.is_ok());
            println!("✅ Downloaded ggml-medium-q5_0.bin model");

            let model_dir = paths::whisper_models_dir();
            let model_path = model_dir.join("ggml-medium-q5_0.bin");
            assert_model_exists(&model_path);
        }

        #[tokio::test]
        #[ignore = "Requires network and writes to disk - run manually"]
        async fn test_download_large_turbo() {
            let result =
                download_recognition_model("ggml-large-v3-turbo-q5_0.bin".to_string()).await;
            assert!(result.is_ok());
            println!("✅ Downloaded ggml-large-v3-turbo-q5_0.bin model");

            let model_dir = paths::whisper_models_dir();
            let model_path = model_dir.join("ggml-large-v3-turbo-q5_0.bin");
            assert_model_exists(&model_path);
        }

        #[tokio::test]
        async fn test_recognition_already_exists() {
            let result = download_recognition_model("ggml-tiny-q5_1.bin".to_string()).await;
            let _ = result;
        }

        // ============================================================
        // ТЕСТЫ: download_file
        // ============================================================

        #[test]
        #[ignore = "Requires network and writes to disk - run manually"]
        fn test_download_file_success() {
            let temp_dir = setup_test_dir();
            let output_path = temp_dir.path().join("test.txt");

            let url = "https://raw.githubusercontent.com/rust-lang/rust/master/LICENSE-APACHE";
            let result = download_file(url, &output_path);

            assert!(result.is_ok());
            assert!(output_path.exists());
            assert!(output_path.metadata().unwrap().len() > 0);
            println!(
                "✅ Downloaded file: {:?} ({} bytes)",
                output_path,
                output_path.metadata().unwrap().len()
            );
        }

        #[test]
        fn test_download_file_invalid_url() {
            let temp_dir = setup_test_dir();
            let output_path = temp_dir.path().join("test.txt");

            let url = "https://invalid.url/that/does/not/exist";
            let result = download_file(url, &output_path);

            assert!(result.is_err());
            let err = result.err().unwrap();
            assert!(
                err.contains("Download request failed") || err.contains("Server returned error")
            );
            println!("❌ Expected error: {}", err);
        }
    }

    // ============================================================
    // МОДУЛЬ 3: ТЕСТЫ СУЩЕСТВОВАНИЯ МОДЕЛЕЙ (exist)
    // ============================================================

    mod exist {
        use super::*;

        // ============================================================
        // ТЕСТЫ: Проверка наличия моделей в папках
        // ============================================================

        #[test]
        fn test_piper_models_directory() {
            let piper_dir = paths::piper_models_dir();
            if piper_dir.exists() {
                println!("✅ Piper models directory exists: {:?}", piper_dir);

                let entries = fs::read_dir(&piper_dir).unwrap();
                let mut count = 0;
                for entry in entries {
                    let entry = entry.unwrap();
                    let path = entry.path();
                    if path.is_file() {
                        count += 1;
                        println!(
                            "  📁 {} ({} bytes)",
                            path.file_name().unwrap().to_string_lossy(),
                            fs::metadata(&path).unwrap().len()
                        );
                    }
                }
                println!("📊 Total Piper models: {}", count);
            } else {
                println!("❌ Piper models directory does not exist: {:?}", piper_dir);
            }
        }

        #[test]
        fn test_whisper_models_directory() {
            let whisper_dir = paths::whisper_models_dir();
            if whisper_dir.exists() {
                println!("✅ Whisper models directory exists: {:?}", whisper_dir);

                let entries = fs::read_dir(&whisper_dir).unwrap();
                let mut count = 0;
                for entry in entries {
                    let entry = entry.unwrap();
                    let path = entry.path();
                    if path.is_file() {
                        count += 1;
                        println!(
                            "  📁 {} ({} bytes)",
                            path.file_name().unwrap().to_string_lossy(),
                            fs::metadata(&path).unwrap().len()
                        );
                    }
                }
                println!("📊 Total Whisper models: {}", count);
            } else {
                println!(
                    "❌ Whisper models directory does not exist: {:?}",
                    whisper_dir
                );
            }
        }

        // ============================================================
        // ТЕСТЫ: Проверка конкретных моделей
        // ============================================================

        #[test]
        fn test_specific_piper_models() {
            let piper_dir = paths::piper_models_dir();
            let models = vec![
                "ru_RU-dmitri-medium.onnx",
                "ru_RU-irina-medium.onnx",
                "en_US-lessac-medium.onnx",
                "en_US-amy-medium.onnx",
            ];

            for model in models {
                let path = piper_dir.join(model);
                if path.exists() {
                    let size = fs::metadata(&path).unwrap().len();
                    println!("✅ {} exists ({} bytes)", model, size);
                } else {
                    println!("❌ {} does not exist", model);
                }
            }
        }

        #[test]
        fn test_specific_whisper_models() {
            let whisper_dir = paths::whisper_models_dir();
            let models = vec![
                "ggml-tiny-q5_1.bin",
                "ggml-base-q5_1.bin",
                "ggml-small-q5_1.bin",
                "ggml-medium-q5_0.bin",
                "ggml-large-v3-turbo-q5_0.bin",
            ];

            for model in models {
                let path = whisper_dir.join(model);
                if path.exists() {
                    let size = fs::metadata(&path).unwrap().len();
                    println!("✅ {} exists ({} bytes)", model, size);
                } else {
                    println!("❌ {} does not exist", model);
                }
            }
        }

        // ============================================================
        // ТЕСТЫ: Очистка всех моделей
        // ============================================================

        #[test]
        #[ignore = "Deletes models - run manually with caution"]
        fn test_cleanup_all_models() {
            let piper_dir = paths::piper_models_dir();
            let whisper_dir = paths::whisper_models_dir();

            println!("⚠️ Deleting all Piper models...");
            if piper_dir.exists() {
                for entry in fs::read_dir(&piper_dir).unwrap() {
                    let entry = entry.unwrap();
                    let path = entry.path();
                    if path.is_file() {
                        fs::remove_file(&path).unwrap();
                        println!("  🗑️ Deleted: {:?}", path.file_name().unwrap());
                    }
                }
            }

            println!("⚠️ Deleting all Whisper models...");
            if whisper_dir.exists() {
                for entry in fs::read_dir(&whisper_dir).unwrap() {
                    let entry = entry.unwrap();
                    let path = entry.path();
                    if path.is_file() {
                        fs::remove_file(&path).unwrap();
                        println!("  🗑️ Deleted: {:?}", path.file_name().unwrap());
                    }
                }
            }

            // Проверяем, что папки пустые
            if piper_dir.exists() {
                let count = fs::read_dir(&piper_dir).unwrap().count();
                assert_eq!(count, 0, "Piper directory should be empty");
                println!("✅ Piper directory is empty: {:?}", piper_dir);
            }

            if whisper_dir.exists() {
                let count = fs::read_dir(&whisper_dir).unwrap().count();
                assert_eq!(count, 0, "Whisper directory should be empty");
                println!("✅ Whisper directory is empty: {:?}", whisper_dir);
            }
        }
    }
}
