// src/convert/xlsx.rs


use rust_xlsxwriter::*;
use serde_json::{Value as Json};
use std::fs::File;
use std::io::Write;
use crate::convert::calculate_conversion_hash;
use crate::convert::get_app_dir_path_with_hash;


/// Создает XLSX из текстовой строки
pub fn stringify_xlsx(text: &str, path: &str, from: &str, to: &str) -> Result<String, String> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    // Разбиваем текст на строки и записываем каждую строку в отдельную ячейку
    let lines: Vec<&str> = text.lines().collect();
    
    for (row_idx, line) in lines.iter().enumerate() {
        // Записываем всю строку в первую колонку
        worksheet.write_string(row_idx as u32, 0, *line)
            .map_err(|e| format!("XLSX write error: {}", e))?;
    }

    // Автоподгонка ширины колонки
    worksheet.set_column_width(0, 50)
        .map_err(|e| format!("XLSX set column width error: {}", e))?;

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



use calamine::{open_workbook_auto, Reader, Xlsx, Xls};

pub fn parse_xlsx(path: &str) -> Result<Json, String> {
    // Автоматически определяет формат (xls, xlsx, ods)
    let mut workbook = open_workbook_auto(path)
        .map_err(|e| format!("Spreadsheet parse error: {}", e))?;
    
    let mut result = Vec::new();
    
    for sheet_name in workbook.sheet_names() {
        let mut sheet_data = Vec::new();
        if let Ok(range) = workbook.worksheet_range(&sheet_name) {
            let mut headers = Vec::new();
            for (row_idx, row) in range.rows().enumerate() {
                let mut row_data: Vec<String> = row.iter().map(|c| c.to_string()).collect();
                if row_idx == 0 {
                    headers = row_data.clone();
                }
                sheet_data.push(row_data);
            }
            
            let mut sheet_json = Vec::new();
            for (idx, row) in sheet_data.iter().enumerate() {
                if idx == 0 { continue; }
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