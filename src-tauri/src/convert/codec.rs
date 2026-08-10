// src-tauri/src/convert/codec.rs

/// Получение кодека для аудио по формату контейнера (Только форматы с полной поддержкой записи/чтения)
pub fn get_audio_codec(format: &str) -> &'static str {
    match format {
        // Стандартные форматы
        "mp3" => "libmp3lame",
        "wav" => "pcm_s16le",
        "aac" | "m4a" | "flv" | "mp4" | "mov" | "mkv" | "3gp" | "m4v" | "ts" => "aac",
        "flac" => "flac",
        "ogg" => "libvorbis",
        "opus" | "webm" => "libopus",
        "wma" => "wmav2",
        "aiff" => "pcm_s16be",
        "ac3" | "vob" => "ac3",
        "eac3" => "eac3",
        "mpg" | "mpeg" => "mp2",
        "avi" => "libmp3lame",
        "caf" | "w64" | "voc" => "pcm_s16le",
        "tta" => "tta",
        "wv" => "wavpack",
        "truehd" => "truehd",

        // ✅ Рабочие специфичные форматы
        "adx" => "adpcm_adx",       
        "aptx" => "aptx",            
        "sbc" => "sbc",              

        _ => "aac",
    }
}

/// Получение кодека для видео по формату контейнера (Только форматы с полной поддержкой записи/чтения)
pub fn get_video_codec(format: &str) -> &'static str {
    match format {
        // Стандартные форматы
        "mp4" | "mov" | "mkv" | "flv" | "3gp" | "m4v" | "ts" | "nut" => "libx264",
        "webm" => "libvpx-vp9",
        "avi" => "mpeg4",
        "mpeg" | "mpg" | "vob" | "wtv" => "mpeg2video",
        "wmv" => "wmv2",
        "gif" => "gif",
        "apng" => "apng",
        "bmp" => "bmp",
        "png" => "png",
        "webp" => "webp",

        _ => "libx264",
    }
}