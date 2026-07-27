// src-tauri/src/convert/image_to_document.rs

use std::fs;
use image::GenericImageView;
use exif::Reader;
use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine};
use crate::convert::{calculate_conversion_hash, get_app_dir_path_with_hash, stringify_document};
use serde_json::{json, Value as Json};

/// Конвертация изображения в документ
pub fn convert_image_to_document(path: &str, from: &str, to: &str) -> Result<String, String> {
    // 1. Читаем изображение
    let img = image::open(path)
        .map_err(|e| format!("Cannot open image: {}", e))?;
    
    // 2. Получаем метаданные
    let metadata = get_image_metadata(path, &img)?;
    
    // 3. Получаем Base64 представление
    let base64_data = get_base64_data(path)?;
    
    // 4. Формируем читаемый текст напрямую (как в image_to_text, но без JSON)
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

    text.push_str(&format!("\nBase64:\n  \"{}\"", base64_data));

    // 5. Создаем документ через stringify_document
    let output_path = stringify_document(&text, path, from, to)?;

    // 6. Перемещаем в нужную директорию с хешем
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
    
    Ok(final_path)
}

/// Получение метаданных изображения
fn get_image_metadata(path: &str, img: &image::DynamicImage) -> Result<Json, String> {
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
fn get_exif_data(path: &str) -> Result<Json, String> {
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
        exif_map.insert(tag_name, Json::String(value_str));
    }

    Ok(Json::Object(exif_map))
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