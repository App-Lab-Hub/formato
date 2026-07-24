// src/convert/rtf.rs

use rtf_parser::{Lexer, Parser, RtfDocument};
use serde_json::{Value as Json};
use std::fs;
use std::path::Path;
use std::process::Command;

/// Парсит RTF в JSON с сохранением структуры
pub fn parse_rtf(input: &str) -> Result<Json, String> {
    // Парсим RTF
    let doc = RtfDocument::try_from(input)
        .map_err(|e| format!("RTF parse error: {}", e))?;
    
    // Извлекаем текст
    let text = doc.get_text();
    
    // Создаём структурированный JSON
    let mut map = serde_json::Map::new();
    
    // Основной текст
    map.insert("text".to_string(), Json::String(text.clone()));
    
    // Собираем стилизованные блоки
    let mut blocks = Vec::new();
    for block in &doc.body {
        let mut block_map = serde_json::Map::new();
        
        // Текст
        block_map.insert("text".to_string(), Json::String(block.text.clone()));
        
        // Стиль
        let mut style = serde_json::Map::new();
        style.insert("bold".to_string(), Json::Bool(block.painter.bold));
        style.insert("italic".to_string(), Json::Bool(block.painter.italic));
        style.insert("underline".to_string(), Json::Bool(block.painter.underline));
        style.insert("font_size".to_string(), Json::Number(serde_json::Number::from(block.painter.font_size)));
        style.insert("font_ref".to_string(), Json::Number(serde_json::Number::from(block.painter.font_ref)));
        block_map.insert("style".to_string(), Json::Object(style));
        
        // Выравнивание
        let alignment = match block.paragraph.alignment {
            rtf_parser::Alignment::LeftAligned => "left",
            rtf_parser::Alignment::RightAligned => "right",
            rtf_parser::Alignment::Center => "center",
            rtf_parser::Alignment::Justify => "justify",
        };
        block_map.insert("alignment".to_string(), Json::String(alignment.to_string()));
        
        blocks.push(Json::Object(block_map));
    }
    map.insert("blocks".to_string(), Json::Array(blocks));
    
    // Метаданные
    map.insert("line_count".to_string(), Json::Number(serde_json::Number::from(text.lines().count())));
    map.insert("char_count".to_string(), Json::Number(serde_json::Number::from(text.chars().count())));
    map.insert("block_count".to_string(), Json::Number(serde_json::Number::from(doc.body.len())));
    
    Ok(Json::Object(map))
}


use crate::convert::{stringify_document, calculate_conversion_hash, get_app_dir_path_with_hash};

/// Конвертирует JSON в RTF через DOCX
pub fn stringify_rtf(value: &Json, original_path: &str, from: &str, to: &str) -> Result<String, String> {
    // 1. Создаем DOCX через stringify_document
    let docx_path = stringify_document(value, original_path, from, "docx")?;
    
    if !Path::new(&docx_path).exists() {
        return Err(format!("DOCX file not created: {}", docx_path));
    }
    
    // 2. Конвертируем DOCX в RTF
    let rtf_path = convert_docx_to_rtf(&docx_path, original_path, to)?;
    
    // 3. Удаляем DOCX
    let _ = fs::remove_file(&docx_path);
    
    Ok(rtf_path)
}

/// Конвертирует DOCX в RTF через soffice
fn convert_docx_to_rtf(docx_path: &str, original_path: &str, to: &str) -> Result<String, String> {
    // Проверяем наличие soffice
    let check = Command::new("soffice")
        .arg("--version")
        .output();
    
    if check.is_err() {
        return Err("soffice not found. Please install LibreOffice.".to_string());
    }

    // Получаем директорию DOCX
    let docx_dir = Path::new(docx_path)
        .parent()
        .ok_or("Invalid docx path")?
        .to_str()
        .ok_or("Invalid docx path")?;

    // Временный RTF с тем же именем
    let docx_stem = Path::new(docx_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or("Invalid docx filename")?;
    
    let temp_rtf = format!("{}/{}.rtf", docx_dir, docx_stem);

    // Конвертируем через soffice
    let status = Command::new("soffice")
        .args(&[
            "--headless",
            "--convert-to", "rtf",
            docx_path,
            "--outdir", docx_dir,
        ])
        .status()
        .map_err(|e| format!("soffice error: {}", e))?;

    if !status.success() {
        return Err("soffice conversion failed".to_string());
    }

    if !Path::new(&temp_rtf).exists() {
        return Err("soffice did not create RTF file".to_string());
    }

    // Перемещаем в нужную папку с хешем
    let hash = calculate_conversion_hash(original_path, "docx", to)
        .map_err(|e| format!("Hash error: {}", e))?;
    
    let final_path = get_app_dir_path_with_hash(original_path, to, &hash)?;

    // Создаем директорию
    if let Some(parent) = Path::new(&final_path).parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Cannot create output dir: {}", e))?;
        }
    }

    // Перемещаем
    fs::rename(&temp_rtf, &final_path)
        .map_err(|e| format!("Cannot rename RTF file: {}", e))?;

    Ok(final_path)
}