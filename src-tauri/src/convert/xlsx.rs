// src/convert/xlsx.rs

use calamine::{Reader, Xlsx, DataType};
use rust_xlsxwriter::*;
use serde_json::{Value as Json};
use std::fs::File;
use std::io::Write;
use crate::convert::calculate_conversion_hash;
use crate::convert::get_app_dir_path_with_hash;



pub fn stringify_xlsx(value: &Json, path: &str, from: &str, to: &str) -> Result<String, String> {
    // Рекурсивно собираем все ключи
    fn collect_keys(value: &Json, prefix: &str, keys: &mut Vec<String>) {
        match value {
            Json::Object(obj) => {
                for (key, val) in obj {
                    let new_prefix = if prefix.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", prefix, key)
                    };
                    collect_keys(val, &new_prefix, keys);
                }
            }
            Json::Array(arr) => {
                if let Some(first) = arr.first() {
                    collect_keys(first, prefix, keys);
                }
            }
            _ => {
                if !keys.contains(&prefix.to_string()) {
                    keys.push(prefix.to_string());
                }
            }
        }
    }
    
    // Преобразуем JSON в плоские строки
    fn flatten_json(value: &Json, prefix: &str) -> Vec<String> {
        match value {
            Json::Object(obj) => {
                let mut row = Vec::new();
                for (key, val) in obj {
                    let new_prefix = if prefix.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", prefix, key)
                    };
                    
                    if let Json::Object(_) = val {
                        let nested = flatten_json(val, &new_prefix);
                        row.extend(nested);
                    } else if let Json::Array(arr) = val {
                        if let Some(first) = arr.first() {
                            let nested = flatten_json(first, &new_prefix);
                            row.extend(nested);
                        } else {
                            row.push("[]".to_string());
                        }
                    } else {
                        row.push(val.to_string());
                    }
                }
                row
            }
            Json::Array(arr) => {
                let mut rows = Vec::new();
                for item in arr {
                    let row = flatten_json(item, prefix);
                    rows.extend(row);
                }
                rows
            }
            _ => {
                vec![value.to_string()]
            }
        }
    }
    
    // Определяем структуру для заголовков
    let mut headers = Vec::new();
    collect_keys(&value, "", &mut headers);
    
    // Преобразуем JSON в строки
    let rows = flatten_json(&value, "");
    
    // Создаём XLSX
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    
    // Заголовки
    for (col_idx, header) in headers.iter().enumerate() {
        worksheet.write_string(0, col_idx as u16, header.as_str())
            .map_err(|e| format!("XLSX write error: {}", e))?;
    }
    
    // Данные (1 строка — значения)
    for (col_idx, cell) in rows.iter().enumerate() {
        worksheet.write_string(1, col_idx as u16, cell.as_str())
            .map_err(|e| format!("XLSX write error: {}", e))?;
    }
    
    // Для массивов объектов — каждая строка отдельно
    if let Json::Array(arr) = &value {
        let mut first = true;
        let mut row_idx = 2;
        for item in arr {
            if let Json::Object(obj) = item {
                let row_data = flatten_json(&Json::Object(obj.clone()), "");
                let current_row = if first { 1 } else { row_idx };
                for (col_idx, cell) in row_data.iter().enumerate() {
                    worksheet.write_string(current_row, col_idx as u16, cell.as_str())
                        .map_err(|e| format!("XLSX write error: {}", e))?;
                }
                first = false;
                row_idx += 1;
            }
        }
    }
    
    // Автоподгонка
    for col_idx in 0..headers.len() {
        worksheet.set_column_width(col_idx as u16, 20)
            .map_err(|e| format!("XLSX set column width error: {}", e))?;
    }
    
    // Сохраняем в буфер
    let buffer = workbook.save_to_buffer()
        .map_err(|e| format!("Failed to save XLSX: {}", e))?;
    
    let hash = calculate_conversion_hash(path, from, to)
        .map_err(|e| format!("Cannot hash file: {}", e))?;
    
    let output_path = get_app_dir_path_with_hash(path, to, &hash)?;
    let mut file = File::create(&output_path)
        .map_err(|e| format!("Cannot create file: {}", e))?;
    
    file.write_all(&buffer)
        .map_err(|e| format!("Cannot write file: {}", e))?;
    
    Ok(output_path)
}

/// Парсит XLSX в JSON
pub fn parse_xlsx(path: &str) -> Result<Json, String> {

    
    let mut workbook: Xlsx<_> = calamine::open_workbook(path)
        .map_err(|e| format!("XLSX parse error: {}", e))?;
    
    let mut result = Vec::new();
    
    // Проходим по всем листам
    for sheet_name in workbook.sheet_names() {
        let mut sheet_data = Vec::new();
        if let Ok(range) = workbook.worksheet_range(&sheet_name) {
            let mut headers = Vec::new();
            for (row_idx, row) in range.rows().enumerate() {
                let mut row_data = Vec::new();
                for cell in row {
                    row_data.push(cell.to_string());
                }
                if row_idx == 0 {
                    headers = row_data.clone();
                }
                sheet_data.push(row_data);
            }
            
            // Преобразуем в JSON
            let mut sheet_json = Vec::new();
            for (idx, row) in sheet_data.iter().enumerate() {
                if idx == 0 { continue; } // Пропускаем заголовки
                let mut obj = serde_json::Map::new();
                for (col_idx, cell) in row.iter().enumerate() {
                    let header = if col_idx < headers.len() { 
                        headers[col_idx].clone() 
                    } else { 
                        format!("col_{}", col_idx + 1) 
                    };
                    obj.insert(header, Json::String(cell.clone()));
                }
                if !obj.is_empty() {
                    sheet_json.push(Json::Object(obj));
                }
            }
            
            let mut sheet_map = serde_json::Map::new();
            sheet_map.insert("name".to_string(), Json::String(sheet_name.clone()));
            sheet_map.insert("data".to_string(), Json::Array(sheet_json));
            result.push(Json::Object(sheet_map));
        }
    }
    
    Ok(Json::Array(result))
}