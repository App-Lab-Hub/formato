
use docx_rs::*;
use std::fs::File;
use serde_json::{Value as Json, json};

use std::io::Read;
use crate::convert::calculate_conversion_hash;
use crate::convert::get_app_dir_path_with_hash;

pub fn stringify_docx(value: &Json, path: &str, from: &str, to: &str) -> Result<String, String> {
    let pretty_json_text = serde_json::to_string_pretty(&value)
        .map_err(|e| format!("JSON serialize error: {}", e))?;

    let mut doc = Docx::new();

    for line in pretty_json_text.lines() {
        doc = doc.add_paragraph(Paragraph::new().add_run(Run::new().add_text(line)));
    }

    let hash = calculate_conversion_hash(path, from, to)
        .map_err(|e| format!("Cannot hash file: {}", e))?;

    let output_path = get_app_dir_path_with_hash(path, to, &hash)?;

    let file = File::create(&output_path)
        .map_err(|e| format!("Cannot create file: {}", e))?;

    doc.build()
        .pack(file)
        .map_err(|e| format!("DOCX pack error: {}", e))?;

    Ok(output_path)
}
    

use std::fs;
use docx_rs::{read_docx, DocumentChild, ParagraphChild, RunChild};

pub fn parse_docx(path: &str) -> Result<Json, String> {
    let buf = fs::read(path)
        .map_err(|e| format!("Cannot read file: {}", e))?;
    
    let docx = read_docx(&buf)
        .map_err(|e| format!("DOCX parse error: {}", e))?;
    
    let text = extract_text_from_docx(&docx.document);
    
    let chars: Vec<String> = text.chars().map(|c| c.to_string()).collect();
    let char_count = chars.len();
    let word_count = text.split_whitespace().count();
    let line_count = text.lines().count();
    
    let paragraphs: Vec<String> = text
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|s| s.to_string())
        .collect();
    let paragraph_count = paragraphs.len();
    
    Ok(json!({
        "text": text,
        "paragraphs": paragraphs,
        "char_count": char_count,
        "word_count": word_count,
        "line_count": line_count,
        "paragraph_count": paragraph_count,
    }))
}

fn extract_text_from_docx(document: &docx_rs::Document) -> String {
    let mut text_parts = Vec::new();
    
    for child in &document.children {
        if let DocumentChild::Paragraph(paragraph) = child {
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
            
            if !para_text.trim().is_empty() {
                text_parts.push(para_text);
            }
        }
    }
    
    text_parts.join("\n")
}