// src/routes/convert/[format]/+page.ts (или +page.server.ts)

export async function entries() {
  return [
    // ============ ТЕКСТОВЫЕ И КОНФИГУРАЦИОННЫЕ ============
    { format: "json" },
    { format: "yaml" },
    { format: "csv" },
    { format: "xml" },
    { format: "toml" },
    { format: "ini" },
    { format: "md" },
    { format: "html" },
    { format: "txt" },
    { format: "rtf" },

    // ============ ДОКУМЕНТЫ ============
    { format: "pdf" },
    { format: "docx" },
    { format: "odt" },
    { format: "xlsx" },

    // ============ ИЗОБРАЖЕНИЯ ============
    { format: "jpg" },
    { format: "jpeg" },
    { format: "png" },
    { format: "webp" },
    { format: "avif" },
    { format: "gif" },
    { format: "bmp" },
    { format: "tiff" },
    { format: "ico" },
    { format: "qoi" },
    { format: "tga" },
    { format: "exr" },
    { format: "hdr" },
    { format: "pnm" },
    { format: "ff" },

    // ============ АУДИО ============
    { format: "mp3" },
    { format: "wav" },
    { format: "aac" },
    { format: "flac" },
    { format: "ogg" },
    { format: "opus" },
    { format: "wma" },
    { format: "m4a" },
    { format: "aiff" },
    { format: "ac3" },
    { format: "eac3" },
    { format: "dts" },
    { format: "tta" },
    { format: "wv" },
    { format: "voc" },
    { format: "adx" },
    { format: "aptx" },
    { format: "sbc" },
    { format: "caf" },
    { format: "w64" },

    // ============ ВИДЕО ============
    { format: "mp4" },
    { format: "mov" },
    { format: "avi" },
    { format: "mkv" },
    { format: "webm" },
    { format: "wmv" },
    { format: "flv" },
    { format: "3gp" },
    { format: "m4v" },
    { format: "ts" },
    { format: "vob" },
    { format: "mpg" },
    { format: "mpeg" },
    { format: "nut" },
  ];
}
