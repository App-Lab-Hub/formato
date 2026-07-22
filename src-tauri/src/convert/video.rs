// src-tauri/src/convert/video.rs

use ffmpeg_sidecar::command::FfmpegCommand;
use crate::utils::init_ffmpeg;
use crate::convert::{calculate_conversion_hash, get_app_dir_path_with_hash};

/// Конвертация видео в видео
pub fn convert_video_to_video(path: &str, from: &str, to: &str) -> Result<String, String> {
    // Если форматы совпадают - просто возвращаем путь
    if from == to {
        return Ok(path.to_string());
    }

    // Убеждаемся, что FFmpeg доступен
    // init_ffmpeg()?;

    // Вычисляем хеш для выходного файла
    let hash = calculate_conversion_hash(path, from, to)
        .map_err(|e| format!("Hash error: {}", e))?;

    let output_path = get_app_dir_path_with_hash(path, to, &hash)?;

    // Определяем кодеки для выходного формата
    let video_codec = get_video_codec(to);
    let audio_codec = get_audio_codec(to);

    // Строим команду FFmpeg
    let mut cmd = FfmpegCommand::new();
    cmd.input(path);
    cmd.args(&["-c:v", video_codec]);
    cmd.args(&["-c:a", audio_codec]);
    
    // Настройки качества для H.264
    if to == "mp4" || to == "mov" || to == "mkv" {
        cmd.args(&["-crf", "23"]);      // Качество (0-51, меньше = лучше)
        cmd.args(&["-preset", "medium"]); // Баланс скорость/качество
    }
    
    // Дополнительные настройки для MP4
    if to == "mp4" {
        cmd.args(&["-profile:v", "high"]);
        cmd.args(&["-level", "4.0"]);
        cmd.args(&["-pix_fmt", "yuv420p"]);
    }
    
    // Для MOV
    if to == "mov" {
        cmd.args(&["-pix_fmt", "yuv420p"]);
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