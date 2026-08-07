// src-tauri/src/convert/codec.rs

/// Получение кодека для аудио по формату контейнера (Только форматы с полной поддержкой записи/чтения)
pub fn get_audio_codec(format: &str) -> &'static str {
    match format {
        // Стандартные форматы
        "mp3" => "libmp3lame",
        "wav" => "pcm_s16le",
        "aac" => "aac",
        "flac" => "flac",
        "ogg" => "libvorbis",
        "m4a" => "aac",
        "opus" => "libopus",     // ИСПРАВЛЕНО: Для кодирования FFmpeg требует libopus
        "wma" => "wmav2",
        "aiff" => "pcm_s16be",
        "ac3" => "ac3",
        "webm" => "libopus",     // ИСПРАВЛЕНО: Для WebM также используем libopus
        "flv" => "aac",
        "avi" => "libmp3lame",   // Классическая аудио-дорожка для AVI
        "vob" => "ac3",
        "mpg" | "mpeg" => "mp2",
        "mp4" | "mov" | "mkv" | "3gp" | "m4v" | "ts" => "aac",

        // Новые форматы с полной поддержкой двусторонней конвертации
        "voc" => "pcm_s16le",     // Creative Voice полностью поддерживается на запись и чтение
        "wv" => "wavpack",       // WavPack имеет нативный рабочий энкодер
        "roq" => "roq_dpcm",     // id Software RoQ аудио можно свободно кодировать в FFmpeg

        // Профессиональные монтажные форматы
        "mxf" => "pcm_s16le",     
        "gxf" => "pcm_s16le",     
        "prores" => "pcm_s16le",  
        "dnxhd" => "pcm_s16le",   

        _ => "aac",
    }
}

/// Получение кодека для видео по формату контейнера (Только форматы с полной поддержкой записи/чтения)
pub fn get_video_codec(format: &str) -> &'static str {
    match format {
        // Стандартные форматы
        "webm" => "libvpx-vp9",
        "avi" => "mpeg4",        // Вынесено отдельно: H.264 в AVI ломает структуру файла. mpeg4 — стандарт.
        "mp4" | "mov" | "mkv" | "flv" | "3gp" | "m4v" | "ts" => "libx264",
        "mpeg" | "mpg" | "vob" => "mpeg2video",
        "wmv" => "wmv2",

        // Новые форматы с полной поддержкой двусторонней конвертации
        "roq" => "roqvideo",     // Исправлено: имя энкодера в FFmpeg — roqvideo

        // Профессиональные монтажные форматы
        "mxf" => "mpeg2video",   
        "gxf" => "mpeg2video",   
        "prores" => "prores_ks", // prores_ks — отличный нативный энкодер ProRes в FFmpeg
        "dnxhd" => "dnxhd",      // Родной энкодер для Avid DNxHD

        _ => "libx264",
    }
}
