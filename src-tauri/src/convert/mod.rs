// src/convert/mod.rs

mod csv;
mod xml;
mod ini;
mod md;
mod txt;
mod rtf;
mod pdf;
mod docx;
mod odt;
mod xlsx;
mod local_utils;
mod audio;
mod video;
mod text_to_audio;
mod document_to_audio;
mod image_to_text; 
pub mod codec; 
mod image_utils; 
mod image_to_document; 
mod audio_to_text;
// mod video_to_text;
use sea_orm::DatabaseConnection;

use crate::AppState;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::Read;
use xxhash_rust::xxh3::Xxh3;
use serde_json::{Value as Json};
use crate::convert::csv::{parse_csv, stringify_csv};
use crate::convert::ini::{parse_ini, stringify_ini};
use crate::convert::md::{parse_markdown, stringify_markdown};
use crate::convert::xml::{parse_xml, stringify_xml};
use crate::convert::txt::{parse_txt, stringify_txt};
use crate::convert::rtf::{parse_rtf};
use crate::convert::pdf::{stringify_pdf, parse_pdf};
// use crate::convert::pdf::{stringify_pdf};

use crate::convert::docx::{stringify_docx, parse_docx};
use crate::convert::odt::{stringify_odt, parse_odt};
use crate::convert::xlsx::{stringify_xlsx, parse_xlsx};

use local_utils::{convert_with_soffice_explicit};

use crate::db;
use crate::html_convert::{convert_to_html, parse_html};
use crate::paths::converted_dir;
use memmap2::Mmap;

// ============================================================
// ТИПЫ
// ============================================================
pub enum ConversionOutput {
    Inline(String),    // Содержимое (строка)
    Save(String),      // Путь к файлу
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentType {
    Text,
    Image,
    Audio,
    Video,
    Document,
}

impl From<String> for ContentType {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "text" => ContentType::Text,
            "image" => ContentType::Image,
            "audio" => ContentType::Audio,
            "video" => ContentType::Video,
            "document" => ContentType::Document,
            _ => ContentType::Text,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ConvertResult {
    pub success: bool,
    pub content: String,
    pub hash: Option<String>,
    pub extension: Option<String>,
    pub error: Option<String>,
}

// ============================================================
// ОСНОВНАЯ ЛОГИКА КОНВЕРТАЦИИ
// ============================================================
/// Проверяет, есть ли файл в кеше БД
async fn is_file_cached(
    db: &DatabaseConnection,
    path: &str,
    from: &str,
    to: &str,
) -> Result<bool, String> {
    let hash = calculate_conversion_hash(path, from, to)
        .map_err(|e| format!("Hash error is_file_cached: {}", e))?;
    println!("HASH=>{}",hash);
    let a = db::find_conversion(db, &hash).await;
    println!("FIND=>{:?}",a);
    println!("Is_Some=>{}",db::find_conversion(db, &hash).await.is_some());

    Ok(db::find_conversion(db, &hash).await.is_some())
}
pub async fn convert(
    db: &DatabaseConnection,
    path: &str,
    from: &str,
    to: &str,
    from_type: &str,
    to_type: &str,
) -> Result<String, String> {
    let from_type: ContentType = from_type.to_string().into();
    let to_type: ContentType = to_type.to_string().into();
    
    match (from_type, to_type) {
        // Text → Text — всегда сохраняем в файл
        (ContentType::Text, ContentType::Text) => {
            convert_text_to_text(db, path, from, to).await
        }
        
        // Text → Document
        (ContentType::Text, ContentType::Document) => {
            convert_text_to_document(db, path, from, to).await
        }
        
        // Text → Audio
        (ContentType::Text, ContentType::Audio) => {
            convert_text_to_audio(path, from, to).await
        }

        // Document → Text
        (ContentType::Document, ContentType::Text) => {
            convert_document_to_text(db,path, from, to).await
        }
        
        // Document → Document
        (ContentType::Document, ContentType::Document) => {
            convert_document_to_document(db,path, from, to).await
        }

        // Document → Audio
        (ContentType::Document, ContentType::Audio) => {
            convert_document_to_audio(path, from, to).await
        }
        
        // Image → Image
        (ContentType::Image, ContentType::Image) => {
            convert_image_to_image(path, from, to).await
        }

        // Image → Text
        (ContentType::Image, ContentType::Text) => {
            convert_image_to_text(path, from, to).await
        }
        (ContentType::Image, ContentType::Document) => {
            convert_image_to_document(db, path, from, to).await
        }
        
        // Audio → Audio
        (ContentType::Audio, ContentType::Audio) => {
            convert_audio_to_audio(path, from, to).await
        }

        // Audio → Text
        (ContentType::Audio, ContentType::Text) => {
            convert_audio_to_text(db, path, from, to).await
        }

        // Audio → Document
        (ContentType::Audio, ContentType::Document) => {
            convert_audio_to_document(db, path, from, to).await
        }
        // Video → Video
        (ContentType::Video, ContentType::Video) => {
            convert_video_to_video(path, from, to).await
        }

        // Video → Audio
        (ContentType::Video, ContentType::Audio) => {
            convert_video_to_audio(path, from, to).await
        }
        // Video → Text (извлекаем аудио, потом распознаем)
        (ContentType::Video, ContentType::Text) => {
            convert_video_to_text(db,path, from, to).await
        }
        
        // Video → Document
        (ContentType::Video, ContentType::Document) => {
            convert_video_to_document(db, path, from, to).await
        }
        
        _ => Err(format!(
            "Conversion from {:?} to {:?} is not supported yet",
            from_type, to_type
        )),
    }
}

// ============================================================
// КОНКРЕТНЫЕ РЕАЛИЗАЦИИ
// ============================================================

async fn convert_text_to_text(db: &DatabaseConnection, path: &str, from: &str, to: &str) -> Result<String, String> {
    let input = std::fs::read_to_string(path)
        .map_err(|e| format!("Cannot read file: {e}"))?;

    // Если конвертируем в RTF - используем stringify_rtf напрямую
    if to == "rtf" {
        // RTF требует DOCX как промежуточный формат
        // Сначала создаем DOCX из текста
        let docx_path = stringify_document(db, &input, path, from, "docx").await?;
        
        // Затем конвертируем DOCX в RTF
        let rtf_path = rtf::convert_docx_to_rtf(&docx_path, path, to)?;
        
        // Проверяем кеш перед удалением
        // if !is_file_cached(db,  path, from, "docx").await? {
        //     let _ = std::fs::remove_file(&docx_path);
        // }
        
        return Ok(rtf_path);
    }

    // Для остальных форматов - стандартная логика
    let value = parse(&input, from)?;
    stringify(&value, to, path, from)
}

fn parse_document(path: &str, from: &str) -> Result<Json, String> {
    match from {
        "docx" => {
            parse_docx(path)
        }
        "pdf" => {
            parse_pdf(path)
        }
        "odt" => {
            parse_odt(path)
        }
        "xlsx" => {
            parse_xlsx(path)
        }
        _ => {
            Err(format!("Unsupported document format: {}", from))
        }
    }
}

// src-tauri/src/convert/mod.rs

/// Сериализует текст напрямую в документ
pub async fn stringify_document(
    db: &DatabaseConnection,
    text: &str,
    path: &str,
    from: &str,
    to: &str,
) -> Result<String, String> {
    match to {
        "docx" => stringify_docx(text, path, from, to),
        "pdf" => stringify_pdf(db, text, path, from, to).await,
        "xlsx" => stringify_xlsx(text, path, from, to),
        "odt" => stringify_odt(text, path, from, to),
        _ => {
            let hash = calculate_conversion_hash(path, from, to)
                .map_err(|e| format!("Hash error stringify_document: {}", e))?;
            let output_path = save_to_app_dir(text, path, to, &hash)?;
            Ok(output_path)
        }
    }
}

async fn convert_document_to_text(
    db: &DatabaseConnection,
    path: &str, 
    from: &str, 
    to: &str
) -> Result<String, String> {
    if to == "rtf" {
        let docx_path = convert_document_to_document(db, path, from, "docx").await?;
        let rtf_path = rtf::convert_docx_to_rtf(&docx_path, path, to)?;
        
        // Проверяем кеш перед удалением
        // if !is_file_cached(db,  path, from, "docx").await? {
        //     let _ = std::fs::remove_file(&docx_path);
        // }
        
        return Ok(rtf_path);
    }
    
    let json_value = parse_document(path, from)?;
    stringify(&json_value, to, path, from)
}




// Функция-обертка (уже есть в вашем коде)
async fn convert_text_to_audio(path: &str, from: &str, to: &str) -> Result<String, String> {
    text_to_audio::convert_text_to_audio(path, from, to)
}



/// Text → Document
async fn convert_text_to_document(
    db: &DatabaseConnection,
    path: &str, 
    from: &str, 
    to: &str
) -> Result<String, String> {
    let input = std::fs::read_to_string(path)
        .map_err(|e| format!("Cannot read file: {e}"))?;
    
    stringify_document(db, &input, path, from, to).await
}

// Функция-обертка:
async fn convert_image_to_text(path: &str, from: &str, to: &str) -> Result<String, String> {
    image_to_text::convert_image_to_text(path, from, to)
}
async fn convert_image_to_document(db: &DatabaseConnection, path: &str, from: &str, to: &str) -> Result<String, String> {
    image_to_document::convert_image_to_document(db, path, from, to).await
}
// ========================================================================================================================
// ========================================================================================================================
// ========================================================================================================================

/// Document → Document
pub async fn convert_document_to_document(
    db: &DatabaseConnection, 
    path: &str, 
    from: &str, 
    to: &str
) -> Result<String, String> {
    // Если форматы совпадают – просто возвращаем исходный путь
    if from == to {
        return Ok(path.to_string());
    }

    // Вычисляем хеш для именования выходного файла
    let hash = calculate_conversion_hash(path, from, to)
        .map_err(|e| format!("Hash error convert_document_to_document: {e}"))?;

    // Вспомогательная функция для генерации пути
    let out_path = |ext: &str| -> Result<String, String> {
        get_app_dir_path_with_hash(path, ext, &hash, true)
    };

    match (from, to) {
        // ---------- DOCX ----------
        ("docx", "pdf") => {
            let out = out_path("pdf")?;
            convert_with_soffice_explicit(path, &out)?;
            Ok(out)
        }
        ("docx", "odt") => {
            let out = out_path("odt")?;
            convert_with_soffice_explicit(path, &out)?;
            Ok(out)
        }
        ("docx", "xlsx") => {
            let out = out_path("xlsx")?;
            let doc = office_oxide::Document::open(path)
                .map_err(|e| format!("Open DOCX: {}", e))?;
            doc.save_as(&out)
                .map_err(|e| format!("Save as XLSX: {}", e))?;
            Ok(out)
        }

        // ---------- ODT ----------
        ("odt", "pdf") => {
            let out = out_path("pdf")?;
            convert_with_soffice_explicit(path, &out)?;
            Ok(out)
        }
        ("odt", "docx") => {
            let out = out_path("docx")?;
            convert_with_soffice_explicit(path, &out)?;
            Ok(out)
        }
        ("odt", "xlsx") => {
            let out = out_path("xlsx")?;
            
            let docx_path = out_path("docx")?;
            convert_with_soffice_explicit(path, &docx_path)?;
            
            let doc = office_oxide::Document::open(&docx_path)
                .map_err(|e| format!("Open DOCX: {}", e))?;
            doc.save_as(&out)
                .map_err(|e| format!("Save as XLSX: {}", e))?;
            
            // Проверяем кеш перед удалением
            // if !is_file_cached(db, path, "odt", "docx").await? {
            //     let _ = std::fs::remove_file(&docx_path);
            // }
            
            Ok(out)
        }

        // ---------- XLSX ----------
        ("xlsx", "docx") => {
            let out = out_path("docx")?;
            let doc = office_oxide::Document::open(path)
                .map_err(|e| format!("Open XLSX: {}", e))?;
            doc.save_as(&out)
                .map_err(|e| format!("Save as DOCX: {}", e))?;
            Ok(out)
        }
        ("xlsx", "odt") => {
            let out = out_path("odt")?;
            
            let docx_path = out_path("docx")?;
            let doc = office_oxide::Document::open(path)
                .map_err(|e| format!("Open XLSX: {}", e))?;
            doc.save_as(&docx_path)
                .map_err(|e| format!("XLSX to DOCX: {}", e))?;
            
            convert_with_soffice_explicit(&docx_path, &out)?;
            
            // Проверяем кеш перед удалением
            // if !is_file_cached(db, path, "xlsx", "docx").await? {
            //     let _ = std::fs::remove_file(&docx_path);
            // }
            
            Ok(out)
        }
        ("xlsx", "pdf") => {
            let out = out_path("pdf")?;
            convert_with_soffice_explicit(path, &out)?;
            Ok(out)
        }
        _ => {
            Err("Unsupported conversion".to_string())
        }
    }
}



// ========================================================================================================================
// ========================================================================================================================
// ========================================================================================================================

use image::{ImageFormat, ImageReader, ImageEncoder, ExtendedColorType, EncodableLayout};
use image::codecs::{
    ico::IcoEncoder,
    openexr::OpenExrEncoder,
    hdr::HdrEncoder,
    farbfeld::FarbfeldEncoder,
};
use std::io::BufWriter;

/// Конвертация изображений между поддерживаемыми форматами
async fn convert_image_to_image(path: &str, from: &str, to: &str) -> Result<String, String> {
    // 1. Декодируем исходное изображение в пиксельную матрицу
    let img = ImageReader::open(path)
        .map_err(|e| format!("Cannot open image: {}", e))?
        .decode()
        .map_err(|e| format!("Cannot decode image: {}", e))?;
    
    // 2. Генерируем путь для сохранения результата
    let hash = calculate_conversion_hash(path, from, to)
        .map_err(|e| format!("Hash error convert_image_to_image: {}", e))?;
    let out_path = get_app_dir_path_with_hash(path, to, &hash, true)?;
    
    let to_lower = to.to_lowercase();
    
    // 3. Выбираем правильный энкодер
    match to_lower.as_str() {
        // ICO: требует strict-размер (макс 256x256) и квадратные пропорции
        "ico" => {
            let file = std::fs::File::create(&out_path).map_err(|e| format!("Cannot create file: {}", e))?;
            let mut writer = BufWriter::new(file);
            let thumb = img.thumbnail(256, 256).to_rgba8();
            let encoder = IcoEncoder::new(&mut writer);
            encoder.write_image(thumb.as_bytes(), thumb.width(), thumb.height(), ExtendedColorType::Rgba8)
                .map_err(|e| format!("Cannot save ICO: {}", e))?;
            Ok(out_path)
        }
        
        // EXR: требует 32-битный float формат (Rgb32F)
        "exr" | "openexr" => {
            let file = std::fs::File::create(&out_path).map_err(|e| format!("Cannot create file: {}", e))?;
            let mut writer = BufWriter::new(file);
            let rgb_f32 = img.to_rgb32f();
            let encoder = OpenExrEncoder::new(&mut writer);
            encoder.write_image(rgb_f32.as_bytes(), rgb_f32.width(), rgb_f32.height(), ExtendedColorType::Rgb32F)
                .map_err(|e| format!("Cannot save EXR: {}", e))?;
            Ok(out_path)
        }
        
        // HDR: требует перевод в Rgb32F
        "hdr" => {
            let file = std::fs::File::create(&out_path).map_err(|e| format!("Cannot create file: {}", e))?;
            let mut writer = BufWriter::new(file);
            let rgb_f32 = img.to_rgb32f();
            let encoder = HdrEncoder::new(&mut writer);
            encoder.write_image(rgb_f32.as_bytes(), rgb_f32.width(), rgb_f32.height(), ExtendedColorType::Rgb32F)
                .map_err(|e| format!("Cannot save HDR: {}", e))?;
            Ok(out_path)
        }
        
        // Farbfeld (ff): Полностью поддерживается! Переводим в 16 бит согласно спецификации
        "ff" | "farbfeld" => {
            let file = std::fs::File::create(&out_path).map_err(|e| format!("Cannot create file: {}", e))?;
            let mut writer = BufWriter::new(file);
            let rgba16 = img.to_rgba16(); 
            let encoder = FarbfeldEncoder::new(&mut writer);
            encoder.write_image(rgba16.as_bytes(), rgba16.width(), rgba16.height(), ExtendedColorType::Rgba16)
                .map_err(|e| format!("Cannot save Farbfeld: {}", e))?;
            Ok(out_path)
        }
        
        // JPEG: требует RGB8 (не поддерживает альфа-канал)
        "jpg" | "jpeg" => {
            let rgb = img.to_rgb8();
            rgb.save_with_format(&out_path, ImageFormat::Jpeg)
                .map_err(|e| format!("Cannot save image to JPEG: {}", e))?;
            Ok(out_path)
        }
        
        // Все остальные стандартные форматы
        _ => {
            let format = match to_lower.as_str() {
                "png" => ImageFormat::Png,
                "gif" => ImageFormat::Gif,
                "webp" => ImageFormat::WebP,
                "avif" => ImageFormat::Avif,
                "bmp" => ImageFormat::Bmp,
                "tiff" | "tif" => ImageFormat::Tiff,
                "tga" => ImageFormat::Tga,
                "pnm" | "pgm" | "ppm" => ImageFormat::Pnm,
                "qoi" => ImageFormat::Qoi,
                // DDS не поддерживает кодирование!
                // "dds" => return Err("DDS encoding is not supported".to_string()),
                _ => return Err(format!("Unsupported output format: {}", to)),
            };
            
            // Для PNG и других форматов, которые поддерживают альфа-канал
            img.save_with_format(&out_path, format)
                .map_err(|e| format!("Cannot save image to {}: {}", to, e))?;
            
            Ok(out_path)
        }
    }
}



async fn convert_audio_to_audio(path: &str, from: &str, to: &str) -> Result<String, String> {
    audio::convert_audio_to_audio(path, from, to)
}

/// Video → Video
async fn convert_video_to_video(path: &str, from: &str, to: &str) -> Result<String, String> {
    video::convert_video_to_video(path, from, to)
}
/// Video → Audio — извлекаем аудио дорожку и конвертируем в целевой формат
async fn convert_video_to_audio(path: &str, from: &str, to: &str) -> Result<String, String> {
    video::convert_video_to_audio(path, from, to)
}

async fn convert_document_to_audio(path: &str, from: &str, to: &str) -> Result<String, String> {
    document_to_audio::convert_document_to_audio(path, from, to)
}
// Функции-обертки:
async fn convert_audio_to_text(db: &DatabaseConnection, path: &str, from: &str, to: &str) -> Result<String, String> {
    audio_to_text::convert_audio_to_text(db,path, from, to).await
}

/// Video → Text (извлекаем аудио, потом распознаем)
async fn convert_video_to_text(
    db: &DatabaseConnection,
    path: &str, 
    from: &str, 
    to: &str
) -> Result<String, String> {
    // 1. Извлекаем аудио из видео в WAV
    let audio_path = video::convert_video_to_audio(path, from, "wav")?;
    
    // 2. Распознаем аудио в текст
    let result = audio_to_text::convert_audio_to_text(db, &audio_path, "wav", to).await?;
    
    // 3. Проверяем кеш перед удалением
    // if !is_file_cached(db, path, from, "wav").await? {
    //     if let Err(e) = std::fs::remove_file(&audio_path) {
    //         eprintln!("Warning: Failed to remove temp audio file: {}", e);
    //     }
    // }
    
    Ok(result)
}

/// Audio → Document
async fn convert_audio_to_document(
    db: &DatabaseConnection,
    path: &str, 
    from: &str, 
    to: &str
) -> Result<String, String> {
    // 1. Распознаем аудио в текст
    let text_path = audio_to_text::convert_audio_to_text(db, path, from, "txt").await?;
    
    // 2. Читаем текст
    let text = std::fs::read_to_string(&text_path)
        .map_err(|e| format!("Cannot read text file: {}", e))?;
    
    // 3. Конвертируем текст в документ (используем оригинальный path)
    let result = stringify_document(db, &text, path, from, to).await?;
    
    // 4. Проверяем кеш перед удалением
    // if !is_file_cached(db, path, from, "txt").await? {
    //     let _ = std::fs::remove_file(&text_path);
    // }
    
    Ok(result)
}

/// Video → Document
async fn convert_video_to_document(
    db: &DatabaseConnection,
    path: &str, 
    from: &str, 
    to: &str
) -> Result<String, String> {
    // 1. Извлекаем аудио из видео в WAV
    let audio_path = video::convert_video_to_audio(path, from, "wav")?;
    
    // 2. Распознаем аудио в текст
    let text_path = audio_to_text::convert_audio_to_text(db, &audio_path, "wav", "txt").await?;
    
    // 3. Читаем текст
    let text = std::fs::read_to_string(&text_path)
        .map_err(|e| format!("Cannot read text file: {}", e))?;
    
    // 4. Конвертируем текст в документ (используем оригинальный path)
    let result = stringify_document(db, &text, path, from, to).await?;
    
    // 5. Проверяем кеш перед удалением временных файлов
    // if !is_file_cached(db, path, from, "wav").await? {
    //     let _ = std::fs::remove_file(&audio_path);
    // }
    
    // if !is_file_cached(db, &audio_path, "wav", "txt").await? {
    //     let _ = std::fs::remove_file(&text_path);
    // }
    
    Ok(result)
}
// ============================================================
// ПАРСЕРЫ И СЕРИАЛИЗАТОРЫ
// ============================================================

pub fn parse(input: &str, format: &str) -> Result<Json, String> {
    match format {
        "json" => serde_json::from_str(input).map_err(|e| format!("JSON: {e}")),//good
        "yaml" | "yml" => serde_yaml::from_str(input).map_err(|e| format!("YAML: {e}")), //good
        "toml" => toml::from_str(input).map_err(|e| format!("TOML: {e}")), //good
        "xml" => parse_xml(input), //good
        "ini" => parse_ini(input),//good
        "md" => parse_markdown(input),//good
        "csv" => parse_csv(input), //good
        "html" => parse_html(input),//good
        "txt" | "text" => parse_txt(input),//good
        "rtf" => parse_rtf(input),//good
        _ => Err(format!("Unsupported: {format}")),//good
    }
}

/// Сериализует JSON в файл и возвращает путь к нему
pub fn stringify(value: &Json, format: &str, path: &str, from: &str) -> Result<String, String> {
    // Получаем содержимое для всех форматов кроме RTF
    let content = match format {
        "json" => serde_json::to_string_pretty(value).map_err(|e| format!("JSON: {e}"))?,
        "yaml" | "yml" => serde_yaml::to_string(value).map_err(|e| format!("YAML: {e}"))?,
        "toml" => {
            let value_for_toml = match value {
                Json::Array(arr) => {
                    let mut map = serde_json::Map::new();
                    map.insert("data".to_string(), Json::Array(arr.clone()));
                    Json::Object(map)
                }
                _ => value.clone(),
            };
            toml::to_string_pretty(&value_for_toml).map_err(|e| format!("TOML: {e}"))?
        }
        "xml" => stringify_xml(value).map_err(|e| format!("XML: {e}"))?,
        "csv" => stringify_csv(value)?,
        "ini" => stringify_ini(value)?,
        "html" => convert_to_html(value),
        "md" => stringify_markdown(value)?,
        "txt" | "text" => stringify_txt(value)?,
        _ => return Err(format!("Unsupported: {format}")),
    };
    
    // Для всех остальных форматов - сохраняем содержимое в файл
    let hash = calculate_conversion_hash(path, from, format)
        .map_err(|e| format!("Hash error stringify: {}", e))?;
    let output_path = save_to_app_dir(&content, path, format, &hash)?;
    
    Ok(output_path)
}



// ============================================================
// ВСПОМОГАТЕЛЬНЫЕ ФУНКЦИИ
// ============================================================

/// Сохраняет контент в файл с заменой хэша в имени
/// Возвращает путь к файлу с заменой хэша в имени
/// 
/// # Arguments
/// * `original_path` - исходный путь (может содержать @hash@)
/// * `to` - целевое расширение
/// * `hash` - новый хэш
/// * `overwrite` - если true, удаляет существующий файл; если false, просто возвращает путь
pub fn get_app_dir_path_with_hash(
    original_path: &str, 
    to: &str, 
    hash: &str,
    overwrite: bool
) -> Result<String, String> {
    let input_path = PathBuf::from(original_path);
    let stem = input_path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("converted");
    
    // Если в имени есть @hash@ - берем часть до него (оригинальное имя)
    let base_name = if let Some(pos) = stem.find("@hash@") {
        &stem[..pos]
    } else {
        stem
    };
    
    let output_dir = converted_dir();
    let output_path = output_dir.join(format!("{}@hash@{}.{}", base_name, hash, to));
    let output_path_str = output_path.to_string_lossy().to_string();
    
    // Если нужно перезаписать и файл существует - удаляем
    // if overwrite && Path::new(&output_path_str).exists() {
    //     std::fs::remove_file(&output_path_str)
    //         .map_err(|e| format!("Cannot remove existing file: {}", e))?;
    // }
    
    Ok(output_path_str)
}

/// Сохраняет контент в файл с заменой хэша в имени
/// Если файл уже существует - возвращает путь без перезаписи
pub fn save_to_app_dir(content: &str, original_path: &str, to: &str, hash: &str) -> Result<String, String> {
    // Получаем путь без перезаписи (overwrite = false)
    let output_path = get_app_dir_path_with_hash(original_path, to, hash, false)?;
    
    // Если файл уже существует - просто возвращаем путь
    if Path::new(&output_path).exists() {
        return Ok(output_path);
    }
    
    // Создаем директорию если нужно
    if let Some(parent) = Path::new(&output_path).parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Cannot create directory: {}", e))?;
        }
    }
    
    // Записываем новый файл
    std::fs::write(&output_path, content)
        .map_err(|e| format!("Cannot write file: {}", e))?;
    
    Ok(output_path)
}


fn calculate_hash(path: &str, extra_data: &[&[u8]]) -> std::io::Result<String> {
    let file = File::open(path)?;
    let mut hasher = Xxh3::new();

    match unsafe { Mmap::map(&file) } {
        Ok(mmap) => {
            hasher.update(&mmap);
        }
        Err(_) => {
            let mut file = file;
            let mut buffer = [0; 65536];
            loop {
                let bytes_read = file.read(&mut buffer)?;
                if bytes_read == 0 {
                    break;
                }
                hasher.update(&buffer[..bytes_read]);
            }
        }
    }

    for data in extra_data {
        hasher.update(data);
    }

    Ok(format!("{:x}", hasher.digest()))
}

fn calculate_conversion_hash(path: &str, from: &str, to: &str) -> std::io::Result<String> {
    calculate_hash(path, &[from.as_bytes(), to.as_bytes()])
}

fn calculate_file_hash(path: &str) -> std::io::Result<String> {
    calculate_hash(path, &[])
}

#[tauri::command]
pub async fn hash_file(path: String) -> Result<String, String> {
    calculate_file_hash(&path).map_err(|e| format!("Cannot hash file: {e}"))
}


#[tauri::command]
pub async fn convert_file(
    state: tauri::State<'_, AppState>,
    path: String,
    from: String,
    to: String,
    #[allow(nonstandard_style)]
    fromType: String,
    #[allow(nonstandard_style)]
    toType: String,
    enable_cache: bool,
) -> Result<ConvertResult, String> {
    let input_hash = calculate_conversion_hash(&path, &from, &to)
        .map_err(|e| format!("Cannot read file: {e}"))?;
    println!("INIT HASH=>{}", input_hash);
    
    let db_guard = state.db.lock().await;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;
    
    // Проверяем кеш
    if enable_cache {
        if let Some(existing_path) = db::find_conversion(db, &input_hash).await {
            let extension = Path::new(&existing_path)
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_string());

            return Ok(ConvertResult {
                success: true,
                content: existing_path,
                hash: Some(input_hash),
                extension,
                error: None,
            });
        }
    }
    
    // Выполняем конвертацию (возвращает путь к файлу)
    let (path_clone, from_clone, to_clone, from_type_clone, to_type_clone) = 
        (path.clone(), from.clone(), to.clone(), fromType.clone(), toType.clone());
    let db_clone = db.clone();
    
    let output_path = tokio::spawn(async move {
        convert(&db_clone, &path_clone, &from_clone, &to_clone, &from_type_clone, &to_type_clone).await
    }).await.map_err(|e| format!("Task join error: {e}"))??;

    // Переименовываем файл с использованием правильного хэша
    let output_path_buf = PathBuf::from(&output_path);
    let output_dir = output_path_buf.parent()
        .ok_or_else(|| "Failed to get output directory".to_string())?;
    
    let original_name = output_path_buf.file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| "Failed to get original file name".to_string())?;
    
    // Разделяем имя по @hash@ и берем первую часть (оригинальное имя)
    let parts: Vec<&str> = original_name.split("@hash@").collect();
    let base_name = parts.first().unwrap_or(&original_name);
    
    // Формируем новое имя с правильным хэшем
    let new_file_name = format!("{}@hash@{}.{}", base_name, input_hash, to);
    let new_output_path = output_dir.join(new_file_name);
    
    // Переименовываем файл только если пути разные
    if output_path_buf != new_output_path {
        tokio::fs::rename(&output_path_buf, &new_output_path).await
            .map_err(|e| format!("Failed to rename file: {e}"))?;
        println!("File renamed: {:?} -> {:?}", output_path_buf, new_output_path);
    }
    
    // Сохраняем в кеш с новым путем
    let final_path = new_output_path.to_string_lossy().to_string();
    db::save_conversion(db, &input_hash, &final_path).await?;

    let extension = Path::new(&final_path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_string());

    Ok(ConvertResult {
        success: true,
        content: final_path,
        hash: Some(input_hash),
        extension,
        error: None,
    })
}

#[tauri::command]
pub async fn read_file_content(path: String) -> Result<String, String> {
    tokio::fs::read_to_string(&path).await.map_err(|e| format!("Cannot read file: {e}"))
}

#[tauri::command]
pub async fn read_file_bytes(path: String) -> Result<Vec<u8>, String> {
    tokio::fs::read(&path)
        .await
        .map_err(|e| format!("Cannot read file: {e}"))
}

#[tauri::command]
pub async fn open_file(path: String) -> Result<(), String> {
    opener::open(&path).map_err(|e| format!("Cannot open file: {e}"))
}