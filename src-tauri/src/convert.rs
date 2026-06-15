// src-tauri/src/convert.rs

use serde::{ Serialize};
use std::{path::PathBuf};
use json2csv::write_json_to_csv;
use xml2json_rs::XmlBuilder;
use std::io::BufReader;
use handlebars::{
    Handlebars, Helper, HelperDef, Output, RenderContext, RenderError, RenderErrorReason,
};
use serde_json::Value as Json;
use crate::html_convert::convert_to_html;

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
        "json" | "json5" | "hjson" => "json",
        "yaml" | "yml" => "yaml",
        "toml" => "toml",
        "csv" => "csv",
        "xml" => "xml",
        "ini" => "ini",
        "html" => "html",
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
    // let json_str = || serde_json::to_string(value).map_err(|e| format!("JSON: {e}"));
    
    match format {
        "json" | "json5" | "hjson" => serde_json::to_string_pretty(value).map_err(|e| format!("JSON: {e}")),
        "yaml" | "yml" => serde_yaml::to_string(value).map_err(|e| format!("YAML: {e}")),
        "toml" => toml::to_string_pretty(value).map_err(|e| format!("TOML: {e}")),
        "xml" => stringify_xml(value).map_err(|e| format!("XML: {e}")),
        "csv" => stringify_csv(value),
        "ini" => stringify_ini(value), // Лаконичный вызов внешней функции
        "html" => Ok(convert_to_html(value)),
        "markdown" | "md" => stringify_markdown(value),
        _ => Err(format!("Unsupported: {format}")),
    }
}



const MD_CSS: &str = r#"<span style="display:none"></span>

<style>
  /* --- 0. БАЗОВЫЕ СБРОСЫ --- */
  body {
    margin: 0 !important;
    padding: 0 !important;
    background: #1e1e1e !important;
  }

  /* --- 1. КАСКАДНАЯ ПОДСВЕТКА (И в VS Code, и на сайтах) --- */
  .markdown-preview blockquote, blockquote { 
    border-left: 2px solid #3f3f46 !important; 
    padding-left: 12px !important; 
    margin: 4px 0 !important; 
    background: transparent !important; 
    transition: border-color 0.15s ease !important; 
  }
  
  .markdown-preview blockquote:hover, blockquote:hover { 
    border-left-color: #9cdcfe !important; 
  }
  
  blockquote blockquote { border-left-color: #3f3f46 !important; }
  blockquote blockquote blockquote { border-left-color: #3f3f46 !important; }
  blockquote blockquote:hover { border-left-color: #9cdcfe !important; }
  blockquote blockquote blockquote:hover { border-left-color: #9cdcfe !important; }

  /* Безопасная поддержка списков (> -) через строгую вложенность */
  blockquote:has(ul:hover), blockquote:has(li:hover) { border-left-color: #9cdcfe !important; }
  blockquote blockquote:has(ul:hover), blockquote blockquote:has(li:hover) { border-left-color: #9cdcfe !important; }


  /* --- 2. МАССИВЫ И ИНДЕКСЫ ([0]) --- */
  blockquote h2, .markdown-preview h2, h2 {
    color: #52525b !important; 
    font-size: 1.2em !important; 
    font-weight: 600 !important;
    margin: 10px 0 4px 0 !important;
  }


  /* --- 3. НАЗВАНИЯ ПАРАМЕТРОВ / КЛЮЧИ (Синие) --- */
  blockquote strong, p strong, .markdown-preview strong, strong {
    color: #4fc1ff !important; 
    font-weight: 600 !important;
  }


  /* --- 4. ЗНАЧЕНИЯ В КОДЕ (`code`) --- */
  .markdown-preview code, code {
    font-family: monospace !important;
    background-color: #1e1e1e !important; 
    color: #ce9178 !important;            
    padding: 2px 5px !important;
    border-radius: 3px !important;
    border: 1px solid #2d2d2d !important;
    font-size: 0.9em !important;
  }
  
  strong code {
    color: #b5cea8 !important;            
    font-weight: bold !important;
  }


  /* --- 5. ТЕКСТ И ЗАГОЛОВКИ СЕКЦИЙ --- */
  .markdown-preview h3, h3 {
    color: #f4f4f5 !important;
    font-size: 1.15em !important;
    margin: 18px 0 6px 0 !important;
    border: none !important;
  }

  /* РЕШЕНИЕ: Точный изолированный селектор для списков внутри цитат */
  blockquote ul, blockquote li {
    color: #a1a1aa !important;            
  }
</style>

"#;

const ENTRY_TEMPLATE: &str = "{{{md _value _key _depth}}}";

#[derive(Clone, Copy)]
struct MdHelper;

impl HelperDef for MdHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        r: &'reg Handlebars<'reg>,
        _: &handlebars::Context,
        _: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> Result<(), RenderError> {
        let value = h.param(0)
            .ok_or_else(|| RenderErrorReason::ParamNotFoundForIndex("md", 0))?
            .value();
        let key = h.param(1).and_then(|p| p.value().as_str()).unwrap_or("");
        let depth: usize = h.param(2).and_then(|p| p.value().as_u64()).unwrap_or(0) as usize;

        let indent = "> ".repeat(depth);
        let is_root = depth == 0 && key.is_empty();

        match value {
            Json::Object(obj) if obj.is_empty() => {
                if !key.is_empty() { write!(out, "{}**{}** `{{}}`\n", indent, key)?; }
            }
            Json::Array(arr) if arr.is_empty() => {
                if !key.is_empty() { write!(out, "{}**{}** `[]`\n", indent, key)?; }
            }
            Json::Object(obj) => {
                if !key.is_empty() {
                    write!(out, "{}### {}\n", indent, key)?;
                }
                let field_indent = if is_root { String::new() } else { format!("{}> ", indent) };
                let next_depth = if is_root { 0 } else { depth + 1 };
                for (k, v) in obj {
                    if k.starts_with('_') { continue; }
                    match v {
                        Json::Object(_) | Json::Array(_) => {
                            out.write(&render_entry(r, v, k, next_depth))?;
                        }
                        _ => write!(out, "{}**{}** {}\n", field_indent, k, format_primitive_md(v))?,
                    }
                }
            }
            Json::Array(arr) if all_primitive(arr) => {
                if !key.is_empty() {
                    write!(out, "{}**{}**\n", indent, key)?;
                    let item_indent = format!("{}> ", indent);
                    for (i, item) in arr.iter().enumerate() {
                        write!(out, "{}- [{}] {}\n", item_indent, i, format_primitive_md(item))?;
                    }
                } else {
                    let items: Vec<String> = arr.iter().map(format_primitive_md).collect();
                    write!(out, "{}", items.join(" "))?;
                }
            }
            Json::Array(arr) => {
                if !key.is_empty() {
                    write!(out, "{}**{}**\n", indent, key)?;
                }
                let item_indent = format!("{}> ", indent);
                for (i, item) in arr.iter().enumerate() {
                    write!(out, "{}## [{}]\n", item_indent, i)?;
                    match item {
                        Json::Object(obj) => {
                            let field_indent = format!("{}> ", item_indent);
                            for (k, v) in obj {
                                if k.starts_with('_') { continue; }
                                match v {
                                    Json::Object(_) | Json::Array(_) => {
                                        out.write(&render_entry(r, v, k, depth + 2))?;
                                    }
                                    _ => write!(out, "{}**{}** {}\n", field_indent, k, format_primitive_md(v))?,
                                }
                            }
                        }
                        Json::Array(_) => out.write(&render_entry(r, item, "", depth + 2))?,
                        _ => write!(out, "{}{}\n", item_indent, format_primitive_md(item))?,
                    }
                    if i < arr.len() - 1 {
                        write!(out, "{}---\n", item_indent)?;
                    }
                }
            }
            _ => {
                let s = format_primitive_md(value);
                if key.is_empty() { write!(out, "{}", s)?; }
                else { write!(out, "{}**{}** {}\n", indent, key, s)?; }
            }
        }
        Ok(())
    }
}

fn render_entry(reg: &Handlebars, value: &Json, key: &str, depth: usize) -> String {
    let mut params = serde_json::Map::new();
    params.insert("_value".to_string(), value.clone());
    params.insert("_key".to_string(), Json::String(key.to_string()));
    params.insert("_depth".to_string(), Json::Number(depth.into()));
    let ctx = Json::Object(params);
    reg.render_template(ENTRY_TEMPLATE, &ctx)
        .unwrap_or_else(|e| format!("*error: {}*", e))
}

fn all_primitive(arr: &[Json]) -> bool {
    arr.iter().all(|v| v.is_string() || v.is_number() || v.is_boolean() || v.is_null())
}

fn format_primitive_md(v: &Json) -> String {
    match v {
        Json::String(s) => format!("`{}`", s.replace('`', "\\`").replace('*', "\\*")),
        Json::Number(n) => format!("`{}`", n),
        Json::Bool(b) => format!("**`{}`**", b),
        Json::Null => "*null*".to_string(),
        _ => unreachable!(),
    }
}

pub fn stringify_markdown(value: &Json) -> Result<String, String> {
    let mut reg = Handlebars::new();
    reg.register_escape_fn(handlebars::no_escape);
    reg.register_helper("md", Box::new(MdHelper));
    let result = match value {
        Json::Object(_) | Json::Array(_) => render_entry(&reg, value, "", 0),
        _ => format_primitive_md(value),
    };
    Ok(format!("{}{}", MD_CSS, result.trim()))
}

fn format_ini_value(s: &str) -> String {
    if s.is_empty() {
        return String::new();
    }

    // Уже в кавычках — не трогаем
    if (s.starts_with('\'') && s.ends_with('\'')) || (s.starts_with('"') && s.ends_with('"')) {
        return s.to_string();
    }

    // JSON-объект или массив — не оборачиваем
    if (s.starts_with('{') && s.ends_with('}')) || (s.starts_with('[') && s.ends_with(']')) {
        return s.to_string();
    }

    // Числа: целые, дробные, отрицательные, научная нотация
    if let Ok(n) = s.parse::<f64>() {
        if n.is_finite() {
            return s.to_string();
        }
    }

    // Булевы значения
    if s == "true" || s == "false" {
        return s.to_string();
    }

    // null
    if s == "null" || s == "Null" || s == "NULL" {
        return s.to_string();
    }

    // Содержит спецсимволы INI — оборачиваем в кавычки
    let needs_quoting = s.contains(';')
        || s.contains('#')
        || s.contains('\n')
        || s.contains('\r')
        || s.contains('\t')
        || s.contains('\\')
        || s.starts_with(' ')
        || s.ends_with(' ')
        || s.contains('\0');

    if needs_quoting {
        // Экранируем внутренние кавычки
        let escaped = s.replace('\'', "\\'");
        format!("'{}'", escaped)
    } else {
        s.to_string()
    }
}

fn stringify_ini(value: &AnyValue) -> Result<String, String> {
    fn process(
        key: &str,
        value: &AnyValue,
        structure: &mut Vec<(String, Vec<(String, String)>)>,
        current_section: &str,
    ) {
        let section = if current_section == "__root__" {
            "__root__".to_string()
        } else {
            current_section.to_string()
        };

        match value {
            AnyValue::Object(nested) if !nested.is_empty() => {
                let new_section = if current_section == "__root__" {
                    key.to_string()
                } else {
                    format!("{}.{}", current_section, key)
                };
                for (k, v) in nested {
                    process(k, v, structure, &new_section);
                }
            }
            AnyValue::Object(_) => {
                // Пустой объект — записываем как "{}"
                if let Some(pos) = structure.iter().position(|(s, _)| s == &section) {
                    structure[pos].1.push((key.to_string(), "{}".to_string()));
                } else {
                    structure.push((section, vec![(key.to_string(), "{}".to_string())]));
                }
            }
            AnyValue::Array(arr) if !arr.is_empty() => {
                for item in arr {
                    let val_str = match item {
                        AnyValue::Object(_) | AnyValue::Array(_) => {
                            // Вложенный объект/массив — сериализуем в JSON
                            format_ini_value(&serde_json::to_string(item).unwrap_or_else(|_| item.to_string()))
                        }
                        _ => format_ini_value(item.as_str().unwrap_or(&item.to_string())),
                    };
                    let array_key = format!("{}[]", key);
                    if let Some(pos) = structure.iter().position(|(s, _)| s == &section) {
                        structure[pos].1.push((array_key, val_str));
                    } else {
                        structure.push((section.clone(), vec![(array_key, val_str)]));
                    }
                }
            }
            AnyValue::Array(_) => {
                // Пустой массив
                if let Some(pos) = structure.iter().position(|(s, _)| s == &section) {
                    structure[pos].1.push((key.to_string(), "[]".to_string()));
                } else {
                    structure.push((section, vec![(key.to_string(), "[]".to_string())]));
                }
            }
            AnyValue::String(s) => {
                let val_str = format_ini_value(s);
                if let Some(pos) = structure.iter().position(|(s_name, _)| s_name == &section) {
                    structure[pos].1.push((key.to_string(), val_str));
                } else {
                    structure.push((section, vec![(key.to_string(), val_str)]));
                }
            }
            AnyValue::Number(n) => {
                let val_str = format_ini_value(&n.to_string());
                if let Some(pos) = structure.iter().position(|(s_name, _)| s_name == &section) {
                    structure[pos].1.push((key.to_string(), val_str));
                } else {
                    structure.push((section, vec![(key.to_string(), val_str)]));
                }
            }
            AnyValue::Bool(b) => {
                let val_str = if *b { "true" } else { "false" };
                if let Some(pos) = structure.iter().position(|(s_name, _)| s_name == &section) {
                    structure[pos].1.push((key.to_string(), val_str.to_string()));
                } else {
                    structure.push((section, vec![(key.to_string(), val_str.to_string())]));
                }
            }
            AnyValue::Null => {
                // null — пропускаем
            }
        }
    }

    let mut ini_structure: Vec<(String, Vec<(String, String)>)> = Vec::new();
    if let AnyValue::Object(map) = value {
        for (k, v) in map {
            process(k, v, &mut ini_structure, "__root__");
        }
    } else if let AnyValue::Array(arr) = value {
        // Корневой массив — оборачиваем в секцию "root"
        process("root", &AnyValue::Array(arr.clone()), &mut ini_structure, "__root__");
    } else {
        // Примитив на корне — в глобальную секцию
        process("value", value, &mut ini_structure, "__root__");
    }

    let mut ini_str = String::new();

    // Глобальные ключи (секция "__root__") — в начале
    if let Some(pos) = ini_structure.iter().position(|(s, _)| s == "__root__") {
        let (_, pairs) = &ini_structure[pos];
        for (k, v) in pairs {
            ini_str.push_str(&format!("{}={}\n", k, v));
        }
        if !pairs.is_empty() {
            ini_str.push('\n');
        }
        ini_structure.remove(pos);
    }

    // Остальные секции
    for (section, pairs) in &ini_structure {
        ini_str.push_str(&format!("[{}]\n", section));
        for (k, v) in pairs {
            ini_str.push_str(&format!("{}={}\n", k, v));
        }
        ini_str.push('\n');
    }

    Ok(ini_str.trim_end().to_string() + "\n")
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