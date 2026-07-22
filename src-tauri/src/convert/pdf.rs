                
use pdfmake_rust::{Document, DocumentNode, Margins, PageSize, PdfMake, TextNode};
use serde_json::{Value as Json,Map, Number};
use crate::convert::calculate_conversion_hash;
use crate::convert::get_app_dir_path_with_hash;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use pdf_extract::extract_text;
// use crate::convert::extract_text_from_pdf;
use encoding_rs::WINDOWS_1251;

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



/// Парсит PDF в JSON с автоматическим исправлением кодировки Windows-1251
pub fn parse_pdf(path: &str) -> Result<Json, String> {
    // 1. Извлекаем текст средствами pdf-extract
    let path_buf = Path::new(path);
    let extracted_text = extract_text(path_buf)
        .map_err(|e| format!("Не удалось прочитать или распарсить PDF: {}", e))?;
    
    // let extracted_text = extract_text_from_pdf(path)
        // .map_err(|e| format!("Не удалось прочитать или распарсить PDF: {}", e))?;
    

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
    
    // 4. Разбиваем текст на логические блоки
    let paragraphs: Vec<Json> = clean_text
        .split("\n\n")
        .filter(|p| !p.trim().is_empty())
        .map(|p| Json::String(p.trim().to_string()))
        .collect();
    
    let lines: Vec<Json> = clean_text
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| Json::String(l.trim().to_string()))
        .collect();
    
    // 5. Собираем финальный JSON-объект
    let mut map = Map::new();
    
    map.insert("text".to_string(), Json::String(clean_text.clone()));
    
    if !paragraphs.is_empty() {
        map.insert("paragraphs".to_string(), Json::Array(paragraphs));
    }
    
    if !lines.is_empty() {
        map.insert("lines".to_string(), Json::Array(lines));
    }
    
    // Считаем метрики
    let line_count = clean_text.lines().count();
    let char_count = clean_text.chars().count();
    let word_count = clean_text.split_whitespace().count();
    
    map.insert("line_count".to_string(), Json::Number(Number::from(line_count)));
    map.insert("char_count".to_string(), Json::Number(Number::from(char_count)));
    map.insert("word_count".to_string(), Json::Number(Number::from(word_count)));
    
    Ok(Json::Object(map))
}

/// Вспомогательная функция для детекции "европейских кракозябр" вместо кириллицы
fn has_latin1_krakozyabry(text: &str) -> bool {
    // Если в тексте подозрительно много специфичных символов Latin-1 верхнего регистра (например, À, Í, ß, ÿ),
    // которые часто появляются при неверном чтении Windows-1251
    let krakozyabry_count = text.chars()
        .filter(|&c| {
            let cp = c as u32;
            (192..=255).contains(&cp) // Диапазон кодов, куда обычно бьет CP1251 в Latin-1
        })
        .count();
        
    // Если таких символов больше 3% от всего текста — это ложная кодировка
    if text.is_empty() {
        false
    } else {
        (krakozyabry_count * 100) / text.chars().count() > 3
    }
}