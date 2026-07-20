
use docx_rs::*;
use std::fs::File;
use serde_json::{Value as Json};

use std::io::Read;
use crate::convert::calculate_conversion_hash;
use crate::convert::get_app_dir_path_with_hash;

pub fn stringify_docx(value: &Json, path: &str, from: &str, to: &str) -> Result<String, String> {
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
    

pub fn parse_docx(path: &str) -> Result<Json, String> {
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