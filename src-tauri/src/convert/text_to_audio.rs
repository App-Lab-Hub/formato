// src-tauri/src/convert/text_to_audio.rs

use std::fs;
use std::path::Path;
use tempfile::Builder;
use kittentts::download;
use crate::convert::{calculate_conversion_hash, get_app_dir_path_with_hash};
use ffmpeg_sidecar::command::FfmpegCommand;

pub fn convert_text_to_audio(path: &str, from: &str, to: &str) -> Result<String, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("Cannot read file: {}", e))?;

    if text.trim().is_empty() {
        return Err("File is empty".to_string());
    }

    let hash = calculate_conversion_hash(path, from, to)
        .map_err(|e| format!("Hash error: {}", e))?;

    let output_path = get_app_dir_path_with_hash(path, to, &hash)?;

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

    convert_wav_to_audio(&temp_wav, &output_path, to)?;
    let _ = fs::remove_file(&temp_wav);

    Ok(output_path)
}

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
    
    // Попробуйте разные голоса: Luna, Jasper, Bruno, Bella
    tts.generate_to_file(
        text,
        Path::new(&temp_path),
        "Jasper",  // Попробуйте другой голос
        1.0,
        true,
    ).map_err(|e| format!("TTS generation failed: {}", e))?;

    println!("✅ Speech generated: {}", temp_path);
    let _ = temp_file.keep();
    
    Ok(temp_path)
}

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
            cmd.args(&["-q:a", "2"]);  // VBR качество
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

    // Фильтры для улучшения качества и подавления шума
    cmd.args(&["-af", "highpass=f=80, lowpass=f=12000"]);
    
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