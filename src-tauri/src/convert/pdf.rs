                
use pdfmake_rust::{Document, DocumentNode, Margins, PageSize, PdfMake, TextNode};
use serde_json::{Value as Json};
use crate::convert::calculate_conversion_hash;
use crate::convert::get_app_dir_path_with_hash;
use std::fs::File;
use std::io::Write;


pub fn stringify_pdf(value: &Json, path: &str, from: &str, to: &str) -> Result<String, String> {



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



use pdf_extract::extract_text;
use encoding_rs::WINDOWS_1251;

/// Парсит PDF в JSON с правильной кодировкой
pub fn parse_pdf(path: &str) -> Result<Json, String> {
    // Читаем файл как байты
    let bytes = std::fs::read(path)
        .map_err(|e| format!("Cannot read file: {}", e))?;
    
    // Пробуем извлечь текст с разными кодировками
    let full_text = match extract_text(path) {
        Ok(text) => {
            // Проверяем, есть ли кракозябры (признак неправильной кодировки)
            if text.contains('�') || text.chars().any(|c| c as u32 > 0xFFFF) {
                // Пробуем декодировать как CP1251
                let (decoded, _, _) = WINDOWS_1251.decode(&bytes);
                decoded.to_string()
            } else {
                text
            }
        }
        Err(_) => {
            // Если extract_text не сработал, пробуем напрямую через encoding
            let (decoded, _, _) = WINDOWS_1251.decode(&bytes);
            decoded.to_string()
        }
    };
    
    // Очищаем текст от мусора
    let clean_text = full_text
        .replace(['\u{0}', '\u{1}', '\u{2}', '\u{3}', '\u{4}', '\u{5}', '\u{6}', '\u{7}', '\u{8}', '\u{9}', '\u{10}', '\u{11}', '\u{12}', '\u{13}', '\u{14}', '\u{15}', '\u{16}', '\u{17}', '\u{18}', '\u{19}', '\u{20}', '\u{21}', '\u{22}', '\u{23}', '\u{24}', '\u{25}', '\u{26}', '\u{27}', '\u{28}', '\u{29}', '\u{30}', '\u{31}'], "");
    
    // Разбиваем на абзацы
    let paragraphs: Vec<String> = clean_text
        .split("\n\n")
        .filter(|p| !p.trim().is_empty())
        .map(|p| p.trim().to_string())
        .collect();
    
    let lines: Vec<String> = clean_text
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.trim().to_string())
        .collect();
    
    let mut map = serde_json::Map::new();
    
    map.insert("text".to_string(), Json::String(clean_text.clone()));
    
    if !paragraphs.is_empty() {
        map.insert("paragraphs".to_string(), Json::Array(
            paragraphs.into_iter().map(Json::String).collect()
        ));
    }
    
    if !lines.is_empty() {
        map.insert("lines".to_string(), Json::Array(
            lines.into_iter().map(Json::String).collect()
        ));
    }
    
    map.insert("line_count".to_string(), Json::Number(serde_json::Number::from(clean_text.lines().count())));
    map.insert("char_count".to_string(), Json::Number(serde_json::Number::from(clean_text.chars().count())));
    map.insert("word_count".to_string(), Json::Number(serde_json::Number::from(clean_text.split_whitespace().count())));
    
    Ok(Json::Object(map))
}