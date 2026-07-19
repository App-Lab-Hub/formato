// src/convert/txt.rs

use shiva::core::{Document, TransformerTrait};
use shiva::text::Transformer as TextTransformer;
use bytes::Bytes;
use serde_json::{Value as Json};

pub fn parse_txt(input: &str) -> Result<Json, String> {
    // 1. Текст → Document через TextTransformer
    let input_bytes = Bytes::from(input.as_bytes().to_vec());
    let document = TextTransformer::parse(&input_bytes)
        .map_err(|e| format!("TextTransformer parse error: {}", e))?;
    
    // 2. Сериализуем Document в JSON через serde_json
    let json_value = serde_json::to_value(&document)
        .map_err(|e| format!("Serialize to JSON error: {}", e))?;
    
    Ok(json_value)
}

pub fn stringify_txt(value: &Json) -> Result<String, String> {
    // 1. Любой JSON → строка → parse_txt → валидный Document
    let json_str = value.to_string();
    let parsed = parse_txt(&json_str)?;  // ← теперь это валидный Document в JSON
    
    // 2. Валидный JSON → Document
    let document: Document = serde_json::from_value(parsed)
        .map_err(|e| format!("Deserialize to Document error: {}", e))?;
    
    // 3. Document → текст
    let text_bytes = TextTransformer::generate(&document)
        .map_err(|e| format!("TextTransformer generate error: {}", e))?;
    
    String::from_utf8(text_bytes.to_vec())
        .map_err(|e| format!("UTF-8 conversion error: {}", e))
}
