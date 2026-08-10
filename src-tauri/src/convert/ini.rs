

use serde_flattened::flatten_json_value::flatten::flattened;
use indexmap::IndexMap;
use serde_json::Map;
use serde_json::{Value as Json, json};
use std::collections::HashMap;

pub fn stringify_ini(value: &Json) -> Result<String, String> {
   
    
    // 1. Разворачиваем JSON в плоскую структуру через serde_flattened
    let flat = flattened(value.clone());
    
    // 2. Преобразуем в HashMap<String, String> для serde_ini
    let mut flat_map = HashMap::new();
    for (key, val) in flat {
        let val_str = match val {
            Json::String(s) => s,
            Json::Number(n) => n.to_string(),
            Json::Bool(b) => b.to_string(),
            Json::Null => "null".to_string(),
            _ => val.to_string(),
        };
        // serde_flattened использует __ как разделитель, меняем на .
        let clean_key = key.replace("__", ".");
        flat_map.insert(clean_key, val_str);
    }
    
    // 3. Сериализуем в INI
    serde_ini::to_string(&flat_map)
        .map_err(|e| format!("INI: {}", e))
}


pub fn parse_ini(input: &str) -> Result<Json, String> {
    let mut raw_map: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    let mut current_section = String::new();
    
    // 🔥 Улучшенный парсинг INI — пропускаем строки без '='
    for line in input.lines() {
        let line = line.trim();
        // Пропускаем пустые строки и комментарии
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        // Секция
        if line.starts_with('[') && line.ends_with(']') {
            current_section = line[1..line.len()-1].trim().to_string();
            continue;
        }
        // 🔥 Пропускаем строки без '='
        if !line.contains('=') {
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
    
    // 🔥 Используем наш ручной парсинг вместо serde_ini::from_str
    // serde_ini падает на строках без '=', а мы уже их отфильтровали
    
    // Строим JSON из raw_map
    let mut flat: Map<String, Json> = Map::new();
    
    for (key, values) in &raw_map {
        if values.len() == 1 {
            // Одиночное значение
            flat.insert(key.clone(), unquote_value(&Json::String(values[0].clone())));
        } else {
            // Массив значений (дубликаты ключей)
            let arr: Vec<Json> = values.iter()
                .map(|s| unquote_value(&Json::String(s.clone())))
                .collect();
            flat.insert(key.clone(), Json::Array(arr));
        }
    }
    
    let has_nesting = flat.keys().any(|k| k.contains('.'));
    
    if !has_nesting {
        let mut result = serde_json::Map::new();
        for (key, val) in &flat {
            result.insert(key.clone(), val.clone());
        }
        return Ok(Json::Object(result));
    }
    
    // Используем json_unflattening для восстановления структуры
    let dot_flat: Map<String, Json> = flat.clone();
    
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


