// src-tauri/src/convert/codec.rs

/// Получение кодека для аудио по формату
pub fn get_audio_codec(format: &str) -> &'static str {
    match format {
        "mp3" => "libmp3lame",
        "wav" => "pcm_s16le",   // ИЗМЕНЕНО: теперь 32-бит Float стоит по дефолту
        "aac" => "aac",
        "flac" => "flac",
        "ogg" => "libvorbis",
        "m4a" => "aac",
        "opus" => "libopus",
        "webm" => "libopus",
        "flv" => "libmp3lame",
        "mp4" | "mov" | "avi" | "mkv" | "3gp" => "aac",
        _ => "aac",
    }
}


/// Получение кодека для видео по формату
pub fn get_video_codec(format: &str) -> &'static str {
    match format {
        "mp4" => "libx264",
        "mov" => "libx264",
        "mkv" => "libx264",
        "webm" => "libvpx-vp9",
        "avi" => "libx264",     // ИСПРАВЛЕНО: Современный H.264 вместо мертвого Xvid
        "flv" => "libx264",     // ИСПРАВЛЕНО: Современный H.264 вместо старого кодека flv (уберет пиксели)
        "3gp" => "libx264",     // ИСПРАВЛЕНО: Современный H.264 вместо h263 из 2000-х годов
        "mpeg" | "mpg" => "mpeg2video",
        "wmv" => "wmv2",
        _ => "libx264",
    }
}
