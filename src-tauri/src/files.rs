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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::db_init;
    use crate::paths::{converted_dir, temp_dir};
    use std::fs;
    use std::path::PathBuf;
    use std::sync::Arc;
    use tauri::test::{mock_builder, noop_assets, MockRuntime};
    use tauri::Manager;
    use tauri::State;
    use tempfile::TempDir;
    use tokio::sync::Mutex;

    // ============================================================
    // ВСПОМОГАТЕЛЬНЫЕ ФУНКЦИИ
    // ============================================================

    /// Создает тестовое приложение с РЕАЛЬНЫМИ директориями
    async fn create_real_app() -> tauri::App<MockRuntime> {
        let db = db_init().await.unwrap();

        std::env::remove_var("FORMATO_CONVERTED_DIR");
        std::env::remove_var("FORMATO_TEMP_DIR");

        mock_builder()
            .manage(AppState {
                db: Arc::new(Mutex::new(Some(db))),
                system_theme: Mutex::new("dark".to_string()),
            })
            .build(tauri::test::mock_context(noop_assets()))
            .unwrap()
    }

    /// Создает тестовое приложение с ВРЕМЕННЫМИ директориями
    async fn create_empty_app() -> (tauri::App<MockRuntime>, TempDir, TempDir) {
        let db = db_init().await.unwrap();

        let temp_converted = TempDir::new().unwrap();
        let temp_temp = TempDir::new().unwrap();

        std::env::set_var("FORMATO_CONVERTED_DIR", temp_converted.path());
        std::env::set_var("FORMATO_TEMP_DIR", temp_temp.path());

        let app = mock_builder()
            .manage(AppState {
                db: Arc::new(Mutex::new(Some(db))),
                system_theme: Mutex::new("dark".to_string()),
            })
            .build(tauri::test::mock_context(noop_assets()))
            .unwrap();

        (app, temp_converted, temp_temp)
    }

    /// Получает State из приложения
    fn get_state(app: &tauri::App<MockRuntime>) -> State<'_, AppState> {
        app.state::<AppState>()
    }

    /// Проверяет, есть ли файлы в директории
    fn has_files(dir: &PathBuf) -> bool {
        if !dir.exists() {
            return false;
        }
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                if entry.path().is_file() {
                    return true;
                }
            }
        }
        false
    }

    // ============================================================
    // ТЕСТ 1: Наличие файлов в реальных директориях (всегда запускается)
    // ============================================================

    #[tokio::test]
    async fn test_files_exist() {
        let result = get_files().await.unwrap();

        assert!(result.total_files > 0, "Должны быть файлы в директориях");
        assert!(result.total_size > 0);
        assert!(result.converted_count > 0 || result.temp_count > 0);
        assert!(!result.files.is_empty());

        for file in &result.files {
            assert!(!file.name.is_empty());
            assert!(!file.path.is_empty());
            assert!(file.size > 0);
            assert!(file.file_type == "converted" || file.file_type == "temp");
            assert!(!file.created.is_empty());
        }

        println!("📁 Найдено файлов: {}", result.total_files);
        println!(
            "📊 Converted: {}, Temp: {}",
            result.converted_count, result.temp_count
        );
        println!("💾 Общий размер: {} байт", result.total_size);
    }

    // ============================================================
    // ТЕСТ 2: Удаление файлов из реальных директорий (ignored по умолчанию)
    // ============================================================

    #[tokio::test]
    #[ignore = "Удаляет реальные файлы! Запускайте только с --ignored"]
    async fn test_delete_files() {
        let app = create_real_app().await;
        let state = get_state(&app);

        let result = get_files().await.unwrap();
        let file_count = result.total_files;

        if file_count == 0 {
            println!("⚠️ Нет файлов для удаления, тест пропущен");
            return;
        }

        println!("🗑️ Начинаем удаление {} файлов...", file_count);

        // Удаляем все файлы из converted_dir
        let converted = converted_dir();
        if converted.exists() {
            if let Ok(entries) = fs::read_dir(&converted) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        let path_str = path.to_string_lossy().to_string();
                        let result = delete_file(state.clone(), path_str).await;
                        assert!(result.is_ok(), "Ошибка удаления файла: {:?}", result);
                        println!("✅ Удален: {}", path.display());
                    }
                }
            }
        }

        // Удаляем все файлы из temp_dir
        let temp = temp_dir();
        if temp.exists() {
            if let Ok(entries) = fs::read_dir(&temp) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        let path_str = path.to_string_lossy().to_string();
                        let result = delete_file(state.clone(), path_str).await;
                        assert!(result.is_ok(), "Ошибка удаления файла: {:?}", result);
                        println!("✅ Удален: {}", path.display());
                    }
                }
            }
        }

        let after = get_files().await.unwrap();
        assert_eq!(after.total_files, 0, "Все файлы должны быть удалены");
        assert_eq!(after.converted_count, 0);
        assert_eq!(after.temp_count, 0);
        assert!(after.files.is_empty());

        println!("✅ Все файлы удалены!");
    }

    // ============================================================
    // ТЕСТ 3: Отсутствие файлов (пустые директории) (ignored по умолчанию)
    // ============================================================

    #[tokio::test]
    #[ignore = "Проверяет пустые директории (использует временные)"]
    async fn test_no_files() {
        let (_app, conv_dir, temp_dir) = create_empty_app().await;
        let conv_path = conv_dir.path().to_path_buf();
        let temp_path = temp_dir.path().to_path_buf();

        assert!(
            !has_files(&conv_path),
            "Converted директория должна быть пустой"
        );
        assert!(!has_files(&temp_path), "Temp директория должна быть пустой");

        let result = get_files().await.unwrap();

        assert_eq!(result.total_files, 0);
        assert_eq!(result.total_size, 0);
        assert_eq!(result.converted_count, 0);
        assert_eq!(result.temp_count, 0);
        assert!(result.files.is_empty());

        println!("✅ Директории пустые, файлов нет");
    }

    // ============================================================
    // ДОПОЛНИТЕЛЬНЫЕ ТЕСТЫ
    // ============================================================

    #[tokio::test]
    async fn test_delete_nonexistent_file() {
        let app = create_real_app().await;
        let state = get_state(&app);

        let result = delete_file(state, "/nonexistent/file.txt".to_string()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_file_no_db() {
        let app = mock_builder()
            .manage(AppState {
                db: Arc::new(Mutex::new(None)),
                system_theme: Mutex::new("dark".to_string()),
            })
            .build(tauri::test::mock_context(noop_assets()))
            .unwrap();
        let state = app.state::<AppState>();

        let result = delete_file(state, "test.txt".to_string()).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Database not initialized"));
    }
}
