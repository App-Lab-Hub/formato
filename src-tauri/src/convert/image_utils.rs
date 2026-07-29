// src-tauri/src/convert/image_utils.rs

use std::fs;
use image::GenericImageView;
use exif::Reader;
use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine};
use serde_json::{json, Value as Json};

/// Получение метаданных изображения
pub fn get_image_metadata(path: &str, img: &image::DynamicImage) -> Result<Json, String> {
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
pub fn get_exif_data(path: &str) -> Result<Json, String> {
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
pub fn get_base64_data(path: &str) -> Result<String, String> {
    let bytes = fs::read(path)
        .map_err(|e| format!("Cannot read file for Base64: {}", e))?;
    Ok(BASE64_STANDARD.encode(bytes))
}

/// Получение размера файла
pub fn get_file_size(path: &str) -> Result<u64, String> {
    let metadata = fs::metadata(path)
        .map_err(|e| format!("Cannot get file metadata: {}", e))?;
    Ok(metadata.len())
}