// src-tauri/src/utils/generate_audio.rs

use tempfile::Builder;
use crate::paths;
use piper_rs::Piper;
use crate::settings::get_settings;
use std::fs::File;
use std::io::{Write, Seek, SeekFrom};
use std::panic::AssertUnwindSafe;

use unicode_segmentation::UnicodeSegmentation;


fn split_text_into_chunks(text: &str, max_chars: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current_chunk = String::with_capacity(max_chars);
    
    for sentence in text.unicode_sentences() {
        // Если само по себе предложение больше лимита, дробим его жестко
        if sentence.len() > max_chars {
            // Сначала сбрасываем то, что накопилось до этого предложения
            if !current_chunk.is_empty() {
                chunks.push(std::mem::take(&mut current_chunk));
                current_chunk.reserve(max_chars);
            }
            
            // Нарезаем гигантское предложение на куски по max_chars символов
            let mut chars = sentence.chars().peekable();
            while chars.peek().is_some() {
                let chunk_part: String = chars.by_ref().take(max_chars).collect();
                chunks.push(chunk_part);
            }
            continue;
        }

        // Стандартная проверка лимита для нормальных предложений
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


/// Определение языка текста (ru или en)
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

/// Инициализация пустого WAV-заголовка (заглушка)
fn write_wav_header_placeholder(w: &mut impl Write) -> Result<(), std::io::Error> {
    w.write_all(b"RIFF")?;
    w.write_all(&0u32.to_le_bytes())?; // Место под размер RIFF (36 + data_len)
    w.write_all(b"WAVEfmt ")?;
    w.write_all(&16u32.to_le_bytes())?; // Subchunk1Size
    w.write_all(&1u16.to_le_bytes())?;  // AudioFormat (PCM)
    w.write_all(&1u16.to_le_bytes())?;  // NumChannels (Mono)
    w.write_all(&0u32.to_le_bytes())?; // Место под SampleRate
    w.write_all(&0u32.to_le_bytes())?; // Место под ByteRate
    w.write_all(&2u16.to_le_bytes())?;  // BlockAlign (Channels * BytesPerSample)
    w.write_all(&16u16.to_le_bytes())?; // BitsPerSample
    w.write_all(b"data")?;
    w.write_all(&0u32.to_le_bytes())?; // Место под data_len
    Ok(())
}

/// Перезапись заголовка WAV реальными данными в конце генерации
fn finalize_wav_header(file: &mut File, total_bytes: u32, sample_rate: u32) -> Result<(), std::io::Error> {
    let byte_rate = sample_rate * 2; // rate * channels * bytes_per_sample
    
    // Корректируем размер RIFF чанка (offset 4)
    file.seek(SeekFrom::Start(4))?;
    file.write_all(&(36 + total_bytes).to_le_bytes())?;
    
    // Записываем реальный SampleRate (offset 24)
    file.seek(SeekFrom::Start(24))?;
    file.write_all(&sample_rate.to_le_bytes())?;
    
    // Записываем реальный ByteRate (offset 28)
    file.write_all(&byte_rate.to_le_bytes())?;
    
    // Корректируем размер подчанка данных 'data' (offset 40)
    file.seek(SeekFrom::Start(40))?;
    file.write_all(&total_bytes.to_le_bytes())?;
    
    Ok(())
}

/// Генерация речи через Piper с поточной записью чанков на диск без аллокаций памяти
pub fn generate_speech_with_piper(text: &str) -> Result<String, String> {
    if text.trim().is_empty() {
        return Err("Text is empty for TTS generation".to_string());
    }

    let lang = detect_language(text);
    let model_name = get_model_for_language_sync(&lang);
    println!("🌐 Определён язык: {} (модель: {})", lang, model_name);
    
    let max_chars = 300;
    let chunks = split_text_into_chunks(text, max_chars);
    let total_chunks = chunks.len();
    println!("📝 Текст разбит на {} частей", total_chunks);
    
    let model_dir = paths::piper_models_dir();
    let onnx_path = model_dir.join(format!("{}.onnx", model_name));
    let config_path = model_dir.join(format!("{}.onnx.json", model_name));
    
    if !onnx_path.exists() || !config_path.exists() {
        return Err("Model files not found. Please download them in settings.".to_string());
    }
    
    println!("🔄 Загружаем модель Piper...");
    let piper_result = std::panic::catch_unwind(AssertUnwindSafe(|| {
        Piper::new(&onnx_path, &config_path)
    }));
    
    let mut piper = match piper_result {
        Ok(Ok(piper)) => piper,
        Ok(Err(e)) => return Err(format!("Failed to load Piper model: {}", e)),
        Err(_) => return Err("Fatal error while loading TTS model".to_string()),
    };
    
    let temp_file = Builder::new()
        .suffix(".wav")
        .prefix("piper_")
        .tempfile()
        .map_err(|e| format!("Cannot create temp file: {}", e))?;
    
    let temp_path = temp_file.path()
        .to_str()
        .ok_or("Invalid temp path")?
        .to_string();
    
    // Открываем файл и оборачиваем в буферизированную запись для оптимизации I/O операций
    let mut file = File::create(&temp_path).map_err(|e| format!("File error: {}", e))?;
    let mut buf_writer = std::io::BufWriter::new(&mut file);
    
    // Записываем заготовку заголовка
    write_wav_header_placeholder(&mut buf_writer).map_err(|e| format!("Header error: {}", e))?;
    
    let mut sample_rate = 22050;
    let mut total_bytes = 0u32;
    
    for (i, chunk) in chunks.iter().enumerate() {
        println!("🔄 [{}/{}] Синтез части ({} символов)...", i + 1, total_chunks, chunk.len());
        
        let synthesis_result = std::panic::catch_unwind(AssertUnwindSafe(|| {
            piper.create(chunk, false, None, Some(1.0), None, None)
        }));
        
        let (samples, rate) = match synthesis_result {
            Ok(Ok(result)) => result,
            Ok(Err(e)) => return Err(format!("TTS failed for chunk {}: {}", i + 1, e)),
            Err(_) => return Err(format!("Fatal error during synthesis of chunk {}", i + 1)),
        };
        
        if i == 0 {
            sample_rate = rate;
        }
        
        // Поточно обрабатываем f32 сэмплы из памяти, конвертируя их в байты i16 на лету
        for &s in &samples {
            let sample_i16 = (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
            buf_writer.write_all(&sample_i16.to_le_bytes())
                .map_err(|e| format!("Write sample error: {}", e))?;
            total_bytes += 2; // 2 байта на один сэмпл i16
        }
        
        // Мгновенное принудительное уничтожение вектора сэмплов текущего чанка
        drop(samples);
        println!("✅ [{}/{}] Чанк успешно записан на диск.", i + 1, total_chunks);
    }
    
    // 🔥 Уничтожаем буфер, сбрасывая данные на диск и возвращая владение над `file`
    let mut file = buf_writer.into_inner().map_err(|e| format!("Buffer unwrap error: {}", e))?;
    
    // Финализируем заголовок WAV файла, вписывая итоговые размеры
    finalize_wav_header(file, total_bytes, sample_rate).map_err(|e| format!("Finalize header error: {}", e))?;
    
    let _ = temp_file.keep();
    let duration_sec = (total_bytes / 2) as f32 / sample_rate as f32;
    println!("✅ Speech generated: {} ({} сек)", temp_path, duration_sec);
    
    Ok(temp_path)
}


pub async fn generate_speech_with_piper_async(text: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        generate_speech_with_piper(&text)
    })
    .await
    .map_err(|e| format!("Task error: {}", e))?
}
