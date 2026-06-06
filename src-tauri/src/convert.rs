// src-tauri/src/convert.rs

use scraper::{ElementRef, Html};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};
use json_to_table::json_to_table;
use json2csv::write_json_to_csv;
use flatten_json_object::Flattener;
use xml2json_rs::XmlBuilder;
use std::io::BufReader;
    
#[derive(Debug, Serialize)]
pub struct ConvertResult {
    pub success: bool,
    pub content: String,
    pub error: Option<String>,
}

type AnyValue = serde_json::Value;

pub fn convert(path: &str, from: &str, to: &str) -> Result<String, String> {
    let input = std::fs::read_to_string(path).map_err(|e| format!("Cannot read file: {e}"))?;
    let value = parse(&input, from)?;
    stringify(&value, to)
}

pub fn save_to_app_dir(content: &str, original_path: &str, to: &str) -> Result<String, String> {
    let input_path = PathBuf::from(original_path);
    let stem = input_path.file_stem().and_then(|s| s.to_str()).unwrap_or("converted");
    let ext = match to {
        "json" | "json5" | "hjson" => "json", "yaml" | "yml" => "yaml",
        "toml" => "toml", "csv" => "csv", "xml" => "xml", "ini" => "ini",
        "markdown" | "md" => "md",
        _ => "txt",
    };
    let data_dir = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    let output_dir = data_dir.join("formato");
    std::fs::create_dir_all(&output_dir).map_err(|e| format!("Cannot create directory: {e}"))?;
    let output_path = output_dir.join(format!("{}.{}", stem, ext));
    std::fs::write(&output_path, content).map_err(|e| format!("Cannot write file: {e}"))?;
    Ok(output_path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn convert_file(path: String, from: String, to: String) -> Result<ConvertResult, String> {
    let output = convert(&path, &from, &to)?;
    let saved_path = save_to_app_dir(&output, &path, &to)?;
    Ok(ConvertResult { success: true, content: saved_path, error: None })
}

// ============================================================
// ПАРСЕРЫ
// ============================================================

fn parse(input: &str, format: &str) -> Result<AnyValue, String> {
    match format {
        "json" | "json5" | "hjson" => serde_json::from_str(input).map_err(|e| format!("JSON: {e}")),
        "yaml" | "yml" => serde_yaml::from_str(input).map_err(|e| format!("YAML: {e}")),
        "toml" => toml::from_str(input).map_err(|e| format!("TOML: {e}")),
        "xml" => parse_xml(input),
        "ini" => serde_ini::from_str(input).map_err(|e| format!("INI: {e}")),
        "markdown" | "md" => parse_markdown(input),
        "csv" => parse_csv(input),
        _ => Err(format!("Unsupported: {format}")),
    }
}

fn parse_csv(input: &str) -> Result<AnyValue, String> {
    let mut reader = csv::Reader::from_reader(input.as_bytes());
    let headers = reader.headers().map_err(|e| format!("CSV: {e}"))?.clone();
    let mut rows = Vec::new();
    for result in reader.records() {
        let record = result.map_err(|e| format!("CSV: {e}"))?;
        let mut map = serde_json::Map::new();
        for (i, field) in record.iter().enumerate() {
            map.insert(headers.get(i).unwrap_or("unknown").to_string(), serde_json::Value::String(field.to_string()));
        }
        rows.push(serde_json::Value::Object(map));
    }
    Ok(serde_json::Value::Array(rows))
}

fn parse_markdown(input: &str) -> Result<AnyValue, String> {
    let parser = pulldown_cmark::Parser::new(input);
    let mut sections: Vec<AnyValue> = Vec::new();
    let mut current_tag: Option<String> = None;
    let mut current_text = String::new();
    for event in parser {
        match event {
            pulldown_cmark::Event::Start(tag) => {
                if !current_text.trim().is_empty() {
                    let mut map = serde_json::Map::new();
                    let key = current_tag.as_deref().map(md_tag_to_name).unwrap_or("p".into());
                    map.insert(key, serde_json::Value::String(current_text.trim().to_string()));
                    sections.push(serde_json::Value::Object(map));
                    current_text.clear();
                }
                current_tag = Some(format!("{:?}", tag));
            }
            pulldown_cmark::Event::Text(text) => current_text.push_str(&text),
            pulldown_cmark::Event::Code(code) => {
                let mut map = serde_json::Map::new();
                map.insert("_code".into(), serde_json::Value::String(code.to_string()));
                sections.push(serde_json::Value::Object(map));
                current_tag = None;
            }
            pulldown_cmark::Event::End(_) => {
                if !current_text.trim().is_empty() {
                    let mut map = serde_json::Map::new();
                    let key = current_tag.as_deref().map(md_tag_to_name).unwrap_or("p".into());
                    map.insert(key, serde_json::Value::String(current_text.trim().to_string()));
                    sections.push(serde_json::Value::Object(map));
                    current_text.clear();
                }
                current_tag = None;
            }
            _ => {}
        }
    }
    if !current_text.trim().is_empty() {
        let mut map = serde_json::Map::new();
        map.insert("p".into(), serde_json::Value::String(current_text.trim().to_string()));
        sections.push(serde_json::Value::Object(map));
    }
    if sections.is_empty() { Ok(serde_json::Value::Object(serde_json::Map::new())) }
    else if sections.len() == 1 { Ok(sections.into_iter().next().unwrap()) }
    else { Ok(serde_json::Value::Array(sections)) }
}

fn md_tag_to_name(tag: &str) -> String {
    if tag.contains("Heading(1") { "h1" }
    else if tag.contains("Heading(2") { "h2" }
    else if tag.contains("Heading(3") { "h3" }
    else if tag.contains("Heading(4") { "h4" }
    else if tag.contains("Paragraph") { "p" }
    else if tag.contains("BlockQuote") { "blockquote" }
    else if tag.contains("CodeBlock") { "codeblock" }
    else { "p" }.to_string()
}

fn parse_xml(input: &str) -> Result<AnyValue, String> {
    let mut reader = quick_xml::Reader::from_str(input);
    let mut buf = Vec::new();
    let mut stack: Vec<serde_json::Value> = Vec::new();
    let mut root: Option<serde_json::Value> = None;
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(quick_xml::events::Event::Start(_)) => { stack.push(serde_json::Value::Object(serde_json::Map::new())); }
            Ok(quick_xml::events::Event::Text(ref e)) => {
                let text = String::from_utf8_lossy(e.as_ref()).to_string();
                if !text.trim().is_empty() { if let Some(obj) = stack.last_mut() { *obj = serde_json::Value::String(text); } }
            }
            Ok(quick_xml::events::Event::End(ref e)) => {
                if let Some(val) = stack.pop() {
                    if let Some(parent) = stack.last_mut() {
                        let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                        if let Some(obj) = parent.as_object_mut() { obj.insert(name, val); }
                    } else { root = Some(val); }
                }
            }
            Ok(quick_xml::events::Event::Eof) => break,
            Err(e) => return Err(format!("XML: {e}")),
            _ => {}
        }
        buf.clear();
    }
    Ok(root.unwrap_or(serde_json::Value::Null))
}

fn stringify_csv(value: &AnyValue) -> Result<String, String> {
    let json_str = serde_json::to_string(value).map_err(|e| format!("JSON: {e}"))?;
    let mut output = Vec::new();
    
    // BOM для Excel UTF-8
    output.extend_from_slice(&[0xEF, 0xBB, 0xBF]);
    
    write_json_to_csv(
        BufReader::new(json_str.as_bytes()),
        &mut output,
        None,
        Some(",".into()),  // точка с запятой
        true,
        None,
        None,
        true,
    ).map_err(|e| format!("CSV: {e}"))?;
    
    String::from_utf8(output).map_err(|e| format!("CSV: {e}"))
}
// ============================================================
// СЕРИАЛИЗАТОРЫ
// ============================================================

fn stringify(value: &AnyValue, format: &str) -> Result<String, String> {
    let json_str = || serde_json::to_string(value).map_err(|e| format!("JSON: {e}"));
    
    match format {
        "json" | "json5" | "hjson" => serde_json::to_string_pretty(value).map_err(|e| format!("JSON: {e}")),
        "yaml" | "yml" => serde_yaml::to_string(value).map_err(|e| format!("YAML: {e}")),
        "toml" => toml::to_string_pretty(value).map_err(|e| format!("TOML: {e}")),
        "xml" => stringify_xml(value).map_err(|e| format!("XML: {e}")),
        "ini" => {
            let flat_json = Flattener::new()
                .set_key_separator(".")
                .flatten(value)
                .map_err(|e| format!("INI flatten: {e}"))?;
            
            let mut ini_map: HashMap<String, HashMap<String, String>> = HashMap::new();
            if let AnyValue::Object(map) = flat_json {
                for (full_key, val) in map {
                    let val_str = val.as_str().map(|s| s.to_string()).unwrap_or_else(|| val.to_string());
                    if let Some((section, key)) = full_key.split_once('.') {
                        ini_map.entry(section.to_string()).or_default().insert(key.to_string(), val_str);
                    } else {
                        ini_map.entry("General".to_string()).or_default().insert(full_key, val_str);
                    }
                }
            }
            serde_ini::to_string(&ini_map).map_err(|e| format!("INI: {e}"))
        },
        "markdown" | "md" => Ok(json_to_table(value).to_string()),
        "csv" => stringify_csv(value),
        _ => Err(format!("Unsupported: {format}")),
    }
}

fn stringify_xml(value: &AnyValue) -> Result<String, String> {
    let json_str = serde_json::to_string(value).map_err(|e| format!("JSON: {e}"))?;
    XmlBuilder::default()
        .build_from_json_string(&json_str)
        .map_err(|e| format!("XML: {e}"))
}



#[tauri::command]
pub async fn read_file_content(path: String) -> Result<String, String> {
    tokio::fs::read_to_string(&path).await.map_err(|e| format!("Cannot read file: {e}"))
}

#[tauri::command]
pub async fn open_file(path: String) -> Result<(), String> {
    opener::open(&path).map_err(|e| format!("Cannot open file: {e}"))
}