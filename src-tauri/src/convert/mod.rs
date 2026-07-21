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
use crate::convert::rtf::{parse_rtf, stringify_rtf};
use crate::convert::pdf::{stringify_pdf, parse_pdf};
// use crate::convert::pdf::{stringify_pdf};

use crate::convert::docx::{stringify_docx, parse_docx};
use crate::convert::odt::{stringify_odt, parse_odt};
use crate::convert::xlsx::{stringify_xlsx, parse_xlsx};

use local_utils::{extract_text_from_pdf, generate_pdf, run_pandoc, create_document_from_text, write_temp_file, xlsx_to_html};

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
    path: &str,
    from: &str,
    to: &str,
    from_type: &str,
    to_type: &str,
) -> Result<ConversionOutput, String> {
    let from_type: ContentType = from_type.to_string().into();
    let to_type: ContentType = to_type.to_string().into();
    
    match (from_type, to_type) {
        // Text → Text — inline
        (ContentType::Text, ContentType::Text) => {
            let result = convert_text_to_text(path, from, to)?;
            Ok(ConversionOutput::Inline(result))
        }
        
        // Text → Document — inline (создаём документ из текста)
        (ContentType::Text, ContentType::Document) => {
            let result = convert_text_to_document(path, from, to)?;
            Ok(ConversionOutput::Save(result))
        }
        
        // Document → Text — inline (извлекаем текст из документа)
        (ContentType::Document, ContentType::Text) => {
            let result = convert_document_to_text(path, from, to)?;
            Ok(ConversionOutput::Inline(result))
        }
        
        // // Document → Document — inline
        (ContentType::Document, ContentType::Document) => {
            let result = convert_document_to_document(path, from, to)?;
            Ok(ConversionOutput::Save(result))
        }
        
        // Image → Image — сохраняем в файл
        (ContentType::Image, ContentType::Image) => {
            let result = convert_image_to_image(path, from, to)?;
            Ok(ConversionOutput::Save(result))
        }
        
        // Audio → Audio — сохраняем в файл
        (ContentType::Audio, ContentType::Audio) => {
            let result = convert_audio_to_audio(path, from, to)?;
            Ok(ConversionOutput::Save(result))
        }
        
        // Video → Video — сохраняем в файл
        (ContentType::Video, ContentType::Video) => {
            let result = convert_video_to_video(path, from, to)?;
            Ok(ConversionOutput::Save(result))
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
fn convert_text_to_text(path: &str, from: &str, to: &str) -> Result<String, String> {
    let input = std::fs::read_to_string(path)
        .map_err(|e| format!("Cannot read file: {e}"))?;
    let value = parse(&input, from)?;
    stringify(&value, to)
}

fn parse_document(path: &str, from: &str) -> Result<Json, String> {
    match from {
        "docx" => {
            parse_docx(path)
        }
        "pdf" => {
            // Err("GG".to_string())
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
fn stringify_document(value:&Json, path: &str, from: &str, to: &str) -> Result<String, String> {
    match to {
        "docx" => {
            stringify_docx(value, path,from, to)
        }
        "pdf" => {
            stringify_pdf(value, path,from, to)

        }
        "xlsx" => {
            stringify_xlsx(value, path,from, to)
        }
            
        "odt" => {
            stringify_odt(value, path,from, to)
 
        }
        
        _ => {
            stringify(value, to)
        }
    }
}



fn convert_document_to_text(path: &str, from: &str, to: &str) -> Result<String, String> {
    let json_value = parse_document(path, from)?;
    stringify(&json_value, to)
}




fn convert_text_to_document(path: &str, from: &str, to: &str) -> Result<String, String> {
    let input = std::fs::read_to_string(path)
        .map_err(|e| format!("Cannot read file: {e}"))?;

    let value = parse(&input, from)?;
    stringify_document(&value, path, from, to)

}




// ========================================================================================================================
// ========================================================================================================================
// ========================================================================================================================

/// Document → Document
fn convert_document_to_document(path: &str, from: &str, to: &str) -> Result<String, String> {
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

    // Прямые конвертации с сохранением форматирования
    match (from, to) {
        // ---------- DOCX ↔ ODT ----------
        ("docx", "odt") => {
            let out = out_path("odt")?;
            run_pandoc(&[path, "-t", "odt", "-o", &out])?;
            Ok(out)
        }
        ("odt", "docx") => {
            let out = out_path("docx")?;
            run_pandoc(&[path, "-t", "docx", "-o", &out])?;
            Ok(out)
        }

        // ---------- DOCX/ODT/XLSX → PDF (через общую функцию) ----------
        ("docx", "pdf") | ("odt", "pdf") => {
            let out = out_path("pdf")?;
            generate_pdf(path, &out)?;
            Ok(out)
        }

        ("xlsx", "docx") => {
            let out = out_path("docx")?;
            // Пробуем через Pandoc
            if let Err(e) = run_pandoc(&[path, "-t", "docx", "-o", &out]) {
                // Если Pandoc не справился - fallback через HTML
                eprintln!("Pandoc failed for XLSX → DOCX: {}, trying fallback...", e);
                let html = xlsx_to_html(path)?;
                let temp_html = write_temp_file(&html)?;
                run_pandoc(&[&temp_html, "-t", "docx", "-o", &out])?;
            }
            Ok(out)
        }
        ("xlsx", "odt") => {
            let out = out_path("odt")?;
            if let Err(e) = run_pandoc(&[path, "-t", "odt", "-o", &out]) {
                eprintln!("Pandoc failed for XLSX → ODT: {}, trying fallback...", e);
                let html = xlsx_to_html(path)?;
                let temp_html = write_temp_file(&html)?;
                run_pandoc(&[&temp_html, "-t", "odt", "-o", &out])?;
            }
            Ok(out)
        }
        ("xlsx", "pdf") => {
            let out = out_path("pdf")?;
            if let Err(e) = generate_pdf(path, &out) {
                eprintln!("Pandoc failed for XLSX → PDF: {}, trying fallback...", e);
                let html = xlsx_to_html(path)?;
                let temp_html = write_temp_file(&html)?;
                generate_pdf(&temp_html, &out)?;
            }
            Ok(out)
        }

        // ---------- PDF → DOCX / ODT / XLSX (извлечение текста) ----------
        ("pdf", "docx") => {
            let text = extract_text_from_pdf(path)?;
            let out = out_path("docx")?;
            create_document_from_text(&text, "docx", &out)?;
            Ok(out)
        }
        ("pdf", "odt") => {
            let text = extract_text_from_pdf(path)?;
            let out = out_path("odt")?;
            create_document_from_text(&text, "odt", &out)?;
            Ok(out)
        }
        ("pdf", "xlsx") => {
            let text = extract_text_from_pdf(path)?;
            let out = out_path("xlsx")?;
            create_document_from_text(&text, "xlsx", &out)?;
            Ok(out)
        }

        // ---------- DOCX/ODT → XLSX (через fallback) ----------
        // ("docx", "xlsx") | ("odt", "xlsx") => {
        //     let value = parse_document(path, from)?;
        //     stringify_document(&value, path, from, to)
        // }
        ("docx", "xlsx") => {
            let out = out_path("xlsx")?;
            let doc = office_oxide::Document::open(path)
                .map_err(|e| format!("Open DOCX failed: {}", e))?;
            doc.save_as(&out)
                .map_err(|e| format!("Save as XLSX failed: {}", e))?;
            Ok(out)
        }
        ("odt", "xlsx") => {
            let out = out_path("xlsx")?;
            
            // Создаем временный DOCX файл (авто-удаление)
            let temp_docx = NamedTempFile::new()
                .map_err(|e| format!("Create temp DOCX: {}", e))?;
            let temp_docx_path = temp_docx.path().to_str().unwrap().to_string();
            
            // ODT → DOCX через Pandoc
            run_pandoc(&[path, "-t", "docx", "-o", &temp_docx_path])?;
            
            // DOCX → XLSX через office_oxide
            let doc = office_oxide::Document::open(&temp_docx_path)
                .map_err(|e| format!("Open temp DOCX: {}", e))?;
            doc.save_as(&out)
                .map_err(|e| format!("Save as XLSX: {}", e))?;
            
            // temp_docx удаляется автоматически здесь
            Ok(out)
        }
        // ---------- ВСЕ ОСТАЛЬНЫЕ ПАРЫ — FALLBACK ----------
        _ => {
            let value = parse_document(path, from)?;
            stringify_document(&value, path, from, to)
        }
    }
}
// ========================================================================================================================
// ========================================================================================================================
// ========================================================================================================================



/// Image → Image
fn convert_image_to_image(path: &str, from: &str, to: &str) -> Result<String, String> {
    // TODO: Использовать image crate или ImageMagick
    Err(format!("Image to image conversion from {} to {} not implemented yet", from, to))
}

/// Audio → Audio
fn convert_audio_to_audio(path: &str, from: &str, to: &str) -> Result<String, String> {
    // TODO: Использовать ffmpeg или symphonia
    Err(format!("Audio to audio conversion from {} to {} not implemented yet", from, to))
}

/// Video → Video
fn convert_video_to_video(path: &str, from: &str, to: &str) -> Result<String, String> {
    // TODO: Использовать ffmpeg
    Err(format!("Video to video conversion from {} to {} not implemented yet", from, to))
}



// ============================================================
// ПАРСЕРЫ И СЕРИАЛИЗАТОРЫ
// ============================================================

fn parse(input: &str, format: &str) -> Result<Json, String> {
    match format {
        "json" => serde_json::from_str(input).map_err(|e| format!("JSON: {e}")),
        "yaml" | "yml" => serde_yaml::from_str(input).map_err(|e| format!("YAML: {e}")),
        "toml" => toml::from_str(input).map_err(|e| format!("TOML: {e}")),
        "xml" => parse_xml(input),
        "ini" => parse_ini(input),
        "md" => parse_markdown(input),
        "csv" => parse_csv(input),
        "html" => parse_html(input),
        "txt" | "text" => parse_txt(input),
        "rtf" => parse_rtf(input),
        _ => Err(format!("Unsupported: {format}")),
    }
}

fn stringify(value: &Json, format: &str) -> Result<String, String> {
    match format {
        "json" => serde_json::to_string_pretty(value).map_err(|e| format!("JSON: {e}")),
        "yaml" | "yml" => serde_yaml::to_string(value).map_err(|e| format!("YAML: {e}")),
        "toml" => {
            let value_for_toml = match value {
                Json::Array(arr) => {
                    let mut map = serde_json::Map::new();
                    map.insert("data".to_string(), Json::Array(arr.clone()));
                    Json::Object(map)
                }
                _ => value.clone(),
            };
            toml::to_string_pretty(&value_for_toml).map_err(|e| format!("TOML: {e}"))
        }
        "xml" => stringify_xml(value).map_err(|e| format!("XML: {e}")),
        "csv" => stringify_csv(value),
        "ini" => stringify_ini(value),
        "html" => Ok(convert_to_html(value)),
        "md" => stringify_markdown(value),
        "txt" | "text" => stringify_txt(value),
        "rtf" => stringify_rtf(value),


        _ => Err(format!("Unsupported: {format}")),
    }
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
    
    let (path_clone, from_clone, to_clone, from_type_clone, to_type_clone) = 
        (path.clone(), from.clone(), to.clone(), fromType.clone(), toType.clone());
    
    let output = tokio::task::spawn_blocking(move || {
        convert(&path_clone, &from_clone, &to_clone, &from_type_clone, &to_type_clone)
    }).await.map_err(|e| format!("Task join error: {e}"))??;
    
    // Обрабатываем результат в зависимости от типа
    match output {
        ConversionOutput::Inline(content) => {
            // inline — сохраняем в файл и кешируем
            let saved_path = save_to_app_dir(&content, &path, &to, &input_hash)?;
            let extension = Path::new(&saved_path)
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_string());

            if enable_cache {
                db::save_conversion(db, &input_hash, &saved_path).await?;
            }

            Ok(ConvertResult {
                success: true,
                content: saved_path,
                hash: Some(input_hash),
                extension,
                error: None,
            })
        }
        ConversionOutput::Save(saved_path) => {
            // уже сохранён, просто возвращаем путь
            let extension = Path::new(&saved_path)
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_string());

            if enable_cache {
                db::save_conversion(db, &input_hash, &saved_path).await?;
            }

            Ok(ConvertResult {
                success: true,
                content: saved_path,
                hash: Some(input_hash),
                extension,
                error: None,
            })
        }
    }
}

#[tauri::command]
pub async fn read_file_content(path: String) -> Result<String, String> {
    tokio::fs::read_to_string(&path).await.map_err(|e| format!("Cannot read file: {e}"))
}

#[tauri::command]
pub async fn open_file(path: String) -> Result<(), String> {
    opener::open(&path).map_err(|e| format!("Cannot open file: {e}"))
}