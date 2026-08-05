// src-tauri/src/utils/generate_audio.rs
use tempfile::Builder;
use crate::paths;
use piper_rs::Piper;
use hound::{WavSpec, WavWriter};
use crate::settings::get_settings;
use std::panic::AssertUnwindSafe;

/// Разбивает текст на части по предложениям
fn split_text_into_chunks(text: &str, max_chars: usize) -> Vec<String> {
    let sentences: Vec<&str> = text
        .split_inclusive(['.', '!', '?', '\n'])
        .collect();
    
    let mut chunks = Vec::new();
    let mut current_chunk = String::new();
    
    for sentence in sentences {
        if current_chunk.len() + sentence.len() > max_chars
            && !current_chunk.is_empty() {
                chunks.push(current_chunk.clone());
                current_chunk.clear();
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

/// Получение модели для языка из настроек (синхронная версия)
fn get_model_for_language(lang: &str) -> String {
    // Блокирующий вызов для синхронной функции
    let settings = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current()
            .block_on(async { get_settings().await })
    });
    
    let synthesis_map = settings.synthesis_model;
    
    match lang {
        "ru" => synthesis_map.get("ru").cloned().unwrap_or("ru_RU-dmitri-medium".to_string()),
        "en" => synthesis_map.get("en").cloned().unwrap_or("en_US-lessac-medium".to_string()),
        _ => synthesis_map.get("en").cloned().unwrap_or("en_US-lessac-medium".to_string()),
    }
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
    let model_dir = paths::piper_models_dir();
    
    let onnx_path = model_dir.join(format!("{}.onnx", model_name));
    let config_path = model_dir.join(format!("{}.onnx.json", model_name));
    
    // Проверяем наличие модели
    if !onnx_path.exists() {
        return Err(format!("Model file not found: {:?}. Please download it in settings.", onnx_path));
    }
    
    if !config_path.exists() {
        return Err(format!("Model config file not found: {:?}. Please download it in settings.", config_path));
    }
    
    println!("🔄 Загружаем модель Piper ({})...", model_name);
    
    // Загружаем модель с обработкой ошибок памяти
    // Используем AssertUnwindSafe для обхода ограничений UnwindSafe
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
    
    // Разбиваем текст на чанки (по 1000 символов для уменьшения нагрузки на память)
    let max_chars = 1000;
    let chunks = split_text_into_chunks(text, max_chars);
    let total_chunks = chunks.len();
    
    println!("📝 Текст разбит на {} частей (макс. {} символов)", total_chunks, max_chars);
    
    // Создаем временный файл для результата
    let temp_file = match Builder::new()
        .suffix(".wav")
        .prefix("piper_")
        .tempfile() {
            Ok(file) => file,
            Err(e) => return Err(format!("Cannot create temp file: {}", e))
        };
    
    let temp_path = match temp_file.path().to_str() {
        Some(path) => path.to_string(),
        None => return Err("Invalid temp path".to_string())
    };
    
    // Первый чанк определяет sample_rate
    let mut sample_rate = 22050;
    let mut all_samples: Vec<i16> = Vec::new();
    
    // Ограничиваем общее количество сэмплов для предотвращения переполнения памяти
    const MAX_TOTAL_SAMPLES: usize = 10_000_000; // ~7.5 минут при 22kHz
    
    for (i, chunk) in chunks.iter().enumerate() {
        println!("🔄 [{}/{}] Синтез части ({} символов)...", i + 1, total_chunks, chunk.len());
        
        // Проверяем, не превышен ли лимит памяти
        if all_samples.len() > MAX_TOTAL_SAMPLES {
            return Err(format!(
                "Generated audio is too long ({} samples). Maximum allowed is {} samples (~7.5 minutes).",
                all_samples.len(),
                MAX_TOTAL_SAMPLES
            ));
        }
        
        // Синтезируем с обработкой ошибок памяти
        // Используем AssertUnwindSafe для обхода ограничений UnwindSafe
        let synthesis_result = std::panic::catch_unwind(AssertUnwindSafe(|| {
            piper.create(
                chunk,
                false,
                None,                    // speaker_id (None = используем дефолтный)
                Some(1.5),              // length_scale - 1.5 = медленнее и плавнее
                None,
                None,
            )
        }));
        
        let (samples, rate) = match synthesis_result {
            Ok(Ok(result)) => result,
            Ok(Err(e)) => {
                let err_msg = format!("TTS synthesis failed for chunk {}: {}", i + 1, e);
                if err_msg.contains("memory") || err_msg.contains("alloc") || err_msg.contains("out of memory") {
                    return Err(format!("Not enough memory to synthesize audio chunk {}. Please reduce text length or close other applications.", i + 1));
                }
                return Err(err_msg);
            }
            Err(_) => {
                return Err(format!("Fatal error during synthesis of chunk {} (possible memory corruption)", i + 1));
            }
        };
        
        if i == 0 {
            sample_rate = rate;
        }
        
        // Конвертируем f32 в i16 с проверкой на переполнение
        let samples_i16: Vec<i16> = samples
            .iter()
            .map(|&s| {
                let clamped = s.clamp(-1.0, 1.0);
                (clamped * i16::MAX as f32) as i16
            })
            .collect();
        
        // Проверяем, не приведет ли добавление к переполнению памяти
        if all_samples.len() + samples_i16.len() > MAX_TOTAL_SAMPLES {
            return Err(format!(
                "Generated audio would exceed maximum length ({} samples). Current: {}, new: {}",
                MAX_TOTAL_SAMPLES,
                all_samples.len(),
                samples_i16.len()
            ));
        }
        let samples_i16_len = samples_i16.len();
        all_samples.extend(samples_i16);
        
        // Принудительно очищаем память после каждого чанка
        std::mem::drop(samples);
        
        println!("✅ [{}/{}] Готово ({} сэмплов, всего: {})", 
            i + 1, total_chunks, samples_i16_len, all_samples.len());
    }
    
    // Если ничего не сгенерировалось
    if all_samples.is_empty() {
        return Err("No audio samples generated. Text might be empty or unsupported.".to_string());
    }
    
    println!("🔄 Сохраняем объединенное аудио ({} сэмплов)...", all_samples.len());
    
    // Сохраняем все сэмплы в один WAV с обработкой ошибок
    let save_result = std::panic::catch_unwind(AssertUnwindSafe(|| {
        let spec = WavSpec {
            channels: 1,
            sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        
        // Создаем writer с явной обработкой ошибок
        let mut writer = match WavWriter::create(&temp_path, spec) {
            Ok(w) => w,
            Err(e) => return Err(format!("Failed to create WAV file: {}", e))
        };
        
        // Записываем сэмплы с явной обработкой ошибок
        for sample in all_samples.iter() {
            if let Err(e) = writer.write_sample(*sample) {
                return Err(format!("Failed to write sample: {}", e));
            }
        }
        
        // Финализируем с явной обработкой ошибок
        if let Err(e) = writer.finalize() {
            return Err(format!("Failed to finalize WAV: {}", e));
        }
        
        Ok::<_, String>(())
    }));
    
    match save_result {
        Ok(Ok(())) => {
            // Успешно сохранено
        }
        Ok(Err(e)) => {
            return Err(format!("Failed to save WAV file: {}", e));
        }
        Err(_) => {
            return Err("Fatal error while saving WAV file (possible memory corruption)".to_string());
        }
    }
    
    let _ = temp_file.keep();
    
    let duration_sec = all_samples.len() as f32 / sample_rate as f32;
    println!("✅ Speech generated: {} ({} сек)", temp_path, duration_sec);
    
    Ok(temp_path)
}