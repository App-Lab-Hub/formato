// src-tauri/src/convert/text_to_audio.rs

use std::fs;
use std::path::Path;
use tempfile::Builder;
use crate::convert::{calculate_conversion_hash, get_app_dir_path_with_hash};
use crate::convert::audio;
use crate::paths;
use piper_rs::Piper;
use std::fs::File;
use reqwest::blocking::Client;
use std::time::Duration;
use anyhow::anyhow;
use std::io::Write;
pub fn convert_text_to_audio(path: &str, from: &str, to: &str) -> Result<String, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("Cannot read file: {}", e))?;

    if text.trim().is_empty() {
        return Err("File is empty".to_string());
    }

    let hash = calculate_conversion_hash(path, from, to)
        .map_err(|e| format!("Hash error: {}", e))?;

    let final_path = get_app_dir_path_with_hash(path, to, &hash)?;

    let temp_wav = generate_speech_with_piper(&text)?;
    
    if !Path::new(&temp_wav).exists() {
        return Err(format!("WAV file not created: {}", temp_wav));
    }

    let metadata = fs::metadata(&temp_wav)
        .map_err(|e| format!("Cannot get WAV metadata: {}", e))?;
    if metadata.len() == 0 {
        return Err("Generated WAV file is empty".to_string());
    }

    println!("✅ WAV file created: {} ({} bytes)", temp_wav, metadata.len());

    // Конвертируем WAV в целевой аудио формат через audio::convert_audio_to_audio
    let audio_output = audio::convert_audio_to_audio(&temp_wav, "wav", to)?;
    
    // Если файл сохранился не туда - перемещаем
    if audio_output != final_path {
        if let Some(parent) = Path::new(&final_path).parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Cannot create output dir: {}", e))?;
            }
        }
        fs::rename(&audio_output, &final_path)
            .map_err(|e| format!("Cannot move file: {}", e))?;
    }

    let _ = fs::remove_file(&temp_wav);

    println!("✅ Text to audio conversion complete: {}", final_path);
    Ok(final_path)
}

/// Генерация речи через Piper с автоматическим скачиванием модели
fn generate_speech_with_piper(text: &str) -> Result<String, String> {
    if text.trim().is_empty() {
        return Err("Text is empty for TTS generation".to_string());
    }

    // Путь к моделям в app_dir
    let model_name = "ru_RU-dmitri-medium";
    let model_dir = paths::app_root().join("models/piper");
    
    if !model_dir.exists() {
        std::fs::create_dir_all(&model_dir)
            .map_err(|e| format!("Failed to create models directory: {}", e))?;
    }
    
    let onnx_path = model_dir.join(format!("{}.onnx", model_name));
    let config_path = model_dir.join(format!("{}.onnx.json", model_name));
    
    // Скачиваем модель если её нет
    if !onnx_path.exists() {
        println!("📥 Скачиваю русскую модель Piper (Дмитрий, ~63 МБ)...");
        download_piper_model(model_name, &onnx_path, &config_path)
            .map_err(|e| format!("Failed to download model: {}", e))?;
        println!("✅ Модель скачана!");
    }
    
    println!("🔄 Загружаем модель Piper (Дмитрий)...");
    
    // Инициализируем Piper
    let mut piper = Piper::new(&onnx_path, &config_path)
        .map_err(|e| format!("Failed to load Piper model: {}", e))?;
    
    println!("🔄 Генерируем речь...");
    
    // Синтезируем речь
    let (samples, sample_rate) = piper
        .create(text, false, None, None, None, None)
        .map_err(|e| format!("TTS synthesis failed: {}", e))?;
    
    // Конвертируем f32 в i16
    let samples_i16: Vec<i16> = samples
        .iter()
        .map(|&s| (s * i16::MAX as f32) as i16)
        .collect();
    
    // Создаем временный файл для WAV
    let temp_file = Builder::new()
        .suffix(".wav")
        .prefix("piper_")
        .tempfile()
        .map_err(|e| format!("Cannot create temp file: {}", e))?;
    
    let temp_path = temp_file.path()
        .to_str()
        .ok_or("Invalid temp path")?
        .to_string();
    
    // Сохраняем в WAV
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    
    let mut writer = hound::WavWriter::create(&temp_path, spec)
        .map_err(|e| format!("Failed to create WAV file: {}", e))?;
    
    for sample in samples_i16 {
        writer.write_sample(sample)
            .map_err(|e| format!("Failed to write sample: {}", e))?;
    }
    writer.finalize()
        .map_err(|e| format!("Failed to finalize WAV: {}", e))?;
    
    println!("✅ Speech generated: {}", temp_path);
    
    Ok(temp_path)
}

/// Скачивание модели Piper с Hugging Face
fn download_piper_model(model_name: &str, onnx_path: &Path, config_path: &Path) -> Result<(), String> {
    let base_url = "https://huggingface.co/rhasspy/piper-voices/resolve/main";
    
    // Правильные URL для Дмитрия
    let onnx_url = format!("{}/ru/ru_RU/dmitri/medium/{}.onnx", base_url, model_name);
    let config_url = format!("{}/ru/ru_RU/dmitri/medium/{}.onnx.json", base_url, model_name);
    
    download_file(&onnx_url, onnx_path)
        .map_err(|e| format!("Failed to download ONNX model: {}", e))?;
    download_file(&config_url, config_path)
        .map_err(|e| format!("Failed to download config: {}", e))?;
    
    Ok(())
}

/// Скачивание файла
fn download_file(url: &str, path: &Path) -> Result<(), String> {
    println!("   Скачиваю {}...", url);
    
    let client = Client::builder()
        .timeout(Duration::from_secs(300))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
    
    let response = client.get(url)
        .send()
        .map_err(|e| format!("Download failed: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()));
    }
    
    let bytes = response.bytes()
        .map_err(|e| format!("Failed to read response: {}", e))?;
    
    let mut file = File::create(path)
        .map_err(|e| format!("Failed to create file: {}", e))?;
    
    file.write_all(&bytes)
        .map_err(|e| format!("Failed to write file: {}", e))?;
    
    println!("   ✅ Скачано ({} байт)", bytes.len());
    
    Ok(())
}