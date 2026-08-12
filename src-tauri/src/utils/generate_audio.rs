// src-tauri/src/utils/generate_audio.rs

use tempfile::Builder;
use crate::paths;
use piper_rs::Piper;
use hound::{WavSpec, WavWriter};
use crate::settings::get_settings;
use std::panic::AssertUnwindSafe;

/// Разбивает текст на части по предложениям (оптимизированная версия)
fn split_text_into_chunks(text: &str, max_chars: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current_chunk = String::with_capacity(max_chars);
    
    for sentence in text.split_inclusive(['.', '!', '?', '\n']) {
        if current_chunk.len() + sentence.len() > max_chars && !current_chunk.is_empty() {
            chunks.push(std::mem::take(&mut current_chunk));
            current_chunk.reserve(max_chars);
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
    let total_chars = text.chars().filter(|c| !c.is_whitespace()).count();
    if total_chars == 0 {
        return "en".to_string();
    }
    
    let cyrillic_count = text.chars()
        .filter(|c| ('\u{0400}'..='\u{04FF}').contains(c))
        .count();
    
    let cyrillic_percent = cyrillic_count as f32 / total_chars as f32 * 100.0;
    println!("🔍 Анализ текста: {}% кириллицы", cyrillic_percent.round());
    
    if cyrillic_percent >= 51.0 { "ru".to_string() } else { "en".to_string() }
}

/// Получение модели для языка из настроек (синхронная версия)
fn get_model_for_language_sync(lang: &str) -> String {
    // 🔥 Используем текущий Handle вместо создания нового Runtime
    let settings = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current()
            .block_on(get_settings())
    });
    
    settings.synthesis_model
        .get(lang)
        .cloned()
        .unwrap_or_else(|| {
            match lang {
                "ru" => "ru_RU-dmitri-medium".to_string(),
                _ => "en_US-lessac-medium".to_string(),
            }
        })
}

/// Генерация речи через Piper с поточной записью на диск (оптимизированная)
pub fn generate_speech_with_piper(text: &str) -> Result<String, String> {
    if text.trim().is_empty() {
        return Err("Text is empty for TTS generation".to_string());
    }

    let lang = detect_language(text);
    let model_name = get_model_for_language_sync(&lang);
    println!("🌐 Определён язык: {} (модель: {})", lang, model_name);
    
    // 🔥 Уменьшаем размер чанков для экономии памяти
    let max_chars = 300;
    let chunks = split_text_into_chunks(text, max_chars);
    let total_chunks = chunks.len();
    println!("📝 Текст разбит на {} частей (макс. {} символов)", total_chunks, max_chars);
    
    let model_dir = paths::piper_models_dir();
    let onnx_path = model_dir.join(format!("{}.onnx", model_name));
    let config_path = model_dir.join(format!("{}.onnx.json", model_name));
    
    if !onnx_path.exists() {
        return Err(format!("Model file not found: {:?}. Please download it in settings.", onnx_path));
    }
    
    if !config_path.exists() {
        return Err(format!("Model config file not found: {:?}. Please download it in settings.", config_path));
    }
    
    println!("🔄 Загружаем модель Piper ({})...", model_name);
    
    let piper_result = std::panic::catch_unwind(AssertUnwindSafe(|| {
        Piper::new(&onnx_path, &config_path)
    }));
    
    let mut piper = match piper_result {
        Ok(Ok(piper)) => piper,
        Ok(Err(e)) => {
            let err_msg = format!("Failed to load Piper model: {}", e);
            if err_msg.contains("memory") || err_msg.contains("alloc") || err_msg.contains("out of memory") {
                return Err("Not enough memory to load TTS model. Please close other applications and try again.".to_string());
            }
            return Err(err_msg);
        }
        Err(_) => {
            return Err("Fatal error while loading TTS model (possible memory corruption)".to_string());
        }
    };
    
    // 🔥 Создаем временный файл
    let temp_file = Builder::new()
        .suffix(".wav")
        .prefix("piper_")
        .tempfile()
        .map_err(|e| format!("Cannot create temp file: {}", e))?;
    
    let temp_path = temp_file.path()
        .to_str()
        .ok_or("Invalid temp path")?
        .to_string();
    
    let mut sample_rate = 22050;
    let mut writer: Option<WavWriter<std::io::BufWriter<std::fs::File>>> = None;
    let mut total_samples = 0;
    
    // 🔥 Пишем чанки с минимальным выделением памяти
    for (i, chunk) in chunks.iter().enumerate() {
        println!("🔄 [{}/{}] Синтез части ({} символов)...", i + 1, total_chunks, chunk.len());
        
        let synthesis_result = std::panic::catch_unwind(AssertUnwindSafe(|| {
            piper.create(chunk, false, None, Some(1.5), None, None)
        }));
        
        let (samples, rate) = match synthesis_result {
            Ok(Ok(result)) => result,
            Ok(Err(e)) => {
                let err_msg = format!("TTS synthesis failed for chunk {}: {}", i + 1, e);
                if err_msg.contains("memory") || err_msg.contains("alloc") || err_msg.contains("out of memory") {
                    return Err(format!("Not enough memory for chunk {}. Try reducing text length.", i + 1));
                }
                return Err(err_msg);
            }
            Err(_) => {
                return Err(format!("Fatal error during synthesis of chunk {}", i + 1));
            }
        };
        
        if i == 0 {
            sample_rate = rate;
            let spec = WavSpec {
                channels: 1,
                sample_rate,
                bits_per_sample: 16,
                sample_format: hound::SampleFormat::Int,
            };
            writer = Some(WavWriter::create(&temp_path, spec)
                .map_err(|e| format!("Failed to create WAV file: {}", e))?);
        }
        
        // 🔥 Конвертируем с минимальным выделением памяти
        let samples_i16: Vec<i16> = samples
            .iter()
            .map(|&s| ((s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16))
            .collect();
        
        if let Some(w) = writer.as_mut() {
            for &sample in &samples_i16 {
                w.write_sample(sample)
                    .map_err(|e| format!("Failed to write sample: {}", e))?;
            }
        }
        
        total_samples += samples_i16.len();
        let samples_len = samples.len();
        
        // 🔥 Принудительно освобождаем память
        drop(samples);
        drop(samples_i16);
        
        println!("✅ [{}/{}] Готово ({} сэмплов, всего: {})", 
            i + 1, total_chunks, samples_len, total_samples);
    }
    
    // 🔥 Финализируем
    if let Some(w) = writer.take() {
        w.finalize()
            .map_err(|e| format!("Failed to finalize WAV: {}", e))?;
    } else {
        return Err("No audio data generated".to_string());
    }
    
    let _ = temp_file.keep();
    let duration_sec = total_samples as f32 / sample_rate as f32;
    println!("✅ Speech generated: {} ({} сек)", temp_path, duration_sec);
    
    Ok(temp_path)
}

/// Асинхронная обертка для generate_speech_with_piper
pub async fn generate_speech_with_piper_async(text: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        generate_speech_with_piper(&text)
    })
    .await
    .map_err(|e| format!("Task error: {}", e))?
}