// src-tauri/src/convert/text_to_audio.rs

use std::fs;
use std::path::Path;
use tempfile::Builder;
use kittentts::download;
use crate::convert::{calculate_conversion_hash, get_app_dir_path_with_hash};
use crate::convert::audio;

pub fn convert_text_to_audio(path: &str, from: &str, to: &str) -> Result<String, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("Cannot read file: {}", e))?;

    if text.trim().is_empty() {
        return Err("File is empty".to_string());
    }

    let hash = calculate_conversion_hash(path, from, to)
        .map_err(|e| format!("Hash error: {}", e))?;

    let final_path = get_app_dir_path_with_hash(path, to, &hash)?;

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
        "Jasper",
        1.0,
        true,
    ).map_err(|e| format!("TTS generation failed: {}", e))?;

    println!("✅ Speech generated: {}", temp_path);
    let _ = temp_file.keep();
    
    Ok(temp_path)
}