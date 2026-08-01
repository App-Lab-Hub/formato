// src-tauri/src/convert/image_to_document.rs
use sea_orm::DatabaseConnection;

use image::GenericImageView;
use crate::convert::{
    calculate_conversion_hash, 
    get_app_dir_path_with_hash, 
    stringify_document,
    image_utils::{
        get_image_metadata,
        get_base64_data,
    }
};

/// Конвертация изображения в документ
pub async fn convert_image_to_document(
    db: &DatabaseConnection, 
    path: &str, 
    from: &str, 
    to: &str
) -> Result<String, String> {
    // 1. Читаем изображение
    let img = image::open(path)
        .map_err(|e| format!("Cannot open image: {}", e))?;
    
    // 2. Получаем метаданные
    let metadata = get_image_metadata(path, &img)?;
    
    // 3. Получаем Base64 представление
    let base64_data = get_base64_data(path)?;
    
    // 4. Формируем читаемый текст
    let mut text = String::new();
    text.push_str(&format!("Image Format: {}\n", from));
    text.push_str(&format!("Width: {} px\n", metadata["width"]));
    text.push_str(&format!("Height: {} px\n", metadata["height"]));
    text.push_str(&format!("Color Type: {}\n", metadata["color_type"]));
    text.push_str(&format!("File Size: {} bytes\n", metadata["file_size"]));

    if let Some(exif) = metadata.get("exif") {
        text.push_str("\nEXIF Data:\n");
        if let Some(obj) = exif.as_object() {
            for (key, value) in obj {
                text.push_str(&format!("  {}: {}\n", key, value));
            }
        }
    }

    text.push_str(&format!("\nBase64: {:#?}", base64_data));

    // 5. Создаем документ через stringify_document
    let output_path = stringify_document(db, &text, path, from, to).await?;

    // 6. Перемещаем в нужную директорию с хешем
    let hash = calculate_conversion_hash(path, from, to)
        .map_err(|e| format!("Hash error: {}", e))?;
    let final_path = get_app_dir_path_with_hash(path, to, &hash, true)?;
    
    if output_path != final_path {
        if let Some(parent) = std::path::Path::new(&final_path).parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("Cannot create output dir: {}", e))?;
            }
        }
        std::fs::rename(&output_path, &final_path)
            .map_err(|e| format!("Cannot move file: {}", e))?;
    }
    
    Ok(final_path)
}