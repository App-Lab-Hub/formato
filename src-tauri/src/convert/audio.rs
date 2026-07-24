// src-tauri/src/convert/audio.rs

use ffmpeg_sidecar::command::FfmpegCommand;
use crate::convert::{calculate_conversion_hash, get_app_dir_path_with_hash};

/// Конвертация аудио в аудио
pub fn convert_audio_to_audio(path: &str, from: &str, to: &str) -> Result<String, String> {
    // Если форматы совпадают - просто возвращаем путь
    if from == to {
        return Ok(path.to_string());
    }



    // Вычисляем хеш для выходного файла
    let hash = calculate_conversion_hash(path, from, to)
        .map_err(|e| format!("Hash error: {}", e))?;

    let output_path = get_app_dir_path_with_hash(path, to, &hash)?;

    // Определяем аудио кодек для выходного формата
    let audio_codec = get_audio_codec(to);

    // Строим команду FFmpeg
    let mut cmd = FfmpegCommand::new();
    cmd.input(path);
    cmd.args(&["-c:a", audio_codec]);
    cmd.args(&["-b:a", "192k"]); // Битрейт 192 kbps
    
    // Дополнительные настройки для MP3
    if to == "mp3" {
        cmd.args(&["-id3v2_version", "3"]);
        cmd.args(&["-write_id3v1", "1"]);
    }

    cmd.args(&["-y"]); // Перезаписать выходной файл
    cmd.output(&output_path);

    // Запускаем и ждем завершения
    let mut child = cmd.spawn()
        .map_err(|e| format!("Failed to spawn ffmpeg: {}", e))?;

    let status = child.wait()
        .map_err(|e| format!("Failed to wait for ffmpeg: {}", e))?;

    if !status.success() {
        return Err(format!("FFmpeg conversion failed with status: {}", status));
    }

    Ok(output_path)
}

/// Получение кодека для аудио по формату
fn get_audio_codec(format: &str) -> &'static str {
    match format {
        "mp3" => "libmp3lame",
        "wav" => "pcm_s16le",
        "aac" => "aac",
        "flac" => "flac",
        "ogg" => "libvorbis",
        "m4a" => "aac",
        "opus" => "libopus",
        _ => "aac", // дефолт
    }
}