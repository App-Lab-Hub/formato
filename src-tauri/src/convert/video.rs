// src-tauri/src/convert/video.rs

use ffmpeg_sidecar::command::FfmpegCommand;
// use crate::ffmpeg::init_ffmpeg;
use crate::convert::{calculate_conversion_hash, get_app_dir_path_with_hash};

/// Конвертация видео в видео
pub fn convert_video_to_video(path: &str, from: &str, to: &str) -> Result<String, String> {
    if from == to {
        return Ok(path.to_string());
    }

    // init_ffmpeg()?;

    let hash = calculate_conversion_hash(path, from, to)
        .map_err(|e| format!("Hash error: {}", e))?;

    let output_path = get_app_dir_path_with_hash(path, to, &hash)?;

    let video_codec = get_video_codec(to);
    let audio_codec = get_audio_codec(to);

    let mut cmd = FfmpegCommand::new();
    cmd.input(path);
    cmd.args(&["-c:v", video_codec]);
    cmd.args(&["-c:a", audio_codec]);
    
    if to == "mp4" || to == "mov" || to == "mkv" {
        cmd.args(&["-crf", "23"]);
        cmd.args(&["-preset", "medium"]);
    }
    
    if to == "mp4" {
        cmd.args(&["-profile:v", "high"]);
        cmd.args(&["-level", "4.0"]);
        cmd.args(&["-pix_fmt", "yuv420p"]);
    }
    
    if to == "mov" {
        cmd.args(&["-pix_fmt", "yuv420p"]);
    }

    cmd.args(&["-y"]);
    cmd.output(&output_path);

    let mut child = cmd.spawn()
        .map_err(|e| format!("Failed to spawn ffmpeg: {}", e))?;

    let status = child.wait()
        .map_err(|e| format!("Failed to wait for ffmpeg: {}", e))?;

    if !status.success() {
        return Err(format!("FFmpeg conversion failed with status: {}", status));
    }

    Ok(output_path)
}

/// Конвертация видео в аудио (извлекает аудио дорожку и конвертирует в целевой формат)
pub fn convert_video_to_audio(path: &str, from: &str, to: &str) -> Result<String, String> {
    // init_ffmpeg()?;

    // Хеш от from (видео) и to (аудио)
    let hash = calculate_conversion_hash(path, from, to)
        .map_err(|e| format!("Hash error: {}", e))?;

    let output_path = get_app_dir_path_with_hash(path, to, &hash)?;

    // Определяем аудио кодек для выходного формата
    let audio_codec = get_audio_codec(to);

    let mut cmd = FfmpegCommand::new();
    cmd.input(path);
    cmd.args(&["-vn"]);              // Отключаем видео дорожку (извлекаем только аудио)
    cmd.args(&["-c:a", audio_codec]); // Конвертируем аудио в нужный кодек
    cmd.args(&["-b:a", "192k"]);      // Битрейт 192 kbps
    
    // Дополнительные настройки для MP3
    if to == "mp3" {
        cmd.args(&["-id3v2_version", "3"]);
        cmd.args(&["-write_id3v1", "1"]);
    }

    cmd.args(&["-y"]);               // Перезаписать выходной файл
    cmd.output(&output_path);

    let mut child = cmd.spawn()
        .map_err(|e| format!("Failed to spawn ffmpeg: {}", e))?;

    let status = child.wait()
        .map_err(|e| format!("Failed to wait for ffmpeg: {}", e))?;

    if !status.success() {
        return Err(format!("FFmpeg extraction failed with status: {}", status));
    }

    Ok(output_path)
}

/// Получение кодека для видео по формату
fn get_video_codec(format: &str) -> &'static str {
    match format {
        "mp4" => "libx264",
        "mov" => "libx264",
        "avi" => "libxvid",
        "mkv" => "libx264",
        "webm" => "libvpx-vp9",
        "flv" => "flv",
        "mpeg" | "mpg" => "mpeg2video",
        "wmv" => "wmv2",
        "3gp" => "h263",
        _ => "libx264",
    }
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
        "mp4" => "aac",
        "mov" => "aac",
        "avi" => "aac",
        "mkv" => "aac",
        "webm" => "libopus",
        "flv" => "mp3",
        "3gp" => "aac",
        _ => "aac",
    }
}