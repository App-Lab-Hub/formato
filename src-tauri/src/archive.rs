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


#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::{Path, PathBuf};
    use tempfile::tempdir;

    // ============================================================
    // ВСПОМОГАТЕЛЬНЫЕ ФУНКЦИИ
    // ============================================================

    fn get_fixture_files(ext: &str) -> Vec<PathBuf> {
        let fixtures_dir = PathBuf::from("../fixtures");
        if !fixtures_dir.exists() {
            return vec![];
        }
        
        let entries = fs::read_dir(&fixtures_dir).unwrap();
        let mut files = Vec::new();
        for entry in entries {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file() {
                if let Some(extension) = path.extension() {
                    if extension == ext {
                        files.push(path);
                    }
                }
            }
        }
        files
    }

    fn has_fixtures(ext: &str) -> bool {
        !get_fixture_files(ext).is_empty()
    }

    /// Проверяет, что архив существует и не пустой
    fn verify_archive_exists(archive_path: &Path) -> Result<(), String> {
        if !archive_path.exists() {
            return Err(format!("Archive not found: {:?}", archive_path));
        }
        
        let metadata = fs::metadata(archive_path)
            .map_err(|e| format!("Cannot get metadata: {}", e))?;
        
        if metadata.len() == 0 {
            return Err("Archive is empty".to_string());
        }
        
        Ok(())
    }

    // ============================================================
    // ТЕСТЫ: archive_file
    // ============================================================

    #[tokio::test]
    async fn test_archive_file_zip_from_fixtures() {
        if !has_fixtures("html") {
            println!("⚠️ Skipping test: no HTML fixtures found");
            return;
        }
        
        let files = get_fixture_files("html");
        let source = &files[0];
        let temp_dir = tempdir().unwrap();
        let output = temp_dir.path().join("archive.zip");
        
        let result = archive_file(
            source.to_string_lossy().to_string(),
            output.to_string_lossy().to_string(),
            "zip".to_string(),
            "test.html".to_string(),
        ).await;
        
        assert!(result.is_ok(), "Archive failed: {:?}", result.err());
        
        if let Err(e) = verify_archive_exists(&output) {
            panic!("Archive verification failed: {}", e);
        }
        println!("✅ Archive created: {} ({} bytes)", 
            output.display(), 
            fs::metadata(&output).unwrap().len()
        );
    }

    #[tokio::test]
    async fn test_archive_file_tar_gz_from_fixtures() {
        if !has_fixtures("docx") {
            println!("⚠️ Skipping test: no DOCX fixtures found");
            return;
        }
        
        let files = get_fixture_files("docx");
        let source = &files[0];
        let temp_dir = tempdir().unwrap();
        let output = temp_dir.path().join("archive.tar.gz");
        
        let result = archive_file(
            source.to_string_lossy().to_string(),
            output.to_string_lossy().to_string(),
            "tar.gz".to_string(),
            "document.docx".to_string(),
        ).await;
        
        assert!(result.is_ok(), "Archive failed: {:?}", result.err());
        
        if let Err(e) = verify_archive_exists(&output) {
            panic!("Archive verification failed: {}", e);
        }
        println!("✅ Archive created: {} ({} bytes)", 
            output.display(), 
            fs::metadata(&output).unwrap().len()
        );
    }

    #[tokio::test]
    async fn test_archive_file_tar_xz_from_fixtures() {
        if !has_fixtures("pdf") {
            println!("⚠️ Skipping test: no PDF fixtures found");
            return;
        }
        
        let files = get_fixture_files("pdf");
        let source = &files[0];
        let temp_dir = tempdir().unwrap();
        let output = temp_dir.path().join("archive.tar.xz");
        
        let result = archive_file(
            source.to_string_lossy().to_string(),
            output.to_string_lossy().to_string(),
            "tar.xz".to_string(),
            "document.pdf".to_string(),
        ).await;
        
        assert!(result.is_ok(), "Archive failed: {:?}", result.err());
        
        if let Err(e) = verify_archive_exists(&output) {
            panic!("Archive verification failed: {}", e);
        }
        println!("✅ Archive created: {} ({} bytes)", 
            output.display(), 
            fs::metadata(&output).unwrap().len()
        );
    }

    // ============================================================
    // ТЕСТЫ: archive_multiple_files
    // ============================================================

    #[tokio::test]
    async fn test_archive_multiple_files_zip_from_fixtures() {
        if !has_fixtures("json") || !has_fixtures("csv") {
            println!("⚠️ Skipping test: need JSON and CSV fixtures");
            return;
        }
        
        let json_files = get_fixture_files("json");
        let csv_files = get_fixture_files("csv");
        
        let temp_dir = tempdir().unwrap();
        let output = temp_dir.path().join("multiple.zip");
        
        let files_data = vec![
            (json_files[0].clone(), "test.json".to_string()),
            (csv_files[0].clone(), "config.csv".to_string()),
        ];
        
        let files_json: Vec<serde_json::Value> = files_data.iter()
            .map(|(path, name)| {
                serde_json::json!({
                    "path": path.to_string_lossy().to_string(),
                    "name": name
                })
            })
            .collect();
        
        let result = archive_multiple_files(
            files_json,
            output.to_string_lossy().to_string(),
            "zip".to_string(),
        ).await;
        
        assert!(result.is_ok(), "Archive failed: {:?}", result.err());
        
        if let Err(e) = verify_archive_exists(&output) {
            panic!("Archive verification failed: {}", e);
        }
        println!("✅ Archive created: {} ({} bytes)", 
            output.display(), 
            fs::metadata(&output).unwrap().len()
        );
    }

    #[tokio::test]
    async fn test_archive_multiple_files_tar_gz_from_fixtures() {
        if !has_fixtures("ini") || get_fixture_files("ini").len() < 2 {
            println!("⚠️ Skipping test: need at least 2 INI fixtures");
            return;
        }
        
        let ini_files = get_fixture_files("ini");
        let temp_dir = tempdir().unwrap();
        let output = temp_dir.path().join("multiple.tar.gz");
        
        let files_data = vec![
            (ini_files[0].clone(), "config1.ini".to_string()),
            (ini_files[1].clone(), "config2.ini".to_string()),
        ];
        
        let files_json: Vec<serde_json::Value> = files_data.iter()
            .map(|(path, name)| {
                serde_json::json!({
                    "path": path.to_string_lossy().to_string(),
                    "name": name
                })
            })
            .collect();
        
        let result = archive_multiple_files(
            files_json,
            output.to_string_lossy().to_string(),
            "tar.gz".to_string(),
        ).await;
        
        assert!(result.is_ok(), "Archive failed: {:?}", result.err());
        
        if let Err(e) = verify_archive_exists(&output) {
            panic!("Archive verification failed: {}", e);
        }
        println!("✅ Archive created: {} ({} bytes)", 
            output.display(), 
            fs::metadata(&output).unwrap().len()
        );
    }

    #[tokio::test]
    async fn test_archive_multiple_files_mixed_from_fixtures() {
        let extensions = ["json", "yaml", "csv", "xml", "toml"];
        let mut files = Vec::new();
        
        for ext in extensions {
            let mut ext_files = get_fixture_files(ext);
            if !ext_files.is_empty() {
                files.push(ext_files.remove(0));
            }
        }
        
        if files.len() < 2 {
            println!("⚠️ Skipping test: need at least 2 fixtures of different types");
            return;
        }
        
        let temp_dir = tempdir().unwrap();
        let output = temp_dir.path().join("mixed.zip");
        
        let files_data: Vec<(PathBuf, String)> = files.iter()
            .enumerate()
            .map(|(i, path)| {
                let name = format!("file_{}.{}", i, path.extension().unwrap().to_string_lossy());
                (path.clone(), name)
            })
            .collect();
        
        let files_json: Vec<serde_json::Value> = files_data.iter()
            .map(|(path, name)| {
                serde_json::json!({
                    "path": path.to_string_lossy().to_string(),
                    "name": name
                })
            })
            .collect();
        
        let result = archive_multiple_files(
            files_json,
            output.to_string_lossy().to_string(),
            "zip".to_string(),
        ).await;
        
        assert!(result.is_ok(), "Archive failed: {:?}", result.err());
        
        if let Err(e) = verify_archive_exists(&output) {
            panic!("Archive verification failed: {}", e);
        }
        println!("✅ Archive created: {} ({} bytes)", 
            output.display(), 
            fs::metadata(&output).unwrap().len()
        );
    }

    // ============================================================
    // ТЕСТЫ: Базовые
    // ============================================================

    #[tokio::test]
    async fn test_archive_file_zip() {
        let temp_dir = tempdir().unwrap();
        let source = temp_dir.path().join("source.txt");
        fs::write(&source, "Test content for ZIP").unwrap();
        let output = temp_dir.path().join("output.zip");
        
        let result = archive_file(
            source.to_string_lossy().to_string(),
            output.to_string_lossy().to_string(),
            "zip".to_string(),
            "renamed.txt".to_string(),
        ).await;
        
        assert!(result.is_ok(), "Archive failed: {:?}", result.err());
        
        if let Err(e) = verify_archive_exists(&output) {
            panic!("Archive verification failed: {}", e);
        }
        println!("✅ Archive created: {} ({} bytes)", 
            output.display(), 
            fs::metadata(&output).unwrap().len()
        );
    }

    #[tokio::test]
    async fn test_archive_file_tar_gz() {
        let temp_dir = tempdir().unwrap();
        let source = temp_dir.path().join("source.txt");
        fs::write(&source, "Test content for TAR.GZ").unwrap();
        let output = temp_dir.path().join("output.tar.gz");
        
        let result = archive_file(
            source.to_string_lossy().to_string(),
            output.to_string_lossy().to_string(),
            "tar.gz".to_string(),
            "renamed.txt".to_string(),
        ).await;
        
        assert!(result.is_ok(), "Archive failed: {:?}", result.err());
        
        if let Err(e) = verify_archive_exists(&output) {
            panic!("Archive verification failed: {}", e);
        }
        println!("✅ Archive created: {} ({} bytes)", 
            output.display(), 
            fs::metadata(&output).unwrap().len()
        );
    }

    #[tokio::test]
    async fn test_archive_file_tar_xz() {
        let temp_dir = tempdir().unwrap();
        let source = temp_dir.path().join("source.txt");
        fs::write(&source, "Test content for TAR.XZ").unwrap();
        let output = temp_dir.path().join("output.tar.xz");
        
        let result = archive_file(
            source.to_string_lossy().to_string(),
            output.to_string_lossy().to_string(),
            "tar.xz".to_string(),
            "renamed.txt".to_string(),
        ).await;
        
        assert!(result.is_ok(), "Archive failed: {:?}", result.err());
        
        if let Err(e) = verify_archive_exists(&output) {
            panic!("Archive verification failed: {}", e);
        }
        println!("✅ Archive created: {} ({} bytes)", 
            output.display(), 
            fs::metadata(&output).unwrap().len()
        );
    }

    #[tokio::test]
    async fn test_archive_multiple_files_zip() {
        let temp_dir = tempdir().unwrap();
        let file1 = temp_dir.path().join("file1.txt");
        let file2 = temp_dir.path().join("file2.txt");
        fs::write(&file1, "Hello from file1").unwrap();
        fs::write(&file2, "Hello from file2").unwrap();
        
        let output = temp_dir.path().join("multiple.zip");
        
        let files = vec![
            (file1, "renamed1.txt".to_string()),
            (file2, "renamed2.txt".to_string()),
        ];
        
        let files_json: Vec<serde_json::Value> = files.iter()
            .map(|(path, name)| {
                serde_json::json!({
                    "path": path.to_string_lossy().to_string(),
                    "name": name
                })
            })
            .collect();
        
        let result = archive_multiple_files(
            files_json,
            output.to_string_lossy().to_string(),
            "zip".to_string(),
        ).await;
        
        assert!(result.is_ok(), "Archive failed: {:?}", result.err());
        
        if let Err(e) = verify_archive_exists(&output) {
            panic!("Archive verification failed: {}", e);
        }
        println!("✅ Archive created: {} ({} bytes)", 
            output.display(), 
            fs::metadata(&output).unwrap().len()
        );
    }

    #[tokio::test]
    async fn test_archive_multiple_files_tar_gz() {
        let temp_dir = tempdir().unwrap();
        let file1 = temp_dir.path().join("file1.txt");
        let file2 = temp_dir.path().join("file2.txt");
        fs::write(&file1, "Hello from file1").unwrap();
        fs::write(&file2, "Hello from file2").unwrap();
        
        let output = temp_dir.path().join("multiple.tar.gz");
        
        let files = vec![
            (file1, "renamed1.txt".to_string()),
            (file2, "renamed2.txt".to_string()),
        ];
        
        let files_json: Vec<serde_json::Value> = files.iter()
            .map(|(path, name)| {
                serde_json::json!({
                    "path": path.to_string_lossy().to_string(),
                    "name": name
                })
            })
            .collect();
        
        let result = archive_multiple_files(
            files_json,
            output.to_string_lossy().to_string(),
            "tar.gz".to_string(),
        ).await;
        
        assert!(result.is_ok(), "Archive failed: {:?}", result.err());
        
        if let Err(e) = verify_archive_exists(&output) {
            panic!("Archive verification failed: {}", e);
        }
        println!("✅ Archive created: {} ({} bytes)", 
            output.display(), 
            fs::metadata(&output).unwrap().len()
        );
    }

    // ============================================================
    // ТЕСТЫ: Ошибки (должны падать, если ошибка НЕ ожидаема)
    // ============================================================

    #[tokio::test]
    async fn test_archive_file_source_not_exists() {
        let temp_dir = tempdir().unwrap();
        let output = temp_dir.path().join("output.zip");
        
        let result = archive_file(
            "/nonexistent/file.txt".to_string(),
            output.to_string_lossy().to_string(),
            "zip".to_string(),
            "renamed.txt".to_string(),
        ).await;
        
        // 🔥 Ожидаем ошибку — тест должен упасть, если ошибки нет
        assert!(result.is_err(), "Expected error but got success");
        let err = result.err().unwrap();
        println!("✅ Expected error: {}", err);
    }

    #[tokio::test]
    async fn test_archive_multiple_files_missing_field() {
        let temp_dir = tempdir().unwrap();
        let output = temp_dir.path().join("output.zip");
        
        let files_json = vec![
            serde_json::json!({
                "path": "/some/path.txt"
            })
        ];
        
        let result = archive_multiple_files(
            files_json,
            output.to_string_lossy().to_string(),
            "zip".to_string(),
        ).await;
        
        // 🔥 Ожидаем ошибку — тест должен упасть, если ошибки нет
        assert!(result.is_err(), "Expected error but got success");
        let err = result.err().unwrap();
        assert!(err.contains("Missing name") || err.contains("Missing path"));
        println!("✅ Expected error: {}", err);
    }

    #[tokio::test]
    async fn test_archive_multiple_files_empty_list() {
        let temp_dir = tempdir().unwrap();
        let output = temp_dir.path().join("empty.zip");
        
        let result = archive_multiple_files(
            vec![],
            output.to_string_lossy().to_string(),
            "zip".to_string(),
        ).await;
        
        // 🔥 Пустой список — может быть ошибка или успех, проверяем оба варианта
        if let Ok(_) = result {
            if let Err(e) = verify_archive_exists(&output) {
                panic!("Archive verification failed: {}", e);
            }
            println!("✅ Empty archive created: {} ({} bytes)", 
                output.display(), 
                fs::metadata(&output).unwrap().len()
            );
        } else {
            println!("❌ Expected error with empty list: {}", result.err().unwrap());
        }
    }
}