// src-tauri/src/convert/document_to_audio.rs

use std::fs;
use std::path::Path;
use tempfile::Builder;
use crate::convert::{calculate_conversion_hash, get_app_dir_path_with_hash, parse_document};
use crate::convert::audio;
use crate::paths;
use piper_rs::Piper;
use std::fs::File;
use reqwest::blocking::Client;
use std::time::Duration;
use anyhow::anyhow;
use std::io::Write;
use hound::{WavSpec, WavWriter};

pub fn convert_document_to_audio(path: &str, from: &str, to: &str) -> Result<String, String> {
    // 1. Парсим документ в JSON
    let json_value = parse_document(path, from)?;
    
    // 2. Преобразуем JSON в читаемый текст в зависимости от формата
    let text = match from {
        "docx" | "odt" | "pdf" => {
            if let Some(text) = json_value.get("text").and_then(|v| v.as_str()) {
                text.to_string()
            } else {
                json_to_speech_text(&json_value, from)?
            }
        }
        "xlsx" => {
            json_to_speech_text_xlsx(&json_value)?
        }
        _ => {
            json_to_speech_text(&json_value, from)?
        }
    };
    
    if text.trim().is_empty() {
        return Err("No readable text found in document".to_string());
    }

    // 3. Вычисляем хеш для выходного файла
    let hash = calculate_conversion_hash(path, from, to)
        .map_err(|e| format!("Hash error: {}", e))?;

    let final_path = get_app_dir_path_with_hash(path, to, &hash)?;

    // 4. Генерируем речь через Piper (WAV)
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

    // 5. Конвертируем WAV в целевой аудио формат через audio::convert_audio_to_audio
    let audio_output = audio::convert_audio_to_audio(&temp_wav, "wav", to)?;
    
    // 6. Если файл сохранился не туда - перемещаем
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

    // 7. Удаляем временный WAV файл
    let _ = fs::remove_file(&temp_wav);

    println!("✅ Document to audio conversion complete: {}", final_path);
    Ok(final_path)
}

/// Преобразование JSON в читаемый текст для озвучивания (общий подход)
fn json_to_speech_text(json: &serde_json::Value, format: &str) -> Result<String, String> {
    match json {
        serde_json::Value::Object(map) => {
            let mut text = String::new();
            text.push_str(&format!("{} document.\n", format.to_uppercase()));
            
            for (key, value) in map {
                text.push_str(&format!("{}: ", key));
                text.push_str(&value_to_text(value, 1)?);
                text.push_str(".\n");
            }
            
            Ok(text)
        }
        serde_json::Value::Array(arr) => {
            let mut text = String::new();
            text.push_str(&format!("{} document with {} items.\n", format.to_uppercase(), arr.len()));
            
            for (i, value) in arr.iter().enumerate() {
                text.push_str(&format!("Item {}: ", i + 1));
                text.push_str(&value_to_text(value, 1)?);
                text.push_str(".\n");
            }
            
            Ok(text)
        }
        serde_json::Value::String(s) => Ok(s.clone()),
        serde_json::Value::Number(n) => Ok(n.to_string()),
        serde_json::Value::Bool(b) => Ok(b.to_string()),
        serde_json::Value::Null => Ok("null".to_string()),
    }
}

/// Специальное преобразование XLSX в читаемый текст для озвучивания
fn json_to_speech_text_xlsx(json: &serde_json::Value) -> Result<String, String> {
    let mut text = String::new();
    text.push_str("Excel document.\n");
    
    if let Some(arr) = json.as_array() {
        for sheet in arr {
            if let Some(sheet_obj) = sheet.as_object() {
                if let Some(name) = sheet_obj.get("name").and_then(|v| v.as_str()) {
                    text.push_str(&format!("Sheet {}.\n", name));
                }
                
                if let Some(data) = sheet_obj.get("data").and_then(|v| v.as_array()) {
                    if data.is_empty() {
                        text.push_str("  Empty sheet.\n");
                        continue;
                    }
                    
                    let headers: Vec<String> = if let Some(first_row) = data.first() {
                        if let Some(row_obj) = first_row.as_object() {
                            row_obj.keys().cloned().collect()
                        } else {
                            Vec::new()
                        }
                    } else {
                        Vec::new()
                    };
                    
                    for (row_idx, row) in data.iter().enumerate() {
                        if let Some(row_obj) = row.as_object() {
                            let row_text: Vec<String> = headers.iter()
                                .filter_map(|header| {
                                    row_obj.get(header)
                                        .and_then(|v| v.as_str())
                                        .map(|v| format!("{}: {}", header, v))
                                })
                                .collect();
                            
                            if !row_text.is_empty() {
                                text.push_str(&format!("  Row {}: ", row_idx + 1));
                                text.push_str(&row_text.join(", "));
                                text.push_str(".\n");
                            }
                        }
                    }
                }
            }
        }
    }
    
    if text == "Excel document.\n" {
        return Err("No data found in Excel document".to_string());
    }
    
    Ok(text)
}

/// Рекурсивное преобразование значения в текст
fn value_to_text(value: &serde_json::Value, depth: usize) -> Result<String, String> {
    let indent = "  ".repeat(depth);
    
    match value {
        serde_json::Value::Object(map) => {
            if map.is_empty() {
                return Ok("empty object".to_string());
            }
            
            let mut parts = Vec::new();
            for (key, val) in map {
                let val_text = value_to_text(val, depth + 1)?;
                parts.push(format!("{}{}: {}", indent, key, val_text));
            }
            Ok(parts.join(". "))
        }
        serde_json::Value::Array(arr) => {
            if arr.is_empty() {
                return Ok("empty array".to_string());
            }
            
            let mut parts = Vec::new();
            for (i, val) in arr.iter().enumerate() {
                let val_text = value_to_text(val, depth + 1)?;
                parts.push(format!("{}{}: {}", indent, i + 1, val_text));
            }
            Ok(parts.join(". "))
        }
        serde_json::Value::String(s) => Ok(s.clone()),
        serde_json::Value::Number(n) => Ok(n.to_string()),
        serde_json::Value::Bool(b) => Ok(b.to_string()),
        serde_json::Value::Null => Ok("null".to_string()),
    }
}

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

/// Генерация речи через Piper с чанкингом, склейкой аудио и автоопределением языка
fn generate_speech_with_piper(text: &str) -> Result<String, String> {
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
        download_piper_model(model_name, &onnx_path, &config_path)
            .map_err(|e| format!("Failed to download model: {}", e))?;
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
    // а не speed со значением 70
    let (samples, rate) = piper
        .create(
            chunk,
            false,
            None,                    // speaker_id (None = используем дефолтный)
            Some(1.5),              // length_scale - 1.5 = медленнее и плавнее
            Some(0.667),            // noise_scale (стандарт)
            Some(0.8),              // noise_w_scale (стандарт)
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

/// Скачивание модели Piper с Hugging Face
fn download_piper_model(model_name: &str, onnx_path: &Path, config_path: &Path) -> Result<(), String> {
    let base_url = "https://huggingface.co/rhasspy/piper-voices/resolve/main";
    
    // Определяем путь в зависимости от модели
    let (lang, voice, voice_name) = if model_name.starts_with("ru_RU") {
        ("ru", "ru_RU", "dmitri")
    } else if model_name.starts_with("en_US") {
        ("en", "en_US", "lessac")
    } else {
        ("ru", "ru_RU", "dmitri")
    };
    
    let onnx_url = format!("{}/{}/{}/{}/medium/{}.onnx", base_url, lang, voice, voice_name, model_name);
    let config_url = format!("{}/{}/{}/{}/medium/{}.onnx.json", base_url, lang, voice, voice_name, model_name);
    
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