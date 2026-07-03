// src-tauri/src/archive.rs
use libarchive2::{WriteArchive, ArchiveFormat, CompressionFormat};
use std::path::PathBuf;
use std::fs::File;
use std::io::Read;
use tauri::async_runtime;

#[tauri::command]
pub async fn archive_file(source_path: String, output_path: String, format: String) -> Result<(), String> {
    let result = async_runtime::spawn_blocking(move || {
        // Определяем формат
        let (archive_format, compression) = match format.as_str() {
            "zip" => (ArchiveFormat::Zip, CompressionFormat::None),
            "tar.gz" => (ArchiveFormat::Tar, CompressionFormat::Gzip),
            "tar.xz" => (ArchiveFormat::Tar, CompressionFormat::Xz),
            _ => return Err(format!("Unsupported format: {}", format)),
        };

        // Создаем архив
        let mut archive = WriteArchive::new()
            .format(archive_format)
            .compression(compression)
            .open_file(&output_path)
            .map_err(|e| format!("Failed to create archive: {}", e))?;

        // Читаем файл
        let mut file = File::open(&source_path)
            .map_err(|e| format!("Failed to open source file: {}", e))?;
        
        let mut content = Vec::new();
        file.read_to_end(&mut content)
            .map_err(|e| format!("Failed to read source file: {}", e))?;

        // Получаем имя файла - сохраняем PathBuf в переменную
        let path = PathBuf::from(&source_path);
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| "Invalid file name".to_string())?;

        // Добавляем файл в архив
        archive.add_file(file_name, &content)
            .map_err(|e| format!("Failed to add file to archive: {}", e))?;

        Ok(())
    })
    .await
    .map_err(|e| format!("Background task failed: {}", e))?;

    result
}