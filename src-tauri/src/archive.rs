// src-tauri/src/archive.rs
use std::path::PathBuf;
use tauri::async_runtime;
use tokio::fs;
use zippylib::{create_tar_gz_archive, create_tar_xz_archive, create_zip_archive};

/// Возвращает безопасную временную директорию внутри Snap.
/// Используется как fallback, если не удалось создать temp рядом с output.
fn get_snap_safe_temp_dir() -> PathBuf {
    println!("🔍 [DEBUG] get_snap_safe_temp_dir() called");
    println!("🔍 [DEBUG] SNAP: {:?}", std::env::var("SNAP"));
    println!("🔍 [DEBUG] SNAP_USER_DATA: {:?}", std::env::var("SNAP_USER_DATA"));
    println!("🔍 [DEBUG] Current dir: {:?}", std::env::current_dir());

    if let Ok(snap_user_data) = std::env::var("SNAP_USER_DATA") {
        let path = PathBuf::from(snap_user_data).join("tmp");
        println!("🔍 [DEBUG] Using SNAP_USER_DATA path: {:?}", path);

        if std::fs::create_dir_all(&path).is_ok() {
            let test_file = path.join(".test_write");
            match std::fs::write(&test_file, b"test") {
                Ok(_) => {
                    let _ = std::fs::remove_file(&test_file);
                    println!("✅ [INFO] Temp directory writable: {:?}", path);
                    return path;
                }
                Err(e) => {
                    eprintln!("❌ [ERROR] Cannot write to temp dir: {}", e);
                }
            }
        } else {
            eprintln!("❌ [ERROR] Failed to create SNAP_USER_DATA/tmp");
        }
    }

    let fallback = std::env::temp_dir();
    println!("🔍 [DEBUG] Using fallback temp dir: {:?}", fallback);
    fallback
}

/// Создаёт временную директорию для архивации.
/// 🔥 ГЛАВНОЕ: пытаемся создать её рядом с итоговым файлом,
/// чтобы `rename` внутри zippylib сработал (одна файловая система).
fn create_temp_dir_for_output(output: &PathBuf, prefix: &str) -> Result<tempfile::TempDir, String> {
    // 1. Пытаемся создать рядом с output
    if let Some(parent) = output.parent() {
        match tempfile::Builder::new()
            .prefix(prefix)
            .tempdir_in(parent)
        {
            Ok(dir) => {
                println!("✅ Temp dir created next to output: {:?}", dir.path());
                return Ok(dir);
            }
            Err(e) => {
                eprintln!("⚠️ Cannot create temp dir next to output ({}). Falling back...", e);
            }
        }
    }

    // 2. Fallback — внутри SNAP_USER_DATA/tmp
    let base_temp = get_snap_safe_temp_dir();
    tempfile::Builder::new()
        .prefix(prefix)
        .tempdir_in(&base_temp)
        .map_err(|e| format!("❌ Cannot create temp dir: {}", e))
}

#[tauri::command]
pub async fn archive_file(
    source_path: String,
    output_path: String,
    format: String,
    name_in_archive: String,
) -> Result<(), String> {
    println!("🚀 [START] archive_file()");
    println!("📁 source_path: {}", source_path);
    println!("📁 output_path: {}", output_path);
    println!("📦 format: {}", format);
    println!("📛 name_in_archive: {}", name_in_archive);

    let source_full = PathBuf::from(&source_path);
    let output = PathBuf::from(&output_path);

    // Проверяем исходный файл
    if !source_full.exists() {
        let err = format!("❌ Source file not found: {:?}", source_full);
        eprintln!("{}", err);
        return Err(err);
    }
    println!("✅ Source file exists");

    if let Ok(metadata) = std::fs::metadata(&source_full) {
        println!("📊 Source size: {} bytes", metadata.len());
    }

    // 🔥 Создаём temp рядом с output (одна ФС → rename сработает)
    let temp_dir = create_temp_dir_for_output(&output, "tauri_archive_")?;
    println!("🔍 [DEBUG] temp_path_ctx: {:?}", temp_dir.path());

    let format_clone = format.clone();
    let output_for_closure = output.clone();

    // ============ ZIP ============
    if format_clone == "zip" {
        println!("📦 Creating ZIP archive...");

        let result = async_runtime::spawn_blocking(move || {
            println!("🔍 [DEBUG] ZIP blocking task started");
            println!("🔍 [DEBUG] Source: {:?}", source_full);
            println!("🔍 [DEBUG] Output: {:?}", output_for_closure);

            match create_zip_archive(&[source_full], output_for_closure.clone()) {
                Ok(_) => {
                    println!("✅ ZIP created successfully");
                    Ok(())
                }
                Err(e) => {
                    let err = format!("❌ Zip error: {}", e);
                    eprintln!("{}", err);
                    Err(err)
                }
            }
        })
        .await
        .map_err(|e| {
            let err = format!("❌ Background task failed: {}", e);
            eprintln!("{}", err);
            err
        })?;

        return result;
    }

    // ============ TAR.GZ / TAR.XZ ============
    println!("📦 Creating {} archive...", format_clone);

    // Копируем файл во временную директорию под нужным именем
    let local_path = temp_dir.path().join(&name_in_archive);
    println!("🔍 [DEBUG] Copying to: {:?}", local_path);

    match fs::copy(&source_full, &local_path).await {
        Ok(bytes) => {
            println!("✅ File copied: {} bytes", bytes);
        }
        Err(e) => {
            eprintln!("❌ Failed to copy file: {}", e);
            println!("🔍 [DEBUG] Trying symlink fallback...");

            #[cfg(target_os = "linux")]
            match std::os::unix::fs::symlink(&source_full, &local_path) {
                Ok(_) => println!("✅ Symlink created"),
                Err(e) => {
                    let err = format!("❌ Symlink failed: {}", e);
                    eprintln!("{}", err);
                    return Err(err);
                }
            }
            #[cfg(not(target_os = "linux"))]
            {
                return Err(format!("❌ Copy failed: {}", e));
            }
        }
    }

    if !local_path.exists() {
        let err = format!("❌ File not found in temp: {:?}", local_path);
        eprintln!("{}", err);
        return Err(err);
    }
    println!("✅ File verified in temp");

    let temp_path_ctx = temp_dir.path().to_path_buf();

    let result = async_runtime::spawn_blocking(move || {
        println!("🔍 [DEBUG] TAR blocking task started");
        println!("🔍 [DEBUG] Current dir: {:?}", std::env::current_dir());

        let relative_name = PathBuf::from(&name_in_archive);

        // Пробуем с относительным путём
        println!("🔍 [DEBUG] Trying relative path...");
        let old_dir = std::env::current_dir().ok();
        let _ = std::env::set_current_dir(&temp_path_ctx);
        println!("🔍 [DEBUG] Changed dir to: {:?}", std::env::current_dir());

        let result = match format_clone.as_str() {
            "tar.gz" => {
                println!("🔍 [DEBUG] Creating tar.gz...");
                let res = create_tar_gz_archive(&[relative_name], output_for_closure.clone())
                    .map_err(|e| format!("Tar.gz error: {}", e));

                if let Err(e) = &res {
                    eprintln!("❌ Tar.gz with relative path failed: {}", e);
                    println!("🔍 [DEBUG] Trying absolute path...");

                    let abs_path = temp_path_ctx.join(&name_in_archive);
                    if abs_path.exists() {
                        println!("🔍 [DEBUG] Using absolute path: {:?}", abs_path);
                        create_tar_gz_archive(&[abs_path], output_for_closure.clone())
                            .map_err(|e| format!("Tar.gz error (absolute): {}", e))
                    } else {
                        res
                    }
                } else {
                    res
                }
            }
            "tar.xz" => {
                println!("🔍 [DEBUG] Creating tar.xz...");
                let res = create_tar_xz_archive(&[relative_name], output_for_closure.clone())
                    .map_err(|e| format!("Tar.xz error: {}", e));

                if let Err(e) = &res {
                    eprintln!("❌ Tar.xz with relative path failed: {}", e);
                    println!("🔍 [DEBUG] Trying absolute path...");

                    let abs_path = temp_path_ctx.join(&name_in_archive);
                    if abs_path.exists() {
                        println!("🔍 [DEBUG] Using absolute path: {:?}", abs_path);
                        create_tar_xz_archive(&[abs_path], output_for_closure.clone())
                            .map_err(|e| format!("Tar.xz error (absolute): {}", e))
                    } else {
                        res
                    }
                } else {
                    res
                }
            }
            _ => {
                let err = format!("❌ Unsupported format: {}", format_clone);
                eprintln!("{}", err);
                Err(err)
            }
        };

        // Возвращаем рабочую директорию
        if let Some(old) = old_dir {
            let _ = std::env::set_current_dir(old);
            println!("🔍 [DEBUG] Restored dir to: {:?}", std::env::current_dir());
        }

        // Проверяем результат
        if result.is_ok() {
            println!("✅ TAR created successfully");
            if let Ok(metadata) = std::fs::metadata(&output_for_closure) {
                println!("📊 Archive size: {} bytes", metadata.len());
                if metadata.len() == 0 {
                    eprintln!("⚠️ WARNING: Archive is empty!");
                }
            }
        } else if let Err(e) = &result {
            eprintln!("❌ TAR creation failed: {}", e);
        }

        result
    })
    .await
    .map_err(|e| {
        let err = format!("❌ Background task failed: {}", e);
        eprintln!("{}", err);
        err
    })?;

    // Финальная проверка
    if output.exists() {
        if let Ok(metadata) = std::fs::metadata(&output) {
            println!("✅ Final archive size: {} bytes", metadata.len());
        }
    } else {
        let err = format!("❌ Archive not found: {:?}", output);
        eprintln!("{}", err);
        return Err(err);
    }

    println!("✅ [END] archive_file() completed successfully");
    result
}

#[tauri::command]
pub async fn archive_multiple_files(
    files: Vec<serde_json::Value>,
    output_path: String,
    format: String,
) -> Result<(), String> {
    println!("🚀 [START] archive_multiple_files()");
    println!("📊 Files count: {}", files.len());
    println!("📁 output_path: {}", output_path);
    println!("📦 format: {}", format);

    let output = PathBuf::from(&output_path);

    // Собираем пути и имена
    let mut files_with_names: Vec<(PathBuf, String)> = Vec::new();
    for (i, item) in files.iter().enumerate() {
        let path = item
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing path".to_string())?;
        let name = item
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing name".to_string())?;
        println!("🔍 [DEBUG] File {}: {:?} -> {}", i, path, name);
        files_with_names.push((PathBuf::from(path), name.to_string()));
    }

    // 🔥 Создаём temp рядом с output
    let temp_dir = create_temp_dir_for_output(&output, "tauri_multiarchive_")?;
    println!("✅ Temp dir created: {:?}", temp_dir.path());

    let format_clone = format.clone();
    let temp_path_ctx = temp_dir.path().to_path_buf();
    let output_for_closure = output.clone();

    // ============ ZIP ============
    if format_clone == "zip" {
        println!("📦 Creating multi ZIP archive...");
        let original_paths: Vec<PathBuf> = files_with_names
            .into_iter()
            .map(|(p, _)| p)
            .collect();

        let result = async_runtime::spawn_blocking(move || {
            println!("🔍 [DEBUG] ZIP blocking task started");
            create_zip_archive(&original_paths, output_for_closure)
                .map_err(|e| format!("Zip error: {}", e))
        })
        .await
        .map_err(|e| {
            let err = format!("❌ Background task failed: {}", e);
            eprintln!("{}", err);
            err
        })?;

        if result.is_ok() {
            println!("✅ Multi ZIP created successfully");
        }
        return result;
    }

    // ============ TAR.GZ / TAR.XZ ============
    println!("📦 Creating multi {} archive...", format_clone);

    let mut relative_names = Vec::new();

    for (source_path, new_name) in files_with_names {
        let local_path = temp_dir.path().join(&new_name);
        println!("🔍 [DEBUG] Copying {:?} -> {:?}", source_path, local_path);

        match fs::copy(&source_path, &local_path).await {
            Ok(bytes) => {
                println!("✅ Copied {} bytes", bytes);
                relative_names.push(PathBuf::from(new_name));
            }
            Err(e) => {
                eprintln!("❌ Failed to copy: {}", e);
                return Err(format!("Failed to copy file: {}", e));
            }
        }
    }

    let output_for_closure = output.clone();

    let result = async_runtime::spawn_blocking(move || {
        println!("🔍 [DEBUG] Multi TAR blocking task started");
        println!("🔍 [DEBUG] Files to archive: {:?}", relative_names);
        println!("🔍 [DEBUG] Current dir: {:?}", std::env::current_dir());

        let old_dir = std::env::current_dir().ok();
        let _ = std::env::set_current_dir(&temp_path_ctx);
        println!("🔍 [DEBUG] Changed dir to: {:?}", std::env::current_dir());

        let result = match format_clone.as_str() {
            "tar.gz" => {
                println!("🔍 [DEBUG] Creating multi tar.gz...");
                create_tar_gz_archive(&relative_names, output_for_closure.clone())
                    .map_err(|e| format!("Tar.gz error: {}", e))
            }
            "tar.xz" => {
                println!("🔍 [DEBUG] Creating multi tar.xz...");
                create_tar_xz_archive(&relative_names, output_for_closure.clone())
                    .map_err(|e| format!("Tar.xz error: {}", e))
            }
            _ => {
                let err = format!("❌ Unsupported format: {}", format_clone);
                eprintln!("{}", err);
                Err(err)
            }
        };

        if let Some(old) = old_dir {
            let _ = std::env::set_current_dir(old);
            println!("🔍 [DEBUG] Restored dir to: {:?}", std::env::current_dir());
        }

        if result.is_ok() {
            println!("✅ Multi TAR created successfully");
            if let Ok(metadata) = std::fs::metadata(output_for_closure.clone()) {
                println!("📊 Archive size: {} bytes", metadata.len());
            }
        } else if let Err(e) = &result {
            eprintln!("❌ Multi TAR creation failed: {}", e);
        }

        result
    })
    .await
    .map_err(|e| {
        let err = format!("❌ Background task failed: {}", e);
        eprintln!("{}", err);
        err
    })?;

    if output.exists() {
        if let Ok(metadata) = std::fs::metadata(&output) {
            println!("✅ Final archive size: {} bytes", metadata.len());
        }
    }

    println!("✅ [END] archive_multiple_files() completed");
    result
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
