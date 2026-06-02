// src-tauri/src/convert.rs

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize)]
pub struct ConvertResult {
    pub success: bool,
    pub content: String,
    pub error: Option<String>,
}

type AnyValue = serde_json::Value;

// ============================================================
// 1. КОНВЕРТАЦИЯ
// ============================================================

pub fn convert(path: &str, from: &str, to: &str) -> Result<String, String> {
    let input = std::fs::read_to_string(path)
        .map_err(|e| format!("Cannot read file: {e}"))?;
    let value = parse(&input, from)?;
    stringify(&value, to)
}

// ============================================================
// 2. СОХРАНЕНИЕ в local data dir
// ============================================================

pub fn save_to_app_dir(content: &str, original_path: &str, to: &str) -> Result<String, String> {
    let input_path = PathBuf::from(original_path);
    let stem = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("converted");

    let ext = match to {
        "json" | "json5" => "json",
        "yaml" | "yml" => "yaml",
        "toml" => "toml",
        "csv" => "csv",
        "xml" => "xml",
        "ini" => "ini",
        "markdown" | "md" => "md",
        "html" => "html",
        _ => "txt",
    };

    let data_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."));
    let output_dir = data_dir.join("formato");
    std::fs::create_dir_all(&output_dir)
        .map_err(|e| format!("Cannot create directory: {e}"))?;

    let output_path = output_dir.join(format!("{}.{}", stem, ext));
    std::fs::write(&output_path, content)
        .map_err(|e| format!("Cannot write file: {e}"))?;

    Ok(output_path.to_string_lossy().to_string())
}

// ============================================================
// Tauri command
// ============================================================

#[tauri::command]
pub fn convert_file(path: String, from: String, to: String) -> Result<ConvertResult, String> {
    let output = convert(&path, &from, &to)?;
    let saved_path = save_to_app_dir(&output, &path, &to)?;

    Ok(ConvertResult {
        success: true,
        content: saved_path,
        error: None,
    })
}

// ============================================================
// Парсеры (без изменений)
// ============================================================

fn parse(input: &str, format: &str) -> Result<AnyValue, String> {
    match format {
        "json" | "json5" | "hjson" => {
            serde_json::from_str(input).map_err(|e| format!("JSON parse error: {e}"))
        }
        "yaml" | "yml" => {
            serde_yaml::from_str(input).map_err(|e| format!("YAML parse error: {e}"))
        }
        "toml" => {
            toml::from_str(input).map_err(|e| format!("TOML parse error: {e}"))
        }
        "csv" => parse_csv(input),
        "xml" => parse_xml(input),
        "ini" => parse_ini(input),
        _ => Err(format!("Unsupported format: {format}")),
    }
}

fn parse_csv(input: &str) -> Result<AnyValue, String> {
    let mut reader = csv::Reader::from_reader(input.as_bytes());
    let mut rows = Vec::new();
    for result in reader.records() {
        let record = result.map_err(|e| format!("CSV parse error: {e}"))?;
        let map: serde_json::Map<String, serde_json::Value> = record
            .iter()
            .enumerate()
            .map(|(i, v)| (i.to_string(), serde_json::Value::String(v.to_string())))
            .collect();
        rows.push(serde_json::Value::Object(map));
    }
    Ok(serde_json::Value::Array(rows))
}

fn parse_xml(input: &str) -> Result<AnyValue, String> {
    let mut reader = quick_xml::Reader::from_str(input);
    let mut buf = Vec::new();
    let mut stack: Vec<serde_json::Value> = Vec::new();
    let mut root: Option<serde_json::Value> = None;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(quick_xml::events::Event::Start(ref e)) => {
                let _name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                stack.push(serde_json::Value::Object(serde_json::Map::new()));
            }
            Ok(quick_xml::events::Event::Text(ref e)) => {
                let text = String::from_utf8_lossy(e.as_ref()).to_string();
                if !text.trim().is_empty() {
                    if let Some(obj) = stack.last_mut() {
                        *obj = serde_json::Value::String(text);
                    }
                }
            }
            Ok(quick_xml::events::Event::End(ref e)) => {
                if let Some(val) = stack.pop() {
                    if let Some(parent) = stack.last_mut() {
                        let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                        if let Some(obj) = parent.as_object_mut() {
                            obj.insert(name, val);
                        }
                    } else {
                        root = Some(val);
                    }
                }
            }
            Ok(quick_xml::events::Event::Eof) => break,
            Err(e) => return Err(format!("XML parse error: {e}")),
            _ => {}
        }
        buf.clear();
    }
    Ok(root.unwrap_or(serde_json::Value::Null))
}

fn parse_ini(input: &str) -> Result<AnyValue, String> {
    let map: serde_json::Map<String, serde_json::Value> =
        serde_ini::from_str(input).map_err(|e| format!("INI parse error: {e}"))?;
    Ok(serde_json::Value::Object(map))
}

// ============================================================
// Сериализаторы (без изменений)
// ============================================================

fn stringify(value: &AnyValue, format: &str) -> Result<String, String> {
    match format {
        "json" | "json5" | "hjson" => {
            serde_json::to_string_pretty(value).map_err(|e| format!("JSON error: {e}"))
        }
        "yaml" | "yml" => {
            serde_yaml::to_string(value).map_err(|e| format!("YAML error: {e}"))
        }
        "toml" => toml::to_string_pretty(value).map_err(|e| format!("TOML error: {e}")),
        "csv" => stringify_csv(value),
        "xml" => stringify_xml(value),
        "ini" => stringify_ini(value),
        _ => Err(format!("Unsupported format: {format}")),
    }
}

fn stringify_csv(value: &AnyValue) -> Result<String, String> {
    match value {
        // Массив объектов — стандартный CSV
        serde_json::Value::Array(arr) => {
            let mut wtr = csv::Writer::from_writer(Vec::new());
            if let Some(first) = arr.first().and_then(|v| v.as_object()) {
                let headers: Vec<&str> = first.keys().map(|k| k.as_str()).collect();
                wtr.write_record(&headers).map_err(|e| format!("CSV error: {e}"))?;
            }
            for row in arr {
                if let Some(obj) = row.as_object() {
                    let vals: Vec<String> = obj
                        .values()
                        .map(|v| match v {
                            serde_json::Value::String(s) => s.clone(),
                            other => other.to_string(),
                        })
                        .collect();
                    wtr.write_record(&vals).map_err(|e| format!("CSV error: {e}"))?;
                }
            }
            let data = wtr.into_inner().map_err(|e| format!("CSV error: {e}"))?;
            String::from_utf8(data).map_err(|e| format!("CSV error: {e}"))
        }
        // Одиночный объект — одна строка CSV
        serde_json::Value::Object(obj) => {
            let mut wtr = csv::Writer::from_writer(Vec::new());
            let headers: Vec<&str> = obj.keys().map(|k| k.as_str()).collect();
            wtr.write_record(&headers).map_err(|e| format!("CSV error: {e}"))?;
            let vals: Vec<String> = obj
                .values()
                .map(|v| match v {
                    serde_json::Value::String(s) => s.clone(),
                    other => other.to_string(),
                })
                .collect();
            wtr.write_record(&vals).map_err(|e| format!("CSV error: {e}"))?;
            let data = wtr.into_inner().map_err(|e| format!("CSV error: {e}"))?;
            String::from_utf8(data).map_err(|e| format!("CSV error: {e}"))
        }
        // Строка/число — одна ячейка
        other => Ok(other.as_str().unwrap_or(&other.to_string()).to_string()),
    }
}



fn stringify_xml(value: &AnyValue) -> Result<String, String> {
    fn value_to_xml(name: &str, value: &serde_json::Value) -> String {
        match value {
            serde_json::Value::Object(map) => {
                let inner: String = map.iter().map(|(k, v)| value_to_xml(k, v)).collect();
                format!("<{name}>{inner}</{name}>")
            }
            serde_json::Value::Array(arr) => arr
                .iter()
                .map(|v| value_to_xml(name, v))
                .collect::<Vec<_>>()
                .join("\n"),
            serde_json::Value::String(s) => format!("<{name}>{s}</{name}>"),
            other => format!("<{name}>{other}</{name}>"),
        }
    }
    Ok(value_to_xml("root", value))
}

fn stringify_ini(value: &AnyValue) -> Result<String, String> {
    let map = value.as_object().ok_or("INI: expected object")?;
    let mut result = String::new();
    for (section_name, section_val) in map {
        result.push_str(&format!("[{}]\n", section_name));
        if let Some(section) = section_val.as_object() {
            for (k, v) in section {
                result.push_str(&format!(
                    "{} = {}\n",
                    k,
                    v.as_str().unwrap_or(&v.to_string())
                ));
            }
        }
        result.push('\n');
    }
    Ok(result)
}


#[tauri::command]
pub async fn read_file_content(path: String) -> Result<String, String> {
    tokio::fs::read_to_string(&path)
        .await
        .map_err(|e| format!("Cannot read file: {e}"))
}

#[tauri::command]
pub async fn open_file(path: String) -> Result<(), String> {
    // opener синхронный, но лёгкий — spawn_blocking не нужен
    opener::open(&path).map_err(|e| format!("Cannot open file: {e}"))
}
