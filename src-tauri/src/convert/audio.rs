// src-tauri/src/convert/audio.rs

use ffmpeg_sidecar::command::FfmpegCommand;
use crate::convert::{calculate_conversion_hash, get_app_dir_path_with_hash};
use crate::convert::codec::get_audio_codec;

pub fn convert_audio_to_audio(path: &str, from: &str, to: &str) -> Result<String, String> {
    if from == to {
        return Ok(path.to_string());
    }

    let hash = calculate_conversion_hash(path, from, to)
        .map_err(|e| format!("Hash error convert_audio_to_audio: {}", e))?;
    let output_path = get_app_dir_path_with_hash(path, to, &hash, true)?;
    let audio_codec = get_audio_codec(to);

    let mut cmd = FfmpegCommand::new();
    cmd.input(path);
    cmd.args(&["-vn"]);
    
    if to != "wav" {
        cmd.args(&["-ar", "44100"]);
        cmd.args(&["-ac", "2"]);
    }

    cmd.args(&["-af", "aresample=dither_method=triangular"]);
    cmd.args(&["-c:a", audio_codec]);
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