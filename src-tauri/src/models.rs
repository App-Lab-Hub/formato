// src-tauri/src/models.rs
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use crate::paths::app_root;

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
    let models_dir = app_root().join("models").join(model_type);
    models_dir.join(model_name)
}

fn check_model_exists(model_path: &PathBuf) -> ModelStatus {
    if model_path.exists() {
        let size = std::fs::metadata(model_path)
            .ok()
            .and_then(|m| Some(m.len()));
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
    let model_dir = app_root().join("models/piper");
    if !model_dir.exists() {
        std::fs::create_dir_all(&model_dir)
            .map_err(|e| format!("Cannot create model dir: {}", e))?;
    }
    
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
    
    let onnx_url = format!("{}/{}/{}/{}/medium/{}.onnx", base_url, lang, voice, voice_name, model_name);
    let config_url = format!("{}/{}/{}/{}/medium/{}.onnx.json", base_url, lang, voice, voice_name, model_name);
    
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
    let model_dir = app_root().join("models/whisper");
    if !model_dir.exists() {
        std::fs::create_dir_all(&model_dir)
            .map_err(|e| format!("Cannot create model dir: {}", e))?;
    }
    
    let model_path = model_dir.join(&model_name);
    if model_path.exists() {
        return Ok(());
    }
    
    // Исправляем URL для turbo модели
    let url = if model_name == "ggml-large-v3-turbo-q5_0.bin" {
        "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo-q5_0.bin".to_string()
    } else {
        format!("https://huggingface.co/ggerganov/whisper.cpp/resolve/main/{}", model_name)
    };
    
    let model_path_clone = model_path.clone();
    
    let result = tokio::task::spawn_blocking(move || {
        download_file(&url, &model_path_clone)
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?;
    
    result
}

fn download_file(url: &str, output_path: &PathBuf) -> Result<(), String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(600))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
    
    let response = client.get(url).send()
        .map_err(|e| format!("Download request failed: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("Server returned error: {}", response.status()));
    }
    
    let bytes = response.bytes()
        .map_err(|e| format!("Failed to read download bytes: {}", e))?;
    
    let mut file = std::fs::File::create(output_path)
        .map_err(|e| format!("Failed to create file: {}", e))?;
    
    std::io::Write::write_all(&mut file, &bytes)
        .map_err(|e| format!("Failed to write data: {}", e))?;
    
    Ok(())
}