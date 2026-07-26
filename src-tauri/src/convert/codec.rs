// src-tauri/src/convert/codec.rs

/// Получение кодека для аудио по формату
pub fn get_audio_codec(format: &str) -> &'static str {
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

/// Получение кодека для видео по формату
pub fn get_video_codec(format: &str) -> &'static str {
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