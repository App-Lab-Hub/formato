// src-tauri/src/convert/image_to_document.rs

use std::fs;
use image::GenericImageView;
use exif::{Reader};
use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine};
use crate::convert::{calculate_conversion_hash, get_app_dir_path_with_hash};
use serde_json::{json, Value as Json};

/// Конвертация изображения в документ (Base64 + метаданные в DOCX/ODT/PDF/XLSX)
pub fn convert_image_to_document(path: &str, from: &str, to: &str) -> Result<String, String> {
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

    // 5. Сохраняем JSON во временный файл
    let temp_json_path = save_temp_json(&result)?;
    
    // 6. Конвертируем JSON в документ через stringify
    let json_value = serde_json::from_str(&std::fs::read_to_string(&temp_json_path).unwrap())
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;
    
    let output_path = crate::convert::stringify_document(&json_value, &temp_json_path, "json", to)?;
    
    // 7. Перемещаем в нужную директорию с хешем
    let hash = calculate_conversion_hash(path, from, to)
        .map_err(|e| format!("Hash error: {}", e))?;
    let final_path = get_app_dir_path_with_hash(path, to, &hash)?;
    
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
    
    // 8. Удаляем временный JSON
    let _ = std::fs::remove_file(&temp_json_path);
    
    Ok(final_path)
}

/// Сохранение JSON во временный файл
fn save_temp_json(value: &serde_json::Value) -> Result<String, String> {
    let temp_file = tempfile::Builder::new()
        .suffix(".json")
        .prefix("image_data_")
        .tempfile()
        .map_err(|e| format!("Cannot create temp file: {}", e))?;
    
    let temp_path = temp_file.path()
        .to_str()
        .ok_or("Invalid temp path")?
        .to_string();
    
    let content = serde_json::to_string_pretty(value)
        .map_err(|e| format!("JSON serialize error: {}", e))?;
    
    std::fs::write(&temp_path, content)
        .map_err(|e| format!("Cannot write temp file: {}", e))?;
    
    let _ = temp_file.keep();
    
    Ok(temp_path)
}

/// Получение метаданных изображения
fn get_image_metadata(path: &str, img: &image::DynamicImage) -> Result<serde_json::Value, String> {
    let dimensions = img.dimensions();
    let color_type = format!("{:?}", img.color());
    
    let mut metadata = json!({
        "width": dimensions.0,
        "height": dimensions.1,
        "color_type": color_type,
        "file_size": get_file_size(path)?,
    });

    if let Ok(exif) = get_exif_data(path) {
        metadata["exif"] = exif;
    }

    Ok(metadata)
}

/// Получение EXIF данных
fn get_exif_data(path: &str) -> Result<serde_json::Value, String> {
    let file = fs::File::open(path)
        .map_err(|e| format!("Cannot open file for EXIF: {}", e))?;
    
    let mut bufreader = std::io::BufReader::new(&file);
    let exifreader = Reader::new();
    
    let exif = exifreader.read_from_container(&mut bufreader)
        .map_err(|e| format!("Cannot read EXIF: {}", e))?;

    let mut exif_map = serde_json::Map::new();

    for field in exif.fields() {
        let tag_name = format!("{:?}", field.tag);
        let value_str = match &field.value {
            exif::Value::Ascii(v) => {
                if let Some(first) = v.first() {
                    String::from_utf8_lossy(first).to_string()
                } else {
                    String::new()
                }
            }
            exif::Value::Byte(v) => format!("{:?}", v),
            exif::Value::Short(v) => format!("{:?}", v),
            exif::Value::Long(v) => format!("{:?}", v),
            exif::Value::Rational(v) => format!("{:?}", v),
            exif::Value::SRational(v) => format!("{:?}", v),
            exif::Value::Undefined(v, _) => format!("{:?}", v),
            exif::Value::SByte(v) => format!("{:?}", v),
            exif::Value::SShort(v) => format!("{:?}", v),
            exif::Value::SLong(v) => format!("{:?}", v),
            exif::Value::Float(v) => format!("{:?}", v),
            exif::Value::Double(v) => format!("{:?}", v),
            _ => format!("{:?}", field.value),
        };
        exif_map.insert(tag_name, serde_json::Value::String(value_str));
    }

    Ok(serde_json::Value::Object(exif_map))
}

/// Получение Base64 представления изображения
fn get_base64_data(path: &str) -> Result<String, String> {
    let bytes = fs::read(path)
        .map_err(|e| format!("Cannot read file for Base64: {}", e))?;
    Ok(BASE64_STANDARD.encode(bytes))
}

/// Получение размера файла
fn get_file_size(path: &str) -> Result<u64, String> {
    let metadata = fs::metadata(path)
        .map_err(|e| format!("Cannot get file metadata: {}", e))?;
    Ok(metadata.len())
}