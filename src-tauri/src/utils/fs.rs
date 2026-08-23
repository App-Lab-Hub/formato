// src-tauri/src/utils/fs.rs
use tokio::fs;
pub async fn move_file_async(src: &str, dst: &str) -> Result<(), std::io::Error> {
    // Пытаемся быстро переместить (rename)
    if fs::rename(src, dst).await.is_ok() {
        return Ok(());
    }

    // Если rename не сработал — копируем и удаляем
    fs::copy(src, dst).await?;
    fs::remove_file(src).await?;
    Ok(())
}
