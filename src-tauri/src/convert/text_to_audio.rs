// src-tauri/src/convert/text_to_audio.rs

use std::fs;
use std::path::Path;
use crate::convert::{calculate_conversion_hash, get_app_dir_path_with_hash};
use crate::convert::audio;
use crate::utils::generate_audio;

pub fn convert_text_to_audio(path: &str, from: &str, to: &str) -> Result<String, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("Cannot read file: {}", e))?;

    if text.trim().is_empty() {
        return Err("File is empty".to_string());
    }

    let hash = calculate_conversion_hash(path, from, to)
        .map_err(|e| format!("Hash error convert_text_to_audio: {}", e))?;

    let final_path = get_app_dir_path_with_hash(path, to, &hash, true)?;

    let temp_wav = generate_audio::generate_speech_with_piper(&text)?;
    
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