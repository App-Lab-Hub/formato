// src-tauri/src/archive.rs
use std::fs::File;
use std::path::PathBuf;
use tauri::async_runtime;

use flate2::write::GzEncoder;
use flate2::Compression;
use tar::{Builder as TarBuilder, Header};
use xz2::write::XzEncoder;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

// ============================================================
// ПРЯМЫЕ ФУНКЦИИ СОЗДАНИЯ АРХИВОВ (без zippylib и rename)
// ============================================================

/// Создаёт ZIP напрямую, без временных файлов и `rename`.
fn create_zip_direct(files: &[(PathBuf, String)], output: &PathBuf) -> Result<(), String> {
    println!("📦 Creating ZIP directly...");

    let file = File::create(output).map_err(|e| format!("Failed to create zip: {}", e))?;
    let mut zip = ZipWriter::new(file);

    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o755);

    for (source_path, name_in_archive) in files {
        println!("  Adding: {:?} -> {}", source_path, name_in_archive);

        if !source_path.exists() {
            return Err(format!("Source file not found: {:?}", source_path));
        }

        let mut f = File::open(source_path).map_err(|e| format!("Failed to open: {}", e))?;
        zip.start_file(name_in_archive, options)
            .map_err(|e| format!("Failed to start file in zip: {}", e))?;
        std::io::copy(&mut f, &mut zip)
            .map_err(|e| format!("Failed to copy into zip: {}", e))?;
    }

    zip.finish().map_err(|e| format!("Failed to finish zip: {}", e))?;
    println!("✅ ZIP created: {:?}", output);
    Ok(())
}

/// Создаёт TAR.GZ напрямую.
fn create_tar_gz_direct(files: &[(PathBuf, String)], output: &PathBuf) -> Result<(), String> {
    println!("📦 Creating TAR.GZ directly...");

    let file = File::create(output).map_err(|e| format!("Failed to create tar.gz: {}", e))?;
    let enc = GzEncoder::new(file, Compression::default());
    let mut tar = TarBuilder::new(enc);

    for (source_path, name_in_archive) in files {
        println!("  Adding: {:?} -> {}", source_path, name_in_archive);

        if !source_path.exists() {
            return Err(format!("Source file not found: {:?}", source_path));
        }

        let mut f = File::open(source_path).map_err(|e| format!("Failed to open: {}", e))?;
        let metadata = f.metadata().map_err(|e| format!("Failed to get metadata: {}", e))?;

        let mut header = Header::new_gnu();
        header.set_size(metadata.len());
        header.set_mode(0o644);
        header.set_cksum();

        tar.append_data(&mut header, name_in_archive, &mut f)
            .map_err(|e| format!("Failed to append to tar.gz: {}", e))?;
    }

    tar.finish().map_err(|e| format!("Failed to finish tar.gz: {}", e))?;
    println!("✅ TAR.GZ created: {:?}", output);
    Ok(())
}

/// Создаёт TAR.XZ напрямую.
fn create_tar_xz_direct(files: &[(PathBuf, String)], output: &PathBuf) -> Result<(), String> {
    println!("📦 Creating TAR.XZ directly...");

    let file = File::create(output).map_err(|e| format!("Failed to create tar.xz: {}", e))?;
    let enc = XzEncoder::new(file, 6);
    let mut tar = TarBuilder::new(enc);

    for (source_path, name_in_archive) in files {
        println!("  Adding: {:?} -> {}", source_path, name_in_archive);

        if !source_path.exists() {
            return Err(format!("Source file not found: {:?}", source_path));
        }

        let mut f = File::open(source_path).map_err(|e| format!("Failed to open: {}", e))?;
        let metadata = f.metadata().map_err(|e| format!("Failed to get metadata: {}", e))?;

        let mut header = Header::new_gnu();
        header.set_size(metadata.len());
        header.set_mode(0o644);
        header.set_cksum();

        tar.append_data(&mut header, name_in_archive, &mut f)
            .map_err(|e| format!("Failed to append to tar.xz: {}", e))?;
    }

    tar.finish().map_err(|e| format!("Failed to finish tar.xz: {}", e))?;
    println!("✅ TAR.XZ created: {:?}", output);
    Ok(())
}

/// Создаёт безопасную временную директорию внутри Snap.
/// Оставлено на случай, если понадобится для других задач.
#[allow(dead_code)]
fn get_snap_safe_temp_dir() -> PathBuf {
    if let Ok(snap_user_data) = std::env::var("SNAP_USER_DATA") {
        let path = PathBuf::from(snap_user_data).join("tmp");
        if std::fs::create_dir_all(&path).is_ok() {
            return path;
        }
    }
    std::env::temp_dir()
}

// ============================================================
// TAURI COMMANDS
// ============================================================

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

    if !source_full.exists() {
        return Err(format!("❌ Source file not found: {:?}", source_full));
    }

    if let Ok(metadata) = std::fs::metadata(&source_full) {
        println!("📊 Source size: {} bytes", metadata.len());
    }

    let format_clone = format.clone();
    let source_for_closure = source_full.clone();
    let output_for_closure = output.clone();
    let name_for_closure = name_in_archive.clone();

    // 🔥 Работаем напрямую, без zippylib и временных файлов
    async_runtime::spawn_blocking(move || {
        let files = vec![(source_for_closure, name_for_closure)];

        match format_clone.as_str() {
            "zip" => create_zip_direct(&files, &output_for_closure),
            "tar.gz" => create_tar_gz_direct(&files, &output_for_closure),
            "tar.xz" => create_tar_xz_direct(&files, &output_for_closure),
            _ => Err(format!("❌ Unsupported format: {}", format_clone)),
        }
    })
    .await
    .map_err(|e| format!("❌ Background task failed: {}", e))??;

    if output.exists() {
        if let Ok(metadata) = std::fs::metadata(&output) {
            println!("✅ Final archive size: {} bytes", metadata.len());
        }
    } else {
        return Err(format!("❌ Archive not found: {:?}", output));
    }

    println!("✅ [END] archive_file() completed");
    Ok(())
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

    let format_clone = format.clone();
    let output_for_closure = output.clone();

    // 🔥 Работаем напрямую
    async_runtime::spawn_blocking(move || {
        match format_clone.as_str() {
            "zip" => create_zip_direct(&files_with_names, &output_for_closure),
            "tar.gz" => create_tar_gz_direct(&files_with_names, &output_for_closure),
            "tar.xz" => create_tar_xz_direct(&files_with_names, &output_for_closure),
            _ => Err(format!("❌ Unsupported format: {}", format_clone)),
        }
    })
    .await
    .map_err(|e| format!("❌ Background task failed: {}", e))??;

    if output.exists() {
        if let Ok(metadata) = std::fs::metadata(&output) {
            println!("✅ Final archive size: {} bytes", metadata.len());
        }
    }

    println!("✅ [END] archive_multiple_files() completed");
    Ok(())
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
