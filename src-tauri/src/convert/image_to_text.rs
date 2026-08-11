// src-tauri/src/convert/image_to_text.rs
use serde_json::json;
use crate::convert::{
    stringify,
    image_utils::{
        get_image_metadata,
        get_base64_data,
    }
};

/// Конвертация изображения в текст (Base64 + метаданные)
pub async fn convert_image_to_text(path: &str, from: &str, to: &str) -> Result<String, String> {
    // 1. Читаем изображение
    let img = image::open(path)
        .map_err(|e| format!("Cannot open image: {}", e))?;
    
    // 2. Получаем метаданные
    let metadata = get_image_metadata(path, &img)?;
    
    // 3. Получаем Base64 представление
    let base64_data = get_base64_data(path)?;
    
    // 4. Собираем всё в JSON
    let result = json!({
        "format": from,
        "metadata": metadata,
        "base64": base64_data,
    });

    // 5. Используем stringify для сериализации и сохранения
    stringify(&result, to, path, from).await
}