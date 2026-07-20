// src/convert/rtf.rs

use rtf_parser::{Lexer, Parser, RtfDocument};
use serde_json::{Value as Json};

/// Парсит RTF в JSON с сохранением структуры
pub fn parse_rtf(input: &str) -> Result<Json, String> {
    // Парсим RTF
    let doc = RtfDocument::try_from(input)
        .map_err(|e| format!("RTF parse error: {}", e))?;
    
    // Извлекаем текст
    let text = doc.get_text();
    
    // Создаём структурированный JSON
    let mut map = serde_json::Map::new();
    
    // Основной текст
    map.insert("text".to_string(), Json::String(text.clone()));
    
    // Собираем стилизованные блоки
    let mut blocks = Vec::new();
    for block in &doc.body {
        let mut block_map = serde_json::Map::new();
        
        // Текст
        block_map.insert("text".to_string(), Json::String(block.text.clone()));
        
        // Стиль
        let mut style = serde_json::Map::new();
        style.insert("bold".to_string(), Json::Bool(block.painter.bold));
        style.insert("italic".to_string(), Json::Bool(block.painter.italic));
        style.insert("underline".to_string(), Json::Bool(block.painter.underline));
        style.insert("font_size".to_string(), Json::Number(serde_json::Number::from(block.painter.font_size)));
        style.insert("font_ref".to_string(), Json::Number(serde_json::Number::from(block.painter.font_ref)));
        block_map.insert("style".to_string(), Json::Object(style));
        
        // Выравнивание
        let alignment = match block.paragraph.alignment {
            rtf_parser::Alignment::LeftAligned => "left",
            rtf_parser::Alignment::RightAligned => "right",
            rtf_parser::Alignment::Center => "center",
            rtf_parser::Alignment::Justify => "justify",
        };
        block_map.insert("alignment".to_string(), Json::String(alignment.to_string()));
        
        blocks.push(Json::Object(block_map));
    }
    map.insert("blocks".to_string(), Json::Array(blocks));
    
    // Метаданные
    map.insert("line_count".to_string(), Json::Number(serde_json::Number::from(text.lines().count())));
    map.insert("char_count".to_string(), Json::Number(serde_json::Number::from(text.chars().count())));
    map.insert("block_count".to_string(), Json::Number(serde_json::Number::from(doc.body.len())));
    
    Ok(Json::Object(map))
}

/// Преобразует JSON обратно в RTF
pub fn stringify_rtf(value: &Json) -> Result<String, String> {
    let text = extract_text_from_json(value);
    
    // Формируем RTF
    let mut rtf = String::from(r"{\rtf1\ansi\deff0");
    rtf.push_str(r"{\fonttbl{\f0\fnil\fcharset0 Arial;}}");
    rtf.push_str(r"\viewkind4\uc1\pard\lang1049\f0\fs20");
    
    // Разбиваем на параграфы
    let lines: Vec<&str> = text.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        if line.trim().is_empty() {
            rtf.push_str(r"\par");
        } else {
            let escaped = line
                .replace('\\', "\\\\")
                .replace('{', "\\{")
                .replace('}', "\\}");
            rtf.push_str(&escaped);
            if i < lines.len() - 1 {
                rtf.push_str(r"\par");
            }
        }
    }
    
    rtf.push_str(r"\par}");
    
    Ok(rtf)
}

/// Извлекает текст из JSON
fn extract_text_from_json(value: &Json) -> String {
    match value {
        Json::String(s) => s.clone(),
        Json::Object(obj) => {
            // Если есть поле "text" — используем его
            if let Some(Json::String(text)) = obj.get("text") {
                return text.clone();
            }
            // Если есть поле "blocks" — собираем тексты
            if let Some(Json::Array(blocks)) = obj.get("blocks") {
                let mut texts = Vec::new();
                for block in blocks {
                    if let Json::Object(b) = block {
                        if let Some(Json::String(t)) = b.get("text") {
                            texts.push(t.clone());
                        }
                    }
                }
                return texts.join(" ");
            }
            // Иначе — собираем все значения
            let mut texts = Vec::new();
            for (_, val) in obj {
                texts.push(extract_text_from_json(val));
            }
            texts.join(" ")
        }
        Json::Array(arr) => {
            let mut texts = Vec::new();
            for item in arr {
                texts.push(extract_text_from_json(item));
            }
            texts.join("\n")
        }
        Json::Number(n) => n.to_string(),
        Json::Bool(b) => b.to_string(),
        Json::Null => String::new(),
    }
}