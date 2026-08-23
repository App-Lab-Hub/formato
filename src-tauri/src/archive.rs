// src-tauri/src/archive.rs
use std::path::PathBuf;
use tauri::async_runtime;
use tokio::fs;
use zippylib::{create_tar_gz_archive, create_tar_xz_archive, create_zip_archive};

#[tauri::command]
pub async fn archive_file(
    source_path: String,
    output_path: String,
    format: String,
    name_in_archive: String,
) -> Result<(), String> {
    let source_full = PathBuf::from(&source_path);
    let output = PathBuf::from(&output_path);

    // Создаем временную директорию, которая автоматически удалится в конце функции
    let temp_dir = tempfile::Builder::new()
        .prefix("tauri_archive_")
        .tempdir()
        .map_err(|e| format!("Failed to create temp dir: {}", e))?;

    let local_path = temp_dir.path().join(&name_in_archive);

    // Копируем исходный файл во временную директорию с новым именем
    fs::copy(&source_full, &local_path)
        .await
        .map_err(|e| format!("Failed to copy file: {}", e))?;

    let format_clone = format.clone();

    async_runtime::spawn_blocking(move || {
        let result = match format_clone.as_str() {
            "zip" => {
                // Для ZIP передаем полный путь к временному файлу
                create_zip_archive(&[local_path], output).map_err(|e| format!("Zip error: {}", e))
            }
            "tar.gz" | "tar.xz" => {
                // Для TAR используем только относительное имя файла внутри архива.
                // Чтобы библиотека zippylib нашла файл, временно меняем текущую директорию процесса.
                let _dir_guard = std::env::current_dir().and_then(|old_dir| {
                    std::env::set_current_dir(temp_dir.path())?;
                    Ok(old_dir)
                });

                let relative_name = PathBuf::from(name_in_archive);
                let files = vec![relative_name];

                let res = match format_clone.as_str() {
                    "tar.gz" => create_tar_gz_archive(&files, output)
                        .map_err(|e| format!("Tar.gz error: {}", e)),
                    "tar.xz" => create_tar_xz_archive(&files, output)
                        .map_err(|e| format!("Tar.xz error: {}", e)),
                    _ => unreachable!(),
                };

                // Возвращаем рабочую директорию назад, если guard успешно создался
                if let Ok(old_dir) = _dir_guard {
                    let _ = std::env::set_current_dir(old_dir);
                }
                res
            }
            _ => Err(format!("Unsupported format: {}", format_clone)),
        };

        drop(temp_dir);
        result
    })
    .await
    .map_err(|e| format!("Background task failed: {}", e))?
}

#[tauri::command]
pub async fn archive_multiple_files(
    files: Vec<serde_json::Value>,
    output_path: String,
    format: String,
) -> Result<(), String> {
    let output = PathBuf::from(&output_path);

    let mut files_with_names: Vec<(PathBuf, String)> = Vec::new();
    for item in files {
        let path = item
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing path".to_string())?;
        let name = item
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing name".to_string())?;
        files_with_names.push((PathBuf::from(path), name.to_string()));
    }

    // Создаем временную директорию для всех файлов
    let temp_dir = tempfile::Builder::new()
        .prefix("tauri_multiarchive_")
        .tempdir()
        .map_err(|e| format!("Failed to create temp dir: {}", e))?;

    let mut temp_paths = Vec::new();
    let mut relative_names = Vec::new();

    for (source_path, new_name) in files_with_names {
        let local_path = temp_dir.path().join(&new_name);
        fs::copy(source_path, &local_path)
            .await
            .map_err(|e| format!("Failed to copy file: {}", e))?;

        temp_paths.push(local_path);
        relative_names.push(PathBuf::from(new_name));
    }

    let format_clone = format.clone();

    async_runtime::spawn_blocking(move || {
        let result = match format_clone.as_str() {
            "zip" => {
                create_zip_archive(&temp_paths, output).map_err(|e| format!("Zip error: {}", e))
            }
            "tar.gz" | "tar.xz" => {
                let _dir_guard = std::env::current_dir().and_then(|old_dir| {
                    std::env::set_current_dir(temp_dir.path())?;
                    Ok(old_dir)
                });

                let res = match format_clone.as_str() {
                    "tar.gz" => create_tar_gz_archive(&relative_names, output)
                        .map_err(|e| format!("Tar.gz error: {}", e)),
                    "tar.xz" => create_tar_xz_archive(&relative_names, output)
                        .map_err(|e| format!("Tar.xz error: {}", e)),
                    _ => unreachable!(),
                };

                if let Ok(old_dir) = _dir_guard {
                    let _ = std::env::set_current_dir(old_dir);
                }
                res
            }
            _ => Err(format!("Unsupported format: {}", format_clone)),
        };

        drop(temp_dir);
        result
    })
    .await
    .map_err(|e| format!("Background task failed: {}", e))?
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::OnceLock;
    use tempfile::tempdir;

    static TEST_MUTEX: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();

    async fn get_test_mutex() -> &'static tokio::sync::Mutex<()> {
        TEST_MUTEX.get_or_init(|| tokio::sync::Mutex::new(()))
    }

    // ============================================================
    // ВСПОМОГАТЕЛЬНЫЕ ФУНКЦИИ
    // ============================================================

    fn get_fixture_files(ext: &str) -> Vec<PathBuf> {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let fixtures_dir = manifest_dir.join("../fixtures");
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

    fn verify_archive_exists(archive_path: &Path) -> Result<(), String> {
        if !archive_path.exists() {
            return Err(format!("Archive not found: {:?}", archive_path));
        }

        let metadata =
            fs::metadata(archive_path).map_err(|e| format!("Cannot get metadata: {}", e))?;

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
        let _lock = get_test_mutex().await.lock().await;

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
        )
        .await;

        assert!(result.is_ok(), "Archive failed: {:?}", result.err());

        if let Err(e) = verify_archive_exists(&output) {
            panic!("Archive verification failed: {}", e);
        }
        println!(
            "✅ Archive created: {} ({} bytes)",
            output.display(),
            fs::metadata(&output).unwrap().len()
        );
    }

    #[tokio::test]
    async fn test_archive_file_tar_gz_from_fixtures() {
        let _lock = get_test_mutex().await.lock().await;

        if !has_fixtures("docx") {
            println!("⚠️ Skipping test: no DOCX fixtures found");
            return;
        }

        let files = get_fixture_files("docx");
        let source = &files[0];
        let temp_dir = tempdir().unwrap();
        let output = temp_dir.path().join("archive.tar.gz");

        let file_name = source
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("document.docx")
            .to_string();

        let result = archive_file(
            source.to_string_lossy().to_string(),
            output.to_string_lossy().to_string(),
            "tar.gz".to_string(),
            file_name,
        )
        .await;

        assert!(result.is_ok(), "Archive failed: {:?}", result.err());

        if let Err(e) = verify_archive_exists(&output) {
            panic!("Archive verification failed: {}", e);
        }
        println!(
            "✅ Archive created: {} ({} bytes)",
            output.display(),
            fs::metadata(&output).unwrap().len()
        );
    }

    #[tokio::test]
    async fn test_archive_file_tar_xz_from_fixtures() {
        let _lock = get_test_mutex().await.lock().await;

        if !has_fixtures("pdf") {
            println!("⚠️ Skipping test: no PDF fixtures found");
            return;
        }

        let files = get_fixture_files("pdf");
        let source = &files[0];
        let temp_dir = tempdir().unwrap();
        let output = temp_dir.path().join("archive.tar.xz");

        let file_name = source
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("document.pdf")
            .to_string();

        let result = archive_file(
            source.to_string_lossy().to_string(),
            output.to_string_lossy().to_string(),
            "tar.xz".to_string(),
            file_name,
        )
        .await;

        assert!(result.is_ok(), "Archive failed: {:?}", result.err());

        if let Err(e) = verify_archive_exists(&output) {
            panic!("Archive verification failed: {}", e);
        }
        println!(
            "✅ Archive created: {} ({} bytes)",
            output.display(),
            fs::metadata(&output).unwrap().len()
        );
    }

    // ============================================================
    // ТЕСТЫ: archive_multiple_files
    // ============================================================

    #[tokio::test]
    async fn test_archive_multiple_files_zip_from_fixtures() {
        let _lock = get_test_mutex().await.lock().await;

        if !has_fixtures("json") || !has_fixtures("csv") {
            println!("⚠️ Skipping test: need JSON and CSV fixtures");
            return;
        }

        let json_files = get_fixture_files("json");
        let csv_files = get_fixture_files("csv");

        let temp_dir = tempdir().unwrap();
        let output = temp_dir.path().join("multiple.zip");

        let files_data = [
            (json_files[0].clone(), "test.json".to_string()),
            (csv_files[0].clone(), "config.csv".to_string()),
        ];

        let files_json: Vec<serde_json::Value> = files_data
            .iter()
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
        )
        .await;

        assert!(result.is_ok(), "Archive failed: {:?}", result.err());

        if let Err(e) = verify_archive_exists(&output) {
            panic!("Archive verification failed: {}", e);
        }
        println!(
            "✅ Archive created: {} ({} bytes)",
            output.display(),
            fs::metadata(&output).unwrap().len()
        );
    }

    #[tokio::test]
    async fn test_archive_multiple_files_tar_gz_from_fixtures() {
        let _lock = get_test_mutex().await.lock().await;

        if !has_fixtures("ini") || get_fixture_files("ini").len() < 2 {
            println!("⚠️ Skipping test: need at least 2 INI fixtures");
            return;
        }

        let ini_files = get_fixture_files("ini");
        let temp_dir = tempdir().unwrap();
        let output = temp_dir.path().join("multiple.tar.gz");

        let files_data = [
            (
                ini_files[0].clone(),
                ini_files[0]
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("config1.ini")
                    .to_string(),
            ),
            (
                ini_files[1].clone(),
                ini_files[1]
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("config2.ini")
                    .to_string(),
            ),
        ];

        let files_json: Vec<serde_json::Value> = files_data
            .iter()
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
        )
        .await;

        assert!(result.is_ok(), "Archive failed: {:?}", result.err());

        if let Err(e) = verify_archive_exists(&output) {
            panic!("Archive verification failed: {}", e);
        }
        println!(
            "✅ Archive created: {} ({} bytes)",
            output.display(),
            fs::metadata(&output).unwrap().len()
        );
    }

    #[tokio::test]
    async fn test_archive_multiple_files_mixed_from_fixtures() {
        let _lock = get_test_mutex().await.lock().await;

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

        let files_data: Vec<(PathBuf, String)> = files
            .iter()
            .enumerate()
            .map(|(i, path)| {
                let name = format!(
                    "file_{}.{}",
                    i,
                    path.extension().unwrap_or_default().to_string_lossy()
                );
                (path.clone(), name)
            })
            .collect();

        let files_json: Vec<serde_json::Value> = files_data
            .iter()
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
        )
        .await;

        assert!(result.is_ok(), "Archive failed: {:?}", result.err());

        if let Err(e) = verify_archive_exists(&output) {
            panic!("Archive verification failed: {}", e);
        }
        println!(
            "✅ Archive created: {} ({} bytes)",
            output.display(),
            fs::metadata(&output).unwrap().len()
        );
    }

    // ============================================================
    // ТЕСТЫ: Базовые
    // ============================================================

    #[tokio::test]
    async fn test_archive_file_zip() {
        let _lock = get_test_mutex().await.lock().await;

        let temp_dir = tempdir().unwrap();
        let source = temp_dir.path().join("source.txt");
        fs::write(&source, "Test content for ZIP").unwrap();
        let output = temp_dir.path().join("output.zip");

        let result = archive_file(
            source.to_string_lossy().to_string(),
            output.to_string_lossy().to_string(),
            "zip".to_string(),
            "renamed.txt".to_string(),
        )
        .await;

        assert!(result.is_ok(), "Archive failed: {:?}", result.err());

        if let Err(e) = verify_archive_exists(&output) {
            panic!("Archive verification failed: {}", e);
        }
        println!(
            "✅ Archive created: {} ({} bytes)",
            output.display(),
            fs::metadata(&output).unwrap().len()
        );
    }

    #[tokio::test]
    async fn test_archive_file_tar_gz() {
        let _lock = get_test_mutex().await.lock().await;

        let temp_dir = tempdir().unwrap();
        let source = temp_dir.path().join("source.txt");
        fs::write(&source, "Test content for TAR.GZ").unwrap();
        let output = temp_dir.path().join("output.tar.gz");

        let result = archive_file(
            source.to_string_lossy().to_string(),
            output.to_string_lossy().to_string(),
            "tar.gz".to_string(),
            "renamed.txt".to_string(),
        )
        .await;

        assert!(result.is_ok(), "Archive failed: {:?}", result.err());

        if let Err(e) = verify_archive_exists(&output) {
            panic!("Archive verification failed: {}", e);
        }
        println!(
            "✅ Archive created: {} ({} bytes)",
            output.display(),
            fs::metadata(&output).unwrap().len()
        );
    }

    #[tokio::test]
    async fn test_archive_file_tar_xz() {
        let _lock = get_test_mutex().await.lock().await;

        let temp_dir = tempdir().unwrap();
        let source = temp_dir.path().join("source.txt");
        fs::write(&source, "Test content for TAR.XZ").unwrap();
        let output = temp_dir.path().join("output.tar.xz");

        let result = archive_file(
            source.to_string_lossy().to_string(),
            output.to_string_lossy().to_string(),
            "tar.xz".to_string(),
            "renamed.txt".to_string(),
        )
        .await;

        assert!(result.is_ok(), "Archive failed: {:?}", result.err());

        if let Err(e) = verify_archive_exists(&output) {
            panic!("Archive verification failed: {}", e);
        }
        println!(
            "✅ Archive created: {} ({} bytes)",
            output.display(),
            fs::metadata(&output).unwrap().len()
        );
    }

    #[tokio::test]
    async fn test_archive_multiple_files_zip() {
        let _lock = get_test_mutex().await.lock().await;

        let temp_dir = tempdir().unwrap();
        let file1 = temp_dir.path().join("file1.txt");
        let file2 = temp_dir.path().join("file2.txt");
        fs::write(&file1, "Hello from file1").unwrap();
        fs::write(&file2, "Hello from file2").unwrap();

        let output = temp_dir.path().join("multiple.zip");

        let files = [
            (file1, "renamed1.txt".to_string()),
            (file2, "renamed2.txt".to_string()),
        ];

        let files_json: Vec<serde_json::Value> = files
            .iter()
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
        )
        .await;

        assert!(result.is_ok(), "Archive failed: {:?}", result.err());

        if let Err(e) = verify_archive_exists(&output) {
            panic!("Archive verification failed: {}", e);
        }
        println!(
            "✅ Archive created: {} ({} bytes)",
            output.display(),
            fs::metadata(&output).unwrap().len()
        );
    }

    #[tokio::test]
    async fn test_archive_multiple_files_tar_gz() {
        let _lock = get_test_mutex().await.lock().await;

        let temp_dir = tempdir().unwrap();
        let file1 = temp_dir.path().join("file1.txt");
        let file2 = temp_dir.path().join("file2.txt");
        fs::write(&file1, "Hello from file1").unwrap();
        fs::write(&file2, "Hello from file2").unwrap();

        let output = temp_dir.path().join("multiple.tar.gz");

        let files = [
            (file1, "renamed1.txt".to_string()),
            (file2, "renamed2.txt".to_string()),
        ];

        let files_json: Vec<serde_json::Value> = files
            .iter()
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
        )
        .await;

        assert!(result.is_ok(), "Archive failed: {:?}", result.err());

        if let Err(e) = verify_archive_exists(&output) {
            panic!("Archive verification failed: {}", e);
        }
        println!(
            "✅ Archive created: {} ({} bytes)",
            output.display(),
            fs::metadata(&output).unwrap().len()
        );
    }

    // ============================================================
    // ТЕСТЫ: Ошибки
    // ============================================================

    #[tokio::test]
    async fn test_archive_file_source_not_exists() {
        let _lock = get_test_mutex().await.lock().await;

        let temp_dir = tempdir().unwrap();
        let output = temp_dir.path().join("output.zip");

        let result = archive_file(
            "/nonexistent/file.txt".to_string(),
            output.to_string_lossy().to_string(),
            "zip".to_string(),
            "renamed.txt".to_string(),
        )
        .await;

        assert!(result.is_err(), "Expected error but got success");
        let err = result.err().unwrap();
        println!("✅ Expected error: {}", err);
    }

    #[tokio::test]
    async fn test_archive_multiple_files_missing_field() {
        let _lock = get_test_mutex().await.lock().await;

        let temp_dir = tempdir().unwrap();
        let output = temp_dir.path().join("output.zip");

        let files_json = vec![serde_json::json!({
            "path": "/some/path.txt"
        })];

        let result = archive_multiple_files(
            files_json,
            output.to_string_lossy().to_string(),
            "zip".to_string(),
        )
        .await;

        assert!(result.is_err(), "Expected error but got success");
        let err = result.err().unwrap();
        assert!(err.contains("Missing name") || err.contains("Missing path"));
        println!("✅ Expected error: {}", err);
    }

    #[tokio::test]
    async fn test_archive_multiple_files_empty_list() {
        let _lock = get_test_mutex().await.lock().await;

        let temp_dir = tempdir().unwrap();
        let output = temp_dir.path().join("empty.zip");

        let result = archive_multiple_files(
            vec![],
            output.to_string_lossy().to_string(),
            "zip".to_string(),
        )
        .await;

        if result.is_ok() {
            if let Err(e) = verify_archive_exists(&output) {
                panic!("Archive verification failed: {}", e);
            }
            println!(
                "✅ Empty archive created: {} ({} bytes)",
                output.display(),
                fs::metadata(&output).unwrap().len()
            );
        } else {
            println!(
                "❌ Expected error with empty list: {}",
                result.err().unwrap()
            );
        }
    }
}
