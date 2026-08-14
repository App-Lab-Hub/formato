// src/convert/rtf.rs

use rtf_parser::RtfDocument;
use serde_json::Value as Json;

/// Парсит RTF в JSON с сохранением структуры
pub fn parse_rtf(input: &str) -> Result<Json, String> {
    // Парсим RTF
    let doc = RtfDocument::try_from(input).map_err(|e| format!("RTF parse error: {}", e))?;

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
        style.insert(
            "font_size".to_string(),
            Json::Number(serde_json::Number::from(block.painter.font_size)),
        );
        style.insert(
            "font_ref".to_string(),
            Json::Number(serde_json::Number::from(block.painter.font_ref)),
        );
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
    map.insert(
        "line_count".to_string(),
        Json::Number(serde_json::Number::from(text.lines().count())),
    );
    map.insert(
        "char_count".to_string(),
        Json::Number(serde_json::Number::from(text.chars().count())),
    );
    map.insert(
        "block_count".to_string(),
        Json::Number(serde_json::Number::from(doc.body.len())),
    );

    Ok(Json::Object(map))
}
