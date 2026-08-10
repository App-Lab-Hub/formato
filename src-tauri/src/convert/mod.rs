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


use image::{ImageFormat, ImageReader, ImageEncoder, ExtendedColorType, EncodableLayout, DynamicImage};
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
    let img = match ImageReader::open(path) {
        Ok(reader) => {
            if from == "pnm" || path.to_lowercase().ends_with(".pnm") {
                match reader.with_guessed_format() {
                    Ok(r) => {
                        match r.decode() {
                            Ok(img) => img,
                            Err(e) => {
                                match ImageReader::open(path) {
                                    Ok(r2) => {
                                        match r2.decode() {
                                            Ok(img) => img,
                                            Err(_) => return Err(format!("Cannot decode PNM/PAM file: {}", e))
                                        }
                                    }
                                    Err(e2) => return Err(format!("Cannot open PNM file: {}", e2))
                                }
                            }
                        }
                    }
                    Err(_) => {
                        match ImageReader::open(path) {
                            Ok(r) => r.decode().map_err(|e| format!("Cannot decode image: {}", e))?,
                            Err(e) => return Err(format!("Cannot open image: {}", e)),
                        }
                    }
                }
            } else {
                reader.decode().map_err(|e| format!("Cannot decode image: {}", e))?
            }
        }
        Err(e) => return Err(format!("Cannot open image: {}", e)),
    };
    
    // 2. Генерируем путь для сохранения результата
    let hash = calculate_conversion_hash(path, from, to)
        .map_err(|e| format!("Hash error convert_image_to_image: {}", e))?;
    let out_path = get_app_dir_path_with_hash(path, to, &hash, true)?;
    
    let to_lower = to.to_lowercase();
    
    // 3. Определяем тип изображения
    let is_rgb32f = matches!(img, DynamicImage::ImageRgb32F(_));
    let is_rgba16 = matches!(img, DynamicImage::ImageRgba16(_));
    
    // 4. Выбираем правильный энкодер
    match to_lower.as_str() {
        // ICO: требует размер ≤256x256 и квадратные пропорции
        "ico" => {
            let file = std::fs::File::create(&out_path).map_err(|e| format!("Cannot create file: {}", e))?;
            let mut writer = BufWriter::new(file);
            let thumb = img.thumbnail(256, 256).to_rgba8();
            let encoder = IcoEncoder::new(&mut writer);
            encoder.write_image(thumb.as_bytes(), thumb.width(), thumb.height(), ExtendedColorType::Rgba8)
                .map_err(|e| format!("Cannot save ICO: {}", e))?;
            Ok(out_path)
        }
        
        // EXR: требует Rgb32F
        "exr" | "openexr" => {
            let file = std::fs::File::create(&out_path).map_err(|e| format!("Cannot create file: {}", e))?;
            let mut writer = BufWriter::new(file);
            let rgb_f32 = img.to_rgb32f();
            let encoder = OpenExrEncoder::new(&mut writer);
            encoder.write_image(rgb_f32.as_bytes(), rgb_f32.width(), rgb_f32.height(), ExtendedColorType::Rgb32F)
                .map_err(|e| format!("Cannot save EXR: {}", e))?;
            Ok(out_path)
        }
        
        // HDR: требует Rgb32F
        "hdr" => {
            let file = std::fs::File::create(&out_path).map_err(|e| format!("Cannot create file: {}", e))?;
            let mut writer = BufWriter::new(file);
            let rgb_f32 = img.to_rgb32f();
            let encoder = HdrEncoder::new(&mut writer);
            encoder.write_image(rgb_f32.as_bytes(), rgb_f32.width(), rgb_f32.height(), ExtendedColorType::Rgb32F)
                .map_err(|e| format!("Cannot save HDR: {}", e))?;
            Ok(out_path)
        }
        
        // Farbfeld: требует Rgba16
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
                _ => return Err(format!("Unsupported output format: {}", to)),
            };
            
            // 🎯 Конвертируем в правильный цветовой тип
            let img_to_save: DynamicImage = if is_rgb32f {
                // Rgb32F → RGBA8
                DynamicImage::ImageRgba8(img.to_rgba8())
            } else if is_rgba16 {
                // Rgba16 → RGBA8 (для Farbfeld)
                DynamicImage::ImageRgba8(img.to_rgba8())
            } else {
                img
            };
            
            img_to_save.save_with_format(&out_path, format)
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

// src-tauri/src/convert/mod.rs

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use sea_orm::{Database, DatabaseConnection, DbErr};
    use std::sync::Arc;

    // ============================================================
    // ВСПОМОГАТЕЛЬНЫЕ ФУНКЦИИ
    // ============================================================

    async fn create_test_db() -> Result<Arc<DatabaseConnection>, DbErr> {
        let db = Database::connect("sqlite::memory:").await?;
        Ok(Arc::new(db))
    }

    fn get_fixture_files(ext: &str) -> Vec<PathBuf> {
        let fixtures_dir = PathBuf::from("../fixtures");
        if !fixtures_dir.exists() {
            return vec![];
        }
        
        let entries = fs::read_dir(&fixtures_dir).unwrap();
        let mut files = Vec::new();
        for entry in entries {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file() {
                if let Some(extension) = path.extension() {
                    if extension == ext {
                        files.push(path);
                    }
                }
            }
        }
        files
    }

    fn has_fixtures(ext: &str) -> bool {
        !get_fixture_files(ext).is_empty()
    }

    // ============================================================
    // УНИВЕРСАЛЬНАЯ ФУНКЦИЯ ДЛЯ ТЕСТИРОВАНИЯ
    // ============================================================

    async fn test_conversion<F, Fut>(
        from_format: &str,
        to_formats: &[&str],
        convert_fn: F,
        db: Option<Arc<DatabaseConnection>>,
    ) where
        F: Fn(Arc<DatabaseConnection>, String, String, String) -> Fut,
        Fut: std::future::Future<Output = Result<String, String>>,
    {
        if !has_fixtures(from_format) {
            println!("⚠️ Skipping test: no {} fixtures found", from_format);
            return;
        }

        let files = get_fixture_files(from_format);
        println!("📁 Found {} {} files", files.len(), from_format.to_uppercase());

        let db_arc = if let Some(db) = db {
            db
        } else {
            create_test_db().await.unwrap()
        };

        // Собираем все ошибки
        let mut errors = Vec::new();
        let mut total_tests = 0;
        let mut passed = 0;

        for input_path in files {
            let path_str = input_path.to_str().unwrap().to_string();
            let file_name = input_path.file_name().unwrap().to_string_lossy();
            println!("🔄 Testing: {}", file_name);
            
            for &to_format in to_formats {
                if to_format == from_format {
                    continue;
                }

                total_tests += 1;
                println!("  → {}", to_format);
                
                let result = convert_fn(
                    db_arc.clone(),
                    path_str.clone(),
                    from_format.to_string(),
                    to_format.to_string()
                ).await;

                match result {
                    Ok(output_path) => {
                        passed += 1;
                        assert!(output_path.ends_with(&format!(".{}", to_format)));
                        assert!(PathBuf::from(&output_path).exists());
                        let metadata = fs::metadata(&output_path).unwrap();
                        assert!(metadata.len() > 0, "File is empty: {}", output_path);
                        println!("    ✅ {} bytes", metadata.len());
                    }
                    Err(e) => {
                        println!("    ❌ Error: {}", e);
                        errors.push((from_format.to_string(), to_format.to_string(), e));
                    }
                }
            }
            println!();
        }

        // Выводим статистику
        println!("\n📊 Results for {} → all formats:", from_format.to_uppercase());
        println!("  ✅ Passed: {}", passed);
        println!("  ❌ Failed: {}", total_tests - passed);
        println!("  📦 Total: {}", total_tests);

        // Если есть ошибки - падаем с подробным отчетом
        if !errors.is_empty() {
            println!("\n❌ ERROR SUMMARY:");
            for (from, to, error) in &errors {
                println!("  {} → {}: {}", from, to, error);
            }
            panic!("\n❌ {} conversions failed (see error summary above)", errors.len());
        }
    }

    // ============================================================
    // ФОРМАТЫ
    // ============================================================

    const AUDIO_FORMATS: &[&str] = &[
        "mp3", "wav", "aac", "flac", "ogg", "opus", "wma", "m4a", 
        "aiff", "ac3", "eac3", "tta", "wv", "voc", "adx", 
        "aptx", "sbc", "caf", "w64"
    ];

    const IMAGE_FORMATS: &[&str] = &[
        "jpg", "jpeg", "png", "webp", "avif", "gif", "bmp", "tiff", 
        "ico", "qoi", "tga", "exr", "hdr", "pnm", "ff"
    ];

    const VIDEO_FORMATS: &[&str] = &[
        "mp4", "mov", "avi", "mkv", "webm", "wmv", "flv", "3gp", 
        "m4v", "ts", "vob", "mpg", "hevc", "mjpeg", "nut"
    ];

    const TEXT_FORMATS: &[&str] = &[
        "json", "yaml", "csv", "xml", "toml", "ini", "md", "html", "txt", "rtf"
    ];

    const DOCUMENT_FORMATS: &[&str] = &[
        "pdf", "docx", "odt", "xlsx"
    ];

    // ============================================================
    // МОДУЛЬ: AUDIO → AUDIO
    // ============================================================
    
    mod audio_to_audio {
        use super::*;

        #[tokio::test]
        async fn test_mp3_to_all_audio_formats() {
            test_conversion("mp3", super::AUDIO_FORMATS, |_db, path, from, to| async move {
                audio::convert_audio_to_audio(&path, &from, &to)
            }, None).await;
        }

        #[tokio::test]
        async fn test_wav_to_all_audio_formats() {
            test_conversion("wav", super::AUDIO_FORMATS, |_db, path, from, to| async move {
                audio::convert_audio_to_audio(&path, &from, &to)
            }, None).await;
        }

        #[tokio::test]
        async fn test_aac_to_all_audio_formats() {
            test_conversion("aac", super::AUDIO_FORMATS, |_db, path, from, to| async move {
                audio::convert_audio_to_audio(&path, &from, &to)
            }, None).await;
        }

        #[tokio::test]
        async fn test_flac_to_all_audio_formats() {
            test_conversion("flac", super::AUDIO_FORMATS, |_db, path, from, to| async move {
                audio::convert_audio_to_audio(&path, &from, &to)
            }, None).await;
        }

        #[tokio::test]
        async fn test_ogg_to_all_audio_formats() {
            test_conversion("ogg", super::AUDIO_FORMATS, |_db, path, from, to| async move {
                audio::convert_audio_to_audio(&path, &from, &to)
            }, None).await;
        }

        #[tokio::test]
        async fn test_opus_to_all_audio_formats() {
            test_conversion("opus", super::AUDIO_FORMATS, |_db, path, from, to| async move {
                audio::convert_audio_to_audio(&path, &from, &to)
            }, None).await;
        }

        #[tokio::test]
        async fn test_wma_to_all_audio_formats() {
            test_conversion("wma", super::AUDIO_FORMATS, |_db, path, from, to| async move {
                audio::convert_audio_to_audio(&path, &from, &to)
            }, None).await;
        }

        #[tokio::test]
        async fn test_m4a_to_all_audio_formats() {
            test_conversion("m4a", super::AUDIO_FORMATS, |_db, path, from, to| async move {
                audio::convert_audio_to_audio(&path, &from, &to)
            }, None).await;
        }

        #[tokio::test]
        async fn test_aiff_to_all_audio_formats() {
            test_conversion("aiff", super::AUDIO_FORMATS, |_db, path, from, to| async move {
                audio::convert_audio_to_audio(&path, &from, &to)
            }, None).await;
        }

        #[tokio::test]
        async fn test_ac3_to_all_audio_formats() {
            test_conversion("ac3", super::AUDIO_FORMATS, |_db, path, from, to| async move {
                audio::convert_audio_to_audio(&path, &from, &to)
            }, None).await;
        }

        #[tokio::test]
        async fn test_eac3_to_all_audio_formats() {
            test_conversion("eac3", super::AUDIO_FORMATS, |_db, path, from, to| async move {
                audio::convert_audio_to_audio(&path, &from, &to)
            }, None).await;
        }


        #[tokio::test]
        async fn test_tta_to_all_audio_formats() {
            test_conversion("tta", super::AUDIO_FORMATS, |_db, path, from, to| async move {
                audio::convert_audio_to_audio(&path, &from, &to)
            }, None).await;
        }

        #[tokio::test]
        async fn test_wv_to_all_audio_formats() {
            test_conversion("wv", super::AUDIO_FORMATS, |_db, path, from, to| async move {
                audio::convert_audio_to_audio(&path, &from, &to)
            }, None).await;
        }

        #[tokio::test]
        async fn test_voc_to_all_audio_formats() {
            test_conversion("voc", super::AUDIO_FORMATS, |_db, path, from, to| async move {
                audio::convert_audio_to_audio(&path, &from, &to)
            }, None).await;
        }

        #[tokio::test]
        async fn test_adx_to_all_audio_formats() {
            test_conversion("adx", super::AUDIO_FORMATS, |_db, path, from, to| async move {
                audio::convert_audio_to_audio(&path, &from, &to)
            }, None).await;
        }

        #[tokio::test]
        async fn test_aptx_to_all_audio_formats() {
            test_conversion("aptx", super::AUDIO_FORMATS, |_db, path, from, to| async move {
                audio::convert_audio_to_audio(&path, &from, &to)
            }, None).await;
        }

        #[tokio::test]
        async fn test_sbc_to_all_audio_formats() {
            test_conversion("sbc", super::AUDIO_FORMATS, |_db, path, from, to| async move {
                audio::convert_audio_to_audio(&path, &from, &to)
            }, None).await;
        }


        #[tokio::test]
        async fn test_caf_to_all_audio_formats() {
            test_conversion("caf", super::AUDIO_FORMATS, |_db, path, from, to| async move {
                audio::convert_audio_to_audio(&path, &from, &to)
            }, None).await;
        }

        #[tokio::test]
        async fn test_w64_to_all_audio_formats() {
            test_conversion("w64", super::AUDIO_FORMATS, |_db, path, from, to| async move {
                audio::convert_audio_to_audio(&path, &from, &to)
            }, None).await;
        }
    }

    // ============================================================
    // МОДУЛЬ: IMAGE → IMAGE
    // ============================================================
    
    mod image_to_image {
        use super::*;

        #[tokio::test]
        async fn test_jpg_to_all_image_formats() {
            test_conversion("jpg", super::IMAGE_FORMATS, |_db, path, from, to| async move {
                convert_image_to_image(&path, &from, &to).await
            }, None).await;
        }

        #[tokio::test]
        async fn test_png_to_all_image_formats() {
            test_conversion("png", super::IMAGE_FORMATS, |_db, path, from, to| async move {
                convert_image_to_image(&path, &from, &to).await
            }, None).await;
        }

        #[tokio::test]
        async fn test_webp_to_all_image_formats() {
            test_conversion("webp", super::IMAGE_FORMATS, |_db, path, from, to| async move {
                convert_image_to_image(&path, &from, &to).await
            }, None).await;
        }

        #[tokio::test]
        async fn test_avif_to_all_image_formats() {
            test_conversion("avif", super::IMAGE_FORMATS, |_db, path, from, to| async move {
                convert_image_to_image(&path, &from, &to).await
            }, None).await;
        }

        #[tokio::test]
        async fn test_gif_to_all_image_formats() {
            test_conversion("gif", super::IMAGE_FORMATS, |_db, path, from, to| async move {
                convert_image_to_image(&path, &from, &to).await
            }, None).await;
        }

        #[tokio::test]
        async fn test_bmp_to_all_image_formats() {
            test_conversion("bmp", super::IMAGE_FORMATS, |_db, path, from, to| async move {
                convert_image_to_image(&path, &from, &to).await
            }, None).await;
        }

        #[tokio::test]
        async fn test_tiff_to_all_image_formats() {
            test_conversion("tiff", super::IMAGE_FORMATS, |_db, path, from, to| async move {
                convert_image_to_image(&path, &from, &to).await
            }, None).await;
        }

        #[tokio::test]
        async fn test_ico_to_all_image_formats() {
            test_conversion("ico", super::IMAGE_FORMATS, |_db, path, from, to| async move {
                convert_image_to_image(&path, &from, &to).await
            }, None).await;
        }

        #[tokio::test]
        async fn test_qoi_to_all_image_formats() {
            test_conversion("qoi", super::IMAGE_FORMATS, |_db, path, from, to| async move {
                convert_image_to_image(&path, &from, &to).await
            }, None).await;
        }

        #[tokio::test]
        async fn test_tga_to_all_image_formats() {
            test_conversion("tga", super::IMAGE_FORMATS, |_db, path, from, to| async move {
                convert_image_to_image(&path, &from, &to).await
            }, None).await;
        }

        #[tokio::test]
        async fn test_exr_to_all_image_formats() {
            test_conversion("exr", super::IMAGE_FORMATS, |_db, path, from, to| async move {
                convert_image_to_image(&path, &from, &to).await
            }, None).await;
        }

        #[tokio::test]
        async fn test_hdr_to_all_image_formats() {
            test_conversion("hdr", super::IMAGE_FORMATS, |_db, path, from, to| async move {
                convert_image_to_image(&path, &from, &to).await
            }, None).await;
        }

        #[tokio::test]
        async fn test_pnm_to_all_image_formats() {
            test_conversion("pnm", super::IMAGE_FORMATS, |_db, path, from, to| async move {
                convert_image_to_image(&path, &from, &to).await
            }, None).await;
        }

        #[tokio::test]
        async fn test_ff_to_all_image_formats() {
            test_conversion("ff", super::IMAGE_FORMATS, |_db, path, from, to| async move {
                convert_image_to_image(&path, &from, &to).await
            }, None).await;
        }
    }

    // ============================================================
    // МОДУЛЬ: VIDEO → VIDEO
    // ============================================================
    
    mod video_to_video {
        use super::*;

        #[tokio::test]
        async fn test_mp4_to_all_video_formats() {
            test_conversion("mp4", super::VIDEO_FORMATS, |_db, path, from, to| async move {
                video::convert_video_to_video(&path, &from, &to)
            }, None).await;
        }

        #[tokio::test]
        async fn test_mov_to_all_video_formats() {
            test_conversion("mov", super::VIDEO_FORMATS, |_db, path, from, to| async move {
                video::convert_video_to_video(&path, &from, &to)
            }, None).await;
        }

        #[tokio::test]
        async fn test_avi_to_all_video_formats() {
            test_conversion("avi", super::VIDEO_FORMATS, |_db, path, from, to| async move {
                video::convert_video_to_video(&path, &from, &to)
            }, None).await;
        }

        #[tokio::test]
        async fn test_mkv_to_all_video_formats() {
            test_conversion("mkv", super::VIDEO_FORMATS, |_db, path, from, to| async move {
                video::convert_video_to_video(&path, &from, &to)
            }, None).await;
        }

        #[tokio::test]
        async fn test_webm_to_all_video_formats() {
            test_conversion("webm", super::VIDEO_FORMATS, |_db, path, from, to| async move {
                video::convert_video_to_video(&path, &from, &to)
            }, None).await;
        }

        #[tokio::test]
        async fn test_wmv_to_all_video_formats() {
            test_conversion("wmv", super::VIDEO_FORMATS, |_db, path, from, to| async move {
                video::convert_video_to_video(&path, &from, &to)
            }, None).await;
        }

        #[tokio::test]
        async fn test_flv_to_all_video_formats() {
            test_conversion("flv", super::VIDEO_FORMATS, |_db, path, from, to| async move {
                video::convert_video_to_video(&path, &from, &to)
            }, None).await;
        }

        #[tokio::test]
        async fn test_3gp_to_all_video_formats() {
            test_conversion("3gp", super::VIDEO_FORMATS, |_db, path, from, to| async move {
                video::convert_video_to_video(&path, &from, &to)
            }, None).await;
        }

        #[tokio::test]
        async fn test_m4v_to_all_video_formats() {
            test_conversion("m4v", super::VIDEO_FORMATS, |_db, path, from, to| async move {
                video::convert_video_to_video(&path, &from, &to)
            }, None).await;
        }

        #[tokio::test]
        async fn test_ts_to_all_video_formats() {
            test_conversion("ts", super::VIDEO_FORMATS, |_db, path, from, to| async move {
                video::convert_video_to_video(&path, &from, &to)
            }, None).await;
        }

        #[tokio::test]
        async fn test_vob_to_all_video_formats() {
            test_conversion("vob", super::VIDEO_FORMATS, |_db, path, from, to| async move {
                video::convert_video_to_video(&path, &from, &to)
            }, None).await;
        }

        #[tokio::test]
        async fn test_mpg_to_all_video_formats() {
            test_conversion("mpg", super::VIDEO_FORMATS, |_db, path, from, to| async move {
                video::convert_video_to_video(&path, &from, &to)
            }, None).await;
        }

        #[tokio::test]
        async fn test_hevc_to_all_video_formats() {
            test_conversion("hevc", super::VIDEO_FORMATS, |_db, path, from, to| async move {
                video::convert_video_to_video(&path, &from, &to)
            }, None).await;
        }

        #[tokio::test]
        async fn test_mjpeg_to_all_video_formats() {
            test_conversion("mjpeg", super::VIDEO_FORMATS, |_db, path, from, to| async move {
                video::convert_video_to_video(&path, &from, &to)
            }, None).await;
        }

        #[tokio::test]
        async fn test_nut_to_all_video_formats() {
            test_conversion("nut", super::VIDEO_FORMATS, |_db, path, from, to| async move {
                video::convert_video_to_video(&path, &from, &to)
            }, None).await;
        }
    }

    // ============================================================
    // МОДУЛЬ: TEXT → TEXT (с БД)
    // ============================================================
    
    mod text_to_text {
        use super::*;

        #[tokio::test]
        async fn test_json_to_all_text_formats() {
            let db = create_test_db().await.unwrap();
            test_conversion("json", super::TEXT_FORMATS, |db, path, from, to| async move {
                convert_text_to_text(&db, &path, &from, &to).await
            }, Some(db)).await;
        }

        #[tokio::test]
        async fn test_yaml_to_all_text_formats() {
            let db = create_test_db().await.unwrap();
            test_conversion("yaml", super::TEXT_FORMATS, |db, path, from, to| async move {
                convert_text_to_text(&db, &path, &from, &to).await
            }, Some(db)).await;
        }

        #[tokio::test]
        async fn test_csv_to_all_text_formats() {
            let db = create_test_db().await.unwrap();
            test_conversion("csv", super::TEXT_FORMATS, |db, path, from, to| async move {
                convert_text_to_text(&db, &path, &from, &to).await
            }, Some(db)).await;
        }

        #[tokio::test]
        async fn test_xml_to_all_text_formats() {
            let db = create_test_db().await.unwrap();
            test_conversion("xml", super::TEXT_FORMATS, |db, path, from, to| async move {
                convert_text_to_text(&db, &path, &from, &to).await
            }, Some(db)).await;
        }

        #[tokio::test]
        async fn test_toml_to_all_text_formats() {
            let db = create_test_db().await.unwrap();
            test_conversion("toml", super::TEXT_FORMATS, |db, path, from, to| async move {
                convert_text_to_text(&db, &path, &from, &to).await
            }, Some(db)).await;
        }

        #[tokio::test]
        async fn test_ini_to_all_text_formats() {
            let db = create_test_db().await.unwrap();
            test_conversion("ini", super::TEXT_FORMATS, |db, path, from, to| async move {
                convert_text_to_text(&db, &path, &from, &to).await
            }, Some(db)).await;
        }

        #[tokio::test]
        async fn test_md_to_all_text_formats() {
            let db = create_test_db().await.unwrap();
            test_conversion("md", super::TEXT_FORMATS, |db, path, from, to| async move {
                convert_text_to_text(&db, &path, &from, &to).await
            }, Some(db)).await;
        }

        #[tokio::test]
        async fn test_html_to_all_text_formats() {
            let db = create_test_db().await.unwrap();
            test_conversion("html", super::TEXT_FORMATS, |db, path, from, to| async move {
                convert_text_to_text(&db, &path, &from, &to).await
            }, Some(db)).await;
        }

        #[tokio::test]
        async fn test_txt_to_all_text_formats() {
            let db = create_test_db().await.unwrap();
            test_conversion("txt", super::TEXT_FORMATS, |db, path, from, to| async move {
                convert_text_to_text(&db, &path, &from, &to).await
            }, Some(db)).await;
        }

        #[tokio::test]
        async fn test_rtf_to_all_text_formats() {
            let db = create_test_db().await.unwrap();
            test_conversion("rtf", super::TEXT_FORMATS, |db, path, from, to| async move {
                convert_text_to_text(&db, &path, &from, &to).await
            }, Some(db)).await;
        }
    }

    // ============================================================
    // МОДУЛЬ: TEXT → DOCUMENT (с БД)
    // ============================================================
    
    mod text_to_document {
        use super::*;

        #[tokio::test]
        async fn test_json_to_all_document_formats() {
            let db = create_test_db().await.unwrap();
            test_conversion("json", super::DOCUMENT_FORMATS, |db, path, from, to| async move {
                convert_text_to_document(&db, &path, &from, &to).await
            }, Some(db)).await;
        }

        #[tokio::test]
        async fn test_yaml_to_all_document_formats() {
            let db = create_test_db().await.unwrap();
            test_conversion("yaml", super::DOCUMENT_FORMATS, |db, path, from, to| async move {
                convert_text_to_document(&db, &path, &from, &to).await
            }, Some(db)).await;
        }

        #[tokio::test]
        async fn test_csv_to_all_document_formats() {
            let db = create_test_db().await.unwrap();
            test_conversion("csv", super::DOCUMENT_FORMATS, |db, path, from, to| async move {
                convert_text_to_document(&db, &path, &from, &to).await
            }, Some(db)).await;
        }

        #[tokio::test]
        async fn test_xml_to_all_document_formats() {
            let db = create_test_db().await.unwrap();
            test_conversion("xml", super::DOCUMENT_FORMATS, |db, path, from, to| async move {
                convert_text_to_document(&db, &path, &from, &to).await
            }, Some(db)).await;
        }

        #[tokio::test]
        async fn test_toml_to_all_document_formats() {
            let db = create_test_db().await.unwrap();
            test_conversion("toml", super::DOCUMENT_FORMATS, |db, path, from, to| async move {
                convert_text_to_document(&db, &path, &from, &to).await
            }, Some(db)).await;
        }

        #[tokio::test]
        async fn test_ini_to_all_document_formats() {
            let db = create_test_db().await.unwrap();
            test_conversion("ini", super::DOCUMENT_FORMATS, |db, path, from, to| async move {
                convert_text_to_document(&db, &path, &from, &to).await
            }, Some(db)).await;
        }

        #[tokio::test]
        async fn test_md_to_all_document_formats() {
            let db = create_test_db().await.unwrap();
            test_conversion("md", super::DOCUMENT_FORMATS, |db, path, from, to| async move {
                convert_text_to_document(&db, &path, &from, &to).await
            }, Some(db)).await;
        }

        #[tokio::test]
        async fn test_html_to_all_document_formats() {
            let db = create_test_db().await.unwrap();
            test_conversion("html", super::DOCUMENT_FORMATS, |db, path, from, to| async move {
                convert_text_to_document(&db, &path, &from, &to).await
            }, Some(db)).await;
        }

        #[tokio::test]
        async fn test_txt_to_all_document_formats() {
            let db = create_test_db().await.unwrap();
            test_conversion("txt", super::DOCUMENT_FORMATS, |db, path, from, to| async move {
                convert_text_to_document(&db, &path, &from, &to).await
            }, Some(db)).await;
        }

        #[tokio::test]
        async fn test_rtf_to_all_document_formats() {
            let db = create_test_db().await.unwrap();
            test_conversion("rtf", super::DOCUMENT_FORMATS, |db, path, from, to| async move {
                convert_text_to_document(&db, &path, &from, &to).await
            }, Some(db)).await;
        }
    }
}