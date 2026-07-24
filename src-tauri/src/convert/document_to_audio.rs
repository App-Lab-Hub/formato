// src-tauri/src/convert/document_to_audio.rs

use std::fs;
use std::path::Path;
use tempfile::Builder;
use kittentts::download;
use crate::convert::{calculate_conversion_hash, get_app_dir_path_with_hash, parse_document};
use crate::convert::audio;

/// Конвертация документа в аудио
pub fn convert_document_to_audio(path: &str, from: &str, to: &str) -> Result<String, String> {
    // 1. Парсим документ в JSON
    let json_value = parse_document(path, from)?;
    
    // 2. Преобразуем JSON в читаемый текст в зависимости от формата
    let text = match from {
        "docx" | "odt" | "pdf" => {
            // Для текстовых документов - берем поле "text"
            if let Some(text) = json_value.get("text").and_then(|v| v.as_str()) {
                text.to_string()
            } else {
                // Если поля "text" нет - используем общий подход
                json_to_speech_text(&json_value, from)?
            }
        }
        "xlsx" => {
            // Для Excel - особый подход
            json_to_speech_text_xlsx(&json_value)?
        }
        _ => {
            // Для остальных - общий подход
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

    // 4. Генерируем речь через KittenTTS (WAV)
    let temp_wav = generate_speech_with_kittentts(&text)?;
    
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
    // Передаем "wav" как исходный формат
    let audio_output = audio::convert_audio_to_audio(&temp_wav, "wav", to)?;
    
    // 6. Если файл сохранился не туда - перемещаем
    if audio_output != final_path {
        // Создаем директорию если нужно
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
    // Структура XLSX: массив объектов с полями "name" и "data"
    // data - это массив объектов с данными строк
    let mut text = String::new();
    text.push_str("Excel document.\n");
    
    if let Some(arr) = json.as_array() {
        for sheet in arr {
            if let Some(sheet_obj) = sheet.as_object() {
                // Имя листа
                if let Some(name) = sheet_obj.get("name").and_then(|v| v.as_str()) {
                    text.push_str(&format!("Sheet {}.\n", name));
                }
                
                // Данные листа
                if let Some(data) = sheet_obj.get("data").and_then(|v| v.as_array()) {
                    if data.is_empty() {
                        text.push_str("  Empty sheet.\n");
                        continue;
                    }
                    
                    // Первая строка - заголовки (если есть)
                    let headers: Vec<String> = if let Some(first_row) = data.first() {
                        if let Some(row_obj) = first_row.as_object() {
                            row_obj.keys().cloned().collect()
                        } else {
                            Vec::new()
                        }
                    } else {
                        Vec::new()
                    };
                    
                    // Озвучиваем как таблицу
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

/// Генерация речи через KittenTTS
fn generate_speech_with_kittentts(text: &str) -> Result<String, String> {
    if text.trim().is_empty() {
        return Err("Text is empty for TTS generation".to_string());
    }

    println!("🔄 Loading TTS model...");
    let tts = download::load_from_hub("KittenML/kitten-tts-nano-0.8")
        .map_err(|e| format!("Failed to load TTS model: {}", e))?;
    println!("✅ TTS model loaded");

    let temp_file = Builder::new()
        .suffix(".wav")
        .prefix("kittentts_")
        .tempfile()
        .map_err(|e| format!("Cannot create temp file: {}", e))?;
    
    let temp_path = temp_file.path()
        .to_str()
        .ok_or("Invalid temp path")?
        .to_string();

    println!("🔄 Generating speech...");
    
    tts.generate_to_file(
        text,
        Path::new(&temp_path),
        "Luna",
        1.0,
        true,
    ).map_err(|e| format!("TTS generation failed: {}", e))?;

    println!("✅ Speech generated: {}", temp_path);
    let _ = temp_file.keep();
    
    Ok(temp_path)
}