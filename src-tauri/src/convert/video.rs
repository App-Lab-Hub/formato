// src-tauri/src/convert/video.rs

use ffmpeg_sidecar::command::FfmpegCommand;
use crate::convert::{calculate_conversion_hash, get_app_dir_path_with_hash};
use crate::convert::audio;
use crate::convert::codec::{get_audio_codec, get_video_codec};
use tempfile::Builder;

/// Конвертация видео в видео
pub fn convert_video_to_video(path: &str, from: &str, to: &str) -> Result<String, String> {
    if from == to {
        return Ok(path.to_string());
    }

    let hash = calculate_conversion_hash(path, from, to)
        .map_err(|e| format!("Hash error convert_video_to_video: {}", e))?;

    let output_path = get_app_dir_path_with_hash(path, to, &hash, true)?;

    let video_codec = get_video_codec(to);
    let audio_codec = get_audio_codec(to);

    let mut cmd = FfmpegCommand::new();
    cmd.input(path);
    
    // 🔧 Исправление ошибки FFmpeg 234: принудительные параметры цвета
    cmd.args(["-colorspace", "bt709"]);
    cmd.args(["-color_primaries", "bt709"]);
    cmd.args(["-color_trc", "bt709"]);
    cmd.args(["-color_range", "pc"]);
    
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

    let mut output = cmd.spawn()
        .map_err(|e| format!("Failed to spawn ffmpeg: {}", e))?;
    
    let status = output.wait()
        .map_err(|e| format!("Failed to wait for ffmpeg: {}", e))?;

    if !status.success() {
        return Err(format!("FFmpeg conversion failed with status: {}", status));
    }

    Ok(output_path)
}

/// Конвертация видео в аудио (извлекает аудио дорожку в WAV, затем использует audio модуль)
pub fn convert_video_to_audio(path: &str, from: &str, to: &str) -> Result<String, String> {
    let temp_wav = extract_audio_to_wav(path)?;
    let audio_output = audio::convert_audio_to_audio(&temp_wav, "wav", to)?;
    
    let hash = calculate_conversion_hash(path, from, to)
        .map_err(|e| format!("Hash error convert_video_to_audio: {}", e))?;
    let final_path = get_app_dir_path_with_hash(path, to, &hash, false)?;
    
    if audio_output != final_path {
        if let Some(parent) = std::path::Path::new(&final_path).parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("Cannot create output dir: {}", e))?;
            }
        }
        std::fs::rename(&audio_output, &final_path)
            .map_err(|e| format!("Cannot move file: {}", e))?;
    }
    
    let _ = std::fs::remove_file(&temp_wav);
    
    Ok(final_path)
}

/// Извлечение аудио дорожки из видео в WAV
fn extract_audio_to_wav(path: &str) -> Result<String, String> {
    let temp_file = Builder::new()
        .suffix(".wav")
        .prefix("video_audio_")
        .tempfile()
        .map_err(|e| format!("Cannot create temp file: {}", e))?;
    
    let temp_path = temp_file.path()
        .to_str()
        .ok_or("Invalid temp path")?
        .to_string();
    
    let mut cmd = FfmpegCommand::new();
    cmd.input(path);
    cmd.args(&["-vn"]);
    cmd.args(&["-acodec", "pcm_s16le"]);
    cmd.args(&["-ar", "22050"]);
    cmd.args(&["-ac", "1"]);
    cmd.args(&["-y"]);
    cmd.output(&temp_path);
    
    let mut output = cmd.spawn()
        .map_err(|e| format!("Failed to spawn ffmpeg: {}", e))?;
    
    let status = output.wait()
        .map_err(|e| format!("Failed to wait for ffmpeg: {}", e))?;
    
    if !status.success() {
        return Err(format!("FFmpeg audio extraction failed with status: {}", status));
    }
    
    if !std::path::Path::new(&temp_path).exists() {
        return Err("WAV file not created".to_string());
    }
    
    let _ = temp_file.keep();
    
    Ok(temp_path)
}