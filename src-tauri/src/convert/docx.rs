
use docx_rs::*;
use std::fs::File;
use serde_json::{Value as Json};

use crate::convert::calculate_conversion_hash;
use crate::convert::get_app_dir_path_with_hash;

/// Создает DOCX из текстовой строки с кастомными отступами
pub fn stringify_docx(text: &str, path: &str, from: &str, to: &str) -> Result<String, String> {
    // 1. Создаем конфигурацию полей (например, по ~1.5 см со всех сторон)
    let margin = PageMargin::new()
        .top(850)
        .bottom(850)
        .left(850)
        .right(850);

    // 2. Инициализируем документ и применяем кастомные поля
    let mut doc = Docx::new().page_margin(margin);

    // Разбиваем текст на строки и добавляем параграфы
    for line in text.lines() {
        doc = doc.add_paragraph(Paragraph::new().add_run(Run::new().add_text(line)));
    }

    let hash = calculate_conversion_hash(path, from, to)
        .map_err(|e| format!("Cannot hash file: {}", e))?;

    let output_path = get_app_dir_path_with_hash(path, to, &hash, true)?;

    let file = File::create(&output_path)
        .map_err(|e| format!("Cannot create file: {}", e))?;

    doc.build()
        .pack(file)
        .map_err(|e| format!("DOCX pack error: {}", e))?;

    Ok(output_path)
}
    

use std::fs;
use docx_rs::{read_docx, DocumentChild, ParagraphChild, RunChild, Table, TableChild, TableRowChild, TableCellContent};

pub fn parse_docx(path: &str) -> Result<Json, String> {
    let buf = fs::read(path)
        .map_err(|e| format!("Cannot read file: {}", e))?;
    
    let docx = read_docx(&buf)
        .map_err(|e| format!("DOCX parse error: {}", e))?;
    
    let mut paragraphs: Vec<String> = Vec::new();
    let mut tables: Vec<Json> = Vec::new();
    let mut images: Vec<Json> = Vec::new();
    let mut lists: Vec<Json> = Vec::new();
    
    parse_docx_content(&docx.document, &mut paragraphs, &mut tables, &mut images, &mut lists);
    
    let full_text: String = paragraphs.join("\n");
    let chars: Vec<String> = full_text.chars().map(|c| c.to_string()).collect();
    let char_count = chars.len();
    let word_count = full_text.split_whitespace().count();
    let line_count = full_text.lines().count();
    let paragraph_count = paragraphs.len();
    
    let mut result = serde_json::Map::new();
    result.insert("text".to_string(), Json::String(full_text));
    result.insert("paragraphs".to_string(), Json::Array(paragraphs.into_iter().map(Json::String).collect()));
    result.insert("char_count".to_string(), Json::Number(serde_json::Number::from(char_count)));
    result.insert("word_count".to_string(), Json::Number(serde_json::Number::from(word_count)));
    result.insert("line_count".to_string(), Json::Number(serde_json::Number::from(line_count)));
    result.insert("paragraph_count".to_string(), Json::Number(serde_json::Number::from(paragraph_count)));
    
    if !images.is_empty() {
        result.insert("images".to_string(), Json::Array(images));
    }
    if !tables.is_empty() {
        result.insert("tables".to_string(), Json::Array(tables));
    }
    if !lists.is_empty() {
        result.insert("lists".to_string(), Json::Array(lists));
    }
    
    Ok(Json::Object(result))
}

fn parse_docx_content(
    document: &docx_rs::Document,
    paragraphs: &mut Vec<String>,
    tables: &mut Vec<Json>,
    images: &mut Vec<Json>,
    lists: &mut Vec<Json>,
) {
    for child in &document.children {
        match child {
            DocumentChild::Paragraph(paragraph) => {
                let para_text = extract_paragraph_text(paragraph);
                if !para_text.trim().is_empty() {
                    paragraphs.push(para_text);
                }
            }
            DocumentChild::Table(table) => {
                let table_data = parse_table(table);
                if !table_data.is_empty() {
                    tables.push(Json::Array(table_data));
                }
            }
            _ => {}
        }
    }
}

fn extract_paragraph_text(paragraph: &docx_rs::Paragraph) -> String {
    let mut para_text = String::new();
    
    for p_child in &paragraph.children {
        if let ParagraphChild::Run(run) = p_child {
            for r_child in &run.children {
                if let RunChild::Text(text) = r_child {
                    para_text.push_str(&text.text);
                }
            }
        }
    }
    
    para_text
}

/// Парсинг таблицы
fn parse_table(table: &Table) -> Vec<Json> {
    let mut rows_data = Vec::new();
    
    for table_child in &table.rows {
        // Table rows are wrapped in TableChild
        if let TableChild::TableRow(row) = table_child {
            let mut row_cells = Vec::new();
            
            for row_child in &row.cells {
                // Row cells are wrapped in TableRowChild
                if let TableRowChild::TableCell(cell) = row_child {
                    let mut cell_text = String::new();
                    
                    // В TableCell дети имеют тип TableCellContent
                    for content in &cell.children {
                        if let TableCellContent::Paragraph(paragraph) = content {
                            let para_text = extract_paragraph_text(paragraph);
                            if !para_text.trim().is_empty() {
                                cell_text.push_str(&para_text);
                            }
                        }
                    }
                    
                    if !cell_text.trim().is_empty() {
                        row_cells.push(Json::String(cell_text.trim().to_string()));
                    }
                }
            }
            
            if !row_cells.is_empty() {
                rows_data.push(Json::Array(row_cells));
            }
        }
    }
    
    rows_data
}