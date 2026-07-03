// src-tauri/src/archive.rs
use std::path::PathBuf;
use tauri::async_runtime;
use zippylib::{
    create_zip_archive,
    create_tar_gz_archive,
    create_tar_xz_archive,
};
use tokio::fs;

#[tauri::command]
pub async fn archive_file(source_path: String, output_path: String, format: String) -> Result<(), String> {
    let source_full = PathBuf::from(&source_path);
    let output = PathBuf::from(&output_path);
    
    let file_name = source_full
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| "Invalid file name".to_string())?
        .to_string();

    // Для TAR форматов копируем файл асинхронно
    let temp_file = match format.as_str() {
        "tar.gz" | "tar.xz" => {
            let current_dir = std::env::current_dir()
                .map_err(|e| format!("Failed to get current dir: {}", e))?;
            let local_path = current_dir.join(&file_name);
            
            // Асинхронное копирование
            fs::copy(&source_full, &local_path)
                .await
                .map_err(|e| format!("Failed to copy file: {}", e))?;
            
            Some(local_path)
        }
        _ => None,
    };

    // Архивируем в spawn_blocking (синхронная часть)
    let result = async_runtime::spawn_blocking(move || {
        let files = match format.as_str() {
            "zip" => vec![source_full],
            "tar.gz" | "tar.xz" => vec![PathBuf::from(&file_name)],
            _ => return Err(format!("Unsupported format: {}", format)),
        };
        
        let result = match format.as_str() {
            "zip" => create_zip_archive(&files, output)
                .map_err(|e| format!("Zip error: {}", e)),
            "tar.gz" => create_tar_gz_archive(&files, output)
                .map_err(|e| format!("Tar.gz error: {}", e)),
            "tar.xz" => create_tar_xz_archive(&files, output)
                .map_err(|e| format!("Tar.xz error: {}", e)),
            _ => Err(format!("Unsupported format: {}", format)),
        };
        
        // Удаляем временный файл синхронно (он в temp_file, но move замыкание)
        if let Some(path) = temp_file {
            let _ = std::fs::remove_file(&path);
        }
        
        result
    })
    .await
    .map_err(|e| format!("Background task failed: {}", e))?;

    result
}