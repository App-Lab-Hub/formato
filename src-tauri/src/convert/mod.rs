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

pub fn convert(
    app_handle: &tauri::AppHandle,
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
            convert_text_to_text(app_handle, path, from, to)
        }
        
        // Text → Document
        (ContentType::Text, ContentType::Document) => {
            convert_text_to_document(app_handle, path, from, to)
        }
        
        // Text → Audio
        (ContentType::Text, ContentType::Audio) => {
            convert_text_to_audio(path, from, to)
        }

        // Document → Text
        (ContentType::Document, ContentType::Text) => {
            convert_document_to_text(path, from, to)
        }
        
        // Document → Document
        (ContentType::Document, ContentType::Document) => {
            convert_document_to_document(path, from, to)
        }

        // Document → Audio
        (ContentType::Document, ContentType::Audio) => {
            convert_document_to_audio(path, from, to)
        }
        
        // Image → Image
        (ContentType::Image, ContentType::Image) => {
            convert_image_to_image(path, from, to)
        }

        // Image → Text
        (ContentType::Image, ContentType::Text) => {
            convert_image_to_text(path, from, to)
        }
        (ContentType::Image, ContentType::Document) => {
            convert_image_to_document(app_handle, path, from, to)
        }
        
        // Audio → Audio
        (ContentType::Audio, ContentType::Audio) => {
            convert_audio_to_audio(path, from, to)
        }

        // Audio → Text
        (ContentType::Audio, ContentType::Text) => {
            convert_audio_to_text(path, from, to)
        }

        // Video → Video
        (ContentType::Video, ContentType::Video) => {
            convert_video_to_video(path, from, to)
        }

        // Video → Audio
        (ContentType::Video, ContentType::Audio) => {
            convert_video_to_audio(path, from, to)
        }
        // Video → Text (извлекаем аудио, потом распознаем)
        (ContentType::Video, ContentType::Text) => {
            convert_video_to_text(path, from, to)
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

/// Text → Text
/// Text → Text
fn convert_text_to_text(app_handle: &tauri::AppHandle, path: &str, from: &str, to: &str) -> Result<String, String> {
    let input = std::fs::read_to_string(path)
        .map_err(|e| format!("Cannot read file: {e}"))?;

    // Если конвертируем в RTF - используем stringify_rtf напрямую
    if to == "rtf" {
        // RTF требует DOCX как промежуточный формат
        // Сначала создаем DOCX из текста
        let docx_path = stringify_document(app_handle, &input, path, from, "docx")?;
        
        // Затем конвертируем DOCX в RTF
        let rtf_path = rtf::convert_docx_to_rtf(&docx_path, path, to)?;
        
        // Удаляем временный DOCX
        let _ = std::fs::remove_file(&docx_path);
        
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
pub fn stringify_document(
    app_handle: &tauri::AppHandle,
    text: &str,
    path: &str,
    from: &str,
    to: &str,
) -> Result<String, String> {
    match to {
        "docx" => stringify_docx(text, path, from, to),
        "pdf" => stringify_pdf(app_handle, text, path, from, to),
        "xlsx" => stringify_xlsx(text, path, from, to),
        "odt" => stringify_odt(text, path, from, to),
        _ => {
            let hash = calculate_conversion_hash(path, from, to)
                .map_err(|e| format!("Hash error: {}", e))?;
            let output_path = save_to_app_dir(text, path, to, &hash)?;
            Ok(output_path)
        }
    }
}


fn convert_document_to_text(path: &str, from: &str, to: &str) -> Result<String, String> {
    // Если конвертируем из документа в RTF
    if to == "rtf" {
        // 1. Конвертируем документ в DOCX через convert_document_to_document
        let docx_path = convert_document_to_document(path, from, "docx")?;
        
        // 2. Конвертируем DOCX в RTF через rtf::convert_docx_to_rtf
        let rtf_path = rtf::convert_docx_to_rtf(&docx_path, path, to)?;
        
        // 3. Удаляем временный DOCX
        let _ = std::fs::remove_file(&docx_path);
        
        return Ok(rtf_path);
    }
    
    // Для остальных форматов - стандартная логика
    let json_value = parse_document(path, from)?;
    stringify(&json_value, to, path, from)
}

// Функция-обертка (уже есть в вашем коде)
fn convert_text_to_audio(path: &str, from: &str, to: &str) -> Result<String, String> {
    text_to_audio::convert_text_to_audio(path, from, to)
}



/// Text → Document
fn convert_text_to_document(
    app_handle: &tauri::AppHandle,
    path: &str, 
    from: &str, 
    to: &str
) -> Result<String, String> {
    let input = std::fs::read_to_string(path)
        .map_err(|e| format!("Cannot read file: {e}"))?;
    
    stringify_document(app_handle, &input, path, from, to)
}

// Функция-обертка:
fn convert_image_to_text(path: &str, from: &str, to: &str) -> Result<String, String> {
    image_to_text::convert_image_to_text(path, from, to)
}
fn convert_image_to_document(app_handle: &tauri::AppHandle, path: &str, from: &str, to: &str) -> Result<String, String> {
    image_to_document::convert_image_to_document(app_handle, path, from, to)
}
// ========================================================================================================================
// ========================================================================================================================
// ========================================================================================================================

/// Document → Document
pub fn convert_document_to_document(path: &str, from: &str, to: &str) -> Result<String, String> {
    // Если форматы совпадают – просто возвращаем исходный путь
    if from == to {
        return Ok(path.to_string());
    }

    // Вычисляем хеш для именования выходного файла
    let hash = calculate_conversion_hash(path, from, to)
        .map_err(|e| format!("Hash error: {e}"))?;

    // Вспомогательная функция для генерации пути
    let out_path = |ext: &str| -> Result<String, String> {
        get_app_dir_path_with_hash(path, ext, &hash)
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
            // DOCX → XLSX через office_oxide
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
            
            // ODT → DOCX через soffice
            let docx_path = out_path("docx")?;
            convert_with_soffice_explicit(path, &docx_path)?;
            
            // DOCX → XLSX через office_oxide
            let doc = office_oxide::Document::open(&docx_path)
                .map_err(|e| format!("Open DOCX: {}", e))?;
            doc.save_as(&out)
                .map_err(|e| format!("Save as XLSX: {}", e))?;
            
            // Удаляем временный DOCX
            let _ = std::fs::remove_file(&docx_path);
            
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
            
            // Шаг 1: XLSX → DOCX через office_oxide
            let docx_path = out_path("docx")?;
            let doc = office_oxide::Document::open(path)
                .map_err(|e| format!("Open XLSX: {}", e))?;
            doc.save_as(&docx_path)
                .map_err(|e| format!("XLSX to DOCX: {}", e))?;
            
            // Шаг 2: DOCX → ODT через soffice
            convert_with_soffice_explicit(&docx_path, &out)?;
            
            // Удаляем временный DOCX
            let _ = std::fs::remove_file(&docx_path);
            
            Ok(out)
        }
        ("xlsx", "pdf") => {
            let out = out_path("pdf")?;
            convert_with_soffice_explicit(path, &out)?;
            Ok(out)
        }

        // // ---------- PDF ----------
        // ("pdf", "docx") => {
        //     let out = out_path("docx")?;
        //     convert_with_soffice_explicit(path, &out)?;
        //     Ok(out)
        // }
        // ("pdf", "odt") => {
        //     let out = out_path("odt")?;
        //     convert_with_soffice_explicit(path, &out)?;
        //     Ok(out)
        // }
        // ("pdf", "xlsx") => {
        //     let out = out_path("xlsx")?;
        //     // PDF → XLSX через office_oxide
        //     let doc = office_oxide::Document::open(path)
        //         .map_err(|e| format!("Open PDF: {}", e))?;
        //     doc.save_as(&out)
        //         .map_err(|e| format!("Save as XLSX: {}", e))?;
        //     Ok(out)
        // }

        // ---------- ВСЕ ОСТАЛЬНЫЕ ПАРЫ — FALLBACK ----------
        _ => {
            Err("Unsupported conversion".to_string())
        }
    }
}

// ========================================================================================================================
// ========================================================================================================================
// ========================================================================================================================



/// Image → Image
use image::{ImageFormat, ImageReader};


/// Конвертация изображений между поддерживаемыми форматами
fn convert_image_to_image(path: &str, from: &str, to: &str) -> Result<String, String> {
    // Открываем и декодируем изображение
    let img = ImageReader::open(path)
        .map_err(|e| format!("Cannot open image: {}", e))?
        .decode()
        .map_err(|e| format!("Cannot decode image: {}", e))?;
    
    // Определяем формат
    let format = match to.to_lowercase().as_str() {
        "jpg" | "jpeg" => ImageFormat::Jpeg,
        "png" => ImageFormat::Png,
        "webp" => ImageFormat::WebP,
        "avif" => ImageFormat::Avif,
        "gif" => ImageFormat::Gif,
        "bmp" => ImageFormat::Bmp,
        "tiff" | "tif" => ImageFormat::Tiff,
        "ico" => ImageFormat::Ico,
        _ => return Err(format!("Unsupported output format: {}", to)),
    };
    
    // Получаем хеш и путь
    let hash = calculate_conversion_hash(path, from, to)
        .map_err(|e| format!("Hash error: {}", e))?;
    let out_path = get_app_dir_path_with_hash(path, &to, &hash)?;
    
    // Сохраняем
    img.save_with_format(&out_path, format)
        .map_err(|e| format!("Cannot save image: {}", e))?;
    
    Ok(out_path)
}

fn convert_audio_to_audio(path: &str, from: &str, to: &str) -> Result<String, String> {
    audio::convert_audio_to_audio(path, from, to)
}

/// Video → Video
fn convert_video_to_video(path: &str, from: &str, to: &str) -> Result<String, String> {
    video::convert_video_to_video(path, from, to)
}
/// Video → Audio — извлекаем аудио дорожку и конвертируем в целевой формат
fn convert_video_to_audio(path: &str, from: &str, to: &str) -> Result<String, String> {
    video::convert_video_to_audio(path, from, to)
}

fn convert_document_to_audio(path: &str, from: &str, to: &str) -> Result<String, String> {
    document_to_audio::convert_document_to_audio(path, from, to)
}
// Функции-обертки:
fn convert_audio_to_text(path: &str, from: &str, to: &str) -> Result<String, String> {
    audio_to_text::convert_audio_to_text(path, from, to)
}

fn convert_video_to_text(path: &str, from: &str, to: &str) -> Result<String, String> {
    // video_to_text::convert_video_to_text(path, from, to)
    Err("gg".to_string())
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
        .map_err(|e| format!("Hash error: {}", e))?;
    let output_path = save_to_app_dir(&content, path, format, &hash)?;
    
    Ok(output_path)
}



// ============================================================
// ВСПОМОГАТЕЛЬНЫЕ ФУНКЦИИ
// ============================================================

pub fn save_to_app_dir(content: &str, original_path: &str, to: &str, hash: &str) -> Result<String, String> {
    let input_path = PathBuf::from(original_path);
    let stem = input_path.file_stem().and_then(|s| s.to_str()).unwrap_or("converted");
    
    let output_dir = converted_dir();
    let output_path = output_dir.join(format!("{}_{}.{}", stem, hash, to));
    std::fs::write(&output_path, content).map_err(|e| format!("Cannot write file: {e}"))?;
    
    Ok(output_path.to_string_lossy().to_string())
}

pub fn get_app_dir_path_with_hash(original_path: &str, to: &str, hash: &str) -> Result<String, String> {
    let input_path = PathBuf::from(original_path);
    let stem = input_path.file_stem().and_then(|s| s.to_str()).unwrap_or("converted");
    let output_dir = converted_dir();
    let output_path = output_dir.join(format!("{}_{}.{}", stem, hash, to));
    
    Ok(output_path.to_string_lossy().to_string())
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
    app_handle: tauri::AppHandle,  // <-- добавляем AppHandle
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
    
    let output_path = tokio::task::spawn_blocking(move || {
        convert(&app_handle, &path_clone, &from_clone, &to_clone, &from_type_clone, &to_type_clone)
    }).await.map_err(|e| format!("Task join error: {e}"))??;

    // Сохраняем в кеш
    if enable_cache {
        db::save_conversion(db, &input_hash, &output_path).await?;
    }

    let extension = Path::new(&output_path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_string());

    Ok(ConvertResult {
        success: true,
        content: output_path,
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
pub async fn open_file(path: String) -> Result<(), String> {
    opener::open(&path).map_err(|e| format!("Cannot open file: {e}"))
}