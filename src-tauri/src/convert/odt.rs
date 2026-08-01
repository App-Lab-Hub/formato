
use serde_json::{Value as Json};
use crate::convert::calculate_conversion_hash;
use crate::convert::get_app_dir_path_with_hash;
use std::fs::File;
use std::io::Write;
use zip::ZipArchive;
use std::io::Read;
use quick_xml::Reader as XmlReader;
use quick_xml::events::Event;

use lo_writer::{WriterEditor, save_odt};

/// Создает структурированный ODT из текстовой строки с сохранением YAML-отступов
pub fn stringify_odt(text: &str, path: &str, from: &str, to: &str) -> Result<String, String> {
    let mut editor = WriterEditor::new("");

    for line in text.lines() {
        // Считаем количество ведущих пробелов в строке
        let leading_spaces = line.len() - line.trim_start().len();
        
        let processed_line = if leading_spaces > 0 {
            // Заменяем ведущие пробелы на неразрывные пробелы (\u{00A0})
            let spaces = "\u{00A0}".repeat(leading_spaces);
            format!("{}{}", spaces, line.trim_start())
        } else {
            line.to_string()
        };

        // Теперь lo_writer запишет строку, и офисный пакет отобразит все отступы
        editor.push_paragraph(&processed_line);
    }

    let document = editor.document;

    let hash = calculate_conversion_hash(path, from, to)
        .map_err(|e| format!("Cannot hash file: {}", e))?;

    let output_path = get_app_dir_path_with_hash(path, to, &hash, true)?;
    
    save_odt(&output_path, &document)
        .map_err(|e| format!("ODT generation error: {:?}", e))?;

    Ok(output_path)
}





pub fn parse_odt(path: &str) -> Result<Json, String> {
    let file = File::open(path)
        .map_err(|e| format!("Cannot open file: {}", e))?;
    let mut archive = ZipArchive::new(file)
        .map_err(|e| format!("ODT parse error: {}", e))?;
    
    let mut content_xml = String::new();
    
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)
            .map_err(|e| format!("Cannot read ODT entry: {}", e))?;
        let name = file.name().to_string();
        
        if name.starts_with("Thumbnails/") || name.starts_with("Pictures/") || name.starts_with("META-INF/") {
            continue;
        }
        
        if name == "content.xml" {
            file.read_to_string(&mut content_xml)
                .map_err(|e| format!("Cannot read content.xml: {}", e))?;
            break;
        }
    }
    
    if content_xml.is_empty() {
        return Err("No content.xml found in ODT".to_string());
    }
    
    parse_odt_xml(&content_xml)
}

fn parse_odt_xml(xml: &str) -> Result<Json, String> {
    let mut reader = XmlReader::from_str(xml);
    let mut paragraphs: Vec<String> = Vec::new();
    let mut current_text = String::new();
    let mut in_paragraph = false;
    let mut buf = Vec::new();
    
    // Дополнительные структуры для богатого парсинга
    let mut images: Vec<Json> = Vec::new();
    let mut tables: Vec<Json> = Vec::new();
    let mut lists: Vec<Json> = Vec::new();
    let mut list_items: Vec<String> = Vec::new();
    let mut current_list_item = String::new();
    let mut in_list = false;
    let mut in_list_item = false;
    let mut in_table = false;
    let mut current_row: Vec<Json> = Vec::new();
    let mut table_rows: Vec<Json> = Vec::new();
    let mut in_cell = false;
    let mut cell_text = String::new();
    
    // Для форматирования текста
    let mut in_span = false;
    let mut span_text = String::new();
    
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = String::from_utf8_lossy(e.name().0);
                match name.as_ref() {
                    "text:p" => {
                        in_paragraph = true;
                        current_text.clear();
                    }
                    "text:span" => {
                        in_span = true;
                        span_text.clear();
                    }
                    "text:list" => {
                        in_list = true;
                        list_items.clear();
                    }
                    "text:list-item" => {
                        in_list_item = true;
                        current_list_item.clear();
                    }
                    "table:table" => {
                        in_table = true;
                        table_rows.clear();
                    }
                    "table:table-row" => {
                        current_row.clear();
                    }
                    "table:table-cell" => {
                        in_cell = true;
                        cell_text.clear();
                    }
                    "draw:image" => {
                        for attr in e.attributes() {
                            if let Ok(attr) = attr {
                                let attr_name = String::from_utf8_lossy(attr.key.0);
                                if attr_name == "xlink:href" {
                                    if let Ok(href) = attr.unescape_value() {
                                        images.push(Json::String(href.to_string()));
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(e)) => {
                if let Ok(text) = e.decode() {
                    let text_str = text.to_string();
                    
                    if in_paragraph && !text_str.trim().is_empty() {
                        current_text.push_str(&text_str);
                        if in_span {
                            span_text.push_str(&text_str);
                        }
                    }
                    
                    if in_table && in_cell {
                        cell_text.push_str(&text_str);
                    }
                    
                    if in_list && in_list_item {
                        current_list_item.push_str(&text_str);
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                let name = String::from_utf8_lossy(e.name().0);
                match name.as_ref() {
                    "text:p" => {
                        let trimmed = current_text.trim();
                        if !trimmed.is_empty() {
                            paragraphs.push(trimmed.to_string());
                        }
                        in_paragraph = false;
                    }
                    "text:span" => {
                        in_span = false;
                    }
                    "text:list-item" => {
                        in_list_item = false;
                        if !current_list_item.trim().is_empty() {
                            list_items.push(current_list_item.trim().to_string());
                        }
                    }
                    "text:list" => {
                        if !list_items.is_empty() {
                            let list_json: Vec<Json> = list_items.iter().map(|s| Json::String(s.clone())).collect();
                            lists.push(Json::Array(list_json));
                        }
                        in_list = false;
                    }
                    "table:table-cell" => {
                        in_cell = false;
                        if !cell_text.trim().is_empty() {
                            current_row.push(Json::String(cell_text.trim().to_string()));
                        }
                    }
                    "table:table-row" => {
                        if !current_row.is_empty() {
                            table_rows.push(Json::Array(current_row.clone()));
                        }
                        current_row.clear();
                    }
                    "table:table" => {
                        if !table_rows.is_empty() {
                            tables.push(Json::Array(table_rows.clone()));
                        }
                        in_table = false;
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                if !e.to_string().contains("syntax") && !e.to_string().contains("invalid") {
                    return Err(format!("XML parse error: {}", e));
                }
            }
            _ => {}
        }
        buf.clear();
    }
    
    // Собираем полный текст из параграфов
    let full_text: String = paragraphs.join("\n");
    
    // Разбиваем на массив символов
    let chars: Vec<String> = full_text.chars().map(|c| c.to_string()).collect();
    let char_count = chars.len();
    let word_count = full_text.split_whitespace().count();
    let line_count = full_text.lines().count();
    let paragraph_count = paragraphs.len();
    
    // Собираем результат в формате, похожем на DOCX
    let mut result = serde_json::Map::new();
    result.insert("text".to_string(), Json::String(full_text));
    result.insert("paragraphs".to_string(), Json::Array(paragraphs.into_iter().map(Json::String).collect()));
    result.insert("char_count".to_string(), Json::Number(serde_json::Number::from(char_count)));
    result.insert("word_count".to_string(), Json::Number(serde_json::Number::from(word_count)));
    result.insert("line_count".to_string(), Json::Number(serde_json::Number::from(line_count)));
    result.insert("paragraph_count".to_string(), Json::Number(serde_json::Number::from(paragraph_count)));
    
    if !images.is_empty() {
        result.insert("images".to_string(), Json::Array(images));
    }
    if !tables.is_empty() {
        result.insert("tables".to_string(), Json::Array(tables));
    }
    if !lists.is_empty() {
        result.insert("lists".to_string(), Json::Array(lists));
    }
    
    Ok(Json::Object(result))
}