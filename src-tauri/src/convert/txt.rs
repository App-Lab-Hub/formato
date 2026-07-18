use serde_json::{Value as Json};

// ============================================================
// ПАРСЕР TXT → JSON (разбивка на предложения)
// ============================================================

/// Парсит txt файл в JSON с полями:
/// - sentence1: "первое предложение"
/// - sentence2: "второе предложение"
/// - ... и так далее
pub fn parse_txt(input: &str) -> Result<Json, String> {
    let sentences = split_into_sentences(input);
    
    let mut map = serde_json::Map::new();
    for (i, sentence) in sentences.iter().enumerate() {
        let key = format!("sentence{}", i + 1);
        map.insert(key, Json::String(sentence.clone()));
    }
    
    // Добавляем метаданные
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
        
        // Проверяем конец предложения: . ! ?
        if matches!(c, '.' | '!' | '?') {
            // Проверяем, что это не аббревиатура (например, "Dr.", "Mr.", "Mrs.", "etc.")
            if !is_abbreviation(&current) {
                // Проверяем следующий символ: если это пробел или конец строки
                if let Some(&next) = chars.peek() {
                    if next.is_whitespace() || next == '\n' {
                        // Если это пробел, пропускаем его
                        if next == ' ' || next == '\n' {
                            chars.next();
                        }
                        // Добавляем предложение, обрезая пробелы
                        let trimmed = current.trim().to_string();
                        if !trimmed.is_empty() {
                            result.push(trimmed);
                        }
                        current.clear();
                        continue;
                    }
                } else {
                    // Конец строки
                    let trimmed = current.trim().to_string();
                    if !trimmed.is_empty() {
                        result.push(trimmed);
                    }
                    current.clear();
                }
            }
        }
    }
    
    // Если остался текст без знаков препинания
    if !current.trim().is_empty() {
        result.push(current.trim().to_string());
    }
    
    result
}

/// Проверяет, является ли текст аббревиатурой
fn is_abbreviation(text: &str) -> bool {
    let trimmed = text.trim();
    // Проверяем известные аббревиатуры
    let abbreviations = [
        "Dr.", "Mr.", "Mrs.", "Ms.", "Prof.", "Rev.", "Hon.", "Capt.", "Lt.", "Col.",
        "Gen.", "Maj.", "Sgt.", "Cpl.", "Pvt.", "Adm.", "Sen.", "Rep.", "Gov.", "Pres.",
        "etc.", "e.g.", "i.e.", "vs.", "inc.", "corp.", "co.", "ltd.",
        "Jan.", "Feb.", "Mar.", "Apr.", "Jun.", "Jul.", "Aug.", "Sep.", "Oct.", "Nov.", "Dec.",
    ];
    
    // Проверяем точные совпадения
    for abbr in abbreviations {
        if trimmed == abbr || trimmed.ends_with(abbr) {
            return true;
        }
    }
    
    // Проверяем, является ли текстом типа "U.S." или "U.S.A."
    if trimmed.len() >= 2 && trimmed.chars().all(|c| c.is_ascii_alphabetic() || c == '.') {
        let dots_count = trimmed.matches('.').count();
        if dots_count >= 1 && dots_count <= 3 {
            return true;
        }
    }
    
    false
}

// ============================================================
// СЕРИАЛИЗАТОР JSON → TXT
// ============================================================
// ============================================================
// СЕРИАЛИЗАТОР JSON → TXT
// ============================================================

/// Преобразует JSON в txt
/// Если JSON содержит поле "content" — использует его
/// Если JSON содержит поля "sentence1", "sentence2" — объединяет их в текст
/// Иначе выводит JSON в человекочитаемом виде
pub fn stringify_txt(value: &Json) -> Result<String, String> {
    // Если есть поле "content" — берём его
    if let Some(obj) = value.as_object() {
        if let Some(Json::String(text)) = obj.get("content") {
            return Ok(text.clone());
        }
        
        // Если есть поля sentence1, sentence2, ... — собираем в текст
        let mut sentences: Vec<String> = Vec::new();
        let mut i = 1;
        while let Some(Json::String(sentence)) = obj.get(&format!("sentence{}", i)) {
            sentences.push(sentence.clone());
            i += 1;
        }
        if !sentences.is_empty() {
            return Ok(sentences.join(" "));
        }
        
        // Если есть поле "lines" — объединяем строки
        if let Some(Json::Array(lines)) = obj.get("lines") {
            let text: Vec<String> = lines
                .iter()
                .filter_map(|item| item.as_str().map(|s| s.to_string())) // ✅ as_str() вместо as_string()
                .collect();
            if !text.is_empty() {
                return Ok(text.join("\n"));
            }
        }
    }
    
    // Если JSON — массив строк
    if let Some(arr) = value.as_array() {
        let mut result = String::new();
        for item in arr {
            if let Some(s) = item.as_str() {
                result.push_str(s);
                result.push(' ');
            } else {
                result.push_str(&item.to_string());
                result.push(' ');
            }
        }
        return Ok(result.trim().to_string());
    }
    
    // Если JSON — просто строка
    if let Some(s) = value.as_str() {
        return Ok(s.to_string());
    }
    
    // Иначе — красиво форматируем JSON
    serde_json::to_string_pretty(value).map_err(|e| format!("JSON to txt: {e}"))
}