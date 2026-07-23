// src-tauri/src/convert/text_to_audio.rs

use std::fs;
use std::path::Path;
use tempfile::Builder;
use kittentts::download;
use crate::convert::{calculate_conversion_hash, get_app_dir_path_with_hash};
// use crate::ffmpeg::init_ffmpeg;
use ffmpeg_sidecar::command::FfmpegCommand;

pub fn convert_text_to_audio(path: &str, from: &str, to: &str) -> Result<String, String> {
    // 1. Читаем файл как простой текст
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("Cannot read file: {}", e))?;

    if text.trim().is_empty() {
        return Err("File is empty".to_string());
    }

    // 2. Убеждаемся, что FFmpeg доступен
    // init_ffmpeg()?;

    // 3. Вычисляем хеш для выходного файла
    let hash = calculate_conversion_hash(path, from, to)
        .map_err(|e| format!("Hash error: {}", e))?;

    let output_path = get_app_dir_path_with_hash(path, to, &hash)?;

    // 4. Генерируем речь через KittenTTS (WAV)
    let temp_wav = generate_speech_with_kittentts(&text)?;
    
    // Добавляем проверку, что WAV файл создан
    if !Path::new(&temp_wav).exists() {
        return Err(format!("WAV file not created: {}", temp_wav));
    }

    // Проверяем размер файла
    let metadata = fs::metadata(&temp_wav)
        .map_err(|e| format!("Cannot get WAV metadata: {}", e))?;
    if metadata.len() == 0 {
        return Err("Generated WAV file is empty".to_string());
    }

    println!("✅ WAV file created: {} ({} bytes)", temp_wav, metadata.len());

    // 5. Конвертируем WAV в целевой аудио формат
    convert_wav_to_audio(&temp_wav, &output_path, to)?;

    // 6. Удаляем временный WAV файл
    let _ = fs::remove_file(&temp_wav);

    Ok(output_path)
}

/// Генерация речи через KittenTTS
fn generate_speech_with_kittentts(text: &str) -> Result<String, String> {
    // Проверяем, что текст не пустой
    if text.trim().is_empty() {
        return Err("Text is empty for TTS generation".to_string());
    }

    // Загружаем модель (скачается при первом запуске и закешируется)
    println!("🔄 Loading TTS model...");
    let tts = download::load_from_hub("KittenML/kitten-tts-nano-0.8-int8")
        .map_err(|e| format!("Failed to load TTS model: {}", e))?;
    println!("✅ TTS model loaded");

    // Создаем временный файл через tempfile Builder
    let temp_file = Builder::new()
        .suffix(".wav")
        .prefix("kittentts_")
        .tempfile()
        .map_err(|e| format!("Cannot create temp file: {}", e))?;
    
    let temp_path = temp_file.path()
        .to_str()
        .ok_or("Invalid temp path")?
        .to_string();

    // Генерируем речь и сохраняем в файл
    println!("🔄 Generating speech...");
    
    tts.generate_to_file(
        text,
        Path::new(&temp_path),
        "Luna",  // Голос (Luna, Jasper, Bruno, Bella)
        1.0,     // Скорость речи (1.0 = нормальная)
        true,    // Предобработка текста (цифры → слова)
    ).map_err(|e| format!("TTS generation failed: {}", e))?;

    println!("✅ Speech generated: {}", temp_path);
    
    // !!! ВАЖНО: Сохраняем файл, чтобы он не удалился !!!
    let _ = temp_file.keep();
    
    Ok(temp_path)
}

/// Конвертация WAV в целевой аудио формат
fn convert_wav_to_audio(input_wav: &str, output_path: &str, to: &str) -> Result<(), String> {
    // Проверяем, что входной файл существует
    if !Path::new(input_wav).exists() {
        return Err(format!("Input WAV file does not exist: {}", input_wav));
    }

    // Проверяем размер
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
    cmd.args(&["-b:a", "192k"]);
    
    if to == "mp3" {
        cmd.args(&["-id3v2_version", "3"]);
        cmd.args(&["-write_id3v1", "1"]);
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

    // Проверяем, что выходной файл создан
    if !Path::new(output_path).exists() {
        return Err("FFmpeg did not create output file".to_string());
    }

    println!("✅ Conversion complete: {}", output_path);
    Ok(())
}