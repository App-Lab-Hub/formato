// src-tauri/src/utils.rs
use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use crate::paths::{converted_dir, temp_dir};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileInfo {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub created: String,
    pub file_type: String, // "converted" или "temp"
}

#[tauri::command]
pub fn get_files() -> Result<Vec<FileInfo>, String> {
    let mut files = Vec::new();
    
    // Получаем файлы из converted_dir
    let converted = converted_dir();
    if converted.exists() {
        for entry in fs::read_dir(&converted).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            if path.is_file() {
                let metadata = fs::metadata(&path).map_err(|e| e.to_string())?;
                let name = path.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                
                files.push(FileInfo {
                    name,
                    path: path.to_string_lossy().to_string(),
                    size: metadata.len(),
                    created: metadata.created()
                        .map(|t| format!("{:?}", t))
                        .unwrap_or_else(|_| "Unknown".to_string()),
                    file_type: "converted".to_string(),
                });
            }
        }
    }
    
    // Получаем файлы из temp_dir
    let temp = temp_dir();
    if temp.exists() {
        for entry in fs::read_dir(&temp).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            if path.is_file() {
                let metadata = fs::metadata(&path).map_err(|e| e.to_string())?;
                let name = path.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                
                files.push(FileInfo {
                    name,
                    path: path.to_string_lossy().to_string(),
                    size: metadata.len(),
                    created: metadata.created()
                        .map(|t| format!("{:?}", t))
                        .unwrap_or_else(|_| "Unknown".to_string()),
                    file_type: "temp".to_string(),
                });
            }
        }
    }
    
    Ok(files)
}