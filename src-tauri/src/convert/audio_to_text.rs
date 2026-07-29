// src-tauri/src/convert/audio_to_text.rs

use transcribe_rs::whisper_cpp::{WhisperEngine, WhisperInferenceParams};
use transcribe_rs::audio::read_wav_samples;
use crate::convert::{calculate_conversion_hash, get_app_dir_path_with_hash};
use std::path::{Path, PathBuf};
use ffmpeg_sidecar::command::FfmpegCommand;

pub fn convert_audio_to_text(path: &str, from: &str, to: &str) -> Result<String, String> {


    let model_path = get_or_download_model()?;
    
    // Загружаем движок (теперь без несуществующих параметров)
    let mut engine = WhisperEngine::load(&model_path)
        .map_err(|e| format!("Failed to load Whisper model: {}", e))?;
    
    // ФИКС FFmpeg: Получаем незаблокированный путь во временной директории
    let temp_path = get_safe_temp_wav_path();
    convert_to_16khz_wav(path, &temp_path)?;
    
    // Читаем PCM сэмплы
    let samples = read_wav_samples(&temp_path)
        .map_err(|e| format!("Failed to load WAV: {}", e))?;
    
    // Сразу очищаем временный файл, так как данные уже в памяти Rust
    let _ = std::fs::remove_file(&temp_path);
    
    let params = WhisperInferenceParams {
        language: if from.contains("ru") || from.contains("russian") { 
            Some("ru".to_string()) 
        } else { 
            Some("en".to_string()) 
        },
        translate: false,
        ..Default::default()
    };
    
    let result = engine.transcribe_with(&samples, &params)
        .map_err(|e| format!("Transcription failed: {}", e))?;
    
    let hash = calculate_conversion_hash(path, from, to)
        .map_err(|e| format!("Hash error: {}", e))?;
    let output_path = get_app_dir_path_with_hash(path, to, &hash)?;
    
    if let Some(parent) = Path::new(&output_path).parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Cannot create output directory: {}", e))?;
        }
    }
    
    std::fs::write(&output_path, &result.text)
        .map_err(|e| format!("Cannot write output file: {}", e))?;
    
    Ok(output_path)
}

/// ФИКС FFmpeg: Принимает чистую строку пути и корректно создаёт аудиофайл
fn convert_to_16khz_wav(input_path: &str, output_path: &Path) -> Result<(), String> {
    let output_str = output_path.to_str().ok_or("Invalid temp path")?;
    
    let mut cmd = FfmpegCommand::new();
    cmd.input(input_path);
    cmd.args(&["-ar", "16000"]);
    cmd.args(&["-ac", "1"]);
    cmd.args(&["-c:a", "pcm_s16le"]);
    cmd.args(&["-y"]); // Принудительно перезаписывать файл
    cmd.output(output_str);
    
    let mut child = cmd.spawn().map_err(|e| format!("FFmpeg spawn failed: {}", e))?;
    let status = child.wait().map_err(|e| format!("FFmpeg wait failed: {}", e))?;
    
    if !status.success() {
        return Err(format!("FFmpeg conversion failed with status: {}", status));
    }
    
    if !output_path.exists() {
        return Err("FFmpeg didn't create output file".to_string());
    }
    
    Ok(())
}

/// Генерирует уникальный путь в системной папке /tmp, предотвращая конфликт дескрипторов
fn get_safe_temp_wav_path() -> PathBuf {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
        
    std::env::temp_dir().join(format!("whisper_input_{}.wav", timestamp))
}

fn get_or_download_model() -> Result<std::path::PathBuf, String> {
    let model_dir = crate::paths::app_root().join("models/whisper");
    if !model_dir.exists() {
        std::fs::create_dir_all(&model_dir).map_err(|e| format!("Cannot create model dir: {}", e))?;
    }
    
    let model_name = "ggml-base-q5_1.bin";
    let model_path = model_dir.join(model_name);
    if model_path.exists() {
        return Ok(model_path);
    }
    
    let url = format!("https://huggingface.co{}", model_name);
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(600))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
    
    let response = client.get(&url).send().map_err(|e| format!("Download request failed: {}", e))?;
    if !response.status().is_success() {
        return Err(format!("Server returned error: {}", response.status()));
    }
    
    let bytes = response.bytes().map_err(|e| format!("Failed to read download bytes: {}", e))?;
    let mut file = std::fs::File::create(&model_path).map_err(|e| format!("Failed to create model file: {}", e))?;
    std::io::Write::write_all(&mut file, &bytes).map_err(|e| format!("Failed to write model data: {}", e))?;
    
    Ok(model_path)
}
