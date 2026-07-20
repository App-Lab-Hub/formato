// src/convert/mod.rs

mod csv;
mod xml;
mod ini;
mod md;
mod txt;
mod rtf;



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
use docx_rs::*;

// use rust_xlsxwriter::*;
use std::io::Write;
// use pdfmake_rust::{Document, DocumentNode, Margins, PageSize, PdfMake, TextNode};
// use odtgen::prelude::*;

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
        // (ContentType::Document, ContentType::Document) => {
        //     let result = convert_document_to_document(path, from, to)?;
        //     Ok(ConversionOutput::Inline(result))
        // }
        
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
            // 1. Открываем файл
            let mut file = File::open(path)
                .map_err(|e| format!("Cannot open file: {}", e))?;
            
            // 2. Читаем файл в байты
            let mut buf = Vec::new();
            file.read_to_end(&mut buf)
                .map_err(|e| format!("Cannot read file: {}", e))?;
            
            // 3. Парсим DOCX в структуру
            let doc = read_docx(&buf)
                .map_err(|e| format!("DOCX parse error: {}", e))?;
            
            // 4. Сериализуем в JSON
            let json_value = serde_json::to_value(&doc)
                .map_err(|e| format!("Serialize to JSON error: {}", e))?;
            Ok(json_value)
        }
        "pdf" => {
            // TODO: PDF парсинг
            Err("PDF parsing not implemented yet".to_string())
        }
        "odt" => {
            // TODO: ODT парсинг
            Err("ODT parsing not implemented yet".to_string())
        }
        _ => {
            Err(format!("Unsupported document format: {}", from))
        }
    }
}

fn convert_document_to_text(path: &str, from: &str, to: &str) -> Result<String, String> {
    let json_value = parse_document(path, from)?;
    stringify(&json_value, to)
}

use serde_json::Value;



fn convert_text_to_document(path: &str, from: &str, to: &str) -> Result<String, String> {
    let input = std::fs::read_to_string(path)
        .map_err(|e| format!("Cannot read file: {e}"))?;

    let value = parse(&input, from)?;
    
    match to {
        "docx" => {
            use docx_rs::*;
            use std::fs::File;
            
            let pretty_json_text = serde_json::to_string_pretty(&value)
                .map_err(|e| format!("JSON serialize error: {}", e))?;

            let mut doc = Docx::new();
            
            for line in pretty_json_text.lines() {
                doc = doc.add_paragraph(Paragraph::new().add_run(Run::new().add_text(line)));
            }

            let hash = calculate_conversion_hash(path, from, to)
                .map_err(|e| format!("Cannot hash file: {}", e))?;
            
            let output_path = get_app_dir_path_with_hash(path, to, &hash)?;
            
            let file = File::create(&output_path)
                .map_err(|e| format!("Cannot create file: {}", e))?;
            
            doc.build()
                .pack(file)
                .map_err(|e| format!("DOCX pack error: {}", e))?;

            Ok(output_path)
        }
        "pdf" => {

                        
            use pdfmake_rust::{Document, DocumentNode, Margins, PageSize, PdfMake, TextNode};
            let text = serde_json::to_string_pretty(&value)
                .map_err(|e| format!("JSON serialize error: {}", e))?;
            
            let doc = Document::builder()
                .page_size(PageSize::a4())
                .page_margins(Margins::all(40.0))
                .content(DocumentNode::Text(TextNode::new(text)))
                .build();

            let pdf = PdfMake::new();
            let bytes = pdf.render(&doc)
                .map_err(|e| format!("PDF render error: {}", e))?;
            
            let hash = calculate_conversion_hash(path, from, to)
                .map_err(|e| format!("Cannot hash file: {}", e))?;
            
            let output_path = get_app_dir_path_with_hash(path, to, &hash)?;
            
            let mut file = File::create(&output_path)
                .map_err(|e| format!("Cannot create file: {}", e))?;
            
            file.write_all(&bytes)
                .map_err(|e| format!("Cannot write file: {}", e))?;
            
            Ok(output_path)
        }
        "xlsx" => {
            use rust_xlsxwriter::*;
            
            // Рекурсивно собираем все ключи
            fn collect_keys(value: &Value, prefix: &str, keys: &mut Vec<String>) {
                match value {
                    Value::Object(obj) => {
                        for (key, val) in obj {
                            let new_prefix = if prefix.is_empty() {
                                key.clone()
                            } else {
                                format!("{}.{}", prefix, key)
                            };
                            collect_keys(val, &new_prefix, keys);
                        }
                    }
                    Value::Array(arr) => {
                        if let Some(first) = arr.first() {
                            collect_keys(first, prefix, keys);
                        }
                    }
                    _ => {
                        if !keys.contains(&prefix.to_string()) {
                            keys.push(prefix.to_string());
                        }
                    }
                }
            }
            
            // Преобразуем JSON в плоские строки
            fn flatten_json(value: &Value, prefix: &str) -> Vec<String> {
                match value {
                    Value::Object(obj) => {
                        let mut row = Vec::new();
                        for (key, val) in obj {
                            let new_prefix = if prefix.is_empty() {
                                key.clone()
                            } else {
                                format!("{}.{}", prefix, key)
                            };
                            
                            if let Value::Object(_) = val {
                                let nested = flatten_json(val, &new_prefix);
                                row.extend(nested);
                            } else if let Value::Array(arr) = val {
                                if let Some(first) = arr.first() {
                                    let nested = flatten_json(first, &new_prefix);
                                    row.extend(nested);
                                } else {
                                    row.push("[]".to_string());
                                }
                            } else {
                                row.push(val.to_string());
                            }
                        }
                        row
                    }
                    Value::Array(arr) => {
                        let mut rows = Vec::new();
                        for item in arr {
                            let row = flatten_json(item, prefix);
                            rows.extend(row);
                        }
                        rows
                    }
                    _ => {
                        vec![value.to_string()]
                    }
                }
            }
            
            // Определяем структуру для заголовков
            let mut headers = Vec::new();
            collect_keys(&value, "", &mut headers);
            
            // Преобразуем JSON в строки
            let rows = flatten_json(&value, "");
            
            // Создаём XLSX
            let mut workbook = Workbook::new();
            let worksheet = workbook.add_worksheet();
            
            // Заголовки
            for (col_idx, header) in headers.iter().enumerate() {
                worksheet.write_string(0, col_idx as u16, header.as_str())
                    .map_err(|e| format!("XLSX write error: {}", e))?;
            }
            
            // Данные (1 строка — значения)
            for (col_idx, cell) in rows.iter().enumerate() {
                worksheet.write_string(1, col_idx as u16, cell.as_str())
                    .map_err(|e| format!("XLSX write error: {}", e))?;
            }
            
            // Для массивов объектов — каждая строка отдельно
            if let Value::Array(arr) = &value {
                let mut first = true;
                let mut row_idx = 2;
                for item in arr {
                    if let Value::Object(obj) = item {
                        let row_data = flatten_json(&Value::Object(obj.clone()), "");
                        let current_row = if first { 1 } else { row_idx };
                        for (col_idx, cell) in row_data.iter().enumerate() {
                            worksheet.write_string(current_row, col_idx as u16, cell.as_str())
                                .map_err(|e| format!("XLSX write error: {}", e))?;
                        }
                        first = false;
                        row_idx += 1;
                    }
                }
            }
            
            // Автоподгонка
            for col_idx in 0..headers.len() {
                worksheet.set_column_width(col_idx as u16, 20)
                    .map_err(|e| format!("XLSX set column width error: {}", e))?;
            }
            
            // Сохраняем в буфер
            let buffer = workbook.save_to_buffer()
                .map_err(|e| format!("Failed to save XLSX: {}", e))?;
            
            let hash = calculate_conversion_hash(path, from, to)
                .map_err(|e| format!("Cannot hash file: {}", e))?;
            
            let output_path = get_app_dir_path_with_hash(path, to, &hash)?;
            let mut file = File::create(&output_path)
                .map_err(|e| format!("Cannot create file: {}", e))?;
            
            file.write_all(&buffer)
                .map_err(|e| format!("Cannot write file: {}", e))?;
            
            Ok(output_path)
        }
            
        "odt" => {
        use odtgen::prelude::*;

        let text = serde_json::to_string_pretty(&value)
            .map_err(|e| format!("JSON serialize error: {}", e))?;
        
        let mut doc = Document::new();
        doc.body.add(Paragraph::from_text_and_style(&text, "Standard"));
        
        // Сохраняем как FODT (OpenDocument Flat XML)
        let temp_path = "temp.fodt";
        let mut file = File::create(temp_path)
            .map_err(|e| format!("Cannot create file: {}", e))?;
        
        doc.generate_fodt(&mut file)
            .map_err(|e| format!("ODT generation error: {}", e))?;
        
        let fodt_content = std::fs::read_to_string(temp_path)
            .map_err(|e| format!("Cannot read FODT: {}", e))?;
        
        let hash = calculate_conversion_hash(path, from, to)
            .map_err(|e| format!("Cannot hash file: {}", e))?;
        
        let output_path = get_app_dir_path_with_hash(path, to, &hash)?;
        let mut output_file = File::create(&output_path)
            .map_err(|e| format!("Cannot create file: {}", e))?;
        
        output_file.write_all(fodt_content.as_bytes())
            .map_err(|e| format!("Cannot write file: {}", e))?;
        
        let _ = std::fs::remove_file(temp_path);
        
        Ok(output_path)
        // Err("".to_string())
        }
        _ => {
            stringify(&value, to)
        }
    }
}


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

/// Document → Document
fn convert_document_to_document(path: &str, from: &str, to: &str) -> Result<String, String> {
    // TODO: Использовать pandoc
    Err(format!("Document to document conversion from {} to {} not implemented yet", from, to))
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