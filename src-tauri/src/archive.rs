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
pub async fn archive_file(
    source_path: String, 
    output_path: String, 
    format: String,
    name_in_archive: String,
) -> Result<(), String> {
    let source_full = PathBuf::from(&source_path);
    let output = PathBuf::from(&output_path);
    
    let file_name = name_in_archive;

    // Для TAR форматов копируем файл с новым именем в текущую директорию
    let temp_file = match format.as_str() {
        "tar.gz" | "tar.xz" => {
            let current_dir = std::env::current_dir()
                .map_err(|e| format!("Failed to get current dir: {}", e))?;
            let local_path = current_dir.join(&file_name);
            
            fs::copy(&source_full, &local_path)
                .await
                .map_err(|e| format!("Failed to copy file: {}", e))?;
            
            Some(local_path)
        }
        "zip" => {
            let current_dir = std::env::current_dir()
                .map_err(|e| format!("Failed to get current dir: {}", e))?;
            let temp_dir = current_dir.join("temp_zip_file");
            std::fs::create_dir_all(&temp_dir)
                .map_err(|e| format!("Failed to create temp dir: {}", e))?;
            
            let local_path = temp_dir.join(&file_name);
            
            fs::copy(&source_full, &local_path)
                .await
                .map_err(|e| format!("Failed to copy file: {}", e))?;
            
            Some(local_path)
        }
        _ => None,
    };

    let result = async_runtime::spawn_blocking(move || {
        let files = match format.as_str() {
            "zip" => {
                if let Some(ref path) = temp_file {
                    vec![path.clone()]
                } else {
                    vec![source_full]
                }
            }
            "tar.gz" | "tar.xz" => {
                if let Some(ref path) = temp_file {
                    // Для TAR используем только имя файла (относительный путь)
                    let name = path.file_name()
                        .and_then(|n| n.to_str())
                        .map(PathBuf::from)
                        .unwrap_or_default();
                    vec![name]
                } else {
                    vec![source_full]
                }
            }
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
        
        if let Some(path) = temp_file {
            let _ = std::fs::remove_file(&path);
            // Удаляем временную директорию для ZIP
            if format == "zip" {
                if let Some(parent) = path.parent() {
                    let _ = std::fs::remove_dir(parent);
                }
            }
        }
        
        result
    })
    .await
    .map_err(|e| format!("Background task failed: {}", e))?;

    result
}

#[tauri::command]
pub async fn archive_multiple_files(
    files: Vec<serde_json::Value>, 
    output_path: String, 
    format: String
) -> Result<(), String> {
    let output = PathBuf::from(&output_path);
    
    let mut files_with_names: Vec<(PathBuf, String)> = Vec::new();
    for item in files {
        let path = item.get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing path".to_string())?;
        let name = item.get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing name".to_string())?;
        files_with_names.push((PathBuf::from(path), name.to_string()));
    }

    // Для ZIP и TAR форматов копируем файлы с новыми именами
    let temp_files: Option<Vec<PathBuf>> = match format.as_str() {
        "zip" | "tar.gz" | "tar.xz" => {
            let current_dir = std::env::current_dir()
                .map_err(|e| format!("Failed to get current dir: {}", e))?;
            
            // Для ZIP создаём временную директорию
            let temp_dir = if format == "zip" {
                let dir = current_dir.join("temp_zip");
                std::fs::create_dir_all(&dir)
                    .map_err(|e| format!("Failed to create temp dir: {}", e))?;
                Some(dir)
            } else {
                None
            };
            
            let mut temp_paths = Vec::new();
            for (source_path, new_name) in &files_with_names {
                let local_path = if let Some(ref dir) = temp_dir {
                    dir.join(new_name)
                } else {
                    current_dir.join(new_name)
                };
                
                fs::copy(source_path, &local_path)
                    .await
                    .map_err(|e| format!("Failed to copy file: {}", e))?;
                temp_paths.push(local_path);
            }
            Some(temp_paths)
        }
        _ => None,
    };

    let result = async_runtime::spawn_blocking(move || {
        let result = match format.as_str() {
            "zip" => {
                let paths: Vec<PathBuf> = if let Some(ref temps) = temp_files {
                    temps.clone()
                } else {
                    files_with_names.iter().map(|(path, _)| path.clone()).collect()
                };
                create_zip_archive(&paths, output)
                    .map_err(|e| format!("Zip error: {}", e))
            }
            "tar.gz" => {
                let names: Vec<PathBuf> = if let Some(ref temps) = temp_files {
                    temps.iter()
                        .map(|p| {
                            p.file_name()
                                .and_then(|n| n.to_str())
                                .map(PathBuf::from)
                                .unwrap_or_default()
                        })
                        .collect()
                } else {
                    files_with_names.iter().map(|(_, name)| PathBuf::from(name)).collect()
                };
                create_tar_gz_archive(&names, output)
                    .map_err(|e| format!("Tar.gz error: {}", e))
            }
            "tar.xz" => {
                let names: Vec<PathBuf> = if let Some(ref temps) = temp_files {
                    temps.iter()
                        .map(|p| {
                            p.file_name()
                                .and_then(|n| n.to_str())
                                .map(PathBuf::from)
                                .unwrap_or_default()
                        })
                        .collect()
                } else {
                    files_with_names.iter().map(|(_, name)| PathBuf::from(name)).collect()
                };
                create_tar_xz_archive(&names, output)
                    .map_err(|e| format!("Tar.xz error: {}", e))
            }
            _ => Err(format!("Unsupported format: {}", format)),
        };
        
        if let Some(paths) = temp_files {
            for path in &paths {
                let _ = std::fs::remove_file(path);
            }
            // Удаляем временную директорию для ZIP
            if format == "zip" {
                if let Some(parent) = paths.first().and_then(|p| p.parent()) {
                    let _ = std::fs::remove_dir(parent);
                }
            }
        }
        
        result
    })
    .await
    .map_err(|e| format!("Background task failed: {}", e))?;

    result
}


