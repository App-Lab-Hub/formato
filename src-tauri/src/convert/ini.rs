

use serde_flattened::flatten_json_value::flatten::flattened;
use indexmap::IndexMap;
use serde_json::Map;
use serde_json::{Value as Json, json};

pub fn stringify_ini(value: &Json) -> Result<String, String> {
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

pub fn parse_ini(input: &str) -> Result<Json, String> {
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


