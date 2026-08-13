
use serde_json::{Value as Json};
use crate::convert::calculate_conversion_hash;
use crate::convert::get_app_dir_path_with_hash;

use std::path::Path;
use pdf_extract::extract_text;
use encoding_rs::WINDOWS_1251;
use tauri::path::BaseDirectory;
use tauri::Manager;

use sea_orm::DatabaseConnection;

use crate::convert::docx::{stringify_docx};
/// Создает PDF из текстовой строки 
pub async fn stringify_pdf(
    db: &DatabaseConnection,
    text: &str, 
    path: &str, 
    from: &str, 
    to: &str
) -> Result<String, String> {
    // 1. Сначала создаем DOCX из текста
    let docx_path = stringify_docx(text, path, from, "docx")?;
    
    // 2. Конвертируем DOCX в PDF через convert_document_to_document
    let pdf_path = crate::convert::convert_document_to_document(db, &docx_path, "docx", "pdf").await?;
    

    
    Ok(pdf_path)
}

/// Парсит PDF в JSON с автоматическим исправлением кодировки Windows-1251
pub fn parse_pdf(path: &str) -> Result<Json, String> {
    // 1. Извлекаем текст средствами pdf-extract
    let path_buf = Path::new(path);
    let extracted_text = extract_text(path_buf)
        .map_err(|e| format!("Не удалось прочитать или распарсить PDF: {}", e))?;
    
    // 2. Исправляем кодировку если нужно
    let full_text = if extracted_text.contains("\u{FFFD}") || has_latin1_krakozyabry(&extracted_text) {
        // Откатываем сломанный UTF-8 обратно в байты ISO-8859-1 (Latin-1) и перекодируем в CP1251
        let raw_bytes: Vec<u8> = extracted_text.chars().map(|c| c as u8).collect();
        let (decoded, _, _) = WINDOWS_1251.decode(&raw_bytes);
        decoded.into_owned()
    } else {
        extracted_text
    };
    
    // 3. Мягкая очистка текста от непечатных управляющих символов (сохраняем \t, \n, \r)
    let clean_text: String = full_text
        .chars()
        .filter(|&c| {
            let cp = c as u32;
            // Пропускаем стандартные пробельные символы (таб, перевод строки, возврат каретки)
            if cp == 9 || cp == 10 || cp == 13 {
                return true;
            }
            // Удаляем остальные управляющие символы ASCII (0..31) и DEL (127)
            !(cp < 32 || cp == 127)
        })
        .collect();
    
    // 4. Разбиваем текст на параграфы
    let paragraphs: Vec<String> = clean_text
        .split("\n\n")
        .filter(|p| !p.trim().is_empty())
        .map(|p| p.trim().to_string())
        .collect();
    let paragraph_count = paragraphs.len();
    
    // 5. Разбиваем на строки
    let lines: Vec<String> = clean_text
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.trim().to_string())
        .collect();
    let line_count = lines.len();
    
    // 6. Считаем метрики
    let chars: Vec<String> = clean_text.chars().map(|c| c.to_string()).collect();
    let char_count = chars.len();
    let word_count = clean_text.split_whitespace().count();
    
    // 7. Собираем результат в едином формате (как DOCX и ODT)
    let mut result = serde_json::Map::new();
    result.insert("text".to_string(), Json::String(clean_text));
    result.insert("paragraphs".to_string(), Json::Array(paragraphs.into_iter().map(Json::String).collect()));
    result.insert("char_count".to_string(), Json::Number(serde_json::Number::from(char_count)));
    result.insert("word_count".to_string(), Json::Number(serde_json::Number::from(word_count)));
    result.insert("line_count".to_string(), Json::Number(serde_json::Number::from(line_count)));
    result.insert("paragraph_count".to_string(), Json::Number(serde_json::Number::from(paragraph_count)));
    
    // 8. Добавляем строки (lines) если есть
    if !lines.is_empty() {
        result.insert("lines".to_string(), Json::Array(lines.into_iter().map(Json::String).collect()));
    }
    
    Ok(Json::Object(result))
}

/// Вспомогательная функция для детекции "европейских кракозябр" вместо кириллицы
fn has_latin1_krakozyabry(text: &str) -> bool {
    // Если в тексте подозрительно много специфичных символов Latin-1 верхнего регистра
    let krakozyabry_count = text.chars()
        .filter(|&c| {
            let cp = c as u32;
            (192..=255).contains(&cp)
        })
        .count();
        
    if text.is_empty() {
        false
    } else {
        (krakozyabry_count * 100) / text.chars().count() > 3
    }
}