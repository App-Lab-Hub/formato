// src-tauri/src/convert.rs

use scraper::{ElementRef, Html};
use serde::{Deserialize, Serialize};
use std::{collections::{BTreeMap, HashMap}, path::PathBuf};
use json_to_table::json_to_table;
use json2csv::write_json_to_csv;
use flatten_json_object::{ArrayFormatting, Flattener};
use xml2json_rs::XmlBuilder;
use std::io::BufReader;
use handlebars::{
    Context, Handlebars, Helper, HelperDef, Output, RenderContext, RenderError, RenderErrorReason,
};
use serde_json::Value as Json;

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
    // let json_str = || serde_json::to_string(value).map_err(|e| format!("JSON: {e}"));
    
    match format {
        "json" | "json5" | "hjson" => serde_json::to_string_pretty(value).map_err(|e| format!("JSON: {e}")),
        "yaml" | "yml" => serde_yaml::to_string(value).map_err(|e| format!("YAML: {e}")),
        "toml" => toml::to_string_pretty(value).map_err(|e| format!("TOML: {e}")),
        "xml" => stringify_xml(value).map_err(|e| format!("XML: {e}")),
        "csv" => stringify_csv(value),
        "ini" => stringify_ini(value), // Лаконичный вызов внешней функции
        "html" => stringify_html(value),
        "markdown" | "md" => stringify_markdown(value),
        _ => Err(format!("Unsupported: {format}")),
    }
}



const CSS: &str = r#"
<style>
/* ========== RESET ========== */
body {
    margin: 0;
    padding: 0;
    background: #1e1e1e;
}

/* ========== CONTAINER ========== */
.json-table-wrap {
    display: inline-block;
    width: 100%;
    border-radius: 8px;
    overflow: hidden;
    box-shadow: 0 2px 8px rgba(0,0,0,0.3);
}

/* ========== TABLE ========== */
.json-table {
    border-collapse: collapse;
    width: 100%;
    table-layout: fixed;
    font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', 'Cascadia Code', monospace;
    font-size: 13px;
    line-height: 1.6;
    background: #1e1e1e;
    color: #d4d4d4;
}

/* ========== CELLS: общие ========== */
.json-table th,
.json-table td {
    padding: 8px 14px;
    text-align: left;
    vertical-align: middle;
    border-bottom: 1px solid #2d2d2d;
    transition: background 0.15s ease, color 0.15s ease;
}

/* ========== KEY / INDEX ========== */
.json-table .key-cell {
    color: #9cdcfe;
    font-weight: 600;
    white-space: nowrap;
    width: 25%;
    min-width: 120px;
    max-width: 300px;
    overflow: hidden;
    text-overflow: ellipsis;
    user-select: all;
}

.json-table .index-cell {
    color: #888;
    font-weight: 400;
    white-space: nowrap;
    width: 64px;
    min-width: 64px;
    max-width: 64px;
    text-align: right;
    padding: 8px 16px 8px 18px;
    user-select: none;
    font-variant-numeric: tabular-nums;
    text-align: center;
}

/* ========== VALUE ========== */
.json-table .value-cell {
    color: #d4d4d4;
    width: auto;
    word-break: break-word;
    overflow-wrap: break-word;
    hyphens: auto;
}

/* ========== ROWS ========== */
.json-table tbody tr {
    transition: background 0.12s ease, box-shadow 0.12s ease;
    position: relative;
}

/* Hover: вся строка подсвечивается */
.json-table tbody tr:hover {
    background: #2a2d35;
}

/* Hover: левая граница-индикатор */
.json-table tbody tr:hover td:first-child {
    box-shadow: inset 3px 0 0 #569cd6;
}

/* Hover: ключ становится ярче */
.json-table tbody tr:hover .key-cell {
    color: #b8e0ff;
}

/* Последняя строка без разделителя */
.json-table tbody tr:last-child td {
    border-bottom: none;
}

/* ========== ВЛОЖЕННЫЕ ТАБЛИЦЫ ========== */
.json-table .json-table {
    margin: 0;
    width: 100%;
    border-radius: 4px;
    overflow: hidden;
}
.json-table .value-cell > .json-table-wrap {
    margin: 4px 0 4px -14px;
    width: calc(100% + 14px);
    box-shadow: none;
    border: 1px solid #2d2d2d;
    border-radius: 4px;
}

/* ========== ПРИМИТИВЫ ========== */
.json-string {
    color: #ce9178;
    word-break: break-word;
}
.json-string::before,
.json-string::after {
    color: #ce9178;
    opacity: 0.7;
}

.json-number {
    color: #b5cea8;
    font-variant-numeric: tabular-nums;
}

.json-bool {
    color: #569cd6;
    font-weight: 500;
}

.json-null {
    color: #6a6a6a;
    font-style: italic;
}

/* ========== EMPTY OBJECT/ARRAY ========== */
.json-empty {
    color: #6a6a6a;
    font-style: italic;
    opacity: 0.7;
}
</style>
"#;

const OBJECT_TEMPLATE: &str = r#"
<table class="json-table">
<tbody>
{{#each this}}
    <tr>
        <td class="key-cell">{{@key}}</td>
        <td class="value-cell">{{{render_value this}}}</td>
    </tr>
{{/each}}
</tbody>
</table>"#;

const ARRAY_TEMPLATE: &str = r#"
<table class="json-table">
<tbody>
{{#each this}}
    <tr>
        <td class="index-cell">{{@index}}</td>
        <td class="value-cell">{{{render_value this}}}</td>
    </tr>
{{/each}}
</tbody>
</table>"#;

#[derive(Clone, Copy)]
struct RenderValueHelper;

impl HelperDef for RenderValueHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        r: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> Result<(), RenderError> {
        let param = h
            .param(0)
            .ok_or_else(|| RenderErrorReason::ParamNotFoundForIndex("render_value", 0))?;
        let value = param.value();

        match value {
            Json::Object(obj) if obj.is_empty() => {
                out.write("<span class=\"json-empty\">{}</span>")?;
            }
            Json::Array(arr) if arr.is_empty() => {
                out.write("<span class=\"json-empty\">[]</span>")?;
            }
            Json::Object(_) | Json::Array(_) => {
                let html = json_to_html(r, value);
                out.write(&html)?;
            }
            Json::String(s) => {
                let escaped = s
                    .replace('&', "&amp;")
                    .replace('<', "&lt;")
                    .replace('>', "&gt;");
                write!(out, "<span class=\"json-string\">{}</span>", escaped)?;
            }
            Json::Number(n) => {
                write!(out, "<span class=\"json-number\">{}</span>", n)?;
            }
            Json::Bool(b) => {
                write!(out, "<span class=\"json-bool\">{}</span>", b)?;
            }
            Json::Null => {
                out.write("<span class=\"json-null\">null</span>")?;
            }
        }

        Ok(())
    }
}

fn json_to_html(reg: &Handlebars, value: &Json) -> String {
    let template = match value {
        Json::Object(_) => OBJECT_TEMPLATE,
        Json::Array(_) => ARRAY_TEMPLATE,
        _ => unreachable!(),
    };

    let inner = reg
        .render_template(template, value)
        .unwrap_or_else(|e| format!("Render error: {}", e));

    format!("<div class=\"json-table-wrap\">{}</div>", inner)
}

pub fn stringify_html(value: &Json) -> Result<String, String> {
    let mut reg = Handlebars::new();
    reg.register_helper("render_value", Box::new(RenderValueHelper));

    Ok(format!("{}{}", CSS, json_to_html(&reg, value)))
}


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

        match value {
            // ── Пустой объект ────────────────────────────────────
            Json::Object(obj) if obj.is_empty() => {
                if !key.is_empty() {
                    writeln!(out, "{}**{}** `{{}}`", "> ".repeat(depth), key)?;
                }
            }
            // ── Пустой массив ────────────────────────────────────
            Json::Array(arr) if arr.is_empty() => {
                if !key.is_empty() {
                    writeln!(out, "{}**{}** `[]`", "> ".repeat(depth), key)?;
                }
            }
            // ── Объект ───────────────────────────────────────────
            Json::Object(obj) => {
                if !key.is_empty() {
                    writeln!(out, "{}### {}", "> ".repeat(depth), key)?;
                    writeln!(out)?;
                }
                for (k, v) in obj {
                    if k.starts_with('_') { continue; }
                    match v {
                        Json::Object(_) | Json::Array(_) => {
                            out.write(&render_entry(r, v, k, depth + 1))?;
                        }
                        _ => writeln!(out, "{}**{}** {}", "> ".repeat(depth + 1), k, format_primitive(v))?,
                    }
                }
            }
            // ── Массив примитивов (без ключа) ────────────────────
            Json::Array(arr) if key.is_empty() && arr.iter().all(|v| v.is_string() || v.is_number() || v.is_boolean() || v.is_null()) => {
                let items: Vec<String> = arr.iter().map(format_primitive).collect();
                write!(out, "{}", items.join(" "))?;
            }
            // ── Массив примитивов (с ключом) ─────────────────────
            Json::Array(arr) if arr.iter().all(|v| v.is_string() || v.is_number() || v.is_boolean() || v.is_null()) => {
                writeln!(out, "{}**{}**", "> ".repeat(depth), key)?;
                for (i, item) in arr.iter().enumerate() {
                    writeln!(out, "{}- [{}] {}", "> ".repeat(depth + 1), i, format_primitive(item))?;
                }
            }
            // ── Массив объектов/массивов ─────────────────────────
            Json::Array(arr) => {
                if !key.is_empty() {
                    writeln!(out, "{}**{}**", "> ".repeat(depth), key)?;
                }
                for (i, item) in arr.iter().enumerate() {
                    writeln!(out, "{}## [{}]", "> ".repeat(depth + 1), i)?;
                    match item {
                        Json::Object(obj) => {
                            for (k, v) in obj {
                                if k.starts_with('_') { continue; }
                                match v {
                                    Json::Object(_) | Json::Array(_) => {
                                        out.write(&render_entry(r, v, k, depth + 2))?;
                                    }
                                    _ => writeln!(out, "{}**{}** {}", "> ".repeat(depth + 2), k, format_primitive(v))?,
                                }
                            }
                        }
                        Json::Array(_) => out.write(&render_entry(r, item, "", depth + 2))?,
                        _ => writeln!(out, "{}{}", "> ".repeat(depth + 2), format_primitive(item))?,
                    }
                    if i < arr.len() - 1 {
                        writeln!(out, "{}---", "> ".repeat(depth + 1))?;
                    }
                }
            }
            // ── Примитив ─────────────────────────────────────────
            _ => {
                let s = format_primitive(value);
                if key.is_empty() {
                    write!(out, "{}", s)?;
                } else {
                    writeln!(out, "{}**{}** {}", "> ".repeat(depth), key, s)?;
                }
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

fn format_primitive(v: &Json) -> String {
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
        _ => format_primitive(value),
    };
    Ok(result.trim().to_string())
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