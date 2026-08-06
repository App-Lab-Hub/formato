// src-tauri/src/convert/codec.rs

/// Получение кодека для аудио по формату контейнера
pub fn get_audio_codec(format: &str) -> &'static str {
    match format {
        "mp3" => "libmp3lame",
        "wav" => "pcm_s16le",   // 16-bit PCM (Little-Endian)
        "aac" => "aac",         // Родной энкодер FFmpeg
        "flac" => "flac",
        "ogg" => "libvorbis",
        "m4a" => "aac",
        "opus" => "libopus",
        "wma" => "wmav2",       // Windows Media Audio 2
        "aiff" => "pcm_s16be",  // AIFF использует Big-Endian PCM
        "ac3" => "ac3",         // Dolby Digital
        "webm" => "libopus",    // Стандарт для WebM
        "flv" => "aac",         // ✅ AAC гораздо совместимее в FLV, чем MP3
        "avi" => "libmp3lame",  // ✅ AVI плохо дружит с AAC, MP3 — стандарт
        "vob" => "ac3",         // ✅ VOB (DVD) требует AC3 или MP2, но не AAC
        // Контейнеры, где AAC является основным стандартом
        "mp4" | "mov" | "mkv" | "3gp" | "m4v" | "ts" => "aac",
        _ => "aac",             // Безопасный дефолт
    }
}

/// Получение кодека для видео по формату контейнера
pub fn get_video_codec(format: &str) -> &'static str {
    match format {
        // ✅ VP9 — баланс скорости и сжатия для WebM
        // Если критически важен AV1, замените на "libsvtav1", но учтите нагрузку на CPU
        "webm" => "libvpx-vp9",  
        // Популярные контейнеры с отличной поддержкой H.264
        "mp4" | "mov" | "mkv" | "avi" | "flv" | "3gp" | "m4v" | "ts" => "libx264",
        // Специфичные старые форматы
        "mpeg" | "mpg" | "vob" => "mpeg2video", // MPEG-2 для DVD/VOB
        "wmv" => "wmv2",
        _ => "libx264",          // Безопасный дефолт
    }
}