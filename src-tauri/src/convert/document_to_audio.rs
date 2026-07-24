// src-tauri/src/convert/document_to_audio.rs

use std::fs;
use std::path::Path;
use tempfile::Builder;
use kittentts::download;
use crate::convert::{calculate_conversion_hash, get_app_dir_path_with_hash, parse_document};
// use crate::ffmpeg::init_ffmpeg;
use ffmpeg_sidecar::command::FfmpegCommand;

/// Конвертация документа в аудио
pub fn convert_document_to_audio(path: &str, from: &str, to: &str) -> Result<String, String> {
    // 1. Парсим документ в JSON
    let json_value = parse_document(path, from)?;
    
    // 2. Преобразуем JSON в читаемый текст
    let text = json_to_speech_text(&json_value, from)?;
    
    if text.trim().is_empty() {
        return Err("No readable text found in document".to_string());
    }

    // 3. Убеждаемся, что FFmpeg доступен
    // init_ffmpeg()?;

    // 4. Вычисляем хеш для выходного файла
    let hash = calculate_conversion_hash(path, from, to)
        .map_err(|e| format!("Hash error: {}", e))?;

    let output_path = get_app_dir_path_with_hash(path, to, &hash)?;

    // 5. Генерируем речь через KittenTTS (WAV)
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

    // 6. Конвертируем WAV в целевой аудио формат
    convert_wav_to_audio(&temp_wav, &output_path, to)?;

    // 7. Удаляем временный WAV файл
    let _ = fs::remove_file(&temp_wav);

    Ok(output_path)
}

/// Преобразование JSON в читаемый текст для озвучивания
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
    // Используем модель nano без квантования (лучше качество)
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
        "Luna",  // Голос (Luna, Jasper, Bruno, Bella)
        1.0,     // Скорость речи (1.0 = нормальная)
        true,    // Предобработка текста (цифры → слова)
    ).map_err(|e| format!("TTS generation failed: {}", e))?;

    println!("✅ Speech generated: {}", temp_path);
    let _ = temp_file.keep();
    
    Ok(temp_path)
}

/// Конвертация WAV в целевой аудио формат
fn convert_wav_to_audio(input_wav: &str, output_path: &str, to: &str) -> Result<(), String> {
    if !Path::new(input_wav).exists() {
        return Err(format!("Input WAV file does not exist: {}", input_wav));
    }

    let metadata = fs::metadata(input_wav)
        .map_err(|e| format!("Cannot get input WAV metadata: {}", e))?;
    if metadata.len() == 0 {
        return Err("Input WAV file is empty".to_string());
    }

    let audio_codec = match to {
        "mp3" => "libmp3lame",
        "wav" => "pcm_s16le",
        "aac" => "aac",
        "flac" => "flac",
        "ogg" => "libvorbis",
        "m4a" => "aac",
        "opus" => "libopus",
        _ => "aac",
    };

    println!("🔄 Converting WAV to {}...", to);

    let mut cmd = FfmpegCommand::new();
    cmd.input(input_wav);
    cmd.args(&["-c:a", audio_codec]);
    
    // Улучшенные настройки
    match to {
        "mp3" => {
            cmd.args(&["-q:a", "2"]);
            cmd.args(&["-id3v2_version", "3"]);
            cmd.args(&["-write_id3v1", "1"]);
        }
        "aac" | "m4a" => {
            cmd.args(&["-q:a", "2"]);
        }
        "ogg" => {
            cmd.args(&["-q:a", "6"]);
        }
        "flac" => {
            cmd.args(&["-compression_level", "8"]);
        }
        _ => {
            cmd.args(&["-b:a", "192k"]);
        }
    }

    cmd.args(&["-y"]);
    cmd.output(output_path);

    let mut child = cmd.spawn()
        .map_err(|e| format!("Failed to spawn ffmpeg: {}", e))?;

    let status = child.wait()
        .map_err(|e| format!("Failed to wait for ffmpeg: {}", e))?;

    if !status.success() {
        return Err(format!("FFmpeg conversion failed with status: {}", status));
    }

    if !Path::new(output_path).exists() {
        return Err("FFmpeg did not create output file".to_string());
    }

    println!("✅ Conversion complete: {}", output_path);
    Ok(())
}