use odtgen::prelude::*;
use serde_json::{Value as Json};
use crate::convert::calculate_conversion_hash;
use crate::convert::get_app_dir_path_with_hash;
use std::fs::File;
use std::io::Write;
use zip::ZipArchive;
use std::io::Read;
use quick_xml::Reader as XmlReader;
use quick_xml::events::Event;
use quick_xml::encoding::Decoder;

pub fn stringify_odt(value: &Json, path: &str, from: &str, to: &str) -> Result<String, String> {

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
    let mut result = serde_json::Map::new();
    let mut paragraphs = Vec::new();
    let mut current_text = String::new();
    let mut in_paragraph = false;
    let mut buf = Vec::new();
    let mut images = Vec::new();
    let mut tables = Vec::new();
    let mut lists = Vec::new();
    let mut current_list = Vec::new();
    let mut in_list = false;
    let mut in_table = false;
    let mut current_row = Vec::new();
    let mut table_rows = Vec::new();
    
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = String::from_utf8_lossy(e.name().0);
                match name.as_ref() {
                    "text:p" => {
                        in_paragraph = true;
                        current_text.clear();
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
                    "table:table" => {
                        in_table = true;
                        table_rows.clear();
                    }
                    "table:table-row" => {
                        current_row.clear();
                    }
                    "table:table-cell" => {}
                    "text:list" => {
                        in_list = true;
                        current_list.clear();
                    }
                    "text:list-item" => {}
                    _ => {}
                }
            }
            Ok(Event::Text(e)) => {
                if in_paragraph {
                    if let Ok(text) = e.decode() {
                        current_text.push_str(&text);
                    }
                }
                if in_table && in_paragraph {
                    if let Ok(text) = e.decode() {
                        current_row.push(Json::String(text.to_string()));
                    }
                }
                if in_list && in_paragraph {
                    if let Ok(text) = e.decode() {
                        current_list.push(Json::String(text.to_string()));
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                let name = String::from_utf8_lossy(e.name().0);
                match name.as_ref() {
                    "text:p" => {
                        let trimmed = current_text.trim();
                        if !trimmed.is_empty() {
                            paragraphs.push(Json::String(trimmed.to_string()));
                        }
                        in_paragraph = false;
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
                    "text:list" => {
                        if !current_list.is_empty() {
                            lists.push(Json::Array(current_list.clone()));
                        }
                        in_list = false;
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
    
    // Сохраняем количество параграфов ДО перемещения
    let paragraph_count = paragraphs.len();
    
    // Собираем результат
    if !paragraphs.is_empty() {
        result.insert("paragraphs".to_string(), Json::Array(paragraphs));
    }
    if !images.is_empty() {
        result.insert("images".to_string(), Json::Array(images));
    }
    if !tables.is_empty() {
        result.insert("tables".to_string(), Json::Array(tables));
    }
    if !lists.is_empty() {
        result.insert("lists".to_string(), Json::Array(lists));
    }
    result.insert("paragraph_count".to_string(), Json::Number(serde_json::Number::from(paragraph_count)));
    
    Ok(Json::Object(result))
}
