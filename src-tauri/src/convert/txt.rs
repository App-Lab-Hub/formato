// src/convert/txt.rs

use serde_json::{Value as Json};

/// Парсит текст в JSON с разбивкой на предложения
pub fn parse_txt(input: &str) -> Result<Json, String> {
    let sentences = split_into_sentences(input);
    
    let mut map = serde_json::Map::new();
    
    for (i, sentence) in sentences.iter().enumerate() {
        let key = format!("sentence{}", i + 1);
        map.insert(key, Json::String(sentence.clone()));
    }
    
    map.insert("total_sentences".to_string(), Json::Number(serde_json::Number::from(sentences.len())));
    
    Ok(Json::Object(map))
}

/// Разбивает текст на предложения
fn split_into_sentences(text: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut chars = text.chars().peekable();
    
    while let Some(c) = chars.next() {
        current.push(c);
        
        if matches!(c, '.' | '!' | '?') {
            if !is_abbreviation(&current) {
                if let Some(&next) = chars.peek() {
                    if next.is_whitespace() || next == '\n' {
                        if next == ' ' || next == '\n' {
                            chars.next();
                        }
                        let trimmed = current.trim().to_string();
                        if !trimmed.is_empty() {
                            result.push(trimmed);
                        }
                        current.clear();
                        continue;
                    }
                } else {
                    let trimmed = current.trim().to_string();
                    if !trimmed.is_empty() {
                        result.push(trimmed);
                    }
                    current.clear();
                }
            }
        }
    }
    
    if !current.trim().is_empty() {
        result.push(current.trim().to_string());
    }
    
    result
}

/// Проверяет, является ли текст аббревиатурой
fn is_abbreviation(text: &str) -> bool {
    let trimmed = text.trim();
    let abbreviations = [
        "Dr.", "Mr.", "Mrs.", "Ms.", "Prof.", "Rev.", "Hon.", 
        "Capt.", "Lt.", "Col.", "Gen.", "Maj.", "Sgt.", "Cpl.", "Pvt.",
        "etc.", "e.g.", "i.e.", "vs.", "inc.", "corp.", "co.", "ltd.",
        "Jan.", "Feb.", "Mar.", "Apr.", "Jun.", "Jul.", "Aug.", 
        "Sep.", "Oct.", "Nov.", "Dec.",
    ];
    
    for abbr in abbreviations {
        if trimmed == abbr || trimmed.ends_with(abbr) {
            return true;
        }
    }
    
    if trimmed.len() >= 2 && trimmed.chars().all(|c| c.is_ascii_alphabetic() || c == '.') {
        let dots = trimmed.matches('.').count();
        if dots >= 1 && dots <= 3 {
            return true;
        }
    }
    
    false
}

/// Преобразует ЛЮБОЙ JSON обратно в текст (рекурсивно обходит все поля)
pub fn stringify_txt(value: &Json) -> Result<String, String> {
    // Рекурсивно извлекаем все текстовые значения
    let texts = extract_texts(value);
    
    if texts.is_empty() {
        // Если ничего не нашли — возвращаем строковое представление JSON
        return Ok(value.to_string());
    }
    
    // Объединяем все найденные тексты
    Ok(texts.join(" "))
}

/// Рекурсивно извлекает все текстовые значения из JSON
fn extract_texts(value: &Json) -> Vec<String> {
    let mut result = Vec::new();
    
    match value {
        Json::String(s) => {
            result.push(s.clone());
        }
        Json::Object(obj) => {
            for (_, val) in obj {
                result.extend(extract_texts(val));
            }
        }
        Json::Array(arr) => {
            for item in arr {
                result.extend(extract_texts(item));
            }
        }
        Json::Number(n) => {
            result.push(n.to_string());
        }
        Json::Bool(b) => {
            result.push(b.to_string());
        }
        Json::Null => {}
    }
    
    result
}