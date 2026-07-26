// src-tauri/src/convert/image_to_text.rs

use std::fs;
use image::GenericImageView;
use exif::{Reader};
use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine};
use serde_json::{json};
use crate::convert::{calculate_conversion_hash, get_app_dir_path_with_hash};

/// Конвертация изображения в текст (Base64 + метаданные)
pub fn convert_image_to_text(path: &str, from: &str, to: &str) -> Result<String, String> {
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

    // 5. Сериализуем в нужный текстовый формат
    let output_text = match to {
        "json" => serde_json::to_string_pretty(&result)
            .map_err(|e| format!("JSON serialize error: {}", e))?,
        "yaml" | "yml" => serde_yaml::to_string(&result)
            .map_err(|e| format!("YAML serialize error: {}", e))?,
        "toml" => toml::to_string_pretty(&result)
            .map_err(|e| format!("TOML serialize error: {}", e))?,
        "xml" => json_to_xml(&result)
            .map_err(|e| format!("XML serialize error: {}", e))?,
        "txt" | "text" => format!("{:#?}", result),
        _ => return Err(format!("Unsupported output format: {}", to)),
    };

    // 6. Сохраняем результат
    let hash = calculate_conversion_hash(path, from, to)
        .map_err(|e| format!("Hash error: {}", e))?;

    let output_path = get_app_dir_path_with_hash(path, to, &hash)?;

    std::fs::write(&output_path, output_text)
        .map_err(|e| format!("Cannot write output file: {}", e))?;

    Ok(output_path)
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

    // Пробуем получить EXIF данные
    if let Ok(exif) = get_exif_data(path) {
        metadata["exif"] = exif;
    }

    Ok(metadata)
}

/// Получение EXIF данных через kamadak-exif
fn get_exif_data(path: &str) -> Result<serde_json::Value, String> {
    let file = fs::File::open(path)
        .map_err(|e| format!("Cannot open file for EXIF: {}", e))?;
    
    let mut bufreader = std::io::BufReader::new(&file);
    let exifreader = Reader::new();
    
    // Читаем EXIF данные
    let exif = exifreader.read_from_container(&mut bufreader)
        .map_err(|e| format!("Cannot read EXIF: {}", e))?;

    let mut exif_map = serde_json::Map::new();

    // Проходим по всем полям
    for field in exif.fields() {
        let tag_name = format!("{:?}", field.tag);
        let value_str = match &field.value {
            exif::Value::Ascii(v) => {
                // v - это &Vec<Vec<u8>>
                if let Some(first) = v.first() {
                    String::from_utf8_lossy(first).to_string()
                } else {
                    String::new()
                }
            }
            exif::Value::Byte(v) => {
                format!("{:?}", v)
            }
            exif::Value::Short(v) => {
                format!("{:?}", v)
            }
            exif::Value::Long(v) => {
                format!("{:?}", v)
            }
            exif::Value::Rational(v) => {
                format!("{:?}", v)
            }
            exif::Value::SRational(v) => {
                format!("{:?}", v)
            }
            exif::Value::Undefined(v, _) => {
                format!("{:?}", v)
            }
            // Добавляем недостающие варианты
            exif::Value::SByte(v) => {
                format!("{:?}", v)
            }
            exif::Value::SShort(v) => {
                format!("{:?}", v)
            }
            exif::Value::SLong(v) => {
                format!("{:?}", v)
            }
            exif::Value::Float(v) => {
                format!("{:?}", v)
            }
            exif::Value::Double(v) => {
                format!("{:?}", v)
            }
            // Если появится что-то еще - используем Debug
            _ => {
                format!("{:?}", field.value)
            }
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

/// Простое преобразование JSON в XML
fn json_to_xml(json: &serde_json::Value) -> Result<String, String> {
    fn json_to_xml_recursive(value: &serde_json::Value, name: &str, depth: usize) -> String {
        let indent = "  ".repeat(depth);
        
        match value {
            serde_json::Value::Object(map) => {
                let mut result = String::new();
                result.push_str(&format!("{}<{}>\n", indent, name));
                for (key, val) in map {
                    result.push_str(&json_to_xml_recursive(val, key, depth + 1));
                }
                result.push_str(&format!("{}</{}>\n", indent, name));
                result
            }
            serde_json::Value::Array(arr) => {
                let mut result = String::new();
                result.push_str(&format!("{}<{}>\n", indent, name));
                for (i, val) in arr.iter().enumerate() {
                    result.push_str(&json_to_xml_recursive(val, &format!("item_{}", i), depth + 1));
                }
                result.push_str(&format!("{}</{}>\n", indent, name));
                result
            }
            serde_json::Value::String(s) => {
                format!("{}<{}>{}</{}>\n", indent, name, s, name)
            }
            serde_json::Value::Number(n) => {
                format!("{}<{}>{}</{}>\n", indent, name, n, name)
            }
            serde_json::Value::Bool(b) => {
                format!("{}<{}>{}</{}>\n", indent, name, b, name)
            }
            serde_json::Value::Null => {
                format!("{}<{} />\n", indent, name)
            }
        }
    }

    Ok(json_to_xml_recursive(json, "image", 0))
}