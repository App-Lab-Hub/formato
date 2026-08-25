// src-tauri/src/utils/generate_audio.rs

use crate::paths;
use crate::settings::get_settings;
use std::fs::File;
use std::io::{Seek, SeekFrom, Write};
use std::panic::AssertUnwindSafe;
use tempfile::Builder;
use unicode_segmentation::UnicodeSegmentation;

// Условный импорт для piper-rs (ТОЛЬКО если НЕ flatpak)
#[cfg(not(flatpak))]
use piper_rs::Piper;

// Функция для piper-rs (используется в DEB/RPM)
#[cfg(not(flatpak))]
fn generate_piper_audio(text: &str, onnx_path: &str, config_path: &str) -> Result<Vec<u8>, String> {
    let mut piper = Piper::new(
        &std::path::Path::new(onnx_path),
        &std::path::Path::new(config_path)
    ).map_err(|e| format!("Failed to load Piper model: {}", e))?;

    let (samples, _rate) = piper.create(text, false, None, Some(1.0), None, None)
        .map_err(|e| format!("TTS failed: {}", e))?;

    // Конвертируем f32 в i16
    let mut output = Vec::with_capacity(samples.len() * 2);
    for &s in &samples {
        let sample = (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
        output.extend_from_slice(&sample.to_le_bytes());
    }

    Ok(output)
}

// Функция для бинарника (используется в Flatpak)
#[cfg(flatpak)]
fn generate_piper_audio(text: &str, onnx_path: &str, config_path: &str) -> Result<Vec<u8>, String> {
    use std::io::{Read, Write};
    use std::process::{Command, Stdio};

    let mut child = Command::new("piper")
        .args([
            "--model", onnx_path,
            "--config", config_path,
            "--output_raw",
            "--sentence_silence", "0.5"
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start piper: {}", e))?;

    // Пишем текст в stdin
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(text.as_bytes())
            .map_err(|e| format!("Write to stdin failed: {}", e))?;
    }

    // Читаем аудио из stdout
    let mut output = Vec::new();
    if let Some(mut stdout) = child.stdout.take() {
        stdout.read_to_end(&mut output)
            .map_err(|e| format!("Read from stdout failed: {}", e))?;
    }

    // Проверяем статус
    let status = child.wait()
        .map_err(|e| format!("Wait for child failed: {}", e))?;
    
    if !status.success() {
        let mut stderr = String::new();
        if let Some(mut child_stderr) = child.stderr.take() {
            child_stderr.read_to_string(&mut stderr).ok();
        }
        return Err(format!("Piper exited with error: {}", stderr));
    }

    Ok(output)
}

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

    let cyrillic_count = text
        .chars()
        .filter(|c| ('\u{0400}'..='\u{04FF}').contains(c))
        .count();

    let cyrillic_percent = cyrillic_count as f32 / total_chars as f32 * 100.0;
    println!("🔍 Анализ текста: {}% кириллицы", cyrillic_percent.round());

    if cyrillic_percent >= 51.0 {
        "ru".to_string()
    } else {
        "en".to_string()
    }
}

/// Получение модели для языка из настроек (синхронная версия)
fn get_model_for_language_sync(lang: &str) -> String {
    let settings =
        tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on(get_settings()));

    settings
        .synthesis_model
        .get(lang)
        .cloned()
        .unwrap_or_else(|| match lang {
            "ru" => "ru_RU-dmitri-medium".to_string(),
            _ => "en_US-lessac-medium".to_string(),
        })
}

/// Инициализация пустого WAV-заголовка (заглушка)
fn write_wav_header_placeholder(w: &mut impl Write) -> Result<(), std::io::Error> {
    w.write_all(b"RIFF")?;
    w.write_all(&0u32.to_le_bytes())?; // Место под размер RIFF (36 + data_len)
    w.write_all(b"WAVEfmt ")?;
    w.write_all(&16u32.to_le_bytes())?; // Subchunk1Size
    w.write_all(&1u16.to_le_bytes())?; // AudioFormat (PCM)
    w.write_all(&1u16.to_le_bytes())?; // NumChannels (Mono)
    w.write_all(&0u32.to_le_bytes())?; // Место под SampleRate
    w.write_all(&0u32.to_le_bytes())?; // Место под ByteRate
    w.write_all(&2u16.to_le_bytes())?; // BlockAlign (Channels * BytesPerSample)
    w.write_all(&16u16.to_le_bytes())?; // BitsPerSample
    w.write_all(b"data")?;
    w.write_all(&0u32.to_le_bytes())?; // Место под data_len
    Ok(())
}

/// Перезапись заголовка WAV реальными данными в конце генерации
fn finalize_wav_header(
    file: &mut File,
    total_bytes: u32,
    sample_rate: u32,
) -> Result<(), std::io::Error> {
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

/// Генерация речи через Piper с поточной записью чанков на диск
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

    // Определяем, какую функцию использовать
    #[cfg(not(flatpak))]
    println!("🔄 Использую piper-rs (Rust крейт)");

    #[cfg(flatpak)]
    println!("🔄 Использую системный бинарник piper");

    let temp_file = Builder::new()
        .suffix(".wav")
        .prefix("piper_")
        .tempfile()
        .map_err(|e| format!("Cannot create temp file: {}", e))?;

    let temp_path = temp_file
        .path()
        .to_str()
        .ok_or("Invalid temp path")?
        .to_string();

    // Открываем файл и оборачиваем в буферизированную запись
    let mut file = File::create(&temp_path).map_err(|e| format!("File error: {}", e))?;
    let mut buf_writer = std::io::BufWriter::new(&mut file);

    // Записываем заготовку заголовка
    write_wav_header_placeholder(&mut buf_writer).map_err(|e| format!("Header error: {}", e))?;

    let sample_rate = 22050;
    let mut total_bytes = 0u32;

    for (i, chunk) in chunks.iter().enumerate() {
        println!(
            "🔄 [{}/{}] Синтез части ({} символов)...",
            i + 1,
            total_chunks,
            chunk.len()
        );

        let onnx_path_str = onnx_path.to_str().ok_or("Invalid model path")?;
        let config_path_str = config_path.to_str().ok_or("Invalid config path")?;

        // Используем выбранную функцию
        let pcm_data = generate_piper_audio(chunk, onnx_path_str, config_path_str)?;

        // Записываем PCM данные в WAV файл
        buf_writer.write_all(&pcm_data).map_err(|e| format!("Write PCM error: {}", e))?;
        total_bytes += pcm_data.len() as u32;

        println!(
            "✅ [{}/{}] Чанк успешно записан на диск ({} байт)",
            i + 1,
            total_chunks,
            pcm_data.len()
        );
    }

    // Сбрасываем буфер
    let file = buf_writer
        .into_inner()
        .map_err(|e| format!("Buffer unwrap error: {}", e))?;

    // Финализируем заголовок WAV файла
    finalize_wav_header(file, total_bytes, sample_rate)
        .map_err(|e| format!("Finalize header error: {}", e))?;

    let _ = temp_file.keep();
    let duration_sec = total_bytes as f32 / (sample_rate * 2) as f32;
    println!("✅ Speech generated: {} ({} сек)", temp_path, duration_sec);

    Ok(temp_path)
}

pub async fn generate_speech_with_piper_async(text: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || generate_speech_with_piper(&text))
        .await
        .map_err(|e| format!("Task error: {}", e))?
}
