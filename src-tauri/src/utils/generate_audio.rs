// src-tauri/src/utils/generate_audio.rs

use std::path::Path;
use tempfile::Builder;
use crate::paths;
use piper_rs::Piper;
use std::fs::File;
use reqwest::blocking::Client;
use std::time::Duration;
use anyhow::anyhow;
use std::io::Write;
use hound::{WavSpec, WavWriter};

/// Разбивает текст на части по предложениям
fn split_text_into_chunks(text: &str, max_chars: usize) -> Vec<String> {
    let sentences: Vec<&str> = text
        .split_inclusive(|c: char| c == '.' || c == '!' || c == '?' || c == '\n')
        .collect();
    
    let mut chunks = Vec::new();
    let mut current_chunk = String::new();
    
    for sentence in sentences {
        if current_chunk.len() + sentence.len() > max_chars {
            if !current_chunk.is_empty() {
                chunks.push(current_chunk.clone());
                current_chunk.clear();
            }
        }
        current_chunk.push_str(sentence);
    }
    
    if !current_chunk.is_empty() {
        chunks.push(current_chunk);
    }
    
    if chunks.is_empty() {
        chunks.push(text.to_string());
    }
    
    chunks
}

/// Определение языка текста (ru или en) на основе соотношения символов
fn detect_language(text: &str) -> String {
    let total_chars: usize = text.chars().filter(|c| !c.is_whitespace()).count();
    if total_chars == 0 {
        return "en".to_string();
    }
    
    let cyrillic_count = text.chars().filter(|c| {
        ('\u{0400}'..='\u{04FF}').contains(c) // Кириллица
    }).count();
    
    let cyrillic_percent = cyrillic_count as f32 / total_chars as f32 * 100.0;
    
    println!("🔍 Анализ текста: {}% кириллицы", cyrillic_percent.round());
    
    if cyrillic_percent >= 51.0 {
        "ru".to_string()
    } else {
        "en".to_string()
    }
}

/// Получение модели для языка
fn get_model_for_language(lang: &str) -> &'static str {
    match lang {
        "ru" => "ru_RU-dmitri-medium",
        "en" => "en_US-lessac-medium",
        _ => "ru_RU-dmitri-medium",
    }
}

/// Скачивание модели Piper с Hugging Face
fn download_piper_model(model_name: &str, onnx_path: &Path, config_path: &Path) -> Result<(), String> {
    let base_url = "https://huggingface.co/rhasspy/piper-voices/resolve/main";
    
    let (lang, voice, voice_name) = if model_name.starts_with("ru_RU") {
        ("ru", "ru_RU", "dmitri")
    } else if model_name.starts_with("en_US") {
        ("en", "en_US", "lessac")
    } else {
        ("ru", "ru_RU", "dmitri")
    };
    
    let onnx_url = format!("{}/{}/{}/{}/medium/{}.onnx", base_url, lang, voice, voice_name, model_name);
    let config_url = format!("{}/{}/{}/{}/medium/{}.onnx.json", base_url, lang, voice, voice_name, model_name);
    
    download_file(&onnx_url, onnx_path)?;
    download_file(&config_url, config_path)?;
    
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

/// Генерация речи через Piper с чанкингом, склейкой аудио и автоопределением языка
pub fn generate_speech_with_piper(text: &str) -> Result<String, String> {
    if text.trim().is_empty() {
        return Err("Text is empty for TTS generation".to_string());
    }

    // Определяем язык текста по соотношению символов (>=51% кириллицы = ru)
    let lang = detect_language(text);
    let model_name = get_model_for_language(&lang);
    
    println!("🌐 Определён язык: {} (модель: {})", lang, model_name);
    
    // Путь к моделям в app_dir
    let model_dir = paths::app_root().join("models/piper");
    
    if !model_dir.exists() {
        std::fs::create_dir_all(&model_dir)
            .map_err(|e| format!("Failed to create models directory: {}", e))?;
    }
    
    let onnx_path = model_dir.join(format!("{}.onnx", model_name));
    let config_path = model_dir.join(format!("{}.onnx.json", model_name));
    
    // Скачиваем модель если её нет
    if !onnx_path.exists() {
        println!("📥 Скачиваю модель Piper ({}, ~63 МБ)...", model_name);
        download_piper_model(model_name, &onnx_path, &config_path)?;
        println!("✅ Модель скачана!");
    }
    
    println!("🔄 Загружаем модель Piper ({})...", model_name);
    
    let mut piper = Piper::new(&onnx_path, &config_path)
        .map_err(|e| format!("Failed to load Piper model: {}", e))?;
    
    // Разбиваем текст на чанки (по 2000 символов)
    let max_chars = 2000;
    let chunks = split_text_into_chunks(text, max_chars);
    let total_chunks = chunks.len();
    
    println!("📝 Текст разбит на {} частей (макс. {} символов)", total_chunks, max_chars);
    
    // Создаем временный файл для результата
    let temp_file = Builder::new()
        .suffix(".wav")
        .prefix("piper_")
        .tempfile()
        .map_err(|e| format!("Cannot create temp file: {}", e))?;
    
    let temp_path = temp_file.path()
        .to_str()
        .ok_or("Invalid temp path")?
        .to_string();
    
    // Первый чанк определяет sample_rate
    let mut sample_rate = 22050;
    let mut all_samples: Vec<i16> = Vec::new();
    
    for (i, chunk) in chunks.iter().enumerate() {
        println!("🔄 [{}/{}] Синтез части ({} символов)...", i + 1, total_chunks, chunk.len());
        
        // Для плавной размеренной речи используем length_scale = 1.5
        let (samples, rate) = piper
            .create(
                chunk,
                false,
                None,                    // speaker_id (None = используем дефолтный)
                Some(1.5),              // length_scale - 1.5 = медленнее и плавнее
                // Some(0.667),            // noise_scale (стандарт)
                // Some(0.8),              // noise_w_scale (стандарт)
                None,
                None,
            )
            .map_err(|e| format!("TTS synthesis failed for chunk {}: {}", i + 1, e))?;
        
        if i == 0 {
            sample_rate = rate;
        }
        
        let samples_i16: Vec<i16> = samples
            .iter()
            .map(|&s| (s * i16::MAX as f32) as i16)
            .collect();
        
        all_samples.extend(samples_i16);
        
        println!("✅ [{}/{}] Готово ({} сэмплов)", i + 1, total_chunks, samples.len());
    }
    
    println!("🔄 Сохраняем объединенное аудио ({} сэмплов)...", all_samples.len());
    
    // Сохраняем все сэмплы в один WAV
    let spec = WavSpec {
        channels: 1,
        sample_rate: sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    
    let mut writer = WavWriter::create(&temp_path, spec)
        .map_err(|e| format!("Failed to create WAV file: {}", e))?;
    
    for sample in all_samples.iter() {
        writer.write_sample(*sample)
            .map_err(|e| format!("Failed to write sample: {}", e))?;
    }
    writer.finalize()
        .map_err(|e| format!("Failed to finalize WAV: {}", e))?;
    
    let _ = temp_file.keep();
    
    let duration_sec = all_samples.len() as f32 / sample_rate as f32;
    println!("✅ Speech generated: {} ({} сек)", temp_path, duration_sec);
    
    Ok(temp_path)
}