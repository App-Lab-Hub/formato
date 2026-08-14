// src-tauri/src/convert/audio.rs

use crate::convert::codec::get_audio_codec;
use crate::convert::{calculate_conversion_hash, get_app_dir_path_with_hash};
use ffmpeg_sidecar::command::FfmpegCommand;

pub fn convert_audio_to_audio(path: &str, from: &str, to: &str) -> Result<String, String> {
    if from == to {
        return Ok(path.to_string());
    }

    let hash = calculate_conversion_hash(path, from, to)
        .map_err(|e| format!("Hash error convert_audio_to_audio: {}", e))?;
    let output_path = get_app_dir_path_with_hash(path, to, &hash)?;
    let audio_codec = get_audio_codec(to);

    let mut cmd = FfmpegCommand::new();
    cmd.input(path);

    // Отсекаем треки обложек в аудиофайлах
    cmd.args(&["-vn"]);

    // 🧠 Умный ресемплинг
    if to != "wav" {
        cmd.args(&["-ar", "44100"]);
        cmd.args(&["-ac", "2"]);
    }

    // 🎵 Специальная обработка для Opus (исправление ошибки 234)
    if to == "opus" {
        cmd.args(&["-c:a", "libopus"]);
        cmd.args(&["-b:a", "128k"]); // Ограничиваем битрейт (макс для Opus - 256k)
        cmd.args(&["-ac", "2"]); // Принудительно стерео (решает проблему 5.1)
        cmd.args(&["-ar", "48000"]); // Стандартная частота для Opus
        cmd.args(&["-application", "audio"]); // Оптимально для музыки
        cmd.args(&["-frame_duration", "20"]); // 20ms фреймы (стандарт)
    } else {
        // Фильтр: мягкое ограничение пиков + дизеринг для всех остальных
        cmd.args(&["-af", "aresample=dither_method=triangular"]);
        cmd.args(&["-c:a", audio_codec]);
    }

    cmd.args(&["-y"]);
    cmd.output(&output_path);

    let mut output = cmd
        .spawn()
        .map_err(|e| format!("Failed to spawn ffmpeg: {}", e))?;

    let status = output
        .wait()
        .map_err(|e| format!("Failed to wait for ffmpeg: {}", e))?;

    if !status.success() {
        return Err(format!("FFmpeg conversion failed with status: {}", status));
    }

    Ok(output_path)
}
