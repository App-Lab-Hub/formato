// src-tauri/src/utils.rs
use crate::db::delete_conversion_by_path;
use crate::paths::{converted_dir, temp_dir};
use crate::AppState;
use serde::{Deserialize, Serialize};
use tokio::fs as tokio_fs;
use tokio::task;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileInfo {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub created: String,
    pub file_type: String, // "converted" или "temp"
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FilesResponse {
    pub files: Vec<FileInfo>,
    pub total_files: usize,
    pub total_size: u64,
    pub converted_count: usize,
    pub temp_count: usize,
}

/// Асинхронное получение всех файлов со статистикой
#[tauri::command]
pub async fn get_files() -> Result<FilesResponse, String> {
    let result = task::spawn(async {
        let mut files = Vec::new();
        let mut total_size = 0u64;
        let mut converted_count = 0;
        let mut temp_count = 0;

        // Получаем файлы из converted_dir
        let converted = converted_dir();
        if converted.exists() {
            let mut entries = tokio_fs::read_dir(&converted)
                .await
                .map_err(|e| format!("Failed to read converted dir: {}", e))?;

            while let Some(entry) = entries
                .next_entry()
                .await
                .map_err(|e| format!("Failed to read entry: {}", e))?
            {
                let path = entry.path();
                if path.is_file() {
                    let metadata = tokio_fs::metadata(&path)
                        .await
                        .map_err(|e| format!("Failed to get metadata: {}", e))?;
                    let name = path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();

                    let size = metadata.len();
                    total_size += size;
                    converted_count += 1;

                    files.push(FileInfo {
                        name,
                        path: path.to_string_lossy().to_string(),
                        size,
                        created: metadata
                            .created()
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
            let mut entries = tokio_fs::read_dir(&temp)
                .await
                .map_err(|e| format!("Failed to read temp dir: {}", e))?;

            while let Some(entry) = entries
                .next_entry()
                .await
                .map_err(|e| format!("Failed to read entry: {}", e))?
            {
                let path = entry.path();
                if path.is_file() {
                    let metadata = tokio_fs::metadata(&path)
                        .await
                        .map_err(|e| format!("Failed to get metadata: {}", e))?;
                    let name = path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();

                    let size = metadata.len();
                    total_size += size;
                    temp_count += 1;

                    files.push(FileInfo {
                        name,
                        path: path.to_string_lossy().to_string(),
                        size,
                        created: metadata
                            .created()
                            .map(|t| format!("{:?}", t))
                            .unwrap_or_else(|_| "Unknown".to_string()),
                        file_type: "temp".to_string(),
                    });
                }
            }
        }

        // Сортируем по дате создания (новые сверху)
        files.sort_by(|a, b| b.created.cmp(&a.created));
        let total_files = files.len();
        Ok::<_, String>(FilesResponse {
            files,
            total_files,
            total_size,
            converted_count,
            temp_count,
        })
    })
    .await
    .map_err(|e| format!("Failed to read files: {}", e))??;

    Ok(result)
}

#[tauri::command]
pub async fn delete_file(
    state: tauri::State<'_, AppState>,
    path: String,
) -> Result<String, String> {
    let db_guard = state.db.lock().await;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;

    delete_conversion_by_path(db, &path).await?;

    if std::path::Path::new(&path).exists() {
        tokio_fs::remove_file(&path)
            .await
            .map_err(|e| format!("Cannot delete file: {e}"))?;
        println!("✅ Deleted file: {}", path);
    } else {
        println!("⚠️ File not found: {}", path);
    }

    Ok(path)
}
