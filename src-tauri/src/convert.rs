// src-tauri/src/convert.rs

use serde::{ Serialize};
use std::{path::PathBuf};
use json2csv::write_json_to_csv;
use xml2json_rs::XmlBuilder;
use std::io::BufReader;
use handlebars::{
    Handlebars, Helper, HelperDef, Output, RenderContext, RenderError, RenderErrorReason,
};
use indexmap::IndexMap;
use serde_json::Map;
use serde_json::{Value as Json, json};
use crate::html_convert::convert_to_html;
use scraper::{Html, ElementRef}; 
use std::collections::HashMap;
use serde_flattened::flatten_json_value::flatten::flattened;
use serde_flattened::flatten_json_value::unflatten::unflattened;
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
        // "ini" => serde_ini::from_str(input).map_err(|e| format!("INI: {e}")),
        "ini" => parse_ini(input),
        "markdown" | "md" => parse_markdown(input),
        "csv" => parse_csv(input),
        "html" => parse_html(input),  // ← добавить
        _ => Err(format!("Unsupported: {format}")),
    }
}


fn parse_html(input: &str) -> Result<AnyValue, String> {
    let document = Html::parse_document(input);

    fn node_to_hast(element: &ElementRef) -> AnyValue {
        let tag_name = element.value().name().to_lowercase();
        
        // Собираем properties (атрибуты)
        let mut properties = serde_json::Map::new();
        for attr in element.value().attrs() {
            let key = match attr.0 {
                "class" => "className".to_string(),
                "for" => "htmlFor".to_string(),
                "tabindex" => "tabIndex".to_string(),
                "onclick" => "onClick".to_string(),
                "onchange" => "onChange".to_string(),
                "oninput" => "onInput".to_string(),
                _ => attr.0.to_string(),
            };
            
            if key == "className" {
                // class → массив строк
                let classes: Vec<Json> = attr.1.split_whitespace()
                    .map(|c| Json::String(c.to_string()))
                    .collect();
                properties.insert(key, Json::Array(classes));
            } else if attr.1.is_empty() {
                // Булевы атрибуты (disabled, checked и т.д.)
                properties.insert(key, Json::Bool(true));
            } else {
                properties.insert(key, Json::String(attr.1.to_string()));
            }
        }

        // Собираем детей
        let mut children: Vec<Json> = Vec::new();
        for child in element.children() {
            match child.value() {
                scraper::Node::Text(text) => {
                    let trimmed = text.text.trim();
                    if !trimmed.is_empty() {
                        children.push(json!({"type": "text", "value": trimmed}));
                    }
                }
                scraper::Node::Comment(comment) => {
                    children.push(json!({"type": "comment", "value": comment.trim()}));
                }
                scraper::Node::Element(_) => {
                    if let Some(el) = ElementRef::wrap(child) {
                        children.push(node_to_hast(&el));
                    }
                }
                _ => {}
            }
        }

        let mut map = serde_json::Map::new();
        map.insert("type".to_string(), Json::String("element".to_string()));
        map.insert("tagName".to_string(), Json::String(tag_name));
        
        if !properties.is_empty() {
            map.insert("properties".to_string(), Json::Object(properties));
        }
        if !children.is_empty() {
            map.insert("children".to_string(), Json::Array(children));
        }
        
        Json::Object(map)
    }

    let body_sel = scraper::Selector::parse("body").unwrap();
    
    let body_children: Vec<AnyValue> = if let Some(body) = document.select(&body_sel).next() {
        match node_to_hast(&body) {
            Json::Object(map) => {
                map.get("children")
                    .and_then(|c| c.as_array())
                    .cloned()
                    .unwrap_or_default()
            }
            other => vec![other],
        }
    } else {
        let root = document.root_element();
        match node_to_hast(&root) {
            Json::Object(map) => {
                map.get("children")
                    .and_then(|c| c.as_array())
                    .cloned()
                    .unwrap_or_default()
            }
            other => vec![other],
        }
    };

    Ok(json!({"type": "root", "children": body_children}))
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
    let parser = pulldown_cmark::Parser::new_ext(input, pulldown_cmark::Options::all());
    let events: Vec<pulldown_cmark::Event> = parser.collect();

    let mut stack: Vec<(String, Vec<AnyValue>, serde_json::Map<String, Json>)> = Vec::new();
    let mut root_children: Vec<AnyValue> = Vec::new();
    let mut current_text = String::new();

    let mut in_table = false;
    let mut table_headers: Vec<String> = Vec::new();
    let mut table_rows: Vec<AnyValue> = Vec::new();
    let mut table_cells: Vec<String> = Vec::new();
    let mut in_table_head = false;

    fn flush_text(text: &mut String, target: &mut Vec<AnyValue>) {
        let t = text.trim().to_string();
        if !t.is_empty() {
            target.push(json!({"type": "text", "value": t}));
        }
        text.clear();
    }

    fn make_node(node_type: &str, children: Vec<AnyValue>, extra: serde_json::Map<String, Json>) -> AnyValue {
        let mut map = extra;
        map.insert("type".to_string(), Json::String(node_type.to_string()));
        if !children.is_empty() {
            map.insert("children".to_string(), Json::Array(children));
        }
        Json::Object(map)
    }

    fn make_text(value: &str) -> AnyValue {
        json!({"type": "text", "value": value})
    }

    for event in &events {
        match event {
            pulldown_cmark::Event::Start(tag) => {
                flush_text(&mut current_text, if let Some((_, children, _)) = stack.last_mut() { children } else { &mut root_children });

                match tag {
                    pulldown_cmark::Tag::Heading { level, .. } => {
                        let mut attrs = serde_json::Map::new();
                        attrs.insert("depth".to_string(), Json::Number((*level as u64).into()));
                        stack.push(("heading".to_string(), Vec::new(), attrs));
                    }
                    pulldown_cmark::Tag::Paragraph => {
                        stack.push(("paragraph".to_string(), Vec::new(), serde_json::Map::new()));
                    }
                    pulldown_cmark::Tag::BlockQuote(_) => {
                        stack.push(("blockquote".to_string(), Vec::new(), serde_json::Map::new()));
                    }
                    pulldown_cmark::Tag::CodeBlock(kind) => {
                        let mut attrs = serde_json::Map::new();
                        if let pulldown_cmark::CodeBlockKind::Fenced(lang) = kind {
                            if !lang.is_empty() {
                                attrs.insert("lang".to_string(), Json::String(lang.to_string()));
                            }
                        }
                        stack.push(("code".to_string(), Vec::new(), attrs));
                    }
                    pulldown_cmark::Tag::List(ordered) => {
                        let mut attrs = serde_json::Map::new();
                        attrs.insert("ordered".to_string(), Json::Bool(ordered.is_some()));
                        if let Some(start) = ordered {
                            attrs.insert("start".to_string(), Json::Number((*start).into()));
                        }
                        stack.push(("list".to_string(), Vec::new(), attrs));
                    }
                    pulldown_cmark::Tag::Item => {
                        stack.push(("listItem".to_string(), Vec::new(), serde_json::Map::new()));
                    }
                    pulldown_cmark::Tag::Table(_) => {
                        in_table = true;
                        table_rows.clear();
                        // headers не чистим — они перезапишутся в TableHead
                    }
                    pulldown_cmark::Tag::TableHead => {
                        in_table_head = true;
                        table_cells.clear();
                    }
                    pulldown_cmark::Tag::TableRow => {
                        table_cells.clear();
                    }
                    pulldown_cmark::Tag::TableCell => {}
                    pulldown_cmark::Tag::Emphasis => {
                        stack.push(("emphasis".to_string(), Vec::new(), serde_json::Map::new()));
                    }
                    pulldown_cmark::Tag::Strong => {
                        stack.push(("strong".to_string(), Vec::new(), serde_json::Map::new()));
                    }
                    pulldown_cmark::Tag::Link { link_type: _, dest_url, title, id: _ } => {
                        let mut attrs = serde_json::Map::new();
                        attrs.insert("url".to_string(), Json::String(dest_url.to_string()));
                        if !title.is_empty() {
                            attrs.insert("title".to_string(), Json::String(title.to_string()));
                        }
                        stack.push(("link".to_string(), Vec::new(), attrs));
                    }
                    pulldown_cmark::Tag::Image { link_type: _, dest_url, title, id: _ } => {
                        let mut attrs = serde_json::Map::new();
                        attrs.insert("url".to_string(), Json::String(dest_url.to_string()));
                        if !title.is_empty() {
                            attrs.insert("title".to_string(), Json::String(title.to_string()));
                        }
                        stack.push(("image".to_string(), Vec::new(), attrs));
                    }
                    _ => {}
                }
            }
            pulldown_cmark::Event::End(tag_end) => {
                flush_text(&mut current_text, if let Some((_, children, _)) = stack.last_mut() { children } else { &mut root_children });

                match tag_end {
                    pulldown_cmark::TagEnd::Heading(_)
                    | pulldown_cmark::TagEnd::Paragraph
                    | pulldown_cmark::TagEnd::BlockQuote(_)
                    | pulldown_cmark::TagEnd::CodeBlock
                    | pulldown_cmark::TagEnd::List(_)
                    | pulldown_cmark::TagEnd::Item
                    | pulldown_cmark::TagEnd::Emphasis
                    | pulldown_cmark::TagEnd::Strong
                    | pulldown_cmark::TagEnd::Link
                    | pulldown_cmark::TagEnd::Image => {
                        if let Some((node_type, children, extra)) = stack.pop() {
                            let node = make_node(&node_type, children, extra);
                            if let Some((_, parent_children, _)) = stack.last_mut() {
                                parent_children.push(node);
                            } else {
                                root_children.push(node);
                            }
                        }
                    }
                    pulldown_cmark::TagEnd::Table => {
                        in_table = false;
                        let mut table_children: Vec<AnyValue> = Vec::new();

                        // Первая строка — заголовок с пометкой header: true
                        if !table_headers.is_empty() {
                            let mut header_attrs = serde_json::Map::new();
                            header_attrs.insert("header".to_string(), Json::Bool(true));
                            let header_cells: Vec<AnyValue> = table_headers
                                .iter()
                                .map(|h| make_node("tableCell", vec![make_text(h)], serde_json::Map::new()))
                                .collect();
                            table_children.push(make_node("tableRow", header_cells, header_attrs));
                        }

                        // Строки данных
                        for row in &table_rows {
                            if let Json::Object(cells) = row {
                                let cell_nodes: Vec<AnyValue> = cells
                                    .values()
                                    .map(|v| make_node("tableCell", vec![make_text(v.as_str().unwrap_or(""))], serde_json::Map::new()))
                                    .collect();
                                table_children.push(make_node("tableRow", cell_nodes, serde_json::Map::new()));
                            }
                        }

                        root_children.push(make_node("table", table_children, serde_json::Map::new()));
                    }
                    pulldown_cmark::TagEnd::TableHead => {
                        // Сохраняем заголовки и очищаем
                        table_headers = std::mem::take(&mut table_cells);
                        in_table_head = false;
                    }
                    pulldown_cmark::TagEnd::TableRow => {
                        let mut row = serde_json::Map::new();
                        for (i, cell) in table_cells.iter().enumerate() {
                            let key = if table_headers.is_empty() {
                                format!("col{}", i)
                            } else {
                                table_headers.get(i).cloned().unwrap_or_else(|| format!("col{}", i))
                            };
                            row.insert(key, Json::String(cell.clone()));
                        }
                        table_rows.push(Json::Object(row));
                    }
                    _ => {}
                }
            }
            pulldown_cmark::Event::Text(text) => {
                if in_table {
                    table_cells.push(text.to_string());
                } else {
                    current_text.push_str(text);
                }
            }
            pulldown_cmark::Event::Code(code) => {
                if in_table {
                    table_cells.push(format!("`{}`", code));
                } else {
                    let node = json!({"type": "inlineCode", "value": code.to_string()});
                    if let Some((_, children, _)) = stack.last_mut() {
                        children.push(node);
                    } else {
                        root_children.push(node);
                    }
                }
            }
            pulldown_cmark::Event::SoftBreak | pulldown_cmark::Event::HardBreak => {
                current_text.push(' ');
            }
            _ => {}
        }
    }

    flush_text(&mut current_text, &mut root_children);

    if root_children.is_empty() {
        Ok(json!({"type": "root", "children": []}))
    } else {
        Ok(json!({"type": "root", "children": root_children}))
    }
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
        // "toml" => toml::to_string_pretty(value).map_err(|e| format!("TOML: {e}")),
        "toml" => {
            // TOML не поддерживает корневые массивы — оборачиваем в объект
            let value_for_toml = match value {
                AnyValue::Array(arr) => {
                    let mut map = serde_json::Map::new();
                    map.insert("data".to_string(), AnyValue::Array(arr.clone()));
                    AnyValue::Object(map)
                }
                _ => value.clone(),
            };
            toml::to_string_pretty(&value_for_toml).map_err(|e| format!("TOML: {e}"))
        }
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


fn unquote_value(val: &Json) -> Json {
    if let Json::String(s) = val {
        let trimmed = s.trim();
        let unquoted = if (trimmed.starts_with('"') && trimmed.ends_with('"')) ||
           (trimmed.starts_with('\'') && trimmed.ends_with('\'')) {
            trimmed[1..trimmed.len()-1].to_string()
        } else {
            trimmed.to_string()
        };
        
        if let Ok(n) = unquoted.parse::<i64>() {
            return Json::Number(n.into());
        }
        if let Ok(n) = unquoted.parse::<f64>() {
            if n.is_finite() { return json!(n); }
        }
        if unquoted == "true" { return Json::Bool(true); }
        if unquoted == "false" { return Json::Bool(false); }
        if unquoted == "null" { return Json::Null; }
        
        return Json::String(unquoted);
    }
    val.clone()
}


fn convert_objects_to_arrays(value: &mut Json) {
    if let Json::Object(map) = value {
        // Проверяем, все ли ключи числовые
        let all_numeric = !map.is_empty() && map.keys().all(|k| k.parse::<usize>().is_ok());
        
        if all_numeric {
            let max_idx = map.keys().filter_map(|k| k.parse::<usize>().ok()).max().unwrap_or(0);
            let mut arr = Vec::new();
            for i in 0..=max_idx {
                if let Some(val) = map.remove(&i.to_string()) {
                    arr.push(val);
                } else {
                    arr.push(Json::Null);
                }
            }
            *value = Json::Array(arr);
            
            // После превращения в массив — обрабатываем элементы
            if let Json::Array(arr) = value {
                for v in arr.iter_mut() {
                    convert_objects_to_arrays(v);
                }
            }
            return;
        }
        
        // Рекурсивно обрабатываем детей (ключи собираем до итерации)
        let keys: Vec<String> = map.keys().cloned().collect();
        for k in keys {
            if let Some(v) = map.get_mut(&k) {
                convert_objects_to_arrays(v);
            }
        }
        return;
    }
    
    if let Json::Array(arr) = value {
        for v in arr.iter_mut() {
            convert_objects_to_arrays(v);
        }
    }
}


fn stringify_ini(value: &AnyValue) -> Result<String, String> {
    let flat = flattened(value.clone());
    
    let dot_flat: IndexMap<String, Json> = flat.into_iter()
        .map(|(k, v)| {
            let clean = k.replace("__", ".")
                         .replace(".idx-", ".")
                         .replace("idx-", "");
            (clean, v)
        })
        .collect();
    
    let mut result = String::new();
    let mut sections: IndexMap<String, Vec<(String, String)>> = IndexMap::new();
    let mut simple_arrays: IndexMap<String, (String, Vec<String>)> = IndexMap::new();
    
    for (key, val) in &dot_flat {
        let val_str = match val {
            Json::String(s) => format!("\"{}\"", s.replace('"', "\\\"")),
            Json::Number(n) => n.to_string(),
            Json::Bool(b) => if *b { "true".to_string() } else { "false".to_string() },
            Json::Null => continue,
            _ => val.to_string(),
        };
        
        if let Some(dot_pos) = key.find('.') {
            let parts: Vec<&str> = key.split('.').collect();
                            
            if parts.len() >= 2 && parts[parts.len()-1].parse::<usize>().is_ok() {
                let idx: usize = parts[parts.len()-1].parse().unwrap();
                let array_name = parts[parts.len()-2].to_string();
                let section = if parts.len() >= 3 {
                    parts[..parts.len()-2].join(".")
                } else {
                    continue;
                };
                
                let all_numeric = dot_flat.keys()
                    .filter(|k| k.starts_with(&format!("{}.{}.", section, array_name)))
                    .all(|k| {
                        let rest = &k[section.len() + array_name.len() + 2..];
                        !rest.contains('.') && rest.parse::<usize>().is_ok()
                    });
                
                if all_numeric {
                    let entry = simple_arrays.entry(section.clone()).or_insert_with(|| (array_name.clone(), Vec::new()));
                    while entry.1.len() <= idx { entry.1.push(String::new()); }
                    entry.1[idx] = val_str;
                    continue;
                }
            }
            
            let sub_key = parts[parts.len()-1].to_string();
            let section = parts[..parts.len()-1].join(".");
            
            sections.entry(section).or_default().push((sub_key, val_str));
        } else {
            result.push_str(&format!("{} = {}\n", key, val_str));
        }
    }
    
    if !result.is_empty() { result.push('\n'); }
    
    // Собираем уникальные секции из обоих map
    let mut all_sections: Vec<String> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    for key in sections.keys().chain(simple_arrays.keys()) {
        if seen.insert(key.clone()) {
            all_sections.push(key.clone());
        }
    }
    
    let root_order: IndexMap<&str, usize> = if let Json::Object(root) = value {
        root.keys().enumerate().map(|(i, k)| (k.as_str(), i)).collect()
    } else {
        IndexMap::new()
    };
    
    all_sections.sort_by(|a, b| {
        let a_root = a.split('.').next().unwrap_or("");
        let b_root = b.split('.').next().unwrap_or("");
        let a_order = root_order.get(a_root).unwrap_or(&usize::MAX);
        let b_order = root_order.get(b_root).unwrap_or(&usize::MAX);
        
        match a_order.cmp(b_order) {
            std::cmp::Ordering::Equal => {
                let a_parts: Vec<&str> = a.split('.').collect();
                let b_parts: Vec<&str> = b.split('.').collect();
                let min_len = a_parts.len().min(b_parts.len());
                for i in 0..min_len {
                    let cmp = a_parts[i].cmp(b_parts[i]);
                    if cmp != std::cmp::Ordering::Equal { return cmp; }
                }
                a_parts.len().cmp(&b_parts.len())
            }
            other => other,
        }
    });
    
    for section in &all_sections {
        let has_pairs = sections.get(section).map(|p| !p.is_empty()).unwrap_or(false);
        let has_array = simple_arrays.contains_key(section);
        
        if has_pairs || has_array {
            result.push_str(&format!("[{}]\n", section));
            
            // Сначала обычные ключи
            if let Some(pairs) = sections.get(section) {
                for (key, val) in pairs {
                    result.push_str(&format!("{} = {}\n", key, val));
                }
            }
            
            // Потом простые массивы
            if let Some((array_name, values)) = simple_arrays.get(section) {
                for val in values {
                    result.push_str(&format!("{}[] = {}\n", array_name, val));
                }
            }
            
            result.push('\n');
        }
    }
    
    Ok(result.trim_end().to_string() + "\n")
}

fn parse_ini(input: &str) -> Result<AnyValue, String> {
    let mut raw_map: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    let mut current_section = String::new();
    
    // Первый проход: собираем ВСЕ значения для каждого ключа (включая дубликаты)
    for line in input.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            current_section = line[1..line.len()-1].trim().to_string();
            continue;
        }
        if let Some(eq_pos) = line.find('=') {
            let key = line[..eq_pos].trim().to_string();
            let val = line[eq_pos + 1..].trim().to_string();
            let full_key = if current_section.is_empty() {
                key.clone()
            } else {
                format!("{}.{}", current_section, key)
            };
            raw_map.entry(full_key).or_default().push(val);
        }
    }
    
    let raw: Json = serde_ini::from_str(input)
        .map_err(|e| format!("INI: {e}"))?;
    
    let mut flat: Map<String, Json> = Map::new();
    flatten_value(&raw, String::new(), &mut flat);
    
    let mut fixed: Map<String, Json> = Map::new();
    
    for (flat_key, flat_val) in &flat {
        // Очищаем ключ от [] и [N]
        let clean_key = flat_key
            .replace('[', ".")
            .replace(']', "");
        let clean_key = clean_key.strip_suffix('.').unwrap_or(&clean_key).to_string();
        
        // Определяем, является ли это [] массивом (не [N] с числом!)
        // Проверяем оригинальный ключ: если заканчивается на [], это простой массив
        let is_bracket_array = flat_key.ends_with("[]");
        
        if is_bracket_array {
            // Ищем в raw_map все значения для этого ключа
            if let Some(entries) = raw_map.get(flat_key.as_str()) {
                if entries.len() >= 1 {
                    let arr: Vec<Json> = entries.iter()
                        .map(|s| unquote_value(&Json::String(s.clone())))
                        .collect();
                    fixed.insert(clean_key, Json::Array(arr));
                    continue;
                }
            }
        }
        
        // Для обычных ключей (включая 0.label) — берём одно значение
        // Если в raw_map несколько значений (дубликаты ключей), берём ПЕРВОЕ
        if let Some(entries) = raw_map.get(flat_key.as_str()) {
            if entries.len() >= 1 && !is_bracket_array {
                // Берём первое значение (не делаем массив для числовых ключей)
                fixed.insert(clean_key, unquote_value(&Json::String(entries[0].clone())));
                continue;
            }
        }
        
        // Fallback: используем значение из flat
        fixed.insert(clean_key, unquote_value(flat_val));
    }
    
    let has_nesting = fixed.keys().any(|k| k.contains('.'));
    
    if !has_nesting {
        let mut result = serde_json::Map::new();
        for (key, val) in &fixed {
            result.insert(key.clone(), val.clone());
        }
        return Ok(Json::Object(result));
    }
    
    // Используем json_unflattening для восстановления структуры
    let dot_flat: Map<String, Json> = fixed.clone();
    
    let mut result = json_unflattening::unflattening::unflatten(&dot_flat)
        .map_err(|e| format!("INI unflatten: {}", e))?;
    
    // Превращаем {"0": {...}, "1": {...}} в [{...}, {...}]
    convert_objects_to_arrays(&mut result);
    
    Ok(result)
}



fn flatten_value(value: &Json, prefix: String, result: &mut Map<String, Json>) {
    match value {
        Json::Object(map) => {
            for (key, val) in map {
                let new_prefix = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", prefix, key)
                };
                flatten_value(val, new_prefix, result);
            }
        }
        Json::Array(arr) => {
            for (i, val) in arr.iter().enumerate() {
                let new_prefix = format!("{}[{}]", prefix, i);
                flatten_value(val, new_prefix, result);
            }
        }
        _ => {
            result.insert(prefix, value.clone());
        }
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