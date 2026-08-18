// src-tauri/src/convert/text_to_audio.rs

use crate::convert::audio;
use crate::convert::{calculate_conversion_hash, get_app_dir_path_with_hash};
use crate::utils::generate_audio;
use std::fs;
use std::path::Path;

pub async fn convert_text_to_audio(path: &str, from: &str, to: &str) -> Result<String, String> {
    let text = tokio::fs::read_to_string(path)
        .await
        .map_err(|e| format!("Cannot read file: {}", e))?;

    if text.trim().is_empty() {
        return Err("File is empty".to_string());
    }

    let hash = calculate_conversion_hash(path, from, to)
        .map_err(|e| format!("Hash error convert_text_to_audio: {}", e))?;

    let final_path = get_app_dir_path_with_hash(path, to, &hash)?;

    // Генерируем аудио асинхронно
    let temp_wav = generate_audio::generate_speech_with_piper_async(text).await?;

    if !Path::new(&temp_wav).exists() {
        return Err(format!("WAV file not created: {}", temp_wav));
    }

    let metadata =
        fs::metadata(&temp_wav).map_err(|e| format!("Cannot get WAV metadata: {}", e))?;
    if metadata.len() == 0 {
        return Err("Generated WAV file is empty".to_string());
    }

    println!(
        "✅ WAV file created: {} ({} bytes)",
        temp_wav,
        metadata.len()
    );

    // 🔥 Клонируем to перед передачей в spawn_blocking
    let to_clone = to.to_string();
    let temp_wav_clone = temp_wav.clone();

    // Конвертируем WAV в целевой аудио формат через audio::convert_audio_to_audio
    let audio_output = tokio::task::spawn_blocking(move || {
        audio::convert_audio_to_audio(&temp_wav_clone, "wav", &to_clone)
    })
    .await
    .map_err(|e| format!("Task error: {}", e))??;

    // Если файл сохранился не туда - перемещаем
    if audio_output != final_path {
        if let Some(parent) = Path::new(&final_path).parent() {
            if !parent.exists() {
                tokio::fs::create_dir_all(parent)
                    .await
                    .map_err(|e| format!("Cannot create output dir: {}", e))?;
            }
        }
        crate::utils::fs::move_file_async(&audio_output, &final_path)
            .await
            .map_err(|e| format!("Cannot move file: {}", e))?;
    }

    tokio::task::spawn_blocking(move || {
        let _ = fs::remove_file(&temp_wav);
    })
    .await
    .map_err(|e| format!("Task error: {}", e))?;

    println!("✅ Text to audio conversion complete: {}", final_path);
    Ok(final_path)
}