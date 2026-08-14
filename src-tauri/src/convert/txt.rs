// src/convert/txt.rs
use serde_json::Value as Json;
use unicode_segmentation::UnicodeSegmentation;

/// Парсит текст в JSON с разбивкой на предложения
pub fn parse_txt(input: &str) -> Result<Json, String> {
    let sentences: Vec<String> = input
        .unicode_sentences()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let mut map = serde_json::Map::new();

    for (i, sentence) in sentences.iter().enumerate() {
        let key = format!("sentence{}", i + 1);
        map.insert(key, Json::String(sentence.clone()));
    }

    map.insert(
        "total_sentences".to_string(),
        Json::Number(serde_json::Number::from(sentences.len())),
    );
    map.insert(
        "total_words".to_string(),
        Json::Number(serde_json::Number::from(input.split_whitespace().count())),
    );
    map.insert(
        "total_chars".to_string(),
        Json::Number(serde_json::Number::from(input.len())),
    );
    map.insert("content".to_string(), Json::String(input.to_string()));

    Ok(Json::Object(map))
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
